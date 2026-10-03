use clearsleeper::sim::flags::StoryBits;
use clearsleeper::sim::layer;

fn sea_room(room: u8, bits: &[u16]) -> u8 {
    let mut stories = StoryBits::default();
    for &bit in bits {
        stories.set(bit);
    }
    layer::no("sea", room, &stories)
}

fn stage(name: &str, bits: &[u16]) -> u8 {
    let mut stories = StoryBits::default();
    for &bit in bits {
        stories.set(bit);
    }
    layer::no(name, 0, &stories)
}

#[test]
fn golden_sea_layer_selects_by_room_and_story_bit() {
    assert_eq!(sea_room(44, &[]), 0);
    assert_eq!(sea_room(44, &[0x0520]), 4);
    assert_eq!(sea_room(44, &[0x0E20]), 2);
    assert_eq!(sea_room(44, &[0x0101]), 9);
    assert_eq!(sea_room(44, &[0x0520, 0x0101]), 4);
    assert_eq!(sea_room(11, &[]), 0);
    assert_eq!(sea_room(11, &[0x2D01]), 4);
    assert_eq!(sea_room(1, &[]), 1);
    assert_eq!(sea_room(1, &[0x1820]), 3);
    assert_eq!(sea_room(13, &[]), 0);
    assert_eq!(sea_room(13, &[0x0520]), 0);
}

#[test]
fn golden_stage_layer_branches_on_story_bits() {
    assert_eq!(stage("A_mori", &[]), 0);
    assert_eq!(stage("A_mori", &[0x0F80]), 2);
    assert_eq!(stage("Asoko", &[0x0520]), 2);
    assert_eq!(stage("M2tower", &[0x2D01]), 2);
    assert_eq!(stage("GanonK", &[]), 8);
    assert_eq!(stage("GanonK", &[0x3B02]), 0);
    assert_eq!(stage("GTower", &[]), 8);
    assert_eq!(stage("GTower", &[0x4002]), 0);
    assert_eq!(stage("M_Dai", &[0x0520]), 0);
}
