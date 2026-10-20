use bevy_ecs::prelude::Resource;

use super::fontdata;
use super::{FontSlot, GLY1, Gly1, INF1, Inf1, MAP1, Map1, Metrics, WID1, Wid1, be16, be32};

const SJIS_LEAD_DEFAULT: u16 = 0x31C;
const GLYPH_HEADER_SIZE: usize = 0x20;
const CACHE_PAGE_HEADER: u32 = 0x40;
const TEX_WRAP_CLAMP: u32 = 0;
const TEX_ANISO_1: u32 = 1;

const HALF_TO_FULL: [u16; 95] = [
    0x8140, 0x8149, 0x8168, 0x8194, 0x8190, 0x8193, 0x8195, 0x8166, 0x8169, 0x816A, 0x8196, 0x817B,
    0x8143, 0x817C, 0x8144, 0x815E, 0x824F, 0x8250, 0x8251, 0x8252, 0x8253, 0x8254, 0x8255, 0x8256,
    0x8257, 0x8258, 0x8146, 0x8147, 0x8183, 0x8181, 0x8184, 0x8148, 0x8197, 0x8260, 0x8261, 0x8262,
    0x8263, 0x8264, 0x8265, 0x8266, 0x8267, 0x8268, 0x8269, 0x826A, 0x826B, 0x826C, 0x826D, 0x826E,
    0x826F, 0x8270, 0x8271, 0x8272, 0x8273, 0x8274, 0x8275, 0x8276, 0x8277, 0x8278, 0x8279, 0x816D,
    0x818F, 0x816E, 0x814F, 0x8151, 0x8165, 0x8281, 0x8282, 0x8283, 0x8284, 0x8285, 0x8286, 0x8287,
    0x8288, 0x8289, 0x828A, 0x828B, 0x828C, 0x828D, 0x828E, 0x828F, 0x8290, 0x8291, 0x8292, 0x8293,
    0x8294, 0x8295, 0x8296, 0x8297, 0x8298, 0x8299, 0x829A, 0x816F, 0x8162, 0x8170, 0x8160,
];

fn safe_div(a: i32, b: i32) -> i32 {
    if b == 0 { 0 } else { a / b }
}

