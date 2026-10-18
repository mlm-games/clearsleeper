use bevy_ecs::prelude::*;

pub const KYTAG03_ACTORS: [&str; 1] = ["ky_tag3"];

/// The stage string is `ky_tag3` (`d_stage.cpp:662`); the model's archive is `M_DOOR`
/// (`d_a_kytag03.cpp:16`). `daKytag03_Execute` (`:50-112`) is gated on
/// `dComIfGp_event_runCheck()` combined with
/// `dComIfGp_event_chkEventFlag(dEvtFlag_STAFF_ALL_e)` (`:52`); the second half needs
/// `dEvt_control_c::mEventFlag` (`d_event.h:128`), which only `afterFlagProc` writes
/// (`d_event.cpp:120-121`) and which this sim does not keep. Without that branch the model
/// never becomes visible (`_draw` is gated on `mbVisible`, `:32`) and `mSwitchNo` is read
/// nowhere else, so the term has no observable behaviour left.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Kytag03;

impl Kytag03 {
    pub fn spawn() -> Self {
        Self
    }
}

pub fn is_kytag03(name: &str) -> bool {
    KYTAG03_ACTORS.contains(&name)
}

pub fn control(_tags: Query<&Kytag03>) {}
