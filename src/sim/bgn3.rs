//! Puppet Ganon's phase-3 body: `daBgn3`, the "G (fast worm)" head of `Bgn.h`.
//!
//! The translation unit this module ports is `d_a_bgn3.cpp` (1357 lines, `Nonmatching == 0`,
//! so it is fully matched and fully portable) together with its own header `d_a_bgn3.h`.
//! `d_a_bgn.cpp` is a DIFFERENT actor - the 3716-line `bgn_class` boss body - and is
//! referenced here only where the two genuinely meet: `fopAcM_create(fpcNm_BGN3_e, ...)`
//! (`d_a_bgn.cpp:3674`) and the `fpcNm_BGN3_e` search callback (`d_a_bgn.cpp:940`).
//!
//! Nothing here reaches the DZB water attribute code `gr_check` needs, the frame counts the
//! `Morf`/BRK frame controllers hold, the particle system, the `cCcD` collider registry, the
//! per-actor status bits, or `attention_info`. Each of those omissions is marked at its site.

use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::{Mat4, Vec3, Vec4};

use super::actor::Actor;
use super::arrow;
use super::collision::Collision;
use super::combat::{Combat, CutKind};
use super::damage;
use super::damage_resolve::{
    AT_SE_ARROW, AT_SE_NONE, AT_SPL_UNK0, AT_SPL_UNK1, AT_TYPE_ALL, AT_TYPE_BOOMERANG,
    AT_TYPE_FIRE_ARROW, AT_TYPE_ICE_ARROW, AT_TYPE_LIGHT, AT_TYPE_LIGHT_ARROW,
    AT_TYPE_NORMAL_ARROW, AT_TYPE_SWORD, AT_TYPE_UNK20000, AT_TYPE_UNK400000, AT_TYPE_WATER,
    AT_TYPE_WIND, Atk, AtkType, Attacker, CcAtInfo, HitAudio, HitStop, NAME_ARROW, NAME_PLAYER,
    PlayerCut, RESULT_BOMB, RESULT_SKULL_HAMMER, RESULT_SWORD, at_power_check, def_se_set,
};
use super::esa::Esa;
use super::events::{SimEvent, SimEvents};
use super::health::Health;
use super::item::Held;
use super::player::Player;
use super::puppet_ganon::PuppetGanon;
use super::vibration::Vibration;
use super::walk;

/// Empty on purpose: `"Bgn3"` has no `OBJNAME` row in `d_stage.cpp` (the table runs
/// `OBJNAME("Bgn", fpcNm_BGN_e, ...)` at `:1193` to `OBJNAME("Arrow", fpcNm_ARROW_e, ...)`
/// at `:1248` and has no `fpcNm_BGN3_e` row at all), so `fpcNm_BGN3_e` is never stage-placed -
/// `d_a_bgn.cpp:3674` `fopAcM_create`s it at runtime. A module with no stage row must not
/// claim a stage string; the name is declared once, by `src/sim/puppet_ganon.rs`
/// (`BODY_3_ACTOR`, cited there to `d_a_bgn.cpp:940` `bgn3_s_sub`) and borrowed by
/// `ganon_form3.rs`.
pub const BGN3_ACTORS: [&str; 0] = []; // <- d_a_bgn.cpp:3674

pub fn is_bgn3(name: &str) -> bool {
    BGN3_ACTORS.contains(&name)
}

/// `ki_c_sub` counts actors whose name is `fpcNm_KS_e` (`d_a_bgn3.cpp:1054`). That enum value
/// has TWO stage rows and neither is the enum spelling: `d_stage.cpp:679`
/// `OBJNAME("kuro_s", fpcNm_KS_e, 255, 20)` and `d_stage.cpp:680`
/// `OBJNAME("kuro_t", fpcNm_KS_e, 255, 0)`. `src/sim/ks.rs` already owns both, so the name
/// list is borrowed rather than duplicated. Case-sensitive, per `d_stage.cpp`.
pub fn is_ks(name: &str) -> bool {
    super::ks::KS_ACTORS.contains(&name)
}

// ---------------------------------------------------------------- daBgn3_HIO_c

/// `l_HIO` (`static daBgn3_HIO_c l_HIO`, `d_a_bgn3.cpp:63`), every field of it, with the
/// defaults `daBgn3_HIO_c::daBgn3_HIO_c` writes (`d_a_bgn3.cpp:72-105`). The struct is a
/// `JORReflexible` whose only other member is `genMessage`, so this list is the whole HIO
/// surface and every default is a proven value, not a guess.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Bgn3Hio {
    pub no: i8,
    pub m05: u8,
    pub m06: u8,
    pub m08: f32,
    pub m0c: i16,
    pub m0e: i16,
    pub m10: f32,
    pub m14: i16,
    pub m16: i16,
    pub m18: f32,
    pub m1c: i16,
    pub m1e: i16,
    pub m20: i16,
    pub m22: i16,
    pub m24: i16,
    pub m26: i16,
    pub m28: i16,
    pub m2a: i16,
    pub m2c: i16,
    pub m2e: i16,
    pub m30: i16,
    pub m34: f32,
    pub m38: f32,
    pub m3c: i16,
    pub m3e: u8,
    pub m40: f32,
    pub m44: f32,
    pub m48: f32,
    pub m4c: f32,
    pub m50: u8,
    pub m52: u8,
    pub m54: u8,
}

/// `mNo = -1` (`d_a_bgn3.cpp:73`), the HIO child id before `mDoHIO_createChild` hands one out.
pub const HIO_NO: i8 = -1; // <- d_a_bgn3.cpp:73
/// `m05 = 0` (`:74`) - the gate on the whole action switch in `move` (`:802`).
pub const HIO_M05: u8 = 0; // <- d_a_bgn3.cpp:74
/// `m06 = 0` (`:75`) - non-zero overrides `actor->health` every `Execute` (`:1076`).
pub const HIO_M06: u8 = 0; // <- d_a_bgn3.cpp:75
/// `m08 = 50.0f` (`:76`) - `move0`'s crawl speed at `health == 3` (`:373`).
pub const HIO_M08: f32 = 50.0; // <- d_a_bgn3.cpp:76
/// `m0C = 800` (`:77`) - `m0FD8E` at `health == 3` (`:417`).
pub const HIO_M0C: i16 = 800; // <- d_a_bgn3.cpp:77
/// `m0E = 3` (`:78`) - wall hits before the turn is rerolled at `health == 3` (`:393`).
pub const HIO_M0E: i16 = 3; // <- d_a_bgn3.cpp:78
/// `m10 = 60.0f` (`:80`) - `move0`'s crawl speed at `health == 2` (`:375`).
pub const HIO_M10: f32 = 60.0; // <- d_a_bgn3.cpp:80
/// `m14 = 1000` (`:81`) - `m0FD8E` at `health == 2` (`:419`).
pub const HIO_M14: i16 = 1000; // <- d_a_bgn3.cpp:81
/// `m16 = 3` (`:82`) - wall-hit cap at `health == 2` (`:395`).
pub const HIO_M16: i16 = 3; // <- d_a_bgn3.cpp:82
/// `m18 = 80.0f` (`:84`) - `move0`'s crawl speed below `health == 2` (`:377`).
pub const HIO_M18: f32 = 80.0; // <- d_a_bgn3.cpp:84
/// `m1C = 0x4b0` (`:85`) - `m0FD8E` below `health == 2` (`:421`).
pub const HIO_M1C: i16 = 0x4b0; // <- d_a_bgn3.cpp:85
/// `m1E = 3` (`:86`) - wall-hit cap below `health == 2` (`:397`).
pub const HIO_M1E: i16 = 3; // <- d_a_bgn3.cpp:86
/// `m20 = 0x3c` (`:79`) - `m0FD90` after the wall cap at `health == 3` (`:403`).
pub const HIO_M20: i16 = 0x3c; // <- d_a_bgn3.cpp:79
/// `m22 = 0x28` (`:83`) - `m0FD90` after the wall cap at `health == 2` (`:405`).
pub const HIO_M22: i16 = 0x28; // <- d_a_bgn3.cpp:83
/// `m24 = 0x1e` (`:87`) - `m0FD90` after the wall cap below `health == 2` (`:407`).
pub const HIO_M24: i16 = 0x1e; // <- d_a_bgn3.cpp:87
/// `m26 = 10` (`:88`) - `m0FDAA[0]` stun for an unclassified hit (`:627`).
pub const HIO_M26: i16 = 10; // <- d_a_bgn3.cpp:88
/// `m28 = 0x14` (`:89`) - `m0FDAA[0]` stun for an arrow (`:623`).
pub const HIO_M28: i16 = 0x14; // <- d_a_bgn3.cpp:89
/// `m2A = 0x2d` (`:90`) - `m0FDAA[0]` stun for a non-critical sword cut (`:618`).
pub const HIO_M2A: i16 = 0x2d; // <- d_a_bgn3.cpp:90
/// `m2C = 0x78` (`:91`) - `m0FDAA[0]` stun for a critical sword cut (`:613`).
pub const HIO_M2C: i16 = 0x78; // <- d_a_bgn3.cpp:91
/// `m2E = 200` (`:92`) - `m0FDAA[0]` stun for a bomb (`:607`).
pub const HIO_M2E: i16 = 200; // <- d_a_bgn3.cpp:92
/// `m30 = 0xfa` (`:93`) - `m0FDAA[0]` stun for a skull hammer (`:603`).
pub const HIO_M30: i16 = 0xfa; // <- d_a_bgn3.cpp:93
/// `m34 = 5.0f` (`:94`) - `m0FDBC`, the neck rest length, chases this (`:275`).
pub const HIO_M34: f32 = 5.0; // <- d_a_bgn3.cpp:94
/// `m38 = 0.5f` (`:95`) - `m0FDB8`, the segment lag, chases this (`:274`).
pub const HIO_M38: f32 = 0.5; // <- d_a_bgn3.cpp:95
/// `m3C = 3` (`:96`) - `m100AC`, the frames the tail sheds `fpcNm_KS_e` for (`:391`).
pub const HIO_M3C: i16 = 3; // <- d_a_bgn3.cpp:96
/// `m3E = 1` (`:97`) - the `ke`/line-material draw gate (`:753`, `:963`, `:124`).
pub const HIO_M3E: u8 = 1; // <- d_a_bgn3.cpp:97
/// `m40 = 150.0f` (`:98`) - the `ke` chain root height (`:896`, `:971`).
pub const HIO_M40: f32 = 150.0; // <- d_a_bgn3.cpp:98
/// `m44 = 45.0f` (`:99`) - the `ke` segment length scale (`:899`, `:974`).
pub const HIO_M44: f32 = 45.0; // <- d_a_bgn3.cpp:99
/// `m48 = 8.0f` (`:100`) - `mLineMat.update`'s fourth argument (`:125`, `:129`). Renderer
/// only, but the HIO default is real and must be transcribed.
pub const HIO_M48: f32 = 8.0; // <- d_a_bgn3.cpp:100
/// `m4C = -8.0f` (`:101`) - `ke_control`'s per-segment sink (`:737`).
pub const HIO_M4C: f32 = -8.0; // <- d_a_bgn3.cpp:101
/// `m50 = 0xbc` (`:102`) - `ke_color.r` (`:1320`).
pub const HIO_M50: u8 = 0xbc; // <- d_a_bgn3.cpp:102
/// `m52 = 0x98` (`:103`) - `ke_color.g` (`:1321`).
pub const HIO_M52: u8 = 0x98; // <- d_a_bgn3.cpp:103
/// `m54 = 0x42` (`:104`) - `ke_color.b` (`:1322`).
pub const HIO_M54: u8 = 0x42; // <- d_a_bgn3.cpp:104

impl Default for Bgn3Hio {
    fn default() -> Self {
        Self {
            no: HIO_NO,
            m05: HIO_M05,
            m06: HIO_M06,
            m08: HIO_M08,
            m0c: HIO_M0C,
            m0e: HIO_M0E,
            m10: HIO_M10,
            m14: HIO_M14,
            m16: HIO_M16,
            m18: HIO_M18,
            m1c: HIO_M1C,
            m1e: HIO_M1E,
            m20: HIO_M20,
            m22: HIO_M22,
            m24: HIO_M24,
            m26: HIO_M26,
            m28: HIO_M28,
            m2a: HIO_M2A,
            m2c: HIO_M2C,
            m2e: HIO_M2E,
            m30: HIO_M30,
            m34: HIO_M34,
            m38: HIO_M38,
            m3c: HIO_M3C,
            m3e: HIO_M3E,
            m40: HIO_M40,
            m44: HIO_M44,
            m48: HIO_M48,
            m4c: HIO_M4C,
            m50: HIO_M50,
            m52: HIO_M52,
            m54: HIO_M54,
        }
    }
}

