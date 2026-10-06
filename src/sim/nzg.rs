use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::player::Player;

/// The stage string is `nezuana` (`d_stage.cpp:669`, `fpcNm_NZG_e`) - NOT `nezumi` and NOT
/// `nezuami`. `nezumi` (`d_stage.cpp:668`) is `fpcNm_NZ_e`, a different actor, and `NpcNz`
/// (`d_stage.cpp:703`) is `fpcNm_NPC_NZ_e`. Neither is claimed here.
pub const NZG_ACTORS: [&str; 1] = ["nezuana"];

/// `parameters |= 0x100` (`d_a_nzg.cpp:66`, `:68`), tested at `:71`.
const CHILD_PARAM_BIT: u32 = 0x100;
/// `cM_rnd() < 0.5f` (`d_a_nzg.cpp:67`).
const RANDOM_CHANCE: f32 = 0.5;
/// `i_this->m2D0 = 400.0f` (`d_a_nzg.cpp:107`), rewritten every Execute.
const SPAWN_DISTANCE: f32 = 400.0;
/// `i_this->m2CC = i_this->m2B8 * 10.0f` (`d_a_nzg.cpp:177`).
const NEAR_SCALE: f32 = 10.0;
/// `s16 m2C2[5]` (`d_a_nzg.h:21`).
const COOLDOWN_SLOTS: usize = 5;
/// `for (s32 i = 4; i != 0; i--)` (`d_a_nzg.cpp:109`): only four of the five slots tick.
const COOLDOWN_TICKS: usize = 4;
/// `fopAcM_GetParam(nzg_this)` byte 0 -> `nzg_this->m2B8` (`d_a_nzg.cpp:169`).
const KIND_MASK: u32 = 0xFF;
/// byte 1 -> `nzg_this->m2B9` (`d_a_nzg.cpp:170`).
const KIND_SHIFT: u32 = 8;
/// byte 2 -> `nzg_this->m2BA` (`d_a_nzg.cpp:171`).
const GROUP_SHIFT: u32 = 16;
/// `i_this->m2BA == 0` (`d_a_nzg.cpp:65`, `:218`).
const GROUP_RAT: u8 = 0;
/// `i_this->m2BA == 1` (`d_a_nzg.cpp:65`).
const GROUP_GUARDED: u8 = 1;
/// `i_this->m2BA == 2` (`d_a_nzg.cpp:67`).
const GROUP_RANDOM: u8 = 2;
/// `if (nzg_this->m2BA > 2) nzg_this->m2BA = 0` (`d_a_nzg.cpp:183`).
const GROUP_MAX: u8 = 2;
/// `nzg_this->m2C0 = 1` (`d_a_nzg.cpp:178`).
const DEFAULT_RATS: i16 = 1;
/// `fpcM_ERROR_PROCESS_ID_e` (`d_a_nzg.cpp:89`, `:101`): the `m2D4` "no blocked actor" sentinel.
const NO_BLOCKER: i16 = -1;
/// `cM_initRnd(100, 100, 100)` (`m_Do_machine.cpp:617`).
const RND_SEED: i32 = 100;
/// `cM_rnd` multipliers (`c_math.cpp:178-180`).
const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
/// `cM_rnd` moduli (`c_math.cpp:178-180`).
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
/// `cM_rnd` reciprocals (`c_math.cpp:181`).
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
/// `fopAc_CULLBOX_0_e` (`d_a_nzg.cpp:248`) -> `f_op_actor_mng.cpp:558`.
const CULL_MIN: [f32; 3] = [-40.0, 0.0, -40.0];
const CULL_MAX: [f32; 3] = [40.0, 125.0, 40.0];

/// `m2BB` (`d_a_nzg.h:17`) is the two-entry dispatch of `daNZG_Execute` (`d_a_nzg.cpp:115-122`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// `case 0: nzg_00_move(i_this)` (`d_a_nzg.cpp:116-117`).
    Move,
    /// `case 1: nzg_01_move(i_this)` (`d_a_nzg.cpp:119-120`).
    Unblock,
}

