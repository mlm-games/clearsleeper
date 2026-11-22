use bevy_ecs::prelude::*;
use glam::Vec3;

use super::health::Health;
use super::intent::PadButtons;
use super::link_proc::{LinkProc, Proc};
use super::lock::Lock;
use super::player::Player;

/// `mCyl.OnTgShield()` and the slip timer, `checkPlayerGuard()` and `checkSuccessGuard()`
/// (`tww/src/d/actor/d_a_player_main.cpp:1036-1039,4329-4388,10958-10965`;
/// `wwhd/wwhd_src/d/actor/d_a_player_main.cpp:428-436`,
/// `wwhd/wwhd_src/d/actor/d_a_player_main_03.cpp:3987-4050`). A guard is binary: a frontal hit
/// with a normal attack level costs no life and starts a slip process that ignores further hits.
#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct Guard {
    pub raised: bool,
    pub slip: u32,
}

/// `procGuardSlip`'s `mProcVar0` (`wwhd/wwhd_src/d/actor/d_a_player_main_05.cpp:1686-1700`).
pub const SLIP_TICKS: u32 = 20;
/// `dCcG_At_Spl_UNK8`: attacks at or above it ignore the shield (`main.cpp:10960`).
pub const UNGUARDABLE_SPL: u8 = 8;

impl Guard {
    /// `checkSuccessGuard` plus `ChkShieldFrontRange` (`tww/src/d/d_cc_s.cpp:83-107`): the blow
    /// must arrive within ±90° of the player's facing and be a guardable attack level.
    pub fn absorb(&mut self, player: &Player, attacker: Vec3, spl: u8) -> bool {
        if self.slip > 0 {
            return true;
        }
        if !self.raised || spl >= UNGUARDABLE_SPL {
            return false;
        }
        let dx = attacker.x - player.pos.x;
        let dz = attacker.z - player.pos.z;
        if dx * dx + dz * dz <= f32::EPSILON {
            return false;
        }
        let delta = dx.atan2(dz);
        let diff = (delta - player.face + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
            - std::f32::consts::PI;
        if diff.abs() > std::f32::consts::FRAC_PI_2 {
            return false;
        }
        self.slip = SLIP_TICKS;
        true
    }
}

/// `setShieldGuard` (`main.cpp:4329-4337`; HD `d_a_player_main_03.cpp:4014-4015`): the
/// reference's shield-equip and stage gates have no port state, so R held with a lock target
/// stands in for `checkAttentionLock() && checkShieldEquip() && checkGuardAccept()`.
pub fn control(
    pad: Res<PadButtons>,
    lock: Res<Lock>,
    proc: Option<Res<LinkProc>>,
    mut guard: ResMut<Guard>,
) {
    let slipping = matches!(
        proc.as_deref().map(|p| p.current),
        Some(Proc::GuardSlip | Proc::CrouchDefenseSlip | Proc::CrouchDefense)
    );
    guard.raised = guard.slip > 0 || slipping || (pad.hold.r && lock.target.is_some());
    if guard.slip > 0 {
        guard.slip -= 1;
    }
}

/// The single entry a melee damage site calls instead of `Health::damage`. Returns `true`
/// when the blow was absorbed, so the caller can start a slip animation if it has one.
pub fn hurt(
    health: &mut Health,
    guard: &mut Guard,
    player: &Player,
    attacker: Vec3,
    amount: u8,
    spl: u8,
) -> bool {
    if guard.absorb(player, attacker, spl) {
        return true;
    }
    health.damage(amount);
    false
}
