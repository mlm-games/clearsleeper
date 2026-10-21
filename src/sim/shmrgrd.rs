use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::events::{SimEvent, SimEvents};
use super::flags::Switches;
use super::hammer::{self, CUT_HAMMER_FRONTSWING, CUT_HAMMER_SIDESWING, Cut};
use super::player::Player;
use super::walk;

pub const GUARD_ACTOR: &str = "Shmrgrd";

pub const MODE_UPPER: u8 = 0;
pub const MODE_U_L: u8 = 1;
pub const MODE_LOWER: u8 = 2;

pub const NO_SWITCH: u8 = 0xFF;

pub const CYL_RADIUS_TG: f32 = 50.0;
pub const CYL_HEIGHT_TG: f32 = 112.5;

pub const CULL_MIN: [f32; 3] = [-90.0, 0.0, -90.0];
pub const CULL_MAX: [f32; 3] = [90.0, 120.0, 90.0];

pub const TOP_DROP: f32 = -85.0;

pub const GAP_RADIUS: f32 = 165.0;
pub const PLAYER_CONE: i16 = 0x2000;

pub const V_SPRING: f32 = 0.8;
pub const INT_DECAY: f32 = 0.55;
pub const EXT_DECAY: f32 = 0.55;
pub const VIB_MAG: f32 = 1.0;
pub const VIB_SPRING: f32 = 0.99;
pub const ANGLE_DECAY: f32 = 0.18;

pub const CRUSH_NONE: u8 = 0;
pub const CRUSH_CRUSHING: u8 = 1;
pub const CRUSH_HELD: u8 = 2;

pub const CRUSH_SCALE: f32 = 0.2;
pub const CRUSH_DURATION: i16 = 18;

/// `prm_get_swSave()` (`d_a_obj_shmrgrd.h:38`) is
/// `daObj::PrmAbstract(this, PRM_SWSAVE_W = 0x8, PRM_SWSAVE_S = 0x0)`
/// (`include/d/d_a_obj.h:12-15`), i.e. `((1 << 8) - 1) & (param >> 0)` - the switch number is the
/// low byte of the parameter.
const SWITCH_SHIFT: u32 = 0x00;
const SWITCH_MASK: u32 = 0xFF;

