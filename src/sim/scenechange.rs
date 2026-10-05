use bevy_ecs::prelude::*;
use glam::Vec3;

use super::walk::from_s16;

pub const SCENE_CHANGE_ACTORS: [&str; 1] = ["ScnChg"];

fn zxy_basis(shape: [i16; 3]) -> [Vec3; 3] {
    let (x_sin, x_cos) = from_s16(shape[0]).sin_cos();
    let (y_sin, y_cos) = from_s16(shape[1]).sin_cos();
    let (z_sin, z_cos) = from_s16(shape[2]).sin_cos();
    [
        Vec3::new(
            y_cos * z_cos - y_sin * x_sin * z_sin,
            -x_cos * z_sin,
            y_sin * z_cos + y_cos * x_sin * z_sin,
        ),
        Vec3::new(
            y_cos * z_sin + y_sin * x_sin * z_cos,
            x_cos * z_cos,
            y_sin * z_sin - y_cos * x_sin * z_cos,
        ),
        Vec3::new(-y_sin * x_cos, x_sin, y_cos * x_cos),
    ]
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct SceneChange {
    pub basis: [Vec3; 3],
}

impl SceneChange {
    pub fn spawn(shape: [i16; 3]) -> Self {
        Self {
            basis: zxy_basis(shape),
        }
    }

    pub fn basis(&self) -> [Vec3; 3] {
        self.basis
    }
}

pub fn is_scene_change(name: &str) -> bool {
    SCENE_CHANGE_ACTORS.contains(&name)
}

pub fn control(changes: Query<&SceneChange>) {
    for change in changes.iter() {
        let _ = change.basis();
    }
}