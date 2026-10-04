use glam::Vec3;

//! Shared contract for the message/text stack.
//!
//! Every sibling module is ported independently against this file. Types,
//! constant values and cross-module signatures are fixed here; implementations
//! live in the per-module files and must not redeclare anything from here.

pub mod font;
pub mod fontdata;
pub mod msgbox;
pub mod msgbox1;
pub mod msgdata;
pub mod msgget;
pub mod msglayout;
pub mod msgproc;
pub mod msgtags;

pub const INF1: u32 = u32::from_be_bytes(*b"INF1");
pub const WID1: u32 = u32::from_be_bytes(*b"WID1");
pub const GLY1: u32 = u32::from_be_bytes(*b"GLY1");
pub const MAP1: u32 = u32::from_be_bytes(*b"MAP1");

pub const FONT_MAGIC: u64 = u64::from_be_bytes(*b"FONTbfn1");

pub const GLYPH_W: i32 = 12;
pub const GLYPH_H: i32 = 12;
pub const ZFONT: [[u16; 2]; 97] = [
    [0x8340, 0x84A4], [0x8341, 0x84A5], [0x8342, 0x84A6], [0x8343, 0x84A7], [0x8344, 0x84A8],
    [0x8345, 0x84A9], [0x8346, 0x84AA], [0x8347, 0x84AB], [0x8348, 0x84AC], [0x8349, 0x84AD],
    [0x834A, 0x84AE], [0x834B, 0x84AF], [0x834C, 0x84B0], [0x834D, 0x84B1], [0x834E, 0x84B2],
    [0x834F, 0x84B3], [0x8350, 0x84B4], [0x8351, 0x84B5], [0x8352, 0x84B6], [0x8353, 0x84B7],
    [0x8354, 0x84B8], [0x8355, 0x84B9], [0x8356, 0x84BA], [0x8357, 0x84BB], [0x8358, 0x84BC],
    [0x8359, 0x84BD], [0x835A, 0x84BE], [0x835B, 0x8740], [0x835C, 0x8741], [0x835D, 0x8742],
    [0x835E, 0x8743], [0x835F, 0x8744], [0x8360, 0x8745], [0x8361, 0x8746], [0x8362, 0x8747],
    [0x8363, 0x8748], [0x8364, 0x8749], [0x8365, 0x874A], [0x8366, 0x874B], [0x8367, 0x874C],
    [0x8368, 0x874D], [0x8369, 0x874E], [0x836A, 0x874F], [0x836B, 0x8750], [0x836C, 0x8751],
    [0x836D, 0x8752], [0x836E, 0x8753], [0x836F, 0x8754], [0x8370, 0x8755], [0x8371, 0x8756],
    [0x8372, 0x8757], [0x8373, 0x8758], [0x8374, 0x8759], [0x8375, 0x875A], [0x8376, 0x875B],
    [0x8377, 0x875C], [0x8378, 0x875D], [0x8379, 0x875F], [0x837A, 0x8760], [0x837B, 0x8761],
    [0x837C, 0x8762], [0x837D, 0x8763], [0x837E, 0x8764], [0x8380, 0x8765], [0x8381, 0x8766],
    [0x8382, 0x8767], [0x8383, 0x8768], [0x8384, 0x8769], [0x8385, 0x876A], [0x8386, 0x876B],
    [0x8387, 0x876C], [0x8388, 0x876D], [0x8389, 0x876E], [0x838A, 0x876F], [0x838B, 0x8770],
    [0x838C, 0x8771], [0x838D, 0x8772], [0x838E, 0x8773], [0x838F, 0x8774], [0x8390, 0x8775],
    [0x8391, 0x877E], [0x8392, 0x8780], [0x8393, 0x8781], [0x824F, 0x8782], [0x8250, 0x8783],
    [0x8251, 0x8784], [0x8252, 0x8785], [0x8253, 0x8786], [0x8254, 0x8787], [0x8255, 0x8788],
    [0x8256, 0x8789], [0x8257, 0x878A], [0x8258, 0x878B], [0x8141, 0x878C], [0x815B, 0x878D],
    [0x8175, 0x878E], [0x8176, 0x878F],
];

