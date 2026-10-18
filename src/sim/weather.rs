use bevy_ecs::prelude::*;
use glam::{Mat4, Vec3};

use super::area::StageName;
use super::camera::{Camera, ViewPort};
use super::collision::Collision;
use super::daynight::DayNight;
use super::frame::CurrentFrame;
use super::kytag00::EnvLight;
use super::player::Player;

/// `G_CM3D_F_INF` (`SSystem/SComponent/c_m3d.h:25`), the ceiling the ground and roof checks report
/// when nothing is under a point.
pub const F_INF: f32 = 1000000000.0;
/// `S2DEG_CONSTANT` (`SSystem/SComponent/c_angle.h:9`), the fixed-point circle to degrees.
pub const S2DEG: f32 = 180.0 / 32768.0;
/// `DEG2RAD_CONSTANT` (`SSystem/SComponent/c_angle.h:7`).
pub const DEG2RAD: f32 = core::f32::consts::PI / 180.0;
/// `walk::RAD_PER_UNIT` (`src/sim/walk.rs:47`): the fixed-point circle to radians.
const RAD_PER_UNIT: f32 = core::f32::consts::PI / 32768.0;

/// `SQUARE` (`global.h:31`).
pub const fn square(value: f32) -> f32 {
    value * value
}

/// `dKankyo_rain_Packet::mEff` (`d_kankyo_wether.h:78`), `mSnowPacket::mEff` (`:119`),
/// `mEffect` (`:240`), `mEff` (`:211`, `:272`) and `mInst` (`:272`).
pub const RAIN_MAX: usize = 250;
pub const SNOW_MAX: usize = 250;
pub const POISON_MAX: usize = 1000;
pub const CLOUD_MAX: usize = 100;
pub const HOUSI_MAX: usize = 300;
pub const VRKUMO_MAX: usize = 100;
pub const WAVE_MAX: usize = 300;
/// `WINDEFF_SET::mWindEff` (`d_kankyo_wether.h:339`).
pub const WINDLINE_MAX: usize = 30;
/// `WINDEFF_SET::mKamomeEff` (`d_kankyo_wether.h:340`).
pub const KAMOME_MAX: usize = 10;
/// `dScnKy_env_light_c::mWindInfluenceEntity` (`d_kankyo.h:211`) and `mpWaveInfl` (`d_kankyo.h:250`).
pub const SQUALL_MAX: usize = 5;
pub const WAVE_INFL_MAX: usize = 10;
/// `dScnKy_env_light_c::mpWindInfluence` (`d_kankyo.h:790`).
pub const POINT_WIND_MAX: usize = 30;
/// `dKankyo_sun_Packet::mVizChkData` (`d_kankyo_wether.h:28`) and the five `sun_chkpnt` entries
/// the loop at `d_kankyo_rain.cpp:604-610` walks.
pub const SUN_CHECKS: usize = 5;
/// `dKankyo_sunlenz_Packet::mPositions` (`d_kankyo_wether.h:20`).
pub const LENZ_POSITIONS: usize = 8;
/// `dStageType_MISC_e` (`d_stage.h:19`), the stage type that turns rain, sun, lens flare, the global
/// wind and the sea-surface skew over to their own fixed behaviour.
pub const STAGE_TYPE_MISC: u32 = 2;
/// `dKyr_housi_move` and `dKyr_snow_move` only ever touch the first two birds and the first count
/// hundred and fifty streaks.
pub const KAMOME_ACTIVE: usize = 2;

/// `sun_chkpnt` (`d_kankyo_rain.cpp:604-610`), the five screen offsets the sun's depth probe is
/// taken at around its projected centre.
pub const SUN_CHKPNT: [(f32, f32); SUN_CHECKS] = [
    (0.0, 0.0),
    (-10.0, -20.0),
    (10.0, 20.0),
    (-20.0, 10.0),
    (20.0, -10.0),
];

/// `room_pat_tbl` (`d_kankyo_rain.cpp:2330-2347`), indexed by `dComIfGp_roomControl_getStayNo()`
/// when that number is below sixteen. A negative entry means the room has no poison pattern.
pub const POISON_ROOM_PATTERN: [i32; 16] =
    [-1, -1, -1, 0, -1, -1, -1, -1, -1, 4, -1, 3, -1, 1, -1, 2];

/// `wind_table` (`d_kankyo_wether.cpp:1375-1379`), the eight direction buckets
/// `dKyw_get_windsdir` walks the compass angle of the wind through.
pub const WIND_DIR_TABLE: [(u16, u16, i32); 9] = [
    (0x0000, 0x1000, 0),
    (0x1000, 0x3000, 1),
    (0x3000, 0x5000, 2),
    (0x5000, 0x7000, 3),
    (0x7000, 0x9000, 4),
    (0x9000, 0xB000, 5),
    (0xB000, 0xD000, 6),
    (0xD000, 0xF000, 7),
    (0xF000, 0xFFFF, 0),
];

const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];

/// The reference's three-lane `cM_rnd` generator, as the port's other modules carry it.
fn rnd(seed: &mut [i32; 3]) -> f32 {
    for lane in 0..3 {
        seed[lane] = (seed[lane] * RND_MUL[lane]) % RND_MOD[lane];
    }
    let sum = (seed[0]) as f32 / RND_SCALE[0]
        + (seed[1]) as f32 / RND_SCALE[1]
        + (seed[2]) as f32 / RND_SCALE[2];
    (sum % 1.0).abs()
}

/// `cM_rndF`.
fn rnd_f(seed: &mut [i32; 3], range: f32) -> f32 {
    rnd(seed) * range
}

/// `cM_rndFX`.
fn rnd_fx(seed: &mut [i32; 3], range: f32) -> f32 {
    range * (rnd(seed) - 0.5) * 2.0
}

/// `cM_rndFX(0x10000)` written into the `u16` `field_0x4c`, which is the draw the reference casts
/// rather than stores, so the value wraps the way its own cast does.
fn rnd_fx_u16(seed: &mut [i32; 3], range: f32) -> u16 {
    rnd_fx(seed, range) as i16 as u16
}

/// `cM_rndFX(0xFFFF)`: the signed fixed-point angle the two circling birds are placed on.
fn rnd_angle(seed: &mut [i32; 3]) -> i16 {
    rnd_fx(seed, 65535.0) as i16
}

/// `cM_rndF(0xFFFF)` into the `u16` the dome cloud's bearing is drawn through.
fn rnd_f_u16(seed: &mut [i32; 3], range: f32) -> u16 {
    (rnd(seed) * range) as u16
}

/// `cM_ssin`: the fixed-point sine.
fn ssin(angle: i16) -> f32 {
    (f32::from(angle) * RAD_PER_UNIT).sin()
}

/// `cM_scos`: the fixed-point cosine.
fn scos(angle: i16) -> f32 {
    (f32::from(angle) * RAD_PER_UNIT).cos()
}

/// `cM_atan2s` (`f_op/f_op_actor.cpp`): radians scaled into the `0..0xFFFF` circle.
fn atan2s(y: f32, x: f32) -> i16 {
    (y.atan2(x) * (32768.0 / core::f32::consts::PI)).round() as i16
}

/// `cM_fsin`: the radian sine the snow, spray and star phases run on.
fn fsin(angle: f32) -> f32 {
    angle.sin()
}

/// `cM_fcos`.
fn fcos(angle: f32) -> f32 {
    angle.cos()
}

/// `cLib_addCalc` (`SSystem/SComponent/c_lib.h:15`).
fn add_calc(value: &mut f32, target: f32, scale: f32, max_step: f32, min_step: f32) {
    if *value == target {
        return;
    }
    let step = scale * (target - *value);
    if step >= min_step || step <= -min_step {
        *value += step.clamp(-max_step, max_step);
    } else if step > 0.0 {
        *value += min_step;
        if *value > target {
            *value = target;
        }
    } else {
        *value -= min_step;
        if *value < target {
            *value = target;
        }
    }
}

/// `cLib_addCalcAngleS` (`SSystem/SComponent/c_lib.h:22`) writing through its `s16*`.
fn add_calc_angle_s(value: &mut i16, target: i16, scale: i16, max_step: i16, min_step: i16) {
    if *value == target {
        return;
    }
    let diff = target.wrapping_sub(*value);
    let mut step = diff / scale;
    if step > min_step || step < -min_step {
        step = step.clamp(-max_step, max_step);
        *value = value.wrapping_add(step);
    } else if diff >= 0 {
        *value = value.wrapping_add(min_step);
        if target.wrapping_sub(*value) <= 0 {
            *value = target;
        }
    } else {
        *value = value.wrapping_sub(min_step);
        if target.wrapping_sub(*value) >= 0 {
            *value = target;
        }
    }
}

/// `cLib_chaseF` (`SSystem/SComponent/c_lib.h:28`).
fn chase_f(value: &mut f32, target: f32, step: f32) -> bool {
    if step == 0.0 {
        let done = *value == target;
        if done {
            *value = target;
        }
        return done;
    }
    let step = if *value > target { -step } else { step };
    *value += step;
    if step * (*value - target) >= 0.0 {
        *value = target;
        true
    } else {
        false
    }
}

/// `vectle_calc` (`d_kankyo_rain.cpp:26-38`): the length is taken in double precision and a zero
/// vector normalises to zero rather than to a NaN.
fn vectle_calc(value: Vec3) -> Vec3 {
    let (x, y, z) = (f64::from(value.x), f64::from(value.y), f64::from(value.z));
    let length = (x * x + y * y + z * z).sqrt();
    if length != 0.0 {
        Vec3::new(
            (x / length) as f32,
            (y / length) as f32,
            (z / length) as f32,
        )
    } else {
        Vec3::ZERO
    }
}

/// `get_vectle_calc` / `dKyr_get_vectle_calc` (`d_kankyo_rain.cpp:41-53`): the unit vector from
/// `from` to `to`.
pub fn vectle(from: Vec3, to: Vec3) -> Vec3 {
    vectle_calc(to - from)
}

/// `dKy_set_eyevect_calc` (`d_kankyo_rain.cpp:56-62`): a point along the camera axis with the
/// height biased two hundred units down, which is what puts the lens flare below the horizon.
fn set_eyevect_calc(eye: Vec3, center: Vec3, along: f32, up: f32) -> Vec3 {
    let direction = vectle(eye, center);
    Vec3::new(
        eye.x + direction.x * along,
        (eye.y + direction.y * up) - 200.0,
        eye.z + direction.z * along,
    )
}

/// `dKy_set_eyevect_calc2` (`d_kankyo_rain.cpp:65-86`): the same point without the height bias and
/// with the vertical component suppressed when `up` is zero.
fn set_eyevect_calc2(eye: Vec3, center: Vec3, along: f32, up: f32) -> Vec3 {
    let mut delta = center - eye;
    if up == 0.0 {
        delta.y = 0.0;
    }
    let direction = vectle_calc(delta);
    let out = Vec3::new(
        eye.x + direction.x * along,
        eye.y + direction.y * up,
        eye.z + direction.z * along,
    );
    if up == 0.0 {
        Vec3::new(out.x, 0.0, out.z)
    } else {
        out
    }
}

/// `cM3d_VectorProduct2d` (`SSystem/SComponent/c_m3d.h:150`): the signed planar cross product of the
/// segment from the first point to the second against the segment from the first point to the
/// third. The weather always passes the origin as the first point, so what it computes is the cross
/// product of the two unit vectors that follow.
fn vector_product_2d(x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32) -> f32 {
    (x2 - x1) * (y3 - y1) - (y2 - y1) * (x3 - x1)
}

/// `GXColorS10`, the four-channel colour every palette write in the environment light lands in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ColorS10 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl ColorS10 {
    /// `GXColorS10`'s three-channel constructor, which leaves the alpha opaque.
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 0xFF }
    }
}

/// `LIGHT_INFLUENCE` (`d_kankyo.h:18-24`): the point, colour, power and flicker rate a registered
/// light carries. The thunder flash is the one influence the weather writes itself.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct LightInfluence {
    pub pos: Vec3,
    pub color: ColorS10,
    pub power: f32,
    pub fluctuation: f32,
    pub idx: i32,
}

/// The eight additive colours `dKyr_thunder_move` drives through the `dKy_*_addcol_*_set` family
/// (`d_kankyo.cpp:5617-5632`): the actor, background and second-background weights against the two
/// colours the flash uses, the near strike driving all eight and the far strike only three.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlashColor {
    /// The colour of the actor, second-background, sky, mist and fog terms.
    pub main: (u8, u8, u8),
    /// The colour of the two background terms, which the near strike tints differently.
    pub background: (u8, u8, u8),
    pub actor: f32,
    pub bg: f32,
    pub bg1: f32,
    pub sky_0: f32,
    pub kasumi: f32,
    pub fog: f32,
}

impl Default for FlashColor {
    fn default() -> Self {
        Self {
            main: (0x5a, 0xa0, 0xf5),
            background: (0x32, 0x78, 0xff),
            actor: 0.0,
            bg: 0.0,
            bg1: 0.0,
            sky_0: 0.0,
            kasumi: 0.0,
            fog: 0.0,
        }
    }
}

/// `RAIN_EFF` (`d_kankyo_wether.h:53-67`), one of the two hundred fifty falling drops.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct RainEff {
    pub status: i8,
    pub pos: Vec3,
    pub base_pos: Vec3,
    /// `field_0x1c` through `field_0x30` (`0x1C`-`0x30`): the fall rate, its two eases, the
    /// per-drop alpha and the height at which the drop dies.
    pub m1c: f32,
    pub m20: f32,
    pub m24: f32,
    pub alpha: f32,
    pub m2c: f32,
    pub m30: f32,
    /// `mTimer` (`0x34`), the frames a drop is hidden after it is respawned.
    pub timer: i16,
}

/// `dKankyo_rain_Packet` (`d_kankyo_wether.h:69-93`), narrowed to what the simulation reads and
/// writes. The two texture pointers at `0x0010` and `0x0014` belong to the blit.
#[derive(Debug, Clone, PartialEq)]
pub struct RainPacket {
    pub eff: [RainEff; RAIN_MAX],
    pub m36c8: f32,
    pub m36cc: f32,
    pub rain_count: i16,
    pub cam_eye_pos: Vec3,
    pub cam_center_pos: Vec3,
    pub center_delta: Vec3,
    /// `mCenterDeltaMul` (`0x36FC`), the one frame's worth of camera motion the drops are pushed by.
    pub center_delta_mul: f32,
    pub m3700: f32,
    /// `mSibukiAlpha` (`0x3704`), the splash layer's alpha.
    pub sibuki_alpha: f32,
    /// `mOverheadFade` (`0x3708`) and the two forward probes `mFwdFade1` (`0x370C`) and
    /// `mFwdFade2` (`0x3710`): what is left of a drop's alpha once the roof over it and the two
    /// points along the camera axis are taken into account.
    pub overhead_fade: f32,
    pub fwd_fade1: f32,
    pub fwd_fade2: f32,
    /// `mStatus` (`0x3714`): bit zero is "under a roof", bit one "under a roof further along the
    /// camera axis".
    pub status: u8,
}

impl Default for RainPacket {
    fn default() -> Self {
        Self {
            eff: core::array::from_fn(|_| RainEff::default()),
            m36c8: 0.0,
            m36cc: 0.0,
            rain_count: 0,
            cam_eye_pos: Vec3::ZERO,
            cam_center_pos: Vec3::ZERO,
            center_delta: Vec3::ZERO,
            center_delta_mul: 0.0,
            m3700: 0.0,
            sibuki_alpha: 0.0,
            overhead_fade: 0.0,
            fwd_fade1: 0.0,
            fwd_fade2: 0.0,
            status: 0,
        }
    }
}

/// `SNOW_EFF` (`d_kankyo_wether.h:95-109`). One record drives falling snow, the snow that piles
/// on the ground and the banked layer, because all three write the same fields.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SnowEff {
    pub status: i8,
    pub pos: Vec3,
    pub base_pos: Vec3,
    pub pos_wave_x: f32,
    pub pos_wave_z: f32,
    pub gravity: f32,
    pub wind_speed: f32,
    pub scale: f32,
    pub timer: i16,
}

/// `dKankyo_snow_Packet` (`d_kankyo_wether.h:111-126`).
#[derive(Debug, Clone, PartialEq)]
pub struct SnowPacket {
    pub eff: [SnowEff; SNOW_MAX],
    pub old_eye_pos: Vec3,
    /// `field_0x36d0` and `field_0x36d4` (`0x36D0`, `0x36D4`): the along-wind and cross-wind factors
    /// the drift is scaled by, both read out of the camera heading and the wind vector.
    pub m36d0: f32,
    pub m36d4: f32,
    pub eff_count: i16,
    /// `mTamariStart` (`0x36DA`), the first index of the banked layer.
    pub tamari_start: i16,
}

impl Default for SnowPacket {
    fn default() -> Self {
        Self {
            eff: core::array::from_fn(|_| SnowEff::default()),
            old_eye_pos: Vec3::ZERO,
            m36d0: 0.0,
            m36d4: 0.0,
            eff_count: 0,
            tamari_start: 0,
        }
    }
}

/// `STAR_EFF` (`d_kankyo_wether.h:128-137`). Only the twinkle phase and its sine are named in the
/// layout; the blit reads the rest.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct StarEff {
    pub anim_counter: f32,
    pub sin: f32,
}

/// `dKankyo_star_Packet` (`d_kankyo_wether.h:139-149`).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct StarPacket {
    pub effect: [StarEff; 1],
    pub effect_num: i16,
}

/// `POISON_EFF` (`d_kankyo_wether.h:151-166`), one of the thousand smoke emitters.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PoisonEff {
    pub status: i8,
    pub pos: Vec3,
    /// `field_0x10` through `field_0x20` (`0x10`-`0x20`): the three gust-accumulated offsets that
    /// ease back to zero, the size phase and the per-emitter drift draw.
    pub m10: f32,
    pub m14: f32,
    pub m18: f32,
    pub m1c: f32,
    pub m20: f32,
    pub alpha: f32,
    pub size: f32,
    /// `field_0x2c` (`0x2C`) is the frames the emitter spends rising before it becomes a grown puff,
    /// and `field_0x2e` (`0x2E`) the frames it is held out for after touching a wall or a light.
    pub m2c: i16,
    pub m2e: i16,
}

/// `dKankyo_poison_Packet` (`d_kankyo_wether.h:170-182`).
#[derive(Debug, Clone, PartialEq)]
pub struct PoisonPacket {
    pub eff: [PoisonEff; POISON_MAX],
    pub base_pos: Vec3,
    /// `field_0xbb9c` (`0xBB9C`), the emitter nearest the camera, kept for the light collision pass.
    pub mbb9c: Vec3,
    pub count: i32,
}

impl Default for PoisonPacket {
    fn default() -> Self {
        Self {
            eff: core::array::from_fn(|_| PoisonEff::default()),
            base_pos: Vec3::ZERO,
            mbb9c: Vec3::ZERO,
            count: 0,
        }
    }
}

/// One row of the poison placement table (`d_kankyo_rain.cpp:2230-2319`), read out of the anonymous
/// struct by offset: the centre, the half-extent vector, the vertical rise, the rise speed, the
/// emitter count, the drift yaw and pitch, and the drift strength.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PoisonPattern {
    pub pos: Vec3,
    pub xc: Vec3,
    pub rise: f32,
    pub rise_speed: f32,
    pub count: i32,
    pub yaw: i16,
    pub pitch: i16,
    pub drift: f32,
}

/// `unk_poison_data` (`d_kankyo_rain.cpp:2243-2319`).
pub const POISON_PATTERNS: [PoisonPattern; 5] = [
    PoisonPattern {
        pos: Vec3::new(1832.0, -120.0, 0.0),
        xc: Vec3::new(1020.0, 50.0, 1500.0),
        rise: 90.0,
        rise_speed: 0.2,
        count: 1000,
        yaw: 0,
        pitch: -16384,
        drift: 0.3,
    },
    PoisonPattern {
        pos: Vec3::new(7600.0, -1500.0, 7660.0),
        xc: Vec3::new(470.0, 40.0, 1080.0),
        rise: 60.0,
        rise_speed: 0.2,
        count: 1000,
        yaw: 0,
        pitch: 32767,
        drift: 0.3,
    },
    PoisonPattern {
        pos: Vec3::new(-3333.0, -5900.0, 9498.0),
        xc: Vec3::new(1700.0, 400.0, 1500.0),
        rise: 170.0,
        rise_speed: 0.05,
        count: 1000,
        yaw: 16384,
        pitch: 0,
        drift: 0.0,
    },
    PoisonPattern {
        pos: Vec3::new(1910.0, -1860.0, 3680.0),
        xc: Vec3::new(1450.0, 100.0, 1700.0),
        rise: 150.0,
        rise_speed: 0.25,
        count: 1000,
        yaw: 0,
        pitch: 0,
        drift: 0.0,
    },
    PoisonPattern {
        pos: Vec3::new(5900.0, -2600.0, 2780.0),
        xc: Vec3::new(2200.0, 500.0, 1250.0),
        rise: 120.0,
        rise_speed: 0.3,
        count: 1000,
        yaw: 0,
        pitch: -16386,
        drift: 0.0,
    },
];

/// `CLOUD_EFF` (`d_kankyo_wether.h:184-199`), one of the hundred ground-shadow cloud billboards.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct CloudEff {
    pub status: i8,
    pub pos: Vec3,
    pub base_pos: Vec3,
    /// `mPntWindVel` (`0x1C`): the gust velocity the shadow is carried by, and `mVelRndm` (`0x28`) the
    /// three per-shadow drift draws.
    pub pnt_wind_vel: Vec3,
    pub vel_rndm: Vec3,
    /// `field_0x34` (`0x34`): the shadow's own drift rate, chased toward `mWindSpeed` (`0x38`).
    pub m34: f32,
    pub wind_speed: f32,
    /// `mAnimCounter` (`0x3C`) is the breathing phase, `mAlpha` (`0x40`) the shadow's opacity, and
    /// `mSize` (`0x44`) with `mInitialSize` (`0x48`) the two sizes the draw scales between.
    pub anim_counter: u16,
    pub alpha: f32,
    pub size: f32,
    pub initial_size: f32,
}

/// `dKankyo_cloud_Packet` (`d_kankyo_wether.h:201-211`).
#[derive(Debug, Clone, PartialEq)]
pub struct CloudPacket {
    pub eff: [CloudEff; CLOUD_MAX],
    pub count: i32,
}

impl Default for CloudPacket {
    fn default() -> Self {
        Self {
            eff: core::array::from_fn(|_| CloudEff::default()),
            count: 0,
        }
    }
}

/// `HOUSI_EFF` (`d_kankyo_wether.h:213-230`), one of the three hundred steam puffs.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct HousiEff {
    pub status: i8,
    pub pos: Vec3,
    pub base_pos: Vec3,
    /// `mSpeed` (`0x1C`): the gust velocity the puff is carried by, and `mScale` (`0x28`) the three
    /// phase counters the sway, rise and drift sines run on.
    pub speed: Vec3,
    pub scale: Vec3,
    /// `field_0x34` (`0x34`) is the rise rate the puff is spawned with, `field_0x38` (`0x38`) unused
    /// by the simulation, and `field_0x3c` (`0x3C`) the frames a puff is hidden after it is respawned.
    pub m34: f32,
    pub m38: f32,
    pub m3c: u16,
    pub alpha: f32,
    /// `field_0x44` (`0x44`) and `field_0x48` (`0x48`) are the puff's two distance terms, and
    /// `field_0x4c` (`0x4C`) the lifetime counter that fades the puff out at sixty-five thousand.
    pub m44: f32,
    pub m48: f32,
    pub m4c: u16,
}

