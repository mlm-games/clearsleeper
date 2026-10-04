use bevy_ecs::prelude::*;

const SIM_HZ: f32 = 30.0;
pub const TOP_BIT: u8 = 1 << 7;

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Schedule {
    pub bit: u8,
    pub timer: u16,
    pub sec: u8,
    pub enabled: bool,
}

impl Default for Schedule {
    fn default() -> Self {
        Self {
            bit: 0,
            timer: 0,
            sec: 0,
            enabled: false,
        }
    }
}

impl Schedule {
    pub fn from_header(header: Option<&crate::dzs::StageHeader>) -> Self {
        match header {
            Some(header) => Self {
                bit: 0,
                timer: 0,
                sec: header.schbit_sec,
                enabled: header.schbit_enable != 0,
            },
            None => Self::default(),
        }
    }

    pub fn test(&self, mask: u8) -> bool {
        self.enabled && self.bit & mask != 0
    }

    pub fn elapsed(&self) -> u16 {
        self.timer
    }

    pub fn window(&self, divisor: f32) -> u16 {
        ((f32::from(self.sec) / divisor) as u32 * SIM_HZ as u32) as u16
    }

    pub fn step(&mut self) {
        if !self.enabled {
            return;
        }
        if self.bit == 0 {
            self.bit = TOP_BIT;
        }
        self.timer += 1;
        if self.timer >= u16::from(self.sec) * 30 {
            self.timer = 0;
            self.bit >>= 1;
            if self.bit == 0 {
                self.bit = TOP_BIT;
            }
        }
    }
}

pub fn control(mut schedule: ResMut<Schedule>) {
    schedule.step();
}