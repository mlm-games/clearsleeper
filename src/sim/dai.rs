use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::flags::StoryBits;
use super::player::Player;

/// `OBJNAME("Fdai", fpcNm_DAI_e, 255, 0)` (`d_stage.cpp:986`) is `fpcNm_DAI_e` = `d_a_dai.cpp`, the
/// only such row in the table.
///
/// The object archive is `"Fdai"` (`dComIfG_resLoad(&mPhsFdai, m_arcname)`, `d_a_dai.cpp:127`, with
/// `const char daDai_c::m_arcname[] = "Fdai"` at `:19`), so stage string and archive agree. A second
/// archive, `"Cloth"` (`daDai_c::m_cloth_arcname`, `:20`), is loaded at `:147` for the DEMO-only
/// joint work this checkout does not need.
///
/// The daiza's placement save bits live in `daDai_c::m_savelabel[]` (`d_com_static.cpp:198-239`),
/// NOT in `m_arcname` - a second string table entirely, see [`DAI_SAVE_LABELS`].
/// Nothing else in `src/sim/` claims `Fdai`.
pub const DAI_ACTORS: [&str; 1] = ["Fdai"];

/// `daDai_c::m_savelabel[]` (`d_com_static.cpp:198-239`) - 41 `u16` `dSv_event_flag_c` ids, the
/// first 41 of the `0xD1FF..=0xF8FF` run. `daDai_c::CreateInit` indexes it with `mSaveID`
/// (`d_a_dai.cpp:101`, `:184`, `:221`, `:291`, `:320-321`), which is
/// `mSaveID = fopAcM_GetParam(this)` (`:98`) truncated into the `u8 mSaveID` field
/// (`d_a_dai.h:55`) - so any parameter above 40 reads past the array, which is retail undefined
/// behaviour with no modelled answer; RULE 10 requires the index be guarded, so anything above 40
/// folds to 0.
///
/// Every entry is a `dSv_event_flag_c::UNK_*` whose VALUE is the literal suffix (the enum's own
/// comment gives it: `/* 0xD1FF */ UNK_D1FF = 0xD1FF`, `d_save_event_flag.inc:582`), so the table is
/// a plain descending run of `0xD1FF + i` - transcribed here as explicit values so each row keeps a
/// citation, and note `StoryBits` is byte-indexed by `hi_byte << 8 | mask` (RULE 11c), which is
/// exactly what these ids are.
const DAI_SAVE_LABELS: [u16; 40] = [
    0xF8FF, // <- d_com_static.cpp:199
    0xF7FF, // <- d_com_static.cpp:200
    0xF6FF, // <- d_com_static.cpp:201
    0xF5FF, // <- d_com_static.cpp:202
    0xF4FF, // <- d_com_static.cpp:203
    0xF3FF, // <- d_com_static.cpp:204
    0xF2FF, // <- d_com_static.cpp:205
    0xF1FF, // <- d_com_static.cpp:206
    0xF0FF, // <- d_com_static.cpp:207
    0xEFFF, // <- d_com_static.cpp:208
    0xEEFF, // <- d_com_static.cpp:209
    0xEDFF, // <- d_com_static.cpp:210
    0xECFF, // <- d_com_static.cpp:211
    0xEBFF, // <- d_com_static.cpp:212
    0xEAFF, // <- d_com_static.cpp:213
    0xE9FF, // <- d_com_static.cpp:214
    0xE8FF, // <- d_com_static.cpp:215
    0xE7FF, // <- d_com_static.cpp:216
    0xE6FF, // <- d_com_static.cpp:217
    0xE5FF, // <- d_com_static.cpp:218
    0xE4FF, // <- d_com_static.cpp:219
    0xE3FF, // <- d_com_static.cpp:220
    0xE2FF, // <- d_com_static.cpp:221
    0xE1FF, // <- d_com_static.cpp:222
    0xE0FF, // <- d_com_static.cpp:223
    0xDFFF, // <- d_com_static.cpp:224
    0xDEFF, // <- d_com_static.cpp:225
    0xDDFF, // <- d_com_static.cpp:226
    0xDCFF, // <- d_com_static.cpp:227
    0xDBFF, // <- d_com_static.cpp:228
    0xDAFF, // <- d_com_static.cpp:229
    0xD9FF, // <- d_com_static.cpp:230
    0xD8FF, // <- d_com_static.cpp:231
    0xD7FF, // <- d_com_static.cpp:232
    0xD6FF, // <- d_com_static.cpp:233
    0xD5FF, // <- d_com_static.cpp:234
    0xD4FF, // <- d_com_static.cpp:235
    0xD3FF, // <- d_com_static.cpp:236
    0xD2FF, // <- d_com_static.cpp:237
    0xD1FF, // <- d_com_static.cpp:238
];
const LABEL_COUNT: u32 = 41; // <- d_com_static.cpp:198-239

