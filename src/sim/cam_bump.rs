use glam::Vec3;

use super::cam_follow::{SAngle, SGlobe, ViewCache, horizontal_distance};
use super::cam_style::{self, CamEngine, param};
use super::camera::Camera;
use super::collision::Collision;
use super::player::Player;
use super::player_status::{self, PlayerStatus};
use super::sea::Sea;

/// `G_CM3D_F_ABS_MIN`, the epsilon `cM3d_IsZero` tests against (`c_m3d.h:135`).
const ABS_MIN: f32 = 3.8146973e-06;

/// `mCamSetup.mBGChk.FwdDistance` / `FwdWeightH` / `FwdWeightL`, `d_cam_param.h:255-258`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChkInfo {
    pub distance: f32,
    pub chk_angle: f32,
    pub weight_h: f32,
    pub weight_l: f32,
}

/// `dCamBGChk_c` from `d_cam_param.h:234-279`, with the constructor's values from
/// `d_cam_param.cpp:119-147`. These are not style parameters: the struct is a member of
/// `dCamSetup_c` (`d_cam_param.h:393`), the thirty-slot style table has no room for them, and
/// the constructor gives every style the same values.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BgChk {
    pub floor_margin: f32,
    pub fwd: [ChkInfo; 2],
    pub fwd_back_margin: f32,
    pub fwd_cushion: f32,
    pub gaze_back_margin: f32,
    pub corner_cushion: f32,
    pub wall_cushion: f32,
    pub wall_up_distance: f32,
    pub wall_back_cushion: f32,
    pub corner_angle_max: f32,
}

impl Default for BgChk {
    fn default() -> Self {
        Self {
            floor_margin: 32.0,
            fwd: [
                ChkInfo {
                    distance: 1.0,
                    chk_angle: 25.0,
                    weight_h: 0.4,
                    weight_l: 0.8,
                },
                ChkInfo {
                    distance: 3.0,
                    chk_angle: 15.0,
                    weight_h: 0.5,
                    weight_l: 0.3,
                },
            ],
            fwd_back_margin: 10.0,
            fwd_cushion: 0.1,
            gaze_back_margin: 10.0,
            corner_cushion: 0.75,
            wall_cushion: 0.25,
            wall_up_distance: 80.0,
            wall_back_cushion: 0.08,
            corner_angle_max: 120.0,
        }
    }
}

/// The event-flag bits the background pass reads and writes. Retail keeps them in
/// `dCamera_c::mEventFlags` alongside bits this port does not track.
pub mod evflag {
    /// `setFlag(8)` at `d_camera.cpp:3239`, read at `:2244` and `:3246`: the placement's own
    /// latitude is blocked.
    pub const GROUND_PROBE: u32 = 0x8;
    /// `setFlag(0x80)` at `d_camera.cpp:2235`, read at `:2945`.
    pub const PULL_IN: u32 = 0x80;
    /// `setFlag(0x4000)` at `d_camera.cpp:2275`, read and cleared at `:2315` and `:2338`.
    pub const WALL_UP: u32 = 0x4000;
    /// `setFlag(0x80000)` at `d_camera.cpp:2225`, read at `:3031`.
    pub const CORNER: u32 = 0x80000;
}

/// `m068`, the flag word `Run` hands `bumpCheck`, built at `d_camera.cpp:784` and
/// `:816-825`. The bits the collision pass branches on are `0x8` (water group, `:2347`),
/// `0x10` (wall-up, `:2244`), `0x20` (the reverse check, `:2173`) and `0x40` (the mass top,
/// `:2141`).
fn check_flags(camera: &Camera) -> u32 {
    let mut flags = 9;
    if camera.has_flag(cam_style::flag::UNK001) {
        flags = 0x3F;
    } else if camera.has_flag(cam_style::flag::UNK002) {
        flags = 0x0F;
    }
    if camera.has_flag(cam_style::flag::UNK400) {
        flags |= 0x40;
    }
    flags
}

/// `dComIfG_Bgsp()->GetTriPla(lin_chk)`, the plane the line check stopped on.
struct Hit {
    /// `lin_chk.GetCross()`, `d_camera.cpp:2201`.
    point: Vec3,
    /// `plane->GetNP()`, `d_camera.cpp:2171`.
    normal: Vec3,
    /// `plane->GetD()`, which the cross point fixes as `normal . point`.
    d: f32,
}

