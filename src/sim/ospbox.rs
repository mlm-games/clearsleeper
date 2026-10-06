use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::bombflower::Blasts;
use super::carry::Carryable;
use super::events::{SimEvent, SimEvents};

/// The stage string is `Ospbox` (`d_stage.cpp:904`, `fpcNm_Obj_Ospbox_e`), which is also
/// `daObjOspbox::Act_c::M_arcname` (`d_a_obj_ospbox.cpp:16`).
pub const OSPBOX_ACTORS: [&str; 1] = ["Ospbox"];

/// `enum Prm_e { PRM_ITEMNO_W = 0x06, PRM_ITEMNO_S = 0x00, PRM_SPEC_W = 0x03, PRM_SPEC_S = 0x08 }`
/// (`d_a_obj_ospbox.h:11-16`) resolved through `daObj::PrmAbstract(this, W, S)` =
/// `((1 << W) - 1) & (param >> S)` (`d_a_obj.h:14`).
const ITEMNO_WIDTH: u32 = 0x06; // <- d_a_obj_ospbox.h:12 (PRM_ITEMNO_W)
const ITEMNO_SHIFT: u32 = 0x00; // <- d_a_obj_ospbox.h:13 (PRM_ITEMNO_S)
const SPEC_WIDTH: u32 = 0x03; // <- d_a_obj_ospbox.h:14 (PRM_SPEC_W)
const SPEC_SHIFT: u32 = 0x08; // <- d_a_obj_ospbox.h:15 (PRM_SPEC_S)

/// `make_item`'s `static const s16 pf_name[]` (`d_a_obj_ospbox.cpp:123-132`). `prm_get_spec()` is
/// three bits (`:140`), so only indices 0..=7 are ever read - the eight entries the decomp lists.
const PF_NAME: [&str; 8] = [
    "Ospbox", // <- d_a_obj_ospbox.cpp:124 (fpcNm_Obj_Ospbox_e)
    "Pig",   // <- d_a_obj_ospbox.cpp:125 (fpcNm_KB_e)
    "Pig",   // <- d_a_obj_ospbox.cpp:126
    "Pig",   // <- d_a_obj_ospbox.cpp:127
    "Ospbox", // <- d_a_obj_ospbox.cpp:128
    "Ospbox", // <- d_a_obj_ospbox.cpp:129
    "Ospbox", // <- d_a_obj_ospbox.cpp:130
    "Ospbox", // <- d_a_obj_ospbox.cpp:131
];
/// `fpcNm_KB_e`'s stage string, `OBJNAME("Pig", fpcNm_KB_e, 255, 0)` (`d_stage.cpp:798`).
const PIG_ACTOR: &str = "Pig"; // <- d_stage.cpp:798 (fpcNm_KB_e)

/// `make_item`'s `static const u32 prm[]` (`d_a_obj_ospbox.cpp:133-138`). The table has four
/// entries and `prm[prm_index]` (`:151`) is only reached when `pf_name[prm_index] != fpcNm_Obj_
/// Ospbox_e`, which is exactly `prm_index` 1..=3 - so retail never reads past the end.
const PIG_PRM: [u32; 4] = [
    0x0000_0000, // <- d_a_obj_ospbox.cpp:134
    0x01FF_0000, // <- d_a_obj_ospbox.cpp:135
    0x01FF_0001, // <- d_a_obj_ospbox.cpp:136
    0x01FF_0002, // <- d_a_obj_ospbox.cpp:137
];
/// `daNpc_Fa1_c`-free `daObjOspbox` index of the first `Pig` entry, so `PIG_PRM` is read only at a
/// real index.
const PIG_PRM_BASE: usize = 1; // <- d_a_obj_ospbox.cpp:125-127 (the three fpcNm_KB_e rows)

/// `M_cyl_src`'s `cM3dGCylS` (`d_a_obj_ospbox.cpp:40-44`) - `Center {0,0,0}`, `Radius 75.0f`,
/// `Height 150.0f`; `mCyl.MoveCAtTg(*&current.pos)` (`:222`) keeps the axis on the box.
const CYL_RADIUS: f32 = 75.0; // <- d_a_obj_ospbox.cpp:42
const CYL_HEIGHT: f32 = 150.0; // <- d_a_obj_ospbox.cpp:43

