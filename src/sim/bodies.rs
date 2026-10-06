use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::contact::{GObjInf, GStts, ObjSet};
use super::contact_run::{ContactBodies, ContactObj};
use super::damage_resolve::{Atk, AtkType};
use super::shape::{Shape, ShapeKind};
use super::weight;

/// The body every actor without a cited row registers.
///
/// The shape is the one retail hands the player's own collision cylinder, radius `30.0` and
/// height `125.0` (`player_main_data.inc:66-70`), which is a human-sized standing volume and so
/// the only defensible stand-in for an actor whose real collider was not located. It is built as a
/// capsule rather than the cylinder the player registers, which is the same standing volume with
/// the caps the port's own sphere tests already model. The registration is the one retail hands
/// every NPC cylinder through `dNpc_cyl_src` (`d_npc.cpp:955-983`): TG set for the enemy group
/// with a type mask that excludes the boomerang, CO set for the player group against every group,
/// no AT at all. Neither number nor membership is invented: a name with no cited row gets this
/// and only this.
pub const DEFAULT_RADIUS: f32 = 30.0;

/// [`DEFAULT_RADIUS`]'s cylinder height, from the same player cylinder.
pub const DEFAULT_HEIGHT: f32 = 125.0;

/// `dCcD_SrcGObjAt` (`d_cc_d.h:131-137`) with the AT object's own power: the four fields
/// [`Atk`] reduces `mpObj` to. A body without one has no attack object and no AT metadata.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AtMeta {
    /// `mObjAt.mType`, the `dCcD_ObjAtType` bits the type AND in `cCcS::ChkNoHitAtTg` tests.
    pub at_type: AtkType,
    /// `mGObjAt.mSe`.
    pub at_se: u8,
    /// `mGObjAt.mSpl`.
    pub at_spl: u8,
    /// `mObjAt.mAtp`.
    pub at_atp: u8,
}

impl AtMeta {
    /// The [`Atk`] an AT body carries, with no attacker behind it: nothing rewires the damage
    /// sites to read this yet.
    pub fn atk(&self) -> Atk {
        Atk {
            at_type: self.at_type,
            at_se: self.at_se,
            at_spl: self.at_spl,
            at_atp: self.at_atp,
            ac: None,
        }
    }
}

/// One collider a reference actor registers: the shape a `dCcD_SrcSph` / `dCcD_SrcCyl` /
/// `dCcD_SrcCps` constant carries, plus the three registration words and the general object info
/// that ride on it. `center` is the shape's offset from its owner's origin, which is what
/// [`follow`] adds to `Actor::pos` so a head sphere stays above the feet.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Volume {
    kind: ShapeKind,
    center: [f32; 3],
    radius: f32,
    height: f32,
    set: ObjSet,
    gobj: GObjInf,
    atk: Option<AtMeta>,
}

impl Volume {
    /// The shape this volume registers, before its owner has moved it.
    pub fn shape(&self) -> Shape {
        let pos = Vec3::from_array(self.center);
        match self.kind {
            ShapeKind::Sphere => Shape::sphere(pos, self.radius),
            ShapeKind::Cylinder => Shape::cylinder(pos, self.radius, self.height),
            ShapeKind::Capsule => Shape::capsule(pos, self.radius, self.height),
            ShapeKind::Box => Shape::capsule(pos, self.radius, self.height),
        }
    }

    /// The `cCcD_Obj` this volume contributes, with the general status and general object info the
    /// reference's own `cCcD_SrcGObjInf` set.
    pub fn obj(&self) -> ContactObj {
        ContactObj {
            set: self.set,
            g_stts: Some(GStts::default()),
            gobj_inf: Some(self.gobj),
        }
    }

    /// The AT metadata, for a volume that has an attack object.
    pub fn atk(&self) -> Option<AtMeta> {
        self.atk
    }
}

/// A `cCcD_SrcSphAttr` radius, `mRadius` (`c_m3d_g_sph.h:18`).
const fn sph(
    center: [f32; 3],
    radius: f32,
    set: ObjSet,
    gobj: GObjInf,
    atk: Option<AtMeta>,
) -> Volume {
    Volume {
        kind: ShapeKind::Sphere,
        center,
        radius,
        height: 0.0,
        set,
        gobj,
        atk,
    }
}

/// A `cCcD_SrcCylAttr` radius and height, `mRadius` and `mHeight` (`c_m3d_g_cyl.h:20-21`).
const fn cyl(
    center: [f32; 3],
    radius: f32,
    height: f32,
    set: ObjSet,
    gobj: GObjInf,
    atk: Option<AtMeta>,
) -> Volume {
    Volume {
        kind: ShapeKind::Cylinder,
        center,
        radius,
        height,
        set,
        gobj,
        atk,
    }
}

/// A `cCcD_SrcCpsAttr` segment length and radius: `mStart` / `mEnd` collapsed to their midpoint
/// and `|mEnd - mStart|` (`c_m3d_g_cps.h:10-12`).
const fn cps(
    center: [f32; 3],
    radius: f32,
    height: f32,
    set: ObjSet,
    gobj: GObjInf,
    atk: Option<AtMeta>,
) -> Volume {
    Volume {
        kind: ShapeKind::Capsule,
        center,
        radius,
        height,
        set,
        gobj,
        atk,
    }
}

/// The three `cCcD_SrcObj` words of `dCcD_SrcGObjInf`, with the AT / TG / CO group masks already
/// reduced to their numeric `cCcD_AtSPrm_e` / `cCcD_TgSPrm_e` / `cCcD_CoSPrm_e` values.
const fn set(
    at_sprm: u32,
    at_type: u32,
    at_atp: u8,
    tg_sprm: u32,
    tg_type: u32,
    co_sprm: u32,
) -> ObjSet {
    ObjSet {
        at_sprm,
        at_rprm: 0,
        at_type,
        at_atp,
        tg_sprm,
        tg_rprm: 0,
        tg_type,
        co_sprm,
        co_rprm: 0,
        flags: 0,
    }
}

/// The four general `mSPrm` / `mRPrm` words `dCcD_SrcGObjAt` / `Tg` / `Co` carry.
const fn gobj(at_sprm: u32, tg_sprm: u32, co_sprm: u32, co_rprm: u32) -> GObjInf {
    GObjInf {
        at_sprm,
        tg_sprm,
        co_sprm,
        co_rprm,
        co_hit_apid: 0,
    }
}

