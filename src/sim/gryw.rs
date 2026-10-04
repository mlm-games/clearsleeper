use bevy_ecs::prelude::*;

use super::events::{SimEvent, SimEvents};
use super::flags::Switches;

pub const GRYW_ACTORS: [&str; 1] = ["Gryw00"];

pub const PHASE_WAIT: u8 = 0;
pub const PHASE_SPREAD: u8 = 1;
pub const PHASE_RISE: u8 = 2;
pub const PHASE_LOOP_WAIT: u8 = 3;
pub const PHASE_HIGH_WATER: u8 = 4;

pub const SWITCH_NONE: u8 = 0xFF;

pub const HIGH_WATER_LEVEL: f32 = 510.0;
pub const LOW_WATER_LEVEL: f32 = -5.0;
pub const SPREAD_WATER_LEVEL: f32 = 5.0;
pub const WAVE_AMPLITUDE: f32 = -10.0;
pub const WAVE_START: f32 = 0.0;
pub const WATER_WAVE_GROWTH_SPEED: f32 = 0.10526316;
pub const WATER_RISE_SPEED: f32 = 1.6777408;

pub const SPREAD_PHASE_ANIM_LEN: f32 = 95.0;
pub const RISE_PHASE_ANIM_LEN: f32 = 299.0;

pub const GEYSER_SOUND_LEN: i32 = 310;

pub const BTK_STOP_FRAME: f32 = 400.0;
pub const BTK_HOLD_RATE: f32 = 0.0;
pub const BTK_PLAY_RATE: f32 = 1.0;
pub const BTK_FRAME_EPS: f32 = 0.001;
pub const BTK_STATE_STOP: u8 = 0x01;

pub const SE_W_OUT: u32 = 0x704B;
pub const SE_GEYSER: u32 = 0x704C;
pub const SE_W_UP: u32 = 0x704D;

const SWSAVE_SHIFT: u32 = 0x00;
const SWSAVE_WIDTH: u32 = 0x08;
const SWSAVE_MASK: u32 = (1 << SWSAVE_WIDTH) - 1;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Gryw {
    pub switch_no: u8,
    pub mode: u8,
    pub water_lv: f32,
    pub water_max: f32,
    pub water_inc: f32,
    pub wave: f32,
    pub wave_max: f32,
    pub wave_inc: f32,
    pub should_play_se: bool,
    pub geyser_se: i32,
    pub btk_end: f32,
    pub btk_rate: f32,
    pub btk_frame: f32,
    pub btk_state: u8,
}

impl Gryw {
    pub fn spawn(parameters: u32, switches: &Switches) -> Self {
        let switch_no = ((parameters >> SWSAVE_SHIFT) & SWSAVE_MASK) as u8;
        let solved = switch_no != SWITCH_NONE && switches.test(switch_no);
        Self {
            switch_no,
            mode: if solved { PHASE_HIGH_WATER } else { PHASE_WAIT },
            water_lv: if solved { HIGH_WATER_LEVEL } else { LOW_WATER_LEVEL },
            water_max: if solved { HIGH_WATER_LEVEL } else { LOW_WATER_LEVEL },
            water_inc: 0.0,
            wave: if solved { WAVE_AMPLITUDE } else { WAVE_START },
            wave_max: if solved { WAVE_AMPLITUDE } else { WAVE_START },
            wave_inc: 0.0,
            should_play_se: false,
            geyser_se: 0,
            btk_end: BTK_STOP_FRAME,
            btk_rate: BTK_HOLD_RATE,
            btk_frame: 0.0,
            btk_state: 0,
        }
    }

    fn btk_play(&mut self) {
        self.btk_state = 0;
        self.btk_frame += self.btk_rate;
        if self.btk_frame < 0.0 {
            self.btk_frame = 0.0;
            self.btk_rate = 0.0;
            self.btk_state |= BTK_STATE_STOP;
        }
        if self.btk_frame >= self.btk_end {
            self.btk_frame = self.btk_end - BTK_FRAME_EPS;
            self.btk_rate = 0.0;
            self.btk_state |= BTK_STATE_STOP;
        }
    }

