//! `daObjSwheavy::Act_c` (`d_a_obj_swheavy.h:19-94`) - the Wind Temple iron-boots switch.
//! `Nonmatching` count: 0, so every token below is transcribed.
//!
//! Nothing in `src/sim/` claims `Hhbot1` or `Hhbot1N`.

use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::events::{SimEvent, SimEvents};
use super::flags::Switches;

/// `OBJNAME("Hhbot1", fpcNm_Obj_Swheavy_e, 0, 0)` (`d_stage.cpp:567`) and
/// `OBJNAME("Hhbot1N", fpcNm_Obj_Swheavy_e, 1, 0)` (`:568`). `fpcNm_Obj_Swheavy_e` is
/// `d_a_obj_swheavy.cpp`; the archive is `daObjSwheavy::Act_c::M_arcname[] = "Hhbot"`
/// (`d_a_obj_swheavy.cpp:12`), which is NOT either stage string.
pub const OBJ_SWHEAVY_ACTORS: [&str; 2] = ["Hhbot1", "Hhbot1N"];

/// `fopAcM_setCullSizeBox(this, -97.5f, -2.0f, -97.5f, 97.5f, 55.0f, 97.5f)`
/// (`d_a_obj_swheavy.cpp:131`) - the actor's one real custom cull box, matching the profile's
/// `fopAc_CULLBOX_CUSTOM_e` (`:485`).
const CULL_MIN: [f32; 3] = [-97.5, -2.0, -97.5]; // <- d_a_obj_swheavy.cpp:131
const CULL_MAX: [f32; 3] = [97.5, 55.0, 97.5]; // <- d_a_obj_swheavy.cpp:131

/// `daObjSwheavy::Prm_e::PRM_TYPE_S = 0x18` with `PRM_TYPE_W = 0x03` (`d_a_obj_swheavy.h:31-32`),
/// so `daObj::PrmAbstract(this, 3, 0x18)` = `((1 << 3) - 1) & (param >> 24)` (`d_a_obj.h:14`) -
/// three bits, not a byte.
const TYPE_SHIFT: u32 = 0x18; // <- d_a_obj_swheavy.h:32
const TYPE_MASK: u32 = 0x07; // <- d_a_obj_swheavy.h:31 (PRM_TYPE_W = 0x03, d_a_obj.h:14)
/// `daObjSwheavy::Prm_e::PRM_SWSAVE_S = 0x08`, `PRM_SWSAVE_W = 0x08` (`:34-35`).
const SWSAVE_SHIFT: u32 = 0x08; // <- d_a_obj_swheavy.h:35
const SWSAVE_MASK: u32 = 0xFF; // <- d_a_obj_swheavy.h:34 (PRM_SWSAVE_W = 0x08, d_a_obj.h:14)

/// `daObjSwheavy::Flag` (`d_a_obj_swheavy.h:23-28`), the bit values `cLib_checkBit<u32>` compares
/// `attr().mFlags` against. Row 0 (`:13-23`) and row 1 (`:24-33`) of `M_attr` are all-zero, so
/// only indices 2 and 3 of [`SWHEAVY_FLAGS`] can be non-zero.
const FLAG_OBEY_SAVE: u32 = 0x1; // <- d_a_obj_swheavy.h:29
const FLAG_STAY_PRESSED: u32 = 0x2; // <- d_a_obj_swheavy.h:30
const FLAG_IS_TOGGLE: u32 = 0x4; // <- d_a_obj_swheavy.h:31
const FLAG_REQ_HEAVY: u32 = 0x8; // <- d_a_obj_swheavy.h:32

/// `daObjSwheavy::Act_c::M_attr[4]` (`.mFlags`, `d_a_obj_swheavy.cpp:16`, `:26`, `:36`, `:46`).
/// Row 2 is `FLAG_IS_TOGGLE | FLAG_REQ_HEAVY` (`:36`) and row 3 is
/// `FLAG_OBEY_SAVE | FLAG_STAY_PRESSED | FLAG_REQ_HEAVY` (`:46`).
///
/// WIDTH HAZARD, guarded at every read: `attr()` is `M_attr[mType]` (`d_a_obj_swheavy.h:21`) over
/// FOUR rows, while `prm_get_type()` is `daObj::PrmAbstract(this, 0x03, 0x18)` - THREE bits
/// (`d_a_obj_swheavy.h:31-32`), so `mType` can be 4..7 and retail reads past the table. Rust must
/// not index out of bounds (RULE 10), so every lookup here goes through [`SwHeavy::row`], which
/// falls back to row 0 - the provably inert row, `mFlags = 0` with all three floats `0.0f`
/// (`d_a_obj_swheavy.cpp:13-23`), i.e. a flagless button whose spring never moves. No Wind Temple
/// stage data in this checkout sets a `prm_get_type()` above 3.
const SWHEAVY_FLAGS: [u32; 4] = [
    0,          // <- d_a_obj_swheavy.cpp:16
    0,          // <- d_a_obj_swheavy.cpp:26
    FLAG_IS_TOGGLE | FLAG_REQ_HEAVY,      // <- d_a_obj_swheavy.cpp:36
    FLAG_OBEY_SAVE | FLAG_STAY_PRESSED | FLAG_REQ_HEAVY, // <- d_a_obj_swheavy.cpp:46
];

