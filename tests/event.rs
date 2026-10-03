use clearsleeper::assets;
use clearsleeper::sim::event::{EVENT_LIST_ENTRY, EventList};

const ORDER_LEN: usize = 0x18;

fn header(count: u32, table: u32) -> Vec<u8> {
    let mut data = vec![0_u8; 0x40];
    data[4..8].copy_from_slice(&count.to_be_bytes());
    data[8..12].copy_from_slice(&table.to_be_bytes());
    for pair in 0..5_u32 {
        let at = 0x0C + pair as usize * 8;
        data[at..at + 4].copy_from_slice(&(count + pair).to_be_bytes());
        data[at + 4..at + 8].copy_from_slice(&(table + 0x40 * (pair + 1)).to_be_bytes());
    }
    data
}

fn order(name: &str, spawn_switch: u8) -> Vec<u8> {
    let mut entry = vec![0_u8; ORDER_LEN];
    entry[..name.len()].copy_from_slice(name.as_bytes());
    entry[ORDER_LEN - 1] = spawn_switch;
    entry
}

fn sample() -> Vec<u8> {
    let mut data = header(3, 0x40);
    data.extend(order("DIRECTOR", 0));
    data.extend(order("", 1));
    data.extend(order("Link", 0xFF));
    data
}

#[test]
fn golden_event_header_names_its_sections() {
    let list = EventList::parse(&sample()).expect("sample parses");
    assert_eq!(list.orders.len(), 3, "one order per count");
    assert_eq!(
        list.section(0).unwrap().count,
        3,
        "the order table is first"
    );
    assert_eq!(list.section(0).unwrap().offset, 0x40);
    assert_eq!(list.section(4).unwrap().count, 7, "five sections follow");
}

#[test]
fn golden_orders_carry_their_anchor_names() {
    let list = EventList::parse(&sample()).expect("sample parses");
    assert_eq!(list.orders[0].name, "DIRECTOR", "named anchors are kept");
    assert_eq!(list.orders[1].name, "", "unnamed steps stay blank");
    assert!(list.finds("Link").is_some(), "and can be found by name");
    assert!(list.finds("Missing").is_none());
}

#[test]
fn golden_orders_carry_their_spawn_switch() {
    let list = EventList::parse(&sample()).expect("sample parses");
    assert_eq!(list.orders[0].spawn_switch, 0);
    assert_eq!(list.orders[1].spawn_switch, 1);
    assert_eq!(
        list.orders[2].spawn_switch, 0xFF,
        "a player-spawn switch of 0xFF means none"
    );
}

#[test]
fn golden_short_files_are_rejected() {
    assert!(
        EventList::parse(&[0; 8]).is_err(),
        "a stub is not an event list"
    );
    let mut truncated = sample();
    truncated.truncate(0x50);
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
fn golden_sea_stage_carries_its_events() {
    let Some(list) = assets::load_event_list(assets::SEA_STAGE) else {
        eprintln!("CS_ASSETS unset; skipping disc-backed event list check");
        return;
    };
    assert!(
        list.orders.len() > 200,
        "the Great Sea script is hundreds of orders: {}",
        list.orders.len()
    );
    assert!(
        list.orders.iter().any(|order| order.name == "DIRECTOR"),
        "and it is anchored to the director"
    );
    assert!(
        list.finds("Link").is_some(),
        "with the player bound into the sequence"
    );
}