/// `mEvtDaiItemIdx = dComIfGp_evmng_getEventIdx("DAI_ITEM");` (`d_a_dai.cpp:96`) - the
/// one-argument form, so `evNo` defaults to `-1` (`0xFF`, `d_com_inf_game.h:3645`) and this is a
/// pure name lookup with no stage event-info redirect (`d_event_manager.cpp:347-348`).
const EV_DAI_ITEM: &str = "DAI_ITEM"; // <- d_a_dai.cpp:96
/// `mEvtDefaultTalkIdx = dComIfGp_evmng_getEventIdx("DEFAULT_TALK");` (`d_a_dai.cpp:97`).
const EV_DEFAULT_TALK: &str = "DEFAULT_TALK"; // <- d_a_dai.cpp:97

/// `fopAcM_setCullSizeBox(this, -50.0f, 0.0f, -50.0f, 50.0f, 200.0f, 50.0f)` (`d_a_dai.cpp:79`). The
/// profile's `fopAc_CULLBOX_CUSTOM_e` (`:394`) resolves to it.
/// `fopAcM_setCullSizeFar(this, 5000.0f / mDoLib_clipper::getFar())` (`:86`) is a FAR-PLANE
/// override only and creates no box; its condition `getFar() > 1.0f` (`:85`) is a live renderer
/// setting with no modelled reader.
const CULL_MIN: [f32; 3] = [-50.0, 0.0, -50.0]; // <- d_a_dai.cpp:79
const CULL_MAX: [f32; 3] = [50.0, 200.0, 50.0]; // <- d_a_dai.cpp:79

/// ```
/// f32 tmp1 = 100.0f;
/// f32 tmp2 = 10.0f;
/// ```
/// (`d_a_dai.cpp:256`, `:258`) - `proc`'s proximity radius and height tolerance.
/// `s16 tmp3 = 12000;` (`:265`) - the facing tolerance in `0x10000 == one full turn` units, i.e.
/// roughly 65.9 degrees each way.
const NEAR_XZ: f32 = 100.0; // <- d_a_dai.cpp:256
const NEAR_Y: f32 = 10.0; // <- d_a_dai.cpp:258
const FACING_LIMIT: u16 = 12000;

