//! `daObjHcbh_c` (`tww/src/d/actor/d_a_obj_hcbh.cpp`): the tall stone pillar that stands until
//! something breaks it, and then falls over onto Link.
//!
//! One tick, per `_execute` (`:436-459`): `mHeight += mFallSpeed`, `mAngle += mAngleSpeed` and the
//! `mAngle >= 0x4000` clamp, then `(this->*mAction)()`. `mAction` is `wait_act_proc` (`:344-364`)
//! and then `fall_act_proc` (`:367-433`), which the `Step` enum mirrors in that order.
//!
//! The break lands in the same tick as the hit, which is why `control` surveys the hit sources
//! between `Hcbh::advance` and the proc: `wait_act_proc` reads the accumulated Tg hit itself
//! (`checkCollision`, `:349`) and tests `mBreakType` on the very next line (`:350`), so a pillar
//! whose cylinder was hit earlier in the frame tips at once. The CO channel the Armos uses
//! (`co_hitCallback`, `:207-211`) is raised by the collision pass, which is likewise before the
//! proc, and `checkCollision` runs last of the two, so the survey tries the AT arms first. The
//! hand-over to `fall_act_proc` (`:362`) is a stored pointer swap, so the tip itself starts on the
//! next tick.
//!
//! Not modelled, and why:
//!
//! - the `AT_TYPE_UNK8` arm, which sets `mBreakType = 1` only for `fpcNm_TN_e` (`:189-195`), and the
//!   `AT_TYPE_DARKNUT_SWORD` arm (`:196-199`). This crate has neither process, so neither arm has a
//!   source to fire on. Both are inside the pillar's `SrcObjTg Type` mask (`:17`), which the
//!   `bodies` table carries, so a future emitter of either type is one survey away.
//! - the four `mPiece` sets (`d_a_obj_hcbh.h:60-65`), which are the rubble chunks. Their whole sim
//!   state is their own fall (`d_a_obj_hcbh.cpp:408-432`), which nothing else reads, and the ground
//!   they give the player is `mpPieceBgW` (`d_a_obj_hcbh.cpp:131-156`), a `dBgW` mesh this crate
//!   does not build.
//! - both `dBgW`s and their release once `mAngle > 0x2000` (`:372-379`), and `mAcch` / `mAcchCir`
//!   (`:265-271`), whose `SetWall(589.0f, 70.0f)` ground and roof follow is what would otherwise
//!   move `current.pos`. The pillar never yields a collision correction for the reason given under
//!   `home_y`, so `current.pos` and `home.pos` stay where they were spawned.
//! - `particle_set` (`:214-223`), the model and the heap `fopAcM_entrySolidHeap` reserves (`:257`),
//!   and the four pieces and both `dBgW`s `create_heap` (`:131-156`) builds; the rendering is
//!   `_draw` (`:462-471`) and the teardown `_delete` (`:306-341`).
//! - the demo build's tuning: `daObjHcbh_HIO_c` (`:31-83`), whose `mParticle` gate would hide the
//!   emitters and whose `mCylR` / `mCylH` would resize the cylinder (`:345-347`); this port follows
//!   the USA-and-later path throughout.
//! - the six spheres' own `SrcObjCo SPrm` (`:21`), whose missing `cCcD_CoSPrm_VsPlayer_e` bit keeps
//!   a falling pillar from shoving Link sideways. Only their `AT_TYPE_UNK800` side is ported,
//!   because that is the side that hurts him.
//! - the pillar's cylinder after the break. `_create` builds it once, but `wait_act_proc` registers
//!   it only while `mBreakType` is still zero (`:350-351`) and clears the CO callback on the way out
//!   (`:361`); `super::bodies` has no way to withdraw a row, so the pillar keeps its solid shape for
//!   the roughly forty-two ticks of the fall.
//! - the item-table band. `fopAcM_createItemFromTable` resolves `0x20..=0x3E` through
//!   `dComIfGp_getItemTable()` (`f_op_actor_mng.cpp:819`), whose tables come from per-stage data
//!   this crate does not carry, so a pillar configured with a table number drops nothing - the same
//!   refusal `switem::created_item` and `leaves` make.

