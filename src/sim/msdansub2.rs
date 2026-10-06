use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::events::{SimEvent, SimEvents};
use super::flags::Switches;
use super::vibration::Vibration;
use super::walk;

/// `OBJNAME("MsdanS2", fpcNm_Obj_MsdanSub2_e, 255, 0)` (`d_stage.cpp:956`) - the OBJNAME table is
/// the authority for the stage string. `daObjMsdanSub2::Act_c::M_arcname` is `"Msdan"`
/// (`d_a_obj_msdan_sub2.cpp:10`), which is the archive `OBJNAME("Msdan", fpcNm_Obj_Msdan_e, ...)`
/// (`d_stage.cpp:953`) resolves to a DIFFERENT process - the same trap as `Lwood`/`lwood`.
///
/// `src/sim/msdan2.rs` spawns its ring children under this string (`fopAcM_create(fpcNm_Obj_
/// MsdanSub2_e, ...)` per `d_a_obj_msdan2.cpp:20`) and now names them through this array rather
/// than repeating the literal, so `"MsdanS2"` is declared exactly once in the crate.
pub const MSDANSUB2_ACTORS: [&str; 1] = ["MsdanS2"];

/// `enum Prm_e { PRM_SWSAVE_W = 8, PRM_SWSAVE_S = 0, PRM_OBJNO_W = 8, PRM_OBJNO_S = 8 }`
/// (`d_a_obj_msdan_sub2.h:18-23`), fed through `daObj::PrmAbstract`
/// (`((1 << width) - 1) & (param >> shift)`, `d_a_obj.h:14-17`).
const SWSAVE_WIDTH: u32 = 0x08; // <- d_a_obj_msdan_sub2.h:19 (PRM_SWSAVE_W)
const SWSAVE_SHIFT: u32 = 0x00; // <- d_a_obj_msdan_sub2.h:20 (PRM_SWSAVE_S)
const OBJNO_WIDTH: u32 = 0x08; // <- d_a_obj_msdan_sub2.h:22 (PRM_OBJNO_W)
const OBJNO_SHIFT: u32 = 0x08; // <- d_a_obj_msdan_sub2.h:23 (PRM_OBJNO_S)

/// `fopAcM_setCullSizeBox(this, -1500.0f, -1000.0f, -1500.0f, 1500.0f, 1000.0f, 1500.0f)`
/// (`d_a_obj_msdan_sub2.cpp:24`) - the profile's `fopAc_CULLBOX_CUSTOM_e` (`:221`) resolves to it.
const CULL_MIN: [f32; 3] = [-1500.0, -1000.0, -1500.0]; // <- d_a_obj_msdan_sub2.cpp:24
const CULL_MAX: [f32; 3] = [1500.0, 1000.0, 1500.0]; // <- d_a_obj_msdan_sub2.cpp:24

/// `field_0x2D4 = 0x10` (`d_a_obj_msdan_sub2.cpp:37`) - `Create`'s "the switch is already on, so
/// this step is at the end of its travel" marker. The same value as [`SLOT_LIMIT`], from two
/// separate tokens.
const SLOT_STARTED: i32 = 0x10; // <- d_a_obj_msdan_sub2.cpp:37
/// `if (field_0x2D4 < 16)` (`d_a_obj_msdan_sub2.cpp:112`) - the per-tick gate on the extend loop.
const SLOT_LIMIT: i32 = 16; // <- d_a_obj_msdan_sub2.cpp:112
/// `cM_scos(current.angle.y) * 600.0f` / `cM_ssin(current.angle.y) * 600.0f`
/// (`d_a_obj_msdan_sub2.cpp:30-34`, `:123-140`) - the step's full travel along its rail, and the
/// `if (field_0x2D8 >= 600.0f)` threshold (`:133`).
const TRAVEL: f32 = 600.0; // <- d_a_obj_msdan_sub2.cpp:30
/// `field_0x2DC += 10.0f` (`d_a_obj_msdan_sub2.cpp:117`) - the per-tick acceleration ramp that
/// makes the step ease out rather than jump.
const RAMP_STEP: f32 = 10.0; // <- d_a_obj_msdan_sub2.cpp:117
/// `fopAcM_seStartCurrent(this, JA_SE_OBJ_SW_STAIR2_ON_1, 0)` (`d_a_obj_msdan_sub2.cpp:114`),
/// `JA_SE_OBJ_SW_STAIR2_ON_1 = 0x69BD` (`JAZelAudio_SE.h:1966`).
const SE_STAIR2_ON: u32 = 0x69BD; // <- JAZelAudio_SE.h:1966
/// `dComIfGp_getVibration().StartShock(1, 1, cXyz(0.0f, 1.0f, 0.0f))`
/// (`d_a_obj_msdan_sub2.cpp:143`) - the clunk as a step reaches the end of its rail.
const SHOCK_PATTERN: i32 = 1; // <- d_a_obj_msdan_sub2.cpp:143
const SHOCK_FLAGS: i32 = 1; // <- d_a_obj_msdan_sub2.cpp:143
const SHOCK_COORD: [f32; 3] = [0.0, 1.0, 0.0]; // <- d_a_obj_msdan_sub2.cpp:143

