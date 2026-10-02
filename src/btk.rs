const HEADER_LEN: usize = 0x20;
const FILE_MAGIC: &[u8; 8] = b"J3D1btk1";
const BLOCK_TAG: &[u8; 4] = b"TTK1";
const BLOCK_MIN: usize = 0x60;
const ENTRY_STRIDE: usize = 0x12;
const KEY_LEN: usize = 6;
const FRAME_MAX: usize = 0x0A;
const TRACK_NUM: usize = 0x0C;
const SCALE_NUM: usize = 0x0E;
const ROTATION_NUM: usize = 0x10;
const TRANSLATION_NUM: usize = 0x12;
const DEC_SHIFT: usize = 0x09;
const KEY_TABLE: usize = 0x14;
const NAME_TABLE: usize = 0x1C;
const TEXTURE_MATRIX_TABLE: usize = 0x20;
const CENTER_TABLE: usize = 0x24;
const SCALE_TABLE: usize = 0x28;
const ROTATION_TABLE: usize = 0x2C;
const TRANSLATION_TABLE: usize = 0x30;
const POST_TRACKS: usize = 0x34;
const CALC_TYPE: usize = 0x5C;
const MAX_DEC_SHIFT: u8 = 15;
const CHANNELS_PER_TRACK: usize = 3;
const SINCOS_BITS: u32 = 12;
const SINCOS_LEN: usize = 1 << SINCOS_BITS;
const SINCOS_COS_BIAS: usize = SINCOS_LEN / 4;
const SINCOS_SHIFT: u32 = 16 - SINCOS_BITS;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BtkError {
    Truncated,
    Magic,
    Block,
    Descriptor,
    Range,
}

