use bevy_ecs::prelude::*;

use super::events::{SimEvent, SimEvents};
use super::frame::CurrentFrame;
use crate::text::driver::Counters;
use crate::text::msgproc::MsgGameInfo;

pub const MODE_NONE: i32 = -1;
pub const MODE_SHIP_RACE: i32 = 2;
pub const MODE_VOLCANO: i32 = 3;
pub const MODE_AUCTION: i32 = 4;
pub const MODE_SHOP_GAME: i32 = 6;
pub const MODE_STOCK: i32 = 7;

pub const STATUS_IDLE: u8 = 0;
pub const STATUS_PENDING: u8 = 1;
pub const STATUS_RUNNING: u8 = 2;
pub const STATUS_PENDING_STOCK: u8 = 3;
pub const STATUS_ENDING: u8 = 4;
pub const STATUS_FINISHED: u8 = 5;
pub const STATUS_DELETE: u8 = 6;

pub const SHOW_CLOCK: u8 = 1;
pub const SHOW_RUPEE: u8 = 2;

pub const STOP_SPEAK: u8 = 1;

pub const END_HOLD_DEFAULT: i32 = 120;
pub const END_HOLD_VOLCANO: i32 = 30;
pub const ANIME_FRAME: i32 = 22;
pub const ANIME_HOLD: i32 = 7;
pub const DRAW_DELAY: i32 = 5;

pub const EVENT_MODE_TALK: u8 = 1;

pub const PARAM_SIZE: usize = 0x34;
pub const PARAM_MODE_AT: usize = 0x1C;
pub const PARAM_LIMIT_AT: usize = 0x20;
pub const PARAM_SHOW_AT: usize = 0x22;
pub const PARAM_ICON_AT: usize = 0x23;
pub const PARAM_TIMER_POS_AT: usize = 0x24;
pub const PARAM_RUPEE_POS_AT: usize = 0x2C;

pub const MS_PER_FRAME: i64 = 1000 / 60;

pub const SE_SHIP_RACE_30: u32 = 0x0845;
pub const SE_SHIP_RACE_10: u32 = 0x0846;
pub const SE_MINIGAME_30: u32 = 0x08AD;
pub const SE_MINIGAME_10: u32 = 0x08AE;
pub const SE_MINIGAME_EACH: u32 = 0x08EA;

pub const TABLE_END: i32 = -1;

pub const SHIP_RACE_SE: [(i32, u32); 13] = [
    (30000, SE_SHIP_RACE_30),
    (20000, SE_SHIP_RACE_30),
    (10000, SE_SHIP_RACE_10),
    (9000, SE_SHIP_RACE_10),
    (8000, SE_SHIP_RACE_10),
    (7000, SE_SHIP_RACE_10),
    (6000, SE_SHIP_RACE_10),
    (5000, SE_SHIP_RACE_10),
    (4000, SE_SHIP_RACE_10),
    (3000, SE_SHIP_RACE_10),
    (2000, SE_SHIP_RACE_10),
    (1000, SE_SHIP_RACE_10),
    (TABLE_END, 0x0000),
];

pub const VOLCANO_SE: [(i32, u32); 13] = [
    (60000, SE_MINIGAME_30),
    (30000, SE_MINIGAME_30),
    (10000, SE_MINIGAME_10),
    (9000, SE_MINIGAME_10),
    (8000, SE_MINIGAME_10),
    (7000, SE_MINIGAME_10),
    (6000, SE_MINIGAME_10),
    (5000, SE_MINIGAME_10),
    (4000, SE_MINIGAME_10),
    (3000, SE_MINIGAME_10),
    (2000, SE_MINIGAME_10),
    (1000, SE_MINIGAME_10),
    (TABLE_END, 0x0000),
];

fn digit(value: i32) -> i32 {
    if value < 0 || value >= 10 { 0 } else { value }
}

