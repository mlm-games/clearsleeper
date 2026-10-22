#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TexFmt {
    I4,
    I8,
    Ia4,
    Ia8,
    Rgb565,
    Rgb5a3,
    Rgba8,
    C4,
    C8,
    Cmpr,
    Unknown,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TlutFmt {
    Ia8,
    Rgb565,
    Rgb5a3,
    Unknown,
}

const FILTER_LINEAR: u8 = 1;
const WRAP_REPEAT: u8 = 1;
const WRAP_MIRROR: u8 = 2;

pub struct Image {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u8>,
    pub mag_linear: bool,
    pub wrap_s: u8,
    pub wrap_t: u8,
    pub alpha_enabled: bool,
}

pub fn tex_fmt(code: u8) -> TexFmt {
    match code {
        0x0 => TexFmt::I4,
        0x1 => TexFmt::I8,
        0x2 => TexFmt::Ia4,
        0x3 => TexFmt::Ia8,
        0x4 => TexFmt::Rgb565,
        0x5 => TexFmt::Rgb5a3,
        0x6 => TexFmt::Rgba8,
        0x8 => TexFmt::C4,
        0x9 => TexFmt::C8,
        0xE => TexFmt::Cmpr,
        _ => TexFmt::Unknown,
    }
}

fn tlut_fmt(code: u8) -> TlutFmt {
    match code {
        0x0 => TlutFmt::Ia8,
        0x1 => TlutFmt::Rgb565,
        0x2 => TlutFmt::Rgb5a3,
        _ => TlutFmt::Unknown,
    }
}

fn expand(value: u8, bits: u32) -> u8 {
    match bits {
        3 => ((u16::from(value) << 5) | (u16::from(value) << 2) | (u16::from(value) >> 1)) as u8,
        _ => ((u16::from(value) << (8 - bits)) | (u16::from(value) >> (bits * 2 - 8))) as u8,
    }
}

fn rgb565(texel: u16) -> [u8; 4] {
    [
        expand(((texel >> 11) & 0x1F) as u8, 5),
        expand(((texel >> 5) & 0x3F) as u8, 6),
        expand((texel & 0x1F) as u8, 5),
        0xFF,
    ]
}

fn rgb5a3(texel: u16) -> [u8; 4] {
    if texel & 0x8000 != 0 {
        [
            expand(((texel >> 10) & 0x1F) as u8, 5),
            expand(((texel >> 5) & 0x1F) as u8, 5),
            expand((texel & 0x1F) as u8, 5),
            0xFF,
        ]
    } else {
        [
            expand(((texel >> 8) & 0xF) as u8, 4),
            expand(((texel >> 4) & 0xF) as u8, 4),
            expand((texel & 0xF) as u8, 4),
            expand(((texel >> 12) & 0x7) as u8, 3),
        ]
    }
}

struct Layout {
    frac: usize,
    texel_bytes: usize,
    block_width: usize,
    block_height: usize,
}

fn layout(format: TexFmt) -> Option<Layout> {
    let (frac, texel_bytes, block_width, block_height) = match format {
        TexFmt::I4 | TexFmt::C4 => (2, 1, 8, 8),
        TexFmt::I8 | TexFmt::C8 | TexFmt::Ia4 => (1, 1, 8, 4),
        TexFmt::Ia8 | TexFmt::Rgb565 | TexFmt::Rgb5a3 => (1, 2, 4, 4),
        TexFmt::Rgba8 => (1, 4, 1, 1),
        _ => return None,
    };
    Some(Layout {
        frac,
        texel_bytes,
        block_width,
        block_height,
    })
}

struct Header {
    format: TexFmt,
    width: usize,
    height: usize,
    tlut: TlutFmt,
    num_colors: usize,
    palette_offset: usize,
    image_offset: usize,
    mip_count: usize,
    mag_linear: bool,
    wrap_s: u8,
    wrap_t: u8,
    alpha_enabled: bool,
}

fn header(src: &[u8]) -> Option<Header> {
    if src.len() < 0x20 {
        return None;
    }
    let be16 = |at: usize| u16::from_be_bytes([src[at], src[at + 1]]) as usize;
    let be32 =
        |at: usize| u32::from_be_bytes([src[at], src[at + 1], src[at + 2], src[at + 3]]) as usize;
    let format = tex_fmt(src[0]);
    layout(format)?;
    Some(Header {
        format,
        width: be16(0x02),
        height: be16(0x04),
        tlut: tlut_fmt(src[0x09]),
        num_colors: be16(0x0A),
        palette_offset: be32(0x0C),
        image_offset: be32(0x1C),
        mip_count: usize::from(src[0x18]).max(1),
        mag_linear: src[0x15] == FILTER_LINEAR,
        wrap_s: src[0x06],
        wrap_t: src[0x07],
        alpha_enabled: src[0x01] != 0,
    })
}

