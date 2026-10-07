use glam::{Vec2, Vec3};

use super::cam_follow::{self, FollowInput, SAngle, SGlobe, ViewCache};
use super::cam_room::{CamRoom, CAM_FIELD_UNSET};
use super::cam_style::{self, flag, param, CamEngine};
use super::camera::Camera;
use super::collision::Collision;
use super::player::Player;
use super::player_status::{self, PlayerStatus};

/// The player's `mHeight`, which `getHeight` answers for a player at `d_camera.cpp:1988-1994`
/// and which the two fixed-placement engines divide their travel by.
const PLAYER_HEIGHT: f32 = 125.0;

/// What one engine call reads besides the camera: the pad the follow and subject engines want,
/// and the stage collision the arrow engines line-check against.
pub struct EngineInput<'a> {
    pub follow: FollowInput<'a>,
    /// `CPad_GET_STICK_POS_X/Y`, the subject engine's aim and zoom stick.
    pub cstick: Vec2,
    pub collision: Option<&'a Collision>,
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

/// `dCamera_c::mWork.subject`, `d_camera.h:325-349`, narrowed to the members the placement arm at
/// `d_camera.cpp:3851-4000` reads and writes. The angle pair `m3B8` / `m3BA` and the settle term
/// `m37C` belong to `CalcSubjectAngle` and to the settled arm, which need a lock-on target.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Subject {
    /// `m37D`: which of the two angle sources the settled arm reads.
    pub from_pad: bool,
    /// `m380`: the style the placement belongs to.
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
            framed: false,
            low: false,
            source: 0,
            pull: 0,
        }
    }
}

/// `dCamera_c::m118`, the frame counter `eventCamera` latches its first frame on,
/// `d_camera.h:193`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Event {
    pub age: u32,
}

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

/// `dCamera_c::mWork`, `d_camera.h:308-380`, narrowed to the engines the port runs. The lock-on
/// union arm and the tag members are left out: both engines decline without an actor the sim
/// does not publish.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Work {
    pub crawl: Crawl,
    pub fixed_frame: FixedFrame,
    pub fixed_pos: FixedPos,
    pub subject: Subject,
    pub event: Event,
    /// The style [`Camera::setup_age`] and [`Camera::settled`] belong to. `onStyleChange` zeroes
    /// `m11C` at `d_camera.cpp:1420` and `onModeChange` zeroes `m100` through `m102` at
    /// `d_camera.cpp:1216-1218`, which is what makes a style replay its placement.
    pub style: i16,
}

impl Default for Work {
    fn default() -> Self {
        Self {
            crawl: Crawl::default(),
            fixed_frame: FixedFrame::default(),
            fixed_pos: FixedPos::default(),
            subject: Subject::default(),
            event: Event::default(),
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
        Some(CamEngine::Subject) => subject(camera, player, input.follow.status, input.cstick),
        Some(CamEngine::Event) => event(camera),
        Some(CamEngine::Lockon) => lockon(camera),
        Some(CamEngine::NonOwner) => non_owner(camera),
        Some(
            CamEngine::Talkto
            | CamEngine::Tower
            | CamEngine::Ride
            | CamEngine::Hung
            | CamEngine::Manual
            | CamEngine::Hookshot
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
            camera.bump.flags |= 0x800;
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

/// `dCamera_c::subjectCamera`'s placement arm, `d_camera.cpp:3829-4001`: the state its first frame
/// writes and the ten frames, or seven while the aim stick is held, the centre, the globe and the
/// field of view are walked across.
///
/// The settled arm at `d_camera.cpp:4003-4176` is not ported. Every branch of it aims at
/// `mpLockonTarget` or reads `mExtendedPos`, and the port carries neither: it needs a lock-on
/// actor with its attention position and its shape angles, plus the extended position a scoped
/// cutscene publishes. The scope's own field-of-view block at `d_camera.cpp:3944-3997` and the
/// framed viewport at `d_camera.cpp:4007-4014` need the scope wipe scale and the clock. The engine
/// therefore declines once the placement has finished, which leaves the published placement to the
/// collision pass.
fn subject(camera: &mut Camera, player: &Player, status: &PlayerStatus, cstick: Vec2) -> bool {
    let mut offset = Vec3::new(
        camera.val(param::UNK1),
        camera.val(param::CENTER_HEIGHT_BASE),
        camera.val(param::UNK0),
    );
    let radius = camera.val(param::UNK10);
    let fovy = camera.val(param::FOVY_BASE);
    let mut work = camera.work.subject;

    if camera.age == 0 {
        work.from_pad = true;
        work.style = i32::from(camera.current_style);
        work.aim_x = 0.0;
        work.aim_y = 0.0;
        work.zoom = 0.0;
        work.entry = 100.0;
        work.framed = false;
        work.source = 0;
        work.low = player.pos.y < camera.bump.floor + 20.0;
        work.pull = if cstick.y < -0.74 { 2 } else { 0 };
    }

    if work.low {
        offset.y = 60.0;
        offset.z = -5.0;
    }

    if camera.has_flag(flag::UNK080) {
        camera.bump.flags |= 0x800;
    }
    if camera.has_flag(flag::UNK100) {
        camera.bump.flags |= 0x1000_0000;
    }

    if camera.settled {
        camera.work.subject = work;
        return false;
    }

    let end = if status.check0(player_status::STTS0_SUBJECT) {
        7
    } else {
        10
    };
    let share = 1.0 / ((end - camera.age as i32) as f32);
    let facing = SAngle::from_radians(player.face);

    let target = if work.source == 1 {
        let mut globe = SGlobe::from_xyz(offset);
        globe.set_longitude(facing + globe.longitude());
        player.pos + Vec3::new(0.0, PLAYER_HEIGHT, 0.0) + globe.to_xyz()
    } else {
        cam_follow::relational_pos(player, facing, offset)
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

/// `dCamera_c::eventCamera`, `d_camera.cpp:4679-4885`: twenty-eight named cutscene actions read
/// off the running cutscene's camera staff. The port has no cutscene feed and no staff index, so
/// the guard at `d_camera.cpp:4761-4770`, which asks for a camera play and then for that play's
/// camera staff, always holds and the engine declines, leaving the placement to the collision pass.
fn event(camera: &mut Camera) -> bool {
    let _ = camera;
    false
}

/// `dCamera_c::lockonCamera`, `d_camera.cpp:3291-3640`. The function is framed on
/// `mpLockonTarget` throughout: its attention position sets the globe at
/// `d_camera.cpp:3366-3374`, the distance between the two actors scales the style's parameters at
/// `d_camera.cpp:3375`, and the sight tests at `d_camera.cpp:3412-3434` and `:3491-3496` need an
/// actor to look at. The port publishes no lock-on actor and no attention feed, so the engine
/// declines rather than inventing one.
fn lockon(camera: &mut Camera) -> bool {
    let _ = camera;
    false
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
    ) -> bool {
        engine(self, player, room, input)
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