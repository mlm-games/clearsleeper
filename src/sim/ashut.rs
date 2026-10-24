use bevy_ecs::prelude::*;

use super::actor::Actor;
use super::event::{ActiveEvent, EventList, Stage};
use super::event_manager::EventManager;
use super::events::{SimEvent, SimEvents};
use super::flags::Switches;
use super::player::Player;
use super::walk;

/// `OBJNAME("Ashut", fpcNm_Obj_Ashut_e, 255, 0)` (`d_stage.cpp:1094`) - the Pirate Ship rope
/// minigame's metal bars.
pub const ASHUT_ACTORS: [&str; 1] = ["Ashut"];

/// `enum Prm_e { PRM_SWSAVE_W = 8, PRM_SWSAVE_S = 0, PRM_EVID_W = 8, PRM_EVID_S = 8 }`
/// (`d_a_obj_ashut.h:10`), fed through `daObj::PrmAbstract` (`d_a_obj.h:14-17`).
const SWSAVE_MASK: u32 = 0xFF; // <- d_a_obj_ashut.h:10 (PRM_SWSAVE_W)
const SWSAVE_SHIFT: u32 = 0x00; // <- d_a_obj_ashut.h:10 (PRM_SWSAVE_S)
const EVID_MASK: u32 = 0xFF; // <- d_a_obj_ashut.h:10 (PRM_EVID_W)
const EVID_SHIFT: u32 = 0x08; // <- d_a_obj_ashut.h:10 (PRM_EVID_S)

/// `L_attr` (`d_a_obj_ashut.cpp:31-34`): the bar's travel and its fall / rise integrators.
const HEIGHT: f32 = 250.0; // <- d_a_obj_ashut.cpp:32
const FALL_ACCEL: f32 = -2.5; // <- d_a_obj_ashut.cpp:32
const FALL_DRAG: f32 = 0.05; // <- d_a_obj_ashut.cpp:32
const FALL_BOUNCES: u8 = 5; // <- d_a_obj_ashut.cpp:32
const FALL_BOUNCE: f32 = -0.5; // <- d_a_obj_ashut.cpp:32
const FALL_MAX_BOUNCE: f32 = 7.0; // <- d_a_obj_ashut.cpp:32
const RISE_ACCEL: f32 = 0.8; // <- d_a_obj_ashut.cpp:33
const RISE_DRAG: f32 = 0.1; // <- d_a_obj_ashut.cpp:33
const RISE_SLOW_DRAG: f32 = 0.4; // <- d_a_obj_ashut.cpp:33
const RISE_TIME: i16 = 14; // <- d_a_obj_ashut.cpp:33
const RISE_SLOW_TIME: i16 = 4; // <- d_a_obj_ashut.cpp:33
const RISE_BOUNCES: u8 = 5; // <- d_a_obj_ashut.cpp:33
const RISE_BOUNCE: f32 = -0.6; // <- d_a_obj_ashut.cpp:33
const RISE_MAX_BOUNCE: f32 = -4.0; // <- d_a_obj_ashut.cpp:33

/// `fopAcM_setCullSizeBox(this, -80.0f, -5.0f, -30.0f, 80.0f, 250.0f, 30.0f)` (`:59`).
const CULL_MIN: [f32; 3] = [-80.0, -5.0, -30.0]; // <- d_a_obj_ashut.cpp:59
const CULL_MAX: [f32; 3] = [80.0, 250.0, 30.0]; // <- d_a_obj_ashut.cpp:59

/// `JA_SE_OBJ_P_SHIP_SHTR_OP = 0x699D` (`JAZelAudio_SE.h:1937`), started at `:173`.
const SE_SHUTTER_OPEN: u32 = 0x699D; // <- JAZelAudio_SE.h:1937
/// `JA_SE_OBJ_P_SHIP_SHTR_CL = 0x699E` (`JAZelAudio_SE.h:1938`), started at `:137`.
const SE_SHUTTER_CLOSE: u32 = 0x699E; // <- JAZelAudio_SE.h:1938

/// `Mode_e { Mode_UPPER, Mode_U_L, Mode_LOWER, Mode_L_U, Mode_DEMOREQ, Mode_NONE }`
/// (`d_a_obj_ashut.h:9`).
const MODE_UPPER: u8 = 0; // <- d_a_obj_ashut.h:9
const MODE_U_L: u8 = 1; // <- d_a_obj_ashut.h:9
const MODE_LOWER: u8 = 2; // <- d_a_obj_ashut.h:9
const MODE_L_U: u8 = 3; // <- d_a_obj_ashut.h:9
const MODE_DEMOREQ: u8 = 4; // <- d_a_obj_ashut.h:9
const MODE_NONE: u8 = 5; // <- d_a_obj_ashut.h:9

