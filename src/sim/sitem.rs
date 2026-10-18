//! WHICH OF THE TWO `"Sitem"` OBJNAME ROWS THIS MODULE PORTS
//!
//! `d_stage.cpp` spells the stage string `"Sitem"` twice, against two different processes:
//!   * `d_stage.cpp:920` `OBJNAME("Sitem", fpcNm_SITEM_e, 255, 0)`   -> `d_a_sitem.cpp`
//!   * `d_stage.cpp:1256` `OBJNAME("Sitem", fpcNm_ShopItem_e, 255, 0)` -> `d_a_shop_item.cpp`
//!
//! `spawn_actor` is ONE name-keyed if/else chain and `dStage_objectNameInf` is
//! `{char name[8]; s16 procname; s8 argument; s8 gbaName;}` (`include/d/d_stage.h:989-994`), so the
//! two rows are indistinguishable by name: whichever half this module claims, the other is
//! unreachable. This module therefore ports `fpcNm_ShopItem_e` (`d_a_shop_item.cpp`,
//! `grep -c Nonmatching` = 0, fully portable) and DECLINES `fpcNm_SITEM_e` under RULE 1-STUB.
//!
//! Why ShopItem and not the `bgn.rs`/`fpcNm_ATT_e` treatment (no stage claim at all):
//! `fpcNm_ATT_e` has no OBJNAME row, so `bgn.rs` correctly declares none. `fpcNm_ShopItem_e` DOES
//! have one (`:1256`), so it is genuinely stage-placed and keeps the claim. The runtime creation
//! path is additional, not exclusive: `fopAcM_createShopItem` (`src/f_op/f_op_actor_mng.cpp:923-929`)
//! and `d_a_auction.cpp:668` both call `fopAcM_create(fpcNm_ShopItem_e, ...)`, and
//! `src/sim/auction.rs:1107` already spawns its auction item with `SITEM_ACTORS[0]` for exactly that
//! call - so the array is load-bearing and cannot be withdrawn.
//!
//! WHY `fpcNm_SITEM_e` IS DECLINED - the complete inventory of real tokens in `d_a_sitem.cpp`
//! (169 lines, `grep -c Nonmatching` = 15). Every behaviour body is the literal marker:
//! `hand_draw` (`:11-13`), `daSitem_Draw` (`:16-18`), `hand_mtx_set` (`:21-23`), `control3`
//! (`:26-28`), `control1` (`:31-33`), `control2` (`:36-38`), `cut_control1` (`:41-43`), `my_break`
//! (`:46-48`), `cut_control2` (`:51-53`), `hand_move` (`:56-58`), `daSitem_Execute` (`:61-63`),
//! `daSitem_Delete` (`:71-73`), `useHeapInit` (`:76-78`), `daSitem_solidHeapCB` (`:81-83`) and
//! `daSitem_Create` (`:86-144`). The ONLY things that exist are:
//!   * `daSitem_IsDelete` (`:66-68`), the bare `return TRUE` of RULE 6b;
//!   * two `static dCcD_SrcSph` tables inside the stubbed `daSitem_Create`: `tg_sph_src`
//!     (`:88-115`, radius 20.0, `SrcObjAt Type = AT_TYPE_UNK800` with `Set|VsPlayer`,
//!     `SrcObjTg Type` all-but-boomerang/water/unk20000/wind/unk400000/light with `Set|IsEnemy`,
//!     `SrcObjCo SPrm = Set|IsPlayer|VsGrpAll`) and `bm_sph_src` (`:116-143`, radius 80.0,
//!     `SrcObjAt Type = 0`, `SrcObjTg Type = AT_TYPE_BOOMERANG | AT_TYPE_WIND`);
//!   * the five dispatch thunks `Mthd_*` (`:763-785`) and `l_daSitem_Method` (`:146-152`);
//!   * `g_profile_SITEM` (`:154-169`): `List ID 0x0007`, `List Prio fpcPi_CURRENT_e`,
//!     `Proc Name fpcNm_SITEM_e`, `Size sizeof(sitem_class)`, `Size Other 0`, `Parameters 0`,
//!     `Status fopAcStts_UNK4000_e | fopAcStts_UNK40000_e`, `Group fopAc_ENEMY_e`,
//!     `Cull Type fopAc_CULLBOX_CUSTOM_e`.
//! There is NO mode enum, NO timer, NO branch table and NO parameter decode anywhere in that file, so
//! every value a port would need would be invented. The function NAMES (`hand_*`, `control1/2/3`,
//! `my_break`, `cut_control1/2`) say it is a hand-carried item with cut-scene behaviour; they say
//! nothing about what any of it does.
//!
//! `d_a_itembase.cpp` (251 lines, `Nonmatching` = 0) is `daShopItem_c`'s base class -
//! `class daShopItem_c : public daItemBase_c` (`include/d/actor/d_a_shop_item.h:20`) - but all
//! fourteen of its functions (`DeleteBase`, `clothCreate`, `CreateItemHeap`, `DrawBase`,
//! `setListStart`, `setListEnd`, `settingBeforeDraw`, `setTevStr`, `setShadow`, `animEntry`,
//! `animPlay`) are model, animation, draw-list, shadow and tev work, so the shared base contributes
//! no behaviour to merge here.

