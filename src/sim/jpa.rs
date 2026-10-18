use bevy_ecs::prelude::*;
use glam::{Mat3, Mat4, Vec3, Vec4};

use super::camera::{Camera, View};
use super::jpa_bank::{JpaBank, JpaBankError, JpaResourceRef};
use super::weather::{Weather, pntwind_get_info};

/// `DEG2RAD_CONSTANT` (`SSystem/SComponent/c_angle.h:7`).
pub const DEG2RAD: f32 = core::f32::consts::PI / 180.0;
/// `S2DEG_CONSTANT` (`SSystem/SComponent/c_angle.h:9`), the fixed-point circle to degrees.
pub const S2DEG: f32 = 180.0 / 32768.0;
/// `cXyz::isNearZeroSquare`'s threshold (`SSystem/SComponent/c_xyz.h:111`).
pub const NEAR_ZERO_SQUARE: f32 = 8.0e-11;
/// `cXyz::isZero`'s per-component threshold (`SSystem/SComponent/c_xyz.cpp:142-144`).
pub const ZERO_EPSILON: f32 = 3.8146972e-06;

/// `new JPAEmitterManager(mCommonResMng, 3000, 150, 200, mHeap)` (`d_particle.cpp:649`), whose
/// three sizes are the particle, emitter and field pool capacities
/// (`JPAEmitterManager.cpp:16-35`). All three are retail, not chosen.
pub const PARTICLE_POOL: usize = 3000;
pub const EMITTER_POOL: usize = 150;
pub const FIELD_POOL: usize = 200;

/// `new dPa_J3Dmodel_c[0x80]` (`d_particle.cpp:170`), the size the model control's free list is
/// walked by at `:174`, `:203` and `:236`.
pub const MODEL_POOL: usize = 0x80;
/// `mSimpleCallbacks[25]` (`d/d_particle.h:563`), the ceiling `newSimple` refuses past
/// (`d_particle.cpp:917`).
pub const SIMPLE_SLOTS: usize = 25;
/// `mSimpleData[32]` (`d/d_particle.h:118`), the bound `set` tests before it stores
/// (`d_particle.cpp:565`).
pub const SIMPLE_DATA: usize = 32;

/// `mDoExt_J3DModel__create`'s second and third arguments (`d_particle.cpp:175`): the model heap
/// size and the creation flags. Neither has a counterpart in this port; both are carried so the
/// blocked half of the model draw is visible rather than implied.
pub const MODEL_HEAP_SIZE: u32 = 0x80000;
pub const MODEL_CREATE_FLAGS: u32 = 0x37221223;

/// `mDoExt_createSolidHeapToCurrent(r29 * 12 + 16, NULL, 0)` (`d_particle.cpp:45`): the size of the
/// per-material texture-index array and the heap's own header.
pub const MATERIAL_ENTRY_SIZE: u32 = 12;
pub const HEAP_HEADER_SIZE: u32 = 16;

/// `dPa_setWindPower`'s gain on the wind a particle is pushed by (`d_particle.cpp:453`).
pub const WIND_POWER_GAIN: f32 = 8.0;
/// `dKyw_get_AllWind_vec`'s gain on the registered gust's share (`d_kankyo_wether.cpp:1294`).
pub const GUST_GAIN: f32 = 5.0;
/// The sphere `dPa_simpleEcallBack::executeAfter` rejects a spawn point outside of
/// (`d_particle.cpp:491`).
pub const SPAWN_CLIP_RADIUS: f32 = 200.0;
/// `dPa_followEcallBack::execute`'s fade: the alpha it chases to and the step it takes
/// (`d_particle.cpp:281`).
pub const FADE_ALPHA_TARGET: i16 = 1;
pub const FADE_ALPHA_STEP: i16 = 4;

/// `ID_AK_ST_O_BKMSATTACKSMOKE00` (`d_particle_name.h:259`), one of the two ids
/// `dPa_simpleEcallBack::create` arms the smoke draw on (`d_particle.cpp:600`).
pub const ID_AK_ST_O_BKMSATTACKSMOKE00: u16 = 0xA06A;
/// `ID_AK_ST_O_KGTT2JUMPHANDSMOKE00` (`d_particle_name.h:1108`), the second such id
/// (`d_particle.cpp:602`).
pub const ID_AK_ST_O_KGTT2JUMPHANDSMOKE00: u16 = 0xA410;

/// The behaviour table: the two effect ids whose *reaction* this module models, namely the smoke
/// particle callback `dPa_simpleEcallBack::create` arms (`d_particle.cpp:598-611`). Every other id
/// the bank carries is created as an identity-only emitter.
///
/// The bank and this table never stand in for each other. The bank decides whether a resource
/// exists and which ids it holds: [`create_emitter_id`] refuses an id no loaded archive carries,
/// exactly as `getByUserIndex` answering NULL does (`JPAEmitterManager.cpp:61-66`), even when the
/// id is named here. This table only decides what an emitter that exists does; an id absent from
/// it gets no smoke callback rather than a guessed one.
pub const PORTED_SMOKE_IDS: [u16; 2] = [
    ID_AK_ST_O_BKMSATTACKSMOKE00,
    ID_AK_ST_O_KGTT2JUMPHANDSMOKE00,
];

/// Whether `res_id` is one of [`PORTED_SMOKE_IDS`].
pub fn is_ported_smoke_id(res_id: u16) -> bool {
    PORTED_SMOKE_IDS.contains(&res_id)
}

/// The resource-manager slot `dPa_control_c::createCommon` mounts `common.jpc` into
/// (`d_particle.cpp:713-735`, the constructor at `:715`): the manager the emitter manager was
/// handed at construction (`JPAEmitterManager.cpp:21`), reached for every effect id whose bit 15
/// is clear (`d_particle.cpp:620`, `:698-701`).
pub const COMMON_RM_ID: u8 = 0;
/// `dPa_control_c::createRoomScene`'s slot: `mEmitterMng->pResMgrArray[1] = mSceneResMng`
/// (`d_particle.cpp:744`), the stage's `Pscene###.jpc`.
pub const SCENE_RM_ID: u8 = 1;
/// The archive `d_s_logo.cpp:931` reads into `createCommon`; `dPa_control_c::readScene` names the
/// per-stage one `Pscene%03d.jpc` (`d_particle.cpp:757-770`).
pub const COMMON_ARCHIVE: &str = "common.jpc";
/// `dPa_control_c::readScene`'s own "no archive" token (`d_particle.cpp:758-760`): the particle
/// number a stage whose `mParticleSceneNo` carries `0xFF` mounts nothing for.
pub const NO_SCENE: u8 = 0xFF;

/// `dPtclGroup_Normal_e` through `dPtclGroup_2DmenuBack_e` (`d/d_particle.h:399-411`): the eleven
/// group ids the manager's per-group `calc` walks and the follow and simple draws branch on.
pub const GROUP_NORMAL: u8 = 0;
pub const GROUP_NORMAL_P1: u8 = 1;
pub const GROUP_TOON: u8 = 2;
pub const GROUP_TOON_P1: u8 = 3;
pub const GROUP_PROJECTION: u8 = 4;
pub const GROUP_SHIP_TAIL: u8 = 5;
pub const GROUP_WIND: u8 = 6;
pub const GROUP_2D_FORE: u8 = 7;
pub const GROUP_2D_BACK: u8 = 8;
pub const GROUP_2D_MENU_FORE: u8 = 9;
pub const GROUP_2D_MENU_BACK: u8 = 10;

/// `JPAEmtrStts_StopEmit` (`JPAEmitter.h:68`).
pub const EMITTER_STOP_EMIT: u32 = 0x01;
/// `JPAEmtrStts_StopDraw` (`JPAEmitter.h:70`).
pub const EMITTER_STOP_DRAW: u32 = 0x04;
/// `JPAEmtrStts_EnableDeleteEmitter` (`JPAEmitter.h:71`), which
/// [`Emitter::is_enable_delete_emitter`] reads.
pub const EMITTER_ENABLE_DELETE: u32 = 0x08;
/// `JPAEmtrStts_Immortal` (`JPAEmitter.h:74`).
pub const EMITTER_IMMORTAL: u32 = 0x40;

/// `GXColor`, the byte colour every palette write and every particle tint lands in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    /// The four-channel constructor.
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// The `dPa_smokeEcallBack` constructor default and the literal `dPa_simpleEcallBack::draw`
    /// passes the same way (`d_particle.cpp:322`, `:333`, `:518`).
    pub const SMOKE: Color = Color::new(0xa0, 0xa0, 0x80, 0xff);

    /// The black the smoke lighting block writes as the channel ambient colour
    /// (`d_com_inf_game.cpp:30`).
    pub const BLACK: Color = Color::new(0x00, 0x00, 0x00, 0xff);
}

/// `GXColorS10` (`GXStruct.h:14-19`), whose four channels are `s16` in retail.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ColorS10 {
    pub r: i16,
    pub g: i16,
    pub b: i16,
    pub a: i16,
}

/// `dKy_tevstr_c::mColorC0` (`0x80`) and `mColorK0` (`0x88`) (`d/d_kankyo.h:109-115`): the two
/// colours a smoke emitter's lighting block is taken from when the emitter carries one. This is the
/// only part of that structure the draw reads.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TevColors {
    pub color_c0: ColorS10,
    pub color_k0: Color,
}

/// The environment light's `mActorC0` (`d/d_kankyo.h:0xB18`) and `mActorK0` (`0xB20`), which
/// `smokeEcallBack` substitutes for `dKy_tevstr_c` when the emitter carries none
/// (`d_particle.cpp:391-400`). Neither is written anywhere in `weather.rs`, which carries the
/// background key colour at `0xB30` instead, so the caller supplies both or the lighting block has
/// no source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ActorLight {
    pub color_c0: ColorS10,
    pub color_k0: Color,
}

/// `GXSetTevKColorSel`'s operand: pass through, `K0` or `K1`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum KColorSel {
    #[default]
    Passthrough,
    K0,
    K1,
}

/// The colour inputs a tev stage takes (`GXSetTevColorIn`'s four operands).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cca {
    /// `GX_CC_C0`, the primary colour register.
    Reg0,
    /// `GX_CC_KONST`, the stage's constant.
    Konst,
    /// `GX_CC_TEXC`, the texel colour.
    Texc,
    /// `GX_CC_ZERO`.
    Zero,
    /// `GX_CC_CPREV`, the previous stage's colour.
    Prev,
}

/// The alpha inputs a tev stage takes (`GXSetTevAlphaIn`'s four operands).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Caa {
    /// `GX_CA_ZERO`.
    Zero,
    /// `GX_CA_RASA`, the vertex alpha scaled by the vertex alpha.
    Rasa,
    /// `GX_CA_A0`, the primary colour register's alpha.
    A0,
    /// `GX_CA_APREV`, the previous stage's alpha.
    Prev,
    /// `GX_CA_TEXA`, the texel alpha.
    Texa,
}

/// `GXSetTevOrder`'s destination coordinate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TexCoordId {
    /// `GX_TEXCOORD_NULL`.
    Null,
    /// `GX_TEXCOORD0`.
    Coord0,
    /// `GX_TEXCOORD1`.
    Coord1,
}

/// `GXSetTevOrder`'s texture map.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TexMapId {
    /// `GX_TEXMAP_NULL`.
    Null,
    /// `GX_TEXMAP0`.
    Map0,
    /// `GX_TEXMAP1`.
    Map1,
}

/// One of the three tev stages the smoke lighting block builds (`d_particle.cpp:410-425`). The
/// colour and alpha operations are the same on all three - add, the texenv register left alone, the
/// full shift, the register-compare flag on, and the result registered for the next stage
/// (`:412`, `:414`, `:417`, `:419`, `:422`, `:424`) - so they are not fields, and neither is an
/// output operand: the operations above are what writes the stage's colour and alpha registers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TevStage {
    pub color_in: [Cca; 4],
    pub alpha_in: [Caa; 4],
    pub tex_coord: TexCoordId,
    pub tex_map: TexMapId,
}