use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::amos_knight::AmosKnight;
use super::bombflower;
use super::events::{SimEvent, SimEvents};
use super::flags::Switches;
use super::hammer::{self, Cut, Hammer};
use super::health::Health;
use super::pickup::{ITEM_ACTORS, Pickup};
use super::player::Player;
use super::sie_flag;
use super::stream;
use super::switem;
use super::vibration::Vibration;
use super::walk;

/// `OBJNAME("Hcbh", fpcNm_Obj_Hcbh_e, 0, 0)` (`d_stage.cpp:1168`), the same string as `l_arcname`
/// (`:15`).
pub const HCBH_ACTORS: [&str; 1] = ["Hcbh"];

/// `chk_appear` tests the switch with `if (sw != 0xFF)` (`:89`), and the same guard sits over the
/// latch, before `fopAcM_onSwitch` (`:233-236`).
pub const NO_SWITCH: u8 = 0xFF;

/// `l_cyl_src`'s `/* Radius */ 70.0f`, which `mCyl.Set(l_cyl_src)` installs at `:273`.
pub const CYL_RADIUS: f32 = 70.0; // <- d_a_obj_hcbh.cpp:18

/// The same source's `/* Height */ 556.0f`. The demo build's HIO would raise 589.0f for the cull box
/// and its own cylinder (`:64`, `:347`).
pub const CYL_HEIGHT: f32 = 556.0; // <- d_a_obj_hcbh.cpp:18

/// `l_sph_src`'s `/* Radius */ 35.0f` (`:22`), which `mSph[i].SetR(70.0f)` overwrites for all six
/// spheres at `:280`.
pub const SPH_RADIUS: f32 = 70.0; // <- d_a_obj_hcbh.cpp:280

/// `l_co_sph_offset`, the six offsets `mSph[i].SetC` starts from (`:284`) and that the fall loop
/// hands to `multVec` (`:402`).
pub const SPHERE_OFFSETS: [Vec3; 6] = [
    Vec3::new(0.0, 157.0, 0.0), // <- d_a_obj_hcbh.cpp:25
    Vec3::new(0.0, 227.0, 0.0), // <- d_a_obj_hcbh.cpp:25
    Vec3::new(0.0, 297.0, 0.0), // <- d_a_obj_hcbh.cpp:26
    Vec3::new(0.0, 367.0, 0.0), // <- d_a_obj_hcbh.cpp:26
    Vec3::new(0.0, 437.0, 0.0), // <- d_a_obj_hcbh.cpp:27
    Vec3::new(0.0, 507.0, 0.0), // <- d_a_obj_hcbh.cpp:27
];

/// `f32 offset = 50.0f * cM_ssin(mAngle)`, the sway the two `transM` calls cancel around the tip
/// rotation.
pub const SWAY_SCALE: f32 = 50.0; // <- d_a_obj_hcbh.cpp:393

/// `(s16)(4.0f + 127.0f * cM_ssin(mAngle))`, the `mAngleSpeed` increment.
pub const ANGLE_BASE: f32 = 4.0; // <- d_a_obj_hcbh.cpp:383
pub const ANGLE_SCALE: f32 = 127.0; // <- d_a_obj_hcbh.cpp:383

/// `if (mAngle >= 0x4000) { mAngle = 0x4000; }`, the quarter turn the pillar stops at.
pub const MAX_ANGLE: i16 = 0x4000; // <- d_a_obj_hcbh.cpp:453-455

/// `mAngle = -0x8000`, the other half of `fall_act_proc`'s opening branch (`:368-370`).
pub const HALF_TURN: i16 = -0x8000; // <- d_a_obj_hcbh.cpp:369

/// `mFallSpeed += -0.011f`, the fall's acceleration.
pub const FALL_ACCEL: f32 = -0.011; // <- d_a_obj_hcbh.cpp:392

/// `if (mHeight < -10.0f + home.pos.y)`, the ground the pillar is smashed on.
pub const GROUND_OFFSET: f32 = -10.0; // <- d_a_obj_hcbh.cpp:386

