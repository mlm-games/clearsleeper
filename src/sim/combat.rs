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
use super::walk::{add_calc, add_calc_angle_s, from_s16, target_yaw, to_s16};

/// The AT/TG pass every swing runs. It lives in a file of its own and is declared here, so the
/// four damage sites share one module without the parent's module list having to name it;
use crate::sim::contact_hit::{AT_SPRM_VS_ENEMY, AtObj, TgObj};

const STICK_GATE: f32 = 0.05;
const CHAIN_FRAME: u32 = 11;
/// The tick the forward step starts on.
///
/// The reference's own step fires on the first tick whose animation frame is within one rate of
/// 6.0 (`d_a_player_sword.inc:642`, `checkPass`), which is tick 1 for the four chain cuts and
/// tick 2 for the two finishers. The port's speed stands in for the swing's root motion, which
/// the reference reads from the sword move's own frames and the port does not carry, so the
/// step keeps the tick the port's swing behaviour is written around.
const LUNGE_FRAME: u32 = 2;
const LUNGE_FACTOR: f32 = 0.2;
const DECEL_SCALE: f32 = 0.7;
const DECEL_MIN: f32 = 0.5;
const SWORD_OFFSET: f32 = 20.0;
/// The sword's AT capsule radius: `20.0` for the Hero's Sword and `30.0` for the Master Sword.
/// `setNormalCutAtParam` and `setFinishCutAtParam` (`d_a_player_sword.inc:301-310`,
/// `:317-326`) take the radius from `mCut.field_0x3C` and `mCut.field_0x44`
/// (`d_a_player_HIO_data.inc:92-95`); the segment the capsule runs along comes from the sword
/// model's own joints, which the port does not carry, so the reach is this radius about the tip
/// the swing puts `SWORD_OFFSET` ahead of the player.
const SWORD_RADIUS: f32 = 20.0;
const MASTER_SWORD_RADIUS: f32 = 30.0;
/// `daPy_HIO_turn_c0::m` (`d_a_player_HIO_data.inc:22-24`), the rate a cut turns its body at:
/// `cLib_addCalcAngleS(&shape_angle.y, target, 30, 0x3CDF, 0x1F40)`
/// (`d_a_player_sword.inc:633-639`).
const CUT_TURN_SCALE: i16 = 30;
const CUT_TURN_MAX: i16 = 0x3CDF;
const CUT_TURN_MIN: i16 = 0x1F40;
const FLINCH_TICKS: u32 = 10;
/// The frames an actor's own TG bodies stay out of reach after a blow lands on it.
///
/// `d_a_bk.cpp:3569` writes `m030E = REG0_S(7) + 5`, and the register starts at zero for a
/// retail build (`d_s_play.cpp:78-86` zeroes every short register and only a debug HIO can move
/// it), so a Bokoblin drops its TG for five frames; `d_a_bb.cpp:2072` (`unk_340`),
/// `d_a_mo2.cpp:3099` (`m05B4`) and `d_a_st.cpp:1579` (`m02F6`) all carry the same five. It is
/// what makes a swing that walks its AT over an actor for several frames charge it once.
pub const HIT_GATE_TICKS: u32 = 5;
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

/// One cut's timing and speed numbers, the values the reference's own tables hold.
///
/// A cut runs on the animation frame its sword move plays: the cut's table gives the frame the
/// animation starts on and the rate it advances at (`d_a_player_HIO_data.inc:27-49`), and the
/// blade's AT object is live while the frame sits inside that table's own window
/// (`d_a_player_sword.inc:645`, `:748`, `:849`, `:948`, `:1026`, `:1107`). Tick `n` of a cut
/// sits on frame `start + rate * n`, which is where the tick numbers below come from.
struct Cut {
    /// The tick the cut ends on: the tick the animation runs out of frames.
    duration: u32,
    /// The first tick the blade's AT object is live on.
    at_first: u32,
    /// The last tick the blade's AT object is live on.
    at_last: u32,
    /// The speed the forward step adds.
    lunge_speed: f32,
    /// The largest step the deceleration may take.
    decel_max: f32,
}

