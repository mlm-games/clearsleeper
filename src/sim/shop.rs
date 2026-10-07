//! `daNpc_Bs1_c` — Beedle, the Bazaar shopkeeper (`Bs1`, `fpcNm_NPC_BS1_e`).

use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::{Vec2, Vec3};

use super::actor::Actor;
use super::event::{ActiveEvent, EventList};
use super::events::{SimEvent, SimEvents};
use super::flags::{Collect, EventFlags, StoryBits};
use super::intent::{DoIntent, MoveIntent, PadButtons};
use super::inventory::{Equipped, NO_ITEM, check_get_item};
use super::pickup::Rupees;
use super::player::Player;
use super::walk;
use crate::assets::MsgArchives;
use crate::text::driver::TextFlags;
use crate::text::msgbox::Msg2Box;
use crate::text::msgbox1::Msg1Box;
use crate::text::msgget::MsgArchive;
use crate::text::msgproc::{
    FPC_M_ERROR_PROCESS_ID, FPC_NM_MSG, FPC_NM_MSG2, MsgGameInfo, MsgProcs,
    fop_msgm_message_set_actor,
};
use crate::text::{
    STATUS_BOX_CLOSED, STATUS_MSG_CONTINUES, STATUS_MSG_DESTROYED, STATUS_MSG_DISPLAYED,
    STATUS_MSG_ENDS, STATUS_SELECT_2,
};

/// `OBJNAME("Bs1", fpcNm_NPC_BS1_e, 0, 59)` — `d_stage.cpp:740`. The masked
/// shopkeeper is `OBJNAME("Bs2", fpcNm_NPC_BS1_e, 1, 0)` — `d_stage.cpp:741`, a
/// second stage string on this same process, not claimed here.
pub const SHOP_ACTOR: &str = "Bs1";

const EVENT_GETDEMO: &str = "BS1_GETDEMO"; // <- dComIfGp_evmng_getEventIdx, :1566
const EVENT_GETDEMO_BS2: &str = "BS2_GETDEMO"; // <- :1572

/// `fopAcM_GetParamBit(param, 0x14, 0x4)` — `f_op_actor_mng.h:131-133`.
const TYPE_SHIFT: u32 = 0x14;
const TYPE_MASK: u32 = 0xF;
const TYPE_NORMAL: i8 = 0; // <- switch (mType) case 0, :2141
const TYPE_MASKED: i8 = 1; // <- switch (mType) case 1, :2144

/// `mShopIndex = fopAcM_GetParam(this) & 0xFF` narrowed to the `s8` at
/// `d_a_npc_bs1.h:141` — `:1588`.
const INDEX_BITS: u32 = 0xFF;
/// `cLib_minMaxLimit<int>(mShopIndex + 2, 3, 5)` — `:1591`.
const INDEX_LO: i32 = 3;
const INDEX_HI: i32 = 5;
/// `mShopIndex = 6` for the masked shopkeeper — `:1594`.
const INDEX_MASKED: i8 = 6;
/// `cLib_minMaxLimit<s8>(mShopIndex, 0, 7)` — `:1597`.
const INDEX_CLAMP_LO: i8 = 0;
const INDEX_CLAMP_HI: i8 = 7;
/// `setCamDataIdx(2)` / `setItemDataIdx(2)` on the `mShopIndex == -1` path — `:1604-1605`.
const INDEX_NONE: i8 = 2;

const SLOT_COUNT: usize = 3; // <- setItemSum(3), :1521; every loop bound is `i < 3`
const NO_SELECTED: i8 = -1; // <- mSelectedItemIdx = -1, :1536

/// `setCollision`'s cylinder — `:1186-1199`.
const CYL_RADIUS: f32 = 46.0; // <- :1193
const CYL_HEIGHT: f32 = 130.0; // <- :1194
/// `l_cyl_src` — `:59-79`. `SrcObjAt Type` is `0` (:63) so no damage ever leaves
/// this actor; `SrcObjTg Type` is `AT_TYPE_ALL` (:65) so every `ChkTgHit` arm
/// against it is live.
const SRC_TG_TYPE_ALL: u32 = 0xFFFF_FFFF; // <- :65
const SRC_TG_SPRM: u32 = 0x04 | 0x08; // <- cCcD_TgSPrm_Set_e | _IsEnemy_e, :66

/// `l_HIO.mChild[mType]` — `:92-137`; both children carry the same numbers.
const HIO_ATTN_Y_OFFSET: f32 = 55.0; // <- mAttnYOffset, :105
const HIO_MAX_ATTN_DIST_XZ: f32 = 300.0; // <- mMaxAttnDistXZ, :107

/// `daNpc_Bs1_c::ActionStatus` — `d_a_npc_bs1.h:15-19`.
const ACTION_STARTING: i8 = 0;
const ACTION_ONGOING: i8 = 1;
const ACTION_ENDING: i8 = -1;

/// `m830` / `m831`, the `wait_action` sub-state — `:1720`, `:1683`.
const MODE_WAIT: i8 = 1; // <- m830 = 1, :1720
const MODE_TALK: i8 = 2; // <- m830 = 2, :1683

/// `m82A`, the event order the actor holds — `:346-388`.
const ORDER_NONE: i8 = 0; // <- m82A = 0, :375
const ORDER_TALK: i8 = 2; // <- m82A == 2, :374
const ORDER_GETDEMO: i8 = 3; // <- m82A == 3, :347
const ORDER_SPEAK: i8 = 1; // <- m82A = 1, :1773

/// `m835`, the `talk()` sub-state — `:1356-1424`.
const TALK_INIT: i8 = 0; // <- if (m835 == 0), :1358
const TALK_SET: i8 = 1; // <- m835 = 1, :1363
const TALK_BOUND: i8 = 2; // <- m835 = 2, :1380
const TALK_SHOP: i8 = 3; // <- m835 = 3, :1414

/// `fopNpc_npc_c::talk`'s "nothing happened" return — `d_npc.cpp:665`, `:1355`.
const NO_STATUS: u8 = 0xFF;

/// `ShopItems_c__BuyConditions` — `d_shop.h:19-26`.
const BUY_MUST_OWN_ITEM: u8 = 0x01; // <- d_shop.h:20
const BUY_MUST_NOT_OWN_ITEM: u8 = 0x02; // <- d_shop.h:21
const BUY_MAXIMUM: u8 = 0x04; // <- d_shop.h:22
const BUY_ANY_BOTTLE: u8 = 0x08; // <- d_shop.h:23
const BUY_EMPTY_BOTTLE: u8 = 0x10; // <- d_shop.h:24
const BUY_RUPEES: u8 = 0x20; // <- d_shop.h:25

/// `ShopItems_c__ItemData::mMustOwnItemNo` / `mMustNotOwnItemNo` sentinel — `:36`.
pub const NO_ITEM_ID: i32 = -1;

/// `fopAcM_seStart(this, JA_SE_SHOP_BOUGHT, 0)` — `:831`; `JAZelAudio_SE.h:136`.
const SE_SHOP_BOUGHT: u32 = 0x087F;

/// `d_sv_event_flag_c` ids — `d_save_event_flag.inc`.
const BIT_BAZAAR_FIRST: u32 = 0x1F10; // <- UNK_1F10, :1115
const BIT_MASKED_RETURN: u32 = 0x1F08; // <- UNK_1F08, :1091
const BIT_BOMB_OFFER: u32 = 0x1F20; // <- UNK_1F20, :1046
const BIT_MASKED_TICKET: u32 = 0x2040; // <- UNK_2040, :1092
const BIT_BOTTLE_SOLD: u32 = 0x2020; // <- UNK_2020, :896
const BIT_HEART_SOLD: u32 = 0x2010; // <- UNK_2010, :899
const BIT_MAP_SOLD: u32 = 0x2008; // <- UNK_2008, :902
const BIT_DEFAULT_ASKED: u32 = 0x2108; // <- UNK_2108, :1251
const REG_STOCK: u16 = 0xBB07; // <- UNK_BB07, :1105
/// `UNK_86FF`, Beedle's loyalty points — `d_save_event_flag.inc:511`.
const REG_POINTS: u16 = 0x86FF;
const POINTS_GOLD: u8 = 60; // <- points >= 60, :688
const POINTS_SILVER: u8 = 30; // <- points == 30, :958
const POINTS_MAX: u8 = 0xFF; // <- cLib_maxLimit<int>(points, 0xFF), :939
/// `dLetter_send(dSv_event_flag_c::LETTER_GOLD_MEMBERSHIP)` — `:950`.
const LETTER_GOLD: u16 = 0xAF03;
/// `dLetter_send(dSv_event_flag_c::LETTER_SILVER_MEMBERSHIP)` — `:961`.
const LETTER_SILVER: u16 = 0xB003;
/// `dLetterStts_e::SEND_e` — `d_letter.cpp:11`.
const LETTER_SEND: u8 = 1;

/// `dItemNo_*` — `d_item_data.h`.
pub const ITEM_HEART_PIECE: u8 = 0x07;
const ITEM_BIRD_BAIT_5: u8 = 0x82;
const ITEM_HYOI_PEAR: u8 = 0x83; // <- :11
const ITEM_BOMB_5: u8 = 0x0B; // <- :17
const ITEM_BOMB_10: u8 = 0x0C; // <- :18
const ITEM_BOMB_20: u8 = 0x0D; // <- :19
const ITEM_BOMB_30: u8 = 0x0E; // <- :20
const ITEM_ARROW_10: u8 = 0x10; // <- :22
const ITEM_ARROW_20: u8 = 0x11; // <- :23
const ITEM_ARROW_30: u8 = 0x12; // <- :24
const ITEM_BOW: u8 = 0x27; // <- :53
const ITEM_BAIT_BAG: u8 = 0x2C; // <- :68
const ITEM_BOMB_BAG: u8 = 0x31; // <- :73
const ITEM_EMPTY_BOTTLE: u8 = 0x50; // <- :118
const ITEM_RED_POTION: u8 = 0x51; // <- :119
const ITEM_COLLECT_MAP_30: u8 = 0xE1; // <- :232
const ITEM_NONE: u8 = 0xFF; // <- dItemNo_NONE_e, :262

