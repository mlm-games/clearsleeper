use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::*;

use super::actor::Actor;
use super::event::{ActiveEvent, EventList, Stage};
use super::events::{SimEvent, SimEvents};
use super::flags::{EventFlags, Switches};

/// `OBJNAME("Hlift", fpcNm_Obj_Hlift_e, 0, 0)` (`d_stage.cpp:869`) and
/// `OBJNAME("Hliftb", fpcNm_Obj_Hlift_e, 1, 0)` (`d_stage.cpp:870`) are the only two
/// `fpcNm_Obj_Hlift_e` rows in the table, and both are `d_a_obj_hlift.cpp`.
///
/// The archive is `Act_c::M_arcname[] = "Hlift"` (`d_a_obj_hlift.cpp:20`), so `Hlift` matches its
/// archive and `Hliftb` does not. The two are distinguished by the `M_data_size[2]` table
/// (`:23-26`), which is indexed by the `prm_get_size` PARAMETER, not by the stage string.
/// `d_stage.cpp`'s third `OBJNAME` field (the "sub" byte) is the value
/// `dStage_actorInit` copies into `fopAcM_prm_class::parameters`, so `Hlift`'s row contributes size
/// bit 0 = 0 and `Hliftb`'s contributes bit 0 = 1 - which is why both stage strings land in the
/// same actor and both reach `prm_get_size()`. Neither string is claimed by any other `*_ACTORS`
/// array or `pub const &str` in `src/sim/`.
pub const HLIFT_ACTORS: [&str; 2] = ["Hlift", "Hliftb"];

/// `fopAcM_setCullSizeBox(this, -151.0f, -1005.0f, -151.0f, 151.0f, 1.0f, 151.0f)`
/// (`d_a_obj_hlift.cpp:55-63`). The profile's `fopAc_CULLBOX_CUSTOM_e` (`:378`) resolves to it.
const CULL_MIN: [f32; 3] = [-151.0, -1005.0, -151.0]; // <- d_a_obj_hlift.cpp:55-63
const CULL_MAX: [f32; 3] = [151.0, 1.0, 151.0]; // <- d_a_obj_hlift.cpp:55-63

/// `daObjHlift::Act_c::M_up_dist[] = { 125, 250, 375, 500, 625, 750, 875, 1000 }`
/// (`d_a_obj_hlift.cpp:22`), indexed by `field_0x2D8` (`prm_get_dist()`).
const UP_DIST: [f32; 8] = [125.0, 250.0, 375.0, 500.0, 625.0, 750.0, 875.0, 1000.0]; // <- d_a_obj_hlift.cpp:22
/// `daObjHlift::Act_c::M_evname[] = "Hlift_up"` (`d_a_obj_hlift.cpp:21`), passed to
/// `dComIfGp_evmng_getEventIdx(M_evname, prm_get_evId())` (`:52`).
const EV_NAME: &str = "Hlift_up"; // <- d_a_obj_hlift.cpp:21

/// The five `L_attr` fields (`d_a_obj_hlift.cpp:14`) that `mode_l_u` and `mode_u_l` read. The
/// initializer is `static const Attr_c L_attr = { 30.0f, 50.0f, 10.0f, 2.0f, 0.3f, 1.2f, 250.0f, 10,
/// 0x4000, 2.5f, 0x00, 0x02, 0x0a, 0x08, 0x07, 0x00, 0x00, 0x00 }`, laid out by
/// `daObjHlift::Attr_c` (`d_a_obj_hlift.h:7-19`).
/// * `field_0x00 = 30.0f` - `set_mtx`'s `y2 = y1 + L_attr.field_0x00` (`:227`), a model offset;
///   its only consumer is renderer.
/// * `field_0x04 = 50.0f` - the near-end threshold (`:134`, `:183`).
/// * `field_0x08 = 10.0f` - the cruise speed (`:135`, `:138`, `:187`).
/// * `field_0x0C = 2.0f` - the decel target (`:140`, `:184`).
/// * `field_0x10 = 0.3f` - the near-end chase step (`:135`, `:189`).
/// * `field_0x14 = 1.2f` - the far-end chase step (`:140`, `:184`).
/// * `field_0x18 = 250.0f` - `rot_set`'s divisor (`:246`); its only consumer is renderer.
/// * `field_0x1C = 10` (an `s16`) and `field_0x1E = 0x4000` (an `s16`) - `vib_set` / `vib_proc`
///   (`:252-265`), whose only consumer is `set_mtx`'s `field_0x2EC` offset; renderer.
/// * `field_0x20 = 2.5f` - `vib_proc`'s amplitude scale (`:261`); renderer.
/// * `field_0x24[]` - the `L_time_lag_num`-indexed start DELAY (`:123`, `:171`).
const NEAR_THRESHOLD: f32 = 50.0; // <- d_a_obj_hlift.cpp:14 (L_attr.field_0x04)
const LIFT_SPEED: f32 = 10.0; // <- d_a_obj_hlift.cpp:14 (L_attr.field_0x08)
const DECEL_SPEED: f32 = 2.0; // <- d_a_obj_hlift.cpp:14 (L_attr.field_0x0C)
const NEAR_STEP: f32 = 0.3; // <- d_a_obj_hlift.cpp:14 (L_attr.field_0x10)
const FAR_STEP: f32 = 1.2; // <- d_a_obj_hlift.cpp:14 (L_attr.field_0x14)
/// `L_attr.field_0x24[L_time_lag_num]` (`d_a_obj_hlift.cpp:123`, `:171`). The struct declares
/// `u8 field_0x24[0x2C - 0x24]` (`d_a_obj_hlift.h:18`) - eight bytes - but `L_time_lag_num %= 5`
/// (`:125`, `:173`) only ever indexes the first five, which are `{ 0x00, 0x02, 0x0a, 0x08, 0x07 }`.
const START_DELAY: [i16; 5] = [0x00, 0x02, 0x0a, 0x08, 0x07]; // <- d_a_obj_hlift.cpp:14 (L_attr.field_0x24)
const START_DELAY_COUNT: i32 = 5; // <- d_a_obj_hlift.cpp:125 (L_time_lag_num %= 5)