/// `dCamera_c::lineBGCheck`, `d_camera.cpp:1794-1852`. `Collision::line_cross` walks the
/// segment against the stage triangles and reports the plane it landed on, so the wall's
/// facing is the triangle's own normal rather than an axis guess. `lineBGCheckBack` `:1855`
/// and `lineBGCheckBoth` `:1863` only add the back and front sidedness flags, and the
/// reference calls neither.
fn line_bg_check(from: Vec3, to: Vec3, solid: Option<&Collision>) -> Option<Hit> {
    let hit = solid?.line_cross(from, to)?;
    Some(Hit {
        point: hit.point,
        normal: hit.normal,
        d: hit.plane_d,
    })
}

/// `dCamera_c::compWallMargin`, `d_camera.cpp:1907-1922`: a sphere of `radius` around
/// `point`, and every triangle plane the camera's centre faces within that distance pushes
/// the point out along its normal by the shortfall. `Collision` exposes no sphere query, so
/// the push is the same intent taken along the ray the cross point was found on: back off
/// `radius` toward `origin`, then let `slide` take whatever of that is still inside.
fn comp_wall_margin(solid: Option<&Collision>, origin: Vec3, point: Vec3, radius: f32) -> Vec3 {
    let delta = point - origin;
    let length = delta.length();
    if !(length > radius) {
        return point;
    }
    let back = origin + delta * ((length - radius) / length);
    match solid {
        Some(solid) => solid.slide(origin, back),
        None => back,
    }
}

/// `dCamera_c::groundHeight`, `d_camera.cpp:1770-1791`: the higher of the object-layer and
/// the camera-layer ground cross, or the query's own height when neither has ground. The
/// port has the one ground query where retail has the pair.
pub fn ground_height(pos: Vec3, solid: Option<&Collision>) -> f32 {
    solid
        .and_then(|solid| solid.ground_at(pos.x, pos.z))
        .unwrap_or(pos.y)
}

/// `dCamera_c::getWaterSurfaceHeight`, `d_camera.cpp:2367-2402`: the camera ground cross
/// plus five, or a sea area's wave height plus twenty, whichever sits above the eye and
/// higher of the two; the query's own height when neither does. The roof check at
/// `d_camera.cpp:2370-2378` has no port equivalent and is left out.
pub fn water_surface_height(pos: Vec3, solid: Option<&Collision>, sea: Option<&Sea>) -> f32 {
    let mut height = f32::NEG_INFINITY;
    if let Some(ground) = solid.and_then(|solid| solid.ground_at(pos.x, pos.z)) {
        if ground + 5.0 > pos.y {
            height = ground + 5.0;
        }
    }
    if let Some(sea) = sea.filter(|sea| sea.draw_contains(pos.x, pos.z)) {
        let wave = sea.height_at(pos.x, pos.z) + 20.0;
        if wave > pos.y && wave > height {
            height = wave;
        }
    }
    if height == f32::NEG_INFINITY {
        pos.y
    } else {
        height
    }
}

/// `cM3d_PlaneCrossLineProcWork` and `cM3d_2PlaneCrossLine`, `c_m3d.cpp:2296-2332`: the two
/// planes' line of intersection as a point on the dominant axis and that point pushed one
/// cross product along.
fn two_plane_cross_line(a: Vec3, a_d: f32, b: Vec3, b_d: f32) -> Option<(Vec3, Vec3)> {
    let direction = a.cross(b);
    if direction.x.abs() < ABS_MIN && direction.y.abs() < ABS_MIN && direction.z.abs() < ABS_MIN {
        return None;
    }
    let (ax, ay, az) = (a.x, a.y, a.z);
    let (bx, by, bz) = (b.x, b.y, b.z);
    let start = if direction.x.abs() >= direction.y.abs() && direction.x.abs() >= direction.z.abs()
    {
        Vec3::new(
            0.0,
            (az * b_d - bz * a_d) / direction.x,
            (by * a_d - ay * b_d) / direction.x,
        )
    } else if direction.y.abs() >= direction.x.abs() && direction.y.abs() >= direction.z.abs() {
        Vec3::new(
            (bz * a_d - az * b_d) / direction.y,
            0.0,
            (ax * b_d - bx * a_d) / direction.y,
        )
    } else {
        Vec3::new(
            (ay * b_d - by * a_d) / direction.z,
            (bx * a_d - ax * b_d) / direction.z,
            0.0,
        )
    };
    Some((start, start + direction * start.length()))
}

