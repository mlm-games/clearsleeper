use glam::Vec3;

use super::cam_engine::{Event, EventView, evflag};
use super::cam_follow::{self, SAngle, SGlobe, ViewCache};
use super::camera::Camera;
use super::collision::Collision;
use super::event_manager::EventManager;
use super::player::Player;

/// `eventCamera`'s action table, `d_camera.cpp:4713-4742`, index for index against the
/// twenty-eight-entry function table beside it at `:4682-4710`.
///
/// The sibling reference carries thirty-four entries against this table's twenty-eight and its
/// own names are *not* the same order: its thirteenth and fourteenth rows are `pauseEvCamera`
/// and `portalWarpEvCamera` where this table has `windDirectionEvCamera` and
/// `tornadoWarpEvCamera`, and it appends `STBWAIT`, `CURRENT`, `PEEPHOLE`, `DIGHOLE`, `HINTTALK`
/// and `BSPTRANS` between `UNIACCELE` and `MAPTOOL`. Every shared name was therefore matched by
/// name and never by index; only `PAUSE` and `WAIT` land on the same handler in both games, and
/// the six rows this table names that the sibling does not are the two above plus nothing else.
///
/// `WINDDIR` and `TORNADO` are the two rows with no counterpart at all in the sibling's readable
/// text, so their arms name what they need rather than borrowing a body.
pub const ACTION_NAMES: [&str; 28] = [
    "PAUSE",
    "WAIT",
    "TALK",
    "FIXEDPOS",
    "FIXEDFRM",
    "UNITRANS",
    "WATCHACTOR",
    "RESTOREPOS",
    "GETITEM",
    "GAMEOVER",
    "TURNTOACTOR",
    "ROLLING",
    "TACT",
    "WINDDIR",
    "TORNADO",
    "STYLE",
    "SAVE",
    "LOAD",
    "USEITEM0",
    "USEITEM1",
    "FIXEDFRMS",
    "BSPLINE",
    "POSSESSED",
    "TWOACTOR0",
    "STOKER",
    "UNIBRAKE",
    "UNIACCELE",
    "MAPTOOL",
];

/// `dCamera_c::eventCamera`'s no-type answer, `d_camera.cpp:4796`: an action index outside the
/// table ends the cut and declines.
const ACTION_NONE: i32 = -1;

/// The three curve tables the uniform transition's `BSpCurve` number selects and the key counts
/// `d2DBSplinePath::Init` is given beside them, `sibling d_ev_camera.cpp:955-960`. Index zero of
/// the key table is the plain linear ramp, which uses no table at all, and index four names no
/// table in the reference, so the arms run only for one through three.
const BSP_CURVE_A: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
const BSP_CURVE_B: [f32; 7] = [0.0, 0.0, 0.4, 0.7, 0.9, 1.0, 1.0];
const BSP_CURVE_C: [f32; 7] = [0.0, 0.0, 0.1, 0.3, 0.6, 1.0, 1.0];
const BSP_CURVE_KEYS: [i32; 4] = [0, 4, 7, 7];

/// `loadEvCamera`'s own four-key curve, `sibling d_ev_camera.cpp:2626`, and its six slots.
const LOAD_CURVE: [f32; 6] = [0.0, 0.0, 1.0, 1.0, 0.0, 0.0];
const LOAD_CURVE_KEYS: i32 = 4;

/// `getItemEvCamera`'s own six-key curve, `sibling d_ev_camera.cpp:3178-3179`, used when the cut
/// names no `CurvePoints`.
const ITEM_CURVE: [f32; 6] = [0.0, 0.0, 0.1, 0.7, 1.0, 1.0];
const ITEM_CURVE_KEYS: i32 = 6;

/// `d2DBSplinePath`, `d_spline_path.cpp:10-76`: the second-order basis `mCurveWeight` and the
/// three keys `mCurveKey` the four event arms read a cut's key list through.
///
/// `mFrame`, `mKeyNum`, `mState`, `mKeyNo`, `mEnd`, `mDuration` and `mStep` are
/// `mFrame`, `mKeyNum`, `mState`, `mKeyNo`, `mEnd`, `mDuration` and `mStep` in the order the
/// target's header declares them, `d_spline_path.h:39-52`. The target's `Step` carries one test
/// the sibling's does not: it also ends the run when the frame reaches `mEnd - 1`, `d_spline_path.cpp:42-45`,
/// which matters only because `Init` gives `mEnd` and `mDuration` the same number.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spline {
    frame: i32,
    key_num: i32,
    state: i32,
    key_no: i32,
    end: i32,
    duration: i32,
    step: f32,
    weight: [f32; 3],
    key: [i32; 3],
}

impl Spline {
    /// `d2DBSplinePath::Init`, `d_spline_path.cpp:10-20`.
    pub fn new(key_num: i32, duration: i32) -> Self {
        let keys = if key_num < 2 { 1 } else { key_num - 2 };
        let frames = if duration != 0 { duration - 1 } else { 1 };
        Self {
            frame: 0,
            key_num,
            state: 1,
            key_no: 0,
            end: duration,
            duration,
            step: keys as f32 / frames as f32,
            weight: [0.0; 3],
            key: [0; 3],
        }
    }

    /// `d2DBSplinePath::Step`, `d_spline_path.cpp:24-63`.
    pub fn step(&mut self) -> bool {
        if self.frame >= self.duration {
            self.state = 0;
        }
        if self.state == 1 {
            self.state = 2;
        }
        if self.state == 2 {
            if self.frame <= self.end - 1 {
                let mut t = self.step * self.frame as f32;
                self.key_no = t as i32;
                t -= self.key_no as f32;
                if self.frame == self.duration - 1 {
                    self.state = 3;
                }
                if self.frame == self.end - 1 {
                    self.state = 3;
                    t = 1.0;
                }
                self.key = [
                    (self.key_no).min(self.key_num - 1),
                    (self.key_no + 1).min(self.key_num - 1),
                    (self.key_no + 2).min(self.key_num - 1),
                ];
                let inv = 1.0 - t;
                self.weight = [inv * inv * 0.5, t * inv + 0.5, t * t * 0.5];
                self.frame += 1;
            } else {
                self.state = 0;
            }
        }
        self.state == 2 || self.state == 3
    }

    /// `d2DBSplinePath::Calc(f32*)`, `d_spline_path.cpp:74-76`.
    pub fn calc(&self, keys: &[f32]) -> Option<f32> {
        let read = |at: usize| keys.get(at).copied();
        Some(
            self.weight[0] * read(self.key[0] as usize)?
                + self.weight[1] * read(self.key[1] as usize)?
                + self.weight[2] * read(self.key[2] as usize)?,
        )
    }

    /// `d2DBSplinePath::Calc(cXyz*)`, `d_spline_path.cpp:65-72`.
    pub fn calc_xyz(&self, keys: &[Vec3]) -> Option<Vec3> {
        let read = |at: usize| keys.get(at).copied();
        Some(
            self.weight[0] * read(self.key[0] as usize)?
                + self.weight[1] * read(self.key[1] as usize)?
                + self.weight[2] * read(self.key[2] as usize)?,
        )
    }

    /// `d2DBSplinePath::Last`, which the target declares as an empty member, `d_spline_path.h:31`,
    /// and the sibling defines as the state-three test, `sibling d_spline_path.h:52`. Only
    /// `bSplineEvCamera` reads it, and only the sibling's definition is available, so this is
    /// that definition and the target gives no second one to choose between.
    pub fn last(&self) -> bool {
        self.state == 3
    }
}

impl Default for Spline {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

/// What one of the five event getters answers for an actor name.
///
/// The sibling's `getEvActor` resolves `'@PLA'` to the player, `'@STA'` and `'@PAR'` to the
/// event's own two participants, `'@TAL'` and `'@DOO'` to the talk and door partners, `'@TAR'` and
/// `'@ITE'` to the item partner, `'Link'` to the built-in hero and anything else through the
/// actor-name search, `sibling d_ev_camera.cpp:397-435` and `:437-475`. The target keeps all of
/// those participants on its event object, `d_event.h:120-122` and `d_com_inf_game.h:3562-3589`,
/// but the port publishes none of them: the event manager carries records and no actors, and the
/// one system that does track participants keeps them private. So the only name this port can
/// answer is the player's, and every other name is [`EvActor::Missing`], which is what makes
/// `stokerEvCamera`, `watchActorEvCamera` and `twoActor0EvCamera` take their own
/// "actor missing" arms and what makes the `RelActor` arms of the other handlers stand absolute.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvActor {
    Player,
    Missing,
}

/// The five event getters and the two list getters beside them, over the port's event manager.
///
/// The target's getters are `void getEvIntData(int*, char*)`, `void getEvIntData(int*, char*, int)`,
/// `void getEvFloatData(f32*, char*, f32)`, `void getEvXyzData(cXyz*, char*, cXyz)` and
/// `void getEvActor(char*)` against the sibling's `bool` and actor-pointer returns,
/// `d_ev_camera.cpp:25-68` against `sibling d_ev_camera.cpp:96-127`, `:253-280`, `:282-308`,
/// `:310-336` and `:397-435`. The only getter that keeps a return in the target is
/// `getEvStringData`, `d_ev_camera.cpp:51-53`, and this port keeps the sibling's returns on all
/// of them, because every arm below tests the found flag: the pause arm stores it, the uniform
/// transition treats a missing `Timer` as a hard error, the spline arm treats a missing `Timer`
/// the same way, and four more arms store it. Which member the target parks that flag in instead
/// is not recoverable: its getter bodies are stubs and its `EventData` has no boolean field the
/// text names.
///
/// The other divergence is the varargs table. The target's `StartEventCamera` takes up to eight
/// `(name, is_reference, value)` triples and `searchEventArgData` finds one by name,
/// `d_ev_camera.cpp:10-13` and `:20-23`, and every getter here consults it first when the
/// forced-camera flag is set, `sibling d_ev_camera.cpp:97-102`. This port runs no such table, so
/// a feed with no manager behind it answers every read with its own default, which is what the
/// target does when the forced flag is set and its argument table names nothing.
///
/// One thing the port cannot answer at all: the two list getters. `getEvFloatListData` and
/// `getEvXyzListData` return a pointer and a count, `sibling d_ev_camera.cpp:158-188` and
/// `:190-220`, and the target's counterpart is not declared at all, `d_ev_camera.cpp:25-68`. The
/// event manager exposes a count per key and one scalar, never the array behind them, so
/// [`Feed::float_list`] and [`Feed::xyz_list`] answer the count and name the array as missing.
/// That is what holds `fixedFramesEvCamera` and `bSplineEvCamera` at their first frame and what
/// holds `getItemEvCamera`'s and `uniformTransEvCamera`'s list arms.
pub struct Feed<'a> {
    manager: Option<&'a mut EventManager>,
    staff: i32,
}

impl<'a> Feed<'a> {
    /// A feed over a staff of a loaded event list, and a feed with nothing behind it, which
    /// answers every read with the default the getter was handed.
    pub fn new(manager: Option<&'a mut EventManager>, staff: i32) -> Self {
        Self { manager, staff }
    }

    /// `dCamera_c::mEventData.mStaffIdx`, the staff every getter below reads through.
    pub fn staff(&self) -> i32 {
        self.staff
    }

    fn int_of(&self, key: &str) -> Option<i32> {
        self.manager
            .as_ref()
            .and_then(|manager| manager.get_my_int(self.staff, key))
    }

    fn float_of(&self, key: &str) -> Option<f32> {
        self.manager
            .as_ref()
            .and_then(|manager| manager.get_my_float(self.staff, key))
    }

    fn vec_of(&self, key: &str) -> Option<Vec3> {
        self.manager
            .as_ref()
            .and_then(|manager| manager.get_my_vec(self.staff, key))
    }

    fn text_of(&self, key: &str) -> Option<&str> {
        self.manager
            .as_ref()
            .and_then(|manager| manager.get_my_text(self.staff, key))
    }

    /// `getEvIntData(int*, char*)`, `sibling d_ev_camera.cpp:96-127`: the value with no default,
    /// `None` for a key the cut does not carry. The sibling also raises the missing-parameter
    /// latch `mEventData.field_0x10`, which no readable line in either game reads back.
    pub fn int(&mut self, key: &str) -> Option<i32> {
        self.int_of(key)
    }

    /// `getEvIntData(int*, char*, int)`, `sibling d_ev_camera.cpp:253-280`.
    pub fn int_or(&mut self, key: &str, fallback: i32) -> i32 {
        self.int_of(key).unwrap_or(fallback)
    }

    /// `getEvFloatData(f32*, char*, f32)`, `sibling d_ev_camera.cpp:282-308`.
    pub fn float_or(&mut self, key: &str, fallback: f32) -> f32 {
        self.float_of(key).unwrap_or(fallback)
    }

    /// `getEvFloatData(f32*, char*)`'s found test. The sibling has no two-argument float getter
    /// and reaches this through the four arms that store the flag beside the value, which is
    /// [`Feed::int`] here for the two arms that store an integer's.
    pub fn float(&mut self, key: &str) -> Option<f32> {
        self.float_of(key)
    }

    /// `getEvXyzData(cXyz*, char*, cXyz)`, `sibling d_ev_camera.cpp:310-336`.
    pub fn xyz_or(&mut self, key: &str, fallback: Vec3) -> Vec3 {
        self.vec_of(key).unwrap_or(fallback)
    }

    /// `getEvStringData(char*, char*, char*)`, `sibling d_ev_camera.cpp:338-364`: the string, or
    /// `None` for a key the cut does not carry. This is the one getter whose return the target
    /// keeps, `d_ev_camera.cpp:51-53`, and `eventCamera` itself is its only reader, at
    /// `d_camera.cpp:4808`; its third argument, the default to copy out, is the caller's to
    /// decide, because the caller is the only one that knows what to leave the field at.
    pub fn string(&mut self, key: &str) -> Option<String> {
        self.text_of(key).map(str::to_owned)
    }

