use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::combat::HitPoints;
use super::health::Health;
use super::intent::{MoveIntent, TargetIntent};
use super::jump::Jump;
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

#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct Lock {
    pub target: Option<Entity>,
    held: bool,
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
    actors: Query<(Entity, &Actor), (Without<Ship>, With<HitPoints>)>,
    mut lock: ResMut<Lock>,
    mut players: Query<&mut Player>,
) {
    let pressed = target_intent.0 && !lock.held;
    lock.held = target_intent.0;
    for mut player in &mut players {
        let player = &mut *player;
        if health.units == 0 || swim.active {
            lock.target = None;
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
        match lock.target {
            Some(target) => {
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
            None if !jump.active => player.face = player.yaw,
            None => {}
        }
        if !jump.active && intent.axis.length().min(1.0) <= STICK_GATE {
            player.yaw = player.face;
        }
    }
}
