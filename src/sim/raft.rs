use bevy_ecs::prelude::*;

pub const RAFT_ACTORS: [&str; 5] = ["Ikada", "ikada_h", "ikadaS", "ikada_u", "Svsp"];

pub const TYPE_PLAIN: u8 = 0;
pub const TYPE_HUMAN: u8 = 1;
pub const TYPE_SMALL: u8 = 2;
pub const TYPE_LARGE: u8 = 3;
pub const TYPE_CRANE: u8 = 4;

pub const NO_PATH: u8 = 0xFF;

pub const ROPE_MIN: i32 = 15;
pub const ROPE_MAX: i32 = 100;
pub const ROPE_CAP: i32 = 200;
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
pub const CULL_FAR: f32 = 10.0;
pub const CONTACT_RADIUS: f32 = 75.0;
pub const STOP_TIMER_MAX: f32 = 120.0;
pub const STOP_TIMER_BASE: f32 = 240.0;

pub const FLAG_PARAM: [u32; 5] = [4, 4, 4, 4, 0x0200_0000];
pub const FLAG_SCALE: [f32; 5] = [0.2, 0.0, 0.0, 0.0, 0.12];
pub const FLAG_LIFT: [f32; 5] = [700.0, 0.0, 0.0, 0.0, 530.0];
pub const FLAG_SIDE: [f32; 5] = [0.0, 0.0, 0.0, 0.0, 100.0];

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
    pub counterweight: bool,
    pub timer: u16,
}

impl Raft {
    pub fn spawn(parameters: u32, angle: [i16; 3], counterweight: bool) -> Self {
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
            counterweight,
            timer: 0,
        };
        if kind == TYPE_CRANE && path_no != NO_PATH {
            raft.mode = Mode::PathMove;
        }
        if kind == TYPE_HUMAN || kind == TYPE_LARGE {
            raft.mode = Mode::StopTerry;
        }
        raft
    }

    pub fn heap(&self) -> u32 {
        HEAP[usize::from(self.kind) % HEAP.len()]
    }

    pub fn flag_scale(&self) -> f32 {
        FLAG_SCALE[usize::from(self.kind) % FLAG_SCALE.len()]
    }

    pub fn flag_lift(&self) -> f32 {
        FLAG_LIFT[usize::from(self.kind) % FLAG_LIFT.len()]
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
        chase(&mut self.swivel, target, scale, max_step);
    }

    pub fn aligned(&self) -> bool {
        distance(self.swivel, self.swivel_target) <= TURN_TOLERANCE
    }

    pub fn lowered(&mut self) -> bool {
        chase(&mut self.swing, 0, ARM_SCALE, ARM_MAX_STEP);
        self.swing < LOWERED
    }

    pub fn enter(&mut self, mode: Mode) {
        self.mode = mode;
        match mode {
            Mode::CraneUpWait | Mode::CraneDownWait => self.timer = HOIST_WAIT,
            Mode::CraneWait => self.timer = HOLD,
            Mode::CraneDown | Mode::CraneTurn | Mode::CraneReset => self.swing = PENDULUM,
            Mode::Wait | Mode::CraneUp | Mode::PathMove | Mode::Stop | Mode::PathMoveTerry
            | Mode::StopTerry | Mode::StopBombTerry => {}
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
                self.swivel_target = if self.counterweight {
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
                if self.rope < ROPE_MAX {
                    self.arm_left();
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

fn chase(value: &mut i16, target: i16, scale: i16, max_step: i16) {
    let step = target.wrapping_sub(*value) / scale;
    *value = if step > max_step {
        value.wrapping_add(max_step)
    } else if step < -max_step {
        value.wrapping_sub(max_step)
    } else {
        value.wrapping_add(step)
    };
}

fn distance(from: i16, to: i16) -> i16 {
    from.wrapping_sub(to).abs()
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

pub fn hull(actor: &Actor, scale: f32) -> (f32, f32, f32) {
    (
        -scale * HULL_RADIUS,
        -scale * HULL_DROP,
        -scale * HULL_RADIUS,
    )
}