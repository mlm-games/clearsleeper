pub const KILL_POWER: i32 = 2;

pub const PLAYER_STTS: i32 = 120;
pub const SWORD: i32 = 2;
pub const MEGA_SWORD: i32 = 4;
pub const SOUP_SWORD: i32 = 8;
pub const SOUP_MEGA_SWORD: i32 = 8;

pub const BOKO_STICK: i32 = 1;
pub const MACHETE: i32 = 2;
pub const STALFOS_MACE: i32 = 4;
pub const DARKNUT_SWORD: i32 = 4;
pub const MOBLIN_SPEAR: i32 = 4;
pub const PGANON_SWORD: i32 = 4;

pub const BOKO_WEAPONS: [i32; 6] = [
    BOKO_STICK,
    MACHETE,
    STALFOS_MACE,
    DARKNUT_SWORD,
    MOBLIN_SPEAR,
    PGANON_SWORD,
];

pub const ARROW: i32 = 10;
pub const BOMB: i32 = 200;
pub const BOMB2: i32 = 200;
pub const BOMERANG: i32 = 0x3C;
pub const TSUBO: i32 = 3;
pub const WATER_JUG: i32 = 4;
pub const MO2_CAP: i32 = 2;
pub const MT_OVERRIDE: i32 = 4;

pub const CUT_BIT_ARROW: u32 = 0x1000;
pub const CUT_BIT_BOMB: u32 = 0x200;
pub const CUT_BIT_HIMO2: u32 = 0x400;
pub const CUT_BIT_BOMERANG: u32 = 0x800;
pub const CUT_BIT_TSUBO: u32 = 0;

pub fn fatal(power: i32) -> bool {
    power >= KILL_POWER
}

pub fn boko_weapon(kind: u8) -> i32 {
    BOKO_WEAPONS[usize::from(kind) % BOKO_WEAPONS.len()]
}

pub fn sword_power(equipped: bool, soup: bool) -> i32 {
    match (equipped, soup) {
        (true, true) => SOUP_SWORD,
        (true, false) => MEGA_SWORD,
        (false, true) => SOUP_SWORD,
        (false, false) => SWORD,
    }
}

pub fn weapon_power(kind: Option<u8>) -> i32 {
    kind.map_or(BOKO_STICK, boko_weapon)
}