use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::bodies::AtMeta;
use super::collision::Collision;
use super::contact_run::ContactBodies;
use super::damage;
use super::damage_resolve::{
    AT_SPL_UNK0, AT_SPL_UNK1, AT_TYPE_SWORD, Attacker, CUT_TYPE_CUT_A, CUT_TYPE_CUT_EA,
    CUT_TYPE_CUT_EB, CUT_TYPE_CUT_F, CUT_TYPE_CUT_L, CUT_TYPE_CUT_R, HitAudio, HitStop,
    NAME_PLAYER, PlayerCut, Target,
};
use super::enemy;
use super::events::{SimEvent, SimEvents};
use super::ground::grounded;
use super::health::Health;
use super::hop::Hop;
use super::intent::{CameraYaw, ItemIntent, MoveIntent};
use super::jump::{self, Jump};
use super::lock::Lock;
use super::player::Player;
use super::registry::{ActorRegistry, ERROR_PROC_ID};
use super::roll::Roll;
use super::shape::Shape;
use super::ship::Ship;
use super::swim::Swim;
use super::walk::{add_calc, from_s16, target_yaw, to_s16, turn_toward};

/// The AT/TG pass every swing runs. It lives in a file of its own and is declared here, so the
/// four damage sites share one module without the parent's module list having to name it;
use crate::sim::contact_hit::{AT_SPRM_VS_ENEMY, AtObj, TgObj};

const STICK_GATE: f32 = 0.05;
const DAMAGE_START: u32 = 1;
const DAMAGE_END: u32 = 6;
const LUNGE_FRAME: u32 = 2;
const CHAIN_FRAME: u32 = 11;
const LUNGE_FACTOR: f32 = 0.2;
const DECEL_SCALE: f32 = 0.7;
const DECEL_MIN: f32 = 0.5;
const SWORD_OFFSET: f32 = 20.0;
/// `l_at_cps_src`'s capsule radius (`player_main_data.inc:159`), the one value of the sword's AT
/// volume the reference states as a number; the segment it runs along comes from the sword
/// model's own joints, which the port does not carry, so the reach is this radius about the tip
/// the swing puts `SWORD_OFFSET` ahead of the player.
const SWORD_RADIUS: f32 = 20.0;
const FLINCH_TICKS: u32 = 10;
/// `dCcG_SE_UNK1` (`d_cc_d.h:66`), the `SrcGObjAt Se` every sword cut's AT object carries: the
/// hit-sound ladder reads it as neither wood, nor metal, nor arrow, nor hookshot.
const AT_SE_SWORD: u8 = 1;
/// `cCcD_Stts::Init`'s first argument for the player's own status, `player_main.cpp:12184`.
const AT_WEIGHT: u8 = 120;
/// The chain step the reference's cut-count field holds on the fourth cut, the finisher and the
/// only one of the four the `mHitSoundId` ladder gives a non-zero selector.
const CUT_COUNT_FINISH: u8 = 4;

