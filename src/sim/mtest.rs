use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::flags::Switches;

/// The eight `fpcNm_Obj_Mtest_e` rows in `d_stage.cpp`, each with the `sub` byte that becomes the
/// actor's size bit 0 - so the stage string and `daObjMtest::Act_c::Type` agree one-for-one:
///
/// | stage string | `d_stage.cpp` | `fpcNm` | `Type` member | `Type` value |
/// |---|---|---|---|---|
/// | `"Mcube"`   | `:895` | `fpcNm_Obj_Mtest_e` | `Type_Mcube`   | 0 |
/// | `"Mcyln"`   | `:896` | `fpcNm_Obj_Mtest_e` | `Type_Mcyln`   | 1 |
/// | `"Mcube10"` | `:897` | `fpcNm_Obj_Mtest_e` | `Type_Mcube10` | 2 |
/// | `"Mcyln10"` | `:898` | `fpcNm_Obj_Mtest_e` | `Type_Mcyln10` | 3 |
/// | `"MwtrSB"`  | `:899` | `fpcNm_Obj_Mtest_e` | `Type_MwtrSB`  | 4 |
/// | `"MygnSB"`  | `:900` | `fpcNm_Obj_Mtest_e` | `Type_MygnSB`  | 5 |
/// | `"Owater"`  | `:901` | `fpcNm_Obj_Mtest_e` | `Type_Owater`  | 6 |
/// | `"Astop"`   | `:902` | `fpcNm_Obj_Mtest_e` | `Type_Astop`   | 7 |
///
/// All eight are `d_a_obj_mtest.cpp`, and `daObjMtest::Act_c::M_arcname[]` (`d_a_obj_mtest.cpp:15-24`)
/// is `"Mtest"` for types 0-5, `"Owater"` for type 6 and `"Astop"` for type 7 - so the archive names
/// disagree with the stage strings for six of the eight, unlike `lbridge` / `ojtree`. None of the
/// eight is claimed by any other `*_ACTORS` array or `pub const &str` in `src/sim/`; note
/// `movebox.rs` claims `"Ecube"` and `wall.rs` `"Wall"`, neither of which is one of these.
pub const MTEST_ACTORS: [&str; 8] = [
    "Mcube",    // <- d_stage.cpp:895, Type_Mcube   = 0  (d_a_obj_mtest.h:10)
    "Mcyln",    // <- d_stage.cpp:896, Type_Mcyln   = 1  (d_a_obj_mtest.h:11)
    "Mcube10",  // <- d_stage.cpp:897, Type_Mcube10 = 2  (d_a_obj_mtest.h:12)
    "Mcyln10",  // <- d_stage.cpp:898, Type_Mcyln10 = 3  (d_a_obj_mtest.h:13)
    "MwtrSB",   // <- d_stage.cpp:899, Type_MwtrSB  = 4  (d_a_obj_mtest.h:14)
    "MygnSB",   // <- d_stage.cpp:900, Type_MygnSB  = 5  (d_a_obj_mtest.h:15)
    "Owater",   // <- d_stage.cpp:901, Type_Owater  = 6  (d_a_obj_mtest.h:16)
    "Astop",    // <- d_stage.cpp:902, Type_Astop   = 7  (d_a_obj_mtest.h:17)
];

/// `daObjMtest::Act_c::Type` (`d_a_obj_mtest.h:9-19`). Every member is named, and each is cited
/// where it is actually used - the eight `Type_e` values are the `SCALE_MULT` index, so a type with
/// no behavioural arm of its own is named by the [`MTEST_ACTORS`] table's row instead.
///
/// * `Type_Mcube = 0`, `Type_Mcyln = 1`, `Type_Mcube10 = 2`, `Type_Mcyln10 = 3`
///   (`d_a_obj_mtest.h:10-13`) - four plain / scaled cubes and cylinders. Each differs from the
///   others only in [`SCALE_MULT`]; none has a branch of its own.
/// * `Type_MwtrSB = 4` (`:14`, commented `// Water splashes`) and `Type_MygnSB = 5` (`:15`) - the
///   two that share [`CULL_WIDE_MIN`] / [`CULL_WIDE_MAX`] (`:112`).
/// * `Type_Owater = 6` (`:16`) - the one type `Create`'s cull arm EXCLUDES (`:113`).
/// * `Type_Astop = 7` (`:17`) - the one type that sets `fopAcStts_NOCULLEXEC_e` (`:117-119`).
const TYPE_MCUBE: u8 = 0; // <- d_a_obj_mtest.h:10
const TYPE_MWTRSB: u8 = 4; // <- d_a_obj_mtest.h:14
const TYPE_MYGNSB: u8 = 5; // <- d_a_obj_mtest.h:15
const TYPE_OWATER: u8 = 6; // <- d_a_obj_mtest.h:16
const TYPE_ASTOP: u8 = 7; // <- d_a_obj_mtest.h:17
/// `Type_Max` (`d_a_obj_mtest.h:18`), the length of `M_arcname`, `M_scl_mult`, `bmd_data` and
/// every `dzb_data` row; `JUT_ASSERT(327, M_type < Type_Max)` (`d_a_obj_mtest.cpp:245`) checks
/// exactly this, and `prm_get_type`'s three bits cannot exceed it.
const TYPE_MAX: u8 = 8; // <- d_a_obj_mtest.h:18

