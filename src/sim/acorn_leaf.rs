//! `daAleaf_c` (`tww/src/d/actor/d_a_acorn_leaf.cpp`): the leaf that hangs one acorn on Outset's
//! trees. It creates the acorn as its own child, lets that acorn grow for sixty-nine ticks, and
//! shakes whenever the acorn is picked off it, whenever the player puts something down beside it,
//! and whenever the player walks into it.

use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::carry::{Carry, Carryable};
use super::player::Player;
use super::tsubo::Tsubo;
use super::walk::from_s16;

/// `OBJNAME("VigaH", fpcNm_ACORN_LEAF_e, 255, 0)` (`d_stage.cpp:949`), the same string as
/// `m_arcname` (`d_a_acorn_leaf.cpp:48`).
pub const ACORN_LEAF_ACTORS: [&str; 1] = ["VigaH"];

/// `OBJNAME("Kmi02", fpcNm_TSUBO_e, 7, 0)` (`d_stage.cpp:528`): the stage spelling of the `TSUBO`
/// row whose `argument` is the acorn type. The reference names no string at all - it creates the
/// acorn through the `fpcNm_TSUBO_e` overload of `fopAcM_createChildFromOffset`
/// (`d_a_acorn_leaf.cpp:121`), which takes the proc id and an `argument` of `-1` (`:121`,
/// `f_op_actor_mng.cpp:233`), and `d_a_tsubo.cpp:1281` picks the data row from `prm_get_type()`
/// alone, which `prm_make_acorn` has already written as 7 into bits 24-27 (`d_a_tsubo.h:201-210`).
/// This name only has to be one of the twelve `super::tsubo::is_tsubo` accepts, and it is the one
/// that means an acorn.
const ACORN_ACTOR: &str = "Kmi02";

pub const NO_SET_ID: u16 = 0xFFFF;

/// `acorn_offset` (`d_a_acorn_leaf.cpp:16`).
const ACORN_OFFSET_Y: f32 = 15.0;

/// `unk_420 = 60` (`:105`): the wait after an acorn has gone before the next one is created.
const RESPAWN_FRAMES: i32 = 60;

/// `if (unk_41C < 69)` and `unk_41C = 70` (`:164`, `:168`, `:172`): the acorn's growth counter,
/// which the shake latch fires at the top and then parks so it cannot fire again.
const GROW_END: i32 = 69;
const GROWN: i32 = 70;

/// `var_f31 < 50.0f` (`:183`).
const NEAR_XZ: f32 = 50.0;
/// `(unk_42C - var_f31) > 3.0f` (`:183`): the player has closed more than this much in one tick.
const APPROACH_STEP: f32 = 3.0;
/// `std::fabsf(player->current.pos.y - current.pos.y) < 5.0f` (`:183`).
const HEIGHT_TOLERANCE: f32 = 5.0;
/// `var_f31 < 200.0f` (`:188`).
const PUT_NEAR_XZ: f32 = 200.0;
/// `var_f1 < 70.0f` (`:194`): how near the leaf the thing just put down has to be.
const PUT_AT_XZ: f32 = 70.0;

// -- parameter decode. `daAleaf_prm` (`:50-58`) reads both fields off `fopAcM_GetParam` on every
//    call, so they are functions of the `Actor`'s parameters and not state of their own.
const ITEM_NO_MASK: u32 = 0x3F; // <- getItemNo, :56
const ITEM_BIT_SHIFT: u32 = 6; // <- getItemBitNo, :52
const ITEM_BIT_MASK: u32 = 0x7F; // <- getItemBitNo, :52

/// `daAleaf_prm::getItemNo` (`:55-57`).
pub fn item_no(parameters: u32) -> u32 {
    parameters & ITEM_NO_MASK
}

/// `daAleaf_prm::getItemBitNo` (`:51-53`).
pub fn item_bit_no(parameters: u32) -> u32 {
    (parameters >> ITEM_BIT_SHIFT) & ITEM_BIT_MASK
}

const ACORN_RESPAWN_SPEC: u32 = 0x4; // <- prm_make_acorn's `a1`, d_a_tsubo.h:202
const ACORN_CREATE_SPEC: u32 = 0x3F; // <- ditto
const ACORN_SPEC_SHIFT: u32 = 8; // <- d_a_tsubo.h:206
const ACORN_BIT_SHIFT: u32 = 16; // <- d_a_tsubo.h:205
const ACORN_MARK: u32 = 7 << 24; // <- d_a_tsubo.h:207, the acorn's `prm_get_type` row
const ACORN_STICK: u32 = 1 << 31; // <- ditto, `PRM_STICK_S` (`d_a_tsubo.h:49`)

