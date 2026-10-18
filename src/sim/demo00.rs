//! The stage-cut actor shell - `fpcNm_DEMO00_e`, `d_a_demo00.cpp`, the actor behind the 32 stage
//! strings `d_act0` .. `d_act31`.
//!
//! These names are NOT stage-placed actor records in the ordinary sense: retail creates them
//! itself. `dDemo_system_c::JSGFindObject` (`d_demo.cpp:642-671`) looks the name up, finds nothing
//! in the stage, and on a `strncmp(name, "d_act", 5) == 0` hit calls `fopAcM_fastCreate(name, 0)`
//! followed by `fopAcM_setStageLayer(ac)` and `mObject->appendActor(ac)`. `appendActor`
//! (`d_demo.cpp:545-562`) is what assigns `ac->demoActorID = mNumActor`.
//!
//! `daDemo00_c::execute` (`:631-822`) opens with the demo table lookup:
//!
//! ```text
//! dDemo_actor_c* demo_actor = dComIfGp_demo_getActor(demoActorID);
//! if (demo_actor == NULL) { fopAcM_delete(this); } else { ... }
//! ```
//!
//! (`:636-639`). `fopAcM_delete` is a real destruction, not the RULE 6b delete request, so an actor
//! with no slot is a one-tick despawn - which is what `super::demo::DemoStage::find_object` hands it
//! when a timeline asks for the name and nothing else answers. With a slot, the `else` branch runs:
//! the shape and animation ids are staged, the parameter record's own actions fire (event bits,
//! items, monotone, vibration and the fader), and `action(demo_actor)` runs the model half.
//!
//! The model half - `actStandby`'s heap entry (`:296-313`), `actPerformance` (`:315-511`) and
//! `actLeaving` (`:513-524`) - is a J3D model built out of `dStage_roomControl_c::getDemoArcName()`,
//! which [`super::demo::RoomDemoArchive`] mounts and reads a staged shape id out of
//! (`d_a_demo00.cpp:169`). [`Demo00::sync_model`] is the drop-and-rebuild the three actions make,
//! written into the demo actor's node table; the staged `mBckID` rides on that table as its morph
//! (`:223-253`, `:304-305`) and `actPerformance`'s frame half poses it (`:412-452`), so a cut's
//! joints move. The btp/btk/brk keys (`:173-221`) are the table's material half: each staged one
//! registers on the slots its entries name, and `actPerformance`'s frame half steps its cursor and
//! writes what it samples back, so a cut's material slots animate where they stood still. The
//! transform write those actions end with
//! (`dDemo_setDemoData(this, ENABLE_TRANS|ENABLE_ROTATE|ENABLE_ANM, NULL, NULL)`, `:400`, which is
//! the `morf == NULL` shape) is applied straight off the demo actor instead.
//!
//! What the model half does not carry is the draw-time shape mask `createHeap` builds beside it:
//! the `field_0x29c == 3` arm makes a `mDoExt_invisibleModel` (`:254-265`) and `draw` runs that
//! instead of the model's own display list (`:546-556`), which is a packet list over the model's
//! shapes (`m_Do_ext.h:537-553`). The port's draw path has whole mesh groups rather than per-shape
//! packets, and a demo actor draws as its cull box, so `mDoExt_invisibleModel` and `draw`'s
//! draw-list switches (`dComIfGd_setListMaskOff` / `setListP1`, `:560-562`) have no side here.
//! Neither is staged as an animation key: the actor never builds a `mDoExt_bvaAnm`, so the cut's
//! joint, material and visibility keys are the morph, the btp/btk/brk passes and nothing else.
//!
//! (a) inbound damage: NONE. (b) outbound damage: NONE. `daDemo00_c` registers no `dCcD` collider -
//! its only ground query is `mModel.mBgc->mGndChk` (`:403`) - and the profile `Group` is
//! `fopAc_ACTOR_e` (`:863`). Neither `combat::HitPoints` nor `enemy::Hurt` is attached.
//!
//! `daDemo00_IsDelete` (`:825-828`) is the bare "accepts a delete request" form of RULE 6b.
//! `daDemo00_Create` (`:837-840`) delegates to `create` (`:37-46`), which always returns
//! `cPhs_COMPLEATE_e`, so nothing here refuses at spawn time.