/// `cXyz M_scl_mult[Type_Max]` (`d_a_obj_mtest.cpp:26-35`) - `scale *= scl_mult` at `:263`.
const SCALE_MULT: [[f32; 3]; 8] = [
    [1.0, 1.0, 1.0],    // Type_Mcube
    [1.0, 0.5, 1.0],    // Type_Mcyln
    [10.0, 10.0, 10.0], // Type_Mcube10
    [10.0, 5.0, 10.0],  // Type_Mcyln10
    [0.2, 0.2, 0.2],   // Type_MwtrSB
    [0.2, 0.2, 0.2],   // Type_MygnSB
    [1.0, 1.0, 1.0],    // Type_Owater
    [40.0, 1.0, 40.0],  // Type_Astop
]; // <- d_a_obj_mtest.cpp:26-35
/// RULE 2: `dStage_actorInit` never writes `fopAcM_prm_class::scale`, so a stage-placed `Mtest`
/// enters at exactly 1.0 and `scale *= scl_mult` leaves precisely `scl_mult` - which is why these
/// multipliers are transcribed rather than folded away. `set_mtx`'s `scaleM(scale)` (`:299`) is
/// model work; `scale`'s behavioural reader is the cylinder sizing at `:127-129`.

/// `M_cyl_src` (`d_a_obj_mtest.cpp:37-65`): `SrcObjTg Type = AT_TYPE_BOMB` (`:44`) with
/// `cCcD_TgSPrm_Set_e | cCcD_TgSPrm_IsOther_e` (`:45`), `Radius 50.0f` (`:61`), `Height 100.0f`
/// (`:62`). `SrcObjAt Type` is 0 (`:41`), so the actor attacks nothing - it is a bomb TARGET.
const CYL_RADIUS: f32 = 50.0; // <- d_a_obj_mtest.cpp:61
const CYL_HEIGHT: f32 = 100.0; // <- d_a_obj_mtest.cpp:62

/// ```
/// if (M_type == Type_MwtrSB || M_type == Type_MygnSB) {
///     fopAcM_setCullSizeBox(this, -3000.0f, -10.0f, -3000.0f, 3000.0f, 10.0f, 3000.0f);
/// } else if (M_type != Type_Owater) {
///     fopAcM_setCullSizeBox(this, -51.0f, -1.0f, -51.0f, 51.0f, 101.0f, 51.0f);
/// }
/// ```
/// (`d_a_obj_mtest.cpp:111-115`). `Type_Owater` gets NO box from either arm, so its
/// `fopAc_CULLBOX_CUSTOM_e` (`:392`) has nothing to resolve to - see [`Mtest::cull`].
const CULL_WIDE_MIN: [f32; 3] = [-3000.0, -10.0, -3000.0]; // <- d_a_obj_mtest.cpp:112
const CULL_WIDE_MAX: [f32; 3] = [3000.0, 10.0, 3000.0]; // <- d_a_obj_mtest.cpp:112
const CULL_MIN: [f32; 3] = [-51.0, -1.0, -51.0]; // <- d_a_obj_mtest.cpp:114
const CULL_MAX: [f32; 3] = [51.0, 101.0, 51.0]; // <- d_a_obj_mtest.cpp:114