// ----------------------------------------------------------------- file statics

/// `l_HIO` (`d_a_bgn3.cpp:63`), `static bool hio_set` (`:62`) - the once-only
/// `mDoHIO_createChild("Ｇ（ハヤムシ）", &l_HIO)` latch (`:1297-1301`), and `ke_color`
/// (`static GXColor ke_color`, `:64`), whose `r/g/b/a` come from the HIO in `Create`
/// (`:1320-1323`) and are read only by `daBgn3_Draw` (`:125`), which has no sim consumer.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Bgn3Static {
    pub hio: Bgn3Hio,
    pub hio_set: bool,
    pub ke_color: [u8; 4],
}

impl Default for Bgn3Static {
    fn default() -> Self {
        let hio = Bgn3Hio::default();
        Self {
            hio,
            hio_set: false,
            ke_color: [hio.m50, hio.m52, hio.m54, 0xFF],
        }
    }
}

/// The three `static`s `daBgn3_Execute` refreshes by search every tick:
/// `static bgn_class* bgn` (`d_a_bgn3.cpp:59`, found by `bgn_s_sub` at `:110`),
/// `static esa_class* esa` (`:60`, found by `esa_s_sub` at `:222`) and
/// `static s32 ki_all_count` (`:65`, counted by `ki_c_sub` at `:1055` and reset by
/// `ki_check` at `:1062`).
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct Bgn3Siblings {
    /// `bgn != NULL` after `fpcM_Search(bgn_s_sub, i_this)` (`:1072`); a `NULL` here returns
    /// `TRUE` from `Execute` before anything else happens (`:1073-1075`).
    pub bgn_found: bool,
    /// `bgn->m02B5`, `s8 m02B5` (`d_a_bgn.h:83`). `!= 2` parks the worm at y = 30000
    /// (`:1080-1088`) and `== 2` is also the `Draw` gate (`:124`).
    pub bgn_phase: i8,
    /// `bgn->current.pos`, which `d_a_bgn.cpp` reads the worm off.
    pub bgn_pos: Vec3,
    /// A non-NULL `esa` doubles `move0`'s turn rate and takes priority over the player
    /// orbit (`:359-362`).
    pub esa_found: bool,
    pub esa_pos: Vec3,
    /// `ki_all_count` after `ki_check`'s search.
    pub ki_count: i32,
}

// ---------------------------------------------------------------- module tables

/// `static u16 ke_max[]` (`d_a_bgn3.cpp:67`), the `ke` chain length of `mParts[0..7)`. Only
/// seven entries exist because the `ke` loop is `if ((i < 7) && ...)` (`:963`).
pub const KE_MAX: [u16; 7] = [0x0028, 0x0023, 0x001E, 0x0019, 0x0014, 0x000F, 0x000F]; // <- d_a_bgn3.cpp:67

/// `static f32 size_d[]` (`d_a_bgn3.cpp:69`), the per-segment scale falloff
/// `mParts[i].m00F4 = (REG0_F(11) + 1.7f) * size_d[i]` (`:836`).
pub const SIZE_D: [f32; 9] = [1.0, 0.9, 0.8, 0.7, 0.6, 0.5, 0.4, 0.3, 0.6]; // <- d_a_bgn3.cpp:69

/// `bgn3_ke_s::m00[5]` / `m3C[5]` (`d_a_bgn3.h:10-11`) - the chain length of the body `ke`,
/// whose `ke_control` loop is `for (i = 1; i < 5; i++)` (`:730`).
pub const KE_STEPS: usize = 5; // <- d_a_bgn3.h:10
/// `bgn3_class::mHairs[40]` (`d_a_bgn3.h:49`), drawn by `move`'s
/// `for (s32 i = 0; i < 40; i++)` (`:890`).
pub const KE_HAIRS: usize = 40; // <- d_a_bgn3.h:49
/// `part_s3::mPart3Hairs[40]` (`d_a_bgn3.h:38`), the per-segment array the `j < ke_max[i]`
/// loop writes (`:970-974`); `ke_max[6]` is 15, so 40 is the declared bound, not a bound the
/// code reaches.
pub const PART_KE_MAX: usize = 40; // <- d_a_bgn3.h:38

/// `mParts[10]` (`d_a_bgn3.h:58`). `part_control` integrates `mParts[1..10)`
/// (`for (i = 1; i < 10; i++)`, `:253`), but only indices `0..9` are scaled (`:836`),
/// drawn (`:931`) and pushed into the collision list (`:961`); index 9's `mPart3Sph` is
/// `Set` at `Create` (`:1309-1317`) and never `Set` into `Ccsp`, so it is inert.
pub const PART_CHAIN: usize = 10; // <- d_a_bgn3.cpp:253
/// The nine live segments: every `i < 9` loop in the file (`:171`, `:196`, `:836`, `:931`).
pub const PARTS: usize = 9; // <- d_a_bgn3.cpp:931
/// `i == 8`, the weak point - `mParts[8]` gets `core_sph_src` (`:1311`) and the only model
/// swap in the file (`:939-948`).
pub const PART_CORE: usize = 8; // <- d_a_bgn3.cpp:939
/// `if ((i < 7) && (l_HIO.m3E != 0))` (`:963`) - the segments that carry a `ke` chain.
pub const PART_KE: usize = 7; // <- d_a_bgn3.cpp:963
/// `for (s32 i = 0; i < 8; i++)` (`:703`) - the segments whose `ChkTgHit` starts
/// `def_se_set(actor, atInfo.mpObj, 0x44)`. Eight, not seven.
pub const PART_TG: usize = 8; // <- d_a_bgn3.cpp:703

/// `for (s32 i = 0; i < 5; i++)` (`:1099`). The array walked is declared `s16 m0FDAA[2]`
/// (`d_a_bgn3.h:76`), so entries 2, 3 and 4 alias the fields that follow it: `0x0FDAE
/// m0FDAE`, `0x0FDB0 m0FDB0` and the `0x0FDB2 u8[2]` pad. That over-read is load bearing -
/// see [`Bgn3::cool`].
pub const COOL_SLOTS: usize = 5; // <- d_a_bgn3.cpp:1099

// --------------------------------------------------------------- folded literals

/// `REG0_F(3) + -5.0f` (`d_a_bgn3.cpp:333`). `d_s_play.cpp:81` zeroes every `mFloatRegs`, so
/// the register term is the proven default `0.0f`, not an unknown.
pub const GRAVITY: f32 = -5.0; // <- d_a_bgn3.cpp:333
/// `a_this->current.pos.x/y/z = 0.0f, 30000.0f, 0.0f` (`:1083`) - where the worm is parked
/// until `bgn->m02B5 == 2`.
pub const HIDDEN_Y: f32 = 30000.0; // <- d_a_bgn3.cpp:1083
/// `actor->current.pos.y <= REG0_F(9) + 150.0f` (`:311`).
pub const GROUND_Y: f32 = 150.0; // <- d_a_bgn3.cpp:311
/// `local_ac.z = REG0_F(4) + 150.0f` (`:293`) - how far ahead `checkWall` probes.
pub const WALL_REACH: f32 = 150.0; // <- d_a_bgn3.cpp:293
/// `local_ac.y = 100.0f` (`:292`) and `local_ac.y += 100.0f` (`:297`) - both probe origins.
pub const WALL_RISE: f32 = 100.0; // <- d_a_bgn3.cpp:292
/// `local_d8.set(0.0f, 0.0f, REG0_F(3) + -300.0f)` (`:832`) - the neck anchor's Z offset.
pub const NECK_BACK: f32 = -300.0; // <- d_a_bgn3.cpp:832
/// `MtxTrans(0.0f, 0.0f, REG0_F(2) + 250.0f, true)` (`:902`) - the weak point's neck offset.
pub const NECK_FWD: f32 = 250.0; // <- d_a_bgn3.cpp:902
/// `fVar1 = (70.0f + REG0_F(19)) * ppVar4[-1].m00F4` (`:254`) - the floor under segment Y.
pub const SEG_REACH: f32 = 70.0; // <- d_a_bgn3.cpp:254
/// `dVar10 = (REG0_F(6) + -5.0f)` (`:245`) - the sink applied to segment Y.
pub const SEG_SINK: f32 = -5.0; // <- d_a_bgn3.cpp:245
/// `local_b0.set(0.0f, 0.0f, dVar8 * ppVar4[-1].m00F4)` with `dVar8 = (REG0_F(8) + 150.0f)`
/// (`:244`, `:266`) - the per-segment advance.
pub const SEG_STEP: f32 = 150.0; // <- d_a_bgn3.cpp:244
/// `mParts[i].m00F4 = (REG0_F(11) + 1.7f) * size_d[i]` (`:836`).
pub const SEG_SCALE: f32 = 1.7; // <- d_a_bgn3.cpp:836
/// `ppVar18->mPart3Sph.SetR(ppVar18->m00F4 * (100.0f + REG0_F(12)))` (`:955`).
pub const SEG_RADIUS: f32 = 100.0; // <- d_a_bgn3.cpp:955
/// `local_d8.y = ppVar18->m00F4 * (60.0f + REG13_F(12))` (`:969`) - the `ke` chain root.
pub const SEG_KE_Y: f32 = 60.0; // <- d_a_bgn3.cpp:969
/// `local_d8.y = 100.0f + REG13_F(13)` (`:894`, `:968`) - the body `ke` chain root.
pub const KE_ROOT_Y: f32 = 100.0; // <- d_a_bgn3.cpp:894
/// `cLib_addCalc2(&i_this->m0FDB8, l_HIO.m38, 1.0f, 0.002f)` (`:274`) - the lag's step cap.
pub const LAG_STEP: f32 = 0.002; // <- d_a_bgn3.cpp:274
/// `cLib_addCalc2(&i_this->m0FDBC, l_HIO.m34, 1.0f, 0.5f)` (`:275`).
pub const TAIL_STEP: f32 = 0.5; // <- d_a_bgn3.cpp:275
/// `cLib_addCalcAngleS2(&actor->current.angle.y, sVar3, 4, maxSpeed)` (`:368`) - turn scale.
pub const TURN_SCALE: i16 = 4; // <- d_a_bgn3.cpp:368
/// `cLib_addCalcAngleS2(&actor->shape_angle.y, actor->current.angle.y, 4, 0x1000)` (`:828`).
pub const SHAPE_Y_SCALE: i16 = 4; // <- d_a_bgn3.cpp:828
pub const SHAPE_Y_STEP: i16 = 0x1000; // <- d_a_bgn3.cpp:828
/// `cLib_addCalcAngleS2(&actor->shape_angle.x, actor->current.angle.x, 8, 0x400)` (`:829`).
pub const SHAPE_X_SCALE: i16 = 8; // <- d_a_bgn3.cpp:829
pub const SHAPE_X_STEP: i16 = 0x400; // <- d_a_bgn3.cpp:829
/// `actor->current.angle.x = 0x3000` / `-0x3000` (`:491-494`, `:561-564`) - the pitch clamp.
pub const PITCH_CAP: i16 = 0x3000; // <- d_a_bgn3.cpp:491
/// `actor->current.angle.y += 0x8000` (`:385`, `:453`, `:498`) - the wall-bounce reversal.
pub const HALF_TURN: i16 = 0x8000; // <- d_a_bgn3.cpp:385
/// `i_this->m0FDB8 = REG0_F(5) + 0.9f` (`:386`, `:454`) - the lag reset on a wall hit.
pub const LAG_RESET: f32 = 0.9; // <- d_a_bgn3.cpp:386
/// `i_this->m0FDB8 = 1.0f` (`:468`, `:476`, `:517`, `:531`) - the lag reset on a stun/drop.
pub const LAG_RESET_FULL: f32 = 1.0; // <- d_a_bgn3.cpp:468
/// `actor->speedF > 10.0f` (`:808`) - the crawl-splash and crawl-sound gate.
pub const CRAWL_GATE: f32 = 10.0; // <- d_a_bgn3.cpp:808
/// `uVar2 = local_28.abs() * 3.5f` (`:772`) and `* 2.0f` (`:780`), each clamped to `100`
/// (`:774`, `:781`).
pub const SE_ROTATE_SCALE: f32 = 3.5; // <- d_a_bgn3.cpp:772
pub const SE_ROPE_SCALE: f32 = 2.0; // <- d_a_bgn3.cpp:780
pub const SE_PITCH_MAX: u32 = 100; // <- d_a_bgn3.cpp:774
/// `dVar22 = i_this->m100A8 * (REG0_F(14) + 500.0f)` (`:906`) - the hit-flash sway amplitude.
pub const FLASH_AMPLITUDE: f32 = 500.0; // <- d_a_bgn3.cpp:906
/// `s16 y = dVar22 * cM_ssin(i_this->m0FD88 * 0x2100)` (`:907`) - the flash yaw phase step.
pub const FLASH_PHASE_Y: i16 = 0x2100; // <- d_a_bgn3.cpp:907
/// `s16 x = dVar22 * cM_scos(i_this->m0FD88 * 0x2300)` (`:908`) - the flash pitch phase step.
pub const FLASH_PHASE_X: i16 = 0x2300; // <- d_a_bgn3.cpp:908
/// `cM_ssin(i_this->m0FD88 * (REG0_S(9) + 0x900))` (`:364`) - the orbit sway phase step.
pub const SWAY_PHASE: i16 = 0x900; // <- d_a_bgn3.cpp:364
/// `sVar3 = sVar3 + (s16)((REG0_F(18) + 4000.0f) * dVar6)` (`:366`) - the orbit sway reach.
pub const SWAY_REACH: f32 = 4000.0; // <- d_a_bgn3.cpp:366
/// `i_this->m0FDAE = cM_rndF(50.0f) + 30.0f` (`:412`) - `cM_rndF(range) + low`
/// (`c_math.cpp:185`), NOT `cLib_getRndValue(min, range)` (`c_lib.h:80`), so the two ends are
/// 30.0 and 80.0 and neither is constant.
pub const TURN_ROLL_RANGE: f32 = 50.0; // <- d_a_bgn3.cpp:412
pub const TURN_ROLL_LOW: f32 = 30.0; // <- d_a_bgn3.cpp:412
/// `if (cM_rndF(1.0f) < 0.5f) i_this->m0FD8E *= -1` (`:423`) - a one-sided sign flip.
pub const TURN_FLIP_RANGE: f32 = 1.0; // <- d_a_bgn3.cpp:423
pub const TURN_FLIP_CHANCE: f32 = 0.5; // <- d_a_bgn3.cpp:423
/// `i_this->m0FDAA[1] = 0x14` (`:388`, `:499`) - the post-bounce wall cooldown.
pub const WALL_COOLDOWN: i16 = 0x14; // <- d_a_bgn3.cpp:388
/// `i_this->m0FDB4 = 0x14` (`:635`, `:650`) - the hit cooldown.
pub const HIT_COOLDOWN: i16 = 0x14; // <- d_a_bgn3.cpp:635
/// `i_this->m0FDB6 = 10` (`:710`) - the armour-sound cooldown.
pub const ARMOUR_COOLDOWN: i16 = 10; // <- d_a_bgn3.cpp:710
/// `i_this->m0FDB4 = 10000` (`:676`) - the cooldown once the weak point is broken.
pub const BROKEN_COOLDOWN: i16 = 10000; // <- d_a_bgn3.cpp:676
/// `dComIfGp_getVibration().StartShock(REG0_S(2) + 5, -0x21, cXyz(0.0f, 1.0f, 0.0f))`
/// (`:389`, `:477`, `:532`).
pub const SHOCK_PATTERN: i32 = 5; // <- d_a_bgn3.cpp:389
pub const SHOCK_FLAGS: i32 = -0x21; // <- d_a_bgn3.cpp:389
pub const SHOCK_DIR: Vec3 = Vec3::new(0.0, 1.0, 0.0); // <- d_a_bgn3.cpp:389
/// `dComIfGp_getVibration().StartShock(REG0_S(2) + 3, -0x21, cXyz(0.0f, 1.0f, 0.0f))` (`:600`)
/// - the weaker shock, inside the `mResultingAttackType == 9` arm only.
pub const SHOCK_FACE_PATTERN: i32 = 3; // <- d_a_bgn3.cpp:600

