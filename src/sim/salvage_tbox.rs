use bevy_ecs::prelude::*;

use super::actor::Actor;
use super::event_manager::{EventManager, StaffActor};
use super::events::{SimEvent, SimEvents};
use super::sea::Sea;
use super::ship::Crane;
use super::water::probe;
use crate::text::msgproc::MsgGameInfo;

/// `OBJNAME("StBox", fpcNm_SALVAGE_TBOX_e, 255, 0)` <- tww/src/d/d_stage.cpp:1234.
/// The staff cut keys off the differently-cased `"STBox"` <- d_a_salvage_tbox.cpp:317.
pub const SALVAGE_TBOX_ACTORS: [&str; 1] = ["STBox"];

pub fn is_salvage_tbox(name: &str) -> bool {
    SALVAGE_TBOX_ACTORS.contains(&name)
}

/// `eventInfo.checkCommandTalk()` <- d_a_salvage_tbox.cpp:328, which is
/// `mCommand == dEvtCmd_INTALK_e` <- f_op_actor.h:174. `mCommand` is reachable at runtime
/// (`d_event.cpp:145,191,233`) but `EventManager` models no per-actor command, so the gate
/// is folded to its "no talk" side. See OMITTED.
pub const TALK_COMMAND: bool = false;

const BOX_TYPE_SHIFT: u32 = 8; // <- d_a_salvage_tbox.h:103 (getBoxType)
const BOX_TYPE_MASK: u32 = 0xF; // <- d_a_salvage_tbox.h:103 (getBoxType)
const ITEM_NO_MASK: u32 = 0xFF; // <- d_a_salvage_tbox.h:104 (getItemNo)

const CRANE_OFFSET: [f32; 3] = [80.0, 125.0, 125.0]; // <- crane_offset[] :400
const DROP_GRAVITY: f32 = -4.0; // <- fopAcM_SetGravity(this, -4.0f) :397
const RIPPLE_OFFSET_Y: f32 = 2500.0; // <- mParticlePos.y += 2500.0f :430
const RIPPLE_MARGIN: f32 = 10.0; // <- current.pos.y > mParticlePos.y - 10.0f :432
const RIPPLE_RATE: f32 = 12.0; // <- mRippleCallBack.setRate(12.0f) :434
const SINK_MARGIN: f32 = 50.0; // <- getWaterY(current.pos) - 50.0f :452
const SHADOW_ALPHA: u8 = 0x78; // <- m_shadow_alpha :18
const SHADOW_DEPTH: f32 = 2000.0; // <- m_shadow_depth :19
const SHADOW_SCROLL: f32 = -0.1; // <- m_shadow_scroll :20
const SHADOW_SCALE: f32 = 4.0; // <- m_shadow_scale :21
const ROPE_MAX_LENGTH: f32 = 2500.0; // <- m_rope_max_length :17
const BGM_BGN_GET_BOX: u32 = 0x8000005D; // <- JA_BGM_BGN_GET_BOX :424
const DROP_WAIT: i32 = 20; // <- field_0x332 = 20 :368

// Emitter XZ scale per box type: box type 0 -> f1 = 3.5f, types 1 and 2 -> f1 = 3.0f
// <- :229-231 and :251-253, fed to setEmitterScale(f1, f2, f1) :239/:262. The Y component
// is f2 = 1.0f in both branches and is not representable in the scalar helper below.
const HOP: [f32; 3] = [3.5, 3.0, 3.0];
// Emitter Y translation per box type: box type 0 -> f4 = -20.0f, types 1 and 2 -> f4 = 20.0f
// <- :232 and :254, fed to setEmitterTranslation(f3, f4, f3) :241/:264.
const RISE: [f32; 3] = [-20.0, 20.0, 20.0];
const EMITTER_GLOBAL: f32 = 1.5; // <- setGlobalParticleScale(1.5f, 1.5f) :237/:260
const EMITTER_COUNT: usize = 3; // <- ARRAY_SIZE(field_0x29C) :233

const ACTIONS: [&str; 5] = [
    // <- action_table[] :316
    "WAIT",         // :316
    "WAIT02",       // :316
    "WAIT_GETITEM", // :316
    "WAIT_DUMMY",   // :316
    "DROP",         // :316
];