/// One row per cut table: `daPy_HIO_cutA_c0::m` (`d_a_player_HIO_data.inc:27-29`) drives
/// `procCutA` (`d_a_player_sword.inc:589`) and the other five drive their own cut the same way.
fn cut(kind: CutKind) -> Cut {
    match kind {
        // The animation starts on frame 4.0 and runs at 1.2 a tick, so the AT window
        // `[5.0, 11.0)` is ticks 1 to 5 and the last frame 19.0 ends the cut on tick 13.
        CutKind::A => Cut {
            duration: 13,
            at_first: 1,
            at_last: 5,
            lunge_speed: 10.0,
            decel_max: 2.6,
        },
        // The window is `[5.0, 12.0)`, so the AT lives one tick longer than the first cut's.
        CutKind::F => Cut {
            duration: 13,
            at_first: 1,
            at_last: 6,
            lunge_speed: 8.0,
            decel_max: 0.95,
        },
        // The window is `[6.0, 12.0)`, so the AT comes up a tick late.
        CutKind::R => Cut {
            duration: 13,
            at_first: 2,
            at_last: 6,
            lunge_speed: 1.0,
            decel_max: 0.95,
        },
        // The window is `[5.0, 10.0)`, so the AT goes down a tick early, and the animation's
        // last frame 18.0 ends the cut on tick 12.
        CutKind::L => Cut {
            duration: 12,
            at_first: 1,
            at_last: 4,
            lunge_speed: 1.0,
            decel_max: 0.95,
        },
        // The finisher runs at 1.0 a tick, so the window `[5.0, 11.0)` is ticks 1 to 6, and its
        // deceleration step is `mCutEA.field_0x14` (`d_a_player_sword.inc:1036-1042`).
        CutKind::EA => Cut {
            duration: 15,
            at_first: 1,
            at_last: 6,
            lunge_speed: 15.0,
            decel_max: 4.0,
        },
        // The finisher runs at 0.9 a tick, so the window `[5.0, 11.0)` is ticks 2 to 7, and its
        // deceleration step is `mCutEB.field_0x14` (`d_a_player_sword.inc:1117-1123`).
        CutKind::EB => Cut {
            duration: 17,
            at_first: 2,
            at_last: 7,
            lunge_speed: 7.0,
            decel_max: 1.5,
        },
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
///
/// `setAtParam` (`d_a_player_main.cpp:9594-9596`) doubles the AT power again while the soup
/// power-up is up.
fn sword_at_power(selected: u8, finisher: bool, soup: bool) -> u8 {
    let normal = selected == damage::ITEM_SWORD;
    let atp = match (normal, finisher) {
        (true, false) => 1,
        (false, false) => 2,
        (true, true) => 2,
        (false, true) => 4,
    };
    if soup { atp * 2 } else { atp }
}

/// The AT capsule radius the sword the player holds cuts with, `setNormalCutAtParam`
/// (`d_a_player_sword.inc:301-310`): the Hero's Sword cuts with `mCut.field_0x3C` (20) and the
/// Master Sword with `mCut.field_0x44` (30).
fn sword_radius(selected: u8) -> f32 {
    if damage::is_master_sword(selected) {
        MASTER_SWORD_RADIUS
    } else {
        SWORD_RADIUS
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

/// One actor's hit gate: the port's `m030E` / `unk_340` / `m05B4` / `m02F6`, the timer a foe
/// starts the moment it processes a hit and runs down once a frame.
///
/// While it runs the actor's own TG bodies are dropped, which is what keeps one swing from
/// charging the same actor on every frame its AT overlaps. The reference does the drop by moving
/// the TG objects: `d_a_bk.cpp:4434-4437` parks both Bokoblin spheres at `y - 20000.0f` before
/// `SetC` / `Set`, and `d_a_mo2.cpp:3605-3607` and `d_a_tn.cpp:3626-3628` do the same. A TG the
/// pass never registers is the same thing as one the AT cannot reach, so [`reachable`] answers
/// for the actors whose reference body drops the volume and for the ones it only refuses in
/// `damage_check` (`d_a_bb.cpp:2071`, `d_a_st.cpp:1565`), which gates the same damage.
///
/// It is per actor: each one carries its own timer, started by the blow that reaches it, so two
/// foes struck by the same swing each take that swing once.
#[derive(Component, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HitGate(u32);

impl HitGate {
    /// Whether the actor's TG bodies register as reachable this frame: `m030E == 0`
    /// (`d_a_bk.cpp:3567`).
    pub fn reachable(&self) -> bool {
        self.0 == 0
    }

    /// Drop this actor's TG bodies for [`HIT_GATE_TICKS`] frames, the write the reference makes
    /// as it processes a hit (`d_a_bk.cpp:3569`).
    pub fn close(&mut self) {
        self.0 = HIT_GATE_TICKS;
    }

    /// One frame off the timer (`d_a_bk.cpp:4206-4207`, `d_a_bb.cpp:2265-2266`).
    pub fn tick(&mut self) {
        self.0 = self.0.saturating_sub(1);
    }
}

/// Whether a damage site may reach this candidate: an actor whose own gate is running has its TG
/// bodies dropped. An actor with no [`HitGate`] is one the port has not given the reference's
/// timer, and stays reachable.
pub fn reachable(gate: Option<&HitGate>) -> bool {
    gate.map_or(true, HitGate::reachable)
}

/// The gate runs down once a frame, before anything can be struck again.
pub fn tick_gates(mut gates: Query<&mut HitGate>) {
    for mut gate in &mut gates {
        gate.tick();
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
            Option<&mut HitGate>,
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
        let cut = cut(out.combat.kind);
        if out.combat.frame >= cut.duration {
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
        // `procCutA` (`d_a_player_sword.inc:629-640`): a cut with a lock-on target squares up
        // to it outright, and one without turns towards the heading the cut started with.
        match gates.lock.target.and_then(|target| {
            actors
                .iter()
                .find(|(entity, ..)| *entity == target)
                .map(|(_, actor, ..)| actor.pos)
        }) {
            Some(pos) => {
                player.face = from_s16(to_s16((pos.x - player.pos.x).atan2(pos.z - player.pos.z)));
                out.combat.heading = to_s16(player.face);
            }
            None => {
                player.face = from_s16(add_calc_angle_s(
                    to_s16(player.face),
                    out.combat.heading,
                    CUT_TURN_SCALE,
                    CUT_TURN_MAX,
                    CUT_TURN_MIN,
                ));
            }
        }
        player.yaw = player.face;
        if out.combat.frame == LUNGE_FRAME {
            player.speed = LUNGE_FACTOR * player.speed.abs() + cut.lunge_speed;
        } else {
            add_calc(
                &mut player.speed,
                0.0,
                DECEL_SCALE,
                cut.decel_max,
                DECEL_MIN,
            );
        }
        if out.combat.frame >= cut.at_first && out.combat.frame <= cut.at_last {
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
                    sword_radius(selected),
                ),
                AT_SPRM_VS_ENEMY,
                &AtMeta {
                    at_type: AT_TYPE_SWORD,
                    at_se: AT_SE_SWORD,
                    at_spl: cut_spl(kind),
                    at_atp: sword_at_power(selected, is_finisher(kind), out.combat.soup),
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
                    .filter_map(|(entity, actor, bodies, hp, _, gate)| {
                        if hp.is_none() || !reachable(gate) {
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
                let Ok((entity, actor, _, hp, mut hurt, gate)) = actors.get_mut(crossing.target)
                else {
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
                if let Some(mut gate) = gate {
                    gate.close();
                }
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