/// `i_this->m100A8 = 0x1e` (`:602`) - a skull-hammer hit's head wobble.
pub const FLASH_FRAME_HAMMER: i16 = 0x1e; // <- d_a_bgn3.cpp:602
/// `i_this->m100A8 = 0x19` (`:606`) - a bomb hit's.
pub const FLASH_FRAME_BOMB: i16 = 0x19; // <- d_a_bgn3.cpp:606
/// `i_this->m100A8 = 0x14` (`:612`) - a critical sword cut's.
pub const FLASH_FRAME_CRITICAL: i16 = 0x14; // <- d_a_bgn3.cpp:612
/// `i_this->m100A8 = 0xf` (`:617`) - a normal sword cut's.
pub const FLASH_FRAME_SWORD: i16 = 0xf; // <- d_a_bgn3.cpp:617
/// `i_this->m100A8 = 10` (`:622`) - an arrow's.
pub const FLASH_FRAME_ARROW: i16 = 10; // <- d_a_bgn3.cpp:622
/// `i_this->m100A8 = 7` (`:626`) - an unclassified hit's.
pub const FLASH_FRAME_OTHER: i16 = 7; // <- d_a_bgn3.cpp:626

/// `dScnPly_ply_c::nextPauseTimer = 5` (`:601`) - a skull hammer only.
pub const PAUSE_HAMMER: i8 = 5; // <- d_a_bgn3.cpp:601
/// `dScnPly_ply_c::nextPauseTimer = 3` (`:611`) - a critical sword cut.
pub const PAUSE_CRITICAL: i8 = 3; // <- d_a_bgn3.cpp:611
/// `dScnPly_ply_c::nextPauseTimer = 2` (`:616`) - a normal sword cut.
pub const PAUSE_SWORD: i8 = 2; // <- d_a_bgn3.cpp:616

/// `def_se_set(actor, atInfo.mpObj, 0x40)` (`:645`) - the head-sphere body-deflect selector.
pub const SE_SELECT_HEAD: u32 = 0x40; // <- d_a_bgn3.cpp:645
/// `def_se_set(actor, atInfo.mpObj, 0x44)` (`:711`) - the body/armour deflect selector.
pub const SE_SELECT_BODY: u32 = 0x44; // <- d_a_bgn3.cpp:711

/// `actor->speed.y = 150.0f` (`:465`, `:513`) - the launch impulse of both `damage` and `end`.
pub const LAUNCH_Y: f32 = 150.0; // <- d_a_bgn3.cpp:465
/// `actor->speedF = 30.0f` (`:466`) - `damage`'s post-stun crawl speed.
pub const DAMAGE_SPEED_F: f32 = 30.0; // <- d_a_bgn3.cpp:466
/// `cLib_addCalc2(&i_this->m100A0, REG0_F(9) + 350.0f, 1.0f, REG0_F(10) + 50.0f)` (`:471`).
pub const DAMAGE_HOVER: f32 = 350.0; // <- d_a_bgn3.cpp:471
pub const DAMAGE_HOVER_STEP: f32 = 50.0; // <- d_a_bgn3.cpp:471
/// `actor->speed.y = 50.0f` (`:473`) - the bounce the second landing starts with.
pub const BOUNCE_Y: f32 = 50.0; // <- d_a_bgn3.cpp:473
/// `i_this->m100A0 = actor->current.pos.y - 150.0f` (`:524`) and the `> 350.0f` clamp (`:525`).
pub const END_HOVER_OFFSET: f32 = 150.0; // <- d_a_bgn3.cpp:524
pub const END_HOVER_MAX: f32 = 350.0; // <- d_a_bgn3.cpp:525
/// `actor->speed.y = REG0_F(11) + 130.0f` (`:534`) - each end bounce's impulse.
pub const END_BOUNCE_Y: f32 = 130.0; // <- d_a_bgn3.cpp:534
/// `if ((i_this->m0FD8C == 4) && (actor->speed.y <= -30.0f))` (`:539`) - the end trigger.
pub const END_TRIGGER_FALL: f32 = -30.0; // <- d_a_bgn3.cpp:539
/// `actor->speed.y += REG0_F(12) + 5.5f` (`:545`) - the final slide's gravity.
pub const END_GRAVITY: f32 = 5.5; // <- d_a_bgn3.cpp:545
/// `cLib_addCalcAngleS2(&i_this->m0FD8E, 5000, 4, 100)` (`:553`) - the final spin-up.
pub const END_TURN_TARGET: i16 = 5000; // <- d_a_bgn3.cpp:553
pub const END_TURN_SCALE: i16 = 4; // <- d_a_bgn3.cpp:553
pub const END_TURN_RATE: i16 = 100; // <- d_a_bgn3.cpp:553
/// `cLib_addCalc2(&i_this->m0FDBC, REG0_F(6) + 50.0f, 1.0f, 1.0f)` (`:554`).
pub const END_TAIL: f32 = 50.0; // <- d_a_bgn3.cpp:554
/// `cLib_addCalc0(&i_this->m100A0, 1.0f, REG0_F(10) + DEMO_SELECT(100.0f, 50.0f))` (`:555`);
/// `DEMO_SELECT(DEMO, RETAIL)` takes the retail arm for `VERSION > VERSION_DEMO`
/// (`global.h:73`), so the cap is 50.0.
pub const END_HOVER_STEP: f32 = 50.0; // <- d_a_bgn3.cpp:555
/// `cLib_addCalc2(&actor->current.pos.x, -15.0f, 0.05f, 30.0f)` (`:573`).
pub const END_SLIDE_X: f32 = -15.0; // <- d_a_bgn3.cpp:573
/// `cLib_addCalc2(&actor->current.pos.z, 375.17f, 0.05f, 30.0f)` (`:574`).
pub const END_SLIDE_Z: f32 = 375.17; // <- d_a_bgn3.cpp:574
pub const END_SLIDE_SCALE: f32 = 0.05; // <- d_a_bgn3.cpp:573
pub const END_SLIDE_STEP: f32 = 30.0; // <- d_a_bgn3.cpp:573

// ---------------------------------------------------- health / state enum members

/// `a_this->health = 3; a_this->max_health = 3;` (`:1318-1319`). `health` is the `s8` of
/// `fopEn_enemy_c` (`d_a_bgn3.h:43`) and is the value every speed and cap ladder in `move0`
/// switches on (`:372-378`, `:392-398`, `:402-408`, `:416-422`).
pub const HEALTH_MAX: i16 = 3; // <- d_a_bgn3.cpp:1318

/// `i_this->m0FD8A`, the action switch of `move` (`:803-824`).
pub const ACT_CRAWL: i16 = 0; // <- d_a_bgn3.cpp:806
pub const ACT_STUN: i16 = 1; // <- d_a_bgn3.cpp:815
pub const ACT_DAMAGE: i16 = 2; // <- d_a_bgn3.cpp:818
pub const ACT_END: i16 = 3; // <- d_a_bgn3.cpp:821
/// `case 10: break;` (`:804`) and `i_this->m0FD8A = 10` (`:1081`, `:1089`) - parked.
pub const ACT_HIDDEN: i16 = 10; // <- d_a_bgn3.cpp:804

