use glam::{Vec2, Vec3};

use super::cam_style::{self, CamEngine, param};
use super::camera::Camera;
use super::player::Player;
use super::player_status::{self, PlayerStatus};

/// `cAngle::DEG2S_CONSTANT`, `c_angle.h:8`.
const DEG2S: f32 = 0x8000 as f32 / 180.0;
/// `cAngle::S2DEG_CONSTANT`, `c_angle.h:9`.
const S2DEG: f32 = 180.0 / 0x8000 as f32;
/// `cAngle::S2RAD_CONSTANT`, `c_angle.h:10`.
const S2RAD: f32 = core::f32::consts::PI / 0x8000 as f32;
/// `cAngle::RAD2S_CONSTANT`, `c_angle.h:11`.
const RAD2S: f32 = 0x8000 as f32 / core::f32::consts::PI;
/// `cM3d_IsZero` against `G_CM3D_F_ABS_MIN` (`c_m3d.h:135`, `c_m3d.cpp:22`).
const ABS_MIN: f32 = 3.8146973e-06;

/// `cSAngle` from `c_angle.h:20`: a signed angle on the 0x8000 circle, so a whole turn is
/// +-32768 units and `_180` is the minimum. Every operation wraps through `i16` and every
/// comparison is a plain signed compare, which is what lets `cSAngle::_270` and `_90` serve as
/// the "more than a quarter turn away" test the follow engine reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct SAngle(i16);

impl SAngle {
    /// `cSAngle::_0`, `c_angle.cpp:10`.
    pub const ZERO: Self = Self(0);
    /// `cSAngle::_90`, `c_angle.cpp:12`.
    pub const QUARTER: Self = Self(0x4000);
    /// `cSAngle::_180`, `c_angle.cpp:13`, the wrap point.
    pub const HALF: Self = Self(i16::MIN);
    /// `cSAngle::_270`, `c_angle.cpp:14`.
    pub const THREE_QUARTER: Self = Self(-0x4000);
    /// `cSAngle::getMaxNegative`, `c_angle.h:70`.
    pub const MAX_NEGATIVE: Self = Self(i16::MIN);

    /// `cSAngle::Val(f32)` / `cAngle::d2s`, `c_angle.cpp:44-46`.
    pub fn from_degrees(degrees: f32) -> Self {
        Self((degrees * DEG2S) as i16)
    }

    /// `cSAngle::Radian`, `c_angle.cpp:54-56`.
    pub fn radian(self) -> f32 {
        self.0 as f32 * S2RAD
    }

    /// `cAngle::r2s`, `c_angle.h:86`.
    pub fn from_radians(radians: f32) -> Self {
        Self((radians * RAD2S) as i16)
    }

    /// `cSAngle::Val`, `c_angle.h:41`.
    pub const fn val(self) -> i16 {
        self.0
    }

    /// `cSAngle::Degree`, `c_angle.cpp:49-51`.
    pub fn degree(self) -> f32 {
        self.0 as f32 * S2DEG
    }

    /// `cSAngle::Inv`, `c_angle.cpp:69-71`: half a turn away, wrapping through `i16`.
    pub fn inv(self) -> Self {
        Self(self.0.wrapping_sub(i16::MIN))
    }

    /// `cSAngle::Sin`, `c_angle.cpp:74-76`.
    pub fn sin(self) -> f32 {
        self.radian().sin()
    }

    /// `cSAngle::Cos`, `c_angle.cpp:79-81`.
    pub fn cos(self) -> f32 {
        self.radian().cos()
    }
}

impl From<i16> for SAngle {
    fn from(value: i16) -> Self {
        Self(value)
    }
}

impl core::ops::Neg for SAngle {
    type Output = Self;
    /// `cSAngle::operator-`, `c_angle.cpp:89-91`.
    fn neg(self) -> Self {
        Self(self.0.wrapping_neg())
    }
}

impl core::ops::Add for SAngle {
    type Output = Self;
    /// `cSAngle::operator+`, `c_angle.cpp:94-96`.
    fn add(self, other: Self) -> Self {
        Self(self.0.wrapping_add(other.0))
    }
}

impl core::ops::Sub for SAngle {
    type Output = Self;
    /// `cSAngle::operator-`, `c_angle.cpp:99-101`.
    fn sub(self, other: Self) -> Self {
        Self(self.0.wrapping_sub(other.0))
    }
}

impl core::ops::AddAssign for SAngle {
    /// `cSAngle::operator+=`, `c_angle.cpp:104-106`.
    fn add_assign(&mut self, other: Self) {
        self.0 = self.0.wrapping_add(other.0);
    }
}

impl core::ops::SubAssign for SAngle {
    /// `cSAngle::operator-=`, `c_angle.cpp:109-111`.
    fn sub_assign(&mut self, other: Self) {
        self.0 = self.0.wrapping_sub(other.0);
    }
}

impl core::ops::Mul<f32> for SAngle {
    type Output = Self;
    /// `cSAngle::operator*(f32)`, `c_angle.cpp:134-136`.
    fn mul(self, scale: f32) -> Self {
        Self((self.0 as f32 * scale) as i16)
    }
}

impl core::ops::MulAssign<f32> for SAngle {
    /// `cSAngle::operator*=(f32)`, `c_angle.cpp:139-141`.
    fn mul_assign(&mut self, scale: f32) {
        self.0 = (self.0 as f32 * scale) as i16;
    }
}

/// `cM3d_IsZero`, `c_m3d.h:135`.
fn is_zero(value: f32) -> bool {
    value.abs() < ABS_MIN
}

