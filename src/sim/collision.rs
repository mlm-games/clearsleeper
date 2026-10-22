use bevy_ecs::prelude::*;
use glam::{Vec2, Vec3};

use super::ground_code::{
    AttrCode, FrontWall, GroundCode, PolyCodes, SpecialCode, WallCode, attribute_from_direct,
};
use super::player::Player;
use super::walk::{from_s16, to_s16};

const HEADER_LEN: usize = 0x34;
const VTX_STRIDE: usize = 12;
const TRI_STRIDE: usize = 10;
const BLOCK_STRIDE: usize = 2;
const TREE_STRIDE: usize = 0x14;
const GROUP_STRIDE: usize = 0x34;
const INFO_STRIDE: usize = 16;
const GROUND_NY: f32 = 0.5;
/// `cBgW::RwgGroundCheckWall`, `c_bg_w.cpp:505`: the wall list's own floor. `cBgS::GroundCross`
/// (`c_bg_s.cpp:130-144`) reads the ground list and the wall list of every block
/// (`c_bg_w.cpp:519-522`), and the wall list keeps the faces this level or steeper, so a slope
/// past 60 degrees still carries the player.
const GROUND_WALL_NY: f32 = 0.014;
const ROOF_NY: f32 = -0.8;
const ACTOR_HEIGHT: f32 = 60.0;
const PLANE_MIN_MAGNITUDE: f32 = 0.02;
const EXIT_MASK: u32 = 0x0007E000;
const EXIT_SHIFT: u32 = 13;
const WALL_MASK: u32 = 0x00000F00;
const WALL_SHIFT: u32 = 8;
const SPECIAL_MASK: u32 = 0x0000F000;
const SPECIAL_SHIFT: u32 = 12;
const ATTRIBUTE_MASK: u32 = 0x001F0000;
const ATTRIBUTE_SHIFT: u32 = 16;
const GROUND_MASK: u32 = 0x03E00000;
const GROUND_SHIFT: u32 = 21;
const DET_EPSILON: f32 = 1.0e-6;
/// `G_CM3D_F_ABS_MIN`, `c_m3d.cpp:22`: the reference's own "is zero" tolerance, which
/// `cM3d_IsZero` at `c_m3d.h:135` compares against.
const ABS_MIN: f32 = 3.8146973e-06;
/// `daPy_lk_c::setBgCheckParam`, `d_a_player_main.cpp:10708`: the standing player's wall
/// radius, which `mAcchCir[0].GetWallR` hands the wall correction.
pub const WALL_RADIUS: f32 = 35.0;
/// `daPy_lk_c::setBgCheckParam`, `d_a_player_main.cpp:10673-10675`: the three wall circle
/// heights of the standing player.
pub const WALL_HEIGHTS: [f32; 3] = [30.1, 89.9, 125.0];
/// `setFrontWallType` at `main.cpp:4610`: how far below its ledge the player has to be before the
/// ledge counts at all.
const LEDGE_DROP: f32 = 125.0;
/// `main.cpp:4758`: the top of the band below which there is no ledge to catch.
const LEDGE_MIN: f32 = 27.09;
/// `main.cpp:4563`: the near reach of the first trace, before the player's own wall radius.
const WALL_REACH: f32 = 25.0;
/// `main.cpp:4698`: the far reach of the second, catch-height trace.
const CATCH_REACH: f32 = 50.0;
/// `main.cpp:4573` and `whide.inc:86`: a traced face has to be this level to be a wall.
const WALL_PLANE_FLAT: f32 = 0.05;
/// `main.cpp:4577`: how far the wall's own facing may sit off the way the player faces.
const WALL_YAW_LIMIT: u16 = 0x2000;
/// `main.cpp:4599`: how far two normals may differ and still be one surface.
const PARALLEL_LIMIT: f32 = 0.001;
/// `main.cpp:4624`: how far back off the ledge point the vine-catch ground probe starts.
const VINE_PROBE_BACK: f32 = 15.0;
/// `main.cpp:4638`: how far above the player that probe's corner may sit and still be a catch.
const VINE_PROBE_UP: f32 = 150.0;
/// `main.cpp:4703`: the roof height that stands in for a catch-height trace that missed.
const ROOF_CATCH_MARGIN: f32 = 10.0;
/// `main.cpp:4711`: how far above the player the hidden-wall trace runs.
const HIDE_TRACE_UP: f32 = 149.9;
/// `main.cpp:4749`: how far below its ledge a swim catch reaches.
const SWIM_CATCH_DROP: f32 = 5.0;
/// `main.cpp:4752`: the drop from the ground the player left that a midair catch needs, and the
/// ledge height past which a hookshot in flight will not take one.
const MIDAIR_CATCH_DROP: f32 = 125.0;
const MIDAIR_CATCH_LIFT: f32 = 100.0;
/// `crawl.inc:177`: the normal a shallow special-`Steep` face is still refused at.
const CRAWL_STEEP: f32 = 0.643832;
/// `whide.inc:106` and `whide.inc:108`: how far a hidden wall's flank trace runs with no side
/// asked for, and with one.
const HIDE_FLANK_NONE: f32 = 49.9;
const HIDE_FLANK_SIDE: f32 = 99.8;
/// `whide.inc:113`: how far past the wall's own normal a flank trace starts, and
/// `whide.inc:117` how long it is.
const HIDE_FLANK_OUT: f32 = 1.25;
const HIDE_FLANK_BACK: f32 = 2.5;
/// `whide.inc:158`: the widest turn either flank may sit off the player's own facing.
const HIDE_TURN_LIMIT: i32 = 0x2AAA;
/// `whide.inc:161`: the widest turn a hidden wall may still answer to.
const HIDE_YAW_LIMIT: u32 = 0x1555;
pub const NO_EXIT: u8 = 0x3F;
/// `dBgS::GetGrpRoomInfId`, `d_bg_s.cpp:120-136`: the room read id the reference answers with
/// when the polygon carries no group, which `dStage_RoomCheck` at `d_stage.cpp:2278-2280`
/// reads as "no room set".
pub const NO_ROOM_READ: u8 = 0xFF;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DzbError {
    Truncated,
    BadTable,
    VertexIndex,
    TooLarge,
}

impl DzbError {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Truncated => "dzb header truncated",
            Self::BadTable => "dzb table out of range",
            Self::VertexIndex => "dzb triangle vertex out of range",
            Self::TooLarge => "dzb merged vertex table exceeds u16 indexing",
        }
    }
}

/// `cM3dGPla`, `c_m3d_g_pla.h`: a unit normal and the plane offset, so a point sits
/// `normal . point + d` off the surface. `dBgS::GetTriPla`, `bg_s.h:117`, hands out one of
/// these per triangle and most of the queries below are read off it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Plane {
    pub normal: Vec3,
    pub d: f32,
}

#[derive(Resource, Debug, Clone)]
pub struct Collision {
    verts: Vec<Vec3>,
    tris: Vec<[u16; 3]>,
    planes: Vec<Plane>,
    codes: Vec<PolyCodes>,
    /// `cBgD_Tri_t::m_grp` (`c_bg_w.h:57`) read through `cBgW::GetGrpInf`
    /// (`c_bg_w.h:179`) and `dBgS::GetGrpRoomInfId` (`d_bg_s.cpp:120-136`): the room read id
    /// each polygon carries, `0xFF` for a polygon whose group names none.
    room_read: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct StageData {
    pub spawn: Vec3,
    pub yaw: f32,
    pub collision: Collision,
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct MoveFrom(pub Vec3);

/// `dBgS::GroundCross`, `bg_s.h:76`, answered for one column: the ground height and the triangle
/// that carried it, which is what every code accessor in [`ground_code`] is read off.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GroundHit {
    pub height: f32,
    pub normal: Vec3,
    /// `GetTriPla`, `bg_s.h:117`: the plane offset, `normal . point`.
    pub plane_d: f32,
    pub tri: usize,
}

/// `dBgS::LineCross` and `LineCrossNonMoveBG`, `bg_s.cpp:309`: the segment `from` to `to` against
/// every triangle. `cBgW::RwgLineCheck` at `c_bg_w.cpp:386` writes the cross point back onto the
/// line's end, so each further triangle is only offered what is still ahead of it and the hit
/// that survives is the nearest one.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineHit {
    /// `cBgS_LinChk::GetCross`, `c_bg_s_lin_chk.h:44`.
    pub point: Vec3,
    /// `GetTriPla`, `bg_s.h:117`.
    pub normal: Vec3,
    /// `GetTriPla`, `bg_s.h:117`.
    pub plane_d: f32,
    /// The triangle the cross landed on, which every code accessor in [`ground_code`] reads.
    pub tri: usize,
}

/// `dBgS::SphChk`, `bg_s.cpp:386`, through `cM3d_Cross_SphTri` (`c_m3d_g_sph.h:47`): the nearest
/// triangle whose closest point to the centre lies within the radius.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SphereHit {
    /// The point of the triangle closest to the centre.
    pub point: Vec3,
    /// `GetTriPla`, `bg_s.h:117`.
    pub normal: Vec3,
    /// `GetTriPla`, `bg_s.h:117`.
    pub plane_d: f32,
    /// How far the centre sits past the surface.
    pub depth: f32,
    pub tri: usize,
}