    /// `getEvStringPntData(char*, char*)`, `sibling d_ev_camera.cpp:366-395`.
    pub fn string_or(&mut self, key: &str, fallback: &str) -> String {
        self.text_of(key).unwrap_or(fallback).to_owned()
    }

    /// `getEvActor(char*)`, `sibling d_ev_camera.cpp:397-435`, which asks for the string with no
    /// default and so answers no actor at all for a key the cut does not carry.
    pub fn actor(&mut self, key: &str) -> EvActor {
        match self.text_of(key) {
            Some(name) => resolve_actor(name),
            None => EvActor::Missing,
        }
    }

    /// `getEvActor(char*, char*)`, `sibling d_ev_camera.cpp:437-475`, which asks for the string
    /// with a default and so resolves the default when the cut names nothing.
    pub fn actor_or(&mut self, key: &str, fallback: &str) -> EvActor {
        resolve_actor(self.text_of(key).unwrap_or(fallback))
    }

    /// `getEvFloatListData(f32**, char*)`, `sibling d_ev_camera.cpp:158-188`: the key count, with
    /// the array itself unavailable. See the note on [`Feed`].
    pub fn float_list(&mut self, key: &str) -> Option<usize> {
        let count = self.substance_num(key);
        (count != 0).then_some(count as usize)
    }

    /// `getEvXyzListData(cXyz**, char*)`, `sibling d_ev_camera.cpp:190-220`: the key count, with
    /// the array itself unavailable. See the note on [`Feed`].
    pub fn xyz_list(&mut self, key: &str) -> Option<usize> {
        self.float_list(key)
    }

    /// The substance count both list getters gate on, reached directly where a handler wants the
    /// count without asking for an array.
    pub fn substance_num(&self, key: &str) -> i32 {
        self.manager
            .as_ref()
            .map_or(0, |manager| manager.get_my_substance_num(self.staff, key))
    }

    /// The event manager's own end-of-cut flag write, `d_camera.cpp:4797`. A feed with nothing
    /// behind it writes nothing, which is what `d_camera.cpp:4797` does with a staff index of
    /// `-1`.
    pub fn cut_end(&mut self) {
        let staff = self.staff;
        if let Some(manager) = self.manager.as_deref_mut() {
            manager.cut_end(staff);
        }
    }

    /// `d_camera.cpp:4786`'s action lookup, which asks the manager for this staff's cut name
    /// against the twenty-eight-entry table and latches the answer on the staff, so the next
    /// frame repeats it until the cut advances. A feed with nothing behind it answers no action,
    /// which is `ACTION_NONE`.
    pub fn act_idx(&mut self) -> i32 {
        let staff = self.staff;
        self.manager.as_deref_mut().map_or(ACTION_NONE, |manager| {
            manager.get_my_act_idx(staff, &ACTION_NAMES, false, 0)
        })
    }
}

/// The four-character comparison every actor name is resolved by,
/// `sibling d_ev_camera.cpp:404-420`: the first four bytes read as one word, so `"@PLAYER"` and
/// `"@PLA"` are the same name and `"@STARTER"` is `'@STA'`.
fn resolve_actor(name: &str) -> EvActor {
    let mut word = [0u8; 4];
    for (lane, byte) in name.bytes().take(4).enumerate() {
        word[lane] = byte;
    }
    if word == *b"@PLA" {
        EvActor::Player
    } else {
        EvActor::Missing
    }
}

/// `dCamera_c::relationalPos`'s one-actor form, `d_camera.cpp:1594-1605`: the offset turned
/// into the actor's yaw frame and added to its attention point.
///
/// The target's `attentionPos` is the actor's own attention record, `d_camera.cpp:1589-1592`, and
/// this port has none for any actor, so the position stands in for it exactly as
/// [`cam_follow::relational_pos`] already does for the player.
fn relational(player: &Player, facing: SAngle, offset: Vec3) -> Vec3 {
    let mut globe = SGlobe::from_xyz(offset);
    globe.set_longitude(facing + globe.longitude());
    player.pos + globe.to_xyz()
}

/// `dCamMath::xyzRotateY`, `d_cam_param.cpp:81-89`: `x' = c*x + s*z`, `z' = -s*x + c*z`.
fn rotate_y(v: Vec3, angle: SAngle) -> Vec3 {
    let (sin, cos) = (angle.sin(), angle.cos());
    Vec3::new(cos * v.x + sin * v.z, v.y, -sin * v.x + cos * v.z)
}

/// `lineBGCheck(cXyz*, cXyz*, u32)`, `d_camera.cpp:1849-1853`, under the `0x4007` the event
/// handlers pass it. The target reads three flag bits off that word, `d_camera.cpp:1794-1835`,
/// and this port's own line check answers one answer, so the flags make no difference here.
fn blocked(collision: Option<&Collision>, from: Vec3, to: Vec3) -> bool {
    collision.is_some_and(|solid| solid.line_cross(from, to).is_some())
}

/// `dCamera_c::pointInSight`, `d_camera.cpp:1659-1673`: the point inside the framed viewport.
fn in_sight(camera: &Camera, point: Vec3) -> bool {
    camera.project(point).is_some_and(|proj| {
        proj.x > 0.0 && proj.x < camera.port.width && proj.y > 0.0 && proj.y < camera.port.height
    })
}

/// `dCamera_c::Reset()`, `d_camera.cpp:4949-4965`: the view cache off the published triple.
///
/// The target's `mFovy` and `mBank` are separate published members and this port has neither,
/// so those two writes are identities and the bank's own decay is left to
/// [`cam_follow::publish`].
fn reset(camera: &mut Camera) {
    let mut cache = camera.view_cache;
    cache.center = camera.bump.center;
    cache.eye = camera.bump.eye;
    cache.direction = SGlobe::from_xyz(cache.eye - cache.center);
    camera.view_cache = cache;
}

/// `pauseEvCamera`'s state, `sibling d_camera.cpp:423-437`. The `mFovy` and `mBank` members of the
/// same struct are written and read by no line of the sibling's body and are left out.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pause {
    /// `field_0x0`: the cut named a `Timer`, so the arm holds until it runs out.
    pub found: bool,
    /// `field_0x8`: the timer.
    pub timer: i32,
    /// `field_0x4`: `Stay`, which raises event flag bit zero.
    pub stay: i32,
}

impl Default for Pause {
    fn default() -> Self {
        Self {
            found: false,
            timer: 0,
            stay: 0,
        }
    }
}

/// `fixedFrameEvCamera`'s state, `sibling d_camera.cpp:450-461`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FixedFrame {
    /// `field_0x4`: the eye.
    pub eye: Vec3,
    /// `field_0x10`: the centre.
    pub center: Vec3,
    /// `field_0x1c`: the field of view.
    pub fovy: f32,
    /// `field_0x20`: the bank, only honoured when `banked` holds.
    pub bank: f32,
    /// `field_0x30`: the cut named a bank.
    pub banked: bool,
    /// `field_0x2c`: the timer, only honoured when `found` holds.
    pub timer: i32,
    /// `field_0x0`: the cut named a timer.
    pub found: bool,
    /// `mRelUseMask`: the centre's and the eye's relation characters.
    pub mask: [u8; 2],
    /// `mpRelActor`: whether the cut named a relative actor, which this port cannot resolve to
    /// anything but the player.
    pub relative: bool,
}

impl Default for FixedFrame {
    fn default() -> Self {
        Self {
            eye: Vec3::ZERO,
            center: Vec3::ZERO,
            fovy: 0.0,
            bank: 0.0,
            banked: false,
            timer: 0,
            found: false,
            mask: *b"oo",
            relative: false,
        }
    }
}

/// `stokerEvCamera`'s state, `sibling d_camera.cpp:463-479`. Both actor members stay
/// [`EvActor::Missing`] in this port, so the body takes its own "actor missing" arm.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stoker {
    /// `field_0x4`: the eye's offset from the stoker.
    pub eye_gap: Vec3,
    /// `field_0x10`: the centre's offset from the target.
    pub ctr_gap: Vec3,
    /// `field_0x1c`: the centre's cushion.
    pub ctr_cus: f32,
    /// `field_0x20`: the eye's cushion.
    pub eye_cus: f32,
    /// `field_0x24`: the field of view.
    pub fovy: f32,
    /// `field_0x28`: the bank, only honoured when `banked` holds.
    pub bank: f32,
    /// `field_0x1`: the cut named a bank.
    pub banked: bool,
    /// `field_0x40`: the timer, only honoured when `found` holds.
    pub timer: i32,
    /// `field_0x0`: the cut named a timer.
    pub found: bool,
}

impl Default for Stoker {
    fn default() -> Self {
        Self {
            eye_gap: Vec3::ZERO,
            ctr_gap: Vec3::ZERO,
            ctr_cus: 1.0,
            eye_cus: 1.0,
            fovy: 0.0,
            bank: 0.0,
            banked: false,
            timer: 0,
            found: false,
        }
    }
}

/// `rollingEvCamera`'s state, `sibling d_camera.cpp:481-498`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rolling {
    /// `field_0x4`: the eye.
    pub eye: Vec3,
    /// `field_0x10`: the centre.
    pub center: Vec3,
    /// `mEye` and `mCenter`: the eye and the centre as the cut named them, which the relation
    /// arms edit before the two derived members above are latched.
    pub want_eye: Vec3,
    pub want_center: Vec3,
    /// `mFovy`: the field of view.
    pub fovy: f32,
    /// `mBank`, only honoured when `banked` holds.
    pub bank: f32,
    /// `field_0x1`: the cut named a bank.
    pub banked: bool,
    /// `mRoll`: the degrees the eye's longitude gains a frame.
    pub roll: f32,
    /// `mRadiusAdd`: the units the eye's radius gains a frame.
    pub radius_add: f32,
    /// `mLatitude`: the degrees the eye's latitude is pinned to for `TransType` two.
    pub latitude: f32,
    /// `mCtrCus`: the centre's cushion.
    pub ctr_cus: f32,
    /// `mTimer`, only honoured when `found` holds.
    pub timer: i32,
    /// `field_0x0`: the cut named a timer.
    pub found: bool,
    /// `mTransType`: which arms run.
    pub trans_type: i32,
    /// `mRelActor`: the centre's and the eye's relation characters.
    pub mask: [u8; 2],
    /// `mpRelActor`: whether the cut named a relative actor.
    pub relative: bool,
}

impl Default for Rolling {
    fn default() -> Self {
        Self {
            eye: Vec3::ZERO,
            center: Vec3::ZERO,
            want_eye: Vec3::ZERO,
            want_center: Vec3::ZERO,
            fovy: 0.0,
            bank: 0.0,
            banked: false,
            roll: 2.0,
            radius_add: 0.0,
            latitude: 0.0,
            ctr_cus: 1.0,
            timer: 0,
            found: false,
            trans_type: 0,
            mask: *b"oo",
            relative: false,
        }
    }
}

/// `fixedPositionEvCamera`'s state, `sibling d_camera.cpp:500-516`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FixedPos {
    /// `field_0x4`: the eye.
    pub eye: Vec3,
    /// `field_0x10`: `CtrGap`, the centre's offset from the target.
    pub ctr_gap: Vec3,
    /// `field_0x28`: the field of view.
    pub fovy: f32,
    /// `field_0x2c`: the bank, only honoured when `banked` holds.
    pub bank: f32,
    /// `field_0x1`: the cut named a bank.
    pub banked: bool,
    /// `field_0x30`: the centre's cushion.
    pub ctr_cus: f32,
    /// `field_0x38`: the radius the eye is held to.
    pub radius: f32,
    /// `field_0x34`: the radius it walks from while the timer runs.
    pub start_radius: f32,
    /// `field_0x4c`: the timer, only honoured when `found` holds.
    pub timer: i32,
    /// `field_0x0`: the cut named a timer.
    pub found: bool,
    /// `field_0x48`: the eye's relation character.
    pub mask: u8,
    /// `field_0x40`: whether the target resolved, which in this port is the player.
    pub target: bool,
}

impl Default for FixedPos {
    fn default() -> Self {
        Self {
            eye: Vec3::ZERO,
            ctr_gap: Vec3::ZERO,
            fovy: 0.0,
            bank: 0.0,
            banked: false,
            ctr_cus: 1.0,
            radius: 100000.0,
            start_radius: 100000.0,
            timer: 0,
            found: false,
            mask: b'o',
            target: false,
        }
    }
}

/// `uniformTransEvCamera`'s shared state, `sibling d_camera.cpp:518-561`, which the three uniform
/// arms enter with the mode one, two and three respectively, `sibling d_ev_camera.cpp:920-930`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Trans {
    /// `mStartCenter` and `mStartEye`: the pair the transition walks from.
    pub start_center: Vec3,
    pub start_eye: Vec3,
    /// `mCenter` and `mEye`: the pair it walks to.
    pub center: Vec3,
    pub eye: Vec3,
    /// `mStartFovy`, `mFovy`: the field-of-view pair.
    pub start_fovy: f32,
    pub fovy: f32,
    /// `field_0x1c` and `field_0x3c`: the bank pair, in degrees.
    pub start_bank: f32,
    pub bank: f32,
    /// `field_0x68`: either end named a bank, so the bank's own blend runs.
    pub banked: bool,
    /// `mCushion`: the cushion the centre, the eye and the field of view all run through.
    pub cushion: f32,
    /// `mTimer`: the frames the transition runs over.
    pub timer: i32,
    /// `mTransType`: which blend runs, one through four.
    pub trans_type: i32,
    /// `mBSpCurve`: the curve number, negative one when the cut named a key list.
    pub curve: i32,
    /// `mBSpPoints`: the key count the cut named, which this port cannot read the values of.
    pub points: usize,
    /// `field_0x60`: the camera's own direction turned inside out, read by the `'c'` mask arm.
    pub inverted: SGlobe,
    /// `mRelUseMask`: the four relation characters.
    pub mask: [u8; 4],
    /// `mRelActor`: whether the cut named a relative actor.
    pub relative: bool,
    /// `field_0xf0`: the curve the run is stepped through.
    pub spline: Spline,
}

