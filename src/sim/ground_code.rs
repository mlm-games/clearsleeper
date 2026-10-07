use super::collision::NO_EXIT;

/// The per-polygon code words a dzb carries, and what each value is for.
///
/// `cBgD_Ti_t` (`c_bg_w.h:44`) is four `u32`s per polygon-info entry, and the triangle points at
/// its entry through `cBgW::GetPolyInfId` (`c_bg_w.h:206`). `dBgS::GetExitId` (`bg_s.cpp:106`)
/// reads `m_info0` bits 13 to 18; all four codes below read `m_info1` through
/// `cBgW::GetPolyInf1` (`c_bg_w.h:214`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PolyCodes {
    /// `dBgS::GetExitId`, `bg_s.cpp:106`: `(m_info0 >> 13) & 0x3F`, the room-change exit the
    /// fall checker opens at `main.cpp:10756`. `0x3F` means the polygon has none.
    pub exit: u8,
    /// `dBgS::GetWallCode`, `bg_s.cpp:171`: `(m_info1 >> 8) & 0xF`.
    pub wall: WallCode,
    /// `dBgS::GetSpecialCode`, `bg_s.cpp:176`: `(m_info1 >> 12) & 0xF`.
    pub special: SpecialCode,
    /// `dBgS::GetAttributeCode`, `bg_s.cpp:227`: `(m_info1 >> 16) & 0x1F` run through
    /// [`attribute_from_direct`].
    pub attribute: AttrCode,
    /// `dBgS::GetAttributeCodeDirect`, `bg_s.cpp:187`: the raw `(m_info1 >> 16) & 0x1F` before
    /// the table. The table is many-to-one — raw `7`, `12`, `13`, `14`, `16` and `26` all become
    /// [`AttrCode::Dirt`] or [`AttrCode::Wood`] — so the raw bits cannot be recovered from
    /// [`PolyCodes::attribute`] and [`dBgS_ChangeAttributeCode`] at `bg_s.cpp:183` writes them
    /// straight back.
    pub attribute_direct: u8,
    /// `dBgS::GetGroundCode`, `bg_s.cpp:235`: `(m_info1 >> 21) & 0x1F`.
    pub ground: GroundCode,
}

impl Default for PolyCodes {
    fn default() -> Self {
        Self {
            exit: NO_EXIT,
            wall: WallCode::Other(0),
            special: SpecialCode::Other(0),
            attribute: AttrCode::Normal,
            attribute_direct: 0,
            ground: GroundCode::Other(0),
        }
    }
}

/// `dBgS::GetWallCode`, `bg_s.cpp:171`: what a vertical face is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WallCode {
    /// `climb.inc:117` keeps a climb going only on this code and `main.cpp:5346` starts a climb
    /// down on it; `main.cpp:4648` turns it into front-wall type 3.
    Climb,
    /// `main.cpp:4610` gives up before taking any ledge point and `main.cpp:5341` excludes it
    /// from the wall jump, so there is nothing to catch on.
    NoCatch,
    /// `main.cpp:4614` reaches it with a jump rather than a hang.
    AutoJump,
    /// `ladder.inc:62` and `ladder.inc:83` keep the ladder states on this code alone.
    Ladder,
    /// `main.cpp:4651` takes it exactly as `Ladder` does but only off the ground, and hands the
    /// climb states front-wall type 5.
    LadderGrounded,
    /// A value the reference never compares against.
    Other(u8),
}

impl WallCode {
    /// The four bits as stored.
    pub fn from_raw(raw: u8) -> Self {
        match raw & 0xF {
            1 => Self::Climb,
            2 => Self::NoCatch,
            3 => Self::AutoJump,
            4 => Self::Ladder,
            5 => Self::LadderGrounded,
            other => Self::Other(other),
        }
    }

    /// The four bits as stored, which is what a comparison in the reference is against.
    pub fn raw(self) -> u8 {
        match self {
            Self::Climb => 1,
            Self::NoCatch => 2,
            Self::AutoJump => 3,
            Self::Ladder => 4,
            Self::LadderGrounded => 5,
            Self::Other(raw) => raw & 0xF,
        }
    }