/// `fopAcM_seStart(this, JA_SE_OBJ_PRT_LIFT_UD, 0)` (`d_a_obj_hlift.cpp:122`, `:170`) -
/// `JA_SE_OBJ_PRT_LIFT_UD = 0x6949` (`JAZelAudio_SE.h:1858`), the start-of-travel sound.
const SE_LIFT_UD: u32 = 0x6949; // <- JAZelAudio_SE.h:1858
/// `fopAcM_seStart(this, JA_SE_OBJ_PRT_LIFT_STOP, 0)` (`d_a_obj_hlift.cpp:146`, `:195`) -
/// `JA_SE_OBJ_PRT_LIFT_STOP = 0x694A` (`JAZelAudio_SE.h:1859`), the arrival sound.
const SE_LIFT_STOP: u32 = 0x694A; // <- JAZelAudio_SE.h:1859

/// `daObjHlift::Act_c::Mode_e` (`d_a_obj_hlift.h:29-33`) - all three named members.
const MODE_L_U: u32 = 0x01; // <- d_a_obj_hlift.h:30
const MODE_U_L: u32 = 0x03; // <- d_a_obj_hlift.h:31
const MODE_UNK: u32 = 0x05; // <- d_a_obj_hlift.h:32

/// `static procFunc mode_proc[] = { &mode_lower, &mode_l_u, &mode_upper, &mode_u_l,
/// &mode_demoreq }` (`d_a_obj_hlift.cpp:292-298`), indexed by `field_0x2E0`.
const SLOT_LOWER: u8 = 0; // <- d_a_obj_hlift.cpp:293 (mode_lower)
const SLOT_L_U: u8 = 1; // <- d_a_obj_hlift.cpp:294 (mode_l_u)
const SLOT_UPPER: u8 = 2; // <- d_a_obj_hlift.cpp:295 (mode_upper)
const SLOT_U_L: u8 = 3; // <- d_a_obj_hlift.cpp:296 (mode_u_l)
const SLOT_DEMOREQ: u8 = 4; // <- d_a_obj_hlift.cpp:297 (mode_demoreq)

/// `prm_get_dist() = daObj::PrmAbstract<PRM_DIST_W, PRM_DIST_S>(this, 3, 0)`
/// (`d_a_obj_hlift.h:48`, `:36-37`) - `((1 << width) - 1) & (param >> shift)` (`d_a_obj.h:12-15`)
/// over the WHOLE u32 (`f_op_actor_mng.h:127-129`). Three bits against `M_up_dist`'s eight entries,
/// so the index cannot escape the table and RULE 10's guard is not needed.
const DIST_WIDTH: u32 = 0x03; // <- d_a_obj_hlift.h:36
const DIST_SHIFT: u32 = 0x00; // <- d_a_obj_hlift.h:37
/// `prm_get_size() = daObj::PrmAbstract<PRM_SIZE_W, PRM_SIZE_S>(this, 1, 4)`
/// (`d_a_obj_hlift.h:50`, `:42-43`). One bit against `M_data_size`'s two entries, so `Hlift` (0)
/// and `Hliftb` (1) both land in range.
const SIZE_WIDTH: u32 = 0x01; // <- d_a_obj_hlift.h:42
const SIZE_SHIFT: u32 = 0x04; // <- d_a_obj_hlift.h:43
/// `prm_get_evId() = daObj::PrmAbstract<PRM_EVENTID_W, PRM_EVENTID_S>(this, 8, 0x10)`
/// (`d_a_obj_hlift.h:49`, `:39-40`) - the second argument of `getEventIdx` at `:52`.
const EVENT_WIDTH: u32 = 0x08; // <- d_a_obj_hlift.h:39
const EVENT_SHIFT: u32 = 0x10; // <- d_a_obj_hlift.h:40
/// `prm_get_swSave() = daObj::PrmAbstract<PRM_SWSAVE_W, PRM_SWSAVE_S>(this, 8, 8)`
/// (`d_a_obj_hlift.h:51`, `:45-46`) - the switch index fed to `fopAcM_isSwitch`.
const SWSAVE_WIDTH: u32 = 0x08; // <- d_a_obj_hlift.h:45
const SWSAVE_SHIFT: u32 = 0x08; // <- d_a_obj_hlift.h:46

