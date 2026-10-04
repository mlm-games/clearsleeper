use std::collections::HashMap;

use bevy_ecs::prelude::*;

use super::intent::DoIntent;

pub const MSG_PREPARING: u8 = 0x01;
pub const MSG_BOX_OPENING: u8 = 0x02;
pub const MSG_TYPING: u8 = 0x06;
pub const MSG_STOP: u8 = 0x07;
pub const MSG_CLOSE_WAIT: u8 = 0x0A;
pub const MSG_DISPLAYED: u8 = 0x0E;
pub const MSG_CONTINUES: u8 = 0x0F;
pub const MSG_ENDS: u8 = 0x10;
pub const MSG_BOX_CLOSING: u8 = 0x11;
pub const MSG_BOX_CLOSED: u8 = 0x12;
pub const MSG_DESTROYED: u8 = 0x13;

const OPEN_TALK_FRAMES: u16 = 13;
const OPEN_ITEM_FRAMES: u16 = 8;
const CLOSE_FRAMES: u16 = 10;
const SPACE_TIMER: i16 = 2;
const FALLBACK_MESSAGE: u32 = 1;

const CTRL_CAMERA: u8 = 0x02;
const CTRL_COLOR: u8 = 0xFF;
const CTRL_INSTANT_ON: u8 = 0x01;
const CTRL_INSTANT_OFF: u8 = 0x02;
const CTRL_WAIT_AUTO_A: u8 = 0x03;
const CTRL_WAIT_AUTO_B: u8 = 0x04;
const CTRL_WAIT_HAND: u8 = 0x05;
const CTRL_WAIT: u8 = 0x07;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MsgEntry {
    pub text: Vec<u8>,
    pub draw_type: u8,
    pub textbox_type: u8,
    pub lines: u16,
}

#[derive(Resource, Debug, Default)]
pub struct MsgTable {
    entries: HashMap<u32, MsgEntry>,
}

impl MsgTable {
    pub fn insert(&mut self, id: u32, entry: MsgEntry) {
        self.entries.insert(id, entry);
    }

    fn get(&self, id: u32) -> Option<&MsgEntry> {
        self.entries.get(&id)
    }
}

#[derive(Resource, Debug)]
pub struct MsgBox {
    active: bool,
    status: u8,
    msg_no: u32,
    data: Vec<u8>,
    draw_type: u8,
    textbox_type: u8,
    lines: u16,
    frame: u16,
    cursor: usize,
    line_count: u16,
    wait_timer: i16,
    instant: bool,
    instant2: bool,
    auto_send: bool,
    hand_send: bool,
    held: bool,
}

impl Default for MsgBox {
    fn default() -> Self {
        Self {
            active: false,
            status: MSG_DESTROYED,
            msg_no: 0,
            data: Vec::new(),
            draw_type: 0,
            textbox_type: 0,
            lines: 0,
            frame: 0,
            cursor: 0,
            line_count: 0,
            wait_timer: 0,
            instant: false,
            instant2: false,
            auto_send: false,
            hand_send: false,
            held: false,
        }
    }
}

impl MsgBox {
    pub fn active(&self) -> bool {
        self.active
    }

    pub fn status(&self) -> u8 {
        self.status
    }

    pub fn message_no(&self) -> u32 {
        self.msg_no
    }

    pub fn set_status(&mut self, status: u8) {
        if self.active {
            self.status = status;
        }
    }

    pub fn message_set(&mut self, msg_no: u32, table: &MsgTable) {
        if self.active {
            self.msg_no = msg_no;
            return;
        }
        let held = self.held;
        *self = Self::default();
        self.held = held;
        self.active = true;
        self.msg_no = msg_no;
        self.load_entry(msg_no, table);
        self.status = MSG_PREPARING;
    }

    fn load_entry(&mut self, msg_no: u32, table: &MsgTable) {
        let entry = table
            .get(msg_no)
            .or_else(|| table.get(FALLBACK_MESSAGE))
            .cloned()
            .unwrap_or_default();
        self.data = entry.text;
        self.draw_type = entry.draw_type;
        self.textbox_type = entry.textbox_type;
        self.lines = entry.lines;
        self.frame = 0;
        self.cursor = 0;
        self.line_count = 0;
        self.wait_timer = 0;
        self.instant = false;
        self.instant2 = false;
        self.auto_send = false;
        self.hand_send = false;
    }

    fn prepare(&mut self) {
        if self.data.first() == Some(&0x1A) {
            let len = usize::from(self.data.get(1).copied().unwrap_or(0));
            let tag = self.data.get(2).copied();
            let color = tag == Some(CTRL_COLOR)
                && self.data.get(3) == Some(&0)
                && self.data.get(4) == Some(&0);
            if len > 0 && (tag == Some(CTRL_CAMERA) || color) {
                self.cursor = len.min(self.data.len());
            }
        }
        self.status = MSG_BOX_OPENING;
    }

    fn open_tick(&mut self) {
        let target = if matches!(self.textbox_type, 1 | 9) {
            OPEN_ITEM_FRAMES
        } else {
            OPEN_TALK_FRAMES
        };
        if self.frame >= target {
            self.status = MSG_TYPING;
        } else {
            self.frame += 1;
        }
    }

    fn typing_tick(&mut self, pressed: bool) {
        if self.draw_type == 0 && pressed {
            self.instant = true;
            if !self.auto_send && !self.hand_send && self.wait_timer != 0 {
                self.wait_timer = 0;
            }
        } else if self.draw_type == 1 {
            self.instant = true;
        }
        if !self.auto_send && !self.hand_send && self.wait_timer != 0 {
            self.wait_timer -= 1;
        } else {
            self.string_set();
        }
        if self.status == MSG_STOP || self.status == MSG_CLOSE_WAIT {
            self.frame = 0;
        }
    }

