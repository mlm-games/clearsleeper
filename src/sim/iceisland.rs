use bevy_ecs::prelude::*;

use super::event::{ActiveEvent, EventList, Stage};
use super::events::{SimEvent, SimEvents};
use super::flags::Switches;

/// The stage string is `GiceL` (`d_stage.cpp:964`, `fpcNm_Obj_Iceisland_e`). It matches the
/// archive name `"GiceL"` that `daObjIceisland_c::_create` loads (`d_a_obj_iceisland.cpp:179`)
/// and releases at `:191`.
pub const ICEISLAND_ACTORS: [&str; 1] = ["GiceL"];

/// `field_0x39C` (`d_a_obj_iceisland.h:35`, an `int`) - the mode `_execute` switches on
/// (`:207-232`). The eight case labels of that switch are the only place the values appear.
const MODE_FREEZE_MAIN: i32 = 0; // <- d_a_obj_iceisland.cpp:208
const MODE_MELT_DEMO_WAIT: i32 = 1; // <- d_a_obj_iceisland.cpp:211
const MODE_MELT_DEMO: i32 = 2; // <- d_a_obj_iceisland.cpp:214
const MODE_MELT_MAIN: i32 = 3; // <- d_a_obj_iceisland.cpp:217
const MODE_FREEZE_DEMO_WAIT: i32 = 4; // <- d_a_obj_iceisland.cpp:220
const MODE_FREEZE_DEMO: i32 = 5; // <- d_a_obj_iceisland.cpp:223
const MODE_FAIL_DEMO_WAIT: i32 = 6; // <- d_a_obj_iceisland.cpp:226
const MODE_FAIL_DEMO_MAIN: i32 = 7; // <- d_a_obj_iceisland.cpp:229

/// `daObjIceisland_prm::getSwitchNo(ac) = (fopAcM_GetParam(ac) >> 8) & 0xFF`
/// (`d_a_obj_iceisland.h:41`) into the `u8` local of `CreateInit` (`:59`), `freeze_main` (`:95`),
/// `melt_main` (`:134`), `fail_demo_wait` (`:164`) and `_execute` (`:236`). `fopAcM_isSwitch(this,
/// swNo)` (`f_op_actor_mng.h:374-376`) is `dComIfGs_isSwitch(swNo, fopAcM_GetHomeRoomNo(this))`,
/// so the home room is implicit and `super::flags::Switches` is the right channel.
const SWITCH_SHIFT: u32 = 8; // <- d_a_obj_iceisland.h:41
const SWITCH_MASK: u32 = 0xFF; // <- d_a_obj_iceisland.h:41

/// `dComIfGp_evmng_getEventIdx("MELT_ICE")` (`d_a_obj_iceisland.cpp:78`) into the
/// `s16 mMeltIceEventIdx` field (`d_a_obj_iceisland.h:34`). The one-argument call uses
/// `evNo = -1` (`d_com_inf_game.h:3611`), i.e. `0xFF`, which skips the stage event-info branch of
/// `getEventIdx` (`d_event_manager.cpp:347-348`) and is a pure name lookup.
const MELT_ICE_EVENT: &str = "MELT_ICE"; // <- d_a_obj_iceisland.cpp:78

/// `dComIfGp_evmng_getEventIdx("FREEZE_ICE")` (`d_a_obj_iceisland.cpp:79`) into the
/// `s16 mFreezeIceEventIdx` field (`d_a_obj_iceisland.h:35`).
const FREEZE_ICE_EVENT: &str = "FREEZE_ICE"; // <- d_a_obj_iceisland.cpp:79

/// `mDoAud_seStart(JA_SE_READ_RIDDLE_1)` (`d_a_obj_iceisland.cpp:105`).
const SE_READ_RIDDLE_1: u32 = 0x0806; // <- JAZelAudio_SE.h:18

