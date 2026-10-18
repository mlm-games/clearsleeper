use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::events::{SimEvent, SimEvents};
use super::flags::{EventFlags, Switches};
use super::player::Player;
use super::player_status::{PlayerStatus, STTS0_SHIP_RIDE};
use super::ship::SHIP_ACTOR;
use super::walk;

/// `OBJNAME("Auzu", fpcNm_Obj_Auzu_e, 255, 58)` (`d_stage.cpp:1027`) is `fpcNm_Obj_Auzu_e` =
/// `d_a_obj_auzu.cpp`. `daObjAuzu::Act_c::M_arcname` is also `"Auzu"` (`d_a_obj_auzu.cpp:18`), so
/// stage string and archive name agree - there is no `Lwood`/`lwood` trap here. No other
/// `*_ACTORS` array in `src/sim/` claims `Auzu`.
pub const AUZU_ACTORS: [&str; 1] = ["Auzu"];

/// `f32 L_radius = 2500.0f` (`d_a_obj_auzu.cpp:15`, file-scope in `namespace daObjAuzu`). Every use
/// is `L_radius * attr().<mult>` (`:157-158`, `:199`, `:229-230`, `:238-239`).
const L_RADIUS: f32 = 2500.0; // <- d_a_obj_auzu.cpp:15
/// `M_attr[0].mRadiusMult` (`d_a_obj_auzu.cpp:21`) - the cull half-extent multiplier (`:76`) and the
/// wave-radius multiplier (`:229`, `:238`).
const RADIUS_SMALL: f32 = 1.5; // <- d_a_obj_auzu.cpp:21
/// `M_attr[0].mOuterActivationMult` (`d_a_obj_auzu.cpp:22`) - the outer pull radius (`:158`, `:199`).
const OUTER_SMALL: f32 = 1.5; // <- d_a_obj_auzu.cpp:22
/// `M_attr[0].mInnerActivationMult` (`d_a_obj_auzu.cpp:23`) is `1.3f` and
/// `M_attr[1].mInnerActivationMult` (`:29`) is `2.0f`. Neither is stored: `inner_activation`
/// (`:157`) only selects `onWhirlFlg` (`:167`) over `onWhirlFlgDirect` (`:169`), and both calls are
/// omitted (see OMITTED).
/// `M_attr[1].mRadiusMult` (`d_a_obj_auzu.cpp:27`).
const RADIUS_BIG: f32 = 2.0; // <- d_a_obj_auzu.cpp:27
/// `M_attr[1].mOuterActivationMult` (`d_a_obj_auzu.cpp:28`).
const OUTER_BIG: f32 = 2.0; // <- d_a_obj_auzu.cpp:28

/// `f32 cull_xz = attr().mRadiusMult * 3000.0f;` (`d_a_obj_auzu.cpp:76`) - the half-extent handed to
/// `fopAcM_setCullSizeBox` (`:77-80`).
const CULL_XZ: f32 = 3000.0; // <- d_a_obj_auzu.cpp:76
/// `fopAcM_setCullSizeBox(this, -cull_xz, -30.0f, -cull_xz, cull_xz, 30.0f, cull_xz)`
/// (`d_a_obj_auzu.cpp:77-80`) - the two Y faces.
const CULL_Y_MIN: f32 = -30.0; // <- d_a_obj_auzu.cpp:78
const CULL_Y_MAX: f32 = 30.0; // <- d_a_obj_auzu.cpp:79

