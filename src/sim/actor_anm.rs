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
pub const BDK_ARCHIVE: &str = crate::assets::BDK_ARCHIVE;
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
publish_actor!(
    npc_p2,
    super::npc_p2::NpcP2,
    "P2",
    |c: &super::npc_p2::NpcP2| c.anm_no,
    |c: &super::npc_p2::NpcP2| c.morph.frame,
    |_c: &super::npc_p2::NpcP2| false,
    0x7,
    0x23
);
publish_actor!(
    zunari,
    super::zunari::Zunari,
    "Rsh",
    |c: &super::zunari::Zunari| c.anm as i32,
    |c: &super::zunari::Zunari| c.anm_frame,
    |_c: &super::zunari::Zunari| false,
    0x14,
    0x1D
);
publish_actor!(
    makar,
    super::makar::Makar,
    "Mk",
    |c: &super::makar::Makar| c.anm as i32,
    |c: &super::makar::Makar| c.anm_frame,
    |_c: &super::makar::Makar| false,
    0x5,
    0x10
);
publish_actor!(
    toripost,
    super::toripost::Toripost,
    "Toripost",
    |c: &super::toripost::Toripost| c.bck_idx as i32,
    |c: &super::toripost::Toripost| c.morf_frame,
    |_c: &super::toripost::Toripost| false,
    0x4,
    0x6
);
publish_actor!(
    npc_bmsw,
    super::npc_bmsw::NpcBmsw,
    "Bmsw",
    |c: &super::npc_bmsw::NpcBmsw| c.anm_idx as i32,
    |c: &super::npc_bmsw::NpcBmsw| c.anm_frame as f32,
    |_c: &super::npc_bmsw::NpcBmsw| false,
    0x9,
    0x16
);

publish_actor!(
    puppet_ganon,
    super::puppet_ganon::PuppetGanon,
    "Bgn",
    |c: &super::puppet_ganon::PuppetGanon| c.curr_bck,
    |c: &super::puppet_ganon::PuppetGanon| c.morph.frame,
    |c: &super::puppet_ganon::PuppetGanon| c.morph.attribute == ACTOR_EMODE_LOOP,
    0x6,
    0x10
);
publish_actor!(
    bwds,
    super::bwds::Bwds,
    "Bwds",
    |c: &super::bwds::Bwds| c.curr_bck,
    |c: &super::bwds::Bwds| c.morph.frame,
    |c: &super::bwds::Bwds| c.morph.attribute == ACTOR_EMODE_LOOP,
    0x4,
    0x5
);

