
use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::atan2s;
use super::canon::SE_LAST_HIT;
use super::combat::HitPoints;
use super::damage;
use super::events::{SimEvent, SimEvents};
use super::registry::{STATUS_HOOK_CARRY, STATUS_UNK80000, STATUS_UNK200000, actor_angle_y};

/// `dCcD_ObjAtType` from `d_cc_d.h`, a newtype so an attack-type mask cannot be confused with
/// one of the plain `u32` fields around it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct AtkType(pub u32);

impl AtkType {
    /// `cCcD_ObjAt::MskType`: any bit of `mask` set in the attack type.
    pub fn chk(self, mask: AtkType) -> bool {
        self.0 & mask.0 != 0
    }
}

impl std::ops::BitOr for AtkType {
    type Output = AtkType;
    fn bitor(self, rhs: AtkType) -> AtkType {
        AtkType(self.0 | rhs.0)
    }
}

/// `dCcD_ObjAtType`, transcribed with its retail values.
pub const AT_TYPE_SWORD: AtkType = AtkType(1 << 1);
pub const AT_TYPE_UNK8: AtkType = AtkType(1 << 3);
pub const AT_TYPE_BOMB: AtkType = AtkType(1 << 5);
pub const AT_TYPE_BOOMERANG: AtkType = AtkType(1 << 6);
pub const AT_TYPE_BOKO_STICK: AtkType = AtkType(1 << 7);
pub const AT_TYPE_WATER: AtkType = AtkType(1 << 8);
pub const AT_TYPE_FIRE: AtkType = AtkType(1 << 9);
pub const AT_TYPE_MACHETE: AtkType = AtkType(1 << 10);
pub const AT_TYPE_UNK800: AtkType = AtkType(1 << 11);
pub const AT_TYPE_SPIKE: AtkType = AtkType(1 << 12);
pub const AT_TYPE_UNK2000: AtkType = AtkType(1 << 13);
pub const AT_TYPE_NORMAL_ARROW: AtkType = AtkType(1 << 14);
pub const AT_TYPE_HOOKSHOT: AtkType = AtkType(1 << 15);
pub const AT_TYPE_SKULL_HAMMER: AtkType = AtkType(1 << 16);
pub const AT_TYPE_UNK20000: AtkType = AtkType(1 << 17);
pub const AT_TYPE_FIRE_ARROW: AtkType = AtkType(1 << 18);
pub const AT_TYPE_ICE_ARROW: AtkType = AtkType(1 << 19);
pub const AT_TYPE_LIGHT_ARROW: AtkType = AtkType(1 << 20);
pub const AT_TYPE_WIND: AtkType = AtkType(1 << 21);
pub const AT_TYPE_UNK400000: AtkType = AtkType(1 << 22);
pub const AT_TYPE_LIGHT: AtkType = AtkType(1 << 23);
pub const AT_TYPE_STALFOS_MACE: AtkType = AtkType(1 << 24);
pub const AT_TYPE_FAN_SWING: AtkType = AtkType(1 << 25);
pub const AT_TYPE_DARKNUT_SWORD: AtkType = AtkType(1 << 26);
pub const AT_TYPE_GRAPPLING_HOOK: AtkType = AtkType(1 << 27);
pub const AT_TYPE_MOBLIN_SPEAR: AtkType = AtkType(1 << 28);
pub const AT_TYPE_PGANON_SWORD: AtkType = AtkType(1 << 29);
pub const AT_TYPE_ALL: AtkType = AtkType(!0);

/// The `dCcG_At_Spl` value `at_power_check` reads through `GetAtSpl` (`d_cc_d.h:106`).
pub const AT_SPL_UNK0: u8 = 0;
pub const AT_SPL_UNK1: u8 = 1;

