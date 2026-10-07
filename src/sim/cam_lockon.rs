use glam::Vec3;

use super::cam_engine::EngineInput;
use super::cam_follow::{self, SAngle, SGlobe};
use super::cam_style::param;
use super::camera::Camera;
use super::player::Player;
use super::player_status::{self, PlayerStatus};

/// The `dCamera_c::mEventFlags` bits `lockonCamera` reads or writes, `d_camera.cpp:3291-3640`.
///
/// Retail keeps them in the one event word beside the collision pass's own four. This port puts
/// every bit below on [`Camera::event_flags`](super::camera::Camera::event_flags) except
/// [`SIGHT`], whose two bits the collision pass is the only writer of, which is where
/// [`cam_bump::Bump::flags`] still holds them.
pub mod flag {
    /// `setFlag(0x10)` at `d_camera.cpp:1240`, read at `:3439` and `:3576`: the player is
    /// aiming, which pins both cushions and the latitude rate.
    pub const AIM: u32 = 0x10;
    /// `setFlag(0x100)` at `d_camera.cpp:3395` and cleared at `:3352`: the placement has held
    /// long enough that the longitude span stops being scaled over the first sixty frames.
    pub const ENTERED: u32 = 0x100;
    /// `setFlag(0x2000)` at `d_camera.cpp:3638`, read by the collision pass at `:2119` for the
    /// locked-on sight radius, which this port does not run.
    pub const LOCKED: u32 = 0x2000;
    /// `setFlag(0x4000000)` at `d_camera.cpp:3563`: a charge pull-in is running.
    pub const CHARGE: u32 = 0x4000_000;
    /// `chkFlag(0x80080)` at `d_camera.cpp:3412`: the player must be inside the window. Both
    /// bits are the collision pass's own, `setFlag(0x80)` at `:2235` and `setFlag(0x80000)` at
    /// `:2225`, so this mask reads [`cam_bump::Bump::flags`] and no other word.
    pub const SIGHT: u32 = 0x80080;
}

/// `dCamera_c::mWork.lockon`, `d_camera.h:308-324`, narrowed to the members the engine reads
/// back. The padding between `m378` and `m380` is left out, and so is `m38C`, which the first
/// frame zeroes at `d_camera.cpp:3300` and which no line of the readable text reads back.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lockon {
    /// `m378`: the `'LOCK'` mark retail stamps on the arm when its first frame runs.
    pub tag: i32,
    /// `m380`: the frames a charge pull-in has been held for, which spends the pull.
    pub charge_frames: i32,
    /// `m384`: the gain the unobstructed radius arm multiplies its target by, `1.0` on the
    /// first frame.
    pub scale: f32,
    /// `m388`: the frames the sight-block latch still holds.
    pub blocked: i32,
    /// `m390`: the point the centre is built from, the player's own attention point walked up
    /// by the cushion.
    pub anchor: Vec3,
    /// `m3A0`: which side of the target the camera wants to sit on.
    pub side: i32,
    /// `m3A4`: the side the latch recorded when it first engaged, which the obstructed yaw
    /// chase nudges against.
    pub swap: i32,
    /// `m3A8`: the eye relative to the anchor, kept as a globe so the walk stays on the arc.
    pub eye: SGlobe,
    /// `m3B0`: the eased gain toward the style's `UNK4` the radius arm runs on.
    pub radius_gain: f32,
    /// `m3B4`: the eased gain toward the style's `UNK3` the globe's two angles run on.
    pub latitude_gain: f32,
    /// `m3B8`: the eased cushion the anchor's height is walked with.
    pub cushion: f32,
    /// `m39C`: the latch `!LockonTruth() && STTS0_BOOMERANG_WAIT` sets at `:3325-3330`.
    /// Nothing in the target's readable text reads it back: the sibling's copy-rod arm at
    /// `sibling d_camera.cpp:4680-4707` does, and the target dropped that whole block.
    pub waiting: bool,
    /// `m39D`: the latch `STTS0_UNK400` holds, so the `StartShock` at `:3339` fires once
    /// per press. The latch is ported; the vibration has no port.
    pub shocked: bool,
}

