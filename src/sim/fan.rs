//! Windfall Island's windmill fans - `fpcNm_FAN_e`, `d_a_fan.cpp`.
//!
//! (a) inbound damage: NONE. (b) outbound damage: NONE modelled. `daFan_c` registers one
//! `dCcD_Cps` whose `SrcObjAt Type` is `AT_TYPE_WIND` (`d_a_fan.cpp:33`) with
//! `cCcD_AtSPrm_Set_e | cCcD_AtSPrm_GrpAll_e` (`:35`) - it is a wind EMITTER, not a damage
//! source, and `AT_TYPE_WIND` collider semantics have no target in this sim. There is no
//! `dCcD` Tg or Co collider anywhere in the file, so nothing can hit a fan: neither
//! `combat::HitPoints` nor `enemy::Hurt` is attached.

use bevy_ecs::prelude::*;

use super::flags::Switches;

/// `d_stage.cpp:614-616` binds `Hsen1`, `Hsen2` and `Hsen3` to `fpcNm_FAN_e`, the profile of
/// `d_a_fan.cpp:297-312`. All three share the archive `"Hsen1"` (`d_a_fan.cpp:17`) and are told
/// apart by the 2-bit type the parameter carries (`d_a_fan.h:56`).
pub const FAN_ACTORS: [&str; 3] = ["Hsen1", "Hsen2", "Hsen3"];

const TYPE_SHIFT: u32 = 8; // <- d_a_fan.h:56 (daFan_prm::getType)
const TYPE_MASK: u32 = 0x03; // <- d_a_fan.h:56 (daFan_prm::getType)
const SWITCH_MASK: u32 = 0xFF; // <- d_a_fan.h:57 (daFan_prm::getSwitchNo)

/// `daFan_c::m_fan_speed` (`d_a_fan.cpp:24`), indexed by the 2-bit type. Every entry is non-zero,
/// so the `len = mFanSpeed / (f32)speed` division at `:214` is always defined.
const FAN_SPEED: [i16; 3] = [0x2000, 0x2000, 0x1D4C]; // <- d_a_fan.cpp:24
/// The `cLib_chaseS` step of all three arms (`d_a_fan.cpp:217`, `:221`, `:224`).
const SPIN_CHASE: i16 = 100; // <- d_a_fan.cpp:217, :221, :224

/// `fopAcM_CULLBOX_CUSTOM_e` (`d_a_fan.cpp:311`) is filled from `m_cull_min[mType]` /
/// `m_cull_max[mType]` by the explicit `fopAcM_setCullSizeBox` at `:113`, so the box is real and
/// per-type.
const CULL_MIN: [[f32; 3]; 3] = [
    [-200.0, -200.0, -50.0],  // <- d_a_fan.cpp:26 (m_cull_min[0])
    [-200.0, -200.0, -50.0],  // <- d_a_fan.cpp:26 (m_cull_min[1])
    [-600.0, -600.0, -50.0],  // <- d_a_fan.cpp:26 (m_cull_min[2])
]; // <- d_a_fan.cpp:26
const CULL_MAX: [[f32; 3]; 3] = [
    [200.0, 200.0, 1600.0],  // <- d_a_fan.cpp:27 (m_cull_max[0])
    [200.0, 200.0, 1600.0],  // <- d_a_fan.cpp:27 (m_cull_max[1])
    [600.0, 600.0, 3500.0],  // <- d_a_fan.cpp:27 (m_cull_max[2])
]; // <- d_a_fan.cpp:27

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fan {
    /// `mType` (`d_a_fan.h:30`), written by `daFan_prm::getType` (`d_a_fan.h:56`) inside `_create`
    /// (`d_a_fan.cpp:159`) and the only discriminator `Execute` branches on.
    kind: u8,
    /// `mSwitchNo` (`d_a_fan.h:29`), written at `d_a_fan.cpp:122`.
    switch_no: u8,
    /// `mFanSpeed` (`d_a_fan.h:33`), the chased rotor rate. `mFanAngle` (`d_a_fan.h:32`) is a
    /// second accumulator, but it is read only by `nodeCallBack` (`:145-148`) for the joint
    /// matrix, so it is renderer state and is not kept.
    spin: i16,
}

impl Fan {
    /// `daFan_c::_create` (`:156-175`) and `Create` (`:108-135`): the only parameter reads are
    /// `daFan_prm::getType` (`d_a_fan.h:56`) and `daFan_prm::getSwitchNo` (`d_a_fan.h:57`).
    /// `fopAcM_ct` (`:157`) leaves `mFanSpeed` at 0, which is also the `f32 speed` initial value
    /// `Execute` chases away from.
    pub fn spawn(parameters: u32) -> Self {
        Self {
            kind: ((parameters >> TYPE_SHIFT) & TYPE_MASK) as u8,
            switch_no: (parameters & SWITCH_MASK) as u8,
            spin: 0,
        }
    }

    /// `fopAcM_setCullSizeBox(this, cullMin.x, ..., cullMax.z)` (`d_a_fan.cpp:113`) with
    /// `m_cull_min` / `m_cull_max` from `:26-27`.
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        let index = usize::from(self.kind) % CULL_MIN.len();
        (CULL_MIN[index], CULL_MAX[index])
    }

    /// `daFan_c::Execute` (`:212-246`), rotor branch only. `speed = m_fan_speed[mType]` (`:213`)
    /// and the three arms are the `if / else if / else` at `:216-226`; `fopAcM_isSwitch(this,
    /// mSwitchNo)` (`:216`) is `dComIfGs_isSwitch(sw, fopAcM_GetHomeRoomNo(pActor))`
    /// (`f_op_actor_mng.h:374-376`), i.e. `Switches::test`.
    fn step(&mut self, blown: bool) {
        let speed = FAN_SPEED[usize::from(self.kind) % FAN_SPEED.len()];
        if (self.kind == 0 && !blown) || (self.kind == 1 && blown) {
            chase_s(&mut self.spin, speed, SPIN_CHASE);
        } else if self.kind == 2 {
            chase_s(&mut self.spin, speed, SPIN_CHASE);
        } else {
            chase_s(&mut self.spin, 0, SPIN_CHASE);
        }
    }
}

/// `cLib_chaseS` (`c_lib.cpp:259-274`). The `step * (*pValue - target) >= 0` test is on `s16`
/// operands in retail and promotes to `int` here, per RULE 10.
fn chase_s(value: &mut i16, target: i16, step: i16) -> bool {
    if step != 0 {
        let step = if *value > target { -step } else { step };
        *value = value.wrapping_add(step);
        if i32::from(step) * i32::from(value.wrapping_sub(target)) >= 0 {
            *value = target;
            return true;
        }
    } else if *value == target {
        return true;
    }
    false
}

pub fn is_fan(name: &str) -> bool {
    FAN_ACTORS.contains(&name)
}

pub fn control(switches: Res<Switches>, mut fans: Query<&mut Fan>) {
    for mut fan in &mut fans {
        let blown = switches.test(fan.switch_no);
        fan.step(blown);
    }
}
