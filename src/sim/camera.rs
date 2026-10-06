use bevy_ecs::prelude::*;
use glam::{Mat4, Vec3};

use super::cam_follow::{self, CamSetup, Follow, FollowInput, SAngle, ViewCache};
use super::cam_style::{self, param, CamEngine, STYLE_COUNT};
use super::cam_type::{TYPE_FIELD, TYPES, type_index};

const DEFAULT_FOVY: f32 = 0.7853982;
const DEFAULT_ASPECT: f32 = 4.0 / 3.0;
const DEFAULT_NEAR: f32 = 0.1;
const DEFAULT_FAR: f32 = 100000.0;
const WIDE_X: f32 = 640.0;
const WIDE_Y: f32 = 480.0;
const BASE_X: f32 = 320.0;
const BASE_Y: f32 = 240.0;
const DEPTH_CLAMP: f32 = 500000.0;

/// `view_port_class` from `f_op_view.h`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewPort {
    pub x_orig: f32,
    pub y_orig: f32,
    pub width: f32,
    pub height: f32,
    pub near_z: f32,
    pub far_z: f32,
}

impl Default for ViewPort {
    fn default() -> Self {
        Self {
            x_orig: 0.0,
            y_orig: 0.0,
            width: WIDE_X,
            height: WIDE_Y,
            near_z: DEFAULT_NEAR,
            far_z: DEFAULT_FAR,
        }
    }
}

/// `lookat_class` from `f_op_view.h`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LookAt {
    pub eye: Vec3,
    pub center: Vec3,
    pub up: Vec3,
}

impl Default for LookAt {
    fn default() -> Self {
        Self {
            eye: Vec3::new(0.0, 100.0, 200.0),
            center: Vec3::ZERO,
            up: Vec3::Y,
        }
    }
}

/// `view_class` from `f_op_view.h`, narrowed to the fields the sim reads back.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct View {
    pub near: f32,
    pub far: f32,
    pub fovy: f32,
    pub aspect: f32,
    pub lookat: LookAt,
    pub proj: Mat4,
    pub view: Mat4,
    pub inv_view: Mat4,
    pub proj_view: Mat4,
    pub view_no_trans: Mat4,
}

impl Default for View {
    fn default() -> Self {
        Self {
            near: DEFAULT_NEAR,
            far: DEFAULT_FAR,
            fovy: DEFAULT_FOVY,
            aspect: DEFAULT_ASPECT,
            lookat: LookAt::default(),
            proj: Mat4::ZERO,
            view: Mat4::ZERO,
            inv_view: Mat4::ZERO,
            proj_view: Mat4::ZERO,
            view_no_trans: Mat4::ZERO,
        }
    }
}

impl View {
    fn refresh(&mut self) {
        self.view = glam::camera::rh::view::look_at_mat4(self.lookat.eye, self.lookat.center, self.lookat.up);
        self.view_no_trans = self.view;
        self.proj = glam::camera::rh::proj::opengl::perspective(self.fovy, self.aspect, self.near, self.far);
        self.inv_view = self.view.inverse();
        self.proj_view = self.proj * self.view;
    }

    pub fn aim(&mut self, eye: Vec3, center: Vec3, up: Vec3) {
        self.lookat = LookAt { eye, center, up };
        self.refresh();
    }

    /// The aim `dCamera_c` publishes after its engine runs: `mCenter`, `mEye`, `mUp` and
    /// `mBank` for the look-at, `mFovy` converted from the degrees `mViewCache` keeps.
    pub fn aim_rolled(&mut self, eye: Vec3, center: Vec3, up: Vec3, fovy: f32, bank: f32) {
        let forward = (center - eye).normalize_or_zero();
        self.lookat = LookAt {
            eye,
            center,
            up: up.normalize_or_zero().rotate_axis(forward, bank),
        };
        self.fovy = fovy;
        self.refresh();
    }

    pub fn set_fovy(&mut self, fovy: f32) {
        self.fovy = fovy;
        self.refresh();
    }

    pub fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;
        self.refresh();
    }

    /// `mDoLib_pos2camera`: the world point through the view matrix only.
    pub fn pos2camera(&self, src: Vec3) -> Vec3 {
        self.view.transform_point3(src)
    }
}

