//! `J3DFrameCtrl` (`J3DAnimation.h:824-872`, `J3DAnimation.cpp:13-215`): the frame cursor the
//! `mDoExt_*Anm` wrappers own (`m_Do_ext.h:21-47`) and each animation reads through its own
//! `getFrame`.
//!
//! The cursor's attribute decides what an advance does at the ends. `createHeap` inits every demo
//! pass with `EMode_NULL` (`d_a_demo00.cpp:173-221`), which is minus one and matches no arm of
//! [`FrameCtrl::update`]'s switch, so that cursor adds its rate forever and never clamps; the
//! sampling functions inside each animation are what clamp at the last key. A staged id carrying
//! `DEMO_SELECT(0x10000, 0x10000000)` inits the same pass with `EMode_LOOP` instead
//! (`d_a_demo00.cpp:359`, `:386`), which wraps the cursor back into `[mLoop, mEnd)`.

/// `J3DFrameCtrl::Attribute_e` - `J3DAnimation.h:826-833`. The reference stores the attribute in a
/// `u8`, so its minus-one `EMode_NULL` is `0xFF` here.
pub const MODE_NULL: u8 = u8::MAX;
pub const MODE_NONE: u8 = 0;
pub const MODE_RESET: u8 = 1;
pub const MODE_LOOP: u8 = 2;
pub const MODE_REVERSE: u8 = 3;
pub const MODE_LOOP_REVERSE: u8 = 4;

/// `J3DFrameCtrl::State_e` - `J3DAnimation.h:835-838`.
pub const STATE_STOP: u8 = 1;
pub const STATE_LOOP: u8 = 2;

/// `J3DFrameCtrl::mEnd - 0.001f` - `J3DAnimation.cpp:153`, the frame a stopped cursor parks on.
const END_EPSILON: f32 = 0.001;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameCtrl {
    attribute: u8,
    state: u8,
    start: i16,
    end: i16,
    loop_frame: i16,
    rate: f32,
    frame: f32,
}

impl Default for FrameCtrl {
    fn default() -> Self {
        Self::init(0)
    }
}

impl FrameCtrl {
    /// `J3DFrameCtrl::init` - `J3DAnimation.cpp:13-21`: the constructor's attribute is
    /// `EMode_LOOP`, the cursor starts at zero and the rate at one.
    pub fn init(end: i16) -> Self {
        Self {
            attribute: MODE_LOOP,
            state: 0,
            start: 0,
            end,
            loop_frame: 0,
            rate: 1.0,
            frame: 0.0,
        }
    }

    /// `mDoExt_baseAnm::initPlay` (`m_Do_ext.cpp:85-114`) for the arguments every `mDoExt_*Anm`
    /// user in this port passes: start `0`, end from the animation's own frame count, rate `1.0`
    /// and no end override. The reference writes the start, end, attribute, rate, frame and loop
    /// frame around its `J3DFrameCtrl::init(frameMax)` in that order; with a zero start and a
    /// rate of one, what survives is the end, the attribute and a cursor at zero.
    pub fn init_play(frame_max: u16, attribute: u8) -> Self {
        let mut ctrl = Self::init(frame_max as i16);
        ctrl.attribute = attribute;
        ctrl.frame = 0.0;
        ctrl.loop_frame = 0;
        ctrl
    }

    pub fn attribute(&self) -> u8 {
        self.attribute
    }

    pub fn state(&self) -> u8 {
        self.state
    }

    pub fn frame(&self) -> f32 {
        self.frame
    }

    pub fn set_frame(&mut self, frame: f32) {
        self.frame = frame;
    }

    /// `mDoExt_baseAnm::isStop` - `m_Do_ext.h:40-42`.
    pub fn is_stop(&self) -> bool {
        self.state & STATE_STOP != 0 || self.rate == 0.0
    }

    /// `mDoExt_baseAnm::isLoop` - `m_Do_ext.h:43`.
    pub fn is_loop(&self) -> bool {
        self.state & STATE_LOOP != 0
    }

    /// `J3DFrameCtrl::update` - `J3DAnimation.cpp:143-215`.
    pub fn update(&mut self) {
        self.state = 0;
        self.frame += self.rate;
        let start = f32::from(self.start);
        let end = f32::from(self.end);
        let loop_frame = f32::from(self.loop_frame);
        match self.attribute {
            MODE_NONE => {
                if self.frame < start {
                    self.frame = start;
                    self.rate = 0.0;
                    self.state |= STATE_STOP;
                }
                if self.frame >= end {
                    self.frame = end - END_EPSILON;
                    self.rate = 0.0;
                    self.state |= STATE_STOP;
                }
            }
            MODE_RESET => {
                if self.frame < start {
                    self.frame = start;
                    self.rate = 0.0;
                    self.state |= STATE_STOP;
                }
                if self.frame >= end {
                    self.frame = start;
                    self.rate = 0.0;
                    self.state |= STATE_STOP;
                }
            }
            MODE_LOOP => {
                while self.frame < start {
                    self.state |= STATE_LOOP;
                    if loop_frame - start <= 0.0 {
                        break;
                    }
                    self.frame += loop_frame - start;
                }
                while self.frame >= end {
                    self.state |= STATE_LOOP;
                    if end - loop_frame <= 0.0 {
                        break;
                    }
                    self.frame -= end - loop_frame;
                }
            }
            MODE_REVERSE => {
                if self.frame >= end {
                    self.frame = end - END_EPSILON;
                    self.rate = -self.rate;
                }
                if self.frame < start {
                    self.frame = start;
                    self.rate = 0.0;
                    self.state |= STATE_STOP;
                }
            }
            MODE_LOOP_REVERSE => {
                if self.frame >= end {
                    self.frame = end - END_EPSILON;
                    self.rate = -self.rate;
                }
                if self.frame < start {
                    self.frame = start;
                    self.rate = -self.rate;
                    self.state |= STATE_LOOP;
                }
            }
            // `EMode_NULL` (minus one) is no case of the reference's switch, so the frame keeps
            // the rate it added above and neither state bit is raised.
            _ => {}
        }
    }
}