/// The `dCcG_hitSe` values `at_se_get` compares `GetAtSe` against (`d_cc_d.h:77`).
pub const AT_SE_NONE: u8 = 0;
pub const AT_SE_WOOD: u8 = 4;
pub const AT_SE_METAL: u8 = 5;
pub const AT_SE_ARROW: u8 = 7;
pub const AT_SE_HOOKSHOT: u8 = 8;

/// The player's cut-type enum, the values the cut bit and the hit-sound ladder read.
pub const CUT_TYPE_NONE: u8 = 0x00;
pub const CUT_TYPE_CUT_A: u8 = 0x01;
pub const CUT_TYPE_CUT_F: u8 = 0x02;
pub const CUT_TYPE_CUT_R: u8 = 0x03;
pub const CUT_TYPE_CUT_L: u8 = 0x04;
pub const CUT_TYPE_BT_JUMPCUT: u8 = 0x05;
pub const CUT_TYPE_CUT_EA: u8 = 0x06;
pub const CUT_TYPE_CUT_EB: u8 = 0x07;
pub const CUT_TYPE_CUT_TURN: u8 = 0x08;
pub const CUT_TYPE_CUT_ROLL: u8 = 0x09;
pub const CUT_TYPE_JUMPCUT_SWORD: u8 = 0x0A;
pub const CUT_TYPE_BT_ROLLCUT: u8 = 0x0F;
pub const CUT_TYPE_BT_VERTICALJUMPCUT: u8 = 0x10;

/// `cc_pl_cut_bit_get`'s bits. `CUT_TYPE_CUT_TURN` and `CUT_TYPE_CUT_ROLL` share one bit.
pub const PL_CUT_NONE: u16 = 0x00;
pub const PL_CUT_A: u16 = 0x01;
pub const PL_CUT_F: u16 = 0x02;
pub const PL_CUT_R: u16 = 0x04;
pub const PL_CUT_L: u16 = 0x08;
pub const PL_CUT_BT_JUMPCUT: u16 = 0x10;
pub const PL_CUT_EA: u16 = 0x20;
pub const PL_CUT_EB: u16 = 0x40;
pub const PL_CUT_TURN: u16 = 0x80;
pub const PL_CUT_JUMPCUT_SWORD: u16 = 0x100;
pub const PL_CUT_BOMB: u16 = 0x200;
pub const PL_CUT_HIMO2: u16 = 0x400;
pub const PL_CUT_BOOMERANG: u16 = 0x800;
pub const PL_CUT_ARROW: u16 = 0x1000;

/// `mResultingAttackType`. The retail field has no enum (`d_cc_uty.h:17` says so), so the values
/// are literal and several branches share one; the doc names every branch that writes each.
pub const RESULT_SWORD: u8 = 0x1;
/// `fpcNm_BOMB_e` and `fpcNm_Bomb2_e`.
pub const RESULT_BOMB: u8 = 0x2;
/// `fpcNm_TSUBO_e` with any type but the water jug.
pub const RESULT_TSUBO: u8 = 0x3;
/// `AT_TYPE_WATER`, and `fpcNm_TSUBO_e` with `TYPE_WATER_JUG`.
pub const RESULT_WATER: u8 = 0x4;
/// `AT_TYPE_UNK20000`, and `AT_TYPE_FIRE | AT_TYPE_FIRE_ARROW`.
pub const RESULT_UNK20000_FIRE: u8 = 0x5;
/// `fpcNm_MT_e`.
pub const RESULT_MT: u8 = 0x6;
/// `fpcNm_MO2_e`.
pub const RESULT_MO2: u8 = 0x7;
/// `AT_TYPE_WIND`.
pub const RESULT_WIND: u8 = 0x8;
/// `fpcNm_PLAYER_e` holding the skull hammer.
pub const RESULT_SKULL_HAMMER: u8 = 0x9;
/// `fpcNm_BOOMERANG_e`.
pub const RESULT_BOOMERANG: u8 = 0xA;
/// `fpcNm_BDK_e`.
pub const RESULT_BDK: u8 = 0xB;
/// Every attack type that reaches no branch above.
pub const RESULT_BASE: u8 = 0xC;
/// `fpcNm_BOMB_e` in `daBomb_c::STATE_8`.
pub const RESULT_BOMB_STATE_8: u8 = 0xD;
/// `fpcNm_HIMO2_e`.
pub const RESULT_HIMO2: u8 = 0xE;

