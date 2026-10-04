use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::stream;

pub const BARREL2_ACTOR: &str = "Ktarur";

pub const MODE_AFL: u8 = 0;
pub const MODE_EXIT_V: u8 = 1;
pub const MODE_EXIT_H: u8 = 2;
pub const MODE_EXIT_MINE: u8 = 3;
pub const MODE_EXPLODE: u8 = 4;
pub const MODE_DEMO_BREAK0: u8 = 5;
pub const MODE_DEMO_BREAK1: u8 = 6;
pub const MODE_DEMO_EXPLODE0: u8 = 7;
pub const MODE_DEMO_EXPLODE1: u8 = 8;

pub const TYPES: usize = 4;
pub const KIND_MINE: usize = 2;

pub const HEAP: [u32; TYPES] = [0x520, 0x520, 0x880, 0x520];
pub const COLLIDE_R: [f32; TYPES] = [200.0, 200.0, 288.0, 288.0];
pub const COLLIDE_H: [f32; TYPES] = [300.0, 400.0, 432.0, 576.0];
pub const CULL_DRAW: [f32; TYPES] = [75.0, 75.0, 375.0, 75.0];
pub const CULL_MOVE: [f32; TYPES] = [800.0, 300.0, 4500.0, 800.0];
pub const BUOY_R: [f32; TYPES] = [50.0; TYPES];
pub const DRIFT: [f32; TYPES] = [100.0; TYPES];
pub const DEPTH: [f32; TYPES] = [0.55, 0.75, 0.3, 0.55];
pub const DRAFT: [f32; TYPES] = [0.5; TYPES];
pub const SCALE: [f32; TYPES] = [1.0; TYPES];
pub const HEIGHT: [f32; TYPES] = [4.0, 4.0, 5.76, 5.76];
pub const TRIP: [f32; TYPES] = [500.0, 450.0, 500.0, 500.0];
pub const REACH: [f32; TYPES] = [300.0; TYPES];
pub const SHIP_SPEED: [f32; TYPES] = [25.0; TYPES];
pub const CRASH_DEPTH: [f32; TYPES] = [0.0, 0.0, 400.0, 0.0];
pub const FLASH_REACH: [f32; TYPES] = [0.0, 0.0, 700.0, 0.0];
pub const BLOW_REACH: [f32; TYPES] = [0.0, 0.0, 900.0, 0.0];
pub const PHASE_STEP: [f32; TYPES] = [1000.0; TYPES];
pub const LEAN_SLOPE: [f32; TYPES] = [0.35; TYPES];
pub const LEAN_LIMIT: [f32; TYPES] = [0.3; TYPES];
pub const LEAN_PUSH: [f32; TYPES] = [0.02; TYPES];
pub const LEAN_DAMP: [f32; TYPES] = [0.04; TYPES];

pub const COMING_DAMP: f32 = 0.92;
pub const COMING_DECAY_HIGH: f32 = 0.02;
pub const SETTLED_DAMP: f32 = 0.94;
pub const SETTLED_DECAY: f32 = 0.01;
pub const COMING_CAP: f32 = 0.6;
pub const SETTLED_CAP: f32 = 0.2;
pub const DRIFT_PUSH: f32 = -0.002;
pub const DRIFT_DECAY: f32 = 0.9;
pub const EXIT_STEP: f32 = 5.0;
pub const EXIT_DAMP_V: f32 = 0.96;
pub const EXIT_DAMP_H: f32 = 0.94;
pub const EXIT_SEED: f32 = 30.0;
pub const EXIT_SEED_MINE: f32 = 40.0;
pub const MINE_SINK: f32 = 50.0;
pub const COMING_SINK: f32 = 50.0;
pub const EXPLODE_FRAMES: i32 = 0x1E;
pub const CLIP_STEP: f32 = 8.0;
pub const CLIP_FLOOR: f32 = -30.0;
pub const CLIP_SEED: f32 = -60.0;
pub const CLIP_COOLDOWN: u8 = 0x14;

pub const TRIGGER_NONE: u8 = 0;
pub const TRIGGER_CRASH: u8 = 1;
pub const TRIGGER_TOUCH: u8 = 2;
pub const TRIGGER_BOMB: u8 = 3;

pub const NO_ITEM_SAVE: u8 = 0x7F;

const TYPE_SHIFT: u32 = 24;
const TYPE_MASK: u32 = 0x3;
const ITEM_MASK: u32 = 0x3F;
const BUOY_SHIFT: u32 = 8;
const TEXTURE_SHIFT: u32 = 10;
const SAVE_SHIFT: u32 = 16;
const SAVE_MASK: u32 = 0x7F;
const COMING_SHIFT: u32 = 28;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Afloat,
    ExitV,
    ExitH,
    ExitMine,
    Explode,
    DemoBreak0,
    DemoBreak1,
    DemoExplode0,
    DemoExplode1,
}

