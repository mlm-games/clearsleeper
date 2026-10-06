use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::bombflower::{Blasts, BLAST_RADIUS};
use super::carry::Carry;
use super::collision::Collision;
use super::combat::Combat;
use super::events::{SimEvent, SimEvents};
use super::flags::Switches;
use super::ground;
use super::hammer;
use super::pickup::{ITEM_ACTORS, Pickup};
use super::player::Player;
use super::sea::Sea;
use super::stream;
use super::vibration::Vibration;
use super::walk;

/// `d_stage.cpp:532` `OBJNAME("koisi1", fpcNm_STONE_e, 0, 0)` and `:533`
/// `OBJNAME("koisi2", fpcNm_STONE_e, 1, 0)`; `c_dylink.cpp:442` maps `fpcNm_STONE_e` to
/// `d_a_stone`. This is the `M_arcname_Always` pair of `M_data` rows and NOT the `fpcNm_Stone2_e`
/// trio of `d_stage.cpp:654-656` that `stone.rs` claims.
///
/// The two names differ only in `dStage_objectNameInf::argument` (`d_stage.h:993`), which retail
/// copies into `fopAc_ac_c::argument` (`f_op_actor.cpp:276`) for `dStage_getName` and the event
/// manager's object search (`d_event_manager.cpp:187`, `f_op_actor_mng.cpp:1421`). `d_a_stone.cpp`
/// never reads it - its row selector is `prm_get_type()`, three parameter bits at shift 24
/// (`d_a_stone.h:84-85`, `:110`) - and `src/dzs.rs`'s `StageActor` carries no `argument` field, so
/// no argument is stored here: an unread value is dead code.
pub const KOISI_ACTORS: [&str; 2] = ["koisi1", "koisi2"];

// -- parameter decode. Every accessor is `daObj::PrmAbstract(this, W, S)` = `(param >> S) & ((1 << W) - 1)`
//    (`include/d/d_a_obj.h:10-14`) reached through the `Prm_e` shifts at `d_a_stone.h:83-101`.
const ITEM_NO_SHIFT: u32 = 0; // <- d_a_stone.h:94 (PRM_ITEMNO_S)
const ITEM_NO_WIDTH: u32 = 6; // <- d_a_stone.h:93 (PRM_ITEMNO_W)
const MOVE_BG_SHIFT: u32 = 6; // <- d_a_stone.h:88 (PRM_MOVEBG_S)
const MOVE_BG_WIDTH: u32 = 2; // <- d_a_stone.h:87 (PRM_MOVEBG_W)
const SWITCH_SHIFT: u32 = 8; // <- d_a_stone.h:100 (PRM_SWSAVE_S)
const SWITCH_WIDTH: u32 = 8; // <- d_a_stone.h:99 (PRM_SWSAVE_W)
const ITEM_SAVE_SHIFT: u32 = 16; // <- d_a_stone.h:97 (PRM_ITEMSAVE_S)
const ITEM_SAVE_WIDTH: u32 = 7; // <- d_a_stone.h:96 (PRM_ITEMSAVE_W)
const KIND_SHIFT: u32 = 24; // <- d_a_stone.h:85 (PRM_TYPE_S)
const KIND_WIDTH: u32 = 3; // <- d_a_stone.h:84 (PRM_TYPE_W)
const CULL_SHIFT: u32 = 28; // <- d_a_stone.h:91 (PRM_CULL_S)
const CULL_WIDTH: u32 = 3; // <- d_a_stone.h:90 (PRM_CULL_W)

/// `daStone::DataFlag_e` (`d_a_stone.h:13-20`), the `data().m70` bits.
const DATAFLG_0: u32 = 0; // <- d_a_stone.h:14
const DATAFLG_1: u32 = 1 << 0; // <- d_a_stone.h:15
const DATAFLG_2: u32 = 1 << 1; // <- d_a_stone.h:16
const DATAFLG_4: u32 = 1 << 2; // <- d_a_stone.h:17
const DATAFLG_8: u32 = 1 << 3; // <- d_a_stone.h:18
const DATAFLG_10: u32 = 1 << 4; // <- d_a_stone.h:19

/// `prm_get_swSave() == 0xff` answers `chk_appear` outright (`:178`) and is the `!= 0xFF` guard of the
/// `damaged` latch (`:502`). Only `prm_get_swSave()`'s eight bits can reach it.
const NO_SWITCH: u8 = 0xFF; // <- d_a_stone.cpp:178, :502

/// `if (m670 != 3 && hitObj->ChkAtType(AT_TYPE_BOMB))` (`:520`) singles out `M_data[3]`, the one row
/// a bomb cannot break.
const BOMB_EXEMPT: u8 = 3; // <- d_a_stone.cpp:520

/// `damaged(int arg1)`'s `arg1` is the `action` argument of `fopAcM_createItemFromTable` (`:489`), a
/// `daItemAct_e` value - NOT a hit power.
const ITEM_ACT_2: u8 = 2; // <- d_a_stone.cpp:563, :579 (lava), :604 (the m38 fall), :616 (dropped)
const ITEM_ACT_3: u8 = 3; // <- d_a_stone.cpp:514 (`damaged(3)` on `mCyl.ChkAtHit()`), :619 (wall/roof)
const ITEM_ACT_7: u8 = 7; // <- d_a_stone.cpp:521 (bomb Tg hit), :524 (skull-hammer Tg hit)