/// `cM3d_lineVsPosSuisenCross` and `cM3d_2PlaneLinePosNearPos`, `c_m3d.cpp:2349-2398`: the
/// point on the planes' intersection line closest to a query point. The reference stores the
/// query point and then divides by the zero length anyway; this keeps the point.
fn plane_line_pos_near_pos(a: Vec3, a_d: f32, b: Vec3, b_d: f32, point: Vec3) -> Option<Vec3> {
    let (start, end) = two_plane_cross_line(a, a_d, b, b_d)?;
    let line = end - start;
    let length = line.length_squared();
    if length < ABS_MIN {
        return Some(point);
    }
    Some(start + line * ((point - start).dot(line) / length))
}

/// `dCamera_c::forwardCheckAngle`, `d_camera.cpp:1981-2062`. Two probes go out in front of
/// the player along the published direction's longitude inverted
/// (`d_camera.cpp:2000`), at `FwdDistance` scaled by the player's height and at
/// `FwdChkAngle` off the horizon. Each is shortened to its line-check cross less
/// `FwdBackMargin`, dropped onto the ground under it, and turned into the latitude from the
/// player's head to that ground point. The two are weighted by `FwdWeightH` above the
/// horizon and `FwdWeightL` below, then combined so the larger of two agreeing probes wins,
/// and a result at or below the horizon is scaled by three quarters.
pub fn forward_check_angle(
    camera: &Camera,
    player: &Player,
    solid: Option<&Collision>,
    player_height: f32,
) -> SAngle {
    let chk = camera.bump.chk;
    let head = player.pos + Vec3::new(0.0, player_height, 0.0);
    let back = camera.bump.direction.longitude().inv();

    let mut weighted = [SAngle::ZERO; 2];
    for (index, slot) in weighted.iter_mut().enumerate() {
        let info = chk.fwd[index];
        let probe = SGlobe::new(
            player_height * info.distance,
            SAngle::from_degrees(info.chk_angle),
            back,
        );
        let mut end = head + probe.to_xyz();

        if let Some(cross) = line_bg_check(head, end, solid) {
            let ray = end - head;
            let length = ray.length();
            end = if length < 1.0 {
                cross.point
            } else {
                cross.point - ray * (chk.fwd_back_margin / length)
            };
        }

        let ground = end.with_y(player_height + ground_height(end, solid));
        let latitude = SGlobe::from_xyz(ground - head).latitude();
        let weight = if latitude >= SAngle::ZERO {
            info.weight_h
        } else {
            info.weight_l
        };
        *slot = latitude * weight;
    }

    let (first, second) = (weighted[0], weighted[1]);
    let mut ret = if first >= SAngle::ZERO && second >= SAngle::ZERO {
        if first >= second { -first } else { -second }
    } else if first <= SAngle::ZERO && second <= SAngle::ZERO {
        if first <= second { -first } else { -second }
    } else {
        -first
    };
    if ret <= SAngle::ZERO {
        ret *= 0.75;
    }
    ret
}

/// The camera-collision state `bumpCheck` carries between frames: `dCamera_c`'s published
/// `mCenter` / `mEye` / `mDirection` triple, the event-flag word, `bumpCheck`'s own
/// `static int prev_hit_type`, and the two ground heights `checkGroundInfo` leaves behind.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bump {
    /// `mCamSetup.mBGChk`.
    pub chk: BgChk,
    /// `bumpCheck`'s `static int prev_hit_type`, `d_camera.cpp:2066`.
    pub hit_type: i32,
    /// `mCenter`, published at `d_camera.cpp:829-843`.
    pub center: Vec3,
    /// `mEye`, published at `d_camera.cpp:2224`, `:2269`, `:2283`, `:2287` and `:2342`.
    pub eye: Vec3,
    /// `mDirection`, published at `d_camera.cpp:2211-2213`, `:2302` and `:2323-2325`.
    pub direction: SGlobe,
    /// `m354`: the ground under the player's feet, `d_camera.cpp:2486`.
    pub floor: f32,
    /// `mBG.m5C.m58`: the ground under the player's head, `d_camera.cpp:2473`.
    pub head_ground: f32,
    /// `m360`, the floor-margin test at `d_camera.cpp:2491-2496`.
    pub on_floor: bool,
}

