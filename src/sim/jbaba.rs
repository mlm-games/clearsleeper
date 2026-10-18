use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::events::{SimEvent, SimEvents};
use super::flags::EventFlags;
use super::player::Player;
use super::player_status::{PlayerStatus, STTS0_UNK80, STTS0_UNK80000000};
use super::walk;

/// The stage string is `jbaba` (`d_stage.cpp:697`); `_arcname` is `"JBO"`
/// (`d_a_jbo.cpp:79`, `:96`, `:164`, `:167`, `:192`, `:196`, `:244`, `:250`), which is why the
/// lowercase stage string - not the archive name - is what this array carries.
pub const JBABA_ACTORS: [&str; 1] = ["jbaba"];

/// `#define JUMP_ANIMATION_TIME 70` (`d_a_jbo.cpp:15`) - the `mFramesUntilJump` seed of `:83` and
/// the origin of `mAnimationSpeed` at `:89`.
const JUMP_ANIMATION_TIME: i16 = 70; // <- d_a_jbo.cpp:15
/// `mType = fopAcM_GetParam(a_this)` narrowed into the `u8 mType` field (`d_a_jbo.h:21`).
const TYPE_MASK: u32 = 0xFF; // <- d_a_jbo.h:21
/// `if (a_this->mType == 0xFF) a_this->mType = daJbo_Type_NORMAL_e` (`d_a_jbo.cpp:199-201`).
const TYPE_NONE: u8 = 0xFF; // <- d_a_jbo.cpp:199
/// `cM3dGSphS.Radius` of `co_sph_src` (`d_a_jbo.cpp:234`).
const CO_SPH_RADIUS: f32 = 15.0; // <- d_a_jbo.cpp:234
/// `mCoSph.SetR(55.0f)` (`d_a_jbo.cpp:143`).
const CO_RADIUS: f32 = 55.0; // <- d_a_jbo.cpp:143
/// `pos.y += 30.0` before `mCoSph.SetC(pos)` (`d_a_jbo.cpp:141-142`).
const CO_OFFSET_Y: f32 = 30.0; // <- d_a_jbo.cpp:141
/// `dComIfGs_isEventBit(dSv_event_flag_c::UNK_1801)` (`d_a_jbo.cpp:125`) - byte-indexed per
/// RULE 11c, so the retail id is passed through unshifted.
const EVENT_BIT_1801: u32 = 0x1801; // <- d_save_event_flag.inc:188 (d_a_jbo.cpp:125)
/// `JA_SE_OBJ_JFLOWER_IN` (`JAZelAudio_SE.h:1855`).
const SE_JFLOWER_IN: u32 = 0x6946; // <- JAZelAudio_SE.h:1855
/// `JA_SE_OBJ_JFLOWER_OUT` (`JAZelAudio_SE.h:1856`).
const SE_JFLOWER_OUT: u32 = 0x6947; // <- JAZelAudio_SE.h:1856
/// `JA_SE_CM_BV_BASE_POPUP` (`JAZelAudio_SE.h:1187`).
const SE_BV_BASE_POPUP: u32 = 0x5843; // <- JAZelAudio_SE.h:1187

/// `enum daJbo_Type` (`d_a_jbo.h:9-14`) - the `mType` selector, decoded from the whole
/// parameter (RULE 7: `fopAcM_GetParam` returns all 32 bits) and narrowed by the `u8` field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// `daJbo_Type_NORMAL_e = 0` (`d_a_jbo.h:10`).
    Normal,
    /// `daJbo_Type_POPS_UP_e = 1` (`d_a_jbo.h:11`) - plays `BCK_UMARERU1` and enters
    /// `daJbo_Mode_SPAWN_e` on Create (`:243-248`).
    PopsUp,
    /// `daJbo_Type_UNK_2_e = 2` (`d_a_jbo.h:12`) - the only other arm is `m2BA = 1` (`:106`).
    Unk2,
    /// `daJbo_Type_APPEAR_AFTER_DEKU_TREE_e = 3` (`d_a_jbo.h:13`) - inert until
    /// `dSv_event_flag_c::UNK_1801`, then `mCoSph.OnCoSetBit()` (`:124-129`).
    AfterDekuTree,
}

