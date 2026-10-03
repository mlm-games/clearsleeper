use bevy_ecs::prelude::*;

/// One-shot presentation events emitted by the sim; the shell drains them
/// each frame. Not part of the saved state.
#[derive(Resource, Default, Debug)]
pub struct SimEvents {
    pub list: Vec<SimEvent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimEvent {
    Footstep,
    Slash,
    Hit,
    Explode,
    Collect,
    Hurt,
    GameOver,
    Restart,
    Credits,
    CreditsDone,
}

/// Walk distance accumulated toward the next footstep.
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct Stride(pub f32);

/// World units between footstep events at full speed.
pub const STRIDE_UNITS: f32 = 200.0;
