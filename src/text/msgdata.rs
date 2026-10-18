use super::{
    CTRL_A_BUTTON, CTRL_ARROW_DOWN, CTRL_ARROW_LEFT, CTRL_ARROW_RIGHT, CTRL_ARROW_UP,
    CTRL_B_BUTTON, CTRL_C_STICK, CTRL_DPAD, CTRL_FLASHING_A_BUTTON, CTRL_HEART, CTRL_L_BUTTON,
    CTRL_MAIN_STICK, CTRL_MAIN_STICK_DOWN, CTRL_MAIN_STICK_LEFT, CTRL_MAIN_STICK_LEFT_RIGHT,
    CTRL_MAIN_STICK_RIGHT, CTRL_MAIN_STICK_UP, CTRL_MAIN_STICK_UP_DOWN, CTRL_MUSIC_NOTE,
    CTRL_PLAYER_NAME, CTRL_R_BUTTON, CTRL_SELECT_THREE, CTRL_SELECT_TWO, CTRL_SELECT_YOKO_LEFT,
    CTRL_SELECT_YOKO_RIGHT, CTRL_STARBURST, CTRL_UNK2B, CTRL_UNK2C, CTRL_UNK2D, CTRL_UNK2E,
    CTRL_UNK2F, CTRL_UNK3B, CTRL_UNK3C, CTRL_UNK3D, CTRL_UNK3E, CTRL_UNK3F, CTRL_UNK4A, CTRL_UNK20,
    CTRL_UNK21, CTRL_UNK22, CTRL_UNK23, CTRL_UNK24, CTRL_UNK25, CTRL_UNK26, CTRL_UNK28, CTRL_UNK29,
    CTRL_UNK30, CTRL_UNK31, CTRL_UNK32, CTRL_UNK33, CTRL_UNK34, CTRL_UNK35, CTRL_UNK36, CTRL_UNK37,
    CTRL_UNK38, CTRL_UNK40, CTRL_UNK41, CTRL_UNK42, CTRL_UNK43, CTRL_UNK44, CTRL_UNK45, CTRL_UNK46,
    CTRL_UNK47, CTRL_UNK48, CTRL_UNK49, CTRL_X_BUTTON, CTRL_Y_BUTTON, CTRL_Z_BUTTON, FontSlot,
    ICON_NONE, ICON_SLOTS, MsgData, SPCL_UNK1,
};

const PLAYER_NAME_PLURAL_MSG_NO: [u16; 6] = [0x33B, 0xC8B, 0x1D21, 0x31D7, 0x37DD, 0x37DE];

const ARROW_ANIME_WRAP: i32 = 45;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MsgLenState<'a> {
    pub player_name: &'a str,
    pub npc_name: &'a str,
    pub item_name: &'a str,
    pub pirate_password: &'a str,
    pub pal_language: i32,
    pub clear_count: u8,
    pub event_reg_beff: i32,
    pub message_count_number: i32,
    pub mini_game_rupee: i32,
    pub event_reg_8aff: i32,
    pub fwater_timer: i32,
    pub event_reg_86ff: i32,
    pub beast_joy_pendant: i32,
    pub event_reg_c0ff: i32,
    pub item_timer: i32,
    pub bomb_max: i32,
    pub arrow_max: i32,
    pub buy_item: i32,
    pub pay_rupee: i32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MsgStickFlags {
    pub right_trigger: bool,
    pub left_trigger: bool,
    pub up_trigger: bool,
    pub down_trigger: bool,
}

impl MsgStickFlags {
    pub const WAIT_PARM: (i32, i32, i32, i32, f32, f32, i32, i32) =
        (5, 2, 3, 2, 0.9, 0.5, 0, 0x2000);
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MsgNumInput {
    pub index: u8,
    pub number: i16,
}

fn msg_at(bmg: &[u8], i: i32) -> u8 {
    bmg.get(i as usize).copied().unwrap_or(0)
}

fn msg_load(src: &[i32], i: i32) -> i32 {
    if i < 0 {
        0
    } else {
        src.get(i as usize).copied().unwrap_or(0)
    }
}

fn msg_store(dst: &mut [i32], i: i32, value: i32) {
    if i >= 0 {
        if let Some(slot) = dst.get_mut(i as usize) {
            *slot = value;
        }
    }
}

impl MsgData {
    pub fn new_data_proc() -> Self {
        let mut data = Self::default();
        data.data_init();
        data.string_color = u32::MAX;
        data
    }