/// `dInvSlot` — `d_com_inf_game.h:1073-1085`.
const SLOT_BOW: usize = 12; // <- dInvSlot_BOW_e, :1084
const SLOT_BOTTLE0: usize = 14; // <- dInvSlot_BOTTLE0_e
const SLOT_BOTTLE_COUNT: usize = 4; // <- dInvSlot_BOTTLE_COUNT_e

/// `cM_initRnd(100, 100, 100)` — `m_Do_machine.cpp:617`.
const RND_SEED: i32 = 100;
const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA]; // <- cM_rnd, c_math.cpp:181
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673]; // <- c_math.cpp:181
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0]; // <- c_math.cpp:182
/// `(int)cM_rndF(4.0f) + 0x2F6E` — `:655`.
const RND_BAZAAR_SPREAD: f32 = 4.0;

/// `setWaitParm(0xF, 0xF, 0, 0, 0.9f, 0.5f, 0, 0x2000)` — `d_lib.h:17`.
const STICK_WAIT: i16 = 0x0F; // <- field_0x12
const STICK_LIMIT: i16 = 0x0F; // <- field_0x14
const STICK_DELTA: i16 = 0; // <- field_0x16
const STICK_REPEAT: i16 = 0; // <- field_0x1c
const STICK_ON: f32 = 0.9; // <- field_0x04
const STICK_OFF: f32 = 0.5; // <- field_0x08
/// `field_0x24 = 0`, so `field_0x22` is 0 on both arms — `d_lib.cpp:406-410`.
const STICK_BASE: i32 = 0;
/// `iVar6 = (0x2000 - field_0x26) >> 1` with `field_0x26 = 0x2000` — `d_lib.cpp:369`.
const STICK_HYST: i32 = 0;
/// The four direction gates — `d_lib.cpp:371-392`.
const STICK_GATE_1: i32 = 0x1000;
const STICK_GATE_3: i32 = 0x3000;
const STICK_GATE_5: i32 = 0x5000;
const STICK_GATE_7: i32 = 0x7000;
const STICK_LEFT: u8 = 1; // <- r6 |= 1, d_lib.cpp:375
const STICK_RIGHT: u8 = 2; // <- r6 |= 2, :384
const STICK_DOWN: u8 = 4; // <- r6 |= 4, :372
const STICK_UP: u8 = 8; // <- r6 |= 8, :382

/// `dShop_get_next_select` direction ids — `d_shop.cpp:1217-1226`.
const DIR_LEFT: i32 = 0;
const DIR_RIGHT: i32 = 1;
const DIR_UP: i32 = 2;
const DIR_DOWN: i32 = 3;
/// `ShopItems_c__select_list_data::mLeftIdx` and friends — `d_shop.h:36-41`.
const NEXT_NONE: i8 = -1;

/// `dComIfGp_evmng_getEventIdx` returning -1 — `:1566`.
const NO_EVENT: usize = usize::MAX;

/// `isBomb` — `d_item.cpp:2762-2768`.
fn is_bomb(item: u8) -> bool {
    matches!(item, ITEM_BOMB_5 | ITEM_BOMB_10 | ITEM_BOMB_20 | ITEM_BOMB_30)
}

/// `isArrow` — `d_item.cpp:2771-2777`.
fn is_arrow(item: u8) -> bool {
    matches!(item, ITEM_ARROW_10 | ITEM_ARROW_20 | ITEM_ARROW_30)
}

/// `ShopItems_c__ItemData` — `d_shop.h:28-33`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemData {
    pub item: u8,
    pub must_not_own: u8,
    pub must_own: i32,
    pub conditions: u8,
}

/// `__shop_items_set_data` — `d_shop.h:35-41`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entry {
    pub item: &'static ItemData,
    pub show_msg: u32,
    pub buy_msg: u32,
    pub sold_msg: u32,
    pub count: i32,
}

const fn item_data(item: u8, must_not_own: u8, must_own: i32, conditions: u8) -> ItemData {
    ItemData {
        item,
        must_not_own,
        must_own,
        conditions,
    }
}

const fn set_data(
    item: &'static ItemData,
    show_msg: u32,
    buy_msg: u32,
    sold_msg: u32,
    count: i32,
) -> Entry {
    Entry {
        item,
        show_msg,
        buy_msg,
        sold_msg,
        count,
    }
}

const FEEDBAG_DATA: ItemData =
    item_data(ITEM_BAIT_BAG, ITEM_BAIT_BAG, NO_ITEM_ID, BUY_RUPEES | BUY_MUST_NOT_OWN_ITEM); // <- d_shop.cpp:33-38
const FOOD_ALL_DATA: ItemData = item_data(
    ITEM_BIRD_BAIT_5,
    ITEM_BIRD_BAIT_5,
    ITEM_BAIT_BAG as i32,
    BUY_RUPEES | BUY_MAXIMUM | BUY_MUST_OWN_ITEM,
); // <- :40-45
const FOOD_HYOI_DATA: ItemData = item_data(
    ITEM_HYOI_PEAR,
    ITEM_HYOI_PEAR,
    ITEM_BAIT_BAG as i32,
    BUY_RUPEES | BUY_MAXIMUM | BUY_MUST_OWN_ITEM,
); // <- :47-52
const ARROW10_DATA: ItemData = item_data(
    ITEM_ARROW_10,
    ITEM_ARROW_10,
    ITEM_BOW as i32,
    BUY_RUPEES | BUY_MAXIMUM | BUY_MUST_OWN_ITEM,
); // <- :54-59
const ARROW30_DATA: ItemData = item_data(
    ITEM_ARROW_30,
    ITEM_ARROW_30,
    ITEM_BOW as i32,
    BUY_RUPEES | BUY_MAXIMUM | BUY_MUST_OWN_ITEM,
); // <- :61-66
const RED_BOTTLE_DATA: ItemData = item_data(
    ITEM_RED_POTION,
    ITEM_RED_POTION,
    ITEM_EMPTY_BOTTLE as i32,
    BUY_RUPEES | BUY_EMPTY_BOTTLE | BUY_ANY_BOTTLE,
); // <- :68-73
const BOMB30_DATA: ItemData = item_data(
    ITEM_BOMB_30,
    ITEM_BOMB_30,
    ITEM_BOMB_BAG as i32,
    BUY_RUPEES | BUY_MAXIMUM,
); // <- :103-108
const EMPTYBOTTLE_DATA: ItemData = item_data(
    ITEM_EMPTY_BOTTLE,
    ITEM_EMPTY_BOTTLE,
    ITEM_EMPTY_BOTTLE as i32,
    BUY_RUPEES,
); // <- :110-115
const HEART_DATA: ItemData =
    item_data(ITEM_HEART_PIECE, ITEM_HEART_PIECE, NO_ITEM_ID, BUY_RUPEES | BUY_MUST_NOT_OWN_ITEM); // <- :117-122
const MAP_DATA: ItemData = item_data(
    ITEM_COLLECT_MAP_30,
    ITEM_COLLECT_MAP_30,
    ITEM_COLLECT_MAP_30 as i32,
    BUY_RUPEES | BUY_MUST_NOT_OWN_ITEM,
); // <- :124-129

const FEEDBAG: Entry = set_data(&FEEDBAG_DATA, 0x0000_0F42, 0x0000_0F45, 0x0000_0F66, -1); // <- :263-269
const FOOD_ALL: Entry = set_data(&FOOD_ALL_DATA, 0x0000_0F43, 0x0000_0F46, 0x0000_0F66, 10); // <- :271-277
const FOOD_HYOI: Entry = set_data(&FOOD_HYOI_DATA, 0x0000_0F44, 0x0000_0F47, 0x0000_0F66, 10); // <- :279-285
const BOMB30_BS: Entry = set_data(&BOMB30_DATA, 0x0000_0F67, 0x0000_0F68, 0x0000_0F63, 30); // <- :287-293
const ARROW10: Entry = set_data(&ARROW10_DATA, 0x0000_0F69, 0x0000_0F6A, 0x0000_0F64, 10); // <- :295-301
const ARROW30: Entry = set_data(&ARROW30_DATA, 0x0000_0F6B, 0x0000_0F6C, 0x0000_0F64, 30); // <- :303-309
const RED_BOTTLE_BS: Entry = set_data(&RED_BOTTLE_DATA, 0x0000_0F6D, 0x0000_0F6E, 0x0000_0F65, -1); // <- :311-317
const BOMB30_BS2: Entry = set_data(&BOMB30_DATA, 0x0000_2F72, 0x0000_2F73, 0x0000_2F6B, 30); // <- :319-325
const ARROW30_BS2: Entry = set_data(&ARROW30_DATA, 0x0000_2F74, 0x0000_2F75, 0x0000_2F6B, 30); // <- :327-333
const RED_BOTTLE_BS2: Entry = set_data(&RED_BOTTLE_DATA, 0x0000_2F76, 0x0000_2F77, 0x0000_2F6B, -1); // <- :335-341
const EMPTYBOTTLE: Entry = set_data(&EMPTYBOTTLE_DATA, 0x0000_2F4A, 0x0000_2F4D, 0x0000_2F78, -1); // <- :343-349
const HEART: Entry = set_data(&HEART_DATA, 0x0000_2F4B, 0x0000_2F4E, 0x0000_2F78, -1); // <- :351-357
const MAP: Entry = set_data(&MAP_DATA, 0x0000_2F4C, 0x0000_2F4F, 0x0000_2F78, -1); // <- :359-365

