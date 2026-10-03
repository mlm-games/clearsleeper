use std::collections::HashSet;

use glam::Vec2;
use repose_core::input::PhysicalKey;

/// Held keys → normalized move axis (+x right, +y up). Focus loss zeroes it.
pub fn axis_from_keys(held: &HashSet<PhysicalKey>, window_focused: bool) -> Vec2 {
    if !window_focused {
        return Vec2::ZERO;
    }
    let mut axis = Vec2::ZERO;
    if held.contains(&PhysicalKey::KeyW) || held.contains(&PhysicalKey::ArrowUp) {
        axis.y += 1.0;
    }
    if held.contains(&PhysicalKey::KeyS) || held.contains(&PhysicalKey::ArrowDown) {
        axis.y -= 1.0;
    }
    if held.contains(&PhysicalKey::KeyA) || held.contains(&PhysicalKey::ArrowLeft) {
        axis.x -= 1.0;
    }
    if held.contains(&PhysicalKey::KeyD) || held.contains(&PhysicalKey::ArrowRight) {
        axis.x += 1.0;
    }
    axis.clamp_length_max(1.0)
}

/// Held debug key → quarter-heart damage staged this frame. Focus loss clears.
pub fn damage_from_keys(held: &HashSet<PhysicalKey>, window_focused: bool) -> u8 {
    u8::from(window_focused && held.contains(&PhysicalKey::KeyK))
}

pub fn do_from_keys(held: &HashSet<PhysicalKey>, window_focused: bool) -> bool {
    window_focused && held.contains(&PhysicalKey::Space)
}

pub fn target_from_keys(held: &HashSet<PhysicalKey>, window_focused: bool) -> bool {
    window_focused && held.contains(&PhysicalKey::KeyL)
}
