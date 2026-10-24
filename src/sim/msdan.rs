use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::event::{ActiveEvent, EventList, Stage};
use super::event_manager::EventManager;
use super::events::{SimEvent, SimEvents};
use super::flags::Switches;
use super::msdansb;
use super::walk;

/// `OBJNAME("Msdan", fpcNm_Obj_Msdan_e, 255, 0)` (`d_stage.cpp:953`) - the stage string is the
/// parent controller, while its rungs are the separate `MsdanSb` process (`d_stage.cpp:954`,
/// `fpcNm_Obj_MsdanSub_e`). `daObjMsdan::Act_c::M_arcname` is `"Msdan"` (`:11`).
pub const MSDAN_ACTORS: [&str; 1] = ["Msdan"];

/// `enum Prm_e` (`d_a_obj_msdan.h:7-12`), fed through `daObj::PrmAbstract`
/// (`((1 << width) - 1) & (param >> shift)`, `d_a_obj.h:14-17`).
const SWSAVE_WIDTH: u32 = 0x08; // <- d_a_obj_msdan.h:8 (PRM_SWSAVE_W)
const SWSAVE_SHIFT: u32 = 0x00; // <- d_a_obj_msdan.h:9 (PRM_SWSAVE_S)
const SIZE_WIDTH: u32 = 0x01; // <- d_a_obj_msdan.h:10 (PRM_SIZE_W)
const SIZE_SHIFT: u32 = 0x10; // <- d_a_obj_msdan.h:11 (PRM_SIZE_S)
const SOUND_WIDTH: u32 = 0x01; // <- d_a_obj_msdan.h:12 (PRM_SOUND_W)
const SOUND_SHIFT: u32 = 0x12; // <- d_a_obj_msdan.h:13 (PRM_SOUND_S)
const EVENT_WIDTH: u32 = 0x08; // <- d_a_obj_msdan.h:14 (PRM_EVENT_W)
const EVENT_SHIFT: u32 = 0x18; // <- d_a_obj_msdan.h:15 (PRM_EVENT_S)

/// `angle.y += 0x8000` (`d_a_obj_msdan.cpp:21`) - the rungs face the opposite way to the
/// controller.
const HALF_TURN: i16 = -0x8000; // <- d_a_obj_msdan.cpp:21
/// `pos.y += 800.0f` (`:23`) / `pos.y += 400.0f` (`:31`) - the two start heights.
const RISE_SIZED: f32 = 800.0; // <- d_a_obj_msdan.cpp:23
const RISE_PLAIN: f32 = 400.0; // <- d_a_obj_msdan.cpp:31
/// `pos.x += 50.0f * cM_ssin(current.angle.y); pos.z += 50.0f * cM_scos(current.angle.y);`
/// (`:25-26`, `:33-34`) - the spacing between rungs along the controller's heading.
const STEP: f32 = 50.0; // <- d_a_obj_msdan.cpp:25
/// `i < 31` (`:24`, `:32`), the rung count.
const RUNG_COUNT: i32 = 31; // <- d_a_obj_msdan.cpp:24
/// `for (int i = 16; ...)` (`:32`) - the plain stair starts at rung sixteen.
const PLAIN_START: i32 = 16; // <- d_a_obj_msdan.cpp:32
/// `(i << 8)` (`:27`, `:35`) - the child's `objNo` parameter.
const SLOT_SHIFT: u32 = 8; // <- d_a_obj_msdan.cpp:27
/// `(prm_get_size() << 16)` (`:27`, `:35`) - the child's `size` parameter.
const CHILD_SIZE_SHIFT: u32 = 16; // <- d_a_obj_msdan.cpp:27
/// `prm_get_evId() == 0xFF` (`:39`) - the sentinel that selects the named event.
const EVENT_NONE: u8 = 0xFF; // <- d_a_obj_msdan.cpp:39
/// `M_evname[] = "Msdan"` (`:12`, used at `:40`).
const EVENT_NAME: &str = "Msdan"; // <- d_a_obj_msdan.cpp:40
/// `JA_SE_READ_RIDDLE_1 = 0x0806` (`JAZelAudio_SE.h:18`), started at `:67` and `:79`.
const SE_READ_RIDDLE: u32 = 0x0806; // <- JAZelAudio_SE.h:18
/// `fopAcM_create` uses `NO_SET_ID` for a runtime-created actor (`f_op_actor_mng.cpp:82`).
const NO_SET_ID: u16 = 0xFFFF; // <- f_op_actor_mng.cpp:82