/// `Item_set_data3` — `:1427-1434`; `Item_set_data4` — `:1435-1439`;
/// `Item_set_data5` — `:1440-1444`; `Item_set_dataBs2` — `:1445-1449`.
const SET3: [&Entry; SLOT_COUNT] = [&FEEDBAG, &FOOD_ALL, &FOOD_HYOI];
const SET4: [&Entry; SLOT_COUNT] = [&ARROW10, &ARROW30, &FOOD_ALL];
const SET5: [&Entry; SLOT_COUNT] = [&ARROW30, &BOMB30_BS, &RED_BOTTLE_BS];
const SET_BS2: [&Entry; SLOT_COUNT] = [&EMPTYBOTTLE, &HEART, &MAP];
/// `Item_set_data3[3]`, the `pDataSet[i + 3][0]` fallback — `:1431`, read at `:1474`.
const SET3_FEEDBAG_OVERRIDE: &Entry = &FOOD_HYOI;
/// `dComIfGs_getItem(dInvSlot_BOW_e) != dItemNo_NONE_e` — `:1491`, `:1501`.
fn has_bow(equipped: &Equipped) -> bool {
    equipped.item(SLOT_BOW) != ITEM_NONE
}

/// `dComIfGs_checkGetBottle` — `d_com_inf_game.h:1972-1978`.
fn has_bottle(equipped: &Equipped) -> bool {
    (SLOT_BOTTLE0..SLOT_BOTTLE0 + SLOT_BOTTLE_COUNT).any(|slot| equipped.item(slot) != NO_ITEM)
}

/// `dSv_player_item_c::checkEmptyBottle` — `d_save.cpp:184-194`.
fn empty_bottles(equipped: &Equipped) -> usize {
    (SLOT_BOTTLE0..SLOT_BOTTLE0 + SLOT_BOTTLE_COUNT)
        .filter(|slot| equipped.item(*slot) == ITEM_EMPTY_BOTTLE)
        .count()
}

/// `cM_rnd` — `c_math.cpp:180-186`. Retail's three lanes are file-static
/// (`c_math.cpp:168`) and shared by every caller in the process; one lane set per
/// shopkeeper, seeded at the retail boot value, keeps each roll starting where
/// retail's does. The sim has no global `cM_rndF`.
fn rnd(seed: &mut [i32; 3]) -> f32 {
    for lane in 0..3 {
        seed[lane] = (seed[lane] * RND_MUL[lane]) % RND_MOD[lane];
    }
    let sum = seed[0] as f32 / RND_SCALE[0]
        + seed[1] as f32 / RND_SCALE[1]
        + seed[2] as f32 / RND_SCALE[2];
    (sum % 1.0).abs()
}

/// `STControl` — `d_lib.h:15-46`; `init` `d_lib.cpp:311-321`, `checkTrigger`
/// `:364-428`, the four `check*Trigger` `:335-405`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Stick {
    mask: u8,
    prev: u8,
    wait_h: i16,
    wait_v: i16,
    interval_h: i16,
    interval_v: i16,
}

impl Stick {
    fn create() -> Self {
        Self {
            mask: 0,
            prev: 0,
            wait_h: 0,
            wait_v: 0,
            interval_h: STICK_WAIT,
            interval_v: STICK_WAIT,
        }
    }

    /// `CPad_GET_STICK_VALUE(0)` / `CPad_GET_STICK_ANGLE(0)` —
    /// `m_Do_controller_pad.h:41-42`.
    fn poll(&mut self, value: f32, angle: i32) {
        self.prev = self.mask;
        if value == 0.0 {
            self.mask = 0;
            self.wait_h = 0;
            self.interval_h = STICK_WAIT;
            self.wait_v = 0;
            self.interval_v = STICK_WAIT;
            return;
        }
        let a = angle;
        let base = STICK_BASE;
        let h = STICK_HYST;
        let mut dirs = 0u8;
        if a < base - STICK_GATE_7 + h {
            dirs |= STICK_DOWN;
        } else if a < base - STICK_GATE_5 - h {
            dirs |= STICK_DOWN | STICK_LEFT;
        } else if a < base - STICK_GATE_3 + h {
            dirs |= STICK_LEFT;
        } else if a < base - STICK_GATE_1 - h {
            dirs |= STICK_LEFT | STICK_UP;
        } else if a < base + STICK_GATE_1 + h {
            dirs |= STICK_UP;
        } else if a < base + STICK_GATE_3 - h {
            dirs |= STICK_UP | STICK_RIGHT;
        } else if a < base + STICK_GATE_5 + h {
            dirs |= STICK_RIGHT;
        } else if a < base + STICK_GATE_7 - h {
            dirs |= STICK_RIGHT | STICK_DOWN;
        } else {
            dirs |= STICK_DOWN;
        }
        if value >= STICK_ON {
            self.mask = dirs;
        } else if value < STICK_OFF {
            self.mask = 0;
        } else {
            self.mask &= !dirs;
        }
        if self.mask & 0x03 == 0 {
            self.wait_h = 0;
            self.interval_h = STICK_WAIT;
        }
        if self.mask & 0x0C == 0 {
            self.wait_v = 0;
            self.interval_v = STICK_WAIT;
        }
        if self.prev & self.mask & 0x03 != 0 && self.wait_h > 0 {
            self.wait_h -= 1;
        }
        if self.prev & self.mask & 0x0C != 0 && self.wait_v > 0 {
            self.wait_v -= 1;
        }
    }

    /// `STControl::checkLeftTrigger` / `checkRightTrigger` — `d_lib.cpp:335-369`.
    fn horizontal(&mut self, bit: u8) -> bool {
        self.fire(bit, true)
    }

    /// `STControl::checkUpTrigger` / `checkDownTrigger` — `d_lib.cpp:371-405`.
    fn vertical(&mut self, bit: u8) -> bool {
        self.fire(bit, false)
    }

    fn fire(&mut self, bit: u8, horiz: bool) -> bool {
        let ready = if horiz { self.wait_h } else { self.wait_v } == 0;
        if !ready || self.mask & bit == 0 {
            return false;
        }
        let interval = if horiz {
            &mut self.interval_h
        } else {
            &mut self.interval_v
        };
        let mut wait = STICK_WAIT;
        if STICK_REPEAT == 0 {
            wait = wait.saturating_sub(STICK_DELTA).max(STICK_LIMIT);
        }
        *interval = wait;
        if horiz {
            self.wait_h = wait;
        } else {
            self.wait_v = wait;
        }
        true
    }
}

/// `select_list_data_bshop_3/4/5/6` — `d_shop.cpp:654-693`. All four tables are
/// identical, and `select_list_data_tbl[dataIdx][selected + 1]` (`d_shop.cpp:1220`)
/// reads the row after the cursor, so a `-1` cursor lands on row 0.
const fn select_row(row: usize) -> [i8; 4] {
    match row {
        0 => [2, 0, NEXT_NONE, NEXT_NONE], // <- select_list_data_bshop_3[0], :654-658
        1 => [NEXT_NONE, 1, 0, 0],         // <- [1], :659-663
        2 => [0, 2, 1, 1],                 // <- [2], :664-668
        _ => [1, NEXT_NONE, 2, 2],         // <- [3], :669-673
    }
}

/// `dShop_get_next_select` — `d_shop.cpp:1217-1239`. Retail's second loop exit
/// (`mItemActorProcessIds[nextIdx] != fpcM_ERROR_PROCESS_ID_e`, `:1234`) can
/// never fire: `createShopList` fills all three ids with a live shop item.
fn next_select(selected: i8, dir: i32) -> i8 {
    let row = select_row((selected + 1).clamp(0, 3) as usize);
    let next = match dir {
        DIR_LEFT => row[0],
        DIR_RIGHT => row[1],
        DIR_UP => row[2],
        _ => row[3],
    };
    if next == selected { selected } else { next }
}

/// The action functions `setAction` selects between — `d_a_npc_bs1.h:22-40`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action {
    Wait,
    GetDemo,
}

/// The three outcomes `next_msgStatus` can return, with its `*pMsgNo` update.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    /// `fopMsgStts_MSG_CONTINUES_e` with the new message number.
    Next(u32),
    /// `fopMsgStts_MSG_DISPLAYED_e`, message number unchanged.
    Displayed,
    /// `fopMsgStts_MSG_ENDS_e`.
    Ends,
}

impl Step {
    fn status(self) -> u8 {
        match self {
            Step::Next(_) => STATUS_MSG_CONTINUES,
            Step::Displayed => STATUS_MSG_DISPLAYED,
            Step::Ends => STATUS_MSG_ENDS,
        }
    }

    fn message(self) -> Option<u32> {
        match self {
            Step::Next(msg) => Some(msg),
            _ => None,
        }
    }
}

/// `daNpc_Bs1_c` — `d_a_npc_bs1.h:80-133`.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Shop {
    kind: i8,
    index: i8,
    slots: [&'static Entry; SLOT_COUNT],
    sold_out: [bool; SLOT_COUNT],
    selected: i8,
    hidden: bool,
    state: i8,
    msg_id: u32,
    action: Action,
    phase: i8,
    mode: i8,
    mode_prev: u8,
    order: i8,
    attn: u8,
    talking: u8,
    seen: u8,
    bomb_asked: u8,
    masked_asked: u8,
    points_msg: u32,
    msg_no: u32,
    scratch: u32,
    jumped: bool,
    entered: bool,
    base: Vec3,
    attn_pos: Vec3,
    collide: Vec3,
    get_demo: usize,
    seed: [i32; 3],
    stick: Stick,
    prev_do: bool,
}

