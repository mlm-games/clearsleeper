use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::barricade::AT_BOMB;
use super::boomerang::Boomerang;
use super::events::{SimEvent, SimEvents};
use super::flags::Switches;
use super::item::{Bomb, InFlight};
use super::stream::{add_calc_angle_s2, angle_s, cos_s, sin_s};
use super::walk::{add_calc, from_s16, to_s16, within};

pub const CANON_ACTORS: [&str; 1] = ["Ocanon"];

pub const NO_SWITCH: u8 = 0xFF;

pub const MODE_WAIT: u8 = 0;
pub const MODE_ATTACK: u8 = 1;
pub const MODE_DELETE: u8 = 2;
pub const MODE_SW_WAIT: u8 = 3;
pub const MODE_APPEAR: u8 = 4;

pub const AT_BOOMERANG: u32 = 1 << 6;

pub const SE_WEAPON_HIT: u32 = 0x2833;
pub const SE_LAST_HIT: u32 = 0x2828;
pub const SE_CANNON_FIRE: u32 = 0x2852;

pub const MAX_HEALTH: i32 = 2;
pub const BOMB_DAMAGE: i32 = 2;
pub const BOOMERANG_DAMAGE: i32 = 1;
pub const HIT_COOLDOWN: i16 = 5;

pub const WAIT_FRAMES: i16 = 0x1E;
pub const ATTACK_FRAMES: i16 = 0x1E;

pub const SPH_RADIUS: f32 = 80.0;
pub const SPHERE_REACH: f32 = 60.0;
pub const MUZZLE_REACH: f32 = 180.0;

pub const SHELL_SPEED: f32 = 90.0;
pub const SHELL_FUSE: u32 = 0x96;

pub const LEAD_START: f32 = 5000.0;
pub const LEAD_TOP: i16 = 0x3000;
pub const PITCH_TOP: i16 = 0x2328;
pub const PITCH_FLOOR: i16 = -0x2328;
pub const YAW_TOP: i16 = 0x2328;
pub const YAW_FLOOR: i16 = -0x2328;

pub const AIM_SCALE: i16 = 0x6;
pub const AIM_STEP: i16 = 0x300;

pub const KICK_DECAY: i16 = 0xA;
pub const KICK_STEP: i16 = 0xA;
pub const KICK_FIRE: i16 = 0xC8;
pub const KICK_BOOM: i16 = 0x190;
pub const KICK_APPEAR: i16 = 0x258;

pub const TWIST_STEP: i16 = 0x1830;
pub const TWIST_GAIN: f32 = 5.0;

pub const DROP: f32 = 120.0;
pub const DROP_SCALE: f32 = 0.1;
pub const DROP_MAX: f32 = 1.0;
pub const DROP_MIN: f32 = 1.0;
pub const DROP_TOP: f32 = 2.0;

pub const SCALE_STEP: f32 = 0.1;

pub const CULL_HALF: f32 = 100.0;
pub const CULL_TOP: f32 = 150.0;
pub const CULL_BASE: f32 = -10.0;

const SCALE_SHIFT: u32 = 0;
const SCALE_MASK: u32 = 0xFF;
const APPEAR_SHIFT: u32 = 8;
const APPEAR_MASK: u32 = 0xFF;
const PATH_SHIFT: u32 = 16;
const PATH_MASK: u32 = 0xFF;
const GO_SHIFT: u32 = 24;
const GO_MASK: u32 = 0xFF;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Canon {
    pub mode: u8,
    pub go_switch: u8,
    pub appear_switch: u8,
    pub scale: f32,
    pub health: i32,
    pub cool: i16,
    pub wait: i16,
    pub target: Vec3,
    pub pitch: i16,
    pub pitch_want: i16,
    pub yaw: i16,
    pub yaw_want: i16,
    pub kick: i16,
    pub phase: i16,
    pub drop: f32,
    pub centre: Vec3,
}

impl Canon {
    pub fn spawn(parameters: u32, switches: &Switches) -> Option<Self> {
        let grow = ((parameters >> SCALE_SHIFT) & SCALE_MASK) as u32;
        let appear = ((parameters >> APPEAR_SHIFT) & APPEAR_MASK) as u8;
        let path = ((parameters >> PATH_SHIFT) & PATH_MASK) as u8;
        let go = ((parameters >> GO_SHIFT) & GO_MASK) as u8;
        if path == NO_SWITCH {
            return None;
        }
        let scale = if grow == NO_SWITCH as u32 {
            1.0
        } else {
            1.0 + grow as f32 * SCALE_STEP
        };
        let mut canon = Self {
            mode: MODE_WAIT,
            go_switch: go,
            appear_switch: appear,
            scale,
            health: MAX_HEALTH,
            cool: 0,
            wait: WAIT_FRAMES,
            target: Vec3::ZERO,
            pitch: 0,
            pitch_want: 0,
            yaw: 0,
            yaw_want: 0,
            kick: 0,
            phase: 0,
            drop: 0.0,
            centre: Vec3::ZERO,
        };
        if go != NO_SWITCH && switches.test(go) {
            canon.arm(MODE_DELETE);
        } else if appear != NO_SWITCH && !switches.test(appear) {
            canon.arm(MODE_SW_WAIT);
        } else {
            canon.arm(MODE_WAIT);
        }
        Some(canon)
    }

