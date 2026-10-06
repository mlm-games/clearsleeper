use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::Vec3;

use super::actor::Actor;
use super::collision::Collision;
use super::combat::Combat;
use super::damage;
use super::events::{SimEvent, SimEvents};
use super::health::Health;
use super::item::Held;
use super::player::Player;

pub const BWDS_ACTOR: &str = "Bwds";

pub fn is_bwds(name: &str) -> bool {
    name == BWDS_ACTOR
}

pub type Xyz = [f32; 3];

const ZERO_XYZ: Xyz = [0.0, 0.0, 0.0];

const HIO_M005: u8 = 0;
const HIO_M008: f32 = 0.8;
const HIO_M010: f32 = 7.0;
const HIO_M014: i16 = 220;
const HIO_M018: f32 = 2000.0;
const HIO_M01C: f32 = 300.0;

const ACTION_UG_MOVE: i16 = 0;
const ACTION_HOOK_ON: i16 = 1;
const ACTION_HOOK_CHANCE: i16 = 2;
const ACTION_FAIL: i16 = 5;

const BCK_KOBOSS_CLOSE: i32 = 0x4;
const BCK_KOBOSS_PAKUPAKU: i32 = 0x5;

const FRAME_KOBOSS_CLOSE: i16 = 10;
const FRAME_KOBOSS_PAKUPAKU: i16 = 16;

const EMODE_NONE: u8 = 0;
const EMODE_LOOP: u8 = 1;
const EMODE_LOOP_ONCE: u8 = 2;
const STATE_STOP: u8 = 2;

const SEG_COUNT: usize = 15;
const SPH_RADIUS: f32 = 15.0;
const HEAD_BIND: Xyz = [0.0, 0.0, 10.0];
const HEAD_SEG_OFFSET: f32 = 48.0;
const HEAD_SPH_OFFSET: f32 = 50.0;
const SEG_STEP: f32 = 0.5;
const SEG_Z_ACTIVE: f32 = -10.0;
const SEG_Z_IDLE: f32 = -5.0;
const SEG_FOLLOW_ACTIVE: f32 = 0.85;
const SEG_OFFSET_Y_ACTIVE: f32 = -5.0;
const SEG_CHAIN_Z: f32 = 70.0;
const SEG_SINK_PROBE: f32 = 200.0;
const BODY_SPH_R: f32 = 10.0;
const HEAD_SPH_NEAR_R: f32 = 40.0;
const HEAD_SPH_NEAR_LIFT: f32 = 60.0;
const HEAD_SPH_AIR_R: f32 = 60.0;
const HEAD_SPH_ATTACK_R: f32 = 80.0;
const HOOK_SPH_R: f32 = 300.0;
const PLAYER_TG_RADIUS: f32 = 30.0;

const GRACE_ATTACK: f32 = 600.0;
const RUN_AHEAD: f32 = 2000.0;
const OUTWARD_SWING: f32 = 2500.0;
const SWING_GAIN: f32 = 7.0;
const SWING_TICK: i32 = 0x514;
const RAND_CHANCE: f32 = 0.3535;
const COIN_FLIP: f32 = 0.5;
const EASY_BG_LIMIT: f32 = 3800.0;
const EASY_BG_SHRINK: f32 = 0.99;
const FAR_GROUND: f32 = 850.0;
const FAR_GROUND_PROBE: f32 = 2500.0;
const GROUND_INF: f32 = 1.0e9;
const GROUND_SINK: f32 = 90.0;
const GROUND_LIFT_MID: f32 = 100.0;
const GROUND_LIFT_LOW: f32 = 100.0;
const GROUND_LIFT_HIGH: f32 = 105.0;
const GROUND_EYE_LIFT: f32 = 90.0;
const ATTN_LIFT: f32 = 10.0;
const FLINCH_KNOCK: f32 = 80.0;
const FLINCH_FRAME: i16 = 7;
const FLINCH_WOBBLE: f32 = 500.0;
const FAIL_SPLIT: i32 = 13;
const LOW_GATE: f32 = 20.0;
const FAIL_RISE_Y: f32 = 60.0;
const FAIL_RISE_Z: f32 = -30.0;
const CULL_MIN: Xyz = [-3000.0, -3000.0, -3000.0];
const CULL_MAX: Xyz = [3000.0, 3000.0, 3000.0];

const SE_CV_BWD_C_HS_DAMAGE: u32 = 0x48E3;
const SE_CV_BWD_C_ATTACK: u32 = 0x48E4;
const SE_CV_BWD_C_DAMAGE: u32 = 0x48E5;
const SE_CV_BWD_C_DIE: u32 = 0x48E6;
const SE_LK_HS_SPIKE: u32 = 0x286F;
const SE_LK_SW_HIT_S: u32 = 0x2803;
const SE_LK_ARROW_HIT: u32 = 0x2879;
const SE_LK_HAMMER_HIT: u32 = 0x2855;
const SE_CM_BWD_C_MOVE_SAND: u32 = 0x61E4;
const SE_CM_BWD_C_JUMP_OUT: u32 = 0x69E2;
const SE_CM_BWD_C_IN_SAND: u32 = 0x69E3;

const REACH_SWORD: f32 = 20.0;
const REACH_ARROW: f32 = 35.0;
const REACH_HAMMER: f32 = 50.0;
const REACH_HOOKSHOT: f32 = 5.0;
const REACH_BOOMERANG: f32 = 90.0;
const HEART_PARAM: u32 = 0;

const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
const RND_SEED: i32 = 100;

const REG0_F0: f32 = 0.0;
const REG0_F1: f32 = 0.0;
const REG0_F2: f32 = 0.0;
const REG0_F3: f32 = 0.0;
const REG0_F4: f32 = 0.0;
const REG0_F5: f32 = 0.0;
const REG0_F6: f32 = 0.0;
const REG0_F7: f32 = 0.0;
const REG0_F8: f32 = 0.0;
const REG0_F9: f32 = 0.0;
const REG0_F10: f32 = 0.0;
const REG0_F11: f32 = 0.0;
const REG0_F12: f32 = 0.0;
const REG0_F13: f32 = 0.0;
const REG0_F14: f32 = 0.0;
const REG0_F15: f32 = 0.0;
const REG0_F16: f32 = 0.0;
const REG0_F17: f32 = 0.0;
const REG0_F19: f32 = 0.0;
const REG0_F20: f32 = 0.0;
const REG0_S2: i16 = 0;
const REG0_S3: i16 = 0;
const REG0_S4: i16 = 0;
const REG0_S5: i16 = 0;
const REG0_S6: i16 = 0;
const REG0_S8: i16 = 0;
const REG6_F7: f32 = 0.0;

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Statics {
    pub hio_set: u8,
}

