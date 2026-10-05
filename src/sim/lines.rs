use bevy_ecs::prelude::*;

use super::actor::Actor;
use super::collision::Collision;
use super::sea::Sea;
use super::water::Water;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LineHit {
    pub point: glam::Vec3,
    pub normal: glam::Vec3,
}

#[derive(Resource, Default)]
pub struct Lines;

impl Lines {
    pub fn cross(from: glam::Vec3, to: glam::Vec3, solid: Option<&Collision>) -> Option<LineHit> {
        let solid = solid?;
        let slid = solid.slide(from, to);
        if (slid - to).length_squared() <= 1.0e-6 {
            return None;
        }
        let normal = solid
            .ground_normal(slid.x, slid.z)
            .unwrap_or(glam::Vec3::Y);
        Some(LineHit {
            point: slid,
            normal,
        })
    }
    pub fn clear(from: glam::Vec3, to: glam::Vec3, solid: Option<&Collision>) -> bool {
        Self::cross(from, to, solid).is_none()
    }
    pub fn cross_from_actor(
        actor: &Actor,
        dest: glam::Vec3,
        lift: f32,
        solid: Option<&Collision>,
    ) -> Option<LineHit> {
        let from = glam::Vec3::new(actor.pos.x, actor.pos.y + lift, actor.pos.z);
        let to = glam::Vec3::new(dest.x, dest.y + lift, dest.z);
        Self::cross(from, to, solid)
    }

    pub fn clear_from_actor(
        actor: &Actor,
        dest: glam::Vec3,
        lift: f32,
        solid: Option<&Collision>,
    ) -> bool {
        Self::cross_from_actor(actor, dest, lift, solid).is_none()
    }
}
pub fn water_height(sea: Option<&Sea>, water: Option<&Water>, x: f32, z: f32) -> Option<f32> {
    match sea {
        Some(sea) if sea.draw_contains(x, z) => Some(sea.height_at(x, z)),
        _ => water.filter(|water| water.in_area).map(|water| water.surface),
    }
}

pub fn init(world: &mut World) {
    world.init_resource::<Lines>();
}