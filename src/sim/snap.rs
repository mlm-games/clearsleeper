//! The snapshot packet - `d_snap.cpp` / `include/d/d_snap.h`: the figurine photographer's
//! capture pass, as far as this port can run it.
//!
//! Retail's flow has three parts. `dSnap_ReleaseShutter` (`d_snap.cpp:2188-2190`) sets the
//! packet's release bit from the picture-box window (`d_picture_box.cpp:673`). While the bit is
//! up, `dSnap_Execute` (`:2193-2195`) empties the 0x3F-row registry every frame, each drawn
//! actor's `_draw` calls `dSnap_RegistFig` (`:2212-2245`) to table a proxy cylinder (or sphere)
//! for itself, and `dSnap_packet::draw` (`:1890-1910`) paints every proxy into the frame's
//! destination-alpha channel with `GXSetDstAlpha(GX_TRUE, col*4)` under an RGBA6 pixel format
//! (`dSnap_MatDl`, `:1838-1887`). `dSnap_packet::Judge` (`:1938-1973`) then reads the shutter
//! window back with `GXPeekARGB`, buckets each pixel by the label in its top six bits
//! (`sp8 >> 26`), ratios the count against the proxy's projected area (`CalcArea2D`,
//! `:1475-1619`), and `SetResult` (`:2146-2180`) scores the buckets into `mResult`/`mResultDetail`
//! - the pair `d_picture_box.cpp:616-617` stores in the photo slot it saves.
//!
//! What this module implements: the release-shutter lifecycle (`release_shutter`,
//! `chk_release_shutter`, `execute`), the 0x3F-row registry with retail's overflow rule
//! (`dSnap_packet::Regist`, `:1802-1835`), the proxy geometry (`SetGeo`/`SetGeoSph`/`SetInf`)
//! and its projection to screen area (`CalcArea2D` over the `cylvtx`/`vdata`/`tindices` tables
//! at `:1398-1440`), the camera-cull angle test (`ChkCamCull`, `:1752-1763`), the per-figure
//! thresholds `l_CharaData` (`:47-1390`) and the full judge dispatch
//! (`m_judge_tbl`, `:16-32` + `JudgeFigure`, `:2105-2143`).
//!
//! What it cannot: the pixels. `Judge` reads the frame the picture box is looking at, and this
//! port's render path is `repame-view3d` groups presented by the shell - there is no
//! counterpart to `GXPeekARGB` and no way to write those labels either, because the label pass
//! is a destination-alpha write (`GXSetDstAlpha`) into the EFB that neither `repose`'s viewport
//! nor a `MeshGroup` exposes. [`SnapPacket::judge`] therefore takes the shutter window's words
//! as an argument: the registry, the proxy maths and the scoring are complete and run the
//! moment a caller can hand over pixels, and until then the packet holds the registrations the
//! judged frame would score. Nothing here approximates the judgement from coverage.

use bevy_ecs::prelude::*;
use glam::{Mat4, Quat, Vec3};

use super::camera::Camera;
use super::registry::ERROR_PROC_ID;
use super::walk::{from_s16, to_s16};

/// `ARRAY_SIZE(m_tbl)` (`d_snap.h:315`), the registry's row count.
pub const TABLE_LEN: usize = 0x3F;

/// `dSnap_packet::m_judge_tbl`'s length (`d_snap.cpp:16-32`): the indices
/// `SetResult` treats as special photo types rather than figures (`:2159`).
pub const JUDGE_TABLE_LEN: usize = 11;

/// `dSnap_PhotoIndex2TableIndex`'s base (`d_snap.cpp:1443-1447`): the first figure index
/// with a `l_CharaData` row.
pub const PHOTO_INDEX_BASE: u32 = 0x49;

/// The photo types the dispatch at `d_snap.cpp:16-32` and the guards at `:2222`, `:2113`
/// name: `DSNAP_TYPE_POST_e`, `DSNAP_TYPE_BIKUTSUKI_e`, `DSNAP_TYPE_COUPLE_LOOK_e`, the three
/// unknown specials, `DSNAP_TYPE_UNK48_e`, `DSNAP_TYPE_NPC_BA1_e`, `DSNAP_TYPE_UNKCF_e` and
/// `DSNAP_TYPE_LAST_INDEX_e` (`d_snap.h:9-217`).
pub const DSNAP_TYPE_NONE: u32 = 0x00;
pub const DSNAP_TYPE_POST: u32 = 0x01;
pub const DSNAP_TYPE_BIKUTSUKI: u32 = 0x02;
pub const DSNAP_TYPE_COUPLE_LOOK: u32 = 0x03;
pub const DSNAP_TYPE_UNK04: u32 = 0x04;
pub const DSNAP_TYPE_UNK05: u32 = 0x05;
pub const DSNAP_TYPE_UNK06: u32 = 0x06;
pub const DSNAP_TYPE_UNK48: u32 = 0x48;
pub const DSNAP_TYPE_NPC_BA1: u32 = 0x4A;
pub const DSNAP_TYPE_UNKCF: u32 = 0xCF;
pub const DSNAP_TYPE_LAST_INDEX: u32 = 0xD0;

/// `DSNAP_SHUTTER_*` (`d_snap.cpp:1392-1396`): the screen rectangle `Judge` reads back.
pub const SHUTTER_LEFT: i32 = 174;
pub const SHUTTER_RIGHT: i32 = 468;
pub const SHUTTER_TOP: i32 = 86;
pub const SHUTTER_BOTTOM: i32 = 284;
pub const SHUTTER_WIDTH: i32 = SHUTTER_RIGHT - SHUTTER_LEFT;
pub const SHUTTER_HEIGHT: i32 = SHUTTER_BOTTOM - SHUTTER_TOP;

/// `mCullAngle`'s no-test value (`d_snap.cpp:1464`, `:1753`).
pub const CULL_ANGLE_NONE: i16 = 0x7FFF;

/// `GXPeekARGB`'s word carries the RGBA6 destination alpha in its top six bits
/// (`d_snap.cpp:1855` sets `GX_PF_RGBA6_Z24`, `:1953` reads `sp8 >> 26`), and the label pass
/// writes `col * 4` there (`:1901`), so the six bits are the registry row.
const LABEL_SHIFT: u32 = 26;

/// The cylinder's unit-circle vertices (`cylvtx`, `d_snap.cpp:1398-1404`). `SetInf`'s `r7` is
/// `4` at the only call site (`:2243`), and `CalcArea2D` walks `i` and `i + 1`, so the closed
/// loop needs the fifth entry.
const CYL_VERTS: [[f32; 3]; 5] = [
    [0.0, 0.0, 1.0],
    [0.707099974155426, 0.0, 0.707099974155426],
    [1.0, 0.0, 0.0],
    [0.707099974155426, 0.0, -0.707099974155426],
    [0.0, 0.0, -1.0],
];

/// `SetInf`'s segment count at the one call site (`d_snap.cpp:2243`): `field_0x19 = 4`.
pub const CYL_SEGMENTS: usize = 4;

/// `vdata` (`d_snap.cpp:1405-1418`), the sphere proxy's unit icosahedron.
const SPHERE_VERTS: [[f32; 3]; 12] = [
    [-0.525731086730957, 0.0, 0.8506507873535156],
    [0.525731086730957, 0.0, 0.8506507873535156],
    [-0.525731086730957, 0.0, -0.8506507873535156],
    [0.525731086730957, 0.0, -0.8506507873535156],
    [0.0, 0.8506507873535156, 0.525731086730957],
    [0.0, 0.8506507873535156, -0.525731086730957],
    [0.0, -0.8506507873535156, 0.525731086730957],
    [0.0, -0.8506507873535156, -0.525731086730957],
    [0.8506507873535156, 0.525731086730957, 0.0],
    [-0.8506507873535156, 0.525731086730957, 0.0],
    [0.8506507873535156, -0.525731086730957, 0.0],
    [-0.8506507873535156, -0.525731086730957, 0.0],
];

/// `tindices` (`d_snap.cpp:1419-1440`).
const TRI_INDICES: [[usize; 3]; 20] = [
    [0x0, 0x4, 0x1],
    [0x0, 0x9, 0x4],
    [0x9, 0x5, 0x4],
    [0x4, 0x5, 0x8],
    [0x4, 0x8, 0x1],
    [0x8, 0xA, 0x1],
    [0x8, 0x3, 0xA],
    [0x5, 0x3, 0x8],
    [0x5, 0x2, 0x3],
    [0x2, 0x7, 0x3],
    [0x7, 0xA, 0x3],
    [0x7, 0x6, 0xA],
    [0x7, 0xB, 0x6],
    [0xB, 0x0, 0x6],
    [0x0, 0x1, 0x6],
    [0x6, 0x1, 0xA],
    [0x9, 0x0, 0xB],
    [0x9, 0xB, 0x2],
    [0x9, 0x2, 0x5],
    [0x7, 0x2, 0xB],
];

/// `CharaData` (`d_snap.cpp:36-45`): one figure's proxy size, judge thresholds and gallery
/// room.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CharaData {
    pub offset: [f32; 3],
    pub radius: f32,
    pub height: f32,
    pub min_ratio: u16,
    pub cull_angle: i16,
    pub min_pixels: u16,
    pub fig_room: u8,
    pub m11: u8,
}