// Actors whose BCK id is stored sim-side but which have no frame cursor in the port: pose the
// bind pose of the selected animation. Still far more faithful than a coloured box.
publish_actor!(
    aryll,
    super::aryll::Aryll,
    "Ls",
    |c: &super::aryll::Aryll| c.bck as i32,
    |_c: &super::aryll::Aryll| 0.0,
    |_c: &super::aryll::Aryll| false,
    0x8,
    0x15
);
publish_actor!(
    sage,
    super::sage::Sage,
    "Hr",
    |c: &super::sage::Sage| c.anm_idx as i32,
    |_c: &super::sage::Sage| 0.0,
    |_c: &super::sage::Sage| false,
    0x6,
    0xF
);
publish_actor!(
    npc_hi1,
    super::npc_hi1::NpcHi1,
    "Hi",
    |c: &super::npc_hi1::NpcHi1| c.anm_no as i32,
    |_c: &super::npc_hi1::NpcHi1| 0.0,
    |_c: &super::npc_hi1::NpcHi1| false,
    0x6,
    0x7
);
publish_actor!(
    npc_nz,
    super::npc_nz::NpcNz,
    "Nz",
    |c: &super::npc_nz::NpcNz| c.anm_idx as i32,
    |_c: &super::npc_nz::NpcNz| 0.0,
    |_c: &super::npc_nz::NpcNz| false,
    0x15,
    0x23
);
publish_actor!(
    npc_km1,
    super::npc_km1::NpcKm1,
    "Km",
    |c: &super::npc_km1::NpcKm1| c.anm as i32,
    |_c: &super::npc_km1::NpcKm1| 0.0,
    |_c: &super::npc_km1::NpcKm1| false,
    0x9,
    0x9
);
publish_actor!(
    npc_jb1,
    super::npc_jb1::NpcJb1,
    "Jb",
    |c: &super::npc_jb1::NpcJb1| c.anm as i32,
    |_c: &super::npc_jb1::NpcJb1| 0.0,
    |_c: &super::npc_jb1::NpcJb1| false,
    0x7,
    0x9
);
publish_actor!(
    npc_ho,
    super::npc_ho::NpcHo,
    "Ho",
    |c: &super::npc_ho::NpcHo| c.anm_idx as i32,
    |_c: &super::npc_ho::NpcHo| 0.0,
    |_c: &super::npc_ho::NpcHo| false,
    0x6,
    0x9
);
publish_actor!(
    lenzo,
    super::lenzo::Lenzo,
    "Po",
    |c: &super::lenzo::Lenzo| c.bck_index as i32,
    |_c: &super::lenzo::Lenzo| 0.0,
    |_c: &super::lenzo::Lenzo| false,
    0x5,
    0x9
);
publish_actor!(
    marlon,
    super::marlon::Marlon,
    "Mn",
    |c: &super::marlon::Marlon| c.bck_idx as i32,
    |_c: &super::marlon::Marlon| 0.0,
    |_c: &super::marlon::Marlon| false,
    0x6,
    0xD
);
publish_actor!(
    npc_ah,
    super::npc_ah::NpcAh,
    "Ah",
    |c: &super::npc_ah::NpcAh| c.bck_idx as i32,
    |_c: &super::npc_ah::NpcAh| 0.0,
    |_c: &super::npc_ah::NpcAh| false,
    0x5,
    0x6
);
publish_actor!(
    npc_tt,
    super::npc_tt::NpcTt,
    "Tt",
    |c: &super::npc_tt::NpcTt| c.anm_idx as i32,
    |_c: &super::npc_tt::NpcTt| 0.0,
    |_c: &super::npc_tt::NpcTt| false,
    0x11,
    0x19
);
publish_actor!(
    npc_mt,
    super::npc_mt::NpcMt,
    "Niten",
    |c: &super::npc_mt::NpcMt| c.bck_index as i32,
    |_c: &super::npc_mt::NpcMt| 0.0,
    |_c: &super::npc_mt::NpcMt| false,
    0x5,
    0xA
);
publish_actor!(
    npc_btsw2,
    super::npc_btsw2::NpcBtsw2,
    "Btsw",
    |c: &super::npc_btsw2::NpcBtsw2| c.anm as i32,
    |_c: &super::npc_btsw2::NpcBtsw2| 0.0,
    |_c: &super::npc_btsw2::NpcBtsw2| false,
    0x8,
    0x11
);

publish_actor!(
    redead,
    super::redead::ReDead,
    "Rd",
    |_c: &super::redead::ReDead| 0x6_i32,
    |c: &super::redead::ReDead| c.timer1 as f32,
    |_c: &super::redead::ReDead| false,
    0x6,
    0x14
);
publish_actor!(
    stalfos,
    super::stalfos::Stalfos,
    "St",
    |c: &super::stalfos::Stalfos| c.curr_bck,
    |c: &super::stalfos::Stalfos| c.frame,
    |c: &super::stalfos::Stalfos| c.looping,
    0x4,
    0x35
);
publish_actor!(
    npc_btsw,
    super::npc_btsw::NpcBtsw,
    "Btsw",
    |c: &super::npc_btsw::NpcBtsw| c.field_9c5 as i32,
    |c: &super::npc_btsw::NpcBtsw| c.morf.frame,
    |c: &super::npc_btsw::NpcBtsw| c.morf.attribute == ACTOR_EMODE_LOOP,
    0x8,
    0x11
);