/// `mVSpring` (`.cpp:37`, `:46`) - the spring constant in `calc_top_pos`'s
/// `mVSpeed -= (mCurHFrac - mTargetHFrac) * attr().mVSpring` (`:220`).
const VSPRING: [f32; 4] = [0.0, 0.0, 0.93, 0.93]; // <- d_a_obj_swheavy.cpp:37
/// `mVSpeedDecay` (`.cpp:38`, `:47`) - `mVSpeed -= mVSpeed * decay` (`:221`).
const VSPEED_DECAY: [f32; 4] = [0.0, 0.0, 0.42, 0.42]; // <- d_a_obj_swheavy.cpp:38
/// `mPushVSpeed0` (`.cpp:39`, `:48`) - `mVSpeed = attr().mPushVSpeed0` in `mode_u_l_init` (`:325`).
const PUSH_VSPEED0: [f32; 4] = [0.0, 0.0, 0.0, 0.0]; // <- d_a_obj_swheavy.cpp:39

/// `m328 = (1.0f - mCurHFrac) * -35.5f` (`.cpp:223`). The stage-top and stage-bottom offsets of
/// the two models are both built from this one scale.
const TRAVEL: f32 = -35.5; // <- d_a_obj_swheavy.cpp:223
/// `m328 = cLib_minMaxLimit(m328, -36.5f, 1.0f)` (`.cpp:224`), and `cLib_minMaxLimit`
/// (`c_lib.h:60-62`) is a plain two-sided clamp. `f32::clamp` asserts `min <= max`, and here both
/// ends are constants with `-36.5 < 1.0`, so the call is safe.
const TRAVEL_MIN: f32 = -36.5; // <- d_a_obj_swheavy.cpp:224
const TRAVEL_MAX: f32 = 1.0; // <- d_a_obj_swheavy.cpp:224
/// `eyePos.y = current.pos.y + m328 - -35.5f` (`.cpp:416`) - the same `-35.5f` as [`TRAVEL`],
/// subtracted, so the sign flips.
const EYE_LIFT: f32 = 35.5; // <- d_a_obj_swheavy.cpp:416

/// `fopAcM_seStart(this, JA_SE_OBJ_FOOT_SW_ON, 0)` (`.cpp:339`,
/// `JAZelAudio_SE.h:1836`).
const SE_FOOT_SW_ON: u32 = 0x6930; // <- JAZelAudio_SE.h:1836

/// `mMode` indices into `_execute`'s `mode_proc[]` (`.cpp:399-404`).
const MODE_UPPER: u8 = 0;
const MODE_U_L: u8 = 1;
const MODE_LOWER: u8 = 2;
const MODE_L_U: u8 = 3;

/// `mTargetHFrac = 1.0f` (`.cpp:144`, `:290`, `:322` target `0.0f` at `:325`, `:347`, `:382`).
const HEIGHT_UP: f32 = 1.0; // <- d_a_obj_swheavy.cpp:144
const HEIGHT_DOWN: f32 = 0.0; // <- d_a_obj_swheavy.cpp:152

