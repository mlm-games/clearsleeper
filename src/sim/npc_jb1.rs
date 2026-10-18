use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::Vec3;

use super::actor::Actor;
use super::event::{ActiveEvent, EventList};
use super::intent::DoIntent;
use super::walk::from_s16;
use crate::text::msgproc::MsgGameInfo;

pub const NPC_JB1_ACTORS: [&str; 1] = ["Jb1"];

pub fn is_npc_jb1(name: &str) -> bool {
    NPC_JB1_ACTORS.contains(&name)
}

const STAFF_JB1: &str = "Jb1"; // <- mEventCut.setActorInfo2("Jb1", this), :64

// daNpc_Jb1_HIO_c::a_prm_tbl - d_a_npc_jb1.cpp:38-47
const HIO_ATTN_Y_OFFSET: f32 = 100.0; // <- mPrmTbl.field_0x00, :39
const LAMP_OFFSET: Vec3 = Vec3::new(0.0, 1100.0, 700.0); // <- :311

const CULL_MIN: [f32; 3] = [-40.0, 0.0, -40.0]; // <- f_op_actor_mng.cpp:559 (fopAc_CULLBOX_0_e)
const CULL_MAX: [f32; 3] = [40.0, 125.0, 40.0]; // <- f_op_actor_mng.cpp:559

const ACTION_ENDING: i8 = 9; // <- set_action, :395
const ACTION_STARTING: i8 = 0; // <- set_action, :399

const STT_APPEAR: i8 = 0; // <- a_anm_prm_tbl[0].field_0x0 == -1, :176
const STT_WAIT: i8 = 1; // <- wait_action1 case 0 -> setStt(1), :428
const CHAR_JB1_0: i8 = 0; // <- charDecide leaves field_0x921 = 0, :323

const ANM_NONE: i8 = -1; // <- a_anm_prm_tbl[0].field_0x0, :176
const ANM_WAIT01: i8 = 0; // <- a_bck_resID_tbl[0] = dRes_ID_JB_BCK_JB_WAIT01_e, :144
const ANM_PRECREATE: i8 = 2; // <- createInit sets field_0x91C = 2, :65

const ORDER_NONE: i8 = 0; // <- checkOrder, :272
const ORDER_SPEAK: i8 = 1; // <- :251
const ORDER_ATTENTION: i8 = 2; // <- :251

