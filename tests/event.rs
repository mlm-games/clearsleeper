use std::time::Duration;

use bevy_ecs::prelude::{Entity, World};
use bevy_ecs::system::RunSystemOnce;
use clearsleeper::assets;
use clearsleeper::sim::actor::Actor;
use clearsleeper::sim::event::{
    ActiveEvent, EVENT_LIST_ENTRY, EventList, ItemPartner, LinkAnms, PresentItem, Stage, Value,
};
use clearsleeper::sim::flags::{EventFlags, StoryBits};
use clearsleeper::sim::intent::DoIntent;
use clearsleeper::sim::msg::{
    MsgBox, MsgEntry, MsgTable, MSG_BOX_CLOSING, MSG_BOX_CLOSED, MSG_BOX_OPENING,
    MSG_DESTROYED, MSG_DISPLAYED, MSG_ENDS, MSG_TYPING,
};
use clearsleeper::sim::talk::Talk;
use clearsleeper::sim::transition::{NextStage, Target};
use clearsleeper::{App, SIM_HZ};
use glam::Vec3;

const HEADER_LEN: usize = 0x40;
const STAFF_LEN: usize = 80;
const CUT_LEN: usize = 80;
const DATA_LEN: usize = 64;

fn header(sections: [(u32, u32); 7]) -> Vec<u8> {
    let mut data = vec![0_u8; HEADER_LEN];
    for (index, (top, count)) in sections.iter().enumerate() {
        data[index * 8..index * 8 + 4].copy_from_slice(&top.to_be_bytes());
        data[index * 8 + 4..index * 8 + 8].copy_from_slice(&count.to_be_bytes());
    }
    data
}

fn name32(text: &str) -> Vec<u8> {
    let mut out = vec![0_u8; 32];
    out[..text.len()].copy_from_slice(text.as_bytes());
    out
}

fn staff(name: &str, kind: u32, first_cut: u32) -> Vec<u8> {
    let mut out = name32(name);
    out[0x2C..0x30].copy_from_slice(&kind.to_be_bytes());
    out[0x30..0x34].copy_from_slice(&first_cut.to_be_bytes());
    out
}

fn cut(first_data: u32, next: i32) -> Vec<u8> {
    let mut out = vec![0_u8; CUT_LEN];
    out[0x38..0x3C].copy_from_slice(&first_data.to_be_bytes());
    out[0x3C..0x40].copy_from_slice(&next.to_be_bytes());
    out
}

fn event_record(name: &str, staffs: &[u32], finish: [i32; 3]) -> Vec<u8> {
    let mut out = vec![0_u8; 176];
    let label = name32(name);
    out[..32].copy_from_slice(&label);
    for (slot, index) in staffs.iter().enumerate() {
        let at = 0x2C + slot * 4;
        out[at..at + 4].copy_from_slice(&index.to_be_bytes());
    }
    out[0x7C..0x80].copy_from_slice(&(staffs.len() as i32).to_be_bytes());
    for (slot, flag) in finish.iter().enumerate() {
        let at = 0x88 + slot * 4;
        out[at..at + 4].copy_from_slice(&flag.to_be_bytes());
    }
    out
}

fn named_cut(
    name: &str,
    start_flags: [i32; 3],
    flag_id: i32,
    first_data: u32,
    next: i32,
) -> Vec<u8> {
    let mut out = name32(name);
    out.resize(CUT_LEN, 0);
    for (slot, flag) in start_flags.iter().enumerate() {
        let at = 0x28 + slot * 4;
        out[at..at + 4].copy_from_slice(&flag.to_be_bytes());
    }
    out[0x34..0x38].copy_from_slice(&flag_id.to_be_bytes());
    out[0x38..0x3C].copy_from_slice(&first_data.to_be_bytes());
    out[0x3C..0x40].copy_from_slice(&next.to_be_bytes());
    out
}

fn record(name: &str, kind: u32, index: u32, next: i32) -> Vec<u8> {
    let mut out = name32(name);
    out[0x24..0x28].copy_from_slice(&kind.to_be_bytes());
    out[0x28..0x2C].copy_from_slice(&index.to_be_bytes());
    out[0x30..0x34].copy_from_slice(&next.to_be_bytes());
    out
}

fn assemble(
    events: Vec<Vec<u8>>,
    staffs: Vec<Vec<u8>>,
    cuts: Vec<Vec<u8>>,
    data: Vec<Vec<u8>>,
    floats: Vec<Vec<u8>>,
    ints: Vec<Vec<u8>>,
    text: &[u8],
) -> Vec<u8> {
    let parts = [events, staffs, cuts, data, floats, ints];
    let mut tops = Vec::with_capacity(7);
    let mut body = Vec::new();
    let mut top = HEADER_LEN;
    for part in &parts {
        tops.push((top as u32, part.len() as u32));
        for entry in part {
            body.extend_from_slice(entry);
        }
        top += part.iter().map(Vec::len).sum::<usize>();
    }
    tops.push((top as u32, text.len() as u32));
    let mut out = header([
        tops[0], tops[1], tops[2], tops[3], tops[4], tops[5], tops[6],
    ]);
    out.extend(body);
    out.extend(text);
    out
}

fn sample() -> Vec<u8> {
    let event_top = HEADER_LEN;
    let staff_top = event_top + 176;
    let cut_top = staff_top + STAFF_LEN;
    let data_top = cut_top + CUT_LEN;
    let fdata_top = data_top + 2 * DATA_LEN;
    let idata_top = fdata_top + 4;
    let sdata_top = idata_top + 4;
    let mut data = header([
        (event_top as u32, 1),
        (staff_top as u32, 1),
        (cut_top as u32, 1),
        (data_top as u32, 2),
        (fdata_top as u32, 1),
        (idata_top as u32, 1),
        (sdata_top as u32, 4),
    ]);
    data.extend(vec![0_u8; 176]);
    data.extend(staff("Um1", 0, 0));
    data.extend(cut(0, -1));
    data.extend(record("MsgNo", 3, 0, 1));
    data.extend(record("Wipe", 0, 0, -1));
    data.extend(1.5_f32.to_be_bytes().to_vec());
    data.extend(12201_i32.to_be_bytes().to_vec());
    data.extend(b"name".to_vec());
    data
}