/// `l_CharaData` (`d_snap.cpp:47-1390`), retail arm of `DEMO_SELECT`/`VERSION_SELECT`
/// (`include/global.h:60-73`), one row per figure index past [`PHOTO_INDEX_BASE`].
pub static CHARA_DATA: [CharaData; 0x86] = [
    // 0x00
    CharaData {
        offset: [0.0, 10.0, 0.0],
        radius: 25.0,
        height: 80.0,
        min_ratio: 500,
        cull_angle: 0x2000,
        min_pixels: 5000,
        fig_room: 0x03,
        m11: 0x02,
    },
    // 0x01
    CharaData {
        offset: [0.0, 20.0, 20.0],
        radius: 40.0,
        height: 90.0,
        min_ratio: 500,
        cull_angle: 0x2000,
        min_pixels: 5000,
        fig_room: 0x03,
        m11: 0x02,
    },
    // 0x02
    CharaData {
        offset: [0.0, 10.0, 0.0],
        radius: 35.0,
        height: 130.0,
        min_ratio: 500,
        cull_angle: 0x2000,
        min_pixels: 5000,
        fig_room: 0x03,
        m11: 0x02,
    },
    // 0x03
    CharaData {
        offset: [0.0, 10.0, 10.0],
        radius: 50.0,
        height: 130.0,
        min_ratio: 300,
        cull_angle: 0x7fff,
        min_pixels: 3000,
        fig_room: 0x03,
        m11: 0x02,
    },
    // 0x04
    CharaData {
        offset: [0.0, 20.0, 0.0],
        radius: 45.0,
        height: 160.0,
        min_ratio: 500,
        cull_angle: 0x2710,
        min_pixels: 2000,
        fig_room: 0x03,
        m11: 0x02,
    },
    // 0x05
    CharaData {
        offset: [0.0, -30.0, 0.0],
        radius: 30.0,
        height: 60.0,
        min_ratio: 500,
        cull_angle: 0x3fff,
        min_pixels: 2000,
        fig_room: 0x03,
        m11: 0x02,
    },
    // 0x06
    CharaData {
        offset: [0.0, 20.0, 20.0],
        radius: 55.0,
        height: 140.0,
        min_ratio: 500,
        cull_angle: 0x2710,
        min_pixels: 5000,
        fig_room: 0x03,
        m11: 0x02,
    },
    // 0x07
    CharaData {
        offset: [0.0, 20.0, 20.0],
        radius: 35.0,
        height: 150.0,
        min_ratio: 500,
        cull_angle: 0x2710,
        min_pixels: 5000,
        fig_room: 0x03,
        m11: 0x02,
    },
    // 0x08
    CharaData {
        offset: [0.0, 10.0, 0.0],
        radius: 25.0,
        height: 70.0,
        min_ratio: 500,
        cull_angle: 0x2000,
        min_pixels: 5000,
        fig_room: 0x03,
        m11: 0x02,
    },
    // 0x09
    CharaData {
        offset: [0.0, 10.0, 0.0],
        radius: 25.0,
        height: 70.0,
        min_ratio: 500,
        cull_angle: 0x2000,
        min_pixels: 5000,
        fig_room: 0x03,
        m11: 0x02,
    },
    // 0x0A
    CharaData {
        offset: [0.0, 0.0, 0.0],
        radius: 50.0,
        height: 150.0,
        min_ratio: 550,
        cull_angle: 0x2000,
        min_pixels: 5000,
        fig_room: 0x03,
        m11: 0x0c,
    },
    // 0x0B
    CharaData {
        offset: [0.0, 0.0, 0.0],
        radius: 50.0,
        height: 60.0,
        min_ratio: 400,
        cull_angle: 0x3e80,
        min_pixels: 4000,
        fig_room: 0x03,
        m11: 0x00,
    },
    // 0x0C
    CharaData {
        offset: [0.0, -40.0, 0.0],
        radius: 60.0,
        height: 80.0,
        min_ratio: 300,
        cull_angle: 0x7fff,
        min_pixels: 3000,
        fig_room: 0x03,
        m11: 0x00,
    },
    // 0x0D
    CharaData {
        offset: [0.0, 0.0, 0.0],
        radius: 20.0,
        height: 20.0,
        min_ratio: 400,
        cull_angle: 0x7d00,
        min_pixels: 3000,
        fig_room: 0x03,
        m11: 0x00,
    },
    // 0x0E
    CharaData {
        offset: [0.0, 30.0, 0.0],
        radius: 50.0,
        height: 120.0,
        min_ratio: 200,
        cull_angle: 0x4650,
        min_pixels: 3000,
        fig_room: 0x02,
        m11: 0x02,
    },
    // 0x0F
    CharaData {
        offset: [0.0, 10.0, 0.0],
        radius: 35.0,
        height: 120.0,
        min_ratio: 500,
        cull_angle: 0x2000,
        min_pixels: 5000,
        fig_room: 0x02,
        m11: 0x02,
    },
    // 0x10
    CharaData {
        offset: [0.0, 10.0, 20.0],
        radius: 60.0,
        height: 160.0,
        min_ratio: 500,
        cull_angle: 0x2710,
        min_pixels: 5000,
        fig_room: 0x02,
        m11: 0x02,
    },
    // 0x11
    CharaData {
        offset: [0.0, 10.0, 0.0],
        radius: 50.0,
        height: 140.0,
        min_ratio: 500,
        cull_angle: 0x55f0,
        min_pixels: 5000,
        fig_room: 0x02,
        m11: 0x02,
    },
    // 0x12
    CharaData {
        offset: [0.0, 10.0, 20.0],
        radius: 60.0,
        height: 200.0,
        min_ratio: 500,
        cull_angle: 0x2710,
        min_pixels: 5000,
        fig_room: 0x02,
        m11: 0x02,
    },
    // 0x13
    CharaData {
        offset: [0.0, 90.0, 30.0],
        radius: 60.0,
        height: 120.0,
        min_ratio: 550,
        cull_angle: 0x2000,
        min_pixels: 5000,
        fig_room: 0x02,
        m11: 0x02,
    },
    // 0x14
    CharaData {
        offset: [0.0, 75.0, 30.0],
        radius: 40.0,
        height: 80.0,
        min_ratio: 550,
        cull_angle: 0x2710,
        min_pixels: 5000,
        fig_room: 0x02,
        m11: 0x02,
    },
    // 0x15
    CharaData {
        offset: [0.0, 20.0, 0.0],
        radius: 80.0,
        height: 180.0,
        min_ratio: 550,
        cull_angle: 0x4a38,
        min_pixels: 5000,
        fig_room: 0x02,
        m11: 0x02,
    },
    // 0x16
    CharaData {
        offset: [0.0, 15.0, 0.0],
        radius: 60.0,
        height: 135.0,
        min_ratio: 550,
        cull_angle: 0x32c8,
        min_pixels: 5000,
        fig_room: 0x02,
        m11: 0x02,
    },
    // 0x17
    CharaData {
        offset: [0.0, 15.0, 0.0],
        radius: 50.0,
        height: 155.0,
        min_ratio: 550,
        cull_angle: 0x2710,
        min_pixels: 5000,
        fig_room: 0x02,
        m11: 0x02,
    },
    // 0x18
    CharaData {
        offset: [0.0, 10.0, 0.0],
        radius: 30.0,
        height: 80.0,
        min_ratio: 550,
        cull_angle: 0x32c8,
        min_pixels: 5000,
        fig_room: 0x02,
        m11: 0x02,
    },
    // 0x19
    CharaData {
        offset: [0.0, 5.0, 0.0],
        radius: 30.0,
        height: 80.0,
        min_ratio: 550,
        cull_angle: 0x2710,
        min_pixels: 5000,
        fig_room: 0x02,
        m11: 0x02,
    },
    // 0x1A
    CharaData {
        offset: [0.0, 5.0, 10.0],
        radius: 45.0,
        height: 180.0,
        min_ratio: 550,
        cull_angle: 0x2710,
        min_pixels: 5000,
        fig_room: 0x02,
        m11: 0x02,
    },
    // 0x1B
    CharaData {
        offset: [0.0, 20.0, 0.0],
        radius: 45.0,
        height: 160.0,
        min_ratio: 550,
        cull_angle: 0x5208,
        min_pixels: 5000,
        fig_room: 0x02,
        m11: 0x02,
    },
    // 0x1C
    CharaData {
        offset: [0.0, 10.0, 10.0],
        radius: 45.0,
        height: 130.0,
        min_ratio: 550,
        cull_angle: 0x2af8,
        min_pixels: 5000,
        fig_room: 0x02,
        m11: 0x02,
    },
    // 0x1D
    CharaData {
        offset: [0.0, 80.0, 0.0],
        radius: 35.0,
        height: 80.0,
        min_ratio: 500,
        cull_angle: 0x4650,
        min_pixels: 5000,
        fig_room: 0x02,
        m11: 0x02,
    },
    // 0x1E
    CharaData {
        offset: [0.0, 80.0, 0.0],
        radius: 40.0,
        height: 85.0,
        min_ratio: 550,
        cull_angle: 0x2710,
        min_pixels: 5000,
        fig_room: 0x02,
        m11: 0x02,
    },
    // 0x1F
    CharaData {
        offset: [0.0, 30.0, 10.0],
        radius: 45.0,
        height: 115.0,
        min_ratio: 550,
        cull_angle: 0x55f0,
        min_pixels: 5000,
        fig_room: 0x02,
        m11: 0x02,
    },
    // 0x20
    CharaData {
        offset: [0.0, 10.0, 0.0],
        radius: 45.0,
        height: 150.0,
        min_ratio: 550,
        cull_angle: 0x4268,
        min_pixels: 5000,
        fig_room: 0x02,
        m11: 0x02,
    },
    // 0x21
    CharaData {
        offset: [0.0, 10.0, 10.0],
        radius: 45.0,
        height: 150.0,
        min_ratio: 550,
        cull_angle: 0x2710,
        min_pixels: 5000,
        fig_room: 0x02,
        m11: 0x02,
    },
    // 0x22
    CharaData {
        offset: [0.0, 10.0, 10.0],
        radius: 45.0,
        height: 145.0,
        min_ratio: 550,
        cull_angle: 0x3a98,
        min_pixels: 5000,
        fig_room: 0x02,
        m11: 0x02,
    },
    // 0x23
    CharaData {
        offset: [0.0, 20.0, 10.0],
        radius: 45.0,
        height: 145.0,
        min_ratio: 550,
        cull_angle: 0x3e80,
        min_pixels: 5000,
        fig_room: 0x02,
        m11: 0x02,
    },
    // 0x24
    CharaData {
        offset: [0.0, 20.0, 10.0],
        radius: 45.0,
        height: 145.0,
        min_ratio: 550,
        cull_angle: 0x3e80,
        min_pixels: 5000,
        fig_room: 0x02,
        m11: 0x02,
    },
    // 0x25
    CharaData {
        offset: [0.0, 30.0, 35.0],
        radius: 80.0,
        height: 160.0,
        min_ratio: 550,
        cull_angle: 0x3a98,
        min_pixels: 5000,
        fig_room: 0x02,
        m11: 0x02,
    },
    // 0x26
    CharaData {
        offset: [0.0, 30.0, 30.0],
        radius: 75.0,
        height: 170.0,
        min_ratio: 550,
        cull_angle: 0x2328,
        min_pixels: 5000,
        fig_room: 0x02,
        m11: 0x02,
    },
    // 0x27
    CharaData {
        offset: [0.0, 30.0, 30.0],
        radius: 75.0,
        height: 170.0,
        min_ratio: 550,
        cull_angle: 0x2328,
        min_pixels: 5000,
        fig_room: 0x02,
        m11: 0x02,
    },
    // 0x28
    CharaData {
        offset: [0.0, 30.0, 30.0],
        radius: 75.0,
        height: 170.0,
        min_ratio: 550,
        cull_angle: 0x2328,
        min_pixels: 5000,
        fig_room: 0x02,
        m11: 0x02,
    },
    // 0x29
    CharaData {
        offset: [0.0, 20.0, 0.0],
        radius: 30.0,
        height: 100.0,
        min_ratio: 500,
        cull_angle: 0x2710,
        min_pixels: 4000,
        fig_room: 0x01,
        m11: 0x02,
    },
    // 0x2A
    CharaData {
        offset: [0.0, 30.0, 0.0],
        radius: 50.0,
        height: 160.0,
        min_ratio: 500,
        cull_angle: 0x2ee0,
        min_pixels: 4500,
        fig_room: 0x01,
        m11: 0x02,
    },
    // 0x2B
    CharaData {
        offset: [0.0, 30.0, 0.0],
        radius: 50.0,
        height: 160.0,
        min_ratio: 500,
        cull_angle: 0x2ee0,
        min_pixels: 4500,
        fig_room: 0x01,
        m11: 0x02,
    },
    // 0x2C
    CharaData {
        offset: [0.0, 30.0, 0.0],
        radius: 50.0,
        height: 160.0,
        min_ratio: 500,
        cull_angle: 0x2710,
        min_pixels: 4500,
        fig_room: 0x01,
        m11: 0x02,
    },
    // 0x2D
    CharaData {
        offset: [0.0, 20.0, 10.0],
        radius: 50.0,
        height: 80.0,
        min_ratio: 500,
        cull_angle: 0x2ee0,
        min_pixels: 4500,
        fig_room: 0x01,
        m11: 0x02,
    },
    // 0x2E
    CharaData {
        offset: [0.0, 10.0, 0.0],
        radius: 50.0,
        height: 90.0,
        min_ratio: 500,
        cull_angle: 0x32c8,
        min_pixels: 5000,
        fig_room: 0x01,
        m11: 0x02,
    },
    // 0x2F
    CharaData {
        offset: [0.0, 20.0, 10.0],
        radius: 50.0,
        height: 80.0,
        min_ratio: 500,
        cull_angle: 0x2ee0,
        min_pixels: 4500,
        fig_room: 0x01,
        m11: 0x02,
    },
    // 0x30
    CharaData {
        offset: [0.0, 10.0, 0.0],
        radius: 55.0,
        height: 120.0,
        min_ratio: 550,
        cull_angle: 0x55f0,
        min_pixels: 5000,
        fig_room: 0x01,
        m11: 0x02,
    },
    // 0x31
    CharaData {
        offset: [0.0, 10.0, 0.0],
        radius: 55.0,
        height: 120.0,
        min_ratio: 550,
        cull_angle: 0x55f0,
        min_pixels: 5000,
        fig_room: 0x01,
        m11: 0x02,
    },
    // 0x32
    CharaData {
        offset: [0.0, 10.0, 0.0],
        radius: 55.0,
        height: 120.0,
        min_ratio: 550,
        cull_angle: 0x55f0,
        min_pixels: 5000,
        fig_room: 0x01,
        m11: 0x02,
    },
    // 0x33
    CharaData {
        offset: [0.0, 10.0, 0.0],
        radius: 55.0,
        height: 120.0,
        min_ratio: 550,
        cull_angle: 0x55f0,
        min_pixels: 5000,
        fig_room: 0x01,
        m11: 0x02,
    },
    // 0x34
    CharaData {
        offset: [0.0, 0.0, 0.0],
        radius: 80.0,
        height: 120.0,
        min_ratio: 200,
        cull_angle: 0x7fff,
        min_pixels: 2000,
        fig_room: 0x01,
        m11: 0x00,
    },
    // 0x35
    CharaData {
        offset: [0.0, 30.0, 10.0],
        radius: 80.0,
        height: 150.0,
        min_ratio: 500,
        cull_angle: 0x2710,
        min_pixels: 3000,
        fig_room: 0x01,
        m11: 0x02,
    },
    // 0x36
    CharaData {
        offset: [0.0, 20.0, 20.0],
        radius: 40.0,
        height: 120.0,
        min_ratio: 500,
        cull_angle: 0x2000,
        min_pixels: 5000,
        fig_room: 0x01,
        m11: 0x02,
    },
    // 0x37
    CharaData {
        offset: [0.0, 20.0, 30.0],
        radius: 40.0,
        height: 90.0,
        min_ratio: 500,
        cull_angle: 0x2000,
        min_pixels: 5000,
        fig_room: 0x01,
        m11: 0x02,
    },
    // 0x38
    CharaData {
        offset: [0.0, 100.0, 40.0],
        radius: 50.0,
        height: 60.0,
        min_ratio: 500,
        cull_angle: 0x4268,
        min_pixels: 5000,
        fig_room: 0x01,
        m11: 0x02,
    },
    // 0x39
    CharaData {
        offset: [0.0, 10.0, 0.0],
        radius: 65.0,
        height: 200.0,
        min_ratio: 550,
        cull_angle: 0x2af8,
        min_pixels: 5000,
        fig_room: 0x01,
        m11: 0x02,
    },
    // 0x3A
    CharaData {
        offset: [0.0, 50.0, 0.0],
        radius: 300.0,
        height: 400.0,
        min_ratio: 100,
        cull_angle: 0x7fff,
        min_pixels: 4000,
        fig_room: 0x01,
        m11: 0x00,
    },
    // 0x3B
    CharaData {
        offset: [0.0, 0.0, 0.0],
        radius: 25.0,
        height: 25.0,
        min_ratio: 550,
        cull_angle: 0x2af8,
        min_pixels: 5000,
        fig_room: 0x01,
        m11: 0x00,
    },
    // 0x3C
    CharaData {
        offset: [0.0, 0.0, 0.0],
        radius: 50.0,
        height: 150.0,
        min_ratio: 550,
        cull_angle: 0x2000,
        min_pixels: 5000,
        fig_room: 0x01,
        m11: 0x0c,
    },
    // 0x3D
    CharaData {
        offset: [0.0, 0.0, 0.0],
        radius: 50.0,
        height: 150.0,
        min_ratio: 550,
        cull_angle: 0x2000,
        min_pixels: 5000,
        fig_room: 0x01,
        m11: 0x0c,
    },
    // 0x3E
    CharaData {
        offset: [0.0, 10.0, 0.0],
        radius: 35.0,
        height: 120.0,
        min_ratio: 550,
        cull_angle: 0x2af8,
        min_pixels: 5000,
        fig_room: 0x01,
        m11: 0x02,
    },
    // 0x3F
    CharaData {
        offset: [0.0, 0.0, 0.0],
        radius: 50.0,
        height: 150.0,
        min_ratio: 550,
        cull_angle: 0x2000,
        min_pixels: 5000,
        fig_room: 0x01,
        m11: 0x0e,
    },
    // 0x40
    CharaData {
        offset: [0.0, 0.0, 0.0],
        radius: 50.0,
        height: 150.0,
        min_ratio: 550,
        cull_angle: 0x2000,
        min_pixels: 5000,
        fig_room: 0x01,
        m11: 0x0c,
    },
    // 0x41
    CharaData {
        offset: [0.0, 20.0, 0.0],
        radius: 30.0,
        height: 100.0,
        min_ratio: 500,
        cull_angle: 0x2710,
        min_pixels: 4000,
        fig_room: 0x06,
        m11: 0x02,
    },
    // 0x42
    CharaData {
        offset: [0.0, 10.0, 0.0],
        radius: 30.0,
        height: 60.0,
        min_ratio: 500,
        cull_angle: 0x2000,
        min_pixels: 2000,
        fig_room: 0x06,
        m11: 0x02,
    },
    // 0x43
    CharaData {
        offset: [0.0, 20.0, 0.0],
        radius: 60.0,
        height: 200.0,
        min_ratio: 500,
        cull_angle: 0x2000,
        min_pixels: 5000,
        fig_room: 0x06,
        m11: 0x02,
    },
    // 0x44
    CharaData {
        offset: [0.0, 20.0, 0.0],
        radius: 50.0,
        height: 150.0,
        min_ratio: 500,
        cull_angle: 0x2ee0,
        min_pixels: 5000,
        fig_room: 0x06,
        m11: 0x02,
    },
    // 0x45
    CharaData {
        offset: [0.0, 20.0, 25.0],
        radius: 70.0,
        height: 170.0,
        min_ratio: 500,
        cull_angle: 0x2000,
        min_pixels: 5000,
        fig_room: 0x06,
        m11: 0x02,
    },
    // 0x46
    CharaData {
        offset: [0.0, 20.0, 25.0],
        radius: 70.0,
        height: 170.0,
        min_ratio: 500,
        cull_angle: 0x2000,
        min_pixels: 5000,
        fig_room: 0x06,
        m11: 0x02,
    },
    // 0x47
    CharaData {
        offset: [0.0, 20.0, 25.0],
        radius: 70.0,
        height: 170.0,
        min_ratio: 500,
        cull_angle: 0x2000,
        min_pixels: 5000,
        fig_room: 0x06,
        m11: 0x02,
    },
    // 0x48
    CharaData {
        offset: [0.0, 20.0, 25.0],
        radius: 70.0,
        height: 170.0,
        min_ratio: 500,
        cull_angle: 0x2000,
        min_pixels: 5000,
        fig_room: 0x06,
        m11: 0x02,
    },
    // 0x49
    CharaData {
        offset: [0.0, 20.0, 25.0],
        radius: 70.0,
        height: 170.0,
        min_ratio: 500,
        cull_angle: 0x2000,
        min_pixels: 5000,
        fig_room: 0x06,
        m11: 0x02,
    },
    // 0x4A
    CharaData {
        offset: [0.0, 20.0, 25.0],
        radius: 70.0,
        height: 170.0,
        min_ratio: 500,
        cull_angle: 0x2000,
        min_pixels: 5000,
        fig_room: 0x06,
        m11: 0x02,
    },
    // 0x4B
    CharaData {
        offset: [0.0, 20.0, 10.0],
        radius: 50.0,
        height: 170.0,
        min_ratio: 500,
        cull_angle: 0x2000,
        min_pixels: 5000,
        fig_room: 0x06,
        m11: 0x02,
    },
    // 0x4C
    CharaData {
        offset: [0.0, 20.0, 10.0],
        radius: 50.0,
        height: 170.0,
        min_ratio: 500,
        cull_angle: 0x2000,
        min_pixels: 5000,
        fig_room: 0x06,
        m11: 0x02,
    },
    // 0x4D
    CharaData {
        offset: [0.0, 90.0, 30.0],
        radius: 60.0,
        height: 90.0,
        min_ratio: 500,
        cull_angle: 0x6000,
        min_pixels: 5000,
        fig_room: 0x06,
        m11: 0x02,
    },
    // 0x4E
    CharaData {
        offset: [0.0, 20.0, 25.0],
        radius: 70.0,
        height: 170.0,
        min_ratio: 500,
        cull_angle: 0x2000,
        min_pixels: 5000,
        fig_room: 0x06,
        m11: 0x02,
    },
    // 0x4F
    CharaData {
        offset: [0.0, 30.0, 0.0],
        radius: 50.0,
        height: 150.0,
        min_ratio: 500,
        cull_angle: 0x2000,
        min_pixels: 5000,
        fig_room: 0x06,
        m11: 0x02,
    },
    // 0x50
    CharaData {
        offset: [0.0, -300.0, 0.0],
        radius: 870.0,
        height: 700.0,
        min_ratio: 100,
        cull_angle: 0x6000,
        min_pixels: 3000,
        fig_room: 0x06,
        m11: 0x00,
    },
    // 0x51
    CharaData {
        offset: [0.0, 0.0, 0.0],
        radius: 1000.0,
        height: 800.0,
        min_ratio: 100,
        cull_angle: 0x4650,
        min_pixels: 1000,
        fig_room: 0x06,
        m11: 0x00,
    },
    // 0x52
    CharaData {
        offset: [0.0, 0.0, 0.0],
        radius: 50.0,
        height: 150.0,
        min_ratio: 550,
        cull_angle: 0x2000,
        min_pixels: 5000,
        fig_room: 0x06,
        m11: 0x0e,
    },
    // 0x53
    CharaData {
        offset: [0.0, 0.0, 0.0],
        radius: 40.0,
        height: 55.0,
        min_ratio: 500,
        cull_angle: 0x2af8,
        min_pixels: 4000,
        fig_room: 0x07,
        m11: 0x00,
    },
    // 0x54
    CharaData {
        offset: [0.0, 15.0, 0.0],
        radius: 40.0,
        height: 100.0,
        min_ratio: 500,
        cull_angle: 0x2af8,
        min_pixels: 4000,
        fig_room: 0x07,
        m11: 0x00,
    },
    // 0x55
    CharaData {
        offset: [0.0, 15.0, 0.0],
        radius: 40.0,
        height: 100.0,
        min_ratio: 500,
        cull_angle: 0x2af8,
        min_pixels: 4000,
        fig_room: 0x07,
        m11: 0x00,
    },
    // 0x56
    CharaData {
        offset: [0.0, 15.0, 0.0],
        radius: 40.0,
        height: 100.0,
        min_ratio: 500,
        cull_angle: 0x2af8,
        min_pixels: 4000,
        fig_room: 0x07,
        m11: 0x00,
    },
    // 0x57
    CharaData {
        offset: [0.0, 15.0, 0.0],
        radius: 40.0,
        height: 100.0,
        min_ratio: 500,
        cull_angle: 0x2af8,
        min_pixels: 4000,
        fig_room: 0x07,
        m11: 0x00,
    },
    // 0x58
    CharaData {
        offset: [0.0, 15.0, 0.0],
        radius: 40.0,
        height: 100.0,
        min_ratio: 500,
        cull_angle: 0x2af8,
        min_pixels: 4000,
        fig_room: 0x07,
        m11: 0x00,
    },
    // 0x59
    CharaData {
        offset: [0.0, 15.0, 0.0],
        radius: 40.0,
        height: 100.0,
        min_ratio: 500,
        cull_angle: 0x2af8,
        min_pixels: 4000,
        fig_room: 0x07,
        m11: 0x00,
    },
    // 0x5A
    CharaData {
        offset: [0.0, 15.0, 0.0],
        radius: 40.0,
        height: 100.0,
        min_ratio: 500,
        cull_angle: 0x2af8,
        min_pixels: 4000,
        fig_room: 0x07,
        m11: 0x00,
    },
    // 0x5B
    CharaData {
        offset: [0.0, 15.0, 0.0],
        radius: 40.0,
        height: 100.0,
        min_ratio: 500,
        cull_angle: 0x2af8,
        min_pixels: 4000,
        fig_room: 0x07,
        m11: 0x00,
    },
    // 0x5C
    CharaData {
        offset: [0.0, 15.0, 0.0],
        radius: 40.0,
        height: 100.0,
        min_ratio: 500,
        cull_angle: 0x2af8,
        min_pixels: 4000,
        fig_room: 0x07,
        m11: 0x00,
    },
    // 0x5D
    CharaData {
        offset: [0.0, 700.0, 800.0],
        radius: 200.0,
        height: 200.0,
        min_ratio: 300,
        cull_angle: 0x7fff,
        min_pixels: 3000,
        fig_room: 0x07,
        m11: 0x00,
    },
    // 0x5E
    CharaData {
        offset: [0.0, 85.0, 0.0],
        radius: 70.0,
        height: 80.0,
        min_ratio: 550,
        cull_angle: 0x2000,
        min_pixels: 5000,
        fig_room: 0x07,
        m11: 0x02,
    },
    // 0x5F
    CharaData {
        offset: [0.0, 0.0, 0.0],
        radius: 50.0,
        height: 150.0,
        min_ratio: 550,
        cull_angle: 0x2000,
        min_pixels: 5000,
        fig_room: 0x07,
        m11: 0x06,
    },
    // 0x60
    CharaData {
        offset: [0.0, 0.0, 0.0],
        radius: 50.0,
        height: 150.0,
        min_ratio: 550,
        cull_angle: 0x2000,
        min_pixels: 5000,
        fig_room: 0x07,
        m11: 0x0e,
    },
    // 0x61
    CharaData {
        offset: [0.0, 20.0, 80.0],
        radius: 60.0,
        height: 100.0,
        min_ratio: 500,
        cull_angle: 0x2000,
        min_pixels: 5000,
        fig_room: 0x05,
        m11: 0x00,
    },
    // 0x62
    CharaData {
        offset: [0.0, 10.0, 20.0],
        radius: 60.0,
        height: 100.0,
        min_ratio: 400,
        cull_angle: 0x3e80,
        min_pixels: 4000,
        fig_room: 0x05,
        m11: 0x00,
    },
    // 0x63
    CharaData {
        offset: [0.0, 20.0, 20.0],
        radius: 40.0,
        height: 50.0,
        min_ratio: 500,
        cull_angle: 0x2000,
        min_pixels: 3000,
        fig_room: 0x05,
        m11: 0x00,
    },
    // 0x64
    CharaData {
        offset: [0.0, 0.0, 0.0],
        radius: 60.0,
        height: 60.0,
        min_ratio: 200,
        cull_angle: 0x7530,
        min_pixels: 3000,
        fig_room: 0x05,
        m11: 0x00,
    },
    // 0x65
    CharaData {
        offset: [0.0, -25.0, 30.0],
        radius: 40.0,
        height: 60.0,
        min_ratio: 300,
        cull_angle: 0x4e20,
        min_pixels: 3000,
        fig_room: 0x05,
        m11: 0x00,
    },
    // 0x66
    CharaData {
        offset: [0.0, -40.0, 50.0],
        radius: 50.0,
        height: 80.0,
        min_ratio: 300,
        cull_angle: 0x2ee0,
        min_pixels: 3000,
        fig_room: 0x05,
        m11: 0x00,
    },
    // 0x67
    CharaData {
        offset: [0.0, -50.0, 0.0],
        radius: 50.0,
        height: 100.0,
        min_ratio: 200,
        cull_angle: 0x4000,
        min_pixels: 3000,
        fig_room: 0x05,
        m11: 0x00,
    },
    // 0x68
    CharaData {
        offset: [0.0, -80.0, 0.0],
        radius: 80.0,
        height: 120.0,
        min_ratio: 300,
        cull_angle: 0x7fff,
        min_pixels: 2000,
        fig_room: 0x05,
        m11: 0x00,
    },
    // 0x69
    CharaData {
        offset: [0.0, 30.0, 0.0],
        radius: 120.0,
        height: 50.0,
        min_ratio: 100,
        cull_angle: 0x2af8,
        min_pixels: 2000,
        fig_room: 0x05,
        m11: 0x00,
    },
    // 0x6A
    CharaData {
        offset: [0.0, -30.0, 0.0],
        radius: 30.0,
        height: 60.0,
        min_ratio: 300,
        cull_angle: 0x7fff,
        min_pixels: 3000,
        fig_room: 0x05,
        m11: 0x00,
    },
    // 0x6B
    CharaData {
        offset: [0.0, -20.0, 0.0],
        radius: 60.0,
        height: 100.0,
        min_ratio: 400,
        cull_angle: 0x7fff,
        min_pixels: 3000,
        fig_room: 0x05,
        m11: 0x00,
    },
    // 0x6C
    CharaData {
        offset: [10.0, -50.0, 0.0],
        radius: 60.0,
        height: 100.0,
        min_ratio: 500,
        cull_angle: 0x2af8,
        min_pixels: 3000,
        fig_room: 0x05,
        m11: 0x00,
    },
    // 0x6D
    CharaData {
        offset: [0.0, 10.0, 10.0],
        radius: 60.0,
        height: 130.0,
        min_ratio: 500,
        cull_angle: 0x2af8,
        min_pixels: 4000,
        fig_room: 0x05,
        m11: 0x00,
    },
    // 0x6E
    CharaData {
        offset: [0.0, 20.0, 20.0],
        radius: 130.0,
        height: 300.0,
        min_ratio: 500,
        cull_angle: 0x2ee0,
        min_pixels: 5000,
        fig_room: 0x05,
        m11: 0x00,
    },
    // 0x6F
    CharaData {
        offset: [0.0, 20.0, 20.0],
        radius: 80.0,
        height: 160.0,
        min_ratio: 400,
        cull_angle: 0x2ee0,
        min_pixels: 3000,
        fig_room: 0x05,
        m11: 0x00,
    },
    // 0x70
    CharaData {
        offset: [0.0, 20.0, 50.0],
        radius: 70.0,
        height: 170.0,
        min_ratio: 300,
        cull_angle: 0x4650,
        min_pixels: 3000,
        fig_room: 0x05,
        m11: 0x00,
    },
    // 0x71
    CharaData {
        offset: [0.0, 20.0, 50.0],
        radius: 70.0,
        height: 170.0,
        min_ratio: 200,
        cull_angle: 0x4650,
        min_pixels: 2000,
        fig_room: 0x05,
        m11: 0x00,
    },
    // 0x72
    CharaData {
        offset: [0.0, 0.0, 0.0],
        radius: 400.0,
        height: 400.0,
        min_ratio: 200,
        cull_angle: 0x7530,
        min_pixels: 3000,
        fig_room: 0x05,
        m11: 0x00,
    },
    // 0x73
    CharaData {
        offset: [0.0, 0.0, 0.0],
        radius: 200.0,
        height: 170.0,
        min_ratio: 300,
        cull_angle: 0x7fbc,
        min_pixels: 2000,
        fig_room: 0x05,
        m11: 0x00,
    },
    // 0x74
    CharaData {
        offset: [0.0, 20.0, 50.0],
        radius: 100.0,
        height: 140.0,
        min_ratio: 300,
        cull_angle: 0x2af8,
        min_pixels: 2000,
        fig_room: 0x04,
        m11: 0x00,
    },
    // 0x75
    CharaData {
        offset: [0.0, -120.0, 0.0],
        radius: 100.0,
        height: 100.0,
        min_ratio: 300,
        cull_angle: 0x2af8,
        min_pixels: 2000,
        fig_room: 0x04,
        m11: 0x00,
    },
    // 0x76
    CharaData {
        offset: [0.0, 10.0, 10.0],
        radius: 100.0,
        height: 200.0,
        min_ratio: 300,
        cull_angle: 0x2af8,
        min_pixels: 2000,
        fig_room: 0x04,
        m11: 0x00,
    },
    // 0x77
    CharaData {
        offset: [0.0, 10.0, 10.0],
        radius: 100.0,
        height: 200.0,
        min_ratio: 300,
        cull_angle: 0x2af8,
        min_pixels: 2000,
        fig_room: 0x04,
        m11: 0x00,
    },
    // 0x78
    CharaData {
        offset: [0.0, 10.0, 10.0],
        radius: 100.0,
        height: 200.0,
        min_ratio: 300,
        cull_angle: 0x2af8,
        min_pixels: 2000,
        fig_room: 0x04,
        m11: 0x00,
    },
    // 0x79
    CharaData {
        offset: [0.0, 10.0, 10.0],
        radius: 100.0,
        height: 250.0,
        min_ratio: 300,
        cull_angle: 0x2af8,
        min_pixels: 2000,
        fig_room: 0x04,
        m11: 0x00,
    },
    // 0x7A
    CharaData {
        offset: [0.0, 20.0, 10.0],
        radius: 70.0,
        height: 210.0,
        min_ratio: 300,
        cull_angle: 0x2af8,
        min_pixels: 3000,
        fig_room: 0x04,
        m11: 0x00,
    },
    // 0x7B
    CharaData {
        offset: [0.0, 20.0, 30.0],
        radius: 70.0,
        height: 150.0,
        min_ratio: 300,
        cull_angle: 0x2af8,
        min_pixels: 3000,
        fig_room: 0x04,
        m11: 0x00,
    },
    // 0x7C
    CharaData {
        offset: [0.0, 20.0, 30.0],
        radius: 70.0,
        height: 150.0,
        min_ratio: 300,
        cull_angle: 0x2af8,
        min_pixels: 3000,
        fig_room: 0x04,
        m11: 0x00,
    },
    // 0x7D
    CharaData {
        offset: [0.0, 200.0, 0.0],
        radius: 1000.0,
        height: 1200.0,
        min_ratio: 100,
        cull_angle: 0x6d60,
        min_pixels: 1000,
        fig_room: 0x04,
        m11: 0x00,
    },
    // 0x7E
    CharaData {
        offset: [0.0, -200.0, 0.0],
        radius: 360.0,
        height: 450.0,
        min_ratio: 500,
        cull_angle: 0x4e20,
        min_pixels: 500,
        fig_room: 0x04,
        m11: 0x00,
    },
    // 0x7F
    CharaData {
        offset: [0.0, 0.0, 0.0],
        radius: 400.0,
        height: 380.0,
        min_ratio: 300,
        cull_angle: 0x7d00,
        min_pixels: 2000,
        fig_room: 0x04,
        m11: 0x00,
    },
    // 0x80
    CharaData {
        offset: [0.0, 0.0, 50.0],
        radius: 350.0,
        height: 400.0,
        min_ratio: 100,
        cull_angle: 0x4650,
        min_pixels: 1000,
        fig_room: 0x04,
        m11: 0x00,
    },
    // 0x81
    CharaData {
        offset: [0.0, 200.0, 50.0],
        radius: 300.0,
        height: 550.0,
        min_ratio: 200,
        cull_angle: 0x3e80,
        min_pixels: 2000,
        fig_room: 0x04,
        m11: 0x00,
    },
    // 0x82
    CharaData {
        offset: [0.0, 200.0, 50.0],
        radius: 350.0,
        height: 400.0,
        min_ratio: 200,
        cull_angle: 0x3e80,
        min_pixels: 4000,
        fig_room: 0x04,
        m11: 0x00,
    },
    // 0x83
    CharaData {
        offset: [0.0, 100.0, 30.0],
        radius: 500.0,
        height: 300.0,
        min_ratio: 200,
        cull_angle: 0x3e80,
        min_pixels: 2000,
        fig_room: 0x04,
        m11: 0x00,
    },
    // 0x84
    CharaData {
        offset: [0.0, 400.0, 0.0],
        radius: 800.0,
        height: 800.0,
        min_ratio: 200,
        cull_angle: 0x4650,
        min_pixels: 1500,
        fig_room: 0x04,
        m11: 0x00,
    },
    // 0x85
    CharaData {
        offset: [0.0, 10.0, 0.0],
        radius: 130.0,
        height: 280.0,
        min_ratio: 200,
        cull_angle: 0x4650,
        min_pixels: 1200,
        fig_room: 0x04,
        m11: 0x0a,
    },
];

