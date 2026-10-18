use bevy_ecs::prelude::*;

use super::events::{SimEvent, SimEvents};
use super::flags::Switches;

pub const SWPUSH_ACTORS: [&str; 3] = ["Kbota_A", "Kbota_B", "KbotaC"];

const PROFILES: usize = 4;

const FLAG_OBEY_SAVE: u32 = 0x1;
const FLAG_STAY_PRESSED: u32 = 0x2;
const FLAG_IS_TOGGLE: u32 = 0x4;
const FLAG_REQ_HEAVY: u32 = 0x8;
const FLAG_ON_IS_UP: u32 = 0x10;
const FLAG_UNK20: u32 = 0x20;

const MODE_UPPER: u8 = 0;
const MODE_U_L: u8 = 1;
const MODE_LOWER: u8 = 2;
const MODE_L_U: u8 = 3;

const SWS_SHIFT: u32 = 0x8;
const SWS_MASK: u32 = 0xFF;
const TYPE_SHIFT: u32 = 0x18;
const TYPE_MASK: u32 = 0x7;
const VER_SHIFT: u32 = 0x1E;
const VER_MASK: u32 = 0x1;
const VER_MIN: u32 = 0x1;

const SWITCH_ABSENT: u8 = 0xFF;

const HEIGHT_UP: f32 = 1.0;
const HEIGHT_HEAVY: f32 = 0.9;
const TOP_DROP: f32 = -35.5;
const JOINT_MIN: f32 = -36.5;
const JOINT_MAX: f32 = 1.0;
const CULL_SIDE: f32 = 60.0;
const CULL_BASE: f32 = -2.0;
const CULL_TOP: f32 = 39.0;

const DEBOUNCE_FRAMES: i16 = 30;
const AIM_FRAMES: i16 = 1;
const RIDE_DEEP: u8 = 2;

const SE_FOOT_SW_ON: u32 = 0x6930;

#[derive(Debug, Clone, Copy)]
struct Profile {
    flags: u32,
    scale: f32,
    spring: f32,
    speed_decay: f32,
    push_speed0: f32,
    rebound: f32,
    down_frames: i16,
    mini_delay1: i16,
    push_delay: i16,
    mini_delay2: i16,
}

const PROFILE_TABLE: [Profile; PROFILES] = [
    Profile {
        flags: FLAG_STAY_PRESSED | FLAG_OBEY_SAVE,
        scale: 1.0,
        spring: 0.9,
        speed_decay: 0.6,
        push_speed0: 0.0,
        rebound: -0.3,
        down_frames: 1,
        mini_delay1: 6,
        push_delay: 6,
        mini_delay2: 13,
    },
    Profile {
        flags: FLAG_UNK20,
        scale: 1.0,
        spring: 0.9,
        speed_decay: 0.6,
        push_speed0: 0.0,
        rebound: -0.3,
        down_frames: 1,
        mini_delay1: 6,
        push_delay: 6,
        mini_delay2: 13,
    },
    Profile {
        flags: FLAG_ON_IS_UP | FLAG_STAY_PRESSED | FLAG_OBEY_SAVE,
        scale: 1.0,
        spring: 0.9,
        speed_decay: 0.6,
        push_speed0: 0.0,
        rebound: -0.3,
        down_frames: 1,
        mini_delay1: 6,
        push_delay: 6,
        mini_delay2: 13,
    },
    Profile {
        flags: FLAG_REQ_HEAVY | FLAG_STAY_PRESSED | FLAG_OBEY_SAVE,
        scale: 1.5,
        spring: 0.93,
        speed_decay: 0.42,
        push_speed0: 0.0,
        rebound: 0.0,
        down_frames: 4,
        mini_delay1: 4,
        push_delay: 15,
        mini_delay2: 4,
    },
];

