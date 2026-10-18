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
    pub gravity: f32,
    pub max_fall_speed: f32,
    pub eye: Vec3,
    /// `itemTableIdx`, the item table the steal-item create call reads.
    pub item_table_idx: i32,
    /// `stealItemBitNo`, the bit number the steal-item create call sets before incrementing it.
    pub steal_item_bit_no: u8,
    /// `stealItemLeft`, the count of steal items still owed, the value that gates the steal.
    pub steal_item_left: i8,
    /// `cCcD_Stts::GetWeightUc`, the collision weight byte `GetRank` buckets and `rank_tbl`
    /// indexes with, set at the reference's `cCcD_Stts::Init` first argument. `0` and `0xFF` are
    /// the two a correction pair early-outs on, and the reference's own player and boat take 120
    /// and 0xF0. `super::weight` holds the per-process values this starts from.
    pub weight_uc: u8,
    /// `cCcD_ObjCo::ChkNoCrr` (`c_cc_d.h:493`) reading `cCcD_CoSPrm_NoCrr_e` (`c_cc_d.h:57`) on the
    /// process's own collision body, which `dCcS::SetPosCorrect` (`d_cc_s.cpp:237`) returns on for
    /// either side of the pair, before any weight is read. `super::weight` names the processes
    /// whose body carries it from the start; the player raises it only while its in-ship mode
    /// flag is up, so the player's module owns this one.
    pub no_crr: bool,
    /// `demoActorID` (`f_op_actor.h:281`), the one-based index into the demo player's actor table
    /// (`dDemo_object_c::appendActor`, `d_demo.cpp:545-562`). `0` is the field's own default and
    /// the test every demo-reading actor module opens with, e.g. `daNpc_Ba1_c::demo`
    /// (`d_a_npc_ba1.cpp:1785-1786`) and `daDemo00_c::execute` (`d_a_demo00.cpp:636-639`).
    pub demo_actor_id: u8,
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
            gravity: 0.0,
            max_fall_speed: 0.0,
            eye: Vec3::ZERO,
            item_table_idx: 0,
            steal_item_bit_no: 0,
            steal_item_left: 0,
            weight_uc: 100,
            no_crr: false,
            demo_actor_id: 0,
        }
    }
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

    /// `speed.y += gravity`, clamped by `maxFallSpeed`.
    pub fn fall(&mut self) {
        self.speed.y += self.gravity;
        if self.max_fall_speed > 0.0 {
            self.speed.y = self.speed.y.max(-self.max_fall_speed);
        }
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

/// `fopAc_Execute` snapshots every actor before the step's actor methods run, so `old`
/// holds the transform each method starts from.
pub fn snapshot(mut actors: Query<&mut Actor>) {
    for mut actor in &mut actors {
        actor.snapshot();
    }
}