/// `dSnap_GetFigRoomId` (`d_snap.cpp:1450-1455`).
pub fn fig_room_id(table_index: u32) -> u8 {
    CHARA_DATA
        .get(table_index as usize)
        .map_or(0xFF, |row| row.fig_room)
}

/// `dSnap_PhotoIndex2TableIndex` (`d_snap.cpp:1443-1447`). Retail asserts the index is in
/// range; the caller guards with the same bounds (`JudgeFigure`, `:2122-2128`).
pub fn photo_index_to_table_index(photo_index: u32) -> Option<usize> {
    let index = photo_index.checked_sub(PHOTO_INDEX_BASE)?;
    (index < CHARA_DATA.len() as u32).then_some(index as usize)
}

/// `dSnap_Obj` (`d_snap.h:219-268`). `field_0x1a` is dropped: `SetInf` stores its `r6` there
/// (`:1709`) and nothing reads it. `field_0x19` is always `CYL_SEGMENTS` at the single call
/// site, so the segment count is that constant rather than a field.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SnapObj {
    pub center: Vec3,
    pub radius: f32,
    pub height: f32,
    /// `mActorPID`: the actor's `fpc_ProcID`, [`ERROR_PROC_ID`] when none. Retail's readers are
    /// its own `GetActor()` stub (`d_snap.h:244`) and the demo-only self-assignment at
    /// `d_snap.cpp:2110`, so the port keeps it as the row's provenance for
    /// [`SnapPacket::registered`] rather than as judge input.
    pub actor_pid: u32,
    pub photo_no: u32,
    /// `field_0x1b & 1`: the sphere proxy `SetGeoSph` selects (`:1698`).
    sphere: bool,
    pub cull_angle: i16,
    /// `mAngleY`: the proxy's own yaw, the angle `ChkCamCull` compares the eye bearing to.
    pub angle_y: i16,
    pub captured_pixels: i32,
    pub captured_ratio: f32,
    pub min_x: i16,
    pub min_y: i16,
    pub max_x: i16,
    pub max_y: i16,
}

