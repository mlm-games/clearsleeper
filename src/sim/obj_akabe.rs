use bevy_ecs::prelude::*;

use super::actor::Actor;
use super::flags::{Collect, Switches};

pub const AKABE_ACTORS: [&str; 1] = ["Akabe"];

pub fn is_akabe(name: &str) -> bool {
    AKABE_ACTORS.contains(&name)
}

const SCALE_MODE_ONE: u32 = 1;
const SCALE_MODE_TWO: u32 = 2;
const SCALE_MODE_THREE: u32 = 3;
const SCALE_FACTOR: f32 = 10.0;
const NO_SWITCH: i32 = 0xFF;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Shape {
    #[default]
    Default,
    Deleted,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Akabe {
    pub shape: Shape,
    pub scale_mode: u32,
    pub always: bool,
    pub switch_save: u8,
    pub scale: glam::Vec3,
}

impl Akabe {
    pub fn spawn(parameters: u32, source_scale: [f32; 3]) -> Self {
        let scale_mode = (parameters >> 8) & 0x3;
        let always = (parameters >> 12) & 0x1 != 0;
        let switch_save = (parameters & 0xFF) as u8;
        let mut scale = glam::Vec3::from_array(source_scale);
        match scale_mode {
            SCALE_MODE_ONE => {
                scale.x *= SCALE_FACTOR;
                scale.y *= SCALE_FACTOR;
                scale.z = 1.0;
            }
            SCALE_MODE_TWO => {}
            SCALE_MODE_THREE => {
                scale *= SCALE_FACTOR;
            }
            _ => {
                scale.z = 1.0;
            }
        }
        Self {
            shape: Shape::Default,
            scale_mode,
            always,
            switch_save,
            scale,
        }
    }
}

pub fn control(
    mut items: Query<&mut Akabe>,
    switches: Res<Switches>,
    collect: Res<Collect>,
) {
    for mut akabe in items.iter_mut() {
        if akabe.shape == Shape::Deleted {
            continue;
        }
        if appears(&akabe, &switches, &collect) {
            continue;
        }
        akabe.shape = Shape::Deleted;
    }
}

fn appears(akabe: &Akabe, switches: &Switches, collect: &Collect) -> bool {
    if akabe.always {
        return true;
    }
    if akabe.switch_save as i32 == NO_SWITCH {
        collect.check(0) == 0
    } else {
        !switches.test(akabe.switch_save)
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
    };
    let item = Akabe::spawn(source.parameters, source.scale);
    world.spawn((actor, item)).id()
}
