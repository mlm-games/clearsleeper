use super::msglayout::{fop_msgm_int_to_char, fop_msgm_int_to_char2};
use super::{FontSlot, ICON_INPUT, MsgData, SELECT_ON};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MsgTagLen {
    pub len: i32,
    pub width: f32,
    pub line_width: i32,
}

fn msg_cr(value: i32) -> String {
    format!("\u{1b}CR[{}]", value)
}

impl MsgData {
    fn msg_tag_measure<F: FontSlot + ?Sized>(
        &self,
        font: &F,
        text: &str,
        scale: i32,
        len: i32,
        width: f32,
    ) -> MsgTagLen {
        let mut len = len;
        let mut width = width;
        for byte in text.bytes() {
            if len == 0 {
                width = self.char_length(font, scale, byte as u16, true);
            } else {
                width += self.char_length(font, scale, byte as u16, false);
            }
            len += 1;
        }
        MsgTagLen {
            len,
            width,
            line_width: (width + 0.5) as i32,
        }
    }

    fn msg_tag_line(&mut self, line: i32) {
        if line != self.line_count {
            self.line_count = line;
        }
    }

    fn msg_tag_push_out(&mut self, text: &str) {
        if let Some(s) = self.out[0].as_mut() {
            s.push_str(text);
        }
        if let Some(s) = self.out[2].as_mut() {
            s.push_str(text);
        }
    }

    fn msg_tag_cursor(&mut self) {
        let idx = self.line_count as usize;
        let base = if self.select_flag != SELECT_ON {
            self.field_0xf8.get(idx).copied().unwrap_or(0)
        } else {
            self.field_0x108.get(idx).copied().unwrap_or(0)
        };
        self.now_cursor_pos = base as f32 + self.field_0x14 + 0.5;
    }

    fn msg_tag_advance(&mut self) {
        let step = self.count.wrapping_add(1);
        self.count = self
            .count
            .wrapping_add(self.bmg.get(step as usize).copied().unwrap_or(0) as i32);
    }

    fn msg_tag_emit<F: FontSlot + ?Sized>(&mut self, font: &F, text: &str) {
        let scale = self.field_0x148;
        for byte in text.bytes() {
            if self.field_0x150 == 0 {
                let w = self.char_length(font, scale, byte as u16, true);
                self.field_0x14 = w;
            } else {
                let w = self.char_length(font, scale, byte as u16, false);
                self.field_0x14 += w;
            }
            self.field_0x150 += 1;
        }
        self.msg_tag_push_out(text);
        self.msg_tag_cursor();
    }

    pub fn tag_len_kaisen_game<F: FontSlot + ?Sized>(
        &mut self,
        font: &F,
        event_reg_beff: i32,
        len: i32,
        width: f32,
        scale: i32,
        line: i32,
    ) -> MsgTagLen {
        let mut buf = String::new();
        fop_msgm_int_to_char(&mut buf, event_reg_beff, false);
        self.msg_tag_measure(font, &buf, scale, len, width)
    }

    pub fn tag_len_rupee<F: FontSlot + ?Sized>(
        &mut self,
        font: &F,
        message_count_number: i32,
        len: i32,
        width: f32,
        scale: i32,
        line: i32,
    ) -> MsgTagLen {
        let mut buf = String::new();
        fop_msgm_int_to_char(&mut buf, message_count_number, false);
        if message_count_number != 1 {
            buf.push_str(" Rupees");
        } else {
            buf.push_str(" Rupee");
        }
        self.msg_tag_measure(font, &buf, scale, len, width)
    }

    pub fn tag_len_num_input<F: FontSlot + ?Sized>(
        &mut self,
        font: &F,
        len: i32,
        width: f32,
        scale: i32,
        line: i32,
    ) -> MsgTagLen {
        self.msg_tag_line(line);
        let buf = String::from("000 Rupee(s)");
        self.msg_tag_measure(font, &buf, scale, len, width)
    }