/// The message-box half of this actor is OMITTED (see the struct docs), so its literals are
/// documented here rather than carried as constants with no reader - RULE 12(b) makes an
/// unread constant dead code. All are `d_a_dai.cpp` verbatim:
/// * `getMsg` (`:287-298`) answers `0xF13` (`:290`, "you already hold this daiza's item"),
///   `0xF0D` (`:292`, "an item is placed here, ask to remove it") or `0xF11` (`:294`, the empty
///   daiza), in that order; the first arm is gated on `dComIfGp_event_chkTalkXY()`.
/// * `next_msgStatus`'s `case 0xF11` (`:304-306`) advances to `0xF12`; its removal branch
///   (`:308-328`) answers `0xF0F` (`:313`) when
///   `dComIfGs_checkReserveItemEmpty() && getRotenItemNumInBag() < 3` (`:311`) and `0xF0E` (`:326`)
///   otherwise; its `default` (`:337-341`) ends the box with `fopMsgStts_MSG_ENDS_e`
///   (`f_op_msg.h:26`).
/// * `JA_SE_LK_W_DAIZA_TAKEOFF = 0x2871` (`JAZelAudio_SE.h:559`) plays at `:312`.
///
/// `isDaizaItem(itemNo)` (`d_item.cpp:2749-2759`) tests the twelve `dItemNo_*` statues from
/// `dItemNo_TOWN_FLOWER_e = 0x8C` (`d_item_data.h:147`) to
/// `dItemNo_SHOP_GURU_STATUE_e = 0x97` (`:158`) - the same span `getRotenItemNumInBag`
/// (`d_item.cpp:2742`) sums `dComIfGs_checkReserveItem` over.
///
/// `l_cyl_src` (`d_a_dai.cpp:22-50`): `SrcObjAt Type` 0 (`:27`), `SrcObjTg Type` 0 (`:29`),
/// `SrcObjCo SPrm = cCcD_CoSPrm_Set_e | cCcD_CoSPrm_IsOther_e | cCcD_CoSPrm_VsGrpAll_e` (`:31`),
/// `SrcGObjAt HitMark = dCcG_AtHitMark_None_e` (`:33`), `SrcGObjTg HitMark = dCcG_Tg_Spl_UNK0`
/// (`:39`), `SrcGObjTg SPrm = dCcG_TgSPrm_NoHitMark_e` (`:41`), `Radius 30.0f` / `Height 30.0f`
/// (`:47-48`). It is a pure COLLISION cylinder that blocks actors - no attack, no damage target.
const CYL_RADIUS: f32 = 30.0; // <- d_a_dai.cpp:47
const CYL_HEIGHT: f32 = 30.0; // <- d_a_dai.cpp:48

/// `dItemNo_DELIVERY_BAG_e` (`d_item_data.h:55`) = `0x30`, the item `_create` gates on
/// (`d_a_dai.cpp:123`).
const ITEM_DELIVERY_BAG: u8 = 0x30; // <- d_item_data.h:55

