use bevy_ecs::prelude::*;

use super::actor::Actor;

/// The weight a name falls back to when no `cCcD_Stts::Init` call was located for it: the same
/// value `Actor::default` carries, a mid-ladder rank whose `rank_tbl` diagonal is an even split.
/// Every name the port spawns that is not in `WEIGHTS` lands here. It is not a weight the
/// reference chose — a process the reference never `Init`ed leaves `cCcD_Stts::m_weight` at
/// whatever the process's own allocation held, which is not a value anything can be read out of.
pub const FALLBACK: u8 = 100;

/// The per-process weights the reference hands `cCcD_Stts::Init` (`d_cc_d.cpp:67-84`), which
/// `dCcS::GetRank` (`d_cc_s.cpp:209-233`) buckets and `rank_tbl` (`d_cc_s.cpp:194-206`) indexes, so
/// two overlapping bodies split the push by these and not evenly.
///
/// Each row is the port actor name, the `cCcD_Stts::GetWeightUc` the reference's own `Init` was
/// handed, and the `mStts.Init` call it was read from. The citation is the reference actor source
/// file's own stem and the line of the call, with the one prefix every actor source file shares
/// left off.
///
/// `0` and `0xFF` are the two weights a `dCcS::SetPosCorrect` pair (`d_cc_s.cpp:257-261`) refuses
/// to correct at all, so an immovable row is not a heavy one: it cannot be pushed by anything.
const WEIGHTS: &[(&str, u8, &str)] = &[
    ("Ah", 0xFF, "npc_ah.cpp:265"),
    ("amos", 254, "am.cpp:1229"),
    ("amos2", 254, "am2.cpp:1342"),
    ("Arrow", 10, "arrow.cpp:1060"),
    ("Ba1", 0xFF, "npc_ba1.cpp:263"),
    ("Bb", 100, "bb.cpp:2769"),
    ("Bdk", 0xFF, "bdk.cpp:4193"),
    ("Bdkobj", 0xFF, "bdkobj.cpp:593"),
    ("BFlower", 0xFF, "bflower.cpp:186"),
    ("Bgn", 0xFF, "bgn.cpp:3605"),
    ("Bgn2", 0xFF, "bgn2.cpp:1226"),
    ("Bgn3", 0xFF, "bgn3.cpp:1302"),
    ("Bk", 200, "bk.cpp:5113"),
    ("Bm1", 0xFF, "npc_bm1.cpp:695"),
    ("Bmdhand", 0xFF, "bmdhand.cpp:809"),
    ("Bmsw", 0xFF, "npc_bmsw.cpp:594"),
    ("Boko", 10, "process create :989"),
    ("Bomb2", 200, "bomb2.cpp:331"),
    ("bridge", 0xFF, "bridge.cpp:1283"),
    ("Bs1", 0xFF, "npc_bs1.cpp:1580"),
    ("Bst", 0xFF, "bst.cpp:3141"),
    ("Btd", 0xFF, "btd.cpp:2972"),
    ("Btsw", 0xFF, "npc_btsw.cpp:473"),
    ("Btsw2", 0xFF, "npc_btsw2.cpp:408"),
    ("Bwd", 0xFA, "bwd.cpp:2517"),
    ("Canon", 0xFF, "obj_canon.cpp:562"),
    ("Cb1", 0xFE, "npc_cb1.cpp:2959"),
    ("Daiocta", 0, "daiocta.cpp:1669"),
    ("Doguu", 0xFF, "obj_doguu.cpp:207"),
    ("Ebomzo", 0xFF, "obj_ebomzo.cpp:66"),
    ("Ep", 0xFF, "ep.cpp:654"),
    ("Fganon", 0xFA, "fganon.cpp:2786"),
    ("Fgmahou", 0xFA, "fgmahou.cpp:417"),
    ("Figure", 0xFF, "obj_figure.cpp:555"),
    ("Fire", 0xFF, "fire.cpp:165"),
    ("FIRE", 0xFF, "fire.cpp:165"),
    ("Gtaki", 0xFF, "obj_gtaki.cpp:110"),
    ("Hfuck1", 0xFF, "obj_hfuck1.cpp:131"),
    ("Hi1", 0xFF, "npc_hi1.cpp:161"),
    ("Hmlif", 0xFF, "hmlif.cpp:241"),
    ("ho", 0xFF, "npc_ho.cpp:549"),
    ("Ho", 0xFF, "npc_ho.cpp:549"),
    ("Hr", 0xFE, "npc_hr.cpp:1357"),
    ("Hys", 255, "hys.cpp:98"),
    ("Ikada", 0xFF, "obj_ikada.cpp:1475"),
    ("item", 0, "item.cpp:140"),
    ("Kamome", 0x32, "kamome.cpp:1530"),
    ("Kantera", 0xE6, "kantera.cpp:412"),
    ("Kita", 0xFF, "kita.cpp:477"),
    ("Km1", 0xFF, "npc_km1.cpp:103"),
    ("Lamp", 0xFF, "lamp.cpp:152"),
    ("Ls1", 0xFF, "npc_ls1.cpp:332"),
    ("Mflft", 0xFF, "mflft.cpp:641"),
    ("Mk", 0xA0, "npc_mk.cpp:407"),
    ("MkieK", 0xFF, "obj_mkiek.cpp:68"),
    ("mo2", 200, "mo2.cpp:4174"),
    ("Mt", 0xFF, "npc_mt.cpp:526"),
    ("Nh", 0xFF, "nh.cpp:169"),
    ("NpcSo", 0xFF, "npc_so.cpp:1707"),
    ("Os", 0xFF, "npc_os.cpp:2073"),
    ("Oship", 0xFF, "process create :910"),
    ("Ospbox", 0xFF, "obj_ospbox.cpp:62"),
    ("Plant", 0xFF, "obj_plant.cpp:156"),
    ("Player", 120, "player_main.cpp:12184"),
    ("Rflw", 0xFF, "obj_rflw.cpp:78"),
    ("Roten", 0xFF, "npc_roten.cpp:1727"),
    ("Rsh1", 0xFF, "npc_rsh1.cpp:956"),
    ("Ship", 0xF0, "ship.cpp:4701"),
    ("SieFlag", 0xFF, "sie_flag.cpp:107"),
    ("Sss", 0xFF, "sss.cpp:757"),
    ("Tama", 0, "tama.cpp:44"),
    ("Tc", 0xFE, "npc_tc.cpp:981"),
    ("Tn", 0xF0, "tn.cpp:4308"),
    ("Tt", 0xFF, "npc_tt.cpp:545"),
    ("Vfan", 0xFF, "obj_vfan.cpp:71"),
    ("VmcBS", 255, "obj_vmc.cpp:141"),
    ("Wall", 0xFF, "wall.cpp:128"),
    ("Warpt", 0xFF, "obj_warpt.cpp:343"),
    ("WindTag", 0xFF, "wind_tag.cpp:134"),
    ("Zl1", 0xFF, "npc_zl1.cpp:393"),
];

