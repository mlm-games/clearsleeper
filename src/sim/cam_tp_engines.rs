use glam::Vec3;

use super::cam_engine::EngineInput;
use super::cam_follow::{self, limitf, rational_bezier_ratio, SAngle, SGlobe};
use super::cam_room::CamRoom;
use super::cam_style::param;
use super::camera::Camera;
use super::player::Player;
use super::player_status;

/// The player's `mHeight`, which `getHeight` answers for a player and which the tower engine
/// divides its measured travel by.
const PLAYER_HEIGHT: f32 = 125.0;

/// `dCamera_c::towerCamera`'s latitude ceiling, `sibling d_camera.cpp:7388`.
const TOWER_LATITUDE_TOP: f32 = 60.0;
/// `sibling d_camera.cpp:7387`: the latitude floor before the player's own state lifts it.
const TOWER_LATITUDE_BOTTOM: f32 = -60.0;
/// `sibling d_camera.cpp:7389`: the divisor of the placement's own measured travel.
const TOWER_FRAMES: f32 = 8.0;
/// `sibling d_camera.cpp:7390`: the value the ground ramp eases toward.
const TOWER_RAMP_TARGET: f32 = 0.9;
/// `sibling d_camera.cpp:7641`: the ease the ground ramp runs on.
const TOWER_RAMP_EASE: f32 = 1.25;
/// `sibling d_camera.cpp:7667`: the degrees the sub-stick may swing the eye's longitude.
const TOWER_YAW_SWING: f32 = 8.0;
/// `sibling d_camera.cpp:7666`: the sub-stick deflection past which the swing counts.
const TOWER_YAW_GATE: f32 = 0.05;
/// `sibling d_camera.cpp:7776`: the ease the ground ramp's own ratio is turned through.
const TOWER_RAMP_LERP: f32 = 0.7;
/// `sibling d_camera.cpp:7638`: the longest the ground ramp counts to.
const TOWER_RAMP_FRAMES: i32 = 80;

/// `dCamera_c::rideCamera`'s latitude limits, `sibling d_camera.cpp:8455-8456`.
const RIDE_LATITUDE_MIN: f32 = -80.0;
/// `sibling d_camera.cpp:8456`.
const RIDE_LATITUDE_MAX: f32 = 80.0;
/// `sibling d_camera.cpp:8487`: the rate the eye's own blend eases toward.
const RIDE_BLEND: f32 = 0.1;
/// `sibling d_camera.cpp:9085`: the degrees the sub-stick may swing the eye's longitude.
const RIDE_YAW_SWING: f32 = 8.0;
/// `sibling d_camera.cpp:9087`: the sub-stick deflection past which the swing counts.
const RIDE_YAW_GATE: f32 = 0.05;
/// `sibling d_camera.cpp:8919`: the ease of the stretch the eye is pulled out by when the measured
/// distance passes the style's own far field-of-view gate.
const RIDE_STRETCH_EASE: f32 = 0.1;

/// `dCamera_c::manualCamera`'s sub-stick gate, `sibling d_camera.cpp:9255-9267`.
const MANUAL_GATE: f32 = 0.75;
/// `sibling d_camera.cpp:9260` and `:9267`: the sub-stick's scale into the gate's own gain.
const MANUAL_STICK_SCALE: f32 = 1.3333334;
/// `sibling d_camera.cpp:9260`: the gain the sub-stick's own scale is run through.
const MANUAL_STICK_GAIN: f32 = 2.0;
/// `sibling d_camera.cpp:9330`: the centre's gain falls away over this distance.
const MANUAL_NEAR: f32 = 100.0;

/// `dCamera_c::hookshotCamera`'s eight probe offsets, `sibling d_camera.cpp:7838-7843`: the chain
/// top is above the player, the first table.
const HOOK_ABOVE: [Vec3; 4] = [
    Vec3::new(180.0, 20.0, -160.0),
    Vec3::new(160.0, 60.0, -200.0),
    Vec3::new(170.0, -80.0, -150.0),
    Vec3::new(100.0, -120.0, -240.0),
];
/// `sibling d_camera.cpp:7844-7849`: the chain top is below the player, the second table.
const HOOK_BELOW: [Vec3; 4] = [
    Vec3::new(40.0, -150.0, 340.0),
    Vec3::new(10.0, -100.0, 280.0),
    Vec3::new(80.0, -50.0, 200.0),
    Vec3::new(40.0, -80.0, -160.0),
];
/// `sibling d_camera.cpp:7851`: the frames that must pass before a found hook point takes the eye.
const HOOK_FRAMES: i32 = 8;

/// `dCamera_c::limitedRangeAddition`, `sibling d_camera.cpp:2307-2326`: the addend folded into the
/// value, the two bounds honoured either way round, and false when the result had to be
/// clamped. `rate` carries the style's own rate and `fallback` the style's slow one, which is
/// what every one of the four call sites in `manualCamera` swaps in when the answer is false.
fn limited_range_addition(
    value: &mut f32,
    addend: f32,
    min: f32,
    max: f32,
    rate: &mut f32,
    fallback: f32,
) -> bool {
    let (addend, min, max) = if min > max {
        (-addend, max, min)
    } else {
        (addend, min, max)
    };
    *value += addend;
    if *value < min {
        *value = min;
        *rate = fallback;
        false
    } else if *value > max {
        *value = max;
        *rate = fallback;
        false
    } else {
        true
    }
}

/// `dCamera_c::mWork.tower`, `d_camera.h:379-414`, narrowed to the members the engine reads
/// back. The tag and the members retail writes and reads inside one frame are left out.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tower {
    /// `field_0x48`: the centre the placement walks onto the target.
    pub walk: Vec3,
    /// `field_0x54`: the arrow's position.
    pub arrow: Vec3,
    /// `field_0x60`: the arrow's own angle.
    pub arrow_angle: SAngle,
    /// `field_0x64`: whether the arrow's angle is a quarter turn or less.
    pub flip: bool,
    /// `field_0x4`: the frames the placement is spread over.
    pub frames: i32,
    /// `field_0x8`: the placement budget, halved into a triangle.
    pub budget: f32,
    /// `field_0xC`: this frame's share of that budget.
    pub share: f32,
    /// `field_0x10`: the frames the ground ramp has counted.
    pub ramp: i32,
    /// `field_0x18`: the ground ramp's own eased value.
    pub ramp_ratio: f32,
    /// `field_0x1C`: the near radius, converging.
    pub radius_near: f32,
    /// `field_0x20`: the far radius, converging.
    pub radius_far: f32,
    /// `field_0x24`: the latitude in degrees.
    pub latitude: f32,
    /// `field_0x28`: the longitude the eye is walked toward.
    pub longitude: SAngle,
    /// `field_0x30`: the eased lateral offset of the eye.
    pub drift: f32,
    /// `field_0x44`: which side of the arrow the eye is pulled to, as a signed one.
    pub side: f32,
    /// `field_0x70`: the radius lerp.
    pub radius_lerp: f32,
    /// `field_0x74`: the latitude lerp.
    pub latitude_lerp: f32,
    /// `field_0x78`: the longitude lerp.
    pub longitude_lerp: f32,
    /// `field_0x7C`: the field-of-view lerp.
    pub fovy_lerp: f32,
    /// `field_0x80`: the centre's lateral lerp.
    pub lerp_lateral: f32,
    /// `field_0x84`: the centre's height lerp.
    pub lerp_height: f32,
    /// `field_0x68`: the eye is blocked, so the offset collapses onto a fixed one.
    pub pushed: bool,
    /// `field_0x69`: the side the player was last on, which resets the longitude lerp.
    pub side_latch: bool,
    /// `field_0x6A`: the sub-stick was swinging the eye's longitude.
    pub aiming: bool,
    /// `field_0x6C`: the main stick's own forty-five degree nudge latch.
    pub nudge: bool,
}