/// Helmaroc's `mDoExt_McaMorf` cursor (`tww/src/d/actor/d_a_bdk.cpp:214`; HD
/// `wwhd/wwhd_src/d/actor/d_a_bdk.cpp:212-219`).
pub fn helmaroc(birds: Query<(Entity, &super::helmaroc::Helmaroc)>, mut anms: ResMut<ActorAnms>) {
    for (entity, bird) in &birds {
        let bck = bird.curr_bck;
        if !(0x1C..=0x42).contains(&bck) {
            continue;
        }
        anms.0.push(ActorAnm {
            entity,
            archive: BDK_ARCHIVE,
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

/// Molgera's `mDoExt_McaMorf` clip and cursor (`tww/src/d/actor/d_a_bwd.cpp:139-147`;
/// `wwhd/wwhd_src/d/actor/d_a_bwd.cpp:101-113`); the ids are
/// `tww/assets/GZLE01/res/Object/Bwd.h:62-78`.
pub fn molgera(bosses: Query<(Entity, &super::molgera::Molgera)>, mut anms: ResMut<ActorAnms>) {
    use super::molgera::Clip;
    for (entity, boss) in &bosses {
        let bck = match boss.clip {
            Clip::Attack => 0x7,
            Clip::AttackEat => 0x8,
            Clip::AttackShippai => 0x9,
            Clip::Dead1 => 0xC,
            Clip::Dead2 => 0xD,
            Clip::Dead3 => 0xE,
            Clip::Fly => 0xF,
            Clip::Jattack1 => 0x10,
            Clip::Jattack2 => 0x11,
            Clip::Jattack3 => 0x12,
            Clip::Wait => 0x17,
        };
        anms.0.push(ActorAnm {
            entity,
            archive: "Bwd",
            bck,
            frame: boss.frame,
            looping: boss.looping,
        });
    }
}

/// Tetra's arc name is chosen per type (sleeping `"Zl2"`, awake `"Zl"`;
/// `tww/src/d/actor/d_a_npc_zl1.cpp:1330-1337`). `_draw` returns early for the demo flags
/// (`:2654`), so those instances have no body to pose.
pub fn tetra(girls: Query<(Entity, &super::zelda::Zelda)>, mut anms: ResMut<ActorAnms>) {
    for (entity, zelda) in &girls {
        if zelda.hidden_demo || zelda.draw_off {
            continue;
        }
        anms.0.push(ActorAnm {
            entity,
            archive: zelda.arc,
            bck: 0x0,
            frame: 0.0,
            looping: false,
        });
    }
}

/// The remaining actors publish the bind pose of their wait BCK: the sim stores no frame
/// cursor for them, but a posed body is still closer than the box fallback. The ids come from
/// `tww/assets/GZLE01/res/Object/Sss.h:21-24`, `tww/assets/GZLE01/res/Object/Bs.h:48`,
/// `tww/assets/GZLE01/res/Object/Ac.h:48`, `tww/assets/GZLE01/res/Object/Co.h:71`,
/// `tww/assets/GZLE01/res/Object/Uk.h:53`, `tww/assets/GZLE01/res/Object/Gp.h:34`,
/// `tww/assets/GZLE01/res/Object/Pf.h:36` and `tww/assets/GZLE01/res/Object/Always.h:137,155`
/// (`tww/src/d/actor/d_a_npc_fa1.cpp:951-955`).
publish_actor!(
    dexivine,
    super::sss::Sss,
    "Sss",
    |c: &super::sss::Sss| if c.anm == 0 { 0x5 } else { 0x6 },
    |_c: &super::sss::Sss| 0.0,
    |_c: &super::sss::Sss| false,
    0x5,
    0x6
);
publish_actor!(
    beedle,
    super::shop::Shop,
    "Bs",
    |_c: &super::shop::Shop| 0x0,
    |_c: &super::shop::Shop| 0.0,
    |_c: &super::shop::Shop| false,
    0x0,
    0x10
);
publish_actor!(
    komali,
    super::npc_ac1::NpcAc1,
    "Ac",
    |_c: &super::npc_ac1::NpcAc1| 0x0,
    |_c: &super::npc_ac1::NpcAc1| 0.0,
    |_c: &super::npc_ac1::NpcAc1| false,
    0x0,
    0xE
);
publish_actor!(
    komali_young,
    super::npc_co1::NpcCo1,
    "Co",
    |_c: &super::npc_co1::NpcCo1| 0x0,
    |_c: &super::npc_co1::NpcCo1| 0.0,
    |_c: &super::npc_co1::NpcCo1| false,
    0x0,
    0x1D
);
publish_actor!(
    roberto,
    super::npc_uk::NpcUk,
    "Uk",
    |_c: &super::npc_uk::NpcUk| 0x7,
    |_c: &super::npc_uk::NpcUk| 0.0,
    |_c: &super::npc_uk::NpcUk| false,
    0x7,
    0x11
);
publish_actor!(
    gp1,
    super::npc_gp1::NpcGp1,
    "Gp",
    |_c: &super::npc_gp1::NpcGp1| 0x0,
    |_c: &super::npc_gp1::NpcGp1| 0.0,
    |_c: &super::npc_gp1::NpcGp1| false,
    0x0,
    0x8
);
publish_actor!(
    pf1,
    super::npc_pf1::NpcPf1,
    "Pf",
    |_c: &super::npc_pf1::NpcPf1| 0x0,
    |_c: &super::npc_pf1::NpcPf1| 0.0,
    |_c: &super::npc_pf1::NpcPf1| false,
    0x0,
    0x7
);
publish_actor!(
    great_fairy,
    super::npc_fa1::NpcFa1,
    "Always",
    |_c: &super::npc_fa1::NpcFa1| 0xC,
    |_c: &super::npc_fa1::NpcFa1| 0.0,
    |_c: &super::npc_fa1::NpcFa1| false,
    0xC,
    0xC
);
