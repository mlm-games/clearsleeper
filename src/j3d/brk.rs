//! `J3D1brk1` (`.brk`) TEV-register animation - `J3DAnmTevRegKey`, the key `mDoExt_brkAnm` wraps
//! (`m_Do/m_Do_ext.h:79-101`).
//!
//! The block is `TRK1`, a `J3DAnmTevRegKeyData` (`J3DAnimation.h:224-255`), and the two register
//! families it drives are separate halves of it: the C registers are `GXColorS10` triples with an
//! alpha (`getTevColorReg`, `J3DAnimation.cpp:901-973`) and the K registers are `GXColor` bytes
//! (`getTevKonstReg`, `:975-1063`). Both halves carry a per-material table of four per-channel key
//! tables and a `colorId`, which `J3DMaterialAnm::calc` turns into
//! `pMaterial->getTevColor(i)->mColor` and `pMaterial->getTevKColor(i)->mColor`
//! (`J3DMaterialAnm.cpp:63-70`). The material a table entry updates is resolved by name at init
//! time (`J3DAnmTevRegKey::searchUpdateMaterialID`), not read from the file's id arrays.
//!
//! The value arrays are `s16` for both families, and every channel samples them with
//! `J3DGetKeyFrameInterpolation<s16>` (`J3DAnimation.cpp:413-456`), whose elements are s16 values
//! promoted to f32 for the hermite segment.

const HEADER_LEN: usize = 0x20;
const FILE_MAGIC: &[u8; 8] = b"J3D1brk1";
const BLOCK_TAG: &[u8; 4] = b"TRK1";
const BLOCK_MIN: usize = 0x58;
const ATTRIBUTE: usize = 0x08;
const FRAME_MAX: usize = 0x0A;
const C_MATERIAL_NUM: usize = 0x0C;
const K_MATERIAL_NUM: usize = 0x0E;
const C_VALUE_NUM: usize = 0x10;
const K_VALUE_NUM: usize = 0x18;
const C_TABLE: usize = 0x20;
const K_TABLE: usize = 0x24;
const C_MATERIAL_IDS: usize = 0x28;
const K_MATERIAL_IDS: usize = 0x2C;
const C_NAME_TABLE: usize = 0x30;
const K_NAME_TABLE: usize = 0x34;
const C_VALUES: usize = 0x38;
const K_VALUES: usize = 0x48;
/// `J3DAnmCRegKeyTable` and `J3DAnmKRegKeyTable` (`J3DAnimation.h:62-78`).
const ENTRY_STRIDE: usize = 0x1C;
const ENTRY_KEY_STRIDE: usize = 6;
const ENTRY_COLOR_ID: usize = 0x18;
const CHANNELS: usize = 4;
/// The C registers' sampled range - `J3DAnimation.cpp:913-918`.
const C_MIN: f32 = -1024.0;
const C_MAX: f32 = 1023.0;
/// The K registers' sampled range - `J3DAnimation.cpp:988-993`.
const K_MIN: f32 = 0.0;
const K_MAX: f32 = 255.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrkError {
    Truncated,
    Magic,
    Block,
    Range,
}

impl BrkError {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Truncated => "brk data truncated",
            Self::Magic => "brk magic mismatch",
            Self::Block => "brk block table invalid",
            Self::Range => "brk key index out of range",
        }
    }
}

/// `J3DAnmKeyTableBase` (`J3DAnimation.h:6-10`): the key span of one channel, whose `mType` picks
/// the three-value or four-value key layout (`J3DGetKeyFrameInterpolation`, `:415`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Key {
    pub max_frame: u16,
    pub offset: u16,
    pub kind: u16,
}

/// One table entry: the four channel key spans and the register index the entry writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entry {
    /// R, G, B and A in the reference's own order (`mRTable`..`mATable`).
    pub channels: [Key; CHANNELS],
    /// `mColorId`, the material's register index: `getTevColor(colorId)` for the C family,
    /// `getTevKColor(colorId)` for the K family (`J3DMaterialAttach.cpp:323`, `:334`).
    pub color_id: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TevRegKey {
    /// `J3DAnmBase::getFrameMax()`, the frame count `mDoExt_baseAnm::initPlay` takes.
    pub frame_max: u16,
    /// `J3DAnmBase::mAttribute`, the file's own `field_0x8`.
    pub attribute: u8,
    pub c_reg: Vec<Entry>,
    pub k_reg: Vec<Entry>,
    /// The four C value arrays in R, G, B, A order.
    pub c_values: [Vec<i16>; CHANNELS],
    /// The four K value arrays in R, G, B, A order.
    pub k_values: [Vec<i16>; CHANNELS],
    pub c_names: Vec<String>,
    pub k_names: Vec<String>,
    /// The files' own pre-resolved material ids; [`TevRegKey::resolve`] overwrites them.
    pub c_ids: Vec<u16>,
    pub k_ids: Vec<u16>,
}