/// One tread of the Storm Temple / Great Sea raised-staircase that `fpcNm_Obj_MsdanSub2_e` drives -
/// `daObjMsdanSub2::Act_c` (`d_a_obj_msdan_sub2.h:8-25`), the actor behind the stage string
/// `MsdanS2`.
///
/// `Msdan2` (`fpcNm_Obj_Msdan2_e`) creates a sixteen-tread ring of these at its own position, one
/// `objNo` per tread (`d_a_obj_msdan2.cpp`), which is why the treads share a switch and differ only
/// in `prm_get_objNo()`. Each tread extends along its rail by 600 units when its switch rises, and
/// the ring assembles a staircase.
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. `daObjMsdanSub2::Act_c` is a
/// `dBgS_MoveBgActor` (`d_a_obj_msdan_sub2.h:8`, `d_bg_s_movebg_actor.h:26`) whose only geometry is
/// `mpBgW` / `mBgMtx` - a `dBgW` moving background, which RULE 3b lists among the absent inputs and
/// which is not a `dCcD` collider. The class registers no `dCcD` collider of any kind and the
/// profile `Group` is `fopAc_ACTOR_e` (`:220`). Neither `combat::HitPoints` nor `enemy::Hurt` is
/// attached; attaching them would let `combat::control` despawn a tread retail cannot damage.
///
/// `Mthd_IsDelete` is `dBgS_MoveBgActor::MoveBGIsDelete` (`:194`), which forwards to the base
/// `IsDelete()` = `return TRUE` (`d_bg_s_movebg_actor.h:35`) - RULE 6b, "accepts a delete request".
/// `Mthd_Create` (`:51-68`) can return `cPhs_ERROR_e` only from `MoveBGCreate` (`:57`), which is
/// heap / `dComIfG_Bgsp()->Regist` work, so `spawn` never refuses.
///
/// RULE 2: `init_mtx`'s `scale *= 1.01` (`:96`) is J3D `setBaseScale` and reaches nothing in this
/// sim, so RULE 2's exact 1.0 stands.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct MsdanSub2 {
    /// `prm_get_swSave()` (`d_a_obj_msdan_sub2.h:11`), the `daObj::PrmAbstract(this, 8, 0)` index
    /// `Create` reads at `:26`, `Mthd_Create` at `:60` and `Execute` re-reads live at `:105`.
    pub switch_no: u8,
    /// `prm_get_objNo()` (`d_a_obj_msdan_sub2.h:10`), `daObj::PrmAbstract(this, 8, 8)` - which tread
    /// of the sixteen this is. `Execute` compares it against `field_0x2D4` (`:121`, `:134`).
    pub obj_no: u8,
    /// `field_0x2D4` (`d_a_obj_msdan_sub2.h:14`) - the distance index along the ring, `0..16`.
    pub slot: i32,
    /// `field_0x2D8` (`d_a_obj_msdan_sub2.h:15`) - how far this tread has travelled from its home,
    /// accumulated from the ramp and reset every step.
    pub dist: f32,
    /// `field_0x2DC` (`d_a_obj_msdan_sub2.h:16`) - the per-tick acceleration ramp; reset to 0 by
    /// `Create`'s else arm (`:40`) and by each completed step (`:147`).
    pub ramp: f32,
    /// `home.pos` (`d_a_obj_msdan_sub2.cpp:30-34`, `:123-140`). Retail reads `fopAcM_GetHomePos_p`,
    /// which for a stage-placed actor is the position `dStage_actorInit` copied into `current.pos`
    /// before `Create` ran - so it is recovered on the first tick, exactly the `htobi3.rs` /
    /// `hbrf1.rs` reduction.
    pub home: Vec3,
    /// One-shot guard so `Create`'s placement happens once, on the first tick, where `Actor` is
    /// mutable. `fopAcM_SetMtx` (`:23`), `init_mtx` (`:44`), `mpBgW->Move()` (`:45`) and
    /// `Mthd_Create` (`:51-68`) are model / `dBgW` / heap work.
    pub entered: bool,
}

