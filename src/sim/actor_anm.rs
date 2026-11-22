use bevy_ecs::prelude::*;

use super::anim_clock::Mode;
use super::ba1::Ba1;

/// One actor's animation as the view needs it: which object archive and resource id its own
/// table named, plus where its frame cursor got to. `dNpc_setAnmIDRes` in both trees looks the
/// BCK up in the actor's object archive (`tww/src/d/d_npc.cpp:546-563`,
/// `wwhd/wwhd_src/d/d_npc.cpp:58-69`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ActorAnm {
    pub entity: Entity,
    pub archive: &'static str,
    pub bck: i32,
    pub frame: f32,
    pub looping: bool,
}

impl ActorAnm {
    pub fn view(&self) -> crate::view::ActorAnm {
        crate::view::ActorAnm {
            archive: self.archive,
            bck: self.bck,
            frame: self.frame,
            looping: self.looping,
        }
    }
}

#[derive(Resource, Debug, Default)]
pub struct ActorAnms(pub Vec<ActorAnm>);

pub const BA_ARCHIVE: &str = "Ba";
pub const CC_ARCHIVE: &str = crate::assets::CC_ARCHIVE;
pub const AM_ARCHIVE: &str = crate::assets::AM_ARCHIVE;
pub const FM_ARCHIVE: &str = crate::assets::FM_ARCHIVE;

/// The queue is rebuilt from the actors each tick, the way a frame's presentation list is.
pub fn clear(mut anms: ResMut<ActorAnms>) {
    anms.0.clear();
}

/// Grandma's cursor is the one non-Link animation already ported from its object archive:
/// `morf_frame` is `mpMorf->getFrame()` and `morf` is the `FrameCtrl` `setAnm` initialised
/// (`tww/src/d/actor/d_a_npc_ba1.cpp:280`, `m_Do_ext.cpp:1414-1437`;
/// `wwhd/wwhd_src/d/actor/d_a_npc_ba1.cpp:402-419`).
pub fn ba1(grandmas: Query<(Entity, &Ba1)>, mut anms: ResMut<ActorAnms>) {
    for (entity, grandma) in &grandmas {
        if !grandma.morf_live || !grandma.drawn() {
            continue;
        }
        let Some(bck) = usize::try_from(grandma.anm_num)
            .ok()
            .and_then(|index| super::ba1::BCK.get(index))
            .copied()
        else {
            continue;
        };
        anms.0.push(ActorAnm {
            entity,
            archive: BA_ARCHIVE,
            bck,
            frame: grandma.morf_frame,
            looping: grandma.morf.attribute() == Mode::Loop,
        });
    }
}

/// Floormaster's `mDoExt_McaMorf` cursor (`tww/src/d/actor/d_a_fm.cpp:2586`).
pub fn floormaster(
    masters: Query<(Entity, &super::floormaster::Floormaster)>,
    mut anms: ResMut<ActorAnms>,
) {
    for (entity, master) in &masters {
        let bck = master.anm_bck;
        if !(0x6..=0x12).contains(&bck) {
            continue;
        }
        anms.0.push(ActorAnm {
            entity,
            archive: FM_ARCHIVE,
            bck,
            frame: master.morph.frame,
            looping: master.morph.attribute == super::anim_clock::ACTOR_EMODE_LOOP,
        });
    }
}

/// Armos' `mDoExt_McaMorph` cursor (`tww/src/d/actor/d_a_am.cpp:132-147`).
pub fn armos(armoses: Query<(Entity, &super::armos::Armos)>, mut anms: ResMut<ActorAnms>) {
    for (entity, armos) in &armoses {
        let bck = armos.bck;
        if !(0x14..=0x1F).contains(&bck) {
            continue;
        }
        anms.0.push(ActorAnm {
            entity,
            archive: AM_ARCHIVE,
            bck,
            frame: armos.morph.frame,
            looping: armos.morph.attribute == super::anim_clock::ACTOR_EMODE_LOOP,
        });
    }
}

/// Chuchu's `mDoExt_McaMorf` cursor, one of the actor-local `ActorFrame`s named in
/// `anim_clock.rs` (`tww/src/d/actor/d_a_cc.cpp:2375-2388`, HD `d_a_cc.cpp` equivalent).
pub fn chuchu(chuchus: Query<(Entity, &super::chuchu::Chuchu)>, mut anms: ResMut<ActorAnms>) {
    for (entity, chuchu) in &chuchus {
        let bck = chuchu.anim_res;
        if bck < 0 || !(0x14..=0x22).contains(&bck) {
            continue;
        }
        anms.0.push(ActorAnm {
            entity,
            archive: CC_ARCHIVE,
            bck,
            frame: chuchu.morph.frame,
            looping: chuchu.morph.attribute == super::anim_clock::ACTOR_EMODE_LOOP,
        });
    }
}
