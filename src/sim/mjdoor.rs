use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::events::{SimEvent, SimEvents};
use super::flags::Switches;
use super::walk::from_s16;

pub const MJDOOR_ACTORS: [&str; 1] = ["MjDoor"];

pub const MODE_WAIT: u8 = 0x0;
pub const MODE_DELETE: u8 = 0x1;

pub const NO_SWITCH: u8 = 0xFF;

pub const MAX_HEALTH: i32 = 30;
pub const DELETE_TICKS: i32 = 600;

pub const SE_GATE_BREAK: u32 = 0x6978;

pub const CULL_MIN: [f32; 3] = [-100.0, 0.0, -100.0];
pub const CULL_MAX: [f32; 3] = [100.0, 800.0, 100.0];

const PARAM_MASK: u32 = 0xFF;
const CYL_OFFSETS: [f32; 10] = [
    -500.0, -400.0, -300.0, -200.0, -100.0, 0.0, 100.0, 200.0, 300.0, 400.0,
];
const CYL_RADIUS: f32 = 100.0;
const CYL_HEIGHT: f32 = 1350.0;
const BLAST_RADIUS: f32 = 200.0;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct MjDoor {
    pub mode: u8,
    pub switch_no: u8,
    pub health: i32,
    pub refused: bool,
    pub delete_timer: i32,
}

impl MjDoor {
    pub fn spawn(parameters: u32, switches: &Switches) -> Self {
        let switch_no = (parameters & PARAM_MASK) as u8;
        Self {
            mode: MODE_WAIT,
            switch_no,
            health: MAX_HEALTH,
            refused: switch_no != NO_SWITCH && switches.test(switch_no),
            delete_timer: 0,
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        self.mode != MODE_DELETE
    }

    fn delete_init(&mut self, switches: &mut Switches) {
        if self.switch_no != NO_SWITCH {
            switches.set(self.switch_no);
        }
        self.mode = MODE_DELETE;
        self.delete_timer = DELETE_TICKS;
    }
}

/// `cM3d_Cross_CylSph` (`tww/src/SSystem/SComponent/c_m3d.cpp:1389-1415`) is the exact test
/// `ProcAtTgHit` runs for a bomb's `AT_TYPE_BOMB` sphere against one of the ten door cylinders:
/// the horizontal span is `cyl.r + sph.r`, then the sphere must straddle the cylinder's
/// `y .. y + h` interval. `setCollision` (`d_a_obj_majyuu_door.cpp:181-188`) rebuilds all ten
/// cylinders on every `modeWait` tick, so `TgHitCallback` (`:64-68`) decrements `health` once per
/// overlapping cylinder per overlapping bomb.
fn bomb_strikes(door: &Actor, origin: Vec3) -> i32 {
    let (sin, cos) = from_s16(door.angle[1]).sin_cos();
    let span = CYL_RADIUS + BLAST_RADIUS;
    let mut strikes = 0;
    for offset in CYL_OFFSETS {
        let dx = origin.x - (door.pos.x + cos * offset);
        let dz = origin.z - (door.pos.z - sin * offset);
        if span * span < dx * dx + dz * dz {
            continue;
        }
        if origin.y + BLAST_RADIUS >= door.pos.y
            && origin.y - BLAST_RADIUS <= door.pos.y + CYL_HEIGHT
        {
            strikes += 1;
        }
    }
    strikes
}

pub fn is_mjdoor(name: &str) -> bool {
    MJDOOR_ACTORS.contains(&name)
}

pub fn control(
    mut switches: ResMut<Switches>,
    mut events: ResMut<SimEvents>,
    mut doors: Query<(Entity, &Actor, &mut MjDoor)>,
    published: Res<super::bombflower::Blasts>,
    mut commands: Commands,
) {
    let blasts: Vec<Vec3> = published.list.clone();
    let mut gone: Vec<Entity> = Vec::new();
    for (entity, actor, mut door) in &mut doors {
        if door.refused {
            gone.push(entity);
            continue;
        }
        if door.mode == MODE_DELETE {
            if door.delete_timer == 0 {
                gone.push(entity);
            } else {
                door.delete_timer -= 1;
            }
            continue;
        }
        for origin in &blasts {
            door.health -= bomb_strikes(actor, *origin);
        }
        if door.health < 0 {
            door.delete_init(&mut switches);
            events.list.push(SimEvent::Se(SE_GATE_BREAK));
        }
    }
    for entity in gone {
        let _ = commands.entity(entity).despawn();
    }
}