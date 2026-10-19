use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::collision::Collision;
use super::combat::HitPoints;
use super::ground::grounded;
use super::health::Health;
use super::player::Player;
use super::walk::{add_calc, add_calc_angle_s, from_s16, to_s16, within};

pub const ENEMY_ACTOR: &str = "Bk";
pub const LANCE_ACTOR: &str = "mo2";

const ALERT_TICKS: u32 = 30;
const SIGHT_WANDER: f32 = 1000.0;
const SIGHT_STAND: f32 = 500.0;
const SIGHT_LANCE: f32 = 1250.0;
const HEIGHT_CENTER: f32 = 50.0;
const MOVE_SCALE: f32 = 0.25;
const HIT_OFFSET: f32 = 20.0;
const ATTACK_POWER: u8 = 1;
const STAND_KINDS: [u32; 3] = [4, 0xA, 0xB];
const PLANTED_KIND: u32 = 0xB;
const STRONG_BIT: u32 = 0x20;
const STRONG_HP: i32 = 7;
const BASE_HP: i32 = 5;
const LANCE_HP: i32 = 10;
const LANCE_FALLEN: u32 = 0xF;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Profile {
    pub steps: &'static [u32],
    pub speeds: &'static [f32],
    pub window: (f32, f32),
    pub track: f32,
    pub radius: f32,
    pub chase_speed: f32,
    pub ramp_step: f32,
    pub lunge: f32,
    pub decel: f32,
    pub attack_range: f32,
    pub reattack_range: f32,
    pub give_up_range: f32,
    pub cooldown: u32,
    pub cone: u16,
    pub height_limit: f32,
    pub box_side: f32,
    pub box_up: f32,
    pub box_near: f32,
    pub box_far: f32,
    pub body_height: f32,
    pub body_half: f32,
}

const SWORD_PROFILE: Profile = Profile {
    steps: &[16, 6, 22],
    speeds: &[1.2, 1.0, 1.0],
    window: (23.0, 26.0),
    track: 10.0,
    radius: 60.0,
    chase_speed: 90.0,
    ramp_step: 0.0,
    lunge: 0.0,
    decel: 20.0,
    attack_range: 240.0,
    reattack_range: 400.0,
    give_up_range: 475.0,
    cooldown: 35,
    cone: 0x6590,
    height_limit: 300.0,
    box_side: 500.0,
    box_up: 300.0,
    box_near: -125.0,
    box_far: 500.0,
    body_height: 24.0,
    body_half: 12.0,
};

const BARE_PROFILE: Profile = Profile {
    radius: 25.0,
    steps: &[23, 8],
    speeds: &[1.0, 1.0],
    window: (0.0, 20.0),
    track: 0.0,
    chase_speed: 45.0,
    ramp_step: 5.0,
    lunge: 70.0,
    ..SWORD_PROFILE
};

const LANCE: Profile = Profile {
    steps: &[18, 11, 17],
    speeds: &[1.0, 1.0, 1.0],
    window: (17.0, 27.0),
    track: 14.0,
    radius: 100.0,
    chase_speed: 70.0,
    ramp_step: 20.0,
    lunge: 0.0,
    decel: 20.0,
    attack_range: 312.5,
    reattack_range: 575.0,
    give_up_range: 650.0,
    cooldown: 15,
    cone: 0x31C4,
    height_limit: 400.0,
    box_side: 500.0,
    box_up: 130.0,
    box_near: -125.0,
    box_far: 500.0,
    body_height: 44.0,
    body_half: 20.0,
};

const LANCE_BARE: Profile = Profile {
    steps: &[12, 11, 11],
    speeds: &[1.0, 1.0, 1.0],
    window: (3.0, 8.0),
    track: 0.0,
    radius: 50.0,
    chase_speed: 70.0,
    ramp_step: 20.0,
    lunge: 0.0,
    decel: 20.0,
    attack_range: 312.5,
    reattack_range: 575.0,
    give_up_range: 650.0,
    cooldown: 15,
    cone: 0x31C4,
    height_limit: 400.0,
    box_side: 500.0,
    box_up: 130.0,
    box_near: -125.0,
    box_far: 500.0,
    body_height: 44.0,
    body_half: 20.0,
};