/// `fopAcM_setCullSizeBox(this, -40.0f, 0.0f, -40.0f, 100.0f, 589.0f, 100.0f)` (`:291`), which the
/// `fopAc_CULLBOX_CUSTOM_e` profile (`:520`) selects.
pub const CULL_MIN: [f32; 3] = [-40.0, 0.0, -40.0]; // <- d_a_obj_hcbh.cpp:291
pub const CULL_MAX: [f32; 3] = [100.0, 589.0, 100.0]; // <- d_a_obj_hcbh.cpp:291

/// `dItemNo_NONE_e`, which `fopAcM_createItemFromTable` refuses outright.
pub const ITEM_NONE: u8 = 0x3F; // <- f_op_actor_mng.cpp:894

/// `if (i_itemNo >= 0x20 && i_itemNo <= 0x3E)`, the branch that needs the absent item table.
pub const TABLE_LOW: u8 = 0x20; // <- f_op_actor_mng.cpp:819

/// The upper end of the same band.
pub const TABLE_HIGH: u8 = 0x3E; // <- f_op_actor_mng.cpp:819

/// `pos.y -= 30.0f;` before `fopAcM_createItemFromTable`.
pub const ITEM_DROP: f32 = 30.0; // <- d_a_obj_hcbh.cpp:231

/// `int switchNo = 0xFF; prmAngle.z = switchNo;` - `fopAcM_fastCreateItem2` overwrites the third
/// angle of every item it makes, whatever the `csXyz angle(0, 0, 0)` at `:230` passed.
const ITEM_SWITCH_ANGLE: i16 = 0xFF; // <- f_op_actor_mng.cpp:1019-1020

/// `mBreakType == 0`, the one value `wait_act_proc` tests for (`:350`). The others differ only in
/// `particle_set`'s `mBreakType != 3` gate (`:218`), which is unmodelled.
const BREAK_NONE: i32 = 0; // <- d_a_obj_hcbh.cpp:350
const BREAK_HAMMER: i32 = 2; // <- d_a_obj_hcbh.cpp:186
const BREAK_BOMB: i32 = 3; // <- d_a_obj_hcbh.cpp:182
const BREAK_ARMOS: i32 = 4; // <- d_a_obj_hcbh.cpp:209

/// The player's own Tg cylinder: `Radius` 30.0f and `Height` 125.0f (`d_a_player_main_data.inc:68-69`),
/// which the player rebuilds each frame with `mCyl.SetC(current.pos)` (`d_a_player_main.cpp:12628`).
/// `d_a_player_main.cpp:9740-9797` sizes it per animation state; the static row is all this survey
/// needs.
const PLAYER_RADIUS: f32 = 30.0; // <- d_a_player_main_data.inc:68
const PLAYER_HEIGHT: f32 = 125.0; // <- d_a_player_main_data.inc:69

/// `mBodyCyl.SetH(300.0f)` / `mBodyCyl.SetR(80.0f)` at `current.pos` (`d_a_am.cpp:1070-1072`), the
/// Armos body whose CO pair with the pillar is what `co_hitCallback` answers.
const AMOS_RADIUS: f32 = 80.0; // <- d_a_am.cpp:1072
const AMOS_HEIGHT: f32 = 300.0; // <- d_a_am.cpp:1071

/// `SrcObjAt Atp` of `l_sph_src` (`:21`), which `cCcS::CalcTgPlusDmg` hands to
/// `cCcD_Stts::PlusDmg` (`d_cc_s.cpp:126-132`) and the player spends as its summed
/// `mStts.GetDmg()` (`d_a_player_main.cpp:5132`).
const CONTACT_DAMAGE: u8 = 1; // <- d_a_obj_hcbh.cpp:21

pub const SE_BREAK: u32 = 0x6A10; // <- JAZelAudio_SE.h:2039 (JA_SE_OBJ_TN_COLUMN_BREAK)
pub const SE_SMASH: u32 = 0x6A11; // <- JAZelAudio_SE.h:2040 (JA_SE_OBJ_TN_COLUMN_SMASH)

