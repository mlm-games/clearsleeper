//! The retail pad tape: one controller state per retrace, replayed into the sim's intent.
//!
//! One file drives both sides of the retail comparison. BlueWake's host prints the pad state
//! it hands the guest under `BLUEWAKE_PAD_TRACE=1` (`runtime/host/src/main.c:4562`), and the
//! port reads the same lines back, so a run on either side sees the same input and the two
//! traces can be compared field by field.
//!
//! # The line
//!
//! One line per retrace, `\n` terminated, the pairs space separated:
//!
//! ```text
//! retrace=<N> button=0x<hex> stick=<x>,<y> c=<x>,<y> l=<L> r=<R>
//! ```
//!
//! - `retrace` is the frame the state belongs to. The port applies the line whose retrace is
//!   the tick it is about to run; a retrace with no line holds the state of the line before it,
//!   and a tick before the first line holds the released pad.
//! - `button` is the pad word the guest reads, the 16 bits `SPEC2_MakeStatus` copies out of
//!   the SI transfer (`dolphin/pad/Pad.c:593`): `A` `0x0100`, `B` `0x0200`, `X` `0x0400`,
//!   `Y` `0x0800`, `Z` `0x0010`, `L` `0x0040`, `R` `0x0020`, `START` `0x1000`, and the D-pad
//!   `0x0001` left, `0x0002` right, `0x0004` down, `0x0008` up (`dolphin/pad/Pad.h:33-45`).
//! - `stick` and `c` are the main stick and the C-stick as the guest's pad status holds them:
//!   the byte the pad sent minus 128, so `-128..127`, centred at 0 (`dolphin/pad/Pad.c:643-646`).
//!   The position the game reads off them is the unit disc, `-1..1` on each axis.
//! - `l` and `r` are the analog triggers as the pad sent them, `0..255`.
//!
//! A line may carry BlueWake's own `[pad-trace] ` prefix, and `#` starts a comment. Every
//! other key must be there: a line without one is a [`TapeError`], never a panic.
//!
//! # What the port honours
//!
//! - The face buttons and the triggers become the pad's hold word, and the press edge is the
//!   difference against the previous poll, the way `mDoCPd_Convert` splits the pad's hold and
//!   trigger words (`m_Do/m_Do_controller_pad.cpp:20-45`).
//! - The main stick becomes the move axis, through the pad's own clamp and scale
//!   ([`PadState::move_axis`]), so the axis is the position `CPad_GET_STICK_POS_X/Y` returns
//!   (`m_Do/m_Do_controller_pad.h:39-42`) and its length is `CPad_GET_STICK_VALUE`.
//! - The C-stick becomes the sub-stick axis ([`PadState::c_axis`]), the position
//!   `CPad_GET_SUBSTICK_POS_X/Y` returns (`m_Do/m_Do_controller_pad.h:44-47`).
//! - The analog triggers are read as the fraction the game reads them as
//!   ([`PadState::trigger_left`]), which the port's intent path has no consumer for.
//! - The D-pad is carried in the pad word, where `mDoCPd_Convert` puts it in the hold word
//!   (`m_Do/m_Do_controller_pad.cpp:21-24`), and mapped to nothing: retail's Wind Waker
//!   moves the player from the main stick alone, and no game code reads the D-pad for
//!   movement.

use crate::input::ButtonBits;
use glam::Vec2;

/// The D-pad left, `PAD_BUTTON_LEFT` (`dolphin/pad/Pad.h:33`).
pub const BUTTON_LEFT: u16 = 0x0001;
/// The D-pad right, `PAD_BUTTON_RIGHT` (`dolphin/pad/Pad.h:34`).
pub const BUTTON_RIGHT: u16 = 0x0002;
/// The D-pad down, `PAD_BUTTON_DOWN` (`dolphin/pad/Pad.h:35`).
pub const BUTTON_DOWN: u16 = 0x0004;
/// The D-pad up, `PAD_BUTTON_UP` (`dolphin/pad/Pad.h:36`).
pub const BUTTON_UP: u16 = 0x0008;
/// The Z trigger, `PAD_TRIGGER_Z` (`dolphin/pad/Pad.h:37`).
pub const BUTTON_Z: u16 = 0x0010;
/// The R trigger, `PAD_TRIGGER_R` (`dolphin/pad/Pad.h:38`).
pub const BUTTON_R: u16 = 0x0020;
/// The L trigger, `PAD_TRIGGER_L` (`dolphin/pad/Pad.h:39`).
pub const BUTTON_L: u16 = 0x0040;
/// The A button, `PAD_BUTTON_A` (`dolphin/pad/Pad.h:40`).
pub const BUTTON_A: u16 = 0x0100;
/// The B button, `PAD_BUTTON_B` (`dolphin/pad/Pad.h:41`).
pub const BUTTON_B: u16 = 0x0200;
/// The X button, `PAD_BUTTON_X` (`dolphin/pad/Pad.h:42`).
pub const BUTTON_X: u16 = 0x0400;
/// The Y button, `PAD_BUTTON_Y` (`dolphin/pad/Pad.h:43`).
pub const BUTTON_Y: u16 = 0x0800;
/// The start button, `PAD_BUTTON_START` (`dolphin/pad/Pad.h:45`).
pub const BUTTON_START: u16 = 0x1000;