/// The struck actor's status word, read through the process id the registry holds for it the way
/// the reference reads it off the actor. An actor the registry has no entry for carries no bits.
pub fn actor_status(registry: &ActorRegistry, entity: Entity) -> u32 {
    registry
        .proc_id(entity)
        .and_then(|proc_id| registry.status(proc_id))
        .unwrap_or(0)
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CutKind {
    #[default]
    A,
    F,
    R,
    L,
    EA,
    EB,
}

fn cut(kind: CutKind) -> (u32, f32, f32) {
    match kind {
        CutKind::A => (13, 10.0, 2.6),
        CutKind::F => (13, 8.0, 0.95),
        CutKind::R => (13, 1.0, 0.95),
        CutKind::L => (12, 1.0, 0.95),
        CutKind::EA => (15, 15.0, 0.95),
        CutKind::EB => (17, 7.0, 0.95),
    }
}

fn is_finisher(kind: CutKind) -> bool {
    kind == CutKind::EA || kind == CutKind::EB
}

/// The process the reference stands up for each cut, which is the state the blade's own tick
/// runs under: `CUT_AIR` admits exactly these, so the cut only advances while the player is in
/// one of them.
fn cut_proc(kind: CutKind) -> super::link_proc::Proc {
    use super::link_proc::Proc;
    match kind {
        CutKind::A => Proc::CutA,
        CutKind::F => Proc::CutF,
        CutKind::R => Proc::CutR,
        CutKind::L => Proc::CutL,
        CutKind::EA => Proc::CutEa,
        CutKind::EB => Proc::CutEb,
    }
}

/// `setNormalCutAtParam` and `setFinishCutAtParam` both branch on `checkNormalSwordEquip`,
/// which reads `dComIfGs_getSelectEquip(0)`: one heart for Hero's Sword, two for the Master
/// Sword, and the finisher doubles it. `inventory::sword` answers with the item the collect bit
/// names, so `selected` is that sword's id.
fn sword_at_power(selected: u8, finisher: bool) -> u8 {
    let normal = selected == damage::ITEM_SWORD;
    match (normal, finisher) {
        (true, false) => 1,
        (false, false) => 2,
        (true, true) => 2,
        (false, true) => 4,
    }
}

/// The player's cut type each cut kind carries. The reference writes the same six cut types onto
/// both the normal-cut and the finish-cut AT parameters, so the mapping is one to one.
fn cut_type(kind: CutKind) -> u8 {
    match kind {
        CutKind::A => CUT_TYPE_CUT_A,
        CutKind::F => CUT_TYPE_CUT_F,
        CutKind::R => CUT_TYPE_CUT_R,
        CutKind::L => CUT_TYPE_CUT_L,
        CutKind::EA => CUT_TYPE_CUT_EA,
        CutKind::EB => CUT_TYPE_CUT_EB,
    }
}

/// The AT special parameter: the reference's normal cuts take `dCcG_At_Spl_UNK0` and its finish
/// cuts take `dCcG_At_Spl_UNK1`, the value that marks the hit a critical one.
fn cut_spl(kind: CutKind) -> u8 {
    if is_finisher(kind) {
        AT_SPL_UNK1
    } else {
        AT_SPL_UNK0
    }
}

/// The cut count the hit-sound selector ladder reads: the chain step, which the reference sets to
/// four on the fourth cut, the finisher.
fn cut_count(kind: CutKind, combo: u32) -> u8 {
    if is_finisher(kind) {
        CUT_COUNT_FINISH
    } else {
        u8::try_from(combo).unwrap_or(0)
    }
}

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct Combat {
    pub active: bool,
    pub soup: bool,
    pub frame: u32,
    pub combo: u32,
    pub kind: CutKind,
    heading: i16,
    queued: bool,
    held: bool,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct HitPoints(pub i32);

impl Default for HitPoints {
    fn default() -> Self {
        Self(1)
    }
}

fn start(
    combat: &mut Combat,
    player: &mut Player,
    intent: &MoveIntent,
    camera: &CameraYaw,
    lock: &Lock,
    proc: Option<&mut super::link_proc::LinkProc>,
) {
    combat.combo += 1;
    let stick = intent.axis.length().min(1.0);
    let desired = target_yaw(intent, camera);
    let dir = if stick <= STICK_GATE {
        None
    } else {
        Some(jump::direction(desired, to_s16(player.face)))
    };
    combat.kind = if combat.combo >= 4 {
        combat.combo = 0;
        match dir {
            Some(jump::Direction::Forward) | Some(jump::Direction::Right) => CutKind::EA,
            _ => CutKind::EB,
        }
    } else {
        match dir {
            Some(jump::Direction::Forward) => CutKind::F,
            Some(jump::Direction::Right) => CutKind::R,
            None if lock.target.is_some() => CutKind::A,
            _ => CutKind::L,
        }
    };
    combat.heading = if lock.target.is_some() || stick <= STICK_GATE {
        to_s16(player.face)
    } else {
        desired
    };
    combat.active = true;
    combat.frame = 1;
    combat.queued = false;
    if let Some(state) = proc {
        state.enter(cut_proc(combat.kind));
    }
    player.yaw = player.face;
}

#[derive(bevy_ecs::system::SystemParam)]
pub struct Sword<'w, 's> {
    collision: Option<Res<'w, Collision>>,
    health: Res<'w, Health>,
    swim: Res<'w, Swim>,
    roll: Res<'w, Roll>,
    hop: Res<'w, Hop>,
    jump: Res<'w, Jump>,
    lock: Res<'w, Lock>,
    held: Res<'w, super::item::Held>,
    equipped: Res<'w, super::inventory::Equipped>,
    collect: Res<'w, super::flags::Collect>,
    proc: Option<ResMut<'w, super::link_proc::LinkProc>>,
    ships: Query<'w, 's, &'static Ship>,
}

#[derive(bevy_ecs::system::SystemParam)]
pub struct Outcome<'w> {
    combat: ResMut<'w, Combat>,
    zones: ResMut<'w, super::flags::ZoneActors>,
    events: ResMut<'w, SimEvents>,
    stop: ResMut<'w, HitStop>,
    audio: ResMut<'w, HitAudio>,
    slain: ResMut<'w, Slain>,
}

/// The actors a swing killed this step. The reference's actor plays its own pop and then deletes
/// itself (`bb_fail_move`, `d_a_bb.cpp:1966-1970`, reached from the faint flag `damage_check` sets
/// at `:2130`), so the module that owns the death gets the rest of the step to see
/// `HitPoints <= 0`; [`reap`] removes what is left standing at its end.
#[derive(Resource, Debug, Default)]
pub struct Slain {
    pub list: Vec<Entity>,
}

pub fn reap(mut slain: ResMut<Slain>, mut commands: Commands) {
    for entity in slain.list.drain(..) {
        commands.entity(entity).try_despawn();
    }
}

pub fn control(
    item: Res<ItemIntent>,
    intent: Res<MoveIntent>,
    camera: Res<CameraYaw>,
    mut gates: Sword<'_, '_>,
    mut actors: Query<
        (
            Entity,
            &Actor,
            Option<&ContactBodies>,
            Option<&mut HitPoints>,
            Option<&mut enemy::Hurt>,
        ),
        Without<Ship>,
    >,
    mut out: Outcome<'_>,
    mut players: Query<(Entity, &mut Player)>,
    registry: Res<ActorRegistry>,
    mut fx: ResMut<super::damage_fx::DamageFxQueue>,
) {
    let pressed = item.0 && !out.combat.held;
    out.combat.held = item.0;
    let riding = gates.ships.iter().any(|ship| ship.riding);
    let selected = super::inventory::sword(&gates.collect);
    let bare_hands =
        gates.equipped.item(super::inventory::HAMMER_SLOT) == super::inventory::ITEM_SKULL_HAMMER;
    for (player_entity, mut player) in &mut players {
        let player = &mut *player;
        if gates.swim.active || gates.health.units == 0 {
            out.combat.active = false;
            out.combat.combo = 0;
            out.combat.queued = false;
            continue;
        }
        if !out.combat.active {
            if !pressed
                || bare_hands
                || riding
                || gates.roll.active
                || gates.hop.active
                || gates.jump.active
                || !grounded(gates.collision.as_deref(), player.pos)
                || !super::link_proc::allows(gates.proc.as_deref(), super::link_proc::CUT_GROUND)
            {
                continue;
            }
            start(
                &mut out.combat,
                player,
                &intent,
                &camera,
                &gates.lock,
                gates.proc.as_deref_mut(),
            );
            out.events.list.push(SimEvent::Slash);
            continue;
        }
        if !super::link_proc::allows(gates.proc.as_deref(), super::link_proc::CUT_AIR) {
            continue;
        }
        let (duration, lunge_base, decel_max) = cut(out.combat.kind);
        if out.combat.frame >= duration {
            out.combat.active = false;
            out.combat.combo = 0;
            out.combat.queued = false;
            player.speed = 0.0;
            continue;
        }
        if out.combat.frame >= CHAIN_FRAME
            && out.combat.queued
            && grounded(gates.collision.as_deref(), player.pos)
        {
            player.speed = 0.0;
            start(
                &mut out.combat,
                player,
                &intent,
                &camera,
                &gates.lock,
                gates.proc.as_deref_mut(),
            );
            continue;
        }
        if pressed && !is_finisher(out.combat.kind) {
            out.combat.queued = true;
        }
        player.face = from_s16(turn_toward(to_s16(player.face), out.combat.heading));
        player.yaw = player.face;
        if out.combat.frame == LUNGE_FRAME {
            player.speed = LUNGE_FACTOR * player.speed.abs() + lunge_base;
        } else {
            add_calc(&mut player.speed, 0.0, DECEL_SCALE, decel_max, DECEL_MIN);
        }
        if out.combat.frame >= DAMAGE_START && out.combat.frame < DAMAGE_END {
            let kind = out.combat.kind;
            let swing = PlayerCut {
                cut_type: cut_type(kind),
                cut_count: cut_count(kind, out.combat.combo),
                pos: player.pos,
            };
            let at = AtObj::new(
                Shape::sphere(
                    player.pos
                        + Vec3::new(
                            SWORD_OFFSET * player.yaw.sin(),
                            0.0,
                            SWORD_OFFSET * player.yaw.cos(),
                        ),
                    SWORD_RADIUS,
                ),
                AT_SPRM_VS_ENEMY,
                &AtMeta {
                    at_type: AT_TYPE_SWORD,
                    at_se: AT_SE_SWORD,
                    at_spl: cut_spl(kind),
                    at_atp: sword_at_power(selected, is_finisher(kind)),
                },
                Attacker {
                    name: NAME_PLAYER.to_owned(),
                    pos: player.pos,
                    speed: Vec3::new(
                        player.speed * player.yaw.sin(),
                        player.speed_y,
                        player.speed * player.yaw.cos(),
                    ),
                    bomb_state: 0,
                    tsubo_type: 0,
                },
                AT_WEIGHT,
                registry.proc_id(player_entity).unwrap_or(ERROR_PROC_ID),
            );
            let crossings = {
                let tg: Vec<TgObj<'_>> = actors
                    .iter()
                    .filter_map(|(entity, actor, bodies, hp, _)| {
                        if hp.is_none() {
                            return None;
                        }
                        Some(TgObj::new(
                            entity,
                            bodies?,
                            actor.weight_uc,
                            registry.proc_id(entity),
                        ))
                    })
                    .collect();
                crate::sim::contact_hit::at_tg(std::slice::from_ref(&at), &tg)
            };
            let mut dead: Vec<(Entity, i8, u16)> = Vec::new();
            for crossing in crossings {
                let Ok((entity, actor, _, hp, mut hurt)) = actors.get_mut(crossing.target) else {
                    continue;
                };
                let Some(mut hp) = hp else {
                    continue;
                };
                let mut target = Target {
                    name: actor.name.clone(),
                    pos: actor.pos,
                    room: actor.room,
                    status: actor_status(&registry, entity),
                    health: *hp,
                    steal_item_left: actor.steal_item_left,
                    steal_item_bit_no: actor.steal_item_bit_no,
                    item_table_idx: actor.item_table_idx,
                    ..Default::default()
                };
                crate::sim::contact_hit::resolve(
                    &at.atk,
                    crossing.cross,
                    &mut target,
                    &swing,
                    &mut out.events,
                    &mut out.stop,
                    &mut out.audio,
                    &mut fx.items,
                );
                hp.0 = target.health.0;
                if let Some(hurt) = hurt.as_deref_mut() {
                    hurt.0 = FLINCH_TICKS;
                }
                out.events.list.push(SimEvent::Hit);
                if hp.0 <= 0 {
                    dead.push((entity, actor.room, actor.set_id));
                }
            }
            for (entity, room, set_id) in dead {
                out.zones.mark_done(room, set_id);
                out.slain.list.push(entity);
            }
        }
        out.combat.frame += 1;
    }
}