    fn btk_check_frame(&self, pass: f32) -> bool {
        let cur = self.btk_frame;
        let mut next = cur + self.btk_rate;
        if next < 0.0 {
            next = 0.0;
        }
        if next >= self.btk_end {
            next = self.btk_end - BTK_FRAME_EPS;
        }
        if cur <= next {
            cur <= pass && pass < next
        } else {
            next <= pass && pass < cur
        }
    }

    fn btk_stop(&self) -> bool {
        self.btk_state & BTK_STATE_STOP != 0 || self.btk_rate == 0.0
    }

    fn set_se(&mut self, events: &mut SimEvents) {
        if !self.should_play_se {
            return;
        }
        if self.water_lv >= HIGH_WATER_LEVEL {
            self.should_play_se = false;
        }
        if self.geyser_se > 0 {
            self.geyser_se -= 1;
            events.list.push(SimEvent::Se(SE_GEYSER));
        }
        events.list.push(SimEvent::Se(SE_W_OUT));
        events.list.push(SimEvent::Se(SE_W_UP));
    }

    fn switch_wait_act_proc(&mut self, switches: &Switches) {
        if self.switch_no == SWITCH_NONE {
            return;
        }
        if !switches.test(self.switch_no) {
            return;
        }
        self.btk_rate = BTK_PLAY_RATE;
        self.wave_inc = -WATER_WAVE_GROWTH_SPEED;
        self.wave_max = WAVE_AMPLITUDE;
        self.water_inc = WATER_WAVE_GROWTH_SPEED;
        self.water_max = SPREAD_WATER_LEVEL;
        self.geyser_se = GEYSER_SOUND_LEN;
        self.should_play_se = true;
        self.mode = PHASE_SPREAD;
    }

    fn spread_water_face_act_proc(&mut self) {
        if self.btk_check_frame(SPREAD_PHASE_ANIM_LEN) {
            self.mode = PHASE_RISE;
        }
    }

    fn water_level_move_wait_act_proc(&mut self) {
        if self.btk_check_frame(RISE_PHASE_ANIM_LEN) {
            self.water_inc = WATER_RISE_SPEED;
            self.water_max = HIGH_WATER_LEVEL;
            self.mode = PHASE_LOOP_WAIT;
        }
    }

    fn anime_loop_start_wait_act_proc(&mut self) {
        if self.btk_stop() {
            self.mode = PHASE_HIGH_WATER;
        }
    }

    fn high_water_level_act_proc(&mut self) {}

    fn act_proc(&mut self, switches: &Switches) {
        match self.mode {
            PHASE_WAIT => self.switch_wait_act_proc(switches),
            PHASE_SPREAD => self.spread_water_face_act_proc(),
            PHASE_RISE => self.water_level_move_wait_act_proc(),
            PHASE_LOOP_WAIT => self.anime_loop_start_wait_act_proc(),
            PHASE_HIGH_WATER => self.high_water_level_act_proc(),
            _ => {}
        }
    }

    fn execute(&mut self, switches: &Switches, events: &mut SimEvents) {
        self.wave += self.wave_inc;
        if self.wave < self.wave_max {
            self.wave = self.wave_max;
            self.wave_inc = 0.0;
        }
        self.water_lv += self.water_inc;
        if self.water_lv > self.water_max {
            self.water_lv = self.water_max;
            self.water_inc = 0.0;
        }
        self.set_se(events);
        self.btk_play();
        self.act_proc(switches);
    }
}

pub fn is_gryw(name: &str) -> bool {
    GRYW_ACTORS.contains(&name)
}

pub fn control(
    switches: Res<Switches>,
    mut events: ResMut<SimEvents>,
    mut pools: Query<&mut Gryw>,
) {
    for mut pool in &mut pools {
        pool.execute(&switches, &mut events);
    }
}