/// `dComIfGp_getVibration().StartShock(8, -17, cXyz(0.0f, 1.0f, 0.0f))` with the pillar.
const SHOCK_PATTERN: i32 = 8; // <- d_a_obj_hcbh.cpp:388
const SHOCK_FLAGS: i32 = -17; // <- d_a_obj_hcbh.cpp:388
const SHOCK_COORD: Vec3 = Vec3::new(0.0, 1.0, 0.0); // <- d_a_obj_hcbh.cpp:388

const ITEM_SHIFT: u32 = 0;
const ITEM_MASK: u32 = 0x3F;
const BIT_SHIFT: u32 = 6;
const BIT_MASK: u32 = 0x7F;
const SAVE_SHIFT: u32 = 13;
const SAVE_MASK: u32 = 0xFF;

/// `mAction` (`d_a_obj_hcbh.h:70`), the two procs at `:344` and `:367`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// `wait_act_proc`, still registering the cylinder for `checkCollision`.
    Wait,
    /// `fall_act_proc`, tipping over and sweeping the six spheres.
    Fall,
}

/// Which channel broke the pillar, out of the two this crate can raise.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Hit {
    /// `case AT_TYPE_BOMB` (`:181-184`), a blast sphere overlapping the cylinder.
    Bomb,
    /// `case AT_TYPE_SKULL_HAMMER` (`:185-188`), the hammer's AT sphere overlapping the cylinder.
    Hammer,
    /// `co_hitCallback`'s `fpcNm_AM_e` arm (`:208-209`), an Armos body cylinder overlapping the
    /// pillar's.
    Armos,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Hcbh {
    /// `mAppear` (`d_a_obj_hcbh.h:69`): `chk_appear` (`d_a_obj_hcbh.cpp:86-93`) works it out and the
    /// first-creating block stores it, next to `mHeight` (`d_a_obj_hcbh.cpp:244`).
    pub appear: bool,
    /// `param_get_swSave` (`d_a_obj_hcbh.h:20`), the switch `make_item` latches (`:233-236`).
    pub switch_no: u8,
    /// `param_get_itemNo` (`d_a_obj_hcbh.h:18`), the item number `make_item` hands to
    /// `fopAcM_createItemFromTable` (`:232`).
    pub item: u8,
    /// `param_get_itemSaveBitNo` (`d_a_obj_hcbh.h:19`), its story bit.
    pub item_bit: u8,
    /// `home.pos.y`, which the smash gate reads (`:386`). `mStts.Init(255, 255, this)` (`:271`) puts
    /// this process at the top of `dCcS::GetRank`'s ladder (`d_cc_s.cpp:207-209`) and that row of
    /// `rank_tbl` is all zeroes (`d_cc_s.cpp:202-203`), so it never takes a position correction and
    /// `home.pos` stays equal to `current.pos` - and to the spawn position.
    home_y: f32,
    /// `mHeight` (`d_a_obj_hcbh.h:56`): `mHeight = current.pos.y` at `:244`, then `mHeight +=
    /// mFallSpeed` at `d_a_obj_hcbh.cpp:446`.
    height: f32,
    /// `mFallSpeed`, which `_create` never writes (`:240-303`) and so starts from the zero the fresh
    /// process block is cleared to (`f_pc_base.cpp:98`, `s_basic.cpp:19-21`).
    fall_speed: f32,
    /// `mAngle = 0` at `:245`.
    angle: i16,
    /// `mAngleSpeed`, likewise zero-initialised and only ever accumulated at `:383`.
    angle_speed: i16,
    /// `mBreakAngle`, which `setup_break_condition` writes at `:161` and the `YrotM` pair of the fall
    /// stack reads (`:395`, `:399`). An Armos break never reaches that call, so it stays zero and the
    /// pillar tips along its own facing.
    break_angle: i16,
    /// `mBreakType`.
    break_type: i32,
    /// `mAction`.
    step: Step,
    /// Whether this fall has already damaged Link. The reference clears and re-raises the hit flags
    /// every frame and gates the damage on `mDamageWaitTimer` (`d_a_player_main.cpp:5153`), which
    /// this crate does not carry; and each of the six spheres is
    /// `dCcG_AtSPrm_NoConHit_e` (`:21`, `d_cc_d.h:45`), which `dCcS::ChkAtTgHitAfterCross`
    /// (`d_cc_s.cpp:147`) makes a once-per-contact hit anyway.
    struck: bool,
}