/// `daObjSwheavy::Act_c` - the Wind Temple button that the iron boots hold down.
///
/// The struct is the live subset of `Act_c` (`d_a_obj_swheavy.h:70-96`). Folded out, with the
/// proof for each fold:
///
/// * `mRiding` (`h:79`) and `mHeavyRiding` (`h:83`) are initialised `false` by `_create` (`:132`,
///   `:136`), reset `false` by every `_execute` (`:410-411`), and are set `true` ONLY inside
///   `rideCB` (`:209`, `:212`) - a `dBgW` moving-background ride callback, which RULE 3b lists as
///   absent. `rideCB`'s other two inputs are `fopAcM_CheckStatus(i_pt, fopAcStts_FREEZE_e)`
///   (`:208`) and `((daPy_py_c*)i_pt)->checkEquipHeavyBoots()` (`:211`); `super::player::Player`
///   carries no freeze status and `super::inventory::Equipped` has no heavy-boots query. So both
///   flags are PROVEN false every tick, which folds `set_push_flag` (`:249-285`) exactly:
///   `mMiniPushFlg` starts false (`:135`) and is set true only at `:261` inside `if (mRiding)`, so
///   the `else` arm runs and `mMiniPushTimer = 0` (`:264`); `mPushFlg` likewise (`:139`, `:279`,
///   `mPushTimer = 0` at `:282`).
/// * Consequently `mMiniPushFlg` / `mPushFlg` are compile-time `false`, so in `mode_upper`
///   (`:295-319`) `pressing` is false and `height` keeps its `1.0f` initialiser (`:297`) - the
///   `height = 0.9f` branch (`:303`) is unreachable. In `mode_lower` (`:352-377`) `pressing` is
///   false, so `pop_back_up` reduces to `!cLib_checkBit(mFlags, FLAG_STAY_PRESSED)` (`:364-365`)
///   and `match_state_up` to `OBEY_SAVE && !is_switch()` (`:366-368`).
/// * `mPrevRiding` / `mPrevHeavyRiding` (`h:80`, `h:84`) are written by `_execute` (`:408-409`) and
///   read by nothing else in the file.
/// * `m330` / `m334` / `m32C` (`h:92-94`) exist only to drive `m338` (`:232-235`) and `mTopPos`
///   (`:236-239`), and `mTopPos`'s only reader is `set_mtx`'s `mDoMtx_stack_c::get()[1][3] +=
///   mTopPos` (`:194`) - model-matrix work. `top_bg_aim_req` (`:243-246`) therefore writes nothing
///   this crate keeps, and `attr().m14` (`:326`), `attr().mMiniPushDelay` (`:252`, `:260`) and
///   `attr().mPushDelay` (`:270`, `:278`) lose their readers with them.
///
/// OMITTED for the same reasons: `create_heap` (`:62-103`, two `dBgW` + two models),
/// `dComIfG_Bgsp()->Regist` / `Release` (`:124-127`, `:169-178`), `_delete` (`:168-181`),
/// `set_mtx` / `init_mtx` (`:184-203`), `mDoExt_modelUpdateDL` (`:427-428`), `_draw` (`:422-431`),
/// and `scale.z *= 1.5f; scale.x *= 1.5f;` (`:113-114`) - `scale` is read only by
/// `mDoMtx_stack_c::scaleM` (`:117`, `:192`) and `setBaseScale` (`:200-201`), and nothing in
/// `src/` reads `Actor::scale`.
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. `Act_c` holds two models, two `dBgW`s and
/// three matrices (`d_a_obj_swheavy.h:70-76`) and registers no `dCcD` collider; the profile
/// `Group` is `fopAc_ACTOR_e` (`.cpp:484`). Neither `combat::HitPoints` nor `enemy::Hurt`.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct SwHeavy {
    pub kind: u8,
    pub switch_no: u8,
    pub mode: u8,
    pub changing_state: bool,
    pub target_h: f32,
    pub cur_h: f32,
    pub v_speed: f32,
    pub travel: f32,
}

impl SwHeavy {
    /// `daObjSwheavy::Act_c::_create` (`.cpp:106-165`), minus `fopAcM_ct` (`:107`),
    /// `dComIfG_resLoad` (`:109`), the `fopAcM_entrySolidHeap` block (`:123-162`), `scale` (`:113`),
    /// `set_mtx` / `init_mtx` (`:115-118`, `:130`), the `fopAcM_setCullSizeBox` call (`:131`) and the
    /// `mRiding` / `mHeavyRiding` / `mMiniPush*` / `mPush*` initialisers (`:132-141`) and the
    /// `m330` / `m334` / `m338` / `mTopPos` / `mVSpeed` initialisers (`:146-148`, `:140`).
    ///
    /// `bool up = cLib_checkBit<u32>(attr().mFlags, FLAG_IS_TOGGLE) || !is_switch();` (`:112`) picks
    /// the branch. `is_switch()` is `fopAcM_isSwitch(this, prm_get_swSave())` (`h:38`), which is
    /// `dComIfGs_isSwitch(sw, fopAcM_GetHomeRoomNo(this))` (`f_op_actor_mng.h:374-376`) - the room
    /// argument is `dSv_memBit_c::isSwitch`'s second parameter, which the single in-memory
    /// `Switches` array already is. RULE 11b: the REAL index goes through; a `swSave >= 0x80`
    /// button would read a per-zone `dSv_zoneBit_c` bit in retail and reads false here.
    ///
    /// `phase_state` can only become `cPhs_ERROR_e` from `fopAcM_entrySolidHeap` (`:160-162`),
    /// which is heap work, so there is no refusal.
    pub fn spawn(parameters: u32, switches: &Switches) -> Self {
        let kind = ((parameters >> TYPE_SHIFT) & TYPE_MASK) as u8;
        let switch_no = ((parameters >> SWSAVE_SHIFT) & SWSAVE_MASK) as u8;
        let row = match kind as usize {
            0..=3 => kind as usize,
            _ => 0,
        };
        let up = SWHEAVY_FLAGS[row] & FLAG_IS_TOGGLE != 0 || !switches.test(switch_no);
        let mut this = Self {
            kind,
            switch_no,
            mode: MODE_UPPER,
            changing_state: false,
            target_h: HEIGHT_UP,
            cur_h: if up { HEIGHT_UP } else { HEIGHT_DOWN },
            v_speed: 0.0,
            travel: if up { 0.0 } else { TRAVEL },
        };
        if up {
            this.mode_upper_init();
        } else {
            this.mode_lower_init();
        }
        this
    }

