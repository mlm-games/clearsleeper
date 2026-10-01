use bevy_ecs::prelude::*;

#[derive(Resource)]
pub struct CurrentFrame(pub u64);

pub fn advance(mut frame: ResMut<CurrentFrame>) {
    frame.0 += 1;
}
