//! `J3D1btp1` (`.btp`) texture-pattern animation - `J3DAnmTexPattern`, the key `mDoExt_btpAnm`
//! wraps (`m_Do/m_Do_ext.h:131-149`).
//!
//! The block is `TPT1`, a `J3DAnmTexPatternFullData` (`J3DAnimation.h:247-262`) whose loader is
//! `J3DAnmFullLoader_v15::setAnmTexPattern` (`J3DAnmLoader.cpp:300-309`): a per-material table of
//! `{maxFrame, offset, texNo}` entries over one flat `u16` array of texture indices. Sampling is
//! `J3DAnmTexPattern::getTexNo` (`J3DAnimation.cpp:840-849`): a plain frame-plus-offset index
//! clamped at the entry's ends, with the `(s32)` truncation and no interpolation. The material a
//! table entry updates is resolved by name at init time
//! (`J3DAnmTexPattern::searchUpdateMaterialID`, `:862-871`), not read from the file's own
//! pre-resolved id array.

const HEADER_LEN: usize = 0x20;
const FILE_MAGIC: &[u8; 8] = b"J3D1btp1";
const BLOCK_TAG: &[u8; 4] = b"TPT1";
const BLOCK_MIN: usize = 0x20;
const ATTRIBUTE: usize = 0x08;
const FRAME_MAX: usize = 0x0A;
const MATERIAL_NUM: usize = 0x0C;
const VALUE_NUM: usize = 0x0E;
const TABLE: usize = 0x10;
const VALUES: usize = 0x14;
const MATERIAL_IDS: usize = 0x18;
const NAME_TABLE: usize = 0x1C;
/// `J3DAnmTexPatternFullTable` (`J3DAnimation.h:46-52`).
const ENTRY_STRIDE: usize = 8;
const ENTRY_MAX_FRAME: usize = 0x00;
const ENTRY_OFFSET: usize = 0x02;
const ENTRY_TEX_NO: usize = 0x04;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BtpError {
    Truncated,
    Magic,
    Block,
    Range,
}

impl BtpError {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Truncated => "btp data truncated",
            Self::Magic => "btp magic mismatch",
            Self::Block => "btp block table invalid",
            Self::Range => "btp key index out of range",
        }
    }
}

/// One `J3DAnmTexPatternFullTable` entry: the key span of one material's texture index and the
/// texture unit it writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entry {
    pub max_frame: u16,
    pub offset: u16,
    /// `mTexNo`, the material's texture unit the key updates. `J3DMaterialAnm::calc` writes it
    /// through `pMaterial->getTevBlock()->setTexNo(i, texNo)` (`J3DMaterialAnm.cpp:60`).
    pub tex_no: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TexPattern {
    /// `J3DAnmBase::getFrameMax()`, the frame count `mDoExt_baseAnm::initPlay` takes
    /// (`m_Do_ext.cpp:105`).
    pub frame_max: u16,
    /// `J3DAnmBase::mAttribute`, the file's own `field_0x8`.
    pub attribute: u8,
    pub entries: Vec<Entry>,
    /// `mTextureIndex`: the flat key array every entry indexes into.
    pub texture_index: Vec<u16>,
    /// `mUpdateMaterialName`, the material names the entries key on.
    pub names: Vec<String>,
    /// `mUpdateMaterialID`, the file's own pre-resolved material ids.
    /// [`TexPattern::resolve`] overwrites them the way `searchUpdateMaterialID` does.
    pub material_ids: Vec<u16>,
}

impl TexPattern {
    /// `J3DAnmTexPattern::getTexNo` - `J3DAnimation.cpp:840-849`: the index at the (truncated)
    /// frame plus the entry's offset, clamped to the entry's first and last key. An entry with no
    /// keys of its own answers its first index, where the reference's `maxFrame - 1` underflows
    /// into a read past the array.
    pub fn tex_no(&self, entry: usize, frame: f32) -> Option<u16> {
        let entry = self.entries.get(entry)?;
        let max_frame = f32::from(entry.max_frame);
        let offset = usize::from(entry.offset);
        let index = if (0.0..max_frame).contains(&frame) {
            offset + frame as usize
        } else if frame < 0.0 {
            offset
        } else {
            offset + usize::from(entry.max_frame.saturating_sub(1))
        };
        self.texture_index.get(index).copied()
    }

    /// `J3DAnmTexPattern::searchUpdateMaterialID` - `J3DAnimation.cpp:862-871`: each entry's name
    /// against the model's material name table, `0xFFFF` for a miss. The reference runs this from
    /// `mDoExt_btpAnm::init` (`m_Do_ext.cpp:199`) and writes the answer over the file's own ids.
    pub fn resolve(&mut self, material: impl Fn(&str) -> Option<u16>) {
        for (index, name) in self.names.iter().enumerate() {
            if let Some(slot) = self.material_ids.get_mut(index) {
                *slot = material(name).unwrap_or(u16::MAX);
            }
        }
    }
}