    /// The `M_attr` row for this actor's `mType`, with the three-bit-versus-four-row guard
    /// documented on [`SWHEAVY_FLAGS`].
    fn row(&self) -> usize {
        match self.kind as usize {
            0..=3 => self.kind as usize,
            _ => 0,
        }
    }

    fn flags(&self) -> u32 {
        SWHEAVY_FLAGS[self.row()]
    }

    fn is_switch(&self, switches: &Switches) -> bool {
        switches.test(self.switch_no)
    }

    /// `fopAcM_revSwitch` (`f_op_actor_mng.h:378-380`) is `dComIfGs_revSwitch` =
    /// `dSv_memBit_c::revSwitch` (`d_save.cpp:1102-1109`), whose body is
    /// `mSwitch[idx] ^= sw` - an exclusive-or of the one bit. `Switches` exposes `test` / `set` /
    /// `clear` but no xor, so the toggle is spelled out; the index is unchanged.
    fn rev_switch(&self, switches: &mut Switches) {
        if self.is_switch(switches) {
            switches.clear(self.switch_no);
        } else {
            switches.set(self.switch_no);
        }
    }

    fn on_switch(&self, switches: &mut Switches) {
        switches.set(self.switch_no);
    }

    fn off_switch(&self, switches: &mut Switches) {
        switches.clear(self.switch_no);
    }

    /// `mode_upper_init` (`.cpp:288-292`).
    fn mode_upper_init(&mut self) {
        self.mode = MODE_UPPER;
        self.target_h = HEIGHT_UP;
        self.changing_state = false;
    }

    /// `mode_lower_init` (`.cpp:345-349`).
    fn mode_lower_init(&mut self) {
        self.mode = MODE_LOWER;
        self.target_h = HEIGHT_DOWN;
        self.changing_state = false;
    }

    /// `mode_u_l_init` (`.cpp:322-327`), minus `top_bg_aim_req(0.0f, attr().m14)` (`:326`).
    fn mode_u_l_init(&mut self) {
        self.mode = MODE_U_L;
        self.target_h = HEIGHT_DOWN;
        self.v_speed = PUSH_VSPEED0[self.row()];
    }

    /// `mode_l_u_init` (`.cpp:380-384`), minus `top_bg_aim_req(1.0f, 1)` (`:383`).
    fn mode_l_u_init(&mut self) {
        self.mode = MODE_L_U;
        self.target_h = HEIGHT_UP;
    }

    /// `mode_upper` (`.cpp:295-319`) with `pressing` folded to false and `height` to `1.0f`:
    /// `if (pressing || (OBEY_SAVE && is_switch()))` (`:310`) -> `if (OBEY_SAVE && is_switch())`,
    /// `if (pressing) mChangingState = true;` (`:311-313`) drops out, and the `else` arm keeps
    /// `mTargetHFrac = height` (`:316`).
    fn mode_upper(&mut self, switches: &Switches) {
        if self.flags() & FLAG_OBEY_SAVE != 0 && self.is_switch(switches) {
            self.mode_u_l_init();
        } else {
            self.target_h = HEIGHT_UP;
        }
    }

    /// `mode_u_l` (`.cpp:330-342`).
    fn mode_u_l(&mut self, switches: &mut Switches, events: &mut SimEvents) {
        if self.cur_h > HEIGHT_DOWN {
            return;
        }
        if self.changing_state {
            if self.flags() & FLAG_IS_TOGGLE != 0 {
                self.rev_switch(switches);
            } else {
                self.on_switch(switches);
            }
        }
        events.list.push(SimEvent::Se(SE_FOOT_SW_ON));
        self.mode_lower_init();
    }

