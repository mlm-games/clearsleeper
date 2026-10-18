//! `daStone2::Act_c` (`d_a_stone2.cpp`, `fpcNm_Stone2_e`) - the three carryable breakable rocks.
//!
//! Damage classification:
//! (a) INBOUND damage: YES, and it is not an HP decrement. `damage_cc_proc` (`:277-296`) calls
//!     `damaged(3)` on `mCyl.ChkAtHit()` and `damaged(7)` on a `ChkTgHit()` whose hit object is
//!     `AT_TYPE_BOMB` and whose `m644 != 3`; `damage_bg_proc_directly` (`:303-327`) calls
//!     `damaged(2)` on `mAcch.ChkGroundHit()` while the rock is in `mode_drop`. `damaged` goes
//!     straight to `mode_fine_init` (`:705`, `:619`) and the 812-line file has no health field and no
//!     flinch write of any kind, so `combat::HitPoints` must NOT be attached: `combat::control`
//!     would subtract a damage number and despawn the entity, losing the item drop, the switch and
//!     the break sound that `damaged` (`:250-274`) performs. `enemy::Hurt` is not attached either.
//! (b) OUTBOUND damage: the rock does register an AT. `M_cyl_src` (`:17-45`) carries
//!     `SrcObjAt Type = AT_TYPE_UNK8` at `Atp = 2` with `Set|VsEnemy|VsOther`, `dComIfG_Ccsp()->Set(&mCyl)`
//!     runs every tick `m648 != 3` (`:713-716`) and `mode_drop_init` turns the AT `SPrm` bit back on
//!     (`:540`). OMITTED: the only retail path for an actor AT striking an actor Tg is the `cCd`
//!     registry, which port_contract.md 3b lists as confirmed-absent, and the crate's AT resolution
//!     only ever runs a player attacker against a `HitPoints` target.
//!
//! COLLIDER REGISTRATION (checked, not just declared): `mCyl.Set(M_cyl_src)` and
//! `mCyl.SetStts(&mStts)` happen in `Create` (`:127-128`) and the object is handed to
//! `dComIfG_Ccsp()->Set` at `:716`, so the cylinder is live in retail while `m648 != 3`. Its `SPrm`
//! bits are per-mode: `mode_wait_init` (`:488-490`) and `mode_carry_init` (`:508-510`) keep only
//! `cCcD_TgSPrm_Set_e`, `mode_drop_init` (`:540-542`) enables AT+Tg+Co and `mode_fine_init`
//! (`:561-563`) disables all three.
//!
//! CULL: the profile is `fopAc_CULLSPHERE_CUSTOM_e` (`:811`), and the only volumes are the two
//! `fopAcM_setCullSizeSphere` calls in `cull_set_draw` (`:237`, `attr().m28/m2A/m2C/m2E`) and
//! `cull_set_move` (`:243`, `attr().m20/m22/m24/m26`). There is no `fopAcM_setCullSizeBox`,
//! `fopAcM_SetMin` or `fopAcM_SetMax` anywhere in the file, so no box exists and this module
//! declares no `cull()`. `attr().m26` is `0xffff` on the two "Ebrock" rows, which is why
//! `cull_set_move` sets `m657 = 1` (`:245`) and `Execute` (`:699`) runs those rocks un-culled.

use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::bombflower::{BLAST_RADIUS, Blasts};
use super::carry::Carry;
use super::collision::Collision;
use super::combat::Combat;
use super::events::{SimEvent, SimEvents};
use super::flags::Switches;
use super::ground;
use super::pickup::{ITEM_ACTORS, Pickup};
use super::player::Player;

/// `d_stage.cpp:654`, `:655` and `:656` bind these three strings to `fpcNm_Stone2_e`, with
/// `dStage_objectNameInf::argument` 0, 1 and 2.
pub const STONE_ACTORS: [&str; 3] = ["Ebrock", "Ekao", "Ebrock2"];