impl Default for Bump {
    fn default() -> Self {
        Self {
            chk: BgChk::default(),
            hit_type: 0,
            center: Vec3::ZERO,
            eye: Vec3::ZERO,
            direction: SGlobe::new(0.0, SAngle::ZERO, SAngle::ZERO),
            floor: f32::NEG_INFINITY,
            head_ground: f32::NEG_INFINITY,
            on_floor: false,
        }
    }
}

impl Camera {
    /// `dCamera_c::checkGroundInfo`, `d_camera.cpp:2428-2496`, and the placement-ramp reset
    /// it gates at `d_camera.cpp:2906-2909`. `Run` runs the ground info before the engine
    /// and the reset inside it, so this runs before `cam_follow::follow`.
    ///
    /// The two ground crosses at `d_camera.cpp:2441-2488` both land on the player's own XZ,
    /// one lifted by the player's height, and the roof clamp between them has no port
    /// equivalent. `m360` is set when the player's feet are within `FloorMargin` of the
    /// ground under its head. `footHeightOf` is not in the reference text, so the player's
    /// own position stands in for it.
    ///
    /// `m388` is what gates the latitude convergence at `d_camera.cpp:3138` and the wall-up
    /// at `d_camera.cpp:2949-2956`, and a fresh placement leaves it at `0x50`
    /// (`d_camera.cpp:2783`), which is why a placement never converges on its own. Retail
    /// clears it where `m360` holds, and that is the whole of what makes the camera tilt:
    /// `forwardCheckAngle` reaches the latitude target only through that branch. `follow`
    /// reads `Follow::ramp` after incrementing it (`cam_follow.rs:995-999`, against
    /// `d_camera.cpp:2910-2913`), so the reset is staged one tick early as `-1`: the
    /// increment lands it back on zero for the frame the convergence tests.
    pub fn ground_pass(&mut self, player: &Player, solid: Option<&Collision>, player_height: f32) {
        let head = player.pos + Vec3::new(0.0, player_height, 0.0);
        let head_ground = ground_height(head, solid);
        let floor = ground_height(player.pos, solid);

        self.bump.head_ground = head_ground;
        self.bump.floor = floor;
        self.bump.on_floor = player.pos.y - head_ground <= self.bump.chk.floor_margin;

        if self.bump.on_floor {
            self.follow.ramp = -1;
            self.follow.ramp_ratio = 0.0;
        }
    }