    pub fn tag_len_sword_game<F: FontSlot + ?Sized>(
        &mut self,
        font: &F,
        message_count_number: i32,
        len: i32,
        width: f32,
        scale: i32,
        line: i32,
    ) -> MsgTagLen {
        let mut buf = String::new();
        fop_msgm_int_to_char(&mut buf, message_count_number, false);
        if message_count_number != 1 {
            buf.push_str(" blows");
        } else {
            buf.push_str(" blow");
        }
        self.msg_tag_measure(font, &buf, scale, len, width)
    }

    pub fn tag_len_letter_game<F: FontSlot + ?Sized>(
        &mut self,
        font: &F,
        mini_game_rupee: i32,
        len: i32,
        width: f32,
        scale: i32,
        line: i32,
    ) -> MsgTagLen {
        let mut buf = String::new();
        fop_msgm_int_to_char(&mut buf, mini_game_rupee, false);
        self.msg_tag_measure(font, &buf, scale, len, width)
    }

    pub fn tag_len_letter_game_max<F: FontSlot + ?Sized>(
        &mut self,
        font: &F,
        event_reg_8aff: i32,
        len: i32,
        width: f32,
        scale: i32,
        line: i32,
    ) -> MsgTagLen {
        let mut buf = String::new();
        fop_msgm_int_to_char(&mut buf, event_reg_8aff, false);
        self.msg_tag_measure(font, &buf, scale, len, width)
    }

    pub fn tag_len_fish<F: FontSlot + ?Sized>(
        &mut self,
        font: &F,
        message_count_number: i32,
        len: i32,
        width: f32,
        scale: i32,
        line: i32,
    ) -> MsgTagLen {
        let mut buf = String::new();
        fop_msgm_int_to_char(&mut buf, message_count_number, false);
        self.msg_tag_measure(font, &buf, scale, len, width)
    }

    pub fn tag_len_fish_rupee<F: FontSlot + ?Sized>(
        &mut self,
        font: &F,
        message_count_number: i32,
        len: i32,
        width: f32,
        scale: i32,
        line: i32,
    ) -> MsgTagLen {
        let num = message_count_number * 10;
        let mut buf = String::new();
        fop_msgm_int_to_char(&mut buf, num, false);
        if num != 1 {
            buf.push_str(" Rupees");
        } else {
            buf.push_str(" Rupee");
        }
        self.msg_tag_measure(font, &buf, scale, len, width)
    }

    pub fn tag_len_letter<F: FontSlot + ?Sized>(
        &mut self,
        font: &F,
        message_count_number: i32,
        len: i32,
        width: f32,
        scale: i32,
        line: i32,
    ) -> MsgTagLen {
        let mut buf = String::new();
        fop_msgm_int_to_char(&mut buf, message_count_number, false);
        if message_count_number != 1 {
            buf.push_str(" letters");
        } else {
            buf.push_str(" letter");
        }
        self.msg_tag_measure(font, &buf, scale, len, width)
    }

    pub fn tag_len_rescue<F: FontSlot + ?Sized>(
        &mut self,
        font: &F,
        message_count_number: i32,
        len: i32,
        width: f32,
        scale: i32,
        line: i32,
    ) -> MsgTagLen {
        let mut buf = String::new();
        fop_msgm_int_to_char(&mut buf, message_count_number, false);
        self.msg_tag_measure(font, &buf, scale, len, width)
    }

    pub fn tag_len_forest_timer<F: FontSlot + ?Sized>(
        &mut self,
        font: &F,
        fwater_timer: i32,
        len: i32,
        width: f32,
        scale: i32,
        line: i32,
    ) -> MsgTagLen {
        let minutes = fwater_timer / 1800;
        let mut buf = String::new();
        fop_msgm_int_to_char(&mut buf, minutes, false);
        buf.push(':');
        let mut seconds = (fwater_timer % 1800) / 30;
        if minutes == 0 && seconds == 0 {
            seconds = 1;
        }
        fop_msgm_int_to_char2(&mut buf, seconds);
        self.msg_tag_measure(font, &buf, scale, len, width)
    }

