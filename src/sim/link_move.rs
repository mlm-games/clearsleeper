use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::{Vec2, Vec3};

use super::anim_clock::{FrameClocks, FrameCtrl, Mode, Under};
use super::collision::{Collision, MoveFrom, WallProbe, WallTrace};
use super::combat::{Combat, CutKind};
use super::ground::grounded;
use super::ground_code::{FrontWall, WallCode};
use super::health::Health;
use super::hop::Hop;
use super::intent::{CameraYaw, MoveIntent, PadButtons};
use super::jump::{Direction, Jump, JumpKind, direction};
use super::link_proc::{CUT_AIR, LinkProc, Proc};
use super::lock::Lock;
use super::player::Player;
use super::pushable::{
    self, PPLABEL_HEAVY, PPLABEL_NONE, PPLABEL_PULL, PPLABEL_PUSH, PPLABEL_UNK4, PushPulls,
};
use super::roll::Roll;
use super::ship::Ship;
use super::swim::Swim;
use super::walk::{add_calc, add_calc_angle_s, from_s16, target_yaw, to_s16};

/// `mStickDistance > 0.05f`, the gate every stick test in the reference opens with.
pub const STICK_GATE: f32 = 0.05;
/// `cLib_distanceAngleS(m34E8, current.angle.y) > 0x7800`, the reversal the turn states ask for.
pub const REVERSAL_LIMIT: u16 = 0x7800;
/// `std::abs(mNormalSpeed) <= 0.001f`, the still-speed test the wait and move turns open with.
pub const REST_SPEED: f32 = 0.001;
/// `mStickDistance > 0.5f` in the landing dispatcher, the stick hold a rolling landing asks for.
pub const LAND_STICK: f32 = 0.5;
/// `crawl.inc:311`, the ceiling a crawl needs above the player's feet before it will lie down.
pub const CRAWL_CLEARANCE: f32 = 125.0;
/// `main.cpp:4944`, the drop the landing dispatcher needs before it will roll off a ledge.
pub const ROLL_DROP: f32 = 300.0;
/// `mSlide.m.field_0x8`, `main.cpp:4900`: the turn under which `changeSlideProc` takes the front
/// slide and over which it takes the back one.
pub const SLIDE_PICK: u16 = 14336;
/// `mWallCatch.m.field_0x0C`, `:0x10`, `:0x14` and `:0x18`, the four bands `setFrontWallType`
/// sorts a wall catch into.
pub const CATCH_LOW: f32 = 75.0;
pub const CATCH_MID: f32 = 110.0;
pub const CATCH_HIGH: f32 = 130.0;
pub const CATCH_TOP: f32 = 170.0;
/// `mWall.m.field_0x50` and `:0x54`, the hidden wall's width and the depth its cross point is
/// pushed out by, `whide.inc:96` and `whide.inc:151`.
pub const WALL_HIDE_WIDTH: f32 = 8.5;
pub const WALL_HIDE_DEPTH: f32 = 25.0;
/// `mSwim.m.field_0x6C`, the band a ledge has to fall inside before the swim catch at
/// `main.cpp:4748` takes it.
pub const SWIM_LOW: f32 = 37.5;
/// `mFall.m.field_0x3C` and `mFall.m.field_0x40` and `mFall.m.field_0x44`, `main.cpp:6944` and
/// `main.cpp:6969`: the rate a side roll's own animation runs at, the frame it starts from and the
/// frame it ends past. The port has no animation reader, so the two frames and the rate give the
/// roll's own length in ticks, which is the one number the reference's roll end is made of.
pub const ROLL_RATE: f32 = 1.3;
pub const ROLL_START: f32 = 6.0;
pub const ROLL_END: f32 = 19.0;

/// `cLib_distanceAngleS` through a signed subtraction, `c_lib.cpp:364`: the turn from one facing
/// to another, which never folds past half a turn.
fn turn_between(a: i16, b: i16) -> u16 {
    i32::from(a.wrapping_sub(b) as i16).unsigned_abs() as u16
}

/// `cLib_addCalc(&mNormalSpeed, 0.0f, mMove.m.field_0x24, mMove.m.field_0x1C,
/// mMove.m.field_0x20)`, `main.cpp:6407`, `main.cpp:6524` and `main.cpp:6570`: the scale, the
/// largest step and the smallest step every settled process decays its own speed by.
const DECEL_SCALE: f32 = 0.6;
const DECEL_MAX_STEP: f32 = 2.5;
const DECEL_MIN_STEP: f32 = 1.8;
/// `cLib_addCalcAngleS(&shape_angle.y, mProcVar2.mTargetAngle, mTurn.m.field_0x4,
/// mTurn.m.field_0x0, mTurn.m.field_0x2)`, `main.cpp:6575`: the scale, the largest step and the
/// smallest step the wait turn runs on, and the three words the turn table carries.
const WAIT_TURN_SCALE: i16 = 30;
const WAIT_TURN_MAX: i16 = 0x3CDF;
const WAIT_TURN_MIN: i16 = 0x1F40;
/// `mMove.m.field_0x0`, `main.cpp:6604` and `main.cpp:6610`: the one value both `procMoveTurn_init`
/// arms scale their step and floor by.
const MOVE_TURN_WORD: i16 = 3000;
/// `main.cpp:6604`, the `param_1 != 0` arm's `(divisor, largest step, smallest step)`, which a
/// reversal past [`REVERSAL_LIMIT`] takes.
const MOVE_TURN_BACK_SCALE: i16 = 2;
const MOVE_TURN_BACK_MAX: i16 = MOVE_TURN_WORD * 4 + 0x4A56;
const MOVE_TURN_BACK_MIN: i16 = MOVE_TURN_WORD * 2;
/// `main.cpp:6610`, the `param_1 == 0` arm's, which a backward direction takes.
const MOVE_TURN_FACE_SCALE: i16 = 3;
const MOVE_TURN_FACE_MAX: i16 = MOVE_TURN_WORD * 2;
const MOVE_TURN_FACE_MIN: i16 = MOVE_TURN_WORD;
/// `cLib_addCalcAngleS(..., 4, 0x1000, 0x400)`, `main.cpp:6703` and `main.cpp:6737`: the scale, the
/// largest step and the smallest step both slides turn their body and their model with.
const SLIDE_TURN_SCALE: i16 = 4;
const SLIDE_TURN_MAX: i16 = 0x1000;
const SLIDE_TURN_MIN: i16 = 0x400;
/// `mSlide.m.field_0x44`, `main.cpp:6692`, the ceiling a slide holds its own speed at.
const SLIDE_SPEED_MAX: f32 = 30.0;
/// `mSlide.m.field_0x48`, `main.cpp:6705`, the constant the downhill push is scaled by.
const SLIDE_DRIVE: f32 = 3.0;
/// `cLib_addCalcAngleS(&shape_angle.y, m34E8, mCrouch.m.field_0x8, mCrouch.m.field_0xA,
/// mCrouch.m.field_0xC)`, `crawl.inc:479`: the scale, the largest step and the smallest step a
/// crawling player turns its model with.
const CRAWL_TURN_SCALE: i16 = 32;
const CRAWL_TURN_MAX: i16 = 7000;
const CRAWL_TURN_MIN: i16 = 500;
/// `cLib_addCalcAngleS(&shape_angle.y, mProcVar2.m34D4, 5, 0x1000, 0x800)`, `crawl.inc:461`: the
/// triple a crawl turns onto the corner wall it just found.
const CRAWL_WALL_SCALE: i16 = 5;
const CRAWL_WALL_MAX: i16 = 0x1000;
const CRAWL_WALL_MIN: i16 = 0x800;
/// `cLib_addCalc(&current.pos.x, m370C.x, 0.5f, 10.0f, 1.0f)`, `crawl.inc:466` and `crawl.inc:644`:
/// the corner walk's chase onto the point it picked.
const CRAWL_CHASE_SCALE: f32 = 0.5;
const CRAWL_CHASE_MAX_STEP: f32 = 10.0;
const CRAWL_CHASE_MIN_STEP: f32 = 1.0;
/// `crawl.inc:226`: the band a crawlable corner's two cross points have to sit in, and the turn the
/// two faces have to differ by.
const CRAWL_SIDE_REACH: f32 = 75.0;
const CRAWL_SIDE_MIN: f32 = 60.0;
const CRAWL_SIDE_TURN: u16 = 0x7F00;
/// `crawl.inc:100`, `crawl.inc:115` and `crawl.inc:123`: how far ahead, how far to one side and
/// how far past each corner trace runs.
const CRAWL_CORNER_REACH: f32 = 75.0;
const CRAWL_CORNER_SIDE: f32 = 50.0;
/// `l_crawl_side_offset`, `player_main_data.inc:37`, the joint the crawl looks round its own
/// shoulder with.
const CRAWL_SIDE_OFFSET: Vec3 = Vec3::new(50.0, 50.0, 0.0);
/// `l_crawl_front_offset`, `player_main_data.inc:29`, and `l_crawl_top_offset`,
/// `player_main_data.inc:31`: the two model offsets `crawlBgCheck` traces.
const CRAWL_FRONT_OFFSET: Vec3 = Vec3::new(0.0, 10.0, 80.0);
const CRAWL_TOP_OFFSET: Vec3 = Vec3::new(0.0, 10.0, 0.0);
/// `l_crawl_start_front_offset`, `player_main_data.inc:28`: the offset `procCrouch` measures the
/// lie-down slope against.
const CRAWL_START_OFFSET: Vec3 = Vec3::new(0.0, 10.0, 112.0);
/// `main.cpp:6540` and `main.cpp:6544`: the height the lie-down ground probe is lifted by, and the
/// slope past which the player stays crouched instead of lying down.
const CRAWL_SLOPE_UP: f32 = 5.0;
const CRAWL_SLOPE_LIMIT: u16 = 0x100;
/// `hang.inc:62`, `hang.inc:63` and `hang.inc:77`: how far ahead a hanging player reaches before it
/// will shuffle along a ledge, the height above the feet that reach starts from, and how far along
/// the wall's own facing the drop trace runs.
const HANG_REACH: f32 = 30.0;
const HANG_REACH_UP: f32 = 5.0;
const HANG_DROP_REACH: f32 = 90.0;
/// `hang.inc:85`, the turn under which the face a drop trace found is the ledge itself.
const HANG_SAME_WALL: u16 = 0x549F;
/// `hang.inc:184`, `hang.inc:189`, `hang.inc:197` and `hang.inc:202`: how far the hanging player
/// stands off the ledge, how far back `procHangStart` grounds under it, and the band that ground has
/// to fall inside.
const HANG_SETTLE: f32 = 1.5;
const HANG_SETTLE_UP: f32 = 10.0;
const HANG_GROUND_BAND: f32 = 30.1;
/// `hang.inc:90`, the offset the hang's own side traces are taken back by.
const HANG_SIDE_OFFSET: Vec3 = Vec3::new(4.5, 62.5, 4.5);
/// `hang.inc:109`, the stand-off `changeHangEndProc` takes its ground probe from.
const HANG_HAND_BACK: f32 = 4.0;
/// `l_toe_pos.y`, `player_main_data.inc:20`: the foot height `changeHangEndProc` compares the
/// ground above the player against.
const HANG_FOOT_UP: f32 = 3.25;
/// `hang.inc:45`, the ground under a hanging player has to be this level before it may shuffle.
const HANG_FLAT: f32 = 0.9986;
/// `climb.inc:112`, `climb.inc:114` and `climb.inc:131`: the height above the feet the climb's wall
/// trace starts from, how far along the model's own facing it runs, and the stand-off the climbing
/// player keeps off the face it found.
const CLIMB_REACH: f32 = 80.0;
const CLIMB_REACH_UP: f32 = 30.0;
const CLIMB_STANDOFF: f32 = 20.5;
/// `climb.inc:118`, the face a climb trace has to meet, and `climb.inc:127` the turn under which the
/// traced face is the wall the player is already on.
const CLIMB_FACE_FLAT: f32 = 0.05;
const CLIMB_SAME_WALL: u16 = 0x549F;
/// `climb.inc:120`, the ground height a climbing player may not come within this of.
const CLIMB_GROUND_BAND: f32 = 15.0;
/// `climb.inc:239`, the stand-off a climb starts from, `climb.inc:293` the reach a climb down steps
/// back by, and `climb.inc:294` the drop it takes off the feet.
const CLIMB_START_REACH: f32 = 25.0;
const CLIMB_DOWN_BACK: f32 = 35.0;
const CLIMB_DOWN_DROP: f32 = 20.0;
/// `mWallCatch.m.field_0x54`, `hang.inc:127`: the fall speed a hang lets go at.
const HANG_FALL_SPEED: f32 = 6.0;
/// `mFall.m.field_0x4C`, `main.cpp:6945`: the speed a side roll enters at.
const ROLL_SPEED: f32 = 26.0;
/// `main.cpp:10689`, the standing wall radius `setFrontWallType` traces from, and
/// `main.cpp:10654-10656` the three heights it traces at.
const WALL_RADIUS: f32 = 35.0;
const WALL_HEIGHTS: [f32; 3] = [30.1, 89.9, 125.0];
/// `main.cpp:4550`, the near reach of the wall trace before the player's own wall radius.
const WALL_TRACE_REACH: f32 = 25.0;
/// `pushpull.inc:118`: the scale, the largest step and the smallest step `procPushPullWait` turns
/// its model onto the grab point with.
const PUSH_PULL_SCALE: i16 = 3;
const PUSH_PULL_MAX: i16 = 0x800;
const PUSH_PULL_MIN: i16 = 0x100;
/// `pushpull.inc:121`, the chase onto the grab point's own stand-off, and `pushpull.inc:122` the
/// `5.0f` the two chases are summed against.
const PUSH_PULL_CHASE_SCALE: f32 = 0.5;
const PUSH_PULL_CHASE_MAX_STEP: f32 = 10.0;
const PUSH_PULL_CHASE_MIN_STEP: f32 = 1.0;
const PUSH_PULL_CHASE_LIMIT: f32 = 5.0;
/// `pushpull.inc:55`, how far off the ledge a grab point sits.
const PUSH_PULL_REACH: f32 = 40.0;
/// `pushpull.inc:169` and `pushpull.inc:172`: the two heights the pull's wall traces run at, how
/// far back they reach, and `pushpull.inc:178` the extra reach the ground probe takes.
const PULL_REACH: f32 = 105.0;
const PULL_PROBE_UP: [f32; 2] = [30.1, 89.9];
const PULL_GROUND_BACK: f32 = 29.0;
/// `pushpull.inc:180`, the drop past which the pull's ground probe is refused.
const PULL_GROUND_DROP: f32 = 30.1;
/// `setBgCheckParam`, `main.cpp:10664-10665`: the wall radius a player in a push or pull state
/// carries, which is the reach the front wall trace of `setFrontWallType` then runs.
const PUSH_PULL_WALL_R: f32 = 40.0;
/// `crawl.inc:528`, the counter `procCrawlAutoMove` holds on its corner for, and `crawl.inc:529`,
/// the ticks the corner walk gives up after.
const CRAWL_AUTO_HOLD: i32 = 20;
const CRAWL_AUTO_LIMIT: i32 = 300;