impl Default for Tower {
    fn default() -> Self {
        Self {
            walk: Vec3::ZERO,
            arrow: Vec3::ZERO,
            arrow_angle: SAngle::ZERO,
            flip: false,
            frames: 0,
            budget: 0.0,
            share: 0.0,
            ramp: 0,
            ramp_ratio: 0.0,
            radius_near: 0.0,
            radius_far: 0.0,
            latitude: 0.0,
            longitude: SAngle::ZERO,
            drift: 0.0,
            side: 1.0,
            radius_lerp: 0.75,
            latitude_lerp: 0.01,
            longitude_lerp: 0.01,
            fovy_lerp: 0.01,
            lerp_lateral: 0.0,
            lerp_height: 0.0,
            pushed: false,
            side_latch: true,
            aiming: false,
            nudge: false,
        }
    }
}

/// `daAlink_c::getHsChainTopPos` and `getHsAngle`, `sibling d_camera.cpp:7849-7850`: the chain the
/// hookshot engine's first frame searches around. Nothing else in the port publishes it, so
/// without a chain the engine holds the camera where the collision pass left it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HookshotChain {
    /// `getHsChainTopPos`: the chain's top position.
    pub top: Vec3,
    /// `getHsAngle`: the chain's three angles, latitude, longitude and roll.
    pub angle: [i16; 3],
}

/// `dCamera_c::mWork.hookshot`, `d_camera.h:415-422`, minus the tag and the three spare bytes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hookshot {
    /// `field_0x8`: the centre at the frame the search ran.
    pub start: Vec3,
    /// `field_0x14`: the hook point the search found.
    pub hook: Vec3,
    /// `field_0x20`: the search found an unobstructed hook point.
    pub found: bool,
    /// The chain out while one is, read on the first frame only.
    pub chain: Option<HookshotChain>,
}

impl Default for Hookshot {
    fn default() -> Self {
        Self {
            start: Vec3::ZERO,
            hook: Vec3::ZERO,
            found: false,
            chain: None,
        }
    }
}

/// `dCamera_c::mWork.manual`, `d_camera.h:870-886`, minus the tag and the two members retail
/// writes on the first frame and reads in none.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Manual {
    /// `field_0x4`: the eye offset carried over from a lock-on frame, faded out by `blend`.
    pub carried: Vec3,
    /// `field_0x10`: the point the centre walks toward.
    pub aim: Vec3,
    /// `field_0x1C`: how much of `carried` is still applied.
    pub blend: f32,
    /// `field_0x20`: the centre's height above the player, which the sub-stick walks.
    pub height: f32,
    /// `field_0x30`: the globe the radius, latitude and longitude arms edit in place.
    pub hold: SGlobe,
    /// `field_0x38`: the centre's lateral lerp.
    pub lerp_lateral: f32,
    /// `field_0x3C`: the centre's height lerp.
    pub lerp_height: f32,
    /// `field_0x40`: which of the three pull-in phases the arm is in.
    pub phase: i32,
    /// `field_0x48`: the sub-stick's horizontal position, or the latch the flag forces.
    pub stick: f32,
}

impl Default for Manual {
    fn default() -> Self {
        Self {
            carried: Vec3::ZERO,
            aim: Vec3::ZERO,
            blend: 0.0,
            height: 0.0,
            hold: SGlobe::new(0.0, SAngle::ZERO, SAngle::ZERO),
            lerp_lateral: 0.0,
            lerp_height: 0.0,
            phase: 0,
            stick: 0.0,
        }
    }
}

/// `dCamera_c::mWork.ride`, `d_camera.h:834-868`, narrowed to the members the engine reads
/// back and with the rider, the horse and the latch all left as the player: the port carries
/// no ride actor, and the engine's own first frame puts the player there when there is neither
/// a horse nor a carried box.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ride {
    /// `field_0x3C`: the centre the placement walks toward.
    pub walk: Vec3,
    /// `field_0x24`: the centre the placement's first arm holds.
    pub start: Vec3,
    /// `field_0x48`: the rider's facing, whose inverse is the eye's longitude.
    pub rider: SAngle,
    /// `field_0x4C`: the radius, converging.
    pub radius: f32,
    /// `field_0x50`: the latitude's near term, converging.
    pub latitude_near: f32,
    /// `field_0x54`: the latitude's base, converging.
    pub latitude: f32,
    /// `field_0x58`: the latitude's own term, converging.
    pub latitude_gain: f32,
    /// `field_0x5C`: the blend's own three rates.
    pub blend: Vec3,
    /// `field_0x68`: the eye's own offset from the rider.
    pub offset: Vec3,
    /// `field_0x74`: the centre's height lerp.
    pub lerp_height: f32,
    /// `field_0x78`: the in-flight ramp.
    pub flight: f32,
    /// `field_0x7C`: the eye's radius, latitude and field-of-view lerp.
    pub lerp: f32,
    /// `field_0x80`: the latch the sub-stick clears.
    pub settle: f32,
    /// `field_0x90`: the sub-stick has swung the eye once.
    pub swung: bool,
    /// `field_0x91`: the rider is off the ground.
    pub airborne: bool,
    /// `field_0x10`: the lash-dash countdown.
    pub lash: i32,
    /// `field_0x18`: the frame the lash dash ends on.
    pub lash_end: i32,
    /// `field_0x20`: which of the six placement arms runs.
    pub mode: i32,
}

impl Default for Ride {
    fn default() -> Self {
        Self {
            walk: Vec3::ZERO,
            start: Vec3::ZERO,
            rider: SAngle::ZERO,
            radius: 0.0,
            latitude_near: 0.0,
            latitude: 0.0,
            latitude_gain: 0.0,
            blend: Vec3::new(0.0, 1.0, RIDE_BLEND),
            offset: Vec3::ZERO,
            lerp_height: 0.0,
            flight: 0.0,
            lerp: 0.0,
            settle: 0.0,
            swung: false,
            airborne: false,
            lash: 0,
            lash_end: -1,
            mode: 0,
        }
    }
}