/// `i_this->m0FD90 = DEMO_SELECT(90, 180)` (`:1092`) - the turn hold the worm enters with.
pub const ENTER_TURN_TIMER: i16 = 180; // <- d_a_bgn3.cpp:1092
/// `i_this->m0FDB0 = DEMO_SELECT(90, 180)` (`:1093`) - the aim-mode hold.
pub const ENTER_AIM_TIMER: i16 = 180; // <- d_a_bgn3.cpp:1093
/// `if (esa != NULL) i_this->m0FD90 = 2;` (`:349`) - an active `Esa` steals the turn.
pub const ESA_TURN_TIMER: i16 = 2; // <- d_a_bgn3.cpp:349
/// `bgn->m02B5 != 2` (`:1080`) / `bgn->m02B5 == 2` (`:124`) - the only phase it runs in.
pub const PHASE_ACTIVE: i8 = 2; // <- d_a_bgn3.cpp:1080
/// `if (ki_check(i_this) < 0x14)` (`:1121`) - the `fpcNm_KS_e` population cap.
pub const KI_CAP: i32 = 0x14; // <- d_a_bgn3.cpp:1121
/// `fopAcM_create(fpcNm_KS_e, 3, &local_28, fopAcM_GetRoomNo(actor))` (`:1125`) - the
/// `parameters` argument. `g_profile_KS` reads it, not `daBgn3`.
pub const KI_PARAM: u32 = 3; // <- d_a_bgn3.cpp:1125
/// `local_28.y = cM_rndF(500.0f) + 3500.0f` (`:1123`) - the shed `fpcNm_KS_e` spawn height.
pub const KI_SPAWN_RANGE: f32 = 500.0; // <- d_a_bgn3.cpp:1123
pub const KI_SPAWN_LOW: f32 = 3500.0; // <- d_a_bgn3.cpp:1123
/// `local_28.x = cM_rndFX(2500.0f)` and `local_28.z = cM_rndFX(2500.0f)` (`:1122`, `:1124`).
pub const KI_SPAWN_SPREAD: f32 = 2500.0; // <- d_a_bgn3.cpp:1122

// ------------------------------------------------------------------ cM_rnd / cM_rndF

/// `cM_rnd`'s three multipliers (`c_math.cpp:179-181`).
pub const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA]; // <- c_math.cpp:179
/// `cM_rnd`'s three moduli (`c_math.cpp:179-181`).
pub const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673]; // <- c_math.cpp:179
/// `cM_rnd`'s three normalisers: `r0 / 30269.0f + r1 / 30307.0f + r2 / 30323.0f`
/// (`c_math.cpp:183`).
pub const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0]; // <- c_math.cpp:183
/// `cM_initRnd(100, 100, 100)` (`m_Do_machine.cpp:617`) is retail's boot seed for the three
/// `cM_rnd` lanes (`c_math.cpp:170-173`). With a zero seed the LCG is a fixed point and every
/// `cM_rndF` in this file would collapse to `0.0f`.
pub const RND_SEED: i32 = 100; // <- m_Do_machine.cpp:617

// -------------------------------------------------------------------- colliders

/// `cc_sph_src`'s `SrcObjAt Atp = 2` (`d_a_bgn3.cpp:1237`) - the contact damage the nine
/// `cc_sph_src` spheres deal, paired with `SrcObjCo SPrm = Set | IsPlayer | VsEnemy` (`:1241`).
/// `core_sph_src`'s `SrcObjAt Type = 0` (`:1264`) means the tenth deals nothing, which is why
/// `mParts[8]` is excluded from [`Bgn3::touches`].
pub const OBJ_ATP: u8 = 2; // <- d_a_bgn3.cpp:1237
/// `core_sph_src`'s `SrcObjTg Type = AT_TYPE_LIGHT_ARROW` (`d_a_bgn3.cpp:1267`) - only a light
/// arrow can Tg-hit the weak point.
pub const CORE_TG_TYPE: AtkType = AT_TYPE_LIGHT_ARROW; // <- d_a_bgn3.cpp:1267
/// `cc_sph_src`'s `SrcObjTg Type` (`d_a_bgn3.cpp:1239`):
/// `AT_TYPE_ALL & ~AT_TYPE_BOOMERANG & ~AT_TYPE_WATER & ~AT_TYPE_UNK20000 & ~AT_TYPE_WIND &
/// ~AT_TYPE_UNK400000 & ~AT_TYPE_LIGHT`. Written out because `AT_TYPE_ALL` is `~0`
/// (`d_cc_d.h:41`) and `AtkType` is a newtype over `dCcD_ObjAtType`.
pub const OBJ_TG_TYPE: AtkType = AtkType(
    AT_TYPE_ALL.0
        & !AT_TYPE_BOOMERANG.0
        & !AT_TYPE_WATER.0
        & !AT_TYPE_UNK20000.0
        & !AT_TYPE_WIND.0
        & !AT_TYPE_UNK400000.0
        & !AT_TYPE_LIGHT.0,
); // <- d_a_bgn3.cpp:1239
/// `Create`'s `dCcD_SrcSph` tables (`:1232-1259`, `:1260-1287`) and
/// `i_this->mStts.Init(0xFF, 0xFF, a_this)` (`:1302`) build the ten colliders; the `cCcD`
/// registry is a confirmed-absent input, so only the two radii the file overwrites every
/// frame are transcribed, and `mParts[8]`'s `core_sph_src` still governs the Tg mask.
/// `i_this->m0FF28.SetR(REG0_F(1) + 200.0f)` (`:853`) - the body sphere's radius.
pub const RADIUS_BODY: f32 = 200.0; // <- d_a_bgn3.cpp:853
/// `i_this->m0FDFC.SetR(REG0_F(0) + 150.0f)` (`:916`) - the head sphere's radius.
pub const RADIUS_HEAD: f32 = 150.0; // <- d_a_bgn3.cpp:916
/// `MtxScale(REG0_F(10) + 2.0f, REG0_F(10) + 2.0f, REG0_F(10) + 2.0f, true)` (`:848`).
pub const MODEL_SCALE: f32 = 2.0; // <- d_a_bgn3.cpp:848

// ------------------------------------------------------------------- sound ids

/// `JA_SE_LK_ARROW_HIT` (`d_a_bgn3.cpp:659`), `JAZelAudio_SE.h:567`.
pub const SE_LK_ARROW_HIT: u32 = 0x2879; // <- d_a_bgn3.cpp:659
/// `JA_SE_CV_BK_SURPRISE` (`d_a_bgn3.cpp:518`), `JAZelAudio_SE.h:728`. Retail plays it on the
/// PLAYER (`fopAcM_monsSeStart(player, ...)`); `SimEvent::Se` carries no anchor.
pub const SE_CV_BK_SURPRISE: u32 = 0x4830; // <- d_a_bgn3.cpp:518
/// `JA_SE_CV_BGN_HIT_1` (`d_a_bgn3.cpp:537`, `:693`), `JAZelAudio_SE.h:1045`.
pub const SE_CV_BGN_HIT_1: u32 = 0x496E; // <- d_a_bgn3.cpp:537
/// `JA_SE_CV_BGN_HIT_LAST` (`d_a_bgn3.cpp:681`), `JAZelAudio_SE.h:1047`.
pub const SE_CV_BGN_HIT_LAST: u32 = 0x4970; // <- d_a_bgn3.cpp:681
/// `JA_SE_CV_BGN_M_HIT_FACE` (`d_a_bgn3.cpp:596`), `JAZelAudio_SE.h:1050`.
pub const SE_CV_BGN_M_HIT_FACE: u32 = 0x4973; // <- d_a_bgn3.cpp:596
/// `JA_SE_CM_BGN_M_HIT_WALL` (`d_a_bgn3.cpp:390`), `JAZelAudio_SE.h:1474`.
pub const SE_CM_BGN_M_HIT_WALL: u32 = 0x5981; // <- d_a_bgn3.cpp:390
/// `JA_SE_CM_BGN_M_FACE_HIT` (`d_a_bgn3.cpp:597`), `JAZelAudio_SE.h:1475`.
pub const SE_CM_BGN_M_FACE_HIT: u32 = 0x5982; // <- d_a_bgn3.cpp:597
/// `JA_SE_CM_BGN_M_BRK_ORB` (`d_a_bgn3.cpp:682`), `JAZelAudio_SE.h:1476`.
pub const SE_CM_BGN_M_BRK_ORB: u32 = 0x5983; // <- d_a_bgn3.cpp:682
/// `JA_SE_CM_BGN_MECHA_ROTATE` (`d_a_bgn3.cpp:776`), `JAZelAudio_SE.h:2171`.
pub const SE_CM_BGN_MECHA_ROTATE: u32 = 0x705A; // <- d_a_bgn3.cpp:776
/// `JA_SE_CM_BGN_MECHA_ROPE` (`d_a_bgn3.cpp:784`), `JAZelAudio_SE.h:2172`.
pub const SE_CM_BGN_MECHA_ROPE: u32 = 0x705B; // <- d_a_bgn3.cpp:784
/// `JA_SE_CM_BGN_M_CRAWL` (`d_a_bgn3.cpp:811`), `JAZelAudio_SE.h:2174`.
pub const SE_CM_BGN_M_CRAWL: u32 = 0x705D; // <- d_a_bgn3.cpp:811

// ------------------------------------------------------------------ the cull box

/// `fopAc_CULLBOX_0_e` (`d_a_bgn3.cpp:1356`), the profile's `Cull Type`. It resolves to a
/// literal at `f_op_actor_mng.cpp:557`, `l_cullSizeBox[0]`:
/// `fopAc_cullSizeBox(cXyz(-40.0f, 0.0f, -40.0f), cXyz(40.0f, 125.0f, 40.0f))`. The profile
/// carries no `fopAcM_setCullSizeBox` and the actor calls neither `fopAcM_SetMin` nor
/// `fopAcM_SetMax`, so this table entry is the whole box.
pub const CULL_MIN: [f32; 3] = [-40.0, 0.0, -40.0]; // <- f_op_actor_mng.cpp:557
pub const CULL_MAX: [f32; 3] = [40.0, 125.0, 40.0]; // <- f_op_actor_mng.cpp:557

// -------------------------------------------------------------- ke fan constants

/// `cMtx_XrotM(*calc_mtx, -((REG0_S(5) + 0x3FFF & (s32)i * (REG0_S(4) + 1000)) -
/// (REG0_S(6) + 0x2000)))` (`d_a_bgn3.cpp:892`) - the body `ke` hair fan.
pub const KE_LEAN_STEP_X: i32 = 1000; // <- d_a_bgn3.cpp:892
pub const KE_LEAN_BASE_X: i32 = 0x2000; // <- d_a_bgn3.cpp:892
pub const KE_LEAN_MASK_X: i32 = 0x3FFF; // <- d_a_bgn3.cpp:892
/// `cMtx_ZrotM(*calc_mtx, -((REG0_S(8) + 0xFFF & (s32)i * (REG0_S(7) + 5000)) -
/// (REG0_S(9) + 0x800)))` (`d_a_bgn3.cpp:893`).
pub const KE_LEAN_STEP_Z: i32 = 5000; // <- d_a_bgn3.cpp:893
pub const KE_LEAN_BASE_Z: i32 = 0x800; // <- d_a_bgn3.cpp:893
pub const KE_LEAN_MASK_Z: i32 = 0xFFF; // <- d_a_bgn3.cpp:893

/// `cMtx_XrotM(*calc_mtx, (REG0_S(5) + 0x3FFF & j * (REG0_S(4) + 1000)) -
/// (REG0_S(6) + 0x2000))` (`d_a_bgn3.cpp:966`) - the same pair without the leading `-`, the
/// `i < 7` per-segment copy.
fn seg_lean_x(index: i32) -> i16 {
    ((KE_LEAN_MASK_X & index.wrapping_mul(KE_LEAN_STEP_X)) - KE_LEAN_BASE_X) as i16
}

/// `cMtx_ZrotM(*calc_mtx, (REG0_S(8) + 0xFFF & j * (REG0_S(7) + 5000)) -
/// (REG0_S(9) + 0x800))` (`d_a_bgn3.cpp:967`).
fn seg_lean_z(index: i32) -> i16 {
    ((KE_LEAN_MASK_Z & index.wrapping_mul(KE_LEAN_STEP_Z)) - KE_LEAN_BASE_Z) as i16
}

/// The same pair with the leading `-` (`:892-893`), the body `ke` copy.
fn ke_lean_x(index: i32) -> i16 {
    seg_lean_x(index).wrapping_neg()
}

/// The same pair with the leading `-` (`:892-893`), the body `ke` copy.
fn ke_lean_z(index: i32) -> i16 {
    seg_lean_z(index).wrapping_neg()
}

// ------------------------------------------------------------------- sub-state

/// `bgn3_ke_s` (`d_a_bgn3.h:9`): `m00[5]` is the chain, `m3C[5]` the mirrored line material
/// (`ke_move` copies `m00` into `param_1->getPos(param_3)`, `:756-760`) and `m78` the root
/// direction `ke_control` taps (`param_1->m78`, `:732-734`).
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Ke {
    pub head: Vec3,
    pub pos: [Vec3; KE_STEPS],
}