impl Shop {
    /// `daNpc_Bs1_c::_create` (`:2132-2170`) plus the parameter decode.
    pub fn spawn(parameters: u32) -> Self {
        let kind = match (parameters >> TYPE_SHIFT) & TYPE_MASK {
            1 => TYPE_MASKED, // <- :2144
            _ => TYPE_NORMAL,  // <- :2141 and the default arm, :2147
        };
        let raw = (parameters & INDEX_BITS) as u8 as i8;
        let index = if raw == -1 {
            INDEX_NONE // <- the `else` arm leaves mShopIndex at -1, :1602-1607
        } else if kind == TYPE_MASKED {
            INDEX_MASKED // <- :1594
        } else {
            clamp_i32(i32::from(raw) + 2, INDEX_LO, INDEX_HI) as i8 // <- :1591
        };
        let index = index.clamp(INDEX_CLAMP_LO, INDEX_CLAMP_HI); // <- :1597
        Self {
            kind,
            index,
            slots: SET3,
            sold_out: [false; SLOT_COUNT],
            selected: NO_SELECTED,
            hidden: false,
            state: TALK_INIT,
            msg_id: FPC_M_ERROR_PROCESS_ID,
            action: Action::Wait,
            phase: ACTION_STARTING,
            mode: MODE_WAIT,
            mode_prev: 0,
            order: ORDER_NONE,
            attn: 0,
            talking: 0,
            seen: 0,
            bomb_asked: 0,
            masked_asked: 0,
            points_msg: 0,
            msg_no: 0,
            scratch: 0,
            jumped: false,
            entered: false,
            base: Vec3::ZERO,
            attn_pos: Vec3::ZERO,
            collide: Vec3::ZERO,
            get_demo: NO_EVENT,
            seed: [RND_SEED; 3],
            stick: Stick::create(),
            prev_do: false,
        }
    }