/// `daBomb_c::State_e::STATE_8`, compared by `chk_state`, which is equality.
pub const BOMB_STATE_8: u8 = 8;
/// `daTsubo::Act_c::TYPE_WATER_JUG`.
pub const TSUBO_TYPE_WATER_JUG: u8 = 0x2;
/// `mt_class::m454` for the ball, whose hit sound replaces `at_se_get`'s.
pub const MT_BALL_DMG: u8 = 2;

/// The name an actor is compared by is its process name, not its OBJNAME string, so every OBJNAME
/// row of a process reaches that process's branch below (`d_stage.cpp:518-1248`).
pub const NAME_PLAYER: &str = "Link";
pub const NAME_BOMB: &str = "Bomb";
pub const NAME_BOMB2: &str = "Bomb2";
pub const NAME_HIMO2: &str = "Himo2";
pub const NAME_BOOMERANG: &str = "Boom";
pub const NAME_ARROW: &str = "Arrow";
pub const NAME_BDK: &str = "Bdk";
pub const NAME_MO2: &str = "mo2";
pub const NAME_MT: &str = "magtail";
pub const NAME_ST: &str = "Stal";

/// `fpcNm_TSUBO_e` has twelve OBJNAME rows (`d_stage.cpp:519-530`) and they are all
/// `daTsubo::Act_c`, so all twelve reach the tsubo branch of the name chain.
pub const TSUBO_ACTORS: [&str; 12] = [
    "kotubo", "ootubo1", "Kmtub", "Ktaru", "Ostool", "Odokuro", "Okioke", "Kmi02", "Ptubo",
    "KkibaB", "Kmi00", "Hbox2S",
];

/// The `seStart` parameter argument: `0x33` for the `fpcNm_ST_e` target, `0x20` otherwise.
const SE_ARG_ST: u32 = 0x33;
const SE_ARG: u32 = 0x20;

/// `atInfo->mpActor->speed.abs()` above which `cc_at_check` takes the knockback direction from
/// the attacker's velocity instead of from the target's position.
const SPEED_FOR_ANGLE: f32 = 50.0;

/// `REG0_S(i)` is `g_regHIO.mChild[0].mShortRegs[i]` (`d_s_play.h:183`), an `s16` that is zero in
/// the retail default build.
const REG0_S5: i16 = 0;
const REG0_S6: i16 = 0;
const REG0_S7: i16 = 0;

/// The status bits a hookshot is harmless against: `STATUS_UNK80000 | STATUS_HOOK_CARRY |
/// STATUS_UNK200000` read off the struck actor.
const HOOK_CARRY_STATUS: u32 = STATUS_UNK80000 | STATUS_HOOK_CARRY | STATUS_UNK200000;

/// `fpcNm_MO2_e`'s damage cap, `damage::KILL_POWER`.
const MO2_DAMAGE_CAP: u8 = damage::KILL_POWER as u8;
/// `fpcNm_MT_e`'s forced damage, `damage::MEGA_SWORD`.
const MT_DAMAGE: u8 = damage::MEGA_SWORD as u8;

/// The `JA_SE_*` ids the two hit-sound ladders return.
const SE_LK_HAMMER_HIT: u32 = 0x2855;
const SE_LK_W_WEP_HIT: u32 = 0x2833;
const SE_LK_MS_WEP_HIT: u32 = 0x2834;
const SE_LK_ARROW_HIT: u32 = 0x2879;
const SE_LK_HS_SPIKE: u32 = 0x286F;
const SE_LK_SW_HIT_S: u32 = 0x2803;
const SE_LK_W_WEP_CRT_HIT: u32 = 0x2835;
const SE_LK_MS_WEP_CRT_HIT: u32 = 0x2836;
const SE_LK_SW_CRT_HIT: u32 = 0x2806;
const SE_OBJ_MG_BALL_DMG: u32 = 0x693A;

