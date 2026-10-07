use bevy_ecs::prelude::*;

/// `J3DFrameCtrl::Attribute_e`, `J3DAnimation.h:830-837`: what the cursor does at the two ends of
/// its range. `EMode_NULL` is absent because the field it is written into is a `u8`
/// (`J3DAnimation.h:869`) and `-1` does not fit it, so the reference's own constructor at
/// `J3DAnimation.cpp:14` is the only way that value is ever named.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// `EMode_NONE`, `J3DAnimation.h:832`: one shot, and the rate is killed at whichever end is
    /// reached first, which is what every `getRate() < 0.01f` test at the end of a state waits on.
    None,
    /// `EMode_RESET`, `J3DAnimation.h:833`: one shot that returns to the first frame instead of
    /// stopping on it.
    Reset,
    /// `EMode_LOOP`, `J3DAnimation.h:834`: wraps between the loop point and the last frame.
    Loop,
    /// `EMode_REVERSE`, `J3DAnimation.h:835`: turns over at the last frame.
    Reverse,
    /// `EMode_LOOP_REVERSE`, `J3DAnimation.h:836`: turns over at the last frame and at the first.
    LoopReverse,
}

/// `J3DFrameCtrl::State_e`, `J3DAnimation.h:839-842`: the bits `update` leaves on the cursor for
/// the one tick that reached an end.
pub const STATE_STOP: u8 = 0x1;
pub const STATE_LOOP: u8 = 0x2;

/// `J3DFrameCtrl`, `J3DAnimation.h:828-876`. The cursor itself is a current frame and a signed rate
/// in frames per frame; `start`, `end` and the loop point are the span the shape's own frame count
/// bounds it by. `init` is `J3DAnimation.cpp:13-21`, `checkPass` is `:24-140` and `update` is
/// `:143-210` — all three are the reference's own translation, not a reading of a call site, so the
/// wrap at the frame limit is settled rather than guessed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameCtrl {
    attribute: Mode,
    state: u8,
    start: i16,
    end: i16,
    loop_point: i16,
    rate: f32,
    frame: f32,
}

impl Default for FrameCtrl {
    /// `J3DFrameCtrl::init(0)`, `J3DAnimation.cpp:13-21`, which the default constructor at
    /// `J3DAnimation.h:844` calls: a cursor already looping over nothing at a rate of one.
    fn default() -> Self {
        Self {
            attribute: Mode::Loop,
            state: 0,
            start: 0,
            end: 0,
            loop_point: 0,
            rate: 1.0,
            frame: 0.0,
        }
    }
}

impl FrameCtrl {
    /// `getAttribute`, `J3DAnimation.h:850`.
    pub fn attribute(&self) -> Mode {
        self.attribute
    }

    /// `setAttribute`, `J3DAnimation.h:851`.
    pub fn set_attribute(&mut self, attribute: Mode) {
        self.attribute = attribute;
    }

    /// `getState`, `J3DAnimation.h:852`: the bits the last `update` left.
    pub fn state(&self) -> u8 {
        self.state
    }

    /// `checkState`, `J3DAnimation.h:853`.
    pub fn check_state(&self, state: u8) -> bool {
        self.state & state != 0
    }

    /// `getStart`, `J3DAnimation.h:854`.
    pub fn start(&self) -> i16 {
        self.start
    }

    /// `setStart`, `J3DAnimation.h:855-858`, which writes the frame as well, so a cursor moved to a
    /// new first frame reads that frame until its next `update`.
    pub fn set_start(&mut self, start: i16) {
        self.start = start;
        self.frame = f32::from(start);
    }

    /// `getEnd`, `J3DAnimation.h:859`.
    pub fn end(&self) -> i16 {
        self.end
    }

    /// `setEnd`, `J3DAnimation.h:860`. The frame count the animation's own `getFrameMax` supplies
    /// through `setSingleMoveAnime`, `main.cpp:12791-12795`.
    pub fn set_end(&mut self, end: i16) {
        self.end = end;
    }

    /// `getLoop`, `J3DAnimation.h:861`.
    pub fn loop_point(&self) -> i16 {
        self.loop_point
    }

    /// `setLoop`, `J3DAnimation.h:862`.
    pub fn set_loop_point(&mut self, loop_point: i16) {
        self.loop_point = loop_point;
    }