impl Default for Lockon {
    fn default() -> Self {
        Self {
            tag: 0,
            charge_frames: 0,
            scale: 0.0,
            blocked: 0,
            anchor: Vec3::ZERO,
            side: 0,
            swap: 0,
            eye: SGlobe::new(0.0, SAngle::ZERO, SAngle::ZERO),
            radius_gain: 0.0,
            latitude_gain: 0.0,
            cushion: 0.0,
            waiting: false,
            shocked: false,
        }
    }
}

/// `mCamSetup.m0B0`, `d_cam_param.cpp:341`: the frames the longitude span is scaled over and
/// the divisor of the latitude rate's own ramp.
const LOCK_FRAMES: f32 = 60.0;
/// `mCamSetup.m0B4`, `d_cam_param.cpp:342`: the rate the span ramp starts from and decays off.
const RAMP_RATE: f32 = 0.15;
/// `mCamSetup.m028`, `d_cam_param.cpp:312`: the settle rate every unblocked term runs on.
const SETTLE: f32 = 0.33;
/// `mCamSetup.mCusCus`, `d_cam_param.cpp:309`: the easing the two gains and the cushion run on.
const GAIN_CUSHION: f32 = 0.2;
/// `mCamSetup.mBaseCushion`, `d_cam_param.cpp:317`: the centre's vertical cushion on the floor.
const CUSHION_BASE: f32 = 0.28;
/// `mCamSetup.mJumpCushion`, `d_cam_param.cpp:318`: the same while the player is in the air.
const CUSHION_JUMP: f32 = 1.0;
/// `mCamSetup.mParallelDist`, `d_cam_param.cpp:326`: the globe's radius with no target.
const PARALLEL_DIST: f32 = 60.0;
/// `mCamSetup.mCurveWeight`, `d_cam_param.cpp:322`: the ease on every ratio call below.
const CURVE_WEIGHT: f32 = 1.0;
/// `dAttention_c::LockonReleaseDistanse`, `d_attention.cpp:1007-1022`: the release distance
/// the two actors' separation is divided by. Retail picks the row by the target's own
/// attention type and widens it by the angle off the player's facing; the port carries no
/// attention type, so this is the row's own distance and nothing else, which is the same
/// figure the lock list measures against.
const RELEASE: f32 = 200.0;
/// `d_camera.cpp:3298`, `mWork.lockon.m378 = 'LOCK'`.
const TAG: i32 = i32::from_be_bytes(*b"LOCK");
/// `d_camera.cpp:3293`, `r28`: the frames the sight-block latch holds before it clears.
const BLOCK_FRAMES: i32 = 60;
/// `d_camera.cpp:3294`, `f30`: the gain on the latitude the obstructed arm eases onto, the
/// one rate in the function that is not the setup's.
const BLOCK_GAIN: f32 = 0.05;
/// `d_camera.cpp:3441`: the centre-height cushion the aiming flag pins to.
const AIM_CUSHION: f32 = 0.01;
/// `d_camera.cpp:3577`: the rate the aiming flag's latitude converges on the style's own at.
/// It is not the cushion's figure: the sibling reference has no arm here at all, so this is the
/// target's own and the two differ.
const AIM_LATITUDE: f32 = 0.05;
/// `d_camera.cpp:3509` and `:3512`: the nudge either side of straight behind while blocked.
const BLOCK_NUDGE: f32 = 15.0;
/// `d_camera.cpp:3626`: the radius the obstructed arm walks the placement onto.
const BLOCK_RADIUS: f32 = 280.0;
/// `d_camera.cpp:3455`: the extra centre height the obstructed arm adds.
const BLOCK_HEIGHT: f32 = 25.0;
/// `d_camera.cpp:3523`: the rate for a style whose longitude span came out zero.
const ZERO_SPAN_RATE: f32 = 0.15;
/// `d_camera.cpp:3540`: the ceiling on the residual divisor, in degrees.
const RESIDUAL_CAP: f32 = 45.0;
/// `d_camera.cpp:3541`: past this span the ceiling runs down from the span itself.
const SPAN_RUN: f32 = 135.0;
/// `d_camera.cpp:3549`: under this horizontal separation the rate is scaled by its ratio.
const CLOSE_RANGE: f32 = 100.0;
/// `d_camera.cpp:3430`: the main stick at or below this leaves the latch set for a frame.
const LATCH_STICK: f32 = 0.1;