/// The Pirate-ship rope-minigame wooden platforms - `daObjHlift::Act_c`
/// (`d_a_obj_hlift.h:21-103`), the actor behind the stage strings `Hlift` and `Hliftb`.
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. `daObjHlift::Act_c` is a
/// `dBgS_MoveBgActor` (`d_a_obj_hlift.h:21`) holding `mPhase`, two `J3DModel*`, two `int`s, a
/// `Mode_e`, four small scalars and an `s16 mEventId` (`:87-102`) - it registers NO `dCcD`
/// collider of any kind: no `dCcD_Stts`, no `dCcD_Sph`/`Cyl`, no `dCcD_Src*` literal anywhere in
/// `d_a_obj_hlift.cpp`. The `dBgW` it builds from `DZB_HLIFT` / `DZB_HLIFTB` (`:24-25`, `:77-82`) is
/// a moving background, not a damage path, and is RULE 3b absent besides. The profile `Group` is
/// `fopAc_ACTOR_e` (`:377`). Neither `combat::HitPoints` nor `enemy::Hurt` is attached - attaching
/// them would let `combat::control` despawn a platform retail cannot damage.
///
/// `Mthd_IsDelete` is `MoveBGIsDelete` (`d_a_obj_hlift.cpp:351-352`), inherited from
/// `dBgS_MoveBgActor`; `Act_c` overrides neither `IsDelete` nor `MoveBGIsDelete`, so the base
/// `MoveBGIsDelete` (`d_bg_s_movebg_actor.h:28`) returning `IsDelete()`, whose base body is the bare
/// `TRUE` (`d_bg_s_movebg_actor.h:35`), is what runs. RULE 6b: "accepts a delete request", never a
/// self-destruct. `Mthd_Create` (`:71-86`) returns `cPhs_ERROR_e` only from `MoveBGCreate` (`:77`),
/// which is heap work, so `spawn` never refuses.
///
/// Dropped as renderer / `dBgW` work: `set_mtx` (`:224-234`), `init_mtx` (`:237-241`), `rot_set`
/// (`:244-248`, sole writer of `shape_angle`), `vib_set` / `vib_proc` (`:251-270`, whose only
/// consumer is `set_mtx`'s `field_0x2EC` at `:226`), `Draw` (`:316-325`) and the `cullMtx` /
/// `M_tmp_mtx` writes (`:53`, `:231`).
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Hlift {
    /// `field_0x2D8` (`d_a_obj_hlift.h:90`) from `prm_get_dist()` (`:75`) - indexes [`UP_DIST`].
    pub dist: u8,
    /// `field_0x2DC` (`d_a_obj_hlift.h:91`) from `prm_get_size()` (`:76`) - indexes
    /// `M_data_size[2]` (`:32`). It selects the model and DZB archive and has no behavioural
    /// reader, but carrying it keeps `Hlift` and `Hliftb` distinguishable, as retail's archive
    /// choice does.
    pub size: u8,
    /// `field_0x2E0` (`d_a_obj_hlift.h:92`) - the `mode_proc[]` dispatch slot (`:301`).
    pub slot: u8,
    /// `field_0x2E4` (`d_a_obj_hlift.h:93`, a `Mode_e`) - [`MODE_L_U`], [`MODE_U_L`] or
    /// [`MODE_UNK`]; `mode_demoreq` dispatches on it (`:216`) to pick the travel direction.
    pub mode: u32,
    /// `field_0x2F4` (`d_a_obj_hlift.h:98`, an `s16`) - the [`START_DELAY`] countdown armed by
    /// `mode_l_u_init` (`:123`) / `mode_u_l_init` (`:171`) and drained by `mode_l_u` (`:130-131`) /
    /// `mode_u_l` (`:179-180`).
    pub start_delay: i16,
    /// `field_0x2F0` (`d_a_obj_hlift.h:97`) - the lift speed, chased by `cLib_chaseF` in `mode_l_u`
    /// (`:135`, `:140`) and `mode_u_l` (`:184`, `:189`) and snapped by `:138` / `:187`.
    pub speed: f32,
    /// `field_0x2F6` (`d_a_obj_hlift.h:99`, a `u8`) - "the travel event has started"; set by
    /// `mode_demoreq` (`:214`) and cleared by `chk_demo_end` (`:276`).
    pub demo_running: bool,
    /// `L_time_lag_num` (`d_a_obj_hlift.cpp:13`, the file-scope `int`) - the rotating index into
    /// [`START_DELAY`]. Retail keeps ONE static shared by every `fpcNm_Obj_Hlift_e` actor, so
    /// successive lift STARTS take successive slots no matter which actor started them; modelled
    /// per-actor here, which is a documented deviation (see the report).
    pub lag_index: i32,
    /// `mEventId` (`d_a_obj_hlift.h:101`, an `s16`) from
    /// `dComIfGp_evmng_getEventIdx(M_evname, prm_get_evId())` (`:52`); `None` is the -1 that a
    /// name missing from the stage list also produces.
    pub event_slot: Option<usize>,
    /// `home.pos.y` - the parked height. `current.pos.y` starts at it (`:105`, `:154-`) and
    /// `home.pos.y + M_up_dist[...]` (`:154`) is the raised target; every `cLib_chaseF` targets one
    /// or the other (`:143`, `:192`). The sim's `Actor` has no separate `home` slot, so it is
    /// recorded once in `create_init`.
    pub home_y: f32,
    /// `prm_get_evId()` (`d_a_obj_hlift.h:49`) - the second argument to `getEventIdx` at `:52` and
    /// to `fopAcM_orderOtherEventId` at `:206`. Stored because `getEventIdx`'s second argument
    /// redirects through the stage event-info table, which is RULE 3b absent; the real index is
    /// kept so the pure name lookup is recognisable as a reduction.
    pub ev_id: u8,
    /// `prm_get_swSave()` (`d_a_obj_hlift.h:51`) as the index passed to `fopAcM_isSwitch` by
    /// `Create` (`:44`), `mode_lower` (`:111`) and `mode_upper` (`:160`).
    ///
    /// `fopAcM_isSwitch(this, sw)` is `dComIfGs_isSwitch(sw, fopAcM_GetHomeRoomNo(pActor))`
    /// (`f_op_actor_mng.h:374-376`), and `dComIfGs_isSwitch` is
    /// `g_dComIfG_gameInfo.save.isSwitch(i_no, i_roomNo)` (`d_com_inf_game.h:1817-1819`).
    /// `dSv_info_c::isSwitch` (`d_save.cpp:1586-1603`) returns FALSE for `i_no == -1 || i_no == 255`,
    /// routes `0..0x80` to the memory bits, `0x80..0xC0` to the DAN bits and `0xC0..` to the
    /// per-zone bits via `dComIfGp_roomControl_getZoneNo(i_roomNo)`. RULE 11b: `super::flags::Switches`
    /// models only the 0..128 array, so the REAL index is passed through unchanged and NEVER
    /// down-shifted into range; a `swSave >= 0x80` platform reads false here. That is the save
    /// model's zone-bit gap, not a defect in this actor - and `swSave == 0xFF` happens to agree with
    /// retail's explicit `i_no == 255 -> FALSE`.
    pub switch_no: u8,
    /// One-shot guard so `Create`'s `getEventIdx` lookup (`:52`) and its switch test (`:44-49`) run
    /// exactly once, on the first tick, where `Switches` and `EventList` are available.
    pub entered: bool,
}

