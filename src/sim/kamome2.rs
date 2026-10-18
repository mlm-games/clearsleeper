use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::{Vec2, Vec3};

use super::actor::Actor;
use super::camera::Camera;
use super::collision::Collision;
use super::events::{SimEvent, SimEvents};
use super::flags::OrderFlags;
use super::ground;
use super::intent::{MoveIntent, PadButtons};
use super::player::Player;
use super::stream;
use super::walk;

pub const KAMOME2_ACTOR: &str = "Kamome2";
pub const KAMOME2_ACTORS: [&str; 0] = [];

pub fn is_kamome2(name: &str) -> bool {
    KAMOME2_ACTORS.contains(&name)
}

const D0_COUNT: usize = 5;

const HIO_M05: u8 = 0;
const HIO_M08: f32 = -2500.0;
const HIO_M0C: f32 = 60.0;

const REG0_F0: f32 = 0.0;
const REG0_S0: i16 = 0;
const REG0_F8: f32 = 0.0;
const REG6_F10: f32 = 0.0;
const REG6_F11: f32 = 0.0;
const REG6_F12: f32 = 0.0;

const RES_BCK_WAIT1: u32 = 0x12;
const RES_BCK_WAIT2: u32 = 0x13;
const RES_BAS_WAIT1: u32 = 0x8;
const RES_BAS_WAIT2: u32 = 0x9;

const J3D_MODE_NONE: u8 = 0;
const J3D_MODE_LOOP: u8 = 2;
const J3D_STATE_STOP: u8 = 1;
const J3D_STATE_LOOP: u8 = 2;

const ANM_WAIT1_END: f32 = 60.0;
const ANM_WAIT2_END: f32 = 34.0;
const ANM_WAIT1_LOOP_ATTR: u8 = J3D_MODE_LOOP;
const ANM_WAIT2_LOOP_ATTR: u8 = J3D_MODE_LOOP;
const ANM_PLAY_SPEED: f32 = 1.0;

const KEY_FRAME_TRIGGER: i32 = 9;

const WAIT_SPEED: f32 = 30.0;
const WAIT_SPEED_STEP: f32 = 0.5;
const GLIDE_SPEED_HOLD_X: f32 = 5.0;
const GLIDE_SPEED_STEP_HOLD_X: f32 = 0.2;
const GLIDE_SPEED: f32 = 20.0;
const GLIDE_SPEED_STEP: f32 = 0.1;
const WAIT2_MORF: f32 = 5.0;
const WAIT1_MORF: f32 = 12.0;

const STICK_GATE: f32 = 0.2;
const STICK_YAW_SCALE: f32 = -5.0;
const STICK_PITCH_SCALE: f32 = 5.0;
const STICK_LEAN_SCALE: f32 = -0.4;
const STICK_UNITS: f32 = 1000.0;

const YAW_MAX_PUSHED: i16 = 0x100;
const YAW_MAX_FREE: i16 = 0x400;
const YAW_MAX_CENTRED: i16 = 0x80;
const PITCH_MAX_PUSHED: i16 = 0x200;
const PITCH_MAX_CENTRED: i16 = 0x80;
const PITCH_MAX_GROUND: i16 = 0x800;
const PITCH_SCALE_GROUND: i16 = 10;
const PITCH_SCALE_AIR: i16 = 4;
const PITCH_SCALE_BANK: i16 = 5;

const YAW_LEAN_SCALE: i16 = 10;
const YAW_LEAN_MAX: f32 = 50.0;
const ROLL_SCALE: i16 = 10;
const ROLL_MAX: i16 = 100;
const ROLL_TARGET_SCALE: f32 = -5.0;
const SPEED_SCALE: f32 = 1.0;

const BGCHECK_DROP: f32 = 40.0;
const AT_RADIUS: f32 = 40.0;
const AT_POWER: u32 = 1;
const AT_SPRM: u32 = 0x01 | 0x02;
const AT_TYPE_SWORD: u32 = 1 << 1;
const STTS_AT_ALL: u16 = 0xff;
const STTS_CO_ALL: u16 = 0x00;

const PARAM3: u8 = 3;
const WALL_PARAM3_HEIGHT: f32 = 50.0;
const WALL_PARAM3_RADIUS: f32 = 80.0;
const WALL_DEFAULT_HEIGHT: f32 = 30.0;
const WALL_DEFAULT_RADIUS: f32 = 20.0;
const GROUND_UP_Y: f32 = 40.0;

const D0_START: i16 = 0x1e;
const D0_CAMERA_FRAME: i16 = 1;