/// `dSnap_Obj::dSnap_Obj` (`d_snap.cpp:1458-1472`).
impl Default for SnapObj {
    fn default() -> Self {
        Self {
            center: Vec3::ZERO,
            radius: 0.0,
            height: 0.0,
            actor_pid: ERROR_PROC_ID,
            photo_no: DSNAP_TYPE_NONE,
            sphere: false,
            cull_angle: CULL_ANGLE_NONE,
            angle_y: 0,
            captured_pixels: 0,
            captured_ratio: 0.0,
            min_x: 0,
            min_y: 0,
            max_x: 0,
            max_y: 0,
        }
    }
}

impl SnapObj {
    /// `dSnap_Obj::SetGeo` (`d_snap.cpp:1688-1694`).
    pub fn set_geo(&mut self, center: Vec3, radius: f32, height: f32, angle_y: i16) {
        self.sphere = false;
        self.center = center;
        self.radius = radius;
        self.height = height;
        self.angle_y = angle_y;
    }

    /// `dSnap_Obj::SetGeoSph` (`d_snap.cpp:1697-1703`). Reachable in retail from the rain
    /// snapshot (`d_kankyo_rain.cpp:3242`: `SetGeoSph(pos, 8000.0f)`).
    pub fn set_geo_sphere(&mut self, center: Vec3, radius: f32) {
        self.sphere = true;
        self.center = center;
        self.radius = radius;
        self.height = 0.0;
        self.angle_y = 0;
    }