/// `if (mScaleAnimFactor > 0.01f)` (`d_a_obj_auzu.cpp:164`) - the gate on `bgm_start()` and on
/// `onWhirlFlg` / `onWhirlFlgDirect`. `VERSION > VERSION_DEMO` (`:156`) is true for retail NTSC-U,
/// so `ship_whirl`'s both-radius form at `:157-158` is the one that compiles.
const ACTIVATE_EPS: f32 = 0.01; // <- d_a_obj_auzu.cpp:164
/// `cLib_chaseF(&mScaleAnimFactor, scale_target, 0.02f)` (`d_a_obj_auzu.cpp:223`) - the appear /
/// disappear lerp.
const SCALE_STEP: f32 = 0.02; // <- d_a_obj_auzu.cpp:223
/// `dComIfGs_isEventBit(dSv_event_flag_c::ENDLESS_NIGHT)` (`d_a_obj_auzu.cpp:116`,
/// `d_save_event_flag.inc:80`) - the type-0 existence gate. Under RULE 11c it is byte-indexed and
/// 0x0A02 <= 0xFF07, so the retail id is passed straight through.
const EVT_ENDLESS_NIGHT: u32 = 0x0A02; // <- d_save_event_flag.inc:80
/// `mDoAud_subBgmStart(JA_BGM_DIOCTA_BATTLE)` (`d_a_obj_auzu.cpp:182`, `JAZelAudio_BGM.h:49`) -
/// latched once by `mbBgmStarted` (`:180-181`).
const BGM_DIOCTA_BATTLE: u32 = 0x8000_002B; // <- JAZelAudio_BGM.h:49
/// `fopAcM_seStart(this, JA_SE_ATM_SWIRL, param)` (`d_a_obj_auzu.cpp:225`,
/// `JAZelAudio_SE.h:299`), where `param = s8(100.0f * mScaleAnimFactor)` (`:224`). The sim's
/// `SimEvent::Se` carries the id alone, so `param` has no channel.
const SE_ATM_SWIRL: u32 = 0x1068; // <- JAZelAudio_SE.h:299

/// `enum Prm_e { PRM_APPEAR_W = 0x1, PRM_APPEAR_S = 0x10, PRM_LINK_ID_W = 0x8, PRM_LINK_ID_S = 0x8,
/// PRM_SWSAVE_W = 0x8, PRM_SWSAVE_S = 0x0, PRM_TYPE_W = 0x4, PRM_TYPE_S = 0x14 }`
/// (`d_a_obj_auzu.h:20-32`), each read through `daObj::PrmAbstract(this, W, S)` =
/// `((1 << W) - 1) & (fopAcM_GetParam(actor) >> S)` (`d_a_obj.h:14`). `fopAcM_GetParam` returns the
/// WHOLE u32 (RULE 7).
const APPEAR_WIDTH: u32 = 0x1; // <- d_a_obj_auzu.h:21
const APPEAR_SHIFT: u32 = 0x10; // <- d_a_obj_auzu.h:22
const SWSAVE_WIDTH: u32 = 0x8; // <- d_a_obj_auzu.h:27
const SWSAVE_SHIFT: u32 = 0x0; // <- d_a_obj_auzu.h:28
const TYPE_WIDTH: u32 = 0x4; // <- d_a_obj_auzu.h:30
const TYPE_SHIFT: u32 = 0x14; // <- d_a_obj_auzu.h:31
/// `PRM_LINK_ID_W = 0x8` at `PRM_LINK_ID_S = 0x8` (`d_a_obj_auzu.h:24-25`) -
/// `prm_get_linkID()` (`d_a_obj_auzu.h:37`). Transcribed here but not stored: its only readers are
/// the three omitted `ship_p->...WhirlFlg(id, prm_get_linkID())` calls at `:167`, `:169`, `:172`.

