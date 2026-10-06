use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::flags::Switches;
use super::player::Player;
use super::player_status::{PlayerStatus, STTS0_SHIP_RIDE, STTS1_WIND_WAKER_CONDUCT};
use crate::text::msgproc::MsgGameInfo;

/// The stage string is `SWtact` (`d_stage.cpp:635`); `_arcname` is `"Itact"`
/// (`d_a_swtact.cpp:16`), which is why the STAGE strings `SWtact` and `SWtactB`
/// (`d_stage.cpp:636`) are what this array carries - the archive name is never a stage name here.
pub const SWTACT_ACTORS: [&str; 2] = ["SWtact", "SWtactB"];

const SWITCH_SHIFT: u32 = 0x00; // <- d_a_swtact.h:37 (daSwTact_prm::getSwitchNo)
const SWITCH_MASK: u32 = 0xFF; // <- d_a_swtact.h:37
const ANSWER_SHIFT: u32 = 0x08; // <- d_a_swtact.h:38 (daSwTact_prm::getAnswer)
const ANSWER_MASK: u32 = 0xFF; // <- d_a_swtact.h:38
const MODEL_SHIFT: u32 = 0x10; // <- d_a_swtact.h:39 (daSwTact_prm::getModel)
const MODEL_MASK: u32 = 0x0F; // <- d_a_swtact.h:39
const MODEL_MARK: u8 = 1; // <- d_a_swtact.cpp:51, :76, :174
const RADIUS_MARK: f32 = 100.0; // <- d_a_swtact.cpp:56
const RADIUS_FLOOR: f32 = 50.0; // <- d_a_swtact.cpp:58 (mDefaultR)
const ANSWER_ANY: u8 = 0xFF; // <- d_a_swtact.cpp:108, :154
const CULL_MIN: [f32; 3] = [-120.0, -10.0, -120.0]; // <- d_a_swtact.cpp:45
const CULL_MAX: [f32; 3] = [120.0, 10.0, 120.0]; // <- d_a_swtact.cpp:46
/// `daPy_lk_c::getTactMusic` returns -1 unless Link is in `daPyProc_TACT_PLAY_e` or
/// `daPyProc_TACT_WAIT_e` (`d_a_player_tact.inc:150-158`); neither player process is ported, so
/// this is the proven value, the same constant `kanban.rs:95` uses.
const TACT_NONE: i32 = -1;

/// A Wind's Requiem answer plate. `mRadius` is `mRadius * scale.x` at use time
/// (`d_a_swtact.h:10`) and stage-placed actors are scale 1.0 (RULE 2), so `getR()` is the field.
/// `_draw` (`:173-181`) returns early for the floor variant, and so does the cull box: only
/// `getModel() == 1` terms call `fopAcM_setCullSizeBox` (`:55`), the floor variant has none.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Swtact {
    /// `mSwitchNo` (`d_a_swtact.h:30`), the switch `fopAcM_onSwitch` turns on.
    pub switch_no: u8,
    /// `mAnswer` (`d_a_swtact.h:31`), the raw answer byte.
    pub answer: u8,
    /// `getModel(this) == 1`: the marked variant.
    pub mark: bool,
    /// `mRadius` (`d_a_swtact.h:29`).
    pub radius: f32,
    /// `mTrigger` (`d_a_swtact.h:32`).
    pub trigger: bool,
    /// `mPlayerStatus` (`d_a_swtact.h:33`).
    pub player_status: u8,
}

impl Swtact {
    /// `CreateInit` (`d_a_swtact.cpp:43-60`): the marked variant takes `mRadius = 100.0f` (`:56`)
    /// and a scaled cull box, everything else `mRadius = 50.0f` (`:58`). `_create` (`:72-91`)
    /// only loads `m_arcname` for the marked variant and has no branch that refuses creation.
    pub fn spawn(parameters: u32) -> Self {
        let mark = ((parameters >> MODEL_SHIFT) & MODEL_MASK) as u8 == MODEL_MARK;
        Self {
            switch_no: ((parameters >> SWITCH_SHIFT) & SWITCH_MASK) as u8,
            answer: ((parameters >> ANSWER_SHIFT) & ANSWER_MASK) as u8,
            mark,
            radius: if mark { RADIUS_MARK } else { RADIUS_FLOOR },
            trigger: false,
            player_status: 0,
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `getAnswer()` (`:94-112`), the permuted table the tact handoff is handed.
    fn answer_of(&self) -> i32 {
        match self.answer {
            0 => 0,
            1 => 1,
            2 => 5,
            3 => 2,
            4 => 3,
            5 => 4,
            _ => -1,
        }
    }

    /// The `switch (mAnswer)` of `_execute` (`:129-158`): which song the plate is listening for,
    /// against `getTactMusic()`. Cases outside 0..=5 and 0xFF fall through and trigger nothing.
    fn listening(&self) -> bool {
        match self.answer {
            0 => TACT_NONE == 0,
            1 => TACT_NONE == 1,
            2 => TACT_NONE == 5,
            3 => TACT_NONE == 2,
            4 => TACT_NONE == 3,
            5 => TACT_NONE == 4,
            ANSWER_ANY => matches!(TACT_NONE, 0..=5),
            _ => false,
        }
    }

    /// `_execute` (`:115-170`) minus `set_mtx` (`:63-69`, model scale and position). The
    /// `player->setTactZev(fopAcM_GetID(this), getAnswer(), NULL)` handoff (`:125`) writes
    /// `daPy_lk_c::mTactZevPartnerId`/`m34CC` (`d_a_player_tact.inc:117-127`), player state this
    /// sim does not keep, so only the arm's guard survives.
    fn step(
        &mut self,
        actor: &Actor,
        player: Option<&Player>,
        status: &PlayerStatus,
        switches: &mut Switches,
        info: &MsgGameInfo,
    ) {
        let Some(player) = player else {
            return;
        };
        if status.check0(STTS0_SHIP_RIDE) {
            return;
        }
        let stts1 = u8::from(status.check1(STTS1_WIND_WAKER_CONDUCT));
        let offset = player.pos - actor.pos;
        let dist = (offset.x * offset.x + offset.z * offset.z).sqrt();
        if dist <= self.radius {
            if self.player_status != stts1 && stts1 != 0 {
                let _ = self.answer_of();
            }
            if self.listening() {
                self.trigger = true;
            }
        }
        if self.trigger && !info.event_run_check() {
            switches.set(self.switch_no);
            self.trigger = false;
        }
        self.player_status = stts1;
    }
}

pub fn is_swtact(name: &str) -> bool {
    SWTACT_ACTORS.contains(&name)
}

pub fn control(
    status: Res<PlayerStatus>,
    info: Res<MsgGameInfo>,
    mut switches: ResMut<Switches>,
    players: Query<&Player>,
    mut plates: Query<(&Actor, &mut Swtact)>,
) {
    let player = players.iter().next().copied();
    for (actor, mut plate) in &mut plates {
        plate.step(actor, player.as_ref(), &status, &mut switches, &info);
    }
}

/// `cXyz::absXZ` (`c_xyz.h:126`) is `sqrtf(dx^2 + dz^2)`, kept nameable so the plate's reach
/// test reads the way the decomp does.
#[allow(dead_code)]
fn abs_xz(delta: Vec3) -> f32 {
    (delta.x * delta.x + delta.z * delta.z).sqrt()
}