    /// `g_profile_NPC_BS1` declares `fopAc_CULLBOX_12_e` (`:2280`), the literal
    /// `(-70, -100, -80) .. (70, 240, 100)` — `f_op_actor_mng.cpp:565`. The actor
    /// never calls `fopAcM_setCullSizeBox`, so the profile table is the box.
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (
            [-70.0, -100.0, -80.0], // <- fopAc_CULLBOX_12_e, f_op_actor_mng.cpp:565
            [70.0, 240.0, 100.0],   // <- f_op_actor_mng.cpp:565
        )
    }

    /// `l_cyl_src` (`:59-79`) placed by `setCollision` (`:1186-1199`):
    /// the centre, the radius, the height, the `SrcObjTg Type` and its SPrm mask.
    pub fn collider(&self) -> (Vec3, f32, f32, u32, u32) {
        (
            self.collide,
            CYL_RADIUS,
            CYL_HEIGHT,
            SRC_TG_TYPE_ALL,
            SRC_TG_SPRM,
        )
    }

    fn entry(&self, at: i8) -> Option<&'static Entry> {
        usize::try_from(at)
            .ok()
            .and_then(|i| self.slots.get(i).copied())
    }

    fn show_msg(&self) -> u32 {
        self.entry(self.selected).map_or(0, |e| e.show_msg)
    }

    fn buy_msg(&self) -> u32 {
        self.entry(self.selected).map_or(0, |e| e.buy_msg)
    }

    fn is_sold_out(&self, at: usize) -> bool {
        self.sold_out.get(at).copied().unwrap_or(false)
    }

    /// `ShopItems_c::isSoldOutItemAll` — `d_shop.cpp:1252-1259`.
    fn all_sold_out(&self) -> bool {
        (0..SLOT_COUNT).all(|i| self.is_sold_out(i))
    }

    /// `daNpc_Bs1_c::isSellBomb` — `:1540-1546`.
    fn sells_bomb(&self) -> bool {
        (0..SLOT_COUNT).any(|i| is_bomb(self.slots[i].item.item) && !self.is_sold_out(i))
    }

    /// `daNpc_Bs1_c::CreateInit` (`:1550-1615`).
    fn create_init(&mut self, actor: &Actor, list: &EventList) {
        self.get_demo = if self.kind == TYPE_MASKED {
            list.event_index(EVENT_GETDEMO_BS2).unwrap_or(NO_EVENT) // <- :1572
        } else {
            list.event_index(EVENT_GETDEMO).unwrap_or(NO_EVENT) // <- :1566
        };
        self.points_msg = 0; // <- m740 = 0, :1587
        self.base = actor.pos; // <- m718 = current.pos, :1579
        self.attn_pos = Vec3::new(
            actor.pos.x,
            actor.pos.y + HIO_ATTN_Y_OFFSET, // <- :1624
            actor.pos.z,
        );
        self.set_collision(actor);
        self.action = Action::Wait; // <- setAction(&wait_action, 0), :1565
        self.phase = ACTION_STARTING;
        self.entered = true;
    }

    /// `daNpc_Bs1_c::createShopList` (`:1426-1538`) minus the three
    /// `fopAcM_createShopItem` calls (`:1478`, `:1517`) and the
    /// `Item_set_pos_data_tbl` lookup they read (`:1478`, `:1517`).
    fn create_shop_list(&mut self, equipped: &Equipped, collect: &Collect, flags: &EventFlags) {
        if self.kind == TYPE_MASKED {
            // :1483-1515 — the Bazaar base list, each sold-out slot replaced once
            // in this order by the bomb / arrow / potion / bomb / arrow set.
            let mut replacement: [&Entry; 4] = [
                &BOMB30_BS2,
                &ARROW30_BS2,
                &RED_BOTTLE_BS2,
                &ARROW30_BS2,
            ];
            let mut next = 0usize;
            if check_get_item(equipped, collect, ITEM_BOMB_BAG) {
                // :1487-1489
                replacement[0] = &BOMB30_BS2;
                next = 1;
            }
            if has_bow(equipped) {
                // :1491-1493
                replacement[next] = &ARROW30_BS2;
                next += 1;
            }
            replacement[next] = &RED_BOTTLE_BS2; // :1495-1496
            next += 1;
            if !check_get_item(equipped, collect, ITEM_BOMB_BAG) {
                // :1497-1499
                replacement[next] = &BOMB30_BS2;
                next += 1;
            }
            if !has_bow(equipped) {
                // :1501-1503
                replacement[next] = &ARROW30_BS2;
            }
            let mut taken = 0usize;
            for (i, base) in SET_BS2.iter().enumerate() {
                let sold = match i {
                    0 => flags.test(BIT_BOTTLE_SOLD), // <- :1508
                    1 => flags.test(BIT_HEART_SOLD),
                    _ => flags.test(BIT_MAP_SOLD),
                };
                self.slots[i] = if sold {
                    let entry = replacement[taken.min(replacement.len() - 1)];
                    taken += 1;
                    entry
                } else {
                    *base
                };
            }
        } else {
            // :1454-1479
            let base: [&'static Entry; SLOT_COUNT] = match self.index {
                4 => SET4, // <- :1460
                5 => SET5, // <- :1463
                _ => SET3, // <- :1466, and the mShopIndex == -1 default
            };
            let mut slots = base;
            // `pDataSet[i + 3][0].mpItemData->mItemNo` — only i == 0 can reach it,
            // because only slot 0 is the bait bag; index 3 is live, index 4 is
            // `NULL` and index 5 is unreachable in this table.
            if slots[0].item.item == ITEM_BAIT_BAG
                && check_get_item(equipped, collect, ITEM_BAIT_BAG)
            {
                // :1473-1476
                slots[0] = SET3_FEEDBAG_OVERRIDE;
            }
            self.slots = slots;
        }
        for i in 0..SLOT_COUNT {
            // :1523-1534
            let item = self.slots[i].item.item;
            self.sold_out[i] = (!check_get_item(equipped, collect, ITEM_BOMB_BAG)
                && is_bomb(item))
                || (!has_bow(equipped) && is_arrow(item));
        }
        self.selected = NO_SELECTED; // <- :1536
    }

    /// `daNpc_Bs1_c::shopMsgCheck` — `:1207-1220`.
    fn shop_msg(&self, msg: u32) -> bool {
        let range = |lo: u32, hi: u32| (lo..=hi).contains(&msg);
        if self.kind == TYPE_MASKED {
            range(0x2F4A, 0x2F53) || range(0x2F6B, 0x2F78) || msg == 0x2F47 // <- :1214
        } else {
            range(0xF42, 0xF54)
                || range(0xF67, 0xF6E)
                || range(0xF63, 0xF66)
                || msg == 0xF3E // <- :1209
        }
    }

    /// `daNpc_Bs1_c::shopStickMoveMsgCheck` — `:1261-1274`.
    fn shop_stick_move_msg(&self, msg: u32) -> bool {
        let range = |lo: u32, hi: u32| (lo..=hi).contains(&msg);
        if self.kind == TYPE_MASKED {
            range(0x2F4A, 0x2F4C)
                || (range(0x2F72, 0x2F76) && msg & 1 == 0)
                || msg == 0x2F78
                || msg == 0x2F6B
                || msg == 0x2F47 // <- :1268
        } else {
            range(0xF42, 0xF44)
                || (range(0xF67, 0xF6E) && msg & 1 != 0)
                || range(0xF63, 0xF66)
                || msg == 0xF3E // <- :1263
        }
    }

    /// `daNpc_Bs1_c::checkBeastItemSellMsg` — `:1277-1300`.
    fn beast_item_msg(&self, msg: u32) -> bool {
        matches!(
            msg,
            0xF7A | 0xF7C | 0xF82 | 0xF84 | 0xF87 | 0xF89 | 0xF8C | 0xF8E | 0xF91 | 0xF93
                | 0xF96 | 0xF98 | 0xF9B | 0xF9D | 0xFA0 | 0xFD3
        )
    }

    /// `daNpc_Bs1_c::getDefaultMsg` — `:1228-1259`.
    fn default_msg(&self, points: u8, sold_out: bool, flags: &EventFlags) -> u32 {
        if self.kind == TYPE_MASKED {
            if !flags.test(BIT_MASKED_RETURN) {
                return 0x2F48; // <- :1247
            }
            if !flags.test(BIT_DEFAULT_ASKED) {
                return 0x2F54; // <- :1251
            }
            return 0x2F55; // <- :1254
        }
        if sold_out {
            return 0xF5E; // <- :1234
        }
        if points >= POINTS_GOLD {
            return 0xF60; // <- :1237
        }
        if points != 0 {
            return 0xF5F; // <- :1240
        }
        0xF3F // <- :1243
    }

    

    /// `daNpc_Bs1_c::chkAttention` — `:337-344`. Retail compares the raw Z delta
    /// against `mMaxAttnDistXZ`; it is not a distance.
    fn chk_attention(&self, player: &Player) -> u8 {
        if player.pos.z - self.base.z < HIO_MAX_ATTN_DIST_XZ {
            1 // <- :341
        } else {
            0 // <- :343
        }
    }

    /// `daNpc_Bs1_c::setAttention` — `:1618-1628`.
    fn set_attention(&mut self) {
        self.attn_pos = Vec3::new(
            self.base.x,
            self.base.y + HIO_ATTN_Y_OFFSET, // <- :1624
            self.base.z,
        );
    }

    /// `daNpc_Bs1_c::wait01` — `:1680-1690`.
    fn wait01(&mut self) {
        if self.talking != 0 {
            self.mode_prev = self.mode as u8; // <- :1682
            self.mode = MODE_TALK; // <- :1683
        } else if self.attn != 0 {
            self.order = ORDER_TALK; // <- :1685
        } else if self.order == ORDER_TALK {
            self.order = ORDER_NONE; // <- :1687
        }
    }

    /// `daNpc_Bs1_c::checkOrder`'s talk arm — `:373-384`.
    fn check_order(&mut self, speak: bool) {
        if speak && (self.order == ORDER_SPEAK || self.order == ORDER_TALK) {
            self.order = ORDER_NONE; // <- :375
            self.talking = 1; // <- :376
            self.state = TALK_INIT; // <- talkInit(), :377 / :1203
        }
    }

    /// `daNpc_Bs1_c::talk` — `:1356-1424`.
    fn talk(&mut self, ctx: &mut TalkCtx<'_>) -> u8 {
        if self.state == TALK_INIT {
            let points = ctx.stories.reg(REG_POINTS);
            self.msg_no = self.get_msg(points, ctx.flags, ctx.stories); // :1359
            self.scratch = 0; // <- :1360
            self.state = TALK_SET; // <- :1361
        }
        if self.msg_id == FPC_M_ERROR_PROCESS_ID {
            // :1367-1372 — the `dComIfGp_event_chkTalkXY()` presenting-event wait
            // is part of `dComIfGp_evmng`, which the sim has no model of.
            self.msg_id = ctx.text.message_set_actor(self.msg_no, self.attn_pos); // <- :1371
            return NO_STATUS;
        }
        let status = match self.state {
            TALK_SET => {
                if ctx.text.bound() {
                    self.state = TALK_BOUND; // <- :1380
                }
                NO_STATUS
            }
            TALK_BOUND => self.normal_talk(ctx),
            _ => self.shop_talk(ctx),
        };
        if ctx.text.send_button() {
            self.msg_no = ctx.text.current_msg(); // <- :1391
            if !self.shop_stick_move_msg(self.msg_no) {
                if !self.shop_msg(self.msg_no) {
                    self.selected = NO_SELECTED; // <- :1396
                    if self.hidden {
                        self.hidden = false; // <- showItem(), :1399
                    }
                }
                self.state = TALK_BOUND; // <- :1403
            } else {
                if self.msg_no == 0xF3E || self.msg_no == 0x2F47 {
                    self.selected = NO_SELECTED; // <- :1407
                    if self.hidden {
                        self.hidden = false; // <- showItem(), :1410
                    }
                }
                self.state = TALK_SHOP; // <- :1414
            }
        }
        status
    }

    /// `daNpc_Bs1_c::normal_talk` — `:1302-1322`.
    fn normal_talk(&mut self, ctx: &mut TalkCtx<'_>) -> u8 {
        let status = ctx.text.status();
        if status == STATUS_MSG_DISPLAYED {
            let step = self.next_msg_status(self.msg_no, ctx);
            ctx.text.set_status(step.status()); // <- :1305
            if let Some(no) = step.message() {
                self.msg_no = no;
                ctx.text.message_set_actor(no, self.attn_pos); // <- :1307
            }
        } else if status == STATUS_BOX_CLOSED {
            ctx.text.set_status(STATUS_MSG_DESTROYED); // <- :1311
        }
        status
    }

    /// `daNpc_Bs1_c::shop_talk` — `:1325-1354`.
    fn shop_talk(&mut self, ctx: &mut TalkCtx<'_>) -> u8 {
        if self.now_triggercheck(ctx) {
            self.jumped = true; // <- m708 = 1, :1330
            self.scratch = 0; // <- m73C = 0, :1331
        }
        let status = ctx.text.status();
        if status == STATUS_MSG_DISPLAYED || status == STATUS_MSG_CONTINUES {
            if self.jumped {
                self.jumped = false; // <- :1340
            } else {
                self.scratch = self.msg_no; // <- m73C = m738, :1343
                let step = self.next_msg_status(self.scratch, ctx);
                if let Some(no) = step.message() {
                    self.scratch = no;
                    ctx.text.message_set_actor(no, self.attn_pos); // <- :1344
                }
                ctx.text.set_status(step.status()); // <- :1342
            }
        } else if status == STATUS_BOX_CLOSED {
            ctx.text.set_status(STATUS_MSG_DESTROYED); // <- :1349
        }
        status
    }

    /// `dShop_now_triggercheck` — `d_shop.cpp:1262-1317`.
    fn now_triggercheck(&mut self, ctx: &mut TalkCtx<'_>) -> bool {
        let selected = self.selected;
        self.stick.poll(ctx.stick_value, ctx.stick_angle); // <- :1279
        let left = next_select(selected, DIR_LEFT);
        let right = next_select(selected, DIR_RIGHT);
        let up = next_select(selected, DIR_UP);
        let down = next_select(selected, DIR_DOWN);
        let mut next = selected;
        let mut changed = false;
        if selected != left && self.stick.horizontal(STICK_LEFT) {
            next = left;
            changed = true;
        } else if selected != right && self.stick.horizontal(STICK_RIGHT) {
            next = right;
            changed = true;
        }
        if selected != up && self.stick.vertical(STICK_UP) {
            next = up;
            changed = true;
        } else if selected != down && self.stick.vertical(STICK_DOWN) {
            next = down;
            changed = true;
        }
        if !changed {
            return false;
        }
        self.msg_no = if next < 0 {
            // `daNpc_Bs1_getDefaultMsgCB` is always non-null at the `Bs1` call
            // site (`d_shop.cpp:1350`), so the callback always wins and
            // `default_select_msg[mItemSetListGlobalIdx]` (`d_shop.cpp:21-31`,
            // read at `:1298`) is dead for this actor.
            let points = ctx.stories.reg(REG_POINTS);
            self.default_msg(points, self.all_sold_out(), ctx.flags) // :1300
        } else if self.is_sold_out(next as usize) {
            self.slots[next as usize].sold_msg // <- :1305
        } else {
            self.slots[next as usize].show_msg // <- :1307
        };
        ctx.text.set_status(STATUS_MSG_CONTINUES); // <- :1311
        let no = self.msg_no;
        ctx.text.message_set_actor(no, self.attn_pos); // <- :1312
        self.selected = next; // <- :1315
        true
    }

    /// `daNpc_Bs1_c::talk01` — `:1693-1715`.
    fn talk01(&mut self, ctx: &mut TalkCtx<'_>) {
        let status = self.talk(ctx);
        if status == STATUS_BOX_CLOSED {
            self.mode = self.mode_prev as i8; // <- m830 = m831, :1697
            self.talking = 0; // <- m731 = 0, :1699
            self.state = TALK_INIT;
        } else if status == STATUS_SELECT_2 {
            // :1704-1711 — both arms raise dComIfGp_setDoStatusForce(CHOOSE) and
            // setAStatusForce(CANCEL), which live in `Msg1Env` with no resource.
            let _ = self.shop_msg(self.msg_no) && self.buy_msg() == self.msg_no
                || self.beast_item_msg(self.msg_no);
        }
    }

    /// `daNpc_Bs1_c::getdemo_action` (`:1745-1790`), minus the item-present-demo
    /// create (`:1757-1766`), which is a separate `ShopItem` process, and minus
    /// the `dComIfGp_evmng` staff-cut calls (`:1747-1755`, `:1771`, `:1785`).
    fn getdemo(&mut self, ctx: &mut TalkCtx<'_>) -> bool {
        if self.phase == ACTION_STARTING {
            self.mode = self.mode_prev as i8; // <- m830 = m831, :1759
            self.selected = NO_SELECTED; // <- :1767
            self.phase = ACTION_ONGOING; // <- :1768
        }
        if self.phase == ACTION_ENDING || self.get_demo == NO_EVENT {
            return false;
        }
        if !ctx.active.end_check(self.get_demo) {
            return false;
        }
        self.order = ORDER_SPEAK; // <- m82A = 1, :1773
        self.points_msg = if self.kind == TYPE_MASKED {
            0x2F53 // <- :1782
        } else if ctx.stories.reg(REG_POINTS) != 0 {
            0xF4C // <- :1777
        } else {
            0xF4E // <- :1779
        };
        self.action = Action::Wait; // <- setAction(&wait_action, NULL), :1785
        true
    }

    /// `daNpc_Bs1_c::getMsg` (`:977-1183`) minus the `dComIfGp_event_chkTalkXY`
    /// arm (`:980-1040`), which needs the presenting event's item number and
    /// `isEmono`, and the beast count arms (`daNpc_Bs1_getBuyItemMax`, `:391-431`).
    fn get_msg(&mut self, points: u8, flags: &mut EventFlags, stories: &mut StoryBits) -> u32 {
        if self.points_msg != 0 {
            let msg = self.points_msg; // <- :979
            self.points_msg = 0; // <- :980
            return msg;
        }
        if self.kind == TYPE_MASKED {
            if self.all_sold_out() {
                return 0x2F62; // <- :1081
            }
            if !flags.test(BIT_BOMB_OFFER) && self.sells_bomb() {
                flags.set(BIT_BOMB_OFFER); // <- :1083
                self.bomb_asked = 1; // <- :1084
                return 0x2F64; // <- :1086
            }
            if self.bomb_asked != 0 {
                return 0x2F67; // <- :1089
            }
            if flags.test(BIT_MASKED_RETURN) {
                if flags.test(BIT_MASKED_TICKET) {
                    return if self.masked_asked == 1 { 0x2F60 } else { 0x2F61 }; // :1094, :1097
                }
                flags.set(BIT_MASKED_TICKET); // <- :1099
                self.masked_asked = 1; // <- :1101
                return 0x2F5F; // <- :1100
            }
            let stock = stories.reg(REG_STOCK) & 0x07; // <- :1105
            let msg = match stock {
                0 => {
                    if self.seen != 0 || flags.test(BIT_BAZAAR_FIRST) {
                        0x2F46 // <- :1110
                    } else {
                        flags.set(BIT_BAZAAR_FIRST); // <- :1113
                        0x2F45 // <- :1114
                    }
                }
                1 => self.seen_stock(0x2F58, 0x2F46), // <- :1121-1122
                2 => self.seen_stock(0x2F59, 0x2F46), // <- :1129-1130
                3 => self.seen_stock(0x2F5A, 0x2F46), // <- :1137-1138
                4 => self.seen_stock(0x2F5B, 0x2F46), // <- :1145-1146
                5 => self.seen_stock(0x2F5C, 0x2F46), // <- :1153-1154
                6 => self.seen_stock(0x2F5D, 0x2F69), // <- :1161-1162
                _ => self.seen_stock(0x2F5E, 0x2F6A), // <- :1169-1170
            };
            if flags.test(BIT_BAZAAR_FIRST) {
                flags.set(BIT_BAZAAR_FIRST); // <- :1172-1173
            }
            self.seen = 1; // <- :1175
            return msg;
        }
        if self.all_sold_out() {
            return 0xF3D; // <- :1044
        }
        if !flags.test(BIT_BOMB_OFFER) && self.sells_bomb() {
            flags.set(BIT_BOMB_OFFER); // <- :1046
            self.bomb_asked = 1; // <- :1047
            return 0xF55; // <- :1049
        }
        if self.bomb_asked != 0 {
            return 0xF58; // <- :1052
        }
        if points >= POINTS_GOLD {
            return self.seen_stock(0xF5C, 0xF5D); // <- :1055-1060
        }
        if points != 0 {
            return self.seen_stock(0xF5A, 0xF5B); // <- :1063-1069
        }
        if self.seen != 0 {
            return 0xF41; // <- :1072-1073
        }
        self.seen = 1; // <- :1076
        0xF3D // <- :1077
    }

    /// The `if (m836) msg = X; else { m836 = 1; msg = Y; }` shape shared by the
    /// stock arms of `getMsg` — `:1055-1171`.
    fn seen_stock(&mut self, first: u32, seen: u32) -> u32 {
        if self.seen != 0 {
            return seen;
        }
        self.seen = 1;
        first
    }

    /// `daNpc_Bs1_c::next_msgStatus` (`:444-975`).
    fn next_msg_status(&mut self, msg: u32, ctx: &mut TalkCtx<'_>) -> Step {
        let range = |lo: u32, hi: u32| (lo..=hi).contains(&msg);
        let points = ctx.stories.reg(REG_POINTS);
        let select = ctx.select_num;
        let cancel = ctx.cancel;
        let trig_b = ctx.trig_b;

        // ---- straight-through advances — :448-471
        if matches!(
            msg,
            0xF50 | 0xF53 | 0xF55 | 0xF56 | 0xF58 | 0xF6F | 0xF70 | 0xF71 | 0xF73 | 0xF79
                | 0xF81 | 0xF86 | 0xF8B | 0xF90 | 0xF95 | 0xF9A | 0xF9F | 0xFD6 | 0xFD7
                | 0x2F62 | 0x2F64 | 0x2F65 | 0x2F67
        ) {
            return Step::Next(msg + 1); // <- :471
        }
        // ---- beast purchase: `daNpc_Bs1_getBuyItemMax` (:391-431) needs
        // `dComIfGs_getBeastNum` and `dComIfGs_getRupeeMax`, so the count arm is
        // omitted and only the sold-out redirect survives — :473-504.
        if matches!(
            msg,
            0xF78 | 0xF80 | 0xF85 | 0xF8A | 0xF8F | 0xF94 | 0xF99 | 0xF9E | 0xFD4
        ) {
            return Step::Next(0xF7E); // <- :490, :503
        }
        // ---- beast quantity select — :506-525
        if range(0xF7A, 0xF7B)
            || range(0xF82, 0xF83)
            || range(0xF87, 0xF88)
            || range(0xF8C, 0xF8D)
            || range(0xF91, 0xF92)
            || range(0xF96, 0xF97)
            || range(0xF9B, 0xF9C)
            || range(0xFA0, 0xFA1)
        {
            if select == 0 && !cancel {
                // :515 — `dComIfGp_setMessageSetNumber(1)` is `Msg1Env` state with
                // no resource, so only the branch is transcribed.
                return if msg == 0xFA0 {
                    Step::Next(0xFD2) // <- :517
                } else {
                    Step::Next(msg + 1) // <- :521
                };
            }
            return Step::Next(0xF7F); // <- :525
        }
        // ---- beast pay — :527-537
        if range(0xF7B, 0xF7C)
            || range(0xF83, 0xF84)
            || range(0xF88, 0xF89)
            || range(0xF8D, 0xF8E)
            || range(0xF92, 0xF93)
            || range(0xF97, 0xF98)
            || range(0xF9C, 0xF9D)
            || msg == 0xFD2
        {
            // `daNpc_Bs1_setPayRupee(m83C, getBuyItem())` — :434-442 — needs the
            // static buy count the beast arm would have set.
            return Step::Next(msg + 1); // <- :536
        }
        // ---- beast confirm — :539-609
        if matches!(
            msg,
            0xF7C | 0xF84 | 0xF89 | 0xF8E | 0xF93 | 0xF98 | 0xF9D | 0xFD3
        ) {
            if cancel {
                return Step::Next(0xF7F); // <- :548
            }
            if select == 0 {
                // `dComIfGp_setItemRupeeCount(getPayRupee())`,
                // `setItemBeastNumCount(beastIdx, -getBuyItem())` (:578-580) and
                // the Knights'-Crest ladder (:586-601) all read the beast bag.
                return Step::Next(0xF7D); // <- :604
            }
            return Step::Next(msg - 1); // <- :608
        }
        if msg == 0xFD5 {
            return Step::Next(0xF7D); // <- :611
        }
        // ---- the Bazaar's opening lines — :613-634
        if matches!(
            msg,
            0x2F45 | 0x2F46 | 0x2F48 | 0x2F50 | 0x2F51 | 0x2F52 | 0x2F53 | 0x2F54 | 0x2F55
                | 0x2F58 | 0x2F59 | 0x2F5A | 0x2F5B | 0x2F5C | 0x2F5D | 0x2F5E | 0x2F5F
                | 0x2F60 | 0x2F61 | 0x2F66 | 0x2F68
        ) {
            return Step::Next(0x2F47); // <- :634
        }
        // ---- the Bazaar's three-item wheel — :636-665
        if msg == 0x2F6B {
            return if trig_b {
                Step::Next(0x2F49) // <- :638
            } else {
                Step::Next(0x2F6C) // <- :644
            };
        }
        if msg == 0x2F6C {
            if cancel {
                return Step::Next(0x2F6B); // <- :649
            }
            if select == 0 {
                let roll = rnd(&mut self.seed) * RND_BAZAAR_SPREAD; // <- :655
                return Step::Next((roll as u32) + 0x2F6E);
            }
            return Step::Next(0x2F6D); // <- :658
        }
        if range(0x2F6D, 0x2F71) {
            return Step::Next(0x2F6B); // <- :665
        }
        // ---- the `0xF3xx` fan-out — :667-714
        if matches!(
            msg,
            0xF3D | 0xF41 | 0xF57 | 0xF59 | 0xF5A | 0xF5B | 0xF5C | 0xF5D | 0xF5F | 0xF60
        ) {
            return Step::Next(0xF3E); // <- :677
        }
        if msg == 0xF5E {
            return Step::Next(0xF62); // <- :680
        }
        if msg == 0xF3F {
            return Step::Next(0xF3E); // <- :714
        }
        // ---- the shop cursor entry points — :682-790
        if msg == 0xF3E
            || range(0x2F47, 0x2F48)
            || range(0xF63, 0xF66)
            || msg == 0x2F78
            || range(0xF42, 0xF44)
            || matches!(msg, 0xF67 | 0xF69 | 0xF6B | 0xF6D)
            || matches!(msg, 0x2F4A | 0x2F4B | 0x2F4C | 0x2F72 | 0x2F74 | 0x2F76)
        {
            if trig_b {
                if self.all_sold_out() {
                    return Step::Next(0xF62); // <- :686, :723, :760
                }
                if points >= POINTS_GOLD {
                    return Step::Next(0xF61); // <- :691, :728, :765
                }
                return Step::Next(0xF40); // <- :695, :732, :769
            }
            // :700-702, :737-739, :774-776 — these arms raise
            // `dComIfGp_setDoStatusForce(CHOOSE)` and `setAStatusForce(CANCEL)`
            // and fall through with `fopMsgStts_MSG_DISPLAYED_e`.
            if matches!(msg, 0xF63 | 0xF64 | 0xF65 | 0xF66 | 0x2F47 | 0x2F78) {
                return Step::Displayed; // <- :702, :739, :776
            }
            return Step::Next(self.buy_msg()); // <- :775, :790
        }
        // ---- the `0xF4x`/`0xF6x` Bazaar purchase — :792-861
        if range(0xF45, 0xF47) || range(0xF68, 0xF6A) || msg == 0xF6C || msg == 0xF6E {
            return self.buy(points, ctx);
        }
        // ---- the `0x2F4x`/`0x2F7x` masked-Bazaar purchase — :863-924
        if matches!(msg, 0x2F4D | 0x2F4E | 0x2F4F | 0x2F73 | 0x2F75 | 0x2F77) {
            return self.buy(points, ctx);
        }
        // ---- the post-purchase error returns — :926-935
        if matches!(
            msg,
            0xF48 | 0xF49 | 0xF4A | 0xF4B | 0xF4D | 0xF4F | 0xF51 | 0xF52 | 0xF54
        ) {
            return Step::Next(0xF3E); // <- :935
        }
        // ---- the loyalty-point award — :937-966
        if msg == 0xF4C || msg == 0xF4E {
            let total = u8::try_from(i32::from(points) + 1)
                .unwrap_or(POINTS_MAX)
                .min(POINTS_MAX); // <- :939-940
            ctx.stories.set_reg(REG_POINTS, total);
            if total > POINTS_GOLD {
                return Step::Next(0xF3E); // <- :944
            }
            if total == POINTS_GOLD {
                ctx.stories.set_reg(LETTER_GOLD, LETTER_SEND); // <- :950
                return Step::Next(0xF53); // <- :951
            }
            if total > POINTS_SILVER {
                return Step::Next(0xF52); // <- :955
            }
            if total == POINTS_SILVER {
                ctx.stories.set_reg(LETTER_SILVER, LETTER_SEND); // <- :961
                return Step::Next(0xF50); // <- :962
            }
            return Step::Next(0xF4F); // <- :965
        }
        Step::Ends // <- :968-973
    }

    /// The two purchase arms, `:792-861` and `:863-924`: the shared
    /// `dShop_BoughtErrorStatus` check, the rupee charge, the sold-out marking
    /// and the `checkItemGet` hand-off to the demonstration.
    fn buy(&mut self, points: u8, ctx: &mut TalkCtx<'_>) -> Step {
        if ctx.cancel {
            return Step::Next(self.show_msg()); // <- :799, :869
        }
        if ctx.select_num != 0 {
            return Step::Next(self.show_msg()); // <- :859, :921
        }
        let Some(entry) = self.entry(self.selected) else {
            return Step::Next(self.show_msg());
        };
        let status = bought_error_status(
            entry,
            ctx.price,
            ctx.rupees.count,
            ctx.equipped,
            ctx.collect,
        );
        let bazaar = self.kind == TYPE_MASKED;
        let at = usize::try_from(self.selected).unwrap_or(SLOT_COUNT);
        if !bazaar {
            if status & BUY_MUST_OWN_ITEM != 0 {
                return Step::Next(0xF49); // <- :809
            }
            if status & BUY_MUST_NOT_OWN_ITEM != 0 {
                return Step::Next(0xF48); // <- :813
            }
            if status & BUY_RUPEES != 0 {
                return Step::Next(0xF4A); // <- :817
            }
            if status & BUY_MAXIMUM != 0 {
                return Step::Next(0xF4B); // <- :821
            }
            if status & BUY_EMPTY_BOTTLE != 0 {
                return Step::Next(0xF4D); // <- :825
            }
        } else {
            if status & BUY_RUPEES != 0 {
                return Step::Next(0x2F50); // <- :879
            }
            if status & BUY_MAXIMUM != 0 {
                return Step::Next(0x2F51); // <- :883
            }
            if status & BUY_EMPTY_BOTTLE != 0 {
                return Step::Next(0x2F52); // <- :887
            }
        }
        ctx.sound = Some(SE_SHOP_BOUGHT); // <- fopAcM_seStart, :831 / :903
        self.hidden = true; // <- mShopItems.hideSelectItem(), :832 / :904
        ctx.rupees.count = ctx
            .rupees
            .count
            .saturating_sub(i32::from(ctx.price).max(0) as u32); // <- :833 / :905
        let item = entry.item.item;
        if bazaar {
            if matches!(item, ITEM_EMPTY_BOTTLE | ITEM_HEART_PIECE | ITEM_COLLECT_MAP_30) {
                self.sold_out[at] = true; // <- SoldOutItem, :891
                match item {
                    ITEM_EMPTY_BOTTLE => ctx.flags.set(BIT_BOTTLE_SOLD), // <- :896
                    ITEM_HEART_PIECE => ctx.flags.set(BIT_HEART_SOLD),    // <- :899
                    _ => ctx.flags.set(BIT_MAP_SOLD),                     // <- :902
                }
            }
        } else if item == ITEM_BAIT_BAG {
            self.sold_out[at] = true; // <- :835
        }
        if !item_get_ok(item) {
            // `!checkItemGet(itemNo, FALSE)` — :839 / :916. Beedle demonstrates
            // the item rather than handing it over.
            self.order = ORDER_GETDEMO; // <- m82A = 3, :840 / :917
            return Step::Ends;
        }
        // `execItemGet(itemNo)` — :855 / :920. The arrow and bomb packs write
        // `dComIfGp_setItemArrowNumCount` / `setItemBombNumCount`, which the sim
        // has no counter for, so the grant itself is omitted.
        if !bazaar {
            return Step::Next(if points != 0 { 0xF4C } else { 0xF4E }); // <- :848-853
        }
        Step::Next(0x2F53) // <- :856, :919
    }

    /// `daNpc_Bs1_c::_execute` (`:2073-2107`) minus the renderer, `setMtx`,
    /// `mAcch.CrrPos`, `playTexPatternAnm`, `mpMorf->play` and
    /// `fopAcM_posMoveF`.
    fn execute(&mut self, ctx: &mut TalkCtx<'_>, player: Option<&Player>) {
        self.check_order(ctx.speak);
        match self.action {
            Action::Wait => self.wait_action(ctx, player),
            Action::GetDemo => {
                if self.getdemo(ctx) {
                    self.action = Action::Wait;
                    self.phase = ACTION_ONGOING;
                }
            }
        }
    }

    /// `daNpc_Bs1_c::wait_action` — `:1718-1742`, with `checkOrder` (`:362`)
    /// folded in ahead of it as `_execute` does (`:2089`).
    fn wait_action(&mut self, ctx: &mut TalkCtx<'_>, player: Option<&Player>) {
        if self.phase == ACTION_STARTING {
            self.mode = MODE_WAIT; // <- :1720
            self.talking = 0; // <- :1721
            self.phase = ACTION_ONGOING; // <- :1722
        } else if self.phase != ACTION_ENDING {
            self.attn = player.map_or(0, |p| self.chk_attention(p)); // <- :1725
            match self.mode {
                MODE_WAIT => self.wait01(),
                MODE_TALK => self.talk01(ctx),
                _ => {}
            }
            self.set_attention(); // <- :1739
        }
    }

    /// `setCollision` — `:1186-1199`. `MtxTrans(current.pos)` then
    /// `cMtx_YrotM(calc_mtx, m726.y)` (`m_Do_mtx.cpp:115-134`) applied to the
    /// local offset `(0, 0, -16)` set at `:1188`.
    fn set_collision(&mut self, actor: &Actor) {
        let yaw = walk::from_s16(actor.angle[1]); // <- m726.y = current.angle.y, :1554
        self.collide = Vec3::new(
            actor.pos.x - 16.0 * yaw.sin(), // <- :1192
            actor.pos.y,
            actor.pos.z - 16.0 * yaw.cos(),
        );
    }
}

