use bevy_ecs::prelude::*;
use glam::Vec3;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Player {
    pub pos: Vec3,
    pub yaw: f32,
    pub face: f32,
    pub speed: f32,
    pub speed_y: f32,
    pub drift: Vec3,
    /// `mNoResetFlg0`'s `daPyFlg0_NO_DRAW` (`d_a_player.h:191`, `0x08000000`), the
    /// flag `onNoResetFlg0`/`offNoResetFlg0` (`:529-530`) latch and the player itself
    /// never clears. `eInit_ACTOR_DRW_CONTROL_` (`d_a_npc_ba1.cpp:971-974`) is one of
    /// its writers; the shell's view is the reader that has yet to be filled in.
    pub no_draw: bool,
}