    /// `dSnap_Obj::SetInf` (`d_snap.cpp:1706-1712`), less the unread `field_0x1a`.
    pub fn set_inf(&mut self, photo_no: u32, actor_pid: u32, cull_angle: i16) {
        self.photo_no = photo_no;
        self.actor_pid = actor_pid;
        self.cull_angle = cull_angle;
    }

    /// `dSnap_Obj::SetAreaClear` (`d_snap.cpp:1715-1720`). The box the writes below track is
    /// read back only by `GetMinX()` and its siblings (`d_snap.h:248-251`), whose only caller in
    /// retail is `DebugArea2D` - an inline empty stub (`:243`) - so the port keeps it as the
    /// capture's recorded extent, exactly as the reference stores it.
    pub fn set_area_clear(&mut self) {
        self.min_x = 0x800;
        self.min_y = 0x800;
        self.max_x = -0x800;
        self.max_y = -0x800;
    }

    /// `dSnap_Obj::SetArea` (`d_snap.cpp:1723-1736`).
    pub fn set_area(&mut self, x: i16, y: i16) {
        if self.min_x > x {
            self.min_x = x;
        }
        if self.min_y > y {
            self.min_y = y;
        }
        if self.max_x < x {
            self.max_x = x;
        }
        if self.max_y < y {
            self.max_y = y;
        }
    }