impl Mode {
    pub fn code(self) -> u8 {
        match self {
            Self::Afloat => MODE_AFL,
            Self::ExitV => MODE_EXIT_V,
            Self::ExitH => MODE_EXIT_H,
            Self::ExitMine => MODE_EXIT_MINE,
            Self::Explode => MODE_EXPLODE,
            Self::DemoBreak0 => MODE_DEMO_BREAK0,
            Self::DemoBreak1 => MODE_DEMO_BREAK1,
            Self::DemoExplode0 => MODE_DEMO_EXPLODE0,
            Self::DemoExplode1 => MODE_DEMO_EXPLODE1,
        }
    }

    pub fn demo(self) -> bool {
        matches!(
            self,
            Self::DemoBreak0 | Self::DemoBreak1 | Self::DemoExplode0 | Self::DemoExplode1
        )
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct BuoyBarrel {
    pub mode: Mode,
    pub kind: usize,
    pub item_no: u8,
    pub item_save: u8,
    pub has_buoy: bool,
    pub texture: u8,
    pub coming: bool,
    pub heave: f32,
    pub heave_vel: f32,
    pub phase: i32,
    pub water: f32,
    pub drift_x: f32,
    pub drift_z: f32,
    pub lean: Vec3,
    pub lean_vel: Vec3,
    pub wave: Vec3,
    pub timer: i32,
    pub trigger: u8,
    pub cooldown: u8,
    pub flashing: bool,
    pub exit_request: bool,
    pub given: bool,
    pub gone: bool,
    pub seed: u32,
}

impl BuoyBarrel {
    pub fn spawn(parameters: u32) -> Self {
        let kind = ((parameters >> TYPE_SHIFT) & TYPE_MASK) as usize;
        let coming = parameters >> COMING_SHIFT & 1 != 0;
        let jitter = if coming && kind != KIND_MINE { 1.0 } else { 0.0 };
        Self {
            mode: Mode::Afloat,
            kind,
            item_no: (parameters & ITEM_MASK) as u8,
            item_save: ((parameters >> SAVE_SHIFT) & SAVE_MASK) as u8,
            has_buoy: parameters >> BUOY_SHIFT & 1 == 0,
            texture: (parameters >> TEXTURE_SHIFT) as u8 & 1,
            coming,
            heave: if coming {
                -(BUOY_R[kind] * HEIGHT[kind] * SCALE[kind]) - COMING_SINK
            } else {
                0.0
            },
            heave_vel: 0.0,
            phase: 0,
            water: 0.0,
            drift_x: 0.0,
            drift_z: 0.0,
            lean: Vec3::new(jitter, 0.0, jitter),
            lean_vel: Vec3::ZERO,
            wave: Vec3::ZERO,
            timer: 0,
            trigger: TRIGGER_NONE,
            cooldown: 0,
            flashing: false,
            exit_request: false,
            given: false,
            gone: false,
            seed: 0x2540_F491,
        }
    }

    pub fn hull(&self) -> (f32, f32) {
        (COLLIDE_H[self.kind], COLLIDE_R[self.kind])
    }

    pub fn draw_cull(&self) -> f32 {
        SCALE[self.kind] * CULL_DRAW[self.kind]
    }

    pub fn move_cull(&self) -> f32 {
        CULL_MOVE[self.kind]
    }

    pub fn cap(&self) -> f32 {
        DRIFT[self.kind] * if self.coming { COMING_CAP } else { SETTLED_CAP }
    }

    pub fn spring(&mut self, decay: f32, damp: f32, cap: f32) {
        self.heave_vel += self.heave * -decay;
        self.heave_vel *= damp;
        self.heave += self.heave_vel;
        let limit = DRIFT[self.kind] * cap;
        if self.heave > limit {
            self.heave = limit;
        }
    }

    pub fn bob(&mut self) {
        self.phase += (self.rand() * PHASE_STEP[self.kind] * 2.0) as i32;
    }

    pub fn float_step(&mut self) {
        if self.cooldown > 0 {
            self.cooldown -= 1;
        }
        let (decay, damp) = if self.coming {
            if self.heave > 0.0 {
                (COMING_DECAY_HIGH, COMING_DAMP)
            } else {
                (SETTLED_DECAY, COMING_DAMP)
            }
        } else {
            (SETTLED_DECAY, SETTLED_DAMP)
        };
        self.spring(decay, damp, if self.coming { COMING_CAP } else { SETTLED_CAP });
        self.bob();
        self.drift_x = (self.drift_x + DRIFT_PUSH) * DRIFT_DECAY;
        self.drift_z = (self.drift_z + DRIFT_PUSH) * DRIFT_DECAY;
        self.lean_step();
    }

    pub fn lean_step(&mut self) {
        let raw = self.wave * LEAN_SLOPE[self.kind];
        let square = raw.x * raw.x + raw.z * raw.z;
        let limit = LEAN_LIMIT[self.kind];
        let push = if square > limit * limit {
            raw * (limit / square.sqrt())
        } else {
            raw
        };
        let gap = Vec3::new(self.lean.x - push.x, 0.0, self.lean.z - push.z);
        self.lean_vel.x -= gap.x * LEAN_PUSH[self.kind] + self.lean_vel.x * LEAN_DAMP[self.kind];
        self.lean_vel.z -= gap.z * LEAN_PUSH[self.kind] + self.lean_vel.z * LEAN_DAMP[self.kind];
        self.lean.x += self.lean_vel.x;
        self.lean.z += self.lean_vel.z;
    }

    pub fn sink_step(&mut self, damp: f32) {
        self.heave_vel -= EXIT_STEP;
        self.heave_vel *= damp;
        self.heave += self.heave_vel;
    }

    pub fn sink_depth(&self) -> f32 {
        -(BUOY_R[self.kind] * HEIGHT[self.kind] * SCALE[self.kind])
    }

    pub fn sunk(&self, depth: f32) -> bool {
        self.heave <= self.sink_depth() - depth
    }

    pub fn clip_step(&mut self) {
        if self.heave_vel > 0.0 {
            self.heave_vel = 0.0;
        }
        self.heave_vel -= CLIP_STEP;
        if self.heave_vel < CLIP_FLOOR {
            self.heave_vel = CLIP_FLOOR;
        }
    }

    pub fn explode_step(&mut self) {
        self.spring(SETTLED_DECAY, SETTLED_DAMP, SETTLED_CAP);
        self.bob();
        self.timer -= 1;
        if self.timer < 0 {
            self.gone = true;
        }
    }

    pub fn height(&self) -> f32 {
        self.water
            + self.heave
            + (DEPTH[self.kind] * -DRIFT[self.kind]
                + DRAFT[self.kind] * stream::sin_s(self.phase as i16))
                * HEIGHT[self.kind]
                * SCALE[self.kind]
    }

    pub fn enter(&mut self, mode: Mode) {
        self.mode = mode;
        match mode {
            Mode::ExitV | Mode::ExitH => self.heave_vel = EXIT_SEED,
            Mode::ExitMine => self.heave_vel = EXIT_SEED_MINE,
            Mode::Explode => self.timer = EXPLODE_FRAMES,
            _ => {}
        }
    }

    pub fn bomb(&mut self) {
        self.trigger = TRIGGER_BOMB;
    }

    pub fn crashed(&mut self) {
        self.trigger = TRIGGER_CRASH;
    }

    pub fn touched(&mut self) {
        self.trigger = TRIGGER_TOUCH;
    }

    pub fn fast_enough(&self, ship_speed: f32) -> bool {
        ship_speed > SHIP_SPEED[self.kind]
    }

    pub fn head_on(&self, gap: Vec3, heading: Vec3) -> bool {
        (gap.x * heading.x + gap.z * heading.z) < CRASH_DEPTH[self.kind] * CRASH_DEPTH[self.kind]
    }

    pub fn clip(&mut self) {
        if self.cooldown == 0 {
            self.heave_vel = CLIP_SEED;
            self.cooldown = CLIP_COOLDOWN;
        }
    }

    pub fn within(&self, range: f32, gap: Vec3) -> bool {
        gap.x * gap.x + gap.z * gap.z < range * range
    }

    pub fn flash_range(&self) -> f32 {
        FLASH_REACH[self.kind]
    }

    pub fn blow_range(&self) -> f32 {
        BLOW_REACH[self.kind]
    }

    pub fn pickup_range(&self) -> f32 {
        REACH[self.kind]
    }

    pub fn request_exit(&mut self) {
        self.exit_request = true;
    }

    pub fn take(&mut self) {
        self.given = true;
    }

    pub fn rand(&mut self) -> f32 {
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 17;
        self.seed ^= self.seed << 5;
        (self.seed >> 8) as f32 / 16777216.0
    }
}

pub fn is_buoy_barrel(name: &str) -> bool {
    name == BARREL2_ACTOR
}

pub fn control(mut barrels: Query<(&mut Actor, &mut BuoyBarrel)>) {
    for (mut actor, mut barrel) in &mut barrels {
        if barrel.mode.demo() || barrel.given {
            continue;
        }
        match barrel.mode {
            Mode::Afloat => {
                barrel.float_step();
                actor.pos.x += barrel.drift_x;
                actor.pos.z += barrel.drift_z;
            }
            Mode::ExitV => {
                barrel.sink_step(EXIT_DAMP_V);
                if barrel.sunk(TRIP[barrel.kind]) {
                    barrel.gone = true;
                }
            }
            Mode::ExitH => {
                barrel.sink_step(EXIT_DAMP_H);
                if barrel.sunk(TRIP[barrel.kind]) {
                    barrel.gone = true;
                }
            }
            Mode::ExitMine => {
                barrel.sink_step(EXIT_DAMP_H);
                if barrel.sunk(MINE_SINK) {
                    barrel.gone = true;
                }
            }
            Mode::Explode => barrel.explode_step(),
            Mode::DemoBreak0 | Mode::DemoBreak1 | Mode::DemoExplode0 | Mode::DemoExplode1 => {}
        }
        actor.pos.y = barrel.height();
    }
}