fn clamp_i32(value: i32, lo: i32, hi: i32) -> i32 {
    if value < lo {
        lo
    } else if value > hi {
        hi
    } else {
        value
    }
}

/// `checkItemGet(itemNo, FALSE)` — `d_item.cpp:539-545` reading
/// `item_getcheck_func_ptr`. Only the arms whose backing bit the sim can read are
/// transcribed; `-1` from the table means "use the caller's default", `FALSE`
/// here. `item_getcheck_func_bird_esa_5` / `_animal_esa` read
/// `dComIfGs_isGetItemBait` and `_empty_bottle` / `_red_bottle` read
/// `dComIfGs_isGetBottleItem`; neither bitmap exists in the sim, so those arms
/// are reported omitted and read `FALSE`.
fn item_get_ok(item: u8) -> bool {
    matches!(
        item,
        ITEM_BAIT_BAG // <- item_getcheck_func_esa_bag, d_item.cpp:2205-2207
            | ITEM_ARROW_10
            | ITEM_ARROW_20
            | ITEM_ARROW_30 // <- :2110-2122
            | ITEM_BOMB_5
            | ITEM_BOMB_10
            | ITEM_BOMB_20
            | ITEM_BOMB_30 // <- :2100-2102
    )
}

/// `dShop_BoughtErrorStatus` — `d_shop.cpp:1343-1407`, called with `param_2 == 0`
/// (`:805`, `:877`), so it never charges and never grants.
fn bought_error_status(
    entry: &Entry,
    price: i16,
    rupees: u32,
    equipped: &Equipped,
    collect: &Collect,
) -> u8 {
    let cond = entry.item.conditions;
    let mut status = 0u8;
    if cond & BUY_MUST_OWN_ITEM != 0 {
        if entry.item.must_own == i32::from(ITEM_BOW) {
            if equipped.item(SLOT_BOW) == ITEM_NONE {
                status |= BUY_MUST_OWN_ITEM; // <- :1356
            }
        } else if !check_get_item(
            equipped,
            collect,
            u8::try_from(entry.item.must_own).unwrap_or(ITEM_NONE),
        ) {
            status |= BUY_MUST_OWN_ITEM; // <- :1362
        }
    }
    if cond & BUY_MUST_NOT_OWN_ITEM != 0 && check_get_item(equipped, collect, entry.item.must_not_own)
    {
        status |= BUY_MUST_NOT_OWN_ITEM; // <- :1367
    }
    if cond & BUY_ANY_BOTTLE != 0 && !has_bottle(equipped) {
        status |= BUY_ANY_BOTTLE; // <- :1372
    }
    if cond & BUY_EMPTY_BOTTLE != 0 && empty_bottles(equipped) == 0 {
        status |= BUY_EMPTY_BOTTLE; // <- :1377
    }
    // `dShop_maxCheck(itemNo, 0)` — :1381-1383 — needs
    // `dComIfGs_checkBaitItemEmpty`, `getArrowNum`/`getArrowMax` and
    // `getBombNum`/`getBombMax`; the sim carries none of them.
    if cond & BUY_RUPEES != 0 {
        let cost = if price == -1 { 0 } else { i32::from(price) }; // <- :1387-1388
        if i32::try_from(rupees).unwrap_or(i32::MAX) < cost {
            status |= BUY_RUPEES; // <- :1389
        }
    }
    status
}