const CAM_TRIM_SIZE: i32 = 2;
const CAM_FOLLOW_SCALE: f32 = 0.1;
const CAM_FOLLOW_MAX: f32 = 1000.0;
const CAM_LEAD_X: f32 = 0.015;
const CAM_LEAD_Y: f32 = -0.01;

const EVT_CND_UNK2: u16 = 0x0002;

const SE_CV_KAMOME: u32 = 0x4805;

const CULL_MIN: [f32; 3] = [-40.0, 0.0, -40.0];
const CULL_MAX: [f32; 3] = [40.0, 125.0, 40.0];

const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
const RND_SEED: i32 = 100;

fn rnd(seed: &mut [i32; 3]) -> f32 {
    for lane in 0..3 {
        seed[lane] = (seed[lane] * RND_MUL[lane]) % RND_MOD[lane];
    }
    let sum = (seed[0]) as f32 / RND_SCALE[0]
        + (seed[1]) as f32 / RND_SCALE[1]
        + (seed[2]) as f32 / RND_SCALE[2];
    (sum % 1.0).abs()
}

fn chase_f(value: &mut f32, target: f32, step: f32) -> bool {
    let mut step = step;
    if step != 0.0 {
        if *value > target {
            step = -step;
        }
        *value += step;
        if step * (*value - target) >= 0.0 {
            *value = target;
            return true;
        }
    } else if *value == target {
        return true;
    }
    false
}

fn add_calc2(value: &mut f32, target: f32, scale: f32, max_step: f32) {
    if *value != target {
        let mut step = scale * (target - *value);
        if step > max_step {
            step = max_step;
        } else if step < -max_step {
            step = -max_step;
        }
        *value += step;
    }
}

fn add_calc_xyz(value: &mut Vec3, target: Vec3, scale: f32, max_step: f32) {
    add_calc2(&mut value.x, target.x, scale, max_step);
    add_calc2(&mut value.y, target.y, scale, max_step);
    add_calc2(&mut value.z, target.z, scale, max_step);
}

fn move_vec(angle_y: i16, angle_x: i16, local: Vec3) -> Vec3 {
    let (sin_x, cos_x) = walk::from_s16(angle_x).sin_cos();
    let (sin_y, cos_y) = walk::from_s16(angle_y).sin_cos();
    let z1 = local.z * cos_x - local.y * sin_x;
    let y1 = local.z * sin_x + local.y * cos_x;
    let x2 = local.x * cos_y + z1 * sin_y;
    let z2 = z1 * cos_y - local.x * sin_y;
    Vec3::new(x2, y1, z2)
}

