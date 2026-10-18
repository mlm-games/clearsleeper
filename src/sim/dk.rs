//! Helmaroc King - `fpcNm_DK_e`, `d_a_dk.cpp`, stage string `Dk` (`d_stage.cpp:844`). A second
//! OBJNAME row exists at `d_stage.cpp:671`, `OBJNAME("Demo_Dk", fpcNm_DEMO_DK_e, 255, 0)`, but that
//! one belongs to `fpcNm_DEMO_DK_e`, not to `fpcNm_DK_e`, so it is not claimed here.
//!
//! This is NOT `src/sim/dragon.rs`. That module ports `d_a_dr.cpp`, whose profile is named
//! `g_profile_DR` with `fpcNm_DR_e`, and whose stage row is the separate string `dragon`
//! (`dragon.rs:8`). Different translation unit, different proc name, different OBJNAME row; the two
//! are not merged, and grepping `d_stage.cpp` is what separates them.
//!
//! (a) inbound damage: NONE. (b) outbound damage: NONE.
//!
//! `dk_class` (`d_a_dk.h:16-51`) holds NO `dCcD`, `dCcD_Stts`, `dCcD_Cyl` or `dCcD_Sph` member at
//! all. Its members are `mPhs`, `field_0x2B4`, `field_0x2B8`, `field_0x2BC`, `field_0x2C0[5]`,
//! the four `tails`, `mpModelKamen`, a `dBgS_AcchCir`, a `dBgS_ObjAcch`, `field_0xAC5`,
//! `field_0xAC8`, a `dPa_smokeEcallBack` and `field_0xAEE`. `dComIfG_Ccsp()->Set()` appears
//! nowhere in the file, so retail registers no collider for this boss at all: the Helmaroc cannot
//! be damaged by the sword and deals no contact damage. Its entire offensive content is
//! `field_0xAC5`'s scripted state machine, which only releases the boss when the events say so.
//! Neither `combat::HitPoints` nor `enemy::Hurt` is attached, and attaching them would be a
//! correctness bug: `combat::control` would then be able to delete a boss retail cannot hurt.
//!
//! `daDk_IsDelete { return TRUE; }` (`:297-299`) only ACCEPTS a delete request per RULE 6b.
//! `daDk_Create` returns `cPhs_ERROR_e` only when `fopAcM_entrySolidHeap` fails (`:373-375`),
//! which is a resource allocation failure rather than a refusal, so there is no `refused` flag.
//!
//! `move()` (`:205-206`) and `daDk_demoProc()` (`:209-210`) are EMPTY in retail. `daDk_Execute`
//! calls `move(a_this)` at `:231` and it does nothing, so the dragon never translates itself and
//! nothing here invents flight.
//!
//! `REG0_F(0..5)` appear at `:67`, `:78`, `:90` and `:172-174` and fold to zero per RULE 2b
//! (the arrays are zero-initialised at `d_s_play.cpp:81` and only ever written by per-stage HIO
//! data). `l_HIO`'s members DO have non-zero constructor defaults (`:17-24`), so unlike `REG0_F`
//! they are not a proven zero; they are folded to their ctor values, which is the same treatment
//! `helmaroc.rs`, `gohma.rs` and `daiocta.rs` give their own per-actor HIO structs.
//!
//! The profile's `Cull Type` is `fopAc_CULLBOX_CUSTOM_e` (`:422`) but the actor DOES call
//! `fopAcM_SetMin` / `fopAcM_SetMax` (`:378-379`), so this is a custom box WITH a box and the
//! literals are real - not the bug class 3 refusal case.
//!
//! `daDk_delete_Bdk()` (`:213-218`) searches for the placeholder boss by name -
//! `fopAcM_searchFromName("Bdk", 0, 0)` - and deletes it. `Bdk` is already claimed as a stage
//! string by `helmaroc::HELMAROC_ACTOR` (`helmaroc.rs:21`), so re-declaring it here would create a
//! duplicate claim and a dead `spawn_actor` branch, which is why the search is not reproduced.
//! The call is additionally unreachable: its only caller is the `if (demo_set)` arm at `:246`,
//! and `demo_set` requires the absent demo registry.

use bevy_ecs::prelude::*;

use super::actor::Actor;
use super::event::{ActiveEvent, EventList};
use super::flags::EventFlags;

/// `d_stage.cpp:844` binds `Dk` to `fpcNm_DK_e`, the profile of `d_a_dk.cpp:408-423`.
pub const DK_ACTORS: [&str; 1] = ["Dk"];

/// `fopAcM_SetMin(i_this, -1000.0f, -1000.0f, -1000.0f)` (`d_a_dk.cpp:378`) and
/// `fopAcM_SetMax(i_this, 1000.0f, 1000.0f, 1000.0f)` (`:379`).
pub const CULL_MIN: [f32; 3] = [-1000.0, -1000.0, -1000.0];
pub const CULL_MAX: [f32; 3] = [1000.0, 1000.0, 1000.0];

/// `daDk_HIO_c::daDk_HIO_c()` (`d_a_dk.cpp:17-24`): `field_0x08 = 1.0f`.
const HIO_M08: f32 = 1.0;
/// `daDk_HIO_c::daDk_HIO_c()` (`d_a_dk.cpp:19`): `field_0x05 = 0`.
const HIO_M05: u8 = 0;