impl Hlift {
    /// `daObjHlift::Act_c::Create` (`d_a_obj_hlift.cpp:42-68`) minus `field_0x2EC = 0.0f` (`:43`,
    /// renderer), the two mode inits (`:45-49`, deferred), `cullMtx` (`:53`), `init_mtx` (`:54`), the
    /// cull box (`:55-63`) and the `M_control_id` latch (`:64-66`), both statics.
    ///
    /// The `fopAcM_isSwitch`-driven choice between `mode_upper_init` and `mode_lower_init` sets
    /// `current.pos.y`, so it is re-made in [`Hlift::create_init`] - the `shutter2` / `lbridge`
    /// shape, since `spawn` cannot see `Switches`.
    ///
    /// RULE 2: `dStage_actorInit` never writes `fopAcM_prm_class::scale`; `mModel1->setBaseScale(
    /// scale)` (`:238`) is renderer work.
    pub fn spawn(parameters: u32) -> Self {
        Self {
            dist: ((parameters >> DIST_SHIFT) & ((1 << DIST_WIDTH) - 1)) as u8,
            size: ((parameters >> SIZE_SHIFT) & ((1 << SIZE_WIDTH) - 1)) as u8,
            // `field_0x2E0` has no initialiser before `Create`'s switch test, so it takes
            // `fopAcM_ct`'s `sBs_ClearArea` zero-fill (`f_pc_base.cpp:98`) - which happens to be
            // `mode_proc[0]`, `mode_lower`. `create_init` overwrites it with the real choice.
            slot: SLOT_LOWER,
            // `field_0x2E4 = Mode_UNK;` (:50)
            mode: MODE_UNK,
            // `field_0x2F4` (`:98`) likewise takes the zero-fill, and `mode_l_u`'s
            // `if (field_0x2F4 > 0)` (`:130`) keeps a zero at zero until an init arms it.
            start_delay: 0,
            // `field_0x2F0` (`:97`) zero-fill; both inits set it to 0.0f explicitly (`:120`, `:169`).
            speed: 0.0,
            // `field_0x2F6 = 0;` (:51)
            demo_running: false,
            // `L_time_lag_num` (`:13`) is a zero-initialised file-scope `int`, so it starts at 0 -
            // the `START_DELAY[0] = 0x00` entry, i.e. the first lift to start has no delay.
            lag_index: 0,
            event_slot: None,
            // Recorded in `create_init`, which is where the actor's own spawn height is available.
            home_y: 0.0,
            ev_id: ((parameters >> EVENT_SHIFT) & ((1 << EVENT_WIDTH) - 1)) as u8,
            switch_no: ((parameters >> SWSAVE_SHIFT) & ((1 << SWSAVE_WIDTH) - 1)) as u8,
            entered: false,
        }
    }