/// `prm_get_type() = daObj::PrmAbstract<PRM_TYPE_W, PRM_TYPE_S>(this, 3, 0)`
/// (`d_a_obj_mtest.h:35`, `:22-23`) - `((1 << 3) - 1) & (param >> 0)` (`d_a_obj.h:12-15`) over the
/// WHOLE u32. Three bits against `Type_Max = 8`, so the index is always in range and RULE 10's
/// guard is not needed - the decomp asserts exactly this at `:245`.
const TYPE_WIDTH: u32 = 0x03; // <- d_a_obj_mtest.h:22
const TYPE_SHIFT: u32 = 0x00; // <- d_a_obj_mtest.h:23
/// `prm_get_swSave() = daObj::PrmAbstract<PRM_SWSAVE_W, PRM_SWSAVE_S>(this, 8, 8)`
/// (`d_a_obj_mtest.h:36`, `:25-26`).
const SWSAVE_WIDTH: u32 = 0x08; // <- d_a_obj_mtest.h:25
const SWSAVE_SHIFT: u32 = 0x08; // <- d_a_obj_mtest.h:26
/// `prm_get_arg0() = daObj::PrmAbstract<PRM_ARG0_W, PRM_ARG0_S>(this, 4, 0x10)`
/// (`d_a_obj_mtest.h:39`, `:28-29`) - decides `chk_appear`'s polarity.
const ARG0_WIDTH: u32 = 0x04; // <- d_a_obj_mtest.h:28
const ARG0_SHIFT: u32 = 0x10; // <- d_a_obj_mtest.h:29
/// `prm_get_arg1() = daObj::PrmAbstract<PRM_ARG1_W, PRM_ARG1_S>(this, 4, 0x18)`
/// (`d_a_obj_mtest.h:42`, `:31-32`) - selects the DZB variant and, at zero, whether the bomb
/// cylinder exists at all.
const ARG1_WIDTH: u32 = 0x04; // <- d_a_obj_mtest.h:31
const ARG1_SHIFT: u32 = 0x18; // <- d_a_obj_mtest.h:32

/// The eight test objects the engine debug menus place - `daObjMtest::Act_c`
/// (`d_a_obj_mtest.h:7-68`), the actor behind `Mcube`, `Mcyln`, `Mcube10`, `Mcyln10`, `MwtrSB`,
/// `MygnSB`, `Owater` and `Astop`.
///
/// (a) inbound damage: NONE in the `combat::HitPoints` sense. The actor DOES register a bomb
/// TARGET - `M_cyl_src`'s `SrcObjTg Type` is `AT_TYPE_BOMB` (`:44`) - and a bomb hitting it calls
/// `fopAcM_delete(this)` (`:316`), so it can be destroyed by a blast, but only via that AT channel.
/// `SrcObjAt Type` is 0 (`:41`), so it has no attack collider: it deals no damage. The profile
/// `Group` is `fopAc_ACTOR_e` (`:391`). Neither `combat::HitPoints` nor `enemy::Hurt` is attached:
/// `HitPoints` models sword/bomb damage that this actor has no collider for, so attaching it would
/// let `combat::control` delete an object retail cannot be hurt by. The real destruction - the bomb
/// hit - is modelled directly through `super::bombflower::Blasts`, which IS `AT_TYPE_BOMB`-shaped
/// and therefore the right channel here (unlike `ff.rs`'s `AT_TYPE_WIND`, which has none).
///
/// `Mthd_IsDelete` is `MoveBGIsDelete` (`d_a_obj_mtest.cpp:365-366`), inherited from
/// `dBgS_MoveBgActor`; `Act_c` overrides neither, so the base `MoveBGIsDelete`
/// (`d_bg_s_movebg_actor.h:28`) returning `IsDelete()`, whose base body is the bare `TRUE`
/// (`d_bg_s_movebg_actor.h:35`), is what runs. RULE 6b: "accepts a delete request", never a
/// self-destruct. `Mthd_Create` (`:136-278`) returns `cPhs_ERROR_e` only from `MoveBGCreate`
/// (`:265`), which is heap work, so `spawn` never refuses.
///
/// Dropped as renderer / `dBgW` / heap work: `CreateHeap` (`:81-104`) and its `bmd_data` table
/// (`:82-91`), `Mthd_Create`'s `dzb_data` and `heap_size` tables (`:137-240`) and `MoveBGCreate`
/// (`:265`), `set_mtx` (`:293-301`), `init_mtx` (`:304-309`), `Draw` (`:335-344`), `Delete` (`:281-283`),
/// `Mthd_Delete` (`:286-290`) and the `Regist` / `Release` / `SetLock` background toggles
/// (`:270-273`, `:319`, `:326`).
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Mtest {
    /// `M_type` (`d_a_obj_mtest.h:66`) from `prm_get_type()` (`:244`) - indexes `M_arcname`,
    /// `M_scl_mult`, `bmd_data` and `dzb_data`, and selects the cull box.
    pub kind: u8,
    /// `prm_get_arg0()` (`d_a_obj_mtest.h:39`) - `chk_appear`'s polarity selector (`:71-77`).
    pub arg0: u8,
    /// `prm_get_arg1()` (`d_a_obj_mtest.h:42`) - the DZB variant index (`:250-258`) and the
    /// `== 0` gate on the bomb cylinder (`:121`, `:321`) and on `Draw` (`:336`).
    pub arg1: u8,
    /// `prm_get_swSave()` (`d_a_obj_mtest.h:36`) - the switch `chk_appear` reads (`:69`, `:72`,
    /// `:74`).
    ///
    /// RULE 11b: the REAL index is passed to `Switches::test` unchanged and never down-shifted into
    /// 0..128. Retail's `dSv_info_c::isSwitch` (`d_save.cpp:1586-1603`) routes `0x80..0xC0` to the DAN
    /// bits and `0xC0..` to per-zone bits, neither of which `super::flags::Switches` models, so a
    /// `swSave >= 0x80` Mtest reads false here. `swSave == 0xFF` agrees with retail's explicit
    /// `i_no == 255 -> FALSE`.
    pub switch_no: u8,
    /// `mbAppear` (`d_a_obj_mtest.h:67`, a `bool`) - the PREVIOUS tick's `chk_appear()` result,
    /// seeded in `Mthd_Create` (`:260`) and refreshed at the END of `Execute` (`:330`), so it lags
    /// the tests by one tick.
    pub appear: bool,
    /// `fopAcM_setCullSizeBox` (`d_a_obj_mtest.cpp:112`, `:114`) - see [`Mtest::cull`] for why this
    /// can be absent.
    pub culled: bool,
    /// Whether the actor has been destroyed by a bomb this tick, so `control` can despawn it.
    pub removed: bool,
    /// One-shot guard so `Mthd_Create`'s `mbAppear = chk_appear()` (`:260`) runs exactly once, on
    /// the first tick, where `Switches` is available.
    pub entered: bool,
}

