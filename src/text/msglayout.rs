use super::{
    CTRL_A_BUTTON, CTRL_ARROW_DOWN, CTRL_ARROW_LEFT, CTRL_ARROW_RIGHT, CTRL_ARROW_UP,
    CTRL_B_BUTTON, CTRL_C_STICK, CTRL_DPAD, CTRL_FLASHING_A_BUTTON, CTRL_HEART, CTRL_L_BUTTON,
    CTRL_MAIN_STICK, CTRL_MAIN_STICK_DOWN, CTRL_MAIN_STICK_LEFT, CTRL_MAIN_STICK_LEFT_RIGHT,
    CTRL_MAIN_STICK_RIGHT, CTRL_MAIN_STICK_UP, CTRL_MAIN_STICK_UP_DOWN, CTRL_MUSIC_NOTE,
    CTRL_PLAYER_NAME, CTRL_R_BUTTON, CTRL_SELECT_THREE, CTRL_SELECT_TWO, CTRL_SELECT_YOKO_LEFT,
    CTRL_SELECT_YOKO_RIGHT, CTRL_STARBURST, CTRL_UNK01, CTRL_UNK02, CTRL_UNK03, CTRL_UNK04,
    CTRL_UNK05, CTRL_UNK20, CTRL_UNK21, CTRL_UNK22, CTRL_UNK23, CTRL_UNK24, CTRL_UNK25,
    CTRL_UNK26, CTRL_UNK28, CTRL_UNK29, CTRL_UNK2B, CTRL_UNK2C, CTRL_UNK2D, CTRL_UNK2E,
    CTRL_UNK2F, CTRL_UNK30, CTRL_UNK31, CTRL_UNK32, CTRL_UNK33, CTRL_UNK34, CTRL_UNK35,
    CTRL_UNK36, CTRL_UNK37, CTRL_UNK38, CTRL_UNK3B, CTRL_UNK3C, CTRL_UNK3D, CTRL_UNK3E,
    CTRL_UNK3F, CTRL_UNK40, CTRL_UNK41, CTRL_UNK42, CTRL_UNK43, CTRL_UNK44, CTRL_UNK45,
    CTRL_UNK46, CTRL_UNK47, CTRL_UNK48, CTRL_UNK49, CTRL_UNK4A, CTRL_WAIT, CTRL_X_BUTTON,
    CTRL_DUMMY, CTRL_Y_BUTTON, CTRL_Z_BUTTON, FontSlot, ICON_A_BUTTON, ICON_B_BUTTON, ICON_C_STICK,
    ICON_DPAD, ICON_FLASHING_A_BUTTON_2, ICON_HEART_2, ICON_L_BUTTON, ICON_MAIN_STICK,
    ICON_MAIN_STICK_DOWN, ICON_MAIN_STICK_LEFT, ICON_MAIN_STICK_LEFT_RIGHT, ICON_MAIN_STICK_RIGHT,
    ICON_MAIN_STICK_UP, ICON_MAIN_STICK_UP_DOWN, ICON_MUSIC_NOTE_2, ICON_R_BUTTON,
    ICON_SELECT_YOKO_LEFT, ICON_SELECT_YOKO_RIGHT, ICON_STARBURST_2, ICON_X_BUTTON, ICON_Y_BUTTON,
    ICON_Z_BUTTON, MSG_MAX_SELECT, MsgData, SELECT_OFF, SELECT_ON, SPCL_COLOR, SPCL_RUBY,
    SPCL_UNK1, STATUS_CLOSE_WAIT, STATUS_INPUT, STATUS_MSG_CONTINUES, STATUS_MSG_DISPLAYED,
    STATUS_MSG_ENDS, STATUS_SELECT_2, STATUS_SELECT_3, STATUS_SELECT_YOKO, STATUS_STOP,
    STATUS_TYPING, TEXT_BUF, be32,
};

const PLURAL_NAME_MSG_NO: [u32; 6] = [0x33B, 0xC8B, 0x1D21, 0x31D7, 0x37DD, 0x37DE];

const COLOR_NAME_MSGNO: [u32; 9] = [
    0x000000FF,
    0xB40000FF,
    0x008282FF,
    0x0000AAFF,
    0xF0F01EFF,
    0x82FFFFFF,
    0x6400FFFF,
    0x505050FF,
    0xFFB400FF,
];

const COLOR_TEXTBOX_2_6_7: [u32; 9] = [
    0x00000000,
    0xB4000000,
    0x00828200,
    0x0000AA00,
    0xF0F01E00,
    0x82FFFF00,
    0x6400FF00,
    0x50505000,
    0xFFB40000,
];

const COLOR_TEXTBOX_0B: [u32; 9] = [
    0x000000FF,
    0xB40000FF,
    0x008282FF,
    0x0000AAFF,
    0xE67D0FFF,
    0x82FFFFFF,
    0x6400FFFF,
    0x3C3C3CFF,
    0xFFB400FF,
];

const NO_NAME_MSG: &[u8] = b"no name";

