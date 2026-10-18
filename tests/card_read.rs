use clearsleeper::ui_menu_save::{
    CARD_STAT_CREATE, CARD_STAT_WAIT, MemCard, SlotState, blank_save,
};

fn card(dir: &std::path::Path) -> MemCard {
    let mut card = MemCard::new(dir.to_path_buf());
    card.restore();
    card
}

#[test]
fn empty_card_creates_and_offers_new_games() {
    let dir = std::env::temp_dir().join(format!("cs-card-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let card = card(&dir);
    assert_eq!(card.status, CARD_STAT_CREATE);
    for index in 0..3 {
        assert_eq!(card.slot_state[index], SlotState::Absent);
        assert!(card.slots[index].is_none());
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn unreadable_slot_is_corrupt_and_holds_no_save() {
    let dir = std::env::temp_dir().join(format!("cs-card-corrupt-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("slot0.json"), "{ not json").unwrap();
    let card = card(&dir);
    assert_eq!(card.slot_state[0], SlotState::Corrupt);
    assert!(card.slots[0].is_none());
    assert_eq!(card.status, CARD_STAT_CREATE);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn readable_save_ends_the_create_state_and_checksums() {
    let dir = std::env::temp_dir().join(format!("cs-card-save-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    clearsleeper::save::write_slot_in(&dir, 1, &blank_save()).unwrap();
    let card = card(&dir);
    assert_eq!(card.slot_state[1], SlotState::Loaded);
    assert!(card.slots[1].is_some());
    assert_ne!(card.checksum[1], 0);
    assert!(card.test_checksum_game_data(1));
    assert_eq!(card.status, CARD_STAT_WAIT);
    let _ = std::fs::remove_dir_all(&dir);
}
