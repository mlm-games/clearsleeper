use bevy_ecs::prelude::*;

use super::actor::Actor;
use super::health::Health;
use super::item::{FIRST_ARROW, FIRST_BOMB, Held, LAST_ARROW, LAST_BOMB};
use super::player::Player;

pub const ITEM_ACTORS: [&str; 3] = ["item", "Sitem", "Bitem"];

const NO_GET_TICKS: u8 = 16;
const MAX_LIFE: u8 = 0x50;
const HEART_ITEMS: u8 = 0x00;
const FIRST_RUPEE: u8 = 0x01;
const LAST_RUPEE: u8 = 0x06;
const SILVER_RUPEE: u8 = 0x0F;
const HEART_PIECE: u8 = 0x07;
const HEART_CONTAINER: u8 = 0x08;
const RUPEE_VALUES: [u32; LAST_RUPEE as usize - FIRST_RUPEE as usize + 1] = [1, 5, 10, 20, 50, 100];

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rupees {
    pub count: u32,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pickup {
    pub item: u8,
    pub timer: u8,
}

pub fn is_pickup(name: &str) -> bool {
    ITEM_ACTORS.contains(&name)
}

impl Pickup {
    pub fn spawn(parameters: u32) -> Self {
        Self {
            item: (parameters & 0xFF) as u8,
            timer: 0,
        }
    }

    fn radius(&self) -> f32 {
        match self.item {
            HEART_PIECE | HEART_CONTAINER => 50.0,
            FIRST_RUPEE..=LAST_RUPEE | SILVER_RUPEE => 35.0,
            _ => 30.0,
        }
    }

    pub fn size(&self) -> f32 {
        match self.item {
            HEART_PIECE => 14.0,
            HEART_CONTAINER => 20.0,
            _ => 10.0,
        }
    }

    fn takes(&self) -> bool {
        matches!(
            self.item,
            HEART_ITEMS
                | FIRST_RUPEE..=LAST_RUPEE
                | SILVER_RUPEE
                | HEART_PIECE
                | HEART_CONTAINER
                | FIRST_BOMB..=LAST_BOMB
                | FIRST_ARROW..=LAST_ARROW
        )
    }
}

fn apply(item: u8, health: &mut Health, rupees: &mut Rupees, held: &mut Held) {
    match item {
        HEART_ITEMS => health.heal(4),
        FIRST_RUPEE..=LAST_RUPEE => {
            rupees.count = rupees
                .count
                .saturating_add(RUPEE_VALUES[(item - FIRST_RUPEE) as usize]);
        }
        SILVER_RUPEE => rupees.count = rupees.count.saturating_add(200),
        HEART_PIECE => {
            health.max = health.max.saturating_add(1).min(MAX_LIFE);
            health.units = health.units.min(heart_floor(health.max));
        }
        HEART_CONTAINER => {
            health.max = health.max.saturating_add(4).min(MAX_LIFE);
            health.units = health.max;
        }
        FIRST_BOMB..=LAST_BOMB | FIRST_ARROW..=LAST_ARROW => held.equip(item),
        _ => {}
    }
}

fn heart_floor(max: u8) -> u8 {
    max / 4 * 4
}

pub fn control(
    mut health: ResMut<Health>,
    mut rupees: ResMut<Rupees>,
    mut held: ResMut<Held>,
    players: Query<&Player>,
    mut items: Query<(Entity, &mut Pickup, &Actor)>,
    mut commands: Commands,
) {
    let Some(player) = players.iter().next() else {
        return;
    };
    if health.units == 0 {
        return;
    }
    for (entity, mut pickup, actor) in &mut items {
        pickup.timer = pickup.timer.saturating_add(1).min(NO_GET_TICKS);
        if pickup.timer < NO_GET_TICKS || !pickup.takes() {
            continue;
        }
        let dx = player.pos.x - actor.pos.x;
        let dz = player.pos.z - actor.pos.z;
        let radius = pickup.radius();
        if dx * dx + dz * dz > radius * radius {
            continue;
        }
        apply(pickup.item, &mut health, &mut rupees, &mut held);
        let _ = commands.entity(entity).despawn();
    }
}