impl Ke {
    pub fn new() -> Self {
        Self {
            head: Vec3::ZERO,
            pos: [Vec3::ZERO; KE_STEPS],
        }
    }
}

impl Default for Ke {
    fn default() -> Self {
        Self::new()
    }
}

/// One `dCcD_Sph`, reduced to what `ChkTgHit` and the `OnAtSetBit`/`OffAtSetBit` pair in `move`
/// need. `Create` also gives it a `dCcD_SrcSph` (`:1232-1259`, `:1260-1287`) whose
/// `dCcD_SrcGObjInf` and source radius are consumed only by the `cCcD` registry, which is a
/// confirmed-absent input; the `SetR` calls in `move` (`:853`, `:916`, `:955`) are the whole
/// collider this port owns.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PartSph {
    pub centre: Vec3,
    pub radius: f32,
    pub at_set: bool,
}

impl PartSph {
    pub fn new() -> Self {
        Self {
            centre: Vec3::ZERO,
            radius: 0.0,
            at_set: false,
        }
    }
}

/// `part_s3` (`d_a_bgn3.h:15`), the live entries of `mParts` (`:930-1015`).
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Part {
    /// `m00D4`, the segment's world position (`:932`).
    pub pos: Vec3,
    /// `m00E8`, the previous `m00D4`, reused as the segment velocity (`:268-272`).
    pub vel: Vec3,
    /// `m00E2`, the segment yaw (`:262`).
    pub angle_y: i16,
    /// `m00E0`, the segment pitch (`:263`).
    pub angle_x: i16,
    /// `m00F4`, the segment scale (`:836`).
    pub scale: f32,
    /// `mPart3Sph` (`:954-961`).
    pub sph: PartSph,
    /// `mPart3Hairs[40]` (`:970-974`), live only for `i < 7` (`:963`).
    pub hair: [Ke; PART_KE_MAX],
}

impl Part {
    pub fn new() -> Self {
        Self {
            pos: Vec3::ZERO,
            vel: Vec3::ZERO,
            angle_y: 0,
            angle_x: 0,
            scale: 0.0,
            sph: PartSph::new(),
            hair: [Ke::new(); PART_KE_MAX],
        }
    }
}

/// `m0FDFC` and `m0FF28`, the two `dCcD_Sph` of `bgn3_class` (`d_a_bgn3.h:85-86`). `m0FDFC` is
/// the head, `m0FF28` the body; both are `Set` into `Ccsp` every `move` (`:854`, `:917`).
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Bgn3Sph {
    pub centre: Vec3,
    pub radius: f32,
    pub at_set: bool,
}

impl Bgn3Sph {
    pub fn new() -> Self {
        Self {
            centre: Vec3::ZERO,
            radius: 0.0,
            at_set: false,
        }
    }
}

/// One `cCcD_Obj` that can reach this actor, as `ChkTgHit` sees it: the AT object itself (for
/// `at_power_check` and `ChkAtType`) and the sphere to test it against. The `cCcD` registry is
/// a confirmed-absent input, so `control` gathers the candidate AT objects itself.
#[derive(Debug, Clone, PartialEq)]
pub struct At {
    pub obj: Atk,
    pub pos: Vec3,
    pub radius: f32,
}

impl At {
    /// `dCcD_Sph::ChkTgHit` against `centre`: the two spheres must overlap.
    pub fn hits(&self, centre: Vec3, radius: f32) -> bool {
        walk::within(self.pos, centre, self.radius + radius)
    }
}

/// `bgn3_class` (`d_a_bgn3.h:42`), field for field, minus everything this port has no input
/// for (each omission is marked at its site).
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Bgn3 {
    /// `a_this->health` (`fopEn_enemy_c`, `d_a_bgn3.h:43`); 3 -> 0 across the three weak points.
    pub health: i16,
    /// `m0FD88`, the global frame counter (`:1097`); the phase of every sway and ripple test.
    pub frame: i16,
    /// `m0FD8A`, the action state.
    pub act: i16,
    /// `m0FD8C`, the per-action sub-state: the second of the actor's two state machines.
    pub sub: i16,
    /// `m0FD8E`, the signed turn rate per frame (`:370`, `:551`).
    pub turn: i16,
    /// `m0FD92`, the wall-hit counter toward the HIO cap (`:399`).
    pub wall_count: i16,
    /// `m0FDB8`, the segment lag, chases `l_HIO.m38` (`:274`).
    pub lag: f32,
    /// `m0FDBC`, the neck rest length, chases `l_HIO.m34` (`:275`).
    pub tail: f32,
    /// `m100A0`, the hover height `damage` and `end` chase (`:471`, `:524`).
    pub hover: f32,
    /// `m0FD7C`, the neck anchor, `current.pos` offset by `REG0_F(3) + -300.0f` on Z (`:834`).
    pub neck: Vec3,
    /// `m100A8`, the head wobble count-down set by `damage_check` and run down in `move`
    /// (`:602-626`, `:903-908`).
    pub flash: i16,
    /// `m100AC`, the frames left of shedding `fpcNm_KS_e` after a wall hit (`:391`).
    pub ks_timer: i32,
    /// `m0FDB4`, the hit cooldown (`:635`, `:650`, `:676`).
    pub cool_hit: i16,
    /// `m0FDB6`, the armour-sound cooldown (`:710`).
    pub cool_armour: i16,
    /// The `for (i = 0; i < 5; i++)` countdown of `:1099`. Because `m0FDAA` is declared
    /// `s16 m0FDAA[2]` (`d_a_bgn3.h:76`) this is NOT five unrelated timers, and the previous
    /// revision of this module got that wrong:
    /// * `[0]` `m0FDAA[0]` - the stun hold-off, read by `mahi` (`:441`) and written by
    ///   `damage_check` (`:631`).
    /// * `[1]` `m0FDAA[1]` - the wall-bounce cooldown (`:383`, `:451`, `:497`).
    /// * `[2]` `m0FDAE` - the turn-rate reroll timer (`:411`).
    /// * `[3]` `m0FDB0` - the aim-mode timer; non-zero picks the origin-facing aim over the
    ///   player orbit (`:357`), and it is set once at `:1093`.
    /// * `[4]` the `0x0FDB2 u8[2]` pad, always zero, so it never decrements.
    pub cool: [i16; COOL_SLOTS],
    /// `mParts`, ten entries (`d_a_bgn3.h:58`).
    pub parts: [Part; PART_CHAIN],
    /// `mHairs[40]` (`d_a_bgn3.h:49`).
    pub hairs: [Ke; KE_HAIRS],
    /// `m0FDFC`, the head sphere.
    pub head: Bgn3Sph,
    /// `m0FF28`, the body sphere.
    pub body: Bgn3Sph,
    /// `cM_rnd`'s three lanes (`c_math.cpp:167`), seeded by `cM_initRnd(100, 100, 100)`
    /// (`m_Do_machine.cpp:617`).
    pub seed: [i32; 3],
    /// `actor->home.pos`, which the worm is snapped to on the tick it leaves `ACT_HIDDEN`
    /// (`:1090`). `fopAcM` writes it from the create position, so it is captured on the first
    /// `Execute` before either early return can park the actor.
    pub home: Vec3,
    /// Whether [`Bgn3::home`] has been captured. `m0FD88` cannot serve: `Execute` returns
    /// before its `++` (`:1097`) while the actor is parked or `bgn` is absent, so `frame`
    /// stays 0 across the whole parked window.
    pub home_set: bool,
}

impl Bgn3 {
    pub fn spawn(parameters: u32) -> Self {
        // `g_profile_BGN3` declares `Parameters = 0` (`d_a_bgn3.cpp:1350`) and this
        // translation unit reads no parameter at all, so `parameters` is deliberately dropped
        // rather than masked into a field nothing reads.
        let _ = parameters;
        Self {
            health: HEALTH_MAX,
            frame: 0,
            act: ACT_HIDDEN,
            sub: 0,
            turn: 0,
            wall_count: 0,
            lag: 0.0,
            tail: 0.0,
            hover: 0.0,
            neck: Vec3::ZERO,
            flash: 0,
            ks_timer: 0,
            cool_hit: 0,
            cool_armour: 0,
            cool: [0; COOL_SLOTS],
            parts: [Part::new(); PART_CHAIN],
            hairs: [Ke::new(); KE_HAIRS],
            head: Bgn3Sph::new(),
            body: Bgn3Sph::new(),
            seed: [RND_SEED; 3],
            home: Vec3::ZERO,
            home_set: false,
        }
    }

    /// `fopAc_CULLBOX_0_e`, the profile's `Cull Type` (`d_a_bgn3.cpp:1356`).
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `daBgn3_Draw` returns `TRUE` with no draw call unless
    /// `bgn != NULL && bgn->m02B5 == 2 && l_HIO.m3E != 0` (`d_a_bgn3.cpp:124-133`), and the
    /// parked state puts the model 30000 units up (`:1083`), so the parked action is the only
    /// case worth skipping.
    pub fn drawn(&self) -> bool {
        self.act != ACT_HIDDEN
    }

    /// `cM_rnd` (`c_math.cpp:177`).
    pub fn rnd(&mut self) -> f32 {
        for lane in 0..3 {
            self.seed[lane] = (self.seed[lane] * RND_MUL[lane]) % RND_MOD[lane];
        }
        let sum = (self.seed[0]) as f32 / RND_SCALE[0]
            + (self.seed[1]) as f32 / RND_SCALE[1]
            + (self.seed[2]) as f32 / RND_SCALE[2];
        (sum % 1.0).abs()
    }

    /// `cM_rndF(f) = cM_rnd() * f` (`c_math.cpp:185`).
    pub fn rnd_f(&mut self, range: f32) -> f32 {
        self.rnd() * range
    }

    /// `cM_rndFX(f) = f * (cM_rnd() - 0.5f) * 2.0f` (`c_math.cpp:190`).
    pub fn rnd_fx(&mut self, range: f32) -> f32 {
        range * (self.rnd() - 0.5) * 2.0
    }

    /// The AT contact damage the actor deals, gated on the AT bit `move` toggles
    /// (`:1016-1022`). `mParts[8]` is excluded because its `core_sph_src` has
    /// `SrcObjAt Type = 0` (`:1264`).
    pub fn touches(&self, at: &At) -> bool {
        if self.body.at_set && at.hits(self.body.centre, self.body.radius) {
            return true;
        }
        for part in self.parts.iter().take(PART_TG) {
            if part.sph.at_set && at.hits(part.sph.centre, part.sph.radius) {
                return true;
            }
        }
        false
    }
}

// -------------------------------------------------------------- lib transcriptions

/// `cLib_addCalc2` (`c_lib.cpp:56`).
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

/// `cLib_addCalc0` (`c_lib.cpp:69`).
fn add_calc0(value: &mut f32, scale: f32, max_step: f32) {
    let mut step = *value * scale;
    if step > max_step {
        step = max_step;
    } else if step < -max_step {
        step = -max_step;
    }
    *value -= step;
}

/// `cM_ssin`.
fn ssin(angle: i16) -> f32 {
    walk::from_s16(angle).sin()
}

/// `cM_scos`.
fn scos(angle: i16) -> f32 {
    walk::from_s16(angle).cos()
}

fn xrot(angle: i16) -> Mat4 {
    let a = walk::from_s16(angle);
    Mat4::from_cols(
        Vec4::X,
        Vec4::new(0.0, a.cos(), a.sin(), 0.0),
        Vec4::new(0.0, -a.sin(), a.cos(), 0.0),
        Vec4::W,
    )
}

fn yrot(angle: i16) -> Mat4 {
    let a = walk::from_s16(angle);
    Mat4::from_cols(
        Vec4::new(a.cos(), 0.0, -a.sin(), 0.0),
        Vec4::Y,
        Vec4::new(a.sin(), 0.0, a.cos(), 0.0),
        Vec4::W,
    )
}

fn zrot(angle: i16) -> Mat4 {
    let a = walk::from_s16(angle);
    Mat4::from_cols(
        Vec4::new(a.cos(), a.sin(), 0.0, 0.0),
        Vec4::new(-a.sin(), a.cos(), 0.0, 0.0),
        Vec4::Z,
        Vec4::W,
    )
}

fn point(m: &Mat4, v: Vec3) -> Vec3 {
    (m * Vec4::new(v.x, v.y, v.z, 1.0)).truncate()
}