    /// `dSnap_Obj::ChkPhoto` (`d_snap.cpp:1739-1741`).
    pub fn chk_photo(&self, photo_no: u32) -> bool {
        self.photo_no == photo_no
    }

    /// `dSnap_Obj::ChkSuccess` (`d_snap.cpp:1744-1749`).
    pub fn chk_success(&self, min_pixels: i32, min_ratio: f32) -> bool {
        self.captured_pixels > min_pixels && self.captured_ratio > min_ratio
    }

    /// `dSnap_Obj::ChkCamCull` (`d_snap.cpp:1752-1763`): a cylinder whose bearing from the
    /// camera's eye is more than `mCullAngle` off its own yaw was photographed from behind.
    /// Retail dereferences the player camera unconditionally; without a camera the port has no
    /// eye to measure from and reports no cull.
    pub fn chk_cam_cull(&self, camera: Option<&Camera>) -> bool {
        if self.sphere || self.cull_angle == CULL_ANGLE_NONE {
            return false;
        }
        let Some(eye) = camera.and_then(Camera::view).map(|view| view.lookat.eye) else {
            return false;
        };
        let delta = eye - self.center;
        let angle = to_s16(delta.x.atan2(delta.z));
        distance_angle_s(angle, self.angle_y) > i32::from(self.cull_angle)
    }

