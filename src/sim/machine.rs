use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::events::{SimEvent, SimEvents};
use super::walk;
use super::windmill::Windmill;

/// `OBJNAME("Hkikai1", fpcNm_MACHINE_e, 255, 0)` (`d_stage.cpp:1030`) is the only row bound to
/// `fpcNm_MACHINE_e` (`d_a_machine.cpp:437`), and the profile's arcname is the same string
/// (`d_a_machine.cpp:73`, `dComIfG_resLoad(&mPhs, m_arcname)` at `:256`).
///
/// The stage string and `daMachine_c::m_arcname` COINCIDE here, which is not the usual case; and
/// the file name spells the class `daMachine_c`, not `Hkikai1`.
pub const MACHINE_ACTORS: [&str; 1] = ["Hkikai1"];

/// `fopAcM_setCullSizeBox(this, -100.0f, -100.0f, -50.0f, 100.0f, 100.0f, 500.0f)`
/// (`d_a_machine.cpp:138`) - the actor's own CUSTOM box, so the profile's `fopAc_CULLBOX_CUSTOM_e`
/// (`:447`) resolves here and `l_cullSizeBox` is not consulted.
const CULL_MIN: [f32; 3] = [-100.0, -100.0, -50.0]; // <- d_a_machine.cpp:138
const CULL_MAX: [f32; 3] = [100.0, 100.0, 500.0]; // <- d_a_machine.cpp:138

/// `JA_SE_OBJ_JAMA_MECHA_MOVE` (`JAZelAudio_SE.h:2151`), the only sound
/// `daMachine_c::_execute` can emit on its own.
pub const SE_MOVE: u32 = 0x7045; // <- JAZelAudio_SE.h:2151 (d_a_machine.cpp:295)

/// `f32 f31 = 5.0f;` (`d_a_machine.cpp:236`) - the speed scale applied to the windmill's
/// normalised spin rate.
pub const SPEED_SCALE: f32 = 5.0; // <- d_a_machine.cpp:236
/// `cLib_addCalc(&old_speedF, f31 * f1, 0.1f, 1.0f, 0.5f)` (`d_a_machine.cpp:248`) - the `scale`,
/// `maxStep` and `minStep` arguments, in that order (`c_lib.h:15`).
pub const SPEED_ADD: f32 = 0.1; // <- d_a_machine.cpp:248
pub const SPEED_MAX: f32 = 1.0; // <- d_a_machine.cpp:248
pub const SPEED_MIN: f32 = 0.5; // <- d_a_machine.cpp:248

/// `const s16 daWindMill_c::m_max_rot_speed[] = { 0x09C4, 0x09C4 };` (`d_com_static.cpp:374-377`),
/// read at `d_a_machine.cpp:241` as `daWindMill_c::m_max_rot_speed[windmill->mType]`. Both entries
/// are `2500`, the same `i16` value, so one constant carries the pair; the length is what makes
/// the index a two-slot lookup rather than a single scalar.
pub const MILL_ROT_MAX: [f32; 2] = [0x09C4 as f32, 0x09C4 as f32]; // <- d_com_static.cpp:374-377
pub const MILL_ROT_MAX_COUNT: usize = 2; // <- d_com_static.cpp:374-377

/// The Wind Temple giant-fan blade trap.
///
/// (a) inbound damage: NONE ATTACHED, and none is reachable. `daMachine_c` is hittable in retail -
/// `l_sph_src_col` (`d_a_machine.cpp:43-71`) sets `SrcObjTg SPrm` to
/// `cCcD_TgSPrm_Set_e | cCcD_TgSPrm_IsOther_e` (`:51`) and `mStts.Init(255, 255, this)` (`:139`)
/// gives it 255 hit points - but every collider in this translation unit is `cCcD` registry work
/// (`mSph.Set`, `SetStts`, `dComIfG_Ccsp()->Set`) and RULE 3b lists the `cCcD` actor-collision
/// registry as confirmed-absent. There is therefore no channel on which a sword swing could reach
/// the machine, so attaching `combat::HitPoints` would only let `combat::control` delete an actor
/// that nothing in the sim can damage.
///
/// (b) outbound damage: NONE ATTACHED. `l_sph_src_at` (`:14-41`) is an `AT_TYPE_UNK800` attack
/// collider against the player (`:18`, `:20`) and `set_body` arms it (`:371`), but the same absent
/// registry carries it. The `attack()` state machine (`:301-353`) is transcribed below only as far
/// as its state field; see the omissions on `attack`.
///
/// `daMachine_IsDelete` (`:421-423`) is the bare "accepts a delete request" form of RULE 6b, and
/// `fopAcM_delete` appears nowhere in this file, so `spawn` never refuses and there is no
/// `refused` flag.
///
/// RULE 2: `scale.setall(1.5f)` (`d_a_machine.cpp:134`) is the actor's real scale, not the
/// stage-placed 1.0, and its only readers are `set_mtx` (`:275`) and the model base scale - both
/// renderer. `dStage_actorInit` still never writes `fopAcM_prm_class::scale`, so nothing about
/// placement depends on it here and it is not stored.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Machine;