fn safe_rem(a: i32, b: i32) -> i32 {
    if b == 0 { 0 } else { a % b }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TWidth {
    pub a: u8,
    pub b: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LeadByte {
    #[default]
    OneByte,
    TwoByte,
    ShiftJis,
}

impl LeadByte {
    pub fn from_font_type(font_type: u16) -> Self {
        match font_type {
            1 => Self::TwoByte,
            2 => Self::ShiftJis,
            _ => Self::OneByte,
        }
    }

    pub fn test(self, chr: i32) -> bool {
        match self {
            Self::OneByte => false,
            Self::TwoByte => true,
            Self::ShiftJis => (0x81..=0x9F).contains(&chr) || (0xE0..=0xFC).contains(&chr),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TexObj {
    pub data_at: usize,
    pub data_size: u32,
    pub width: u16,
    pub height: u16,
    pub format: u16,
    pub wrap_s: u32,
    pub wrap_t: u32,
    pub mipmap: bool,
    pub min_lod: f32,
    pub max_lod: f32,
    pub lod_bias: f32,
    pub max_anisotropy: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CharQuad {
    pub advance: f32,
    pub pos: [[f32; 2]; 4],
    pub color: [u32; 4],
    pub uv: [[i32; 2]; 4],
    /// The glyph's own page: a font spreads its cells over several GLY1 pages, and a run
    /// that mixes pages must sample each quad from the page it was measured on.
    pub tex: TexObj,
}

#[derive(Clone, Debug)]
pub struct ResFont {
    pub valid: bool,
    pub fixed: bool,
    pub fixed_width: i32,
    pub color1: u32,
    pub color2: u32,
    pub color3: u32,
    pub color4: u32,
    pub cell_x: i32,
    pub cell_y: i32,
    pub tex_page_idx: i32,
    pub glyph_block: u16,
    pub max_code: u16,
    pub lead_byte: LeadByte,
    pub info: Inf1,
    pub glyph: Gly1,
    pub data: Vec<u8>,
    pub wid1_at: usize,
    pub map1_at: usize,
    pub gly1_at: usize,
    pub wid_block_num: u16,
    pub glyph_block_num: u16,
    pub map_block_num: u16,
    pub tex: TexObj,
}

impl Default for ResFont {
    fn default() -> Self {
        Self::new()
    }
}

impl ResFont {
    pub fn new() -> Self {
        Self {
            valid: false,
            fixed: false,
            fixed_width: 0,
            color1: 0,
            color2: 0,
            color3: 0,
            color4: 0,
            cell_x: 0,
            cell_y: 0,
            tex_page_idx: -1,
            glyph_block: 0,
            max_code: u16::MAX,
            lead_byte: LeadByte::OneByte,
            info: Inf1::default(),
            glyph: Gly1::default(),
            data: Vec::new(),
            wid1_at: 0,
            map1_at: 0,
            gly1_at: 0,
            wid_block_num: 0,
            glyph_block_num: 0,
            map_block_num: 0,
            tex: TexObj::default(),
        }
    }

    pub fn ascent_fix12() -> Self {
        Self::from_bytes(fontdata::ASCENT_FIX12)
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut font = Self::new();
        font.data = bytes.to_vec();
        font.info = fontdata::inf1(&font.data).unwrap_or_default();
        font.glyph = fontdata::gly1(&font.data).unwrap_or_default();
        font.wid1_at = Self::block_at(&font.data, WID1);
        font.map1_at = Self::block_at(&font.data, MAP1);
        font.gly1_at = Self::block_at(&font.data, GLY1);
        font.wid_block_num = if fontdata::wid1(&font.data).is_some() {
            1
        } else {
            0
        };
        font.glyph_block_num = if fontdata::gly1(&font.data).is_some() {
            1
        } else {
            0
        };
        font.map_block_num = if fontdata::map1(&font.data).is_some() {
            1
        } else {
            0
        };
        font.valid = true;
        font.set_block();
        font
    }

    fn block_at(bytes: &[u8], kind: u32) -> usize {
        fontdata::blocks(bytes)
            .into_iter()
            .find(|b| b.kind == kind)
            .map_or(0, |b| b.at)
    }

    pub fn set_block(&mut self) {
        self.max_code = u16::MAX;
        self.lead_byte = LeadByte::from_font_type(self.info.font_type);
        if let Some(map) = self.map_at(0) {
            if self.max_code > map.start_code {
                self.max_code = map.start_code;
            }
        }
    }

    pub fn font_type(&self) -> u16 {
        self.info.font_type
    }

    pub fn leading(&self) -> i32 {
        self.info.leading as i32
    }

    pub fn ascent(&self) -> i32 {
        self.info.ascent as i32
    }

    pub fn descent(&self) -> i32 {
        self.info.descent as i32
    }

    pub fn height(&self) -> i32 {
        self.ascent() + self.descent()
    }

    pub fn width(&self) -> i32 {
        self.info.width as i32
    }

    pub fn cell_width(&self) -> i32 {
        if self.glyph_block_num != 0 {
            return self.glyph.cell_width as i32;
        }
        self.width()
    }

    pub fn cell_height(&self) -> i32 {
        if self.glyph_block_num != 0 {
            return self.glyph.cell_height as i32;
        }
        self.height()
    }

    pub fn metrics(&self) -> Metrics {
        Metrics {
            width: self.width(),
            ascent: self.ascent(),
            descent: self.descent(),
            leading: self.leading(),
            cell_width: self.cell_width(),
            cell_height: self.cell_height(),
        }
    }

    pub fn set_char_color(&mut self, col: u32) {
        self.color1 = col;
        self.color2 = col;
        self.color3 = col;
        self.color4 = col;
    }

    pub fn set_grad_color(&mut self, col1: u32, col2: u32) {
        self.color1 = col1;
        self.color2 = col1;
        self.color3 = col2;
        self.color4 = col2;
    }

    pub fn set_fixed_width(&mut self, fixed: bool, width: i32) {
        self.fixed = fixed;
        self.fixed_width = width;
    }

    pub fn is_lead_byte(&self, chr: i32) -> bool {
        self.lead_byte.test(chr)
    }

    pub fn wid_at(&self, index: usize) -> Option<Wid1> {
        if index < self.wid_block_num as usize {
            fontdata::wid1(&self.data)
        } else {
            None
        }
    }

    pub fn map_at(&self, index: usize) -> Option<Map1> {
        if index < self.map_block_num as usize {
            fontdata::map1(&self.data)
        } else {
            None
        }
    }

    pub fn glyph_at(&self, index: usize) -> Option<Gly1> {
        if index < self.glyph_block_num as usize {
            Some(self.glyph)
        } else {
            None
        }
    }

    pub fn width_entry(&self, code: u16) -> TWidth {
        let mut width = TWidth {
            a: 0,
            b: self.info.width as u8,
        };
        let font_code = self.font_code(code as i32);
        for i in 0..self.wid_block_num as usize {
            let Some(wid) = self.wid_at(i) else { continue };
            if wid.start_code as i32 <= font_code && font_code <= wid.end_code as i32 {
                let chunk = (font_code - wid.start_code as i32) * 2;
                let at = self.wid1_at + 0x0C + chunk as usize;
                width.a = self.data.get(at).copied().unwrap_or(0);
                width.b = self.data.get(at + 1).copied().unwrap_or(0);
                break;
            }
        }
        width
    }

    pub fn char_width(&self, code: u16) -> i32 {
        self.width_entry(code).b as i32
    }

    pub fn char_offset(&self, code: u16) -> i32 {
        self.width_entry(code).a as i32
    }

    pub fn font_code(&self, chr_in: i32) -> i32 {
        let mut chr = chr_in;
        let mut ret = self.info.default_code as i32;
        if self.font_type() == 2 && self.max_code >= 0x8000 && chr >= 0x20 && chr < 0x7F {
            chr = HALF_TO_FULL
                .get((chr - 0x20) as usize)
                .copied()
                .unwrap_or_default() as i32;
        }
        for i in 0..self.map_block_num as usize {
            let Some(map) = self.map_at(i) else { continue };
            if map.start_code as i32 <= chr && chr <= map.end_code as i32 {
                match map.mapping_method {
                    0 => ret = chr - map.start_code as i32,
                    2 => {
                        let at = self.map1_at + 0x10 + 2 * (chr as usize - map.start_code as usize);
                        ret = be16(&self.data, at).unwrap_or(0) as i32;
                    }
                    3 => {
                        let mut lo = 0i32;
                        let mut hi = map.num_entries as i32 - 1;
                        while hi >= lo {
                            let mid = (hi + lo) / 2;
                            let key = be16(&self.data, self.map1_at + 0x10 + 4 * mid as usize)
                                .unwrap_or(0) as i32;
                            if chr < key {
                                hi = mid - 1;
                                continue;
                            }
                            if chr > key {
                                lo = mid + 1;
                                continue;
                            }
                            let at = self.map1_at + 0x12 + 4 * mid as usize;
                            ret = be16(&self.data, at).unwrap_or(0) as i32;
                            break;
                        }
                    }
                    1 => {
                        let lead = if map.num_entries == 1 {
                            be16(&self.data, self.map1_at + 0x10)
                        } else {
                            None
                        };
                        ret = self.convert_sjis(chr, lead);
                    }
                    _ => {}
                }
                break;
            }
        }
        ret
    }

    pub fn convert_sjis(&self, in_char: i32, in_lead: Option<u16>) -> i32 {
        let wide = in_char as u16;
        let hi = (wide >> 8) as i32;
        let mut lo = (wide & 0xFF) as i32 - 0x40;
        if lo >= 0x40 {
            lo -= 1;
        }
        let lead = match in_lead {
            Some(v) => v as i32,
            None => SJIS_LEAD_DEFAULT as i32,
        };
        lo + (hi - 0x88) * 0xBC + -0x5E + lead
    }

    pub fn load_font(&mut self, code: u16, width: Option<&mut TWidth>) {
        if let Some(dst) = width {
            let entry = self.width_entry(code);
            dst.a = entry.a;
            dst.b = entry.b;
        }
        let font_code = self.font_code(code as i32);
        self.load_image(font_code as u16);
    }

    pub fn load_image(&mut self, code_in: u16) {
        let mut code = code_in as i32;
        let mut i = 0usize;
        while i < self.glyph_block_num as usize {
            if let Some(glyph) = self.glyph_at(i) {
                if glyph.start_code as i32 <= code && code <= glyph.end_code as i32 {
                    code -= glyph.start_code as i32;
                    break;
                }
            }
            i += 1;
        }
        if i == self.glyph_block_num as usize {
            return;
        }
        let glyph = self.glyph_at(i).unwrap_or_default();
        let cell_w = glyph.cell_width.max(1) as i32;
        let cell_h = glyph.cell_height.max(1) as i32;
        let across = i32::from(glyph.texture_width) / cell_w;
        let down = i32::from(glyph.texture_height) / cell_h;
        if across < 1 || down < 1 {
            return;
        }
        let page_num_cells = across * down;
        let page_idx = safe_div(code, page_num_cells);
        let cell_idx = safe_rem(code, page_num_cells);
        let cell_col = safe_rem(cell_idx, across);
        let cell_row = safe_div(cell_idx, across);
        self.cell_x = cell_col * cell_w;
        self.cell_y = cell_row * cell_h;

        let image_bytes =
            (usize::from(glyph.texture_width) * usize::from(glyph.texture_height) / 2).max(1);
        let page_at = self
            .gly1_at
            .wrapping_add(GLYPH_HEADER_SIZE)
            .wrapping_add((page_idx as usize).wrapping_mul(image_bytes));
        if page_idx != self.tex_page_idx || i as u16 != self.glyph_block {
            self.tex = TexObj {
                data_at: page_at,
                data_size: image_bytes as u32,
                width: glyph.texture_width,
                height: glyph.texture_height,
                format: glyph.texture_format,
                wrap_s: TEX_WRAP_CLAMP,
                wrap_t: TEX_WRAP_CLAMP,
                mipmap: false,
                min_lod: 0.0,
                max_lod: 0.0,
                lod_bias: 0.0,
                max_anisotropy: TEX_ANISO_1,
            };
            self.tex_page_idx = page_idx;
            self.glyph_block = i as u16;
        }
    }

    pub fn tex_page(&self) -> &[u8] {
        let at = self.tex.data_at;
        let end = at.wrapping_add(self.tex.data_size as usize);
        self.data.get(at..end).unwrap_or(&[])
    }

    pub fn char_quad(
        &mut self,
        pos_x: f32,
        pos_y: f32,
        scale_x: f32,
        scale_y: f32,
        chr: u16,
        visible: bool,
    ) -> CharQuad {
        let mut width = TWidth::default();
        self.load_font(chr, Some(&mut width));

        let ratio_x = scale_x / self.cell_width() as f32;
        let ratio_y = scale_y / self.height() as f32;

        let x1 = if self.fixed || !visible {
            pos_x
        } else {
            pos_x - width.a as f32 * ratio_x
        };
        let mut advance = self.fixed_width as f32 * ratio_x;
        if !self.fixed {
            advance = if !visible {
                (width.b as f32 + width.a as f32) * ratio_x
            } else {
                width.b as f32 * ratio_x
            };
        }
        let x2 = x1 + scale_x;
        let y1 = pos_y - self.ascent() as f32 * ratio_y;
        let y2 = self.descent() as f32 * ratio_y + pos_y;

        let glyph = self.glyph_at(self.glyph_block as usize).unwrap_or_default();
        let u1 = safe_div(self.cell_x * 0x8000, glyph.texture_width as i32);
        let v1 = safe_div(self.cell_y * 0x8000, glyph.texture_height as i32);
        let u2 = safe_div(
            (self.cell_x + glyph.cell_width as i32) * 0x8000,
            glyph.texture_width as i32,
        );
        let v2 = safe_div(
            (self.cell_y + glyph.cell_height as i32) * 0x8000,
            glyph.texture_height as i32,
        );

        CharQuad {
            advance,
            pos: [[x1, y1], [x2, y1], [x2, y2], [x1, y2]],
            color: [self.color1, self.color2, self.color4, self.color3],
            uv: [[u1, v1], [u2, v1], [u2, v2], [u1, v2]],
            tex: self.tex,
        }
    }

    pub fn draw_string_size_scale(
        &mut self,
        pos_x: f32,
        pos_y: f32,
        width: f32,
        height: f32,
        text: &[u8],
        visible: bool,
    ) -> f32 {
        let start = pos_x;
        let mut x = pos_x;
        let mut vis = visible;
        let mut i = 0usize;
        while i < text.len() {
            let mut code = text[i] as u32;
            i += 1;
            if self.is_lead_byte(code as i32) {
                let trail = text.get(i).copied().unwrap_or(0) as u32;
                i += 1;
                code = (code << 8) | trail;
            }
            x += self
                .char_quad(x, pos_y, width, height, code as u16, vis)
                .advance;
            vis = true;
        }
        x - start
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PagingType {
    #[default]
    Page0,
    Page1,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GlyphCacheInfo {
    pub prev: Option<usize>,
    pub next: Option<usize>,
    pub code_start: u16,
    pub code_end: u16,
    pub cell_width: u16,
    pub cell_height: u16,
    pub page_size: u32,
    pub tex_format: u16,
    pub num_rows: u16,
    pub num_columns: u16,
    pub tex_width: u16,
    pub tex_height: u16,
    pub pinned: u16,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CachePage {
    pub info: GlyphCacheInfo,
    pub tex: TexObj,
    pub pixels: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct CacheFont {
    pub res: ResFont,
    pub paging_type: PagingType,
    pub max_sheet_size: u32,
    pub page_stride: u32,
    pub cache_page: u32,
    pub aram: Vec<u8>,
    pub glyph_infos: Vec<GlyphCacheInfo>,
    pub glyph_aram: Vec<u32>,
    pub pages: Vec<CachePage>,
    pub head: Option<usize>,
    pub tail: Option<usize>,
    pub free: Option<usize>,
    pub last: Option<usize>,
    pub loaded_page: Option<usize>,
    pub unused_taken: i32,
    pub cache_buffer: Vec<u8>,
}

impl Default for CacheFont {
    fn default() -> Self {
        Self {
            res: ResFont::new(),
            paging_type: PagingType::Page0,
            max_sheet_size: 0,
            page_stride: 0,
            cache_page: 0,
            aram: Vec::new(),
            glyph_infos: Vec::new(),
            glyph_aram: Vec::new(),
            pages: Vec::new(),
            head: None,
            tail: None,
            free: None,
            last: None,
            loaded_page: None,
            unused_taken: 0,
            cache_buffer: Vec::new(),
        }
    }
}

impl CacheFont {
    pub fn new(bytes: &[u8], cache_size: u32) -> Self {
        let mut font = Self {
            res: ResFont::from_bytes(bytes),
            ..Default::default()
        };
        let counts = Self::memory_size(&font.res.data);
        font.res.wid_block_num = counts.wid_count;
        font.res.glyph_block_num = counts.gly_count;
        font.res.map_block_num = counts.map_count;
        font.max_sheet_size = counts.max_sheet;
        font.page_stride = font.max_sheet_size.saturating_add(CACHE_PAGE_HEADER);
        font.cache_page = if font.page_stride == 0 {
            0
        } else {
            cache_size / font.page_stride
        };
        if font.cache_page == 0 {
            font.res.valid = false;
            return font;
        }
        let payload = counts
            .gly_size
            .saturating_sub((font.res.glyph_block_num as u32) * (GLYPH_HEADER_SIZE as u32));
        font.aram = vec![0u8; payload as usize];
        font.cache_buffer = vec![0u8; font.page_stride.wrapping_mul(font.cache_page) as usize];
        font.pages = vec![CachePage::default(); font.cache_page as usize];
        font.set_block();
        font.invalidiate_all_cache();
        font
    }

    fn memory_size(data: &[u8]) -> BlockMemory {
        let mut out = BlockMemory::default();
        for block in fontdata::blocks(data) {
            match block.kind {
                INF1 => {}
                WID1 => {
                    out.wid_count += 1;
                    out.wid_size += block.size;
                }
                GLY1 => {
                    out.gly_count += 1;
                    out.gly_size += block.size;
                    let tex_size = be32(data, block.at + 0x10).unwrap_or(0);
                    if tex_size > out.max_sheet {
                        out.max_sheet = tex_size;
                    }
                }
                MAP1 => {
                    out.map_count += 1;
                    out.map_size += block.size;
                }
                _ => {}
            }
        }
        out
    }

    pub fn set_paging_type(&mut self, paging: PagingType) {
        self.paging_type = paging;
    }

    pub fn metrics(&self) -> Metrics {
        self.res.metrics()
    }

    pub fn char_width(&self, code: u16) -> i32 {
        self.res.char_width(code)
    }

    pub fn char_offset(&self, code: u16) -> i32 {
        self.res.char_offset(code)
    }

    pub fn is_lead_byte(&self, chr: i32) -> bool {
        self.res.is_lead_byte(chr)
    }

    pub fn cell_x(&self) -> i32 {
        self.res.cell_x
    }

    pub fn cell_y(&self) -> i32 {
        self.res.cell_y
    }

    pub fn font_code(&self, chr: i32) -> i32 {
        self.res.font_code(chr)
    }

    pub fn set_block(&mut self) {
        self.res.set_block();
        let glyph = fontdata::gly1(&self.res.data).unwrap_or_default();
        self.glyph_infos.clear();
        self.glyph_aram.clear();
        let mut aram_at = 0u32;
        for block in fontdata::blocks(&self.res.data) {
            if block.kind != GLY1 {
                continue;
            }
            self.glyph_infos.push(GlyphCacheInfo {
                prev: None,
                next: None,
                code_start: glyph.start_code,
                code_end: glyph.end_code,
                cell_width: glyph.cell_width,
                cell_height: glyph.cell_height,
                page_size: glyph.texture_size,
                tex_format: glyph.texture_format,
                num_rows: glyph.num_rows,
                num_columns: glyph.num_columns,
                tex_width: glyph.texture_width,
                tex_height: glyph.texture_height,
                pinned: glyph.padding,
            });
            let src_at = block.at + GLYPH_HEADER_SIZE;
            let src_end = (src_at + (block.size as usize).saturating_sub(GLYPH_HEADER_SIZE))
                .min(self.res.data.len());
            let src = self.res.data.get(src_at..src_end).unwrap_or(&[]).to_vec();
            let dst_at = aram_at as usize;
            let len = src.len().min(self.aram.len().saturating_sub(dst_at));
            let dst_end = dst_at.wrapping_add(len);
            if let (Some(dst), Some(head)) = (self.aram.get_mut(dst_at..dst_end), src.get(..len)) {
                dst.copy_from_slice(head);
            }
            self.glyph_aram.push(aram_at);
            if glyph.texture_size > self.max_sheet_size {
                self.max_sheet_size = glyph.texture_size;
            }
            aram_at = block
                .size
                .wrapping_add(aram_at)
                .wrapping_sub(GLYPH_HEADER_SIZE as u32);
        }
    }

    pub fn invalidiate_all_cache(&mut self) {
        let count = self.pages.len();
        for i in 0..count {
            let prev = if i == 0 { None } else { Some(i - 1) };
            let next = if i + 1 == count { None } else { Some(i + 1) };
            if let Some(page) = self.pages.get_mut(i) {
                page.info.prev = prev;
                page.info.next = next;
            }
        }
        self.last = count.checked_sub(1);
        self.free = if count == 0 { None } else { Some(0) };
        self.head = None;
        self.tail = None;
    }

    fn unlink(&mut self, index: usize) {
        let Some(page) = self.pages.get(index) else {
            return;
        };
        let prev = page.info.prev;
        let next = page.info.next;
        match prev {
            None => self.head = next,
            Some(p) => {
                if let Some(pp) = self.pages.get_mut(p) {
                    pp.info.next = next;
                }
            }
        }
        match next {
            None => self.tail = prev,
            Some(n) => {
                if let Some(np) = self.pages.get_mut(n) {
                    np.info.prev = prev;
                }
            }
        }
    }

    fn prepend(&mut self, index: usize) {
        let old_head = self.head;
        self.head = Some(index);
        if let Some(page) = self.pages.get_mut(index) {
            page.info.prev = None;
            page.info.next = old_head;
        }
        match old_head {
            None => self.tail = Some(index),
            Some(old) => {
                if let Some(op) = self.pages.get_mut(old) {
                    op.info.prev = Some(index);
                }
            }
        }
    }

    pub fn determine_blank_page(&mut self) -> Option<usize> {
        if let Some(free) = self.free {
            let next = self.pages.get(free).and_then(|p| p.info.next);
            self.free = next;
            match next {
                None => self.last = None,
                Some(n) => {
                    if let Some(np) = self.pages.get_mut(n) {
                        np.info.prev = None;
                    }
                }
            }
            return Some(free);
        }
        let mut cursor = self.tail;
        while let Some(index) = cursor {
            let prev = self.pages.get(index).and_then(|p| p.info.prev);
            let unused = self.pages.get(index).map(|p| p.info.pinned) == Some(0);
            if unused {
                self.unlink(index);
                self.unused_taken += 1;
                return Some(index);
            }
            cursor = prev;
        }
        None
    }

    pub fn get_glyph_from_aram(
        &mut self,
        glyph: usize,
        page: usize,
        code: &mut i32,
    ) -> Option<i32> {
        let src = self.glyph_infos.get(glyph).copied()?;
        if let Some(dst) = self.pages.get_mut(page) {
            dst.info = src;
        }
        self.prepend(page);
        let info = self.pages.get(page).map(|p| p.info)?;
        let cells = info.num_rows as i32 * info.num_columns as i32;
        let page_idx = safe_div(*code, cells);
        if let Some(dst) = self.pages.get_mut(page) {
            let start = dst.info.code_start.wrapping_add((page_idx * cells) as u16);
            let last = start.wrapping_add(cells as u16).wrapping_sub(1);
            dst.info.code_start = start;
            if dst.info.code_end < last {
                dst.info.code_end = last;
            }
        }
        *code -= page_idx * cells;
        let at = (self.glyph_aram.get(glyph).copied().unwrap_or(0) as usize)
            .wrapping_add((page_idx as usize).wrapping_mul(info.page_size as usize));
        let end = at.wrapping_add(info.page_size as usize);
        let pixels = self.aram.get(at..end).unwrap_or(&[]).to_vec();
        if let Some(dst) = self.pages.get_mut(page) {
            dst.pixels = pixels;
            dst.tex = TexObj {
                data_at: 0,
                data_size: info.page_size,
                width: info.tex_width,
                height: info.tex_height,
                format: info.tex_format,
                wrap_s: TEX_WRAP_CLAMP,
                wrap_t: TEX_WRAP_CLAMP,
                mipmap: false,
                min_lod: 0.0,
                max_lod: 0.0,
                lod_bias: 0.0,
                max_anisotropy: TEX_ANISO_1,
            };
        }
        Some(page_idx)
    }

    pub fn load_cache_char(&mut self, code: &mut i32, pin: bool) -> Option<usize> {
        let mut found = None;
        let mut cursor = self.head;
        while let Some(index) = cursor {
            let Some(info) = self.pages.get(index).map(|p| p.info) else {
                break;
            };
            if info.code_start as i32 <= *code && *code <= info.code_end as i32 {
                found = Some(index);
                *code -= info.code_start as i32;
                break;
            }
            cursor = info.next;
        }
        let index = match found {
            Some(index) => index,
            None => {
                let mut glyph = 0usize;
                while glyph < self.res.glyph_block_num as usize {
                    if let Some(info) = self.glyph_infos.get(glyph) {
                        if info.code_start as i32 <= *code && *code <= info.code_end as i32 {
                            *code -= info.code_start as i32;
                            break;
                        }
                    }
                    glyph += 1;
                }
                if glyph >= self.res.glyph_block_num as usize {
                    return None;
                }
                let blank = self.determine_blank_page()?;
                let tex_page = self.get_glyph_from_aram(glyph, blank, code)?;
                self.res.tex_page_idx = tex_page;
                self.res.glyph_block = glyph as u16;
                blank
            }
        };
        if pin {
            if let Some(page) = self.pages.get_mut(index) {
                page.info.pinned = 1;
            }
        }
        Some(index)
    }

    pub fn load_image(&mut self, code: u16) {
        let mut rel = code as i32;
        let Some(page) = self.load_cache_char(&mut rel, false) else {
            return;
        };
        let Some(info) = self.pages.get(page).map(|p| p.info) else {
            return;
        };
        self.res.cell_x = info.cell_width as i32 * safe_rem(rel, info.num_rows as i32);
        self.res.cell_y = info.cell_height as i32 * safe_div(rel, info.num_rows as i32);
        self.loaded_page = Some(page);
        if self.paging_type == PagingType::Page1 {
            self.unlink(page);
            self.prepend(page);
        }
    }

    pub fn load_font(&mut self, code: u16, width: Option<&mut TWidth>) {
        if let Some(dst) = width {
            let entry = self.res.width_entry(code);
            dst.a = entry.a;
            dst.b = entry.b;
        }
        let font_code = self.font_code(code as i32);
        self.load_image(font_code as u16);
    }

    pub fn tex_page(&self) -> &[u8] {
        self.loaded_page
            .and_then(|page| self.pages.get(page))
            .map_or(&[], |page| page.pixels.as_slice())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BlockMemory {
    pub wid_count: u16,
    pub wid_size: u32,
    pub gly_count: u16,
    pub gly_size: u32,
    pub map_count: u16,
    pub map_size: u32,
    pub max_sheet: u32,
}

#[derive(Resource, Clone, Debug, Default)]
pub struct FontBank {
    fonts: Vec<ResFont>,
}

impl FontBank {
    pub fn new() -> Self {
        Self { fonts: Vec::new() }
    }

    pub fn with_fonts(fonts: Vec<ResFont>) -> Self {
        Self { fonts }
    }

    pub fn ascent_fix12(count: usize) -> Self {
        let mut bank = Self::new();
        for _ in 0..count {
            bank.insert(ResFont::ascent_fix12());
        }
        bank
    }

    pub fn insert(&mut self, font: ResFont) -> usize {
        self.fonts.push(font);
        self.fonts.len() - 1
    }

    pub fn len(&self) -> usize {
        self.fonts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.fonts.is_empty()
    }

    pub fn get(&self, slot: usize) -> Option<&ResFont> {
        self.fonts.get(slot)
    }

    pub fn get_mut(&mut self, slot: usize) -> Option<&mut ResFont> {
        self.fonts.get_mut(slot)
    }

    pub fn fonts(&self) -> &[ResFont] {
        &self.fonts
    }
}

impl FontSlot for FontBank {
    fn metrics(&self, slot: usize) -> Metrics {
        self.get(slot)
            .map_or_else(Metrics::default, ResFont::metrics)
    }

    fn char_width(&self, slot: usize, code: u16) -> i32 {
        self.get(slot).map_or(0, |font| font.char_width(code))
    }
}