    pub fn tag_len_birdman<F: FontSlot + ?Sized>(
        &mut self,
        font: &F,
        message_count_number: i32,
        len: i32,
        width: f32,
        scale: i32,
        line: i32,
    ) -> MsgTagLen {
        let mut buf = String::new();
        fop_msgm_int_to_char(&mut buf, message_count_number, false);
        if message_count_number != 1 {
            buf.push_str(" yards");
        } else {
            buf.push_str(" yard");
        }
        self.msg_tag_measure(font, &buf, scale, len, width)
    }

    pub fn tag_len_point<F: FontSlot + ?Sized>(
        &mut self,
        font: &F,
        event_reg_86ff: i32,
        len: i32,
        width: f32,
        scale: i32,
        line: i32,
    ) -> MsgTagLen {
        let mut buf = String::new();
        fop_msgm_int_to_char(&mut buf, event_reg_86ff, false);
        if event_reg_86ff != 1 {
            buf.push_str(" points");
        } else {
            buf.push_str(" point");
        }
        self.msg_tag_measure(font, &buf, scale, len, width)
    }

    pub fn tag_len_get_pendant<F: FontSlot + ?Sized>(
        &mut self,
        font: &F,
        beast_joy_pendant: i32,
        len: i32,
        width: f32,
        scale: i32,
        line: i32,
    ) -> MsgTagLen {
        let mut buf = String::new();
        fop_msgm_int_to_char(&mut buf, beast_joy_pendant, false);
        self.msg_tag_measure(font, &buf, scale, len, width)
    }

    pub fn tag_len_rev_pendant<F: FontSlot + ?Sized>(
        &mut self,
        font: &F,
        event_reg_c0ff: i32,
        len: i32,
        width: f32,
        scale: i32,
        line: i32,
    ) -> MsgTagLen {
        let mut buf = String::new();
        fop_msgm_int_to_char(&mut buf, event_reg_c0ff, false);
        self.msg_tag_measure(font, &buf, scale, len, width)
    }

    pub fn tag_len_pig_timer<F: FontSlot + ?Sized>(
        &mut self,
        font: &F,
        item_timer: i32,
        len: i32,
        width: f32,
        scale: i32,
        line: i32,
    ) -> MsgTagLen {
        let minutes = item_timer / 1800;
        let mut buf = String::new();
        fop_msgm_int_to_char(&mut buf, minutes, false);
        buf.push(':');
        let mut seconds = (item_timer % 1800) / 30;
        if minutes == 0 && seconds == 0 {
            seconds = 1;
        }
        fop_msgm_int_to_char2(&mut buf, seconds);
        self.msg_tag_measure(font, &buf, scale, len, width)
    }

    pub fn tag_len_get_bomb<F: FontSlot + ?Sized>(
        &mut self,
        font: &F,
        bomb_max: i32,
        len: i32,
        width: f32,
        scale: i32,
        line: i32,
    ) -> MsgTagLen {
        let mut buf = String::new();
        fop_msgm_int_to_char(&mut buf, bomb_max, false);
        if bomb_max != 1 {
            buf.push_str(" bombs");
        } else {
            buf.push_str(" bomb");
        }
        self.msg_tag_measure(font, &buf, scale, len, width)
    }

    pub fn tag_len_get_arrow<F: FontSlot + ?Sized>(
        &mut self,
        font: &F,
        arrow_max: i32,
        len: i32,
        width: f32,
        scale: i32,
        line: i32,
    ) -> MsgTagLen {
        let mut buf = String::new();
        fop_msgm_int_to_char(&mut buf, arrow_max, false);
        if arrow_max != 1 {
            buf.push_str(" arrows");
        } else {
            buf.push_str(" arrow");
        }
        self.msg_tag_measure(font, &buf, scale, len, width)
    }

    pub fn tag_len_stock_bokobaba<F: FontSlot + ?Sized>(
        &mut self,
        font: &F,
        buy_item: i32,
        len: i32,
        width: f32,
        scale: i32,
        line: i32,
    ) -> MsgTagLen {
        let mut buf = String::new();
        fop_msgm_int_to_char(&mut buf, buy_item, false);
        if buy_item != 1 {
            buf.push_str(" seeds");
        } else {
            buf.push_str(" seed");
        }
        self.msg_tag_measure(font, &buf, scale, len, width)
    }

