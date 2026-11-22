//! `J3D1bpk1` (`.bpk`) material-colour animation, the `J3DAnmColor` a `mDoExt_bpkAnm` wraps
//! (`m_Do/m_Do_ext.h:122-131`). The sword tip-stab files in `LkAnm.arc` are Key animations
//! (`PAK1`), sampled per material by `J3DAnmColorKey::getColor`
//! (`J3DAnimation.cpp:616-693`) over four channels of keyed `u8` values.

const HEADER_LEN: usize = 0x20;
const FILE_MAGIC: &[u8; 8] = b"J3D1bpk1";
const BLOCK_TAG: &[u8; 4] = b"PAK1";
const BLOCK_MIN: usize = 0x34;
const ATTRIBUTE: usize = 0x08;
const FRAME_MAX: usize = 0x0C;
const MATERIAL_NUM: usize = 0x0E;
const TABLE: usize = 0x18;
const MATERIAL_IDS: usize = 0x1C;
const NAME_TABLE: usize = 0x20;
const VALUES: [usize; 4] = [0x24, 0x28, 0x2C, 0x30];
const CHANNELS: usize = 4;
const ENTRY_STRIDE: usize = 0x18;
const KEY_STRIDE: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BpkError {
    Truncated,
    Magic,
    Block,
    Range,
}

impl BpkError {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Truncated => "bpk data truncated",
            Self::Magic => "bpk magic mismatch",
            Self::Block => "bpk block table invalid",
            Self::Range => "bpk key index out of range",
        }
    }
}

/// One channel's key span from `J3DAnmKeyTableBase` (`J3DAnimation.h:8-12`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Key {
    pub max_frame: u16,
    pub offset: u16,
    pub kind: u16,
}

/// One `J3DAnmColorKeyTable`, four channels in RGBA order (`J3DAnimation.h:14-22`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorEntry {
    pub channels: [Key; CHANNELS],
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnmColor {
    pub frame_max: u16,
    pub attribute: u8,
    pub entries: Vec<ColorEntry>,
    pub values: [Vec<u8>; CHANNELS],
    pub names: Vec<String>,
    pub material_ids: Vec<u16>,
}

impl AnmColor {
    /// `J3DAnmColorKey::getColor` (`J3DAnimation.cpp:616-693`): a one-key channel reads its value
    /// directly, a keyed channel interpolates the two keys bracketing the frame and clamps to a
    /// byte.
    pub fn color(&self, entry: usize, frame: f32) -> Option<[u8; 4]> {
        let entry = self.entries.get(entry)?;
        let mut rgba = [0_u8; CHANNELS];
        for (channel, (slot, key)) in rgba.iter_mut().zip(entry.channels).enumerate() {
            *slot = match key.max_frame {
                0 => 0,
                1 => self.values[channel]
                    .get(usize::from(key.offset))
                    .copied()
                    .unwrap_or(0),
                _ => interpolate(&self.values[channel], &key, frame),
            };
        }
        Some(rgba)
    }

    /// `J3DAnmColor::searchUpdateMaterialID`, matching an entry's name to a model material.
    pub fn material_id(&self, name: &str) -> Option<u16> {
        self.names
            .iter()
            .position(|candidate| candidate == name)
            .and_then(|index| self.material_ids.get(index).copied())
    }
}

fn u16_at(data: &[u8], at: usize) -> Result<u16, BpkError> {
    let bytes = data.get(at..at + 2).ok_or(BpkError::Truncated)?;
    Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
}

fn i32_at(data: &[u8], at: usize) -> Result<usize, BpkError> {
    let bytes = data.get(at..at + 4).ok_or(BpkError::Truncated)?;
    Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize)
}

fn relative(block: usize, offset: usize, end: usize) -> Result<usize, BpkError> {
    let at = block.checked_add(offset).ok_or(BpkError::Range)?;
    (at < end).then_some(at).ok_or(BpkError::Block)
}

fn table(
    data: &[u8],
    block: usize,
    end: usize,
    offset: usize,
    count: usize,
) -> Result<Vec<ColorEntry>, BpkError> {
    let start = relative(block, offset, end)?;
    data.get(start..start + count * ENTRY_STRIDE)
        .ok_or(BpkError::Block)?
        .chunks_exact(ENTRY_STRIDE)
        .map(|entry| {
            let mut channels = [Key {
                max_frame: 0,
                offset: 0,
                kind: 0,
            }; CHANNELS];
            for (slot, chunk) in channels.iter_mut().zip(entry.chunks_exact(KEY_STRIDE)) {
                *slot = Key {
                    max_frame: u16::from_be_bytes([chunk[0], chunk[1]]),
                    offset: u16::from_be_bytes([chunk[2], chunk[3]]),
                    kind: u16::from_be_bytes([chunk[4], chunk[5]]),
                };
            }
            Ok(ColorEntry { channels })
        })
        .collect()
}

