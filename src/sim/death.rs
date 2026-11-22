use bevy_ecs::prelude::*;
use glam::Vec2;

use super::area::Area;
use super::ba1::{ITEM_BOTTLE_SLOT_0, ITEM_BOTTLE_SLOTS, ITEM_FAIRY_BOTTLE};
use super::collision::Collision;
use super::events::{SimEvent, SimEvents};
use super::ground::grounded;
use super::health::Health;
use super::hop::Hop;
use super::intent::{GameOverChoice, MoveIntent};
use super::inventory::Equipped;
use super::player::Player;
use super::roll::Roll;
use super::swim::Swim;
use super::talk::Dialogue;

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
    mut equipped: ResMut<Equipped>,
    mut swim: ResMut<Swim>,
    mut roll: ResMut<Roll>,
    mut hop: ResMut<Hop>,
    mut dialogue: ResMut<Dialogue>,
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
            respawn(
                player,
                &area,
                &mut swim,
                &mut roll,
                &mut hop,
                &mut dialogue,
                &mut events,
            );
            // A fairy bottle lets Continue revive at full life and is consumed, the port's
            // stand-in for the retail game-over fairy choice (`d_a_player_main.cpp:5006`,
            // `d_item.cpp:1011-1014`; HD `wwhd/wwhd_src/d/actor/d_a_player_main.cpp`).
            if let Some(slot) = (ITEM_BOTTLE_SLOT_0..ITEM_BOTTLE_SLOT_0 + ITEM_BOTTLE_SLOTS)
                .find(|&slot| equipped.item(slot) == ITEM_FAIRY_BOTTLE)
            {
                equipped.set(slot, super::ba1::ITEM_EMPTY_BOTTLE);
                health.units = health.max;
            } else {
                health.units = health.max.min(12);
            }
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
                respawn(
                    player,
                    &area,
                    &mut swim,
                    &mut roll,
                    &mut hop,
                    &mut dialogue,
                    &mut events,
                );
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
    dialogue: &mut Dialogue,
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
    *dialogue = Dialogue::default();
    events.list.push(SimEvent::Restart);
}
