use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::Vec3;

use super::actor::Actor;
use super::collision::Collision;
use super::combat::Combat;
use super::damage_fx::DamageFxQueue;
use super::damage_resolve::DamageFx;
use super::events::{SimEvent, SimEvents};
use super::health::Health;
use super::phantom_ganon::PhantomGanon;
use super::player::Player;
use super::stream::add_calc_angle_s2;
use super::walk;

pub const FGMAHOU_ACTORS: [&str; 1] = ["Fgmahou"];

pub const FGMAHOU_ACTOR: &str = "Fgmahou";

pub fn is_fgmahou(name: &str) -> bool {
    FGMAHOU_ACTORS.contains(&name)
}

const PRM_ORB_SHIFT: u32 = 0;
const PRM_ORB_MASK: u32 = 0xF;

const ORB_SLOTS: usize = 8;
const SPDD: [f32; ORB_SLOTS] = [0.65, 0.3, 0.1, 0.0, 0.0, 0.1, 0.3, 0.65];
const ANG_XD: [i16; ORB_SLOTS] = [
    0x0000,
    -0x0C00,
    -0x1800,
    -0x2000,
    -0x2000,
    -0x1800,
    -0x0C00,
    0x0000,
];

const SPD_SLOPE: f32 = 0.2;
const SPD_BIAS: f32 = 1.0;
const MOVE_SPEED_BASE: f32 = 50.0;
const ORB_YAW_STEP: i32 = 0x1400;
const YAW_BASE: i32 = 0x4000;
const RANGE_MIN: f32 = 500.0;
const ANG_SCALE: i16 = 2;
const ANG_STEP: i16 = (HIO_REG8_S4 + 0x500) as i16;
const RANGE_SCALE: f32 = 3.0;
const RANGE_CLAMP: f32 = 8000.0;
const AMP_STEP_A: i32 = 0xDAC;
const AMP_STEP_B: i32 = 0xCE4;
const AMP_B_SCALE: f32 = 0.75;
const STEP_SINGLE: i32 = 1;
const STEP_DOUBLE: i32 = 2;
const FLIP_YAW: i16 = 0x8000;
const TARGET_Y_BASE: f32 = 50.0;
const TARGET_Y_SPAN: f32 = 50.0;
const RND_FULL: f32 = 65536.0;
const WARP_LIFE: i8 = 0x32;
const WARP_YAW: i16 = 0x8000;
const WARP_POP_LIMIT: i16 = 8;
const TIMER_REARM: i16 = 5;
const TIMER_GRACE: i16 = 0x14;
const DESPAWN_DISTANCE: f32 = 5000.0;

const HIO_REG0_F3: f32 = 0.0;
const HIO_REG0_F4: f32 = 0.0;
const HIO_REG0_F8: f32 = 0.0;
const HIO_REG0_F13: f32 = 0.0;
const HIO_REG0_F17: f32 = 0.0;
const HIO_REG0_S2: i32 = 0;
const HIO_REG0_S6: i32 = 0;
const HIO_REG0_S7: i32 = 0;
const HIO_REG6_F5: f32 = 0.0;
const HIO_REG6_S2: i32 = 0;
const HIO_REG6_S3: i32 = 0;
const HIO_REG8_S2: i32 = 0;
const HIO_REG8_S3: i32 = 0;
const HIO_REG8_S4: i32 = 0;
const HIO_REG8_S9: i32 = 0;

const STTS_WEIGHT: u8 = 0xFA;
const STTS_SECOND: u8 = 0xFF;
const BRK_FRAME: f32 = 6.999;

const AT_RADIUS: f32 = 30.0;
const AT_RADIUS_WARP: f32 = 60.0;
const AT_POWER: u8 = 8;
const TG_RADIUS: f32 = 110.0;

const PLAYER_EYE_Y: f32 = 60.0;
const PLAYER_BODY_RADIUS: f32 = 20.0;
const SWORD_REACH: f32 = 20.0;
const SWORD_RADIUS: f32 = 20.0;
const SWORD_FIRST: u32 = 1;
const SWORD_LAST: u32 = 6;

const CULL_MODEL: f32 = 90.0;

const PARTICLE_DARKSHOT: u16 = 0x821E;
const PARTICLE_SMASH_DARKSHOT: u16 = 0x8244;
const PARTICLE_HIT_DARKSHOT0: u16 = 0x8245;
const PARTICLE_HIT_DARKSHOT1: u16 = 0x8246;