/// `fopAcM_seStartCurrent(this, JA_SE_ATM_ICEBERG_WIND, field_0x396)`
/// (`d_a_obj_iceisland.cpp:244`). Its third argument is `mDoAud_seStart`'s `param`, not a volume:
/// `fopAcM_seStartCurrent` is `mDoAud_seStart(id, &pos, param, reverb)`
/// (`f_op_actor_mng.h:691-693`), which is `seStart(id, pos, param, reverb, 1.0f, 1.0f, -1.0f,
/// -1.0f, 0)` with the volume pair separate (`m_Do_audio.h:149-151`). `field_0x396` (the `s16` of
/// `d_a_obj_iceisland.h:33`, seeded 0 or 100 at `:72` / `:76` and ramped at `:236-243`) is written
/// only to feed that argument, and `super::events::SimEvent::Se(u32)` carries the id alone - the
/// shell's cue bank keeps fixed gains and has no per-sound param either - so the ramp is not
/// modelled; the sound itself is pushed once per tick, exactly as retail calls it.
const SE_ATM_ICEBERG_WIND: u32 = 0x1072; // <- JAZelAudio_SE.h:306

/// The profile's `fopAc_CULLBOX_CUSTOM_e` (`d_a_obj_iceisland.cpp:303`) has NO
/// `fopAcM_setCullSizeBox` and no `fopAcM_SetMin` / `fopAcM_SetMax` anywhere in
/// `daObjIceisland_c`, so `cull.box` stays at the `sBs_ClearArea` zero-fill (`f_pc_base.cpp:98`)
/// and there is NO box to expose - `cull()` is deliberately not written. The one cull datum the
/// class does set is `fopAcM_setCullSizeFar(this, 1.0f)` (`:53`), the far clip, which
/// `fopAcM_cullingCheck` (`f_op_actor_mng.cpp:589+`) uses only to pick the clipper and which
/// `view::ActorView` cannot carry.

/// The icy clouds surrounding Ice Ring Isle - `daObjIceisland_c` (`d_a_obj_iceisland.h:8-39`),
/// the actor behind the stage string `GiceL`.
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. The class holds two BTK animators, one BRK
/// animator, two emitters, a tev string, three counters and a mode int
/// (`d_a_obj_iceisland.h:25-35`) and registers no `dCcD` collider of any kind. The profile
/// `Group` is `fopAc_ACTOR_e` (`:302`). Neither `combat::HitPoints` nor `enemy::Hurt` is attached.
///
/// `daObjIceisland_IsDelete` (`:277-279`) is the bare "accepts a delete request" form of RULE 6b,
/// and `_create` returns `cPhs_ERROR_e` only from `fopAcM_entrySolidHeap` (`:181-182`), which is
/// heap work.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Iceisland {
    /// `field_0x39C` (`d_a_obj_iceisland.h:35`), the mode `_execute`'s switch dispatches on
    /// (`:207-232`).
    pub mode: i32,
    /// `daObjIceisland_prm::getSwitchNo(this)` (`d_a_obj_iceisland.h:41`), read through
    /// `fopAcM_isSwitch` (`:60`, `:96`, `:135`, `:165`, `:237`). Zero at spawn because
    /// `CreateInit` has not run yet - `create_init` fills it on the first tick.
    pub switch_no: u8,
    /// `mMeltIceEventIdx` (`d_a_obj_iceisland.h:34`); `None` is `getEventIdx`'s `-1`
    /// (`d_event_manager.cpp:355`).
    pub melt_slot: Option<usize>,
    /// `mFreezeIceEventIdx` (`d_a_obj_iceisland.h:35`); `None` is the same `-1`.
    pub freeze_slot: Option<usize>,
    /// One-shot guard so `CreateInit`'s two `getEventIdx` lookups (`:78-79`) and its switch test
    /// (`:60`) happen once.
    pub entered: bool,
}