use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::demo::{
    self, ACTOR_ENABLE_ANM, ACTOR_ENABLE_ANM_FRAME, ACTOR_ENABLE_ROTATE, ACTOR_ENABLE_SHAPE,
    ACTOR_ENABLE_TRANS, ACTOR_ENABLE_UNK, DemoActor, DemoGap, DemoItemGet, DemoManager,
    DemoNotices,
};
use super::fade::{BLACK_COLOR, Fader, Monotone, SAFTY_WHITE_COLOR};
use super::flags::StoryBits;
use super::vibration::Vibration;

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

/// The parameter id whose `l_eventBit` table the action switch reads - `d_a_demo00.cpp:651-713`.
const PRM_ACTION_EVENT_BIT: u32 = 4;
/// `l_itemNo`, `:714-736`.
const PRM_ACTION_ITEM: u32 = 5;
/// `setMonotoneRateSpeed`, `:737-744`.
const PRM_ACTION_MONOTONE: u32 = 6;
/// `StartShock` / `StartQuake` / `StopQuake`, `:745-758`.
const PRM_ACTION_VIBRATION: u32 = 7;
/// `startFadeOut` / `startFadeIn` with the fader's colour, `:759-785`. The two ids differ only in
/// which colour they set.
const PRM_ACTION_FADE_BLACK: u32 = 9;
const PRM_ACTION_FADE_WHITE: u32 = 10;

/// `l_eventBit[]` - `d_a_demo00.cpp:652-703`, all 50 rows. `0xFFFF` is the table's own `-1` and
/// the value the code tests against (`:710`). The comments are the reference's own flag names
/// (`d_save_event_flag.inc`).
pub const DEMO00_EVENT_BITS: [u16; 50] = [
    0xFFFF, // 0x00
    0x2A80, // 0x01 UNK_2A80 - the Hero's Clothes sequence's flag
    0xFFFF, // 0x02
    0xFFFF, // 0x03
    0xFFFF, // 0x04
    0x2401, // 0x05 UNK_2401
    0xFFFF, // 0x06
    0xFFFF, // 0x07
    0xFFFF, // 0x08
    0xFFFF, // 0x09
    0xFFFF, // 0x0A
    0xFFFF, // 0x0B
    0xFFFF, // 0x0C
    0xFFFF, // 0x0D
    0xFFFF, // 0x0E
    0xFFFF, // 0x0F
    0xFFFF, // 0x10
    0x2110, // 0x11 UNK_2110
    0xFFFF, // 0x12
    0xFFFF, // 0x13
    0xFFFF, // 0x14
    0xFFFF, // 0x15
    0xFFFF, // 0x16
    0x2D01, // 0x17 UNK_2D01
    0xFFFF, // 0x18
    0xFFFF, // 0x19
    0xFFFF, // 0x1A
    0xFFFF, // 0x1B
    0xFFFF, // 0x1C
    0xFFFF, // 0x1D
    0xFFFF, // 0x1E
    0xFFFF, // 0x1F
    0xFFFF, // 0x20
    0xFFFF, // 0x21
    0xFFFF, // 0x22
    0xFFFF, // 0x23
    0xFFFF, // 0x24
    0xFFFF, // 0x25
    0xFFFF, // 0x26
    0xFFFF, // 0x27
    0xFFFF, // 0x28
    0xFFFF, // 0x29
    0xFFFF, // 0x2A
    0xFFFF, // 0x2B
    0xFFFF, // 0x2C
    0xFFFF, // 0x2D
    0xFFFF, // 0x2E
    0xFFFF, // 0x2F
    0xFFFF, // 0x30
    0x3802, // 0x31 COLORS_IN_HYRULE
];