    pub fn drawn(&self) -> bool {
        self.mode != MODE_DELETE && self.mode != MODE_SW_WAIT
    }

    pub fn hull(&self) -> (f32, f32, f32) {
        (
            CULL_BASE * self.scale,
            (CULL_TOP - CULL_BASE) * self.scale,
            CULL_HALF * self.scale,
        )
    }

    fn arm(&mut self, mode: u8) {
        self.mode = mode;
        match mode {
            MODE_WAIT => self.wait = WAIT_FRAMES,
            MODE_ATTACK => self.wait = ATTACK_FRAMES,
            MODE_DELETE => self.kick = 0,
            MODE_SW_WAIT => self.drop = DROP,
            MODE_APPEAR => {
                self.kick = KICK_APPEAR;
                self.drop = DROP;
            }
            _ => {}
        }
    }

    fn enter(&mut self, mode: u8, switches: &mut Switches) {
        self.arm(mode);
        if mode == MODE_DELETE && self.go_switch != NO_SWITCH {
            switches.set(self.go_switch);
        }
    }

    fn tick_wait(&mut self) -> bool {
        if self.wait != 0 {
            self.wait -= 1;
        }
        self.wait == 0
    }

    fn shot(&self, pos: Vec3, angle: [i16; 3], reach: f32) -> Vec3 {
        let twist = angle_s((self.kick) as f32 * TWIST_GAIN * from_s16(self.phase).sin());
        let mut v = Vec3::new(0.0, reach * self.scale, 0.0);
        v = zrot(v, self.yaw.wrapping_neg());
        v = xrot(v, self.pitch.wrapping_neg());
        v = xrot(v, twist);
        v.y -= self.drop;
        v = xrot(v, angle[0]);
        v = zrot(v, angle[2]);
        v = yrot(v, angle[1]);
        pos + v
    }

    fn lock_on(&mut self, pos: Vec3, angle: [i16; 3]) {
        let delta = self.target - pos;
        let flat = (delta.x * delta.x + delta.z * delta.z).sqrt();
        let mut lead = 0;
        if flat > LEAD_START {
            lead = (flat - LEAD_START) as i16;
            if lead > LEAD_TOP {
                lead = LEAD_TOP;
            }
        }
        let lift = to_s16(delta.y.atan2(flat));
        self.pitch_want = limit(lead.wrapping_add(lift), PITCH_FLOOR, PITCH_TOP);
        self.yaw_want = limit(
            to_s16(delta.x.atan2(delta.z)).wrapping_sub(angle[1]),
            YAW_FLOOR,
            YAW_TOP,
        );
    }

    fn fire(
        &mut self,
        pos: Vec3,
        angle: [i16; 3],
        events: &mut SimEvents,
        commands: &mut Commands,
    ) {
        let pitch = angle[0].wrapping_sub(self.pitch);
        let yaw = angle[1].wrapping_add(self.yaw);
        let launch = self.shot(pos, angle, MUZZLE_REACH);
        let speed = SHELL_SPEED * cos_s(pitch);
        let lift = -(SHELL_SPEED * sin_s(pitch));
        let (sin, cos) = from_s16(yaw).sin_cos();
        self.kick = KICK_FIRE;
        events.list.push(SimEvent::Se(SE_CANNON_FIRE));
        commands.spawn((
            Bomb {
                pos: launch,
                fuse: SHELL_FUSE,
                vel: Vec3::new(speed * sin, lift, speed * cos),
                rest: false,
            },
            InFlight,
        ));
    }