impl Mtest {
    /// `daObjMtest::Act_c::Mthd_Create` (`d_a_obj_mtest.cpp:136-278`) minus the resource load, the
    /// DZB/heap tables, `MoveBGCreate`, `mbAppear`'s initial `chk_appear()` (deferred) and the
    /// `dBgW` lock/release - all of which is asset or RULE 3b work. What remains is the parameter
    /// decode, `scale *= M_scl_mult[M_type]` (`:262-263`) and the background-registration state.
    ///
    /// `scale` is the `fopAc_ac_c::scale` `cXyz` (`f_op_actor.h:214`); RULE 2 gives a stage-placed
    /// actor exactly 1.0, and `dStage_actorInit` never writes it, so `scale *= scl_mult` leaves
    /// precisely [`SCALE_MULT`][kind]. Only `x` and `z` (for the radius) and `y` (for the height)
    /// are read back, at `:127-129`.
    ///
    /// `chk_appear()`'s initial value and `Create`'s cull box both need state the sim resolves on
    /// the first tick, so `spawn` only decodes parameters.
    pub fn spawn(parameters: u32) -> Self {
        let kind = ((parameters >> TYPE_SHIFT) & ((1 << TYPE_WIDTH) - 1)) as u8;
        // RULE 10: `kind` is three bits against `Type_Max = 8`, so it cannot exceed the tables, but
        // the fold is kept explicit because `SCALE_MULT` and `cull` index it and an out-of-range
        // read would be UB in retail too.
        let kind = if kind < TYPE_MAX { kind } else { TYPE_MCUBE };
        let arg0 = ((parameters >> ARG0_SHIFT) & ((1 << ARG0_WIDTH) - 1)) as u8;
        let arg1 = ((parameters >> ARG1_SHIFT) & ((1 << ARG1_WIDTH) - 1)) as u8;
        Self {
            kind,
            arg0,
            arg1,
            switch_no: ((parameters >> SWSAVE_SHIFT) & ((1 << SWSAVE_WIDTH) - 1)) as u8,
            // `mbAppear` (`:67`) has no initialiser before `Mthd_Create`'s `mbAppear = chk_appear()`
            // (`:260`), so it takes `fopAcM_ct`'s `sBs_ClearArea` zero-fill (`f_pc_base.cpp:98`).
            appear: false,
            // `Create`'s two `setCullSizeBox` arms (`:111-115`) - see [`Mtest::cull`].
            culled: kind == TYPE_MWTRSB || kind == TYPE_MYGNSB || kind != TYPE_OWATER,
            removed: false,
            entered: false,
        }
    }

