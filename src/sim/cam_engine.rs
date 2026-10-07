use glam::{Vec2, Vec3};

use super::cam_event;
use super::cam_follow::{self, FollowInput, SAngle, SGlobe, ViewCache};
use super::cam_lockon;
use super::cam_room::{CamRoom, CAM_FIELD_UNSET};
use super::cam_style::{self, flag, param, CamEngine};
use super::event_manager::EventManager;
use super::cam_tp_engines;
use super::camera::Camera;
use super::collision::Collision;
use super::player::Player;
use super::player_status;

/// The player's `mHeight`, which `getHeight` answers for a player at `d_camera.cpp:1988-1994`
/// and which the two fixed-placement engines divide their travel by.
const PLAYER_HEIGHT: f32 = 125.0;

/// `mWork.subject.m378 = 'SUBJ'`, `d_camera.cpp:3852`.
const SUBJECT_TAG: i32 = i32::from_be_bytes(*b"SUBJ");
/// `mWork.subject.m3A8 = 100`, `d_camera.cpp:3861`.
const SUBJECT_COUNTER: i32 = 100;
/// `at.y -= 25.0f`, `d_camera.cpp:4060`: how far below the target's attention point the settled
/// arm's aim sits.
const SUBJECT_DROP: f32 = 25.0;
/// `mCamSetup.m030`, `d_cam_param.cpp:316`: the scoped styles' zoom gain, which
/// `CalcSubjectAngle` folds into the style's own gate at `d_camera.cpp:3791`.
const SUBJECT_CUSHION: f32 = 0.75;
/// `mCamSetup.mCurveWeight`, `d_cam_param.cpp:322`, the ease `CalcSubjectAngle` and the scope's
/// zoom both run their stick through.
const CURVE_WEIGHT: f32 = 1.0;

/// The `dCamera_c::mEventFlags` bits `subjectCamera` and `eventCamera` read or write, on
/// [`Camera::event_flags`].
///
/// The four bits `bumpCheck` writes are named in [`cam_bump::evflag`](super::cam_bump::evflag),
/// because that module owns them and nothing in this file is allowed to move them.
pub mod evflag {
    /// `setFlag(0x800)` at `d_camera.cpp:3892` from the style's `UNK080`, read at `:904`.
    pub const SUBJECT: u32 = 0x800;
    /// `setFlag(0x10000000)` at `d_camera.cpp:3895` from the style's `UNK100`, read at `:908`.
    pub const SUBJECT_WIDE: u32 = 0x1000_0000;
    /// `setFlag(0x3000000)` in `SubjectLockOn` / `SubjectLockOff`, `d_camera.cpp:5222-5230`.
    /// Bit 25 is what `subjectCamera`'s settled arm tests twice, at `:4058` and `:4077`.
    pub const SUBJECT_LOCK: u32 = 0x2000_000;
    /// `chkFlag(0x20000000)` at `d_camera.cpp:4761`, `:388`, `:697`, `:716`, `:924` and `:1270`.
    /// Nothing in the target's own sources writes it: it is the "an event camera is running" bit
    /// and the target gates those six sites on the event manager's camera-play query beside it.
    pub const EVENT_PLAY: u32 = 0x2000_0000;
    /// `clrFlag(0x200000)` at `d_camera.cpp:1273` and `SetTrimTypeForce`'s own caller, read back
    /// at `d_camera.cpp:696` and `:786`: the cut named a placement the camera is not to move out
    /// of. `eventCamera` clears it when the camera staff changes and when a cut's first frame
    /// runs, and raises it when the cut named `WaitAnyKey`, `d_camera.cpp:4773`, `:4803` and
    /// `:4830`; [`cam_event`](crate::sim::cam_event) owns both writes.
    pub const FIXED: u32 = 0x0020_0000;
    /// `setFlag(0x400)`, `d_camera.cpp:4864` and `d_camera.cpp:808-809`: an event arm has set the
    /// bank itself, which is what holds the bank's own decay off for that frame. Only the event
    /// arms raise it, at `d_camera.cpp:4810-4824` and its eleven siblings.
    pub const BANK: u32 = 0x400;
    /// `setFlag(1)` from the pause arm's `Stay` parameter, `d_camera.cpp:488`, cleared again by
    /// the run's own `clrFlag(0x10149C01)` at `d_camera.cpp:690` before every engine call.
    pub const STAY: u32 = 0x1;
}

/// The lock-on state `dCamera_c` carries beside the style: the one target the camera is aimed
/// at and the two answers `dAttention_c` gives about it.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct LockInput {
    /// `dCamera_c::mpLockonTarget`, `d_camera.h:198`, as `attentionPos` answers it at
    /// `d_camera.cpp:1589-1591`. Retail's `Att` at `d_camera.cpp:638-650` fills it from the
    /// attention's lock-on list, and its other three writers are the mode and type choosers.
    pub target: Option<Vec3>,
    /// `dAttention_c::LockonTruth`, `d_attention.cpp:1048-1051`, read at `d_camera.cpp:3331`.
    pub truth: bool,
    /// `chkFlag(AttnFlag_00000008) || chkFlag(AttnFlag_00000020)` at `d_camera.cpp:3348`, the
    /// pair that restarts the placement. The port's attention keeps no flag word.
    pub focus: bool,
}

/// What one engine call reads besides the camera: the pad the follow and subject engines want,
/// the lock-on target the lock-on engine is framed on, and the stage collision the arrow
/// engines line-check against.
pub struct EngineInput<'a> {
    pub follow: FollowInput<'a>,
    /// `CPad_GET_STICK_POS_X/Y`, the subject engine's aim and zoom stick.
    pub cstick: Vec2,
    pub collision: Option<&'a Collision>,
    pub lockon: LockInput,
}

/// `dCamera_c::mWork.crawl`, `d_camera.h:350-361`. The tag `m378` and the spare `m39A` bytes are
/// left out; every member the engine touches is here.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Crawl {
    /// `m37C`: the frames the placement is spread over.
    pub frames: i32,
    /// `m380`: the placement budget, halved into a triangle when the placement starts.
    pub budget: f32,
    /// `m384`: this frame's share of the budget.
    pub share: f32,
    /// `m388`: which side of the player the eye sits on, `0` in front and `1` behind.
    pub behind: bool,
    /// `m38C`: the arrow's position, which the settled arm snaps the eye onto.
    pub arrow: Vec3,
    /// `m398`: the style's `UNK080`, read once when the placement starts.
    pub tight: bool,
    /// `m399`: whether an arrow is in front of the camera and owns the eye.
    pub aiming: bool,
    /// `m39C`: retail's own tag, written once and read by no line in the readable text.
    pub tag: i32,
}

impl Default for Crawl {
    fn default() -> Self {
        Self {
            frames: 0,
            budget: 0.0,
            share: 0.0,
            behind: false,
            arrow: Vec3::ZERO,
            tight: false,
            aiming: false,
            tag: 0,
        }
    }
}