/// `l_itemNo[]` - `d_a_demo00.cpp:715-726`, ten ids. `dItemNo_NONE_e` is the last row and the one
/// the code tests against (`:733`).
pub const DEMO00_ITEMS: [u8; 10] = [
    0x39, // dItemNo_MASTER_SWORD_1_e  (d_item_data.h:64)
    0x3A, // dItemNo_MASTER_SWORD_2_e  (:65)
    0x3E, // dItemNo_MASTER_SWORD_3_e  (:67)
    0x6A, // dItemNo_PEARL_DIN_e       (:113)
    0x6B, // dItemNo_PEARL_FARORE_e    (:114)
    0x69, // dItemNo_PEARL_NAYRU_e     (:112)
    0x30, // dItemNo_DELIVERY_BAG_e    (:55)
    0x3B, // dItemNo_SHIELD_e          (:66)
    0x35, // dItemNo_MAGIC_ARROW_e     (:60)
    0xFF, // dItemNo_NONE_e
];

/// The ids `execute` stages for `actStandby`'s heap entry - `daDemo00_resID_c`
/// (`d_a_demo00.h:16-28`): `mShapeID`, `mBckID`, `field_0x08`, `mBtpID`, `mBtkID`, `mBrkID`,
/// `field_0x18`, `field_0x1C`, `mPlightID`, `mShadowID`. Every field starts at `-1`
/// (`d_a_demo00.cpp:49-60`) and the model half is what reads them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StagedIds {
    pub shape_id: i32,
    pub bck_id: i32,
    pub field_0x08: i32,
    pub btp_id: i32,
    pub btk_id: i32,
    pub brk_id: i32,
    pub field_0x18: i32,
    pub field_0x1c: i32,
    pub plight_id: i32,
    pub shadow_id: i32,
}

impl StagedIds {
    /// `reset()` - `d_a_demo00.cpp:49-60`.
    pub fn new() -> Self {
        Self {
            shape_id: -1,
            bck_id: -1,
            field_0x08: -1,
            btp_id: -1,
            btk_id: -1,
            brk_id: -1,
            field_0x18: -1,
            field_0x1c: -1,
            plight_id: -1,
            shadow_id: -1,
        }
    }

    /// `mID.reset()` - `d_a_demo00.cpp:63-74`, the same `-1` sweep, run when a model is dropped.
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

/// One demo-cut actor. `demo_actor_id` lives on the [`Actor`] (`f_op_actor.h:281`); the fields here
/// are the two the parameter action switch keeps across ticks plus the staged ids.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Demo00 {
    /// `field_0x29c` - `:633`, `:648-650`: the parameter id last seen.
    prm_id: u32,
    /// `field_0x29d` - `:642`, `:766-771`: the fader argument last applied.
    fade_arg: i32,
    /// `mNextID` - `d_a_demo00.h:33`: the ids `execute` stages each tick.
    pub next: StagedIds,
    /// `mModel.mID` - `d_a_demo00.h:47`: the ids the model was built from. It meets `next` where
    /// [`Demo00::sync_model`] rebuilt the model, and a pair they disagree on is the rebuild
    /// `actPerformance` asks for (`:318-320`).
    pub model: StagedIds,
}

impl Demo00 {
    /// `create` (`:37-46`) minus `fopAcM_ct` (`:38`), `dKy_tevstr_init` (`:39`), `setBaseMtx` and
    /// the three stores `setAction(&daDemo00_c::actStandby)` (`:40`), `mNextID.reset()` (`:41`) and
    /// `field_0x29d = -1` (`:43`). The profile's `Parameters` field is `0` (`:859`).
    pub fn spawn(_parameters: u32) -> Self {
        Self {
            prm_id: 0,
            fade_arg: -1,
            next: StagedIds::new(),
            model: StagedIds::new(),
        }
    }