fn tick(app: &mut App) {
    app.advance(Duration::from_secs_f64(1.0 / SIM_HZ));
}

fn boot(list: EventList) -> App {
    let mut app = App::new();
    app.sim.world.insert_resource(list);
    app
}

fn active(app: &App) -> &ActiveEvent {
    app.sim.world.resource::<ActiveEvent>()
}

fn flags(app: &App) -> EventFlags {
    *app.sim.world.resource::<EventFlags>()
}

fn press(app: &mut App, held: bool) {
    app.sim.world.resource_mut::<DoIntent>().0 = held;
}

fn box_status(app: &App) -> u8 {
    app.sim.world.resource::<MsgBox>().status()
}

fn talker(app: &mut App) -> Entity {
    app.sim
        .world
        .spawn((
            Actor {
                name: "So".to_owned(),
                parameters: 0,
                pos: Vec3::new(1000.0, 0.0, 0.0),
                angle: [0; 3],
                set_id: 0,
                room: 0,
            },
            Talk::new(0, 0),
        ))
        .id()
}

fn run_control(list: &EventList) -> World {
    let mut world = World::new();
    world.insert_resource(list.clone());
    world.insert_resource(ActiveEvent::default());
    world.insert_resource(EventFlags::default());
    world.insert_resource(StoryBits::default());
    world.insert_resource(NextStage::default());
    world.insert_resource(ItemPartner::default());
    world.insert_resource(LinkAnms::default());
    world.insert_resource(MsgBox::default());
    world.insert_resource(MsgTable::default());
    assert!(
        world.resource_mut::<ActiveEvent>().order(list, 0, Entity::PLACEHOLDER),
        "the event takes the order"
    );
    world
        .run_system_once(clearsleeper::sim::event::control)
        .expect("the interpreter runs");
    world
}

#[test]
fn golden_header_counts_every_section() {
    let list = EventList::parse(&sample()).expect("sample parses");
    assert_eq!(list.event_count(), 1, "the event table comes first");
    assert_eq!(list.staffs.len(), 1);
    assert_eq!(list.section(3).unwrap().count, 2, "two parameter records");
    assert_eq!(list.section(6).unwrap().count, 4, "and the string blob");
}

#[test]
fn golden_staff_names_and_kinds_are_read() {
    let list = EventList::parse(&sample()).expect("sample parses");
    assert_eq!(list.staffs[0].name, "Um1", "staff names survive parsing");
    assert_eq!(list.staff_of("Um1"), Some(0), "and can be looked up");
    assert_eq!(list.staff_of("Missing"), None);
    assert_eq!(list.staffs[0].kind, 0);
}

#[test]
fn golden_parameters_carry_their_types() {
    let list = EventList::parse(&sample()).expect("sample parses");
    let params = list.params(0);
    assert_eq!(params.len(), 2, "both records are named");
    assert_eq!(params[0], ("MsgNo".to_owned(), Value::Int(12201)));
    assert_eq!(params[1], ("Wipe".to_owned(), Value::Float(1.5)));
}

#[test]
fn golden_numbers_can_be_read_by_key() {
    let list = EventList::parse(&sample()).expect("sample parses");
    assert_eq!(list.number(0, "MsgNo"), Some(12201));
    assert_eq!(list.number(0, "Wipe"), Some(1));
    assert_eq!(list.number(0, "Missing"), None);
    assert_eq!(list.number(9, "MsgNo"), None, "no staff, no answer");
}

#[test]
fn golden_short_files_are_rejected() {
    assert!(
        EventList::parse(&[0; 8]).is_err(),
        "a stub is not an event list"
    );
    let mut truncated = sample();
    truncated.truncate(HEADER_LEN + 200);
    assert!(
        EventList::parse(&truncated).is_err(),
        "and a table that runs off the end is not either"
    );
}

#[test]
fn golden_event_entry_name_matches_the_retail_archive() {
    assert_eq!(EVENT_LIST_ENTRY, "dat/event_list.dat");
}

#[test]
fn golden_great_sea_script_parses_from_the_disc() {
    let Some(list) = assets::load_event_list(assets::SEA_STAGE) else {
        eprintln!("CS_ASSETS unset; skipping disc-backed event list check");
        return;
    };
    assert!(
        list.staffs.len() > 900,
        "the sea script is huge: {}",
        list.staffs.len()
    );
    assert!(list.event_count() > 200, "with hundreds of event orders");
    let um1 = list.staff_of("Um1").expect("the stall staff is present");
    assert_eq!(
        list.number(um1, "MsgNo"),
        Some(12201),
        "and it names the line it speaks"
    );
    let meet = list
        .event_index("SO_1ST_MEET")
        .expect("the soldier's opening scene is scripted");
    let event = list.event(meet).expect("the index resolves");
    assert_eq!(event.staffs.len(), 3, "npc, link and camera");
    assert_eq!(event.finish_flags, [7623, -1, -1]);
    let gannon = list
        .event_index("FROM_GANON")
        .expect("the return from the tower is scripted");
    assert_eq!(list.event(gannon).unwrap().finish_flags, [8824, 8831, -1]);
    assert_eq!(
        list.event_index("END_CHECK"),
        None,
        "no sea event carries that name"
    );
}