/// `dBgS::RoofChk`, `bg_s.cpp:351`, through `dBgW::RwgRoofChk` at `d_bg_w.cpp:432`: the lowest
/// surface over a point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RoofHit {
    pub height: f32,
    pub normal: Vec3,
    pub plane_d: f32,
    pub tri: usize,
}

/// What `setFrontWallType`, `main.cpp:4544`, reads off the player before it traces.
#[derive(Debug, Clone, Copy)]
pub struct WallProbe {
    /// `current.pos`.
    pub pos: Vec3,
    /// `shape_angle.y` in radians, which the ledge and catch traces aim from.
    pub shape_yaw: f32,
    /// `current.angle.y` in radians, which only the hidden-wall trace at `main.cpp:4713` aims
    /// from.
    pub yaw: f32,
    /// `mAcchCir[0].GetWallR`, `main.cpp:4545`: `35` standing (`main.cpp:10689`), `30` crawling
    /// (`:10677`), `12.5` hanging (`:10682`) and `5` on a ladder or a vine (`:10663`).
    pub radius: f32,
    /// `mAcchCir[i].GetWallH`, `main.cpp:10654-10656`: `30.1`, `89.9` and `125` standing, and
    /// `main.cpp:10683-10685` puts all three at `25` while hanging.
    pub heights: [f32; 3],
    /// `mAcch.ChkWallHit`, `bg_s_acch.h:152`, which opens the trace at `main.cpp:4550`.
    pub wall_hit: bool,
    /// `m35D4`, the height a ledge has to clear, `main.cpp:4610`.
    pub ledge_h: f32,
    /// `mAcch.GetGroundH`, `bg_s_acch.h:137`, which the airborne band compares against at
    /// `main.cpp:4752`.
    pub ground_h: f32,
    /// `ModeFlg_MIDAIR`.
    pub midair: bool,
    /// `ModeFlg_SWIM`.
    pub swim: bool,
    /// `ModeFlg_HOOKSHOT`.
    pub hookshot: bool,
    /// `ModeFlg_PUSHPULL`.
    pub pushpull: bool,
    /// `checkGrabWear`, `main.cpp:4553`.
    pub grab_wear: bool,
    /// `checkGrabAnime`, `main.cpp:4710` and `main.cpp:4757`.
    pub grab_anime: bool,
    /// The swim-mode flag bit, which `swim.inc:194` sets on entering the swim states and
    /// `swim.inc:134` clears on leaving; `main.cpp:4613` reads its clear state as the way into the
    /// climb and ladder bands.
    pub swim_flag: bool,
    /// `m_HIO->mWallCatch.m.field_0x0C`, the top of the low band at `main.cpp:4760`.
    pub catch_low: f32,
    /// `m_HIO->mWallCatch.m.field_0x10`, the top of the hand band at `main.cpp:4762`.
    pub catch_mid: f32,
    /// `m_HIO->mWallCatch.m.field_0x14`, the top of the mid band at `main.cpp:4764`.
    pub catch_high: f32,
    /// `m_HIO->mWallCatch.m.field_0x18`, the trace offset at `main.cpp:4697` and the height past
    /// which there is no ledge at all, `main.cpp:4755`.
    pub catch_top: f32,
    /// `m_HIO->mSwim.m.field_0x6C`, the swim band at `main.cpp:4748`.
    pub swim_low: f32,
    /// `m_HIO->mWall.m.field_0x50`, the hidden-wall width at `whide.inc:96`.
    pub wall_hide_width: f32,
    /// `m_HIO->mWall.m.field_0x54`, how far along the wall's own normal a hidden wall's cross
    /// point is pushed, `whide.inc:151-156`.
    pub wall_hide_depth: f32,
    /// `mCurProc`, the one exception `main.cpp:4752` makes: a hookshot in flight takes no midair
    /// catch that is more than 100 above the player's feet.
    pub hookshot_fly: bool,
}

/// Which way `getWHideModePolygon`, `whide.inc:76`, was asked about a hidden wall. The two flank
/// traces at `whide.inc:111` and `whide.inc:129` and the turn limit at `whide.inc:148` all key
/// off this word.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HideSide {
    /// `whide.inc:106`: the shorter flank reach, no turn limit, and instead a check that the
    /// wall faces away from the player's own body.
    None,
    /// `whide.inc:149`: the cross point is pushed one way along the wall normal and the turn is
    /// measured off the model's facing.
    Left,
    /// `whide.inc:154`: the same, with the push and the turn the other way round.
    Right,
}

impl HideSide {
    /// The word `getWHideModePolygon` takes.
    pub fn raw(self) -> i32 {
        match self {
            Self::None => 4,
            Self::Left => 2,
            Self::Right => 3,
        }
    }
}

/// `getWHideModePolygon`, `whide.inc:76`: a wall the player can be squeezed past but should not
/// walk through, and the point on it where they stand.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HiddenWall {
    /// `HiddenWall` cross point, moved out along the wall normal by the hidden-wall depth when a side
    /// was asked for, `whide.inc:151-156`.
    pub point: Vec3,
    /// `GetTriPla`, `bg_s.h:117`.
    pub plane: Plane,
    /// The triangle the cross landed on.
    pub tri: usize,
}

/// `getCrawlMoveVec`, `crawl.inc:164`: how much of a crawl joint's segment the stage takes back.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CrawlStop {
    /// The delta to take off the player's position.
    pub push: Vec3,
    /// `crawl.inc:170`: the cross landed on a face `cBgW_CheckBWall`, `c_bg_w.h:285`, calls a
    /// wall, so the push is scaled by what is left of the segment on the ground
    /// (`crawl.inc:172`) rather than taken whole.
    pub wall: bool,
}

/// What `setFrontWallType`, `main.cpp:4544`, leaves behind. The hang, ladder, climb and jump
/// selectors read these words at `main.cpp:4786-4879`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WallTrace {
    /// `mFrontWallType`.
    pub kind: FrontWall,
    /// `m3724`, `main.cpp:4584`: the point on the wall a ledge runs from, which the hang states
    /// stand on and the ladder states count rungs down from.
    pub ledge: Option<Vec3>,
    /// `m352C`, `main.cpp:4576`: the wall's own facing.
    pub wall_yaw: Option<i16>,
    /// `m35F8`, `main.cpp:4666`: the top of a ladder or of a horizontal-edge wall, which
    /// `main.cpp:4809` stops the climb short of.
    pub ledge_top: Option<f32>,
    /// The wall triangle the classification landed on, which every code accessor in
    /// [`ground_code`] reads.
    pub tri: Option<usize>,
    /// `main.cpp:4615`: the trace landed on an auto-jump wall, which latches a reset flag
    /// `main.cpp:2237` and `main.cpp:4821` read to hold the jump back. The reference keeps that in
    /// a flag rather than in the front-wall word, so it rides along here.
    pub auto_jump: bool,
}

/// What `dBgW::RwgWallCorrect`, `d_bg_w.cpp:42`, leaves behind after it has pushed the player
/// out of the walls it found.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WallCorrection {
    /// The player's position after the correction.
    pub pos: Vec3,
    /// `RwgWallCorrect`'s own return value: whether a wall was corrected against.
    pub hit: bool,
    /// `SetWallPolyIndex`, `d_bg_w.cpp:180`: the polygon the correction landed on.
    pub tri: Option<usize>,
    /// `SetWallAngleY`, `d_bg_w.cpp:181`: the wall's own facing.
    pub angle_y: Option<i16>,
}