/// `MAKE_ITEM_PARAMS(itemNo, i_itemBitNo, switchNo2, type, action)` packs
/// `itemNo << 0 | i_itemBitNo << 8 | switchNo2 << 16 | type << 24 | action << 26`
/// (`src/f_op/f_op_actor_mng.cpp:31-32`). `fopAcM_fastCreateItem2` supplies `switchNo2 = -1` and
/// `type = 0` (`:982`) and forwards the `action` it was handed (`:983`).
const ACTION_SHIFT: u32 = 26; // <- f_op_actor_mng.cpp:32
const ACTION_MASK: u32 = 0x3F; // <- f_op_actor_mng.cpp:32
const ITEM_SAVE_PACK_SHIFT: u32 = 8; // <- f_op_actor_mng.cpp:32 (the `(i_itemBitNo & 0xFF) << 8` term)
const ITEM_SAVE_PARAM_MASK: u32 = 0xFF; // <- f_op_actor_mng.cpp:32
const SWITCH_NO_SLOT: u32 = 0xFF; // <- f_op_actor_mng.cpp:982 (`int switchNo2 = -1;`)
const SWITCH_NO_PACK_SHIFT: u32 = 16; // <- f_op_actor_mng.cpp:32

/// `i_itemNo == 0x3f || i_itemNo == 0xff` creates nothing (`src/f_op/f_op_actor_mng.cpp:892`).
/// `prm_get_itemNo()` is six bits wide, so only `0x3f` is reachable from a stage parameter.
const NO_ITEM: u8 = 0x3F; // <- f_op_actor_mng.cpp:892

/// `actor->maxFallSpeed = -100.0f` (`src/f_op/f_op_actor.cpp:286`), the clamp `posMoveF_grade`
/// applies to `speed.y` (`src/d/d_a_obj.cpp:159-161`). `d_a_stone.cpp` never writes it.
const MAX_FALL: f32 = -100.0; // <- f_op_actor.cpp:286

/// `dComIfGp_getVibration().StartShock(data().m40, -0x21, cXyz(0.0f, 1.0f, 0.0f))` (`:494`).
const SHOCK_FLAGS: i32 = -0x21; // <- d_a_stone.cpp:494
const SHOCK_DIR: Vec3 = Vec3::new(0.0, 1.0, 0.0); // <- d_a_stone.cpp:494

/// `water_tention` (`:844-848`) damps the water-height delta to `0.8` on the way down and `0.2` on
/// the way up.
const TENSION_SINK: f32 = 0.8; // <- d_a_stone.cpp:845
const TENSION_RISE: f32 = 0.2; // <- d_a_stone.cpp:847

/// `m4F4.y = -G_CM3D_F_INF;` (`bg_crr_water`, `:793`) - the "this step the rock is dry" sentinel
/// that `water_tention` tests twice (`:841`, `:842`) before applying any tension.
const NO_WATER: f32 = f32::NEG_INFINITY; // <- d_a_stone.cpp:793

/// The sword reach is the crate-wide `combat::control` window rather than a value from this TU: the
/// player is the only AT attacker whose hits this crate resolves. `stone::control` reads the same
/// four, off an identical `dCcD_SrcCyl`.
const CUT_START: u32 = 1; // <- src/sim/combat.rs (DAMAGE_START)
const CUT_END: u32 = 6; // <- src/sim/combat.rs (DAMAGE_END)
const SWORD_OFFSET: f32 = 20.0; // <- src/sim/combat.rs (SWORD_OFFSET)
const SWORD_REACH: f32 = 20.0; // <- src/sim/combat.rs (SWORD_RADIUS)

/// One row of `daStone::Data_c` (`d_a_stone.h:22-80`), reduced to the fields the simulation reads.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Attr {
    /// `m00` - `fopAcM_SetGravity(this, data().m00)` (`:228`) and `gravity = data().m00`
    /// (`:288`, `:367`).
    pub gravity: f32,
    /// `m08` - `fVar2 = current.pos.y + data().m08` in `bg_crr_water` (`:769`) and
    /// `attention_info.position.y = current.pos.y + data().m08` (`:255`).
    pub reach: f32,
    /// `m14` - `fopAcM_GetSpeed(this).y = data().m14`, the toss launch speed (`:365`).
    pub lift: f32,
    /// `m18` - the base of `data().m18 + player->getSpeedF() * data().m1C` (`:351`).
    pub toss: f32,
    /// `m1C` - the player's-speed gain on that toss (`:351`).
    pub toss_gain: f32,
    /// `m20` - `posMoveF_stream`'s linear resist coefficient in `mode_wait` (`:299`) and
    /// `mode_drop` (`:375`).
    pub damp: f32,
    /// `m24` - `posMoveF_stream`'s quadratic resist coefficient in the same two calls.
    pub inert: f32,
    /// `m38` - the drop `m688 - current.pos.y > data().m38` that shatters a waiting rock on the
    /// landing tick (`:603`).
    pub fall_limit: f32,
    /// `m40` - the `StartShock` pattern index, guarded by `if (data().m40 > 0)` (`:493`).
    pub shock: i16,
    /// `m44` - added to the gravity by `mode_sink_init`, `data().m00 + data().m44` (`:390`).
    pub sink_gravity: f32,
    /// `m48` - `mode_sink`'s `posMoveF_stream` linear resist (`:409`).
    pub sink_damp: f32,
    /// `m4C` - `mode_sink`'s `posMoveF_stream` quadratic resist (`:409`).
    pub sink_inert: f32,
    /// `m58` - `if (sqrt > data().m58) { sqrt = data().m58 / sqrt; speed *= sqrt; speedF *= sqrt; }`
    /// in `mode_sink_init` (`:393-397`).
    pub speed_cap: f32,
    /// `m70` - `daStone::DataFlag_e`, tested at `:177` and `:500` (`& 8`), `:251` (`& 2`) and `:559`
    /// and `:576` (`& 1`).
    pub flags: u32,
    /// `m82` - `mCyl.SetR(data().m82)` (`:224`) and `mAcchCir.SetWall(30.0f, data().m82)` (`:142`).
    pub radius: f32,
    /// `m8C` - `fopAcM_seStart(this, data().m8C, 0)`, the shatter sound (`:498`).
    pub break_se: u32,
    /// `m94` - `fopAcM_seStart(this, data().m94, uVar4)`, the water entry sound (`:883`).
    pub water_se: u32,
    /// `m9C` - `daObj::HitSeStart(&eyePos, current.roomNo, &mCyl, data().m9C)` (`:530`), the clink of
    /// a Tg hit that breaks nothing.
    pub hit_se: u32,
}

