//! `daComing3::Act_c` (`d_a_coming3.h:12-54`) - the Great Sea barrel-buoy race driver behind the
//! stage string `Com_C`. `Nonmatching` count: 0, so every token below is transcribed.
//!
//! IS THIS A DUPLICATE? NO - it is a genuine third variant. The three `Coming` actors are three
//! different `actor_process_profile_definition`s with three different OBJNAME rows and three
//! different translation units:
//!
//! | stage string | profile | TU | module |
//! |---|---|---|---|
//! | `Com_A` | `g_profile_Obj_Coming` | `d_a_obj_coming.cpp` (`:571-586`) | `coming.rs` |
//! | `Com_B` | `g_profile_Coming2` | `d_a_coming2.cpp` (`:676-691`) | `coming2.rs` |
//! | `Com_C` | `g_profile_Coming3` | `d_a_coming3.cpp` (`:512-527`) | this file |
//!
//! `d_stage.cpp:940`, `:1167` and `:1168` respectively; `include/f_pc/f_pc_name.h:284-285` gives
//! `fpcNm_Coming2_e` and `fpcNm_Coming3_e` as adjacent, distinct ids. The behaviour differs too:
//! `Com_A` is a barrel-BOMB course spawner, `Com_B` is a five-mode course flag driver, and `Com_C`
//! is this three-mode buoy racer whose only state machine is the one transcribed below.
//!
//! Nothing else in `src/sim/` claims `Com_C`. `src/sim/gaship.rs`'s `GASHIP_ACTORS = ["Gaship1"]`
//! and `src/sim/gaship2.rs`'s `["Gaship2"]` are different profiles.
//!
//! CULL: the profile says `fopAc_CULLBOX_CUSTOM_e` (`:526`) but the file calls neither
//! `fopAcM_setCullSizeBox` nor `fopAcM_SetMin` / `SetMax` - grep confirms neither appears anywhere in
//! `d_a_coming3.cpp` or `d_a_coming3.h`, and the `mDoMtx_stack_c` writes in `set_mtx` are not those
//! functions. Under the crate's cull rule a CUSTOM profile with no box call means there is NO box,
//! so this module exposes no `cull()` and no `actor_views` branch (the `coming2.rs:10-14`
//! precedent, which audited the same pair of symbols).
//!
//! OMITTED - the barrel PLACEMENT. `coming_start_main`'s over-limit arm can only spawn through
//!
//! ```c
//! if (unk_47C == 0 && get_water_height(&waterHeight, NULL, &current.pos)
//!     && position_is_none_obj(&current.pos) == TRUE) { ... }
//! ```
//!
//! (`:188`). `get_water_height`'s sea branch maps onto `super::sea::Sea` (`daSea_ChkArea` `:65`,
//! `daSea_calcWave` `:66`), but its tail is a per-poly-group ground query -
//!
//! ```c
//! sp08 = *pos; sp08.y += 20000.0f;
//! gnd_work.SetPos(&sp08);
//! if (dComIfG_Bgsp()->GroundCross(&gnd_work) > waterHeight - 100.0f) { return FALSE; }
//! return TRUE;
//! ```
//!
//! (`:97-108`) - which RULE 3b lists as absent and which `super::ground::surface` cannot answer
//! (merged-DZB plane only). So the function's TRUE / FALSE verdict has no value here, and with it
//! the whole gate: `make_coming` (`:199-207`), `position_is_none_obj` (`:136-145`, callback
//! `:118-133`), `make_item_table` (`:196-198`) and the `coming_process_init(1)` transition
//! (`:211`). Every one of those arms is therefore dead in the port exactly as the missing verdict
//! makes it dead - and because `unk_400` is written ONLY inside that arm (`:199`) and cleared at
//! `:269`, it stays `fpcM_ERROR_PROCESS_ID_e` (`f_pc_manager.h:11`) forever, so
//! `fopAcM_SearchByID` (`:230`, `:291`) always fails. That single fact is why `unk_400` is not a
//! component field (RULE 7: an unread field is dead code) and why `coming_wait_main` and the
//! `case 1` / `case 2` arms of `coming_game_main` are transcribed but inert.
//!
//! ALSO OMITTED: `collision_init` / `collision_main` (`:389-415` - `mCyl.ChkTgHit()` is a `cCcD`
//! test and the other arm is `dComIfG_Ccsp()->Set(&mCyl)`, RULE 3b), `eff_break_tsubo`
//! (`:315-356` - `dBgS_ObjGndChk` + `GetMtrlSndId`, two `dComIfG_getObjectRes`, three particle
//! calls), `set_mtx` (`:374-387`), `create_heap` / `solidHeapCB` (`:417-430`), `_create`'s
//! `dComIfG_resLoad` / `fopAcM_entrySolidHeap` (`:436-445`), `_delete` (`:450-454`), `_execute`'s
//! `set_mtx()` (`:459`) and `_draw` (`:463-471`).
//!
//! `L_Attr[0x28]` (`:17-23`) is a file-scope byte table this translation unit never reads, and
//! `static const f32 unused_4595[1] = {6000.0f}` (`:372`) is named `unused` by the decomp itself;
//! neither is transcribed.
//!
//! `Mthd_IsDelete` (`:497-500`) is the bare `return TRUE` form - RULE 6b, it ACCEPTS a delete
//! request and is not a despawn. `_create` (`:432-448`) returns `cPhs_COMPLEATE_e` on every path
//! that is not `fopAcM_entrySolidHeap` failing (`:443-445`, heap work), so `spawn` never refuses.