    /// `fopAc_CULLBOX_0_e` (`:864`) -> `l_cullSizeBox[0]` (`f_op_actor_mng.cpp:558`).
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `execute` (`:631-822`), returning `true` for the `fopAcM_delete(this)` it ends in.
    ///
    /// `demo_actor` is the `dComIfGp_demo_getActor(demoActorID)` lookup of `:636`. The three
    /// flag-gated id stores (`:640-645`) and the whole `ENABLE_UNK_e` switch (`:646-817`) run
    /// first; the model half `action(demo_actor)` (`:819`) runs is [`Demo00::sync_model`], off the
    /// staged ids this leaves behind.
    pub fn execute(
        &mut self,
        actor: &mut Actor,
        demo_actor: Option<&DemoActor>,
        ctx: &mut Env<'_>,
    ) -> bool {
        let Some(slot) = demo_actor else {
            return true;
        };
        if slot.check(ACTOR_ENABLE_SHAPE) != 0 {
            self.next.shape_id = slot.shape_id() as i32;
        }
        if slot.check(ACTOR_ENABLE_ANM) != 0 {
            self.next.bck_id = slot.anm_id() as i32;
        }
        if slot.check(ACTOR_ENABLE_UNK) != 0 {
            let previous = self.prm_id;
            self.prm_id = slot.prm().id;
            self.action(slot, previous, ctx);
        }
        // `actPerformance` ends with `dDemo_setDemoData(this, TRANS|ROTATE|ANM, NULL, NULL)` -
        // `:398` - and `morf == NULL` is the transform-only shape (`d_demo.cpp:206-207`), so the
        // demo actor's position, angle and scale land on this actor the way they do in retail.
        let mask = ACTOR_ENABLE_TRANS | ACTOR_ENABLE_ROTATE | ACTOR_ENABLE_ANM;
        demo::apply_transform(actor, slot, mask);
        false
    }

    /// `TParseData_fixed<S>` (`d_a_demo00.cpp:705-708`): the record's first byte is the
    /// handler tag `S` and the iterator `begin()`s on the byte after it, so a parameter
    /// argument is `data[1]`, never `data[0]`. `isValid()` is the tag test; an
    /// untagged record is ignored the way retail's `!spCC.isValid()` ignores it.
    fn content(data: &[u8], tag: u8) -> Option<&[u8]> {
        match data.split_first() {
            Some((&first, rest)) if first == tag => Some(rest),
            _ => None,
        }
    }

    /// `TParseData_fixed<49, …>` — ASCII `'1'` (`d_a_demo00.cpp:705`).
    const TAG_ONE: u8 = 49;
    /// `TParseData_fixed<33, …>` — ASCII `'!'` (`d_a_demo00.cpp:739`).
    const TAG_BANG: u8 = 33;
    /// `TParseData_fixed<51, …>` — ASCII `'3'` (`d_a_demo00.cpp:789`).
    const TAG_THREE: u8 = 51;

