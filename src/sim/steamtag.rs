use bevy_ecs::prelude::*;

use super::actor::Actor;
use super::area::StageName;
use super::flags::Collect;
use super::inventory::{self, Equipped, ITEM_PEARL_DIN};

pub const STEAMTAG_ACTORS: [&str; 2] = ["Ystm0", "Ystm1"];

const REFUSE_STAGE: &str = "Adanmae"; // <- d_a_steam_tag.cpp:201
const REFUSE_ROOM: i8 = 0; // <- d_a_steam_tag.cpp:202

/// The stage strings are `Ystm0` and `Ystm1` (`d_stage.cpp:845-846`); this actor has no
/// `M_arcname` of its own (`d_a_steam_tag.h:24-55` lists none) - the vent's only asset is the
/// particle archive `dPa_name::ID_AK_SN_STEAM00/01` (`d_a_steam_tag.cpp:96-98`).
///
/// `daSteamTag_c::create` (`:196-210`) returns `cPhs_ERROR_e` when the stage is `Adanmae`,
/// the term sits in room 0 and the Din's Pearl is already held. `ERROR` cancels the request
/// inside Create (`f_pc_create_req.cpp:106-108`), so retail never runs `_execute` on such a
/// term; `refused` records that and the vent is dropped on its first tick.
///
/// Everything else in the file is reachable only through an emitter. `_execute` (`:126-175`)
/// has two arms and neither can run in this sim:
///
/// * arm one (`:128-144`) counts `mCreateTimer` down and calls `createEmitter` (`:92-107`),
///   whose only success path is `dComIfG_particle_setToon` returning non-NULL (`:100`). With
///   no particle system it returns FALSE, so `mCreateTimer` is reset to `30` (`:141`) forever
///   and `mEmitTimer` never leaves `0`;
/// * arm two (`:145-173`) requires `mCreateTimer == 0 && mEmitTimer > 0`, and `mEmitTimer`
///   only becomes positive inside a successful `createEmitter` (`:138`).
///
/// `fopAcM_seStart(this, JA_SE_OBJ_STEAM, 0)` (`:165`), `mCps` / `dComIfG_Ccsp()->Set(&mCps)`
/// (`:169-170`) and `mPointWind.set_pwind_move()` (`:171`) all live in arm two, and the
/// `mPntWindCpsS` collider is an `AT_TYPE_WIND` source (`l_cps_src`, `:19`) whose semantics the
/// contract lists as confirmed-absent. `m29B` (`:64`) is the actor's only parameter read and
/// nothing ever reads it back, so it is not stored. `daSteamTag_Draw` (`:116-118`) returns TRUE
/// with no draw call and `CreateInit` even calls `fopAcM_offDraw` (`:87`), so the vent renders
/// nothing and needs no `actor_views` branch.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Steamtag;

impl Steamtag {
    pub fn spawn() -> Self {
        Self
    }

    fn refused(&self, stage: &StageName, room: i8, equipped: &Equipped, collect: &Collect) -> bool {
        stage.0 == REFUSE_STAGE
            && room == REFUSE_ROOM
            && inventory::check_get_item(equipped, collect, ITEM_PEARL_DIN)
    }
}

pub fn is_steamtag(name: &str) -> bool {
    STEAMTAG_ACTORS.contains(&name)
}

pub fn control(
    stage: Res<StageName>,
    equipped: Res<Equipped>,
    collect: Res<Collect>,
    vents: Query<(Entity, &Actor, &Steamtag)>,
    mut commands: Commands,
) {
    for (entity, actor, vent) in &vents {
        if vent.refused(&stage, actor.room, &equipped, &collect) {
            let _ = commands.entity(entity).despawn();
        }
    }
}