/// `dCamera_c::mWork.fixedFrame`, `d_camera.h:362-371`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FixedFrame {
    /// `m378`: the arrow's position, which is the centre the placement converges onto.
    pub center: Vec3,
    /// `m384`: the eye relative to the centre, kept as a globe so the walk stays on the arc.
    pub direction: SGlobe,
    /// `m38C`: the frames the placement is spread over.
    pub frames: i32,
    /// `m390`: the placement budget, halved into a triangle.
    pub budget: f32,
    /// `m394`: this frame's share of the budget.
    pub share: f32,
    /// `m398`: the camera row's field of view, or the style's when the row leaves it unset.
    pub fovy: f32,
    /// `m39C`: the centre the placement walks from.
    pub start_center: Vec3,
    /// `m3A8`: the eye the placement walks from.
    pub start_eye: Vec3,
}

impl Default for FixedFrame {
    fn default() -> Self {
        Self {
            center: Vec3::ZERO,
            direction: SGlobe::new(0.0, SAngle::ZERO, SAngle::ZERO),
            frames: 0,
            budget: 0.0,
            share: 0.0,
            fovy: 0.0,
            start_center: Vec3::ZERO,
            start_eye: Vec3::ZERO,
        }
    }
}

/// `dCamera_c::mWork.fixedPos`, `d_camera.h:372-379`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FixedPos {
    /// `m378`: the frames the placement is spread over.
    pub frames: i32,
    /// `m37C`: the placement budget.
    pub budget: f32,
    /// `m380`: this frame's share of the budget.
    pub share: f32,
    /// `m384`: the centre the placement walks onto.
    pub center: Vec3,
    /// `m390`: the last arrow position, which only the DMC check at `d_camera.cpp:4553` reads.
    pub arrow: Vec3,
    /// `m39C`: set when the camera row names a fixed number of placement frames.
    pub timed: bool,
}

impl Default for FixedPos {
    fn default() -> Self {
        Self {
            frames: 0,
            budget: 0.0,
            share: 0.0,
            center: Vec3::ZERO,
            arrow: Vec3::ZERO,
            timed: false,
        }
    }
}

/// `dCamera_c::mWork.subject`, `d_camera.h:325-349`, narrowed to the members `subjectCamera` and
/// `CalcSubjectAngle` read and write at `d_camera.cpp:3829-4176` and `:3692-3827`.
///
/// `m390` and `m394` are written by the placement arm's last frame and read by no line of the
/// readable text; `m398` is the scope's wipe target. `m3B8`, the pitch the angle source measures,
/// is written at `:3737` and `:3740` and read nowhere, so the port does not carry it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Subject {
    /// `m378`: the `'SUBJ'` mark retail stamps on the arm when its first frame runs.
    pub tag: i32,
    /// `m37C`: the settled arm's aim-source latch, read by `CalcSubjectAngle` at `:3716` and
    /// written at `:4017`, `:4076`, `:4096` and `:4108`.
    pub pad: bool,
    /// `m37D`: which of the two angle sources the settled arm reads.
    pub from_pad: bool,
    /// `m380`: the style the placement belongs to, which is the row `CalcSubjectAngle` reads.
    pub style: i32,
    /// `m384`: the aim stick's horizontal contribution.
    pub aim_x: f32,
    /// `m388`: the aim stick's vertical contribution.
    pub aim_y: f32,
    /// `m38C`: the zoom stick's eased position.
    pub zoom: f32,
    /// `m390`: the aim stick's own gain.
    pub zoom_gain: f32,
    /// `m394`: the aim stick's settle term.
    pub settle: f32,
    /// `m398`: the entry field-of-view term.
    pub entry: f32,
    /// `m39C`: the field of view the scoped zoom walks from.
    pub zoom_fovy: f32,
    /// `m3A0`: the scoped field of view.
    pub fovy: f32,
    /// `m3A8`: the frames the settled arm has run, counted to ten at `:4105-4106`.
    pub counter: i32,
    /// `m3BC`: whether the framed viewport has been taken once.
    pub framed: bool,
    /// `m3BD`: set when the player stands inside the floor margin, which pins the offset.
    pub low: bool,
    /// `m3C0`: the centre source: `0` relational, `1` from the eye, `2` the extended position.
    pub source: u32,
    /// `m3C4`: the aim stick's own state machine.
    pub pull: i32,
}

impl Default for Subject {
    fn default() -> Self {
        Self {
            tag: 0,
            pad: false,
            from_pad: true,
            style: i32::from(cam_style::NONE),
            aim_x: 0.0,
            aim_y: 0.0,
            zoom: 0.0,
            zoom_gain: 0.0,
            settle: 0.0,
            entry: 0.0,
            zoom_fovy: 0.0,
            fovy: 0.0,
            counter: 0,
            framed: false,
            low: false,
            source: 0,
            pull: 0,
        }
    }
}

/// One of the two view stacks `eventCamera` snapshots on its first frame,
/// `d_camera.h:168-178`: `m0A4[1]` and `m0A4[0]`, `d_camera.cpp:4735-4749`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EventView {
    pub center: Vec3,
    pub eye: Vec3,
    pub fovy: f32,
    pub bank: SAngle,
    /// `m1E`: the frame counter the stack keeps, zeroed by the snapshot.
    pub age: i16,
}

impl Default for EventView {
    fn default() -> Self {
        Self {
            center: Vec3::ZERO,
            eye: Vec3::ZERO,
            fovy: 0.0,
            bank: SAngle::ZERO,
            age: 0,
        }
    }
}

/// `dCamera_c::m118`, the frame counter `eventCamera` latches its first frame on,
/// `d_camera.h:193`, with the cut of `mEventData`, `d_camera.h:79-96`, the parts `eventCamera`
/// itself touches at `d_camera.cpp:4735-4854`.
///
/// `staff` and `action` are the seam the engine dispatch cannot fill itself: `staff` is
/// `mEventData.mStaffIdx`, the event manager's camera-play lookup, and `action` is the index its
/// action lookup hands back for that staff. Both default to `-1`, which is what retail's
/// constructor writes (`d_camera.cpp:224-225`) and what makes the guard at `:4761-4770` decline
/// when no feed is published. [`cam_event`](super::cam_event) owns both, and the guard that
/// writes them.
///
/// `trim`, `bg_check`, `auto_focus`, `move_bg` and `found` are the `mEventData` cut parameters
/// `d_camera.cpp:4801-4840` reads on a cut's first frame, and `bump_flags` is the collision word
/// `m068` the same block selects at `d_camera.cpp:4842-4855`; see
/// [`cam_event`](super::cam_event) for what consumes each.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Event {
    pub age: u32,
    /// `mEventData.mStaffIdx`.
    pub staff: i32,
    /// `mEventData.field_0x18`: the cut `eventCamera` replays when no staff is running.
    pub action: i32,
    /// `mEventData.field_0x1c`: the trim type, latched to `2` on the cut's first frame.
    pub trim: i32,
    /// `mEventData.field_0x20`: the collision flag word the cut selects.
    pub bg_check: i32,
    /// `mEventData.field_0x08`: the "this cut found what it asked for" latch the style and
    /// map-tool arms zero on their first frame, `d_camera.cpp:1925` and `:1851`.
    pub found: i32,
    /// `mEventData.field_0x24` as `WaitAnyKey`, `d_camera.cpp:4827`, which raises
    /// [`evflag::FIXED`] when it is set.
    pub wait_key: i32,
    /// `mEventData.field_0x24` reused as `AutoForcus`, `d_camera.cpp:4834`, the same member the
    /// wait key used one line earlier.
    pub auto_focus: i32,
    /// `mEventData.field_0x28`: `MoveBGCheck`, `d_camera.cpp:4839`.
    pub move_bg: i32,
    /// `dCamera_c::m068`, the collision word `eventCamera` selects off `bg_check` and
    /// `bumpCheck` then runs, `d_camera.cpp:4842-4855` and `:849`.
    pub bump_flags: i32,
    /// `m0A4[0]` and `m0A4[1]`.
    pub saved: [EventView; 2],
}

