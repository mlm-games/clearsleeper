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
const LOCK_REACH: f32 = 150.0;
const LOCK_RELEASE: f32 = 200.0;
const LOCK_DELTA_Y_MAX: f32 = 100.0;
const LOCK_DELTA_Y_MIN: f32 = -100.0;
const LOCK_CONE: u16 = 0x2000;
const FACE_SCALE: i16 = 2;
const FACE_MAX_STEP: i16 = 0x2000;
const FACE_MIN_STEP: i16 = 0x800;
const WEIGHT_RATIO: f32 = 0.5;
const WEIGHT_DIVISOR: f32 = 2.0;

/// `dAttention_c::LockState`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LockState {
    #[default]
    None,
    Lock,
    Release,
}

/// `dAttention_c`: the lock-on and action lists plus the state that `chkAttention` reads.
#[derive(Resource, Default, Debug, Clone)]
pub struct Lock {
    pub target: Option<Entity>,
    pub state: LockState,
    pub lockon_list: Vec<Entity>,
    pub lockon_offset: usize,
    pub action_list: Vec<Entity>,
    pub action_offset: usize,
    held: bool,
}

const LOCKON_MAX: usize = 8;
const ACTION_MAX: usize = 4;

impl Lock {
    /// `dAttention_c::LockonTarget`.
    pub fn lockon_target(&self, index: usize) -> Option<Entity> {
        self.lockon_at(index)
    }

    /// `dAttention_c::ActionTarget`.
    pub fn action_target(&self, index: usize) -> Option<Entity> {
        self.action_at(index)
    }

    /// `dAttention_c::LockonTruth`.
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

    fn lockon_at(&self, index: usize) -> Option<Entity> {
        if index >= self.lockon_list.len() {
            return None;
        }
        let slot = (self.lockon_offset + index) % self.lockon_list.len().max(1);
        self.lockon_list.get(slot).copied()
    }

    fn action_at(&self, index: usize) -> Option<Entity> {
        if index >= self.action_list.len() {
            return None;
        }
        let slot = (self.action_offset + index) % self.action_list.len().max(1);
        self.action_list.get(slot).copied()
    }

    fn rebuild(&mut self) {
        self.lockon_list.truncate(LOCKON_MAX);
        self.action_list.truncate(ACTION_MAX);
        self.lockon_offset = 0;
        self.action_offset = 0;
    }
}

fn azimuth(player: &Player, pos: Vec3) -> i16 {
    to_s16((pos.x - player.pos.x).atan2(pos.z - player.pos.z))
}

fn in_window(player: &Player, pos: Vec3, reach: f32) -> bool {
    let dx = pos.x - player.pos.x;
    let dz = pos.z - player.pos.z;
    let dy = pos.y - player.pos.y;
    dx * dx + dz * dz <= reach * reach
        && dy > LOCK_DELTA_Y_MIN
        && dy < LOCK_DELTA_Y_MAX
        && azimuth(player, pos)
            .wrapping_sub(to_s16(player.face))
            .unsigned_abs()
            <= LOCK_CONE
}

fn weight(player: &Player, pos: Vec3) -> f32 {
    let angle = azimuth(player, pos).wrapping_sub(to_s16(player.face));
    let turns = angle as f32 / 32768.0;
    (pos - player.pos).length() * (WEIGHT_RATIO + WEIGHT_RATIO * turns * turns) / WEIGHT_DIVISOR
}

fn pick(
    player: &Player,
    actors: &Query<(Entity, &Actor), (Without<Ship>, With<HitPoints>)>,
) -> Option<Entity> {
    let mut best: Option<(Entity, f32)> = None;
    for (entity, actor) in actors.iter() {
        if !in_window(player, actor.pos, LOCK_REACH) {
            continue;
        }
        let candidate = weight(player, actor.pos);
        if best.map_or(true, |(_, current)| candidate < current) {
            best = Some((entity, candidate));
        }
    }
    best.map(|(entity, _)| entity)
}

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
    let pressed = target_intent.0 && !lock.held;
    lock.held = target_intent.0;
    for mut player in &mut players {
        let player = &mut *player;
        if !allows(proc.as_deref(), LOCK_ALLOWED) {
            continue;
        }
        if health.units == 0 || swim.active {
            lock.target = None;
            lock.state = LockState::None;
            lock.lockon_list.clear();
            lock.action_list.clear();
            player.face = player.yaw;
            continue;
        }
        if let Some(target) = lock.target {
            let kept = actors
                .get(target)
                .map(|(_, actor)| in_window(player, actor.pos, LOCK_RELEASE))
                .unwrap_or(false);
            if !kept {
                lock.target = None;
            }
        }
        if lock.target.is_none() && pressed {
            lock.target = pick(player, &actors);
        }
        lock.rebuild();
        match lock.target {
            Some(target) => {
                lock.lockon_list = vec![target];
                lock.state = LockState::Lock;
                if let Ok((_, actor)) = actors.get(target) {
                    let want = azimuth(player, actor.pos);
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
                lock.state = LockState::None;
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
