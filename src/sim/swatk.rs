use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::Vec3;

use super::actor::Actor;
use super::arrow;
use super::barricade::{AT_BOMB, AT_DARKNUT_SWORD, AT_FIRE_ARROW, AT_MACHETE, AT_SWORD};
use super::bombflower;
use super::boomerang;
use super::combat::Combat;
use super::flags::Switches;
use super::hammer;
use super::hammerhit;
use super::hookshot;
use super::player::Player;
use super::walk;

pub const SWAT_ACTORS: [&str; 1] = ["SWat00"];

pub const AT_SHIFT: u32 = 0x00;
pub const AT_MASK: u32 = 0xFF;
pub const SWITCH_SHIFT: u32 = 0x08;
pub const SWITCH_MASK: u32 = 0xFF;

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

const SWORD_OFFSET: f32 = 20.0;
const SWORD_REACH: f32 = 20.0;
const CUT_FIRST: u32 = 1;
const CUT_LAST: u32 = 6;
const HOOK_STEP: f32 = hookshot::SHOT_STEP;

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

/// `mCyl.GetTgHitObj()` (`tww/src/d/actor/d_a_swattack.cpp:70`) hands back exactly one object per
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

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Swatk {
    pub at_type: u8,
    pub switch_no: u8,
}

impl Swatk {
    pub fn spawn(parameters: u32) -> Self {
        Self {
            at_type: ((parameters >> AT_SHIFT) & AT_MASK) as u8,
            switch_no: ((parameters >> SWITCH_SHIFT) & SWITCH_MASK) as u8,
        }
    }

    /// The `switch (mAtType)` block of `daSwAt_c::_execute`
    /// (`tww/src/d/actor/d_a_swattack.cpp:72`). `obj->ChkAtType(mask)` is `mObjAt.mType & mask`
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

    fn execute(&self, base: Vec3, watch: &Watch<'_, '_>, player: &Player) -> bool {
        let at = source_at(watch);
        self.fires(at) && overlaps(at, base, watch, player)
    }
}

pub fn is_swatk(name: &str) -> bool {
    SWAT_ACTORS.contains(&name)
}

pub fn control(
    watch: Watch<'_, '_>,
    triggers: Query<(&Actor, &Swatk)>,
    mut switches: ResMut<Switches>,
) {
    let Some(player) = watch.players.iter().next().copied() else {
        return;
    };
    for (actor, trigger) in &triggers {
        if trigger.execute(actor.pos, &watch, &player) {
            switches.set(trigger.switch_no);
        }
    }
}