    /// The value's name, for a reader with the stage open beside this.
    pub fn name(self) -> &'static str {
        match self {
            Self::Climb => "climb",
            Self::NoCatch => "no catch",
            Self::AutoJump => "auto jump",
            Self::Ladder => "ladder",
            Self::LadderGrounded => "ladder, grounded only",
            Self::Other(_) => "unassigned wall code",
        }
    }
}

/// `dBgS::GetSpecialCode`, `bg_s.cpp:176`: what a polygon does besides hold the player up.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialCode {
    /// `main.cpp:4002` is the only code `getSlidePolygon` takes, so this is the sliding surface;
    /// `crawl.inc:177` refuses to crawl onto it at any angle.
    Slide,
    /// `crawl.inc:177` refuses to crawl onto it only when the plane normal is under `0.643832f`,
    /// so a shallow one can still be crawled on.
    Steep,
    /// `whide.inc:90` finds no polygon for it, so the wall cannot be squeezed past.
    SolidWall,
    /// `kb.cpp:2170` records an event bit when the ground under a walker carries it.
    PathMarker,
    /// `kb.cpp:1919` runs a walker along a ground that carries it.
    RunSurface,
    /// A value the reference never compares against.
    Other(u8),
}

impl SpecialCode {
    /// The four bits as stored.
    pub fn from_raw(raw: u8) -> Self {
        match raw & 0xF {
            1 => Self::Slide,
            2 => Self::Steep,
            3 => Self::SolidWall,
            4 => Self::PathMarker,
            5 => Self::RunSurface,
            other => Self::Other(other),
        }
    }

    /// The four bits as stored.
    pub fn raw(self) -> u8 {
        match self {
            Self::Slide => 1,
            Self::Steep => 2,
            Self::SolidWall => 3,
            Self::PathMarker => 4,
            Self::RunSurface => 5,
            Self::Other(raw) => raw & 0xF,
        }
    }

    /// The value's name, for a reader with the stage open beside this.
    pub fn name(self) -> &'static str {
        match self {
            Self::Slide => "slide",
            Self::Steep => "steep",
            Self::SolidWall => "solid wall",
            Self::PathMarker => "path marker",
            Self::RunSurface => "run surface",
            Self::Other(_) => "unassigned special code",
        }
    }
}

/// `dBgS::GetGroundCode`, `bg_s.cpp:235`: what standing on a polygon means.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroundCode {
    /// `main.cpp:10741` handles it with an absent ground and restarts the room on contact, and
    /// `main.cpp:3632` counts it with an absent ground when it decides a fall voice.
    Abyss,
    /// `main.cpp:10764` lets a ground hit on it open the exit anyway, `main.cpp:10792` clears the
    /// hit, and `main.cpp:10811` records the drop as a pitfall rather than a walk.
    Pitfall,
    /// `main.cpp:2409` takes no ground angle on it and `main.cpp:3016` refuses the slope walk, so
    /// the player travels along it as if it were flat.
    TooSteep,
    /// A value the reference never compares against. `main.cpp:5331` tests 3 against the previous
    /// frame's code to hold an auto jump back, but never says what 3 is.
    Other(u8),
}

impl GroundCode {
    /// The five bits as stored.
    pub fn from_raw(raw: u8) -> Self {
        match raw & 0x1F {
            4 => Self::Abyss,
            5 => Self::Pitfall,
            8 => Self::TooSteep,
            other => Self::Other(other),
        }
    }

    /// The five bits as stored.
    pub fn raw(self) -> u8 {
        match self {
            Self::Abyss => 4,
            Self::Pitfall => 5,
            Self::TooSteep => 8,
            Self::Other(raw) => raw & 0x1F,
        }
    }

    /// The value's name, for a reader with the stage open beside this.
    pub fn name(self) -> &'static str {
        match self {
            Self::Abyss => "abyss",
            Self::Pitfall => "pitfall",
            Self::TooSteep => "too steep",
            Self::Other(_) => "unassigned ground code",
        }
    }
}

