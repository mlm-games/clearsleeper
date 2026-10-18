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
/// `fopMsgM_messageTypeSelect`'s fallback when the header or the message is missing:
/// `*i_msgNo = 1` (`f_op_msg_mng.cpp:384-393`), a bare code, so group 0's message 1.
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
    /// `JMSMesgEntry_c::mMsgNo` (`f_op_msg_mng.h:24`): the number the talk path's scan matches
    /// (`f_op_msg_mng.cpp:1482-1500`), *not* the entry's position in the list.
    pub msg_no: u32,
    /// `JMSMesgEntry_c::mDataOffs` (`f_op_msg_mng.h:23`). Zero is an entry with no text: the
    /// talk path's scan skips those rows (`f_op_msg_mng.cpp:1487-1489`), while the position read
    /// still returns them.
    pub data_offs: u32,
    pub text: Vec<u8>,
    pub draw_type: u8,
    pub textbox_type: u8,
    pub lines: u16,
}

/// `JMessage::TResourceContainer` (`resource.h:52-77`) narrowed to what a lookup needs: one entry
/// list per group, where a `bmg`'s group is the `INF1` block's `JUTMesgInfo::groupID`
/// (`data.h:64-73`) that `dMesg_parse` feeds the container one archive at a time
/// (`d_mesg.cpp:2186-2202`).
#[derive(Resource, Debug, Default)]
pub struct MsgTable {
    groups: HashMap<u16, MsgGroup>,
}

#[derive(Debug, Default)]
struct MsgGroup {
    /// `JUTMesgInfo::messageEntryTable` (`data.h:74`): the entries in block order, which is the
    /// index `TResource::getMessageEntry(messageIndex)` reads (`resource.h:34-44`).
    entries: Vec<MsgEntry>,
}

impl MsgTable {
    /// The container's `Do_create`/`setData_*` pair (`resource.cpp`) over one `bmg`: the INF1
    /// entry list in block order under the block's own group id.
    pub fn insert(&mut self, group: u16, entry: MsgEntry) {
        self.groups.entry(group).or_default().entries.push(entry);
    }

    /// `TResourceContainer::Get_groupID(groupID)` (`resource.cpp`) answering with a resource.
    pub fn has_group(&self, group: u16) -> bool {
        self.groups.contains_key(&group)
    }

    /// `TResource::getMessageEntry(messageIndex)` - `resource.h:34-44`: the entry at the index,
    /// which is the read `JMessage::TControl::getMessageEntry` and the demo's `dMesg_waitProc`
    /// make.
    pub fn entry_at(&self, group: u16, index: u32) -> Option<&MsgEntry> {
        self.groups
            .get(&group)?
            .entries
            .get(usize::try_from(index).ok()?)
    }

    /// `fopMsgM_msgGet_c::getMessage`'s scan - `f_op_msg_mng.cpp:1482-1500`: the first entry whose
    /// `mMsgNo` matches and whose `mDataOffs` is set, the talk path's lookup.
    pub fn entry_for_no(&self, group: u16, msg_no: u32) -> Option<&MsgEntry> {
        self.groups
            .get(&group)?
            .entries
            .iter()
            .find(|entry| entry.data_offs != 0 && entry.msg_no == msg_no)
    }
}

#[derive(Resource, Debug)]
pub struct MsgBox {
    active: bool,
    status: u8,
    msg_no: u32,
    /// `JMessage::TControl::getMessageCode()` - `(getMessageGroupID() << 16) | getMessageIndex()`
    /// (`JMessage/control.h:55`). The demo's `JMSG` command is the only writer: retail's
    /// `JStudio_JMessage::TAdaptor_message::adaptor_do_MESSAGE` calls `setMessageCode(packed)`
    /// (`JStudio_JMessage/object-message.cpp:22-30`), whose two halves are stored on the control
    /// before `setMessageCode_flush_` loads the entry (`JMessage/control.cpp:100-111`).
    message_code: u32,
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
            message_code: 0,
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
        // `fopMsgM_msgGet_c::getMesgHeader` - `f_op_msg_mng.cpp:1441-1462`: the group is the
        // code's high half, and the entry is the one whose `mMsgNo` matches the low half
        // (`:1482-1500`). A miss is `fopMsgM_messageTypeSelect`'s `*i_msgNo = 1` (`:384-393`).
        let group = (msg_no >> 16) as u16;
        let index = msg_no & 0xFFFF;
        let entry = table
            .entry_for_no(group, index)
            .or_else(|| table.entry_for_no(group, FALLBACK_MESSAGE));
        self.open(msg_no, entry);
    }

    /// `JMessage::TControl::setMessageCode(packed)` - `JMessage/control.h:18-24`: the group is
    /// the code's high half and the message index its low half, and the flush loads the entry for
    /// the pair (`JMessage/control.cpp:100-111`). The entry itself is
    /// `TResource::getMessageEntry(messageIndex)` (`resource.h:34-44`), the entry **at** that
    /// index: this path indexes the group's list, it does not scan it, and a miss falls back to
    /// nothing - `setMessageCode_flush_` has no fallback, unlike the talk path's
    /// `fopMsgM_messageTypeSelect`. `dMesg_waitProc` is what turns the changed code into a box
    /// rebuild: it compares the code it last opened with against `getMessageCode()` and, on a
    /// difference, builds the screen and re-reads the control's text (`d_mesg.cpp:2361-2445`),
    /// which is the reload below.
    pub fn message_set_code(&mut self, code: u32, table: &MsgTable) {
        let changed = !self.active || self.message_code != code;
        self.message_code = code;
        if !changed {
            return;
        }
        let held = self.held;
        *self = Self::default();
        self.held = held;
        self.active = true;
        self.message_code = code;
        let group = (code >> 16) as u16;
        let index = code & 0xFFFF;
        self.msg_no = index;
        self.load_entry(table.entry_at(group, index));
        self.status = MSG_PREPARING;
    }

    /// The packed code the last [`MsgBox::message_set_code`] carried.
    pub fn message_code(&self) -> u32 {
        self.message_code
    }

    /// The box's shared open: the `held` flag survives the reset, the entry's fields become the
    /// box's and the counter state starts clean. `message_set` and `message_set_code` are the two
    /// callers, one with the entry the scan found and one with the entry at the code's index.
    fn open(&mut self, msg_no: u32, entry: Option<&MsgEntry>) {
        let held = self.held;
        *self = Self::default();
        self.held = held;
        self.active = true;
        self.msg_no = msg_no;
        self.load_entry(entry);
        self.status = MSG_PREPARING;
    }

    fn load_entry(&mut self, entry: Option<&MsgEntry>) {
        let entry = entry.cloned().unwrap_or_default();
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

    /// The continue step's reload: the entry the box already holds stays, its counter state
    /// starts over. `dMesg` re-reads the control's message data at the same code
    /// (`d_mesg.cpp:2361-2445`), which is the same entry the code resolved to.
    fn continue_tick(&mut self, pressed: bool) {
        if !pressed {
            return;
        }
        self.frame = 0;
        self.cursor = 0;
        self.line_count = 0;
        self.wait_timer = 0;
        self.instant = false;
        self.instant2 = false;
        self.auto_send = false;
        self.hand_send = false;
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
            (Some(hi), Some(lo)) => i16::from_be_bytes([hi, lo]),
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

pub fn control(do_intent: Res<DoIntent>, mut msg: ResMut<MsgBox>) {
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
        MSG_CONTINUES => msg.continue_tick(pressed),
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