/// The Forsaken Fortress rat-spawner clump. It registers `dCcD_SrcCyl body_cyl_src`
/// (`d_a_nzg.cpp:187-215`) with `SrcObjAt Type = 0` (`:191`), `SrcObjTg Type = 0` (`:194`) and a
/// Co-only `SrcObjCo SPrm` (`:196`), so it can neither be hit nor hurt anything - even though
/// `nzg_class` derives from `fopEn_enemy_c` (`d_a_nzg.h:11`), which carries no health field
/// (`f_op_actor.h:323-367`). Its profile `Group` is `fopAc_ACTOR_e` (`:247`), not `fopAc_ENEMY_e`.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Nzg {
    /// `nzg_this->m2BA` (`d_a_nzg.h:16`), clamped by `:183`.
    pub group: u8,
    /// `nzg_this->m2BB` (`d_a_nzg.h:17`).
    pub mode: Mode,
    /// `nzg_this->m2BE` (`d_a_nzg.h:19`).
    pub spawned: i16,
    /// `nzg_this->m2C0` (`d_a_nzg.h:20`).
    pub rat_cap: i16,
    /// `nzg_this->m2C2` (`d_a_nzg.h:21`).
    pub cooldown: [i16; COOLDOWN_SLOTS],
    /// `nzg_this->m2CC` (`d_a_nzg.h:22`).
    pub near: f32,
    /// `nzg_this->m2D0` (`d_a_nzg.h:23`), rewritten to `400.0f` every Execute (`:107`).
    pub spawn_distance: f32,
    /// `nzg_this->m2D4` (`d_a_nzg.h:24`); written only at `:101` and by the omitted Co-hit path
    /// (`:56`), so it holds `NO_BLOCKER` throughout here.
    pub blocker: i16,
    seed: [i32; 3],
}

impl Nzg {
    /// `daNZG_Create` (`d_a_nzg.cpp:160-224`): `m2B8`/`m2B9`/`m2BA` (`:169-171`),
    /// `m2CC = m2B8 * 10.0f` (`:177`), `m2C0 = 1` then `= m2B9` (`:178-181`), the `m2BA > 2`
    /// clamp (`:183-185`) and the `mCyl.Set(body_cyl_src)` profile (`:216-217`). Byte 3 of the
    /// parameter (`m2E9`, `:172`) feeds only the omitted `dPath_GetRoomPath` (`:173-175`) and is
    /// not stored (RULE 7); `m2C2`, `m2BE`, `m2D0`, `m2D4` and `m2BB` start at zero.
    pub fn spawn(parameters: u32) -> Self {
        let group = ((parameters >> GROUP_SHIFT) & KIND_MASK) as u8;
        let cap = ((parameters >> KIND_SHIFT) & KIND_MASK) as i16;
        Self {
            group: if group > GROUP_MAX { GROUP_RAT } else { group },
            mode: Mode::Move,
            spawned: 0,
            rat_cap: if cap != 0 { cap } else { DEFAULT_RATS },
            cooldown: [0; COOLDOWN_SLOTS],
            near: (parameters & KIND_MASK) as f32 * NEAR_SCALE,
            spawn_distance: SPAWN_DISTANCE,
            blocker: NO_BLOCKER,
            seed: [RND_SEED; 3],
        }
    }

    /// The profile is `fopAc_CULLBOX_0_e` (`d_a_nzg.cpp:248`), so no `setCullSizeBox` is needed.
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `daNZG_Execute` (`d_a_nzg.cpp:106-124`).
    fn step(&mut self, pos: Vec3, player: Vec3) {
        self.spawn_distance = SPAWN_DISTANCE;
        for slot in self.cooldown.iter_mut().take(COOLDOWN_TICKS) {
            if *slot != 0 {
                *slot -= 1;
            }
        }
        match self.mode {
            Mode::Move => self.move_00(pos, player),
            Mode::Unblock => self.move_01(),
        }
    }