/// The main stick's dead zone, `ClampRegion.minStick` (`dolphin/pad/Padclamp.c:10`).
const MAIN_MIN: i8 = 15;
/// The main stick's limit, `ClampRegion.maxStick` (`dolphin/pad/Padclamp.c:11`).
const MAIN_MAX: i8 = 72;
/// The main stick's diagonal, `ClampRegion.xyStick` (`dolphin/pad/Padclamp.c:12`).
const MAIN_XY: i8 = 40;
/// The main stick's scale, the `54` of `JUTGamePad::CStick::update`
/// (`JSystem/JUtility/JUTGamePad.cpp:303`).
const MAIN_SCALE: f32 = 54.0;
/// The C-stick's dead zone, `ClampRegion.minSubstick` (`dolphin/pad/Padclamp.c:13`).
const SUB_MIN: i8 = 15;
/// The C-stick's limit, `ClampRegion.maxSubstick` (`dolphin/pad/Padclamp.c:14`).
const SUB_MAX: i8 = 59;
/// The C-stick's diagonal, `ClampRegion.xySubstick` (`dolphin/pad/Padclamp.c:15`).
const SUB_XY: i8 = 31;
/// The C-stick's scale, the `42` of `JUTGamePad::CStick::update`
/// (`JSystem/JUtility/JUTGamePad.cpp:303`).
const SUB_SCALE: f32 = 42.0;
/// The trigger's full travel, the `140` of `mDoCPd_TRIGGER_CONV`
/// (`m_Do/m_Do_controller_pad.h:80-85`).
const TRIGGER_FULL: f32 = 140.0;

/// One controller state, the pad word and the four axes of one retrace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PadState {
    /// The pad word, the bits `dolphin/pad/Pad.h:33-45` names.
    pub buttons: u16,
    /// The shell's held keys as the input log's bit mask (`input::SHELL_KEYS`). A line that
    /// carries it replays through the key path the live session took.
    pub keys: u64,
    /// Whether the line carried the key mask at all, which separates a logged session from
    /// a pad tape: a logged frame with no key held still names its empty mask.
    pub keys_given: bool,
    /// The main stick's x, the centred byte the pad sent.
    pub stick_x: i8,
    /// The main stick's y, the centred byte the pad sent.
    pub stick_y: i8,
    /// The C-stick's x, the centred byte the pad sent.
    pub c_stick_x: i8,
    /// The C-stick's y, the centred byte the pad sent.
    pub c_stick_y: i8,
    /// The left trigger, the raw byte the pad sent.
    pub trigger_left: u8,
    /// The right trigger, the raw byte the pad sent.
    pub trigger_right: u8,
}

impl PadState {
    /// The pad word as the port's hold bits, `CPad_CHECK_HOLD_*`
    /// (`m_Do/m_Do_controller_pad.h:13-24`).
    pub fn hold(&self) -> ButtonBits {
        ButtonBits {
            a: self.buttons & BUTTON_A != 0,
            b: self.buttons & BUTTON_B != 0,
            x: self.buttons & BUTTON_X != 0,
            y: self.buttons & BUTTON_Y != 0,
            z: self.buttons & BUTTON_Z != 0,
            l: self.buttons & BUTTON_L != 0,
            r: self.buttons & BUTTON_R != 0,
            start: self.buttons & BUTTON_START != 0,
        }
    }

    /// The main stick as the game reads it: the position `CPad_GET_STICK_POS_X/Y` returns
    /// (`m_Do/m_Do_controller_pad.h:39-42`).
    pub fn move_axis(&self) -> Vec2 {
        stick_axis(
            self.stick_x,
            self.stick_y,
            MAIN_MAX,
            MAIN_XY,
            MAIN_MIN,
            MAIN_SCALE,
        )
    }