impl Iceisland {
    /// `daObjIceisland_c::_create` (`:177-188`) minus `fopAcM_ct` (`:178`), `dComIfG_resLoad`
    /// (`:179`), `fopAcM_entrySolidHeap` / `CheckCreateHeap` / `CreateHeap` (`:181-182`,
    /// `:14-16`, `:19-38`) and `CreateInit` (`:184`), which is deferred to the first tick because
    /// it needs `Switches` and `EventList`. What survives is the `getSwitchNo` decode, which
    /// `CreateInit` (`:59`) would have applied to the `u8` local of the same name.
    pub fn spawn(parameters: u32) -> Self {
        Self {
            // `field_0x39C` has no initialiser in `CreateInit`; before it runs the value is the
            // `sBs_ClearArea` zero-fill (`f_pc_base.cpp:98`), which is `MODE_FREEZE_MAIN`.
            mode: MODE_FREEZE_MAIN,
            switch_no: ((parameters >> SWITCH_SHIFT) & SWITCH_MASK) as u8,
            melt_slot: None,
            freeze_slot: None,
            entered: false,
        }
    }

    /// `daObjIceisland_c::CreateInit` (`:51-83`) minus `fopAcM_SetMtx` (`:52`),
    /// `fopAcM_setCullSizeFar` (`:53`, a constant with nowhere to be read), `set_mtx` (`:54`),
    /// `dKy_tevstr_init` (`:55`), `g_env_light.settingTevStruct` (`:56`),
    /// `daObjIceisland_particle_set` (`:74`), `mBrkAnm.setFrame(mBrkAnm.getEndFrame())` (`:61`) and
    /// `mEmitter1 = mEmitter2 = NULL` (`:57-58`) - renderer, particles and BRK archive work whose
    /// frame count is not in this checkout (RULE 3b).
    ///
    /// The `dComIfGs_getStartPoint() == 2 && current.roomNo == dComIfGs_getRestartRoomNo()`
    /// sub-branch (`:62-68`) that would pick `MODE_FAIL_DEMO_WAIT` over `MODE_MELT_MAIN` is
    /// omitted: the save block's start code and restart room have no resource in this sim, so
    /// `MODE_FAIL_DEMO_WAIT` is never entered from `CreateInit`. It is still reachable from
    /// `_execute` if `field_0x39C` ever becomes 6 by another route.
    fn create_init(&mut self, list: &EventList, switches: &Switches) {
        if switches.test(self.switch_no) {
            self.mode = MODE_MELT_MAIN;
        } else {
            self.mode = MODE_FREEZE_MAIN;
        }
        self.melt_slot = list.event_index(MELT_ICE_EVENT);
        self.freeze_slot = list.event_index(FREEZE_ICE_EVENT);
        self.entered = true;
    }

    /// `daObjIceisland_c::daObjIceisland_freeze_main` (`:94-100`): `if (fopAcM_isSwitch(this,
    /// switchNo)) { fopAcM_orderOtherEventId(this, mMeltIceEventIdx); field_0x39C = 1; }`.
    /// Guarded exactly as `vfan::Vfan::order` / `tagetc::TagEtc::order` do - re-ordering an event
    /// that is already ORDER / PLAY / CLOSE rewrites its state (`d_event_manager.cpp:367`).
    fn freeze_main(&mut self, ctx: &mut Context<'_>) {
        if !ctx.switches.test(self.switch_no) {
            return;
        }
        self.order(ctx, self.melt_slot);
        self.mode = MODE_MELT_DEMO_WAIT;
    }