    fn stop_tick(&mut self, pressed: bool) {
        if self.auto_send {
            if self.wait_timer > 0 {
                self.wait_timer -= 1;
            }
            if self.wait_timer == 0 {
                self.status = MSG_TYPING;
            }
        } else if pressed {
            self.status = MSG_TYPING;
        }
    }

    fn close_wait_tick(&mut self, pressed: bool) {
        if self.wait_timer > 0 {
            self.wait_timer -= 1;
        }
        if self.wait_timer == 0 {
            self.status = MSG_BOX_CLOSING;
            self.frame = 0;
        } else if !self.auto_send && pressed {
            self.status = MSG_BOX_CLOSING;
            self.frame = 0;
        }
    }

    fn continue_tick(&mut self, pressed: bool, table: &MsgTable) {
        if !pressed {
            return;
        }
        self.load_entry(self.msg_no, table);
        self.status = MSG_TYPING;
    }

    fn ends_tick(&mut self, pressed: bool) {
        if pressed {
            self.status = MSG_BOX_CLOSING;
            self.frame = 0;
        }
    }

    fn closing_tick(&mut self) {
        self.frame += 1;
        if self.frame >= CLOSE_FRAMES {
            self.status = MSG_BOX_CLOSED;
        }
    }

    fn typing_gate(&self) -> bool {
        !self.instant && !self.instant2 && !self.auto_send && !self.hand_send
    }

    fn string_set(&mut self) {
        loop {
            let Some(byte) = self.data.get(self.cursor).copied() else {
                break;
            };
            if byte == 0 {
                break;
            }
            if byte == b'\n' {
                self.cursor += 1;
                self.line_count += 1;
                if self.line_count >= self.lines {
                    if self.data.get(self.cursor).is_some_and(|next| *next != 0) {
                        self.status = MSG_STOP;
                    }
                    self.instant = false;
                    self.instant2 = false;
                    return;
                }
                if !self.auto_send {
                    self.wait_timer = SPACE_TIMER;
                }
                if self.typing_gate() {
                    return;
                }
                continue;
            }
            if byte == 0x1A {
                if !self.control_code() {
                    return;
                }
                continue;
            }
            let len = glyph_len(byte);
            let pair = if len == 2 {
                (u16::from(byte) << 8)
                    | u16::from(self.data.get(self.cursor + 1).copied().unwrap_or(0))
            } else {
                u16::from(byte)
            };
            self.cursor = (self.cursor + len).min(self.data.len());
            if !self.auto_send && matches!(pair, 0x8140 | 0x8141 | 0x878C) {
                self.wait_timer = SPACE_TIMER;
            }
            if self.typing_gate() {
                return;
            }
        }
        if self.typing_gate() {
            return;
        }
        if self.auto_send || self.hand_send {
            self.status = MSG_CLOSE_WAIT;
        } else {
            self.status = MSG_DISPLAYED;
        }
    }

    fn control_code(&mut self) -> bool {
        let cursor = self.cursor;
        let Some(len) = self.data.get(cursor + 1).copied() else {
            self.cursor = self.data.len();
            return false;
        };
        if len == 0 {
            self.cursor = self.data.len();
            return false;
        }
        let kind = self.data.get(cursor + 2).copied().unwrap_or(0);
        if kind == 0x01 {
            self.cursor = (cursor + usize::from(len)).min(self.data.len());
            return false;
        }
        if kind == CTRL_CAMERA || kind == CTRL_COLOR {
            self.cursor = (cursor + usize::from(len)).min(self.data.len());
            return true;
        }
        let op = self.data.get(cursor + 4).copied().unwrap_or(0);
        let payload = match (
            self.data.get(cursor + 5).copied(),
            self.data.get(cursor + 6).copied(),
        ) {
            (Some(hi), Some(lo)) => i16::from(u16::from_be_bytes([hi, lo])),
            _ => 0,
        };
        self.cursor = (cursor + usize::from(len)).min(self.data.len());
        match op {
            CTRL_INSTANT_ON => {
                self.instant2 = true;
                true
            }
            CTRL_INSTANT_OFF => {
                self.instant2 = false;
                true
            }
            CTRL_WAIT_AUTO_A | CTRL_WAIT_AUTO_B => {
                self.wait_timer = payload;
                self.auto_send = true;
                true
            }
            CTRL_WAIT_HAND => {
                self.wait_timer = payload;
                self.hand_send = true;
                true
            }
            CTRL_WAIT => {
                self.wait_timer = payload;
                false
            }
            _ => true,
        }
    }
}

pub fn control(do_intent: Res<DoIntent>, table: Res<MsgTable>, mut msg: ResMut<MsgBox>) {
    let pressed = do_intent.0 && !msg.held;
    msg.held = do_intent.0;
    if !msg.active {
        return;
    }
    match msg.status {
        MSG_PREPARING => msg.prepare(),
        MSG_BOX_OPENING => msg.open_tick(),
        MSG_TYPING => msg.typing_tick(pressed),
        MSG_STOP => msg.stop_tick(pressed),
        MSG_CLOSE_WAIT => msg.close_wait_tick(pressed),
        MSG_CONTINUES => msg.continue_tick(pressed, &table),
        MSG_ENDS => msg.ends_tick(pressed),
        MSG_BOX_CLOSING => msg.closing_tick(),
        MSG_DESTROYED => msg.active = false,
        _ => {}
    }
}

fn glyph_len(byte: u8) -> usize {
    match byte >> 4 {
        8 | 9 => 2,
        _ => 1,
    }
}
