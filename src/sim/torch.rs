use bevy_ecs::prelude::*;

use glam::Vec3;

use super::actor::Actor;
use super::flags::Switches;

pub const TORCH_ACTOR: &str = "Ep";

pub const NO_SWITCH: u8 = 0xFF;
pub const UNSET_TYPE: u8 = 0x3F;
pub const WILD_TYPE: u8 = 0;

pub const STATE_OFF: u8 = 0;
pub const STATE_WAKE: u8 = 3;
pub const STATE_BURN: u8 = 4;
pub const STATE_DYING: u8 = 10;
pub const STATE_GONE: u8 = 11;

pub const FLAME_RADIUS: f32 = 40.0;
pub const FLAME_LIFT: f32 = 30.0;
pub const POST_RADIUS: f32 = 35.0;
pub const POST_HEIGHT: f32 = 120.0;
pub const POST_TOP: f32 = 180.0;
pub const ATTENTION_LIFT: f32 = 100.0;
pub const EYE_LIFT: f32 = 130.0;
pub const GROUND_PROBE: f32 = 200.0;
pub const GROUND_MASK: u32 = 0xF;
pub const IGNITE_STEP: f32 = 0.5;
pub const IGNITE_MIN: f32 = 0.05;
pub const BURN_SCALE: f32 = 0.5;
pub const BURN_MIN: f32 = 0.2;
pub const DIE_STEP: f32 = 1.0;
pub const DIE_MIN: f32 = 0.1;
pub const DIE_LIMIT: f32 = 0.05;
pub const BURN_TICK: u16 = 10;
pub const BURN_CEILING: u16 = 10000;
pub const LIGHT_GAIN: f32 = 150.0;
pub const FLICKER: f32 = 250.0;
pub const LIGHT_RED: f32 = 600.0;
pub const LIGHT_GREEN: f32 = 400.0;
pub const LIGHT_BLUE: f32 = 0x78;
pub const AT_FLAME: u32 = 1 << 17;
pub const AT_WIND_MASK: u32 = (1 << 21) | (1 << 22);

const TYPE_MASK: u32 = 0x3F;
const GA_SHIFT: u32 = 6;
const OBM_SHIFT: u32 = 7;
const FLAME_SWITCH_SHIFT: u32 = 8;
const FLAME_MASK: u32 = 0xFF;
const GUARD_SHIFT: u32 = 16;
const ON_SWITCH_SHIFT: u32 = 24;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Torch {
    pub kind: u8,
    pub has_garland: bool,
    pub has_bowl: bool,
    pub flame_switch: u8,
    pub guard_switch: u8,
    pub on_switch: u8,
    pub state: u8,
    pub burn: u16,
    pub light: f32,
    pub peak: f32,
    pub ground_tick: u32,
    pub gone: bool,
}

impl Torch {
    pub fn spawn(parameters: u32) -> Self {
        let kind = (parameters & TYPE_MASK) as u8;
        Self {
            kind: if kind == UNSET_TYPE { WILD_TYPE } else { kind },
            has_garland: parameters >> GA_SHIFT & 1 != 0,
            has_bowl: parameters >> OBM_SHIFT & 1 != 0,
            flame_switch: (parameters >> FLAME_SWITCH_SHIFT & FLAME_MASK) as u8,
            guard_switch: (parameters >> GUARD_SHIFT & FLAME_MASK) as u8,
            on_switch: (parameters >> ON_SWITCH_SHIFT & FLAME_MASK) as u8,
            state: STATE_OFF,
            burn: 0,
            light: 0.0,
            peak: 1.0,
            ground_tick: 0,
            gone: false,
        }
    }

    pub fn standing(&self) -> bool {
        !matches!(self.kind, WILD_TYPE | 3)
    }

    pub fn relights(&self) -> bool {
        matches!(self.kind, WILD_TYPE | 3)
    }

    pub fn flame(&self, pos: Vec3) -> Vec3 {
        pos + Vec3::new(0.0, FLAME_LIFT, 0.0)
    }