/// `daTsubo::Act_c::prm_make_acorn` (`d_a_tsubo.h:201-210`). The reference's own argument is
/// anonymous (`arg1`) and its two call sites name the respawn for it: `create_acorn_sub(true)`
/// (`d_a_acorn_leaf.cpp:104`) is the replacement and `create_acorn_sub(false)` (`:96`) is the
/// original, so the acorn the leaf is born with gets the carried-item `spec` of 0x3F and a
/// respawned one gets 4. Both are created stuck.
pub fn make_acorn_params(respawn: bool, item_no: u32, bit_no: u32) -> u32 {
    let spec = if respawn {
        ACORN_RESPAWN_SPEC
    } else {
        ACORN_CREATE_SPEC
    };
    ((bit_no & ITEM_BIT_MASK) << ACORN_BIT_SHIFT)
        | ((item_no & ITEM_NO_MASK) | (spec << ACORN_SPEC_SHIFT))
        | ACORN_MARK
        | ACORN_STICK
}

#[derive(Component, Debug, Clone, Copy)]
pub struct AcornLeaf {
    /// `unk_424` (`d_a_acorn_leaf.h:34`): the acorn's `fpc_ProcID`, which is this port's entity.
    pub acorn: Option<Entity>,
    /// `unk_420` (`:33`): the respawn countdown.
    pub respawn: i32,
    /// `unk_41C` (`:32`): the acorn's growth counter.
    pub grow: i32,
    /// `unk_418` (`:28`): the latch that says the shake BCK is playing.
    pub shaking: bool,
    /// `unk_419` (`:29`): last tick's `prm_get_stick` of the acorn.
    pub was_stuck: bool,
    /// `unk_428` (`:35`): last tick's `player->getGrabActorID()`.
    pub grab: Option<Entity>,
    /// `unk_42C` (`:36`): last tick's `absXZ` to the player.
    pub last_distance: f32,
}

impl AcornLeaf {
    /// The field initialisers `fopAc_ct` leaves behind, plus `CreateInit`'s two writes:
    /// `unk_424`/`unk_428` both start at `fpcM_ERROR_PROCESS_ID_e` (`d_a_acorn_leaf.cpp:93-94`) and
    /// `create_acorn_sub(false)` (`:96`) fills in the acorn.
    fn create(acorn: Option<Entity>) -> Self {
        Self {
            acorn,
            respawn: 0,
            grow: 0,
            shaking: false,
            was_stuck: false,
            grab: None,
            last_distance: 0.0,
        }
    }
}

/// `create_acorn_sub` (`:114-126`): the acorn bundle, with the parameters, the position and the
/// room the reference hands to `fopAcM_createChildFromOffset`. The reference returns
/// `fpcM_ERROR_PROCESS_ID_e` when the create table is full; a `Commands::spawn` always succeeds,
/// so this needs no failure arm.
fn acorn_child(leaf: &Actor, respawn: bool) -> (Actor, Tsubo, Carryable) {
    let parameters = make_acorn_params(
        respawn,
        item_no(leaf.parameters),
        item_bit_no(leaf.parameters),
    );
    (
        Actor {
            name: ACORN_ACTOR.to_owned(),
            parameters,
            pos: acorn_position(leaf),
            // `csXyz angle(angleOffs); angle.y += parentAngleY;` (`f_op_actor_mng.cpp:225-226`)
            // with a null `pAngleOffs` (`d_a_acorn_leaf.cpp:121`): the acorn takes the leaf's Y
            // angle and nothing else.
            angle: [0, leaf.angle[1], 0],
            set_id: NO_SET_ID,
            room: leaf.room,
            ..Default::default()
        },
        Tsubo::spawn(parameters),
        Carryable::Tsubo,
    )
}

/// `create_acorn_sub`'s X-rotation of the offset,
/// `mDoMtx_stack_c::XrotS(home.angle.x)` / `multVec(&acorn_offset, &sp10)` (`:117-119`).
/// `current.angle = home.angle` (`:92`) is that field's only write and no writer follows it, so the
/// leaf's own angle is what the offset is turned by, and then
/// `fopAcM_createChildFromOffset` swings the result around the parent
/// (`f_op_actor_mng.cpp:230-232`): only the Y component of the offset survives the X rotation, so
/// there is nothing to add in X.
fn acorn_position(leaf: &Actor) -> Vec3 {
    let (sin_x, cos_x) = from_s16(leaf.angle[0]).sin_cos();
    let offset = Vec3::new(0.0, ACORN_OFFSET_Y * cos_x, ACORN_OFFSET_Y * sin_x);
    let (sin_y, cos_y) = from_s16(leaf.angle[1]).sin_cos();
    Vec3::new(
        leaf.pos.x + offset.z * sin_y,
        leaf.pos.y + offset.y,
        leaf.pos.z + offset.z * cos_y,
    )
}

pub fn is_acorn_leaf(name: &str) -> bool {
    ACORN_LEAF_ACTORS.contains(&name)
}