#[test]
fn golden_event_records_expose_their_orders() {
    let list = EventList::parse(&assemble(
        vec![
            event_record("First", &[0], [1, -1, -1]),
            event_record("Second", &[], [2, -1, -1]),
        ],
        vec![staff("walker", 0, 0)],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        &[],
    ))
    .expect("fixture parses");
    assert_eq!(list.event_index("First"), Some(0));
    assert_eq!(list.event_index("Second"), Some(1));
    assert_eq!(list.event_index("Missing"), None);
    let first = list.event(0).expect("the index resolves");
    assert_eq!(first.name, "First");
    assert_eq!(first.staffs, vec![0]);
    assert_eq!(first.finish_flags, [1, -1, -1]);
    assert_eq!(list.event(7), None);
}

#[test]
fn golden_order_starts_plays_and_closes() {
    let list = EventList::parse(&assemble(
        vec![event_record("One", &[0], [7, -1, -1])],
        vec![staff("guard", 1, 0)],
        vec![named_cut("zero", [-1, -1, -1], 7, u32::MAX, -1)],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        &[],
    ))
    .expect("fixture parses");
    let mut app = boot(list.clone());
    assert!(
        app.sim.world.resource_mut::<ActiveEvent>().order(&list, 0, Entity::PLACEHOLDER),
        "the event exists and takes the order"
    );
    assert_eq!(
        active(&app).stage_of(0),
        Some(Stage::Order),
        "and waits for the next tick"
    );
    assert!(
        !app.sim.world.resource_mut::<ActiveEvent>().order(&list, 9, Entity::PLACEHOLDER),
        "unknown indices are refused"
    );
    app.sim.world.resource_mut::<EventFlags>().set(100);
    tick(&mut app);
    assert!(active(&app).start_check(0), "one tick moves it into play");
    assert_eq!(
        active(&app).cut_of(0, 0),
        Some(0),
        "the staff starts at its first cut"
    );
    assert!(
        !flags(&app).test(100),
        "starting an event clears the flag space"
    );
    assert!(
        flags(&app).test(7),
        "the all-purpose staff ends its cut every frame"
    );
    tick(&mut app);
    assert!(
        active(&app).end_check(0),
        "the finish flag was set, so the event closes"
    );
    assert!(
        app.sim.world.resource_mut::<ActiveEvent>().end(0),
        "the order tears it down"
    );
    assert_eq!(active(&app).stage_of(0), None);
    assert!(
        !app.sim.world.resource_mut::<ActiveEvent>().end(0),
        "and it is gone"
    );
}

#[test]
fn golden_timekeeper_counts_down_before_the_chain_moves() {
    let list = EventList::parse(&assemble(
        vec![event_record("Tick", &[0], [6, -1, -1])],
        vec![staff("clock", 4, 0)],
        vec![
            named_cut("COUNTDOWN", [-1, -1, -1], 5, 0, 1),
            named_cut("WAIT", [-1, -1, -1], 6, u32::MAX, -1),
        ],
        vec![record("Timer", 3, 0, -1)],
        Vec::new(),
        vec![3_i32.to_be_bytes().to_vec()],
        &[],
    ))
    .expect("fixture parses");
    let mut app = boot(list.clone());
    app.sim.world.resource_mut::<ActiveEvent>().order(&list, 0, Entity::PLACEHOLDER);
    tick(&mut app);
    assert_eq!(active(&app).cut_of(0, 0), Some(0), "counting down");
    assert!(!flags(&app).test(5), "the first frame loads the timer");
    tick(&mut app);
    tick(&mut app);
    assert!(!flags(&app).test(5), "two more frames of countdown");
    tick(&mut app);
    assert!(flags(&app).test(5), "three frames later the wait cut ends");
    tick(&mut app);
    assert_eq!(
        active(&app).cut_of(0, 0),
        Some(1),
        "the chain advances once the flag lands"
    );
    assert!(
        flags(&app).test(6),
        "and the wait cut ends on the same tick"
    );
    tick(&mut app);
    assert!(
        active(&app).end_check(0),
        "the finish flag closes the event"
    );
}

#[test]
fn golden_cut_start_flags_hold_the_chain_until_they_are_set() {
    let list = EventList::parse(&assemble(
        vec![event_record("Gate", &[0], [44, -1, -1])],
        vec![staff("walker", 0, 0)],
        vec![
            named_cut("step", [-1, -1, -1], 8, u32::MAX, 1),
            named_cut("hold", [9, -1, -1], -1, u32::MAX, -1),
        ],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        &[],
    ))
    .expect("fixture parses");
    let mut app = boot(list.clone());
    app.sim.world.resource_mut::<ActiveEvent>().order(&list, 0, Entity::PLACEHOLDER);
    tick(&mut app);
    assert_eq!(
        active(&app).cut_of(0, 0),
        Some(0),
        "the next cut waits on its start flag"
    );
    assert!(!flags(&app).test(8), "nothing has released the chain");
    app.sim.world.resource_mut::<EventFlags>().set(9);
    tick(&mut app);
    assert_eq!(
        active(&app).cut_of(0, 0),
        Some(1),
        "the satisfied start flag moves the chain"
    );
    assert!(
        flags(&app).test(8),
        "and the old cut's flag lands on the way out"
    );
}

#[test]
fn golden_cut_parameters_come_from_the_current_cut() {
    let list = EventList::parse(&assemble(
        vec![event_record("Timed", &[0], [1, -1, -1])],
        vec![staff("clock", 4, 0)],
        vec![
            named_cut("COUNTDOWN", [-1, -1, -1], 2, 0, 1),
            named_cut("WAIT", [-1, -1, -1], 3, 1, -1),
        ],
        vec![record("Timer", 3, 0, 1), record("Rate", 0, 0, -1)],
        vec![0.5_f32.to_be_bytes().to_vec()],
        vec![9_i32.to_be_bytes().to_vec()],
        &[],
    ))
    .expect("fixture parses");
    assert_eq!(list.cut_number(0, "Timer"), Some(9));
    assert_eq!(list.cut_number(0, "Rate"), Some(0), "a float reads as int");
    assert_eq!(list.cut_float(0, "Rate"), Some(0.5));
    assert_eq!(
        list.cut_number(1, "Timer"),
        None,
        "the second cut has its own chain"
    );
    assert_eq!(list.cut_number(9, "Timer"), None, "no cut, no answer");
    assert_eq!(list.cut_number(0, "Missing"), None, "no key, no answer");
}

