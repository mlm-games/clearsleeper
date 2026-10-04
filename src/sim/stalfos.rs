use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::collision::Collision;
use super::combat::HitPoints;
use super::events::{SimEvent, SimEvents};
use super::flags::Switches;
use super::ground;
use super::health::Health;
use super::player::Player;
use super::stream;
use super::walk;

pub const STAL_ACTORS: [&str; 1] = ["Stal"];

pub const BEHAVIOR_NORMAL: u8 = 0;
pub const BEHAVIOR_UNDERGROUND: u8 = 1;
pub const BEHAVIOR_IN_HORIZONTAL_COFFIN: u8 = 2;
pub const BEHAVIOR_IN_VERTICAL_COFFIN: u8 = 3;
pub const BEHAVIOR_UPPER_BODY_ONLY: u8 = 14;

pub const BEHAVIOR_MASK: u32 = 0xF;
pub const AMBUSH_RANGE_SHIFT: u32 = 8;
pub const AMBUSH_SWITCH_SHIFT: u32 = 0x18;
pub const FIELD_MASK: u32 = 0xFF;
pub const NO_SWITCH: u8 = 0xFF;

pub const HIO_M06: i16 = 9;
pub const HIO_M08: i16 = 0x2C;
pub const HIO_M0A: i16 = 0x15;
pub const HIO_M0C: i16 = 0x35;
pub const HIO_M10: f32 = 6.0;
pub const HIO_M18: i16 = 0x46;
pub const HIO_M1A: i16 = 200;

pub const ACT_JYUNKAI: i16 = 0;
pub const ACT_FIGHT: i16 = 5;
pub const ACT_GROUND_WAIT: i16 = 0xF;
pub const ACT_KAN_WAIT: i16 = 0x10;
pub const ACT_BREAK_WAIT: i16 = 0x1E;

pub const PATROL_WALK: i16 = 0;
pub const PATROL_WAIT: i16 = 1;

pub const FIGHT_IDLE_RIGHT: u8 = 3;
pub const FIGHT_IDLE_LEFT: u8 = 8;
pub const FIGHT_SPIN_LEFT: u8 = 0x1A;
pub const FIGHT_SPIN_RIGHT: u8 = 0x15;

pub const TRIGGER_WAIT: i16 = 0;
pub const TRIGGER_RISE: i16 = 1;
pub const TRIGGER_HOLD: i16 = 2;
pub const TRIGGER_STAND: i16 = 3;

pub const BREAK_START: i16 = 0;
pub const BREAK_HOLD: i16 = 1;
pub const BREAK_REGEN: i16 = 2;
pub const BREAK_SETTLE: i16 = 0x64;

pub const TURN_DIV: i16 = 2;
pub const WALK_TURN: i16 = 0x800;
pub const FIGHT_TURN: i16 = 0x200;
pub const SPEED_SCALE: f32 = 1.0;
pub const SPEED_STEP: f32 = 1.0;
pub const PATROL_SPEED_STEP: f32 = 5.0;
pub const PATROL_SPEED: f32 = 0.0;
pub const IDLE_SPEED: f32 = 0.0;
pub const GRAVITY: f32 = -5.0;
pub const RETREAT_SPEED: f32 = -2.0;
pub const SPIN_CHARGE: f32 = 10.0;
pub const SWAY_AMPLITUDE: f32 = 10000.0;
pub const SWAY_FREQ: u32 = 500;

pub const SWING_REST: f32 = 0.0;
pub const SWING_RATE_SLOW: f32 = 0.02;
pub const SWING_RATE_FAST: f32 = 0.1;
pub const SWING_LEFT: f32 = -1.0;
pub const SWING_RIGHT: f32 = 1.0;
pub const SPIN_STEP: f32 = 5000.0;
pub const SPIN_GATE: i16 = 0x400;
pub const SPIN_DECAY_DIV: i16 = 2;
pub const SPIN_DECAY_STEP: i16 = 0x40;
pub const SWING_DEADZONE: f32 = 0.05;
pub const SWING_SOUND_GATE: i16 = 100;