/// `M_data[0]` (`d_a_stone.cpp:62-75`) - the small round rock: gravity `-6.0f`, reach `22.0f`, toss
/// lift `27.0f` over a base speed of `36.0f`, collider radius `0x1E`, `DATAFLG_0_e`, shock pattern
/// `0`, break `0x692B`, water `0x6918`.
pub const ALWAYS_SMALL: Attr = Attr {
    gravity: -6.0,
    reach: 22.0,
    lift: 27.0,
    toss: 36.0,
    toss_gain: 0.0,
    damp: 0.0,
    inert: 0.0,
    fall_limit: 200.0,
    shock: 0,
    sink_gravity: 5.4,
    sink_damp: 0.04,
    sink_inert: 0.014,
    speed_cap: 30.0,
    flags: DATAFLG_0,
    radius: 30.0,
    break_se: 0x692B,
    water_se: 0x6918,
    hit_se: 0xD,
};

/// `M_data[1]` (`d_a_stone.cpp:77-90`) - the heavy round rock: gravity `-10.0f`, lift `25.0f` over
/// `23.0f`, `DATAFLG_2_e` (the `fopAcStts_UNK10000_e` that `_create` sets at `:251-253`), shock
/// pattern `0x2`, break `0x692B`, water `0x6919`.
pub const ALWAYS_HEAVY: Attr = Attr {
    gravity: -10.0,
    reach: 22.0,
    lift: 25.0,
    toss: 23.0,
    toss_gain: 0.0,
    damp: 0.0,
    inert: 0.0,
    fall_limit: 200.0,
    shock: 0x2,
    sink_gravity: 6.5,
    sink_damp: 0.04,
    sink_inert: 0.014,
    speed_cap: 30.0,
    flags: DATAFLG_2,
    radius: 30.0,
    break_se: 0x692B,
    water_se: 0x6919,
    hit_se: 0xD,
};

/// `M_data[2]` (`d_a_stone.cpp:92-101`) - `M_arcname_Ebrock`, the great rock: reach `200.0f`, collider
/// radius `0x118 = 280`, sink gravity `5.4f`, every flag bar `DATAFLG_10_e`, shock pattern `0x6`,
/// break `0x696A`, water `0x691A`.
pub const EBROCK: Attr = Attr {
    gravity: -6.0,
    reach: 200.0,
    lift: 27.0,
    toss: 36.0,
    toss_gain: 0.0,
    damp: 0.0,
    inert: 0.0,
    fall_limit: 200.0,
    shock: 0x6,
    sink_gravity: 5.4,
    sink_damp: 0.04,
    sink_inert: 0.014,
    speed_cap: 30.0,
    flags: DATAFLG_1 | DATAFLG_2 | DATAFLG_4 | DATAFLG_8,
    radius: 280.0,
    break_se: 0x696A,
    water_se: 0x691A,
    hit_se: 0xD,
};

/// `M_data[3]` (`d_a_stone.cpp:103-106`) - `M_arcname_Ekao`: reach `175.0f`, collider radius
/// `0x113 = 275`, and the `DATAFLG_10_e` that suppresses the random create-time yaw (`:205`). This
/// is the row `m670 != 3` of the bomb arm (`:520`) names.
pub const EKAO: Attr = Attr {
    gravity: -6.0,
    reach: 175.0,
    lift: 27.0,
    toss: 36.0,
    toss_gain: 0.0,
    damp: 0.0,
    inert: 0.0,
    fall_limit: 200.0,
    shock: 0x6,
    sink_gravity: 5.4,
    sink_damp: 0.04,
    sink_inert: 0.014,
    speed_cap: 30.0,
    flags: DATAFLG_1 | DATAFLG_2 | DATAFLG_4 | DATAFLG_8 | DATAFLG_10,
    radius: 275.0,
    break_se: 0x6969,
    water_se: 0x691A,
    hit_se: 0xD,
};

/// `M_data[4]` (`d_a_stone.cpp:108-117`) - `M_arcname_Ebrock`, the little rock: reach `90.0f`,
/// collider radius `0x6E = 110`, shock pattern `0x4`, break `0x696B`, water `0x691A`.
pub const EBROCK_SMALL: Attr = Attr {
    gravity: -6.0,
    reach: 90.0,
    lift: 27.0,
    toss: 36.0,
    toss_gain: 0.0,
    damp: 0.0,
    inert: 0.0,
    fall_limit: 200.0,
    shock: 0x4,
    sink_gravity: 5.4,
    sink_damp: 0.04,
    sink_inert: 0.014,
    speed_cap: 30.0,
    flags: DATAFLG_1 | DATAFLG_2 | DATAFLG_4 | DATAFLG_8,
    radius: 110.0,
    break_se: 0x696B,
    water_se: 0x691A,
    hit_se: 0xD,
};

/// `data()` (`d_a_stone.h:104`) - `M_data[m670]` in table order, the five entries declared at
/// `d_a_stone.h:155`. `M_data` is a five-row C array, so a `m670` past the end reads whatever
/// follows; the wrap below is the same guard `stone.rs` applies to `M_attr`.
const ATTRS: [Attr; 5] = [
    ALWAYS_SMALL,
    ALWAYS_HEAVY,
    EBROCK,
    EKAO,
    EBROCK_SMALL,
];

/// `m674`'s four values, in `mode_proc[]` order (`:416-421`) - `0` `mode_wait` (`:417`), `1`
/// `mode_carry` (`:418`), `2` `mode_drop` (`:419`), `3` `mode_sink` (`:420`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Wait,
    Carry,
    Drop,
    Sink,
}

