use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::Vec3;

use super::actor::{Actor, atan2s};
use super::area::StageName;
use super::combat::HitPoints;
use super::events::{SimEvent, SimEvents};
use super::flags::MemBits;
use super::gohma::Gohma;
use super::player::Player;
use super::stream::add_calc_angle_s2;
use super::walk::from_s16;

pub const DR2_ACTORS: [&str; 1] = ["Dr2"];

pub fn is_dr2(name: &str) -> bool {
    DR2_ACTORS.contains(&name)
}

const HIO_NO_INIT: i8 = -1;
const HIO_SCALE: f32 = 1.0;

const STATE_RISE: i16 = -1;
const STATE_IDLE: i16 = 0;
const STATE_FALL: i16 = 1;
const STATE_ATTACH: i16 = 2;
const STATE_LIFT: i16 = 3;
const STATE_SHATTER: i16 = 10;
const STATE_SHATTER_DONE: i16 = 11;

const MODE_REST: u8 = 0;
const MODE_SCALE: u8 = 1;
const MODE_AIM: u8 = 2;
const MODE_RATTLE: u8 = 3;

const EMIT_NONE: u8 = 0;
const EMIT_SPAWN: u8 = 1;
const EMIT_RUN: u8 = 2;

const RETURN_TICKS: i16 = 0x82;
const HOLD_JOINT: i16 = 0x32;
const HOLD_RISE: i16 = 1;

const MAX_HEALTH: i32 = 1;

const COUNT_HAHEN: usize = 6;
const COUNT_HOLD: usize = 5;
const COUNT_JOINT: usize = 12;
const JOINT_WAVE_FIRST: usize = 2;

const ANGLE_X: usize = 0;
const ANGLE_Y: usize = 1;
const ANGLE_Z: usize = 2;

const DROP_RANGE: f32 = 200.0;
const DROP_EPS: f32 = 1.0;
const RISE_SCALE: f32 = 0.5;
const RISE_MAX: f32 = 30.0;
const SPIN_SCALE: i16 = 1;
const SPIN_MAX: i16 = 0x100;
const TRACK_SCALE: f32 = 0.5;
const TRACK_MAX: f32 = 50.0;
const ATTACH_SCALE: i16 = 1;
const ATTACH_MAX: i16 = 0x200;
const GRAVITY: f32 = 10.0;
const ATTACH_PITCH: f32 = -330.0;
const TOP_OFFSET: f32 = 50.0;
const TOP_FILL: f32 = 0.3;
const TOP_BOTTOM: f32 = 100.0;
const LIFT_SIT: f32 = 100.0;
const LIFT_LEAD: f32 = 1200.0;
const LIFT_RISE: f32 = 100.0;
const RATTLE_TARGET: f32 = 2000.0;
const RATTLE_SCALE: f32 = 0.5;
const RATTLE_MAX_STEP: f32 = 100.0;
const SHATTER_SPREAD: f32 = 3.0;
const SHATTER_TUMBLE: i16 = 300;
const SHATTER_DRIFT: i16 = 256;
const FALL_GRAVITY: f32 = 0.5;
const DEATH_Y: f32 = -100.0;
const SWAY_Y_STEP: f32 = 0.04;
const SWAY_Z_STEP: f32 = 0.05;
const TAU_F: f32 = 6.2831855;
const SWAY_AMP: f32 = 0.048;
const SWAY_TILT: f32 = -0.175;
const SWAY_TILT_SCALE: f32 = 0.5;
const SWAY_TILT_STEP: f32 = 0.0125;
const SWAY_Z_TILT_SCALE: f32 = 0.5;
const SWAY_Z_TILT_MAX: f32 = 0.10000000149011612;
const TILT_LIMIT: f32 = -0.1;
const STRETCH_TARGET: f32 = 1.04;
const STRETCH_SCALE: f32 = 0.1;
const STRETCH_STEP: f32 = 0.005;
const REST_SCALE: f32 = 0.98;
const AIM_TRACK: f32 = -0.125;
const AIM_SCALE: i16 = 0x10;
const AIM_MAX: i16 = 0x40;
const HOLD_TICKS: i16 = 20;
const SWAY_EPS: f32 = 0.001;
const HILL_SCALE: f32 = 300.0;
const HILL_OFF: f32 = 20.0;
const HILL_RATE: i16 = 3;
const HOLD_RATE: i16 = 1;
const FALL_RATE: i16 = 30;
const DROP_RATE: i16 = 10;
const BURN_TICKS: i16 = 20;
const DROP_TICKS: i16 = 1;
const REST_TICKS: i16 = 2;
const HAND_Y: f32 = 55.0;