#[test]
fn golden_cut_strings_come_from_the_text_blob() {
    let list = EventList::parse(&assemble(
        vec![event_record("Warp", &[0], [-1, -1, -1])],
        vec![staff("director", 6, 0)],
        vec![
            named_cut("NEXT", [-1, -1, -1], -1, 0, -1),
            named_cut("NEXT", [-1, -1, -1], -1, 2, -1),
        ],
        vec![
            record("Stage", 4, 0, 1),
            record("Timer", 3, 0, -1),
            record("Stage", 4, 4, -1),
        ],
        Vec::new(),
        vec![9_i32.to_be_bytes().to_vec()],
        b"sea\0island\0",
    ))
    .expect("fixture parses");
    assert_eq!(list.cut_text(0, "Stage"), Some("sea"));
    assert_eq!(
        list.cut_text(1, "Stage"),
        Some("island"),
        "the second cut reads its own slice of the blob"
    );
    assert_eq!(list.cut_text(0, "Timer"), None, "a number is not a string");
    assert_eq!(list.cut_text(0, "Missing"), None, "no key, no answer");
    assert_eq!(list.cut_text(9, "Stage"), None, "no cut, no answer");
}

#[test]
fn golden_director_next_queues_the_stage_change() {
    let list = EventList::parse(&assemble(
        vec![event_record("Warp", &[0], [-1, -1, -1])],
        vec![staff("director", 6, 0)],
        vec![named_cut("NEXT", [-1, -1, -1], -1, 0, -1)],
        vec![
            record("Stage", 4, 0, 1),
            record("StartCode", 3, 0, 2),
            record("RoomNo", 3, 1, -1),
        ],
        Vec::new(),
        vec![2_i32.to_be_bytes().to_vec(), 7_i32.to_be_bytes().to_vec()],
        b"sea\0",
    ))
    .expect("fixture parses");
    let world = run_control(&list);
    let next = world.resource::<NextStage>();
    assert_eq!(
        next.target,
        Some(Target {
            stage: "sea".to_owned(),
            room: 7,
            start: 2,
        }),
        "the director queues the retail stage target"
    );
    assert_eq!(next.exit, None, "and leaves the walk-in exits alone");
    let active = world.resource::<ActiveEvent>();
    assert_eq!(
        active.stage_of(0),
        Some(Stage::Play),
        "the swap has not run yet"
    );
    assert_eq!(
        active.cut_of(0, 0),
        Some(0),
        "the transition cut hangs until the stage changes"
    );
}

#[test]
fn golden_director_next_demands_a_stage_name() {
    let list = EventList::parse(&assemble(
        vec![event_record("Warp", &[0], [-1, -1, -1])],
        vec![staff("director", 6, 0)],
        vec![named_cut("NEXT", [-1, -1, -1], -1, 0, -1)],
        vec![record("StartCode", 3, 0, -1)],
        Vec::new(),
        vec![2_i32.to_be_bytes().to_vec()],
        &[],
    ))
    .expect("fixture parses");
    let world = run_control(&list);
    let next = world.resource::<NextStage>();
    assert_eq!(next.target, None, "no stage name, no transition");
    let active = world.resource::<ActiveEvent>();
    assert_eq!(
        active.stage_of(0),
        Some(Stage::Play),
        "so the event lives on"
    );
    assert_eq!(active.cut_of(0, 0), Some(0), "and the cut hangs");
}

#[test]
fn golden_soldier_wait_counts_down_and_unknown_cuts_end_at_once() {
    let list = EventList::parse(&assemble(
        vec![event_record("Talk", &[0], [8, -1, -1])],
        vec![staff("NpcSo", 0, 0)],
        vec![
            named_cut("WAIT", [-1, -1, -1], 7, 0, 1),
            named_cut("STOP", [-1, -1, -1], 8, u32::MAX, -1),
        ],
        vec![record("Timer", 3, 0, -1)],
        Vec::new(),
        vec![2_i32.to_be_bytes().to_vec()],
        &[],
    ))
    .expect("fixture parses");
    let mut app = boot(list.clone());
    assert!(
        app.sim.world.resource_mut::<ActiveEvent>().order(&list, 0, Entity::PLACEHOLDER),
        "the event takes the order"
    );
    tick(&mut app);
    assert_eq!(
        active(&app).stage_of(0),
        Some(Stage::Play),
        "the event plays"
    );
    assert_eq!(active(&app).cut_of(0, 0), Some(0), "the wait is first");
    assert!(!flags(&app).test(7), "the wait is still counting");
    tick(&mut app);
    assert!(flags(&app).test(7), "the wait ran out");
    assert_eq!(active(&app).cut_of(0, 0), Some(0), "and advances next tick");
    tick(&mut app);
    assert_eq!(active(&app).cut_of(0, 0), Some(1), "the wait passed");
    assert!(flags(&app).test(8), "an unknown soldier cut ends at once");
    tick(&mut app);
    assert!(active(&app).end_check(0), "the event closes");
}

