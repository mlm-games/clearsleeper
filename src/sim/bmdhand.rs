use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::{Mat4, Vec3, Vec4};

use super::actor::{Actor, atan2s};
use super::area::StageName;
use super::arrow;
use super::boomerang;
use super::collision::Collision;
use super::combat::Combat;
use super::events::{SimEvent, SimEvents};
use super::flags::MemBits;
use super::hammer;
use super::kalle_demos::KalleDemos;
use super::player::Player;
use super::walk;

pub const BMD_HAND_ACTOR: &str = "Bmdhand";
pub const BMDHAND_ACTORS: [&str; 1] = [BMD_HAND_ACTOR];

pub fn is_bmdhand(name: &str) -> bool {
    BMDHAND_ACTORS.contains(&name)
}

const HANDS: usize = 20;
const HEAD: usize = 0;
const HAND_LAST: usize = 0x13;
const HAND_REV_LAST: usize = 0x12;
const TIMER_COUNT: usize = 4;
const ATTENTION_LOCKON_BATTLE: u32 = 0x0000_0004;

const STATE_WAIT: i16 = 0;
const STATE_CUT: i16 = 1;
const STATE_ATTACH: i16 = 2;
const STATE_ENTER: i16 = 3;
const STATE_STOW: i16 = 4;
const STATE_RESTORE: i16 = 5;

const HAND_WAIT: i16 = 0;
const HAND_RISE: i16 = 1;
const HAND_SNAP: i16 = 2;

const ENTER_STANDBY: i16 = 0;
const ENTER_WAIT: i16 = 1;
const ENTER_RISE: i16 = 2;
const ENTER_HANG: i16 = 3;

const BOSS_CAM_ATTACH: i8 = 3;
const BOSS_CAM_RESTORE: i8 = 4;
const BOSS_CAM_DROP: i8 = 5;
const BOSS_CAM_RESET: i8 = 7;
const BOSS_CAM_GIVE: i8 = 9;
const BOSS_STATE_LANDING: i16 = 2;
const BOSS_VINE_CAP: i8 = 0x14;
const BOSS_CUT_TIME: i16 = 3;

const HIO_M08: f32 = -20.0;
const HIO_M0C: i16 = 1000;
const HIO_NO_INIT: i8 = -1;

const SE_CV_BKM_CUT_VINE_1: u32 = 0x4853;
const SE_CM_BKM_VINE_SET: u32 = 0x584B;
const SE_CM_BKM_CUT_VINE: u32 = 0x584E;

const PARTICLE_BLOOD: u16 = 1 << 0;
const PARTICLE_SIBOUBAKUEN: u16 = 1 << 1;
const PARTICLE_SIBOUFLASH: u16 = 1 << 2;

const REG0_S4: i32 = 0;
const REG0_S5: i32 = 0;
const REG0_S6: i32 = 0;
const REG0_S7: i32 = 0;
const REG0_S8: i32 = 0;
const REG0_S9: i32 = 0;
const REG0_F7: f32 = 0.0;
const REG0_F8: f32 = 0.0;
const REG0_F6: f32 = 0.0;
const REG0_F9: f32 = 0.0;
const REG0_F10: f32 = 0.0;
const REG0_FE: f32 = 0.0;
const REG7_F10: f32 = 0.0;
const REG8_S2: i32 = 0;
const REG8_S4: i32 = 0;
const REG12_F4: f32 = 0.0;
const REG12_F5: f32 = 0.0;
const REG12_F6: f32 = 0.0;
const REG13_S5: i32 = 0;
const REG13_S7: i32 = 0;
const REG13_F1: f32 = 0.0;
const REG13_F2: f32 = 0.0;
const REG13_F3: f32 = 0.0;
const REG13_F4: f32 = 0.0;
const REG13_F5: f32 = 0.0;
const REG13_F6: f32 = 0.0;
const REG13_F7: f32 = 0.0;
const REG14_F0: f32 = 0.0;
const REG14_F6: f32 = 0.0;
const REG14_F7: f32 = 0.0;
const REG14_F8: f32 = 0.0;
const REG14_FB: f32 = 0.0;
const REG14_F11: f32 = 0.0;
const REG14_FC: f32 = 0.0;

const AT_TYPE_ALL: u32 = u32::MAX;
const AT_TYPE_SWORD: u32 = 1 << 1;
const AT_TYPE_BOMB: u32 = 1 << 5;
const AT_TYPE_BOOMERANG: u32 = 1 << 6;
const AT_TYPE_WATER: u32 = 1 << 8;
const AT_TYPE_NORMAL_ARROW: u32 = 1 << 14;
const AT_TYPE_HOOKSHOT: u32 = 1 << 15;
const AT_TYPE_SKULL_HAMMER: u32 = 1 << 16;
const AT_TYPE_UNK20000: u32 = 1 << 17;
const AT_TYPE_FIRE_ARROW: u32 = 1 << 18;
const AT_TYPE_ICE_ARROW: u32 = 1 << 19;
const AT_TYPE_LIGHT_ARROW: u32 = 1 << 20;
const AT_TYPE_WIND: u32 = 1 << 21;
const AT_TYPE_UNK400000: u32 = 1 << 22;
const AT_TYPE_LIGHT: u32 = 1 << 23;

const TG_TYPE_ALL: u32 = AT_TYPE_ALL
    & !AT_TYPE_WATER
    & !AT_TYPE_UNK20000
    & !AT_TYPE_WIND
    & !AT_TYPE_UNK400000
    & !AT_TYPE_LIGHT;

const SRC_RADIUS: f32 = 100.0;
const SPH_A_RADIUS: f32 = 90.0;
const SPH_A_RADIUS_SET: f32 = 110.0;
const SPH_A_INDEX: usize = 10;
const STTS_INIT: u32 = 0xFFFF;

const PARAM_ODD: u32 = 1;
const PARAM_BRANCH: u32 = 0x1f;
const PARAM_XAD: u32 = 0x3;
const PARAM_YAW_STEP: i32 = -0xccc;
const PARAM_YAW_BASE: i32 = -13000;

const HAND_SIZE_A: f32 = 10.5;
const HAND_TAPER_LAST: i32 = 15;
const HAND_GROW_BASE: i32 = 15;
const HAND_GROW: f32 = 0.7;