// -- parameter decode: every accessor is `daObj::PrmAbstract(this, W, S)` = `(param >> S) & ((1 << W) - 1)`
//    (`include/d/d_a_obj.h:12-16`), reached through the `Prm_e` shifts in `d_a_stone2.h:46-63`.
const ITEM_NO_SHIFT: u32 = 0; // <- d_a_stone2.h:47 (PRM_ITEMNO_S)
const ITEM_NO_WIDTH: u32 = 6; // <- d_a_stone2.h:46 (PRM_ITEMNO_W)
const SWITCH_SHIFT: u32 = 8; // <- d_a_stone2.h:51 (PRM_SWSAVE_S)
const SWITCH_WIDTH: u32 = 8; // <- d_a_stone2.h:50 (PRM_SWSAVE_W)
const ITEM_SAVE_SHIFT: u32 = 16; // <- d_a_stone2.h:54 (PRM_ITEMSAVE_S)
const ITEM_SAVE_WIDTH: u32 = 7; // <- d_a_stone2.h:53 (PRM_ITEMSAVE_W)
const KIND_SHIFT: u32 = 24; // <- d_a_stone2.h:57 (PRM_TYPE_S)
const KIND_WIDTH: u32 = 3; // <- d_a_stone2.h:56 (PRM_TYPE_W)

/// `prm_get_swSave()` returns `0xff` for "this rock owns no switch": both `is_switch` (`:100`) and
/// `on_switch` (`:110`) test that exact value, never `0`.
const NO_SWITCH: u8 = 0xFF; // <- d_a_stone2.cpp:100, :110

// -- `m648`, the `mode_proc` index read by `mode_proc_call` (`:594-601`).
const MODE_WAIT: u8 = 0; // <- d_a_stone2.cpp:594
const MODE_CARRY: u8 = 1; // <- d_a_stone2.cpp:595
const MODE_DROP: u8 = 2; // <- d_a_stone2.cpp:596
const MODE_FINE: u8 = 3; // <- d_a_stone2.cpp:597

/// `if (m644 != 3 && hitObj->ChkAtType(AT_TYPE_BOMB))` (`:286`) and the riddle arm
/// `m644 == 3 && !dComIfGp_evmng_existence(m650)` (`:585`) both single out `M_attr[3]`, "Ekao" -
/// the one rock a bomb cannot break.
const EKAO_KIND: u8 = 3; // <- d_a_stone2.cpp:286, :585

// -- `damaged(int arg1)`'s `arg1`. It is the `action` argument of `fopAcM_createItemFromTable`
//    (`:254`), i.e. a `daItemAct_e` value (`include/f_op/f_op_actor_mng.h:69-88`) - NOT a hit power.
const DA_ITEM_ACT_2: u8 = 2; // <- d_a_stone2.cpp:314 (`damaged(2)` on `mAcch.ChkGroundHit()`)
const DA_ITEM_ACT_3: u8 = 3; // <- d_a_stone2.cpp:281 (`damaged(3)` on `mCyl.ChkAtHit()`), :317 (wall/roof)
const DA_ITEM_ACT_7: u8 = 7; // <- d_a_stone2.cpp:287 (`damaged(7)` on an `AT_TYPE_BOMB` Tg hit)

/// `MAKE_ITEM_PARAMS(itemNo, i_itemBitNo, switchNo2, type, action)` packs
/// `itemNo << 0 | i_itemBitNo << 8 | switchNo2 << 16 | type << 24 | action << 26`
/// (`src/f_op/f_op_actor_mng.cpp:31-32`). `fopAcM_fastCreateItem2` calls it with `switchNo2 = -1`
/// (`:982`, `:983`) and forwards the `type` and `action` arguments it was handed - which for
/// `damaged` are the literal `0` and `arg1` (`:254`).
const ACTION_SHIFT: u32 = 26; // <- f_op_actor_mng.cpp:32
const ACTION_MASK: u32 = 0x3F; // <- f_op_actor_mng.cpp:32
const ITEM_SAVE_PACK_SHIFT: u32 = 8; // <- f_op_actor_mng.cpp:32 (the `(i_itemBitNo & 0xFF) << 8` term)
const ITEM_SAVE_PARAM_MASK: u32 = 0xFF; // <- f_op_actor_mng.cpp:32
const SWITCH_NO_SLOT: u32 = 0xFF; // <- f_op_actor_mng.cpp:982 (`int switchNo2 = -1;`)
const SWITCH_NO_PACK_SHIFT: u32 = 16; // <- f_op_actor_mng.cpp:32 (the `(switchNo2 & 0xFF) << 16` term)

