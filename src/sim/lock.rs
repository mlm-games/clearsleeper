//! The lock-on and attention system: `dAttention_c`.
//!
//! Retail does not pick the nearest actor. It weighs every actor the attention manager
//! registers, keeps the best eight in a list, and drives a three-state machine
//! (`NONE`, `LOCK`, `RELEASE`) off the lock button's own state. The button is a hold:
//! the target is locked while the button is down and the lock is released when it comes
//! up (`d_attention.cpp:805-844`).

use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::combat::HitPoints;
use super::health::Health;
use super::intent::{MoveIntent, TargetIntent};
use super::jump::Jump;
use super::link_proc::{LOCK_ALLOWED, allows};
use super::player::Player;
use super::ship::Ship;
use super::swim::Swim;
use super::walk::{add_calc_angle_s, from_s16, to_s16};

const STICK_GATE: f32 = 0.05;
const FACE_SCALE: i16 = 2;
const FACE_MAX_STEP: i16 = 0x2000;
const FACE_MIN_STEP: i16 = 0x800;
const WEIGHT_RATIO: f32 = 0.5;

/// `dAttention_c::mLockOnList`, `d_attention.h:282`: the lock-on list holds eight entries.
const LOCKON_MAX: usize = 8;

/// `dAttention_c::field_0x01a`, the lock button's own state (`d_attention.cpp:716-741`).
const BUTTON_UP: u8 = 0;
const BUTTON_DOWN: u8 = 1;
const BUTTON_HELD: u8 = 2;

/// The frames the release runs for before the lock ends. Retail ends the release when the
/// lock-on mark's delete animation finishes (`d_attention.cpp:688-699`, `:795-798`), and
/// that animation is the `YJ_DELETE` one `d_attention.cpp:663` sets: eleven frames in
/// `res/Object/Always.arc`'s `bck/yj_delete.bck`, whose `ANK1` block carries a frame max
/// of ten.
const RELEASE_FRAMES: i32 = 11;

/// `fopAc_Attn_TYPE_BATTLE_e`, `f_op_actor.h:65`: the type the port's one registered class
/// answers to. The port has no per-actor attention type, so every registered actor is a
/// battle one.
const TYPE_BATTLE: u32 = 0x2;

/// `dAttention_c::DistTbl`, `d_attention.h:245-253`: the reach, the release reach, the
/// angle widen, the height window, the weight divisor and the front-angle bits of one row.
struct DistTbl {
    m_dist_xz_max: f32,
    m_dist_xz_max_release: f32,
    m_dist_xz_angle_adjust: f32,
    m_delta_y_max: f32,
    m_delta_y_min: f32,
    m_weight_divisor: f32,
    m_front_angle_check_bits: u32,
}

/// The one row the port uses. The port's actors carry no `attention_info`, so it has no
/// per-actor distance to read, and the row it already spelled out is the battle row at
/// `d_att_dist.cpp:15`: a reach of 150, a release reach of 200, a height window of a
/// hundred either way, a weight divisor of two and the front-angle bits that keep a
/// target inside forty-five degrees of the player's facing.
const ROW: DistTbl = DistTbl {
    m_dist_xz_max: 150.0,
    m_dist_xz_max_release: 200.0,
    m_dist_xz_angle_adjust: 0.0,
    m_delta_y_max: 100.0,
    m_delta_y_min: -100.0,
    m_weight_divisor: 2.0,
    m_front_angle_check_bits: 0x0011,
};

/// `dAttention_c::LockState`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LockState {
    #[default]
    None,
    Lock,
    Release,
}

/// `dAttList_c`, `d_attention.h:104-115`: the actor, the weight it was given, the distance
/// it was measured at and the attention type it was registered under.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct AttEntry {
    pub actor: Option<Entity>,
    pub weight: f32,
    pub distance: f32,
    pub kind: u32,
}