/// `daStone::Act_c` (`d_a_stone.h:82-194`) - the carryable breakable boulder behind `koisi1` and
/// `koisi2`.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Koisi {
    /// `m670 = prm_get_type()` (`:191`) - the `M_data[m670]` row index.
    pub kind: u8,
    /// `prm_get_itemNo()` (`d_a_stone.h:106`) - `i_itemNo` of `damaged` (`:485`).
    pub item_no: u8,
    /// `prm_get_itemSave()` (`d_a_stone.h:107`) - `i_itemBitNo` of `damaged` (`:486`).
    pub item_save: u8,
    /// `prm_get_swSave()` (`d_a_stone.h:109`) - the `fopAcM_isSwitch` (`:180`) and `fopAcM_onSwitch`
    /// (`:503`) index.
    pub switch_no: u8,
    /// `prm_get_moveBg()` (`d_a_stone.h:108`).
    pub move_bg: bool,
    /// `prm_get_cull()` (`d_a_stone.h:105`), whose `== 0` arm forces `_execute`'s update block
    /// (`:920`).
    pub cull: u8,
    /// `m674` - the `mode_proc[m674]` state (`:431`).
    pub mode: Mode,
    /// `m688` - the height `damage_bg_proc_directly` measures a fall from (`:603`, `:607`), also
    /// written by `mode_carry` on release (`:335`) and by `_create` (`:243`).
    pub last_y: f32,
    /// `m68C = chk_appear()` (`:192`). It starts `false` because `Mthd_Create` needs the live
    /// `Switches` that `spawn` is not handed, and `control` resolves it on the first tick; while it
    /// is `false` it also stands in for `m67A`, which `_create` sets to 1 (`:241`) and `_execute`
    /// clears (`:923`).
    pub appears: bool,
    /// `m500` - the whole body of `chk_sink_water` (`:833`) and the guard of `water_tention`
    /// (`:840`).
    pub wet: bool,
    /// `m4F4.y` - this step's water surface height, or [`NO_WATER`] (`:785`, `:789`, `:793`).
    pub water_y: f32,
    /// `m4F4.z` - the previous step's `m4F4.y`, written by `m4F4.z = m4F4.y` (`:774`).
    pub water_prev: f32,
    /// `speed` - `fopAcM_GetSpeed(this)`.
    pub speed: Vec3,
    /// `speedF`.
    pub speed_f: f32,
    /// `gravity` - the `fopAcM_SetGravity` value `posMoveF_grade` adds to `speed.y`
    /// (`src/d/d_a_obj.cpp:153`).
    pub gravity: f32,
}

impl Koisi {
    /// `_create` (`:189-266`) minus the archive load (`:197`), the solid heap (`:203`), the model, the
    /// collider registration, the create-time random yaw (`:205-209`) and the attention request
    /// (`:248-256`). What survives is the six `PrmAbstract` reads, `fopAcM_SetGravity` (`:228`), the
    /// inits `m678 = 1` / `m679 = 0x14` (`:241-242`) and `mode_wait_init` (`:246`, `:290`).
    pub fn spawn(parameters: u32) -> Self {
        Self {
            kind: field(parameters, KIND_SHIFT, KIND_WIDTH),
            item_no: field(parameters, ITEM_NO_SHIFT, ITEM_NO_WIDTH),
            item_save: field(parameters, ITEM_SAVE_SHIFT, ITEM_SAVE_WIDTH),
            switch_no: field(parameters, SWITCH_SHIFT, SWITCH_WIDTH),
            move_bg: field(parameters, MOVE_BG_SHIFT, MOVE_BG_WIDTH) == 1,
            cull: field(parameters, CULL_SHIFT, CULL_WIDTH),
            // m674 = 0 by mode_wait_init() (:290)
            mode: Mode::Wait,
            // m688 = current.pos.y at create (:243), resolved on the first tick from Actor::old.
            last_y: 0.0,
            appears: false,
            wet: false,
            water_y: NO_WATER,
            water_prev: NO_WATER,
            speed: Vec3::ZERO,
            speed_f: 0.0, // fopAcM_SetSpeedF(this, 0.0f) (:287)
            gravity: 0.0, // fopAcM_SetGravity(this, data().m00) (:228)
        }
    }

    /// `data()` (`d_a_stone.h:104`) - `M_data[m670]`.
    pub fn attr(&self) -> Attr {
        ATTRS[usize::from(self.kind) % ATTRS.len()]
    }

    /// `chk_appear()` (`:175-186`): `if (data().m70 & 8) { if (switchIndex == 0xff) return true;
    /// else if (fopAcM_isSwitch(this, switchIndex)) return false; return true; } return true;`.
    /// `Mthd_Create` reads it once, before `Create` (`:192`), and a false answer leaves the
    /// `cPhs_ERROR_e` initialiser (`:194`) in place so the process never runs at all.
    pub fn chk_appear(&mut self, switches: &Switches) -> bool {
        let flagged = self.attr().flags & DATAFLG_8 != 0;
        self.appears =
            !flagged || self.switch_no == NO_SWITCH || !switches.test(self.switch_no);
        self.appears
    }

    /// `mode_drop_init` (`:349-369`), minus the three collider `SPrm` bits (`:353-355`), the
    /// `mStts.Init` (`:356`), the `mAcch` clearing (`:358-362`) and the attention bit (`:364`).
    ///
    /// `src/sim/carry.rs:138` calls this on release, which is the crate's stand-in for `mode_carry`'s
    /// `init_rot_throw(); mode_drop_init(); mode_drop();` chain (`:337-339`); the `mode_drop` half is
    /// the next statement of the same tick's `control`.
    ///
    /// `forward` is the player's facing vector `carry::control` passes. Retail does not use it:
    /// `mode_drop_init` writes only `fopAcM_SetSpeedF`, and `mode_drop`'s `posMoveF_stream` derives
    /// `speed.x` / `speed.z` from `speedF * (cM_ssin, cM_scos)(current.angle.y)`
    /// (`src/d/d_a_obj.cpp:152-154`). `carry::control` already steers `actor.angle[1]` onto the
    /// player's yaw every tick it holds the rock (`src/sim/carry.rs:127`), so the drop heading is
    /// right without the argument.
    pub fn toss(&mut self, _forward: Vec3, player_speed: f32) {
        let attr = self.attr();
        // f32 tmp1 = data().m18 + player->getSpeedF() * data().m1C; (:351)
        self.speed_f = attr.toss + player_speed * attr.toss_gain;
        self.speed.y = attr.lift; // fopAcM_GetSpeed(this).y = data().m14 (:365)
        self.gravity = attr.gravity; // gravity = data().m00 (:367)
        self.mode = Mode::Drop; // m674 = 2 (:368)
    }