    pub fn data_init(&mut self) {
        self.field_0x14 = 0.0;
        self.field_0x18 = 0.0;
        self.field_0x1c = 0.0;
        self.now_cursor_pos = 0.0;
        self.field_0x24 = 0.0;
        self.field_0x28 = 0.0;
        for i in 0..4 {
            self.field_0xd8[i] = 0;
            self.field_0xf8[i] = 0;
            self.field_0xe8[i] = 0;
            self.field_0x108[i] = 0;
        }
        self.select_length = 0;
        self.field_0xd4[2] = 0;
        self.field_0xd4[1] = 0;
        self.field_0xd4[0] = 0;
        self.count = 0;
        self.char_space = 0;
        self.ruby_char_space = 0;
        self.line_space = 0;
        self.line_width = 0;
        self.center_line_width = 0;
        self.line_count = 0;
        self.field_0x134 = 0;
        self.now_line = 0;
        self.aim_line = 0;
        self.field_0x140 = 0;
        self.font_size = 0;
        self.field_0x148 = 0;
        self.ruby_font_size = 0;
        self.field_0x150 = 0;
        self.field_0x154 = 0;
        self.wait_timer = 0;
        self.space_timer = 1;
        self.send_speed = 1;
        self.key_wait_timer = 0;
        self.field_0x29d = 0;
        self.field_0x260 = 0;
        self.field_0x264 = 0.0;
        self.field_0x268 = 0.0;
        self.field_0x26c = 0.0;
        self.field_0x270 = 0.0;
        self.field_0x278 = 0.0;
        self.field_0x274 = 0.0;
        self.field_0x293 = 0;
        self.field_0x292 = 0;
        self.field_0x291 = 0;
        self.field_0x290 = 0;
        self.mesg_status = super::STATUS_TYPING;
        self.field_0x27d = 0;
        self.field_0x27e = 0;
        self.field_0x27f = 0;
        self.field_0x280 = 0;
        for i in 0..ICON_SLOTS {
            self.icon_pos_x[i] = 0;
            self.icon_pos_y[i] = 0;
            self.icon_scale[i] = 0;
            self.icon_num[i] = ICON_NONE;
            self.icon_color[i] = 0;
        }
        self.field_0x299 = 0;
        self.field_0x29a = 0;
        self.select_off();
        self.field_0x29b = 0;
        self.auto_send_flag = 0;
        self.hand_send_flag = 0;
        self.space_flag = 0;
        self.field_0x295 = 0;
        self.field_0x296 = 0;
    }

    pub fn char_length<F: FontSlot + ?Sized>(
        &self,
        font: &F,
        scale: i32,
        code: u16,
        spaced: bool,
    ) -> f32 {
        let char_width = font.char_width(self.font, code) as f32;
        let cell_width = scale as f32 / font.metrics(self.font).cell_width as f32;
        let width = char_width * cell_width;
        if spaced {
            width
        } else {
            self.char_space as f32 + width
        }
    }

    pub fn ruby_length<F: FontSlot + ?Sized>(&self, font: &F, code: u16, spaced: bool) -> f32 {
        let width = font.char_width(self.ruby_font, code);
        let cell_width = font.metrics(self.ruby_font).cell_width as f32;
        let scale = self.ruby_font_size as f32 / cell_width;
        if spaced {
            width as f32 * scale
        } else {
            self.ruby_char_space as f32 + width as f32 * scale
        }
    }

