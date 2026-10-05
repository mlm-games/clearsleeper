use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::flags::Switches;
use super::player::Player;

pub const SWC00_ACTORS: [&str; 1] = ["SW_C00"];

pub const SW1_MASK: u32 = 0xFF; // <- d_a_swc00.h:9
pub const SW2_SHIFT: u32 = 8; // <- d_a_swc00.h:10
pub const SW2_MASK: u32 = 0xFF; // <- d_a_swc00.h:10
pub const TYPE_SHIFT: u32 = 0x10; // <- d_a_swc00.h:11
pub const TYPE_MASK: u32 = 0x3; // <- d_a_swc00.h:11
pub const NO_SWITCH: u8 = 0xFF; // <- d_a_swc00.cpp:26
pub const SCALE_MUL: f32 = 100.0; // <- d_a_swc00.cpp:78
pub const SCALE_ADD: f32 = 30.0; // <- d_a_swc00.cpp:79
pub const SCALE_SPAN: f32 = 1.0; // <- d_a_swc00.cpp:75 (retail actor scale, RULE 2)
pub const Y_MIN: f32 = -100.0; // <- d_a_swc00.cpp:30

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Swc00 {
    pub kind: u8,
    pub sw1: u8,
    pub sw2: u8,
    pub reach_xz2: f32,
    pub span_y: f32,
}

impl Swc00 {
    pub fn spawn(parameters: u32, switches: &mut Switches) -> Option<Self> {
        let sw1 = (parameters & SW1_MASK) as u8;
        let sw2 = ((parameters >> SW2_SHIFT) & SW2_MASK) as u8;
        let kind = ((parameters >> TYPE_SHIFT) & TYPE_MASK) as u8;
        if switches.test(sw1) {
            if kind != 0 {
                return None;
            }
            switches.clear(sw1);
        }
        let reach = SCALE_SPAN * SCALE_MUL + SCALE_ADD;
        Some(Self {
            kind,
            sw1,
            sw2,
            reach_xz2: reach * reach,
            span_y: SCALE_SPAN * SCALE_MUL,
        })
    }

    fn armed(&self, switches: &Switches) -> bool {
        self.sw2 == NO_SWITCH || switches.test(self.sw2)
    }

    pub fn inside(&self, pos: Vec3, player: Vec3) -> bool {
        let dx = player.x - pos.x;
        let dz = player.z - pos.z;
        let dy = player.y - pos.y;
        dx * dx + dz * dz < self.reach_xz2 && Y_MIN < dy && dy < self.span_y
    }
}

pub fn is_swc00(name: &str) -> bool {
    SWC00_ACTORS.contains(&name)
}

pub fn control(
    players: Query<&Player>,
    mut switches: ResMut<Switches>,
    mut triggers: Query<(Entity, &Actor, &Swc00)>,
    mut commands: Commands,
) {
    let Some(player) = players.iter().next().copied() else {
        return;
    };
    for (entity, actor, trigger) in &mut triggers {
        if !trigger.armed(&switches) {
            continue;
        }
        if trigger.inside(actor.pos, player.pos) {
            switches.set(trigger.sw1);
            if trigger.kind != 0 {
                let _ = commands.entity(entity).despawn();
            }
        } else if trigger.kind == 0 {
            switches.clear(trigger.sw1);
        }
    }
}