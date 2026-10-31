//! `daObjEkskz::Act_c` (`tww/src/d/actor/d_a_obj_ekskz.cpp`), the eight-eyed statue. `_create`
//! refuses the actor when its switch is already on (`:146`); the Tg type is
//! `AT_TYPE_SKULL_HAMMER` (`:24`); a hit turns the switch on (`:195`) and plays the break model
//! until its BRK stops (`:218-222`). The port does not load the BRK, so the break lasts a fixed
//! 30 frames before the actor is deleted; the cull box is `:100`.
use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::flags::Switches;
use super::hammer::{self, Cut, Hammer};
use super::player::Player;

pub const EKSKZ_ACTOR: &str = "Ekskz";

const SWSAVE_MASK: u32 = 0xFF;
const NO_SWITCH: u8 = 0xFF;
const CULL_MIN: [f32; 3] = [-4000.0, -500.0, -4000.0];
const CULL_MAX: [f32; 3] = [4000.0, 500.0, 4000.0];
const BREAK_FRAMES: i32 = 30;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Ekskz {
    pub refused: bool,
    pub broken: bool,
    pub break_timer: i32,
    switch: u8,
}

impl Ekskz {
    pub fn spawn(parameters: u32, switches: &Switches) -> Self {
        let switch = (parameters & SWSAVE_MASK) as u8;
        let refused = switch != NO_SWITCH && switches.test(switch);
        Self {
            refused,
            broken: false,
            break_timer: 0,
            switch,
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        !self.refused
    }
}

pub fn is_ekskz(name: &str) -> bool {
    name == EKSKZ_ACTOR
}

pub fn control(
    cut: Res<Cut>,
    hammers: Query<&Hammer>,
    players: Query<&Player>,
    mut switches: ResMut<Switches>,
    mut statues: Query<(Entity, &Actor, &mut Ekskz)>,
    mut commands: Commands,
) {
    let striking = cut.kind != hammer::CUT_NONE && hammers.iter().any(|hammer| hammer.striking());
    let player = players.iter().next();
    for (entity, actor, mut statue) in &mut statues {
        if statue.refused {
            let _ = commands.entity(entity).despawn();
            continue;
        }
        if statue.broken {
            statue.break_timer -= 1;
            if statue.break_timer <= 0 {
                let _ = commands.entity(entity).despawn();
            }
            continue;
        }
        let Some(player) = player else {
            continue;
        };
        if !striking {
            continue;
        }
        let gap = actor.pos - player.pos;
        if gap.x * gap.x + gap.z * gap.z > hammer::AT_RADIUS * hammer::AT_RADIUS {
            continue;
        }
        if statue.switch != NO_SWITCH {
            switches.set(statue.switch);
        }
        statue.broken = true;
        statue.break_timer = BREAK_FRAMES;
    }
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
    let statue = Ekskz::spawn(source.parameters, world.resource::<Switches>());
    world.spawn((actor, statue)).id()
}