/// The lie animation `procCrawlEnd_init` stands the player up on, `crawl.inc:675-679`: played back
/// at `m_HIO->mCrouch.m.field_0x40` from `field_0x44` to `field_0xE`, so the stand-up runs the
/// lie-down in reverse and stops at the first frame. `procCrawlEnd`, `crawl.inc:712`, leaves once
/// the frame is back under `m_HIO->mCrouch.m.field_0x48`.
const CRAWL_END_RATE: f32 = -1.0;
const CRAWL_END_START: i16 = 0;
const CRAWL_END_LAST: i16 = 8;
const CRAWL_END_FRAME: f32 = 3.0;

/// The slide recovery animations `procSlideFrontLand_init` and `procSlideBackLand_init` play,
/// `main.cpp:6754` and `main.cpp:6780`: each runs from its own first frame over its own rate, and
/// each leaves once the frame is past `m_HIO->mSlide.m.field_0x2C` or `field_0x3C`
/// (`main.cpp:6766` and `main.cpp:6791`).
const SLIDE_FRONT_LAND_RATE: f32 = 0.8;
const SLIDE_FRONT_LAND_START: i16 = 2;
const SLIDE_FRONT_LAND_LAST: i16 = 9;
const SLIDE_FRONT_LAND_FRAME: f32 = 6.0;
const SLIDE_BACK_LAND_RATE: f32 = 1.0;
const SLIDE_BACK_LAND_START: i16 = 2;
const SLIDE_BACK_LAND_LAST: i16 = 8;
const SLIDE_BACK_LAND_FRAME: f32 = 6.0;

/// `main.cpp:6761` and `main.cpp:6786`: the speed both slide recoveries run down at.
const SLIDE_LAND_SCALE: f32 = 0.5;
const SLIDE_LAND_MAX_STEP: f32 = 5.0;
const SLIDE_LAND_MIN_STEP: f32 = 1.0;

/// The vault `procHangClimb_init` plays, `hang.inc:559-565`: it runs from the first frame of
/// `m_HIO->mWallCatch.m.field_0x2C` at `field_0x24` over to `field_0x4`, and `procHangClimb`,
/// `hang.inc:576`, leaves once the frame is past `m_HIO->mWallCatch.m.field_0x38`.
const HANG_CLIMB_RATE: f32 = 0.75;
const HANG_CLIMB_START: i16 = 0;
const HANG_CLIMB_LAST: i16 = 24;
const HANG_CLIMB_FRAME: f32 = 23.0;

/// What a state needs that the port cannot answer. Every guard below asks one question about the
/// tick and reports the one thing its state cannot be entered without, rather than standing in a
/// geometry that is not there.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveGap {
    /// The guard asked for the state and the port answered it, so the state is entered.
    Ready,
    /// The under-body frame cursor the reference's frame-rate and frame-position tests run on,
    /// which drives every transition out of a state that is otherwise pure collision. The player
    /// owns one at [`FrameClocks`], so a row still carrying this is a state whose own initialiser
    /// has not been written: the sword, hammer and boat families are the ones left.
    FrameClock,
    /// The damage vector off the player's own collision cylinder, which the guard slip takes its
    /// facing and its speed from.
    DamageVector,
    /// The crouch and guard face, which a world that stages no pad cannot report.
    CrouchButton,
}

impl MoveGap {
    /// The thing the port is missing, for a reader who has the reference open beside this.
    pub fn reason(self) -> &'static str {
        match self {
            Self::Ready => "nothing",
            Self::FrameClock => "the per-polygon frame clock",
            Self::DamageVector => "the damage vector off the player's cylinder",
            Self::CrouchButton => "the crouch and guard face",
        }
    }
}

/// One row per process this module owns: where the reference asks for it, where it leaves through,
/// and the gap that keeps the port from asking. The ladder family is the stage's own module and
/// has no row, and neither do the sword, hammer and boat families.
#[derive(Debug, Clone, Copy)]
pub struct MoveSite {
    /// The process the row describes.
    pub proc: Proc,
    /// Where the reference asks for it, by file and line.
    pub entry: &'static str,
    /// Where the reference leaves it, by file and line.
    pub exit: &'static str,
    /// What the port cannot answer for it.
    pub gap: MoveGap,
}

/// Every process with no module of its own, ordered by the sequence the reference tries them in:
/// the crouch and crawl pair first, because every other grounded state reaches its button test
/// through the crouch state, and the wall and vine selection last, because the reference runs it
/// after the action dispatch has had its turn.
pub const SITES: &[MoveSite] = &[
    MoveSite {
        proc: Proc::Crouch,
        entry: "main.cpp:4287",
        exit: "main.cpp:6529",
        gap: MoveGap::Ready,
    },
    MoveSite {
        proc: Proc::CrouchDefense,
        entry: "main.cpp:4294",
        exit: "main.cpp:6412",
        gap: MoveGap::Ready,
    },
    MoveSite {
        proc: Proc::CrouchDefenseSlip,
        entry: "main.cpp:5258",
        exit: "main.cpp:6496",
        gap: MoveGap::DamageVector,
    },
    MoveSite {
        proc: Proc::CrawlStart,
        entry: "main.cpp:6530",
        exit: "crawl.inc:296",
        gap: MoveGap::Ready,
    },
    MoveSite {
        proc: Proc::CrawlMove,
        entry: "crawl.inc:301",
        exit: "crawl.inc:400",
        gap: MoveGap::Ready,
    },
    MoveSite {
        proc: Proc::CrawlAutoMove,
        entry: "crawl.inc:473",
        exit: "crawl.inc:657",
        gap: MoveGap::Ready,
    },
    MoveSite {
        proc: Proc::CrawlEnd,
        entry: "crawl.inc:296",
        exit: "crawl.inc:712",
        gap: MoveGap::Ready,
    },
    MoveSite {
        proc: Proc::PushPullWait,
        entry: "main.cpp:4138",
        exit: "pushpull.inc:104",
        gap: MoveGap::Ready,
    },
    MoveSite {
        proc: Proc::PushMove,
        entry: "pushpull.inc:101",
        exit: "pushpull.inc:137",
        gap: MoveGap::Ready,
    },
    MoveSite {
        proc: Proc::PullMove,
        entry: "pushpull.inc:110",
        exit: "pushpull.inc:184",
        gap: MoveGap::Ready,
    },
    MoveSite {
        proc: Proc::SlideFront,
        entry: "main.cpp:4901",
        exit: "main.cpp:6712",
        gap: MoveGap::Ready,
    },
    MoveSite {
        proc: Proc::SlideBack,
        entry: "main.cpp:4903",
        exit: "main.cpp:6746",
        gap: MoveGap::Ready,
    },
    MoveSite {
        proc: Proc::SlideFrontLand,
        entry: "main.cpp:6712",
        exit: "main.cpp:6764",
        gap: MoveGap::Ready,
    },
    MoveSite {
        proc: Proc::SlideBackLand,
        entry: "main.cpp:6746",
        exit: "main.cpp:6789",
        gap: MoveGap::Ready,
    },
    MoveSite {
        proc: Proc::WaitTurn,
        entry: "main.cpp:4482",
        exit: "main.cpp:6586",
        gap: MoveGap::Ready,
    },
    MoveSite {
        proc: Proc::MoveTurn,
        entry: "main.cpp:4494",
        exit: "main.cpp:6621",
        gap: MoveGap::Ready,
    },
    MoveSite {
        proc: Proc::SideRoll,
        entry: "main.cpp:4952",
        exit: "main.cpp:6964",
        gap: MoveGap::Ready,
    },
    MoveSite {
        proc: Proc::HangStart,
        entry: "main.cpp:4783",
        exit: "hang.inc:259",
        gap: MoveGap::Ready,
    },
    MoveSite {
        proc: Proc::HangFallStart,
        entry: "main.cpp:5350",
        exit: "hang.inc:375",
        gap: MoveGap::Ready,
    },
    MoveSite {
        proc: Proc::HangUp,
        entry: "hang.inc:374",
        exit: "hang.inc:410",
        gap: MoveGap::Ready,
    },
    MoveSite {
        proc: Proc::HangWait,
        entry: "hang.inc:263",
        exit: "hang.inc:443",
        gap: MoveGap::Ready,
    },
    MoveSite {
        proc: Proc::HangMove,
        entry: "hang.inc:446",
        exit: "hang.inc:487",
        gap: MoveGap::Ready,
    },
    MoveSite {
        proc: Proc::HangClimb,
        entry: "hang.inc:259",
        exit: "hang.inc:576",
        gap: MoveGap::Ready,
    },
    MoveSite {
        proc: Proc::HangWallCatch,
        entry: "main.cpp:4875",
        exit: "hang.inc:620",
        gap: MoveGap::Ready,
    },
    MoveSite {
        proc: Proc::ClimbUpStart,
        entry: "main.cpp:4835",
        exit: "climb.inc:270",
        gap: MoveGap::Ready,
    },
    MoveSite {
        proc: Proc::ClimbDownStart,
        entry: "main.cpp:5346",
        exit: "climb.inc:315",
        gap: MoveGap::Ready,
    },
    MoveSite {
        proc: Proc::ClimbMoveUpDown,
        entry: "main.cpp:4789",
        exit: "climb.inc:381",
        gap: MoveGap::Ready,
    },
    MoveSite {
        proc: Proc::ClimbMoveSide,
        entry: "climb.inc:91",
        exit: "climb.inc:447",
        gap: MoveGap::Ready,
    },
];

