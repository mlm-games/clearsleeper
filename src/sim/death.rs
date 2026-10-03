use bevy_ecs::prelude::*;
use glam::Vec2;

use super::area::Area;
use super::collision::Collision;
use super::events::{SimEvent, SimEvents};
use super::ground::grounded;
use super::health::Health;
use super::hop::Hop;
use super::intent::{GameOverChoice, MoveIntent};
use super::player::Player;
use super::roll::Roll;
use super::swim::Swim;

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct GameOver {
    pub active: bool,
}

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
    mut roll: ResMut<Roll>,
    mut hop: ResMut<Hop>,
    mut game_over: ResMut<GameOver>,
    mut choice: ResMut<GameOverChoice>,
    mut events: ResMut<SimEvents>,
    mut players: Query<&mut Player>,
) {
    for mut player in &mut players {
        let player = &mut *player;
        if game_over.active {
            if *choice == GameOverChoice::None {
                continue;
            }
            respawn(player, &area, &mut swim, &mut roll, &mut hop, &mut events);
            health.units = health.max.min(12);
            game_over.active = false;
            *choice = GameOverChoice::None;
            continue;
        }
        if health.units == 0 {
            if swim.active || grounded(collision.as_deref(), player.pos) {
                game_over.active = true;
                events.list.push(SimEvent::GameOver);
            }
            continue;
        }
        if swim.active && swim.air == 0 {
            health.damage(1);
            if health.units > 0 {
                respawn(player, &area, &mut swim, &mut roll, &mut hop, &mut events);
            }
        }
    }
}

fn respawn(
    player: &mut Player,
    area: &Area,
    swim: &mut Swim,
    roll: &mut Roll,
    hop: &mut Hop,
    events: &mut SimEvents,
) {
    player.pos = area.spawn;
    player.yaw = area.yaw;
    player.speed = 0.0;
    player.speed_y = 0.0;
    swim.active = false;
    swim.surfaced = false;
    swim.air = 0;
    roll.active = false;
    roll.frame = 0.0;
    hop.active = false;
    events.list.push(SimEvent::Restart);
}