/// `dKankyo_housi_Packet` (`d_kankyo_wether.h:232-245`).
#[derive(Debug, Clone, PartialEq)]
pub struct HousiPacket {
    pub effect: [HousiEff; HOUSI_MAX],
    /// `field_0x5ddc` (`0x5DDC`): the whole layer's alpha, eased toward one while a weather tag asks
    /// for steam and toward zero when it stops. [`HousiPacket::count`] is `mCount` (`0x5DE0`).
    pub m5ddc: f32,
    pub count: i16,
}

impl Default for HousiPacket {
    fn default() -> Self {
        Self {
            effect: core::array::from_fn(|_| HousiEff::default()),
            m5ddc: 0.0,
            count: 0,
        }
    }
}

/// `VRKUMO_EFF` (`d_kankyo_wether.h:247-259`), one of the hundred dome-cloud instances.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct VrkumoEff {
    pub status: i8,
    pub position: Vec3,
    /// `mHeight` (`0x1C`) is the per-instance height draw the dome radius is added to, and
    /// `mAlpha` (`0x20`) the instance's opacity.
    pub height: f32,
    pub alpha: f32,
    /// `mDistFalloff` (`0x24`) is the instance's share of the ring's own falloff, and `mSpeed`
    /// (`0x28`) the fraction of the ring speed this instance drifts at.
    pub dist_falloff: f32,
    pub speed: f32,
}

/// `dKankyo_vrkumo_Packet` (`d_kankyo_wether.h:261-273`).
#[derive(Debug, Clone, PartialEq)]
pub struct VrkumoPacket {
    pub inst: [VrkumoEff; VRKUMO_MAX],
}

impl Default for VrkumoPacket {
    fn default() -> Self {
        Self {
            inst: core::array::from_fn(|_| VrkumoEff::default()),
        }
    }
}

/// `WAVE_EFF` (`d_kankyo_wether.h:275-290`), one of the three hundred sea-surface wave caps.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct WaveEff {
    pub pos: Vec3,
    pub base_pos: Vec3,
    /// `mSpeed` (`0x18`) is the crest's drift rate, `mScale` (`0x1C`) its drawn size, and
    /// `mCounterSpeed` (`0x20`) with `mCounter` (`0x24`) the phase and its rate.
    pub speed: f32,
    pub scale: f32,
    pub counter_speed: f32,
    pub counter: f32,
    /// `mAlpha` (`0x28`) is the crest's opacity, faded by distance from the camera, and
    /// `mStrengthEnv` (`0x2C`) how much of its drift the registered gusts at its position leave it.
    pub alpha: f32,
    pub strength_env: f32,
    /// `field_0x30` (`0x30`) is unused by the simulation and `field_0x32` (`0x32`) the per-crest
    /// draw, both read only by the blit.
    pub m30: i16,
    pub m32: i16,
    pub status: i8,
}

/// `dKankyo_wave_Packet` (`d_kankyo_wether.h:292-304`).
#[derive(Debug, Clone, PartialEq)]
pub struct WavePacket {
    pub eff: [WaveEff; WAVE_MAX],
    /// `mSkewWidth` (`0x41B8`) and `mSkewDir` (`0x41BC`): how hard and in which direction the wind
    /// skews the sea surface.
    pub skew_width: f32,
    pub skew_dir: f32,
}

impl Default for WavePacket {
    fn default() -> Self {
        Self {
            eff: core::array::from_fn(|_| WaveEff::default()),
            skew_width: 0.0,
            skew_dir: 0.0,
        }
    }
}

/// `WAVE_INFO` (`d_kankyo.h:102-107`): a sea-surface gust's centre and the two radii it fades
/// between. Ten of these are registered at `dScnKy_env_light_c::mpWaveInfl`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct WaveInfo {
    pub pos: Vec3,
    pub outer_radius: f32,
    pub inner_radius: f32,
    /// `field_0x14` (`0x14`), read by the blit.
    pub m14: f32,
}

/// `dKankyo_sun_Packet` (`d_kankyo_wether.h:13-29`). `mPos[0]` is the body's position eight
/// thousand units along the light direction; the blit reads `mPos[1]` as well.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SunPacket {
    pub pos: [Vec3; 2],
    /// `mVizChkData` (`0x28`), the five depth reads. Written only by the reference's own peek.
    pub viz_chk_data: [u32; SUN_CHECKS],
    /// `field_0x3c` (`0x3C`): the counter the visible arm pushes up by two and the frame's own
    /// decrement pulls back down, and `field_0x3d` (`0x3D`) the flag beside it, together meaning the
    /// sun was up and unoccluded for `field_0x3c` frames' worth of easing.
    pub m3c: u8,
    pub m3d: bool,
    pub visibility: f32,
    pub sun_alpha: f32,
    pub moon_alpha: f32,
    pub color: ColorS10,
}

/// `dKankyo_sunlenz_Packet` (`d_kankyo_wether.h:31-51`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SunLenzPacket {
    /// `mPositions` (`0x20`), the eight points the flare is strung along.
    pub positions: [Vec3; LENZ_POSITIONS],
    /// `field_0x80` through `field_0x8c` (`0x80`-`0x8C`) are the four last-frame depth distances the
    /// sun's blend reads, each carried forward one slot each frame it is up.
    pub m80: f32,
    pub m84: f32,
    pub m88: f32,
    pub m8c: f32,
    /// `mAngleDeg` (`0x90`) and `mDistFalloff` (`0x94`), both read from the sun's projected screen
    /// position.
    pub angle_deg: f32,
    pub dist_falloff: f32,
    pub m98: i16,
    /// `mbDrawLenzInSky` (`0x9A`): the flare belongs to the sky rather than the lens when the sun's
    /// own depth probe is occluded at fewer than two of its five sample points.
    pub draw_lenz_in_sky: bool,
}

/// `KAMOME_EFF` (`d_kankyo_wether.h:306-318`), one of the ten circling birds.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct KamomeEff {
    /// `mPos` (`0x04`) is where the bird is placed, `mAngleY` (`0x10`) and `mAngleX` (`0x12`) the two
    /// circles it flies, and `mAngleYSpeed` (`0x14`) how fast the first one turns.
    pub pos: Vec3,
    pub angle_y: i16,
    pub angle_x: i16,
    pub angle_y_speed: f32,
    /// `mScale` (`0x18`) is the bird's drawn size, `mTimer` (`0x1C`) the frames before it is retired,
    /// and `mStatus` (`0x1E`) which of the three stages it is in.
    pub scale: f32,
    pub timer: i16,
    pub status: u8,
}

/// `WIND_EFF` (`d_kankyo_wether.h:320-335`), one of the thirty wind streaks. Every field the
/// emitter is placed from lives here; the `0x00` emitter pointer belongs to the particle system.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct WindEff {
    pub base_pos: Vec3,
    pub pos: Vec3,
    /// `mStateTimer` (`0x1C`) is the streak's life phase and `mAlpha` (`0x20`) its own opacity.
    pub state_timer: f32,
    pub alpha: f32,
    /// `mState` (`0x24`) is which of the five stages the streak is in, and `field_0x28` (`0x28`) the
    /// loop-the-loop counter that turns the streak back on itself.
    pub state: i32,
    pub m28: i32,
    /// `field_0x2c` (`0x2C`) is the sway phase, `mAngleY` (`0x2E`) the streak's pitch and
    /// `mAngleXZ` (`0x30`) its bearing.
    pub m2c: i16,
    pub angle_y: i16,
    pub angle_xz: i16,
    /// `field_0x32` (`0x32`) says the streak is allowed to loop.
    pub m32: u8,
    /// The alpha `dKyr_wind_move` hands the streak's emitter
    /// (`d_kankyo_rain.cpp:456-461`), which is `mAlpha` scaled by the background key colour and the
    /// distance fade. The reference keeps it in the emitter rather than in this record, and a port
    /// with no particle emitters is the only place left it can live.
    pub emitter_alpha: f32,
}

/// `WINDEFF_SET` (`d_kankyo_wether.h:338-343`).
#[derive(Debug, Clone, PartialEq)]
pub struct WindEffSet {
    pub wind_eff: [WindEff; WINDLINE_MAX],
    pub kamome_eff: [KamomeEff; KAMOME_MAX],
    /// `mbHasCustomWindPower` (`0x758`): whether the streak layer is running on a weather tag's own
    /// wind power rather than the room's. The transition is what resets every streak's state.
    pub has_custom_wind_power: u8,
    /// `field_0x759` (`0x759`), cleared by `dKyr_wind_init` and read by nothing.
    pub m759: u8,
}

impl Default for WindEffSet {
    fn default() -> Self {
        Self {
            wind_eff: core::array::from_fn(|_| WindEff::default()),
            kamome_eff: core::array::from_fn(|_| KamomeEff::default()),
            has_custom_wind_power: 0,
            m759: 0,
        }
    }
}

/// `WIND_INFLUENCE` (`d_kankyo.h:33-41`): one moving gust's volume, its direction and its fade.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct WindInfluence {
    pub pos: Vec3,
    /// `mDir` (`0x0C`) is the gust's direction, `mRadius` (`0x18`) the radius it fades over, and
    /// `mStrength` (`0x1C`) its peak power there.
    pub dir: Vec3,
    pub radius: f32,
    pub strength: f32,
    /// `field_0x20` (`0x20`) is the radius as it was registered.
    pub m20: f32,
    /// `mRegistIdx` (`0x24`) is the slot in `mpWindInfluence` this gust answers to, or `9999` when it
    /// could not be registered.
    pub regist_idx: i32,
    /// `mbConstant` (`0x28`): a constant gust blows along `mDir` from its centre, a growing one blows
    /// outward from the point on its leading edge that the query is inside of.
    pub constant: u8,
}

/// `WIND_INF_ENTITY` (`d_kankyo.h:43-49`): the owning record of a gust, with the three rates
/// `squal_proc` steers it by.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct WindEntity {
    /// `mbInUse` (`0x00`) is the slot's claim, `mMinRadius` (`0x04`) the radius the gust collapses to,
    /// `mSpeed` (`0x08`) how fast its centre travels and `mStrengthMaxVel` (`0x0C`) how fast its
    /// power can change.
    pub in_use: u8,
    pub min_radius: f32,
    pub speed: f32,
    pub strength_max_vel: f32,
    pub influence: WindInfluence,
}

/// `GB_WIND_INFLUENCE` (`d_kankyo.h:51-64`), the global wind every particle layer is steered by.
/// The two override pointers become a flag and their strength, because nothing outside this module
/// can supply a pointer.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Wind {
    /// `mWindVec` (`0x00`) is the direction every particle layer is pushed along, and
    /// `mpWindVecOverride` (`0x0C`) becomes the flag standing in for a pointer nothing here can supply.
    pub wind_vec: Vec3,
    pub has_wind_vec_override: bool,
    /// `field_0x8` (`0x10`): the position the inner light is projected onto, whose `y` the wind streak
    /// layer uses as its one-shot ceiling test.
    pub m08: Vec3,
    /// `mWindPower` (`0x1C`) is the direction already scaled by its strength,
    /// `mWindStrengthOverride` (`0x20`) the strength an override supplies, and `mCustomWindPower`
    /// (`0x24`) the strength a weather tag asked for.
    pub wind_power: f32,
    pub wind_strength_override: f32,
    pub custom_wind_power: f32,
    /// `mTactWindAngleX` and `mTactWindAngleY` (`0x28`, `0x2A`) are the weather tag's wind, and
    /// `mEvtWindAngleX` and `mEvtWindAngleY` (`0x2C`, `0x2E`) the event's, which wins when set.
    pub tact_wind_angle_x: i16,
    pub tact_wind_angle_y: i16,
    pub evt_wind_angle_x: i16,
    pub evt_wind_angle_y: i16,
    /// `mTactWindAngleFlags` (`0x30`): bit zero says the weather tag's wind is live and bit seven says
    /// it blows the other way. `mEvtWindSet` (`0x31`) is the event wind's mode, where `2` means the
    /// wind is off and `0xFF` means the tag's wind is suspended.
    pub tact_wind_angle_flags: u8,
    pub evt_wind_set: u8,
}

/// `EF_THUNDER` (`d_kankyo.h:66-76`): the flash's state machine, its ramp timer and the light it
/// registers while it burns. The two state ranges either side of ten are the lit strike and the
/// unlit one.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Thunder {
    /// `mStatus` (`0x00`) is whether the storm is armed, `mState` (`0x01`) which of its four phases it
    /// is in, `field_0x2` (`0x02`) unused, and `mMode` (`0x04`) the weather tag's storm mode, which
    /// both arms the storm and holds it lit.
    pub status: u8,
    pub state: u8,
    pub m2: u8,
    pub mode: i32,
    /// `mFlashTimer` (`0x08`) is the flash's ramp, `field_0xc` (`0x0C`) and `field_0x10` (`0x10`) are
    /// cleared with it and read by nothing, and `mLightInfluence` (`0x18`) is the light the near
    /// strike registers while it burns.
    pub flash_timer: f32,
    pub mc: f32,
    pub m10: f32,
    pub light_influence: LightInfluence,
}

/// `WAVECHAN` (`d_kankyo.h:85-100`): the channel the sea-surface wave effect reads. The three
/// unnamed floats at `0x00`, `0x04` and `0x08` are only ever assigned by the channel's own init.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WaveChan {
    pub m00: f32,
    pub m04: f32,
    pub m08: f32,
    /// `mWaveSpeed` (`0x0C`) is the crest's drift rate, `mWaveSpawnDist` (`0x10`) how far along the
    /// camera axis it is spawned, and `mWaveSpawnRadius` (`0x14`) the square it is spawned in.
    pub wave_speed: f32,
    pub wave_spawn_dist: f32,
    pub wave_spawn_radius: f32,
    /// `mWaveScale` (`0x18`) is the drawn size, `mWaveScaleRand` (`0x1C`) the floor its per-crest
    /// draw is taken between, `mWaveCounterSpeedScale` (`0x20`) the scale on its phase rate and
    /// `mWaveScaleBottom` (`0x24`) the rate's floor.
    pub wave_scale: f32,
    pub wave_scale_rand: f32,
    pub wave_counter_speed_scale: f32,
    pub wave_scale_bottom: f32,
    /// `mWaveFlatInter` (`0x28`) is how hard the swell flattens under the player: at one the layer
    /// stops entirely, and below it each crest fades over a disc that grows with the player's wake.
    pub wave_flat_inter: f32,
    /// `mWaveCount` (`0x2C`) is how many crests the channel holds, `mWaveReset` (`0x2E`) whether they
    /// are rebuilt every frame, and `field_0x2f` (`0x2F`) is unused.
    pub wave_count: i16,
    pub wave_reset: u8,
    pub m2f: u8,
}

impl WaveChan {
    /// `dKy_wave_chan_init` (`d_kankyo_rain.cpp:2950-2965`). The last store overwrites the speed
    /// the channel was given two lines earlier, so the reference ends up at a tenth.
    pub fn init() -> Self {
        Self {
            m00: -1.0,
            m04: 0.0,
            m08: 0.0,
            wave_speed: 0.1,
            wave_spawn_dist: 3000.0,
            wave_spawn_radius: 3150.0,
            wave_scale: 250.0,
            wave_scale_bottom: 5.0,
            wave_scale_rand: 0.217,
            wave_counter_speed_scale: 1.6,
            wave_flat_inter: 0.0,
            wave_count: 0,
            wave_reset: 0,
            m2f: 0,
        }
    }
}

impl Default for WaveChan {
    fn default() -> Self {
        Self::init()
    }
}

/// The five screen-space points `dKyr_sun_move` probes around the sun's projected centre, and the
/// tally it reads back from the depth buffer (`d_kankyo_rain.cpp:604-629`). Filling the tally needs
/// a `dComIfGd_peekZ`, which no renderer in this port offers, so the host supplies it through
/// [`SunVisibility`] and this module supplies only the points, which is the part it can compute.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SunVisibility {
    /// `numPointsVisible`: how many of the five sample points read as clear sky.
    pub num_points_visible: u8,
    /// `numCenterPointsVisible`: how many of those were the centre point, which is the only sample
    /// the reference's partial-visibility correction keys on.
    pub num_center_points_visible: u8,
}

/// What one frame of the weather reads from the world, gathered once by [`Frame::capture`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frame<'a> {
    /// `dComIfGp_getCamera(0)->view.mLookat.mEye` and `mCenter`. Both are zero when the camera has
    /// never been aimed, which is what `dComIfGd_getView() == NULL` leaves behind.
    pub eye: Vec3,
    pub center: Vec3,
    /// `dCamera_c::mCamera.mTrimHeight`, the lower screen edge the sun's depth probe clips against.
    pub trim_height: f32,
    /// `dComIfGd_getView()->mFovy`, the projection's vertical field of view in radians.
    pub fovy: f32,
    /// `dComIfGd_getView()->projViewMat` and `dComIfGd_getViewport()`, which `mDoLib_project` needs.
    /// The sun's screen distance and the lens flare's screen angle are both read off a projected
    /// point, and this is what they are projected through.
    pub proj_view: Mat4,
    pub port: ViewPort,
    /// Whether the camera has been aimed at all. The reference guards its sea-level and view-matrix
    /// reads on `dComIfGd_getView() != NULL`, and it is absent until the first frame.
    pub has_view: bool,
    /// `dComIfGp_getLinkPlayer()->current.pos`, read by the wind streaks, the sea surface and the
    /// poison curse.
    pub player_pos: Vec3,
    /// `g_Counter.mCounter0`, the frame counter the streaks and the piled snow stagger on.
    pub counter0: u32,
    /// `dStage_stagInfo_GetSTType(dComIfGp_getStageStagInfo())`.
    pub stage_type: u32,
    /// `dComIfGp_getStartStageName()`, the one place the reference compares the stage by name.
    pub stage: &'a str,
    /// `dComIfGp_roomControl_getStayNo()`, which the room's own tables are indexed by.
    pub room: i32,
    /// `dStage_FileList_dt_SeaLevel(fili_p)` for the current room. `wave_move` reads its own local
    /// copy of it uninitialised when the room has no file list (`d_kankyo_rain.cpp:1770-1776`),
    /// which is why this is an input rather than a zero.
    pub sea_level: f32,
    /// `dStage_FileList_dt_GlobalWindLevel(fili_p)`, which the global wind strength comes from.
    pub wind_level: i32,
    /// `dStage_FileList_dt_ChkPathWindEffect(fili_p)`, whether the room wants wind streaks at all.
    pub path_wind_effect: bool,
    /// `dComIfGp_particle_getParticleNum()`: the streaks stop spawning above fifteen hundred.
    pub particle_count: u32,
    /// `dComIfGp_checkPlayerStatus1(0, LEAF_FLIGHT)`, the player's leaf-flight status, which
    /// forces the streak count to ten.
    pub player_leaf_flight: bool,
    /// `dComIfGp_checkStatus(1)`, the mode gate on the sun, the stars and the dome cloud.
    pub status_1: bool,
}

impl Frame<'_> {
    /// The screen point `dKyr_sun_move` probes at, and `dKyr_lenzflare_move` measures the flare's
    /// screen angle from: `mDoLib_project` on the sun's own position.
    pub fn project(&self, src: Vec3) -> Vec3 {
        project(self, src)
    }
}

impl<'a> Frame<'a> {
    /// The per-frame snapshot the whole weather step is written against.
    pub fn capture<'b>(
        weather: &Weather,
        camera: Option<&Camera>,
        counter0: u32,
        stage: Option<&'b StageName>,
    ) -> Frame<'b> {
        let view = camera.and_then(Camera::view);
        Frame {
            eye: view.map_or(Vec3::ZERO, |view| view.lookat.eye),
            center: view.map_or(Vec3::ZERO, |view| view.lookat.center),
            trim_height: weather.trim_height,
            fovy: view.map_or(0.0, |view| view.fovy),
            proj_view: view.map_or(Mat4::ZERO, |view| view.proj_view),
            port: camera.map_or_else(ViewPort::default, |camera| camera.port),
            has_view: view.is_some(),
            player_pos: weather.player_pos,
            counter0,
            stage_type: weather.stage_type,
            stage: stage.map_or("", |stage| stage.0.as_str()),
            room: weather.room,
            sea_level: weather.sea_level,
            wind_level: weather.wind_level,
            path_wind_effect: weather.path_wind_effect,
            particle_count: weather.particle_count,
            player_leaf_flight: weather.player_leaf_flight,
            status_1: weather.status_1,
        }
    }
}

/// `overhead_bg_chk` (`d_kankyo_rain.cpp:748-767`): either a roof over the camera or a ground
/// surface above it, either of which stops the rain falling where the player is.
fn overhead_bg_chk(collision: Option<&Collision>, eye: Vec3) -> bool {
    let Some(collision) = collision else {
        return false;
    };
    let mut hit = false;
    let mut pos = eye;
    pos.y += 50.0;
    if collision.roof_chk(pos).is_some() {
        hit = true;
    }
    pos.y += 10000.0;
    let ground = collision
        .ground_cross_below(pos)
        .map_or(F_INF, |hit| hit.height);
    ground > eye.y + 50.0 || hit
}

/// `forward_overhead_bg_chk` (`d_kankyo_rain.cpp:770-796`): the same two arms taken at a point
/// `dist` along the camera axis, which the caller keeps for the fade test that follows.
fn forward_overhead_bg_chk(
    collision: Option<&Collision>,
    eye: Vec3,
    center: Vec3,
    dist: f32,
) -> (bool, Vec3) {
    let look = vectle(eye, center);
    let mut pos = Vec3::new(
        eye.x + look.x * dist,
        eye.y + look.y * dist,
        eye.z + look.z * dist,
    );
    pos.y = eye.y + 50.0;
    let Some(collision) = collision else {
        return (false, pos);
    };
    let mut hit = false;
    if collision.roof_chk(pos).is_some() {
        hit = true;
    }
    pos.y += 10000.0;
    let ground = collision
        .ground_cross_below(pos)
        .map_or(F_INF, |hit| hit.height);
    (ground > eye.y + 50.0 || hit, pos)
}

/// The weather half of `dScnKy_env_light_c` (`d_kankyo.h:169-357`), laid out by the offsets that
/// header documents rather than by reading order. Its [`Default`] is `dKyw_wether_init`
/// (`d_kankyo_wether.cpp:266-291`) followed by `dKyw_wind_init` (`:961-983`),
/// `dKyw_pntwind_init` (`:1167-1175`) and `dKy_wave_chan_init` (`d_kankyo_rain.cpp:2950-2965`).
#[derive(Resource, Debug, Clone)]
pub struct Weather {
    /// The `cM_rnd` lanes, carried because every spawn in the weather draws from them.
    pub seed: [i32; 3],