impl Default for Statics {
    fn default() -> Self {
        Statics { hio_set: 0 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frame {
    pub attribute: u8,
    pub state: u8,
    pub start: i16,
    pub end: i16,
    pub loop_point: i16,
    pub rate: f32,
    pub frame: f32,
    pub cur_morf: f32,
    pub prev_morf: f32,
    pub morf_step: f32,
}

impl Frame {
    fn idle() -> Self {
        Self {
            attribute: EMODE_NONE,
            state: 0,
            start: 0,
            end: 0,
            loop_point: 0,
            rate: 1.0,
            frame: 0.0,
            cur_morf: 1.0,
            prev_morf: 1.0,
            morf_step: 0.0,
        }
    }

    fn set_morf(&mut self, morf: f32) {
        if self.prev_morf < 0.0 || morf <= 0.0 {
            self.cur_morf = 1.0;
        } else {
            self.cur_morf = 0.0;
            self.morf_step = 1.0 / morf;
        }
        self.prev_morf = self.cur_morf;
    }

    fn update(&mut self) {
        self.state = 0;
        self.frame += self.rate;
        let start = (self.start) as f32;
        let end = (self.end) as f32;
        match self.attribute {
            EMODE_NONE | EMODE_LOOP_ONCE => {
                if self.frame < start {
                    self.frame = start;
                    self.rate = 0.0;
                    self.state |= STATE_STOP;
                }
                if self.frame >= end {
                    self.frame = end - 0.001;
                    self.rate = 0.0;
                    self.state |= STATE_STOP;
                }
            }
            EMODE_LOOP => {
                let low = i32::from(self.loop_point) - i32::from(self.start);
                let high = i32::from(self.end) - i32::from(self.loop_point);
                while self.frame < start {
                    self.state |= EMODE_LOOP;
                    if low <= 0 {
                        break;
                    }
                    self.frame += (low) as f32;
                }
                while self.frame >= end {
                    self.state |= EMODE_LOOP;
                    if high <= 0 {
                        break;
                    }
                    self.frame -= (high) as f32;
                }
            }
            _ => {}
        }
    }

    fn play(&mut self) {
        if self.cur_morf < 1.0 {
            self.prev_morf = self.cur_morf;
            super::walk::chase_f(&mut self.cur_morf, 1.0, self.morf_step);
        }
        self.update();
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sph {
    pub centre: Xyz,
    pub radius: f32,
    pub at_set: bool,
    pub at_atp: u8,
    pub co_set: bool,
    pub tg_set: bool,
    pub tg_shield: bool,
    pub tg_hookshot_only: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum At {
    Idle,
    Sword,
    Arrow,
    FireArrow,
    IceArrow,
    LightArrow,
    Hammer,
    Bomb,
    Boomerang,
    Hookshot,
}

impl At {
    fn reach(self) -> f32 {
        match self {
            At::Idle => 0.0,
            At::Sword => REACH_SWORD,
            At::Hookshot => REACH_HOOKSHOT,
            At::Arrow | At::FireArrow | At::IceArrow | At::LightArrow => REACH_ARROW,
            At::Hammer => REACH_HAMMER,
            At::Bomb => super::bombflower::BLAST_RADIUS,
            At::Boomerang => REACH_BOOMERANG,
        }
    }

    fn power(self, blade: u8, soup: bool) -> i32 {
        match self {
            At::Sword => damage::sword_power(blade, soup),
            At::Arrow | At::FireArrow | At::IceArrow | At::LightArrow => damage::ARROW,
            At::Bomb => damage::BOMB,
            At::Boomerang => damage::BOMERANG,
            At::Hammer => super::hammer::AT_POWER,
            At::Idle | At::Hookshot => 0,
        }
    }

    fn se(self) -> u32 {
        match self {
            At::Arrow | At::FireArrow | At::IceArrow | At::LightArrow => SE_LK_ARROW_HIT,
            At::Hammer => SE_LK_HAMMER_HIT,
            At::Hookshot => SE_LK_HS_SPIKE,
            At::Idle | At::Sword | At::Bomb | At::Boomerang => SE_LK_SW_HIT_S,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Blow {
    pub at: At,
    pub blade: u8,
    pub soup: bool,
}

impl Blow {
    fn reach(&self) -> f32 {
        self.at.reach()
    }

    fn power(&self) -> i32 {
        self.at.power(self.blade, self.soup)
    }

    fn se(&self) -> u32 {
        self.at.se()
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Bwds {
    pub health: i32,
    pub max_health: i32,
    pub m02b4: u8,
    pub morph: Frame,
    pub m02f4: i16,
    pub m02f6: i16,
    pub m02f8: i16,
    pub m02fa: i8,
    pub m02fb: i8,
    pub m02fc: Xyz,
    pub m030c: f32,
    pub m0310: f32,
    pub m0314: [i16; 5],
    pub m031e: i16,
    pub m0320: f32,
    pub m0324: i16,
    pub m0326: i16,
    pub m0328: i16,
    pub m032c: [Xyz; SEG_COUNT],
    pub m03e0: [[i16; 3]; SEG_COUNT],
    pub m043c: [Xyz; SEG_COUNT],
    pub m04f0: i32,
    pub m04f4: i8,
    pub m04f5: i8,
    pub m04fc: i16,
    pub m0500: i16,
    pub m0502: i16,
    pub m0504: f32,
    pub m0544: Sph,
    pub m0670: [Sph; SEG_COUNT],
    pub m1804: Xyz,
    pub m1810: Xyz,
    pub m181c: i8,
    pub m18c8: u8,
    pub shape_angle: [i16; 3],
    pub speed: Vec3,
    pub speed_f: f32,
    pub eye_pos: Vec3,
    pub attention_flags: u32,
    pub attention_position: Vec3,
    pub old_pos: Xyz,
    pub seed: [i32; 3],
    pub entered: bool,
}

#[derive(SystemParam)]
pub struct Watch<'w, 's> {
    collision: Option<Res<'w, Collision>>,
    combat: Res<'w, Combat>,
    held: Res<'w, Held>,
    blasts: Res<'w, super::bombflower::Blasts>,
    hammer: Res<'w, super::hammer::Cut>,
    players: Query<'w, 's, (Entity, &'static Player)>,
    shafts: Query<'w, 's, &'static super::arrow::Arrow>,
    bolts: Query<'w, 's, &'static super::boomerang::Boomerang>,
    chains: Query<'w, 's, (Entity, &'static super::hookshot::Hookshot)>,
}

pub struct Sink<'a> {
    pub events: &'a mut SimEvents,
    pub health: &'a mut Health,
}

impl Sink<'_> {
    fn se(&mut self, id: u32) {
        self.events.list.push(SimEvent::Se(id));
    }

    fn hurt_player(&mut self, units: u8) {
        self.health.damage(units);
        self.events.list.push(SimEvent::Hurt);
    }
}

pub struct Spawn {
    pub parameters: u32,
    pub pos: Vec3,
    pub angle: [i16; 3],
    pub room: i8,
    pub set_id: u16,
}

pub struct Env {
    pub player_pos: Vec3,
    pub player_eye: Vec3,
    pub blow: Blow,
    pub solid: Option<Collision>,
    pub hooked: bool,
}

pub fn control(
    watch: Watch<'_, '_>,
    mut events: ResMut<SimEvents>,
    mut health: ResMut<Health>,
    mut statics: ResMut<Statics>,
    mut commands: Commands,
    mut bosses: Query<(Entity, &mut Actor, &mut Bwds), (With<Bwds>, Without<Player>)>,
) {
    let players = watch.players;
    let Some((_, player)) = players.iter().next() else {
        return;
    };
    let blow = {
        let mut found = At::Idle;
        if watch
            .chains
            .iter()
            .any(|(_, chain)| chain.phase != super::hookshot::Phase::Wait)
        {
            found = At::Hookshot;
        } else {
            for shaft in watch.shafts.iter() {
                if shaft.phase != super::arrow::Phase::Flying {
                    continue;
                }
                found = match shaft.kind {
                    super::arrow::TYPE_LIGHT => At::LightArrow,
                    super::arrow::TYPE_FIRE => At::FireArrow,
                    super::arrow::TYPE_ICE => At::IceArrow,
                    _ => At::Arrow,
                };
                break;
            }
            if found == At::Idle
                && watch
                    .bolts
                    .iter()
                    .any(|bolt| bolt.parameter == super::boomerang::MODE_THROW)
            {
                found = At::Boomerang;
            }
            if found == At::Idle
                && (watch.hammer.kind == super::hammer::CUT_HAMMER_SIDESWING
                    || watch.hammer.kind == super::hammer::CUT_HAMMER_FRONTSWING)
            {
                found = At::Hammer;
            }
            if found == At::Idle && watch.combat.active && watch.combat.frame >= 1 && watch.combat.frame < 6
            {
                found = At::Sword;
            }
            if found == At::Idle && !watch.blasts.list.is_empty() {
                found = At::Bomb;
            }
        }
        Blow {
            at: found,
            blade: watch.held.item,
            soup: watch.combat.soup,
        }
    };
    let solid = watch.collision.as_deref().cloned();
    let hooks: Vec<(Vec3, Vec3, super::hookshot::Phase, Option<Entity>)> = watch
        .chains
        .iter()
        .map(|(_, chain)| (chain.pos, chain.heading, chain.phase, chain.carry))
        .collect();
    let mut spawns: Vec<Spawn> = Vec::new();
    let mut slain: Vec<Entity> = Vec::new();
    for (entity, mut actor, mut larva) in &mut bosses {
        let head = Vec3::new(
            larva.m0544.centre[0],
            larva.m0544.centre[1],
            larva.m0544.centre[2],
        );
        let hooked = hooks
            .iter()
            .any(|(pos, heading, phase, carry)| match phase {
                super::hookshot::Phase::Pull => *carry == Some(entity),
                super::hookshot::Phase::Shot => super::hookshot::swept(
                    *pos,
                    *pos + *heading * super::hookshot::SHOT_STEP,
                    head,
                    HOOK_SPH_R * HIO_M008,
                ),
                _ => false,
            });
        let env = Env {
            player_pos: player.pos,
            player_eye: player.pos,
            blow,
            solid: solid.clone(),
            hooked,
        };
        let larva = &mut *larva;
        let mut sink = Sink {
            events: &mut events,
            health: &mut health,
        };
        larva.execute(&mut *actor, &env, &mut statics, &mut spawns, &mut sink);
        if larva.health <= 0 {
            slain.push(entity);
        }
    }
    for spawn in spawns {
        let actor = Actor {
            name: super::pickup::ITEM_ACTORS[0].to_owned(),
            parameters: spawn.parameters,
            pos: spawn.pos,
            angle: spawn.angle,
            set_id: spawn.set_id,
            room: spawn.room,
            ..Default::default()
        };
        commands.spawn((actor, super::pickup::Pickup::spawn(spawn.parameters)));
    }
    for entity in slain {
        if let Ok((_, _, larva)) = bosses.get(entity)
            && larva.m18c8 != 0
        {
            statics.hio_set = 0;
        }
        let _ = commands.entity(entity).despawn();
    }
}

fn rnd(seed: &mut [i32; 3]) -> f32 {
    for lane in 0..3 {
        seed[lane] = (seed[lane] * RND_MUL[lane]) % RND_MOD[lane];
    }
    let sum = (seed[0]) as f32 / RND_SCALE[0]
        + (seed[1]) as f32 / RND_SCALE[1]
        + (seed[2]) as f32 / RND_SCALE[2];
    (sum % 1.0).abs()
}

fn add_calc2(value: &mut f32, target: f32, scale: f32, max_step: f32) {
    if *value != target {
        let mut step = scale * (target - *value);
        if step > max_step {
            step = max_step;
        } else if step < -max_step {
            step = -max_step;
        }
        *value += step;
    }
}

fn add_calc0(value: &mut f32, scale: f32, max_step: f32) {
    let mut step = *value * scale;
    if step > max_step {
        step = max_step;
    } else if step < -max_step {
        step = -max_step;
    }
    *value -= step;
}

fn atan2s(y: f32, x: f32) -> i16 {
    super::walk::to_s16(y.atan2(x))
}

fn ssin(angle: i32) -> f32 {
    super::walk::from_s16(angle as i16).sin()
}

fn scos(angle: i32) -> f32 {
    super::walk::from_s16(angle as i16).cos()
}

fn spread(x: f32, z: f32) -> f32 {
    (x * x + z * z).sqrt()
}

fn length_of(v: Xyz) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn rot_x(v: Xyz, angle: i16) -> Xyz {
    let (s, c) = super::walk::from_s16(angle).sin_cos();
    [v[0], v[1] * c + v[2] * s, v[2] * c - v[1] * s]
}

fn rot_y(v: Xyz, angle: i16) -> Xyz {
    let (s, c) = super::walk::from_s16(angle).sin_cos();
    [v[0] * c - v[2] * s, v[1], v[0] * s + v[2] * c]
}

fn rot_z(v: Xyz, angle: i16) -> Xyz {
    let (s, c) = super::walk::from_s16(angle).sin_cos();
    [v[0] * c - v[1] * s, v[0] * s + v[1] * c, v[2]]
}

fn xyz(v: Vec3) -> Xyz {
    [v.x, v.y, v.z]
}

struct Base {
    pos: Vec3,
    yaw: i16,
    pitch: i16,
    roll: i16,
}

const ORIGIN: Vec3 = Vec3::ZERO;

impl Base {
    fn apply(&self, point: Xyz) -> Vec3 {
        let scaled = [
            point[0] * HIO_M008,
            point[1] * HIO_M008,
            point[2] * HIO_M008,
        ];
        let p = rot_z(scaled, self.roll);
        let p = rot_x(p, self.pitch);
        let p = rot_y(p, self.yaw);
        self.pos + Vec3::new(p[0], p[1], p[2])
    }
}

fn frame_max(bck: i32) -> i16 {
    match bck {
        BCK_KOBOSS_PAKUPAKU => FRAME_KOBOSS_PAKUPAKU,
        _ => FRAME_KOBOSS_CLOSE,
    }
}

impl Bwds {
    pub fn spawn(parameters: u32) -> Self {
        let mut m0670 = [Sph {
            centre: ZERO_XYZ,
            radius: SPH_RADIUS,
            at_set: false,
            at_atp: 0,
            co_set: false,
            tg_set: true,
            tg_shield: true,
            tg_hookshot_only: false,
        }; SEG_COUNT];
        m0670[0].at_atp = 2;
        m0670[0].tg_shield = false;
        Bwds {
            health: 4,
            max_health: 4,
            m02b4: (parameters & 0xFF) as u8,
            morph: Frame::idle(),
            m02f4: 0,
            m02f6: 0,
            m02f8: 0,
            m02fa: 0,
            m02fb: 0,
            m02fc: ZERO_XYZ,
            m030c: 0.0,
            m0310: 0.0,
            m0314: [0; 5],
            m031e: 0,
            m0320: 0.0,
            m0324: 0,
            m0326: 0,
            m0328: 0,
            m032c: [ZERO_XYZ; SEG_COUNT],
            m03e0: [[0, 0, 0]; SEG_COUNT],
            m043c: [ZERO_XYZ; SEG_COUNT],
            m04f0: 0,
            m04f4: 0,
            m04f5: 0,
            m04fc: 0,
            m0500: 0,
            m0502: 0,
            m0504: 0.0,
            m0544: Sph {
                centre: ZERO_XYZ,
                radius: SPH_RADIUS,
                at_set: false,
                at_atp: 0,
                co_set: false,
                tg_set: true,
                tg_shield: false,
                tg_hookshot_only: true,
            },
            m0670,
            m1804: ZERO_XYZ,
            m1810: ZERO_XYZ,
            m181c: 0,
            m18c8: 0,
            shape_angle: [0, 0, 0],
            speed: Vec3::ZERO,
            speed_f: 0.0,
            eye_pos: Vec3::ZERO,
            attention_flags: 0x0000_0001,
            attention_position: Vec3::ZERO,
            old_pos: ZERO_XYZ,
            seed: [RND_SEED; 3],
            entered: false,
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        true
    }

    fn rnd_f(&mut self, range: f32) -> f32 {
        rnd(&mut self.seed) * range
    }

    fn rnd_fx(&mut self, range: f32) -> f32 {
        range * (rnd(&mut self.seed) - 0.5) * 2.0
    }

    fn anm_init(&mut self, bck: i32, morf: f32, loop_mode: u8, speed: f32) {
        self.morph.set_morf(morf);
        self.morph.start = 0;
        self.morph.end = frame_max(bck);
        self.morph.loop_point = 0;
        self.morph.attribute = loop_mode;
        self.morph.rate = speed;
        self.morph.frame = 0.0;
        self.morph.state = 0;
    }

    fn search_player_angle_y(&self, env: &Env) -> i16 {
        atan2s(env.player_eye.x - self.eye_pos.x, env.player_eye.z - self.eye_pos.z)
    }

    fn execute(
        &mut self,
        actor: &mut Actor,
        env: &Env,
        statics: &mut Statics,
        spawns: &mut Vec<Spawn>,
        sink: &mut Sink<'_>,
    ) {
        if !self.entered {
            self.create_init(actor, statics);
            self.entered = true;
        }
        self.tick(actor, env, spawns, sink);
    }

    fn create_init(&mut self, actor: &Actor, statics: &mut Statics) {
        self.m02b4 = (actor.parameters & 0xFF) as u8;
        self.morph = Frame::idle();
        self.m02f4 = 0;
        self.m02f6 = ACTION_UG_MOVE;
        self.m02f8 = 0;
        self.m02fa = 0;
        self.m02fb = 0;
        self.m02fc = ZERO_XYZ;
        self.m030c = 0.0;
        self.m0310 = 0.0;
        self.m0314 = [0; 5];
        self.m031e = 0;
        self.m0320 = 0.0;
        self.m0324 = 0;
        self.m0326 = 0;
        self.m0328 = 0;
        let here = [actor.pos.x, actor.pos.y, actor.pos.z];
        for i in 0..SEG_COUNT {
            self.m032c[i] = here;
            self.m03e0[i] = [0, 0, 0];
            self.m043c[i] = here;
        }
        self.m04f0 = 0;
        self.m04f4 = 0;
        self.m04f5 = 0;
        self.m04fc = 0;
        self.m0500 = 0;
        self.m0502 = 0;
        self.m0504 = 0.0;
        self.m1804 = ZERO_XYZ;
        self.m1810 = ZERO_XYZ;
        self.m181c = 0;
        self.shape_angle = [0, 0, 0];
        self.speed = Vec3::ZERO;
        self.speed_f = 0.0;
        self.eye_pos = actor.pos;
        self.attention_position = actor.pos;
        self.attention_flags = 0x0000_0001;
        self.old_pos = here;
        self.seed = [RND_SEED; 3];
        self.health = self.max_health;
        for i in 0..SEG_COUNT {
            self.m0670[i].centre = here;
            self.m0670[i].radius = SPH_RADIUS;
            self.m0670[i].at_set = false;
            self.m0670[i].at_atp = if i == 0 { 2 } else { 0 };
            self.m0670[i].co_set = false;
            self.m0670[i].tg_set = true;
            self.m0670[i].tg_shield = i != 0;
            self.m0670[i].tg_hookshot_only = false;
        }
        self.m0544.centre = here;
        self.m0544.radius = SPH_RADIUS;
        self.m0544.at_set = false;
        self.m0544.at_atp = 0;
        self.m0544.co_set = false;
        self.m0544.tg_set = true;
        self.m0544.tg_shield = false;
        self.m0544.tg_hookshot_only = true;
        if self.m02b4 == 0x23 {
            self.m02f8 = -10;
        }
        if statics.hio_set == 0 {
            self.m18c8 = 1;
            statics.hio_set = 1;
        }
    }

    fn tick(
        &mut self,
        actor: &mut Actor,
        env: &Env,
        spawns: &mut Vec<Spawn>,
        sink: &mut Sink<'_>,
    ) {
        self.old_pos = [actor.pos.x, actor.pos.y, actor.pos.z];
        self.m02f4 = self.m02f4.wrapping_add(1);
        for i in 0..5 {
            if self.m0314[i] != 0 {
                self.m0314[i] -= 1;
            }
        }
        if self.m031e != 0 {
            self.m031e -= 1;
        }
        if self.m0328 != 0 {
            self.m0328 -= 1;
        }

        if HIO_M005 == 0 {
            self.m0670[0].at_set = false;
            self.m0670[0].co_set = false;
            self.move_step(actor, env, spawns, sink);
            self.easy_bg_check2(actor);
            self.m1810[0] = actor.pos.x;
            self.m1810[2] = actor.pos.z;
            let probe = actor.pos + Vec3::new(0.0, REG0_F13 + FAR_GROUND_PROBE, 0.0);
            if let Some(cross) = env.solid.as_ref().and_then(|s| s.ground_at(probe.x, probe.z))
                && cross < FAR_GROUND
                && cross != -GROUND_INF
            {
                self.m1804[1] = cross;
                self.m1810[1] = cross;
                self.m0504 = ((cross - 60.0) - 30.0 + REG0_F16) as i32 as f32;
            }
            self.morph.play();
        }

        self.shape_angle[1] = actor.angle[1];
        self.shape_angle[0] = actor.angle[0];
        let wobble = self.m0328 as f32 * FLINCH_WOBBLE;
        let s1 = (wobble * ssin(self.m02f4 as i32 * 0x2100)) as i16;
        let s2 = (wobble * scos(self.m02f4 as i32 * 0x2300)) as i16;
        let head = Base {
            pos: actor.pos,
            yaw: self.shape_angle[1].wrapping_add(s1),
            pitch: self.shape_angle[0].wrapping_add(s2),
            roll: self.shape_angle[2],
        };

        self.eye_pos = head.apply(HEAD_BIND);
        if self.m02fa != 0 {
            self.eye_pos.y += GROUND_EYE_LIFT;
        }
        self.attention_position = self.eye_pos;
        self.attention_position.y += ATTN_LIFT;

        let mut centre = head.apply([REG0_F0 + HEAD_SPH_OFFSET, REG0_F1, REG0_F2]);
        self.m0670[0].centre = xyz(centre);
        if self.m02fa != 0 {
            centre.y += HEAD_SPH_NEAR_LIFT;
            self.m0670[0].radius = (REG0_F11 + HEAD_SPH_NEAR_R) * HIO_M008;
        } else if self.m04f5 == 1 {
            self.m0670[0].radius = (REG0_F11 + HEAD_SPH_ATTACK_R) * HIO_M008;
        } else {
            self.m0670[0].radius = (REG0_F10 + HEAD_SPH_AIR_R) * HIO_M008;
        }
        self.m0544.centre = xyz(centre);
        self.m0544.radius = HIO_M01C * HIO_M008;
        self.attack_player(env, sink);

        self.body_control(&head, env);
        self.m04f5 = 0;

        self.sand_effect(actor, sink);
    }

    fn attack_player(&mut self, env: &Env, sink: &mut Sink<'_>) {
        if !self.m0670[0].at_set || self.m0670[0].at_atp == 0 {
            return;
        }
        let centre = Vec3::new(
            self.m0670[0].centre[0],
            self.m0670[0].centre[1],
            self.m0670[0].centre[2],
        );
        if (env.player_pos - centre).length() <= self.m0670[0].radius + PLAYER_TG_RADIUS {
            sink.hurt_player(self.m0670[0].at_atp);
        }
    }

    fn sand_effect(&mut self, actor: &Actor, sink: &mut Sink<'_>) {
        if self.m181c == 0 {
            return;
        }
        if self.m181c == 1 {
            self.m181c += 1;
        }
        if self.m181c < 0 || actor.pos.y > self.m1804[1] {
            self.m181c = 0;
        }
        if self.m181c != 0 {
            sink.se(SE_CM_BWD_C_MOVE_SAND);
        }
    }

    fn easy_bg_check2(&mut self, actor: &mut Actor) {
        for _ in 0..0x14 {
            let local = [actor.pos.x, 0.0, actor.pos.z];
            let mut reach = local[0].abs();
            reach = local[1].abs() + local[2].abs() + reach;
            if !(reach > EASY_BG_LIMIT) {
                break;
            }
            actor.pos.x *= EASY_BG_SHRINK;
            actor.pos.z *= EASY_BG_SHRINK;
        }
    }

    fn pos_move(&mut self, actor: &mut Actor, param_2: i16) {
        let delta = [
            self.m02fc[0] - actor.pos.x,
            self.m02fc[1] - actor.pos.y,
            self.m02fc[2] - actor.pos.z,
        ];
        let target = param_2.wrapping_add(atan2s(delta[0], delta[2]));
        let max_step = (self.m030c * self.m0310) as i16;
        super::stream::add_calc_angle_s2(&mut actor.angle[1], target, REG0_S3 + 5, max_step);
        add_calc2(&mut self.m0310, 1.0, 1.0, 0.05);
        let local = rot_y([0.0, 0.0, self.speed_f], actor.angle[1]);
        self.speed.x = local[0];
        self.speed.z = local[2];
        actor.pos += self.speed;
        actor.angle[0] = -atan2s(self.speed.y, spread(self.speed.x, self.speed.z));
    }

    fn move_step(
        &mut self,
        actor: &mut Actor,
        env: &Env,
        spawns: &mut Vec<Spawn>,
        sink: &mut Sink<'_>,
    ) {
        match self.m02f6 {
            ACTION_UG_MOVE => self.ug_move(actor, env, sink),
            ACTION_HOOK_ON => self.hook_on(actor, env),
            ACTION_HOOK_CHANCE => self.hook_chance(actor),
            ACTION_FAIL => self.fail(actor, env, spawns),
            _ => {}
        }
        if actor.pos.y <= self.m0504 + LOW_GATE {
            self.m02fa = 1;
        } else {
            self.m02fa = 0;
        }
        self.damage_check(env, sink);
        self.m02fb = 0;

        if self.m0320 > 0.01 {
            let local = rot_y([0.0, 0.0, self.m0320], self.m0324);
            let local = rot_x(local, self.m0326);
            actor.pos += Vec3::new(local[0], local[1], local[2]);
            add_calc0(&mut self.m0320, 1.0, 7.0);
        }
    }

    fn ug_move(&mut self, actor: &mut Actor, env: &Env, sink: &mut Sink<'_>) {
        let mut s_var5: i16 = 0;
        match self.m02f8 {
            -10 => {
                self.anm_init(BCK_KOBOSS_PAKUPAKU, 2.0, EMODE_LOOP_ONCE, 1.0);
                self.m02f8 = 3;
                self.m030c = REG0_F3 + 3000.0;
                self.speed_f = HIO_M010;
                self.speed.y = REG0_F13 + 60.0 + self.rnd_f(20.0);
                actor.angle[1] = self.rnd_fx(0x8000 as f32) as i16;
                let local = rot_y([0.0, 0.0, RUN_AHEAD], actor.angle[1]);
                self.m02fc = [
                    actor.pos.x + local[0],
                    actor.pos.y + local[1],
                    actor.pos.z + local[2],
                ];
                if self.rnd_f(1.0) < RAND_CHANCE {
                    self.m0500 = 0;
                } else {
                    self.m0500 = self.rnd_fx(REG0_F9 + 3000.0) as i16;
                }
                sink.se(SE_CM_BWD_C_JUMP_OUT);
                sink.se(SE_CV_BWD_C_ATTACK);
            }
            0 => {
                self.anm_init(BCK_KOBOSS_CLOSE, 2.0, EMODE_NONE, 1.0);
                actor.pos.y = self.m0504;
                self.m02f8 += 1;
                self.shape_angle[2] = 0;
                self.m181c = 1;
                self.ug_move_chase(actor, env, &mut s_var5);
            }
            1 => self.ug_move_chase(actor, env, &mut s_var5),
            2 => {
                self.m0670[0].at_set = true;
                self.m0670[0].co_set = true;
                self.ug_move_fall(actor, sink);
            }
            3 => self.ug_move_fall(actor, sink),
            4 => self.ug_move_sink(),
            5 => self.ug_move_hunt(actor),
            _ => {}
        }
        self.pos_move(actor, s_var5);
        if self.speed.y < 0.0 {
            self.shape_angle[2] = self.shape_angle[2].wrapping_add(self.m0500);
        }
    }

    fn ug_move_chase(&mut self, actor: &mut Actor, env: &Env, s_var5: &mut i16) {
        actor.pos.y = self.m0504;
        self.attention_flags = 0x0000_0001;
        self.m02fc = xyz(env.player_pos);
        self.m030c = REG0_F3 + 3000.0;
        self.speed_f = HIO_M010;
        let delta = [
            self.m02fc[0] - actor.pos.x,
            self.m02fc[1] - actor.pos.y,
            self.m02fc[2] - actor.pos.z,
        ];
        let f_var8 = length_of(delta);
        let mut f_var1 = f_var8 * SWING_GAIN;
        if f_var1 > HIO_M018 {
            f_var1 = HIO_M018;
        }
        *s_var5 = (f_var1 * ssin(self.m02f4 as i32 * (i32::from(REG0_S3) + SWING_TICK))) as i16;
        if f_var8 < GRACE_ATTACK + REG0_F12 {
            self.anm_init(BCK_KOBOSS_PAKUPAKU, 2.0, EMODE_LOOP_ONCE, 1.0);
            self.m02f8 += 1;
            self.speed_f = HIO_M010;
            self.speed.y = REG0_F13 + 50.0;
            actor.angle[1] = self.search_player_angle_y(env);
            let local = rot_y([0.0, 0.0, RUN_AHEAD], actor.angle[1]);
            self.m02fc = [
                actor.pos.x + local[0],
                actor.pos.y + local[1],
                actor.pos.z + local[2],
            ];
            if self.rnd_f(1.0) < RAND_CHANCE {
                self.m0500 = 0;
            } else {
                self.m0500 = self.rnd_fx(REG0_F9 + 3000.0) as i16;
            }
        }
    }

    fn ug_move_fall(&mut self, actor: &mut Actor, sink: &mut Sink<'_>) {
        self.m02fb = 1;
        self.speed.y -= REG0_F6 + 2.7;
        if actor.pos.y <= self.m0504 {
            self.m02f8 = 4;
            self.speed_f = REG0_F5 + 5.0;
            self.m0314[0] = REG0_S3 + 0x23;
            self.attention_flags = 0;
            self.m04f5 = 2;
            sink.se(SE_CM_BWD_C_IN_SAND);
        }
    }

    fn ug_move_sink(&mut self) {
        self.m04f5 = 2;
        self.speed.y = REG0_F8 + -20.0;
        if self.m0314[0] == 0 {
            self.m02f8 = 5;
            self.speed.y = 0.0;
            self.speed_f = HIO_M010;
            self.m0314[0] = 0x1e;
            self.m0314[1] = (self.rnd_f(50.0) + 120.0) as i16;
            self.m0314[2] = (self.rnd_f(30.0) + 30.0) as i16;
        }
    }

    fn ug_move_hunt(&mut self, actor: &mut Actor) {
        self.shape_angle[2] = 0;
        self.m0500 = 0;
        if self.m0314[0] == 0 {
            self.attention_flags = 0x0000_0001;
            self.m0314[0] = (self.rnd_f(20.0) + 20.0) as i16;
            self.m02fc[0] = actor.pos.x;
            self.m02fc[1] = actor.pos.y;
            self.m02fc[2] = actor.pos.z;
            self.m02fc[0] += self.rnd_fx(REG0_F12 + 600.0);
            self.m02fc[2] += self.rnd_fx(REG0_F12 + 600.0);
        }
        if self.m0314[2] == 0 {
            self.m0314[2] = (self.rnd_f(60.0) + 40.0) as i16;
            self.speed.y = REG0_F14 + 30.0;
            actor.pos.y += self.speed.y;
        }
        self.speed.y -= REG0_F15 + 3.0;
        self.m030c = REG0_F4 + 1000.0;
        self.speed_f = HIO_M010;
        if actor.pos.y <= self.m0504 {
            actor.pos.y = self.m0504;
            self.speed.y = 0.0;
            if self.m0314[1] == 0 {
                self.m02f8 = 0;
            }
        }
    }

    fn hook_on(&mut self, actor: &mut Actor, env: &Env) {
        self.m031e = 3;
        match self.m02f8 {
            0 => {
                self.anm_init(BCK_KOBOSS_CLOSE, 2.0, EMODE_NONE, 1.0);
                self.m02f8 += 1;
                let local = [
                    env.player_eye.x - self.eye_pos.x,
                    env.player_eye.y - self.eye_pos.y,
                    env.player_eye.z - self.eye_pos.z,
                ];
                actor.angle[1] = atan2s(local[0], local[2]);
                actor.angle[0] = -atan2s(local[1], spread(local[0], local[2]));
                self.speed_f = 0.0;
            }
            1 => {
                if !env.hooked {
                    self.m02f6 = ACTION_HOOK_CHANCE;
                    self.m02f8 = 0;
                }
            }
            _ => {}
        }
    }

    fn hook_chance(&mut self, actor: &mut Actor) {
        self.m0670[0].co_set = true;
        match self.m02f8 {
            0 => {
                self.anm_init(BCK_KOBOSS_PAKUPAKU, 2.0, EMODE_LOOP_ONCE, 1.0);
                self.m02f8 += 1;
                self.m0314[0] = HIO_M014;
                self.hook_chance_lift(actor);
            }
            1 => self.hook_chance_lift(actor),
            _ => {}
        }
        actor.angle[1] = actor.angle[1].wrapping_add(self.m0502);
        super::stream::add_calc_angle_s2(&mut self.shape_angle[2], self.m0500, 1, REG0_S6 + 0x800);
        super::stream::add_calc_angle_s2(&mut actor.angle[0], self.m04fc, 1, REG0_S6 + 0x800);
        self.m04f5 = 1;
        actor.pos += self.speed;
        self.speed.y -= REG0_F4 + 5.0;
        self.m02fb = 1;
    }

    fn hook_chance_lift(&mut self, actor: &mut Actor) {
        if actor.pos.y <= self.m0504 + GROUND_LIFT_LOW + REG0_F14 {
            actor.pos.y = self.m0504 + GROUND_LIFT_LOW + REG0_F14;
            self.speed.y = REG6_F7 + self.rnd_f(REG0_F6 + 40.0);
            self.speed.x = self.rnd_fx(REG0_F8 + 5.0);
            self.speed.z = self.rnd_fx(REG0_F8 + 5.0);
            if self.rnd_f(1.0) < COIN_FLIP {
                self.m04fc = 0;
            } else {
                self.m04fc = REG0_S5 + 0x7f00;
            }
            self.m0500 = self.rnd_fx(0x8000 as f32) as i16;
            self.m0502 = self.rnd_fx(REG0_F5 + OUTWARD_SWING) as i16;
            if self.rnd_f(1.0) < COIN_FLIP {
                if self.rnd_f(1.0) < COIN_FLIP {
                    self.anm_init(BCK_KOBOSS_CLOSE, 5.0, EMODE_NONE, 1.0);
                } else {
                    self.anm_init(BCK_KOBOSS_PAKUPAKU, 5.0, EMODE_LOOP_ONCE, 1.0);
                }
            }
        }
        if self.m0314[0] == 0 {
            self.m02f6 = ACTION_UG_MOVE;
            self.m02f8 = 3;
        }
    }

    fn fail(&mut self, actor: &mut Actor, env: &Env, spawns: &mut Vec<Spawn>) {
        self.m031e = 3;
        self.m04f5 = 1;
        match self.m02f8 {
            0 => {
                let angle = self.search_player_angle_y(env);
                let local = rot_y(
                    [0.0, REG0_F19 + FAIL_RISE_Y, REG0_F20 + FAIL_RISE_Z],
                    angle,
                );
                self.speed = Vec3::new(local[0], local[1], local[2]);
                self.m02f8 += 1;
                self.m0314[0] = REG0_S4 + 10;
                self.fail_burst();
            }
            1 => self.fail_burst(),
            2 => {
                if actor.pos.y <= self.m0504 + GROUND_LIFT_HIGH + REG0_F14 {
                    spawns.push(Spawn {
                        parameters: HEART_PARAM,
                        pos: actor.pos,
                        angle: actor.angle,
                        room: actor.room,
                        set_id: actor.set_id,
                    });
                    self.health = 0;
                }
            }
            _ => {}
        }
        actor.pos += self.speed;
        self.speed.y -= REG0_F4 + 5.0;
        if actor.pos.y <= self.m0504 + GROUND_LIFT_LOW + REG0_F14 {
            actor.pos.y = self.m0504 + GROUND_LIFT_LOW + REG0_F14;
            self.speed.x = 0.0;
            self.speed.y = 0.0;
            self.speed.z = 0.0;
        } else {
            actor.angle[1] = atan2s(self.speed.x, self.speed.z);
            actor.angle[0] = -atan2s(self.speed.y, spread(self.speed.x, self.speed.z));
        }
    }

    fn fail_burst(&mut self) {
        if self.m0314[0] == 0 {
            self.m0314[0] = REG0_S5 + 1;
            self.m04f0 += 1;
            if self.m04f0 >= i32::from(REG0_S8) + FAIL_SPLIT {
                self.m02f8 += 1;
            }
        }
    }

    fn damage_check(&mut self, env: &Env, sink: &mut Sink<'_>) {
        if self.m031e != 0 {
            return;
        }
        if env.hooked {
            self.m02f6 = ACTION_HOOK_ON;
            self.m02f8 = 0;
            self.m031e = 10;
            sink.se(SE_LK_HS_SPIKE);
            sink.se(SE_CV_BWD_C_HS_DAMAGE);
            return;
        }
        if env.blow.at != At::Hookshot
            && env.blow.reach() > 0.0
            && self.m02fb == 1
            && self.tg_reaches(&self.m0670[0], env)
        {
            self.m031e = 6;
            let power = env.blow.power();
            if power > 0 {
                self.health -= power;
            }
            if self.health <= 0 {
                self.m02f6 = ACTION_FAIL;
                self.m02f8 = 0;
                sink.se(SE_CV_BWD_C_DIE);
            } else {
                self.m0320 = REG0_F11 + FLINCH_KNOCK;
                self.m0328 = REG0_S2 + FLINCH_FRAME;
                sink.se(SE_CV_BWD_C_DAMAGE);
            }
        }
        for i in 3..SEG_COUNT {
            if env.blow.at != At::Hookshot && self.tg_reaches(&self.m0670[i], env) {
                sink.se(env.blow.se());
                break;
            }
        }
        if self.m04f4 != 0 {
            self.health = 0;
        }
    }

    fn tg_reaches(&self, sph: &Sph, env: &Env) -> bool {
        if !sph.tg_set {
            return false;
        }
        if env.blow.reach() <= 0.0 {
            return false;
        }
        let delta = env.player_eye - Vec3::new(sph.centre[0], sph.centre[1], sph.centre[2]);
        delta.length() <= sph.radius + env.blow.reach()
    }

    fn body_control(&mut self, head: &Base, env: &Env) {
        self.m032c[0] = xyz(head.apply([REG0_F0 + HEAD_SEG_OFFSET, REG0_F1, REG0_F2]));
        self.m03e0[0] = self.shape_angle;

        let shape = Base {
            pos: ORIGIN,
            yaw: self.shape_angle[1],
            pitch: self.shape_angle[0],
            roll: 0,
        };
        let mut f_var1 = 0.0f32;
        let seg_z_local = if self.m04f5 != 0 {
            f_var1 = REG0_F5 + SEG_FOLLOW_ACTIVE;
            REG0_F6 + SEG_Z_ACTIVE
        } else {
            REG0_F17 + SEG_Z_IDLE
        };
        let mut vec3 = shape.apply([0.0, 0.0, seg_z_local]);
        if self.m04f5 == 1 {
            vec3.y += REG0_F15 + SEG_OFFSET_Y_ACTIVE;
        }
        let seg_y = vec3.y;
        let seg_z = vec3.z;

        for i in 1..SEG_COUNT {
            let f_var2 = (0xe - i) as f32 * (REG0_F17 + SEG_STEP) + 1.0;
            let mut f_var3 = self.m032c[i][1] + self.m043c[i][1] + seg_y * f_var2;
            if self.m04f5 == 1 {
                let probe = [
                    self.m032c[i][0],
                    self.m032c[i][1] + SEG_SINK_PROBE,
                    self.m032c[i][2],
                ];
                let floor = env
                    .solid
                    .as_ref()
                    .and_then(|s| s.ground_at(probe[0], probe[2]))
                    .unwrap_or(-GROUND_INF);
                let d_var8 = floor - GROUND_SINK + REG0_F16 + GROUND_LIFT_MID + REG0_F19;
                if f_var3 <= d_var8 {
                    f_var3 = d_var8;
                }
            }
            let diff_x = self.m043c[i][0]
                + (self.m032c[i][0] - self.m032c[i - 1][0])
                + seg_z * f_var2;
            let diff_y = f_var3 - self.m032c[i - 1][1];
            let diff_z = self.m043c[i][2]
                + (self.m032c[i][2] - self.m032c[i - 1][2])
                + seg_z * f_var2;

            let s_var2 = atan2s(diff_x, diff_z);
            let s_var1 = -atan2s(diff_y, spread(diff_x, diff_z));

            let chain = Base {
                pos: ORIGIN,
                yaw: s_var2,
                pitch: s_var1,
                roll: 0,
            };
            let step = chain.apply([0.0, 0.0, SEG_CHAIN_Z]);

            self.m03e0[i - 1][1] = s_var2;
            self.m03e0[i - 1][0] = s_var1.wrapping_add(0x8000);

            self.m043c[i] = self.m032c[i];
            self.m032c[i] = [
                self.m032c[i - 1][0] + step.x,
                self.m032c[i - 1][1] + step.y,
                self.m032c[i - 1][2] + step.z,
            ];
            self.m043c[i][0] = f_var1 * (self.m032c[i][0] - self.m043c[i][0]);
            self.m043c[i][1] = f_var1 * (self.m032c[i][1] - self.m043c[i][1]);
            self.m043c[i][2] = f_var1 * (self.m032c[i][2] - self.m043c[i][2]);

            if i >= 3 {
                self.m0670[i].centre = self.m032c[i];
                self.m0670[i].radius = (REG0_F9 + BODY_SPH_R) * HIO_M008;
            }
        }
    }
}