pub const AT_DEADZONE: f32 = 0.5;
pub const AT_SPIN_GAIN: f32 = 1.5;
pub const AT_RADIUS_EVEN: f32 = 50.0;
pub const AT_RADIUS_ODD: f32 = 70.0;
pub const AT_MACE_POWER: u8 = 2;

pub const PATROL_SPAN: f32 = 60.0;
pub const PATROL_BASE: f32 = 100.0;
pub const AGGRO_RANGE: f32 = 280.0;
pub const ENGAGE_SPAN: f32 = 40.0;
pub const FIGHT_HOLD_SPAN: f32 = 50.0;
pub const FIGHT_HOLD_BASE: f32 = 30.0;
pub const RETREAT_RANGE: f32 = 300.0;
pub const RND_HALF: f32 = 0.5;
pub const GROUND_RISE_FRAMES: i16 = 60;
pub const AMBUSH_RANGE_SCALE: f32 = 10.0;
pub const AMBUSH_RANGE_DEFAULT: f32 = 500.0;
pub const COFFIN_HOLD: i16 = 0x14;
pub const BREAK_SETTLE_FRAMES: i16 = 10;
pub const LIMB_FULL: i8 = 0x19;

pub const SE_WALK: u32 = 0x3832;
pub const SE_OUT_OF_GROUND: u32 = 0x589A;
pub const SE_CLUB_SWING_L: u32 = 0x5895;
pub const SE_RECOVER_BODY: u32 = 0x5892;

const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
const RND_SEED: i32 = 100;

fn rnd(seed: &mut [i32; 3]) -> f32 {
    for lane in 0..3 {
        seed[lane] = (seed[lane] * RND_MUL[lane]) % RND_MOD[lane];
    }
    let sum = f32::from(seed[0]) / RND_SCALE[0]
        + f32::from(seed[1]) / RND_SCALE[1]
        + f32::from(seed[2]) / RND_SCALE[2];
    (sum % 1.0).abs()
}

fn rnd_f(seed: &mut [i32; 3], range: f32) -> f32 {
    rnd(seed) * range
}

