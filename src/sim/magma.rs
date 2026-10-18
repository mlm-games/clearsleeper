use bevy_ecs::prelude::*;

pub const MAGMA_ACTORS: [&str; 1] = ["magma"];

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Magma;

impl Magma {
    pub fn spawn() -> Self {
        Self
    }
}

pub fn is_magma(name: &str) -> bool {
    MAGMA_ACTORS.contains(&name)
}

pub fn control(mut pools: Query<(Entity, &Magma)>, mut commands: Commands) {
    for (entity, _) in &mut pools {
        let _ = commands.entity(entity).despawn();
    }
}