#[test]
fn golden_camera_cuts_wait_out_timers_or_end_at_once() {
    let list = EventList::parse(&assemble(
        vec![event_record("Shot", &[0], [7, -1, -1])],
        vec![staff("CAMERA", 2, 0)],
        vec![
            named_cut("UNITRANS", [-1, -1, -1], 5, 0, 1),
            named_cut("FIXEDFRM", [-1, -1, -1], 6, u32::MAX, 2),
            named_cut("HAXX", [-1, -1, -1], 7, u32::MAX, -1),
        ],
        vec![record("Timer", 3, 0, -1)],
        Vec::new(),
        vec![3_i32.to_be_bytes().to_vec()],
        &[],
    ))
    .expect("fixture parses");
    let mut app = boot(list.clone());
    assert!(
        app.sim.world.resource_mut::<ActiveEvent>().order(&list, 0, Entity::PLACEHOLDER),
        "the event takes the order"
    );
    tick(&mut app);
    assert_eq!(active(&app).cut_of(0, 0), Some(0), "the unit cut is first");
    assert!(!flags(&app).test(5), "the camera counts its frames");
    tick(&mut app);
    tick(&mut app);
    tick(&mut app);
    assert!(flags(&app).test(5), "the unit cut waits out its timer");
    assert_eq!(active(&app).cut_of(0, 0), Some(0), "and advances next tick");
    tick(&mut app);
    assert_eq!(active(&app).cut_of(0, 0), Some(1), "the frame cut follows");
    assert!(flags(&app).test(6), "a frameless cut ends at once");
    tick(&mut app);
    assert_eq!(active(&app).cut_of(0, 0), Some(2), "and moved on");
    assert!(flags(&app).test(7), "an unknown camera cut ends at once");
    tick(&mut app);
    assert!(active(&app).end_check(0), "the event closes");
}

#[test]
fn golden_link_and_ship_cuts_end_or_wait_on_their_locomotion() {
    let list = EventList::parse(&assemble(
        vec![event_record("Sail", &[0, 1], [3, 5, -1])],
        vec![staff("Link", 0, 0), staff("Ship", 0, 2)],
        vec![
            named_cut("006n_talk", [-1, -1, -1], 2, u32::MAX, 1),
            named_cut("003n_dash", [-1, -1, -1], 3, u32::MAX, -1),
            named_cut("05keep", [-1, -1, -1], 4, u32::MAX, 3),
            named_cut("01move", [-1, -1, -1], 5, u32::MAX, -1),
        ],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        &[],
    ))
    .expect("fixture parses");
    let mut app = boot(list.clone());
    assert!(
        app.sim.world.resource_mut::<ActiveEvent>().order(&list, 0, Entity::PLACEHOLDER),
        "the event takes the order"
    );
    tick(&mut app);
    assert_eq!(active(&app).cut_of(0, 0), Some(0), "the talk cut is first");
    assert!(flags(&app).test(2), "the talk cut ends at once");
    assert_eq!(active(&app).cut_of(0, 1), Some(2), "the ship cut is first");
    assert!(flags(&app).test(4), "the ship holds at once");
    tick(&mut app);
    assert_eq!(active(&app).cut_of(0, 0), Some(1), "the talk moved on");
    assert_eq!(active(&app).cut_of(0, 1), Some(3), "the ship moved on");
    for _ in 0..3 {
        tick(&mut app);
    }
    assert_eq!(
        active(&app).cut_of(0, 0),
        Some(1),
        "the dash waits on locomotion"
    );
    assert_eq!(
        active(&app).cut_of(0, 1),
        Some(3),
        "the sail waits on the ship"
    );
    assert!(
        !flags(&app).test(3) && !flags(&app).test(5),
        "neither pending cut ever ends"
    );
    assert_eq!(
        active(&app).stage_of(0),
        Some(Stage::Play),
        "and the event never closes"
    );
}

#[test]
fn golden_link_anim_cut_waits_out_its_animation() {
    let list = EventList::parse(&assemble(
        vec![event_record("Nod", &[0], [3, -1, -1])],
        vec![staff("Link", 0, 0)],
        vec![
            named_cut("036nod", [-1, -1, -1], 2, u32::MAX, 1),
            named_cut("003n_dash", [-1, -1, -1], 3, u32::MAX, -1),
        ],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        &[],
    ))
    .expect("fixture parses");
    let mut app = boot(list.clone());
    assert!(
        app.sim.world.resource_mut::<ActiveEvent>().order(&list, 0, Entity::PLACEHOLDER),
        "the event takes the order"
    );
    for _ in 0..28 {
        tick(&mut app);
    }
    assert!(!flags(&app).test(2), "the nod waits out its 28 frames");
    assert_eq!(active(&app).cut_of(0, 0), Some(0), "still on the nod");
    tick(&mut app);
    assert!(flags(&app).test(2), "the nod ends after its animation");
    tick(&mut app);
    assert_eq!(
        active(&app).cut_of(0, 0),
        Some(1),
        "the cut advances once the animation ran out"
    );
}

#[test]
fn golden_link_anim_cut_reads_frame_counts_from_the_animation_table() {
    let list = EventList::parse(&assemble(
        vec![event_record("Salute", &[0], [3, -1, -1])],
        vec![staff("Link", 0, 0)],
        vec![
            named_cut("019salutet", [-1, -1, -1], 2, u32::MAX, 1),
            named_cut("003n_dash", [-1, -1, -1], 3, u32::MAX, -1),
        ],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        &[],
    ))
    .expect("fixture parses");
    let mut pend = boot(list.clone());
    pend
        .sim
        .world
        .insert_resource(LinkAnms::default());
    assert!(
        pend.sim
            .world
            .resource_mut::<ActiveEvent>()
            .order(&list, 0, Entity::PLACEHOLDER),
        "the event takes the order"
    );
    for _ in 0..40 {
        tick(&mut pend);
    }
    assert!(
        !flags(&pend).test(2),
        "without the table the salute cannot end"
    );
    assert_eq!(active(&pend).cut_of(0, 0), Some(0), "it stays pending");
    let mut app = boot(list.clone());
    app.sim
        .world
        .resource_mut::<LinkAnms>()
        .insert("saltation", 0, 150);
    assert!(
        app.sim.world.resource_mut::<ActiveEvent>().order(&list, 0, Entity::PLACEHOLDER),
        "the event takes the order"
    );
    for _ in 0..150 {
        tick(&mut app);
    }
    assert!(!flags(&app).test(2), "the salute waits out 150 frames");
    tick(&mut app);
    assert!(
        flags(&app).test(2),
        "the salute ends on the frame count from the table"
    );
    tick(&mut app);
    assert_eq!(active(&app).cut_of(0, 0), Some(1), "and moves on");
}