/// `dPa_name::ID_AK_JN_OK` (`d_particle_name.h:46`), the normal hit mark.
pub const PARTICLE_OK: u16 = 0x000D;
/// `dPa_name::ID_AK_JN_CRITICALHITFLASH` (`d_particle_name.h:49`), the flash the death branch puts
/// under the mark.
pub const PARTICLE_CRITICALHITFLASH: u16 = 0x0010;

/// `mpObj->GetAc()`, the attacking actor, as the two functions read it: the name the process
/// name is compared against, the position and speed, and the two downcasts `at_power_check`
/// makes through the pointer.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Attacker {
    pub name: String,
    pub pos: Vec3,
    pub speed: Vec3,
    /// `daBomb_c::prm_get_state`.
    pub bomb_state: u8,
    /// `daTsubo::Act_c::prm_get_type`.
    pub tsubo_type: u8,
}

/// `mpObj`, a `cCcD_Obj`, reduced to the four AT fields these utilities read plus the actor
/// `GetAc()` hands back.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Atk {
    /// `ChkAtType`.
    pub at_type: AtkType,
    /// `GetAtSe`: a `dCcG_SE_*` value.
    pub at_se: u8,
    /// `GetAtSpl`: a `dCcG_At_Spl_*` value.
    pub at_spl: u8,
    /// `GetAtAtp`: the AT object's own damage, which `at_power_check` reads before any override.
    pub at_atp: u8,
    /// `GetAc()`.
    pub ac: Option<Attacker>,
}

impl Atk {
    /// `ChkAtType`.
    pub fn chk_at_type(&self, mask: AtkType) -> bool {
        self.at_type.chk(mask)
    }
}

/// `tgActor`, the struck actor, as `cc_at_check` reads it: the name and position, the room the
/// steal-item spawn is created in, the status bits, `health`, the steal bookkeeping, and the one
/// actor-class field the function casts to.
#[derive(Debug, Clone, Default)]
pub struct Target {
    pub name: String,
    pub pos: Vec3,
    pub room: i8,
    /// `actor_status`, read by [`Target::chk_status`].
    pub status: u32,
    /// `health`; `s8` in the base class, the port's own component here.
    pub health: HitPoints,
    /// `stealItemLeft`.
    pub steal_item_left: i8,
    /// `stealItemBitNo`.
    pub steal_item_bit_no: u8,
    /// `itemTableIdx`.
    pub item_table_idx: i32,
    /// `mt_class::m454`.
    pub m454: u8,
}

impl Target {
    /// The status-bit test, `actor_status & flag`.
    pub fn chk_status(&self, flag: u32) -> bool {
        self.status & flag != 0
    }
}

/// `CcAtInfo` from `d_cc_uty.h`: what `at_power_check` computes and `cc_at_check` charges.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CcAtInfo {
    /// `mpObj`, the attack object. `None` is the `mpObj == NULL` bail of `at_power_check`.
    pub mp_obj: Option<Atk>,
    /// `mpActor`, the attacking actor, written by `at_power_check`.
    pub mp_actor: Option<Attacker>,
    /// `mDamage`. A `u8` that `cc_at_check` tests as `s8`.
    pub m_damage: u8,
    /// `mbDead`.
    pub mb_dead: bool,
    /// `mResultingAttackType`, a `RESULT_*` value.
    pub m_resulting_attack_type: u8,
    /// `m0C`, an angle triple; only `.y` is ever written, the yaw the knockback is aimed from.
    pub m0c: [i16; 3],
    /// `mPlCutBit`.
    pub m_pl_cut_bit: u16,
    /// `pParticlePos`, the point the death branch's hit mark is put at.
    pub p_particle_pos: Option<Vec3>,
    /// `mHitSoundId`: the argument to `mDoAud_bgmHitSound`, which is a hit-sound selector
    /// rather than a sound id.
    pub m_hit_sound_id: i32,
}

