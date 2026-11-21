use std::collections::HashSet;

use glam::Vec2;
use repose_core::input::{GamepadAxis, GamepadButton, GamepadEvent, PhysicalKey};

/// The keys the shell reads, in the order the input log numbers them. The log writes the
/// held set as one bit per key, so a logged session replays through the same key path
/// the live session took.
pub const SHELL_KEYS: [PhysicalKey; 47] = [
    PhysicalKey::KeyA,
    PhysicalKey::KeyB,
    PhysicalKey::KeyC,
    PhysicalKey::KeyD,
    PhysicalKey::KeyE,
    PhysicalKey::KeyF,
    PhysicalKey::KeyG,
    PhysicalKey::KeyH,
    PhysicalKey::KeyI,
    PhysicalKey::KeyJ,
    PhysicalKey::KeyK,
    PhysicalKey::KeyL,
    PhysicalKey::KeyM,
    PhysicalKey::KeyN,
    PhysicalKey::KeyO,
    PhysicalKey::KeyP,
    PhysicalKey::KeyQ,
    PhysicalKey::KeyR,
    PhysicalKey::KeyS,
    PhysicalKey::KeyT,
    PhysicalKey::KeyU,
    PhysicalKey::KeyV,
    PhysicalKey::KeyW,
    PhysicalKey::KeyX,
    PhysicalKey::KeyY,
    PhysicalKey::KeyZ,
    PhysicalKey::Space,
    PhysicalKey::Enter,
    PhysicalKey::Escape,
    PhysicalKey::ArrowUp,
    PhysicalKey::ArrowDown,
    PhysicalKey::ArrowLeft,
    PhysicalKey::ArrowRight,
    PhysicalKey::Numpad0,
    PhysicalKey::Numpad1,
    PhysicalKey::Numpad2,
    PhysicalKey::Numpad3,
    PhysicalKey::Numpad4,
    PhysicalKey::Numpad5,
    PhysicalKey::Numpad6,
    PhysicalKey::Numpad7,
    PhysicalKey::Numpad8,
    PhysicalKey::Numpad9,
    PhysicalKey::ShiftLeft,
    PhysicalKey::ControlLeft,
    PhysicalKey::Tab,
    PhysicalKey::Backspace,
];

/// The held keys as the log's bit mask.
pub fn key_mask(held: &HashSet<PhysicalKey>) -> u64 {
    let mut mask = 0_u64;
    for (bit, key) in SHELL_KEYS.iter().enumerate() {
        if held.contains(key) {
            mask |= 1 << bit;
        }
    }
    mask
}

/// The keys a log line's bit mask names.
pub fn keys_from_mask(mask: u64) -> HashSet<PhysicalKey> {
    let mut held = HashSet::new();
    for (bit, key) in SHELL_KEYS.iter().enumerate() {
        if mask & (1 << bit) != 0 {
            held.insert(*key);
        }
    }
    held
}

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
    pub fn union(self, other: Self) -> Self {
        Self {
            a: self.a || other.a,
            b: self.b || other.b,
            x: self.x || other.x,
            y: self.y || other.y,
            z: self.z || other.z,
            l: self.l || other.l,
            r: self.r || other.r,
            start: self.start || other.start,
        }
    }

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

/// The trigger level that counts as the shoulder button down, and the level it has to
/// fall under to count as released. Retail locks a trigger once it crosses the high
/// threshold and only frees it under the low one, so a trigger held at the threshold
/// does not chatter (`m_Do_controller_pad.cpp:104-132`).
pub const TRIGGER_ON: f32 = 0.5;
pub const TRIGGER_OFF: f32 = 0.25;

/// One poll of the pad, in the retail pad's shape: the face buttons' levels, the main
/// stick and the sub-stick.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PadSnapshot {
    pub hold: ButtonBits,
    pub move_axis: Vec2,
    pub c_axis: Vec2,
}

/// Folds hardware gamepad events into the retail pad's shape. The button mapping is
/// aurora's standard one (`aurora/lib/dolphin/pad/pad.cpp:21-35`): the face buttons
/// carry A, B, X and Y, the right shoulder carries Z, and L and R are the analog
/// triggers, which is what the retail pad has.
#[derive(Debug, Clone, Copy, Default)]
pub struct PadBridge {
    hold: ButtonBits,
    move_axis: Vec2,
    c_axis: Vec2,
    dpad_up: bool,
    dpad_down: bool,
    dpad_left: bool,
    dpad_right: bool,
}