    /// `nzg_00_move` (`d_a_nzg.cpp:39-85`) minus the `mCyl` registration (`:46-49`), the
    /// `mCyl.ChkCoHit()` block (`:51-61`) and the `fopAcM_createChild` tail (`:74-82`).
    fn move_00(&mut self, pos: Vec3, player: Vec3) {
        if self.cooldown[0] != 0 || self.spawned >= self.rat_cap {
            return;
        }
        let parameters = self.child_params();
        let distance = search_player_distance(pos, player);
        let near = (parameters & CHILD_PARAM_BIT) != 0 || distance < self.near * 0.5;
        if near && distance > self.spawn_distance * 0.5 {
            // `parameters |= 1` (:73), `child_angle.y += cM_rndFX(0x2000)` (:74-76),
            // `fopAcM_createChild(fpcNm_NZ_e, ...)` (:77) and its success block (:79-82) are
            // omitted: neither `fpcNm_NZ_e` nor `NpcNz` exists in this sim, and
            // `m2C2[0] = (childProcID & 3) * 10 + 20` needs a proc id this sim has no allocator
            // for.
        }
    }

    /// The `parameters` assembly of `d_a_nzg.cpp:63-69`. The high byte of
    /// `fopAcM_GetParam(actor)` is dropped because only the omitted `createChild` consumes it,
    /// and bit `0x100` is the sole bit the `if` at `:71` tests.
    fn child_params(&mut self) -> u32 {
        if self.group == GROUP_GUARDED {
            CHILD_PARAM_BIT
        } else if self.group == GROUP_RANDOM && rnd(&mut self.seed) < RANDOM_CHANCE {
            CHILD_PARAM_BIT
        } else {
            0
        }
    }

    /// `nzg_01_move` (`d_a_nzg.cpp:88-103`) minus the `fopAcM_SearchByID` probe (`:90-99`),
    /// which can never run because `m2D4`'s only other writer is the omitted Co-hit block.
    fn move_01(&mut self) {
        if self.blocker != NO_BLOCKER {
            return;
        }
        self.blocker = NO_BLOCKER;
        self.mode = Mode::Move;
    }
}

/// `fopAcM_searchPlayerDistance` (`f_op_actor_mng.cpp:665-667`) reaches
/// `fopAcM_searchActorDistance` (`f_op_actor_mng.cpp:499-502`), which returns
/// `(*fopAcM_GetPosition_p(other) - *fopAcM_GetPosition_p(this)).abs()` - the three-axis
/// distance (`c_xyz.h:124`).
fn search_player_distance(pos: Vec3, player: Vec3) -> f32 {
    (player - pos).length()
}

/// `cM_rnd` (`c_math.cpp:177-182`), with the three global lanes held on the component.
fn rnd(seed: &mut [i32; 3]) -> f32 {
    for lane in 0..3 {
        seed[lane] = (seed[lane] * RND_MUL[lane]) % RND_MOD[lane];
    }
    let sum =
        seed[0] as f32 / RND_SCALE[0] + seed[1] as f32 / RND_SCALE[1] + seed[2] as f32 / RND_SCALE[2];
    (sum % 1.0).abs()
}

/// `fpcNm_NZ_e` is `d_stage.cpp:668` and `NpcNz` is `d_stage.cpp:703`; neither actor exists in
/// this sim, so `nzg` spawns nothing and its own spawn bookkeeping stays inert.
pub fn is_nzg(name: &str) -> bool {
    NZG_ACTORS.contains(&name)
}

pub fn control(players: Query<&Player>, mut clumps: Query<(&Actor, &mut Nzg)>) {
    let Some(player) = players.iter().next() else {
        return;
    };
    for (actor, mut clump) in &mut clumps {
        clump.step(actor.pos, player.pos);
    }
}