/// `event_init_tbl` / `event_action_tbl` order <- :297-312.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoxAct {
    Wait,
    Wait02,
    WaitGetItem,
    WaitDummy,
    Drop,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct SalvageTbox {
    pub act: BoxAct,
    /// `field_0x331` <- :276. Retail indexes the 3-entry `crane_offset` (:419), `m_bdlidx`
    /// (:191) and `m_heapsize` (:279) with this 0..15 value and reads out of bounds for
    /// box types 3..15; the lookups below clamp to 2 rather than reproduce that read.
    pub box_type: u32,
    /// `field_0x330` <- daSTBox_prm::getItemNo :217.
    pub item_no: u32,
    /// `field_0x332` <- initWait02 :368. Only written; never read in retail.
    pub wait: i32,
    /// `fopAcM_offDraw(this)` <- initWaitGetItem :379.
    pub drawn: bool,
    /// `field_0x334` <- actDrop :456, the `JA_SE_OBJ_FALL_WATER_M` one-shot.
    pub splashed: bool,
    /// `field_0x336` <- CreateInit :220 / actWait :425.
    pub bgm_played: bool,
    /// `field_0x335` <- actWait :428, the ripple emitter latch.
    pub ripple: bool,
    /// `attention_info.position = current.pos` <- actWait :421.
    pub attention: glam::Vec3,
    pub shadow_alpha: u8,
    pub shadow_water_y: f32,
    pub shadow_flat_y: f32,
    pub shadow_depth: f32,
    pub shadow_pos: glam::Vec3,
    pub shadow_scroll: f32,
    /// `fopAcM_SetGravity` <- initDrop :397.
    pub gravity: f32,
}

impl SalvageTbox {
    pub fn spawn(parameters: u32) -> Self {
        Self {
            act: BoxAct::Wait,
            box_type: (parameters >> BOX_TYPE_SHIFT) & BOX_TYPE_MASK,
            item_no: parameters & ITEM_NO_MASK,
            wait: 0,
            drawn: true,
            splashed: false,
            bgm_played: false,
            ripple: false,
            attention: glam::Vec3::ZERO,
            shadow_alpha: SHADOW_ALPHA,
            shadow_water_y: 0.0,
            shadow_flat_y: 0.0,
            shadow_depth: SHADOW_DEPTH,
            shadow_pos: glam::Vec3::ZERO,
            shadow_scroll: SHADOW_SCROLL,
            gravity: 0.0,
        }
    }

    /// `event_init_tbl` <- :297-303.
    fn init(&mut self, act: BoxAct) {
        self.act = act;
        match act {
            // initWait :362
            BoxAct::Wait | BoxAct::WaitDummy => {}
            // initWait02 :367
            BoxAct::Wait02 => self.wait = DROP_WAIT,
            // initWaitGetItem :378
            BoxAct::WaitGetItem => {
                self.drawn = false;
                self.ripple = false;
            }
            // initDrop :396
            BoxAct::Drop => self.gravity = DROP_GRAVITY,
        }
    }

    /// `event_action_tbl` <- :306-312.
    fn step(
        &mut self,
        actor: &mut Actor,
        crane: Option<&Crane>,
        sea: Option<&Sea>,
        events: &mut SimEvents,
    ) -> bool {
        match self.act {
            // actWait :403
            BoxAct::Wait => {
                let Some(crane) = crane else {
                    return false;
                };
                let Some(top) = crane.top else {
                    return false;
                };
                // actWait samples the water at the crane top (:413-415), not at the rope
                // segment `_execute` sampled for the shadow callback.
                let top_water_y = probe(sea, top.x, top.z).1;
                // current.angle.y = ship->getCraneHookAngleY() :417
                actor.angle[1] = crane.angle;
                // craneTopPos2.y -= crane_offset[field_0x331] :418-420
                let mut pos = top;
                pos.y -= CRANE_OFFSET[self.box_type.min(2) as usize];
                actor.pos = pos;
                // attention_info.position = current.pos :421
                self.attention = actor.pos;
                // mDoAud_subBgmStart(JA_BGM_BGN_GET_BOX) :423-426 -- gated on the crate
                // being clear of the water, not merely on the box type.
                if (self.box_type == 1 || self.box_type == 2)
                    && actor.pos.y > top_water_y
                    && !self.bgm_played
                {
                    events.list.push(SimEvent::Bgm(BGM_BGN_GET_BOX));
                    self.bgm_played = true;
                }
                // :428-437 -- the latch only latches once the crate is near the surface.
                if !self.ripple {
                    let mut ripple = actor.pos;
                    ripple.y += RIPPLE_OFFSET_Y;
                    // mParticlePos.y = dBgS_GetWaterHeight(mParticlePos) :431 is a
                    // per-poly-group query the merged-DZB `Collision` cannot answer; the
                    // sea surface probe is a partial stand-in. See OMITTED.
                    ripple.y = probe(sea, ripple.x, ripple.z).1;
                    if actor.pos.y > ripple.y - RIPPLE_MARGIN {
                        self.ripple = true;
                    }
                }
                false
            }
            // actWait02 :467
            BoxAct::Wait02 => {
                if let Some(crane) = crane {
                    if let Some(top) = crane.top {
                        let mut pos = top;
                        pos.y -= CRANE_OFFSET[self.box_type.min(2) as usize];
                        actor.pos = pos;
                    }
                }
                false
            }
            // actWaitGetItem :478 / actWaitDummy :483 -- both return TRUE, ending the cut.
            BoxAct::WaitGetItem | BoxAct::WaitDummy => true,
            // actDrop :450
            BoxAct::Drop => {
                // fopAcM_posMoveF(this, NULL) :451
                actor.pos.y += self.gravity;
                let surface = probe(sea, actor.pos.x, actor.pos.z).1;
                if actor.pos.y < surface - SINK_MARGIN {
                    return true;
                }
                if actor.pos.y < surface {
                    if !self.splashed {
                        // fopAcM_seStart(JA_SE_OBJ_FALL_WATER_M, 0) :457
                        self.splashed = true;
                    }
                    // mRippleCallBack.end() :461
                    self.ripple = false;
                }
                false
            }
        }
    }
}