impl TevRegKey {
    /// `J3DAnmTevRegKey::getTevColorReg` - `J3DAnimation.cpp:901-973`.
    pub fn tev_color(&self, entry: usize, frame: f32) -> Option<[i16; 4]> {
        let entry = self.c_reg.get(entry)?;
        let mut rgba = [0_i16; CHANNELS];
        for (channel, slot) in rgba.iter_mut().enumerate() {
            let key = entry.channels[channel];
            *slot = match key.max_frame {
                0 => 0,
                1 => channel_value(&self.c_values[channel], key.offset),
                _ => clamp(
                    interpolate(&key, &self.c_values[channel], frame),
                    C_MIN,
                    C_MAX,
                ) as i16,
            };
        }
        Some(rgba)
    }

    /// `J3DAnmTevRegKey::getTevKonstReg` - `J3DAnimation.cpp:975-1063`. The K family's case-one arm
    /// reads the s16 array straight into a `u8` field, so a key value outside the byte range is
    /// truncated; the interpolated arm clamps instead.
    pub fn tev_k_color(&self, entry: usize, frame: f32) -> Option<[u8; 4]> {
        let entry = self.k_reg.get(entry)?;
        let mut rgba = [0_u8; CHANNELS];
        for (channel, slot) in rgba.iter_mut().enumerate() {
            let key = entry.channels[channel];
            *slot = match key.max_frame {
                0 => 0,
                1 => channel_value(&self.k_values[channel], key.offset) as u8,
                _ => clamp(
                    interpolate(&key, &self.k_values[channel], frame),
                    K_MIN,
                    K_MAX,
                ) as u8,
            };
        }
        Some(rgba)
    }

    /// `J3DAnmTevRegKey::searchUpdateMaterialID` for both halves: each entry's name against the
    /// model's material name table, `0xFFFF` for a miss. The reference runs this from
    /// `mDoExt_brkAnm::init` (`m_Do_ext.cpp:328`) and writes the answer over the file's own ids.
    pub fn resolve(&mut self, material: impl Fn(&str) -> Option<u16>) {
        for (index, name) in self.c_names.iter().enumerate() {
            if let Some(slot) = self.c_ids.get_mut(index) {
                *slot = material(name).unwrap_or(u16::MAX);
            }
        }
        for (index, name) in self.k_names.iter().enumerate() {
            if let Some(slot) = self.k_ids.get_mut(index) {
                *slot = material(name).unwrap_or(u16::MAX);
            }
        }
    }
}

pub fn parse(data: &[u8]) -> Result<TevRegKey, BrkError> {
    if data.len() < HEADER_LEN {
        return Err(BrkError::Truncated);
    }
    if &data[0..8] != FILE_MAGIC {
        return Err(BrkError::Magic);
    }
    if read_u32(data, 8)? as usize > data.len() {
        return Err(BrkError::Truncated);
    }
    if read_u32(data, 12)? != 1 {
        return Err(BrkError::Block);
    }
    let block = HEADER_LEN;
    let tag = data.get(block..block + 4).ok_or(BrkError::Truncated)?;
    if tag != BLOCK_TAG {
        return Err(BrkError::Block);
    }
    let block_size = read_u32(data, block + 4)? as usize;
    if block_size < BLOCK_MIN || block + block_size > data.len() {
        return Err(BrkError::Block);
    }
    let end = block + block_size;
    let frame_max = read_i16(data, block + FRAME_MAX)?;
    if frame_max < 0 {
        return Err(BrkError::Block);
    }
    let attribute = *data.get(block + ATTRIBUTE).ok_or(BrkError::Truncated)?;
    let c_num = usize::from(read_u16(data, block + C_MATERIAL_NUM)?);
    let k_num = usize::from(read_u16(data, block + K_MATERIAL_NUM)?);

    let c_reg = entries(data, block, end, C_TABLE, c_num)?;
    let k_reg = entries(data, block, end, K_TABLE, k_num)?;
    let c_values = channels(data, block, end, C_VALUES, C_VALUE_NUM)?;
    let k_values = channels(data, block, end, K_VALUES, K_VALUE_NUM)?;
    for entry in &c_reg {
        for (channel, key) in entry.channels.iter().enumerate() {
            validate(key, c_values[channel].len())?;
        }
    }
    for entry in &k_reg {
        for (channel, key) in entry.channels.iter().enumerate() {
            validate(key, k_values[channel].len())?;
        }
    }

    let c_ids = ids(data, block, end, C_MATERIAL_IDS, c_num)?;
    let k_ids = ids(data, block, end, K_MATERIAL_IDS, k_num)?;
    let c_names = names(data, block, end, C_NAME_TABLE, c_num)?;
    let k_names = names(data, block, end, K_NAME_TABLE, k_num)?;
    Ok(TevRegKey {
        frame_max: frame_max as u16,
        attribute,
        c_reg,
        k_reg,
        c_values,
        k_values,
        c_names,
        k_names,
        c_ids,
        k_ids,
    })
}