/// `case 0:` (`d_a_dk.cpp:253`).
const MODE_BOSS: i8 = 0;
/// `a_this->field_0xAC5 = 2;` (`:255`).
const MODE_FLY_START: i8 = 2;
/// `a_this->field_0xAC5 = 3;` (`:260`).
const MODE_FLYING: i8 = 3;
/// `a_this->field_0xAC5 = -1;` (`:265`).
const MODE_RELEASED: i8 = -1;
/// `a_this->field_0xAC5 = 10;` (`:392`), which is also `case 10: default:` (`:268-270`).
const MODE_DORMANT: i8 = 10;

/// `a_this->eyePos = a_this->current.pos; a_this->eyePos.y += 100.0f;` (`d_a_dk.cpp:233-234`).
const EYE_LIFT: f32 = 100.0;

/// `if (a_this->field_0x2B4 == 1)` (`d_a_dk.cpp:389`).
const PARAM_FLYING: u8 = 1;

/// `dComIfGs_isEventBit(dSv_event_flag_c::UNK_0310)` (`d_a_dk.cpp:254`). The id is
/// `d_save_event_flag.inc:29`. RULE 12: retail `dSv_event_c` indexes `mFlags[i_no >> 8]` by BYTE,
/// so the id is passed through untouched - never pre-shifted, pre-masked or renumbered.
const FLAG_UNK_0310: u32 = 0x0310;
/// `dComIfGs_isEventBit(dSv_event_flag_c::UNK_0001)` (`d_a_dk.cpp:264`), `d_save_event_flag.inc:13`.
const FLAG_UNK_0001: u32 = 0x0001;

/// `dComIfGp_evmng_startCheck("zelda_fly")` (`d_a_dk.cpp:259`).
const EVENT_ZELDA_FLY: &str = "zelda_fly";

#[derive(Component, Debug, Clone, PartialEq)]
pub struct Dk {
    /// `field_0xAC5` (`d_a_dk.cpp:390` and `:392`), the scripted state. Driven by the switch at
    /// `:252-271`.
    mode: i8,
}

impl Dk {
    /// `daDk_Create` (`:366-398`). The profile's `Parameters` field is `0` (`d_a_dk.cpp:431`) and
    /// the only read of the parameter is `fopAcM_GetParam(a_this)` at `:372`, stored into the
    /// `u8 field_0x2B4` (`d_a_dk.h:29`) - so the whole u32 is truncated by that store, which is
    /// reproduced by the `as u8` below. `field_0x2B4` itself is not kept as a field: its only
    /// other reads (`:274`, `:383`) gate the `field_0x900` accommodation, which is omitted, so
    /// keeping it would be an unread field.
    pub fn spawn(parameters: u32) -> Self {
        let param = parameters as u8;
        let mode = if param == PARAM_FLYING {
            MODE_BOSS
        } else {
            MODE_DORMANT
        };
        Self { mode }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `a_this->scale.x = a_this->scale.y = a_this->scale.z = l_HIO.field_0x08;`
    /// (`d_a_dk.cpp:278`). This OVERWRITES the RULE 2 stage scale of 1.0 every tick with the HIO
    /// default, and because `HIO_M08` is 1.0 the visible effect is that the root scale stays 1.0.
    pub fn scale(&self) -> f32 {
        HIO_M08
    }

    /// `daDk_Execute` (`:221-294`), the whole of its one `if (l_HIO.field_0x05 == 0)` block
    /// (`:230-272`).
    fn execute(
        &mut self,
        actor: &mut Actor,
        list: &EventList,
        active: &ActiveEvent,
        flags: &EventFlags,
    ) {
        if HIO_M05 != 0 {
            return;
        }

        // `move(a_this);` (`:231`) - an empty function in retail (`:205-206`).
        actor.eye = actor.pos + glam::Vec3::Y * EYE_LIFT;

        // `dDemo_setDemoData(...)` (`:236-241`) needs the demo actor registry, which does not exist
        // here. Retail's `else` arm (`:247-250`) plays the model and sets `field_0xAEE = 1`; its
        // only reader is `daDk_Draw` (`:185`), which is renderer-only and omitted.
        match self.mode {
            MODE_BOSS => {
                if flags.test(FLAG_UNK_0310) {
                    self.mode = MODE_FLY_START;
                }
            }
            // `case 2:` falls through into `case 3:` in retail (`:258-266`), so the UNK_0001 test
            // runs on this tick too.
            MODE_FLY_START => {
                if zelda_fly_running(list, active) {
                    self.mode = MODE_FLYING;
                }
                if flags.test(FLAG_UNK_0001) {
                    self.mode = MODE_RELEASED;
                }
            }
            MODE_FLYING => {
                if flags.test(FLAG_UNK_0001) {
                    self.mode = MODE_RELEASED;
                }
            }
            // `case 10: default: break;` (`:268-270`).
            _ => {}
        }

        actor.scale = glam::Vec3::splat(self.scale());
    }
}

/// `dComIfGp_evmng_startCheck("zelda_fly")` (`d_a_dk.cpp:259`). The name is resolved through
/// `EventList::event_index`, which is the house substitute for `dComIfGp_evmng_getEventIdx` and
/// the same substitution `mbdoor.rs`, `vgnfd.rs` and `zunari.rs` already use.
fn zelda_fly_running(list: &EventList, active: &ActiveEvent) -> bool {
    list.event_index(EVENT_ZELDA_FLY)
        .is_some_and(|slot| active.start_check(slot))
}

pub fn is_dk(name: &str) -> bool {
    DK_ACTORS.contains(&name)
}

pub fn control(
    list: Res<EventList>,
    active: Res<ActiveEvent>,
    flags: Res<EventFlags>,
    mut dragons: Query<(&mut Actor, &mut Dk)>,
) {
    for (mut actor, mut dragon) in &mut dragons {
        dragon.execute(&mut actor, &list, &active, &flags);
    }
}