/// `fopAcM_setCullSizeBox(this, -76.0f, -1.0f, -76.0f, 76.0f, 151.0f, 76.0f)`
/// (`d_a_obj_ospbox.cpp:60`) - a real custom cull box, so the profile's
/// `fopAc_CULLBOX_CUSTOM_e` (`:298`) resolves.
const CULL_MIN: [f32; 3] = [-76.0, -1.0, -76.0]; // <- d_a_obj_ospbox.cpp:60
const CULL_MAX: [f32; 3] = [76.0, 151.0, 76.0]; // <- d_a_obj_ospbox.cpp:60

/// `m49A = 2` (`d_a_obj_ospbox.cpp:73`), the `u8 m49A` (`d_a_obj_ospbox.h:44`) `Execute` drains
/// into `mpBgW->SetLock()` (`:220`).
const LOCK_DELAY: u8 = 2; // <- d_a_obj_ospbox.cpp:73

/// `sound_break`'s `fopAcM_seStart(this, JA_SE_OBJ_SURP_BOX_BREAK, sound_id)`
/// (`d_a_obj_ospbox.cpp:184`). `JA_SE_OBJ_SURP_BOX_BREAK = 0x69B6` (`JAZelAudio_SE.h:1959`).
const SE_OBJ_SURP_BOX_BREAK: u32 = 0x69B6; // <- JAZelAudio_SE.h:1959

/// The bomb-breakable surprise box - `daObjOspbox::Act_c` (`d_a_obj_ospbox.h:9-45`), the actor
/// behind the stage string `Ospbox`.
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. `M_cyl_src` (`d_a_obj_ospbox.cpp:17-45`)
/// sets `SrcObjAt Type = 0` (`:20`), `SrcObjAt Atp = 0` (`:21`) and `SrcObjAt SPrm = 0` (`:22`), so
/// the box owns no AT collider and cannot hurt anything; its `SrcObjTg Type = AT_TYPE_BOMB` (`:24`)
/// with `cCcD_TgSPrm_Set_e | cCcD_TgSPrm_IsOther_e` (`:25`) is a pure trigger - a bomb blast
/// *senses* it and the box reacts, damage flows nowhere - and its `SrcObjCo SPrm = 0` (`:26`) means
/// it registers no solid either. The profile `Group` is `fopAc_ACTOR_e` (`:297`). Neither
/// `combat::HitPoints` nor `enemy::Hurt` is attached.
///
/// `Mthd_Execute` is `MoveBGExecute()` (`:261`), which forwards to the virtual `Execute(Mtx**)`, so
/// `Execute` (`:212-231`) is the real per-tick body. `Mthd_IsDelete` (`:270-272`) is
/// `dBgS_MoveBgActor::IsDelete()`. `Mthd_Create` (`:80-93`) can only return `cPhs_ERROR_e` from
/// `MoveBGCreate` on a heap or `dComIfG_Bgsp()` registration failure, and returns
/// `cPhs_COMPLEATE_e` otherwise (`:92`) - `Create` (`:56-75`) never refuses - so there is no
/// spawn-then-despawn path.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ospbox {
    /// `prm_get_spec()` = `daObj::PrmAbstract(this, PRM_SPEC_W, PRM_SPEC_S)`
    /// (`d_a_obj_ospbox.h:21-23`), the `make_item` branch selector (`:140`).
    pub spec: u8,
    /// `prm_get_itemNo()` = `daObj::PrmAbstract(this, PRM_ITEMNO_W, PRM_ITEMNO_S)`
    /// (`d_a_obj_ospbox.h:18-20`), the item the box drops (`:142`).
    pub item_no: u8,
    /// `m49A` (`d_a_obj_ospbox.h:44`), seeded to 2 by `Create` (`:73`) and drained by `Execute`
    /// (`:219-221`).
    pub lock: u8,
    /// `m498` (`d_a_obj_ospbox.h:43`), the `init_ground` retry counter (`:207`). Only ever feeds the
    /// `dBgS_ObjGndChk` shadow probe, which RULE 3b excludes; it is carried so the retry cadence
    /// is not silently shortened.
    pub ground_tries: i16,
    /// One-shot guard so the `Create` stores that need no live input are applied once.
    pub entered: bool,
}