use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::player::Player;

/// `OBJNAME("Com_C", fpcNm_Coming3_e, 255, 0)` (`d_stage.cpp:1168`) - the only row for
/// `g_profile_Coming3` (`d_a_coming3.cpp:512-527`). `daComing3::Act_c::M_arcname[] = "Always"`
/// (`:474`) is the shared model archive, not a stage name.
pub const COMING3_ACTORS: [&str; 1] = ["Com_C"];

/// ```
/// BOOL coming_start_init() {
///     unk_478 = 0.0f;
///     unk_400 = fpcM_ERROR_PROCESS_ID_e;
///     unk_404 = 3;
///     unk_47C = 30;
///     return TRUE;
/// }
/// ```
/// (`:159-165`) and `} else { unk_47C = 30; }` (`:215`) - the "player is still in range" reload of
/// the away timer. `unk_47C` counts DOWN by one per tick while the player is beyond
/// `get_limit_dist()` (`:183-185`) and is reloaded whenever they come back inside it, so the
/// placement gate at `:188` only opens after `AWAY_TICKS` consecutive far ticks.
const AWAY_TICKS: i16 = 30; // <- d_a_coming3.cpp:163, :215
/// ```
/// f32 limits[] = {3600.0f, 5000.0f};
/// return limits[get_challenge_id()];
/// ```
/// (`:154-155`) - the radius, in stage units, inside which the player is considered present.
const LIMIT_DIST: [f32; 2] = [3600.0, 5000.0]; // <- d_a_coming3.cpp:154
/// `unk_47C = 3` - the value both `coming_start_init` (`:162`) and `coming_wait_init` (`:282`)
/// write into `unk_404`, and the arm `coming_game_main`'s switch sends to `default` (`:274-276`).
/// It is the "no run in progress" sub-step.
const SUB_NONE: i32 = 3; // <- d_a_coming3.cpp:162, :282
/// `unk_404 = 0;` in `coming_game_init` (`:221`) - `case 0` (`:238`), which measures the distance
/// and waits for the barrel to appear.
const SUB_MEASURE: i32 = 0; // <- d_a_coming3.cpp:221
/// `unk_404 = 1;` in `coming_game_main` (`:245`) - `case 1` (`:249`), the wait for the player to
/// close the gap.
const SUB_CLOSE: i32 = 1; // <- d_a_coming3.cpp:245
/// `unk_404 = 2;` in `coming_game_main` (`:263`) - `case 2` (`:267`), where the barrel has been
/// told to leave and this actor waits for it to disappear.
const SUB_LEAVE: i32 = 2; // <- d_a_coming3.cpp:263
/// The three `coming_process_init` table entries (`d_a_coming3.cpp:302-306`), indexed by the
/// `unk_444` argument.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    /// index 0 - `coming_start_init` / `coming_start_main` (`:159-165`, `:168-217`).
    #[default]
    Start, // <- d_a_coming3.cpp:303
    /// index 1 - `coming_game_init` / `coming_game_main` (`:220-223`, `:226-278`).
    Game, // <- d_a_coming3.cpp:304
    /// index 2 - `coming_wait_init` / `coming_wait_main` (`:281-284`, `:287-297`).
    Wait, // <- d_a_coming3.cpp:305
}