impl Default for Trans {
    fn default() -> Self {
        Self {
            start_center: Vec3::ZERO,
            start_eye: Vec3::ZERO,
            center: Vec3::ZERO,
            eye: Vec3::ZERO,
            start_fovy: 0.0,
            fovy: 0.0,
            start_bank: 0.0,
            bank: 0.0,
            banked: false,
            cushion: 1.0,
            timer: 0,
            trans_type: 0,
            curve: 0,
            points: 0,
            inverted: SGlobe::new(0.0, SAngle::ZERO, SAngle::ZERO),
            mask: *b"--oo",
            relative: false,
            spline: Spline::default(),
        }
    }
}

/// `restorePosEvCamera`'s state, `sibling d_camera.cpp:563-606`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RestorePos {
    /// `field_0x0`: `CtrGap`, the centre's offset from the target.
    pub ctr_gap: Vec3,
    /// `field_0xc`: the target's own relational centre.
    pub target_center: Vec3,
    /// `field_0x18`: the cushion.
    pub cushion: f32,
    /// `field_0x20`: the near distance.
    pub near_dist: f32,
    /// `field_0x28`: the far distance.
    pub far_dist: f32,
    /// `field_0x1c`: the near timer, the frames the near arms walk over.
    pub near_timer: i32,
    /// `field_0x24`: the far timer, the frames the middle arms walk over.
    pub far_timer: i32,
    /// `field_0x38`: the direction from the destination's centre to its eye, latched on the first
    /// frame of the walk.
    pub direction: SGlobe,
    /// `field_0x40`: which of the four arms runs.
    pub arm: i32,
    /// `field_0x48`: the destination view.
    pub dest: Option<EventView>,
    /// `field_0x68`: `TargetType`, where one aims the walk at the destination's own centre.
    pub target_type: i32,
}

impl Default for RestorePos {
    fn default() -> Self {
        Self {
            ctr_gap: Vec3::ZERO,
            target_center: Vec3::ZERO,
            cushion: 1.0,
            near_dist: 750.0,
            far_dist: 1500.0,
            near_timer: 20,
            far_timer: 30,
            direction: SGlobe::new(0.0, SAngle::ZERO, SAngle::ZERO),
            arm: 0,
            dest: None,
            target_type: 0,
        }
    }
}

/// `loadEvCamera`'s state, `sibling d_camera.cpp:608-637`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Load {
    /// `field_0x1c` and `field_0x0c`: the centre and the direction the walk starts from.
    pub start_center: Vec3,
    pub start: SGlobe,
    /// `field_0x14`: the direction it walks to.
    pub want: SGlobe,
    /// `field_0x28` and `field_0x34`: the centre and the eye it walks to.
    pub dest_center: Vec3,
    pub dest_eye: Vec3,
    /// `field_0x40` and `field_0x42`: the bank pair.
    pub start_bank: SAngle,
    pub dest_bank: SAngle,
    /// `field_0x44` and `field_0x48`: the field-of-view pair.
    pub start_fovy: f32,
    pub dest_fovy: f32,
    /// `mSlot`, `mTimer` and `mType`.
    pub slot: i32,
    pub timer: i32,
    pub curve_type: i32,
    /// `field_0xf0`: the curve the walk is stepped through.
    pub spline: Spline,
}

impl Default for Load {
    fn default() -> Self {
        Self {
            start_center: Vec3::ZERO,
            start: SGlobe::new(0.0, SAngle::ZERO, SAngle::ZERO),
            want: SGlobe::new(0.0, SAngle::ZERO, SAngle::ZERO),
            dest_center: Vec3::ZERO,
            dest_eye: Vec3::ZERO,
            start_bank: SAngle::ZERO,
            dest_bank: SAngle::ZERO,
            start_fovy: 0.0,
            dest_fovy: 0.0,
            slot: 0,
            timer: 0,
            curve_type: 0,
            spline: Spline::default(),
        }
    }
}

/// `getItemEvCamera`'s state, `sibling d_camera.cpp:639-655`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GetItem {
    /// `mStartCenter` and `mStartEye`.
    pub start_center: Vec3,
    pub start_eye: Vec3,
    /// `field_0x3c` and `field_0x64`: the direction pair.
    pub start: SGlobe,
    pub want: SGlobe,
    /// `field_0x44` and `field_0x50`: the centre and the eye it walks to.
    pub dest_center: Vec3,
    pub dest_eye: Vec3,
    /// `mTimer`: the frames the walk runs over.
    pub timer: i32,
    /// `field_0x8`: the type, which selects the pair of table offsets below.
    pub item_type: i32,
    /// `field_0x18`: the curve key count.
    pub keys: i32,
    /// `mRelActor`: whether the cut named a relative actor, which this port resolves to the
    /// player or to nothing.
    pub relative: bool,
    /// `field_0xf0`: the curve the walk is stepped through.
    pub spline: Spline,
}

impl Default for GetItem {
    fn default() -> Self {
        Self {
            start_center: Vec3::ZERO,
            start_eye: Vec3::ZERO,
            start: SGlobe::new(0.0, SAngle::ZERO, SAngle::ZERO),
            want: SGlobe::new(0.0, SAngle::ZERO, SAngle::ZERO),
            dest_center: Vec3::ZERO,
            dest_eye: Vec3::ZERO,
            timer: 0,
            item_type: 0,
            keys: ITEM_CURVE_KEYS,
            relative: false,
            spline: Spline::default(),
        }
    }
}

/// `possessedEvCamera`'s state, `sibling d_camera.cpp:657-670`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Possessed {
    /// `field_0x0`: which of the three arms runs.
    pub arm: i32,
    /// `mType`: whether the arm saves the outgoing view or takes the incoming one.
    pub item_type: i32,
    /// `field_0xc`: the frames left in the walk.
    pub left: i32,
    /// `mTimer`: the frames the walk runs over.
    pub timer: i32,
    /// `mRadius`, `mLatitude` and `mLongitude`: the globe the eye converges on.
    pub radius: f32,
    pub latitude: SAngle,
    pub longitude: SAngle,
    /// `mFovy`: the field of view the walk ends on.
    pub fovy: f32,
    /// `mCushion`: the eye's own cushion.
    pub cushion: f32,
    /// `field_0x28`: the globe the walk converges on.
    pub want: SGlobe,
    /// `mBlure`: the blur arm, which this port has no renderer for.
    pub blure: i32,
}

impl Default for Possessed {
    fn default() -> Self {
        Self {
            arm: 0,
            item_type: 0,
            left: 0,
            timer: 0,
            radius: 0.0,
            latitude: SAngle::ZERO,
            longitude: SAngle::ZERO,
            fovy: 0.0,
            cushion: 1.0,
            want: SGlobe::new(0.0, SAngle::ZERO, SAngle::ZERO),
            blure: 0,
        }
    }
}

/// `fixedFramesEvCamera`'s state, `sibling d_camera.cpp:672-684`. Every member is latched on the
/// first frame out of a key list, which this port cannot read, so the whole arm stays at the
/// frame it latches and holds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FixedFrames {
    /// `field_0x0` and `mTimer`: whether the cut named a timer, and the timer.
    pub found: bool,
    pub timer: i32,
    /// `field_0x38`: how many frames the three lists agreed on.
    pub frames: i32,
    /// `field_0x3c`: `Choice`, which picks a frame to start on.
    pub choice: i32,
    /// `field_0x4` and `field_0x10`: the centre and the eye the arm holds.
    pub center: Vec3,
    pub eye: Vec3,
    /// `field_0x28`: the field of view the arm holds.
    pub fovy: f32,
    /// `mRelUseMask`.
    pub mask: [u8; 2],
    /// `mRelActor`: whether the cut named a relative actor.
    pub relative: bool,
}

impl Default for FixedFrames {
    fn default() -> Self {
        Self {
            found: false,
            timer: 0,
            frames: 0,
            choice: 0,
            center: Vec3::ZERO,
            eye: Vec3::ZERO,
            fovy: 0.0,
            mask: *b"oo",
            relative: false,
        }
    }
}

/// `bSplineEvCamera`'s state, `sibling d_camera.cpp:686-695`. Every member is latched out of a
/// key list too, so the arm holds at the frame it latches.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BSpline {
    /// `field_0x1c`: the walk is over.
    pub done: bool,
    /// `mTimer`: the frames the walk runs over.
    pub timer: i32,
    /// `field_0x10`: how many frames the three lists agreed on.
    pub frames: i32,
    /// `mRelActor`: whether the cut named a relative actor.
    pub relative: bool,
    /// `field_0xf0`: the curve the walk is stepped through.
    pub spline: Spline,
}

impl Default for BSpline {
    fn default() -> Self {
        Self {
            done: false,
            timer: 0,
            frames: 0,
            relative: false,
            spline: Spline::default(),
        }
    }
}

/// `twoActor0EvCamera`'s state, `sibling d_camera.cpp:697-731`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TwoActor {
    /// `mActor1` and `mActor2`: whether each resolved. The second is `@STARTER` by default, which
    /// this port cannot resolve, so the arm takes its own "actor missing" return.
    pub actor1: bool,
    pub actor2: bool,
    /// `mCtrGap`, `mCtrRatio`, `mCtrCus` and `mEyeCus`.
    pub ctr_gap: Vec3,
    pub ctr_ratio: f32,
    pub ctr_cus: f32,
    pub eye_cus: f32,
    /// `mRadiusMin`, `mRadiusMax`, `mLatitudeMin`, `mLatitudeMax`, `mLongitudeMin` and
    /// `mLongitudeMax`.
    pub radius_min: f32,
    pub radius_max: f32,
    pub latitude_min: SAngle,
    pub latitude_max: SAngle,
    pub longitude_min: SAngle,
    pub longitude_max: SAngle,
    /// `mLongitudeType` and `mFovy`.
    pub longitude_type: i32,
    pub fovy: f32,
    /// `field_0x4c`, `field_0x54`, `field_0x56`: the globe the eye walks onto.
    pub radius: f32,
    pub latitude: SAngle,
    pub longitude: SAngle,
    /// `field_0x50`, `field_0x58` and `field_0x5a`: the three targets it walks onto.
    pub want_radius: f32,
    pub want_latitude: SAngle,
    pub want_longitude: SAngle,
}

impl Default for TwoActor {
    fn default() -> Self {
        Self {
            actor1: false,
            actor2: false,
            ctr_gap: Vec3::ZERO,
            ctr_ratio: -0.1,
            ctr_cus: 1.0,
            eye_cus: 1.0,
            radius_min: 100.0,
            radius_max: 10000.0,
            latitude_min: SAngle::from_degrees(-60.0),
            latitude_max: SAngle::from_degrees(60.0),
            longitude_min: SAngle::from_degrees(20.0),
            longitude_max: SAngle::from_degrees(160.0),
            longitude_type: -1,
            fovy: 60.0,
            radius: 0.0,
            latitude: SAngle::ZERO,
            longitude: SAngle::ZERO,
            want_radius: 0.0,
            want_latitude: SAngle::ZERO,
            want_longitude: SAngle::ZERO,
        }
    }
}

/// `getItemEvCamera`'s five rows of eye and centre offsets and their field of view, frame count
/// and side flag, `sibling d_ev_camera.cpp:3152-3176`. The offsets themselves are the eight
/// `cStack_90` entries at `:3152-3161`, taken in pairs.
const ITEM_OFFSETS: [Vec3; 8] = [
    Vec3::new(0.0, -27.0, 32.0),
    Vec3::new(84.0, -18.0, 134.0),
    Vec3::new(17.0, 10.0, -45.0),
    Vec3::new(-45.0, 128.0, 45.0),
    Vec3::new(0.0, -27.0, -62.0),
    Vec3::new(-84.0, -18.0, -164.0),
    Vec3::new(-12.0, 10.0, -130.0),
    Vec3::new(50.0, 128.0, -220.0),
];
/// `getItemEvCamera`'s per-row centre and eye offset indices into [`ITEM_OFFSETS`], field of
/// view, frame count and side flag, `sibling d_ev_camera.cpp:3170-3176`. The side flag is one for
/// "negate the eye's lateral gap when the camera is behind the relative actor", negative one for
/// the same test the other way, and zero for neither.
const ITEM_ROWS: [(usize, usize, f32, i32, i32); 5] = [
    (0, 1, 50.0, 12, 1),
    (2, 3, 50.0, 12, 0),
    (4, 5, 50.0, 17, -1),
    (4, 5, 50.0, 17, -1),
    (6, 7, 50.0, 17, 0),
];

/// Every piece of state the event engine keeps across frames, beside the cut bookkeeping on
/// [`Event`]. This is the target's one `mWork` union, `d_camera.h:378-380`, split by arm; the
/// sibling's union carries the same thirteen arms beside its own, `sibling d_camera.h:450-731`,
/// and no arm reads another arm's members, so the split loses nothing.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct State {
    pub pause: Pause,
    pub fixed_frame: FixedFrame,
    pub stoker: Stoker,
    pub rolling: Rolling,
    pub fixed_pos: FixedPos,
    pub trans: Trans,
    pub restore_pos: RestorePos,
    pub load: Load,
    pub get_item: GetItem,
    pub possessed: Possessed,
    pub fixed_frames: FixedFrames,
    pub b_spline: BSpline,
    pub two_actor: TwoActor,
}