/// The bars over the Pirate Ship's rope minigame - `daObjAshut::Act_c`
/// (`d_a_obj_ashut.h:7-50`). When the switch drops, the bar falls under gravity with the
/// `L_attr` drag and bounce integrators, and rises back with its own accel / drag pair when the
/// switch rises. Any change of state first passes through the demo request, which asks the
/// stage event named by `prm_get_evId()` (`:61`, `:215`).
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. The actor is a `dBgS_MoveBgActor`
/// (`d_a_obj_ashut.h:5`); its only geometry is `mpBgW`, a moving background, which RULE 3b
/// lists among the absent inputs. No `dCcD` collider is registered, so neither `HitPoints` nor
/// `Hurt` is attached.
///
/// `Mthd_IsDelete` forwards to `MoveBGIsDelete` (`:272`), the base `IsDelete()` = `TRUE`, so it
/// accepts a delete request rather than self-destructing. `Mthd_Create` can only fail in
/// resource / heap load (`:68-77`), so `spawn` never refuses.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Ashut {
    /// `prm_get_swSave()` (`d_a_obj_ashut.h:14`) - true raises the bars.
    pub switch_no: u8,
    /// `prm_get_evId()` (`d_a_obj_ashut.h:13`) - the event-info index for the demo request.
    pub event_id: u8,
    /// `mEventIdx` (`d_a_obj_ashut.h:48`), resolved in `Create` (`:61`); `-1` is a miss.
    pub event_idx: i32,
    /// `mMode` (`d_a_obj_ashut.h:42`), one of [`MODE_UPPER`] .. [`MODE_NONE`].
    pub mode: u8,
    /// `mOffset` (`d_a_obj_ashut.h:43`), the bar's vertical offset from `current.pos`.
    pub offset: f32,
    /// `mSpeed` (`d_a_obj_ashut.h:44`), the integrator velocity feeding `mOffset`.
    pub speed: f32,
    /// `mBounces` (`d_a_obj_ashut.h:45`), how many bounce replies remain.
    pub bounces: u8,
    /// `mTimer` (`d_a_obj_ashut.h:46`), the rise's slow-drag countdown.
    pub timer: i16,
    /// `mDemoAccepted` (`d_a_obj_ashut.h:47`) - set once an event was accepted and never cleared.
    pub demo_accepted: bool,
    /// `mDemoNext` (`d_a_obj_ashut.h:49`), which travel the pending request is for.
    pub demo_next: u8,
    /// One-shot guard for `Create`'s event lookup, which needs the [`EventManager`].
    pub entered: bool,
}