const CX_ALT_A: f32 = 250.0;
const CX_ALT_B: f32 = 350.0;
const CX_SNAP_RANGE: f32 = 100.0;
const CX_RISE_STEP: f32 = 150.0;
const CX_RISE_RATE: f32 = 0.1;
const CX_SNAP_TIME: i16 = 0x14;
const CX_SNAP_JUMP: i16 = 0x28;
const CX_SNAP_RATE: f32 = 30.0;

const LINE_HEIGHT: f32 = 2500.0;
const LINE_MIN_Y: f32 = 5.0;
const LINE_OFF_Y: f32 = -20000.0;
const LINE_CUT_Y: f32 = -3.0;
const LINE_CUT_Y_STEP: f32 = 0.1;
const LINE_CUT_Z: f32 = 20.0;
const LINE_CUT_Z_STEP: f32 = 0.2;
const LINE_SWAY: f32 = 0.5;
const LINE_TAPER: f32 = 0.03763158;
const LINE_SETTLE: f32 = 10.0;

const CONTROL1_Y: f32 = -10.0;
const CONTROL1_Y_STEP: f32 = 0.1;
const CONTROL1_Z: f32 = 50.0;
const CONTROL1_Z_STEP: f32 = 1.0;
const CONTROL2_BEND: f32 = 10.0;
const CONTROL2_TRACK: f32 = 50.0;
const CONTROL2_EXTRA: f32 = 70.0;
const START_Y_STEP: f32 = 0.1;
const START_Z_STEP: f32 = 1.0;
const START_BEND: f32 = 5.0;
const START_TOP_Y: f32 = 10000.0;
const START_NEAR: f32 = 10.0;
const START_SPIN: f32 = 0.1;
const START_SPIN_GAIN: f32 = 2.0;
const START_FAR: f32 = 800.0;
const START_TRACK: f32 = 50.0;
const START_RISE: f32 = 10.0;
const START_RATE: f32 = 0.01;

const WAIT_TARGET: f32 = 30.0;
const WAIT_TARGET_RATE: f32 = 0.1;
const WAIT_TARGET_STEP: f32 = 1.0;
const WAIT_SWAY: f32 = 50.0;
const WAIT_SWAY_ADD: f32 = 20.0;
const WAIT_SWAY_RATE: f32 = 0.1;
const WAIT_SWAY_STEP: f32 = 0.5;
const SCALE_STEP: f32 = 0.01;

const CUT_TARGET: f32 = 20.0;
const CUT_TARGET_RATE: f32 = 0.1;
const CUT_TARGET_STEP: f32 = 0.5;
const CUT_RECOVER: i16 = 100;
const CUT_OFF: i16 = 0x14;
const ATTACH_STEP: f32 = 3.0;
const ATTACH_SWING: f32 = 40.0;
const ATTACH_SWAY: f32 = 30.0;
const ATTACH_SWAY_SPAN: f32 = 20.0;
const ATTACH_HOLD: f32 = 50.0;
const RESTORE_TIME: i16 = 0x46;
const RESTORE_SPIN: f32 = 0.2;
const STOW_SPIN: f32 = 0.1;
const STOW_SPIN_STEP: f32 = 0.2;
const ENTER_SWING: f32 = 30.0;
const ENTER_SWING_RATE: f32 = 0.1;
const ENTER_SWING_STEP: f32 = 0.5;
const ENTER_SWAY: f32 = 200.0;
const ENTER_SWAY_RATE: f32 = 0.1;
const ENTER_SWAY_STEP: f32 = 10.0;
const ENTER_SCALE_STEP: f32 = 0.02;
const VIS_PIVOT: f32 = 0.1;

const MORF_RATE: f32 = 5.0;
const FRAME_HIRAKU: i16 = 5;
const FRAME_TOJIRU: i16 = 5;
const ANM_HIRAKU: i32 = 0;
const ANM_TOJIRU: i32 = 1;

const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
const RND_SEED: i32 = 100;

const JNT_HANAA7: i32 = 28;
const JNT_HANAB7: i32 = 37;
const JNT_HANAC7: i32 = 46;
const JNT_HANAE7: i32 = 64;
const JNT_HANAD7: i32 = 55;
const JOINT_SLOTS: usize = 5;

const BOSS_JOINT_D: [i32; HANDS] = [
    JNT_HANAA7,
    JNT_HANAA7,
    JNT_HANAA7,
    JNT_HANAA7,
    JNT_HANAB7,
    JNT_HANAB7,
    JNT_HANAB7,
    JNT_HANAB7,
    JNT_HANAC7,
    JNT_HANAC7,
    JNT_HANAC7,
    JNT_HANAC7,
    JNT_HANAE7,
    JNT_HANAE7,
    JNT_HANAE7,
    JNT_HANAE7,
    JNT_HANAD7,
    JNT_HANAD7,
    JNT_HANAD7,
    JNT_HANAD7,
];

const BOSS_JOINT_XAD: [f32; 4] = [60.0, 20.0, -20.0, -60.0];

const JOINT_BIND: [[[f32; 4]; 3]; JOINT_SLOTS] = [
    [
        [0.86609, -0.49989, 0.0, -575.47529],
        [0.49989, 0.86609, 0.0, -34.57768],
        [0.0, 0.0, 1.0, 0.0],
    ],
    [
        [0.26765, -0.15448, -0.95105, -177.84214],
        [0.49989, 0.86609, 0.0, -34.57768],
        [0.82369, -0.47542, 0.30904, -547.30612],
    ],
    [
        [-0.70071, 0.40444, -0.58774, 465.58875],
        [0.49989, 0.86609, 0.0, -34.57768],
        [0.50903, -0.29380, -0.80905, -338.22911],
    ],
    [
        [-0.70066, 0.40441, 0.58782, 465.55632],
        [0.49989, 0.86609, 0.0, -34.57768],
        [-0.50910, 0.29384, -0.80899, 338.27374],
    ],
    [
        [0.26765, -0.15448, 0.95105, -177.84214],
        [0.49989, 0.86609, 0.0, -34.57768],
        [-0.82369, 0.47542, 0.30904, 547.30612],
    ],
];

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct BmdhandHio {
    pub no: i8,
    pub m08: f32,
    pub m0c: i16,
}

