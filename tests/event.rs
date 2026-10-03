use std::time::Duration;

use bevy_ecs::prelude::World;
use bevy_ecs::system::RunSystemOnce;
use clearsleeper::assets;
use clearsleeper::sim::event::{ActiveEvent, EVENT_LIST_ENTRY, EventList, Stage, Value};
use clearsleeper::sim::flags::{EventFlags, StoryBits};
use clearsleeper::sim::transition::{NextStage, Target};
use clearsleeper::{App, SIM_HZ};

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

fn run_control(list: &EventList) -> World {
    let mut world = World::new();
    world.insert_resource(list.clone());
    world.insert_resource(ActiveEvent::default());
    world.insert_resource(EventFlags::default());
    world.insert_resource(StoryBits::default());
    world.insert_resource(NextStage::default());
    assert!(
        world.resource_mut::<ActiveEvent>().order(list, 0),
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
        app.sim.world.resource_mut::<ActiveEvent>().order(&list, 0),
        "the event exists and takes the order"
    );
    assert_eq!(
        active(&app).stage_of(0),
        Some(Stage::Order),
        "and waits for the next tick"
    );
    assert!(
        !app.sim.world.resource_mut::<ActiveEvent>().order(&list, 9),
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
    app.sim.world.resource_mut::<ActiveEvent>().order(&list, 0);
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
    app.sim.world.resource_mut::<ActiveEvent>().order(&list, 0);
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