    pub fn tag_len_stock_dokuro<F: FontSlot + ?Sized>(
        &mut self,
        font: &F,
        buy_item: i32,
        len: i32,
        width: f32,
        scale: i32,
        line: i32,
    ) -> MsgTagLen {
        let mut buf = String::new();
        fop_msgm_int_to_char(&mut buf, buy_item, false);
        if buy_item != 1 {
            buf.push_str(" neckalces");
        } else {
            buf.push_str(" neckalce");
        }
        self.msg_tag_measure(font, &buf, scale, len, width)
    }

    pub fn tag_len_stock_chuchu<F: FontSlot + ?Sized>(
        &mut self,
        font: &F,
        buy_item: i32,
        len: i32,
        width: f32,
        scale: i32,
        line: i32,
    ) -> MsgTagLen {
        let mut buf = String::new();
        fop_msgm_int_to_char(&mut buf, buy_item, false);
        self.msg_tag_measure(font, &buf, scale, len, width)
    }

    pub fn tag_len_stock_pendant<F: FontSlot + ?Sized>(
        &mut self,
        font: &F,
        buy_item: i32,
        len: i32,
        width: f32,
        scale: i32,
        line: i32,
    ) -> MsgTagLen {
        let mut buf = String::new();
        fop_msgm_int_to_char(&mut buf, buy_item, false);
        if buy_item != 1 {
            buf.push_str(" necklaces");
        } else {
            buf.push_str(" necklace");
        }
        self.msg_tag_measure(font, &buf, scale, len, width)
    }

    pub fn tag_len_stock_hane<F: FontSlot + ?Sized>(
        &mut self,
        font: &F,
        buy_item: i32,
        len: i32,
        width: f32,
        scale: i32,
        line: i32,
    ) -> MsgTagLen {
        let mut buf = String::new();
        fop_msgm_int_to_char(&mut buf, buy_item, false);
        if buy_item != 1 {
            buf.push_str(" feathers");
        } else {
            buf.push_str(" feather");
        }
        self.msg_tag_measure(font, &buf, scale, len, width)
    }

    pub fn tag_len_stock_kenshi<F: FontSlot + ?Sized>(
        &mut self,
        font: &F,
        buy_item: i32,
        len: i32,
        width: f32,
        scale: i32,
        line: i32,
    ) -> MsgTagLen {
        let mut buf = String::new();
        fop_msgm_int_to_char(&mut buf, buy_item, false);
        if buy_item != 1 {
            buf.push_str(" crests");
        } else {
            buf.push_str(" crest");
        }
        self.msg_tag_measure(font, &buf, scale, len, width)
    }

    pub fn tag_len_terry_rupee<F: FontSlot + ?Sized>(
        &mut self,
        font: &F,
        pay_rupee: i32,
        len: i32,
        width: f32,
        scale: i32,
        line: i32,
    ) -> MsgTagLen {
        let mut buf = String::new();
        fop_msgm_int_to_char(&mut buf, pay_rupee, false);
        if pay_rupee != 1 {
            buf.push_str(" Rupees");
        } else {
            buf.push_str(" Rupee");
        }
        self.msg_tag_measure(font, &buf, scale, len, width)
    }

    pub fn tag_len_input_bokobaba<F: FontSlot + ?Sized>(
        &mut self,
        font: &F,
        len: i32,
        width: f32,
        scale: i32,
        line: i32,
    ) -> MsgTagLen {
        self.msg_tag_line(line);
        let buf = String::from("00 seed(s)");
        self.msg_tag_measure(font, &buf, scale, len, width)
    }

    pub fn tag_len_input_dokuro<F: FontSlot + ?Sized>(
        &mut self,
        font: &F,
        len: i32,
        width: f32,
        scale: i32,
        line: i32,
    ) -> MsgTagLen {
        self.msg_tag_line(line);
        let buf = String::from("00 necklace(s)");
        self.msg_tag_measure(font, &buf, scale, len, width)
    }