fn rest_time_ms(limit: i64, time: i64, start: i64, pause: i64) -> i64 {
    limit - (time - start - pause)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimerParams {
    pub mode: i32,
    pub limit_time: u16,
    pub show_type: u8,
    pub icon_type: u8,
    pub timer_pos: [f32; 2],
    pub rupee_pos: [f32; 2],
}

impl TimerParams {
    pub fn new(
        mode: i32,
        limit_time: u16,
        show_type: u8,
        icon_type: u8,
        timer_x: f32,
        timer_y: f32,
        rupee_x: f32,
        rupee_y: f32,
    ) -> Self {
        Self {
            mode,
            limit_time,
            show_type,
            icon_type,
            timer_pos: [timer_x, timer_y],
            rupee_pos: [rupee_x, rupee_y],
        }
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        if data.len() < PARAM_SIZE {
            return None;
        }
        let mode = i32::from_le_bytes(data[PARAM_MODE_AT..PARAM_MODE_AT + 4].try_into().ok()?);
        let limit_time =
            u16::from_le_bytes(data[PARAM_LIMIT_AT..PARAM_LIMIT_AT + 2].try_into().ok()?);
        let timer_pos = [
            f32::from_le_bytes(
                data[PARAM_TIMER_POS_AT..PARAM_TIMER_POS_AT + 4]
                    .try_into()
                    .ok()?,
            ),
            f32::from_le_bytes(
                data[PARAM_TIMER_POS_AT + 4..PARAM_TIMER_POS_AT + 8]
                    .try_into()
                    .ok()?,
            ),
        ];
        let rupee_pos = [
            f32::from_le_bytes(
                data[PARAM_RUPEE_POS_AT..PARAM_RUPEE_POS_AT + 4]
                    .try_into()
                    .ok()?,
            ),
            f32::from_le_bytes(
                data[PARAM_RUPEE_POS_AT + 4..PARAM_RUPEE_POS_AT + 8]
                    .try_into()
                    .ok()?,
            ),
        ];
        Some(Self {
            mode,
            limit_time,
            show_type: data[PARAM_SHOW_AT],
            icon_type: data[PARAM_ICON_AT],
            timer_pos,
            rupee_pos,
        })
    }
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct IsleTimer {
    pub mode: i32,
    pub now_time_ms: i32,
    pub limit_time_ms: i32,
    pub exists: bool,
    pub menu_pause: bool,
    pub timer_mode: i32,
    pub show_type: u8,
    pub icon_type: u8,
    pub timer_pos: [f32; 2],
    pub rupee_pos: [f32; 2],
    pub limit_time: i64,
    pub start_time: i64,
    pub time: i64,
    pub end_time: i64,
    pub stop_time: i64,
    pub pause_time: i64,
    pub prev_rest_ms: i32,
    pub end_hold: i32,
    pub draw_delay: i32,
    pub start_delay: i16,
    pub stopped: u8,
    pub stop_tag: u8,
    pub status: u8,
    pub se_time_left: i32,
    pub se_table_index: u32,
    pub anime_timer: i32,
    pub anime_done: u8,
    pub rupee_num: i32,
    pub digits: [i32; 6],
    pub rupee_digits: [i32; 3],
    pub clock_ms: i64,
}

impl Default for IsleTimer {
    fn default() -> Self {
        Self {
            mode: MODE_NONE,
            now_time_ms: 0,
            limit_time_ms: 0,
            exists: false,
            menu_pause: false,
            timer_mode: MODE_NONE,
            show_type: 0,
            icon_type: 0,
            timer_pos: [0.0; 2],
            rupee_pos: [0.0; 2],
            limit_time: 0,
            start_time: 0,
            time: 0,
            end_time: 0,
            stop_time: 0,
            pause_time: 0,
            prev_rest_ms: 0,
            end_hold: 0,
            draw_delay: 0,
            start_delay: 0,
            stopped: 0,
            stop_tag: 0,
            status: STATUS_IDLE,
            se_time_left: 0,
            se_table_index: 0,
            anime_timer: 0,
            anime_done: 0,
            rupee_num: -1,
            digits: [0; 6],
            rupee_digits: [0; 3],
            clock_ms: 0,
        }
    }
}

impl IsleTimer {
    pub fn set_clock(&mut self, now_ms: i64) {
        self.clock_ms = now_ms;
    }

    pub fn exists(&self) -> bool {
        self.exists
    }

    pub fn mode(&self) -> i32 {
        self.mode
    }

    pub fn set_mode(&mut self, mode: i32) {
        self.mode = mode;
    }

    pub fn now_time_ms(&self) -> i32 {
        self.now_time_ms
    }

    pub fn limit_time_ms(&self) -> i32 {
        self.limit_time_ms
    }

    pub fn rest_time(&self) -> i32 {
        self.limit_time_ms - self.now_time_ms
    }

    pub fn status(&self) -> u8 {
        self.status
    }

    pub fn get_time_ms(&self) -> i32 {
        (self.time - self.start_time - self.pause_time) as i32
    }

    pub fn get_limit_time_ms(&self) -> i32 {
        self.limit_time as i32
    }

    pub fn get_rest_time_ms(&self) -> i32 {
        rest_time_ms(self.limit_time, self.time, self.start_time, self.pause_time) as i32
    }

    pub fn rest_time_check(&self, time: i32) -> bool {
        self.get_rest_time_ms() <= time && time > self.prev_rest_ms
    }

    pub fn timer_digits(&self) -> [i32; 6] {
        let total = self.get_rest_time_ms();
        let sec = total / 1000;
        let ms = total - sec * 1000;
        let min = sec / 60;
        let sec = sec - min * 60;
        [
            digit(min / 10),
            digit(min % 10),
            digit(sec / 10),
            digit(sec % 10),
            digit(ms / 100),
            digit((ms % 100) / 10),
        ]
    }

    pub fn set_rupee(&mut self, num: i16) -> Option<[i32; 3]> {
        let num = i32::from(num);
        if num == self.rupee_num {
            return None;
        }
        if num < self.rupee_num {
            self.rupee_num -= 1;
        } else {
            self.rupee_num += 1;
        }
        if self.rupee_num < 0 {
            self.rupee_num = 0;
        }
        let ones = self.rupee_num % 10;
        let tens = (self.rupee_num % 100) / 10;
        let hundreds = self.rupee_num / 100;
        Some([digit(hundreds), digit(tens), digit(ones)])
    }

    pub fn take_rupee_digits(&mut self, num: i16) -> [i32; 3] {
        if let Some(digits) = self.set_rupee(num) {
            self.rupee_digits = digits;
        }
        self.rupee_digits
    }

    pub fn create(&mut self, params: &TimerParams) {
        let fresh = Self {
            mode: self.mode,
            now_time_ms: self.now_time_ms,
            limit_time_ms: self.limit_time_ms,
            rupee_num: -1,
            ..Self::default()
        };
        *self = fresh;
        self.rupee_pos = params.rupee_pos;
        self.timer_pos = params.timer_pos;
        self.show_type = params.show_type;
        self.icon_type = params.icon_type;
        self.timer_mode = params.mode;
        if self.timer_mode == MODE_STOCK {
            self.limit_time = i64::from(self.limit_time_ms);
            let now = self.clock_ms;
            self.time = now;
            self.start_time = now;
            self.start_time -= i64::from(self.now_time_ms);
            self.status = STATUS_IDLE;
            self.timer_mode = self.mode;
            self.now_time_ms = self.get_time_ms();
            self.limit_time_ms = self.get_limit_time_ms();
            self.mode = self.timer_mode;
            self.exists = true;
            if self.mode == MODE_VOLCANO {
                self.show_type = SHOW_CLOCK;
            }
            self.stock_start_delayed(10);
        } else {
            self.limit_time = i64::from(params.limit_time) * 1000;
            self.start_time = 0;
            self.time = 0;
            self.status = STATUS_IDLE;
            self.now_time_ms = self.get_time_ms();
            self.limit_time_ms = self.get_limit_time_ms();
            self.mode = self.timer_mode;
            self.exists = true;
        }
        self.end_time = 0;
        self.stop_time = 0;
        self.pause_time = 0;
        self.stopped = 0;
        self.stop_tag = 0;
        self.end_hold = END_HOLD_DEFAULT;
        self.draw_delay = 0;
        self.se_table_index = 0;
        self.se_time_left = 0;
        let table = match self.timer_mode {
            MODE_SHIP_RACE => Some(&SHIP_RACE_SE[..]),
            MODE_VOLCANO => Some(&VOLCANO_SE[..]),
            _ => None,
        };
        if let Some(table) = table {
            while self.se_time_left >= 0 {
                self.se_time_left = table[self.se_table_index as usize].0;
                if self.get_rest_time_ms() > self.se_time_left {
                    break;
                }
                self.se_table_index += 1;
            }
        }
    }

    pub fn create_timer(
        &mut self,
        mode: i32,
        limit_time: u16,
        show_type: u8,
        icon_type: u8,
        timer_x: f32,
        timer_y: f32,
        rupee_x: f32,
        rupee_y: f32,
    ) -> bool {
        if self.mode != MODE_NONE {
            return false;
        }
        self.create(&TimerParams::new(
            mode, limit_time, show_type, icon_type, timer_x, timer_y, rupee_x, rupee_y,
        ));
        true
    }

    pub fn create_stock_timer(&mut self) -> bool {
        if self.mode == MODE_NONE {
            return false;
        }
        self.create(&TimerParams::new(
            MODE_STOCK, 0, 3, 0, 221.0, 439.0, 32.0, 419.0,
        ));
        true
    }

    pub fn start(&mut self) -> bool {
        if self.status == STATUS_IDLE || self.status == STATUS_PENDING {
            self.status = STATUS_RUNNING;
            self.time = self.clock_ms;
            self.start_time = self.clock_ms;
            true
        } else {
            false
        }
    }

    pub fn start_delayed(&mut self, delay: i16) -> bool {
        if self.status == STATUS_IDLE {
            self.start_delay = delay;
            self.status = STATUS_PENDING;
            true
        } else {
            false
        }
    }

    pub fn stock_start(&mut self) -> bool {
        if self.status == STATUS_PENDING_STOCK {
            self.status = STATUS_RUNNING;
            let now = self.clock_ms;
            self.start_time = now;
            self.time = now;
            self.start_time -= i64::from(self.now_time_ms);
            true
        } else {
            false
        }
    }

    pub fn stock_start_delayed(&mut self, delay: i16) -> bool {
        if self.status == STATUS_IDLE {
            self.start_delay = delay;
            self.status = STATUS_PENDING_STOCK;
            true
        } else {
            false
        }
    }

    pub fn stop(&mut self, tag: u8) -> bool {
        if self.stopped == 1 || self.stop_tag != 0 {
            return false;
        }
        if self.status != STATUS_RUNNING {
            return false;
        }
        self.stop_time = self.clock_ms;
        self.stopped = 1;
        self.stop_tag = tag;
        true
    }

    pub fn restart(&mut self, tag: u8) -> bool {
        if self.stopped != 1 || self.stop_tag != tag {
            return false;
        }
        if self.status != STATUS_RUNNING {
            return false;
        }
        self.time = self.clock_ms;
        self.pause_time += self.time - self.stop_time;
        self.stopped = 0;
        self.stop_tag = 0;
        true
    }

    pub fn end(&mut self, frames: i32) -> bool {
        if self.status != STATUS_RUNNING {
            return false;
        }
        self.end_time = self.clock_ms;
        self.status = STATUS_ENDING;
        if frames != -1 {
            self.end_hold = frames;
        }
        true
    }

    pub fn delete_request(&mut self) -> bool {
        self.status = STATUS_DELETE;
        true
    }

    pub fn delete_check(&self) -> bool {
        self.status == STATUS_FINISHED
    }

    pub fn timer_start(&mut self, mode: i32, delay: i16) {
        if self.mode == mode && self.exists {
            if delay == 0 {
                self.start();
            } else {
                self.start_delayed(delay);
            }
        }
    }

    pub fn timer_restart(&mut self, tag: u8) -> bool {
        if self.exists {
            self.restart(tag)
        } else {
            false
        }
    }

    pub fn timer_stop(&mut self, tag: u8) -> bool {
        if self.exists { self.stop(tag) } else { false }
    }

    pub fn timer_delete_request(&mut self, mode: i32) -> i32 {
        if mode == self.mode && self.exists {
            self.delete_request();
            1
        } else {
            0
        }
    }

    pub fn delete(&mut self) {
        if self.timer_mode == MODE_VOLCANO
            && self.status != STATUS_DELETE
            && self.status != STATUS_FINISHED
            && self.status != STATUS_ENDING
        {
            self.now_time_ms = self.get_time_ms();
            self.limit_time_ms = self.get_limit_time_ms();
            self.mode = self.timer_mode;
        } else {
            self.now_time_ms = 0;
            self.limit_time_ms = 0;
            self.mode = MODE_NONE;
        }
        self.exists = false;
    }

    fn anime(&mut self) {
        if self.anime_done == 0 {
            if self.anime_timer < ANIME_FRAME {
                self.anime_timer += 1;
            } else {
                self.anime_done = 1;
            }
        }
    }

    fn close_anime(&mut self) -> bool {
        self.anime_timer += 1;
        self.anime_timer >= ANIME_HOLD
    }

    fn anime_timer_reset(&mut self) {
        self.anime_timer = 0;
    }

    fn set_se(&mut self, events: &mut SimEvents) {
        if self.stopped == 0
            && self.timer_mode != MODE_AUCTION
            && self.get_rest_time_ms() / 1000 < self.prev_rest_ms / 1000
        {
            events.list.push(SimEvent::Se(SE_MINIGAME_EACH));
        }
        if !self.rest_time_check(self.se_time_left) {
            return;
        }
        let table = match self.timer_mode {
            MODE_SHIP_RACE => &SHIP_RACE_SE[..],
            MODE_VOLCANO => &VOLCANO_SE[..],
            _ => return,
        };
        let at = self.se_table_index as usize;
        if table[at].1 != TABLE_END as u32 {
            events.list.push(SimEvent::Se(table[at].1));
            self.se_table_index += 1;
            self.se_time_left = table[self.se_table_index as usize].0;
        }
    }

    pub fn execute(&mut self, talk: bool, events: &mut SimEvents) {
        if self.stopped != 1 {
            self.prev_rest_ms = self.get_rest_time_ms();
            match self.status {
                STATUS_IDLE | STATUS_PENDING | STATUS_PENDING_STOCK => {
                    self.start_time = 0;
                    self.time = 0;
                    if self.status == STATUS_PENDING || self.status == STATUS_PENDING_STOCK {
                        self.start_delay -= 1;
                        if self.start_delay <= 0 {
                            if self.status == STATUS_PENDING_STOCK {
                                self.stock_start();
                            } else {
                                self.start();
                            }
                        }
                    }
                }
                STATUS_RUNNING => {
                    self.time = self.clock_ms;
                    self.now_time_ms = self.get_time_ms();
                    self.limit_time_ms = self.get_limit_time_ms();
                    if self.get_rest_time_ms() <= 0 {
                        self.time = self.limit_time + self.start_time;
                        self.pause_time = 0;
                        self.now_time_ms = self.get_time_ms();
                        if self.timer_mode == MODE_VOLCANO {
                            self.end(END_HOLD_VOLCANO);
                        }
                    }
                }
                _ => {
                    self.time = self.end_time;
                }
            }
        }

        if self.status == STATUS_IDLE || self.status == STATUS_RUNNING {
            if self.timer_mode == MODE_VOLCANO || self.timer_mode == MODE_SHIP_RACE {
                if self.menu_pause || talk {
                    self.stop(STOP_SPEAK);
                } else {
                    self.restart(STOP_SPEAK);
                }
            }
        } else if self.status == STATUS_DELETE {
            if self.close_anime() {
                self.delete();
            }
        } else if self.status != STATUS_FINISHED && self.status == STATUS_ENDING {
            if self.end_hold <= 0 {
                if self.timer_mode == MODE_VOLCANO {
                    self.delete_request();
                    self.anime_timer_reset();
                } else {
                    self.anime_timer_reset();
                    self.status = STATUS_FINISHED;
                }
            } else {
                self.end_hold -= 1;
            }
        }

        if self.draw_delay > DRAW_DELAY {
            self.anime();
        } else {
            self.draw_delay += 1;
        }

        let digits = self.timer_digits();
        self.digits = digits;
        self.set_se(events);
    }
}

pub fn control(
    frame: Res<CurrentFrame>,
    info: Res<MsgGameInfo>,
    counters: Res<Counters>,
    mut timer: ResMut<IsleTimer>,
    mut events: ResMut<SimEvents>,
) {
    timer.set_clock((frame.0 as i64).saturating_mul(MS_PER_FRAME));
    let talk = info.event_mode == EVENT_MODE_TALK;
    timer.execute(talk, &mut events);
    timer.take_rupee_digits(counters.mini_game_rupee as i16);
}

pub fn init(world: &mut World) {
    world.init_resource::<IsleTimer>();
}