/// `TEV_STAGES`, the three stages in the order they are set (`d_particle.cpp:410-425`).
pub const TEV_STAGES: [TevStage; 3] = [
    TevStage {
        color_in: [Cca::Reg0, Cca::Konst, Cca::Texc, Cca::Zero],
        alpha_in: [Caa::Rasa, Caa::Zero, Caa::Zero, Caa::Zero],
        tex_coord: TexCoordId::Coord1,
        tex_map: TexMapId::Map1,
    },
    TevStage {
        color_in: [Cca::Reg0, Cca::Prev, Cca::Texc, Cca::Zero],
        alpha_in: [Caa::Zero, Caa::Prev, Caa::Texa, Caa::Zero],
        tex_coord: TexCoordId::Coord0,
        tex_map: TexMapId::Map0,
    },
    TevStage {
        color_in: [Cca::Zero, Cca::Prev, Cca::Konst, Cca::Zero],
        alpha_in: [Caa::Zero, Caa::Zero, Caa::Zero, Caa::Prev],
        tex_coord: TexCoordId::Null,
        tex_map: TexMapId::Null,
    },
];

/// `GXSetTexCoordGen`'s generation mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TexGenKind {
    /// `GX_TG_MTX2x4`: a 2x4 texture matrix, identity here.
    Matrix2x4,
    /// `GX_TG_SRTG`: scale, rotate and translate taken from a colour register.
    Srtg,
}

/// `GXSetTexCoordGen`'s source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TexGenSource {
    /// `GX_TG_TEX0`.
    Tex0,
    /// `GX_TG_COLOR0`.
    Color0,
}

/// `TEX_GENS`, the two coordinate generators the smoke lighting block builds
/// (`d_particle.cpp:407-408`), each with the identity post-transform.
pub const TEX_GENS: [(TexGenKind, TexGenSource); 2] = [
    (TexGenKind::Matrix2x4, TexGenSource::Tex0),
    (TexGenKind::Srtg, TexGenSource::Color0),
];

/// The state `initiateLighting` writes and `smokeEcallBack` then builds its three stages around.
///
/// The lighting is per emitter, not per particle. The draw is a no-op because a renderer whose
/// vocabulary is indexed triangles, one texture page, one coordinate set and one group alpha cannot
/// express a register ambient colour, a K colour selected per stage, a three-stage tev chain, or a
/// camera normal matrix standing in for the light direction. What a renderer would need is a
/// particle pass with a configurable tev chain and per-stage K-colour selection, two texture
/// coordinate generators, and a per-emitter lighting uniform block.
#[derive(Debug, Clone, PartialEq)]
pub struct Lighting {
    /// `GXSetTevColorS10(GX_TEVREG0, dif)`.
    pub dif: ColorS10,
    /// `GXSetTevKColor(GX_KCOLOR0, amb)`, selected on stage zero.
    pub k0: Color,
    /// `GXSetTevKColor(GX_KCOLOR1, k1)`, selected on stage two. Its alpha is replaced by the
    /// emitter's global alpha once the draw gets that far.
    pub k1: Color,
    /// `GXSetChanAmbColor(GX_COLOR0A0, ...)`: the fixed black ambient of `d_com_inf_game.cpp:30`.
    pub channel_amb: Color,
    /// `GXSetChanCtrl`'s lit-from-vertex operand, on `GX_COLOR0A0` with the difference clamped and
    /// no attenuation.
    pub channel_light_from_vertex: bool,
    pub num_chans: u8,
    pub num_tex_gens: u8,
    pub num_tev_stages: u8,
    pub kcolor_sel: [KColorSel; 3],
    /// `emtr->loadTexture(1, GX_TEXMAP1)`: the emitter's second texture on the second map.
    pub tex_map1: bool,
    /// The z-draw test the whole block hangs off, inverted: [`smoke_draw_state`] returns nothing
    /// at all when it is set.
    pub z_draw: bool,
}

/// `initiateLighting` (`d_particle.cpp:364-372`).
pub fn initiate_lighting(dif: ColorS10, amb: Color, k1: Color) -> Lighting {
    let mut kcolor_sel = [KColorSel::Passthrough; 3];
    kcolor_sel[0] = KColorSel::K0;
    kcolor_sel[2] = KColorSel::K1;
    Lighting {
        dif,
        k0: amb,
        k1,
        channel_amb: Color::BLACK,
        channel_light_from_vertex: true,
        num_chans: 1,
        num_tex_gens: 2,
        num_tev_stages: 3,
        kcolor_sel,
        tex_map1: true,
        z_draw: false,
    }
}

/// The environment light's actor colour and diffuse: the source the smoke lighting block falls back
/// to when the emitter carries no `dKy_tevstr_c` (`d_particle.cpp:391-400`). The fourth channel of
/// each is forced opaque there, whatever the environment light holds.
pub fn actor_lighting(actor: ActorLight) -> (ColorS10, Color) {
    (
        ColorS10 {
            r: actor.color_c0.r,
            g: actor.color_c0.g,
            b: actor.color_c0.b,
            a: 0xFF,
        },
        Color {
            r: actor.color_k0.r,
            g: actor.color_k0.g,
            b: actor.color_k0.b,
            a: 0xFF,
        },
    )
}

/// `dPa_smokePcallBack` and `dPa_windPcallBack`, the two per-particle callbacks the ported emitters
/// install (`d/d_particle.h:271-297`). The smoke callback's `draw` is `Nonmatching` in retail
/// (`d_particle.cpp:464-466`) and the wind callback has no `draw` at all, so neither has a draw to
/// leave a no-op.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleCallback {
    /// `dPa_control_c::mSmokePcallback` (`d_particle.cpp:610`), installed by
    /// `dPa_smokeEcallBack::setup` (`:359`) and by `dPa_simpleEcallBack::create` (`:540`).
    Smoke,
    /// `dPa_windPcallBack`, whose one static instance `dPa_simpleEcallBack::executeAfter` installs
    /// per particle (`d_particle.cpp:503-504`).
    Wind,
}

/// `JPABaseParticle` (`JPAParticle.h:50-115`) narrowed to the fields the ported callbacks read and
/// write, by the offsets that header documents. The integration the base class does over the rest -
/// `calcVelocity`, `calcPosition`, `incFrame` - is not ported.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Particle {
    /// `mOffsetPosition` (`0x10`), the position every ported callback moves.
    pub offset_position: Vec3,
    /// `mLocalPosition` (`0x1C`).
    pub local_position: Vec3,
    /// `mGlobalPosition` (`0x28`), which the model emitter's draw reads (`:137`) and which the
    /// ripple callback reads its water height through (`:1011`).
    pub global_position: Vec3,
    /// `mVelocity` (`0x34`), the direction the model emitter's draw aligns a model to (`:89`).
    pub velocity: Vec3,
    /// `mDrawParams::mAxis` (`JPAParticle.h:34`): the billboard's facing, which the model emitter's
    /// draw rebuilds in place and which the cross product is taken against (`:96`).
    pub axis: Vec3,
    /// `mDrawParams::mScaleX` (`0x10`) and `mScaleY` (`0x14`).
    pub scale_x: f32,
    pub scale_y: f32,
    /// `mDrawParams::mScaleOut` (`0x0C`) and `mAlphaOut` (`0x20`).
    pub scale_out: f32,
    pub alpha_out: f32,
    /// `mDrawParams::mPrmColor` (`0x2C`) and `mEnvColor` (`0x30`).
    pub prm_color: Color,
    pub env_color: Color,
    /// `mDrawParams::mRotateAngle` (`0x34`), the fixed-point roll the model emitter's draw reads
    /// through `S2DEG` (`:114`), and `mRotateSpeed` (`0x36`).
    pub rotate_angle: u16,
    pub rotate_speed: i16,
    /// `mDrawParams::mTexIdx` (`0x3A`).
    pub tex_idx: u16,
    /// `mpCallBack2` (`0xC8`).
    pub callback: Option<ParticleCallback>,
}

/// `cLib_chaseS` (`c_lib.cpp:259-273`): step `value` toward `target` by `step`, land on the target
/// and report true when the step reaches or passes it. A zero step only ever reports whether the
/// value is already there.
pub fn chase_s(value: &mut i16, target: i16, step: i16) -> bool {
    if step == 0 {
        return *value == target;
    }
    let step = if *value > target { -step } else { step };
    *value = value.wrapping_add(step);
    if step.wrapping_mul(*value - target) >= 0 {
        *value = target;
        return true;
    }
    false
}

/// `cXyz::isZero` (`c_xyz.cpp:141-145`): every component within [`ZERO_EPSILON`] of zero.
pub fn is_zero(value: Vec3) -> bool {
    value.x.abs() < ZERO_EPSILON && value.y.abs() < ZERO_EPSILON && value.z.abs() < ZERO_EPSILON
}

/// `cXyz::normZP` (`c_xyz.cpp:76-84`): a unit vector, or zero when the input is near zero.
pub fn norm_zp(value: Vec3) -> Vec3 {
    if value.length_squared() >= NEAR_ZERO_SQUARE {
        value.normalize()
    } else {
        Vec3::ZERO
    }
}

/// `dKyw_get_AllWind_vec` (`d_kankyo_wether.cpp:1284-1308`): the registered gust at a point and the
/// room wind blended, reduced to a direction and a magnitude. The gust's own share is scaled by
/// [`GUST_GAIN`] and the room wind's by whatever the room's strength leaves over; when the sum is
/// near zero the room wind's direction comes back unscaled.
pub fn all_wind_vec(weather: &Weather, pos: Vec3) -> (Vec3, f32) {
    let (gust_dir, gust_power) = pntwind_get_info(weather, pos);
    let room = weather.wind_vec() * (weather.wind_pow() * (1.0 - gust_power));
    let sum = room + gust_dir * (gust_power * GUST_GAIN);
    let power = sum.length();
    let direction = norm_zp(sum);
    if direction != Vec3::ZERO {
        (direction, power)
    } else {
        (weather.wind_vec(), power)
    }
}

/// `dPa_setWindPower` (`d_particle.cpp:447-455`): push a particle's offset position along the
/// blended wind, gaining [`WIND_POWER_GAIN`] over the wind's own magnitude.
pub fn set_wind_power(weather: &Weather, particle: &mut Particle) {
    let (wind, power) = all_wind_vec(weather, particle.offset_position);
    particle.offset_position += wind * (power * WIND_POWER_GAIN);
}

/// `dPa_windPcallBack::execute` (`d_particle.cpp:583-585`).
pub fn wind_particle_execute(weather: &Weather, particle: &mut Particle) {
    set_wind_power(weather, particle);
}

/// `dPa_smokePcallBack::execute` (`d_particle.cpp:458-461`): the wind push, gated on the emitter's
/// user work, which `dPa_smokeEcallBack::setup` set from the callback's own wind-off flag (`:360`).
pub fn smoke_particle_execute(weather: &Weather, emitter: &Emitter, particle: &mut Particle) {
    if emitter.user_work == 0 {
        set_wind_power(weather, particle);
    }
}

/// Which `end` a base-class call reaches. `dPa_followEcallBack::execute` (`d_particle.cpp:276`) and
/// `dPa_followEcallBack::setup` (`:295`) call `end()` unqualified, and it is virtual, so a smoke
/// emitter reaches the override rather than the base.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndKind {
    Follow,
    Smoke,
}