/// `enum daJbo_Mode` (`d_a_jbo.cpp:17-21`), the `switch (i_this->mMode)` of `jbo_move` (`:76`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// `daJbo_Mode_IDLE_e = 0` (`d_a_jbo.cpp:18`).
    Idle,
    /// `daJbo_Mode_WAIT_JUMP_e = 1` (`d_a_jbo.cpp:19`).
    WaitJump,
    /// `daJbo_Mode_SPAWN_e = 2` (`d_a_jbo.cpp:20`).
    Spawn,
}

/// The Baba Bud flower.
///
/// `co_sph_src` (`d_a_jbo.cpp:209-236`) has `SrcObjAt Type = 0` (`:212`), `SrcObjAt SPrm = 0`
/// (`:214`), `SrcObjTg Type = 0` (`:215`) and `SrcObjTg SPrm = 0` (`:216`): it is a Co-only sphere
/// whose `SrcObjCo SPrm = cCcD_CoSPrm_Set_e | cCcD_CoSPrm_IsOther_e | cCcD_CoSPrm_VsEnemy_e`
/// (`:218`) makes the bud a solid body for whoever walks into it. No damage crosses it in either
/// direction and `jbo_class` (`d_a_jbo.h:16-38`) has no health field; the profile `Group` is
/// `fopAc_ENV_e` (`d_a_jbo.cpp:278`).
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE.
///
/// `daJBO_Create` (`:189-254`) can only return `cPhs_ERROR_e` from a failed `fopAcM_entrySolidHeap`
/// (`:196`) or `useHeapInit` (`:178-180`), never from `mType`, and `daJBO_Execute` (`:122-147`) ends
/// on an unconditional TRUE - so there is no spawn-then-despawn path. `daJBO_IsDelete` (`:150-152`)
/// is the bare "accepts a delete request" form of RULE 6b.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Baba {
    /// `mType` (`d_a_jbo.h:21`).
    pub kind: Kind,
    /// `mMode` (`d_a_jbo.h:25`).
    pub mode: Mode,
    /// `mFramesUntilJump` (`d_a_jbo.h:26`) - seeded to `JUMP_ANIMATION_TIME` at `:83` and
    /// decremented at `:131-133`.
    pub frames_until_jump: i16,
    /// `false` until the first `Execute`, so Create's `mCoSph` is what `jbo_move` still sees.
    entered: bool,
    /// `fopAcM_seStart(JA_SE_CM_BV_BASE_POPUP)` at `:246` is a Create-time effect and `spawn`
    /// cannot reach `SimEvents`.
    pending_popup: bool,
}

impl Baba {
    /// `daJBO_Create` (`:189-254`), minus `fopAcM_ct_Retail`/`fopAcM_ct_Demo`, `dComIfG_resLoad`,
    /// `fopAcM_entrySolidHeap`/`useHeapInit`, `fopAcM_SetMtx`, `mStts.Init`, `mCoSph.Set`,
    /// `mCoSph.SetStts`, `mCoSph.ClrCoSet`, the three `setAnm` calls, `jbo_draw_SUB`, the emitter's
    /// `setGlobalAlpha` and `gbaName` - archive, renderer and particle work. What survives is the
    /// `mType` decode (`:198-205`) and the `POPS_UP` arm's `mMode = daJbo_Mode_SPAWN_e` (`:243-248`).
    ///
    /// `REG8_S(9)` at `:203-205` folds away under RULE 2b: `g_regHIO.mChild[*]` is zero-initialised
    /// at `d_s_play.cpp:81` and only per-stage HIO data - absent from this checkout - ever overwrites
    /// it, so `REG8_S(9) != 0` is a proven false.
    pub fn spawn(parameters: u32) -> Self {
        let raw = (parameters & TYPE_MASK) as u8;
        let kind = match raw {
            1 => Kind::PopsUp,
            2 => Kind::Unk2,
            3 => Kind::AfterDekuTree,
            TYPE_NONE => Kind::Normal,
            _ => Kind::Normal,
        };
        Self {
            kind,
            mode: if kind == Kind::PopsUp {
                Mode::Spawn
            } else {
                Mode::Idle
            },
            frames_until_jump: 0,
            entered: false,
            pending_popup: kind == Kind::PopsUp,
        }
    }

