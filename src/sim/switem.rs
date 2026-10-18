use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::Vec3;

use super::actor::Actor;
use super::arrow;
use super::barricade::{AT_BOMB, AT_DARKNUT_SWORD, AT_FIRE_ARROW, AT_MACHETE, AT_SWORD};
use super::bombflower;
use super::boomerang;
use super::combat::Combat;
use super::events::{SimEvent, SimEvents};
use super::hammer;
use super::hammerhit;
use super::health::Health;
use super::hookshot;
use super::pickup::{ITEM_ACTORS, Pickup};
use super::player::Player;
use super::walk;

pub const SWITEM_ACTORS: [&str; 1] = ["ITat00"];

pub const AT_SHIFT: u32 = 0x00;
pub const AT_MASK: u32 = 0xFF;
pub const TABLE_SHIFT: u32 = 0x08;
pub const TABLE_MASK: u32 = 0x3F;
pub const BITNO_SHIFT: u32 = 0x0E;
pub const BITNO_MASK: u32 = 0x7F;

pub const TRIGGER_ANY: u8 = 0xFF;
pub const TRIGGER_MELEE: u8 = 0;
pub const TRIGGER_BOMB: u8 = 1;
pub const TRIGGER_BOOMERANG: u8 = 2;
pub const TRIGGER_HAMMER: u8 = 3;
pub const TRIGGER_ARROW: u8 = 4;
pub const TRIGGER_HOOKSHOT: u8 = 5;

pub const AT_BOOMERANG: u32 = 1 << 6;
pub const AT_BOKO_STICK: u32 = 1 << 7;
pub const AT_NORMAL_ARROW: u32 = 1 << 14;
pub const AT_HOOKSHOT: u32 = 0x8000;
pub const AT_ICE_ARROW: u32 = 1 << 19;
pub const AT_LIGHT_ARROW: u32 = 1 << 20;
pub const AT_STALFOS_MACE: u32 = 1 << 24;

pub const MELEE_AT: u32 =
    AT_SWORD | AT_BOKO_STICK | AT_STALFOS_MACE | AT_MACHETE | AT_DARKNUT_SWORD;
pub const ARROW_AT: u32 = AT_FIRE_ARROW | AT_ICE_ARROW | AT_LIGHT_ARROW | AT_NORMAL_ARROW;

pub const CYL_RADIUS: f32 = 25.0;
pub const CYL_HEIGHT: f32 = 50.0;

pub const TABLE_LOW: u8 = 0x20;
pub const TABLE_HIGH: u8 = 0x3E;
pub const TABLE_NONE: u8 = 0x3F;

pub const ITEM_HEART: u8 = 0x00;
pub const ITEM_TRIPLE_HEART: u8 = 0x1E;

pub const RUPEE_GREEN: u8 = 0x01;
pub const RUPEE_BLUE: u8 = 0x02;
pub const RUPEE_YELLOW: u8 = 0x03;
pub const RUPEE_RED: u8 = 0x04;
pub const RUPEE_PURPLE: u8 = 0x05;
pub const RUPEE_ORANGE: u8 = 0x06;
pub const RUPEE_SILVER: u8 = 0x0F;

pub const LIFE_MASK: u8 = 0xFC;
pub const SE_ITEM_OUT: u32 = 0x69ED;
pub const SE_LUPY_OUT: u32 = 0x69E9;

const SWORD_OFFSET: f32 = 20.0;
const SWORD_REACH: f32 = 20.0;
const CUT_FIRST: u32 = 1;
const CUT_LAST: u32 = 6;
const HOOK_STEP: f32 = hookshot::SHOT_STEP;

const ITEM_PARAM_BIT_SHIFT: u32 = 0x08;
const ITEM_PARAM_SWITCH2: u32 = 0xFF;
const ITEM_PARAM_SWITCH2_SHIFT: u32 = 0x10;
const ITEM_PARAM_TYPE: u32 = 0;
const ITEM_PARAM_ACT: u32 = 1;
const ITEM_PARAM_TYPE_SHIFT: u32 = 0x18;
const ITEM_PARAM_TYPE_MASK: u32 = 0x03;
const ITEM_PARAM_ACT_SHIFT: u32 = 0x1A;
const ITEM_PARAM_ACT_MASK: u32 = 0x3F;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum At {
    Idle,
    Sword,
    Hammer,
    Hook,
    Arrow,
    FireArrow,
    IceArrow,
    LightArrow,
    Boomerang,
    Bomb,
}

