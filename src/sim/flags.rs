use bevy_ecs::prelude::*;

use super::actor::Actor;
use super::event::{ActiveEvent, EventList};
use super::player::Player;

pub const TAG_EVENT_ACTOR: &str = "TagEv";
pub const TAG_ISLAND_ACTOR: &str = "TagIsl";

const STORY_BYTES: usize = 0x100;
pub const FLAG_BYTES: usize = 0x100;
const SWITCH_BITS: u32 = 128;
pub const COLLECT_SLOTS: usize = 8;
pub const COLLECT_ITEMS: u8 = 8;

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct StoryBits {
    pub bytes: [u8; STORY_BYTES],
}

impl Default for StoryBits {
    fn default() -> Self {
        Self {
            bytes: [0_u8; STORY_BYTES],
        }
    }
}

impl StoryBits {
    pub(crate) fn cell_of(id: u16) -> (usize, u8) {
        ((id >> 8) as usize, (id & 0xFF) as u8)
    }

    fn cell(&self, id: u16) -> (usize, u8) {
        Self::cell_of(id)
    }

    pub fn test(&self, id: u16) -> bool {
        let (cell, mask) = self.cell(id);
        self.bytes[cell] & mask != 0
    }

    pub fn set(&mut self, id: u16) {
        let (cell, mask) = self.cell(id);
        self.bytes[cell] |= mask;
    }

    pub fn clear(&mut self, id: u16) {
        let (cell, mask) = self.cell(id);
        self.bytes[cell] &= !mask;
    }

    pub fn reg(&self, id: u16) -> u8 {
        let (cell, mask) = self.cell(id);
        self.bytes[cell] & mask
    }

    pub fn set_reg(&mut self, id: u16, value: u8) {
        let (cell, mask) = self.cell(id);
        self.bytes[cell] &= !mask;
        self.bytes[cell] |= value;
    }
}

/// `dSv_event_c`'s temporary instance: `dComIfGs_isTmpBit`, `onTmpBit` and `offTmpBit`
/// are `save.getTmp().isEventBit` and its two writers (`d_com_inf_game.h:1376-1386`),
/// a second flag store of the same shape as the one [`StoryBits`] models. The numeric
/// ids are the same space, which is the trap: `dSv_event_flag_c::UNK_0310` (the tale
/// told, read by `d_a_npc_p1.cpp:1302` and `d_a_obj_pirateship.cpp:431`) and
/// `dSv_event_tmp_flag_c::UNK_0310` (Grandma's offer, `d_a_npc_ba1.cpp:1410`) are two
/// different bits at `0x310`, so the tmp store is its own resource here. Nothing reads
/// the register half (`dComIfGs_setTmpReg`/`getTmpReg`) yet, so only the flag half is
/// modelled.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TmpBits {
    bytes: [u8; STORY_BYTES],
}

impl Default for TmpBits {
    fn default() -> Self {
        Self {
            bytes: [0_u8; STORY_BYTES],
        }
    }
}

impl TmpBits {
    pub fn test(&self, id: u16) -> bool {
        let (cell, mask) = StoryBits::cell_of(id);
        self.bytes[cell] & mask != 0
    }

    pub fn set(&mut self, id: u16) {
        let (cell, mask) = StoryBits::cell_of(id);
        self.bytes[cell] |= mask;
    }

