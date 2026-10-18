use bevy_ecs::prelude::*;

use super::actor::Actor;
use super::events::{SimEvent, SimEvents};
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
/// `daPy_lk_c::setTactZev`'s `seStartSystem(JA_SE_PRE_TAKT)` (`d_a_player_tact.inc:124`),
/// reached whenever `fopAcM_GetID(this) != fpcM_ERROR_PROCESS_ID_e` (`:121`), which a live
/// actor always satisfies.
const SE_PRE_TAKT: u32 = 0x08A7; // <- JAZelAudio_SE.h:176 (JA_SE_PRE_TAKT)
const RADIUS_MARK: f32 = 100.0; // <- d_a_swtact.cpp:56
const RADIUS_FLOOR: f32 = 50.0; // <- d_a_swtact.cpp:58 (mDefaultR)
const ANSWER_ANY: u8 = 0xFF; // <- d_a_swtact.cpp:108, :154
const CULL_MIN: [f32; 3] = [-120.0, -10.0, -120.0]; // <- d_a_swtact.cpp:45
const CULL_MAX: [f32; 3] = [120.0, 10.0, 120.0]; // <- d_a_swtact.cpp:46
/// `daPy_lk_c::getTactMusic` returns -1 unless Link is in `daPyProc_TACT_PLAY_e` or
/// `daPyProc_TACT_WAIT_e` (`d_a_player_tact.inc:150-156`). Neither player process is ported, so
/// this is the proven value - the same constant `kanban.rs:95` uses for the same call.
const TACT_NONE: i32 = -1;

/// A Wind's Requiem answer plate. `getR()` is `mRadius * scale.x` (`d_a_swtact.h:10`) and
/// stage-placed actors are scale 1.0 (RULE 2), so the field is the radius as used.
/// `CreateInit` (`d_a_swtact.cpp:43-60`) only calls `fopAcM_setCullSizeBox` for the marked
/// variant (`:55`), and `_draw` (`:174-175`) only draws it, so `cull()` returns the marked box
/// and the floor variant has none. `_create` (`:72-91`) has no branch that refuses creation,
/// and `IsDelete` (`:204-207`) is the bare "accepts a delete request" form of RULE 6b.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Swtact {
    /// `mSwitchNo` (`d_a_swtact.h:30`), the switch `fopAcM_onSwitch` (`:162`) turns on.
    pub switch_no: u8,
    /// `mAnswer` (`d_a_swtact.h:31`), the raw answer byte.
    pub answer: u8,
    /// `daSwTact_prm::getModel(this) == 1`: the marked, drawn variant.
    pub mark: bool,
    /// `mRadius` (`d_a_swtact.h:29`).
    pub radius: f32,
    /// `mTrigger` (`d_a_swtact.h:32`).
    pub trigger: bool,
    /// `mPlayerStatus` (`d_a_swtact.h:33`).
    pub player_status: u8,
}

impl Swtact {
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

    pub fn cull(&self) -> Option<([f32; 3], [f32; 3])> {
        if self.mark {
            Some((CULL_MIN, CULL_MAX))
        } else {
            None
        }
    }

    /// The `switch (mAnswer)` of `_execute` (`d_a_swtact.cpp:129-158`): which song the plate is
    /// listening for, against `getTactMusic()`. Answers outside 0..=5 and `0xFF` fall through
    /// the decomp's switch and trigger nothing.
    fn listening(&self, music: i32) -> bool {
        match self.answer {
            0 => music == 0,
            1 => music == 1,
            2 => music == 5,
            3 => music == 2,
            4 => music == 3,
            5 => music == 4,
            ANSWER_ANY => matches!(music, 0..=5),
            _ => false,
        }
    }

    /// `_execute` (`d_a_swtact.cpp:115-170`) minus `set_mtx` (`:63-69`, model scale and
    /// position) and minus `getAnswer()` (`:94-112`).
    ///
    /// `setTactZev` (`d_a_player_tact.inc:117-129`) keeps its `seStartSystem(JA_SE_PRE_TAKT)`
    /// (`:121-125`) but drops the three stores it makes after it (`:126-128`,
    /// `mTactZevPartnerId` / `m34CC` / `m3494`): that is the player's tact-partner state, which
    /// this sim does not model, and `getAnswer()` only ever reached the sound's argument list, so
    /// the permutation table has no reader here. `dist` is `cXyz::absXZ` (`c_xyz.h:126`), the
    /// XZ-only distance.
    fn step(
        &mut self,
        actor: &Actor,
        player: Option<&Player>,
        status: &PlayerStatus,
        switches: &mut Switches,
        info: &MsgGameInfo,
        events: &mut SimEvents,
    ) {
        let Some(player) = player else {
            return;
        };
        if status.check0(STTS0_SHIP_RIDE) {
            return;
        }
        let stts1 = u8::from(status.check1(STTS1_WIND_WAKER_CONDUCT));
        let offset = player.pos - actor.pos;
        if (offset.x * offset.x + offset.z * offset.z).sqrt() <= self.radius {
            if self.player_status != stts1 && stts1 != 0 {
                events.list.push(SimEvent::Se(SE_PRE_TAKT));
            }
            if self.listening(TACT_NONE) {
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
    mut events: ResMut<SimEvents>,
    players: Query<&Player>,
    mut plates: Query<(&Actor, &mut Swtact)>,
) {
    let player = players.iter().next().copied();
    for (actor, mut plate) in &mut plates {
        plate.step(
            actor,
            player.as_ref(),
            &status,
            &mut switches,
            &info,
            &mut events,
        );
    }
}