const ITEM_NO_TELESCOPE: u8 = 0x20;
const ITEM_NO_TINGLE_TUNER: u8 = 0x21;
const ITEM_NO_WIND_WAKER: u8 = 0x22;
const ITEM_NO_PICTO_BOX: u8 = 0x23;
const ITEM_NO_SPOILS_BAG: u8 = 0x24;
const ITEM_NO_GRAPPLING_HOOK: u8 = 0x25;
const ITEM_NO_BOW: u8 = 0x27;
const ITEM_NO_IRON_BOOTS: u8 = 0x29;
const ITEM_NO_MAGIC_ARMOR: u8 = 0x2A;
const ITEM_NO_BAIT_BAG: u8 = 0x2C;
const ITEM_NO_BOOMERANG: u8 = 0x2D;
const ITEM_NO_HOOKSHOT: u8 = 0x2F;
const ITEM_NO_DELIVERY_BAG: u8 = 0x30;
const ITEM_NO_BOMB_BAG: u8 = 0x31;
const ITEM_NO_SKULL_HAMMER: u8 = 0x33;
const ITEM_NO_DEKU_LEAF: u8 = 0x34;
const ITEM_NO_JOY_PENDANT: u8 = 0x1F;
const ITEM_NO_EMPTY_BOTTLE: u8 = 0x50;
const ITEM_NO_RED_POTION: u8 = 0x51;
const ITEM_NO_GREEN_POTION: u8 = 0x52;
const ITEM_NO_BLUE_POTION: u8 = 0x53;
const ITEM_NO_SKULL_NECKLACE: u8 = 0x45;
const ITEM_NO_BOKOBABA_SEED: u8 = 0x46;
const ITEM_NO_GOLDEN_FEATHER: u8 = 0x47;
const ITEM_NO_KNIGHTS_CREST: u8 = 0x48;
const ITEM_NO_RED_JELLY: u8 = 0x49;
const ITEM_NO_GREEN_JELLY: u8 = 0x4A;
const ITEM_NO_BLUE_JELLY: u8 = 0x4B;
const ITEM_NO_SAIL: u8 = 0x78;
const ITEM_NO_BIRD_BAIT_5: u8 = 0x82;
const ITEM_NO_HYOI_PEAR: u8 = 0x83;
const ITEM_NO_TOWN_FLOWER: u8 = 0x8C;
const ITEM_NO_SEA_FLOWER: u8 = 0x8D;
const ITEM_NO_EXOTIC_FLOWER: u8 = 0x8E;
const ITEM_NO_HEROS_FLAG: u8 = 0x8F;
const ITEM_NO_BIG_CATCH_FLAG: u8 = 0x90;
const ITEM_NO_BIG_SALE_FLAG: u8 = 0x91;
const ITEM_NO_PINWHEEL: u8 = 0x92;
const ITEM_NO_SICKLE_MOON_FLAG: u8 = 0x93;
const ITEM_NO_SKULL_TOWER_IDOL: u8 = 0x94;
const ITEM_NO_FOUNTAIN_IDOL: u8 = 0x95;
const ITEM_NO_POSTMAN_STATUE: u8 = 0x96;
const ITEM_NO_SHOP_GURU_STATUE: u8 = 0x97;
const ITEM_NO_NONE: u8 = 0xFF;

const ITEM_ICON: [u8; 60] = [
    ITEM_NO_TELESCOPE,
    ITEM_NO_SAIL,
    ITEM_NO_WIND_WAKER,
    ITEM_NO_GRAPPLING_HOOK,
    ITEM_NO_SPOILS_BAG,
    ITEM_NO_BOOMERANG,
    ITEM_NO_DEKU_LEAF,
    ITEM_NO_TINGLE_TUNER,
    ITEM_NO_PICTO_BOX,
    ITEM_NO_IRON_BOOTS,
    ITEM_NO_MAGIC_ARMOR,
    ITEM_NO_BAIT_BAG,
    ITEM_NO_BOW,
    ITEM_NO_BOMB_BAG,
    ITEM_NO_EMPTY_BOTTLE,
    ITEM_NO_RED_POTION,
    ITEM_NO_GREEN_POTION,
    ITEM_NO_BLUE_POTION,
    ITEM_NO_DELIVERY_BAG,
    ITEM_NO_HOOKSHOT,
    ITEM_NO_SKULL_HAMMER,
    ITEM_NO_MAGIC_ARMOR,
    ITEM_NO_MAGIC_ARMOR,
    ITEM_NO_MAGIC_ARMOR,
    ITEM_NO_SKULL_NECKLACE,
    ITEM_NO_BOKOBABA_SEED,
    ITEM_NO_GOLDEN_FEATHER,
    ITEM_NO_KNIGHTS_CREST,
    ITEM_NO_RED_JELLY,
    ITEM_NO_GREEN_JELLY,
    ITEM_NO_BLUE_JELLY,
    ITEM_NO_JOY_PENDANT,
    ITEM_NO_NONE,
    ITEM_NO_NONE,
    ITEM_NO_NONE,
    ITEM_NO_NONE,
    ITEM_NO_HYOI_PEAR,
    ITEM_NO_BIRD_BAIT_5,
    ITEM_NO_NONE,
    ITEM_NO_NONE,
    ITEM_NO_NONE,
    ITEM_NO_NONE,
    ITEM_NO_NONE,
    ITEM_NO_NONE,
    ITEM_NO_NONE,
    ITEM_NO_NONE,
    ITEM_NO_NONE,
    ITEM_NO_NONE,
    ITEM_NO_TOWN_FLOWER,
    ITEM_NO_SEA_FLOWER,
    ITEM_NO_EXOTIC_FLOWER,
    ITEM_NO_HEROS_FLAG,
    ITEM_NO_BIG_CATCH_FLAG,
    ITEM_NO_BIG_SALE_FLAG,
    ITEM_NO_PINWHEEL,
    ITEM_NO_SICKLE_MOON_FLAG,
    ITEM_NO_SKULL_TOWER_IDOL,
    ITEM_NO_FOUNTAIN_IDOL,
    ITEM_NO_POSTMAN_STATUE,
    ITEM_NO_SHOP_GURU_STATUE,
];



#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MsgSetFlow {
    pub set_continue_msg: bool,
    pub continue_msg_no: u16,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MsgLayoutState<'a> {
    pub player_name: &'a str,
    pub npc_name_msg_no: u32,
    pub npc_name_msg: &'a [u8],
    pub item_name_msg_no: u32,
    pub item_name_msg: &'a [u8],
    pub pirate_password_msg_no: u32,
    pub pirate_password_msg: &'a [u8],
    pub pal_language: i32,
    pub clear_count: u8,
    pub next_msg_flag: bool,
    pub msg_hio_0x6c: i8,
    pub color_clt1: &'a [u8],
    pub event_reg_beff: i32,
    pub event_reg_ba0f: i32,
    pub event_reg_8aff: i32,
    pub event_reg_86ff: i32,
    pub event_reg_c0ff: i32,
    pub message_count_number: i32,
    pub mini_game_rupee: i32,
    pub fwater_timer: i32,
    pub item_timer: i32,
    pub beast_joy_pendant: i32,
    pub bomb_max: i32,
    pub arrow_max: i32,
    pub buy_item: i32,
    pub pay_rupee: i32,
}

pub fn fop_msgm_int_to_char(dest: &mut String, mut value: i32, append: bool) {
    if !append {
        dest.clear();
    }
    let mut place = 10000i32;
    let mut started = false;
    for _ in 0..5 {
        if value / place != 0 || started || place == 1 {
            dest.push(((value / place) as u8).wrapping_add(b'0') as char);
            if !started {
                started = true;
            }
        }
        value %= place;
        place /= 10;
    }
}

pub fn fop_msgm_int_to_char2(dest: &mut String, value: i32) {
    dest.push(((value / 10) as u8).wrapping_add(b'0') as char);
    dest.push(((value % 10) as u8).wrapping_add(b'0') as char);
}

