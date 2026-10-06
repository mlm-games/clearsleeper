use bevy_ecs::prelude::*;

use super::actor::Actor;
use super::combat::HitPoints;
use super::enemy::{self, Hurt};

pub const BALLOON_ACTOR: &str = "Bb";

const HITS: i32 = super::damage::KILL_POWER;
const EMPTY_KIND: u32 = 3;
const LANCE_KIND: u32 = 5;
const PAYLOAD_PARAM: u32 = 0xFFFF05;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Payload {
    Small,
    Lance,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Balloon {
    pub payload: Option<Payload>,
}

pub fn spawn(parameters: u32) -> (Balloon, HitPoints) {
    let payload = match parameters & 0xFF {
        EMPTY_KIND => None,
        LANCE_KIND => Some(Payload::Lance),
        _ => Some(Payload::Small),
    };
    (Balloon { payload }, HitPoints(HITS))
}

fn drop(actor: &Actor, payload: Payload, commands: &mut Commands) {
    let name = match payload {
        Payload::Lance => enemy::LANCE_ACTOR,
        Payload::Small => enemy::ENEMY_ACTOR,
    };
    let child = Actor {
        name: name.to_owned(),
        parameters: PAYLOAD_PARAM,
        pos: actor.pos,
        angle: actor.angle,
        set_id: actor.set_id,
        room: actor.room,
        ..Default::default()
    };
    let yaw = actor.angle[1];
    match payload {
        Payload::Lance => {
            if let Some((state, hit_points)) = enemy::spawn_lance(PAYLOAD_PARAM, yaw) {
                commands.spawn((child, state, hit_points, Hurt::default()));
            }
        }
        Payload::Small => {
            let (state, hit_points) = enemy::spawn(PAYLOAD_PARAM, yaw);
            commands.spawn((child, state, hit_points, Hurt::default()));
        }
    }
}

pub fn control(
    mut balloons: Query<(&mut Balloon, &Actor, &mut HitPoints)>,
    mut commands: Commands,
) {
    for (mut balloon, actor, hit_points) in &mut balloons {
        if hit_points.0 > 0 {
            continue;
        }
        let Some(payload) = balloon.payload.take() else {
            continue;
        };
        drop(actor, payload, &mut commands);
    }
}
