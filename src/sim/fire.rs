use bevy_ecs::prelude::*;

use super::actor::Actor;
use super::enemy::Enemy;
use super::flags::Switches;
use super::health::Health;
use super::item::Bomb;
use super::player::Player;

pub const FIRE_ACTOR: &str = "Fire";

const SWITCH_BYTE: u32 = 0xFF;
const TYPE_SHIFT: u32 = 17;
const TYPE_MASK: u32 = 0x7;
const SWITCH_TYPE: u32 = 0;
const ROOM_TYPE: u32 = 1;
const REACH: f32 = 50.0;
const POWER: u8 = 1;
const LIGHT_RADIUS: f32 = 100.0;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fire {
    pub lit: bool,
    pub switch: u8,
    pub room: bool,
}

pub fn spawn(parameters: u32, switches: &Switches) -> Fire {
    let switch = (parameters & SWITCH_BYTE) as u8;
    let mut kind = (parameters >> TYPE_SHIFT) & TYPE_MASK;
    if kind == SWITCH_TYPE && switch == 0xFF {
        kind = 2;
    }
    Fire {
        lit: !(switch != 0xFF && switches.test(switch)),
        switch,
        room: kind == ROOM_TYPE,
    }
}

pub fn control(
    mut health: ResMut<Health>,
    switches: Res<Switches>,
    players: Query<&Player>,
    mut fires: Query<(&Actor, &mut Fire)>,
    bombs: Query<&Bomb>,
    foes: Query<(), With<Enemy>>,
) {
    let Some(player) = players.iter().next().copied() else {
        return;
    };
    let fighting = foes.iter().next().is_some();
    for (actor, mut fire) in &mut fires {
        if fire.switch != 0xFF && switches.test(fire.switch) {
            fire.lit = false;
            continue;
        }
        if fire.room && !fighting {
            fire.lit = false;
            continue;
        }
        if !fire.lit {
            let pos = actor.pos;
            if !bombs
                .iter()
                .any(|bomb| (bomb.pos - pos).length() <= LIGHT_RADIUS)
            {
                continue;
            }
            fire.lit = true;
        }
        let delta = player.pos - actor.pos;
        if delta.length() <= REACH {
            health.damage(POWER);
        }
    }
}