pub fn fop_msgm_item_num_idx(i_no: u8) -> u8 {
    ITEM_ICON.get(i_no as usize).copied().unwrap_or(0)
}

pub fn fop_msgm_item_num(i_item_no: u8) -> u8 {
    for (i, &item_no) in ITEM_ICON.iter().enumerate() {
        if i_item_no == item_no {
            return i as u8;
        }
    }
    0
}

pub fn fop_msgm_get_color_table(clt1: &[u8], color_no: u16) -> u32 {
    be32(clt1, 0x0C + 4 * color_no as usize).unwrap_or(0)
}

pub fn set_fontsize_center(out: &mut [Option<String>; MSG_MAX_SELECT], old_size: i32, new_size: i32) {
    let temp = (new_size - old_size) / 2;
    let buf1 = format!("\u{1b}FX[{new_size}]\u{1b}FY[{new_size}]");
    if temp > 0 {
        out_at(out, 0, &buf1);
        out_at(out, 2, &buf1);
        out_at(out, 1, "");
        out_at(out, 3, "");
    } else if temp < 0 {
        out_at(out, 0, &buf1);
        out_at(out, 2, &buf1);
        out_at(out, 1, "");
        out_at(out, 3, "");
    } else if old_size != new_size {
        out_at(out, 0, &buf1);
        out_at(out, 2, &buf1);
    }
}

pub fn set_fontsize_center2(
    out: &mut [Option<String>; MSG_MAX_SELECT],
    _old_size: i32,
    new_size: i32,
    _font_size: i32,
    _line_space: i32,
) {
    out_at(out, 0, &format!("\u{1b}FX[{new_size}]\u{1b}FY[{new_size}]"));
    out_at(out, 2, &format!("\u{1b}FX[{new_size}]\u{1b}FY[{new_size}]"));
    out_at(out, 1, "");
    out_at(out, 3, "");
}

fn out_at(out: &mut [Option<String>; MSG_MAX_SELECT], idx: usize, text: &str) {
    if let Some(slot) = out.get_mut(idx) {
        if let Some(s) = slot.as_mut() {
            s.push_str(text);
        }
    }
}

fn cstr_bytes(bytes: &[u8]) -> String {
    bytes.iter().copied().take_while(|&b| b != 0).map(|b| b as char).collect()
}

fn textbox_flashing(textbox_type: u8) -> bool {
    textbox_type == 0
        || textbox_type == 1
        || textbox_type == 5
        || textbox_type == 8
        || textbox_type == 0xC
        || textbox_type == 0xE
}

fn player_name_plural(name: &str, msg_no: u32, pal_language: i32, cap: usize) -> Vec<u8> {
    let mut bytes: Vec<u8> = name.as_bytes().iter().copied().take(cap).collect();
    if pal_language == 1 && PLURAL_NAME_MSG_NO.contains(&msg_no) {
        let last = bytes.last().copied().unwrap_or(0);
        if matches!(last, b's' | b'S' | b'z' | b'Z' | b'x' | b'X') {
            bytes.push(b'\'');
        } else {
            bytes.push(b's');
        }
    }
    bytes
}

impl MsgData {
    fn layout_at(&self, i: i32) -> u8 {
        self.bmg.get(i as usize).copied().unwrap_or(0)
    }

    fn layout_step(&mut self) {
        let step = self.layout_at(self.count.wrapping_add(1)) as i8 as i32;
        self.count = self.count.wrapping_add(step);
    }

    fn layout_out(&mut self, idx: usize, text: &str) {
        out_at(&mut self.out, idx, text);
    }

    fn layout_out_main(&mut self, text: &str) {
        self.layout_out(0, text);
        self.layout_out(2, text);
    }

    fn layout_out_ruby(&mut self, text: &str) {
        self.layout_out(1, text);
        self.layout_out(3, text);
    }

    fn layout_out_all(&mut self, text: &str) {
        for i in 0..MSG_MAX_SELECT {
            self.layout_out(i, text);
        }
    }

    fn layout_line_base(&self) -> i32 {
        let idx = self.line_count as usize;
        if self.select_flag != SELECT_ON {
            self.field_0xf8.get(idx).copied().unwrap_or(0)
        } else {
            self.field_0x108.get(idx).copied().unwrap_or(0)
        }
    }

    fn layout_cursor(&mut self) {
        let base = self.layout_line_base() as f32;
        self.now_cursor_pos = base + self.field_0x14 + 0.5f32;
    }

    fn layout_idle(&self) -> bool {
        self.field_0x299 == 0
            && self.field_0x29a == 0
            && self.auto_send_flag == 0
            && self.hand_send_flag == 0
            && self.select_flag == SELECT_OFF
    }

    fn layout_d4(&self) -> String {
        cstr_bytes(&self.field_0xd4)
    }

    fn layout_ruby_buf(&self) -> String {
        cstr_bytes(&self.field_0x70)
    }

    fn layout_select_start(&mut self, three: bool) {
        self.select_on();
        self.line_count = 0;
        self.now_cursor_pos = self.field_0x108.get(0).copied().unwrap_or(0) as f32;
        self.layout_step();
        self.mesg_status = if three { STATUS_SELECT_3 } else { STATUS_SELECT_2 };
    }

    fn layout_flashing_on(&mut self) {
        if textbox_flashing(self.entry.textbox_type) {
            self.field_0x296 = 1;
        }
    }

    fn layout_ruby_tail(&mut self) {
        if self.field_0x154 != 0 {
            self.field_0x154 -= 1;
            if self.field_0x154 == 0 {
                self.field_0x1c = self.now_cursor_pos - self.field_0x28;
                let f0 = self.field_0x28 + self.field_0x1c / 2.0f32;
                let f3 = ((f0 - self.field_0x18 / 2.0f32 + 0.5f32) as i32) as f32;
                if self.field_0x24 < f3 {
                    let f31 = f3 - self.field_0x24;
                    let s = format!("\u{1b}CR[{}]", (f31 + 0.5f32) as i32);
                    self.layout_out_ruby(&s);
                    self.field_0x24 += f31;
                } else if self.field_0x24 == 0.0f32 {
                    let f31 = self.field_0x24 - f3;
                    let s = format!("\u{1b}CL[{}]", (f31 + 0.5f32) as i32);
                    self.layout_out_ruby(&s);
                    self.field_0x24 -= f31;
                }
                let ruby = self.layout_ruby_buf();
                self.layout_out_ruby(&ruby);
                self.field_0x24 += self.field_0x18;
            }
        }
    }