/// `dPa_followEcallBack` (`d/d_particle.h:43-70`), whose fields are laid out by the offsets that
/// header documents and not by reading order: `mpEmitter` at `0x04`, `mPos` at `0x08`, `mAngle` at
/// `0x0C`, `mFlag` at `0x10`, `mRateOff` at `0x11`, `field_0x12` at `0x12` and `field_0x13` at
/// `0x13`.
///
/// The constructor writes only four of those. `mFlag` and `mpEmitter` have no writer there
/// (`d_particle.cpp:258-263`); retail takes them from the zero-initialised static storage the
/// callback lives in, so they start clear here too. Nothing in this module writes bit one, which is
/// the follow suppression `execute` tests (`:267`); `field_0x12` is the switch `setFollowOff`
/// flips (`d/d_particle.h:58`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Follow {
    /// `mFlag` (`0x10`): bit zero is `end`, bit one suppresses the follow.
    pub flag: u8,
    /// `mRateOff` (`0x11`).
    pub rate_off: u8,
    /// `field_0x12` (`0x12`): clear for follow, set by `setFollowOff`.
    pub follow_off: u8,
    /// `field_0x13` (`0x13`): the one-shot latch `setup` arms the emitter against.
    pub immortal_taken: u8,
    /// `mpEmitter` (`0x04`), as an index into the emitter pool.
    pub emitter: Option<usize>,
    /// `mPos` (`0x08`): the point followed, copied rather than pointed at.
    pub pos: Option<Vec3>,
    /// `mAngle` (`0x0C`): the fixed-point rotation applied with the translation.
    pub angle: Option<[i16; 3]>,
}

impl Follow {
    /// `isEnd` (`d/d_particle.h:55`).
    pub fn is_end(&self) -> bool {
        self.flag & 1 != 0
    }

    /// `onEnd` (`d/d_particle.h:56`).
    pub fn on_end(&mut self) {
        self.flag |= 1;
    }

    /// `setFollowOff` (`d/d_particle.h:58`).
    pub fn set_follow_off(&mut self) {
        self.follow_off = 1;
    }
}

/// `dPa_followEcallBack::dPa_followEcallBack` (`d_particle.cpp:258-263`).
pub fn follow_new(rate_off: u8, follow_off: u8) -> Follow {
    Follow {
        rate_off,
        follow_off,
        immortal_taken: 0,
        ..Follow::default()
    }
}

/// The body of `dPa_followEcallBack::setup` from the latch onwards (`d_particle.cpp:296-306`): the
/// one-shot immortal latch and, when following is on, the continuous flag, the followed point, the
/// rotation and a cleared flag word.
///
/// Split from [`follow_setup`] and [`smoke_setup`] only because the retirement those two begin with
/// is virtual.
pub fn follow_adopt(
    callback: &mut Follow,
    emitters: &mut [Emitter],
    index: usize,
    pos: Vec3,
    angle: Option<[i16; 3]>,
) {
    if callback.immortal_taken == 0 {
        callback.emitter = Some(index);
        if let Some(emitter) = emitters.get_mut(index) {
            emitter.become_immortal();
        }
    }
    if callback.follow_off == 0 {
        if callback.rate_off != 0 {
            if let Some(emitter) = emitters.get_mut(index) {
                emitter.become_continuous_particle();
            }
        }
        callback.pos = Some(pos);
        callback.angle = angle;
        callback.flag = 0;
    }
}

/// `dPa_followEcallBack::end` (`d_particle.cpp:310-318`): invalidate the emitter, drop its
/// immortality and its callback, then raise the end flag. The early return on an emitter-less
/// callback is the guard, not a state change.
pub fn follow_end(callback: &mut Follow, emitters: &mut [Emitter]) {
    let Some(index) = callback.emitter else {
        return;
    };
    if let Some(emitter) = emitters.get_mut(index) {
        emitter.become_invalid();
        emitter.quit_immortal();
        emitter.callback = None;
    }
    callback.flag |= 1;
    callback.emitter = None;
}

/// `dPa_smokeEcallBack::end` (`d_particle.cpp:435-444`): the same teardown, but reinstalling the
/// shared smoke emitter callback rather than clearing it.
pub fn smoke_end(callback: &mut Smoke, emitters: &mut [Emitter], shared: &Smoke) {
    let Some(index) = callback.follow.emitter else {
        return;
    };
    if let Some(emitter) = emitters.get_mut(index) {
        emitter.become_invalid();
        emitter.quit_immortal();
        emitter.callback = Some(EmitterCallback::Smoke(Box::new(shared.clone())));
    }
    callback.follow.flag |= 1;
    callback.follow.emitter = None;
}

/// `dPa_followEcallBack::setup` (`d_particle.cpp:294-307`).
pub fn follow_setup(
    callback: &mut Follow,
    emitters: &mut [Emitter],
    index: usize,
    shared: &Smoke,
    pos: Vec3,
    angle: Option<[i16; 3]>,
) {
    let following = callback.emitter;
    retire_emitter(following, emitters, EndKind::Follow, shared, callback);
    follow_adopt(callback, emitters, index, pos, angle);
}

/// `dPa_smokeEcallBack::setup` (`d_particle.cpp:355-361`): the base follow setup, which retires
/// through the smoke override; the setup info kept for the draw; the shared smoke particle callback
/// installed; and the wind suppression published as the emitter's user work. The
/// `static dPa_smokePcallBack` the reference declares at `:356` is never used.
pub fn smoke_setup(
    callback: &mut Smoke,
    emitters: &mut [Emitter],
    index: usize,
    shared: &Smoke,
    pos: Vec3,
    angle: Option<[i16; 3]>,
    setup_info: i8,
) {
    let mut base = callback.follow;
    let following = base.emitter;
    retire_emitter(following, emitters, EndKind::Smoke, shared, &mut base);
    callback.follow = base;
    follow_adopt(&mut callback.follow, emitters, index, pos, angle);
    callback.setup_info = setup_info;
    if let Some(emitter) = emitters.get_mut(index) {
        emitter.particle_callback = Some(ParticleCallback::Smoke);
        emitter.user_work = u32::from(callback.wind_off);
    }
}

/// The retire the two setups and [`follow_execute`] all dispatch through, the way the unqualified
/// virtual `end()` call at `d_particle.cpp:276` and `:295` is dispatched. The `following` argument
/// is the callback's own emitter handle, which is the one the retail `end` acts on and which need
/// not be the emitter the caller was handed.
fn retire_emitter(
    following: Option<usize>,
    emitters: &mut [Emitter],
    kind: EndKind,
    shared: &Smoke,
    callback: &mut Follow,
) {
    if following.is_none() {
        return;
    }
    match kind {
        EndKind::Follow => follow_end(callback, emitters),
        EndKind::Smoke => {
            let mut smoke = shared.clone();
            smoke.follow = *callback;
            smoke_end(&mut smoke, emitters, shared);
            *callback = smoke.follow;
        }
    }
}

/// `dPa_smokeEcallBack` (`d/d_particle.h:72-97`) over its `dPa_followEcallBack` base: the extra
/// fields are `field_0x14` at `0x14`, `mWindOff` at `0x15`, `mColor` at `0x16` and `mTevstr` at
/// `0x1C`.
///
/// None of the three constructors (`d_particle.cpp:321-352`) writes `mpEmitter`, `mFlag` or
/// `field_0x13`; retail takes them from zero-initialised static storage, so they start clear here
/// too.
#[derive(Debug, Clone, PartialEq)]
pub struct Smoke {
    pub follow: Follow,
    /// `mColor` (`0x16`), the K1 colour the lighting block writes.
    pub color: Color,
    /// `mTevstr` (`0x1C`): the colour pair the lighting block is taken from instead of the
    /// environment light's actor colour.
    pub tev: Option<TevColors>,
    /// `field_0x14` (`0x14`): the setup info the caller passed, handed straight to the draw.
    pub setup_info: i8,
    /// `mWindOff` (`0x15`): the wind suppression the setup publishes as the emitter's user work.
    pub wind_off: u8,
}

impl Default for Smoke {
    fn default() -> Self {
        Smoke {
            follow: Follow::default(),
            color: Color::SMOKE,
            tev: None,
            setup_info: -1,
            wind_off: 0,
        }
    }
}

/// `dPa_smokeEcallBack::dPa_smokeEcallBack(u8)` (`d_particle.cpp:321-329`).
pub fn smoke_new(rate_off: u8) -> Smoke {
    Smoke {
        follow: follow_new(rate_off, 0),
        ..Smoke::default()
    }
}

/// `dPa_smokeEcallBack::dPa_smokeEcallBack(u8, u8, u8, u8)` (`d_particle.cpp:332-343`): the four
/// arguments are the rate flag, the follow switch, the flag word and the immortal latch, in that
/// order. `dPa_control_c::mSmokeEcallback` is built from `(0, 1, 1, 1)` (`d_particle.cpp:609`),
/// which is the shared instance [`Jpa::smoke_callback`] carries.
pub fn smoke_new_shared(rate_off: u8, follow_off: u8, flag: u8, immortal_taken: u8) -> Smoke {
    Smoke {
        follow: Follow {
            flag,
            rate_off,
            follow_off,
            immortal_taken,
            ..Follow::default()
        },
        ..Smoke::default()
    }
}

/// `dPa_smokeEcallBack::dPa_smokeEcallBack(const GXColor&, dKy_tevstr_c*, u8)`
/// (`d_particle.cpp:346-352`): the caller supplies the colour, the colour source and the rate flag,
/// and every other field takes the same default the one-argument form uses.
pub fn smoke_new_with(color: Color, tev: Option<TevColors>, rate_off: u8) -> Smoke {
    Smoke {
        follow: follow_new(rate_off, 0),
        color,
        tev,
        ..Smoke::default()
    }
}

/// `smokeEcallBack` (`d_particle.cpp:375-427`) reduced to the state it computes, with the emitter's
/// z-draw test as the gate and the colour the callback passes in taking its alpha from the emitter's
/// global alpha.
///
/// Left a no-op past the lighting block, which needs the tev-chain pass [`Lighting`] names. The
/// camera normal matrix the draw loads before anything else (`:377`) is a renderer-internal matrix
/// this port has no slot for, and the light direction it carries belongs to the environment light,
/// which `weather.rs` does not expose either.
pub fn smoke_draw_state(
    emitter: &Emitter,
    tev: Option<TevColors>,
    actor: ActorLight,
    mut color: Color,
) -> Option<Lighting> {
    if emitter.z_draw {
        return None;
    }
    let (dif, amb) = match tev {
        Some(tev) => (tev.color_c0, tev.color_k0),
        None => actor_lighting(actor),
    };
    color.a = emitter.global_prm_color.a;
    Some(initiate_lighting(dif, amb, color))
}

/// `dPa_smokeEcallBack::draw` (`d_particle.cpp:430-432`): the draw is the lighting block above with
/// this callback's own colour and its own colour source. The setup info it also passes is read by
/// nothing there.
pub fn smoke_draw(callback: &Smoke, emitter: &Emitter, actor: ActorLight) -> Option<Lighting> {
    smoke_draw_state(emitter, callback.tev, actor, callback.color)
}

/// The emitter callbacks, so a follow's `end` can reinstall the shared smoke callback and a simple
/// emitter's `executeAfter` can find its own registry entry.
#[derive(Debug, Clone, PartialEq)]
pub enum EmitterCallback {
    /// `dPa_followEcallBack`.
    Follow(Box<Follow>),
    /// `dPa_smokeEcallBack`.
    Smoke(Box<Smoke>),
    /// `dPa_simpleEcallBack`, as an index into [`Jpa::simple`].
    Simple(usize),
}

/// A resolution the emitter-creation path could not make. Retail warns and returns no emitter
/// (`JPAEmitterManager.cpp:52-78`); the port records the ids here for the shell instead of
/// creating a stand-in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JpaGap {
    /// `createSimpleEmitterID`'s `pResMgrArray[rmID]` is NULL: no archive is mounted in the slot
    /// the effect id selects (`JPAEmitterManager.cpp:55` asserts it is not). A scene id has no
    /// manager until the stage's `Pscene###.jpc` is mounted (`d_particle.cpp:744`).
    ManagerNotMounted {
        group_id: u8,
        rm_id: u8,
        user_id: u16,
    },
    /// The manager is mounted and `getByUserIndex(user_id)` answered NULL
    /// (`JPAEmitterManager.cpp:61-66`): the archive does not carry the id.
    UnknownResource {
        group_id: u8,
        rm_id: u8,
        user_id: u16,
    },
}