impl Hcbh {
    /// `_create` (`:240-303`) from `fopAcM_IsFirstCreating` through
    /// `mAction = &daObjHcbh_c::wait_act_proc` (`:290`), minus the resource load, the solid heap, the
    /// two `dBgW` registrations, `mAcch` and the collider construction.
    ///
    /// The three parameter fields are `daObj::PrmAbstract`, which is
    /// `((1 << width) - 1) & (param >> shift)` (`d_a_obj.h:12-15`) over the `Param_e` widths and
    /// shifts at `d_a_obj_hcbh.h:14-16`.
    pub fn spawn(parameters: u32, switches: &Switches, pos: Vec3) -> Self {
        let switch_no = ((parameters >> SAVE_SHIFT) & SAVE_MASK) as u8;
        Self {
            // `if (fopAcM_IsFirstCreating(this)) { mAppear = chk_appear(); }` (`:243-252`), and a
            // spawn is always the first creating of that actor.
            appear: switch_no == NO_SWITCH || !switches.test(switch_no),
            switch_no,
            item: ((parameters >> ITEM_SHIFT) & ITEM_MASK) as u8,
            item_bit: ((parameters >> BIT_SHIFT) & BIT_MASK) as u8,
            home_y: pos.y,
            height: pos.y, // `mHeight = current.pos.y` (`:244`)
            fall_speed: 0.0,
            angle: 0, // `mAngle = 0` (`:245`)
            angle_speed: 0,
            break_angle: 0,
            break_type: BREAK_NONE,
            step: Step::Wait, // `mAction = &daObjHcbh_c::wait_act_proc` (`:290`)
            struck: false,
        }
    }

    /// `fopAcM_setCullSizeBox` (`:291`).
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `_create` leaves `phase` at `cPhs_ERROR_e` unless `mAppear == 1` started the archive load
    /// (`:241`, `:253-255`), so a pillar whose switch is already set never becomes an actor at all.
    pub fn drawn(&self) -> bool {
        self.appear
    }

    /// `setup_break_condition`'s one line this sim carries (`d_a_obj_hcbh.cpp:161`) is the s16
    /// `cM_atan2s`, the `atan2` of `c_math.cpp:118-141` turned to s16 turns: the pillar's centre from
    /// `mCyl.SetC(current.pos)` (`d_a_obj_hcbh.cpp:275`) minus the hitter's position. The four
    /// `mPiece` writes of the same function (`:162-170`) are the rubble.
    fn aim(pillar: Vec3, at: Vec3) -> i16 {
        walk::to_s16((pillar.x - at.x).atan2(pillar.z - at.z))
    }

    /// `checkCollision` (`:176-204`) or `co_hitCallback` (`:207-211`) for one source. The two differ
    /// in that only the AT arms reach `setup_break_condition`, so a CO break keeps the `mBreakAngle`
    /// the pillar started with.
    fn broke(&mut self, hit: Hit, pillar: Vec3, at: Vec3) {
        match hit {
            Hit::Bomb => {
                self.break_type = BREAK_BOMB;
                self.break_angle = Hcbh::aim(pillar, at);
            }
            Hit::Hammer => {
                self.break_type = BREAK_HAMMER;
                self.break_angle = Hcbh::aim(pillar, at);
            }
            Hit::Armos => self.break_type = BREAK_ARMOS,
        }
    }