fn rot_y(angle: i16, vec: Vec3) -> Vec3 {
    let rad = from_s16(angle);
    Vec3::new(
        vec.x * rad.cos() + vec.z * rad.sin(),
        vec.y,
        vec.z * rad.cos() - vec.x * rad.sin(),
    )
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct NpcJb1 {
    pub entered: bool,
    pub prev_do: bool,
    pub status: i8,
    pub state: i8,
    pub char_index: i8,
    pub order: i8,
    pub anm: i8,
    pub att_pos: Vec3,
    pub attention: Vec3,
}

impl NpcJb1 {
    pub fn spawn(parameters: u32) -> Self {
        let _ = parameters;
        Self {
            entered: false,
            prev_do: false,
            status: ACTION_STARTING,
            state: STT_APPEAR,
            char_index: 0,
            order: ORDER_NONE,
            anm: ANM_PRECREATE,
            att_pos: Vec3::ZERO,
            attention: Vec3::ZERO,
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        true
    }

    /// `daNpc_Jb1_c::setAnm_anm` (`:151-171`), minus the `dNpc_setAnmIDRes` call and the
    /// `mFrame` / `field_0x909` / `field_0x908` resets (all renderer state).
    fn set_anm_anm(&mut self, idx: i8) {
        if self.anm == idx {
            return;
        }
        self.anm = idx;
    }

    /// `daNpc_Jb1_c::setAnm` (`:174-185`): `a_anm_prm_tbl[field_0x91E].field_0x0`.
    fn anm_prm_anm(&self) -> i8 {
        match self.state {
            STT_WAIT => ANM_WAIT01,
            _ => ANM_NONE,
        }
    }

    fn set_anm(&mut self) {
        let idx = self.anm_prm_anm();
        if idx >= 0 {
            self.set_anm_anm(idx);
        }
    }

    /// `daNpc_Jb1_c::setStt` (`:406-415`).
    fn set_stt(&mut self, stt: i8) {
        self.state = stt;
        if stt == STT_APPEAR || stt == STT_WAIT {
            self.set_anm();
        }
    }

    /// `daNpc_Jb1_c::setAttention` (`:310-317`).
    pub fn set_attention(&mut self, actor: &Actor) {
        self.att_pos = actor.pos + rot_y(actor.angle[1], LAMP_OFFSET);
        self.attention = Vec3::new(
            self.att_pos.x,
            self.att_pos.y + HIO_ATTN_Y_OFFSET,
            self.att_pos.z,
        );
    }

    /// `daNpc_Jb1_c::wait_action1` (`:423-443`); `chkAttention` (`:299-307`) reads the
    /// attention registry and `wait_1` (`:418-420`) only returns TRUE, so neither
    /// `mHasAttention` nor `field_0x910` is observable.
    fn wait_action1(&mut self) {
        if self.status == ACTION_ENDING {
            return;
        }
        if self.status == ACTION_STARTING {
            self.set_stt(STT_WAIT);
            self.status += 1;
        }
    }

    /// `daNpc_Jb1_c::checkOrder` (`:260-274`).
    fn check_order(&mut self, demo_accept: bool, talk_command: bool) {
        if demo_accept {
            return;
        }
        if !talk_command {
            return;
        }
        if self.order != ORDER_SPEAK && self.order != ORDER_ATTENTION {
            return;
        }
        self.order = ORDER_NONE;
    }

    /// `daNpc_Jb1_c::charDecide` (`:320-325`) + `init_JB1_0` (`:54-57`) + `createInit`
    /// (`:60-85`), minus `dKy_plight_set`, `mpMorf->setMorf` and the `setMtx` matrix work.
    /// Returns `false` where `createInit` returns `false`, which cancels creation.
    pub fn create_init(&mut self) -> bool {
        self.anm = ANM_PRECREATE;
        if self.char_index != CHAR_JB1_0 {
            return false;
        }
        self.status = ACTION_STARTING;
        self.wait_action1();
        self.entered = true;
        true
    }

    /// `daNpc_Jb1_c::event_proc` (`:385-389`): `privateCut` needs the stage event-info
    /// table, so only the `mEventCut.cutProc()` gate is left.
    fn event_proc(&mut self, cut_proc: bool) {
        if !cut_proc {
            self.private_cut();
        }
    }

    fn private_cut(&mut self) {}

    /// `daNpc_Jb1_c::_execute` (`:497-529`).
    pub fn execute(&mut self, actor: &Actor, ctx: &Ctx<'_>) {
        let demo_accept = ctx.demo_accept;
        let talk_command = ctx.talk_command;
        self.check_order(demo_accept, talk_command);
        let in_event = ctx.event_run && !talk_command && ctx.is_event_entry();
        if in_event {
            self.event_proc(ctx.cut_proc);
        } else {
            self.wait_action1();
        }
        self.set_attention(actor);
    }
}

pub struct Ctx<'a> {
    pub talk_command: bool,
    pub demo_accept: bool,
    pub event_run: bool,
    pub cut_proc: bool,
    pub list: &'a EventList,
}

impl Ctx<'_> {
    /// `daNpc_Jb1_c::isEventEntry` (`:380-382`).
    fn is_event_entry(&self) -> bool {
        self.list.staff_of(STAFF_JB1).is_some()
    }
}

#[derive(SystemParam)]
pub struct Env<'w> {
    pub info: Res<'w, MsgGameInfo>,
    pub active: Res<'w, ActiveEvent>,
}

pub fn control(
    world: Env,
    list: Res<EventList>,
    do_intent: Res<DoIntent>,
    mut commands: Commands,
    mut actors: Query<(Entity, &Actor, &mut NpcJb1)>,
) {
    let demo_accept = world.info.event_run_check();
    let do_now = do_intent.0;
    for (entity, actor, mut jb1) in &mut actors {
        let tap = do_now && !jb1.prev_do;
        jb1.prev_do = do_now;
        let cut_proc = ActiveEvent::staff_cut(&world.active, &list, STAFF_JB1).is_some();
        let ctx = Ctx {
            talk_command: tap,
            demo_accept,
            event_run: demo_accept,
            cut_proc,
            list: &list,
        };
        if !jb1.entered {
            if !jb1.create_init() {
                let _ = commands.entity(entity).despawn();
                continue;
            }
        }
        jb1.execute(actor, &ctx);
    }
}