pub const ICON_SLOTS: usize = 0xF;
pub const MSG_MAX_SELECT: usize = 4;
pub const TEXT_BUF: usize = 0x64;
pub const NUM_DIGITS: usize = 4;

pub const SELECT_OFF: u8 = 0;
pub const SELECT_ON: u8 = 1;
pub const SELECT_YOKO: u8 = 2;
pub const SELECT_AUCTION: u8 = 3;

pub const CTRL_PLAYER_NAME: u8 = 0x00;
pub const CTRL_UNK01: u8 = 0x01;
pub const CTRL_UNK02: u8 = 0x02;
pub const CTRL_UNK03: u8 = 0x03;
pub const CTRL_UNK04: u8 = 0x04;
pub const CTRL_UNK05: u8 = 0x05;
pub const CTRL_DUMMY: u8 = 0x06;
pub const CTRL_WAIT: u8 = 0x07;
pub const CTRL_SELECT_TWO: u8 = 0x08;
pub const CTRL_SELECT_THREE: u8 = 0x09;
pub const CTRL_A_BUTTON: u8 = 0x0A;
pub const CTRL_B_BUTTON: u8 = 0x0B;
pub const CTRL_C_STICK: u8 = 0x0C;
pub const CTRL_L_BUTTON: u8 = 0x0D;
pub const CTRL_R_BUTTON: u8 = 0x0E;
pub const CTRL_X_BUTTON: u8 = 0x0F;
pub const CTRL_Y_BUTTON: u8 = 0x10;
pub const CTRL_Z_BUTTON: u8 = 0x11;
pub const CTRL_DPAD: u8 = 0x12;
pub const CTRL_MAIN_STICK: u8 = 0x13;
pub const CTRL_ARROW_LEFT: u8 = 0x14;
pub const CTRL_ARROW_RIGHT: u8 = 0x15;
pub const CTRL_ARROW_UP: u8 = 0x16;
pub const CTRL_ARROW_DOWN: u8 = 0x17;
pub const CTRL_MAIN_STICK_UP: u8 = 0x18;
pub const CTRL_MAIN_STICK_DOWN: u8 = 0x19;
pub const CTRL_MAIN_STICK_LEFT: u8 = 0x1A;
pub const CTRL_MAIN_STICK_RIGHT: u8 = 0x1B;
pub const CTRL_MAIN_STICK_UP_DOWN: u8 = 0x1C;
pub const CTRL_MAIN_STICK_LEFT_RIGHT: u8 = 0x1D;
pub const CTRL_SELECT_YOKO_LEFT: u8 = 0x1E;
pub const CTRL_SELECT_YOKO_RIGHT: u8 = 0x1F;
pub const CTRL_UNK20: u8 = 0x20;
pub const CTRL_UNK21: u8 = 0x21;
pub const CTRL_UNK22: u8 = 0x22;
pub const CTRL_UNK23: u8 = 0x23;
pub const CTRL_UNK24: u8 = 0x24;
pub const CTRL_UNK25: u8 = 0x25;
pub const CTRL_UNK26: u8 = 0x26;
pub const CTRL_FLASHING_A_BUTTON: u8 = 0x27;
pub const CTRL_UNK28: u8 = 0x28;
pub const CTRL_UNK29: u8 = 0x29;
pub const CTRL_STARBURST: u8 = 0x2A;
pub const CTRL_UNK2B: u8 = 0x2B;
pub const CTRL_UNK2C: u8 = 0x2C;
pub const CTRL_UNK2D: u8 = 0x2D;
pub const CTRL_UNK2E: u8 = 0x2E;
pub const CTRL_UNK2F: u8 = 0x2F;
pub const CTRL_UNK30: u8 = 0x30;
pub const CTRL_UNK31: u8 = 0x31;
pub const CTRL_UNK32: u8 = 0x32;
pub const CTRL_UNK33: u8 = 0x33;
pub const CTRL_UNK34: u8 = 0x34;
pub const CTRL_UNK35: u8 = 0x35;
pub const CTRL_UNK36: u8 = 0x36;
pub const CTRL_UNK37: u8 = 0x37;
pub const CTRL_UNK38: u8 = 0x38;
pub const CTRL_HEART: u8 = 0x39;
pub const CTRL_MUSIC_NOTE: u8 = 0x3A;
pub const CTRL_UNK3B: u8 = 0x3B;
pub const CTRL_UNK3C: u8 = 0x3C;
pub const CTRL_UNK3D: u8 = 0x3D;
pub const CTRL_UNK3E: u8 = 0x3E;
pub const CTRL_UNK3F: u8 = 0x3F;
pub const CTRL_UNK40: u8 = 0x40;
pub const CTRL_UNK41: u8 = 0x41;
pub const CTRL_UNK42: u8 = 0x42;
pub const CTRL_UNK43: u8 = 0x43;
pub const CTRL_UNK44: u8 = 0x44;
pub const CTRL_UNK45: u8 = 0x45;
pub const CTRL_UNK46: u8 = 0x46;
pub const CTRL_UNK47: u8 = 0x47;
pub const CTRL_UNK48: u8 = 0x48;
pub const CTRL_UNK49: u8 = 0x49;
pub const CTRL_UNK4A: u8 = 0x4A;