    /// `fopAcM_setCullSizeBox(this, -151.0f, -1005.0f, -151.0f, 151.0f, 1.0f, 151.0f)`
    /// (`d_a_obj_hlift.cpp:55-63`) - the actor's one real custom cull box, matching the profile's
    /// `fopAc_CULLBOX_CUSTOM_e` (`:378`).
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `M_up_dist[field_0x2D8]` - the raised height `mode_upper_init` parks at (`:154`) and
    /// `mode_l_u` chases (`:143`).
    fn up_dist(&self) -> f32 {
        UP_DIST[(self.dist & 0x07) as usize]
    }

    /// `daObjHlift::Act_c::Create` (`:42-68`), first tick only.
    ///
    /// `home.pos.y` is the actor's spawn `y`: `current.pos.y` and `home.pos.y` differ only by the
    /// travel itself (`mode_lower_init` sets one from the other at `:105`, `mode_upper_init` at
    /// `:154`), so recording the spawn height once is exact.
    fn create_init(&mut self, actor: &mut Actor, switches: &Switches, list: &EventList) {
        self.home_y = actor.pos.y;
        // `mEventId = dComIfGp_evmng_getEventIdx(M_evname, prm_get_evId());` (:52)
        self.event_slot = list.event_index(EV_NAME);
        // ```
        // if (fopAcM_isSwitch(const_cast<Act_c*>(this), switchIndex)) { mode_upper_init(); }
        // else { mode_lower_init(); }
        // ```
        // (:44-49)
        self.slot = if switches.test(self.switch_no) {
            self.upper_init(actor)
        } else {
            self.lower_init(actor)
        };
        self.entered = true;
    }

    /// `daObjHlift::Act_c::mode_upper_init` (`d_a_obj_hlift.cpp:153-156`).
    fn upper_init(&mut self, actor: &mut Actor) -> u8 {
        // `current.pos.y = home.pos.y + M_up_dist[field_0x2D8];` (:154)
        actor.pos.y = self.home_y + self.up_dist();
        // `field_0x2E0 = 2;` (:155)
        SLOT_UPPER
    }

    /// `daObjHlift::Act_c::mode_lower_init` (`d_a_obj_hlift.cpp:104-107`).
    fn lower_init(&mut self, actor: &mut Actor) -> u8 {
        // `current.pos.y = home.pos.y;` (:105)
        actor.pos.y = self.home_y;
        // `field_0x2E0 = 0;` (:106)
        SLOT_LOWER
    }

    /// `daObjHlift::Act_c::Execute` (`d_a_obj_hlift.cpp:290-313`), with the renderer and the
    /// `M_control_id` motor gate dropped - see the struct docs and `execute`'s tail comment.
    fn execute(
        &mut self,
        actor: &mut Actor,
        switches: &Switches,
        list: &EventList,
        active: &mut ActiveEvent,
        ev_flags: &mut EventFlags,
        events: &mut SimEvents,
        entity: Entity,
    ) {
        self.chk_demo_end(active);
        match self.slot {
            SLOT_LOWER => self.mode_lower(switches, list, active, ev_flags, events, entity),
            SLOT_L_U => self.mode_l_u(actor, events),
            SLOT_UPPER => self.mode_upper(switches, list, active, ev_flags, events, entity),
            SLOT_U_L => self.mode_u_l(actor, events),
            _ => self.mode_demoreq(events, active),
        }
        // ```
        // if (field_0x2E0 == 1 || field_0x2E0 == 3 || field_0x2E0 == 4) { M_lift_move_flag = 1; }
        // ```
        // (:303-305) - DROPPED, with `se_whole` (`:282-287`) and `JA_SE_OBJ_PRT_LIFT_WORK`
        // (`JAZelAudio_SE.h:618`). `M_lift_move_flag` (`d_a_obj_hlift.cpp:18`) and `M_control_id`
        // (`:19`) are file-scope statics; `se_whole` plays the motor loop only from the ONE actor
        // whose `mBsPcId` won the `M_control_id` latch at `:64-66`, and only after clearing the
        // flag. Reproducing that needs a shared-static resource the sim has none of, so rather than
        // play the loop once per lift - which would be audibly wrong with two platforms - the
        // motor sound is omitted entirely. The four per-lift sounds at `:122`, `:146`, `:170` and
        // `:195` go through `fopAcM_seStart`, need no static, and ARE transcribed.
    }