impl std::fmt::Display for JpaGap {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ManagerNotMounted {
                group_id,
                rm_id,
                user_id,
            } => write!(
                formatter,
                "particle resource {user_id:#06x} needs resource manager {rm_id}, which is not mounted (group {group_id})"
            ),
            Self::UnknownResource {
                group_id,
                rm_id,
                user_id,
            } => write!(
                formatter,
                "particle resource {user_id:#06x} is not in resource manager {rm_id} (group {group_id})"
            ),
        }
    }
}

/// `JPAEmitterManager::createSimpleEmitterID` (`JPAEmitterManager.cpp:52-78`) up to the callback
/// installation: resolve `user_id` in the bank slot `rm_id` names, claim a free pool slot, make
/// the emitter from the resolved resource and place it at `pos` in `group_id`.
///
/// A miss creates nothing and records a [`JpaGap`], which is retail's warn-and-return-NULL arm
/// (`:63-66`). The bank is the only source of resources: the [`PORTED_SMOKE_IDS`] table chooses
/// what an emitter that exists does, never whether it exists.
///
/// The slot is rebuilt from [`Emitter::default`] before its ids are written. Retail's pool
/// objects are constructed once and `create` re-initialises the fields the resource drives; the
/// port's pool recycles values, so the construction the type default stands in for is what makes
/// a reclaimed slot match a fresh one.
pub fn create_emitter_id(
    emitters: &mut [Emitter],
    bank: &JpaBank,
    gaps: &mut Vec<JpaGap>,
    pos: Vec3,
    user_id: u16,
    group_id: u8,
    rm_id: u8,
) -> Option<usize> {
    let Some(manager) = bank.manager(rm_id) else {
        gaps.push(JpaGap::ManagerNotMounted {
            group_id,
            rm_id,
            user_id,
        });
        return None;
    };
    let Some((resource_index, resource)) = manager.find_by_user_index(user_id) else {
        gaps.push(JpaGap::UnknownResource {
            group_id,
            rm_id,
            user_id,
        });
        return None;
    };
    let index = (0..emitters.len()).find(|index| !emitters[*index].active)?;
    let emitter = &mut emitters[index];
    *emitter = Emitter::default();
    emitter.active = true;
    emitter.res_id = resource.user_index;
    emitter.res_mgr_id = rm_id;
    emitter.group_id = group_id;
    emitter.resource = Some(JpaResourceRef {
        rm_id,
        index: resource_index,
        user_index: resource.user_index,
    });
    emitter.global_translation = pos;
    Some(index)
}

/// `JPABaseEmitter` (`JPAEmitter.h:163-350`) narrowed to the fields the ported callbacks read and
/// write, plus the active particle list they walk. The emission, key, field and draw integration
/// the base class performs over the rest is not ported.
#[derive(Debug, Clone, PartialEq)]
pub struct Emitter {
    /// `mGroupID`, read by the follow and simple draws and set by the emitter factory.
    pub group_id: u8,
    /// `mResID` and `mResMgrID`: the effect id the emitter was created from, which is also the key
    /// `dPa_control_c::getSimple` searches on (`:1008-1014`), and the resource-manager selector
    /// its top bit picks (`:699-701`).
    pub res_id: u16,
    pub res_mgr_id: u8,
    /// `mpDataLinkInfo` (`JPAEmitter.h:410`), the link info `create` was handed
    /// (`JPAEmitter.cpp:134`), reduced to where the resource came from in the bank: the manager
    /// slot, the registration index and the user index. `None` for an emitter no bank resource
    /// created; this port never manufactures one as a substitute.
    pub resource: Option<JpaResourceRef>,
    /// `mFlags` (`JPAEmitter.h:196-199`), the [`EMITTER_*`] bits.
    pub flags: u32,
    /// `mMaxFrame` (`JPAEmitter.h:254`), the frame count `becomeInvalidEmitter` and
    /// `becomeContinuousParticle` set to `-1` and `0` (`:287-292`).
    pub max_frame: i32,
    /// `mRate` (`JPAEmitter.h:272`) and `mSpread` (`:277`), which `setSimpleLand` sets at
    /// `d_particle.cpp:879-880`.
    pub rate: f32,
    pub spread: f32,
    /// `mInitialVelDir` (`JPAEmitter.h:274`). No callback in this module writes it; it is carried
    /// because the model draw reads the emitter's global fields.
    pub directional_speed: f32,
    /// `mGlobalTranslation` (`JPAEmitter.h:225`).
    pub global_translation: Vec3,
    /// `mGlobalRotation` (`JPAEmitter.h:221-223`): retail converts the fixed-point triple through
    /// `JPAGetXYZRotateMtx` into a rotation matrix. Nothing in this module reads it back, so the
    /// triple is what is carried and the conversion is not reproduced.
    pub global_rotation: [i16; 3],
    /// `mGlobalRotation` (`JPAEmitter.h:414`) as `setGlobalSRTMatrix` writes it: the rotation
    /// half of the decomposed matrix (`JPAMath.cpp:115-137`). The two writers above and below
    /// store different spellings of the one retail field, and nothing reads either back.
    pub global_rotation_matrix: Option<Mat3>,
    /// `mGlobalDynamicsScale` (`JPAEmitter.h:242`).
    pub global_dynamics_scale: Vec3,
    /// `mGlobalParticleScale` (`JPAEmitter.h:234`), the scale the model draw multiplies a particle's
    /// own scale by (`:143`).
    pub global_particle_scale: Vec3,
    /// `mGlobalPrmColor` (`JPAEmitter.h:255-259`). Its alpha is the global alpha the follow and the
    /// simple callbacks both read and write (`:213-214`).
    pub global_prm_color: Color,
    /// `mGlobalEnvColor` (`JPAEmitter.h:260-264`).
    pub global_env_color: Color,
    /// `mUserData` (`JPAEmitter.h:319`), which the smoke setup publishes the wind suppression
    /// through and the smoke particle callback gates on.
    pub user_work: u32,
    /// `getCurrentCreateNumber` (`JPAEmitter.h:345-347`): how many particles the emitter's volume
    /// pass decided to create this frame. Its only writer is `calcCreatePtcls`
    /// (`JPAEmitter.cpp:251-282`), which reads `mRate`, `mRateRndm`, `mDivNumber` and `mVolumeType`
    /// out of the packed emitter data this port does not read, so it is not advanced here and is
    /// not assumed to be zero: a simple emitter drains only on a frame some other system counted.
    pub current_create_number: i32,
    /// `getFrame` (`JPAEmitter.h:321`), the tick `calc` advances at `JPAEmitter.cpp:243`.
    pub frame: i32,
    /// `mDraw.isZDraw` (`JPAEmitter.h:308`), the gate the whole smoke lighting block hangs off.
    pub z_draw: bool,
    /// `mpEmitterCallBack` (`JPAEmitter.h:294-296`) and `mpParticleCallBack` (`:297-299`).
    pub callback: Option<EmitterCallback>,
    pub particle_callback: Option<ParticleCallback>,
    /// `getParticleList` (`JPAEmitter.h:302`): the active particle list the model emitter's draw
    /// walks, as indices into [`Jpa::particles`].
    pub particles: Vec<usize>,
    /// `setEmitterTranslation` and `setEmitterScale` (`JPAEmitter.h:245-250`), the local frame the
    /// global translation is measured against by the base class's own position pass.
    pub emitter_translation: Vec3,
    pub emitter_scale: Vec3,
    /// Whether the slot is in use.
    pub active: bool,
}

impl Default for Emitter {
    fn default() -> Self {
        Emitter {
            group_id: GROUP_NORMAL,
            res_id: 0,
            res_mgr_id: 0,
            resource: None,
            flags: 0,
            max_frame: 0,
            rate: 0.0,
            spread: 0.0,
            directional_speed: 0.0,
            global_translation: Vec3::ZERO,
            global_rotation: [0; 3],
            global_rotation_matrix: None,
            global_dynamics_scale: Vec3::ONE,
            global_particle_scale: Vec3::ONE,
            global_prm_color: Color::new(0xff, 0xff, 0xff, 0xff),
            global_env_color: Color::new(0xff, 0xff, 0xff, 0xff),
            user_work: 0,
            current_create_number: 0,
            frame: 0,
            z_draw: false,
            callback: None,
            particle_callback: None,
            particles: Vec::new(),
            emitter_translation: Vec3::ZERO,
            emitter_scale: Vec3::ONE,
            active: false,
        }
    }
}

impl Emitter {
    /// `becomeImmortalEmitter` (`JPAEmitter.h:284`).
    pub fn become_immortal(&mut self) {
        self.flags |= EMITTER_IMMORTAL;
    }

    /// `quitImmortalEmitter` (`JPAEmitter.h:285`).
    pub fn quit_immortal(&mut self) {
        self.flags &= !EMITTER_IMMORTAL;
    }

    /// `becomeContinuousParticle` (`JPAEmitter.h:287`).
    pub fn become_continuous_particle(&mut self) {
        self.max_frame = 0;
    }

    /// `becomeInvalidEmitter` (`JPAEmitter.h:289-292`).
    pub fn become_invalid(&mut self) {
        self.max_frame = -1;
        self.stop_create_particle();
    }

    /// `stopCreateParticle` (`JPAEmitter.h:279`).
    pub fn stop_create_particle(&mut self) {
        self.flags |= EMITTER_STOP_EMIT;
    }

    /// `playCreateParticle` (`JPAEmitter.h:280`).
    pub fn play_create_particle(&mut self) {
        self.flags &= !EMITTER_STOP_EMIT;
    }

    /// `stopDrawParticle` (`JPAEmitter.h:281`).
    pub fn stop_draw_particle(&mut self) {
        self.flags |= EMITTER_STOP_DRAW;
    }

    /// `playDrawParticle` (`JPAEmitter.h:282`).
    pub fn play_draw_particle(&mut self) {
        self.flags &= !EMITTER_STOP_DRAW;
    }

    /// `getParticleNumber` (`JPAEmitter.h:203-205`).
    pub fn particle_number(&self) -> usize {
        self.particles.len()
    }

    /// `isEnableDeleteEmitter` (`JPAEmitter.h:211`).
    pub fn is_enable_delete_emitter(&self) -> bool {
        self.flags & EMITTER_ENABLE_DELETE != 0 && self.particle_number() == 0
    }

    /// `doTerminationProcess` (`JPAEmitter.cpp:446-460`). This is not one of the ported callbacks,
    /// but it is the only writer of the deletion flag [`Emitter::is_enable_delete_emitter`] reads,
    /// so without it the follow callback's retire branch and the model control's retirement branch
    /// can never be taken. A `max_frame` of `-1` is what [`Emitter::become_invalid`] leaves and it
    /// reports the emitter drained; a positive one reports drained only once the tick has reached
    /// it and the emitter is not immortal.
    pub fn do_termination_process(&mut self) -> bool {
        if self.max_frame == 0 {
            false
        } else if self.max_frame < 0 {
            self.flags |= EMITTER_ENABLE_DELETE;
            self.particle_number() == 0
        } else if self.frame >= self.max_frame {
            self.flags |= EMITTER_ENABLE_DELETE;
            if self.flags & EMITTER_IMMORTAL != 0 {
                return false;
            }
            self.particle_number() == 0
        } else {
            false
        }
    }

    /// `setGlobalAlpha` (`JPAEmitter.h:214`), which writes the primary colour's alpha.
    pub fn set_global_alpha(&mut self, alpha: i16) {
        self.global_prm_color.a = alpha as u8;
    }