    pub fn tag_len_input_chuchu<F: FontSlot + ?Sized>(
        &mut self,
        font: &F,
        len: i32,
        width: f32,
        scale: i32,
        line: i32,
    ) -> MsgTagLen {
        self.msg_tag_line(line);
        let buf = String::from("00 ");
        self.msg_tag_measure(font, &buf, scale, len, width)
    }

    pub fn tag_len_input_pendant<F: FontSlot + ?Sized>(
        &mut self,
        font: &F,
        len: i32,
        width: f32,
        scale: i32,
        line: i32,
    ) -> MsgTagLen {
        self.msg_tag_line(line);
        let buf = String::from("00 pendant(s)");
        self.msg_tag_measure(font, &buf, scale, len, width)
    }

    pub fn tag_len_input_hane<F: FontSlot + ?Sized>(
        &mut self,
        font: &F,
        len: i32,
        width: f32,
        scale: i32,
        line: i32,
    ) -> MsgTagLen {
        self.msg_tag_line(line);
        let buf = String::from("00 feather(s)");
        self.msg_tag_measure(font, &buf, scale, len, width)
    }

    pub fn tag_len_input_kenshi<F: FontSlot + ?Sized>(
        &mut self,
        font: &F,
        len: i32,
        width: f32,
        scale: i32,
        line: i32,
    ) -> MsgTagLen {
        self.msg_tag_line(line);
        let buf = String::from("00 crest(s)");
        self.msg_tag_measure(font, &buf, scale, len, width)
    }

    pub fn tag_kaisen_game<F: FontSlot + ?Sized>(&mut self, font: &F, event_reg_beff: i32) {
        let mut buf = String::new();
        fop_msgm_int_to_char(&mut buf, event_reg_beff, false);
        self.msg_tag_emit(font, &buf);
        self.get_ruby_string(font, b"", b"");
        self.msg_tag_advance();
    }

    pub fn tag_rupee<F: FontSlot + ?Sized>(&mut self, font: &F, message_count_number: i32) {
        let mut buf = String::new();
        fop_msgm_int_to_char(&mut buf, message_count_number, false);
        if message_count_number != 1 {
            buf.push_str(" Rupees");
        } else {
            buf.push_str(" Rupee");
        }
        self.msg_tag_emit(font, &buf);
        self.msg_tag_advance();
    }

    pub fn tag_num_input<F: FontSlot + ?Sized>(&mut self, font: &F) {
        let scale = self.field_0x148;
        let temp = (scale + self.char_space) * 3;
        self.select_auction_on();
        self.icon_select(scale, ICON_INPUT);
        self.mesg_status = super::STATUS_INPUT;
        self.msg_tag_push_out(&msg_cr(temp));
        self.field_0x14 = temp as f32;
        self.field_0x150 += 3;
        self.msg_tag_emit(font, " Rupee(s)");
        self.msg_tag_advance();
    }

    pub fn tag_sword_game<F: FontSlot + ?Sized>(&mut self, font: &F, message_count_number: i32) {
        let mut buf = String::new();
        fop_msgm_int_to_char(&mut buf, message_count_number, false);
        self.msg_tag_emit(font, &buf);
        let suffix = if message_count_number != 1 {
            " blows"
        } else {
            " blow"
        };
        self.get_ruby_string(font, suffix.as_bytes(), b"");
        self.msg_tag_advance();
    }

    pub fn tag_letter_game<F: FontSlot + ?Sized>(&mut self, font: &F, mini_game_rupee: i32) {
        let mut buf = String::new();
        fop_msgm_int_to_char(&mut buf, mini_game_rupee, false);
        self.msg_tag_emit(font, &buf);
        self.msg_tag_advance();
    }

    pub fn tag_letter_game_max<F: FontSlot + ?Sized>(&mut self, font: &F, event_reg_8aff: i32) {
        let mut buf = String::new();
        fop_msgm_int_to_char(&mut buf, event_reg_8aff, false);
        self.msg_tag_emit(font, &buf);
        self.msg_tag_advance();
    }

    pub fn tag_fish<F: FontSlot + ?Sized>(&mut self, font: &F, message_count_number: i32) {
        let mut buf = String::new();
        fop_msgm_int_to_char(&mut buf, message_count_number, false);
        self.msg_tag_emit(font, &buf);
        self.msg_tag_advance();
    }