/// `prm_get_swSave()` (`d_a_obj_shmrgrd.h:38`) - the switch this guard reads and writes.
pub fn switch_no(parameters: u32) -> u8 {
    ((parameters >> SWITCH_SHIFT) & SWITCH_MASK) as u8
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Shmrgrd {
    pub mode: u8,
    pub switch_no: u8,
    pub cur: f32,
    pub target: f32,
    pub speed: f32,
    pub top: f32,
    pub crush: u8,
    pub crush_timer: i16,
    pub scale_y: f32,
    pub angle_x: f32,
    pub angle_z: f32,
    pub speed_x: f32,
    pub speed_z: f32,
    pub gap: Option<Entity>,
    pub gap_pos: Vec3,
}

impl Shmrgrd {
    pub fn spawn(parameters: u32, set: bool) -> Self {
        let switch_no = switch_no(parameters);
        let (cur, top) = if set { (0.0, TOP_DROP) } else { (1.0, 0.0) };
        Self {
            mode: if set { MODE_LOWER } else { MODE_UPPER },
            switch_no,
            cur,
            target: 0.0,
            speed: 0.0,
            top,
            crush: CRUSH_NONE,
            crush_timer: 0,
            scale_y: 1.0,
            angle_x: 0.0,
            angle_z: 0.0,
            speed_x: 0.0,
            speed_z: 0.0,
            gap: None,
            gap_pos: Vec3::ZERO,
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn raised(&self) -> bool {
        self.mode == MODE_UPPER
    }

    pub fn down(&mut self) {
        self.target = 0.0;
        self.mode = MODE_U_L;
    }

    pub fn lowered(&mut self) {
        self.target = 0.0;
        self.mode = MODE_LOWER;
    }

    pub fn crush_start(&mut self) {
        self.crush = CRUSH_CRUSHING;
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
                    self.scale_y = 1.0;
                    self.crush = CRUSH_NONE;
                }
            }
            _ => {}
        }
    }

    pub fn vib_start(&mut self, dir: i16, mag: f32) {
        let scaled = mag * VIB_MAG;
        let (sin_dir, cos_dir) = walk::from_s16(dir).sin_cos();
        self.speed_z = scaled * cos_dir;
        self.speed_x = scaled * sin_dir;
    }

    pub fn vib_step(&mut self) {
        let accel_x = -(self.angle_x * VIB_SPRING) - self.speed_x * ANGLE_DECAY;
        let accel_z = -(self.angle_z * VIB_SPRING) - self.speed_z * ANGLE_DECAY;
        self.speed_z += accel_z;
        self.speed_x += accel_x;
        self.angle_z += self.speed_z;
        self.angle_x += self.speed_x;
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
}

pub fn is_guard(name: &str) -> bool {
    name == GUARD_ACTOR
}

pub fn contact(guard: &Shmrgrd, base: Vec3, player: &Player) -> bool {
    let reach = CYL_RADIUS_TG + hammer::AT_RADIUS;
    let gap = player.pos - base;
    gap.x * gap.x + gap.z * gap.z <= reach * reach && gap.y.abs() <= CYL_HEIGHT_TG * 0.5
}

pub fn facing(base: Vec3, player: &Player) -> bool {
    let want = walk::to_s16((base.x - player.pos.x).atan2(base.z - player.pos.z));
    walk::to_s16(player.face).wrapping_sub(want).unsigned_abs() < PLAYER_CONE as u16
}

pub fn control(
    cut: Res<Cut>,
    players: Query<&Player>,
    mut guards: Query<(Entity, &Actor, &mut Shmrgrd)>,
    mut switches: ResMut<Switches>,
    mut events: ResMut<SimEvents>,
) {
    let player = players.iter().next().copied();
    let list: Vec<(Entity, Vec3, bool)> = guards
        .iter()
        .map(|(entity, actor, guard)| (entity, actor.pos, guard.raised()))
        .collect();
    for (entity, actor, mut guard) in &mut guards {
        match guard.mode {
            MODE_UPPER => {
                let touched = player.is_some_and(|player| {
                    contact(&guard, actor.pos, &player) && facing(actor.pos, &player)
                });
                // `mode_upper` (`d_a_obj_shmrgrd.cpp:522-531`) tests the damage before the
                // switch and the switch before the side swing, and it runs every frame - the
                // switch test is not tied to a hammer hit, so a guard whose switch another
                // guard already set lowers on its own.
                if touched && cut.kind == CUT_HAMMER_FRONTSWING {
                    guard.crush_start();
                    events.list.push(SimEvent::Se(hammer::SE_ON));
                    guard.down();
                } else if guard.switch_no != NO_SWITCH && switches.test(guard.switch_no) {
                    events.list.push(SimEvent::Se(hammer::SE_ON));
                    guard.down();
                } else if touched && cut.kind == CUT_HAMMER_SIDESWING {
                    guard.vib_start(walk::to_s16(player.map_or(0.0, |player| player.yaw)), 1.0);
                    events.list.push(SimEvent::Se(hammer::SE_SWING));
                }
            }
            MODE_U_L => {
                if guard.settled() {
                    if guard.switch_no != NO_SWITCH {
                        switches.set(guard.switch_no);
                    }
                    guard.lowered();
                }
            }
            _ => {}
        }
        guard.height_step();
        guard.vib_step();
        guard.crush_step();
        guard.gap = search_gap(entity, actor.pos, &list);
        if let Some(gap) = guard.gap {
            guard.gap_pos = list
                .iter()
                .find(|(other, _, _)| *other == gap)
                .map(|(_, pos, _)| (*pos - actor.pos) * 0.5 + actor.pos)
                .unwrap_or(actor.pos);
        }
    }
}

fn search_gap(entity: Entity, pos: Vec3, list: &[(Entity, Vec3, bool)]) -> Option<Entity> {
    for (other, other_pos, raised) in list {
        if *other == entity || !*raised {
            continue;
        }
        let delta_x = other_pos.x - pos.x;
        let delta_z = other_pos.z - pos.z;
        if delta_x * delta_x + delta_z * delta_z <= GAP_RADIUS * GAP_RADIUS {
            return Some(*other);
        }
    }
    None
}
