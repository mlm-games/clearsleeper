use bevy_ecs::prelude::*;

use super::actor::Actor;
use super::flags::MemBits;
use super::pickup::Pickup;

pub const BITEM_ACTORS: [&str; 1] = ["Bitem"];

const ITEM_ACTOR: &str = "item"; // <- d_stage.cpp:647 (OBJNAME("item", fpcNm_ITEM_e))
const ITEM_PARAMETERS: u32 = 0x33FF_FF08; // <- f_op_actor_mng.cpp:31-32 MAKE_ITEM_PARAMS(0x08, -1, -1, 3, 12)
const ITEM_SWITCH_ANGLE: i16 = 0xFF; // <- f_op_actor_mng.cpp:978-979 (prmAngle.z = switchNo)
const ITEM_SET_ID: u16 = 0xFFFF; // <- d_stage.cpp:1476

/// `daBossItem_Create` (`d_a_boss_item.cpp:25-41`) always returns `cPhs_ERROR_e`, and
/// `daBossItem_METHODS[2]` (`:46`) is NULL, so retail cancels the term inside Create
/// (`f_pc_create_req.cpp:106-108`) and it never executes even once. The only thing it does
/// before cancelling is offer the boss heart container, so `reward` carries that verdict.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bitem {
    pub reward: bool,
}

impl Bitem {
    /// `daBossItem_prm::getStage` (`d_a_boss_item.h:13`) reads `param & 0xFF` as the save
    /// table id; `dComIfGs_isStageBossEnemy` (`d_com_inf_game.cpp:826-833`) then reads the
    /// live bit when that id is the running stage and the save slot otherwise. `MemBits`
    /// keeps only the running stage's bit, and a `Bitem` is only ever placed inside the
    /// stage whose save slot its parameter names, so the parameter is not stored.
    pub fn spawn(mem: &MemBits) -> Self {
        Self {
            reward: mem.is_stage_boss_enemy() && !mem.is_stage_life(),
        }
    }

    /// `fopAcM_createItemForBoss(..., param_6 = 1)` (`d_a_boss_item.cpp:34-37`) becomes
    /// `fopAcM_createItem(pos, dItemNo_HEART_CONTAINER_e, -1, room, daItemType_3_e, angle,
    /// daItemAct_BOSS_e, NULL)` (`f_op_actor_mng.cpp:958-959`).
    fn reward_item(&self, actor: &Actor) -> Actor {
        Actor {
            name: ITEM_ACTOR.to_owned(),
            parameters: ITEM_PARAMETERS,
            pos: actor.pos,
            angle: [actor.angle[0], actor.angle[1], ITEM_SWITCH_ANGLE],
            set_id: ITEM_SET_ID,
            room: actor.room,
        }
    }
}

pub fn is_bitem(name: &str) -> bool {
    BITEM_ACTORS.contains(&name)
}

pub fn control(bosses: Query<(Entity, &Actor, &Bitem)>, mut commands: Commands) {
    for (entity, actor, boss) in &bosses {
        if boss.reward {
            let item = boss.reward_item(actor);
            commands.spawn((item, Pickup::spawn(ITEM_PARAMETERS)));
        }
        let _ = commands.entity(entity).despawn();
    }
}