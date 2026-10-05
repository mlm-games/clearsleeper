pub const KILL_POWER: i32 = 2;

pub const SWORD: i32 = 2;
pub const MEGA_SWORD: i32 = 4;
pub const SOUP_SWORD: i32 = 4;
pub const SOUP_MEGA_SWORD: i32 = 8;

pub const BOKO_STICK: i32 = 1;
pub const MACHETE: i32 = 2;
pub const STALFOS_MACE: i32 = 4;
pub const DARKNUT_SWORD: i32 = 4;
pub const MOBLIN_SPEAR: i32 = 4;
pub const PGANON_SWORD: i32 = 4;

pub const ITEM_SWORD: u8 = 0x38;

pub const BOKO_WEAPONS: [i32; 6] = [
    BOKO_STICK,
    MACHETE,
    STALFOS_MACE,
    DARKNUT_SWORD,
    MOBLIN_SPEAR,
    PGANON_SWORD,
];

pub const ARROW: i32 = 2;
pub const BOMB: i32 = 4;
pub const BOMERANG: i32 = 1;
pub const TSUBO: i32 = 3;
pub const WATER_JUG: i32 = 4;


pub fn sword_power(item: u8, soup: bool) -> i32 {
    if item == ITEM_SWORD {
        if soup { SOUP_SWORD } else { SWORD }
    } else if is_master_sword(item) {
        if soup { SOUP_MEGA_SWORD } else { MEGA_SWORD }
    } else {
        BOKO_STICK
    }
}

pub const MASTER_SWORD_1: u8 = 0x39;
pub const MASTER_SWORD_2: u8 = 0x3A;
pub const MASTER_SWORD_3: u8 = 0x3E;

pub fn is_master_sword(item: u8) -> bool {
    item == MASTER_SWORD_1 || item == MASTER_SWORD_2 || item == MASTER_SWORD_3
}