pub const SPCL_COLOR: u8 = 0x00;
pub const SPCL_UNK1: u8 = 0x01;
pub const SPCL_RUBY: u8 = 0x02;

pub const ICON_A_BUTTON: u8 = 0x00;
pub const ICON_B_BUTTON: u8 = 0x01;
pub const ICON_C_STICK: u8 = 0x02;
pub const ICON_L_BUTTON: u8 = 0x03;
pub const ICON_R_BUTTON: u8 = 0x04;
pub const ICON_X_BUTTON: u8 = 0x05;
pub const ICON_Y_BUTTON: u8 = 0x06;
pub const ICON_Z_BUTTON: u8 = 0x07;
pub const ICON_DPAD: u8 = 0x08;
pub const ICON_MAIN_STICK: u8 = 0x09;
pub const ICON_ARROW_LEFT: u8 = 0x0A;
pub const ICON_ARROW_RIGHT: u8 = 0x0B;
pub const ICON_ARROW_UP: u8 = 0x0C;
pub const ICON_ARROW_DOWN: u8 = 0x0D;
pub const ICON_MAIN_STICK_UP: u8 = 0x0E;
pub const ICON_MAIN_STICK_DOWN: u8 = 0x0F;
pub const ICON_MAIN_STICK_LEFT: u8 = 0x10;
pub const ICON_MAIN_STICK_RIGHT: u8 = 0x11;
pub const ICON_MAIN_STICK_UP_DOWN: u8 = 0x12;
pub const ICON_MAIN_STICK_LEFT_RIGHT: u8 = 0x13;
pub const ICON_FLASHING_A_BUTTON: u8 = 0x14;
pub const ICON_HEART: u8 = 0x15;
pub const ICON_MUSIC_NOTE: u8 = 0x16;
pub const ICON_STARBURST: u8 = 0x17;
pub const ICON_SELECT_YOKO_LEFT: u8 = 0x14;
pub const ICON_SELECT_YOKO_RIGHT: u8 = 0x15;
pub const ICON_INPUT: u8 = 0x16;
pub const ICON_FLASHING_A_BUTTON_2: u8 = 0x17;
pub const ICON_HEART_2: u8 = 0x18;
pub const ICON_MUSIC_NOTE_2: u8 = 0x19;
pub const ICON_STARBURST_2: u8 = 0x1A;
pub const ICON_NONE: u8 = 0xFF;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MesgEntry {
    pub data_offs: u32,
    pub msg_no: u16,
    pub item_price: i16,
    pub next_msg_no: u16,
    pub field_0x0a: u16,
    pub textbox_type: u8,
    pub draw_type: u8,
    pub textbox_position: u8,
    pub item_image: u8,
    pub text_alignment: u8,
    pub initial_sound: u8,
    pub initial_camera: u8,
    pub initial_animation: u8,
    pub field_0x14: u8,
    pub field_0x15: u8,
    pub field_0x16: u8,
    pub field_0x17: u8,
}