/// `i_itemNo == 0x3f || i_itemNo == 0xff` creates nothing
/// (`src/f_op/f_op_actor_mng.cpp:892`). `prm_get_itemNo()` is six bits wide, so only `0x3f` is
/// reachable from a stage parameter.
const NO_ITEM: u8 = 0x3F; // <- f_op_actor_mng.cpp:892

/// `actor->maxFallSpeed = -100.0f` (`src/f_op/f_op_actor.cpp:286`), the clamp `posMoveF_grade`
/// applies to `speed_y` (`src/d/d_a_obj.cpp:166-168`).
const MAX_FALL: f32 = -100.0; // <- f_op_actor.cpp:286

/// `shape_angle.x += 1000` (`:556`).
const TUMBLE_STEP: i16 = 1000; // <- d_a_stone2.cpp:556

// The sword reach is the crate-wide `combat::control` window rather than a value from this TU: the
// player is the only AT attacker whose hits this crate resolves, and `combat::control` places its
// `Atk` at the sword tip for `DAMAGE_START..DAMAGE_END`. `bridge::control` reads the same four.
const CUT_START: u32 = 1; // <- src/sim/combat.rs (DAMAGE_START)
const CUT_END: u32 = 6; // <- src/sim/combat.rs (DAMAGE_END)
const SWORD_OFFSET: f32 = 20.0; // <- src/sim/combat.rs (SWORD_OFFSET)
const SWORD_REACH: f32 = 20.0; // <- src/sim/combat.rs (SWORD_RADIUS)

/// One row of `daStone2::Attr_c` (`d_a_stone2.h:11-43`), reduced to the fields the simulation reads.
/// `attr().m1C` and `attr().m4C` are absent: `grep 'attr()\.m1C\|attr()\.m4C' d_a_stone2.cpp` is empty.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Attr {
    /// `attr().m12` - `mCyl.SetR(attr().m12)` (`:130`) and `mAcchCir.SetWall(30.0f, attr().m12)` (`:133`).
    pub radius: f32,
    /// `attr().m30` - `fopAcM_SetGravity(this, attr().m30)` in `Create` (`:146`) and again in
    /// `mode_drop_init` (`:546`).
    pub gravity: f32,
    /// `attr().m3C` - `fopAcM_GetSpeed(this).y = attr().m3C` (`:544`), the drop launch speed.
    pub lift: f32,
    /// `attr().m40` - `f32 fVar1 = attr().m40` (`:539`) then `fopAcM_SetSpeedF(this, fVar1)` (`:545`).
    pub toss: f32,
    /// `attr().m18` - `fopAcM_seStart(this, attr().m18, 0)` (`:272`), the break sound.
    pub break_se: u32,
}

/// `M_attr[2]` (`d_a_stone2.cpp:56-57`) - the great rock. `attr().m12 = 0x118 = 280` is the cylinder
/// radius and `attr().m14 = 0x1C6 = 454` its height; the height is not read here because the
/// vertical half of the overlap test is omitted (see `sword_struck`). `attr().m44 = attr().m48 =
/// 0.0f` are `posMoveF_stream`'s two resist coefficients and `attr().m50 = 6` is the vibration power.
const EBROCK: Attr = Attr {
    radius: 280.0,
    gravity: -6.0,
    lift: 27.0,
    toss: 36.0,
    break_se: 0x696A,
};

/// `M_attr[3]` (`d_a_stone2.cpp:66-67`) - Ekao, the face rock.
const EKAO: Attr = Attr {
    radius: 275.0,
    gravity: -6.0,
    lift: 27.0,
    toss: 36.0,
    break_se: 0x6969,
};

/// `M_attr[4]` (`d_a_stone2.cpp:76-77`) - Ebrock2, the little rock.
const EBROCK2: Attr = Attr {
    radius: 110.0,
    gravity: -6.0,
    lift: 27.0,
    toss: 36.0,
    break_se: 0x696B,
};

