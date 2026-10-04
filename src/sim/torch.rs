use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::collision::Collision;
use super::flags::Switches;
use super::ground;

pub const TORCH_ACTOR: &str = "Ep";

pub const NO_SWITCH: u8 = 0xFF;
pub const UNSET_TYPE: u8 = 0x3F;
pub const WALL_TYPE: u8 = 3;
pub const NO_EMBERS_TYPE: u8 = 2;

pub const STATE_OFF: u8 = 0;
pub const STATE_WAKE: u8 = 3;
pub const STATE_BURN: u8 = 4;
pub const STATE_DYING: u8 = 10;
pub const STATE_GONE: u8 = 11;

pub const FLAME_RADIUS: f32 = 40.0;
pub const FLAME_LIFT: f32 = 10.0;
pub const FLAME_SET_AT: f32 = 0.5;
pub const POST_RADIUS: f32 = 35.0;
pub const POST_HEIGHT: f32 = 120.0;
pub const TOP_HEIGHT: f32 = 140.0;
pub const CULL_TOP: f32 = 180.0;
pub const ATTENTION_LIFT: f32 = 100.0;
pub const EYE_LIFT: f32 = 130.0;
pub const GROUND_PROBE: f32 = 200.0;
pub const GROUND_MASK: u32 = 0xF;

pub const IGNITE_SCALE: f32 = 0.5;
pub const IGNITE_MAX_STEP: f32 = 0.05;
pub const BURN_SCALE: f32 = 0.5;
pub const BURN_MAX_STEP: f32 = 0.2;
pub const DIE_SCALE: f32 = 1.0;
pub const DIE_MAX_STEP: f32 = 0.10000000149011612;
pub const DIE_LIMIT: f32 = 0.05;

pub const BURN_TICK: u16 = 10;
pub const BURN_CEILING: u16 = 10000;
pub const CLIP_SEED: f32 = -60.0;
pub const CLIP_COOLDOWN: u8 = 0x14;

pub const LIGHT_GAIN: f32 = 150.0;
pub const LIGHT_RED: f32 = 600.0;
pub const LIGHT_GREEN: f32 = 400.0;
pub const LIGHT_BLUE: f32 = 0x78;
pub const LIGHT_FLUCTUATION: f32 = 250.0;

pub const AT_WIND_MASK: u32 = (1 << 21) | (1 << 22);

const TYPE_MASK: u32 = 0x3F;
const GA_SHIFT: u32 = 6;
const OBM_SHIFT: u32 = 7;
const TIMER_SEED_SHIFT: u32 = 8;
const GUARD_SHIFT: u32 = 16;
const ON_SWITCH_SHIFT: u32 = 24;
const FIELD_MASK: u32 = 0xFF;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Off,
    Wake,
    Burn,
    Dying,
    Gone,
}

impl State {
    pub fn code(self) -> u8 {
        match self {
            Self::Off => STATE_OFF,
            Self::Wake => STATE_WAKE,
            Self::Burn => STATE_BURN,
            Self::Dying => STATE_DYING,
            Self::Gone => STATE_GONE,
        }
    }