/// `dCamera_c::eventCamera`, `d_camera.cpp:4679-4885`, and the twenty arms behind its table.
///
/// `manager` is the target's event manager, the `dComIfGp` global the guard at `:4762` and
/// `:4766` and the selector at `:4786` all read. The engine dispatch in
/// [`super::cam_engine`] cannot hand it over, because `EngineInput` is built by a struct literal
/// in the camera's own track pass; with `None` the guard declines at `:4768-4770` and the engine
/// settles, which is what it did before this module existed. Passing the manager is the one
/// wiring change needed to make the table live.
///
/// The forced branch, `d_camera.cpp:4788-4794`, needs no manager at all: the action comes off
/// [`Event::action`] and every parameter comes off the forced flag's argument table, which this
/// port does not carry, so a feed with nothing behind it stands in and answers every read with
/// its own default. That is what the target does when the argument table names nothing, and it is
/// why the twenty arms run on the forced branch with default parameters.
///
/// What the twenty arms here do not carry, named once: the varargs argument table behind
/// `StartEventCamera`, the wolf and companion adjustments the sibling's relation arms make, the
/// actor search behind every non-player actor name, the two key-list getters behind four of the
/// arms, and the stage's map-event table behind `maptoolIdEvCamera`. Each is named again on the
/// arm it stops.
pub fn event(
    camera: &mut Camera,
    player: &Player,
    collision: Option<&Collision>,
    manager: Option<&mut EventManager>,
) -> bool {
    let mut cut = camera.work.event;

    if cut.age == 0 {
        let view = EventView {
            center: camera.bump.center,
            eye: camera.bump.eye,
            fovy: camera.view_cache.fovy,
            bank: camera.view_cache.bank,
            age: 0,
        };
        cut.saved = [view, view];
    }

    let forced = camera.event_flags & evflag::EVENT_PLAY != 0;

    if forced {
        cut.staff = -1;
        if cut.age == 0 {
            camera.setup_age = 0;
        }
        if !(ACTION_NONE..ACTION_NAMES.len() as i32).contains(&cut.action) {
            camera.work.event = cut;
            return false;
        }
        run(
            camera,
            player,
            collision,
            &mut Feed::new(None, cut.staff),
            &mut cut,
        );
        return true;
    }

    let Some(manager) = manager else {
        camera.work.event = cut;
        return false;
    };
    if manager.camera_play == 0 {
        camera.work.event = cut;
        return false;
    }
    let staff = manager.get_my_staff_id("CAMERA", None, 0);
    if staff < 0 {
        camera.work.event = cut;
        return false;
    }
    if cut.staff != staff {
        camera.event_flags &= !evflag::FIXED;
        camera.setup_age = 0;
        camera.age = 0;
        cut.age = 0;
    }
    cut.staff = staff;
    if manager.get_is_advance(cut.staff) {
        camera.settled = false;
        camera.setup_age = 0;
    }
    let mut feed = Feed::new(Some(manager), cut.staff);
    cut.action = feed.act_idx();
    if !(ACTION_NONE..ACTION_NAMES.len() as i32).contains(&cut.action) {
        feed.cut_end();
        camera.work.event = cut;
        return false;
    }
    run(camera, player, collision, &mut feed, &mut cut);
    true
}

/// The trim name the target copies out when a cut names none, `d_camera.cpp:4808`. Its first four
/// bytes are `'CINE'`, which the target's own list at `:4810-4824` does not test, so the field is
/// left at whatever it held; the sibling's default is `'DEFAULT'` and its list does carry a
/// `'CINE'` arm, `sibling d_camera.cpp:10134-10163`. Nothing reads this name, because the getter
/// hands back nothing at all when the cut names no trim, so the branch below never runs; it is
/// here so the target's default is written down.
pub const TRIM_DEFAULT: &str = "CINESCO";

/// Everything `eventCamera` does between the selector and the dispatch, `d_camera.cpp:4801-4884`.
///
/// The target's `m118` is [`Event::age`], its `m11C` is `mCurCamStyleTimer`, which this port
/// publishes as [`Camera::setup_age`], and its `m100`/`m101`/`m102` triple is
/// [`Camera::settled`].
///
/// Three of the writes here have no consumer in this port and are carried on [`Event`] instead:
/// `mTrimSize` off `mEventData.field_0x1c`, `d_camera.cpp:4857`, which the trim calculator and
/// `SetTrimSize` read; the `AutoForcus` zero's `offAutoForcus`, `d_camera.cpp:4836-4838`, which
/// is a graph-blender call; and the attention-status pair on the settle triple,
/// `d_camera.cpp:4860-4869`, which this port does not track — [`Camera::settled`] already *is*
/// that triple, so the pair is the port's own zero. `m068`, the collision word, is carried too,
/// because [`super::cam_bump`] keeps its own and nothing hands it this one.
///
/// The trailing line check, `d_camera.cpp:4876-4881`, runs on `mEventData.field_0x20` four and
/// pulls the eye onto the cross. The target guards it behind its own build-version test,
/// `d_camera.cpp:4875`, which is true for the retail disc, so it runs here too.
fn run(
    camera: &mut Camera,
    player: &Player,
    collision: Option<&Collision>,
    feed: &mut Feed<'_>,
    cut: &mut Event,
) {
    if camera.setup_age == 0 {
        if cut.age == 0 {
            camera.event_flags &= !evflag::FIXED;
            cut.trim = 2;
        }
        if let Some(trim) = feed.string("Trim") {
            let head = &trim.as_bytes()[..4.min(trim.len())];
            cut.trim = match head {
                b"STAN" => 0,
                b"VIST" => 1,
                b"DEMO" => 3,
                b"NONE" => 4,
                b"KEEP" => 999,
                _ => cut.trim,
            };
        }
        cut.wait_key = feed.int_or("WaitAnyKey", 0);
        if cut.wait_key != 0 {
            camera.event_flags |= evflag::FIXED;
        }
        cut.bg_check = feed.int_or("BGCheck", 1);
        cut.auto_focus = feed.int_or("AutoForcus", 1);
        cut.move_bg = feed.int_or("MoveBGCheck", 1);
    }

    cut.bump_flags = match cut.bg_check {
        1 => 0xF,
        2 => 0x3F,
        3 | 4 => 0,
        _ => 9,
    };

    let finished = dispatch(camera, player, collision, feed, cut);
    camera.work.event = *cut;

    if cut.bg_check == 4 {
        let (center, eye) = (camera.view_cache.center, camera.view_cache.eye);
        if let Some(hit) = collision.and_then(|solid| solid.line_cross(center, eye)) {
            camera.view_cache.eye = hit.point;
        }
    }

    if finished {
        feed.cut_end();
    }
}

/// The twenty-eight-way dispatch on the action index, `d_camera.cpp:4871`.
fn dispatch(
    camera: &mut Camera,
    player: &Player,
    collision: Option<&Collision>,
    feed: &mut Feed<'_>,
    cut: &mut Event,
) -> bool {
    let name = ACTION_NAMES[cut.action as usize];
    match name {
        "PAUSE" | "WAIT" => pause(camera, feed),
        "TALK" => talkto(camera),
        "FIXEDPOS" => fixed_position(camera, player, feed),
        "FIXEDFRM" => fixed_frame(camera, player, collision, feed),
        "UNITRANS" => trans(camera, player, collision, feed, cut, 1),
        "RESTOREPOS" => restore_pos(camera, player, collision, feed, cut),
        "GETITEM" => get_item(camera, player, collision, feed),
        "TURNTOACTOR" => turn_to_actor(camera),
        "ROLLING" => rolling(camera, player, collision, feed),
        "WINDDIR" => wind_direction(camera),
        "TORNADO" => tornado_warp(camera),
        "STYLE" => style(camera, feed, cut),
        "SAVE" => save(camera, feed, cut),
        "LOAD" => load(camera, feed, cut),
        "FIXEDFRMS" => fixed_frames(camera, feed),
        "BSPLINE" => b_spline(camera, feed),
        "POSSESSED" => possessed(camera, player, feed, cut),
        "TWOACTOR0" => two_actor(camera, feed),
        "STOKER" => stoker(camera, player, feed),
        "UNIBRAKE" => trans(camera, player, collision, feed, cut, 2),
        "UNIACCELE" => trans(camera, player, collision, feed, cut, 3),
        "MAPTOOL" => maptool_id(camera, feed, cut),
        "WATCHACTOR" => watch_actor(camera, feed),
        "GAMEOVER" => game_over(camera),
        "TACT" => tact(camera),
        "USEITEM0" => use_item0(camera),
        "USEITEM1" => use_item1(camera),
        _ => false,
    }
}

/// `dCamera_c::pauseEvCamera`, `sibling d_ev_camera.cpp:477-496`.
///
/// Nothing is left out. The sibling latches the found flag into `field_0x0` because its getter
/// returns it; the target's getter is `void`, `d_ev_camera.cpp:25-27`, so the target's own arm
/// has to reach the flag another way this port cannot see. The reset the first frame runs is
/// [`reset`], and `Stay` raises [`evflag::STAY`], which is event flag bit zero.
fn pause(camera: &mut Camera, feed: &mut Feed<'_>) -> bool {
    let mut w = camera.work.ev.pause;
    if camera.setup_age == 0 {
        reset(camera);
        camera.settled = true;
        match feed.int("Timer") {
            Some(timer) => {
                w.found = true;
                w.timer = timer;
            }
            None => {
                w.found = false;
                w.timer = -1;
            }
        }
        w.stay = feed.int_or("Stay", 0);
    }
    if w.stay != 0 {
        camera.event_flags |= evflag::STAY;
    }
    let hold = w.found && (camera.setup_age as i32) < w.timer;
    camera.work.ev.pause = w;
    !hold
}

/// `dCamera_c::talktoEvCamera`, `sibling d_ev_camera.cpp:1824-1844`.
///
/// Half of it runs. The first frame's `clrFlag(0x200000)` and the `'TT01'` fallback both run; the
/// style itself cannot, because it comes out of `mEventData.field_0x0c`, the camera type the
/// engine that declined belonged to. The target's `Run` is the only place that member is written
/// from and it is a stub, `d_camera.cpp:814-823`, so the type this arm would read is not published
/// anywhere the port can reach; it is `mCurType` before the run switches to the event type, and
/// `Camera::cam_type` has already been switched by the time the event engine runs.
///
/// What the arm produces is `talktoCamera`'s own frame, which this port also does not carry: it
/// is a stub in both games, `d_camera.cpp:3677`, and the port's own `CamEngine::Talkto` is an
/// empty function that settles and declines, `cam_tp_engines.rs:456-459`. It needs a second
/// actor: its listener and speaker, their attention points, eye points, base points and facings,
/// and the message system's camera cut number.
fn talkto(camera: &mut Camera) -> bool {
    if camera.setup_age == 0 {
        camera.event_flags &= !evflag::FIXED;
    }
    false
}

/// `dCamera_c::styleEvCamera`, `sibling d_ev_camera.cpp:1923-1932`.
///
/// The first frame's two zeroing writes run. The style comes from a four-character name the cut
/// carries, defaulting to `'FN01'`, and the frame it produces is another engine's, chosen by that
/// style's own row; the port's dispatch cannot be redirected from inside an arm, so the arm
/// settles and declines where the sibling calls `engine_tbl`.
///
/// `isModeOK()` is the target's own empty member, `d_camera.h:667`, and the sibling's return is
/// the settle triple this port already carries as [`Camera::settled`]. The second zeroing write,
/// `field_0x160`, is a main-stick delta this port has no member for; the first, `field_0x8`, is
/// [`Event::found`]. The name is read and the style it names is not resolved, because nothing
/// downstream of the resolution exists in this port.
fn style(camera: &mut Camera, feed: &mut Feed<'_>, cut: &mut Event) -> bool {
    if camera.setup_age == 0 {
        cut.found = 0;
    }
    let _name = feed.string_or("Name", "FN01");
    camera.settled
}

/// `dCamera_c::saveEvCamera`, `sibling d_ev_camera.cpp:2610-2622`.
///
/// The slot write runs for slots zero and one, which are the two views `eventCamera` snapshots
/// and every other arm reads back, `d_camera.cpp:4744-4756`. Slot nine is the save file through
/// `dComIfGp_saveCameraPosition`, which this port has no writer for, so the arm settles there as
/// the sibling does.
fn save(camera: &mut Camera, feed: &mut Feed<'_>, cut: &mut Event) -> bool {
    let slot = feed.int_or("Slot", 0).max(0) as usize;
    if let Some(view) = cut.saved.get_mut(slot) {
        view.center = camera.view_cache.center;
        view.eye = camera.view_cache.eye;
        view.fovy = camera.view_cache.fovy;
        view.bank = camera.view_cache.bank;
        view.age = 1;
    }
    camera.settled = true;
    true
}

/// `dCamera_c::turnToActorEvCamera`, `sibling d_ev_camera.cpp:2552-2554`, which is `return true`
/// and nothing else.
///
/// The target's stub claims 1528 bytes of text for this arm, `d_ev_camera.cpp:151-159`, against
/// the sibling's three lines, so the target almost certainly does something here and its text
/// cannot say what. Nothing is invented: the sibling's answer stands and the doubt is recorded.
fn turn_to_actor(camera: &mut Camera) -> bool {
    let _ = camera;
    true
}

/// `dCamera_c::windDirectionEvCamera`, which the target names at index thirteen,
/// `d_camera.cpp:4727`, and which has no counterpart in the sibling's readable text at all: the
/// sibling's thirteenth row is `pauseEvCamera` against a `WINDDIR` name,
/// `sibling d_camera.cpp:9942` against `sibling d_camera.cpp:9979`.
///
/// The name says the arm frames along a wind direction and the target's own text says nothing
/// more, so nothing is written. It needs the stage's wind vector, which the target reads from the
/// weather system this port does not carry.
fn wind_direction(camera: &mut Camera) -> bool {
    let _ = camera;
    false
}

/// `dCamera_c::tornadoWarpEvCamera`, which the target names at index fourteen,
/// `d_camera.cpp:4728`, and which the sibling names `portalWarpEvCamera` at its own index
/// fourteen, `sibling d_ev_camera.cpp:4087-4089`.
///
/// The sibling's arm is on the map's warp table, which this port has no reader for, so the arm
/// reports rather than standing in. It needs the map's warp-node list, its own per-node
/// destination table and the sky-warp transition the target plays.
fn tornado_warp(camera: &mut Camera) -> bool {
    let _ = camera;
    false
}