const SE_LK_PG_BOMB_STRIKE: u32 = 0x28A5;
const SE_OBJ_PG_EBALL_EXP_L: u32 = 0x6A36;

const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
const RND_SEED: i32 = 100;

const STATE_LAUNCH: i16 = 0;
const STATE_APPROACH: i16 = 1;
const STATE_HOLD: i16 = 2;
const STATE_WARP: i16 = 5;
const STATE_WARP_FLY: i16 = 6;
const STATE_WARP_END: i16 = 7;
const STATE_RELEASE: i16 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct FgMahouSph {
    pub centre: Vec3,
    pub radius: f32,
    pub at_set: bool,
    pub tg_set: bool,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct FgMahou {
    pub orb_number: u8,
    pub age: i16,
    pub state: i16,
    pub target_pos: Vec3,
    pub field_2d4: i16,
    pub field_2d6: i16,
    pub timers: [i16; 2],
    pub field_2de: i16,
    pub field_2e0: i16,
    pub field_2e2: i16,
    pub emitter: bool,
    pub health: i32,
    pub max_health: i32,
    pub home_angle: [i16; 3],
    pub shape_angle: [i16; 3],
    pub at: FgMahouSph,
    pub tg: FgMahouSph,
    pub life: i8,
    pub stts_weight: u8,
    pub stts_cold: u8,
    pub brk_frame: f32,
    pub seed: [i32; 3],
}

impl FgMahou {
    pub fn spawn(parameters: u32) -> Self {
        FgMahou {
            orb_number: ((parameters >> PRM_ORB_SHIFT) & PRM_ORB_MASK) as u8,
            age: 0,
            state: STATE_LAUNCH,
            target_pos: Vec3::ZERO,
            field_2d4: 0,
            field_2d6: 0,
            timers: [0; 2],
            field_2de: 0,
            field_2e0: 0,
            field_2e2: 0,
            emitter: false,
            health: 0,
            max_health: 0,
            home_angle: [0; 3],
            shape_angle: [0; 3],
            at: FgMahouSph {
                radius: AT_RADIUS,
                ..FgMahouSph::default()
            },
            tg: FgMahouSph {
                radius: TG_RADIUS,
                ..FgMahouSph::default()
            },
            life: 0,
            stts_weight: STTS_WEIGHT,
            stts_cold: STTS_SECOND,
            brk_frame: BRK_FRAME,
            seed: [RND_SEED; 3],
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        let r = self.tg.radius.max(self.at.radius) + CULL_MODEL;
        ([-r, -r, -r], [r, r, r])
    }

    pub fn drawn(&self) -> bool {
        true
    }

    pub fn is_delete(&self) -> bool {
        true
    }

    fn rnd(&mut self) -> f32 {
        for lane in 0..3 {
            self.seed[lane] = (self.seed[lane] * RND_MUL[lane]) % RND_MOD[lane];
        }
        let sum = (self.seed[0]) as f32 / RND_SCALE[0]
            + (self.seed[1]) as f32 / RND_SCALE[1]
            + (self.seed[2]) as f32 / RND_SCALE[2];
        (sum % 1.0).abs()
    }

    fn rnd_f(&mut self, range: f32) -> f32 {
        self.rnd() * range
    }

    fn rnd_fx(&mut self, range: f32) -> f32 {
        range * (self.rnd() - 0.5) * 2.0
    }

    pub fn execute(&mut self, actor: &mut Actor, env: &mut Env<'_>) {
        self.age = self.age.wrapping_add(1);
        for i in 0..2 {
            if self.timers[i] != 0 {
                self.timers[i] -= 1;
            }
        }
        if self.field_2de != 0 {
            self.field_2de -= 1;
        }
        let struck = self.at_hit(env);
        if struck {
            env.health.damage(AT_POWER);
        }
        if self.life == 0 {
            self.move_state(actor, env, struck);
        } else {
            self.life -= 1;
            if self.life == 0 {
                env.delete = true;
                return;
            }
        }
        if self.emitter && self.life == WARP_LIFE {
            self.emitter = false;
        }
        actor.eye = actor.pos;
    }

    fn move_state(&mut self, actor: &mut Actor, env: &mut Env<'_>, struck: bool) {
        let mut stage = Stage::of(self.state);
        let mut temp3 = 0.0f32;
        let mut steps = STEP_SINGLE;
        loop {
            match stage {
                Stage::Launch => {
                    let orb = i32::from(self.orb_number);
                    self.state = STATE_APPROACH;
                    self.field_2d4 = (orb * HIO_REG6_S2 - HIO_REG6_S3) as i16;
                    self.field_2d6 = self.rnd_f(RND_FULL) as i16;
                    actor.speed_f = HIO_REG0_F13 + MOVE_SPEED_BASE;
                    actor.speed_f *= (SPD_SLOPE + HIO_REG0_F4) * spdd(self.orb_number) + SPD_BIAS;
                    self.target_pos = env.player_eye.unwrap_or(actor.pos);
                    let diff2 = self.target_pos - actor.pos;
                    let yaw = orb * (HIO_REG8_S2 + ORB_YAW_STEP)
                        - (HIO_REG8_S3 + YAW_BASE)
                        + i32::from(atan2s(diff2.x, diff2.z));
                    self.home_angle[1] = yaw as i16;
                    self.home_angle[0] = ang_xd(usize::from(self.orb_number));
                    self.shape_angle = self.home_angle;
                    self.timers[0] = (HIO_REG8_S9 + TIMER_REARM as i32) as i16;
                    self.timers[1] = TIMER_GRACE;
                    self.emitter = true;
                    particle(env, PARTICLE_DARKSHOT, actor.pos, None);
                    self.tg.radius = HIO_REG6_F5 + TG_RADIUS;
                    steps = HIO_REG0_S2 + STEP_DOUBLE;
                    stage = Stage::Chase;
                }
                Stage::Chase => {
                    let diff3 = self.target_pos - actor.pos;
                    temp3 = diff3.length();
                    if self.timers[0] != 0 {
                        break;
                    }
                    if temp3 > HIO_REG0_F17 + RANGE_MIN && self.state < STATE_HOLD {
                        let diff2 = self.target_pos - actor.pos;
                        add_calc_angle_s2(
                            &mut self.home_angle[1],
                            atan2s(diff2.x, diff2.z),
                            ANG_SCALE,
                            ANG_STEP,
                        );
                        add_calc_angle_s2(
                            &mut self.home_angle[0],
                            atan2s(diff2.y, (diff2.x * diff2.x + diff2.z * diff2.z).sqrt())
                                .wrapping_neg(),
                            ANG_SCALE,
                            ANG_STEP,
                        );
                    } else {
                        self.state = STATE_HOLD;
                    }
                    if struck && env.boss_eye.is_some() {
                        env.laugh = true;
                        break;
                    }
                    if !self.tg_hit(env) {
                        break;
                    }
                    let angle = Vec3::new(0.0, f32::from(self.home_angle[1]), 0.0);
                    particle(env, PARTICLE_SMASH_DARKSHOT, actor.pos, Some(angle));
                    env.events.list.push(SimEvent::Se(SE_LK_PG_BOMB_STRIKE));
                    self.state = STATE_WARP;
                    stage = Stage::Warp;
                }
                Stage::Warp => {
                    let Some(eye) = env.boss_eye else {
                        self.life = WARP_LIFE;
                        break;
                    };
                    let drop = self.rnd_fx(TARGET_Y_SPAN) + TARGET_Y_BASE;
                    self.target_pos = eye;
                    self.target_pos.y -= drop;
                    self.home_angle[1] = self.home_angle[1].wrapping_sub(FLIP_YAW);
                    self.home_angle[0] = self.home_angle[0].wrapping_mul(-1);
                    self.timers[0] = (HIO_REG8_S9 + TIMER_REARM as i32) as i16;
                    self.field_2e0 = 1;
                    self.field_2e2 = WARP_YAW;
                    self.shape_angle = self.home_angle;
                    self.state = STATE_WARP_FLY;
                    self.at.radius = AT_RADIUS_WARP;
                    stage = Stage::WarpFly;
                }
                Stage::WarpFly => {
                    let diff3 = self.target_pos - actor.pos;
                    temp3 = diff3.length();
                    if self.timers[0] != 0 {
                        break;
                    }
                    if temp3 > HIO_REG0_F17 + RANGE_MIN && self.state < STATE_WARP_END {
                        let diff2 = self.target_pos - actor.pos;
                        add_calc_angle_s2(
                            &mut self.home_angle[1],
                            atan2s(diff2.x, diff2.z),
                            ANG_SCALE,
                            ANG_STEP,
                        );
                        add_calc_angle_s2(
                            &mut self.home_angle[0],
                            atan2s(diff2.y, (diff2.x * diff2.x + diff2.z * diff2.z).sqrt())
                                .wrapping_neg(),
                            ANG_SCALE,
                            ANG_STEP,
                        );
                    } else {
                        self.state = STATE_WARP_END;
                    }
                    break;
                }
                Stage::Release => {
                    if self.timers[0] == 0 {
                        env.delete = true;
                    }
                    return;
                }
                Stage::Hold => break,
            }
        }
        self.tail(actor, env, temp3, steps, struck);
    }

    fn tail(&mut self, actor: &mut Actor, env: &mut Env<'_>, temp3: f32, steps: i32, struck: bool) {
        let mut temp4 = temp3 * (HIO_REG0_F3 + RANGE_SCALE);
        if temp4 > HIO_REG0_F8 + RANGE_CLAMP {
            temp4 = HIO_REG0_F8 + RANGE_CLAMP;
        }
        self.field_2d4 = (i32::from(self.field_2d4) + HIO_REG0_S6 + AMP_STEP_A) as i16;
        self.field_2d6 = (i32::from(self.field_2d6) + HIO_REG0_S7 + AMP_STEP_B) as i16;
        let temp5 = (temp4 * ssin(self.field_2d4)) as i16;
        let temp7 = (temp4 * ssin(self.field_2d6) * AMP_B_SCALE) as i16;
        self.shape_angle[1] = self.home_angle[1].wrapping_add(temp5);
        self.shape_angle[0] = self.home_angle[0].wrapping_add(temp7);
        let base = Vec3::new(0.0, 0.0, actor.speed_f);
        actor.speed = turned(self.shape_angle[1], xrot(base, self.shape_angle[0]));
        for _ in 0..steps.max(1) {
            actor.pos += actor.speed;
        }
        let temp6 = actor.pos;
        if self.timers[1] == 0 {
            self.tg.centre = temp6;
            self.tg.tg_set = true;
            self.at.centre = temp6;
            self.at.at_set = true;
            let (next, wall, ground) = crr(env.solid, actor.old.pos, actor.pos);
            actor.pos = next;
            let far = match env.player_pos {
                Some(player) => (player - actor.pos).length() > DESPAWN_DISTANCE,
                None => true,
            };
            if far || struck || ground || wall {
                self.life = WARP_LIFE;
                self.health = 1;
                particle(env, PARTICLE_HIT_DARKSHOT0, actor.pos, None);
                particle(env, PARTICLE_HIT_DARKSHOT1, actor.pos, None);
                env.events.list.push(SimEvent::Se(SE_OBJ_PG_EBALL_EXP_L));
            }
        }
        if self.field_2e0 != 0 {
            self.field_2e0 += 1;
            if self.field_2e0 > WARP_POP_LIMIT {
                self.field_2e0 = 0;
            }
        }
    }

    fn at_hit(&self, env: &Env<'_>) -> bool {
        if !self.at.at_set {
            return false;
        }
        match env.player_pos {
            Some(player) => {
                walk::within(self.at.centre, player, self.at.radius + PLAYER_BODY_RADIUS)
            }
            None => false,
        }
    }

    fn tg_hit(&self, env: &Env<'_>) -> bool {
        if !self.tg.tg_set {
            return false;
        }
        match env.sword {
            Some(centre) => walk::within(self.tg.centre, centre, self.tg.radius + SWORD_RADIUS),
            None => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Stage {
    Launch,
    Chase,
    Warp,
    WarpFly,
    Release,
    Hold,
}

impl Stage {
    fn of(state: i16) -> Stage {
        match state {
            STATE_LAUNCH => Stage::Launch,
            STATE_APPROACH | STATE_HOLD => Stage::Chase,
            STATE_WARP => Stage::Warp,
            STATE_WARP_FLY | STATE_WARP_END => Stage::WarpFly,
            STATE_RELEASE => Stage::Release,
            _ => Stage::Hold,
        }
    }
}

pub struct Env<'a> {
    pub player_pos: Option<Vec3>,
    pub player_eye: Option<Vec3>,
    pub boss_eye: Option<Vec3>,
    pub sword: Option<Vec3>,
    pub solid: Option<&'a Collision>,
    pub events: &'a mut SimEvents,
    pub health: &'a mut Health,
    pub fx: &'a mut DamageFxQueue,
    pub laugh: bool,
    pub delete: bool,
}

fn particle(env: &mut Env<'_>, id: u16, pos: Vec3, angle: Option<Vec3>) {
    env.fx.push(DamageFx::Particle {
        id,
        pos,
        angle,
        scale: None,
    });
}

fn crr(solid: Option<&Collision>, from: Vec3, to: Vec3) -> (Vec3, bool, bool) {
    let Some(solid) = solid else {
        return (to, false, false);
    };
    let mut next = solid.slide(from, to);
    let wall = next.x != to.x || next.z != to.z;
    let mut ground = false;
    if let Some(y) = solid.ground_at(next.x, next.z) {
        if to.y < y {
            next.y = y;
            ground = true;
        }
    }
    (next, wall, ground)
}

fn atan2s(y: f32, x: f32) -> i16 {
    if y == 0.0 && x == 0.0 {
        return 0;
    }
    walk::to_s16(y.atan2(x))
}

fn ssin(angle: i16) -> f32 {
    walk::from_s16(angle).sin()
}

fn turned(yaw: i16, v: Vec3) -> Vec3 {
    let (s, c) = walk::from_s16(yaw).sin_cos();
    Vec3::new(v.x * c + v.z * s, v.y, v.z * c - v.x * s)
}

fn xrot(v: Vec3, pitch: i16) -> Vec3 {
    let (s, c) = walk::from_s16(pitch).sin_cos();
    Vec3::new(v.x, v.y * c - v.z * s, v.y * s + v.z * c)
}

fn spdd(orb: u8) -> f32 {
    let index = usize::from(orb);
    if index < ORB_SLOTS {
        return SPDD[index];
    }
    let word = index - ORB_SLOTS;
    if word + 1 >= ORB_SLOTS {
        return 0.0;
    }
    let hi = ANG_XD[word] as u16 as u32;
    let lo = ANG_XD[word + 1] as u16 as u32;
    f32::from_bits((hi << 16) | lo)
}

fn ang_xd(index: usize) -> i16 {
    ANG_XD.get(index).copied().unwrap_or(0)
}

#[derive(SystemParam)]
pub struct World<'w, 's> {
    collision: Option<Res<'w, Collision>>,
    combat: Res<'w, Combat>,
    players: Query<'w, 's, &'static Player>,
    boss: Query<
        'w,
        's,
        (&'static Actor, &'static mut PhantomGanon),
        (With<PhantomGanon>, Without<FgMahou>),
    >,
}

pub fn control(
    mut commands: Commands,
    mut world: World<'_, '_>,
    mut events: ResMut<SimEvents>,
    mut health: ResMut<Health>,
    mut fx: ResMut<DamageFxQueue>,
    mut orbs: Query<
        (Entity, &mut Actor, &mut FgMahou),
        (With<FgMahou>, Without<Player>, Without<PhantomGanon>),
    >,
) {
    let player_pos = world.players.iter().next().map(|player| player.pos);
    let player_eye = player_pos.map(|pos| pos + Vec3::new(0.0, PLAYER_EYE_Y, 0.0));
    let boss_eye = world
        .boss
        .iter()
        .map(|(actor, _)| actor.eye)
        .next();
    let sword = if world.combat.active
        && world.combat.frame >= SWORD_FIRST
        && world.combat.frame < SWORD_LAST
    {
        player_pos.map(|pos| pos + Vec3::new(0.0, 0.0, SWORD_REACH))
    } else {
        None
    };
    let mut laughed = false;
    let mut gone: Vec<Entity> = Vec::new();
    for (entity, mut actor, mut orb) in orbs.iter_mut() {
        let actor = &mut *actor;
        let orb = &mut *orb;
        let mut env = Env {
            player_pos,
            player_eye,
            boss_eye,
            sword,
            solid: world.collision.as_deref(),
            events: &mut events,
            health: &mut health,
            fx: &mut fx,
            laugh: false,
            delete: false,
        };
        orb.execute(actor, &mut env);
        laughed |= env.laugh;
        if env.delete {
            gone.push(entity);
        }
    }
    for entity in gone {
        let _ = commands.entity(entity).despawn();
    }
    if laughed {
        for (_, mut boss) in world.boss.iter_mut() {
            boss.hurt_gauge = 1;
        }
    }
}