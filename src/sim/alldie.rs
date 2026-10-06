use bevy_ecs::prelude::*;

use super::actor::Actor;
use super::combat::HitPoints;
use super::flags::Switches;

pub const ALLDIE_ACTORS: [&str; 1] = ["ALLdie"];

const SWBIT_SHIFT: u32 = 8; // <- d_a_alldie.cpp:19 (fopAcM_GetParam(this) >> 0x8)
const SWBIT_MASK: u32 = 0xFF; // <- d_a_alldie.h:18 (getSwbit returns u8)
const TIMER_START: i16 = 65; // <- d_a_alldie.cpp:31
const ACT_WAIT: u8 = 0; // <- d_a_alldie.cpp:12
const ACT_CHECK: u8 = 1; // <- d_a_alldie.cpp:13
const ACT_TIMER: u8 = 2; // <- d_a_alldie.cpp:14

/// `daAlldie_c` (`d_a_alldie.cpp`). `daAlldie_Draw` (`:85-87`) returns TRUE with no draw call,
/// so retail renders nothing, and `daAlldie_IsDelete` (`:96-98`) is a bare TRUE - an accepted
/// delete request, never a self-destruct. `create` (`:67-82`) is unconditionally
/// `cPhs_COMPLEATE_e`, so there is no spawn-then-despawn path. The actor registers no AT or Tg
/// collider (`d_a_alldie.h` has no `dCcD_*` members), so it carries no `HitPoints`/`Hurt` and
/// cannot be damaged.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Alldie {
    pub switch_no: u8,
    pub action: u8,
    pub timer: i16,
    pub created: bool,
}

impl Alldie {
    /// `daAlldie_c::create` (`:70-74`).
    pub fn spawn(parameters: u32, switches: &Switches) -> Self {
        let switch_no = ((parameters >> SWBIT_SHIFT) & SWBIT_MASK) as u8;
        let action = if switches.test(switch_no) {
            ACT_WAIT
        } else {
            ACT_CHECK
        };
        Self {
            switch_no,
            action,
            timer: 0,
            created: false,
        }
    }

    /// `daAlldie_c::actionCheck` (`:28-35`).
    fn check(&mut self, enemy: bool) {
        if !enemy {
            self.action = ACT_TIMER;
            self.timer = TIMER_START;
        }
    }

    /// `daAlldie_c::actionTimer` (`:38-48`).
    fn countdown(&mut self, enemy: bool, switches: &mut Switches) {
        if enemy {
            self.action = ACT_CHECK;
        } else if self.timer > 0 {
            self.timer -= 1;
        } else {
            self.action = ACT_WAIT;
            switches.set(self.switch_no);
        }
    }

    /// `daAlldie_c::execute` (`:51-65`).
    fn step(&mut self, enemy: bool, switches: &mut Switches) {
        match self.action {
            ACT_CHECK => self.check(enemy),
            ACT_TIMER => self.countdown(enemy, switches),
            _ => {}
        }
    }
}

pub fn is_alldie(name: &str) -> bool {
    ALLDIE_ACTORS.contains(&name)
}

pub fn control(
    mut switches: ResMut<Switches>,
    enemies: Query<(&Actor, Option<&HitPoints>), Without<Alldie>>,
    mut markers: Query<(&mut Actor, &mut Alldie), With<Alldie>>,
) {
    for (mut actor, mut marker) in &mut markers {
        if !marker.created {
            marker.created = true;
            actor.angle[0] = 0;
            actor.angle[2] = 0;
        }
        let room = actor.room;
        let enemy = room >= 0
            && enemies
                .iter()
                .any(|(other, hit_points)| hit_points.is_some() && other.room == room);
        marker.step(enemy, &mut switches);
    }
}