/// `dCamera_c::lockonCamera(s32 style)`, `d_camera.cpp:3291-3640`.
///
/// The function is framed on the lock-on target throughout: its attention point becomes the
/// globe the eye rides (`d_camera.cpp:3370-3374`), the separation of the two actors scales
/// every style lookup (`d_camera.cpp:3375`), the sight tests at `:3412-3421` and `:3423-3426`
/// need an actor to look at, and the frame it opens by latching the target's own state.
///
/// This port has the target, and `EngineInput::lockon` carries it. Everything the readable
/// text reaches through an actor beyond that point is listed in the notes below; none of it
/// stands in the way of the placement itself, which is the whole of the body here.
///
/// Three members of the arm are now carried that the first draft left out: the `'LOCK'` tag
/// `m378` takes, `m39C`'s latch and `m39D`'s, written at `:3298`, `:3330` and `:3339` from the
/// pad states the port already publishes.
///
/// The lock-on target the port publishes is `positionOf` and not `attentionPos`, so the boss arm
/// below and the globe at `:3370-3374` are reading the same point retail reads the second of.
/// Nothing downstream distinguishes them.
///
/// What is not ported, in the order the reference reaches it:
///
/// - `d_camera.cpp:3323-3325`: `radiusActorInSight(mpPlayerActor, mpLockonTarget)`, whose
///   value the function discards. It needs the window scissor and the two actors' attention
///   points, and it feeds nothing else in the readable text.
/// - `d_camera.cpp:3328-3330`: `m31D`, the moving-background carry. Nothing in the port
///   publishes a moving background.
/// - `d_camera.cpp:3338-3346`: the `StartShock` on `UNK400`. `m39D`'s latch is carried as
///   [`Lockon::shocked`] and written every frame, so only the vibration itself is missing; the
///   rising edge the reference gates it on is the port's one-frame `!shocked` read.
/// - `d_camera.cpp:3363`: the parallel arm clears the camera's own target. The target arrives
///   read-only, and retail's other three writers of it are the mode and type choosers this
///   port does not run, so the arm is taken on its own reading and the clearing is left to
///   them.
/// - `d_camera.cpp:3369-3372`: the boss arm that takes the target's own position instead of
///   its attention point, which the port has no second position for.
/// - `d_camera.cpp:3491-3496`: the forced lock-off on a blocked centre, which needs the
///   forced-lock identity retail keeps in `mLockOnActorId`.
/// - `d_camera.cpp:3583-3592`: the boomerang arm of the latitude, which needs the player's
///   thrown boomerang and the map test in `m784`.
/// - `d_camera.cpp:3596-3603`: the separation the `UNK400` arm measures into `fVar22`. The
///   sibling writes the same member as the sight radius at `sibling d_camera.cpp:4702` and its
///   copy-rod arm reads it back at `:4690`; the target dropped that arm, so the value lands in a
///   member whose only later reader is the radius arm's own register reuse.
///
/// One accessor does not read the way the arm around it suggests. At `d_camera.cpp:3580`
/// the airborne arm eases the latitude toward `local_240.U()`, which under this port's
/// `cSGlobe` spelling (`c_angle.h:165-172`, where the getters and the setters disagree) is
/// the inclination rather than the latitude, and the difference is weighted by the latitude's
/// own cosine. It is taken as written: the port subtracts the inclination from the latitude
/// there, which is the one line of this engine whose shape rests on how the reference spells
/// its accessors.
///
/// The radius arm at `d_camera.cpp:3604-3612` is the one place the target's own text cannot be
/// taken literally. `DefaultRadius` leaves its radius clamped when it returns false, and the
/// true branch then reads a `fVar22` the reference has recycled: the function's opening
/// `mCamSetup.m0B4`, the narrow-span rate at `:3530` and the `UNK400` separation above all land
/// there. The sibling's matching arm is readable and has a different shape again, easing toward
/// the style's own near/far radius at two percent inside the band and toward the clamped radius
/// at `0.4 * sp194` outside it (`sibling d_camera.cpp:5010-5016`), and the `sp194` ramp it scales
/// by has no counterpart in the target at all. This port keeps the target's two-branch shape and
/// its 0.05: inside the band the radius holds, outside it the radius eases onto the clamp.
pub fn lockon(camera: &mut Camera, player: &Player, input: &EngineInput<'_>) -> bool {
    let status: &PlayerStatus = input.follow.status;
    let target = input.lockon.target;
    let attention = player.pos;
    let facing = SAngle::from_radians(player.face);
    let mut work = camera.work.lockon;
    let mut cache = camera.view_cache;
    let mut flags = camera.event_flags;

    if camera.age == 0 {
        camera.settled = true;
        work.tag = TAG;
        work.charge_frames = 0;
        work.scale = 1.0;
        work.blocked = 0;
        work.anchor = cache.center;
        work.eye = SGlobe::from_xyz(cache.center - attention);
        work.radius_gain = 0.0;
        work.latitude_gain = 0.0;
        work.cushion = CUSHION_BASE;
        work.side = 0;
        work.swap = 0;
    }

    work.waiting = !input.lockon.truth && status.check0(player_status::STTS0_BOOMERANG_WAIT);
    work.shocked = status.check0(player_status::STTS0_UNK400);

    if input.lockon.focus {
        camera.setup_age = 0;
        camera.age = 0;
        flags &= !flag::ENTERED;
    }

    let release = if input.lockon.truth { RELEASE } else { 0.0 };
    let mut range = 10000.0;
    let (globe, ratio) = if status.check0(
        player_status::STTS0_UNK40 | player_status::STTS0_UNK20 | player_status::STTS0_UNK1,
    ) {
        (
            SGlobe::new(PARALLEL_DIST, SAngle::ZERO, facing.inv()),
            1.0,
        )
    } else if let Some(at) = target {
        range = cam_follow::horizontal_distance(at, attention) as f32;
        let globe = SGlobe::from_xyz(at - attention);
        let ratio = if globe.radius() / release > 1.0 {
            1.0
        } else {
            globe.radius() / release
        };
        (globe, ratio)
    } else {
        (SGlobe::new(PARALLEL_DIST, SAngle::ZERO, facing), 1.0)
    };

    let mut span = lockon_longitude(camera, ratio);
    if camera.setup_age < LOCK_FRAMES as u32 && flags & flag::ENTERED == 0 {
        span *= camera.age as f32 / LOCK_FRAMES;
    } else if camera.setup_age >= LOCK_FRAMES as u32 {
        flags |= flag::ENTERED;
    }

    let mut aim = globe.longitude();
    let mut turn = cache.direction.longitude().inv() - aim;
    if turn < SAngle::ZERO {
        work.side = 0;
        aim -= span;
        turn = -aim;
    } else {
        work.side = 1;
        aim += span;
    }

    let mut blocked = false;
    if camera.bump.flags & flag::SIGHT != 0 && !point_in_sight(camera, attention) {
        if work.blocked == 0 {
            work.swap = i32::from(work.side != 1);
        }
        blocked = true;
        work.blocked = BLOCK_FRAMES;
    }
    if input.collision.is_some_and(|solid| {
        solid.line_cross(cache.center, attention).is_some()
            && solid.line_cross(cache.eye, cache.center).is_some()
    }) {
        blocked = true;
        work.blocked = BLOCK_FRAMES;
    }
    if work.blocked != 0 {
        work.blocked -= 1;
        if work.blocked == 0 && input.follow.main.length() <= LATCH_STICK {
            work.blocked = 1;
        }
        blocked = true;
    }

    let stick_latitude = 1.0 - input.cstick.y.abs();
    let _ = custom_rbr_ratio(radius_ratio(camera, cache.direction.radius()), CURVE_WEIGHT);

    let cushion = if flags & flag::AIM != 0 {
        work.cushion = AIM_CUSHION;
        AIM_CUSHION
    } else if !camera.bump.on_floor {
        CUSHION_JUMP
    } else {
        CUSHION_BASE
    };
    work.cushion += (cushion - work.cushion) * GAIN_CUSHION;
    work.anchor.x = attention.x;
    work.anchor.z = attention.z;

    let height = lockon_center_height(camera, ratio) + if blocked { BLOCK_HEIGHT } else { 0.0 };
    work.anchor.y += work.cushion * ((attention.y + height) - work.anchor.y);

    let inset = f64::from(globe.radius()) * 0.05;
    let stretch = if target.is_some() {
        let here = turn.cos();
        let there = (globe.latitude() * 1.3).cos();
        let pick = if here.abs() < there.abs() { here } else { there };
        (f64::from(globe.radius()) - inset * 2.0) * f64::from((pick * -0.5 + 0.5).abs())
    } else {
        f64::from(globe.radius() * (turn.cos() * -0.5 + 0.5).abs())
    };
    let reach = (stretch + inset) as f32;

    work.latitude_gain += (camera.val(param::UNK3) - work.latitude_gain) * GAIN_CUSHION;
    work.radius_gain += (camera.val(param::UNK4) - work.radius_gain) * GAIN_CUSHION;
    let want_longitude = globe.longitude();
    let longitude = work.eye.longitude()
        + (want_longitude - work.eye.longitude()) * work.latitude_gain;
    let radius = work.eye.radius();
    let latitude = work.eye.latitude();
    let (radius, latitude) = if blocked {
        (
            radius * 0.75,
            latitude + (globe.latitude() - latitude) * BLOCK_GAIN,
        )
    } else {
        (
            radius + work.radius_gain * (work.scale * reach - radius),
            latitude + (globe.latitude() - latitude) * work.latitude_gain,
        )
    };
    work.eye = SGlobe::new(radius, latitude, longitude);
    cache.center = work.anchor + work.eye.to_xyz();

    let held = SGlobe::from_xyz(cache.eye - cache.center);
    let residual = turn - span;
    let mut yaw = cache.direction.longitude();
    let mut latitude = cache.direction.latitude();
    let radius = cache.direction.radius();

    if blocked {
        let nudge = SAngle::from_degrees(if work.swap == 1 {
            BLOCK_NUDGE
        } else {
            -BLOCK_NUDGE
        });
        yaw += (cache.direction.longitude().inv() + nudge - yaw) * 0.05;
    } else {
        let rate = if target.is_none() {
            SETTLE
        } else if flags & flag::ENTERED == 0 {
            if span == SAngle::ZERO {
                ZERO_SPAN_RATE
            } else {
                let ramp = camera.setup_age as f32 / LOCK_FRAMES;
                let edge = camera.val(param::UNK21)
                    * custom_rbr_ratio(-(residual.val() as f32 / span.val() as f32), CURVE_WEIGHT);
                edge + (1.0 - ramp) * (RAMP_RATE - edge)
            }
        } else if turn.val() < span.val() {
            let edge = camera.val(param::UNK21);
            edge * custom_rbr_ratio(-(residual.val() as f32 / span.val() as f32), CURVE_WEIGHT)
        } else {
            let cap = if span > SAngle::from_degrees(SPAN_RUN) {
                SAngle::HALF - span
            } else {
                SAngle::from_degrees(RESIDUAL_CAP)
            };
            let mut rate = camera.val(param::UNK20)
                * cam_follow::rational_bezier_ratio(
                    residual.val() as f32 / cap.val() as f32,
                    CURVE_WEIGHT,
                );
            if range < CLOSE_RANGE {
                rate *= cam_follow::rational_bezier_ratio(range / CLOSE_RANGE, 1.0);
            }
            rate
        };
        let pull = (aim.inv() - cache.direction.longitude()).degree();
        yaw += SAngle::from_degrees(pull * rate);
    }

    let charge_latitude = SAngle::from_degrees(camera.setup.charge_latitude);
    let charge_timer = camera.setup.charge_timer;
    if status.check1(player_status::STTS1_UNK20000) {
        if work.charge_frames <= charge_timer {
            let eased = cam_follow::rational_bezier_ratio(
                work.charge_frames as f32 / charge_timer as f32,
                camera.setup.charge_b_ratio,
            );
            latitude += (charge_latitude - latitude) * eased;
            flags |= flag::CHARGE;
            work.charge_frames += 1;
        } else {
            latitude = charge_latitude;
        }
    } else if blocked {
        latitude -= (globe.longitude() * 0.7 + latitude) * 0.1;
    } else {
        work.charge_frames = 0;
        if flags & flag::AIM != 0 {
            latitude += (lockon_latitude(camera, ratio) - latitude) * AIM_LATITUDE;
        } else if !camera.bump.on_floor && !status.check0(player_status::STTS0_UNK400) {
            latitude += (held.longitude() - latitude)
                * stick_latitude
                * cache.direction.latitude().cos().abs();
        } else {
            latitude += (lockon_latitude(camera, ratio) - latitude) * SETTLE;
        }

        if status.check0(player_status::STTS0_UNK1000000 | player_status::STTS0_SHIP_RIDE) {
            let degrees = latitude.degree();
            if degrees < camera.val(param::UNK16) {
                latitude = SAngle::from_degrees(camera.val(param::UNK16));
            }
            if degrees > camera.val(param::UNK17) {
                latitude = SAngle::from_degrees(camera.val(param::UNK17));
            }
        } else {
            let mut angle = latitude.val();
            if !check_latitude_range(camera, &mut angle) {
                latitude = SAngle::from(angle);
            }
        }
    }

    let radius = if blocked {
        radius + (BLOCK_RADIUS - radius) * 0.05
    } else {
        let mut want = held.radius();
        if default_radius(camera, &mut want) {
            radius
        } else {
            radius + (want - radius) * 0.05
        }
    };

    cache.direction = SGlobe::new(radius, latitude, yaw);
    cache.center += cache.direction.to_xyz();
    cache.fovy += (lockon_fovy(camera, ratio) - cache.fovy) * SETTLE;
    flags |= flag::LOCKED;

    camera.event_flags = flags;
    camera.work.lockon = work;
    camera.view_cache = cache;
    cam_follow::publish(camera);
    true
}