/// `U_GetAtanTable`, `c_math.cpp:112-115`: `atan(y / x)` for a ratio in `[0, 1]`, in 0x8000
/// units, read one entry per 1/1024 of the ratio.
static ATN_TABLE: [u16; 1025] = [
    0x0000, 0x000A, 0x0014, 0x001F, 0x0029, 0x0033, 0x003D, 0x0047, 0x0051, 0x005C, 0x0066, 0x0070,
    0x007A, 0x0084, 0x008F, 0x0099, 0x00A3, 0x00AD, 0x00B7, 0x00C2, 0x00CC, 0x00D6, 0x00E0, 0x00EA,
    0x00F4, 0x00FF, 0x0109, 0x0113, 0x011D, 0x0127, 0x0131, 0x013C, 0x0146, 0x0150, 0x015A, 0x0164,
    0x016F, 0x0179, 0x0183, 0x018D, 0x0197, 0x01A1, 0x01AC, 0x01B6, 0x01C0, 0x01CA, 0x01D4, 0x01DE,
    0x01E9, 0x01F3, 0x01FD, 0x0207, 0x0211, 0x021B, 0x0226, 0x0230, 0x023A, 0x0244, 0x024E, 0x0258,
    0x0262, 0x026D, 0x0277, 0x0281, 0x028B, 0x0295, 0x029F, 0x02A9, 0x02B4, 0x02BE, 0x02C8, 0x02D2,
    0x02DC, 0x02E6, 0x02F0, 0x02FB, 0x0305, 0x030F, 0x0319, 0x0323, 0x032D, 0x0337, 0x0341, 0x034C,
    0x0356, 0x0360, 0x036A, 0x0374, 0x037E, 0x0388, 0x0392, 0x039C, 0x03A7, 0x03B1, 0x03BB, 0x03C5,
    0x03CF, 0x03D9, 0x03E3, 0x03ED, 0x03F7, 0x0401, 0x040C, 0x0416, 0x0420, 0x042A, 0x0434, 0x043E,
    0x0448, 0x0452, 0x045C, 0x0466, 0x0470, 0x047A, 0x0484, 0x048E, 0x0499, 0x04A3, 0x04AD, 0x04B7,
    0x04C1, 0x04CB, 0x04D5, 0x04DF, 0x04E9, 0x04F3, 0x04FD, 0x0507, 0x0511, 0x051B, 0x0525, 0x052F,
    0x0539, 0x0543, 0x054D, 0x0557, 0x0561, 0x056B, 0x0575, 0x057F, 0x0589, 0x0593, 0x059D, 0x05A7,
    0x05B1, 0x05BB, 0x05C5, 0x05CF, 0x05D9, 0x05E3, 0x05ED, 0x05F7, 0x0601, 0x060B, 0x0615, 0x061F,
    0x0629, 0x0633, 0x063D, 0x0647, 0x0651, 0x065B, 0x0665, 0x066E, 0x0678, 0x0682, 0x068C, 0x0696,
    0x06A0, 0x06AA, 0x06B4, 0x06BE, 0x06C8, 0x06D2, 0x06DC, 0x06E5, 0x06EF, 0x06F9, 0x0703, 0x070D,
    0x0717, 0x0721, 0x072B, 0x0735, 0x073E, 0x0748, 0x0752, 0x075C, 0x0766, 0x0770, 0x077A, 0x0783,
    0x078D, 0x0797, 0x07A1, 0x07AB, 0x07B5, 0x07BE, 0x07C8, 0x07D2, 0x07DC, 0x07E6, 0x07EF, 0x07F9,
    0x0803, 0x080D, 0x0817, 0x0820, 0x082A, 0x0834, 0x083E, 0x0848, 0x0851, 0x085B, 0x0865, 0x086F,
    0x0878, 0x0882, 0x088C, 0x0896, 0x089F, 0x08A9, 0x08B3, 0x08BD, 0x08C6, 0x08D0, 0x08DA, 0x08E3,
    0x08ED, 0x08F7, 0x0901, 0x090A, 0x0914, 0x091E, 0x0927, 0x0931, 0x093B, 0x0944, 0x094E, 0x0958,
    0x0961, 0x096B, 0x0975, 0x097E, 0x0988, 0x0992, 0x099B, 0x09A5, 0x09AE, 0x09B8, 0x09C2, 0x09CB,
    0x09D5, 0x09DE, 0x09E8, 0x09F2, 0x09FB, 0x0A05, 0x0A0E, 0x0A18, 0x0A22, 0x0A2B, 0x0A35, 0x0A3E,
    0x0A48, 0x0A51, 0x0A5B, 0x0A64, 0x0A6E, 0x0A77, 0x0A81, 0x0A8B, 0x0A94, 0x0A9E, 0x0AA7, 0x0AB1,
    0x0ABA, 0x0AC4, 0x0ACD, 0x0AD7, 0x0AE0, 0x0AE9, 0x0AF3, 0x0AFC, 0x0B06, 0x0B0F, 0x0B19, 0x0B22,
    0x0B2C, 0x0B35, 0x0B3F, 0x0B48, 0x0B51, 0x0B5B, 0x0B64, 0x0B6E, 0x0B77, 0x0B80, 0x0B8A, 0x0B93,
    0x0B9D, 0x0BA6, 0x0BAF, 0x0BB9, 0x0BC2, 0x0BCB, 0x0BD5, 0x0BDE, 0x0BE7, 0x0BF1, 0x0BFA, 0x0C03,
    0x0C0D, 0x0C16, 0x0C1F, 0x0C29, 0x0C32, 0x0C3B, 0x0C45, 0x0C4E, 0x0C57, 0x0C60, 0x0C6A, 0x0C73,
    0x0C7C, 0x0C86, 0x0C8F, 0x0C98, 0x0CA1, 0x0CAB, 0x0CB4, 0x0CBD, 0x0CC6, 0x0CCF, 0x0CD9, 0x0CE2,
    0x0CEB, 0x0CF4, 0x0CFD, 0x0D07, 0x0D10, 0x0D19, 0x0D22, 0x0D2B, 0x0D34, 0x0D3E, 0x0D47, 0x0D50,
    0x0D59, 0x0D62, 0x0D6B, 0x0D74, 0x0D7D, 0x0D87, 0x0D90, 0x0D99, 0x0DA2, 0x0DAB, 0x0DB4, 0x0DBD,
    0x0DC6, 0x0DCF, 0x0DD8, 0x0DE1, 0x0DEA, 0x0DF3, 0x0DFC, 0x0E05, 0x0E0F, 0x0E18, 0x0E21, 0x0E2A,
    0x0E33, 0x0E3C, 0x0E45, 0x0E4E, 0x0E56, 0x0E5F, 0x0E68, 0x0E71, 0x0E7A, 0x0E83, 0x0E8C, 0x0E95,
    0x0E9E, 0x0EA7, 0x0EB0, 0x0EB9, 0x0EC2, 0x0ECB, 0x0ED4, 0x0EDC, 0x0EE5, 0x0EEE, 0x0EF7, 0x0F00,
    0x0F09, 0x0F12, 0x0F1B, 0x0F23, 0x0F2C, 0x0F35, 0x0F3E, 0x0F47, 0x0F50, 0x0F58, 0x0F61, 0x0F6A,
    0x0F73, 0x0F7C, 0x0F84, 0x0F8D, 0x0F96, 0x0F9F, 0x0FA7, 0x0FB0, 0x0FB9, 0x0FC2, 0x0FCA, 0x0FD3,
    0x0FDC, 0x0FE5, 0x0FED, 0x0FF6, 0x0FFF, 0x1007, 0x1010, 0x1019, 0x1021, 0x102A, 0x1033, 0x103B,
    0x1044, 0x104D, 0x1055, 0x105E, 0x1067, 0x106F, 0x1078, 0x1080, 0x1089, 0x1092, 0x109A, 0x10A3,
    0x10AB, 0x10B4, 0x10BC, 0x10C5, 0x10CE, 0x10D6, 0x10DF, 0x10E7, 0x10F0, 0x10F8, 0x1101, 0x1109,
    0x1112, 0x111A, 0x1123, 0x112B, 0x1134, 0x113C, 0x1145, 0x114D, 0x1156, 0x115E, 0x1166, 0x116F,
    0x1177, 0x1180, 0x1188, 0x1191, 0x1199, 0x11A1, 0x11AA, 0x11B2, 0x11BB, 0x11C3, 0x11CB, 0x11D4,
    0x11DC, 0x11E4, 0x11ED, 0x11F5, 0x11FD, 0x1206, 0x120E, 0x1216, 0x121F, 0x1227, 0x122F, 0x1237,
    0x1240, 0x1248, 0x1250, 0x1259, 0x1261, 0x1269, 0x1271, 0x127A, 0x1282, 0x128A, 0x1292, 0x129A,
    0x12A3, 0x12AB, 0x12B3, 0x12BB, 0x12C3, 0x12CC, 0x12D4, 0x12DC, 0x12E4, 0x12EC, 0x12F4, 0x12FC,
    0x1305, 0x130D, 0x1315, 0x131D, 0x1325, 0x132D, 0x1335, 0x133D, 0x1345, 0x134D, 0x1355, 0x135E,
    0x1366, 0x136E, 0x1376, 0x137E, 0x1386, 0x138E, 0x1396, 0x139E, 0x13A6, 0x13AE, 0x13B6, 0x13BE,
    0x13C6, 0x13CE, 0x13D6, 0x13DE, 0x13E6, 0x13ED, 0x13F5, 0x13FD, 0x1405, 0x140D, 0x1415, 0x141D,
    0x1425, 0x142D, 0x1435, 0x143D, 0x1444, 0x144C, 0x1454, 0x145C, 0x1464, 0x146C, 0x1473, 0x147B,
    0x1483, 0x148B, 0x1493, 0x149B, 0x14A2, 0x14AA, 0x14B2, 0x14BA, 0x14C1, 0x14C9, 0x14D1, 0x14D9,
    0x14E0, 0x14E8, 0x14F0, 0x14F8, 0x14FF, 0x1507, 0x150F, 0x1516, 0x151E, 0x1526, 0x152D, 0x1535,
    0x153D, 0x1544, 0x154C, 0x1554, 0x155B, 0x1563, 0x156B, 0x1572, 0x157A, 0x1581, 0x1589, 0x1591,
    0x1598, 0x15A0, 0x15A7, 0x15AF, 0x15B7, 0x15BE, 0x15C6, 0x15CD, 0x15D5, 0x15DC, 0x15E4, 0x15EB,
    0x15F3, 0x15FA, 0x1602, 0x1609, 0x1611, 0x1618, 0x1620, 0x1627, 0x162F, 0x1636, 0x163E, 0x1645,
    0x164C, 0x1654, 0x165B, 0x1663, 0x166A, 0x1671, 0x1679, 0x1680, 0x1688, 0x168F, 0x1696, 0x169E,
    0x16A5, 0x16AC, 0x16B4, 0x16BB, 0x16C2, 0x16CA, 0x16D1, 0x16D8, 0x16E0, 0x16E7, 0x16EE, 0x16F6,
    0x16FD, 0x1704, 0x170B, 0x1713, 0x171A, 0x1721, 0x1728, 0x1730, 0x1737, 0x173E, 0x1745, 0x174C,
    0x1754, 0x175B, 0x1762, 0x1769, 0x1770, 0x1778, 0x177F, 0x1786, 0x178D, 0x1794, 0x179B, 0x17A2,
    0x17AA, 0x17B1, 0x17B8, 0x17BF, 0x17C6, 0x17CD, 0x17D4, 0x17DB, 0x17E2, 0x17E9, 0x17F0, 0x17F7,
    0x17FE, 0x1806, 0x180D, 0x1814, 0x181B, 0x1822, 0x1829, 0x1830, 0x1837, 0x183E, 0x1845, 0x184C,
    0x1853, 0x185A, 0x1860, 0x1867, 0x186E, 0x1875, 0x187C, 0x1883, 0x188A, 0x1891, 0x1898, 0x189F,
    0x18A6, 0x18AD, 0x18B3, 0x18BA, 0x18C1, 0x18C8, 0x18CF, 0x18D6, 0x18DD, 0x18E3, 0x18EA, 0x18F1,
    0x18F8, 0x18FF, 0x1906, 0x190C, 0x1913, 0x191A, 0x1921, 0x1928, 0x192E, 0x1935, 0x193C, 0x1943,
    0x1949, 0x1950, 0x1957, 0x195D, 0x1964, 0x196B, 0x1972, 0x1978, 0x197F, 0x1986, 0x198C, 0x1993,
    0x199A, 0x19A0, 0x19A7, 0x19AE, 0x19B4, 0x19BB, 0x19C2, 0x19C8, 0x19CF, 0x19D5, 0x19DC, 0x19E3,
    0x19E9, 0x19F0, 0x19F6, 0x19FD, 0x1A04, 0x1A0A, 0x1A11, 0x1A17, 0x1A1E, 0x1A24, 0x1A2B, 0x1A31,
    0x1A38, 0x1A3E, 0x1A45, 0x1A4B, 0x1A52, 0x1A58, 0x1A5F, 0x1A65, 0x1A6C, 0x1A72, 0x1A79, 0x1A7F,
    0x1A86, 0x1A8C, 0x1A93, 0x1A99, 0x1A9F, 0x1AA6, 0x1AAC, 0x1AB3, 0x1AB9, 0x1AC0, 0x1AC6, 0x1ACC,
    0x1AD3, 0x1AD9, 0x1ADF, 0x1AE6, 0x1AEC, 0x1AF2, 0x1AF9, 0x1AFF, 0x1B05, 0x1B0C, 0x1B12, 0x1B18,
    0x1B1F, 0x1B25, 0x1B2B, 0x1B32, 0x1B38, 0x1B3E, 0x1B44, 0x1B4B, 0x1B51, 0x1B57, 0x1B5D, 0x1B64,
    0x1B6A, 0x1B70, 0x1B76, 0x1B7D, 0x1B83, 0x1B89, 0x1B8F, 0x1B95, 0x1B9C, 0x1BA2, 0x1BA8, 0x1BAE,
    0x1BB4, 0x1BBA, 0x1BC1, 0x1BC7, 0x1BCD, 0x1BD3, 0x1BD9, 0x1BDF, 0x1BE5, 0x1BEB, 0x1BF2, 0x1BF8,
    0x1BFE, 0x1C04, 0x1C0A, 0x1C10, 0x1C16, 0x1C1C, 0x1C22, 0x1C28, 0x1C2E, 0x1C34, 0x1C3A, 0x1C40,
    0x1C46, 0x1C4C, 0x1C52, 0x1C58, 0x1C5E, 0x1C64, 0x1C6A, 0x1C70, 0x1C76, 0x1C7C, 0x1C82, 0x1C88,
    0x1C8E, 0x1C94, 0x1C9A, 0x1CA0, 0x1CA6, 0x1CAC, 0x1CB2, 0x1CB8, 0x1CBE, 0x1CC3, 0x1CC9, 0x1CCF,
    0x1CD5, 0x1CDB, 0x1CE1, 0x1CE7, 0x1CED, 0x1CF3, 0x1CF8, 0x1CFE, 0x1D04, 0x1D0A, 0x1D10, 0x1D16,
    0x1D1B, 0x1D21, 0x1D27, 0x1D2D, 0x1D33, 0x1D38, 0x1D3E, 0x1D44, 0x1D4A, 0x1D4F, 0x1D55, 0x1D5B,
    0x1D61, 0x1D66, 0x1D6C, 0x1D72, 0x1D78, 0x1D7D, 0x1D83, 0x1D89, 0x1D8E, 0x1D94, 0x1D9A, 0x1DA0,
    0x1DA5, 0x1DAB, 0x1DB1, 0x1DB6, 0x1DBC, 0x1DC2, 0x1DC7, 0x1DCD, 0x1DD3, 0x1DD8, 0x1DDE, 0x1DE3,
    0x1DE9, 0x1DEF, 0x1DF4, 0x1DFA, 0x1DFF, 0x1E05, 0x1E0B, 0x1E10, 0x1E16, 0x1E1B, 0x1E21, 0x1E26,
    0x1E2C, 0x1E32, 0x1E37, 0x1E3D, 0x1E42, 0x1E48, 0x1E4D, 0x1E53, 0x1E58, 0x1E5E, 0x1E63, 0x1E69,
    0x1E6E, 0x1E74, 0x1E79, 0x1E7F, 0x1E84, 0x1E8A, 0x1E8F, 0x1E94, 0x1E9A, 0x1E9F, 0x1EA5, 0x1EAA,
    0x1EB0, 0x1EB5, 0x1EBA, 0x1EC0, 0x1EC5, 0x1ECB, 0x1ED0, 0x1ED5, 0x1EDB, 0x1EE0, 0x1EE6, 0x1EEB,
    0x1EF0, 0x1EF6, 0x1EFB, 0x1F00, 0x1F06, 0x1F0B, 0x1F10, 0x1F16, 0x1F1B, 0x1F20, 0x1F26, 0x1F2B,
    0x1F30, 0x1F36, 0x1F3B, 0x1F40, 0x1F45, 0x1F4B, 0x1F50, 0x1F55, 0x1F5A, 0x1F60, 0x1F65, 0x1F6A,
    0x1F6F, 0x1F75, 0x1F7A, 0x1F7F, 0x1F84, 0x1F8A, 0x1F8F, 0x1F94, 0x1F99, 0x1F9E, 0x1FA4, 0x1FA9,
    0x1FAE, 0x1FB3, 0x1FB8, 0x1FBD, 0x1FC3, 0x1FC8, 0x1FCD, 0x1FD2, 0x1FD7, 0x1FDC, 0x1FE1, 0x1FE6,
    0x1FEC, 0x1FF1, 0x1FF6, 0x1FFB, 0x2000,
];