/// `init_table[3]` / `main_table[3]` (`d_a_coming3.cpp:302-306`, `:361-365`) - the bound in
/// `coming_process_init`'s `arg1 >= 0 && arg1 < 3` and `coming_process_main`'s `unk_444 >= 0 &&
/// unk_444 < 3` (`:308`, `:367`). The bound IS the table, so it is transcribed rather than guessed.
const TABLE_LEN: i16 = 3; // <- d_a_coming3.cpp:302

impl Mode {
    /// ```
    /// if (arg1 >= 0 && arg1 < 3 && (this->*init_table[arg1])()) { unk_444 = arg1; return true; }
    /// return false;
    /// ```
    /// (`:308-311`) - the guard is the table bound, which `TABLE_LEN` carries.
    fn of(index: i16) -> Option<Self> {
        if index < 0 || index >= TABLE_LEN {
            return None;
        }
        match index {
            0 => Some(Mode::Start),
            1 => Some(Mode::Game),
            2 => Some(Mode::Wait),
            _ => None,
        }
    }
}

/// `daComing3::Act_c` (`d_a_coming3.h:12-54`).
///
/// (a) inbound damage: NONE in the model. `collision_init` (`:389-396`) DOES register a live target
/// collider - `collision_main` (`:410-411`) calls `dComIfG_Ccsp()->Set(&mCyl)` with
/// `SetR(130.0f)` / `SetH(225.0f)` and `M_cyl_src`'s `SrcObjTg Type` is
/// `AT_TYPE_ALL & ~AT_TYPE_WATER & ~AT_TYPE_UNK20000 & ~AT_TYPE_WIND & ~AT_TYPE_UNK400000 &
/// ~AT_TYPE_LIGHT` (`:33`) - so the buoy really can be touched. But `SrcObjAt Type = 0` (`:31`)
/// means the actor deals NO damage, `dCcG_AtHitMark_None_e` / `dCcG_At_Spl_UNK0` (`:37-38`) mean a
/// hit changes nothing about the actor, and `ChkTgHit` at `:255` only ends the run. There is no
/// hit-point field, no `chkHp` and no damage call anywhere in the translation unit, so neither
/// `combat::HitPoints` nor `enemy::Hurt` is attached - `combat::control` would despawn an actor
/// retail cannot damage. The profile `Group` is `fopAc_ACTOR_e` (`:525`).
/// (b) outbound damage: NONE, as above.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Coming3 {
    /// `unk_444` (`d_a_coming3.h:49`) - the `coming_process_init` table index, stored at `:309` and
    /// read by `coming_process_main`'s `main_table[unk_444]` dispatch (`:367-368`).
    pub mode: Mode,
    /// `unk_404` (`d_a_coming3.h:45`) - the sub-step inside `coming_game_main`'s switch (`:237`).
    /// Written by all three inits (`:162`, `:221`, `:282`) and by the switch itself (`:245`, `:256`,
    /// `:259`, `:263`), and read by `_draw` (`:465`).
    pub sub_state: i32,
    /// `unk_47C` (`d_a_coming3.h:53`) - the consecutive-far-ticks countdown, `AWAY_TICKS` at init.
    pub away_ticks: i16,
    /// `unk_478` (`d_a_coming3.h:51`) - the last measured distance to the player: the FLAT one from
    /// `coming_start_main` (`:175-181`) or the FULL 3D one from `coming_game_main` (`:240-242`,
    /// `:251-253`).
    pub player_dist: f32,
}

