use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::events::{SimEvent, SimEvents};
use super::flags::Switches;
use super::player::Player;
use super::sea::Sea;
use super::ship::Ship;
use super::stream;
use super::walk;
use super::water;

pub const DAIOCTA_ACTORS: [&str; 1] = ["Daiocta"];

const NO_SWITCH: u8 = 0xFF;
const PARM_APPEAR_SHIFT: u32 = 8;
const PARM_APPEAR_MASK: u32 = 0xFF;
const PARM_SWITCH_SHIFT: u32 = 24;
const PARM_SWITCH_MASK: u32 = 0xFF;
const APPEAR_MULT_MAX: u8 = 0xFF;
const APPEAR_STEP: f32 = 100.0;

const HIO_DEFAULT_APPEAR_RADIUS: f32 = 5000.0;
const HIO_MIN_APPEAR_RADIUS: f32 = 4500.0;
const HIO_M010: i16 = 0x100;
const HIO_M012: u16 = 0x1500;
const HIO_M014: i16 = -0x800;
const HIO_M080: f32 = 30.0;
const HIO_M084: f32 = 1800.0;
const HIDE_DEPTH: f32 = 3000.0;
const ENTER_RANGE: f32 = 1900.0;
const SETTLE_RANGE: f32 = 0.1;
const APPEAR_TURN_SCALE: i16 = 4;
const RISE_SCALE: f32 = 0.1;
const NEAR_ZERO_SQUARE: f32 = 8e-11;

const PRM_SETTLE: i8 = 2;
const PRM_INIT: i8 = 1;
const PRM_DAMAGE_BOMB: i8 = 3;
const PRM_DAMAGE: i8 = 4;

const CULL_MIN: Vec3 = Vec3::new(-2300.0, -500.0, -2300.0);
const CULL_MAX: Vec3 = Vec3::new(2300.0, 2300.0, 2300.0);

