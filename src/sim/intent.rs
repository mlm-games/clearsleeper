use bevy_ecs::prelude::*;
use glam::Vec2;

pub use crate::input::ButtonBits;

#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct MoveIntent {
    pub axis: Vec2,
}

#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct CameraYaw(pub f32);

/// Quarter-heart damage staged per frame (0 clears), like [`MoveIntent`].
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct DamageIntent(pub u8);

#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct DoIntent(pub bool);

#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct TargetIntent(pub bool);

#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct ItemIntent(pub bool);

#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct UseIntent(pub bool);

/// The retail pad's face buttons, split the way the shell's pad interface splits them:
/// [`PadButtons::trig`] is the frame's press edge and [`PadButtons::hold`] the level still
/// down. The edge is taken while the pad is polled, where retail takes it in
/// `m_Do_controller_pad.cpp`'s read of the hardware trigger word, and not by a system inside
/// the step: the pad is read once per drawn frame while the fixed step may take several
/// ticks or none, so an edge taken inside a tick would repeat a swing once per tick of a
/// slow frame and drop the press of a frame the hit-stop gate skipped.
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct PadButtons {
    /// `CPad_CHECK_TRIG_*`: true only on the frame its button went down.
    pub trig: ButtonBits,
    /// `CPad_CHECK_HOLD_*`: true while its button is down.
    pub hold: ButtonBits,
}

/// The C-stick as an axis of its own, the shell's sub-stick, kept apart from
/// [`MoveIntent`] so a camera can read it without moving the player.
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct CStick {
    /// `CPad_GET_SUBSTICK_POS_X/Y`: +x right, +y up, clamped to the unit disc.
    pub axis: Vec2,
    /// `CPad_GET_SUBSTICK_ANGLE`: `atan2f(x, -y)` on the 0x10000 circle, through the same
    /// conversion the walk's angles use. It stays latched while the stick sits centred,
    /// because retail's `CStick::update` only writes the angle on a frame with travel.
    pub angle: i16,
}

impl CStick {
    /// Take one poll: `axis` is the stick's clamped vector and the angle its direction.
    pub fn poll(&mut self, axis: Vec2) {
        if axis != Vec2::ZERO {
            self.angle = super::walk::to_s16(axis.x.atan2(-axis.y));
        }
        self.axis = axis;
    }

    /// `CPad_GET_SUBSTICK_VALUE`: the stick's magnitude, the gate a consumer compares
    /// against before it trusts the angle.
    pub fn value(self) -> f32 {
        self.axis.length()
    }

    /// The latched angle as radians, for a consumer that carries yaw in floats.
    pub fn radians(self) -> f32 {
        super::walk::from_s16(self.angle)
    }
}

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameOverChoice {
    #[default]
    None,
    Continue,
    Quit,
}