    pub fn string_length<F: FontSlot + ?Sized>(&mut self, font: &F, state: MsgLenState<'_>) -> u32 {
        let mut sp18 = [0i32; 4];
        let mut sp14 = 0i32;
        let mut sp10 = 0i32;
        let mut save = 0u32;
        let mut pos = self.count;
        let mut sp0c = 0.0f32;
        let mut scale_changed = false;
        let mut selecting = false;
        self.line_count = 0;
        let mut scale = self.font_size;
        self.field_0x148 = self.font_size;
        for i in 0..4 {
            sp18[i] = 0;
            self.field_0xd8[i] = 0;
            self.field_0xe8[i] = 0;
            self.field_0x108[i] = 0;
        }

        while msg_at(&self.bmg, pos) != 0 {
            if msg_at(&self.bmg, pos) == 0x1A {
                let b2 = msg_at(&self.bmg, pos + 2);
                let b3 = msg_at(&self.bmg, pos + 3);
                let b4 = msg_at(&self.bmg, pos + 4);
                let ctrl = b2 == 0 && b3 == 0;
                match b4 {
                    SPCL_UNK1 if b2 == 0xFF && b3 == 0 => {
                        let c = ((msg_at(&self.bmg, pos + 5) as i32) << 8)
                            | (msg_at(&self.bmg, pos + 6) as i32);
                        let ratio = c as f32 * 0.01f32;
                        if ratio > 1.0f32 {
                            scale_changed = true;
                        }
                        scale = (ratio * self.font_size as f32 + 0.5f32) as i32;
                    }
                    CTRL_PLAYER_NAME if ctrl => {
                        let mut name: Vec<u8> = state
                            .player_name
                            .as_bytes()
                            .iter()
                            .copied()
                            .take(19)
                            .collect();
                        if state.pal_language == 1
                            && PLAYER_NAME_PLURAL_MSG_NO.contains(&self.entry.msg_no)
                        {
                            let last = name.last().copied().unwrap_or(0);
                            if matches!(last, b's' | b'S' | b'z' | b'Z' | b'x' | b'X') {
                                name.push(b'\'');
                            } else {
                                name.push(b's');
                            }
                        }
                        let mut i = 0usize;
                        while i < name.len() {
                            let hi_nibble = (name[i] >> 4) & 0xF;
                            let c = if hi_nibble == 8 || hi_nibble == 9 {
                                let c = ((name[i] as i32) << 8)
                                    | (name.get(i + 1).copied().unwrap_or(0) as i32);
                                i += 2;
                                c
                            } else {
                                let c = name[i] as i32;
                                i += 1;
                                c
                            };
                            sp0c = if sp10 != 0 {
                                self.char_length(font, scale, c as u16, false) + sp0c
                            } else {
                                self.char_length(font, scale, c as u16, true)
                            };
                            sp10 += 1;
                        }
                        msg_store(&mut sp18, sp14, (sp0c + 0.5f32) as i32);
                    }
                    CTRL_SELECT_TWO if ctrl => {
                        selecting = true;
                        let mut i = 0;
                        while i < sp14 {
                            let v = msg_load(&sp18, i);
                            msg_store(&mut self.field_0xd8, i, v);
                            i += 1;
                        }
                        self.line_count = sp14 - 1;
                        sp14 = 0;
                    }
                    CTRL_SELECT_THREE if ctrl => {
                        selecting = true;
                        let mut i = 0;
                        while i < sp14 {
                            let v = msg_load(&sp18, i);
                            msg_store(&mut self.field_0xd8, i, v);
                            msg_store(&mut sp18, i, 0);
                            i += 1;
                        }
                        self.line_count = sp14 - 1;
                        sp14 = 0;
                    }
                    CTRL_A_BUTTON if ctrl => {
                        if self.field_0x295 == 0 && self.line_count == 0 && self.field_0x14 == 0.0 {
                            self.field_0x295 = 1;
                        }
                        sp0c += scale as f32;
                        msg_store(&mut sp18, sp14, (sp0c + 0.5f32) as i32);
                        sp10 += 1;
                    }
                    CTRL_B_BUTTON if ctrl => {
                        if self.field_0x295 == 0 && self.line_count == 0 && self.field_0x14 == 0.0 {
                            self.field_0x295 = 1;
                        }
                        sp0c += scale as f32;
                        msg_store(&mut sp18, sp14, (sp0c + 0.5f32) as i32);
                        sp10 += 1;
                    }
                    CTRL_C_STICK if ctrl => {
                        if self.field_0x295 == 0 && self.line_count == 0 && self.field_0x14 == 0.0 {
                            self.field_0x295 = 1;
                        }
                        sp0c += scale as f32;
                        msg_store(&mut sp18, sp14, (sp0c + 0.5f32) as i32);
                        sp10 += 1;
                    }
                    CTRL_L_BUTTON if ctrl => {
                        if self.field_0x295 == 0 && self.line_count == 0 && self.field_0x14 == 0.0 {
                            self.field_0x295 = 1;
                        }
                        sp0c += scale as f32;
                        msg_store(&mut sp18, sp14, (sp0c + 0.5f32) as i32);
                        sp10 += 1;
                    }
                    CTRL_R_BUTTON if ctrl => {
                        if self.field_0x295 == 0 && self.line_count == 0 && self.field_0x14 == 0.0 {
                            self.field_0x295 = 1;
                        }
                        sp0c += scale as f32;
                        msg_store(&mut sp18, sp14, (sp0c + 0.5f32) as i32);
                        sp10 += 1;
                    }
                    CTRL_X_BUTTON if ctrl => {
                        if self.field_0x295 == 0 && self.line_count == 0 && self.field_0x14 == 0.0 {
                            self.field_0x295 = 1;
                        }
                        sp0c += scale as f32;
                        msg_store(&mut sp18, sp14, (sp0c + 0.5f32) as i32);
                        sp10 += 1;
                    }
                    CTRL_Y_BUTTON if ctrl => {
                        if self.field_0x295 == 0 && self.line_count == 0 && self.field_0x14 == 0.0 {
                            self.field_0x295 = 1;
                        }
                        sp0c += scale as f32;
                        msg_store(&mut sp18, sp14, (sp0c + 0.5f32) as i32);
                        sp10 += 1;
                    }
                    CTRL_Z_BUTTON if ctrl => {
                        if self.field_0x295 == 0 && self.line_count == 0 && self.field_0x14 == 0.0 {
                            self.field_0x295 = 1;
                        }
                        sp0c += scale as f32;
                        msg_store(&mut sp18, sp14, (sp0c + 0.5f32) as i32);
                        sp10 += 1;
                    }
                    CTRL_DPAD if ctrl => {
                        if self.field_0x295 == 0 && self.line_count == 0 && self.field_0x14 == 0.0 {
                            self.field_0x295 = 1;
                        }
                        sp0c += scale as f32;
                        msg_store(&mut sp18, sp14, (sp0c + 0.5f32) as i32);
                        sp10 += 1;
                    }
                    CTRL_MAIN_STICK if ctrl => {
                        if self.field_0x295 == 0 && self.line_count == 0 && self.field_0x14 == 0.0 {
                            self.field_0x295 = 1;
                        }
                        sp0c += scale as f32;
                        msg_store(&mut sp18, sp14, (sp0c + 0.5f32) as i32);
                        sp10 += 1;
                    }
                    CTRL_ARROW_LEFT if ctrl => {
                        if self.field_0x295 == 0 && self.line_count == 0 && self.field_0x14 == 0.0 {
                            self.field_0x295 = 1;
                        }
                        sp0c += scale as f32;
                        msg_store(&mut sp18, sp14, (sp0c + 0.5f32) as i32);
                        sp10 += 1;
                    }
                    CTRL_ARROW_RIGHT if ctrl => {
                        if self.field_0x295 == 0 && self.line_count == 0 && self.field_0x14 == 0.0 {
                            self.field_0x295 = 1;
                        }
                        sp0c += scale as f32;
                        msg_store(&mut sp18, sp14, (sp0c + 0.5f32) as i32);
                        sp10 += 1;
                    }
                    CTRL_ARROW_UP if ctrl => {
                        if self.field_0x295 == 0 && self.line_count == 0 && self.field_0x14 == 0.0 {
                            self.field_0x295 = 1;
                        }
                        sp0c += scale as f32;
                        msg_store(&mut sp18, sp14, (sp0c + 0.5f32) as i32);
                        sp10 += 1;
                    }
                    CTRL_ARROW_DOWN if ctrl => {
                        if self.field_0x295 == 0 && self.line_count == 0 && self.field_0x14 == 0.0 {
                            self.field_0x295 = 1;
                        }
                        sp0c += scale as f32;
                        msg_store(&mut sp18, sp14, (sp0c + 0.5f32) as i32);
                        sp10 += 1;
                    }
                    CTRL_MAIN_STICK_UP if ctrl => {
                        if self.field_0x295 == 0 && self.line_count == 0 && self.field_0x14 == 0.0 {
                            self.field_0x295 = 1;
                        }
                        sp0c += scale as f32;
                        msg_store(&mut sp18, sp14, (sp0c + 0.5f32) as i32);
                        sp10 += 1;
                    }
                    CTRL_MAIN_STICK_DOWN if ctrl => {
                        if self.field_0x295 == 0 && self.line_count == 0 && self.field_0x14 == 0.0 {
                            self.field_0x295 = 1;
                        }
                        sp0c += scale as f32;
                        msg_store(&mut sp18, sp14, (sp0c + 0.5f32) as i32);
                        sp10 += 1;
                    }
                    CTRL_MAIN_STICK_LEFT if ctrl => {
                        if self.field_0x295 == 0 && self.line_count == 0 && self.field_0x14 == 0.0 {
                            self.field_0x295 = 1;
                        }
                        sp0c += scale as f32;
                        msg_store(&mut sp18, sp14, (sp0c + 0.5f32) as i32);
                        sp10 += 1;
                    }
                    CTRL_MAIN_STICK_RIGHT if ctrl => {
                        if self.field_0x295 == 0 && self.line_count == 0 && self.field_0x14 == 0.0 {
                            self.field_0x295 = 1;
                        }
                        sp0c += scale as f32;
                        msg_store(&mut sp18, sp14, (sp0c + 0.5f32) as i32);
                        sp10 += 1;
                    }
                    CTRL_MAIN_STICK_UP_DOWN if ctrl => {
                        if self.field_0x295 == 0 && self.line_count == 0 && self.field_0x14 == 0.0 {
                            self.field_0x295 = 1;
                        }
                        sp0c += scale as f32;
                        msg_store(&mut sp18, sp14, (sp0c + 0.5f32) as i32);
                        sp10 += 1;
                    }
                    CTRL_MAIN_STICK_LEFT_RIGHT if ctrl => {
                        if self.field_0x295 == 0 && self.line_count == 0 && self.field_0x14 == 0.0 {
                            self.field_0x295 = 1;
                        }
                        sp0c += scale as f32;
                        msg_store(&mut sp18, sp14, (sp0c + 0.5f32) as i32);
                        sp10 += 1;
                    }
                    CTRL_SELECT_YOKO_LEFT if ctrl => {
                        sp0c += scale as f32;
                        sp10 += 1;
                    }
                    CTRL_SELECT_YOKO_RIGHT if ctrl => {
                        sp0c += scale as f32;
                        sp10 += 1;
                    }
                    CTRL_HEART if ctrl => {
                        if self.field_0x295 == 0 && self.line_count == 0 && self.field_0x14 == 0.0 {
                            self.field_0x295 = 1;
                        }
                        sp0c += scale as f32;
                        msg_store(&mut sp18, sp14, (sp0c + 0.5f32) as i32);
                        sp10 += 1;
                    }
                    CTRL_MUSIC_NOTE if ctrl => {
                        if self.field_0x295 == 0 && self.line_count == 0 && self.field_0x14 == 0.0 {
                            self.field_0x295 = 1;
                        }
                        sp0c += scale as f32;
                        msg_store(&mut sp18, sp14, (sp0c + 0.5f32) as i32);
                        sp10 += 1;
                    }
                    CTRL_UNK20 if ctrl => {
                        let r = self.tag_len_kaisen_game(
                            font,
                            state.event_reg_beff,
                            sp10,
                            sp0c,
                            scale,
                            sp14,
                        );
                        sp10 = r.len;
                        sp0c = r.width;
                        msg_store(&mut sp18, sp14, r.line_width);
                    }
                    CTRL_UNK21 | CTRL_UNK24 | CTRL_UNK25 | CTRL_UNK2C | CTRL_UNK36 if ctrl => {
                        let r = self.tag_len_rupee(
                            font,
                            state.message_count_number,
                            sp10,
                            sp0c,
                            scale,
                            sp14,
                        );
                        sp10 = r.len;
                        sp0c = r.width;
                        msg_store(&mut sp18, sp14, r.line_width);
                    }
                    CTRL_UNK22 | CTRL_UNK23 if ctrl => {
                        let text = if b4 == CTRL_UNK22 {
                            state.npc_name
                        } else {
                            state.item_name
                        };
                        for byte in text.bytes() {
                            sp0c = if sp10 != 0 {
                                self.char_length(font, scale, byte as u16, false) + sp0c
                            } else {
                                self.char_length(font, scale, byte as u16, true)
                            };
                            sp10 += 1;
                        }
                        msg_store(&mut sp18, sp14, (sp0c + 0.5f32) as i32);
                    }
                    CTRL_UNK26 if ctrl => {
                        let r = self.tag_len_num_input(font, sp10, sp0c, scale, sp14);
                        sp10 = r.len;
                        sp0c = r.width;
                        msg_store(&mut sp18, sp14, r.line_width);
                    }
                    CTRL_FLASHING_A_BUTTON if ctrl => {
                        sp0c += scale as f32;
                        sp10 += 1;
                    }
                    CTRL_UNK28 if ctrl => {
                        let r = self.tag_len_sword_game(
                            font,
                            state.message_count_number,
                            sp10,
                            sp0c,
                            scale,
                            sp14,
                        );
                        sp10 = r.len;
                        sp0c = r.width;
                        msg_store(&mut sp18, sp14, r.line_width);
                    }
                    CTRL_UNK29 if ctrl => {
                        for byte in state.pirate_password.bytes() {
                            sp0c = if sp10 != 0 {
                                self.char_length(font, scale, byte as u16, false) + sp0c
                            } else {
                                self.char_length(font, scale, byte as u16, true)
                            };
                            sp10 += 1;
                        }
                        msg_store(&mut sp18, sp14, (sp0c + 0.5f32) as i32);
                    }
                    CTRL_STARBURST if ctrl => {
                        if self.field_0x295 == 0 && self.line_count == 0 && self.field_0x14 == 0.0 {
                            self.field_0x295 = 1;
                        }
                        sp0c += scale as f32;
                        msg_store(&mut sp18, sp14, (sp0c + 0.5f32) as i32);
                        sp10 += 1;
                    }
                    CTRL_UNK2B if ctrl => {
                        let r = self.tag_len_letter_game(
                            font,
                            state.mini_game_rupee,
                            sp10,
                            sp0c,
                            scale,
                            sp14,
                        );
                        sp10 = r.len;
                        sp0c = r.width;
                        msg_store(&mut sp18, sp14, r.line_width);
                    }
                    CTRL_UNK3B if ctrl => {
                        let r = self.tag_len_letter_game_max(
                            font,
                            state.event_reg_8aff,
                            sp10,
                            sp0c,
                            scale,
                            sp14,
                        );
                        sp10 = r.len;
                        sp0c = r.width;
                        msg_store(&mut sp18, sp14, r.line_width);
                    }
                    CTRL_UNK3C if ctrl => {
                        let r = self.tag_len_fish(
                            font,
                            state.message_count_number,
                            sp10,
                            sp0c,
                            scale,
                            sp14,
                        );
                        sp10 = r.len;
                        sp0c = r.width;
                        msg_store(&mut sp18, sp14, r.line_width);
                    }
                    CTRL_UNK3D if ctrl => {
                        let r = self.tag_len_fish_rupee(
                            font,
                            state.message_count_number,
                            sp10,
                            sp0c,
                            scale,
                            sp14,
                        );
                        sp10 = r.len;
                        sp0c = r.width;
                        msg_store(&mut sp18, sp14, r.line_width);
                    }
                    CTRL_UNK2D if ctrl => {
                        let r = self.tag_len_letter(
                            font,
                            state.message_count_number,
                            sp10,
                            sp0c,
                            scale,
                            sp14,
                        );
                        sp10 = r.len;
                        sp0c = r.width;
                        msg_store(&mut sp18, sp14, r.line_width);
                    }
                    CTRL_UNK2E if ctrl => {
                        let r = self.tag_len_rescue(
                            font,
                            state.message_count_number,
                            sp10,
                            sp0c,
                            scale,
                            sp14,
                        );
                        sp10 = r.len;
                        sp0c = r.width;
                        msg_store(&mut sp18, sp14, r.line_width);
                    }
                    CTRL_UNK2F if ctrl => {
                        let r = self.tag_len_forest_timer(
                            font,
                            state.fwater_timer,
                            sp10,
                            sp0c,
                            scale,
                            sp14,
                        );
                        sp10 = r.len;
                        sp0c = r.width;
                        msg_store(&mut sp18, sp14, r.line_width);
                    }
                    CTRL_UNK30 | CTRL_UNK31 if ctrl => {
                        let r = self.tag_len_birdman(
                            font,
                            state.message_count_number,
                            sp10,
                            sp0c,
                            scale,
                            sp14,
                        );
                        sp10 = r.len;
                        sp0c = r.width;
                        msg_store(&mut sp18, sp14, r.line_width);
                    }
                    CTRL_UNK32 if ctrl => {
                        let r =
                            self.tag_len_point(font, state.event_reg_86ff, sp10, sp0c, scale, sp14);
                        sp10 = r.len;
                        sp0c = r.width;
                        msg_store(&mut sp18, sp14, r.line_width);
                    }
                    CTRL_UNK33 if ctrl => {
                        let r = self.tag_len_get_pendant(
                            font,
                            state.beast_joy_pendant,
                            sp10,
                            sp0c,
                            scale,
                            sp14,
                        );
                        sp10 = r.len;
                        sp0c = r.width;
                        msg_store(&mut sp18, sp14, r.line_width);
                    }
                    CTRL_UNK34 if ctrl => {
                        let r = self.tag_len_rev_pendant(
                            font,
                            state.event_reg_c0ff,
                            sp10,
                            sp0c,
                            scale,
                            sp14,
                        );
                        sp10 = r.len;
                        sp0c = r.width;
                        msg_store(&mut sp18, sp14, r.line_width);
                    }
                    CTRL_UNK35 if ctrl => {
                        let r =
                            self.tag_len_pig_timer(font, state.item_timer, sp10, sp0c, scale, sp14);
                        sp10 = r.len;
                        sp0c = r.width;
                        msg_store(&mut sp18, sp14, r.line_width);
                    }
                    CTRL_UNK37 if ctrl => {
                        let r =
                            self.tag_len_get_bomb(font, state.bomb_max, sp10, sp0c, scale, sp14);
                        sp10 = r.len;
                        sp0c = r.width;
                        msg_store(&mut sp18, sp14, r.line_width);
                    }
                    CTRL_UNK38 if ctrl => {
                        let r =
                            self.tag_len_get_arrow(font, state.arrow_max, sp10, sp0c, scale, sp14);
                        sp10 = r.len;
                        sp0c = r.width;
                        msg_store(&mut sp18, sp14, r.line_width);
                    }
                    CTRL_UNK3E if ctrl => {
                        let r = self.tag_len_stock_bokobaba(
                            font,
                            state.buy_item,
                            sp10,
                            sp0c,
                            scale,
                            sp14,
                        );
                        sp10 = r.len;
                        sp0c = r.width;
                        msg_store(&mut sp18, sp14, r.line_width);
                    }
                    CTRL_UNK3F if ctrl => {
                        let r = self.tag_len_stock_dokuro(
                            font,
                            state.buy_item,
                            sp10,
                            sp0c,
                            scale,
                            sp14,
                        );
                        sp10 = r.len;
                        sp0c = r.width;
                        msg_store(&mut sp18, sp14, r.line_width);
                    }
                    CTRL_UNK40 if ctrl => {
                        let r = self.tag_len_stock_chuchu(
                            font,
                            state.buy_item,
                            sp10,
                            sp0c,
                            scale,
                            sp14,
                        );
                        sp10 = r.len;
                        sp0c = r.width;
                        msg_store(&mut sp18, sp14, r.line_width);
                    }
                    CTRL_UNK41 if ctrl => {
                        let r = self.tag_len_stock_pendant(
                            font,
                            state.buy_item,
                            sp10,
                            sp0c,
                            scale,
                            sp14,
                        );
                        sp10 = r.len;
                        sp0c = r.width;
                        msg_store(&mut sp18, sp14, r.line_width);
                    }
                    CTRL_UNK42 if ctrl => {
                        let r =
                            self.tag_len_stock_hane(font, state.buy_item, sp10, sp0c, scale, sp14);
                        sp10 = r.len;
                        sp0c = r.width;
                        msg_store(&mut sp18, sp14, r.line_width);
                    }
                    CTRL_UNK43 if ctrl => {
                        let r = self.tag_len_stock_kenshi(
                            font,
                            state.buy_item,
                            sp10,
                            sp0c,
                            scale,
                            sp14,
                        );
                        sp10 = r.len;
                        sp0c = r.width;
                        msg_store(&mut sp18, sp14, r.line_width);
                    }
                    CTRL_UNK44 if ctrl => {
                        let r = self.tag_len_terry_rupee(
                            font,
                            state.pay_rupee,
                            sp10,
                            sp0c,
                            scale,
                            sp14,
                        );
                        sp10 = r.len;
                        sp0c = r.width;
                        msg_store(&mut sp18, sp14, r.line_width);
                    }
                    CTRL_UNK45 if ctrl => {
                        let r = self.tag_len_input_bokobaba(font, sp10, sp0c, scale, sp14);
                        sp10 = r.len;
                        sp0c = r.width;
                        msg_store(&mut sp18, sp14, r.line_width);
                    }
                    CTRL_UNK46 if ctrl => {
                        let r = self.tag_len_input_dokuro(font, sp10, sp0c, scale, sp14);
                        sp10 = r.len;
                        sp0c = r.width;
                        msg_store(&mut sp18, sp14, r.line_width);
                    }
                    CTRL_UNK47 if ctrl => {
                        let r = self.tag_len_input_chuchu(font, sp10, sp0c, scale, sp14);
                        sp10 = r.len;
                        sp0c = r.width;
                        msg_store(&mut sp18, sp14, r.line_width);
                    }
                    CTRL_UNK48 if ctrl => {
                        let r = self.tag_len_input_pendant(font, sp10, sp0c, scale, sp14);
                        sp10 = r.len;
                        sp0c = r.width;
                        msg_store(&mut sp18, sp14, r.line_width);
                    }
                    CTRL_UNK49 if ctrl => {
                        let r = self.tag_len_input_hane(font, sp10, sp0c, scale, sp14);
                        sp10 = r.len;
                        sp0c = r.width;
                        msg_store(&mut sp18, sp14, r.line_width);
                    }
                    CTRL_UNK4A if ctrl => {
                        let r = self.tag_len_input_kenshi(font, sp10, sp0c, scale, sp14);
                        sp10 = r.len;
                        sp0c = r.width;
                        msg_store(&mut sp18, sp14, r.line_width);
                    }
                    _ => {}
                }
                pos += msg_at(&self.bmg, pos + 1) as i32;
            } else {
                if self.field_0x295 != 0 && msg_at(&self.bmg, pos) != 0 {
                    self.field_0x295 = 0;
                }
                if msg_at(&self.bmg, pos) == 0xA {
                    sp10 = 0;
                    sp0c = 0.0;
                    pos += 1;
                    if msg_at(&self.bmg, pos) == 0 {
                        self.field_0x140 = self.aim_line;
                    }

                    if !selecting {
                        if scale_changed {
                            if self.aim_line == self.now_line {
                                sp14 += 1;
                                self.line_count += 1;
                            } else {
                                self.now_line += 1;
                            }
                            scale_changed = false;
                        }
                        if self.aim_line == self.now_line {
                            sp14 += 1;
                            if sp14 >= self.entry.field_0x16 as i32 {
                                let nb2 = msg_at(&self.bmg, pos + 2);
                                let nb3 = msg_at(&self.bmg, pos + 3);
                                let nb4 = msg_at(&self.bmg, pos + 4);
                                if msg_at(&self.bmg, pos) == 0x1A
                                    && nb2 == 0
                                    && nb3 == 0
                                    && (nb4 == CTRL_SELECT_TWO || nb4 == CTRL_SELECT_THREE)
                                {
                                    selecting = true;
                                    let mut i = 0;
                                    while i < sp14 {
                                        let v = msg_load(&sp18, i);
                                        msg_store(&mut self.field_0xd8, i, v);
                                        i += 1;
                                    }
                                    self.line_count = sp14 - 1;
                                    sp14 = 0;
                                    pos += msg_at(&self.bmg, pos + 1) as i32;
                                } else {
                                    for i in 0..4 {
                                        self.field_0xd8[i] = sp18[i];
                                    }
                                    self.field_0x134 = self.line_count;
                                    return save;
                                }
                            }
                        } else {
                            self.now_line += 1;
                            save = pos as u32;
                            msg_store(&mut sp18, 0, 0);
                        }
                        continue;
                    } else if selecting {
                        let v = msg_load(&sp18, sp14);
                        msg_store(&mut self.field_0xe8, sp14, v);
                    }
                    sp14 += 1;
                } else {
                    let byte0 = msg_at(&self.bmg, pos);
                    let hi_nibble = byte0 >> 4;
                    let (code, narrow) = if hi_nibble == 8 || hi_nibble == 9 {
                        let mut code = ((byte0 as i32) << 8) | (msg_at(&self.bmg, pos + 1) as i32);
                        if state.clear_count == 0 && self.entry.textbox_type == 0xC {
                            for entry_pair in super::ZFONT.iter() {
                                if entry_pair[0] == code as u16 {
                                    code = entry_pair[1] as i32;
                                    break;
                                }
                            }
                        }
                        (code, false)
                    } else {
                        (byte0 as i32, true)
                    };
                    sp0c = if sp10 != 0 {
                        self.char_length(font, scale, code as u16, false) + sp0c
                    } else {
                        self.char_length(font, scale, code as u16, true)
                    };
                    sp10 += 1;

                    let previous = msg_load(&sp18, sp14);
                    msg_store(&mut sp18, sp14, (sp0c + 0.5f32) as i32);
                    if !selecting && sp14 != self.line_count {
                        self.line_count += 1;
                    }

                    if self.line_width <= msg_load(&sp18, sp14) {
                        msg_store(&mut sp18, sp14, previous);
                        sp14 += 1;
                        sp10 = 1;
                        sp0c = 0.0;

                        if msg_at(&self.bmg, pos) == 0 {
                            self.field_0x140 = self.aim_line;
                        }

                        if self.aim_line == self.now_line {
                            if sp14 >= self.entry.field_0x16 as i32 {
                                let nb2 = msg_at(&self.bmg, pos + 2);
                                let nb3 = msg_at(&self.bmg, pos + 3);
                                let nb4 = msg_at(&self.bmg, pos + 4);
                                if msg_at(&self.bmg, pos) == 0x1A
                                    && nb2 == 0
                                    && nb3 == 0
                                    && (nb4 == CTRL_SELECT_TWO || nb4 == CTRL_SELECT_THREE)
                                {
                                    selecting = true;
                                    let mut i = 0;
                                    while i < sp14 {
                                        let v = msg_load(&sp18, i);
                                        msg_store(&mut self.field_0xd8, i, v);
                                        i += 1;
                                    }
                                    self.line_count = sp14 - 1;
                                    sp14 = 0;
                                    pos += msg_at(&self.bmg, pos + 1) as i32;
                                } else {
                                    self.field_0x134 = self.line_count;
                                    for i in 0..4 {
                                        self.field_0xd8[i] = sp18[i];
                                    }
                                    return save;
                                }
                            } else {
                                sp0c = self.char_length(font, scale, code as u16, false);
                                msg_store(&mut sp18, sp14, (sp0c + 0.5f32) as i32);
                            }
                        } else {
                            self.now_line += 1;
                            sp14 = 0;
                            self.line_count = 0;
                            msg_store(&mut sp18, 0, 0);
                            sp0c = self.char_length(font, scale, code as u16, false);
                            msg_store(&mut sp18, sp14, (sp0c + 0.5f32) as i32);
                        }
                    }

                    if narrow {
                        pos += 1;
                    } else {
                        pos += 2;
                    }
                }
            }
        }

        if msg_at(&self.bmg, pos) == 0 {
            self.field_0x140 = self.aim_line;
        }

        if selecting {
            let v = msg_load(&sp18, sp14);
            msg_store(&mut self.field_0xe8, sp14, v);
        } else {
            for i in 0..4 {
                self.field_0xd8[i] = sp18[i];
            }
            if self.field_0x295 != 0 && self.line_count == 0 && scale_changed {
                self.line_count = 1;
            }
        }

        self.field_0x134 = self.line_count;

        save
    }