fn banked_up(eye: Vec3, center: Vec3, bank: i16) -> Vec3 {
    let forward = center - eye;
    if forward.length_squared() <= 0.0 {
        return Vec3::Y;
    }
    let right = forward.normalize().cross(Vec3::Y);
    if right.length_squared() <= 0.0 {
        return Vec3::Y;
    }
    let (sin_bank, cos_bank) = walk::from_s16(bank).sin_cos();
    (Vec3::Y * cos_bank + right.normalize() * sin_bank).normalize()
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frame {
    pub attribute: u8,
    pub state: u8,
    pub start: f32,
    pub end: f32,
    pub loop_point: f32,
    pub rate: f32,
    pub frame: f32,
    pub morf_cur: f32,
    pub morf_prev: f32,
    pub morf_step: f32,
    pub bck: u32,
    pub bas: u32,
}

impl Frame {
    fn new(bck: u32, bas: u32, end: f32) -> Self {
        Self {
            attribute: J3D_MODE_LOOP,
            state: 0,
            start: 0.0,
            end,
            loop_point: 0.0,
            rate: 1.0,
            frame: 0.0,
            morf_cur: 1.0,
            morf_prev: 1.0,
            morf_step: 1.0,
            bck,
            bas,
        }
    }

    fn end_of(bck: u32) -> f32 {
        match bck {
            RES_BCK_WAIT1 => ANM_WAIT1_END,
            RES_BCK_WAIT2 => ANM_WAIT2_END,
            _ => ANM_WAIT1_END,
        }
    }

    fn set_morf(&mut self, morf: f32) {
        if self.morf_prev < 0.0 || morf <= 0.0 {
            self.morf_cur = 1.0;
        } else {
            self.morf_cur = 0.0;
            self.morf_step = 1.0 / morf;
        }
        self.morf_prev = self.morf_cur;
    }

    fn update(&mut self) {
        self.state = 0;
        self.frame += self.rate;
        match self.attribute {
            J3D_MODE_NONE => {
                if self.frame < self.start {
                    self.frame = self.start;
                    self.rate = 0.0;
                    self.state |= J3D_STATE_STOP;
                }
                if self.frame >= self.end {
                    self.frame = self.end - 0.001;
                    self.rate = 0.0;
                    self.state |= J3D_STATE_STOP;
                }
            }
            J3D_MODE_LOOP => {
                while self.frame < self.start {
                    self.state |= J3D_STATE_LOOP;
                    if self.loop_point - self.start <= 0.0 {
                        break;
                    }
                    self.frame += self.loop_point - self.start;
                }
                while self.frame >= self.end {
                    self.state |= J3D_STATE_LOOP;
                    if self.end - self.loop_point <= 0.0 {
                        break;
                    }
                    self.frame -= self.end - self.loop_point;
                }
            }
            _ => {}
        }
    }

    fn is_stop(&self) -> bool {
        self.state & J3D_STATE_STOP != 0 || self.rate == 0.0
    }

    fn play(&mut self) -> bool {
        if self.morf_cur < 1.0 {
            self.morf_prev = self.morf_cur;
            chase_f(&mut self.morf_cur, 1.0, self.morf_step);
        }
        self.update();
        self.is_stop()
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Kamome2Hio {
    pub no: i8,
    pub m05: u8,
    pub m08: f32,
    pub m0c: f32,
}

impl Default for Kamome2Hio {
    fn default() -> Self {
        Self {
            no: 0,
            m05: HIO_M05,
            m08: HIO_M08,
            m0c: HIO_M0C,
        }
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Default)]
pub struct Kamome2Cam {
    pub active: bool,
    pub trim_size: i32,
    pub center: Vec3,
    pub eye: Vec3,
    pub bank: i16,
    pub fovy: f32,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Kamome2 {
    pub h68c: bool,
    pub m2a4: u8,
    pub m2a8: i32,
    pub m2ac: i8,
    pub m2bc: f32,
    pub m2c0: f32,
    pub d0: [i16; D0_COUNT],
    pub m2da: i16,
    pub m2dc: i16,
    pub m2ec: u8,
    pub m2f0: Vec3,
    pub m2fc: Vec3,
    pub m308: i16,
    pub m314: f32,
    pub speed_f: f32,
    pub speed: Vec3,
    pub wall_height: f32,
    pub wall_radius: f32,
    pub ground_up_y: f32,
    pub stts_at: u16,
    pub stts_co: u16,
    pub ground_hit: bool,
    pub at_center: Vec3,
    pub at_radius: f32,
    pub at_power: u32,
    pub at_sprm: u32,
    pub at_type: u32,
    pub eye: Vec3,
    pub morph: Frame,
    pub seed: [i32; 3],
}

impl Kamome2 {
    pub fn spawn(parameters: u32) -> Self {
        let mut seed = [RND_SEED; 3];
        let m2a8 = (rnd(&mut seed) * 10000.0) as i32 as i16 as i32;
        let mut d0 = [0; D0_COUNT];
        if let Some(slot) = d0.first_mut() {
            *slot = D0_START;
        }
        let m2a4 = parameters as u8;
        let (wall_height, wall_radius, ground_up_y) = if m2a4 == PARAM3 {
            (WALL_PARAM3_HEIGHT, WALL_PARAM3_RADIUS, GROUND_UP_Y)
        } else {
            (WALL_DEFAULT_HEIGHT, WALL_DEFAULT_RADIUS, 0.0)
        };
        Self {
            h68c: true,
            m2a4,
            m2a8,
            m2ac: 0,
            m2bc: 0.0,
            m2c0: 0.0,
            d0,
            m2da: 0,
            m2dc: 0,
            m2ec: 0,
            m2f0: Vec3::ZERO,
            m2fc: Vec3::ZERO,
            m308: 0,
            m314: 0.0,
            speed_f: 0.0,
            speed: Vec3::ZERO,
            wall_height,
            wall_radius,
            ground_up_y,
            stts_at: STTS_AT_ALL,
            stts_co: STTS_CO_ALL,
            ground_hit: false,
            at_center: Vec3::ZERO,
            at_radius: AT_RADIUS,
            at_power: AT_POWER,
            at_sprm: AT_SPRM,
            at_type: AT_TYPE_SWORD,
            eye: Vec3::ZERO,
            morph: Frame::new(RES_BCK_WAIT1, RES_BAS_WAIT1, ANM_WAIT1_END),
            seed,
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        true
    }

    fn anm_init(
        &mut self,
        anm_res_idx: u32,
        bas_res_idx: u32,
        morf: f32,
        loop_mode: u8,
        play_speed: f32,
    ) {
        let end = Frame::end_of(anm_res_idx);
        let mut next = Frame::new(anm_res_idx, bas_res_idx, end);
        next.attribute = loop_mode;
        next.rate = play_speed;
        next.frame = 0.0;
        next.loop_point = next.frame;
        next.set_morf(morf);
        self.morph = next;
    }

    fn bgcheck(&mut self, actor: &mut Actor, ctx: &Ctx<'_, '_>) {
        let probe = actor.pos - Vec3::Y * BGCHECK_DROP;
        self.ground_hit =
            ground::surface(ctx.collision, probe).is_some_and(|height| probe.y <= height);
        actor.pos.y += BGCHECK_DROP;
    }

    fn key_move(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        let f_var3 = ctx.stick.x;
        let f_var4 = ctx.stick.y;
        let i_var1 = self.morph.frame as i32;

        match self.m2ac {
            0 => {
                self.m2bc = WAIT_SPEED;
                self.m2c0 = WAIT_SPEED_STEP;
                if !ctx.hold_a && i_var1 == i32::from(REG0_S0) + KEY_FRAME_TRIGGER {
                    self.m2ac = 1;
                    self.anm_init(
                        RES_BCK_WAIT1,
                        RES_BAS_WAIT1,
                        REG0_F0 + WAIT1_MORF,
                        ANM_WAIT1_LOOP_ATTR,
                        ANM_PLAY_SPEED,
                    );
                }
            }
            1 => {
                if ctx.hold_x {
                    self.m2bc = GLIDE_SPEED_HOLD_X;
                    self.m2c0 = GLIDE_SPEED_STEP_HOLD_X;
                } else {
                    self.m2bc = GLIDE_SPEED;
                    self.m2c0 = GLIDE_SPEED_STEP;
                }
                if ctx.hold_a {
                    self.m2ac = 0;
                    self.anm_init(
                        RES_BCK_WAIT2,
                        RES_BAS_WAIT2,
                        WAIT2_MORF,
                        ANM_WAIT2_LOOP_ATTR,
                        ANM_PLAY_SPEED,
                    );
                    ctx.se(SE_CV_KAMOME);
                }
            }
            _ => {}
        }

        let mut i_var2 = (f_var3 * (REG6_F10 + STICK_YAW_SCALE) * STICK_UNITS) as i16;
        let mut i_var10 = (f_var4 * (REG6_F11 + STICK_PITCH_SCALE) * STICK_UNITS) as i16;

        let (u_var6, u_var7) = if f_var3.abs() > STICK_GATE {
            (YAW_MAX_PUSHED, YAW_MAX_FREE)
        } else {
            (YAW_MAX_CENTRED, YAW_MAX_CENTRED)
        };
        let mut u_var8 = if f_var4.abs() > STICK_GATE {
            PITCH_MAX_PUSHED
        } else {
            PITCH_MAX_CENTRED
        };
        let u_var9;
        if i_var10 > 0 && self.ground_hit {
            i_var10 = 0;
            u_var8 = PITCH_MAX_GROUND;
            u_var9 = PITCH_SCALE_GROUND;
        } else {
            u_var9 = PITCH_SCALE_AIR;
        }

        stream::add_calc_angle_s2(&mut self.m2dc, i_var2, YAW_LEAN_SCALE, u_var6);
        stream::add_calc_angle_s2(&mut self.m2da, i_var10, u_var9, u_var8);
        add_calc2(
            &mut self.m314,
            f_var3 * (REG6_F12 + STICK_LEAN_SCALE) * STICK_UNITS,
            SPEED_SCALE,
            YAW_LEAN_MAX,
        );

        actor.angle[1] = actor.angle[1].wrapping_add(self.m314 as i16);

        stream::add_calc_angle_s2(&mut actor.angle[0], i_var10, u_var9, u_var8);
        i_var2 = i_var2.wrapping_neg();
        stream::add_calc_angle_s2(&mut actor.angle[2], i_var2, PITCH_SCALE_BANK, u_var7);
        let roll_target = (self.m314 * (REG0_F8 + ROLL_TARGET_SCALE)) as i16;
        stream::add_calc_angle_s2(&mut self.m308, roll_target, ROLL_SCALE, ROLL_MAX);
        add_calc2(&mut self.speed_f, self.m2bc, SPEED_SCALE, self.m2c0);

        self.speed = move_vec(
            actor.angle[1],
            actor.angle[0],
            Vec3::new(0.0, 0.0, self.speed_f),
        );

        actor.pos.x += self.speed.x;
        actor.pos.y += self.speed.y;
        actor.pos.z += self.speed.z;

        self.bgcheck(actor, ctx);

        self.at_center = actor.pos;
    }

    fn execute(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        self.m2a8 = self.m2a8.wrapping_add(1);
        if ctx.hio.m05 == 0 {
            for slot in self.d0.iter_mut() {
                if *slot != 0 {
                    *slot = slot.wrapping_sub(1);
                }
            }

            if self.m2a4 == 0 {
                self.key_move(actor, ctx);
            }
            self.morph.play();
        }

        let mut follow = false;
        match self.m2ec {
            0 => {
                if self.d0.first().copied().unwrap_or(0) == D0_CAMERA_FRAME {
                    self.m2ec = 1;
                }
            }
            1 => {
                if !ctx.demo_accept {
                    ctx.order_flags.set(EVT_CND_UNK2);
                } else {
                    ctx.cam.active = false;
                    ctx.cam.trim_size = CAM_TRIM_SIZE;
                    self.m2ec = 1;
                    self.m2f0 = actor.pos;
                    self.m2fc = actor.pos;
                    follow = true;
                }
            }
            2 => follow = true,
            _ => {}
        }

        if follow {
            let far = move_vec(
                actor.angle[1],
                actor.angle[0],
                Vec3::new(0.0, 0.0, ctx.hio.m08),
            );
            add_calc_xyz(
                &mut self.m2f0,
                actor.pos + far,
                CAM_FOLLOW_SCALE,
                CAM_FOLLOW_MAX,
            );
            let near = move_vec(
                actor.angle[1],
                actor.angle[0],
                Vec3::new(
                    f32::from(self.m2dc) * CAM_LEAD_X,
                    f32::from(self.m2da) * CAM_LEAD_Y,
                    0.0,
                ),
            );
            add_calc_xyz(
                &mut self.m2fc,
                actor.pos + near,
                CAM_FOLLOW_SCALE,
                CAM_FOLLOW_MAX,
            );
        }

        if self.m2ec != 0 {
            ctx.cam.center = self.m2fc;
            ctx.cam.eye = self.m2f0;
            ctx.cam.bank = self.m308;
            ctx.cam.fovy = ctx.hio.m0c;
            let up = banked_up(self.m2f0, self.m2fc, self.m308);
            if let Some(view) = ctx.camera.view_mut() {
                view.aim(self.m2f0, self.m2fc, up);
                view.set_fovy(ctx.hio.m0c);
            }
        }

        self.eye = actor.pos;
    }
}

struct Ctx<'a, 'b> {
    stick: Vec2,
    hold_a: bool,
    hold_x: bool,
    demo_accept: bool,
    hio: &'b Kamome2Hio,
    cam: &'a mut Kamome2Cam,
    camera: &'a mut Camera,
    collision: Option<&'b Collision>,
    events: &'a mut SimEvents,
    order_flags: &'a mut OrderFlags,
}

impl Ctx<'_, '_> {
    fn se(&mut self, id: u32) {
        self.events.list.push(SimEvent::Se(id));
    }
}

#[derive(SystemParam)]
pub struct Watch<'w> {
    collision: Option<Res<'w, Collision>>,
    pad: Res<'w, PadButtons>,
    intent: Res<'w, MoveIntent>,
    camera: ResMut<'w, Camera>,
}

pub fn control(
    mut watch: Watch<'_>,
    hio: Res<Kamome2Hio>,
    mut cam: ResMut<Kamome2Cam>,
    mut events: ResMut<SimEvents>,
    mut order_flags: ResMut<OrderFlags>,
    mut birds: Query<(Entity, &mut Actor, &mut Kamome2), (With<Kamome2>, Without<Player>)>,
) {
    for (_entity, mut actor, mut bird) in &mut birds {
        let actor = &mut *actor;
        let bird = &mut *bird;
        let mut ctx = Ctx {
            stick: watch.intent.axis,
            hold_a: watch.pad.hold.a,
            hold_x: watch.pad.hold.x,
            demo_accept: false,
            hio: &hio,
            cam: &mut cam,
            camera: &mut watch.camera,
            collision: watch.collision.as_deref(),
            events: &mut events,
            order_flags: &mut order_flags,
        };
        bird.execute(actor, &mut ctx);
    }
}
