use bevy_ecs::prelude::*;

use super::stream;

pub const RAFT_ACTORS: [&str; 5] = ["Ikada", "ikada_h", "ikadaS", "ikada_u", "Svsp"];

pub const TYPE_PLAIN: u8 = 0;
pub const TYPE_HUMAN: u8 = 1;
pub const TYPE_SMALL: u8 = 2;
pub const TYPE_LARGE: u8 = 3;
pub const TYPE_CRANE: u8 = 4;

pub const NO_PATH: u8 = 0xFF;

pub const ROPE_MIN: i32 = 15;
pub const ROPE_MAX: i32 = 100;
pub const JOINT_LEN: f32 = 10.0;
pub const ARM_FRAME: f32 = 1.5;
pub const PENDULUM: i16 = 150;
pub const LOWERED: i16 = 0x14;
pub const ARM_SCALE: i16 = 10;
pub const ARM_MAX_STEP: i16 = 10;
pub const TURN_RATE: i16 = 10;
pub const TURN_MAX_STEP: i16 = 0x100;
pub const TURN_TOLERANCE: i16 = 0x100;
pub const TURN_HALF: i16 = 0x4000;
pub const HOIST_WAIT: u16 = 0x3C;
pub const HOLD: u16 = 600;
pub const HEAP: [u32; 5] = [0x1D00, 0x2F60, 0x2E20, 0x32E0, 0x6D60];
pub const HULL_RADIUS: f32 = 1000.0;
pub const HULL_DROP: f32 = 50.0;

pub const ROLL_SPLIT: f32 = 0.5;
pub const SV_OFFSET_X: [f32; 4] = [265.0, -18.0, -97.0, -175.0];
pub const SV_WAVE_OFFSET_X: f32 = -320.0;
pub const SV_TRACK_OFFSET_X: f32 = 240.0;
pub const TERRY_WAVE_Z: f32 = 660.0;
pub const TERRY_WAVE_Y: f32 = 20.0;
pub const TERRY_TRACK_Z: f32 = -180.0;
pub const SHIP_ATTENTION_LIFT: f32 = 900.0;
pub const SHIP_EYE_LIFT: f32 = 300.0;
pub const FLAG_SCALE: f32 = 0.0;
pub const FLAG_OFFSET: [f32; 3] = [0.0, 0.0, 0.0];
pub const CONTACT_RADIUS: f32 = 300.0;
pub const STOP_TIMER_JITTER: f32 = 120.0;
pub const STOP_TIMER_BASE: f32 = 240.0;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Wait,
    CraneUp,
    CraneUpWait,
    CraneDown,
    CraneDownWait,
    CraneTurn,
    CraneReset,
    CraneWait,
    PathMove,
    Stop,
    PathMoveTerry,
    StopTerry,
    StopBombTerry,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Raft {
    pub kind: u8,
    pub mode: Mode,
    pub path_no: u8,
    pub rope: i32,
    pub arm: f32,
    pub swing: i16,
    pub swivel: i16,
    pub swivel_target: i16,
    pub roll_dir: u8,
    pub seed: u32,
    pub timer: u16,
}

impl Raft {
    pub fn spawn(parameters: u32, angle: [i16; 3]) -> Self {
        let kind = (parameters & 0xF) as u8;
        let packed = u32::from(angle[0] as u16);
        let path_no = if kind == TYPE_CRANE {
            ((parameters >> 16) & 0xFF) as u8
        } else {
            ((packed >> 8) & 0xFF) as u8
        };
        let mut raft = Self {
            kind,
            mode: Mode::Wait,
            path_no,
            rope: if kind == TYPE_CRANE { ROPE_MIN } else { 0 },
            arm: 0.0,
            swing: PENDULUM,
            swivel: 0,
            swivel_target: 0,
            roll_dir: 0,
            seed: 0x2540_F491,
            timer: 0,
        };
        if kind == TYPE_CRANE && path_no != NO_PATH {
            raft.mode = Mode::PathMove;
            raft.roll_dir = if raft.rand() < ROLL_SPLIT { 0 } else { 1 };
        }
        if kind == TYPE_HUMAN || kind == TYPE_LARGE {
            raft.mode = Mode::StopTerry;
        }
        raft
    }

    pub fn heap(&self) -> Option<u32> {
        HEAP.get(usize::from(self.kind)).copied()
    }