/// The whirlpool in Jabu-Jabu's cave - `daObjAuzu::Act_c` (`d_a_obj_auzu.h:18-76`), the actor behind
/// the stage string `Auzu`.
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. `daObjAuzu::Act_c` is a bare `fopAc_ac_c`
/// holding a model, a BTK animator, the type, the scale factor, three booleans and a child id
/// (`d_a_obj_auzu.h:66-76`); it registers no `dCcD` collider of any kind, and the profile `Group`
/// is `fopAc_ACTOR_e` (`d_a_obj_auzu.cpp:330`). Neither `combat::HitPoints` nor `enemy::Hurt` is
/// attached - attaching them would let `combat::control` despawn a whirlpool retail cannot damage.
///
/// `_create` can return `cPhs_ERROR_e` from three places: `entrySolidHeap` returning 0 (`:96-98`,
/// heap work), and - the one real refusal - `mbIsExist = is_exist()` being 0 (`:69-70`), which for
/// `mType == 0` means `!dComIfGs_isEventBit(ENDLESS_NIGHT)` (`:115-117`). `cPhs_ERROR_e` cancels the
/// actor inside Create (`f_pc_create_req.cpp:106-108`), so that arm is a spawn-then-despawn on the
/// FIRST TICK, never `Option<Self> -> None` (RULE 6b). `Mthd_IsDelete` (`:302-305`) is the bare
/// "accepts a delete request" form of RULE 6b, and `fopAcM_delete` appears nowhere in this
/// translation unit.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Auzu {
    /// `mType` (`d_a_obj_auzu.h:69`, an `s32`) from `prm_get_type()` (`:66`). Selects `M_attr[mType]`
    /// (`d_a_obj_auzu.h:34`) and discriminates `is_exist()` (`:115`) and `bgm_start()` (`:180`).
    pub kind: u8,
    /// `mScaleAnimFactor` (`d_a_obj_auzu.h:70`) - the appear / disappear lerp value.
    pub scale: f32,
    /// `mbToAppear` (`d_a_obj_auzu.h:71`). Written by `to_appear()` / `to_disappear()`
    /// (`d_a_obj_auzu.h:47-48`); the only callers in the decomp are `d_a_daiocta.cpp:954` and
    /// `:1240`, and `src/sim/daiocta.rs` models neither, so this holds the `fopAcM_ct` zero-fill
    /// (`f_pc_base.cpp:98`) - i.e. FALSE - which is exactly what retail holds until Octa's death
    /// cut reaches DEATH1.
    pub to_appear: bool,
    /// `mbIsExist` (`d_a_obj_auzu.h:72`) - `is_exist()`'s result. 0 is the `cPhs_ERROR_e` refusal.
    pub exist: bool,
    /// `mbBgmStarted` (`d_a_obj_auzu.h:73`) - `bgm_start()`'s one-shot latch.
    pub bgm_started: bool,
    /// `prm_get_swSave()` (`d_a_obj_auzu.h:38`) - `daObj::PrmAbstract(this, 8, 0)`, the switch that
    /// decides `scale_target` when `prm_get_appear() != 1` (`:216`).
    pub switch_no: u8,
    /// `prm_get_appear()` (`d_a_obj_auzu.h:36`) - `daObj::PrmAbstract(this, 1, 0x10)`. Selects
    /// whether `mbToAppear` (`:210`) or `fopAcM_isSwitch` (`:216`) decides `scale_target`.
    pub appear: u8,
    /// One-shot guard so `is_exist()`'s single `dComIfGs_isEventBit` read (`:116`) happens once.
    pub entered: bool,
}