pub fn parse(data: &[u8]) -> Result<TexPattern, BtpError> {
    if data.len() < HEADER_LEN {
        return Err(BtpError::Truncated);
    }
    if &data[0..8] != FILE_MAGIC {
        return Err(BtpError::Magic);
    }
    if read_u32(data, 8)? as usize > data.len() {
        return Err(BtpError::Truncated);
    }
    if read_u32(data, 12)? != 1 {
        return Err(BtpError::Block);
    }
    let block = HEADER_LEN;
    let tag = data.get(block..block + 4).ok_or(BtpError::Truncated)?;
    if tag != BLOCK_TAG {
        return Err(BtpError::Block);
    }
    let block_size = read_u32(data, block + 4)? as usize;
    if block_size < BLOCK_MIN || block + block_size > data.len() {
        return Err(BtpError::Block);
    }
    let end = block + block_size;
    let frame_max = read_i16(data, block + FRAME_MAX)?;
    if frame_max < 0 {
        return Err(BtpError::Block);
    }
    let attribute = *data.get(block + ATTRIBUTE).ok_or(BtpError::Truncated)?;
    let material_num = usize::from(read_u16(data, block + MATERIAL_NUM)?);
    let value_num = usize::from(read_u16(data, block + VALUE_NUM)?);

    let table_at = table(data, block, end, TABLE, material_num * ENTRY_STRIDE)?;
    let mut entries = Vec::with_capacity(material_num);
    for index in 0..material_num {
        let entry = at(table_at, index * ENTRY_STRIDE)?;
        if entry + ENTRY_STRIDE > end {
            return Err(BtpError::Block);
        }
        entries.push(Entry {
            max_frame: read_u16(data, entry + ENTRY_MAX_FRAME)?,
            offset: read_u16(data, entry + ENTRY_OFFSET)?,
            tex_no: *data.get(entry + ENTRY_TEX_NO).ok_or(BtpError::Truncated)?,
        });
    }

    let values_at = table(data, block, end, VALUES, value_num * 2)?;
    let mut texture_index = Vec::with_capacity(value_num);
    for index in 0..value_num {
        texture_index.push(read_u16(data, at(values_at, index * 2)?)?);
    }
    for entry in &entries {
        let offset = usize::from(entry.offset);
        if offset + usize::from(entry.max_frame) > value_num {
            return Err(BtpError::Range);
        }
    }

    let ids_at = table(data, block, end, MATERIAL_IDS, material_num * 2)?;
    let mut material_ids = Vec::with_capacity(material_num);
    for index in 0..material_num {
        material_ids.push(read_u16(data, at(ids_at, index * 2)?)?);
    }

    let material_names = names(data, block, end, NAME_TABLE, material_num)?;
    Ok(TexPattern {
        frame_max: frame_max as u16,
        attribute,
        entries,
        texture_index,
        names: material_names,
        material_ids,
    })
}

fn at(base: usize, offset: usize) -> Result<usize, BtpError> {
    base.checked_add(offset).ok_or(BtpError::Block)
}

/// A block-relative table offset, resolved the way the loader resolves it: zero means the table is
/// absent, and any other value is a byte offset from the block start.
fn table(
    data: &[u8],
    block: usize,
    end: usize,
    field: usize,
    bytes: usize,
) -> Result<usize, BtpError> {
    if bytes == 0 {
        return Ok(end);
    }
    let rel = read_u32(data, at(block, field)?)? as usize;
    if rel < BLOCK_MIN {
        return Err(BtpError::Block);
    }
    let start = at(block, rel)?;
    if at(start, bytes)? > end {
        return Err(BtpError::Block);
    }
    Ok(start)
}

/// `JUTNameTab` over a `ResNTAB` (`JUTNameTab.h:6-17`): a `u16` count, then `{keyCode, nameOffset}`
/// pairs whose offsets are measured from the table's own start.
fn names(
    data: &[u8],
    block: usize,
    end: usize,
    field: usize,
    count: usize,
) -> Result<Vec<String>, BtpError> {
    if count == 0 {
        return Ok(Vec::new());
    }
    let base = table(data, block, end, field, 4 + count * 4)?;
    if usize::from(read_u16(data, base)?) < count {
        return Err(BtpError::Block);
    }
    let mut names = Vec::with_capacity(count);
    for index in 0..count {
        let entry = at(base, 4 + index * 4)?;
        let offset = usize::from(read_u16(data, entry + 2)?);
        let string = at(base, offset)?;
        if string >= end {
            return Err(BtpError::Block);
        }
        let bytes = data.get(string..end).ok_or(BtpError::Truncated)?;
        let length = bytes
            .iter()
            .position(|&byte| byte == 0)
            .ok_or(BtpError::Block)?;
        names.push(
            std::str::from_utf8(&bytes[..length])
                .map_err(|_| BtpError::Block)?
                .to_owned(),
        );
    }
    Ok(names)
}

fn read_u16(data: &[u8], start: usize) -> Result<u16, BtpError> {
    let end = start.checked_add(2).ok_or(BtpError::Block)?;
    let bytes = data.get(start..end).ok_or(BtpError::Truncated)?;
    Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
}

fn read_i16(data: &[u8], start: usize) -> Result<i16, BtpError> {
    let end = start.checked_add(2).ok_or(BtpError::Block)?;
    let bytes = data.get(start..end).ok_or(BtpError::Truncated)?;
    Ok(i16::from_be_bytes([bytes[0], bytes[1]]))
}

fn read_u32(data: &[u8], start: usize) -> Result<u32, BtpError> {
    let end = start.checked_add(4).ok_or(BtpError::Block)?;
    let bytes = data.get(start..end).ok_or(BtpError::Truncated)?;
    Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}