fn profile_of(index: u8) -> Profile {
    PROFILE_TABLE
        .get(usize::from(index))
        .copied()
        .unwrap_or(PROFILE_TABLE[0])
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Swpush {
    pub mode: u8,
    pub switch_no: u8,
    pub switch2_no: u8,
    pub top: f32,
    pub half: f32,
    pub cull_base: f32,
    pub cull_top: f32,
    type_index: u8,
    target_h: f32,
    cur_h: f32,
    speed: f32,
    joint: f32,
    aim_hold: f32,
    shown: f32,
    aim_timer: i16,
    aim_value: f32,
    mini_timer: i16,
    mini_flg: bool,
    push_timer: i16,
    push_flg: bool,
    changing: bool,
    debounce: i16,
    riding_mode: u8,
    prev_riding: bool,
    heavy_riding: bool,
}

impl Swpush {
    pub fn spawn(parameters: u32, angle_z: i16, switches: &Switches) -> Self {
        let version = (parameters >> VER_SHIFT) & VER_MASK;
        let type_index = ((parameters >> TYPE_SHIFT) & TYPE_MASK) as u8;
        let switch_no = ((parameters >> SWS_SHIFT) & SWS_MASK) as u8;
        let switch2_no = if version >= VER_MIN {
            ((angle_z as u16) & SWS_MASK as u16) as u8
        } else {
            SWITCH_ABSENT
        };
        let profile = profile_of(type_index);
        let is_sw = switches.test(switch_no);
        let is_sw2 = switch2_no != SWITCH_ABSENT && switches.test(switch2_no);
        let on_is_up = profile.flags & FLAG_ON_IS_UP != 0;
        let toggle = profile.flags & FLAG_IS_TOGGLE != 0;
        let hold20 = profile.flags & FLAG_UNK20 != 0;
        let rising = toggle || (!is_sw && !on_is_up) || (is_sw && on_is_up);
        let mut button = Self {
            mode: MODE_UPPER,
            switch_no,
            switch2_no,
            top: 0.0,
            half: profile.scale * CULL_SIDE,
            cull_base: CULL_BASE,
            cull_top: CULL_TOP,
            type_index,
            target_h: 0.0,
            cur_h: 0.0,
            speed: 0.0,
            joint: 0.0,
            aim_hold: 0.0,
            shown: 0.0,
            aim_timer: 0,
            aim_value: 0.0,
            mini_timer: 0,
            mini_flg: false,
            push_timer: 0,
            push_flg: false,
            changing: false,
            debounce: 0,
            riding_mode: 0,
            prev_riding: false,
            heavy_riding: false,
        };
        if rising && (!hold20 || !is_sw2) {
            button.cur_h = HEIGHT_UP;
            button.joint = 0.0;
            button.aim_hold = HEIGHT_UP;
            button.shown = HEIGHT_UP;
            button.top = 0.0;
            button.enter_upper();
        } else {
            button.joint = TOP_DROP;
            button.top = TOP_DROP;
            button.enter_lower();
        }
        button
    }

    fn profile(&self) -> Profile {
        profile_of(self.type_index)
    }

    fn enter_upper(&mut self) {
        self.mode = MODE_UPPER;
        self.target_h = HEIGHT_UP;
        self.changing = false;
        self.debounce = 0;
    }

    fn enter_lower(&mut self) {
        self.mode = MODE_LOWER;
        self.target_h = 0.0;
        self.changing = false;
    }

    fn enter_down(&mut self, profile: &Profile) {
        self.mode = MODE_U_L;
        self.target_h = 0.0;
        self.speed = profile.push_speed0;
        self.aim_req(0.0, profile.down_frames);
    }

    fn enter_up(&mut self) {
        self.mode = MODE_L_U;
        self.target_h = HEIGHT_UP;
        self.aim_req(HEIGHT_UP, AIM_FRAMES);
    }

    fn aim_req(&mut self, value: f32, frames: i16) {
        self.aim_value = value;
        self.aim_timer = frames;
    }

    fn flip_switch(&self, switches: &mut Switches, on: bool) {
        if on {
            switches.set(self.switch_no);
        } else {
            switches.clear(self.switch_no);
        }
    }

    fn push_flags(&mut self, profile: &Profile) {
        if self.mini_flg {
            if self.riding_mode != 0 {
                self.mini_timer = if self.riding_mode == RIDE_DEEP {
                    profile.mini_delay2
                } else {
                    profile.mini_delay1
                };
            } else {
                self.mini_timer = self.mini_timer.wrapping_sub(1);
                if self.mini_timer <= 0 {
                    self.mini_flg = false;
                }
            }
        } else if self.riding_mode != 0 {
            self.mini_timer = self.mini_timer.wrapping_add(1);
            if self.mini_timer >= profile.mini_delay1 {
                self.mini_flg = true;
            }
        } else {
            self.mini_timer = 0;
        }

        if self.push_flg {
            if self.heavy_riding {
                self.push_timer = profile.push_delay;
            } else {
                self.push_timer = self.push_timer.wrapping_sub(1);
                if self.push_timer <= 0 {
                    self.push_flg = false;
                }
            }
        } else if self.heavy_riding {
            self.push_timer = self.push_timer.wrapping_add(1);
            if self.push_timer >= profile.push_delay {
                self.push_flg = true;
            }
        } else {
            self.push_timer = 0;
        }
    }

    fn pressing(&self, profile: &Profile) -> bool {
        if !self.mini_flg {
            return false;
        }
        profile.flags & FLAG_REQ_HEAVY == 0 || self.push_flg
    }

    fn rest_height(&self, profile: &Profile) -> f32 {
        if self.mini_flg && profile.flags & FLAG_REQ_HEAVY != 0 && !self.push_flg {
            HEIGHT_HEAVY
        } else {
            HEIGHT_UP
        }
    }

    fn on_is_up(profile: &Profile) -> bool {
        profile.flags & FLAG_ON_IS_UP != 0
    }

    fn upper_step(&mut self, profile: &Profile, switches: &Switches) {
        let pressing = self.pressing(profile);
        let height = self.rest_height(profile);
        if profile.flags & FLAG_REQ_HEAVY == 0
            && self.debounce <= 0
            && self.riding_mode != 0
            && !self.prev_riding
        {
            self.speed = profile.rebound;
            self.debounce = DEBOUNCE_FRAMES;
        } else if self.debounce > 0 {
            self.debounce = self.debounce.wrapping_sub(1);
        }

        let is_sw = switches.test(self.switch_no);
        let on_is_up = Self::on_is_up(profile);
        let obey_save = profile.flags & FLAG_OBEY_SAVE != 0;
        let match_state = obey_save && ((is_sw && !on_is_up) || (!is_sw && on_is_up));
        if pressing || match_state {
            if pressing {
                self.changing = true;
            }
            self.enter_down(profile);
        } else {
            self.target_h = height;
            self.aim_req(height, AIM_FRAMES);
        }
    }

    fn down_step(&mut self, profile: &Profile, switches: &mut Switches, events: &mut SimEvents) {
        if self.cur_h > 0.0 {
            return;
        }
        if self.changing {
            if profile.flags & FLAG_IS_TOGGLE != 0 {
                if switches.test(self.switch_no) {
                    switches.clear(self.switch_no);
                } else {
                    switches.set(self.switch_no);
                }
            } else {
                self.flip_switch(switches, !Self::on_is_up(profile));
            }
        }
        events.list.push(SimEvent::Se(SE_FOOT_SW_ON));
        self.enter_lower();
    }

    fn lower_step(&mut self, profile: &Profile, switches: &Switches) {
        let pressing = self.pressing(profile);
        let is_sw = switches.test(self.switch_no);
        let is_sw2 = self.switch2_no != SWITCH_ABSENT && switches.test(self.switch2_no);
        let pop_back_up = profile.flags & FLAG_STAY_PRESSED == 0 && !pressing;
        let on_is_up = Self::on_is_up(profile);
        let match_state_up = profile.flags & FLAG_OBEY_SAVE != 0
            && ((!is_sw && !on_is_up) || (is_sw && on_is_up))
            && !pressing;
        let gate = profile.flags & FLAG_UNK20 == 0 || !is_sw2;
        if (pop_back_up || match_state_up) && gate {
            if pop_back_up && profile.flags & FLAG_IS_TOGGLE == 0 && !pressing {
                self.changing = true;
            }
            self.enter_up();
        }
    }

    fn up_step(&mut self, profile: &Profile, switches: &mut Switches) {
        if self.cur_h < HEIGHT_UP {
            return;
        }
        if self.changing {
            self.flip_switch(switches, Self::on_is_up(profile));
        }
        self.enter_upper();
    }

    fn top_step(&mut self, profile: &Profile) {
        self.speed -= (self.cur_h - self.target_h) * profile.spring;
        self.speed -= self.speed * profile.speed_decay;
        self.cur_h += self.speed;
        self.joint = ((HEIGHT_UP - self.cur_h) * TOP_DROP).clamp(JOINT_MIN, JOINT_MAX);

        if self.aim_timer > 0 {
            self.aim_timer = self.aim_timer.wrapping_sub(1);
            if self.aim_timer == 0 {
                self.aim_hold = self.aim_value;
            }
        }
        self.shown = if self.mode == MODE_UPPER {
            self.cur_h
        } else {
            self.aim_hold
        };
        self.top = (HEIGHT_UP - self.shown) * TOP_DROP;
        if self.top < self.joint {
            self.top = self.joint;
        }
    }

    fn release_ride(&mut self) {
        self.prev_riding = self.riding_mode != 0;
        self.riding_mode = 0;
        self.heavy_riding = false;
    }
}

pub fn is_swpush(name: &str) -> bool {
    SWPUSH_ACTORS.contains(&name)
}

pub fn control(
    mut switches: ResMut<Switches>,
    mut events: ResMut<SimEvents>,
    mut buttons: Query<&mut Swpush>,
) {
    for mut button in &mut buttons {
        let profile = button.profile();
        button.push_flags(&profile);
        match button.mode {
            MODE_UPPER => button.upper_step(&profile, &switches),
            MODE_U_L => button.down_step(&profile, &mut switches, &mut events),
            MODE_LOWER => button.lower_step(&profile, &switches),
            _ => button.up_step(&profile, &mut switches),
        }
        button.top_step(&profile);
        button.release_ride();
    }
}