    pub fn rises(&self) -> bool {
        self.kind == TYPE_CRANE
    }

    pub fn arm_right(&mut self) {
        self.arm += ARM_FRAME;
    }

    pub fn arm_left(&mut self) {
        self.arm -= ARM_FRAME;
    }

    pub fn settle(&mut self, target: i16, scale: i16, max_step: i16) {
        stream::add_calc_angle_s2(&mut self.swivel, target, scale, max_step);
    }

    pub fn aligned(&self) -> bool {
        distance(self.swivel, self.swivel_target) <= TURN_TOLERANCE as u16
    }

    pub fn lowered(&mut self) -> bool {
        stream::add_calc_angle_s2(&mut self.swing, 0, ARM_SCALE, ARM_MAX_STEP);
        self.swing < LOWERED
    }

    pub fn flip(&mut self) {
        self.roll_dir = if self.rand() < ROLL_SPLIT { 0 } else { 1 };
    }

    pub fn rand(&mut self) -> f32 {
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 17;
        self.seed ^= self.seed << 5;
        (self.seed >> 8) as f32 / 16777216.0
    }

    pub fn enter(&mut self, mode: Mode) {
        self.mode = mode;
        match mode {
            Mode::CraneUpWait | Mode::CraneDownWait => self.timer = HOIST_WAIT,
            Mode::CraneWait => self.timer = HOLD,
            Mode::CraneDown | Mode::CraneTurn | Mode::CraneReset => self.swing = PENDULUM,
            Mode::PathMove => self.flip(),
            Mode::Wait | Mode::CraneUp | Mode::Stop | Mode::PathMoveTerry | Mode::StopTerry
            | Mode::StopBombTerry => {}
        }
    }

    pub fn elapsed(&mut self) -> bool {
        self.timer = self.timer.saturating_sub(1);
        self.timer == 0
    }

    pub fn advance(&mut self) -> Mode {
        match self.mode {
            Mode::CraneUp => {
                if self.rope > ROPE_MIN {
                    self.arm_right();
                    self.rope -= 1;
                    Mode::CraneUp
                } else {
                    Mode::CraneUpWait
                }
            }
            Mode::CraneUpWait => {
                if self.elapsed() {
                    Mode::CraneReset
                } else {
                    Mode::CraneUpWait
                }
            }
            Mode::CraneReset => {
                if !self.lowered() {
                    return Mode::CraneReset;
                }
                self.arm_left();
                self.swivel_target = 0;
                self.settle(0, TURN_RATE, TURN_MAX_STEP);
                if self.aligned() {
                    Mode::PathMove
                } else {
                    Mode::CraneReset
                }
            }
            Mode::CraneTurn => {
                if !self.lowered() {
                    return Mode::CraneTurn;
                }
                self.arm_right();
                self.swivel_target = if self.roll_dir == 0 {
                    TURN_HALF
                } else {
                    -TURN_HALF
                };
                self.settle(self.swivel_target, TURN_RATE, TURN_MAX_STEP);
                if self.aligned() {
                    Mode::CraneDownWait
                } else {
                    Mode::CraneTurn
                }
            }
            Mode::CraneDownWait => {
                if self.elapsed() {
                    Mode::CraneDown
                } else {
                    Mode::CraneDownWait
                }
            }
            Mode::CraneDown => {
                if !self.lowered() {
                    return Mode::CraneDown;
                }
                self.arm_left();
                if self.rope < ROPE_MAX {
                    self.rope += 1;
                    Mode::CraneDown
                } else {
                    Mode::CraneWait
                }
            }
            Mode::CraneWait => {
                if self.elapsed() {
                    Mode::CraneUp
                } else {
                    Mode::CraneWait
                }
            }
            Mode::Wait
            | Mode::PathMove
            | Mode::Stop
            | Mode::PathMoveTerry
            | Mode::StopTerry
            | Mode::StopBombTerry => self.mode,
        }
    }
}

fn distance(from: i16, to: i16) -> u16 {
    from.wrapping_sub(to).unsigned_abs()
}

pub fn is_raft(name: &str) -> bool {
    RAFT_ACTORS.contains(&name)
}

pub fn control(mut rafts: Query<&mut Raft>) {
    for mut raft in &mut rafts {
        if !raft.rises() {
            continue;
        }
        let next = raft.advance();
        if next != raft.mode {
            raft.enter(next);
        }
    }
}