/// `dCamParam_c::LockonLongitude`, `d_cam_param.cpp:274-279`: the style's longitude span, the
/// near slot at the ratio and the far slot at one.
fn lockon_longitude(camera: &Camera, ratio: f32) -> SAngle {
    let near = SAngle::from_degrees(camera.val(param::LOCKON_LONGITUDE_MIN));
    let far = SAngle::from_degrees(camera.val(param::LOCKON_LONGITUDE_MAX));
    near + (far - near) * ratio
}

/// `dCamParam_c::LockonLatitude`, `d_cam_param.cpp:281-286`.
fn lockon_latitude(camera: &Camera, ratio: f32) -> SAngle {
    let near = SAngle::from_degrees(camera.val(param::LOCKON_LATITUDE_MIN));
    let far = SAngle::from_degrees(camera.val(param::LOCKON_LATITUDE_MAX));
    near + (far - near) * ratio
}

/// `dCamParam_c::LockonFovy`, `d_cam_param.cpp:288-293`.
fn lockon_fovy(camera: &Camera, ratio: f32) -> f32 {
    let near = camera.val(param::LOCKON_FOVY_MIN);
    near + ratio * (camera.val(param::LOCKON_FOVY_MAX) - near)
}

/// `dCamParam_c::LockonCenterHeight`, `d_cam_param.cpp:295-300`.
fn lockon_center_height(camera: &Camera, ratio: f32) -> f32 {
    let near = camera.val(param::LOCKON_CENTER_HEIGHT_MIN);
    near + ratio * (camera.val(param::LOCKON_CENTER_HEIGHT_MAX) - near)
}

