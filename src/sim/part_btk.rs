use bevy_ecs::prelude::*;

/// The frame cursors of the master grip's and mirror shield's texture-scroll animations.
/// Retail keeps each `mDoExt_btkAnm` cursor on the model and advances it once per draw frame with
/// a 2% chance of restarting a stopped scroll (`tww/src/d/actor/d_a_player_main.cpp:1261-1268,1283-1294`;
/// HD `wwhd/wwhd_src/d/actor/d_a_player_main_04.cpp:3167-3169,3191-3201`).
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct PartBtk {
    pub grip: f32,
    pub shield: f32,
    seed: u32,
}

/// `tswgripms.btk` and `tshms.btk` frame maxima in `Link.arc` (ids 0x67 and 0x66).
pub const GRIP_FRAMES: f32 = 29.0;
pub const SHIELD_FRAMES: f32 = 59.0;

impl PartBtk {
    fn roll(&mut self) -> f32 {
        self.seed = self.seed.wrapping_mul(1664525).wrapping_add(1013904223);
        (self.seed >> 16) as f32 / 65536.0
    }

    fn step(frame: &mut f32, max: f32, restart: bool) {
        if *frame > 0.1 || restart {
            *frame += 1.0;
            if *frame >= max {
                *frame = 0.0;
            }
        }
    }
}

pub fn control(mut btk: ResMut<PartBtk>) {
    let restart = btk.roll() < 0.02;
    PartBtk::step(&mut btk.grip, GRIP_FRAMES, restart);
    PartBtk::step(&mut btk.shield, SHIELD_FRAMES, restart);
}