const WAVE_Y_MUL: i32 = 2800;
const WAVE_Y_STEP: i32 = 3000;
const WAVE_Z_MUL: i32 = 2300;
const WAVE_Z_STEP: i32 = 2000;

const REG0_S2: i16 = 0;
const REG0_S3: i16 = 0;
const REG0_S4: i16 = 0;
const REG0_S5: i16 = 0;
const REG0_S6: i16 = 0;
const REG0_F2: f32 = 0.0;
const REG0_F3: f32 = 0.0;
const REG0_F5: f32 = 0.0;
const REG0_F6: f32 = 0.0;
const REG0_F7: f32 = 0.0;
const REG0_F9: f32 = 0.0;
const REG0_F10: f32 = 0.0;
const REG0_F14: f32 = 0.0;

const SE_BTD_STN_FALL: u32 = 0x5828;
const SE_BTD_ROCK_FALL: u32 = 0x582A;
const SE_BTD_ROCK_HIT: u32 = 0x582B;
const SE_BTD_ROCK_ATTACH: u32 = 0x582F;

const RND_SEED: i32 = 0x3a5f_1b2c;
const RND_MUL: [i32; 3] = [1103515245, 1103515245, 1103515245];
const RND_MOD: [i32; 3] = [2147483647, 2147483647, 2147483647];
const RND_SCALE: [f32; 3] = [2147483647.0, 2147483647.0, 2147483647.0];

const KUI_ACTOR: &str = "Kui";
const KUI_PARAMETERS: u32 = 0x511;

const HSX: [f32; COUNT_HAHEN] = [-10.0, -10.0, 0.0, 10.0, 10.0, 0.0];
const HSZ: [f32; COUNT_HAHEN] = [-7.0, 7.0, 12.0, 7.0, -7.0, -12.0];

const CULL_MIN: [f32; 3] = [-1000.0, -1000.0, -1000.0];
const CULL_MAX: [f32; 3] = [1000.0, 1000.0, 1000.0];

#[derive(Resource, Debug, Clone, Copy)]
pub struct Dr2Hio {
    pub no: i8,
    pub scale: f32,
}