impl Default for BmdhandHio {
    fn default() -> Self {
        Self {
            no: HIO_NO_INIT,
            m08: HIO_M08,
            m0c: HIO_M0C,
        }
    }
}

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct BmdhandStatics {
    pub boss: Option<Entity>,
    pub hio_set: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Blow {
    Idle,
    Sword,
    Boomerang,
    Hookshot,
    Arrow,
    FireArrow,
    IceArrow,
    LightArrow,
    Hammer,
    Bomb,
}

impl Blow {
    fn reach(self) -> f32 {
        match self {
            Self::Sword => 20.0,
            Self::Arrow | Self::FireArrow | Self::IceArrow | Self::LightArrow => 35.0,
            Self::Boomerang => 90.0,
            Self::Hammer => 50.0,
            Self::Hookshot => 5.0,
            Self::Bomb => 150.0,
            Self::Idle => 0.0,
        }
    }

    fn cc_type(self) -> u32 {
        match self {
            Self::Sword => AT_TYPE_SWORD,
            Self::Bomb => AT_TYPE_BOMB,
            Self::Boomerang => AT_TYPE_BOOMERANG,
            Self::Hookshot => AT_TYPE_HOOKSHOT,
            Self::Arrow => AT_TYPE_NORMAL_ARROW,
            Self::FireArrow => AT_TYPE_FIRE_ARROW,
            Self::IceArrow => AT_TYPE_ICE_ARROW,
            Self::LightArrow => AT_TYPE_LIGHT_ARROW,
            Self::Hammer => AT_TYPE_SKULL_HAMMER,
            Self::Idle => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frame {
    pub end: i16,
    pub rate: f32,
    pub frame: f32,
    pub cur_morf: f32,
    pub prev_morf: f32,
    pub morf_step: f32,
    pub stop: bool,
}

impl Frame {
    fn init(end: i16, rate: f32) -> Self {
        Self {
            end,
            rate,
            frame: 0.0,
            cur_morf: 1.0,
            prev_morf: 1.0,
            morf_step: 0.0,
            stop: false,
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

    fn play(&mut self) {
        if self.cur_morf < 1.0 {
            self.prev_morf = self.cur_morf;
            add_calc2(&mut self.cur_morf, 1.0, 1.0, self.morf_step);
        }
        self.frame += self.rate;
        if self.frame >= f32::from(self.end) {
            self.frame = f32::from(self.end) - 0.001;
            self.rate = 0.0;
            self.stop = true;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sph {
    pub centre: Vec3,
    pub radius: f32,
    pub at_set: bool,
    pub at_vs_player: bool,
    pub at_type: u32,
    pub tg_set: bool,
    pub tg_type: u32,
    pub tg_hit: bool,
    pub co_set: bool,
}

impl Sph {
    const SRC: Sph = Sph {
        centre: Vec3::ZERO,
        radius: SRC_RADIUS,
        at_set: true,
        at_vs_player: true,
        at_type: 0,
        tg_set: true,
        tg_type: TG_TYPE_ALL,
        tg_hit: false,
        co_set: true,
    };

    fn touched(&self, blow: Blow, player: Vec3) -> bool {
        if !self.tg_set || blow.reach() <= 0.0 {
            return false;
        }
        if self.tg_type & blow.cc_type() == 0 {
            return false;
        }
        walk::within(self.centre, player, self.radius + blow.reach())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hand {
    pub m00: Vec3,
    pub m0c: Vec3,
    pub m18: f32,
}

impl Default for Hand {
    fn default() -> Self {
        Self {
            m00: Vec3::ZERO,
            m0c: Vec3::ZERO,
            m18: 0.0,
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Bmdhand {
    pub health: i32,
    pub max_health: i32,
    pub m2b8: i16,
    pub m2ba: i16,
    pub m2bc: i16,
    pub m2be: i16,
    pub m2c0: [i16; TIMER_COUNT],
    pub m2c8: i16,
    pub m2ca: i16,
    pub m2cc: Vec3,
    pub m2d8: Vec3,
    pub m2e4: Vec3,
    pub m2f0: Vec3,
    pub m2fc: f32,
    pub m300: i16,
    pub m302: i16,
    pub m304: i8,
    pub m308: f32,
    pub m30c: f32,
    pub m310: f32,
    pub m314: f32,
    pub m318: f32,
    pub m31c: f32,
    pub m320: f32,
    pub m324: [Hand; HANDS],
    pub sph_a: Sph,
    pub sph_b: Sph,
    pub anm: i32,
    pub morph: Frame,
    pub attention_flags: u32,
    pub attention: Vec3,
    pub status: u32,
    pub hio_set: bool,
    pub particles: u16,
    pub seed: [i32; 3],
    pub entered: bool,
}

impl Bmdhand {
    pub fn spawn(parameters: u32) -> Self {
        let _ = parameters;
        Self {
            health: 2,
            max_health: 2,
            m2b8: 0,
            m2ba: STATE_WAIT,
            m2bc: HAND_WAIT,
            m2be: ENTER_STANDBY,
            m2c0: [0; TIMER_COUNT],
            m2c8: 0,
            m2ca: 0,
            m2cc: Vec3::ZERO,
            m2d8: Vec3::ZERO,
            m2e4: Vec3::ZERO,
            m2f0: Vec3::ZERO,
            m2fc: 0.0,
            m300: 0,
            m302: 0,
            m304: 0,
            m308: 0.0,
            m30c: 0.0,
            m310: 0.0,
            m314: 0.0,
            m318: 0.0,
            m31c: 0.0,
            m320: 0.0,
            m324: [Hand::default(); HANDS],
            sph_a: Sph {
                radius: SPH_A_RADIUS_SET,
                ..Sph::SRC
            },
            sph_b: Sph::SRC,
            anm: ANM_HIRAKU,
            morph: Frame::init(FRAME_HIRAKU, 1.0),
            attention_flags: 0,
            attention: Vec3::ZERO,
            status: STTS_INIT,
            hio_set: false,
            particles: 0,
            seed: [RND_SEED; 3],
            entered: false,
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        let mut min = self.m324[HEAD].m00;
        let mut max = min;
        for hand in &self.m324 {
            min = min.min(hand.m00);
            max = max.max(hand.m00);
        }
        let pad = Vec3::splat(SPH_A_RADIUS);
        ((min - pad).to_array(), (max + pad).to_array())
    }

    pub fn drawn(&self) -> bool {
        self.m310 > VIS_PIVOT
    }

    fn rnd_f(&mut self, range: f32) -> f32 {
        rnd(&mut self.seed) * range
    }

    pub fn create(
        &mut self,
        hio: &mut BmdhandHio,
        statics: &mut BmdhandStatics,
        boss_demo: bool,
        start_stage_first: u8,
    ) {
        if !statics.hio_set {
            statics.hio_set = true;
            self.hio_set = true;
            hio.no = HIO_NO_INIT;
        }
        self.health = 2;
        self.m2b8 = self.rnd_f(10000.0) as i32 as i16;
        statics.boss = None;
        self.status = STTS_INIT;
        self.sph_a = Sph {
            radius: SPH_A_RADIUS_SET,
            ..Sph::SRC
        };
        self.sph_b = Sph::SRC;
        self.sph_a.radius = SPH_A_RADIUS;
        if !boss_demo && start_stage_first != b'X' {
            self.m2ba = STATE_ENTER;
        }
    }

    fn hand_open(&mut self) {
        self.anm = ANM_HIRAKU;
        self.morph = Frame::init(FRAME_HIRAKU, 1.0);
        self.morph.set_morf(REG0_F6 + MORF_RATE);
    }

    fn hand_close(&mut self) {
        self.anm = ANM_TOJIRU;
        self.morph = Frame::init(FRAME_TOJIRU, 1.0);
        self.morph.set_morf(REG0_F6 + MORF_RATE);
    }

    fn hand_sizes(&mut self) {
        for index in 0..HANDS {
            if index < 10 {
                self.m324[index].m18 = HAND_SIZE_A;
            } else {
                self.m324[index].m18 = (HAND_GROW_BASE - (index as i32 - 10)) as f32 * HAND_GROW;
            }
        }
    }

    fn control1(&mut self, actor: &Actor) {
        self.m324[HEAD].m00 = actor.pos;
        let local_94 = Vec3::new(0.0, self.m314, self.m318);
        let local_ac = turned(actor.angle[1], local_94);
        add_calc2(&mut self.m314, REG0_F7 + CONTROL1_Y, 1.0, CONTROL1_Y_STEP);
        add_calc2(&mut self.m318, REG0_F8 + CONTROL1_Z, 1.0, CONTROL1_Z_STEP);
        let local_94 = Vec3::new(0.0, self.m314, self.m310);
        let dvar9 = self.m31c;
        for i in 1..HAND_LAST {
            let local_c4 = Vec3::new(
                dvar9
                    * walk::from_s16(sni(
                        i32::from(self.m2b8) * (REG0_S5 + 0x44c)
                            + i as i32 * (REG0_S6 + 4000),
                    ))
                    .sin(),
                0.0,
                dvar9
                    * walk::from_s16(sni(
                        i32::from(self.m2b8) * (REG0_S7 + 800)
                            + i as i32 * (REG0_S8 + 4000),
                    ))
                    .cos(),
            );
            let fvar1 = if (i as i32) < HAND_TAPER_LAST {
                1.0
            } else {
                1.0 - (i as i32 - HAND_TAPER_LAST) as f32 * 0.2
            };
            let dvar8 = self.m324[i].m00.x - self.m324[i - 1].m00.x
                + local_ac.x * fvar1
                + local_c4.x * fvar1;
            let dvar10 = self.m324[i].m00.y - self.m324[i - 1].m00.y + local_ac.y;
            let dvar7 = self.m324[i].m00.z - self.m324[i - 1].m00.z
                + local_ac.z * fvar1
                + local_c4.z * fvar1;
            let yaw = atan2s(dvar8, dvar7);
            let pitch = -atan2s(dvar10, (dvar8 * dvar8 + dvar7 * dvar7).sqrt());
            let point = mtx_position(yaw, pitch, local_94);
            self.m324[i].m00 = self.m324[i - 1].m00 + point;
        }
    }

    fn control2(&mut self) {
        let step = self.m30c * CONTROL2_TRACK;
        add_calc2(&mut self.m324[HAND_LAST].m00.x, self.m2d8.x, 1.0, step);
        add_calc2(&mut self.m324[HAND_LAST].m00.y, self.m2d8.y, 1.0, step);
        add_calc2(&mut self.m324[HAND_LAST].m00.z, self.m2d8.z, 1.0, step);
        add_calc2(&mut self.m30c, 1.0, 1.0, SCALE_STEP);
        for i in (1..=HAND_REV_LAST).rev() {
            let fvar1 = self.m324[i].m00.x - self.m324[i + 1].m00.x;
            let dvar9 = (self.m324[i].m00.y - self.m324[i + 1].m00.y) - CONTROL2_BEND;
            let fvar2 = self.m324[i].m00.z - self.m324[i + 1].m00.z;
            let yaw = atan2s(fvar1, fvar2);
            let pitch = -atan2s(dvar9, (fvar1 * fvar1 + fvar2 * fvar2).sqrt());
            let local = if i == HAND_REV_LAST {
                Vec3::new(0.0, 0.0, self.m310 + CONTROL2_EXTRA + REG12_F6)
            } else {
                Vec3::new(0.0, 0.0, self.m310)
            };
            let point = mtx_position(yaw, pitch, local);
            self.m324[i].m00 = self.m324[i + 1].m00 + point;
        }
        self.m2f0 = self.m324[HAND_LAST].m00;
        let local = self.m324[HAND_REV_LAST].m00 - self.m324[HAND_LAST].m00;
        self.m300 = -atan2s(local.y, local.z);
        self.m302 = atan2s(local.x, (local.y * local.y + local.z * local.z).sqrt());
    }

    fn cut_control(&mut self, actor: &Actor, boss_ground: f32) {
        self.m324[HEAD].m00 = actor.pos;
        let local_e8 = Vec3::new(0.0, self.m314, self.m318);
        let local_100 = turned(actor.angle[1], local_e8);
        add_calc2(
            &mut self.m314,
            REG14_F7 + LINE_CUT_Y,
            1.0,
            REG12_F4 + LINE_CUT_Y_STEP,
        );
        add_calc2(
            &mut self.m318,
            REG14_F8 + LINE_CUT_Z + 10.0,
            1.0,
            REG12_F5 + LINE_CUT_Z_STEP,
        );
        add_calc0(&mut self.m31c, 1.0, REG12_F6 + 1.0);
        let local_e8 = Vec3::new(0.0, self.m314, self.m310);
        let dvar9 = REG7_F10 + LINE_SWAY;
        for i in 1..HANDS {
            let local_124 = Vec3::new(
                self.m31c
                    * walk::from_s16(sni(
                        i32::from(self.m2b8) * (REG0_S4 + 0xdac)
                            + i as i32 * (REG0_S5 + 4000),
                    ))
                    .sin(),
                self.m31c
                    * walk::from_s16(sni(
                        i32::from(self.m2b8) * (REG0_S6 + 4000)
                            + i as i32 * (REG0_S7 + 4000),
                    ))
                    .cos(),
                self.m31c
                    * walk::from_s16(sni(
                        i32::from(self.m2b8) * (REG0_S8 + 0xed8)
                            + i as i32 * (REG0_S9 + 4000),
                    ))
                    .cos(),
            );
            let factor = 1.0 - i as f32 * (REG0_F9 + LINE_TAPER);
            let scaled = Vec3::new(local_100.x * factor, local_100.y, local_100.z * factor);
            let fvar_x = self.m324[i].m0c.x
                + (self.m324[i].m00.x - self.m324[i - 1].m00.x + scaled.x + local_124.x);
            let mut fvar_y = self.m324[i].m0c.y + (self.m324[i].m00.y + local_100.y + local_124.y);
            if fvar_y < LINE_MIN_Y + boss_ground {
                fvar_y = LINE_MIN_Y + boss_ground;
            }
            let delta_y = fvar_y - self.m324[i - 1].m00.y;
            let fvar_z = self.m324[i].m0c.z
                + (self.m324[i].m00.z - self.m324[i - 1].m00.z + scaled.z + local_124.z);
            let yaw = atan2s(fvar_x, fvar_z);
            let pitch = -atan2s(delta_y, (fvar_x * fvar_x + fvar_z * fvar_z).sqrt());
            let point = mtx_position(yaw, pitch, local_e8);
            self.m324[i].m0c = self.m324[i].m00;
            self.m324[i].m00 = self.m324[i - 1].m00 + point;
            self.m324[i].m0c.x = dvar9 * (self.m324[i].m00.x - self.m324[i].m0c.x);
            self.m324[i].m0c.y = dvar9 * (self.m324[i].m00.y - self.m324[i].m0c.y);
            self.m324[i].m0c.z = dvar9 * (self.m324[i].m00.z - self.m324[i].m0c.z);
            if i == HAND_LAST && self.m2ca != 0 {
                self.particles |= PARTICLE_BLOOD;
            }
        }
    }

    fn cut_control3(&mut self) {
        self.hand_sizes();
        add_calc2(&mut self.m2f0.y, self.m2cc.y, 1.0, LINE_SETTLE);
    }

    fn start_control1(&mut self, actor: &Actor) {
        self.m324[HEAD].m00 = actor.pos;
        let local_94 = Vec3::new(0.0, self.m314, self.m318);
        let local_ac = turned(actor.angle[1], local_94);
        add_calc2(&mut self.m314, REG0_F7, 1.0, START_Y_STEP);
        add_calc2(&mut self.m318, REG0_F8 + 5.0, 1.0, START_Z_STEP);
        let local_94 = Vec3::new(0.0, self.m314, self.m310);
        let dvar9 = self.m31c;
        for i in 1..HAND_LAST {
            let local_c4 = Vec3::new(
                dvar9
                    * walk::from_s16(sni(
                        i32::from(self.m2b8) * (REG0_S5 + 0x640)
                            + i as i32 * (REG0_S6 + 3000),
                    ))
                    .sin(),
                dvar9
                    * walk::from_s16(sni(
                        i32::from(self.m2b8) * (REG0_S5 + 0x6a4)
                            + i as i32 * (REG0_S6 + 4000),
                    ))
                    .sin(),
                dvar9
                    * walk::from_s16(sni(
                        i32::from(self.m2b8) * (REG0_S7 + 0x578)
                            + i as i32 * (REG0_S8 + 0xdac),
                    ))
                    .cos(),
            );
            let fvar1 = local_c4.x + (local_ac.x + (self.m324[i].m00.x - self.m324[i - 1].m00.x));
            let dvar10 = local_c4.y + (local_ac.y + (self.m324[i].m00.y - self.m324[i - 1].m00.y));
            let fvar2 = local_c4.z + (local_ac.z + (self.m324[i].m00.z - self.m324[i - 1].m00.z));
            let yaw = atan2s(fvar1, fvar2);
            let pitch = -atan2s(dvar10, (fvar1 * fvar1 + fvar2 * fvar2).sqrt());
            let point = mtx_position(yaw, pitch, local_94);
            self.m324[i].m00 = self.m324[i - 1].m00 + point;
        }
    }

    fn start_control2(&mut self) {
        self.m324[HAND_LAST].m00 = self.m2d8;
        for i in (1..=HAND_REV_LAST).rev() {
            let fvar1 = self.m324[i].m00.x - self.m324[i + 1].m00.x;
            let dvar9 = (self.m324[i].m00.y - self.m324[i + 1].m00.y) - START_BEND;
            let fvar2 = self.m324[i].m00.z - self.m324[i + 1].m00.z;
            let yaw = atan2s(fvar1, fvar2);
            let pitch = -atan2s(dvar9, (fvar1 * fvar1 + fvar2 * fvar2).sqrt());
            let point = mtx_position(yaw, pitch, Vec3::new(0.0, 0.0, self.m310));
            self.m324[i].m00 = self.m324[i + 1].m00 + point;
        }
        self.m2f0 = self.m324[HAND_LAST].m00;
        let local = self.m324[HAND_REV_LAST].m00 - self.m324[HAND_LAST].m00;
        self.m300 = -atan2s(local.y, local.z);
        self.m302 = atan2s(local.x, (local.y * local.y + local.z * local.z).sqrt());
    }

    fn ceiling(&mut self, solid: Option<&Collision>, hio: &BmdhandHio) {
        let from = self.m2d8;
        let to = self.m2d8 + Vec3::new(0.0, LINE_HEIGHT, 0.0);
        let cross = solid.map(|solid| solid.slide(from, to)).filter(|at| *at != to);
        if let Some(cross) = cross {
            self.m2cc = cross;
            self.m2cc.y += hio.m08;
        }
    }

    fn hand_calc(
        &mut self,
        actor: &Actor,
        param: u32,
        solid: Option<&Collision>,
        hio: &BmdhandHio,
        boss: &mut KalleDemos,
        sink: &mut Sink<'_>,
    ) {
        if self.m2b8 & 0xf == 0 {
            self.ceiling(solid, hio);
        }
        match self.m2bc {
            HAND_WAIT => {
                let local_c4 = turned(actor.angle[1], Vec3::new(0.0, 0.0, branch_z(param)));
                self.m2d8 = actor.pos + local_c4;
                self.m2d8.y = self.m2cc.y;
                self.m2e4 = self.m2d8;
                if self.m30c > 0.9 {
                    if boss.vine_count < BOSS_VINE_CAP {
                        boss.vine_count = boss.vine_count.wrapping_add(1);
                    }
                    self.m2bc = HAND_RISE;
                    self.m2c0[0] = (self.rnd_f(30.0) + 10.0) as i32 as i16;
                    sink.events.list.push(SimEvent::Se(SE_CM_BKM_VINE_SET));
                }
            }
            HAND_RISE => {
                let local_c4 = turned(actor.angle[1], Vec3::new(0.0, 0.0, branch_z(param)));
                self.m2e4.x = actor.pos.x + local_c4.x;
                self.m2e4.z = actor.pos.z + local_c4.z;
                let gap = self.m2e4 - self.m2d8;
                if gap.x.hypot(gap.z) > REG14_F0 + CX_SNAP_RANGE {
                    if self.m2c0[0] == 0 {
                        self.m2bc = HAND_SNAP;
                        self.m2c0[0] = CX_SNAP_JUMP;
                        self.m308 = 0.0;
                    }
                } else {
                    self.m2c0[0] = self.rnd_f(30.0) as i32 as i16;
                }
            }
            HAND_SNAP => {
                let (rate, target) = if self.m2c0[0] <= CX_SNAP_TIME {
                    if self.m2c0[0] == CX_SNAP_TIME {
                        self.m308 = 0.0;
                    }
                    (1.0, self.m2cc.y)
                } else {
                    (CX_RISE_RATE, self.m2cc.y - CX_RISE_STEP + REG0_FE)
                };
                self.m2e4.y = target;
                let step = self.m308 * CX_SNAP_RATE;
                add_calc2(&mut self.m2d8.x, self.m2e4.x, rate, step);
                add_calc2(&mut self.m2d8.z, self.m2e4.z, rate, step);
                add_calc2(&mut self.m2d8.y, self.m2e4.y, rate, step);
                add_calc2(&mut self.m308, 1.0, 1.0, 0.1);
                if self.m2c0[0] == 0 {
                    self.m2bc = HAND_RISE;
                    self.m2c0[0] = self.rnd_f(30.0) as i32 as i16;
                }
            }
            _ => {}
        }
    }

    fn start_hand_calc(
        &mut self,
        actor: &Actor,
        param: u32,
        solid: Option<&Collision>,
        hio: &BmdhandHio,
    ) {
        self.ceiling(solid, hio);
        match self.m2bc {
            HAND_WAIT => {
                let mut fvar1 = self.m2e4.y - self.m2d8.y;
                fvar1 *= REG13_F4 + START_SPIN;
                fvar1 *= REG13_F5 + 2.0;
                let local_a8 = Vec3::new(
                    fvar1 * walk::from_s16(sni(i32::from(self.m2b8) * (REG13_S5 + 0x5dc))).sin(),
                    fvar1 * walk::from_s16(sni(i32::from(self.m2b8) * (REG13_S7 + 500))).sin(),
                    branch_z(param),
                );
                let cstack_b4 = turned(actor.angle[1], local_a8);
                self.m2e4 = actor.pos + cstack_b4;
                self.m2e4.y = self.m2cc.y;
                let step = self.m308 * START_TRACK;
                add_calc2(&mut self.m2d8.x, self.m2e4.x, 1.0, step);
                add_calc2(&mut self.m2d8.z, self.m2e4.z, 1.0, step);
                add_calc2(&mut self.m2d8.y, self.m2e4.y, 1.0, self.m308 * START_RISE);
                add_calc2(&mut self.m308, REG13_F6 + 1.0, 1.0, REG13_F7 + START_RATE);
            }
            _ => {}
        }
    }

    fn hand_move(
        &mut self,
        actor: &mut Actor,
        param: u32,
        solid: Option<&Collision>,
        hio: &BmdhandHio,
        boss: &mut KalleDemos,
        blow: Blow,
        player: Vec3,
        sink: &mut Sink<'_>,
    ) {
        let r3 = sni(
            REG8_S4 + (param & PARAM_BRANCH) as i32 * PARAM_YAW_STEP + PARAM_YAW_BASE,
        );
        actor.angle[1] = r3.wrapping_add(boss.shape_angle[1]);
        let joint = boss_joint_matrix(boss, joint_slot(param));
        let local_40 = Vec3::new(
            REG14_F6,
            REG14_F7,
            BOSS_JOINT_XAD[(param & PARAM_XAD) as usize],
        );
        actor.pos = (joint * Vec4::new(local_40.x, local_40.y, local_40.z, 1.0)).truncate();
        if self.m2ba != STATE_ATTACH && boss.camera_type == BOSS_CAM_ATTACH {
            self.m2ba = STATE_ATTACH;
            self.m314 = ATTACH_STEP;
            self.m318 = ATTACH_SWING;
            self.m31c = self.rnd_f(ATTACH_SWAY_SPAN) + ATTACH_SWAY;
            self.m2c0[1] = (self.rnd_f(30.0) + ATTACH_HOLD) as i32 as i16;
        }
        match self.m2ba {
            STATE_WAIT => {
                add_calc2(
                    &mut self.m310,
                    REG14_FB + WAIT_TARGET,
                    WAIT_TARGET_RATE,
                    WAIT_TARGET_STEP,
                );
                add_calc2(
                    &mut self.m31c,
                    REG14_F11 + WAIT_SWAY + WAIT_SWAY_ADD,
                    WAIT_SWAY_RATE,
                    WAIT_SWAY_STEP,
                );
                add_calc2(&mut self.m320, 1.0, 1.0, SCALE_STEP);
                self.hand_calc(actor, param, solid, hio, boss, sink);
                self.control1(actor);
                self.control2();
                self.hand_sizes();
                self.attention_flags = ATTENTION_LOCKON_BATTLE;
                if self.m2bc > 0 && (self.sph_a.tg_hit || self.sph_b.tg_hit) {
                    self.m2ba = STATE_CUT;
                    sink.events.list.push(SimEvent::Se(SE_CV_BKM_CUT_VINE_1));
                    sink.events.list.push(SimEvent::Se(SE_CM_BKM_CUT_VINE));
                    self.particles |= PARTICLE_SIBOUBAKUEN;
                    self.particles |= PARTICLE_SIBOUFLASH;
                    if boss.vine_count > 0 {
                        boss.vine_count = boss.vine_count.wrapping_sub(1);
                        boss.cut_timer = BOSS_CUT_TIME;
                    }
                    self.m2c0[0] = hio.m0c;
                    self.m314 = ATTACH_STEP;
                    self.m318 = ATTACH_SWING;
                    self.m31c = self.rnd_f(ATTACH_SWAY_SPAN) + ATTACH_SWAY;
                    self.m2c8 = CUT_OFF;
                }
            }
            STATE_CUT => {
                self.m2c8 = CUT_OFF;
                self.m2ca = 2;
                add_calc2(
                    &mut self.m310,
                    REG14_FC + CUT_TARGET,
                    CUT_TARGET_RATE,
                    CUT_TARGET_STEP,
                );
                self.cut_control(actor, boss.ground);
                self.cut_control3();
                if self.m2c0[0] < CUT_RECOVER {
                    add_calc2(&mut self.m320, 0.0, 1.0, SCALE_STEP);
                }
                if self.m2c0[0] == 0 {
                    self.m2ba = STATE_WAIT;
                    self.m2bc = HAND_WAIT;
                    self.m30c = 0.0;
                    self.m31c = 0.0;
                }
            }
            STATE_ATTACH => {
                self.m2c8 = CUT_OFF;
                if self.m2c0[1] == 0 {
                    add_calc2(
                        &mut self.m310,
                        REG14_FC + CUT_TARGET,
                        CUT_TARGET_RATE,
                        CUT_TARGET_STEP,
                    );
                }
                if boss.state == BOSS_STATE_LANDING && boss.timers[0] < CUT_RECOVER {
                    add_calc0(&mut self.m320, 1.0, SCALE_STEP);
                }
                self.cut_control(actor, boss.ground);
                self.cut_control3();
                if boss.camera_type == BOSS_CAM_RESTORE {
                    self.m2c0[0] = self.rnd_f(50.0) as i32 as i16;
                    self.m2ba = STATE_RESTORE;
                    self.m2bc = HAND_WAIT;
                } else if boss.camera_type == BOSS_CAM_GIVE {
                    self.m2ba = STATE_STOW;
                }
            }
            STATE_ENTER => {
                match self.m2be {
                    ENTER_STANDBY => {
                        if boss.camera_type == BOSS_CAM_DROP {
                            self.m2be = ENTER_WAIT;
                            self.m2e4.y = START_TOP_Y;
                            self.m2d8 = actor.pos;
                            self.m30c = 1.0;
                            self.m2c0[0] = self.rnd_f(50.0) as i32 as i16;
                        }
                    }
                    ENTER_WAIT => {
                        if self.m2c0[0] == 0 {
                            self.m2be = ENTER_RISE;
                        }
                    }
                    ENTER_RISE => {
                        if self.m2e4.y - self.m2d8.y < START_NEAR {
                            boss.vine_count = boss.vine_count.wrapping_add(1);
                            self.m2be = ENTER_HANG;
                        }
                        self.enter_tail(actor, param, solid, hio);
                    }
                    ENTER_HANG => self.enter_tail(actor, param, solid, hio),
                    _ => {}
                }
                if boss.camera_type == BOSS_CAM_RESET {
                    self.m2ba = STATE_WAIT;
                    self.m2bc = HAND_WAIT;
                }
            }
            STATE_STOW => {
                self.m2c8 = CUT_OFF;
                add_calc0(&mut self.m310, STOW_SPIN, STOW_SPIN_STEP);
                add_calc0(&mut self.m320, 1.0, SCALE_STEP);
                self.cut_control(actor, boss.ground);
                self.cut_control3();
            }
            STATE_RESTORE => {
                self.m2c8 = sni(REG8_S2 + i32::from(RESTORE_TIME));
                if self.m2c0[0] == 0 {
                    self.m2ba = STATE_WAIT;
                    self.m2bc = HAND_WAIT;
                    self.m30c = self.rnd_f(RESTORE_SPIN);
                    self.m31c = 0.0;
                }
            }
            _ => {}
        }
        self.place(actor);
        self.sph_a.tg_hit = self.sph_a.touched(blow, player);
        self.sph_b.tg_hit = self.sph_b.touched(blow, player);
    }

    fn place(&mut self, actor: &mut Actor) {
        for index in 0..HANDS {
            if index == SPH_A_INDEX {
                actor.eye = self.m324[index].m00;
                self.attention = actor.eye;
                self.sph_a.centre = actor.eye;
            } else {
                let r0 = i32::from(self.m2b8 & 3) * 4 + 3;
                if r0 == index as i32 {
                    self.sph_b.centre = self.m324[index].m00;
                }
            }
        }
        if self.m2c8 != 0 {
            let off = Vec3::new(0.0, LINE_OFF_Y, 0.0);
            self.sph_a.centre = off;
            self.sph_b.centre = off;
            self.status &= !0;
            self.attention_flags = 0;
        }
    }

    fn enter_tail(
        &mut self,
        actor: &Actor,
        param: u32,
        solid: Option<&Collision>,
        hio: &BmdhandHio,
    ) {
        add_calc2(
            &mut self.m310,
            REG13_F1 + ENTER_SWING,
            ENTER_SWING_RATE,
            REG13_F2 + ENTER_SWING_STEP,
        );
        add_calc2(
            &mut self.m31c,
            REG13_F3 + ENTER_SWAY,
            ENTER_SWAY_RATE,
            ENTER_SWAY_STEP,
        );
        if self.m2e4.y - self.m2d8.y < START_FAR {
            add_calc2(&mut self.m320, 1.0, 1.0, ENTER_SCALE_STEP);
        }
        self.start_hand_calc(actor, param, solid, hio);
        self.start_control1(actor);
        self.start_control2();
        self.hand_sizes();
    }

    pub fn execute(
        &mut self,
        actor: &mut Actor,
        param: u32,
        solid: Option<&Collision>,
        hio: &BmdhandHio,
        boss: Option<&mut KalleDemos>,
        blow: Blow,
        player: Vec3,
        sink: &mut Sink<'_>,
    ) {
        self.particles = 0;
        self.m2b8 = self.m2b8.wrapping_add(1);
        for index in 0..TIMER_COUNT {
            if self.m2c0[index] != 0 {
                self.m2c0[index] = self.m2c0[index].wrapping_sub(1);
            }
        }
        if self.m2c8 != 0 {
            self.m2c8 = self.m2c8.wrapping_sub(1);
        }
        if self.m2ca != 0 {
            self.m2ca = self.m2ca.wrapping_sub(1);
        }
        self.m2fc = self.m2f0.y;
        match boss {
            Some(boss) => self.hand_move(actor, param, solid, hio, boss, blow, player, sink),
            None => self.place(actor),
        }
        let dvar4 = self.m2cc.y - 1.0;
        if self.m2f0.y < dvar4 && self.m2fc >= dvar4 {
            self.hand_open();
        }
        if self.m2f0.y > dvar4 && self.m2fc <= dvar4 {
            self.m304 = 3;
        }
        if self.m304 != 0 {
            self.m304 = self.m304.wrapping_sub(1);
            if self.m304 == 0 {
                self.hand_close();
            }
        }
        self.morph.play();
    }
}

struct Sink<'a> {
    events: &'a mut SimEvents,
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

fn sni(value: i32) -> i16 {
    value as i16
}

fn turned(yaw: i16, offset: Vec3) -> Vec3 {
    let (sin, cos) = walk::from_s16(yaw).sin_cos();
    Vec3::new(
        offset.x * cos + offset.z * sin,
        offset.y,
        offset.z * cos - offset.x * sin,
    )
}

fn tilted(pitch: i16, value: Vec3) -> Vec3 {
    let (sin, cos) = walk::from_s16(pitch).sin_cos();
    Vec3::new(
        value.x,
        value.y * cos - value.z * sin,
        value.y * sin + value.z * cos,
    )
}

fn mtx_position(yaw: i16, pitch: i16, value: Vec3) -> Vec3 {
    turned(yaw, tilted(pitch, value))
}

fn branch_z(param: u32) -> f32 {
    if (param & PARAM_ODD) == 0 {
        REG0_F9 + CX_ALT_A
    } else {
        REG0_F10 + CX_ALT_B
    }
}

fn joint_slot(param: u32) -> usize {
    let index = (param & PARAM_BRANCH) as usize;
    let joint = BOSS_JOINT_D.get(index).copied().unwrap_or(JNT_HANAA7);
    match joint {
        JNT_HANAB7 => 1,
        JNT_HANAC7 => 2,
        JNT_HANAE7 => 3,
        JNT_HANAD7 => 4,
        _ => 0,
    }
}

fn boss_joint_matrix(boss: &KalleDemos, slot: usize) -> Mat4 {
    let rows = JOINT_BIND[slot];
    let local = Mat4::from_cols_array(&[
        rows[0][0], rows[1][0], rows[2][0], 0.0, rows[0][1], rows[1][1], rows[2][1], 0.0,
        rows[0][2], rows[1][2], rows[2][2], 0.0, rows[0][3], rows[1][3], rows[2][3], 1.0,
    ]);
    let mut base = Mat4::from_translation(boss.pos);
    base *= Mat4::from_rotation_y(walk::from_s16(boss.shape_angle[1]));
    base *= Mat4::from_rotation_x(walk::from_s16(boss.shape_angle[0]));
    base *= Mat4::from_rotation_z(walk::from_s16(boss.shape_angle[2]));
    base * local
}

#[derive(SystemParam)]
pub struct Watch<'w, 's> {
    collision: Option<Res<'w, Collision>>,
    combat: Res<'w, Combat>,
    hammer: Res<'w, hammer::Cut>,
    blasts: Res<'w, super::bombflower::Blasts>,
    players: Query<
        'w,
        's,
        &'static Actor,
        (With<Player>, Without<Bmdhand>, Without<KalleDemos>),
    >,
    shafts: Query<'w, 's, &'static arrow::Arrow, Without<Bmdhand>>,
    bolts: Query<'w, 's, &'static boomerang::Boomerang, Without<Bmdhand>>,
}

fn blow_of(watch: &Watch<'_, '_>) -> Blow {
    if let Some(shaft) = watch.shafts.iter().next() {
        return match shaft.kind {
            arrow::TYPE_LIGHT => Blow::LightArrow,
            arrow::TYPE_FIRE => Blow::FireArrow,
            arrow::TYPE_ICE => Blow::IceArrow,
            _ => Blow::Arrow,
        };
    }
    if watch
        .bolts
        .iter()
        .any(|bolt| bolt.parameter == boomerang::MODE_THROW)
    {
        return Blow::Boomerang;
    }
    if watch.hammer.kind == hammer::CUT_HAMMER_SIDESWING
        || watch.hammer.kind == hammer::CUT_HAMMER_FRONTSWING
    {
        return Blow::Hammer;
    }
    if watch.combat.active && watch.combat.frame >= 1 && watch.combat.frame < 6 {
        return Blow::Sword;
    }
    if !watch.blasts.list.is_empty() {
        return Blow::Bomb;
    }
    Blow::Idle
}

pub fn control(
    watch: Watch<'_, '_>,
    mut events: ResMut<SimEvents>,
    mut hio: ResMut<BmdhandHio>,
    mut statics: ResMut<BmdhandStatics>,
    mem: Res<MemBits>,
    stage: Res<StageName>,
    mut hands: Query<
        (Entity, &mut Actor, &mut Bmdhand),
        (With<Bmdhand>, Without<KalleDemos>, Without<Player>),
    >,
    mut bosses: Query<
        (Entity, &mut KalleDemos),
        (With<KalleDemos>, Without<Bmdhand>, Without<Player>),
    >,
) {
    let solid = watch.collision.as_deref();
    let blow = blow_of(&watch);
    let player = watch
        .players
        .iter()
        .next()
        .map(|actor| actor.pos)
        .unwrap_or(Vec3::ZERO);
    let boss_demo = mem.is_stage_boss_demo();
    let start_stage_first = stage.0.as_bytes().first().copied().unwrap_or(0);
    let hio_now = *hio;
    let found = bosses.iter_mut().next();
    if statics.boss.is_none() {
        statics.boss = found.as_ref().map(|(entity, _)| *entity);
    }
    let mut boss = found.map(|(_, boss)| boss);
    let mut sink = Sink {
        events: &mut events,
    };
    for (_, mut actor, mut hand) in &mut hands {
        if !hand.entered {
            hand.entered = true;
            hand.create(&mut hio, &mut statics, boss_demo, start_stage_first);
        }
        let param = actor.parameters;
        hand.execute(
            &mut actor,
            param,
            solid,
            &hio_now,
            boss.as_deref_mut(),
            blow,
            player,
            &mut sink,
        );
    }
}