/// `U_GetAtanTable`, `c_math.cpp:112-115`.
fn atn_table(y: f32, x: f32) -> u16 {
    ATN_TABLE[(y / x * 1024.0) as usize]
}

/// `cM_atan2s`, `c_cam_param` aside, `c_math.cpp:117-160`: each arm computes in the `int` the
/// reference widens into, then narrows to `s16`, so the arms that pass a quarter or a half turn
/// wrap rather than saturate.
fn atan2s(y: f32, x: f32) -> i16 {
    if is_zero(y) {
        return if x >= 0.0 { 0 } else { 0x8000_u32 as i16 };
    }
    if is_zero(x) {
        return if y >= 0.0 { 0x4000 } else { -0x4000 };
    }
    if y >= 0.0 {
        if x >= 0.0 {
            if x >= y {
                atn_table(y, x) as i16
            } else {
                0x4000 - atn_table(x, y) as i16
            }
        } else if -x < y {
            (0x4000 + i32::from(atn_table(-x, y))) as i16
        } else {
            (0x8000 - i32::from(atn_table(y, -x))) as i16
        }
    } else if x < 0.0 {
        if x <= y {
            (0x8000 + i32::from(atn_table(-y, -x))) as i16
        } else {
            (0xC000 - i32::from(atn_table(-x, -y))) as i16
        }
    } else if x < -y {
        (0xC000 + i32::from(atn_table(x, -y))) as i16
    } else {
        -(atn_table(-y, x) as i16)
    }
}