impl Bgn3 {
    /// `checkGround` (`d_a_bgn3.cpp:309`).
    pub fn check_ground(&self, actor: &mut Actor) -> bool {
        if actor.pos.y <= GROUND_Y {
            actor.pos.y = GROUND_Y;
            actor.speed.y = 0.0;
            true
        } else {
            false
        }
    }

    /// `checkWall` (`d_a_bgn3.cpp:279`). Retail MUTATES the actor on a hit: it rewinds
    /// `current.pos.x` and `.z` to `old.pos` (`:300-301`) and only then returns `TRUE`, so a
    /// bounce undoes this tick's horizontal move as well as flipping the heading.
    pub fn check_wall(&self, actor: &mut Actor, solid: Option<&Collision>) -> bool {
        let delta = actor.pos - actor.old.pos;
        let flat = Vec3::new(delta.x, 0.0, delta.z);
        if flat.length() <= 0.0 {
            return false;
        }
        let Some(solid) = solid else {
            return false;
        };
        let m = yrot(walk::to_s16(flat.x.atan2(flat.z)));
        let ahead = point(&m, Vec3::new(0.0, WALL_RISE, WALL_REACH)) + actor.pos;
        let from = Vec3::new(actor.pos.x, actor.pos.y + WALL_RISE, actor.pos.z);
        if solid.slide(from, ahead) != ahead {
            actor.pos.x = actor.old.pos.x;
            actor.pos.z = actor.old.pos.z;
            return true;
        }
        false
    }

    /// `pos_move` (`d_a_bgn3.cpp:320`).
    pub fn pos_move(&self, actor: &mut Actor) {
        let m = yrot(actor.angle[1]);
        let v = point(&m, Vec3::new(0.0, 0.0, actor.speed_f));
        actor.speed.x = v.x;
        actor.speed.z = v.z;
        actor.pos += actor.speed;
        actor.speed.y += GRAVITY;
    }

    /// `actor->current.angle.x = -cM_atan2s(local_34.y, sqrtf(SQUARE(local_34.x) +
    /// SQUARE(local_34.z)))` clamped to `+-0x3000` (`d_a_bgn3.cpp:490-495`, `:560-565`).
    pub fn pitch_from_move(&self, actor: &mut Actor) {
        let delta = actor.pos - actor.old.pos;
        let flat = (delta.x * delta.x + delta.z * delta.z).sqrt();
        let pitch = -walk::to_s16(delta.y.atan2(flat));
        actor.angle[0] = pitch.clamp(-PITCH_CAP, PITCH_CAP);
    }

    /// `part_control` (`d_a_bgn3.cpp:231`).
    pub fn part_control(&mut self, actor: &Actor, hio: &Bgn3Hio) {
        let lag = self.lag;
        let base = yrot(actor.shape_angle[1]) * xrot(actor.shape_angle[0]);
        let root = point(&base, Vec3::new(0.0, 0.0, -self.tail));
        self.parts[0].pos = self.neck;
        for i in 1..PART_CHAIN {
            let prev = self.parts[i - 1];
            let next = self.parts[i];
            let reach = SEG_REACH * prev.scale;
            let mut high = next.vel.y + (next.pos.y + SEG_SINK);
            if high <= reach {
                high = reach;
            }
            let flat_x = root.x + (next.vel.x + (next.pos.x - prev.pos.x));
            let rise = high - prev.pos.y;
            let flat_z = root.z + (next.vel.z + (next.pos.z - prev.pos.z));
            let span = (flat_x * flat_x + flat_z * flat_z).sqrt();
            self.parts[i - 1].angle_y = walk::to_s16(flat_x.atan2(flat_z));
            self.parts[i - 1].angle_x = -walk::to_s16(rise.atan2(span));
            let m = yrot(self.parts[i - 1].angle_y) * xrot(self.parts[i - 1].angle_x);
            let step = point(&m, Vec3::new(0.0, 0.0, SEG_STEP * prev.scale));
            self.parts[i].pos = prev.pos + step;
            self.parts[i].vel = lag * (self.parts[i].pos - next.pos);
        }
        add_calc2(&mut self.lag, hio.m38, 1.0, LAG_STEP);
        add_calc2(&mut self.tail, hio.m34, 1.0, TAIL_STEP);
    }

    /// `move0` (`d_a_bgn3.cpp:337`) - the crawl.
    pub fn move0(
        &mut self,
        actor: &mut Actor,
        hio: &Bgn3Hio,
        esa: Option<Vec3>,
        player: Vec3,
        solid: Option<&Collision>,
        shake: &mut Vibration,
        events: &mut SimEvents,
    ) {
        if esa.is_some() {
            self.cool[0] = ESA_TURN_TIMER;
        }
        if self.cool[0] != 0 {
            self.cool[0] -= 1;
            let mut max_step = if self.turn < 0 {
                self.turn.wrapping_neg()
            } else {
                self.turn
            };
            let aim = if self.cool[3] != 0 {
                walk::to_s16((-actor.pos.x).atan2(-actor.pos.z))
            } else if let Some(esa) = esa {
                let d = esa - actor.pos;
                max_step = max_step.wrapping_mul(2);
                walk::to_s16(d.x.atan2(d.z))
            } else {
                let sway = ssin(self.frame.wrapping_mul(SWAY_PHASE));
                let face = walk::to_s16((player.x - actor.pos.x).atan2(player.z - actor.pos.z));
                face.wrapping_add((SWAY_REACH * sway) as i32 as i16)
            };
            super::stream::add_calc_angle_s2(&mut actor.angle[1], aim, TURN_SCALE, max_step);
        } else {
            actor.angle[1] = actor.angle[1].wrapping_add(self.turn);
        }
        let want = if self.health == 3 {
            hio.m08
        } else if self.health == 2 {
            hio.m10
        } else {
            hio.m18
        };
        add_calc2(&mut actor.speed_f, want, 1.0, 3.0);
        self.pos_move(actor);
        self.check_ground(actor);
        let mut reroll = false;
        if self.check_wall(actor, solid) && self.cool[1] == 0 {
            self.cool[0] = 0;
            actor.angle[1] = actor.angle[1].wrapping_add(HALF_TURN);
            self.lag = LAG_RESET;
            reroll = true;
            self.cool[1] = WALL_COOLDOWN;
            events.list.push(SimEvent::Se(SE_CM_BGN_M_HIT_WALL));
            shake.start_shock(SHOCK_PATTERN, SHOCK_FLAGS, SHOCK_DIR);
            self.ks_timer = i32::from(hio.m3c);
            let cap = if self.health == 3 {
                hio.m0e
            } else if self.health == 2 {
                hio.m16
            } else {
                hio.m1e
            };
            self.wall_count += 1;
            if self.wall_count >= cap {
                self.wall_count = 0;
                self.cool[0] = if self.health == 3 {
                    hio.m20
                } else if self.health == 2 {
                    hio.m22
                } else {
                    hio.m24
                };
            }
        }
        if self.cool[2] == 0 {
            let roll = self.rnd_f(TURN_ROLL_RANGE) + TURN_ROLL_LOW;
            self.cool[2] = roll as i16;
            reroll = true;
        }
        if reroll {
            self.turn = if self.health == 3 {
                hio.m0c
            } else if self.health == 2 {
                hio.m14
            } else {
                hio.m1c
            };
            if self.rnd_f(TURN_FLIP_RANGE) < TURN_FLIP_CHANCE {
                self.turn = self.turn.wrapping_neg();
            }
        }
        self.body.at_set = true;
        self.head.at_set = true;
        actor.angle[0] = 0;
    }

    /// `mahi` (`d_a_bgn3.cpp:433`) - the stun. `case 0` FALLS THROUGH into `case 1` (`:438`),
    /// so the first tick both flips the sub-state and runs the shared body.
    pub fn mahi(&mut self, actor: &mut Actor, solid: Option<&Collision>) {
        match self.sub {
            0 => {
                self.sub = 1;
                self.mahi_1();
            }
            1 => self.mahi_1(),
            _ => {}
        }
        add_calc0(&mut actor.speed_f, 1.0, 3.0);
        self.pos_move(actor);
        actor.angle[0] = 0;
        self.check_ground(actor);
        if self.check_wall(actor, solid) && self.cool[1] == 0 {
            self.cool[0] = 0;
            actor.angle[1] = actor.angle[1].wrapping_add(HALF_TURN);
            self.lag = LAG_RESET;
        }
    }

    /// The shared body of `mahi`'s `case 0` fall-through and its `case 1` (`:439-445`).
    /// `i_this->mpMorf->play(&actor->current.pos, 0, 0)` on both paths advances a frame
    /// controller whose `getFrameMax()` lives in
    /// `dRes_INDEX_BGN_BCK_BGN_HEAD1_e`, which is not in this checkout: omitted, never stood
    /// in for.
    fn mahi_1(&mut self) {
        if self.cool[0] == 0 {
            self.act = ACT_CRAWL;
            self.sub = 0;
        }
    }

    /// `damage` (`d_a_bgn3.cpp:459`) - the weak-point-broken reaction.
    pub fn damage(&mut self, actor: &mut Actor, solid: Option<&Collision>, shake: &mut Vibration) {
        match self.sub {
            0 => {
                actor.speed.y = LAUNCH_Y;
                actor.speed_f = DAMAGE_SPEED_F;
                self.sub = 1;
                self.lag = LAG_RESET_FULL;
            }
            1 => {
                add_calc2(&mut self.hover, DAMAGE_HOVER, 1.0, DAMAGE_HOVER_STEP);
                if self.check_ground(actor) {
                    actor.speed.y = BOUNCE_Y;
                    self.sub = 2;
                    actor.angle[1] = (self.rnd_f(0x10000) as i32) as i16;
                    self.lag = LAG_RESET_FULL;
                    shake.start_shock(SHOCK_PATTERN, SHOCK_FLAGS, SHOCK_DIR);
                }
            }
            2 => {
                if self.check_ground(actor) {
                    self.act = ACT_CRAWL;
                    self.sub = 0;
                }
            }
            _ => {}
        }
        self.pos_move(actor);
        self.pitch_from_move(actor);
        self.check_ground(actor);
        if self.check_wall(actor, solid) && self.cool[1] == 0 {
            actor.angle[1] = actor.angle[1].wrapping_add(HALF_TURN);
            self.cool[1] = WALL_COOLDOWN;
        }
        self.body.at_set = false;
        self.head.at_set = false;
    }

    /// `end` (`d_a_bgn3.cpp:506`) - the final action: five bounces then a slide off stage.
    pub fn end(&mut self, actor: &mut Actor, shake: &mut Vibration, events: &mut SimEvents) {
        match self.sub {
            0 => {
                actor.speed.y = LAUNCH_Y;
                actor.speed_f = 0.0;
                self.sub = 1;
                // `bgn->mCSMode = 20` (`d_a_bgn3.cpp:516`, `s8 mCSMode` at `d_a_bgn.h:173`)
                // writes a field of the SIBLING actor. `super::puppet_ganon::PuppetGanon`
                // carries no `mCSMode`, and a value parked on a resource nobody reads is
                // inert, so it is omitted rather than invented into a dead field.
                self.lag = LAG_RESET_FULL;
                events.list.push(SimEvent::Se(SE_CV_BK_SURPRISE));
            }
            1..=4 => {
                self.hover = actor.pos.y - END_HOVER_OFFSET;
                if self.hover > END_HOVER_MAX {
                    self.hover = END_HOVER_MAX;
                }
                if self.check_ground(actor) {
                    self.sub += 1;
                    actor.angle[1] = (self.rnd_f(0x10000) as i32) as i16;
                    self.lag = LAG_RESET_FULL;
                    shake.start_shock(SHOCK_PATTERN, SHOCK_FLAGS, SHOCK_DIR);
                    actor.speed.y = END_BOUNCE_Y;
                    actor.speed_f = 0.0;
                    actor.angle[1] = (self.rnd_f(0x10000) as i32) as i16;
                    events.list.push(SimEvent::Se(SE_CV_BGN_HIT_1));
                }
                if self.sub == 4 && actor.speed.y <= END_TRIGGER_FALL {
                    self.sub = 5;
                    self.turn = 0;
                }
            }
            5 => {
                actor.speed.y += END_GRAVITY;
                // `if (actor->speed.y > 0.0) { cLib_addCalc2(&i_this->m10060, 1.0f, 1.0f,
                // 0.015f); }` (`:547-549`) writes `m10060`, which is read nowhere in the file
                // and outside it only a renderer could, so the whole branch is omitted.
                actor.angle[1] = actor.angle[1].wrapping_add(self.turn);
                actor.shape_angle[1] = actor.shape_angle[1].wrapping_add(self.turn);
                super::stream::add_calc_angle_s2(
                    &mut self.turn,
                    END_TURN_TARGET,
                    END_TURN_SCALE,
                    END_TURN_RATE,
                );
                add_calc2(&mut self.tail, END_TAIL, 1.0, 1.0);
                add_calc0(&mut self.hover, 1.0, END_HOVER_STEP);
            }
            _ => {}
        }
        self.pos_move(actor);
        self.pitch_from_move(actor);
        self.check_ground(actor);
        self.body.at_set = false;
        self.head.at_set = false;
        add_calc2(&mut actor.pos.x, END_SLIDE_X, END_SLIDE_SCALE, END_SLIDE_STEP);
        add_calc2(&mut actor.pos.z, END_SLIDE_Z, END_SLIDE_SCALE, END_SLIDE_STEP);
    }