impl Ospbox {
    /// `Create` (`:56-75`) plus `Mthd_Create` (`:80-93`), minus `fopAcM_SetMtx` (`:57`),
    /// `init_mtx` (`:58`, `:116-119`), `fopAcM_setCullSizeBox` (`:60`), `mStts.Init` (`:62`),
    /// `mCyl.Set` / `mCyl.SetStts` / `mCyl.SetTgVec` / `mCyl.OnTgNoHitMark` (`:63-66`), the
    /// attention block (`:68-69`), `init_ground` (`:71`, `:206-209`), `dComIfG_resLoad` (`:82`) and
    /// `MoveBGCreate` (`:89`) - matrix, collider, attention, ground-probe, archive and `dBgW` work.
    /// What survives is the two parameter decodes and `m49A = 2`.
    pub fn spawn(parameters: u32) -> Self {
        Self {
            spec: ((parameters >> SPEC_SHIFT) & ((1 << SPEC_WIDTH) - 1)) as u8,
            item_no: ((parameters >> ITEMNO_SHIFT) & ((1 << ITEMNO_WIDTH) - 1)) as u8,
            lock: LOCK_DELAY,
            // `init_ground`'s `m498 = 5` (`:207`)
            ground_tries: GROUND_TRIES,
            entered: false,
        }
    }

    /// `fopAcM_setCullSizeBox(this, -76.0f, -1.0f, -76.0f, 76.0f, 151.0f, 76.0f)`
    /// (`d_a_obj_ospbox.cpp:60`) - the actor's one real custom cull box.
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `Execute`'s `if (mCyl.ChkTgHit())` (`:213`) for the box's own `AT_TYPE_BOMB` Tg cylinder,
    /// which `mCyl.MoveCAtTg(*&current.pos)` (`:222`) keeps on `current.pos`. `ChkTgHit` is
    /// "some registered AT object overlaps me", and the sim's blast channel is exactly the bomb's
    /// published origin - the same reduction `leaves.rs` makes for the same `AT_BOMB` bit, but
    /// against `M_cyl_src`'s cylinder rather than a sphere. `cM3d_CylAttr::CalcAabBox`
    /// (`SSystem/SComponent/c_cc_d.cpp:438-447`) spans the cylinder from its `Center` to
    /// `Center.y + h`, and `scale.x` is 1.0 for a stage-placed actor (RULE 2), so neither is scaled.
    fn bomb_touches(hit: Vec3, base: Vec3) -> bool {
        let dx = hit.x - base.x;
        let dz = hit.z - base.z;
        if CYL_RADIUS * CYL_RADIUS < dx * dx + dz * dz {
            return false;
        }
        hit.y >= base.y && hit.y <= base.y + CYL_HEIGHT
    }

    /// `make_item` (`:122-159`). `pf_name[prm_index] == fpcNm_Obj_Ospbox_e` (`:141`) selects the item
    /// arm; every other index is a `fpcNm_KB_e` spawn with `prm[prm_index]`, whose decoded fields
    /// are `mShapeType = (param >> 0) & 0xF` (`d_a_kb.cpp:2473`), `m401 = (param >> 4) & 0xF`
    /// (`d_a_kb.cpp:2474`), `mbCanBeBigPig = param >> 8` (`d_a_kb.cpp:2475`) and
    /// `m41D = param >> 0x10` (`d_a_kb.cpp:2476`) - i.e. pig shapes 0, 1 and 2. `m41D = 0x1FF` makes
    /// `kb_class::_create` take `mpPath = dPath_GetRoomPath(0x1FF, room)` (`d_a_kb.cpp:2488-2489`),
    /// the absent room-`PATH` block of
    /// RULE 3b, so the pig's route motion is not modelled - but the actor itself is.
    fn make_item(&self, actor: &Actor, commands: &mut Commands) {
        let index = usize::from(self.spec);
        let Some(name) = PF_NAME.get(index) else {
            return;
        };
        if *name == PF_NAME[0] {
            // `fopAcM_createItemFromTable(&current.pos, item_no, 0x7F, fopAcM_GetHomeRoomNo(this),
            // daItemType_0_e, &current.angle, daItemAct_7_e, NULL)` (`:143-149`). That resolves
            // `i_itemNo` through the absent `dComIfGp_getItemTable()` for 0x20..=0x3E
            // (`f_op_actor_mng.cpp:817-826`), refuses `0x3F` outright (`:892`), and otherwise runs
            // `check_itemno` (`f_op_actor_mng.cpp:1020`) - the bow / bomb / bait-bag / spoils-bag ownership rewrite,
            // which needs `dComIfGs_isGetItem`'s got-item save table and `dComIfGs_getMaxMagic`.
            // `prm_get_itemNo()` is six bits, so the provable range is 0x00..=0x1F plus the 0x3F
            // refusal, and the middle band has no provable item number at all: no item is spawned.
            return;
        }
        let Some(parameters) = PIG_PRM.get(index.wrapping_sub(PIG_PRM_BASE)) else {
            return;
        };
        commands.spawn((
            Actor {
                name: PIG_ACTOR.to_owned(),
                parameters: *parameters,
                pos: actor.pos,
                angle: actor.angle,
                set_id: 0,
                room: actor.room,
            },
            Carryable::Pig,
        ));
    }