impl Ashut {
    /// `Create` (`d_a_obj_ashut.cpp:55-66`), minus the renderer, the cull box (carried by
    /// [`Ashut::cull`]) and the event lookup, which needs the stage event table.
    pub fn spawn(parameters: u32, switches: &Switches) -> Self {
        let switch_no = ((parameters >> SWSAVE_SHIFT) & SWSAVE_MASK) as u8;
        let (mode, offset) = if switches.test(switch_no) {
            (MODE_UPPER, HEIGHT)
        } else {
            (MODE_LOWER, 0.0)
        };
        Self {
            switch_no,
            event_id: ((parameters >> EVID_SHIFT) & EVID_MASK) as u8,
            event_idx: -1,
            mode,
            offset,
            speed: 0.0,
            bounces: 0,
            timer: 0,
            demo_accepted: false,
            demo_next: MODE_NONE,
            entered: false,
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `mEventIdx = dComIfGp_evmng_getEventIdx(NULL, prm_get_evId())` (`:61`).
    fn resolve_event(&mut self, manager: &EventManager) {
        self.event_idx = manager.get_event_idx(None, self.event_id);
    }

    /// `fopAcM_orderOtherEventId(this, mEventIdx, prm_get_evId())` (`:215`), guarded so an
    /// already-running slot is not re-ordered (`d_event_manager.cpp:367`).
    fn order(&self, list: &EventList, active: &mut ActiveEvent, entity: Entity) {
        let Some(slot) = usize::try_from(self.event_idx).ok() else {
            return;
        };
        if active.stage_of(slot).is_none() {
            let _ = active.order(list, slot, entity);
        }
    }

    /// `eventInfo.checkCommandDemoAccrpt()` (`:211`) - the `dEvtCmd_INDEMO_e` latch, the
    /// `Stage::Play | Stage::Close` reduction of `ferris.rs:814`.
    fn demo_running(&self, active: &ActiveEvent) -> bool {
        let Some(slot) = usize::try_from(self.event_idx).ok() else {
            return false;
        };
        matches!(
            active.stage_of(slot),
            Some(Stage::Play) | Some(Stage::Close)
        )
    }

    /// `mode_upper_init` (`:119-122`).
    fn upper_init(&mut self) {
        self.mode = MODE_UPPER;
        self.offset = HEIGHT;
    }

    /// `mode_u_l_init` (`:133-138`).
    fn u_l_init(&mut self, events: &mut SimEvents) {
        self.mode = MODE_U_L;
        self.speed = 0.0;
        self.bounces = FALL_BOUNCES;
        events.list.push(SimEvent::Se(SE_SHUTTER_CLOSE));
    }

    /// `mode_lower_init` (`:157-160`).
    fn lower_init(&mut self) {
        self.mode = MODE_LOWER;
        self.offset = 0.0;
    }

    /// `mode_l_u_init` (`:168-174`).
    fn l_u_init(&mut self, events: &mut SimEvents) {
        self.mode = MODE_L_U;
        self.speed = 0.0;
        self.bounces = RISE_BOUNCES;
        self.timer = RISE_TIME;
        events.list.push(SimEvent::Se(SE_SHUTTER_OPEN));
    }

    /// `mode_demoreq_init` (`:196-205`). The two legal targets are the vertical travels; a
    /// missing event falls through to `mode_demoreq`, which starts at once when the event info
    /// does not exist (`:218-220`).
    fn demoreq_init(&mut self, next: u8, events: &mut SimEvents) {
        if self.demo_accepted {
            if next == MODE_U_L {
                self.u_l_init(events);
            } else {
                self.l_u_init(events);
            }
        } else {
            self.mode = MODE_DEMOREQ;
            self.demo_next = next;
        }
    }

    /// `chk_safe_area` (`:108-116`): the player's position rotated into the bar's frame, with
    /// the three box extents the decomp tests. `mDoMtx_stack_c::YrotS(angle)` with
    /// `multVecSR` is the row-vector Y rotation (`sbox.rs:119-124`).
    fn safe_area(&self, actor: &Actor, player: &Player) -> bool {
        let delta = player.pos - actor.pos;
        let (sin, cos) = walk::from_s16(-actor.shape_angle[1]).sin_cos();
        let x = delta.x * cos + delta.z * sin;
        let z = -delta.x * sin + delta.z * cos;
        delta.y.abs() < 100.0 && z.abs() < 55.0 && x.abs() < 105.0
    }

    fn execute(
        &mut self,
        actor: &Actor,
        player: Option<&Player>,
        entity: Entity,
        switches: &Switches,
        manager: &EventManager,
        list: &EventList,
        active: &mut ActiveEvent,
        events: &mut SimEvents,
    ) {
        match self.mode {
            MODE_UPPER => {
                // `mode_upper` (`:125-130`)
                if !switches.test(self.switch_no)
                    && !player.is_some_and(|player| self.safe_area(actor, player))
                {
                    self.demoreq_init(MODE_U_L, events);
                }
            }
            MODE_U_L => {
                // `mode_u_l` (`:141-154`)
                self.speed += FALL_ACCEL;
                self.speed *= 1.0 - FALL_DRAG;
                self.offset += self.speed;
                if self.offset <= 0.0 {
                    if self.bounces == 0 {
                        self.lower_init();
                    } else {
                        self.bounces -= 1;
                        self.offset *= FALL_BOUNCE;
                        self.speed *= FALL_BOUNCE;
                        if self.speed > FALL_MAX_BOUNCE {
                            self.speed = FALL_MAX_BOUNCE;
                        }
                    }
                }
            }
            MODE_LOWER => {
                // `mode_lower` (`:163-165`)
                if switches.test(self.switch_no) {
                    self.demoreq_init(MODE_L_U, events);
                }
            }
            MODE_L_U => {
                // `mode_l_u` (`:177-193`)
                if self.timer > 0 {
                    self.timer -= 1;
                }
                self.speed += RISE_ACCEL;
                if self.timer > 0 && self.timer < RISE_SLOW_TIME {
                    self.speed *= 1.0 - RISE_SLOW_DRAG;
                } else {
                    self.speed *= 1.0 - RISE_DRAG;
                }
                self.offset += self.speed;
                if self.offset >= HEIGHT {
                    if self.bounces == 0 {
                        self.upper_init();
                    } else {
                        self.bounces -= 1;
                        self.offset = HEIGHT + (self.offset - HEIGHT) * RISE_BOUNCE;
                        self.speed *= RISE_BOUNCE;
                        if self.speed < RISE_MAX_BOUNCE {
                            self.speed = RISE_MAX_BOUNCE;
                        }
                    }
                }
            }
            MODE_DEMOREQ => {
                // `mode_demoreq` (`:208-225`)
                let mut start = false;
                if manager.existence(self.event_idx) {
                    if self.demo_running(active) {
                        start = true;
                        self.demo_accepted = true;
                    } else {
                        self.order(list, active, entity);
                    }
                } else {
                    start = true;
                }
                if start {
                    if self.demo_next == MODE_U_L {
                        self.u_l_init(events);
                    } else {
                        self.l_u_init(events);
                    }
                }
            }
            _ => {}
        }
    }
}

pub fn is_ashut(name: &str) -> bool {
    ASHUT_ACTORS.contains(&name)
}

pub fn control(
    manager: Res<EventManager>,
    list: Res<EventList>,
    mut active: ResMut<ActiveEvent>,
    switches: Res<Switches>,
    mut events: ResMut<SimEvents>,
    players: Query<&Player>,
    mut bars: Query<(Entity, &Actor, &mut Ashut)>,
) {
    let player = players.iter().next().copied();
    for (entity, actor, mut bar) in &mut bars {
        let bar = &mut *bar;
        if !bar.entered {
            bar.entered = true;
            bar.resolve_event(&manager);
        }
        bar.execute(
            actor,
            player.as_ref(),
            entity,
            &switches,
            &manager,
            &list,
            &mut active,
            &mut events,
        );
    }
}