    /// `mCurTime` (`0xC38`), the clock the sun, the moon and the stars are timed on. `setDaytime`
    /// (`d_kankyo.cpp:502-546`) owns it: it reads the game clock in, steps it while a waker is held
    /// and no event is running, and writes it back.
    pub time: f32,
    /// `mDayOfWeek` (`0xC82`), the day the same function carries.
    pub day_of_week: u16,
    /// `mTimeAdv` (`0xC40`), how far `setDaytime` steps the clock each frame.
    pub time_adv: f32,

    /// `mColpatPrev` (`0xC84`): the applied colour pattern being faded out of, and `mColpatCurr`
    /// (`0xC85`) the one being faded into. `mColPatBlend` (`0xBE0`) is how far along the fade is.
    pub colpat_prev: u8,
    pub colpat_curr: u8,
    pub colpat_blend: f32,
    /// `mColPatMode` (`0xC8F`): the gather handoff's mode, latched from `mColPatModeGather`
    /// (`0xC90`) before the gather is advanced.
    pub colpat_mode: u8,

    /// `mAllColRatio` (`0xBFC`) and its five siblings at `0xC00`, `0xC04`, `0xC08`, `0xC0C` and
    /// `0xC10`: the six factors `setLight` multiplies every palette entry by, which the sun's own
    /// wash is what moves.
    pub all_col_ratio: f32,
    pub act_col_ratio: f32,
    pub bg_col_ratio: f32,
    pub fog_col_ratio: f32,
    pub vrbox_sora_col_ratio: f32,
    pub vrbox_kumo_col_ratio: f32,
    /// `mAllColGatherRatio` (`0xC14`) and its five siblings up to `0xC28`: the same six factors one
    /// step earlier in the handoff. Whatever writes these this frame is promoted to the applied pair
    /// by the next [`color_pattern_handoff`] and then cleared, which is why the sun's wash lags the
    /// sun by a frame.
    pub all_col_gather_ratio: f32,
    pub act_col_gather_ratio: f32,
    pub bg_col_gather_ratio: f32,
    pub fog_col_gather_ratio: f32,
    pub vrbox_sora_col_gather_ratio: f32,
    pub vrbox_kumo_col_gather_ratio: f32,

    /// `mSunPos` (`0xAB4`) and `mMoonPos` (`0xAC0`), the two points `setSunpos` swings eight
    /// thousand and forty-eight thousand units out from the camera on a circle that turns once a day.
    pub sun_pos: Vec3,
    pub moon_pos: Vec3,
    /// `mSunPos2` (`0xA9C`), the position `dKyr_sun_move` puts the sun at instead when the base light
    /// is lit or the stage turns it off.
    pub sun_pos2: Vec3,
    /// `mBaseLightInfluence` (`0x018`): the sun doubles as the directional light, so its colour is
    /// read to decide which position to aim from.
    pub base_light: LightInfluence,
    /// `mBG0_K0` (`0xB30`), the background key colour the wind streaks and the birds tint from.
    pub bg0_k0: ColorS10,

    /// `mWind` (`0x97C`).
    pub wind: Wind,
    /// `mWindInfluenceEntity` (`0x808`) and `mpWindInfluence` (`0x790`). The pointer array becomes
    /// an index array because the gusts are stored by value here.
    pub wind_entity: [WindEntity; SQUALL_MAX],
    /// The thirty slots of `mpWindInfluence` (`0x790`), each holding the index of the gust that
    /// claimed it. The reference holds pointers; here the gusts live in [`Weather::wind_entity`] by
    /// value, so a slot holds the index instead.
    pub point_wind: [Option<usize>; POINT_WIND_MAX],

    /// `mWaveChan` (`0x94C`).
    pub wave_chan: WaveChan,
    /// `mpWaveInfl` (`0xA68`): the ten registered sea-surface gusts.
    pub wave_infl: [Option<WaveInfo>; WAVE_INFL_MAX],

    /// `mThunderEff` (`0xA30`), and the additive colours its flash drives.
    pub thunder: Thunder,
    pub flash_color: FlashColor,

    /// `mWindlineCount` (`0xA28`) and `mbWindlineInitialized` (`0xA24`).
    pub windline_count: i32,
    pub windline_initialized: bool,
    /// `mpWind` (`0xA2C`): the wind streaks and the birds.
    pub wind_eff: WindEffSet,

    /// `mStarCount` (`0x9D8`), `mStarCountTarget` (`0x9DC`), `mbStarInitialized` (`0x9D4`) and
    /// `mpStarPacket` (`0x9E0`). The count is a `s16` in the reference, taken from the target scaled
    /// by a thousand, so a target of one is a thousand stars.
    pub star_count: i16,
    pub star_count_target: f32,
    pub star_initialized: bool,
    pub star: StarPacket,

    /// `mPoisonCount` (`0x9E8`), `mbPoisonInitialized` (`0x9E4`) and `mpPoisonPacket` (`0x9EC`).
    pub poison_count: i32,
    pub poison_initialized: bool,
    pub poison: PoisonPacket,
    /// `now_room` (`d_kankyo_rain.cpp:23`), the room the poison emitters were last rebuilt in.
    pub poison_room: i32,
    /// True on the frame `onPoisonCurse` would have run: the poison touched the player.
    pub poison_touching: bool,

    /// `mbHousiInitialized` (`0x9F0`), `mHousiCount` (`0x9F4`) and `mpHousiPacket` (`0x9F8`). The
    /// count itself is owned by [`EnvLight`], which is what the weather tags write.
    pub housi_initialized: bool,
    pub housi: HousiPacket,

    /// `mMoyaInitialized` (`0x9FC`), `mMoyaMode` (`0x9FD`), `mMoyaCount` (`0xA00`) and
    /// `mpMoyaPacket` (`0xA04`). The mode and count are owned by [`EnvLight`].
    pub moya_initialized: bool,
    pub cloud: CloudPacket,

    /// `mVrkumoStatus` (`0xA08`), `mVrkumoCount` (`0xA0C`), `mVrkumoStrength` (`0xA10`) and
    /// `mpVrkumoPacket` (`0xA14`).
    pub vrkumo_status: u8,
    pub vrkumo_count: i32,
    pub vrkumo_strength: f32,
    pub vrkumo: VrkumoPacket,

    /// `mWaveInitialized` (`0xA18`) and `field_0xa1c` (`0xA1C`), which `dKyw_wether_init` clears and
    /// nothing in the reference reads again.
    pub wave_initialized: bool,
    pub m_a1c: i32,
    pub wave: WavePacket,

    /// `mbSunInitialized` (`0x9B0`), then `mpSunPacket` (`0x9B4`), `mpSunlenzPacket` (`0x9B8`),
    /// `mbRainInitialized` (`0x9BC`) and `mRainCount` (`0x9C0`).
    pub sun_initialized: bool,
    pub sun: SunPacket,
    pub sun_lenz: SunLenzPacket,
    /// [`SunVisibility`], filled by the host because the five-point probe is a depth-buffer read
    /// (`dComIfGd_peekZ`) no renderer here offers.
    pub sun_visibility: SunVisibility,

    /// `mbRainInitialized` (`0x9BC`), the flag the weather state machine keys the rain layer on.
    pub rain_initialized: bool,
    pub rain: RainPacket,
    /// `mbSnowInitialized` (`0x9C8`), `mSnowCount` (`0x9CC`) and `mpSnowPacket` (`0x9D0`).
    pub snow_initialized: bool,
    pub snow: SnowPacket,

    /// `mbVrboxInvisible` (`0xC96`). Its only writer in the reference is the virtual-box actor,
    /// which is not part of this port's simulation, so the stars and the dome cloud see it clear and
    /// the host is expected to set it when it has one.
    pub vrbox_invisible: bool,
    /// `mbContrastFlag` (`0xC97`). `dKy_contrast_flg_set` (`d_kankyo.cpp:3560`) is its only writer
    /// and nothing in the reference calls it, so it stays clear.
    pub contrast_flag: bool,
    /// `field_0xc98` (`0xC98`), set by the item-get colour mode at `d_kankyo.cpp:2985-2987`: while it
    /// is set the thunder flash leaves the palette alone.
    pub item_col_mode: bool,
    /// `mInitAnimTimer` (`0xC94`): while it is set the wind snaps rather than eases and the sun and
    /// moon do not move.
    pub init_anim_timer: u8,
    /// `mMoyaSE` (`0xC50`): the sound effect the cloud layer last chose, cleared at the top of every
    /// weather draw pass.
    pub moya_se: u32,

    /// `dComIfGp_roomControl_getStayNo()`. The reference reads it from the room-control process,
    /// which this module does not own, so the host writes it here.
    pub room: i32,
    /// `dStage_FileList_dt_SeaLevel` for the current room, which the wave and dome-cloud layers
    /// measure against.
    pub sea_level: f32,
    /// `dStage_FileList_dt_GlobalWindLevel` for the current room, which the global wind strength
    /// comes from.
    pub wind_level: i32,
    /// `dStage_FileList_dt_ChkPathWindEffect` for the current room.
    pub path_wind_effect: bool,
    /// `dComIfGp_particle_getParticleNum()`: the wind streaks stop spawning above fifteen hundred.
    pub particle_count: u32,
    /// `dComIfGp_checkPlayerStatus1(0, LEAF_FLIGHT)`, the player's leaf-flight status, which
    /// forces the streak count to ten.
    pub player_leaf_flight: bool,
    /// `dComIfGp_checkStatus(1)`, the mode gate on the sun, the stars and the dome cloud.
    pub status_1: bool,
    /// `dStage_stagInfo_GetSTType(dComIfGp_getStageStagInfo())`.
    pub stage_type: u32,
    /// `dComIfGp_getLinkPlayer()->current.pos`, which the streaks, the sea surface and the poison
    /// curse all read.
    pub player_pos: Vec3,
    /// `dCamera_c::mCamera.mTrimHeight`.
    pub trim_height: f32,
}

impl Default for Weather {
    fn default() -> Self {
        Self {
            seed: [100, 100, 100],
            time: 0.0,
            day_of_week: 0,
            time_adv: 0.02,
            colpat_prev: 0,
            colpat_curr: 0,
            colpat_blend: 0.0,
            colpat_mode: 0,
            all_col_ratio: 1.0,
            act_col_ratio: 1.0,
            bg_col_ratio: 1.0,
            fog_col_ratio: 1.0,
            vrbox_sora_col_ratio: 1.0,
            vrbox_kumo_col_ratio: 1.0,
            all_col_gather_ratio: 1.0,
            act_col_gather_ratio: 1.0,
            bg_col_gather_ratio: 1.0,
            fog_col_gather_ratio: 1.0,
            vrbox_sora_col_gather_ratio: 1.0,
            vrbox_kumo_col_gather_ratio: 1.0,
            sun_pos: Vec3::new(1.0, 0.0, 0.0),
            moon_pos: Vec3::new(-1.0, 0.0, 0.0),
            sun_pos2: Vec3::ZERO,
            base_light: LightInfluence::default(),
            bg0_k0: ColorS10::rgb(0, 0, 0),
            wind: Wind {
                wind_vec: Vec3::new(-1.0, 0.0, 0.0),
                m08: Vec3::splat(10000000.0),
                ..Wind::default()
            },
            wind_entity: core::array::from_fn(|_| WindEntity::default()),
            point_wind: [None; POINT_WIND_MAX],
            wave_chan: WaveChan::init(),
            wave_infl: [None; WAVE_INFL_MAX],
            thunder: Thunder::default(),
            flash_color: FlashColor::default(),
            windline_count: 0,
            windline_initialized: false,
            wind_eff: WindEffSet::default(),
            star_count: 0,
            star_count_target: 0.0,
            star_initialized: false,
            star: StarPacket::default(),
            poison_count: 0,
            poison_initialized: false,
            poison: PoisonPacket::default(),
            poison_room: -1,
            poison_touching: false,
            housi_initialized: false,
            housi: HousiPacket::default(),
            moya_initialized: false,
            cloud: CloudPacket::default(),
            vrkumo_count: 0,
            vrkumo_strength: 0.0,
            vrkumo_status: 0,
            vrkumo: VrkumoPacket::default(),
            wave_initialized: false,
            m_a1c: 0,
            wave: WavePacket::default(),
            sun_initialized: false,
            sun: SunPacket::default(),
            sun_lenz: SunLenzPacket::default(),
            sun_visibility: SunVisibility::default(),
            rain_initialized: false,
            rain: RainPacket::default(),
            snow_initialized: false,
            snow: SnowPacket::default(),
            vrbox_invisible: false,
            contrast_flag: false,
            item_col_mode: false,
            init_anim_timer: 0,
            moya_se: 0,
            room: -1,
            sea_level: 0.0,
            wind_level: 0,
            path_wind_effect: false,
            particle_count: 0,
            player_leaf_flight: false,
            status_1: true,
            stage_type: 0,
            player_pos: Vec3::ZERO,
            trim_height: 0.0,
        }
    }
}

impl Weather {
    /// `dKyw_get_wind_vec` (`d_kankyo_wether.cpp:1088-1090`).
    pub fn wind_vec(&self) -> Vec3 {
        self.wind.wind_vec
    }

    /// `dKyw_get_wind_pow` (`d_kankyo_wether.cpp:1093-1095`).
    pub fn wind_pow(&self) -> f32 {
        self.wind.wind_power
    }

    /// `dKyw_get_wind_vecpow` (`d_kankyo_wether.cpp:1103-1106`): the direction already scaled by the
    /// power, which is what every particle layer actually reads.
    pub fn wind_vecpow(&self) -> Vec3 {
        self.wind.wind_vec * self.wind.wind_power
    }

    /// `dKy_rain_check` (`d_kankyo.cpp:3422-3425`).
    pub fn rain_check(&self) -> i32 {
        i32::from(self.rain.rain_count)
    }
}

/// `dKyw_pntwind_get_info` (`d_kankyo_wether.cpp:1215-1271`): the first registered gust whose volume
/// holds the point wins outright. A growing gust (`mbConstant` clear) blows outward from the point on
/// its leading edge and falls off quadratically; a constant one blows along its own `mDir` and falls
/// off linearly.
pub fn pntwind_get_info(weather: &Weather, pos: Vec3) -> (Vec3, f32) {
    for slot in weather.point_wind.iter().flatten() {
        let index = *slot;
        let influence = weather.wind_entity[index].influence;
        let dist = pos.distance(influence.pos);
        if dist >= influence.radius || influence.strength <= 0.0 || dist == 0.0 {
            continue;
        }
        if influence.constant == 0 {
            let falloff = if influence.radius > 0.0 {
                1.0 - square(dist / influence.radius)
            } else {
                1.0
            };
            let lead = influence.pos - influence.dir * influence.radius;
            let mut dir = vectle(lead, pos);
            if pos.distance(lead) < influence.radius {
                dir = Vec3::ZERO;
            }
            return (dir, influence.strength * falloff);
        }
        let falloff = if influence.radius > 0.0 {
            1.0 - (dist / influence.radius)
        } else {
            1.0
        };
        return (influence.dir, influence.strength * falloff);
    }
    (Vec3::ZERO, 0.0)
}

/// `dKyw_pntwind_get_vecpow` (`d_kankyo_wether.cpp:1274-1281`).
pub fn pntwind_vecpow(weather: &Weather, pos: Vec3) -> Vec3 {
    let (dir, power) = pntwind_get_info(weather, pos);
    dir * power
}

/// `dKyw_pntwind_set` over `pntwind_set` (`d_kankyo_wether.cpp:1178-1197`): take the first free
/// registration slot and record the index in the influence.
pub fn pntwind_set(weather: &mut Weather, entity: usize) {
    weather.wind_entity[entity].influence.constant = 0;
    for slot in 0..POINT_WIND_MAX {
        if weather.point_wind[slot].is_none() {
            weather.point_wind[slot] = Some(entity);
            weather.wind_entity[entity].influence.regist_idx = entity as i32;
            return;
        }
    }
    weather.wind_entity[entity].influence.regist_idx = 9999;
}

/// `dKyw_pntwind_cut` (`d_kankyo_wether.cpp:1206-1212`).
pub fn pntwind_cut(weather: &mut Weather, entity: usize) {
    let idx = weather.wind_entity[entity].influence.regist_idx;
    if idx >= 0 && (idx as usize) < POINT_WIND_MAX {
        weather.point_wind[idx as usize] = None;
    }
}

/// `dKyw_squal_set` (`d_kankyo_wether.cpp:1109-1133`): claim the first free gust slot and register
/// it. Returns the slot claimed.
pub fn squal_set(
    weather: &mut Weather,
    pos: Vec3,
    x: i16,
    y: i16,
    radius: f32,
    min_radius: f32,
    strength: f32,
    speed: f32,
    max_strength: f32,
) -> Option<usize> {
    let slot = weather
        .wind_entity
        .iter()
        .position(|entity| entity.in_use == 0)?;
    let entity = &mut weather.wind_entity[slot];
    entity.in_use = 1;
    entity.min_radius = min_radius;
    entity.speed = speed;
    entity.strength_max_vel = max_strength;
    entity.influence.pos = pos;
    entity.influence.dir = Vec3::new(scos(x) * ssin(y), ssin(x), scos(x) * scos(y));
    entity.influence.radius = radius;
    entity.influence.m20 = radius;
    entity.influence.strength = strength;
    pntwind_set(weather, slot);
    Some(slot)
}

/// `squal_proc` (`d_kankyo_wether.cpp:1136-1164`), the one thing `dKyw_wether_proc`
/// (`d_kankyo.cpp:2082`) runs inside the environment light's execute.
pub fn squal_proc(weather: &mut Weather) {
    for index in 0..SQUALL_MAX {
        if weather.wind_entity[index].in_use == 0 {
            continue;
        }
        let speed = weather.wind_entity[index].speed;
        let max_vel = weather.wind_entity[index].strength_max_vel;
        let min_radius = weather.wind_entity[index].min_radius;
        let dir = weather.wind_entity[index].influence.dir;
        weather.wind_entity[index].influence.pos += dir * speed;

        add_calc(
            &mut weather.wind_entity[index].influence.strength,
            0.0,
            0.2,
            max_vel,
            0.001,
        );
        let strength = weather.wind_entity[index].influence.strength;
        let chase = 1.0 - strength;
        add_calc(
            &mut weather.wind_entity[index].influence.radius,
            min_radius,
            chase,
            chase * min_radius * 0.05,
            0.01,
        );
        if strength < 0.01 {
            pntwind_cut(weather, index);
            weather.wind_entity[index].in_use = 0;
        }
    }
}

/// `dKyw_custom_windpower` (`d_kankyo_wether.cpp:1363-1365`).
pub fn custom_wind_power(weather: &mut Weather, power: f32) {
    weather.wind.custom_wind_power = power;
}

/// `dKyw_evt_wind_set` (`d_kankyo_wether.cpp:1395-1398`).
pub fn evt_wind_set(weather: &mut Weather, x: i16, y: i16) {
    weather.wind.evt_wind_angle_x = x;
    weather.wind.evt_wind_angle_y = y;
}

/// `dKyw_evt_wind_set_go` (`d_kankyo_wether.cpp:1401-1403`).
pub fn evt_wind_set_go(weather: &mut Weather) {
    weather.wind.evt_wind_set = 1;
}

/// `dKyw_tact_wind_set_go` (`d_kankyo_wether.cpp:1347-1350`).
pub fn tact_wind_set_go(weather: &mut Weather) {
    weather.wind.tact_wind_angle_flags |= 1;
    weather.wind.evt_wind_set = 0xFF;
}

/// `dKyw_tact_wind_set` (`d_kankyo_wether.cpp:1329-1344`).
pub fn tact_wind_set(weather: &mut Weather, x: i16, y: i16) {
    weather.wind.tact_wind_angle_x = x;
    weather.wind.tact_wind_angle_y = y;
    let back = -(atan2s(weather.wind.wind_vec.x, weather.wind.wind_vec.z).wrapping_add(0x4000));
    weather.wind.tact_wind_angle_flags = 1;
    if back.wrapping_sub(weather.wind.tact_wind_angle_y) < 0 {
        weather.wind.tact_wind_angle_flags |= 0x80;
    }
}

/// `dKyw_get_tactwind_dir` (`d_kankyo_wether.cpp:1353-1360`).
pub fn tactwind_dir(weather: &Weather) -> i32 {
    i32::from(weather.wind.tact_wind_angle_flags & 0x80 != 0)
}

/// `dKyw_get_windsdir` (`d_kankyo_wether.cpp:1368-1392`): the compass angle of the wind run through
/// the eight buckets. The reference reads an uninitialised local when nothing matches, which cannot
/// happen because the last bucket ends at the top of the circle.
pub fn winds_dir(weather: &Weather) -> i32 {
    let angle =
        -(atan2s(weather.wind.wind_vec.x, weather.wind.wind_vec.z).wrapping_add(0x4000)) as u16;
    let mut dir = 0;
    for (min, max, value) in WIND_DIR_TABLE {
        if angle >= min && angle < max {
            dir = value;
        }
    }
    dir
}

/// The per-stage wind bias both `wave_move` and `vrkumo_move` add to the tactile wind
/// (`d_kankyo_rain.cpp:1801-1811` and `:2793-2803`).
fn stage_misc_wind(stage: &str) -> i16 {
    match stage {
        "LinkRM" => 0x4000,
        "Orichh" => -0x4000,
        "Ojhous2" => 0x7FFF,
        "Omasao" => -0x4000,
        "Onobuta" => 0x4000,
        _ => 0,
    }
}

/// `dKyw_wind_set` (`d_kankyo_wether.cpp:986-1085`): the direction comes from the event wind, then
/// the tactile wind, then zero; the strength comes from the room's global wind level, then the
/// custom override, and is cut by the second event-wind mode before it is eased into place.
pub fn wind_set(weather: &mut Weather, frame: &Frame) {
    let (mut wind_vec, mut strength) = if weather.wind.has_wind_vec_override {
        (weather.wind.wind_vec, weather.wind.wind_strength_override)
    } else {
        let (x, y) = if weather.wind.evt_wind_set != 0 && weather.wind.evt_wind_set != 0xFF {
            (weather.wind.evt_wind_angle_x, weather.wind.evt_wind_angle_y)
        } else if weather.wind.tact_wind_angle_flags != 0 {
            (
                weather.wind.tact_wind_angle_x,
                weather.wind.tact_wind_angle_y,
            )
        } else {
            (0, 0)
        };
        let vec = Vec3::new(scos(x) * scos(y), ssin(x), scos(x) * ssin(y));
        let level = match frame.wind_level {
            0 => 0.3,
            1 => 0.6,
            2 => 0.9,
            _ => 0.0,
        };
        (vec, level)
    };
    if frame.stage_type == STAGE_TYPE_MISC {
        let angle_y = weather
            .wind
            .tact_wind_angle_y
            .wrapping_add(stage_misc_wind(frame.stage));
        let normal = Vec3::new(
            scos(weather.wind.tact_wind_angle_x) * scos(angle_y),
            ssin(weather.wind.tact_wind_angle_x),
            scos(weather.wind.tact_wind_angle_x) * ssin(angle_y),
        );
        wind_vec = normal * 0.6;
        strength = 0.6;
    }
    if weather.wind.custom_wind_power > 0.0 {
        strength = weather.wind.custom_wind_power;
        weather.wind.wind_power = weather.wind.custom_wind_power;
    }
    if weather.wind.evt_wind_set == 2 {
        strength = 0.0;
    }
    strength = strength.min(1.0);
    if weather.init_anim_timer != 0 {
        weather.wind.wind_vec = wind_vec;
        weather.wind.wind_power = strength;
        return;
    }
    add_calc(&mut weather.wind.wind_vec.x, wind_vec.x, 0.1, 2.0, 0.001);
    add_calc(&mut weather.wind.wind_vec.y, wind_vec.y, 0.1, 2.0, 0.001);
    add_calc(&mut weather.wind.wind_vec.z, wind_vec.z, 0.1, 2.0, 0.001);
    add_calc(&mut weather.wind.wind_power, strength, 0.1, 1.0, 0.005);
}