/// `M_attr[m644]` (`d_a_stone2.h:70`) in table order, five entries as declared at
/// `d_a_stone2.h:128`. `M_attr[0]` and `M_attr[1]` (`:48-54`) are the all-zero filler rows: their
/// `resName` is `NULL`, so `dComIfG_getObjectRes(attr().resName, attr().m04)` (`:117`) has no
/// archive to open and `JUT_ASSERT(592, model_data != NULL)` fires. They are folded onto row 2,
/// which is the row the retail table itself walks into for any `m644` past the end.
const ATTRS: [Attr; 5] = [EBROCK, EBROCK, EBROCK, EKAO, EBROCK2];

/// A carryable breakable rock - `daStone2::Act_c` (`d_a_stone2.h:45-153`).
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stone {
    /// `m644 = prm_get_type()` (`:180`) - the `M_attr` row index.
    pub kind: u8,
    /// `prm_get_itemNo()` (`d_a_stone2.h:72`) - `i_itemNo` of `damaged` (`:251`).
    pub item_no: u8,
    /// `prm_get_itemSave()` (`d_a_stone2.h:73`) - `i_itemBitNo` of `damaged` (`:252`).
    pub item_save: u8,
    /// `prm_get_swSave()` (`d_a_stone2.h:75`) - the `fopAcM_isSwitch` (`:104`) / `fopAcM_onSwitch`
    /// (`:111`) index.
    pub switch_no: u8,
    /// `m648` - the `mode_proc[m648]` state (`:601`).
    pub mode: u8,
    /// `m659 = chk_appear()` (`:181`, `:172-174`), which is `is_switch() == false`. It starts
    /// `false` and `control` resolves it on the first tick: `Mthd_Create` needs the live `Switches`,
    /// which `spawn` does not take. When it resolves to `false`, `Mthd_Create` returned its
    /// `cPhs_ERROR_e` initialiser (`:182`) without ever reaching `MoveBGCreate`, and RULE 6b makes
    /// this a spawn-then-despawn-on-first-tick rock rather than a live one.
    pub appears: bool,
    /// `m648 == MODE_FINE` (`:579`). `Draw` submits nothing in that mode (`:750-758`), which is
    /// what `src/lib.rs`'s `actor_views` reads to stop drawing the rock.
    pub broken: bool,
}

fn field(parameters: u32, shift: u32, width: u32) -> u8 {
    ((parameters >> shift) & ((1 << width) - 1)) as u8
}

impl Stone {
    /// `Mthd_Create` (`:177-195`), `Create` (`:125-169`) and `prmZ_init` (`:86-95`), minus the
    /// archive load, the model, the moving background and the collider. What survives is the four
    /// `PrmAbstract` reads; `chk_appear` is left to the first tick because it reads `Switches`.
    pub fn spawn(parameters: u32) -> Self {
        Self {
            kind: field(parameters, KIND_SHIFT, KIND_WIDTH),
            item_no: field(parameters, ITEM_NO_SHIFT, ITEM_NO_WIDTH),
            item_save: field(parameters, ITEM_SAVE_SHIFT, ITEM_SAVE_WIDTH),
            switch_no: field(parameters, SWITCH_SHIFT, SWITCH_WIDTH),
            mode: MODE_WAIT,
            appears: false,
            broken: false,
        }
    }

    /// `chk_appear()` (`:172-174`) = `is_switch()` (`:98-105`): `prm_get_swSave() == 0xff` answers
    /// false outright, otherwise `fopAcM_isSwitch(this, switchIndex)` decides. `Mthd_Create` reads it
    /// once, before `Create`, which is what makes a rock whose switch is already latched a
    /// never-created actor (`:182-183`).
    pub fn chk_appear(&mut self, switches: &Switches) -> bool {
        self.appears = self.switch_no == NO_SWITCH || !switches.test(self.switch_no);
        self.appears
    }

    /// `attr()` (`d_a_stone2.h:70`) - `M_attr[m644]`.
    pub fn attr(&self) -> Attr {
        ATTRS[usize::from(self.kind) % ATTRS.len()]
    }