    /// `dCamera_c::bumpCheck`, `d_camera.cpp:2065-2363`, with the two things `Run` wraps
    /// around it: `m148` at `d_camera.cpp:775` and the placement probe that raises the
    /// wall-up flag at `d_camera.cpp:3234-3241`.
    ///
    /// `bumpCheck` writes `mDirection` and `mEye`, the pair `Run` publishes at
    /// `d_camera.cpp:845-872`, and it tilts them: the corner case eases the latitude and the
    /// longitude onto the corner at `d_camera.cpp:2211-2213`, the recovery arm eases the
    /// radius and the latitude back onto the placement at `d_camera.cpp:2323-2325`, and the
    /// wall-up at `d_camera.cpp:2257-2276` walks the eye up the wall plane by
    /// `WallUpDistance` so the player stays visible over whatever the camera was pulled
    /// against.
    ///
    /// The locked-on sight radius at `d_camera.cpp:2119-2139` and the mass-top tilt at
    /// `:2141-2150` are left out: the first needs `mpLockonTarget` and the second needs
    /// `m364`, which comes from the actor mass results at `d_camera.cpp:2594-2605`. The
    /// grabbed-actor override of `wall_up_distance` at `d_camera.cpp:2078-2114` needs the
    /// grab target the port does not carry.
    pub fn bump_check(
        &mut self,
        player: &Player,
        solid: Option<&Collision>,
        sea: Option<&Sea>,
        status: &PlayerStatus,
        player_height: f32,
    ) -> bool {
        self.update_forward_angle(player, solid, status, player_height);
        self.probe_placement(solid);
        self.event_flags &= !0x0009_0080;

        let chk = self.bump.chk;
        let flags = check_flags(self);
        let attention = player.pos + Vec3::new(0.0, self.val(param::CENTER_HEIGHT_BASE), 0.0);

        let cache = self.view_cache;
        let floor_limit = self.bump.floor + chk.floor_margin;
        let center = if cache.center.y < floor_limit {
            Vec3::new(cache.center.x, floor_limit, cache.center.z)
        } else {
            cache.center
        };
        if self.bump.direction.radius() == 0.0 {
            self.bump.center = center;
            self.bump.eye = cache.eye;
            self.bump.direction = cache.direction;
        }

        let wall_up_distance = chk.wall_up_distance;
        let mut result = 0u32;

        // every arm assigns both, so they are bound by the match rather than pre-seeded
        let (mut eye, mut direction, hit_type) = match line_bg_check(center, cache.eye, solid) {
            Some(hit) => {
                let back = line_bg_check(cache.eye, center, solid);
                let mut kind = self.classify_hit(&hit, back.as_ref(), flags);
                let corner = if kind == 3 {
                    self.corner_eye(cache, &hit, back.as_ref(), solid)
                } else {
                    None
                };
                match corner {
                    Some((corner_eye, corner_direction)) => {
                        result |= 2;
                        self.event_flags |= evflag::CORNER;
                        (corner_eye, corner_direction, kind)
                    }
                    None => {
                        if kind == 3 {
                            kind = 2;
                        }
                        let (pulled_eye, pulled_direction) =
                            self.pull_in_eye(attention, &hit, kind, wall_up_distance, flags, solid);
                        result |= 1;
                        (pulled_eye, pulled_direction, kind)
                    }
                }
            }
            None => {
                let (relaxed_eye, relaxed_direction) = self.relax_eye(center, cache, flags, solid);
                (relaxed_eye, relaxed_direction, 0)
            }
        };

        if flags & 0x8 != 0 {
            let surface = water_surface_height(eye, solid, sea);
            if surface > eye.y {
                eye.y = surface;
                direction = SGlobe::from_xyz(eye - center);
                result |= 8;
            }
        }

        self.bump.hit_type = hit_type;
        self.bump.eye = eye;
        self.bump.direction = direction;
        self.event_flags &= !0x1014_9C01;
        self.publish_bump();
        result != 0
    }

    /// `m148 += (forwardCheckAngle() - m148) * FwdCushion()` under a style's `UNK004` flag,
    /// `d_camera.cpp:774-779`, and the `m148 *= 0.33f` an airborne player applies at
    /// `d_camera.cpp:2677`, which needs the flight states this port does not publish.
    /// `followCamera` reads the result as the latitude target's offset at
    /// `d_camera.cpp:3147`, through `Follow::lean`.
    fn update_forward_angle(
        &mut self,
        player: &Player,
        solid: Option<&Collision>,
        status: &PlayerStatus,
        player_height: f32,
    ) {
        let angle = if self.has_flag(cam_style::flag::UNK004)
            && !status.check0(player_status::STTS0_UNK4000000)
            && !status.check1(player_status::STTS1_UNK40000)
        {
            forward_check_angle(self, player, solid, player_height)
        } else {
            SAngle::ZERO
        };
        let cushion = self.bump.chk.fwd_cushion;
        self.follow.lean += (angle - self.follow.lean) * cushion;
    }

    /// `d_camera.cpp:3234-3241`: with a pull-in still flagged and the placement's radius
    /// inside the style's near radius, a line from the centre to the eye the placement wants
    /// at the style's own latitude raises the wall-up flag when it is blocked.
    fn probe_placement(&mut self, solid: Option<&Collision>) {
        if !self.follow.probe_allowed || !self.has_flag(cam_style::flag::UNK001) {
            return;
        }
        let near = self.val(param::UNK11);
        if self.event_flags & evflag::PULL_IN == 0 || !(self.bump.direction.radius() < near) {
            return;
        }
        let mut want = self.view_cache.direction;
        want.set_latitude(SAngle::from_degrees(self.val(param::UNK15)));
        let centre = self.view_cache.center;
        if line_bg_check(centre, centre + want.to_xyz(), solid).is_some() {
            self.event_flags |= evflag::GROUND_PROBE;
        }
    }

