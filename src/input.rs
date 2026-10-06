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

pub fn item_from_keys(held: &HashSet<PhysicalKey>, window_focused: bool) -> bool {
    window_focused && held.contains(&PhysicalKey::KeyB)
}

pub fn use_from_keys(held: &HashSet<PhysicalKey>, window_focused: bool) -> bool {
    window_focused && held.contains(&PhysicalKey::KeyJ)
}

/// The face buttons' levels for one poll, the shape of the pad's hold word: one flag per
/// button, cleared wholesale when the window loses focus.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ButtonBits {
    pub a: bool,
    pub b: bool,
    pub x: bool,
    pub y: bool,
    pub z: bool,
    pub l: bool,
    pub r: bool,
    pub start: bool,
}

impl ButtonBits {
    /// Every button but START released. A menu owns the screen, and START is both the edge
    /// that opens one and the edge that closes it, so it stays readable through a menu.
    pub fn except_start(self) -> Self {
        Self {
            start: self.start,
            ..Self::default()
        }
    }
}

/// The pad's bindings. The main stick is [`axis_from_keys`]'s WASD and arrows; each face
/// button takes the free key nearest its own letter — `KeyA` is the stick's left, `KeyB` the
/// item button, `KeyL` the target button — so A moves to `KeyQ` and L to `KeyO` while X, Y,
/// Z and R keep theirs.
pub const PAD_A: PhysicalKey = PhysicalKey::KeyQ;
pub const PAD_B: PhysicalKey = PhysicalKey::KeyV;
pub const PAD_X: PhysicalKey = PhysicalKey::KeyX;
pub const PAD_Y: PhysicalKey = PhysicalKey::KeyY;
pub const PAD_Z: PhysicalKey = PhysicalKey::KeyZ;
pub const PAD_L: PhysicalKey = PhysicalKey::KeyO;
pub const PAD_R: PhysicalKey = PhysicalKey::KeyR;

/// START is the desktop pause key, and the menus already read `Escape` as Back, which is
/// retail's other edge of the same button: one key opens a pause menu and closes it again.
pub const PAD_START: PhysicalKey = PhysicalKey::Escape;

/// The face buttons held right now, `CPad_CHECK_HOLD_*` over the bindings above.
pub fn pad_held_from_keys(held: &HashSet<PhysicalKey>, window_focused: bool) -> ButtonBits {
    let down = |key| window_focused && held.contains(&key);
    ButtonBits {
        a: down(PAD_A),
        b: down(PAD_B),
        x: down(PAD_X),
        y: down(PAD_Y),
        z: down(PAD_Z),
        l: down(PAD_L),
        r: down(PAD_R),
        start: down(PAD_START),
    }
}

/// The face buttons that went down since the last poll, `CPad_CHECK_TRIG_*`. The edge lives
/// in the poll rather than in a level, so it is the two samples that make it: a trigger is
/// true for exactly the frame its button changed from up to down.
pub fn pad_trig(now: ButtonBits, prev: ButtonBits) -> ButtonBits {
    ButtonBits {
        a: now.a && !prev.a,
        b: now.b && !prev.b,
        x: now.x && !prev.x,
        y: now.y && !prev.y,
        z: now.z && !prev.z,
        l: now.l && !prev.l,
        r: now.r && !prev.r,
        start: now.start && !prev.start,
    }
}

/// The C-stick on the keypad's directional cluster: 8/4/6/2 for the four directions and
/// 7/9/1/3 for the diagonals, the free keys laid out like a stick. Retail's sub-stick
/// carries no deadzone of its own, so neither does this — the consumer gates on the value.
pub fn c_stick_from_keys(held: &HashSet<PhysicalKey>, window_focused: bool) -> Vec2 {
    if !window_focused {
        return Vec2::ZERO;
    }
    let mut axis = Vec2::ZERO;
    if held.contains(&PhysicalKey::Numpad8) {
        axis.y += 1.0;
    }
    if held.contains(&PhysicalKey::Numpad2) {
        axis.y -= 1.0;
    }
    if held.contains(&PhysicalKey::Numpad4) {
        axis.x -= 1.0;
    }
    if held.contains(&PhysicalKey::Numpad6) {
        axis.x += 1.0;
    }
    if held.contains(&PhysicalKey::Numpad7) {
        axis += Vec2::new(-1.0, 1.0);
    }
    if held.contains(&PhysicalKey::Numpad9) {
        axis += Vec2::new(1.0, 1.0);
    }
    if held.contains(&PhysicalKey::Numpad1) {
        axis += Vec2::new(-1.0, -1.0);
    }
    if held.contains(&PhysicalKey::Numpad3) {
        axis += Vec2::new(1.0, -1.0);
    }
    axis.clamp_length_max(1.0)
}
