use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::jbaba;

/// The stage string is `Ojtree` (`d_stage.cpp:903`, `fpcNm_Obj_Ojtree_e`). It happens to match
/// `daObjOjtree::Act_c::M_arcname`, which is also `"Ojtree"` (`d_a_obj_ojtree.cpp:14`) - unlike
/// `Lwood`/`lwood` and `Oyashi`/`Oyashi`, which differ.
pub const OJTREE_ACTORS: [&str; 1] = ["Ojtree"];

/// `fopAcM_setCullSizeBox(this, -500.0f, -1.0f, -300.0f, 251.0f, 5001.0f, 251.0f)`
/// (`d_a_obj_ojtree.cpp:30`). The profile's `fopAc_CULLBOX_CUSTOM_e` (`:147`) resolves to this, so
/// a real box exists.
const CULL_MIN: [f32; 3] = [-500.0, -1.0, -300.0]; // <- d_a_obj_ojtree.cpp:30
const CULL_MAX: [f32; 3] = [251.0, 5001.0, 251.0]; // <- d_a_obj_ojtree.cpp:30

/// `pos.y += 5000.0f` (`d_a_obj_ojtree.cpp:32`) - the height at which the `JBO` actor is created,
/// well above the pole so the Deku Leaf's jump-flower sits out of reach of ordinary play.
const LEAF_LIFT: f32 = 5000.0; // <- d_a_obj_ojtree.cpp:32

/// `daJbo_Type_NORMAL_e = 0` (`d_a_jbo.h:10`) - the `i_parameter` of
/// `fopAcM_create` (`d_a_obj_ojtree.cpp:33`), i.e. `jbo_class::mType` (`d_a_jbo.h:20`).
const JBO_TYPE_NORMAL: u32 = 0; // <- d_a_obj_ojtree.cpp:33 (daJbo_Type_NORMAL_e, d_a_jbo.h:10)

/// The tall thin poles that lead up to the Deku Leaf in Forest Haven - `daObjOjtree::Act_c`
/// (`d_a_obj_ojtree.h:8-23`), the actor behind the stage string `Ojtree`.
///
/// `daObjOjtree::Act_c::Create` (`:25-36`) does exactly two things that leave the sim: it sets the
/// cull box and it calls
/// `fopAcM_create(fpcNm_JBO_e, daJbo_Type_NORMAL_e, &pos, fopAcM_GetHomeRoomNo(this), &shape_angle,
/// NULL, 0xff, NULL)` (`:33`) with `pos.y` lifted by 5000 (`:32`). That creates a `jbaba` - the
/// jump flower - high above the pole, and `jbaba.rs` models its whole behaviour
/// (`d_a_jbo.cpp:73-135`). `init_mtx` (`:72-75`), `set_mtx` (`:64-69`) and `Draw` (`:92-99`) are
/// J3D model and tev-strategy work.
///
/// `mLockTimer` (`:34`, `:79-85`) exists only to reach `mpBgW->SetLock()`, and `mpBgW` is a
/// `dBgW` moving background (RULE 3b), so neither the field nor its countdown survives.
///
/// `Mthd_IsDelete` is `dBgS_MoveBgActor::MoveBGIsDelete` (`:119-121`), which is
/// `return IsDelete()` (`d_bg_s_movebg_actor.h:28`) and the class overrides neither - the base
/// `IsDelete()` is `return TRUE` (`d_bg_s_movebg_actor.h:35`). RULE 6b: this is "accepts a delete
/// request", never a self-destruct. `Mthd_Create` (`:39-49`) can return `cPhs_ERROR_e` only from
/// `MoveBGCreate` (`:44`), which is heap work.
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. `daObjOjtree::Act_c` is a
/// `dBgS_MoveBgActor` holding only `mPhs`, `mpModel` and `mLockTimer`
/// (`d_a_obj_ojtree.h:20-22`) - it registers no `dCcD` collider of any kind, and the profile
/// `Group` is `fopAc_ACTOR_e` (`:146`). Neither `combat::HitPoints` nor `enemy::Hurt` is attached.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ojtree {
    /// One-shot guard so `Create`'s `fopAcM_create` (`:33`) happens exactly once, on the first
    /// tick, where `Commands` is available. `fopAcM_ct` (`:40`) and `MoveBGCreate` (`:44`) are not
    /// modelled; `setCullSizeBox` (`:30`) is a constant carried by `cull()`.
    pub entered: bool,
}

impl Ojtree {
    /// `daObjOjtree::Act_c::Create` (`:25-36`), minus `fopAcM_SetMtx` / `init_mtx` (`:28-29`),
    /// the cull box (`:30`, a constant) and the `fopAcM_create` call (`:32-33`), which is deferred
    /// to the first tick because it needs `Commands`. `M_arcname` (`:14`) is an archive name, and
    /// `daObjOjtree::Act_c` carries no `fopAcM_prm_class` parameter - the profile's `Parameters`
    /// field is `0` (`:141`) - so there is no parameter to decode.
    pub fn spawn(_parameters: u32) -> Self {
        Self { entered: false }
    }

    /// `fopAcM_setCullSizeBox(this, -500.0f, -1.0f, -300.0f, 251.0f, 5001.0f, 251.0f)`
    /// (`d_a_obj_ojtree.cpp:30`) - the actor's one real custom cull box, matching the profile's
    /// `fopAc_CULLBOX_CUSTOM_e` (`:147`).
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `daObjOjtree::Act_c::Execute` (`:78-89`).
    ///
    /// `if (mLockTimer != 0) { if (--mLockTimer == 0) mpBgW->SetLock(); }` (`:79-85`) is dropped
    /// with the field: `mpBgW` is a `dBgW` (RULE 3b) and `MoveBGExecute` (`:111-113`) calls
    /// `mpBgW->Move()` anyway. `set_mtx()` (`:86`) and `*pMtx = &M_tmp_mtx` (`:87`) are renderer
    /// work. `mLockTimer = 2` (`:34`) is therefore not stored.
    ///
    /// What survives is `Create`'s `fopAcM_create` (`:32-33`). `fopAcM_GetHomeRoomNo(this)` is the
    /// home room, which the sim carries as `Actor::room` - the same reduction every other port
    /// makes. `shape_angle` is never written for a stage-placed actor (RULE 2), so the created
    /// actor gets `current.angle`, i.e. `[0; 3]`.
    fn execute(&mut self, actor: &Actor, commands: &mut Commands) {
        if self.entered {
            return;
        }
        self.entered = true;
        commands.spawn((
            Actor {
                name: jbaba::JBABA_ACTORS[0].to_owned(),
                parameters: JBO_TYPE_NORMAL,
                pos: actor.pos + Vec3::new(0.0, LEAF_LIFT, 0.0),
                angle: [0; 3],
                set_id: 0,
                room: actor.room,
                ..Default::default()
            },
            jbaba::Baba::spawn(JBO_TYPE_NORMAL),
        ));
    }
}

pub fn is_ojtree(name: &str) -> bool {
    OJTREE_ACTORS.contains(&name)
}

pub fn control(mut poles: Query<(&Actor, &mut Ojtree)>, mut commands: Commands) {
    for (actor, mut pole) in &mut poles {
        pole.execute(actor, &mut commands);
    }
}
