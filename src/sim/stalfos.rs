use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::collision::Collision;
use super::combat::{Combat, CutKind, HitPoints};
use super::events::{SimEvent, SimEvents};
use super::flags::{Switches, ZoneActors};
use super::ground;
use super::health::Health;
use super::player::Player;
use super::stream::{add_calc_angle_s2, cos_s, sin_s};
use super::walk::{from_s16, to_s16};

pub const STALFOS_ACTOR: &str = "Stal";

pub const BEHAVIOR_NORMAL: u8 = 0;
pub const BEHAVIOR_UNDERGROUND: u8 = 1;
pub const BEHAVIOR_IN_HORIZONTAL_COFFIN: u8 = 2;
pub const BEHAVIOR_IN_VERTICAL_COFFIN: u8 = 3;
pub const BEHAVIOR_UPPER_BODY_ONLY: u8 = 14;

const BEHAVIOR_SHIFT: u32 = 0x00;
const BEHAVIOR_MASK: u32 = 0xF;
const SIGHT_SHIFT: u32 = 0x08;
const UNUSED_SHIFT: u32 = 0x10;
const AMBUSH_SHIFT: u32 = 0x18;
const BYTE_MASK: u32 = 0xFF;

const NO_SWITCH: u8 = 0xFF;
const PART_TOTAL: i8 = 0x19;
const PARTS_SEALED: i8 = 0x19;

const HEAD_HEALTH: i32 = 16;
const BODY_HEALTH: i16 = 0x10;
const UPPER_BODY_HEALTH: i16 = 5;
const STEAL_ITEMS: u8 = 3;
const BODY_GRACE: i16 = 0x10;

const HEAD_RADIUS: f32 = 45.0;
const BODY_RADIUS: f32 = 25.0;
const WEAPON_RADIUS: f32 = 30.0;
const WEAPON_RADIUS_LONG: f32 = 70.0;
const WEAPON_RADIUS_SHORT: f32 = 50.0;
const WEAPON_REACH_LONG: f32 = 80.0;
const WEAPON_REACH_SHORT: f32 = 30.0;
const WEAPON_SCALE: f32 = 1.5;
const WEAPON_POWER: u8 = 2;

const FALL_LIMIT: f32 = 4000.0;
const GRAVITY: f32 = 5.0;
const SPEED_STEP_MAX: f32 = 1.0;
const SPEED_RAMP: f32 = 5.0;
const ONE: f32 = 1.0;
const SHAPE_SCALE: i16 = 2;
const SHAPE_MAX: i16 = 0x1000;
const ROT_SCALE: i16 = 2;
const ROT_MAX: i16 = 0x40;
const TURN_DIV: i16 = 2;

const APPROACH_RANGE: f32 = 280.0;
const CLOSE_RANGE: f32 = 300.0;
const DASH_RANGE: f32 = 530.0;
const DASH_ENTER: f32 = 480.0;
const KICK_RANGE: f32 = 260.0;
const AMBUSH_RANGE: f32 = 500.0;
const AMBUSH_SCALE: f32 = 10.0;
const WALK_SPEED: f32 = 6.0;
const WALK_TURN: i16 = 0x800;
const CHARGE_SPEED: f32 = 10.0;
const CHARGE_TURN: i16 = 0x2000;
const CHARGE_TURN_FAST: i16 = 0x1000;
const KICK_TURN: i16 = 0x800;
const KICK_SPEED: f32 = 5.0;
const KICK_HOLD: f32 = 5.0;
const RETREAT_SPEED: f32 = -2.0;
const LEAN: f32 = 10000.0;
const LEAN_RATE: i32 = 500;
const SPIN_GATE: i16 = 0x400;
const SPIN_GATE_LOW: i16 = -0x400;
const SPIN_SETTLE: f32 = 0.01;
const SWING_STEP: f32 = 0.02;
const SWING_STEP_FAST: f32 = 0.1;
const SWING_BODY_RATE: f32 = 5000.0;
const SWING_THRESHOLD: f32 = 0.05;
const SWING_TRIGGER: i16 = 100;
const SWING_HALF: f32 = 0.5;

const NUN_FOLLOW: f32 = 1.3;
const NUN_SPIN: f32 = 0.3;
const NUN_FAST: f32 = 2.5;
const NUN_LINK: f32 = 30.0;
const NUN_BASE: f32 = 200.0;
const NUN_DROP: f32 = 20.0;
const NUN_ARM_REACH: f32 = 10.0;
const NUN_LEAN: i16 = -8000;
const NUN_SOUND_GATE: u32 = 7;

const PART_GRAVITY: f32 = 5.0;
const PART_TERM: f32 = -50.0;
const PART_PROBE: f32 = 100.0;
const PART_DROP: f32 = 10.0;
const PART_BOUNCE: f32 = -5.0;
const PART_FAR: f32 = 2000.0;
const PART_UP: f32 = 20.0;
const PART_SNAP_GATE: f32 = 60.0;
const PART_SNAP_SCALE: f32 = 1.0;
const PART_PULL_MIN: f32 = 50.0;
const PART_PULL_MAX: f32 = 300.0;
const PART_PULL_GAIN: f32 = 1.0;
const PART_PULL_RISE: f32 = 0.5;
const PART_PULL_BOB: f32 = 0.7;
const PART_PULL_RATE_X: i32 = 1000;
const PART_PULL_RATE_Y: i32 = 0x44C;
const PART_PULL_RATE_Z: i32 = 0x4B0;
const PART_PULL_TURN: i16 = 4;
const PART_PULL_TURN_MAX: i16 = 0x2000;
const PART_LINK: f32 = 20.0;
const PART_STRETCH_TOP: f32 = 1.0;
const PART_STRETCH_SCALE: f32 = 1.0;
const PART_STRETCH_STEP: f32 = 0.01;
const PART_CLOSED: f32 = 5.0;
const PART_BOUNCE_GATE: f32 = -40.0;
const PART_REST_GAIN: f32 = 0.3;
const PART_POP_GATE: f32 = -10.0;
const PART_HURT_GATE: f32 = -20.0;
const PART_HEAD_SEAT: f32 = 35.0;
const PART_HEAD_UP: f32 = 20.0;
const PART_HEAD_SEAT_Z: f32 = 20.0;
const PART_WAIT_SPREAD: f32 = 10.0;
const PART_WAIT_BASE: f32 = 2.0;
const PART_JUMP_SPREAD: f32 = 20000.0;
const PART_DIE_WAIT: i16 = 30;
const PART_HURT_WAIT: i16 = 0x7F;
const PART_SPAN: f32 = 65536.0;

const HIO_WALK_A: i32 = 9;
const HIO_WALK_A_END: i32 = 0x15;
const HIO_WALK_B: i32 = 0x2C;
const HIO_WALK_B_END: i32 = 0x35;
const HIO_RISE: i16 = 0x46;
const HIO_STAGGER: i16 = 200;

const WAIT_SPREAD: f32 = 60.0;
const WAIT_BASE: f32 = 100.0;
const REST_SPREAD: f32 = 50.0;
const REST_BASE: f32 = 30.0;
const IDLE_SPREAD: f32 = 40.0;
const QUICK: i16 = 10;
const FUSE: i16 = 500;
const SEAT_TIMER: i16 = 0x14;
const DEATH_FRAMES: i8 = 5;
const BROKEN_TIMER: i16 = 100;
const HURT_TICKS: i16 = 5;

const SE_WALK: u32 = 0x3832;
const SE_CV_HIT_BODY: u32 = 0x487B;
const SE_CV_BREAK: u32 = 0x487C;
const SE_CV_JUMP_HEAD: u32 = 0x487D;
const SE_CV_ATTACK_S: u32 = 0x487F;
const SE_CV_ATTACK_L: u32 = 0x4880;
const SE_CV_NO_CLUB: u32 = 0x4881;
const SE_CV_NUNCHAKU_S: u32 = 0x4882;
const SE_CV_NUNCHAKU_L: u32 = 0x4883;
const SE_CV_DIE: u32 = 0x4884;
const SE_TURN_HEAD: u32 = 0x588F;
const SE_STOP_HEAD: u32 = 0x5890;
const SE_BREAD_BODY: u32 = 0x5891;
const SE_RECOVER_BODY: u32 = 0x5892;
const SE_CLUB_UP: u32 = 0x5893;
const SE_CLUB_SWING: u32 = 0x5894;
const SE_CLUB_SWING_L: u32 = 0x5895;
const SE_JUMP_HEAD: u32 = 0x5896;
const SE_PULLOUT_ARM: u32 = 0x5897;
const SE_SWING_NUN_S: u32 = 0x5898;
const SE_SWING_NUN_L: u32 = 0x5899;
const SE_OUT_OF_GROUND: u32 = 0x589A;

const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
const RND_SEED: i32 = 100;