    /// `daObjIceisland_c::daObjIceisland_melt_demo_wait` (`:103-121`):
    /// `if (eventInfo.checkCommandDemoAccrpt()) { ... mMode = 2; } else { fopAcM_orderOtherEventId(
    /// this, mMeltIceEventIdx); }`.
    ///
    /// `eventInfo.checkCommandDemoAccrpt()` (`:104`) is `mCommand == dEvtCmd_INDEMO_e`
    /// (`f_op_actor.h:177`), a latch `dEvt_control_c::demoCheck` sets on both ordered actors when
    /// the event starts (`d_event.cpp:314-316`) and `demoEnd` clears (`:331-337`) - so, exactly as
    /// `htobi3.rs` and `tagetc.rs` argue, "the event this actor ordered is in `Stage::Play` or
    /// `Stage::Close`".
    ///
    /// `mDoAud_seStart(JA_SE_READ_RIDDLE_1)` (`:105`) survives as a `SimEvent::Se`. The two
    /// `emitter->becomeInvalidEmitter(); mEmitterN = NULL` pairs (`:107-115`) and
    /// `mBrkAnm.setPlaySpeed(1.0f)` (`:116`) are particles and BRK archive work.
    fn melt_demo_wait(&mut self, ctx: &mut Context<'_>) {
        if ctx.demo_running(self.melt_slot) {
            ctx.events.list.push(SimEvent::Se(SE_READ_RIDDLE_1));
            self.mode = MODE_MELT_DEMO;
        } else {
            self.order(ctx, self.melt_slot);
        }
    }

    /// `daObjIceisland_c::daObjIceisland_melt_demo` (`:124-130`):
    /// `mBrkAnm.play(); if (dComIfGp_evmng_endCheck(mMeltIceEventIdx)) { dComIfGp_event_reset();
    /// field_0x39C = 3; }`. `dComIfGp_event_reset()` is `dEvDtBase_c::reset()` (`d_event.h:131`)
    /// = `mEventFlag |= 8` on the running event, which `dEvt_control_c::check` (`d_event.cpp:546-554`)
    /// turns into `endProc`; the sim's `ActiveEvent::end` is that teardown.
    fn melt_demo(&mut self, ctx: &mut Context<'_>) {
        let Some(slot) = self.melt_slot else {
            return;
        };
        if ctx.active.end_check(slot) {
            let _ = ctx.active.end(slot);
            self.mode = MODE_MELT_MAIN;
        }
    }

    /// `daObjIceisland_c::daObjIceisland_melt_main` (`:133-139`):
    /// `if (!fopAcM_isSwitch(this, switchNo)) { fopAcM_orderOtherEventId(this, mFreezeIceEventIdx);
    /// field_0x39C = 4; }`.
    fn melt_main(&mut self, ctx: &mut Context<'_>) {
        if ctx.switches.test(self.switch_no) {
            return;
        }
        self.order(ctx, self.freeze_slot);
        self.mode = MODE_FREEZE_DEMO_WAIT;
    }

    /// `daObjIceisland_c::daObjIceisland_freeze_demo_wait` (`:142-151`):
    /// `if (eventInfo.checkCommandDemoAccrpt()) { daObjIceisland_particle_set();
    /// mBrkAnm.setPlaySpeed(-1.0f); field_0x39C = 5; } else { fopAcM_orderOtherEventId(this,
    /// mFreezeIceEventIdx); }`. The particle spawn and the BRK play speed are particles and
    /// archive work.
    fn freeze_demo_wait(&mut self, ctx: &mut Context<'_>) {
        if ctx.demo_running(self.freeze_slot) {
            self.mode = MODE_FREEZE_DEMO;
        } else {
            self.order(ctx, self.freeze_slot);
        }
    }

    /// `daObjIceisland_c::daObjIceisland_freeze_demo` (`:154-160`): the mirror of `melt_demo`,
    /// ending `mFreezeIceEventIdx` and landing on `field_0x39C = 0`.
    fn freeze_demo(&mut self, ctx: &mut Context<'_>) {
        let Some(slot) = self.freeze_slot else {
            return;
        };
        if ctx.active.end_check(slot) {
            let _ = ctx.active.end(slot);
            self.mode = MODE_FREEZE_MAIN;
        }
    }

    /// `daObjIceisland_c::daObjIceisland_fail_demo_wait` (`:163-170`):
    /// `if (!fopAcM_isSwitch(this, switchNo)) { daObjIceisland_particle_set();
    /// mBrkAnm.setPlaySpeed(1.0f); field_0x39C = 7; }`. Note there is NO else branch - a set
    /// switch leaves the island parked in `MODE_FAIL_DEMO_WAIT`.
    fn fail_demo_wait(&mut self, ctx: &mut Context<'_>) {
        if !ctx.switches.test(self.switch_no) {
            self.mode = MODE_FAIL_DEMO_MAIN;
        }
    }