impl Auzu {
    /// `daObjAuzu::Act_c::_create` (`:64-103`), minus `dComIfG_resLoad` (`:71`),
    /// `fopAcM_entrySolidHeap` / `create_heap` (`:73`, `:41-61`), `fopAcM_SetMtx` (`:74`),
    /// `init_mtx` (`:75`, `:129-138`), the cull box (`:76-80`, carried by [`Auzu::cull`]) and
    /// `set_state_map` (`:95`, `:141-149`).
    ///
    /// The `mScaleAnimFactor` choice (`:83-91`) needs `Switches` and `is_exist()` needs
    /// `EventFlags`, so both are re-decided on the first tick - the `hbrf1` / `tagetc` shape, which
    /// cannot see those resources from `spawn`.
    ///
    /// `fopAc_Init` seeds `current` and `old` from `home` (`f_op_actor.cpp:282-284`), so the
    /// whirlpool starts at its placement position; RULE 2 makes its scale exactly 1.0, and
    /// `init_mtx`'s `scale.x * attr().mRadiusMult` (`:130-135`) is renderer work.
    pub fn spawn(parameters: u32) -> Self {
        // `prm_get_linkID()` = `((1 << 8) - 1) & (param >> 8)` (`d_a_obj_auzu.h:24-25`, `:37`).
        // Copied exactly, then deliberately NOT stored: its only readers are the omitted
        // `ship_p->onWhirlFlg(fopAcM_GetID(this), prm_get_linkID())` (`:167`), its `Direct` twin
        // (`:169`) and `ship_p->offWhirlFlg()` (`:172`), and an unread field is dead code (RULE 7 /
        // RULE 12b).
        //
        // The `fopAcM_create(fpcNm_KYTAG01_e, ...)` at `:231-234` names `OBJNAME("ky_tag1",
        // fpcNm_KYTAG01_e, 255, 0)` (`d_stage.cpp:660`); see OMITTED for why no child is spawned.
        Self {
            kind: ((parameters >> TYPE_SHIFT) & ((1 << TYPE_WIDTH) - 1)) as u8,
            scale: 0.0,
            to_appear: false,
            // Re-decided by `create_init`, which is the only reader of `is_exist()` (`:69`).
            exist: false,
            bgm_started: false,
            switch_no: ((parameters >> SWSAVE_SHIFT) & ((1 << SWSAVE_WIDTH) - 1)) as u8,
            appear: ((parameters >> APPEAR_SHIFT) & ((1 << APPEAR_WIDTH) - 1)) as u8,
            entered: false,
        }
    }