/// The corner `procCrawlAutoMove` walks: which faces it found round it, the point it is walking to,
/// the yaw it is turning that point's wall to, and the two counters that hold it on the corner
/// before it takes the stick again. The reference keeps these in `mProcVar6.m3570`, `m370C`,
/// `mProcVar2.m34D4`, `m35A0`, `mProcVar0.m34D0` and `mProcVar3.m34D6`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CrawlCorner {
    /// `mProcVar6.m3570`, `crawl.inc:527`: which of the three walls round the corner it found.
    pub walls: u8,
    /// `m370C`, `crawl.inc:525`: the point it is walking to.
    pub target: Vec3,
    /// `mProcVar2.m34D4`, `crawl.inc:601`: the yaw the model's own facing is turned onto.
    pub wall_yaw: i16,
    /// `m35A0`, `crawl.inc:582`: which way the model turns onto that wall, `0.0f`, `1.0f` or
    /// `-1.0f`.
    pub turn: f32,
    /// `mProcVar0.m34D0`, `crawl.inc:528`: the counter, `-1` once the walk has started.
    pub hold: i32,
    /// `mProcVar3.m34D6`, `crawl.inc:529`: the ticks the walk gives up after.
    pub limit: i32,
}

impl CrawlCorner {
    /// `procCrawlAutoMove_init`, `crawl.inc:522-541`.
    pub fn of(walls: u8, target: Vec3, wall_yaw: i16, turn: f32) -> Self {
        Self {
            walls,
            target,
            wall_yaw,
            turn,
            hold: CRAWL_AUTO_HOLD,
            limit: CRAWL_AUTO_LIMIT,
        }
    }
}

/// What the owned processes are doing and what is stopping them: which process asked this tick, the
/// one thing the port could not answer for it, and the words the states latch on entry that the
/// reference keeps in its own per-process scratch.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct MoveState {
    /// The process whose guard or tick asked this tick, if any did.
    pub asked: Option<Proc>,
    /// What the port could not answer for `asked`.
    pub gap: MoveGap,
    /// `m3724`, `main.cpp:4584`: the ledge point the wall classification left, which the hang and
    /// climb states stand on and the push and pull states count their stand-off from.
    pub ledge: Option<Vec3>,
    /// `m352C`, `main.cpp:4576`: the wall's own facing, which every hang, climb and push and pull
    /// state takes its stand-off and its body yaw from.
    pub wall_yaw: i16,
    /// `m370C` of `procPushPullWait_init`, `pushpull.inc:57`: the point the player stands at while
    /// it waits for a grab, and from `setPushPullKeepData` on, `pushpull.inc:46`, the position
    /// the grabbed actor stood at the last time the label was handed to it.
    pub grab: Vec3,
    /// `mProcVar2.mPushPullPrevAngleY`, `pushpull.inc:34`: the grabbed actor's own facing the last
    /// time its callback ran, which the delta at `pushpull.inc:37` turns the player by.
    pub keep_yaw: i16,
    /// `mProcVar6.m3570`, `pushpull.inc:54`: the word that holds the grab alignment until it clears
    /// itself at `pushpull.inc:101`, raises the unknown bit of every label that carries one at
    /// `pushpull.inc:24`, and is raised again by each moving state's initialiser at `:130` and `:173`.
    pub armed: bool,
    /// The corner `procCrawlAutoMove` is walking, while it is walking one.
    pub corner: Option<CrawlCorner>,
}

impl Default for MoveState {
    fn default() -> Self {
        Self {
            asked: None,
            gap: MoveGap::Ready,
            ledge: None,
            wall_yaw: 0,
            grab: Vec3::ZERO,
            keep_yaw: 0,
            armed: false,
            corner: None,
        }
    }
}

impl MoveState {
    /// The row naming `proc`, if this module owns it.
    pub fn site(proc: Proc) -> Option<&'static MoveSite> {
        SITES.iter().find(|site| site.proc == proc)
    }

    /// Whether this module owns `proc`.
    pub fn owns(proc: Proc) -> bool {
        SITES.iter().any(|site| site.proc == proc)
    }

    /// The gap standing between `proc` and the port, or [`MoveGap::Ready`] for a process no row
    /// names.
    pub fn gap_of(proc: Proc) -> MoveGap {
        Self::site(proc).map_or(MoveGap::Ready, |site| site.gap)
    }

    /// `setFrontWallType`, `main.cpp:4584` and `main.cpp:4576`: take the ledge point and the wall's
    /// own facing off a classification, which is the only place the reference writes either.
    fn latch_wall(&mut self, trace: &WallTrace) {
        self.ledge = trace.ledge;
        self.wall_yaw = trace.wall_yaw.unwrap_or(0);
    }
}

/// The ground polygon the slide states run on, `getSlidePolygon` at `main.cpp:6700` answered: the
/// height it stands at, the normal it presents, and the yaw that runs downhill.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Slide {
    height: f32,
    normal: Vec3,
    downhill: i16,
}

/// Everything a guard or a tick may read, taken once so every one of them sees the same tick. The
/// collision queries answer `None` when no stage is loaded, which is the same answer a stage with
/// nothing there gives.
struct Probe<'a> {
    camera: &'a CameraYaw,
    collision: Option<&'a Collision>,
    pad: Option<&'a PadButtons>,
    swim: &'a Swim,
    jump: &'a Jump,
    roll: &'a Roll,
    hop: &'a Hop,
    combat: &'a Combat,
    lock: &'a Lock,
    intent: &'a MoveIntent,
    /// `cBgS::m_chk_element`, the table `setPushPullKeepData` dispatches through. It is absent until
    /// the first actor that owns a background has put it there, which is nothing to push.
    pushpull: Option<&'a PushPulls>,
    /// The words `setFrontWallType` reads off the player before it traces, rebuilt from the port's
    /// own fields each tick.
    wall: WallProbe,
    /// `MoveFrom`, the position the player stood at when the tick began.
    from: Vec3,
    /// `m35F0 - current.pos.y`, the drop the landing dispatcher compares against before it rolls.
    drop: f32,
    dead: bool,
    riding: bool,
}

impl Probe<'_> {
    fn stick(&self) -> f32 {
        self.intent.axis.length().min(1.0)
    }

    /// `m34E8`, the heading the stick is asking for.
    fn wanted(&self) -> i16 {
        target_yaw(self.intent, self.camera)
    }

    /// Whether a module above already owns the player's motion, which is what the reference reads
    /// off its own process word when it asks for a new one.
    fn busy(&self) -> bool {
        self.swim.active
            || self.jump.active
            || self.roll.active
            || self.hop.active
            || self.combat.active
            || self.lock.target.is_some()
    }

    /// `spActionButton`, `main.cpp:1896`: the crouch and guard face of the pad, read from the level
    /// the pad is polled at rather than from an edge.
    fn crouch_button(&self) -> Option<bool> {
        self.pad.map(|pad| pad.hold.r)
    }

    /// `doTrigger`, `main.cpp:1891`: the jump face on the frame it went down.
    fn jump_trigger(&self) -> bool {
        self.pad.is_some_and(|pad| pad.trig.a)
    }

    /// `PUSH_PULL_KEEP`, the word `procPushMove` and `procPullMove` read to tell a button
    /// that is still down from one that has just gone down, `pushpull.inc:144`. The port keeps no
    /// such word, so the frame's own press edge stands in for it.
    fn fresh_press(&self) -> bool {
        self.pad.is_some_and(|pad| pad.trig.r)
    }

    /// `setFrontWallType`, `main.cpp:4544`: the ten words the wall, hang, climb and jump selectors
    /// are chosen by.
    fn front_wall(&self) -> Option<WallTrace> {
        self.collision
            .map(|collision| collision.front_wall(&self.wall))
    }

    /// `checkNotCrawlStand`, `crawl.inc:309`: whether a point has the room a crawl needs over it.
    /// A missing roof answers `-G_CM3D_F_INF` in the reference, which clears the test, so a world
    /// with no stage under it is free to crawl.
    fn crawl_clear(&self, pos: Vec3, feet: f32) -> bool {
        match self.collision {
            Some(collision) => collision.crawl_clear(pos, feet, CRAWL_CLEARANCE),
            None => true,
        }
    }

    /// `getSlidePolygon`, `main.cpp:6700`: the ground polygon the slide states run on.
    fn slide(&self, pos: Vec3) -> Option<Slide> {
        let poly = self.collision?.slide_polygon(pos.x, pos.z)?;
        Some(Slide {
            height: poly.height,
            normal: poly.normal,
            downhill: to_s16(poly.normal.x.atan2(poly.normal.z)),
        })
    }

    /// `dBgS::GroundCross`, which the reference reads back through `mAcch.GetGroundH`,
    /// `bg_s_acch.h:137`: the ground under a column, which is `-G_CM3D_F_INF` when the stage has
    /// none there.
    fn ground(&self, x: f32, z: f32) -> f32 {
        self.collision
            .and_then(|collision| collision.ground_at(x, z))
            .unwrap_or(f32::NEG_INFINITY)
    }

    /// `mAcch.ChkGroundHit` beside `GetTriPla(...)->GetNP()->y`, `hang.inc:42-47`: the ground under
    /// a hanging player has to be there and has to be level before it may shuffle.
    fn hang_ground(&self, pos: Vec3) -> bool {
        self.collision
            .and_then(|collision| collision.ground_cross(pos.x, pos.z))
            .is_some_and(|hit| hit.normal.y >= HANG_FLAT)
    }

    /// The polygon `mPolyInfo` holds for a push or pull, `main.cpp:4616`: the front wall trace of
    /// `setFrontWallType`, `main.cpp:4561-4567`, offered the registered backgrounds and the room's
    /// own triangles, which is the same test the reference runs once its own geometry holds every
    /// background rather than only the rooms'.
    fn push_pull_poly(&self) -> Option<pushable::Poly> {
        let collision = self.collision?;
        let polys = self.pushpull?;
        let ahead = facing(self.wall.shape_yaw) * (WALL_TRACE_REACH + PUSH_PULL_WALL_R);
        for up in WALL_HEIGHTS.iter().rev() {
            let start = self.wall.pos + Vec3::new(0.0, *up, 0.0);
            if let Some(poly) = polys.line_cross(Some(collision), start, start + ahead) {
                return poly.is_face().then_some(poly);
            }
        }
        None
    }

    /// The actor `setPushPullKeepData` reaches through `dBgS::PushPullCallBack`, `d_bg_s.cpp:542`,
    /// with the wall's own facing and the ledge point `setFrontWallType` leaves on it. `m352C` and
    /// `m3724` are the two words every push and pull state counts its stand-off from.
    fn pushable(&self) -> Option<Pushable> {
        let poly = self.push_pull_poly()?;
        let actor = self.pushpull?.push_pull_actor(poly)?;
        Some(Pushable {
            poly,
            actor,
            wall_yaw: poly.wall_yaw(),
            ledge: Vec3::new(poly.point.x, self.wall.pos.y, poly.point.z),
        })
    }
}

/// The six modules above that own the player's own motion, which the reference reads off its own
/// process word. They travel as one param because a system's own arity runs out at sixteen.
#[derive(SystemParam)]
pub struct Modes<'w> {
    pub swim: Res<'w, Swim>,
    pub jump: Res<'w, Jump>,
    pub roll: Res<'w, Roll>,
    pub hop: Res<'w, Hop>,
    pub combat: Res<'w, Combat>,
    pub lock: Res<'w, Lock>,
}

/// The push and pull state a player stands in front of: the polygon `mPolyInfo` holds, the actor
/// the stage's dispatch answers with behind it, the wall's own facing `m352C` and the ledge point
/// `m3724` the two counts their stand-off from.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pushable {
    pub poly: pushable::Poly,
    pub actor: Entity,
    pub wall_yaw: i16,
    pub ledge: Vec3,
}

/// `setPushPullKeepData`, `pushpull.inc:18-49`: the label the caller passed with the
/// power glove and the grab word folded in, the actor the stage's dispatch answers with, and the
/// player's own position and facing brought onto that actor's since the last time it moved. The
/// box's half of the call is [`pushable::Call::send`].
fn set_push_pull_keep_data(
    player: &mut Player,
    state: &mut MoveState,
    polys: Option<&PushPulls>,
    reach: &mut pushable::Call,
    poly: pushable::Poly,
    label: u32,
) -> Option<Entity> {
    let polys = polys?;
    let mut label = label;
    if label != PPLABEL_NONE {
        if reach.power_glove() {
            label |= PPLABEL_HEAVY;
        }
        if state.armed {
            label |= PPLABEL_UNK4;
        }
    }
    let answer = reach.send(polys, poly, to_s16(player.face), label)?;
    if label != PPLABEL_NONE {
        let turned = answer.shape_yaw.wrapping_sub(state.keep_yaw);
        if turned != 0 {
            player.pos = turned_about(state.grab, turned, player.pos - state.grab);
            player.face = from_s16(to_s16(player.face).wrapping_add(turned));
            player.yaw = from_s16(to_s16(player.yaw).wrapping_add(turned));
        }
        player.pos.x += answer.pos.x - state.grab.x;
        player.pos.z += answer.pos.z - state.grab.z;
    }
    state.grab = answer.pos;
    state.keep_yaw = answer.shape_yaw;
    Some(answer.actor)
}