    /// `daObjIceisland_c::daObjIceisland_fail_demo_main` (`:173-175`) is `mBrkAnm.play();` alone -
    /// pure BRK archive work, so the mode is a stable sink with no body.
    fn fail_demo_main(&mut self) {}

    /// `fopAcM_orderOtherEventId(this, i_eventIdx)` (`:97`, `:119`, `:136`, `:149`).
    fn order(&self, ctx: &mut Context<'_>, slot: Option<usize>) {
        let Some(slot) = slot else {
            return;
        };
        if ctx.active.stage_of(slot).is_none() {
            let _ = ctx.active.order(ctx.list, slot, ctx.entity);
        }
    }

    /// `daObjIceisland_c::_execute` (`:206-254`). The switch (`:207-232`) has exactly these eight
    /// labels and `field_0x39C` is an `int`, so every value in range lands on a real arm and
    /// anything else falls through - which is what `fail_demo_main` is for.
    ///
    /// `mBtkAnm1.play()` / `mBtkAnm2.play()` (`:233-234`) and `set_mtx()` (`:235`) are archive and
    /// renderer work. The `field_0x396` ramp (`:236-243`) exists only to scale
    /// `fopAcM_seStartCurrent(this, JA_SE_ATM_ICEBERG_WIND, field_0x396)` (`:244`); `SimEvent::Se`
    /// carries the id alone, so the field is dropped and the sound is pushed once per tick, exactly
    /// as retail does. The `mEmitter1->setGlobalPrmColor` / `setGlobalEnvColor` pair (`:245-252`)
    /// reads `mTevStr.mColorK0` (renderer) and is dropped.
    fn execute(&mut self, ctx: &mut Context<'_>) {
        match self.mode {
            MODE_FREEZE_MAIN => self.freeze_main(ctx),
            MODE_MELT_DEMO_WAIT => self.melt_demo_wait(ctx),
            MODE_MELT_DEMO => self.melt_demo(ctx),
            MODE_MELT_MAIN => self.melt_main(ctx),
            MODE_FREEZE_DEMO_WAIT => self.freeze_demo_wait(ctx),
            MODE_FREEZE_DEMO => self.freeze_demo(ctx),
            MODE_FAIL_DEMO_WAIT => self.fail_demo_wait(ctx),
            MODE_FAIL_DEMO_MAIN => self.fail_demo_main(),
            _ => {}
        }
        ctx.events.list.push(SimEvent::Se(SE_ATM_ICEBERG_WIND));
    }
}

/// Everything `_execute` needs beyond the component, in one bundle so the state machine has a
/// single reader - the `tagetc` / `dekuitem` shape.
struct Context<'a> {
    switches: &'a Switches,
    events: &'a mut SimEvents,
    list: &'a EventList,
    active: &'a mut ActiveEvent,
    entity: Entity,
}

impl Context<'_> {
    fn demo_running(&self, slot: Option<usize>) -> bool {
        slot.is_some_and(|slot| {
            matches!(
                self.active.stage_of(slot),
                Some(Stage::Play) | Some(Stage::Close)
            )
        })
    }
}

pub fn is_iceisland(name: &str) -> bool {
    ICEISLAND_ACTORS.contains(&name)
}

pub fn control(
    list: Res<EventList>,
    mut active: ResMut<ActiveEvent>,
    mut events: ResMut<SimEvents>,
    switches: Res<Switches>,
    mut islands: Query<(Entity, &mut Iceisland)>,
) {
    for (entity, mut island) in &mut islands {
        let island = &mut *island;
        if !island.entered {
            island.create_init(&list, &switches);
        }
        let mut ctx = Context {
            switches: &switches,
            events: &mut events,
            list: &list,
            active: &mut active,
            entity,
        };
        island.execute(&mut ctx);
    }
}
