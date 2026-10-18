use bevy_ecs::prelude::*;

pub const WOOD_BOX_ACTORS: [&str; 2] = ["woodbx", "woodb"];

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct WoodBox;

impl WoodBox {
    pub fn spawn() -> Self {
        Self
    }
}

pub fn is_wood_box(name: &str) -> bool {
    WOOD_BOX_ACTORS.contains(&name)
}

pub fn control(mut boxes: Query<(Entity, &WoodBox)>, mut commands: Commands) {
    for (entity, _) in &mut boxes {
        let _ = commands.entity(entity).despawn();
    }
}