    /// `setGlobalSRTMatrix` (`JPAEmitter.h:218-220`): `JPASetRMtxSTVecfromMtx`
    /// (`JPAMath.cpp:115-137`) stores the 3x3 column norms as the dynamics scale, the normalised
    /// columns as the rotation matrix and the last column as the translation - exactly what
    /// [`Mat4::to_scale_rotation_translation`] answers. It does not touch the particle scale,
    /// which only the `setGlobalScale` spelling writes (`JPAEmitter.h:229-233`).
    pub fn set_global_srt_matrix(&mut self, matrix: Mat4) {
        let (scale, rotation, translation) = matrix.to_scale_rotation_translation();
        self.global_dynamics_scale = scale;
        self.global_rotation_matrix = Some(Mat3::from_quat(rotation));
        self.global_translation = translation;
    }

    /// `getGlobalAlpha` (`JPAEmitter.h:213`), widened from the byte the field actually is.
    pub fn global_alpha(&self) -> i16 {
        i16::from(self.global_prm_color.a)
    }

    /// `dPa_control_c::getRM_ID` (`d_particle.cpp:699-701`): the resource-manager selector the
    /// effect id's top bit names.
    pub fn resource_manager_id(res_id: u16) -> u8 {
        ((res_id >> 15) & 0x01) as u8
    }
}

/// `dPa_followEcallBack::execute` (`d_particle.cpp:266-284`): carry the emitter to the followed
/// point and rotation unless following is switched off or suppressed, retire it once it is both
/// marked for deletion and empty, and then fade the global alpha toward [`FADE_ALPHA_TARGET`].
pub fn follow_execute(
    callback: &mut Follow,
    emitters: &mut [Emitter],
    index: usize,
    kind: EndKind,
    shared: &Smoke,
) {
    let follow = callback.follow_off == 0 && callback.flag & 2 == 0;
    if follow {
        let pos = callback.pos.unwrap_or(Vec3::ZERO);
        let angle = callback.angle;
        if let Some(emitter) = emitters.get_mut(index) {
            emitter.global_translation = pos;
            if let Some(angle) = angle {
                emitter.global_rotation = angle;
            }
        }
    }
    let drain = emitters
        .get(index)
        .is_some_and(|emitter| emitter.is_enable_delete_emitter());
    if drain {
        let following = callback.emitter;
        retire_emitter(following, emitters, kind, shared, callback);
    }
    if callback.flag & 1 != 0 {
        let mut alpha = emitters.get(index).map_or(0, Emitter::global_alpha);
        chase_s(&mut alpha, FADE_ALPHA_TARGET, FADE_ALPHA_STEP);
        if let Some(emitter) = emitters.get_mut(index) {
            emitter.set_global_alpha(alpha);
        }
    }
}

/// `dPa_followEcallBack::draw` (`d_particle.cpp:287-291`): a projection-group emitter takes its
/// alpha from the register alone.
///
/// Left a no-op. It needs a per-draw tev-stage override on a draw whose alpha a renderer with one
/// fixed path cannot express:
/// `GXSetTevAlphaIn(GX_TEVSTAGE0, GX_CA_ZERO, GX_CA_ZERO, GX_CA_ZERO, GX_CA_A0)`.
pub fn follow_draw(_emitter: &Emitter) {}

/// `dPa_simpleData_c` (`d/d_particle.h:25-34`), the queued spawn the simple emitter writes and
/// `executeAfter` drains: `mPos` at `0x00`, `mPrmColor` at `0x0C`, `mEnvColor` at `0x10` and
/// `mbAffectedByWind` at `0x13`.
///
/// `dPa_simpleData_c::dPa_simpleData_c` (`d_particle.cpp:480`) writes nothing, so retail's entries
/// are clear until a `set` fills them.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SimpleData {
    pub pos: Vec3,
    /// `mPrmColor`: the three bytes are the emitter's primary colour and the fourth is the global
    /// alpha `set` stores alongside them (`d_particle.cpp:570-573`).
    pub prm_color: Color,
    /// `mEnvColor` (`0x10`), three bytes wide.
    pub env_color: [u8; 3],
    /// `mbAffectedByWind` (`0x13`).
    pub affected_by_wind: u8,
}

/// `dPa_simpleEcallBack` (`d/d_particle.h:99-119`) by offset: `mpBaseEmitter` at `0x004`, `mResID` at
/// `0x008`, `mGrpID` at `0x00A`, `mbIsSmoke` at `0x00B`, `mCount` at `0x00C` and `mSimpleData` at
/// `0x010`.
///
/// The constructor writes `mpBaseEmitter` and `mCount` only (`d_particle.cpp:474-477`).
/// `mResID`, `mGrpID` and `mbIsSmoke` have no writer until `create` (`:594-616`), and `create`
/// leaves `mbIsSmoke` alone when the emitter it wanted could not be made (`:598-611`), so it is an
/// [`Option`] here rather than a flag assumed false.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Simple {
    pub base_emitter: Option<usize>,
    pub res_id: u16,
    pub group_id: u8,
    pub is_smoke: Option<bool>,
    /// `mCount` (`0x0C`): how many queued spawns are left to drain.
    pub count: i16,
    /// `mSimpleData` (`0x010`).
    pub data: [SimpleData; SIMPLE_DATA],
}

/// `dPa_simpleEcallBack::dPa_simpleEcallBack` (`d_particle.cpp:474-477`).
pub fn simple_new() -> Simple {
    Simple::default()
}

/// `dPa_simpleData_c::dPa_simpleData_c` (`d_particle.cpp:480`).
pub fn simple_data_new() -> SimpleData {
    SimpleData::default()
}

/// `dPa_simpleEcallBack::set` (`d_particle.cpp:564-580`): queue one spawn, taking the alpha as the
/// primary colour's own alpha, and refuse once the queue is full.
pub fn simple_set(
    callback: &mut Simple,
    pos: Vec3,
    alpha: u8,
    prm: Color,
    env: Color,
    affected_by_wind: i32,
) -> bool {
    if callback.count >= SIMPLE_DATA as i16 {
        return false;
    }
    let index = callback.count as usize;
    callback.data[index] = SimpleData {
        pos,
        prm_color: Color::new(prm.r, prm.g, prm.b, alpha),
        env_color: [env.r, env.g, env.b],
        affected_by_wind: affected_by_wind as u8,
    };
    callback.count += 1;
    true
}

/// `dPa_simpleEcallBack::createEmitter` (`d_particle.cpp:618-630`): make the emitter at the
/// origin with the resource manager the effect id's top bit selects, install this callback, keep
/// it emitting for as long as the host says, and stop its own emission until the drain runs.
///
/// The manager call is [`create_emitter_id`]'s bank lookup. An id the loaded bank cannot resolve
/// leaves the callback exactly as the reference does on a null return - no emitter, no callback,
/// no frame limit and no stop - and records a [`JpaGap`].
pub fn simple_create_emitter(
    callback: &mut Simple,
    emitters: &mut [Emitter],
    bank: &JpaBank,
    gaps: &mut Vec<JpaGap>,
    slot: usize,
) -> Option<usize> {
    if callback.base_emitter.is_some() {
        return callback.base_emitter;
    }
    let index = create_emitter_id(
        emitters,
        bank,
        gaps,
        Vec3::ZERO,
        callback.res_id,
        callback.group_id,
        Emitter::resource_manager_id(callback.res_id),
    )?;
    let emitter = &mut emitters[index];
    emitter.callback = Some(EmitterCallback::Simple(slot));
    emitter.max_frame = 0;
    emitter.stop_create_particle();
    callback.base_emitter = Some(index);
    callback.base_emitter
}

/// `dPa_simpleEcallBack::create` (`d_particle.cpp:594-616`): clear the emitter pointer, adopt the
/// group and the effect id, create the emitter, then arm the smoke draw for the two effect ids in
/// [`PORTED_SMOKE_IDS`] and install the shared smoke particle callback.
///
/// A resource the bank cannot resolve refuses the whole entry: the callback keeps no emitter, and
/// `dPa_control_c::newSimple` does not spend a registry slot on it (`d_particle.cpp:984-996`
/// increments `mNumSimple` only after `create` answers non-NULL).
pub fn simple_create(
    callback: &mut Simple,
    emitters: &mut [Emitter],
    bank: &JpaBank,
    gaps: &mut Vec<JpaGap>,
    slot: usize,
) -> Option<usize> {
    callback.base_emitter = None;
    let emitter = simple_create_emitter(callback, emitters, bank, gaps, slot)?;
    if is_ported_smoke_id(callback.res_id) {
        callback.is_smoke = Some(true);
        emitters[emitter].particle_callback = Some(ParticleCallback::Smoke);
    } else {
        callback.is_smoke = Some(false);
    }
    Some(emitter)
}

/// `dPa_simpleEcallBack::executeAfter` (`d_particle.cpp:483-513`): drain the queue. With nothing to
/// create this frame the queue is simply emptied; otherwise the emitter is un-stopped, and for each
/// queued spawn that survives a [`SPAWN_CLIP_RADIUS`] sphere test against the view frustum the
/// emitter is moved and recoloured there and [`Emitter::current_create_number`] particles are
/// created at the point, each taking the wind callback when its spawn asked for one.
///
/// The queue drains either way: the count and the cursor advance whether or not the spawn survived.
pub fn simple_execute_after(
    callback: &mut Simple,
    emitter: &mut Emitter,
    particles: &mut Particles,
    clip: Option<&Clip>,
) {
    let create_count = emitter.current_create_number;
    if create_count <= 0 {
        callback.count = 0;
        emitter.stop_create_particle();
        return;
    }
    emitter.play_create_particle();
    let mut index = 0usize;
    while callback.count != 0 {
        let data = callback.data[index];
        let visible = match clip {
            Some(clip) => !clip.clip(data.pos, SPAWN_CLIP_RADIUS),
            None => true,
        };
        if visible {
            emitter.global_translation = data.pos;
            emitter.global_prm_color.r = data.prm_color.r;
            emitter.global_prm_color.g = data.prm_color.g;
            emitter.global_prm_color.b = data.prm_color.b;
            emitter.global_prm_color.a = data.prm_color.a;
            emitter.global_env_color.r = data.env_color[0];
            emitter.global_env_color.g = data.env_color[1];
            emitter.global_env_color.b = data.env_color[2];
            for _ in 0..create_count {
                let Some(particle) = particles.create(emitter) else {
                    break;
                };
                particle.offset_position = data.pos;
                if data.affected_by_wind != 0 {
                    particle.callback = Some(ParticleCallback::Wind);
                }
            }
        }
        callback.count -= 1;
        index += 1;
    }
    emitter.stop_create_particle();
}

/// `dPa_simpleEcallBack::draw` (`d_particle.cpp:516-522`): a smoke-flagged emitter takes the same
/// lighting block with the shared colour and no colour source of its own, and a projection-group
/// emitter takes its alpha from the register alone.
///
/// Left a no-op past the lighting block. The projection arm needs the same per-draw tev-stage
/// override [`follow_draw`] names.
pub fn simple_draw(
    callback: &Simple,
    emitter: &Emitter,
    actor: ActorLight,
) -> (Option<Lighting>, bool) {
    let lighting = match callback.is_smoke {
        Some(true) => smoke_draw_state(emitter, None, actor, Color::SMOKE),
        _ => None,
    };
    (lighting, emitter.group_id == GROUP_PROJECTION)
}

/// `J3DUClipper` (`J3DUClipper.cpp:29-70`) over the camera the game publishes: the four frustum
/// planes, rebuilt from the vertical field of view, the aspect, the near plane and the far plane
/// the camera's own setup hands it (`d_camera.cpp:5340-5347`).
///
/// The far plane there is the stage's cull point rather than the view's while the scope is at its
/// initial state (`:5341-5345`), and the stage cull point has no source in this port, so the view's
/// own far plane is what is used.
#[derive(Debug, Clone, PartialEq)]
pub struct Clip {
    view: Mat4,
    planes: [Vec4; 4],
    near: f32,
    far: f32,
}