    /// `daObjHlift::Act_c::mode_lower` (`d_a_obj_hlift.cpp:110-115`).
    fn mode_lower(
        &mut self,
        switches: &Switches,
        list: &EventList,
        active: &mut ActiveEvent,
        ev_flags: &mut EventFlags,
        events: &mut SimEvents,
        entity: Entity,
    ) {
        // ```
        // int switchIndex = prm_get_swSave();
        // if (fopAcM_isSwitch(const_cast<Act_c*>(this), switchIndex)) {
        //     mode_demoreq_init(Mode_L_U);
        // }
        // ```
        // (:111-114)
        if switches.test(self.switch_no) {
            self.mode_demoreq_init(MODE_L_U, list, active, ev_flags, events, entity);
        }
    }

    /// `daObjHlift::Act_c::mode_upper` (`d_a_obj_hlift.cpp:159-164`).
    fn mode_upper(
        &mut self,
        switches: &Switches,
        list: &EventList,
        active: &mut ActiveEvent,
        ev_flags: &mut EventFlags,
        events: &mut SimEvents,
        entity: Entity,
    ) {
        // ```
        // int switchIndex = prm_get_swSave();
        // if (!fopAcM_isSwitch(const_cast<Act_c*>(this), switchIndex)) {
        //     mode_demoreq_init(Mode_U_L);
        // }
        // ```
        // (:160-163)
        if !switches.test(self.switch_no) {
            self.mode_demoreq_init(MODE_U_L, list, active, ev_flags, events, entity);
        }
    }

    /// `daObjHlift::Act_c::mode_l_u_init` (`d_a_obj_hlift.cpp:118-126`). `vib_set()` (`:119`) is
    /// renderer.
    fn mode_l_u_init(&mut self, events: &mut SimEvents) {
        // `field_0x2F0 = 0.0f;` (:120)
        self.speed = 0.0;
        // `field_0x2E0 = 1;` (:121)
        self.slot = SLOT_L_U;
        // `fopAcM_seStart(this, JA_SE_OBJ_PRT_LIFT_UD, 0);` (:122)
        events.list.push(SimEvent::Se(SE_LIFT_UD));
        // ```
        // field_0x2F4 = L_attr.field_0x24[L_time_lag_num];
        // L_time_lag_num++;
        // L_time_lag_num %= 5;
        // ```
        // (:123-125) - the index is read BEFORE the increment.
        self.start_delay = START_DELAY[self.lag_index as usize % START_DELAY_COUNT as usize];
        self.lag_index = (self.lag_index + 1) % START_DELAY_COUNT;
    }

    /// `daObjHlift::Act_c::mode_u_l_init` (`d_a_obj_hlift.cpp:167-175`). `vib_set()` (`:168`) is
    /// renderer. The only difference from [`Hlift::mode_l_u_init`] is that `field_0x2E0 = 3` is
    /// written LAST (`:174`) instead of second.
    fn mode_u_l_init(&mut self, events: &mut SimEvents) {
        // `field_0x2F0 = 0.0f;` (:169)
        self.speed = 0.0;
        // `fopAcM_seStart(this, JA_SE_OBJ_PRT_LIFT_UD, 0);` (:170)
        events.list.push(SimEvent::Se(SE_LIFT_UD));
        // `field_0x2F4 = L_attr.field_0x24[L_time_lag_num]; L_time_lag_num++; %= 5;` (:171-173)
        self.start_delay = START_DELAY[self.lag_index as usize % START_DELAY_COUNT as usize];
        self.lag_index = (self.lag_index + 1) % START_DELAY_COUNT;
        // `field_0x2E0 = 3;` (:174)
        self.slot = SLOT_U_L;
    }