use bevy_ecs::prelude::*;

/// `d_stage.cpp:1256` binds `"Sitem"` to `fpcNm_ShopItem_e`. `_arcname` is the per-item archive name
/// `getShopArcname()` returns (`d_a_shop_item.cpp:23-31`, `:99`, `:204`, `:250`), which is why the
/// stage string - never an archive name - is what this array carries.
pub const SITEM_ACTORS: [&str; 1] = ["Sitem"];

/// `m_itemNo = fopAcM_GetParamBit(fopAcM_GetParam(this), 0, 8)` (`d_a_shop_item.cpp:197`), where
/// `fopAcM_GetParamBit(param, shift, bit)` is `(param >> shift) & ((1 << bit) - 1)`
/// (`include/f_op/f_op_actor_mng.h:131-133`).
const ITEM_SHIFT: u32 = 0; // <- d_a_shop_item.cpp:197
const ITEM_WIDTH: u32 = 8; // <- d_a_shop_item.cpp:197
/// `u8 type = fopAcM_GetParamBit(fopAcM_GetParam(this), 8, 4)` (`d_a_shop_item.cpp:24`, `:35`,
/// `:219`) - the field/shop model override.
const TYPE_SHIFT: u32 = 8; // <- d_a_shop_item.cpp:24
const TYPE_WIDTH: u32 = 4; // <- d_a_shop_item.cpp:24
/// `fopAcM_setCullSizeBox(this, -100.0f, 0.0f, -100.0f, 100.0f, 200.0f, 100.0f)`
/// (`d_a_shop_item.cpp:47`) - a real custom cull box, so this profile's
/// `fopAc_CULLBOX_CUSTOM_e` (`:292`) resolves.
const CULL_MIN: [f32; 3] = [-100.0, 0.0, -100.0]; // <- d_a_shop_item.cpp:47
/// `fopAcM_setCullSizeBox(this, ..., 100.0f, 200.0f, 100.0f)` (`d_a_shop_item.cpp:47`).
const CULL_MAX: [f32; 3] = [100.0, 200.0, 100.0]; // <- d_a_shop_item.cpp:47