    /// `mCoSph.SetC(pos)` / `mCoSph.SetR(55.0f)` / `dComIfG_Ccsp()->Set(&mCoSph)`
    /// (`d_a_jbo.cpp:140-144`) - the collider `jbo_move`'s `mCoSph.ChkCoHit()` (`:78`) reads. Note
    /// the ordering in `daJBO_Execute`: `jbo_move` runs at `:134-138`, the `SetC`/`SetR` at
    /// `:140-144`, so on the very first tick `ChkCoHit` still sees `co_sph_src`'s own
    /// `Center {0,0,0}` / `Radius 15` from `mCoSph.Set(co_sph_src)` (`:237`) - a live one-tick
    /// collider sitting at the world origin.
    fn collider_centre(&self, pos: Vec3) -> Vec3 {
        if self.entered {
            pos + Vec3::new(0.0, CO_OFFSET_Y, 0.0)
        } else {
            Vec3::ZERO
        }
    }

    /// `mCoSph.ChkCoHit()` (`d_a_jbo.cpp:78`) against `SrcObjCo SPrm` with
    /// `cCcD_CoSPrm_IsOther_e` (`:218`) - the bud is a solid body, so the only Co it can meet is
    /// Link's. `APPEAR_AFTER_DEKU_TREE_e` never reaches here: `mCoSph.ClrCoSet()` (`:241`) clears
    /// the set bit and `Execute` returns at `:129` before `jbo_move` while that type stands.
    fn co_hit(&self, pos: Vec3, player: &Player) -> bool {
        let radius = if self.entered {
            CO_RADIUS
        } else {
            CO_SPH_RADIUS
        };
        walk::within(player.pos, self.collider_centre(pos), radius)
    }

    /// The `daJbo_Mode_IDLE_e` arm of `jbo_move` (`:77-87`):
    /// `dComIfGp_checkPlayerStatus0(0, daPyStts0_UNK80_e) && i_this->mCoSph.ChkCoHit()`. The
    /// `mpMorf->setAnm(BCK_IN1)` at `:80` is renderer work; `fopAcM_seStart(JA_SE_OBJ_JFLOWER_IN)`
    /// (`:81`) is ported; `dComIfGp_setItemMagicCount(4)` (`:82`) is omitted.
    ///
    /// `dComIfGp_setItemMagicCount` accumulates into `mItemMagicCount` (`d_com_inf_game.h:422`,
    /// `:2753-2755`), the `s16` the game-info play record carries (`:797`). Its only reader is
    /// `dMeter2_c::move`'s magic arm, `dComIfGs_getMagic() + dComIfGp_getItemMagicCount()`
    /// (`d_meter.cpp:4220-4221`), and this port models neither that record slot nor the player's
    /// own magic pair (`dComIfGs_getMagic` / `getMaxMagic`), so the four units the bud hands over
    /// have no field to land in.
    fn mode_idle(
        &mut self,
        pos: Vec3,
        player: &Player,
        status: &PlayerStatus,
        events: &mut SimEvents,
    ) {
        if !status.check0(STTS0_UNK80) || !self.co_hit(pos, player) {
            return;
        }
        events.list.push(SimEvent::Se(SE_JFLOWER_IN));
        self.frames_until_jump = JUMP_ANIMATION_TIME;
        self.mode = Mode::WaitJump;
    }