fn palette(src: &[u8], head: &Header) -> Vec<[u8; 4]> {
    let mut out = Vec::with_capacity(head.num_colors);
    for index in 0..head.num_colors {
        let at = head.palette_offset + index * 2;
        let Some(bytes) = src.get(at..at + 2) else {
            break;
        };
        let texel = u16::from_be_bytes([bytes[0], bytes[1]]);
        out.push(match head.tlut {
            TlutFmt::Ia8 => {
                let intensity = (texel >> 8) as u8;
                [intensity, intensity, intensity, texel as u8]
            }
            TlutFmt::Rgb565 => rgb565(texel),
            TlutFmt::Rgb5a3 => rgb5a3(texel),
            TlutFmt::Unknown => [0, 0, 0, 0xFF],
        });
    }
    out
}

fn read_texel(src: &[u8], at: usize, format: TexFmt, x: usize) -> u32 {
    match format {
        TexFmt::I4 | TexFmt::C4 => {
            let byte = src.get(at + x / 2).copied().unwrap_or(0);
            u32::from((byte >> if x & 1 == 0 { 4 } else { 0 }) & 0x0F)
        }
        TexFmt::I8 | TexFmt::C8 | TexFmt::Ia4 => u32::from(src.get(at + x).copied().unwrap_or(0)),
        TexFmt::Rgba8 => {
            let mut bytes = [0_u8; 4];
            for (index, slot) in bytes.iter_mut().enumerate() {
                *slot = src.get(at + x * 4 + index).copied().unwrap_or(0);
            }
            u32::from_le_bytes(bytes)
        }
        _ => {
            let high = src.get(at + x * 2).copied().unwrap_or(0);
            let low = src.get(at + x * 2 + 1).copied().unwrap_or(0);
            u32::from(u16::from_be_bytes([high, low]))
        }
    }
}

fn shade(format: TexFmt, raw: u32, pal: &[[u8; 4]]) -> [u8; 4] {
    match format {
        // `J2DPicture::setTevMode` takes the picture's alpha from the texture when the
        // texture carries one (`J2DPicture.cpp:495-503`), and the intensity formats have no
        // alpha of their own: the texel's own value is the alpha
        // (`gecko/crates/gecko/src/flipper/gx/texture.rs:77-117`, `:118-156`). A zero texel is
        // therefore transparent, which is how the file select's ring and disc plates are drawn.
        TexFmt::I4 => {
            let level = expand(raw as u8, 4);
            [level, level, level, level]
        }
        TexFmt::I8 => {
            let level = raw as u8;
            [level, level, level, level]
        }
        TexFmt::Ia4 => {
            let value = raw as u8;
            [
                expand(value & 0x0F, 4),
                expand(value & 0x0F, 4),
                expand(value & 0x0F, 4),
                expand(value >> 4, 4),
            ]
        }
        TexFmt::Ia8 => {
            let value = raw as u32;
            [
                value as u8,           // intensity (low byte)
                value as u8,           // intensity (low byte)
                value as u8,           // intensity (low byte)
                (value >> 8) as u8,    // alpha (high byte)
            ]
        }
        TexFmt::Rgb565 => rgb565(raw as u16),
        TexFmt::Rgb5a3 => rgb5a3(raw as u16),
        TexFmt::Rgba8 => [
            (raw >> 24) as u8,
            (raw >> 16) as u8,
            (raw >> 8) as u8,
            raw as u8,
        ],
        TexFmt::C4 | TexFmt::C8 => pal.get(raw as usize).copied().unwrap_or([0, 0, 0, 0x00]),
        TexFmt::Cmpr | TexFmt::Unknown => [0, 0, 0, 0x00],
    }
}

pub fn decode(src: &[u8]) -> Option<Image> {
    let head = header(src)?;
    let blocks = layout(head.format)?;
    if head.width == 0 || head.height == 0 {
        return None;
    }
    let pal = palette(src, &head);
    let base = if head.image_offset == 0 {
        0x20
    } else {
        head.image_offset
    };
    let data = src.get(base..)?;

    let mut pixels = vec![0_u8; head.width * head.height * 4];
    let mut at = 0_usize;
    let mut width = head.width;
    let mut height = head.height;

    for mip in 0..head.mip_count {
        let blocks_across = width.div_ceil(blocks.block_width);
        let blocks_down = height.div_ceil(blocks.block_height);
        for block_y in 0..blocks_down {
            let rows = (height - block_y * blocks.block_height).min(blocks.block_height);
            for block_x in 0..blocks_across {
                let cols = (width - block_x * blocks.block_width).min(blocks.block_width);
                for row in 0..rows {
                    for col in 0..cols {
                        if mip == 0 {
                            let raw = read_texel(data, at, head.format, col);
                            let px = shade(head.format, raw, &pal);
                            let target = (block_y * blocks.block_height + row) * head.width
                                + block_x * blocks.block_width
                                + col;
                            pixels[target * 4..target * 4 + 4].copy_from_slice(&px);
                        }
                    }
                    at += blocks.block_width * blocks.texel_bytes / blocks.frac;
                }
                at += blocks.block_width * blocks.texel_bytes * (blocks.block_height - rows)
                    / blocks.frac;
            }
        }
        width = (width / 2).max(1);
        height = (height / 2).max(1);
    }

    Some(Image {
        width: head.width,
        height: head.height,
        pixels,
        mag_linear: head.mag_linear,
        wrap_s: head.wrap_s,
        wrap_t: head.wrap_t,
        alpha_enabled: head.alpha_enabled,
    })
}