fn entries(
    data: &[u8],
    block: usize,
    end: usize,
    field: usize,
    count: usize,
) -> Result<Vec<Entry>, BrkError> {
    let base = table(data, block, end, field, count * ENTRY_STRIDE)?;
    let mut entries = Vec::with_capacity(count);
    for index in 0..count {
        let entry_at = at(base, index * ENTRY_STRIDE)?;
        if entry_at + ENTRY_STRIDE > end {
            return Err(BrkError::Block);
        }
        let mut channels = [Key {
            max_frame: 0,
            offset: 0,
            kind: 0,
        }; CHANNELS];
        for (channel, slot) in channels.iter_mut().enumerate() {
            let key = at(entry_at, channel * ENTRY_KEY_STRIDE)?;
            *slot = Key {
                max_frame: read_u16(data, key)?,
                offset: read_u16(data, key + 2)?,
                kind: read_u16(data, key + 4)?,
            };
        }
        entries.push(Entry {
            channels,
            color_id: *data
                .get(entry_at + ENTRY_COLOR_ID)
                .ok_or(BrkError::Truncated)?,
        });
    }
    Ok(entries)
}

/// The four per-channel value arrays of one register family. Each array's length is its own
/// count field, and an absent array (a count of zero) has no offset at all.
fn channels(
    data: &[u8],
    block: usize,
    end: usize,
    field: usize,
    count_field: usize,
) -> Result<[Vec<i16>; CHANNELS], BrkError> {
    let mut out: [Vec<i16>; CHANNELS] = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];
    for (channel, slot) in out.iter_mut().enumerate() {
        let count = usize::from(read_u16(data, block + count_field + channel * 2)?);
        if count == 0 {
            continue;
        }
        let base = table(data, block, end, field + channel * 4, count * 2)?;
        let mut values = Vec::with_capacity(count);
        for index in 0..count {
            values.push(read_i16(data, at(base, index * 2)?)?);
        }
        *slot = values;
    }
    Ok(out)
}

fn ids(
    data: &[u8],
    block: usize,
    end: usize,
    field: usize,
    count: usize,
) -> Result<Vec<u16>, BrkError> {
    if count == 0 {
        return Ok(Vec::new());
    }
    let base = table(data, block, end, field, count * 2)?;
    let mut ids = Vec::with_capacity(count);
    for index in 0..count {
        ids.push(read_u16(data, at(base, index * 2)?)?);
    }
    Ok(ids)
}

fn at(base: usize, offset: usize) -> Result<usize, BrkError> {
    base.checked_add(offset).ok_or(BrkError::Block)
}

fn table(
    data: &[u8],
    block: usize,
    end: usize,
    field: usize,
    bytes: usize,
) -> Result<usize, BrkError> {
    if bytes == 0 {
        return Ok(end);
    }
    let rel = read_u32(data, at(block, field)?)? as usize;
    if rel < BLOCK_MIN {
        return Err(BrkError::Block);
    }
    let start = at(block, rel)?;
    if at(start, bytes)? > end {
        return Err(BrkError::Block);
    }
    Ok(start)
}

