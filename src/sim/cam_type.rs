use super::cam_style::CamStyleId;

/// One row of `dCamera_c::types`; twenty style slots indexed by `mCurMode`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CamType {
    pub name: &'static str,
    pub styles: [i16; 20],
}

macro_rules! slots {
    ($($style:ident),+ $(,)?) => {
        [$(CamStyleId::$style as i16),+]
    };
}

/// `dCamera_c::type_num`.
pub const TYPE_COUNT: usize = 63;

/// `dCamera_c::types`.
pub const TYPES: &[CamType] = &[
    CamType { name: "Empty", styles: [CamStyleId::NONE as i16; 20] },
    CamType {
        name: "Plain",
        styles: slots! {
            FN03, FN03, LL03, TT03, SS01, LW01, LE01, LH01, LA01, LB01, SX01, SY01, MM01, CC01,
            SN15, JN05, EN00, HN18, HN16, DD01,
        },
    },
    CamType {
        name: "Keep",
        styles: slots! {
            NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE,
            NONE, NONE, EN00, NONE, NONE, NONE,
        },
    },
    CamType {
        name: "Event",
        styles: slots! {
            EN00, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE,
            NONE, NONE, EN00, NONE, NONE, NONE,
        },
    },
    CamType {
        name: "Boss01",
        styles: slots! {
            FN15, FN15, LL05, TT02, SS01, LW01, LE01, LH02, LA01, LB01, SX02, SY01, MM05, CC01,
            SN15, JN05, EN00, HN18, HN16, DD01,
        },
    },
    CamType {
        name: "Cliff",
        styles: slots! {
            FN19, FN19, LL06, TT02, SS01, LW02, LE01, LH02, LA02, LB02, SX01, SY01, MM04, CC01,
            SN15, JN05, EN00, HN18, HN16, DD01,
        },
    },
    CamType {
        name: "PlaySlide",
        styles: slots! {
            FN20, FN20, LL01, TT01, SS01, LW01, LE01, LH01, LA01, LB01, SX01, SY01, MM01, CC01,
            SN15, JN05, EN00, HN18, HN16, DD01,
        },
    },
    CamType {
        name: "Field",
        styles: slots! {
            FN01, FN01, LL01, TT01, SS01, LW01, LE01, LH01, LA01, LB01, SX01, SY01, MM01, CC01,
            SN15, JN05, EN00, HN18, HN16, DD01,
        },
    },
    CamType {
        name: "ThrowUp",
        styles: slots! {
            FN21, FN21, LL01, TT01, SS01, LW01, LE01, LH01, LA01, LB01, SX01, SY01, MM09, CC01,
            SN15, JN05, EN00, HN18, HN16, DD01,
        },
    },
    CamType {
        name: "Gamoss",
        styles: slots! {
            FN23, FN23, LL09, TT02, SS01, LW01, LE01, LH02, LA02, LB02, SX01, SY01, MM03, CC01,
            SN15, JN05, EN00, HN18, HN16, DD01,
        },
    },
    CamType {
        name: "MiniIsland",
        styles: slots! {
            FN24, FN24, LL01, TT01, SS01, LW01, LE01, LH01, LA01, LB01, SX01, SY01, MM10, CC01,
            SN15, JN05, EN00, HN18, HN16, DD01,
        },
    },
    CamType {
        name: "Cliff2",
        styles: slots! {
            FN26, FN07, LL07, TT02, SS01, LW01, LE01, LH01, LA02, LB02, SX01, SY01, MM08, CC01,
            SN15, JN05, EN00, HN18, HN16, DD01,
        },
    },
    CamType {
        name: "FieldCushion",
        styles: slots! {
            FN10, FN10, LL01, TT01, SS01, LW01, LE01, LH01, LA01, LB01, SX01, SY01, MM01, CC01,
            SN15, JN05, EN00, HN18, HN16, DD01,
        },
    },
    CamType {
        name: "FixdPos",
        styles: slots! {
            PN05, FN02, LL02, TT02, SS01, NONE, NONE, LH02, LA01, LB01, SX01, SY01, MM01, CC01,
            SN15, JN05, EN00, HN18, HN16, DD01,
        },
    },
    CamType {
        name: "SmoothFixdPos",
        styles: slots! {
            PN12, FN02, LL02, TT02, SS01, NONE, NONE, LH02, LA01, LB01, SX01, SY01, MM01, CC01,
            SN15, JN05, EN00, HN18, HN16, DD01,
        },
    },
    CamType {
        name: "WaterBattle",
        styles: slots! {
            FN25, FN25, LL11, TT02, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, MM15, NONE,
            SN15, JN05, EN00, HN18, HN16, DD01,
        },
    },
    CamType {
        name: "SmoothFixdPosC",
        styles: slots! {
            PN13, FN02, LL02, TT02, SS01, NONE, NONE, LH02, LA01, LB01, SX01, SY01, MM01, CC01,
            SN15, JN05, EN00, HN18, HN16, DD01,
        },
    },
    CamType {
        name: "ThrowInto",
        styles: slots! {
            FN29, FN29, LL01, TT01, SS01, LW01, LE01, LH01, LA01, LB01, SX01, SY01, MM09, CC01,
            SN15, JN05, EN00, HN18, HN16, DD01,
        },
    },
    CamType {
        name: "Boss03",
        styles: slots! {
            FN30, FN30, LL12, TT02, SS01, LW01, LE01, LH02, LA01, LB01, SX02, SY01, MM01, CC01,
            SN15, JN05, EN00, HN18, HN16, DD01,
        },
    },
    CamType {
        name: "P_Ganon1",
        styles: slots! {
            FN31, FN31, LL13, TT02, SS01, LW01, LE01, LH02, LA01, LB01, SX02, SY01, MM01, CC01,
            SN15, JN05, EN00, HN18, HN16, DD01,
        },
    },
    CamType {
        name: "P_Ganon2",
        styles: slots! {
            FN32, FN32, LL14, TT02, SS01, LW01, LE01, LH02, LA01, LB01, SX02, SY01, MM01, CC01,
            SN15, JN05, EN00, HN18, HN16, DD01,
        },
    },
    CamType {
        name: "WindBoss",
        styles: slots! {
            FN33, FN33, LL15, TT02, SS01, LW01, LE01, LH02, LA02, LB02, SX01, SY01, MM03, CC01,
            SN15, JN05, EN00, HN18, HN16, DD01,
        },
    },
    CamType {
        name: "P_Ganon3",
        styles: slots! {
            FN34, FN34, LL14, TT02, SS01, LW01, LE01, LH02, LA01, LB01, SX02, SY01, MM01, CC01,
            SN15, JN05, EN00, HN18, HN16, DD01,
        },
    },
    CamType {
        name: "Test01",
        styles: slots! {
            FN27, FN27, LL10, TT02, SS01, LW01, LE01, LH02, LA01, LB01, SX02, SY01, MM01, CC01,
            SN15, JN05, EN00, HN18, HN16, DD01,
        },
    },
    CamType {
        name: "Amoss",
        styles: slots! {
            FN35, FN35, LL16, TT02, SS01, LW01, LE01, LH02, LA01, LB01, SX02, SY01, MM01, CC01,
            SN15, JN05, EN00, HN18, HN16, DD01,
        },
    },
    CamType {
        name: "Dungeon",
        styles: slots! {
            FN02, FN02, LL02, TT02, SS01, LW01, LE01, LH02, LA02, LB02, SX01, SY01, MM03, CC01,
            SN15, JN05, EN00, HN18, HN16, DD02,
        },
    },
    CamType {
        name: "Jump",
        styles: slots! {
            FN08, FN08, LL02, TT02, SS01, LW01, LE01, LH02, LA02, LB02, SX01, SY01, MM03, CC01,
            SN15, JN05, EN00, HN18, HN16, DD02,
        },
    },
    CamType {
        name: "DungeonCorner",
        styles: slots! {
            FN06, FN06, LL08, TT02, SS01, LW01, LE01, LH02, LA02, LB02, SX01, SY01, MM03, CC01,
            SN15, JN05, EN00, HN18, HN16, DD02,
        },
    },
    CamType {
        name: "DungeonWide",
        styles: slots! {
            FN07, FN07, LL07, TT02, SS01, LW01, LE01, LH01, LA02, LB02, SX01, SY01, MM08, CC01,
            SN15, JN05, EN00, HN18, HN16, DD02,
        },
    },
    CamType {
        name: "CannonGame",
        styles: slots! {
            ZZ00, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE,
            NONE, NONE, EN00, NONE, NONE, NONE,
        },
    },
    CamType {
        name: "FieldFOX",
        styles: slots! {
            GN01, GN01, LL01, TT01, SS01, LW01, LE01, LH01, LA01, LB01, SX01, SY01, MM01, CC01,
            SN15, JN05, EN00, HN18, HN16, DD01,
        },
    },
    CamType {
        name: "G_Roof",
        styles: slots! {
            FN82, FN82, LL82, TT02, SS01, LW01, LE01, LH02, LA01, LB01, SX02, SY01, MM82, CC01,
            SN15, JN05, EN00, HN18, HN16, DD01,
        },
    },
    CamType {
        name: "G_BedRoom",
        styles: slots! {
            FN81, FN81, LL81, TT02, SS01, LW01, LE01, LH02, LA01, LB01, SX02, SY01, MM81, CC01,
            SN15, JN05, EN00, HN18, HN16, DD01,
        },
    },
    CamType {
        name: "G_BedRoom2",
        styles: slots! {
            FN81, FN81, LL81, TT02, SS01, LW01, LE01, LH02, LA01, LB01, SX02, SY01, MM83, CC01,
            SN15, JN05, EN00, HN18, HN16, DD01,
        },
    },
    CamType {
        name: "TowerUp",
        styles: slots! {
            IN02, FN24, LL01, TT01, SS01, LW01, LE01, LH01, LA01, LB01, SX01, SY01, MM01, CC01,
            SN15, JN05, EN00, HN18, HN16, DD01,
        },
    },
    CamType {
        name: "MajTower",
        styles: slots! {
            IN03, FN02, IN03, TT02, SS01, LW01, LE01, LH02, LA02, LB02, SX01, SY01, MM03, CC01,
            SN15, JN05, EN00, HN18, HN16, DD01,
        },
    },
    CamType {
        name: "Tower",
        styles: slots! {
            IN01, FN24, IN02, TT01, SS01, LW01, LE01, LH01, LA01, LB01, SX01, SY01, MM01, CC01,
            SN15, JN05, EN00, HN18, HN16, DD01,
        },
    },
    CamType {
        name: "Boat",
        styles: slots! {
            BN07, BP07, LL11, TT01, SS01, BW07, BE07, NONE, NONE, NONE, SX01, SY01, MM07, NONE,
            SN15, JN05, EN00, HN18, HN16, DD01,
        },
    },
    CamType {
        name: "BoatBattle",
        styles: slots! {
            BN08, BP08, LL17, TT01, SS01, BW07, BE07, NONE, NONE, NONE, SX01, SY01, MM07, NONE,
            SN15, JN05, EN00, HN18, HN16, DD01,
        },
    },
    CamType {
        name: "WindHall",
        styles: slots! {
            FN38, FP38, LL07, TT02, SS01, LW01, LE01, LH01, LA02, LB02, SX01, SY01, MM08, CC01,
            SN15, JN05, EN00, HN18, HN16, DD02,
        },
    },
    CamType {
        name: "Tornado",
        styles: slots! {
            QN07, BP07, LL11, TT01, SS01, BW07, BE08, NONE, NONE, NONE, SX01, SY01, NONE, NONE,
            SN15, JN05, EN00, HN18, HN16, DD01,
        },
    },
    CamType {
        name: "OverLook",
        styles: slots! {
            FN11, FN11, LL01, TT04, SS01, LW02, LE01, LH01, LA01, LB01, SX01, SY01, MM02, CC01,
            SN15, JN05, EN00, HN18, HN16, DD02,
        },
    },
    CamType {
        name: "Dome",
        styles: slots! {
            FN39, FN05, LL02, TT02, SS01, LW01, LE01, LH02, LA02, LB02, SX01, SY01, MM02, CC01,
            SN15, JN05, EN00, HN18, HN16, DD02,
        },
    },
    CamType {
        name: "ForceFixdPos",
        styles: slots! {
            PN05, NONE, NONE, NONE, SS01, NONE, NONE, NONE, NONE, NONE, SX01, SY01, NONE, CC01,
            SN15, NONE, EN00, NONE, NONE, NONE,
        },
    },
    CamType {
        name: "FixdFrm",
        styles: slots! {
            XN06, NONE, NONE, NONE, SS01, NONE, NONE, NONE, NONE, NONE, SX01, SY01, NONE, CC01,
            SN15, NONE, EN00, NONE, NONE, NONE,
        },
    },
    CamType {
        name: "DStairs",
        styles: slots! {
            FN02, FN02, LL02, TT02, SS01, LW01, LE01, LH02, LA02, LB02, SX01, SY01, MM16, CC01,
            SN15, JN05, EN00, HN18, HN16, DD02,
        },
    },
    CamType {
        name: "Boss02",
        styles: slots! {
            FN40, FN40, LL05, TT02, SS01, LW01, LE01, LH02, LA01, LB01, SX02, SY01, MM01, CC01,
            SN15, JN05, EN00, HN18, HN16, DD01,
        },
    },
    CamType {
        name: "Seagal",
        styles: slots! {
            FN36, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, MM01, NONE,
            NONE, NONE, EN00, NONE, NONE, NONE,
        },
    },
    CamType {
        name: "SmallRoom",
        styles: slots! {
            FN41, FN41, LL04, TT02, SS01, LW01, LE01, LH02, LA01, LB01, SX01, SY01, MM17, CC01,
            SN15, JN05, EN00, HN18, HN16, DD01,
        },
    },
    CamType {
        name: "DanMae",
        styles: slots! {
            FN42, FN42, LL02, TT02, SS01, LW01, LE01, LH02, LA02, LB02, SX01, SY01, MM19, CC01,
            SN15, JN05, EN00, HN18, HN16, DD02,
        },
    },
    CamType {
        name: "Cafe",
        styles: slots! {
            FN28, FN28, LL04, TT02, SS01, LW01, LE01, LH02, LA01, LB01, SX01, SY01, MM20, CC01,
            SN15, JN05, EN00, HN18, HN16, DD01,
        },
    },
    CamType {
        name: "DungeonPassage",
        styles: slots! {
            FN17, FN17, LL08, TT02, SS01, LW01, LE01, LH02, LA02, LB02, SX01, SY01, MM20, CC01,
            SN15, JN05, EN00, HN18, HN16, DD02,
        },
    },
    CamType {
        name: "Corridor",
        styles: slots! {
            FN12, FN12, LL07, TT02, SS01, LW01, LE01, LH01, LA02, LB02, SX01, SY01, MM20, CC01,
            SN15, JN05, EN00, HN18, HN16, DD02,
        },
    },
    CamType {
        name: "Subject",
        styles: slots! {
            LN17, NONE, NONE, NONE, SS01, NONE, NONE, NONE, NONE, NONE, SX01, SY01, NONE, CC02,
            SN15, NONE, EN00, NONE, NONE, NONE,
        },
    },
    CamType {
        name: "BigBird",
        styles: slots! {
            FN43, FN43, LL18, TT02, SS01, LW01, LE01, LH02, LA01, LB01, SX02, SY01, MM01, CC01,
            SN15, JN05, EN00, HN18, HN16, DD01,
        },
    },
    CamType {
        name: "Room",
        styles: slots! {
            FN09, FN09, LL04, TT02, SS01, LW01, LE01, LH02, LA01, LB01, SX01, SY01, MM18, CC01,
            SN15, JN05, EN00, HN18, HN16, DD01,
        },
    },
    CamType {
        name: "DungeonDown",
        styles: slots! {
            FN05, FN05, LL02, TT02, SS01, LW01, LE01, LH02, LA02, LB02, SX01, SY01, MM21, CC01,
            SN15, JN05, EN00, HN18, HN16, DD02,
        },
    },
    CamType {
        name: "Boss04",
        styles: slots! {
            FN37, FN05, LL02, TT02, SS01, LW01, LE01, LH02, LA02, LB02, SX01, SY01, MM02, CC01,
            SN15, JN05, EN00, HN18, HN16, DD04,
        },
    },
    CamType {
        name: "UnderTheEaves",
        styles: slots! {
            FN01, FN01, LL01, TT01, SS01, LW01, LE01, LH01, LA01, LB01, SX01, SY01, MM22, CC01,
            SN15, JN05, EN00, HN18, HN16, DD01,
        },
    },
    CamType {
        name: "Restrict",
        styles: slots! {
            LN01, NONE, NONE, NONE, SS01, NONE, NONE, NONE, NONE, NONE, SX01, SY01, NONE, CC01,
            SN15, NONE, EN00, NONE, NONE, NONE,
        },
    },
    CamType {
        name: "DungeonUp",
        styles: slots! {
            FN04, FN04, LL06, TT02, SS01, LW02, LE01, LH02, LA02, LB02, SX01, SY01, MM08, CC01,
            SN15, JN05, EN00, HN18, HN16, DD02,
        },
    },
    CamType {
        name: "Block",
        styles: slots! {
            FN01, FN01, LL01, TT01, SS01, LW01, LE01, LH01, LA01, LB01, SX01, SY01, MM23, CC01,
            SN15, JN05, EN00, HN18, HN16, DD01,
        },
    },
    CamType {
        name: "Water",
        styles: slots! {
            FN14, FN14, LL02, TT02, SS01, NONE, NONE, NONE, NONE, NONE, SX01, SY01, MM14, CC01,
            SN15, JN05, EN00, NONE, NONE, NONE,
        },
    },
];