impl At {
    fn mask(self) -> u32 {
        match self {
            Self::Sword => AT_SWORD,
            Self::Hammer => hammer::AT_SKULL_HAMMER,
            Self::Hook => AT_HOOKSHOT,
            Self::Arrow => AT_NORMAL_ARROW,
            Self::FireArrow => AT_FIRE_ARROW,
            Self::IceArrow => AT_ICE_ARROW,
            Self::LightArrow => AT_LIGHT_ARROW,
            Self::Boomerang => AT_BOOMERANG,
            Self::Bomb => AT_BOMB,
            Self::Idle => 0,
        }
    }

    fn reach(self) -> f32 {
        match self {
            Self::Sword => SWORD_REACH,
            Self::Hammer => hammer::AT_RADIUS,
            Self::Hook => hookshot::CAPSULE_RADIUS,
            Self::Arrow | Self::FireArrow | Self::IceArrow | Self::LightArrow => {
                arrow::CAPSULE_RADIUS
            }
            Self::Boomerang => boomerang::CAPSULE_RADIUS,
            Self::Bomb => bombflower::BLAST_RADIUS,
            Self::Idle => 0.0,
        }
    }

    fn shaft(self) -> bool {
        matches!(
            self,
            Self::Arrow | Self::FireArrow | Self::IceArrow | Self::LightArrow
        )
    }
}

#[derive(SystemParam)]
pub struct Watch<'w, 's> {
    combat: Res<'w, Combat>,
    cut: Res<'w, hammer::Cut>,
    blasts: Res<'w, bombflower::Blasts>,
    players: Query<'w, 's, &'static Player>,
    arrows: Query<'w, 's, &'static arrow::Arrow>,
    bolts: Query<'w, 's, &'static boomerang::Boomerang>,
    chains: Query<'w, 's, &'static hookshot::Hookshot>,
    hammers: Query<'w, 's, &'static hammer::Hammer>,
}

/// `mCyl.GetTgHitObj()` (`tww/src/d/actor/d_a_switem.cpp:83`) hands back exactly one object per
/// tick, so the sim collapses every live AT emitter down to the single highest-priority source:
/// the order `iball::source_at` already uses (`src/sim/iball.rs:206`) with the bomb blast channel
/// in the `leaves::control` slot (`src/sim/leaves.rs:231`).
fn source_at(watch: &Watch<'_, '_>) -> At {
    if watch
        .chains
        .iter()
        .any(|chain| chain.phase != hookshot::Phase::Wait)
    {
        return At::Hook;
    }
    for shaft in watch.arrows.iter() {
        if shaft.phase != arrow::Phase::Flying {
            continue;
        }
        return match shaft.kind {
            arrow::TYPE_FIRE => At::FireArrow,
            arrow::TYPE_ICE => At::IceArrow,
            arrow::TYPE_LIGHT => At::LightArrow,
            _ => At::Arrow,
        };
    }
    if !watch.blasts.list.is_empty() {
        return At::Bomb;
    }
    if watch
        .bolts
        .iter()
        .any(|bolt| bolt.parameter == boomerang::MODE_THROW)
    {
        return At::Boomerang;
    }
    if watch.cut.kind != hammer::CUT_NONE && watch.hammers.iter().any(|head| head.striking()) {
        return At::Hammer;
    }
    if watch.combat.active && watch.combat.frame >= CUT_FIRST && watch.combat.frame < CUT_LAST {
        return At::Sword;
    }
    At::Idle
}

/// `cCcD_CylAttr::CalcAabBox` (`tww/src/SSystem/SComponent/c_cc_d.cpp:439`) spans the cylinder from
/// `GetCP()->y` to `GetCP()->y + GetH()`.
fn cyl_touches(base: Vec3, at: Vec3, reach: f32) -> bool {
    let dx = at.x - base.x;
    let dz = at.z - base.z;
    let span = CYL_RADIUS + reach;
    if span * span < dx * dx + dz * dz {
        return false;
    }
    at.y + reach >= base.y && at.y - reach <= base.y + CYL_HEIGHT
}

