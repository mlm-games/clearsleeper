use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::combat::HitPoints;
use super::enemy::Hurt;
use super::events::{SimEvent, SimEvents};
use super::flags::ZoneActors;
use super::hammer::{self, Cut, Hammer};
use super::health::Health;
use super::player::Player;
use super::roll::Roll;
use super::ship::Ship;
use super::swim::Swim;

pub const FLINCH_TICKS: u32 = 10;
pub const REACH: f32 = hammer::AT_RADIUS;

pub fn reaches(centre: Vec3, target: Vec3) -> bool {
    let gap = target - centre;
    gap.x * gap.x + gap.z * gap.z <= REACH * REACH
}

pub fn control(
    cut: Res<Cut>,
    health: Res<Health>,
    swim: Res<Swim>,
    roll: Res<Roll>,
    ships: Query<&Ship>,
    hammers: Query<&Hammer>,
    players: Query<&Player>,
    mut actors: Query<
        (Entity, &Actor, &mut HitPoints, Option<&mut Hurt>),
        (Without<Player>, Without<Ship>),
    >,
    mut zones: ResMut<ZoneActors>,
    mut events: ResMut<SimEvents>,
    mut commands: Commands,
) {
    if health.units == 0 || swim.active || roll.active || ships.iter().any(|ship| ship.riding) {
        return;
    }
    if cut.kind == hammer::CUT_NONE || !hammers.iter().any(|hammer| hammer.striking()) {
        return;
    }
    let Some(player) = players.iter().next() else {
        return;
    };
    let hand = player.pos;
    let head = hammer::strike_origin(player);
    let power = hammer::AT_POWER;
    let mut dead: Vec<(Entity, i8, u16)> = Vec::new();
    for (entity, actor, mut hp, mut hurt) in &mut actors {
        if !reaches(hand, actor.pos) && !reaches(head, actor.pos) {
            continue;
        }
        hp.0 -= power;
        if let Some(hurt) = hurt.as_deref_mut() {
            hurt.0 = FLINCH_TICKS;
        }
        events.list.push(SimEvent::Hit);
        if hp.0 <= 0 {
            dead.push((entity, actor.room, actor.set_id));
        }
    }
    for (entity, room, set_id) in dead {
        zones.mark_done(room, set_id);
        let _ = commands.entity(entity).despawn();
    }
}