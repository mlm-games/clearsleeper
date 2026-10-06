use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::collision::Collision;
use super::events::{SimEvent, SimEvents};
use super::ground;
use super::health::Health;
use super::inventory::{self, Equipped, Got};
use super::item::Held;
use super::pickup::Rupees;
use super::player::Player;
use super::sea::Sea;
use super::stream;
use super::walk;
use super::water::Water;

/// Stage object name (`tww/src/d/d_stage.cpp:1255`). `daRaceItem_c` is
/// `fpcNm_RACEITEM_e`, listed `0x0007`, draw prio `fpcDwPi_RACEITEM_e`
/// (`d_a_race_item.cpp:277-292`); its archive is per item
/// (`dItem_data::getFieldArc(m_itemNo)`, `d_a_race_item.cpp:69`), so the stage
/// string is the only stable name.
pub const RITEM_ACTORS: [&str; 1] = ["Ritem"];

const CULL_MIN: [f32; 3] = [-40.0, 0.0, -40.0]; // <- fopAc_CULLBOX_0_e (f_op_actor_mng.cpp:551)
const CULL_MAX: [f32; 3] = [40.0, 125.0, 40.0]; // <- fopAc_CULLBOX_0_e (f_op_actor_mng.cpp:551)

const ITEM_SHIFT: u32 = 0;
const ITEM_MASK: u32 = 0xFF;
const BIT_NO_SHIFT: u32 = 8;
const BIT_NO_MASK: u32 = 0x7F;
const KIND_SHIFT: u32 = 0xF;
const KIND_MASK: u32 = 0xF;

const KIND_NORMAL: u8 = 1;

const BOOMERANG_BIT: u8 = 0x01;

/// `s16 spin = 0xFFFF / daItemBase_c::m_data.mRotateYSpeed` (`d_a_race_item.cpp:143`), i.e.
/// 65535 / 82 (`d_a_itembase_static.cpp:22`).
const SPIN: i16 = 799;
const GET_TICKS: i32 = 0xD;
const CARRY_LIFT: f32 = 23.0;
const CARRY_GRAVITY: f32 = -6.0;
const FALL_FLOOR: f32 = -5000.0;
const MAX_FALL: f32 = -100.0;
const BOOMERANG_REACH: f32 = super::boomerang::CAPSULE_RADIUS;

