use bevy_ecs::prelude::*;

use glam::Vec3;

use super::actor::Actor;
use super::clock::Clock;
use super::flags::{Charts, EventReg, GHOST_SHIP_DONE, REG_GHOST_SHIP};
use super::player::Player;

pub const GHOSTSHIP_ACTOR: &str = "Ayush";
pub const NO_PATH: u8 = 0xFF;

pub const SHIP_ALPHA: f32 = 0.51;
pub const ALPHA_STEP: f32 = 0.02;
pub const HIDE_DISTANCE: f32 = 3000.0;
pub const ENTER_DISTANCE: f32 = 1500.0;
pub const SEA_FROM: f32 = 90.0;
pub const SEA_TO: f32 = 285.0;
pub const CHART_SLOT: usize = 35;
pub const RING_STEP: f32 = 100.0;
pub const RINGS: usize = 12;
pub const RING_SPEED: f32 = 0.1;
pub const RING_MIN: f32 = 10.0;
pub const RING_FLOOR: f32 = 900.0;
pub const RING_NEAR: f32 = 800.0;
pub const RING_OUTER: f32 = 1500.0;
pub const RING_MARGIN: f32 = 100.0;
pub const RING_WOBBLE: f32 = 5.0;
pub const HOVER_HEIGHT: f32 = 600.0;
pub const LOOP_RADIUS: f32 = 600.0;
pub const FAR_OFF: f32 = f32::MAX;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Ghostship {
    pub moon: u8,
    pub path_no: u8,
    pub alpha: f32,
    pub entered: bool,
    pub rings: [f32; RINGS],
    pub ring_speed: [f32; RINGS],
    pub ring_spin: [f32; RINGS],
}

impl Ghostship {
    pub fn spawn(parameters: u32) -> Self {
        let mut ship = Self {
            moon: (parameters & 0xFF) as u8,
            path_no: ((parameters >> 8) & 0xFF) as u8,
            alpha: 0.0,
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
        if ship.path_no == NO_PATH {
            ship.entered = false;
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

    pub fn boarded(&mut self, charts: &mut Charts) {
        self.entered = true;
        charts.give(CHART_SLOT);
    }

    pub fn can_board(&self, distance: f32) -> bool {
        self.alpha == SHIP_ALPHA && distance < ENTER_DISTANCE && !self.entered
    }

    fn rings(&mut self) {
        for ring in 0..RINGS {
            let radius = &mut self.rings[ring];
            let target = if *radius < RING_FLOOR {
                RING_STEP * ring as f32 + RING_OUTER
            } else if *radius > RING_STEP * ring as f32 + RING_OUTER - RING_MARGIN {
                RING_NEAR
            } else {
                *radius
            };
            let gap = target - *radius;
            if gap.abs() > RING_MIN {
                *radius += gap.signum() * RING_SPEED.min(gap.abs());
            }
            self.ring_spin[ring] += self.ring_speed[ring] * RING_WOBBLE;
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
    if regs.reg(REG_GHOST_SHIP) >= GHOST_SHIP_DONE {
        return;
    }
    let home = players.iter().next().map(|player| player.pos);
    for (actor, mut ship) in &mut ships {
        let range = home.map_or(FAR_OFF, |player| distance(player, actor.pos));
        if !ship.visible(&clock) {
            ship.alpha = 0.0;
            continue;
        }
        let target = ship.target_alpha(range, &charts);
        ship.alpha = approach(ship.alpha, target, ALPHA_STEP);
        if ship.can_board(range) {
            ship.boarded(&mut charts);
        }
        if ship.alpha > 0.0 {
            ship.rings();
        }
    }
}

fn distance(from: Vec3, to: Vec3) -> f32 {
    let delta = from - to;
    delta.x.hypot(delta.z)
}

fn approach(current: f32, target: f32, step: f32) -> f32 {
    if (target - current).abs() <= step {
        target
    } else {
        current + (target - current).signum() * step
    }
}