/// `dCamera_c::watchActorEvCamera`, `sibling d_ev_camera.cpp:1410-1711`, one of the six arms
/// another pass owns.
///
/// Its whole body is built on a second actor: `Target` defaults to `'@STARTER'`, and the arm
/// accepts and reports when that is missing, `sibling d_ev_camera.cpp:1427-1430`, which is
/// exactly what this port produces on its first frame, because [`EvActor::Missing`] is the only
/// answer the port has for any name but the player's; on the later frames the sibling instead
/// dereferences the null actor and this port holds. Behind the guard the arm also needs the
/// target's own actor id and its profile name, `sibling d_ev_camera.cpp:1450-1459`, the attention
/// point, the eye point and the facing of a second actor, and the point-in-sight test against the
/// framed viewport, which this port does have.
fn watch_actor(camera: &mut Camera, feed: &mut Feed<'_>) -> bool {
    if camera.setup_age == 0 {
        feed.xyz_or("CtrGap", Vec3::ZERO);
        feed.float_or("Cushion", 1.0);
        feed.float_or("NearDist", 750.0);
        feed.float_or("ZoomDist", 400.0);
        feed.float_or("ZoomVAngle", 0.0);
        feed.float_or("FarDist", 1500.0);
        feed.float_or("Fovy", 62.0);
        feed.int_or("NearTimer", 20);
        feed.int_or("FarTimer", 30);
        feed.float_or("FrontAngle", 179.0);
        feed.int_or("Blure", 0);
        if feed.actor_or("Target", "@STARTER") == EvActor::Missing {
            return true;
        }
    }
    false
}

/// `dCamera_c::gameOverEvCamera`, `sibling d_ev_camera.cpp:1934-2272`, one of the six arms
/// another pass owns.
///
/// It frames the player and the companion from a table of five fixed eye and centre pairs the
/// cut chooses with `Type`, `sibling d_ev_camera.cpp:1946-1990`, and its whole selection is
/// keyed on `checkMidnaRide` and the companion's own no-draw flag, `sibling d_ev_camera.cpp:1942`
/// and `:1987-1990`. This port carries no wolf, no companion and no ride state, so the
/// selection has no answer; the arm reports.
fn game_over(camera: &mut Camera) -> bool {
    let _ = camera;
    false
}

/// `dCamera_c::tactEvCamera`, `sibling d_ev_camera.cpp:2274-2550`, one of the six arms another
/// pass owns.
///
/// It is a small talk camera built on six centre and eye pairs and seven more, chosen by the
/// cut's own `Type`, `sibling d_ev_camera.cpp:2276-2336`, and it alternates between two of them
/// on the global frame counter's low bit, `sibling d_ev_camera.cpp:2338-2342`, which it latches
/// into `mEventData.field_0x20` and so reaches the camera's own collision flags. It needs the
/// two actors it frames between, the same pair `CamEngine::Talkto` needs and does not have.
fn tact(camera: &mut Camera) -> bool {
    let _ = camera;
    false
}

/// `dCamera_c::useItem0EvCamera`, `sibling d_ev_camera.cpp:2689-2955`, one of the six arms another
/// pass owns.
///
/// It picks one of eleven centre and eye offset pairs the cut names with `Type`,
/// `sibling d_ev_camera.cpp:2691-2760`, blends the centre onto the player and the eye onto the
/// pair over a measured number of frames, and hands a blur the port has no renderer for. It
/// needs the player's item state, which the cut's `Type` is matched against, and the screen
/// blur's own position, scale and alpha writer.
fn use_item0(camera: &mut Camera) -> bool {
    let _ = camera;
    false
}

/// `dCamera_c::useItem1EvCamera`, `sibling d_ev_camera.cpp:2957-3148`, one of the six arms another
/// pass owns.
///
/// Its type zero arm is the small pull-in, `sibling d_ev_camera.cpp:2961-2990`, which needs only
/// the player's attention point and so is the one part of any of the six this port could reach;
/// it is left to that pass because it is that pass's arm. Its other arms are six more offset
/// pairs keyed on the wolf and the companion, `sibling d_ev_camera.cpp:2992-3050`.
fn use_item1(camera: &mut Camera) -> bool {
    let _ = camera;
    false
}

/// `dCamera_c::maptoolIdEvCamera`, `sibling d_ev_camera.cpp:1846-1921`.
///
/// The first frame's two zeroing writes and its two parameter reads run, and then the arm accepts,
/// which is what the sibling does whenever the stage's map-event record is absent: it defaults
/// `CameraID` to `0xFF` and declines the cut on that, `sibling d_ev_camera.cpp:1858-1870`. This
/// port has no map-event record, so the arm takes that arm, and it therefore accepts whatever
/// `CameraID` the cut names.
///
/// The rest is the stage's own map-event table: `searchMapEventData` by the stage's map-tool id,
/// its `MAPTOOLCAMERA` record's `CameraID`, the room it belongs to, its two flag words and
/// `GetCameraTypeFromMapToolID`, `sibling d_ev_camera.cpp:1854-1917`. The port resolves a camera
/// type from a map-tool id already, [`super::cam_room::CamRoom::type_from_map_tool_id`], but it
/// is the stage-wide entry only and takes its arrow from the room table, not from a map-event
/// record's flag words. The `0x20` and `0x80` flag words at
/// `sibling d_ev_camera.cpp:1877-1885` and `:1913-1916` need the record, and so does the stage
/// event check at `:1873`, which the target spells `isStageEvent`.
fn maptool_id(camera: &mut Camera, feed: &mut Feed<'_>, cut: &mut Event) -> bool {
    if camera.setup_age == 0 {
        feed.int_or("ID", -1);
        cut.found = 0;
    }
    let _camera_id = feed.int_or("CameraID", 0xFF);
    true
}

/// `dCamera_c::fixedPositionEvCamera`, `sibling d_ev_camera.cpp:849-918`.
///
/// The one thing left out is the target's actor-id liveness test, `sibling d_ev_camera.cpp:884-887`,
/// which needs the process id the port does not carry; with the target resolved to the player,
/// which is what `Target`'s `'@PLAYER'` default gives, that test can only ever pass.
fn fixed_position(camera: &mut Camera, player: &Player, feed: &mut Feed<'_>) -> bool {
    let facing = SAngle::from_radians(player.face);
    let mut w = camera.work.ev.fixed_pos;
    let mut cache = camera.view_cache;
    let mut done = true;

    if camera.setup_age == 0 {
        w.ctr_gap = feed.xyz_or("CtrGap", Vec3::ZERO);
        let eye = feed.xyz_or("Eye", camera.bump.eye);
        w.fovy = feed.float_or("Fovy", camera.view_cache.fovy);
        w.ctr_cus = feed.float_or("CtrCus", 1.0);
        w.radius = feed.float_or("Radius", 100000.0);
        w.start_radius = feed.float_or("StartRadius", w.radius);
        w.bank = feed.float_or("Bank", 0.0);
        w.banked = feed.float("Bank").is_some();
        let mask = feed.string_or("RelUseMask", "o");
        w.mask = mask.as_bytes().first().copied().unwrap_or(b'x');
        w.timer = feed.int_or("Timer", -1);
        w.found = feed.int("Timer").is_some();
        w.target = feed.actor_or("Target", "@PLAYER") != EvActor::Missing;
        if !w.target {
            camera.work.ev.fixed_pos = w;
            return true;
        }
        w.eye = if feed.actor("RelActor") != EvActor::Missing && is_rel_char(w.mask) {
            relational(player, facing, eye)
        } else {
            eye
        };
        camera.settled = true;
    }

    if !w.target {
        camera.work.ev.fixed_pos = w;
        return true;
    }

    let target = relational(player, facing, w.ctr_gap);
    cache.center += (target - cache.center) * w.ctr_cus;
    cache.eye = w.eye;
    cache.direction = SGlobe::from_xyz(cache.eye - cache.center);

    let mut radius = w.radius;
    if w.found && (camera.setup_age as i32) < w.timer {
        let span = if w.timer != 0 {
            camera.setup_age as f32 / w.timer as f32
        } else {
            1.0
        };
        radius = w.start_radius + (w.radius - w.start_radius) * span;
        done = false;
    }

    if cache.direction.radius() > radius {
        cache.direction.set_radius(radius);
        cache.eye = cache.center + cache.direction.to_xyz();
    }

    cache.fovy = w.fovy;
    if w.banked {
        cache.bank = SAngle::from_degrees(w.bank);
        camera.event_flags |= evflag::BANK;
    }

    camera.work.ev.fixed_pos = w;
    camera.view_cache = cache;
    cam_follow::publish(camera);
    done
}

/// `isRelChar`, `sibling d_ev_camera.cpp:844-846`: anything but a dash or an `x` reads as a
/// relation the cut asked for.
fn is_rel_char(letter: u8) -> bool {
    letter != b'-' && letter != b'x'
}

/// `dCamera_c::fixedFrameEvCamera`, `sibling d_ev_camera.cpp:502-634`.
///
/// Two things are left out and neither is reachable in the target. The wolf adjustment the
/// `'w'` and `'W'` arms add to the gap, `sibling d_ev_camera.cpp:527-537` and `:566-576`, is a
/// wolf-only offset the target has no actor for, so those two arms are the plain relation arms;
/// and the `'r'` arm's frame-parity flip, `sibling d_ev_camera.cpp:578-580`, reads `mTicks`, a
/// per-`Run` counter at offset `0x0AC` the target's own header does not carry at all. That flip
/// reads [`Camera::age`]'s parity here, which is the target's `m108` and runs on the same cadence.
///
/// One read is left out because nothing in the sibling reads what it writes: `BasePos`,
/// `sibling d_ev_camera.cpp:508`, fills `mBasePos` and no line of that body or of the eleven
/// arms beside it looks at it again.
fn fixed_frame(
    camera: &mut Camera,
    player: &Player,
    collision: Option<&Collision>,
    feed: &mut Feed<'_>,
) -> bool {
    let facing = SAngle::from_radians(player.face);
    let mut w = camera.work.ev.fixed_frame;
    let mut cache = camera.view_cache;

    if camera.setup_age == 0 {
        let eye = feed.xyz_or("Eye", camera.bump.eye);
        let center = feed.xyz_or("Center", camera.bump.center);
        w.fovy = feed.float_or("Fovy", camera.view_cache.fovy);
        w.bank = feed.float_or("Bank", 0.0);
        w.banked = feed.float("Bank").is_some();
        w.timer = feed.int_or("Timer", -1);
        w.found = feed.int("Timer").is_some();
        let mask = feed.string_or("RelUseMask", "oo");
        let bytes = mask.as_bytes();
        w.mask[0] = bytes.first().copied().unwrap_or(b'x');
        w.mask[1] = bytes.get(1).copied().unwrap_or(b'x');
        let relative = feed.actor("RelActor");
        w.relative = relative != EvActor::Missing;

        w.center = if w.relative {
            relation_arm(player, facing, center, w.mask[0], camera)
        } else {
            center
        };
        w.eye = if w.relative && w.mask[1] == b'r' {
            flip_side(player, facing, eye, camera, w.center, collision)
        } else if w.relative {
            relation_arm(player, facing, eye, w.mask[1], camera)
        } else {
            eye
        };
        camera.settled = true;
    }

    cache.center = w.center;
    cache.eye = w.eye;
    cache.direction = SGlobe::from_xyz(cache.eye - cache.center);
    cache.fovy = w.fovy;
    if w.banked {
        cache.bank = SAngle::from_degrees(w.bank);
        camera.event_flags |= evflag::BANK;
    }

    let hold = w.found && (camera.setup_age as i32) < w.timer;
    camera.work.ev.fixed_frame = w;
    camera.view_cache = cache;
    cam_follow::publish(camera);
    !hold
}

/// The relation arms of `fixedFrameEvCamera` and `rollingEvCamera`,
/// `sibling d_ev_camera.cpp:523-561` and `:722-748`: the plain relation for the letters that ask
/// for one, the side the camera is already on for `'n'`, the nearer of the two sides for `'p'`,
/// the attention point alone for `'t'`, and the gap alone for anything else.
///
/// The `'w'` and `'W'` letters are the plain relation here. In the sibling they add and subtract
/// a fixed offset when the player is a wolf and is being carried, `sibling d_ev_camera.cpp:527-537`
/// and `:566-576`, and the target has neither a wolf nor a carrier, so the offset is gone.
fn relation_arm(player: &Player, facing: SAngle, gap: Vec3, letter: u8, camera: &Camera) -> Vec3 {
    match letter {
        b'o' | b'w' | b'W' => relational(player, facing, gap),
        b'n' => {
            let mut turned = gap;
            if SGlobe::from_xyz(camera.bump.eye - player.pos).longitude() - facing < SAngle::ZERO {
                turned.x = -turned.x;
            }
            relational(player, facing, turned)
        }
        b'p' => {
            let near = relational(player, facing, gap);
            let mut turned = gap;
            turned.x = -turned.x;
            let far = relational(player, facing, turned);
            if (near - player.pos).length() > (far - player.pos).length() {
                turned.x = -turned.x;
            }
            relational(player, facing, turned)
        }
        b't' => player.pos + gap,
        _ => gap,
    }
}

/// The `'r'` relation arm, `sibling d_ev_camera.cpp:577-587` and `:753-761`: the gap's lateral
/// half is negated on odd frames and again whenever the line from the centre to the candidate eye
/// is blocked, which picks the side the camera can see the actor from.
///
/// The odd-frame half reads [`Camera::age`]'s parity, because `mTicks` at offset `0x0AC` is not a
/// member the target's header carries; `m108` is the target's per-`Run` counter and runs on the
/// same cadence.
fn flip_side(
    player: &Player,
    facing: SAngle,
    gap: Vec3,
    camera: &Camera,
    center: Vec3,
    collision: Option<&Collision>,
) -> Vec3 {
    let mut turned = gap;
    if camera.age & 1 != 0 {
        turned.x = -turned.x;
    }
    let first = relational(player, facing, turned);
    let mut flipped = gap;
    flipped.x = -flipped.x;
    if blocked(collision, center, first) {
        relational(player, facing, flipped)
    } else {
        first
    }
}