/// `dAttention_c`: the lock-on and action lists plus the state that `chkAttention` reads.
#[derive(Resource, Default, Debug, Clone)]
pub struct Lock {
    /// `dAttention_c::LockonTarget(0)`, the target the lock is on.
    pub target: Option<Entity>,
    pub state: LockState,
    /// `dAttention_c::mLockOnList`, the eight best candidates, the best first.
    pub lockon_list: Vec<AttEntry>,
    /// `dAttention_c::mLockOnOffset`, the entry the list is read from.
    pub lockon_offset: usize,
    /// `dAttention_c::mActionList`, `d_attention.h:285`: the action list holds four entries
    /// and the ones the port registers none of, so it stays empty.
    pub action_list: Vec<AttEntry>,
    /// `dAttention_c::mActionOffset`.
    pub action_offset: usize,
    /// `dAttention_c::field_0x01a`, the lock button's own state.
    button: u8,
    /// The frames the release still has to run for.
    release: i32,
}

/// The actors the attention manager registers. The retail reads each one's
/// `attention_info` - its flags, its attention point and its own distance row - and the port
/// has none of that, so it stands the registration in with the hit points a foe carries and
/// reads the actor's own position for the attention point.
type Registered<'w, 's, 'a> = Query<'w, 's, (Entity, &'a Actor), (Without<Ship>, With<HitPoints>)>;

impl Lock {
    /// `dAttention_c::LockonTarget`, `d_attention.cpp:995-1004`.
    pub fn lockon_target(&self, index: usize) -> Option<Entity> {
        if index >= self.lockon_list.len() {
            return None;
        }
        let mut slot = self.lockon_offset + index;
        if slot >= self.lockon_list.len() {
            slot -= self.lockon_list.len();
        }
        self.lockon_list[slot].actor
    }

    /// `dAttention_c::ActionTarget`, `d_attention.cpp:1037-1046`.
    pub fn action_target(&self, index: usize) -> Option<Entity> {
        if index >= self.action_list.len() {
            return None;
        }
        let mut slot = self.action_offset + index;
        if slot >= self.action_list.len() {
            slot -= self.action_list.len();
        }
        self.action_list[slot].actor
    }

    /// `dAttention_c::LockonTruth`, `d_attention.cpp:1049-1051`.
    pub fn lockon_truth(&self) -> bool {
        self.state == LockState::Lock
            || (self.state == LockState::Release && self.lockon_target(0).is_some())
    }

    /// `chkAttention`: the lock-on target when locked on, else the action target.
    pub fn chk_attention(&self, self_entity: Entity) -> bool {
        if self.lockon_truth() {
            return self.lockon_target(0) == Some(self_entity);
        }
        self.action_target(0) == Some(self_entity)
    }

    /// `dAttention_c::judgementButton`, `d_attention.cpp:716-741`: the button's own state.
    /// One frame of `BUTTON_DOWN` on the press, `BUTTON_HELD` while it stays down, and
    /// `BUTTON_UP` once it comes up.
    fn judgement_button(&mut self, held: bool) {
        match self.button {
            BUTTON_UP => {
                if held {
                    self.button = BUTTON_DOWN;
                }
            }
            BUTTON_DOWN => {
                self.button = BUTTON_HELD;
                if !held {
                    self.button = BUTTON_UP;
                }
            }
            _ => {
                if !held {
                    self.button = BUTTON_UP;
                }
            }
        }
    }

    /// `dAttention_c::judgementStatusHd`, `d_attention.cpp:805-844`: the state machine the
    /// d-pad option runs.
    fn judgement_status(&mut self, player: &Player, actors: &Registered<'_, '_, '_>) {
        match self.state {
            LockState::None => {
                self.stock_attention(player, actors);
                if self.button == BUTTON_DOWN {
                    self.judgement_trigger(player, actors);
                }
            }
            LockState::Lock => {
                if !self.judgement_lost_check(player, actors) && self.button == BUTTON_UP {
                    self.state = LockState::Release;
                    self.release = RELEASE_FRAMES;
                }
            }
            LockState::Release => {
                if self.button == BUTTON_DOWN {
                    if self.next_attention(player, actors).is_some() {
                        self.state = LockState::Lock;
                    } else {
                        self.state = LockState::None;
                        self.free_attention();
                    }
                } else {
                    if self.release > 0 {
                        self.release -= 1;
                    }
                    if self.lockon_target(0).is_none() || self.release == 0 {
                        self.state = LockState::None;
                        self.free_attention();
                    }
                }
            }
        }
    }

