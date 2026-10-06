//! Dishes and cutlery inside Forsaken Fortress - `fpcNm_Obj_Mshokki_e`, `d_a_obj_mshokki.cpp`,
//! stage strings `MPot`, `MOsara` and `MKoppu` (`d_stage.cpp:1131-1133`, all three rows bound to
//! the one profile name `fpcNm_Obj_Mshokki_e`). These are the MODEL names, not a filename:
//! `bdl_idx[]` is `{ dRes_INDEX_MSHOKKI_BDL_POT_e, dRes_INDEX_MSHOKKI_BDL_OSARA_e,
//! dRes_INDEX_MSHOKKI_BDL_KOPPU_e }` (`d_a_obj_mshokki.cpp:144`).
//!
//! (a) inbound damage: NONE. (b) outbound damage: NONE.
//!
//! `l_cyl_src` sets `SrcObjAt Type` to `0` (`d_a_obj_mshokki.cpp:67`), so this actor presents NO
//! attack type at all: nothing in retail can register a hit against it and it cannot damage
//! anything. Neither `combat::HitPoints` nor `enemy::Hurt` is attached.
//!
//! `SrcObjTg Type` IS live - it is `AT_TYPE_ALL & ~AT_TYPE_WATER & ~AT_TYPE_UNK20000 &
//! ~AT_TYPE_UNK400000 & ~AT_TYPE_LIGHT` (`:70`) - and `checkCollision()` (`:161-177`) genuinely
//! calls `mCyl.ChkTgHit()`. Both of its arms are unreachable in this sim. The `if` arm tests
//! `pcVar3->ChkAtType(AT_TYPE_WIND)` and RULE 3b lists `AT_TYPE_WIND` collider semantics as
//! confirmed-absent ("five retail emitters, none ported as targets"); the `else` arm calls
//! `break_proc()` on a non-wind target hit, which needs the `cCcD` actor-collision registry. Both
//! are therefore transcribed as a `false` return, which is exactly the value retail returns when
//! `ChkTgHit()` is false - the only state this sim can represent - and keeps `_execute`'s
//! `if (!checkCollision())` arm (`:351`) taken exactly as retail takes it on a clean frame.
//! `co_hitCallback` (`:180-218`) is omitted for the same reason: it is invoked only through
//! `mCyl`'s own Co-hit callback, set at `:288`.
//!
//! `daObjMshokki_IsDelete { return TRUE; }` (`:395-397`) only ACCEPTS a delete request per
//! RULE 6b. The real destruction is `break_proc()`'s `fopAcM_delete(this)` (`:228`), reached from
//! `_execute` (`:343-344`) once the piece has touched down and then settled, and that IS modelled
//! as a despawn.
//!
//! `L_HIO`'s members are `static const f32` (`d_a_obj_mshokki.cpp:43-58`), not HIO-writable
//! registers, so unlike `REG0_F` they fold to their literals exactly. Only the non-DEMO members
//! are transcribed; the `#if VERSION == VERSION_DEMO` arms at `:14-41`, `:107-126`, `:184-190`,
//! `:212-216`, `:236-240`, `:295-299` and `:307-312` are excluded because retail is
//! `VERSION_NTSC_U`.
//!
//! The profile's `Cull Type` is `fopAc_CULLBOX_0_e` (`:421`), NOT custom, so the box resolves
//! through the shared table at `f_op_actor_mng.cpp:558`: min `(-40.0, 0.0, -40.0)`, max
//! `(40.0, 125.0, 40.0)`. That is a resolved literal, not an invented one.
//!
//! ONE MODELLED SIMPLIFICATION, stated plainly: `dBgS_ObjAcch` distinguishes `ChkGroundHit()`
//! (contact this frame) from `ChkGroundLanding()` (settled), via per-poly-group ground splitters
//! that RULE 3b lists as absent from this checkout. Both are therefore resolved to the single
//! `ground::grounded` test, which is what `fallrock.rs` uses for the same `ChkGroundLanding` call.
//! The affected retail expressions are `:326`, `:343` and `:346`. The consequence is that a piece
//! sets `m612` on its first ground-contact tick and breaks on the following one, which keeps
//! retail's two-tick bounce-then-shatter shape.
//!
//! `mAcch`'s accommodation setup is omitted wholesale: `mAcchCir.SetWall(50.0f, 35.0f)` (`:266`),
//! `mAcch.SetRoofCrrHeight(50.0f)` (`:279`), `mAcch.ClrWaterNone()` (`:277`),
//! `mAcch.ClrRoofNone()` (`:278`) and `mAcch.ClrGroundLanding()` (`:281`) all configure
//! `dBgS_AcchCir`/`dBgS_ObjAcch`, i.e. the per-poly-group ground splitters that RULE 3b lists as
//! absent. `mCyl.SetR(l_data[m60C].radius)` and `mCyl.SetH(l_data[m60C].height)` (`:286-287`)
//! are carried in the `l_data` table above and used by the omitted collider.