/// The Delivery Daiza statues - `daDai_c` (`d_a_dai.h:9-58`), the actor behind the single stage
/// string `Fdai`.
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. `l_cyl_src` (`d_a_dai.cpp:22-50`) sets
/// `SrcObjAt Type` to 0 (`:27`) and `SrcObjTg Type` to 0 (`:29`), so the daiza registers no attack
/// collider and no damage target - only a `cCcD_CoSPrm_Set_e | cCcD_CoSPrm_IsOther_e |
/// cCcD_CoSPrm_VsGrpAll_e` collision cylinder (`SrcGObjAt HitMark = dCcG_AtHitMark_None_e`, `:33`;
/// `SrcGObjTg SPrm = dCcG_TgSPrm_NoHitMark_e`, `:41`). The profile `Group` is `fopAc_ACTOR_e`
/// (`:392`). Neither `combat::HitPoints` nor `enemy::Hurt` is attached; attaching them would let
/// `combat::control` despawn a statue retail cannot damage.
///
/// `daDai_IsDelete` (`:368-370`) is the bare "accepts a delete request" form of RULE 6b, and
/// `fopAcM_delete` appears nowhere in this translation unit. `_create` (`:120-160`) can return
/// `cPhs_ERROR_e` from TWO places - the `checkItemGet` guard (`:123-125`) and
/// `fopAcM_entrySolidHeap` (`:154-156`) - but the first is the one that matters and it is modelled:
/// see [`Dai::create_refused`].
///
/// The message-box half of this actor - `XyCheckCB` (`:218-226`), `XyEventCB` (`:234-240`),
/// `checkOrder` (`:179-210`), `getMsg` (`:287-298`) and `next_msgStatus` (`:300-345`) - is DROPPED.
/// See the OMITTED list in the report for the per-function reason.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Dai {
    /// `mSaveID` (`d_a_dai.h:55`, a `u8`) from `mSaveID = fopAcM_GetParam(this)` (`d_a_dai.cpp:98`)
    /// over the WHOLE u32 (`f_op_actor_mng.h:127-129`) - so it is the parameter's low byte, and it
    /// indexes [`DAI_SAVE_LABELS`] (folded above [`LABEL_COUNT`] under RULE 10).
    pub save_id: u8,
    /// `m84C` (`d_a_dai.h:54`, a `u8`) - set by `checkOrder` (`:185`, `:207`) when the talk is
    /// accepted, and cleared by `proc` (`:274`) once the box closes.
    pub talk_accepted: bool,
    /// `m844` (`d_a_dai.h:49`, an `s16`) - `proc`'s "player is close enough and facing me" flag,
    /// zeroed at `:259` and set to 1 at `:267`. `eventOrder` (`:171-176`) raises the
    /// `dEvtCnd_CANTALK_e` / `dEvtCnd_CANTALKITEM_e` conditions from it.
    pub in_reach: bool,
    /// `mEvtDaiItemIdx` (`d_a_dai.h:50`, an `s16`) from `getEventIdx("DAI_ITEM")` (`:96`).
    pub item_slot: Option<usize>,
    /// `mEvtDefaultTalkIdx` (`d_a_dai.h:51`, an `s16`) from `getEventIdx("DEFAULT_TALK")` (`:97`).
    pub talk_slot: Option<usize>,
    /// One-shot guard so `CreateInit` (`:77-117`) runs exactly once, on the first tick, where the
    /// save-bit store and `Commands` are available.
    pub entered: bool,
    /// Whether `_create`'s `checkItemGet(dItemNo_DELIVERY_BAG_e, TRUE)` guard refuses creation.
    /// RULE 6b: a `cPhs_ERROR_e` return from `create()` cancels the process during Create
    /// (`f_pc_create_req.cpp:106-108`), so retail never executes the actor even once - the sim
    /// spawns it and despawns on the first tick via this flag rather than using
    /// `Option<Self> -> None`, which would leak a bare `Actor` entity retail never keeps.
    pub refused: bool,
    /// `dComIfGs_getEventReg(m_savelabel[mSaveID])` (`d_a_dai.cpp:101`) - whether this daiza already
    /// holds a statue. Retail only ever reads it, never writes it, outside the dropped message flow
    /// (`dComIfGs_setEventReg` appears at `:201` and `:321`, both inside `checkOrder` /
    /// `next_msgStatus`), so it is sampled once in `create_init` and stays fixed.
    pub has_item: bool,
    /// `current.pos` - `proc` reads `current.pos.y` (`:257`) alongside `player->current.pos`, and
    /// `control` already holds the `&mut Actor` borrow, so the height is copied in before the call.
    pub pos: Vec3,
}