    /// `mode_lower` (`.cpp:352-377`) with `pressing` folded to false: `pop_back_up` is
    /// `!(flags & FLAG_STAY_PRESSED)` and `match_state_up` is `(flags & FLAG_OBEY_SAVE) &&
    /// !is_switch()`, and `if (pop_back_up && !is_toggle && !pressing)` (`:372-374`) keeps only
    /// `pop_back_up && !is_toggle`.
    fn mode_lower(&mut self, switches: &Switches) {
        let flags = self.flags();
        let pop_back_up = flags & FLAG_STAY_PRESSED == 0;
        let match_state_up = flags & FLAG_OBEY_SAVE != 0 && !self.is_switch(switches);
        if !pop_back_up && !match_state_up {
            return;
        }
        if pop_back_up && flags & FLAG_IS_TOGGLE == 0 {
            self.changing_state = true;
        }
        self.mode_l_u_init();
    }

    /// `mode_l_u` (`.cpp:387-394`) verbatim - it has no omitted arm.
    fn mode_l_u(&mut self, switches: &mut Switches) {
        if self.cur_h < HEIGHT_UP {
            return;
        }
        if self.changing_state {
            self.off_switch(switches);
        }
        self.mode_upper_init();
    }

    /// `calc_top_pos` (`.cpp:218-240`), minus the `m330` / `m32C` / `m338` / `mTopPos` half
    /// (`:226-239`), whose readers are all in the omitted `set_mtx` / renderer path.
    fn calc_top_pos(&mut self) {
        let row = self.row();
        self.v_speed -= (self.cur_h - self.target_h) * VSPRING[row];
        self.v_speed -= self.v_speed * VSPEED_DECAY[row];
        self.cur_h += self.v_speed;
        let raw = (1.0 - self.cur_h) * TRAVEL;
        self.travel = if raw < TRAVEL_MIN {
            TRAVEL_MIN
        } else if raw > TRAVEL_MAX {
            TRAVEL_MAX
        } else {
            raw
        };
    }

    /// `eyePos.x = current.pos.x; eyePos.y = current.pos.y + m328 - -35.5f; eyePos.z =
    /// current.pos.z;` (`.cpp:415-417`).
    fn eye_pos(&self, actor: &Actor) -> Vec3 {
        Vec3::new(actor.pos.x, actor.pos.y + self.travel + EYE_LIFT, actor.pos.z)
    }

    /// `_execute` (`.cpp:397-419`): `set_push_flag()` (`:405`) folds to the two timer clears
    /// (documented on the struct and elided), then `(this->*mode_proc[mMode])()` (`:406`),
    /// `calc_top_pos()` (`:407`), the `mPrev*` snapshots and the `mRiding` resets (`:408-411`,
    /// all write-only), `set_mtx()` / `mpBgW*->Move()` (`:412-414`) and the `eyePos` write (`:415-417`).
    fn execute(&mut self, actor: &mut Actor, switches: &mut Switches, events: &mut SimEvents) {
        match self.mode {
            MODE_UPPER => self.mode_upper(switches),
            MODE_U_L => self.mode_u_l(switches, events),
            MODE_LOWER => self.mode_lower(switches),
            MODE_L_U => self.mode_l_u(switches),
            _ => {}
        }
        self.calc_top_pos();
        let eye = self.eye_pos(actor);
        actor.eye = eye;
    }

    /// `fopAcM_setCullSizeBox(this, -97.5f, -2.0f, -97.5f, 97.5f, 55.0f, 97.5f)` (`.cpp:131`).
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

pub fn is_obj_swheavy(name: &str) -> bool {
    OBJ_SWHEAVY_ACTORS.contains(&name)
}

/// One system, three params. Access set per param: `Switches` (W - `rev_switch` / `on_switch` /
/// `off_switch`), `SimEvents` (W - `events.list.push`), `Actor` (W - `eye` at `.cpp:415-417`),
/// `SwHeavy` (RW). No two params touch the same component with a conflicting mode, so there is no
/// intra-system ambiguity.
pub fn control(
    mut switches: ResMut<Switches>,
    mut events: ResMut<SimEvents>,
    mut buttons: Query<(&mut Actor, &mut SwHeavy)>,
) {
    for (mut actor, mut button) in &mut buttons {
        let actor = &mut *actor;
        let button = &mut *button;
        button.execute(actor, &mut switches, &mut events);
    }
}