/// The state [`super::cam_engine::Work`] carries for the five engines here.
///
/// These are the five placement engines the sibling reference carries in full readable text and
/// the target game leaves as stubs: `talktoCamera`, `towerCamera`, `rideCamera`, `manualCamera`
/// and `hookshotCamera`. Structure, constants and every decision come from the sibling's text;
/// every number is read out of the target game's own style table through
/// [`super::cam_style::param`]. The two games do not number their style slots the same way, so
/// every read is taken semantically.
///
/// A bare `d_camera.cpp:NNNN` or `d_cam_param.cpp:NNNN` is the target game's own file, which is
/// what the rest of this module cites; `sibling d_camera.cpp:NNNN` is the reference's, and that
/// is where every behaviour and every constant's meaning comes from. A bare `sibling d_camera.cpp`
/// is not a path the reader can open, only a line number in the reference the port was written
/// against.
///
/// # Slot remapping
///
/// The sibling names all twenty-eight of its slots in the enum at `d_cam_param.cpp:18-26`, and
/// the target names fourteen of its thirty in `d_cam_param.cpp`. The fourteen named slots fix
/// fourteen pairs outright, and the two engines whose slot sets are identical across the games
/// fix ten more, which leaves the whole table:
///
/// | sibling | target | meaning | evidence |
/// |---|---|---|---|
/// | 0 `GAPX` | [`param::UNK1`] | lateral gap | the target's `crawlCamera` and `subjectCamera` both read 1 and 0 into the eye offset's lateral and depth, `d_camera.cpp:4191` and `:3833`, and [`super::cam_engine::crawl`] puts them in those same two slots |
/// | 1 `GAPZ` | [`param::UNK0`] | depth gap | the other half of that pair |
/// | 2 `HBAS` | [`param::CENTER_HEIGHT_BASE`] | height base | [`param::CENTER_HEIGHT_BASE`] is the third argument of the target's `CenterHeight`, `d_cam_param.cpp:265`, and the role the sibling's `crawlCamera` gives slot 2 at `sibling d_camera.cpp:4190` |
/// | 3 `HMIN` | [`param::CENTER_HEIGHT_UPPER`] | height min | the first argument of that `CenterHeight` |
/// | 4 `HMAX` | [`param::CENTER_HEIGHT_LOWER`] | height max | the second argument of that `CenterHeight` |
/// | 5 `CDST` | [`param::UNK4`] | distance, the height lerp | the target's `fixedPositionCamera` reads 4 and 3 where the sibling's reads 5 and 6, `d_camera.cpp:4533-4534` against `sibling d_camera.cpp:4228-4229`, and its `lockonCamera` gives 4 the radius gain, `d_camera.cpp:3302` |
/// | 6 `CANG` | [`param::UNK3`] | angle, the lateral lerp | the other half of that pair |
/// | 7 `RBAS` | [`param::UNK10`] | radius base | `RadiusRatio` centres on 10 and spans 10/11/12, `d_cam_param.cpp:227-262`, and the target's `crawlCamera` reads 10 for its radius, `d_camera.cpp:4194` |
/// | 8 `RMIN` | [`param::UNK11`] | radius min | the eleven-thirty second argument of `RadiusRatio` |
/// | 9 `RMAX` | [`param::UNK12`] | radius max | the twelve-th argument of `RadiusRatio` |
/// | 10 `RNAR` | [`param::UNK13`] | radius near | the target's `lockonCamera` eases 13 into `m3DC`, `d_camera.cpp:3226`, and the sibling's `rideCamera` eases 10 into the same role, `sibling d_camera.cpp:8705` |
/// | 11 `RFAR` | [`param::UNK14`] | radius far | the sibling's `rideCamera` eases 11 into the next term, `sibling d_camera.cpp:8706` |
/// | 12 `VBAS` | [`param::UNK15`] | latitude base | the sibling's `towerCamera` reads 12 for its latitude, `sibling d_camera.cpp:7402`, and the target's `followCamera` reads 15 for the same term, `d_camera.cpp:2642` |
/// | 13 `VMIN` | [`param::UNK16`] | latitude min | the target's `lockonCamera` clamps 16 from below, `d_camera.cpp:3608` |
/// | 14 `VMAX` | [`param::UNK17`] | latitude max | the target's `lockonCamera` clamps 17 from above, `d_camera.cpp:3612` |
/// | 15 `VNAR` | [`param::LOCKON_LATITUDE_MIN`] | latitude near | the first argument of the target's `LockonLatitude`, `d_cam_param.cpp:282` |
/// | 16 `VFAR` | [`param::LOCKON_LATITUDE_MAX`] | latitude far | the second argument of that `LockonLatitude`, `d_cam_param.cpp:283` |
/// | 17 `FBAS` | [`param::FOVY_BASE`] | field-of-view base | the third argument of the target's `Fovy`, `d_cam_param.cpp:271` |
/// | 18 `FMIN` | [`param::FOVY_UPPER`] | field-of-view min | the first argument of that `Fovy`, `d_cam_param.cpp:271` |
/// | 19 `FMAX` | [`param::FOVY_LOWER`] | field-of-view max | the second argument of that `Fovy`, `d_cam_param.cpp:271` |
/// | 20 `FNAR` | [`param::LOCKON_FOVY_MIN`] | field-of-view near | the first argument of the target's `LockonFovy`, `d_cam_param.cpp:289` |
/// | 21 `FFAR` | [`param::LOCKON_FOVY_MAX`] | field-of-view far | the second argument of that `LockonFovy`, `d_cam_param.cpp:290` |
/// | 22 `UTRN` | [`param::UNK20`] | turn gain | the target's `lockonCamera` scales its pull gain by 20, `d_camera.cpp:3547` |
/// | 23 `UNAR` | [`param::LOCKON_LONGITUDE_MIN`] | longitude near | the first argument of the target's `LockonLongitude`, `d_cam_param.cpp:275`; this pair did not move |
/// | 24 `UFAR` | [`param::LOCKON_LONGITUDE_MAX`] | longitude far | the second argument of that `LockonLongitude`, `d_cam_param.cpp:276`; this pair did not move |
/// | 25 `HNAR` | [`param::LOCKON_CENTER_HEIGHT_MIN`] | height near | the first argument of the target's `LockonCenterHeight`, `d_cam_param.cpp:296` |
/// | 26 `HFAR` | [`param::LOCKON_CENTER_HEIGHT_MAX`] | height far | the second argument of that `LockonCenterHeight`, `d_cam_param.cpp:297` |
/// | 27 `USLW` | [`param::UNK21`] | slow return rate | the target's `followCamera` eases its eye rate toward 21, `d_camera.cpp:2643`, and its `lockonCamera` uses 21 for the narrow-span rate, `d_camera.cpp:3527` |
///
/// That is twenty-eight sibling slots against twenty-eight target slots, and the two the target
/// added are [`param::UNK2`] and [`param::UNK22`], which no readable engine in either game names.
/// Every read the five engines here make has a name behind it.
///
/// The decisive check is the pair whose slot sets are identical across the games:
/// [`super::cam_engine::fixed_position`] reads the target's 1, 5, 0, 4, 3, 10, 11, 24, 23 and 25
/// at `d_camera.cpp:4530-4539`, and the sibling's `fixedPositionCamera` reads its 0, 2, 1, 5, 6,
/// 7, 8, 24, 23 and 17 at `sibling d_camera.cpp:4529-4538`. Ten slots, ten distinct pairs and no
/// slack, so the mapping is pinned by the pair alone.
///
/// # Engine table
///
/// The two tables agree on the first nine indices and then diverge by insertion. The target
/// slots `hungCamera` in at 9 and `crawlCamera` in at 12, which moves `manualCamera` from 9 to 10,
/// `eventCamera` from 10 to 11 and `hookshotCamera` from 11 to 13. Index 1 is the same engine
/// under its own name, the target's `followCamera` being the sibling's `chaseCamera`. So of the
/// five here, `talktoCamera`, `towerCamera` and `rideCamera` keep their index and `manualCamera`
/// and `hookshotCamera` do not.
///
/// # Left out
///
/// Three things cannot be answered by this port and are left out with what they need named
/// where they stood: the moving-background correction, which needs a moving stage; the monitor
/// distance and the fake-angle system, which the ride and manual engines scale their rates by;
/// and the attention point, eye point, base point, facing and actor name of a second actor, which
/// the talk engine is entirely built on.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct State {
    pub tower: Tower,
    pub hookshot: Hookshot,
    pub manual: Manual,
    pub ride: Ride,
}

impl Default for State {
    fn default() -> Self {
        Self {
            tower: Tower::default(),
            hookshot: Hookshot::default(),
            manual: Manual::default(),
            ride: Ride::default(),
        }
    }
}

/// `dCamera_c::talktoCamera`, `sibling d_camera.cpp:5199-6769`.
///
/// Nothing of the body is ported and the engine settles and declines, which is what its own
/// guard at `sibling d_camera.cpp:5296-5299` does when either of the two actors it frames between is
/// missing. That guard is the whole of what the port can answer, because every line below it
/// reaches one of those two actors:
///
/// - The listener and the speaker themselves. `sibling d_camera.cpp:5276-5286` picks them from the
///   message camera, from the running event camera's staff, or from the player and its lock-on
///   target, and `sibling d_camera.cpp:5316-5320` then reads the speaker's own name to pick between the
///   cut arms. The port has a lock-on position and nothing else: no second actor, no message
///   camera, no event staff and no actor names.
/// - The attention point, position, eye point, base point, facing and attention radius of each.
///   `relationalPos2` at `sibling d_camera.cpp:117-142` takes an explicit camera yaw, `relationalPos` at
///   `:2348-2374` reads the eye's own, and `talkEyePos` and `talkBasePos` behind the cut arms
///   from `sibling d_camera.cpp:5691-6600` read the rest.
/// - The message camera's cut number, which `getMsgCmdCut` answers from the message system and
///   which chooses between those cut arms.
/// - The thirty-six step search at `sibling d_camera.cpp:5572-5625` and the eighteen step searches
///   inside the cut arms, which need an actor collision the port does not query.
pub fn talkto(camera: &mut Camera, _player: &Player, _input: &EngineInput<'_>) -> bool {
    camera.settled = true;
    false
}