    /// `fopAcM_setCullSizeBox(this, -cull_xz, -30.0f, -cull_xz, cull_xz, 30.0f, cull_xz)`
    /// (`d_a_obj_auzu.cpp:77-80`), where `cull_xz = attr().mRadiusMult * 3000.0f` (`:76`). A real
    /// box, so the profile's `fopAc_CULLBOX_CUSTOM_e` (`:331`) resolves.
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        let reach = self.radius_mult() * CULL_XZ;
        ([-reach, CULL_Y_MIN, -reach], [reach, CULL_Y_MAX, reach])
    }

    /// `attr()` = `M_attr[mType]` (`d_a_obj_auzu.h:34`). `M_attr` has exactly two entries
    /// (`:19-32`) while `prm_get_type()` decodes FOUR bits (`d_a_obj_auzu.h:31`, `d_a_obj.h:14`), so
    /// retail itself reads past the table for a type above 1 - RULE 10 requires the index be
    /// guarded, and anything that is not exactly 1 folds to 0.
    fn radius_mult(&self) -> f32 {
        if self.kind == 1 {
            RADIUS_BIG
        } else {
            RADIUS_SMALL
        }
    }

    /// `attr().mOuterActivationMult` (`d_a_obj_auzu.cpp:22` / `:28`).
    fn outer_mult(&self) -> f32 {
        if self.kind == 1 {
            OUTER_BIG
        } else {
            OUTER_SMALL
        }
    }

    /// `M_attr[0].mInnerActivationMult` (`d_a_obj_auzu.cpp:23`) and `M_attr[1].mInnerActivationMult`
    /// (`:29`) - the inner pull radius, squared at `:157`. NOT stored: that value only chooses between
    /// `ship_p->onWhirlFlg` (`:167`) and `ship_p->onWhirlFlgDirect` (`:169`), both omitted, so a stored
    /// copy would be dead code (RULE 12b).

    /// `is_exist()` (`:114-119`) plus `_create`'s `mScaleAnimFactor` choice (`:83-91`).
    fn create_init(&mut self, switches: &Switches, flags: &EventFlags) {
        // `if (mType == 0) { return dComIfGs_isEventBit(ENDLESS_NIGHT); } return true;` (:114-119)
        self.exist = self.kind != 0 || flags.test(EVT_ENDLESS_NIGHT);
        // `if (prm_get_appear() == 1) { mScaleAnimFactor = 0.0f; } else if (fopAcM_isSwitch(this,
        // prm_get_swSave())) { mScaleAnimFactor = 0.0f; } else { mScaleAnimFactor = 1.0f; }`
        // (:83-91)
        self.scale = if self.appear == 1 || switches.test(self.switch_no) {
            0.0
        } else {
            1.0
        };
        self.entered = true;
    }

    /// `daObjAuzu::Act_c::_execute` (`:187-250`), minus `set_mtx` (`:189`), `mBtkAnm.play()` (`:206`)
    /// and `set_state_map` (`:248`). `#if VERSION > VERSION_JPN` (`:188`) is true for retail NTSC-U,
    /// so the `set_mtx()` at `:189` IS compiled - and it is model-matrix work.
    fn execute(&mut self, actor: &Actor, ctx: &mut Context<'_>) {
        // `if (dComIfGp_checkPlayerStatus0(0, daPyStts0_SHIP_RIDE_e)) { ship_whirl(); }`
        // (:192-194) - `checkPlayerStatus(param_0, 0, flag)` (`d_com_inf_game.h:2841-2843`).
        if ctx.status.check0(STTS0_SHIP_RIDE) {
            self.ship_whirl(actor, ctx);
        } else {
            // The `else` arm (`:194-204`) is compiled for retail NTSC-U because
            // `DEMO_SELECT(true, ...)` (`:195`) takes the `true` branch above VERSION_DEMO. It calls
            // `daPy_getPlayerLinkActorClass()` (`:196`) and `setWhirlId` - OMITTED.
            self.player_whirl(actor, ctx);
        }

        // `f32 scale_target;` filled at `:208-221`.
        let scale_target = if self.appear == 1 {
            // `if (mbToAppear) { scale_target = 1.0f; } else { scale_target = 0.0f; }` (:209-214)
            if self.to_appear { 1.0 } else { 0.0 }
        } else if ctx.switches.test(self.switch_no) {
            0.0
        } else {
            1.0
        };
        // `cLib_chaseF(&mScaleAnimFactor, scale_target, 0.02f);` (:223)
        walk::chase_f(&mut self.scale, scale_target, SCALE_STEP);
        // `s8 param = s8(100.0f * mScaleAnimFactor); fopAcM_seStart(this, JA_SE_ATM_SWIRL, param);`
        // (:224-225)
        ctx.events.list.push(SimEvent::Se(SE_ATM_SWIRL));

        // The three-way `mKytagPcId` chain (`:227-246`) exists only to create, resize and delete a
        // `kytag01_class` child whose `mWaveInfo.mInnerRadius` / `mOuterRadius` (`:238-239`) are the
        // whirlpool ring itself. `src/sim/kytag01.rs` is a unit component with no radii, and the
        // child is searched by `fopAcM_SearchByID` (`:236`, `:242`) - see OMITTED.
    }

    /// The `else` arm of `_execute` (`:194-204`), which runs when the player is NOT riding the ship.
    ///
    /// `daPy_getPlayerLinkActorClass()` (`:196`) gives Link; the decomp then evaluates
    /// `fopAcM_searchActorDistanceXZ2(this, link_p)` twice - once discarded at `:197`, once in the
    /// `if (... < SQUARE(L_radius * attr().mOuterActivationMult))` at `:198-200` - and its only
    /// effect is `link_p->setWhirlId(fopAcM_GetID(this))` (`:201`). That write targets the player
    /// actor's `mWhirlId` (`d_a_player_main.h:1857`), a field `super::player::Player` does not carry,
    /// so nothing here mutates state. Deliberately empty rather than invented.
    fn player_whirl(&self, _actor: &Actor, _ctx: &Context<'_>) {}

    /// `daObjAuzu::Act_c::ship_whirl` (`:152-176`).
    ///
    /// `daShip_c* ship_p = dComIfGp_getShipActor(); if (ship_p && fopAcM_GetName(ship_p) ==
    /// fpcNm_SHIP_e)` (`:153-154`) - the name test is `Actor::name == "Ship"`
    /// (`d_stage.cpp:650`, `src/sim/ship.rs:12`).
    ///
    /// `f32 inner_activation = SQUARE(L_radius * attr().mInnerActivationMult);` (`:157`) only
    /// chooses between `onWhirlFlg` (`:167`) and `onWhirlFlgDirect` (`:169`), both omitted, so the
    /// inner multiplier has no surviving reader here.
    fn ship_whirl(&mut self, actor: &Actor, ctx: &mut Context<'_>) {
        let Some(ship) = ctx.ship else {
            return;
        };
        // `f32 sqr_mag_xz = fopAcM_searchActorDistanceXZ2(this, ship_p);` (:155)
        let sqr = abs2_xz(ship - actor.pos);
        // `f32 outer_activation = SQUARE(L_radius * attr().mOuterActivationMult);` (:158)
        let outer = L_RADIUS * self.outer_mult();
        // `if (sqr_mag_xz < outer_activation) { ... }` (:163)
        if sqr >= outer * outer {
            return;
        }
        // `if (mScaleAnimFactor > 0.01f) { bgm_start(); ... } else { ship_p->offWhirlFlg(); }`
        // (:164-173). `offWhirlFlg` (`:172`), `onWhirlFlg` (`:167`) and `onWhirlFlgDirect` (`:169`)
        // are all omitted - see the struct doc - but `bgm_start` is real and one-shot.
        if self.scale > ACTIVATE_EPS {
            self.bgm_start(ctx);
        }
    }

    /// `daObjAuzu::Act_c::bgm_start` (`:179-184`): `if (mType == 0 && !mbBgmStarted) {
    /// mbBgmStarted = true; mDoAud_subBgmStart(JA_BGM_DIOCTA_BATTLE); }`.
    fn bgm_start(&mut self, ctx: &mut Context<'_>) {
        if self.kind != 0 || self.bgm_started {
            return;
        }
        self.bgm_started = true;
        ctx.events.list.push(SimEvent::Bgm(BGM_DIOCTA_BATTLE));
    }
}