/// `dBgS_AttributeCode`, `bg_s.h:28`: the surface material. `dBgS::GetAttributeCode`
/// (`bg_s.cpp:227`) reaches these values by running the raw bits through
/// [`ATTRIBUTE_FROM_DIRECT`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttrCode {
    /// `0x00`.
    Normal,
    /// `0x01`.
    Dirt,
    /// `0x02`.
    Wood,
    /// `0x03`.
    Stone,
    /// `0x04`.
    Grass,
    /// `0x05`, which `main.cpp:4927` and `main.cpp:4957` exempt from fall damage.
    GiantFlower,
    /// `0x06`, which `main.cpp:10721` and `main.cpp:11439` send to the lava face test.
    Lava,
    /// `0x08`, which `main.cpp:10730` and `main.cpp:11439` send to the same face test.
    Void,
    /// `0x09`, which `main.cpp:5053` and `main.cpp:5119` hurt on.
    Damage,
    /// `0x0A`.
    Carpet,
    /// `0x0B`, which `hookshot.cpp:269` lets a hook bite.
    Sand,
    /// `0x0F`, which `arrow.cpp:819` sticks to and `main.cpp:2449` skips sliding on.
    Ice,
    /// `0x13`, which `bgn2.cpp:126` takes as the water surface.
    Water,
    /// `0x14`.
    Metal,
    /// `0x15`, which `arrow.cpp:821` sticks to and `main.cpp:5102` slips on.
    Freeze,
    /// `0x16`, which `main.cpp:5053` hurts on.
    Electricity,
    /// `0x17`.
    Waterfall,
    /// `0x1B`, the value `hang.inc:118` falls back to when the ground under a hanging player has
    /// no cross at all.
    Unknown,
}

impl AttrCode {
    /// The enum's own value at `bg_s.h:29` and following.
    pub fn raw(self) -> u8 {
        match self {
            Self::Normal => 0x00,
            Self::Dirt => 0x01,
            Self::Wood => 0x02,
            Self::Stone => 0x03,
            Self::Grass => 0x04,
            Self::GiantFlower => 0x05,
            Self::Lava => 0x06,
            Self::Void => 0x08,
            Self::Damage => 0x09,
            Self::Carpet => 0x0A,
            Self::Sand => 0x0B,
            Self::Ice => 0x0F,
            Self::Water => 0x13,
            Self::Metal => 0x14,
            Self::Freeze => 0x15,
            Self::Electricity => 0x16,
            Self::Waterfall => 0x17,
            Self::Unknown => 0x1B,
        }
    }

    /// The material's name, for a reader with the stage open beside this.
    pub fn name(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Dirt => "dirt",
            Self::Wood => "wood",
            Self::Stone => "stone",
            Self::Grass => "grass",
            Self::GiantFlower => "giant flower",
            Self::Lava => "lava",
            Self::Void => "void",
            Self::Damage => "damage",
            Self::Carpet => "carpet",
            Self::Sand => "sand",
            Self::Ice => "ice",
            Self::Water => "water",
            Self::Metal => "metal",
            Self::Freeze => "freeze",
            Self::Electricity => "electricity",
            Self::Waterfall => "waterfall",
            Self::Unknown => "unknown",
        }
    }
}

/// `dBgS::GetAttributeCode`, `bg_s.cpp:191-232`: the raw five bits of `m_info1` run through this
/// table. `bg_s.cpp:229` returns `0` outside the table, and the mask at `bg_s.cpp:188` keeps the
/// raw value inside it, so the lookup is total.
pub const ATTRIBUTE_FROM_DIRECT: [AttrCode; 32] = [
    AttrCode::Normal,
    AttrCode::Dirt,
    AttrCode::Wood,
    AttrCode::Stone,
    AttrCode::Grass,
    AttrCode::GiantFlower,
    AttrCode::Lava,
    AttrCode::Dirt,
    AttrCode::Void,
    AttrCode::Damage,
    AttrCode::Carpet,
    AttrCode::Sand,
    AttrCode::Wood,
    AttrCode::Wood,
    AttrCode::Wood,
    AttrCode::Ice,
    AttrCode::Wood,
    AttrCode::Metal,
    AttrCode::Dirt,
    AttrCode::Water,
    AttrCode::Metal,
    AttrCode::Freeze,
    AttrCode::Electricity,
    AttrCode::Waterfall,
    AttrCode::Metal,
    AttrCode::Carpet,
    AttrCode::Wood,
    AttrCode::Normal,
    AttrCode::Normal,
    AttrCode::Normal,
    AttrCode::Normal,
    AttrCode::Normal,
];

