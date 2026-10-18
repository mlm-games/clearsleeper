//! `J3D1bck1` (`.bck`) transform-key animation, baked to a repame track.
//!
//! Layout, from `J3DAnmLoader::setAnmTransform` and `J3DAnmTransformKey::calcTransform`:
//! a `J3D1bck1` container of sequential blocks, then one `ANK1` block holding a
//! per-joint table of `{max_frame, offset, kind}` triples — one for each of
//! scale, rotation and translation per axis, indexed `joint * 3 + axis`.
//! Key data lives in three arrays: scale and translation as `f32`, rotation as
//! `s16` angles of 65536 per turn (`TSinCosTable::sinShort`) shifted by
//! `mDecShift`.
//!
//! [`Morph`] takes the same parse the other way: the baked tracks are sampled against a game
//! model's `JNT1` joint order and composed down its hierarchy, which is the `getAnmMtx` answer a
//! `mDoExt_McaMorf`-driven actor reads (`m_Do/m_Do_ext.h:324-380`).

use std::collections::HashMap;

use glam::{Mat4, Quat};
use repame_view3d::{Animation, Interp, JointPose, NodePose, Skeleton};

const HEADER_LEN: usize = 0x20;
const BLOCK_LEN: usize = 8;
/// `J3DAnmTransformKeyTable`: three `J3DAnmKeyTableBase` of six bytes.
const ENTRY_STRIDE: usize = 0x12;
const CHANNEL_STRIDE: usize = 6;
const CHANNEL_SCALE: usize = 0;
const CHANNEL_ROTATION: usize = 1;
const CHANNEL_TRANSLATION: usize = 2;
const ANGLE_SCALE: f32 = std::f32::consts::TAU / 65536.0;
/// `J3DFrameCtrl` starts `mRate` at 1.0 and the game ticks at 60 Hz.
const FRAME_RATE: f32 = 60.0;
const WIDE_S16: usize = 2;
const WIDE_F32: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BckError {
    Truncated,
    Magic,
    Block,
    Table,
}

impl BckError {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Truncated => "bck truncated",
            Self::Magic => "bck magic invalid",
            Self::Block => "bck block invalid",
            Self::Table => "bck key table invalid",
        }
    }
}

#[derive(Clone, Copy)]
struct Channel {
    max_frame: usize,
    offset: usize,
    kind: usize,
}

fn slice(data: &[u8], at: usize, len: usize) -> Result<&[u8], BckError> {
    let end = at.checked_add(len).ok_or(BckError::Truncated)?;
    data.get(at..end).ok_or(BckError::Truncated)
}

fn read_u8(data: &[u8], at: usize) -> Result<u8, BckError> {
    Ok(*slice(data, at, 1)?.first().ok_or(BckError::Truncated)?)
}

fn read_u16(data: &[u8], at: usize) -> Result<u16, BckError> {
    let b = slice(data, at, 2)?;
    Ok(u16::from_be_bytes([b[0], b[1]]))
}

fn read_i16(data: &[u8], at: usize) -> Result<i16, BckError> {
    let b = slice(data, at, 2)?;
    Ok(i16::from_be_bytes([b[0], b[1]]))
}