/// `mState` (`d_a_obj_msdan.h:20`): the four labels of `Mthd_Execute`'s switch (`:56-90`).
const STATE_IDLE: u8 = 0; // <- d_a_obj_msdan.cpp:48
const STATE_DEMO_WAIT: u8 = 1; // <- d_a_obj_msdan.cpp:71
const STATE_DEMO_RUN: u8 = 2; // <- d_a_obj_msdan.cpp:78
const STATE_DONE: u8 = 3; // <- d_a_obj_msdan.cpp:46

/// The controller of the `Msdan` rising staircase - `daObjMsdan::Act_c` (`d_a_obj_msdan.h:5-22`).
///
/// `Mthd_Create` (`d_a_obj_msdan.cpp:15-52`) builds 31 rungs along its own heading, each a
/// separate `MsdanSb` actor ([`super::msdansb::Msdansb`]), and remembers the event that raises
/// them. The controller itself draws nothing (the draw method is the `Mthd_Draw` stub elsewhere
/// in the TU); only its children have geometry.
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. The class registers no `dCcD` collider
/// and the profile's `Group` is `fopAc_ACTOR_e` (`:149`); the rungs are moving backgrounds. No
/// `combat::HitPoints` / `enemy::Hurt` is attached, so nothing here can be damaged or despawned
/// by combat.
///
/// `Mthd_IsDelete` returns `TRUE` (`:104`) - RULE 6b's "accepts a delete request", not a
/// per-frame self-destruct. `Mthd_Create` can only fail in `dComIfG_resLoad` (`:17`), which is
/// resource loading, so `spawn` never refuses.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Msdan {
    /// `prm_get_swSave()` (`d_a_obj_msdan.h:8`), the switch `Create` reads at `:44` and
    /// `Execute` re-reads at `:60`.
    pub switch_no: u8,
    /// `prm_get_size()` (`d_a_obj_msdan.h:10`) - true builds all 31 rungs from rung 0 and
    /// starts them 800 up; false builds rungs 16..31 from 400 up (`:22-38`).
    pub size: bool,
    /// `prm_get_sound()` (`d_a_obj_msdan.h:12`) - true finishes silently in state 0 (`:64`).
    pub sound: bool,
    /// `prm_get_evId()` (`d_a_obj_msdan.h:14`) - `0xFF` selects the named event, otherwise the
    /// event-info index (`:39-43`).
    pub event_id: u8,
    /// `mEventIdx` (`d_a_obj_msdan.h:19`), resolved in `Create` (`:39-43`). `-1` is
    /// `dComIfGp_evmng_getEventIdx`'s miss value (`ep.rs:60`).
    pub event_idx: i32,
    /// `mState` (`d_a_obj_msdan.h:20`) - `0` idle, `1` demo wait, `2` demo run, `3` done.
    pub state: u8,
    /// One-shot guard for `Create`'s child spawn and switch read, which need a mutable `World`
    /// and the [`Switches`] resource. The same first-tick reduction as `msdan2.rs`.
    pub entered: bool,
}

impl Msdan {
    /// `daObjMsdan::Act_c::Mthd_Create` (`d_a_obj_msdan.cpp:15-52`), minus the resource load
    /// (`:17`) and the deferred placement / switch read.
    pub fn spawn(parameters: u32) -> Self {
        Self {
            switch_no: ((parameters >> SWSAVE_SHIFT) & ((1 << SWSAVE_WIDTH) - 1)) as u8,
            size: ((parameters >> SIZE_SHIFT) & ((1 << SIZE_WIDTH) - 1)) != 0,
            sound: ((parameters >> SOUND_SHIFT) & ((1 << SOUND_WIDTH) - 1)) != 0,
            event_id: ((parameters >> EVENT_SHIFT) & ((1 << EVENT_WIDTH) - 1)) as u8,
            event_idx: -1,
            state: STATE_IDLE,
            entered: false,
        }
    }

    /// `Create`'s event lookup (`:39-43`):
    /// `prm_get_evId() == 0xFF` asks by name, otherwise by the event-info index.
    fn resolve_event(&mut self, manager: &EventManager) {
        self.event_idx = if self.event_id == EVENT_NONE {
            manager.get_event_idx(Some(EVENT_NAME), EVENT_NONE)
        } else {
            manager.get_event_idx(None, self.event_id)
        };
    }

