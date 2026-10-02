use bevy_ecs::prelude::*;
use glam::Vec2;

use super::area::Area;
use super::collision::Collision;
use super::events::{SimEvent, SimEvents};
use super::health::Health;
use super::intent::MoveIntent;
use super::player::Player;
use super::swim::Swim;

pub fn freeze(
    health: Res<Health>,
    mut intent: ResMut<MoveIntent>,
    mut players: Query<&mut Player>,
) {
    if health.units > 0 {
        return;
    }
    intent.axis = Vec2::ZERO;
    for mut player in &mut players {
        player.speed = 0.0;
    }
}

pub fn check(
    collision: Option<Res<Collision>>,
    area: Res<Area>,
    mut health: ResMut<Health>,
    mut swim: ResMut<Swim>,
    mut events: ResMut<SimEvents>,
    mut players: Query<&mut Player>,
) {
    for mut player in &mut players {
        let player = &mut *player;
        if health.units == 0 {
            if swim.active || grounded(collision.as_deref(), player) {
                respawn(player, &area, &mut swim, &mut events);
                health.units = health.max.min(12);
            }
            continue;
        }
        if swim.active && swim.air == 0 {
            health.damage(1);
            respawn(player, &area, &mut swim, &mut events);
        }
    }
}

fn grounded(collision: Option<&Collision>, player: &Player) -> bool {
    let surface = match collision {
        Some(collision) => collision.ground_at(player.pos.x, player.pos.z),
        None => Some(0.0),
    };
    matches!(surface, Some(ground) if player.pos.y <= ground)
}

fn respawn(player: &mut Player, area: &Area, swim: &mut Swim, events: &mut SimEvents) {
    player.pos = area.spawn;
    player.yaw = area.yaw;
    player.speed = 0.0;
    player.speed_y = 0.0;
    swim.active = false;
    swim.surfaced = false;
    swim.air = 0;
    events.list.push(SimEvent::Restart);
}