    /// `getRate`, `J3DAnimation.h:863`: frames per frame, signed. A negative rate is the same
    /// animation run backwards, which is why a state that reads the sign at `crawl.inc:710` and
    /// `hang.inc:574` is asking whether the animation has run out rather than which way it went.
    pub fn rate(&self) -> f32 {
        self.rate
    }

    /// `setRate`, `J3DAnimation.h:864`.
    pub fn set_rate(&mut self, rate: f32) {
        self.rate = rate;
    }

    /// `getFrame`, `J3DAnimation.h:865`.
    pub fn frame(&self) -> f32 {
        self.frame
    }

    /// `setFrame`, `J3DAnimation.h:866`.
    pub fn set_frame(&mut self, frame: f32) {
        self.frame = frame;
    }

    /// `setFrameCtrl`, `main.cpp:2938-2949`: the whole cursor in one call, with the loop point put
    /// on the end the rate runs away from.
    pub fn set_frame_ctrl(
        &mut self,
        attribute: Mode,
        start: i16,
        end: i16,
        rate: f32,
        frame: f32,
    ) {
        self.attribute = attribute;
        self.end = end;
        self.rate = rate;
        self.start = start;
        self.frame = frame;
        self.loop_point = if rate >= 0.0 { start } else { end };
    }

    /// `checkPass`, `J3DAnimation.cpp:24-140`: whether the step the cursor is about to take crosses
    /// `pass_frame`, wrapping the way this attribute wraps. It reads the cursor without moving it,
    /// which is what lets a state ask "has the animation got past frame seventeen yet" on the tick
    /// before it has.
    pub fn check_pass(&self, pass_frame: f32) -> bool {
        let cur = self.frame;
        let start = f32::from(self.start);
        let end = f32::from(self.end);
        let loop_point = f32::from(self.loop_point);
        match self.attribute {
            Mode::None | Mode::Reset => {
                let mut next = cur + self.rate;
                if next < start {
                    next = start;
                }
                if next >= end {
                    next = end - 0.001;
                }
                if cur <= next {
                    return cur <= pass_frame && pass_frame < next;
                }
                next <= pass_frame && pass_frame < cur
            }
            Mode::Loop => {
                let mut next = cur + self.rate;
                if cur < start {
                    while next < start {
                        if loop_point - start <= 0.0 {
                            break;
                        }
                        next += loop_point - start;
                    }
                    next <= pass_frame && pass_frame < loop_point
                } else if end <= cur {
                    while next >= end {
                        if end - loop_point <= 0.0 {
                            break;
                        }
                        next -= end - loop_point;
                    }
                    loop_point <= pass_frame && pass_frame < next
                } else if next < start {
                    while next < start {
                        if loop_point - start <= 0.0 {
                            break;
                        }
                        next += loop_point - start;
                    }
                    (start <= pass_frame && pass_frame < cur)
                        || (next <= pass_frame && pass_frame < loop_point)
                } else if end <= next {
                    while next >= end {
                        if end - loop_point <= 0.0 {
                            break;
                        }
                        next -= end - loop_point;
                    }
                    (cur <= pass_frame && pass_frame < end)
                        || (loop_point <= pass_frame && pass_frame < next)
                } else if cur <= next {
                    cur <= pass_frame && pass_frame < next
                } else {
                    next <= pass_frame && pass_frame < cur
                }
            }
            Mode::Reverse | Mode::LoopReverse => {
                let mut next = cur + self.rate;
                if next >= end {
                    next = end - 0.001;
                }
                if next < start {
                    next = start;
                }
                if cur <= next {
                    return cur <= pass_frame && pass_frame < next;
                }
                next <= pass_frame && pass_frame < cur
            }
        }
    }

