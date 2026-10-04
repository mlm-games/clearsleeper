use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::player::Player;
use super::ship::Ship;
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

pub const COLLIDE_R: [f32; TYPES] = [200.0, 200.0, 288.0, 288.0];
pub const COLLIDE_H: [f32; TYPES] = [300.0, 400.0, 432.0, 576.0];
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
pub const FLASH_REACH: [f32; TYPES] = [0.0, 0.0, 7500.0, 0.0];
pub const BLOW_REACH: [f32; TYPES] = [0.0, 0.0, 900.0, 0.0];
pub const STRIKE_LIFT: [f32; TYPES] = [0.0, 0.0, 250.0, 0.0];
pub const STRIKE_DAMAGE: [f32; TYPES] = [0.0, 0.0, 700.0, 0.0];
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


    pub fn strike_lift(&self) -> f32 {
        STRIKE_LIFT[self.kind]
    }

    pub fn damage_reach(&self) -> f32 {
        STRIKE_DAMAGE[self.kind]
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
        self.phase += (self.rand() + 1.0) as i32 * PHASE_STEP[self.kind] as i32;
    }

    pub fn tick_cooldown(&mut self) {
        if self.cooldown > 0 {
            self.cooldown -= 1;
        }
    }

    pub fn float_step(&mut self, anchor: Vec3, pos: Vec3) {
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
        self.drift_x = (self.drift_x + (pos.x - anchor.x) * DRIFT_PUSH) * DRIFT_DECAY;
        self.drift_z = (self.drift_z + (pos.z - anchor.z) * DRIFT_PUSH) * DRIFT_DECAY;
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


    pub fn sunk(&self, depth: f32) -> bool {
        self.heave <= self.sink_depth() - depth
    }

    pub fn clip_barge(&mut self) {
        if self.cooldown == 0 {
            self.heave_vel = CLIP_SEED;
            self.cooldown = CLIP_COOLDOWN;
        }
    }

    pub fn clip_step(&mut self) {
        if self.heave_vel > 0.0 {
            self.heave_vel = 0.0;
        }
        self.heave_vel = (self.heave_vel - CLIP_STEP).max(CLIP_FLOOR);
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


    pub fn crashed(&mut self) {
        self.trigger = TRIGGER_CRASH;
    }

    pub fn touched(&mut self) {
        self.trigger = TRIGGER_TOUCH;
    }

    pub fn ship_speed(&self) -> f32 {
        SHIP_SPEED[self.kind]
    }

    pub fn head_on(&self, gap: Vec3, heading: Vec3) -> bool {
        (gap.x * heading.x + gap.z * heading.z) < CRASH_DEPTH[self.kind] * CRASH_DEPTH[self.kind]
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

pub fn control(
    mut switches: ResMut<super::flags::Switches>,
    mut barrels: Query<(&mut Actor, &mut BuoyBarrel)>,
    ships: Query<(&Actor, &Ship)>,
    players: Query<&Player>,
) {
    let ship = ships.iter().next();
    let player = players.iter().next();
    for (mut actor, mut barrel) in &mut barrels {
        if barrel.mode.demo() || barrel.given {
            continue;
        }
        dispatch(&actor, &mut barrel, ship);
        barrel.tick_cooldown();
        if let Some(player) = player
            && barrel.pickup_range() > 0.0
            && barrel.within(barrel.pickup_range(), player.pos - actor.pos)
        {
            barrel.take();
            if barrel.item_save != NO_ITEM_SAVE {
                switches.set(barrel.item_save);
            }
        }
        if barrel.exit_request && barrel.mode == Mode::Afloat {
            match barrel.kind {
                0 => barrel.enter(Mode::ExitV),
                1 => barrel.enter(Mode::ExitH),
                KIND_MINE if !barrel.flashing => barrel.enter(Mode::ExitMine),
                _ => {}
            }
            barrel.exit_request = false;
        }
        match barrel.trigger {
            TRIGGER_BOMB => {
                barrel.trigger = TRIGGER_NONE;
                barrel.given = true;
                continue;
            }
            TRIGGER_CRASH if !barrel.mode.demo() => barrel.enter(Mode::DemoBreak0),
            TRIGGER_TOUCH => barrel.clip_step(),
            _ => {}
        }
        barrel.trigger = TRIGGER_NONE;
        let anchor = actor.pos;
        match barrel.mode {
            Mode::Afloat => {
                barrel.float_step(anchor, actor.pos);
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

fn dispatch(actor: &Actor, barrel: &mut BuoyBarrel, ship: Option<(&Actor, &Ship)>) {
    let Some((hull, ship)) = ship else {
        return;
    };
    let gap = hull.pos - actor.pos;
    if barrel.kind == KIND_MINE
        && barrel.mode != Mode::Explode
        && !barrel.mode.demo()
        && barrel.within(barrel.blow_range(), gap)
    {
        barrel.enter(Mode::Explode);
        return;
    }
    if barrel.within(barrel.flash_range(), gap) {
        barrel.flashing = true;
    }
    if barrel.flashing
        && barrel.within(barrel.damage_reach(), gap)
        && ship.riding
        && hull.pos.y < actor.pos.y + barrel.strike_lift()
        && barrel.head_on(gap, ship_heading(gap, ship.speed))
    {
        barrel.enter(Mode::DemoExplode0);
        return;
    }
    if barrel.kind == 0 {
        if ship.speed > barrel.ship_speed() && barrel.head_on(gap, ship_heading(gap, ship.speed)) {
            barrel.crashed();
        } else if barrel.cooldown == 0 {
            barrel.clip_barge();
        }
    } else {
        barrel.touched();
    }
}

fn ship_heading(gap: Vec3, speed: f32) -> Vec3 {
    if gap.x == 0.0 && gap.z == 0.0 {
        return Vec3::Z * speed;
    }
    gap.normalize() * speed
}