    /// `daObjHlift::Act_c::mode_l_u` (`d_a_obj_hlift.cpp:129-150`) - lower to upper travel.
    fn mode_l_u(&mut self, actor: &mut Actor, events: &mut SimEvents) {
        // ```
        // if (field_0x2F4 > 0) {
        //     field_0x2F4--;
        // } else {
        //     ... chase the top ...
        // }
        // ```
        // (:130-149) - the `else` arm is the whole travel, so returning early during the delay is
        /// exactly the decomp's structure.
        if self.start_delay > 0 {
            self.start_delay -= 1;
            return;
        }
        // ```
        // f32 fVar1 = current.pos.y - home.pos.y;
        // if (fVar1 < L_attr.field_0x04) {
        //     cLib_chaseF(&field_0x2F0, L_attr.field_0x08, L_attr.field_0x10);
        // } else {
        //     if (fVar1 < M_up_dist[field_0x2D8] - L_attr.field_0x04) {
        //         field_0x2F0 = L_attr.field_0x08;
        //     } else {
        //         cLib_chaseF(&field_0x2F0, L_attr.field_0x0C, L_attr.field_0x14);
        //     }
        // }
        // ```
        // (:133-142)
        let travelled = actor.pos.y - self.home_y;
        if travelled < NEAR_THRESHOLD {
            super::walk::chase_f(&mut self.speed, LIFT_SPEED, NEAR_STEP);
        } else if travelled < self.up_dist() - NEAR_THRESHOLD {
            // `field_0x2F0 = L_attr.field_0x08;` (:138) - a DIRECT assignment in the middle band,
            // not a chase, so the speed snaps to the cruise value there.
            self.speed = LIFT_SPEED;
        } else {
            super::walk::chase_f(&mut self.speed, DECEL_SPEED, FAR_STEP);
        }
        // ```
        // int iVar2 = cLib_chaseF(&current.pos.y, home.pos.y + M_up_dist[field_0x2D8], field_0x2F0);
        // if (iVar2 != 0) {
        //     vib_set();
        //     fopAcM_seStart(this, JA_SE_OBJ_PRT_LIFT_STOP, 0);
        //     mode_upper_init();
        // }
        // ```
        // (:143-148) - `vib_set()` is renderer; the sound and the re-init are not.
        if super::walk::chase_f(&mut actor.pos.y, self.home_y + self.up_dist(), self.speed) {
            events.list.push(SimEvent::Se(SE_LIFT_STOP));
            self.upper_init(actor);
        }
    }

    /// `daObjHlift::Act_c::mode_u_l` (`d_a_obj_hlift.cpp:178-199`) - upper to lower travel. The near
    /// and far arms SWAP their chase targets relative to [`Hlift::mode_l_u`], and the final target
    /// is `home.pos.y` instead of the raised height; those two differences are the whole function.
    fn mode_u_l(&mut self, actor: &mut Actor, events: &mut SimEvents) {
        // `if (field_0x2F4 > 0) { field_0x2F4--; } else { ... chase the bottom ... }` (:179-198)
        if self.start_delay > 0 {
            self.start_delay -= 1;
            return;
        }
        // ```
        // f32 fVar1 = current.pos.y - home.pos.y;
        // if (fVar1 < L_attr.field_0x04) {
        //     cLib_chaseF(&field_0x2F0, L_attr.field_0x0C, L_attr.field_0x14);
        // } else {
        //     if (fVar1 < M_up_dist[field_0x2D8] - L_attr.field_0x04) {
        //         field_0x2F0 = L_attr.field_0x08;
        //     } else {
        //         cLib_chaseF(&field_0x2F0, L_attr.field_0x08, L_attr.field_0x10);
        //     }
        // }
        // ```
        // (:182-191)
        let travelled = actor.pos.y - self.home_y;
        if travelled < NEAR_THRESHOLD {
            super::walk::chase_f(&mut self.speed, DECEL_SPEED, FAR_STEP);
        } else if travelled < self.up_dist() - NEAR_THRESHOLD {
            self.speed = LIFT_SPEED;
        } else {
            super::walk::chase_f(&mut self.speed, LIFT_SPEED, NEAR_STEP);
        }
        // ```
        // int iVar2 = cLib_chaseF(&current.pos.y, home.pos.y, field_0x2F0);
        // if (iVar2 != 0) {
        //     vib_set();
        //     fopAcM_seStart(this, JA_SE_OBJ_PRT_LIFT_STOP, 0);
        //     mode_lower_init();
        // }
        // ```
        // (:192-197)
        if super::walk::chase_f(&mut actor.pos.y, self.home_y, self.speed) {
            events.list.push(SimEvent::Se(SE_LIFT_STOP));
            self.lower_init(actor);
        }
    }

    /// `daObjHlift::Act_c::mode_demoreq_init` (`d_a_obj_hlift.cpp:202-209`).
    fn mode_demoreq_init(
        &mut self,
        next: u32,
        list: &EventList,
        active: &mut ActiveEvent,
        ev_flags: &mut EventFlags,
        events: &mut SimEvents,
        entity: Entity,
    ) {
        // `field_0x2E4 = next;` (:204)
        self.mode = next;
        // `field_0x2F6 = 0;` (:205)
        self.demo_running = false;
        // ```
        // fopAcM_orderOtherEventId(this, mEventId, prm_get_evId());
        // eventInfo.onCondition(dEvtCnd_UNK2_e);
        // ```
        // (:206-207). The `dEvtCnd_UNK2_e` condition is dropped for the reason `htobi3.rs` gives:
        // nothing in the decomp ever calls `chkCondition` with `dEvtCnd_UNK2_e`, and `beforeProc`
        // clears `mCondition` on every `fopAc_Execute` (`f_op_actor.cpp:176`). `ev_flags` and
        // `events` are therefore threaded through unused, matching `lbridge`'s handling.
        let _ = (ev_flags, events);
        self.order(list, active, entity);
        // `field_0x2E0 = 4;` (:208)
        self.slot = SLOT_DEMOREQ;
    }

