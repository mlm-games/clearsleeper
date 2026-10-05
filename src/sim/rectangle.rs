use bevy_ecs::prelude::*;

pub const RECTANGLE_ACTORS: [&str; 1] = ["hotest"];

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rectangle;

impl Rectangle {
    pub fn spawn() -> Self {
        Self
    }
}

pub fn is_rectangle(name: &str) -> bool {
    RECTANGLE_ACTORS.contains(&name)
}

pub fn control(_rectangles: Query<&Rectangle>) {}