    /// `dAttention_c::judgementTriggerProc`, `d_attention.cpp:744-750`: the press takes the
    /// target the list holds and keeps it only if it passes the validity test.
    fn judgement_trigger(&mut self, player: &Player, actors: &Registered<'_, '_, '_>) {
        if self.chase_attention(player, actors) {
            self.state = LockState::Lock;
        }
    }

    /// `dAttention_c::judgementLostCheck`, `d_attention.cpp:753-763`.
    fn judgement_lost_check(&mut self, player: &Player, actors: &Registered<'_, '_, '_>) -> bool {
        if self.chase_attention(player, actors) {
            return false;
        }
        self.state = LockState::None;
        self.free_attention();
        true
    }

    /// `dAttention_c::chaseAttention`, `d_attention.cpp:564-609`: the target's own validity
    /// test. A target that no longer weighs anything is still held while it is inside the
    /// row's release reach, which is what lets the lock survive the target walking away.
    fn chase_attention(&mut self, player: &Player, actors: &Registered<'_, '_, '_>) -> bool {
        let offset = self.lockon_offset;
        let Some(entry) = self.lockon_list.get(offset) else {
            return false;
        };
        let Some(entity) = entry.actor else {
            return false;
        };
        let Some((_, actor)) = actors.iter().find(|(candidate, _)| *candidate == entity) else {
            return false;
        };
        let (weight, distance, angle1, angle2) = measure(player, actor);
        if weight <= 0.0 {
            if check_front_of_player(ROW.m_front_angle_check_bits, angle1, angle2) {
                return false;
            }
            if !check_distance(
                player.pos,
                angle1,
                actor.pos,
                ROW.m_dist_xz_max_release,
                ROW.m_dist_xz_angle_adjust,
                ROW.m_delta_y_max,
                ROW.m_delta_y_min,
            ) {
                return false;
            }
            self.lockon_list[offset].weight = distance_weight(distance, angle1, WEIGHT_RATIO);
            return true;
        }
        self.lockon_list[offset].weight = weight;
        self.lockon_list[offset].distance = distance;
        true
    }

    /// `dAttention_c::stockAttention`, `d_attention.cpp:520-538`: the candidate list, built
    /// and sorted every frame the player is not locked on.
    ///
    /// The retail also raises the mark's own flags here - the ones that start the lock-on
    /// mark's scale-in and delete animations (`d_attention.cpp:526-535`). The port draws no
    /// mark, so the flags have nothing to drive.
    fn stock_attention(&mut self, player: &Player, actors: &Registered<'_, '_, '_>) {
        self.init_list();
        if self.make_list(player, actors) > 0 {
            self.sort_list();
        }
    }

    /// `dAttention_c::nextAttention`, `d_attention.cpp:541-551`: the list is built again and
    /// the offset steps to the next candidate.
    fn next_attention(
        &mut self,
        player: &Player,
        actors: &Registered<'_, '_, '_>,
    ) -> Option<Entity> {
        let previous = self.lockon_target(0);
        self.init_list();
        if self.make_list(player, actors) > 0 {
            self.sort_list();
        }
        let best = self.lockon_list.first().and_then(|entry| entry.actor);
        if previous == best && self.lockon_list.len() > 1 {
            self.lockon_offset = 1;
        }
        self.lockon_target(0)
    }

    /// `dAttention_c::makeList`, `d_attention.cpp:461-464`, over `SelectAttention`
    /// (`:467-487`): every registered actor is weighed and the list takes the best eight.
    fn make_list(&mut self, player: &Player, actors: &Registered<'_, '_, '_>) -> usize {
        for (entity, actor) in actors.iter() {
            let (weight, distance, _, _) = measure(player, actor);
            self.set_lockon_list(entity, weight, distance);
        }
        self.lockon_list.len()
    }