fn spin_open(angle: i16) -> bool {
    angle > -SPIN_GATE && angle < SPIN_GATE
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Stalfos {
    pub ambush_range: u8,
    pub ambush_switch: u8,
    pub action: i16,
    pub state: i16,
    pub fight: u8,
    pub timers: [i16; 5],
    pub limbs: i8,
    pub frame: i32,
    pub tick: u32,
    pub speed_f: f32,
    pub speed_y: f32,
    pub speed_target: f32,
    pub speed_step: f32,
    pub turn_step: i16,
    pub lean: i16,
    pub swing: f32,
    pub spin_angle: i16,
    pub swing_sound: i16,
    pub armour: u8,
    pub weapon_at: u8,
    pub seed: [i32; 3],
}

impl Stalfos {
    pub fn spawn(parameters: u32, shape_z: i16, switches: &Switches) -> Option<(Self, HitPoints)> {
        let behavior = (parameters & BEHAVIOR_MASK) as u8;
        if behavior == BEHAVIOR_UPPER_BODY_ONLY {
            return None;
        }
        let ambush_range = ((parameters >> AMBUSH_RANGE_SHIFT) & FIELD_MASK) as u8;
        let mut ambush_switch = ((parameters >> AMBUSH_SWITCH_SHIFT) & FIELD_MASK) as u8;
        let death_switch = (shape_z as u32 & FIELD_MASK) as u8;
        let death_switch = if death_switch == NO_SWITCH {
            0
        } else {
            death_switch
        };
        if death_switch != 0 && switches.test(death_switch) {
            return None;
        }
        let action = if behavior == BEHAVIOR_UNDERGROUND {
            ACT_GROUND_WAIT
        } else if behavior == BEHAVIOR_IN_HORIZONTAL_COFFIN
            || behavior == BEHAVIOR_IN_VERTICAL_COFFIN
        {
            ACT_KAN_WAIT
        } else {
            ACT_JYUNKAI
        };
        if behavior != BEHAVIOR_NORMAL {
            ambush_switch = if ambush_switch == NO_SWITCH {
                NO_SWITCH
            } else {
                ambush_switch + 1
            };
        }
        (
            Self {
                ambush_range,
                ambush_switch,
                action,
                state: if action == ACT_JYUNKAI {
                    PATROL_WALK
                } else {
                    TRIGGER_WAIT
                },
                fight: FIGHT_IDLE_RIGHT,
                timers: [0; 5],
                limbs: LIMB_FULL,
                frame: 0,
                tick: 0,
                speed_f: 0.0,
                speed_y: 0.0,
                speed_target: PATROL_SPEED,
                speed_step: SPEED_STEP,
                turn_step: 0,
                lean: 0,
                swing: 0.0,
                spin_angle: 0,
                swing_sound: 0,
                armour: 0,
                weapon_at: 0,
                seed: [RND_SEED; 3],
            },
            HitPoints(super::damage::KILL_POWER),
        )
    }

    fn ambush_reach(&self) -> f32 {
        if self.ambush_range == NO_SWITCH {
            AMBUSH_RANGE_DEFAULT
        } else {
            f32::from(self.ambush_range) * AMBUSH_RANGE_SCALE
        }
    }

    fn gait(&mut self, events: &mut SimEvents) {
        let frame = self.frame as i16;
        let stride = (HIO_M06..=HIO_M0A).contains(&frame) || (HIO_M08..=HIO_M0C).contains(&frame);
        if !stride {
            return;
        }
        self.speed_target = HIO_M10;
        self.turn_step = WALK_TURN;
        if frame == HIO_M06 || frame == HIO_M08 {
            events.list.push(SimEvent::Se(SE_WALK));
        }
    }

    fn sway(&self) -> i16 {
        (SWAY_AMPLITUDE * stream::sin_s(self.tick.wrapping_mul(SWAY_FREQ) as i16)) as i16
    }

    fn hold(&mut self, span: f32, base: f32) {
        self.timers[0] = (rnd_f(&mut self.seed, span) + base) as i16;
    }

    fn mode(&mut self, dist: f32, switches: &Switches, events: &mut SimEvents) {
        self.weapon_at = 0;
        self.frame += 1;
        match self.action {
            ACT_GROUND_WAIT => self.ground_wait(dist, switches, events),
            ACT_KAN_WAIT => self.kan_wait(dist, switches),
            ACT_BREAK_WAIT => self.break_wait(events),
            ACT_FIGHT => self.fight(dist, events),
            _ => self.jyunkai(dist, events),
        }
    }

    fn jyunkai(&mut self, dist: f32, events: &mut SimEvents) {
        self.speed_target = PATROL_SPEED;
        self.speed_step = PATROL_SPEED_STEP;
        let walking = self.state == PATROL_WALK;
        if self.timers[0] == 0 {
            self.state = if walking { PATROL_WAIT } else { PATROL_WALK };
            self.frame = 0;
            self.hold(PATROL_SPAN, PATROL_BASE);
        }
        if walking {
            self.gait(events);
        }
        if self.timers[1] == 0 && dist < AGGRO_RANGE {
            self.action = ACT_FIGHT;
            self.state = TRIGGER_WAIT;
            self.fight = FIGHT_IDLE_RIGHT;
            self.swing = 0.0;
            self.hold(FIGHT_HOLD_SPAN, FIGHT_HOLD_BASE);
        }
    }

    fn end_fight(&mut self) {
        self.action = ACT_JYUNKAI;
        self.state = PATROL_WAIT;
        self.frame = 0;
        self.timers[1] = rnd_f(&mut self.seed, ENGAGE_SPAN) as i16;
    }

    fn fight(&mut self, dist: f32, events: &mut SimEvents) {
        self.turn_step = FIGHT_TURN;
        let mut target = SWING_REST;
        let mut rate = SWING_RATE_SLOW;
        match self.fight {
            FIGHT_SPIN_LEFT | FIGHT_SPIN_RIGHT => {
                let left = self.fight == FIGHT_SPIN_LEFT;
                self.speed_target = if left { -SPIN_CHARGE } else { SPIN_CHARGE };
                self.lean = self.sway();
                target = if left { SWING_LEFT } else { SWING_RIGHT };
                rate = SWING_RATE_FAST;
                if self.timers[0] == 0 && spin_open(self.spin_angle) {
                    self.fight = if left {
                        FIGHT_IDLE_LEFT
                    } else {
                        FIGHT_IDLE_RIGHT
                    };
                }
            }
            _ => {
                self.speed_target = RETREAT_SPEED;
                self.lean = self.sway();
                if self.timers[0] == 0 {
                    if rnd(&mut self.seed) < RND_HALF || dist < RETREAT_RANGE {
                        self.end_fight();
                    } else {
                        let right = self.fight == FIGHT_IDLE_RIGHT;
                        self.fight = if right {
                            FIGHT_SPIN_LEFT
                        } else {
                            FIGHT_SPIN_RIGHT
                        };
                        self.hold(FIGHT_HOLD_SPAN, FIGHT_HOLD_BASE);
                    }
                }
            }
        }
        walk::add_calc(&mut self.swing, target, SPEED_SCALE, rate, 0.0);
        self.spin_angle = self
            .spin_angle
            .wrapping_add((SPIN_STEP * self.swing) as i16);
        if self.swing.abs() > SWING_DEADZONE {
            let before = self.swing_sound;
            self.swing_sound = before.wrapping_add((SPIN_STEP * self.swing.abs()) as i16);
            if self.swing_sound >= SWING_SOUND_GATE && before < SWING_SOUND_GATE {
                events.list.push(SimEvent::Se(SE_CLUB_SWING_L));
            }
        } else {
            self.swing_sound = 0;
        }
        if self.swing.abs() > AT_DEADZONE {
            self.weapon_at = 2;
        }
    }

    fn ground_wait(&mut self, dist: f32, switches: &Switches, events: &mut SimEvents) {
        self.armour = 2;
        match self.state {
            TRIGGER_WAIT => self.trigger(dist, switches),
            TRIGGER_RISE => {
                self.ambush_switch = 0;
                events.list.push(SimEvent::Se(SE_OUT_OF_GROUND));
                self.state = TRIGGER_HOLD;
                self.timers[0] = GROUND_RISE_FRAMES;
            }
            _ => {}
        }
    }

    fn kan_wait(&mut self, dist: f32, switches: &Switches) {
        self.armour = 2;
        match self.state {
            TRIGGER_WAIT => self.trigger(dist, switches),
            TRIGGER_RISE => {
                self.ambush_switch = 0;
                self.state = TRIGGER_HOLD;
                self.timers[0] = HIO_M18;
                self.timers[2] = HIO_M18 + COFFIN_HOLD;
            }
            TRIGGER_HOLD => {
                if self.timers[0] == 0 {
                    self.state = TRIGGER_STAND;
                }
            }
            _ => {}
        }
    }

    fn trigger(&mut self, dist: f32, switches: &Switches) {
        if self.ambush_switch != NO_SWITCH {
            if switches.test(self.ambush_switch - 1) {
                self.state = TRIGGER_RISE;
            }
        } else if dist < self.ambush_reach() {
            self.state = TRIGGER_RISE;
        }
    }

    fn break_wait(&mut self, events: &mut SimEvents) {
        self.speed_target = IDLE_SPEED;
        self.armour = 2;
        match self.state {
            BREAK_START => {
                self.state = BREAK_HOLD;
                self.timers[0] = HIO_M1A;
            }
            BREAK_HOLD => {
                if self.timers[0] == 0 {
                    self.state = BREAK_REGEN;
                    events.list.push(SimEvent::Se(SE_RECOVER_BODY));
                }
            }
            BREAK_REGEN => {
                if self.limbs >= LIMB_FULL {
                    self.state = BREAK_SETTLE;
                    self.timers[0] = BREAK_SETTLE_FRAMES;
                }
            }
            _ => {
                if self.timers[0] == 0 {
                    self.action = ACT_JYUNKAI;
                    self.state = PATROL_WAIT;
                    self.frame = 0;
                    self.timers[1] = rnd_f(&mut self.seed, ENGAGE_SPAN) as i16;
                }
            }
        }
    }

    fn steer(&mut self, actor: &mut Actor, target: Vec3, solid: Option<&Collision>) {
        if ground::grounded(solid, actor.pos) && self.turn_step != 0 {
            let bearing = walk::to_s16((target.x - actor.pos.x).atan2(target.z - actor.pos.z));
            stream::add_calc_angle_s2(&mut actor.angle[1], bearing, TURN_DIV, self.turn_step);
            self.turn_step = 0;
        }
        let heading = actor.angle[1].wrapping_add(self.lean);
        let (sin, cos) = walk::from_s16(heading).sin_cos();
        let step = Vec3::new(sin * self.speed_f, 0.0, cos * self.speed_f);
        walk::add_calc(
            &mut self.speed_f,
            self.speed_target,
            SPEED_SCALE,
            self.speed_step,
            0.0,
        );
        self.speed_step = SPEED_STEP;
        actor.pos += step;
        actor.pos.y += self.speed_y;
        self.speed_y += GRAVITY;
    }

    fn settle(&self, actor: &mut Actor, solid: Option<&Collision>) {
        if let Some(floor) = ground::surface(solid, actor.pos)
            && actor.pos.y < floor
        {
            actor.pos.y = floor;
        }
    }

    fn swing_at(&self, actor: &Actor, player: &Player, health: &mut Health) {
        if self.weapon_at == 0 || self.armour != 0 {
            return;
        }
        let scale = if self.weapon_at == 2 {
            self.swing.abs() * AT_SPIN_GAIN
        } else {
            1.0
        };
        let radius = if self.tick & 1 == 0 {
            scale * AT_RADIUS_EVEN
        } else {
            scale * AT_RADIUS_ODD
        };
        if walk::within(actor.pos, player.pos, radius) {
            health.damage(AT_MACE_POWER);
        }
    }

    fn tail(&mut self) {
        self.lean = 0;
        stream::add_calc_angle_s2(
            &mut self.spin_angle,
            0,
            SPIN_DECAY_DIV,
            SPIN_DECAY_STEP,
        );
    }
}

pub fn is_stalfos(name: &str) -> bool {
    STAL_ACTORS.contains(&name)
}

pub fn control(
    collision: Option<Res<Collision>>,
    switches: Res<Switches>,
    players: Query<&Player>,
    mut health: ResMut<Health>,
    mut events: ResMut<SimEvents>,
    mut stalfos: Query<(&mut Actor, &mut Stalfos)>,
) {
    let Some(player) = players.iter().next().copied() else {
        return;
    };
    let solid = collision.as_deref();
    for (mut actor, mut st) in &mut stalfos {
        let actor = &mut *actor;
        let st = &mut *st;
        st.tick = st.tick.wrapping_add(1);
        for slot in st.timers.iter_mut() {
            if *slot != 0 {
                *slot -= 1;
            }
        }
        let dist = (player.pos - actor.pos).length();
        st.mode(dist, &switches, &mut events);
        if st.action != ACT_KAN_WAIT {
            st.steer(actor, player.pos, solid);
        }
        st.settle(actor, solid);
        st.swing_at(actor, &player, &mut health);
        st.tail();
    }
}