/// `eventCamera`'s action table, `d_camera.cpp:4713-4742`, which lives in
/// [`cam_event`](super::cam_event) beside the arms it selects and is re-exported here under both
/// the name it is declared with and the one this file used before it had a home of its own.
pub use super::cam_event::ACTION_NAMES;
/// The same twenty-eight entries as [`ACTION_NAMES`], the name this module first declared them
/// under.
pub const EVENT_ACTIONS: [&str; 28] = super::cam_event::ACTION_NAMES;

/// `dCamera_c::m548` through `mBankShake`, `d_camera.h:398-407`: the shake `StartShake` loads and
/// `shakeCamera` runs. The pattern is five bytes because the sixteen-bit read at
/// `d_camera.cpp:4995` walks past the four of `m548` into the `m54C` lane behind it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shake {
    /// `m548`, and the `m54C` lane the read at `d_camera.cpp:4995` reaches into.
    pub pattern: [u8; 5],
    /// `m550`: the pattern's length in bits.
    pub length: i32,
    /// `m554`: the bit the next frame reads.
    pub cursor: i32,
    /// `m55C`: the axis the offset is built on.
    pub axis: Vec3,
    /// `m568`: `mCenterShake`, the centre's offset.
    pub center: Vec3,
    /// `m574`: `mEyeShake`, the eye's offset.
    pub eye: Vec3,
    /// `m580`: `mFovYShake`, the field of view's offset.
    pub fovy: f32,
    /// `m584`: `mBankShake`, the bank's offset.
    pub bank: SAngle,
    /// `m588`: the mask `StartShake` was given.
    pub flags: i32,
    /// The camera's own generator, since `cM_rndFX` is a global the port does not carry.
    pub seed: u32,
    /// Where the `&0x20` arm last published its window-blur position, `d_camera.cpp:5041-5049`.
    pub anchor: Option<Vec3>,
}

impl Default for Shake {
    fn default() -> Self {
        Self {
            pattern: [0; 5],
            length: 0,
            cursor: 0,
            axis: Vec3::ZERO,
            center: Vec3::ZERO,
            eye: Vec3::ZERO,
            fovy: 0.0,
            bank: SAngle::ZERO,
            flags: 0,
            seed: 0x1234_5678,
            anchor: None,
        }
    }
}

/// `shakeCamera`'s wave table, `d_camera.cpp:4990`.
const WAVE: [f32; 4] = [0.4, 0.9, 2.1, 3.2];
/// `d_camera.cpp:5003`: what each missing bit takes off the weight the next one carries.
const WAVE_FALLOFF: f32 = 0.43;
/// `d_camera.cpp:5010`: the amplitude's own jitter.
const WAVE_JITTER: f32 = 0.05;
/// `d_camera.cpp:5017-5018`: the per-axis jitter on the shake axis.
const AXIS_JITTER: f32 = 0.045;
/// `d_camera.cpp:5027`: the field-of-view shake's own jitter.
const FOVY_JITTER: f32 = 0.12;
/// `d_camera.cpp:5031`: the bank shake's own jitter.
const BANK_JITTER: f32 = 0.15;
/// `d_camera.cpp:5078-5081`: the decay the exhausted-pattern arm runs on.
const SHAKE_DECAY: f32 = 0.1;
/// `d_camera.cpp:5035`: the scale the graph-blur arm would hand the renderer.
pub const BLUR_RATE_SCALE: f32 = 30.0;
/// `d_camera.cpp:5128`: `PATTERN_LENGTH_MAX << 3`, the longest pattern `StartShake` accepts.
const PATTERN_BITS_MAX: i32 = 32;

/// `cM_rndFX`, `c_math.h:28`: a uniform draw over `[-max, max]`. The camera has no access to the
/// engine's global generator, so this walks the generator [`Shake::seed`] carries.
fn rnd_fx(seed: &mut u32, range: f32) -> f32 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 17;
    *seed ^= *seed << 5;
    range * ((*seed >> 8) as f32 / 16777216.0 - 0.5) * 2.0
}

impl Shake {
    /// `dCamera_c::StartShake`, `d_camera.cpp:5118-5157`: the pattern is packed sixteen bits to
    /// a byte, the last byte masked to the bits the length leaves, and the axis normalised.
    ///
    /// The pattern is five bytes because a length of 32 masks the fifth one, which retail reads
    /// one byte past the caller's four.
    pub fn start(&mut self, length: i32, pattern: &[u8; 5], flags: i32, pos: Vec3) {
        let length = if !(0..=PATTERN_BITS_MAX).contains(&length) {
            PATTERN_BITS_MAX
        } else {
            length
        };
        self.length = length;
        self.pattern = [0; 5];
        let whole = (length >> 3) as usize;
        for lane in 0..whole.min(4) {
            self.pattern[lane] = pattern[lane];
        }
        let rest = (length & 7) as u32;
        let lane = whole.min(4);
        self.pattern[lane] = (0xFFu16 << (8 - rest)) as u8 & pattern[lane];
        if length == PATTERN_BITS_MAX {
            self.pattern[lane] |= pattern[0] >> rest;
        }
        self.axis = pos.normalize_or_zero();
        self.cursor = 0;
        self.flags = flags;
    }

    /// `dCamera_c::StopShake`, `d_camera.cpp:5160-5165`.
    pub fn stop(&mut self) {
        self.length = 0;
        self.cursor = 0;
        self.flags = 0;
    }