pub fn control(
    mut boxes: Query<(&mut Actor, &mut SalvageTbox)>,
    cranes: Query<&Crane>,
    sea: Option<Res<Sea>>,
    info: Res<MsgGameInfo>,
    mut manager: ResMut<EventManager>,
    mut events: ResMut<SimEvents>,
) {
    let crane = cranes.iter().next();
    let staff = manager.get_my_staff_id("STBox", None::<StaffActor>, 0);
    // dComIfGp_event_runCheck() && !checkCommandTalk() && staffIdx != -1 <- :328
    let running = info.event_run_check() && !TALK_COMMAND && staff != -1;
    let ripple_pos = crane.map(|crane| crane.segment);
    let water_y = match ripple_pos {
        Some(pos) => probe(sea.as_deref(), pos.x, pos.z).1,
        None => 0.0,
    };
    for (mut actor, mut box_state) in boxes.iter_mut() {
        // The action table runs ONLY while the cut is live <- :328-340. Outside it the
        // retail `_execute` touches nothing but the emitters (:342-347), the shadow
        // callback (:349-356) and set_mtx (:357), so the crate keeps whatever position it
        // was spawned or last left at.
        if running {
            let idx = manager.get_my_act_idx(staff, &ACTIONS, false, 0);
            if idx == -1 {
                // dComIfGp_evmng_cutEnd(staffIdx) :331
                manager.cut_end(staff);
            } else {
                // dComIfGp_evmng_getIsAddvance(staffIdx) :333
                if manager.get_is_advance(staff) {
                    box_state.init(ACTIONS[idx as usize].into());
                }
                let done = box_state.step(&mut actor, crane, sea.as_deref(), &mut events);
                if done {
                    // dComIfGp_evmng_cutEnd(staffIdx) :337
                    manager.cut_end(staff);
                }
            }
        }
        // mShadowCallback.setWaterY/setWaterFlatY/setDepth/setPos :350-353
        if actor.pos.y < water_y {
            box_state.shadow_water_y = water_y + 2.0;
            box_state.shadow_flat_y = water_y + 2.0;
            box_state.shadow_depth = water_y - actor.pos.y;
            if let Some(pos) = ripple_pos {
                box_state.shadow_pos = pos;
            }
        }
        if box_state.ripple {
            box_state.shadow_scroll = SHADOW_SCROLL;
        }
    }
}

impl From<&str> for BoxAct {
    /// `event_init_tbl` / `event_action_tbl` are parallel to `action_table` <- :297-312.
    fn from(name: &str) -> Self {
        match name {
            "WAIT02" => BoxAct::Wait02,
            "WAIT_GETITEM" => BoxAct::WaitGetItem,
            "WAIT_DUMMY" => BoxAct::WaitDummy,
            "DROP" => BoxAct::Drop,
            _ => BoxAct::Wait,
        }
    }
}

pub fn spawn(world: &mut World, source: &crate::dzs::StageActor) -> Entity {
    let actor = Actor {
        name: source.name.clone(),
        parameters: source.parameters,
        pos: glam::Vec3::from_array(source.pos),
        angle: source.angle,
        set_id: source.set_id,
        room: source.room,
    };
    let state = SalvageTbox::spawn(source.parameters);
    world.spawn((actor, state)).id()
}

pub fn hook_length() -> f32 {
    ROPE_MAX_LENGTH
}

/// Emitter XZ scale; see `HOP`.
pub fn emitter_hop(box_type: u32) -> f32 {
    HOP[box_type.min(2) as usize]
}

/// Emitter Y translation; see `RISE`.
pub fn emitter_rise(box_type: u32) -> f32 {
    RISE[box_type.min(2) as usize]
}

pub fn emitter_global() -> f32 {
    EMITTER_GLOBAL
}
