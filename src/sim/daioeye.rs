use bevy_ecs::prelude::*;

use super::actor::Actor;
use super::arrow;
use super::bombflower::Blasts;
use super::boomerang;
use super::events::{SimEvent, SimEvents};
use super::hookshot;
use super::walk;

/// Stage object name (`tww/src/d/d_stage.cpp:805`). `daDaiocta_Eye_c` is
/// `fpcNm_DAIOCTA_EYE_e`, listed `0x0007`, draw prio `fpcDwPi_DAIOCTA_EYE_e`, status
/// `fopAcStts_UNK4000_e | fopAcStts_UNK40000_e`, cull type `fopAc_CULLBOX_4_e`
/// (`d_a_daiocta_eye.cpp:521-536`). Its archive is `"daiocta"` (`d_a_daiocta_eye.cpp:17`),
/// which differs from the stage string.
///
/// `createInit` sets `mbAppear = false` (`d_a_daiocta_eye.cpp:451`) and `_execute` bails on it
/// (`:371-373`), so retail only ever raises the flag from `daDaiocta_c::modeAppear`
/// (`d_a_daiocta.cpp:914-919`) on the eyes `daDaiocta_c::createInit` spawns with
/// `fopAcM_createChild` (`d_a_daiocta.cpp:1651-1659`). A stage-placed `Daioeye` therefore never
/// appears, never registers its sphere (`d_a_daiocta_eye.cpp:403-405`) and can never be hit, so
/// `appear` stays false here for the same reason: `src/sim/daiocta.rs` does not spawn eyes.
pub const DAIOEYE_ACTORS: [&str; 1] = ["Daioeye"];

const CULL_MIN: [f32; 3] = [-100.0, 0.0, -100.0]; // <- fopAc_CULLBOX_4_e (f_op_actor_mng.cpp:555)
const CULL_MAX: [f32; 3] = [100.0, 800.0, 100.0]; // <- fopAc_CULLBOX_4_e (f_op_actor_mng.cpp:555)

/// `createInit`: `max_health = 4; health = max_health;` (`d_a_daiocta_eye.cpp:457-458`).
const MAX_HEALTH: i32 = 4;
const SCALE_DAMAGE_TIME: i32 = 8;

/// `daDaiocta_Eye_HIO_c::mSphRadius` (`d_a_daiocta_eye.cpp:53`), re-applied every tick at
/// `:403`.
const SPH_RADIUS: f32 = 195.0;

/// `scale_table[]` of `modeDamage` (`d_a_daiocta_eye.cpp:312-317`).
const SCALE_TABLE: [f32; 8] = [1.0, 0.975, 1.0, 0.95, 1.0, 0.9, 1.0, 0.85];
const SCALE_STEP_SCALE: f32 = 0.2;
const SCALE_STEP_MAX: f32 = 10.0;
const SCALE_STEP_MIN: f32 = 0.025;

const SE_LK_LAST_HIT: u32 = 0x2828;
const SE_LK_W_WEP_HIT: u32 = 0x2833;
const SE_LK_MS_WEP_HIT: u32 = 0x2834;
const SE_CV_DO_DAMAGE: u32 = 0x48C8;
const SE_CV_DO_EYE_LOSE: u32 = 0x48CB;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Wait,
    Damage,
    Death,
}

/// The AT types `checkTgHit`'s `switch (tg_hit_obj_p->GetAtType())` tests
/// (`d_a_daiocta_eye.cpp:229-253`), in the order the decomp probes them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TgAt {
    Arrow,
    LightArrow,
    Hookshot,
    Boomerang,
}

impl TgAt {
    fn power(self) -> i32 {
        match self {
            Self::LightArrow => 4,
            Self::Arrow | Self::Hookshot => 2,
            Self::Boomerang => 1,
        }
    }