const SE_LK_LAST_HIT: u32 = 0x2828;
const SE_CM_DO_ENTER: u32 = 0x58C2;
const SE_CV_DO_DIE: u32 = 0x48C9;
const SE_CM_DO_JITABATA: u32 = 0x58C5;
const BGM_DIOCTA_BATTLE: u32 = 0x8000002B;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Hide,
    Appear,
    Wait,
    Damage,
    DamageBomb,
    Delete,
    Demo,
    Null,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Proc {
    Init,
    Exec,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Daiocta {
    pub mode: Mode,
    pub prm_idx: i8,
    pub appear_radius: f32,
    pub water_y: f32,
    pub battle: bool,
    pub placed: bool,
}

fn flat_distance(from: Vec3, to: Vec3) -> f32 {
    let delta = to - from;
    (delta.x * delta.x + delta.z * delta.z).sqrt()
}

fn bearing(from: Vec3, to: Vec3) -> i16 {
    walk::to_s16((to.x - from.x).atan2(to.z - from.z))
}

fn norm_zp(vector: Vec3) -> Vec3 {
    if vector.length_squared() < NEAR_ZERO_SQUARE {
        Vec3::ZERO
    } else {
        vector.normalize()
    }
}

fn add_calc_pos2(value: &mut Vec3, target: Vec3, scale: f32, max_step: f32) {
    if *value != target {
        let mut diff = (*value - target) * scale;
        if diff.length() > max_step {
            diff = norm_zp(diff) * max_step;
        }
        *value -= diff;
    }
}

impl Daiocta {
    pub fn spawn(parameters: u32, switches: &Switches) -> Option<Self> {
        let switch_no = ((parameters >> PARM_SWITCH_SHIFT) & PARM_SWITCH_MASK) as u8;
        if switch_no != NO_SWITCH && switches.test(switch_no) {
            return None;
        }
        let multiplier = ((parameters >> PARM_APPEAR_SHIFT) & PARM_APPEAR_MASK) as u8;
        let appear_radius = if multiplier == APPEAR_MULT_MAX {
            HIO_DEFAULT_APPEAR_RADIUS
        } else {
            f32::from(multiplier) * APPEAR_STEP
        };
        Some(Self {
            mode: Mode::Hide,
            prm_idx: PRM_INIT,
            appear_radius,
            water_y: 0.0,
            battle: false,
            placed: false,
        })
    }

    pub fn cull(&self) -> (Vec3, Vec3) {
        (CULL_MIN, CULL_MAX)
    }

    fn create_init(&mut self, actor: &mut Actor, events: &mut SimEvents) {
        self.placed = true;
        self.prm_idx = PRM_INIT;
        self.mode_proc(Proc::Init, Mode::Hide, actor, None, false, events);
    }

    fn set_water(&mut self, actor: &Actor, sea: Option<&Sea>) {
        let (wet, surface) = water::probe(sea, actor.pos.x, actor.pos.z);
        self.water_y = if wet { surface } else { 0.0 };
    }

    fn jump_to_water(&mut self, actor: &mut Actor) -> f32 {
        let mut surface = actor.pos;
        surface.y = self.water_y;
        add_calc_pos2(&mut actor.pos, surface, RISE_SCALE, HIO_M080);
        (actor.pos - surface).length()
    }

    fn mode_hide_init(&mut self, actor: &mut Actor) {
        actor.pos.y = self.water_y - HIDE_DEPTH;
    }

    fn mode_hide(
        &mut self,
        actor: &mut Actor,
        player: Option<&Player>,
        riding: bool,
        events: &mut SimEvents,
    ) {
        let Some(player) = player else {
            return;
        };
        let range = flat_distance(actor.pos, player.pos);
        if (range < self.appear_radius || range < HIO_MIN_APPEAR_RADIUS) && riding {
            self.mode_proc(Proc::Init, Mode::Appear, actor, Some(player), riding, events);
        }
    }

    fn mode_appear_init(&mut self, actor: &mut Actor, player: Option<&Player>) {
        let Some(player) = player else {
            return;
        };
        let outward = norm_zp(actor.pos - player.pos) * HIO_MIN_APPEAR_RADIUS;
        actor.pos.x = player.pos.x + outward.x;
        actor.pos.z = player.pos.z + outward.z;
    }

    fn mode_appear(
        &mut self,
        actor: &mut Actor,
        player: Option<&Player>,
        riding: bool,
        events: &mut SimEvents,
    ) {
        let gap = self.jump_to_water(actor);
        if gap < ENTER_RANGE && !self.battle {
            events.list.push(SimEvent::Se(SE_CM_DO_ENTER));
            events.list.push(SimEvent::Bgm(BGM_DIOCTA_BATTLE));
            self.battle = true;
        }
        if gap <= SETTLE_RANGE {
            self.mode_proc(Proc::Init, Mode::Wait, actor, player, riding, events);
        } else if let Some(player) = player {
            stream::add_calc_angle_s2(
                &mut actor.angle[1],
                bearing(actor.pos, player.pos),
                APPEAR_TURN_SCALE,
                HIO_M010,
            );
        }
    }

    fn mode_wait_init(&mut self) {}

    fn mode_wait(&mut self, actor: &mut Actor) {
        actor.pos.y = self.water_y;
    }

    fn mode_damage_init(&mut self) {
        self.prm_idx = PRM_DAMAGE;
    }

    fn mode_damage(
        &mut self,
        actor: &mut Actor,
        player: Option<&Player>,
        riding: bool,
        events: &mut SimEvents,
    ) {
        actor.pos.y = self.water_y;
        if self.prm_idx == PRM_SETTLE {
            self.jump_to_water(actor);
            self.recover(actor, player, riding, events);
        }
    }

    fn mode_damage_bomb_init(&mut self, events: &mut SimEvents) {
        events.list.push(SimEvent::Se(SE_LK_LAST_HIT));
        self.prm_idx = PRM_DAMAGE_BOMB;
    }

    fn mode_damage_bomb(
        &mut self,
        actor: &mut Actor,
        player: Option<&Player>,
        riding: bool,
        events: &mut SimEvents,
    ) {
        actor.pos.y = self.water_y;
        if self.prm_idx == PRM_SETTLE {
            self.jump_to_water(actor);
            self.recover(actor, player, riding, events);
        }
    }

    fn recover(
        &mut self,
        actor: &mut Actor,
        player: Option<&Player>,
        riding: bool,
        events: &mut SimEvents,
    ) {
        if (actor.pos.y - self.water_y).abs() <= SETTLE_RANGE {
            self.mode_proc(Proc::Init, Mode::Wait, actor, player, riding, events);
        } else {
            self.mode_proc(Proc::Init, Mode::Appear, actor, player, riding, events);
        }
    }

    fn mode_demo_init(&mut self) {}

    fn mode_demo(&mut self, actor: &mut Actor) {
        actor.pos.y = self.water_y;
    }

    fn mode_delete_init(&mut self, events: &mut SimEvents) {
        events.list.push(SimEvent::Se(SE_CV_DO_DIE));
        events.list.push(SimEvent::Se(SE_CM_DO_JITABATA));
        events.list.push(SimEvent::Se(SE_LK_LAST_HIT));
    }

    fn mode_delete(&mut self) {}

    fn mode_proc(
        &mut self,
        proc: Proc,
        mode: Mode,
        actor: &mut Actor,
        player: Option<&Player>,
        riding: bool,
        events: &mut SimEvents,
    ) {
        if proc == Proc::Init {
            self.mode = mode;
            match mode {
                Mode::Hide => self.mode_hide_init(actor),
                Mode::Appear => self.mode_appear_init(actor, player),
                Mode::Wait => self.mode_wait_init(),
                Mode::Damage => self.mode_damage_init(),
                Mode::DamageBomb => self.mode_damage_bomb_init(events),
                Mode::Delete => self.mode_delete_init(events),
                Mode::Demo => self.mode_demo_init(),
                Mode::Null => {}
            }
            return;
        }
        match self.mode {
            Mode::Hide => self.mode_hide(actor, player, riding, events),
            Mode::Appear => self.mode_appear(actor, player, riding, events),
            Mode::Wait => self.mode_wait(actor),
            Mode::Damage => self.mode_damage(actor, player, riding, events),
            Mode::DamageBomb => self.mode_damage_bomb(actor, player, riding, events),
            Mode::Delete => self.mode_delete(),
            Mode::Demo => self.mode_demo(actor),
            Mode::Null => {}
        }
    }

    fn execute(
        &mut self,
        actor: &mut Actor,
        player: Option<&Player>,
        sea: Option<&Sea>,
        riding: bool,
        events: &mut SimEvents,
    ) {
        if self.mode != Mode::Hide && self.mode != Mode::Appear {
            if let Some(player) = player {
                let range = flat_distance(actor.pos, player.pos);
                let error = actor.angle[1]
                    .wrapping_add(HIO_M014)
                    .wrapping_sub(bearing(actor.pos, player.pos))
                    .unsigned_abs();
                if range < HIO_M084
                    && self.mode != Mode::Demo
                    && self.mode != Mode::Delete
                    && error < HIO_M012
                {
                    self.mode_proc(Proc::Init, Mode::Demo, actor, Some(player), riding, events);
                }
            }
        }
        if self.mode != Mode::Hide {
            self.set_water(actor, sea);
        }
        self.mode_proc(Proc::Exec, Mode::Null, actor, player, riding, events);
    }
}

pub fn is_daiocta(name: &str) -> bool {
    DAIOCTA_ACTORS.contains(&name)
}

pub fn control(
    sea: Option<Res<Sea>>,
    players: Query<&Player>,
    ships: Query<&Ship>,
    mut events: ResMut<SimEvents>,
    mut octos: Query<(&mut Actor, &mut Daiocta)>,
) {
    let sea = sea.as_deref();
    let player = players.iter().next().copied();
    let riding = ships.iter().any(|ship| ship.riding);
    for (mut actor, mut octo) in &mut octos {
        let actor = &mut *actor;
        let octo = &mut *octo;
        if !octo.placed {
            octo.create_init(actor, &mut events);
        }
        octo.execute(actor, player.as_ref(), sea, riding, &mut events);
    }
}