/// `mDoMtx_stack_c::transS(m370C)` then `YrotM`, `pushpull.inc:36-38`: a point swung about the
/// grab point by the actor's own turn since the last label.
fn turned_about(around: Vec3, angle: i16, offset: Vec3) -> Vec3 {
    let (sin, cos) = from_s16(angle).sin_cos();
    around
        + Vec3::new(
            offset.x * cos + offset.z * sin,
            offset.y,
            -offset.x * sin + offset.z * cos,
        )
}

/// `setFrontWallType` at `main.cpp:4544` opens by reading the player, and every word it reads has a
/// value the port already keeps. `m35D4`, the height a ledge has to clear, is written in one place
/// only — the lava face check at `main.cpp:10710-10713` — and its value while the player is standing
/// on the ground is the negative-infinity sentinel, which is what leaves the test at `main.cpp:4610`
/// open.
fn wall_probe(
    player: &Player,
    collision: Option<&Collision>,
    midair: bool,
    swim: bool,
) -> WallProbe {
    let wall_hit = collision.is_some_and(|collision| {
        WALL_HEIGHTS.iter().any(|up| {
            let start = player.pos + Vec3::new(0.0, *up, 0.0);
            let reach = WALL_TRACE_REACH + WALL_RADIUS;
            collision
                .line_cross(start, start + facing(player.face) * reach)
                .is_some()
        })
    });
    WallProbe {
        pos: player.pos,
        shape_yaw: player.face,
        yaw: player.yaw,
        radius: WALL_RADIUS,
        heights: WALL_HEIGHTS,
        wall_hit,
        ledge_h: f32::NEG_INFINITY,
        ground_h: collision
            .and_then(|collision| collision.ground_at(player.pos.x, player.pos.z))
            .unwrap_or(f32::NEG_INFINITY),
        midair,
        swim,
        hookshot: false,
        pushpull: false,
        grab_wear: false,
        grab_anime: false,
        swim_flag: swim,
        catch_low: CATCH_LOW,
        catch_mid: CATCH_MID,
        catch_high: CATCH_HIGH,
        catch_top: CATCH_TOP,
        swim_low: SWIM_LOW,
        wall_hide_width: WALL_HIDE_WIDTH,
        wall_hide_depth: WALL_HIDE_DEPTH,
        hookshot_fly: false,
    }
}

fn cut_proc(kind: CutKind) -> Proc {
    match kind {
        CutKind::A => Proc::CutA,
        CutKind::F => Proc::CutF,
        CutKind::R => Proc::CutR,
        CutKind::L => Proc::CutL,
        CutKind::EA => Proc::CutEa,
        CutKind::EB => Proc::CutEb,
    }
}

fn jump_proc(kind: JumpKind) -> Proc {
    match kind {
        JumpKind::Back => Proc::BackJump,
        JumpKind::Left | JumpKind::Right => Proc::SideStep,
    }
}

/// The process the modules above already are, which is what every one of the reference's guards
/// falls back on when none of its own condition holds, `main.cpp:4430-4477`.
fn named(player: &Player, probe: &Probe) -> Proc {
    if probe.swim.active {
        return if probe.swim.surfaced {
            Proc::SwimWait
        } else {
            Proc::SwimMove
        };
    }
    if probe.combat.active {
        return cut_proc(probe.combat.kind);
    }
    if probe.hop.active {
        return Proc::AutoJump;
    }
    if probe.roll.active {
        return Proc::FrontRoll;
    }
    if probe.jump.active {
        return jump_proc(probe.jump.kind);
    }
    if probe.lock.target.is_some() {
        return if player.speed.abs() > REST_SPEED {
            Proc::AtnActorMove
        } else {
            Proc::AtnActorWait
        };
    }
    if player.speed.abs() > REST_SPEED {
        Proc::Move
    } else {
        Proc::Wait
    }
}

/// The unit a yaw points along, `cM_ssin` and `cM_scos` of a facing.
fn facing(yaw: f32) -> Vec3 {
    Vec3::new(yaw.sin(), 0.0, yaw.cos())
}

/// A model-space offset run through the base transform. Every offset the crawl, hang and climb
/// states measure is the base transform with the root at zero — `m3700 = cXyz::Zero` at
/// `crawl.inc:267` — so it is the player's own position turned by the model's own yaw.
fn joint(player: &Player, offset: Vec3) -> Vec3 {
    player.pos
        + Vec3::new(
            offset.x * player.face.cos() + offset.z * player.face.sin(),
            offset.y,
            -offset.x * player.face.sin() + offset.z * player.face.cos(),
        )
}

/// `checkCrawlSideWall`, `crawl.inc:209-232`: the first trace finds the face, the second a face a
/// full reach further along its own normal and turned well off it, and the gap between the two cross
/// points has to sit inside the band that makes a crawlable corner. The answer is the second cross
/// point and that face's own yaw.
fn crawl_side_wall(collision: &Collision, from: Vec3, to: Vec3) -> Option<(Vec3, i16)> {
    let first = collision.line_cross(from, to)?;
    let ahead = first.point + first.normal * CRAWL_SIDE_REACH;
    let middle = (ahead + first.point) * 0.5;
    let second = collision.line_cross(middle, ahead)?;
    let gap = first.point - second.point;
    let flat = Vec2::new(gap.x, gap.z).length();
    let first_yaw = to_s16(first.normal.x.atan2(first.normal.z));
    let second_yaw = to_s16(second.normal.x.atan2(second.normal.z));
    (turn_between(first_yaw, second_yaw) > CRAWL_SIDE_TURN
        && flat < CRAWL_SIDE_REACH * CRAWL_SIDE_REACH
        && flat > CRAWL_SIDE_MIN * CRAWL_SIDE_MIN)
        .then_some((second.point, second_yaw))
}

/// `cM3d_Len2dSqPntAndSegLine`, `c_m3d.cpp:1148`: whether the perpendicular from a point lands
/// inside a segment, and where it lands. This is the test `changeCrawlAutoMoveProc` makes before it
/// hands the corner walk its target.
fn point_on_segment(point: Vec2, a: Vec2, b: Vec2) -> Option<Vec2> {
    let travel = b - a;
    let length = travel.length_squared();
    if length <= 0.0 {
        return None;
    }
    let along = ((point - a).dot(travel) / length).clamp(0.0, 1.0);
    (along > 0.0 && along < 1.0).then(|| a + travel * along)
}

/// `changeCrawlAutoMoveProc`, `crawl.inc:84-161`: the corner round the player's front joint, and the
/// point on it a corner walk would run to. `None` when there is no corner, which is the whole of
/// what stops the corner walk from being entered.
fn change_crawl_auto_move(player: &Player, collision: &Collision) -> Option<CrawlCorner> {
    let body = facing(player.yaw);
    let shape = facing(player.face);
    let start = player.pos;
    let ahead = start + body * CRAWL_CORNER_REACH;
    if collision.line_cross(start, ahead).is_none() {
        let side = ahead + Vec3::new(CRAWL_CORNER_SIDE * body.z, 0.0, -CRAWL_CORNER_SIDE * body.x);
        if crawl_side_wall(collision, ahead, side).is_none() {
            return None;
        }
    }

    let left = start
        + Vec3::new(
            CRAWL_CORNER_SIDE * shape.z,
            0.0,
            -CRAWL_CORNER_SIDE * shape.x,
        );
    let across = start * 2.0 - left;
    let mut walls = 0;
    let found;
    if collision.line_cross(start, left).is_some() {
        walls |= 8;
        found = crawl_side_wall(collision, across, across + shape * CRAWL_CORNER_REACH);
        if found.is_none() {
            return None;
        }
    } else {
        walls |= 4;
        found = crawl_side_wall(collision, left, left - shape * CRAWL_CORNER_REACH);
        if found.is_none() {
            return None;
        }
        if collision.line_cross(start, across).is_none() {
            walls |= 8;
        }
    }
    let (wall_point, wall_yaw) = found?;
    let behind = start - body * CRAWL_CORNER_REACH;
    let foot = point_on_segment(
        Vec2::new(wall_point.x, wall_point.z),
        Vec2::new(ahead.x, ahead.z),
        Vec2::new(behind.x, behind.z),
    )?;
    Some(CrawlCorner::of(
        walls,
        Vec3::new(foot.x, player.pos.y, foot.y),
        wall_yaw,
        0.0,
    ))
}

/// `crawlBgCheck`, `crawl.inc:186-206`: the two joint traces run against the stage, and whichever of
/// the two pushes back harder is the one that moves the player.
fn crawl_bg_check(player: &mut Player, collision: &Collision) {
    let front = joint(player, CRAWL_FRONT_OFFSET);
    let top = joint(player, CRAWL_TOP_OFFSET);
    match (
        collision.crawl_stop(top, front),
        collision.crawl_stop(top, top),
    ) {
        (Some(one), Some(two)) => {
            player.pos -= if one.push.length_squared() > two.push.length() {
                one.push
            } else {
                two.push
            };
        }
        (Some(one), None) => player.pos -= one.push,
        (None, Some(two)) => player.pos -= two.push,
        (None, None) => {}
    }
}

/// The lie-down slope test `procCrouch` runs at `main.cpp:6538-6546`: the joint trace the stage would
/// push back out of, the ground under where that push would leave the player, and the turn between
/// the slope and the player's own pitch. `true` leaves the player crouched.
fn crawl_slope_blocked(player: &Player, collision: &Collision) -> bool {
    let front = joint(player, CRAWL_START_OFFSET);
    let top = joint(player, CRAWL_TOP_OFFSET);
    let Some(stop) = collision.crawl_stop(top, front) else {
        return false;
    };
    let back = player.pos - stop.push;
    let lifted = Vec3::new(back.x, back.y + CRAWL_SLOPE_UP, back.z);
    let floor = collision.ground_at(lifted.x, lifted.z).unwrap_or(lifted.y);
    let flat = player.pos - Vec3::new(back.x, floor, back.z);
    let slope = to_s16((-flat.y).atan2((flat.x * flat.x + flat.z * flat.z).sqrt()));
    turn_between(slope, to_s16(player.yaw)) > CRAWL_SLOPE_LIMIT
}

/// `setMoveBGCorrectClimb`, `climb.inc:99-139`: the wall trace every climb state runs first. It
/// answers whether the wall is still under the player's hands, re-seats the player onto whatever face
/// the trace found, and turns the model and the body together by however far the face turned.
/// `Some(proc)` when the climb has to be abandoned. The moving-background half of the reference's
/// function has no counterpart here, because one [`Collision`] holds one room set's static rooms.
fn climb_correct(player: &mut Player, probe: &Probe, moving: bool) -> Option<Proc> {
    let collision = probe.collision?;
    let from = player.pos + Vec3::new(0.0, CLIMB_REACH_UP, 0.0);
    let hit = collision.line_cross(from, from + facing(player.face) * CLIMB_REACH)?;
    if collision.wall_code(hit.tri) != Some(WallCode::Climb) || hit.normal.y.abs() > CLIMB_FACE_FLAT
    {
        return Some(Proc::Fall);
    }
    if moving && probe.ground(player.pos.x, player.pos.z) > player.pos.y - CLIMB_GROUND_BAND {
        return Some(Proc::Fall);
    }
    let wall_yaw = to_s16(hit.normal.x.atan2(hit.normal.z));
    if turn_between(wall_yaw, to_s16(player.face)) <= CLIMB_SAME_WALL {
        return Some(Proc::Fall);
    }
    let wall = facing(from_s16(wall_yaw));
    player.pos.x = hit.point.x + CLIMB_STANDOFF * wall.x;
    player.pos.z = hit.point.z + CLIMB_STANDOFF * wall.z;
    let turned = from_s16(wall_yaw.wrapping_add(-0x8000));
    player.yaw += turned - player.face;
    player.face = turned;
    None
}