    pub fn string_shift(&mut self) {
        let align = self.entry.text_alignment;
        let shift = if self.entry.text_alignment != 3 && self.entry.textbox_type != 0xD {
            1
        } else {
            align
        };

        match shift {
            1 => {
                for i in 0..4 {
                    self.field_0xf8[i] = if self.field_0x108[i] != 0 {
                        self.field_0x108[i]
                    } else {
                        0
                    };
                }
            }
            0 | 3 => {
                for i in 0..4 {
                    self.field_0xf8[i] = if self.field_0x108[i] != 0 {
                        self.field_0x108[i]
                    } else {
                        (self.center_line_width - self.field_0xd8[i]) / 2
                    };
                }
            }
            2 => {
                for i in 0..4 {
                    self.field_0xf8[i] = self.line_width - self.field_0xd8[i] - 1;
                }
            }
            _ => {}
        }

        self.now_cursor_pos = self.field_0xf8[0] as f32;
        self.field_0x150 = 0;
        let buf = format!("\x1bCR[{}]", self.now_cursor_pos as i32);
        if let Some(s) = self.out[0].as_mut() {
            s.push_str(&buf);
        }
        if let Some(s) = self.out[2].as_mut() {
            s.push_str(&buf);
        }
        for i in 0..4 {
            if self.select_length < self.field_0xe8[i] {
                self.select_length = self.field_0xe8[i];
            }
        }
        if self.entry.textbox_type == 0x5 {
            self.short_cut();
        }
    }