/// `cM_atan2f`, `c_math.cpp:163-165`.
fn atan2f(y: f32, x: f32) -> f32 {
    atan2s(y, x) as f32 * S2RAD
}

/// `cSPolar` from `c_angle.h:131`, the polar half of a globe and the only place the retail
/// camera's trigonometry happens.
#[derive(Debug, Clone, Copy, PartialEq)]
struct SPolar {
    radial: f32,
    /// The polar angle, measured off +Y.
    angle1: SAngle,
    /// The XZ azimuth, zero along +Z.
    angle2: SAngle,
}

impl SPolar {
    /// `cSPolar::Val(f32, s16, s16)`, `c_angle.cpp:208-213`.
    fn val(&mut self, radial: f32, angle1: SAngle, angle2: SAngle) {
        self.radial = radial;
        self.angle1 = angle1;
        self.angle2 = angle2;
        self.formal();
    }

    /// `cSPolar::Val(const cXyz&)`, `c_angle.cpp:220-232`.
    fn val_xyz(&mut self, v: Vec3) {
        let horizontal = f64::from(v.z) * f64::from(v.z) + f64::from(v.x) * f64::from(v.x);
        let square = f64::from(v.y) * f64::from(v.y) + horizontal;
        let polar = if horizontal > 0.0 {
            horizontal.sqrt() as f32
        } else {
            0.0
        };
        self.radial = if square > 0.0 {
            square.sqrt() as f32
        } else {
            0.0
        };
        self.angle1 = SAngle::from_radians(atan2f(polar, v.y));
        self.angle2 = SAngle::from_radians(atan2f(v.x, v.z));
        self.formal();
    }

    /// `cSPolar::Formal`, `c_angle.cpp:191-205`.
    fn formal(&mut self) {
        if self.radial < 0.0 {
            self.radial = -self.radial;
            self.angle1 = SAngle(i16::MIN) - self.angle1;
            self.angle2 = self.angle2.inv();
        }
        if self.angle1.val() < 0 && self.angle1.val() != i16::MIN {
            self.angle1 = -self.angle1;
            self.angle2 = self.angle2.inv();
        }
    }

    /// `cSPolar::Xyz`, `c_angle.cpp:235-243`.
    fn to_xyz(&self) -> Vec3 {
        let a1_sin = self.radial * self.angle1.sin();
        let a1_cos = self.radial * self.angle1.cos();
        Vec3::new(
            a1_sin * self.angle2.sin(),
            a1_cos,
            a1_sin * self.angle2.cos(),
        )
    }
}

/// `cSGlobe` from `c_angle.h:151`: the spherical state the retail camera carries for its own
/// eye-to-centre vector.
///
/// The retail accessors disagree with each other about which member they touch: the getters
/// read `V()` for `mAzimuth` and `U()` for `mInclination`, while the setters in the
/// reference source write the other member each time. The getters are the ones the call sites
/// agree with, because `V()` is the field `followCamera` clamps against the style's latitude
/// limits at `d_camera.cpp:2870-2876` and `calc` compares against +-90 degrees at
/// `d_camera.cpp:867`, and `U()` is the field it chases with a yaw. `Xyz` settles it
/// independently through `Polar` at `c_angle.cpp:327-329`. So this port names the two fields
/// after what they measure and drops the accessor spellings.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SGlobe {
    radius: f32,
    latitude: SAngle,
    longitude: SAngle,
}

impl SGlobe {
    /// `cSGlobe::Val(f32, const cSAngle&, const cSAngle&)`, `c_angle.cpp:305-310`: the first
    /// angle is the latitude, the second the longitude, in the member order the ctor takes.
    pub fn new(radius: f32, latitude: SAngle, longitude: SAngle) -> Self {
        let mut globe = Self {
            radius,
            latitude,
            longitude,
        };
        globe.formal();
        globe
    }

    /// `cSGlobe::Val(const cXyz&)`, `c_angle.cpp:313-317`, through `cSPolar::Globe`.
    pub fn from_xyz(v: Vec3) -> Self {
        let mut polar = SPolar::ZERO;
        polar.val_xyz(v);
        Self::new(polar.radial, SAngle::QUARTER - polar.angle1, polar.angle2)
    }

    /// `cSGlobe::R`, `c_angle.h:175`.
    pub fn radius(&self) -> f32 {
        self.radius
    }

    /// `cSGlobe::R(f32)`, `c_angle.h:179`.
    pub fn set_radius(&mut self, radius: f32) {
        self.radius = radius;
    }

    /// `cSGlobe::V`, `c_angle.h:176`: the latitude off the XZ plane.
    pub fn latitude(&self) -> SAngle {
        self.latitude
    }

    /// `cSGlobe::V(cSAngle)`, `c_angle.h:181`.
    pub fn set_latitude(&mut self, latitude: SAngle) {
        self.latitude = latitude;
    }

    /// `cSGlobe::U`, `c_angle.h:177`: the azimuth in the XZ plane, zero along +Z.
    pub fn longitude(&self) -> SAngle {
        self.longitude
    }

    /// `cSGlobe::U(cSAngle)`, `c_angle.h:180`.
    pub fn set_longitude(&mut self, longitude: SAngle) {
        self.longitude = longitude;
    }

    /// `cSGlobe::Xyz`, `c_angle.cpp:320-324`, through `Polar` at `c_angle.cpp:327-329`: the
    /// longitude turns the radius about Y and the latitude lifts it off the XZ plane.
    pub fn to_xyz(&self) -> Vec3 {
        let mut polar = SPolar::ZERO;
        polar.val(self.radius, SAngle::QUARTER - self.latitude, self.longitude);
        polar.to_xyz()
    }

    /// `cSGlobe::Invert`, `c_angle.cpp:332-335`.
    pub fn invert(&mut self) {
        self.radius = -self.radius;
        self.formal();
    }

    /// `cSGlobe::Formal`, `c_angle.cpp:273-286`: a negative radius or a latitude past the pole
    /// folds back by negating the radius and inverting the other angle.
    fn formal(&mut self) {
        if self.radius < 0.0 {
            self.radius = -self.radius;
            self.latitude = -self.latitude;
            self.longitude = self.longitude.inv();
        }
        if self.latitude.val() < -0x4000 || 0x4000 < self.latitude.val() {
            self.latitude = SAngle(i16::MIN) - self.latitude;
            self.longitude = self.longitude.inv();
        }
    }
}

impl SPolar {
    /// The ctor retail leaves defaulted, which `cSPolar::Val` overwrites before anything reads.
    const ZERO: Self = Self {
        radial: 0.0,
        angle1: SAngle::ZERO,
        angle2: SAngle::ZERO,
    };
}