    /// `dCamera_c::shakeCamera`, `d_camera.cpp:4989-5115`, returning the amplitude.
    ///
    /// The wave table, the amplitude jitter, the odd-bit sign flip, the eye and centre offset,
    /// the field-of-view and bank offsets and the decay are ported. Both blur arms are not: the
    /// `&0x10` arm hands a rate to `mDoGph_gInf_c::setBlureRate` and the `&0x20` arm hands a
    /// matrix to its `onBlure`, and this port has no graph renderer to call. The `&0x20` arm's
    /// own state, the player's eye projected and divided by the scissor, still is.
    ///
    /// `anchor` is that projection, which the caller reads through [`Camera::project`] while it
    /// still holds the camera.
    pub fn run(&mut self, anchor: Option<Vec3>) -> f32 {
        if self.cursor >= self.length {
            self.center -= self.center * SHAKE_DECAY;
            self.eye -= self.eye * SHAKE_DECAY;
            self.fovy -= self.fovy * SHAKE_DECAY;
            let bank = self.bank.degree() * SHAKE_DECAY;
            self.bank -= SAngle::from_degrees(bank);
            self.flags &= !0x20;
            return 0.0;
        }

        let pair = (self.cursor >> 3) as usize;
        let bits = (i32::from(self.pattern[pair]) << 8) | i32::from(self.pattern[pair + 1]);
        let mut mask = 1 << (15 - (self.cursor & 7));
        let mut weight = 1.0;
        let mut value = 0.0f32;
        for wave in WAVE {
            if mask & bits != 0 {
                value += weight * wave;
            } else {
                weight *= WAVE_FALLOFF;
            }
            mask >>= 1;
        }
        self.cursor += 1;

        value *= rnd_fx(&mut self.seed, WAVE_JITTER) + 0.95;
        if self.cursor & 1 != 0 {
            value = -value;
        }

        let mut offset = self.axis;
        offset.x += rnd_fx(&mut self.seed, AXIS_JITTER);
        offset.z += rnd_fx(&mut self.seed, AXIS_JITTER);
        offset *= value;

        if self.flags & 2 != 0 {
            self.eye = offset;
            self.center = offset;
        }
        if self.flags & 4 != 0 {
            self.fovy = value * rnd_fx(&mut self.seed, FOVY_JITTER);
        }
        if self.flags & 8 != 0 {
            self.bank = SAngle::from_degrees(value * rnd_fx(&mut self.seed, BANK_JITTER));
        }
        if self.flags & 0x10 == 0 && self.flags & 0x20 != 0 {
            self.anchor = anchor;
        }
        value
    }
}

/// `dCamera_c::mWork`, `d_camera.h:308-380`, narrowed to the engines the port runs. The tag
/// members of each arm are left out: retail writes them and reads them back nowhere.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Work {
    pub lockon: cam_lockon::Lockon,
    pub crawl: Crawl,
    pub fixed_frame: FixedFrame,
    pub fixed_pos: FixedPos,
    pub subject: Subject,
    pub event: Event,
    /// `mWork.tower`, `mWork.hookshot`, `mWork.manual` and `mWork.ride`: the four arms of the
    /// union the port runs.
    pub tp: cam_tp_engines::State,
    /// `mWork` as the event camera sees it: the seventeen arms its own twenty-eight-entry table
    /// selects between, which the target lays over the same union the four arms above use.
    pub ev: cam_event::State,
    /// The style [`Camera::setup_age`] and [`Camera::settled`] belong to. `onStyleChange` zeroes
    /// `m11C` at `d_camera.cpp:1420` and `onModeChange` zeroes `m100` through `m102` at
    /// `d_camera.cpp:1216-1218`, which is what makes a style replay its placement.
    pub style: i16,
}

impl Default for Work {
    fn default() -> Self {
        Self {
            lockon: cam_lockon::Lockon::default(),
            crawl: Crawl::default(),
            fixed_frame: FixedFrame::default(),
            fixed_pos: FixedPos::default(),
            subject: Subject::default(),
            event: Event::default(),
            tp: cam_tp_engines::State::default(),
            ev: cam_event::State::default(),
            style: cam_style::NONE,
        }
    }
}

/// `dCamera_c::Run`'s engine call, `d_camera.cpp:796`: `engine_tbl[mCamParam.Algorythmn(style)]`,
/// which is how `CamEngine` routes. False is retail's "the engine declined", which `Run` reads
/// at `d_camera.cpp:804-806`.
///
/// The counters `Run` bumps after the call at `d_camera.cpp:797-801` are bumped here: `m118`
/// through [`Work::event`], `m11C` through [`Camera::setup_age`], and `m108` through
/// [`Camera::age`] for every engine except the follow pair, which spends its own inside
/// [`cam_follow::follow`] at `cam_follow.rs:834-835`.
pub fn engine(
    camera: &mut Camera,
    player: &Player,
    room: Option<&CamRoom>,
    input: &EngineInput<'_>,
    events: Option<&mut EventManager>,
) -> bool {
    let kind = camera.engine();
    if camera.work.style != camera.current_style {
        camera.work.style = camera.current_style;
        camera.age = 0;
        camera.setup_age = 0;
        camera.settled = false;
    }
    let result = match kind {
        Some(CamEngine::Follow) | Some(CamEngine::Follow2) => {
            cam_follow::follow(camera, player, &input.follow)
        }
        Some(CamEngine::Let) | Some(CamEngine::Demo) => true,
        Some(CamEngine::Crawl) => crawl(camera, player, room, input.collision),
        Some(CamEngine::FixedFrame) => fixed_frame(camera, player, room, input.collision),
        Some(CamEngine::FixedPosition) => fixed_position(camera, player, room),
        Some(CamEngine::Subject) => subject(camera, player, input),
        Some(CamEngine::Event) => event(camera, player, input.collision, events),
        Some(CamEngine::Lockon) => cam_lockon::lockon(camera, player, input),
        Some(CamEngine::NonOwner) => non_owner(camera),
        Some(CamEngine::Talkto) => cam_tp_engines::talkto(camera, player, input),
        Some(CamEngine::Tower) => cam_tp_engines::tower(camera, player, room, input),
        Some(CamEngine::Ride) => cam_tp_engines::ride(camera, player, input),
        Some(CamEngine::Manual) => cam_tp_engines::manual(camera, player, input),
        Some(CamEngine::Hookshot) => cam_tp_engines::hookshot(camera, player, input),
        Some(
            CamEngine::Hung
            | CamEngine::Tornado
            | CamEngine::Vomit
            | CamEngine::Shield,
        ) => false,
        None => false,
    };
    camera.work.event.age = camera.work.event.age.wrapping_add(1);
    camera.setup_age = camera.setup_age.wrapping_add(1);
    if !matches!(kind, Some(CamEngine::Follow | CamEngine::Follow2)) {
        camera.age = camera.age.wrapping_add(1);
    }
    result
}