impl MsdanSub2 {
    /// `daObjMsdanSub2::Act_c::Create` (`:22-48`), minus `fopAcM_SetMtx` (`:23`), the cull box
    /// (`:24`, carried by [`MsdanSub2::cull`]), `init_mtx` (`:44`), `mpBgW->Move()` (`:45`) and the
    /// placement arm (`:28-42`), which is deferred to the first tick because it needs `Switches`
    /// and a mutable `Actor`.
    ///
    /// `daObjMsdanSub2::Act_c` carries no `fopAcM_prm_class` parameter - the profile's `Parameters`
    /// field is `0` (`:215`) - so there is nothing else to decode.
    pub fn spawn(parameters: u32) -> Self {
        Self {
            switch_no: ((parameters >> SWSAVE_SHIFT) & ((1 << SWSAVE_WIDTH) - 1)) as u8,
            obj_no: ((parameters >> OBJNO_SHIFT) & ((1 << OBJNO_WIDTH) - 1)) as u8,
            slot: 0,
            dist: 0.0,
            ramp: 0.0,
            home: Vec3::ZERO,
            entered: false,
        }
    }

    /// `fopAcM_setCullSizeBox(this, -1500.0f, -1000.0f, -1500.0f, 1500.0f, 1000.0f, 1500.0f)`
    /// (`d_a_obj_msdan_sub2.cpp:24`) - the actor's one real custom cull box.
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `Create`'s placement and its `field_0x2D4` arm (`:28-42`).
    fn create_init(&mut self, actor: &mut Actor, switches: &Switches) {
        self.home = actor.pos;
        // `BOOL isSwitch = fopAcM_isSwitch(this, prm_get_swSave());` (:26) - the room- and
        // zone-agnostic half of `dComIfGs_isSwitch` (`f_op_actor_mng.h:374-376`), which is
        // `Switches::test`. RULE 11b: an index >= 0x80 routes to the per-zone `dSv_zoneBit_c`
        // arrays (`d_save.h:982-984`, `d_save.cpp:1588-1598`), so such a tread reads false here and
        // `set()` on it is a no-op. The real index is carried through unchanged.
        if switches.test(self.switch_no) {
            // `if (!(prm_get_objNo() & 1)) { pos.x = home.x + cM_scos(angle.y) * 600.0f; ... }`
            // else the mirrored pair (`:29-35`).
            let angle_y = actor.angle[1];
            let home = self.home;
            actor.pos = ring_place(home, angle_y, TRAVEL, self.obj_no & 1 == 1, actor.pos);
            self.slot = SLOT_STARTED;
        } else {
            // `field_0x2D4 = 0; field_0x2D8 = 0.0f; field_0x2DC = 0.0f;` (:39-41)
            self.slot = 0;
            self.dist = 0.0;
            self.ramp = 0.0;
        }
        self.entered = true;
    }