/// The text-process bridge, mirroring `npc_nz.rs:1338-1399`.
pub struct Text<'a> {
    pub procs: &'a mut MsgProcs,
    pub info: &'a mut MsgGameInfo,
    pub clear_count: u8,
    pub one: &'a mut Msg1Box,
    pub two: &'a mut Msg2Box,
    pub send: u8,
    pub archive: Option<&'a dyn MsgArchive>,
}

impl Text<'_> {
    fn proc_name(&self) -> Option<u16> {
        if self.procs.search_by_name(FPC_NM_MSG).is_some() {
            Some(FPC_NM_MSG)
        } else if self.procs.search_by_name(FPC_NM_MSG2).is_some() {
            Some(FPC_NM_MSG2)
        } else {
            None
        }
    }

    /// `l_msg->mStatus` — `:1303`.
    fn status(&self) -> u8 {
        match self.proc_name() {
            Some(FPC_NM_MSG) => self.one.status,
            Some(FPC_NM_MSG2) => self.two.status,
            _ => NO_STATUS,
        }
    }

    fn set_status(&mut self, status: u8) {
        match self.proc_name() {
            Some(FPC_NM_MSG) => self.one.status = status,
            Some(FPC_NM_MSG2) => self.two.status = status,
            _ => {}
        }
    }

    /// `fopMsgM_SearchByID(l_msgId)` — `:1378`.
    fn bound(&self) -> bool {
        self.procs.search_by_name(FPC_NM_MSG).is_some()
            || self.procs.search_by_name(FPC_NM_MSG2).is_some()
    }

    /// `l_msg->mMsgNo` — `:1391`.
    fn current_msg(&self) -> u32 {
        match self.proc_name() {
            Some(FPC_NM_MSG) => self.one.msg_no,
            Some(FPC_NM_MSG2) => self.two.msg_no,
            _ => 0,
        }
    }

    /// `l_msg->mSelectNum` — `:1345`.
    fn select_num(&self) -> u8 {
        self.one.select_num
    }

    /// `dComIfGp_checkMesgCancelButton` — `d_com_inf_game.h:2871-2873`. The crate
    /// has no `mMesgCancelButton` resource; `Msg1Box::m1169` is the same latch,
    /// raised by the same `CPad_CHECK_TRIG_B(0)` press (`d_msg.cpp:2258-2259`).
    fn cancel_button(&self) -> bool {
        self.one.m1169 != 0
    }

    /// `dComIfGp_getMessageRupee` — `d_com_inf_game.h:2806-2808`.
    fn price(&self) -> i16 {
        self.info.message_rupee
    }

    /// `dComIfGp_checkMesgSendButton` — `:1390`.
    fn send_button(&self) -> bool {
        self.send != 0
    }

    fn message_set_actor(&mut self, msg_no: u32, eye: Vec3) -> u32 {
        let Some(archive) = self.archive else {
            return FPC_M_ERROR_PROCESS_ID;
        };
        fop_msgm_message_set_actor(
            self.procs,
            self.info,
            archive,
            self.clear_count,
            msg_no,
            None,
            eye,
        )
    }
}

