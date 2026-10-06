use bevy_ecs::prelude::*;

use super::actor::Actor;
use super::events::{SimEvent, SimEvents};
use super::flags::Switches;
use super::walk;

pub const AMIPROP_ACTORS: [&str; 2] = ["Hami1", "HamiY"];

const SWITCH_SHIFT: u32 = 0x00; // <- d_a_amiprop.cpp:50
const SWITCH_MASK: u32 = 0xFF; // <- d_a_amiprop.cpp:50
const TYPE_SHIFT: u32 = 0x08; // <- d_a_amiprop.cpp:54
const TYPE_MASK: u32 = 0x0F; // <- d_a_amiprop.cpp:54

const TYPE_ROLL: u8 = 0; // <- d_a_amiprop.cpp:160
const TYPE_PITCH: u8 = 1; // <- d_a_amiprop.cpp:164

const HALF_TURN: i16 = 0x4000; // <- d_a_amiprop.cpp:111, :170
const CHASE_SCALE: i16 = 10; // <- d_a_amiprop.cpp:161
const CHASE_MAX: i16 = 0x200; // <- d_a_amiprop.cpp:161
const CHASE_MIN: i16 = 0x10; // <- d_a_amiprop.cpp:161

const SE_PROP: u32 = 0x6166; // <- JAZelAudio_SE.h:1695 (JA_SE_OBJ_AMI_PROP)

pub const CULL_MIN: [f32; 3] = [-850.0, -30.0, -850.0]; // <- d_a_amiprop.cpp:105
pub const CULL_MAX: [f32; 3] = [850.0, 30.0, 850.0]; // <- d_a_amiprop.cpp:105

/// The stage strings are `Hami1` and `HamiY` (`d_stage.cpp:623-624`), but the model, DZB and
/// resource archive are all `Hami1` (`daAmiProp_c::m_arcname`, `d_a_amiprop.cpp:46`) - `HamiY`
/// loads the same `Hami1` assets. Both names go in `AMIPROP_ACTORS` because those are the
/// STAGE strings.
///
/// The prop's collider is a degenerate cylinder - radius 0, height 0 (`:41-42`) - with no AT
/// type at all (`:20`), so nothing can damage it and neither `HitPoints` nor `Hurt` is
/// attached.
///
/// `daAmiProp_prm::getType` (`:53-55`) values above `1` reach no `case` of `_execute`'s switch
/// (`:159-167`), so retail reads the uninitialised local `tmp` declared at `:157`. This sim
/// takes no branch there and leaves `m444` set; that is the one reading which invents nothing.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Amiprop {
    pub switch_no: u8,
    pub kind: u8,
    pub read: bool,
    pub swinging: bool,
    pub target: i16,
}

impl Amiprop {
    pub fn spawn(parameters: u32, angle: [i16; 3]) -> Self {
        Self {
            switch_no: ((parameters >> SWITCH_SHIFT) & SWITCH_MASK) as u8,
            kind: ((parameters >> TYPE_SHIFT) & TYPE_MASK) as u8,
            read: false,
            swinging: false,
            target: angle[0].wrapping_add(HALF_TURN),
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `cLib_addCalcAngleS(&current.angle.x|y, unk_446, 10, 0x200, 0x10)` (`:161`, `:165`)
    /// writes through the pointer and returns `target - *pValue`, which `_execute` tests
    /// against zero at `:169`.
    fn chase(&self, angle: &mut [i16; 3]) -> Option<bool> {
        let axis = match self.kind {
            TYPE_ROLL => 0,
            TYPE_PITCH => 1,
            _ => return None,
        };
        let value = walk::add_calc_angle_s(
            angle[axis],
            self.target,
            CHASE_SCALE,
            CHASE_MAX,
            CHASE_MIN,
        );
        angle[axis] = value;
        Some(value == self.target)
    }

    /// `daAmiProp_c::_execute` (`:150-184`) minus `set_mtx` (`:181`), `mpBgW->Move()` (`:182`)
    /// and the `dComIfGp_getVibration().StartShock` rumble (`:173`). Returns true where retail
    /// calls `fopAcM_seStart`.
    fn execute(&mut self, angle: &mut [i16; 3], switches: &Switches) -> bool {
        let is_switch = switches.test(self.switch_no);
        if is_switch != self.read {
            self.swinging = true;
        }
        let mut sound = false;
        if self.swinging {
            match self.chase(angle) {
                Some(true) => {
                    self.target = self.target.wrapping_add(HALF_TURN);
                    self.swinging = false;
                }
                Some(false) => sound = true,
                None => {}
            }
        }
        self.read = is_switch;
        sound
    }
}

pub fn is_amiprop(name: &str) -> bool {
    AMIPROP_ACTORS.contains(&name)
}

pub fn control(
    switches: Res<Switches>,
    mut events: ResMut<SimEvents>,
    mut props: Query<(&mut Actor, &mut Amiprop)>,
) {
    for (mut actor, mut prop) in &mut props {
        let angle = &mut actor.angle;
        if prop.execute(angle, &switches) {
            events.list.push(SimEvent::Se(SE_PROP));
        }
    }
}