    fn layout_finish(&mut self, state: &MsgLayoutState<'_>) -> MsgSetFlow {
        if self.mesg_status == STATUS_SELECT_2
            || self.mesg_status == STATUS_SELECT_3
            || self.mesg_status == STATUS_SELECT_YOKO
            || self.mesg_status == STATUS_INPUT
        {
            return MsgSetFlow::default();
        }

        let mut flow = MsgSetFlow::default();
        if state.next_msg_flag {
            if self.entry.next_msg_no != 0 {
                flow.set_continue_msg = true;
                flow.continue_msg_no = self.entry.next_msg_no;
                self.mesg_status = STATUS_MSG_CONTINUES;
            } else {
                self.mesg_status = STATUS_MSG_ENDS;
            }
        } else if self.auto_send_flag != 0 || self.hand_send_flag != 0 {
            self.mesg_status = STATUS_CLOSE_WAIT;
        } else {
            self.mesg_status = STATUS_MSG_DISPLAYED;
        }
        flow
    }

    pub fn get_string(
        &self,
        dest: &mut String,
        src: &[u8],
        msg_no: u32,
        player_name: &str,
        pal_language: i32,
    ) {
        let src: &[u8] = if msg_no == 0 { NO_NAME_MSG } else { src };
        let mut offset = 0usize;
        while src.get(offset).copied().unwrap_or(0) != 0 {
            let b = src[offset];
            if b == 0x1A {
                let code_len = src.get(offset.wrapping_add(1)).copied().unwrap_or(0);
                if src.get(offset.wrapping_add(2)).copied().unwrap_or(0) == 0
                    && src.get(offset.wrapping_add(3)).copied().unwrap_or(0) == 0
                    && src.get(offset.wrapping_add(4)).copied().unwrap_or(0) == CTRL_PLAYER_NAME
                {
                    let name = player_name_plural(player_name, msg_no, pal_language, 19);
                    for byte in name {
                        dest.push(byte as char);
                    }
                }
                offset = offset.wrapping_add(code_len as usize);
            } else {
                dest.push(b as char);
                offset = offset.wrapping_add(1);
            }
        }
    }

    pub fn get_string_layout<F: FontSlot + ?Sized>(
        &mut self,
        font: &F,
        src: &[u8],
        msg_no: u32,
        player_name: &str,
        pal_language: i32,
    ) {
        let src: &[u8] = if msg_no == 0 { NO_NAME_MSG } else { src };
        let mut offset = 0usize;
        let mut ruby_start = 0.0f32;
        let mut ruby_width = 0.0f32;
        let mut ruby_count = 0i32;
        let mut ruby_text = String::new();
        while src.get(offset).copied().unwrap_or(0) != 0 {
            let b = src[offset];
            if b == 0x1A {
                let code_len = src.get(offset.wrapping_add(1)).copied().unwrap_or(0);
                let b2 = src.get(offset.wrapping_add(2)).copied().unwrap_or(0);
                let b3 = src.get(offset.wrapping_add(3)).copied().unwrap_or(0);
                let b4 = src.get(offset.wrapping_add(4)).copied().unwrap_or(0);
                if b2 == 0 && b3 == 0 && b4 == 0 {
                    let name = player_name_plural(player_name, msg_no, pal_language, 19);
                    let mut i = 0usize;
                    while i < name.len() {
                        let hi_nibble = (name[i] >> 4) & 0xF;
                        let c: i32;
                        if hi_nibble == 8 || hi_nibble == 9 {
                            let lo = name.get(i.wrapping_add(1)).copied().unwrap_or(0);
                            let pair = [name[i], lo];
                            self.layout_out_main(&cstr_bytes(&pair));
                            c = ((name[i] as i32) << 8) | (lo as i32);
                            i = i.wrapping_add(2);
                        } else {
                            let single = [name[i]];
                            self.layout_out_main(&cstr_bytes(&single));
                            c = name[i] as i32;
                            i = i.wrapping_add(1);
                        }
                        let scale = self.field_0x148;
                        if self.field_0x150 == 0 {
                            let base = self.layout_line_base() as f32;
                            let w = self.char_length(font, scale, c as u16, true);
                            self.now_cursor_pos = w + base;
                        } else {
                            let w = self.char_length(font, scale, c as u16, false);
                            self.now_cursor_pos += w;
                        }
                        self.field_0x150 += 1;
                    }
                } else if b2 == 0xFF && b3 == 0 && b4 == SPCL_RUBY {
                    let mut r17 = 6usize;
                    ruby_width = 0.0f32;
                    ruby_start = self.now_cursor_pos;
                    ruby_count = src.get(offset.wrapping_add(5)).copied().unwrap_or(0) as i32;
                    ruby_text.clear();
                    while (code_len as i32) > (r17 as i32) {
                        let hi = src.get(offset.wrapping_add(r17)).copied().unwrap_or(0);
                        let lo = src.get(offset.wrapping_add(r17).wrapping_add(1)).copied().unwrap_or(0);
                        ruby_text.push(hi as char);
                        ruby_text.push(lo as char);
                        let c = ((hi as i32) << 8) | (lo as i32);
                        let w = self.ruby_length(font, c as u16, false);
                        ruby_width += w;
                        r17 = r17.wrapping_add(2);
                    }
                }
                offset = offset.wrapping_add(code_len as usize);
            } else {
                let hi_nibble = (b >> 4) & 0xF;
                let c: i32;
                if hi_nibble == 8 || hi_nibble == 9 {
                    let lo = src.get(offset.wrapping_add(1)).copied().unwrap_or(0);
                    let pair = [b, lo];
                    self.layout_out_main(&cstr_bytes(&pair));
                    c = ((b as i32) << 8) | (lo as i32);
                    offset = offset.wrapping_add(2);
                } else {
                    let single = [b];
                    self.layout_out_main(&cstr_bytes(&single));
                    c = b as i32;
                    offset = offset.wrapping_add(1);
                }
                let scale = self.field_0x148;
                if self.field_0x150 == 0 {
                    let base = self.field_0xf8.get(self.line_count as usize).copied().unwrap_or(0);
                    let w = self.char_length(font, scale, c as u16, true);
                    self.now_cursor_pos = w + base as f32;
                } else {
                    let w = self.char_length(font, scale, c as u16, false);
                    self.now_cursor_pos += w;
                }
                self.field_0x150 += 1;

                if ruby_count != 0 {
                    ruby_count -= 1;
                    if ruby_count == 0 {
                        let f0 = (self.now_cursor_pos - ruby_start) / 2.0f32;
                        let temp2_2 =
                            (ruby_start + f0 - (ruby_width / 2.0f32) + 0.5f32) as i32;
                        let temp2 = ((temp2_2 as f32 - self.field_0x24) + 0.5f32) as i32;
                        if temp2 > 0 {
                            let s = format!("\u{1b}CR[{temp2}]");
                            self.layout_out_ruby(&s);
                            self.field_0x24 += temp2 as f32;
                        } else if temp2 == 0 {
                            let s = format!("\u{1b}CL[{temp2}]");
                            self.layout_out_ruby(&s);
                            self.field_0x24 -= temp2 as f32;
                        }
                        self.field_0x24 += ruby_width;
                        self.layout_out_ruby(&ruby_text);
                    }
                }
            }
        }
    }