/// The names whose `SrcObjCo SPrm` carries `cCcD_CoSPrm_NoCrr_e` (`c_cc_d.h:57`) from the moment
/// the body is built, so `cCcD_ObjCo::ChkNoCrr` (`c_cc_d.h:493`) is already true and
/// `dCcS::SetPosCorrect` (`d_cc_s.cpp:237`) returns before it weighs anything. The player is the
/// one name the reference raises the same bit on at runtime instead: its `setCollision` turns
/// `NoCrr` on only while the in-ship mode flag is up (`player_main.cpp:9743`), so "Player" belongs
/// to this list by mode and not by name and the player's own module owns it.
const NO_CRR: &[(&str, &str)] = &[
    ("amos", "am.cpp:1277"),
    ("Arrow", "arrow.cpp:61"),
    ("Eskban", "obj_eskban.cpp:87"),
    ("Gtaki", "obj_gtaki.cpp:21"),
    ("Ikada", "obj_ikada.cpp:92"),
    ("NpcSo", "npc_so.cpp:79"),
    ("VmcBS", "obj_vmc.cpp:27"),
];

/// The `WEIGHTS` row for `name`, or [`FALLBACK`] when no `Init` call was found for it.
pub fn weight_for(name: &str) -> u8 {
    WEIGHTS
        .iter()
        .find(|(actor, _, _)| *actor == name)
        .map_or(FALLBACK, |(_, weight, _)| *weight)
}

/// Whether `name` is a `NO_CRR` row: whether the reference's own collision body for this process
/// refuses correction outright.
pub fn no_crr_for(name: &str) -> bool {
    NO_CRR.iter().any(|(actor, _)| *actor == name)
}

/// Stamp every actor already in `world`, the point `build` reaches once the stage is spawned and
/// before the step is scheduled. Actors spawned later take the fallback until `assign` runs.
pub fn init(world: &mut World) {
    let mut actors = world.query::<&mut Actor>();
    for mut actor in actors.iter_mut(world) {
        actor.weight_uc = weight_for(&actor.name);
        actor.no_crr = no_crr_for(&actor.name);
    }
}

/// Give every actor the weight and the correction refusal its process's name carries, which is
/// what the reference's own `create` does with its `mStts.Init`. Register this ahead of the
/// contact pass so anything spawned during the step is weighed from its first frame.
pub fn assign(mut actors: Query<&mut Actor>) {
    for mut actor in &mut actors {
        actor.weight_uc = weight_for(&actor.name);
        actor.no_crr = no_crr_for(&actor.name);
    }
}