    /// `Execute` registers `mCyl` only while `m648 != 3` (`:713-716`), and `mode_wait_init` (`:488`),
    /// `mode_carry_init` (`:508`), `mode_drop_init` (`:540`) and `mode_fine_init` (`:561`) are the
    /// four `SPrm` states of that one collider. `m648 == 3` is the only dead state.
    fn hittable(&self) -> bool {
        self.mode != MODE_FINE
    }

    /// `damage_cc_proc` first arm (`:279-282`): `mCyl.ChkAtHit()` then `damaged(3)`. The registered
    /// collider is the `M_cyl_src` cylinder (`:17-45`) moved to `current.pos` by
    /// `mCyl.MoveCAtTg(current.pos)` (`:715`), so the test is horizontal against `attr().m12`.
    ///
    /// The vertical half of the `cM3dGCylS` overlap is not modelled. The convention IS readable -
    /// `cM3d_Cross_CylPnt` (`c_m3d.cpp:1877-1888`) keeps a point strictly inside
    /// `center.y < y < center.y + Height`, `cM3d_Cross_CylSph` (`:1398-1404`) tests the sphere's
    /// `y ± R` against the same span, and `cM3d_Cross_CpsCyl` (`:1962-2064`) treats the cylinder
    /// as the segment `center` to `center + (0, Height, 0)` with `Radius` around it - so `Height`
    /// runs upward from the centre. What is missing is the geometry pair on the actor side: this
    /// sim's AT/Tg reduction is `walk::within`, a point against a radius, and its `Collision`
    /// carries no cylinder or capsule overlap (`cCcD_Obj::ChkAtHit` over `cM3d_Cross_CpsCyl` is
    /// the `cCcD` registry RULE 3b lists as absent, the same gap `koisi.rs` and `fallrock.rs`
    /// record).
    pub fn sword_struck(&self, centre: Vec3, at: Vec3, reach: f32) -> bool {
        self.hittable() && super::walk::within(at, centre, self.attr().radius + reach)
    }

    /// `damage_cc_proc` second arm (`:283-291`): `mCyl.ChkTgHit()`, then
    /// `if (hitObj != NULL) { if (m644 != 3 && hitObj->ChkAtType(AT_TYPE_BOMB)) damaged(7); }`.
    /// `SrcObjTg Type = AT_TYPE_BOMB` with `Set|IsOther` (`:24-25`) is what makes the Tg arm a bomb
    /// arm, and `m644 != 3` is what makes Ekao immune.
    pub fn bomb_struck(&self, centre: Vec3, blasts: &[Vec3]) -> bool {
        self.hittable()
            && self.kind != EKAO_KIND
            && blasts.iter().any(|origin| {
                super::walk::within(*origin, centre, BLAST_RADIUS + self.attr().radius)
            })
    }

    /// `damage_bg_proc_directly` (`:303-327`) guards on `m648 != 0` and then `m648 == 2`, so only a
    /// dropped rock can shatter on impact, and only the `mAcch.ChkGroundHit()` arm (`:309`, `:314`)
    /// is modelled here - `ChkWallHit` / `ChkRoofHit` (`:310-311`) need a wall collider this crate
    /// does not carry.
    pub fn shatters_on_landing(&self, grounded: bool) -> bool {
        self.mode == MODE_DROP && grounded
    }

    /// `mode_wait` (`:495-504`): `if (fopAcM_checkCarryNow(this)) mode_carry_init();`. Its other
    /// arm, the two-tick `m65A` countdown into `mpBgW->SetLock()` (`:158`, `:499-502`), is a
    /// `dBgW`/`cBgW` call and port_contract.md 3b lists moving backgrounds as absent, so it is
    /// omitted rather than faked.
    fn begin_carry(&mut self, switches: &mut Switches) {
        // `mode_carry_init` (`:507-515`) clears the carry attention bit (`:511`), releases the
        // moving background (`:512`) and calls `on_switch()` (`:513`).
        self.mode = MODE_CARRY;
        self.latch(switches);
    }

    /// `mode_drop_init` (`:538-549`) then `mode_drop` (`:552-557`), chained by `mode_carry`
    /// (`:531-534`) so both run on the release tick.
    fn begin_drop(&mut self, actor: &mut Actor, attr: Attr) {
        actor.speed.y = attr.lift; // fopAcM_GetSpeed(this).y = attr().m3C (:544)
        actor.speed_f = attr.toss; // fopAcM_SetSpeedF(this, fVar1) (:545)
        actor.gravity = attr.gravity; // fopAcM_SetGravity(this, attr().m30) (:546)
        self.mode = MODE_DROP; // m648 = 2 (:548)
        self.fall(actor);
    }