/// `dCamera_c::towerCamera`, `sibling d_camera.cpp:7383-7823`.
///
/// The arrow is the whole anchor: the eye's target is the player turned onto the arrow, and
/// then the player's own back by the style's longitude span, and the placement walks the centre
/// out to it over a measured number of frames. Once settled the arrow's direction is where the
/// eye's longitude chases, and the style's own latitude pair is where its latitude converges.
///
/// What is left out, named where it stood:
///
/// - `sibling d_camera.cpp:7580-7583`: the moving-background correction on the placement's own centre,
///   which needs a moving stage.
/// - `sibling d_camera.cpp:7766`: `mForwardTiltOffset`, the ground probe's own latitude, which this
///   engine zeroes for every style without the `0x4000` flag and which the collision pass
///   keeps to itself.
/// - `sibling d_camera.cpp:7635`: the arm that zeroes the ground ramp. Its test is an unnamed
///   integer on the background's own sub-record, `mBG.field_0xc0.field_0x44`, which the readable
///   text never names. [`Bump::on_floor`](super::cam_bump::Bump::on_floor) is this port's reading of it: the one
///   the ramp's shape supports, and one nothing in the text proves.
/// - `sibling d_camera.cpp:7542`: the immediate settle, which reads the event flag word or the camera
///   mode. The mode half is ported; the flag half is not, because the port keeps no event flag
///   word.
/// - `sibling d_camera.cpp:7688` and `:7707`: the monitor's measured distance, which divides the
///   player's maximum speed to scale the side drift's rate. With no monitor the rate is zero
///   and the drift holds its sign.
/// - `sibling d_camera.cpp:7501`, `:7513`, `:7530` and `:7548`: the three stage-keyed arms.
/// - `sibling d_camera.cpp:7418-7423`: the `mGear` arm, which folds the radius, height, latitude and
///   field-of-view triples together for one camera mode. `CamSetup` carries no gear, so the
///   triples stand as the style sets them.
/// - `sibling d_camera.cpp:7531` and the override at `:7604`: the `bVar5` arm, which puts the
///   placement's latitude on the style's base for one stage and one pad action. Neither the
///   stage number nor the action is published to the port, so the placement walks the camera's
///   own latitude, which is what `sibling d_camera.cpp:7603` gives when the override is off.
/// - `sibling d_camera.cpp:7815-7818`: the ceiling clamp on the eye, which needs the background's own
///   upper surface.
/// - `sibling d_camera.cpp:7806`: the arm that falls back to the chase engine when the room has no
///   arrow, which the port's dispatch cannot redirect, so the engine declines instead.
pub fn tower(
    camera: &mut Camera,
    player: &Player,
    room: Option<&CamRoom>,
    input: &EngineInput<'_>,
) -> bool {
    let status = input.follow.status;
    let mut w = camera.work.tp.tower;
    let mut cache = camera.view_cache;
    let facing = SAngle::from_radians(player.face);

    let gap_x = camera.val(param::UNK1);
    let gap_z = camera.val(param::UNK0);
    let height = camera.val(param::CENTER_HEIGHT_BASE);
    let mut height_low = camera.val(param::CENTER_HEIGHT_UPPER);
    let lerp_lateral = camera.val(param::UNK3);
    let lerp_height = camera.val(param::UNK4);
    let radius_far = camera.val(param::UNK10);
    let radius_near = camera.val(param::UNK11);
    let radius_lerp = camera.val(param::UNK13);
    let latitude_gain = camera.val(param::UNK14);
    let latitude_pull = camera.val(param::UNK16);
    let latitude_base = camera.val(param::UNK15);
    let latitude_top = camera.val(param::LOCKON_LATITUDE_MAX);
    let fovy_base = camera.val(param::FOVY_BASE);
    let fovy_low = camera.val(param::FOVY_UPPER);
    let yaw_curve = camera.val(param::LOCKON_FOVY_MIN);
    let fovy_lerp = camera.val(param::LOCKON_FOVY_MAX);
    let yaw_near = SAngle::from_degrees(camera.val(param::LOCKON_LONGITUDE_MIN));
    let yaw_far = SAngle::from_degrees(camera.val(param::LOCKON_LONGITUDE_MAX));
    let lockon_blend = camera.val(param::LOCKON_CENTER_HEIGHT_MAX);
    let swing_base = camera.val(param::UNK21);
    let swing_weight = camera.val(param::UNK20);
    let monitor_rate = 0.0f32;

    if camera.setup_age == 0 {
        if let Some(arrow) = room.and_then(CamRoom::arrow_entry) {
            w.arrow = arrow.position;
            w.arrow_angle = SAngle::from(arrow.angle[1]);
            w.flip = yaw_near <= SAngle::QUARTER;
        }
    }

    if room.and_then(CamRoom::arrow_entry).is_none() {
        return false;
    }

    let mut floor = TOWER_LATITUDE_BOTTOM;
    if status.check0(0x0810_0000) {
        if floor < 4.0 {
            floor = 4.0;
        }
        if height_low < -10.0 {
            height_low = -10.0;
        }
    }

    let to_arrow = SGlobe::from_xyz(player.pos - w.arrow);
    let player_side = to_arrow.longitude() - facing > SAngle::ZERO;
    let eye_side = to_arrow.longitude() - cache.direction.longitude() > SAngle::ZERO;

    if camera.setup_age == 0 {
        cache.direction = SGlobe::from_xyz(camera.bump.eye - camera.bump.center);
        w.ramp_ratio = 0.0;
        w.radius_near = camera.bump.direction.radius();
        w.radius_far = w.radius_near;
        w.ramp = 0;
        w.latitude = camera.bump.direction.latitude().degree();
        w.walk = cache.center;
        w.latitude_lerp = 0.01;
        w.longitude_lerp = 0.01;
        w.fovy_lerp = 0.01;
        w.radius_lerp = 0.75;
        w.lerp_lateral = lerp_lateral;
        w.lerp_height = lerp_height;
        let signed = if camera.has_flag(0x100) {
            to_arrow.longitude() - cache.direction.longitude().inv()
        } else {
            to_arrow.longitude() - facing
        };
        w.side = if signed > SAngle::ZERO { 1.0 } else { -1.0 };
        w.pushed = false;
        w.longitude = cache.direction.longitude();
        w.drift = 0.0;
        w.aiming = false;
        w.nudge = false;
    }

    let spread = limitf(
        (cache.direction.radius() - radius_near) / (radius_far - radius_near),
        0.0,
        1.0,
    );

    if camera.has_flag(0x400) {
        w.drift = gap_x * (w.arrow_angle - to_arrow.longitude()).sin();
    } else if camera.event_flags & 0x10_0000 != 0 && status.check0(0x000A_50C0) {
        if input.follow.main.x < -0.2 {
            w.nudge = true;
        }
        if input.follow.main.x > 0.2 {
            w.nudge = false;
        }
        w.drift += ((if w.nudge { -45.0 } else { 45.0 }) - w.drift) * 0.04;
    } else {
        w.drift += (gap_x - w.drift) * 0.06;
    }

    let offset = if w.pushed {
        Vec3::new(w.drift, 20.0, 1.0)
    } else {
        Vec3::new(
            w.drift,
            height_low + (height - height_low) * spread,
            gap_z * spread,
        )
    };

    if camera.setup_age == 0 {
        let anchor = cam_follow::relational_pos(player, facing, offset);
        let eye_gap = (camera.bump.eye - anchor).length() - radius_far;
        let centre_gap = (camera.bump.center - anchor).length() - radius_far;
        let travel = eye_gap.max(centre_gap).abs() / PLAYER_HEIGHT.max(10.0);
        w.frames = (travel.sqrt() * TOWER_FRAMES) as i32 + 1;
        if camera.mode == 1 {
            camera.settled = true;
            w.frames = 1;
        }
        w.budget = ((w.frames * (w.frames + 1)) >> 1) as f32;
        w.share = 0.0;
    }

    let mut placed = SGlobe::from_xyz(offset);
    placed.set_longitude(
        placed.longitude()
            + if camera.has_flag(0x1000) {
                facing
            } else {
                to_arrow.longitude()
            },
    );
    let target = match (camera.has_flag(0x800), input.lockon.target) {
        (true, Some(lockon)) => {
            camera.event_flags |= 0x2000;
            player.pos + placed.to_xyz() + (lockon - player.pos) * lockon_blend
        }
        _ => player.pos + placed.to_xyz(),
    };

    if !camera.settled {
        w.share = w.frames as f32 - camera.setup_age as f32;
        let ratio = w.share / w.budget;
        w.walk += (target - w.walk) * ratio;
        cache.center += (w.walk - cache.center) * lerp_lateral;

        let mut probe = player.pos;
        probe.y -= 15.0;
        if let Some(hit) = input
            .collision
            .and_then(|solid| solid.line_cross(probe, cache.center))
        {
            cache.center = hit.point + hit.normal;
        }

        let radius = limitf(cache.direction.radius(), radius_near, radius_far);
        let latitude = limitf(
            cache.direction.latitude().degree(),
            floor,
            TOWER_LATITUDE_TOP,
        );
        let wanted = SGlobe::new(radius, SAngle::from_degrees(latitude), camera.angle_y.inv());
        let current = cache.direction;
        cache.direction = SGlobe::new(
            current.radius() + (wanted.radius() - current.radius()) * ratio,
            current.latitude() + (wanted.latitude() - current.latitude()) * ratio,
            current.longitude() + (wanted.longitude() - current.longitude()) * ratio,
        );
        cache.eye = cache.center + cache.direction.to_xyz();

        if camera.setup_age as i32 >= w.frames - 1 {
            camera.settled = true;
        }
        w.latitude = cache.direction.latitude().degree();
        w.radius_near = cache.direction.radius();
        w.radius_far = w.radius_near;
        cache.fovy += (fovy_base - cache.fovy) * ratio;
        w.budget -= w.share;

        camera.work.tp.tower = w;
        camera.view_cache = cache;
        cam_follow::publish(camera);
        return true;
    }

    if camera.bump.on_floor {
        w.ramp_ratio = 0.0;
        w.ramp = 0;
    } else if w.ramp < TOWER_RAMP_FRAMES {
        w.ramp += 1;
        w.ramp_ratio += (TOWER_RAMP_TARGET - w.ramp_ratio)
            * rational_bezier_ratio(w.ramp as f32 / TOWER_RAMP_FRAMES as f32, TOWER_RAMP_EASE);
    }

    cache.center += (target - cache.center) * Vec3::new(w.lerp_lateral, w.lerp_height, w.lerp_lateral);
    let held = SGlobe::from_xyz(camera.bump.eye - camera.bump.center);

    let mut reach = 1.0f32;
    if radius_far > radius_near {
        reach = limitf(
            (cache.direction.radius() - radius_near) / (radius_far - radius_near),
            0.0,
            1.0,
        );
    }

    let stick = input.cstick.x;
    let press = input.follow.main.length();
    let span = yaw_near + (yaw_near - yaw_far) * press;

    if !camera.has_flag(0x040) && stick.abs() > TOWER_YAW_GATE {
        let swing = cache.direction.longitude()
            + SAngle::from_degrees(rational_bezier_ratio(stick, 0.5) * TOWER_YAW_SWING);
        let share = stick.abs() - TOWER_YAW_GATE;
        w.longitude = cache.direction.longitude() + (swing - cache.direction.longitude()) * share;
        w.aiming = true;
        w.longitude_lerp += (0.8 - w.longitude_lerp) * 0.05;
    } else {
        if w.aiming {
            w.longitude_lerp = 0.0;
            w.aiming = false;
        }
        if camera.has_flag(0x800) {
            let side = to_arrow.longitude() - facing;
            if side > SAngle::ZERO {
                if w.side < 1.0 {
                    w.side += monitor_rate * 0.025;
                }
            } else if w.side > -1.0 {
                w.side -= monitor_rate * 0.025;
            }
            w.longitude_lerp += (0.5 - w.longitude_lerp) * 0.05;
            w.longitude = if !w.flip {
                to_arrow.longitude().inv() - (SAngle::HALF - span) * w.side
            } else {
                to_arrow.longitude() - span * w.side
            };
        } else if camera.has_flag(0x100) {
            let turn = if eye_side {
                to_arrow.longitude() - held.longitude()
            } else {
                held.longitude() - to_arrow.longitude()
            };
            let mut share = rational_bezier_ratio(reach, yaw_curve);
            if turn > span {
                share = 1.0;
            }
            let target_longitude = if eye_side {
                to_arrow.longitude() - span
            } else {
                to_arrow.longitude() + span
            };
            w.longitude +=
                (held.longitude() + (target_longitude - held.longitude()) * share - w.longitude)
                    * 0.33;
            let held_press = if status.check0(0x0200_0108) { 0.0 } else { press };
            w.longitude_lerp = swing_base + swing_weight * held_press;
        } else {
            let target_longitude = if player_side {
                to_arrow.longitude() + span
            } else {
                to_arrow.longitude() - span
            };
            w.longitude +=
                (held.longitude() + (target_longitude - held.longitude()) - w.longitude) * 0.33;
            if status.check0(0x0200_0108) || w.side_latch != player_side {
                w.longitude_lerp = 0.0;
            } else {
                w.longitude_lerp += (swing_base + swing_weight * press - w.longitude_lerp) * 0.05;
            }
        }
    }

    w.side_latch = player_side;
    cache.direction.set_longitude(
        cache.direction.longitude()
            + (w.longitude - cache.direction.longitude()) * w.longitude_lerp,
    );

    let want_latitude = latitude_pull + (latitude_base - latitude_pull) * reach;
    w.latitude += (want_latitude - w.latitude) * latitude_gain;

    let settled_latitude = if w.ramp == 0 {
        w.latitude_lerp += (latitude_top - w.latitude_lerp) * 0.01;
        SAngle::from_degrees(w.latitude)
    } else {
        w.latitude_lerp = rational_bezier_ratio(w.ramp_ratio, TOWER_RAMP_LERP);
        w.latitude = held.latitude().degree();
        held.latitude()
    };
    let settled_latitude = if settled_latitude < SAngle::from_degrees(floor) {
        SAngle::from_degrees(floor)
    } else if settled_latitude > SAngle::from_degrees(TOWER_LATITUDE_TOP) {
        SAngle::from_degrees(TOWER_LATITUDE_TOP)
    } else {
        settled_latitude
    };
    cache.direction.set_latitude(
        cache.direction.latitude()
            + (settled_latitude - cache.direction.latitude()) * w.latitude_lerp,
    );

    let mut radius = held.radius();
    w.radius_near += (radius_near - w.radius_near) * latitude_gain;
    w.radius_far += (radius_far - w.radius_far) * latitude_gain;
    if radius < w.radius_near {
        w.radius_lerp += (radius_lerp - w.radius_lerp) * 0.01;
        radius = w.radius_near;
    } else if radius > w.radius_far {
        w.radius_lerp += (radius_lerp - w.radius_lerp) * 0.01;
        radius = w.radius_far;
    } else {
        w.radius_lerp = 1.0;
    }

    if camera.has_flag(0x400) {
        let probe = SGlobe::new(radius, cache.direction.latitude(), cache.direction.longitude());
        w.pushed = input
            .collision
            .is_some_and(|solid| solid.line_cross(cache.center + probe.to_xyz(), cache.center).is_some());
    }

    cache
        .direction
        .set_radius(cache.direction.radius() + (radius - cache.direction.radius()) * w.radius_lerp);
    cache.eye = cache.center + cache.direction.to_xyz();

    w.fovy_lerp += (fovy_lerp - w.fovy_lerp) * 0.01;
    cache.fovy += (fovy_low + (fovy_base - fovy_low) * reach - cache.fovy) * w.fovy_lerp;

    camera.work.tp.tower = w;
    camera.view_cache = cache;
    cam_follow::publish(camera);
    true
}