/// `dKyr_thunder_init` (`d_kankyo_rain.cpp:5541-5543`).
pub fn thunder_init(weather: &mut Weather) {
    weather.thunder.state = 0;
}

/// `dKyr_thunder_move` (`d_kankyo_rain.cpp:5546-5628`). The near strike registers an influence and
/// the far one does not, which is why the two state ranges sit either side of ten.
pub fn thunder_move(weather: &mut Weather, frame: &Frame) {
    let state = weather.thunder.state;
    match state {
        0 => {
            weather.thunder.flash_timer = 0.0;
            weather.thunder.mc = 0.0;
            weather.thunder.m10 = 0.0;
            if rnd(&mut weather.seed) < 0.007 {
                weather.thunder.state = 11;
            } else if rnd(&mut weather.seed) < 0.005 && weather.thunder.mode < 10 {
                weather.thunder.light_influence.pos = frame.eye;
                weather.thunder.light_influence.color = ColorS10::rgb(0, 0, 0);
                weather.thunder.light_influence.power = 90000.0;
                weather.thunder.light_influence.fluctuation = 150.0;
                weather.thunder.state += 1;
            }
        }
        1 | 11 => {
            add_calc(&mut weather.thunder.flash_timer, 1.0, 0.3, 0.2, 0.001);
            if weather.thunder.flash_timer >= 1.0 {
                weather.thunder.state += 1;
            }
        }
        2 | 12 => {
            add_calc(&mut weather.thunder.flash_timer, 0.0, 0.1, 0.05, 0.001);
            if weather.thunder.flash_timer <= 0.0 {
                weather.thunder.state = 0;
                if weather.thunder.mode == 0 {
                    weather.thunder.status = 0;
                }
            }
        }
        _ => {}
    }
    if weather.thunder.state == 0 {
        return;
    }
    let ramp = weather.thunder.flash_timer;
    if weather.thunder.state < 10 {
        weather.thunder.light_influence.pos =
            Vec3::new(frame.eye.x, frame.eye.y + 150.0, frame.eye.z);
        weather.thunder.light_influence.color = ColorS10::rgb(
            (ramp * 0.2 * 180.0) as u8,
            (ramp * 0.2 * 235.0) as u8,
            (ramp * 0.2 * 255.0) as u8,
        );
        if !weather.item_col_mode {
            weather.flash_color.actor = ramp * 0.5;
            weather.flash_color.bg = ramp * 0.7;
            weather.flash_color.bg1 = ramp * 0.35;
            weather.flash_color.sky_0 = ramp * 0.4;
            weather.flash_color.kasumi = ramp * 0.5;
            weather.flash_color.fog = ramp * 0.3;
        }
    } else {
        weather.flash_color.sky_0 = ramp * 0.15;
        weather.flash_color.kasumi = ramp * 0.35;
        weather.flash_color.fog = ramp * 0.12;
    }
}

/// `wether_move_thunder` (`d_kankyo_wether.cpp:453-465`): the storm arms itself once, when the mode
/// a weather tag asked for becomes non-zero, and never disarms while that mode holds.
pub fn wether_move_thunder(weather: &mut Weather, frame: &Frame) {
    if weather.thunder.status == 0 {
        if weather.thunder.mode != 0 {
            thunder_init(weather);
            weather.thunder.status = 1;
        }
        return;
    }
    thunder_move(weather, frame);
}

/// `dKy_set_actcol_ratio` over `dKy_set_bgcol_ratio`, `dKy_set_fogcol_ratio` and
/// `dKy_set_vrboxcol_ratio` (`d_kankyo.cpp:3511-3530`) as `dKyr_sun_move` calls them: looking
/// straight into the sun washes the actor and background colours out and brightens the fog. Each of
/// those writes the gather-side ratio, which `exeKankyo` promotes to the applied one on the next
/// frame and then clears, so the wash lands a frame behind the sun.
fn sun_colour_ratios(weather: &mut Weather, staring: f32, visibility: f32, pulse: f32) {
    weather.act_col_gather_ratio = 1.0 - staring * visibility;
    weather.bg_col_gather_ratio = 1.0 - staring * visibility;
    weather.fog_col_gather_ratio = staring * visibility * pulse * 0.5 + 1.0;
    weather.vrbox_sora_col_gather_ratio = staring * visibility * pulse * 0.5 + 1.0;
    weather.vrbox_kumo_col_gather_ratio = staring * visibility * pulse * 0.5 + 1.0;
}

/// `dKyr_moon_arrival_check` (`d_kankyo_rain.cpp:545-550`): the moon is up through the first half
/// of the night and gone through the second.
pub fn moon_arrival_check(weather: &Weather) -> bool {
    weather.time > 277.5 || weather.time < 112.5
}

/// `dKyr_sun_move`'s five-point screen test (`d_kankyo_rain.cpp:612-629`): how many of the
/// `sun_chkpnt` offsets around the sun's projected centre fall outside the visible screen rectangle,
/// which is what the reference's partial-visibility correction keys on. The lower edge is the
/// camera's trim height and the upper one is the fixed four-ninety the reference clips against.
fn sun_chk_culled(frame: &Frame, projected: Vec3) -> i32 {
    let border = frame.trim_height;
    let bottom = 490.0 - border;
    let mut culled = 0;
    for (offset_x, offset_y) in SUN_CHKPNT {
        let sample_x = projected.x - offset_x;
        let sample_y = projected.y - offset_y;
        if !(sample_x > 0.0 && sample_x < 640.0 && sample_y > border && sample_y < bottom) {
            culled += 1;
        }
    }
    culled
}

/// `dKyr_sun_move` (`d_kankyo_rain.cpp:553-718`). `colpat_weather` is `mColpatWeather` (`0xC8A`),
/// which the environment light's own colour gather writes (`d_kankyo.cpp:2058`). Everything the
/// function computes from the camera runs here, including the five screen sample points and the
/// partial-visibility correction they feed. The one arm that cannot is the depth read itself: a
/// `dComIfGd_peekZ` no renderer in this port offers, so the visible and centre-point counts arrive
/// through [`SunVisibility`] instead.
pub fn sun_move(weather: &mut Weather, frame: &Frame, colpat_weather: u8) {
    let light_dir = if weather.base_light.color.r == 0 && frame.stage_type != STAGE_TYPE_MISC {
        vectle(frame.eye, weather.base_light.pos)
    } else {
        vectle(frame.eye, weather.sun_pos2)
    };
    let sun_pos = frame.eye + light_dir * 8000.0;
    weather.sun.pos[0] = sun_pos;

    let mut horizon = (sun_pos.y - frame.eye.y) / 8000.0;
    horizon = horizon.clamp(0.0, 1.0);
    horizon = 1.0 - horizon;
    horizon = square(horizon);
    let pulse_pos = 1.0 - horizon;

    if weather.sun.m3c != 0 {
        weather.sun.m3c -= 1;
    }
    weather.sun.m3d = false;

    let mut staring_at_sun = 0.0f32;
    let mut num_points_visible;
    let mut num_center_points_visible;
    if weather.time > 97.5 && weather.time < 292.5 {
        add_calc(&mut weather.sun.sun_alpha, 1.0, 0.5, 0.1, 0.01);

        let projected = project(frame, weather.sun.pos[0]);
        let culled = sun_chk_culled(frame, projected);

        num_points_visible = weather.sun_visibility.num_points_visible;
        num_center_points_visible = weather.sun_visibility.num_center_points_visible;
        if culled != 0 && num_points_visible != 0 && num_center_points_visible != 0 {
            num_center_points_visible = 1;
            num_points_visible = 5;
        }

        let centre = Vec3::new(320.0, 240.0, 0.0);
        weather.sun_lenz.dist_falloff = (centre.distance(projected) / 450.0).min(1.0);
        weather.sun_lenz.dist_falloff = 1.0 - weather.sun_lenz.dist_falloff;
        staring_at_sun = square(weather.sun_lenz.dist_falloff);
        weather.sun_lenz.dist_falloff = 1.0 - staring_at_sun;
        staring_at_sun = square(staring_at_sun);

        let prev = weather.sun_lenz.m88;
        let prev_b = weather.sun_lenz.m8c;
        weather.sun_lenz.m80 = prev;
        weather.sun_lenz.m84 = prev_b;
        weather.sun_lenz.m88 = 1000000000.0;
        weather.sun_lenz.m8c = 0.0;
        if num_points_visible != 0 {
            if weather.sun.m3c < 5 {
                weather.sun.m3c += 2;
            }
            weather.sun.m3d = true;
        }
    } else {
        add_calc(&mut weather.sun.sun_alpha, 0.0, 0.5, 0.1, 0.01);
        num_points_visible = 0;
        num_center_points_visible = 0;
        weather.sun.m3c = 0;
        weather.sun.m3d = false;
    }

    if colpat_weather != 0 || (weather.colpat_curr != 0 && weather.colpat_blend > 0.5) {
        num_points_visible = 0;
        num_center_points_visible = 0;
    }
    if frame.stage_type == STAGE_TYPE_MISC {
        num_points_visible = 0;
        num_center_points_visible = 0;
    }
    if weather.time < 120.0 || weather.time > 270.0 {
        num_points_visible = 0;
        num_center_points_visible = 0;
    }

    if num_center_points_visible != 0 {
        if num_points_visible == 4 {
            add_calc(&mut weather.sun.visibility, 1.0, 0.1, 0.1, 0.001);
        }
        if num_points_visible <= 3 {
            add_calc(&mut weather.sun.visibility, 0.0, 0.1, 0.2, 0.001);
        } else {
            add_calc(&mut weather.sun.visibility, 1.0, 0.5, 0.2, 0.01);
        }
    } else if num_points_visible < 3 {
        add_calc(&mut weather.sun.visibility, 0.0, 0.5, 0.2, 0.001);
    } else {
        add_calc(&mut weather.sun.visibility, 1.0, 0.1, 0.1, 0.001);
    }

    weather.sun_lenz.draw_lenz_in_sky = num_points_visible < 2;

    if weather.sun.pos[0].y > 0.0 && !weather.sun_lenz.draw_lenz_in_sky {
        let visibility = weather.sun.visibility;
        sun_colour_ratios(weather, staring_at_sun, visibility, pulse_pos);
    }

    if moon_arrival_check(weather) {
        let mut alpha = (weather.sun.pos[0].y - frame.eye.y) / -8000.0;
        alpha = square(alpha);
        alpha *= 6.0;
        add_calc(
            &mut weather.sun.moon_alpha,
            alpha.min(1.0),
            0.2,
            0.01,
            0.001,
        );
    } else {
        add_calc(&mut weather.sun.moon_alpha, 0.0, 0.2, 0.01, 0.001);
    }
}

/// `dKyr_lenzflare_move` (`d_kankyo_rain.cpp:499-542`): a fan of eight points strung back from the sun
/// along its own direction, spaced by how squarely the sun faces the camera, and turned by the angle
/// the sun's projected position makes against the middle of the screen.
pub fn lenzflare_move(weather: &mut Weather, frame: &Frame) {
    if weather.sun.visibility < 0.0001 {
        return;
    }
    let eyevect = set_eyevect_calc(frame.eye, frame.center, 7200.0005, 7200.0005);
    let sun_pos = weather.sun.pos[0];
    let sun_dir = vectle(eyevect, sun_pos);
    weather.sun_lenz.positions[0] = sun_pos;
    weather.sun_lenz.positions[1] = sun_pos;

    let projected = project(frame, weather.sun_lenz.positions[0]);
    let centre = Vec3::new(320.0, 240.0, 0.0);
    let to_sun = vectle(centre, projected);
    weather.sun_lenz.angle_deg = atan2s(to_sun.x, to_sun.y) as f32 * S2DEG + 180.0;

    let cam_fwd = vectle(frame.eye, frame.center);
    let size = sun_dir.dot(cam_fwd).abs() * 350.0 + 250.0;
    for index in 2..LENZ_POSITIONS {
        weather.sun_lenz.positions[index] = sun_pos - sun_dir * (size * index as f32);
    }
}

/// `dKyr_rain_init` (`d_kankyo_rain.cpp:721-740`).
pub fn rain_init(weather: &mut Weather, frame: &Frame) {
    weather.rain.cam_eye_pos = frame.eye;
    weather.rain.cam_center_pos = frame.center;
    weather.rain.center_delta_mul = 0.0;
    weather.rain.m3700 = 0.0;
    weather.rain.sibuki_alpha = 0.0;
    weather.rain.overhead_fade = 0.0;
    weather.rain.fwd_fade1 = 0.0;
    weather.rain.fwd_fade2 = 0.0;
    weather.rain.status = 0;
    weather.rain.center_delta = Vec3::ZERO;
    for eff in &mut weather.rain.eff {
        eff.status = 0;
    }
    weather.rain.rain_count = 0;
}

/// `rain_bg_chk` (`d_kankyo_rain.cpp:743-745`): the height a drop dies at, which is the camera's
/// look-at point rather than anything on the ground.
fn rain_bg_chk(weather: &mut Weather, index: usize, frame: &Frame) {
    weather.rain.eff[index].m30 = frame.center.y + -800.0;
}

/// `dKyr_rain_move` (`d_kankyo_rain.cpp:799-1088`).
pub fn rain_move(
    weather: &mut Weather,
    frame: &Frame,
    rain_count: i32,
    collision: Option<&Collision>,
) {
    if i32::from(weather.rain.rain_count) <= rain_count {
        weather.rain.rain_count = rain_count as i16;
    }
    if weather.rain.rain_count == 0 {
        return;
    }

    let center_delta = vectle(frame.center, weather.rain.cam_center_pos);
    let mut eye_delta = weather.rain.cam_eye_pos.distance(frame.eye);
    eye_delta = if eye_delta > 10.0 {
        ((eye_delta - 10.0) / 50.0).min(1.0)
    } else {
        0.0
    };
    weather.rain.cam_eye_pos = frame.eye;
    let mut center_delta_len = weather.rain.cam_center_pos.distance(frame.center);
    center_delta_len = if center_delta_len > 10.0 {
        ((center_delta_len - 10.0) / 50.0).min(1.0)
    } else {
        0.0
    };
    add_calc(
        &mut weather.rain.center_delta_mul,
        eye_delta * center_delta_len,
        0.2,
        0.1,
        0.01,
    );
    if weather.rain.center_delta_mul > 0.3 {
        weather.rain.center_delta_mul = 0.3;
    }
    add_calc(
        &mut weather.rain.center_delta.x,
        center_delta.x,
        0.2,
        0.1,
        0.01,
    );
    add_calc(
        &mut weather.rain.center_delta.y,
        center_delta.y,
        0.2,
        0.1,
        0.01,
    );
    add_calc(
        &mut weather.rain.center_delta.z,
        center_delta.z,
        0.2,
        0.1,
        0.01,
    );
    weather.rain.cam_center_pos = frame.center;

    let spawn = set_eyevect_calc2(frame.eye, frame.center, 700.0, 600.0);
    let wind_pow = weather.wind_vecpow();
    weather.rain.m36cc = 0.0;
    weather.rain.m36c8 = 0.0;
    weather.rain.status = 0;

    let mut overhead = false;
    let mut forward1 = false;
    let mut forward2 = false;
    let mut fade_everywhere = false;
    let mut fade_max_dist = 0.0f32;
    let mut at1 = spawn;
    let mut at2 = spawn;
    if frame.stage_type == STAGE_TYPE_MISC && frame.stage != "Ocrogh" && frame.stage != "Omori" {
        fade_everywhere = true;
        weather.rain.status |= 1;
        fade_max_dist = if frame.stage == "Orichh" {
            2300.0
        } else {
            1200.0
        };
    } else {
        overhead = overhead_bg_chk(collision, frame.eye);
        let (hit1, point1) = forward_overhead_bg_chk(collision, frame.eye, frame.center, 700.0);
        forward1 = hit1;
        at1 = point1;
        let (hit2, point2) = forward_overhead_bg_chk(collision, frame.eye, frame.center, 1400.0);
        forward2 = hit2;
        at2 = point2;
        if overhead {
            weather.rain.status |= 1;
        } else if forward1 {
            weather.rain.status |= 2;
        }
        if frame.stage == "Ocrogh" {
            weather.rain.status = 0;
            overhead = false;
            forward1 = false;
            forward2 = false;
        }
    }

    if overhead {
        add_calc(&mut weather.rain.overhead_fade, 0.0, 0.5, 0.2, 0.01);
    } else {
        add_calc(&mut weather.rain.overhead_fade, 1.0, 0.1, 0.1, 0.001);
    }
    if forward1 {
        add_calc(&mut weather.rain.fwd_fade1, 0.0, 0.5, 0.2, 0.01);
    } else {
        add_calc(&mut weather.rain.fwd_fade1, 1.0, 0.1, 0.1, 0.001);
    }
    if forward2 {
        add_calc(&mut weather.rain.fwd_fade2, 0.0, 0.5, 0.2, 0.01);
    } else {
        add_calc(&mut weather.rain.fwd_fade2, 1.0, 0.1, 0.1, 0.001);
    }

    let mut index = i32::from(weather.rain.rain_count) - 1;
    while index >= 0 {
        let i = index as usize;
        weather.rain.eff[i].base_pos.y = spawn.y;
        match weather.rain.eff[i].status {
            0 => {
                let fall = -(rnd_f(&mut weather.seed, 10.0) + 35.5);
                weather.rain.eff[i].m24 = fall;
                weather.rain.eff[i].timer = 0;
                weather.rain.eff[i].base_pos = spawn;
                weather.rain.eff[i].pos = Vec3::new(
                    rnd_fx(&mut weather.seed, 800.0),
                    rnd_f(&mut weather.seed, 600.0),
                    rnd_fx(&mut weather.seed, 800.0),
                );
                weather.rain.eff[i].alpha = 1.0;
                weather.rain.eff[i].m1c = rnd_f(&mut weather.seed, 360.0);
                weather.rain.eff[i].m20 = rnd_f(&mut weather.seed, 360.0);
                rain_bg_chk(weather, i, frame);
                weather.rain.eff[i].status += 1;
            }
            1 | 2 | 3 => {
                let current = weather.rain.eff[i].m24;
                let jitter = rnd_fx(&mut weather.seed, 0.1);
                add_calc(
                    &mut weather.rain.eff[i].m24,
                    current - jitter,
                    0.5,
                    0.1,
                    0.01,
                );
                let mul = weather.rain.center_delta_mul;
                let delta = weather.rain.center_delta;
                weather.rain.eff[i].pos.x +=
                    20.0 * (wind_pow.x + 10.0 * (delta.x * mul) + 0.08 * (i & 7) as f32);
                weather.rain.eff[i].pos.y += 20.0 * (-2.5 + wind_pow.y + (delta.y * mul) * 10.0);
                weather.rain.eff[i].pos.z +=
                    20.0 * (wind_pow.z + 10.0 * (delta.z * mul) + 0.08 * (i & 3) as f32);

                let base = weather.rain.eff[i].base_pos;
                let mut probe = Vec3::new(
                    base.x + weather.rain.eff[i].pos.x,
                    spawn.y,
                    base.z + weather.rain.eff[i].pos.z,
                );
                let dist = probe.distance(spawn);
                if weather.rain.eff[i].timer == 0 {
                    if dist > 800.0 {
                        weather.rain.eff[i].timer = 10;
                        weather.rain.eff[i].m24 = -(rnd_f(&mut weather.seed, 10.0) + 35.5);
                        weather.rain.eff[i].base_pos = spawn;
                        if probe.distance(spawn) > 850.0 {
                            weather.rain.eff[i].pos = Vec3::new(
                                rnd_fx(&mut weather.seed, 800.0),
                                rnd_fx(&mut weather.seed, 800.0),
                                rnd_fx(&mut weather.seed, 800.0),
                            );
                        } else {
                            let push = rnd_fx(&mut weather.seed, 40.0);
                            let dir = vectle(probe, spawn);
                            weather.rain.eff[i].pos = dir * (push + 800.0);
                        }
                        weather.rain.eff[i].status = 1;
                        rain_bg_chk(weather, i, frame);
                    }
                    probe.y = weather.rain.eff[i].base_pos.y + weather.rain.eff[i].pos.y;
                    if probe.y < weather.rain.eff[i].m30 + 20.0 {
                        weather.rain.eff[i].base_pos = spawn;
                        weather.rain.eff[i].pos = Vec3::new(
                            rnd_fx(&mut weather.seed, 800.0),
                            200.0,
                            rnd_fx(&mut weather.seed, 800.0),
                        );
                        rain_bg_chk(weather, i, frame);
                        weather.rain.eff[i].timer = 10;
                    }
                } else {
                    weather.rain.eff[i].timer = weather.rain.eff[i].timer.wrapping_sub(1);
                }
            }
            _ => {}
        }

        let pos = weather.rain.eff[i].base_pos + weather.rain.eff[i].pos;
        if i as i32 > rain_count - 1 && i == weather.rain.rain_count as usize - 1 {
            weather.rain.rain_count = weather.rain.rain_count.wrapping_sub(1);
        }

        let mut alpha = 1.0f32;
        if overhead || weather.rain.overhead_fade < 1.0 {
            let mut flat = pos;
            flat.y = frame.eye.y;
            if frame.eye.distance(flat) < 800.0 {
                alpha *= weather.rain.overhead_fade;
            }
        }
        if forward1 || weather.rain.fwd_fade1 < 1.0 {
            let mut flat = pos;
            flat.y = at1.y;
            if at1.distance(flat) < 550.0 {
                alpha *= weather.rain.fwd_fade1;
            }
        }
        if forward2 || weather.rain.fwd_fade2 < 1.0 {
            let mut flat = pos;
            flat.y = at2.y;
            if at2.distance(flat) < 550.0 {
                alpha *= weather.rain.fwd_fade2;
            }
        }
        if fade_everywhere {
            let mut flat = pos;
            flat.y = 0.0;
            if flat.length() < fade_max_dist {
                alpha = 0.0;
            }
            if frame.stage == "Atorizk" && pos.y < 1300.0 {
                alpha = 0.0;
            }
        }
        weather.rain.eff[i].alpha = alpha;
        index -= 1;
    }
}

/// `dKyr_snow_init` (`d_kankyo_rain.cpp:1257-1272`).
pub fn snow_init(weather: &mut Weather, frame: &Frame) {
    for eff in &mut weather.snow.eff {
        eff.status = 0;
    }
    weather.snow.eff_count = 0;
    weather.snow.old_eye_pos = frame.eye;
}