    /// The C-stick as the game reads it: the position `CPad_GET_SUBSTICK_POS_X/Y` returns
    /// (`m_Do/m_Do_controller_pad.h:44-47`).
    pub fn c_axis(&self) -> Vec2 {
        stick_axis(
            self.c_stick_x,
            self.c_stick_y,
            SUB_MAX,
            SUB_XY,
            SUB_MIN,
            SUB_SCALE,
        )
    }

    /// The left trigger as the game reads it: `mDoCPd_TRIGGER_CONV`
    /// (`m_Do/m_Do_controller_pad.h:80-85`) turns the raw byte into a fraction of the full
    /// travel, `analog / 140` clamped to 1.
    pub fn trigger_left(&self) -> f32 {
        trigger_fraction(self.trigger_left)
    }

    /// The right trigger as the game reads it, the same conversion as
    /// [`PadState::trigger_left`].
    pub fn trigger_right(&self) -> f32 {
        trigger_fraction(self.trigger_right)
    }
}

fn trigger_fraction(raw: u8) -> f32 {
    (f32::from(raw) / TRIGGER_FULL).min(1.0)
}

/// `ClampStick` (`dolphin/pad/Padclamp.c:28-81`): the dead zone under the stick, then the
/// rounded square the pad's calibration region allows.
fn clamp_stick(x: i8, y: i8, max: i8, xy: i8, min: i8) -> (i8, i8) {
    let sign_x = if x < 0 { -1 } else { 1 };
    let sign_y = if y < 0 { -1 } else { 1 };
    let mut ax = i32::from(x.unsigned_abs());
    let mut ay = i32::from(y.unsigned_abs());
    if ax <= i32::from(min) {
        ax = 0;
    } else {
        ax -= i32::from(min);
    }
    if ay <= i32::from(min) {
        ay = 0;
    } else {
        ay -= i32::from(min);
    }
    if ax == 0 && ay == 0 {
        return (0, 0);
    }
    let corner = i32::from(xy) * i32::from(max);
    if i32::from(xy) * ay <= i32::from(xy) * ax {
        let d = i32::from(xy) * ax + (i32::from(max) - i32::from(xy)) * ay;
        if corner < d {
            ax = corner * ax / d;
            ay = corner * ay / d;
        }
    } else {
        let d = i32::from(xy) * ay + (i32::from(max) - i32::from(xy)) * ax;
        if corner < d {
            ax = corner * ax / d;
            ay = corner * ay / d;
        }
    }
    ((sign_x * ax) as i8, (sign_y * ay) as i8)
}

/// `JUTGamePad::CStick::update` (`JSystem/JUtility/JUTGamePad.cpp:302-330`): the clamped
/// byte over the stick's scale, and the position pulled onto the unit circle when it is
/// longer than the stick's reach.
fn stick_axis(x: i8, y: i8, max: i8, xy: i8, min: i8, scale: f32) -> Vec2 {
    let (cx, cy) = clamp_stick(x, y, max, xy, min);
    let mut pos = Vec2::new(f32::from(cx) / scale, f32::from(cy) / scale);
    let value = pos.length();
    if value > 1.0 {
        pos /= value;
    }
    pos
}

/// A tape: the pad state of every retrace that carries one, in retrace order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InputTape {
    states: Vec<(u64, PadState)>,
}

impl InputTape {
    /// Read a whole tape. A line without a key, a key whose value does not parse and a
    /// retrace that is not a number are [`TapeError`]s; a line that is not a line at all
    /// (blank, a comment) is skipped.
    pub fn parse(text: &str) -> Result<Self, TapeError> {
        let mut states: Vec<(u64, PadState)> = Vec::new();
        for (at, line) in text.lines().enumerate() {
            let number = at + 1;
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let (retrace, state) = parse_line(trimmed, number)?;
            match states.binary_search_by_key(&retrace, |entry| entry.0) {
                Ok(index) => states[index] = (retrace, state),
                Err(index) => states.insert(index, (retrace, state)),
            }
        }
        Ok(Self { states })
    }

    /// The state a retrace runs with: the line of that retrace, or the line before it when
    /// this retrace carries none. A retrace before the first line runs with the pad
    /// released.
    pub fn state_at(&self, retrace: u64) -> PadState {
        let index = self.states.partition_point(|entry| entry.0 <= retrace);
        match index.checked_sub(1) {
            Some(last) => self.states[last].1,
            None => PadState::default(),
        }
    }

    /// The number of lines the tape carries.
    pub fn len(&self) -> usize {
        self.states.len()
    }

    /// Whether the tape carries no line at all.
    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }
}