/// The retail source each named row's collider set was transcribed from, in reference-file
/// order. Rows absent here take [`DEFAULT_RADIUS`] / [`DEFAULT_HEIGHT`].
pub const CITATIONS: &[(&str, &str)] = &[
    ("Ah", "npc_ah.cpp:268"),
    ("Ajav", "obj_ajav.cpp:43,72,102"),
    ("BFlower", "bflower.cpp:15,45,74"),
    ("Ba1", "npc_ba1.cpp:1926"),
    ("Bb", "bb.cpp:2569,2597,2625,2653"),
    ("Bdk", "bdk.cpp:3928,3956,3984,4012,4040,4068,4096,4124"),
    ("Bdkobj", "bdkobj.cpp:503,532"),
    ("Bgn", "bgn.cpp:3523,3551"),
    ("Bgn2", "bgn2.cpp:1155,1183"),
    ("Bgn3", "bgn3.cpp:1232,1260"),
    ("Bita", "bita.cpp:210"),
    ("Bk", "bk.cpp:5114,5145,5176,5206,5236"),
    ("Bm1", "npc_bm1.cpp:4006"),
    ("Bmdhand", "bmdhand.cpp:760"),
    ("Bmsw", "npc_bmsw.cpp:53"),
    ("Boko", "daBoko_c sph_src 883, at_cps_src 911"),
    ("Bs1", "npc_bs1.cpp:59"),
    ("Bst", "bst.cpp:2964,2993,3023,3051,3080"),
    ("Btd", "btd.cpp:2792,2820,2848,2876"),
    ("Btsw", "npc_btsw.cpp:54"),
    ("Btsw2", "npc_btsw2.cpp:35"),
    ("Bwd", "bwd.cpp:2389,2417,2445"),
    ("Bwds", "bwds.cpp:943,971,999"),
    ("Canon", "canon.cpp:32,62"),
    ("Cb1", "npc_cb1.cpp:93,124"),
    ("Doguu", "obj_doguu.cpp:15"),
    ("Ds1", "npc_ds1.cpp:10"),
    ("Ebomzo", "obj_ebomzo.cpp:22"),
    ("Ep", "ep.cpp:579,607"),
    ("Eskban", "obj_eskban.cpp:19,48,77"),
    ("Fganon", "fganon.cpp:2788,2818,2847,2876"),
    ("Fgmahou", "fgmahou.cpp:343,371"),
    ("Figure", "obj_figure.cpp:63"),
    ("Fire", "fire.cpp:11,40,69"),
    ("Gtaki", "obj_gtaki.cpp:12"),
    ("Hfuck1", "obj_hfuck1.cpp:23"),
    ("Hi1", "npc_hi1.cpp:896"),
    ("Hmlif", "hmlif.cpp:46"),
    ("Ho", "npc_ho.cpp:552"),
    ("Hr", "npc_hr.cpp:1233"),
    ("Hys", "hys.cpp:23"),
    ("Kantera", "kantera.cpp:349"),
    ("Kita", "kita.cpp:391"),
    ("Km1", "npc_km1.cpp:671"),
    ("Lamp", "lamp.cpp:154"),
    ("Ls1", "npc_ls1.cpp:2415"),
    ("Mflft", "mflft.cpp:563"),
    ("Mk", "npc_mk.cpp:262"),
    ("MkieK", "obj_mkiek.cpp:15"),
    ("Mn", "npc_mn.cpp:179"),
    ("Mt", "npc_mt.cpp:487"),
    ("Nh", "nh.cpp:21"),
    ("Os", "npc_os.cpp:1947"),
    ("Plant", "obj_plant.cpp:12"),
    ("Player", "player_main_data.inc:66"),
    ("Rflw", "obj_rflw.cpp:13"),
    ("Rsh1", "npc_rsh1.cpp:46"),
    ("Ship", "ship.cpp:4487,4517"),
    ("Sitem", "sitem.cpp:88,116"),
    ("Sss", "sss.cpp:676,704"),
    ("Tama", "tama.cpp:11"),
    ("Tbox", "tbox.cpp:899"),
    ("Tc", "npc_tc.cpp:1799"),
    ("Tn", "tn.cpp:4010,4039,4068,4096,4124,4152"),
    ("Tt", "npc_tt.cpp:188"),
    ("Vfan", "obj_vfan.cpp:19"),
    ("Zl1", "npc_zl1.cpp:2842"),
    ("amos", "am.cpp:1237,1268,1299,1331"),
    ("amos2", "am2.cpp:1353,1385,1417,1449"),
    ("bridge", "bridge.cpp:1251"),
    ("c_black", "cc.cpp:2524"),
    ("c_blue", "cc.cpp:2524"),
    ("c_green", "cc.cpp:2524"),
    ("c_kiiro", "cc.cpp:2524"),
    ("c_red", "cc.cpp:2524"),
    ("keeth", "ki.cpp:1283,1311,1339"),
    ("kuro_s", "ks.cpp:1641"),
    ("mo2", "mo2.cpp:3899,3928,3957,3985,4013,4041"),
    ("zouK", "obj_zouK.cpp:28"),
];