/// `dCamera_c::crawlCamera`, `d_camera.cpp:4185-4337`. The player is the only actor in the text, so
/// `directionOf` is the player's own facing and `attentionPos` its position.
///
/// Two reads the port cannot make. `mpPlayerActor->shape_angle.x`, the player's pitch, is not
/// published here and reads as level, which is what every latitude lerp below converges on. The
/// settled arm's `lineBGCheck` at `d_camera.cpp:4296` is the only reason a blocked arrow falls
/// through to the default arm at `d_camera.cpp:4308`, so a blocked arrow is unreachable and every
/// aiming arrow takes the snap branch.
fn crawl(
    camera: &mut Camera,
    player: &Player,
    room: Option<&CamRoom>,
    collision: Option<&Collision>,
) -> bool {
    let offset = Vec3::new(
        camera.val(param::UNK1),
        camera.val(param::CENTER_HEIGHT_BASE),
        camera.val(param::UNK0),
    );
    let lerp_x = camera.val(param::UNK3);
    let lerp_y = camera.val(param::UNK4);
    let style_radius = camera.val(param::UNK10);
    let style_fovy = camera.val(param::FOVY_BASE);
    let tight = camera.has_flag(flag::UNK080);
    let behind = camera.has_flag(flag::UNK040);
    let arc = camera.crawl_arc;

    let row = room.and_then(CamRoom::cam_entry);
    let facing = SAngle::from_radians(player.face);
    let mut work = camera.work.crawl;
    let mut cache = camera.view_cache;
    let mut turn = facing - cache.direction.longitude();

    if camera.age == 0 {
        work.tight = tight;
        work.aiming = false;
        work.frames = 10;
        if arc {
            work.frames = 6;
        }
        work.tag = 0x5;
        work.budget = ((work.frames * (work.frames + 1)) >> 1) as f32;
        work.behind = if !behind {
            true
        } else {
            !(turn > SAngle::THREE_QUARTER && turn < SAngle::QUARTER)
        };
    } else if !behind {
        work.behind = true;
    } else if turn > SAngle::from_degrees(-75.0) && turn < SAngle::from_degrees(75.0) {
        work.behind = false;
    } else if turn < SAngle::from_degrees(-135.0) || turn > SAngle::from_degrees(135.0) {
        work.behind = true;
    }

    work.aiming = false;
    if behind && row.is_some_and(|row| row.field_0x12 != CAM_FIELD_UNSET) {
        if let Some(arrow) = room.and_then(CamRoom::arrow_entry) {
            turn = cache.direction.longitude() - SAngle::from(arrow.angle[1]);
            if turn > SAngle::THREE_QUARTER && turn < SAngle::QUARTER {
                work.arrow = arrow.position;
                work.aiming = true;
                camera.settled = true;
            }
        }
    }

    let anchor = cam_follow::relational_pos(player, facing, offset);

    if !camera.settled {
        work.share = (work.frames - camera.age as i32) as f32;
        let share = work.share / work.budget;
        cache.center += (anchor - cache.center) * share;

        let want_radius = if tight { 2.0 } else { style_radius };
        let current = cache.direction.radius();
        let radius = current + share * (want_radius - current);
        let current = cache.direction.latitude();
        let latitude = current + (SAngle::ZERO - current) * share;
        let side = if work.behind { facing.inv() } else { facing };
        let current = cache.direction.longitude();
        let longitude = current + (side - current) * share;

        cache.direction = SGlobe::new(radius, latitude, longitude);
        cache.eye = cache.center + cache.direction.to_xyz();
        cache.fovy += share * (style_fovy - cache.fovy);

        if camera.age >= (work.frames - 1) as u32 {
            camera.settled = true;
        }
        work.budget -= work.share;
        camera.work.crawl = work;
        camera.view_cache = cache;
        cam_follow::publish(camera);
        return true;
    }

    cache.center += (anchor - cache.center) * Vec3::new(lerp_x, lerp_y, lerp_x);

    let blocked = collision.is_some_and(|solid| solid.line_cross(cache.center, work.arrow).is_some());
    if work.aiming && !blocked {
        cache.eye = work.arrow;
        cache.direction = SGlobe::from_xyz(cache.eye - cache.center);
        camera.crawl_arc = false;
    } else {
        let side = if work.behind { facing.inv() } else { facing };
        let (want_radius, rate, turn_rate) = if work.tight {
            (2.0, 0.8, 0.8)
        } else {
            (style_radius, 0.25, 0.2)
        };
        let current = cache.direction.radius();
        let radius = current + (want_radius - current) * rate;
        let current = cache.direction.longitude();
        let longitude = current + (side - current) * turn_rate;
        let current = cache.direction.latitude();
        let latitude = current + (SAngle::ZERO - current) * 0.25;
        cache.direction = SGlobe::new(radius, latitude, longitude);
        cache.eye = cache.center + cache.direction.to_xyz();
        if work.tight {
            cache.center.y = player.pos.y + 30.0;
            camera.event_flags |= 0x800;
        }
    }
    cache.fovy += (style_fovy - cache.fovy) * 0.5;

    camera.work.crawl = work;
    camera.view_cache = cache;
    cam_follow::publish(camera);
    true
}

/// `dCamera_c::fixedFrameCamera`, `d_camera.cpp:4430-4517`. The arrow is the whole placement: the
/// centre sits on it, the eye at a globe of radius 3000 built from the arrow's first two angles,
/// and that eye is pulled onto the arrow when the player stands between it and the arrow.
///
/// The `m530` guard at `d_camera.cpp:4463` lifts the eye to a radius of 350, and `m530` is set by
/// the extended-position and ship paths this port does not carry, so that lift never runs.
fn fixed_frame(
    camera: &mut Camera,
    player: &Player,
    room: Option<&CamRoom>,
    collision: Option<&Collision>,
) -> bool {
    let Some(arrow) = room.and_then(CamRoom::arrow_entry) else {
        return false;
    };
    let span = camera.val(param::LOCKON_LONGITUDE_MAX);
    let style_fovy = camera.val(param::FOVY_BASE);
    let row = room.and_then(CamRoom::cam_entry);
    let mut work = camera.work.fixed_frame;

    if camera.setup_age == 0 {
        work.center = arrow.position;
        work.start_eye = work.center
            + SGlobe::new(
                3000.0,
                -SAngle::from(arrow.angle[0]),
                SAngle::from(arrow.angle[1]),
            )
            .to_xyz();
        work.start_center = work.start_eye;
        if let Some(nearest) = nearest_on_segment(work.start_eye, work.center, player.pos) {
            work.start_eye = nearest;
        }
        work.direction = SGlobe::from_xyz(work.center - work.start_eye);
        work.fovy = match row {
            Some(row) if row.field_0x12 != CAM_FIELD_UNSET => f32::from(row.field_0x12),
            _ => style_fovy,
        };
        work.frames = match row {
            Some(row) if row.field_0x13 != CAM_FIELD_UNSET => i32::from(row.field_0x13),
            _ => {
                let travel = (work.center - camera.bump.eye).length() / PLAYER_HEIGHT;
                (span * travel.sqrt()) as i32 + 1
            }
        };
        work.budget = ((work.frames * (work.frames + 1)) >> 1) as f32;
    }

    if !camera.settled {
        work.share = (work.frames - camera.setup_age as i32) as f32;
        let share = work.share / work.budget;
        work.budget -= work.share;

        walk_center(&mut camera.view_cache, work.start_center, share);
        walk_globes(&mut camera.view_cache, work.direction, share);
        camera.view_cache.eye = camera.view_cache.center + camera.view_cache.direction.to_xyz();
        if let Some(solid) = collision {
            if let Some(hit) = solid.line_cross(camera.view_cache.eye, camera.view_cache.center) {
                camera.view_cache.center = hit.point;
            }
        }
        let fovy = camera.view_cache.fovy;
        camera.view_cache.fovy = fovy + share * (work.fovy - fovy);

        if camera.setup_age >= (work.frames - 1) as u32 {
            camera.settled = true;
        }
    }

    camera.work.fixed_frame = work;
    cam_follow::publish(camera);
    true
}