impl Coming3 {
    /// `daComing3::Act_c::_create` (`:432-448`) plus `coming_process_init(0)` (`:441`).
    ///
    /// ```
    /// unk_47E = shape_angle.z + 1;   // :439 - folded into `challenge_id` below
    /// collision_init();             // :440 - cCcD, omitted
    /// coming_process_init(0);       // :441
    /// set_mtx();                    // :442 - matrix work, omitted
    /// ```
    ///
    /// DROPPED: `fopAcM_ct` (`:434`), `dComIfG_resLoad` (`:436`),
    /// `fopAcM_entrySolidHeap` / `solidHeapCB` / `create_heap` (`:438-439`), `collision_init`
    /// (`:440`) and `set_mtx` (`:442`). The profile's `Parameters` field is `0` (`:518`), so this
    /// actor reads NO parameter word - RULE 7 is satisfied trivially and `unk_47E` comes from the
    /// stage file's roll angle rather than from a parameter.
    pub fn spawn() -> Self {
        // `coming_start_init` (:159-165), reached through `coming_process_init(0)` (:441).
        Self {
            // `unk_444 = 0;` (:309)
            mode: Mode::Start,
            // `unk_404 = 3;` (:162)
            sub_state: SUB_NONE,
            // `unk_47C = 30;` (:163)
            away_ticks: AWAY_TICKS,
            // `unk_478 = 0.0f;` (:160)
            player_dist: 0.0,
        }
    }

    /// `daComing3::Act_c::get_challenge_id` (`:147-150`):
    /// ```
    /// s32 daComing3::Act_c::get_challenge_id() { return (unk_47E - 1) & 1; }
    /// ```
    /// with `unk_47E = shape_angle.z + 1` in `_create` (`:439`), so the stored field and the `& 1`
    /// fold exactly to `shape_angle.z & 1` for every `i16` roll. The same id indexes
    /// `LIMIT_DIST` (`:155`) and `make_item_table` (`:203`).
    fn challenge_id(shape_roll: i16) -> usize {
        (shape_roll & 1) as usize
    }

    /// `daComing3::Act_c::get_limit_dist` (`:152-156`):
    /// ```
    /// f32 limits[] = {3600.0f, 5000.0f};
    /// return limits[get_challenge_id()];
    /// ```
    /// RULE 10 guards the `get_challenge_id()` index; `challenge_id` already masks to 0..=1.
    fn limit_dist(shape_roll: i16) -> f32 {
        LIMIT_DIST[Self::challenge_id(shape_roll)]
    }

    /// `daComing3::Act_c::coming_process_init` (`:299-313`).
    fn process_init(&mut self, index: i16) -> bool {
        // `if (arg1 >= 0 && arg1 < 3 && (this->*init_table[arg1])()) { unk_444 = arg1; return true; }`
        // (:308-311)
        let Some(mode) = Mode::of(index) else {
            return false;
        };
        let ok = match mode {
            // ```
            // BOOL coming_start_init() {
            //     unk_478 = 0.0f;
            //     unk_400 = fpcM_ERROR_PROCESS_ID_e;
            //     unk_404 = 3;
            //     unk_47C = 30;
            //     return TRUE;
            // }
            // ```
            // (:159-165) - `unk_400 = fpcM_ERROR_PROCESS_ID_e;` (:161) is dropped with the barrel
            // handle (module docs): it is already the sentinel and nothing else can change it.
            Mode::Start => {
                self.player_dist = 0.0;
                self.sub_state = SUB_NONE;
                self.away_ticks = AWAY_TICKS;
                true
            }
            // ```
            // BOOL coming_game_init() { unk_404 = 0; return unk_400 != fpcM_ERROR_PROCESS_ID_e; }
            // ```
            // (:220-223) - the return value is FALSE for the whole port, because `unk_400` is only
            // ever assigned by the omitted placement gate at `:199`. So `unk_444` is NOT updated and
            // the caller sees `false` - exactly what retail does when no barrel exists.
            Mode::Game => {
                self.sub_state = SUB_MEASURE;
                false
            }
            // ```
            // BOOL coming_wait_init() { unk_404 = 3; return TRUE; }
            // ```
            // (:281-284)
            Mode::Wait => {
                self.sub_state = SUB_NONE;
                true
            }
        };
        // `unk_444 = arg1;` (:309)
        if ok {
            self.mode = mode;
        }
        ok
    }

    /// `daComing3::Act_c::coming_process_main` (`:358-370`):
    /// ```
    /// if (unk_444 >= 0 && unk_444 < 3) { (this->*main_table[unk_444])(); }
    /// ```
    /// against `main_table[3] = { coming_start_main, coming_game_main, coming_wait_main }`
    /// (`:361-365`). `main_func` returns `void`, so there is no value to propagate.
    fn process_main(&mut self, player: Option<Vec3>, origin: Vec3, shape_roll: i16) {
        match self.mode {
            // `coming_start_main` (:168-217)
            Mode::Start => self.start_main(player, origin, shape_roll),
            // `coming_game_main` (:226-278)
            Mode::Game => self.game_main(player, origin),
            // `coming_wait_main` (:287-297)
            Mode::Wait => self.wait_main(),
        }
    }