impl Dai {
    /// `daDai_c::_create` (`d_a_dai.cpp:120-160`) minus the two `dComIfG_resLoad` calls and the
    /// `fopAcM_entrySolidHeap` guard (both asset work), plus `CreateInit`, which runs on the first
    /// tick.
    ///
    /// The one part that survives here is the refusal:
    /// ```
    /// if (!checkItemGet(dItemNo_DELIVERY_BAG_e, TRUE)) { return cPhs_ERROR_e; }
    /// ```
    /// (`:123-125`). `checkItemGet(itemNo, defaultVal)` (`d_item.cpp:539-545`) calls
    /// `item_getcheck_func_ptr[itemNo]()`, and for `0x30` that is
    /// `item_getcheck_func_warasibe_bag` (`d_item.cpp:2225-2227`) = `dComIfGs_isGetItem(0x12, 0)`.
    /// The `defaultVal` of TRUE only applies when the check function returns -1, and this one
    /// returns a real boolean, so the delivery bag must actually be held.
    ///
    /// The sim's `super::inventory::check_get_item` (`src/sim/inventory.rs:144-`) is the established
    /// model of `checkItemGet`, and it takes the item number - so [`Dai::create_refused`] passes
    /// [`ITEM_DELIVERY_BAG`] to it. NOTE the reduction is not bit-exact: retail tests
    /// `dSv_player_get_item_c::mItemFlags[0x12] & 1` (`d_save.cpp:217-221`), i.e. the "warasibe bag
    /// obtained" flag, while `check_get_item`'s fallback is
    /// `(0..60).any(|slot| equipped.item(slot) == item)` (`src/sim/inventory.rs:207`), i.e. "the
    /// bag is in an equipped slot". Both answer the same question in practice - you cannot have the
    /// bag equipped without having obtained it - and using the house helper is preferable to
    /// hand-rolling a private `mItemFlags` model the crate already has a name for.
    ///
    /// RULE 2: `dStage_actorInit` never writes `fopAcM_prm_class::scale`; `set_mtx`'s
    /// `mpModel->setBaseScale(scale)` (`:164`) is renderer work.
    pub fn spawn(parameters: u32) -> Self {
        Self {
            // `mSaveID` (`d_a_dai.h:55`) is a `u8`, so the whole-u32 parameter is truncated to its
            // low byte - NOT shifted. `u32::from` on a `u8` is the narrowing the C++ assignment does.
            save_id: parameters as u8,
            // `m84C` (`:54`) takes `fopAcM_ct`'s `sBs_ClearArea` zero-fill (`f_pc_base.cpp:98`);
            // `checkOrder` (`:185`, `:207`) sets it and `proc` (`:274`) clears it.
            talk_accepted: false,
            // `m844` (`:49`) likewise; `proc` zeroes it at `:259` and sets it at `:267`.
            in_reach: false,
            item_slot: None,
            talk_slot: None,
            entered: false,
            refused: false,
            // `dComIfGs_getEventReg(...)` (`:101`) is sampled in `create_init`, which needs the
            // `StoryBits` store; `fopAcM_ct`'s zero-fill is FALSE until then.
            has_item: false,
            // `fopAc_Init` seeds `current` and `old` from `home` (`f_op_actor.cpp:282-284`), so
            // `current.pos` starts at the spawn position; `control` overwrites it before the first
            // `execute`.
            pos: Vec3::ZERO,
        }
    }

    /// `fopAcM_setCullSizeBox(this, -50.0f, 0.0f, -50.0f, 50.0f, 200.0f, 50.0f)` (`d_a_dai.cpp:79`)
    /// - the actor's one real custom cull box, matching the profile's `fopAc_CULLBOX_CUSTOM_e`
    /// (`:394`).
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `l_cyl_src`'s collision cylinder - `Radius 30.0f` / `Height 30.0f` (`d_a_dai.cpp:47-48`),
    /// re-registered at `current.pos` every tick (`:247-248`). Pure collision: `SrcObjAt Type` and
    /// `SrcObjTg Type` are both 0 (`:27`, `:29`), so it neither attacks nor is a damage target.
    pub fn cylinder(&self) -> (f32, f32) {
        (CYL_RADIUS, CYL_HEIGHT)
    }

    /// `m_savelabel[mSaveID]` - the save-bit id this daiza reads and writes. RULE 10: `mSaveID` is
    /// the parameter's low byte and the table has 41 entries, so a larger value reads past the array
    /// in retail; anything at or above [`LABEL_COUNT`] folds to index 0.
    fn save_label(&self) -> u16 {
        let index = if u32::from(self.save_id) < LABEL_COUNT {
            self.save_id as usize
        } else {
            0
        };
        DAI_SAVE_LABELS[index]
    }