/// `dCamMath::rationalBezierRatio`, `d_cam_param.cpp:13-46`: how far a quadratic Bezier of
/// control distance `y` has come at `x`, in the double precision retail computes it in.
pub fn rational_bezier_ratio(mut x: f32, y: f32) -> f32 {
    let sign = if x >= 0.0 {
        1.0
    } else {
        x = -x;
        -1.0
    };
    let two_x = 2.0 * x;
    let two_y = 2.0 * y;
    let product = two_x * y;
    let lead = f64::from(product) - f64::from(two_x) - f64::from(two_y);
    let c = -lead - 1.0;
    let discriminant = lead * lead - 4.0 * c * f64::from(x);
    let root = -lead
        - if discriminant > 0.0 {
            discriminant.sqrt()
        } else {
            0.0
        };
    let denom = c * 2.0;
    if denom > 1e-07 || denom < -1e-07 {
        let t = root / denom;
        let square = t * t;
        let back = 1.0 - t;
        let norm = square + (back * back + f64::from(y) * (2.0 * back * t));
        if norm > 1e-07 {
            return (sign * (square / norm)) as f32;
        }
    }
    0.0
}

/// `dCamMath::xyzHorizontalDistance`, `d_cam_param.cpp:86-91`.
pub fn horizontal_distance(a: Vec3, b: Vec3) -> f64 {
    let x = f64::from(a.x) - f64::from(b.x);
    let z = f64::from(a.z) - f64::from(b.z);
    (x * x + z * z).sqrt()
}

/// `limitf` from the anonymous namespace at `d_camera.cpp:38-45`.
pub fn limitf(value: f32, min: f32, max: f32) -> f32 {
    if value > max {
        max
    } else if value < min {
        min
    } else {
        value
    }
}

/// `dCamSetup_c` from `d_cam_param.h:281`, narrowed to the members the follow engine reads.
/// Retail keeps it on `dCamera_c`, so it lives on [`Camera`] here. Every default is
/// `dCamSetup_c::dCamSetup_c()`, `d_cam_param.cpp:303-364`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CamSetup {
    /// `m060`: the latitude a placement may not pass above.
    pub latitude_upper: f32,
    /// `m05C`: the latitude a placement may not pass below.
    pub latitude_lower: f32,
    /// `mChargeLatitude`, in degrees.
    pub charge_latitude: f32,
    /// `mChargeTimer`: the frames a charge pull-in is spread over.
    pub charge_timer: i32,
    /// `mChargeBRatio`: the ease a charge pull-in runs on.
    pub charge_b_ratio: f32,
    /// `m080`: the bank a leaf fan's camera rolls to.
    pub fan_bank: f32,
    /// `m084`: how far the wind swings that bank.
    pub fan_bank_wind: f32,
    /// `m078`: the field of view a leaf fan adds while it is held.
    pub leaf_fovy: f32,
    /// `m07C`: how fast that field of view comes and goes.
    pub leaf_fovy_rate: f32,
}

impl Default for CamSetup {
    fn default() -> Self {
        Self {
            latitude_upper: 80.0,
            latitude_lower: -60.0,
            charge_latitude: 27.0,
            charge_timer: 20,
            charge_b_ratio: 2.0,
            fan_bank: 10.0,
            fan_bank_wind: 4.0,
            leaf_fovy: 0.5,
            leaf_fovy_rate: 0.4,
        }
    }
}

impl CamSetup {
    /// `dCamSetup_c::FanBank`, `d_cam_param.cpp:381`: the base plus a wind-driven wobble. The
    /// port has no weather feed, so the caller supplies the power and zero leaves the base.
    pub fn bank(&self, wind_pow: f32) -> f32 {
        self.fan_bank + self.fan_bank_wind * wind_pow
    }
}

/// `dCamera_c::mViewCache`, `d_camera.h:142-155`: the eye, the point it looks at, and the globe
/// that joins them.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewCache {
    pub direction: SGlobe,
    pub center: Vec3,
    pub eye: Vec3,
    pub bank: SAngle,
    /// Degrees, as `mFovy` keeps it against the radians projection.
    pub fovy: f32,
}

impl Default for ViewCache {
    fn default() -> Self {
        Self {
            direction: SGlobe::new(0.0, SAngle::ZERO, SAngle::ZERO),
            center: Vec3::ZERO,
            eye: Vec3::ZERO,
            bank: SAngle::ZERO,
            fovy: 45.0,
        }
    }
}

/// `dCamera_c::mWork.follow`, `d_camera.h:275-307`, member for member. The members
/// `followCamera` never reads back (`m378`, `m390`, `m3DA`, `m3DB`, `m3E4`) are left out
/// rather than carried unread.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Follow {
    /// The style the placement state belongs to, so a style change replays it.
    pub style: i16,
    /// `m37C`: the frames the placement is spread over.
    pub steps: i32,
    /// `m380`: the per-frame steps still to be spent.
    pub step_budget: f32,
    /// `m384`: this frame's share of that budget.
    pub step_left: f32,
    /// `m388`: the frames spent bringing a fresh placement up to speed.
    pub ramp: i32,
    /// `m38C`: the side lock's one-shot direction latch and its countdown.
    pub side_lock: i32,
    /// `m392`: the frames a charge has been held for.
    pub charge_frames: i16,
    /// `m394`: how far along the ramp the camera is.
    pub ramp_ratio: f32,
    /// `m398`: the style's near radius, converging.
    pub radius_near: f32,
    /// `m39C`: the style's far radius, converging.
    pub radius_far: f32,
    /// `m3A0`: the latitude, in degrees.
    pub pitch_deg: f32,
    /// `m3A4`: the player's height last frame, for the flying test.
    pub player_y: f32,
    /// `m3A8`: the field of view the placement started from.
    pub fovy: f32,
    /// `m3AC`: the eye's yaw offset from straight behind.
    pub yaw_deg: f32,
    /// `m3B0`: the eye's drift above the attention point.
    pub drift_y: f32,
    /// `m3B4`: set while the player aims or hangs, which stiffens the latitude.
    pub aiming: bool,
    /// `m3B8`: how hard the yaw is pulled onto the player.
    pub pull: f32,
    /// `m3BC`: the latitude the style wants, in degrees.
    pub pitch_target: f32,
    /// `m3C0`: the point the centre walks toward.
    pub target: Vec3,
    /// `m3CC`: the eye the placement wants.
    pub eye: Vec3,
    /// `m3D8`: set while a charge pull-in is running.
    pub charging: bool,
    /// `m3DC`: the radius lerp, quick inside the style's range and slow outside it.
    pub radius_lerp: f32,
    /// `m3E0`: the latitude lerp.
    pub pitch_lerp: f32,
    /// `m3E8`: the field-of-view lerp.
    pub fovy_lerp: f32,
    /// `m3EC`: the centre lerp across X and Z.
    pub lerp_horizontal: f32,
    /// `m3F0`: the centre lerp on Y.
    pub lerp_vertical: f32,
    /// `m148`: the leaf fan's lean, which biases the latitude target.
    pub lean: SAngle,
    /// `bVar3` from `d_camera.cpp:2945-2947`, the near half of the style's radius.
    pub probe_near: bool,
    /// `bVar4` from `d_camera.cpp:2949-2956`, whether the ground probe runs at all.
    pub probe_allowed: bool,
}

impl Default for Follow {
    fn default() -> Self {
        Self {
            style: cam_style::NONE,
            steps: 1,
            step_budget: 0.0,
            step_left: 0.0,
            ramp: 0,
            side_lock: 0,
            charge_frames: 0,
            ramp_ratio: 0.0,
            radius_near: 0.0,
            radius_far: 0.0,
            pitch_deg: 0.0,
            player_y: 0.0,
            fovy: 0.0,
            yaw_deg: 0.0,
            drift_y: 0.0,
            aiming: false,
            pull: 0.0,
            pitch_target: 0.0,
            target: Vec3::ZERO,
            eye: Vec3::ZERO,
            charging: false,
            radius_lerp: 1.0,
            pitch_lerp: 0.0,
            fovy_lerp: 0.0,
            lerp_horizontal: 0.0,
            lerp_vertical: 0.0,
            lean: SAngle::ZERO,
            probe_near: false,
            probe_allowed: true,
        }
    }
}

