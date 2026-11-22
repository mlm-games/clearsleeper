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
pub const AM2_ARCHIVE: &str = crate::assets::AM2_ARCHIVE;
pub const OQ_ARCHIVE: &str = crate::assets::OQ_ARCHIVE;
pub const FGANON_ARCHIVE: &str = crate::assets::FGANON_ARCHIVE;

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

macro_rules! publish_actor {
    ($name:ident, $comp:ty, $archive:expr, $bck:expr, $frame:expr, $looping:expr, $min:expr, $max:expr) => {
        pub fn $name(entities: Query<(Entity, &$comp)>, mut anms: ResMut<ActorAnms>) {
            for (entity, actor) in &entities {
                let bck = ($bck)(actor) as i32;
                if bck < $min || bck > $max {
                    continue;
                }
                anms.0.push(ActorAnm {
                    entity,
                    archive: $archive,
                    bck,
                    frame: ($frame)(actor),
                    looping: ($looping)(actor),
                });
            }
        }
    };
}

use super::anim_clock::ACTOR_EMODE_LOOP;
publish_actor!(
    gohdan,
    super::gohdan::Gohdan,
    "Bst",
    |c: &super::gohdan::Gohdan| c.anm,
    |c: &super::gohdan::Gohdan| c.morph.frame,
    |c: &super::gohdan::Gohdan| c.morph.attribute == ACTOR_EMODE_LOOP,
    0x6,
    0x29
);
publish_actor!(
    gohma,
    super::gohma::Gohma,
    "Btd",
    |c: &super::gohma::Gohma| c.morph_bck,
    |c: &super::gohma::Gohma| c.morph.frame,
    |c: &super::gohma::Gohma| c.morph.attribute == ACTOR_EMODE_LOOP,
    0x8,
    0x39
);
publish_actor!(
    kalle_demos,
    super::kalle_demos::KalleDemos,
    "Bmd",
    |c: &super::kalle_demos::KalleDemos| c.anm,
    |c: &super::kalle_demos::KalleDemos| c.body_morph.frame,
    |c: &super::kalle_demos::KalleDemos| c.body_morph.attribute == ACTOR_EMODE_LOOP,
    0x8,
    0x25
);
publish_actor!(
    kamome2,
    super::kamome2::Kamome2,
    "Kamome",
    |c: &super::kamome2::Kamome2| c.morph.bck as i32,
    |c: &super::kamome2::Kamome2| c.morph.frame,
    |c: &super::kamome2::Kamome2| c.morph.attribute == ACTOR_EMODE_LOOP,
    0xC,
    0x14
);
publish_actor!(
    kamome,
    super::kamome::Kamome,
    "Kamome",
    |c: &super::kamome::Kamome| c.anm_idx,
    |c: &super::kamome::Kamome| c.morph.frame,
    |c: &super::kamome::Kamome| c.morph.attribute == ACTOR_EMODE_LOOP,
    0xC,
    0x14
);
publish_actor!(
    npc_p1,
    super::npc_p1::NpcP1,
    "P1",
    |c: &super::npc_p1::NpcP1| c.anm_num,
    |c: &super::npc_p1::NpcP1| c.morph.frame,
    |c: &super::npc_p1::NpcP1| c.morph.attribute == ACTOR_EMODE_LOOP,
    0x1A,
    0x29
);
publish_actor!(
    npc_tc,
    super::npc_tc::NpcTc,
    "Tc",
    |c: &super::npc_tc::NpcTc| c.bck_idx as i32,
    |c: &super::npc_tc::NpcTc| c.frame.frame,
    |c: &super::npc_tc::NpcTc| c.frame.attribute == ACTOR_EMODE_LOOP,
    0x6,
    0x16
);
publish_actor!(
    bmdhand,
    super::bmdhand::Bmdhand,
    "Bmdhand",
    |c: &super::bmdhand::Bmdhand| c.anm + 5,
    |c: &super::bmdhand::Bmdhand| c.morph.frame,
    |_c: &super::bmdhand::Bmdhand| false,
    0x5,
    0x6
);
publish_actor!(
    medli,
    super::medli::Medli,
    "Md",
    |c: &super::medli::Medli| c.anm,
    |c: &super::medli::Medli| c.anm_frame,
    |_c: &super::medli::Medli| false,
    0x7,
    0x59
);
publish_actor!(
    servant,
    super::servant::Servant,
    "Sv",
    |c: &super::servant::Servant| c.anm,
    |c: &super::servant::Servant| c.morph_frame,
    |_c: &super::servant::Servant| false,
    0x4,
    0xF
);

/// Helmaroc's `mDoExt_McaMorf` cursor (`tww/src/d/actor/d_a_oq.cpp:144-146`).
pub fn helmaroc(birds: Query<(Entity, &super::helmaroc::Helmaroc)>, mut anms: ResMut<ActorAnms>) {
    for (entity, bird) in &birds {
        let bck = bird.curr_bck;
        if !(0x1C..=0x42).contains(&bck) {
            continue;
        }
        anms.0.push(ActorAnm {
            entity,
            archive: OQ_ARCHIVE,
            bck,
            frame: bird.morph.frame,
            looping: bird.morph.attribute == super::anim_clock::ACTOR_EMODE_LOOP,
        });
    }
}

/// Phantom Ganon's `mDoExt_McaMorf` cursor (`tww/src/d/actor/d_a_fganon.cpp:56-63`).
pub fn phantom_ganon(
    ganons: Query<(Entity, &super::phantom_ganon::PhantomGanon)>,
    mut anms: ResMut<ActorAnms>,
) {
    for (entity, ganon) in &ganons {
        let bck = ganon.curr_bck;
        if !(0x6..=0x16).contains(&bck) {
            continue;
        }
        anms.0.push(ActorAnm {
            entity,
            archive: FGANON_ARCHIVE,
            bck,
            frame: ganon.morph.frame,
            looping: ganon.morph.attribute == super::anim_clock::ACTOR_EMODE_LOOP,
        });
    }
}

/// Amos Knight's `mDoExt_McaMorf` cursor (`tww/src/d/actor/d_a_am2.cpp:132-147`).
pub fn amos_knight(
    knights: Query<(Entity, &super::amos_knight::AmosKnight)>,
    mut anms: ResMut<ActorAnms>,
) {
    for (entity, knight) in &knights {
        let bck = knight.curr_bck;
        if !(0x6..=0xF).contains(&bck) {
            continue;
        }
        anms.0.push(ActorAnm {
            entity,
            archive: AM2_ARCHIVE,
            bck,
            frame: knight.morph.frame,
            looping: knight.morph.attribute == super::anim_clock::ACTOR_EMODE_LOOP,
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