impl MesgEntry {
    pub const LEN: usize = 0x18;

    pub fn parse(bytes: &[u8], at: usize) -> Option<Self> {
        Some(Self {
            data_offs: be32(bytes, at)?,
            msg_no: be16(bytes, at + 4)?,
            item_price: be16(bytes, at + 6)? as i16,
            next_msg_no: be16(bytes, at + 8)?,
            field_0x0a: be16(bytes, at + 0x0A)?,
            textbox_type: *bytes.get(at + 0x0C)?,
            draw_type: *bytes.get(at + 0x0D)?,
            textbox_position: *bytes.get(at + 0x0E)?,
            item_image: *bytes.get(at + 0x0F)?,
            text_alignment: *bytes.get(at + 0x10)?,
            initial_sound: *bytes.get(at + 0x11)?,
            initial_camera: *bytes.get(at + 0x12)?,
            initial_animation: *bytes.get(at + 0x13)?,
            field_0x14: *bytes.get(at + 0x14)?,
            field_0x15: *bytes.get(at + 0x15)?,
            field_0x16: *bytes.get(at + 0x16)?,
            field_0x17: *bytes.get(at + 0x17)?,
        })
    }
}

pub fn be16(bytes: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_be_bytes(bytes.get(at..at + 2)?.try_into().ok()?))
}

pub fn be32(bytes: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_be_bytes(bytes.get(at..at + 4)?.try_into().ok()?))
}