/// What the follow engine reads off the pad: the main stick's clamped vector, which retail
/// keeps as `mStickMainPosXLast` / `mStickMainPosYLast` / `mStickMainValueLast`, and the
/// player's own status words.
pub struct FollowInput<'a> {
    pub main: Vec2,
    pub status: &'a PlayerStatus,
}

/// The player's `mHeight`, which the create handler writes once and nothing else touches, so
/// the whole of what `getHeight` returns for a player is this number.
const PLAYER_HEIGHT: f32 = 125.0;

/// `fVar40` at `d_camera.cpp:2625`: the lerp a placement moves at.
const CUSHION: f32 = 0.9;

/// `local_494` at `d_camera.cpp:2631`: the hard ceiling the latitude may not pass.
const PITCH_CAP_DEG: f32 = 80.0;

/// `f14` at `d_camera.cpp:2655`: the placement length's gain in height.
const HEIGHT_SCALE: f32 = 3.8;

/// `m388`'s ceiling, `d_camera.cpp:2910-2912`, and the divisor of eighty it is scaled by.
const RAMP_FRAMES: i32 = 0x50;

/// `d_camera.cpp:2912`: the ease the ramp runs on.
const RAMP_EASE: f32 = 1.25;

/// `d_camera.cpp::relationalPos`, `d_camera.cpp:1594-1604`: the offset turned into the
/// actor's yaw frame and added to the attention position.
///
/// The reference source spells the setter `V`, which its own `cSGlobe` setters would route to the
/// latitude and so would turn nothing; the getter it reads beside that write, `U`, is the
/// longitude, and adding a yaw to a longitude is the turn the function exists to perform. This
/// port follows the call's meaning and writes the longitude. `attentionPos` is the player's own
/// position, because the port has no `dAttention_c`; the style's `CENTER_HEIGHT_BASE` still
/// carries the height the offset puts back.
pub fn relational_pos(player: &Player, facing: SAngle, offset: Vec3) -> Vec3 {
    let mut globe = SGlobe::from_xyz(offset);
    globe.set_longitude(facing + globe.longitude());
    player.pos + globe.to_xyz()
}

/// `dCamera_c::initialize`'s aim, `d_camera.cpp:341-356`: the one place retail aims a camera
/// that has never been aimed. The centre lands on the attention point lifted by the style's
/// centre height and the globe straight back along the player's facing, at a fixed radius the
/// style then pulls in over the placement ramp. `initialize` runs once, so this only ever fires
/// on a view cache that is still empty.
fn seed(cache: &mut ViewCache, player: &Player, center_height: f32, facing: SAngle) {
    cache.center = player.pos + Vec3::new(0.0, center_height, 0.0);
    cache.direction = SGlobe::new(200.0, SAngle::ZERO, facing.inv());
    cache.eye = cache.center + cache.direction.to_xyz();
}