/// `dComIfGd_getView()` and `dComIfGd_getViewport()`, which `mDoLib_project` needs. The view is
/// absent until the camera is first aimed, matching `dComIfGd_getView() == NULL`.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Camera {
    pub view: Option<View>,
    pub port: ViewPort,
    /// `dCamera_c::mCurType`, a row of [`TYPES`].
    pub cam_type: usize,
    /// `dCamera_c::mCurMode`. Retail gives it no name; it selects one of the twenty style slots.
    pub mode: usize,
    /// `dCamera_c::mCurStyle` / `dCamParam_c::mStyleIdx`, a [`cam_style::CamStyleId`] value.
    pub current_style: i16,
    /// `dCamera_c::m524`, which retail keeps at `0xFF` while no force is pending.
    pub forced_type: Option<usize>,
    /// `dCamera_c::m528`, the actor a forced type is aimed at.
    pub forced_owner: Option<Entity>,
    /// `dCamera_c::mViewCache`: the eye, the point it looks at and the globe joining them.
    pub view_cache: ViewCache,
    /// `dCamera_c::mWork.follow`, the follow engine's cross-frame placement state.
    pub follow: Follow,
    /// `dCamera_c::mCamSetup`, narrowed to what the engines read.
    pub setup: CamSetup,
    /// `dCamera_c::m108`, the frames since the placement was last restarted.
    pub age: u32,
    /// `dCamera_c::m100` with `m101` and `m102`, which `followCamera` always writes together.
    pub settled: bool,
    /// `dCamera_c::mAngleY`, the camera's own yaw channel, `d_camera.cpp:857-862`.
    pub angle_y: SAngle,
    /// This port's own free yaw, staged for this frame only: the debug orbit's movement and the
    /// sub-stick's. The follow engine adds it to the placement's own longitude and clears it.
    /// Retail reaches its free yaw through `mAngleY`, which `d_camera.cpp:861` derives from the
    /// direction itself, so whatever steered it from outside is not in the readable text.
    pub yaw_trim: SAngle,
    /// The orbit yaw last folded into [`Camera::yaw_trim`], so only its movement counts.
    pub orbit_yaw: f32,
}

impl Default for Camera {
    fn default() -> Self {
        let mut view = View::default();
        view.refresh();
        let mut camera = Self {
            view: Some(view),
            port: ViewPort::default(),
            cam_type: TYPE_FIELD,
            mode: 0,
            current_style: cam_style::NONE,
            forced_type: None,
            forced_owner: None,
            view_cache: ViewCache::default(),
            follow: Follow::default(),
            setup: CamSetup::default(),
            age: 0,
            settled: false,
            angle_y: SAngle::ZERO,
            yaw_trim: SAngle::ZERO,
            orbit_yaw: 0.0,
        };
        camera.set_style(TYPE_FIELD, 0);
        camera
    }
}

impl Camera {
    pub fn view(&self) -> Option<&View> {
        self.view.as_ref()
    }

    pub fn view_mut(&mut self) -> Option<&mut View> {
        self.view.as_mut()
    }

    pub fn fovy(&self) -> Option<f32> {
        self.view.as_ref().map(|view| view.fovy)
    }

    /// `onTypeChange` / `onModeChange` / `onStyleChange` / `dCamParam_c::Change`: adopt the style
    /// a type holds for a mode. False when the type, the mode or the style is out of range.
    pub fn set_style(&mut self, cam_type: usize, mode: usize) -> bool {
        let style = match TYPES.get(cam_type).and_then(|t| t.styles.get(mode)) {
            Some(style) => *style,
            None => return false,
        };
        self.cam_type = cam_type;
        self.mode = mode;
        self.change_style(style)
    }

    /// `dCamParam_c::Change`: indices past the table fall back to `dCamStyle_NN00_e`.
    pub fn change_style(&mut self, style: i16) -> bool {
        let known = usize::try_from(style).is_ok_and(|i| i < STYLE_COUNT);
        let style = if known { style } else { 0 };
        if self.current_style != style {
            self.current_style = style;
            self.refresh_fovy();
        }
        known
    }

    /// `dCamParam_c::Val`: the current style's parameter at `index`.
    pub fn val(&self, index: usize) -> f32 {
        cam_style::val_by_id(self.current_style, index)
    }

    /// `dCamParam_c::Algorythmn`, the engine the current style dispatches to.
    pub fn engine(&self) -> Option<CamEngine> {
        cam_style::style_by_id(self.current_style).map(|s| s.engine)
    }

    /// `dCamParam_c::CheckFlag` against the current style.
    pub fn has_flag(&self, mask: u16) -> bool {
        cam_style::style_by_id(self.current_style).is_some_and(|s| s.flag & mask != 0)
    }

    /// The current style's four-character tag.
    pub fn style_tag(&self) -> Option<[u8; 4]> {
        cam_style::style_by_id(self.current_style).map(|s| s.tag)
    }

    /// `dCamParam_c::Fovy(0.0f)` at the neutral ratio: the style's base vertical field of view,
    /// which retail keeps in degrees against a radians projection. Styles that leave the slot at
    /// [`cam_style::PARAM_UNSET`] keep the field of view they already had.
    pub fn refresh_fovy(&mut self) {
        let base = self.val(param::FOVY_BASE);
        if base == cam_style::PARAM_UNSET {
            return;
        }
        self.view_cache.fovy = base;
        if let Some(view) = self.view_mut() {
            view.set_fovy(base.to_radians());
        }
    }