    /// The `mHeight += mFallSpeed` / `mAngle += mAngleSpeed` pair and the clamp at the head of
    /// `_execute` (`:446-455`). The sum is a `s16` add and so wraps the way the reference's does.
    fn advance(&mut self) {
        self.height += self.fall_speed;
        self.angle = self.angle.wrapping_add(self.angle_speed);
        if self.angle >= MAX_ANGLE {
            self.angle = MAX_ANGLE;
        }
    }

    /// `wait_act_proc` (`:344-364`). `dComIfG_Ccsp()->Set(&mCyl)` (`:351`) is the survey `control`
    /// runs in its place and `particle_set` (`:358`) is particles, so what survives is the break
    /// sound, which `mDoAud_seStart` starts at `current.pos` with the room's reverb (`:360`), and the
    /// hand-over to `fall_act_proc` (`:362`). `mCyl.SetCoHitCallback(NULL)` (`:361`) is covered by
    /// leaving `Step::Wait`.
    fn wait_proc(&mut self, pos: Vec3, events: &mut SimEvents) {
        if self.break_type == BREAK_NONE {
            return;
        }
        events.list.push(SimEvent::SeAt(SE_BREAK, pos));
        self.step = Step::Fall;
    }

    /// `fall_act_proc` (`:367-433`) as far as this sim carries it, returning whether the pillar
    /// deleted itself. The `mAngle > 0x2000` release of both `dBgW`s (`:372-379`) and the four-piece
    /// loop (`:408-432`) are the omitted parts, and the `mDoMtx_stack_c` block with the six
    /// `dComIfG_Ccsp()->Set` calls (`:393-406`) is the survey `control` runs in its place off
    /// `Hcbh::spheres`.
    fn fall_proc(&mut self) -> bool {
        if self.angle < 0 {
            self.angle = HALF_TURN;
            self.angle_speed = 0;
        } else {
            self.angle_speed = self.angle_speed.wrapping_add(angle_step(self.angle));
        }
        if self.height < GROUND_OFFSET + self.home_y {
            return true;
        }
        self.fall_speed += FALL_ACCEL;
        false
    }

    /// The six `mSph[i]` centres, for the tick whose `mAngle` and `mHeight` `advance` and
    /// `fall_proc` have just settled (`:393`, `:401-404`).
    fn spheres(&self, actor: &Actor) -> [Vec3; 6] {
        let sway = SWAY_SCALE * stream::sin_s(self.angle);
        let mut out = [Vec3::ZERO; 6];
        for (index, centre) in out.iter_mut().enumerate() {
            *centre = self.tip(SPHERE_OFFSETS[index], sway, actor);
        }
        out
    }

    /// The `mDoMtx_stack_c` chain of `:394-400` applied to one `l_co_sph_offset` and read back by
    /// `multVec` (`:403`). Every stack call but `transS` right-multiplies the running matrix
    /// (`mDoMtx_concat` is `MTXConcat`, `m_Do_mtx.h:132-134`, over the stack's own `now` at `:350`),
    /// so the *last* call is the first applied to the point; and `mDoMtx_XYZrotM` appends Z, then Y,
    /// then X (`m_Do_mtx.cpp:29-45`), so a point turns about X first.
    fn tip(&self, offset: Vec3, sway: f32, actor: &Actor) -> Vec3 {
        let mut point = xyz_rot(offset, actor.shape_angle); // `:400`
        point = sie_flag::rot_y(point, self.break_angle.wrapping_neg()); // `:399`
        point.y -= sway; // `:398`
        point = sie_flag::rot_x(point, self.angle); // `:397`
        point.y += sway; // `:396`
        point = sie_flag::rot_y(point, self.break_angle); // `:395`
        point + Vec3::new(actor.pos.x, self.height, actor.pos.z) // `:394`
    }