    pub fn icon_idx_refresh(&mut self) {
        for i in 0..ICON_SLOTS {
            self.icon_num[i] = ICON_NONE;
        }
        self.field_0x280 = 0;
    }

    pub fn arrow_anime(&mut self) {
        let timer = self.field_0x260 as i32 + 1;
        self.field_0x260 = if timer >= ARROW_ANIME_WRAP {
            0
        } else {
            timer as i16
        };
    }

    pub fn select_check2(
        &mut self,
        pane_width: f32,
        pane_height: f32,
        arrow_size: i16,
        x: i32,
        y: i32,
        spacing: i32,
        stick_y: f32,
    ) -> u8 {
        self.field_0x264 = pane_width / 2.0f32;
        self.field_0x268 = pane_height / 2.0f32;
        self.field_0x278 = arrow_size as f32;
        self.field_0x274 = arrow_size as f32;

        if self.dec_key_wait_timer() == 0 {
            if stick_y > 0.7f32 {
                if self.field_0x27d == 1 {
                    self.field_0x27d = 0;
                    self.key_wait_timer = 10;
                }
            } else if stick_y < -0.7f32 {
                if self.field_0x27d == 0 {
                    self.field_0x27d = 1;
                    self.key_wait_timer = 10;
                }
            }
        }

        self.field_0x26c = x as f32;
        self.field_0x270 = (y + self.field_0x27d as i32 * spacing) as f32;
        self.field_0x27d
    }

