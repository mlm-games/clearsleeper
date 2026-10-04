use bevy_ecs::prelude::*;

use glam::Vec3;

use super::actor::Actor;
use super::clock::Clock;
use super::flags::{Charts, EventReg, GHOST_SHIP_DONE, REG_GHOST_SHIP};
use super::player::Player;

pub const GHOSTSHIP_ACTOR: &str = "Ayush";

pub const NO_PATH: u8 = 0xFF;
pub const PATH_SPEED: f32 = 10.0;
pub const SHIP_ALPHA: f32 = 0.51;
pub const ALPHA_STEP: f32 = 0.02;
pub const HIDE_DISTANCE: f32 = 3000.0;
pub const ENTER_DISTANCE: f32 = 1500.0;
pub const SEA_FROM: f32 = 90.0;
pub const SEA_TO: f32 = 285.0;
pub const CHART_SLOT: usize = 35;
pub const HOVER_HEIGHT: f32 = 600.0;
pub const LOOP_RADIUS: f32 = 600.0;
pub const LOOP_OFFSET: f32 = 500.0;
pub const WOBBLE: f32 = 300.0;
pub const RING_STEP: f32 = 100.0;
pub const RINGS: usize = 12;
pub const RING_SPEED: f32 = 0.1;
pub const RING_MIN: f32 = 10.0;
pub const RING_FLOOR: f32 = 900.0;
pub const RING_NEAR: f32 = 800.0;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Wait,
    PathMove,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Ghostship {
    pub moon: u8,
    pub path_no: u8,
    pub mode: Mode,
    pub alpha: f32,
    pub speed: f32,
    pub heading: Vec3,
    pub entered: bool,
    pub rings: [f32; RINGS],
    pub ring_speed: [f32; RINGS],
    pub ring_spin: [f32; RINGS],
}

impl Ghostship {
    pub fn spawn(parameters: u32, yaw: f32) -> Self {
        let mut ship = Self {
            moon: (parameters & 0xFF) as u8,
            path_no: ((parameters >> 8) & 0xFF) as u8,
            mode: Mode::Wait,
            alpha: 0.0,
            speed: 0.0,
            heading: Vec3::new(0.0, 0.0, yaw.cos()).normalize_or_zero(),
            entered: false,
            rings: [0.0; RINGS],
            ring_speed: [0.0; RINGS],
            ring_spin: [0.0; RINGS],
        };
        for ring in 0..RINGS {
            ship.rings[ring] = RING_STEP * ring as f32;
            ship.ring_speed[ring] = if ring % 2 == 0 { 1.0 } else { -1.0 };
            ship.ring_spin[ring] = 8192.0 * ring as f32;
        }
        if ship.path_no != NO_PATH {
            ship.mode = Mode::PathMove;
        }
        ship
    }

    pub fn visible(&self, clock: &Clock) -> bool {
        if u32::from(self.moon) != clock.moon() {
            return false;
        }
        !(clock.time > SEA_FROM && clock.time < SEA_TO)
    }

    pub fn target_alpha(&self, distance: f32, charts: &Charts) -> f32 {
        if charts.is_open(CHART_SLOT) && charts.has(CHART_SLOT) {
            return SHIP_ALPHA;
        }
        if distance < HIDE_DISTANCE {
            0.0
        } else {
            SHIP_ALPHA
        }
    }

    pub fn can_board(&self, distance: f32, charts: &Charts, reg: &EventReg) -> bool {
        self.alpha == SHIP_ALPHA
            && distance < ENTER_DISTANCE
            && reg.get(REG_GHOST_SHIP) < GHOST_SHIP_DONE
            && !self.entered
            && charts.is_open(CHART_SLOT)
            && charts.has(CHART_SLOT)
    }

    pub fn rings(&mut self) {
        for ring in 0..RINGS {
            let radius = &mut self.rings[ring];
            let target = if *radius < RING_FLOOR {
                RING_STEP * ring as f32 + 1500.0
            } else if *radius > RING_STEP * ring as f32 + 1400.0 {
                RING_NEAR
            } else {
                *radius
            };
            let gap = target - *radius;
            if gap.abs() > RING_MIN {
                *radius += gap.signum() * RING_SPEED.min(gap.abs());
            }
        }
    }

    pub fn hover(&self, wave: f32) -> f32 {
        HOVER_HEIGHT + LOOP_RADIUS * wave
    }
}

pub fn control(
    clock: Res<Clock>,
    mut charts: ResMut<Charts>,
    regs: Res<EventReg>,
    mut ships: Query<(&Actor, &mut Ghostship)>,
    players: Query<&Player>,
) {
    if regs.get(REG_GHOST_SHIP) >= GHOST_SHIP_DONE {
        return;
    }
    let home = players.iter().next().map(|player| player.pos);
    for (actor, mut ship) in &mut ships {
        let range = home.map_or(ENTER_DISTANCE, |player| reach(player, actor.pos));
        if !ship.visible(&clock) {
            ship.alpha = 0.0;
            continue;
        }
        let target = ship.target_alpha(range, &charts);
        ship.alpha = approach(ship.alpha, target, ALPHA_STEP);
        if ship.alpha > 0.0 {
            ship.rings();
        }
        if ship.can_board(range, &charts, &regs) {
            ship.entered = true;
            charts.give(CHART_SLOT);
            ship.alpha = SHIP_ALPHA;
        }
    }
}

fn reach(from: Vec3, to: Vec3) -> f32 {
    let delta = from - to;
    delta.x * delta.x + delta.z * delta.z
}

fn approach(current: f32, target: f32, step: f32) -> f32 {
    if (target - current).abs() <= step {
        target
    } else {
        current + (target - current).signum() * step
    }
}