const TURN_SCALE: i16 = 4;
const THROW_FORWARD: f32 = 25.0;
const THROW_LIFT: f32 = 25.0;
/// The half turn the reference adds to the Moblin's shape angle when it launches the
/// spear (`d_a_mo2.cpp:176`): `0x8000`.
const THROW_HALF_TURN: i16 = -0x8000;
/// `fopAcM_SetGravity(this, -3.0f)` at the throw (`d_a_boko.cpp:355`) and the `gravity = -3.0f`
/// the throw arms keep (`:379`).
const THROW_GRAVITY: f32 = -3.0;
const THROW_LIFE: u32 = 120;
const THROW_REACH: f32 = 20.0;
const THROW_POWER: u8 = 1;
const TURN_MAX: i16 = 0x800;
const HOME_TURN_SCALE: i16 = 2;
const EYE_HEIGHT: f32 = 60.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyState {
    Wait,
    Alert,
    Chase,
    Attack,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Hurt(pub u32);

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Spear {
    pub pos: Vec3,
    pub vel: Vec3,
    pub life: u32,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Enemy {
    pub profile: Profile,
    pub always_sees: bool,
    pub armed: bool,
    pub lance: bool,
    pub sight: f32,
    pub home_yaw: i16,
    pub state: EnemyState,
    pub alert: u32,
    pub cooldown: u32,
    pub closing: bool,
    pub speed: f32,
    pub heading: i16,
    pub track: f32,
    pub clock: f32,
    pub step: usize,
    pub step_tick: u32,
    pub struck: bool,
}

pub fn spawn(parameters: u32, home_yaw: i16) -> (Enemy, HitPoints) {
    let kind = parameters & 0xF;
    let armed = kind != PLANTED_KIND;
    let strong = parameters & STRONG_BIT != 0 && armed;
    let sight = if STAND_KINDS.contains(&kind) {
        let band = (parameters >> 8) & 0xFF;
        if band == 0xFF {
            SIGHT_STAND
        } else {
            band as f32 * 10.0
        }
    } else {
        SIGHT_WANDER
    };
    (
        Enemy {
            profile: if armed { SWORD_PROFILE } else { BARE_PROFILE },
            always_sees: kind == 0xA,
            armed,
            lance: false,
            sight,
            home_yaw,
            state: EnemyState::Wait,
            alert: 0,
            cooldown: 0,
            closing: false,
            speed: 0.0,
            heading: home_yaw,
            track: 0.0,
            clock: 0.0,
            step: 0,
            step_tick: 0,
            struck: false,
        },
        HitPoints(if strong { STRONG_HP } else { BASE_HP }),
    )
}

pub fn spawn_lance(parameters: u32, home_yaw: i16) -> Option<(Enemy, HitPoints)> {
    if parameters & 0xF == LANCE_FALLEN {
        return None;
    }
    Some((
        Enemy {
            profile: LANCE,
            always_sees: false,
            armed: true,
            lance: true,
            sight: SIGHT_LANCE,
            home_yaw,
            state: EnemyState::Wait,
            alert: 0,
            cooldown: 0,
            closing: false,
            speed: 0.0,
            heading: home_yaw,
            track: 0.0,
            clock: 0.0,
            step: 0,
            step_tick: 0,
            struck: false,
        },
        HitPoints(LANCE_HP),
    ))
}

fn visible(profile: &Profile, enemy: &Enemy, actor: &Actor, yaw: i16, player_pos: Vec3) -> bool {
    if enemy.always_sees {
        return true;
    }
    let dx = player_pos.x - actor.pos.x;
    let dy = player_pos.y - actor.pos.y;
    let dz = player_pos.z - actor.pos.z;
    let eye = actor.pos.y + EYE_HEIGHT;
    if (player_pos.y + HEIGHT_CENTER - eye).abs() > profile.height_limit {
        return false;
    }
    let angle = to_s16(dx.atan2(dz));
    if yaw.wrapping_sub(angle).unsigned_abs() < profile.cone {
        return true;
    }
    let (sin, cos) = from_s16(yaw).sin_cos();
    let local_x = dx * cos - dz * sin;
    let local_z = dx * sin + dz * cos;
    local_x.abs() < profile.box_side
        && dy.abs() < profile.box_up
        && local_z > profile.box_near
        && local_z < profile.box_far
}

fn advance(actor: &mut Actor, enemy: &Enemy) {
    let yaw = from_s16(actor.angle[1]);
    actor.pos.x += enemy.speed * MOVE_SCALE * yaw.sin();
    actor.pos.z += enemy.speed * MOVE_SCALE * yaw.cos();
}

fn begin(enemy: &mut Enemy, angle: i16) {
    enemy.state = EnemyState::Attack;
    enemy.heading = angle;
    enemy.track = enemy.profile.track;
    enemy.clock = 0.0;
    enemy.step = 0;
    enemy.step_tick = 0;
    enemy.struck = false;
}

fn finish(enemy: &mut Enemy, actor: &Actor, player: &Player, dist: f32) {
    let profile = enemy.profile;
    enemy.cooldown = profile.cooldown;
    enemy.clock = 0.0;
    enemy.step = 0;
    enemy.step_tick = 0;
    let seen = visible(&profile, enemy, actor, actor.angle[1], player.pos);
    if dist < profile.reattack_range && seen {
        enemy.state = EnemyState::Chase;
    } else {
        enemy.state = EnemyState::Wait;
        enemy.speed = 0.0;
    }
}

fn tick(enemy: &mut Enemy, actor: &mut Actor, player: &Player, health: &mut Health) {
    let profile = enemy.profile;
    let dx = player.pos.x - actor.pos.x;
    let dz = player.pos.z - actor.pos.z;
    let dist = (dx * dx + dz * dz).sqrt();
    let bearing = to_s16(dx.atan2(dz));
    match enemy.state {
        EnemyState::Wait => {
            enemy.speed = 0.0;
            enemy.closing = false;
            actor.angle[1] =
                add_calc_angle_s(actor.angle[1], enemy.home_yaw, HOME_TURN_SCALE, TURN_MAX, 0);
            if dist < enemy.sight && visible(&profile, enemy, actor, actor.angle[1], player.pos) {
                enemy.state = EnemyState::Alert;
                enemy.alert = ALERT_TICKS;
            }
        }
        EnemyState::Alert => {
            enemy.speed = 0.0;
            enemy.alert = enemy.alert.saturating_sub(1);
            if enemy.alert == 0 {
                enemy.state = EnemyState::Chase;
            }
        }
        EnemyState::Chase => {
            actor.angle[1] = add_calc_angle_s(actor.angle[1], bearing, TURN_SCALE, TURN_MAX, 0);
            if profile.ramp_step > 0.0 {
                add_calc(
                    &mut enemy.speed,
                    profile.chase_speed,
                    1.0,
                    profile.ramp_step,
                    0.0,
                );
            } else {
                enemy.speed = profile.chase_speed;
            }
            if dist <= profile.give_up_range {
                enemy.closing = true;
            }
            if dist > enemy.sight || (enemy.closing && dist > profile.give_up_range) {
                enemy.state = EnemyState::Wait;
                enemy.speed = 0.0;
                enemy.closing = false;
            } else if dist < profile.attack_range && enemy.cooldown == 0 {
                begin(enemy, bearing);
            }
            advance(actor, enemy);
        }
        EnemyState::Attack => {
            if enemy.clock == 0.0 && profile.lunge > 0.0 {
                enemy.speed = profile.lunge;
            }
            add_calc(&mut enemy.speed, 0.0, 1.0, profile.decel, 0.0);
            enemy.clock += profile.speeds[enemy.step];
            if enemy.clock < enemy.track {
                enemy.heading = bearing;
            }
            actor.angle[1] =
                add_calc_angle_s(actor.angle[1], enemy.heading, TURN_SCALE, TURN_MAX, 0);
            let (low, high) = profile.window;
            if !enemy.struck && enemy.clock >= low && enemy.clock <= high {
                let yaw = from_s16(actor.angle[1]);
                let center_x = actor.pos.x + HIT_OFFSET * yaw.sin();
                let center_z = actor.pos.z + HIT_OFFSET * yaw.cos();
                let hit_x = player.pos.x - center_x;
                let hit_z = player.pos.z - center_z;
                if hit_x * hit_x + hit_z * hit_z <= profile.radius * profile.radius {
                    enemy.struck = true;
                    health.damage(ATTACK_POWER);
                }
            }
            enemy.step_tick += 1;
            if enemy.step_tick == profile.steps[enemy.step] {
                enemy.step += 1;
                enemy.step_tick = 0;
                if enemy.step == profile.steps.len() {
                    finish(enemy, actor, player, dist);
                }
            }
            advance(actor, enemy);
        }
    }
}

pub fn control(
    collision: Option<Res<Collision>>,
    mut health: ResMut<Health>,
    players: Query<&Player>,
    mut hurts: Query<&mut Hurt, With<Enemy>>,
    mut idle: Query<&mut Hurt, (Without<Enemy>, Without<super::redead::ReDead>)>,
    mut enemies: Query<(Entity, &mut Enemy, &mut Actor)>,
    mut spears: Query<(Entity, &mut Spear)>,
    mut commands: Commands,
) {
    let Some(player) = players.iter().next() else {
        return;
    };
    let floored = collision.is_some();
    let solid = collision.as_deref();
    for mut hurt in &mut idle {
        hurt.0 = hurt.0.saturating_sub(1);
    }
    for (entity, mut enemy, mut actor) in &mut enemies {
        let mut stunned = false;
        let mut recovering = false;
        if let Ok(mut hurt) = hurts.get_mut(entity) {
            if hurt.0 > 0 {
                hurt.0 -= 1;
                stunned = true;
                recovering = hurt.0 == 0;
            }
        }
        if stunned {
            enemy.speed = 0.0;
            enemy.clock = 0.0;
            enemy.step = 0;
            enemy.step_tick = 0;
            enemy.state = if recovering {
                EnemyState::Chase
            } else {
                EnemyState::Wait
            };
            enemy.closing = false;
        } else if floored {
            tick(&mut enemy, &mut actor, player, &mut health);
        } else {
            enemy.speed = 0.0;
        }
        if enemy.cooldown > 0 {
            enemy.cooldown -= 1;
        }
        if recovering && enemy.armed && enemy.lance {
            let dx = player.pos.x - actor.pos.x;
            let dz = player.pos.z - actor.pos.z;
            let dist = (dx * dx + dz * dz).sqrt();
            if dist > enemy.profile.attack_range {
                enemy.armed = false;
                enemy.profile = LANCE_BARE;
                // `yari_off_check` launches the spear at the Moblin's own shape angle plus a
                // half turn (`d_a_mo2.cpp:176`), and the Moblin's shape angle is the way it
                // faces: `execute` copies `current.angle` into it every frame
                // (`d_a_mo2.cpp:3527-3529`) and the Moblin turns that way towards the player
                // (`:2931-2934`, `:2261-2262`). The spear therefore leaves over the Moblin's
                // shoulder, not at the player.
                //
                // OMITTED: the spread of up to 8 000 either side of that
                // (`(s16)cM_rndFX(8000.0f)`, `d_a_mo2.cpp:176`, the draw `c_math.h:28`) is the
                // global `cM_rndFX` channel, which this sim does not carry.
                let yaw = from_s16(actor.angle[1].wrapping_add(THROW_HALF_TURN));
                commands.spawn(Spear {
                    pos: actor.pos,
                    vel: Vec3::new(
                        THROW_FORWARD * yaw.sin(),
                        THROW_LIFT,
                        THROW_FORWARD * yaw.cos(),
                    ),
                    life: THROW_LIFE,
                });
            }
        }
    }
    for (entity, mut spear) in &mut spears {
        spear.vel.y += THROW_GRAVITY;
        let step = spear.vel;
        spear.pos += step;
        spear.life = spear.life.saturating_sub(1);
        if within(spear.pos, player.pos, THROW_REACH) {
            health.damage(THROW_POWER);
            let _ = commands.entity(entity).despawn();
            continue;
        }
        if spear.life == 0 || grounded(solid, spear.pos) {
            let _ = commands.entity(entity).despawn();
        }
    }
}