    /// `mode_drop` (`:552-557`).
    ///
    /// `daObj::posMoveF_stream(this, mStts.GetCCMoveP(), zero, attr().m44, attr().m48)` (`:555`)
    /// passes `attr().m44 == attr().m48 == 0.0f` on all three rows (`:57`, `:67`, `:77`), and
    /// `posMoveF_stream` forwards them as `param_4` / `param_5` (`src/d/d_a_obj.cpp:94-95`), which
    /// zeroes `posMoveF_resist_acc` (`:100-110`); `pNorm` is NULL there too (`:141`), which zeroes
    /// the grade pull (`:113-135`). What survives is `speed.y + gravity`, the `maxFallSpeed` clamp
    /// (`:166-168`) and `fopAcM_posMove(i_this, p_add_vel)` (`:173`), whose `p_add_vel` is the `cCd`
    /// momentum channel `mStts.GetCCMoveP()` - the channel `fallrock.rs` also reports absent.
    fn fall(&mut self, actor: &mut Actor) {
        actor.speed.y += actor.gravity;
        if actor.speed.y < MAX_FALL {
            actor.speed.y = MAX_FALL;
        }
        actor.pos += actor.speed;
        actor.shape_angle[0] = actor.shape_angle[0].wrapping_add(TUMBLE_STEP);
    }

    /// `on_switch` (`:108-113`): `if (switchNo != 0xff) fopAcM_onSwitch(this, switchNo)`.
    fn latch(&self, switches: &mut Switches) {
        if self.switch_no != NO_SWITCH {
            switches.set(self.switch_no);
        }
    }

    /// `mode_fine_init` (`:538-579`): the collider's three `SPrm` bits go off (`:561-563`), the
    /// carry attention bit goes off (`:575`) and `m648 = 3` (`:578`).
    fn rubble(&mut self) {
        self.mode = MODE_FINE;
        self.broken = true;
    }

    /// `damaged(int arg1)` (`:250-274`).
    ///
    /// Omitted from that function, with the decomp line: the `eff_m_break` / `eff_b_break` /
    /// `eff_break_ebrock` / `eff_break_ekao` / `eff_break_ebrock2` bodies and `attr().m54`, `m58`,
    /// `m5C` (`:329-430`) are particle work; `dComIfGp_getVibration().StartShock(attr().m50,
    /// -0x21, cXyz(0.0f, 1.0f, 0.0f))` (`:268-269`) and `mStts.SetWeight(200)` (`:543`) have no
    /// home in this sim; `set_senv(0xff, 10)` (`:271`) is `dKy_Sound_set` (`:477`), a sound-position
    /// marker. What is left is the item, the SE and the switch.
    fn damaged(
        &self,
        action: u8,
        actor: &Actor,
        attr: Attr,
        switches: &mut Switches,
        events: &mut SimEvents,
        commands: &mut Commands,
    ) {
        // `fopAcM_createItemFromTable(&current.pos, prm_get_itemNo(), prm_get_itemSave(),
        // fopAcM_GetHomeRoomNo(this), 0, &csXyz(0, home.angle.y, 0), arg1, NULL)` (`:251-254`).
        // Its `0x20 <= i_itemNo <= 0x3e` arm resolves through `dComIfGp_getItemTable()` and
        // `cM_rndF(15.9999f)` (`src/f_op/f_op_actor_mng.cpp:817-889`) - a stage item table that does
        // not exist here, so that whole arm is omitted rather than guessed.
        if self.item_no < NO_ITEM {
            // `MAKE_ITEM_PARAMS(itemNo, i_itemBitNo, switchNo2 = -1, type = 0, action)`
            // (`src/f_op/f_op_actor_mng.cpp:32`, called from `:983` with the values `damaged` supplies).
            let parameters = u32::from(self.item_no)
                | (u32::from(self.item_save) & ITEM_SAVE_PARAM_MASK) << ITEM_SAVE_PACK_SHIFT
                | SWITCH_NO_SLOT << SWITCH_NO_PACK_SHIFT
                | (u32::from(action) & ACTION_MASK) << ACTION_SHIFT;
            commands.spawn((
                Actor {
                    name: ITEM_ACTORS[0].to_owned(),
                    parameters,
                    pos: actor.pos,
                    angle: actor.angle,
                    set_id: 0,
                    room: actor.room,
                    ..Default::default()
                },
                Pickup::spawn(parameters),
            ));
        }
        events.list.push(SimEvent::Se(attr.break_se)); // fopAcM_seStart(this, attr().m18, 0) (:272)
        self.latch(switches); // on_switch() (:273)
    }
}