    /// `dComIfGs_getEventReg(m_savelabel[mSaveID])` (`d_a_dai.cpp:101`, `:184`, `:221`, `:291`,
    /// `:320`) as "is this daiza already holding an item".
    ///
    /// `dComIfGs_getEventReg(i_reg)` (`d_com_inf_game.h:1357-1359`) is
    /// `save.getEvent().getEventReg(i_reg)`, and `dSv_event_c::getEventReg`
    /// (`d_save.cpp:1207-1210`) is `(u8)i_reg & mFlags[i_reg >> 8]` - the same byte-indexed shape
    /// `super::flags::StoryBits::reg` implements (`src/sim/flags.rs:49-52`), which is why RULE 11c's
    /// "pass the id straight through, never pre-shift" applies verbatim.
    pub fn placed(&self, bits: &StoryBits) -> bool {
        bits.reg(self.save_label()) != 0
    }

    /// `!checkItemGet(dItemNo_DELIVERY_BAG_e, TRUE)` (`d_a_dai.cpp:123-125`) - see [`Dai::spawn`]
    /// for the full reduction.
    fn create_refused(&mut self, equipped: &super::inventory::Equipped, collect: &super::flags::Collect) {
        self.refused = !super::inventory::check_get_item(equipped, collect, ITEM_DELIVERY_BAG);
    }

    /// `daDai_c::CreateInit` (`d_a_dai.cpp:77-117`), first tick only.
    ///
    /// Dropped: `fopAcM_SetMtx` (`:78`), the cull box (`:79-87`), `mStts.Init` / `mCyl.Set` /
    /// `mCyl.SetStts` (`:81-83`), `set_mtx` (`:89`), `shape_angle = current.angle` (`:90`),
    /// `cLib_onBit<u32>(attention_info.flags, fopAc_Attn_TALKFLAG_CHECK_e |
    /// fopAc_Attn_ACTION_SPEAK_e)` (`:91`; RULE 3b lists attention/catch requests as absent),
    /// `eventInfo.setXyCheckCB` / `setXyEventCB` (`:93-94`), the STANDITEM creation block
    /// (`:101-115`, see below) and `incNowDaizaNum()` (`:116`).
    ///
    /// The STANDITEM block is dropped for two reasons. `dComIfGs_getEventReg` IS modelled
    /// ([`Dai::placed`]), but `fopAcM_fastCreate(fpcNm_STANDITEM_e, itemReg, &current.pos, roomNo,
    /// &current.angle)` (`:102-112`) targets `fpcNm_STANDITEM_e`, which the sim spells `"Sitem"` -
    /// a DIFFERENT actor: `d_stage.cpp:1235` has `OBJNAME("Fobj00", fpcNm_STANDITEM_e, 255, 0)`, so
    /// `fpcNm_STANDITEM_e` is the stage string `"Fobj00"` while `src/sim/sitem.rs` owns `"Sitem"`
    /// (`fpcNm_SITEM_e`, presumably). Creating an `Fobj00` here would need that actor's own module
    /// and its parameter decode, which is out of scope for this file. And `incNowItemNum()`
    /// (`:114`) / `incNowDaizaNum()` (`:116`) write the `static s8 mNowItemNum` /
    /// `mNowDaizaNum` (`d_com_static.cpp:196-197`), which are read only by
    /// `daNpc_c`/npc-people shop logic (`d_a_npc_people.cpp:8273-8278`) - a different translation
    /// unit with no counterpart here.
    fn create_init(
        &mut self,
        list: &super::event::EventList,
        equipped: &super::inventory::Equipped,
        collect: &super::flags::Collect,
        bits: &StoryBits,
    ) {
        // `mEvtDaiItemIdx = dComIfGp_evmng_getEventIdx("DAI_ITEM");` (:96)
        self.item_slot = list.event_index(EV_DAI_ITEM);
        // `mEvtDefaultTalkIdx = dComIfGp_evmng_getEventIdx("DEFAULT_TALK");` (:97)
        self.talk_slot = list.event_index(EV_DEFAULT_TALK);
        // ```
        // if (!checkItemGet(dItemNo_DELIVERY_BAG_e, TRUE)) { return cPhs_ERROR_e; }
        // ```
        // (:123-125) - the guard sits in `_create`, i.e. BEFORE `CreateInit`, so it is evaluated
        // first and a refusal leaves the rest of `CreateInit` unrun, exactly as retail's early
        // return does.
        self.create_refused(equipped, collect);
        if self.refused {
            return;
        }
        // ```
        // if (dComIfGs_getEventReg(m_savelabel[mSaveID])) {
        //     void* dai_item = fopAcM_fastCreate(fpcNm_STANDITEM_e,
        //         dComIfGs_getEventReg(m_savelabel[mSaveID]), &current.pos,
        //         fopAcM_GetRoomNo(this), &current.angle);
        //     m850 = fopAcM_GetID(dai_item);
        //     incNowItemNum();
        // }
        // ```
        // (:101-115) - the STANDITEM creation itself is dropped (see above), but its CONDITION is
        // modelled, because it is what decides whether a daiza is already occupied - a fact the
        // dropped message flow would have read through `getMsg` / `checkOrder`. `mbHasItem` is that
        // one bit, refreshed from `StoryBits` on the first tick.
        self.has_item = self.placed(bits);
        // `mSaveID = fopAcM_GetParam(this);` (:98) - already decoded in `spawn`.
        // `m850 = fpcM_ERROR_PROCESS_ID_e;` (:99) - the "no held STANDITEM proc" sentinel, only ever
        // overwritten by the dropped STANDITEM block, so it needs no storage.
        self.entered = true;
    }