    /// The `ENABLE_UNK_e` switch - `:646-817`. `previous` is the reference's `r29`, the parameter id
    /// from the tick before (`:648`).
    fn action(&mut self, demo_actor: &DemoActor, previous: u32, ctx: &mut Env<'_>) {
        let prm = demo_actor.prm();
        match prm.id {
            PRM_ACTION_EVENT_BIT => {
                let Some(arg) =
                    Self::content(&prm.data, Self::TAG_ONE).and_then(|rest| rest.first().copied())
                else {
                    return;
                };
                let Some(&bit) = DEMO00_EVENT_BITS.get(usize::from(arg)) else {
                    return;
                };
                if bit != 0xFFFF {
                    ctx.stories.set(bit);
                }
            }
            PRM_ACTION_ITEM => {
                let Some(arg) =
                    Self::content(&prm.data, Self::TAG_ONE).and_then(|rest| rest.first().copied())
                else {
                    return;
                };
                let Some(&item) = DEMO00_ITEMS.get(usize::from(arg)) else {
                    return;
                };
                if item != 0xFF {
                    ctx.notices.item_gets.push(DemoItemGet { item });
                }
            }
            PRM_ACTION_MONOTONE => {
                // `TValueIterator_misaligned<s8>` - `:739`: a signed byte of speed.
                let Some(speed) = Self::content(&prm.data, Self::TAG_BANG)
                    .and_then(|rest| rest.first().copied())
                    .map(|byte| byte as i8)
                else {
                    return;
                };
                ctx.monotone.set_rate_speed(i16::from(speed));
            }
            PRM_ACTION_VIBRATION => {
                let Some(arg) =
                    Self::content(&prm.data, Self::TAG_ONE).and_then(|rest| rest.first().copied())
                else {
                    return;
                };
                let at = Vec3::new(0.0, 1.0, 0.0);
                if arg < 100 {
                    // `StartShock(argID, 1, ...)` - `:752`, whose `1` is `RUMBLE_SHOCK`'s flags half.
                    ctx.vibration.start_shock(i32::from(arg), 1, at);
                } else if arg != 0xFF {
                    ctx.vibration.start_quake(i32::from(arg) - 100, 1, at); // <- :754
                } else {
                    ctx.vibration.stop_quake(1); // <- :756
                }
            }
            PRM_ACTION_FADE_BLACK | PRM_ACTION_FADE_WHITE => {
                let Some(rest) = Self::content(&prm.data, Self::TAG_BANG) else {
                    return;
                };
                let Some(first) = rest.first().copied().map(|byte| byte as i8) else {
                    return;
                };
                let second = rest.get(1).copied().map(|byte| byte as i8).unwrap_or(0);
                let changed = self.prm_id != previous || i32::from(first) != self.fade_arg;
                if !changed {
                    return;
                }
                self.fade_arg = i32::from(first);
                if first == 0 {
                    ctx.fader.start_fade_out(i32::from(second)); // <- :777 startFadeOut
                } else {
                    ctx.fader.start_fade_in(i32::from(second)); // <- :779 startFadeIn
                }
                ctx.fader.set_color(if prm.id == PRM_ACTION_FADE_BLACK {
                    BLACK_COLOR // <- :782
                } else {
                    SAFTY_WHITE_COLOR
                });
            }
            _ => {
                // The remaining ids stage the texture, tev, light and shadow ids in the
                // key/value pairs `TValueIterator_misaligned<u32>` walks (`:787-816`). Those ids
                // index `dComIfG_getObjectIDRes` lookups in the demo archive, so they are kept as
                // staged values rather than resolved.
                let Some(rest) = Self::content(&prm.data, Self::TAG_THREE) else {
                    return;
                };
                let mut pairs = rest.chunks_exact(4);
                while let Some(pair) = pairs.next() {
                    let key = u32::from_be_bytes([pair[0], pair[1], pair[2], pair[3]]);
                    let Some(value) = pairs.next() else {
                        break;
                    };
                    let value = i32::from_be_bytes([value[0], value[1], value[2], value[3]]);
                    match key {
                        0 => self.next.btp_id = value,
                        1 => self.next.btk_id = value,
                        2 => self.next.plight_id = value,
                        3 => self.next.field_0x1c = value,
                        4 => self.next.brk_id = value,
                        5 => self.next.shadow_id = value,
                        6 => self.next.btk_id = value | 0x1000_0000, // <- :808 DEMO_SELECT
                        7 => self.next.brk_id = value | 0x1000_0000, // <- :810
                        _ => {}
                    }
                }
            }
        }
    }

    /// The staged ids' `actStandby` gate - `:283`: a shape or a point light is what the heap entry
    /// needs before it can build the model.
    pub fn staging_ready(&self) -> bool {
        self.next.shape_id != -1 || self.next.plight_id != -1
    }