    /// `daComing3::Act_c::coming_start_main` (`:168-217`).
    ///
    /// ```
    /// daPy_py_c* player = daPy_getPlayerActorClass();
    /// if (player == NULL) { return; }
    /// cXyz sp30 = player->current.pos;
    /// cXyz sp24 = current.pos;
    /// sp24.y = 0.0f;
    /// sp30.y = 0.0f;
    /// unk_478 = std::sqrtf(sp30.getSquareDistance(sp24));
    /// if (unk_478 > get_limit_dist()) {
    ///     if (unk_47C > 0) { unk_47C--; }
    ///     if (unk_47C == 0 && get_water_height(&waterHeight, NULL, &current.pos)
    ///         && position_is_none_obj(&current.pos) == TRUE) { ...spawn + coming_process_init(1)... }
    /// } else {
    ///     unk_47C = 30;
    /// }
    /// ```
    ///
    /// OMITTED: everything inside the `unk_47C == 0 && ...` gate (`:188`) - see the module docs.
    fn start_main(&mut self, player: Option<Vec3>, origin: Vec3, shape_roll: i16) {
        // `if (player == NULL) { return; }` (:171-173) - `daPy_getPlayerActorClass()`.
        let Some(player) = player else {
            return;
        };
        // ```
        // sp24.y = 0.0f; sp30.y = 0.0f;
        // unk_478 = std::sqrtf(sp30.getSquareDistance(sp24));
        // ```
        // (:178-181) - FLAT distance: both `.y` components are zeroed before the square distance.
        let flat_player = Vec3::new(player.x, 0.0, player.z);
        let flat_self = Vec3::new(origin.x, 0.0, origin.z);
        self.player_dist = flat_player.distance(flat_self);
        // `if (unk_478 > get_limit_dist()) {` (:182)
        if self.player_dist > Self::limit_dist(shape_roll) {
            // `if (unk_47C > 0) { unk_47C--; }` (:183-185)
            if self.away_ticks > 0 {
                self.away_ticks -= 1;
            }
            // OMITTED - the placement gate `unk_47C == 0 && get_water_height(...) &&
            // position_is_none_obj(...)` (:188) and everything it opens: `make_coming` (:199-207),
            // `make_item_table[get_challenge_id()]` (:196-198, :203) and
            // `if (unk_400 != fpcM_ERROR_PROCESS_ID_e) { coming_process_init(1); }` (:210-212).
        } else {
            // `} else { unk_47C = 30; }` (:215)
            self.away_ticks = AWAY_TICKS;
        }
    }