    /// `mode_drop_init` (`:349-369`) then `mode_drop` (`:372-377`) on one tick, which is how
    /// `mode_carry` (`:337-339`) chains them. `daObj::get_wind_spd(this, 100.0f)` (`:374`) is a `dKy`
    /// wind vector, one of port_contract.md 3b's confirmed-absent inputs, so `mode_drop`'s stream
    /// vector is left at zero rather than guessed.
    fn begin_drop(&mut self, actor: &mut Actor, attr: Attr, player_speed: f32) {
        self.toss(Vec3::ZERO, player_speed);
        self.stream_move(actor, attr.damp, attr.inert);
    }

    /// `mode_proc_call` (`:423-426`): `if (fopAcM_CheckStatus(this, fopAcStts_CARRY_e) && m674 != 1)
    /// { init_rot_clean(); mode_carry_init(); }`, and `mode_carry_init` (`:305-318`) - the AT and Co
    /// `SPrm` bits go off, the carry attention bit goes off, `m68D = 0` and `m674 = 1`.
    fn begin_carry(&mut self, held: bool) {
        if held && self.mode != Mode::Carry {
            self.mode = Mode::Carry;
        }
    }

    /// `mode_wait_init` (`:277-291`): `fopAcM_SetSpeedF(this, 0.0f)` (`:287`), `gravity =
    /// data().m00` (`:288`) and `m674 = 0` (`:290`).
    fn rest(&mut self, attr: Attr) {
        self.speed_f = 0.0;
        self.gravity = attr.gravity;
        self.mode = Mode::Wait;
    }

    /// `mode_sink_init` (`:380-401`): `fopAcM_SetGravity(this, data().m00 + data().m44)` (`:390`),
    /// then the `sqrtf(speed.y * speed.y + speedF * speedF) > data().m58` rescale of `speed` and
    /// `speedF` (`:392-397`), then `m674 = 3` (`:400`).
    fn begin_sink(&mut self, attr: Attr) {
        self.gravity = attr.gravity + attr.sink_gravity;
        let length = (self.speed.y * self.speed.y + self.speed_f * self.speed_f).sqrt();
        if length > attr.speed_cap {
            let scale = attr.speed_cap / length;
            self.speed *= scale;
            self.speed_f *= scale;
        }
        self.mode = Mode::Sink;
    }

    /// `daObj::posMoveF_stream(this, mStts.GetCCMoveP(), stream, p4, p5)` (`:299`, `:375`, `:409`)
    /// = `posMoveF_grade` with a NULL ground normal (`src/d/d_a_obj.cpp:94-95`), which zeroes the
    /// grade pull (`:113-135`) and leaves the resist pair plus `speed.y + gravity`, then
    /// `fopAcM_posMove` (`:173`) integrates `pos += speed`. `mStts.GetCCMoveP()` is the `cCd`
    /// momentum channel this crate does not carry, the same absence `stone.rs` records for its own
    /// `posMoveF_stream` calls.
    fn stream_move(&mut self, actor: &mut Actor, damp: f32, inert: f32) {
        let (speed, speed_f, facing) = stream::move_stream(
            self.speed,
            Vec3::ZERO,
            actor.angle[1],
            self.speed_f,
            self.gravity,
            MAX_FALL,
            damp,
            inert,
        );
        self.speed = speed;
        self.speed_f = speed_f;
        actor.angle[1] = facing; // current.angle.y = cM_atan2s(speed_x, speed_z) (:170)
        actor.gravity = self.gravity; // fopAcM_SetGravity's value, readable by the shared walkers
        actor.pos += self.speed;
    }

    /// `mode_wait`'s grounded arm (`:296`): `fopAcM_posMoveF(this, mStts.GetCCMoveP())` =
    /// `fopAcM_calcSpeed` (`src/f_op/f_op_actor_mng.cpp:454-466`) then `fopAcM_posMove` (`:469`).
    fn settle(&mut self, actor: &mut Actor) {
        let (speed, speed_f) = stream::calc_speed(
            self.speed_f,
            actor.angle[1],
            self.speed.y,
            self.gravity,
            MAX_FALL,
        );
        self.speed = speed;
        self.speed_f = speed_f;
        actor.pos += self.speed;
    }

    /// `bg_crr_water` (`:765-797`), reduced to its `daSea` arm. `mAcch.ChkWaterIn()` and
    /// `mAcch.m_wtr.GetHeight()` (`:766`, `:770`) read the acch water plane, which the crate's
    /// merged-DZB `Collision` does not carry, so `bVar3` is left false and its tie-break against
    /// `bVar4` (`:776-782`) cannot fire; `water::probe` supplies the `daSea_ChkArea` /
    /// `daSea_calcWave` pair (`:767-768`) and the sentinel writes and `m500` flag below are exactly
    /// the decomp's remaining statements.
    ///
    /// `m501` (`:791`, `:794`) has no consumer here: it only steers the `bgs_arr[0]` slot of
    /// `se_fall_water` (`:865`), whose first entry is the same absent `mAcch.m_wtr`.
    fn probe_water(&mut self, attr: Attr, pos: Vec3, sea: Option<&Sea>) {
        self.water_prev = self.water_y; // m4F4.z = m4F4.y (:774)
        let (in_sea, wave) = super::water::probe(sea, pos.x, pos.z);
        // fVar2 = current.pos.y + data().m08; (:769)
        let top = pos.y + attr.reach;
        if in_sea && top < wave {
            // bVar4 = bVar5 && fVar2 < fVar7 (:771) -> m4F4.y = fVar7 (:789)
            self.water_y = wave;
            self.wet = true; // m500 = uVar6 (:796)
        } else {
            // m4F4.y = -G_CM3D_F_INF (:793)
            self.water_y = NO_WATER;
            self.wet = false; // m500 = uVar6 (:796)
        }
    }

