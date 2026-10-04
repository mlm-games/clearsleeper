use bevy_ecs::prelude::*;
use glam::Vec3;

use super::flags::Switches;
use super::stream;

pub const ICE_ACTOR: &str = "Ikori";

pub const NO_SWITCH: u8 = 0xFF;
pub const SHELLS: usize = 3;
pub const RADII: [f32; SHELLS] = [40.0, 65.0, 75.0];
pub const HEIGHTS: [f32; SHELLS] = [120.0, 90.0, 60.0];

pub const MELT_STEP: f32 = 0.0009;
pub const THAW_STEP: f32 = 0.011111111;
pub const FADE_STEP: f32 = 2.8333333;
pub const ALPHA_MAX: f32 = 255.0;
pub const PROFILE: i16 = 0x4000;
pub const CHIPPED_AT: f32 = 0.1;
pub const RELEASE_AT: f32 = 0.4;
pub const FADE_FRAMES: i16 = 0x5A;
pub const SOUND_MELT: u32 = 0x6A16;
pub const STTS_WEIGHT: u8 = 0xFF;
pub const STTS_POWER: u8 = 0xFF;

pub const STATE_WHOLE: u8 = 0;
pub const STATE_CHIPPED: u8 = 1;
pub const STATE_MELTING: u8 = 2;
pub const STATE_GONE: u8 = 3;

pub const AT_FIRE: u32 = 1 << 9;
pub const AT_UNK20000: u32 = 1 << 17;
pub const AT_FIRE_ARROW: u32 = 1 << 18;
pub const AT_BOMB: u32 = 1 << 5;
pub const HEAT: u32 = AT_FIRE | AT_UNK20000 | AT_FIRE_ARROW;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Whole,
    Chipped,
    Melting,
    Gone,
}

impl State {
    pub fn code(self) -> u8 {
        match self {
            Self::Whole => STATE_WHOLE,
            Self::Chipped => STATE_CHIPPED,
            Self::Melting => STATE_MELTING,
            Self::Gone => STATE_GONE,
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Ice {
    pub switch_no: u8,
    pub state: State,
    pub radius: f32,
    pub height: f32,
    pub frames: i16,
    pub alpha: f32,
    pub scale: Vec3,
}

impl Ice {
    pub fn spawn(parameters: u32, switches: &Switches) -> Self {
        let switch_no = (parameters & 0xFF) as u8;
        let gone = switch_no != NO_SWITCH && switches.test(switch_no);
        Self {
            switch_no,
            state: if gone { State::Gone } else { State::Whole },
            radius: 1.0,
            height: 1.0,
            frames: FADE_FRAMES,
            alpha: ALPHA_MAX,
            scale: Vec3::ONE,
        }
    }

    pub fn whole(&self) -> bool {
        self.state == State::Whole
    }

    pub fn blocks(&self) -> bool {
        self.state != State::Gone
    }

    pub fn solid(&self) -> bool {
        self.state == State::Whole || self.state == State::Chipped
    }

    pub fn hull(&self) -> f32 {
        self.radius * self.scale.x.max(self.scale.z) * RADII[SHELLS - 1]
    }

    pub fn hull_height(&self) -> f32 {
        self.height * self.scale.y * HEIGHTS[0]
    }

    pub fn local_hit(&self, pos: Vec3, heading: i16, hit: Vec3) -> Option<Vec3> {
        let facing = super::walk::from_s16(-heading);
        let (sin_yaw, cos_yaw) = facing.sin_cos();
        let gap_x = hit.x - pos.x;
        let gap_z = hit.z - pos.z;
        let local = Vec3::new(
            gap_x * cos_yaw + gap_z * sin_yaw,
            hit.y - pos.y,
            -gap_x * sin_yaw + gap_z * cos_yaw,
        );
        for shell in 0..SHELLS {
            let reach_x = self.radius * (self.scale.x * RADII[shell]);
            let reach_y = self.height * (self.scale.y * HEIGHTS[shell]);
            let reach_z = self.radius * (self.scale.z * RADII[shell]);
            if local.y < reach_y
                && local.x * local.x / (reach_x * reach_x) + local.z * local.z / (reach_z * reach_z)
                    < 1.0
            {
                return Some(local);
            }
        }
        None
    }

    pub fn strike(&mut self, at: u32) -> bool {
        match at {
            AT_BOMB => false,
            HEAT => {
                self.state = State::Melting;
                self.frames = FADE_FRAMES;
                true
            }
            _ => {
                self.radius = (self.radius - MELT_STEP).max(0.0);
                self.height = profile(self.radius);
                self.state = if self.radius < CHIPPED_AT {
                    State::Chipped
                } else {
                    State::Whole
                };
                true
            }
        }
    }

    pub fn thaw(&mut self) -> bool {
        self.radius = (self.radius - THAW_STEP).max(0.0);
        self.alpha = (self.alpha - FADE_STEP).max(0.0);
        self.height = profile(self.radius);
        self.frames -= 1;
        if self.frames <= 0 {
            self.state = State::Gone;
        }
        self.radius < RELEASE_AT
    }

    pub fn latch(&mut self, switches: &mut Switches) {
        if self.switch_no != NO_SWITCH {
            switches.set(self.switch_no);
        }
    }
}

fn profile(radius: f32) -> f32 {
    1.0 - stream::cos_s((radius * PROFILE) as i16)
}

pub fn is_ice(name: &str) -> bool {
    name == ICE_ACTOR
}

pub fn control(mut switches: ResMut<Switches>, mut blocks: Query<&mut Ice>) {
    for mut block in &mut blocks {
        if !matches!(block.state, State::Melting | State::Chipped) {
            continue;
        }
        if block.thaw() {
            block.latch(&mut switches);
        }
    }
}