    /// `daComing3::Act_c::coming_game_main` (`:226-278`).
    ///
    /// ```
    /// daObjBarrel2::Act_c* barrel;
    /// BOOL foundBarrel = fopAcM_SearchByID(unk_400, (fopAc_ac_c**)&barrel);
    /// if (barrel != NULL) { current.pos = barrel->current.pos; shape_angle = barrel->shape_angle; }
    /// switch (unk_404) { case 0: ... case 1: ... case 2: ... default: coming_process_init(2); }
    /// ```
    ///
    /// `unk_400` is permanently `fpcM_ERROR_PROCESS_ID_e` (module docs), so `foundBarrel` is FALSE
    /// and `barrel` is NULL on every tick. The `current.pos` / `shape_angle` copy (`:232-235`) is
    /// therefore skipped, and the arms that need the barrel are transcribed but unreachable.
    fn game_main(&mut self, player: Option<Vec3>, origin: Vec3) {
        // `switch (unk_404) {` (:237)
        match self.sub_state {
            // ```
            // case 0:
            //     if (player != NULL) {
            //         f32 sqDist = player->current.pos.getSquareDistance(current.pos);
            //         unk_478 = std::sqrtf(sqDist);
            //     }
            //     if (barrel != NULL) { unk_404 = 1; }
            //     break;
            // ```
            // (:238-247) - FULL 3D distance here, unlike `coming_start_main`'s flattened one.
            SUB_MEASURE => {
                if let Some(player) = player {
                    self.player_dist = player.distance(origin);
                }
                // `if (barrel != NULL) { unk_404 = 1; }` (:244-246) - unreachable (module docs).
            }
            // ```
            // case 1:
            //     if (player != NULL) { unk_478 = std::sqrtf(player->current.pos
            //                                              .getSquareDistance(current.pos)); }
            //     if (collision_main() == true) { unk_404 = 3; eff_break_tsubo(); }
            //     else if (!foundBarrel) { unk_404 = 3; eff_break_tsubo(); }
            //     else if ((unk_478 <= get_limit_dist()) && (barrel != NULL)) {
            //         barrel->exit_req(); unk_404 = 2;
            //     }
            //     break;
            // ```
            // (:249-265) - the distance is the same FULL 3D one as `case 0`.
            SUB_CLOSE => {
                if let Some(player) = player {
                    self.player_dist = player.distance(origin);
                }
                // OMITTED: `collision_main()` (:255) - its `mCyl.ChkTgHit()` is a `cCcD` test and
                // its other arm is `dComIfG_Ccsp()->Set(&mCyl)` (`:399-415`); `eff_break_tsubo()`
                // (:256`, `:260`) - `dBgS_ObjGndChk`, `GetMtrlSndId`, `dComIfG_getObjectRes` and
                // three `dComIfGp_particle_set` calls (`:315-356`).
                //
                // ```
                // } else if (!foundBarrel) { unk_404 = 3; eff_break_tsubo(); }
                // ```
                // (:258-260) - `foundBarrel` is always FALSE, so this arm is taken every tick.
                self.sub_state = SUB_NONE;
                // `else if ((unk_478 <= get_limit_dist()) && (barrel != NULL)) { barrel->exit_req();
                // unk_404 = 2; }` (:261-264) - unreachable: it is the `else` of the arm above, which
                // is why `get_limit_dist` takes no second argument here.
            }
            // ```
            // case 2:
            //     if (!foundBarrel) { unk_400 = fpcM_ERROR_PROCESS_ID_e; coming_process_init(0); }
            //     break;
            // ```
            // (:267-272) - `unk_400 = fpcM_ERROR_PROCESS_ID_e;` (:269) is the value it already
            // holds (module docs), so only the transition is transcribed.
            SUB_LEAVE => {
                self.process_init(0);
            }
            // ```
            // default:
            //     coming_process_init(2);
            //     break;
            // ```
            // (:274-276)
            _ => {
                self.process_init(2);
            }
        }
    }

    /// `daComing3::Act_c::coming_wait_main` (`:287-297`):
    /// ```
    /// fopAcM_SearchByID(unk_400, &ac1);
    /// daObjBarrel2::Act_c* barrel = (daObjBarrel2::Act_c*)ac1;
    /// if (barrel != NULL && !fopAcM_SearchByID(barrel->get_item_id(), &ac2)) {
    ///     barrel->exit_req();
    ///     fopAcM_delete(this);
    /// }
    /// ```
    ///
    /// The body needs the barrel AND the absence of the item actor the barrel pops out. `unk_400`
    /// is never assigned (module docs), so `fopAcM_SearchByID` fails, `barrel` is NULL and the
    /// condition cannot hold. Transcribed as the inert form rather than invented.
    fn wait_main(&mut self) {}
}

pub fn is_coming3(name: &str) -> bool {
    COMING3_ACTORS.contains(&name)
}

pub fn control(
    mut buoys: Query<(&Actor, &mut Coming3), With<Coming3>>,
    players: Query<&Player>,
) {
    // RULE 9 / RULE 12(d): the access set is `&Actor` (read, `With<Coming3>`), `&mut Coming3`
    // (write, `With<Coming3>`) and `&Player` (read). No param writes a component another reads, and
    // nothing here writes `Actor`, so there is no intra-system ambiguity to guard.
    //
    // `daPy_getPlayerActorClass()` (:169`, `:227) - the single `Player` in the stage.
    let player = players.iter().next().map(|player| player.pos);
    for (actor, mut buoy) in &mut buoys {
        // `_execute` (:456-461): `coming_process_main(); set_mtx();` - the `set_mtx()` half is
        // matrix work and is omitted.
        buoy.process_main(player, actor.pos, actor.shape_angle[2]);
    }
}