    /// The `curr_hit_type` classification at `d_camera.cpp:2170-2197`. Without `0x20` in the
    /// flag word the reverse check never runs and every hit is a plain wall. With it, a
    /// reverse cross whose plane faces within `CornerAngleMax` of the forward one, and whose
    /// normals cross more than half way up Y, is a corner; the rest are a wall, and the
    /// corner's own state carries through as `5`.
    fn classify_hit(&self, hit: &Hit, back: Option<&Hit>, flags: u32) -> i32 {
        if flags & 0x20 == 0 {
            return 2;
        }
        let Some(back) = back else {
            return match self.bump.hit_type {
                3 | 5 => 5,
                _ => 2,
            };
        };
        let dot = hit.normal.dot(back.normal);
        let up = hit.normal.cross(back.normal).y.abs();
        if dot > SAngle::from_degrees(self.bump.chk.corner_angle_max).cos() && up > 0.5 {
            3
        } else if self.bump.hit_type != 3 {
            4
        } else {
            5
        }
    }

    /// `d_camera.cpp:2199-2230`, the corner case. The midpoint between the two crosses is
    /// projected onto the planes' intersection line, lifted clear of both by twice their
    /// summed normals, and the direction is eased onto it: the latitude by five percent and
    /// the longitude by `CornerCushion`. That eased direction is where the eye goes, once a
    /// line to a point fifty further along it comes back clear.
    fn corner_eye(
        &self,
        cache: ViewCache,
        hit: &Hit,
        back: Option<&Hit>,
        solid: Option<&Collision>,
    ) -> Option<(Vec3, SGlobe)> {
        let center = self.bump.center;
        let back = back?;
        let mid = hit.point + (back.point - hit.point) * 0.5;
        let corner = plane_line_pos_near_pos(hit.normal, hit.d, back.normal, back.d, mid)?;
        let corner = corner + (hit.normal + back.normal) * 2.0;

        let mut direction = self.bump.direction;
        direction.set_radius(cache.direction.radius());
        let globe = SGlobe::from_xyz(corner - center);
        let latitude = direction.latitude();
        direction.set_latitude(latitude + (globe.latitude() - latitude) * 0.05);
        let longitude = direction.longitude();
        let cushion = self.bump.chk.corner_cushion;
        direction.set_longitude(longitude + (globe.longitude() - longitude) * cushion);

        let mut eye = center + direction.to_xyz();
        let mut reach = globe;
        reach.set_radius(globe.radius() + 50.0);
        if line_bg_check(center, center + reach.to_xyz(), solid).is_some() {
            return None;
        }
        if let Some(cross) = line_bg_check(corner, eye, solid) {
            let margin = self.bump.chk.gaze_back_margin;
            eye = comp_wall_margin(solid, corner, cross.point, margin);
        }
        Some((eye, direction))
    }

    /// `d_camera.cpp:2232-2305`, the wall case. The cross is pushed out of the wall by half
    /// a unit plus `GazeBackMargin`. Then, when the ground probe says the placement's own
    /// latitude is blocked and the flag word allows it, the eye is walked further up the wall
    /// plane if the push did not clear `WallUpDistance` on its own, eased in at
    /// `WallCushion`; otherwise it eases back at `WallBackCushion`. A follow or manual style
    /// then keeps the eye at least forty units off the attention point.
    #[allow(clippy::too_many_arguments)]
    fn pull_in_eye(
        &mut self,
        attention: Vec3,
        hit: &Hit,
        hit_type: i32,
        wall_up_distance: f32,
        flags: u32,
        solid: Option<&Collision>,
    ) -> (Vec3, SGlobe) {
        let chk = self.bump.chk;
        let center = self.bump.center;
        self.event_flags |= evflag::PULL_IN;
        let cross = hit.point;
        let margin = comp_wall_margin(solid, center, cross, 0.5 + chk.gaze_back_margin);

        let lift =
            self.event_flags & evflag::GROUND_PROBE != 0 && flags & 0x10 != 0 && hit_type != 4;
        let lifted = if lift {
            let mut reach = wall_up_distance - (center.y - attention.y);
            let distance = horizontal_distance(cross, center) as f32;
            if !(distance < 20.0) {
                reach = if distance > 320.0 {
                    0.0
                } else {
                    reach * (1.0 - (distance - 20.0) / 300.0)
                };
            }
            if margin.y - center.y < reach {
                let mut up = SGlobe::from_xyz(hit.normal);
                let latitude = up.latitude();
                up.set_latitude(latitude + SAngle::QUARTER);
                up.set_radius(reach * up.latitude().sin());
                let lifted = margin + up.to_xyz();
                let pulled = match line_bg_check(margin, lifted, solid) {
                    Some(hit) => comp_wall_margin(solid, margin, hit.point, chk.gaze_back_margin),
                    None => lifted,
                };
                self.event_flags |= evflag::WALL_UP;
                self.bump.eye += (pulled - self.bump.eye) * chk.wall_cushion;
                return self.finish_pull(attention);
            }
            match line_bg_check(margin, margin, solid) {
                Some(hit) => comp_wall_margin(solid, margin, hit.point, chk.gaze_back_margin),
                None => margin,
            }
        } else {
            margin
        };

        let cushion = if lift { chk.wall_back_cushion } else { 1.0 };
        self.bump.eye += (lifted - self.bump.eye) * cushion;
        self.finish_pull(attention)
    }

