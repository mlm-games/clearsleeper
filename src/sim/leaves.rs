//! `daObjLeaves_c` (`tww/src/d/actor/d_a_obj_leaves.cpp`): the pile of orange leaves a stage hides
//! an item under. A bomb blast, a Deku Leaf gust or anything burning ignites it, and `m44C` then
//! hands the actor over to a three-stage fade that creates the item once `m408` has fallen to 200
//! and deletes the pile once it reaches zero. This port surveys the bomb, fire-arrow and skull
//! hammer sources that can reach the trigger sphere; the other arms are listed below.
//!
//! One tick, per `_execute` (`:704-718`): `m430` and `m434` are decremented and clamped to zero,
//! then `mMemberFunc` runs. `mMemberFunc` is one of `wait_proc` (`:560-597`),
//! `alpha_calc_start_wait_proc` (`:600-620`), `item_set_wait_proc` (`:623-655`) and
//! `retire_wait_proc` (`:658-688`), which the `Step` enum mirrors in that order.
//!
//! The hit that starts all of this arrives from outside the procs. `wait_proc` registers the
//! trigger sphere with `dComIfG_Ccsp()->Set(&mSph)` (`:575`) and the collision manager calls
//! `tg_hitCallback` (`:384-452`) later in the same tick; the callback only writes `m44C` and starts
//! particles. The *next* tick's proc tests the flag at `ClrTgHit` / `if (!m44C)` (`:571-572`),
//! latches the save switch with `param_on_swSave` (`:577`) and steps to the fade, so a hit is
//! always followed by one quiet tick before the switch is set.
//! That is why `control` surveys the hit sources after `Leaves::execute`, not before.
//!
//! Not modelled, and why:
//!
//! - the `AT_TYPE_WIND` arm (`:394-406`), which dissolves the pile as the bomb arm does. This crate
//!   has no `AT_TYPE_WIND` emitter: `acorn_leaf` carries the Deku Leaf without a collider, and
//!   `wind::control` moves stage gusts rather than the leaf's blow, so the arm has no source to
//!   fire on. Its `birthEffect` pair selects the FAN and SMOKE rows (`:208`, `:209`), whose three
//!   counters match the `ID_AK_SN_FALLENLEAFBOMB00` row the bomb arm picks (`:213`), so even with a
//!   source only the two emitters would be lost.
//! - the `AT_TYPE_FIRE` and `AT_TYPE_UNK20000` labels sharing the fire arm (`:441-443`). No actor
//!   here raises an `AT_TYPE_FIRE` collider - the same refusal `bita` records - so the fire arrow
//!   carries that arm alone.
//! - the sixteen melee types at `:417-433`. Their sole state effect is `birthEffect(2)`, the `RUN`
//!   row, which writes `m430 = 0`, `m434 = 1` and `m40C = -12` without setting `m44C`: the pile
//!   stays in `wait_proc`, where nothing reads those three, and the next real hit overwrites them.
//!   `AT_TYPE_SKULL_HAMMER` (`:436-439`) is kept because it plays a sound.
//! - `rideCallBack` (`:257-275`), which needs `dBgW`'s ride notification and
//!   `daPy_py_c::getFootOnGround`; the former is never registered here and no foot-on-ground feed
//!   exists, so its `birthEffect(2)` - already unobservable, per the melee note - is unreachable.
//! - `m438` (`:224`), which only the demo build's `_delete` (`:529`) reads, and
//!   `fopAcM_OffStatus` / `mpBgW->ChkUsed` / `dComIfG_Bgsp()->Release` (`:578`, `:580-582`,
//!   `:535-546`), which are background-collision and renderer bookkeeping.
//! - the model, its materials and the particle emitters. Only `m408`'s effect on visibility is
//!   kept, because `set_material_sub` hides every shape at alpha zero (`:723-724`) and `_draw`
//!   feeds it `m408` (`:753`).
//! - the JPN-only fire cylinder: `l_fire_cyl_src` (`:75-103`), `registFireCollision` (`:455-461`)
//!   and its registration under `#if VERSION <= VERSION_JPN` (`:497-501`).
//! - the demo build's tuning: `l_HIO` (`:14-40`) and the `daObjLeaves_HIO_c` constructor
//!   (`:107-122`); this port follows the USA-and-later path throughout.
//! - the item-table band. `fopAcM_createItemFromTable` resolves `0x20..=0x3E` through
//!   `dComIfGp_getItemTable()` (`f_op_actor_mng.cpp:819`), whose tables come from per-stage data
//!   this crate does not carry, so a pile configured with a table number drops nothing - the same
//!   refusal `switem::created_item` makes.