const FIGHT_END: u8 = 0x65;
const FIGHT_THIRD: u8 = 100;
const FIGHT_KAITEN_L: u8 = 0x19;
const FIGHT_KAITEN_R: u8 = 0x14;
const FIGHT_STALE: i16 = 8;
const FIGHT_TURN: i16 = 0x200;

const JYUN_WALK: i16 = 0;
const JYUN_WAIT: i16 = 1;
const JYUN_NUNCHAKU: i16 = 3;

const F2_PULL: i16 = 0;
const F2_DRAW: i16 = 1;
const F2_SHEATHE: i16 = 2;
const F2_NUNCHAKU_LOOP: i16 = 3;
const F2_NUNCHAKU_SWING: i16 = 4;
const F2_NUNCHAKU_WAIT: i16 = 0x32;
const F2_NUNCHAKU_SETTLE: i16 = 0x33;
const F2_NUNCHAKU_STUMBLE: i16 = 0x34;
const F2_NUNCHAKU_RISE: i16 = 0x35;
const F2_GIVE_UP: i16 = 0x64;
const F2_END: i16 = 0x65;

const GW_IDLE: i16 = 0;
const GW_RISE: i16 = 1;
const GW_DONE: i16 = 2;

const KW_IDLE: i16 = 0;
const KW_RISE: i16 = 1;
const KW_HOLD: i16 = 2;
const KW_DONE: i16 = 3;

const BW_LOOP: i16 = 0;
const BW_RISE: i16 = 1;
const BW_HOLD: i16 = 2;
const BW_HOLD_ALT: i16 = 3;
const BW_RETRY: i16 = 0xA;
const BW_RETRY_HOLD: i16 = 0xB;
const BW_LOST_HEAD: i16 = 0x14;
const BW_NO_HEAD: i16 = 0x23;

const HD_TURN: i16 = 0;
const HD_WAIT: i16 = 1;
const HD_TURN2: i16 = 10;
const HD_WAIT2: i16 = 11;

const UE_AIRBORNE: i16 = -1;
const UE_JUMP: i16 = 0;
const UE_RISE: i16 = 1;
const UE_LAND: i16 = 2;
const UE_WALK: i16 = 3;
const UE_DASH: i16 = 4;
const UE_WAIT: i16 = 5;

const SI_SETTLE: i16 = 0;
const SI_WALK: i16 = 1;
const SI_LOOP: i16 = 2;
const SI_APPROACH: i16 = 3;
const SI_WINDUP: i16 = 5;
const SI_KICK: i16 = 6;
const SI_KICK_END: i16 = 7;
const SI_DONE: i16 = 10;

const NUNCHAKU_NONE: i8 = 0;
const NUNCHAKU_DRAWN: i8 = 1;
const NUNCHAKU_SPIN: i8 = 2;
const NUNCHAKU_WHIR: i8 = 3;

const WEAPON_GONE: i8 = 0;
const WEAPON_LOOSE: i8 = 1;
const WEAPON_DROPPED: i8 = 2;
const WEAPON_HELD: i8 = 10;

const AT_OFF: i8 = 0;
const AT_CLUB: i8 = 1;
const AT_BIG: i8 = 2;
const AT_KICK: i8 = 3;

const TG_OPEN: i8 = 0;
const TG_BODY: i8 = 1;
const TG_SEALED: i8 = 2;

const BREAK_NONE: i8 = 0;
const BREAK_BODY: i8 = 1;
const BREAK_REBUILD: i8 = 2;
const BREAK_UPPER: i8 = 0xB;
const BREAK_UPPER_ONLY: i8 = 0x14;

const PART_ATTACHED: i8 = 0;
const PART_SEEK: i8 = 1;
const PART_SNAP: i8 = 2;
const PART_MACE_FALL: i8 = 5;
const PART_HEAD_POP: i8 = 6;
const PART_HEAD_HURT: i8 = 7;
const PART_HEAD_DEAD: i8 = 8;
const PART_IDLE_UPPER: i8 = 10;

const HEAD_ON: i8 = 0;
const HEAD_LOOSE: i8 = 1;
const HEAD_GONE: i8 = 2;

const SPHERE_COUNT: usize = 7;

const DASH_SPREAD: f32 = 10000.0;
const DASH_REACH: f32 = 500.0;
const GROUND_TIMER: i16 = 60;
const COFFIN_TAIL: i16 = 0x14;
const SEAT_SETTLE: i16 = 60;
const SEAT_SPREAD: f32 = 200.0;
const SEAT_DONE: f32 = 350.0;
const LOOP_SPEED: f32 = 1.2;
const SIT_TURN: i16 = 0x2000;
const RISE_LIFT: f32 = 40.0;
const RISE_FORWARD: f32 = 20.0;
const RISE_STALK_SPEED: f32 = 10.0;
const RISE_STALK_TURN: i16 = 0x1000;
const RISE_WAIT: i16 = 0x14;
const HONE_SPEED: f32 = 3.0;
const NUNCHAKU_SPREAD: f32 = 3.0;
const NUNCHAKU_PERIOD: i16 = 0x28;
const NUNCHAKU_HONE: [i32; 4] = [1, 11, 21, 31];

const FRAME_CLUB_UP_R: i32 = 10;
const FRAME_CLUB_SWING_R: i32 = 6;
const FRAME_CLUB_UP_L: i32 = 15;
const FRAME_CLUB_SWING_L: i32 = 8;
const FRAME_THIRD: i32 = 15;
const FRAME_HEAD_TURN: i32 = 32;
const FRAME_HEAD_TURN2: i32 = 30;
const FRAME_KICK: i32 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Wait,
    Fight,
    GroundWait,
    Coffin,
    Fight2,
    Break,
    Head,
    Upper,
    Sita,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Clip {
    Wait,
    LWait,
    Walk,
    LWalk,
    Hyunhyun,
    Start,
    YokoStart,
    TateStart,
    AttackR1,
    AttackR1End,
    AttackREnd,
    AttackR2,
    AttackL1,
    AttackL1End,
    AttackLEnd,
    AttackL2,
    KaitenR1,
    KaitenR1End,
    KaitenR2,
    KaitenR2End,
    KaitenL1,
    KaitenL1End,
    KaitenL2,
    KaitenL2End,
    OtotoR,
    OtotoL,
    Kyoro,
    Nuku1,
    Nuku2,
    HoneAttack,
    Tukeru1,
    Tukeru2,
    FukkiWait,
    MahiR,
    Mahi2,
    Kick1,
    Kick2,
    UeJump1,
    UeJump2,
    UeWalk,
    UeWait,
    SitaWalk,
}

const CLIP_FRAMES: [i16; 42] = [
    40, 40, 70, 70, 40, 89, 139, 59, 39, 14, 19, 71, 39, 14, 19, 71, 8, 10, 14, 19, 8, 10, 14, 19, 13,
    13, 50, 19, 29, 40, 24, 54, 30, 59, 49, 10, 28, 8, 7, 20, 20, 20,
];