    /// `update`, `J3DAnimation.cpp:143-210`: the one step a frame takes, then the end the cursor
    /// has run into and what happens to the rate there. A one-shot that reaches an end stops, which
    /// is the `getRate() < 0.01f` a state at `main.cpp:6762` and `hang.inc:574` is waiting on; a
    /// looping one folds the frame back over its loop point and leaves the rate alone.
    pub fn update(&mut self) {
        self.state = 0;
        self.frame += self.rate;
        let start = f32::from(self.start);
        let end = f32::from(self.end);
        let loop_point = f32::from(self.loop_point);
        match self.attribute {
            Mode::None => {
                if self.frame < start {
                    self.frame = start;
                    self.rate = 0.0;
                    self.state |= STATE_STOP;
                }
                if self.frame >= end {
                    self.frame = end - 0.001;
                    self.rate = 0.0;
                    self.state |= STATE_STOP;
                }
            }
            Mode::Reset => {
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
            Mode::Loop => {
                while self.frame < start {
                    self.state |= STATE_LOOP;
                    if loop_point - start <= 0.0 {
                        break;
                    }
                    self.frame += loop_point - start;
                }
                while self.frame >= end {
                    self.state |= STATE_LOOP;
                    if end - loop_point <= 0.0 {
                        break;
                    }
                    self.frame -= end - loop_point;
                }
            }
            Mode::Reverse => {
                if self.frame >= end {
                    self.frame = end - 0.001;
                    self.rate = -self.rate;
                }
                if self.frame < start {
                    self.frame = start;
                    self.rate = 0.0;
                    self.state |= STATE_STOP;
                }
            }
            Mode::LoopReverse => {
                if self.frame >= end {
                    self.frame = end - 0.001;
                    self.rate = -self.rate;
                }
                if self.frame < start {
                    self.frame = start;
                    self.rate = -self.rate;
                    self.state |= STATE_LOOP;
                }
            }
        }
    }
}

/// The under slot enum, `main.h:879-882`: the two under-body cursors. `Move0` is the one every
/// locomotion state reads; `Move1` carries the second under animation the sword, hammer and boat
/// families blend in, and is stepped beside it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Under {
    /// `UNDER_MOVE0_e`, `main.h:880`.
    Move0 = 0,
    /// `UNDER_MOVE1_e`, `main.h:881`.
    Move1 = 1,
}

/// `mFrameCtrlUnder`, `main.h:2081`: the player's two under-body cursors. The under bodies are the
/// only half of the animation the locomotion states are timed on — the upper half
/// `mFrameCtrlUpper` at `main.h:2082` is never read by a state this port owns.
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct FrameClocks {
    under: [FrameCtrl; 2],
}

impl FrameClocks {
    /// The cursor at `slot`.
    pub fn under(&self, slot: Under) -> &FrameCtrl {
        &self.under[slot as usize]
    }

    /// The cursor at `slot`, for a state writing it.
    pub fn under_mut(&mut self, slot: Under) -> &mut FrameCtrl {
        &mut self.under[slot as usize]
    }

    /// `getBaseAnimeFrameRate`, `main.h:1972`: the under-body cursor's rate, which is the face
    /// every `getRate() < 0.01f` test in the player reads through.
    pub fn base_rate(&self) -> f32 {
        self.under(Under::Move0).rate()
    }

    /// `getBaseAnimeFrame`, `main.h:1973`: the under-body cursor's frame, the face every
    /// `getFrame() > ...` exit test in the player reads through.
    pub fn base_frame(&self) -> f32 {
        self.under(Under::Move0).frame()
    }
}

/// `animeUpdate`, `main.cpp:12871-12907`: every cursor the player holds takes its one step for the
/// frame. The reference runs this at the tail of its own execute, `main.cpp:11324`, after the
/// process dispatch has read the clock for this frame — which is what `checkPass` is written to
/// answer for, since it reports on the step that has not happened yet. The reference steps a slot
/// only when the animation pack behind it is present and is not the same object as another slot's,
/// which a port that carries no packs cannot ask, so both under slots step every tick.
///
/// Hit-stop reaches this the way it reaches every other tick: `Pause::advance`, `pause.rs:67`,
/// stops the fixed step whole, so a held frame takes no step here either.
/// `daPy_py_c::animeUpdate` runs its frame cursors once per execute, after the process
/// dispatch has read them for the tick's exits (`main.cpp:11324`).
pub fn init(world: &mut World) {
    world.init_resource::<FrameClocks>();
}

pub fn advance(mut clocks: ResMut<FrameClocks>) {
    for slot in [Under::Move0, Under::Move1] {
        clocks.under_mut(slot).update();
    }
}