/// `dCamera_c::hookshotCamera`, `sibling d_camera.cpp:7826-7940`.
///
/// The first frame searches eight offsets around the chain's top for a hook point that neither
/// the player nor the camera can see, and settles. Every frame after that the eye is walked
/// onto that point, and left where it is when the search found nothing.
///
/// The chain itself is not published by the port, so with no chain the search never runs and
/// the engine holds the camera where the collision pass left it. The search's mirror seed,
/// which the reference takes from a global frame counter, starts unmirrored.
pub fn hookshot(camera: &mut Camera, player: &Player, input: &EngineInput<'_>) -> bool {
    let mut w = camera.work.tp.hookshot;
    let gap_x = camera.val(param::UNK1);
    let gap_z = camera.val(param::UNK0);
    let height = camera.val(param::CENTER_HEIGHT_BASE);
    let lerp_height = camera.val(param::UNK4);
    let lerp_lateral = camera.val(param::UNK3);
    let radius_far = camera.val(param::UNK10);
    let radius_near = camera.val(param::UNK11);
    let reach = camera.val(param::UNK14);
    let fovy = camera.val(param::FOVY_BASE);
    let gain = camera.val(param::LOCKON_LONGITUDE_MIN);

    if camera.setup_age == 0 {
        w.start = camera.bump.center;
        if let Some(chain) = w.chain {
            let low = SAngle::from(chain.angle[0]).val() < -0x3333;
            let table = if low { &HOOK_BELOW } else { &HOOK_ABOVE };
            let mut latitude = SAngle::from(chain.angle[0]);
            let mut longitude = SAngle::from(chain.angle[1]);
            if low {
                let globe = SGlobe::from_xyz(chain.top - player.pos);
                latitude = globe.latitude();
                longitude = globe.longitude();
            }
            if (chain.top - player.pos).length() > reach && camera.has_flag(0x800) {
                let mut mirror = false;
                for probe in 0..8usize {
                    let mut offset = table[probe >> 1];
                    if mirror {
                        offset.x = -offset.x;
                    }
                    let mut globe = SGlobe::from_xyz(offset);
                    globe.set_longitude(globe.longitude() + longitude);
                    if !low {
                        globe.set_latitude(globe.latitude() + latitude);
                    }
                    w.hook = chain.top + globe.to_xyz();
                    if input
                        .collision
                        .is_none_or(|solid| solid.line_cross(player.pos, w.hook).is_none())
                    {
                        w.found = true;
                        break;
                    }
                    mirror = !mirror;
                }
            }
        }
        camera.settled = true;
        camera.work.tp.hookshot = w;
        return true;
    }

    let mut want = camera.bump.eye;
    let mut rate = gain;
    if w.found && camera.setup_age as i32 > HOOK_FRAMES && camera.has_flag(0x800) {
        want = w.hook;
        rate = 1.0;
    }

    let facing = SAngle::from_radians(player.face);
    let mut cache = camera.view_cache;
    let anchor = cam_follow::relational_pos(player, facing, Vec3::new(gap_x, height, gap_z));
    let lerp = Vec3::new(lerp_lateral, lerp_height, lerp_lateral);
    cache.center += (anchor - cache.center) * lerp;

    let mut wanted = SGlobe::from_xyz(want - cache.center);
    if wanted.radius() < radius_near {
        wanted.set_radius(radius_near);
    }
    if wanted.radius() > radius_far {
        wanted.set_radius(radius_far);
    }
    let current = cache.direction;
    cache.direction = SGlobe::new(
        current.radius() + (wanted.radius() - current.radius()) * rate,
        current.latitude() + (wanted.latitude() - current.latitude()) * rate,
        current.longitude() + (wanted.longitude() - current.longitude()) * rate,
    );
    cache.eye = cache.center + cache.direction.to_xyz();
    cache.fovy += (fovy - cache.fovy) * rate;

    camera.work.tp.hookshot = w;
    camera.view_cache = cache;
    cam_follow::publish(camera);
    true
}