#[test]
fn golden_link_instant_cuts_end_at_once() {
    let list = EventList::parse(&assemble(
        vec![event_record("Wait", &[0], [4, -1, -1])],
        vec![staff("Link", 0, 0)],
        vec![
            named_cut("022t_wait", [-1, -1, -1], 2, u32::MAX, 1),
            named_cut("008e_talk", [-1, -1, -1], 3, u32::MAX, 2),
            named_cut("003n_dash", [-1, -1, -1], 4, u32::MAX, -1),
        ],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        &[],
    ))
    .expect("fixture parses");
    let mut app = boot(list.clone());
    assert!(
        app.sim.world.resource_mut::<ActiveEvent>().order(&list, 0, Entity::PLACEHOLDER),
        "the event takes the order"
    );
    tick(&mut app);
    assert!(flags(&app).test(2), "the wait cut ends at once");
    tick(&mut app);
    assert!(flags(&app).test(3), "the talk cut ends at once too");
    tick(&mut app);
    assert_eq!(active(&app).cut_of(0, 0), Some(2), "then the dash");
    tick(&mut app);
    assert!(
        !flags(&app).test(4),
        "the dash still waits on locomotion"
    );
}

#[test]
fn golden_soldier_turn_and_move_cuts_turn_then_walk_to_the_player() {
    let list = EventList::parse(&assemble(
        vec![event_record("Walk", &[0], [8, -1, -1])],
        vec![staff("NpcSo", 0, 0)],
        vec![
            named_cut("TURN_TO_ACTOR", [-1, -1, -1], 6, u32::MAX, 1),
            named_cut("MOVE_TO_ACTOR", [-1, -1, -1], 7, u32::MAX, 2),
            named_cut("STOP", [-1, -1, -1], 8, u32::MAX, -1),
        ],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        &[],
    ))
    .expect("fixture parses");
    let mut app = boot(list.clone());
    let mut players = app.sim.world.query::<&mut clearsleeper::sim::player::Player>();
    {
        let mut player = players.iter_mut(&mut app.sim.world).next().unwrap();
        player.pos = Vec3::ZERO;
        player.yaw = 0.0;
    }
    let soldier = app
        .sim
        .world
        .spawn(Actor {
            name: "So".to_owned(),
            parameters: 0,
            pos: Vec3::new(200.0, 0.0, 0.0),
            angle: [0; 3],
            set_id: 0,
            room: 0,
        })
        .id();
    app.sim
        .world
        .resource_mut::<ActiveEvent>()
        .order(&list, 0, soldier);
    for _ in 0..400 {
        if flags(&app).test(8) {
            break;
        }
        tick(&mut app);
    }
    assert!(flags(&app).test(7), "the walk cut reached the player");
    assert!(flags(&app).test(8), "and the chain finished");
    let actor = app
        .sim
        .world
        .get::<Actor>(soldier)
        .expect("the soldier lives");
    assert!(
        actor.pos.x < 6.0 && actor.pos.x > -1.0,
        "the soldier stopped next to the player at {}",
        actor.pos.x
    );
    assert_eq!(
        actor.angle[1], -16384,
        "the turn ended facing the player"
    );
    tick(&mut app);
    assert!(active(&app).end_check(0), "the event closes");
}

#[test]
fn golden_soldier_offset_and_mapopen_cuts_resolve_through_the_ship() {
    let list = EventList::parse(&assemble(
        vec![event_record("Vanish", &[0], [9, -1, -1])],
        vec![staff("NpcSo", 0, 0)],
        vec![
            named_cut("APPEAR", [-1, -1, -1], 6, u32::MAX, 1),
            named_cut("DIVE", [-1, -1, -1], 7, u32::MAX, 2),
            named_cut("JUMP_MAPOPEN", [-1, -1, -1], 8, u32::MAX, 3),
            named_cut("STOP", [-1, -1, -1], 9, u32::MAX, -1),
        ],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        &[],
    ))
    .expect("fixture parses");
    let mut app = boot(list.clone());
    let stale = {
        let mut query = app.sim.world.query::<(Entity, &Actor)>();
        query
            .iter(&app.sim.world)
            .filter(|(_, actor)| actor.name == "Ship")
            .map(|(entity, _)| entity)
            .collect::<Vec<_>>()
    };
    for entity in stale {
        let _ = app.sim.world.despawn(entity);
    }
    app.sim.world.spawn(Actor {
        name: "Ship".to_owned(),
        parameters: 0,
        pos: Vec3::new(220.0, 0.0, 0.0),
        angle: [0; 3],
        set_id: 0,
        room: 0,
    });
    let soldier = app
        .sim
        .world
        .spawn(Actor {
            name: "So".to_owned(),
            parameters: 0,
            pos: Vec3::new(100.0, 0.0, 0.0),
            angle: [0; 3],
            set_id: 0,
            room: 0,
        })
        .id();
    app.sim
        .world
        .resource_mut::<ActiveEvent>()
        .order(&list, 0, soldier);
    for _ in 0..300 {
        if flags(&app).test(9) {
            break;
        }
        tick(&mut app);
    }
    assert!(flags(&app).test(6), "the rise finishes converging");
    assert!(flags(&app).test(7), "the dive follows it down");
    assert!(flags(&app).test(8), "the chase reaches the ship");
    assert!(flags(&app).test(9), "and the chain finished");
    let actor = app
        .sim
        .world
        .get::<Actor>(soldier)
        .expect("the soldier lives");
    assert!(
        (actor.pos.x + 80.0).abs() < 0.01,
        "the chase ended at the ship's far side at {}",
        actor.pos.x
    );
    tick(&mut app);
    assert!(active(&app).end_check(0), "the event closes");
}

