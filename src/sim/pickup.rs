use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::events::{SimEvent, SimEvents};
use super::health::Health;
use super::item::{BOOMERANG, FIRST_ARROW, FIRST_BOMB, Held, LAST_ARROW, LAST_BOMB};
use super::player::Player;

pub const ITEM_ACTORS: [&str; 2] = ["item", "itemFLY"];

const NO_GET_TICKS: u8 = 16;
const MAX_LIFE: u8 = 0x50;
const HEART_ITEMS: u8 = 0x00;
const FIRST_RUPEE: u8 = 0x01;
const LAST_RUPEE: u8 = 0x06;
const SILVER_RUPEE: u8 = 0x0F;
const HEART_PIECE: u8 = 0x07;
const HEART_CONTAINER: u8 = 0x08;
const TRIPLE_HEART: u8 = 0x1E;
const RUPEE_VALUES: [u32; LAST_RUPEE as usize - FIRST_RUPEE as usize + 1] = [1, 5, 10, 20, 50, 100];

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rupees {
    pub count: u32,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Pickup {
    pub item: u8,
    pub timer: u8,
    pub speed_f: f32,
    pub speed_y: f32,
    pub gravity: f32,
}

pub fn is_pickup(name: &str) -> bool {
    ITEM_ACTORS.contains(&name)
}

impl Pickup {
    pub fn spawn(parameters: u32) -> Self {
        Self {
            item: (parameters & 0xFF) as u8,
            timer: 0,
            speed_f: 0.0,
            speed_y: 0.0,
            gravity: 0.0,
        }
    }

    /// `fopAcM_fastCreateItem`: the item's forward speed, vertical launch speed and gravity.
    /// Hearts double the forward speed before the create.
    pub fn thrown(parameters: u32, speed_f: f32, speed_y: f32, gravity: f32) -> Self {
        let item = (parameters & 0xFF) as u8;
        let speed_f = if is_heart(item) { 2.0 * speed_f } else { speed_f };
        Self {
            item,
            timer: 0,
            speed_f,
            speed_y,
            gravity,
        }
    }

    pub fn launch(&self) -> Vec3 {
        Vec3::new(self.speed_f.sin(), self.speed_y, self.speed_f.cos())
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
                | BOOMERANG
        ) || super::inventory::slot_of(self.item).is_some()
    }
}

fn is_heart(item: u8) -> bool {
    matches!(item, HEART_ITEMS | TRIPLE_HEART)
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
        FIRST_BOMB..=LAST_BOMB | FIRST_ARROW..=LAST_ARROW | BOOMERANG => held.equip(item),
        _ => {}
    }
}

fn heart_floor(max: u8) -> u8 {
    max / 4 * 4
}

pub fn control(
    do_intent: Res<super::intent::DoIntent>,
    mut health: ResMut<Health>,
    mut partner: ResMut<super::event::ItemPartner>,
    mut rupees: ResMut<Rupees>,
    mut held: ResMut<Held>,
    mut zones: ResMut<super::flags::ZoneActors>,
    mut equipped: ResMut<super::inventory::Equipped>,
    mut got: ResMut<super::inventory::Got>,
    mut chest_flags: ResMut<super::chest::Chests>,
    mut switches: ResMut<super::flags::Switches>,
    mut events: ResMut<SimEvents>,
    players: Query<&Player>,
    mut items: Query<(Entity, &mut Pickup, &Actor)>,
    mut chests: Query<(
        Entity,
        &Actor,
        &mut super::chest::Chest,
        Option<&super::chest::ChestOpen>,
    )>,
    mut commands: Commands,
) {
    partner.max_life = health.max;
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
        if let Some(slot) = super::inventory::slot_of(pickup.item) {
            got.gain(slot, 0);
        }
        equipped.collect(pickup.item);
        events.list.push(SimEvent::Collect);
        zones.mark_done(actor.room, actor.set_id);
        let _ = commands.entity(entity).despawn();
    }
    for (entity, actor, mut chest, marker) in &mut chests {
        let tapped = do_intent.0 && !chest.pressed;
        chest.pressed = do_intent.0;
        if chest.opened || !super::chest::present(&chest, &switches) {
            continue;
        }
        let forced = marker.is_some();
        if !forced {
            if !tapped {
                continue;
            }
            let dx = player.pos.x - actor.pos.x;
            let dz = player.pos.z - actor.pos.z;
            let reach = super::chest::GRAB_RANGE;
            if dx * dx + dz * dz > reach * reach {
                continue;
            }
        }
        if marker.is_some() {
            commands.entity(entity).remove::<super::chest::ChestOpen>();
        }
        chest.opened = true;
        chest_flags.open(super::chest::stage_of(&chest), chest.index);
        if chest.open_switch != super::flags::NO_SWITCH {
            switches.set(chest.open_switch);
        }
        super::chest::loot(&chest, &mut rupees, &mut equipped, &mut got);
        events.list.push(SimEvent::Collect);
    }
}