use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::arrow::{self, Arrow, Phase, TYPE_FIRE};
use super::bombflower;
use super::events::{SimEvent, SimEvents};
use super::flags::Switches;
use super::hammer::{self, Cut, Hammer};
use super::health::Health;
use super::pickup::{ITEM_ACTORS, Pickup};
use super::player::Player;
use super::switem;
use super::walk;

/// `OBJNAME("Vochi", fpcNm_Obj_Leaves_e, 0, 0)` (`d_stage.cpp:1125`), the same string as
/// `l_arcname` (`:43`).
pub const LEAF_ACTORS: [&str; 1] = ["Vochi"];

/// `if (switchIndex != 0xff)` in `chk_appear` (`:161`), and the `swSave != 0xFF` guard on the
/// latch (`d_a_obj_leaves.h:32`).
pub const NO_SWITCH: u8 = 0xFF;

/// `l_sph_src`: `/* Radius */ 150.0f`, re-applied by `mSph.SetR(150.0f)`.
pub const SPH_RADIUS: f32 = 150.0; // <- d_a_obj_leaves.cpp:151, :494

/// `cXyz sp08(current.pos.x, current.pos.y - 115.0f, current.pos.z)`, the trigger sphere's centre.
pub const SPH_DROP: f32 = 115.0; // <- d_a_obj_leaves.cpp:491

/// The `default:` arm's gate on the hit point: `absXZ` against `current.pos`, `if (abs < 110.0f)`.
pub const HIT_REACH: f32 = 110.0; // <- d_a_obj_leaves.cpp:414-415

/// `fopAc_CULLBOX_3_e` is `fopAc_cullSizeBox(cXyz(-75.0f, 0.0f, -75.0f), cXyz(75.0f, 150.0f,
/// 75.0f))`, the box `g_profile_Obj_Leaves` selects.
pub const CULL_HALF: f32 = 75.0; // <- f_op_actor_mng.cpp:563, d_a_obj_leaves.cpp:805

/// The same box's top, `150.0f`.
pub const CULL_TOP: f32 = 150.0; // <- f_op_actor_mng.cpp:563

/// `dItemNo_NONE_e`, which `fopAcM_createItemFromTable` refuses outright.
pub const ITEM_NONE: u8 = 0x3F; // <- f_op_actor_mng.cpp:894

/// `if (i_itemNo >= 0x20 && i_itemNo <= 0x3E)`, the branch that needs the absent item table.
pub const TABLE_LOW: u8 = 0x20; // <- f_op_actor_mng.cpp:819

/// The upper end of the same band.
pub const TABLE_HIGH: u8 = 0x3E; // <- f_op_actor_mng.cpp:819

/// `sp10.y -= 30.0f;` before `fopAcM_createItemFromTable`.
pub const ITEM_DROP: f32 = 30.0; // <- d_a_obj_leaves.cpp:640

/// `int switchNo = 0xFF; prmAngle.z = switchNo;` - `fopAcM_fastCreateItem2` overwrites the third
/// angle of every item it makes, whatever the caller passed.
const ITEM_SWITCH_ANGLE: i16 = 0xFF; // <- f_op_actor_mng.cpp:1019-1020

/// `m408 = 255.0f`, the opaque start.
pub const OPAQUE: f32 = 255.0; // <- d_a_obj_leaves.cpp:505

/// `if (m408 <= 200.0f)`, the threshold at which the item is created.
pub const ITEM_TOP: f32 = 200.0; // <- d_a_obj_leaves.cpp:631

/// `m408 = 0.0f`, the clamp `retire_wait_proc` applies before it tests `m434`.
pub const TRANSPARENT: f32 = 0.0; // <- d_a_obj_leaves.cpp:679

pub const SE_BLOW: u32 = 0x69EF; // <- JAZelAudio_SE.h:2007 (JA_SE_OBJ_OCHIBA_BLOW)
pub const SE_BURN: u32 = 0x69F0; // <- JAZelAudio_SE.h:2008 (JA_SE_OBJ_OCHIBA_BURN)