#[test]
fn golden_talk_msg_drives_one_message_through_open_type_and_close() {
    let list = EventList::parse(&assemble(
        vec![event_record("Talk", &[0], [8, -1, -1])],
        vec![staff("NpcSo", 0, 0)],
        vec![
            named_cut("TALK_MSG", [-1, -1, -1], 7, 0, 1),
            named_cut("STOP", [-1, -1, -1], 8, u32::MAX, -1),
        ],
        vec![record("msg_num", 3, 0, -1)],
        Vec::new(),
        vec![4242_i32.to_be_bytes().to_vec()],
        &[],
    ))
    .expect("fixture parses");
    let mut app = boot(list.clone());
    app.sim.world.resource_mut::<MsgTable>().insert(
        4242,
        MsgEntry {
            text: b"Hi\x00".to_vec(),
            draw_type: 0,
            textbox_type: 0,
            lines: 4,
        },
    );
    let soldier = talker(&mut app);
    app.sim
        .world
        .resource_mut::<ActiveEvent>()
        .order(&list, 0, soldier);
    tick(&mut app);
    assert_eq!(
        box_status(&app),
        MSG_BOX_OPENING,
        "the create frame opens the box"
    );
    let mut opened = 0;
    while box_status(&app) != MSG_TYPING {
        tick(&mut app);
        opened += 1;
        assert!(opened < 40, "the box starts typing");
    }
    assert_eq!(opened, 14, "thirteen open frames then typing");
    press(&mut app, true);
    tick(&mut app);
    assert_eq!(
        box_status(&app),
        MSG_DISPLAYED,
        "one press finishes the message"
    );
    press(&mut app, false);
    tick(&mut app);
    assert_eq!(box_status(&app), MSG_ENDS, "the driver marks it for closing");
    press(&mut app, true);
    tick(&mut app);
    assert_eq!(
        box_status(&app),
        MSG_BOX_CLOSING,
        "the second press closes it"
    );
    press(&mut app, false);
    let mut closed = 0;
    while box_status(&app) != MSG_BOX_CLOSED {
        tick(&mut app);
        closed += 1;
        assert!(closed < 20, "the box closes");
    }
    assert_eq!(closed, 10, "closing takes ten frames");
    tick(&mut app);
    assert_eq!(
        box_status(&app),
        MSG_DESTROYED,
        "the driver destroys the box"
    );
    assert!(flags(&app).test(7), "the talk cut ended with the box");
    let mut done = 0;
    while !active(&app).end_check(0) {
        tick(&mut app);
        done += 1;
        assert!(done < 20, "the event closes");
    }
}

#[test]
fn golden_get_item_cut_presents_its_partner_from_the_cut_parameter() {
    let list = EventList::parse(&assemble(
        vec![event_record("Rupe", &[0], [2, -1, -1])],
        vec![staff("Link", 0, 0)],
        vec![named_cut("011get_item", [-1, -1, -1], 2, 0, -1)],
        vec![record("prm0", 3, 0, -1)],
        Vec::new(),
        vec![15_i32.to_be_bytes().to_vec()],
        &[],
    ))
    .expect("fixture parses");
    let mut app = boot(list.clone());
    app.sim.world.resource_mut::<MsgTable>().insert(
        116,
        MsgEntry {
            text: b"Green rupee\x00".to_vec(),
            draw_type: 0,
            textbox_type: 0,
            lines: 4,
        },
    );
    assert!(app
        .sim
        .world
        .resource_mut::<ActiveEvent>()
        .order(&list, 0, Entity::PLACEHOLDER));
    tick(&mut app);
    let partner = *app.sim.world.resource::<ItemPartner>();
    let entity = partner
        .entity
        .expect("the cut spawns its present partner");
    assert_eq!(partner.item, 15, "the cut parameter names the item");
    let present = app
        .sim
        .world
        .get::<PresentItem>(entity)
        .expect("the partner is a present item");
    assert_eq!(present.item, 15, "the partner carries the item");
    assert!(present.hidden, "the partner stays hidden through the animation");
    let mut opened = 0;
    while !app.sim.world.resource::<MsgBox>().active() {
        tick(&mut app);
        opened += 1;
        assert!(opened < 60, "the cut opens the message after the animation");
    }
    let mut typed = 0;
    while box_status(&app) != MSG_TYPING {
        tick(&mut app);
        typed += 1;
        assert!(typed < 40, "the box starts typing");
    }
    assert!(
        !app.sim
            .world
            .get::<PresentItem>(entity)
            .expect("the partner lives with its message")
            .hidden,
        "the partner shows itself with the message"
    );
    assert_eq!(
        app.sim.world.resource::<MsgBox>().message_no(),
        116,
        "the message comes from the item number"
    );
    press(&mut app, true);
    tick(&mut app);
    assert_eq!(
        box_status(&app),
        MSG_DISPLAYED,
        "one press finishes the message"
    );
    press(&mut app, false);
    tick(&mut app);
    assert_eq!(box_status(&app), MSG_ENDS, "the driver marks it for closing");
    press(&mut app, true);
    tick(&mut app);
    assert_eq!(
        box_status(&app),
        MSG_BOX_CLOSING,
        "the second press closes it"
    );
    press(&mut app, false);
    let mut closed = 0;
    while box_status(&app) != MSG_BOX_CLOSED {
        tick(&mut app);
        closed += 1;
        assert!(closed < 20, "the box closes");
    }
    tick(&mut app);
    assert_eq!(
        box_status(&app),
        MSG_DESTROYED,
        "the driver destroys the box"
    );
    assert!(flags(&app).test(2), "the get item cut ended with the box");
    assert!(
        app.sim.world.resource::<ItemPartner>().entity.is_none(),
        "the partner dies with the box"
    );
    assert!(
        app.sim.world.get::<PresentItem>(entity).is_none(),
        "the partner entity is gone"
    );
    let mut done = 0;
    while !active(&app).end_check(0) {
        tick(&mut app);
        done += 1;
        assert!(done < 20, "the event closes");
    }
}