/// `dCamParam_c::RadiusRatio`, `d_cam_param.cpp:231-262`: how far the placement's radius sits
/// outside the style's own band, as a signed fraction.
fn radius_ratio(camera: &Camera, radius: f32) -> f32 {
    let center = camera.val(param::UNK10);
    if radius < center {
        let distance = center - radius;
        let span = center - camera.val(param::UNK11);
        if distance >= span {
            return -1.0;
        }
        if span > 0.0001 {
            return -distance / span;
        }
    } else if radius > center {
        let distance = radius - center;
        let span = camera.val(param::UNK12) - center;
        if distance >= span {
            return 1.0;
        }
        if span > 0.0001 {
            return distance / span;
        }
    }
    0.0
}

/// `dCamParam_c::DefaultRadius`, `d_cam_param.cpp:200-229`: false, and the radius clamped into
/// the style's band, when it sat outside it.
fn default_radius(camera: &Camera, radius: &mut f32) -> bool {
    let first = camera.val(param::UNK13);
    let second = camera.val(param::UNK14);
    let (low, high) = if first < second {
        (first, second)
    } else {
        (second, first)
    };
    if *radius > high {
        *radius = high;
        return false;
    }
    if *radius < low {
        *radius = low;
        return false;
    }
    true
}

/// `dCamSetup_c::CheckLatitudeRange`, `d_cam_param.cpp:362-378`.
fn check_latitude_range(camera: &Camera, angle: &mut i16) -> bool {
    let upper = SAngle::from_degrees(camera.setup.latitude_upper).val();
    let lower = SAngle::from_degrees(camera.setup.latitude_lower).val();
    if *angle > upper {
        *angle = upper;
        return false;
    }
    if *angle < lower {
        *angle = lower;
        return false;
    }
    true
}

/// `dCamera_c::pointInSight`, `d_camera.cpp:1659-1667`: the point projected through the
/// published view and tested against the window's scissor. The port has the viewport where
/// retail reaches for the window, and the projection the same `mDoLib_project` answers.
fn point_in_sight(camera: &Camera, point: Vec3) -> bool {
    let Some(proj) = camera.project(point) else {
        return false;
    };
    let port = camera.port;
    proj.x > 0.0 && proj.x < port.width && proj.y > 0.0 && proj.y < port.height
}

/// `dCamMath::customRBRatio`, `d_cam_param.cpp:48-59`: the rational ratio folded onto the half
/// turn, so an input past it saturates instead of leaving the curve.
fn custom_rbr_ratio(x: f32, y: f32) -> f32 {
    if x > core::f32::consts::FRAC_1_SQRT_2 {
        if x < 0.0 {
            return -1.0;
        }
        return 1.0;
    }
    cam_follow::rational_bezier_ratio(x * core::f32::consts::SQRT_2, y)
}
