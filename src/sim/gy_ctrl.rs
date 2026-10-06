use bevy_ecs::prelude::*;

use super::actor::Actor;
use super::flags::Switches;
use super::kamome::Kamome;
use super::lines::Lines;
use super::player::Player;
use super::player_status::{STTS0_SHIP_RIDE, STTS0_SWIM, PlayerStatus};
use super::sea::Sea;
use super::ship::Ship;
use super::stream::sin_s;

pub const GY_CTRL_ACTORS: [&str; 1] = ["GyCtrlB"];

pub fn is_gy_ctrl(name: &str) -> bool {
    GY_CTRL_ACTORS.contains(&name)
}

pub const FLOCK_MAX: usize = 5;
pub const PATH_STEP: i16 = 0xFFF;
pub const WATER_LIFT: f32 = 1000.0;
pub const NEAR_ACTOR_RANGE: f32 = 6000.0;
pub const PATH_SWAY: f32 = 10.0;
pub const PATH_SWAY_OFFSET: f32 = -500.0;
pub const PATH_HEIGHT_OFFSET: f32 = 500.0;
pub const PATROL_SPEED: f32 = 700.0;
pub const BOAT_SPEED: f32 = 1000.0;
pub const PLAYER_SPEED: f32 = 700.0;
pub const STEP_MAX: f32 = 10.0;
pub const TARGET_TICK: i32 = 0x1e;
pub const BASE_STEP: f32 = 0.1;
pub const HIDE_RETRY: i32 = 39;
pub const CLEAR_TIMER: i32 = 0;
pub const PHASE_FOLLOW: f32 = 0.0;
pub const TARGET_NONE: i32 = 0;
pub const TARGET_SWIM: i32 = 1;
pub const TARGET_BOAT: i32 = 2;

#[derive(Resource, Debug, Clone, Copy)]
pub struct GyTuning {
    pub m06: i16,
    pub m08: i16,
    pub m0a: i16,
    pub m0c: i16,
    pub m10: f32,
    pub m18: f32,
    pub m1c: f32,
    pub m24: f32,
    pub m28: f32,
    pub m2c: f32,
}