/// The two packet fields both snow halves write: the cross-wind factor and the along-wind factor
/// the ground layer scales its drift by (`d_kankyo_rain.cpp:1313-1318` and `:1479-1484`).
fn snow_wind_factors(weather: &mut Weather, frame: &Frame) {
    let wind_vec = weather.wind_vec();
    let wind_power = weather.wind_pow();
    let heading = vectle_calc(Vec3::new(
        frame.center.x - frame.eye.x,
        0.0,
        frame.center.z - frame.eye.z,
    ));
    weather.snow.m36d4 =
        vector_product_2d(0.0, 0.0, -wind_vec.x, -wind_vec.z, heading.x, heading.z);
    let along = (wind_vec.x * heading.x + wind_vec.z * heading.z).abs();
    weather.snow.m36d0 =
        (1.0 - along) * wind_power * (1.0 - wind_vec.y.abs()) * weather.snow.m36d4.abs();
}

/// `dKyr_snow_move` (`d_kankyo_rain.cpp:1275-1427`).
pub fn snow_move(weather: &mut Weather, frame: &Frame, snow_count: i32) {
    if i32::from(weather.snow.eff_count) <= snow_count {
        weather.snow.eff_count = snow_count as i16;
    }
    if weather.snow.eff_count == 0 {
        return;
    }
    let spawn = set_eyevect_calc2(frame.eye, frame.center, 500.0, 500.0);
    if weather.snow.old_eye_pos.distance(frame.eye) > 1500.0 {
        weather.snow.eff_count = 0;
    }
    weather.snow.old_eye_pos = frame.eye;
    snow_wind_factors(weather, frame);
    let wind_pow = weather.wind_vecpow();

    let mut index = i32::from(weather.snow.eff_count) - 1;
    while index >= 0 {
        let i = index as usize;
        let gravity = -(1.5 + rnd_f(&mut weather.seed, 8.5));
        let speed = 5.0 + (i & 15) as f32;
        match weather.snow.eff[i].status {
            0 => {
                weather.snow.eff[i].wind_speed = speed;
                weather.snow.eff[i].gravity = gravity;
                weather.snow.eff[i].timer = 0;
                weather.snow.eff[i].base_pos = spawn;
                weather.snow.eff[i].pos = Vec3::new(
                    rnd_fx(&mut weather.seed, 550.0),
                    550.0,
                    rnd_fx(&mut weather.seed, 550.0),
                );
                weather.snow.eff[i].scale = 0.0;
                weather.snow.eff[i].pos_wave_x = rnd_f(&mut weather.seed, 360.0);
                weather.snow.eff[i].pos_wave_z = rnd_f(&mut weather.seed, 360.0);
                weather.snow.eff[i].status += 1;
            }
            1 | 2 | 3 => {
                let current = weather.snow.eff[i].wind_speed;
                let jitter = rnd_fx(&mut weather.seed, 0.1);
                add_calc(
                    &mut weather.snow.eff[i].wind_speed,
                    current - jitter,
                    0.5,
                    0.1,
                    0.01,
                );
                if weather.snow.eff[i].status != 3 || weather.snow.eff[i].scale < 0.01 {
                    let speed = weather.snow.eff[i].wind_speed;
                    weather.snow.eff[i].pos.x += wind_pow.x * speed;
                    weather.snow.eff[i].pos.z += wind_pow.z * speed;
                    weather.snow.eff[i].pos.y += weather.snow.eff[i].gravity + wind_pow.y * speed;
                    weather.snow.eff[i].pos.x += fsin(weather.snow.eff[i].pos_wave_x) * 1.5;
                    weather.snow.eff[i].pos.z += fsin(weather.snow.eff[i].pos_wave_z) * 1.5;
                    weather.snow.eff[i].pos_wave_x += 0.02;
                    weather.snow.eff[i].pos_wave_z += 0.02;
                }
                let probe = weather.snow.eff[i].base_pos + weather.snow.eff[i].pos;
                if weather.snow.eff[i].timer == 0 {
                    if probe.distance(spawn) > 550.0 {
                        weather.snow.eff[i].timer = 10;
                        weather.snow.eff[i].wind_speed = speed;
                        weather.snow.eff[i].gravity = gravity;
                        weather.snow.eff[i].base_pos = spawn;
                        if probe.distance(spawn) > 600.0 {
                            weather.snow.eff[i].pos = Vec3::new(
                                rnd_fx(&mut weather.seed, 550.0),
                                rnd_fx(&mut weather.seed, 550.0),
                                rnd_fx(&mut weather.seed, 550.0),
                            );
                        } else {
                            let push = rnd_fx(&mut weather.seed, 27.5);
                            let dir = vectle(probe, spawn);
                            weather.snow.eff[i].pos = dir * (push + 550.0);
                        }
                        weather.snow.eff[i].status = 1;
                    }
                } else {
                    weather.snow.eff[i].timer = weather.snow.eff[i].timer.wrapping_sub(1);
                }
            }
            _ => {}
        }

        let pos = weather.snow.eff[i].base_pos + weather.snow.eff[i].pos;
        let mut scale = if weather.snow.eff[i].status != 3 {
            let fade = (pos.distance(frame.eye) + 62.5).max(0.0);
            (1.0 - fade / 1125.0).clamp(0.0, 1.0)
        } else {
            0.0
        };
        if i as i32 > snow_count - 1 {
            scale = 0.0;
            add_calc(&mut weather.snow.eff[i].scale, scale, 0.2, 0.1, 0.01);
        } else {
            weather.snow.eff[i].scale = scale;
        }
        if i as i32 > snow_count - 1
            && weather.snow.eff[i].scale < 0.01
            && i == weather.snow.eff_count as usize - 1
        {
            weather.snow.eff_count = weather.snow.eff_count.wrapping_sub(1);
        }
        index -= 1;
    }
}

/// `dKyr_kazanbai_move` (`d_kankyo_rain.cpp:1431-1627`): the snow that piles on the ground of the one
/// stage that has it, using the same record the falling snow uses.
pub fn kazanbai_move(
    weather: &mut Weather,
    frame: &Frame,
    snow_count: i32,
    collision: Option<&Collision>,
) {
    if i32::from(weather.snow.eff_count) <= snow_count {
        weather.snow.eff_count = snow_count as i16;
    }
    if weather.snow.eff_count == 0 {
        return;
    }
    let ground = collision
        .and_then(|collision| {
            collision
                .ground_cross_below(Vec3::new(734.0, 3000.0, 491.0))
                .map(|hit| hit.height)
        })
        .unwrap_or(F_INF);
    let spawn = set_eyevect_calc2(frame.eye, frame.center, 800.0, 800.0);
    if weather.snow.old_eye_pos.distance(frame.eye) > 1500.0 {
        weather.snow.eff_count = 0;
    }
    weather.snow.old_eye_pos = frame.eye;
    snow_wind_factors(weather, frame);
    let wind_pow = weather.wind_vecpow();

    let mut settled = 0i32;
    let mut index = i32::from(weather.snow.eff_count) - 1;
    while index >= 0 {
        let i = index as usize;
        let gravity = -(2.5 + rnd_fx(&mut weather.seed, 2.0));
        let speed = 9.0 + rnd_f(&mut weather.seed, 60.0);
        match weather.snow.eff[i].status {
            0 => {
                weather.snow.eff[i].wind_speed = speed;
                weather.snow.eff[i].gravity = gravity;
                weather.snow.eff[i].timer = 0;
                let heading = snow_heading(frame);
                weather.snow.eff[i].base_pos = heading;
                weather.snow.eff[i].pos = Vec3::new(
                    rnd_fx(&mut weather.seed, 800.0),
                    800.0,
                    rnd_fx(&mut weather.seed, 800.0),
                );
                weather.snow.eff[i].scale = 0.0;
                weather.snow.eff[i].pos_wave_x = rnd_f(&mut weather.seed, 360.0);
                weather.snow.eff[i].pos_wave_z = rnd_f(&mut weather.seed, 360.0);
                weather.snow.eff[i].status += 1;
            }
            1 => {
                let current = weather.snow.eff[i].wind_speed;
                let jitter = rnd_fx(&mut weather.seed, 0.1);
                add_calc(
                    &mut weather.snow.eff[i].wind_speed,
                    current - jitter,
                    0.5,
                    0.1,
                    0.01,
                );
                let speed = weather.snow.eff[i].wind_speed;
                weather.snow.eff[i].pos.x += wind_pow.x * speed;
                weather.snow.eff[i].pos.z += wind_pow.z * speed;
                weather.snow.eff[i].pos.y += weather.snow.eff[i].gravity + wind_pow.y * speed;
                weather.snow.eff[i].pos.x += fsin(weather.snow.eff[i].pos_wave_x) * 2.5;
                weather.snow.eff[i].pos.z += fsin(weather.snow.eff[i].pos_wave_z) * 3.0;
                weather.snow.eff[i].pos.y += fsin(weather.snow.eff[i].pos_wave_z);
                weather.snow.eff[i].pos_wave_x += 0.03;
                weather.snow.eff[i].pos_wave_z += 0.03;

                let probe = weather.snow.eff[i].base_pos + weather.snow.eff[i].pos;
                if weather.snow.eff[i].timer == 0 && probe.distance(spawn) > 800.0 {
                    weather.snow.eff[i].timer = 10;
                    weather.snow.eff[i].wind_speed = speed;
                    weather.snow.eff[i].gravity = gravity;
                    weather.snow.eff[i].base_pos = spawn;
                    if probe.distance(spawn) > 950.0 {
                        weather.snow.eff[i].pos = Vec3::new(
                            rnd_fx(&mut weather.seed, 800.0),
                            rnd_fx(&mut weather.seed, 800.0),
                            rnd_fx(&mut weather.seed, 800.0),
                        );
                    } else {
                        let push = rnd_fx(&mut weather.seed, 40.0);
                        let dir = vectle(probe, spawn);
                        weather.snow.eff[i].pos = dir * (push + 800.0);
                    }
                    weather.snow.eff[i].status = 1;
                } else if weather.snow.eff[i].timer != 0 {
                    weather.snow.eff[i].timer = weather.snow.eff[i].timer.wrapping_sub(1);
                }

                if ground > 0.0 && probe.y < ground {
                    if settled < 100 && frame.counter0 & 7 == i as u32 & 7 {
                        weather.snow.eff[i].scale = 0.01;
                        let base = weather.snow.eff[i].base_pos;
                        weather.snow.eff[i].pos.y = ground - base.y;
                        weather.snow.eff[i].timer = (i & 15) as i16 + 20;
                        weather.snow.eff[i].status = 2;
                        settled += 1;
                    } else {
                        weather.snow.eff[i].status = 0;
                    }
                }
            }
            2 => {
                let speed = weather.snow.eff[i].wind_speed;
                weather.snow.eff[i].pos.x += wind_pow.x * (speed * 0.028);
                weather.snow.eff[i].pos.z += wind_pow.z * (speed * 0.028);
                let base = weather.snow.eff[i].base_pos;
                weather.snow.eff[i].pos.y =
                    (ground - base.y) - (1.0 - weather.snow.eff[i].scale) * 7.0;
                if weather.snow.eff[i].timer != 0 {
                    weather.snow.eff[i].timer = weather.snow.eff[i].timer.wrapping_sub(1);
                }
                if weather.snow.eff[i].scale <= 0.001 {
                    weather.snow.eff[i].status = 0;
                }
                settled += 1;
            }
            _ => {}
        }

        let pos = weather.snow.eff[i].base_pos + weather.snow.eff[i].pos;
        let fade = (pos.distance(frame.eye) + 30.0).max(0.0);
        let mut scale = (1.0 - fade / 800.0).clamp(0.0, 1.0);
        if i as i32 >= snow_count - 1 || weather.snow.eff[i].status == 2 {
            if weather.snow.eff[i].status == 2 {
                if weather.snow.eff[i].timer != 0 {
                    add_calc(&mut weather.snow.eff[i].scale, 1.0, 0.5, 0.1, 0.00001);
                } else {
                    add_calc(&mut weather.snow.eff[i].scale, 0.0, 0.1, 0.03, 0.00001);
                }
            } else {
                scale = 0.0;
                add_calc(&mut weather.snow.eff[i].scale, scale, 0.2, 0.1, 0.01);
            }
        } else {
            weather.snow.eff[i].scale = scale;
        }
        if i as i32 >= snow_count - 1
            && weather.snow.eff[i].scale < 0.01
            && i == weather.snow.eff_count as usize - 1
        {
            weather.snow.eff_count = weather.snow.eff_count.wrapping_sub(1);
        }
        index -= 1;
    }
    kazanbai_tamari_move(weather, snow_count, collision);
}

/// The normalised camera heading `dKyr_kazanbai_move` rebases a fresh flake on
/// (`d_kankyo_rain.cpp:1474-1497`). It is a direction, not a position, and the reference stores it
/// as the base position anyway.
fn snow_heading(frame: &Frame) -> Vec3 {
    vectle_calc(Vec3::new(
        frame.center.x - frame.eye.x,
        0.0,
        frame.center.z - frame.eye.z,
    ))
}

/// `dKyr_kazanbai_tamari_move` (`d_kankyo_rain.cpp:1630-1712`): the banked layer that sits in the
/// second half of the array and fades in and out on its own timer.
pub fn kazanbai_tamari_move(weather: &mut Weather, snow_count: i32, collision: Option<&Collision>) {
    if snow_count == 0 {
        return;
    }
    weather.snow.tamari_start = 50;
    let ground = collision
        .and_then(|collision| {
            collision
                .ground_cross_below(Vec3::new(734.0, 3000.0, 491.0))
                .map(|hit| hit.height)
        })
        .unwrap_or(F_INF);
    let (base, spread) = if ground > 0.0 {
        (Vec3::new(-367.0, 0.0, 368.0), 1000.0f32)
    } else {
        (Vec3::new(109.0, 0.0, 402.0), 450.0)
    };
    let wind_pow = weather.wind_vecpow();

    let mut index = i32::from(weather.snow.tamari_start) + 200;
    while index >= 200 {
        let i = index as usize;
        let gravity = -(1.5 + rnd_fx(&mut weather.seed, 3.0));
        let speed = 9.0 + rnd_f(&mut weather.seed, 60.0);
        match weather.snow.eff[i].status {
            0 => {
                weather.snow.eff[i].wind_speed = speed;
                weather.snow.eff[i].gravity = gravity;
                weather.snow.eff[i].base_pos = Vec3::new(base.x, 0.0, base.z);
                weather.snow.eff[i].pos = Vec3::new(
                    rnd_fx(&mut weather.seed, spread),
                    ground,
                    rnd_fx(&mut weather.seed, spread),
                );
                weather.snow.eff[i].scale = 0.0;
                weather.snow.eff[i].pos_wave_x = rnd_f(&mut weather.seed, 360.0);
                weather.snow.eff[i].pos_wave_z = rnd_f(&mut weather.seed, 360.0);
                weather.snow.eff[i].timer = (rnd_f(&mut weather.seed, 20.0) + 20.0) as i16;
                weather.snow.eff[i].status += 1;
            }
            1 => {
                if weather.snow.eff[i].timer != 0 {
                    weather.snow.eff[i].timer = weather.snow.eff[i].timer.wrapping_sub(1);
                }
                let drift = weather.snow.eff[i].wind_speed * weather.snow.eff[i].scale * 0.02;
                weather.snow.eff[i].pos.x += wind_pow.x * drift;
                weather.snow.eff[i].pos.z += wind_pow.z * drift;
                weather.snow.eff[i].pos.y =
                    (ground - 4.0) - (1.0 - weather.snow.eff[i].scale) * 5.0;
                if weather.snow.eff[i].timer <= 0 && weather.snow.eff[i].scale < 0.0001 {
                    weather.snow.eff[i].status = 0;
                }
                if weather.snow.eff[i].timer != 0 {
                    add_calc(&mut weather.snow.eff[i].scale, 1.0, 0.5, 0.1, 0.0001);
                } else {
                    add_calc(&mut weather.snow.eff[i].scale, 0.0, 0.1, 0.02, 0.0001);
                }
            }
            _ => {}
        }
        index -= 1;
    }
}

/// `dKyr_housi_move` (`d_kankyo_rain.cpp:1091-1254`): the steam that boils off hot ground. The other
/// sibling's version of this same-named function diverges — it carries a fourth non-init stage, a
/// dark-world check that overrides the wind outright, a fishing-hole branch and a camera-distance
/// fade on both the alpha and `field_0x48`, none of which is in this game — so it was not used here
/// and every constant is this file's own.
pub fn housi_move(weather: &mut Weather, frame: &Frame, housi_count: i32) {
    if housi_count != 0 || weather.housi.m5ddc <= 0.0 {
        weather.housi.count = housi_count as i16;
    }
    if housi_count != 0 {
        add_calc(&mut weather.housi.m5ddc, 1.0, 0.2, 0.05, 0.01);
    } else {
        add_calc(&mut weather.housi.m5ddc, 0.0, 0.2, 0.05, 0.01);
    }
    if weather.housi.count == 0 {
        return;
    }
    let spawn = set_eyevect_calc2(frame.eye, frame.center, 800.0, 800.0);
    let wind_pow = weather.wind_vecpow();

    let mut index = i32::from(weather.housi.count) - 1;
    while index >= 0 {
        let i = index as usize;
        let mut target_alpha = weather.housi.m5ddc;
        match weather.housi.effect[i].status {
            0 => {
                weather.housi.effect[i].m34 = rnd_f(&mut weather.seed, 1.5) + 0.2;
                weather.housi.effect[i].m3c = 0;
                weather.housi.effect[i].m4c = rnd_fx_u16(&mut weather.seed, 65536.0);
                weather.housi.effect[i].base_pos = spawn;
                weather.housi.effect[i].pos = Vec3::new(
                    rnd_fx(&mut weather.seed, 1000.0),
                    1000.0,
                    rnd_fx(&mut weather.seed, 1000.0),
                );
                weather.housi.effect[i].alpha = 0.0;
                weather.housi.effect[i].m48 = 0.0;
                weather.housi.effect[i].scale = Vec3::new(
                    rnd_f(&mut weather.seed, 360.0),
                    rnd_f(&mut weather.seed, 360.0),
                    rnd_f(&mut weather.seed, 360.0),
                );
                weather.housi.effect[i].speed = Vec3::ZERO;
                let base_y = weather.housi.effect[i].base_pos.y;
                if base_y + weather.housi.effect[i].pos.y < -100149.9 {
                    weather.housi.effect[i].pos.y = (-99999.9 - base_y) + 10.0;
                }
                weather.housi.effect[i].status += 1;
            }
            1 | 2 | 3 => {
                if weather.housi.effect[i].status == 1 {
                    let rate = weather.housi.effect[i].m34;
                    let wave = fsin(weather.housi.effect[i].scale.x);
                    weather.housi.effect[i].pos.x += rate * (wind_pow.x * 5.0);
                    weather.housi.effect[i].pos.y += rate * (wind_pow.y * 5.0);
                    weather.housi.effect[i].pos.z += rate * (wind_pow.z * 5.0);
                    weather.housi.effect[i].pos.y -= rate * 0.6;
                    weather.housi.effect[i].pos.x += wave * rate;
                    weather.housi.effect[i].pos.y +=
                        rate * 0.5 * fsin(weather.housi.effect[i].scale.y);
                    weather.housi.effect[i].pos.z += fsin(weather.housi.effect[i].scale.z) * rate;
                    weather.housi.effect[i].scale.x += 0.03;
                    weather.housi.effect[i].scale.y += 0.02;
                    weather.housi.effect[i].scale.z += 0.01;
                }
                let probe = weather.housi.effect[i].base_pos + weather.housi.effect[i].pos;
                let gust = pntwind_vecpow(weather, probe);
                if weather.housi.effect[i].speed.x < 30.0 {
                    weather.housi.effect[i].speed.x += gust.x * 9.0;
                }
                if weather.housi.effect[i].speed.y < 30.0 {
                    weather.housi.effect[i].speed.y += gust.y * 9.0;
                }
                if weather.housi.effect[i].speed.z < 30.0 {
                    weather.housi.effect[i].speed.z += gust.z * 9.0;
                }
                add_calc(&mut weather.housi.effect[i].speed.x, 0.0, 0.2, 0.1, 0.00001);
                add_calc(&mut weather.housi.effect[i].speed.y, 0.0, 0.2, 0.1, 0.00001);
                add_calc(&mut weather.housi.effect[i].speed.z, 0.0, 0.2, 0.1, 0.00001);
                let speed = weather.housi.effect[i].speed;
                weather.housi.effect[i].pos += speed;

                let probe = weather.housi.effect[i].base_pos + weather.housi.effect[i].pos;
                let dist = probe.distance(spawn);
                if weather.housi.effect[i].m3c == 0 {
                    if dist > 1000.0 || probe.y < -99979.9 {
                        weather.housi.effect[i].m3c = 10;
                        weather.housi.effect[i].base_pos = spawn;
                        if probe.distance(spawn) > 1050.0 {
                            weather.housi.effect[i].pos = Vec3::new(
                                rnd_fx(&mut weather.seed, 1000.0),
                                rnd_fx(&mut weather.seed, 1000.0),
                                rnd_fx(&mut weather.seed, 1000.0),
                            );
                        } else {
                            let push = rnd_fx(&mut weather.seed, 50.0);
                            let dir = vectle(probe, spawn);
                            weather.housi.effect[i].pos = dir * (push + 1000.0);
                        }
                        weather.housi.effect[i].speed = Vec3::ZERO;
                        weather.housi.effect[i].status = 1;
                    }
                } else {
                    weather.housi.effect[i].m3c = weather.housi.effect[i].m3c.wrapping_sub(1);
                }
            }
            _ => {}
        }
        weather.housi.effect[i].m4c = weather.housi.effect[i].m4c.wrapping_add(600);
        if weather.housi.effect[i].m4c > 30000 {
            target_alpha = 0.0;
        }
        add_calc(
            &mut weather.housi.effect[i].alpha,
            target_alpha,
            0.5,
            0.02,
            0.00001,
        );
        index -= 1;
    }
}

/// `dKyr_star_init` (`d_kankyo_rain.cpp:1715-1722`).
pub fn star_init(weather: &mut Weather) {
    weather.star.effect[0].sin = 1.0;
    weather.star.effect_num = 0;
}

/// `dKyr_star_move` (`d_kankyo_rain.cpp:1725-1734`).
pub fn star_move(weather: &mut Weather, star_count: i16) {
    weather.star.effect_num = star_count;
    if weather.star.effect_num != 0 {
        let wave = fsin(weather.star.effect[0].anim_counter).abs();
        weather.star.effect[0].anim_counter += 0.01;
        weather.star.effect[0].sin = wave;
        add_calc(&mut weather.star.effect[0].sin, wave, 0.5, 0.1, 0.01);
    }
}