/// `particle_data[]`: the counters `birthEffect` copies out of the row it selects - `m08` into
/// `m430`, `m0C` into `m434` and `m10` into `m40C`. `m00` names an emitter and `m04` picks the
/// particle call, neither of which this sim runs, and the three `(0, 1, -12)` rows it would use are
/// listed under what is not modelled above.
const PARTICLE_DATA: [(i32, i32, f32); 6] = [
    (0, 1, -12.0),   // <- d_a_obj_leaves.cpp:208 (ID_AK_SN_FALLENLEAFFAN00)
    (0, 1, -12.0),   // <- d_a_obj_leaves.cpp:209 (ID_AK_ST_FALLENLEAFSMOKE00)
    (0, 1, -12.0),   // <- d_a_obj_leaves.cpp:210 (ID_AK_SN_FALLENLEAFRUN00)
    (60, 120, -3.0), // <- d_a_obj_leaves.cpp:211 (ID_AK_SN_FALLENLEAFFIRE00)
    (60, 120, -3.0), // <- d_a_obj_leaves.cpp:212 (ID_AK_SN_FALLENLEAFFIRE01)
    (0, 1, -12.0),   // <- d_a_obj_leaves.cpp:213 (ID_AK_SN_FALLENLEAFBOMB00)
];

/// `birthEffect`'s `arg1`, the index into `PARTICLE_DATA` (`:217`, `:222`).
const EFFECT_FIRE_0: usize = 3;
const EFFECT_FIRE_1: usize = 4;
const EFFECT_BOMB: usize = 5;

const ITEM_SHIFT: u32 = 0;
const ITEM_MASK: u32 = 0x3F;
const BIT_SHIFT: u32 = 6;
const BIT_MASK: u32 = 0x7F;
const SAVE_SHIFT: u32 = 13;
const SAVE_MASK: u32 = 0xFF;

/// `mMemberFunc` (`d_a_obj_leaves.h:80`), the four procs at `:560-688` installed in order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// `wait_proc`, still registering the sphere for `tg_hitCallback`.
    Wait,
    /// `alpha_calc_start_wait_proc`, waiting out the emitter's `m430`.
    Alpha,
    /// `item_set_wait_proc`, fading until the item can be created.
    Item,
    /// `retire_wait_proc`, fading to nothing and then deleting the actor.
    Retire,
}

/// Which arm of `tg_hitCallback`'s `switch (cVar5)` (`:392`) the pile is standing in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Hit {
    /// `case AT_TYPE_BOMB:`, taken straight out of the outer switch (`:407-411`).
    Bomb,
    /// `case AT_TYPE_FIRE_ARROW:`, inside the `default:` arm's distance gate (`:413`, `:441-448`).
    FireArrow,
    /// `case AT_TYPE_SKULL_HAMMER:`, inside the same gate (`:413`, `:436-439`).
    Hammer,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Leaves {
    /// `m43C`: `chk_appear` (`:158-165`), latched once by `_create` (`:470-472`).
    pub appear: bool,
    /// `param_get_swSave` (`d_a_obj_leaves.h:29`), the switch this pile sets when it dissolves.
    pub switch_no: u8,
    /// `param_get_itemNo` (`d_a_obj_leaves.h:27`), the item number handed to
    /// `fopAcM_createItemFromTable`.
    pub item: u8,
    /// `param_get_itemSaveBitNo` (`d_a_obj_leaves.h:28`), its story bit.
    pub item_bit: u8,
    /// `mMemberFunc`.
    step: Step,
    /// `m408`, the model alpha.
    alpha: f32,
    /// `m40C`, the delta `m408` moves by each tick of the last two procs.
    fade: f32,
    /// `m430`, the emitter wait the alpha proc counts down.
    wait: i32,
    /// `m434`, the deletion wait the retire proc counts down.
    life: i32,
    /// `m44C`, set by a dissolving `tg_hitCallback` and consumed by the next `wait_proc`.
    dissolve: bool,
}