impl PadBridge {
    pub fn feed(&mut self, event: &GamepadEvent) {
        match event {
            GamepadEvent::Button {
                button, pressed, ..
            } => {
                let down = *pressed;
                match button {
                    GamepadButton::South => self.hold.a = down,
                    GamepadButton::East => self.hold.b = down,
                    GamepadButton::West => self.hold.x = down,
                    GamepadButton::North => self.hold.y = down,
                    GamepadButton::RightShoulder => self.hold.z = down,
                    GamepadButton::Start => self.hold.start = down,
                    GamepadButton::DPadUp => self.dpad_up = down,
                    GamepadButton::DPadDown => self.dpad_down = down,
                    GamepadButton::DPadLeft => self.dpad_left = down,
                    GamepadButton::DPadRight => self.dpad_right = down,
                    _ => {}
                }
            }
            GamepadEvent::Axis { axis, value, .. } => match axis {
                GamepadAxis::LeftStickX => self.move_axis.x = *value,
                GamepadAxis::LeftStickY => self.move_axis.y = *value,
                GamepadAxis::RightStickX => self.c_axis.x = *value,
                GamepadAxis::RightStickY => self.c_axis.y = *value,
                GamepadAxis::LeftTrigger => {
                    self.hold.l = Self::trigger(self.hold.l, *value);
                }
                GamepadAxis::RightTrigger => {
                    self.hold.r = Self::trigger(self.hold.r, *value);
                }
            },
            _ => {}
        }
    }

    fn trigger(held: bool, value: f32) -> bool {
        if held {
            value > TRIGGER_OFF
        } else {
            value >= TRIGGER_ON
        }
    }

    /// The poll's pad state. The D-pad rides the main stick: the retail pad reports the
    /// cross as a main-stick direction (`m_Do_controller_pad.cpp:47-49` reads the stick
    /// the hardware reports, and the cross lands in it).
    pub fn snapshot(&self) -> PadSnapshot {
        let mut move_axis = self.move_axis;
        if self.dpad_up {
            move_axis.y = 1.0;
        }
        if self.dpad_down {
            move_axis.y = -1.0;
        }
        if self.dpad_left {
            move_axis.x = -1.0;
        }
        if self.dpad_right {
            move_axis.x = 1.0;
        }
        PadSnapshot {
            hold: self.hold,
            move_axis: move_axis.clamp_length_max(1.0),
            c_axis: self.c_axis.clamp_length_max(1.0),
        }
    }
}

/// The pad state one frame stages: the hardware pad when it says something, the keys
/// otherwise. A button held on either source is held, and a stick that moved on one
/// source wins over the other's rest position.
pub fn merge_pad(keyboard: PadSnapshot, pad: PadSnapshot) -> PadSnapshot {
    PadSnapshot {
        hold: keyboard.hold.union(pad.hold),
        move_axis: if pad.move_axis != Vec2::ZERO {
            pad.move_axis
        } else {
            keyboard.move_axis
        },
        c_axis: if pad.c_axis != Vec2::ZERO {
            pad.c_axis
        } else {
            keyboard.c_axis
        },
    }
}

/// The pad state the keys alone give: the face buttons from the key bindings, the main
/// stick from WASD and the arrows, the sub-stick from the keypad's cluster.
pub fn pad_snapshot_from_keys(held: &HashSet<PhysicalKey>, window_focused: bool) -> PadSnapshot {
    PadSnapshot {
        hold: pad_held_from_keys(held, window_focused),
        move_axis: axis_from_keys(held, window_focused),
        c_axis: c_stick_from_keys(held, window_focused),
    }
}

/// The stylus: the mouse pointer over the canvas. The position is in the 640x480
/// canvas's own pixels, which is what the screens draw in, so a pointer over the
/// letterboxed canvas lands where the letter is.
pub fn stylus(
    canvas: [f32; 2],
    window: (u32, u32),
    pointer: Option<(f32, f32)>,
    mouse_down: bool,
) -> Option<(Vec2, bool)> {
    let (x, y) = pointer?;
    let pos = canvas_point_in(canvas, window, x, y)?;
    Some((pos, mouse_down))
}

/// A window position in the canvas's own pixels, or nothing when the position is
/// outside the letterboxed canvas.
pub fn canvas_point_in(canvas: [f32; 2], window: (u32, u32), x: f32, y: f32) -> Option<Vec2> {
    let (width, height) = (window.0 as f32, window.1 as f32);
    if width <= 0.0 || height <= 0.0 {
        return None;
    }
    let fit = (width / canvas[0]).min(height / canvas[1]);
    let offset_x = (width - canvas[0] * fit) * 0.5;
    let offset_y = (height - canvas[1] * fit) * 0.5;
    let pos = Vec2::new((x - offset_x) / fit, (y - offset_y) / fit);
    if pos.x < 0.0 || pos.y < 0.0 || pos.x > canvas[0] || pos.y > canvas[1] {
        return None;
    }
    Some(pos)
}

/// The retail screen's own pixels, for a window position.
pub fn canvas_point(window: (u32, u32), x: f32, y: f32) -> Option<Vec2> {
    canvas_point_in([640.0, 480.0], window, x, y)
}
