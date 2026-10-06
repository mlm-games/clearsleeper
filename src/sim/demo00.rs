//! The stage-cut actor shell - `fpcNm_DEMO00_e`, `d_a_demo00.cpp`, the actor behind the 32 stage
//! strings `d_act0` .. `d_act31`.
//!
//! These names are NOT stage-placed actor records in the ordinary sense: retail creates them itself.
//! `dDemo_system_c::JSGFindObject` (`d_demo.cpp:587-590`) looks the name up, finds nothing, and on a
//! `strncmp(name, "d_act", 5) == 0` hit calls `fopAcM_fastCreate(name, 0)` followed by
//! `fopAcM_setStageLayer(ac)` and `mObject->appendActor(ac)`. `appendActor`
//! (`d_demo.cpp:482-498`) is what assigns `ac->demoActorID = mNumActor`. An actor this sim spawns
//! from the stage table never goes through `appendActor`, so `demoActorID` stays at the `0` that
//! `fopAcM_ct` wrote, and `dDemo_object_c::getActor` (`d_demo.cpp:501-505`) answers
//! `if (no == 0 || no > ARRAY_SIZE(mpActors)) return NULL;`.
//!
//! `daDemo00_c::execute` (`:631-822`) opens with exactly that lookup:
//!
//! ```text
//! dDemo_actor_c* demo_actor = dComIfGp_demo_getActor(demoActorID);
//! if (demo_actor == NULL) { fopAcM_delete(this); } else { ... }
//! ```
//!
//! (`:636-639`). `fopAcM_delete` is a real destruction, not the RULE 6b delete request, so the whole
//! actor is one first-tick despawn. That is the same conclusion `dmgroom.rs` reaches about
//! `d_a_obj_dmgroom.cpp`, whose `_execute` is driven by the identical `demoActorID` lookup
//! (`d_a_obj_dmgroom.cpp:76`).
//!
//! (a) inbound damage: NONE. (b) outbound damage: NONE. `daDemo00_c` registers no `dCcD` collider -
//! its only ground query is `mModel.mBgc->mGndChk` (`:403`) - and the profile `Group` is
//! `fopAc_ACTOR_e` (`:863`). Neither `combat::HitPoints` nor `enemy::Hurt` is attached.
//!
//! `daDemo00_IsDelete` (`:825-828`) is the bare "accepts a delete request" form of RULE 6b.
//! `daDemo00_Create` (`:837-840`) delegates to `create` (`:37-46`), which always returns
//! `cPhs_COMPLEATE_e`, so nothing here refuses at spawn time.

use bevy_ecs::prelude::*;

/// `d_stage.cpp:1196-1227` binds all 32 spellings to `fpcNm_DEMO00_e` with `sub` running 0..31 -
/// that column is `dStage_objectNameInf::argument` (`d_stage.h:992`), not a parameter, and
/// `d_a_demo00_c` never reads it.
pub const DEMO00_ACTORS: [&str; 32] = [
    "d_act0", "d_act1", "d_act2", "d_act3", "d_act4", "d_act5", "d_act6", "d_act7", "d_act8",
    "d_act9", "d_act10", "d_act11", "d_act12", "d_act13", "d_act14", "d_act15", "d_act16",
    "d_act17", "d_act18", "d_act19", "d_act20", "d_act21", "d_act22", "d_act23", "d_act24",
    "d_act25", "d_act26", "d_act27", "d_act28", "d_act29", "d_act30", "d_act31",
];

/// `fopAc_CULLBOX_0_e` (`:864`) resolves through `l_cullSizeBox[0]` (`f_op_actor_mng.cpp:558`);
/// this file calls neither `fopAcM_setCullSizeBox` nor `fopAcM_SetMin` / `SetMax`.
const CULL_MIN: [f32; 3] = [-40.0, 0.0, -40.0]; // <- f_op_actor_mng.cpp:558 (fopAc_CULLBOX_0_e)
const CULL_MAX: [f32; 3] = [40.0, 125.0, 40.0]; // <- f_op_actor_mng.cpp:558 (fopAc_CULLBOX_0_e)

/// One demo-cut actor. `entered` is the one-shot guard: retail deletes on the first `Execute`
/// because the demo record is missing, so the check runs once and never again.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Demo00 {
    /// Guards `execute`'s single `fopAcM_delete(this)` (`:638`).
    entered: bool,
}

impl Demo00 {
    /// `create` (`:37-46`) minus `fopAcM_ct` (`:38`) and `dKy_tevstr_init` (`:39`), `setBaseMtx` (`:39`)
/// and the three stores `setAction(&daDemo00_c::actStandby)` (`:40`), `mNextID.reset()` (`:41`)
    /// and `field_0x29d = -1` (`:43`) - the action and the two ID records have no reader once the
    /// delete at `:638` is taken. The profile's `Parameters` field is `0` (`:859`).
    pub fn spawn(_parameters: u32) -> Self {
        Self { entered: false }
    }

    /// `fopAc_CULLBOX_0_e` (`:864`) -> `l_cullSizeBox[0]` (`f_op_actor_mng.cpp:558`).
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `execute` (`:631-822`), reduced to its first three statements and returning `true` for the
    /// `fopAcM_delete(this)` that closes `execute` for good. `field_0x29e = 0` (`:633`)
    /// has `draw` as its only reader (`:584`), and the entire `else` branch - `checkEnable`,
    /// `getShapeId`, `getAnmId`, `getPrm`, the `l_eventBit` / `l_itemNo` tables (`:651-726`),
    /// the monotone rate (`:743`), the rumble (`:752-757`), the fades (`:777-783`) and the
    /// `mNextID` key/value parse (`:788-815`) - needs the demo cut table this checkout does not
    /// parse.
    fn execute(&mut self) -> bool {
        if self.entered {
            return false;
        }
        self.entered = true;
        true
    }
}

pub fn is_demo00(name: &str) -> bool {
    DEMO00_ACTORS.contains(&name)
}

pub fn control(mut actors: Query<(Entity, &mut Demo00)>, mut commands: Commands) {
    let mut spent: Vec<Entity> = Vec::new();
    for (entity, mut demo) in &mut actors {
        if demo.execute() {
            spent.push(entity);
        }
    }
    for entity in spent {
        let _ = commands.entity(entity).despawn();
    }
}