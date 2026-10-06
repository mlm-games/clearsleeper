use bevy_ecs::prelude::*;
use glam::Vec3;

/// `actor_place` from `f_op_actor.h`: a position, an XYZ angle triple and a room number.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Place {
    pub pos: Vec3,
    pub angle: [i16; 3],
    pub room: i8,
}

impl Place {
    pub fn of(pos: Vec3, angle: [i16; 3], room: i8) -> Self {
        Self { pos, angle, room }
    }
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct Actor {
    pub name: String,
    pub parameters: u32,
    pub pos: Vec3,
    pub angle: [i16; 3],
    pub set_id: u16,
    pub room: i8,
    pub old: Place,
    pub shape_angle: [i16; 3],
    pub scale: Vec3,
    pub speed: Vec3,
    pub speed_f: f32,
    pub eye: Vec3,
}

impl Default for Actor {
    fn default() -> Self {
        Self {
            name: String::new(),
            parameters: 0,
            pos: Vec3::ZERO,
            angle: [0; 3],
            set_id: 0,
            room: 0,
            old: Place::default(),
            shape_angle: [0; 3],
            scale: Vec3::ONE,
            speed: Vec3::ZERO,
            speed_f: 0.0,
            eye: Vec3::ZERO,
        }
    },
    ..Default::default()
}

impl Actor {
    pub fn place(&self) -> Place {
        Place::of(self.pos, self.angle, self.room)
    }

    pub fn set_place(&mut self, place: Place) {
        self.pos = place.pos;
        self.angle = place.angle;
        self.room = place.room;
    }

    /// `fopAc_Execute` copies `current` into `old` immediately before the actor method runs,
    /// so every read of `old` sees the transform from the start of that step.
    pub fn snapshot(&mut self) {
        self.old = self.place();
    }

    /// `actor->current.pos = actor->old.pos + actor->speed`.
    pub fn integrate(&mut self) {
        self.pos += self.speed;
    }

    pub fn fall(&mut self, gravity: f32, max_fall_speed: f32) {
        self.speed.y = (self.speed.y - gravity).max(-max_fall_speed);
    }

    /// `actor->current.angle.x = -cM_atan2s(speed.y, speed.absXZ())`.
    pub fn pitch_from_speed(&mut self) {
        let flat = self.speed.x.hypot(self.speed.z);
        self.angle[0] = -atan2s(self.speed.y, flat);
    }

    /// `actor->current.angle.y = cM_atan2s(speed.x, speed.z)`.
    pub fn yaw_from_speed(&mut self) {
        self.angle[1] = atan2s(self.speed.x, self.speed.z);
    }
}

/// `cM_atan2s`: angle in the fixed-point 0..0xFFFF circle scaled from radians.
pub fn atan2s(y: f32, x: f32) -> i16 {
    (y.atan2(x) * (32768.0 / core::f32::consts::PI)).round() as i16
}