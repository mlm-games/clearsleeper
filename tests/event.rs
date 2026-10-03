use clearsleeper::assets;
use clearsleeper::sim::event::{EVENT_LIST_ENTRY, EventList, Value};

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

fn record(name: &str, kind: u32, index: u32, next: i32) -> Vec<u8> {
    let mut out = name32(name);
    out[0x24..0x28].copy_from_slice(&kind.to_be_bytes());
    out[0x28..0x2C].copy_from_slice(&index.to_be_bytes());
    out[0x30..0x34].copy_from_slice(&next.to_be_bytes());
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
    assert_eq!(EVENT_LIST_ENTRY, "event_list.dat");
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
}