    pub fn tag_fish_rupee<F: FontSlot + ?Sized>(&mut self, font: &F, message_count_number: i32) {
        let num = message_count_number * 10;
        let mut buf = String::new();
        fop_msgm_int_to_char(&mut buf, num, false);
        if num != 1 {
            buf.push_str(" Rupees");
        } else {
            buf.push_str(" Rupee");
        }
        self.msg_tag_emit(font, &buf);
        self.msg_tag_advance();
    }

    pub fn tag_letter<F: FontSlot + ?Sized>(&mut self, font: &F, message_count_number: i32) {
        let mut buf = String::new();
        fop_msgm_int_to_char(&mut buf, message_count_number, false);
        self.msg_tag_emit(font, &buf);
        let suffix = if message_count_number != 1 {
            " letters"
        } else {
            " letter"
        };
        self.get_ruby_string(font, suffix.as_bytes(), b"");
        self.msg_tag_advance();
    }

    pub fn tag_rescue<F: FontSlot + ?Sized>(&mut self, font: &F, message_count_number: i32) {
        let mut buf = String::new();
        fop_msgm_int_to_char(&mut buf, message_count_number, false);
        self.msg_tag_emit(font, &buf);
        self.get_ruby_string(font, b"", b"");
        self.msg_tag_advance();
    }

    pub fn tag_forest_timer<F: FontSlot + ?Sized>(&mut self, font: &F, fwater_timer: i32) {
        let minutes = fwater_timer / 1800;
        let mut buf = String::new();
        fop_msgm_int_to_char(&mut buf, minutes, false);
        self.msg_tag_emit(font, &buf);
        self.get_ruby_string(font, b":", b"");
        buf.clear();
        let mut seconds = (fwater_timer % 1800) / 30;
        if minutes == 0 && seconds == 0 {
            seconds = 1;
        }
        fop_msgm_int_to_char2(&mut buf, seconds);
        self.msg_tag_emit(font, &buf);
        self.get_ruby_string(font, b"", b"");
        self.msg_tag_advance();
    }

    pub fn tag_birdman<F: FontSlot + ?Sized>(&mut self, font: &F, message_count_number: i32) {
        let mut buf = String::new();
        fop_msgm_int_to_char(&mut buf, message_count_number, false);
        if message_count_number != 1 {
            buf.push_str(" yards");
        } else {
            buf.push_str(" yard");
        }
        self.msg_tag_emit(font, &buf);
        self.msg_tag_advance();
    }

    pub fn tag_point<F: FontSlot + ?Sized>(&mut self, font: &F, event_reg_86ff: i32) {
        let mut buf = String::new();
        fop_msgm_int_to_char(&mut buf, event_reg_86ff, false);
        if event_reg_86ff != 1 {
            buf.push_str(" points");
        } else {
            buf.push_str(" point");
        }
        self.msg_tag_emit(font, &buf);
        self.msg_tag_advance();
    }

    pub fn tag_get_pendant<F: FontSlot + ?Sized>(&mut self, font: &F, beast_joy_pendant: i32) {
        let mut buf = String::new();
        fop_msgm_int_to_char(&mut buf, beast_joy_pendant, false);
        self.msg_tag_emit(font, &buf);
        self.get_ruby_string(font, b"", b"");
        self.msg_tag_advance();
    }

    pub fn tag_rev_pendant<F: FontSlot + ?Sized>(&mut self, font: &F, event_reg_c0ff: i32) {
        let mut buf = String::new();
        fop_msgm_int_to_char(&mut buf, event_reg_c0ff, false);
        self.msg_tag_emit(font, &buf);
        self.get_ruby_string(font, b"", b"");
        self.msg_tag_advance();
    }