/// `dComIfGp_getPlayer(0)` as the two functions read it: `getCutType`, `getCutCount`, and the
/// position the player-angle search measures from the struck actor.
#[derive(Resource, Debug, Clone, Copy, Default, PartialEq)]
pub struct PlayerCut {
    pub cut_type: u8,
    pub cut_count: u8,
    pub pos: Vec3,
}

/// `dScnPly_ply_c::setPauseTimer`: the scene player's hit-stop, which only a sword-class attack
/// (`mResultingAttackType == 1`) ever sets.
#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HitStop {
    pub pause_timer: i8,
}

impl HitStop {
    pub fn set_pause_timer(&mut self, frames: i8) {
        self.pause_timer = frames;
    }
}

/// One sound start: the id `SimEvent::Se` carries, the `u32` parameter both overloads take, and
/// the position the positional overload was given (`None` for the actor-anchored one).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HitSe {
    pub id: u32,
    pub param: u32,
    pub pos: Option<Vec3>,
}

/// The `mDoAud_*` calls `cc_at_check` makes: the argumentless `mDoAud_onEnemyDamage`, the
/// `mDoAud_bgmHitSound` whose argument is the hit-sound selector `m_hit_sound_id` rather than a
/// music id, and every sound start with the arguments `SimEvent::Se` leaves out.
#[derive(Resource, Debug, Clone, Default, PartialEq)]
pub struct HitAudio {
    pub enemy_damage: bool,
    pub bgm_hit_sound: Option<i32>,
    pub se: Vec<HitSe>,
}

impl HitAudio {
    /// `mDoAud_onEnemyDamage()`.
    pub fn on_enemy_damage(&mut self) {
        self.enemy_damage = true;
    }

    /// `mDoAud_bgmHitSound(id)`.
    pub fn bgm_hit_sound(&mut self, id: i32) {
        self.bgm_hit_sound = Some(id);
    }

    /// The actor-anchored sound start.
    pub fn se_start(&mut self, events: &mut SimEvents, id: u32, param: u32) {
        self.se.push(HitSe {
            id,
            param,
            pos: None,
        });
        events.list.push(SimEvent::Se(id));
    }

    /// `mDoAud_seStart(id, se_pos, param, reverb)`.
    pub fn se_start_p(&mut self, events: &mut SimEvents, id: u32, pos: Vec3, param: u32) {
        self.se.push(HitSe {
            id,
            param,
            pos: Some(pos),
        });
        events.list.push(SimEvent::Se(id));
    }
}

/// The presentation `cc_at_check` produces that `SimEvent` has no variant for: the death
/// branch's two hit marks, and the steal-item spawn.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DamageFx {
    /// `dComIfGp_particle_set`. `angle` and `scale` are `None` for the overload that takes
    /// neither.
    Particle {
        id: u16,
        pos: Vec3,
        angle: Option<Vec3>,
        scale: Option<Vec3>,
    },
    /// The steal-item create call, recorded with the bit number the reference passes before it
    /// increments the actor's.
    StealItem {
        pos: Vec3,
        table: i32,
        room: i8,
        item_bit_no: u8,
    },
}