impl Clip {
    /// `mDoLib_clipper::setup` (`m_Do_lib.cpp:39-49`) and `J3DUClipper::calcViewFrustum`
    /// (`J3DUClipper.cpp:30-49`). The vertical field of view arrives in radians, because that is
    /// what the view carries, and retail multiplies its degree value by the radian constant.
    pub fn new(view: &View) -> Self {
        let tan = (view.fovy.to_degrees() * 0.5 * DEG2RAD).tan();
        let near_y = view.near * tan;
        let near_x = view.aspect * near_y;
        let corner0 = Vec3::new(-near_x, -near_y, -view.near);
        let corner1 = Vec3::new(-near_x, near_y, -view.near);
        let corner2 = Vec3::new(near_x, near_y, -view.near);
        let corner3 = Vec3::new(near_x, -near_y, -view.near);
        let planes = [
            corner1.cross(corner0),
            corner2.cross(corner1),
            corner3.cross(corner2),
            corner0.cross(corner3),
        ]
        .map(|plane| {
            let plane = plane.normalize();
            Vec4::new(plane.x, plane.y, plane.z, 0.0)
        });
        Clip {
            view: view.view,
            planes,
            near: view.near,
            far: view.far,
        }
    }

    /// `J3DUClipper::clip(const Mtx, Vec, f32)` (`J3DUClipper.cpp:52-70`): reject the sphere when it
    /// is wholly outside the near plane, wholly outside the far plane, or wholly outside any of the
    /// four side planes.
    pub fn clip(&self, pos: Vec3, radius: f32) -> bool {
        let p = self.view.transform_point3(pos);
        if -p.z < self.near - radius {
            return true;
        }
        if -p.z > self.far + radius {
            return true;
        }
        self.planes
            .iter()
            .any(|plane| plane.x.mul_add(p.x, plane.y.mul_add(p.y, plane.z * p.z)) > radius)
    }

    /// `J3DUClipper::clip(const Mtx, Vec*, Vec*)` (`J3DUClipper.cpp:73-148`): the eight corners of
    /// `min`/`max` in the order retail builds them, each mapped through the model matrix onto the
    /// view, rejected when every corner is outside one plane. A corner inside all six planes ends
    /// the test with `false` first, which is retail's `if (!any) return FALSE;`. The model matrix
    /// is the caller's `fopAcM_GetMtx` (`f_op_actor_mng.cpp:594-599` concatenates it onto the
    /// view before the walk).
    pub fn clip_box(&self, model: Mat4, min: Vec3, max: Vec3) -> bool {
        let m = self.view * model;
        let corners = [
            Vec3::new(max.x, max.y, min.z),
            Vec3::new(max.x, max.y, max.z),
            Vec3::new(min.x, max.y, max.z),
            Vec3::new(min.x, max.y, min.z),
            Vec3::new(max.x, min.y, min.z),
            Vec3::new(max.x, min.y, max.z),
            Vec3::new(min.x, min.y, max.z),
            Vec3::new(min.x, min.y, min.z),
        ];
        let mut clip = [0_u32; 6];
        for corner in corners {
            let p = m.transform_point3(corner);
            let mut any = false;
            if -p.z < self.near {
                clip[4] += 1;
                any = true;
            }
            if -p.z > self.far {
                clip[5] += 1;
                any = true;
            }
            for (slot, plane) in self.planes.iter().enumerate() {
                if plane.x.mul_add(p.x, plane.y.mul_add(p.y, plane.z * p.z)) > 0.0 {
                    clip[slot] += 1;
                    any = true;
                }
            }
            if !any {
                return false;
            }
        }
        clip.iter().any(|count| *count == 8)
    }
}

/// The particle pool the emitter manager allocates once and recycles for the whole game
/// (`JPAEmitterManager.cpp:27-32`, sized at `d_particle.cpp:649`). A create past the pool's end
/// hands back nothing, which is the null the drain's `break` tests (`d_particle.cpp:498`).
#[derive(Resource, Debug, PartialEq)]
pub struct Particles {
    pub items: Vec<Particle>,
    free: Vec<usize>,
}

impl Default for Particles {
    fn default() -> Self {
        Particles {
            items: vec![Particle::default(); PARTICLE_POOL],
            free: (0..PARTICLE_POOL).rev().collect(),
        }
    }
}

impl Particles {
    /// `createParticle` through the vacant list: take a particle, hand its slot back and append it
    /// to the emitter's active list.
    pub fn create(&mut self, emitter: &mut Emitter) -> Option<&mut Particle> {
        let index = self.free.pop()?;
        emitter.particles.push(index);
        Some(&mut self.items[index])
    }

    /// `deleteParticle` (`JPAEmitter.h:186`) back onto the vacant list.
    pub fn remove(&mut self, emitter: &mut Emitter, index: usize) -> Option<Particle> {
        let slot = emitter.particles.iter().position(|slot| *slot == index)?;
        emitter.particles.remove(slot);
        self.free.push(index);
        Some(self.items[index])
    }

    /// `JPABaseEmitter::deleteAllParticle` (`JPAEmitter.cpp:403-410`): every active particle goes
    /// back onto the vacant list. The child-particle list the reference sweeps next (`:412-421`)
    /// has no counterpart here - the port's emitter holds one list - and the field data it clears
    /// per particle is not modelled.
    pub fn delete_all(&mut self, emitter: &mut Emitter) {
        for slot in emitter.particles.drain(..) {
            self.free.push(slot);
        }
    }
}

/// `dPa_J3Dmodel_c` (`d/d_particle.h:262-269`): `mInit` at `0x00` and `mModel` at `0x04`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ModelSlot {
    /// `mInit`: claimed for this frame or free. `dPa_J3Dmodel_c`'s constructor sets it to zero
    /// (`d_particle.cpp:183-185`) and the control's draw clears the whole pool every frame
    /// (`:236-237`).
    pub init: u8,
    /// `mModel`: the model the slot owns. Retail builds one per slot in the control's constructor
    /// (`d_particle.cpp:175`); nothing in this port builds models, so it is the index of one the
    /// host owns or nothing at all.
    pub model: Option<usize>,
}

/// `node_class` (`c_node.h:6-11`), the three links a `dPa_modelEmitter_c` clears in its
/// constructor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ModelNode {
    pub prev: Option<usize>,
    pub data: Option<usize>,
    pub next: Option<usize>,
}

/// The per-particle transform `dPa_J3DmodelEmitter_c::draw` computes (`d_particle.cpp:88-157`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModelTransform {
    /// `sp44`: columns zero to two are the facing frame, and the fourth component of each is the
    /// particle's global position (`:104-112`, `:138-140`).
    pub mtx: Mat4,
    /// The emitter's global particle scale carrying the particle's own `mScaleX` on the x and z
    /// axes and `mScaleY` on the y (`:142-147`).
    pub scale: Vec3,
    /// `vel`: the particle's velocity made a unit direction, or straight up when it is zero
    /// (`:89-94`).
    pub velocity_dir: Vec3,
    /// `tmp`: the normalised cross product of the old axis and the velocity, or the z axis when
    /// that is zero (`:96-101`).
    pub cross: Vec3,
    /// `params->mAxis` after the draw rebuilt it (`:102-103`), which is written back into the
    /// particle.
    pub axis: Vec3,
}

/// `dPa_J3DmodelEmitter_c` (`d/d_particle.h:245-260`) by offset: `mpBaseEmitter` at `0x10`,
/// `mpHeap` at `0x14`, `modelData` at `0x18`, `field_0x1c` at `0x1C`, `field_0x20` at `0x20`,
/// `field_0x24` at `0x24` and `field_0x26` at `0x26`.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelEmitter {
    /// `node_class`, which `dPa_modelEmitter_c`'s constructor clears (`:29-31`).
    pub node: ModelNode,
    /// `mpBaseEmitter` (`0x10`), as an index into the emitter pool.
    pub base_emitter: Option<usize>,
    /// `modelData` (`0x18`) and `field_0x1c` (`0x1C`): the model and its texture-pattern animator,
    /// as indices into whatever the host owns. Neither is allocated here.
    pub model_data: Option<usize>,
    pub tex_pattern: Option<usize>,
    /// `J3DAnmTexPattern::getUpdateMaterialNum` (`d_particle.cpp:44`): how many texture-index
    /// entries the pattern animates, and so the size of the solid heap and of the entry array. It
    /// comes out of the packed animator this port does not read, so the caller supplies it.
    pub update_material_num: u16,
    /// `field_0x20`: the texture-index array, each entry's own index set to its own position in the
    /// constructor's loop (`:51-55`). The material ids the pattern then searched for live in the
    /// animator (`:49`), so they are not reproduced.
    pub anm_index: Vec<u16>,
    /// `field_0x24`: the frame the pattern is set to at draw time (`:152`). The constructor writes
    /// it only when the entry array allocated (`:56`), so it stays absent otherwise.
    pub anm_frame: Option<u16>,
    /// `field_0x26`: the axis the roll is taken about - zero for y, one for x, two for z, three for
    /// the diagonal (`:116-130`).
    pub rotate_axis: u8,
    /// `tevStr` (`0x5C`), the colour pair the draw writes onto each model.
    pub tev: TevColors,
    /// The per-particle transforms the last draw computed, in list order.
    pub transforms: Vec<ModelTransform>,
}

/// `dPa_modelEmitter_c::dPa_modelEmitter_c` (`d_particle.cpp:29-31`): the forced node clear empties
/// all three links (`c_node.cpp:111-115`).
pub fn model_emitter_new() -> ModelEmitter {
    ModelEmitter {
        node: ModelNode::default(),
        base_emitter: None,
        model_data: None,
        tex_pattern: None,
        update_material_num: 0,
        anm_index: Vec::new(),
        anm_frame: None,
        rotate_axis: 0,
        tev: TevColors::default(),
        transforms: Vec::new(),
    }
}

/// `dPa_J3DmodelEmitter_c::dPa_J3DmodelEmitter_c` (`d_particle.cpp:34-63`): record the emitter, the
/// model, the rotation axis and the colour pair; when a texture-pattern animator is present, size
/// the solid heap from its updated-material count, fill the per-material entry array with each
/// entry's own index, adopt the frame, restore the heap and adjust it; and finally make the emitter
/// immortal.
///
/// `update_material_num` stands in for `getUpdateMaterialNum`, which reads the packed animator
/// this port does not read; a pattern of `None` takes the branch that leaves the heap and the entry
/// array null. The heap itself is not allocated - a solid heap is an allocator this port has no
/// counterpart for - so [`ModelEmitter::heap_size`] carries the size that was asked for. The
/// `searchUpdateMaterialID` call at `:49` and the material ids it resolves are inside the blocked
/// half, and `mpHeap` and `field_0x20` therefore stay absent rather than zero-sized.
pub fn model_emitter_new_with(
    emitter: usize,
    model_data: Option<usize>,
    tex_pattern: Option<usize>,
    update_material_num: u16,
    frame: u16,
    rotate_axis: u8,
    tev: TevColors,
    emitters: &mut [Emitter],
) -> ModelEmitter {
    let mut node = model_emitter_new();
    node.base_emitter = Some(emitter);
    node.model_data = model_data;
    node.tex_pattern = tex_pattern;
    node.update_material_num = update_material_num;
    node.rotate_axis = rotate_axis;
    node.tev = tev;
    if tex_pattern.is_some() {
        node.anm_index = (0..update_material_num).collect();
        if update_material_num > 0 {
            node.anm_frame = Some(frame);
        }
    }
    if let Some(emitter) = emitters.get_mut(emitter) {
        emitter.become_immortal();
    }
    node
}