    pub fn tag_pig_timer<F: FontSlot + ?Sized>(&mut self, font: &F, item_timer: i32) {
        let minutes = item_timer / 1800;
        let mut buf = String::new();
        fop_msgm_int_to_char(&mut buf, minutes, false);
        self.msg_tag_emit(font, &buf);
        self.get_ruby_string(font, b":", b"");
        buf.clear();
        let mut seconds = (item_timer % 1800) / 30;
        if minutes == 0 && seconds == 0 {
            seconds = 1;
        }
        fop_msgm_int_to_char2(&mut buf, seconds);
        self.msg_tag_emit(font, &buf);
        self.get_ruby_string(font, b"", b"");
        self.msg_tag_advance();
    }

    pub fn tag_get_bomb<F: FontSlot + ?Sized>(&mut self, font: &F, bomb_max: i32) {
        let mut buf = String::new();
        fop_msgm_int_to_char(&mut buf, bomb_max, false);
        self.msg_tag_emit(font, &buf);
        let suffix = if bomb_max != 1 { " bombs" } else { " bomb" };
        self.get_ruby_string(font, suffix.as_bytes(), b"");
        self.msg_tag_advance();
    }

    pub fn tag_get_arrow<F: FontSlot + ?Sized>(&mut self, font: &F, arrow_max: i32) {
        let mut buf = String::new();
        fop_msgm_int_to_char(&mut buf, arrow_max, false);
        self.msg_tag_emit(font, &buf);
        let suffix = if arrow_max != 1 { " arrows" } else { " arrow" };
        self.get_ruby_string(font, suffix.as_bytes(), b"");
        self.msg_tag_advance();
    }

    pub fn tag_stock_bokobaba<F: FontSlot + ?Sized>(&mut self, font: &F, buy_item: i32) {
        let mut buf = String::new();
        fop_msgm_int_to_char(&mut buf, buy_item, false);
        self.msg_tag_emit(font, &buf);
        let suffix = if buy_item != 1 { " seeds" } else { " seed" };
        self.get_ruby_string(font, suffix.as_bytes(), b"");
        self.msg_tag_advance();
    }

    pub fn tag_stock_dokuro<F: FontSlot + ?Sized>(&mut self, font: &F, buy_item: i32) {
        let mut buf = String::new();
        fop_msgm_int_to_char(&mut buf, buy_item, false);
        self.msg_tag_emit(font, &buf);
        let suffix = if buy_item != 1 {
            " necklaces"
        } else {
            " necklace"
        };
        self.get_ruby_string(font, suffix.as_bytes(), b"");
        self.msg_tag_advance();
    }

    pub fn tag_stock_chuchu<F: FontSlot + ?Sized>(&mut self, font: &F, buy_item: i32) {
        let mut buf = String::new();
        fop_msgm_int_to_char(&mut buf, buy_item, false);
        self.msg_tag_emit(font, &buf);
        self.get_ruby_string(font, b"", b"");
        self.msg_tag_advance();
    }

    pub fn tag_stock_pendant<F: FontSlot + ?Sized>(&mut self, font: &F, buy_item: i32) {
        let mut buf = String::new();
        fop_msgm_int_to_char(&mut buf, buy_item, false);
        self.msg_tag_emit(font, &buf);
        let suffix = if buy_item != 1 {
            " necklaces"
        } else {
            " necklace"
        };
        self.get_ruby_string(font, suffix.as_bytes(), b"");
        self.msg_tag_advance();
    }

    pub fn tag_stock_hane<F: FontSlot + ?Sized>(&mut self, font: &F, buy_item: i32) {
        let mut buf = String::new();
        fop_msgm_int_to_char(&mut buf, buy_item, false);
        self.msg_tag_emit(font, &buf);
        let suffix = if buy_item != 1 {
            " feathers"
        } else {
            " feather"
        };
        self.get_ruby_string(font, suffix.as_bytes(), b"");
        self.msg_tag_advance();
    }

    pub fn tag_stock_kenshi<F: FontSlot + ?Sized>(&mut self, font: &F, buy_item: i32) {
        let mut buf = String::new();
        fop_msgm_int_to_char(&mut buf, buy_item, false);
        self.msg_tag_emit(font, &buf);
        let suffix = if buy_item != 1 { " crests" } else { " crest" };
        self.get_ruby_string(font, suffix.as_bytes(), b"");
        self.msg_tag_advance();
    }