    fn sound(self) -> u32 {
        match self {
            Self::Boomerang => SE_LK_W_WEP_HIT,
            _ => SE_LK_MS_WEP_HIT,
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct DaioEye {
    pub appear: bool,
    pub mode: Mode,
    pub health: i32,
    pub dead: bool,
    pub damaged: bool,
    pub damaged_bomb: bool,
    pub eye_scale: f32,
    pub scale_idx: i32,
}

impl DaioEye {
    pub fn spawn() -> Self {
        Self {
            appear: false,
            mode: Mode::Wait,
            health: MAX_HEALTH,
            dead: false,
            damaged: false,
            damaged_bomb: false,
            eye_scale: 1.0,
            scale_idx: 0,
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        self.appear
    }

    /// `_coHit` (`d_a_daiocta_eye.cpp:184-202`): the Co callback only fires for a
    /// `daBomb_c` in `STATE_4`, and `Blasts` is the sim's bomb-blast channel.
    fn co_hit(&mut self, events: &mut SimEvents) {
        if self.dead {
            self.damaged_bomb = true;
            return;
        }
        self.health = 0;
        self.dead = true;
        self.mode_death_init(events);
    }

    /// `modeDamageInit` (`d_a_daiocta_eye.cpp:288-308`) minus the BRK/BTK re-init.
    fn mode_damage_init(&mut self, events: &mut SimEvents) {
        self.mode = Mode::Damage;
        self.scale_idx = SCALE_DAMAGE_TIME;
        self.damaged = true;
        events.list.push(SimEvent::Se(SE_CV_DO_DAMAGE));
    }

    /// `modeDeathInit` (`d_a_daiocta_eye.cpp:335-352`) minus the BRK/BTK re-init.
    fn mode_death_init(&mut self, events: &mut SimEvents) {
        self.mode = Mode::Death;
        self.damaged = true;
        self.eye_scale = 1.0;
        events.list.push(SimEvent::Se(SE_CV_DO_EYE_LOSE));
    }

    /// `checkTgHit` (`d_a_daiocta_eye.cpp:222-275`).
    fn check_tg_hit(&mut self, at: Option<TgAt>, events: &mut SimEvents) {
        if self.dead {
            return;
        }
        let Some(at) = at else {
            return;
        };
        events.list.push(SimEvent::Se(at.sound()));
        self.health -= at.power();
        if self.health <= 0 {
            events.list.push(SimEvent::Se(SE_LK_LAST_HIT));
            self.health = 0;
            self.mode_death_init(events);
        } else {
            self.mode_damage_init(events);
        }
    }

    /// `modeWait` (`d_a_daiocta_eye.cpp:283-285`).
    fn mode_wait(&mut self, at: Option<TgAt>, events: &mut SimEvents) {
        self.check_tg_hit(at, events);
    }

    /// `modeDamage` (`d_a_daiocta_eye.cpp:311-332`) - `dLib_scaleAnime`
    /// (`d_lib.cpp:178-191`) walks `scale_table` backwards once the index runs off the end.
    fn mode_damage(&mut self) {
        if self.scale_idx < SCALE_TABLE.len() as i32 {
            let target = SCALE_TABLE
                .get(self.scale_idx as usize)
                .copied()
                .unwrap_or(1.0);
            if add_calc(&mut self.eye_scale, target) == 0.0 {
                self.scale_idx -= 1;
                if self.scale_idx < 0 {
                    self.scale_idx = 0;
                }
            }
            return;
        }
        self.scale_idx -= 1;
    }

    /// `modeDeath` (`d_a_daiocta_eye.cpp:355-357`).
    fn mode_death(&mut self) {
        self.dead = true;
    }

    fn mode_proc(&mut self, at: Option<TgAt>, events: &mut SimEvents) {
        match self.mode {
            Mode::Wait => self.mode_wait(at, events),
            Mode::Damage => self.mode_damage(),
            Mode::Death => self.mode_death(),
        }
    }

    /// `_execute` (`d_a_daiocta_eye.cpp:370-427`) minus the particle, matrix, HIO copy and
    /// attention work.
    fn execute(
        &mut self,
        actor: &Actor,
        at: Option<TgAt>,
        blasts: &Blasts,
        events: &mut SimEvents,
    ) {
        if !self.appear {
            return;
        }
        self.damaged = false;
        self.damaged_bomb = false;
        if blasts
            .list
            .iter()
            .any(|origin| walk::within(*origin, actor.pos, SPH_RADIUS))
        {
            self.co_hit(events);
        }
        self.mode_proc(at, events);
    }
}

/// `cLib_addCalc` (`c_lib.cpp:22-53`): returns `|target - value|`, zero on arrival.
fn add_calc(value: &mut f32, target: f32) -> f32 {
    if *value != target {
        let mut step = SCALE_STEP_SCALE * (target - *value);
        if step >= SCALE_STEP_MIN || step <= -SCALE_STEP_MIN {
            if step > SCALE_STEP_MAX {
                step = SCALE_STEP_MAX;
            }
            if step < -SCALE_STEP_MAX {
                step = -SCALE_STEP_MAX;
            }
            *value += step;
        } else if step > 0.0 {
            if step < SCALE_STEP_MIN {
                *value += SCALE_STEP_MIN;
                if *value > target {
                    *value = target;
                }
            }
        } else {
            let back = -SCALE_STEP_MIN;
            if step > back {
                *value += back;
                if *value < target {
                    *value = target;
                }
            }
        }
    }
    (target - *value).abs()
}

/// `mSph.GetTgHitObj()` hands back one object per tick, so the sim collapses the live AT
/// emitters the way `switem::source_at` does (`src/sim/switem.rs:154-190`), keeping
/// `checkTgHit`'s probe order.
fn source_at(
    shafts: &Query<&arrow::Arrow>,
    chains: &Query<&hookshot::Hookshot>,
    bolts: &Query<&boomerang::Boomerang>,
) -> Option<TgAt> {
    for shaft in shafts.iter() {
        if shaft.phase != arrow::Phase::Flying {
            continue;
        }
        return Some(match shaft.kind {
            arrow::TYPE_LIGHT => TgAt::LightArrow,
            _ => TgAt::Arrow,
        });
    }
    if chains
        .iter()
        .any(|chain| chain.phase != hookshot::Phase::Wait)
    {
        return Some(TgAt::Hookshot);
    }
    bolts
        .iter()
        .any(|bolt| bolt.parameter == boomerang::MODE_THROW)
        .then_some(TgAt::Boomerang)
}

pub fn is_daioeye(name: &str) -> bool {
    DAIOEYE_ACTORS.contains(&name)
}

pub fn control(
    blasts: Res<Blasts>,
    shafts: Query<&arrow::Arrow>,
    chains: Query<&hookshot::Hookshot>,
    bolts: Query<&boomerang::Boomerang>,
    mut events: ResMut<SimEvents>,
    mut eyes: Query<(&Actor, &mut DaioEye)>,
) {
    for (actor, mut eye) in &mut eyes {
        eye.execute(
            actor,
            source_at(&shafts, &chains, &bolts),
            &blasts,
            &mut events,
        );
    }
}