pub fn be64(bytes: &[u8], at: usize) -> Option<u64> {
    Some(u64::from_be_bytes(bytes.get(at..at + 8)?.try_into().ok()?))
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Inf1 {
    pub block_type: u32,
    pub size: u32,
    pub font_type: u16,
    pub ascent: u16,
    pub descent: u16,
    pub width: u16,
    pub leading: u16,
    pub default_code: u16,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Wid1 {
    pub block_type: u32,
    pub size: u32,
    pub start_code: u16,
    pub end_code: u16,
    pub chunk_num: [u8; 4],
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Map1 {
    pub block_type: u32,
    pub size: u32,
    pub mapping_method: u16,
    pub start_code: u16,
    pub end_code: u16,
    pub num_entries: u16,
    pub leading: u16,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Gly1 {
    pub block_type: u32,
    pub size: u32,
    pub start_code: u16,
    pub end_code: u16,
    pub cell_width: u16,
    pub cell_height: u16,
    pub texture_size: u32,
    pub texture_format: u16,
    pub num_rows: u16,
    pub num_columns: u16,
    pub texture_width: u16,
    pub texture_height: u16,
    pub padding: u16,
    pub data_at: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MsgData {
    pub font: usize,
    pub ruby_font: usize,
    pub entry: MesgEntry,
    pub actor_position: Vec3,
    pub field_0x14: f32,
    pub field_0x18: f32,
    pub field_0x1c: f32,
    pub now_cursor_pos: f32,
    pub field_0x24: f32,
    pub field_0x28: f32,
    pub bmg: Vec<u8>,
    pub out: [Option<String>; MSG_MAX_SELECT],
    pub select_message: [Option<String>; MSG_MAX_SELECT],
    pub field_0x70: [u8; TEXT_BUF],
    pub field_0xd4: [u8; 4],
    pub field_0xd8: [i32; 4],
    pub field_0xe8: [i32; 4],
    pub field_0xf8: [i32; 4],
    pub field_0x108: [i32; 4],
    pub count: i32,
    pub char_space: i32,
    pub line_space: i32,
    pub ruby_char_space: i32,
    pub line_width: i32,
    pub center_line_width: i32,
    pub line_count: i32,
    pub field_0x134: i32,
    pub now_line: i32,
    pub aim_line: i32,
    pub field_0x140: i32,
    pub font_size: i32,
    pub field_0x148: i32,
    pub ruby_font_size: i32,
    pub field_0x150: i32,
    pub field_0x154: i32,
    pub wait_timer: i32,
    pub space_timer: i32,
    pub send_speed: i32,
    pub key_wait_timer: i32,
    pub icon_pos_x: [i32; ICON_SLOTS],
    pub icon_pos_y: [i32; ICON_SLOTS],
    pub icon_scale: [i32; ICON_SLOTS],
    pub select_length: i32,
    pub icon_color: [u32; ICON_SLOTS],
    pub string_color: u32,
    pub field_0x260: i16,
    pub field_0x264: f32,
    pub field_0x268: f32,
    pub field_0x26c: f32,
    pub field_0x270: f32,
    pub field_0x274: f32,
    pub field_0x278: f32,
    pub mesg_status: u8,
    pub field_0x27d: u8,
    pub field_0x27e: u8,
    pub field_0x27f: u8,
    pub field_0x280: u8,
    pub icon_num: [u8; ICON_SLOTS],
    pub field_0x290: u8,
    pub field_0x291: u8,
    pub field_0x292: u8,
    pub field_0x293: u8,
    pub select_flag: u8,
    pub field_0x295: u8,
    pub field_0x296: u8,
    pub auto_send_flag: u8,
    pub hand_send_flag: u8,
    pub field_0x299: u8,
    pub field_0x29a: u8,
    pub field_0x29b: u8,
    pub space_flag: u8,
    pub field_0x29d: u8,
}

impl Default for MsgData {
    fn default() -> Self {
        Self {
            font: 0,
            ruby_font: 0,
            entry: MesgEntry::default(),
            actor_position: Vec3::ZERO,
            field_0x14: 0.0,
            field_0x18: 0.0,
            field_0x1c: 0.0,
            now_cursor_pos: 0.0,
            field_0x24: 0.0,
            field_0x28: 0.0,
            bmg: Vec::new(),
            out: Default::default(),
            select_message: Default::default(),
            field_0x70: [0; TEXT_BUF],
            field_0xd4: [0; 4],
            field_0xd8: [0; 4],
            field_0xe8: [0; 4],
            field_0xf8: [0; 4],
            field_0x108: [0; 4],
            count: 0,
            char_space: 0,
            line_space: 0,
            ruby_char_space: 0,
            line_width: 0,
            center_line_width: 0,
            line_count: 0,
            field_0x134: 0,
            now_line: 0,
            aim_line: 0,
            field_0x140: 0,
            font_size: 0,
            field_0x148: 0,
            ruby_font_size: 0,
            field_0x150: 0,
            field_0x154: 0,
            wait_timer: 0,
            space_timer: 1,
            send_speed: 1,
            key_wait_timer: 0,
            icon_pos_x: [0; ICON_SLOTS],
            icon_pos_y: [0; ICON_SLOTS],
            icon_scale: [0; ICON_SLOTS],
            select_length: 0,
            icon_color: [0; ICON_SLOTS],
            string_color: u32::MAX,
            field_0x260: 0,
            field_0x264: 0.0,
            field_0x268: 0.0,
            field_0x26c: 0.0,
            field_0x270: 0.0,
            field_0x274: 0.0,
            field_0x278: 0.0,
            mesg_status: 0,
            field_0x27d: 0,
            field_0x27e: 0,
            field_0x27f: 0,
            field_0x280: 0,
            icon_num: [ICON_NONE; ICON_SLOTS],
            field_0x290: 0,
            field_0x291: 0,
            field_0x292: 0,
            field_0x293: 0,
            select_flag: SELECT_OFF,
            field_0x295: 0,
            field_0x296: 0,
            auto_send_flag: 0,
            hand_send_flag: 0,
            field_0x299: 0,
            field_0x29a: 0,
            field_0x29b: 0,
            space_flag: 0,
            field_0x29d: 0,
        }
    }
}

pub const STATUS_UNK0: u8 = 0x00;
pub const STATUS_PREPARING: u8 = 0x01;
pub const STATUS_BOX_OPENING: u8 = 0x02;
pub const STATUS_SCOPE_OPENING_1: u8 = 0x03;
pub const STATUS_SCOPE_OPENING_2: u8 = 0x04;
pub const STATUS_UNK5: u8 = 0x05;
pub const STATUS_TYPING: u8 = 0x06;
pub const STATUS_STOP: u8 = 0x07;
pub const STATUS_SELECT_2: u8 = 0x08;
pub const STATUS_SELECT_3: u8 = 0x09;
pub const STATUS_CLOSE_WAIT: u8 = 0x0A;
pub const STATUS_SCOPE_ACTIVE: u8 = 0x0B;
pub const STATUS_SCOPE_DEMO: u8 = 0x0C;
pub const STATUS_SCOPE_WAIT: u8 = 0x0D;
pub const STATUS_MSG_DISPLAYED: u8 = 0x0E;
pub const STATUS_MSG_CONTINUES: u8 = 0x0F;
pub const STATUS_MSG_ENDS: u8 = 0x10;
pub const STATUS_BOX_CLOSING: u8 = 0x11;
pub const STATUS_BOX_CLOSED: u8 = 0x12;
pub const STATUS_MSG_DESTROYED: u8 = 0x13;
pub const STATUS_SELECT_YOKO: u8 = 0x14;
pub const STATUS_INPUT: u8 = 0x15;
pub const STATUS_TACT: u8 = 0x16;
pub const STATUS_DEMO: u8 = 0x17;

pub const WAIT_TIMER_MAX: i32 = 0x7FFF;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Metrics {
    pub width: i32,
    pub ascent: i32,
    pub descent: i32,
    pub leading: i32,
    pub cell_width: i32,
    pub cell_height: i32,
}

pub trait FontSlot {
    fn metrics(&self, slot: usize) -> Metrics;
    fn char_width(&self, slot: usize, code: u16) -> i32;
}

impl MsgData {
    pub fn set_bmg(&mut self, bmg: Vec<u8>) {
        self.bmg = bmg;
        self.count = 0;
    }

    pub fn icon_select(&mut self, scale: i32, icon: u8) {
        if (self.field_0x280 as usize) < ICON_SLOTS {
            let i = self.field_0x280 as usize;
            self.icon_num[i] = icon;
            self.icon_pos_x[i] = self.now_cursor_pos as i32;
            self.icon_pos_y[i] = self.line_count;
            self.icon_scale[i] = scale;
            self.icon_color[i] = self.string_color;
            self.field_0x280 += 1;
        }
        if icon != ICON_INPUT {
            self.field_0x150 += 1;
            self.field_0x14 += scale as f32;
            let idx = self.line_count as usize;
            let base = if self.select_flag != SELECT_ON {
                self.field_0xf8.get(idx).copied().unwrap_or(0)
            } else {
                self.field_0x108.get(idx).copied().unwrap_or(0)
            };
            self.now_cursor_pos = base as f32 + self.field_0x14 + 0.5;
            let tag = format!("\x1bCR[{}]", scale);
            if let Some(s) = self.out[0].as_mut() {
                s.push_str(&tag);
            }
            if let Some(s) = self.out[2].as_mut() {
                s.push_str(&tag);
            }
            let step = self.count.wrapping_add(1);
            self.count = self
                .count
                .wrapping_add(self.bmg.get(step as usize).copied().unwrap_or(0) as i32);
        }
    }

    pub fn dec_wait_timer(&mut self) -> i32 {
        self.wait_timer = if self.wait_timer > 0 {
            self.wait_timer - 1
        } else {
            0
        };
        self.wait_timer
    }

    pub fn dec_key_wait_timer(&mut self) -> i32 {
        self.key_wait_timer = if self.key_wait_timer > 0 {
            self.key_wait_timer - 1
        } else {
            0
        };
        self.key_wait_timer
    }

    pub fn select_off(&mut self) {
        self.select_flag = SELECT_OFF;
    }

    pub fn select_on(&mut self) {
        self.select_flag = SELECT_ON;
    }

    pub fn select_yoko_on(&mut self) {
        self.select_flag = SELECT_YOKO;
    }

    pub fn select_auction_on(&mut self) {
        self.select_flag = SELECT_AUCTION;
    }

    pub fn short_cut(&mut self) {
        self.field_0x299 = 1;
    }

    pub fn set_char_alpha(&mut self, a: u8, b: u8, c: u8, d: u8) {
        self.field_0x290 = a;
        self.field_0x291 = b;
        self.field_0x292 = c;
        self.field_0x293 = d;
    }
}