/// Read one line: the pairs split on single spaces and each pair on its first `=`.
fn parse_line(line: &str, number: usize) -> Result<(u64, PadState), TapeError> {
    let text = line.strip_prefix("[pad-trace]").unwrap_or(line).trim();
    let mut retrace = None;
    let mut buttons = None;
    let mut stick = None;
    let mut c_stick = None;
    let mut trigger_left = None;
    let mut trigger_right = None;
    let mut keys = None;
    for token in text.split(' ') {
        if token.is_empty() {
            continue;
        }
        let Some((key, value)) = token.split_once('=') else {
            return Err(TapeError::BadPair {
                line: number,
                text: line.to_string(),
            });
        };
        match key {
            "retrace" => retrace = Some(parse_number(value, number, key)?),
            "button" => buttons = Some(parse_word(value, number, key)?),
            "stick" => stick = Some(parse_pair(value, number, key)?),
            "c" => c_stick = Some(parse_pair(value, number, key)?),
            "l" => trigger_left = Some(parse_byte(value, number, key)?),
            "r" => trigger_right = Some(parse_byte(value, number, key)?),
            "keys" => keys = Some(parse_word64(value, number, key)?),
            _ => {}
        }
    }
    let retrace = retrace.ok_or_else(|| TapeError::Missing {
        line: number,
        key: "retrace",
    })?;
    let buttons = buttons.ok_or_else(|| TapeError::Missing {
        line: number,
        key: "button",
    })?;
    let (stick_x, stick_y) = stick.ok_or_else(|| TapeError::Missing {
        line: number,
        key: "stick",
    })?;
    let (c_stick_x, c_stick_y) = c_stick.ok_or_else(|| TapeError::Missing {
        line: number,
        key: "c",
    })?;
    let trigger_left = trigger_left.ok_or_else(|| TapeError::Missing {
        line: number,
        key: "l",
    })?;
    let trigger_right = trigger_right.ok_or_else(|| TapeError::Missing {
        line: number,
        key: "r",
    })?;
    Ok((
        retrace,
        PadState {
            buttons,
            stick_x,
            stick_y,
            c_stick_x,
            c_stick_y,
            trigger_left,
            trigger_right,
            keys: keys.unwrap_or(0),
            keys_given: keys.is_some(),
        },
    ))
}

fn parse_number(value: &str, line: usize, key: &str) -> Result<u64, TapeError> {
    value.parse::<u64>().map_err(|_| TapeError::BadValue {
        line,
        key: key.to_string(),
        text: value.to_string(),
    })
}

fn parse_word64(value: &str, line: usize, key: &str) -> Result<u64, TapeError> {
    let digits = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
        .unwrap_or(value);
    u64::from_str_radix(digits, 16).map_err(|_| TapeError::BadValue {
        line,
        key: key.to_string(),
        text: value.to_string(),
    })
}

fn parse_word(value: &str, line: usize, key: &str) -> Result<u16, TapeError> {
    let digits = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"));
    let digits = digits.unwrap_or(value);
    u16::from_str_radix(digits, 16).map_err(|_| TapeError::BadValue {
        line,
        key: key.to_string(),
        text: value.to_string(),
    })
}

fn parse_byte(value: &str, line: usize, key: &str) -> Result<u8, TapeError> {
    value.parse::<u8>().map_err(|_| TapeError::BadValue {
        line,
        key: key.to_string(),
        text: value.to_string(),
    })
}

fn parse_pair(value: &str, line: usize, key: &str) -> Result<(i8, i8), TapeError> {
    let bad = || TapeError::BadValue {
        line,
        key: key.to_string(),
        text: value.to_string(),
    };
    let (left, right) = value.split_once(',').ok_or_else(bad)?;
    let x = left.parse::<i8>().map_err(|_| bad())?;
    let y = right.parse::<i8>().map_err(|_| bad())?;
    Ok((x, y))
}

/// What a tape line can be wrong about. Every failure is one of these; nothing panics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TapeError {
    /// A token without a `=`.
    BadPair { line: usize, text: String },
    /// A key whose value does not parse.
    BadValue {
        line: usize,
        key: String,
        text: String,
    },
    /// A line without a key the grammar needs.
    Missing { line: usize, key: &'static str },
}

impl std::fmt::Display for TapeError {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TapeError::BadPair { line, text } => {
                write!(out, "line {line} has no key=value pair: {text:?}")
            }
            TapeError::BadValue { line, key, text } => {
                write!(out, "line {line} has no value for {key}: {text:?}")
            }
            TapeError::Missing { line, key } => {
                write!(out, "line {line} has no {key} pair")
            }
        }
    }
}

impl std::error::Error for TapeError {}