/// `dItem_data::item_info[no].mCollisionH` (`d_item_data.cpp:8177-8440`).
const ITEM_HEIGHT: [u8; 256] = [
    40, 55, 55, 55, 55, 55, 55, 80, 80, 40, 80, 60, 60, 60, 60, 55, //
    80, 80, 80, 0, 0, 40, 60, 0, 0, 0, 0, 0, 0, 0, 40, 70, //
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 40, 0, 0, 0, //
    0, 0, 0, 0, 0, 0, 0, 0, 50, 0, 0, 60, 0, 50, 0, 0, //
    0, 0, 0, 0, 0, 90, 80, 90, 70, 70, 70, 70, 0, 0, 0, 40, //
    40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40, //
    40, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
    0, 0, 0, 0, 0, 0, 0, 0, 40, 40, 40, 40, 40, 40, 40, 40, //
    0, 0, 55, 40, 40, 40, 40, 40, 40, 40, 40, 0, 40, 40, 40, 40, //
    40, 40, 40, 40, 40, 40, 40, 40, 0, 0, 0, 0, 0, 0, 0, 0, //
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

/// `dItem_data::item_info[no].mCollisionR` (`d_item_data.cpp:8177-8440`).
const ITEM_RADIUS: [u8; 256] = [
    30, 35, 35, 35, 35, 35, 35, 50, 50, 30, 30, 30, 30, 30, 30, 35, //
    50, 50, 50, 0, 0, 30, 30, 0, 0, 0, 0, 0, 0, 0, 30, 35, //
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 40, 0, 0, 0, //
    0, 0, 0, 0, 0, 0, 0, 0, 30, 0, 0, 40, 0, 30, 0, 0, //
    0, 0, 0, 0, 0, 50, 40, 40, 40, 40, 40, 40, 0, 0, 0, 40, //
    40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40, //
    40, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
    0, 0, 0, 0, 0, 0, 0, 0, 40, 40, 40, 40, 40, 40, 40, 40, //
    0, 0, 35, 40, 40, 40, 40, 40, 40, 40, 40, 0, 40, 40, 40, 40, //
    40, 40, 40, 40, 40, 40, 40, 40, 0, 0, 0, 0, 0, 0, 0, 0, //
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

const ITEM_HEART: u8 = 0x00;
const ITEM_GREEN_RUPEE: u8 = 0x01;
const ITEM_BLUE_RUPEE: u8 = 0x02;
const ITEM_YELLOW_RUPEE: u8 = 0x03;
const ITEM_ORANGE_RUPEE: u8 = 0x06;
const ITEM_HEART_PIECE: u8 = 0x07;
const ITEM_HEART_CONTAINER: u8 = 0x08;
const ITEM_SMALL_MAGIC: u8 = 0x09;
const ITEM_LARGE_MAGIC: u8 = 0x0A;
const ITEM_BOMB_5: u8 = 0x0B;
const ITEM_BOMB_30: u8 = 0x0E;
const ITEM_SILVER_RUPEE: u8 = 0x0F;
const ITEM_ARROW_10: u8 = 0x10;
const ITEM_ARROW_30: u8 = 0x12;
const ITEM_TRIPLE_HEART: u8 = 0x1E;
const ITEM_BOOMERANG: u8 = 0x2D;
const ITEM_HOOKSHOT: u8 = 0x2F;

/// `dComIfGp_setItemRupeeCount` per `item_func_*_rupee` (`d_item.cpp:553-575, 646`).
const RUPEE_VALUES: [u32; 6] = [1, 5, 10, 20, 50, 100];
const SILVER_RUPEE_VALUE: u32 = 200;
const HEART_UNITS: u8 = 4;
const MAX_LIFE: u8 = 0x50;

const SE_LUPY_GET: u32 = 0x0826;
const SE_CONSUMP_ITEM_GET: u32 = 0x0827;
const SE_BLUE_LUPY_GET: u32 = 0x0835;
const SE_RED_LUPY_GET: u32 = 0x0836;
const SE_MAGIC_POT_GET_S: u32 = 0x0879;
const SE_MAGIC_POT_GET_L: u32 = 0x087A;
const SE_HEART_PIECE: u32 = 0x0821;
const SE_FALL_WATER: u32 = 0x6918;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Rest,
    Get,
    Carry,
    Fall,
    Ground,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Ritem {
    pub item: u8,
    pub item_bit_no: u8,
    pub kind: u8,
    pub mode: Mode,
    pub countdown: i32,
    pub bits: u8,
    pub speed: Vec3,
    pub gravity: f32,
}

fn item_height(item: u8) -> f32 {
    f32::from(ITEM_HEIGHT.get(usize::from(item)).copied().unwrap_or(0))
}

fn item_radius(item: u8) -> f32 {
    f32::from(ITEM_RADIUS.get(usize::from(item)).copied().unwrap_or(0))
}

/// `cCcD_CylAttr::CalcAabBox` (`tww/src/SSystem/SComponent/c_cc_d.cpp:439`) spans the cylinder
/// from `GetCP()->y` to `GetCP()->y + GetH()`; `CreateInit` sets the height from
/// `dItem_data::getH` and the radius from `dItem_data::getR` (`d_a_race_item.cpp:91-98`), and
/// `scale.x` is 1.0 for a stage-placed actor so neither is scaled.
fn cyl_touches(base: Vec3, at: Vec3, radius: f32, height: f32, reach: f32) -> bool {
    let dx = at.x - base.x;
    let dz = at.z - base.z;
    let span = radius + reach;
    if span * span < dx * dx + dz * dz {
        return false;
    }
    at.y + reach >= base.y && at.y - reach <= base.y + height
}

fn timer(value: &mut i32) -> i32 {
    if *value != 0 {
        *value -= 1;
    }
    *value
}

impl Ritem {
    pub fn spawn(parameters: u32) -> Self {
        Self {
            item: ((parameters >> ITEM_SHIFT) & ITEM_MASK) as u8,
            item_bit_no: ((parameters >> BIT_NO_SHIFT) & BIT_NO_MASK) as u8,
            kind: ((parameters >> KIND_SHIFT) & KIND_MASK) as u8,
            mode: Mode::Rest,
            countdown: 0,
            bits: 0,
            speed: Vec3::ZERO,
            gravity: 0.0,
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `normalItemGet`'s `switch (m_itemNo)` sound table (`d_a_race_item_static.cpp:68-108`);
    /// the decomp has no `default:` arm, so anything else is silent.
    fn item_sound(item: u8) -> Option<u32> {
        match item {
            ITEM_GREEN_RUPEE => Some(SE_LUPY_GET),
            ITEM_BLUE_RUPEE => Some(SE_BLUE_LUPY_GET),
            ITEM_YELLOW_RUPEE..=ITEM_ORANGE_RUPEE => Some(SE_RED_LUPY_GET),
            ITEM_SMALL_MAGIC => Some(SE_MAGIC_POT_GET_S),
            ITEM_LARGE_MAGIC => Some(SE_MAGIC_POT_GET_L),
            ITEM_BOMB_5..=ITEM_ARROW_30 => Some(SE_CONSUMP_ITEM_GET),
            ITEM_TRIPLE_HEART => Some(SE_HEART_PIECE),
            _ => None,
        }
    }

    /// `normalItemGet` (`d_a_race_item_static.cpp:61-109`): `field_0x640 = 1` then
    /// `execItemGet(m_itemNo)` (`d_item.cpp:533-536`) then the item-get sound.
    fn normal_item_get(
        &mut self,
        health: &mut Health,
        rupees: &mut Rupees,
        held: &mut Held,
        equipped: &mut Equipped,
        got: &mut Got,
        events: &mut SimEvents,
    ) {
        self.mode = Mode::Get;
        let item = self.item;
        match item {
            ITEM_HEART => health.heal(HEART_UNITS),
            ITEM_GREEN_RUPEE..=ITEM_ORANGE_RUPEE => {
                let value = RUPEE_VALUES
                    .get(usize::from(item - ITEM_GREEN_RUPEE))
                    .copied()
                    .unwrap_or(0);
                rupees.count = rupees.count.saturating_add(value);
            }
            ITEM_HEART_PIECE => {
                health.max = health.max.saturating_add(1).min(MAX_LIFE);
            }
            ITEM_HEART_CONTAINER => {
                health.max = health.max.saturating_add(HEART_UNITS).min(MAX_LIFE);
                health.units = health.units.min(health.max / HEART_UNITS * HEART_UNITS);
            }
            ITEM_BOMB_5..=ITEM_BOMB_30 | ITEM_ARROW_10..=ITEM_ARROW_30 => held.equip(item),
            ITEM_SILVER_RUPEE => rupees.count = rupees.count.saturating_add(SILVER_RUPEE_VALUE),
            ITEM_BOOMERANG | ITEM_HOOKSHOT => held.equip(item),
            _ => {}
        }
        if let Some(slot) = inventory::slot_of(item) {
            got.gain(slot, 0);
        }
        equipped.collect(item);
        if let Some(sound) = Self::item_sound(item) {
            events.list.push(SimEvent::Se(sound));
        }
    }

    /// `checkGet` (`d_a_race_item.cpp:112-116`).
    fn check_get(
        &mut self,
        actor: &Actor,
        player: &Player,
        health: &mut Health,
        rupees: &mut Rupees,
        held: &mut Held,
        equipped: &mut Equipped,
        got: &mut Got,
        events: &mut SimEvents,
    ) {
        if self.kind != KIND_NORMAL {
            return;
        }
        if !cyl_touches(
            actor.pos,
            player.pos,
            item_radius(self.item),
            item_height(self.item),
            0.0,
        ) {
            return;
        }
        self.normal_item_get(health, rupees, held, equipped, got, events);
    }

    /// `fopAcM_posMoveF(this, mStts.GetCCMoveP())` (`d_a_race_item.cpp:198`) is
    /// `fopAcM_calcSpeed` + `fopAcM_posMove` (`f_op_actor_mng.cpp:454-484`); `mAcch.CrrPos` is the
    /// merged-DZB ground test.
    fn integrate(&mut self, actor: &mut Actor, solid: Option<&Collision>) {
        let (travel, _) =
            stream::calc_speed(0.0, actor.angle[1], self.speed.y, self.gravity, MAX_FALL);
        self.speed = travel;
        actor.pos += travel;
        if let Some(floor) = ground::surface(solid, actor.pos)
            && actor.pos.y < floor
        {
            actor.pos.y = floor;
            self.speed.y = 0.0;
        }
    }

    /// `case 4:` (`d_a_race_item.cpp:195-221`) - `case 3:` sets `field_0x640 = 4` and falls
    /// straight through into it. Returns true when the actor asked to be deleted.
    fn ground_step(
        &mut self,
        actor: &mut Actor,
        player: &Player,
        solid: Option<&Collision>,
        wet: bool,
        sea: Option<&Sea>,
        health: &mut Health,
        rupees: &mut Rupees,
        held: &mut Held,
        equipped: &mut Equipped,
        got: &mut Got,
        events: &mut SimEvents,
    ) -> bool {
        self.check_get(actor, player, health, rupees, held, equipped, got, events);
        self.integrate(actor, solid);
        if self.bits & BOOMERANG_BIT == 0 {
            if wet {
                events.list.push(SimEvent::Se(SE_FALL_WATER));
                return true;
            }
            if sea.is_some_and(|sea| sea.draw_contains(actor.pos.x, actor.pos.z)) {
                let wave = sea.map_or(0.0, |sea| sea.height_at(actor.pos.x, actor.pos.z));
                if wave != 0.0 && actor.pos.y - wave < 0.0 {
                    return true;
                }
            }
        }
        actor.pos.y < FALL_FLOOR
    }

    /// `case 1:` (`d_a_race_item.cpp:150-172`) falls through into `case 2:`
    /// (`d_a_race_item.cpp:173-192`). The head-top tracking of both cases needs
    /// `daPy_py_c::getHeadTopPos` (`d_a_player.h:509`), a joint-matrix value, so only the
    /// countdown and the delete remain.
    fn carry_step(&mut self) -> bool {
        timer(&mut self.countdown) < 0
    }
}

pub fn is_ritem(name: &str) -> bool {
    RITEM_ACTORS.contains(&name)
}

pub fn control(
    collision: Option<Res<Collision>>,
    wet: Res<Water>,
    sea: Option<Res<Sea>>,
    mut health: ResMut<Health>,
    mut rupees: ResMut<Rupees>,
    mut held: ResMut<Held>,
    mut equipped: ResMut<Equipped>,
    mut got: ResMut<Got>,
    players: Query<&Player>,
    bolts: Query<&super::boomerang::Boomerang>,
    mut events: ResMut<SimEvents>,
    mut items: Query<(Entity, &mut Actor, &mut Ritem)>,
    mut commands: Commands,
) {
    let solid = collision.as_deref();
    let sea = sea.as_deref();
    let soaked = wet.in_area;
    let Some(player) = players.iter().next().copied() else {
        return;
    };
    let mut gone: Vec<Entity> = Vec::new();
    for (entity, mut actor, mut ritem) in &mut items {
        let actor = &mut *actor;
        let ritem = &mut *ritem;
        let bolt = bolts
            .iter()
            .find(|bolt| bolt.parameter == super::boomerang::MODE_THROW);
        if ritem.kind == KIND_NORMAL
            && let Some(bolt) = bolt
            && walk::within(actor.pos, bolt.pos, item_radius(ritem.item) + BOOMERANG_REACH)
        {
            ritem.bits |= BOOMERANG_BIT;
        }
        if ritem.bits & BOOMERANG_BIT != 0 {
            match bolt {
                Some(bolt) => actor.pos = bolt.pos,
                None => ritem.bits &= !BOOMERANG_BIT,
            }
        }
        let deleted = match ritem.mode {
            Mode::Rest => {
                ritem.check_get(
                    actor,
                    &player,
                    &mut health,
                    &mut rupees,
                    &mut held,
                    &mut equipped,
                    &mut got,
                    &mut events,
                );
                actor.angle[1] = actor.angle[1].wrapping_add(SPIN);
                false
            }
            Mode::Get => {
                ritem.countdown = GET_TICKS;
                ritem.speed = Vec3::new(0.0, CARRY_LIFT, 0.0);
                ritem.gravity = CARRY_GRAVITY;
                ritem.bits &= !BOOMERANG_BIT;
                ritem.mode = Mode::Carry;
                ritem.carry_step()
            }
            Mode::Carry => ritem.carry_step(),
            Mode::Fall => {
                ritem.mode = Mode::Ground;
                ritem.ground_step(
                    actor,
                    &player,
                    solid,
                    soaked,
                    sea,
                    &mut health,
                    &mut rupees,
                    &mut held,
                    &mut equipped,
                    &mut got,
                    &mut events,
                )
            }
            Mode::Ground => ritem.ground_step(
                actor,
                &player,
                solid,
                soaked,
                sea,
                &mut health,
                &mut rupees,
                &mut held,
                &mut equipped,
                &mut got,
                &mut events,
            ),
        };
        if deleted {
            gone.push(entity);
        }
    }
    for entity in gone {
        let _ = commands.entity(entity).despawn();
    }
}