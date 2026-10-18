use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::events::{SimEvent, SimEvents};
use super::flags::Switches;
use super::hammer::{self, CUT_HAMMER_FRONTSWING, CUT_HAMMER_SIDESWING, Cut};
use super::player::Player;
use super::walk;

pub const SWHAMMER_ACTOR: &str = "MhmrSW0";

pub const NO_SWITCH: u8 = 0xFF;

pub const MODE_UPPER: u8 = 0;
pub const MODE_U_L: u8 = 1;
pub const MODE_LOWER: u8 = 2;
pub const MODE_L_U: u8 = 3;

pub const CYL_RADIUS_TG: f32 = 50.0;
pub const CYL_HEIGHT_UP: f32 = 112.5;
pub const CYL_HEIGHT_LOW: f32 = 52.5;

pub const CULL_RADIUS: f32 = 30.0;
pub const CULL_TOP: f32 = 120.0;

pub const TOP_DROP: f32 = -60.0;
pub const V_SPRING: f32 = 0.8;
pub const INT_DECAY: f32 = 0.55;
pub const EXT_DECAY: f32 = 0.55;
pub const VIB_MAG: f32 = 1.0;
pub const VIB_SPRING: f32 = 0.99;
pub const ANGLE_DECAY: f32 = 0.18;

pub const CRUSH_SCALE: f32 = 0.2;
pub const CRUSH_DURATION: i16 = 18;
pub const UNCRUSH_SPEED: f32 = 0.0;
pub const UNCRUSH_SPRING: f32 = 0.8;
pub const UNCRUSH_DECAY: f32 = 0.45;

pub const CRUSH_NONE: u8 = 0;
pub const CRUSH_CRUSHING: u8 = 1;
pub const CRUSH_HELD: u8 = 2;

const SWITCH_SHIFT: u32 = 8;
const SWITCH2_SHIFT: u32 = 0;
const SWITCH_MASK: u32 = 0xFF;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Swhammer {
    pub mode: u8,
    pub switch_no: u8,
    pub switch2_no: u8,
    pub cur: f32,
    pub target: f32,
    pub speed: f32,
    pub top: f32,
    pub crush: u8,
    pub crush_timer: i16,
    pub scale_y: f32,
    pub scale_speed: f32,
    pub angle_x: f32,
    pub angle_z: f32,
    pub speed_x: f32,
    pub speed_z: f32,
    pub hit: bool,
}

impl Swhammer {
    pub fn spawn(parameters: u32, set: bool) -> Self {
        let switch_no = ((parameters >> SWITCH_SHIFT) & SWITCH_MASK) as u8;
        let switch2_no = ((parameters >> SWITCH2_SHIFT) & SWITCH_MASK) as u8;
        let (cur, top) = if set { (0.0, TOP_DROP) } else { (1.0, 0.0) };
        Self {
            mode: if set { MODE_LOWER } else { MODE_UPPER },
            switch_no,
            switch2_no,
            cur,
            target: 0.0,
            speed: 0.0,
            top,
            crush: CRUSH_NONE,
            crush_timer: 0,
            scale_y: 1.0,
            scale_speed: 0.0,
            angle_x: 0.0,
            angle_z: 0.0,
            speed_x: 0.0,
            speed_z: 0.0,
            hit: false,
        }
    }

    pub fn cyl_height(&self) -> f32 {
        match self.mode {
            MODE_LOWER | MODE_L_U => CYL_HEIGHT_LOW,
            _ => CYL_HEIGHT_UP,
        }
    }

    pub fn cull(&self) -> (f32, f32) {
        (CULL_RADIUS, CULL_TOP)
    }

    pub fn up(&mut self) {
        self.target = 1.0;
        self.hit = false;
        self.mode = MODE_UPPER;
    }

    pub fn down(&mut self) {
        self.target = 0.0;
        self.mode = MODE_U_L;
    }

    pub fn raised(&mut self) {
        self.target = 1.0;
        self.mode = MODE_L_U;
    }

    pub fn lowered(&mut self) {
        self.target = 0.0;
        self.hit = false;
        self.mode = MODE_LOWER;
    }

    pub fn crush_start(&mut self) {
        self.crush = CRUSH_CRUSHING;
    }