#[test]
fn golden_get_item_cut_waits_for_an_outside_partner() {
    let list = EventList::parse(&assemble(
        vec![event_record("Chest", &[0], [2, -1, -1])],
        vec![staff("Link", 0, 0)],
        vec![named_cut("011get_item", [-1, -1, -1], 2, 0, -1)],
        vec![record("prm0", 3, 0, -1)],
        Vec::new(),
        vec![0_i32.to_be_bytes().to_vec()],
        &[],
    ))
    .expect("fixture parses");
    let mut app = boot(list.clone());
    assert!(app
        .sim
        .world
        .resource_mut::<ActiveEvent>()
        .order(&list, 0, Entity::PLACEHOLDER));
    for _ in 0..40 {
        tick(&mut app);
    }
    assert!(
        app.sim.world.resource::<ItemPartner>().entity.is_none(),
        "a zero parameter spawns nothing of its own"
    );
    assert!(
        !app.sim.world.resource::<MsgBox>().active(),
        "the cut holds its message for the outside partner"
    );
    assert!(!flags(&app).test(2), "the cut stays inside");
}

#[test]
fn golden_talk_msg_ends_the_cut_when_current_message_is_the_end() {
    let list = EventList::parse(&assemble(
        vec![event_record("Talk", &[0], [8, -1, -1])],
        vec![staff("NpcSo", 0, 0)],
        vec![
            named_cut("TALK_MSG", [-1, -1, -1], 7, 0, 1),
            named_cut("STOP", [-1, -1, -1], 8, u32::MAX, -1),
        ],
        vec![
            record("msg_num", 3, 0, 1),
            record("end_msg", 3, 1, -1),
        ],
        Vec::new(),
        vec![
            4242_i32.to_be_bytes().to_vec(),
            4242_i32.to_be_bytes().to_vec(),
        ],
        &[],
    ))
    .expect("fixture parses");
    let mut app = boot(list.clone());
    app.sim.world.resource_mut::<MsgTable>().insert(
        4242,
        MsgEntry {
            text: b"Hi\x00".to_vec(),
            draw_type: 0,
            textbox_type: 0,
            lines: 4,
        },
    );
    let soldier = talker(&mut app);
    app.sim
        .world
        .resource_mut::<ActiveEvent>()
        .order(&list, 0, soldier);
    for _ in 0..20 {
        if flags(&app).test(7) {
            break;
        }
        tick(&mut app);
    }
    assert!(
        flags(&app).test(7),
        "the talk cut ends when current is the end message"
    );
    assert!(
        app.sim.world.resource::<MsgBox>().active(),
        "the box outlives the cut"
    );
    let mut done = 0;
    while !active(&app).end_check(0) {
        tick(&mut app);
        done += 1;
        assert!(done < 20, "the event closes");
    }
}

#[test]
fn golden_message_staff_creates_pushes_and_finishes_the_box() {
    let list = EventList::parse(&assemble(
        vec![event_record("Box", &[0], [9, -1, -1])],
        vec![staff("Msg", 7, 0)],
        vec![
            named_cut("CREATE_MSG", [-1, -1, -1], 7, 0, 1),
            named_cut("PUSHBUTTON", [-1, -1, -1], 8, u32::MAX, 2),
            named_cut("FINISH", [-1, -1, -1], 9, u32::MAX, -1),
        ],
        vec![record("msgNo", 3, 0, -1)],
        Vec::new(),
        vec![500_i32.to_be_bytes().to_vec()],
        &[],
    ))
    .expect("fixture parses");
    let mut app = boot(list.clone());
    app.sim.world.resource_mut::<MsgTable>().insert(
        500,
        MsgEntry {
            text: b"OK\x00".to_vec(),
            draw_type: 0,
            textbox_type: 0,
            lines: 4,
        },
    );
    app.sim
        .world
        .resource_mut::<ActiveEvent>()
        .order(&list, 0, Entity::PLACEHOLDER);
    tick(&mut app);
    assert_eq!(
        box_status(&app),
        MSG_BOX_OPENING,
        "CREATE_MSG opens the box"
    );
    tick(&mut app);
    tick(&mut app);
    assert!(flags(&app).test(7), "CREATE_MSG ends after the resolve frames");
    let mut typed = 0;
    while box_status(&app) != MSG_TYPING {
        tick(&mut app);
        typed += 1;
        assert!(typed < 40, "the box starts typing");
    }
    press(&mut app, true);
    tick(&mut app);
    assert_eq!(
        box_status(&app),
        MSG_DISPLAYED,
        "the press finishes the message"
    );
    press(&mut app, false);
    tick(&mut app);
    assert!(
        flags(&app).test(8),
        "PUSHBUTTON waits for the message then advances"
    );
    tick(&mut app);
    assert_eq!(
        box_status(&app),
        MSG_ENDS,
        "FINISH marks the box for closing"
    );
    press(&mut app, true);
    tick(&mut app);
    assert_eq!(box_status(&app), MSG_BOX_CLOSING, "the press closes it");
    press(&mut app, false);
    let mut closed = 0;
    while box_status(&app) != MSG_BOX_CLOSED {
        tick(&mut app);
        closed += 1;
        assert!(closed < 20, "the box closes");
    }
    assert_eq!(closed, 10, "closing takes ten frames");
    tick(&mut app);
    assert_eq!(
        box_status(&app),
        MSG_DESTROYED,
        "FINISH destroys the box"
    );
    assert!(flags(&app).test(9), "the chain finishes");
    let mut done = 0;
    while !active(&app).end_check(0) {
        tick(&mut app);
        done += 1;
        assert!(done < 20, "the event closes");
    }
}