/// `changeHangMoveProc`, `hang.inc:41-103`: whether a hanging player may shuffle along the ledge the
/// way the stick is asking for, and the yaw it would shuffle in. `None` refuses.
fn hang_move(player: &Player, probe: &Probe, way: Direction) -> Option<i16> {
    let collision = probe.collision?;
    if !probe.hang_ground(player.pos) {
        return None;
    }
    let shape = to_s16(player.face);
    let yaw = match way {
        Direction::Left => shape.wrapping_add(0x4000),
        Direction::Right => shape.wrapping_sub(0x4000),
        _ => to_s16(player.yaw),
    };
    let from = player.pos + Vec3::new(0.0, HANG_REACH_UP, 0.0);
    let to = from + facing(from_s16(yaw)) * HANG_REACH;
    if collision.line_cross(from, to).is_some() {
        return None;
    }
    if (probe.ground(to.x, to.z) - player.pos.y).abs() > HANG_REACH_UP {
        let drop = Vec3::new(to.x, player.pos.y - HANG_REACH_UP, to.z);
        let hit = collision.line_cross(drop, drop + facing(player.face) * HANG_DROP_REACH)?;
        if turn_between(to_s16(hit.normal.x.atan2(hit.normal.z)), shape) < HANG_SAME_WALL {
            return None;
        }
    }
    let start = from - HANG_SIDE_OFFSET;
    let end = to - HANG_SIDE_OFFSET;
    if let Some(hit) = collision.line_cross(start, end)
        && turn_between(to_s16(hit.normal.x.atan2(hit.normal.z)), shape) < HANG_SAME_WALL
    {
        return None;
    }
    Some(yaw)
}

/// `getHangDirectionFromAngle`, `hang.inc:23-38`: the stick read against the model's own facing on
/// the narrower band a hang uses.
fn hang_direction(probe: &Probe, player: &Player) -> Direction {
    let angle = probe.wanted().wrapping_sub(to_s16(player.face));
    if angle.unsigned_abs() > 0x78E4 {
        Direction::Back
    } else if angle >= 0x071C {
        Direction::Left
    } else if angle <= -0x071C {
        Direction::Right
    } else {
        Direction::Forward
    }
}

/// `changeHangEndProc`, `hang.inc:106-131`: ground above the player's feet ends the hang, and the
/// jump button lets go of it. `Some(proc)` when the hang is over.
fn hang_end(player: &mut Player, probe: &Probe) -> Option<Proc> {
    let back = facing(player.face) * HANG_HAND_BACK;
    let hands = Vec3::new(player.pos.x - back.x, player.pos.y, player.pos.z - back.z);
    if probe.ground(hands.x, hands.z) > player.pos.y + HANG_FOOT_UP {
        return Some(Proc::Land);
    }
    if probe.jump_trigger() {
        player.speed_y = -HANG_FALL_SPEED;
        return Some(Proc::Fall);
    }
    None
}

/// `procHangStart_init` and `procHangWallCatch_init` share their placement, `hang.inc:182-214` and
/// `hang.inc:588-600`: the ledge has to be within a wall radius of the player, and the ground under it
/// within half a foot of the ledge's own height. `None` when the catch is refused.
fn hang_settle(player: &Player, probe: &Probe, state: &MoveState) -> Option<Vec3> {
    let ledge = state.ledge?;
    let wall = facing(from_s16(state.wall_yaw));
    let reach = Vec2::new(ledge.x - player.pos.x, ledge.z - player.pos.z).length();
    if reach > WALL_RADIUS + 20.0 {
        return None;
    }
    let mut stand = Vec3::new(
        ledge.x - HANG_SETTLE * wall.x,
        ledge.y + HANG_SETTLE_UP,
        ledge.z - HANG_SETTLE * wall.z,
    );
    stand.y = probe.ground(stand.x, stand.z);
    ((stand.y - ledge.y).abs() <= HANG_GROUND_BAND).then_some(stand)
}

fn want_crouch(_player: &Player, probe: &Probe) -> Option<(Proc, MoveGap)> {
    if probe.dead || probe.riding || probe.busy() {
        return None;
    }
    match probe.crouch_button() {
        Some(false) => None,
        Some(true) => Some((Proc::Crouch, MoveGap::Ready)),
        None => Some((Proc::Crouch, MoveGap::CrouchButton)),
    }
}

/// `checkNextActionFromButton`'s push and pull arm, `main.cpp:4136-4138`: the button has to be
/// down and the stage's dispatch has to answer with an actor behind the polygon the player's own
/// front wall trace landed on. `dComIfGp_getRStatus() == dActStts_GRAB_e` has no port word, and the
/// reference's trigger is the press edge, which a button held across a stage change would never
/// raise again, so the level of the button is what opens the arm.
fn want_push_pull(_player: &Player, probe: &Probe) -> Option<(Proc, MoveGap)> {
    if probe.dead || probe.riding || probe.busy() {
        return None;
    }
    match probe.crouch_button() {
        Some(true) => probe
            .pushable()
            .map(|_| (Proc::PushPullWait, MoveGap::Ready)),
        Some(false) | None => None,
    }
}

/// `procCrouch`'s own crawl test, `main.cpp:6530-6548`: the button has to be down, the stick has to be
/// out of its gate, and the joint trace's push has to leave the player a slope flat enough to lie on.
fn want_crawl(player: &Player, probe: &Probe) -> Option<(Proc, MoveGap)> {
    if probe.dead || probe.riding || probe.busy() {
        return None;
    }
    match probe.crouch_button() {
        Some(false) => return None,
        Some(true) => {}
        None => return Some((Proc::CrawlStart, MoveGap::CrouchButton)),
    }
    if !grounded(probe.collision, player.pos) || probe.stick() <= STICK_GATE {
        return None;
    }
    let collision = probe.collision?;
    (!crawl_slope_blocked(player, collision)).then_some((Proc::CrawlStart, MoveGap::Ready))
}

/// `changeSlideProc`, `main.cpp:4887-4909`, reached from `checkNextMode` at `main.cpp:4441` and from
/// `procWaitTurn` at `main.cpp:6572`: the polygon the slide runs on and which way it faces. The
/// function's first arm, the `m3526` countdown of `main.cpp:4893-4898`, opens only while `m34C3` is
/// nonzero and `m34C3` is the player's own animation state, which no port field carries, so it is not
/// ported and this arm is the one that always answers.
fn want_slide(player: &Player, probe: &Probe) -> Option<(Proc, MoveGap)> {
    if probe.dead || probe.riding || probe.busy() {
        return None;
    }
    if !grounded(probe.collision, player.pos) {
        return None;
    }
    let slide = probe.slide(player.pos)?;
    Some(
        if turn_between(slide.downhill, to_s16(player.face)) < SLIDE_PICK {
            (Proc::SlideFront, MoveGap::Ready)
        } else {
            (Proc::SlideBack, MoveGap::Ready)
        },
    )
}

/// `main.cpp:4481-4484`: a standing player whose stick asks for a turn past a half turn turns in
/// place rather than starting to move.
fn want_wait_turn(player: &Player, probe: &Probe) -> Option<(Proc, MoveGap)> {
    if probe.dead || probe.riding || probe.busy() {
        return None;
    }
    if player.speed.abs() > REST_SPEED {
        return None;
    }
    if turn_between(probe.wanted(), to_s16(player.yaw)) <= REVERSAL_LIMIT
        || probe.stick() <= STICK_GATE
    {
        return None;
    }
    Some((Proc::WaitTurn, MoveGap::Ready))
}

/// `main.cpp:4492-4506`: a moving player in the same turn takes the moving turn, which splits the
/// model's own facing off the body's.
fn want_move_turn(player: &Player, probe: &Probe) -> Option<(Proc, MoveGap)> {
    if probe.dead || probe.riding || probe.busy() {
        return None;
    }
    if player.speed.abs() <= REST_SPEED {
        return None;
    }
    if turn_between(probe.wanted(), to_s16(player.yaw)) <= REVERSAL_LIMIT
        || probe.stick() <= STICK_GATE
    {
        return None;
    }
    Some((Proc::MoveTurn, MoveGap::Ready))
}

/// `changeLandProc`, `main.cpp:4943-4953`: the stick has to be held, the body must not be asked to
/// reverse, the drop from the height the jump left the ground at must clear
/// [`ROLL_DROP`], and the body must be asked to go forward. The speed and the rate the roll itself
/// runs on are the three words above; the roll ends on its own animation frame, which the port has
/// no clock for.
fn want_side_roll(player: &Player, probe: &Probe) -> Option<(Proc, MoveGap)> {
    if probe.dead || probe.riding {
        return None;
    }
    if probe.stick() <= LAND_STICK || probe.drop < ROLL_DROP {
        return None;
    }
    if matches!(
        direction(probe.wanted(), to_s16(player.yaw)),
        Direction::Back
    ) {
        return None;
    }
    if matches!(
        direction(probe.wanted(), to_s16(player.face)),
        Direction::Back
    ) {
        return None;
    }
    Some((Proc::SideRoll, MoveGap::Ready))
}

/// `changeFrontWallTypeProc`, `main.cpp:4822-4836`: a front wall of type 3 is a climbable face.
fn want_climb(player: &Player, probe: &Probe) -> Option<(Proc, MoveGap)> {
    if probe.dead || probe.riding || probe.busy() {
        return None;
    }
    if !grounded(probe.collision, player.pos) {
        return None;
    }
    match probe.front_wall().map(|wall| wall.kind) {
        Some(FrontWall::Climb) => Some((Proc::ClimbUpStart, MoveGap::Ready)),
        _ => None,
    }
}

/// `changeFrontWallTypeProc`, `main.cpp:4873-4876`: the catch counter has to be past the wall catch
/// table's own word and the classification has to be one of the four ledge bands, which are the only
/// ones that leave a point to catch on.
fn want_hang(_player: &Player, probe: &Probe) -> Option<(Proc, MoveGap)> {
    if probe.dead || probe.riding || probe.busy() {
        return None;
    }
    let wall = probe.front_wall()?;
    if !matches!(
        wall.kind,
        FrontWall::LowLedge | FrontWall::HandLedge | FrontWall::MidLedge | FrontWall::HighLedge
    ) {
        return None;
    }
    wall.ledge?;
    Some((Proc::HangStart, MoveGap::Ready))
}

/// `checkNextMode`, `main.cpp:4438-4444`: the slide is asked first and the button's own dispatcher
/// next, and the push and pull arm of that dispatcher is asked before the crouch, so a player
/// holding the button in front of a registered background pushes rather than lies down.
const ASKS: &[for<'a> fn(&Player, &Probe<'a>) -> Option<(Proc, MoveGap)>] = &[
    want_push_pull,
    want_crouch,
    want_crawl,
    want_slide,
    want_wait_turn,
    want_move_turn,
    want_side_roll,
    want_climb,
    want_hang,
];

/// The reference asks its own guards in this order, and only a process this module does not own may
/// be replaced.
fn resolve(player: &Player, probe: &Probe, current: Proc) -> Option<(Proc, MoveGap)> {
    if MoveState::owns(current) {
        return None;
    }
    ASKS.iter().find_map(|ask| ask(player, probe))
}

/// `procWaitTurn`, `main.cpp:6569-6593`: the speed decays to nothing, the model turns onto the yaw
/// latched on entry, and reaching it ends the turn. The body following the model, `main.cpp:6577`,
/// is the shared ground integrator's own work.
fn wait_turn_tick(player: &mut Player, proc: &LinkProc) -> Option<(Proc, MoveGap)> {
    add_calc(
        &mut player.speed,
        0.0,
        DECEL_SCALE,
        DECEL_MAX_STEP,
        DECEL_MIN_STEP,
    );
    let held = add_calc_angle_s(
        to_s16(player.face),
        proc.target_angle,
        WAIT_TURN_SCALE,
        WAIT_TURN_MAX,
        WAIT_TURN_MIN,
    );
    player.face = from_s16(held);
    (held == 0).then_some((Proc::Wait, MoveGap::Ready))
}

/// `procMoveTurn`, `main.cpp:6618-6625`: the model is turned onto the body at the triple the entry
/// armed. The body itself is turned by the shared ground integrator, which stands in for
/// `setSpeedAndAngleNormal` at `main.cpp:6619`.
fn move_turn_tick(player: &mut Player, proc: &LinkProc) {
    player.face = from_s16(add_calc_angle_s(
        to_s16(player.face),
        to_s16(player.yaw),
        proc.turn_scale,
        proc.turn_step,
        proc.turn_floor,
    ));
}