    /// `eff_break` (`:162-175`) is a `dPa_name::ID_IT_JN_TR_HAHEN_B` emitter with a 30-frame life
    /// and `setAwayFromAxisSpeed(30.0f)` - particles only (RULE 3b), with no sim state.
    ///
    /// `sound_break` (`:178-185`) plays `JA_SE_OBJ_SURP_BOX_BREAK`; its second argument is
    /// `dComIfG_Bgsp()->GetMtrlSndId(mObjGndChk)` (`:182`), a per-poly-group material query that
    /// RULE 3b excludes, so only the sound id is emitted.
    fn sound_break(&self, events: &mut SimEvents) {
        events.list.push(SimEvent::Se(SE_OBJ_SURP_BOX_BREAK));
    }

    /// `Execute` (`:212-231`), minus `set_mtx` (`:227`), `*o_mtx = &M_tmp_mtx` (`:229`),
    /// `mCyl.MoveCAtTg` / `dComIfG_Ccsp()->Set` (`:222-223`) and `set_ground` (`:226`). Returns
    /// true when `fopAcM_delete(this)` (`:217`) fires.
    ///
    /// The `else` arm's `if (m49A != 0 && --m49A == 0) mpBgW->SetLock();` (`:219-221`) drains the
    /// counter - `m49A` is carried for that - but `SetLock` locks the absent `dBgW` moving
    /// background, so nothing observable follows.
    fn execute(
        &mut self,
        actor: &Actor,
        blasts: &Blasts,
        events: &mut SimEvents,
        commands: &mut Commands,
    ) -> bool {
        if blasts
            .list
            .iter()
            .any(|origin| Self::bomb_touches(*origin, actor.pos))
        {
            self.make_item(actor, commands);
            self.sound_break(events);
            return true;
        }
        if self.lock != 0 {
            self.lock -= 1;
        }
        false
    }

    /// `set_ground` (`:188-203`): `if (m498 <= 0) return;` then the `dBgS_ObjGndChk` probe at
    /// `current.pos + (0, 100.0f, 0)` (`:193`) whose `GroundCross` result `m494` (`:197`) is
    /// compared against `-G_CM3D_F_INF` to decide whether to keep retrying. `GroundCross` is the
    /// per-poly-group query RULE 3b excludes, and `m494`'s only reader is the
    /// `dComIfGd_setSimpleShadow2` shadow in `Draw` (`:242`) - renderer work - so the comparison is
    /// dropped and only the retry countdown runs.
    fn set_ground(&mut self) {
        if self.ground_tries <= 0 {
            return;
        }
        self.ground_tries -= 1;
    }

    /// The `Create` stores that need no live input: `m498 = 5` via `init_ground` (`:207`) and the
    /// one-shot guard for the matrix work `Create` does (`:57-58`).
    fn create_init(&mut self) {
        self.entered = true;
    }
}

/// `init_ground`'s `m498 = 5` (`d_a_obj_ospbox.cpp:207`).
const GROUND_TRIES: i16 = 5; // <- d_a_obj_ospbox.cpp:207

pub fn is_ospbox(name: &str) -> bool {
    OSPBOX_ACTORS.contains(&name)
}

pub fn control(
    blasts: Res<Blasts>,
    mut events: ResMut<SimEvents>,
    mut boxes: Query<(Entity, &Actor, &mut Ospbox)>,
    mut commands: Commands,
) {
    let mut gone: Vec<Entity> = Vec::new();
    for (entity, actor, mut box_) in &mut boxes {
        if !box_.entered {
            box_.create_init();
        }
        if box_.execute(actor, &blasts, &mut events, &mut commands) {
            gone.push(entity);
        }
        box_.set_ground();
    }
    for entity in gone {
        let _ = commands.entity(entity).despawn();
    }
}