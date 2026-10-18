use bevy_ecs::prelude::*;

pub const GNNBTAKI_ACTORS: [&str; 1] = ["Gnbtaki"];

/// Ganondorf-tower background waterfall. `daObjGnnbtaki_c::_execute`
/// (`d_a_obj_gnnbtltaki.cpp:76-91`) only unfreezes the BTK when a demo is playing, which
/// needs the absent demo-playback system, so it reduces to `mBtkAnm.play()` — a BTK
/// texture animation the sim has no frames for. The `_arcname` is `Gnnbtltaki`
/// (`d_a_obj_gnnbtltaki.cpp:9`); the stage object name `Gnbtaki`
/// (`tww/src/d/d_stage.cpp:1186`) is what dispatches.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gnbtaki;

impl Gnbtaki {
    pub fn spawn() -> Self {
        Self
    }
}

pub fn is_gnbtaki(name: &str) -> bool {
    GNNBTAKI_ACTORS.contains(&name)
}

pub fn control(_falls: Query<&Gnbtaki>) {}
