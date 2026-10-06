use bevy_ecs::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bounds {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Bounds {
    pub fn right(&self) -> f32 {
        self.x + self.w
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Pane,
    Picture,
    TextBox,
    Window,
    Unknown,
}

#[derive(Clone, Debug)]
pub struct ScreenPane {
    pub name: String,
    pub kind: Kind,
    pub visible: bool,
    pub bounds: Bounds,
    pub alpha: u8,
    pub base_position: u8,
    pub rotation: u16,
    pub texture: Option<String>,
    pub palette: Option<String>,
    pub color_black: u32,
    pub color_white: u32,
    pub corner_color: [u32; 4],
    pub frame_textures: [Option<String>; 4],
    pub contents_texture: Option<String>,
    pub window_box: Bounds,
    pub window_field: u8,
    pub font: Option<String>,
    pub char_color: u32,
    pub grad_color: u32,
    pub binding_h: u8,
    pub binding_v: u8,
    pub char_space: i16,
    pub line_space: i16,
    pub font_size_x: u16,
    pub font_size_y: u16,
    pub text: String,
}

impl Default for ScreenPane {
    fn default() -> Self {
        Self {
            name: String::new(),
            kind: Kind::Pane,
            visible: true,
            bounds: Bounds {
                x: 0.0,
                y: 0.0,
                w: 0.0,
                h: 0.0,
            },
            alpha: 0xFF,
            base_position: 0,
            rotation: 0,
            texture: None,
            palette: None,
            color_black: 0x0000_0000,
            color_white: 0xFFFF_FFFF,
            corner_color: [0xFFFF_FFFF; 4],
            frame_textures: [None; 4],
            contents_texture: None,
            window_box: Bounds {
                x: 0.0,
                y: 0.0,
                w: 0.0,
                h: 0.0,
            },
            window_field: 0,
            font: None,
            char_color: 0xFFFF_FFFF,
            grad_color: 0xFFFF_FFFF,
            binding_h: 0,
            binding_v: 0,
            char_space: 0,
            line_space: 0,
            font_size_x: 0,
            font_size_y: 0,
            text: String::new(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Screen {
    pub width: u16,
    pub height: u16,
    pub color: u32,
    pub panes: Vec<ScreenPane>,
}

impl Screen {
    pub fn find(&self, name: &str) -> Option<&ScreenPane> {
        self.panes.iter().find(|pane| pane.name == name)
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    fn u8(&mut self) -> u8 {
        let value = self.bytes.get(self.at).copied().unwrap_or(0);
        self.at += 1;
        value
    }

    fn s16(&mut self) -> i16 {
        let high = self.u8() as i16;
        let low = self.u8() as i16;
        (high << 8) | low
    }

    fn u16(&mut self) -> u16 {
        self.s16() as u16
    }

    fn u32(&mut self) -> u32 {
        let high = self.u16() as u32;
        let low = self.u16() as u32;
        (high << 16) | low
    }

    fn bytes(&mut self, len: usize) -> &'a [u8] {
        let start = self.at.min(self.bytes.len());
        let end = (self.at + len).min(self.bytes.len());
        self.at += len;
        &self.bytes[start..end]
    }

    fn align(&mut self, to: usize) {
        let rem = self.at % to;
        if rem != 0 {
            self.at += to - rem;
        }
    }

    fn at_end(&self) -> bool {
        self.at >= self.bytes.len()
    }
}

fn tag_name(bytes: [u8; 4]) -> String {
    bytes
        .iter()
        .filter(|byte| (0x21..0x7Fu8).contains(*byte))
        .map(|byte| *byte as char)
        .collect()
}

fn resource_name(reader: &mut Reader<'_>) -> Option<String> {
    let kind = reader.u8();
    let len = reader.u8() as usize;
    let name = reader.bytes(len);
    let text = String::from_utf8_lossy(name)
        .trim_end_matches('\0')
        .to_string();
    match kind {
        0 => None,
        _ if text.is_empty() => None,
        _ => Some(text),
    }
}

fn pane_stream(reader: &mut Reader<'_>) -> ScreenPane {
    let mut pane = ScreenPane::default();
    let mut size = reader.u8() as i32;
    pane.visible = reader.u8() != 0;
    reader.bytes(2);
    pane.name = tag_name(reader.u32().to_be_bytes());
    let x0 = reader.s16() as f32;
    let y0 = reader.s16() as f32;
    let w = reader.s16() as f32;
    let h = reader.s16() as f32;
    pane.bounds = Bounds { x: x0, y: y0, w, h };
    size -= 6;
    if size != 0 {
        pane.rotation = reader.u16();
        size -= 1;
    }
    if size != 0 {
        pane.base_position = reader.u8();
        size -= 1;
    }
    if size != 0 {
        pane.alpha = reader.u8();
        size -= 1;
    }
    reader.align(4);
    pane
}

pub fn parse(bytes: &[u8]) -> Option<Screen> {
    if bytes.len() < 0x20 {
        return None;
    }
    if &bytes[0..4] != b"SCRN" || &bytes[4..8] != b"blo1" {
        return None;
    }
    let mut reader = Reader { bytes, at: 32 };
    if bytes.get(reader.at..reader.at + 4) != Some(b"INF1".as_slice()) {
        return None;
    }
    reader.at += 4;
    let info_size = reader.u32() as usize;
    let mut screen = Screen::default();
    screen.width = reader.u16();
    screen.height = reader.u16();
    screen.color = reader.u32();
    reader.at = 32 + info_size;
    hierarchy(&mut reader, &mut screen);
    Some(screen)
}

fn hierarchy(reader: &mut Reader<'_>, screen: &mut Screen) {
    group(reader, screen, 0);
}

fn group(reader: &mut Reader<'_>, screen: &mut Screen, depth: u32) {
    loop {
        if reader.at_end() || reader.at + 8 > reader.bytes.len() {
            return;
        }
        let head = &reader.bytes[reader.at..reader.at + 8];
        let magic = &head[0..4];
        let size = u32::from_be_bytes([head[4], head[5], head[6], head[7]]) as usize;
        let block_start = reader.at;
        let block_end = (block_start + size).min(reader.bytes.len());
        match magic {
            b"EXT1" => return,
            b"END1" => {
                reader.at = block_end;
                return;
            }
            b"BGN1" => {
                reader.at = block_end;
                group(reader, screen, depth + 1);
                continue;
            }
            _ => {}
        }
        reader.at += 8;
        let kind = match magic {
            b"PIC1" => Kind::Picture,
            b"TBX1" => Kind::TextBox,
            b"WIN1" => Kind::Window,
            b"PAN1" => Kind::Pane,
            _ => Kind::Unknown,
        };
        let mut pane = pane_stream(reader);
        pane.kind = kind;
        match kind {
            Kind::Picture => picture(reader, &mut pane),
            Kind::TextBox => text_box(reader, &mut pane),
            Kind::Window => window(reader, &mut pane),
            _ => {}
        }
        screen.panes.push(pane);
        reader.at = block_end;
    }
}

fn picture(reader: &mut Reader<'_>, pane: &mut ScreenPane) {
    let mut count = reader.u8() as i32;
    pane.texture = resource_name(reader);
    pane.palette = resource_name(reader);
    let _ = reader.u8();
    count -= 3;
    for index in 0..count {
        match index {
            0 => {
                let _ = reader.u8();
            }
            1 => {
                let _ = reader.u8();
            }
            2 => {
                pane.color_black = reader.u32();
            }
            3 => {
                pane.color_white = reader.u32();
            }
            _ => {
                let slot = (index - 4) as usize;
                let color = reader.u32();
                if slot < 4 {
                    pane.corner_color[slot] = color;
                }
            }
        }
    }
}

fn window(reader: &mut Reader<'_>, pane: &mut ScreenPane) {
    let mut count = reader.u8() as i32;
    let x0 = reader.u16() as f32;
    let y0 = reader.u16() as f32;
    let w = reader.u16() as f32;
    let h = reader.u16() as f32;
    pane.window_box = Bounds { x: x0, y: y0, w, h };
    for slot in pane.frame_textures.iter_mut() {
        *slot = resource_name(reader);
    }
    pane.palette = resource_name(reader);
    pane.window_field = reader.u8();
    for slot in pane.corner_color.iter_mut() {
        *slot = reader.u32();
    }
    count -= 14;
    if count > 0 {
        pane.contents_texture = resource_name(reader);
        count -= 1;
    }
    if count > 0 {
        pane.color_black = reader.u32();
        count -= 1;
    }
    if count > 0 {
        pane.color_white = reader.u32();
    }
}

fn text_box(reader: &mut Reader<'_>, pane: &mut ScreenPane) {
    let mut count = reader.u8() as i32;
    pane.font = resource_name(reader);
    pane.char_color = reader.u32();
    pane.grad_color = reader.u32();
    let binding = reader.u8();
    pane.binding_h = (binding >> 2) & 0x03;
    pane.binding_v = binding & 0x03;
    pane.char_space = reader.s16();
    pane.line_space = reader.s16();
    pane.font_size_x = reader.u16();
    pane.font_size_y = reader.u16();
    let len = reader.u16() as usize;
    let text = reader.bytes(len);
    let text = String::from_utf8_lossy(text).trim_end_matches('\0').to_string();
    pane.text = match std::str::from_utf8(text.as_bytes()) {
        Ok(_) => text,
        Err(_) => text,
    };
    count -= 10;
    for index in 0..count {
        match index {
            0 => {
                let _ = reader.u8();
            }
            _ => {
                let _ = reader.u32();
            }
        }
    }
}

#[derive(Resource, Default)]
pub struct Screens {
    pub cache: Vec<(String, Screen)>,
}

impl Screens {
    pub fn load(&mut self, label: &str, bytes: &[u8]) -> Option<&Screen> {
        if !self.cache.iter().any(|(name, _)| name == label) {
            let screen = parse(bytes)?;
            self.cache.push((label.to_string(), screen));
        }
        self.cache
            .iter()
            .find(|(name, _)| name == label)
            .map(|(_, screen)| screen)
    }
}