use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::collision::Collision;
use super::events::{SimEvent, SimEvents};
use super::ground;

/// `d_stage.cpp:1131-1133` binds `MPot`, `MOsara` and `MKoppu` to `fpcNm_Obj_Mshokki_e`, the
/// profile of `d_a_obj_mshokki.cpp:407-422`.
pub const MSHOKKI_ACTORS: [&str; 3] = ["MPot", "MOsara", "MKoppu"];

/// `L_HIO` (`d_a_obj_mshokki.cpp:43-58`), all `static const f32`.
const HIO_M10: f32 = 128.0;
const HIO_M14: f32 = 3072.0;
const HIO_M18: f32 = 4096.0;
const HIO_M3C: f32 = 0.1;
const HIO_M38: f32 = 0.8;

/// `l_data[]` (`d_a_obj_mshokki.cpp:100-104`), indexed by `m60C` as
/// `{ m00, radius, height, m0C }`.
const POT: [f32; 4] = [20.0, 15.0, 40.0, 6144.0];
const OSARA: [f32; 4] = [7.5, 30.0, 15.0, 2048.0];
const KOPPU: [f32; 4] = [15.0, 15.0, 30.0, 8192.0];

/// `if (m60C < 0 || m60C >= 3)` (`d_a_obj_mshokki.cpp:255`). `param_get_arg()` is
/// `daObj::PrmAbstract(PRM_ARG_W=2, PRM_ARG_S=0)` (`d_a_obj_mshokki.h:16-18`), i.e.
/// `((1 << 2) - 1) & (param >> 0)` (`d_a_obj.h:14`), so the decoded value is only ever 0..=3 and
/// the `3` case really is reachable and really is folded to `0` by `:255-258`.
const ARG_COUNT: i32 = 3;

/// `fopAcM_SetGravity(this, -6.0f)` (`d_a_obj_mshokki.cpp:289`).
pub const GRAVITY: f32 = -6.0;

/// The shared `fopAc_CULLBOX_0_e` profile at `f_op_actor_mng.cpp:558`.
pub const CULL_MIN: [f32; 3] = [-40.0, 0.0, -40.0];
pub const CULL_MAX: [f32; 3] = [40.0, 125.0, 40.0];

/// `se_flag[]` (`d_a_obj_mshokki.cpp:233`):
/// `{ JA_SE_OBJ_CUP_SWING, JA_SE_OBJ_DISH_SWING, JA_SE_OBJ_CUP_SWING }`.
const SE_FLAG: [u32; 3] = [0x69E7, 0x69E8, 0x69E7];

/// `l_data[i]` for the dish index `m60C`.
fn dish(kind: u8) -> [f32; 4] {
    match kind {
        1 => OSARA,
        2 => KOPPU,
        _ => POT,
    }
}