fn overlaps(at: At, base: Vec3, watch: &Watch<'_, '_>, player: &Player) -> bool {
    match at {
        At::Sword => {
            let tip = Vec3::new(
                player.pos.x + SWORD_OFFSET * player.yaw.sin(),
                player.pos.y,
                player.pos.z + SWORD_OFFSET * player.yaw.cos(),
            );
            cyl_touches(base, tip, at.reach())
        }
        At::Hammer => {
            hammerhit::reaches(base, player.pos)
                || cyl_touches(base, hammer::strike_origin(player), at.reach())
        }
        At::Hook => watch.chains.iter().any(|chain| {
            if chain.phase == hookshot::Phase::Wait {
                return false;
            }
            let span = CYL_RADIUS + at.reach();
            if walk::within(chain.pos, base, span) {
                return true;
            }
            chain.phase == hookshot::Phase::Shot
                && walk::swept(chain.pos, chain.pos + chain.heading * HOOK_STEP, base, span)
        }),
        At::Boomerang => watch.bolts.iter().any(|bolt| {
            bolt.parameter == boomerang::MODE_THROW
                && walk::within(bolt.pos, base, CYL_RADIUS + at.reach())
        }),
        At::Bomb => watch
            .blasts
            .list
            .iter()
            .any(|origin| walk::within(base, *origin, CYL_RADIUS + at.reach())),
        _ if at.shaft() => watch.arrows.iter().any(|shaft| {
            shaft.phase == arrow::Phase::Flying
                && walk::swept(
                    shaft.pos - shaft.speed,
                    shaft.pos,
                    base,
                    CYL_RADIUS + at.reach(),
                )
        }),
        _ => false,
    }
}

fn is_rupee(item: u8) -> bool {
    matches!(
        item,
        RUPEE_GREEN
            | RUPEE_BLUE
            | RUPEE_YELLOW
            | RUPEE_RED
            | RUPEE_PURPLE
            | RUPEE_ORANGE
            | RUPEE_SILVER
    )
}

fn life_percent(health: &Health) -> u8 {
    let max = i32::from(health.max & LIFE_MASK);
    if max <= 0 {
        return 0;
    }
    ((i32::from(health.units) * 100 / max) & 0xFF) as u8
}

/// `getItemNoByLife` (`tww/src/d/d_item.cpp:2860`): at full health a heart drops as a rupee.
fn item_no_by_life(item: u8, health: &Health) -> u8 {
    if life_percent(health) != 100 {
        return item;
    }
    if item == ITEM_HEART {
        return RUPEE_GREEN;
    }
    if item == ITEM_TRIPLE_HEART {
        return RUPEE_YELLOW;
    }
    item
}