    /// The follow and manual engines' floor on the eye's distance from the attention point,
    /// `d_camera.cpp:2290-2300`.
    fn finish_pull(&mut self, attention: Vec3) -> (Vec3, SGlobe) {
        if matches!(
            self.engine(),
            Some(CamEngine::Follow) | Some(CamEngine::Manual)
        ) {
            let mut globe = SGlobe::from_xyz(self.bump.eye - attention);
            if globe.radius() < 40.0 {
                globe.set_radius(40.0);
                self.bump.eye = attention + globe.to_xyz();
            }
        }
        let direction = SGlobe::from_xyz(self.bump.eye - self.bump.center);
        (self.bump.eye, direction)
    }

    /// `d_camera.cpp:2312-2345`, the frame nothing blocks. With the wall-up flag latched the
    /// radius and the latitude ease back onto the placement at `WallBackCushion`, or at a
    /// fifth where the flag word does not allow the wall-up, the longitude snaps onto it, and
    /// the result is then pushed out of whatever it lands in. The latch clears once the
    /// latitude has come back within a fifth of a degree.
    fn relax_eye(
        &mut self,
        center: Vec3,
        cache: ViewCache,
        flags: u32,
        solid: Option<&Collision>,
    ) -> (Vec3, SGlobe) {
        if self.event_flags & evflag::WALL_UP == 0 {
            return (cache.eye, cache.direction);
        }
        let cushion = if flags & 0x10 != 0 {
            self.bump.chk.wall_back_cushion
        } else {
            0.2
        };
        let mut direction = self.bump.direction;
        let radius = direction.radius();
        direction.set_radius(radius + (cache.direction.radius() - radius) * cushion);
        let latitude = direction.latitude();
        direction.set_latitude(latitude + (cache.direction.latitude() - latitude) * cushion);
        direction.set_longitude(cache.direction.longitude());

        let mut eye = center + direction.to_xyz();
        if let Some(hit) = line_bg_check(center, eye, solid) {
            eye = comp_wall_margin(
                solid,
                center,
                hit.point,
                0.5 + self.bump.chk.gaze_back_margin,
            );
        }
        let gap = (direction.latitude() - cache.direction.latitude())
            .degree()
            .abs();
        if gap < 0.2 {
            self.event_flags &= !evflag::WALL_UP;
        }
        (eye, direction)
    }

    /// `d_camera.cpp:845-872`: `Run` copies the field of view and the bank off the cache,
    /// runs `bumpCheck`, and derives the aim and the free yaw from what came back.
    fn publish_bump(&mut self) {
        let bump = self.bump;
        let up = if bump.center.x == bump.eye.x && bump.center.z == bump.eye.z {
            Vec3::new(0.01, 1.0, 0.0)
        } else if bump.direction.latitude() >= SAngle::from_degrees(-90.0)
            && bump.direction.latitude() <= SAngle::from_degrees(90.0)
        {
            Vec3::Y
        } else {
            Vec3::new(0.0, -1.0, 0.0)
        };
        self.angle_y = bump.direction.longitude().inv();
        let cache = self.view_cache;
        if let Some(view) = self.view_mut() {
            view.aim_rolled(
                bump.eye,
                bump.center,
                up,
                cache.fovy.to_radians(),
                cache.bank.radian(),
            );
        }
    }
}
