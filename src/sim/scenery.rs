use bevy_ecs::prelude::*;

pub const SCENERY_ACTORS: [&str; 21] = [
    "kusax1",
    "kusax7",
    "kusax21",
    "swood",
    "swood3",
    "swood5",
    "lwood",
    "woodb",
    "flower",
    "pflower",
    "pflwrx7",
    "flwr7",
    "flwr17",
    "koisi1",
    "Oyashi",
    "Pirates",
    "bridge",
    "Ojtree",
    "Mhsg15",
    "Kui",
    "Pitfall",
];

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Scenery;

pub fn is_scenery(name: &str) -> bool {
    SCENERY_ACTORS.contains(&name)
}