    pub fn select_check3(
        &mut self,
        pane_width: f32,
        pane_height: f32,
        arrow_size: i16,
        x: i32,
        y: i32,
        spacing: i32,
        stick_y: f32,
    ) -> u8 {
        self.field_0x264 = pane_width / 2.0f32;
        self.field_0x268 = pane_height / 2.0f32;
        self.field_0x278 = arrow_size as f32;
        self.field_0x274 = arrow_size as f32;

        if self.dec_key_wait_timer() == 0 {
            if stick_y > 0.7f32 {
                if self.field_0x27d == 1 {
                    self.field_0x27d = 0;
                    self.key_wait_timer = 10;
                } else if self.field_0x27d == 2 {
                    self.field_0x27d = 1;
                    self.key_wait_timer = 10;
                }
            } else if stick_y < -0.7f32 {
                if self.field_0x27d == 0 {
                    self.field_0x27d = 1;
                    self.key_wait_timer = 10;
                } else if self.field_0x27d == 1 {
                    self.field_0x27d = 2;
                    self.key_wait_timer = 10;
                }
            }
        }

        self.field_0x26c = x as f32;
        self.field_0x270 = (y + self.field_0x27d as i32 * spacing) as f32;
        self.field_0x27d
    }

    pub fn select_check_yoko(
        &mut self,
        pane_width: f32,
        pane_height: f32,
        arrow_size: i16,
        x: i32,
        y: i32,
        spacing: i32,
        stick_x: f32,
    ) -> u8 {
        self.field_0x264 = pane_width / 2.0f32;
        self.field_0x268 = pane_height / 2.0f32;
        self.field_0x278 = arrow_size as f32;
        self.field_0x274 = arrow_size as f32;

        if self.dec_key_wait_timer() == 0 {
            if stick_x < -0.7f32 {
                if self.field_0x27d == 1 {
                    self.field_0x27d = 0;
                    self.key_wait_timer = 10;
                }
            } else if stick_x > 0.7f32 {
                if self.field_0x27d == 0 {
                    self.field_0x27d = 1;
                    self.key_wait_timer = 10;
                }
            }
        }

        self.field_0x26c = (x + self.field_0x27d as i32 * spacing) as f32;
        self.field_0x270 = y as f32;
        self.field_0x27d
    }