    /// `daDai_c::_execute` (`d_a_dai.cpp:243-250`).
    fn execute(&mut self, player: Vec3, player_shape_yaw: i16) {
        // `checkOrder();` (:244) - dropped (message box).
        self.proc(player, player_shape_yaw);
        // `eventOrder();` (:245) - `if (m844 == 1) { onCondition(CANTALK); onCondition(CANTALKITEM); }`
        // (`:171-176`). The two `onCondition` calls are dropped for the reason `htobi3.rs` gives:
        // nothing in the decomp ever calls `chkCondition` with those ids, and `beforeProc` clears
        // `mCondition` on every `fopAc_Execute` (`f_op_actor.cpp:176`). The gate that sets `m844`
        // is transcribed inside `proc`, which is the part with a modelled input.
        // `mCyl.SetC(current.pos); dComIfG_Ccsp()->Set(&mCyl);` (:247-248) is `cCcD_CoSPrm_*`
        // registration with no channel here; the cylinder's real dimensions are
        // [`Dai::cylinder`].
    }

    /// `daDai_c::proc` (`d_a_dai.cpp:253-276`).
    ///
    /// ```
    /// fopAc_ac_c* player = dComIfGp_getPlayer(0);
    /// f32 sqrt = (player->current.pos - current.pos).absXZ();
    /// f32 tmp1 = 100.0f;
    /// f32 abs = std::fabsf(player->current.pos.y - current.pos.y);
    /// f32 tmp2 = 10.0f;
    /// m844 = 0;
    /// if (sqrt < tmp1) {
    ///     if (abs < tmp2) {
    ///         s16 sVar3 = cLib_targetAngleY(&player->current.pos, &current.pos);
    ///         s32 sVar4 = cLib_distanceAngleS(sVar3, player->shape_angle.y);
    ///         s16 tmp3 = 12000;
    ///         if (sVar4 < tmp3) { m844 = 1; }
    ///     }
    /// }
    /// if ((m84C != 0) && talk(1) == fopMsgStts_BOX_CLOSED_e) {
    ///     dComIfGp_event_reset();
    ///     m84C = 0;
    /// }
    /// ```
    /// * `absXZ()` is the XZ-only magnitude (`c_xyz.h`), so `sqrt` is a distance in the ground plane.
    /// * `cLib_targetAngleY(lhs, rhs)` is `cM_atan2s(rhs->x - lhs->x, rhs->z - lhs->z)`
    ///   (`c_lib.cpp:343-345`) - the yaw from the PLAYER toward the DAIZA, which is the opposite
    ///   sense from a "target yaw" read, and the sign matters: the test passes only when the player
    ///   is facing the statue.
    /// * `cLib_distanceAngleS(x, y)` is `abs(static_cast<s16>(x - y))` (`c_lib.cpp:364-366`) - a
    ///   signed `s16` subtraction that is then ABSOLUTED, so RULE 10 requires `unsigned_abs()` and
    ///   the result is an `s32`.
    /// * `player->shape_angle.y` is the player's BODY facing. The sim's `super::player::Player`
    ///   carries `face` for exactly this - `npc_km1` and `amos_knight` read `player.face` the same
    ///   way - and RULE 5b's single converter `super::walk::to_s16` turns it into angle units.
    ///
    /// The `talk(1)` tail is DROPPED. `talk(1)` is `fopNpc_npc_c::talk(1)` (`d_npc.cpp:664-697`),
    /// which drives `fopMsgM_messageSet` and calls this actor's own `getMsg` / `next_msgStatus` -
    /// i.e. it needs the message-box system plus the select-number and reserve-bag inputs listed
    /// under OMITTED. `fopMsgStts_BOX_CLOSED_e` is `0x12` (`f_op_msg.h:28`), but the path that sets
    /// `m84C` in the first place is `checkOrder`, which is dropped too, so the whole tail is
    /// unreachable in the sim.
    fn proc(&mut self, player: Vec3, player_shape_yaw: i16) {
        let delta = player - self.pos;
        // `std::fabsf(player->current.pos.y - current.pos.y)` (:257)
        let height = (player.y - self.pos.y).abs();
        // `m844 = 0;` (:259)
        self.in_reach = false;
        if (delta.x * delta.x + delta.z * delta.z).sqrt() < NEAR_XZ && height < NEAR_Y {
            // `cLib_targetAngleY(&player->current.pos, &current.pos)` (:263)
            let to_dai = super::walk::to_s16(delta.x.atan2(delta.z));
            // `cLib_distanceAngleS(sVar3, player->shape_angle.y)` (:264)
            let facing = to_dai.wrapping_sub(player_shape_yaw).unsigned_abs();
            if facing < FACING_LIMIT {
                // `m844 = 1;` (:267)
                self.in_reach = true;
            }
        }
        if self.talk_accepted {
            // `if ((m84C != 0) && talk(1) == fopMsgStts_BOX_CLOSED_e) { dComIfGp_event_reset();
            // m84C = 0; }` (:272-275) - the message-box tail, see the doc comment. `m84C` is
            // cleared so a stale acceptance cannot survive the box closing.
            self.talk_accepted = false;
        }
    }
}