    /// `dAttention_c::setLList`, `d_attention.cpp:382-405`: the list keeps the eight lowest
    /// weights, the lower the better.
    fn set_lockon_list(&mut self, entity: Entity, weight: f32, distance: f32) {
        if weight <= 0.0 {
            return;
        }
        let index = if self.lockon_list.len() < LOCKON_MAX {
            self.lockon_list.push(AttEntry {
                actor: Some(entity),
                weight,
                distance,
                kind: TYPE_BATTLE,
            });
            self.lockon_list.len() - 1
        } else {
            let mut worst = 0;
            for index in 1..self.lockon_list.len() {
                if self.lockon_list[index].weight > self.lockon_list[worst].weight {
                    worst = index;
                }
            }
            worst
        };
        if self.lockon_list[index].weight > weight {
            self.lockon_list[index] = AttEntry {
                actor: Some(entity),
                weight,
                distance,
                kind: TYPE_BATTLE,
            };
        }
    }

    /// `dAttention_c::sortList`, `d_attention.cpp:490-517`: the lowest weight first.
    fn sort_list(&mut self) {
        for index in 0..self.lockon_list.len().saturating_sub(1) {
            for other in index + 1..self.lockon_list.len() {
                if self.lockon_list[index].weight > self.lockon_list[other].weight {
                    self.lockon_list.swap(index, other);
                }
            }
        }
    }

    /// `dAttention_c::initList`, `d_attention.cpp:434-452`.
    fn init_list(&mut self) {
        self.lockon_list.clear();
        self.lockon_offset = 0;
        self.action_list.clear();
        self.action_offset = 0;
    }

    /// `dAttention_c::freeAttention`, `d_attention.cpp:554-561`.
    fn free_attention(&mut self) {
        self.init_list();
    }
}

/// The retail's own measurement of one actor (`d_attention.cpp:473-485`): the distance, the
/// two angles and the weight the row gives it.
fn measure(player: &Player, actor: &Actor) -> (f32, f32, i16, i16) {
    let delta = actor.pos - player.pos;
    let distance = delta.length();
    let angle1 = azimuth(delta).wrapping_sub(to_s16(player.face));
    let angle2 = azimuth(player.pos - actor.pos).wrapping_sub(actor.shape_angle[1]);
    let weight = if check_front_of_player(ROW.m_front_angle_check_bits, angle1, angle2)
        || !check_distance(
            player.pos,
            angle1,
            actor.pos,
            ROW.m_dist_xz_max,
            ROW.m_dist_xz_angle_adjust,
            ROW.m_delta_y_max,
            ROW.m_delta_y_min,
        ) {
        0.0
    } else {
        distance_weight(distance, angle1, WEIGHT_RATIO) / ROW.m_weight_divisor
    };
    (weight, distance, angle1, angle2)
}

/// `cSGlobe::U`, `c_angle.cpp:305-317`: the azimuth of a vector, measured from +Z in the
/// retail's signed sixteen-bit angle.
fn azimuth(delta: Vec3) -> i16 {
    to_s16(delta.x.atan2(delta.z))
}

/// `distace_weight`, `d_attention.cpp:294-298`.
fn distance_weight(distance: f32, angle: i16, ratio: f32) -> f32 {
    let turns = f32::from(angle) / 32768.0;
    distance * ((1.0 - ratio) + ratio * turns * turns)
}

/// `distace_angle_adjust`, `d_attention.cpp:301-308`.
fn distance_angle_adjust(distance: f32, angle: i16, ratio: f32) -> f32 {
    let turns = f32::from(angle) / 32768.0;
    let turns = if turns < 0.0 { -turns } else { turns };
    let sq = (1.0 - turns) * (1.0 - turns);
    distance * ((1.0 - ratio) + ratio * sq)
}

/// `check_distace`, `d_attention.cpp:311-324`: the height window and the reach, the reach
/// widened by the angle the target is off the player's facing.
fn check_distance(
    player: Vec3,
    angle: i16,
    actor: Vec3,
    max_xz: f32,
    angle_adjust: f32,
    max_dy: f32,
    min_dy: f32,
) -> bool {
    let delta = actor - player;
    if delta.y <= min_dy || delta.y >= max_dy {
        return false;
    }
    let adjust = max_xz + distance_angle_adjust(angle_adjust, angle, 1.0);
    adjust >= delta.x.hypot(delta.z)
}