/// `dCamera_c::manualCamera`, `sibling d_camera.cpp:9177-9432`.
///
/// The engine holds the camera in one place: the sub-stick walks the centre's height, the
/// radius, the latitude and the field of view inside the style's own bounds, and the centre
/// itself is walked onto a point held just off the player. With a lock-on target that point is
/// held between the two actors instead, and the eye is pulled off the straight line to it for
/// as long as the view along that line stays clear.
///
/// The reference's body sits inside a build-time switch, `sibling d_camera.cpp:9178-9430`, so this is
/// the only readable shape of the engine there is.
///
/// What is left out, named where it stood:
///
/// - `sibling d_camera.cpp:9272-9274`: `setUSOAngle`, which the fake-angle system owns and which needs
///   that system's own latch word.
/// - `sibling d_camera.cpp:9404`, `:9415`, `:9416` and `:9418`: the four `setComStat` calls, which are
///   communication state and not camera state.
/// - `sibling d_camera.cpp:9367`: `sp34`, which the reference leaves unassigned under the `0x100` flag,
///   so only the flag-gated pair at `:9368-9370` runs. This port holds the globe's own
///   longitude across the frame instead, the one reading of that slot the lines around it
///   support.
pub fn manual(camera: &mut Camera, player: &Player, input: &EngineInput<'_>) -> bool {
    let status = input.follow.status;
    let mut w = camera.work.tp.manual;
    let mut cache = camera.view_cache;
    let facing = SAngle::from_radians(player.face);

    let gap_x = camera.val(param::UNK1);
    let mut gap_z = camera.val(param::UNK0);
    let mut lerp_lateral = camera.val(param::UNK3);
    let lerp_height = camera.val(param::UNK4);
    let gain = camera.val(param::UNK21);
    let slow = camera.val(param::UNK20);
    let stick_height = camera.val(param::LOCKON_CENTER_HEIGHT_MAX);
    let mut height_low = camera.val(param::CENTER_HEIGHT_UPPER);
    let height_high = camera.val(param::CENTER_HEIGHT_LOWER);
    let radius_near = camera.val(param::UNK11);
    let mut radius_far = camera.val(param::UNK12);
    let reach = camera.val(param::UNK14);
    let mut latitude_low = camera.val(param::UNK16);
    let mut latitude_high = camera.val(param::UNK17);
    let mut stick_latitude = camera.val(param::LOCKON_LATITUDE_MAX);
    let fovy_low = camera.val(param::FOVY_UPPER);
    let fovy_high = camera.val(param::FOVY_LOWER);
    let stick_fovy = camera.val(param::LOCKON_FOVY_MAX);
    let swing = camera.val(param::LOCKON_LONGITUDE_MAX);

    if camera.setup_age == 0 {
        camera.settled = true;
        w.blend = f32::from(input.lockon.truth && input.lockon.target.is_some());
        w.height = cache.center.y - player.pos.y;
        cache.direction = SGlobe::from_xyz(cache.eye - cache.center);
        w.hold = cache.direction;
        w.carried = Vec3::ZERO;
        w.phase = 0;
        w.stick = input.cstick.x;
        if camera.has_flag(0x100) {
            w.stick = if w.stick > 0.0 { -0.8 } else { 0.8 };
        }
    }

    if status.check0(0x0800_0000) {
        if latitude_low < 4.0 {
            latitude_low = 4.0;
        }
        if height_low < -10.0 {
            height_low = -10.0;
        }
    } else if status.check0(0x2800_0108) {
        latitude_low = if status.check0(0x0200_0000) { -58.0 } else { -70.0 };
        latitude_high = 70.0;
        stick_latitude = 4.0;
        lerp_lateral = 0.45;
        if radius_far > 500.0 {
            radius_far = 500.0;
        }
    }

    let stick_x = if input.cstick.x >= MANUAL_GATE {
        1.0
    } else if input.cstick.x <= -MANUAL_GATE {
        -1.0
    } else {
        rational_bezier_ratio(input.cstick.x * MANUAL_STICK_SCALE, MANUAL_STICK_GAIN)
    };
    let stick_y = if input.cstick.y >= MANUAL_GATE {
        1.0
    } else if input.cstick.y <= -MANUAL_GATE {
        -1.0
    } else {
        rational_bezier_ratio(input.cstick.y * MANUAL_STICK_SCALE, MANUAL_STICK_GAIN)
    };

    let mut rate_radius = gain;
    let mut rate_latitude = gain;
    let rate_longitude = gain;
    let mut rate_fovy = gain;

    let mut height = w.height;
    limited_range_addition(
        &mut height,
        -stick_y * stick_height,
        height_low,
        height_high,
        &mut rate_fovy,
        slow,
    );
    w.height += rate_fovy * (height - w.height);

    if status.check0(player_status::STTS0_HANG) {
        if gap_z > -10.0 {
            gap_z = -10.0;
        }
        if w.height < 30.0 {
            w.height = 30.0;
        }
    }

    let offset = Vec3::new(gap_x, w.height, gap_z);
    let locked = input.lockon.truth && input.lockon.target.is_some();
    match (locked, input.lockon.target) {
        (true, Some(target)) => {
            w.aim = relational_pair(player.pos, target, offset, 0.5, cache.direction.longitude());
            let plain = cam_follow::relational_pos(player, facing, offset);
            let blocked = input.collision.is_some_and(|solid| {
                solid.line_cross(plain, w.aim).is_some()
                    && solid.line_cross(cache.eye, plain).is_some()
            });
            if !blocked {
                w.carried = w.aim - plain;
                if w.blend < 1.0 {
                    w.blend += 0.05;
                    w.aim = plain + w.carried * w.blend;
                } else if w.blend > 1.0 {
                    w.blend = 1.0;
                }
            }
        }
        _ => {
            w.aim = cam_follow::relational_pos(player, facing, offset);
            if w.blend > 0.0 {
                w.blend -= 0.05;
                w.aim += w.carried * w.blend;
            } else if w.blend < 0.0 {
                w.blend = 0.0;
            }
        }
    }

    if camera.setup_age == 0 {
        let away = (w.aim - cache.center).length();
        let share = if away > MANUAL_NEAR {
            0.0
        } else {
            1.0 - away / MANUAL_NEAR
        };
        w.lerp_lateral = lerp_lateral * share;
        w.lerp_height = lerp_height * share;
    } else {
        w.lerp_lateral += (lerp_lateral - w.lerp_lateral) * 0.05;
        w.lerp_height += (lerp_height - w.lerp_height) * 0.05;
    }
    cache.center += (w.aim - cache.center) * Vec3::new(w.lerp_lateral, w.lerp_height, w.lerp_lateral);

    if input.lockon.target.is_none() {
        let mut probe = player.pos;
        if status.check0(0x0810_0100) {
            probe.y = player.pos.y + PLAYER_HEIGHT + 30.0;
        } else {
            probe.y -= 15.0;
        }
        if let Some(hit) = input
            .collision
            .and_then(|solid| solid.line_cross(probe, cache.center))
        {
            cache.center = hit.point + hit.normal;
        }
    }

    let mut radius = w.hold.radius();
    limited_range_addition(
        &mut radius,
        -stick_y * reach,
        radius_near,
        radius_far,
        &mut rate_radius,
        slow,
    );

    let mut latitude = w.hold.latitude().degree();
    limited_range_addition(
        &mut latitude,
        -stick_y * stick_latitude,
        latitude_low,
        latitude_high,
        &mut rate_latitude,
        slow,
    );

    let mut longitude = w.hold.longitude().degree();
    if !camera.has_flag(0x100) {
        longitude = w.hold.longitude().degree() + stick_x * swing;
    }
    w.hold = SGlobe::new(
        radius,
        SAngle::from_degrees(latitude),
        SAngle::from_degrees(longitude),
    );

    if locked {
        camera.event_flags |= 0x2000;
    }

    let current = cache.direction;
    cache.direction = SGlobe::new(
        current.radius() + (w.hold.radius() - current.radius()) * rate_radius,
        current.latitude() + (w.hold.latitude() - current.latitude()) * rate_latitude,
        current.longitude() + (w.hold.longitude() - current.longitude()) * rate_longitude,
    );
    cache.eye = cache.center + cache.direction.to_xyz();

    let mut fovy = cache.fovy;
    limited_range_addition(
        &mut fovy,
        -stick_y * stick_fovy,
        fovy_low,
        fovy_high,
        &mut rate_fovy,
        slow,
    );
    cache.fovy += rate_fovy * (fovy - cache.fovy);

    w.phase = if input.follow.main.length() > 0.01
        || input.lockon.truth
        || status.check0(0x0100_0000)
    {
        0
    } else {
        match w.phase {
            1 => {
                if stick_y < -0.01 {
                    0
                } else if stick_y > 0.9 {
                    2
                } else {
                    1
                }
            }
            2 => {
                if stick_y < 0.9 {
                    0
                } else {
                    2
                }
            }
            _ => {
                if radius < radius_near * 1.0001 && stick_y < 0.3 {
                    1
                } else {
                    0
                }
            }
        }
    };

    camera.work.tp.manual = w;
    camera.view_cache = cache;
    cam_follow::publish(camera);
    true
}