    /// `ke_control` (`d_a_bgn3.cpp:717`).
    pub fn ke_control(ke: &mut Ke, scale: f32, hio: &Bgn3Hio) {
        let step = Vec3::new(0.0, 0.0, scale);
        for i in 1..KE_STEPS {
            let taper = 0.1 * (KE_STEPS - i) as f32;
            let push = ke.head * taper;
            let flat_x = push.x + (ke.pos[i].x - ke.pos[i - 1].x);
            let flat_z = push.z + (ke.pos[i].z - ke.pos[i - 1].z);
            let rise = hio.m4c + (ke.pos[i].y + push.y) - ke.pos[i - 1].y;
            let pitch = -walk::to_s16(rise.atan2(flat_z));
            let yaw = walk::to_s16(flat_x.atan2((flat_x * flat_x + flat_z * flat_z).sqrt()));
            let m = xrot(pitch) * yrot(yaw);
            ke.pos[i] = ke.pos[i - 1] + point(&m, step);
        }
    }

    /// `ke_move` (`d_a_bgn3.cpp:750`).
    pub fn ke_move(ke: &mut Ke, scale: f32, hio: &Bgn3Hio) {
        if hio.m3e == 0 {
            return;
        }
        Self::ke_control(ke, scale, hio);
    }

    /// `move_se_set` (`d_a_bgn3.cpp:765`) - the two speed-pitched mech sounds. `SimEvent::Se`
    /// carries no pitch argument, so the pitch the reference clamps before the call is
    /// computed and dropped rather than invented into an event field.
    pub fn move_se_set(&self, actor: &Actor, events: &mut SimEvents) {
        let mut delta = actor.pos - actor.old.pos;
        delta.y = 0.0;
        let rotate = (delta.length() * SE_ROTATE_SCALE) as u32;
        events.list.push(SimEvent::Se(SE_CM_BGN_MECHA_ROTATE));
        let _ = rotate.min(SE_PITCH_MAX);
        delta = actor.pos - actor.old.pos;
        delta.z = 0.0;
        delta.x = 0.0;
        let rope = (delta.length() * SE_ROPE_SCALE) as u32;
        events.list.push(SimEvent::Se(SE_CM_BGN_MECHA_ROPE));
        let _ = rope.min(SE_PITCH_MAX);
    }

    /// `damage_check` (`d_a_bgn3.cpp:579`). The head arm runs the real `at_power_check`
    /// (`d_cc_uty.cpp:87`) because `mResultingAttackType` is the only thing that picks a
    /// branch, and reproduces the decomp's ladder in its order.
    pub fn damage_check(
        &mut self,
        hits: &[At],
        hio: &Bgn3Hio,
        shake: &mut Vibration,
        stop: &mut HitStop,
        audio: &mut HitAudio,
        events: &mut SimEvents,
    ) {
        if self.cool_hit != 0 {
            return;
        }
        if let Some(hit) = hits
            .iter()
            .find(|at| {
                at.obj.chk_at_type(OBJ_TG_TYPE) && at.hits(self.head.centre, self.head.radius)
            })
        {
            let mut info = CcAtInfo {
                mp_obj: Some(hit.obj.clone()),
                p_particle_pos: None,
                ..Default::default()
            };
            // `at_power_check` reads `PlayerCut` only to fill `mPlCutBit` and `mHitSoundId`
            // (`d_cc_uty.cpp:123-137`), and `damage_check` reads neither, so the player's cut
            // identity cannot change which arm is taken.
            at_power_check(&mut info, &PlayerCut::default());
            // `fopAcM_monsSeStart(actor, JA_SE_CV_BGN_M_HIT_FACE, 0)` and
            // `fopAcM_seStart(actor, JA_SE_CM_BGN_M_FACE_HIT, 0)` (`:596-597`), both AFTER
            // `at_power_check` has written `mResultingAttackType`.
            events.list.push(SimEvent::Se(SE_CV_BGN_M_HIT_FACE));
            events.list.push(SimEvent::Se(SE_CM_BGN_M_FACE_HIT));
            let result = info.m_resulting_attack_type;
            let arrow = info
                .mp_actor
                .as_ref()
                .is_some_and(|attacker| attacker.name == NAME_ARROW);
            let (frame, stun) = if result == RESULT_SKULL_HAMMER {
                shake.start_shock(SHOCK_FACE_PATTERN, SHOCK_FLAGS, SHOCK_DIR);
                stop.set_pause_timer(PAUSE_HAMMER);
                (FLASH_FRAME_HAMMER, hio.m30)
            } else if result == RESULT_BOMB {
                (FLASH_FRAME_BOMB, hio.m2e)
            } else if result == RESULT_SWORD && hit.obj.at_spl == AT_SPL_UNK1 {
                stop.set_pause_timer(PAUSE_CRITICAL);
                (FLASH_FRAME_CRITICAL, hio.m2c)
            } else if result == RESULT_SWORD {
                stop.set_pause_timer(PAUSE_SWORD);
                (FLASH_FRAME_SWORD, hio.m2a)
            } else if arrow {
                (FLASH_FRAME_ARROW, hio.m28)
            } else {
                (FLASH_FRAME_OTHER, hio.m26)
            };
            if self.act != ACT_STUN {
                self.cool[0] = stun;
                self.act = ACT_STUN;
            }
            self.sub = 0;
            self.cool_hit = HIT_COOLDOWN;
            self.flash = frame;
            def_se_set(&hit.obj, SE_SELECT_HEAD, audio, events);
            return;
        }
        let core = self.parts[PART_CORE].sph;
        let core_hit = hits
            .iter()
            .any(|at| at.obj.chk_at_type(CORE_TG_TYPE) && at.hits(core.centre, core.radius));
        if core_hit {
            self.cool_hit = HIT_COOLDOWN;
            self.act = ACT_DAMAGE;
            self.sub = 0;
            if self.health != 0 {
                self.health -= 1;
                events.list.push(SimEvent::Se(SE_LK_ARROW_HIT));
                if self.health == 0 {
                    // `mDoAud_bgmStop(30)` (`:663`) has no `SimEvent` variant; the two ids
                    // around it are transcribed.
                    events.list.push(SimEvent::Se(SE_CV_BGN_HIT_LAST));
                    events.list.push(SimEvent::Se(SE_CM_BGN_M_BRK_ORB));
                    self.act = ACT_END;
                    self.sub = 0;
                    self.cool_hit = BROKEN_COOLDOWN;
                } else {
                    events.list.push(SimEvent::Se(SE_CV_BGN_HIT_1));
                }
            }
        }
        // `cVar5 = 0;` (`:698`), then `1` from the body (`:699-702`) and `2` from any segment
        // (`:703-708`), so the object `def_se_set(actor, atInfo.mpObj, 0x44)` names is the LAST
        // segment that hit, and the body only when no segment did.
        let mut struck: Option<&At> = None;
        for at in hits.iter() {
            if !at.obj.chk_at_type(OBJ_TG_TYPE) {
                continue;
            }
            if at.hits(self.body.centre, self.body.radius) {
                struck = Some(at);
            }
        }
        for i in 0..PART_TG {
            let sphere = self.parts[i].sph;
            for at in hits.iter() {
                if at.obj.chk_at_type(OBJ_TG_TYPE) && at.hits(sphere.centre, sphere.radius) {
                    struck = Some(at);
                }
            }
        }
        if struck.is_some() && self.cool_armour == 0 {
            self.cool_armour = ARMOUR_COOLDOWN;
            if let Some(hit) = struck {
                def_se_set(&hit.obj, SE_SELECT_BODY, audio, events);
            }
        }
    }

    /// `move` (`d_a_bgn3.cpp:788`).
    pub fn move(
        &mut self,
        actor: &mut Actor,
        hio: &Bgn3Hio,
        esa: Option<Vec3>,
        player: Vec3,
        solid: Option<&Collision>,
        shake: &mut Vibration,
        stop: &mut HitStop,
        audio: &mut HitAudio,
        hits: &[At],
        events: &mut SimEvents,
    ) {
        let mut crawling = false;
        if hio.m05 == 0 {
            match self.act {
                ACT_HIDDEN => {}
                ACT_CRAWL => {
                    self.move0(actor, hio, esa, player, solid, shake, events);
                    if actor.speed_f > CRAWL_GATE {
                        crawling = true;
                        events.list.push(SimEvent::Se(SE_CM_BGN_M_CRAWL));
                        self.move_se_set(actor, events);
                    }
                }
                ACT_STUN => self.mahi(actor, solid),
                ACT_DAMAGE => self.damage(actor, solid, shake),
                ACT_END => self.end(actor, shake, events),
                _ => {}
            }
        }
        self.damage_check(hits, hio, shake, stop, audio, events);
        super::stream::add_calc_angle_s2(
            &mut actor.shape_angle[1],
            actor.angle[1],
            SHAPE_Y_SCALE,
            SHAPE_Y_STEP,
        );
        super::stream::add_calc_angle_s2(
            &mut actor.shape_angle[0],
            actor.angle[0],
            SHAPE_X_SCALE,
            SHAPE_X_STEP,
        );
        let base = yrot(actor.shape_angle[1]) * xrot(actor.shape_angle[0]);
        self.neck = actor.pos + point(&base, Vec3::new(0.0, 0.0, NECK_BACK));
        for i in 0..PARTS {
            self.parts[i].scale = SEG_SCALE * SIZE_D[i];
        }
        self.part_control(actor, hio);
        // `mpMorf->play()`, `mJyakutenCBrkAnm->play()` and `mJyakutenBBrkAnm->play()`
        // (`:839-841`) advance frame controllers whose `getFrameMax()` lives in the `Bgn`
        // archives, which are not in this checkout: omitted, never stood in for.
        let model =
            base * zrot(actor.shape_angle[2]) * Mat4::from_scale(Vec3::splat(MODEL_SCALE));
        self.body.centre = point(&model, Vec3::ZERO);
        self.body.radius = RADIUS_BODY;
        for i in 0..KE_HAIRS {
            let m = model * xrot(ke_lean_x(i as i32)) * zrot(ke_lean_z(i as i32));
            let mut ke = self.hairs[i];
            ke.pos[0] = point(&m, Vec3::new(0.0, KE_ROOT_Y, 0.0));
            ke.head = point(&m, Vec3::new(0.0, hio.m40, 0.0)) - self.body.centre;
            Self::ke_move(&mut ke, 2.0 * hio.m44, hio);
            self.hairs[i] = ke;
        }
        if self.flash != 0 {
            self.flash -= 1;
        }
        let sway = f32::from(self.flash) * FLASH_AMPLITUDE;
        let head = base
            * zrot(actor.shape_angle[2])
            * Mat4::from_translation(Vec3::new(0.0, 0.0, NECK_FWD))
            * yrot((sway * ssin(self.frame.wrapping_mul(FLASH_PHASE_Y))) as i16)
            * xrot((sway * scos(self.frame.wrapping_mul(FLASH_PHASE_X))) as i16);
        self.head.centre = point(&head, Vec3::ZERO);
        self.head.radius = RADIUS_HEAD;
        for i in 0..PARTS {
            let part = self.parts[i];
            let m = Mat4::from_translation(part.pos)
                * yrot(part.angle_y)
                * xrot(part.angle_x)
                * Mat4::from_scale(Vec3::splat(part.scale));
            let centre = point(&m, Vec3::ZERO);
            self.parts[i].sph.centre = centre;
            self.parts[i].sph.radius = part.scale * SEG_RADIUS;
            self.parts[i].sph.at_set = crawling;
            if i < PART_KE && hio.m3e != 0 {
                for j in 0..usize::from(KE_MAX[i]) {
                    let km = m
                        * xrot(seg_lean_x(j as i32))
                        * zrot(seg_lean_z(j as i32))
                        * Mat4::from_translation(Vec3::new(0.0, part.scale * SEG_KE_Y, 0.0));
                    let mut ke = part.hair[j];
                    ke.pos[0] = point(&km, Vec3::ZERO);
                    ke.head = point(&km, Vec3::new(0.0, hio.m40, 0.0)) - centre;
                    Self::ke_move(&mut ke, part.scale * hio.m44, hio);
                    self.parts[i].hair[j] = ke;
                }
            }
        }
        self.head.at_set = crawling;
        self.body.at_set = crawling;
    }