    /// `fopAcM_orderOtherEventId(this, mEventId, prm_get_evId())` (`d_a_obj_hlift.cpp:206`),
    /// guarded exactly as `hbrf1::Hbrf1::order` and `lbridge::Lbridge::order` guard it -
    /// re-ordering an event already in ORDER / PLAY / CLOSE rewrites its state
    /// (`d_event_manager.cpp:367`).
    fn order(&mut self, list: &EventList, active: &mut ActiveEvent, entity: Entity) {
        let Some(slot) = self.event_slot else {
            return;
        };
        if active.stage_of(slot).is_none() {
            let _ = active.order(list, slot, entity);
        }
    }

    /// `daObjHlift::Act_c::mode_demoreq` (`d_a_obj_hlift.cpp:212-221`).
    ///
    /// `dComIfGp_evmng_existence(mEventId)` (`d_a_obj_hlift.cpp:213`) is
    /// `getEventData(eventIdx) != NULL` (`d_com_inf_game.h:3757-3759`), i.e. exactly "this actor
    /// holds a real event index", so `event_slot.is_some()` is that half.
    /// `eventInfo.checkCommandDemoAccrpt()` is the `mCommand == dEvtCmd_INDEMO_e` latch that
    /// `dEvt_control_c::demoCheck` sets on both ordered actors when the event starts
    /// (`d_event.cpp:314-316`) and `demoEnd` clears (`:331-337`) - so it stays true through the
    /// event's CLOSE stage, which is exactly "one of this actor's events is in `Stage::Play` or
    /// `Stage::Close`", the two states `getMyStaffId` scans (`d_event_manager.cpp:412`).
    ///
    /// Note the function does NOT wait: it arms `field_0x2F6` and then falls straight into
    /// `mode_u_l_init` / `mode_l_u_init` on the SAME tick (`:216-220`), so the lift begins moving
    /// without waiting for the event to actually play.
    fn mode_demoreq(&mut self, events: &mut SimEvents, active: &mut ActiveEvent) {
        if let Some(slot) = self.event_slot
            && matches!(
                active.stage_of(slot),
                Some(Stage::Play) | Some(Stage::Close)
            )
        {
            // `field_0x2F6 = 1;` (:214)
            self.demo_running = true;
        }
        // `if (field_0x2E4 == 3) { mode_u_l_init(); } else { mode_l_u_init(); }` (:216-220).
        // The dispatch is on `field_0x2E4` (the `Mode_e`), NOT on `field_0x2E0`, so the requested
        // direction comes from what `mode_demoreq_init` recorded - and that init's own
        // `field_0x2E0 = 4` is overwritten here immediately, which is why `field_0x2E0` is 1 or 3 by
        // the time `Execute`'s `== 1 || == 3 || == 4` test (`:303`) reads it.
        if self.mode == MODE_U_L {
            self.mode_u_l_init(events);
        } else {
            self.mode_l_u_init(events);
        }
    }

    /// `daObjHlift::Act_c::chk_demo_end` (`d_a_obj_hlift.cpp:273-279`).
    ///
    /// ```c
    /// if (field_0x2F6 != 0 && dComIfGp_evmng_endCheck(mEventId)) {
    ///     dComIfGp_event_reset();
    ///     field_0x2F6 = 0;
    /// }
    /// return field_0x2F6 == 0;
    /// ```
    /// The RETURN VALUE IS DISCARDED at the call site - `chk_demo_end();` is a bare statement at
    /// `:299` - so only the body is transcribed. `dComIfGp_evmng_endCheck(s16)` is `endCheck(
    /// eventIdx)` (`d_com_inf_game.h:3701-3703`), which is `mEventState == CLOSE_e`
    /// (`d_event_manager.cpp:386-392`) - `ActiveEvent::end_check`. `dComIfGp_event_reset()` is
    /// `dEvDtBase_c::reset()` (`d_event.h:131`) = `mEventFlag |= 8`, which `dEvt_control_c::check`
    /// (`d_event.cpp:546-554`) turns into `endProc` -> `demoEnd`; `ActiveEvent::end` is that
    /// teardown, and it clears no `dSv_event_flag_c`.
    fn chk_demo_end(&mut self, active: &mut ActiveEvent) {
        if !self.demo_running {
            return;
        }
        let Some(slot) = self.event_slot else {
            return;
        };
        if active.end_check(slot) {
            let _ = active.end(slot);
            self.demo_running = false;
        }
    }
}

pub fn is_hlift(name: &str) -> bool {
    HLIFT_ACTORS.contains(&name)
}

pub fn control(
    switches: Res<Switches>,
    list: Res<EventList>,
    mut active: ResMut<ActiveEvent>,
    mut ev_flags: ResMut<EventFlags>,
    mut events: ResMut<SimEvents>,
    mut lifts: Query<(Entity, &mut Actor, &mut Hlift)>,
) {
    for (entity, mut actor, mut lift) in &mut lifts {
        if !lift.entered {
            lift.create_init(&mut actor, &switches, &list);
        }
        lift.execute(
            &mut actor,
            &switches,
            &list,
            &mut active,
            &mut ev_flags,
            &mut events,
            entity,
        );
    }
}