    pub fn tag_terry_rupee<F: FontSlot + ?Sized>(&mut self, font: &F, pay_rupee: i32) {
        let mut buf = String::new();
        fop_msgm_int_to_char(&mut buf, pay_rupee, false);
        if pay_rupee != 1 {
            buf.push_str(" Rupees");
        } else {
            buf.push_str(" Rupee");
        }
        self.msg_tag_emit(font, &buf);
        self.msg_tag_advance();
    }

    pub fn tag_input_bokobaba<F: FontSlot + ?Sized>(&mut self, font: &F) {
        let scale = self.field_0x148;
        let temp = (scale + self.char_space) * 2;
        self.select_auction_on();
        self.icon_select(scale, ICON_INPUT);
        self.mesg_status = super::STATUS_INPUT;
        self.msg_tag_push_out(&msg_cr(temp));
        self.field_0x14 = temp as f32;
        self.field_0x150 += 2;
        self.msg_tag_cursor();
        self.get_ruby_string(font, b"seed(s)", b"");
        self.msg_tag_advance();
    }

    pub fn tag_input_dokuro<F: FontSlot + ?Sized>(&mut self, font: &F) {
        let scale = self.field_0x148;
        let temp = (scale + self.char_space) * 2;
        self.select_auction_on();
        self.icon_select(scale, ICON_INPUT);
        self.mesg_status = super::STATUS_INPUT;
        self.msg_tag_push_out(&msg_cr(temp));
        self.field_0x14 = temp as f32;
        self.field_0x150 += 2;
        self.msg_tag_cursor();
        self.get_ruby_string(font, b"necklace(s)", b"");
        self.msg_tag_advance();
    }

    pub fn tag_input_chuchu<F: FontSlot + ?Sized>(&mut self, font: &F) {
        let scale = self.field_0x148;
        let temp = (scale + self.char_space) * 2;
        self.select_auction_on();
        self.icon_select(scale, ICON_INPUT);
        self.mesg_status = super::STATUS_INPUT;
        self.msg_tag_push_out(&msg_cr(temp));
        self.field_0x14 = temp as f32;
        self.field_0x150 += 2;
        self.msg_tag_cursor();
        self.get_ruby_string(font, b"", b"");
        self.msg_tag_advance();
    }

    pub fn tag_input_pendant<F: FontSlot + ?Sized>(&mut self, font: &F) {
        let scale = self.field_0x148;
        let temp = (scale + self.char_space) * 2;
        self.select_auction_on();
        self.icon_select(scale, ICON_INPUT);
        self.mesg_status = super::STATUS_INPUT;
        self.msg_tag_push_out(&msg_cr(temp));
        self.field_0x14 = temp as f32;
        self.field_0x150 += 2;
        self.msg_tag_cursor();
        self.get_ruby_string(font, b"pendant(s)", b"");
        self.msg_tag_advance();
    }

    pub fn tag_input_hane<F: FontSlot + ?Sized>(&mut self, font: &F) {
        let scale = self.field_0x148;
        let temp = (scale + self.char_space) * 2;
        self.select_auction_on();
        self.icon_select(scale, ICON_INPUT);
        self.mesg_status = super::STATUS_INPUT;
        self.msg_tag_push_out(&msg_cr(temp));
        self.field_0x14 = temp as f32;
        self.field_0x150 += 2;
        self.msg_tag_cursor();
        self.get_ruby_string(font, b"feather(s)", b"");
        self.msg_tag_advance();
    }

    pub fn tag_input_kenshi<F: FontSlot + ?Sized>(&mut self, font: &F) {
        let scale = self.field_0x148;
        let temp = (scale + self.char_space) * 2;
        self.select_auction_on();
        self.icon_select(scale, ICON_INPUT);
        self.mesg_status = super::STATUS_INPUT;
        self.msg_tag_push_out(&msg_cr(temp));
        self.field_0x14 = temp as f32;
        self.field_0x150 += 2;
        self.msg_tag_cursor();
        self.get_ruby_string(font, b"crest(s)", b"");
        self.msg_tag_advance();
    }
}