pub fn is_dai(name: &str) -> bool {
    DAI_ACTORS.contains(&name)
}

pub fn control(
    list: Res<super::event::EventList>,
    equipped: Res<super::inventory::Equipped>,
    collect: Res<super::flags::Collect>,
    bits: Res<StoryBits>,
    players: Query<&Player>,
    mut daizas: Query<(Entity, &mut Actor, &mut Dai)>,
    mut commands: Commands,
) {
    // `dComIfGp_getPlayer(0)` (`d_a_dai.cpp:254`) is dereferenced unconditionally in `proc`, so a
    // tick with no player actor has no defined behaviour in retail either. The sim skips rather than
    // substituting a position.
    let Some(player) = players.iter().next().copied() else {
        return;
    };
    let player_yaw = super::walk::to_s16(player.face);
    for (entity, mut actor, mut dai) in &mut daizas {
        if !dai.entered {
            dai.create_init(&list, &equipped, &collect, &bits);
            // RULE 6b: `_create` returned `cPhs_ERROR_e`, which cancels the process during Create
            // (`f_pc_create_req.cpp:106-108`) - the actor NEVER executes even once. Despawn on this
            // first tick rather than leaking a bare `Actor` entity.
            if dai.refused {
                let _ = commands.entity(entity).despawn();
                continue;
            }
        }
        dai.pos = actor.pos;
        dai.execute(player.pos, player_yaw);
    }
}