/// `cc_pl_cut_bit_get` (`d_cc_uty.cpp:15`): the player's cut type as the bit the struck actor's
/// AI reads. `CUT_TYPE_CUT_TURN` and `CUT_TYPE_CUT_ROLL` share `PL_CUT_TURN`, and every cut type
/// outside the ladder is `PL_CUT_NONE`.
pub fn cc_pl_cut_bit_get(player: &PlayerCut) -> u16 {
    match player.cut_type {
        CUT_TYPE_CUT_A => PL_CUT_A,
        CUT_TYPE_CUT_F => PL_CUT_F,
        CUT_TYPE_CUT_R => PL_CUT_R,
        CUT_TYPE_CUT_L => PL_CUT_L,
        CUT_TYPE_BT_JUMPCUT => PL_CUT_BT_JUMPCUT,
        CUT_TYPE_CUT_EA => PL_CUT_EA,
        CUT_TYPE_CUT_EB => PL_CUT_EB,
        CUT_TYPE_CUT_TURN | CUT_TYPE_CUT_ROLL => PL_CUT_TURN,
        CUT_TYPE_JUMPCUT_SWORD => PL_CUT_JUMPCUT_SWORD,
        _ => PL_CUT_NONE,
    }
}

/// `at_se_get` (`d_cc_uty.cpp:41`): the hit sound for an attack that did not kill.
pub fn at_se_get(obj: &Atk) -> u32 {
    if obj.chk_at_type(AT_TYPE_SKULL_HAMMER | AT_TYPE_STALFOS_MACE) {
        SE_LK_HAMMER_HIT
    } else if obj.at_se == AT_SE_WOOD {
        SE_LK_W_WEP_HIT
    } else if obj.at_se == AT_SE_METAL {
        SE_LK_MS_WEP_HIT
    } else if obj.at_se == AT_SE_ARROW {
        SE_LK_ARROW_HIT
    } else if obj.at_se == AT_SE_HOOKSHOT {
        SE_LK_HS_SPIKE
    } else {
        SE_LK_SW_HIT_S
    }
}

/// `at_se_getC` (`d_cc_uty.cpp:59`): `at_se_get` with the critical-hit fall-through, differing
/// only in the wood, metal and default ids.
pub fn at_se_get_c(obj: &Atk) -> u32 {
    if obj.chk_at_type(AT_TYPE_SKULL_HAMMER | AT_TYPE_STALFOS_MACE) {
        SE_LK_HAMMER_HIT
    } else if obj.at_se == AT_SE_WOOD {
        SE_LK_W_WEP_CRT_HIT
    } else if obj.at_se == AT_SE_METAL {
        SE_LK_MS_WEP_CRT_HIT
    } else if obj.at_se == AT_SE_ARROW {
        SE_LK_ARROW_HIT
    } else if obj.at_se == AT_SE_HOOKSHOT {
        SE_LK_HS_SPIKE
    } else {
        SE_LK_SW_CRT_HIT
    }
}

/// `def_se_set` (`d_cc_uty.cpp:77`): the actor-anchored sound start of `at_se_get(obj)`, so the
/// actor is the one thing the call has that [`def_se_set_p`] does not.
pub fn def_se_set(obj: &Atk, r5: u32, audio: &mut HitAudio, events: &mut SimEvents) {
    audio.se_start(events, at_se_get(obj), r5);
}

/// `def_se_set_p` (`d_cc_uty.cpp:82`): the positional `mDoAud_seStart(at_se_get(obj), se_pos, r6,
/// reverb)`, where the reverb is the one of the actor's room.
pub fn def_se_set_p(
    obj: &Atk,
    se_pos: Vec3,
    r6: u32,
    audio: &mut HitAudio,
    events: &mut SimEvents,
) {
    audio.se_start_p(events, at_se_get(obj), se_pos, r6);
}