    /// `water_tention` (`:839-853`): `if (chk_sink_water()) { if (m4F4.y != -G_CM3D_F_INF) { if
    /// (m4F4.z != -G_CM3D_F_INF) { f32 fVar1 = m4F4.y - m4F4.z; fVar1 = (fVar1 < 0.0f) ? fVar1 * 0.8f
    /// : fVar1 * 0.2f; current.pos.y += fVar1; } } }`.
    fn tension(&self) -> f32 {
        if !self.wet || self.water_y == NO_WATER || self.water_prev == NO_WATER {
            return 0.0;
        }
        let delta = self.water_y - self.water_prev;
        if delta < 0.0 {
            delta * TENSION_SINK
        } else {
            delta * TENSION_RISE
        }
    }

    /// `damage_cc_proc` (`:512-515`), first arm: `if (mCyl.ChkAtHit()) { mCyl.ClrAtHit();
    /// damaged(3); iVar3 = true; }`. The registered collider is the `M_cyl_src` cylinder
    /// (`:31-59`), moved to `current.pos` by `mCyl.MoveCAtTg` and handed to `dComIfG_Ccsp()->Set`
    /// (`:935-936`), so the test is horizontal against `data().m82`.
    ///
    /// `cCcD_AtSPrm_Set_e` is the rock's own AT arm, and `mode_drop_init` (`:353`) is the only init
    /// that leaves it on - `mode_wait_init` (`:278`), `mode_carry_init` (`:306`) and
    /// `mode_sink_init` (`:381`) all clear it - so in retail only a falling rock can be cut.
    ///
    /// The vertical half of the `cM3dGCylS` overlap is not modelled: `cM3dGCylS::Height` is consumed
    /// by the collision engine alone and its axis convention is not resolvable from anything in
    /// this checkout, the same omission `fallrock.rs` records.
    fn sword_hit(&self, centre: Vec3, tip: Vec3) -> bool {
        self.mode == Mode::Drop && walk::within(tip, centre, self.attr().radius + SWORD_REACH)
    }

    /// `if (m670 != 3 && hitObj->ChkAtType(AT_TYPE_BOMB))` (`:520`) - `M_data[3]`, the one row a bomb
    /// cannot break. `M_cyl_src`'s `SrcObjTg Type` (`:38`) is `AT_TYPE_ALL` minus water, wind, light
    /// and two unknowns, with `cCcD_TgSPrm_Set_e | cCcD_TgSPrm_IsOther_e` (`:39`), which is what
    /// makes the Tg arm see a bomb's AT. Every init leaves `cCcD_TgSPrm_Set_e` on (`:279`, `:307`,
    /// `:382`), so this arm is live in all four modes.
    fn bomb_breaks(&self) -> bool {
        self.kind != BOMB_EXEMPT
    }

    /// The reach half of the Tg arm: a bomb AT against the registered cylinder. The bomb's blast is
    /// the crate's `AT_TYPE_BOMB` channel, `bombflower::Blasts`.
    fn bomb_reached(&self, centre: Vec3, blasts: &[Vec3]) -> bool {
        blasts
            .iter()
            .any(|origin| walk::within(*origin, centre, BLAST_RADIUS + self.attr().radius))
    }

    /// `else if (hitObj->ChkAtType(AT_TYPE_SKULL_HAMMER)) { damaged(7); }` (`:523-524`). Unlike
    /// the bomb arm this one is not gated on `m670 != 3`.
    fn hammered(&self, centre: Vec3, head: Vec3) -> bool {
        walk::within(head, centre, self.attr().radius + hammer::AT_RADIUS)
    }

    /// `damage_bg_proc_directly` (`:600-608`), the `m674 == 0` arm: `if (uVar7) { bool groundLanding
    /// = mAcch.ChkGroundLanding(); if (groundLanding && m688 - current.pos.y > data().m38) {
    /// damaged(2); iVar9 = true; } m688 = current.pos.y; }`. A waiting rock that has been in the air
    /// shatters when it lands more than `data().m38` below the height it last rested at.
    ///
    /// `mAcch.ChkGroundLanding()` has no crate equivalent - `ground::grounded` reports the standing
    /// state, not a one-tick landing edge - so the conjunct folds onto the grounded test. Because
    /// `m688` is refreshed on every grounded tick, a rock already resting can never satisfy
    /// `m688 - current.pos.y > data().m38`, so the folded form is inert off the landing tick.
    fn landing_fatal(&self, grounded: bool, y: f32, attr: Attr) -> bool {
        grounded && self.last_y - y > attr.fall_limit
    }