impl ModelEmitter {
    /// The size of the solid heap the constructor asks for: the entry array at
    /// [`MATERIAL_ENTRY_SIZE`] a piece, plus the heap's own [`HEAP_HEADER_SIZE`]
    /// (`d_particle.cpp:45`).
    pub fn heap_size(&self) -> u32 {
        u32::from(self.update_material_num) * MATERIAL_ENTRY_SIZE + HEAP_HEADER_SIZE
    }
}

/// `dPa_J3Dmodel_c::dPa_J3Dmodel_c` (`d_particle.cpp:183-185`): one cleared flag.
pub fn j3d_model_new() -> ModelSlot {
    ModelSlot::default()
}

/// `dPa_modelControl_c` (`d/d_particle.h:383-393`) over `node_list_class` (`c_list.h:6-10`): a head,
/// a tail, a size, and the [`MODEL_POOL`]-entry model pool `newModel` hands out from.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelControl {
    pub models: Vec<ModelSlot>,
    pub nodes: Vec<ModelEmitter>,
    pub head: Option<usize>,
    pub tail: Option<usize>,
    pub size: usize,
}

impl Default for ModelControl {
    /// `dPa_modelControl_c::dPa_modelControl_c` (`d_particle.cpp:169-180`): clear every one of the
    /// [`MODEL_POOL`] slots and empty the list. The reference then builds a model into each slot and
    /// asserts it, which this port has nothing to build with, so the slots carry no model.
    fn default() -> Self {
        ModelControl {
            models: vec![j3d_model_new(); MODEL_POOL],
            nodes: Vec::new(),
            head: None,
            tail: None,
            size: 0,
        }
    }
}

impl ModelControl {
    /// `cLs_Addition` (`c_list.cpp:32`): append an emitter to the tail.
    pub fn add(&mut self, emitter: ModelEmitter) -> usize {
        let index = self.nodes.len();
        let mut emitter = emitter;
        emitter.node.prev = self.tail;
        emitter.node.next = None;
        self.nodes.push(emitter);
        match self.tail {
            Some(tail) => self.nodes[tail].node.next = Some(index),
            None => self.head = Some(index),
        }
        self.tail = Some(index);
        self.size += 1;
        index
    }

    /// `cLs_SingleCut` (`c_list.cpp:18-29`) with the object cleared after: unlink a node and report
    /// whether the list still has anything in it.
    pub fn single_cut(&mut self, index: usize) -> bool {
        let (prev, next) = {
            let node = &self.nodes[index].node;
            (node.prev, node.next)
        };
        if self.head == Some(index) {
            self.head = next;
        }
        if self.tail == Some(index) {
            self.tail = prev;
        }
        if let Some(prev) = prev {
            self.nodes[prev].node.next = next;
        }
        if let Some(next) = next {
            self.nodes[next].node.prev = prev;
        }
        let node = &mut self.nodes[index].node;
        node.prev = None;
        node.next = None;
        node.data = None;
        self.size -= 1;
        self.size > 0
    }

    /// `dPa_modelControl_c::newModel` (`d_particle.cpp:201-231`): claim the first slot whose flag is
    /// clear and mark it taken, or hand back nothing when all [`MODEL_POOL`] are claimed. The
    /// shape packet, material packet and vertex buffer the reference binds to a freshly claimed
    /// model live inside the model, so they are not reproduced.
    pub fn new_model(&mut self) -> Option<usize> {
        let index = self.models.iter().position(|slot| slot.init == 0)?;
        self.models[index].init = 1;
        self.models[index].model
    }

    /// `dPa_modelControl_c::draw` (`d_particle.cpp:234-255`): free the whole model pool, then walk
    /// the emitter list from the head and either retire each emitter that is marked for deletion -
    /// dropping its immortality on the way - or draw it.
    ///
    /// Left a no-op past the list walk, for the reason [`ModelEmitter::draw`] gives. The reference
    /// also sets the graphics fog first (`:239`), which is a renderer's own fog state.
    pub fn draw(&mut self, emitters: &mut [Emitter], particles: &mut Particles) {
        for slot in &mut self.models {
            slot.init = 0;
        }
        let mut cursor = self.head;
        while let Some(current) = cursor {
            cursor = self.nodes[current].node.next;
            let base = self.nodes[current].base_emitter;
            let retire = base.is_some_and(|base| {
                emitters
                    .get(base)
                    .is_some_and(|emitter| emitter.is_enable_delete_emitter())
            });
            if retire {
                if let Some(base) = base {
                    if let Some(emitter) = emitters.get_mut(base) {
                        emitter.quit_immortal();
                    }
                }
                self.single_cut(current);
            } else {
                let Self { models, nodes, .. } = self;
                nodes[current].draw(models, emitters, particles);
            }
        }
    }
}

impl ModelEmitter {
    /// `dPa_J3DmodelEmitter_c::draw` (`d_particle.cpp:73-166`): un-stop the emitter's draw, and for
    /// every live particle claim a model, align it to the particle's velocity, rebuild the
    /// particle's axis from that frame, roll it about the configured axis, place it at the particle's
    /// global position and scale it by the emitter's particle scale and the particle's own.
    ///
    /// Two details are carried faithfully rather than tidied. The roll matrix is built once outside
    /// the loop (`:78-80`) and only rewritten inside the `if` (`:115-132`), so a particle with no
    /// roll is drawn through the previous particle's. And the concatenation at `:134` runs again
    /// after the one at `:131`, which rolls a particle that has a roll twice.
    ///
    /// The draw returns early - leaving `stopDrawParticle` uncalled - when the pool cannot supply a
    /// model (`:83-84`), and the rest of the list is then left undrawn.
    ///
    /// Left a no-op past the transform. It needs a model pass with a base transform matrix, a base
    /// scale and a model-data shape, material and vertex binding, which a renderer whose vocabulary
    /// is indexed triangles, one texture page and one group alpha does not offer. The light-colour
    /// write at `:149` and the texture-index animator calls at `:151-161` are part of that same
    /// pass. The rebuilt axis is written back into the particle, which is state the reference keeps.
    pub fn draw(
        &mut self,
        models: &mut [ModelSlot],
        emitters: &mut [Emitter],
        particles: &mut Particles,
    ) {
        self.transforms.clear();
        let Some(base) = self.base_emitter else {
            return;
        };
        if base >= emitters.len() {
            return;
        }
        emitters[base].play_draw_particle();
        let owned = emitters[base].particles.clone();
        let mut roll = Mat4::IDENTITY;
        for index in owned {
            let Some(particle) = particles.items.get(index).copied() else {
                continue;
            };
            let (old_axis, velocity_raw) = (particle.axis, particle.velocity);
            let (scale_x, scale_y) = (particle.scale_x, particle.scale_y);
            let (rotate_angle, position) = (particle.rotate_angle, particle.global_position);
            let Some(claimed) = models.iter().position(|slot| slot.init == 0) else {
                return;
            };
            models[claimed].init = 1;
            let velocity = if is_zero(velocity_raw) {
                Vec3::Y
            } else {
                velocity_raw.normalize()
            };
            let cross = old_axis.cross(velocity);
            let cross = if is_zero(cross) {
                Vec3::Z
            } else {
                cross.normalize()
            };
            let axis = velocity.cross(cross).normalize();
            let mut mtx = Mat4::from_cols(
                Vec4::new(axis.x, axis.y, axis.z, 0.0),
                Vec4::new(velocity.x, velocity.y, velocity.z, 0.0),
                Vec4::new(cross.x, cross.y, cross.z, 0.0),
                Vec4::new(0.0, 0.0, 0.0, 1.0),
            );
            let degrees = f32::from(rotate_angle) * S2DEG;
            if degrees != 0.0 {
                roll = match self.rotate_axis {
                    0 => Mat4::from_rotation_y(degrees * DEG2RAD),
                    1 => Mat4::from_rotation_x(degrees * DEG2RAD),
                    2 => Mat4::from_rotation_z(degrees * DEG2RAD),
                    3 => Mat4::from_axis_angle(Vec3::ONE, degrees * DEG2RAD),
                    _ => Mat4::IDENTITY,
                };
                mtx = mtx * roll;
            }
            mtx = mtx * roll;
            mtx.w_axis = Vec4::new(position.x, position.y, position.z, 1.0);
            let base_scale = emitters[base].global_particle_scale;
            let scale = Vec3::new(
                base_scale.x * scale_x,
                base_scale.y * scale_y,
                base_scale.z * scale_x,
            );
            particles.items[index].axis = axis;
            self.transforms.push(ModelTransform {
                mtx,
                scale,
                velocity_dir: velocity,
                cross,
                axis,
            });
        }
        emitters[base].stop_draw_particle();
    }
}

/// The environment particle system: the pool, the emitter list, the registry entries and the shared
/// smoke callback the ported callbacks install.
///
/// This is the whole of the particle callback layer. The emitter, the particle and the emitter
/// manager it is written against are not present in this port, and neither are the packed field
/// tables they read, so what is here is only the state the callbacks themselves read and write -
/// laid out by the offsets `d/d_particle.h` documents - over which the ported callbacks run as
/// self-contained functions. Four retail writes are unreachable for want of the base classes or the
/// packed data, and each is reported at the item that would have carried it rather than faked: the
/// emission and integration step, the emitter factory, the frustum clipper's stage cull point, and
/// the environment light's actor colour. Every draw is a documented no-op: the state its compute
/// half produces is ported and kept, and the graphics half it would have been consumed by is named
/// in the doc where it sits.
#[derive(Resource, Debug, PartialEq)]
pub struct Jpa {
    pub particles: Particles,
    /// The emitter list, [`EMITTER_POOL`] slots. An unused slot is one with `active` clear.
    pub emitters: Vec<Emitter>,
    /// `JPAEmitterManager::pResMgrArray` (`JPAEmitterManager.h:46`): the loaded JPA archives, one
    /// per resource-manager slot. `dPa_control_c::createCommon` mounts `common.jpc` into slot 0
    /// (`d_particle.cpp:713-735`, read at `d_s_logo.cpp:931`) and `createRoomScene` the stage's
    /// `Pscene###.jpc` into slot 1 (`:736-756`, `:744`, read by `readScene` at `:757-770`).
    pub bank: JpaBank,
    /// The [`JpaGap`]s [`create_emitter_id`] recorded since the shell last read them.
    pub gaps: Vec<JpaGap>,
    pub model_control: ModelControl,
    /// `mSimpleCallbacks[25]` (`d/d_particle.h:563`), the registry `getSimple` searches.
    pub simple: Vec<Simple>,
    /// `dPa_control_c::mSmokeEcallback` (`d_particle.cpp:609`), built from `(0, 1, 1, 1)`. A smoke
    /// emitter's `end` reinstalls this instance rather than clearing the callback, so it is held
    /// here and copied into the emitter.
    pub smoke_callback: Smoke,
    /// `mNumSimple` (`d/d_particle.h:562`): how many registry entries have been made.
    pub num_simple: usize,
    /// `dPa_control_c::mStatus` (`d_particle.cpp:615`), read through `isStatus` (`:548`). None of
    /// this module's callbacks tests it; the ripple, splash and track draws do, and those are out of
    /// scope, so it is carried clear.
    pub status: u8,
}

impl Default for Jpa {
    fn default() -> Self {
        Jpa {
            particles: Particles::default(),
            emitters: vec![Emitter::default(); EMITTER_POOL],
            bank: JpaBank::default(),
            gaps: Vec::new(),
            model_control: ModelControl::default(),
            simple: vec![simple_new(); SIMPLE_SLOTS],
            smoke_callback: smoke_new_shared(0, 1, 1, 1),
            num_simple: 0,
            status: 0,
        }
    }
}