/// `MAKE_ITEM_PARAMS(itemNo, itemBitNo, switchNo2, type, action)`
/// (`tww/src/f_op/f_op_actor_mng.cpp:31`) as `fopAcM_fastCreateItem2` applies it at line 1022,
/// with `switchNo2 = -1`, `daItemType_0_e` and `daItemAct_1_e`.
fn item_parameters(item: u8, bit_no: u8) -> u32 {
    (u32::from(item) & 0xFF)
        | ((u32::from(bit_no) & 0xFF) << ITEM_PARAM_BIT_SHIFT)
        | (ITEM_PARAM_SWITCH2 << ITEM_PARAM_SWITCH2_SHIFT)
        | ((ITEM_PARAM_TYPE & ITEM_PARAM_TYPE_MASK) << ITEM_PARAM_TYPE_SHIFT)
        | ((ITEM_PARAM_ACT & ITEM_PARAM_ACT_MASK) << ITEM_PARAM_ACT_SHIFT)
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Switem {
    pub at_type: u8,
    pub item_table: u8,
    pub item_bit_no: u8,
    pub spawned: bool,
}

impl Switem {
    pub fn spawn(parameters: u32) -> Self {
        Self {
            at_type: ((parameters >> AT_SHIFT) & AT_MASK) as u8,
            item_table: ((parameters >> TABLE_SHIFT) & TABLE_MASK) as u8,
            item_bit_no: ((parameters >> BITNO_SHIFT) & BITNO_MASK) as u8,
            spawned: false,
        }
    }

    /// The `switch (mAtTypeTrigger)` block of `daSwItem_c::_execute`
    /// (`tww/src/d/actor/d_a_switem.cpp:85`). `obj->ChkAtType(mask)` is `mObjAt.mType & mask`
    /// (`tww/include/SSystem/SComponent/c_cc_d.h:538`), so every case is a single mask test.
    fn fires(&self, at: At) -> bool {
        if at == At::Idle {
            return false;
        }
        match self.at_type {
            TRIGGER_ANY => true,
            TRIGGER_MELEE => at.mask() & MELEE_AT != 0,
            TRIGGER_BOMB => at.mask() & AT_BOMB != 0,
            TRIGGER_BOOMERANG => at.mask() & AT_BOOMERANG != 0,
            TRIGGER_HAMMER => at.mask() & hammer::AT_SKULL_HAMMER != 0,
            TRIGGER_ARROW => at.mask() & ARROW_AT != 0,
            TRIGGER_HOOKSHOT => at.mask() & AT_HOOKSHOT != 0,
            _ => false,
        }
    }

    /// `fopAcM_createItemFromTable` (`tww/src/f_op/f_op_actor_mng.cpp:805`) resolves
    /// `0x20..=0x3E` through `dComIfGp_getItemTable()`, whose `mpItemTable` pointer is only ever
    /// written by absent per-stage data (`tww/include/d/d_com_inf_game.h:632`), and refuses
    /// `0x3F` at line 892. Only a literal item number is provable here.
    fn created_item(&self, health: &Health) -> Option<u8> {
        if (TABLE_LOW..=TABLE_HIGH).contains(&self.item_table) || self.item_table == TABLE_NONE {
            return None;
        }
        Some(item_no_by_life(self.item_table, health))
    }

    /// `isRupee(item->getItemNo()) || isRupeeInAllCreateTable(itemTbl)`
    /// (`tww/src/d/actor/d_a_switem.cpp:144`). `isRupeeInAllCreateTable` reads the absent item
    /// table, but only ever for a table index, and `created_item` refuses those, so the second
    /// disjunct is always false on this path.
    fn created_sound(item: u8) -> u32 {
        if is_rupee(item) {
            SE_LUPY_OUT
        } else {
            SE_ITEM_OUT
        }
    }

    fn execute(
        &mut self,
        base: Vec3,
        home_angle: i16,
        room: i8,
        watch: &Watch<'_, '_>,
        player: &Player,
        health: &Health,
        events: &mut SimEvents,
        commands: &mut Commands,
    ) {
        let at = source_at(watch);
        if self.spawned || !self.fires(at) || !overlaps(at, base, watch, player) {
            return;
        }
        self.spawned = true;
        let Some(item) = self.created_item(health) else {
            return;
        };
        let parameters = item_parameters(item, self.item_bit_no);
        commands.spawn((
            Actor {
                name: ITEM_ACTORS[0].to_owned(),
                parameters,
                pos: base,
                angle: [0, home_angle, 0],
                set_id: 0,
                room,
                ..Default::default()
            },
            Pickup::spawn(parameters),
        ));
        events.list.push(SimEvent::Se(Self::created_sound(item)));
    }
}

pub fn is_switem(name: &str) -> bool {
    SWITEM_ACTORS.contains(&name)
}

pub fn control(
    watch: Watch<'_, '_>,
    health: Res<Health>,
    mut triggers: Query<(&Actor, &mut Switem)>,
    mut events: ResMut<SimEvents>,
    mut commands: Commands,
) {
    let Some(player) = watch.players.iter().next().copied() else {
        return;
    };
    for (actor, mut trigger) in &mut triggers {
        trigger.execute(
            actor.pos,
            actor.angle[1],
            actor.room,
            &watch,
            &player,
            &health,
            &mut events,
            &mut commands,
        );
    }
}