/// The shared body of both slide ticks, `main.cpp:6699-6749`: the body and the model turn onto the
/// downhill yaw, the speed is pushed along it and held at the ceiling, and losing the polygon is the
/// only way out.
fn slide_tick(player: &mut Player, probe: &Probe, back: bool) -> Option<(Proc, MoveGap)> {
    let Some(slide) = probe.slide(player.pos) else {
        return Some((
            if back {
                Proc::SlideBackLand
            } else {
                Proc::SlideFrontLand
            },
            MoveGap::Ready,
        ));
    };
    player.yaw = from_s16(add_calc_angle_s(
        to_s16(player.yaw),
        slide.downhill,
        SLIDE_TURN_SCALE,
        SLIDE_TURN_MAX,
        SLIDE_TURN_MIN,
    ));
    let shape = to_s16(player.yaw).wrapping_add(if back { -0x8000 } else { 0 });
    player.face = from_s16(add_calc_angle_s(
        to_s16(player.face),
        shape,
        SLIDE_TURN_SCALE,
        SLIDE_TURN_MAX,
        SLIDE_TURN_MIN,
    ));
    let turn = from_s16(to_s16(player.yaw).wrapping_sub(slide.downhill));
    player.speed += (1.0 + 0.5 * (1.0 - slide.normal.y)) * SLIDE_DRIVE * turn.cos();
    if player.speed > SLIDE_SPEED_MAX {
        player.speed = SLIDE_SPEED_MAX;
    }
    let step = facing(player.yaw) * player.speed;
    player.pos.x += step.x;
    player.pos.z += step.z;
    player.pos.y = slide.height;
    player.speed_y = 0.0;
    None
}

/// `procSlideFrontLand`, `main.cpp:6759-6775`, and `procSlideBackLand`, `main.cpp:6784-6800`: the
/// speed runs down, and the recovery animation's own cursor ends the state — its rate reads as
/// stopped at `main.cpp:6762` and `main.cpp:6787`, and its frame past the slide table's mark at
/// `main.cpp:6766` and `main.cpp:6791` hands the player to the dispatcher.
fn slide_land_tick(player: &mut Player, clock: &FrameCtrl, mark: f32) -> Option<(Proc, MoveGap)> {
    add_calc(
        &mut player.speed,
        0.0,
        SLIDE_LAND_SCALE,
        SLIDE_LAND_MAX_STEP,
        SLIDE_LAND_MIN_STEP,
    );
    (clock.rate() < 0.01 || clock.frame() > mark).then_some((Proc::Wait, MoveGap::Ready))
}

/// `procCrawlStart`, `crawl.inc:285-306`, and `procCrawlMove`, `crawl.inc:356-519`: the clearance a
/// crawl needs over its own joints, the slide face it refuses, the turn its model runs on, and the
/// push the stage takes back out of the joint traces. The lie-down itself is a frame-rate question
/// at `crawl.inc:297-301`, so the crawl start runs its push-out once and hands over.
fn crawl_tick(player: &mut Player, probe: &Probe, moving: bool) -> Option<(Proc, MoveGap)> {
    if !probe.crawl_clear(player.pos, player.pos.y) {
        return Some((Proc::CrawlEnd, MoveGap::Ready));
    }
    if moving && probe.slide(player.pos).is_some() {
        return Some((Proc::CrawlEnd, MoveGap::Ready));
    }
    if !probe.crouch_button().unwrap_or(false) {
        return Some((Proc::CrawlEnd, MoveGap::Ready));
    }
    if moving && probe.stick() > STICK_GATE {
        player.face = from_s16(add_calc_angle_s(
            to_s16(player.face),
            probe.wanted(),
            CRAWL_TURN_SCALE,
            CRAWL_TURN_MAX,
            CRAWL_TURN_MIN,
        ));
        player.yaw = player.face;
    }
    if let Some(collision) = probe.collision {
        crawl_bg_check(player, collision);
    }
    None
}

/// `procCrawlMove`'s corner test, `crawl.inc:450-473`: a side wall round the crawl's own shoulder
/// turns the corner walk on, and a clear floor round its front joint turns it off and asks the whole
/// corner walk to be worked out instead.
fn crawl_corner(player: &Player, probe: &Probe) -> Option<CrawlCorner> {
    let collision = probe.collision?;
    let side = joint(player, CRAWL_SIDE_OFFSET);
    let top = joint(player, CRAWL_TOP_OFFSET);
    if let Some((point, yaw)) = crawl_side_wall(collision, top, side) {
        return Some(CrawlCorner::of(1, point, yaw.wrapping_add(0x4000), 1.0));
    }
    change_crawl_auto_move(player, collision)
}

/// `procCrawlAutoMove`, `crawl.inc:544-661`: the corner walk holds its own corner while its counter
/// runs down at `crawl.inc:550-570`, waits on the stick at `crawl.inc:572` while the counter is at
/// zero, and once the stick asks for a direction it turns its model onto that corner's wall at
/// `crawl.inc:636-646` and chases the point it picked at `crawl.inc:644-645`. The speed is
/// `getCrawlMoveSpeed`, `crawl.inc:29-35`, which is the crawl animation's own frame rate times a half
/// sine, so it stays at nothing here. The counter running out hands the walk back at `crawl.inc:656`.
fn crawl_corner_tick(
    player: &mut Player,
    probe: &Probe,
    corner: &mut CrawlCorner,
) -> Option<(Proc, MoveGap)> {
    if corner.hold > 0 {
        corner.hold -= 1;
        return None;
    }
    if corner.hold == 0 {
        if probe.stick() <= STICK_GATE {
            return None;
        }
        corner.hold = -1;
    }
    if corner.turn.abs() > 0.5 {
        player.face = from_s16(add_calc_angle_s(
            to_s16(player.face),
            corner.wall_yaw,
            CRAWL_WALL_SCALE,
            CRAWL_WALL_MAX,
            CRAWL_WALL_MIN,
        ));
        player.yaw = if corner.turn < -0.5 {
            player.face + core::f32::consts::PI
        } else {
            player.face
        };
        add_calc(
            &mut player.pos.x,
            corner.target.x,
            CRAWL_CHASE_SCALE,
            CRAWL_CHASE_MAX_STEP,
            CRAWL_CHASE_MIN_STEP,
        );
        add_calc(
            &mut player.pos.z,
            corner.target.z,
            CRAWL_CHASE_SCALE,
            CRAWL_CHASE_MAX_STEP,
            CRAWL_CHASE_MIN_STEP,
        );
    }
    corner.limit -= 1;
    if corner.limit == 0 {
        return back_to_crawl();
    }
    None
}

/// `procHangStart_init` and `procHangWallCatch_init`'s shared placement, `hang.inc:226-233` and
/// `hang.inc:609-614`: the player is stood on the ledge, turned to face away from the wall, and its
/// speed killed.
fn stand_on_ledge(player: &mut Player, state: &MoveState, stand: Option<Vec3>) {
    if let Some(stand) = stand {
        player.pos = stand;
    }
    player.face = from_s16(state.wall_yaw.wrapping_add(-0x8000));
    player.yaw = player.face;
    player.speed = 0.0;
    player.speed_y = 0.0;
}

/// `procHangStart`, `hang.inc:237-266`, and `procHangWallCatch`, `hang.inc:619-624`: the placement
/// runs, then the catch is only kept while the ground above the player's feet does not end it.
fn hang_catch(player: &mut Player, probe: &Probe, state: &MoveState) -> Option<(Proc, MoveGap)> {
    let stand = hang_settle(player, probe, state);
    stand_on_ledge(player, state, stand);
    hang_end(player, probe).map(|proc| (proc, MoveGap::Ready))
}

/// `procHangFallStart`, `hang.inc:351-378`: the player is taken off the wall it was falling past and
/// turned to face away from it, then the hang's own stick read begins.
fn hang_fall_tick(
    player: &mut Player,
    probe: &Probe,
    state: &MoveState,
) -> Option<(Proc, MoveGap)> {
    let stand = hang_settle(player, probe, state);
    stand_on_ledge(player, state, stand);
    if let Some(next) = hang_end(player, probe) {
        return Some((next, MoveGap::Ready));
    }
    if probe.stick() > STICK_GATE && !matches!(hang_direction(probe, player), Direction::Back) {
        return Some((Proc::HangUp, MoveGap::Ready));
    }
    None
}

/// `procHangUp_init`, `hang.inc:381-394`: the pull-up kills the player's own speed, which is what the
/// climb's own frames would otherwise carry.
fn arm_hang_up(player: &mut Player) {
    player.speed = 0.0;
}

/// `procHangUp`, `hang.inc:397-418`: the end test first, then forward climbs and a side the ledge
/// allows shuffles.
fn hang_up_tick(player: &mut Player, probe: &Probe) -> Option<(Proc, MoveGap)> {
    if let Some(next) = hang_end(player, probe) {
        return Some((next, MoveGap::Ready));
    }
    if probe.stick() <= STICK_GATE {
        return None;
    }
    let way = hang_direction(probe, player);
    if matches!(way, Direction::Back) {
        return None;
    }
    Some(if matches!(way, Direction::Forward) {
        (Proc::HangClimb, MoveGap::Ready)
    } else {
        hang_move(player, probe, way)
            .map(|_| (Proc::HangMove, MoveGap::Ready))
            .unwrap_or((Proc::HangWait, MoveGap::Ready))
    })
}

/// `procHangWait`, `hang.inc:433-451`: the hang is re-tested every tick and the stick read against
/// the model's own facing. Forward climbs; a side the ledge allows shuffles; a backward ask does
/// nothing.
fn hang_wait_tick(player: &mut Player, probe: &Probe) -> Option<(Proc, MoveGap)> {
    if let Some(next) = hang_end(player, probe) {
        return Some((next, MoveGap::Ready));
    }
    if probe.stick() <= STICK_GATE {
        return None;
    }
    match hang_direction(probe, player) {
        Direction::Forward => Some((Proc::HangClimb, MoveGap::Ready)),
        Direction::Back => None,
        way => hang_move(player, probe, way)
            .map(|_| (Proc::HangMove, MoveGap::Ready))
            .or(None),
    }
}

/// `procHangMove`, `hang.inc:474-550`: the shuffle is re-tested and forward climbs. The speed is the
/// hand span's own change across one frame, `hang.inc:495`, which is a joint the port does not carry,
/// so it stays at nothing.
fn hang_move_tick(player: &mut Player, probe: &Probe) -> Option<(Proc, MoveGap)> {
    if let Some(next) = hang_end(player, probe) {
        return Some((next, MoveGap::Ready));
    }
    if probe.stick() > STICK_GATE && matches!(hang_direction(probe, player), Direction::Forward) {
        return Some((Proc::HangClimb, MoveGap::Ready));
    }
    player.speed = 0.0;
    None
}

/// `procCrouch`, `main.cpp:6516-6551`: the speed decays to nothing, letting go of the button ends it,
/// and a held stick with a slope flat enough to lie on becomes the crawl. The reference waits for the
/// crouch animation to reach its end first, which the port has no clock for.
fn crouch_tick(player: &mut Player, probe: &Probe) -> Option<(Proc, MoveGap)> {
    add_calc(
        &mut player.speed,
        0.0,
        DECEL_SCALE,
        DECEL_MAX_STEP,
        DECEL_MIN_STEP,
    );
    if !probe.crouch_button().unwrap_or(false) {
        return Some((Proc::Wait, MoveGap::Ready));
    }
    if probe.stick() <= STICK_GATE {
        return None;
    }
    match probe.collision {
        Some(collision) if crawl_slope_blocked(player, collision) => None,
        _ => Some((Proc::CrawlStart, MoveGap::Ready)),
    }
}

/// `procCrouchDefense`, `main.cpp:6399-6428`: the same decay and the same release. The body lean at
/// `main.cpp:6414-6425` runs on the `mBodyAngle` pair, which the port carries on no component.
fn crouch_defense_tick(player: &mut Player, probe: &Probe) -> Option<(Proc, MoveGap)> {
    add_calc(
        &mut player.speed,
        0.0,
        DECEL_SCALE,
        DECEL_MAX_STEP,
        DECEL_MIN_STEP,
    );
    (!probe.crouch_button().unwrap_or(false)).then_some((Proc::Wait, MoveGap::Ready))
}

/// `procSideRoll`, `main.cpp:6936-6981`: the speed is the side roll's own word, the body follows the
/// model, and a centred stick kills the speed. The reference ends the roll on its animation frame at
/// `main.cpp:6959` and `main.cpp:6969`, which is its own rate between [`ROLL_START`] and [`ROLL_END`];
/// with no animation reader the port runs that same number of ticks itself.
fn side_roll_tick(player: &mut Player, probe: &Probe, proc: &LinkProc) -> Option<(Proc, MoveGap)> {
    player.yaw = player.face;
    let step = facing(player.yaw) * player.speed;
    player.pos.x += step.x;
    player.pos.z += step.z;
    if probe.stick() <= STICK_GATE {
        player.speed = 0.0;
    }
    let length = (ROLL_END - ROLL_START) / ROLL_RATE;
    (proc.frame as f32 >= length).then_some((Proc::Land, MoveGap::Ready))
}

