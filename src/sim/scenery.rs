use bevy_ecs::prelude::*;

pub const SCENERY_ACTORS: [&str; 19] = [
    "kusax1",
    "kusax7",
    "kusax21",
    "swood",
    "swood3",
    "swood5",
    "lwood",
    "flower",
    "pflower",
    "pflwrx7",
    "flwr7",
    "flwr17",
    "Oyashi",
    "bridge",
    "Ojtree",
    "Kui",
    "Pitfall",
    "Grass",
    "Ypit00",
];

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Scenery;

pub fn is_scenery(name: &str) -> bool {
    SCENERY_ACTORS.contains(&name)
}