/// `dCamera_c::fixedPositionCamera`, `d_camera.cpp:4520-4676`. With no arrow the eye is where the
/// camera already is, which is retail's own fallback at `d_camera.cpp:4573-4577`.
///
/// The `setDMCAngle` call at `d_camera.cpp:4554` is not ported: it needs the pad-driven DMC system
/// the port does not track, and nothing else in the readable text reads what it writes.
fn fixed_position(camera: &mut Camera, player: &Player, room: Option<&CamRoom>) -> bool {
    let offset = Vec3::new(
        camera.val(param::UNK1),
        camera.val(param::CENTER_HEIGHT_BASE),
        camera.val(param::UNK0),
    );
    let lerp_y = camera.val(param::UNK4);
    let lerp_x = camera.val(param::UNK3);
    let radius_far = camera.val(param::UNK10);
    let radius_near = camera.val(param::UNK11);
    let span = camera.val(param::LOCKON_LONGITUDE_MAX);
    let rate = camera.val(param::LOCKON_LONGITUDE_MIN);
    let style_fovy = camera.val(param::FOVY_BASE);
    let row = room.and_then(CamRoom::cam_entry);
    let arrow = if camera.has_flag(flag::UNK040) {
        room.and_then(CamRoom::arrow_entry)
    } else {
        None
    };
    let mut work = camera.work.fixed_pos;

    if camera.setup_age == 0 {
        work.arrow = Vec3::ZERO;
    }
    work.timed = false;

    let (position, fovy_target, named_frames) = match arrow {
        Some(arrow) => {
            work.arrow = arrow.position;
            let fovy_target = match row {
                Some(row) if row.field_0x12 != CAM_FIELD_UNSET => f32::from(row.field_0x12),
                _ => style_fovy,
            };
            let named = row
                .and_then(|row| (row.field_0x13 != CAM_FIELD_UNSET).then_some(row.field_0x13));
            work.timed = named.is_some();
            (arrow.position, fovy_target, named.map(i32::from).unwrap_or(-1))
        }
        None => (camera.bump.eye, style_fovy, -1),
    };

    if camera.setup_age == 0 {
        if work.timed {
            work.frames = named_frames;
            work.budget = work.frames as f32;
        } else {
            let reach = (position - camera.bump.eye).length().min(radius_far);
            let facing = SAngle::from_radians(player.face);
            let anchor = cam_follow::relational_pos(player, facing, offset);
            let travel = (camera.bump.center - anchor).length().max(reach);
            work.frames = (span * (travel / PLAYER_HEIGHT.max(10.0)).sqrt()) as i32 + 1;
            work.budget = ((work.frames * (work.frames + 1)) >> 1) as f32;
        }
        work.center = camera.view_cache.center;
    }

    let facing = SAngle::from_radians(player.face);
    let anchor = cam_follow::relational_pos(player, facing, offset);
    let lerp = Vec3::new(lerp_x, lerp_y, lerp_x);

    if !camera.settled {
        work.share = if work.timed {
            let share = 1.0 / work.budget;
            work.budget -= 1.0;
            share
        } else {
            work.share = (work.frames - camera.setup_age as i32) as f32;
            let share = work.share / work.budget;
            work.budget -= work.share;
            share
        };
        work.center += (anchor - work.center) * work.share;
        let center = camera.view_cache.center;
        camera.view_cache.center = center + (work.center - center) * lerp;
        walk_towards(&mut camera.view_cache, position, radius_near, radius_far, work.share);
        let fovy = camera.view_cache.fovy;
        camera.view_cache.fovy = fovy + work.share * (fovy_target - fovy);
        if camera.setup_age >= (work.frames - 1) as u32 {
            camera.settled = true;
        }
    } else {
        let center = camera.view_cache.center;
        camera.view_cache.center = center + (anchor - center) * lerp;
        walk_towards(&mut camera.view_cache, position, radius_near, radius_far, rate);
        let fovy = camera.view_cache.fovy;
        camera.view_cache.fovy = fovy + rate * (fovy_target - fovy);
    }

    camera.work.fixed_pos = work;
    camera.view_cache.eye = camera.view_cache.center + camera.view_cache.direction.to_xyz();
    cam_follow::publish(camera);
    true
}

/// `dCamera_c::subjectCamera`, `d_camera.cpp:3829-4176`: the state its first frame writes, the
/// ten frames, or seven while the aim stick is held, the placement is walked across, and then
/// the settled arm, which is [`subject_settled`].
///
/// The centre source `m3C0` is chosen on the first frame at `d_camera.cpp:3865-3869`: `1` when the
/// subject action is off and the style lacks `UNK010`, `2` under a mini-game this port does not
/// track, and `0` otherwise. The placement arm uses the longitude accessor and the settled arm
/// the latitude one, as each does in the target.
///
/// The scope's own field-of-view block inside the placement at `d_camera.cpp:3944-3997` is still
/// out: it needs `g_dComIfG_gameInfo.play.mItemScopeWipeScale`, and the centre source `2` needs
/// the extended position a scoped cutscene publishes.
fn subject(camera: &mut Camera, player: &Player, input: &EngineInput<'_>) -> bool {
    let status = input.follow.status;
    let cstick = input.cstick;
    let mut offset = Vec3::new(
        camera.val(param::UNK1),
        camera.val(param::CENTER_HEIGHT_BASE),
        camera.val(param::UNK0),
    );
    let radius = camera.val(param::UNK10);
    let fovy = camera.val(param::FOVY_BASE);
    let mut work = camera.work.subject;

    if camera.age == 0 {
        work.tag = SUBJECT_TAG;
        work.pad = false;
        work.from_pad = true;
        work.style = i32::from(camera.current_style);
        work.aim_x = 0.0;
        work.aim_y = 0.0;
        work.zoom = 0.0;
        work.counter = SUBJECT_COUNTER;
        work.framed = false;
        work.source = 0;
        work.low = player.pos.y < camera.bump.floor + 20.0;
        work.pull = if cstick.y < -0.74 { 2 } else { 0 };
        if !status.check0(player_status::STTS0_SUBJECT) && !camera.has_flag(flag::UNK010) {
            work.source = 1;
        }
    }

    if work.low {
        offset.y = 60.0;
        offset.z = -5.0;
    }

    if camera.has_flag(flag::UNK080) {
        camera.event_flags |= evflag::SUBJECT;
    }
    if camera.has_flag(flag::UNK100) {
        camera.event_flags |= evflag::SUBJECT_WIDE;
    }

    if camera.settled {
        return subject_settled(camera, player, input, work, offset, radius, fovy);
    }

    let end = if status.check0(player_status::STTS0_SUBJECT) {
        7
    } else {
        10
    };
    let share = 1.0 / ((end - camera.age as i32) as f32);
    let facing = SAngle::from_radians(player.face);

    let target = match work.source {
        1 => {
            let mut globe = SGlobe::from_xyz(offset);
            globe.set_longitude(facing + globe.longitude());
            player.pos + Vec3::new(0.0, PLAYER_HEIGHT, 0.0) + globe.to_xyz()
        }
        _ => cam_follow::relational_pos(player, facing, offset),
    };

    let center = camera.view_cache.center;
    camera.view_cache.center = center + (target - center) * share;
    walk_globes(
        &mut camera.view_cache,
        SGlobe::new(radius, SAngle::ZERO, facing.inv()),
        share,
    );
    camera.view_cache.eye = camera.view_cache.center + camera.view_cache.direction.to_xyz();

    if !camera.has_flag(flag::UNK010) {
        let held = camera.view_cache.fovy;
        camera.view_cache.fovy = held + share * (fovy - held);
    }

    if camera.age == (end - 1) as u32 {
        camera.settled = true;
        work.aim_x = 0.0;
        work.aim_y = 0.0;
        work.zoom = 0.0;
        work.zoom_fovy = camera.view_cache.fovy;
        work.entry = 0.0;
        work.zoom_gain = 0.0;
    }

    camera.work.subject = work;
    cam_follow::publish(camera);
    true
}