    pub fn input_number(
        &mut self,
        columns: i32,
        number: i16,
        buy_item_max: u8,
        stick: MsgStickFlags,
    ) -> MsgNumInput {
        let mut value = number;

        if stick.right_trigger {
            if self.field_0x27d != 0 {
                self.field_0x27d -= 1;
            }
        } else if stick.left_trigger {
            if (self.field_0x27d as i32) < columns - 1 {
                self.field_0x27d = self.field_0x27d.wrapping_add(1);
            }
        }

        if stick.up_trigger {
            if self.field_0x27d == 0 {
                if value < 999 {
                    value += 1;
                }
            } else if self.field_0x27d == 1 {
                if value < 999 {
                    value += 10;
                    if value > 999 {
                        value = 99;
                    }
                }
            } else if self.field_0x27d == 2 {
                if value < 999 {
                    value += 100;
                    if value > 999 {
                        value = 999;
                    }
                }
            }
        } else if stick.down_trigger {
            if self.field_0x27d == 0 {
                if value > 0 {
                    value -= 1;
                }
            } else if self.field_0x27d == 1 {
                if value > 0 {
                    value -= 10;
                    if value < 0 {
                        value = 0;
                    }
                }
            } else if self.field_0x27d == 2 {
                if value > 0 {
                    value -= 100;
                    if value < 0 {
                        value = 0;
                    }
                }
            }
        }

        if columns == 2 {
            if value as i32 > buy_item_max as i32 {
                value = buy_item_max as i16;
            } else if value < 1 {
                value = 1;
            }
        }

        MsgNumInput {
            index: self.field_0x27d,
            number: value,
        }
    }

    pub fn select_arrow(&mut self, _x: f32, _y: f32, _width: f32, _height: f32) {
        self.arrow_anime();
    }

    pub fn select_arrow_size(&mut self, _width: f32, _height: f32) {
        self.arrow_anime();
    }

    pub fn color_anime(&mut self) {
        self.arrow_anime();
    }
}