/// `at_power_check` (`d_cc_uty.cpp:87`): what the attack object is worth.
///
/// The AT power is read before the per-attacker overrides, so the two can only ever lower or
/// replace it, and `mbDead` is cleared before the test that sets it.
pub fn at_power_check(at_info: &mut CcAtInfo, player: &PlayerCut) -> Option<Attacker> {
    let obj = at_info.mp_obj.clone()?;
    at_info.mp_actor = obj.ac.clone();
    at_info.m_resulting_attack_type = RESULT_BASE;
    at_info.m_damage = 0;

    if obj.chk_at_type(AT_TYPE_WATER) {
        at_info.m_resulting_attack_type = RESULT_WATER;
        return at_info.mp_actor.clone();
    } else if obj.chk_at_type(AT_TYPE_WIND) {
        at_info.m_resulting_attack_type = RESULT_WIND;
        return at_info.mp_actor.clone();
    } else if obj.chk_at_type(AT_TYPE_UNK20000) {
        at_info.m_resulting_attack_type = RESULT_UNK20000_FIRE;
        return at_info.mp_actor.clone();
    } else if obj.chk_at_type(AT_TYPE_FIRE | AT_TYPE_FIRE_ARROW) {
        at_info.m_resulting_attack_type = RESULT_UNK20000_FIRE;
    }

    if let Some(attacker) = &obj.ac {
        at_info.m_damage = obj.at_atp;
        match attacker.name.as_str() {
            NAME_PLAYER => {
                if obj.chk_at_type(AT_TYPE_SKULL_HAMMER) {
                    at_info.m_resulting_attack_type = RESULT_SKULL_HAMMER;
                } else {
                    at_info.m_resulting_attack_type = RESULT_SWORD;
                    at_info.m_pl_cut_bit = cc_pl_cut_bit_get(player);
                    at_info.m_hit_sound_id = if player.cut_count == 3 {
                        1
                    } else if player.cut_count == 4 {
                        4
                    } else if player.cut_type == CUT_TYPE_JUMPCUT_SWORD {
                        2
                    } else if matches!(
                        player.cut_type,
                        CUT_TYPE_BT_JUMPCUT | CUT_TYPE_BT_ROLLCUT | CUT_TYPE_BT_VERTICALJUMPCUT
                    ) {
                        3
                    } else {
                        0
                    };
                }
            }
            NAME_BOMB => {
                at_info.m_resulting_attack_type = if attacker.bomb_state == BOMB_STATE_8 {
                    RESULT_BOMB_STATE_8
                } else {
                    RESULT_BOMB
                };
                at_info.m_pl_cut_bit = PL_CUT_BOMB;
            }
            NAME_BOMB2 => {
                at_info.m_resulting_attack_type = RESULT_BOMB;
                at_info.m_pl_cut_bit = PL_CUT_BOMB;
            }
            NAME_HIMO2 => {
                at_info.m_resulting_attack_type = RESULT_HIMO2;
                at_info.m_pl_cut_bit = PL_CUT_HIMO2;
            }
            NAME_BOOMERANG => {
                at_info.m_resulting_attack_type = RESULT_BOOMERANG;
                at_info.m_pl_cut_bit = PL_CUT_BOOMERANG;
            }
            NAME_ARROW => {
                at_info.m_pl_cut_bit = PL_CUT_ARROW;
            }
            NAME_BDK => {
                at_info.m_resulting_attack_type = RESULT_BDK;
            }
            name if TSUBO_ACTORS.contains(&name) => {
                at_info.m_resulting_attack_type = if attacker.tsubo_type == TSUBO_TYPE_WATER_JUG {
                    RESULT_WATER
                } else {
                    RESULT_TSUBO
                };
            }
            NAME_MO2 => {
                at_info.m_resulting_attack_type = RESULT_MO2;
                if at_info.m_damage > MO2_DAMAGE_CAP {
                    at_info.m_damage = MO2_DAMAGE_CAP;
                }
            }
            NAME_MT => {
                at_info.m_resulting_attack_type = RESULT_MT;
                at_info.m_damage = MT_DAMAGE;
            }
            _ => {}
        }
    }

    at_info.mb_dead = false;
    let attacker_is_player = obj
        .ac
        .as_ref()
        .is_some_and(|attacker| attacker.name == NAME_PLAYER);
    if attacker_is_player {
        if obj.at_spl == AT_SPL_UNK1 {
            at_info.mb_dead = true;
        }
    } else if at_info.m_damage >= MO2_DAMAGE_CAP {
        at_info.mb_dead = true;
    }

    at_info.mp_actor.clone()
}