    /// `make_item` (`:226-237`): the create `fopAcM_createItemFromTable(&pos, item, saveBit,
    /// fopAcM_GetHomeRoomNo(this), 0, &angle, 1, NULL)` (`:232`) stands in as `switem`'s existing
    /// translation of that call, and the `fopAcM_onSwitch` of `:233-236` is the latch.
    ///
    /// `0x3F` is the reference's own refusal (`f_op_actor_mng.cpp:894`), and `0xFF` cannot be encoded
    /// in a six-bit field; the table band is refused for the reason given at the top of this module.
    /// The `dItemNo_TRIPLE_HEART_e` and `dItemNo_RECOVER_FAIRY_e` special cases
    /// (`f_op_actor_mng.cpp:1027-1036`) are not modelled, matching `switem`.
    ///
    /// The angle is the caller's own `csXyz angle(0, 0, 0)` (`:230`) with its third component
    /// overwritten by `fopAcM_fastCreateItem2`, and the randomised `current.angle.y` that action 1
    /// asks for (`daItem_c::initAction`, `d_a_item.cpp:1394`) is not reproducible in a deterministic
    /// sim.
    fn release(
        &self,
        actor: &Actor,
        health: &Health,
        switches: &mut Switches,
        commands: &mut Commands,
    ) {
        if self.switch_no != NO_SWITCH {
            switches.set(self.switch_no);
        }
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

/// `mDoMtx_XYZrotM(shape_angle)` (`:400`, and the same call in `set_mtx` at `:104`): the three
/// `mDoMtx_concat(mtx, tmp, mtx)` calls of `m_Do_mtx.cpp:29-45` append Z, then Y, then X.
fn xyz_rot(value: Vec3, angle: [i16; 3]) -> Vec3 {
    sie_flag::rot_z(
        sie_flag::rot_y(sie_flag::rot_x(value, angle[0]), angle[1]),
        angle[2],
    )
}

/// `(s16)(4.0f + 127.0f * cM_ssin(mAngle))` (`:383`), with the same truncation to `s16` as the cast.
fn angle_step(angle: i16) -> i16 {
    (ANGLE_BASE + ANGLE_SCALE * stream::sin_s(angle)) as i16
}

/// `cM3d_Cross_CylSph` (`c_m3d.cpp:1389-1398`): the horizontal span is `cyl.r + sph.r`, then the
/// sphere must straddle the cylinder's `y .. y + h` interval. The base `y` is what the pillar's
/// `mCyl.SetC(current.pos)` (`d_a_obj_hcbh.cpp:275`) and the Armos' `mBodyCyl` (`d_a_am.cpp:1070`)
/// both set to their own position.
fn cyl_sph_cross(base: Vec3, radius: f32, height: f32, at: Vec3, reach: f32) -> bool {
    let dx = at.x - base.x;
    let dz = at.z - base.z;
    let span = radius + reach;
    if span * span < dx * dx + dz * dz {
        return false;
    }
    at.y + reach >= base.y && at.y - reach <= base.y + height
}

/// `cM3d_Cross_CylCyl` (`c_m3d.cpp:1568-1583`), which is the test behind the CO pair
/// `co_hitCallback` is called for.
fn cyl_cyl_cross(base: Vec3, radius: f32, height: f32, at: Vec3, other: f32, other_h: f32) -> bool {
    let dx = base.x - at.x;
    let dz = base.z - at.z;
    let span = radius + other;
    if dx * dx + dz * dz > span * span {
        return false;
    }
    base.y + height >= at.y && base.y <= at.y + other_h
}

/// The first AT source overlapping the pillar's cylinder. `cc_set` leaves a blast in state 2 at a
/// `radius` of 200.0f (`d_a_bomb2.cpp:468-472`), and `bombflower::Blasts` publishes exactly those
/// origins at `mSph.SetC(pos)` (`:497`); the hammer is the `AT_TYPE_SKULL_HAMMER` sphere of
/// `hammer::AT_RADIUS` at `hammer::strike_origin`.
///
/// Both types are in the pillar's `l_cyl_src` Tg mask (`d_a_obj_hcbh.cpp:17`), which is why they are
/// the only two arms of `checkCollision` (`:180-200`) with a source here.
fn at_source(
    pos: Vec3,
    blasts: &[Vec3],
    head: Option<Vec3>,
    striking: bool,
) -> Option<(Hit, Vec3)> {
    if let Some(origin) = blasts.iter().find(|origin| {
        cyl_sph_cross(
            pos,
            CYL_RADIUS,
            CYL_HEIGHT,
            **origin,
            bombflower::BLAST_RADIUS,
        )
    }) {
        return Some((Hit::Bomb, *origin));
    }
    if striking
        && let Some(head) = head
        && cyl_sph_cross(pos, CYL_RADIUS, CYL_HEIGHT, head, hammer::AT_RADIUS)
    {
        return Some((Hit::Hammer, head));
    }
    None
}

pub fn is_hcbh(name: &str) -> bool {
    HCBH_ACTORS.contains(&name)
}

/// `_execute` (`:436-459`) for every `Hcbh`, plus the collision pass whose hit flags and callbacks
/// the reference raises outside `_execute`.
pub fn control(
    players: Query<&Player>,
    hammers: Query<&Hammer>,
    cut: Res<Cut>,
    blasts: Res<bombflower::Blasts>,
    mut health: ResMut<Health>,
    knights: Query<&Actor, (With<AmosKnight>, Without<Hcbh>)>,
    mut pillars: Query<(Entity, &Actor, &mut Hcbh)>,
    mut switches: ResMut<Switches>,
    mut events: ResMut<SimEvents>,
    mut vibration: ResMut<Vibration>,
    mut commands: Commands,
) {
    let player = players.iter().next().copied();
    let striking = cut.kind != hammer::CUT_NONE && hammers.iter().any(|hammer| hammer.striking());
    let head = player.map(|player| hammer::strike_origin(&player));
    for (entity, actor, mut pillar) in &mut pillars {
        // `cPhs_ERROR_e` unless the archive load ran (`:241`, `:253-255`).
        if !pillar.appear {
            let _ = commands.entity(entity).despawn();
            continue;
        }
        pillar.advance();
        match pillar.step {
            Step::Wait => {
                // The survey `dComIfG_Ccsp()->Set(&mCyl)` (`:351`) stands in for, tried in the order
                // the two channels are read: `checkCollision` (`:349`) after `co_hitCallback`.
                if let Some((hit, at)) = at_source(actor.pos, &blasts.list, head, striking) {
                    pillar.broke(hit, actor.pos, at);
                } else if let Some(knight) = knights.iter().find(|knight| {
                    cyl_cyl_cross(
                        actor.pos,
                        CYL_RADIUS,
                        CYL_HEIGHT,
                        knight.pos,
                        AMOS_RADIUS,
                        AMOS_HEIGHT,
                    )
                }) {
                    pillar.broke(Hit::Armos, actor.pos, knight.pos);
                }
                pillar.wait_proc(actor.pos, &mut events);
            }
            Step::Fall => {
                if pillar.fall_proc() {
                    pillar.release(actor, &health, &mut switches, &mut commands);
                    vibration.start_shock(SHOCK_PATTERN, SHOCK_FLAGS, SHOCK_COORD);
                    events.list.push(SimEvent::SeAt(SE_SMASH, actor.pos));
                    let _ = commands.entity(entity).despawn(); // `fopAcM_delete(this)` (`:390`)
                    continue;
                }
                // The six spheres are registered only on this path (`:405`), and they are the only
                // hazard a falling pillar raises at Link.
                if !pillar.struck {
                    let hits = overlap_count(&pillar.spheres(actor), player);
                    if hits > 0 {
                        pillar.struck = true;
                        health.damage(hits * CONTACT_DAMAGE);
                    }
                }
            }
        }
    }
}

/// How many of the six fall spheres overlap the player's own Tg cylinder, which is the count
/// `dCcS::CalcTgPlusDmg` (`d_cc_s.cpp:126-132`) would have summed `AtAtp` over.
fn overlap_count(spheres: &[Vec3; 6], player: Option<Player>) -> u8 {
    let Some(player) = player else {
        return 0;
    };
    spheres
        .iter()
        .filter(|centre| {
            cyl_sph_cross(
                player.pos,
                PLAYER_RADIUS,
                PLAYER_HEIGHT,
                **centre,
                SPH_RADIUS,
            )
        })
        .count() as u8
}