/// `procPushPullWait`, `pushpull.inc:82-120`: the button release ends it, the grab word holds the
/// model turning onto the grab point and the player walking to its stand-off, and once that word is
/// clear the stick picks the push or the pull, each of which asks the stage for the actor before it
/// commits, `pushpull.inc:124` and `:167`.
fn push_pull_wait_tick(
    player: &mut Player,
    probe: &Probe,
    state: &mut MoveState,
    reach: &mut pushable::Call,
) -> Option<(Proc, MoveGap)> {
    if !probe.crouch_button().unwrap_or(false) {
        return Some((Proc::Wait, MoveGap::Ready));
    }
    let pushable = probe.pushable()?;
    if state.armed {
        let grab = state.grab;
        let target = pushable.wall_yaw.wrapping_add(-0x8000);
        player.face = from_s16(add_calc_angle_s(
            to_s16(player.face),
            target,
            PUSH_PULL_SCALE,
            PUSH_PULL_MAX,
            PUSH_PULL_MIN,
        ));
        player.yaw = player.face;
        add_calc(
            &mut player.pos.x,
            grab.x,
            PUSH_PULL_CHASE_SCALE,
            PUSH_PULL_CHASE_MAX_STEP,
            PUSH_PULL_CHASE_MIN_STEP,
        );
        add_calc(
            &mut player.pos.z,
            grab.z,
            PUSH_PULL_CHASE_SCALE,
            PUSH_PULL_CHASE_MAX_STEP,
            PUSH_PULL_CHASE_MIN_STEP,
        );
        let residual = (player.pos.x - grab.x).abs() + (player.pos.z - grab.z).abs();
        if turn_between(to_s16(player.face), target) == 0 && residual < PUSH_PULL_CHASE_LIMIT {
            player.pos.x = grab.x;
            player.pos.z = grab.z;
            state.armed = false;
        }
    }
    if state.armed || probe.stick() <= STICK_GATE {
        return None;
    }
    let want = match direction(probe.wanted(), to_s16(player.face)) {
        Direction::Forward => Proc::PushMove,
        Direction::Back => Proc::PullMove,
        Direction::Left | Direction::Right => return None,
    };
    set_push_pull_keep_data(
        player,
        state,
        probe.pushpull,
        reach,
        pushable.poly,
        PPLABEL_NONE,
    )
    .is_some()
    .then_some((want, MoveGap::Ready))
}

/// `procPushMove`, `pushpull.inc:137-163`: the stick is read against the model's own facing on the
/// frame the button went down, and every frame after that hands the push label to the actor behind
/// `mPolyInfo`, which is what starts the box walking.
fn push_tick(
    player: &mut Player,
    probe: &Probe,
    state: &mut MoveState,
    reach: &mut pushable::Call,
) -> Option<(Proc, MoveGap)> {
    if !probe.crouch_button().unwrap_or(false) {
        return Some((Proc::Wait, MoveGap::Ready));
    }
    let pushable = probe.pushable()?;
    if probe.fresh_press() {
        return match direction(probe.wanted(), to_s16(player.face)) {
            Direction::Forward => {
                set_push_pull_keep_data(
                    player,
                    state,
                    probe.pushpull,
                    reach,
                    pushable.poly,
                    PPLABEL_PUSH,
                );
                None
            }
            Direction::Back => enter_move(player, state, probe, reach, Proc::PullMove),
            Direction::Left | Direction::Right => Some((Proc::PushPullWait, MoveGap::Ready)),
        };
    }
    state.armed = false;
    set_push_pull_keep_data(
        player,
        state,
        probe.pushpull,
        reach,
        pushable.poly,
        PPLABEL_PUSH,
    );
    None
}

/// `procPushMove_init` at `pushpull.inc:124` and `procPullMove_init` at `:167`: both ask the stage
/// for the actor with no label before they commit, and a null answer leaves the player where it
/// stands. The label the new state then sends is what starts the box walking.
fn enter_move(
    player: &mut Player,
    state: &mut MoveState,
    probe: &Probe,
    reach: &mut pushable::Call,
    want: Proc,
) -> Option<(Proc, MoveGap)> {
    let pushable = probe.pushable()?;
    set_push_pull_keep_data(
        player,
        state,
        probe.pushpull,
        reach,
        pushable.poly,
        PPLABEL_NONE,
    )
    .is_some()
    .then_some((want, MoveGap::Ready))
}

/// `procPullMove`, `pushpull.inc:180-232`: the two wall traces behind the player and the ground
/// probe past them come first on the frame the button went down, and only a probe that passes hands
/// the pull label over.
fn pull_tick(
    player: &mut Player,
    probe: &Probe,
    state: &mut MoveState,
    reach: &mut pushable::Call,
) -> Option<(Proc, MoveGap)> {
    if !probe.crouch_button().unwrap_or(false) {
        return Some((Proc::Wait, MoveGap::Ready));
    }
    let pushable = probe.pushable()?;
    if probe.fresh_press() {
        return match direction(probe.wanted(), to_s16(player.face)) {
            Direction::Back => {
                if !pull_clear(player, probe) {
                    set_push_pull_keep_data(
                        player,
                        state,
                        probe.pushpull,
                        reach,
                        pushable.poly,
                        PPLABEL_PULL,
                    );
                }
                None
            }
            Direction::Forward => enter_move(player, state, probe, reach, Proc::PushMove),
            Direction::Left | Direction::Right => Some((Proc::PushPullWait, MoveGap::Ready)),
        };
    }
    state.armed = false;
    set_push_pull_keep_data(
        player,
        state,
        probe.pushpull,
        reach,
        pushable.poly,
        PPLABEL_PULL,
    );
    None
}

/// `procPullMove`'s two wall traces and its ground probe, `pushpull.inc:190-216`: a wall behind the
/// player at either height, or a floor more than `30.1` below the feet further back, refuses the
/// pull. The second trace's endpoint is the first one's shifted along the same facing, `pushpull.inc:210`.
fn pull_clear(player: &Player, probe: &Probe) -> bool {
    if let Some(collision) = probe.collision {
        let shape = facing(player.face);
        for up in PULL_PROBE_UP {
            let from = player.pos + Vec3::new(0.0, up, 0.0);
            if collision
                .line_cross(from, from - shape * PULL_REACH)
                .is_some()
            {
                return false;
            }
        }
    }
    let back = player.pos - facing(player.face) * (PULL_REACH - PULL_GROUND_BACK);
    !(probe.ground(back.x, back.z) - player.pos.y < -PULL_GROUND_DROP)
}

/// Back into the plain crawl, which is where `procCrawlAutoMove`'s counter running out at
/// `crawl.inc:656-657` and `procCrawlEnd` at `crawl.inc:711` both hand the player.
fn back_to_crawl() -> Option<(Proc, MoveGap)> {
    Some((Proc::CrawlMove, MoveGap::Ready))
}

/// `procCrawlEnd`, `crawl.inc:708-719`: the stand-up runs on the lie animation played back, so it
/// holds until the cursor is back under the crouch table's mark at `crawl.inc:712`, and a rate the
/// animation has already stopped at `crawl.inc:710` leaves at once. A ceiling the player no longer
/// fits under is the one condition the reference tests here that the port can answer, and it ends
/// the crawl outright.
fn crawl_end_tick(
    player: &mut Player,
    probe: &Probe,
    clock: &FrameCtrl,
) -> Option<(Proc, MoveGap)> {
    if !probe.crawl_clear(player.pos, player.pos.y) {
        return Some((Proc::Wait, MoveGap::Ready));
    }
    if clock.rate() > -0.01 || clock.frame() < CRAWL_END_FRAME {
        return back_to_crawl();
    }
    None
}

/// `procHangClimb`, `hang.inc:573-580`: the vault's own cursor ends the state — its rate reads as
/// stopped at `hang.inc:574`, and its frame past the wall-catch table's mark at `hang.inc:576`
/// hands the player to the dispatcher.
fn hang_climb_tick(clock: &FrameCtrl) -> Option<(Proc, MoveGap)> {
    (clock.rate() < 0.01 || clock.frame() > HANG_CLIMB_FRAME)
        .then_some((Proc::Wait, MoveGap::Ready))
}

/// `procClimbMoveUpDown`, `climb.inc:365-391`, and `procClimbMoveSide`, `climb.inc:432-465`: the wall
/// trace first, and then the stick is read against the model's own facing to choose which of the two
/// moving states the climb continues in. Both directions hold the player still, because their speed
/// is the animation's own frame rate and their side move's is the hand span's change across a frame,
/// `climb.inc:449-462`.
fn climb_move_tick(player: &mut Player, probe: &Probe, previous: Proc) -> Option<(Proc, MoveGap)> {
    if let Some(fall) = climb_correct(player, probe, true) {
        return Some((fall, MoveGap::Ready));
    }
    if probe.jump_trigger() {
        return Some((Proc::Fall, MoveGap::Ready));
    }
    if probe.stick() <= STICK_GATE {
        return None;
    }
    match direction(probe.wanted(), to_s16(player.face)) {
        Direction::Forward | Direction::Back => {
            (Proc::ClimbMoveUpDown != previous).then_some((Proc::ClimbMoveUpDown, MoveGap::Ready))
        }
        _ => (Proc::ClimbMoveSide != previous).then_some((Proc::ClimbMoveSide, MoveGap::Ready)),
    }
}

/// The tick of whichever owned process is standing. `None` when it stays where it is, which is the
/// normal answer for a state whose only exit is its own animation.
fn tick(
    player: &mut Player,
    probe: &Probe,
    state: &mut MoveState,
    proc: &LinkProc,
    reach: &mut pushable::Call,
    clock: &FrameCtrl,
) -> Option<(Proc, MoveGap)> {
    match proc.current {
        Proc::Crouch => crouch_tick(player, probe),
        Proc::CrouchDefense => crouch_defense_tick(player, probe),
        Proc::CrawlStart => match crawl_tick(player, probe, false) {
            Some(ask) => Some(ask),
            None => back_to_crawl(),
        },
        Proc::CrawlMove => {
            state.corner = None;
            crawl_tick(player, probe, true)?;
            crawl_corner(player, probe).map(|corner| {
                state.corner = Some(corner);
                (Proc::CrawlAutoMove, MoveGap::Ready)
            })
        }
        Proc::CrawlAutoMove => {
            let Some(mut corner) = state.corner else {
                return back_to_crawl();
            };
            let next = crawl_corner_tick(player, probe, &mut corner);
            state.corner = Some(corner);
            next
        }
        Proc::CrawlEnd => crawl_end_tick(player, probe, clock),
        Proc::PushPullWait => push_pull_wait_tick(player, probe, state, reach),
        Proc::PushMove => push_tick(player, probe, state, reach),
        Proc::PullMove => pull_tick(player, probe, state, reach),
        Proc::SlideFront => slide_tick(player, probe, false),
        Proc::SlideBack => slide_tick(player, probe, true),
        Proc::SlideFrontLand => slide_land_tick(player, clock, SLIDE_FRONT_LAND_FRAME),
        Proc::SlideBackLand => slide_land_tick(player, clock, SLIDE_BACK_LAND_FRAME),
        Proc::WaitTurn => wait_turn_tick(player, proc),
        Proc::MoveTurn => {
            move_turn_tick(player, proc);
            None
        }
        Proc::SideRoll => side_roll_tick(player, probe, proc),
        Proc::HangStart | Proc::HangWallCatch => hang_catch(player, probe, state),
        Proc::HangFallStart => hang_fall_tick(player, probe, state),
        Proc::HangUp => {
            arm_hang_up(player);
            hang_up_tick(player, probe)
        }
        Proc::HangWait => hang_wait_tick(player, probe),
        Proc::HangMove => hang_move_tick(player, probe),
        Proc::HangClimb => hang_climb_tick(clock),
        Proc::ClimbUpStart | Proc::ClimbDownStart => {
            climb_correct(player, probe, false).map(|proc| (proc, MoveGap::Ready))
        }
        Proc::ClimbMoveUpDown => climb_move_tick(player, probe, Proc::ClimbMoveUpDown),
        Proc::ClimbMoveSide => climb_move_tick(player, probe, Proc::ClimbMoveSide),
        _ => Some((Proc::Wait, MoveGap::Ready)),
    }
}