/// `wether_move_star` (`d_kankyo_wether.cpp:596-661`): how many stars the sky holds, from the clock
/// and the colour pattern, and the packet's whole life around that count.
pub fn wether_move_star(weather: &mut Weather, frame: &Frame, colpat_weather: u8) {
    if !(frame.status_1 && !weather.vrbox_invisible && frame.stage != "M_DragB")
        && frame.stage != "Name"
    {
        return;
    }
    let mut target = if frame.stage == "Name" {
        if weather.time >= 315.0 || weather.time < 60.0 {
            1.0
        } else if weather.time > 270.0 {
            1.0 - (315.0 - weather.time) * (1.0 / 45.0)
        } else if weather.time < 75.0 {
            (75.0 - weather.time) * (1.0 / 15.0)
        } else {
            0.0
        }
    } else if weather.time >= 330.0 || weather.time < 90.0 {
        1.0
    } else if weather.time > 300.0 {
        1.0 - (330.0 - weather.time) * (1.0 / 30.0)
    } else if weather.time < 105.0 {
        (105.0 - weather.time) * (1.0 / 15.0)
    } else {
        0.0
    };
    if colpat_weather != 0 {
        target = 0.0;
    }
    if weather.colpat_curr != 0 && weather.colpat_blend > 0.5 {
        target = 0.0;
    }
    add_calc(&mut weather.star_count_target, target, 0.1, 0.01, 0.000001);
    weather.star_count = (weather.star_count_target * 1000.0) as i16;

    if !weather.star_initialized {
        if weather.star_count != 0 {
            star_init(weather);
            star_move(weather, weather.star_count);
            weather.star_initialized = true;
        }
    } else {
        star_move(weather, weather.star_count);
        if weather.star_count == 0 {
            weather.star_initialized = false;
        }
    }
}

/// `dKyr_poison_live_check` (`d_kankyo_rain.cpp:2178-2183`).
pub fn poison_live_check(weather: &Weather) -> bool {
    weather.poison_count != 0
}

/// `poison_init` (`d_kankyo_rain.cpp:2204-2217`).
pub fn poison_init(weather: &mut Weather, room: i32) {
    weather.poison.mbb9c = Vec3::ZERO;
    weather.poison.count = 0;
    for eff in &mut weather.poison.eff {
        eff.status = 0;
    }
    weather.poison_room = room;
}

/// `poison_move` (`d_kankyo_rain.cpp:2221-2751`). The room picks the pattern, the pattern is rebased
/// on the emitter's centre, and each emitter rises, drifts, is turned back at the pattern's walls
/// and is faded by how close the camera is to it.
pub fn poison_move(weather: &mut Weather, frame: &Frame) {
    weather.poison_touching = false;
    let room = frame.room;
    if weather.poison_room != room {
        weather.poison_room = room;
        for eff in &mut weather.poison.eff {
            eff.status = 0;
        }
    }
    let pattern = if room >= 0 && room < 16 {
        POISON_ROOM_PATTERN[room as usize]
    } else {
        -1
    };
    let row = if pattern >= 0 {
        POISON_PATTERNS[pattern as usize]
    } else {
        POISON_PATTERNS[1]
    };
    weather.poison_count = if pattern >= 0 { row.count } else { 0 };
    let spread = Vec3::new(row.xc.x - row.rise, row.xc.y, row.xc.z - row.rise);
    weather.poison.base_pos = Vec3::new(row.pos.x, row.pos.y + row.rise, row.pos.z);

    let wind_pow = weather.wind_vecpow();
    let cam_forward = vectle(frame.eye, frame.center);
    let mut nearest = F_INF;
    let mut index = 0i32;
    while index < weather.poison_count {
        let i = index as usize;
        let mut ceiling = spread.y;
        match weather.poison.eff[i].status {
            0 => {
                weather.poison.eff[i].size = 0.0;
                weather.poison.eff[i].pos = if pattern == 2 {
                    let x = rnd_fx(&mut weather.seed, 32767.0);
                    let z = rnd_fx(&mut weather.seed, 32767.0);
                    Vec3::new(
                        spread.x * ssin(x as i16),
                        rnd_f(&mut weather.seed, spread.x),
                        spread.x * scos(z as i16),
                    )
                } else {
                    Vec3::new(
                        rnd_fx(&mut weather.seed, spread.x),
                        rnd_f(&mut weather.seed, spread.y),
                        rnd_fx(&mut weather.seed, spread.z),
                    )
                };
                weather.poison.eff[i].m10 = 0.0;
                weather.poison.eff[i].m14 = 0.0;
                weather.poison.eff[i].m18 = 0.0;
                weather.poison.eff[i].m1c = rnd_f(&mut weather.seed, 360.0);
                weather.poison.eff[i].m20 = rnd_f(&mut weather.seed, 1.0);
                weather.poison.eff[i].m2c = rnd_f(&mut weather.seed, 65536.0) as i16;
                weather.poison.eff[i].alpha = 0.0;
                weather.poison.eff[i].m2e = 0;
                weather.poison.eff[i].status += 1;
            }
            1 | 2 => {
                let (puff, rise_speed) = if weather.poison.eff[i].status == 1 {
                    (1.5, row.rise_speed)
                } else {
                    (10.0, 5.0)
                };
                let jitter = weather.poison.eff[i].m20;
                ceiling = spread.y * (1.0 + 0.8 * jitter);
                weather.poison.eff[i].m2c = weather.poison.eff[i].m2c.wrapping_add(400);

                let mut travel = Vec3::new(wind_pow.x * 10.0, 0.0, wind_pow.z * 10.0);
                travel.x += (travel.x * 0.5) * jitter;
                travel.z += (travel.z * 0.5) * jitter;
                travel.y = rise_speed * (0.5 + jitter);
                travel.y += (travel.y * 0.5) * jitter;

                let probe = weather.poison.base_pos + weather.poison.eff[i].pos;
                let mut sample = probe;
                sample.y += 200.0;
                let gust = pntwind_vecpow(weather, sample);
                weather.poison.eff[i].m10 += gust.x * 39.0;
                weather.poison.eff[i].m14 += gust.y * 39.0;
                if weather.poison.eff[i].m14 < 0.0 {
                    weather.poison.eff[i].m14 = 0.0;
                }
                weather.poison.eff[i].m18 += gust.z * 39.0;
                if weather.poison.eff[i].m10 > 43.0
                    || weather.poison.eff[i].m14 > 43.0
                    || weather.poison.eff[i].m18 > 43.0
                {
                    weather.poison.eff[i].m2e = 180;
                }
                add_calc(&mut weather.poison.eff[i].m10, 0.0, 0.2, 0.6, 0.001);
                add_calc(&mut weather.poison.eff[i].m14, 0.0, 0.2, 0.6, 0.001);
                add_calc(&mut weather.poison.eff[i].m18, 0.0, 0.2, 0.6, 0.001);
                let nudge = Vec3::new(
                    weather.poison.eff[i].m10,
                    weather.poison.eff[i].m14,
                    weather.poison.eff[i].m18,
                );
                travel += nudge;
                weather.poison.eff[i].pos += travel;

                let drift = Vec3::new(
                    5.0 * (row.drift * (scos(row.yaw) * ssin(row.pitch))),
                    5.0 * (row.drift * ssin(row.yaw)),
                    5.0 * (row.drift * (scos(row.yaw) * scos(row.pitch))),
                );
                weather.poison.eff[i].pos += drift;
                let wobble = jitter * puff;
                weather.poison.eff[i].pos.x += wobble * fsin(jitter * 65535.0);
                weather.poison.eff[i].pos.z += wobble * fcos(jitter * 65535.0);

                if weather.poison.eff[i].m2e == 0 {
                    let mut bounced = false;
                    match pattern {
                        2 => {
                            let base = weather.poison.base_pos;
                            let flat = Vec3::new(
                                base.x + weather.poison.eff[i].pos.x,
                                base.y,
                                base.z + weather.poison.eff[i].pos.z,
                            );
                            if flat.distance(base) >= spread.x {
                                bounced = true;
                                weather.poison.eff[i].pos.x *= -0.99;
                                weather.poison.eff[i].pos.z *= -0.99;
                            }
                        }
                        4 => {
                            let base = weather.poison.base_pos;
                            let world = base + weather.poison.eff[i].pos;
                            let pos = weather.poison.eff[i].pos;
                            if pos.x > spread.x {
                                bounced = true;
                                weather.poison.eff[i].pos.x = -spread.x;
                            } else if pos.x < -spread.x {
                                bounced = true;
                                weather.poison.eff[i].pos.x = spread.x;
                            }
                            if world.x > 7200.0 {
                                if world.z < 2300.0 + row.rise {
                                    weather.poison.eff[i].pos.z = (2300.0 + row.rise) - base.z;
                                    bounced = true;
                                } else if world.z > 3200.0 - row.rise {
                                    weather.poison.eff[i].pos.z = (3200.0 - row.rise) - base.z;
                                    bounced = true;
                                }
                            } else if world.x > 6400.0 {
                                if world.z < 2400.0 + row.rise {
                                    weather.poison.eff[i].pos.z = (2400.0 + row.rise) - base.z;
                                    bounced = true;
                                } else if world.z > 3200.0 - row.rise {
                                    weather.poison.eff[i].pos.z = (3200.0 - row.rise) - base.z;
                                    bounced = true;
                                }
                            } else if world.x < 5550.0 && world.x > 4646.0 {
                                if world.z < 2100.0 + row.rise {
                                    weather.poison.eff[i].pos.z = (2100.0 + row.rise) - base.z;
                                    bounced = true;
                                } else if world.z > 3500.0 - row.rise {
                                    weather.poison.eff[i].pos.z = (3500.0 - row.rise) - base.z;
                                    bounced = true;
                                }
                            } else if pos.z > spread.z {
                                weather.poison.eff[i].pos.z = spread.z;
                                bounced = true;
                            } else if pos.z < -spread.z {
                                weather.poison.eff[i].pos.z = -spread.z;
                                bounced = true;
                            }
                            if bounced {
                                weather.poison.eff[i].m20 = rnd_f(&mut weather.seed, 1.0);
                            }
                        }
                        3 => {
                            let base = weather.poison.base_pos;
                            let world = base + weather.poison.eff[i].pos;
                            let pos = weather.poison.eff[i].pos;
                            if pos.z > spread.z {
                                weather.poison.eff[i].pos.z = -spread.z;
                                bounced = true;
                            } else if pos.z < -spread.z {
                                weather.poison.eff[i].pos.z = spread.z;
                                bounced = true;
                            }
                            if world.z > 4600.0 {
                                if world.x < 1060.0 + row.rise {
                                    weather.poison.eff[i].pos.x = (1060.0 + row.rise) - base.x;
                                    bounced = true;
                                } else if world.x > 2730.0 - row.rise {
                                    weather.poison.eff[i].pos.x = (2730.0 - row.rise) - base.x;
                                    bounced = true;
                                }
                            } else if pos.x > spread.x {
                                weather.poison.eff[i].pos.x = spread.x;
                                bounced = true;
                            } else if pos.x < -spread.x {
                                weather.poison.eff[i].pos.x = -spread.x;
                                bounced = true;
                            }
                            if bounced {
                                weather.poison.eff[i].m20 = rnd_f(&mut weather.seed, 1.0);
                            }
                        }
                        _ => {
                            let pos = weather.poison.eff[i].pos;
                            if pos.x > spread.x {
                                weather.poison.eff[i].pos.x = -spread.x;
                                bounced = true;
                            } else if pos.x < -spread.x {
                                weather.poison.eff[i].pos.x = spread.x;
                                bounced = true;
                            }
                            if pos.z > spread.z {
                                weather.poison.eff[i].pos.z = -spread.z;
                                bounced = true;
                            } else if pos.z < -spread.z {
                                weather.poison.eff[i].pos.x = spread.z;
                                bounced = true;
                            }
                        }
                    }
                    if weather.poison.eff[i].pos.y > ceiling && weather.poison.eff[i].alpha <= 0.001
                    {
                        weather.poison.eff[i].pos.y = 0.0;
                    } else if weather.poison.eff[i].pos.y < 0.0
                        && weather.poison.eff[i].alpha <= 0.001
                    {
                        weather.poison.eff[i].pos.y = ceiling;
                    }
                    if bounced {
                        weather.poison.eff[i].alpha = 0.0;
                        weather.poison.eff[i].m2e = 120;
                    }
                }
            }
            _ => {}
        }

        let base = weather.poison.base_pos;
        let pos = base + weather.poison.eff[i].pos;
        let focus = if pattern == 0 {
            frame.eye + cam_forward * 2000.0
        } else {
            frame.eye
        };
        let dist = pos.distance(focus);
        if dist < nearest {
            nearest = dist;
            weather.poison.mbb9c = pos;
        }
        let reach = (dist / (spread.x + spread.z)).min(1.0);
        let pulse = fsin(weather.poison.eff[i].m1c);
        weather.poison.eff[i].m1c += 0.05;
        weather.poison.eff[i].size = row.rise + row.rise * reach + (0.2 * row.rise) * pulse;

        let height = if weather.poison.eff[i].pos.y > ceiling * 0.5 {
            (ceiling - weather.poison.eff[i].pos.y) / (ceiling * 0.5)
        } else if weather.poison.eff[i].pos.y < ceiling * 0.1 {
            weather.poison.eff[i].pos.y / (ceiling * 0.1)
        } else {
            1.0
        };

        let edge = if pattern == 2 {
            let flat = Vec3::new(pos.x, base.y, pos.z);
            let mut power = (flat.distance(base) / spread.x).min(1.0);
            power *= power;
            power *= power;
            power *= power;
            1.0 - power
        } else {
            let pos = weather.poison.eff[i].pos;
            let mut fade = 1.0f32;
            if pos.x.abs() > spread.x - 100.0 {
                let width = spread.x - (spread.x - 100.0);
                fade = 0.0;
                if width > fade {
                    fade = (spread.x - pos.x.abs()) / width;
                }
            }
            if pos.z.abs() > spread.z - 100.0 {
                let width = spread.z - (spread.z - 100.0);
                if width > 0.0 {
                    fade *= (spread.z - pos.z.abs()) / width;
                } else {
                    fade = 0.0;
                }
            }
            fade
        };
        weather.poison.eff[i].size *= edge;

        let mut show = 1.0 - square(1.0 - height);
        if weather.poison.eff[i].m2e != 0 {
            weather.poison.eff[i].m2e = weather.poison.eff[i].m2e.wrapping_sub(1);
            show = 0.0;
        } else if weather.poison.eff[i].status == 2 {
            weather.poison.eff[i].status = 0;
        }
        if weather.contrast_flag {
            show = 0.0;
        }
        add_calc(
            &mut weather.poison.eff[i].alpha,
            edge * (0.2 * show),
            0.05,
            0.01,
            0.001,
        );

        let near_radius = if pattern == 3 { 450.0 } else { 200.0 };
        let dist = pos.distance(frame.eye);
        if dist < near_radius {
            let falloff = (dist / 450.0).min(1.0);
            weather.poison.eff[i].alpha *= falloff * falloff * falloff;
        }
        if weather.poison.eff[i].alpha > 0.0001 && weather.poison.eff[i].m2e == 0 {
            let mut probe = pos;
            probe.y -= 50.0;
            if probe.distance(frame.player_pos) < weather.poison.eff[i].size * 0.8 {
                weather.poison_touching = true;
            }
        }
        index += 1;
    }
    weather.poison.count += 1;
}

/// `dKyr_poison_light_colision` (`d_kankyo_rain.cpp:2186-2201`): an emitter that has reached a lit
/// object is put out. `is_light` answers the collision system's mass-type query, the one call here
/// the port has no equivalent for.
pub fn poison_light_collision(weather: &mut Weather, is_light: impl Fn(Vec3) -> bool) {
    if !poison_live_check(weather) {
        return;
    }
    for index in 0..weather.poison_count as usize {
        let mut probe = weather.poison.base_pos + weather.poison.eff[index].pos;
        probe.y -= 70.0;
        if is_light(probe) && weather.poison.eff[index].status == 1 {
            weather.poison.eff[index].status = 2;
            weather.poison.eff[index].m2e = 60;
        }
    }
}

/// `cloud_shadow_move` (`d_kankyo_rain.cpp:1984-2164`): the ground-shadow cloud layer, which the
/// stage's own mode steers.
pub fn cloud_shadow_move(weather: &mut Weather, frame: &Frame, moya_mode: u8, moya_count: i32) {
    if !frame.has_view {
        return;
    }
    if weather.cloud.count <= moya_count {
        weather.cloud.count = moya_count;
    }
    if weather.cloud.count == 0 {
        return;
    }
    let range = 2000.0f32;
    let center = set_eyevect_calc2(frame.eye, frame.center, 1500.0, 1500.0);
    let wind_pow = weather.wind_vecpow();

    let mut index = 0i32;
    while index < weather.cloud.count {
        let i = index as usize;
        match weather.cloud.eff[i].status {
            0 => {
                weather.cloud.eff[i].initial_size = 400.0 + rnd_fx(&mut weather.seed, 80.0);
                weather.cloud.eff[i].size = weather.cloud.eff[i].initial_size;
                weather.cloud.eff[i].base_pos = center;
                weather.cloud.eff[i].pos = Vec3::new(
                    rnd_fx(&mut weather.seed, range),
                    rnd_fx(&mut weather.seed, range),
                    rnd_fx(&mut weather.seed, range),
                );
                weather.cloud.eff[i].wind_speed = 0.7 + rnd_f(&mut weather.seed, 0.3);
                weather.cloud.eff[i].alpha = 0.0;
                weather.cloud.eff[i].anim_counter = rnd_f(&mut weather.seed, 65535.0) as u16;
                weather.cloud.eff[i].vel_rndm = Vec3::new(
                    rnd_f(&mut weather.seed, 360.0),
                    rnd_f(&mut weather.seed, 360.0),
                    rnd_f(&mut weather.seed, 360.0),
                );
                weather.cloud.eff[i].pnt_wind_vel = Vec3::ZERO;
                weather.cloud.eff[i].status += 1;
            }
            1 | 2 => {
                let base = weather.cloud.eff[i].base_pos;
                let pos = base + weather.cloud.eff[i].pos;
                weather.cloud.eff[i].anim_counter =
                    weather.cloud.eff[i].anim_counter.wrapping_add(120);
                chase_f(
                    &mut weather.cloud.eff[i].m34,
                    weather.cloud.eff[i].wind_speed,
                    7.0,
                );
                if moya_mode & 1 != 0 {
                    if frame.stage != "M_DragB" || weather.vrbox_invisible {
                        if moya_mode == 1 {
                            weather.cloud.eff[i].pos.y += 20.0;
                        }
                    } else {
                        weather.cloud.eff[i].pos.y += 5.0;
                    }
                }
                let speed = if moya_mode == 1 { 10.0 } else { 40.0 };
                let drift = wind_pow * speed * weather.cloud.eff[i].wind_speed;
                weather.cloud.eff[i].pos += drift;
                let gust = pntwind_vecpow(weather, pos);
                if weather.cloud.eff[i].pnt_wind_vel.x < 30.0 {
                    weather.cloud.eff[i].pnt_wind_vel.x += gust.x * 9.0;
                }
                if weather.cloud.eff[i].pnt_wind_vel.y < 30.0 {
                    weather.cloud.eff[i].pnt_wind_vel.y += gust.y * 9.0;
                }
                if weather.cloud.eff[i].pnt_wind_vel.z < 30.0 {
                    weather.cloud.eff[i].pnt_wind_vel.z += gust.z * 9.0;
                }
                add_calc(
                    &mut weather.cloud.eff[i].pnt_wind_vel.x,
                    0.0,
                    0.2,
                    0.1,
                    0.00001,
                );
                add_calc(
                    &mut weather.cloud.eff[i].pnt_wind_vel.y,
                    0.0,
                    0.2,
                    0.1,
                    0.00001,
                );
                add_calc(
                    &mut weather.cloud.eff[i].pnt_wind_vel.z,
                    0.0,
                    0.2,
                    0.1,
                    0.00001,
                );
                let push = weather.cloud.eff[i].pnt_wind_vel;
                weather.cloud.eff[i].pos += push;

                let jitter = if frame.stage == "Adanmae" { 20.0 } else { 8.0 };
                weather.cloud.eff[i].pos += Vec3::new(
                    (weather.cloud.eff[i].vel_rndm.x / 360.0 - 0.5) * jitter,
                    (weather.cloud.eff[i].vel_rndm.y / 360.0 - 0.5) * jitter,
                    (weather.cloud.eff[i].vel_rndm.z / 360.0 - 0.5) * jitter,
                );

                let base = weather.cloud.eff[i].base_pos;
                let pos = base + weather.cloud.eff[i].pos;
                if pos.distance(center) > range {
                    weather.cloud.eff[i].base_pos = center;
                    if pos.distance(center) > range + 0.1 * range {
                        weather.cloud.eff[i].pos = Vec3::new(
                            rnd_fx(&mut weather.seed, range),
                            rnd_fx(&mut weather.seed, range),
                            rnd_fx(&mut weather.seed, range),
                        );
                    } else {
                        let mut dir = vectle(pos, center);
                        add_calc(&mut weather.cloud.eff[i].alpha, 0.0, 0.1, 0.01, 0.001);
                        if weather.cloud.eff[i].alpha < 0.001 {
                            dir.x += rnd_f(&mut weather.seed, 0.5);
                            dir.y += rnd_f(&mut weather.seed, 0.5);
                            dir.z += rnd_f(&mut weather.seed, 0.5);
                            weather.cloud.eff[i].pos = dir * range;
                        }
                    }
                    weather.cloud.eff[i].alpha = 0.0;
                }
            }
            3 => weather.cloud.eff[i].status = 0,
            _ => {}
        }

        let base = weather.cloud.eff[i].base_pos;
        let pos = base + weather.cloud.eff[i].pos;
        let growth = ((pos.distance(frame.eye) - 1000.0) / range) * 1.5;
        weather.cloud.eff[i].size = growth
            + 40.0 * ssin(weather.cloud.eff[i].anim_counter as i16)
            + weather.cloud.eff[i].initial_size * growth;

        let dist_center = pos.distance(center).max(0.0);
        let mut target = 1.0 - dist_center / 1000.0;
        let mut max_alpha = match moya_mode {
            3 => 0.06,
            4 => 0.05,
            _ => 0.035,
        };
        max_alpha *= 1.2;
        if index > moya_count - 1 {
            target = 0.0;
            if weather.cloud.eff[i].alpha < 0.001 && index == weather.cloud.count - 1 {
                weather.cloud.count -= 1;
            }
        }
        add_calc(
            &mut weather.cloud.eff[i].alpha,
            target * max_alpha,
            0.1,
            0.1,
            0.000000001,
        );
        index += 1;
    }
}