    /// The model half of `execute`'s `action(demo_actor)` (`:819`), which is `actStandby`'s heap
    /// entry (`:296-313`) and `actPerformance`'s per-key swaps (`:318-397`). A changed shape or
    /// point light drops the model (`:318-320`) and rebuilds it out of the room's demo archive
    /// the way `createHeap` reads it (`dComIfG_getObjectIDRes(dStage_roomControl_c::getDemoArcName(),
    /// mShapeID)`, `:169`), written where `act->setModel(mModel.mpModel)` writes it (`:303`).
    /// The build carries `createHeap`'s animation half: the staged `mBckID` becomes the model's
    /// morph (`:223-253`), `actStandby`'s `setAnmFrameMax` takes the morph's frame count
    /// (`:304-305`), and the btp, btk and brk keys (`:173-221`) register on the material slots
    /// their entries name, each with the cursor `initPlay` leaves at zero.
    ///
    /// With the shape and point light unchanged, only the staged animation ids have moved, and
    /// the reference re-inits each key on the model it already has (`:322-397`): the bck swap
    /// keeps the cursor (`setAnm`, `:332`), the btk and brk swaps take the
    /// [`demo::PASS_SELECT_BIT`] attribute and keep a self-advancing pass's frame (`:365`, `:392`).
    /// A staged `-1` is that arm's own early return (`anm == NULL`), so the pass it would replace
    /// is left where it is. Those arms dereference their own `mpBtpAnm`, `mpBtkAnm` or `mpBrkAnm`
    /// without a NULL check, which a cut that never staged that key does not have; the port
    /// registers the pass it resolved instead of following that read.
    ///
    /// Every tick the model stands, `actPerformance`'s frame half runs (`:412-511`): with
    /// `ENABLE_ANM_FRAME_e` up the morph and the three passes are set to `getAnmFrame()`; without
    /// the bit the `else if` chain (`:481-490`) plays the morph, or the first staged pass when
    /// there is no morph.
    ///
    /// `shape_id == -1` is `createHeap`'s own gate (`:168`): nothing is built, and the drop is
    /// still the answer, because `daDemo00_model_c::reset` nulls `mpModel` (`:63-73`) without
    /// telling the demo actor. The port clears the table there instead of leaving the demo actor
    /// pointing at the model the reference's heap entry has already dropped.
    pub fn sync_model(
        &mut self,
        demo: &mut DemoManager,
        archive: &mut demo::RoomDemoArchive,
        slot: u8,
    ) -> Vec<DemoGap> {
        let mut gaps = Vec::new();
        let rebuilt = self.model.shape_id != self.next.shape_id
            || self.model.plight_id != self.next.plight_id;
        if rebuilt {
            let shape = self.next.shape_id;
            self.model = self.next;
            if let Some(actor) = demo.actor_mut(slot) {
                actor.set_model(None);
                if shape != -1 {
                    match archive.model(shape as u16) {
                        Ok(mut table) => {
                            if self.next.bck_id != -1 {
                                match archive.morph(shape as u16, self.next.bck_id as u16) {
                                    Ok(morph) => {
                                        table.set_morph(morph);
                                        // `setBaseMtx`'s `mpModel->calc()` (`:84-91`) runs the
                                        // morph at the zero frame `initPlay` left its cursor at,
                                        // before `actPerformance` reads the demo actor's frame.
                                        table.set_frame(0.0);
                                        // `if (mModel.mpMorf != NULL) act->setAnmFrameMax(
                                        // mModel.mpMorf->getEndFrame())` - `:304-305`.
                                        if let Some(frame_max) = table.morph_frame_max() {
                                            actor.set_anm_frame_max(frame_max);
                                        }
                                    }
                                    Err(gap) => gaps.push(gap),
                                }
                            }
                            gaps.extend(self.build_material_passes(archive, &mut table));
                            actor.set_model(Some(table));
                        }
                        Err(gap) => gaps.push(gap),
                    }
                }
            }
        } else if self.model != self.next {
            let previous = self.model;
            self.model = self.next;
            if let Some(table) = demo.actor_mut(slot).and_then(DemoActor::model_mut) {
                if previous.bck_id != self.next.bck_id
                    && self.next.bck_id != -1
                    && table.morph_frame_max().is_some()
                {
                    // `if (mModel.mpMorf != NULL && mModel.mID.mBckID != mNextID.mBckID)`
                    // (`:322`): the clip is swapped under the standing cursor, which keeps its
                    // frame. A cut with no morph takes that arm's own skip.
                    let cursor = table.frame();
                    match archive.morph(self.next.shape_id as u16, self.next.bck_id as u16) {
                        Ok(morph) => {
                            table.set_morph(morph);
                            if let Some(frame) = cursor {
                                table.set_frame(frame);
                            }
                        }
                        Err(gap) => gaps.push(gap),
                    }
                }
                if previous.btp_id != self.next.btp_id && self.next.btp_id != -1 {
                    match archive.tex_pattern(self.next.btp_id as u16) {
                        Ok(pass) => table.set_tex_pattern(pass),
                        Err(gap) => gaps.push(gap),
                    }
                }
                if previous.btk_id != self.next.btk_id && self.next.btk_id != -1 {
                    match archive.texture_srt(self.next.btk_id as u16) {
                        Ok(pass) => {
                            table.set_texture_srt(pass, self.next.btk_id, demo::PassInit::Swap)
                        }
                        Err(gap) => gaps.push(gap),
                    }
                }
                if previous.brk_id != self.next.brk_id && self.next.brk_id != -1 {
                    match archive.tev_reg(self.next.brk_id as u16) {
                        Ok(pass) => table.set_tev_reg(pass, self.next.brk_id, demo::PassInit::Swap),
                        Err(gap) => gaps.push(gap),
                    }
                }
            }
        }
        let animated = demo
            .actor(slot)
            .filter(|actor| actor.check(ACTOR_ENABLE_ANM_FRAME) != 0)
            .map(DemoActor::anm_frame);
        if let Some(model) = demo.actor_mut(slot).and_then(DemoActor::model_mut) {
            match animated {
                Some(frame) => {
                    model.set_frame(frame);
                    model.set_pass_frames(frame);
                }
                None => model.play_pass(),
            }
        }
        gaps
    }