/// `dBgS::GetAttributeCode`, `bg_s.cpp:227`, for a raw code straight out of `m_info1`.
pub fn attribute_from_direct(direct: u8) -> AttrCode {
    ATTRIBUTE_FROM_DIRECT[usize::from(direct & 0x1F)]
}

/// `mFrontWallType`, the word `setFrontWallType` at `main.cpp:4544` writes and the hang, ladder,
/// climb and jump selectors read back at `main.cpp:4786-4879`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrontWall {
    /// `main.cpp:4546`: the word was still `0` on entry, so nothing was traced.
    Untested,
    /// `main.cpp:4549`: a wall was traced and no ledge point survived it.
    Wall,
    /// `main.cpp:4720`: a wall `getWHideModePolygon` found where the player had none, which
    /// `main.cpp:4710` keeps off the player unless neither the grab nor the swim nor the airborne
    /// mode is on.
    Hidden,
    /// `main.cpp:4648`: a vertical face, which the climb states take at `main.cpp:4835`.
    Climb,
    /// `main.cpp:4653`: the top edge of a ladder, which the ladder-up state takes at
    /// `main.cpp:4858`.
    LadderUp,
    /// `main.cpp:4655`: the same edge reachable only off the ground, which the ladder-down state
    /// takes at `main.cpp:4864`.
    LadderDown,
    /// `main.cpp:4761`: a ledge between `27.09f` and the lowest wall-catch band, which the small
    /// jump takes at `main.cpp:4869`.
    LowLedge,
    /// `main.cpp:4763`, and `main.cpp:4749` and `main.cpp:4753` in the swim and airborne forms:
    /// a ledge at hand height, which the wall catch takes at `main.cpp:4875`.
    HandLedge,
    /// `main.cpp:4765`: the band above hand height, which the vertical jump takes with
    /// [`FrontWall::HighLedge`] at `main.cpp:4877`.
    MidLedge,
    /// `main.cpp:4767`: the top band.
    HighLedge,
}

impl FrontWall {
    /// The value `mFrontWallType` holds.
    pub fn from_raw(raw: u8) -> Self {
        match raw {
            0 => Self::Untested,
            1 => Self::Wall,
            2 => Self::Hidden,
            3 => Self::Climb,
            4 => Self::LadderUp,
            5 => Self::LadderDown,
            6 => Self::LowLedge,
            7 => Self::HandLedge,
            8 => Self::MidLedge,
            _ => Self::HighLedge,
        }
    }

    /// The value `mFrontWallType` holds.
    pub fn raw(self) -> u8 {
        match self {
            Self::Untested => 0,
            Self::Wall => 1,
            Self::Hidden => 2,
            Self::Climb => 3,
            Self::LadderUp => 4,
            Self::LadderDown => 5,
            Self::LowLedge => 6,
            Self::HandLedge => 7,
            Self::MidLedge => 8,
            Self::HighLedge => 9,
        }
    }

    /// The value's name, for a reader with the reference open beside this.
    pub fn name(self) -> &'static str {
        match self {
            Self::Untested => "not tested",
            Self::Wall => "wall, no ledge",
            Self::Hidden => "hidden wall",
            Self::Climb => "climb",
            Self::LadderUp => "ladder up",
            Self::LadderDown => "ladder down",
            Self::LowLedge => "low ledge",
            Self::HandLedge => "hand ledge",
            Self::MidLedge => "mid ledge",
            Self::HighLedge => "high ledge",
        }
    }
}