    /// `damaged(int arg1)` (`:484-506`).
    ///
    /// Omitted from that function, with the decomp line: the `eff_break` body (`:720-736`) and
    /// `data().m88` are particle work; `set_senv(data().m68, data().m69)` (`:496`) is
    /// `dKy_Sound_set` (`:907-909`), a sound-position marker with no home in this sim; and the
    /// `&csXyz(0, home.angle.y, 0)` spawn yaw (`:488`) is the stage-placed angle, which `Actor`
    /// does not carry - `stone.rs` records the same omission for its identical call.
    fn damaged(
        &self,
        action: u8,
        attr: Attr,
        actor: &Actor,
        carry: &mut Carry,
        switches: &mut Switches,
        events: &mut SimEvents,
        shake: &mut Vibration,
        commands: &mut Commands,
    ) {
        // fopAcM_createItemFromTable(&current.pos, prm_get_itemNo(), prm_get_itemSave(),
        // fopAcM_GetHomeRoomNo(this), 0, &csXyz(0, home.angle.y, 0), arg1, NULL) (:489). Its
        // `0x20 <= i_itemNo <= 0x3e` arm resolves through `dComIfGp_getItemTable()` and
        // `cM_rndF(15.9999f)` (`src/f_op/f_op_actor_mng.cpp:817-889`) - a stage item table absent
        // from this checkout, so that whole arm is omitted rather than guessed.
        if self.item_no < NO_ITEM {
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
        carry.held = None; // fopAcM_cancelCarryNow(this) (:490)
        // if (data().m40 > 0) StartShock(data().m40, -0x21, cXyz(0.0f, 1.0f, 0.0f)); (:493-495)
        if attr.shock > 0 {
            let _ = shake.start_shock(i32::from(attr.shock), SHOCK_FLAGS, SHOCK_DIR);
        }
        events.list.push(SimEvent::Se(attr.break_se)); // fopAcM_seStart(this, data().m8C, 0) (:498)
        // if (data().m70 & 8) { iVar1 = prm_get_swSave(); if (iVar1 != 0xFF) fopAcM_onSwitch(this,
        // iVar1); } (:500-505)
        if attr.flags & DATAFLG_8 != 0 && self.switch_no != NO_SWITCH {
            switches.set(self.switch_no);
        }
    }

    /// The clink a Tg hit that breaks nothing makes: `daObj::HitSeStart(&eyePos, current.roomNo,
    /// &mCyl, data().m9C)` (`:530`), reached from the `if (!iVar3)` fallthrough (`:529-533`). Its
    /// `set_senv(data().m6A, data().m6B)` (`:531`) is the sound-position marker and
    /// `daObj::HitEff_hibana(this, &mCyl)` (`:532`) is the spark particle; neither is ported.
    fn clang(&self, attr: Attr, events: &mut SimEvents) {
        events.list.push(SimEvent::Se(attr.hit_se));
    }

    /// `se_fall_water` (`:857-885`): `fopAcM_seStart(this, data().m94, uVar4)`. `uVar4` starts at
    /// `0x13` (`:870`) and is only replaced by a `dComIfG_Bgsp()->GetMtrlSndId` read, which needs
    /// the per-poly-group material data port_contract.md 3b lists as absent - so the decomp's own
    /// default selector stands.
    fn splash(&self, attr: Attr, events: &mut SimEvents) {
        events.list.push(SimEvent::Se(attr.water_se));
    }
}

fn field(parameters: u32, shift: u32, width: u32) -> u8 {
    ((parameters >> shift) & ((1 << width) - 1)) as u8
}

pub fn is_koisi(name: &str) -> bool {
    KOISI_ACTORS.contains(&name)
}

pub fn control(
    collision: Option<Res<Collision>>,
    sea: Option<Res<Sea>>,
    combat: Res<Combat>,
    blasts: Res<Blasts>,
    cut: Res<hammer::Cut>,
    mut carry: ResMut<Carry>,
    mut switches: ResMut<Switches>,
    mut events: ResMut<SimEvents>,
    mut shake: ResMut<Vibration>,
    mut rocks: Query<(Entity, &mut Actor, &mut Koisi)>,
    players: Query<&Player>,
    hammers: Query<&hammer::Hammer>,
    mut commands: Commands,
) {
    let solid = collision.as_deref();
    let sea = sea.as_deref();
    let player = players.iter().next();
    let swing = combat.active && combat.frame >= CUT_START && combat.frame < CUT_END;
    let fired: Vec<Vec3> = blasts.list.clone();
    let swinging = cut.kind != hammer::CUT_NONE && hammers.iter().any(|head| head.striking());
    for (entity, mut actor, mut rock) in &mut rocks {
        let actor = &mut *actor;
        let rock = &mut *rock;
        // m68C = chk_appear() (:192). RULE 6b: a false answer leaves `cPhs_ERROR_e` in place so the
        // process never runs - spawn it, despawn on the first tick.
        if !rock.appears && !rock.chk_appear(&switches) {
            let _ = commands.entity(entity).despawn();
            continue;
        }
        let first = !rock.appears;
        rock.appears = true;
        let attr = rock.attr();
        // `_execute` (`:920`) gates the whole update block on
        // `m67A != 0 || m674 != 0 || !mAcch.ChkGroundHit() || mAcch.ChkGroundLanding() ||
        // prm_get_cull() == 0 || !fopAcM_cullingCheck(this) || prm_get_moveBg()`. `m67A` is one
        // until the first tick (`:241`, `:923`); `ChkGroundLanding` has no crate equivalent; the
        // crate models no `fopAcM_cullingCheck` at all, so every actor reads as on-screen and that
        // term is false; and the moving-background arms are absent. `fopAc_Execute` snapshots `old`
        // before the actor method runs (`src/sim/actor.rs:86`) while `mAcch.CrrPos` (:438) does not
        // run until inside `mode_proc_call`, so the gate sees the previous step's corrected position.
        let grounded_before = ground::grounded(solid, actor.old.pos);
        let run = first
            || rock.mode != Mode::Wait
            || !grounded_before
            || rock.cull == 0
            || rock.move_bg;
        if !run {
            continue;
        }
        if first {
            rock.last_y = actor.pos.y; // m688 = current.pos.y (:243)
        }
        let tip = player.map(|player| {
            Vec3::new(
                player.pos.x + SWORD_OFFSET * player.yaw.sin(),
                player.pos.y,
                player.pos.z + SWORD_OFFSET * player.yaw.cos(),
            )
        });
        // damage_cc_proc() || damage_bg_proc() (:926) runs ahead of mode_proc_call.
        let bomb_reached = rock.bomb_reached(actor.pos, &fired);
        let cut_hit = swing && tip.is_some_and(|tip| rock.sword_hit(actor.pos, tip));
        let bomb_hit = bomb_reached && rock.bomb_breaks();
        let hammer_hit = player.is_some_and(|player| {
            swinging && rock.hammered(actor.pos, hammer::strike_origin(player))
        });
        if cut_hit || bomb_hit || hammer_hit {
            let action = if cut_hit { ITEM_ACT_3 } else { ITEM_ACT_7 };
            rock.damaged(
                action,
                attr,
                actor,
                &mut carry,
                &mut switches,
                &mut events,
                &mut shake,
                &mut commands,
            );
            // _execute (:948-950): `if (bVar1) fopAcM_delete(this);`
            let _ = commands.entity(entity).despawn();
            continue;
        }
        if bomb_reached {
            rock.clang(attr, &mut events); // :529-533
        }
        // damage_bg_proc (:543-589), reduced to its water arm. `chk_sink_lava` (:812-818) reads the
        // `dBgS_ObjGndChk_Yogan` lava `GroundCross` (:807), which is an absent input, so its two
        // branches - and the `data().m70 & 1` test inside them (:559`, `:576`) - are omitted.
        match rock.mode {
            // m674 == 0: if (!uVar7) { if (bVar2) { ... } } (:549-555)
            Mode::Wait => {
                if !grounded_before && rock.wet {
                    rock.splash(attr, &mut events); // se_fall_water() (:552)
                    // eff_hit_water_splash() (:553) is the wpillar particle; init_rot_clean() (:554)
                    // is the renderer-only tumble reset.
                    rock.begin_sink(attr); // mode_sink_init() (:555)
                }
            }
            // m674 == 2: if (bVar2) { ... } (:568-572)
            Mode::Drop => {
                if rock.wet {
                    rock.splash(attr, &mut events); // se_fall_water() (:570)
                    // eff_hit_water_splash() (:571) is the wpillar particle.
                    rock.begin_sink(attr); // mode_sink_init() (:572)
                }
            }
            // m674 == 3: if (uVar7 || (!bVar2 && !bVar3)) mode_wait_init(); (:583-587)
            Mode::Sink => {
                if grounded_before || !rock.wet {
                    rock.rest(attr);
                }
            }
            Mode::Carry => {}
        }
        // mode_proc_call (:414-471): the carry latch, then exactly one mode_proc[m674].
        let held = carry.held == Some(entity);
        let restore = rock.move_bg; // uVar7 = prm_get_moveBg() (:428)
        let before = actor.pos; // sp08 = current.pos (:429)
        rock.begin_carry(held);
        match rock.mode {
            // mode_wait (:294-302)
            Mode::Wait => {
                if grounded_before {
                    rock.settle(actor); // fopAcM_posMoveF(this, mStts.GetCCMoveP()) (:296)
                } else {
                    // daObj::posMoveF_stream(this, CCMoveP, &cXyz::Zero, data().m20, data().m24)
                    rock.stream_move(actor, attr.damp, attr.inert); // (:299)
                }
            }
            // mode_carry (:321-346)
            Mode::Carry => {
                if !held {
                    rock.last_y = actor.pos.y; // m688 = current.pos.y (:335)
                    // fopAcM_GetSpeedF(this) > 0.0f || fopAcM_CheckStatus(this,
                    // fopAcStts_UNK10000_e) (:336); the status bit is the `data().m70 & 2` that
                    // `_create` sets (:251-253).
                    let thrown = rock.speed_f > 0.0 || attr.flags & DATAFLG_2 != 0;
                    if thrown {
                        match player {
                            Some(player) => rock.begin_drop(actor, attr, player.speed),
                            // mode_drop_init reads `player->getSpeedF()`, so without a player the
                            // release cannot run; the rock stays carried.
                            None => continue,
                        }
                    } else {
                        // m67B = 2 (:342) is the two-tick dComIfG_Ccsp()->SetMass window (:937-939).
                        rock.rest(attr); // mode_wait_init() (:343)
                    }
                }
                // The `m68D` grab-up bookkeeping (:325-333) only gates
                // `fopAcM_SetModel(this, (m674 == 1 && m68D != 0) ? mpModel : NULL)` (:947), i.e.
                // whether the held rock is drawn at all.
            }
            // mode_drop (:372-377) - on the release tick `begin_drop` above already ran it.
            Mode::Drop => rock.stream_move(actor, attr.damp, attr.inert),
            // mode_sink (:404-411)
            Mode::Sink => {
                actor.pos.y += rock.tension(); // water_tention() (:407)
                // daObj::posMoveF_stream(this, CCMoveP, &cXyz::Zero, data().m48, data().m4C) (:409)
                rock.stream_move(actor, attr.sink_damp, attr.sink_inert);
            }
        }
        if rock.mode == Mode::Carry {
            // if (m674 == 1) { sp08 = current.pos; uVar7 = true; } (:433-436)
            actor.pos = before;
        } else if restore {
            // if (uVar7) current.pos = sp08; (:456-458), with the move-background ground snap of
            // :444-454 omitted - `ChkMoveBG_NoDABg`, `ChkMoveBG` and `prm_off_moveBg` are all
            // `dBgW` work, which port_contract.md 3b lists as absent.
            actor.pos = before;
        }
        // mAcch.CrrPos (:438), bg_crr_water (:440) and bg_crr_lava (:442).
        rock.probe_water(attr, actor.pos, sea);
        // damage_bg_proc_directly (:592-642) runs last, on the corrected position.
        if rock.mode == Mode::Wait {
            let grounded = ground::grounded(solid, actor.pos);
            if rock.landing_fatal(grounded, actor.pos.y, attr) {
                rock.damaged(
                    ITEM_ACT_2,
                    attr,
                    actor,
                    &mut carry,
                    &mut switches,
                    &mut events,
                    &mut shake,
                    &mut commands,
                );
                let _ = commands.entity(entity).despawn();
            }
            rock.last_y = actor.pos.y; // m688 = current.pos.y (:607)
        } else if rock.mode == Mode::Drop && ground::grounded(solid, actor.pos) {
            // m674 == 2: if (uVar7) { damaged(2); iVar9 = true; } (:615-617)
            rock.damaged(
                ITEM_ACT_2,
                attr,
                actor,
                &mut carry,
                &mut switches,
                &mut events,
                &mut shake,
                &mut commands,
            );
            let _ = commands.entity(entity).despawn();
        }
    }
}