impl Clip {
    fn frames(self) -> i16 {
        CLIP_FRAMES
            .get(self as usize)
            .copied()
            .unwrap_or(CLIP_FRAMES[0])
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UpperBody(pub bool);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Part {
    pub state: i8,
    pub pos: Vec3,
    pub vel: Vec3,
    pub wait: i16,
    pub age: i32,
    pub aim_y: i16,
    pub aim_x: i16,
    pub stretch: f32,
}

impl Part {
    fn attached() -> Self {
        Self {
            state: PART_ATTACHED,
            pos: Vec3::ZERO,
            vel: Vec3::ZERO,
            wait: 0,
            age: 0,
            aim_y: 0,
            aim_x: 0,
            stretch: 0.0,
        }
    }

    fn loose(state: i8, pos: Vec3, vel: Vec3) -> Self {
        Self {
            state,
            pos,
            vel,
            wait: 0,
            age: 0,
            aim_y: 0,
            aim_x: 0,
            stretch: 0.0,
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Stalfos {
    pub behavior: u8,
    pub sight: u8,
    pub unused: u8,
    pub ambush: u8,
    pub death_switch: u8,
    pub ambush_switch: u8,
    pub state: Step,
    pub sub: i16,
    pub fight: u8,
    pub handed: i8,
    pub swing_count: i16,
    pub target: Vec3,
    pub speed: Vec3,
    pub speed_f: f32,
    pub speed_target: f32,
    pub speed_step: f32,
    pub turn: i16,
    pub tick: u32,
    pub timers: [i16; 5],
    pub hurt_window: i16,
    pub fuse: i16,
    pub sway: i16,
    pub wobble: i16,
    pub swing: f32,
    pub aim: i16,
    pub clip: Clip,
    pub frame: f32,
    pub rate: f32,
    pub looping: bool,
    pub nunchaku: i8,
    pub nun_arm: Vec3,
    pub nun_tip: Vec3,
    pub nun_vel: Vec3,
    pub parts_left: i8,
    pub part_break: i8,
    pub at_body: i8,
    pub at_weapon: i8,
    pub flinch: i8,
    pub grab_lock: i16,
    pub body: i16,
    pub head_hp: i16,
    pub countdown: i8,
    pub weapon_state: i8,
    pub upper: Option<Entity>,
    pub head: Part,
    pub head_state: i8,
    pub mace: Part,
    pub head_live: bool,
    pub body_live: bool,
    pub at_live: bool,
    pub at_center: Vec3,
    pub at_radius: f32,
    pub placed: bool,
    pub home_y: f32,
    pub steal_left: u8,
    pub seed: [i32; 3],
}

fn add_calc2(value: &mut f32, target: f32, scale: f32, max_step: f32) {
    if *value != target {
        let step = (scale * (target - *value)).clamp(-max_step, max_step);
        *value += step;
    }
}

fn flat(from: Vec3, to: Vec3, radius: f32) -> bool {
    let dx = to.x - from.x;
    let dz = to.z - from.z;
    dx * dx + dz * dz <= radius * radius
}

fn rnd(seed: &mut [i32; 3]) -> f32 {
    for lane in 0..3 {
        seed[lane] = (seed[lane] * RND_MUL[lane]) % RND_MOD[lane];
    }
    let sum = seed[0] as f32 / RND_SCALE[0] + seed[1] as f32 / RND_SCALE[1] + seed[2] as f32 / RND_SCALE[2];
    (sum % 1.0).abs()
}

fn bearing(from: Vec3, to: Vec3) -> i16 {
    to_s16((to.x - from.x).atan2(to.z - from.z))
}

fn posmove(part: &mut Part, solid: Option<&Collision>) -> f32 {
    part.pos += part.vel;
    if part.vel.y >= PART_TERM {
        part.vel.y -= PART_GRAVITY;
    }
    let at = part.pos.y;
    let height = ground::surface(solid, part.pos)
        .filter(|floor| *floor >= at - PART_PROBE && *floor <= at + PART_PROBE);
    let mut landing = at - PART_PROBE;
    if let Some(floor) = height {
        if part.vel.y > 0.0 {
            if at >= floor {
                part.pos.y = floor - PART_DROP;
                part.vel.y = PART_BOUNCE;
            }
        } else {
            landing = floor;
        }
    }
    landing
}

fn part_land(
    part: &mut Part,
    seed: &mut [i32; 3],
    home_y: f32,
    actor: &Actor,
    solid: Option<&Collision>,
    gate: f32,
) -> Option<u32> {
    let mut landing = posmove(part, solid);
    if (home_y - part.pos.y).abs() > PART_FAR {
        part.pos = actor.pos;
        part.vel.y = 0.0;
        landing = actor.pos.y;
    }
    if part.pos.y > landing {
        return None;
    }
    part.pos.y = landing;
    let mut sound = None;
    if part.vel.y < gate {
        part.wait = (PART_WAIT_SPREAD * rnd(seed) + PART_WAIT_BASE) as i16;
        sound = Some(SE_JUMP_HEAD);
    }
    if part.wait == 0 {
        return sound;
    }
    part.vel = Vec3::ZERO;
    if part.wait != 1 {
        return sound;
    }
    let yaw = bearing(part.pos, actor.pos)
        .wrapping_add((PART_JUMP_SPREAD * (rnd(seed) - 0.5) * 2.0) as i16);
    let (sin, cos) = from_s16(yaw).sin_cos();
    part.vel = Vec3::new(
        PART_HEAD_SEAT_Z * sin,
        PART_HEAD_SEAT * rnd(seed) + PART_HEAD_UP,
        PART_HEAD_SEAT_Z * cos,
    );
    Some(SE_CV_JUMP_HEAD)
}

fn seek(part: &mut Part, actor: &Actor) {
    let gap = (actor.pos - part.pos).length();
    if gap < PART_SNAP_GATE {
        part.state = PART_SNAP;
        return;
    }
    let reach = (gap - PART_PULL_MIN).min(PART_PULL_MAX) * PART_PULL_GAIN;
    let wobble = Vec3::new(
        reach * sin_s(part.age.wrapping_mul(PART_PULL_RATE_X) as i16),
        reach * PART_PULL_RISE
            + reach * cos_s(part.age.wrapping_mul(PART_PULL_RATE_Y) as i16) * PART_PULL_BOB,
        reach * cos_s(part.age.wrapping_mul(PART_PULL_RATE_Z) as i16),
    );
    add_calc_angle_s2(
        &mut part.aim_y,
        to_s16(wobble.x.atan2(wobble.z)),
        PART_PULL_TURN,
        PART_PULL_TURN_MAX,
    );
    let span = (wobble.x * wobble.x + wobble.z * wobble.z).sqrt();
    add_calc_angle_s2(
        &mut part.aim_x,
        -to_s16(wobble.y.atan2(span)),
        PART_PULL_TURN,
        PART_PULL_TURN_MAX,
    );
    let (tilt_sin, tilt_cos) = from_s16(part.aim_x).sin_cos();
    let (yaw_sin, yaw_cos) = from_s16(part.aim_y).sin_cos();
    let link = part.stretch * PART_LINK;
    part.vel = Vec3::new(
        link * yaw_sin,
        -link * yaw_cos * tilt_sin,
        link * yaw_cos * tilt_cos,
    );
    part.pos += part.vel;
    add_calc2(&mut part.stretch, PART_STRETCH_TOP, PART_STRETCH_SCALE, PART_STRETCH_STEP);
}

fn snap(part: &mut Part, actor: &Actor) -> bool {
    add_calc2(&mut part.pos.x, actor.pos.x, PART_SNAP_SCALE, part.vel.x.abs() + PART_CLOSED);
    add_calc2(&mut part.pos.y, actor.pos.y, PART_SNAP_SCALE, part.vel.y.abs() + PART_CLOSED);
    add_calc2(&mut part.pos.z, actor.pos.z, PART_SNAP_SCALE, part.vel.z.abs() + PART_CLOSED);
    if (actor.pos - part.pos).length() > PART_CLOSED {
        return false;
    }
    part.state = PART_ATTACHED;
    part.vel = Vec3::ZERO;
    true
}

impl Stalfos {
    pub fn spawn(parameters: u32) -> Self {
        let behavior = ((parameters >> BEHAVIOR_SHIFT) & BEHAVIOR_MASK) as u8;
        let ambush = ((parameters >> AMBUSH_SHIFT) & BYTE_MASK) as u8;
        let armed = behavior == BEHAVIOR_UNDERGROUND
            || behavior == BEHAVIOR_IN_HORIZONTAL_COFFIN
            || behavior == BEHAVIOR_IN_VERTICAL_COFFIN;
        let upper = behavior == BEHAVIOR_UPPER_BODY_ONLY;
        Self {
            behavior,
            sight: ((parameters >> SIGHT_SHIFT) & BYTE_MASK) as u8,
            unused: ((parameters >> UNUSED_SHIFT) & BYTE_MASK) as u8,
            ambush,
            death_switch: 0,
            ambush_switch: if armed && ambush != NO_SWITCH {
                ambush.wrapping_add(1)
            } else if armed {
                NO_SWITCH
            } else {
                0
            },
            state: match behavior {
                BEHAVIOR_UNDERGROUND => Step::GroundWait,
                BEHAVIOR_IN_HORIZONTAL_COFFIN | BEHAVIOR_IN_VERTICAL_COFFIN => Step::Coffin,
                _ if upper => Step::Upper,
                _ => Step::Wait,
            },
            sub: JYUN_WALK,
            fight: 0,
            handed: 0,
            swing_count: 0,
            target: Vec3::ZERO,
            speed: Vec3::ZERO,
            speed_f: 0.0,
            speed_target: 0.0,
            speed_step: 0.0,
            turn: 0,
            tick: 0,
            timers: [0; 5],
            hurt_window: 0xF,
            fuse: 0,
            sway: 0,
            wobble: 0,
            swing: 0.0,
            aim: 0,
            clip: Clip::Wait,
            frame: 0.0,
            rate: ONE,
            looping: true,
            nunchaku: NUNCHAKU_NONE,
            nun_arm: Vec3::ZERO,
            nun_tip: Vec3::ZERO,
            nun_vel: Vec3::ZERO,
            parts_left: PART_TOTAL,
            part_break: if upper { BREAK_UPPER_ONLY } else { BREAK_NONE },
            at_body: TG_OPEN,
            at_weapon: AT_OFF,
            flinch: 0,
            grab_lock: 0,
            body: if upper { UPPER_BODY_HEALTH } else { BODY_HEALTH },
            head_hp: HEAD_HEALTH as i16,
            countdown: 0,
            weapon_state: if upper { WEAPON_GONE } else { WEAPON_LOOSE },
            upper: None,
            head: Part::attached(),
            head_state: HEAD_ON,
            mace: Part::attached(),
            head_live: false,
            body_live: false,
            at_live: false,
            at_center: Vec3::ZERO,
            at_radius: 0.0,
            placed: false,
            home_y: 0.0,
            steal_left: STEAL_ITEMS,
            seed: [RND_SEED; 3],
        }
    }

    pub fn hit_points(&self) -> i32 {
        HEAD_HEALTH
    }

    pub fn place(&mut self, shape_z: i16) {
        self.placed = true;
        let raw = shape_z as u8;
        self.death_switch = if raw == 0xFF { 0 } else { raw };
    }

    pub fn is_stalfos(name: &str) -> bool {
        name == STALFOS_ACTOR
    }

    fn rnd_f(&mut self, max: f32) -> f32 {
        max * rnd(&mut self.seed)
    }

    fn rnd_fx(&mut self, max: f32) -> f32 {
        max * (rnd(&mut self.seed) - 0.5) * 2.0
    }

    fn se(&self, events: &mut SimEvents, id: u32) {
        events.list.push(SimEvent::Se(id));
    }

    fn set_anm(&mut self, clip: Clip, looping: bool, speed: f32) {
        self.clip = clip;
        self.frame = 0.0;
        self.rate = speed;
        self.looping = looping;
    }

    fn step_anm(&mut self) {
        self.frame += self.rate;
        let end = (self.clip.frames()) as f32;
        if self.looping {
            if end > 0.0 && self.frame >= end {
                self.frame -= end;
            }
        } else if self.frame >= end {
            self.frame = end - 0.001;
            self.rate = 0.0;
        }
    }

    fn at_frame(&self) -> i32 {
        self.frame as i32
    }

    fn stopped(&self) -> bool {
        self.rate == 0.0
    }

    fn wait_set(&mut self, events: &mut SimEvents) {
        if self.nunchaku != NUNCHAKU_NONE {
            self.set_anm(Clip::Hyunhyun, true, ONE);
            self.nunchaku = NUNCHAKU_SPIN;
            self.se(events, SE_CV_NUNCHAKU_S);
        } else if self.handed == 0 {
            self.set_anm(Clip::Wait, true, ONE);
        } else {
            self.set_anm(Clip::LWait, true, ONE);
        }
    }

    fn walk_set(&mut self, speed: f32) {
        if self.nunchaku != NUNCHAKU_NONE {
            self.set_anm(Clip::Hyunhyun, true, ONE);
            self.nunchaku = NUNCHAKU_SPIN;
        } else if self.handed == 0 {
            self.set_anm(Clip::Walk, true, speed);
        } else {
            self.set_anm(Clip::LWalk, true, speed);
        }
    }

    fn idle(&mut self, events: &mut SimEvents) {
        self.state = Step::Wait;
        self.wait_set(events);
        self.sub = JYUN_WAIT;
        self.timers[1] = self.rnd_f(IDLE_SPREAD) as i16;
    }

    fn rest(&mut self) {
        self.timers[0] = (self.rnd_f(REST_SPREAD) + REST_BASE) as i16;
    }

    fn lean(&mut self) {
        self.sway = (LEAN * sin_s(self.tick.wrapping_mul(LEAN_RATE) as i16)) as i16;
    }

    fn centred(&self) -> bool {
        self.wobble > SPIN_GATE_LOW && self.wobble < SPIN_GATE
    }

    fn spin_settled(&self) -> bool {
        self.stopped() && self.swing.abs() < SPIN_SETTLE
    }

    fn ambushed(&self, actor: &Actor, player: &Player, switches: &Switches) -> bool {
        if self.ambush_switch != NO_SWITCH {
            return switches.test(self.ambush_switch.wrapping_sub(1));
        }
        let range = if self.sight == NO_SWITCH {
            AMBUSH_RANGE
        } else {
            (self.sight) as f32 * AMBUSH_SCALE
        };
        (player.pos - actor.pos).length() < range
    }

    fn speed_pos_calc(&mut self, actor: &mut Actor, solid: Option<&Collision>, player: Vec3) {
        if ground::grounded(solid, actor.pos) && self.turn != 0 {
            add_calc_angle_s2(&mut self.aim, bearing(actor.pos, player), TURN_DIV, self.turn);
            self.turn = 0;
        }
        let (sin, cos) = from_s16(self.aim.wrapping_add(self.sway)).sin_cos();
        let travel = self.speed_f;
        add_calc2(&mut self.speed_f, self.speed_target, ONE, self.speed_step);
        self.speed_step = SPEED_STEP_MAX;
        self.speed.x = travel * sin;
        self.speed.z = travel * cos;
        let before = actor.pos;
        actor.pos.x += self.speed.x;
        actor.pos.y += self.speed.y;
        actor.pos.z += self.speed.z;
        self.speed.y -= GRAVITY;
        if let Some(solid) = solid {
            actor.pos = solid.slide(before, actor.pos);
            if let Some(floor) = ground::surface(solid, actor.pos)
                && actor.pos.y < floor
            {
                actor.pos.y = floor;
                self.speed.y = 0.0;
            }
        }
    }

    fn jyunkai(&mut self, actor: &Actor, player: &Player, events: &mut SimEvents) {
        self.speed_f = 0.0;
        self.speed_step = SPEED_RAMP;
        match self.sub {
            JYUN_WAIT => {
                if self.timers[0] == 0 {
                    self.walk_set(ONE);
                    self.sub = JYUN_WALK;
                    self.timers[0] = (self.rnd_f(WAIT_SPREAD) + WAIT_BASE) as i16;
                }
            }
            JYUN_WALK => {
                let frame = self.at_frame();
                if (frame >= HIO_WALK_A && frame <= HIO_WALK_A_END)
                    || (frame >= HIO_WALK_B && frame <= HIO_WALK_B_END)
                {
                    self.speed_f = WALK_SPEED;
                    self.turn = WALK_TURN;
                    if frame == HIO_WALK_A || frame == HIO_WALK_B {
                        self.se(events, SE_WALK);
                    }
                }
                self.target = player.pos;
                if self.timers[0] == 0 {
                    self.wait_set(events);
                    self.sub = JYUN_WAIT;
                    self.timers[0] = (self.rnd_f(WAIT_SPREAD) + WAIT_BASE) as i16;
                }
            }
            _ => {}
        }
        if self.timers[1] == 0 && (player.pos - actor.pos).length() < APPROACH_RANGE {
            if self.weapon_state == WEAPON_HELD {
                self.state = Step::Fight;
                self.sub = JYUN_WALK;
                self.swing = 0.0;
                if self.rnd_f(ONE) < 0.5 {
                    self.fight = if self.handed == 0 { 0 } else { 5 };
                } else {
                    self.fight = if self.handed == 0 { 10 } else { 15 };
                }
            } else if self.weapon_state == WEAPON_GONE {
                self.state = Step::Fight2;
                self.sub = if self.nunchaku == NUNCHAKU_NONE {
                    JYUN_WALK
                } else {
                    JYUN_NUNCHAKU
                };
            }
        }
    }

    fn fight(&mut self, actor: &Actor, player: &Player, events: &mut SimEvents) {
        let gap = (player.pos - actor.pos).length();
        self.target = player.pos;
        self.turn = FIGHT_TURN;
        if self.sub == JYUN_WALK {
            self.speed_f = 0.0;
        }
        self.at_weapon = AT_OFF;
        let mut spin = 0.0f32;
        let mut spin_step = SWING_STEP;
        match self.fight {
            0 => {
                self.set_anm(Clip::AttackR1, false, ONE);
                self.fight = 1;
            }
            1 => {
                if self.at_frame() == FRAME_CLUB_UP_R {
                    self.se(events, SE_CLUB_UP);
                    self.se(events, SE_CV_ATTACK_S);
                }
                if self.stopped() {
                    self.set_anm(Clip::AttackR1End, false, ONE);
                    self.fight = 2;
                }
            }
            2 => {
                if self.at_frame() == FRAME_CLUB_SWING_R {
                    self.se(events, SE_CLUB_SWING);
                }
                self.at_weapon = AT_CLUB;
                if self.stopped() {
                    self.set_anm(Clip::OtotoR, true, ONE);
                    self.fight = 3;
                    self.rest();
                }
            }
            3 => {
                self.speed_f = RETREAT_SPEED;
                self.lean();
                if self.timers[0] == 0 {
                    if self.rnd_f(ONE) < 0.5 || gap < CLOSE_RANGE {
                        if self.rnd_f(ONE) < 0.5 {
                            self.set_anm(Clip::AttackREnd, false, ONE);
                            self.handed = 1;
                            self.fight = FIGHT_END;
                        } else {
                            self.set_anm(Clip::AttackR2, false, ONE);
                            self.handed = 0;
                            self.fight = FIGHT_THIRD;
                        }
                    } else {
                        self.fight = FIGHT_KAITEN_L;
                    }
                }
            }
            5 => {
                self.set_anm(Clip::AttackL1, false, ONE);
                self.fight = 6;
            }
            6 => {
                if self.at_frame() == FRAME_CLUB_UP_L {
                    self.se(events, SE_CLUB_UP);
                    self.se(events, SE_CV_ATTACK_S);
                }
                if self.stopped() {
                    self.set_anm(Clip::AttackL1End, false, ONE);
                    self.fight = 7;
                }
            }
            7 => {
                if self.at_frame() == FRAME_CLUB_SWING_L {
                    self.se(events, SE_CLUB_SWING);
                }
                self.at_weapon = AT_CLUB;
                if self.stopped() {
                    self.set_anm(Clip::OtotoL, true, ONE);
                    self.fight = 8;
                    self.rest();
                }
            }
            8 => {
                self.speed_f = RETREAT_SPEED;
                self.lean();
                if self.timers[0] == 0 {
                    if self.rnd_f(ONE) < 0.5 || gap < CLOSE_RANGE {
                        if self.rnd_f(ONE) < 0.5 {
                            self.set_anm(Clip::AttackLEnd, false, ONE);
                            self.handed = 0;
                            self.fight = FIGHT_END;
                        } else {
                            self.set_anm(Clip::AttackL2, false, ONE);
                            self.handed = 1;
                            self.fight = FIGHT_THIRD;
                        }
                    } else {
                        self.fight = FIGHT_KAITEN_R;
                    }
                }
            }
            10 => {
                self.set_anm(Clip::AttackR1, false, ONE);
                self.fight = 11;
                self.se(events, SE_CV_ATTACK_L);
            }
            11 => {
                if self.stopped() {
                    self.set_anm(Clip::KaitenR1, false, ONE);
                    self.fight = 12;
                }
            }
            12 => {
                spin = 1.0;
                spin_step = SWING_STEP_FAST;
                if self.stopped() {
                    self.fight = 13;
                    self.rest();
                }
            }
            13 => {
                self.speed_f = CHARGE_SPEED;
                spin = 1.0;
                spin_step = SWING_STEP_FAST;
                self.lean();
                if self.timers[0] == 0 && self.centred() {
                    self.set_anm(Clip::KaitenR1End, false, ONE);
                    self.fight = 14;
                }
            }
            14 => {
                if self.spin_settled() {
                    self.set_anm(Clip::OtotoR, true, ONE);
                    self.rest();
                    self.fight = 3;
                }
            }
            15 => {
                self.set_anm(Clip::AttackL1, false, ONE);
                self.fight = 16;
                self.se(events, SE_CV_ATTACK_L);
            }
            16 => {
                if self.stopped() {
                    self.set_anm(Clip::KaitenL1, false, ONE);
                    self.fight = 17;
                }
            }
            17 => {
                spin = -1.0;
                spin_step = SWING_STEP_FAST;
                if self.stopped() {
                    self.fight = 18;
                    self.rest();
                }
            }
            18 => {
                self.speed_f = CHARGE_SPEED;
                spin = -1.0;
                spin_step = SWING_STEP_FAST;
                self.lean();
                if self.timers[0] == 0 && self.centred() {
                    self.set_anm(Clip::KaitenL1End, false, ONE);
                    self.fight = 19;
                }
            }
            19 => {
                if self.spin_settled() {
                    self.set_anm(Clip::OtotoL, true, ONE);
                    self.rest();
                    self.fight = 8;
                }
            }
            20 => {
                self.set_anm(Clip::KaitenR2, false, ONE);
                self.fight = 21;
            }
            21 => {
                spin = 1.0;
                spin_step = SWING_STEP_FAST;
                if self.stopped() {
                    self.fight = 22;
                    self.rest();
                }
            }
            22 => {
                self.speed_f = CHARGE_SPEED;
                spin = 1.0;
                spin_step = SWING_STEP_FAST;
                self.lean();
                if self.timers[0] == 0 && self.centred() {
                    self.set_anm(Clip::KaitenR2End, false, ONE);
                    self.fight = 23;
                }
            }
            23 => {
                if self.spin_settled() {
                    self.set_anm(Clip::OtotoR, true, ONE);
                    self.rest();
                    self.fight = 3;
                }
            }
            25 => {
                self.set_anm(Clip::KaitenL2, false, ONE);
                self.fight = 26;
            }
            26 => {
                self.at_weapon = AT_BIG;
                spin = -1.0;
                spin_step = SWING_STEP_FAST;
                if self.stopped() {
                    self.fight = 27;
                    self.rest();
                }
            }
            27 => {
                self.speed_f = CHARGE_SPEED;
                spin = -1.0;
                spin_step = SWING_STEP_FAST;
                self.lean();
                if self.timers[0] == 0 && self.centred() {
                    self.set_anm(Clip::KaitenL2End, false, ONE);
                    self.fight = 28;
                }
            }
            28 => {
                if self.spin_settled() {
                    self.set_anm(Clip::OtotoL, true, ONE);
                    self.sub = FIGHT_STALE;
                    self.rest();
                }
            }
            FIGHT_THIRD => {
                if self.at_frame() == FRAME_THIRD {
                    self.se(events, SE_CLUB_SWING);
                }
                self.at_weapon = AT_CLUB;
                if self.stopped() {
                    self.idle(events);
                }
            }
            FIGHT_END => {
                if self.stopped() {
                    self.idle(events);
                }
            }
            _ => {}
        }
        add_calc2(&mut self.swing, spin, ONE, spin_step);
        self.wobble = self.wobble.wrapping_add((SWING_BODY_RATE * self.swing) as i16);
        if self.swing.abs() > SWING_THRESHOLD {
            let before = self.swing_count;
            self.swing_count = before.wrapping_add((SWING_BODY_RATE * self.swing.abs()) as i16);
            if self.swing_count >= SWING_TRIGGER && before < SWING_TRIGGER {
                self.se(events, SE_CLUB_SWING_L);
            }
        } else {
            self.swing_count = 0;
        }
        if self.swing.abs() > SWING_HALF {
            self.at_weapon = AT_BIG;
        }
    }

    fn fight2(&mut self, events: &mut SimEvents) {
        self.speed_f = 0.0;
        let mut done = false;
        match self.sub {
            F2_PULL => {
                self.hurt_window = 10;
                self.at_body = TG_BODY;
                self.set_anm(Clip::Kyoro, false, ONE);
                self.sub = F2_DRAW;
                self.se(events, SE_CV_NO_CLUB);
            }
            F2_DRAW => {
                self.hurt_window = 10;
                self.at_body = TG_BODY;
                if self.stopped() {
                    self.set_anm(Clip::Nuku1, false, ONE);
                    self.sub = F2_SHEATHE;
                }
            }
            F2_SHEATHE => {
                self.hurt_window = 10;
                self.at_body = TG_BODY;
                if self.stopped() {
                    self.set_anm(Clip::Nuku2, false, ONE);
                    self.sub = F2_END;
                    self.nunchaku = NUNCHAKU_DRAWN;
                    self.se(events, SE_PULLOUT_ARM);
                }
            }
            F2_NUNCHAKU_LOOP => {
                self.nunchaku = NUNCHAKU_WHIR;
                self.set_anm(Clip::HoneAttack, true, HONE_SPEED);
                self.sub = F2_NUNCHAKU_SWING;
                self.timers[0] = (self.rnd_f(NUNCHAKU_SPREAD) as i32 * i32::from(NUNCHAKU_PERIOD)) as i16;
                self.se(events, SE_CV_NUNCHAKU_L);
            }
            F2_NUNCHAKU_SWING => {
                self.at_weapon = AT_CLUB;
                if NUNCHAKU_HONE.contains(&self.at_frame()) {
                    self.se(events, SE_SWING_NUN_L);
                }
                if self.timers[0] == 0 {
                    done = true;
                }
            }
            F2_NUNCHAKU_WAIT => {
                self.set_anm(Clip::Wait, true, ONE);
                self.nunchaku = NUNCHAKU_DRAWN;
                self.sub = F2_NUNCHAKU_SETTLE;
                self.timers[0] = SEAT_TIMER;
            }
            F2_NUNCHAKU_SETTLE => {
                self.hurt_window = 10;
                self.at_body = TG_BODY;
                if self.timers[0] == 0 {
                    self.set_anm(Clip::Tukeru1, false, ONE);
                    self.sub = F2_NUNCHAKU_STUMBLE;
                }
            }
            F2_NUNCHAKU_STUMBLE => {
                self.hurt_window = 10;
                self.at_body = TG_BODY;
                if self.stopped() {
                    self.set_anm(Clip::Tukeru2, false, ONE);
                    self.sub = F2_NUNCHAKU_RISE;
                    self.nunchaku = NUNCHAKU_NONE;
                }
            }
            F2_NUNCHAKU_RISE => {
                self.hurt_window = 10;
                self.at_body = TG_BODY;
                if self.stopped() {
                    self.set_anm(Clip::FukkiWait, true, ONE);
                    self.sub = FIGHT_THIRD as i16;
                }
            }
            F2_GIVE_UP => {
                done = self.weapon_state == WEAPON_HELD;
            }
            F2_END => {
                done = self.stopped();
            }
            _ => {}
        }
        if done {
            self.idle(events);
        }
    }

    fn ground_wait(
        &mut self,
        actor: &Actor,
        player: &Player,
        events: &mut SimEvents,
        switches: &Switches,
    ) {
        self.hurt_window = 10;
        self.at_body = TG_BODY;
        match self.sub {
            GW_IDLE => {
                if self.ambushed(actor, player, switches) {
                    self.sub = GW_RISE;
                }
            }
            GW_RISE => {
                self.ambush_switch = 0;
                self.se(events, SE_OUT_OF_GROUND);
                self.set_anm(Clip::Start, false, ONE);
                self.sub = GW_DONE;
                self.timers[0] = GROUND_TIMER;
            }
            _ => {
                if self.stopped() {
                    self.state = Step::Wait;
                    self.sub = GW_IDLE;
                }
            }
        }
    }

    fn kan_wait(&mut self, actor: &Actor, player: &Player, switches: &Switches) {
        self.hurt_window = 10;
        self.at_body = TG_BODY;
        match self.sub {
            KW_IDLE => {
                if self.ambushed(actor, player, switches) {
                    self.sub = KW_RISE;
                }
            }
            KW_RISE => {
                if self.behavior == BEHAVIOR_IN_HORIZONTAL_COFFIN {
                    self.set_anm(Clip::YokoStart, false, 0.0);
                } else {
                    self.set_anm(Clip::TateStart, false, 0.0);
                }
                self.sub = KW_HOLD;
                self.ambush_switch = 0;
                self.timers[0] = HIO_RISE;
                self.timers[2] = HIO_RISE + COFFIN_TAIL;
            }
            KW_HOLD => {
                if self.timers[0] == 0 {
                    self.rate = ONE;
                    self.sub = KW_DONE;
                }
            }
            _ => {
                if self.stopped() {
                    self.state = Step::Wait;
                    self.sub = KW_IDLE;
                }
            }
        }
    }

    fn sealed(&mut self) {
        if self.parts_left >= PARTS_SEALED {
            self.sub = FIGHT_THIRD as i16;
            self.timers[0] = QUICK;
        }
    }

    fn st_break_wait(&mut self, events: &mut SimEvents) {
        self.speed_f = 0.0;
        if self.grab_lock == 0 {
            self.hurt_window = 10;
        }
        self.at_body = TG_BODY;
        match self.sub {
            BW_LOOP => {
                self.set_anm(Clip::FukkiWait, true, ONE);
                self.sub = BW_RISE;
                self.timers[0] = HIO_STAGGER;
                self.grab_lock = HIO_STAGGER;
            }
            BW_RISE => {
                if self.timers[0] == 0 {
                    self.sub = BW_HOLD;
                    self.part_break = BREAK_REBUILD;
                    self.se(events, SE_RECOVER_BODY);
                    self.fuse = FUSE;
                }
            }
            BW_HOLD => {
                self.at_body = TG_SEALED;
                self.sealed();
            }
            BW_HOLD_ALT => {
                self.at_body = TG_SEALED;
                if self.parts_left >= PARTS_SEALED && self.weapon_state == WEAPON_HELD {
                    self.sub = FIGHT_THIRD as i16;
                    self.timers[0] = QUICK;
                }
            }
            BW_RETRY => {
                self.set_anm(Clip::FukkiWait, true, ONE);
                self.sub = BW_RETRY_HOLD;
            }
            BW_RETRY_HOLD => {
                self.at_body = TG_SEALED;
                self.sealed();
            }
            BW_LOST_HEAD | BW_NO_HEAD => {
                self.grab_lock = BROKEN_TIMER;
                self.timers[0] = BROKEN_TIMER;
                if self.sub == BW_NO_HEAD {
                    self.state = Step::Wait;
                    self.sub = JYUN_WAIT;
                }
            }
            _ => {
                if self.timers[0] == 0 {
                    self.idle(events);
                    self.body = BODY_HEALTH;
                    self.hurt_window = BODY_GRACE;
                }
            }
        }
        if self.fuse != 0 {
            self.fuse -= 1;
            if self.fuse == 0 {
                self.countdown = DEATH_FRAMES;
            }
        }
    }

    fn head_damage(&mut self, events: &mut SimEvents) {
        self.speed_f = 0.0;
        self.at_body = TG_BODY;
        match self.sub {
            HD_TURN => {
                self.set_anm(Clip::MahiR, false, ONE);
                self.sub = HD_WAIT;
                self.se(events, SE_TURN_HEAD);
            }
            HD_WAIT => {
                if self.at_frame() == FRAME_HEAD_TURN {
                    self.se(events, SE_STOP_HEAD);
                }
                if self.stopped() {
                    self.idle(events);
                }
            }
            HD_TURN2 => {
                self.set_anm(Clip::Mahi2, false, ONE);
                self.sub = HD_WAIT2;
                self.se(events, SE_TURN_HEAD);
            }
            _ => {
                if self.at_frame() == FRAME_HEAD_TURN2 {
                    self.se(events, SE_STOP_HEAD);
                }
                if self.stopped() {
                    self.state = Step::Upper;
                    self.sub = UE_AIRBORNE;
                }
            }
        }
    }

    fn ue_move(&mut self, actor: &mut Actor, player: &Player, solid: Option<&Collision>) {
        self.target = player.pos;
        match self.sub {
            UE_AIRBORNE => {
                self.set_anm(Clip::UeJump1, false, ONE);
                self.sub = UE_RISE;
                self.speed.y = RISE_LIFT;
            }
            UE_JUMP => {
                self.set_anm(Clip::UeJump1, false, ONE);
                self.sub = UE_RISE;
                self.aim = bearing(actor.pos, player.pos).wrapping_add(KICK_TURN);
                self.speed.y = RISE_LIFT;
                let yaw = from_s16(self.aim);
                self.speed.x = RISE_FORWARD * yaw.sin();
                self.speed.z = RISE_FORWARD * yaw.cos();
                self.speed_f = RISE_FORWARD;
            }
            UE_RISE => {
                if ground::grounded(solid, actor.pos) {
                    self.set_anm(Clip::UeJump2, false, ONE);
                    self.sub = UE_LAND;
                }
            }
            UE_LAND => {
                if self.stopped() {
                    self.set_anm(Clip::UeWalk, true, LOOP_SPEED);
                    self.sub = UE_WALK;
                }
            }
            UE_WALK => {
                let bias = self.rnd_fx(DASH_SPREAD) as i16;
                let yaw = from_s16(self.aim.wrapping_add(bias));
                self.target =
                    player.pos + Vec3::new(DASH_REACH * yaw.sin(), 0.0, DASH_REACH * yaw.cos());
                self.sub = UE_DASH;
                self.timers[0] = RISE_WAIT;
            }
            UE_DASH => {
                self.speed_f = RISE_STALK_SPEED;
                self.turn = RISE_STALK_TURN;
                if self.timers[0] == 0 {
                    self.sub = UE_WALK;
                }
                if (player.pos - actor.pos).length() > DASH_RANGE {
                    self.set_anm(Clip::UeWait, true, ONE);
                    self.sub = UE_WAIT;
                    self.timers[0] = SEAT_TIMER;
                }
            }
            _ => {
                self.speed_f = 0.0;
                self.turn = CHARGE_TURN_FAST;
                self.target = player.pos;
                if self.timers[0] == 0 && (player.pos - actor.pos).length() < DASH_ENTER {
                    self.sub = UE_AIRBORNE;
                }
            }
        }
        if solid.is_some() && ground::surface(solid, actor.pos).is_none() {
            actor.pos.y = self.home_y;
        }
    }

    fn sita_move(&mut self, actor: &Actor, player: &Player, commands: &mut Commands) {
        match self.sub {
            SI_SETTLE => {
                self.sub = SI_WALK;
                self.timers[0] = SEAT_SETTLE;
                self.timers[1] = (self.rnd_f(SEAT_SPREAD) + SEAT_DONE) as i16;
            }
            SI_WALK => {
                self.set_anm(Clip::SitaWalk, true, LOOP_SPEED);
                self.sub = SI_LOOP;
            }
            SI_LOOP => {
                self.sub = SI_APPROACH;
            }
            SI_APPROACH => {
                self.speed_f = CHARGE_SPEED;
                self.turn = CHARGE_TURN;
                self.target = player.pos;
                if self.timers[0] == 0 && (player.pos - actor.pos).length() < KICK_RANGE {
                    self.sub = SI_WINDUP;
                }
            }
            SI_WINDUP => {
                self.target = player.pos;
                self.turn = SIT_TURN;
                self.set_anm(Clip::Kick1, false, ONE);
                self.sub = SI_KICK;
            }
            SI_KICK => {
                self.speed_f = KICK_SPEED;
                if self.stopped() {
                    self.set_anm(Clip::Kick2, false, ONE);
                    self.sub = SI_KICK_END;
                }
            }
            _ => {
                if self.at_frame() < FRAME_KICK {
                    self.at_weapon = AT_KICK;
                }
                self.speed_f = KICK_HOLD;
                if self.stopped() {
                    self.sub = SI_WALK;
                    self.timers[0] = RISE_WAIT;
                }
            }
        }
        if self.timers[1] < 5 {
            self.at_body = TG_SEALED;
        }
        if self.timers[1] == 0 {
            self.part_break = BREAK_UPPER;
            self.state = Step::Break;
            self.sub = SI_DONE;
            self.fuse = 0;
            if let Some(upper) = self.upper.take() {
                let _ = commands.entity(upper).despawn();
            }
        }
    }

    fn st_move(
        &mut self,
        actor: &mut Actor,
        player: &Player,
        events: &mut SimEvents,
        switches: &Switches,
        commands: &mut Commands,
        solid: Option<&Collision>,
    ) {
        match self.state {
            Step::Wait => self.jyunkai(actor, player, events),
            Step::Fight => self.fight(actor, player, events),
            Step::GroundWait => self.ground_wait(actor, player, events, switches),
            Step::Coffin => self.kan_wait(actor, player, switches),
            Step::Fight2 => self.fight2(events),
            Step::Break => self.st_break_wait(events),
            Step::Head => self.head_damage(events),
            Step::Upper => self.ue_move(actor, player, solid),
            Step::Sita => self.sita_move(actor, player, commands),
        }
        if self.state != Step::Coffin {
            self.speed_pos_calc(actor, solid, player.pos);
        }
    }

    fn damage_check(
        &mut self,
        actor: &Actor,
        player: &Player,
        taken: i32,
        finisher: bool,
        events: &mut SimEvents,
        commands: &mut Commands,
    ) {
        if taken <= 0 || self.hurt_window != 0 {
            return;
        }
        let head_hit = self.head_live && flat(self.head.pos, player.pos, HEAD_RADIUS);
        if head_hit && self.head_state != HEAD_ON {
            self.hurt_window = HURT_TICKS;
            self.head_hp -= taken as i16;
            self.flinch = 10;
            if self.head_hp <= 0 {
                self.head.state = PART_HEAD_DEAD;
                self.head.wait = PART_DIE_WAIT;
                self.head_state = HEAD_GONE;
                self.countdown = DEATH_FRAMES;
                self.se(events, SE_CV_DIE);
            } else {
                self.head.state = PART_HEAD_HURT;
                self.head.wait = PART_HURT_WAIT;
                self.se(events, SE_CV_HIT_BODY);
            }
            return;
        }
        let body_hit = self.body_live && flat(actor.pos, player.pos, BODY_RADIUS);
        if !body_hit && !head_hit {
            return;
        }
        self.hurt_window = HURT_TICKS;
        self.body -= taken as i16;
        self.flinch = 10;
        self.se(events, SE_CV_HIT_BODY);
        if self.body > 0 {
            return;
        }
        self.nunchaku = NUNCHAKU_NONE;
        if finisher && self.state != Step::Sita && self.state != Step::Upper {
            self.state = Step::Sita;
            self.sub = SI_SETTLE;
            self.part_break = BREAK_UPPER;
            self.body = UPPER_BODY_HEALTH;
            self.upper = Some(
                commands
                    .spawn((
                        Actor {
                            name: STALFOS_ACTOR.to_owned(),
                            parameters: u32::from(BEHAVIOR_UPPER_BODY_ONLY),
                            pos: actor.pos,
                            angle: actor.angle,
                            set_id: actor.set_id,
                            room: actor.room,
                        },
                        Stalfos::spawn(u32::from(BEHAVIOR_UPPER_BODY_ONLY)),
                        HitPoints(HEAD_HEALTH),
                        UpperBody::default(),
                    ))
                    .id(),
            );
            return;
        }
        self.sub = if self.state == Step::Sita {
            BW_NO_HEAD
        } else if self.state == Step::Upper || (self.state == Step::Head && self.sub >= HD_TURN2) {
            BW_LOST_HEAD
        } else {
            JYUN_WALK
        };
        self.state = Step::Break;
        self.part_break = BREAK_BODY;
        self.fuse = 0;
        self.se(events, SE_BREAD_BODY);
        self.se(events, SE_CV_BREAK);
    }

    fn head_step(&mut self, actor: &Actor, solid: Option<&Collision>, events: &mut SimEvents) {
        match self.head.state {
            PART_ATTACHED => {
                if self.part_break == BREAK_BODY {
                    self.head_state = HEAD_LOOSE;
                    let vel = self.kick(20.0);
                    self.head = Part::loose(PART_HEAD_POP, actor.pos, vel);
                } else if self.part_break == BREAK_UPPER {
                    self.head = Part::loose(PART_IDLE_UPPER, actor.pos, Vec3::ZERO);
                } else if self.part_break == BREAK_UPPER_ONLY {
                    self.head = Part::loose(PART_IDLE_UPPER, actor.pos, Vec3::ZERO);
                } else {
                    self.head.pos = actor.pos;
                }
                return;
            }
            PART_IDLE_UPPER | PART_HEAD_DEAD => return,
            _ => {}
        }
        self.head.age = self.head.age.wrapping_add(1);
        if self.head.wait != 0 {
            self.head.wait -= 1;
        }
        if matches!(self.head.state, PART_HEAD_POP | PART_HEAD_HURT)
            && self.grab_lock == 0
            && self.head.vel.y <= 0.0
        {
            self.head.state = PART_SEEK;
            self.head.vel = Vec3::ZERO;
            self.head.wait = 0;
            self.head.stretch = 0.0;
            self.head.age = (rnd(&mut self.seed) * PART_SPAN) as i32;
            return;
        }
        match self.head.state {
            PART_HEAD_POP => {
                let seed = &mut self.seed;
                if let Some(hit) =
                    part_land(&mut self.head, seed, self.home_y, actor, solid, PART_POP_GATE)
                {
                    events.list.push(SimEvent::Se(hit));
                }
            }
            PART_HEAD_HURT => {
                let seed = &mut self.seed;
                if let Some(hit) =
                    part_land(&mut self.head, seed, self.home_y, actor, solid, PART_HURT_GATE)
                {
                    events.list.push(SimEvent::Se(hit));
                }
                if self.head.wait == 0 {
                    self.head.state = PART_HEAD_POP;
                }
            }
            PART_SEEK => seek(&mut self.head, actor),
            _ => {
                if snap(&mut self.head, actor) {
                    self.head_state = HEAD_ON;
                    self.head.pos = actor.pos;
                }
            }
        }
    }

    fn kick(&mut self, spread: f32) -> Vec3 {
        Vec3::new(
            self.rnd_fx(spread),
            self.rnd_f(spread) + CHARGE_SPEED,
            self.rnd_fx(spread),
        )
    }

    fn mace_step(&mut self, actor: &Actor, solid: Option<&Collision>) {
        match self.mace.state {
            PART_ATTACHED => {
                self.mace.pos = actor.pos;
                if self.weapon_state == WEAPON_LOOSE {
                    self.weapon_state = WEAPON_HELD;
                }
                if self.part_break == BREAK_BODY || self.part_break == BREAK_UPPER {
                    if self.weapon_state == WEAPON_HELD {
                        let vel = self.kick(20.0);
                        self.mace = Part::loose(PART_MACE_FALL, actor.pos, vel);
                    }
                }
            }
            PART_IDLE_UPPER => {}
            PART_MACE_FALL => {
                let mut landing = posmove(&mut self.mace, solid);
                if (self.home_y - self.mace.pos.y).abs() > PART_FAR {
                    self.mace.pos = actor.pos;
                    self.mace.vel.y = 0.0;
                    landing = actor.pos.y;
                }
                if self.mace.pos.y <= landing {
                    self.mace.pos.y = landing;
                    if self.mace.vel.y < PART_BOUNCE_GATE {
                        self.mace.vel.y = self.rnd_f(PART_UP) + PART_UP;
                        self.mace.vel.x *= self.rnd_fx(PART_REST_GAIN);
                        self.mace.vel.z *= self.rnd_fx(PART_REST_GAIN);
                    } else {
                        self.mace.vel = Vec3::ZERO;
                    }
                    self.weapon_state = WEAPON_GONE;
                }
                if self.part_break != BREAK_REBUILD && self.part_break != BREAK_UPPER {
                    return;
                }
                self.mace.state = PART_SEEK;
                self.mace.pos = actor.pos;
                self.mace.vel = Vec3::ZERO;
                self.mace.wait = 0;
                self.mace.stretch = 0.0;
                self.mace.age = (rnd(&mut self.seed) * PART_SPAN) as i32;
                self.weapon_state = WEAPON_DROPPED;
                if self.nunchaku != NUNCHAKU_NONE {
                    self.state = Step::Fight2;
                    self.sub = F2_NUNCHAKU_WAIT;
                } else {
                    self.state = Step::Break;
                    self.sub = BW_HOLD_ALT;
                    self.fuse = FUSE;
                    self.set_anm(Clip::FukkiWait, true, ONE);
                }
            }
            PART_SEEK => seek(&mut self.mace, actor),
            _ => {
                if snap(&mut self.mace, actor) {
                    self.weapon_state = WEAPON_HELD;
                    self.state = Step::Break;
                    self.sub = BW_HOLD_ALT;
                    self.fuse = FUSE;
                    self.set_anm(Clip::FukkiWait, true, ONE);
                }
            }
        }
    }

    fn cc_set(&mut self, actor: &Actor) {
        let mut head = false;
        let mut body = false;
        for i in 0..SPHERE_COUNT {
            let live = if self.at_body == TG_SEALED {
                false
            } else if self.state == Step::Sita || self.sub == BW_NO_HEAD {
                self.at_body == TG_OPEN && i >= 5
            } else if self.state == Step::Upper {
                (self.at_body == TG_OPEN || i == 0) && i < 5
            } else {
                self.at_body == TG_OPEN || i == 0
            };
            if i == 0 {
                head = live;
            } else if live {
                body = true;
            }
        }
        self.head_live = head;
        self.body_live = body;
        self.at_live = self.at_weapon != AT_OFF && self.at_body == TG_OPEN;
        if self.at_live {
            let yaw = from_s16(actor.angle[1]);
            let long = self.tick & 1 != 0;
            self.at_center = if self.nunchaku != NUNCHAKU_NONE {
                self.nun_tip
            } else {
                let reach = if long {
                    WEAPON_REACH_LONG
                } else {
                    WEAPON_REACH_SHORT
                };
                actor.pos + Vec3::new(reach * yaw.sin(), 0.0, reach * yaw.cos())
            };
            self.at_radius = if self.nunchaku != NUNCHAKU_NONE || self.at_weapon == AT_KICK {
                WEAPON_RADIUS
            } else {
                let scale = if self.at_weapon == AT_BIG {
                    self.swing.abs() * WEAPON_SCALE
                } else {
                    ONE
                };
                if long {
                    WEAPON_RADIUS_LONG * scale
                } else {
                    WEAPON_RADIUS_SHORT * scale
                }
            };
        } else {
            self.at_center = Vec3::ZERO;
            self.at_radius = 0.0;
        }
        self.at_body = TG_OPEN;
        self.at_weapon = AT_OFF;
    }

    fn nun_control(&mut self, actor: &Actor, events: &mut SimEvents) {
        let yaw = from_s16(self.aim);
        self.nun_arm =
            actor.pos + Vec3::new(NUN_ARM_REACH * yaw.sin(), 0.0, NUN_ARM_REACH * yaw.cos());
        let mut anchor = Vec3::ZERO;
        if self.nunchaku == NUNCHAKU_SPIN {
            let lean = from_s16(self.aim.wrapping_add(NUN_LEAN));
            anchor = Vec3::new(NUN_BASE * lean.sin(), 0.0, NUN_BASE * lean.cos());
            if self.tick & NUN_SOUND_GATE == 0 {
                self.se(events, SE_SWING_NUN_S);
            }
        }
        let follow = match self.nunchaku {
            NUNCHAKU_DRAWN => NUN_FOLLOW,
            NUNCHAKU_SPIN => NUN_SPIN,
            _ => NUN_FAST,
        };
        let drift = self.nun_tip + anchor;
        let dx = drift.x + (self.nun_tip.x - self.nun_arm.x);
        let dy = drift.y + ((self.nun_tip.y - self.nun_arm.y) - NUN_DROP);
        let dz = drift.z + (self.nun_tip.z - self.nun_arm.z);
        let pitch = -to_s16(dy.atan2(dz));
        let span = (dy * dy + dz * dz).sqrt();
        let roll = to_s16(dx.atan2(span));
        let (pitch_sin, pitch_cos) = from_s16(pitch).sin_cos();
        let (roll_sin, roll_cos) = from_s16(roll).sin_cos();
        self.nun_vel = self.nun_tip;
        self.nun_tip = self.nun_arm
            + Vec3::new(
                NUN_LINK * roll_sin,
                -NUN_LINK * roll_cos * pitch_sin,
                NUN_LINK * roll_cos * pitch_cos,
            );
        self.nun_vel = follow * (self.nun_tip - self.nun_vel);
    }

    fn shape_settle(&mut self, actor: &mut Actor) {
        add_calc_angle_s2(
            &mut actor.angle[1],
            self.aim.wrapping_add(self.sway),
            SHAPE_SCALE,
            SHAPE_MAX,
        );
        self.sway = 0;
    }
}

pub fn is_stalfos(name: &str) -> bool {
    Stalfos::is_stalfos(name)
}

pub fn control(
    collision: Option<Res<Collision>>,
    combat: Res<Combat>,
    players: Query<&Player>,
    mut stalfoses: Query<(Entity, &mut Actor, &mut Stalfos, &mut HitPoints), Without<Player>>,
    mut uppers: Query<&mut UpperBody, With<Stalfos>>,
    mut health: ResMut<Health>,
    mut events: ResMut<SimEvents>,
    mut switches: ResMut<Switches>,
    mut zones: ResMut<ZoneActors>,
    mut commands: Commands,
) {
    let solid = collision.as_deref();
    let Some(player) = players.iter().next().copied() else {
        return;
    };
    let finisher = matches!(combat.kind, CutKind::EA | CutKind::EB);
    let mut gone: Vec<(Entity, i8, u16)> = Vec::new();
    for (entity, mut actor, mut st, mut hp) in &mut stalfoses {
        let actor = &mut *actor;
        let st = &mut *st;
        if !st.placed {
            st.place(actor.angle[2]);
            st.home_y = actor.pos.y;
            st.aim = actor.angle[1];
            st.head.pos = actor.pos;
            st.mace.pos = actor.pos;
        }
        if st.countdown > 0 {
            st.countdown -= 1;
            if let Ok(mut upper) = uppers.get_mut(entity) {
                upper.0 = true;
            }
            if st.countdown == 0 {
                if st.death_switch != 0 {
                    switches.set(st.death_switch);
                }
                gone.push((entity, actor.room, actor.set_id));
            }
            continue;
        }
        if st.home_y - actor.pos.y > FALL_LIMIT {
            if st.death_switch != 0 {
                switches.set(st.death_switch);
            }
            gone.push((entity, actor.room, actor.set_id));
            continue;
        }
        let orphan = match st.upper {
            Some(upper) => uppers.get(upper).map(|flag| flag.0).unwrap_or(false),
            None => false,
        };
        if orphan {
            if st.death_switch != 0 {
                switches.set(st.death_switch);
            }
            gone.push((entity, actor.room, actor.set_id));
            continue;
        }
        st.tick = st.tick.wrapping_add(1);
        for slot in 0..5 {
            if st.timers[slot] != 0 {
                st.timers[slot] -= 1;
            }
        }
        if st.hurt_window != 0 {
            st.hurt_window -= 1;
        }
        if st.grab_lock != 0 {
            st.grab_lock -= 1;
        }
        let taken = (HEAD_HEALTH - hp.0).clamp(0, HEAD_HEALTH);
        hp.0 = HEAD_HEALTH;
        st.st_move(actor, &player, &mut events, &switches, &mut commands, solid);
        st.damage_check(
            actor,
            &player,
            taken,
            finisher,
            &mut events,
            &mut commands,
        );
        st.shape_settle(actor);
        st.step_anm();
        st.cc_set(actor);
        st.head_step(actor, solid, &mut events);
        st.mace_step(actor, solid);
        if st.at_live && flat(st.at_center, player.pos, st.at_radius) {
            health.damage(WEAPON_POWER);
        }
        if st.part_break != BREAK_REBUILD {
            st.part_break = BREAK_NONE;
        }
        add_calc_angle_s2(&mut st.wobble, 0, ROT_SCALE, ROT_MAX);
        if st.nunchaku != NUNCHAKU_NONE {
            st.nun_control(actor, &mut events);
        }
        if st.flinch != 0 {
            st.flinch -= 1;
        }
    }
    for (entity, room, set_id) in gone {
        zones.mark_done(room, set_id);
        let _ = commands.entity(entity).despawn();
    }
}