/// `cXyz::abs2XZ()` - the value `fopAcM_searchActorDistanceXZ2` returns
/// (`f_op_actor_mng.cpp:518-522`, `(*other - *this).abs2XZ()`) and which `d_a_obj_auzu.cpp:155`
/// compares against `SQUARE(...)`.
fn abs2_xz(delta: Vec3) -> f32 {
    delta.x * delta.x + delta.z * delta.z
}

/// Everything `execute` needs beyond the component, in one bundle so the state machine has a single
/// reader - the `hbrf1` / `tagetc` shape.
struct Context<'a> {
    switches: &'a Switches,
    events: &'a mut SimEvents,
    status: &'a PlayerStatus,
    player: Option<Vec3>,
    ship: Option<Vec3>,
}

pub fn is_auzu(name: &str) -> bool {
    AUZU_ACTORS.contains(&name)
}

pub fn control(
    switches: Res<Switches>,
    flags: Res<EventFlags>,
    mut events: ResMut<SimEvents>,
    status: Res<PlayerStatus>,
    players: Query<&Player>,
    mut whirlpools: Query<(Entity, &Actor, &mut Auzu)>,
    ships: Query<&Actor>,
    mut commands: Commands,
) {
    let player = players.iter().next().map(|link| link.pos);
    // `dComIfGp_getShipActor()` is the live ship; the decomp re-checks its name with
    // `fopAcM_GetName(ship_p) == fpcNm_SHIP_e` (`d_a_obj_auzu.cpp:154`).
    let ship = ships
        .iter()
        .find(|actor| actor.name == SHIP_ACTOR)
        .map(|actor| actor.pos);
    let mut gone: Vec<Entity> = Vec::new();
    for (entity, actor, mut auzu) in &mut whirlpools {
        if !auzu.entered {
            auzu.create_init(&switches, &flags);
        }
        if !auzu.exist {
            // `cPhs_ERROR_e` from `_create` (`:65`, `:70`) cancels the actor inside Create, so the
            // entity exists for exactly one tick and is gone on the next (RULE 6b).
            gone.push(entity);
            continue;
        }
        let mut ctx = Context {
            switches: &switches,
            events: &mut events,
            status: &status,
            player,
            ship,
        };
        auzu.execute(actor, &mut ctx);
    }
    for entity in gone {
        let _ = commands.entity(entity).despawn();
    }
}
