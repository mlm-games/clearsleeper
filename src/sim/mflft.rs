use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::arrow::{self, Arrow};
use super::collision::Collision;
use super::events::{SimEvent, SimEvents};
use super::flags::{EventFlags, EventReg, Switches};
use super::ground;
use super::hammer::{self, Cut, Hammer};
use super::player::Player;
use super::stream;
use super::walk;

pub const MFLFT_ACTORS: [&str; 1] = ["Mflft"];

pub const ROPES: usize = 3;
pub const NO_SWITCH: u8 = 0xFF;

pub const MODE_HANG: u8 = 0;
pub const MODE_DROP: u8 = 1;

pub const ROPE_HOLD: i32 = 3;
pub const PROBE: f32 = 1000.0;

pub const CUT_BIT: [u8; ROPES] = [1, 2, 4];
pub const CUT_ALL: u8 = 7;
pub const ROLL_HOLD: [i16; 8] = [
    0x0001, 0x0000, 0x5555, 0x2AAA, 0xAAAA, 0xD555, 0x8000, 0x0001,
];
pub const LEVEL_ROLL: i16 = 0x0001;
pub const HANG_NOD: i16 = -0x3000;

pub const NOD_SCALE: i16 = 4;
pub const NOD_STEP: i16 = 0x2000;
pub const ROLL_SCALE: i16 = 2;
pub const ROLL_STEP: i16 = 0x2000;
pub const LEVEL_SCALE: i16 = 10;
pub const LEVEL_STEP: i16 = 0x200;
pub const FLAT_SCALE: i16 = 10;
pub const FLAT_STEP: i16 = 0x300;
pub const REST_NOD_SCALE: i16 = 4;
pub const REST_NOD_STEP: i16 = 0x200;
pub const STILL_NOD_SCALE: i16 = 2;
pub const STILL_NOD_STEP: i16 = 0x2000;

pub const LEAD_SCALE: f32 = 1.0;
pub const LEAD_MIN: f32 = 0.0;
pub const HANG_LEAN_TOP: f32 = 100.0;
pub const HANG_LEAN_STEP: f32 = 20.0;
pub const DROP_LEAN_TOP: f32 = 400.0;
pub const DROP_LEAN_STEP: f32 = 20.0;
pub const LAND_SHAKE: f32 = 2000.0;
pub const SWING_DECAY: f32 = 0.25;

pub const HANG_LEAN_X: i32 = 0x5dc;
pub const HANG_LEAN_Z: i32 = 0x514;
pub const HANG_SWING_X: i32 = 0x2ee;
pub const HANG_SWING_Z: i32 = 900;
pub const DROP_LEAN_X: i32 = 800;
pub const DROP_LEAN_Z: i32 = 700;

pub const FALL_GRAVITY: f32 = -5.0;
pub const FALL_MIN: f32 = -300.0;
pub const LAND_OFFSET: f32 = 80.0;
pub const LAND_SPEED: f32 = -50.0;

pub const RESTORE_SCALE: f32 = 0.1;
pub const RESTORE_STEP: f32 = 5.0;

pub const ANCHOR_X: [f32; ROPES] = [0.0, -117.0, 117.0];
pub const ANCHOR_Y: f32 = 15.0;
pub const ANCHOR_Z: [f32; ROPES] = [-134.0, 70.0, 70.0];

pub const CYL_RADIUS: f32 = 10.0;
pub const CYL_HEIGHT: f32 = 1000.0;
pub const SWAY_GATE: i16 = 10;
pub const SWAY_HOLD: i16 = 0x0f;
pub const SETTLE_FRAMES: i8 = 0x14;

pub const POWER_FLOOR: i32 = 1;
pub const CUT_DAMAGE: i32 = 4;

pub const SE_ROPE_HIT: u32 = 0x2837;
pub const SE_ROPE_CUT: u32 = 0x2838;
pub const SE_LANDING: u32 = 0x69EB;

pub const EVENT_BIT: u32 = 0x2A10;
pub const EVENT_REG: u16 = 0xA507;
pub const EVENT_REG_TOP: u8 = 6;