/// `dCamera_c::mvBGTypes`; motion-background ids name a camera type.
pub const MV_BG_TYPES: [&str; 35] = [
    "????", "Field", "Dungeon", "Plain", "DungeonDown", "DungeonUp", "DungeonCorner", "Jump",
    "DungeonWide", "Room", "FieldCushion", "OverLook", "Corridor", "Subject", "DungeonPassage",
    "Cliff", "Cliff2", "MajTower", "Boss01", "Boss02", "Gamoss", "MiniIsland", "Amoss", "Cafe",
    "P_Ganon1", "P_Ganon2", "WindBoss", "P_Ganon3", "G_BedRoom", "G_Roof", "G_BedRoom2", "Boss04",
    "WindHall", "BigBird", "DStairs",
];

/// `dCamera_c::GetCameraTypeFromCameraName`: first table row whose name matches.
pub fn type_index(name: &str) -> Option<usize> {
    TYPES.iter().position(|t| t.name == name)
}

/// `dCamera_c::type(name)`.
pub fn cam_type(name: &str) -> Option<&'static CamType> {
    type_index(name).map(|i| &TYPES[i])
}

/// The camera type a motion-background id names, `None` for `????` and unknown ids.
pub fn mv_bg_type_index(id: usize) -> Option<usize> {
    MV_BG_TYPES.get(id).copied().and_then(type_index)
}

/// `dCamera_c::mCamTypeField`.
pub const TYPE_FIELD: usize = 7;
/// `dCamera_c::mCamTypeEvent`.
pub const TYPE_EVENT: usize = 3;
/// `dCamera_c::mCamTypeWater`.
pub const TYPE_WATER: usize = 62;
/// `dCamera_c::mCamTypeSubject`.
pub const TYPE_SUBJECT: usize = 53;
/// `dCamera_c::mCamTypeBoat`.
pub const TYPE_BOAT: usize = 37;
/// `dCamera_c::mCamTypeBoatBattle`.
pub const TYPE_BOAT_BATTLE: usize = 38;
/// `dCamera_c::mCamTypeRestrict`.
pub const TYPE_RESTRICT: usize = 59;
/// `dCamera_c::mCamTypeKeep`.
pub const TYPE_KEEP: usize = 2;
