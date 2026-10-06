use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::collision::Collision;
use super::damage;
use super::damage_resolve::{
    AT_SPL_UNK0, AT_SPL_UNK1, AT_TYPE_SWORD, CUT_TYPE_CUT_A, CUT_TYPE_CUT_EA,
    CUT_TYPE_CUT_EB, CUT_TYPE_CUT_F, CUT_TYPE_CUT_L, CUT_TYPE_CUT_R, Atk, Attacker, CcAtInfo,
    DamageFx, HitAudio, HitStop, NAME_PLAYER, PlayerCut, Target, cc_at_check,
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
use super::registry::ActorRegistry;
use super::roll::Roll;
use super::ship::Ship;
use super::swim::Swim;
use super::walk::{add_calc, from_s16, target_yaw, to_s16, turn_toward};

const STICK_GATE: f32 = 0.05;
const DAMAGE_START: u32 = 1;
const DAMAGE_END: u32 = 6;
const LUNGE_FRAME: u32 = 2;
const CHAIN_FRAME: u32 = 11;
const LUNGE_FACTOR: f32 = 0.2;
const DECEL_SCALE: f32 = 0.7;
const DECEL_MIN: f32 = 0.5;
const SWORD_OFFSET: f32 = 20.0;
const SWORD_RADIUS: f32 = 20.0;
const FLINCH_TICKS: u32 = 10;
/// `dCcG_SE_UNK1` (`d_cc_d.h:66`), the `SrcGObjAt Se` every sword cut's AT object carries: the
/// hit-sound ladder reads it as neither wood, nor metal, nor arrow, nor hookshot.
const AT_SE_SWORD: u8 = 1;
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
    if is_finisher(kind) { AT_SPL_UNK1 } else { AT_SPL_UNK0 }
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
    ships: Query<'w, 's, &'static Ship>,
}

#[derive(bevy_ecs::system::SystemParam)]
pub struct Outcome<'w> {
    combat: ResMut<'w, Combat>,
    zones: ResMut<'w, super::flags::ZoneActors>,
    events: ResMut<'w, SimEvents>,
    stop: ResMut<'w, HitStop>,
    audio: ResMut<'w, HitAudio>,
}

pub fn control(
    item: Res<ItemIntent>,
    intent: Res<MoveIntent>,
    camera: Res<CameraYaw>,
    gates: Sword<'_, '_>,
    mut actors: Query<
        (
            Entity,
            &Actor,
            Option<&mut HitPoints>,
            Option<&mut enemy::Hurt>,
        ),
        Without<Ship>,
    >,
    mut out: Outcome<'_>,
    mut players: Query<&mut Player>,
    registry: Res<ActorRegistry>,
    mut commands: Commands,
) {
    let pressed = item.0 && !out.combat.held;
    out.combat.held = item.0;
    let riding = gates.ships.iter().any(|ship| ship.riding);
    let blade = gates.held.item;
    let bare_hands =
        gates.equipped.item(super::inventory::HAMMER_SLOT) == super::inventory::ITEM_SKULL_HAMMER;
    let soup = out.combat.soup;
    for mut player in &mut players {
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
            {
                continue;
            }
            start(&mut out.combat, player, &intent, &camera, &gates.lock);
            out.events.list.push(SimEvent::Slash);
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
            start(&mut out.combat, player, &intent, &camera, &gates.lock);
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
            let center_x = player.pos.x + SWORD_OFFSET * player.yaw.sin();
            let center_z = player.pos.z + SWORD_OFFSET * player.yaw.cos();
            let kind = out.combat.kind;
            let swing = PlayerCut {
                cut_type: cut_type(kind),
                cut_count: cut_count(kind, out.combat.combo),
                pos: player.pos,
            };
            let swing_atk = Atk {
                at_type: AT_TYPE_SWORD,
                at_se: AT_SE_SWORD,
                at_spl: cut_spl(kind),
                at_atp: damage::sword_power(blade, soup) as u8,
                ac: Some(Attacker {
                    name: NAME_PLAYER.to_owned(),
                    pos: player.pos,
                    speed: Vec3::new(
                        player.speed * player.yaw.sin(),
                        player.speed_y,
                        player.speed * player.yaw.cos(),
                    ),
                    bomb_state: 0,
                    tsubo_type: 0,
                }),
            };
            let mut fx: Vec<DamageFx> = Vec::new();
            let mut dead: Vec<(Entity, i8, u16)> = Vec::new();
            for (entity, actor, hp, mut hurt) in actors.iter_mut() {
                let Some(mut hp) = hp else {
                    continue;
                };
                let dx = actor.pos.x - center_x;
                let dz = actor.pos.z - center_z;
                if dx * dx + dz * dz > SWORD_RADIUS * SWORD_RADIUS {
                    continue;
                }
                let mut target = Target {
                    name: actor.name.clone(),
                    pos: actor.pos,
                    room: actor.room,
                    status: actor_status(&registry, entity),
                    health: *hp,
                    ..Default::default()
                };
                let mut info = CcAtInfo {
                    mp_obj: Some(swing_atk.clone()),
                    p_particle_pos: Some(actor.pos),
                    ..Default::default()
                };
                cc_at_check(
                    &mut target,
                    &mut info,
                    &swing,
                    &mut out.events,
                    &mut out.stop,
                    &mut out.audio,
                    &mut fx,
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
                let _ = commands.entity(entity).despawn();
            }
        }
        out.combat.frame += 1;
    }
}