    pub fn lit(&self) -> bool {
        matches!(self.state, STATE_WAKE | STATE_BURN | STATE_DYING)
    }

    pub fn solid(&self) -> bool {
        matches!(self.kind, WILD_TYPE | 3)
    }

    pub fn touch(&mut self, cut: u32, switches: &mut Switches) {
        if cut & AT_WIND_MASK != 0 || cut & AT_FLAME == 0 {
            return;
        }
        if self.flame_switch != NO_SWITCH {
            self.burn = u16::from(self.flame_switch) * BURN_TICK + 1;
            switches.set(self.flame_switch.wrapping_sub(1));
        }
    }

    pub fn tick(&mut self, switches: &Switches) {
        if self.burn > 0 && self.burn < BURN_CEILING {
            self.burn -= 1;
        }
        self.state = match self.state {
            STATE_OFF => {
                if self.flame_switch != NO_SWITCH && switches.test(self.flame_switch.wrapping_sub(1)) {
                    self.state = STATE_WAKE;
                }
                self.state
            }
            STATE_WAKE => {
                self.light = add_zero(self.light, IGNITE_STEP, IGNITE_MIN);
                if !switches.test(self.flame_switch.wrapping_sub(1)) {
                    self.state = STATE_WAKE;
                } else {
                    self.flame_switch = 0;
                    self.state
                }
            }
            STATE_BURN => {
                add_target(&mut self.light, self.peak, BURN_SCALE, BURN_MIN);
                if self.on_switch != NO_SWITCH && !switches.test(self.on_switch) {
                    self.state = STATE_DYING;
                }
                self.state
            }
            STATE_DYING => {
                add_zero(&mut self.light, DIE_STEP, DIE_MIN);
                if self.light < DIE_LIMIT {
                    if self.relights() {
                        self.state = STATE_OFF;
                    } else {
                        self.state = STATE_GONE;
                        self.gone = true;
                    }
                }
                self.state
            }
            other => other,
        };
    }

    pub fn doused(&mut self, switches: &mut Switches) {
        if self.state < STATE_BURN {
            return;
        }
        if self.guard_switch == NO_SWITCH || !switches.test(self.guard_switch) {
            if self.on_switch != NO_SWITCH {
                switches.clear(self.on_switch);
            }
        }
        if self.on_switch != NO_SWITCH && !switches.test(self.on_switch) {
            self.state = STATE_OFF;
            if self.on_switch != NO_SWITCH {
                self.flame_switch = self.on_switch + 1;
            }
        }
    }

    pub fn flood(&mut self, floor: Option<f32>, top: Vec3) {
        if self.state == STATE_OFF || self.state == STATE_DYING {
            return;
        }
        self.ground_tick += 1;
        if self.ground_tick & GROUND_MASK != 0 {
            return;
        }
        if let Some(floor) = floor
            && top.y < floor
        {
            self.state = STATE_DYING;
        }
    }

    pub fn power(&self) -> f32 {
        self.light * LIGHT_GAIN
    }

}

fn add_zero(current: f32, max_step: f32, min_step: f32) -> f32 {
    let gap = current.abs();
    if gap <= min_step {
        return 0.0;
    }
    current - current.signum() * max_step.min(gap)
}

fn add_target(current: &mut f32, target: f32, max_step: f32, min_step: f32) {
    let gap = target - *current;
    if gap.abs() <= min_step {
        *current = target;
        return;
    }
    *current += gap.signum() * max_step.min(gap.abs());
}

pub fn is_torch(name: &str) -> bool {
    name == TORCH_ACTOR
}

pub fn control(
    collision: Option<Res<super::collision::Collision>>,
    mut switches: ResMut<Switches>,
    mut torches: Query<(&Actor, &mut Torch)>,
) {
    let solid = collision.as_deref();
    for (actor, mut torch) in &mut torches {
        torch.tick(&switches);
        let top = actor.pos;
        let probe = Vec3::new(top.x, top.y + GROUND_PROBE, top.z);
        torch.flood(super::ground::surface(solid, probe), top);
        torch.doused(&mut switches);
    }
}