    fn struck(
        &mut self,
        blasts: &[Vec3],
        spins: &[Vec3],
        switches: &mut Switches,
        events: &mut SimEvents,
    ) -> bool {
        if self.cool > 0 {
            self.cool -= 1;
        }
        if self.cool != 0 {
            return false;
        }
        let reach = SPH_RADIUS * self.scale;
        let at = if blasts.iter().any(|hit| within(*hit, self.centre, reach)) {
            AT_BOMB
        } else if spins.iter().any(|hit| within(*hit, self.centre, reach)) {
            AT_BOOMERANG
        } else {
            return false;
        };
        match at {
            AT_BOMB => self.health -= BOMB_DAMAGE,
            AT_BOOMERANG => {
                self.kick = KICK_BOOM;
                events.list.push(SimEvent::Se(SE_WEAPON_HIT));
                self.health -= BOOMERANG_DAMAGE;
            }
            _ => return false,
        }
        self.cool = HIT_COOLDOWN;
        if self.health <= 0 {
            events.list.push(SimEvent::Se(SE_LAST_HIT));
            self.enter(MODE_DELETE, switches);
        } else {
            self.enter(MODE_WAIT, switches);
        }
        true
    }

    fn wait_mode(
        &mut self,
        blasts: &[Vec3],
        spins: &[Vec3],
        switches: &mut Switches,
        events: &mut SimEvents,
    ) {
        if self.struck(blasts, spins, switches, events) {
            return;
        }
        if self.tick_wait() {
            self.pitch_want = 0;
            self.yaw_want = 0;
            self.wait = WAIT_FRAMES;
        }
    }

    fn attack_mode(
        &mut self,
        pos: Vec3,
        angle: [i16; 3],
        blasts: &[Vec3],
        spins: &[Vec3],
        switches: &mut Switches,
        events: &mut SimEvents,
        commands: &mut Commands,
    ) {
        if self.struck(blasts, spins, switches, events) {
            return;
        }
        self.lock_on(pos, angle);
        if self.tick_wait() {
            self.fire(pos, angle, events, commands);
            self.enter(MODE_WAIT, switches);
        }
    }

    fn appear_mode(&mut self, switches: &mut Switches) {
        add_calc(&mut self.drop, 0.0, DROP_SCALE, DROP_MAX, DROP_MIN);
        if self.drop < DROP_TOP {
            self.drop = 0.0;
            self.enter(MODE_WAIT, switches);
        }
    }

    fn execute(
        &mut self,
        pos: Vec3,
        angle: [i16; 3],
        blasts: &[Vec3],
        spins: &[Vec3],
        switches: &mut Switches,
        events: &mut SimEvents,
        commands: &mut Commands,
    ) {
        add_calc_angle_s2(&mut self.kick, 0, KICK_DECAY, KICK_STEP);
        add_calc_angle_s2(&mut self.pitch, self.pitch_want, AIM_SCALE, AIM_STEP);
        add_calc_angle_s2(&mut self.yaw, self.yaw_want, AIM_SCALE, AIM_STEP);
        match self.mode {
            MODE_WAIT => self.wait_mode(blasts, spins, switches, events),
            MODE_ATTACK => self.attack_mode(
                pos, angle, blasts, spins, switches, events, commands,
            ),
            MODE_DELETE => {}
            MODE_SW_WAIT => {
                if self.appear_switch != NO_SWITCH && switches.test(self.appear_switch) {
                    self.enter(MODE_APPEAR, switches);
                }
            }
            MODE_APPEAR => self.appear_mode(switches),
            _ => {}
        }
        self.phase = self.phase.wrapping_add(TWIST_STEP);
        self.centre = self.shot(pos, angle, SPHERE_REACH);
    }
}

fn limit(value: i16, low: i16, high: i16) -> i16 {
    if value < low {
        low
    } else if value > high {
        high
    } else {
        value
    }
}

fn yrot(v: Vec3, units: i16) -> Vec3 {
    let (s, c) = from_s16(units).sin_cos();
    Vec3::new(v.x * c + v.z * s, v.y, v.z * c - v.x * s)
}

fn xrot(v: Vec3, units: i16) -> Vec3 {
    let (s, c) = from_s16(units).sin_cos();
    Vec3::new(v.x, v.y * c - v.z * s, v.y * s + v.z * c)
}

fn zrot(v: Vec3, units: i16) -> Vec3 {
    let (s, c) = from_s16(units).sin_cos();
    Vec3::new(v.x * c - v.y * s, v.x * s + v.y * c, v.z)
}

pub fn is_canon(name: &str) -> bool {
    CANON_ACTORS.contains(&name)
}

pub fn control(
    blasts: Res<super::bombflower::Blasts>,
    booms: Query<&Boomerang>,
    mut switches: ResMut<Switches>,
    mut events: ResMut<SimEvents>,
    mut canons: Query<(&Actor, &mut Canon)>,
    mut commands: Commands,
) {
    let published: Vec<Vec3> = blasts.list.clone();
    let spins: Vec<Vec3> = booms.iter().map(|boom| boom.pos).collect();
    for (actor, mut canon) in &mut canons {
        canon.execute(
            actor.pos,
            actor.angle,
            &published,
            &spins,
            &mut switches,
            &mut events,
            &mut commands,
        );
    }
}