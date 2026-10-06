use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::area::StageName;
use super::events::{SimEvent, SimEvents};
use super::flags::EventFlags;
use super::walk::from_s16;

pub const PIRATE_ACTORS: [&str; 1] = ["Pirates"];
pub const TOUSEKI_ACTOR: &str = "Touseki";

pub const UMIKZ_STAGE: &str = "A_umikz";
pub const SE_CRUISE: u32 = 0x7034;
pub const SE_SAIL: u32 = 0x50BE;

pub const EVENT_SHIP_READY: u32 = 0x0310;

pub const KIND_FIRST_ISLAND: u8 = 0;
pub const KIND_ARRIVAL: u8 = 3;

pub const NO_SET_ID: u16 = 0xFFFF;

pub const CATAPULT_OFFSET: Vec3 = Vec3::new(0.0, 700.0, 850.0);

pub const CULL_MIN: Vec3 = Vec3::new(-1400.0, -200.0, -1800.0);
pub const CULL_MAX: Vec3 = Vec3::new(1400.0, 3300.0, 2200.0);

const SAIL_MASK: u32 = 0xF;
const KIND_SHIFT: u32 = 0x18;
const KIND_MASK: u32 = 0xFF;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Pirate {
    pub kind: u8,
    pub sail: bool,
    pub visible: bool,
    pub catapult: Option<Entity>,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Catapult {
    pub ship: Entity,
}

impl Pirate {
    pub fn spawn(parameters: u32) -> Self {
        let kind = ((parameters >> KIND_SHIFT) & KIND_MASK) as u8;
        Self {
            kind,
            sail: parameters & SAIL_MASK != 0,
            visible: kind != KIND_FIRST_ISLAND,
            catapult: None,
        }
    }

    pub fn cull(&self) -> (Vec3, Vec3) {
        (CULL_MIN, CULL_MAX)
    }
}

fn mounted(angle: [i16; 3]) -> Vec3 {
    let (sx, cx) = from_s16(angle[0]).sin_cos();
    let (sy, cy) = from_s16(angle[1]).sin_cos();
    let (sz, cz) = from_s16(angle[2]).sin_cos();
    let off = CATAPULT_OFFSET;
    Vec3::new(
        cz * cy * off.x + (cz * sy * sx - sz * cx) * off.y + (cz * sy * cx + sz * sx) * off.z,
        sz * cy * off.x + (sz * sy * sx + cz * cx) * off.y + (sz * sy * cx - cz * sx) * off.z,
        -sy * off.x + cy * sx * off.y + cy * cx * off.z,
    )
}

pub fn is_pirateship(name: &str) -> bool {
    PIRATE_ACTORS.contains(&name)
}

pub fn control(
    stage: Res<StageName>,
    mut flags: ResMut<EventFlags>,
    mut events: ResMut<SimEvents>,
    mut ships: Query<(Entity, &Actor, &mut Pirate), With<Pirate>>,
    mut guns: Query<(&mut Actor, &Catapult), Without<Pirate>>,
    mut commands: Commands,
) {
    for (entity, actor, mut pirate) in &mut ships {
        if pirate.catapult.is_none() && pirate.kind != KIND_ARRIVAL {
            let child = Actor {
                name: TOUSEKI_ACTOR.to_owned(),
                parameters: 0,
                pos: actor.pos + mounted(actor.angle),
                angle: actor.angle,
                set_id: NO_SET_ID,
                room: actor.room,
                ..Default::default()
            };
            pirate.catapult = Some(commands.spawn((child, Catapult { ship: entity })).id());
        }
        for (mut mount, gun) in &mut guns {
            if gun.ship != entity {
                continue;
            }
            mount.pos = actor.pos + mounted(actor.angle);
            mount.angle = actor.angle;
        }
        if !pirate.visible {
            if !flags.test(EVENT_SHIP_READY) {
                continue;
            }
            pirate.visible = true;
        }
        if pirate.kind == KIND_FIRST_ISLAND {
            continue;
        }
        if stage.0 == UMIKZ_STAGE {
            events.list.push(SimEvent::Se(SE_CRUISE));
        }
        if pirate.sail {
            events.list.push(SimEvent::Se(SE_SAIL));
        }
    }
}