    /// `fopAcM_create(fpcNm_Obj_MsdanSub_e, (i << 8) + prm_get_swSave() + (prm_get_size() << 16),
    /// &pos, current.roomNo, &angle)` (`:27-28`, `:35-36`), once per rung.
    ///
    /// `pos` starts at the controller, is lifted 800 (sized) or 400 (plain), then steps 50
    /// along `current.angle.y` before each create. The child's own angle is `current.angle.y +
    /// 0x8000` (`:21`), and its `home.pos` is the point passed to `fopAcM_create`, which is why
    /// the spawn takes this `pos.y` as `home_y`.
    fn spawn_rungs(&self, actor: &Actor, switches: &Switches, commands: &mut Commands) {
        let (sin, cos) = walk::from_s16(actor.angle[1]).sin_cos();
        let mut pos = Vec3::new(
            actor.pos.x,
            actor.pos.y + if self.size { RISE_SIZED } else { RISE_PLAIN },
            actor.pos.z,
        );
        let mut angle = actor.angle;
        angle[1] = angle[1].wrapping_add(HALF_TURN);
        let start = if self.size { 0 } else { PLAIN_START };
        for slot in start..RUNG_COUNT {
            pos.x += STEP * sin;
            pos.z += STEP * cos;
            let size = if self.size { 1 } else { 0 };
            let parameters = ((slot as u32) << SLOT_SHIFT)
                + u32::from(self.switch_no)
                + (size << CHILD_SIZE_SHIFT);
            commands.spawn((
                Actor {
                    name: msdansb::MSDANSB_ACTORS[0].to_owned(),
                    parameters,
                    pos,
                    angle,
                    set_id: NO_SET_ID,
                    room: actor.room,
                    ..Default::default()
                },
                msdansb::Msdansb::spawn(parameters, pos.y, switches),
            ));
        }
    }

    /// `fopAcM_orderOtherEventId(this, mEventIdx)` (`:70`, `:81`). Re-ordering a slot that is
    /// already ORDER / PLAY / CLOSE rewrites its state (`d_event_manager.cpp:367`), so this
    /// only orders from idle, the `ferris.rs` / `hbrf1.rs` house rule.
    fn order(&self, list: &EventList, active: &mut ActiveEvent, entity: Entity) {
        let Some(slot) = usize::try_from(self.event_idx).ok() else {
            return;
        };
        if active.stage_of(slot).is_none() {
            let _ = active.order(list, slot, entity);
        }
    }

    /// `eventInfo.checkCommandDemoAccrpt()` (`:77`) - the per-actor `dEvtCmd_INDEMO_e` latch,
    /// reduced to `Stage::Play | Stage::Close` on this actor's own slot, as in `ferris.rs:814`.
    fn demo_running(&self, active: &ActiveEvent) -> bool {
        let Some(slot) = usize::try_from(self.event_idx).ok() else {
            return false;
        };
        matches!(
            active.stage_of(slot),
            Some(Stage::Play) | Some(Stage::Close)
        )
    }

    /// `daObjMsdan::Act_c::Mthd_Execute` (`d_a_obj_msdan.cpp:55-92`).
    fn execute(
        &mut self,
        list: &EventList,
        active: &mut ActiveEvent,
        switches: &Switches,
        events: &mut SimEvents,
        entity: Entity,
    ) {
        match self.state {
            STATE_DONE => {}
            STATE_IDLE => {
                if !switches.test(self.switch_no) {
                    return;
                }
                if self.size {
                    self.state = STATE_DONE;
                } else if self.sound {
                    self.state = STATE_DONE;
                } else if self.event_idx == -1 {
                    events.list.push(SimEvent::Se(SE_READ_RIDDLE));
                    self.state = STATE_DONE;
                } else {
                    self.order(list, active, entity);
                    self.state = STATE_DEMO_WAIT;
                }
            }
            STATE_DEMO_WAIT => {
                if self.demo_running(active) {
                    events.list.push(SimEvent::Se(SE_READ_RIDDLE));
                    self.state = STATE_DEMO_RUN;
                } else {
                    self.order(list, active, entity);
                }
            }
            STATE_DEMO_RUN => {
                let Some(slot) = usize::try_from(self.event_idx).ok() else {
                    return;
                };
                if active.end_check(slot) {
                    // `dComIfGp_event_reset();` (`:86`) - the sim's `ActiveEvent::end`.
                    active.end(slot);
                    self.state = STATE_DONE;
                }
            }
            _ => {}
        }
    }
}

pub fn is_msdan(name: &str) -> bool {
    MSDAN_ACTORS.contains(&name)
}

pub fn control(
    manager: Res<EventManager>,
    list: Res<EventList>,
    mut active: ResMut<ActiveEvent>,
    switches: Res<Switches>,
    mut events: ResMut<SimEvents>,
    mut controllers: Query<(Entity, &Actor, &mut Msdan)>,
    mut commands: Commands,
) {
    for (entity, actor, mut controller) in &mut controllers {
        let controller = &mut *controller;
        if !controller.entered {
            controller.entered = true;
            controller.resolve_event(&manager);
            controller.state = if switches.test(controller.switch_no) {
                STATE_DONE
            } else {
                STATE_IDLE
            };
            controller.spawn_rungs(actor, &switches, &mut commands);
        }
        controller.execute(&list, &mut active, &switches, &mut events, entity);
    }
}