impl Leaves {
    /// `_create` (`:465-522`) from `fopAcM_ct` through `mMemberFunc = &daObjLeaves_c::wait_proc`,
    /// minus the resource load, the model, the background object and the collider registration.
    ///
    /// The three parameter fields are `daObj::PrmAbstract`, which is
    /// `((1 << width) - 1) & (param >> shift)` (`d_a_obj.h:12-15`) over the `Param_e` widths and
    /// shifts at `d_a_obj_leaves.h:15-24`.
    pub fn spawn(parameters: u32, switches: &Switches) -> Self {
        let switch_no = ((parameters >> SAVE_SHIFT) & SAVE_MASK) as u8;
        Self {
            // `if (fopAcM_IsFirstCreating(this)) { m43C = chk_appear(); }` (`:470-472`), and a
            // spawn is always the first creating of that actor.
            appear: switch_no == NO_SWITCH || !switches.test(switch_no),
            switch_no,
            item: ((parameters >> ITEM_SHIFT) & ITEM_MASK) as u8,
            item_bit: ((parameters >> BIT_SHIFT) & BIT_MASK) as u8,
            step: Step::Wait,
            alpha: OPAQUE,
            fade: 0.0,
            wait: 0,
            life: 0,
            dissolve: false,
        }
    }

    /// `mSph.SetC(sp08)` with `sp08` at `current.pos.y - 115.0f`.
    pub fn centre(&self, pos: Vec3) -> Vec3 {
        Vec3::new(pos.x, pos.y - SPH_DROP, pos.z)
    }

    /// `fopAc_CULLBOX_3_e`, which `g_profile_Obj_Leaves` selects at `:805`.
    pub fn cull(&self) -> (f32, f32) {
        (CULL_HALF, CULL_TOP)
    }

    /// `set_material(mModel, m408)` hides every joint's mesh when the alpha byte is zero (`:753`,
    /// `:723-724`), which is what makes an un-created or fully retired pile disappear.
    pub fn drawn(&self) -> bool {
        self.appear && self.alpha as u8 != 0
    }

    /// `birthEffect(arg1)` without the two particle calls: the selected row's `m08`, `m0C` and
    /// `m10` become `m430`, `m434` and `m40C` (`:222`, `:247-252`).
    fn birth_effect(&mut self, index: usize) {
        let (wait, life, fade) = PARTICLE_DATA[index];
        self.wait = wait;
        self.life = life;
        self.fade = fade;
    }

    /// `param_on_swSave()` (`:577`), which is `fopAcM_onSwitch` unless the number is 0xFF
    /// (`d_a_obj_leaves.h:30-35`).
    fn latch(&self, switches: &mut Switches) {
        if self.switch_no != NO_SWITCH {
            switches.set(self.switch_no);
        }
    }

    /// `tg_hitCallback` (`:384-452`) for one hit object, as far as this sim can carry it. The
    /// caller has already reproduced `wait_proc`'s gate (`:571-575`): a pile that has dissolved is
    /// no longer in the collision space, so no callback reaches it.
    ///
    /// `at` stands in for `mSph.GetTgHitPosP()` (`:414`), the intersection point the reference
    /// measures, and the returned id is the one `fopAcM_seStartCurrent` plays at `current.pos`
    /// (`:403`, `:409`, `:438`, `:446`).
    fn struck(&mut self, hit: Hit, at: Vec3, pos: Vec3) -> Option<u32> {
        match hit {
            // No distance test: the sphere overlap that raised the callback is the whole gate
            // (`:407-411`).
            Hit::Bomb => {
                self.birth_effect(EFFECT_BOMB);
                self.dissolve = true;
                Some(SE_BLOW)
            }
            // `:441-448`. The two fire rows carry the same `(60, 120, -3.0)`, so the second call
            // exists only to mirror the second `birthEffect`.
            Hit::FireArrow if near(at, pos) => {
                self.birth_effect(EFFECT_FIRE_0);
                self.birth_effect(EFFECT_FIRE_1);
                self.dissolve = true;
                Some(SE_BURN)
            }
            // `:436-439`: the sound and the `BOMB00` row, but no `m44C`, so the pile goes on
            // waiting and stays in the collision space.
            Hit::Hammer if near(at, pos) => {
                self.birth_effect(EFFECT_BOMB);
                Some(SE_BLOW)
            }
            _ => None,
        }
    }

    /// The `VERSION >= VERSION_JPN` head of `_execute` (`:704-715`): both counters fall by one and
    /// stop at zero, before whichever proc runs.
    fn advance(&mut self) {
        self.wait = self.wait.saturating_sub(1);
        self.life = self.life.saturating_sub(1);
    }