/// `dCamera_c::subjectCamera`'s settled arm, `d_camera.cpp:4003-4176`.
///
/// The angle pair comes from [`subject_angle`], which is `CalcSubjectAngle`, and it is reachable
/// here because the pitch it measures, `m3B8`, is written at `:3737` and read by nothing while
/// `m3BA` cancels against the `+ m3BA` the return at `:3820-3822` adds. `m37D` is `1` on every
/// frame the settled arm runs, `:4157` re-arming it after `:4017` sets it, so the actor-shape arm
/// at `:4021-4032` is dead code in the target and the pad arm always takes it.
///
/// The three smoothing arms at `:4058-4110` are chosen on `mEventFlags`'s subject-lock bit and on
/// `mLockOnActorId`. The port has neither the bit, which only `SubjectLockOn` and
/// `SubjectLockOff` set, nor the forced-lock identity, so the first arm's guard
/// `((mEventFlags >> 0x19) & 1) == 0 && (mLockOnActorId == -1)` always holds and the frame aims
/// at the lock-on target. That arm dereferences `mpLockonTarget` unguarded, as the target's own
/// text does, so with no target there is nothing to aim at and the engine declines rather than
/// invent one.
///
/// The second arm at `:4077` needs `mExtendedPos`, the position a scoped cutscene publishes
/// through `SetExtendedPosition`, `d_camera.cpp:5262-5268`, and the third at `:4090` needs the
/// arm above, so the engine declines outright once [`evflag::SUBJECT_LOCK`] is set, the only way
/// into either. The centre source `2`, the mini-game path at `:4044-4047`, declines for the same
/// reason, as does its placement counterpart at `:3917-3921`. The viewport block at `:4007-4014`
/// needs the game clock, the `setComStat` calls have no port, and the scope's `setComZoomScale` /
/// `setComZoomForcus` / auto-focus at `:4140-4148` are the zoom block's only remaining effect, so
/// the field of view moves and the view scale does not.
fn subject_settled(
    camera: &mut Camera,
    player: &Player,
    input: &EngineInput<'_>,
    mut work: Subject,
    offset: Vec3,
    radius: f32,
    fovy: f32,
) -> bool {
    let status = input.follow.status;
    let facing = SAngle::from_radians(player.face);
    let base_yaw = facing.inv();
    let wide_angle = camera.val(param::LOCKON_LONGITUDE_MAX);
    let tall_angle = camera.val(param::LOCKON_LATITUDE_MAX);
    let rate = camera.val(param::UNK20);

    if work.from_pad {
        work.pad = true;
        subject_angle(camera, input, &mut work);
    }
    let ang_x = SAngle::from_degrees(tall_angle) * work.aim_y;
    let ang_y = SAngle::from_degrees(wide_angle) * work.aim_x;

    if camera.event_flags & evflag::SUBJECT_LOCK != 0 || work.source == 2 {
        camera.work.subject = work;
        return false;
    }

    let Some(lockon) = input.lockon.target else {
        camera.work.subject = work;
        return false;
    };

    let mut cache = camera.view_cache;
    let rotated = rotate_y(rotate_x(offset, ang_x), ang_y);
    cache.center = if work.source == 1 {
        let mut globe = SGlobe::from_xyz(rotated);
        globe.set_latitude(facing + globe.longitude());
        player.pos + Vec3::new(0.0, PLAYER_HEIGHT, 0.0) + globe.to_xyz()
    } else {
        cam_follow::relational_pos(player, facing, rotated)
    };

    let want = SGlobe::from_xyz(
        cache.center - Vec3::new(lockon.x, lockon.y - SUBJECT_DROP, lockon.z),
    );
    let current = cache.direction.radius();
    cache
        .direction
        .set_radius(current + (radius - current) * 0.04);
    let current = cache.direction.longitude();
    cache
        .direction
        .set_longitude(current + (want.longitude() - current) * 0.04);
    let current = cache.direction.latitude();
    cache
        .direction
        .set_latitude(current + (want.latitude() - current) * 0.04);

    let yaw = cache.direction.longitude();
    work.aim_x = (yaw - base_yaw).degree() / wide_angle;
    work.aim_y = yaw.degree() / tall_angle;
    work.counter = 0;
    work.pad = false;

    cache.eye = cache.center + cache.direction.to_xyz();

    if camera.has_flag(flag::UNK010) {
        let ease = cam_follow::rational_bezier_ratio(-input.cstick.y, CURVE_WEIGHT);
        let next = work.zoom + camera.val(param::FOVY_UPPER) * (0.0 - ease) * 0.1;
        work.zoom = next.clamp(0.0, 1.0);
        let zoomed = zoom_fovy(work.zoom_fovy * 0.5, work.zoom * 8.0 + 1.0) * 2.0;
        cache.fovy += rate * (zoomed - cache.fovy);
    } else {
        cache.fovy += rate * (fovy - cache.fovy);
    }

    work.from_pad = true;
    if status.check0(player_status::STTS0_SUBJECT) {
        if work.pull == 2 && input.cstick.y >= -0.001 {
            work.pull = 0;
        } else if work.pull == 1 && input.cstick.y < -0.74 {
            work.pull = 2;
        } else if input.cstick.y < -0.001 {
            work.pull = 1;
        } else {
            work.pull = 0;
        }
    }

    camera.work.subject = work;
    camera.view_cache = cache;
    cam_follow::publish(camera);
    true
}