    /// `dCamera_c::SetTypeForce(char*, actor*)`: records a pending camera type and its owner
    /// actor, and refuses while a force is already pending. The type itself is applied by the next
    /// `nextType`, not here.
    pub fn set_type_force(&mut self, name: &str, owner: Option<Entity>) -> bool {
        if self.forced_type.is_some() {
            return false;
        }
        self.forced_type = type_index(name);
        self.forced_owner = owner;
        self.forced_type.is_some()
    }

    /// `dCamera_c::nextType` drops the force once it has been applied.
    pub fn clear_type_force(&mut self) {
        self.forced_type = None;
        self.forced_owner = None;
    }

    /// `mDoLib_project__FP3VecP3Vec`.
    pub fn project(&self, src: Vec3) -> Option<Vec3> {
        self.view
            .as_ref()
            .map(|view| project_with(&view.proj_view, &self.port, src))
    }

    pub fn pos2camera(&self, src: Vec3) -> Option<Vec3> {
        self.view.as_ref().map(|view| view.pos2camera(src))
    }
}

/// `mDoLib_project__FP3VecP3Vec` against an explicit projection-view matrix and viewport.
/// Returns `None` for the no-view case, where the original writes a zero vector.
pub fn project(proj_view: &Mat4, port: &ViewPort, src: Vec3) -> Vec3 {
    project_with(proj_view, port, src)
}

fn project_with(proj_view: &Mat4, port: &ViewPort, src: Vec3) -> Vec3 {
    let m = proj_view.to_cols_array();
    let clip = Vec3::new(
        m[0] * src.x + m[1] * src.y + m[2] * src.z + m[3],
        m[4] * src.x + m[5] * src.y + m[6] * src.z + m[7],
        m[8] * src.x + m[9] * src.y + m[10] * src.z + m[11],
    );
    let w = src.x * m[12] + src.y * m[13] + src.z * m[14] + m[15];
    let mut z = clip.z;
    if z >= 0.0 {
        z = 0.0;
    }
    let scale = if w <= 0.0 {
        if w == 0.0 {
            z *= DEPTH_CLAMP;
        } else {
            z *= 0.5 / w;
        }
        DEPTH_CLAMP
    } else {
        let scale = 0.5 / w;
        z *= scale;
        scale
    };
    let (x_offset, x_size) = if port.x_orig != 0.0 {
        (0.5 * ((2.0 * port.x_orig) + port.width) - BASE_X, WIDE_X)
    } else {
        (port.x_orig, port.width)
    };
    let (y_offset, y_size) = if port.y_orig != 0.0 {
        (0.5 * ((2.0 * port.y_orig) + port.height) - BASE_Y, WIDE_Y)
    } else {
        (port.y_orig, port.height)
    };
    Vec3::new(
        (0.5 + (clip.x * scale)) * x_size + x_offset,
        (0.5 + (clip.y * -scale)) * y_size + y_offset,
        z,
    )
}

fn row(m: &Mat4, index: usize, v: Vec3) -> f32 {
    let r = m.to_cols_array();
    r[index * 4] * v.x + r[index * 4 + 1] * v.y + r[index * 4 + 2] * v.z + r[index * 4 + 3]
}

/// The sub-stick's contribution to the free yaw, in degrees at full deflection, and the gate
/// below which it counts as centred. Retail routes the pad into `mAngleY`, which
/// `d_camera.cpp:851-862` feeds from a camera flag the sim does not carry, so this trim and its
/// rate are this port's own.
const C_STICK_YAW: f32 = 3.0;
const C_STICK_GATE: f32 = 0.2;

/// Retail's camera trails the player along `mLookat`, so the projection the scope and the
/// `pos2camera` reads depend on where the view is actually aimed. Scheduled ahead of the
/// actor systems so every same-tick read sees this tick's view.
///
/// The active style drives the placement: its own radii, latitude, blend rates and field of
/// view, smoothed the way `dCamera_c::followCamera` smooths them. The debug orbit and the
/// sub-stick fold a free yaw in on top of that, which retail's pull chase then pulls back
/// onto the player's back.
pub fn track(
    orbit: Res<super::intent::CameraYaw>,
    sub: Res<super::intent::CStick>,
    main: Res<super::intent::MoveIntent>,
    status: Res<super::player_status::PlayerStatus>,
    players: Query<&super::player::Player>,
    mut camera: ResMut<Camera>,
) {
    let Some(player) = players.iter().next() else {
        return;
    };
    let orbit_delta = (orbit.0 - camera.orbit_yaw).to_degrees();
    camera.orbit_yaw = orbit.0;
    let turn = if sub.value() > C_STICK_GATE {
        sub.axis.x * C_STICK_YAW
    } else {
        0.0
    };
    camera.yaw_trim += SAngle::from_degrees(orbit_delta + turn);
    cam_follow::follow(
        &mut camera,
        player,
        &FollowInput {
            main: main.axis,
            status: &status,
        },
    );
}