/// `cc_at_check` (`d_cc_uty.cpp:188`): charge the attack to the struck actor, aim the knockback,
/// steal from a `fpcNm_HIMO2_e`, and pick the hit sound, the hit mark and the hit-stop.
pub fn cc_at_check(
    target: &mut Target,
    at_info: &mut CcAtInfo,
    player: &PlayerCut,
    events: &mut SimEvents,
    stop: &mut HitStop,
    audio: &mut HitAudio,
    fx: &mut Vec<DamageFx>,
) -> Option<Attacker> {
    at_info.m_hit_sound_id = 0;
    at_power_check(at_info, player);
    let Some(obj) = at_info.mp_obj.clone() else {
        return None;
    };
    let Some(attacker) = at_info.mp_actor.clone() else {
        return None;
    };

    audio.on_enemy_damage();

    let (along, across) = if attacker.speed.length() > SPEED_FOR_ANGLE {
        (attacker.speed.x, attacker.speed.z)
    } else {
        (target.pos.x - attacker.pos.x, target.pos.z - attacker.pos.z)
    };
    at_info.m0c[1] = atan2s(-along, -across);

    if obj.chk_at_type(AT_TYPE_HOOKSHOT) && target.chk_status(HOOK_CARRY_STATUS) {
        at_info.m_damage = 0;
    }

    if attacker.name == NAME_HIMO2 && target.steal_item_left != 0 {
        target.steal_item_left = target.steal_item_left.wrapping_sub(1);
        fx.push(DamageFx::StealItem {
            pos: target.pos,
            table: target.item_table_idx,
            room: target.room,
            item_bit_no: target.steal_item_bit_no,
        });
        target.steal_item_bit_no = target.steal_item_bit_no.wrapping_add(1);
        at_info.m_damage = 0;
    }

    if at_info.m_damage as i8 > 0 {
        target.health.0 -= i32::from(at_info.m_damage);
    }

    let pause_time;
    if target.health.0 <= 0 {
        at_info.mb_dead = true;

        audio.se_start(events, SE_LAST_HIT, 0);

        pause_time = (6 + REG0_S7) as i8;

        if let Some(pos) = at_info.p_particle_pos {
            fx.push(DamageFx::Particle {
                id: PARTICLE_CRITICALHITFLASH,
                pos,
                angle: None,
                scale: None,
            });
            let player_angle = actor_angle_y(target.pos, player.pos);
            fx.push(DamageFx::Particle {
                id: PARTICLE_OK,
                pos,
                angle: Some(Vec3::new(0.0, f32::from(player_angle), 0.0)),
                scale: Some(Vec3::splat(2.0)),
            });
        }
    } else {
        let se_arg = if target.name == NAME_ST { SE_ARG_ST } else { SE_ARG };
        if at_info.mb_dead {
            audio.se_start(events, at_se_get_c(&obj), se_arg);
            pause_time = if at_info.m_resulting_attack_type == RESULT_SKULL_HAMMER {
                6
            } else {
                (4 + REG0_S6) as i8
            };
        } else {
            if target.name == NAME_MT && target.m454 == MT_BALL_DMG {
                audio.se_start(events, SE_OBJ_MG_BALL_DMG, 0);
            } else {
                audio.se_start(events, at_se_get(&obj), se_arg);
            }
            pause_time = (1 + REG0_S5) as i8;
        }
    }

    if at_info.m_resulting_attack_type == RESULT_SWORD {
        stop.set_pause_timer(pause_time);
    }

    if at_info.m_damage != 0 {
        audio.bgm_hit_sound(at_info.m_hit_sound_id);
    }

    Some(attacker)
}