    /// `dSnap_Obj::CalcArea2D` (`d_snap.cpp:1475-1619`): the proxy's screen area, the sum of
    /// the positive halves of the 2D cross products of its projected triangles
    /// (`cM3d_VectorProduct2d`, `c_m3d.h:150-152`). The proxy's model matrix is
    /// `transS(center); scaleM(...); YrotM(angleY)` on `mDoMtx_stack_c`, i.e. rotate, then
    /// scale, then translate, and `mDoLib_project` needs the frame's projection-view matrix
    /// and viewport, which the port's camera publishes (`Camera::project`, `camera.rs:318-323`).
    pub fn calc_area_2d(&self, camera: Option<&Camera>) -> f32 {
        let mut area = 0.0_f32;
        if !self.sphere {
            let model = Mat4::from_translation(self.center)
                * Mat4::from_scale(Vec3::new(self.radius, self.height, self.radius))
                * Mat4::from_rotation_y(from_s16(self.angle_y));
            let at = |local: Vec3| project_point(camera, model.transform_point3(local));
            let origin = at(Vec3::ZERO);
            let top = at(Vec3::Y);
            for i in 0..CYL_SEGMENTS {
                let v = CYL_VERTS[i];
                let next = CYL_VERTS[i + 1];
                let bottom_i = at(Vec3::new(v[0], 0.0, v[2]));
                let top_i = at(Vec3::new(v[0], 1.0, v[2]));
                let top_next = at(Vec3::new(next[0], 1.0, next[2]));
                let bottom_next = at(Vec3::new(next[0], 0.0, next[2]));
                area += positive_triangle(bottom_i, top_i, top_next);
                area += positive_triangle(bottom_i, top_next, bottom_next);
                area += positive_triangle(top, top_next, top_i);
                area += positive_triangle(origin, bottom_i, bottom_next);

                let top_mirror = at(Vec3::new(-v[0], 1.0, v[2]));
                let bottom_mirror = at(Vec3::new(-v[0], 0.0, v[2]));
                let bottom_mirror_next = at(Vec3::new(-next[0], 0.0, next[2]));
                let top_mirror_next = at(Vec3::new(-next[0], 1.0, next[2]));
                area += positive_triangle(top_mirror, bottom_mirror, bottom_mirror_next);
                area += positive_triangle(top_mirror, bottom_mirror_next, top_mirror_next);
                area += positive_triangle(top, top_mirror, top_mirror_next);
                area += positive_triangle(origin, bottom_mirror_next, bottom_mirror);
            }
        } else {
            let model =
                Mat4::from_translation(self.center) * Mat4::from_scale(Vec3::splat(self.radius));
            for tri in TRI_INDICES {
                let a = project_point(
                    camera,
                    model.transform_point3(Vec3::from(SPHERE_VERTS[tri[0]])),
                );
                let b = project_point(
                    camera,
                    model.transform_point3(Vec3::from(SPHERE_VERTS[tri[1]])),
                );
                let c = project_point(
                    camera,
                    model.transform_point3(Vec3::from(SPHERE_VERTS[tri[2]])),
                );
                area += positive_triangle(a, b, c);
            }
        }
        area
    }
}

/// `mDoLib_project__FP3VecP3Vec` (`m_Do_lib.cpp:51-58`) with no view: the no-view arm writes
/// a zero vector, which is what the port's absent camera produces.
fn project_point(camera: Option<&Camera>, src: Vec3) -> Vec3 {
    camera
        .and_then(|camera| camera.project(src))
        .unwrap_or(Vec3::ZERO)
}

/// `cLib_distanceAngleS` (`c_lib.cpp:363-366`): `abs((x - y) as s16)`.
fn distance_angle_s(x: i16, y: i16) -> i32 {
    i32::from(x.wrapping_sub(y)).abs()
}

/// `0.5f * cM3d_VectorProduct2d(a, b, c)` when the cross product is positive (`d_snap.cpp`'s
/// `if (f0 > 0.0f) f28 += f0;` in `CalcArea2D`).
fn positive_triangle(a: Vec3, b: Vec3, c: Vec3) -> f32 {
    let cross = (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
    let half = 0.5 * cross;
    if half > 0.0 { half } else { 0.0 }
}

/// `dSnap_RegistObjElm` (`d_snap.h:272-283`): a registry row is the proxy plus the squared
/// camera-player distance `Regist` ranks it by.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SnapTable {
    pub obj: SnapObj,
    pub dist2: f32,
}

impl Default for SnapTable {
    fn default() -> Self {
        Self {
            obj: SnapObj::default(),
            dist2: f32::INFINITY,
        }
    }
}

/// The snapshot packet, `dSnap_packet` (`d_snap.h:285-321`), as a world resource. The free
/// functions are its methods here: `dSnap_Create` is [`SnapPacket::default`],
/// `dSnap_ReleaseShutter` is [`SnapPacket::release_shutter`], `dSnap_Execute` is
/// [`SnapPacket::execute`], `dSnap_GetResult`/`dSnap_GetResultDetail` are
/// [`SnapPacket::result`]/[`SnapPacket::result_detail`].
#[derive(Resource, Debug, PartialEq)]
pub struct SnapPacket {
    flag: u32,
    table: [SnapTable; TABLE_LEN],
    /// `field_0x14`: how many rows `Regist` has filled this frame.
    count: usize,
    /// `mPhotoNo`: the type the running judge is scoring (`JudgeFigure`, `:2113-2142`).
    photo_no: u32,
    /// `mResult`: `dSnap_GetResult` (`:2198-2200`).
    result: i32,
    /// `mResultDetail`: bit 1 from [`SnapObj::chk_cam_cull`], bit 2 from a low ratio
    /// (`JudgeFigure`, `:2134-2140`).
    result_detail: u8,
}

impl Default for SnapPacket {
    /// `dSnap_packet::Create` (`d_snap.cpp:1774-1778`).
    fn default() -> Self {
        Self {
            flag: 0,
            table: std::array::from_fn(|_| SnapTable::default()),
            count: 0,
            photo_no: DSNAP_TYPE_NONE,
            result: 0,
            result_detail: 0,
        }
    }
}

impl SnapPacket {
    /// `dSnap_packet::ReleaseShutter` (`d_snap.cpp:1781-1783`).
    pub fn release_shutter(&mut self) {
        self.flag |= 1;
    }

    /// `dSnap_packet::ChkReleaseShutter` (`d_snap.cpp:1786-1788`).
    pub fn chk_release_shutter(&self) -> u32 {
        self.flag & 1
    }

    /// `dSnap_packet::Execute` (`d_snap.cpp:1791-1799`): a released shutter empties the
    /// registry every frame, so each frame's draws re-fill it from row 0. `Init` is empty in
    /// retail, so only the ranking distance is reset before the count.
    pub fn execute(&mut self) {
        if self.chk_release_shutter() == 0 {
            return;
        }
        for slot in &mut self.table {
            slot.dist2 = f32::INFINITY;
        }
        self.count = 0;
    }

    /// `dSnap_packet::Regist` (`d_snap.cpp:1802-1835`), with the player position passed in for
    /// `dComIfGp_getPlayer(0)->current.pos.abs2(...)`. The first 0x3F registrations fill rows in
    /// order; past that, a registration is ranked by its squared distance to the player against
    /// the most distant figure row (`GetPhoto() >= 0x48`), and the nearest wins.
    pub fn regist(&mut self, obj: SnapObj, player_pos: Vec3) -> i32 {
        if self.chk_release_shutter() == 0 {
            return 0;
        }
        if self.count >= TABLE_LEN {
            let mut candidate = false;
            let mut slot = 0;
            let mut farthest = f32::NEG_INFINITY;
            for col in 0..TABLE_LEN {
                if farthest < self.table[col].dist2
                    && self.table[col].obj.photo_no >= DSNAP_TYPE_UNK48
                {
                    candidate = true;
                    slot = col;
                    farthest = self.table[col].dist2;
                }
            }
            if !candidate {
                return 0;
            }
            let dist2 = player_pos.distance_squared(obj.center);
            if obj.photo_no >= DSNAP_TYPE_UNK48 && dist2 > farthest {
                return 0;
            }
            self.table[slot].obj = obj;
            self.table[slot].dist2 = dist2;
            0
        } else {
            let slot = self.count;
            self.table[slot].obj = obj;
            let dist2 = player_pos.distance_squared(self.table[slot].obj.center);
            self.table[slot].dist2 = dist2;
            self.count += 1;
            1
        }
    }

    /// `dSnap_RegistFig` (`d_snap.cpp:2212-2245`): the early-out unless the shutter is
    /// released, the figure's `l_CharaData` row scaled by `f1`/`f2`/`f3`, and the proxy
    /// registered with `SetInf(photoNo, actor, 0, 4, chara.cullAngle)`. The 3-argument
    /// overload (`:2213-2215`) is this with the actor's own position and `shape_angle.y`, and
    /// `f1`/`f2`/`f3` all `1.0`.
    pub fn regist_fig(
        &mut self,
        photo_no: u32,
        actor_pid: u32,
        pos: Vec3,
        angle_y: i16,
        f1: f32,
        f2: f32,
        f3: f32,
        player_pos: Vec3,
    ) {
        if self.chk_release_shutter() == 0 {
            return;
        }
        if photo_no >= DSNAP_TYPE_UNKCF {
            return;
        }
        let Some(table_index) = photo_index_to_table_index(photo_no) else {
            return;
        };
        let chara = CHARA_DATA[table_index];
        let offset = Vec3::from(chara.offset) * f3;
        let angle = Quat::from_rotation_y(from_s16(angle_y));
        let center = pos + angle * offset;
        let mut obj = SnapObj::default();
        obj.set_geo(center, chara.radius * f2, chara.height * f1, angle_y);
        obj.set_inf(photo_no, actor_pid, chara.cull_angle);
        self.regist(obj, player_pos);
    }