/// `dCamera_c::stokerEvCamera`, `sibling d_ev_camera.cpp:636-703`.
///
/// The body is whole; it takes its own "stoker or target actor missing" arm on the first frame,
/// `sibling d_ev_camera.cpp:651-654`, which is where this port lands because `Stoker` defaults to
/// `'@STARTER'`, and again on either actor's death, `sibling d_ev_camera.cpp:662-666` and
/// `:676-680`. Both actors' shape angles, the pitch and the yaw the centre and eye offsets turn
/// on, read as level, which is what every latitude lerp in the port converges on anyway.
fn stoker(camera: &mut Camera, player: &Player, feed: &mut Feed<'_>) -> bool {
    let mut w = camera.work.ev.stoker;
    let mut cache = camera.view_cache;

    if camera.setup_age == 0 {
        w.eye_gap = feed.xyz_or("EyeGap", Vec3::ZERO);
        w.ctr_gap = feed.xyz_or("CtrGap", Vec3::ZERO);
        w.eye_cus = feed.float_or("EyeCus", 1.0);
        w.ctr_cus = feed.float_or("CtrCus", 1.0);
        w.fovy = feed.float_or("Fovy", camera.view_cache.fovy);
        w.bank = feed.float_or("Bank", 0.0);
        w.banked = feed.float("Bank").is_some();
        w.timer = feed.int_or("Timer", -1);
        w.found = feed.int("Timer").is_some();
        if feed.actor_or("Stoker", "@STARTER") == EvActor::Missing
            || feed.actor_or("Target", "@PLAYER") == EvActor::Missing
        {
            camera.work.ev.stoker = w;
            return true;
        }
        camera.settled = true;
    }

    let target = feed.actor_or("Target", "@PLAYER");
    if target == EvActor::Player {
        let mut globe = SGlobe::from_xyz(w.ctr_gap);
        globe.set_latitude(facing_of(player) + globe.longitude());
        cache.center += (player.pos + globe.to_xyz() - cache.center) * w.ctr_cus;
    }

    let stoker = feed.actor_or("Stoker", "@STARTER");
    if stoker == EvActor::Player {
        let mut globe = SGlobe::from_xyz(w.eye_gap);
        globe.set_latitude(facing_of(player) + globe.longitude());
        cache.eye += (player.pos + globe.to_xyz() - cache.eye) * w.eye_cus;
    }

    cache.direction = SGlobe::from_xyz(cache.eye - cache.center);
    cache.fovy = w.fovy;
    if w.banked {
        cache.bank = SAngle::from_degrees(w.bank);
        camera.event_flags |= evflag::BANK;
    }

    let hold = w.found && (camera.setup_age as i32) < w.timer;
    camera.work.ev.stoker = w;
    camera.view_cache = cache;
    cam_follow::publish(camera);
    !hold
}

/// `dCamera_c::directionOf`, `d_camera.cpp:1579-1581`.
fn facing_of(player: &Player) -> SAngle {
    SAngle::from_radians(player.face)
}

/// `dCamera_c::rollingEvCamera`, `sibling d_ev_camera.cpp:705-839`.
///
/// Two things are left out. The wolf and companion adjustments on the start pair, which the
/// sibling's uniform transition shares and this arm never had, do not appear here; and the `'r'`
/// arm's frame-parity flip reads [`Camera::age`]'s parity here, because `mTicks` at offset
/// `0x0AC` is not a member the target's header carries.
fn rolling(
    camera: &mut Camera,
    player: &Player,
    collision: Option<&Collision>,
    feed: &mut Feed<'_>,
) -> bool {
    let facing = facing_of(player);
    let mut w = camera.work.ev.rolling;
    let mut cache = camera.view_cache;

    if camera.setup_age == 0 {
        w.want_eye = feed.xyz_or("Eye", camera.bump.eye);
        w.want_center = feed.xyz_or("Center", camera.bump.center);
        w.ctr_cus = feed.float_or("CtrCus", 1.0);
        w.trans_type = feed.int_or("TransType", 0);
        w.fovy = feed.float_or("Fovy", camera.view_cache.fovy);
        w.bank = feed.float_or("Bank", 0.0);
        w.banked = feed.float("Bank").is_some();
        w.roll = feed.float_or("Roll", 2.0);
        w.radius_add = feed.float_or("RadiusAdd", 0.0);
        w.latitude = feed.float_or(
            "Latitude",
            SGlobe::from_xyz(w.want_eye - w.want_center)
                .latitude()
                .degree(),
        );
        w.timer = feed.int_or("Timer", -1);
        w.found = feed.int("Timer").is_some();
        let mask = feed.string_or("RelUseMask", "oo");
        let bytes = mask.as_bytes();
        w.mask[0] = bytes.first().copied().unwrap_or(b'x');
        w.mask[1] = bytes.get(1).copied().unwrap_or(b'x');
        let relative = feed.actor("RelActor");
        w.relative = relative != EvActor::Missing;

        if w.relative {
            w.center = relation_arm(player, facing, w.want_center, w.mask[0], camera);
            w.eye = if w.mask[1] == b'r' {
                flip_side(player, facing, w.want_eye, camera, w.center, collision)
            } else {
                relation_arm(player, facing, w.want_eye, w.mask[1], camera)
            };
        } else {
            w.center = w.want_center;
            w.eye = w.want_eye;
        }
        camera.settled = true;
    }

    if (w.trans_type == 1 || w.trans_type == 2) && w.relative {
        w.center = relation_arm(player, facing, w.want_center, w.mask[0], camera);
    }

    cache.center += (w.center - cache.center) * w.ctr_cus;
    cache.direction = SGlobe::from_xyz(w.eye - w.center);

    if w.trans_type == 2 {
        cache
            .direction
            .set_latitude(SAngle::from_degrees(w.latitude));
    }

    let frames = camera.setup_age as f32;
    let spin = SAngle::from_degrees(frames * w.roll);
    let longitude = cache.direction.longitude() + spin;
    cache.direction.set_longitude(longitude);
    cache
        .direction
        .set_radius(cache.direction.radius() + frames * w.radius_add);
    cache.eye = cache.center + cache.direction.to_xyz();
    cache.fovy = w.fovy;
    if w.banked {
        cache.bank = SAngle::from_degrees(w.bank);
        camera.event_flags |= evflag::BANK;
    }

    let hold = w.found && (camera.setup_age as i32) < w.timer;
    camera.work.ev.rolling = w;
    camera.view_cache = cache;
    cam_follow::publish(camera);
    !hold
}

/// `dCamera_c::uniformTransEvCamera`, `uniformBrakeEvCamera` and `uniformAcceleEvCamera`, the one
/// body behind all three, `sibling d_ev_camera.cpp:920-930` entering
/// `sibling d_ev_camera.cpp:944-1408`.
///
/// Three things are left out, all on the first frame. The wolf and companion adjustment on the
/// four relation offsets, `sibling d_ev_camera.cpp:1022-1058`, has no actor to key on. The
/// `StartSlot` and `EndSlot` arms, `sibling d_ev_camera.cpp:984-1014`, read the two saved view
/// stacks, and this port's stacks are only written by `saveEvCamera` and by `eventCamera`'s own
/// first frame; so they run when the cut names a slot inside that range. And the key-list arm,
/// `sibling d_ev_camera.cpp:974-981`, needs the array behind `BSpPoints`, which the feed cannot
/// read; when the cut names one, the arm latches the count and then has no values to step
/// through, so it settles and declines. The three built-in curve tables and the plain linear ramp
/// both run.
fn trans(
    camera: &mut Camera,
    player: &Player,
    collision: Option<&Collision>,
    feed: &mut Feed<'_>,
    cut: &Event,
    mode: i32,
) -> bool {
    let facing = facing_of(player);
    let mut w = camera.work.ev.trans;
    let mut cache = camera.view_cache;
    let done;

    if camera.setup_age == 0 {
        let Some(timer) = feed.int("Timer") else {
            camera.work.ev.trans = w;
            return true;
        };
        w.timer = timer;
        w.curve = feed.int_or("BSpCurve", mode);
        w.banked = false;
        if w.curve != 0 {
            let Some(keys) = BSP_CURVE_KEYS.get(w.curve.max(0) as usize).copied() else {
                camera.work.ev.trans = w;
                return true;
            };
            w.spline = Spline::new(keys, timer + 1);
            w.spline.step();
        } else {
            w.points = feed.float_list("BSpPoints").unwrap_or(0);
            if w.points != 0 {
                w.spline = Spline::new(w.points as i32, timer + 1);
                w.spline.step();
                w.curve = -1;
            }
        }

        if let Some(slot) = feed.int("EndSlot") {
            if let Some(view) = cut.saved.get(slot.max(0) as usize) {
                w.center = view.center;
                w.eye = view.eye;
                w.fovy = view.fovy;
                w.bank = view.bank.degree();
                w.banked = view.bank != SAngle::ZERO;
            }
        } else {
            w.eye = feed.xyz_or("Eye", camera.bump.eye);
            w.center = feed.xyz_or("Center", camera.bump.center);
            w.fovy = feed.float_or("Fovy", camera.view_cache.fovy);
            w.bank = feed.float_or("Bank", cache.bank.degree());
            w.banked = feed.float("Bank").is_some();
        }

        if let Some(slot) = feed.int("StartSlot") {
            if let Some(view) = cut.saved.get(slot.max(0) as usize) {
                w.start_center = view.center;
                w.start_eye = view.eye;
                w.start_fovy = view.fovy;
                w.start_bank = view.bank.degree();
                w.banked = w.banked || view.bank != SAngle::ZERO;
            }
        } else {
            w.start_eye = feed.xyz_or("StartEye", camera.bump.eye);
            w.start_center = feed.xyz_or("StartCenter", camera.bump.center);
            w.start_fovy = feed.float_or("StartFovy", camera.view_cache.fovy);
            w.start_bank = feed.float_or("StartBank", cache.bank.degree());
            w.banked = w.banked || feed.float("StartBank").is_some();
        }

        w.trans_type = feed.int_or("TransType", 0);
        let relative = feed.actor("RelActor");
        w.relative = relative != EvActor::Missing;
        let mask = feed.string_or("RelUseMask", "--oo");
        let bytes = mask.as_bytes();
        for lane in 0..4 {
            w.mask[lane] = bytes.get(lane).copied().unwrap_or(b'x');
        }
        w.cushion = feed.float_or("Cushion", 1.0);

        if w.relative {
            first_frame_relational(camera, player, collision, &mut w, facing);
        } else if w.mask[2] == b'a' {
            w.center = cache.center
                + rotate_y(
                    w.center,
                    SAngle::from(cache.direction.longitude().inv().val()),
                );
        } else if w.mask[3] == b'a' {
            w.eye =
                cache.eye + rotate_y(w.eye, SAngle::from(cache.direction.longitude().inv().val()));
        }

        w.inverted = camera.bump.direction;
        w.inverted.invert();
        camera.settled = true;
    }

    if w.curve == -1 && w.points != 0 {
        camera.work.ev.trans = w;
        return false;
    }

    if camera.setup_age as i32 >= w.timer {
        done = true;
        blend(camera, &mut cache, &w, 1.0, player);
    } else {
        done = false;
        let ratio = if w.curve == 0 {
            (camera.setup_age as f32 + 1.0) / w.timer.max(1) as f32
        } else {
            w.spline.step();
            curve(&w, w.curve)
        };
        blend(camera, &mut cache, &w, ratio, player);
    }

    cache.direction = SGlobe::from_xyz(cache.eye - cache.center);
    camera.work.ev.trans = w;
    camera.view_cache = cache;
    cam_follow::publish(camera);
    done
}

/// `d2DBSplinePath::Calc` over the three built-in tables, `sibling d_ev_camera.cpp:955-957`.
fn curve(w: &Trans, index: i32) -> f32 {
    let table: &[f32] = match index {
        1 => &BSP_CURVE_A,
        2 => &BSP_CURVE_B,
        _ => &BSP_CURVE_C,
    };
    w.spline.calc(table).unwrap_or(0.0)
}