/// `dCamera_c::followCamera(s32 style)`, `d_camera.cpp:2618-3274`.
///
/// False for a style whose engine is not `Follow`, which is what dispatching through
/// `engine_tbl[Algorythmn]` does at `d_camera.cpp:797`. This is the readable spine of the
/// function: the style parameters, the placement ramp, the centre, radius, latitude and
/// field-of-view smoothing, the yaw chase and the charge pull-in.
///
/// The order below is the order the reference runs in, and each line names what this port
/// does there and what it leaves out:
///
/// - `d_camera.cpp:1211-1217`: `onModeChange` clears the placement flags and the frame
///   counter on a style change, which is what makes a style replay its placement. The port
///   notices the change against `Follow::style` rather than in `change_style`, which holds no
///   placement state of its own.
/// - `d_camera.cpp:2633-2653`: the nineteen style parameters, read through `Camera::val`.
/// - `d_camera.cpp:2661-2666`: a placement that has just started carries no offset yet.
/// - `d_camera.cpp:2710-2718`: crawling or swimming drops the eye's floor and lifts the
///   latitude's, both from the player's published status.
/// - `d_camera.cpp:2720-2729`: hanging drops the eye under the attention point outright,
///   otherwise the drift eases onto the style's at six percent a frame.
/// - `d_camera.cpp:2731-2738`: a leaf fan in flight forces a fixed placement, keyed on
///   `STTS1_UNK40000`. The flying overrides above it and the peep angle below it need actor
///   states the sim does not publish.
/// - `d_camera.cpp:2749-2751`: the style's own flag pins the yaw to the player.
/// - `d_camera.cpp:2773-2775`: the eye's yaw offset eases onto the style's at six percent a
///   frame. The arms above it at `d_camera.cpp:2753-2772` lean the offset while aiming in
///   water or in flight, which need those states.
/// - `d_camera.cpp:2778`: the eye's offset in the player's frame.
/// - `d_camera.cpp:2779-2833`: the frame a placement starts on, which measures how far the
///   camera has to come and spreads the move over a computed number of frames. The first arm
///   at `d_camera.cpp:2802` is `chkFlag(0x8000) || !m110`, read off the camera's own flag word
///   and the mode-pair test `onModeChange` makes; neither is available here, so the placement
///   is always measured.
/// - `d_camera.cpp:2835-2836`: the point the centre walks toward, and the player's facing.
///   The water-surface clamp at `d_camera.cpp:2838` needs `getWaterSurfaceHeight`, which
///   reads a roof and a sea plane this port has no feed for.
/// - `d_camera.cpp:2840-2898`: the placement ramp itself, spent a frame at a time out of the
///   budget the first frame measured. The background move at `d_camera.cpp:2841-2843` and the
///   centre's line-check push-out at `d_camera.cpp:2854-2863` belong to the collision pass.
/// - `d_camera.cpp:2906-2913`: the arm at `d_camera.cpp:2906-2909` cancels the ramp while
///   swimming, which needs that state; the ramp itself is ported.
/// - `d_camera.cpp:2919-2924`: aiming from far off the player's back stiffens the centre's
///   horizontal lerp to a tenth; the status mask behind that arm is not published here.
/// - `d_camera.cpp:2926-2938`: the vertical lerp, which only relaxes once the ramp has begun.
///   The lock-on override on the last two lines needs a mode this port does not track.
/// - `d_camera.cpp:2945-2956`: the two flags that gate the ground probe, recorded on
///   [`Follow::probe_near`] and [`Follow::probe_allowed`] for the collision pass.
/// - `d_camera.cpp:2969`: the centre follows the target at the style's own per-axis lerp,
///   `cStack_284` at `d_camera.cpp:2941`. The mode-one clamp above it at
///   `d_camera.cpp:2958-2967` and the line check below it at `d_camera.cpp:2971-2986` are
///   lock-on and collision.
/// - `d_camera.cpp:2988`: the placement as it stood before this frame moved it.
/// - `d_camera.cpp:2990-2994`: the charge pull-in. The frames held are what spend the pull and
///   the ease is the setup's, so a charge drags the camera in over its own length rather than
///   snapping.
/// - `d_camera.cpp:3014-3090`: with no charge running the pull follows the main stick, leaning
///   the camera off the player's back as the stick is pushed. The arms above it at
///   `d_camera.cpp:2995-3013` release the pull in flight or on a leaf fan.
/// - `d_camera.cpp:3069-3086`: a forced-yaw style settles onto the player's side first,
///   counting up out of the side lock before it will follow the stick. The peeping arm at
///   `d_camera.cpp:3077-3079` needs a ground line check.
/// - `d_camera.cpp:3093-3106`: where the yaw is pulled to. The first arm holds the yaw where it
///   is while the style allows it and nothing aims, on a flag the collision pass sets.
///   [`Camera::yaw_trim`] is this port's own addition, folded into the placement's own
///   longitude at the chase: retail reaches its free yaw through `mAngleY`, which
///   `d_camera.cpp:861` derives from the direction itself, so whatever steered it from outside
///   is not in the readable text.
/// - `d_camera.cpp:3108-3109`: the chase, scaled by the pull and by the cosine of the latitude,
///   so a pull at the pole turns nothing.
/// - `d_camera.cpp:3111-3186`: the latitude the placement eases toward.
/// - `d_camera.cpp:3112-3123`: while a charge is held the latitude goes straight to the setup's,
///   eased on the charge's own curve, and the held count ticks up.
/// - `d_camera.cpp:3138-3162`: a placement still being spent holds its latitude and eases
///   toward the style's, on a hundredth of a frame unless aiming replaces it with a quarter.
///   The demote-cam arm at `d_camera.cpp:3156-3158` needs the monitoring distances
///   `updateMonitor` keeps. The convergence at `d_camera.cpp:3139` reuses the one local the
///   reference listing also reads at `d_camera.cpp:3216-3217`, so it takes the same style
///   parameter here.
/// - `d_camera.cpp:3180-3184`: once the ramp is running the placement holds its own latitude
///   and eases onto the player at the style's flying ratio. The arms above it at
///   `d_camera.cpp:3164-3179` need aiming or flight.
/// - `d_camera.cpp:3188`: the player's height is recorded for the flying test.
/// - `d_camera.cpp:3190-3195`: aiming and hanging hold the latitude. The status mask behind it
///   is not published here, so the flag reads clear.
/// - `d_camera.cpp:3197-3213`: the latitude clamped to the style's range and eased onto it.
/// - `d_camera.cpp:3215-3231`: the radius. The style's near and far radii converge onto the
///   placement's, and the lerp is only slow while the placement sits outside them.
/// - `d_camera.cpp:3243-3244`: the eye trails the placement's own eye at three quarters of a
///   frame and the globe is read back off the result.
/// - `d_camera.cpp:3250-3251`: the field of view, on a lerp that settles onto the style's. The
///   leaf fan's jitter at `d_camera.cpp:3253-3255` and the flight sway at
///   `d_camera.cpp:3257-3271` need states and a wind feed this port does not have.
/// - `d_camera.cpp:3234-3241`: the ground probe along the placement, left to the collision pass.
pub fn follow(camera: &mut Camera, player: &Player, input: &FollowInput) -> bool {
    if camera.engine() != Some(CamEngine::Follow) {
        return false;
    }

    let mut follow = camera.follow;
    let mut cache = camera.view_cache;

    if follow.style != camera.current_style {
        follow.style = camera.current_style;
        camera.age = 0;
        camera.settled = false;
    }
    let first = camera.age == 0;
    camera.age = camera.age.wrapping_add(1);

    if first {
        follow.yaw_deg = 0.0;
        follow.drift_y = 0.0;
    }

    let setup = camera.setup;
    let charge_latitude = SAngle::from_degrees(setup.charge_latitude);
    let charge_timer = setup.charge_timer;
    let charge_b_ratio = setup.charge_b_ratio;
    let pitch_cap = SAngle::from_degrees(PITCH_CAP_DEG);

    let yaw_offset = camera.val(param::UNK1);
    let mut center_height = camera.val(param::CENTER_HEIGHT_BASE);
    let drift_target = camera.val(param::UNK0);
    let lerp_vertical = camera.val(param::UNK4);
    let lerp_horizontal = camera.val(param::UNK3);
    let mut radius_far = camera.val(param::UNK10);
    let mut radius_near = camera.val(param::UNK11);
    let radius_lerp_target = camera.val(param::UNK13);
    let radius_converge = camera.val(param::UNK14);
    let mut pitch_deg = camera.val(param::UNK15);
    let mut pitch_floor = SAngle::from_degrees(camera.val(param::UNK16));
    let pitch_ceiling = SAngle::from_degrees(camera.val(param::UNK17));
    let mut pitch_weight = camera.val(param::LOCKON_LATITUDE_MAX);
    let fly_pitch_weight = camera.val(param::LOCKON_LATITUDE_MIN);
    let mut target_fovy = camera.val(param::FOVY_BASE);
    let fovy_converge = camera.val(param::LOCKON_FOVY_MAX);
    let pull_up_ratio = camera.val(param::LOCKON_LONGITUDE_MIN);
    let pull_down_ratio = camera.val(param::LOCKON_LONGITUDE_MAX);
    let pull_gain = camera.val(param::UNK20);

    if input
        .status
        .check0(player_status::STTS0_CRAWL | player_status::STTS0_SWIM)
    {
        pitch_floor = pitch_floor.max(SAngle::from_degrees(4.0));
        center_height = center_height.max(-10.0);
    }

    if input
        .status
        .check0(player_status::STTS0_UNK200 | player_status::STTS0_HANG)
        && !input.status.check0(player_status::STTS0_UNK2000000)
    {
        if drift_target > -10.0 {
            follow.drift_y = -10.0;
        }
    } else {
        follow.drift_y += (drift_target - follow.drift_y) * 0.06;
    }

    let mut forced = camera.has_flag(cam_style::flag::UNK200);

    if input.status.check1(player_status::STTS1_UNK40000) {
        follow.lean = SAngle::ZERO;
        pitch_deg = -24.0;
        radius_near = 420.0;
        radius_far = 420.0;
        target_fovy = 80.0;
        center_height = 140.0;
        forced = true;
    }

    follow.yaw_deg += (yaw_offset - follow.yaw_deg) * 0.06;

    let offset = Vec3::new(follow.yaw_deg, center_height, follow.drift_y);
    let facing = SAngle::from_radians(player.face);

    if first {
        if cache.direction.radius() == 0.0 {
            seed(&mut cache, player, center_height, facing);
        }

        cache.direction = SGlobe::from_xyz(cache.eye - cache.center);
        follow.ramp = RAMP_FRAMES;
        follow.ramp_ratio = CUSHION;
        follow.radius_near = radius_near;
        follow.radius_far = radius_far;
        follow.side_lock = 0;
        follow.charge_frames = 0;
        follow.pitch_deg = cache.direction.latitude().degree();
        follow.pitch_target = follow.pitch_deg;
        follow.target = cache.center;
        follow.eye = cache.eye;
        follow.fovy_lerp = 0.01;
        follow.pitch_lerp = 0.01;
        follow.radius_lerp = 0.75;
        follow.lerp_horizontal = lerp_horizontal;
        follow.lerp_vertical = lerp_vertical;
        follow.aiming = false;
        follow.charging = true;
        follow.fovy = cache.fovy;
        follow.pull = 0.0;
        follow.player_y = player.pos.y;

        let anchor = relational_pos(player, facing, offset);
        let aim = anchor
            + SGlobe::new(
                radius_far,
                SAngle::from_degrees(pitch_deg),
                camera.angle_y.inv(),
            )
            .to_xyz();
        let to_eye = cache.eye - aim;
        let to_center = (cache.center - anchor).abs().length() * 4.0;
        let travel = to_eye.abs().length().max(to_center).abs();
        follow.steps = ((PLAYER_HEIGHT / travel.max(10.0)).sqrt() * HEIGHT_SCALE) as i32 + 1;

        follow.radius_near = cache.direction.radius();
        follow.radius_far = cache.direction.radius();
        follow.fovy = cache.fovy;
        follow.step_budget = ((follow.steps * (follow.steps + 1)) >> 1) as f32;
        follow.step_left = 0.0;
    }

    let facing_diff = facing - cache.direction.longitude();
    let target = relational_pos(player, facing, offset);

    if !camera.settled {
        follow.step_left = follow.steps as f32 - camera.age as f32;
        let share = follow.step_left / follow.step_budget;
        follow.target += (target - follow.target) * share;
        cache.center += (follow.target - cache.center) * lerp_horizontal;

        let radius = limitf(cache.direction.radius(), radius_near, radius_far);
        let latitude = cache
            .direction
            .latitude()
            .max(pitch_floor)
            .min(pitch_ceiling);
        let want = SGlobe::new(radius, latitude, camera.angle_y.inv());
        let current = cache.direction.radius();
        cache
            .direction
            .set_radius(current + CUSHION * (want.radius() - current));
        let current = cache.direction.latitude();
        cache
            .direction
            .set_latitude(current + (want.latitude() - current) * CUSHION);
        cache.eye = cache.center + cache.direction.to_xyz();
        follow.eye = cache.eye;

        if camera.age >= (follow.steps - 1) as u32 {
            camera.settled = true;
        }

        follow.pitch_deg = cache.direction.latitude().degree();
        follow.radius_near = cache.direction.radius();
        follow.radius_far = cache.direction.radius();
        cache.fovy += CUSHION * (target_fovy - cache.fovy);
        follow.step_budget -= follow.step_left;

        camera.follow = follow;
        camera.view_cache = cache;
        publish(camera);
        return true;
    }

    // `m388` counts up from the reset `checkGroundInfo` performs when the floor margin test
    // fails; the counter gates the pitch convergence and the wall-up lift. `cam_bump` owns the
    // reset, so it also owns the increment, and this stage only consumes the value.
    if follow.ramp >= 0 && follow.ramp < RAMP_FRAMES {
        follow.ramp += 1;
        follow.ramp_ratio += (CUSHION - follow.ramp_ratio)
            * rational_bezier_ratio(follow.ramp as f32 / 80.0, RAMP_EASE);
    }

    follow.lerp_horizontal = lerp_horizontal;

    follow.lerp_vertical = if follow.ramp == 0 {
        lerp_vertical
    } else {
        lerp_vertical * 0.1 + (1.0 - lerp_vertical * 0.1) * follow.ramp_ratio
    };

    follow.probe_near = false;
    follow.probe_allowed = follow.ramp == 0;

    let centre_lerp = Vec3::new(
        follow.lerp_horizontal,
        follow.lerp_vertical,
        follow.lerp_horizontal,
    );
    cache.center += (target - cache.center) * centre_lerp;

    let before = SGlobe::from_xyz(cache.eye - cache.center);

    let stick_x = input.main.x;
    let stick_y = input.main.y;
    let stick_value = input.main.length();

    if follow.charge_frames > 0 && follow.charge_frames as i32 <= charge_timer {
        follow.pull = (1.0 - follow.pull)
            * rational_bezier_ratio(
                follow.charge_frames as f32 / charge_timer as f32,
                charge_b_ratio,
            );
        follow.charging = true;
    } else {
        follow.charging = true;
        follow.pull = if stick_y >= 0.0 {
            let eased = rational_bezier_ratio(stick_x, pull_up_ratio);
            1.0 - (SAngle::from_degrees(eased * 180.0).cos() * 0.5 + 0.5)
        } else {
            let eased = rational_bezier_ratio(stick_x, pull_down_ratio);
            1.0 - (SAngle::from_degrees(eased * 180.0).cos() * 0.25 + 0.75)
        };
        follow.pull *= stick_value;
        follow.pull *= 0.1;

        if forced {
            if follow.side_lock == 0
                && (facing_diff <= SAngle::THREE_QUARTER || facing_diff >= SAngle::QUARTER)
            {
                follow.side_lock = 1;
            } else if follow.side_lock < 0xf {
                follow.pull = follow.side_lock as f32 * 0.033333335;
                follow.side_lock += 1;
            } else if stick_value < 0.1 {
                follow.pull = 0.05;
            } else {
                follow.pull = rational_bezier_ratio(facing_diff.sin().abs(), 12.0);
            }
        } else {
            follow.pull *= pull_gain;
            follow.side_lock = 0;
        }
    }

    let want_yaw = if forced { facing } else { facing.inv() };
    let trim = camera.yaw_trim;
    camera.yaw_trim = SAngle::ZERO;
    let base_yaw = before.longitude() + trim;

    cache
        .direction
        .set_longitude(base_yaw + (want_yaw - base_yaw) * (follow.pull * before.latitude().cos()));

    let mut latitude;
    if input.status.check1(player_status::STTS1_UNK20000) {
        latitude = charge_latitude;
        follow.pitch_lerp = if follow.charge_frames as i32 <= charge_timer {
            let eased = rational_bezier_ratio(
                follow.charge_frames as f32 / charge_timer as f32,
                charge_b_ratio,
            );
            follow.charge_frames += 1;
            eased
        } else {
            1.0
        };
        follow.pitch_target = latitude.degree();
        follow.pitch_deg = latitude.degree();
    } else {
        if follow.charge_frames != 0 {
            follow.pitch_lerp = 0.0;
        }
        follow.charge_frames = 0;

        if follow.ramp == 0 {
            follow.pitch_target += radius_converge * (pitch_deg - follow.pitch_target);
            let lerp: f32 = if follow.aiming {
                pitch_weight = 0.25;
                0.25
            } else {
                0.01
            };
            follow.pitch_deg +=
                pitch_weight * ((follow.pitch_target + follow.lean.degree()) - follow.pitch_deg);
            latitude = SAngle::from_degrees(follow.pitch_deg);
            follow.pitch_lerp += lerp * (CUSHION - follow.pitch_lerp);
        } else {
            latitude = before.latitude();
            follow.pitch_target = latitude.degree();
            follow.pitch_deg = latitude.degree();
            follow.pitch_lerp = rational_bezier_ratio(follow.ramp_ratio, fly_pitch_weight);
        }
    }

    follow.player_y = player.pos.y;

    follow.aiming = false;

    if latitude < pitch_floor {
        latitude = pitch_floor;
    } else if latitude > pitch_ceiling {
        latitude = pitch_ceiling;
    }

    let current = cache.direction.latitude();
    cache
        .direction
        .set_latitude(current + (latitude - current) * follow.pitch_lerp);

    if cache.direction.latitude() > pitch_cap {
        cache.direction.set_latitude(pitch_cap);
    }

    let mut radius = before.radius();
    follow.radius_near += radius_converge * (radius_near - follow.radius_near);
    follow.radius_far += radius_converge * (radius_far - follow.radius_far);
    if radius < follow.radius_near {
        follow.radius_lerp += (radius_lerp_target - follow.radius_lerp) * 0.01;
        radius = follow.radius_near;
    } else if radius > follow.radius_far {
        follow.radius_lerp += (radius_lerp_target - follow.radius_lerp) * 0.01;
        radius = follow.radius_far;
    } else {
        follow.radius_lerp = 1.0;
    }
    let current = cache.direction.radius();
    cache
        .direction
        .set_radius(current + follow.radius_lerp * (radius - current));

    follow.eye = cache.center + cache.direction.to_xyz();

    cache.eye += (follow.eye - cache.eye) * 0.75;
    cache.direction = SGlobe::from_xyz(cache.eye - cache.center);

    follow.fovy_lerp += (fovy_converge - follow.fovy_lerp) * 0.01;
    cache.fovy += follow.fovy_lerp * (target_fovy - cache.fovy);

    camera.follow = follow;
    camera.view_cache = cache;
    publish(camera);
    true
}

/// `d_camera.cpp:808-812`, `:829-847` and `:864-872`, the three ends of `Run` that turn the
/// placement the engine wrote into the published aim. Every engine runs it, so it lives here
/// rather than beside the follow engine.
pub fn publish(camera: &mut Camera) {
    let mut cache = camera.view_cache;
    if camera.event_flags & crate::sim::cam_engine::evflag::BANK == 0 {
        cache.bank -= cache.bank * 0.05;
    }
    let up = if cache.center.x == cache.eye.x && cache.center.z == cache.eye.z {
        Vec3::new(0.01, 1.0, 0.0)
    } else if cache.direction.latitude() >= SAngle::from_degrees(-90.0)
        && cache.direction.latitude() <= SAngle::from_degrees(90.0)
    {
        Vec3::Y
    } else {
        Vec3::new(0.0, -1.0, 0.0)
    };
    camera.angle_y = cache.direction.longitude().inv();
    let eye = cache.eye;
    let center = cache.center;
    let fovy = cache.fovy;
    let bank = cache.bank;
    camera.view_cache = cache;
    if let Some(view) = camera.view_mut() {
        view.aim_rolled(eye, center, up, fovy.to_radians(), bank.radian());
    }
}
