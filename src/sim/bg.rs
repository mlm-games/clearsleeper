use bevy_ecs::prelude::*;

use super::area::StageName;

/// `OBJNAME("Bg", fpcNm_BG_e, 255, 0)` (`d_stage.cpp:1238`) is the only row bound to `fpcNm_BG_e`
/// (`d_a_bg.cpp:458`). There is no `_arcname`: the archive name is BUILT at run time by
/// `daBg_c::setArcName()` (`:28-32`) as `sprintf(arcName, "Room%d", fopAcM_GetParam(this))`, so the
/// resource depends on the actor's own parameter and there is no constant to cite for it.
pub const BG_ACTORS: [&str; 1] = ["Bg"];

/// `strcmp(dComIfGp_getStartStageName(), "sea") == 0` (`d_a_bg.cpp:308`) - the stage whose room
/// background models are given the longer unload delay.
pub const SEA_STAGE: &str = "sea"; // <- d_a_bg.cpp:308
/// ```
/// if (strcmp(dComIfGp_getStartStageName(), "sea") == 0) { mUnloadTimer = 16; }
/// else { mUnloadTimer = 1; }
/// ```
/// (`d_a_bg.cpp:308-311`). Retail is `VERSION_NTSC_U > VERSION_JPN`, so the `#else` arm of the
/// `#if VERSION <= VERSION_JPN` guard at `:305` is the one that ships, and these are its values.
pub const UNLOAD_SEA: i32 = 16; // <- d_a_bg.cpp:309
pub const UNLOAD_ROOM: i32 = 1; // <- d_a_bg.cpp:311

/// The two `dComIfGp_roomControl` status bits this translation unit touches are `0x04`
/// (`dComIfGp_roomControl_checkStatusFlag(fopAcM_GetParam(this), 0x04)`, `d_a_bg.cpp:304`) - the
/// sole trigger that arms [`Bg::unload_timer`] - and `0x10`
/// (`dComIfGp_roomControl_onStatusFlag(roomNo, 0x10)`, `:441`, cleared by
/// `dComIfGp_roomControl_offStatusFlag(roomNo, 0x10)` in the destructor at `:254`). Neither is
/// carried as a constant here: both belong to a `dComIfGp_roomControl` status-flag channel that
/// does not exist in this sim, so they are part of the omitted behaviour rather than live inputs.
/// See the report.

/// The stage-room background model actor (`daBg_c`, `d_a_bg.cpp`).
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. `daBg_c` registers no collider of any kind
/// anywhere in the translation unit - there is no `dCcD_Sph`, no `mStts`, no `fopAcM_entrySolidHeap`
/// collider - and its profile `Group` is `fopAc_ACTOR_e` (`:466`). Neither `combat::HitPoints` nor
/// `enemy::Hurt` is attached.
///
/// `daBg_IsDelete` (`:352-355`) is NOT the bare accepts-a-delete-request form: it forwards
/// `daBg_c::isDelete()` (`:345-350`), which returns `TRUE` only while `mUnloadTimer == 0` and
/// `FALSE` once the timer is armed. So the actor refuses deletion while it is mid-unload, and
/// `fopAcM_delete(this)` at `:299` is what actually ends it. RULE 6b's other arm applies here: the
/// timer expiry is a real despawn, modelled with `Commands`, not with `Option<Self> -> None`.
///
/// The `fopAc_CULLBOX_0_e` in the profile (`:468`) resolves to a literal box at
/// `f_op_actor_mng.cpp:558`, but the profile's `Status` (`:466`) is
/// `fopAcStts_UNK4000_e | fopAcStts_UNK40000_e` with NO `fopAcStts_CULL_e`, and
/// `fopAcM_cullingCheck` is gated on that flag, so the box is inert data here. There is therefore
/// no `cull()` and no `actor_views` branch; see the report.
///
/// The room number the actor reads is `fopAcM_GetParam(this)` (`:258`, `:304`, `:372`) - the WHOLE
/// u32 parameter, not a masked byte - and it comes from `Actor::parameters` rather than a field.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bg {
    /// `mUnloadTimer` - declared by `daBg_c` and read at `:293`, `:298` and `:346`. Zero means the
    /// actor is not unloading and accepts a delete request; non-zero counts down to the despawn.
    pub unload_timer: i32,
}

impl Bg {
    pub fn spawn() -> Self {
        // The ctor's zero-fill is the whole initial state: `mUnloadTimer` is only ever assigned
        // inside `execute` (`:306`, `:309`, `:311`) and the destructor never resets it.
        Self { unload_timer: 0 }
    }
}

/// `cLib_calcTimer(T* value)` (`c_lib.h:85-90`): decrement unless already zero, then return the
/// stored value. The decomp tests the RETURN (`d_a_bg.cpp:298`), so the despawn lands on the tick
/// the counter reaches zero.
fn calc_timer(value: &mut i32) -> i32 {
    if *value != 0 {
        *value -= 1;
    }
    *value
}

/// `daBg_c::execute` (`d_a_bg.cpp:292-338`), NTSC-U arms only.
pub fn control(
    stage: Res<StageName>,
    mut backgrounds: Query<(Entity, &mut Bg), With<Bg>>,
    mut commands: Commands,
) {
    for (entity, mut background) in &mut backgrounds {
        // ```
        // if (mUnloadTimer != 0) {
        //     if (cLib_calcTimer(&mUnloadTimer) == 0) { fopAcM_delete(this); }
        //     return TRUE;
        // }
        // ```
        // (:293-302), minus the `#if VERSION <= VERSION_JPN` arm at `:294-297` which zeroes the
        // timer immediately and is excluded by the retail version ordering.
        if background.unload_timer != 0 {
            if calc_timer(&mut background.unload_timer) == 0 {
                let _ = commands.entity(entity).despawn();
            }
            // `return TRUE;` (:301) - the room-status test and the animation loop are skipped.
            continue;
        }
        // ```
        // if (dComIfGp_roomControl_checkStatusFlag(fopAcM_GetParam(this), 0x04)) {
        //     if (strcmp(dComIfGp_getStartStageName(), "sea") == 0) { mUnloadTimer = 16; }
        //     else { mUnloadTimer = 1; }
        // } else { ...play the four btk/brk animation pairs... }
        // ```
        // (:304-335). The `else` arm is the four-iteration `bgm->btk->play()` / `bgm->brk->play()`
        // loop, which is BRK/BTK archive playback - renderer, and RULE 3b confirms model/BRK/BCK
        // archives are unavailable. The `if` arm's gate has no modelled channel: there is no
        // `dComIfGp_roomControl` status-flag resource in this sim, so the gate is transcribed and
        // fed `false` (see the report). The two constants and the stage-name comparison are kept
        // because they are the arm's real contents, not because the arm is currently reachable.
        let room_flagged = false;
        if room_flagged {
            background.unload_timer = if stage.0 == SEA_STAGE {
                UNLOAD_SEA
            } else {
                UNLOAD_ROOM
            };
        }
        // `s32 roomNo = fopAcM_GetParam(this);` (:258, `:304`, `:372`) - read, not stored, because
        // with the gate fed `false` the parameter has no other modelled reader. RULE 3's reason
        // for not carrying it is NOT "the value is unknown": the OBJNAME row is
        // `OBJNAME("Bg", fpcNm_BG_e, 255, 0)` so nothing in the checkout fixes it either way.
    }
}

pub fn is_bg(name: &str) -> bool {
    BG_ACTORS.contains(&name)
}