/// `vrkumo_move` (`d_kankyo_rain.cpp:2755-2947`): the dome cloud ring that closes in on the horizon.
pub fn vrkumo_move(weather: &mut Weather, frame: &Frame) {
    let strength = weather.vrkumo_strength;
    let mut wind_pow = weather.wind_vecpow();
    let speed = 4.0 + strength * (8.3 - 4.0);
    let mut radius = 1000.0 + strength * (500.0 - 1000.0);
    let height = 3000.0 + strength * (2000.0 - 3000.0);
    let inner = 0.98 + strength * (0.98 - 0.98);
    let ceiling = 1.0 + strength * (0.45 - 1.0);

    if frame.stage == "M_DragB" {
        wind_pow = Vec3::new(-1.0, 0.0, 0.0);
        radius = 300.0;
    } else if frame.stage_type == STAGE_TYPE_MISC {
        let angle_y = weather
            .wind
            .tact_wind_angle_y
            .wrapping_add(stage_misc_wind(frame.stage));
        let normal = Vec3::new(
            scos(weather.wind.tact_wind_angle_x) * scos(angle_y),
            ssin(weather.wind.tact_wind_angle_x),
            scos(weather.wind.tact_wind_angle_x) * ssin(angle_y),
        );
        wind_pow = normal * 0.6;
    }
    if frame.has_view {
        let sea = if frame.stage == "Siren" && frame.room == 17 {
            -14101.0
        } else {
            frame.sea_level
        };
        radius -= 0.09 * (frame.eye.y - sea);
    }

    for index in 0..VRKUMO_MAX {
        match weather.vrkumo.inst[index].status {
            0 => {
                let bearing = rnd_f_u16(&mut weather.seed, 65535.0);
                let mut distance = rnd_f(&mut weather.seed, 18000.0);
                if distance > 15000.0 {
                    distance = 14000.0 + rnd_f(&mut weather.seed, 1000.0);
                }
                weather.vrkumo.inst[index].position = Vec3::new(
                    distance * fsin(f32::from(bearing)),
                    0.0,
                    distance * fcos(f32::from(bearing)),
                );
                weather.vrkumo.inst[index].height = 0.3 * rnd_fx(&mut weather.seed, 0.3);
                weather.vrkumo.inst[index].alpha = 0.0;
                weather.vrkumo.inst[index].speed = 0.5 + rnd_f(&mut weather.seed, 4.0);
                weather.vrkumo.inst[index].status += 1;
            }
            1 => {
                let mut flat = weather.vrkumo.inst[index].position;
                flat.y = 0.0;
                if flat.length() > 15000.0 {
                    if flat.length() > 15100.0 {
                        weather.vrkumo.inst[index].position.x = rnd_fx(&mut weather.seed, 14000.0);
                        weather.vrkumo.inst[index].position.z = rnd_fx(&mut weather.seed, 14000.0);
                    } else {
                        weather.vrkumo.inst[index].position.x *= -1.0;
                        weather.vrkumo.inst[index].position.z *= -1.0;
                    }
                    weather.vrkumo.inst[index].alpha = 0.0;
                }
                if weather.vrkumo.inst[index].alpha <= 0.0 {
                    let step = speed + (index as f32 / 100.0) * speed;
                    weather.vrkumo.inst[index].position.x += wind_pow.x * step;
                    weather.vrkumo.inst[index].position.z += wind_pow.z * step;
                } else {
                    let falloff = weather.vrkumo.inst[index].dist_falloff;
                    let speed = speed * weather.vrkumo.inst[index].speed * falloff;
                    weather.vrkumo.inst[index].position.x += wind_pow.x * speed;
                    weather.vrkumo.inst[index].position.z += wind_pow.z * speed;
                }
            }
            3 => {}
            _ => {}
        }

        let mut flat = weather.vrkumo.inst[index].position;
        flat.y = 0.0;
        let mut ratio = flat.length() / 15000.0;
        let inside = (1.0 - ratio).max(0.0);
        let rank = index as f32 / 100.0;
        let mut edge = 1.0 - inside;
        edge = 1.0 - (edge * edge * edge);
        weather.vrkumo.inst[index].position.y = (500.0 * rank) + (radius + height * edge);
        if ratio > 1.0 {
            ratio = 1.0;
        }
        weather.vrkumo.inst[index].dist_falloff =
            1.0 - (ratio * ratio * ratio * ratio * ratio * ratio);

        let (mut max_alpha, alpha_step) = if frame.stage == "M_DragB" {
            weather.vrkumo.inst[index].alpha = 1.0;
            (1.0, 1.0)
        } else if index as i32 >= weather.vrkumo_count {
            (0.0, 0.005)
        } else if weather.vrkumo.inst[index].dist_falloff < 0.2 {
            if weather.vrkumo.inst[index].dist_falloff < 0.05 {
                (0.0, 0.1)
            } else {
                ((weather.vrkumo.inst[index].dist_falloff - 0.05) / 0.15, 0.1)
            }
        } else {
            (ceiling, 0.1)
        };
        if edge > inner {
            max_alpha = 0.0;
        } else if edge > 0.88 {
            if inner - 0.88 > 0.0 {
                max_alpha *= (inner - edge) / (inner - 0.88);
            } else {
                max_alpha = 0.0;
            }
        }
        add_calc(
            &mut weather.vrkumo.inst[index].alpha,
            max_alpha,
            0.2,
            alpha_step,
            0.01,
        );
    }
}

/// `wave_move` (`d_kankyo_rain.cpp:1737-1980`): the crests that ride the sea surface. The other
/// sibling has no counterpart for this, so it is taken from here.
pub fn wave_move(weather: &mut Weather, frame: &Frame) {
    if weather.wave_chan.wave_flat_inter >= 1.0 || !frame.has_view {
        return;
    }
    let spawn = set_eyevect_calc2(
        frame.eye,
        frame.center,
        weather.wave_chan.wave_spawn_dist,
        0.0,
    );
    let mut wind_pow = weather.wind_vecpow();
    let mut wind_power = weather.wind_pow();
    if frame.stage_type == STAGE_TYPE_MISC {
        let angle_y = weather
            .wind
            .tact_wind_angle_y
            .wrapping_add(stage_misc_wind(frame.stage));
        let normal = Vec3::new(
            scos(weather.wind.tact_wind_angle_x) * scos(angle_y),
            ssin(weather.wind.tact_wind_angle_x),
            scos(weather.wind.tact_wind_angle_x) * ssin(angle_y),
        );
        wind_pow = normal * 0.6;
        wind_power = 0.6;
    }
    let heading = snow_heading(frame);
    let wind_vec = weather.wind_vec();
    weather.wave.skew_dir =
        vector_product_2d(0.0, 0.0, -wind_vec.x, -wind_vec.z, heading.x, heading.z);
    weather.wave.skew_width = wind_power
        * (1.0 - wind_vec.y.abs())
        * (1.0 - (wind_vec.x * heading.x + wind_vec.z * heading.z).abs());
    weather.wave.skew_width *= 0.6 * weather.wave.skew_dir.abs();

    for index in 0..weather.wave_chan.wave_count.max(0) as usize {
        if weather.wave_chan.wave_reset != 0 {
            weather.wave.eff[index].status = 0;
        }
        if weather.wave.eff[index].status == 0 {
            weather.wave.eff[index].base_pos = Vec3::new(spawn.x, frame.sea_level, spawn.z);
            weather.wave.eff[index].pos = Vec3::new(
                rnd_fx(&mut weather.seed, weather.wave_chan.wave_spawn_radius),
                0.0,
                rnd_fx(&mut weather.seed, weather.wave_chan.wave_spawn_radius),
            );
            weather.wave.eff[index].counter = rnd_f(&mut weather.seed, 65536.0);
            weather.wave.eff[index].alpha = 0.0;
            weather.wave.eff[index].m32 = rnd_f(&mut weather.seed, 65535.0) as i16;
            weather.wave.eff[index].strength_env = 1.0;
            weather.wave.eff[index].scale = weather.wave_chan.wave_scale_rand
                + rnd_f(&mut weather.seed, 1.0 - weather.wave_chan.wave_scale_rand);
            weather.wave.eff[index].speed = weather.wave.eff[index].scale;
            weather.wave.eff[index].counter_speed = ((1.0 - weather.wave.eff[index].scale) * 0.05
                + 0.02)
                * weather.wave_chan.wave_counter_speed_scale;
            weather.wave.eff[index].m30 = 0;
            weather.wave.eff[index].status += 1;
        }
        if weather.wave.eff[index].status == 1 || weather.wave.eff[index].status == 2 {
            let drift = wind_pow
                * weather.wave_chan.wave_speed
                * weather.wave.eff[index].speed
                * (weather.wave.eff[index].strength_env * 0.5 + 0.5)
                * (weather.wave.eff[index].alpha * 0.8 + 0.2);
            weather.wave.eff[index].pos += drift;
            weather.wave.eff[index].counter += weather.wave.eff[index].counter_speed;
            let mut pos = weather.wave.eff[index].base_pos + weather.wave.eff[index].pos;
            if pos.distance(spawn) > weather.wave_chan.wave_spawn_radius {
                weather.wave.eff[index].base_pos.x = spawn.x;
                weather.wave.eff[index].base_pos.z = spawn.z;
                if pos.distance(spawn) > weather.wave_chan.wave_spawn_radius + 350.0 {
                    weather.wave.eff[index].pos.x =
                        rnd_fx(&mut weather.seed, weather.wave_chan.wave_spawn_radius);
                    weather.wave.eff[index].pos.z =
                        rnd_fx(&mut weather.seed, weather.wave_chan.wave_spawn_radius);
                } else {
                    let dir = vectle(pos, spawn);
                    weather.wave.eff[index].pos.x = dir.x * weather.wave_chan.wave_spawn_radius;
                    weather.wave.eff[index].pos.z = dir.z * weather.wave_chan.wave_spawn_radius;
                }
                weather.wave.eff[index].alpha = 0.0;
            }
            pos = weather.wave.eff[index].base_pos + weather.wave.eff[index].pos;

            weather.wave.eff[index].strength_env = 1.0;
            for gust in weather.wave_infl.iter().flatten() {
                let mut sample = gust.pos;
                sample.y = pos.y;
                let dist = pos.distance(sample);
                let outer = gust.outer_radius;
                let inner = gust.inner_radius;
                if dist < outer {
                    if dist < inner {
                        weather.wave.eff[index].strength_env = 0.0;
                        break;
                    }
                    let range = outer - inner;
                    if range > 0.0 {
                        if weather.wave.eff[index].strength_env > (dist - inner) / range {
                            weather.wave.eff[index].strength_env = (dist - inner) / range;
                        }
                    } else {
                        weather.wave.eff[index].strength_env = 0.0;
                    }
                }
            }

            if weather.wave_chan.wave_flat_inter > 0.0 {
                let mut sample = frame.eye;
                sample.y = pos.y;
                let dist = pos.distance(sample);
                let inner =
                    weather.wave_chan.wave_flat_inter * (weather.wave_chan.wave_spawn_radius * 1.5);
                let outer = inner + 1000.0;
                let range = outer - inner;
                if range > 0.0 {
                    if weather.wave.eff[index].strength_env > (dist - inner) / range {
                        weather.wave.eff[index].strength_env = (dist - inner) / range;
                    }
                } else {
                    weather.wave.eff[index].strength_env = 0.0;
                }
            }

            let mut sample = frame.player_pos;
            sample.y = pos.y;
            let dist = pos.distance(sample);
            if dist < 2000.0 {
                if dist < 200.0 {
                    weather.wave.eff[index].strength_env = 0.0;
                } else {
                    weather.wave.eff[index].strength_env *= (dist - 200.0) / 1800.0;
                }
            }
        } else if weather.wave.eff[index].status == 3 {
            weather.wave.eff[index].status = 0;
        }

        let pos = weather.wave.eff[index].base_pos + weather.wave.eff[index].pos;
        let dist = pos.distance(frame.eye).max(0.0);
        let mut target = 1.0 - (dist / (2.0 * weather.wave_chan.wave_spawn_dist));
        target *= 1.03;
        target *= weather.wave.eff[index].counter.sin();
        add_calc(
            &mut weather.wave.eff[index].alpha,
            target.clamp(0.0, 1.0),
            0.5,
            0.5,
            0.001,
        );
        weather.wave.eff[index].base_pos.y = frame.sea_level;
    }
}

/// `dKyr_kamome_move` (`d_kankyo_rain.cpp:109-225`): the two birds that circle the sea. They spawn
/// only on the one stage and never while a weather colour pattern is fading. The emitter calls are
/// the particle system's; every field the emitter is placed and rotated from is kept here.
pub fn kamome_move(weather: &mut Weather, frame: &Frame) {
    let mut spawn_birds = false;
    if frame.stage == "sea" {
        let current = weather.colpat_curr;
        let previous = weather.colpat_prev;
        let blend = weather.colpat_blend;
        if (current == 1 && blend > 0.0)
            || (previous == 1 && blend < 1.0)
            || (current == 2 && blend > 0.0)
            || (previous == 2 && blend < 1.0)
        {
            spawn_birds = false;
        } else {
            spawn_birds = true;
        }
    }
    for slot in 0..KAMOME_ACTIVE {
        let i = slot;
        match weather.wind_eff.kamome_eff[i].status {
            0 => {
                if spawn_birds {
                    if weather.wind_eff.kamome_eff[i].timer == 0 {
                        weather.wind_eff.kamome_eff[i].angle_y = rnd_angle(&mut weather.seed);
                        weather.wind_eff.kamome_eff[i].angle_x = rnd_angle(&mut weather.seed);
                        weather.wind_eff.kamome_eff[i].pos = Vec3::new(
                            frame.eye.x + ssin(weather.wind_eff.kamome_eff[i].angle_y) * 7000.0,
                            4500.0,
                            frame.eye.z + scos(weather.wind_eff.kamome_eff[i].angle_y) * 7000.0,
                        );
                        weather.wind_eff.kamome_eff[i].angle_y_speed =
                            rnd_fx(&mut weather.seed, 1.0);
                        weather.wind_eff.kamome_eff[i].scale = 0.0;
                        weather.wind_eff.kamome_eff[i].timer =
                            (rnd_f(&mut weather.seed, 180.0) + 300.0) as i16;
                        weather.wind_eff.kamome_eff[i].status += 1;
                    } else {
                        weather.wind_eff.kamome_eff[i].timer =
                            weather.wind_eff.kamome_eff[i].timer.wrapping_sub(1);
                    }
                }
            }
            1 => {
                let current = weather.wind_eff.kamome_eff[i].angle_y_speed;
                let next = current + rnd_fx(&mut weather.seed, 1.0);
                weather.wind_eff.kamome_eff[i].angle_y_speed = if current >= 0.0 {
                    next.clamp(80.0, 100.0)
                } else {
                    next.clamp(-100.0, -80.0)
                };
                let old_y = ssin(weather.wind_eff.kamome_eff[i].angle_y) * 7000.0;
                let old_x = fsin(f32::from(weather.wind_eff.kamome_eff[i].angle_x)).abs() * 3200.0;
                let old = Vec3::new(
                    old_y,
                    old_x,
                    scos(weather.wind_eff.kamome_eff[i].angle_y) * 7000.0,
                );
                let turn = (weather.wind_eff.kamome_eff[i].angle_y_speed
                    * weather.wind_eff.kamome_eff[i].scale) as i16;
                weather.wind_eff.kamome_eff[i].angle_y =
                    weather.wind_eff.kamome_eff[i].angle_y.wrapping_add(turn);
                weather.wind_eff.kamome_eff[i].angle_x =
                    weather.wind_eff.kamome_eff[i].angle_x.wrapping_add(15);
                let new_y = ssin(weather.wind_eff.kamome_eff[i].angle_y) * 7000.0;
                let new_x = fsin(f32::from(weather.wind_eff.kamome_eff[i].angle_x)).abs() * 3200.0;
                let new = Vec3::new(
                    new_y,
                    new_x,
                    scos(weather.wind_eff.kamome_eff[i].angle_y) * 7000.0,
                );
                weather.wind_eff.kamome_eff[i].pos =
                    Vec3::new(frame.eye.x + new.x, new.y + 4800.0, frame.eye.z + new.z);
                if weather.wind_eff.kamome_eff[i].timer != 0 && spawn_birds {
                    add_calc(
                        &mut weather.wind_eff.kamome_eff[i].scale,
                        1.0,
                        0.1,
                        0.003,
                        0.000001,
                    );
                    weather.wind_eff.kamome_eff[i].timer =
                        weather.wind_eff.kamome_eff[i].timer.wrapping_sub(1);
                } else {
                    add_calc(
                        &mut weather.wind_eff.kamome_eff[i].scale,
                        0.0,
                        0.1,
                        0.003,
                        0.000001,
                    );
                    if weather.wind_eff.kamome_eff[i].scale < 0.001 {
                        weather.wind_eff.kamome_eff[i].status += 1;
                    }
                }
                let _ = old;
            }
            2 => {
                weather.wind_eff.kamome_eff[i].status = 0;
                weather.wind_eff.kamome_eff[i].timer =
                    (rnd_f(&mut weather.seed, 600.0) + 900.0) as i16;
            }
            _ => {}
        }
    }
}

/// `dKyr_wind_init` (`d_kankyo_rain.cpp:228-238`).
pub fn wind_init(weather: &mut Weather) {
    weather.wind_eff.has_custom_wind_power = 0;
    weather.wind_eff.m759 = 0;
    for slot in 0..KAMOME_ACTIVE {
        weather.wind_eff.kamome_eff[slot].status = 0;
        weather.wind_eff.kamome_eff[slot].scale = 0.0;
        weather.wind_eff.kamome_eff[slot].timer = rnd_f(&mut weather.seed, 1800.0) as i16;
    }
}

/// `dKyr_wind_move` (`d_kankyo_rain.cpp:242-496`): the streaks of wind that blow past the camera.
/// Every field the emitters are placed and rotated from is kept here. The reference only advances a
/// streak out of its spawn state once its emitter has been created, which no port can do, so the
/// state advances regardless and the streak is a bare path with nothing on it.
pub fn wind_move(weather: &mut Weather, frame: &Frame, collision: Option<&Collision>) {
    let mut windline_count = weather.windline_count;
    if frame.player_leaf_flight {
        windline_count = 10;
    }

    let mut range = 4000.0f32;
    let mut pos_range = 2000.0f32;
    let mut speed = 80.0f32;
    let mut grow = 0.18f32;
    let mut pos_wind_scale = 2500.0f32;
    let mut offset_y = 1000.0f32;
    let mut angle_scale = 250.0f32;
    let mut sway = 800.0f32;

    let mut custom_changed = false;
    if weather.wind.custom_wind_power > 0.0 {
        windline_count = 9;
        range = 50.0;
        pos_range = 160.0;
        pos_wind_scale = pos_range;
        speed = 8.0;
        grow = 0.18;
        sway = 150.0;
        angle_scale = 200.0;
        offset_y = angle_scale;
        if weather.wind_eff.has_custom_wind_power == 0 {
            weather.wind_eff.has_custom_wind_power = 1;
            custom_changed = true;
        }
    } else if weather.wind_eff.has_custom_wind_power != 0 {
        custom_changed = true;
        weather.wind_eff.has_custom_wind_power = 0;
    }

    if custom_changed {
        for eff in &mut weather.wind_eff.wind_eff {
            eff.state = 0;
        }
    }

    let spawn = set_eyevect_calc2(frame.eye, frame.center, range, range);
    let wind_vec = weather.wind_vec();
    let wind_power = weather.wind_pow();
    let flat = (wind_vec.x * wind_vec.x + wind_vec.z * wind_vec.z).sqrt();

    for index in 0..WINDLINE_MAX {
        let i = index;
        if i as i32 >= windline_count && weather.wind_eff.wind_eff[i].state == 0 {
            continue;
        }
        match weather.wind_eff.wind_eff[i].state {
            0 => {
                if wind_power < 0.3 {
                    continue;
                }
                if frame.particle_count <= 1500
                    && ((frame.counter0 >> 4) & 7 != i as u32 & 3
                        || weather.wind_eff.has_custom_wind_power != 0)
                {
                    let custom = weather.wind_eff.has_custom_wind_power != 0;
                    let base = if custom { frame.player_pos } else { spawn };
                    weather.wind_eff.wind_eff[i].state_timer = 0.0;
                    weather.wind_eff.wind_eff[i].alpha = 0.0;
                    weather.wind_eff.wind_eff[i].m2c = 0;
                    weather.wind_eff.wind_eff[i].base_pos =
                        Vec3::new(base.x, base.y + offset_y, base.z);
                    let mut pos = Vec3::new(
                        rnd_fx(&mut weather.seed, pos_range),
                        rnd_fx(&mut weather.seed, pos_range),
                        rnd_fx(&mut weather.seed, pos_range),
                    );
                    let wind_scale =
                        pos_wind_scale + pos_wind_scale * rnd_f(&mut weather.seed, 1.0);
                    pos -= wind_vec * wind_scale;
                    weather.wind_eff.wind_eff[i].pos = pos;

                    let base = weather.wind_eff.wind_eff[i].base_pos;
                    if !custom {
                        let mut probe = base + weather.wind_eff.wind_eff[i].pos;
                        probe.y += 10000.0;
                        let ground = ground_y(collision, probe);
                        let pos_y = weather.wind_eff.wind_eff[i].base_pos.y
                            + weather.wind_eff.wind_eff[i].pos.y;
                        if pos_y < ground {
                            let roll = rnd_f(&mut weather.seed, offset_y);
                            weather.wind_eff.wind_eff[i].pos.y =
                                (ground + offset_y + roll) - base.y;
                        }
                    }

                    weather.wind_eff.wind_eff[i].m2c = rnd_f(&mut weather.seed, 65535.0) as i16;
                    weather.wind_eff.wind_eff[i].angle_xz = atan2s(wind_vec.x, wind_vec.z);
                    weather.wind_eff.wind_eff[i].angle_y = atan2s(wind_vec.y, flat);
                    weather.wind_eff.wind_eff[i].m28 = 0;
                    weather.wind_eff.wind_eff[i].m32 =
                        u8::from(rnd_f(&mut weather.seed, 1.0) <= 0.2);
                    weather.wind_eff.wind_eff[i].state = 1;
                }
            }
            1 | 2 => {
                let swing = angle_scale - ((0.2 * angle_scale) * (1.0 - wind_power));
                let phase = f32::from(weather.wind_eff.wind_eff[i].m2c) + sway * wind_power;
                weather.wind_eff.wind_eff[i].m2c = phase as i16;
                let wobble = (swing * fsin(phase)) as i16;
                let angle_y = weather.wind_eff.wind_eff[i].angle_y;
                let angle_xz = weather.wind_eff.wind_eff[i].angle_xz;
                if i & 1 != 0 {
                    weather.wind_eff.wind_eff[i].angle_y = angle_y.wrapping_add(wobble);
                    weather.wind_eff.wind_eff[i].angle_xz = angle_xz.wrapping_add(wobble);
                } else {
                    weather.wind_eff.wind_eff[i].angle_y = angle_y.wrapping_add(wobble);
                    weather.wind_eff.wind_eff[i].angle_xz = angle_xz.wrapping_sub(wobble);
                }
                if weather.wind_eff.wind_eff[i].state_timer > 0.4
                    && weather.wind_eff.wind_eff[i].m32 == 1
                {
                    weather.wind_eff.wind_eff[i].m28 += i as i32 * 200 + 3600;
                    weather.wind_eff.wind_eff[i].angle_y =
                        angle_y.wrapping_add(i as i16 * 200 + 3600);
                    if weather.wind_eff.wind_eff[i].m28 > 60535 {
                        weather.wind_eff.wind_eff[i].m32 = 0;
                    }
                } else {
                    let target_xz = atan2s(wind_vec.x, wind_vec.z);
                    let target_y = atan2s(wind_vec.y, flat);
                    add_calc_angle_s(
                        &mut weather.wind_eff.wind_eff[i].angle_y,
                        target_y,
                        10,
                        1000,
                        1,
                    );
                    add_calc_angle_s(
                        &mut weather.wind_eff.wind_eff[i].angle_xz,
                        target_xz,
                        10,
                        1000,
                        1,
                    );
                }

                let angle_y = weather.wind_eff.wind_eff[i].angle_y;
                let angle_xz = weather.wind_eff.wind_eff[i].angle_xz;
                let travel = Vec3::new(
                    scos(angle_y) * ssin(angle_xz),
                    ssin(angle_y),
                    scos(angle_y) * scos(angle_xz),
                );
                let base_speed = speed - ((speed * 0.2) * (1.0 - wind_power));
                let ramp = (f32::from(weather.wind_eff.wind_eff[i].m2c) / 65535.0).clamp(0.0, 1.0);
                let step = base_speed + (0.3 * base_speed * ramp);
                weather.wind_eff.wind_eff[i].pos += travel * step;

                let base = weather.wind_eff.wind_eff[i].base_pos;
                let pos = base + weather.wind_eff.wind_eff[i].pos;
                let mut fade_step = grow + ((grow * 0.1) * (i / 30) as f32);
                let key = (i32::from(weather.bg0_k0.r)
                    + i32::from(weather.bg0_k0.g)
                    + i32::from(weather.bg0_k0.b)) as f32
                    * 0.33333334
                    / 255.0;
                let distance = (pos.distance(frame.eye) / 200.0).min(1.0);
                let fade = (wind_power * (distance * square(key))).max(0.5);
                weather.wind_eff.wind_eff[i].emitter_alpha =
                    fade * 255.0 * weather.wind_eff.wind_eff[i].alpha;
                let mut probe = pos + travel * 250.0;
                probe.y += 10000.0;
                if ground_y(collision, probe) > pos.y {
                    fade_step = 0.2 + i as f32 * 0.01;
                    weather.wind_eff.wind_eff[i].state = 2;
                }
                if weather.wind_eff.wind_eff[i].state == 1 {
                    add_calc(
                        &mut weather.wind_eff.wind_eff[i].state_timer,
                        1.0,
                        0.3,
                        fade_step * 0.1,
                        0.01,
                    );
                    if weather.wind_eff.wind_eff[i].state_timer >= 1.0 {
                        weather.wind_eff.wind_eff[i].state = 2;
                    }
                    if weather.wind_eff.wind_eff[i].state_timer > 0.5 {
                        add_calc(
                            &mut weather.wind_eff.wind_eff[i].alpha,
                            1.0,
                            0.5,
                            0.05,
                            0.001,
                        );
                    }
                } else {
                    add_calc(
                        &mut weather.wind_eff.wind_eff[i].state_timer,
                        0.0,
                        0.4,
                        fade_step * (i as f32 * 0.01 + 0.1),
                        0.01,
                    );
                    if weather.wind_eff.wind_eff[i].state_timer <= 0.0 {
                        weather.wind_eff.wind_eff[i].state =
                            if weather.wind_eff.has_custom_wind_power != 0 {
                                4
                            } else {
                                0
                            };
                    }
                    if weather.wind_eff.wind_eff[i].state_timer < 0.5 {
                        add_calc(
                            &mut weather.wind_eff.wind_eff[i].alpha,
                            0.0,
                            0.5,
                            0.05,
                            0.001,
                        );
                    }
                }
            }
            3 => {
                if weather.wind_eff.has_custom_wind_power == 0 {
                    weather.wind_eff.wind_eff[i].state = 0;
                }
            }
            _ => {}
        }
    }

    kamome_move(weather, frame);
}