/// `check_flontofplayer`, `d_attention.cpp:254-291`: the angles that put the target out of
/// the player's front. The first angle is the one off the player's own facing, the second
/// the one off the target's.
fn check_front_of_player(bits: u32, angle1: i16, angle2: i16) -> bool {
    const FTP: [u32; 9] = [0x04, 0x01, 0x02, 0x08, 0x10, 0x20, 0x40, 0x80, 0x100];
    const ANG: [i16; 3] = [0x4000, 0x2000, 0x0AAA];
    const ANG2: [i16; 6] = [0x0AAA, 0x2000, 0x2AAA, 0x4000, 0x4E38, 0x6000];
    let angle1 = angle1.wrapping_abs();
    let angle2 = angle2.wrapping_abs();
    for index in 0..3 {
        if bits & FTP[index] != 0 && angle1 > ANG[index] {
            return true;
        }
    }
    for index in (3..9).rev() {
        if bits & FTP[index] != 0 && angle2 > ANG2[index - 3] {
            return true;
        }
    }
    false
}

/// The attention manager: the lock-on and the attention lists, and the state `chkAttention`
/// reads.
///
/// The list is not a nearest-actor pick: `dAttention_c` weighs every actor the manager
/// registers and keeps the best eight (`d_attention.cpp:467` `SelectAttention`, `:327`
/// `calcWeight`, `:382` `setLList`, read back through `:131` `GetLockonList`). Everything
/// else registers nothing and takes no part - a pot's own flags are zero
/// (`d_a_ks.cpp:1624`) - and this port stands that registration in with
/// [`super::combat::HitPoints`], which the ships do not carry: they register the action half
/// instead.
///
/// The lock is a hold, not a tap. The button's own state (`d_attention.cpp:716-741`) is one
/// on the press and two while it stays down, and the state machine
/// (`d_attention.cpp:805-844`) keeps the target while the button is down and releases it
/// when it comes up: the release runs for the delete animation's eleven frames and then
/// the lock ends. A press during the release takes the next candidate
/// (`d_attention.cpp:541-551`).
pub fn control(
    target_intent: Res<TargetIntent>,
    intent: Res<MoveIntent>,
    health: Res<Health>,
    swim: Res<Swim>,
    jump: Res<Jump>,
    proc: Option<Res<super::link_proc::LinkProc>>,
    actors: Query<(Entity, &Actor), (Without<Ship>, With<HitPoints>)>,
    mut lock: ResMut<Lock>,
    mut players: Query<&mut Player>,
) {
    for mut player in &mut players {
        let player = &mut *player;
        if !allows(proc.as_deref(), LOCK_ALLOWED) {
            continue;
        }
        if health.units == 0 || swim.active {
            lock.state = LockState::None;
            lock.button = BUTTON_UP;
            lock.release = 0;
            lock.free_attention();
            lock.target = None;
            player.face = player.yaw;
            continue;
        }
        lock.judgement_button(target_intent.0);
        lock.judgement_status(player, &actors);
        let target = match lock.state {
            LockState::None => None,
            _ => lock.lockon_target(0),
        };
        lock.target = target;
        match lock.target {
            Some(target) => {
                if let Some((_, actor)) = actors.iter().find(|(candidate, _)| *candidate == target)
                {
                    let want = azimuth(actor.pos - player.pos);
                    let turned = add_calc_angle_s(
                        to_s16(player.face),
                        want,
                        FACE_SCALE,
                        FACE_MAX_STEP,
                        FACE_MIN_STEP,
                    );
                    player.face = from_s16(turned);
                }
            }
            None => {
                if !jump.active {
                    player.face = player.yaw;
                }
            }
        }
        if !jump.active && intent.axis.length().min(1.0) <= STICK_GATE {
            player.yaw = player.face;
        }
    }
}
