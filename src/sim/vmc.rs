use bevy_ecs::prelude::*;

use super::actor::Actor;
use super::event::{ActiveEvent, EventList};
use super::events::{SimEvent, SimEvents};
use super::flags::{EventFlags, Switches};

pub const VMC_ACTORS: [&str; 1] = ["VmcBS"];

/// `enum { STATE_BASE_MAIN, ... }` (`d_a_obj_vmc.cpp:11-16`). The demo entries are
/// `VERSION > VERSION_DEMO`; the DEMO build swaps the `cyl_src_tree` collider in
/// (`:151`, `:230`) instead of transitioning.
const STATE_BASE_MAIN: u8 = 0; // <- d_a_obj_vmc.cpp:12
const STATE_TREE_DEMO_WAIT: u8 = 1; // <- d_a_obj_vmc.cpp:13
const STATE_TREE_DEMO_MAIN: u8 = 2; // <- d_a_obj_vmc.cpp:14
const STATE_TREE_MAIN: u8 = 3; // <- d_a_obj_vmc.cpp:15

const PRM_SWSAVE_WIDTH: u32 = 8; // <- d_a_obj_vmc.h:17
const PRM_SWSAVE_SHIFT: u32 = 8; // <- d_a_obj_vmc.h:18

/// `fopAcM_setCullSizeBox(this, -300.0f, -0.0f, -300.0f, 300.0f, 500.0f, 300.0f)`
/// (`d_a_obj_vmc.cpp:140`) — the class calls `fopAcM_setCullSizeBox`, so this is a real box
/// and not a `fopAc_CULLBOX_*_e` table entry.
const CULL_MIN: [f32; 3] = [-300.0, 0.0, -300.0]; // <- d_a_obj_vmc.cpp:140
const CULL_MAX: [f32; 3] = [300.0, 500.0, 300.0]; // <- d_a_obj_vmc.cpp:140

/// `cyl_src_base` (`d_a_obj_vmc.cpp:18-46`). `SrcObjAt Type/Atp/SPrm` and `SrcObjTg
/// Type/SPrm` are all 0 and the only Co term is `cCcD_CoSPrm_Set_e | VsEnemy_e |
/// IsOther_e | NoCrr_e`, so the trunk deals no damage and detects no AT hit. The
/// `ChkTgHit` arm at `:275` therefore always reads false: its `SrcObjTg Type` is 0, so
/// retail's own collider cannot be the hookshot source. That arm is transcribed as an
/// unreachable branch rather than given a substitute AT channel.
const CYL_RADIUS: f32 = 100.0; // <- d_a_obj_vmc.cpp:41
const CYL_HEIGHT: f32 = 10.0; // <- d_a_obj_vmc.cpp:42

const ATTENTION_DISTANCE: f32 = 0xA7 as f32; // <- d_a_obj_vmc.cpp:170 (TALK and SPEAK)
const ATTENTION_LIFT: f32 = 40.0; // <- d_a_obj_vmc.cpp:172
const SHAPE_ANGLE_OFFSET: i16 = 0x1800; // <- d_a_obj_vmc.cpp:234
const SEED_GERMIN_TIMER: u32 = 10; // <- d_a_obj_vmc.cpp:248
const SEED_GROW_TIMER: u32 = 40; // <- d_a_obj_vmc.cpp:250
const SE_OBJ_CB_SEED_GERMIN: u32 = 0x6A13; // <- JAZelAudio_SE.h:2042
const SE_OBJ_CB_SEED_GROW: u32 = 0x6A14; // <- JAZelAudio_SE.h:2043
const EVT_3420: u16 = 0x3420; // <- d_save_event_flag.inc:395
const EVENT_NAME: &str = "cb_sow"; // <- d_a_obj_vmc.cpp:176

/// `daObj::PrmAbstract(this, PRM_SWSAVE_W, PRM_SWSAVE_S)` (`d_a_obj_vmc.h:22`) is
/// `((1 << W) - 1) & (param >> S)` (`d_a_obj.h:12`).
fn prm_get_sw_save(parameters: u32) -> u32 {
    ((1u32 << PRM_SWSAVE_WIDTH) - 1) & (parameters >> PRM_SWSAVE_SHIFT)
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vmc {
    pub state: u8,
    pub sw_save: u32,
    pub has_tree: bool,
    pub has_tree_bg: bool,
    pub hookshot_anim: bool,
    pub timer: u32,
    pub link_range_check: bool,
}

impl Vmc {
    /// `CreateInit` (`d_a_obj_vmc.cpp:138-177`): the switch decides whether the mound
    /// starts bare or already grown.
    pub fn spawn(parameters: u32, switches: &Switches) -> Self {
        let sw_save = prm_get_sw_save(parameters);
        let has_tree = switches.test(sw_save as u8);
        Vmc {
            state: if has_tree {
                STATE_TREE_MAIN
            } else {
                STATE_BASE_MAIN
            },
            sw_save,
            has_tree,
            // `dComIfG_Bgsp()->Regist(mpBgTree, ...)` returns non-zero on failure, which
            // sets `mHasTreeBg` (`:146-147`).
            has_tree_bg: !has_tree,
            hookshot_anim: false,
            timer: 0,
            link_range_check: true,
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

pub fn is_vmc(name: &str) -> bool {
    VMC_ACTORS.contains(&name)
}

pub fn control(
    list: Res<EventList>,
    mut active: ResMut<ActiveEvent>,
    mut flags: ResMut<EventFlags>,
    mut switches: ResMut<Switches>,
    mut events: ResMut<SimEvents>,
    mut mounds: Query<(&mut Actor, &mut Vmc)>,
) {
    for (actor, mut mound) in &mut mounds {
        match mound.state {
            STATE_BASE_MAIN => {}
            STATE_TREE_DEMO_WAIT => mound.state = STATE_TREE_DEMO_MAIN,
            STATE_TREE_DEMO_MAIN => {
                mound.timer += 1;
                if mound.timer == SEED_GERMIN_TIMER {
                    events.list.push(SimEvent::Se(SE_OBJ_CB_SEED_GERMIN));
                } else if mound.timer == SEED_GROW_TIMER {
                    events.list.push(SimEvent::Se(SE_OBJ_CB_SEED_GROW));
                }
                // `mBckAnmGrow.play(); if (mBckAnmGrow.isStop())` (`:253-255`). The BCK
                // end frame is archive data and is not in this checkout, so no stand-in
                // count is invented; the arm stays unreachable here and `fopAcM_onSwitch`
                // with it.
            }
            _ => {}
        }
        let _ = (&list, &mut active, &mut flags, &mut switches, actor);
    }
}