    pub fn lit(self) -> bool {
        matches!(self, Self::Wake | Self::Burn | Self::Dying)
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Torch {
    pub kind: u8,
    pub has_garland: bool,
    pub has_bowl: bool,
    pub timer_seed: u8,
    pub guard: u8,
    pub on_switch: u8,
    pub pending: u8,
    pub state: State,
    pub burn: u16,
    pub clip: u8,
    pub light: f32,
    pub peak: f32,
    pub ground_tick: u32,
}

impl Torch {
    pub fn spawn(parameters: u32, scale: f32) -> Self {
        let kind = (parameters & TYPE_MASK) as u8;
        Self {
            kind: if kind == UNSET_TYPE { 0 } else { kind },
            has_garland: parameters >> GA_SHIFT & 1 != 0,
            has_bowl: parameters >> OBM_SHIFT & 1 != 0,
            timer_seed: (parameters >> TIMER_SEED_SHIFT & FIELD_MASK) as u8,
            guard: (parameters >> GUARD_SHIFT & FIELD_MASK) as u8,
            on_switch: (parameters >> ON_SWITCH_SHIFT & FIELD_MASK) as u8,
            pending: 0,
            state: State::Off,
            burn: 0,
            clip: 0,
            light: 0.0,
            peak: scale,
            ground_tick: 0,
        }
    }

    pub fn wall(&self) -> bool {
        self.kind == WALL_TYPE
    }

    pub fn relights(&self) -> bool {
        self.kind == 0 || self.kind == WALL_TYPE
    }

    pub fn embers(&self) -> bool {
        self.kind != NO_EMBERS_TYPE
    }

    pub fn solid(&self) -> bool {
        self.kind == 0 || self.kind == WALL_TYPE
    }

    pub fn flame(&self, pos: Vec3) -> Vec3 {
        pos + Vec3::new(0.0, FLAME_LIFT, 0.0)
    }

    pub fn ballasted(&self) -> bool {
        self.light >= FLAME_SET_AT
    }

    pub fn struck(&mut self) {
        if self.timer_seed != NO_SWITCH {
            self.burn = u16::from(self.timer_seed) * BURN_TICK + 1;
        }
    }

    pub fn light_it(&mut self, switches: &mut Switches) {
        if self.pending != 0 {
            switches.set(self.pending - 1);
            self.pending = 0;
        }
    }

    pub fn restrike(&mut self) {
        self.state = State::Wake;
        if self.on_switch != NO_SWITCH {
            self.pending = self.on_switch + 1;
        }
    }

    pub fn tick_burn(&mut self) {
        if self.burn > 0 && self.burn < BURN_CEILING {
            self.burn -= 1;
        }
    }

    pub fn tick_clip(&mut self) {
        if self.clip > 0 {
            self.clip -= 1;
        }
    }

    pub fn clip_barge(&mut self) {
        if self.clip == 0 {
            self.light = CLIP_SEED;
            self.clip = CLIP_COOLDOWN;
        }
    }

    pub fn snuff(&mut self, switches: &mut Switches) {
        if self.burn == 1 && (self.guard == NO_SWITCH || !switches.test(self.guard))
            && self.on_switch != NO_SWITCH
        {
            switches.clear(self.on_switch);
        }
        if self.on_switch != NO_SWITCH && !switches.test(self.on_switch) {
            self.state = State::Off;
            self.pending = self.on_switch + 1;
        }
    }

    pub fn spent(&mut self, switches: &mut Switches) {
        if self.relights() {
            self.state = State::Off;
            if self.on_switch != NO_SWITCH {
                switches.clear(self.on_switch);
                self.pending = self.on_switch + 1;
            }
        } else {
            self.state = State::Gone;
        }
    }

    pub fn flood(&mut self, floor: Option<f32>, top: f32) {
        if !self.state.lit() || self.state == State::Dying {
            return;
        }
        self.ground_tick += 1;
        if self.ground_tick & GROUND_MASK == 0 && floor.is_some_and(|floor| top < floor) {
            self.state = State::Dying;
        }
    }

    pub fn power(&self) -> f32 {
        self.light * LIGHT_GAIN
    }

    pub fn colour(&self) -> (f32, f32, f32) {
        (LIGHT_RED, LIGHT_GREEN, LIGHT_BLUE)
    }
}

fn add_calc_two(current: f32, target: f32, scale: f32, max_step: f32) -> f32 {
    if current == target {
        return current;
    }
    current + (scale * (target - current)).clamp(-max_step, max_step)
}

fn add_calc_zero(current: f32, scale: f32, max_step: f32) -> f32 {
    current - (current * scale).clamp(-max_step, max_step)
}

pub fn is_torch(name: &str) -> bool {
    name == TORCH_ACTOR
}

pub fn control(
    collision: Option<Res<Collision>>,
    mut switches: ResMut<Switches>,
    mut torches: Query<(&Actor, &mut Torch)>,
) {
    let solid = collision.as_deref();
    for (actor, mut torch) in &mut torches {
        torch.tick_burn();
        torch.tick_clip();
        match torch.state {
            State::Off => {
                if torch.pending != 0 {
                    torch.light = add_calc_zero(torch.light, IGNITE_SCALE, IGNITE_MAX_STEP);
                    torch.light_it(&mut switches);
                }
            }
            State::Wake => {
                torch.light = add_calc_zero(torch.light, IGNITE_SCALE, IGNITE_MAX_STEP);
                if torch.pending == 0 {
                    torch.state = State::Burn;
                    torch.peak = 1.0;
                }
            }
            State::Burn => {
                let peak = torch.peak;
                torch.light = add_calc_two(torch.light, peak, BURN_SCALE, BURN_MAX_STEP);
                torch.snuff(&mut switches);
            }
            State::Dying => {
                torch.light = add_calc_zero(torch.light, DIE_SCALE, DIE_MAX_STEP);
                if torch.light < DIE_LIMIT {
                    torch.spent(&mut switches);
                }
            }
            State::Gone => {}
        }
        let probe = Vec3::new(actor.pos.x, actor.pos.y + GROUND_PROBE, actor.pos.z);
        torch.flood(ground::surface(solid, probe), actor.pos.y);
    }
}
