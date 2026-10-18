use bevy_ecs::prelude::*;
use glam::Vec3;

/// `OBJNAME("TagSo", fpcNm_TAG_SO_e, 0, 0)` (`d_stage.cpp:570`) - the stage-placed jump tag.
pub const TAG_ACTOR: &str = "TagSo";

/// `OBJNAME("TagMSo", fpcNm_TAG_SO_e, 1, 0)` (`d_stage.cpp:571`) - the same process, differing
/// only in the `argument` column (`OBJNAME`'s third argument, `d_stage.cpp:432`). `daTag_So_c`
/// never reads `fopAcM_prm_class::argument`, so both rows build the identical actor and one
/// predicate covers them.
pub const MAP_TAG_ACTOR: &str = "TagMSo";

/// `mType = fopAcM_GetParamBit(param, 16, 8)` (`d_a_tag_so.cpp:58`) into the `u8 mType`
/// (`d_a_tag_so.h:23`). `fopAcM_GetParamBit(param, shift, bit)` is
/// `(param >> shift) & ((1 << bit) - 1)` (`f_op_actor_mng.h:131-133`).
const TYPE_SHIFT: u32 = 16; // <- d_a_tag_so.cpp:58
const TYPE_MASK: u32 = 0xFF; // <- d_a_tag_so.cpp:58

/// `BOOL isMinigame() { return mType == 1; }` (`d_a_tag_so.h:10`) and
/// `BOOL isTag() { return mType != 1; }` (`:11`) - the only two readers of `mType`.
const MINIGAME_TYPE: u32 = 1; // <- d_a_tag_so.h:10

/// The stage-placed jump tag - `daTag_So_c` (`d_a_tag_so.h:6-25`), the actor behind both stage
/// strings above.
///
/// The struct has ONE field, `minigame`, which is `isMinigame()` / `mType == 1`. The other two
/// decoded parameters are deliberately not stored - see `spawn`.
///
/// `daTag_So_c::_execute` (`d_a_tag_so.cpp:31-33`) is `return TRUE;` and NOTHING else: there is no
/// statement, no timer, no state and no mode in the whole 127-line translation unit, so no
/// `control` system is derivable. The rest of the class is equally empty:
///
/// * `daTag_So_c::_draw` (`d_a_tag_so.cpp:44-48`) is
///   `if (l_HIO.m05) debugDraw(); return TRUE;`, and `daTag_So_HIO_c`'s constructor
///   (`d_a_tag_so.cpp:23-26`) leaves `m05 = 0`, so `debugDraw` (`:36-41`) never runs in retail and
///   is a `dLib_debugDrawFan` debug primitive besides - renderer work.
/// * `daTag_So_c::_create` (`:68-72`) is `fopAcM_ct` + `getArg()` + `cPhs_COMPLEATE_e`: it can
///   never refuse creation, so there is no spawn-then-despawn path.
/// * `daTag_So_c::_delete` (`:75-77`) and `daTag_SoIsDelete` (`:100-102`) are both the bare
///   "accepts a delete request" form of RULE 6b.
///
/// There is no `cull()` here either, and that is a deliberate refusal rather than an omission:
/// the profile's `fopAc_CULLBOX_4_e` (`d_a_tag_so.cpp:126`) does resolve to a real box through
/// `l_cullSizeBox[4]` - `(-100.0f, 0.0f, -100.0f)..(100.0f, 800.0f, 100.0f)`
/// (`f_op_actor_mng.cpp:562`) - but `src/lib.rs`'s `actor_views` has no branch for `Tag`, so a
/// `cull()` would have no reader, and an unread `pub fn` is dead code (RULE 12b).
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. `daTag_So_c` holds only `mRndNum`,
/// `mJumpRange`, `mType` and a padding byte (`d_a_tag_so.h:21-24`), registers no `dCcD` collider,
/// and its profile `Group` is `fopAc_ACTOR_e` (`d_a_tag_so.cpp:125`). Neither `combat::HitPoints`
/// nor `enemy::Hurt` is attached.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Tag {
    /// `isMinigame()` (`d_a_tag_so.h:10`) - read by `super::talk`'s `any_minigame_tag`
    /// (`talk.rs:331-333`), the sim's stand-in for `daNpc_So_c::_searchMinigameTagSo`.
    pub minigame: bool,
}

impl Tag {
    /// `daTag_So_c::getArg` (`d_a_tag_so.cpp:51-65`), the whole of `daTag_So_c::_create`'s
    /// behaviour beyond `fopAcM_ct` (`:69`).
    ///
    /// `pos` and `yaw` are `i_this->current.pos` and `i_this->shape_angle.y`, which retail reads
    /// only in the dev-only `debugDraw` (`:37-40`) and in `daNpc_So_c::_searchTagSo`
    /// (`d_a_npc_so.cpp:241-244`, an actor this crate does not have - `NpcSo` /
    /// `fpcNm_NPC_SO_e`, `d_stage.cpp:702`, has no module). They are already on `Actor` as
    /// `pos` and `angle[1]`, so storing them again would be a duplicated, unread field (RULE 12b);
    /// `src/sim/mod.rs`'s `spawn_actor` branch passes them, hence the signature.
    ///
    /// The two parameters that are transcribed and then dropped are:
    ///
    /// * `mRndNum = fopAcM_GetParamBit(param, 0, 8)` (`:56`) - its only reader is
    ///   `daNpc_So_c::_searchTagSo`'s `mA79 == tag_so->getRndNum()` (`d_a_npc_so.cpp:241`).
    /// * `paramRadius = fopAcM_GetParamBit(param, 8, 8)` (`:57`) then
    ///   `if (paramRadius == 0xff) { mJumpRange = 1600.0f; } else { mJumpRange = paramRadius * 100; }`
    ///   (`:59-64`) - `mJumpRange` is an `f32` (`d_a_tag_so.h:22`) whose only readers are
    ///   `_searchTagSo`'s `mA7C = tag_so->getJumpRange()` (`d_a_npc_so.cpp:242`) and the dev-only
    ///   `dLib_debugDrawFan(..., mJumpRange, ...)` (`:40`). Note the `0xFF` sentinel branch is
    ///   NOT a constant fold: `paramRadius == 0xff` yields `1600.0f`, while `paramRadius == 255`
    ///   would yield `255 * 100 == 25500`, i.e. sixteen times as far.
    pub fn spawn(parameters: u32, _pos: Vec3, _yaw: i16) -> Self {
        let minigame = ((parameters >> TYPE_SHIFT) & TYPE_MASK) == MINIGAME_TYPE;
        Self { minigame }
    }
}

pub fn is_tag(name: &str) -> bool {
    name == TAG_ACTOR || name == MAP_TAG_ACTOR
}