impl Default for Dr2Hio {
    fn default() -> Self {
        Self {
            no: HIO_NO_INIT,
            scale: HIO_SCALE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hahen {
    pub live: bool,
    pub pos: Vec3,
    pub vel: Vec3,
    pub rot: [i16; 3],
}

impl Hahen {
    const SRC: Hahen = Hahen {
        live: false,
        pos: Vec3::ZERO,
        vel: Vec3::ZERO,
        rot: [0; 3],
    };
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Dr2 {
    pub health: i32,
    pub max_health: i32,
    pub tick: i16,
    pub fragments: [Hahen; COUNT_HAHEN],
    pub hane_l2: Vec3,
    pub wave_y: [i16; COUNT_JOINT],
    pub wave_z: [i16; COUNT_JOINT],
    pub tilt_y: f32,
    pub tilt_z: f32,
    pub phase_y: f32,
    pub phase_z: f32,
    pub kui_parameters: u32,
    pub hold: [i16; COUNT_HOLD],
    pub mode: u8,
    pub scale_x: f32,
    pub amp: f32,
    pub rattle: f32,
    pub mbyo_broken: bool,
    pub rock_pos: Vec3,
    pub rock_ang: [i16; 3],
    pub state: i16,
    pub home_angle: i16,
    pub lift: f32,
    pub dr_state: u8,
    pub dr_timer: i16,
    pub rise_timer: i16,
    pub hold_timer: i16,
    pub spin_timer: i16,
    pub spin_ang: i16,
    pub spin_rad: f32,
    pub emitter: bool,
    pub emitter_delete: bool,
    pub dr_live: bool,
    pub rate: u8,
    pub emit_mode: u8,
    pub emit_hold: i16,
    pub home: Vec3,
    pub seed: [i32; 3],
    pub entered: bool,
}

impl Dr2 {
    pub fn spawn(parameters: u32) -> Self {
        let _ = parameters;
        Self {
            health: MAX_HEALTH,
            max_health: MAX_HEALTH,
            tick: 0,
            fragments: [Hahen::SRC; COUNT_HAHEN],
            hane_l2: Vec3::ZERO,
            wave_y: [0; COUNT_JOINT],
            wave_z: [0; COUNT_JOINT],
            tilt_y: 0.0,
            tilt_z: 0.0,
            phase_y: 0.0,
            phase_z: 0.0,
            kui_parameters: 0,
            hold: [0; COUNT_HOLD],
            mode: MODE_REST,
            scale_x: 0.0,
            amp: 0.0,
            rattle: 0.0,
            mbyo_broken: false,
            rock_pos: Vec3::ZERO,
            rock_ang: [0; 3],
            state: STATE_IDLE,
            home_angle: 0,
            lift: 0.0,
            dr_state: 0,
            dr_timer: 0,
            rise_timer: 0,
            hold_timer: 0,
            spin_timer: 0,
            spin_ang: 0,
            spin_rad: 0.0,
            emitter: false,
            emitter_delete: false,
            dr_live: false,
            rate: 0,
            emit_mode: EMIT_NONE,
            emit_hold: 0,
            home: Vec3::ZERO,
            seed: [RND_SEED; 3],
            entered: false,
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        true
    }

    fn rnd(&mut self) -> f32 {
        for lane in 0..3 {
            self.seed[lane] = self.seed[lane].wrapping_mul(RND_MUL[lane]) % RND_MOD[lane];
        }
        let sum = self.seed[0] as f32 / RND_SCALE[0]
            + self.seed[1] as f32 / RND_SCALE[1]
            + self.seed[2] as f32 / RND_SCALE[2];
        (sum % 1.0).abs()
    }

    fn rnd_fx(&mut self, range: f32) -> f32 {
        range * (self.rnd() - 0.5) * 2.0
    }
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

fn add_calc0(value: &mut f32, scale: f32, max_step: f32) {
    let mut step = *value * scale;
    if step > max_step {
        step = max_step;
    } else if step < -max_step {
        step = -max_step;
    }
    *value -= step;
}

fn bearing(from: Vec3, to: Vec3) -> i16 {
    let units = (to.x - from.x).atan2(to.z - from.z) * (65536.0 / core::f32::consts::PI);
    units.round().rem_euclid(65536.0) as i32 as i16
}

struct Env {
    player_pos: Vec3,
    player_angle: [i16; 3],
    player_shape: [i16; 3],
    stage_boss_enemy: bool,
    start_stage_first: u8,
}

struct Sink<'a> {
    events: &'a mut SimEvents,
}

impl Sink<'_> {
    fn se(&mut self, id: u32) {
        self.events.list.push(SimEvent::Se(id));
    }
}

#[derive(SystemParam)]
pub struct Watch<'w, 's> {
    gohmas: Query<
        'w,
        's,
        (Entity, &'static Actor, &'static mut Gohma),
        (With<Gohma>, Without<Dr2>, Without<Player>),
    >,
    kuis: Query<
        'w,
        's,
        (&'static Actor, Option<&'static HitPoints>),
        (Without<Player>, Without<Dr2>, Without<Gohma>),
    >,
    mem: Res<'w, MemBits>,
    stage: Res<'w, StageName>,
}

struct BtdProbe {
    angle: i16,
    ground: Vec3,
    far: Vec3,
    ledge: Vec3,
    down: bool,
}

pub fn control(
    watch: Watch<'_, '_>,
    mut events: ResMut<SimEvents>,
    hio: Res<Dr2Hio>,
    mut commands: Commands,
    mut players: Query<(Entity, &mut Actor, &Player), (With<Player>, Without<Dr2>)>,
    mut rocks: Query<(Entity, &mut Actor, &mut Dr2), (With<Dr2>, Without<Player>)>,
) {
    let stage_boss_enemy = watch.mem.is_stage_boss_enemy();
    let start_stage_first = watch.stage.0.as_bytes().first().copied().unwrap_or(0);
    let scale = hio.scale;
    let kui = watch
        .kuis
        .iter()
        .find(|(actor, _)| actor.name == KUI_ACTOR && actor.parameters == KUI_PARAMETERS)
        .map(|(actor, hp)| (actor.pos, hp.map(|points| points.0).unwrap_or(0)));
    let mut gohmas = watch.gohmas;
    let mut player = players.iter_mut().next();
    for (_, mut actor, mut rock) in &mut rocks {
        let (player_pos, player_angle, player_shape) = match player.as_ref() {
            Some((_, pactor, pplayer)) => (pplayer.pos, pactor.angle, pactor.shape_angle),
            None => (Vec3::ZERO, [0; 3], [0; 3]),
        };
        let btd = gohmas.iter_mut().next().map(|(_, bactor, boss)| BtdProbe {
            angle: bactor.angle[ANGLE_Y],
            ground: Vec3::from(boss.m02fc[3]),
            far: Vec3::from(boss.m02fc[4]),
            ledge: Vec3::from(boss.m02fc[8]),
            down: boss.m02f6 != 0,
        });
        if !rock.entered {
            rock.begin(&mut actor, &mut commands);
        }
        let mut sink = Sink {
            events: &mut events,
        };
        let mut env = Env {
            player_pos,
            player_angle,
            player_shape,
            stage_boss_enemy,
            start_stage_first,
        };
        let struck = rock.execute(&mut actor, &mut env, btd, kui, &mut sink);
        if struck {
            if let Some((_, _, ref mut boss)) = gohmas.iter_mut().next() {
                boss.m02f6 = 1;
            }
        }
        actor.scale = Vec3::splat(scale);
        if let Some((_, pactor, _)) = player.as_mut() {
            pactor.angle = env.player_angle;
            pactor.shape_angle = env.player_shape;
        }
    }
}

impl Dr2 {
    fn begin(&mut self, actor: &mut Actor, commands: &mut Commands) {
        self.entered = true;
        self.home = actor.pos;
        self.home_angle = actor.angle[ANGLE_Y];
        self.rock_pos = Vec3::new(actor.pos.x, actor.pos.y + REG0_F7 - TOP_OFFSET, actor.pos.z);
        self.kui_parameters = KUI_PARAMETERS;
        commands.spawn((
            Actor {
                name: KUI_ACTOR.to_owned(),
                parameters: KUI_PARAMETERS,
                pos: actor.pos,
                angle: actor.angle,
                set_id: 0,
                room: actor.room,
                ..Default::default()
            },
            super::kui::Kui::spawn(KUI_PARAMETERS),
        ));
    }

    fn execute(
        &mut self,
        actor: &mut Actor,
        env: &mut Env,
        btd: Option<BtdProbe>,
        kui: Option<(Vec3, i32)>,
        sink: &mut Sink<'_>,
    ) -> bool {
        self.tick = self.tick.wrapping_add(1);
        for slot in self.hold.iter_mut() {
            if *slot != 0 {
                *slot -= 1;
            }
        }
        self.move_tail(actor, env, kui);
        let struck = self.iwa_move(actor, btd, sink);
        self.dr_move();
        self.yuka_move(env);
        struck
    }

    fn move_tail(&mut self, actor: &mut Actor, env: &mut Env, kui: Option<(Vec3, i32)>) {
        if self.mode == MODE_RATTLE {
            let tick = i32::from(self.tick);
            for slot in JOINT_WAVE_FIRST..COUNT_JOINT {
                let idx = slot as i32;
                let sy = from_s16(
                    tick.wrapping_mul(WAVE_Y_MUL)
                        .wrapping_add(idx.wrapping_mul(WAVE_Y_STEP)) as i16,
                );
                let cz = from_s16(
                    tick.wrapping_mul(WAVE_Z_MUL)
                        .wrapping_add(idx.wrapping_mul(WAVE_Z_STEP)) as i16,
                );
                self.wave_y[slot] = (self.rattle * sy.sin()) as i32 as i16;
                self.wave_z[slot] = (self.rattle * cz.cos()) as i32 as i16;
            }
        } else {
            self.mode = MODE_REST;
            match kui {
                Some((kui_pos, 3)) => {
                    let delta = env.player_pos + Vec3::new(0.0, HAND_Y, 0.0) - kui_pos;
                    let rad = from_s16(env.player_shape[ANGLE_Y].wrapping_neg());
                    let (sin, cos) = rad.sin_cos();
                    let turned = Vec3::new(
                        delta.x * cos + delta.z * sin,
                        delta.y,
                        -delta.x * sin + delta.z * cos,
                    );
                    let atan = atan2s(turned.z, -turned.y);
                    actor.angle[ANGLE_Y] = env.player_shape[ANGLE_Y];
                    let atan2 = ((atan as i32 as f32) * (REG0_F3 + AIM_TRACK)) as i32 as i16;
                    add_calc_angle_s2(&mut self.wave_y[0], atan2, AIM_SCALE, AIM_MAX);
                    add_calc_angle_s2(&mut self.wave_z[0], 0, AIM_SCALE, AIM_MAX);
                    self.mode = MODE_AIM;
                }
                Some((_, 2)) => {
                    self.mode = MODE_SCALE;
                    actor.angle[ANGLE_Y] = bearing(actor.pos, env.player_pos);
                    let flip = actor.angle[ANGLE_Y].wrapping_sub(-0x8000);
                    env.player_shape[ANGLE_Y] = flip;
                    env.player_angle[ANGLE_Y] = flip;
                    add_calc2(
                        &mut self.tilt_y,
                        REG0_F9 + SWAY_TILT,
                        SWAY_TILT_SCALE,
                        REG0_F10 + SWAY_TILT_STEP,
                    );
                    add_calc0(&mut self.tilt_z, SWAY_Z_TILT_SCALE, SWAY_Z_TILT_MAX);
                    if self.tilt_y < TILT_LIMIT {
                        add_calc2(
                            &mut self.scale_x,
                            STRETCH_TARGET,
                            STRETCH_SCALE,
                            REG0_F14 + STRETCH_STEP,
                        );
                    }
                }
                _ => {
                    self.scale_x = REST_SCALE;
                    actor.angle[ANGLE_Y] = self.home_angle;
                    self.phase_y += SWAY_Y_STEP;
                    if self.phase_y > TAU_F {
                        self.tilt_y -= TAU_F;
                    }
                    self.phase_z += SWAY_Z_STEP;
                    if self.phase_z > TAU_F {
                        self.tilt_z -= TAU_F;
                    }
                    self.tilt_y = self.amp * self.phase_y.sin();
                    self.tilt_z = self.amp * self.phase_z.sin();
                    self.hane_l2 = self.hane_l2_at_hinge(env.player_shape[ANGLE_Y]);
                    if kui.map(|(_, health)| health) == Some(1) {
                        add_calc2(&mut self.amp, 0.0, SWAY_TILT_SCALE, SWAY_EPS);
                        self.spin_timer = HOLD_TICKS;
                    } else {
                        add_calc2(&mut self.amp, SWAY_AMP, SWAY_TILT_SCALE, SWAY_EPS);
                    }
                }
            }
        }
    }

    fn iwa_move(
        &mut self,
        actor: &mut Actor,
        mut btd: Option<BtdProbe>,
        sink: &mut Sink<'_>,
    ) -> bool {
        let mut struck = false;
        if self.hold_timer != 0 {
            self.hold_timer -= 1;
        }
        match self.state {
            STATE_RISE => {
                if (actor.pos.y - self.home.y).abs() > DROP_RANGE {
                    self.amp = 0.0;
                }
                if self.rise_timer == 0 {
                    add_calc2(&mut actor.pos.y, self.home.y, RISE_SCALE, RISE_MAX);
                    if (actor.pos.y - self.home.y).abs() < DROP_EPS {
                        self.state = STATE_IDLE;
                    }
                } else {
                    self.rise_timer -= 1;
                }
                self.state_idle(sink);
            }
            STATE_IDLE => {
                self.state_idle(sink);
            }
            STATE_FALL => {
                self.rock_pos.y += actor.speed.y;
                actor.speed.y -= REG0_F2 + GRAVITY;
                if let Some(probe) = btd.as_mut() {
                    let floor = probe.ground.y + REG0_F3;
                    if self.rock_pos.y < floor {
                        self.rock_pos.y = floor;
                        self.state = STATE_ATTACH;
                        self.hold[0] = HOLD_JOINT;
                        struck = true;
                        sink.se(SE_BTD_ROCK_HIT);
                    }
                }
            }
            STATE_ATTACH => {
                self.mode = MODE_RATTLE;
                add_calc2(
                    &mut self.rattle,
                    RATTLE_TARGET,
                    RATTLE_SCALE,
                    RATTLE_MAX_STEP,
                );
                if let Some(probe) = btd.as_mut() {
                    self.rock_pos.y = probe.ground.y + REG0_F3;
                    self.rock_ang[ANGLE_Y] = probe.angle;
                    if probe.ground.y < REG0_F6 + TOP_BOTTOM {
                        self.lift = (REG0_F6 + TOP_BOTTOM) - probe.ground.y;
                        self.lift *= REG0_F7 + TOP_FILL;
                    }
                    let mut aim = probe.ground - probe.far;
                    aim.y += REG0_F5 + ATTACH_PITCH + self.lift;
                    let target =
                        atan2s(aim.y, (aim.x * aim.x + aim.z * aim.z).sqrt()).wrapping_neg();
                    add_calc_angle_s2(
                        &mut self.rock_ang[ANGLE_X],
                        target,
                        ATTACH_SCALE,
                        ATTACH_MAX,
                    );
                    if !probe.down {
                        self.state = STATE_LIFT;
                    }
                }
            }
            STATE_LIFT => {
                if let Some(probe) = btd.as_mut() {
                    self.rock_pos.y = probe.ledge.y + LIFT_SIT + REG0_F9;
                    add_calc_angle_s2(&mut self.rock_ang[ANGLE_X], 0, SPIN_SCALE, SPIN_MAX);
                    add_calc_angle_s2(&mut self.rock_ang[ANGLE_Y], 0, SPIN_SCALE, SPIN_MAX);
                    if self.rock_pos.y > self.home.y - LIFT_LEAD {
                        actor.pos.y += REG0_F9 + LIFT_RISE;
                        self.mode = MODE_REST;
                    }
                    if self.rock_pos.y > self.home.y - TOP_OFFSET {
                        self.rock_pos.y = self.home.y - TOP_OFFSET;
                        self.state = STATE_RISE;
                        self.rise_timer = RETURN_TICKS;
                        actor.speed.y = 0.0;
                        self.lift = 0.0;
                        self.rate = REG0_S2.wrapping_add(DROP_RATE) as u8;
                        self.emit_mode = EMIT_SPAWN;
                        self.emit_hold = REST_TICKS;
                        sink.se(SE_BTD_ROCK_ATTACH);
                    }
                }
            }
            STATE_SHATTER => {
                self.state = STATE_SHATTER_DONE;
                let rad = from_s16(self.rock_ang[ANGLE_Y]);
                let (sin, cos) = rad.sin_cos();
                for idx in 0..COUNT_HAHEN {
                    if self.fragments[idx].live {
                        continue;
                    }
                    self.fragments[idx].live = true;
                    self.fragments[idx].pos = self.rock_pos;
                    self.fragments[idx].rot = self.rock_ang;
                    let along = (REG0_F2 + SHATTER_SPREAD) * HSX[idx];
                    let across = (REG0_F2 + SHATTER_SPREAD) * HSZ[idx];
                    self.fragments[idx].vel =
                        Vec3::new(along * cos + across * sin, 0.0, across * cos - along * sin);
                }
                self.state_shatter_done();
            }
            STATE_SHATTER_DONE => {
                self.state_shatter_done();
            }
            _ => {}
        }
        self.emit_switch();
        self.hahen_move();
        struck
    }

    fn state_idle(&mut self, sink: &mut Sink<'_>) {
        add_calc_angle_s2(&mut self.rock_ang[ANGLE_X], 0, SPIN_SCALE, SPIN_MAX);
        add_calc_angle_s2(&mut self.rock_ang[ANGLE_Y], 0, SPIN_SCALE, SPIN_MAX);
        if self.mode == MODE_SCALE || self.hold_timer != 0 {
            if self.spin_timer == 0 {
                self.spin_timer = REG0_S5.wrapping_add(HILL_RATE);
                self.spin_ang = self.rnd_fx(REG0_F7 + HILL_SCALE) as i32 as i16;
                self.spin_rad = self.rnd_fx(REG0_F6 + HILL_OFF);
                if !self.has_emitter() {
                    self.rate = REG0_S5.wrapping_add(HOLD_RATE) as u8;
                    self.emit_mode = EMIT_SPAWN;
                    self.emit_hold = BURN_TICKS;
                    sink.se(SE_BTD_STN_FALL);
                }
            } else {
                self.spin_timer -= 1;
            }
        } else {
            self.spin_ang = 0;
            self.spin_rad = 0.0;
        }
        add_calc_angle_s2(
            &mut self.rock_ang[ANGLE_Z],
            self.spin_ang,
            SPIN_SCALE,
            REG0_S6.wrapping_add(SPIN_MAX),
        );
        add_calc2(
            &mut self.rock_pos.y,
            (self.spin_rad + self.home.y) - TOP_OFFSET,
            TRACK_SCALE,
            TRACK_MAX,
        );
        if self.hold_timer == HOLD_RISE {
            self.state = STATE_FALL;
            self.rate = REG0_S2.wrapping_add(FALL_RATE) as u8;
            self.emit_mode = EMIT_SPAWN;
            self.emit_hold = DROP_TICKS;
            sink.se(SE_BTD_ROCK_FALL);
        }
    }

    fn has_emitter(&self) -> bool {
        self.emit_mode != EMIT_NONE
    }

    fn hane_l2_at_hinge(&mut self, shape_y: i16) -> Vec3 {
        let rad = from_s16(shape_y.wrapping_neg());
        let (sin, cos) = rad.sin_cos();
        let local = Vec3::new(215.0, 0.0, -5.0);
        Vec3::new(
            local.x * cos + local.z * sin,
            local.y,
            -local.x * sin + local.z * cos,
        )
    }

    fn state_shatter_done(&mut self) {
        self.mode = MODE_REST;
    }

    fn emit_switch(&mut self) {
        match self.emit_mode {
            EMIT_SPAWN => {
                if self.emitter {
                    self.emitter_delete = true;
                }
                self.emitter = true;
                self.emitter_delete = false;
                self.emit_mode = EMIT_RUN;
            }
            EMIT_RUN => {
                if self.emit_hold != 0 {
                    self.emit_hold -= 1;
                    if self.emit_hold == 0 && self.emitter {
                        self.emitter_delete = true;
                    }
                }
                if self.emitter && self.emitter_delete {
                    self.emit_mode = EMIT_NONE;
                    self.emitter = false;
                }
            }
            _ => {}
        }
    }

    fn hahen_move(&mut self) {
        for idx in 0..COUNT_HAHEN {
            if !self.fragments[idx].live {
                continue;
            }
            self.fragments[idx].pos += self.fragments[idx].vel;
            if idx & 1 != 0 {
                self.fragments[idx].rot[ANGLE_Z] = self.fragments[idx].rot[ANGLE_Z]
                    .wrapping_add(REG0_S4.wrapping_add(SHATTER_TUMBLE));
                self.fragments[idx].rot[ANGLE_X] = self.fragments[idx].rot[ANGLE_X]
                    .wrapping_add(REG0_S3.wrapping_add(SHATTER_DRIFT));
            } else {
                self.fragments[idx].rot[ANGLE_Z] = self.fragments[idx].rot[ANGLE_Z]
                    .wrapping_sub(REG0_S4.wrapping_add(SHATTER_TUMBLE));
                self.fragments[idx].rot[ANGLE_X] = self.fragments[idx].rot[ANGLE_X]
                    .wrapping_sub(REG0_S3.wrapping_add(SHATTER_DRIFT));
            }
            self.fragments[idx].vel.y -= REG0_F7 + FALL_GRAVITY;
            if self.fragments[idx].pos.y < DEATH_Y {
                self.fragments[idx].live = false;
            }
        }
    }

    fn dr_move(&mut self) {
        if self.dr_timer != 0 {
            self.dr_timer -= 1;
        }
        match self.dr_state {
            0 => {
                if self.dr_live {
                    self.dr_timer = 0;
                    self.dr_state = 1;
                }
            }
            1 => {
                if !self.dr_live {
                    self.dr_state = 0;
                }
            }
            _ => {}
        }
    }

    fn yuka_move(&mut self, env: &Env) {
        if env.stage_boss_enemy && env.start_stage_first != b'X' {
            self.mbyo_broken = true;
        }
    }
}
