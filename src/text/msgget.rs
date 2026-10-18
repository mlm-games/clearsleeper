use super::{MesgEntry, be16, be32};

pub const ZEL_ENGLISH: &str = "zel_00.bmg";
pub const ZEL_HYRULE: &str = "zel_01.bmg";

const FIRST_BLOCK: usize = 0x20;
const BLOCK_TYPE_AT: usize = 0x04;
const NUM_ENTRY_AT: usize = 0x08;
const ENTRY_TABLE_AT: usize = 0x10;
const DATA_BLOB_AT: usize = 0x08;
const SEARCH_LIMIT: u32 = 0xFFFF;

const HYRULE_MSGS: [u32; 15] = [
    0xD49, 0xD4B, 0xD4D, 0xD4F, 0xD51, 0xD53, 0xD54, 0xD55, 0xD57, 0xD59, 0x1178, 0x117A, 0x1389,
    0x138A, 0x196E,
];

pub trait MsgArchive {
    fn resource(&self, name: &str) -> Option<&[u8]>;
}

pub fn hyrule_language_check(clear_count: u8, msg_no: u32) -> bool {
    if clear_count != 0 {
        return false;
    }
    HYRULE_MSGS.contains(&msg_no)
}

pub fn search_message_number<F: MsgArchive + ?Sized>(
    archive: &F,
    clear_count: u8,
    msg_no: u32,
) -> u32 {
    let mut get = MsgGet::default();
    let mut i = msg_no & 0xFFFF;
    while i < SEARCH_LIMIT {
        if let Some(head) = get.get_mesg_header(archive, clear_count, i) {
            if get.get_message(head).is_some() {
                return i;
            }
        }
        i += 1;
    }
    msg_no
}

fn info_size(head: &[u8]) -> Option<usize> {
    Some(be32(head, FIRST_BLOCK + BLOCK_TYPE_AT)? as usize)
}

fn num_entry(head: &[u8]) -> Option<usize> {
    Some(usize::from(be16(head, FIRST_BLOCK + NUM_ENTRY_AT)?))
}

fn entry_at(head: &[u8], index: u32) -> Option<MesgEntry> {
    MesgEntry::parse(
        head,
        FIRST_BLOCK + ENTRY_TABLE_AT + index as usize * MesgEntry::LEN,
    )
}

fn data_block(head: &[u8]) -> Option<usize> {
    Some(FIRST_BLOCK.checked_add(info_size(head)?)?)
}

fn text_blob(head: &[u8]) -> Option<&[u8]> {
    let block = data_block(head)?;
    let size = be32(head, block + BLOCK_TYPE_AT)? as usize;
    head.get(block + DATA_BLOB_AT..block.checked_add(size)?)
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MsgGet {
    pub m_msg_idx: u32,
    pub m_group_id: u16,
    pub m_msg_no: u16,
    pub m_res_msg_no: u16,
}

impl MsgGet {
    pub fn get_mesg_number(&self) -> u32 {
        (self.m_group_id as u32) << 8 | self.m_res_msg_no as u32
    }

    pub fn get_mesg_header<'a, F: MsgArchive + ?Sized>(
        &mut self,
        archive: &'a F,
        clear_count: u8,
        msg_no: u32,
    ) -> Option<&'a [u8]> {
        self.m_group_id = (msg_no >> 16) as u16;
        self.m_msg_no = msg_no as u16;
        let name = if hyrule_language_check(clear_count, msg_no) {
            ZEL_HYRULE
        } else {
            ZEL_ENGLISH
        };
        archive.resource(name)
    }

    pub fn get_mesg_info(head: &[u8]) -> Option<&[u8]> {
        head.get(FIRST_BLOCK..)
    }

    pub fn get_mesg_data(head: &[u8]) -> Option<&[u8]> {
        head.get(data_block(head)?..)
    }

    pub fn get_mesg_entry(&self, head: &[u8]) -> Option<MesgEntry> {
        entry_at(head, self.m_msg_idx)
    }

    pub fn get_message<'a>(&mut self, head: &'a [u8]) -> Option<&'a [u8]> {
        let text = text_blob(head)?;
        let count = num_entry(head)?;
        for i in 0..count {
            let entry = entry_at(head, i as u32)?;
            if entry.data_offs == 0 {
                continue;
            }
            self.m_msg_idx = i as u32;
            if self.m_msg_no == entry.msg_no {
                let message = text.get(entry.data_offs as usize..)?;
                self.m_res_msg_no = entry.msg_no;
                return Some(message);
            }
        }
        None
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ItemMsgGet {
    pub m_msg_idx: u32,
    pub m_msg_no: u16,
    pub m_res_msg_no: u16,
}

impl ItemMsgGet {
    pub fn get_mesg_header<'a, F: MsgArchive + ?Sized>(
        &mut self,
        archive: &'a F,
        clear_count: u8,
        msg_no: u32,
    ) -> Option<&'a [u8]> {
        self.m_msg_no = msg_no as u16;
        let name = if hyrule_language_check(clear_count, msg_no) {
            ZEL_HYRULE
        } else {
            ZEL_ENGLISH
        };
        archive.resource(name)
    }

    pub fn get_mesg_info(head: &[u8]) -> Option<&[u8]> {
        head.get(FIRST_BLOCK..)
    }

    pub fn get_mesg_data(head: &[u8]) -> Option<&[u8]> {
        head.get(data_block(head)?..)
    }

    pub fn get_mesg_entry(&self, head: &[u8]) -> Option<MesgEntry> {
        entry_at(head, self.m_msg_idx)
    }

    pub fn get_message<'a>(&mut self, head: &'a [u8]) -> Option<&'a [u8]> {
        let text = text_blob(head)?;
        let count = num_entry(head)?;
        for i in 0..count {
            let entry = entry_at(head, i as u32)?;
            if entry.data_offs == 0 {
                continue;
            }
            self.m_msg_idx = i as u32;
            if self.m_msg_no == entry.msg_no {
                break;
            }
        }
        let entry = entry_at(head, self.m_msg_idx)?;
        self.m_res_msg_no = entry.msg_no;
        text.get(entry.data_offs as usize..)
    }
}
