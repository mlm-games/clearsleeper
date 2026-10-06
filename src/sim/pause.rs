use std::time::Duration;

use bevy_ecs::prelude::*;

use super::damage_resolve::HitStop;

/// `cLib_calcTimer` (`SSystem/SComponent/c_lib.h:85`): step `value` down unless it is already zero,
/// then hand it back. The zero guard is what floors a countdown at zero instead of letting it run
/// negative, and the retail field is an `s8`, so the step itself wraps.
pub fn c_lib_calc_timer(value: &mut i8) -> i8 {
    if *value != 0 {
        *value = (*value).wrapping_sub(1);
    }
    *value
}

/// `dScnPly_ply_c::pauseTimer` and `nextPauseTimer` (`d/d_s_play.h:26`): the hit-stop every process
/// in the scene gates on. A hit asks for a length through [`Pause::next_pause_timer`] and spends it
/// one frame at a time out of [`Pause::pause_timer`].
#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Pause {
    /// `pauseTimer`: the frames of execute still to halt.
    pub pause_timer: i8,
    /// `nextPauseTimer`: the length a hit just asked for, which takes hold on the next
    /// `calcPauseTimer` rather than on the frame that asks.
    pub next_pause_timer: i8,
    /// Wall time a host is still holding a freeze for. Retail spends the countdown inside the
    /// halted frame, one sim tick at a time; a host that stops the whole fixed step has no tick
    /// left to spend it on, so the hold carries the remainder in wall time and a freeze lasts the
    /// same whatever the display rate.
    pub hold: Duration,
}

impl Pause {
    /// `dScnPly_ply_c::isPause` (`d/d_s_play.h:17`): the debug build ORs both counters and the
    /// retail build reads `pauseTimer` alone, and the two agree at the gate because
    /// [`Pause::calc_pause_timer`] empties `next_pause_timer` before anything reads it.
    pub fn is_paused(&self) -> bool {
        (self.pause_timer | self.next_pause_timer) != 0
    }

    /// `dScnPly_ply_c::setPauseTimer` (`d/d_s_play.h:21`).
    pub fn set_pause_timer(&mut self, time: i8) {
        self.next_pause_timer = time;
    }

    /// `dScnPly_ply_c::calcPauseTimer` (`d/d_s_play.cpp:58`): a pending request is adopted whole and
    /// spends nothing this frame, otherwise one frame of the countdown is spent.
    pub fn calc_pause_timer(&mut self) -> i8 {
        if self.next_pause_timer != 0 {
            self.pause_timer = self.next_pause_timer;
            self.next_pause_timer = 0;
            return self.pause_timer;
        }
        c_lib_calc_timer(&mut self.pause_timer)
    }
}

/// The host's per-frame pause gate: `true` while the frame is held and the fixed step must not
/// advance. Retail runs `calcPauseTimer` above the execute gate that reads `isPause`
/// (`f_op/f_op_actor.cpp:175`), so a request becomes a hold worth the sim ticks it asked for and
/// then runs down as wall time passes; the request a damage site left in [`HitStop`] is picked up
/// on the way through, the same field retail `setPauseTimer` writes. Halting the step is the same
/// observable result as retail halting actors, camera, message and environment together
/// (`f_op/f_op_camera.cpp:27`, `f_op/f_op_msg.cpp:23`, `f_op/f_op_kankyo.cpp:30`) while the frame
/// keeps drawing.
pub fn advance(world: &mut World, dt: Duration, step: Duration) -> bool {
    let request = world
        .get_resource::<HitStop>()
        .map_or(0, |stop| stop.pause_timer);
    if request != 0 {
        world.resource_mut::<Pause>().set_pause_timer(request);
        world.resource_mut::<HitStop>().pause_timer = 0;
    }
    let mut pause = world.resource_mut::<Pause>();
    if pause.hold > Duration::ZERO {
        pause.hold = pause.hold.saturating_sub(dt);
        return true;
    }
    let frames = pause.calc_pause_timer();
    if frames <= 0 {
        return false;
    }
    pause.pause_timer = 0;
    pause.hold = step * u32::from(frames as u8);
    true
}
