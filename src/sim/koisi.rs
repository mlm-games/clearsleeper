use bevy_ecs::prelude::*;

use glam::Vec3;

use super::actor::Actor;
use super::collision::Collision;
use super::ground;

pub const KOISI_ACTORS: [&str; 2] = ["koisi1", "koisi2"];

pub const DATAFLG_0: u32 = 0;
pub const DATAFLG_1: u32 = 1 << 0;
pub const DATAFLG_2: u32 = 1 << 1;
pub const DATAFLG_4: u32 = 1 << 2;
pub const DATAFLG_8: u32 = 1 << 3;
pub const DATAFLG_10: u32 = 1 << 4;

pub const SWITCH_FLAG: u32 = DATAFLG_8;
pub const NO_SWITCH: u32 = 0xFF;
pub const SWORD_HIT: i32 = 3;
pub const BOMB_HIT: i32 = 7;
pub const HAMMER_HIT: i32 = 7;
pub const BOMB_EXEMPT_KIND: u8 = 3;
pub const WAVE_SINK_GAIN: f32 = 0.8;
pub const WAVE_RISE_GAIN: f32 = 0.2;
pub const LAVA_SINK_LIMIT: f32 = 20.0;
pub const NO_LAVA: f32 = f32::NEG_INFINITY;

const TYPE_MASK: u32 = 0xF;
const TYPE_SHIFT: u32 = 24;
const MOVE_BG_SHIFT: u32 = 6;
const MOVE_BG_MASK: u32 = 0x3;
const ITEM_NO_MASK: u32 = 0x3F;
const ITEM_SAVE_SHIFT: u32 = 16;
const ITEM_SAVE_MASK: u32 = 0x7F;
const SWITCH_SHIFT: u32 = 8;
const SWITCH_MASK: u32 = 0xFF;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Profile {
    pub gravity: f32,
    pub clearance: f32,
    pub stiffness: u8,
    pub pop: f32,
    pub toss: f32,
    pub toss_gain: f32,
    pub damp: f32,
    pub fall_scale: f32,
    pub shake_power: i16,
    pub sink_gravity: f32,
    pub sink_damp: f32,
    pub sink_fall: f32,
    pub speed_cap: f32,
    pub flags: u32,
    pub break_se: u32,
}

const SMALL: Profile = Profile {
    gravity: -6.0,
    clearance: 15.0,
    stiffness: 0xC8,
    pop: 27.0,
    toss: 36.0,
    toss_gain: 0.0,
    damp: 0.0,
    fall_scale: 0.0,
    shake_power: 0,
    sink_gravity: 5.4,
    sink_damp: 0.04,
    sink_fall: 0.014,
    speed_cap: 30.0,
    flags: DATAFLG_0,
    break_se: 0x692B,
};

const HEAVY: Profile = Profile {
    gravity: -10.0,
    clearance: 15.0,
    stiffness: 0xC8,
    pop: 25.0,
    toss: 23.0,
    toss_gain: 0.0,
    damp: 0.0,
    fall_scale: 0.0,
    shake_power: 2,
    sink_gravity: 6.5,
    sink_damp: 0.04,
    sink_fall: 0.014,
    speed_cap: 30.0,
    flags: DATAFLG_2,
    break_se: 0x692B,
};

const TALL: Profile = Profile {
    gravity: -6.0,
    clearance: 200.0,
    stiffness: 0xFF,
    pop: 27.0,
    toss: 36.0,
    toss_gain: 0.0,
    damp: 0.0,
    fall_scale: 0.0,
    shake_power: 6,
    sink_gravity: 5.4,
    sink_damp: 0.04,
    sink_fall: 0.014,
    speed_cap: 30.0,
    flags: DATAFLG_1 | DATAFLG_2 | DATAFLG_4 | DATAFLG_8,
    break_se: 0x696A,
};

const LONG: Profile = Profile {
    gravity: -6.0,
    clearance: 120.0,
    stiffness: 0xFF,
    pop: 27.0,
    toss: 36.0,
    toss_gain: 0.0,
    damp: 0.0,
    fall_scale: 0.0,
    shake_power: 6,
    sink_gravity: 5.4,
    sink_damp: 0.04,
    sink_fall: 0.014,
    speed_cap: 30.0,
    flags: DATAFLG_1 | DATAFLG_2 | DATAFLG_4 | DATAFLG_8 | DATAFLG_10,
    break_se: 0x6969,
};

const SHORT: Profile = Profile {
    gravity: -6.0,
    clearance: 90.0,
    stiffness: 0xFF,
    pop: 27.0,
    toss: 36.0,
    toss_gain: 0.0,
    damp: 0.0,
    fall_scale: 0.0,
    shake_power: 4,
    sink_gravity: 5.4,
    sink_damp: 0.04,
    sink_fall: 0.014,
    speed_cap: 30.0,
    flags: DATAFLG_1 | DATAFLG_2 | DATAFLG_4 | DATAFLG_8,
    break_se: 0x696B,
};