/// `JUTNameTab` over a `ResNTAB` (`JUTNameTab.h:6-17`), the same shape the loader hands
/// `mUpdateMaterialName`.
fn names(
    data: &[u8],
    block: usize,
    end: usize,
    field: usize,
    count: usize,
) -> Result<Vec<String>, BrkError> {
    if count == 0 {
        return Ok(Vec::new());
    }
    let base = table(data, block, end, field, 4 + count * 4)?;
    if usize::from(read_u16(data, base)?) < count {
        return Err(BrkError::Block);
    }
    let mut names = Vec::with_capacity(count);
    for index in 0..count {
        let entry = at(base, 4 + index * 4)?;
        let offset = usize::from(read_u16(data, entry + 2)?);
        let string = at(base, offset)?;
        if string >= end {
            return Err(BrkError::Block);
        }
        let bytes = data.get(string..end).ok_or(BrkError::Truncated)?;
        let length = bytes
            .iter()
            .position(|&byte| byte == 0)
            .ok_or(BrkError::Block)?;
        names.push(
            std::str::from_utf8(&bytes[..length])
                .map_err(|_| BrkError::Block)?
                .to_owned(),
        );
    }
    Ok(names)
}

fn stride(kind: u16) -> usize {
    if kind == 0 { 3 } else { 4 }
}

fn validate(key: &Key, values: usize) -> Result<(), BrkError> {
    let count = usize::from(key.max_frame);
    let offset = usize::from(key.offset);
    let needed = if count == 0 {
        return Ok(());
    } else if count == 1 {
        offset + 1
    } else {
        offset + count * stride(key.kind)
    };
    if needed > values {
        return Err(BrkError::Range);
    }
    Ok(())
}

fn channel_value(values: &[i16], offset: u16) -> i16 {
    values.get(usize::from(offset)).copied().unwrap_or(0)
}

fn clamp(value: f32, min: f32, max: f32) -> f32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

/// `J3DGetKeyFrameInterpolation<s16>` - `J3DAnimation.cpp:413-456`: the last key at or before the
/// frame, hermite-interpolated over the two keys that bracket it. Both key layouts are here: a
/// three-value key stores its tangent at `+2`, and the four-value layout stores the left key's at
/// `+3` and the right key's at `+2`.
fn interpolate(key: &Key, values: &[i16], frame: f32) -> f32 {
    let stride = stride(key.kind);
    let count = usize::from(key.max_frame);
    let base = usize::from(key.offset);
    let value = |index: usize| f32::from(values.get(base + index).copied().unwrap_or(0));
    if frame < value(0) {
        return value(1);
    }
    let last = stride * (count - 1);
    if value(last) <= frame {
        return value(last + 1);
    }
    let mut start = 0;
    let mut remaining = count;
    while remaining > 1 {
        let mid = remaining / 2;
        if frame >= value(start + stride * mid) {
            start += stride * mid;
            remaining -= mid;
        } else {
            remaining = mid;
        }
    }
    let segment = |index: usize| value(start + index);
    if key.kind == 0 {
        hermite(
            frame,
            segment(0),
            segment(1),
            segment(2),
            segment(3),
            segment(4),
            segment(5),
        )
    } else {
        hermite(
            frame,
            segment(0),
            segment(1),
            segment(3),
            segment(4),
            segment(5),
            segment(6),
        )
    }
}

/// `JMAHermiteInterpolation` - the cubic the reference's key sampler runs over a bracketing pair.
fn hermite(frame: f32, t0: f32, v0: f32, m0: f32, t1: f32, v1: f32, m1: f32) -> f32 {
    let length = t1 - t0;
    let inv = 1.0 / length;
    let delta = frame - t0;
    let f2 = delta * delta * inv;
    let f10 = f2 * inv;
    let f11 = delta * f10;
    let f12 = f11 * inv;
    v0 * (1.0 + ((2.0 * f12) - (3.0 * f10)))
        + v1 * ((-2.0 * f12) + (3.0 * f10))
        + m0 * (delta + (f11 - (2.0 * f2)))
        + m1 * (f11 - f2)
}

fn read_u16(data: &[u8], start: usize) -> Result<u16, BrkError> {
    let end = start.checked_add(2).ok_or(BrkError::Block)?;
    let bytes = data.get(start..end).ok_or(BrkError::Truncated)?;
    Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
}

fn read_i16(data: &[u8], start: usize) -> Result<i16, BrkError> {
    let end = start.checked_add(2).ok_or(BrkError::Block)?;
    let bytes = data.get(start..end).ok_or(BrkError::Truncated)?;
    Ok(i16::from_be_bytes([bytes[0], bytes[1]]))
}

fn read_u32(data: &[u8], start: usize) -> Result<u32, BrkError> {
    let end = start.checked_add(4).ok_or(BrkError::Block)?;
    let bytes = data.get(start..end).ok_or(BrkError::Truncated)?;
    Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}