/// `dCamera_c::CalcSubjectAngle`, `d_camera.cpp:3692-3827`: the aim pair `m384` and `m388` the
/// settled arm rotates the eye offset by, as the fraction of the two style angles the raw angle
/// spans. `m384` is scaled by slot 24 and `m388` by slot 19 on the way out at `:3819-3822`, and
/// the settled arm divides them back the same way at `:4068-4069`.
///
/// The three gates at `:3704-3714` are the camera play, the `'SUBJ'` tag and `m100`; the caller
/// has already cleared the first and holds the other two. The `!m37C` arm at `:3725-3731` writes
/// the two outputs from the stale pair and then falls through to the live computation, which
/// writes them again at `:3819-3822`, so nothing is lost by leaving it out. `m3B8`, the pitch read
/// at `:3736-3741`, is written and read by nothing, and `m3BA` is `shape_angle.y`, the player's
/// facing, which cancels against the `+ m3BA` the return adds: neither is carried.
fn subject_angle(camera: &Camera, input: &EngineInput<'_>, work: &mut Subject) {
    let status = input.follow.status;
    let style = work.style as i16;
    let row = cam_style::style_by_id(style);
    let gate = cam_style::val_by_id(style, param::LOCKON_FOVY_MAX);
    let open = if status.check0(player_status::STTS0_SUBJECT) {
        gate
    } else if row.is_some_and(|s| s.flag & flag::UNK010 != 0) {
        gate - work.zoom * (gate * SUBJECT_CUSHION)
    } else {
        gate
    };

    let stick_x = cam_follow::limitf(input.cstick.x / 0.7, -1.0, 1.0);
    let mut stick_y = cam_follow::limitf(input.cstick.y / 0.7, -1.0, 1.0);

    if (camera.bump.eye.y <= camera.bump.floor + 5.0
        || camera.bump.eye.y <= camera.bump.head_ground + 5.0)
        && ((work.aim_y >= 0.0 && stick_y > 0.0) || (work.aim_y < 0.0 && stick_y <= 0.0))
    {
        stick_y = 0.0;
    }
    if status.check0(player_status::STTS0_CRAWL) {
        stick_y = 0.0;
    }

    if status.check0(player_status::STTS0_UNK40000) {
        work.aim_x = 0.0;
    } else if row.is_some_and(|s| s.flag & flag::UNK020 != 0) {
        work.aim_x = -stick_x;
        work.aim_y = stick_y;
    } else {
        let ease_x = cam_follow::rational_bezier_ratio(stick_x, CURVE_WEIGHT);
        let ease_y = cam_follow::rational_bezier_ratio(stick_y, CURVE_WEIGHT);
        work.aim_x = (-ease_x * open).clamp(-1.0, 1.0);
        work.aim_y = (work.aim_y + ease_y * open).clamp(-1.0, 1.0);
    }

    work.from_pad = false;
}

/// `dCamera_c::eventCamera`, `d_camera.cpp:4679-4885`: twenty-eight named cutscene actions read
/// off the running cutscene's camera staff, through
/// [`cam_event::event`](super::cam_event::event).
///
/// The whole body, the two view stacks the first frame fills at `d_camera.cpp:4744-4756`, the
/// two-branch guard at `:4761-4794`, the trim and collision block at `:4801-4855`, the
/// twenty-eight-way dispatch at `:4871` and the trailing line check at `:4876-4881`, is
/// [`cam_event::event`].
///
/// The only thing this arm withholds is the event manager, which the guard's first branch reads
/// the camera play, the `CAMERA` staff and the cut's action name off. [`EngineInput`] is built by
/// a struct literal in the camera's own track pass, so it has no room for one; with `None` the
/// guard declines and the engine settles, which is what it did before [`cam_event`] existed.
/// Passing the manager is the one wiring change needed to make the table live.
fn event(
    camera: &mut Camera,
    player: &Player,
    collision: Option<&Collision>,
    manager: Option<&mut EventManager>,
) -> bool {
    cam_event::event(camera, player, collision, manager)
}

/// `dCamera_c::nonOwnerCamera`, `d_camera.cpp:4375-4427`: the placement follows whichever actor
/// holds the camera canon, which the port does not track, so the guard at
/// `d_camera.cpp:4381-4383` holds and the engine declines.
fn non_owner(camera: &mut Camera) -> bool {
    let _ = camera;
    false
}

impl Camera {
    /// `dCamera_c::Run`'s engine call at `d_camera.cpp:796`, through [`engine`].
    pub fn run_engine(
        &mut self,
        player: &Player,
        room: Option<&CamRoom>,
        input: &EngineInput<'_>,
        events: Option<&mut EventManager>,
    ) -> bool {
        engine(self, player, room, input, events)
    }

    /// `dCamera_c::Run`'s `shakeCamera` at `d_camera.cpp:812`: once a frame, after the engine and
    /// before the collision pass. The four offsets stay on [`Camera::shake`] for the renderer.
    pub fn shake_pass(&mut self) {
        let anchor = if self.shake.flags & 0x10 == 0 && self.shake.flags & 0x20 != 0 {
            self.project(self.view_cache.eye).map(|proj| {
                Vec3::new(
                    proj.x / self.port.width,
                    proj.y / self.port.height,
                    0.0,
                )
            })
        } else {
            None
        };
        self.shake.run(anchor);
    }
}

/// `dCamMath::xyzRotateX`, `d_cam_param.cpp:71-79`, which is `cMtx_multVec` against the matrix
/// `mDoMtx_XrotS` builds at `m_Do_mtx.cpp:87-104`: `y' = c*y - s*z`, `z' = s*y + c*z`.
fn rotate_x(v: Vec3, angle: SAngle) -> Vec3 {
    let (sin, cos) = (angle.sin(), angle.cos());
    Vec3::new(v.x, cos * v.y - sin * v.z, sin * v.y + cos * v.z)
}

/// `dCamMath::xyzRotateY`, `d_cam_param.cpp:81-89`, against `mDoMtx_YrotS` at `m_Do_mtx.cpp:116-133`:
/// `x' = c*x + s*z`, `z' = -s*x + c*z`.
fn rotate_y(v: Vec3, angle: SAngle) -> Vec3 {
    let (sin, cos) = (angle.sin(), angle.cos());
    Vec3::new(cos * v.x + sin * v.z, v.y, -sin * v.x + cos * v.z)
}

/// `dCamMath::zoomFovy`, `d_cam_param.cpp:61-64`: the angle off the horizon whose tangent is the
/// base angle's tangent over the zoom, in degrees.
fn zoom_fovy(degrees: f32, scale: f32) -> f32 {
    let base = SAngle::from_degrees(degrees);
    base.sin().atan2(base.cos() * scale).to_degrees()
}

/// `cM3d_Len3dSqPntAndSegLine`, `c_m3d.cpp:69-91`: the point of the segment nearest `pos`, and
/// only when `pos` projects inside the segment.
fn nearest_on_segment(start: Vec3, end: Vec3, pos: Vec3) -> Option<Vec3> {
    let axis = end - start;
    let square = axis.length_squared();
    if square <= 3.8146973e-06 {
        return None;
    }
    let t = (pos - start).dot(axis) / square;
    if !(0.0..=1.0).contains(&t) {
        return None;
    }
    Some(start + axis * t)
}

/// `d_camera.cpp:4496`: the centre walked onto its target by one share of the placement.
fn walk_center(cache: &mut ViewCache, target: Vec3, share: f32) {
    cache.center += (target - cache.center) * share;
}

/// `d_camera.cpp:4498-4500`: the three globe members walked onto their targets by one share.
fn walk_globes(cache: &mut ViewCache, want: SGlobe, share: f32) {
    let current = cache.direction;
    cache.direction = SGlobe::new(
        current.radius() + (want.radius() - current.radius()) * share,
        current.latitude() + (want.latitude() - current.latitude()) * share,
        current.longitude() + (want.longitude() - current.longitude()) * share,
    );
}

/// `d_camera.cpp:4634-4645`: the eye's globe off a fixed position, held between the style's two
/// radii and then walked onto by one share.
fn walk_towards(cache: &mut ViewCache, position: Vec3, near: f32, far: f32, share: f32) {
    let mut globe = SGlobe::from_xyz(position - cache.center);
    globe.set_radius(globe.radius().max(near).min(far));
    walk_globes(cache, globe, share);
}