/// Everything `next_msgStatus` and `talk` write through.
pub struct TalkCtx<'a> {
    pub text: &'a mut Text<'a>,
    pub stories: &'a mut StoryBits,
    pub flags: &'a mut EventFlags,
    pub active: &'a ActiveEvent,
    pub select_num: u8,
    pub cancel: bool,
    pub trig_b: bool,
    pub stick_value: f32,
    pub stick_angle: i32,
    pub price: i16,
    pub speak: bool,
    pub sound: Option<u32>,
    pub rupees: &'a mut Rupees,
    pub equipped: &'a Equipped,
    pub collect: &'a Collect,
}

#[derive(SystemParam)]
pub struct World<'w> {
    pub text_flags: Res<'w, TextFlags>,
    pub rupees: ResMut<'w, Rupees>,
    pub equipped: Res<'w, Equipped>,
    pub collect: Res<'w, Collect>,
    pub stories: ResMut<'w, StoryBits>,
    pub flags: ResMut<'w, EventFlags>,
    pub list: Res<'w, EventList>,
    pub active: ResMut<'w, ActiveEvent>,
    pub archive: Option<Res<'w, MsgArchives>>,
    pub events: ResMut<'w, SimEvents>,
}

#[derive(SystemParam)]
pub struct Boxes<'w, 's> {
    pub procs: ResMut<'w, MsgProcs>,
    pub info: ResMut<'w, MsgGameInfo>,
    pub one: ResMut<'w, Msg1Box>,
    pub two: ResMut<'w, Msg2Box>,
    pub players: Query<'w, 's, &'static Player>,
}

/// `CPad_GET_STICK_ANGLE(0)` — `m_Do_controller_pad.h:42`, via the house
/// conversion at `src/sim/carry.rs:182`.
fn stick_angle(axis: Vec2) -> i32 {
    if axis == Vec2::ZERO {
        0
    } else {
        i32::from(walk::to_s16(axis.x.atan2(axis.y)))
    }
}

pub fn control(
    mut world: World,
    mut boxes: Boxes,
    intent: Res<MoveIntent>,
    do_intent: Res<DoIntent>,
    pad: Option<Res<PadButtons>>,
    mut shops: Query<(Entity, &Actor, &mut Shop)>,
) {
    let clear_count = world.text_flags.clear_count;
    let send = boxes.one.m116a;
    let archive = world
        .archive
        .as_deref()
        .map(|held| held as &dyn MsgArchive);
    let player = boxes.players.iter().next().copied();
    let stick_value = intent.axis.length().min(1.0);
    let stick_angle = stick_angle(intent.axis);
    let trig_b = pad.map_or(false, |p| p.trig.b);
    let do_now = do_intent.0;
    for (entity, actor, mut shop) in &mut shops {
        if !shop.entered {
            shop.create_init(&actor, &world.list);
            shop.create_shop_list(&world.equipped, &world.collect, &world.flags);
        }
        let mut text = Text {
            procs: &mut boxes.procs,
            info: &mut boxes.info,
            clear_count,
            one: &mut boxes.one,
            two: &mut boxes.two,
            send,
            archive,
        };
        let select_num = text.select_num();
        let cancel = text.cancel_button();
        let price = text.price();
        let speak = do_now && !shop.prev_do && shop.attn != 0;
        shop.prev_do = do_now;
        let mut ctx = TalkCtx {
            text: &mut text,
            stories: &mut world.stories,
            flags: &mut world.flags,
            active: &world.active,
            select_num,
            cancel,
            trig_b,
            stick_value,
            stick_angle,
            price,
            speak,
            sound: None,
            rupees: &mut world.rupees,
            equipped: &world.equipped,
            collect: &world.collect,
        };
        shop.execute(&mut ctx, player.as_ref());
        let sound = ctx.sound;
        // `fopAcM_orderOtherEventId(this, m83A)` — :348.
        if shop.order == ORDER_GETDEMO
            && shop.get_demo != NO_EVENT
            && world.active.stage_of(shop.get_demo).is_none()
        {
            world.active.order(&world.list, shop.get_demo, entity);
        }
        if let Some(se) = sound {
            world.events.list.push(SimEvent::Se(se));
        }
    }
}