use bevy_ecs::prelude::*;

use super::player::Player;
use super::sea::Sea;

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct Water {
    pub in_area: bool,
    pub surface: f32,
}

pub fn probe(sea: Option<&Sea>, x: f32, z: f32) -> (bool, f32) {
    match sea {
        Some(sea) if sea.draw_contains(x, z) => (true, sea.height_at(x, z)),
        _ => (false, 0.0),
    }
}

pub fn water_probe(sea: Option<Res<Sea>>, players: Query<&Player>, mut water: ResMut<Water>) {
    let Some(player) = players.iter().next() else {
        return;
    };
    let (in_area, surface) = probe(sea.as_deref(), player.pos.x, player.pos.z);
    water.in_area = in_area;
    water.surface = surface;
}