    pub fn clear(&mut self, id: u16) {
        let (cell, mask) = StoryBits::cell_of(id);
        self.bytes[cell] &= !mask;
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Collect {
    bytes: [u8; COLLECT_SLOTS],
    pub tact: u8,
    pub triforce: u8,
    pub symbol: u8,
}

impl Default for Collect {
    fn default() -> Self {
        Self {
            bytes: [0_u8; COLLECT_SLOTS],
            tact: 0,
            triforce: 0,
            symbol: 0,
        }
    }
}

impl Collect {
    pub fn check(&self, idx: u8) -> u8 {
        self.bytes.get(usize::from(idx)).copied().unwrap_or(0)
    }

    pub fn set_byte(&mut self, idx: u8, byte: u8) {
        if let Some(slot) = self.bytes.get_mut(usize::from(idx)) {
            *slot = byte;
        }
    }

    pub fn test(&self, idx: u8, item: u8) -> bool {
        if item >= COLLECT_ITEMS {
            return false;
        }
        self.bytes
            .get(usize::from(idx))
            .is_some_and(|byte| byte & (1 << item) != 0)
    }

    pub fn set(&mut self, idx: u8, item: u8) {
        if item >= COLLECT_ITEMS {
            return;
        }
        if let Some(byte) = self.bytes.get_mut(usize::from(idx)) {
            *byte |= 1 << item;
        }
    }

    pub fn clear(&mut self, idx: u8, item: u8) {
        if item >= COLLECT_ITEMS {
            return;
        }
        if let Some(byte) = self.bytes.get_mut(usize::from(idx)) {
            *byte &= !(1 << item);
        }
    }

    pub fn is_tact(&self, no: u8) -> bool {
        no < 8 && self.tact & (1 << no) != 0
    }

    pub fn on_tact(&mut self, no: u8) {
        if no < 8 {
            self.tact |= 1 << no;
        }
    }

    pub fn is_triforce(&self, no: u8) -> bool {
        no < 8 && self.triforce & (1 << no) != 0
    }

    pub fn on_triforce(&mut self, no: u8) {
        if no < 8 {
            self.triforce |= 1 << no;
        }
    }

    pub fn is_symbol(&self, no: u8) -> bool {
        no < 8 && self.symbol & (1 << no) != 0
    }

    pub fn on_symbol(&mut self, no: u8) {
        if no < 8 {
            self.symbol |= 1 << no;
        }
    }
}

pub const SYMBOL_NAYRU: u8 = 0;
pub const SYMBOL_DIN: u8 = 1;
pub const SYMBOL_FARORE: u8 = 2;

pub const DI_MAP: u8 = 0;
pub const DI_COMPASS: u8 = 1;
pub const DI_BOSS_KEY: u8 = 2;
pub const DI_STAGE_BOSS_ENEMY: u8 = 3;
pub const DI_STAGE_LIFE: u8 = 4;
pub const DI_STAGE_BOSS_DEMO: u8 = 5;
pub const DI_COUNT: u8 = 6;

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemBits {
    pub dungeon_item: u8,
}

impl Default for MemBits {
    fn default() -> Self {
        Self { dungeon_item: 0 }
    }
}

impl MemBits {
    pub fn byte(&self) -> u8 {
        self.dungeon_item
    }

    pub fn from_byte(byte: u8) -> Self {
        Self {
            dungeon_item: byte & 0x3F,
        }
    }

    pub fn is_dungeon_item(&self, no: u8) -> bool {
        no < DI_COUNT && self.dungeon_item & (1 << no) != 0
    }

    pub fn on_dungeon_item(&mut self, no: u8) {
        if no < DI_COUNT {
            self.dungeon_item |= 1 << no;
        }
    }

    pub fn off_dungeon_item(&mut self, no: u8) {
        if no < DI_COUNT {
            self.dungeon_item &= !(1 << no);
        }
    }

    pub fn is_map(&self) -> bool {
        self.is_dungeon_item(DI_MAP)
    }

    pub fn on_map(&mut self) {
        self.on_dungeon_item(DI_MAP);
    }

    pub fn is_compass(&self) -> bool {
        self.is_dungeon_item(DI_COMPASS)
    }

    pub fn on_compass(&mut self) {
        self.on_dungeon_item(DI_COMPASS);
    }

    pub fn is_boss_key(&self) -> bool {
        self.is_dungeon_item(DI_BOSS_KEY)
    }

    pub fn on_boss_key(&mut self) {
        self.on_dungeon_item(DI_BOSS_KEY);
    }

    pub fn is_stage_boss_enemy(&self) -> bool {
        self.is_dungeon_item(DI_STAGE_BOSS_ENEMY)
    }

    pub fn on_stage_boss_enemy(&mut self) {
        self.on_dungeon_item(DI_STAGE_BOSS_ENEMY);
    }

    pub fn off_stage_boss_enemy(&mut self) {
        self.off_dungeon_item(DI_STAGE_BOSS_ENEMY);
    }

    pub fn is_stage_life(&self) -> bool {
        self.is_dungeon_item(DI_STAGE_LIFE)
    }

    pub fn on_stage_life(&mut self) {
        self.on_dungeon_item(DI_STAGE_LIFE);
    }

    pub fn is_stage_boss_demo(&self) -> bool {
        self.is_dungeon_item(DI_STAGE_BOSS_DEMO)
    }

    pub fn on_stage_boss_demo(&mut self) {
        self.on_dungeon_item(DI_STAGE_BOSS_DEMO);
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventFlags {
    pub bytes: [u8; FLAG_BYTES],
}

impl Default for EventFlags {
    fn default() -> Self {
        Self {
            bytes: [0_u8; FLAG_BYTES],
        }
    }
}

impl EventFlags {
    /// `dEvDtFlag_c::flagCheck` (`d_event_data.cpp:56-65`): the reference indexes an array of
    /// 32-bit words with `flag_id / 0x20` and tests `1 << (flag_id & 0x1F)` as a word mask, so a
    /// mask can reach bit 31. The save keeps the flags as 256 bytes, so the word is read big-endian
    /// out of that array: four bytes per word, which holds the reference's layout for every flag
    /// whose word still fits. `None` is the reference's `flagMaxCheck` reject.
    fn slot(&self, flag: u32) -> Option<(usize, u32)> {
        let word = (flag / 32) as usize;
        let at = word.checked_mul(4)?;
        let bytes = self.bytes.get(at..at + 4)?;
        let mask = 1_u32 << (flag & 0x1F);
        Some((at, mask))
    }

    pub fn test(&self, flag: u32) -> bool {
        match self.slot(flag) {
            Some((at, mask)) => self.word(at) & mask != 0,
            None => false,
        }
    }

    pub fn set(&mut self, flag: u32) {
        if let Some((at, mask)) = self.slot(flag) {
            let word = self.word(at) | mask;
            self.write_word(at, word);
        }
    }

    pub fn clear(&mut self, flag: u32) {
        if let Some((at, mask)) = self.slot(flag) {
            let word = self.word(at) & !mask;
            self.write_word(at, word);
        }
    }

    fn word(&self, at: usize) -> u32 {
        u32::from_be_bytes([
            self.bytes[at],
            self.bytes[at + 1],
            self.bytes[at + 2],
            self.bytes[at + 3],
        ])
    }

    fn write_word(&mut self, at: usize, word: u32) {
        self.bytes[at..at + 4].copy_from_slice(&word.to_be_bytes());
    }

    pub fn words(&self) -> &[u8; FLAG_BYTES] {
        &self.bytes
    }

    pub fn from_words(bytes: [u8; FLAG_BYTES]) -> Self {
        Self { bytes }
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct OrderFlags {
    bits: u16,
}

impl OrderFlags {
    pub fn set(&mut self, bits: u16) {
        self.bits |= bits;
    }

    pub fn has(&self, bits: u16) -> bool {
        self.bits & bits == bits
    }

    pub fn clear(&mut self) {
        self.bits = 0;
    }
}

/// `fopAc_Execute` calls `actor->eventInfo.beforeProc()` on every actor before it
/// runs (`f_op_actor.cpp:176`), which is `mCondition = dEvtCnd_NONE_e`
/// (`f_op_actor.h:172`). `chkCondition` is `(mCondition & c) == c`
/// (`f_op_actor.h:170`), so a condition raised by one actor frame is visible only
/// to that frame. One shared word stands in for every actor's own word, so it has
/// to be cleared once per tick before any actor executes.
pub fn clear_order_flags(mut flags: ResMut<OrderFlags>) {
    flags.clear();
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Switches {
    words: [u32; SWITCH_BITS as usize / 32],
}

impl Switches {
    fn slot(&self, no: u8) -> Option<(usize, u32)> {
        if u32::from(no) >= SWITCH_BITS {
            return None;
        }
        Some(((no >> 5) as usize, 1 << (no & 0x1F)))
    }

    pub fn test(&self, no: u8) -> bool {
        match self.slot(no) {
            Some((word, bit)) => self.words[word] & bit != 0,
            None => false,
        }
    }

    pub fn set(&mut self, no: u8) {
        if let Some((word, bit)) = self.slot(no) {
            self.words[word] |= bit;
        }
    }

    pub fn clear(&mut self, no: u8) {
        if let Some((word, bit)) = self.slot(no) {
            self.words[word] &= !bit;
        }
    }

    pub fn words(&self) -> [u32; 4] {
        self.words
    }

    pub fn from_words(words: [u32; 4]) -> Self {
        Self { words }
    }
}

const ISLE_COUNT: usize = 49;
const ARRIVE_BIT: u8 = 1;
const INITIAL_ARRIVALS: [(usize, u8); 3] = [(0, 1 | 2), (10, 1 | 2), (43, 1 | 2)];

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Arrivals {
    bits: [u8; ISLE_COUNT],
}

impl Default for Arrivals {
    fn default() -> Self {
        let mut bits = [0_u8; ISLE_COUNT];
        for (index, value) in INITIAL_ARRIVALS {
            bits[index] = value;
        }
        Self { bits }
    }
}

impl Arrivals {
    pub fn is_arrived(&self, index: usize) -> bool {
        self.bits
            .get(index)
            .is_some_and(|value| value & ARRIVE_BIT != 0)
    }

    pub fn set(&mut self, index: usize) {
        if let Some(value) = self.bits.get_mut(index) {
            *value |= ARRIVE_BIT;
        }
    }

    pub fn bytes(&self) -> [u8; ISLE_COUNT] {
        self.bits
    }

    pub fn from_bytes(bytes: [u8; ISLE_COUNT]) -> Self {
        Self { bits: bytes }
    }
}

pub const REG_PAGES: usize = 256;
pub const REG_GHOST_SHIP: u16 = 0x8803;
pub const REG_GHOST_SHIP_ROOM: u16 = 0xC3FF;
pub const REG_GHOST_SHIP_START: u16 = 0x85FF;
pub const GHOST_SHIP_DONE: u8 = 3;

pub type EventReg = StoryBits;

pub const CHART_SLOTS: usize = 128;
pub const CHART_WORDS: usize = CHART_SLOTS / 32;
pub const CHART_COUNT: usize = 61;
pub const CHART_GREAT_SEA_A: usize = 35;
pub const CHART_GREAT_SEA_B: usize = 36;
pub const CHART_MIN_SECRET: usize = 52;
pub const TRIFORCE_SLOTS: u8 = 8;

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Charts {
    got: [u32; CHART_WORDS],
    open: [u32; CHART_WORDS],
    done: [u32; CHART_WORDS],
    triforce: u8,
}

impl Default for Charts {
    fn default() -> Self {
        Self {
            got: [0; CHART_WORDS],
            open: [0; CHART_WORDS],
            done: [0; CHART_WORDS],
            triforce: 0,
        }
    }
}

impl Charts {
    fn test(words: &[u32; CHART_WORDS], slot: usize) -> bool {
        words
            .get(slot >> 5)
            .is_some_and(|word| word & (1 << (slot & 0x1F)) != 0)
    }

    fn mark(words: &mut [u32; CHART_WORDS], slot: usize, on: bool) {
        if let Some(word) = words.get_mut(slot >> 5) {
            if on {
                *word |= 1 << (slot & 0x1F);
            } else {
                *word &= !(1 << (slot & 0x1F));
            }
        }
    }

    pub fn has(&self, slot: usize) -> bool {
        Self::test(&self.got, slot)
    }

    pub fn is_open(&self, slot: usize) -> bool {
        Self::test(&self.open, slot)
    }

    pub fn is_done(&self, slot: usize) -> bool {
        Self::test(&self.done, slot)
    }

    pub fn give(&mut self, slot: usize) {
        Self::mark(&mut self.got, slot, true);
        Self::mark(&mut self.open, slot, true);
    }

    pub fn open(&mut self, slot: usize) {
        Self::mark(&mut self.open, slot, true);
    }

    pub fn close(&mut self, slot: usize) {
        Self::mark(&mut self.open, slot, false);
    }

    pub fn complete(&mut self, slot: usize) {
        Self::mark(&mut self.done, slot, true);
    }

    pub fn triforce(&self, slot: u8) -> bool {
        slot < TRIFORCE_SLOTS && self.triforce & (1 << slot) != 0
    }

    pub fn set_triforce(&mut self, slot: u8) {
        if slot < TRIFORCE_SLOTS {
            self.triforce |= 1 << slot;
        }
    }

    pub fn total(&self) -> usize {
        (1..=CHART_COUNT)
            .filter(|slot| {
                *slot != CHART_GREAT_SEA_A
                    && *slot != CHART_GREAT_SEA_B
                    && *slot < CHART_MIN_SECRET
                    && self.has(*slot - 1)
            })
            .count()
    }
}

const NO_SET_ID: u16 = 0xFFFF;
const SET_ID_WORDS: usize = 16;
const SET_ID_MAX: usize = SET_ID_WORDS * 32;
const ROOM_COUNT: usize = 64;

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZoneActors {
    bits: [[u32; SET_ID_WORDS]; ROOM_COUNT],
}

impl Default for ZoneActors {
    fn default() -> Self {
        Self {
            bits: [[0_u32; SET_ID_WORDS]; ROOM_COUNT],
        }
    }
}

impl ZoneActors {
    pub fn is_done(&self, room: i8, set_id: u16) -> bool {
        let Some((word, mask)) = self.slot(room, set_id) else {
            return false;
        };
        self.bits[usize::from(room as u8)][word] & mask != 0
    }

    pub fn mark_done(&mut self, room: i8, set_id: u16) {
        let Some((word, mask)) = self.slot(room, set_id) else {
            return;
        };
        self.bits[usize::from(room as u8)][word] |= mask;
    }

    pub fn clear(&mut self) {
        self.bits = [[0_u32; SET_ID_WORDS]; ROOM_COUNT];
    }

    fn slot(&self, room: i8, set_id: u16) -> Option<(usize, u32)> {
        if room < 0 || set_id == NO_SET_ID {
            return None;
        }
        let id = usize::from(set_id);
        if id >= SET_ID_MAX {
            return None;
        }
        Some((id / 32, 1 << (id % 32)))
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwitchOn(pub u8);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LetterMsg;

const LETTER_STOCK: u8 = 2;
const TPOST_LETTERS: [(u32, u16); 13] = [
    (0x1AAF, 0xAC03),
    (0x0CF9, 0xB503),
    (0x0CFA, 0x7D03),
    (0x0CFC, 0x7B03),
    (0x0805, 0x9D03),
    (0x0CFD, 0x7A03),
    (0x0DB6, 0xB203),
    (0x1148, 0x8B03),
    (0x1AAF, 0xAC03),
    (0x0F76, 0xB003),
    (0x19A6, 0xAE03),
    (0x0CFB, 0x7C03),
    (0x0F77, 0xAF03),
];

fn letter_msg(regs: &EventReg) -> u32 {
    TPOST_LETTERS
        .iter()
        .skip(1)
        .find(|(_, reg)| regs.reg(*reg) == LETTER_STOCK)
        .map_or(0, |(msg, _)| *msg)
}

pub fn control(
    stories: Res<StoryBits>,
    mut switches: ResMut<Switches>,
    mut active: ResMut<ActiveEvent>,
    list: Res<EventList>,
    players: Query<&Player>,
    mut tags: Query<(Entity, &Actor, &mut TagEvent)>,
    forced: Query<(Entity, &SwitchOn)>,
    mut posts: Query<(Entity, &LetterMsg, &mut super::talk::Talk)>,
    mut commands: Commands,
) {
    for (entity, on) in &forced {
        switches.set(on.0);
        commands.entity(entity).try_remove::<SwitchOn>();
    }
    for (entity, _, mut talk) in &mut posts {
        talk.msg_curr = letter_msg(&stories);
        talk.msg_end = -1;
        commands.entity(entity).try_remove::<LetterMsg>();
    }
    if let Some(player) = players.iter().next() {
        tag_triggers(
            &mut active,
            &list,
            &stories,
            &mut switches,
            player,
            &mut tags,
        );
    }
}

pub const NO_SWITCH: u8 = 0xFF;
const NO_EVENT_FLAG: u16 = 0xFFFF;
const RADIUS_SCALE: f32 = 100.0;
const ISLAND_SCALE: f32 = 10000.0;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagState {
    Hunt,
    Event,
    Wait,
}

#[derive(Component, Debug, Clone)]
pub struct TagEvent {
    pub index: usize,
    pub swbit: u8,
    pub swbit2: u8,
    pub kind: u8,
    pub event_flag: u16,
    pub arrival: u16,
    pub radius: f32,
    pub height: f32,
    pub state: TagState,
}

pub fn is_tag_event(name: &str) -> bool {
    name == TAG_EVENT_ACTOR || name == TAG_ISLAND_ACTOR
}

pub fn tag_event(
    parameters: u32,
    event_flag: u16,
    scale: [f32; 3],
    island: bool,
    index: usize,
) -> TagEvent {
    let kind = (parameters & 0xFF) as u8;
    let swbit2 = if island {
        NO_SWITCH
    } else {
        ((parameters >> 16) & 0xFF) as u8
    };
    let reach = if island { ISLAND_SCALE } else { RADIUS_SCALE };
    TagEvent {
        index,
        swbit: ((parameters >> 8) & 0xFF) as u8,
        swbit2,
        kind,
        event_flag,
        arrival: if island { arrival_flag(kind) } else { 0 },
        radius: scale[0] * reach,
        height: scale[1] * reach,
        state: TagState::Hunt,
    }
}

fn arrival_flag(kind: u8) -> u16 {
    match kind {
        1 => 0x0902,
        2 => 0x0A20,
        3 => 0x0A02,
        4 => 0x1F04,
        5 => 0x2E04,
        6 => 0x2E02,
        7 => 0x3E10,
        _ => 0,
    }
}

fn tag_triggers(
    active: &mut ActiveEvent,
    list: &EventList,
    stories: &StoryBits,
    switches: &mut Switches,
    player: &Player,
    tags: &mut Query<(Entity, &Actor, &mut TagEvent)>,
) {
    for (entity, actor, mut tag) in tags.iter_mut() {
        if tag.state == TagState::Event {
            if active.end_check(tag.index) {
                tag.state = TagState::Wait;
            }
            continue;
        }
        if tag.state == TagState::Wait {
            continue;
        }
        if tag.arrival != 0 && stories.test(tag.arrival) {
            tag.state = TagState::Wait;
            continue;
        }
        let terms = if tag.arrival != 0 {
            tag.swbit == NO_SWITCH || !switches.test(tag.swbit)
        } else if tag.kind == 0x0C {
            tag.swbit2 == NO_SWITCH || !switches.test(tag.swbit2)
        } else {
            (tag.swbit2 == NO_SWITCH || switches.test(tag.swbit2))
                && (tag.event_flag == NO_EVENT_FLAG
                    || tag.event_flag == 0
                    || stories.test(tag.event_flag))
        };
        if !terms {
            continue;
        }
        if tag.swbit != NO_SWITCH && switches.test(tag.swbit) {
            tag.state = TagState::Wait;
            continue;
        }
        if tag.state == TagState::Hunt {
            let dx = player.pos.x - actor.pos.x;
            let dz = player.pos.z - actor.pos.z;
            let dy = (player.pos.y - actor.pos.y).abs();
            if dx * dx + dz * dz >= tag.radius * tag.radius || dy > tag.height {
                continue;
            }
        }
        if active.order(list, tag.index, entity) {
            if tag.swbit != NO_SWITCH {
                switches.set(tag.swbit);
            }
            tag.state = TagState::Event;
        }
    }
}