    /// `daBgn3_Execute` (`d_a_bgn3.cpp:1068`). Returns the `fpcNm_KS_e` create the reference
    /// performs at `:1125` as `(parameters, position, room)` for the caller to act on.
    pub fn execute(
        &mut self,
        actor: &mut Actor,
        stat: &Bgn3Static,
        sib: &Bgn3Siblings,
        player: Vec3,
        solid: Option<&Collision>,
        shake: &mut Vibration,
        stop: &mut HitStop,
        audio: &mut HitAudio,
        hits: &[At],
        events: &mut SimEvents,
    ) -> Option<(u32, Vec3, i8)> {
        if !self.home_set {
            self.home_set = true;
            self.home = actor.pos;
        }
        if !sib.bgn_found {
            return None;
        }
        let hio = stat.hio;
        if hio.m06 != 0 {
            self.health = i16::from(hio.m06);
        }
        let esa = if sib.esa_found {
            Some(sib.esa_pos)
        } else {
            None
        };
        if sib.bgn_phase != PHASE_ACTIVE {
            self.act = ACT_HIDDEN;
            actor.pos = Vec3::new(0.0, HIDDEN_Y, 0.0);
            // `fopAcM_OffStatus(actor, 0)` (`:1085`) clears every status bit, including the
            // profile's `fopAcStts_UNK4000_e | UNK40000_e | BOSS_e` (`:1354`), and
            // `actor->attention_info.flags = 0` (`:1086`) blanks the lock-on. Per-actor
            // status bits and `attention_info` are confirmed-absent inputs, so both are
            // omitted rather than faked.
            return None;
        }
        if self.act == ACT_HIDDEN {
            actor.pos = self.home;
            self.act = ACT_CRAWL;
            self.cool[0] = ENTER_TURN_TIMER;
            self.cool[3] = ENTER_AIM_TIMER;
        }
        // `cLib_addCalc2(&i_this->m10060, 0.0f, 1.0f, 0.01f)` (`:1096`) has no consumer in
        // this sim either: `m10060` is read nowhere in the file and outside it only a
        // renderer could. Omitted rather than invented.
        self.frame = self.frame.wrapping_add(1);
        for slot in self.cool.iter_mut() {
            if *slot != 0 {
                *slot -= 1;
            }
        }
        if self.cool_hit != 0 {
            self.cool_hit -= 1;
        }
        if self.cool_armour != 0 {
            self.cool_armour -= 1;
        }
        self.move(
            actor, &hio, esa, player, solid, shake, stop, audio, hits, events,
        );
        if self.ks_timer != 0 {
            self.ks_timer -= 1;
            // `ki_check(i_this) < 0x14` (`:1121`) over `ki_all_count`, which `ki_check` itself
            // resets and `ki_c_sub` recounts (`:1061-1064`).
            if sib.ki_count < KI_CAP {
                let x = self.rnd_fx(KI_SPAWN_SPREAD);
                let y = self.rnd_f(KI_SPAWN_RANGE) + KI_SPAWN_LOW;
                let z = self.rnd_fx(KI_SPAWN_SPREAD);
                return Some((KI_PARAM, Vec3::new(x, y, z), actor.room));
            }
        }
        None
    }
}

// ------------------------------------------------------------------- the system

#[derive(SystemParam)]
pub struct Watch<'w, 's> {
    collision: Option<Res<'w, Collision>>,
    combat: Res<'w, Combat>,
    held: Res<'w, Held>,
    players: Query<'w, 's, (Entity, &'static Player), Without<Bgn3>>,
    arrows: Query<'w, 's, (&'static Actor, &'static arrow::Arrow), Without<Bgn3>>,
    bgns: Query<'w, 's, (&'static Actor, &'static PuppetGanon), Without<Bgn3>>,
    esas: Query<'w, 's, (&'static Actor, &'static Esa), Without<Bgn3>>,
    kis: Query<'w, 's, &'static Actor, Without<Bgn3>>,
}

pub fn control(
    watch: Watch<'_, '_>,
    mut stat: ResMut<Bgn3Static>,
    mut sib: ResMut<Bgn3Siblings>,
    mut events: ResMut<SimEvents>,
    mut audio: ResMut<HitAudio>,
    mut stop: ResMut<HitStop>,
    mut shake: ResMut<Vibration>,
    mut health: ResMut<Health>,
    mut bosses: Query<(Entity, &mut Actor, &mut Bgn3), (With<Bgn3>, Without<Player>)>,
) {
    let solid = watch.collision.as_deref();
    let mut bgn: Option<(Vec3, i8)> = None;
    for (actor, boss) in watch.bgns.iter() {
        if bgn.is_none() {
            bgn = Some((actor.pos, boss.hio_phase as i8));
        }
    }
    sib.bgn_found = bgn.is_some();
    sib.bgn_phase = bgn.map_or(0, |(_, phase)| phase);
    if let Some((pos, _)) = bgn {
        sib.bgn_pos = pos;
    }
    let esa = watch.esas.iter().next().map(|(actor, _esa)| actor.pos);
    sib.esa_found = esa.is_some();
    if let Some(esa) = esa {
        sib.esa_pos = esa;
    }
    sib.ki_count = watch.kis.iter().filter(|actor| is_ks(&actor.name)).count() as i32;
    if !stat.hio_set {
        // `if (hio_set == false) { i_this->m100B0 = 1; hio_set = true; l_HIO.mNo =
        // mDoHIO_createChild(...); }` (`d_a_bgn3.cpp:1297-1301`), and
        // `ke_color.r/g/b/a = l_HIO.m50 / m52 / m54 / 0xFF` (`:1320-1323`).
        stat.hio_set = true;
        stat.ke_color = [stat.hio.m50, stat.hio.m52, stat.hio.m54, 0xFF];
    }

    let mut hits: Vec<At> = Vec::new();
    for (_, body) in watch.players.iter() {
        // `combat.rs`'s own damage window: `DAMAGE_START = 1`, `DAMAGE_END = 6`, which are
        // the frames the sword's `mAtCps[0..3)` are `OnAtSetBit` for.
        if watch.combat.active && watch.combat.frame >= 1 && watch.combat.frame < 6 {
            hits.push(At {
                obj: Atk {
                    at_type: AT_TYPE_SWORD,
                    at_se: AT_SE_NONE,
                    at_spl: cut_spl(watch.combat.kind),
                    at_atp: damage::sword_power(watch.held.item, watch.combat.soup) as u8,
                    ac: Some(Attacker {
                        name: NAME_PLAYER.to_owned(),
                        pos: body.pos,
                        speed: Vec3::ZERO,
                        bomb_state: 0,
                        tsubo_type: 0,
                    }),
                },
                pos: body.pos + Vec3::new(0.0, 0.0, SWORD_REACH),
                radius: SWORD_RADIUS,
            });
        }
    }
    for (_, shaft) in watch.arrows.iter() {
        if shaft.phase != arrow::Phase::Flying {
            continue;
        }
        let at_type = match shaft.kind {
            arrow::TYPE_LIGHT => AT_TYPE_LIGHT_ARROW,
            arrow::TYPE_FIRE => AT_TYPE_FIRE_ARROW,
            arrow::TYPE_ICE => AT_TYPE_ICE_ARROW,
            _ => AT_TYPE_NORMAL_ARROW,
        };
        // `cXyz end = current.pos + speed*1.25f; mAtCps.SetStartEnd(current.pos, end);
        // mAtCps.SetR(5.0f)` (`d_a_arrow.cpp:293-295`). A capsule is exactly the union of the
        // spheres of radius `r + half_length` about its midpoint, so pairing the midpoint
        // with that radius reproduces the retail overlap test against `m0FDFC` and
        // `mParts[8].mPart3Sph` without approximating it.
        let tip = shaft.pos + shaft.speed * ARROW_STRETCH;
        let centre = (shaft.pos + tip) * 0.5;
        let reach = ARROW_RADIUS + (tip - shaft.pos).length() * 0.5;
        hits.push(At {
            obj: Atk {
                at_type,
                at_se: AT_SE_ARROW,
                at_spl: AT_SPL_UNK0,
                at_atp: damage::ARROW as u8,
                ac: Some(Attacker {
                    name: NAME_ARROW.to_owned(),
                    pos: shaft.pos,
                    speed: shaft.speed,
                    bomb_state: 0,
                    tsubo_type: 0,
                }),
            },
            pos: centre,
            radius: reach,
        });
    }
    // `fpcNm_BOMB_e` builds its attack object through the shared blast channel, which this
    // sim carries as positions on `super::bombflower::Blasts` rather than as a `cCcD_Obj`
    // with a radius anywhere in it, so no bomb AT object can be offered to `ChkTgHit` here.
    // The `mResultingAttackType == 2` arm of `damage_check` is transcribed but has no input.
    let player = watch
        .players
        .iter()
        .next()
        .map_or(Vec3::ZERO, |(_, body)| body.pos);
    for (_, mut actor, mut boss) in bosses.iter_mut() {
        let actor = &mut *actor;
        let boss = &mut *boss;
        let spawned = boss.execute(
            actor,
            &stat,
            &sib,
            player,
            solid,
            &mut shake,
            &mut stop,
            &mut audio,
            &hits,
            &mut events,
        );
        // `fopAcM_create(fpcNm_KS_e, 3, &local_28, fopAcM_GetRoomNo(actor))` (`:1125`) has to
        // go through `spawn_actor`, which `src/sim/mod.rs` owns and this file may not touch.
        // The request is computed here in full and dropped with its three arguments intact;
        // see WIRING NEEDED in the audit report.
        let _ = spawned;
        for at in hits.iter() {
            if boss.touches(at) {
                health.damage(OBJ_ATP);
            }
        }
    }
}

/// `setAtParam`'s `spl` argument (`d_a_player_main.cpp:9577`) as `src/sim/combat.rs` feeds it:
/// the finish cuts take `dCcG_At_Spl_UNK1` (`d_cc_d.h:108`), every other cut
/// `dCcG_At_Spl_UNK0` (`:107`).
fn cut_spl(kind: CutKind) -> u8 {
    match kind {
        CutKind::EA | CutKind::EB => AT_SPL_UNK1,
        _ => AT_SPL_UNK0,
    }
}

/// The player's sword AT object is a `cCcD_Cps` of radius `20.0f`
/// (`d_a_player_main_data.inc:159`) whose start and end are the sword joint positions
/// (`daPy_lk_c::setSwordAtCollision`, `d_a_player_main.cpp:9670-9685`). Joint matrices are a
/// confirmed-absent input, so the reach is the one `src/sim/combat.rs` uses for the same
/// object: 20.0 units along the facing, 20.0 radius.
const SWORD_REACH: f32 = 20.0; // <- d_a_player_main_data.inc:159
const SWORD_RADIUS: f32 = 20.0; // <- d_a_player_main_data.inc:159
/// `mAtCps.SetR(5.0f)` (`d_a_arrow.cpp:295`), matching `daArrow_c::m_at_cps_src`'s
/// `cM3dGCpsS Radius 5.0f` (`d_a_arrow.cpp:49`).
const ARROW_RADIUS: f32 = 5.0; // <- d_a_arrow.cpp:295
/// `cXyz end = current.pos + speed*1.25f` (`d_a_arrow.cpp:293`).
const ARROW_STRETCH: f32 = 1.25; // <- d_a_arrow.cpp:293