    /// `dSnap_packet::Judge` (`d_snap.cpp:1938-1973`): bucket the shutter window's pixels by
    /// their label, ratio the counts to the proxies' projected areas, and score the result.
    /// `pixels` is the window read row-major from [`SHUTTER_LEFT`]/[`SHUTTER_TOP`], in the
    /// ARGB layout `GXPeekARGB` returns under `GX_PF_RGBA6_Z24`. False when the shutter is up
    /// or the buffer is too short to hold the window.
    pub fn judge(&mut self, pixels: &[u32], camera: Option<&Camera>) -> bool {
        if self.chk_release_shutter() == 0 {
            return false;
        }
        let expected = (SHUTTER_WIDTH * SHUTTER_HEIGHT) as usize;
        if pixels.len() < expected {
            return false;
        }
        self.result = 0;
        for slot in &mut self.table {
            slot.obj.captured_pixels = 0;
            slot.obj.set_area_clear();
        }
        for y in SHUTTER_TOP..SHUTTER_BOTTOM {
            for x in SHUTTER_LEFT..SHUTTER_RIGHT {
                let row = (y - SHUTTER_TOP) * SHUTTER_WIDTH;
                let col = x - SHUTTER_LEFT;
                let word = pixels[(row + col) as usize];
                let label = (word >> LABEL_SHIFT) as usize;
                if label >= TABLE_LEN {
                    continue;
                }
                let obj = &mut self.table[label].obj;
                obj.captured_pixels += 1;
                obj.set_area(x as i16, y as i16);
            }
        }
        for col in 0..self.count {
            if self.table[col].obj.captured_pixels == 0 {
                continue;
            }
            let area = self.table[col].obj.calc_area_2d(camera);
            if area == 0.0 {
                continue;
            }
            self.table[col].obj.captured_ratio = self.table[col].obj.captured_pixels as f32 / area;
        }
        self.set_result(camera);
        self.flag &= !1;
        true
    }

    /// `dSnap_packet::FindPhoto` (`d_snap.cpp:1976-1986`).
    pub fn find_photo(&self, from: usize, photo_no: u32) -> Option<usize> {
        if from >= self.count {
            return None;
        }
        (from..self.count).find(|col| self.table[*col].obj.chk_photo(photo_no))
    }

    /// `dSnap_packet::Judge`'s dispatch (`d_snap.cpp:16-32`, `:2159-2171`): the ten special
    /// photo types have their own thresholds; any other index scores through
    /// [`SnapPacket::judge_figure`].
    fn judge_photo(&mut self, photo_no: u32) -> i32 {
        match photo_no {
            0 => 0,
            DSNAP_TYPE_POST => self.judge_post(),
            DSNAP_TYPE_BIKUTSUKI => self.judge_bikutsuki(),
            DSNAP_TYPE_COUPLE_LOOK => self.judge_couple_look(),
            DSNAP_TYPE_UNK04 => self.judge_gf(),
            DSNAP_TYPE_UNK05 => self.judge_genzo(),
            DSNAP_TYPE_UNK06 => self.judge_obasan4(),
            7..=9 => self.judge_gene(),
            10 => self.judge_test_m(),
            _ => 0,
        }
    }

    /// `dSnap_packet::JudgePost` (`d_snap.cpp:1989-1999`).
    fn judge_post(&self) -> i32 {
        let Some(col) = self.find_photo(0, DSNAP_TYPE_POST) else {
            return 0;
        };
        i32::from(self.table[col].obj.chk_success(2000, 0.25))
    }

    /// `dSnap_packet::JudgeBikutsuki` (`d_snap.cpp:2002-2012`): returns `2`, the photo type.
    fn judge_bikutsuki(&self) -> i32 {
        let Some(col) = self.find_photo(0, DSNAP_TYPE_BIKUTSUKI) else {
            return 0;
        };
        if self.table[col].obj.chk_success(2000, 0.25) {
            DSNAP_TYPE_BIKUTSUKI as i32
        } else {
            0
        }
    }

    /// `dSnap_packet::JudgeCoupleLook` (`d_snap.cpp:2015-2031`): both proxies must pass.
    fn judge_couple_look(&self) -> i32 {
        let Some(col1) = self.find_photo(0, DSNAP_TYPE_COUPLE_LOOK) else {
            return 0;
        };
        let Some(col2) = self.find_photo(col1 + 1, DSNAP_TYPE_COUPLE_LOOK) else {
            return 0;
        };
        if self.table[col1].obj.chk_success(2000, 0.25)
            && self.table[col2].obj.chk_success(2000, 0.25)
        {
            DSNAP_TYPE_COUPLE_LOOK as i32
        } else {
            0
        }
    }

    /// `dSnap_packet::JudgeGF` (`d_snap.cpp:2034-2044`).
    fn judge_gf(&self) -> i32 {
        let Some(col) = self.find_photo(0, DSNAP_TYPE_UNK04) else {
            return 0;
        };
        if self.table[col].obj.chk_success(5000, 0.45) {
            DSNAP_TYPE_UNK04 as i32
        } else {
            0
        }
    }

    /// `dSnap_packet::JudgeGenzo` (`d_snap.cpp:2047-2067`): the two unknowns `0x05` and `0x06`
    /// both pass on retail.
    fn judge_genzo(&self) -> i32 {
        let Some(col1) = self.find_photo(0, DSNAP_TYPE_UNK05) else {
            return 0;
        };
        let Some(col2) = self.find_photo(0, 6) else {
            return 0;
        };
        if self.table[col1].obj.chk_success(1500, 0.2)
            && self.table[col2].obj.chk_success(1500, 0.2)
        {
            DSNAP_TYPE_UNK05 as i32
        } else {
            0
        }
    }

    /// `dSnap_packet::JudgeObasan4` (`d_snap.cpp:2070-2084`): `0x06` passes only when no
    /// `0x05` was registered at all.
    fn judge_obasan4(&self) -> i32 {
        let Some(col1) = self.find_photo(0, DSNAP_TYPE_UNK06) else {
            return 0;
        };
        if self.find_photo(0, DSNAP_TYPE_UNK05).is_some() {
            return 0;
        }
        if self.table[col1].obj.chk_success(4000, 0.5) {
            DSNAP_TYPE_UNK06 as i32
        } else {
            0
        }
    }

    /// `dSnap_packet::JudgeTestM` (`d_snap.cpp:2087-2089`).
    fn judge_test_m(&self) -> i32 {
        0
    }

    /// `dSnap_packet::JudgeGene` (`d_snap.cpp:2092-2102`): the photo type itself when it
    /// passes.
    fn judge_gene(&self) -> i32 {
        let Some(col) = self.find_photo(0, self.photo_no) else {
            return 0;
        };
        if self.table[col].obj.chk_success(4000, 0.5) {
            self.photo_no as i32
        } else {
            0
        }
    }

    /// `dSnap_packet::JudgeFigure` (`d_snap.cpp:2105-2143`): the per-figure pixel and ratio
    /// thresholds from `l_CharaData`, with the detail bits a low ratio and a rear-facing
    /// photograph raise.
    fn judge_figure(&mut self, col: usize, camera: Option<&Camera>) -> i32 {
        if self.photo_no >= DSNAP_TYPE_LAST_INDEX {
            return 0;
        }
        if self.photo_no >= DSNAP_TYPE_UNKCF {
            return 0;
        }
        if self.photo_no <= DSNAP_TYPE_UNK48 {
            return 0;
        }
        let Some(table_index) = photo_index_to_table_index(self.photo_no) else {
            return 0;
        };
        let chara = CHARA_DATA[table_index];
        let min_pixels = i32::from(chara.min_pixels);
        let min_ratio = f32::from(chara.min_ratio) / 1000.0;
        if self.table[col].obj.captured_pixels < min_pixels {
            return 0;
        }
        if self.table[col].obj.captured_ratio < min_ratio {
            self.result_detail |= 2;
        }
        if self.table[col].obj.chk_cam_cull(camera) {
            self.result_detail |= 1;
        }
        self.photo_no as i32
    }

    /// `dSnap_packet::SetResult` (`d_snap.cpp:2146-2180`): the special photo types win on
    /// first sight; figure rows are scored highest-pixel-count first.
    fn set_result(&mut self, camera: Option<&Camera>) {
        self.result = 0;
        self.result_detail = 0;
        let mut seen = [false; JUDGE_TABLE_LEN];
        let mut best_pixels = -1;
        for col in 0..self.count {
            let photo_no = self.table[col].obj.photo_no;
            if self.table[col].obj.captured_pixels == 0 || photo_no == DSNAP_TYPE_NONE {
                continue;
            }
            let pixels = self.table[col].obj.captured_pixels;
            if (photo_no as usize) < seen.len() {
                if seen[photo_no as usize] {
                    continue;
                }
                self.photo_no = photo_no;
                self.result = self.judge_photo(photo_no);
                best_pixels = pixels;
                if self.result != 0 {
                    return;
                }
                seen[photo_no as usize] = true;
            } else if best_pixels < pixels {
                self.photo_no = photo_no;
                self.result = self.judge_figure(col, camera);
                best_pixels = pixels;
            }
        }
    }

    /// `dSnap_GetResult` (`d_snap.cpp:2198-2200`), the value `d_picture_box.cpp:616` stores in
    /// the photo slot's `snap_result`.
    pub fn result(&self) -> i32 {
        self.result
    }

    /// `dSnap_GetResultDetail` (`d_snap.cpp:2202-2205`), the follow-up
    /// `d_picture_box.cpp:617` stores beside it.
    pub fn result_detail(&self) -> u8 {
        self.result_detail
    }

    /// The registrations this frame's draws tabled, for a caller that wants to know what a
    /// judgement would score.
    pub fn registered(&self) -> &[SnapTable] {
        &self.table[..self.count]
    }
}

/// `dSnap_Execute` (`d_snap.cpp:2193-2195`), which the play scene runs once per frame
/// (`d_s_play.cpp:372`) ahead of the draws that re-register.
pub fn control(mut snap: ResMut<SnapPacket>) {
    snap.execute();
}