fn read_u32(data: &[u8], at: usize) -> Result<u32, BckError> {
    let b = slice(data, at, 4)?;
    Ok(u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
}

fn read_i32(data: &[u8], at: usize) -> Result<i32, BckError> {
    let b = slice(data, at, 4)?;
    Ok(i32::from_be_bytes([b[0], b[1], b[2], b[3]]))
}

/// Offset of a block field, measured from the block start.
fn field(base: usize, delta: i32) -> Result<usize, BckError> {
    if delta < 0 {
        return Err(BckError::Table);
    }
    base.checked_add(delta as usize).ok_or(BckError::Table)
}

fn find_block(data: &[u8], tag: &[u8; 4]) -> Result<usize, BckError> {
    if data.len() < HEADER_LEN {
        return Err(BckError::Truncated);
    }
    if &data[..8] != b"J3D1bck1" {
        return Err(BckError::Magic);
    }
    let count = read_u32(data, 12)? as usize;
    let mut cursor = HEADER_LEN;
    for _ in 0..count {
        let head = slice(data, cursor, BLOCK_LEN)?;
        let size = u32::from_be_bytes([head[4], head[5], head[6], head[7]]) as usize;
        if size < BLOCK_LEN {
            return Err(BckError::Block);
        }
        let end = cursor.checked_add(size).ok_or(BckError::Block)?;
        if end > data.len() {
            return Err(BckError::Block);
        }
        if &head[..4] == tag {
            return Ok(cursor);
        }
        cursor = end;
    }
    Err(BckError::Block)
}

fn read_slot(
    data: &[u8],
    base: usize,
    count: usize,
    wide: usize,
    index: usize,
) -> Result<f32, BckError> {
    if index >= count {
        return Err(BckError::Table);
    }
    let at = base
        .checked_add(index.checked_mul(wide).ok_or(BckError::Table)?)
        .ok_or(BckError::Table)?;
    let b = slice(data, at, wide)?;
    Ok(if wide == WIDE_S16 {
        f32::from(i16::from_be_bytes([b[0], b[1]]))
    } else {
        f32::from_be_bytes([b[0], b[1], b[2], b[3]])
    })
}

/// `JMAHermiteInterpolation`: tangents are derivatives per key, so the segment
/// length scales them.
fn hermite(
    frame: f32,
    time0: f32,
    value0: f32,
    tangent0: f32,
    time1: f32,
    value1: f32,
    tangent1: f32,
) -> f32 {
    let length = time1 - time0;
    let delta = frame - time0;
    let inv = 1.0 / length;
    let f2 = delta * delta * inv;
    let f10 = f2 * inv;
    let f11 = delta * f10;
    let f12 = f11 * inv;
    value0 * (1.0 + (2.0 * f12 - 3.0 * f10))
        + value1 * (-2.0 * f12 + 3.0 * f10)
        + tangent0 * (delta + (f11 - 2.0 * f2))
        + tangent1 * (f11 - f2)
}

fn sample(
    data: &[u8],
    base: usize,
    count: usize,
    wide: usize,
    channel: Channel,
    frame: f32,
    default: f32,
) -> Result<f32, BckError> {
    match channel.max_frame {
        0 => Ok(default),
        1 => read_slot(data, base, count, wide, channel.offset),
        _ => {
            let stride = if channel.kind == 0 { 3 } else { 4 };
            let at = |key: usize| channel.offset + key * stride;
            let time = |key: usize| read_slot(data, base, count, wide, at(key));
            let first = time(0)?;
            if frame < first {
                return read_slot(data, base, count, wide, at(0) + 1);
            }
            let last = channel.max_frame - 1;
            if time(last)? <= frame {
                return read_slot(data, base, count, wide, at(last) + 1);
            }
            let mut lo = 0_usize;
            let mut hi = channel.max_frame;
            while hi - lo > 1 {
                let mid = (lo + hi) / 2;
                if frame >= time(mid)? {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }
            let value0 = read_slot(data, base, count, wide, at(lo) + 1)?;
            let value1 = read_slot(data, base, count, wide, at(lo + 1) + 1)?;
            // A three-value key stores its tangent at +2; the four-value layout
            // stores the left key's at +3 and the right key's at +2.
            let tangent0 = read_slot(data, base, count, wide, at(lo) + stride - 1)?;
            let tangent1 = read_slot(data, base, count, wide, at(lo + 1) + 2)?;
            Ok(hermite(
                frame,
                time(lo)?,
                value0,
                tangent0,
                time(lo + 1)?,
                value1,
                tangent1,
            ))
        }
    }
}

/// `J3DGetTranslateRotateMtx` composes the s16 triple `Rz*Ry*Rx`.
fn rotation(units: [i32; 3]) -> [f32; 4] {
    let [x, y, z] = units;
    let rx = x as f32 * ANGLE_SCALE;
    let ry = y as f32 * ANGLE_SCALE;
    let rz = z as f32 * ANGLE_SCALE;
    (Quat::from_rotation_z(rz) * Quat::from_rotation_y(ry) * Quat::from_rotation_x(rx)).to_array()
}

pub fn parse(data: &[u8]) -> Result<Animation, BckError> {
    let start = find_block(data, b"ANK1")?;
    let shift = u32::from(read_u8(data, start + 9)?).min(31);
    let frame_max = usize::try_from(read_i16(data, start + 0x0A)?).map_err(|_| BckError::Table)?;
    let joint_count = usize::from(read_u16(data, start + 0x0C)?);
    let counts = [
        usize::from(read_u16(data, start + 0x0E)?),
        usize::from(read_u16(data, start + 0x10)?),
        usize::from(read_u16(data, start + 0x12)?),
    ];
    let table = field(start, read_i32(data, start + 0x14)?)?;
    let scale_base = field(start, read_i32(data, start + 0x18)?)?;
    let rot_base = field(start, read_i32(data, start + 0x1C)?)?;
    let trans_base = field(start, read_i32(data, start + 0x20)?)?;
    let frames = frame_max + 1;
    if frames.saturating_mul(joint_count) > data.len() {
        return Err(BckError::Table);
    }
    slice(data, table, joint_count * ENTRY_STRIDE * 3)?;

    let times = (0..frames)
        .map(|frame| frame as f32 / FRAME_RATE)
        .collect::<Vec<_>>();
    let linear = vec![Interp::Linear; frames];
    let mut tracks = HashMap::new();

    for joint in 0..joint_count {
        let entry = table + joint * ENTRY_STRIDE * 3;
        const EMPTY: Channel = Channel {
            max_frame: 0,
            offset: 0,
            kind: 0,
        };
        let mut channels = [[EMPTY; 3]; 3];
        for axis in 0..3 {
            let at = entry + axis * ENTRY_STRIDE;
            for channel in 0..3 {
                let head = at + channel * CHANNEL_STRIDE;
                channels[channel][axis] = Channel {
                    max_frame: usize::from(read_u16(data, head)?),
                    offset: usize::from(read_u16(data, head + 2)?),
                    kind: usize::from(read_u16(data, head + 4)?),
                };
            }
        }

        let mut pose = NodePose {
            times: times.clone(),
            ..NodePose::default()
        };
        if channels[CHANNEL_SCALE].iter().any(|c| c.max_frame > 0) {
            pose.scales = sample_axes(
                data,
                scale_base,
                counts[0],
                WIDE_F32,
                &channels[CHANNEL_SCALE],
                frames,
                1.0,
            )?;
            pose.scale_interp = linear.clone();
        }
        if channels[CHANNEL_ROTATION].iter().any(|c| c.max_frame > 0) {
            let mut rotations = Vec::with_capacity(frames);
            for frame in 0..frames {
                let frame = frame as f32;
                let mut units = [0_i32; 3];
                for (axis, channel) in channels[CHANNEL_ROTATION].iter().enumerate() {
                    let value = sample(data, rot_base, counts[1], WIDE_S16, *channel, frame, 0.0)?;
                    // `mRotation` is an `SVec`, so the shifted key truncates to 16 bits.
                    units[axis] = ((value as i32) << shift) as i16 as i32;
                }
                rotations.push(rotation(units));
            }
            pose.rotations = rotations;
            pose.rotation_interp = linear.clone();
        }
        if channels[CHANNEL_TRANSLATION]
            .iter()
            .any(|c| c.max_frame > 0)
        {
            pose.translations = sample_axes(
                data,
                trans_base,
                counts[2],
                WIDE_F32,
                &channels[CHANNEL_TRANSLATION],
                frames,
                0.0,
            )?;
            pose.translation_interp = linear.clone();
        }
        if !pose.is_empty() {
            tracks.insert(joint, pose);
        }
    }

    Ok(Animation {
        name: String::from("bck"),
        tracks,
        ..Animation::default()
    })
}

/// A `J3D1bck1` paired with the model hierarchy it drives: the [`Animation`] the loader read and
/// the joint parents a `JNT1` block carries, sampled into the world matrices a `J3DModel`
/// answers with (`J3DModel.h:75`).
///
/// A track is keyed by joint index, which is `J3DAnmTransformKey::calcTransform`'s `idx`
/// (`J3DAnimation.cpp:459`), and holds one joint local per disk frame, so a sample at a whole
/// frame reproduces the reference's key-table read exactly.
#[derive(Debug, Clone)]
pub struct Morph {
    animation: Animation,
    skeleton: Skeleton,
    joint_count: usize,
    frame_max: f32,
}

impl Morph {
    /// `J3DAnmKeyLoader_v15::readAnmTransform` (`J3DAnmLoader.cpp:450-465`) over the `ANK1` walk
    /// above: parse `data`, then pair its joint tracks with the hierarchy `parents` carries - in
    /// joint order, [`crate::j3d::bdl::RoomMesh::joint_parents`]'s spelling, where a root is a
    /// value no joint index is. `frame_max` is the block's own `mFrameMax`, the count
    /// `mDoExt_McaMorf::getEndFrame` reports (`m_Do/m_Do_ext.h:349`).
    pub fn parse(data: &[u8], parents: &[u16]) -> Result<Self, BckError> {
        let animation = parse(data)?;
        let start = find_block(data, b"ANK1")?;
        let frame_max = f32::from(read_i16(data, start + 0x0A)?);
        let skeleton = Skeleton {
            parents: parents
                .iter()
                .enumerate()
                .map(|(joint, parent)| {
                    let parent = usize::from(*parent);
                    (joint, (parent < joint).then_some(parent))
                })
                .collect(),
            bind_local: HashMap::new(),
        };
        Ok(Self {
            animation,
            skeleton,
            joint_count: parents.len(),
            frame_max,
        })
    }

    /// `mDoExt_McaMorf::getEndFrame` (`m_Do/m_Do_ext.h:349`) over the `ANK1` frame count.
    pub fn frame_max(&self) -> f32 {
        self.frame_max
    }

    /// `mModel->getAnmMtx(no)` (`J3DModel.h:75`) at `frame`, in joint order: sample every track at
    /// that frame and compose each joint's local down the hierarchy, `J3DMtxCalcAnm::calc`
    /// (`J3DJoint.cpp:17-27`) over `J3DMtxCalcBasic::calcTransform` (`:57-85`).
    ///
    /// A joint the animation carries no track for composes from identity, which is the local the
    /// reference leaves for a channel its key table does not cover (`J3DAnimation.cpp:459-516`):
    /// the model's own bind locals are deliberately not a fallback here, because the reference
    /// replaces the whole joint local rather than the channels the file happens to key.
    pub fn joint_matrices(&self, frame: f32) -> Vec<Mat4> {
        let mut locals: HashMap<usize, JointPose> = HashMap::new();
        for (joint, track) in &self.animation.tracks {
            locals.insert(*joint, track.sample(frame / FRAME_RATE));
        }
        let world = self.skeleton.world_matrices(&locals);
        (0..self.joint_count)
            .map(|joint| world.get(&joint).copied().unwrap_or(Mat4::IDENTITY))
            .collect()
    }
}

fn sample_axes(
    data: &[u8],
    base: usize,
    count: usize,
    wide: usize,
    channels: &[Channel; 3],
    frames: usize,
    default: f32,
) -> Result<Vec<[f32; 3]>, BckError> {
    (0..frames)
        .map(|frame| {
            let frame = frame as f32;
            let mut axis = [default; 3];
            for (slot, channel) in channels.iter().enumerate() {
                axis[slot] = sample(data, base, count, wide, *channel, frame, default)?;
            }
            Ok(axis)
        })
        .collect()
}