fn ground_y(collision: Option<&Collision>, probe: Vec3) -> f32 {
    collision
        .and_then(|collision| collision.ground_cross_below(probe))
        .map_or(F_INF, |hit| hit.height)
}

/// `wether_move_windline` (`d_kankyo_wether.cpp:400-444`): how many streaks the room asks for, and
/// the one-time arming of the whole layer.
pub fn wether_move_windline(weather: &mut Weather, frame: &Frame, collision: Option<&Collision>) {
    weather.windline_count = 0;
    if frame.path_wind_effect && frame.stage != "GTower" {
        weather.windline_count = (weather.wind_pow() * 10.0) as i32;
    }
    if weather.wind.m08.y < 10000000.0 {
        weather.windline_count = 0;
        weather.wind.m08.y = 10000000.0;
    }
    if weather.windline_initialized {
        wind_move(weather, frame, collision);
    } else if weather.windline_count != 0 {
        for eff in &mut weather.wind_eff.wind_eff {
            eff.state = 0;
        }
        wind_init(weather);
        wind_move(weather, frame, collision);
        weather.windline_initialized = true;
    }
}

/// `wether_move_sun` (`d_kankyo_wether.cpp:474-524`): the sun and its lens flare are built on the
/// first frame they are asked for and stepped every frame after.
pub fn wether_move_sun(weather: &mut Weather, frame: &Frame, colpat_weather: u8) {
    if !(frame.status_1 && !weather.vrbox_invisible) {
        return;
    }
    if !weather.sun_initialized {
        weather.sun.viz_chk_data = [0; SUN_CHECKS];
        weather.sun.m3c = 0;
        weather.sun.m3d = false;
        weather.sun.visibility = 0.0;
        weather.sun.sun_alpha = 0.0;
        weather.sun.moon_alpha = 0.0;
        weather.sun_lenz.m88 = 1000000000.0;
        weather.sun_lenz.m8c = 0.0;
        weather.sun_lenz.dist_falloff = 0.0;
        weather.sun_lenz.draw_lenz_in_sky = false;
        weather.sun_initialized = true;
    }
    sun_move(weather, frame, colpat_weather);
    lenzflare_move(weather, frame);
}

/// `wether_move_rain` (`d_kankyo_wether.cpp:527-563`).
pub fn wether_move_rain(
    weather: &mut Weather,
    frame: &Frame,
    rain_count: i32,
    collision: Option<&Collision>,
) {
    if weather.rain_initialized {
        rain_move(weather, frame, rain_count, collision);
        if rain_count == 0 {
            weather.rain_initialized = false;
        }
    } else if rain_count != 0 {
        rain_init(weather, frame);
        rain_move(weather, frame, rain_count, collision);
        weather.rain_initialized = true;
    }
}

/// `wether_move_snow` (`d_kankyo_wether.cpp:566-593`): the one stage with ground snow steps the
/// piled-snow half instead of the falling half.
pub fn wether_move_snow(
    weather: &mut Weather,
    frame: &Frame,
    snow_count: i32,
    collision: Option<&Collision>,
) {
    let piled = frame.stage == "Adanmae";
    if !weather.snow_initialized {
        if snow_count == 0 {
            return;
        }
        snow_init(weather, frame);
        if piled {
            kazanbai_move(weather, frame, snow_count, collision);
        } else {
            snow_move(weather, frame, snow_count);
        }
        weather.snow_initialized = true;
    } else if piled {
        kazanbai_move(weather, frame, snow_count, collision);
    } else {
        snow_move(weather, frame, snow_count);
    }
}

/// `wether_move_poison` (`d_kankyo_wether.cpp:664-680`).
pub fn wether_move_poison(weather: &mut Weather, frame: &Frame) {
    if frame.stage == "M_Dai" {
        weather.poison_count = 500;
    }
    if weather.poison_initialized {
        poison_move(weather, frame);
    } else if weather.poison_count != 0 {
        poison_init(weather, frame.room);
        weather.poison_initialized = true;
    }
}

/// `wether_move_housi` (`d_kankyo_wether.cpp:683-713`): the steam is asked for once and then spent,
/// because the count is cleared on every frame that steps it.
pub fn wether_move_housi(weather: &mut Weather, frame: &Frame, light: &mut EnvLight) {
    let housi_count = light.housi_count as i32;
    if !weather.housi_initialized {
        if housi_count != 0 {
            weather.housi.m5ddc = 0.0;
            for eff in &mut weather.housi.effect {
                eff.status = 0;
            }
            housi_move(weather, frame, housi_count);
            weather.housi_initialized = true;
        }
    } else if housi_count == 0 && weather.housi.m5ddc <= 0.0 {
        weather.housi_initialized = false;
    } else {
        housi_move(weather, frame, housi_count);
        light.housi_count = 0.0;
    }
}

/// `wether_move_moya` (`d_kankyo_wether.cpp:716-761`).
pub fn wether_move_moya(weather: &mut Weather, frame: &Frame, light: &EnvLight) {
    let moya_count = light.moya_count as i32;
    if !weather.moya_initialized {
        if moya_count != 0 {
            for eff in &mut weather.cloud.eff {
                eff.status = 0;
            }
            weather.cloud.count = 0;
            cloud_shadow_move(weather, frame, light.moya_mode, moya_count);
            weather.moya_initialized = true;
        }
        return;
    }
    if light.moya_mode == 0 && weather.wind_pow() > 0.2 && moya_count != 0 {
        if frame.stage == "M_NewD2" {
            weather.moya_se = if weather.vrbox_invisible {
                0x0852
            } else {
                0x0851
            };
        }
    } else if light.moya_mode == 1 {
        weather.moya_se = 0x0852;
    } else if light.moya_mode == 2 && frame.stage == "A_mori" {
        weather.moya_se = 0x0853;
    }
    cloud_shadow_move(weather, frame, light.moya_mode, moya_count);
    if moya_count == 0 && weather.cloud.count == 0 {
        weather.moya_initialized = false;
    }
}

/// `wether_move_vrkumo` (`d_kankyo_wether.cpp:764-846`): the count and the strength the dome cloud
/// eases toward, then the layer's own arming.
pub fn wether_move_vrkumo(weather: &mut Weather, frame: &Frame) {
    const VRKUMO_MARKER: Vec3 = Vec3::new(-180000.0, 750.0, -200000.0);
    if frame.stage == "Name" {
        weather.vrkumo_count = 70;
    } else if frame.status_1 && !weather.vrbox_invisible {
        let current = weather.colpat_curr;
        let previous = weather.colpat_prev;
        if (current == 1 && weather.colpat_blend > 0.0)
            || (previous == 1 && weather.colpat_blend < 1.0)
            || (current == 2 && weather.colpat_blend > 0.0)
            || (previous == 2 && weather.colpat_blend < 1.0)
        {
            add_calc(&mut weather.vrkumo_strength, 1.0, 0.1, 0.003, 0.0000001);
        } else {
            add_calc(&mut weather.vrkumo_strength, 0.0, 0.08, 0.002, 0.00000001);
        }
        if frame.stage == "sea" && frame.room == 9 && frame.eye.distance(VRKUMO_MARKER) < 2500.0 {
            weather.vrkumo_strength = 1.0;
        }
        weather.vrkumo_count = (weather.vrkumo_strength * 50.0 + 50.0) as i32;
        if frame.stage == "GTower" {
            weather.vrkumo_count = 0;
        }
    } else {
        weather.vrkumo_count = 0;
    }
    match weather.vrkumo_status {
        0 => {
            if weather.vrkumo_count != 0 {
                for inst in &mut weather.vrkumo.inst {
                    inst.status = 0;
                }
                vrkumo_move(weather, frame);
                weather.vrkumo_status = 1;
            }
        }
        1 => vrkumo_move(weather, frame),
        _ => {}
    }
}

/// `wether_move_wave` (`d_kankyo_wether.cpp:849-878`).
pub fn wether_move_wave(weather: &mut Weather, frame: &Frame) {
    if !weather.wave_initialized {
        if weather.wave_chan.wave_count == 0 {
            return;
        }
        for eff in &mut weather.wave.eff {
            eff.status = 0;
        }
        wave_move(weather, frame);
        weather.wave_initialized = true;
    } else {
        if weather.wave_chan.wave_count == 0 {
            weather.wave_initialized = false;
        }
        wave_move(weather, frame);
    }
}

/// `dKyw_wether_move` (`d_kankyo_wether.cpp:468-471`): the thunder and the wind streaks, which the
/// one stage with its own calendar does not run.
pub fn wether_move(weather: &mut Weather, frame: &Frame, collision: Option<&Collision>) {
    if frame.stage == "Name" {
        return;
    }
    wether_move_thunder(weather, frame);
    wether_move_windline(weather, frame, collision);
}

/// `dKyw_wether_move_draw` (`d_kankyo_wether.cpp:881-901`): every weather layer in the reference's
/// order, with the calendar stage skipping all but the stars.
pub fn wether_move_draw(
    weather: &mut Weather,
    frame: &Frame,
    light: &mut EnvLight,
    collision: Option<&Collision>,
) {
    weather.moya_se = 0;
    if frame.stage != "Name" {
        wether_move_sun(weather, frame, light.colpat_weather);
        wether_move_rain(weather, frame, light.rain_count as i32, collision);
        wether_move_snow(weather, frame, light.snow_count as i32, collision);
    }
    wether_move_star(weather, frame, light.colpat_weather);
    if frame.stage != "Name" {
        wether_move_poison(weather, frame);
        wether_move_housi(weather, frame, light);
        wether_move_moya(weather, frame, light);
        wether_move_wave(weather, frame);
    }
}

/// `dKyw_wether_move_draw2` (`d_kankyo_wether.cpp:904-906`): the dome cloud runs as its own process,
/// one step after the rest.
pub fn wether_move_draw2(weather: &mut Weather, frame: &Frame) {
    wether_move_vrkumo(weather, frame);
}

/// The colour-pattern handoff `exeKankyo` (`d_kankyo.cpp:2000-2087`) runs before it steps the
/// weather: the gathered values become the applied ones. The ratio pair at `0xC14` is left out
/// because the gather half lives in the stage palette tables rather than in this structure.
pub fn color_pattern_handoff(weather: &mut Weather, light: &mut EnvLight) {
    weather.colpat_mode = light.colpat_mode_gather;
    if light.colpat_mode_gather != 0 {
        light.colpat_mode_gather = if light.colpat_mode_gather >= 3 {
            0
        } else {
            light.colpat_mode_gather + 1
        };
    }
    if weather.colpat_mode != 0 {
        if light.colpat_prev_gather != 0xFF {
            weather.colpat_prev = light.colpat_prev_gather;
            if light.colpat_mode_gather == 0 {
                light.colpat_prev_gather = 0xFF;
            }
        }
        if light.colpat_curr_gather != 0xFF {
            weather.colpat_curr = light.colpat_curr_gather;
            if light.colpat_mode_gather == 0 {
                light.colpat_curr_gather = 0xFF;
            }
        }
        if light.colpat_blend_gather >= 0.0 {
            weather.colpat_blend = light.colpat_blend_gather;
            if light.colpat_mode_gather == 0 {
                light.colpat_blend_gather = -1.0;
            }
        }
    } else if weather.colpat_prev == weather.colpat_curr {
        if light.colpat_prev_gather != 0xFF {
            weather.colpat_prev = light.colpat_prev_gather;
            light.colpat_prev_gather = 0xFF;
        }
        let current = light.colpat_curr_gather;
        if current != 0xFF {
            weather.colpat_curr = current;
            light.colpat_curr_gather = 0xFF;
            light.colpat_weather = current;
        }
        if light.colpat_blend_gather >= 0.0 {
            weather.colpat_blend = light.colpat_blend_gather;
            light.colpat_blend_gather = -1.0;
        }
    }
    weather.all_col_ratio = weather.all_col_gather_ratio;
    weather.act_col_ratio = weather.act_col_gather_ratio;
    weather.bg_col_ratio = weather.bg_col_gather_ratio;
    weather.fog_col_ratio = weather.fog_col_gather_ratio;
    weather.vrbox_sora_col_ratio = weather.vrbox_sora_col_gather_ratio;
    weather.vrbox_kumo_col_ratio = weather.vrbox_kumo_col_gather_ratio;

    weather.all_col_gather_ratio = 1.0;
    weather.act_col_gather_ratio = 1.0;
    weather.bg_col_gather_ratio = 1.0;
    weather.fog_col_gather_ratio = 1.0;
    weather.vrbox_sora_col_gather_ratio = 1.0;
    weather.vrbox_kumo_col_gather_ratio = 1.0;
}

/// `dScnKy_env_light_c::setSunpos` (`d_kankyo.cpp:582-606`): the sun and the moon sit opposite one
/// another on an eighty-thousand unit circle that turns once a day.
pub fn sun_pos(weather: &mut Weather, frame: &Frame) {
    let angle = if weather.time > 15.0 {
        weather.time - 15.0
    } else {
        weather.time + 345.0
    };
    let offset = Vec3::new(
        (angle * DEG2RAD).sin() * 80000.0,
        (angle * DEG2RAD).cos() * 80000.0,
        (angle * DEG2RAD).cos() * -48000.0,
    );
    weather.sun_pos = Vec3::new(
        frame.eye.x + offset.x,
        frame.eye.y - offset.y,
        frame.eye.z + offset.z,
    );
    weather.moon_pos = Vec3::new(
        frame.eye.x - offset.x,
        frame.eye.y + offset.y,
        frame.eye.z - offset.z,
    );
}

/// `mDoLib_project` (`m_Do/m_Do_lib.cpp:52-105`), which the sun's depth probe and the lens flare's
/// screen angle both take. The projection, the far clamp and the viewport map are already the
/// camera module's, so this is that function over a [`Frame`]'s own matrices.
pub fn project(frame: &Frame, src: Vec3) -> Vec3 {
    super::camera::project(&frame.proj_view, &frame.port, src)
}

/// The environment light's execute half (`dKy_Execute` at `d_kankyo.cpp:2399-2409` calling
/// `exeKankyo` at `:2000-2087`): the clock copy, the colour-pattern handoff, the gust step and the
/// wind. The sun position and the effect layers are the draw half and live in [`control_draw`].
pub fn control(
    mut weather: ResMut<Weather>,
    mut light: ResMut<EnvLight>,
    camera: Res<Camera>,
    day: Res<DayNight>,
    frame_res: Res<CurrentFrame>,
    stage: Option<Res<StageName>>,
    players: Query<&Player>,
) {
    if let Some(player) = players.iter().next() {
        weather.player_pos = player.pos;
    }
    let frame = Frame::capture(
        &weather,
        Some(&camera),
        frame_res.0 as u32,
        stage.as_deref(),
    );
    set_daytime(&mut weather, &day);
    color_pattern_handoff(&mut weather, &mut light);
    squal_proc(&mut weather);
    wind_set(&mut weather, &frame);
}

/// `setDaytime` (`d_kankyo.cpp:502-546`) and `getDaytime` (`:609-611`): the weather's own copy of the
/// clock. The reference reads the game clock in, steps it while a waker is held and no event is
/// running, and writes it back; [`DayNight`] already carries that step, so what is left here is the
/// pair of copies and the day-of-week field the star and dome-cloud layers time themselves on.
pub fn set_daytime(weather: &mut Weather, day: &DayNight) {
    weather.time = day.time;
    weather.day_of_week = day.weekday() as u16;
}

/// The draw pass: `drawKankyo`'s `setSunpos` (`d_kankyo.cpp:582-606`) and then the effect process's
/// `dKyw_wether_move` and `dKyw_wether_move_draw` (`d_kyeff.cpp:107-116`), in that order.
pub fn control_draw(
    mut weather: ResMut<Weather>,
    mut light: ResMut<EnvLight>,
    camera: Res<Camera>,
    frame_res: Res<CurrentFrame>,
    stage: Option<Res<StageName>>,
    collision: Option<Res<Collision>>,
) {
    let frame = Frame::capture(
        &weather,
        Some(&camera),
        frame_res.0 as u32,
        stage.as_deref(),
    );
    sun_pos(&mut weather, &frame);
    wether_move(&mut weather, &frame, collision.as_deref());
    wether_move_draw(&mut weather, &frame, &mut light, collision.as_deref());
}

/// The dome cloud, which the reference runs from its own process one step after the rest
/// (`d_kyeff2.cpp:19-21`).
pub fn control_clouds(
    mut weather: ResMut<Weather>,
    camera: Res<Camera>,
    stage: Option<Res<StageName>>,
) {
    let frame = Frame::capture(&weather, Some(&camera), 0, stage.as_deref());
    wether_move_draw2(&mut weather, &frame);
}

/// The lighting half of `dScnKy_env_light_c` — the palette tables behind `setLight_palno_get`, the
/// `dKy_tevstr_c` structures behind `setLight_bg`, `setLight_actor` and the `settingTevStruct*`
/// family, and the `setLightTevColorType` walk — is deliberately absent. Every one of those bodies
/// in `d_kankyo.cpp` is mostly `GXSetTevColor` and light-object stores, and the colour tables they
/// read come out of the stage's palette blobs rather than being computed. What survives here is
/// everything those functions read: the sun and moon positions, the colour-pattern handoff and the
/// six ratios `setLight` multiplies its palette entries by.
pub fn lighting_note() {}

/// `dKyr_drawSun` (`d_kankyo_rain.cpp:2996-3309`). Documented no-op: it needs a textured billboard
/// pass with per-vertex colour and alpha, an eight-vertex quad set and a depth test against the
/// virtual box, none of which this port's renderer offers.
pub fn draw_sun(_weather: &Weather) {}

/// `dKyr_drawLenzflare` (`d_kankyo_rain.cpp:3310-3642`). Documented no-op: it needs a screen-aligned
/// additive quad drawn after the opaque pass with the lens, ring and snow textures.
pub fn draw_lenzflare(_weather: &Weather) {}

/// `dKyr_drawRain` (`d_kankyo_rain.cpp:3643-3805`). Documented no-op: it needs one camera-facing
/// textured quad per drop with a per-vertex alpha.
pub fn draw_rain(_weather: &Weather) {}

/// `dKyr_drawSibuki` (`d_kankyo_rain.cpp:3806-3934`). Documented no-op: the splash layer is a
/// ground-aligned textured quad pass.
pub fn draw_sibuki(_weather: &Weather) {}

/// `drawPoison` (`d_kankyo_rain.cpp:3935-4078`). Documented no-op: a soft-particle textured quad
/// pass with the smoke texture, one quad per emitter.
pub fn draw_poison(_weather: &Weather) {}

/// `dKyr_drawHousi` (`d_kankyo_rain.cpp:4079-4255`). Documented no-op: a camera-facing textured quad
/// pass with per-quad scale and alpha, one quad per steam puff.
pub fn draw_housi(_weather: &Weather) {}

/// `dKyr_drawKazanbai` (`d_kankyo_rain.cpp:4256-4519`). Documented no-op: a ground-aligned textured
/// quad pass for the piled snow.
pub fn draw_kazanbai(_weather: &Weather) {}

/// `dKyr_drawSnow` (`d_kankyo_rain.cpp:4520-4668`). Documented no-op: a camera-facing textured quad
/// pass with per-quad scale, one quad per flake.
pub fn draw_snow(_weather: &Weather) {}

/// `dKyr_drawStar` (`d_kankyo_rain.cpp:4669-4888`). Documented no-op: a point-sprite textured pass
/// whose brightness is the twinkle sine the simulation keeps.
pub fn draw_star(_weather: &Weather) {}

/// `drawWave` (`d_kankyo_rain.cpp:4889-5041`). Documented no-op: a sea-surface textured strip pass
/// clipped against the player's wake.
pub fn draw_wave(_weather: &Weather) {}

/// `drawCloudShadow` (`d_kankyo_rain.cpp:5042-5187`). Documented no-op: a ground-projected
/// transparent decal pass for the cloud shadows.
pub fn draw_cloud_shadow(_weather: &Weather) {}

/// `drawVrkumo` (`d_kankyo_rain.cpp:5188-5538`). Documented no-op: a sky-dome textured pass with
/// three cloud layers blended over the virtual box.
pub fn draw_vrkumo(_weather: &Weather) {}

pub fn init(world: &mut World) {
    world.init_resource::<Weather>();
}