    pub fn get_ruby_string<F: FontSlot + ?Sized>(&mut self, font: &F, base: &[u8], ruby: &[u8]) {
        let start = self.now_cursor_pos;
        let scale = self.field_0x148;
        for b in base.iter().copied().take_while(|&b| b != 0) {
            if self.field_0x150 == 0 {
                let w = self.char_length(font, scale, b as u16, true);
                self.field_0x14 = w;
            } else {
                let w = self.char_length(font, scale, b as u16, false);
                self.field_0x14 += w;
            }
            self.field_0x150 += 1;
        }

        self.layout_out_main(&cstr_bytes(base));
        self.layout_cursor();

        let mut temp3 = 0.0f32;
        for b in ruby.iter().copied().take_while(|&b| b != 0) {
            let w = self.ruby_length(font, b as u16, false);
            temp3 += w;
        }

        let f0 = (self.now_cursor_pos - start) / 2.0f32;
        let temp2_2 = (start + f0 - (temp3 / 2.0f32) + 0.5f32) as i32;
        let temp2 = ((temp2_2 as f32 - self.field_0x24) + 0.5f32) as i32;
        if temp2 > 0 {
            let s = format!("\u{1b}CR[{temp2}]");
            self.layout_out_ruby(&s);
            self.field_0x24 += temp2 as f32;
        } else if temp2 == 0 {
            let s = format!("\u{1b}CL[{temp2}]");
            self.layout_out_ruby(&s);
            self.field_0x24 -= temp2 as f32;
        }
        self.field_0x24 += temp3;
        self.layout_out_ruby(&cstr_bytes(ruby));
    }