/// The first frame's relation rewrite, `sibling d_ev_camera.cpp:1021-1218`.
fn first_frame_relational(
    camera: &Camera,
    player: &Player,
    collision: Option<&Collision>,
    w: &mut Trans,
    facing: SAngle,
) {
    let behind = SGlobe::from_xyz(camera.bump.eye - player.pos).longitude() - facing < SAngle::ZERO;

    if w.mask[1] == b'r' {
        if camera.age & 1 != 0 {
            w.start_eye.x = -w.start_eye.x;
        }
        let center = relational(player, facing, w.start_center);
        let eye = relational(player, facing, w.start_eye);
        if blocked(collision, center, eye) {
            w.start_eye.x = -w.start_eye.x;
        }
    }

    if w.mask[0] == b'n' || w.mask[1] == b'n' {
        if behind {
            if w.mask[0] == b'n' {
                w.start_center.x = -w.start_center.x;
            }
            if w.mask[1] == b'n' {
                w.start_eye.x = -w.start_eye.x;
            }
        }
        let center = relational(player, facing, w.start_center);
        let eye = relational(player, facing, w.start_eye);
        if blocked(collision, center, eye) {
            w.start_eye.x = -w.start_eye.x;
        }
    }

    if w.mask[2] == b'n' || w.mask[3] == b'n' {
        if behind {
            if w.mask[2] == b'n' {
                w.center.x = -w.center.x;
            }
            if w.mask[3] == b'n' {
                w.eye.x = -w.eye.x;
            }
        }
        let center = relational(player, facing, w.center);
        let eye = relational(player, facing, w.eye);
        if blocked(collision, center, eye) {
            w.eye.x = -w.eye.x;
        }
    }

    if w.mask[0] == b'N' || w.mask[1] == b'N' {
        if !behind {
            if w.mask[0] == b'N' {
                w.start_center.x = -w.start_center.x;
            }
            if w.mask[1] == b'N' {
                w.start_eye.x = -w.start_eye.x;
            }
        }
        let center = relational(player, facing, w.start_center);
        let eye = relational(player, facing, w.start_eye);
        if blocked(collision, center, eye) {
            w.start_eye.x = -w.start_eye.x;
        }
    }

    if w.mask[2] == b'N' || w.mask[3] == b'N' {
        if !behind {
            if w.mask[2] == b'N' {
                w.center.x = -w.center.x;
            }
            if w.mask[3] == b'N' {
                w.eye.x = -w.eye.x;
            }
        }
        let center = relational(player, facing, w.center);
        let eye = relational(player, facing, w.eye);
        if blocked(collision, center, eye) {
            w.eye.x = -w.eye.x;
        }
    }

    for lane in 0..2 {
        if w.mask[lane] == b'f' {
            let offset = if lane == 0 {
                w.start_center
            } else {
                w.start_eye
            };
            let mut globe = SGlobe::from_xyz(offset);
            globe.set_longitude(facing + globe.longitude());
            let turned = player.pos + globe.to_xyz();
            if lane == 0 {
                w.start_center = turned;
            } else {
                w.start_eye = turned;
            }
            w.mask[lane] = b'x';
        }
    }

    for lane in 2..4 {
        match w.mask[lane] {
            b'p' => {
                let offset = if lane == 2 { w.center } else { w.eye };
                let near = relational(player, facing, offset);
                let mut turned = offset;
                turned.x = -turned.x;
                let far = relational(player, facing, turned);
                if (near - player.pos).length() < (far - player.pos).length() {
                    turned.x = -turned.x;
                    let picked = relational(player, facing, turned);
                    if lane == 2 {
                        w.center = picked;
                    } else {
                        w.eye = picked;
                    }
                }
            }
            b'r' if lane == 3 => {
                let center = relational(player, facing, w.center);
                if camera.age & 1 != 0 {
                    w.eye.x = -w.eye.x;
                }
                let eye = relational(player, facing, w.eye);
                if blocked(collision, center, eye) {
                    w.eye.x = -w.eye.x;
                }
            }
            b'f' => {
                let offset = if lane == 2 { w.center } else { w.eye };
                let mut globe = SGlobe::from_xyz(offset);
                globe.set_longitude(facing + globe.longitude());
                let turned = player.pos + globe.to_xyz();
                if lane == 2 {
                    w.center = turned;
                } else {
                    w.eye = turned;
                }
                w.mask[lane] = b'x';
            }
            _ => {}
        }
    }
}

/// The four blend arms, `sibling d_ev_camera.cpp:1317-1402`, and the field-of-view and bank pair
/// that follows them.
///
/// Three shapes and no more. The globe arms, `sibling d_ev_camera.cpp:1317-1380`, build the eye off
/// the *start* pair's own globe, not off the centre the blend just wrote, which is what
/// `:1321-1328`, `:1336-1354` and `:1362-1376` do; the transition type two arm, `:1382-1387`, adds
/// the end offsets to the start ones instead of interpolating between them, because on a relative
/// actor the per-frame relation arms above have already turned them into world positions; and the
/// arm at `:1388-1393`, which catches every other type number, interpolates the two world positions
/// and never builds a globe at all.
fn blend(camera: &mut Camera, cache: &mut ViewCache, w: &Trans, ratio: f32, player: &Player) {
    let facing = facing_of(player);
    let (start_center, start_eye, end_center, end_eye) = if w.relative {
        (
            per_frame_relation(player, w, facing, 0, w.start_center),
            per_frame_relation(player, w, facing, 1, w.start_eye),
            per_frame_relation(player, w, facing, 2, w.center),
            per_frame_relation(player, w, facing, 3, w.eye),
        )
    } else {
        (w.start_center, w.start_eye, w.center, w.eye)
    };

    let center = match w.trans_type {
        2 => start_center + end_center * ratio,
        _ => start_center + (end_center - start_center) * ratio,
    };
    cache.center += (center - cache.center) * w.cushion;

    match w.trans_type {
        2 => cache.eye += (start_eye + end_eye * ratio - cache.eye) * w.cushion,
        1 | 3 | 4 => {
            let from = SGlobe::from_xyz(start_eye - start_center);
            let to = SGlobe::from_xyz(end_eye - end_center);
            let longitude = if w.trans_type == 1 {
                from.longitude() + (to.longitude() - from.longitude()) * ratio
            } else {
                from.longitude() + wrap_longitude(from, to, ratio, w.trans_type == 4)
            };
            let globe = SGlobe::new(
                from.radius() + (to.radius() - from.radius()) * ratio,
                from.latitude() + (to.latitude() - from.latitude()) * ratio,
                longitude,
            );
            cache.eye += (cache.center + globe.to_xyz() - cache.eye) * w.cushion;
        }
        _ => cache.eye += (start_eye + (end_eye - start_eye) * ratio - cache.eye) * w.cushion,
    }

    let fovy = w.start_fovy + ratio * (w.fovy - w.start_fovy);
    cache.fovy += w.cushion * (fovy - cache.fovy);
    if w.banked {
        let bank = w.start_bank + ratio * (w.bank - w.start_bank);
        cache.bank += (SAngle::from_degrees(bank) - cache.bank) * w.cushion;
        camera.event_flags |= evflag::BANK;
    }
}

/// The longitude blend of the accelerating and decelerating arms,
/// `sibling d_ev_camera.cpp:1339-1348` and `:1365-1372`: a whole turn when the two longitudes
/// agree, then the short way forward for the acceleration and the short way backward for the
/// brake. The plain uniform transition, `:1321-1328`, is not one of these two and has no such
/// rule.
fn wrap_longitude(from: SGlobe, to: SGlobe, ratio: f32, backward: bool) -> SAngle {
    let span = to.longitude() - from.longitude();
    if backward {
        if span == SAngle::ZERO {
            SAngle::from_degrees(ratio * -360.0)
        } else if span < SAngle::ZERO {
            span * ratio
        } else {
            SAngle::from_degrees(ratio * (span.degree() - 360.0))
        }
    } else if span == SAngle::ZERO {
        SAngle::from_degrees(ratio * 360.0)
    } else if span > SAngle::ZERO {
        span * ratio
    } else {
        SAngle::from_degrees(ratio * (span.degree() + 360.0))
    }
}

/// The per-frame relation arms, `sibling d_ev_camera.cpp:1242-1313`: the attention point for
/// `'t'`, the camera's own turned frame for `'c'`, the plain relation for anything the mask asks
/// a relation of, the actor's own yaw for a transition type two offset it does not, and the
/// offset alone for a dash or an `x`.
fn per_frame_relation(
    player: &Player,
    w: &Trans,
    facing: SAngle,
    lane: usize,
    offset: Vec3,
) -> Vec3 {
    match w.mask[lane] {
        b't' => player.pos + offset,
        b'c' => {
            let mut globe = SGlobe::from_xyz(offset);
            globe.set_longitude(w.inverted.longitude() + globe.longitude());
            player.pos + globe.to_xyz()
        }
        letter if is_rel_char(letter) => relational(player, facing, offset),
        _ if w.trans_type == 2 && lane >= 2 => rotate_y(offset, facing),
        _ => offset,
    }
}

/// `dCamera_c::restorePosEvCamera`, `sibling d_ev_camera.cpp:1713-1822`.
///
/// One thing is left out: the sibling's third destination, its own `mSavedView`, filled by the
/// `SavePos` parameter `Save` names as two, `sibling d_ev_camera.cpp:1743-1746` and
/// `sibling d_camera.cpp:10186-10189`. The target's `eventCamera` has no `SavePos` and no
/// `LoadPos` at all, `d_camera.cpp:4801-4840`, and its header carries only the two saved views,
/// `d_camera.h:177`, so there is no third slot to read. Slot nine, the save file through
/// `dComIfGp_loadCameraPosition`, has no reader here either. A destination this port cannot
/// produce leaves the arm at its own zero arm, which holds the camera where it is and accepts,
/// `sibling d_ev_camera.cpp:1777-1779`.
fn restore_pos(
    camera: &mut Camera,
    player: &Player,
    collision: Option<&Collision>,
    feed: &mut Feed<'_>,
    cut: &Event,
) -> bool {
    let facing = facing_of(player);
    let mut w = camera.work.ev.restore_pos;
    let mut cache = camera.view_cache;

    if camera.setup_age == 0 {
        w.ctr_gap = feed.xyz_or("CtrGap", Vec3::ZERO);
        w.cushion = feed.float_or("Cushion", 1.0);
        w.near_dist = feed.float_or("NearDist", 750.0);
        w.far_dist = feed.float_or("FarDist", 1500.0);
        w.near_timer = feed.int_or("NearTimer", 20);
        w.far_timer = feed.int_or("FarTimer", 30);
        let dest = feed.int_or("Dest", 2);
        w.target_type = feed.int_or("TargetType", 0);
        w.dest = match dest {
            0 | 1 => cut.saved.get(dest.max(0) as usize).copied(),
            _ => None,
        };
        if feed.actor_or("Target", "@PLAYER") == EvActor::Missing {
            camera.work.ev.restore_pos = w;
            return true;
        }
        w.target_center = relational(player, facing, w.ctr_gap);

        let Some(dest) = w.dest else {
            w.arm = 0;
            camera.settled = true;
            camera.work.ev.restore_pos = w;
            return true;
        };

        let span = SGlobe::from_xyz(dest.eye - cache.center);
        w.arm = if span.radius() < w.near_dist {
            if in_sight(camera, w.target_center) {
                0
            } else {
                1
            }
        } else if span.radius() < w.far_dist {
            if blocked(collision, camera.bump.eye, w.target_center) {
                3
            } else {
                2
            }
        } else {
            3
        };
        camera.settled = true;
    }

    match w.arm {
        0 => {}
        1 | 2 => {
            if camera.setup_age == 0 {
                let dest = w.dest.unwrap_or_default();
                w.direction = SGlobe::from_xyz(dest.eye - dest.center);
            }
            if (camera.setup_age as i32) < w.far_timer {
                let dest = w.dest.unwrap_or_default();
                let span = camera.setup_age as f32 / w.far_timer as f32;
                let walk = if w.target_type == 1 {
                    dest.center
                } else {
                    w.target_center
                };
                cache.center += (walk - cache.center) * span;
                let radius = cache.direction.radius()
                    + span * (w.direction.radius() - cache.direction.radius());
                cache.direction.set_radius(radius);
                let longitude = cache.direction.longitude()
                    + (w.direction.longitude() - cache.direction.longitude()) * span;
                cache.direction.set_longitude(longitude);
                let latitude = cache.direction.latitude()
                    + (w.direction.latitude() - cache.direction.latitude()) * span;
                cache.direction.set_latitude(latitude);
                cache.eye = cache.center + cache.direction.to_xyz();
                cache.fovy += span * (dest.fovy - cache.fovy);
                camera.work.ev.restore_pos = w;
                camera.view_cache = cache;
                cam_follow::publish(camera);
                return false;
            }
        }
        _ => {
            if camera.setup_age == 0 {
                let dest = w.dest.unwrap_or_default();
                cache.center = if w.target_type == 1 {
                    dest.center
                } else {
                    w.target_center
                };
                cache.direction = SGlobe::from_xyz(dest.eye - dest.center);
                cache.eye = cache.center + cache.direction.to_xyz();
                cache.fovy = dest.fovy;
            }
        }
    }

    camera.settled = true;
    camera.work.ev.restore_pos = w;
    camera.view_cache = cache;
    cam_follow::publish(camera);
    true
}

/// `dCamera_c::loadEvCamera`, `sibling d_ev_camera.cpp:2624-2687`.
///
/// One thing is left out: slot nine, the save file through `dComIfGp_loadCameraPosition`, for the
/// same reason as in [`restore_pos`]. The built-in four-key curve runs, so both `mType` values do;
/// the curve type one and two both use it, and a type above one is clamped to one first,
/// `sibling d_ev_camera.cpp:2632-2635`.
fn load(camera: &mut Camera, feed: &mut Feed<'_>, cut: &Event) -> bool {
    let mut w = camera.work.ev.load;
    let mut cache = camera.view_cache;

    if camera.setup_age == 0 {
        w.slot = feed.int_or("Slot", 0);
        w.timer = feed.int_or("Timer", 1);
        w.curve_type = feed.int_or("Type", if w.timer < 10 { 0 } else { 1 });
        if w.curve_type > 1 {
            w.curve_type = 1;
        }
        if w.curve_type != 0 {
            w.spline = Spline::new(LOAD_CURVE_KEYS, w.timer + 1);
            w.spline.step();
        }
        let dest = cut
            .saved
            .get(w.slot.max(0) as usize)
            .copied()
            .unwrap_or_default();
        w.dest_center = dest.center;
        w.dest_eye = dest.eye;
        w.dest_fovy = dest.fovy;
        w.dest_bank = dest.bank;
        w.start_center = cache.center;
        w.start = cache.direction;
        w.want = SGlobe::from_xyz(w.dest_eye - w.dest_center);
        w.start_fovy = cache.fovy;
        w.start_bank = cache.bank;
    }

    if (camera.setup_age as i32) < w.timer {
        let ratio = if w.curve_type != 0 {
            w.spline.step();
            w.spline.calc(&LOAD_CURVE).unwrap_or(0.0)
        } else {
            (camera.setup_age as f32 + 1.0) / w.timer.max(1) as f32
        };
        cache.center = w.start_center + (w.dest_center - w.start_center) * ratio;
        cache
            .direction
            .set_radius(w.start.radius() + ratio * (w.want.radius() - w.start.radius()));
        cache
            .direction
            .set_latitude(w.start.latitude() + (w.want.latitude() - w.start.latitude()) * ratio);
        cache.direction.set_longitude(
            w.start.longitude() + (w.want.longitude() - w.start.longitude()) * ratio,
        );
        cache.eye = cache.center + cache.direction.to_xyz();
        cache.fovy = w.start_fovy + ratio * (w.dest_fovy - w.start_fovy);
        cache.bank = w.start_bank + (w.dest_bank - w.start_bank) * ratio;
        camera.work.ev.load = w;
        camera.view_cache = cache;
        cam_follow::publish(camera);
        return false;
    }

    camera.settled = true;
    camera.work.ev.load = w;
    true
}