/// `daObj::PrmAbstract(PRM_ARG_W=2, PRM_ARG_S=0)` (`d_a_obj.h:14`) with the out-of-range fold at
/// `d_a_obj_mshokki.cpp:255-258`.
fn param_arg(parameters: u32) -> u8 {
    let arg = (parameters & 0x3) as i32;
    if arg < 0 || arg >= ARG_COUNT {
        0
    } else {
        arg as u8
    }
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct Mshokki {
    /// `m60C`, set in `_create` from `param_get_arg()` (`d_a_obj_mshokki.cpp:254`). Indexed into
    /// `bdl_idx[]` (`:147`), `l_data[]` (`:286-287`), `se_flag[]` (`:233`, `:245`) and
    /// `particle_id[]` (`:222`, `:224`).
    kind: u8,
    /// `m612` (`:343`, `:346`, `:348`). Set once the piece has touched the ground, and then it
    /// shatters the moment it settles.
    touched: bool,
    /// `m61C` (`:215`), written only by the omitted `co_hitCallback` and read only by `set_se` at
    /// `:235`. Held at its zeroed value.
    swing_base: i16,
    /// Set on the first tick so `_create`'s side effects run once, as `andsw0::Andsw0::create_init`
    /// does.
    entered: bool,
    /// Set once `break_proc()`'s `fopAcM_delete(this)` (`:228`) has been issued.
    broken: bool,
}

impl Mshokki {
    /// `_create` (`:250-301`). The profile's `Parameters` field is `0`
    /// (`d_a_obj_mshokki.cpp:419`) and the only parameter read is `param_get_arg()`, decoded here.
    pub fn spawn(parameters: u32) -> Self {
        Self {
            kind: param_arg(parameters),
            touched: false,
            swing_base: 0,
            entered: false,
            broken: false,
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `fopAcM_SetGravity(this, -6.0f)` (`d_a_obj_mshokki.cpp:289`).
    pub fn gravity(&self) -> f32 {
        GRAVITY
    }

    /// `_create` (`:250-301`), the parts that are not renderer-only.
    fn create_init(&mut self, actor: &mut Actor) {
        actor.gravity = GRAVITY;
    }

    /// `checkCollision()` (`:161-177`), transcribed as always `false`; see the header.
    fn check_collision(&mut self) -> bool {
        false
    }

    /// `set_se()` (`:232-247`). The `s16 uVar4 = (shape_angle.x / l_data[m60C].m0C) * 0x4000;
    /// if (uVar4 > 0x4000) uVar4 = 0x4000;` pair (`:241-244`) feeds only the second argument of
    /// `fopAcM_seStartCurrent`, where `cM_ssin(uVar4) * 100.0f` becomes the packed `param_2`
    /// handed straight to `mDoAud_seStart` (`f_op_actor_mng.h:675-677`). `super::events::SimEvent::Se`
    /// carries the id alone, so the clamp and the volume are transcribed as citations only and the
    /// sound fires on exactly the same condition as retail.
    fn set_se(&self, shape_angle: [i16; 3], events: &mut SimEvents) {
        let delta = i32::from(shape_angle[1]) - i32::from(self.swing_base);
        if delta.unsigned_abs() >= HIO_M18 as u32 {
            return;
        }
        events.list.push(SimEvent::Se(SE_FLAG[self.kind as usize]));
    }

    /// `break_proc()` (`:221-229`). The particle (`:224`), the SE whose second argument is
    /// `dComIfG_Bgsp()->GetMtrlSndId(mAcch.m_gnd)` (`:225`) and `dKy_Sound_set(pos, 150, id, 5)`
    /// (`:227`) are all omitted; `fopAcM_delete(this)` (`:228`) is the modelled despawn.
    fn break_proc(&mut self) {
        self.broken = true;
    }

    /// `attention_info.position` (`d_a_obj_mshokki.cpp:356-358`): the current position lifted by
    /// `l_data[m60C].m00`.
    fn attention_point(&self, pos: Vec3) -> Vec3 {
        pos + Vec3::Y * dish(self.kind)[0]
    }

    /// `_execute()` (`:317-364`).
    fn execute(&mut self, actor: &mut Actor, solid: Option<&Collision>, events: &mut SimEvents) {
        // `fopAcM_posMoveF(this, mStts.GetCCMoveP())` (`:318`); `mAcch` is configured with the
        // gravity-enabled flag `1` (`:270`), and `mAcch.CrrPos(*dComIfG_Bgsp())` (`:319`) is the
        // merged-DZB collision pass.
        actor.fall();
        actor.integrate();

        actor.speed_f *= HIO_M38;
        if actor.speed_f < HIO_M3C {
            actor.speed_f = 0.0;
        }

        let down = ground::grounded(solid, actor.pos);

        if down {
            if i32::from(actor.shape_angle[0]).unsigned_abs() > HIO_M10 as u32 {
                // `shape_angle.x += -(s16)HIO(m10); shape_angle.y += (s16)HIO(m18);` (`:328-329`)
                actor.shape_angle[0] = actor.shape_angle[0].wrapping_sub(HIO_M10 as i16);
                actor.shape_angle[1] = actor.shape_angle[1].wrapping_add(HIO_M18 as i16);
                let shape = actor.shape_angle;
                self.set_se(shape, events);
            } else {
                actor.shape_angle[0] = 0;
            }
        } else {
            // `shape_angle.x += -(s32)HIO(m14)` (`:335`) - an s32 sum truncated back into the
            // s16 field, which `as i16` reproduces exactly.
            actor.shape_angle[0] = (i32::from(actor.shape_angle[0]) - HIO_M14 as i32) as i16;
        }

        if self.touched && down {
            self.break_proc();
        } else {
            if down {
                // `if (mAcch.ChkGroundHit() && !mAcch.ChkGroundLanding()) m612 = 1;` (`:346-348`)
                self.touched = true;
            }
            if !self.check_collision() {
                // `set_mtx()` (`:352`), `mCyl.MoveCTg(current.pos)` (`:353`),
                // `mStts.SetRoomId(current.roomNo)` (`:354`) and
                // `dComIfG_Ccsp()->Set(&mCyl)` (`:355`) are collider-registry work and are omitted;
                // `:359` `eyePos = attention_info.position;` is the actor's own field.
                actor.eye = self.attention_point(actor.pos);
            }
        }
    }
}

pub fn is_mshokki(name: &str) -> bool {
    MSHOKKI_ACTORS.contains(&name)
}

pub fn control(
    collision: Option<Res<Collision>>,
    mut events: ResMut<SimEvents>,
    mut dishes: Query<(Entity, &mut Actor, &mut Mshokki)>,
    mut commands: Commands,
) {
    let solid = collision.as_deref();
    for (entity, mut actor, mut dish) in &mut dishes {
        if dish.broken {
            let _ = commands.entity(entity).despawn();
            continue;
        }
        if !dish.entered {
            dish.entered = true;
            dish.create_init(&mut actor);
            continue;
        }
        dish.execute(&mut actor, solid, &mut events);
    }
}