    /// The `daJbo_Mode_WAIT_JUMP_e` arm of `jbo_move` (`:88-111`).
    /// `mAnimationSpeed`/`mAnimRotation` (`:89-90`, `:99-100`) and `mParticlePos` (`:101-104`) are
    /// read only by `jbo_draw_SUB` and the particle system, `mpMorf->setAnm(BCK_OUT1)` (`:97`) is
    /// renderer work, `m2BA = 1` (`:106`) is written but never read anywhere in the decomp, and
    /// `player->onForceVomitJump()` (`:91-94`) is omitted: it is
    /// `onNoResetFlg1(daPyFlg1_FORCE_VOMIT_JUMP)` (`d_a_player.h:555`, bit `0x10` at `:203`), a
    /// player no-reset flag whose main reader is `daPy_lk_c::checkNextMode`, which enters
    /// `procVomitJump_init(2)` on it (`d_a_player_main.cpp:4417-4419`), with the process itself in
    /// `d_a_player_vomit.inc:101-105`. This port carries no `mNoResetFlg1` word - the one player
    /// latch it does carry is `daPyFlg0_NO_DRAW` ([`Player::no_draw`]) - and its `link_move`
    /// reducer has no vomit process: the `Proc::Vomit*` entries `link_proc.rs` declares are the
    /// reference's own table rows, not a running arm.
    /// `mFramesUntilJump == 1` therefore has no reader in this port; the countdown itself is
    /// transcribed by `execute`.
    fn mode_wait_jump(&mut self, status: &PlayerStatus, events: &mut SimEvents) {
        if !status.check0(STTS0_UNK80000000) {
            return;
        }
        events.list.push(SimEvent::Se(SE_JFLOWER_OUT));
        self.mode = Mode::Idle;
    }

    /// The `daJbo_Mode_SPAWN_e` arm of `jbo_move` (`:112-117`), whose only test is
    /// `mpMorf->isStop()` (`:113`) - a BCK frame state this checkout does not carry (RULE 3b), so
    /// the arm cannot leave `SPAWN_e` here.
    fn mode_spawn(&mut self) {}

    /// `jbo_move` (`:74-119`).
    fn step(&mut self, pos: Vec3, player: &Player, status: &PlayerStatus, events: &mut SimEvents) {
        match self.mode {
            Mode::Idle => self.mode_idle(pos, player, status, events),
            Mode::WaitJump => self.mode_wait_jump(status, events),
            Mode::Spawn => self.mode_spawn(),
        }
    }

    /// `daJBO_Execute` (`:122-147`). The `mType == daJbo_Type_APPEAR_AFTER_DEKU_TREE_e` arm
    /// (`:124-130`) returns before anything else: on `dSv_event_flag_c::UNK_1801` the type becomes
    /// `NORMAL_e` and `mCoSph.OnCoSetBit()` re-enables the set bit `mCoSph.ClrCoSet()` cleared at
    /// `:241`. Everything after - `if (mFramesUntilJump != 0) mFramesUntilJump--` (`:131-133`), the
    /// `switch (i_this->m2BB) { case 0: jbo_move(i_this); }` (`:134-138`), `mpMorf->play` (`:139`),
    /// the Co re-registration (`:140-144`) and `jbo_draw_SUB` (`:145`) - is reached only by the
    /// other variants. `m2BB` is never written anywhere in the decomp, so the switch is the
    /// unconditional `jbo_move` call that `:136` is.
    fn execute(
        &mut self,
        pos: Vec3,
        player: &Player,
        status: &PlayerStatus,
        flags: &EventFlags,
        events: &mut SimEvents,
    ) {
        if self.pending_popup {
            self.pending_popup = false;
            events.list.push(SimEvent::Se(SE_BV_BASE_POPUP));
        }
        if self.kind == Kind::AfterDekuTree {
            if flags.test(EVENT_BIT_1801) {
                self.kind = Kind::Normal;
            }
            return;
        }
        if self.frames_until_jump != 0 {
            self.frames_until_jump -= 1;
        }
        // `jbo_move` runs at `:136` while `mCoSph.SetC`/`SetR` only land at `:140-144`, so the
        // collider `step` reads is always the one the previous tick installed.
        self.step(pos, player, status, events);
        self.entered = true;
    }
}

pub fn is_jbaba(name: &str) -> bool {
    JBABA_ACTORS.contains(&name)
}

pub fn control(
    status: Res<PlayerStatus>,
    flags: Res<EventFlags>,
    mut events: ResMut<SimEvents>,
    players: Query<&Player>,
    mut buds: Query<(&Actor, &mut Baba)>,
) {
    let Some(player) = players.iter().next().copied() else {
        return;
    };
    for (actor, mut bud) in &mut buds {
        bud.execute(actor.pos, &player, &status, &flags, &mut events);
    }
}