/// `dCamera_c::relationalPos` between two actors, `sibling d_camera.cpp:2348-2374`: the offset turned
/// into the pair's own frame and added to their midpoint, the pair's separation halved by the
/// cosine of how far the camera already is from that frame and by the caller's weight. Both
/// attention radii read as zero, because the port carries a target's position and nothing
/// else about it.
fn relational_pair(
    listener: Vec3,
    speaker: Vec3,
    offset: Vec3,
    weight: f32,
    camera_yaw: SAngle,
) -> Vec3 {
    let delta = speaker - listener;
    let mut span = SGlobe::from_xyz(delta);
    let mid = listener + delta * 0.5;
    let angle = camera_yaw - span.longitude();
    let mut turned = offset;
    if angle < SAngle::ZERO {
        turned.x = -turned.x;
    }
    let mut lifted = SGlobe::from_xyz(turned);
    lifted.set_longitude(span.longitude() + lifted.longitude());
    span.set_radius(0.5 * span.radius() * angle.cos() * weight);
    mid + span.to_xyz() + lifted.to_xyz()
}

/// `dCamera_c::rideCamera`, `sibling d_camera.cpp:8454-9174`.
///
/// Without a ride actor the engine's own first frame puts the player in its place, so that is
/// what runs here: the eye rides a globe behind the player, the centre is walked onto a point
/// held off the player's own offset, and the whole placement is spread over a measured number
/// of frames before the eye chases the globe.
///
/// What is left out, named where it stood:
///
/// - `sibling d_camera.cpp:8519-8536`: the horse and the carried box, which are the ride actor and the
///   latch that whole branch keys on, and `sibling d_camera.cpp:8596-8600` with it: the settle the
///   carried box causes and the canoe test that suppresses it.
/// - `sibling d_camera.cpp:8698-8760`: the lock-on arms, which need four actors' names.
/// - `sibling d_camera.cpp:8765-8794` and `:8842-8852`: the tandem, board and flight states, which the
///   player status does not publish.
/// - `sibling d_camera.cpp:8616`: the monitor's measured distance, which every rate in the function is
///   scaled by and which the bank reads at `:9167`. With no monitor it reads zero, so those
///   rates stand still and the bank holds.
/// - `sibling d_camera.cpp:8656` and its nine writers, of which `:8910-8915` is the lash-dash hookshot
///   arm: the eye's lateral slant. It needs the horse at `:8676`, `:8725`, `:8741` and `:8872`,
///   and the hookshot the player does not report at `:8911`. With every writer left out the
///   slant is zero and its own ease at `:8931` still runs.
/// - `sibling d_camera.cpp:8918`, `:8935` and `:9005`: the rider's own pitch, which the player does not
///   publish, so the style's own latitude stands in for it.
/// - `sibling d_camera.cpp:8602`: the `specialType[9]` test on the third placement arm. The port keeps
///   no special-type table, so only the flag half of that pair runs.
/// - `sibling d_camera.cpp:8624-8630` and `:9056-9061`, `:9157-9163`: the lash-dash countdown, which
///   needs the horse.
/// - `sibling d_camera.cpp:9049`: the canoe test that stops the eye's yaw, which needs the canoe.
/// - `sibling d_camera.cpp:8959`: `jutOutCheck`, which keeps the walked centre off the ride actor by a
///   fixed margin; the port's line-check already answers that.
pub fn ride(camera: &mut Camera, player: &Player, input: &EngineInput<'_>) -> bool {
    let mut w = camera.work.tp.ride;
    let mut cache = camera.view_cache;
    let facing = SAngle::from_radians(player.face);

    let gap_z = camera.val(param::UNK0);
    let height = camera.val(param::CENTER_HEIGHT_BASE);
    let lerp_height = camera.val(param::UNK4);
    let lerp_lateral = camera.val(param::UNK3);
    let radius_far = camera.val(param::UNK10);
    let monitor_span = camera.val(param::UNK14);
    let latitude_near = camera.val(param::UNK13);
    let latitude = camera.val(param::UNK15);
    let latitude_gain = camera.val(param::LOCKON_LATITUDE_MIN);
    let fovy_base = camera.val(param::FOVY_BASE);
    let fovy_far = camera.val(param::LOCKON_FOVY_MIN);
    let fovy_gate = camera.val(param::LOCKON_FOVY_MAX);
    let chase = camera.val(param::UNK20);
    let bank = camera.val(param::LOCKON_LONGITUDE_MIN);
    let height_extra = camera.val(param::LOCKON_CENTER_HEIGHT_MIN);
    let mut blend = camera.val(param::LOCKON_CENTER_HEIGHT_MAX);
    let lerp = camera.val(param::UNK21);

    if !(0.0..=1.0).contains(&blend) {
        blend = 0.5;
    }
    let monitor = 0.0f32;
    let span = limitf(monitor / monitor_span.max(1.0), -1.0, 1.0);

    if camera.setup_age == 0 {
        cache.center = camera.bump.center;
        cache.eye = camera.bump.eye;
        cache.direction = camera.bump.direction;
        w.rider = facing;
        w.walk = cache.center;
        w.start = cache.center;
        cache.direction = SGlobe::from_xyz(cache.eye - cache.center);
        w.radius = cache.direction.radius();
        w.latitude = cache.direction.latitude().degree();
        w.latitude_near = 0.0;
        w.latitude_gain = 0.0;
        w.blend = Vec3::new(0.0, 1.0, RIDE_BLEND);
        w.lerp_height = lerp_height;
        w.lash = 0;
        w.lash_end = -1;
        w.flight = 0.0;
        w.settle = 1.0;
        w.swung = false;
        w.airborne = false;
        w.mode = if camera.event_flags & 0x8000 != 0 {
            2
        } else if camera.event_flags & 0x20 != 0 {
            3
        } else if camera.event_flags & 0x1_0000 != 0 {
            4
        } else if camera.has_flag(0x400) {
            5
        } else {
            0
        };
        if w.mode == 2 {
            camera.settled = true;
        }
        w.lerp = lerp;
    }

    w.lash = 0;
    w.lash_end = -1;
    w.flight = 0.0;
    w.lerp += (lerp - w.lerp) * 0.1;

    let stretch = if span > fovy_gate {
        rational_bezier_ratio(
            (span - fovy_gate) / (1.0 - fovy_gate),
            RIDE_STRETCH_EASE,
        )
    } else {
        0.0
    };

    w.lerp_height += (lerp_height - w.lerp_height) * 0.05;
    w.lerp_height += (1.0 - w.lerp_height) * w.flight;

    w.blend.z += (RIDE_BLEND - w.blend.z) * 0.25;
    w.radius += w.blend.z * (radius_far - w.radius);
    w.latitude_near += w.blend.z * (latitude_near - w.latitude_near);
    w.latitude += w.blend.z * (latitude - w.latitude);
    w.latitude_gain += w.blend.z * (latitude_gain - w.latitude_gain);

    let want_radius = w.radius + w.latitude_near * stretch;
    let want_latitude = w.latitude + w.latitude_gain * stretch;
    let want_fovy = fovy_base + fovy_far * stretch;

    w.offset.y = height + height_extra * stretch;
    w.offset.z += (gap_z * stretch - w.offset.z) * 0.1;
    w.offset.x *= 0.95;

    let wanted = SGlobe::new(
        want_radius,
        SAngle::from_degrees(want_latitude),
        w.rider.inv(),
    );

    if camera.setup_age == 0 {
        w.lerp_height = lerp_height;
        w.settle = 1.0;
    } else {
        w.settle += (1.0 - w.settle) * 0.025;
    }

    let mut hold = cam_follow::relational_pos(player, facing, w.offset);
    if let Some(hit) = input
        .collision
        .and_then(|solid| solid.line_cross(player.pos, hold))
    {
        hold = hit.point + hit.normal;
    }

    w.blend.y += (1.0 - w.blend.y) * 0.2;
    w.walk += (hold - w.walk) * w.blend.y;

    if !camera.settled {
        let frames = if camera.mode == 1 { 8 } else { 20 };
        let share = 1.0 / (frames as f32 - camera.setup_age as f32).max(1.0);
        cache.center += (w.walk - cache.center) * share;
        let current = cache.direction;
        cache.direction = SGlobe::new(
            current.radius() + (wanted.radius() - current.radius()) * share,
            current.latitude() + (wanted.latitude() - current.latitude()) * share,
            current.longitude() + (wanted.longitude() - current.longitude()) * share,
        );
        cache.eye = cache.center + cache.direction.to_xyz();
        cache.fovy += share * (want_fovy - cache.fovy);
        if camera.setup_age as i32 >= frames - 1 {
            camera.settled = true;
        }
        camera.work.tp.ride = w;
        camera.view_cache = cache;
        cam_follow::publish(camera);
        return true;
    }

    cache.center +=
        (w.walk - cache.center) * Vec3::new(lerp_lateral, w.lerp_height, lerp_lateral);
    let mut held = SGlobe::from_xyz(cache.eye - cache.center);
    let radius = cache.direction.radius() + w.lerp * (wanted.radius() - cache.direction.radius());

    let rate = (chase * blend + chase * (1.0 - blend) * span) * w.settle;
    let stick = input.cstick.x;
    let swung = !camera.has_flag(0x040) && (stick.abs() > RIDE_YAW_GATE || w.swung);
    let want_yaw = if swung {
        let swing = cache.direction.longitude()
            + SAngle::from_degrees(rational_bezier_ratio(stick, 0.5) * RIDE_YAW_SWING);
        w.swung = true;
        w.settle = 0.0;
        cache.direction.longitude()
            + (swing - cache.direction.longitude()) * (stick.abs() - RIDE_YAW_GATE)
    } else if camera.has_flag(0x1000) {
        cache.direction.longitude() + (facing.inv() - cache.direction.longitude()) * rate
    } else {
        cache.direction.longitude() + (wanted.longitude() - cache.direction.longitude()) * rate
    };
    if !swung {
        w.settle += (1.0 - w.settle) * 0.01;
    }

    let mut latitude_rate = 0.25f32;
    if camera.has_flag(0x100) {
        latitude_rate = 0.4;
    }
    if w.airborne {
        latitude_rate += (1.0 - latitude_rate) * w.flight;
        held.set_latitude(held.latitude() + SAngle::from_degrees(10.0));
    }
    let mut wanted = wanted;
    wanted.set_latitude(wanted.latitude() + (held.latitude() - wanted.latitude()) * latitude_rate);

    let mut latitude = cache.direction.latitude()
        + (wanted.latitude() - cache.direction.latitude()) * w.lerp;
    if latitude < SAngle::from_degrees(RIDE_LATITUDE_MIN) {
        latitude = SAngle::from_degrees(RIDE_LATITUDE_MIN);
    } else if latitude > SAngle::from_degrees(RIDE_LATITUDE_MAX) {
        latitude = SAngle::from_degrees(RIDE_LATITUDE_MAX);
    }

    cache.direction = SGlobe::new(radius, latitude, want_yaw);
    cache.eye = cache.center + cache.direction.to_xyz();
    cache.fovy += w.lerp * (want_fovy - cache.fovy);

    cache.bank += (SAngle::from_degrees(bank * input.follow.main.x) - cache.bank)
        * (span.abs() * 0.01);
    camera.event_flags |= 0x400;

    camera.work.tp.ride = w;
    camera.view_cache = cache;
    cam_follow::publish(camera);
    true
}