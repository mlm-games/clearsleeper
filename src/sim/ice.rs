use bevy_ecs::prelude::*;

use glam::Vec3;

use super::flags::Switches;

pub const ICE_ACTOR: &str = "Ikori";

pub const NO_SWITCH: u8 = 0xFF;
pub const SHELLS: usize = 3;
pub const RADII: [f32; SHELLS] = [40.0, 65.0, 75.0];
pub const HEIGHTS: [f32; SHELLS] = [120.0, 90.0, 60.0];
pub const MELT_STEP: f32 = 0.0009;
pub const THAW_STEP: f32 = 0.011111111;
pub const THAW_RADIUS: f32 = 80.0;
pub const THAW_HEIGHT: f32 = 150.0;
pub const ALPHA_MAX: f32 = 255.0;
pub const FADE_STEP: f32 = 2.8333333;
pub const FADE_FRAMES: i16 = 0x5A;
pub const RELEASE_AT: f32 = 0.4;
pub const SOLID_AT: f32 = 0.1;
pub const HULL_RADIUS: f32 = 40.0;
pub const HULL_HEIGHT: f32 = 120.0;

pub const STATE_WHOLE: u8 = 0;
pub const STATE_CHIPPED: u8 = 1;
pub const STATE_MELTING: u8 = 2;
pub const STATE_GONE: u8 = 3;

pub const AT_FIRE: u32 = 1 << 9;
pub const AT_UNK20000: u32 = 1 << 17;
pub const AT_FIRE_ARROW: u32 = 1 << 18;
pub const AT_BOMB: u32 = 1 << 5;
pub const HEAT: u32 = AT_FIRE | AT_UNK20000 | AT_FIRE_ARROW;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Ice {
    pub switch_no: u8,
    pub state: u8,
    pub shrink: f32,
    pub height: f32,
    pub frames: i16,
    pub alpha: f32,
    pub scale: Vec3,
    pub gone: bool,
}

impl Ice {
    pub fn spawn(parameters: u32, switches: &Switches) -> Self {
        let switch_no = (parameters & 0xFF) as u8;
        let whole = switch_no == NO_SWITCH || !switches.test(switch_no);
        Self {
            switch_no,
            state: if whole { STATE_WHOLE } else { STATE_GONE },
            shrink: 1.0,
            height: 1.0,
            frames: FADE_FRAMES,
            alpha: ALPHA_MAX,
            scale: Vec3::ONE,
            gone: !whole,
        }
    }

    pub fn whole(&self) -> bool {
        self.state == STATE_WHOLE
    }

    pub fn blocks(&self) -> bool {
        self.state != STATE_GONE
    }

    pub fn solid(&self) -> bool {
        self.state == STATE_WHOLE || self.state == STATE_CHIPPED
    }

    pub fn hit(&mut self, at: u32, local: Vec3) -> bool {
        if self.state != STATE_WHOLE && self.state != STATE_CHIPPED {
            return false;
        }
        if at & HEAT != 0 {
            self.state = STATE_MELTING;
            self.frames = FADE_FRAMES;
            return true;
        }
        if at != AT_BOMB {
            return false;
        }
        self.shrink = (self.shrink - MELT_STEP).max(0.0);
        self.height = 1.0 - (self.shrink * core::f32::consts::PI).cos();
        if self.shrink < SOLID_AT {
            self.state = STATE_CHIPPED;
        }
        true
    }

    pub fn inside(&self, local: Vec3) -> bool {
        for shell in 0..SHELLS {
            let radius_x = self.shrink * (self.scale.x * RADII[shell]);
            let height = self.height * (self.scale.y * HEIGHTS[shell]);
            let radius_z = self.shrink * (self.scale.z * RADII[shell]);
            if local.y < height
                && local.x * local.x / (radius_x * radius_x)
                    + local.z * local.z / (radius_z * radius_z)
                    < 1.0
            {
                return true;
            }
        }
        false
    }

    pub fn tick(&mut self, switches: &mut Switches) {
        match self.state {
            STATE_WHOLE | STATE_CHIPPED => {}
            STATE_MELTING => {
                let shrink = (self.shrink - THAW_STEP).max(0.0);
                self.alpha = (self.alpha - FADE_STEP).max(0.0);
                self.height = 1.0 - (shrink * core::f32::consts::PI).cos();
                if shrink < RELEASE_AT {
                    self.blocks_off(switches);
                }
                self.shrink = shrink;
                self.frames -= 1;
                if self.frames <= 0 {
                    self.state = STATE_GONE;
                    self.gone = true;
                }
            }
            _ => {}
        }
    }

    fn blocks_off(&mut self, switches: &mut Switches) {
        if self.switch_no != NO_SWITCH {
            switches.set(self.switch_no);
        }
    }

    pub fn hull(&self) -> f32 {
        self.scale.x.max(self.scale.z) * RADII[SHELLS - 1]
    }

    pub fn hull_height(&self) -> f32 {
        self.scale.y * HEIGHTS[0]
    }
}

pub fn is_ice(name: &str) -> bool {
    name == ICE_ACTOR
}

pub fn control(mut switches: ResMut<Switches>, mut blocks: Query<&mut Ice>) {
    for mut block in &mut blocks {
        block.tick(&mut switches);
    }
}