/// Every port actor name with a cited retail collider set, and the volumes it registers. A name
/// that is absent falls back to [`DEFAULT_RADIUS`] / [`DEFAULT_HEIGHT`].
const TABLE: &[(&str, &[Volume])] = &[
    (
        "Ah",
        &[
            cyl(
                [0.0, 0.0, 0.0],
                60.0,
                150.0,
                set(0x00, 0x00, 0, 0x03, 0xFFFFFFBF, 0x75),
                gobj(0x00, 0x04, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Ajav",
        &[
            sph(
                [0.0, 0.0, 0.0],
                280.0,
                set(0x00, 0x00, 0, 0x0F, 0x20, 0x00),
                gobj(0x00, 0x04, 0x00, 0x00),
                None,
            ),
            cyl(
                [0.0, -40.0, 0.0],
                780.0,
                3300.0,
                set(0x00, 0x00, 0, 0x09, 0x20, 0x00),
                gobj(0x00, 0x06, 0x00, 0x00),
                None,
            ),
            cyl(
                [0.0, -40.0, 0.0],
                780.0,
                3300.0,
                set(0x00, 0x00, 0, 0x09, 0x20, 0x00),
                gobj(0x00, 0x06, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "BFlower",
        &[
            cyl(
                [0.0, 0.0, 0.0],
                50.0,
                10.0,
                set(0x00, 0x00, 0, 0x09, 0x100, 0x00),
                gobj(0x00, 0x04, 0x00, 0x00),
                None,
            ),
            sph(
                [0.0, 0.0, 0.0],
                30.0,
                set(0x00, 0x00, 0, 0x09, 0xFF1DFEFF, 0x79),
                gobj(0x00, 0x04, 0x00, 0x00),
                None,
            ),
            sph(
                [0.0, 0.0, 0.0],
                15.0,
                set(0x00, 0x00, 0, 0x00, 0x00, 0x79),
                gobj(0x00, 0x04, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Ba1",
        &[
            cyl(
                [0.0, 0.0, 0.0],
                50.0,
                110.0,
                set(0x00, 0x00, 0, 0x03, 0xFFFFFFBF, 0x75),
                gobj(0x00, 0x04, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Bb",
        &[
            sph(
                [0.0, 0.0, 0.0],
                62.5,
                set(0x05, 0x800, 1, 0x00, 0x00, 0x00),
                gobj(0x01, 0x00, 0x00, 0x00),
                Some(AtMeta { at_type: AtkType(0x800), at_se: 5, at_spl: 0, at_atp: 1 }),
            ),
            sph(
                [0.0, 0.0, 0.0],
                62.5,
                set(0x00, 0x00, 1, 0x03, 0xFF1DFEFF, 0x00),
                gobj(0x00, 0x06, 0x00, 0x00),
                None,
            ),
            sph(
                [0.0, 0.0, 0.0],
                62.5,
                set(0x00, 0x00, 1, 0x03, 0xFF1DFEFF, 0x00),
                gobj(0x00, 0x06, 0x00, 0x00),
                None,
            ),
            sph(
                [0.0, 0.0, 0.0],
                87.5,
                set(0x00, 0x00, 1, 0x00, 0x00, 0x75),
                gobj(0x00, 0x00, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Bdk",
        &[
            sph(
                [0.0, 0.0, 0.0],
                70.0,
                set(0x0F, 0x800, 2, 0x00, 0x00, 0x75),
                gobj(0x01, 0x00, 0x00, 0x00),
                Some(AtMeta { at_type: AtkType(0x800), at_se: 5, at_spl: 9, at_atp: 2 }),
            ),
            sph(
                [0.0, 0.0, 0.0],
                120.0,
                set(0x00, 0x00, 1, 0x03, 0xFF1DFEFF, 0x75),
                gobj(0x00, 0x03, 0x00, 0x00),
                None,
            ),
            sph(
                [0.0, 0.0, 0.0],
                30.0,
                set(0x00, 0x00, 1, 0x03, 0xFF1DFEFF, 0x75),
                gobj(0x00, 0x02, 0x00, 0x00),
                None,
            ),
            sph(
                [0.0, 0.0, 0.0],
                250.0,
                set(0x00, 0x00, 1, 0x03, 0xFF1DFEFF, 0x75),
                gobj(0x00, 0x02, 0x00, 0x00),
                None,
            ),
            sph(
                [0.0, 0.0, 0.0],
                50.0,
                set(0x0F, 0x800, 2, 0x03, 0xFF1DFEFF, 0x75),
                gobj(0x00, 0x03, 0x00, 0x00),
                Some(AtMeta { at_type: AtkType(0x800), at_se: 0, at_spl: 0, at_atp: 2 }),
            ),
            sph(
                [0.0, 0.0, 0.0],
                300.0,
                set(0x0F, 0x200000, 10, 0x00, 0x00, 0x00),
                gobj(0x03, 0x00, 0x00, 0x00),
                Some(AtMeta { at_type: AtkType(0x200000), at_se: 0, at_spl: 1, at_atp: 10 }),
            ),
            sph(
                [0.0, 0.0, 0.0],
                70.0,
                set(0x00, 0x00, 0, 0x03, 0xFF3DFEFF, 0x15),
                gobj(0x00, 0x03, 0x00, 0x00),
                None,
            ),
            sph(
                [0.0, 0.0, 0.0],
                100.0,
                set(0x00, 0x00, 0, 0x03, 0x200000, 0x00),
                gobj(0x00, 0x00, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Bdkobj",
        &[
            cyl(
                [0.0, 0.0, 0.0],
                100.0,
                450.0,
                set(0x00, 0x00, 0, 0x03, 0xFF1DFEBF, 0x00),
                gobj(0x00, 0x03, 0x00, 0x00),
                None,
            ),
            sph(
                [0.0, 0.0, 0.0],
                30.0,
                set(0x00, 0x00, 0, 0x03, 0xFF3DFEFF, 0x75),
                gobj(0x00, 0x03, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Bgn",
        &[
            sph(
                [0.0, 0.0, 0.0],
                50.0,
                set(0x05, 0x08, 4, 0x03, 0xFF1DFEBF, 0x15),
                gobj(0x01, 0x03, 0x00, 0x00),
                Some(AtMeta { at_type: AtkType(0x08), at_se: 0, at_spl: 9, at_atp: 4 }),
            ),
            sph(
                [0.0, 0.0, 0.0],
                70.0,
                set(0x00, 0x00, 0, 0x03, 0x100000, 0x00),
                gobj(0x01, 0x02, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Bgn2",
        &[
            sph(
                [0.0, 0.0, 0.0],
                50.0,
                set(0x0F, 0x08, 4, 0x03, 0xFF1DFEBF, 0x15),
                gobj(0x01, 0x03, 0x00, 0x00),
                Some(AtMeta { at_type: AtkType(0x08), at_se: 0, at_spl: 10, at_atp: 4 }),
            ),
            sph(
                [0.0, 0.0, 0.0],
                70.0,
                set(0x00, 0x00, 0, 0x03, 0x100000, 0x15),
                gobj(0x01, 0x02, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Bgn3",
        &[
            sph(
                [0.0, 0.0, 0.0],
                50.0,
                set(0x0F, 0x08, 2, 0x03, 0xFF1DFEBF, 0x15),
                gobj(0x01, 0x03, 0x00, 0x00),
                Some(AtMeta { at_type: AtkType(0x08), at_se: 0, at_spl: 9, at_atp: 2 }),
            ),
            sph(
                [0.0, 0.0, 0.0],
                70.0,
                set(0x00, 0x00, 0, 0x03, 0x100000, 0x15),
                gobj(0x01, 0x02, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Bita",
        &[
            cyl(
                [0.0, 0.0, 0.0],
                50.0,
                20.0,
                set(0x00, 0x00, 0, 0x09, 0x200, 0x00),
                gobj(0x00, 0x00, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Bk",
        &[
            cyl(
                [0.0, 0.0, 0.0],
                62.5,
                100.0,
                set(0x00, 0x00, 0, 0x00, 0x00, 0x75),
                gobj(0x00, 0x02, 0x00, 0x00),
                None,
            ),
            cyl(
                [0.0, 0.0, 0.0],
                30.0,
                112.5,
                set(0x00, 0x00, 0, 0x03, 0xFF1DFEFF, 0x00),
                gobj(0x00, 0x02, 0x00, 0x00),
                None,
            ),
            sph(
                [0.0, 0.0, 0.0],
                37.5,
                set(0x00, 0x00, 0, 0x03, 0xFF1DFEFF, 0x00),
                gobj(0x00, 0x02, 0x00, 0x00),
                None,
            ),
            sph(
                [0.0, 0.0, 0.0],
                100.0,
                set(0x0F, 0x800, 2, 0x00, 0x00, 0x00),
                gobj(0x01, 0x00, 0x00, 0x00),
                Some(AtMeta { at_type: AtkType(0x800), at_se: 0, at_spl: 0, at_atp: 2 }),
            ),
            sph(
                [0.0, 0.0, 0.0],
                62.5,
                set(0x00, 0x00, 0, 0x03, 0xFF1DFEFF, 0x00),
                gobj(0x00, 0x03, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Bm1",
        &[
            cyl(
                [0.0, 0.0, 0.0],
                50.0,
                160.0,
                set(0x00, 0x00, 0, 0x03, 0xFFFFFFBF, 0x75),
                gobj(0x00, 0x04, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Bmdhand",
        &[
            sph(
                [0.0, 0.0, 0.0],
                100.0,
                set(0x05, 0x00, 0, 0x03, 0xFF1DFEFF, 0x75),
                gobj(0x00, 0x00, 0x00, 0x00),
                Some(AtMeta { at_type: AtkType(0x00), at_se: 0, at_spl: 0, at_atp: 0 }),
            ),
        ],
    ),
    (
        "Bmsw",
        &[
            cyl(
                [0.0, 0.0, 0.0],
                30.0,
                80.0,
                set(0x00, 0x00, 0, 0x03, 0xFFFFFFBF, 0x75),
                gobj(0x00, 0x04, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Boko",
        &[
            sph(
                [0.0, 0.0, 0.0],
                20.0,
                set(0x0E, 0x20000, 0, 0x09, 0x20300, 0x00),
                gobj(0x02, 0x00, 0x00, 0x00),
                None,
            ),
            cps(
                [0.0, 0.0, 0.0],
                20.0,
                0.0,
                set(0x0B, 0x80, 1, 0x00, 0x00, 0x00),
                gobj(0x00, 0x00, 0x00, 0x00),
                Some(AtMeta { at_type: AtkType(0x80), at_se: 0, at_spl: 1, at_atp: 1 }),
            ),
        ],
    ),
    (
        "Bs1",
        &[
            cyl(
                [0.0, 0.0, 0.0],
                30.0,
                80.0,
                set(0x00, 0x00, 0, 0x03, 0xFFFFFFFF, 0x75),
                gobj(0x00, 0x04, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Bst",
        &[
            cyl(
                [0.0, 0.0, 0.0],
                100.0,
                400.0,
                set(0x05, 0x800, 1, 0x03, 0xFF1DFEFF, 0x75),
                gobj(0x00, 0x03, 0x00, 0x00),
                Some(AtMeta { at_type: AtkType(0x800), at_se: 0, at_spl: 0, at_atp: 1 }),
            ),
            cyl(
                [0.0, 0.0, 0.0],
                50.0,
                200.0,
                set(0x00, 0x00, 0, 0x03, 0xF71DFEBF, 0x45),
                gobj(0x00, 0x02, 0x00, 0x00),
                None,
            ),
            sph(
                [0.0, 0.0, 0.0],
                30.0,
                set(0x05, 0x800, 1, 0x03, 0xFF1DFEFF, 0x75),
                gobj(0x01, 0x03, 0x00, 0x00),
                Some(AtMeta { at_type: AtkType(0x800), at_se: 0, at_spl: 0, at_atp: 1 }),
            ),
            sph(
                [0.0, 0.0, 0.0],
                30.0,
                set(0x00, 0x00, 0, 0x03, 0xF71DFEBF, 0x00),
                gobj(0x01, 0x02, 0x00, 0x00),
                None,
            ),
            sph(
                [0.0, 0.0, 0.0],
                60.0,
                set(0x05, 0x200, 2, 0x00, 0x00, 0x00),
                gobj(0x00, 0x00, 0x00, 0x00),
                Some(AtMeta { at_type: AtkType(0x200), at_se: 0, at_spl: 10, at_atp: 2 }),
            ),
        ],
    ),
    (
        "Btd",
        &[
            sph(
                [0.0, 0.0, 0.0],
                200.0,
                set(0x0F, 0x1000, 1, 0x03, 0xFF1DFEFF, 0x75),
                gobj(0x00, 0x01, 0x00, 0x00),
                Some(AtMeta { at_type: AtkType(0x1000), at_se: 0, at_spl: 9, at_atp: 1 }),
            ),
            sph(
                [0.0, 0.0, 0.0],
                50.0,
                set(0x05, 0x00, 0, 0x03, 0xFF1DFEFF, 0x75),
                gobj(0x00, 0x01, 0x00, 0x00),
                Some(AtMeta { at_type: AtkType(0x00), at_se: 0, at_spl: 0, at_atp: 0 }),
            ),
            sph(
                [0.0, 0.0, 0.0],
                100.0,
                set(0x0F, 0x200, 2, 0x03, 0x00, 0x00),
                gobj(0x00, 0x00, 0x00, 0x00),
                Some(AtMeta { at_type: AtkType(0x200), at_se: 0, at_spl: 10, at_atp: 2 }),
            ),
            sph(
                [0.0, 0.0, 0.0],
                40.0,
                set(0x05, 0x200, 1, 0x03, 0x00, 0x00),
                gobj(0x00, 0x00, 0x00, 0x00),
                Some(AtMeta { at_type: AtkType(0x200), at_se: 0, at_spl: 2, at_atp: 1 }),
            ),
        ],
    ),
    (
        "Btsw",
        &[
            cyl(
                [0.0, 0.0, 0.0],
                30.0,
                80.0,
                set(0x00, 0x00, 0, 0x03, 0xFFFFFFBF, 0x75),
                gobj(0x00, 0x04, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Btsw2",
        &[
            cyl(
                [0.0, 0.0, 0.0],
                30.0,
                80.0,
                set(0x00, 0x00, 0, 0x03, 0xFFFFFFBF, 0x75),
                gobj(0x00, 0x04, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Bwd",
        &[
            sph(
                [0.0, 0.0, 0.0],
                15.0,
                set(0x05, 0x800, 8, 0x03, 0xFF1DFEFF, 0x75),
                gobj(0x01, 0x03, 0x00, 0x00),
                Some(AtMeta { at_type: AtkType(0x800), at_se: 0, at_spl: 9, at_atp: 8 }),
            ),
            sph(
                [0.0, 0.0, 0.0],
                70.0,
                set(0x00, 0x00, 0, 0x03, 0xFF1DFEFF, 0x00),
                gobj(0x00, 0x02, 0x00, 0x00),
                None,
            ),
            sph(
                [0.0, 0.0, 0.0],
                70.0,
                set(0x00, 0x00, 0, 0x03, 0xFF1DFEFF, 0x75),
                gobj(0x00, 0x02, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Bwds",
        &[
            sph(
                [0.0, 0.0, 0.0],
                15.0,
                set(0x05, 0x800, 2, 0x03, 0xFF1D7EFF, 0x75),
                gobj(0x00, 0x02, 0x00, 0x00),
                Some(AtMeta { at_type: AtkType(0x800), at_se: 0, at_spl: 0, at_atp: 2 }),
            ),
            sph(
                [0.0, 0.0, 0.0],
                15.0,
                set(0x00, 0x00, 0, 0x03, 0xFF1D7EFF, 0x00),
                gobj(0x00, 0x03, 0x00, 0x00),
                None,
            ),
            sph(
                [0.0, 0.0, 0.0],
                15.0,
                set(0x00, 0x00, 0, 0x03, 0x8000, 0x00),
                gobj(0x00, 0x02, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Canon",
        &[
            cyl(
                [0.0, 0.0, 0.0],
                75.0,
                50.0,
                set(0x00, 0x00, 0, 0x00, 0x00, 0x79),
                gobj(0x00, 0x00, 0x00, 0x00),
                None,
            ),
            cyl(
                [0.0, 0.0, 0.0],
                300.0,
                500.0,
                set(0x0B, 0x20, 0, 0x00, 0x00, 0x00),
                gobj(0x00, 0x04, 0x00, 0x00),
                Some(AtMeta { at_type: AtkType(0x20), at_se: 0, at_spl: 0, at_atp: 0 }),
            ),
        ],
    ),
    (
        "Cb1",
        &[
            cyl(
                [0.0, 0.0, 0.0],
                30.0,
                80.0,
                set(0x00, 0x00, 0, 0x05, 0xFF5FFFBF, 0x75),
                gobj(0x00, 0x04, 0x00, 0x00),
                None,
            ),
            cyl(
                [0.0, 0.0, 0.0],
                30.0,
                80.0,
                set(0x00, 0x00, 0, 0x05, 0xFF3DFEFF, 0x00),
                gobj(0x00, 0x04, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Doguu",
        &[
            cyl(
                [0.0, 0.0, 0.0],
                200.0,
                200.0,
                set(0x00, 0x00, 0, 0x00, 0x00, 0x19),
                gobj(0x00, 0x04, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Ds1",
        &[
            cyl(
                [0.0, 0.0, 0.0],
                30.0,
                80.0,
                set(0x00, 0x00, 0, 0x03, 0xFFFFFFFF, 0x75),
                gobj(0x00, 0x04, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Ebomzo",
        &[
            sph(
                [0.0, 0.0, 0.0],
                10.0,
                set(0x00, 0x00, 0, 0x09, 0x20, 0x00),
                gobj(0x00, 0x00, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Ep",
        &[
            sph(
                [0.0, 0.0, 0.0],
                40.0,
                set(0x0E, 0x20000, 0, 0x09, 0x660200, 0x00),
                gobj(0x00, 0x00, 0x00, 0x00),
                None,
            ),
            cyl(
                [0.0, 0.0, 0.0],
                35.0,
                120.0,
                set(0x00, 0x00, 0, 0x0F, 0xFF1DFEBF, 0x79),
                gobj(0x00, 0x01, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Eskban",
        &[
            cyl(
                [0.0, 0.0, 0.0],
                200.0,
                600.0,
                set(0x00, 0x00, 0, 0x09, 0x20, 0x00),
                gobj(0x00, 0x00, 0x00, 0x00),
                None,
            ),
            cyl(
                [0.0, 0.0, 0.0],
                1000.0,
                100.0,
                set(0x00, 0x00, 0, 0x00, 0x00, 0x149),
                gobj(0x00, 0x00, 0x00, 0x00),
                None,
            ),
            sph(
                [0.0, 0.0, 0.0],
                300.0,
                set(0x00, 0x00, 0, 0x09, 0x20, 0x139),
                gobj(0x00, 0x00, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Fganon",
        &[
            cyl(
                [0.0, -150.0, 0.0],
                100.0,
                100.0,
                set(0x00, 0x00, 0, 0x03, 0xFF1DFEF7, 0x75),
                gobj(0x00, 0x02, 0x00, 0x00),
                None,
            ),
            sph(
                [0.0, 0.0, 0.0],
                200.0,
                set(0x0F, 0x08, 0, 0x00, 0x00, 0x00),
                gobj(0x01, 0x02, 0x00, 0x00),
                Some(AtMeta { at_type: AtkType(0x08), at_se: 0, at_spl: 7, at_atp: 0 }),
            ),
            sph(
                [0.0, 0.0, 0.0],
                60.0,
                set(0x00, 0x00, 0, 0x03, 0xFF1DFEFF, 0x00),
                gobj(0x00, 0x00, 0x00, 0x00),
                None,
            ),
            sph(
                [0.0, 0.0, 0.0],
                30.0,
                set(0x0F, 0x800, 2, 0x00, 0x00, 0x00),
                gobj(0x00, 0x00, 0x00, 0x00),
                Some(AtMeta { at_type: AtkType(0x800), at_se: 0, at_spl: 10, at_atp: 2 }),
            ),
        ],
    ),
    (
        "Fgmahou",
        &[
            sph(
                [0.0, 0.0, 0.0],
                60.0,
                set(0x00, 0x00, 0, 0x03, 0x02, 0x00),
                gobj(0x00, 0x02, 0x00, 0x00),
                None,
            ),
            sph(
                [0.0, 0.0, 0.0],
                30.0,
                set(0x0F, 0x800, 8, 0x00, 0x00, 0x00),
                gobj(0x01, 0x00, 0x00, 0x00),
                Some(AtMeta { at_type: AtkType(0x800), at_se: 0, at_spl: 10, at_atp: 8 }),
            ),
        ],
    ),
    (
        "Figure",
        &[
            cyl(
                [0.0, 0.0, 0.0],
                60.0,
                150.0,
                set(0x00, 0x00, 0, 0x03, 0xFFFFFFFF, 0x79),
                gobj(0x00, 0x04, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Fire",
        &[
            cyl(
                [0.0, 0.0, 0.0],
                150.0,
                150.0,
                set(0x05, 0x200, 1, 0x09, 0x200100, 0x00),
                gobj(0x00, 0x04, 0x00, 0x00),
                Some(AtMeta { at_type: AtkType(0x200), at_se: 0, at_spl: 1, at_atp: 1 }),
            ),
            cyl(
                [0.0, 0.0, 0.0],
                130.0,
                170.0,
                set(0x00, 0x00, 0, 0x00, 0x00, 0x19),
                gobj(0x00, 0x04, 0x00, 0x00),
                None,
            ),
            cyl(
                [0.0, 0.0, 0.0],
                150.0,
                150.0,
                set(0x09, 0x20000, 0, 0x09, 0x00, 0x00),
                gobj(0x00, 0x04, 0x00, 0x00),
                Some(AtMeta { at_type: AtkType(0x20000), at_se: 0, at_spl: 0, at_atp: 0 }),
            ),
        ],
    ),
    (
        "Gtaki",
        &[
            cyl(
                [0.0, 0.0, 0.0],
                70.0,
                1000.0,
                set(0x00, 0x00, 0, 0x00, 0x00, 0x119),
                gobj(0x00, 0x04, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Hfuck1",
        &[
            sph(
                [0.0, 0.0, 0.0],
                35.0,
                set(0x00, 0x00, 0, 0x09, 0x8000, 0x00),
                gobj(0x00, 0x06, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Hi1",
        &[
            cyl(
                [0.0, 0.0, 0.0],
                110.0,
                260.0,
                set(0x00, 0x00, 0, 0x03, 0xFFFFFFBF, 0x75),
                gobj(0x00, 0x04, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Hmlif",
        &[
            sph(
                [0.0, 0.0, 0.0],
                35.0,
                set(0x00, 0x00, 0, 0x09, 0x1C4000, 0x00),
                gobj(0x00, 0x04, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Ho",
        &[
            cyl(
                [0.0, 0.0, 0.0],
                40.0,
                140.0,
                set(0x00, 0x00, 0, 0x03, 0xFFFFFFBF, 0x75),
                gobj(0x00, 0x04, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Hr",
        &[
            cyl(
                [0.0, 0.0, 0.0],
                45.0,
                100.0,
                set(0x00, 0x00, 0, 0x03, 0xFFFFFFBF, 0x75),
                gobj(0x00, 0x04, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Hys",
        &[
            sph(
                [0.0, 0.0, 0.0],
                30.0,
                set(0x00, 0x00, 0, 0x09, 0x1C4000, 0x00),
                gobj(0x00, 0x04, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Kantera",
        &[
            sph(
                [0.0, 0.0, 0.0],
                40.0,
                set(0x05, 0x200, 1, 0x00, 0x00, 0x75),
                gobj(0x00, 0x00, 0x00, 0x00),
                Some(AtMeta { at_type: AtkType(0x200), at_se: 0, at_spl: 8, at_atp: 1 }),
            ),
        ],
    ),
    (
        "Kita",
        &[
            sph(
                [0.0, 0.0, 0.0],
                400.0,
                set(0x05, 0x00, 0, 0x03, 0x200000, 0x00),
                gobj(0x00, 0x02, 0x00, 0x00),
                Some(AtMeta { at_type: AtkType(0x00), at_se: 0, at_spl: 0, at_atp: 0 }),
            ),
        ],
    ),
    (
        "Km1",
        &[
            cyl(
                [0.0, 0.0, 0.0],
                60.0,
                150.0,
                set(0x00, 0x00, 0, 0x03, 0xFFFFFFBF, 0x75),
                gobj(0x00, 0x04, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Lamp",
        &[
            sph(
                [0.0, 0.0, 0.0],
                30.0,
                set(0x00, 0x00, 0, 0x09, 0x660200, 0x00),
                gobj(0x00, 0x00, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Ls1",
        &[
            cyl(
                [0.0, 0.0, 0.0],
                40.0,
                100.0,
                set(0x00, 0x00, 0, 0x03, 0xFFFFFFBF, 0x75),
                gobj(0x00, 0x04, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Mflft",
        &[
            cyl(
                [0.0, 0.0, 0.0],
                10.0,
                1000.0,
                set(0x00, 0x00, 0, 0x03, 0xFF1DFEBF, 0x75),
                gobj(0x00, 0x01, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Mk",
        &[
            cyl(
                [0.0, 0.0, 0.0],
                40.0,
                80.0,
                set(0x00, 0x00, 0, 0x03, 0xFFFFFFBF, 0x75),
                gobj(0x00, 0x04, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "MkieK",
        &[
            sph(
                [0.0, 150.0, 0.0],
                100.0,
                set(0x00, 0x00, 0, 0x03, 0x800000, 0x00),
                gobj(0x00, 0x04, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Mn",
        &[
            cyl(
                [0.0, 0.0, 0.0],
                40.0,
                160.0,
                set(0x00, 0x00, 0, 0x03, 0xFFFFFFFF, 0x15),
                gobj(0x00, 0x04, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Mt",
        &[
            cyl(
                [0.0, 0.0, 0.0],
                60.0,
                150.0,
                set(0x00, 0x00, 0, 0x03, 0xFFFFFFBF, 0x75),
                gobj(0x00, 0x04, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Nh",
        &[
            cyl(
                [0.0, 0.0, 0.0],
                10.0,
                20.0,
                set(0x00, 0x00, 0, 0x03, 0xFFFFFFFF, 0x75),
                gobj(0x00, 0x00, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Os",
        &[
            cyl(
                [0.0, 0.0, 0.0],
                30.0,
                120.0,
                set(0x00, 0x00, 0, 0x03, 0xFFFFFFBF, 0x75),
                gobj(0x00, 0x04, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Plant",
        &[
            cyl(
                [0.0, 0.0, 0.0],
                30.0,
                200.0,
                set(0x00, 0x00, 0, 0x09, 0x20, 0x19),
                gobj(0x00, 0x04, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Player",
        &[
            cyl(
                [0.0, 0.0, 0.0],
                30.0,
                125.0,
                set(0x00, 0x00, 0, 0x05, 0xFF1DFEFF, 0x73),
                gobj(0x00, 0x00, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Rflw",
        &[
            cyl(
                [0.0, 0.0, 0.0],
                30.0,
                200.0,
                set(0x00, 0x00, 0, 0x09, 0x20, 0x19),
                gobj(0x00, 0x04, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Rsh1",
        &[
            cyl(
                [0.0, 0.0, 0.0],
                60.0,
                80.0,
                set(0x00, 0x00, 0, 0x03, 0xFFFFFFBF, 0x75),
                gobj(0x00, 0x04, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Ship",
        &[
            cyl(
                [0.0, 0.0, 0.0],
                75.0,
                115.0,
                set(0x00, 0x00, 0, 0x05, 0xFF3DFEFF, 0x79),
                gobj(0x00, 0x00, 0x00, 0x00),
                None,
            ),
            sph(
                [0.0, 0.0, 0.0],
                75.0,
                set(0x00, 0x00, 0, 0x05, 0xFF3DFEFF, 0x79),
                gobj(0x00, 0x00, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Sitem",
        &[
            sph(
                [0.0, 0.0, 0.0],
                20.0,
                set(0x05, 0x800, 0, 0x03, 0xFF1DFEBF, 0x75),
                gobj(0x00, 0x02, 0x00, 0x00),
                Some(AtMeta { at_type: AtkType(0x800), at_se: 0, at_spl: 0, at_atp: 0 }),
            ),
            sph(
                [0.0, 0.0, 0.0],
                80.0,
                set(0x00, 0x00, 0, 0x03, 0x200040, 0x00),
                gobj(0x00, 0x02, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Sss",
        &[
            sph(
                [0.0, 0.0, 0.0],
                20.0,
                set(0x05, 0x00, 0, 0x03, 0xFF1DFEBF, 0x75),
                gobj(0x00, 0x02, 0x00, 0x00),
                Some(AtMeta { at_type: AtkType(0x00), at_se: 0, at_spl: 0, at_atp: 0 }),
            ),
            sph(
                [0.0, 0.0, 0.0],
                80.0,
                set(0x05, 0x00, 0, 0x03, 0x200040, 0x00),
                gobj(0x00, 0x02, 0x00, 0x00),
                Some(AtMeta { at_type: AtkType(0x00), at_se: 0, at_spl: 0, at_atp: 0 }),
            ),
        ],
    ),
    (
        "Tama",
        &[
            sph(
                [0.0, 0.0, 0.0],
                30.0,
                set(0x00, 0x00, 0, 0x03, 0xFFFFFFFF, 0x55),
                gobj(0x00, 0x04, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Tbox",
        &[
            cyl(
                [0.0, 0.0, 0.0],
                40.0,
                110.0,
                set(0x00, 0x00, 0, 0x03, 0xFFFFFFBF, 0x75),
                gobj(0x00, 0x04, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Tc",
        &[
            cyl(
                [0.0, 0.0, 0.0],
                60.0,
                150.0,
                set(0x00, 0x00, 0, 0x03, 0xFFFFFFBF, 0x75),
                gobj(0x00, 0x04, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Tn",
        &[
            cyl(
                [0.0, 0.0, 0.0],
                90.0,
                125.0,
                set(0x00, 0x00, 0, 0x00, 0x00, 0x75),
                gobj(0x00, 0x02, 0x00, 0x00),
                None,
            ),
            cyl(
                [0.0, 0.0, 0.0],
                45.0,
                137.5,
                set(0x00, 0x00, 0, 0x03, 0xFF1DFEFF, 0x00),
                gobj(0x00, 0x03, 0x00, 0x00),
                None,
            ),
            sph(
                [0.0, 0.0, 0.0],
                55.0,
                set(0x00, 0x00, 1, 0x03, 0xFF1DFEFF, 0x00),
                gobj(0x00, 0x03, 0x00, 0x00),
                None,
            ),
            sph(
                [0.0, 0.0, 0.0],
                100.0,
                set(0x0F, 0x10008, 0, 0x00, 0x00, 0x00),
                gobj(0x01, 0x02, 0x00, 0x00),
                Some(AtMeta { at_type: AtkType(0x10008), at_se: 0, at_spl: 0, at_atp: 0 }),
            ),
            sph(
                [0.0, 0.0, 0.0],
                62.5,
                set(0x0F, 0x10008, 0, 0x00, 0x00, 0x00),
                gobj(0x01, 0x02, 0x00, 0x00),
                Some(AtMeta { at_type: AtkType(0x10008), at_se: 0, at_spl: 0, at_atp: 0 }),
            ),
            sph(
                [0.0, 0.0, 0.0],
                62.5,
                set(0x00, 0x00, 1, 0x03, 0xFF1DFEFF, 0x00),
                gobj(0x00, 0x03, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Tt",
        &[
            cyl(
                [0.0, 0.0, 0.0],
                48.0,
                140.0,
                set(0x00, 0x00, 0, 0x03, 0xFFFFFFBF, 0x75),
                gobj(0x00, 0x04, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Vfan",
        &[
            cyl(
                [0.0, 0.0, 0.0],
                100.0,
                300.0,
                set(0x00, 0x00, 0, 0x09, 0x20000000, 0x00),
                gobj(0x00, 0x00, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "Zl1",
        &[
            cyl(
                [0.0, 0.0, 0.0],
                50.0,
                140.0,
                set(0x00, 0x00, 0, 0x03, 0xFFFFFFBF, 0x75),
                gobj(0x00, 0x04, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "amos",
        &[
            sph(
                [0.0, 0.0, 0.0],
                15.0,
                set(0x00, 0x1000, 0, 0x03, 0x81C4000, 0x00),
                gobj(0x00, 0x06, 0x00, 0x00),
                None,
            ),
            sph(
                [0.0, 0.0, 0.0],
                15.0,
                set(0x00, 0x1000, 0, 0x00, 0x00, 0x145),
                gobj(0x00, 0x02, 0x00, 0x00),
                None,
            ),
            sph(
                [0.0, 0.0, 0.0],
                15.0,
                set(0x00, 0x1000, 0, 0x03, 0xFF1DFEFF, 0x75),
                gobj(0x00, 0x03, 0x00, 0x00),
                None,
            ),
            sph(
                [0.0, 0.0, 0.0],
                15.0,
                set(0x0F, 0x1000, 2, 0x00, 0x00, 0x75),
                gobj(0x00, 0x03, 0x00, 0x00),
                Some(AtMeta { at_type: AtkType(0x1000), at_se: 0, at_spl: 6, at_atp: 2 }),
            ),
        ],
    ),
    (
        "amos2",
        &[
            sph(
                [0.0, 0.0, 0.0],
                15.0,
                set(0x00, 0x1000, 0, 0x03, 0x81C4000, 0x00),
                gobj(0x00, 0x06, 0x00, 0x00),
                None,
            ),
            sph(
                [0.0, 0.0, 0.0],
                15.0,
                set(0x00, 0x1000, 0, 0x03, 0xFF1DFEFF, 0x00),
                gobj(0x00, 0x06, 0x00, 0x00),
                None,
            ),
            sph(
                [0.0, 0.0, 0.0],
                15.0,
                set(0x00, 0x1000, 0, 0x03, 0xFF1DFEFF, 0x75),
                gobj(0x00, 0x03, 0x00, 0x00),
                None,
            ),
            sph(
                [0.0, 0.0, 0.0],
                15.0,
                set(0x05, 0x1000, 1, 0x00, 0x00, 0x61),
                gobj(0x00, 0x03, 0x00, 0x00),
                Some(AtMeta { at_type: AtkType(0x1000), at_se: 0, at_spl: 6, at_atp: 1 }),
            ),
        ],
    ),
    (
        "bridge",
        &[
            cyl(
                [0.0, 0.0, 0.0],
                5.0,
                1000.0,
                set(0x00, 0x00, 0, 0x03, 0xFF1DFEBF, 0x75),
                gobj(0x00, 0x01, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "c_black",
        &[
            sph(
                [0.0, 0.0, 0.0],
                15.0,
                set(0x05, 0x2000, 0, 0x0B, 0xFFBDFEFF, 0x75),
                gobj(0x00, 0x06, 0x00, 0x00),
                Some(AtMeta { at_type: AtkType(0x2000), at_se: 6, at_spl: 0, at_atp: 0 }),
            ),
        ],
    ),
    (
        "c_blue",
        &[
            sph(
                [0.0, 0.0, 0.0],
                15.0,
                set(0x05, 0x2000, 0, 0x0B, 0xFFBDFEFF, 0x75),
                gobj(0x00, 0x06, 0x00, 0x00),
                Some(AtMeta { at_type: AtkType(0x2000), at_se: 6, at_spl: 0, at_atp: 0 }),
            ),
        ],
    ),
    (
        "c_green",
        &[
            sph(
                [0.0, 0.0, 0.0],
                15.0,
                set(0x05, 0x2000, 0, 0x0B, 0xFFBDFEFF, 0x75),
                gobj(0x00, 0x06, 0x00, 0x00),
                Some(AtMeta { at_type: AtkType(0x2000), at_se: 6, at_spl: 0, at_atp: 0 }),
            ),
        ],
    ),
    (
        "c_kiiro",
        &[
            sph(
                [0.0, 0.0, 0.0],
                15.0,
                set(0x05, 0x2000, 0, 0x0B, 0xFFBDFEFF, 0x75),
                gobj(0x00, 0x06, 0x00, 0x00),
                Some(AtMeta { at_type: AtkType(0x2000), at_se: 6, at_spl: 0, at_atp: 0 }),
            ),
        ],
    ),
    (
        "c_red",
        &[
            sph(
                [0.0, 0.0, 0.0],
                15.0,
                set(0x05, 0x2000, 0, 0x0B, 0xFFBDFEFF, 0x75),
                gobj(0x00, 0x06, 0x00, 0x00),
                Some(AtMeta { at_type: AtkType(0x2000), at_se: 6, at_spl: 0, at_atp: 0 }),
            ),
        ],
    ),
    (
        "keeth",
        &[
            sph(
                [0.0, 0.0, 0.0],
                30.0,
                set(0x05, 0x800, 1, 0x00, 0x00, 0x00),
                gobj(0x01, 0x00, 0x00, 0x00),
                Some(AtMeta { at_type: AtkType(0x800), at_se: 0, at_spl: 0, at_atp: 1 }),
            ),
            sph(
                [0.0, 0.0, 0.0],
                40.0,
                set(0x00, 0x00, 0, 0x03, 0xFF3DFEFF, 0x00),
                gobj(0x00, 0x02, 0x00, 0x00),
                None,
            ),
            sph(
                [0.0, 0.0, 0.0],
                80.0,
                set(0x00, 0x00, 0, 0x00, 0x00, 0x25),
                gobj(0x00, 0x00, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "kuro_s",
        &[
            sph(
                [0.0, 0.0, 0.0],
                15.0,
                set(0x15, 0x1000000, 0, 0x03, 0xFF3DFEFF, 0x65),
                gobj(0x00, 0x06, 0x00, 0x00),
                Some(AtMeta { at_type: AtkType(0x1000000), at_se: 6, at_spl: 0, at_atp: 0 }),
            ),
        ],
    ),
    (
        "mo2",
        &[
            cyl(
                [0.0, 0.0, 0.0],
                90.0,
                125.0,
                set(0x00, 0x00, 0, 0x00, 0x00, 0x75),
                gobj(0x00, 0x02, 0x00, 0x00),
                None,
            ),
            cyl(
                [0.0, 0.0, 0.0],
                45.0,
                137.5,
                set(0x00, 0x00, 0, 0x03, 0xFF1DFEFF, 0x00),
                gobj(0x00, 0x02, 0x00, 0x00),
                None,
            ),
            sph(
                [0.0, 0.0, 0.0],
                55.0,
                set(0x00, 0x00, 1, 0x03, 0xFF1DFEFF, 0x00),
                gobj(0x00, 0x02, 0x00, 0x00),
                None,
            ),
            sph(
                [0.0, 0.0, 0.0],
                100.0,
                set(0x0F, 0x08, 0, 0x00, 0x00, 0x00),
                gobj(0x01, 0x02, 0x00, 0x00),
                Some(AtMeta { at_type: AtkType(0x08), at_se: 0, at_spl: 0, at_atp: 0 }),
            ),
            sph(
                [0.0, 0.0, 0.0],
                62.5,
                set(0x0F, 0x08, 0, 0x00, 0x00, 0x00),
                gobj(0x01, 0x02, 0x00, 0x00),
                Some(AtMeta { at_type: AtkType(0x08), at_se: 0, at_spl: 0, at_atp: 0 }),
            ),
            sph(
                [0.0, 0.0, 0.0],
                62.5,
                set(0x00, 0x00, 1, 0x03, 0xFF1DFEFF, 0x75),
                gobj(0x00, 0x03, 0x00, 0x00),
                None,
            ),
        ],
    ),
    (
        "zouK",
        &[
            cyl(
                [0.0, 0.0, 0.0],
                100.0,
                200.0,
                set(0x00, 0x00, 0, 0x0F, 0xFF1DFEFF, 0x79),
                gobj(0x00, 0x03, 0x00, 0x00),
                None,
            ),
        ],
    ),
];

/// The volumes `name`'s own reference `create` registers.
pub fn rows_for(name: &str) -> Option<&'static [Volume]> {
    TABLE
        .iter()
        .find(|(actor, _)| *actor == name)
        .map(|(_, volumes)| *volumes)
}

/// The whole collider set for one actor name, ready to hand to the contact pass.
pub fn bodies_for(name: &str) -> ContactBodies {
    let mut bodies = ContactBodies::default();
    if let Some(volumes) = rows_for(name) {
        for volume in volumes {
            bodies.add(volume.shape(), volume.obj());
        }
        return bodies;
    }
    bodies.add(
        Shape::capsule(Vec3::ZERO, DEFAULT_RADIUS, DEFAULT_HEIGHT),
        default_obj(),
    );
    bodies
}

fn default_obj() -> ContactObj {
    ContactObj {
        set: set(0x00, 0x00, 0, 0x03, 0xFFFF_FFBF, 0x75),
        g_stts: Some(GStts::default()),
        gobj_inf: Some(gobj(0x00, 0x04, 0x00, 0x00)),
    }
}

fn anchor(bodies: &mut ContactBodies, pos: Vec3) {
    for index in 0..bodies.shapes.bodies.len() {
        let offset = bodies
            .shapes
            .body(index)
            .map_or(Vec3::ZERO, |body| body.shape.pos);
        bodies.follow(index, pos + offset);
    }
}

/// Give every entity in `spawned` the collider set its name carries, put each body on the
/// position the actor was spawned at, and stamp the weight its process's `cCcD_Stts::Init` was
/// handed, which `super::weight` already tabulates and which nothing else reaches for a stage
/// actor.
pub fn attach(world: &mut World, spawned: &[Entity]) {
    for id in spawned {
        let Ok(mut entity) = world.get_entity_mut(*id) else {
            continue;
        };
        let Some(actor) = entity.get::<Actor>() else {
            continue;
        };
        let name = actor.name.clone();
        let pos = actor.pos;
        let mut bodies = bodies_for(&name);
        anchor(&mut bodies, pos);
        entity.insert(bodies);
        if let Some(mut actor) = entity.get_mut::<Actor>() {
            actor.weight_uc = weight::weight_for(&name);
            actor.no_crr = weight::no_crr_for(&name);
        }
    }
}

/// `cCcS::MoveObj` re-registration, once per step: put every body back on the transform its owner
/// just computed. Retail processes rebuild their colliders from `current.pos` each step, which is
/// what keeps a head sphere above the feet after the owner moves; without this a body set at spawn
/// would sit still while its actor walks away from it. Only the position moves, so the local
/// offset a volume's source constant carries survives.
pub fn follow(mut actors: Query<(&Actor, &mut ContactBodies)>) {
    for (actor, mut bodies) in &mut actors {
        anchor(&mut bodies, actor.pos);
    }
}