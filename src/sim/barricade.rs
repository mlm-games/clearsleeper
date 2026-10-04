use bevy_ecs::prelude::*;
use glam::Vec3;

use super::flags::Switches;

pub const BARRICADE_ACTORS: [&str; 2] = ["Ksaku", "Dsaku"];

pub const HALVES: usize = 2;
pub const POSTS: usize = 3;

pub const TYPE_STRAW: u8 = 0;
pub const TYPE_WOOD: u8 = 1;

pub const ABSENT: u8 = 0;
pub const WHOLE: u8 = 1;
pub const BURNING: u8 = 2;
pub const BROKEN: u8 = 3;

pub const BURN_FRAMES: i16 = 0x5A;
pub const RELEASE_FRAMES: i16 = 50;
pub const IDLE_FRAMES: i16 = -1;
pub const ALPHA_OUT: i16 = 2;
pub const TOP_LIFT: f32 = 200.0;
pub const POST_Y: f32 = 20.0;
pub const TOP_POST_Y: f32 = 220.0;
pub const POST_SIDE: f32 = 100.0;
pub const STTS_WEIGHT: u8 = 0xFF;
pub const STTS_POWER: u8 = 0xFF;
pub const HEAP: u32 = 0x2280;

pub const STRAW_BREAKERS: u32 = AT_SWORD
    | AT_UNK8
    | AT_BOMB
    | AT_MACHETE
    | AT_UNK800
    | AT_DARKNUT_SWORD
    | AT_MOBLIN_SPEAR
    | AT_SKULL_HAMMER;

pub const WOOD_BREAKERS: u32 = AT_MACHETE | AT_BOMB | AT_UNK800 | AT_DARKNUT_SWORD;

pub const SCORCHERS: u32 = AT_FIRE | AT_UNK20000 | AT_FIRE_ARROW;

pub const AT_SWORD: u32 = 1 << 1;
pub const AT_UNK8: u32 = 1 << 3;
pub const AT_BOMB: u32 = 1 << 5;
pub const AT_FIRE: u32 = 1 << 9;
pub const AT_MACHETE: u32 = 1 << 10;
pub const AT_UNK800: u32 = 1 << 11;
pub const AT_SKULL_HAMMER: u32 = 1 << 16;
pub const AT_UNK20000: u32 = 1 << 17;
pub const AT_FIRE_ARROW: u32 = 1 << 18;
pub const AT_DARKNUT_SWORD: u32 = 1 << 26;
pub const AT_MOBLIN_SPEAR: u32 = 1 << 28;

pub const NO_SWITCH: u8 = 0xFF;

const TYPE_SHIFT: u32 = 4;
const TYPE_MASK: u32 = 0xF;
const SWITCH_SHIFT: u32 = 8;
const TOP_SWITCH_SHIFT: u32 = 16;
const FIELD_MASK: u32 = 0xFF;
const SUB_MASK: u32 = 0xF;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Barricade {
    pub kind: u8,
    pub halves: [u8; HALVES],
    pub counters: [i16; HALVES],
    pub alpha_out: i16,
    pub lit: bool,
    pub hold: i16,
    pub switches: [u8; HALVES],
}

impl Barricade {
    pub fn spawn(parameters: u32, switches: &Switches) -> Self {
        let bottom = ((parameters >> SWITCH_SHIFT) & FIELD_MASK) as u8;
        let top = ((parameters >> TOP_SWITCH_SHIFT) & FIELD_MASK) as u8;
        let paired = parameters & SUB_MASK != 0;
        Self {
            kind: ((parameters >> TYPE_SHIFT) & TYPE_MASK) as u8,
            halves: [
                if bottom == NO_SWITCH || !switches.test(bottom) {
                    WHOLE
                } else {
                    BROKEN
                },
                if !paired {
                    ABSENT
                } else if top == NO_SWITCH || !switches.test(top) {
                    WHOLE
                } else {
                    BROKEN
                },
            ],
            counters: [IDLE_FRAMES; HALVES],
            alpha_out: ALPHA_OUT,
            lit: false,
            hold: 0,
            switches: [bottom, top],
        }
    }

    pub fn paired(&self) -> bool {
        self.halves[1] != ABSENT
    }

    pub fn blocking(&self, half: usize) -> bool {
        self.halves[half] == WHOLE
    }

    pub fn splits(&self, cut: u32) -> bool {
        cut & if self.kind == TYPE_STRAW {
            STRAW_BREAKERS
        } else {
            WOOD_BREAKERS
        } != 0
    }

    pub fn scorches(&self, cut: u32) -> bool {
        cut & SCORCHERS != 0
    }

    pub fn shatter(&mut self, half: usize, switches: &mut Switches) {
        if self.halves[half] != WHOLE {
            return;
        }
        if half == 1 && self.halves[0] == WHOLE {
            self.halves[0] = BROKEN;
            self.counters[0] = 0;
            self.latch(switches, 0);
        }
        self.halves[half] = BROKEN;
        self.counters[half] = 0;
        self.latch(switches, half);
    }

    pub fn scorch(&mut self, switches: &mut Switches) {
        if self.lit {
            return;
        }
        self.lit = true;
        for half in 0..HALVES {
            if self.halves[half] == WHOLE {
                self.halves[half] = BURNING;
                self.counters[half] = RELEASE_FRAMES;
            }
        }
        self.hold = BURN_FRAMES;
        self.latch(switches, 0);
        if self.paired() {
            self.latch(switches, 1);
        }
    }

    fn latch(&self, switches: &mut Switches, half: usize) {
        let no = self.switches[half];
        if no != NO_SWITCH {
            switches.set(no);
        }
    }

    pub fn tick(&mut self) {
        if self.hold > 0 {
            self.hold -= 1;
        }
        for half in 0..HALVES {
            if self.counters[half] >= 0 {
                self.counters[half] -= 1;
            }
        }
    }

    pub fn posts(&self, half: usize) -> [Vec3; POSTS] {
        let lift = if half == 1 { TOP_LIFT } else { 0.0 };
        [
            Vec3::new(0.0, POST_Y + lift, 0.0),
            Vec3::new(-POST_SIDE, POST_Y + lift, 0.0),
            Vec3::new(POST_SIDE, POST_Y + lift, 0.0),
        ]
    }

    pub fn standing(&self) -> f32 {
        match (self.halves[0], self.halves[1]) {
            (WHOLE, WHOLE) => TOP_LIFT + TOP_POST_Y,
            (WHOLE, _) => TOP_POST_Y,
            _ => 0.0,
        }
    }
}

pub fn is_barricade(name: &str) -> bool {
    BARRICADE_ACTORS.contains(&name)
}

pub fn control(mut barricades: Query<&mut Barricade>) {
    for mut barricade in &mut barricades {
        barricade.tick();
    }
}

pub fn strike(
    barricade: &mut Barricade,
    cut: u32,
    half: usize,
    switches: &mut Switches,
) -> bool {
    if barricade.scorches(cut) {
        barricade.scorch(switches);
        return true;
    }
    if !barricade.splits(cut) {
        return false;
    }
    barricade.shatter(half, switches);
    true
}