/// `dCamera_c::getItemEvCamera`, `sibling d_ev_camera.cpp:3150-3263`.
///
/// Half of it runs. The five rows of offsets and their field of view, frame count and side flag
/// are ported, as is the whole walk; what is left out is the cut's own `CurvePoints` key list,
/// `sibling d_ev_camera.cpp:3196-3199`, which the feed can count but not read, so a cut that
/// names one has no keys to step through and the arm settles and declines. A cut that names none
/// runs on the built-in six-key table.
///
/// The two divergences from the sibling are its wolf tests: the `Type` default is
/// `mIsWolf != 1 ? 0 : 1`, `sibling d_ev_camera.cpp:3183`, and a `Type` of two becomes three for
/// a hero and four for a wolf, `sibling d_ev_camera.cpp:3185-3191`. The target has neither form,
/// so the default is zero and two becomes three.
///
/// One divergence from the sibling is the port's: the sibling's side test negates the eye offset
/// *inside its own static table*, `sibling d_ev_camera.cpp:3214` and `:3219`, so a cut that once
/// took the far side leaves every later cut on that side too. This port negates a local copy, so
/// each cut starts from the table. The written-back position is never read again in either game,
/// so nothing else turns on it.
fn get_item(
    camera: &mut Camera,
    player: &Player,
    collision: Option<&Collision>,
    feed: &mut Feed<'_>,
) -> bool {
    let facing = facing_of(player);
    let mut w = camera.work.ev.get_item;
    let mut cache = camera.view_cache;
    let mut own_curve = true;

    if camera.setup_age == 0 {
        w.item_type = feed.int_or("Type", 0);
        if w.item_type == 2 {
            w.item_type = 3;
        }
        let Some(row) = ITEM_ROWS.get(w.item_type.max(0) as usize).copied() else {
            camera.work.ev.get_item = w;
            return true;
        };
        w.timer = feed.int_or("Timer", row.3);
        match feed.float_list("CurvePoints") {
            Some(count) => {
                own_curve = false;
                w.keys = count as i32;
                w.spline = Spline::new(w.keys, w.timer + 1);
                w.spline.step();
            }
            None => {
                w.keys = ITEM_CURVE_KEYS;
                w.spline = Spline::new(ITEM_CURVE_KEYS, w.timer + 1);
                w.spline.step();
            }
        }
        w.start_center = feed.xyz_or("StartCenter", camera.bump.center);
        w.start_eye = feed.xyz_or("StartEye", camera.bump.eye);
        w.relative = feed.actor_or("RelActor", "@PLAYER") == EvActor::Player;

        let ctr_gap = feed.xyz_or("Center", ITEM_OFFSETS[row.0]);
        let mut eye_gap = ITEM_OFFSETS[row.1];
        let side = SGlobe::from_xyz(camera.bump.eye - player.pos).longitude() - facing;
        let flip = (row.4 > 0 && side < SAngle::ZERO) || (row.4 < 0 && side > SAngle::ZERO);
        if w.relative && row.4 != 0 && flip {
            eye_gap.x = -eye_gap.x;
        }

        w.dest_center = if w.relative {
            relational(player, facing, ctr_gap)
        } else {
            ctr_gap
        };

        let wanted = feed.xyz_or("Eye", eye_gap);
        if w.relative && row.4 != 0 {
            let turned = relational(player, facing, wanted);
            w.dest_eye = if blocked(collision, w.dest_center, turned) {
                let mut flipped = wanted;
                flipped.x = -flipped.x;
                relational(player, facing, flipped)
            } else {
                turned
            };
        } else if w.relative {
            w.dest_eye = relational(player, facing, wanted);
        } else {
            w.dest_eye = wanted;
        }

        w.start = SGlobe::from_xyz(w.start_eye - w.start_center);
        w.want = SGlobe::from_xyz(w.dest_eye - w.dest_center);
    }

    if !own_curve {
        camera.work.ev.get_item = w;
        return false;
    }

    if (camera.setup_age as i32) < w.timer {
        w.spline.step();
        let ratio = w.spline.calc(&ITEM_CURVE).unwrap_or(0.0);
        cache.center = w.start_center + (w.dest_center - w.start_center) * ratio;
        cache
            .direction
            .set_radius(w.start.radius() + ratio * (w.want.radius() - w.start.radius()));
        cache
            .direction
            .set_latitude(w.start.latitude() + (w.want.latitude() - w.start.latitude()) * ratio);
        cache.direction.set_longitude(
            w.start.longitude() + (w.want.longitude() - w.start.longitude()) * ratio,
        );
        cache.eye = cache.center + cache.direction.to_xyz();
        let row = ITEM_ROWS[w.item_type.max(0) as usize];
        cache.fovy += ratio * (row.2 - cache.fovy);
        camera.work.ev.get_item = w;
        camera.view_cache = cache;
        cam_follow::publish(camera);
        return false;
    }

    true
}

/// `dCamera_c::possessedEvCamera`, `sibling d_ev_camera.cpp:3275-3371`.
///
/// The whole body runs except the two blur arms, `sibling d_ev_camera.cpp:3319-3326` and
/// `:3341-3355`: `StartBlure`, `SetBlurePosition`, `SetBlureAlpha` and `SetBlureScale` have no
/// port, and the position needs the framed window's scissor, which the port's viewport is.
///
/// The type-zero arm's own save into the second view stack, `sibling d_ev_camera.cpp:3307-3308`,
/// runs; the stack write needs `pushInfo`, which the target declares as taking a count the port
/// has no use for, so the view is copied into [`Event::saved`] directly.
fn possessed(camera: &mut Camera, player: &Player, feed: &mut Feed<'_>, cut: &mut Event) -> bool {
    let facing = facing_of(player);
    let mut w = camera.work.ev.possessed;
    let mut cache = camera.view_cache;
    let mut done = false;

    if camera.setup_age == 0 {
        w.arm = 0;
    }

    match w.arm {
        1 => {
            let share = if w.left != 0 {
                1.0 / w.left as f32
            } else {
                1.0
            };
            cache.center += (player.pos - cache.center) * share;
            cache.direction.set_radius(
                cache.direction.radius() + share * (w.want.radius() - cache.direction.radius()),
            );
            cache.direction.set_latitude(
                cache.direction.latitude()
                    + (w.want.latitude() - cache.direction.latitude()) * share,
            );
            cache.direction.set_longitude(
                cache.direction.longitude()
                    + (w.want.longitude() - cache.direction.longitude()) * share,
            );
            let eye = cache.center + cache.direction.to_xyz();
            cache.eye += (eye - cache.eye) * w.cushion;
            cache.fovy += share * (w.fovy - cache.fovy);
            w.left -= 1;
            if w.left <= 0 {
                w.arm = 99;
            }
        }
        99 => {
            done = true;
            camera.settled = true;
        }
        _ => {
            if feed.actor_or("Target", "@PLAYER") == EvActor::Missing {
                camera.work.ev.possessed = w;
                return true;
            }
            w.item_type = feed.int_or("Type", 0);
            w.timer = feed.int_or("Timer", 10);
            w.radius = feed.float_or("Radius", 60.0);
            w.cushion = feed.float_or("Cushion", 1.0);
            w.latitude = SAngle::from_degrees(feed.float_or("Latitude", -5.0));
            w.longitude = SAngle::from_degrees(feed.float_or("Longitude", 0.0));
            w.fovy = feed.float_or("Fovy", 45.0);
            w.blure = feed.int_or("Blure", 0);
            let aim = SGlobe::new(w.radius, w.latitude, w.longitude + facing);

            if w.item_type == 0 {
                w.want = aim;
                cut.saved[1] = EventView {
                    center: cache.center,
                    eye: cache.eye,
                    fovy: cache.fovy,
                    bank: cache.bank,
                    age: 2,
                };
            } else {
                let saved = cut.saved[1];
                w.want = SGlobe::from_xyz(saved.eye - saved.center);
                cache.center = player.pos;
                cache.direction = aim;
                cache.eye = cache.center + cache.direction.to_xyz();
                cache.fovy = w.fovy;
            }

            w.arm = 1;
            w.left = w.timer;
        }
    }

    camera.work.ev.possessed = w;
    camera.view_cache = cache;
    cam_follow::publish(camera);
    done
}

/// `dCamera_c::fixedFramesEvCamera`, `sibling d_ev_camera.cpp:3373-3477`.
///
/// The arm is stopped at its first frame. All three of its parameters are key lists, `Centers`,
/// `Eyes` and `Fovys`, `sibling d_ev_camera.cpp:3380-3412`, and the feed can count them but not
/// read them, so the frame count, the chosen frame, the chosen centre and eye and the chosen
/// field of view have no values and the arm has nothing to hold. The arm would also want the
/// `ChkCamera` actor-collision test, `sibling d_ev_camera.cpp:3453-3454`, which is a separate
/// collision query from the one this port has.
fn fixed_frames(camera: &mut Camera, feed: &mut Feed<'_>) -> bool {
    let mut w = camera.work.ev.fixed_frames;

    if camera.setup_age == 0 {
        let centers = feed.xyz_list("Centers");
        let eyes = feed.xyz_list("Eyes");
        let fovys = feed.float_list("Fovys");
        if centers.is_none() || eyes.is_none() || fovys.is_none() {
            return true;
        }
        w.timer = feed.int_or("Timer", 1);
        w.found = feed.int("Timer").is_some();
        let mask = feed.string_or("RelUseMask", "oo");
        let bytes = mask.as_bytes();
        w.mask[0] = bytes.first().copied().unwrap_or(b'x');
        w.mask[1] = bytes.get(1).copied().unwrap_or(b'x');
        w.relative = feed.actor("RelActor") != EvActor::Missing;
        w.choice = feed.int_or("Choice", 0);
        w.frames = centers
            .unwrap_or(0)
            .min(eyes.unwrap_or(0))
            .min(fovys.unwrap_or(0)) as i32;
        camera.settled = true;
    }

    camera.work.ev.fixed_frames = w;
    false
}

/// `dCamera_c::bSplineEvCamera`, `sibling d_ev_camera.cpp:3479-3544`.
///
/// The arm is stopped at its first frame for the same reason as [`fixed_frames`]: all three of
/// its parameters are key lists, `sibling d_ev_camera.cpp:3484-3512`, and the curve it steps
/// reads them one value at a time.
fn b_spline(camera: &mut Camera, feed: &mut Feed<'_>) -> bool {
    let mut w = camera.work.ev.b_spline;

    if camera.setup_age == 0 {
        let centers = feed.xyz_list("Centers");
        let eyes = feed.xyz_list("Eyes");
        let fovys = feed.float_list("Fovys");
        if centers.is_none() || eyes.is_none() || fovys.is_none() {
            return true;
        }
        w.frames = centers
            .unwrap_or(0)
            .min(eyes.unwrap_or(0))
            .min(fovys.unwrap_or(0)) as i32;
        let Some(timer) = feed.int("Timer") else {
            return true;
        };
        w.timer = timer;
        w.relative = feed.actor("RelActor") != EvActor::Missing;
        w.spline = Spline::new(w.frames as i32, w.timer);
        camera.settled = true;
    }

    camera.work.ev.b_spline = w;
    false
}

/// `dCamera_c::twoActor0EvCamera`, `sibling d_ev_camera.cpp:3546-3662`.
///
/// The arm is stopped at its first frame by its own guard: the second actor defaults to
/// `'@STARTER'` and the port cannot resolve any name but the player's, so the sibling's
/// "actor 1 or 2 missing" return, `sibling d_ev_camera.cpp:3554-3557`, is what runs, and it keeps
/// running on the later frames where the sibling would dereference the two null actors. With both
/// actors the body would also want each one's attention point, the player's own facing for the
/// `LongitudeType` arms, and the two actors' process ids for the liveness test.
fn two_actor(camera: &mut Camera, feed: &mut Feed<'_>) -> bool {
    let mut w = camera.work.ev.two_actor;

    if camera.setup_age == 0 {
        w.actor1 = feed.actor_or("Actor1", "@PLAYER") != EvActor::Missing;
        w.actor2 = feed.actor_or("Actor2", "@STARTER") != EvActor::Missing;
        w.ctr_gap = feed.xyz_or("CtrGap", Vec3::ZERO);
        w.ctr_ratio = feed.float_or("CtrRatio", -0.1);
        w.ctr_cus = feed.float_or("CtrCus", 1.0);
        w.eye_cus = feed.float_or("EyeCus", 1.0);
        w.radius_min = feed.float_or("RadiusMin", 100.0);
        w.radius_max = feed.float_or("RadiusMax", 10000.0);
        w.latitude_min = SAngle::from_degrees(feed.float_or("LatitudeMin", -60.0));
        w.latitude_max = SAngle::from_degrees(feed.float_or("LatitudeMax", 60.0));
        w.longitude_min = SAngle::from_degrees(feed.float_or("LongitudeMin", 20.0));
        w.longitude_max = SAngle::from_degrees(feed.float_or("LongitudeMax", 160.0));
        w.longitude_type = feed.int_or("LongitudeType", -1);
        w.fovy = feed.float_or("Fovy", 60.0);
    }

    if !w.actor1 || !w.actor2 {
        camera.work.ev.two_actor = w;
        return true;
    }

    camera.work.ev.two_actor = w;
    true
}