impl BtkError {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Truncated => "btk data truncated",
            Self::Magic => "btk magic mismatch",
            Self::Block => "btk block table invalid",
            Self::Descriptor => "btk descriptor unsupported",
            Self::Range => "btk index out of range",
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Key {
    pub max_frame: u16,
    pub offset: u16,
    pub kind: u16,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Entry {
    pub scale: Key,
    pub rotation: Key,
    pub translation: Key,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TexSrt {
    pub scale_x: f32,
    pub scale_y: f32,
    pub rotation: i16,
    pub translation_x: f32,
    pub translation_y: f32,
}

impl Default for TexSrt {
    fn default() -> Self {
        Self {
            scale_x: 1.0,
            scale_y: 1.0,
            rotation: 0,
            translation_x: 0.0,
            translation_y: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UvMatrix {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub tx: f32,
    pub ty: f32,
}

#[derive(Debug, Clone)]
pub struct TextureAnimation {
    pub frame_max: u16,
    pub dec_shift: u8,
    pub texture_matrix_ids: Vec<u8>,
    pub centers: Vec<[f32; 3]>,
    pub names: Vec<String>,
    pub scale_values: Vec<f32>,
    pub rotation_values: Vec<i16>,
    pub translation_values: Vec<f32>,
    pub entries: Vec<Entry>,
}

impl TextureAnimation {
    pub fn track_count(&self) -> usize {
        self.entries.len() / CHANNELS_PER_TRACK
    }

    pub fn frame_at(&self, tick: u64) -> f32 {
        (tick % u64::from(self.frame_max)) as f32
    }

    pub fn track(&self, material: &str, matrix: u8) -> Option<u32> {
        let mut track = None;
        for index in 0..self.track_count() {
            let name = self.names.get(index).map(String::as_str) == Some(material);
            let slot = self.texture_matrix_ids.get(index) == Some(&matrix);
            if name && slot {
                track = Some(index as u32);
            }
        }
        track
    }

    pub fn has_track(&self, material: &str, matrix: u8) -> bool {
        self.track(material, matrix).is_some()
    }

    pub fn center(&self, track: u32) -> Option<[f32; 3]> {
        usize::try_from(track)
            .ok()
            .and_then(|index| self.centers.get(index))
            .copied()
    }

    pub fn eval(&self, track: u32, frame: f32) -> TexSrt {
        let slots = usize::try_from(track)
            .ok()
            .and_then(|track| track.checked_mul(CHANNELS_PER_TRACK))
            .and_then(|base| self.entries.get(base..))
            .and_then(|slice| slice.get(..CHANNELS_PER_TRACK));
        let Some([x, y, rotation]) = slots else {
            return TexSrt::default();
        };
        TexSrt {
            scale_x: channel(&x.scale, &self.scale_values, frame, 1.0),
            scale_y: channel(&y.scale, &self.scale_values, frame, 1.0),
            rotation: rotation_channel(&rotation.rotation, &self.rotation_values, frame, self.dec_shift),
            translation_x: channel(&x.translation, &self.translation_values, frame, 0.0),
            translation_y: channel(&y.translation, &self.translation_values, frame, 0.0),
        }
    }

    pub fn uv_transform(&self, track: u32, frame: f32) -> Option<UvMatrix> {
        let center = self.center(track)?;
        Some(texture_matrix(&self.eval(track, frame), center))
    }
}

pub fn parse(data: &[u8]) -> Result<TextureAnimation, BtkError> {
    if data.len() < HEADER_LEN {
        return Err(BtkError::Truncated);
    }
    if &data[0..8] != FILE_MAGIC {
        return Err(BtkError::Magic);
    }
    if read_u32(data, 8)? as usize > data.len() {
        return Err(BtkError::Truncated);
    }
    if read_u32(data, 12)? != 1 {
        return Err(BtkError::Block);
    }
    let block = HEADER_LEN;
    let tag = data.get(block..block + 4).ok_or(BtkError::Truncated)?;
    if tag != BLOCK_TAG {
        return Err(BtkError::Block);
    }
    let block_size = read_u32(data, block + 4)? as usize;
    if block_size < BLOCK_MIN || block + block_size > data.len() {
        return Err(BtkError::Block);
    }
    let end = block + block_size;
    let dec_shift = *data.get(block + DEC_SHIFT).ok_or(BtkError::Truncated)?;
    if dec_shift > MAX_DEC_SHIFT {
        return Err(BtkError::Descriptor);
    }
    let frame_max = read_i16(data, block + FRAME_MAX)?;
    if frame_max < 1 {
        return Err(BtkError::Block);
    }
    let track_num = read_u16(data, block + TRACK_NUM)?;
    if track_num % CHANNELS_PER_TRACK as u16 != 0 {
        return Err(BtkError::Block);
    }
    let track_count = usize::from(track_num / CHANNELS_PER_TRACK as u16);
    if read_u16(data, block + POST_TRACKS)? != 0 {
        return Err(BtkError::Descriptor);
    }
    if read_u32(data, block + CALC_TYPE)? == 1 {
        return Err(BtkError::Descriptor);
    }

    let scale_num = usize::from(read_u16(data, block + SCALE_NUM)?);
    let rotation_num = usize::from(read_u16(data, block + ROTATION_NUM)?);
    let translation_num = usize::from(read_u16(data, block + TRANSLATION_NUM)?);

    let scale_at = table(data, block, end, SCALE_TABLE, scale_num * 4)?;
    let mut scale_values = Vec::with_capacity(scale_num);
    for index in 0..scale_num {
        scale_values.push(read_f32(data, at(scale_at, index * 4)?)?);
    }
    let rotation_at = table(data, block, end, ROTATION_TABLE, rotation_num * 2)?;
    let mut rotation_values = Vec::with_capacity(rotation_num);
    for index in 0..rotation_num {
        rotation_values.push(read_i16(data, at(rotation_at, index * 2)?)?);
    }
    let translation_at = table(data, block, end, TRANSLATION_TABLE, translation_num * 4)?;
    let mut translation_values = Vec::with_capacity(translation_num);
    for index in 0..translation_num {
        translation_values.push(read_f32(data, at(translation_at, index * 4)?)?);
    }

    let center_at = table(data, block, end, CENTER_TABLE, track_count * 12)?;
    let mut centers = Vec::with_capacity(track_count);
    for index in 0..track_count {
        let at = at(center_at, index * 12)?;
        centers.push([
            read_f32(data, at)?,
            read_f32(data, at + 4)?,
            read_f32(data, at + 8)?,
        ]);
    }

    let name_at = table(data, block, end, NAME_TABLE, 4)?;
    if usize::from(read_u16(data, name_at)?) < track_count {
        return Err(BtkError::Block);
    }
    if name_at.checked_add(4 + track_count * 4).ok_or(BtkError::Block)? > end {
        return Err(BtkError::Block);
    }
    let mut names = Vec::with_capacity(track_count);
    for index in 0..track_count {
        let entry = name_at + 4 + index * 4;
        let offset = usize::from(read_u16(data, entry + 2)?);
        let string = name_at.checked_add(offset).ok_or(BtkError::Block)?;
        if string >= end {
            return Err(BtkError::Block);
        }
        let bytes = data.get(string..end).ok_or(BtkError::Truncated)?;
        let length = bytes
            .iter()
            .position(|&byte| byte == 0)
            .ok_or(BtkError::Block)?;
        names.push(std::str::from_utf8(&bytes[..length]).map_err(|_| BtkError::Block)?.to_owned());
    }

    let matrix_rel = read_u32(data, block + TEXTURE_MATRIX_TABLE)? as usize;
    let texture_matrix_ids = if matrix_rel == 0 {
        vec![0; track_count]
    } else {
        let start = table(data, block, end, TEXTURE_MATRIX_TABLE, track_count)?;
        data[start..start + track_count].to_vec()
    };

    let key_at = table(data, block, end, KEY_TABLE, usize::from(track_num) * ENTRY_STRIDE)?;
    let mut entries = Vec::with_capacity(usize::from(track_num));
    for index in 0..usize::from(track_num) {
        let slot = at(key_at, index * ENTRY_STRIDE)?;
        entries.push(Entry {
            scale: read_key(data, slot)?,
            rotation: read_key(data, slot + KEY_LEN)?,
            translation: read_key(data, slot + 2 * KEY_LEN)?,
        });
    }
    for entry in &entries {
        validate(&entry.scale, scale_values.len())?;
        validate(&entry.rotation, rotation_values.len())?;
        validate(&entry.translation, translation_values.len())?;
    }

    Ok(TextureAnimation {
        frame_max: frame_max as u16,
        dec_shift,
        texture_matrix_ids,
        centers,
        names,
        scale_values,
        rotation_values,
        translation_values,
        entries,
    })
}

fn at(base: usize, offset: usize) -> Result<usize, BtkError> {
    base.checked_add(offset).ok_or(BtkError::Block)
}

fn table(data: &[u8], block: usize, end: usize, field: usize, bytes: usize) -> Result<usize, BtkError> {
    if bytes == 0 {
        return Ok(end);
    }
    let rel = read_u32(data, at(block, field)?)? as usize;
    if rel < BLOCK_MIN {
        return Err(BtkError::Block);
    }
    let start = at(block, rel)?;
    if at(start, bytes)? > end {
        return Err(BtkError::Block);
    }
    Ok(start)
}

fn read_u16(data: &[u8], start: usize) -> Result<u16, BtkError> {
    let end = start.checked_add(2).ok_or(BtkError::Block)?;
    let bytes = data.get(start..end).ok_or(BtkError::Truncated)?;
    Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
}

fn read_i16(data: &[u8], start: usize) -> Result<i16, BtkError> {
    let end = start.checked_add(2).ok_or(BtkError::Block)?;
    let bytes = data.get(start..end).ok_or(BtkError::Truncated)?;
    Ok(i16::from_be_bytes([bytes[0], bytes[1]]))
}

fn read_u32(data: &[u8], start: usize) -> Result<u32, BtkError> {
    let end = start.checked_add(4).ok_or(BtkError::Block)?;
    let bytes = data.get(start..end).ok_or(BtkError::Truncated)?;
    Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn read_f32(data: &[u8], start: usize) -> Result<f32, BtkError> {
    let end = start.checked_add(4).ok_or(BtkError::Block)?;
    let bytes = data.get(start..end).ok_or(BtkError::Truncated)?;
    Ok(f32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn read_key(data: &[u8], start: usize) -> Result<Key, BtkError> {
    Ok(Key {
        max_frame: read_u16(data, start)?,
        offset: read_u16(data, start + 2)?,
        kind: read_u16(data, start + 4)?,
    })
}

fn stride(kind: u16) -> usize {
    if kind == 0 {
        3
    } else {
        4
    }
}

fn validate(key: &Key, values: usize) -> Result<(), BtkError> {
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
        return Err(BtkError::Range);
    }
    Ok(())
}

fn channel(key: &Key, values: &[f32], frame: f32, constant: f32) -> f32 {
    match key.max_frame {
        0 => constant,
        1 => values
            .get(usize::from(key.offset))
            .copied()
            .unwrap_or(constant),
        _ => interpolate(frame, key, |index| {
            values.get(index).copied().unwrap_or(constant)
        }),
    }
}

fn rotation_channel(key: &Key, values: &[i16], frame: f32, dec_shift: u8) -> i16 {
    let quantize = |value: f32| ((value as i32) << dec_shift) as i16;
    match key.max_frame {
        0 => 0,
        1 => values
            .get(usize::from(key.offset))
            .map(|&value| ((i32::from(value)) << dec_shift) as i16)
            .unwrap_or(0),
        _ => quantize(interpolate(frame, key, |index| {
            values.get(index).map(|&value| f32::from(value)).unwrap_or(0.0)
        })),
    }
}

fn interpolate(frame: f32, key: &Key, at: impl Fn(usize) -> f32) -> f32 {
    let stride = stride(key.kind);
    let count = usize::from(key.max_frame);
    let base = usize::from(key.offset);
    let value = |index: usize| at(base + index);
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

fn hermite(frame: f32, t0: f32, v0: f32, m0: f32, t1: f32, v1: f32, m1: f32) -> f32 {
    let length = t1 - t0;
    let f1 = 1.0 / length;
    let f9 = frame - t0;
    let f2 = f9 * f9 * f1;
    let f10 = f2 * f1;
    let f11 = f9 * f10;
    let f12 = f11 * f1;
    v0 * (1.0 + ((2.0 * f12) - (3.0 * f10)))
        + v1 * ((-2.0 * f12) + (3.0 * f10))
        + m0 * (f9 + (f11 - (2.0 * f2)))
        + m1 * (f11 - f2)
}

fn texture_matrix(srt: &TexSrt, center: [f32; 3]) -> UvMatrix {
    let sin = jma_sin(srt.rotation);
    let cos = jma_cos(srt.rotation);
    let cx = srt.scale_x * cos;
    let sx = srt.scale_x * sin;
    let sy = srt.scale_y * sin;
    let cy = srt.scale_y * cos;
    UvMatrix {
        a: cx,
        b: -sx,
        c: sy,
        d: cy,
        tx: (-cx * center[0] + sx * center[1]) + center[0] + srt.translation_x,
        ty: (-sy * center[0] - cy * center[1]) + center[1] + srt.translation_y,
    }
}

fn jma_sample(index: usize) -> f32 {
    let step = std::f64::consts::TAU / SINCOS_LEN as f64;
    ((step * index as f64).sin()) as f32
}

fn jma_sin(angle: i16) -> f32 {
    jma_sample(usize::from((angle as u16) >> SINCOS_SHIFT))
}

fn jma_cos(angle: i16) -> f32 {
    jma_sample(SINCOS_COS_BIAS + usize::from((angle as u16) >> SINCOS_SHIFT))
}