    /// The actor's own cull box, or `None` for `Type_Owater`.
    ///
    /// `fopAc_CULLBOX_CUSTOM_e` (`d_a_obj_mtest.cpp:392`) resolves through `fopAcM_setCullSizeBox`,
    /// and `Create`'s second arm is `else if (M_type != Type_Owater)` (`:113-115`) - so for
    /// `Type_Owater` NEITHER arm runs and the actor registers no box of its own. This file also calls
    /// neither `fopAcM_SetMin` nor `fopAcM_SetMax`, so there is no second route to one.
    ///
    /// Returning `None` for that one type is the honest answer, and it is the same shape
    /// `swtact::Swtact::cull` uses for its box-less variant (`src/sim/swtact.rs:71-77`) and the same
    /// refusal `iceisland` and `tornado` make for their box-less profiles.
    ///
    /// For the record, `Type_Owater` still gets culled - it falls back to whatever the profile's
    /// `fopAc_CULLBOX_CUSTOM_e` path defaults to, i.e. the `l_cullSizeBox[0]` table entry
    /// `fopAc_cullSizeBox(cXyz(-40.0f, 0.0f, -40.0f), cXyz(40.0f, 125.0f, 40.0f))`
    /// (`f_op_actor_mng.cpp:558`, `fopAc_CULLBOX_0_e`). That is a DIFFERENT actor's box, so it is
    /// cited here rather than written into [`Mtest::cull`] - the wiring owner should decline an
    /// `actor_views` branch for `Type_Owater` rather than borrow `fopAc_CULLBOX_0_e`.
    pub fn cull(&self) -> Option<([f32; 3], [f32; 3])> {
        if self.culled {
            Some(if self.kind == TYPE_MWTRSB || self.kind == TYPE_MYGNSB {
                (CULL_WIDE_MIN, CULL_WIDE_MAX)
            } else {
                (CULL_MIN, CULL_MAX)
            })
        } else {
            None
        }
    }

    /// `scale *= M_scl_mult[M_type]` (`d_a_obj_mtest.cpp:262-263`), i.e. the actor's scale after
    /// RULE 2's 1.0.
    pub fn scale(&self) -> [f32; 3] {
        SCALE_MULT[(self.kind & 0x07) as usize]
    }

    /// `daObjMtest::Act_c::chk_appear` (`d_a_obj_mtest.cpp:68-78`):
    /// ```
    /// s32 swSave = prm_get_swSave();
    /// s32 arg0 = prm_get_arg0();
    /// if (arg0 == 0) { return fopAcM_isSwitch(this, swSave) ? true : false; }
    /// else if (arg0 == 1) { return !fopAcM_isSwitch(this, swSave) ? true : false; }
    /// else { return true; }
    /// ```
    /// `arg0 == 0` appears on switch set, `arg0 == 1` on switch clear, anything else always.
    pub fn appear_now(&self, switches: &Switches) -> bool {
        match self.arg0 {
            0 => switches.test(self.switch_no),
            1 => !switches.test(self.switch_no),
            _ => true,
        }
    }

    /// `Mthd_Create`'s `mbAppear = chk_appear();` (`d_a_obj_mtest.cpp:260`), first tick only.
    fn create_init(&mut self, switches: &Switches) {
        self.appear = self.appear_now(switches);
        self.entered = true;
    }