    /// `(this->*mMemberFunc)()` (`:716`).
    fn execute(
        &mut self,
        entity: Entity,
        actor: &Actor,
        health: &Health,
        switches: &mut Switches,
        commands: &mut Commands,
    ) {
        match self.step {
            Step::Wait => self.wait_proc(switches),
            Step::Alpha => self.alpha_proc(),
            Step::Item => self.item_proc(actor, health, commands),
            Step::Retire => self.retire_proc(entity, commands),
        }
    }

    /// `wait_proc` (`:560-597`). `mpBgW->Move()` and `mStts.Move()` (`:565-566`) are background and
    /// collider motion, and `dComIfG_Ccsp()->Set(&mSph)` (`:575`) is the survey `control` runs in
    /// its place, so only the arm taken when `m44C` is set has an effect here.
    fn wait_proc(&mut self, switches: &mut Switches) {
        if !self.dissolve {
            return;
        }
        self.latch(switches);
        self.step = Step::Alpha;
    }

    /// `alpha_calc_start_wait_proc` (`:600-620`): the demo build compares the emitter's frame
    /// number to `m430`, the USA and later builds just wait the count out.
    fn alpha_proc(&mut self) {
        if self.wait <= 0 {
            self.step = Step::Item;
        }
    }

    /// `item_set_wait_proc` (`:623-655`): `m408 += m40C` until `m408 <= 200.0f`, then one item
    /// create and the hand-over to the retire proc.
    fn item_proc(&mut self, actor: &Actor, health: &Health, commands: &mut Commands) {
        self.alpha += self.fade;
        if self.alpha > ITEM_TOP {
            return;
        }
        self.release(actor, health, commands);
        self.step = Step::Retire;
    }

    /// `retire_wait_proc` (`:658-688`): the same delta clamped at zero, and `fopAcM_delete` once
    /// `m434` has run out. The reference plays nothing on the way out.
    fn retire_proc(&mut self, entity: Entity, commands: &mut Commands) {
        self.alpha += self.fade;
        if self.alpha > TRANSPARENT {
            return;
        }
        self.alpha = TRANSPARENT;
        if self.life <= 0 {
            let _ = commands.entity(entity).despawn();
        }
    }

    /// The create `item_set_wait_proc` asks for,
    /// `fopAcM_createItemFromTable(&sp10, item, itemSaveBit, fopAcM_GetHomeRoomNo(this), 0, &angle,
    /// 1, NULL)` (`:641`), which is `switem`'s existing translation of that function.
    ///
    /// `0x3F` is the reference's own refusal (`f_op_actor_mng.cpp:894`), and `0xFF` cannot be
    /// encoded in a six-bit field; the table band is refused for the reason given at the top of
    /// this module. The `dItemNo_TRIPLE_HEART_e` and `dItemNo_RECOVER_FAIRY_e` special cases
    /// (`f_op_actor_mng.cpp:1027-1036`) are not modelled, matching `switem`.
    ///
    /// `angle` is the caller's `csXyz angle(0, 0, 0)` (`d_a_obj_leaves.cpp:638`) with its third
    /// component overwritten
    /// by `fopAcM_fastCreateItem2`, and the randomised `current.angle.y` action 1 asks for
    /// (`daItem_c::initAction`, `d_a_item.cpp:1394`) is not reproducible in a deterministic sim.
    fn release(&self, actor: &Actor, health: &Health, commands: &mut Commands) {
        if self.item == ITEM_NONE || (TABLE_LOW..=TABLE_HIGH).contains(&self.item) {
            return;
        }
        // `getItemNoByLife` (`f_op_actor_mng.cpp:897`), then
        // `MAKE_ITEM_PARAMS(itemNo, i_itemBitNo, -1, 0, 1)` (`f_op_actor_mng.cpp:1024`).
        let item = switem::item_no_by_life(self.item, health);
        let parameters = switem::item_parameters(item, self.item_bit);
        commands.spawn((
            Actor {
                name: ITEM_ACTORS[0].to_owned(),
                parameters,
                pos: actor.pos + Vec3::new(0.0, -ITEM_DROP, 0.0),
                angle: [0, 0, ITEM_SWITCH_ANGLE],
                set_id: 0,
                room: actor.room,
                ..Default::default()
            },
            Pickup::spawn(parameters),
        ));
    }
}