/// The owned processes whose reference tick ends by asking the whole dispatcher again, so their own
/// guards have to run under them too: `procWaitTurn` at `main.cpp:6578`, `procMoveTurn` at
/// `main.cpp:6621`, `procCrouch` at `main.cpp:6529`, `procCrouchDefense` at `main.cpp:6409` and
/// `procSideRoll` at `main.cpp:6964` all call `checkNextMode` or `checkNextActionFromButton`.
fn asks_dispatcher(proc: Proc) -> bool {
    matches!(
        proc,
        Proc::Crouch | Proc::CrouchDefense | Proc::WaitTurn | Proc::MoveTurn | Proc::SideRoll
    )
}

/// Ask the guard chain without the ownership test, for a tick that reported no transition of its own.
fn next_mode(player: &Player, probe: &Probe) -> Option<(Proc, MoveGap)> {
    ASKS.iter().find_map(|ask| ask(player, probe))
}

/// `procClimbUpStart_init`, `climb.inc:237-243`: the player is stood off the ledge along the wall's
/// own facing and turned to face into it.
fn arm_climb(player: &mut Player, state: &MoveState) {
    if let Some(ledge) = state.ledge {
        let wall = facing(from_s16(state.wall_yaw));
        player.pos.x = ledge.x + CLIMB_START_REACH * wall.x;
        player.pos.z = ledge.z + CLIMB_START_REACH * wall.z;
    }
    player.face = from_s16(state.wall_yaw.wrapping_add(-0x8000));
    player.yaw = player.face;
    player.speed = 0.0;
    player.speed_y = 0.0;
}

/// `procClimbDownStart_init`, `climb.inc:293-296`: the same turn, one reach further back off the wall
/// and a fifth of the height lower.
fn arm_climb_down(player: &mut Player, state: &MoveState) {
    arm_climb(player, state);
    let wall = facing(from_s16(state.wall_yaw));
    player.pos.x -= CLIMB_DOWN_BACK * wall.x;
    player.pos.z -= CLIMB_DOWN_BACK * wall.z;
    player.pos.y -= CLIMB_DOWN_DROP;
}

/// The words each state latches on entry, which the reference writes in its own initialiser. The
/// last argument is whether the reference's own dispatcher asked for the process rather than one of
/// its own ticks, which is the only thing that separates `procPushPullWait_init(1)` at
/// `main.cpp:4138` from `procPushPullWait_init(0)` at `pushpull.inc:154` and `:223`.
///
/// The four states whose whole length is an animation hand the cursor their own initialiser plays,
/// through `setSingleMoveAnime` at `main.cpp:12796-12797`: a rate under zero starts the cursor at
/// its last frame and a rate over or at zero at its first, and the loop point goes on the end the
/// rate runs away from. The reference takes the attribute off the animation's own header, which no
/// port can read, so each is stood up as the one shot its exit test is written against; all four
/// leave on a frame mark inside the span, so the attribute cannot change the answer. The speed each
/// initialiser kills or halves is written beside it: `crawl.inc:702`, `main.cpp:6755` and
/// `hang.inc:567`.
fn arm(
    state: &mut MoveState,
    proc: &mut LinkProc,
    probe: &Probe,
    player: &mut Player,
    clock: &mut FrameCtrl,
    want: Proc,
    dispatched: bool,
) {
    match want {
        Proc::WaitTurn => {
            proc.target_angle = probe.wanted();
            player.face = player.yaw;
        }
        Proc::MoveTurn => {
            if turn_between(probe.wanted(), to_s16(player.yaw)) > REVERSAL_LIMIT {
                proc.turn_scale = MOVE_TURN_BACK_SCALE;
                proc.turn_step = MOVE_TURN_BACK_MAX;
                proc.turn_floor = MOVE_TURN_BACK_MIN;
            } else {
                proc.turn_scale = MOVE_TURN_FACE_SCALE;
                proc.turn_step = MOVE_TURN_FACE_MAX;
                proc.turn_floor = MOVE_TURN_FACE_MIN;
            }
        }
        Proc::SideRoll => player.speed = ROLL_SPEED,
        Proc::ClimbUpStart => arm_climb(player, state),
        Proc::ClimbDownStart => arm_climb_down(player, state),
        Proc::PushPullWait => arm_push_pull_wait(probe, state, player, dispatched),
        Proc::PushMove | Proc::PullMove => state.armed = true,
        Proc::CrawlEnd => {
            player.speed = 0.0;
            clock.set_frame_ctrl(
                Mode::None,
                CRAWL_END_START,
                CRAWL_END_LAST,
                CRAWL_END_RATE,
                f32::from(CRAWL_END_LAST) - 0.001,
            );
        }
        Proc::SlideFrontLand => {
            player.speed *= 0.5;
            clock.set_frame_ctrl(
                Mode::None,
                SLIDE_FRONT_LAND_START,
                SLIDE_FRONT_LAND_LAST,
                SLIDE_FRONT_LAND_RATE,
                f32::from(SLIDE_FRONT_LAND_START),
            );
        }
        Proc::SlideBackLand => clock.set_frame_ctrl(
            Mode::None,
            SLIDE_BACK_LAND_START,
            SLIDE_BACK_LAND_LAST,
            SLIDE_BACK_LAND_RATE,
            f32::from(SLIDE_BACK_LAND_START),
        ),
        Proc::HangClimb => {
            player.speed = 0.0;
            clock.set_frame_ctrl(
                Mode::None,
                HANG_CLIMB_START,
                HANG_CLIMB_LAST,
                HANG_CLIMB_RATE,
                f32::from(HANG_CLIMB_START),
            );
        }
        _ => {}
    }
}

/// `procPushPullWait_init`, `pushpull.inc:52-79`: the grab point is the ledge point `m3724` a
/// reach along the wall's own facing `m352C`, at the player's own height, and `mProcVar6.m3570` is
/// the parameter of that initialiser, which the tick at `pushpull.inc:94-102` runs its alignment
/// under and clears at `:101`. `procPushMove_init` and `procPullMove_init`, `pushpull.inc:130` and
/// `:173`, raise that word too, which is what puts the unknown bit on the first label of either
/// state; the grab point is the actor's own position by then, written by the keep data their
/// initialiser ran first at `pushpull.inc:124` and `:167`.
fn arm_push_pull_wait(probe: &Probe, state: &mut MoveState, player: &Player, dispatched: bool) {
    state.grab = probe.pushable().map_or(Vec3::ZERO, |pushable| {
        let wall = facing(from_s16(pushable.wall_yaw));
        Vec3::new(
            pushable.ledge.x + PUSH_PULL_REACH * wall.x,
            player.pos.y,
            pushable.ledge.z + PUSH_PULL_REACH * wall.z,
        )
    });
    state.armed = dispatched;
}

/// The takeoff height, as the reference keeps it. Every grounded tick rewrites it with the height
/// the feet are standing at, which is the same number `posMoveFromFootPos` writes on the tick a fall
/// begins at `main.cpp:2476-2478` once the last grounded tick has passed, and a tick that enters a
/// mid-air process rewrites it with the height the tick began at, which is the launch height every
/// jump initialiser stamps at `main.cpp:7090`. While the player is in the air and neither happened
/// the stored height is what the drop is measured from.
fn takeoff(player: &Player, probe: &Probe, entered_air: bool, held: f32) -> f32 {
    if entered_air {
        return probe.from.y;
    }
    if grounded(probe.collision, player.pos) {
        return player.pos.y;
    }
    held
}

/// One guarded transition, with the entry words the reference's own initialiser writes.
fn enter(
    state: &mut MoveState,
    proc: &mut LinkProc,
    probe: &Probe,
    player: &mut Player,
    clock: &mut FrameCtrl,
    want: Proc,
    dispatched: bool,
) {
    if proc.enter(want) {
        arm(state, proc, probe, player, clock, want, dispatched);
    }
}

/// Report what the tick of the standing process asked, and either take it or, when it carries a gap,
/// hand the player back to the process the modules above name.
fn settle(
    player: &mut Player,
    probe: &Probe,
    state: &mut MoveState,
    proc: &mut LinkProc,
    clock: &mut FrameCtrl,
    ask: Option<(Proc, MoveGap)>,
) -> bool {
    let Some((want, gap)) = ask else {
        return false;
    };
    state.asked = Some(want);
    state.gap = gap;
    if gap == MoveGap::Ready {
        enter(state, proc, probe, player, clock, want, false);
        return true;
    }
    state.corner = None;
    proc.set_proc(named(player, probe));
    false
}

/// Name the process the modules above already are, ask the reference's own guards for the ones no
/// module owns, then run the tick of whichever owned process is standing. A guard or a tick that
/// reports a gap does not enter its process, and a tick that reports one leaves whatever owned
/// process was standing, because a state with no behaviour to run is worse than the fallback.
pub fn control(
    from: Option<Res<MoveFrom>>,
    intent: Res<MoveIntent>,
    camera: Res<CameraYaw>,
    collision: Option<Res<Collision>>,
    pad: Option<Res<PadButtons>>,
    health: Res<Health>,
    modes: Modes<'_>,
    polys: Option<Res<PushPulls>>,
    mut reach: pushable::Call<'_, '_>,
    ships: Query<&Ship>,
    mut proc: ResMut<LinkProc>,
    mut state: ResMut<MoveState>,
    mut clocks: ResMut<FrameClocks>,
    mut players: Query<&mut Player>,
) {
    let Modes {
        swim,
        jump,
        roll,
        hop,
        combat,
        lock,
    } = modes;
    let riding = ships.iter().any(|ship| ship.riding);
    let Some(mut player) = players.iter_mut().next() else {
        return;
    };
    let player = &mut *player;
    let clock = clocks.under_mut(Under::Move0);
    let entered_air = proc.changed && proc.current.in_group(super::link_proc::AIRBORNE);
    let solid = collision.as_deref();
    let mut probe = Probe {
        camera: &camera,
        collision: solid,
        pad: pad.as_deref(),
        swim: &swim,
        jump: &jump,
        roll: &roll,
        hop: &hop,
        combat: &combat,
        lock: &lock,
        intent: &intent,
        pushpull: polys.as_deref(),
        wall: wall_probe(player, solid, !grounded(solid, player.pos), swim.active),
        from: from.as_deref().map_or(player.pos, |from| from.0),
        drop: proc.takeoff_y - player.pos.y,
        dead: health.units == 0,
        riding,
    };
    let launch = takeoff(player, &probe, entered_air, proc.takeoff_y);
    proc.mark_takeoff(launch);
    probe.drop = proc.takeoff_y - player.pos.y;
    proc.advance();
    if !probe.dead
        && !probe.riding
        && !probe.busy()
        && let Some(trace) = probe.front_wall()
    {
        state.latch_wall(&trace);
    }
    if !MoveState::owns(proc.current) {
        state.asked = None;
        match resolve(player, &probe, proc.current) {
            Some((want, MoveGap::Ready)) => {
                state.gap = MoveGap::Ready;
                enter(&mut state, &mut proc, &probe, player, clock, want, true);
            }
            Some((want, gap)) => {
                state.asked = Some(want);
                state.gap = gap;
            }
            None => {
                state.gap = MoveGap::Ready;
                // The cuts are `Combat`'s, so nothing in the asks above ever answers them; without
                // this arm a finished swing keeps the process and `allows(CUT_GROUND)` refuses
                // every later one. `procCutL` and its siblings leave through `checkNextMode` the
                // tick their own frame control does (`d_a_player_sword.inc:900-903`, `:909-919`;
                // `checkNextMode`'s entry, `d_a_player_main.cpp:4412-4413`), and the port's cut
                // frame control is the combat state itself: while the swing lives `named` answers
                // the cut back, so once it is gone the cut resolves to the guard that now holds.
                if !probe.combat.active && CUT_AIR.contains(&proc.current) {
                    enter(
                        &mut state,
                        &mut proc,
                        &probe,
                        player,
                        clock,
                        named(player, &probe),
                        true,
                    );
                }
            }
        }
        return;
    }
    let standing = proc.current;
    let ask = tick(player, &probe, &mut state, &proc, &mut reach, clock);
    let moved = settle(player, &probe, &mut state, &mut proc, clock, ask);
    if moved || proc.current != standing || !asks_dispatcher(standing) {
        return;
    }
    let ask = next_mode(player, &probe);
    match ask {
        Some((want, MoveGap::Ready)) => {
            state.gap = MoveGap::Ready;
            enter(&mut state, &mut proc, &probe, player, clock, want, true);
        }
        Some((want, gap)) => {
            state.asked = Some(want);
            state.gap = gap;
        }
        None => state.gap = MoveState::gap_of(standing),
    }
}