pub fn control(
    mut commands: Commands,
    mut leaves: Query<(&Actor, &mut AcornLeaf)>,
    actors: Query<&Actor>,
    pots: Query<&Tsubo>,
    players: Query<&Player>,
    carry: Res<Carry>,
) {
    // `daPy_py_c* player = (daPy_py_c*)dComIfGp_getPlayer(0)` (`:152`): the reference dereferences
    // the result unconditionally, the sim returns until the player is in the stage.
    let Some(player) = players.iter().next() else {
        return;
    };
    let held = carry.held; // <- player->getGrabActorID(), :188
    for (actor, mut leaf) in &mut leaves {
        // `fopAc_ac_c* actor = fopAcM_SearchByID(unk_424)` (`:153`) - the acorn as it reads at the
        // top of the tick, before `create_acorn` (`:161`) has had its say, which is why a fresh
        // acorn only starts growing on the tick after it is created.
        let acorn = leaf.acorn.filter(|id| actors.get(*id).is_ok());

        // `bool var_r29 = false; if (actor != NULL) var_r29 = actor->prm_get_stick()` (`:155-159`).
        let stuck = acorn.is_some_and(|id| pots.get(id).is_ok_and(|pot| pot.stick));

        // `create_acorn` (`:100-111`): a lost acorn is replaced, and the countdown only runs while
        // it is gone - the decrement sits below the create in the reference, so the tick that
        // creates spends the first of its sixty frames.
        if leaf.respawn == 0 && acorn.is_none() {
            leaf.acorn = Some(commands.spawn(acorn_child(actor, true)).id());
            leaf.respawn = RESPAWN_FRAMES;
        }
        if leaf.respawn > 0 {
            leaf.respawn -= 1;
        }

        // `if (actor != NULL) { if (unk_41C < 69) unk_41C++; if (unk_41C == 69) {...} }`
        // (`:163-174`): the acorn grows while it exists, and the leaf shakes once it is out.
        // `setPlaySpeed(1.0f)` and `setFrame(0.0f)` (`:170-171`) are the BCK cursor's own restart.
        if acorn.is_some() {
            if leaf.grow < GROW_END {
                leaf.grow += 1;
            }
            if leaf.grow == GROW_END {
                leaf.shaking = true;
                leaf.grow = GROWN;
            }
        }

        // `if (!var_r29 && unk_419)` (`:176-179`): the stick bit went down since the last tick,
        // which is `mode_carry_init`'s `prm_off_stick()` (`d_a_tsubo.cpp:1911`) - the acorn has
        // just been picked off the leaf.
        if !stuck && leaf.was_stuck {
            leaf.shaking = true;
        }

        // `f32 var_f31 = (player->current.pos - current.pos).absXZ()` (`:181`).
        let distance = abs_xz(player.pos - actor.pos);

        // `:183-186`: the player walking into the leaf.
        if distance < NEAR_XZ
            && (leaf.last_distance - distance) > APPROACH_STEP
            && (player.pos.y - actor.pos.y).abs() < HEIGHT_TOLERANCE
        {
            leaf.shaking = true;
        }

        // `:188-199`: the player has just let go of something - the grab id changed and now reads
        // `fpcM_ERROR_PROCESS_ID_e` - and what it let go of is still within 70 units of the leaf.
        if distance < PUT_NEAR_XZ && leaf.grab != held && held.is_none() {
            if let Some(previous) = leaf.grab {
                if let Ok(dropped) = actors.get(previous) {
                    if abs_xz(dropped.pos - actor.pos) < PUT_AT_XZ {
                        leaf.shaking = true;
                    }
                }
            }
        }

        // `if (unk_418 && unk_408.play()) unk_418 = false;` (`:201-203`): `mDoExt_baseAnm::play()`
        // steps the cursor and returns `isStop()` (`m_Do_ext.cpp:116-120`), and the cursor's end
        // frame is `initPlay(mpAnm->getFrameMax(), ...)` (`:454`) - archive data, which this
        // checkout does not carry, so the latch's one clear is unreachable and no stand-in count is
        // invented for it. `set_mtx` (`d_a_acorn_leaf.cpp:205`, body `:143-148`) is matrix work for
        // `_draw` (`:215-224`) and is the renderer's.
        leaf.was_stuck = stuck;
        leaf.grab = held;
        leaf.last_distance = distance;
    }
}

/// `cXyz::absXZ` (`:181`, `:192`): the horizontal length of a difference vector.
fn abs_xz(value: Vec3) -> f32 {
    (value.x * value.x + value.z * value.z).sqrt()
}

pub fn spawn(world: &mut World, source: &crate::dzs::StageActor) -> Entity {
    let actor = Actor {
        name: source.name.clone(),
        parameters: source.parameters,
        pos: glam::Vec3::from_array(source.pos),
        angle: source.angle,
        set_id: source.set_id,
        room: source.room,
        ..Default::default()
    };
    // `CreateInit`'s `create_acorn_sub(false)` (`:96`): the leaf is born holding an acorn, and the
    // `unk_424` write at `:121` is what the tick's `fopAcM_SearchByID` (`:153`) reads back.
    let acorn = world.spawn(acorn_child(&actor, false)).id();
    let leaf = AcornLeaf::create(Some(acorn));
    world.spawn((actor, leaf)).id()
}