/// `(*i_this->mSph.GetTgHitPosP() - a_this->current.pos).absXZ()`, tested `if (abs < 110.0f)`
/// (`:414-415`): horizontal only and strict, so a hit directly above the pile fails even though the
/// trigger sphere still reaches it.
fn near(hit: Vec3, pos: Vec3) -> bool {
    let dx = hit.x - pos.x;
    let dz = hit.z - pos.z;
    dx * dx + dz * dz < HIT_REACH * HIT_REACH
}

/// The first live `AT_TYPE_*` overlapping the trigger sphere, and where it is. The survey order is
/// the one `switem::source_at` uses (arrows, then blasts, then the hammer); the reference leaves the
/// winner to the collision manager's iteration order, so a tick carrying two dissolving sources
/// takes the earlier one here instead of whichever callback ran last.
///
/// `centre` is the sphere `l_sph_src` describes, which is what `dCcD_Sph`'s hit test intersects.
fn hit_source(
    centre: Vec3,
    arrows: &Query<&Arrow>,
    blasts: &[Vec3],
    head: Option<Vec3>,
    striking: bool,
) -> Option<(Hit, Vec3)> {
    for shaft in arrows.iter() {
        if shaft.kind != TYPE_FIRE || shaft.phase != Phase::Flying {
            continue;
        }
        // `mAtCps.SetStartEnd(current.pos, end)` and `mAtCps.SetR(5.0f)` (`d_a_arrow.cpp:293-295`)
        // swept over the pile's sphere.
        let span = SPH_RADIUS + arrow::CAPSULE_RADIUS;
        if walk::swept(shaft.pos - shaft.speed, shaft.pos, centre, span) {
            return Some((Hit::FireArrow, shaft.pos));
        }
    }
    // A bomb in state 2 keeps `radius = 200.0f` at `current.pos` (`d_a_bomb2.cpp:472`, `:497`),
    // which is the origin `bombflower::Blasts` publishes.
    let span = SPH_RADIUS + bombflower::BLAST_RADIUS;
    if let Some(origin) = blasts
        .iter()
        .find(|origin| walk::within(**origin, centre, span))
    {
        return Some((Hit::Bomb, *origin));
    }
    if striking && let Some(head) = head {
        // `hammer::AT_RADIUS` is the skull hammer's own sphere, at `hammer::strike_origin`.
        let span = SPH_RADIUS + hammer::AT_RADIUS;
        if walk::within(head, centre, span) {
            return Some((Hit::Hammer, head));
        }
    }
    None
}

pub fn is_leaves(name: &str) -> bool {
    LEAF_ACTORS.contains(&name)
}

/// `_execute` (`:704-718`) for every `Vochi`, plus the collider pass whose callback the reference
/// raises outside `_execute`.
pub fn control(
    players: Query<&Player>,
    hammers: Query<&Hammer>,
    cut: Res<Cut>,
    arrows: Query<&Arrow>,
    blasts: Res<bombflower::Blasts>,
    health: Res<Health>,
    mut piles: Query<(Entity, &Actor, &mut Leaves)>,
    mut switches: ResMut<Switches>,
    mut events: ResMut<SimEvents>,
    mut commands: Commands,
) {
    let player = players.iter().next().copied();
    let striking = cut.kind != hammer::CUT_NONE && hammers.iter().any(|hammer| hammer.striking());
    let head = player.map(|player| hammer::strike_origin(&player));
    for (entity, actor, mut pile) in &mut piles {
        // `_create` only loads the archive `if (m43C == true)` (`:474`), so a pile that failed
        // `chk_appear` has nothing to run and nothing to draw.
        if !pile.appear {
            let _ = commands.entity(entity).despawn();
            continue;
        }
        pile.advance();
        pile.execute(entity, actor, &health, &mut switches, &mut commands);
        // `mSph.ClrTgHit()` then `if (!m44C)` (`:571-572`): only a waiting, undissolved pile is
        // still in the collision space, so only one can be hit.
        if pile.step == Step::Wait
            && !pile.dissolve
            && let Some((hit, at)) = hit_source(
                pile.centre(actor.pos),
                &arrows,
                &blasts.list,
                head,
                striking,
            )
            && let Some(sound) = pile.struck(hit, at, actor.pos)
        {
            events.list.push(SimEvent::Se(sound));
        }
    }
}
