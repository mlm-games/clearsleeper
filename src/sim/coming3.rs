//! `daComing3::Act_c` (`d_a_coming3.h:12-96`) - the Great Sea barrel buoy that drifts in with a rupee
//! for the Round 2 barrel-buoy race. `Nonmatching` count: 0, so every token below is transcribed.
//!
//! Nothing in `src/sim/` claims `Com_C`. `src/sim/gaship.rs`'s `GASHIP_ACTORS = ["Gaship1"]`
//! (`:3`) and `src/sim/gaship2.rs`'s `["Gaship2"]` (`:3`) are different profiles.

use bevy_ecs::prelude::*;

/// `OBJNAME("Com_C", fpcNm_Coming3_e, 255, 0)` (`d_stage.cpp:1168`). `fpcNm_Coming3_e` is
/// `d_a_coming3.cpp`; `daComing3::Act_c::M_arcname[] = "Always"` (`:474`) is the shared archive,
/// not a stage name.
pub const COMING3_ACTORS: [&str; 1] = ["Com_C"];

/// `daComing3::Act_c` - the round-2 barrel buoy.
///
/// The struct has NO fields and there is NO `control` system, and both facts are forced by the
/// data rather than chosen. Every executable line of the actor funnels through one predicate that
/// cannot be evaluated here, and every other behaviour is archive, collider, particle or
/// `dBgW` work. Recording the actor at all is still correct - the stage string must be claimed so
/// no other module takes it, and `spawn_actor` needs a branch - but there is nothing to run.
///
/// * `coming_start_init` (`:159-165`) seeds `unk_478`, `unk_400 = fpcM_ERROR_PROCESS_ID_e`,
///   `unk_404 = 3` and `unk_47C = 30`, and `coming_process_init` (`:300-313`) stores `unk_444 = 0`
///   from it. Those four fields are read only by the three `coming_*_main` bodies and by
///   `_draw` (`:465`), all omitted below, so RULE 7 forbids storing them.
/// * `get_water_height` (`:58-110`) is the gate on the whole actor. Its sea branch
///   (`daSea_ChkArea` at `:65` and `daSea_calcWave` at `:66`) maps onto `super::sea::Sea`, but the
///   else branch's `dBgS_WtrChk` (`:68-75`, `:77-86`) and - decisively - the tail
///   `gnd_work.SetPos(&sp08); if (dComIfG_Bgsp()->GroundCross(&gnd_work) > waterHeight - 100.0f)
///   { return FALSE; } return TRUE;` (`:97-108`) are per-poly-group ground queries, which RULE 3b
///   lists as absent: `super::collision::Collision` is merged-DZB only and `super::ground::surface`
///   is NOT a substitute - using it would be the invented equivalent RULE 1 forbids. So the
///   function's TRUE / FALSE verdict, and therefore `coming_start_main`'s
///   `if (unk_47C == 0 && get_water_height(&waterHeight, NULL, &current.pos) &&
///   position_is_none_obj(&current.pos) == TRUE)` (`:188`), has no value here. (The `f32
///   waterHeight = 6000.0f;` at `:62` is dead anyway - `:63` overwrites it with `-G_CM3D_F_INF`.)
/// * `coming_start_main` (`:168-217`) - the player distance `sp30` / `sp24` with both `.y` zeroed
///   (`:175-181`) and the `unk_478 > get_limit_dist()` test (`:182`) are plain arithmetic and
///   transcribe, but the only thing the over-limit branch can do is spawn the barrel, and the
///   spawn is inside the unavailable gate. `unk_47C = 30` on the near branch (`:215`) is also
///   unreachable, since `unk_47C` is only ever counted down from `:163` inside the branch that
///   reaches it.
/// * `coming_game_init` (`:220-223`) returns `unk_400 != fpcM_ERROR_PROCESS_ID_e`, i.e. it only
///   succeeds once a barrel exists - which needs the omitted spawn.
/// * `coming_game_main` (`:226-278`) copies `current.pos` and `shape_angle` from the barrel
///   (`fopAcM_SearchByID(unk_400)`, `:230-235`), which is a process-id lookup; the map to an ECS
///   `Entity` is fine, but `unk_400` is never set because the spawn is omitted. Its branch
///   conditions are `collision_main()` (`:255`, collider), `!foundBarrel` (`:258`),
///   `unk_478 <= get_limit_dist()` (`:261`) and `barrel->exit_req()` (`:262`).
/// * `coming_wait_main` (`:287-297`) needs the barrel AND `barrel->get_item_id()`'s process to have
///   gone. `get_item_id` returns the item actor the barrel pops out; `src/sim/buoybarrel.rs`'s
///   `BuoyBarrel::take()` (`:379-382`) instead sets the barrel's `item_save` SWITCH bit, and
///   `make_coming` forces `item_save = 0x7F` (`d_a_obj_barrel2.h:87`), i.e.
///   `buoybarrel::NO_ITEM_SAVE` (`:72`), so no bit is written and no item actor exists to wait
///   for. `fopAcM_delete(this)` at `:295` goes with it.
/// * `collision_main` (`:399-415`) - `mCyl.ChkTgHit()` (`:400`) is a `cCcD` test, RULE 3b absent.
///   Its `else` arm is `dComIfG_Ccsp()->Set(&mCyl)` with `SetR(130.0f)` / `SetH(225.0f)` and an
///   `eyePos` write (`:408-414`) - collider and camera data. `collision_init` (`:390-396`) and
///   `M_cyl_src`'s `Radius 30.0f` / `Height 60.0f` (`:51-52`) with `SrcObjTg Type = AT_TYPE_ALL &
///   ~AT_TYPE_WATER & ~AT_TYPE_UNK20000 & ~AT_TYPE_WIND & ~AT_TYPE_UNK400000 & ~AT_TYPE_LIGHT`
///   (`:33`) are the same story.
/// * `eff_break_tsubo` (`:316-356`) - `dBgS_ObjGndChk` + `GetMtrlSndId` for the
///   `JA_SE_OBJ_COL_SWC_CHNS` variant (`:317-328`), two `dComIfG_getObjectRes("Always", ...)` loads
///   (`:337-338`), two `dComIfGp_particle_set` calls (`:340`, `:355`) and a
///   `dPa_J3DmodelEmitter_c` allocation (`:343-346`).
/// * `position_is_none_obj` (`:136-145`) and its callback (`:118-133`) - a `fopAcM_Search` over
///   every actor looking for `fopAcM_GetGroup(actor) == fopAc_ENEMY_e` within `200.0f` of XZ.
///   The group test is `Query<&Actor, With<Enemy>>` and the distance is plain arithmetic, but it is
///   only ever called from the unavailable `get_water_height` gate (`:188`).
/// * `daObjBarrel2::Act_c::make_coming` (`:199-207`, `d_a_obj_barrel2.h:77-92`) - the spawn itself.
///   It builds the parameter word `make_prm(Type_01_e = 0x1, droppedItem, !hasFlag = true, true,
///   daObjBuoyflag::Texture_00_e = 0x0)` = `(itemNo << 0) | (0x7F << 0x10) | (1 << 0x18) | (0 << 0x8)
///   | (0 << 0xA) | (1 << 0x1C)`, which `BuoyBarrel::spawn` would decode, but it is only reached
///   from `coming_start_main` behind the gate. `make_item_table` is
///   `{dItemNo_BLUE_RUPEE_e, dItemNo_YELLOW_RUPEE_e}` (`:196-198`, `d_item_data.h:9-10`) indexed by
///   `get_challenge_id()`, and the table is not stored for the same reason.
/// * `get_challenge_id` (`:148-150`) is `(unk_47E - 1) & 1` with `unk_47E = shape_angle.z + 1`
///   (`:439`), which folds exactly to `shape_angle.z & 1`; `get_limit_dist` (`:153-156`) is
///   `{3600.0f, 5000.0f}` (`:154`) indexed by it. Both transcribe cleanly and both are read only
///   by the omitted `coming_start_main` / `coming_game_main` / the spawn.
/// * `create_heap` (`:424-430`), `set_mtx` (`:375-387`) with its `cXyz(0.0f, 390.0f, 0.0f)`
///   translation and `cXyz(2.0f, 2.0f, 2.0f)` scale (`:376-377`), `_create`'s
///   `dComIfG_resLoad` / `fopAcM_entrySolidHeap` (`:436-445`), `_delete` (`:451-454`),
///   `_execute`'s `set_mtx()` (`:459`) and `_draw` (`:464-471`) - model, matrix, heap and renderer
///   work. `_create` can only return `cPhs_ERROR_e` from `fopAcM_entrySolidHeap` (`:443-445`), i.e.
///   heap work, so there is no refusal.
///
/// `Mthd_IsDelete` (`:497-500`) is the bare `return TRUE` form - RULE 6b, it ACCEPTS a delete
/// request and is not a despawn.
///
/// `L_Attr[0x28]` (`:17-23`) is a file-scope byte table that this translation unit never reads, so
/// it is not transcribed; `static const f32 unused_4595[1] = {6000.0f}` (`:372`) is named
/// `unused` by the decomp itself.
///
/// Cull: the profile says `fopAc_CULLBOX_CUSTOM_e` (`:526`) but the file calls neither
/// `fopAcM_setCullSizeBox` nor `fopAcM_SetMin` / `SetMax`, and `mDoMtx_stack_c` writes are not
/// `fopAcM_SetMin` / `SetMax`. Under the crate's cull rule a CUSTOM profile with no box call means
/// there is NO box, so this module deliberately exposes no `cull()` and no `actor_views` branch.
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. `M_cyl_src` sets `SrcObjAt Type = 0`
/// (`:31`) - the buoy is a pure target - and `dCcG_AtHitMark_None_e` / `dCcG_At_Spl_UNK0` (`:37-38`)
/// mean a hit changes nothing. The profile `Group` is `fopAc_ACTOR_e` (`:525`). Neither
/// `combat::HitPoints` nor `enemy::Hurt`.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Coming3;

impl Coming3 {
    /// `daComing3::Act_c::_create` (`:433-448`), minus `fopAcM_ct` (`:434`),
    /// `dComIfG_resLoad` (`:436`), `fopAcM_entrySolidHeap` / `solidHeapCB` / `create_heap`
    /// (`:438-439`), `collision_init` (`:440`), `coming_process_init` (`:441`) and `set_mtx` (`:442`).
    /// The profile's `Parameters` field is `0` (`:518`), and `unk_47E` comes from the stage file's
    /// roll angle rather than from a parameter word.
    pub fn spawn() -> Self {
        Coming3
    }
}

pub fn is_coming3(name: &str) -> bool {
    COMING3_ACTORS.contains(&name)
}