impl Machine {
    pub fn spawn() -> Self {
        Self
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

/// `daMachine_c::set_speed` (`d_a_machine.cpp:235-250`).
///
/// ```
/// f32 f31 = 5.0f;
/// daWindMill_c* windmill = search_wind_mill();
/// if (windmill != NULL) { f1 = (f32)windmill->mAngle[1] / (f32)daWindMill_c::m_max_rot_speed[windmill->mType]; }
/// else { f1 = 0.0f; }
/// f32 old_speedF = speedF;
/// cLib_addCalc(&old_speedF, f31 * f1, 0.1f, 1.0f, 0.5f);
/// speedF = old_speedF;
/// ```
///
/// `search_wind_mill` (`:230-232`) is `fopAcM_SearchByName(fpcNm_WINDMILL_e)`, and
/// `d_stage.cpp:1028-1029` binds `Hpu1` / `Hpu2` to that enum - the two actors
/// `src/sim/windmill.rs` already owns. `src/sim/windmill.rs` writes `Actor::angle[1]` from
/// `Windmill::turn()` (`windmill.rs:256`, `:128-131`), so `Windmill::spin` IS retail's
/// `mAngle[1]`, and `Windmill::kind` IS `daWindMill_c::mType`.
fn set_speed(actor: &mut Actor, mill: Option<&Windmill>) {
    let rate = match mill {
        Some(mill) => {
            // `daWindMill_c::m_max_rot_speed[windmill->mType]` (`:241`). `m_max_rot_speed` has
            // exactly two entries (`d_com_static.cpp:374-377`) and `windmill.rs` already reduces
            // `mType` to `0..2` (`windmill.rs:84`), but RULE 10 still requires the runtime index to
            // be guarded rather than trusted.
            let index = usize::from(mill.kind) % MILL_ROT_MAX_COUNT;
            mill.spin as f32 / MILL_ROT_MAX[index]
        }
        None => 0.0,
    };
    let target = SPEED_SCALE * rate;
    // `cLib_addCalc(&old_speedF, ...)` mutates a local that is copied back afterwards (`:247-249`),
    // so the read of `speedF` happens once, before the chase.
    let mut speed_f = actor.speed_f;
    add_calc(
        &mut speed_f,
        target,
        SPEED_ADD,
        SPEED_MAX,
        SPEED_MIN,
    );
    actor.speed_f = speed_f;
}

/// `cLib_addCalc(f32* pValue, f32 target, f32 scale, f32 maxStep, f32 minStep)`
/// (`c_lib.h:15`, `c_lib.cpp:21-54`). Unlike `cLib_addCalc2` (`c_lib.cpp:56-66`) this one has the
/// `minStep` arm: when the proportional step is smaller than `minStep` it commits `minStep`
/// instead and clamps the result back onto the target from whichever side it overshot. All four
/// arguments are non-zero here, so no division or signed-zero branch can arise.
fn add_calc(value: &mut f32, target: f32, scale: f32, max_step: f32, min_step: f32) {
    if *value == target {
        return;
    }
    let mut step = scale * (target - *value);
    if step.abs() >= min_step {
        step = step.clamp(-max_step, max_step);
        *value += step;
    } else if step > 0.0 {
        *value += min_step;
        if *value > target {
            *value = target;
        }
    } else {
        *value -= min_step;
        if *value < target {
            *value = target;
        }
    }
}

/// `daMachine_c::_execute` (`d_a_machine.cpp:282-299`), minus the omitted halves.
pub fn control(
    windmills: Query<(With<Windmill>, Without<Machine>)>,
    mut machines: Query<(With<Machine>, &mut Actor)>,
    mut events: ResMut<SimEvents>,
) {
    let mill = windmills.iter().next().map(|(_, mill)| *mill);
    for (_, mut actor) in &mut machines {
        // `set_speed();` (:284)
        set_speed(&mut *actor, mill.as_ref());
        // ```
        // xyz = cXyz::Zero - current.pos;
        // current.angle.y = cM_atan2s(xyz.x, xyz.z);
        // ```
        // (:285-286). The machine always faces the WORLD ORIGIN, not the player and not a
        // travel direction - the minus signs are on both components.
        let to_origin = Vec3::ZERO - actor.pos;
        actor.angle[1] = walk::to_s16(to_origin.x.atan2(to_origin.z));
        // `path_move();` (:287) - OMITTED, see the report: `dPath_GetRoomPath` is a confirmed-absent
        // input (RULE 3b), so `CreateInit`'s `:184-186` else arm leaves `field_0xc14 == 0xFF` on
        // every actor and `path_move`'s own guard (`:192`) returns immediately. `current.pos`
        // therefore never changes in this sim and needs no field.
        //
        // `attack(); set_body(); set_at(); set_mtx();` (:288-291) - OMITTED: `attack`'s trigger
        // predicate is unreadable in this checkout and `set_body` / `set_at` are `cCcD` registry
        // plus BCK-frame work. See the report.
        //
        // ```
        // f32 temp = speedF;
        // if (temp != 0.0f) { fopAcM_seStart(this, JA_SE_OBJ_JAMA_MECHA_MOVE, 0); }
        // ```
        // (:293-296) - the move loop sound, emitted every tick while the blade is travelling.
        if actor.speed_f != 0.0 {
            events.list.push(SimEvent::Se(SE_MOVE));
        }
    }
}

pub fn is_machine(name: &str) -> bool {
    MACHINE_ACTORS.contains(&name)
}