/// A shop item display model - `daShopItem_c` (`d_a_shop_item.h:14-51`).
///
/// `daShopItem_c` registers no `dCcD_Src*` table anywhere: `_create` (`:194-241`) and
/// `CreateInit` (`:45-65`) touch only `mpModel`, `field_0x64C`, `tevStr`, `scale`, `mTevType` and
/// the cloth packet. There is no AT and no Tg collider, so nothing damages it and it damages
/// nothing, and `daShopItem_c_m_data` carries no health. The profile `Group` is `fopAc_ACTOR_e`
/// (`:291`).
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. Neither `combat::HitPoints` nor
/// `enemy::Hurt` is attached.
///
/// `_execute` (`:148-153`) is exactly `animPlay(1.0f, 1.0f, 1.0f, 1.0f, 1.0f)` followed by
/// `set_mtx()` (`:114-145`), and both are model/matrix work with no sim-side effect - every other
/// field (`mTevType`, `field_0x644`, `field_0x648`, `field_0x64C`, `scale`, `home`) is read only
/// by `set_mtx`, `_draw`, `settingBeforeDraw` or `setTevStr`. That is why this module has no
/// `control` system: there is no per-tick behaviour to run, and an empty system would only spend
/// a slot in a `.chain()` group that is capped at 20.
///
/// `_create` can return `cPhs_ERROR_e` from the two `fopAcM_entrySolidHeap` arms (`:221-222`,
/// `:226-227`, `:231-232`), from the item archive load (`:205-208`) or from the cloth archive load
/// (`:212`, `:215-217`), never from a parameter, and `_execute` ends on an unconditional `true`
/// (`:152`), so there is no spawn-then-despawn path.
/// `daShopItem_IsDelete` (`:266-268`) is the bare "accepts a delete request" form of RULE 6b.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SItem {
    /// `m_itemNo` (`d_a_shop_item.h:47`, `d_a_itembase.h:88`), the 8-bit parameter field of `:197`.
    pub item_no: u8,
    /// `type` (`d_a_shop_item.cpp:24`, `:219`), the 4-bit field/shop model override. It selects
    /// `CheckFieldItemCreateHeap` / `dItem_data::getFieldHeapSize` when it is 1 (`:225-227`),
    /// `CheckItemCreateHeap` / `dItem_data::getHeapSize` when it is 2 (`:220-222`), and leaves the
    /// rest to `mModelType[m_itemNo]` (`src/d/d_a_shop_item_static.cpp:267`, a 255-entry table that
    /// is 2 almost everywhere) through the `type == 0` arms at `:25`, `:36`, `:220` and `:225`.
    pub model_type: u8,
}

impl SItem {
    /// `_create` (`:194-241`) minus `dComIfG_resLoad`, `fopAcM_ct`, the two `dComIfG_resDelete` /
    /// cloth-phase loads, the three `fopAcM_entrySolidHeap` arms and their `dItem_data` heap sizes,
    /// `clothCreate` (`:68-111`) and `CreateInit` (`:45-65`) - archive, heap, cloth and renderer
    /// work. The two parameter decodes are the whole of the surviving state.
    ///
    /// The `getShopBmdIdx() == -1 || arcName == 0` substitute at `:200-202`, which installs
    /// `dItemNo_GREEN_RUPEE_e`, reads `dItem_data::getBmdIdx` / `getFieldBmdIdx` / `getArcname` /
    /// `getFieldArc` (`src/d/d_item_data.cpp`) - the retail item table. Every consumer of those four
    /// functions in this file is an archive name or index that only the loads and the delete path
    /// touch (`:98`, `:200`, `:205`, `:250`), so nothing in this sim observes the answer and no
    /// `item_no` is substituted here.
    pub fn spawn(parameters: u32) -> Self {
        Self {
            item_no: ((parameters >> ITEM_SHIFT) & ((1 << ITEM_WIDTH) - 1)) as u8,
            model_type: ((parameters >> TYPE_SHIFT) & ((1 << TYPE_WIDTH) - 1)) as u8,
        }
    }

    /// `fopAcM_setCullSizeBox(this, -100.0f, 0.0f, -100.0f, 100.0f, 200.0f, 100.0f)`
    /// (`d_a_shop_item.cpp:47`) - the actor's one real custom cull box.
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

pub fn is_sitem(name: &str) -> bool {
    SITEM_ACTORS.contains(&name)
}