    pub fn vib_start(&mut self, dir: i16, mag: f32) {
        let scaled = mag * VIB_MAG;
        let (sin_dir, cos_dir) = walk::from_s16(dir).sin_cos();
        self.speed_z = scaled * cos_dir;
        self.speed_x = scaled * sin_dir;
    }

    pub fn vib_step(&mut self) {
        let accel_z = -(self.angle_z * VIB_SPRING) - self.speed_z * ANGLE_DECAY;
        let accel_x = -(self.angle_x * VIB_SPRING) - self.speed_x * ANGLE_DECAY;
        self.speed_z += accel_z;
        self.speed_x += accel_x;
        self.angle_z += self.speed_z;
        self.angle_x += self.speed_x;
    }

    pub fn crush_step(&mut self) {
        match self.crush {
            CRUSH_CRUSHING => {
                self.scale_y = CRUSH_SCALE;
                self.crush = CRUSH_HELD;
                self.crush_timer = CRUSH_DURATION;
            }
            CRUSH_HELD => {
                self.crush_timer -= 1;
                if self.crush_timer <= 0 {
                    self.crush = CRUSH_NONE;
                    self.scale_speed = UNCRUSH_SPEED;
                }
            }
            _ => {
                let spring = -((self.scale_y - 1.0) * UNCRUSH_SPRING);
                self.scale_speed += spring - self.scale_speed * UNCRUSH_DECAY;
                self.scale_y += self.scale_speed;
            }
        }
    }

    pub fn height_step(&mut self) {
        let diff = self.cur - self.target;
        let decay = if self.cur > 0.0 && self.cur < 1.0 {
            INT_DECAY
        } else {
            EXT_DECAY
        };
        self.speed -= diff * V_SPRING;
        self.speed -= self.speed * decay;
        self.cur = (self.cur + self.speed).min(1.0);
        self.top = (1.0 - self.cur) * TOP_DROP;
    }

    pub fn settled(&self) -> bool {
        self.cur <= 0.0
    }

    pub fn risen(&self) -> bool {
        self.cur >= 1.0
    }
}

pub fn is_swhammer(name: &str) -> bool {
    name == SWHAMMER_ACTOR
}

pub fn contact(hammer: &Swhammer, base: Vec3, player: &Player) -> bool {
    let reach = CYL_RADIUS_TG + hammer::AT_RADIUS;
    let gap = player.pos - base;
    gap.x * gap.x + gap.z * gap.z <= reach * reach && gap.y.abs() <= hammer.cyl_height() * 0.5
}

pub fn control(
    cut: Res<Cut>,
    players: Query<&Player>,
    mut hammers: Query<(&Actor, &mut Swhammer)>,
    mut switches: ResMut<Switches>,
    mut events: ResMut<SimEvents>,
) {
    let player = players.iter().next().copied();
    for (actor, mut hammer) in &mut hammers {
        let touching = player.is_some_and(|player| contact(&hammer, actor.pos, &player));
        match hammer.mode {
            MODE_UPPER => {
                if !touching {
                    continue;
                }
                let front = cut.kind == CUT_HAMMER_FRONTSWING;
                let side = cut.kind == CUT_HAMMER_SIDESWING;
                if front {
                    hammer.crush_start();
                    hammer.hit = true;
                    events.list.push(SimEvent::Se(hammer::SE_ON));
                    hammer.down();
                } else if side {
                    hammer.vib_start(walk::to_s16(player.map_or(0.0, |player| player.yaw)), 1.0);
                    events.list.push(SimEvent::Se(hammer::SE_SWING));
                } else if switches.test(hammer.switch_no) {
                    events.list.push(SimEvent::Se(hammer::SE_ON));
                    hammer.down();
                }
            }
            MODE_U_L => {
                if hammer.settled() {
                    if hammer.switch_no != NO_SWITCH {
                        switches.set(hammer.switch_no);
                    }
                    if hammer.hit && hammer.switch2_no != NO_SWITCH {
                        switches.clear(hammer.switch2_no);
                    }
                    hammer.lowered();
                }
            }
            MODE_LOWER => {
                if hammer.switch_no == NO_SWITCH || !switches.test(hammer.switch_no) {
                    events.list.push(SimEvent::Se(hammer::SE_OFF));
                    hammer.raised();
                }
            }
            MODE_L_U => {
                if hammer.risen() {
                    hammer.up();
                }
            }
            _ => {}
        }
        hammer.height_step();
        hammer.vib_step();
        hammer.crush_step();
    }
}
