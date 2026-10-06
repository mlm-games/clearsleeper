use bevy_ecs::prelude::*;

/// The stage string is `Sitem` (`d_stage.cpp:1256`); `_arcname` is the per-item archive name
/// `getShopArcname()` returns (`d_a_shop_item.cpp:23-31`, `:99`, `:205`), which is why the stage
/// string - never an archive name - is what this array carries.
pub const SITEM_ACTORS: [&str; 1] = ["Sitem"];

/// `m_itemNo = fopAcM_GetParamBit(fopAcM_GetParam(this), 0, 8)` (`d_a_shop_item.cpp:197`), where
/// `fopAcM_GetParamBit(param, shift, bit)` is `(param >> shift) & ((1 << bit) - 1)`
/// (`f_op_actor_mng.h:131-133`).
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
/// `_create` can only return `cPhs_ERROR_e` from the two `fopAcM_entrySolidHeap` arms (`:222`,
/// `:227`) or `dComIfG_resLoad` (`:206-208`), never from a parameter, and `_execute` ends on an
/// unconditional `true` (`:152`), so there is no spawn-then-despawn path.
/// `daShopItem_IsDelete` (`:266-268`) is the bare "accepts a delete request" form of RULE 6b.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SItem {
    /// `m_itemNo` (`d_a_shop_item.h:47`), the 8-bit parameter field of `:197`.
    pub item_no: u8,
    /// `type` (`d_a_shop_item.cpp:24`, `:219`), the 4-bit field/shop model override. It selects
    /// `CheckFieldItemCreateHeap`/`dItem_data::getFieldHeapSize` when it is 1 (`:225-227`),
    /// `CheckItemCreateHeap`/`dItem_data::getHeapSize` when it is 2 (`:220-222`), and leaves the
    /// rest to `mModelType[m_itemNo]` (`d_a_shop_item_static.cpp`'s 255-entry table) through the
    /// `type == 0` arms at `:25`, `:36`, `:220` and `:225`.
    pub model_type: u8,
}

impl SItem {
    /// `_create` (`:194-241`) minus `dComIfG_resLoad`, `fopAcM_ct`, the two
    /// `dComIfG_resDelete`/cloth-phase loads, the three `fopAcM_entrySolidHeap` arms and their
    /// `dItem_data` heap sizes, `clothCreate` (`:68-111`) and `CreateInit` (`:45-65`) - archive,
    /// heap, cloth and renderer work. The two parameter decodes are the whole of the surviving
    /// state.
    ///
    /// The `getShopBmdIdx() == -1 || arcName == 0` substitute at `:200-202`, which installs
    /// `dItemNo_GREEN_RUPEE_e`, reads `dItem_data::getBmdIdx` / `getFieldBmdIdx` / `getArcname` /
    /// `getFieldArc` (`d_item_data.cpp`) - the retail item table. Every consumer of those four
    /// functions in this file is the archive load itself (`:205`, `:98`), so nothing in this sim
    /// observes the answer and no `item_no` is substituted here.
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