fn values(data: &[u8], block: usize, end: usize, offset: usize) -> Result<Vec<u8>, BpkError> {
    let start = relative(block, offset, end)?;
    data.get(start..end)
        .map(<[u8]>::to_vec)
        .ok_or(BpkError::Block)
}

fn read_u32_at_end(data: &[u8], at: usize, end: usize) -> Result<usize, BpkError> {
    if at + 4 > end {
        return Err(BpkError::Block);
    }
    Ok(u32::from_be_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]]) as usize)
}

fn names(
    data: &[u8],
    block: usize,
    end: usize,
    offset: usize,
    count: usize,
) -> Result<Vec<String>, BpkError> {
    let start = relative(block, offset, end)?;
    let table = read_u32_at_end(data, start, end)?;
    let base = start.checked_add(table).ok_or(BpkError::Range)?;
    (0..count)
        .map(|index| {
            let entry = base + index * 8;
            let text_offset = read_u32_at_end(data, entry + 4, end)?;
            let text_start = base.checked_add(text_offset).ok_or(BpkError::Range)?;
            let text_end = data
                .get(text_start..end)
                .ok_or(BpkError::Truncated)?
                .iter()
                .position(|&byte| byte == 0)
                .map(|len| text_start + len)
                .ok_or(BpkError::Block)?;
            Ok(
                String::from_utf8_lossy(data.get(text_start..text_end).ok_or(BpkError::Truncated)?)
                    .into_owned(),
            )
        })
        .collect()
}

fn interpolate(values: &[u8], key: &Key, frame: f32) -> u8 {
    let count = usize::from(key.max_frame);
    let base = usize::from(key.offset);
    let value = |index: usize| f32::from(values.get(base + index).copied().unwrap_or(0));
    if frame < value(0) {
        return value(1) as u8;
    }
    let last = KEY_STRIDE * (count - 1);
    if value(last) <= frame {
        return value(last + 1) as u8;
    }
    let mut start = 0;
    let mut remaining = count;
    while remaining > 1 {
        let mid = remaining / 2;
        if frame >= value(start + KEY_STRIDE * mid) {
            start += KEY_STRIDE * mid;
            remaining -= mid;
        } else {
            remaining = mid;
        }
    }
    let (t0, v0) = (value(start), value(start + 1));
    let (t1, v1) = (value(start + KEY_STRIDE), value(start + KEY_STRIDE + 1));
    let mix = if t1 == t0 {
        0.0
    } else {
        (frame - t0) / (t1 - t0)
    };
    (v0 + (v1 - v0) * mix).clamp(0.0, 255.0) as u8
}

pub fn parse(data: &[u8]) -> Result<AnmColor, BpkError> {
    if data.len() < HEADER_LEN {
        return Err(BpkError::Truncated);
    }
    if &data[0..8] != FILE_MAGIC {
        return Err(BpkError::Magic);
    }
    if read_u32_at_end(data, 8, data.len())? != 1 {
        return Err(BpkError::Block);
    }
    let block = HEADER_LEN;
    if data.get(block..block + 4) != Some(&BLOCK_TAG[..]) {
        return Err(BpkError::Block);
    }
    let block_size = read_u32_at_end(data, block + 4, data.len())?;
    if block_size < BLOCK_MIN || block + block_size > data.len() {
        return Err(BpkError::Block);
    }
    let end = block + block_size;
    let frame_max = i16::from_be_bytes([data[block + FRAME_MAX], data[block + FRAME_MAX + 1]]);
    if frame_max < 0 {
        return Err(BpkError::Block);
    }
    let attribute = data[block + ATTRIBUTE];
    let count = usize::from(u16_at(data, block + MATERIAL_NUM)?);
    let entries = table(data, block, end, i32_at(data, block + TABLE)?, count)?;
    let material_ids = (0..count)
        .map(|index| {
            u16_at(
                data,
                block + i32_at(data, block + MATERIAL_IDS)? + index * 2,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let names = names(data, block, end, i32_at(data, block + NAME_TABLE)?, count)?;
    let mut channels = Vec::new();
    for offset in VALUES {
        channels.push(values(data, block, end, i32_at(data, block + offset)?)?);
    }
    let mut value_sets = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];
    for (slot, channel) in value_sets.iter_mut().zip(channels) {
        *slot = channel;
    }
    Ok(AnmColor {
        frame_max: frame_max as u16,
        attribute,
        entries,
        values: value_sets,
        names,
        material_ids,
    })
}