impl Jpa {
    /// `JPAEmitterManager::createSimpleEmitterID` (`JPAEmitterManager.cpp:52-78`), the resolution
    /// the demo particle adaptor is one caller of: `TAdaptor_particle` decodes its packed id
    /// first - low half user index, bits 24-31 group, bits 16-23 resource manager
    /// (`JStudio_JParticle/object-particle.cpp:141-162`). The resolved resource becomes the new
    /// emitter's [`Emitter::resource`]; a miss records a [`JpaGap`] and creates nothing.
    pub fn create_simple_emitter_id(
        &mut self,
        pos: Vec3,
        user_id: u16,
        group_id: u8,
        rm_id: u8,
    ) -> Option<usize> {
        create_emitter_id(
            &mut self.emitters,
            &self.bank,
            &mut self.gaps,
            pos,
            user_id,
            group_id,
            rm_id,
        )
    }

    /// [`Jpa::create_simple_emitter_id`] for the packed `PARTICLE` id whole, which is how the
    /// demo adaptor carries it (`JStudio_JParticle/object-particle.cpp:86-87`, `:144-155`).
    pub fn create_simple_emitter_packed_id(&mut self, pos: Vec3, id: u32) -> Option<usize> {
        create_emitter_id(
            &mut self.emitters,
            &self.bank,
            &mut self.gaps,
            pos,
            (id & 0xFFFF) as u16,
            (id >> 24) as u8,
            ((id >> 16) & 0xFF) as u8,
        )
    }

    /// The [`JpaGap`]s [`create_emitter_id`] recorded since the shell last read them.
    pub fn take_gaps(&mut self) -> Vec<JpaGap> {
        core::mem::take(&mut self.gaps)
    }

    /// Mount one archive into a bank slot: `JPAResourceManager`'s constructor over
    /// `JPAEmitterArchiveLoaderDataBase::load` (`JPAResourceManager.cpp:79-82`), the call
    /// `dPa_control_c::createCommon` and `createRoomScene` make (`d_particle.cpp:715`, `:741`).
    /// An unreadable archive is a [`JpaBankError`] and leaves the slot as it was.
    pub fn load_manager(&mut self, rm_id: u8, data: &[u8]) -> Result<(), JpaBankError> {
        self.bank.register(rm_id, data)
    }

    /// `JPAEmitterManager::clearResourceManager` (`JPAEmitterManager.cpp:139-151`): every emitter
    /// whose `mResMgrID` is `rmID` is force-deleted - `forceDeleteEmitter`'s `deleteAllParticle`
    /// (`JPAEmitter.cpp:403-418`) then `deleteEmitter` (`JPAEmitterManager.cpp:114-119`), which
    /// hands the slot back to the pool - and the resource-manager slot is nulled last.
    ///
    /// This is the sweep `dPa_control_c::removeRoomScene` runs when the last holder of a stage's
    /// `Pscene###.jpc` goes away (`d_particle.cpp:784`), which is why the port's stage mount calls
    /// it: without it an emitter the previous stage created out of the vanished slot keeps
    /// ticking. The deletion is [`Emitter::is_enable_delete_emitter`]'s unconditional form; the
    /// reference walks its sixteen group lists, which the pool's `active` flag stands in for.
    ///
    /// A [`Simple`] callback keeps its own `base_emitter` handle through all of this - retail's
    /// `dPa_simpleEcallBack::removeEmitter` is a `removeScene` step, not a `clearResourceManager`
    /// one (`d_particle.cpp:799-800`) - so nothing here rewrites one.
    pub fn clear_resource_manager(&mut self, rm_id: u8) {
        let Self {
            emitters,
            particles,
            bank,
            ..
        } = self;
        for emitter in emitters.iter_mut() {
            if !emitter.active || emitter.res_mgr_id != rm_id {
                continue;
            }
            particles.delete_all(emitter);
            *emitter = Emitter::default();
        }
        bank.clear(rm_id);
    }

    /// `dPa_control_c::newSimple` (`d_particle.cpp:984-996`): take the next registry entry in
    /// order and make its emitter. Refused past [`SIMPLE_SLOTS`], which is the count the reference
    /// reports as an over-registration before it returns false, and refused when the effect id's
    /// resource cannot be resolved, in which case the entry is not consumed either
    /// (`:990-994` increments `mNumSimple` only after `create` answers non-NULL).
    pub fn new_simple(&mut self, effect_id: u16, group_id: u8) -> Option<usize> {
        if self.num_simple >= SIMPLE_SLOTS {
            return None;
        }
        let slot = self.num_simple;
        self.simple[slot].res_id = effect_id;
        self.simple[slot].group_id = group_id;
        simple_create(
            &mut self.simple[slot],
            &mut self.emitters,
            &self.bank,
            &mut self.gaps,
            slot,
        )?;
        self.num_simple += 1;
        Some(slot)
    }

    /// `dPa_control_c::getSimple` (`d_particle.cpp:1008-1014`): the first registry entry whose
    /// effect id matches.
    pub fn get_simple(&self, id: u16) -> Option<usize> {
        (0..self.num_simple).find(|slot| self.simple[*slot].res_id == id)
    }

    /// `dPa_control_c::setSimple` (`d_particle.cpp:998-1005`): find the entry and queue on it.
    pub fn set_simple(
        &mut self,
        id: u16,
        pos: Vec3,
        alpha: u8,
        prm: Color,
        env: Color,
        affected_by_wind: i32,
    ) -> bool {
        match self.get_simple(id) {
            Some(slot) => simple_set(
                &mut self.simple[slot],
                pos,
                alpha,
                prm,
                env,
                affected_by_wind,
            ),
            None => false,
        }
    }

    /// `dPa_control_c::drawModelParticle` (`d/d_particle.h:508`).
    pub fn draw_models(&mut self) {
        let Self {
            model_control,
            emitters,
            particles,
            ..
        } = self;
        model_control.draw(emitters, particles);
    }
}

/// The per-frame step, in `JPABaseEmitter::calc`'s order (`JPAEmitter.cpp:230-248`): the emitter
/// callback's `execute`, then its `executeAfter`, then each live particle's callback, then the frame
/// counter and the termination test. What that order buys the port is the one thing the callbacks
/// here need: the spawns a simple emitter has queued during the frame are drained in the same frame,
/// and the wind push sees the position the emitter has just been moved to.
///
/// The emission half of `calc` - `calcCreatePtcls` and the packet-driven key, field and draw passes -
/// is not ported, so nothing advances [`Emitter::current_create_number`] here and a simple emitter
/// drains only on a frame whose create count some other system has already set.
pub fn control(weather: Res<Weather>, camera: Option<Res<Camera>>, mut jpa: ResMut<Jpa>) {
    let clip = camera.as_deref().and_then(Camera::view).map(Clip::new);
    let shared = jpa.smoke_callback.clone();
    let Jpa {
        emitters,
        particles,
        simple,
        ..
    } = &mut *jpa;
    for index in 0..emitters.len() {
        if !emitters[index].active {
            continue;
        }
        match emitters[index].callback.clone() {
            Some(EmitterCallback::Follow(mut callback)) => {
                follow_execute(&mut callback, emitters, index, EndKind::Follow, &shared);
                if matches!(emitters[index].callback, Some(EmitterCallback::Follow(_))) {
                    emitters[index].callback = Some(EmitterCallback::Follow(callback));
                }
            }
            Some(EmitterCallback::Smoke(mut callback)) => {
                let mut base = callback.follow;
                follow_execute(&mut base, emitters, index, EndKind::Smoke, &shared);
                if matches!(emitters[index].callback, Some(EmitterCallback::Smoke(_))) {
                    callback.follow = base;
                    emitters[index].callback = Some(EmitterCallback::Smoke(callback));
                }
            }
            Some(EmitterCallback::Simple(slot)) => {
                if let Some(callback) = simple.get_mut(slot) {
                    simple_execute_after(callback, &mut emitters[index], particles, clip.as_ref());
                }
            }
            None => {}
        }
        let owned = std::mem::take(&mut emitters[index].particles);
        for slot in owned.iter().copied() {
            let Some(entry) = particles.items.get(slot) else {
                continue;
            };
            let callback = entry.callback;
            let Some(particle) = particles.items.get_mut(slot) else {
                continue;
            };
            match callback {
                Some(ParticleCallback::Smoke) => {
                    smoke_particle_execute(&weather, &emitters[index], particle);
                }
                Some(ParticleCallback::Wind) => wind_particle_execute(&weather, particle),
                None => {}
            }
        }
        emitters[index].particles = owned;
        emitters[index].frame += 1;
        emitters[index].do_termination_process();
    }
}

/// The per-frame model draw, `dPa_control_c::drawModelParticle` (`d/d_particle.h:508`). Every
/// per-particle transform it produces is real; the model it would have bound is a no-op for the
/// reason [`ModelEmitter::draw`] gives.
pub fn draw_model_particles(mut jpa: ResMut<Jpa>) {
    jpa.draw_models();
}

/// `dPa_control_c::readScene` (`d_particle.cpp:757-770`) with `createScene`'s mount (`:773-780`
/// over `createRoomScene`, `:736-756`): the stage's `/res/Particle/Pscene%03d.jpc` into slot
/// [`SCENE_RM_ID`]. `particle_no` is `dStage_stagInfo_GetParticleNo`'s answer
/// (`d_stage.h:1047-1049`, read at `d_s_play.cpp:1378-1379`); `room_particle_no` is a room's own
/// `FILI` number (`d_s_room.cpp:237-241`), which `readScene` reaches only as the fallback: a
/// stage number that is not [`NO_SCENE`] consumes the reader's single read (`mCount++`),
/// `:761-765`, and the room's read then does nothing. A stage that named [`NO_SCENE`] returns
/// false before the counter moves (`:758-760`), so the first room whose record names a number is
/// the read that mounts.
///
/// A stage whose token is [`NO_SCENE`] and whose rooms named no number mounts nothing - the
/// reader's own `if (i_no == 0xff) return false` (`:758-760`) - and an archive the dump does not
/// carry mounts nothing either, so a scene particle id keeps resolving to
/// [`JpaGap::ManagerNotMounted`]. Either way the slot is first swept by
/// [`Jpa::clear_resource_manager`], the `clearResourceManager(1)` of `removeRoomScene`
/// (`:781-792`), so the previous stage's emitters do not survive the mount.
pub fn mount_scene_archive(jpa: &mut Jpa, particle_no: u8, room_particle_no: Option<u8>) {
    jpa.clear_resource_manager(SCENE_RM_ID);
    let number = if particle_no == NO_SCENE {
        room_particle_no
    } else {
        Some(particle_no)
    };
    let Some(number) = number else {
        return;
    };
    let name = format!("Pscene{number:03}.jpc");
    let Some(data) = crate::assets::load_particle_archive(&name) else {
        return;
    };
    match jpa.load_manager(SCENE_RM_ID, &data) {
        Ok(()) => {}
        Err(error) => log::warn!("clearsleeper: scene JPA archive {name} unreadable ({error})"),
    }
}

/// The particle layer's construction: `dPa_control_c`'s constructor and the common-archive mount
/// that precedes every stage. `d_s_logo.cpp:931` reads `/res/Particle/common.jpc` into
/// `dPa_control_c::createCommon` (`d_particle.cpp:713-735`), which is slot 0's mount. The stage's
/// own `Pscene###.jpc` - or, when the stage named none, the one a room's `FILI` record names
/// (`d_s_room.cpp:237-241`) - goes into slot 1 through [`mount_scene_archive`] at the stage load
/// that names one.
pub fn init(world: &mut World) {
    let mut jpa = Jpa::default();
    if let Some(data) = crate::assets::load_particle_archive(COMMON_ARCHIVE) {
        match jpa.load_manager(COMMON_RM_ID, &data) {
            Ok(()) => {}
            Err(error) => log::warn!("clearsleeper: common JPA archive unreadable ({error})"),
        }
    }
    world.insert_resource(jpa);
}