    pub fn string_set<F: FontSlot + ?Sized>(
        &mut self,
        font: &F,
        state: &MsgLayoutState<'_>,
    ) -> MsgSetFlow {
        let mut r30 = state.msg_hio_0x6c;

        self.mesg_status = STATUS_TYPING;

        if self.layout_at(self.count) == 0 {
            return self.layout_finish(state);
        }

        while self.layout_at(self.count) != 0 {
            let b0 = self.layout_at(self.count);
            if b0 == 0x1A {
                let b1 = self.layout_at(self.count.wrapping_add(1));
                let b2 = self.layout_at(self.count.wrapping_add(2));
                let b3 = self.layout_at(self.count.wrapping_add(3));
                let b4 = self.layout_at(self.count.wrapping_add(4));

                if b2 == 0xFF && b3 == 0 && b4 == SPCL_COLOR {
                    let idx = self.layout_at(self.count.wrapping_add(5));
                    let msg_no = self.entry.msg_no as u32;
                    if (0x42..=0x4B).contains(&msg_no) {
                        self.string_color =
                            COLOR_NAME_MSGNO.get(idx as usize).copied().unwrap_or(0);
                        let s =
                            format!("\u{1b}CC[{:08x}]\u{1b}GM[0]", self.string_color);
                        self.layout_out_main(&s);
                    } else if (idx as i8) >= 0 && idx <= 8 {
                        let textbox_type = self.entry.textbox_type;
                        if textbox_type == 2 || textbox_type == 6 || textbox_type == 7 {
                            self.string_color =
                                COLOR_TEXTBOX_2_6_7.get(idx as usize).copied().unwrap_or(0);
                            let r5 = self.string_color | self.field_0x290 as u32;
                            let r6 = self.string_color | self.field_0x291 as u32;
                            let s = format!("\u{1b}CC[{:08x}]\u{1b}GC[{:08x}]", r5, r6);
                            self.layout_out_main(&s);
                        } else if textbox_type == 0xB {
                            self.string_color =
                                COLOR_TEXTBOX_0B.get(idx as usize).copied().unwrap_or(0);
                            let s =
                                format!("\u{1b}CC[{:08x}]\u{1b}GM[0]", self.string_color);
                            self.layout_out_main(&s);
                        } else {
                            self.string_color =
                                fop_msgm_get_color_table(state.color_clt1, idx as u16);
                            let s =
                                format!("\u{1b}CC[{:08x}]\u{1b}GM[0]", self.string_color);
                            self.layout_out_main(&s);
                        }
                    }
                    self.layout_step();
                } else if b2 == 0xFF && b3 == 0 && b4 == SPCL_UNK1 {
                    let c = ((self.layout_at(self.count.wrapping_add(5)) as i32) << 8)
                        | (self.layout_at(self.count.wrapping_add(6)) as i32);
                    let f31 = c as f32 * 0.01f32;
                    let temp3 = self.field_0x148;
                    let font_size = self.font_size;
                    self.field_0x148 = (font_size as f32 * f31 + 0.5f32) as i32;
                    if self.field_0x134 == 0 {
                        let new_size = self.field_0x148;
                        set_fontsize_center(&mut self.out, temp3, new_size);
                        if f31 > 1.0f32 && self.field_0x29d == 0 {
                            self.field_0x29d = 1;
                        }
                    } else {
                        if f31 > 1.0f32 && self.field_0x29d == 0 {
                            self.layout_out_all("\n");
                            let s = format!("\u{1b}CR[{}]", self.now_cursor_pos as i32);
                            self.layout_out_main(&s);
                            self.field_0x29d = 1;
                        }
                        let new_size = self.field_0x148;
                        set_fontsize_center2(
                            &mut self.out,
                            temp3,
                            new_size,
                            font_size,
                            self.line_space,
                        );
                    }
                    self.layout_step();
                } else if b2 == 0xFF && b3 == 0 && b4 == SPCL_RUBY {
                    let code_len = b1 as i32;
                    if code_len != 5 {
                        let end = self.count.wrapping_add(code_len);
                        self.field_0x70 = [0; TEXT_BUF];
                        self.field_0x18 = 0.0f32;
                        self.field_0x1c = 0.0f32;
                        self.field_0x28 = self.now_cursor_pos;
                        self.field_0x154 =
                            self.layout_at(self.count.wrapping_add(5)) as i32;
                        self.count = self.count.wrapping_add(6);
                        let mut acc = 0usize;
                        while end > self.count {
                            let hi = self.layout_at(self.count);
                            let lo = self.layout_at(self.count.wrapping_add(1));
                            if acc + 1 < TEXT_BUF {
                                self.field_0x70[acc] = hi;
                                self.field_0x70[acc + 1] = lo;
                            }
                            acc += 2;
                            let c = ((hi as i32) << 8) | (lo as i32);
                            if self.field_0x29b == 0 {
                                let w = self.ruby_length(font, c as u16, true);
                                self.field_0x18 = w;
                                self.field_0x29b = 1;
                            } else {
                                let w = self.ruby_length(font, c as u16, false);
                                self.field_0x18 += w;
                            }
                            self.count = self.count.wrapping_add(2);
                        }
                    } else {
                        self.layout_step();
                    }
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_PLAYER_NAME {
                    let name = player_name_plural(
                        state.player_name,
                        self.entry.msg_no as u32,
                        state.pal_language,
                        19,
                    );
                    let mut i = 0usize;
                    while i < name.len() {
                        let hi_nibble = name[i] >> 4;
                        let c: i32;
                        if hi_nibble == 8 || hi_nibble == 9 {
                            let hi = name[i];
                            let lo = name.get(i.wrapping_add(1)).copied().unwrap_or(0);
                            self.field_0xd4[0] = hi;
                            self.field_0xd4[1] = lo;
                            self.field_0xd4[2] = 0;
                            c = ((hi as i32) << 8) | (lo as i32);
                            i = i.wrapping_add(2);
                        } else {
                            c = name[i] as i32;
                            self.field_0xd4[0] = name[i];
                            self.field_0xd4[1] = 0;
                            i = i.wrapping_add(1);
                        }
                        let scale = self.field_0x148;
                        if self.field_0x150 == 0 {
                            let w = self.char_length(font, scale, c as u16, true);
                            self.field_0x14 = w;
                        } else {
                            let w = self.char_length(font, scale, c as u16, false);
                            self.field_0x14 += w;
                        }
                        let d4 = self.layout_d4();
                        self.layout_out_main(&d4);
                        self.field_0x150 += 1;
                    }
                    self.layout_cursor();
                    self.layout_step();
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_UNK01 {
                    self.field_0x29a = 1;
                    self.layout_step();
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_UNK02 {
                    self.field_0x29a = 0;
                    self.layout_step();
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_UNK03 {
                    let v = ((self.layout_at(self.count.wrapping_add(5)) as i32) << 8)
                        | (self.layout_at(self.count.wrapping_add(6)) as i32);
                    self.wait_timer = v;
                    self.auto_send_flag = 1;
                    self.layout_step();
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_UNK04 {
                    self.layout_flashing_on();
                    let v = ((self.layout_at(self.count.wrapping_add(5)) as i32) << 8)
                        | (self.layout_at(self.count.wrapping_add(6)) as i32);
                    self.wait_timer = v;
                    self.auto_send_flag = 1;
                    self.layout_step();
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_UNK05 {
                    let v = ((self.layout_at(self.count.wrapping_add(5)) as i32) << 8)
                        | (self.layout_at(self.count.wrapping_add(6)) as i32);
                    self.wait_timer = v;
                    self.hand_send_flag = 1;
                    self.layout_step();
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_DUMMY {
                    self.layout_step();
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_WAIT {
                    let v = ((self.layout_at(self.count.wrapping_add(5)) as i32) << 8)
                        | (self.layout_at(self.count.wrapping_add(6)) as i32);
                    if self.field_0x299 == 0 {
                        self.layout_flashing_on();
                        self.wait_timer = v;
                        self.layout_step();
                        return MsgSetFlow::default();
                    }
                    if textbox_flashing(self.entry.textbox_type) {
                        self.field_0x296 = 1;
                        self.wait_timer = v;
                        self.layout_step();
                        return MsgSetFlow::default();
                    }
                    self.layout_step();
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_SELECT_TWO {
                    self.layout_select_start(false);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_SELECT_THREE {
                    self.layout_select_start(true);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_A_BUTTON {
                    let scale = self.field_0x148;
                    self.icon_select(scale, ICON_A_BUTTON);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_B_BUTTON {
                    let scale = self.field_0x148;
                    self.icon_select(scale, ICON_B_BUTTON);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_C_STICK {
                    let scale = self.field_0x148;
                    self.icon_select(scale, ICON_C_STICK);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_L_BUTTON {
                    let scale = self.field_0x148;
                    self.icon_select(scale, ICON_L_BUTTON);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_R_BUTTON {
                    let scale = self.field_0x148;
                    self.icon_select(scale, ICON_R_BUTTON);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_X_BUTTON {
                    let scale = self.field_0x148;
                    self.icon_select(scale, ICON_X_BUTTON);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_Y_BUTTON {
                    let scale = self.field_0x148;
                    self.icon_select(scale, ICON_Y_BUTTON);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_Z_BUTTON {
                    let scale = self.field_0x148;
                    self.icon_select(scale, ICON_Z_BUTTON);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_DPAD {
                    let scale = self.field_0x148;
                    self.icon_select(scale, ICON_DPAD);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_MAIN_STICK {
                    let scale = self.field_0x148;
                    self.icon_select(scale, ICON_MAIN_STICK);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_ARROW_LEFT {
                    let scale = self.field_0x148;
                    self.icon_select(scale, ICON_ARROW_LEFT);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_ARROW_RIGHT {
                    let scale = self.field_0x148;
                    self.icon_select(scale, ICON_ARROW_RIGHT);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_ARROW_UP {
                    let scale = self.field_0x148;
                    self.icon_select(scale, ICON_ARROW_UP);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_ARROW_DOWN {
                    let scale = self.field_0x148;
                    self.icon_select(scale, ICON_ARROW_DOWN);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_MAIN_STICK_UP {
                    let scale = self.field_0x148;
                    self.icon_select(scale, ICON_MAIN_STICK_UP);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_MAIN_STICK_DOWN {
                    let scale = self.field_0x148;
                    self.icon_select(scale, ICON_MAIN_STICK_DOWN);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_MAIN_STICK_LEFT {
                    let scale = self.field_0x148;
                    self.icon_select(scale, ICON_MAIN_STICK_LEFT);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_MAIN_STICK_RIGHT {
                    let scale = self.field_0x148;
                    self.icon_select(scale, ICON_MAIN_STICK_RIGHT);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_MAIN_STICK_UP_DOWN {
                    let scale = self.field_0x148;
                    self.icon_select(scale, ICON_MAIN_STICK_UP_DOWN);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_MAIN_STICK_LEFT_RIGHT {
                    let scale = self.field_0x148;
                    self.icon_select(scale, ICON_MAIN_STICK_LEFT_RIGHT);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_SELECT_YOKO_LEFT {
                    self.select_yoko_on();
                    let scale = self.field_0x148;
                    self.icon_select(scale, ICON_SELECT_YOKO_LEFT);
                    self.mesg_status = STATUS_SELECT_YOKO;
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_SELECT_YOKO_RIGHT {
                    if self.select_flag != SELECT_OFF {
                        self.field_0x27d = 1;
                    } else {
                        self.field_0x27d = 0;
                    }
                    let scale = self.field_0x148;
                    self.icon_select(scale, ICON_SELECT_YOKO_RIGHT);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_UNK20 {
                    self.tag_kaisen_game(font, state.event_reg_beff);
                } else if b2 == 0
                    && b3 == 0
                    && (b4 == CTRL_UNK21
                        || b4 == CTRL_UNK24
                        || b4 == CTRL_UNK25
                        || b4 == CTRL_UNK2C
                        || b4 == CTRL_UNK36)
                {
                    self.tag_rupee(font, state.message_count_number);
                } else if b2 == 0 && b3 == 0 && (b4 == CTRL_UNK22 || b4 == CTRL_UNK23) {
                    if b4 == CTRL_UNK22 {
                        self.get_string_layout(
                            font,
                            state.npc_name_msg,
                            state.npc_name_msg_no,
                            state.player_name,
                            state.pal_language,
                        );
                    } else if b4 == CTRL_UNK23 {
                        self.get_string_layout(
                            font,
                            state.item_name_msg,
                            state.item_name_msg_no,
                            state.player_name,
                            state.pal_language,
                        );
                    }
                    let base = self.layout_line_base() as f32;
                    self.field_0x14 = self.now_cursor_pos - base;
                    self.layout_step();
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_UNK26 {
                    self.tag_num_input(font);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_FLASHING_A_BUTTON {
                    let scale = self.field_0x148;
                    self.icon_select(scale, ICON_FLASHING_A_BUTTON_2);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_UNK28 {
                    self.tag_sword_game(font, state.message_count_number);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_UNK29 {
                    let msg_no = 0x1B37u32.wrapping_add(state.event_reg_ba0f as u32);
                    self.get_string_layout(
                        font,
                        state.pirate_password_msg,
                        msg_no,
                        state.player_name,
                        state.pal_language,
                    );
                    let base = self.layout_line_base() as f32;
                    self.field_0x14 = self.now_cursor_pos - base;
                    self.layout_step();
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_STARBURST {
                    let scale = self.field_0x148;
                    self.icon_select(scale, ICON_STARBURST_2);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_UNK2B {
                    self.tag_letter_game(font, state.mini_game_rupee);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_UNK3B {
                    self.tag_letter_game_max(font, state.event_reg_8aff);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_UNK3C {
                    self.tag_fish(font, state.message_count_number);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_UNK3D {
                    self.tag_fish_rupee(font, state.message_count_number);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_UNK2D {
                    self.tag_letter(font, state.message_count_number);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_UNK2E {
                    self.tag_rescue(font, state.message_count_number);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_UNK2F {
                    self.tag_forest_timer(font, state.fwater_timer);
                } else if b2 == 0 && b3 == 0 && (b4 == CTRL_UNK30 || b4 == CTRL_UNK31) {
                    self.tag_birdman(font, state.message_count_number);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_UNK32 {
                    self.tag_point(font, state.event_reg_86ff);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_UNK33 {
                    self.tag_get_pendant(font, state.beast_joy_pendant);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_UNK34 {
                    self.tag_rev_pendant(font, state.event_reg_c0ff);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_UNK35 {
                    self.tag_pig_timer(font, state.item_timer);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_UNK37 {
                    self.tag_get_bomb(font, state.bomb_max);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_UNK38 {
                    self.tag_get_arrow(font, state.arrow_max);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_UNK3E {
                    self.tag_stock_bokobaba(font, state.buy_item);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_UNK3F {
                    self.tag_stock_dokuro(font, state.buy_item);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_UNK40 {
                    self.tag_stock_chuchu(font, state.buy_item);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_UNK41 {
                    self.tag_stock_pendant(font, state.buy_item);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_UNK42 {
                    self.tag_stock_hane(font, state.buy_item);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_UNK43 {
                    self.tag_stock_kenshi(font, state.buy_item);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_UNK44 {
                    self.tag_terry_rupee(font, state.pay_rupee);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_UNK45 {
                    self.tag_input_bokobaba(font);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_UNK46 {
                    self.tag_input_dokuro(font);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_UNK47 {
                    self.tag_input_chuchu(font);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_UNK48 {
                    self.tag_input_pendant(font);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_UNK49 {
                    self.tag_input_hane(font);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_UNK4A {
                    self.tag_input_kenshi(font);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_HEART {
                    let scale = self.field_0x148;
                    self.icon_select(scale, ICON_HEART_2);
                } else if b2 == 0 && b3 == 0 && b4 == CTRL_MUSIC_NOTE {
                    let scale = self.field_0x148;
                    self.icon_select(scale, ICON_MUSIC_NOTE_2);
                } else if b2 == 1 {
                    self.layout_step();
                    return MsgSetFlow::default();
                } else if b2 == 2 {
                    self.layout_step();
                } else if b2 == 3 {
                    self.layout_step();
                } else {
                    self.layout_step();
                }
            } else if b0 == 0x0A {
                if self.field_0x154 != 0 {
                    self.field_0x1c = self.now_cursor_pos - self.field_0x28;
                    let f0 = self.field_0x28 + self.field_0x1c / 2.0f32;
                    let f3 = ((f0 - self.field_0x18 / 2.0f32 + 0.5f32) as i32) as f32;
                    if self.field_0x24 < f3 {
                        let f31 = f3 - self.field_0x24;
                        let s = format!("\u{1b}CR[{}]", (f31 + 0.5f32) as i32);
                        self.layout_out_ruby(&s);
                    } else if self.field_0x24 == 0.0f32 {
                        let f31 = self.field_0x24 - f3;
                        let s = format!("\u{1b}CL[{}]", (f31 + 0.5f32) as i32);
                        self.layout_out_ruby(&s);
                    }
                    let ruby = self.layout_ruby_buf();
                    self.layout_out_ruby(&ruby);
                    self.field_0x154 = 0;
                }
                self.layout_out_all("\n");
                self.count = self.count.wrapping_add(1);
                self.line_count += 1;
                if self.field_0x29d != 0 {
                    self.line_count += 1;
                    self.field_0x29d = 0;
                }
                self.field_0x14 = 0.0f32;
                self.field_0x24 = 0.0f32;
                self.field_0x29b = 0;
                if self.select_flag != SELECT_ON {
                    if self.line_count >= self.entry.field_0x16 as i32 {
                        let n0 = self.layout_at(self.count);
                        let n2 = self.layout_at(self.count.wrapping_add(2));
                        let n3 = self.layout_at(self.count.wrapping_add(3));
                        let n4 = self.layout_at(self.count.wrapping_add(4));
                        if n0 == 0x1A && n2 == 0 && n3 == 0 && n4 == CTRL_SELECT_TWO {
                            self.layout_select_start(false);
                            continue;
                        } else if n0 == 0x1A && n2 == 0 && n3 == 0 && n4 == CTRL_SELECT_THREE {
                            self.layout_select_start(true);
                            continue;
                        } else if n0 != 0x00 {
                            self.mesg_status = STATUS_STOP;
                        }
                        self.field_0x150 = 0;
                        self.field_0x299 = 0;
                        self.field_0x29a = 0;
                        return MsgSetFlow::default();
                    }
                    self.field_0x150 = 0;
                    self.now_cursor_pos = self.layout_line_base() as f32;
                    let s = format!("\u{1b}CR[{}]", self.now_cursor_pos as i32);
                    self.layout_out_main(&s);
                    if self.auto_send_flag == 0 {
                        self.wait_timer = self.space_timer;
                    }
                    if self.layout_idle() {
                        return MsgSetFlow::default();
                    }
                } else {
                    self.field_0x150 = 0;
                    self.now_cursor_pos = self.layout_line_base() as f32;
                    let s = format!("\u{1b}CR[{}]", self.now_cursor_pos as i32);
                    self.layout_out_main(&s);
                }
            } else {
                let hi_nibble = (b0 >> 4) & 0xF;
                let mut code: i32;
                let narrow: bool;
                if hi_nibble == 8 || hi_nibble == 9 {
                    let lo = self.layout_at(self.count.wrapping_add(1));
                    self.field_0xd4[0] = b0;
                    self.field_0xd4[1] = lo;
                    self.field_0xd4[2] = 0;
                    code = ((b0 as i32) << 8) | (lo as i32);
                    if state.clear_count == 0 && self.entry.textbox_type == 0xC {
                        for pair in super::ZFONT.iter() {
                            if pair[0] == code as u16 {
                                self.field_0xd4[0] = (pair[1] >> 8) & 0xFF;
                                self.field_0xd4[1] = pair[1] & 0xFF;
                                self.field_0xd4[2] = 0;
                                code = pair[1] as i32;
                                break;
                            }
                        }
                    }
                    narrow = false;
                } else {
                    self.field_0xd4[0] = b0;
                    self.field_0xd4[1] = 0;
                    code = b0 as i32;
                    narrow = true;
                }

                let scale = self.field_0x148;
                if self.field_0x150 == 0 {
                    let w = self.char_length(font, scale, code as u16, true);
                    self.field_0x14 = w;
                } else {
                    let w = self.char_length(font, scale, code as u16, false);
                    self.field_0x14 += w;
                }
                self.field_0x150 += 1;
                self.layout_cursor();

                if self.line_width <= (self.field_0x14 + 0.5f32) as i32 {
                    self.layout_out_all("\n");
                    self.line_count += 1;
                    self.field_0x24 = 0.0f32;
                    let w = self.char_length(font, scale, code as u16, true);
                    self.field_0x14 = w;
                    if self.select_flag != SELECT_ON {
                        if self.line_count >= self.entry.field_0x16 as i32 {
                            let n0 = self.layout_at(self.count);
                            let n2 = self.layout_at(self.count.wrapping_add(2));
                            let n3 = self.layout_at(self.count.wrapping_add(3));
                            let n4 = self.layout_at(self.count.wrapping_add(4));
                            if n0 == 0x1A && n2 == 0 && n3 == 0 && n4 == CTRL_SELECT_TWO {
                                self.layout_select_start(false);
                            } else if n0 == 0x1A && n2 == 0 && n3 == 0 && n4 == CTRL_SELECT_THREE {
                                self.layout_select_start(true);
                            } else {
                                if self.layout_at(self.count) != 0x00 {
                                    self.mesg_status = STATUS_STOP;
                                }
                                self.field_0x299 = 0;
                                self.field_0x29a = 0;
                                return MsgSetFlow::default();
                            }
                        } else {
                            self.field_0x150 = 0;
                            self.now_cursor_pos = self.layout_line_base() as f32;
                            let s = format!("\u{1b}CR[{}]", self.now_cursor_pos as i32);
                            self.layout_out_main(&s);
                            if self.layout_idle() {
                                return MsgSetFlow::default();
                            }
                        }
                    } else {
                        self.field_0x150 = 0;
                        self.now_cursor_pos = self.layout_line_base() as f32;
                        let s = format!("\u{1b}CR[{}]", self.now_cursor_pos as i32);
                        self.layout_out_main(&s);
                    }
                }

                self.layout_ruby_tail();

                self.count = self.count.wrapping_add(if narrow { 1 } else { 2 });

                let d4 = self.layout_d4();
                self.layout_out_main(&d4);

                if self.auto_send_flag == 0 {
                    if code == 0x8140 || code == 0x8141 || code == 0x878C {
                        self.wait_timer = self.space_timer;
                    }
                    if code == 0x20 && self.space_flag != 0 {
                        self.wait_timer = self.space_timer;
                    }
                }
                if self.layout_idle() {
                    if r30 == 0 {
                        return MsgSetFlow::default();
                    }
                    r30 = r30.wrapping_sub(1);
                }
            }
        }

        if self.layout_idle() {
            return MsgSetFlow::default();
        }
        if self.layout_at(self.count) != 0 {
            return MsgSetFlow::default();
        }
        self.layout_finish(state)
    }
}