    /// `daObjMsdanSub2::Act_c::Execute(Mtx**)` (`:104-156`), minus `set_mtx()` (`:152`) and
    /// `*i_mtx = &M_tmp_mtx` (`:153`), which `MoveBGExecute`
    /// (`d_bg_s_movebg_actor.cpp:86-101`) only copies into `mBgMtx` for `dBgW::Move()`.
    fn execute(
        &mut self,
        actor: &mut Actor,
        switches: &Switches,
        events: &mut SimEvents,
        vibration: &mut Vibration,
    ) {
        // `if (isSwitch)` (:107)
        if !switches.test(self.switch_no) {
            return;
        }
        // `if (field_0x2DC < 0.0f) { field_0x2DC = 0.0f; }` (:108-110)
        if self.ramp < 0.0 {
            self.ramp = 0.0;
        }
        // `if (field_0x2D4 < 16) {` (:112)
        if self.slot >= SLOT_LIMIT {
            return;
        }
        // `if (field_0x2DC == 0.0f) { fopAcM_seStartCurrent(this, JA_SE_OBJ_SW_STAIR2_ON_1, 0); }`
        // (:113-115) - once per step, on the tick the ramp is still zero.
        if self.ramp == 0.0 {
            events.list.push(SimEvent::Se(SE_STAIR2_ON));
        }
        // `field_0x2DC += 10.0f;` (:117)
        self.ramp += RAMP_STEP;
        // `field_0x2D8 += field_0x2DC;` (:118)
        self.dist += self.ramp;
        // `s32 objNo = prm_get_objNo(); if (field_0x2D4 == objNo) { ... }` (:120-131) - only the
        // one tread whose turn it is moves.
        let odd = self.slot & 1 == 1;
        let angle_y = actor.angle[1];
        let home = self.home;
        let dist = self.dist;
        if self.slot == i32::from(self.obj_no) {
            actor.pos = ring_place(home, angle_y, dist, odd, actor.pos);
        }
        // `if (field_0x2D8 >= 600.0f) {` (:133)
        if self.dist >= TRAVEL {
            // `if (field_0x2D4 == prm_get_objNo()) {` - snap to the exact end of the rail
            // (`:134-142`) - `dComIfGp_getVibration().StartShock(1, 1, cXyz(0.0f, 1.0f, 0.0f))`
            // (`:143`).
            if self.slot == i32::from(self.obj_no) {
                actor.pos = ring_place(home, angle_y, TRAVEL, odd, actor.pos);
                vibration.start_shock(SHOCK_PATTERN, SHOCK_FLAGS, Vec3::from_array(SHOCK_COORD));
            }
            // `field_0x2D4 += 1; field_0x2DC = 0.0f; field_0x2D8 = 0.0f;` (:146-148)
            self.slot += 1;
            self.ramp = 0.0;
            self.dist = 0.0;
        }
    }
}

/// `current.pos.x/z = home.pos.x/z +- (cM_scos|cM_ssin)(current.angle.y) * distance`, with the sign
/// taken from the tread's own parity. The three copies are `Create` (`:29-35`, parity from
/// `prm_get_objNo()`), `Execute`'s in-flight placement (`:122-131`, parity from `field_0x2D4`) and
/// `Execute`'s end-of-rail snap (`:135-141`, parity from `field_0x2D4` again). `cM_scos` / `cM_ssin`
/// are `JMASCos` / `JMASSin` on an `s16` angle (`c_math.h:35-41`), so the one conversion is
/// `walk::from_s16` - RULE 5b. Neither decomp arm writes `pos.y`, so `y` passes through.
fn ring_place(home: Vec3, angle_y: i16, distance: f32, odd: bool, pos: Vec3) -> Vec3 {
    let (sin, cos) = walk::from_s16(angle_y).sin_cos();
    let sign = if odd { -1.0 } else { 1.0 };
    Vec3::new(
        home.x + sign * cos * distance,
        pos.y,
        home.z + sign * sin * distance,
    )
}

pub fn is_msdansub2(name: &str) -> bool {
    MSDANSUB2_ACTORS.contains(&name)
}

pub fn control(
    switches: Res<Switches>,
    mut events: ResMut<SimEvents>,
    mut vibration: ResMut<Vibration>,
    mut treads: Query<(&mut Actor, &mut MsdanSub2)>,
) {
    for (mut actor, mut tread) in &mut treads {
        let tread = &mut *tread;
        let actor = &mut *actor;
        if !tread.entered {
            tread.create_init(actor, &switches);
        }
        tread.execute(actor, &switches, &mut events, &mut vibration);
    }
}