    /// `daObjMtest::Act_c::Execute` (`d_a_obj_mtest.cpp:312-332`), minus `set_mtx`'s consumers,
    /// `*pMtx = &mMtx` (`:329`) and the three `dBgW` calls.
    fn execute(&mut self, actor: &Actor, switches: &Switches, blasts: &super::bombflower::Blasts) {
        // `bool appear = chk_appear();` (:313)
        let appear = self.appear_now(switches);
        // ```
        // if (mCyl.ChkTgHit()) {
        //     fopAcM_delete(this);
        // } else if (appear) {
        //     if (!mbAppear) { dComIfG_Bgsp()->Regist(mpBgW, this); }
        //     if (prm_get_arg1() == 0) { mCyl.MoveCAtTg(current.pos); dComIfG_Ccsp()->Set(&mCyl); }
        // } else if (mbAppear) {
        //     dComIfG_Bgsp()->Release(mpBgW);
        // }
        // *pMtx = &mMtx;
        // mbAppear = appear;
        // ```
        // (:315-330)
        //
        // `mCyl` only exists when `prm_get_arg1() == 0` (`Create`, `:121`), and it is the
        // `AT_TYPE_BOMB` Tg collider from `M_cyl_src` (`:44`), so `ChkTgHit()` is "a bomb blast
        // overlapped this actor". `super::bombflower::Blasts` is the shared `AT_TYPE_BOMB` blast
        // channel (RULE 13 / RULE 3b's AT note), which makes it the correct reader here - unlike
        // `ff.rs`, whose sphere is `AT_TYPE_WIND`.
        //
        // Retail re-centres the cylinder on `current.pos` every tick (`:322`), so the hit test is
        // against the actor's own position; the cylinder's `Radius` / `Height` are
        // [`CYL_RADIUS`] / [`CYL_HEIGHT`] scaled by `scale.x`/`scale.z`/`scale.y` (`:127-129`).
        // `Blasts::at` is an exact-position test, so it is a narrower reading of the same channel -
        // documented in the report rather than widened with an invented radius.
        if self.arg1 == 0 && blasts.at(actor.pos) {
            // `fopAcM_delete(this);` (:316) - a real self-deletion request, unlike the bare-`TRUE`
            // `IsDelete`. `fopAcM_delete` is `fpcM_Delete(pActor)` (`f_op_actor_mng.cpp:134-138`).
            self.removed = true;
        }
        // `Regist` (`:319`) and `Release` (`:326`) are the `dBgW` background toggles - RULE 3b
        // absent, and `mbAppear`'s update below is the state they track, so it is kept.
        // `mbAppear = appear;` (:330) - the value sampled at the TOP of this tick.
        self.appear = appear;
    }

    /// `f32 scaleMag = scale.x < scale.z ? scale.x : scale.z; mCyl.SetR(scaleMag * 50.0f);
    /// mCyl.SetH(scale.y * 100.0f);` (`d_a_obj_mtest.cpp:127-129`), from the post-`scl_mult` scale
    /// ([`Mtest::scale`]).
    ///
    /// This is the only reader of [`Mtest::scale`] in the transcribed half of the actor - `Create`'s
    /// `if (prm_get_arg1() == 0)` arm (`:121-130`) is the only place `scale` is read back, because
    /// `set_mtx`'s `scaleM(scale)` (`:299`) and `init_mtx`'s `setBaseScale(scale)` (`:306`) are model
    /// work. So the scale multiplier carries real gameplay meaning - it sets how big a blast must be
    /// to hit - and [`Mtest::scale`] is kept rather than dropped as renderer-only.
    pub fn bomb_cylinder(&self) -> (f32, f32) {
        let scale = self.scale();
        let magnitude = if scale[0] < scale[2] {
            scale[0]
        } else {
            scale[2]
        };
        (magnitude * CYL_RADIUS, scale[1] * CYL_HEIGHT)
    }

    /// Whether `M_type == Type_Astop`, whose `fopAcM_OnStatus(this,
    /// fopAcStts_NOCULLEXEC_e)` (`d_a_obj_mtest.cpp:117-119`) opts it out of culled execution -
    /// i.e. it never SKIPS its execute/Draw cycle while off-screen. `fopAcM_CheckStatus` has no
    /// counterpart in the sim, so the status bit itself is OMITTED; this is the flag the `actor_views`
    /// wiring should read to pick a box for `Astop`, since `Astop`'s `Scale` multiplier is 40x on x
    /// and z ([`SCALE_MULT`], `d_a_obj_mtest.cpp:34`).
    ///
    /// `Draw` also gates on `prm_get_arg1() != 3 && prm_get_arg1() != 4` (`:336`), i.e. the two arg1
    /// variants that select the `MCUBELK` / `MYYN` DZB pair get no model update at all. That is
    /// renderer, and with `Draw` dropped there is nothing to gate.
    pub fn astop(&self) -> bool {
        self.kind == TYPE_ASTOP
    }
}

pub fn is_mtest(name: &str) -> bool {
    MTEST_ACTORS.contains(&name)
}

pub fn control(
    switches: Res<Switches>,
    blasts: Res<super::bombflower::Blasts>,
    mut objects: Query<(Entity, &Actor, &mut Mtest)>,
    mut commands: Commands,
) {
    for (entity, actor, mut object) in &mut objects {
        if object.removed {
            let _ = commands.entity(entity).despawn();
            continue;
        }
        if !object.entered {
            object.create_init(&switches);
        }
        object.execute(actor, &switches, &blasts);
    }
}