    /// The btp, btk and brk arms of `createHeap` (`:173-221`): each staged id is resolved out of
    /// the room's demo archive and registered on the material slots its entries name, with the
    /// cursor `initPlay` leaves at zero. `id == -1` is that arm's own guard (`:173`, `:189`,
    /// `:209`), so no pass is built for that key. The reference assigns its wrapper inside the
    /// same guard, which leaves a previous cut's wrapper in place for a key the timeline then
    /// lowers to `-1`, and its draw would re-register that stale animator on this model's
    /// material table; the port builds each model's passes from its own staged ids instead.
    fn build_material_passes(
        &self,
        archive: &mut demo::RoomDemoArchive,
        table: &mut demo::DemoNodeTable,
    ) -> Vec<DemoGap> {
        let mut gaps = Vec::new();
        if self.next.btp_id != -1 {
            match archive.tex_pattern(self.next.btp_id as u16) {
                Ok(pass) => table.set_tex_pattern(pass),
                Err(gap) => gaps.push(gap),
            }
        }
        if self.next.btk_id != -1 {
            match archive.texture_srt(self.next.btk_id as u16) {
                Ok(pass) => table.set_texture_srt(pass, self.next.btk_id, demo::PassInit::Heap),
                Err(gap) => gaps.push(gap),
            }
        }
        if self.next.brk_id != -1 {
            match archive.tev_reg(self.next.brk_id as u16) {
                Ok(pass) => table.set_tev_reg(pass, self.next.brk_id, demo::PassInit::Heap),
                Err(gap) => gaps.push(gap),
            }
        }
        gaps
    }
}

/// The mutable resources `execute`'s parameter actions reach, grouped so the system below stays
/// readable.
pub struct Env<'a> {
    pub stories: &'a mut StoryBits,
    pub notices: &'a mut DemoNotices,
    pub vibration: &'a mut Vibration,
    pub fader: &'a mut Fader,
    pub monotone: &'a mut Monotone,
}

pub fn is_demo00(name: &str) -> bool {
    DEMO00_ACTORS.contains(&name)
}

pub fn control(
    mut actors: Query<(Entity, &mut Actor, &mut Demo00)>,
    mut demo: ResMut<DemoManager>,
    mut archive: ResMut<demo::RoomDemoArchive>,
    mut stories: ResMut<StoryBits>,
    mut notices: ResMut<DemoNotices>,
    mut vibration: ResMut<Vibration>,
    mut fader: ResMut<Fader>,
    mut monotone: ResMut<Monotone>,
    mut commands: Commands,
) {
    let mut env = Env {
        stories: &mut stories,
        notices: &mut notices,
        vibration: &mut vibration,
        fader: &mut fader,
        monotone: &mut monotone,
    };
    for (entity, mut actor, mut demo00) in &mut actors {
        let demo_actor = demo.actor(actor.demo_actor_id);
        if demo00.execute(&mut actor, demo_actor, &mut env) {
            // `fopAcM_delete(this)` is where `execute` ends when the demo table has no slot
            // (`d_a_demo00.cpp:636-639`), so the model half does not run for a deleted actor.
            let _ = commands.entity(entity).despawn();
            continue;
        }
        for gap in demo00.sync_model(&mut demo, &mut archive, actor.demo_actor_id) {
            log::warn!("clearsleeper: demo cut model gap ({gap})");
        }
    }
}