impl Default for GyTuning {
    fn default() -> Self {
        Self {
            m06: 100,
            m08: 240,
            m0a: 200,
            m0c: 250,
            m10: 10.0,
            m18: 1000.0,
            m1c: 700.0,
            m24: 500.0,
            m28: -500.0,
            m2c: 10.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Mode {
    SwWait,
    #[default]
    Create,
    Wait,
    Hide,
}

#[derive(Component, Debug, Clone)]
pub struct GyCtrl {
    pub mode: Mode,
    pub use_area: u8,
    pub count: u8,
    pub switch_no: u8,
    pub exists: u8,
    pub room_no: i32,
    pub base_room: u8,
    pub near_radius: f32,
    pub patrol_speed: f32,
    pub base_angle: i16,
    pub sway: f32,
    pub sway_step: f32,
    pub spawned: usize,
    pub angles: [i16; FLOCK_MAX],
    pub path: [glam::Vec3; FLOCK_MAX],
    pub ahead: [glam::Vec3; FLOCK_MAX],
    pub blocked: [bool; FLOCK_MAX],
    pub alive: [bool; FLOCK_MAX],
    pub reached: bool,
    pub clear: bool,
    pub target: i32,
    pub timer: i32,
    pub hide_timer: i32,
    pub first: glam::Vec3,
}

impl Default for GyCtrl {
    fn default() -> Self {
        Self {
            mode: Mode::SwWait,
            use_area: 0,
            count: 0,
            switch_no: 0xFF,
            exists: 0,
            room_no: -1,
            base_room: 0,
            near_radius: 0.0,
            patrol_speed: PHASE_FOLLOW,
            base_angle: 0,
            sway: 0.0,
            sway_step: 0.0,
            spawned: 0,
            angles: [0; FLOCK_MAX],
            path: [glam::Vec3::ZERO; FLOCK_MAX],
            ahead: [glam::Vec3::ZERO; FLOCK_MAX],
            blocked: [false; FLOCK_MAX],
            alive: [false; FLOCK_MAX],
            reached: false,
            clear: false,
            target: TARGET_NONE,
            timer: 0,
            hide_timer: 0,
            first: glam::Vec3::ZERO,
        }
    }
}

impl GyCtrl {
    pub fn spawn(parameters: u32, angle_x: i16, room: i8) -> Self {
        let mut tag = Self::default();
        tag.use_area = bit_field(parameters, 0, 4);
        if tag.use_area == 0x0F {
            tag.use_area = 0;
        }
        tag.count = bit_field(parameters, 4, 4);
        if tag.count == 0x0F {
            tag.count = 1;
        }
        let radius = bit_field(parameters, 8, 8);
        tag.near_radius = if radius == 0xFF {
            10000.0
        } else {
            f32::from(radius) * 1000.0
        };
        tag.switch_no = bit_field(parameters, 0x18, 8);
        tag.base_room = (angle_x & 0xFF) as u8;
        tag.room_no = i32::from(room);
        tag
    }

    pub fn mode_proc(&mut self, mode: Mode) {
        self.mode = mode;
        if mode == Mode::Create {
            self.timer = 0;
        }
        if mode == Mode::Hide {
            self.hide_timer = HIDE_RETRY;
        }
    }

    pub fn target_player_state(&mut self, status: &PlayerStatus, riding: bool) {
        if status.check0(STTS0_SHIP_RIDE) || riding {
            self.target = TARGET_BOAT;
        } else if status.check0(STTS0_SWIM) {
            self.target = TARGET_SWIM;
        } else {
            self.target = TARGET_NONE;
        }
        if !self.clear {
            self.target = TARGET_NONE;
        }
        if self.target != TARGET_NONE {
            if self.timer == CLEAR_TIMER {
                self.timer = TARGET_TICK;
            }
        } else {
            self.timer = TARGET_TICK;
        }
    }

    pub fn path_target(
        &mut self,
        tuning: &GyTuning,
        player: glam::Vec3,
        ship: Option<glam::Vec3>,
    ) -> Option<glam::Vec3> {
        let mut at = player;
        match self.target {
            1 => {
                if let Some(pos) = ship {
                    at = pos;
                    self.patrol_speed = tuning.m18;
                    self.base_angle = self.base_angle.wrapping_add(tuning.m0c);
                } else {
                    return None;
                }
            }
            0 => {
                self.patrol_speed = tuning.m1c;
                self.base_angle = self.base_angle.wrapping_add(tuning.m0a);
            }
            _ => return None,
        }
        if self.use_area == 0 {
            self.reached = true;
            return Some(at);
        }
        let flat = glam::Vec3::new(self.first.x - at.x, 0.0, self.first.z - at.z).length();
        if flat < self.near_radius {
            self.reached = true;
            return Some(at);
        }
        self.reached = false;
        Some(at)
    }

    pub fn water_y(&self, at: glam::Vec3, sea: Option<&Sea>) -> f32 {
        let lifted = glam::Vec3::new(at.x, at.y + WATER_LIFT, at.z);
        sea.map_or(0.0, |sea| sea.height_at(lifted.x, lifted.z))
    }

    pub fn set_path_pos(&mut self, tuning: &GyTuning, origin: glam::Vec3, sea: Option<&Sea>, solid: Option<&super::collision::Collision>) {
        let count = usize::from(self.count);
        let step = (0xFFFF_i32 / i32::from(self.count.max(1))) as i16;
        for index in 0..count.min(FLOCK_MAX) {
            let angle = self.base_angle.wrapping_add(step * index as i16);
            self.angles[index] = angle;
            let ahead_angle = angle.wrapping_add(0x500);
            let sway = tuning.m10 * sin_s(angle);
            let lateral = self.sway + sway;
            let point = rotate(origin, angle, lateral, 0.0);
            let mut dest = rotate(origin, ahead_angle, lateral, tuning.m28);
            self.path[index] = point;
            dest.y = self.water_y(dest, sea);
            self.ahead[index] = dest;
            let raised = glam::Vec3::new(dest.x, dest.y + tuning.m24, dest.z);
            let hit = Lines::cross(raised, dest, solid);
            self.blocked[index] = hit.is_some();
            if let Some(hit) = hit {
                self.ahead[index] = hit.point;
            }
        }
        self.sway_step = add_calc(self.sway_step, self.patrol_speed, BASE_STEP, tuning.m2c);
        self.sway = self.sway_step;
    }

    pub fn check_path(
        &self,
        tuning: &GyTuning,
        origin: glam::Vec3,
        event_running: bool,
        solid: Option<&super::collision::Collision>,
    ) -> bool {
        if event_running {
            return false;
        }
        for index in 0..FLOCK_MAX {
            let angle = PATH_STEP * index as i16;
            let dest = rotate(origin, angle, self.sway, tuning.m28);
            let raised = glam::Vec3::new(dest.x, dest.y + tuning.m24 + tuning.m28, dest.z);
            if Lines::cross(raised, dest, solid).is_some() {
                return false;
            }
        }
        true
    }
}

fn sin_cos(angle: i16) -> (f32, f32) {
    let radians = f32::from(angle) * (std::f32::consts::TAU / 65536.0);
    (radians.sin(), radians.cos())
}

fn rotate(origin: glam::Vec3, angle: i16, lateral: f32, forward: f32) -> glam::Vec3 {
    let (sin_y, cos_y) = sin_cos(angle);
    glam::Vec3::new(
        origin.x + forward * sin_y + lateral * cos_y,
        origin.y,
        origin.z + forward * cos_y - lateral * sin_y,
    )
}

fn bit_field(parameters: u32, shift: u32, width: u32) -> u8 {
    (((1u32 << width) - 1) & (parameters >> shift)) as u8
}

fn add_calc(current: f32, target: f32, scale: f32, max_step: f32) -> f32 {
    let diff = (target - current) * scale;
    let step = diff.clamp(-max_step, max_step);
    current + step
}

pub fn control(
    mut tags: Query<(&mut Actor, &mut GyCtrl)>,
    players: Query<&Player>,
    ships: Query<&Ship>,
    flock: Query<&Kamome>,
    status: Res<PlayerStatus>,
    tuning: Res<GyTuning>,
    sea: Option<Res<Sea>>,
    solid: Option<Res<super::collision::Collision>>,
    switches: Res<Switches>,
) {
    let Some(player) = players.iter().next().copied() else {
        return;
    };
    let riding = ships.iter().any(|ship| ship.riding);
    let ship_pos = ships.iter().next().map(|ship| ship.velocity(0));
    let sea_ref = sea.as_deref();
    let solid_ref = solid.as_deref();

    for (mut actor, mut tag) in tags.iter_mut() {
        tag.timer = (tag.timer - 1).max(0);
        tag.target_player_state(&status, riding);
        if let Some(at) = tag.path_target(&tuning, player.pos, ship_pos) {
            let reached = tag.reached;
            if reached {
                actor.pos = at;
            }
        }
        tag.clear = tag.check_path(&tuning, actor.pos, false, solid_ref);
        tag.set_path_pos(&tuning, actor.pos, sea_ref, solid_ref);

        let live = flock.iter().count();
        tag.spawned = live;
        for index in 0..FLOCK_MAX {
            tag.alive[index] = index < live;
        }

        match tag.mode {
            Mode::SwWait => {
                if tag.switch_no != 0xFF && switches.test(tag.switch_no) {
                    tag.mode_proc(Mode::Create);
                }
            }
            Mode::Create => {
                if (tag.use_area == 1 && tag.reached) || !tag.clear {
                    tag.mode_proc(Mode::Hide);
                } else if tag.spawned >= usize::from(tag.count) {
                    tag.mode_proc(Mode::Wait);
                }
            }
            Mode::Wait => {
                if (tag.use_area == 1 && tag.reached) || !tag.clear {
                    tag.mode_proc(Mode::Hide);
                }
            }
            Mode::Hide => {
                let ready = (tag.use_area == 0 && tag.target != TARGET_NONE)
                    || (tag.use_area == 1 && tag.reached && tag.target != TARGET_NONE);
                if ready && tag.clear {
                    tag.mode_proc(Mode::SwWait);
                }
            }
        }
        tag.room_no = i32::from(actor.room);
    }
}

pub fn init(world: &mut World) {
    world.init_resource::<GyTuning>();
}

pub fn spawn(world: &mut World, source: &crate::dzs::StageActor) -> Entity {
    let actor = Actor {
        name: source.name.clone(),
        parameters: source.parameters,
        pos: glam::Vec3::from_array(source.pos),
        angle: source.angle,
        set_id: source.set_id,
        room: source.room,
        ..Default::default()
    };
    let tag = GyCtrl::spawn(source.parameters, source.angle[0], source.room);
    world.spawn((actor, tag)).id()
}