pub fn is_stone(name: &str) -> bool {
    STONE_ACTORS.contains(&name)
}

struct Blow {
    live: bool,
    pos: Vec3,
}

fn blow(combat: &Combat, player: Option<&Player>) -> Blow {
    Blow {
        live: combat.active && combat.frame >= CUT_START && combat.frame < CUT_END,
        pos: player.map_or(Vec3::ZERO, |player| {
            Vec3::new(
                player.pos.x + SWORD_OFFSET * player.yaw.sin(),
                player.pos.y,
                player.pos.z + SWORD_OFFSET * player.yaw.cos(),
            )
        }),
    }
}

pub fn control(
    collision: Option<Res<Collision>>,
    combat: Res<Combat>,
    blasts: Res<Blasts>,
    mut carry: ResMut<Carry>,
    mut switches: ResMut<Switches>,
    mut events: ResMut<SimEvents>,
    mut rocks: Query<(Entity, &mut Actor, &mut Stone)>,
    players: Query<&Player>,
    mut commands: Commands,
) {
    let solid = collision.as_deref();
    let swing = blow(&combat, players.iter().next());
    let fired: Vec<Vec3> = blasts.list.clone();
    for (entity, mut actor, mut stone) in &mut rocks {
        let actor = &mut *actor;
        let stone = &mut *stone;
        // `Mthd_Create` (`:182-183`) returns `cPhs_ERROR_e` whenever `chk_appear()` is false, which
        // cancels the process before `Create` ever runs. RULE 6b: spawn it, despawn on tick one.
        if !stone.appears && !stone.chk_appear(&switches) {
            let _ = commands.entity(entity).despawn();
            continue;
        }
        if stone.broken {
            continue;
        }
        let attr = stone.attr();
        let held = carry.held == Some(entity);
        // `Execute` (`:702-708`) runs `damage_cc_proc() || damage_bg_proc()` ahead of
        // `mode_proc_call`, gated on `m648 != 3`; `damage_bg_proc` is the constant false (`:299-301`).
        let cut = swing.live && stone.sword_struck(actor.pos, swing.pos, SWORD_REACH);
        if cut || stone.bomb_struck(actor.pos, &fired) {
            if held {
                carry.held = None; // fopAcM_cancelCarryNow(this) (:255)
            }
            let action = if cut { DA_ITEM_ACT_3 } else { DA_ITEM_ACT_7 };
            stone.damaged(
                action,
                &*actor,
                attr,
                &mut switches,
                &mut events,
                &mut commands,
            );
            stone.rubble(); // mode_fine_init() (:705)
            continue;
        }
        match stone.mode {
            MODE_WAIT => {
                if held {
                    stone.begin_carry(&mut switches);
                }
            }
            MODE_CARRY => {
                if !held {
                    stone.begin_drop(actor, attr);
                }
            }
            MODE_DROP => {
                stone.fall(actor);
                // `mAcch.CrrPos(*dComIfG_Bgsp())` (`:607`) precedes `damage_bg_proc_directly()`
                // (`:613`, `:303-327`), whose ground arm calls `damaged(2)` then `mode_fine_init`.
                if stone.shatters_on_landing(ground::grounded(solid, actor.pos)) {
                    stone.damaged(
                        DA_ITEM_ACT_2,
                        &*actor,
                        attr,
                        &mut switches,
                        &mut events,
                        &mut commands,
                    );
                    stone.rubble();
                }
            }
            _ => {}
        }
    }
}