pub const CULL_HALF: f32 = 200.0;
pub const CULL_TOP: f32 = 20000.0;

const SWITCH_SHIFT: u32 = 24;

fn post(index: usize) -> Vec3 {
    Vec3::new(ANCHOR_X[index], ANCHOR_Y, ANCHOR_Z[index])
}

fn yrot(v: Vec3, units: i16) -> Vec3 {
    let (s, c) = walk::from_s16(units).sin_cos();
    Vec3::new(v.x * c + v.z * s, v.y, v.z * c - v.x * s)
}

fn xrot(v: Vec3, units: i16) -> Vec3 {
    let (s, c) = walk::from_s16(units).sin_cos();
    Vec3::new(v.x, v.y * c - v.z * s, v.y * s + v.z * c)
}

fn zrot(v: Vec3, units: i16) -> Vec3 {
    let (s, c) = walk::from_s16(units).sin_cos();
    Vec3::new(v.x * c - v.y * s, v.x * s + v.y * c, v.z)
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Mflft {
    pub switch_no: u8,
    pub mode: u8,
    pub frame: i16,
    pub shape: [i16; 3],
    pub severed: [bool; ROPES],
    pub mask: u8,
    pub hp: [i32; ROPES],
    pub sway: [i16; ROPES],
    pub anchor: [Vec3; ROPES],
    pub settle: i8,
    pub twist: i16,
    pub nod: i16,
    pub swing_x: f32,
    pub swing_z: f32,
    pub lean_x: f32,
    pub lean_z: f32,
    pub floor: f32,
    pub speed_y: f32,
    pub home: Vec3,
    pub placed: Option<f32>,
}

impl Mflft {
    pub fn spawn(
        parameters: u32,
        home: Vec3,
        switches: &Switches,
        collision: Option<&Collision>,
    ) -> Self {
        let switch_no = (parameters >> SWITCH_SHIFT) as u8;
        let severed = switch_no != NO_SWITCH && switches.test(switch_no);
        let placed = if severed {
            ground::surface(collision, home + Vec3::new(0.0, PROBE, 0.0))
        } else {
            None
        };
        let anchor = std::array::from_fn(|index| post(index) + home);
        Self {
            switch_no,
            mode: MODE_HANG,
            frame: 0,
            shape: [0; 3],
            severed: [severed; ROPES],
            mask: if severed { CUT_ALL } else { 0 },
            hp: [ROPE_HOLD; ROPES],
            sway: [0; ROPES],
            anchor,
            settle: 0,
            twist: 0,
            nod: 0,
            swing_x: 0.0,
            swing_z: 0.0,
            lean_x: 0.0,
            lean_z: 0.0,
            floor: 0.0,
            speed_y: 0.0,
            home,
            placed,
        }
    }

    pub fn cull(&self) -> (f32, f32) {
        (CULL_HALF, CULL_TOP)
    }

    fn spin(&self, rate: i32) -> i16 {
        (i32::from(self.frame) * rate) as i16
    }

    fn mount(&self, index: usize, pos: Vec3) -> Vec3 {
        let mut v = post(index);
        v = yrot(v, self.shape[1]);
        v = yrot(v, self.twist);
        v = xrot(v, self.nod);
        v = yrot(v, self.twist.wrapping_neg());
        v = xrot(v, self.shape[0]);
        v = zrot(v, self.shape[2]);
        pos + v
    }

    fn pose(&mut self, actor: &mut Actor, nod_x: i16, nod_z: i16) {
        self.shape = [
            actor.angle[0].wrapping_add(nod_x),
            actor.angle[1],
            actor.angle[2].wrapping_add(nod_z),
        ];
    }

    fn hang(&mut self, actor: &mut Actor, roll: i16, target: i16) {
        stream::add_calc_angle_s2(&mut self.nod, target, NOD_SCALE, NOD_STEP);
        stream::add_calc_angle_s2(&mut self.twist, roll, ROLL_SCALE, ROLL_STEP);
        stream::add_calc_angle_s2(&mut actor.angle[0], 0, LEVEL_SCALE, LEVEL_STEP);
        stream::add_calc_angle_s2(&mut actor.angle[2], 0, LEVEL_SCALE, LEVEL_STEP);
        let nod_x = (self.lean_x * stream::sin_s(self.spin(HANG_LEAN_X))) as i16;
        let nod_z = (self.lean_z * stream::sin_s(self.spin(HANG_LEAN_Z))) as i16;
        walk::add_calc(
            &mut self.lean_x,
            HANG_LEAN_TOP,
            LEAD_SCALE,
            HANG_LEAN_STEP,
            LEAD_MIN,
        );
        walk::add_calc(
            &mut self.lean_z,
            HANG_LEAN_TOP,
            LEAD_SCALE,
            HANG_LEAN_STEP,
            LEAD_MIN,
        );
        let drift_x = self.swing_x * stream::sin_s(self.spin(HANG_SWING_X));
        let drift_z = self.swing_z * stream::sin_s(self.spin(HANG_SWING_Z));
        walk::chase_f(&mut self.swing_x, 0.0, SWING_DECAY);
        walk::chase_f(&mut self.swing_z, 0.0, SWING_DECAY);
        self.pose(actor, nod_x, nod_z);
        actor.pos = self.home + Vec3::new(drift_x, 0.0, drift_z);
    }

    fn fall(&mut self, actor: &mut Actor) -> bool {
        walk::add_calc(
            &mut actor.pos.x,
            self.home.x,
            RESTORE_SCALE,
            RESTORE_STEP,
            LEAD_MIN,
        );
        walk::add_calc(
            &mut actor.pos.z,
            self.home.z,
            RESTORE_SCALE,
            RESTORE_STEP,
            LEAD_MIN,
        );
        stream::add_calc_angle_s2(&mut actor.angle[0], 0, FLAT_SCALE, FLAT_STEP);
        stream::add_calc_angle_s2(&mut actor.angle[2], 0, FLAT_SCALE, FLAT_STEP);
        stream::add_calc_angle_s2(&mut self.nod, 0, REST_NOD_SCALE, REST_NOD_STEP);
        let nod_x = (self.lean_x * stream::sin_s(self.spin(DROP_LEAN_X))) as i16;
        let nod_z = (self.lean_z * stream::sin_s(self.spin(DROP_LEAN_Z))) as i16;
        walk::add_calc(
            &mut self.lean_x,
            DROP_LEAN_TOP,
            LEAD_SCALE,
            DROP_LEAN_STEP,
            LEAD_MIN,
        );
        walk::add_calc(
            &mut self.lean_z,
            DROP_LEAN_TOP,
            LEAD_SCALE,
            DROP_LEAN_STEP,
            LEAD_MIN,
        );
        self.pose(actor, nod_x, nod_z);
        actor.pos.y += self.speed_y;
        self.speed_y = (self.speed_y + FALL_GRAVITY).max(FALL_MIN);
        let deck = self.floor + LAND_OFFSET;
        if actor.pos.y > deck {
            return false;
        }
        actor.pos.y = deck;
        let landed = self.speed_y < LAND_SPEED;
        if landed {
            self.lean_x = LAND_SHAKE;
            self.lean_z = LAND_SHAKE;
        }
        self.speed_y = 0.0;
        stream::add_calc_angle_s2(&mut self.nod, 0, STILL_NOD_SCALE, STILL_NOD_STEP);
        landed
    }

    fn posts(&mut self, actor: &Actor) {
        for index in 0..ROPES {
            self.anchor[index] = self.mount(index, actor.pos);
            if self.sway[index] != 0 {
                self.sway[index] -= 1;
            }
        }
    }

    fn touch(&self, index: usize, at: Vec3, reach: f32) -> bool {
        if self.severed[index] || self.sway[index] >= SWAY_GATE {
            return false;
        }
        let gap = at - self.anchor[index];
        gap.x * gap.x + gap.z * gap.z <= reach * reach
            && gap.y.unsigned_abs() <= CYL_HEIGHT * 0.5
    }

    fn clash(&mut self, index: usize, at: Vec3, reach: f32, power: i32) -> Option<u32> {
        if !self.touch(index, at, reach) {
            return None;
        }
        self.sway[index] = SWAY_HOLD;
        self.hp[index] -= if power > POWER_FLOOR {
            CUT_DAMAGE
        } else {
            power
        };
        if self.hp[index] > 0 {
            return Some(SE_ROPE_HIT);
        }
        self.severed[index] = true;
        self.mask |= CUT_BIT[index];
        self.lean_x = LAND_SHAKE;
        self.lean_z = LAND_SHAKE;
        self.settle = SETTLE_FRAMES;
        Some(SE_ROPE_CUT)
    }
}

pub fn is_mflft(name: &str) -> bool {
    MFLFT_ACTORS.contains(&name)
}

pub fn control(
    collision: Option<Res<Collision>>,
    players: Query<&Player>,
    hammers: Query<&Hammer>,
    shafts: Query<&Arrow>,
    cut: Res<Cut>,
    mut switches: ResMut<Switches>,
    mut flags: ResMut<EventFlags>,
    mut regs: ResMut<EventReg>,
    mut events: ResMut<SimEvents>,
    mut lifts: Query<(&mut Actor, &mut Mflft)>,
) {
    let solid = collision.as_deref();
    let striking = cut.kind != hammer::CUT_NONE && hammers.iter().any(|swing| swing.striking());
    let blow = players.iter().next().map(hammer::strike_origin);
    let arrows: Vec<(Vec3, i32)> = shafts
        .iter()
        .filter(|shaft| shaft.phase == arrow::Phase::Flying)
        .map(|shaft| (shaft.pos + shaft.speed * arrow::CAPSULE_STRETCH, arrow::POWER))
        .collect();
    for (actor, lift) in &mut lifts {
        let actor = &mut *actor;
        let lift = &mut *lift;
        if let Some(placed) = lift.placed.take() {
            actor.pos.y = placed;
        }
        lift.frame = lift.frame.wrapping_add(1);
        match lift.mode {
            MODE_HANG => {
                let roll = ROLL_HOLD[usize::from(lift.mask) & usize::from(CUT_ALL)];
                let target = if roll != LEVEL_ROLL {
                    HANG_NOD
                } else {
                    lift.nod = 0;
                    LEVEL_ROLL
                };
                if lift.mask == CUT_ALL {
                    lift.mode = MODE_DROP;
                    flags.set(EVENT_BIT);
                    if let Some(floor) =
                        ground::surface(solid, actor.pos + Vec3::new(0.0, PROBE, 0.0))
                    {
                        lift.floor = floor;
                    }
                    if lift.switch_no != NO_SWITCH {
                        switches.set(lift.switch_no);
                    }
                } else {
                    if lift.settle != 0 {
                        lift.settle -= 1;
                        if lift.settle == 0 && lift.mask != 0 {
                            let reg = regs.get(EVENT_REG);
                            if reg < EVENT_REG_TOP {
                                regs.set(EVENT_REG, reg + 1);
                            }
                        }
                    }
                    lift.hang(actor, roll, target);
                }
            }
            MODE_DROP => {
                if lift.fall(actor) {
                    events.list.push(SimEvent::Se(SE_LANDING));
                }
            }
            _ => {}
        }
        lift.posts(actor);
        for index in 0..ROPES {
            let mut sound = None;
            if striking {
                if let Some(at) = blow {
                    sound = lift.clash(
                        index,
                        at,
                        CYL_RADIUS + hammer::AT_RADIUS,
                        hammer::AT_POWER,
                    );
                }
            }
            if sound.is_none() {
                for (at, power) in &arrows {
                    if let Some(hit) =
                        lift.clash(index, *at, CYL_RADIUS + arrow::CAPSULE_RADIUS, *power)
                    {
                        sound = Some(hit);
                        break;
                    }
                }
            }
            if let Some(sound) = sound {
                events.list.push(SimEvent::Se(sound));
            }
        }
    }
}