fn u32_at(data: &[u8], at: usize) -> Option<u32> {
    let bytes = data.get(at..at + 4)?;
    Some(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn f32_at(data: &[u8], at: usize) -> Option<f32> {
    let bytes = data.get(at..at + 4)?;
    Some(f32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn u16_at(data: &[u8], at: usize) -> Option<u16> {
    let bytes = data.get(at..at + 2)?;
    Some(u16::from_be_bytes([bytes[0], bytes[1]]))
}

fn check_table(len: usize, count: u32, offset: u32, stride: usize) -> Result<(), DzbError> {
    let end = (count as usize)
        .checked_mul(stride)
        .and_then(|bytes| (offset as usize).checked_add(bytes))
        .ok_or(DzbError::BadTable)?;
    if end > len {
        return Err(DzbError::BadTable);
    }
    Ok(())
}

fn plane(a: Vec3, b: Vec3, c: Vec3) -> Plane {
    let cross = (b - a).cross(c - a);
    let magnitude = cross.length();
    if magnitude >= PLANE_MIN_MAGNITUDE {
        let normal = cross / magnitude;
        Plane {
            normal,
            d: -normal.dot(a),
        }
    } else {
        Plane {
            normal: Vec3::ZERO,
            d: 0.0,
        }
    }
}

fn inside_xz(a: Vec3, b: Vec3, c: Vec3, x: f32, z: f32) -> bool {
    let d1 = (x - b.x) * (a.z - b.z) - (a.x - b.x) * (z - b.z);
    let d2 = (x - c.x) * (b.z - c.z) - (b.x - c.x) * (z - c.z);
    let d3 = (x - a.x) * (c.z - a.z) - (c.x - a.x) * (z - a.z);
    let has_negative = d1 < 0.0 || d2 < 0.0 || d3 < 0.0;
    let has_positive = d1 > 0.0 || d2 > 0.0 || d3 > 0.0;
    !(has_negative && has_positive)
}

fn crosses(from: Vec3, to: Vec3, p: Vec3, q: Vec3) -> bool {
    let rx = to.x - from.x;
    let rz = to.z - from.z;
    let sx = q.x - p.x;
    let sz = q.z - p.z;
    let denominator = rx * sz - rz * sx;
    if denominator == 0.0 {
        return false;
    }
    let ox = p.x - from.x;
    let oz = p.z - from.z;
    let t = (ox * sz - oz * sx) / denominator;
    if !(t > 0.0 && t <= 1.0) {
        return false;
    }
    let u = (ox * rz - oz * rx) / denominator;
    u >= 0.0 && u <= 1.0
}

/// `cM3d_Cross_LinTri`, called from `cBgW::RwgLineCheck` at `c_bg_w.cpp:385`: the segment meets
/// a triangle where the barycentric coordinates say so. The parameter along `dir` at which that
/// happens, and `None` when the segment misses or lies in the triangle. Which side of the face
/// the segment comes from is the caller's business, `cM3d_Cross_LinPla` at `c_m3d.cpp:249`.
fn segment_triangle(from: Vec3, dir: Vec3, a: Vec3, b: Vec3, c: Vec3) -> Option<f32> {
    let edge1 = b - a;
    let edge2 = c - a;
    let perp = dir.cross(edge2);
    let det = edge1.dot(perp);
    if det.abs() < DET_EPSILON {
        return None;
    }
    let inv = det.recip();
    let offset = from - a;
    let u = offset.dot(perp) * inv;
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let qvec = offset.cross(edge1);
    let v = dir.dot(qvec) * inv;
    if !(0.0..=1.0).contains(&(u + v)) {
        return None;
    }
    let t = edge2.dot(qvec) * inv;
    (t > 0.0 && t <= 1.0).then_some(t)
}

/// `cM3d_Cross_SphTri`, `c_m3d_g_sph.h:47`, which is the closest point of a triangle to a point.
fn closest_on_triangle(point: Vec3, a: Vec3, b: Vec3, c: Vec3) -> Vec3 {
    let ab = b - a;
    let ac = c - a;
    let ap = point - a;
    let d1 = ab.dot(ap);
    let d2 = ac.dot(ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return a;
    }
    let bp = point - b;
    let d3 = ab.dot(bp);
    let d4 = ac.dot(bp);
    if d3 >= 0.0 && d4 <= d3 {
        return b;
    }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        return a + ab * (d1 / (d1 - d3));
    }
    let cp = point - c;
    let d5 = ab.dot(cp);
    let d6 = ac.dot(cp);
    if d6 >= 0.0 && d5 <= d6 {
        return c;
    }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        return a + ac * (d2 / (d2 - d6));
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        return b + (c - b) * ((d4 - d3) / ((d4 - d3) + (d5 - d6)));
    }
    let denom = va + vb + vc;
    if denom == 0.0 {
        return a;
    }
    a + ab * (vb / denom) + ac * (vc / denom)
}

/// `cM3d_SignedLenPlaAndPos`, `c_m3d.h:70`: how far a point sits off a plane along its normal.
fn signed_len(surface: Plane, point: Vec3) -> f32 {
    surface.normal.dot(point) + surface.d
}

/// `cM3d_3PlaneCrossPos`, `c_m3d.h:128`: the one point three planes share, or `None` when their
/// normals do not span three dimensions.
fn intersect3(a: Plane, b: Plane, c: Plane) -> Option<Vec3> {
    let bc = b.normal.cross(c.normal);
    let ca = c.normal.cross(a.normal);
    let ab = a.normal.cross(b.normal);
    let det = a.normal.dot(bc);
    if det.abs() < DET_EPSILON {
        return None;
    }
    Some((bc * -a.d + ca * -b.d + ab * -c.d) / det)
}

/// `cM_ssin` over a 16-bit facing, `c_math.h`, which the hidden-wall ground probe reaches
/// through `cM_atan2s`.
fn sin_of(facing: i16) -> f32 {
    from_s16(facing).sin()
}

/// `cM_atan2s`, `c_math.h:9`: the 16-bit facing of a direction.
fn atan2s(x: f32, z: f32) -> i16 {
    to_s16(x.atan2(z))
}

/// `cLib_distanceAngleS`, `c_lib.cpp:364`: the turn from one facing to another, taken through a
/// signed 16-bit subtraction and an absolute value, so it never folds past half a turn.
fn angle_distance(a: i16, b: i16) -> u16 {
    i32::from(a.wrapping_sub(b) as i16).unsigned_abs() as u16
}

/// `cM3d_Len2dSqPntAndSegLine`, `c_m3d.cpp:46-66`: the closest point of a segment to a point,
/// the squared distance to it, and whether the point's projection falls on the segment.
fn point_segment_sq(point: Vec2, start: Vec2, end: Vec2) -> (bool, Vec2, f32) {
    let delta = end - start;
    let dot = delta.dot(delta);
    if dot < ABS_MIN {
        return (false, start, 0.0);
    }
    let along = delta.dot(point - start) / dot;
    let closest = start + delta * along;
    (
        (0.0..=1.0).contains(&along),
        closest,
        closest.distance_squared(point),
    )
}

/// `cM2d_CrossCirLin`, `c_m2d.cpp:11-55`: where a ray out of a point inside a circle leaves
/// it. The reference takes the larger of the two roots, which is the far side of the circle.
fn cross_circle_line(center: Vec2, radius: f32, start: Vec2, direction: Vec2) -> Vec2 {
    let offset = start - center;
    let quad = direction.dot(direction);
    let linear = 2.0 * direction.dot(offset);
    let constant = offset.length_squared() - radius * radius;
    let mut t = 0.0;
    if quad < ABS_MIN {
        if linear.abs() >= ABS_MIN {
            t = -constant / linear;
        }
    } else {
        let disc = linear * linear - 4.0 * quad * constant;
        if disc.abs() < ABS_MIN {
            t = -linear / (2.0 * quad);
        } else if disc > 0.0 {
            let scale = 1.0 / (2.0 * quad);
            let first = scale * (-linear + disc.sqrt());
            let second = scale * (-linear - disc.sqrt());
            t = first.max(second);
        }
    }
    if t.abs() < ABS_MIN {
        start
    } else {
        start + direction * t
    }
}

/// `dBgS_Acch::GetWallAddY`, `d_bg_s_acch.cpp:467-480`: the ground plane's own drop over the
/// offset the wall test makes, which lifts the wall test height on a slope. The reference's
/// `getCrossYLessD` at `c_m3d_g_pla.h:42` leaves the plane's own offset out, so only the
/// plane's normal is read here.
fn wall_add_y(ground: Option<Vec3>, offset: Vec2) -> f32 {
    let Some(normal) = ground else {
        return 0.0;
    };
    if normal.y < GROUND_NY {
        return 0.0;
    }
    let cross = -(normal.x * offset.x + normal.z * offset.y) / normal.y;
    if cross > 0.0 { 0.0 } else { -cross }
}

/// `dBgW::RwgWallCorrect`'s own cross section, `d_bg_w.cpp:80-151`: the two points where the
/// wall polygon meets the height the wall is tested at. A polygon that lies wholly above or
/// wholly below that height carries no cross section, and one with a single vertex exactly at
/// it is left alone, which is the reference's own `sp8C != 1` arm.
fn wall_cross_section(corners: [Vec3; 3], y: f32) -> Option<(Vec2, Vec2)> {
    let heights = [corners[0].y - y, corners[1].y - y, corners[2].y - y];
    if (heights[0] > 0.0 && heights[1] > 0.0 && heights[2] > 0.0)
        || (heights[0] < 0.0 && heights[1] < 0.0 && heights[2] < 0.0)
    {
        return None;
    }
    let zeros = heights
        .iter()
        .filter(|height| height.abs() < ABS_MIN)
        .count();
    if zeros == 1 {
        return None;
    }
    let (top, left, right) = if (heights[0] > 0.0 && heights[1] <= 0.0 && heights[2] <= 0.0)
        || (heights[0] < 0.0 && heights[1] >= 0.0 && heights[2] >= 0.0)
    {
        (0usize, 1usize, 2usize)
    } else if (heights[1] > 0.0 && heights[0] <= 0.0 && heights[2] <= 0.0)
        || (heights[1] < 0.0 && heights[0] >= 0.0 && heights[2] >= 0.0)
    {
        (1, 0, 2)
    } else {
        (2, 0, 1)
    };
    let over_left = heights[top] - heights[left];
    let over_right = heights[top] - heights[right];
    if over_left.abs() < ABS_MIN || over_right.abs() < ABS_MIN {
        return None;
    }
    let along_left = -heights[left] / over_left;
    let along_right = -heights[right] / over_right;
    let first = corners[left] + (corners[top] - corners[left]) * along_left;
    let second = corners[right] + (corners[top] - corners[right]) * along_right;
    Some((Vec2::new(first.x, first.z), Vec2::new(second.x, second.z)))
}

/// `main.cpp:4662-4692`: the pair of corners of a ladder polygon at one height is its foot, and
/// the third corner its top. The foot point and the top height both come out of that pairing.
fn ladder_ledge(points: [Vec3; 3]) -> (Vec3, f32) {
    let [a, b, c] = points;
    let (foot, apex) = if (a.y - b.y).abs() < 1.0 {
        ((a, b), c)
    } else if (a.y - c.y).abs() < 1.0 {
        ((a, c), b)
    } else {
        ((b, c), a)
    };
    let (first, second) = foot;
    let mut corner = Vec3::new(0.5 * (first.x + second.x), 0.0, 0.5 * (first.z + second.z));
    let top = if apex.y > first.y {
        corner.y = first.y;
        apex.y
    } else {
        corner.y = apex.y;
        first.y
    };
    (corner, top)
}

impl Collision {
    pub fn parse(data: &[u8]) -> Result<Self, DzbError> {
        if data.len() < HEADER_LEN {
            return Err(DzbError::Truncated);
        }
        let vertex_count = u32_at(data, 0).ok_or(DzbError::Truncated)?;
        let vertex_offset = u32_at(data, 4).ok_or(DzbError::Truncated)?;
        let tri_count = u32_at(data, 8).ok_or(DzbError::Truncated)?;
        let tri_offset = u32_at(data, 0x0C).ok_or(DzbError::Truncated)?;
        check_table(data.len(), vertex_count, vertex_offset, VTX_STRIDE)?;
        check_table(data.len(), tri_count, tri_offset, TRI_STRIDE)?;
        check_table(
            data.len(),
            u32_at(data, 0x10).ok_or(DzbError::Truncated)?,
            u32_at(data, 0x14).ok_or(DzbError::Truncated)?,
            BLOCK_STRIDE,
        )?;
        check_table(
            data.len(),
            u32_at(data, 0x18).ok_or(DzbError::Truncated)?,
            u32_at(data, 0x1C).ok_or(DzbError::Truncated)?,
            TREE_STRIDE,
        )?;
        check_table(
            data.len(),
            u32_at(data, 0x20).ok_or(DzbError::Truncated)?,
            u32_at(data, 0x24).ok_or(DzbError::Truncated)?,
            GROUP_STRIDE,
        )?;
        check_table(
            data.len(),
            u32_at(data, 0x28).ok_or(DzbError::Truncated)?,
            u32_at(data, 0x2C).ok_or(DzbError::Truncated)?,
            INFO_STRIDE,
        )?;

        let mut verts = Vec::with_capacity(vertex_count as usize);
        for index in 0..vertex_count as usize {
            let at = vertex_offset as usize + index * VTX_STRIDE;
            verts.push(Vec3::new(
                f32_at(data, at).ok_or(DzbError::Truncated)?,
                f32_at(data, at + 4).ok_or(DzbError::Truncated)?,
                f32_at(data, at + 8).ok_or(DzbError::Truncated)?,
            ));
        }

        let info_count = u32_at(data, 0x28).ok_or(DzbError::Truncated)?;
        let info_offset = u32_at(data, 0x2C).ok_or(DzbError::Truncated)?;
        let mut info0 = Vec::with_capacity(info_count as usize);
        let mut info1 = Vec::with_capacity(info_count as usize);
        for index in 0..info_count as usize {
            let at = info_offset as usize + index * INFO_STRIDE;
            info0.push(u32_at(data, at).ok_or(DzbError::Truncated)?);
            info1.push(u32_at(data, at + 4).ok_or(DzbError::Truncated)?);
        }

        let group_count = u32_at(data, 0x20).ok_or(DzbError::Truncated)?;
        let group_offset = u32_at(data, 0x24).ok_or(DzbError::Truncated)?;
        let mut group_info = Vec::with_capacity(group_count as usize);
        for index in 0..group_count as usize {
            let at = group_offset as usize + index * GROUP_STRIDE;
            group_info.push(u32_at(data, at + 0x30).ok_or(DzbError::Truncated)?);
        }

        let mut tris = Vec::with_capacity(tri_count as usize);
        let mut planes = Vec::with_capacity(tri_count as usize);
        let mut codes = Vec::with_capacity(tri_count as usize);
        let mut room_read = Vec::with_capacity(tri_count as usize);
        for index in 0..tri_count as usize {
            let at = tri_offset as usize + index * TRI_STRIDE;
            let ids = [
                u16_at(data, at).ok_or(DzbError::Truncated)?,
                u16_at(data, at + 2).ok_or(DzbError::Truncated)?,
                u16_at(data, at + 4).ok_or(DzbError::Truncated)?,
            ];
            if ids.iter().any(|&id| usize::from(id) >= verts.len()) {
                return Err(DzbError::VertexIndex);
            }
            let info_id = usize::from(u16_at(data, at + 6).ok_or(DzbError::Truncated)?);
            codes.push(match (info0.get(info_id), info1.get(info_id)) {
                (Some(&info), Some(&raw)) => Self::codes_of(info, raw),
                _ => PolyCodes::default(),
            });
            // `cBgD_Tri_t::m_grp` (`c_bg_w.h:57`) and `cBgW::GetGrpInf` (`c_bg_w.h:179`):
            // the group a polygon belongs to, and the group's own word whose low byte
            // `dBgS::GetGrpRoomInfId` (`d_bg_s.cpp:135`) reads the room read id out of. A
            // polygon whose group is `0xFFFF` names none, which is that function's `0xFF`.
            let group = u16_at(data, at + 8).ok_or(DzbError::Truncated)?;
            room_read.push(
                group_info
                    .get(usize::from(group))
                    .map_or(NO_ROOM_READ, |info| (info & 0xFF) as u8),
            );
            tris.push(ids);
            planes.push(plane(
                verts[usize::from(ids[0])],
                verts[usize::from(ids[1])],
                verts[usize::from(ids[2])],
            ));
        }

        Ok(Self {
            verts,
            tris,
            planes,
            codes,
            room_read,
        })
    }

    /// `cBgD_Ti_t`'s `m_info0` and `m_info1` (`c_bg_w.h:46-50`) split into the five codes the
    /// reference accessors read: `GetExitId` `bg_s.cpp:107` on `m_info0`, and `GetWallCode`
    /// `:172`, `GetSpecialCode` `:177`, `GetAttributeCode` `:188` and `GetGroundCode` `:236` on
    /// `m_info1`.
    fn codes_of(info: u32, raw: u32) -> PolyCodes {
        let attribute = ((raw & ATTRIBUTE_MASK) >> ATTRIBUTE_SHIFT) as u8;
        PolyCodes {
            exit: ((info & EXIT_MASK) >> EXIT_SHIFT) as u8,
            wall: WallCode::from_raw(((raw & WALL_MASK) >> WALL_SHIFT) as u8),
            special: SpecialCode::from_raw(((raw & SPECIAL_MASK) >> SPECIAL_SHIFT) as u8),
            attribute: attribute_from_direct(attribute),
            attribute_direct: attribute,
            ground: GroundCode::from_raw(((raw & GROUND_MASK) >> GROUND_SHIFT) as u8),
        }
    }

    pub fn append(&mut self, other: Self) -> Result<(), DzbError> {
        let total = self
            .verts
            .len()
            .checked_add(other.verts.len())
            .ok_or(DzbError::TooLarge)?;
        if total > usize::from(u16::MAX) + 1 {
            return Err(DzbError::TooLarge);
        }
        let base = self.verts.len() as u32;
        self.tris.extend(other.tris.iter().map(|tri| {
            [
                (tri[0] as u32 + base) as u16,
                (tri[1] as u32 + base) as u16,
                (tri[2] as u32 + base) as u16,
            ]
        }));
        self.planes.extend(other.planes);
        self.codes.extend(other.codes);
        self.room_read.extend(other.room_read);
        self.verts.extend(other.verts);
        Ok(())
    }

    pub fn bounds(&self) -> (Vec2, Vec2) {
        if self.verts.is_empty() {
            return (Vec2::ZERO, Vec2::ZERO);
        }
        let mut min = Vec2::splat(f32::INFINITY);
        let mut max = Vec2::splat(f32::NEG_INFINITY);
        for vert in &self.verts {
            let xz = Vec2::new(vert.x, vert.z);
            min = min.min(xz);
            max = max.max(xz);
        }
        (min, max)
    }

    fn corners(&self, tri: &[u16; 3]) -> [Vec3; 3] {
        [
            self.verts[usize::from(tri[0])],
            self.verts[usize::from(tri[1])],
            self.verts[usize::from(tri[2])],
        ]
    }

    /// `cBgW::RwgGroundCheckCommon`, `c_bg_w.cpp:471`: the highest ground triangle whose column
    /// passes through the point and whose height is under `top`. The reference's ground check
    /// takes its ceiling from the point it was handed, `main.cpp:4629` and `main.cpp:4731`, while
    /// the ground the player stands on is asked for with no ceiling at all.
    ///
    /// `min_ny` is the classification the reference's own query applies to a face: the ground
    /// list is the faces at `cBgW_CheckBGround` (`c_bg_w.h:276`) and steeper, and the wall list
    /// the faces between the roof and the ground, of which `cBgW::RwgGroundCheckWall`
    /// (`c_bg_w.cpp:505`) keeps the ones at `0.014f` and steeper.
    fn ground_hit_filtered(&self, x: f32, top: f32, z: f32, min_ny: f32) -> Option<(f32, usize)> {
        let mut highest: Option<(f32, usize)> = None;
        for (index, tri) in self.tris.iter().enumerate() {
            let surface = self.planes[index];
            if surface.normal.y < min_ny {
                continue;
            }
            let [a, b, c] = self.corners(tri);
            if !inside_xz(a, b, c, x, z) {
                continue;
            }
            let y = -(surface.d + surface.normal.x * x + surface.normal.z * z) / surface.normal.y;
            if y >= top {
                continue;
            }
            let replace = match highest {
                Some((current, _)) => y > current,
                None => true,
            };
            if replace {
                highest = Some((y, index));
            }
        }
        highest
    }

    /// `dBgS::GroundCross`, `c_bg_s.cpp:130-144`, as `cBgW::RwgGroundCheckCommon` at
    /// `c_bg_w.cpp:470` runs it: the ground list and the wall list of every block, the highest
    /// of them under the ceiling.
    fn ground_hit_below(&self, x: f32, top: f32, z: f32) -> Option<(f32, usize)> {
        self.ground_hit_filtered(x, top, z, GROUND_WALL_NY)
    }

    /// The floor under a point, which is the ground list alone: `cBgW_CheckBGround`,
    /// `c_bg_w.h:276`.
    fn ground_hit(&self, x: f32, z: f32) -> Option<(f32, usize)> {
        self.ground_hit_filtered(x, f32::INFINITY, z, GROUND_NY)
    }

    pub fn ground_at(&self, x: f32, z: f32) -> Option<f32> {
        self.ground_hit(x, z).map(|(y, _)| y)
    }

    pub fn ground_normal(&self, x: f32, z: f32) -> Option<Vec3> {
        self.ground_hit(x, z)
            .map(|(_, index)| self.planes[index].normal)
    }

    /// The exit id of the floor under a point.
    pub fn exit_at(&self, x: f32, z: f32) -> Option<u8> {
        let (_, index) = self.ground_hit(x, z)?;
        self.codes.get(index).map(|codes| codes.exit)
    }

    /// `dBgS::GetExitId`, `bg_s.cpp:107`, on the polygon the ground check settles on: the fall
    /// checker reads it off `mAcch.m_gnd` (`main.cpp:10756`), the very triangle the player's own
    /// ground check found, so the exit belongs to the ground the player is on and not to the
    /// highest surface above it. `pos` is the ceiling the ground check is handed.
    pub fn exit_below(&self, pos: Vec3) -> Option<u8> {
        let (_, index) = self.ground_hit_below(pos.x, pos.y, pos.z)?;
        self.codes.get(index).map(|codes| codes.exit)
    }

    /// `dBgS::GroundCross`, `bg_s.h:76`, as `cBgW::RwgGroundCheckCommon` at `c_bg_w.cpp:470`
    /// runs it: the highest ground triangle whose column passes through the point, with the
    /// triangle that carried it.
    pub fn ground_cross(&self, x: f32, z: f32) -> Option<GroundHit> {
        let (y, index) = self.ground_hit(x, z)?;
        Some(GroundHit {
            height: y,
            normal: self.planes[index].normal,
            plane_d: self.planes[index].d,
            tri: index,
        })
    }

    /// `dBgS::GetWallCode` `bg_s.cpp:171`, `GetSpecialCode` `:176`, `GetAttributeCode` `:227` and
    /// `GetGroundCode` `:235` on one triangle, plus the exit `GetExitId` `:106` reads off the
    /// same polygon.
    pub fn tri_codes(&self, tri: usize) -> Option<PolyCodes> {
        self.codes.get(tri).copied()
    }

    /// The same four codes on the ground triangle under a point, which is what
    /// `dComIfG_Bgsp()->GetGroundCode(mAcch.m_gnd)` and its siblings read at `main.cpp:4001`
    /// and `main.cpp:11429`.
    pub fn ground_codes(&self, x: f32, z: f32) -> Option<PolyCodes> {
        let (_, index) = self.ground_hit(x, z)?;
        self.codes.get(index).copied()
    }

    /// `dBgS::GroundCross`, `bg_s.h:76`, with the reference's own ceiling: the highest ground
    /// strictly under `pos`. `main.cpp:4629` and `main.cpp:4731` both hand the ground check a
    /// point and read its height back, which is only this query.
    pub fn ground_cross_below(&self, pos: Vec3) -> Option<GroundHit> {
        let (y, index) = self.ground_hit_below(pos.x, pos.y, pos.z)?;
        Some(GroundHit {
            height: y,
            normal: self.planes[index].normal,
            plane_d: self.planes[index].d,
            tri: index,
        })
    }

    /// `dBgS::GetWallCode`, `bg_s.cpp:171`: `(m_info1 >> 8) & 0xF` on one polygon, which is what
    /// tells a vine from a ladder from a wall with nothing to catch on.
    pub fn wall_code(&self, tri: usize) -> Option<WallCode> {
        self.codes.get(tri).map(|codes| codes.wall)
    }

    /// `dBgS::GetSpecialCode`, `bg_s.cpp:176`: `(m_info1 >> 12) & 0xF`, which is what
    /// `getSlidePolygon` at `main.cpp:4002` and the crawl refusal at `crawl.inc:170` read.
    pub fn special_code(&self, tri: usize) -> Option<SpecialCode> {
        self.codes.get(tri).map(|codes| codes.special)
    }

    /// `dBgS::GetAttributeCode`, `bg_s.cpp:227`: the surface material, read through the
    /// conversion table rather than the raw five bits.
    pub fn attribute_code(&self, tri: usize) -> Option<AttrCode> {
        self.codes.get(tri).map(|codes| codes.attribute)
    }

    /// `dBgS::GetAttributeCodeDirect`, `bg_s.cpp:187`: the raw five bits, which is what
    /// `dBgS_ChangeAttributeCode` at `bg_s.cpp:183` writes and what a caller needs when it wants
    /// to put a material back the way it found it.
    pub fn attribute_code_direct(&self, tri: usize) -> Option<u8> {
        self.codes.get(tri).map(|codes| codes.attribute_direct)
    }

    /// `dBgS::GetGroundCode`, `bg_s.cpp:235`: `(m_info1 >> 21) & 0x1F`, which is what refuses a
    /// ground angle at `main.cpp:2409` and a slope walk at `main.cpp:3016`.
    pub fn ground_code(&self, tri: usize) -> Option<GroundCode> {
        self.codes.get(tri).map(|codes| codes.ground)
    }

    /// `dBgS::GetExitId`, `bg_s.cpp:106`: `(m_info0 >> 13) & 0x3F`, with `0x3F` for a polygon that
    /// opens no exit.
    pub fn exit_id(&self, tri: usize) -> Option<u8> {
        self.codes.get(tri).map(|codes| codes.exit)
    }

    /// `dBgS::GetTriPla`, `bg_s.h:117`: the plane a triangle stands on.
    pub fn tri_plane(&self, tri: usize) -> Option<Plane> {
        self.planes.get(tri).copied()
    }

    /// `dBgS::GetTriPnt`, which `main.cpp:4660` reads the three corners of a ladder polygon off.
    pub fn tri_points(&self, tri: usize) -> Option<[Vec3; 3]> {
        self.tris.get(tri).map(|corners| self.corners(corners))
    }

    /// `dBgS::LineCross`, `bg_s.h:117`, as `cBgW::RwgLineCheck` runs it at `c_bg_w.cpp:385`:
    /// the nearest triangle `from` to `to` runs into, and where. `cBgW_LinChk::SetEnd` writes the
    /// cross back onto the line, so each further triangle is only offered what is still ahead of it
    /// and the hit that survives is the nearest one.
    ///
    /// `cM3d_Cross_LinPla`, `c_m3d.cpp:249-267`, only counts a segment that runs from a face's
    /// front side to its back side: the line check is built with its front flag on and its back
    /// flag off (`c_bg_s_lin_chk.cpp:9`), so a segment that meets the face from behind passes
    /// through it.
    ///
    /// `dBgS::LineCrossNonMoveBG` at `bg_s.cpp:309` is the same test with the moving rooms left
    /// out. A [`Collision`] holds the static rooms of one room set, so there is nothing to
    /// exclude here; a room that moves needs its own [`Collision`].
    pub fn line_cross(&self, from: Vec3, to: Vec3) -> Option<LineHit> {
        let dir = to - from;
        let mut nearest: Option<(f32, usize)> = None;
        for (index, tri) in self.tris.iter().enumerate() {
            let surface = self.planes[index];
            if signed_len(surface, from) < 0.0 || signed_len(surface, to) > 0.0 {
                continue;
            }
            let [a, b, c] = self.corners(tri);
            let Some(t) = segment_triangle(from, dir, a, b, c) else {
                continue;
            };
            if nearest.is_none_or(|(best, _)| t < best) {
                nearest = Some((t, index));
            }
        }
        let (t, tri) = nearest?;
        let surface = self.planes[tri];
        Some(LineHit {
            point: from + dir * t,
            normal: surface.normal,
            plane_d: surface.d,
            tri,
        })
    }

    /// `dBgS::SphChk`, `bg_s.h:121`, as `dBgS::SphChk` at `bg_s.cpp:387` runs it through
    /// `cM3d_Cross_SphTri`, `c_m3d.cpp:1489`: whether a sphere reaches the stage at all, and the
    /// nearest triangle it does. The reference's own answer, `cM3d_CalcSphVsTriCrossPoint`, lies
    /// on the sphere's surface; [`SphereHit::point`] is the point of the triangle, and
    /// [`SphereHit::depth`] is how far short of the surface the centre is, so a caller that wants
    /// the reference's point takes it `radius` out along the same line.
    pub fn sphere_chk(&self, center: Vec3, radius: f32) -> Option<SphereHit> {
        let mut nearest: Option<(f32, usize, Vec3)> = None;
        for (index, tri) in self.tris.iter().enumerate() {
            let [a, b, c] = self.corners(tri);
            let point = closest_on_triangle(center, a, b, c);
            let distance = point.distance(center);
            if distance > radius {
                continue;
            }
            let nearer = match nearest {
                Some((best, _, _)) => best < distance,
                None => true,
            };
            if nearer {
                nearest = Some((distance, index, point));
            }
        }
        let (distance, tri, point) = nearest?;
        let surface = self.planes[tri];
        Some(SphereHit {
            point,
            normal: surface.normal,
            plane_d: surface.d,
            depth: radius - distance,
            tri,
        })
    }

    /// `dBgS::RoofChk`, `bg_s.h:119`, as `dBgW::RwgRoofChk` at `d_bg_w.cpp:432` runs it: the
    /// lowest surface above `pos`. `main.cpp:4703`, `crawl.inc:311` and `climb.inc:176` all take
    /// the answer back as a height and subtract the player's own feet from it.
    ///
    /// The roof is the roof list of the block, `dBgW::ClassifyPlane` at `c_bg_w.cpp:178-180`:
    /// the faces `cBgW_CheckBRoof` (`c_bg_w.h:280`) calls a roof, which point downwards. A face
    /// that points up is a ground, and the room's own floor is never a roof.
    pub fn roof_chk(&self, pos: Vec3) -> Option<RoofHit> {
        let mut lowest: Option<(f32, usize)> = None;
        for (index, tri) in self.tris.iter().enumerate() {
            let surface = self.planes[index];
            if surface.normal.y >= ROOF_NY {
                continue;
            }
            let [a, b, c] = self.corners(tri);
            if !inside_xz(a, b, c, pos.x, pos.z) {
                continue;
            }
            let y = -(surface.d + surface.normal.x * pos.x + surface.normal.z * pos.z)
                / surface.normal.y;
            if y <= pos.y {
                continue;
            }
            let lower = match lowest {
                Some((best, _)) => y < best,
                None => true,
            };
            if lower {
                lowest = Some((y, index));
            }
        }
        let (y, tri) = lowest?;
        let surface = self.planes[tri];
        Some(RoofHit {
            height: y,
            normal: surface.normal,
            plane_d: surface.d,
            tri,
        })
    }

    /// `getSlidePolygon`, `main.cpp:3995`: the ground polygon the slide states run on, which is
    /// the one whose special code is [`SpecialCode::Slide`] and whose ground code is not
    /// [`GroundCode::TooSteep`]. `main.cpp:4900` and `main.cpp:4902` enter the front and back
    /// slides off its normal.
    pub fn slide_polygon(&self, x: f32, z: f32) -> Option<GroundHit> {
        let hit = self.ground_cross(x, z)?;
        let codes = self.codes.get(hit.tri)?;
        (codes.special == SpecialCode::Slide && codes.ground != GroundCode::TooSteep).then_some(hit)
    }

    /// `getCrawlMoveVec`, `crawl.inc:164`: how much of one crawl joint's segment the stage takes
    /// back. A face `cBgW_CheckBWall`, `c_bg_w.h:285`, calls a wall and the push is scaled by the
    /// ground distance the segment still had to run; [`SpecialCode::Slide`] takes the whole
    /// remainder off, and [`SpecialCode::Steep`] takes it only while the normal is under
    /// `CRAWL_STEEP`.
    pub fn crawl_stop(&self, from: Vec3, to: Vec3) -> Option<CrawlStop> {
        let hit = self.line_cross(from, to)?;
        let remaining = to - hit.point;
        if Self::wall(self.planes[hit.tri]) {
            let flat = Vec2::new(remaining.x, remaining.z).length();
            return Some(CrawlStop {
                push: Vec3::new(-flat * hit.normal.x, remaining.y, -flat * hit.normal.z),
                wall: true,
            });
        }
        let special = self.special_code(hit.tri)?;
        let refuses = special == SpecialCode::Slide
            || (special == SpecialCode::Steep && hit.normal.y < CRAWL_STEEP);
        refuses.then_some(CrawlStop {
            push: remaining,
            wall: false,
        })
    }

    /// `getWHideModePolygon`, `whide.inc:76`: a wall the player may be squeezed past but should
    /// not walk through. It has to be level, must not carry [`SpecialCode::SolidWall`], must not
    /// have the player standing on ground behind it, and both flanks have to run along the same
    /// facing. `main.cpp:4718` asks for it with [`HideSide::None`] and reads a hit as front-wall
    /// type 2.
    pub fn hidden_wall(
        &self,
        start: Vec3,
        end: Vec3,
        side: HideSide,
        probe: &WallProbe,
    ) -> Option<HiddenWall> {
        let first = self.line_cross(start, end)?;
        let plane = self.planes[first.tri];
        if plane.normal.y.abs() > WALL_PLANE_FLAT {
            return None;
        }
        let facing = atan2s(plane.normal.x, plane.normal.z);
        if self.special_code(first.tri)? == SpecialCode::SolidWall {
            return None;
        }
        let mut point = first.point;
        let ahead = sin_of(facing) * probe.wall_hide_width;
        let ground = Vec3::new(point.x + ahead, point.y, point.z + ahead);
        if let Some(under) = self.ground_cross_below(ground) {
            if under.normal.y < GROUND_NY {
                return None;
            }
        }
        let reach = if side == HideSide::None {
            HIDE_FLANK_NONE
        } else {
            HIDE_FLANK_SIDE
        };
        let out = plane.normal;
        if side != HideSide::Right {
            let flank = Vec3::new(
                point.x + reach * out.z + out.x * HIDE_FLANK_OUT,
                point.y,
                point.z - reach * out.x + out.z * HIDE_FLANK_OUT,
            );
            if !self.flank_same_facing(flank, out, facing)? {
                return None;
            }
        }
        if side != HideSide::Left {
            let flank = Vec3::new(
                point.x - reach * out.z + out.x * HIDE_FLANK_OUT,
                point.y,
                point.z + reach * out.x + out.z * HIDE_FLANK_OUT,
            );
            if !self.flank_same_facing(flank, out, facing)? {
                return None;
            }
        }
        if side != HideSide::None {
            let depth = probe.wall_hide_depth;
            let turn = if side == HideSide::Left {
                point.x += depth * out.z;
                point.z -= depth * out.x;
                i32::from(facing) - i32::from(to_s16(probe.shape_yaw))
            } else {
                point.x -= depth * out.z;
                point.z += depth * out.x;
                i32::from(to_s16(probe.shape_yaw)) - i32::from(facing)
            };
            if turn > HIDE_TURN_LIMIT {
                return None;
            }
        } else {
            let turned = (i32::from(facing) - 0x8000 - i32::from(to_s16(probe.yaw))) as i16;
            if i32::from(turned).unsigned_abs() > HIDE_YAW_LIMIT {
                return None;
            }
        }
        Some(HiddenWall {
            point,
            plane,
            tri: first.tri,
        })
    }

    /// `whide.inc:111-146`: the short trace each flank of a hidden wall runs, which has to come
    /// back on the same facing as the wall it flanks.
    fn flank_same_facing(&self, from: Vec3, out: Vec3, facing: i16) -> Option<bool> {
        let to = Vec3::new(
            from.x - HIDE_FLANK_BACK * out.x,
            from.y,
            from.z - HIDE_FLANK_BACK * out.z,
        );
        let hit = self.line_cross(from, to)?;
        let flank = self.planes[hit.tri].normal;
        Some(atan2s(flank.x, flank.z) == facing)
    }

    /// `setFrontWallType`, `main.cpp:4544`: what is in front of the player, sorted into the ten
    /// words `mFrontWallType` holds. It traces a level wall ahead of the model, takes the point on
    /// it at the player's own height as the ledge point, and then reads the wall code of whatever
    /// the trace landed on: [`FrontWall::Climb`] off code 1, [`FrontWall::LadderUp`] off 4,
    /// [`FrontWall::LadderDown`] off 5, and [`FrontWall::Wall`] off the rest when the ledge it
    /// solved is out of reach or below [`LEDGE_MIN`]. Every path that gives up leaves
    /// [`FrontWall::Wall`], which is the word `main.cpp:4549` writes before it starts.
    ///
    /// [`FrontWall::Untested`] is only ever the word on entry at `main.cpp:4546`, which happens
    /// when something earlier in the same frame already classified the wall; a caller that asks
    /// once per tick never sees it.
    pub fn front_wall(&self, probe: &WallProbe) -> WallTrace {
        let mut wall = WallTrace {
            kind: FrontWall::Wall,
            ledge: None,
            wall_yaw: None,
            ledge_top: None,
            tri: None,
            auto_jump: false,
        };
        if !probe.wall_hit && !probe.hookshot && !probe.pushpull {
            return wall;
        }
        if probe.grab_wear {
            return wall;
        }
        let (sin, cos) = (probe.shape_yaw.sin(), probe.shape_yaw.cos());
        let mut wall_hit = None;
        for index in (0..probe.heights.len()).rev() {
            let start = Vec3::new(probe.pos.x, probe.pos.y + probe.heights[index], probe.pos.z);
            let reach = WALL_REACH + probe.radius;
            wall_hit = self.line_cross(start, start + Vec3::new(sin * reach, 0.0, cos * reach));
            if wall_hit.is_some() {
                break;
            }
        }
        let Some(mut hit) = wall_hit else {
            return wall;
        };
        let face = self.planes[hit.tri];
        if face.normal.y.abs() > WALL_PLANE_FLAT {
            return wall;
        }
        let facing = atan2s(face.normal.x, face.normal.z);
        let behind = to_s16(probe.shape_yaw).wrapping_add(-0x8000);
        if angle_distance(facing, behind) > WALL_YAW_LIMIT {
            return wall;
        }
        let mut ledge = if probe.midair {
            hit.point
        } else {
            let out = signed_len(face, probe.pos);
            let ledge = Vec3::new(
                probe.pos.x - out * face.normal.x,
                probe.pos.y,
                probe.pos.z - out * face.normal.z,
            );
            let code = self.wall_code(hit.tri);
            if code != Some(WallCode::Ladder) && code != Some(WallCode::LadderGrounded) {
                let back = 2.0 * out;
                let mut flat = None;
                for index in (0..probe.heights.len()).rev() {
                    let start =
                        Vec3::new(probe.pos.x, probe.pos.y + probe.heights[index], probe.pos.z);
                    let end = start - Vec3::new(back * face.normal.x, 0.0, back * face.normal.z);
                    if let Some(even) = self.line_cross(start, end) {
                        if (face.normal - self.planes[even.tri].normal).length() < PARALLEL_LIMIT {
                            flat = Some(even);
                            break;
                        }
                    }
                }
                let Some(even) = flat else {
                    return wall;
                };
                hit = even;
            }
            ledge
        };
        let code = self.wall_code(hit.tri);
        if code == Some(WallCode::NoCatch) || probe.pos.y - probe.ledge_h < LEDGE_DROP {
            return wall;
        }
        let auto_jump =
            probe.swim_flag && code == Some(WallCode::AutoJump) && !probe.swim && !probe.midair;
        wall.auto_jump = auto_jump;
        if auto_jump && probe.pushpull {
            return wall;
        }
        if probe.swim_flag && !auto_jump {
            if code == Some(WallCode::Climb) {
                if probe.midair {
                    let probe_point = Vec3::new(
                        ledge.x - VINE_PROBE_BACK * face.normal.x,
                        VINE_PROBE_UP + probe.pos.y,
                        ledge.z - VINE_PROBE_BACK * face.normal.z,
                    );
                    if let Some(ground) = self.ground_cross_below(probe_point) {
                        if ground.height >= probe.pos.y && ground.normal.y >= GROUND_NY {
                            let across =
                                Vec3::new(face.normal.z, 0.0, face.normal.x).normalize_or_zero();
                            let slice = Plane {
                                normal: across,
                                d: -(across.x * ledge.x + across.z * ledge.z),
                            };
                            if let Some(corner) = intersect3(face, self.planes[ground.tri], slice) {
                                if corner.y - probe.pos.y <= VINE_PROBE_UP {
                                    return WallTrace {
                                        kind: FrontWall::HandLedge,
                                        ledge: Some(corner),
                                        wall_yaw: Some(facing),
                                        ledge_top: None,
                                        tri: Some(hit.tri),
                                        auto_jump: false,
                                    };
                                }
                            }
                        }
                    }
                }
                return WallTrace {
                    kind: FrontWall::Climb,
                    ledge: Some(ledge),
                    wall_yaw: Some(facing),
                    ledge_top: None,
                    tri: Some(hit.tri),
                    auto_jump: false,
                };
            }
            let grounded = code == Some(WallCode::LadderGrounded) && !probe.midair;
            if code == Some(WallCode::Ladder) || grounded {
                let Some(points) = self.tri_points(hit.tri) else {
                    return wall;
                };
                let (corner, top) = ladder_ledge(points);
                return WallTrace {
                    kind: if code == Some(WallCode::Ladder) {
                        FrontWall::LadderUp
                    } else {
                        FrontWall::LadderDown
                    },
                    ledge: Some(corner),
                    wall_yaw: Some(facing),
                    ledge_top: Some(top),
                    tri: Some(hit.tri),
                    auto_jump: false,
                };
            }
        }
        let catch_top = probe.catch_top;
        let start = Vec3::new(probe.pos.x, probe.pos.y + catch_top + 0.1, probe.pos.z);
        let reach = CATCH_REACH + probe.radius;
        let caught = self
            .line_cross(start, start + Vec3::new(sin * reach, 0.0, cos * reach))
            .is_some()
            || self
                .roof_chk(probe.pos)
                .is_some_and(|roof| roof.height - probe.pos.y <= ROOF_CATCH_MARGIN + catch_top);
        if caught {
            if !probe.grab_anime && !probe.midair && !probe.swim {
                let start = Vec3::new(probe.pos.x, probe.pos.y + HIDE_TRACE_UP, probe.pos.z);
                let wide = 2.0 * probe.radius;
                let end = start + Vec3::new(wide * probe.yaw.sin(), 0.0, wide * probe.yaw.cos());
                if self
                    .hidden_wall(start, end, HideSide::None, probe)
                    .is_some()
                {
                    return WallTrace {
                        kind: FrontWall::Hidden,
                        ledge: None,
                        wall_yaw: None,
                        ledge_top: None,
                        tri: None,
                        auto_jump: false,
                    };
                }
            }
            return wall;
        }
        let floor_probe = Vec3::new(
            ledge.x - VINE_PROBE_BACK * face.normal.x,
            probe.pos.y + catch_top + 0.1,
            ledge.z - VINE_PROBE_BACK * face.normal.z,
        );
        let Some(ground) = self.ground_cross_below(floor_probe) else {
            return wall;
        };
        if (!probe.swim && ground.height < probe.pos.y) || ground.normal.y < GROUND_NY {
            return wall;
        }
        let across = Vec3::new(face.normal.z, 0.0, -face.normal.x).normalize_or_zero();
        let slice = Plane {
            normal: across,
            d: -(across.x * ledge.x + across.z * ledge.z),
        };
        let Some(corner) = intersect3(face, self.planes[ground.tri], slice) else {
            return wall;
        };
        ledge = corner;
        let lift = ledge.y - probe.pos.y;
        let kind = if probe.swim {
            (lift <= 0.1 + probe.swim_low && lift >= -SWIM_CATCH_DROP)
                .then_some(FrontWall::HandLedge)
        } else if probe.midair {
            ((ground.height - probe.ground_h) > MIDAIR_CATCH_DROP
                && (!probe.hookshot_fly || lift < MIDAIR_CATCH_LIFT))
                .then_some(FrontWall::HandLedge)
        } else if lift >= 0.1 + catch_top {
            None
        } else if !probe.grab_anime || lift < 0.1 + probe.catch_low {
            if lift < LEDGE_MIN {
                None
            } else if lift < probe.catch_low + 0.1 {
                Some(FrontWall::LowLedge)
            } else if lift < probe.catch_mid + 0.1 {
                Some(FrontWall::HandLedge)
            } else if lift < probe.catch_high + 0.1 {
                Some(FrontWall::MidLedge)
            } else {
                Some(FrontWall::HighLedge)
            }
        } else {
            None
        };
        let Some(kind) = kind else {
            return wall;
        };
        WallTrace {
            kind,
            ledge: Some(ledge),
            wall_yaw: Some(facing),
            ledge_top: None,
            tri: Some(hit.tri),
            auto_jump,
        }
    }

    /// The ledge point `setFrontWallType` leaves in `m3724`, which the hang states stand on and
    /// the ladder states count rungs down from: the point on the front wall at the player's own
    /// height, moved to where the wall meets the floor above it. It is `None` for every
    /// classification that leaves no ledge to catch.
    pub fn ledge(&self, probe: &WallProbe) -> Option<Vec3> {
        self.front_wall(probe).ledge
    }

    /// `checkNotCrawlStand`, `crawl.inc:309`: whether a crawl joint has room, which is the roof above
    /// the point less the player's own feet against the `125` that line compares. `CRAWL_CLEARANCE`
    /// in the move table is the same number.
    pub fn crawl_clear(&self, pos: Vec3, feet: f32, clearance: f32) -> bool {
        self.roof_chk(pos)
            .is_some_and(|roof| roof.height - feet <= clearance)
    }

    /// `dBgW::RwgWallCorrect`, `d_bg_w.cpp:42-259`, as `dBgS::WallCorrect` at `d_bg_s.cpp:332`
    /// runs it: the player's own circle against the wall polygons of the stage, corrected along
    /// the wall's own normal instead of by clamping a coordinate.
    ///
    /// `radius` is `mAcchCir[0].GetWallR` and `heights` the three `GetWallH` values, which
    /// `daPy_lk_c::setBgCheckParam` at `d_a_player_main.cpp:10673-10712` gives the standing
    /// player: `35` for the radius, and `30.1`, `89.9` and `125` for the heights.
    ///
    /// `speed_y` is `dBgS_Acch::GetSpeedY`, `d_bg_s_acch.h:105`, and `ground` the normal of the
    /// plane the ground check found, which `GetWallAddY` at `d_bg_s_acch.cpp:467-480` reads to
    /// lift the wall test height by the ground's own drop over the radius.
    pub fn wall_correct(
        &self,
        pos: Vec3,
        radius: f32,
        heights: &[f32],
        speed_y: f32,
        ground: Option<Vec3>,
    ) -> WallCorrection {
        let mut out = WallCorrection {
            pos,
            hit: false,
            tri: None,
            angle_y: None,
        };
        for index in 0..self.tris.len() {
            let plane = self.planes[index];
            // `cBgW::ClassifyPlane`, `c_bg_w.cpp:175-184`: the ground list takes the faces at
            // `0.5` and flatter, the roof list the faces under `-0.8`, and the wall list the
            // rest. `WallCorrectRp` at `d_bg_w.cpp:270-272` hands the wall list and the roof
            // list to `RwgWallCorrect`, so a roof is corrected against as well.
            if plane.normal.y >= GROUND_NY {
                continue;
            }
            // `d_bg_w.cpp:52-61`: a face whose normal has no horizontal part carries no
            // correction, and the reference walks on to the next polygon of the list.
            let horizontal = plane.normal.x.hypot(plane.normal.z);
            if horizontal < ABS_MIN {
                continue;
            }
            let inverse = 1.0 / horizontal;
            let corners = self.corners(&self.tris[index]);
            // The reference culls by the node tree's box (`d_bg_w.cpp:264`), which holds a whole
            // block of polygons. A box around one polygon is tighter, so the reach is doubled:
            // the player stands within the radius of the wall, and the wall's own cross section
            // is pushed out by the radius as well.
            let reach = 2.0 * radius;
            let min_x = corners[0].x.min(corners[1].x).min(corners[2].x);
            let max_x = corners[0].x.max(corners[1].x).max(corners[2].x);
            let min_z = corners[0].z.min(corners[1].z).min(corners[2].z);
            let max_z = corners[0].z.max(corners[1].z).max(corners[2].z);
            let away_x = (min_x - out.pos.x).max(0.0).max(out.pos.x - max_x);
            let away_z = (min_z - out.pos.z).max(0.0).max(out.pos.z - max_z);
            if away_x * away_x + away_z * away_z > reach * reach {
                continue;
            }
            for height in heights {
                // `d_bg_w.cpp:65-78`: the offset the wall's normal carries over the radius, and
                // the height the wall is tested at.
                let scale = inverse * radius;
                let offset = Vec2::new(scale * plane.normal.x, scale * plane.normal.z);
                let test = wall_add_y(ground, offset) + (out.pos.y + *height) - speed_y;
                let Some((first, second)) = wall_cross_section(corners, test) else {
                    continue;
                };
                // `d_bg_w.cpp:153-173`: the wall's own cross section, pushed out by the radius,
                // and the player's distance to it.
                let (on_segment, closest, seg_sq) = point_segment_sq(
                    Vec2::new(out.pos.x, out.pos.z),
                    first + offset,
                    second + offset,
                );
                let delta = closest - Vec2::new(out.pos.x, out.pos.z);
                let rr = radius * radius;
                if seg_sq > rr || delta.dot(offset) < 0.0 {
                    continue;
                }
                if on_segment {
                    // `dBgW::positionWallCorrect`, `d_bg_w.cpp:32-40`: the player is pushed out
                    // along the wall's own normal by the distance it stands inside the radius.
                    let push = seg_sq.sqrt() * inverse;
                    out.pos.x += push * plane.normal.x;
                    out.pos.z += push * plane.normal.z;
                } else {
                    // `d_bg_w.cpp:186-244`: the corner of the wall, and the circle around the
                    // player the corner has to leave.
                    let player = Vec2::new(out.pos.x, out.pos.z);
                    let first_sq = (first - player).length_squared();
                    let second_sq = (second - player).length_squared();
                    let (corner, corner_sq) = if first_sq < second_sq {
                        (first, first_sq)
                    } else {
                        (second, second_sq)
                    };
                    if corner_sq > rr || (corner_sq - rr).abs() < 0.008 {
                        continue;
                    }
                    let exit = cross_circle_line(
                        player,
                        radius,
                        corner,
                        Vec2::new(-plane.normal.x, -plane.normal.z),
                    );
                    out.pos.x += corner.x - exit.x;
                    out.pos.z += corner.y - exit.y;
                }
                out.hit = true;
                out.tri = Some(index);
                out.angle_y = Some(atan2s(plane.normal.x, plane.normal.z));
            }
        }
        out
    }

    /// `dBgS::GetGrpRoomInfId`, `d_bg_s.cpp:120-136`: the room read id the ground polygon's
    /// group carries, which `dStage_RoomCheck` at `d_stage.cpp:2277` reads to pick the room set
    /// to load. `None` is that function's own `0xFF`, the answer for a polygon whose group
    /// names none. The background's own room number, `mRoomNo2` (`d_bg_w.cpp:23`), is `0xFF` for
    /// the static room backgrounds, so the group is always what answers here.
    pub fn room_read_id(&self, tri: usize) -> Option<u8> {
        let id = *self.room_read.get(tri)?;
        (id != NO_ROOM_READ).then_some(id)
    }

    /// The room read id of the ground polygon under a point, which is the polygon the player
    /// stands on and the one `dStage_RoomCheck` is handed at `d_stage.cpp:2277`.
    pub fn ground_room_read_id(&self, x: f32, z: f32) -> Option<u8> {
        let (_, index) = self.ground_hit(x, z)?;
        self.room_read_id(index)
    }

    fn wall(surface: Plane) -> bool {
        surface.normal != Vec3::ZERO && surface.normal.y < GROUND_NY && surface.normal.y >= ROOF_NY
    }

    fn blocked(&self, from: Vec3, to: Vec3) -> bool {
        if from.x == to.x && from.z == to.z {
            return false;
        }
        let feet = to.y;
        let head = feet + ACTOR_HEIGHT;
        for (index, tri) in self.tris.iter().enumerate() {
            if !Self::wall(self.planes[index]) {
                continue;
            }
            let a = self.verts[usize::from(tri[0])];
            let b = self.verts[usize::from(tri[1])];
            let c = self.verts[usize::from(tri[2])];
            let lowest = a.y.min(b.y).min(c.y);
            let highest = a.y.max(b.y).max(c.y);
            if highest < feet || lowest > head {
                continue;
            }
            if inside_xz(a, b, c, from.x, from.z) {
                continue;
            }
            if inside_xz(a, b, c, to.x, to.z) {
                return true;
            }
            if crosses(from, to, a, b) || crosses(from, to, b, c) || crosses(from, to, c, a) {
                return true;
            }
        }
        false
    }

    pub fn slide(&self, from: Vec3, to: Vec3) -> Vec3 {
        if !self.blocked(from, to) {
            return to;
        }
        if !self.blocked(from, Vec3::new(to.x, to.y, from.z)) {
            return Vec3::new(to.x, to.y, from.z);
        }
        if !self.blocked(from, Vec3::new(from.x, to.y, to.z)) {
            return Vec3::new(from.x, to.y, to.z);
        }
        Vec3::new(from.x, to.y, from.z)
    }
}

pub fn remember_move_from(mut source: ResMut<MoveFrom>, players: Query<&Player>) {
    if let Some(player) = players.iter().next() {
        source.0 = player.pos;
    }
}