const PROFILES: [Profile; 5] = [SMALL, HEAVY, TALL, LONG, SHORT];

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Wait,
    Carry,
    Drop,
    Sink,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Koisi {
    pub kind: u8,
    pub item_no: u8,
    pub item_save: u8,
    pub switch_no: u8,
    pub move_bg: bool,
    pub mode: Mode,
    pub speed: f32,
    pub speed_y: f32,
    pub heading: Vec3,
    pub water_y: f32,
    pub previous_water: f32,
    pub in_sea: bool,
    pub broken: bool,
}

impl Koisi {
    pub fn spawn(parameters: u32) -> Self {
        Self {
            kind: ((parameters >> TYPE_SHIFT) & TYPE_MASK) as u8,
            item_no: (parameters & ITEM_NO_MASK) as u8,
            item_save: ((parameters >> ITEM_SAVE_SHIFT) & ITEM_SAVE_MASK) as u8,
            switch_no: ((parameters >> SWITCH_SHIFT) & SWITCH_MASK) as u8,
            move_bg: ((parameters >> MOVE_BG_SHIFT) & MOVE_BG_MASK) == 1,
            mode: Mode::Wait,
            speed: 0.0,
            speed_y: 0.0,
            heading: Vec3::ZERO,
            water_y: NO_LAVA,
            previous_water: NO_LAVA,
            in_sea: false,
            broken: false,
        }
    }

    pub fn profile(&self) -> Profile {
        PROFILES[usize::from(self.kind) % PROFILES.len()]
    }

    pub fn toss(&mut self, forward: Vec3, player_speed: f32) {
        let profile = self.profile();
        self.speed = profile.toss + player_speed * profile.toss_gain;
        self.speed_y = profile.pop;
        self.heading = forward;
        self.mode = Mode::Drop;
    }

    pub fn bomb_breaks(&self) -> bool {
        self.kind != BOMB_EXEMPT_KIND
    }

    pub fn latches_switch(&self) -> bool {
        self.profile().flags & SWITCH_FLAG != 0 && self.switch_no as u32 != NO_SWITCH
    }

    pub fn waves(&self) -> bool {
        self.water_y != NO_LAVA && self.previous_water != NO_LAVA
    }

    pub fn tension(&self) -> f32 {
        if !self.waves() {
            return 0.0;
        }
        let delta = self.water_y - self.previous_water;
        if delta < 0.0 {
            delta * WAVE_SINK_GAIN
        } else {
            delta * WAVE_RISE_GAIN
        }
    }

    pub fn sinks(&self) -> bool {
        self.mode == Mode::Sink
    }
}

pub fn is_koisi(name: &str) -> bool {
    KOISI_ACTORS.contains(&name)
}

pub fn lava_sinks(lava: f32, y: f32, clearance: f32) -> bool {
    lava > y + clearance.min(LAVA_SINK_LIMIT)
}

pub fn control(
    collision: Option<Res<Collision>>,
    wet: Option<Res<super::water::Water>>,
    mut rocks: Query<(&mut Actor, &mut Koisi)>,
) {
    let solid = collision.as_deref();
    let soaked = wet.as_deref().is_some_and(|water| water.in_area);
    for (mut actor, mut rock) in &mut rocks {
        let profile = rock.profile();
        rock.previous_water = rock.water_y;
        rock.water_y = if soaked { actor.pos.y } else { NO_LAVA };
        match rock.mode {
            Mode::Wait => {
                if ground::surface(solid, actor.pos).is_some_and(|floor| actor.pos.y <= floor) {
                    rock.speed = 0.0;
                    rock.speed_y = 0.0;
                    continue;
                }
                rock.speed_y += profile.gravity;
                actor.pos.y += rock.speed_y;
                rock.speed_y *= 1.0 - profile.damp;
            }
            Mode::Carry => {}
            Mode::Drop => {
                rock.speed_y += profile.gravity;
                actor.pos.y += rock.speed_y;
                actor.pos.x += rock.heading.x * rock.speed;
                actor.pos.z += rock.heading.z * rock.speed;
                rock.speed *= 0.9;
                if ground::surface(solid, actor.pos).is_some_and(|floor| actor.pos.y <= floor) {
                    rock.mode = Mode::Wait;
                    rock.speed = 0.0;
                    rock.speed_y = 0.0;
                }
            }
            Mode::Sink => {
                rock.speed_y += profile.gravity + profile.sink_gravity;
                actor.pos.y += rock.speed_y;
                actor.pos.y += rock.tension();
                let length = (rock.speed_y * rock.speed_y + rock.speed * rock.speed).sqrt();
                if length > profile.speed_cap {
                    let scale = profile.speed_cap / length;
                    rock.speed_y *= scale;
                    rock.speed *= scale;
                }
            }
        }
    }
}