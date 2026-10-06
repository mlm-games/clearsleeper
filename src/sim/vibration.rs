use bevy_ecs::prelude::*;
use glam::Vec3;

pub const RESET_FRAME: i32 = -99;
pub const PATTERN_OFF: i32 = -1;
pub const MAX_MOTOR_QUAKE_FRAME: i32 = 900;
pub const MASK_CAMERA_SHAKE: i32 = 0x3E;
pub const FLAG_MOTOR_SHAKE: i32 = 0x1;

pub const RUMBLE_SHOCK: u32 = 0x1;
pub const RUMBLE_QUAKE: u32 = 0x2;

pub const STATE_PAUSED: i32 = -1;
pub const STATE_WAITING: i32 = 0;
pub const STATE_RUNNING: i32 = 1;

pub const MOTOR_WAVE_RATE: i32 = 60;

pub const SHOCK_PATTERN_COUNT: usize = 26;
pub const QUAKE_PATTERN_COUNT: usize = 12;

pub const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
pub const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
pub const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
pub const RND_SEED: i32 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pattern {
    pub rounds: u16,
    pub length: u16,
    pub pattern: u32,
}

pub const MOTOR_SHOCK_PATTERNS: [Pattern; SHOCK_PATTERN_COUNT] = [
    Pattern {
        rounds: 0,
        length: 0,
        pattern: 0,
    },
    Pattern {
        rounds: 0,
        length: 3,
        pattern: 0xc000_0000,
    },
    Pattern {
        rounds: 0,
        length: 4,
        pattern: 0xe000_0000,
    },
    Pattern {
        rounds: 0,
        length: 6,
        pattern: 0xe800_0000,
    },
    Pattern {
        rounds: 0,
        length: 7,
        pattern: 0xec00_0000,
    },
    Pattern {
        rounds: 0,
        length: 8,
        pattern: 0xf600_0000,
    },
    Pattern {
        rounds: 0,
        length: 9,
        pattern: 0xfb00_0000,
    },
    Pattern {
        rounds: 0,
        length: 10,
        pattern: 0xf680_0000,
    },
    Pattern {
        rounds: 0,
        length: 11,
        pattern: 0xfb40_0000,
    },
    Pattern {
        rounds: 0,
        length: 3,
        pattern: 0xe000_0000,
    },
    Pattern {
        rounds: 0,
        length: 5,
        pattern: 0xf800_0000,
    },
    Pattern {
        rounds: 0,
        length: 23,
        pattern: 0xe020_2000,
    },
    Pattern {
        rounds: 0,
        length: 13,
        pattern: 0xf038_0000,
    },
    Pattern {
        rounds: 0,
        length: 13,
        pattern: 0xf038_0000,
    },
    Pattern {
        rounds: 0,
        length: 3,
        pattern: 0xe000_0000,
    },
    Pattern {
        rounds: 0,
        length: 8,
        pattern: 0xd000_0000,
    },
    Pattern {
        rounds: 0,
        length: 7,
        pattern: 0xe000_0000,
    },
    Pattern {
        rounds: 0,
        length: 8,
        pattern: 0xf000_0000,
    },
    Pattern {
        rounds: 0,
        length: 5,
        pattern: 0xf800_0000,
    },
    Pattern {
        rounds: 0,
        length: 2,
        pattern: 0xc000_0000,
    },
    Pattern {
        rounds: 0,
        length: 8,
        pattern: 0xf500_0000,
    },
    Pattern {
        rounds: 0,
        length: 12,
        pattern: 0xf8d0_0000,
    },
    Pattern {
        rounds: 0,
        length: 6,
        pattern: 0xc000_0000,
    },
    Pattern {
        rounds: 0,
        length: 9,
        pattern: 0xe800_0000,
    },
    Pattern {
        rounds: 0,
        length: 10,
        pattern: 0xf400_0000,
    },
    Pattern {
        rounds: 0,
        length: 15,
        pattern: 0xfaa0_0000,
    },
];

pub const CAMERA_SHOCK_PATTERNS: [Pattern; SHOCK_PATTERN_COUNT] =
    MOTOR_SHOCK_PATTERNS;

pub const MOTOR_QUAKE_PATTERNS: [Pattern; QUAKE_PATTERN_COUNT] = [
    Pattern {
        rounds: 0,
        length: 0,
        pattern: 0,
    },
    Pattern {
        rounds: 1,
        length: 32,
        pattern: 0x1020_2020,
    },
    Pattern {
        rounds: 1,
        length: 32,
        pattern: 0x4210_4208,
    },
    Pattern {
        rounds: 2,
        length: 32,
        pattern: 0x2222_2222,
    },
    Pattern {
        rounds: 2,
        length: 32,
        pattern: 0x2249_2249,
    },
    Pattern {
        rounds: 3,
        length: 32,
        pattern: 0x5252_5252,
    },
    Pattern {
        rounds: 3,
        length: 32,
        pattern: 0x4a94_aa95,
    },
    Pattern {
        rounds: 4,
        length: 32,
        pattern: 0x3655_5655,
    },
    Pattern {
        rounds: 4,
        length: 32,
        pattern: 0x6b6d_6b6d,
    },
    Pattern {
        rounds: 0,
        length: 3,
        pattern: 0x8000_0000,
    },
    Pattern {
        rounds: 0,
        length: 3,
        pattern: 0xc000_0000,
    },
    Pattern {
        rounds: 0,
        length: 32,
        pattern: 0xaaaa_a000,
    },
];

pub const CAMERA_QUAKE_PATTERNS: [Pattern; QUAKE_PATTERN_COUNT] = MOTOR_QUAKE_PATTERNS;

pub fn wave_data(pattern: u32, length: i32) -> [u16; 4] {
    [
        length as u16,
        (pattern >> 16) as u16,
        pattern as u16,
        0,
    ]
}

pub fn roll_shift(pattern: u32, pattern_len: i32, current: i32) -> u32 {
    if pattern_len <= 0 {
        return 0;
    }
    let loop_num = current / pattern_len;
    let loop_start = loop_num * pattern_len;
    let pos = current - loop_start;
    let lo = pattern.wrapping_shr(pos as u32);
    let hi = pattern.wrapping_shl((pattern_len - pos) as u32);
    lo | hi
}

pub fn make_bits(pattern: u32, pattern_len: i32, num_bits: i32) -> u32 {
    if pattern_len <= 0 {
        return pattern;
    }
    let masked = pattern & u32::MAX.wrapping_shl((0x20 - pattern_len) as u32);
    let mut out = masked;
    let mut at = pattern_len;
    while at < num_bits {
        out = masked | out.wrapping_shr(pattern_len as u32);
        at += pattern_len;
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraChannel {
    pub pattern_idx: i32,
    pub pattern: u32,
    pub length: i32,
    pub rounds: i32,
    pub flags: i32,
    pub coord: Vec3,
    pub current_frame: i32,
}

impl Default for CameraChannel {
    fn default() -> Self {
        Self {
            pattern_idx: PATTERN_OFF,
            pattern: 0,
            length: 0,
            rounds: 0,
            flags: 0,
            coord: Vec3::ZERO,
            current_frame: RESET_FRAME,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MotorChannel {
    pub pattern_idx: i32,
    pub pattern: u32,
    pub length: i32,
    pub rounds: i32,
    pub current_frame: i32,
    pub stop_frame: i32,
}

impl Default for MotorChannel {
    fn default() -> Self {
        Self {
            pattern_idx: PATTERN_OFF,
            pattern: 0,
            length: 0,
            rounds: 0,
            current_frame: RESET_FRAME,
            stop_frame: RESET_FRAME,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShakeRequest {
    pub active: bool,
    pub length: i32,
    pub pattern: u32,
    pub flags: i32,
    pub coord: Vec3,
}

impl Default for ShakeRequest {
    fn default() -> Self {
        Self {
            active: false,
            length: 0,
            pattern: 0,
            flags: 0,
            coord: Vec3::ZERO,
        }
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Vibration {
    pub camera_shock: CameraChannel,
    pub camera_quake: CameraChannel,
    pub motor_shock: MotorChannel,
    pub motor_quake: MotorChannel,
    pub frame_idx: i32,
    pub state: i32,
    pub vibration_enabled: bool,
    pub seed: [i32; 3],
    pub shake: ShakeRequest,
    pub wave_active: bool,
    pub wave_pattern: u32,
    pub wave_length: i32,
    pub wave_loop_forever: bool,
    pub wave_rate: i32,
}

impl Default for Vibration {
    fn default() -> Self {
        Self {
            camera_shock: CameraChannel::default(),
            camera_quake: CameraChannel::default(),
            motor_shock: MotorChannel::default(),
            motor_quake: MotorChannel::default(),
            frame_idx: 0,
            state: STATE_WAITING,
            vibration_enabled: true,
            seed: [RND_SEED; 3],
            shake: ShakeRequest::default(),
            wave_active: false,
            wave_pattern: 0,
            wave_length: 0,
            wave_loop_forever: false,
            wave_rate: MOTOR_WAVE_RATE,
        }
    }
}

impl Vibration {
    pub fn rnd(&mut self) -> f32 {
        for lane in 0..3 {
            self.seed[lane] = (self.seed[lane] * RND_MUL[lane]) % RND_MOD[lane];
        }
        let sum = (self.seed[0]) as f32 / RND_SCALE[0]
            + (self.seed[1]) as f32 / RND_SCALE[1]
            + (self.seed[2]) as f32 / RND_SCALE[2];
        (sum % 1.0).abs()
    }

    pub fn random_bit(&mut self, rounds: i32, pattern_len: i32) -> u32 {
        let mut pattern: u32 = 0;
        for _ in 0..rounds {
            let shift = (pattern_len as f32 * self.rnd()) as u32;
            pattern |= (1u32 << 30).wrapping_shr(shift);
        }
        pattern
    }

    pub fn rumbling(&self) -> bool {
        self.wave_active
    }

    pub fn strength(&self) -> u32 {
        self.wave_pattern
    }

    pub fn wave_data(&self) -> [u16; 4] {
        wave_data(self.wave_pattern, self.wave_length)
    }

    pub fn set_default(&mut self) {
        self.motor_shock.pattern_idx = PATTERN_OFF;
        self.camera_shock.pattern_idx = PATTERN_OFF;
        self.motor_quake.pattern_idx = PATTERN_OFF;
        self.camera_quake.pattern_idx = PATTERN_OFF;
        self.motor_shock.current_frame = RESET_FRAME;
        self.camera_shock.current_frame = RESET_FRAME;
        self.motor_quake.current_frame = RESET_FRAME;
        self.camera_quake.current_frame = RESET_FRAME;
        self.motor_quake.stop_frame = RESET_FRAME;
        self.motor_shock.stop_frame = RESET_FRAME;
        self.state = STATE_WAITING;
        self.frame_idx = 0;
        self.shake.active = false;
        self.wave_active = false;
    }

    pub fn initialize(&mut self) {
        self.set_default();
    }

    pub fn kill(&mut self) {
        self.set_default();
    }

    pub fn check_quake(&self) -> bool {
        self.camera_quake.pattern_idx != PATTERN_OFF
            || self.motor_quake.pattern_idx != PATTERN_OFF
    }

    pub fn pause(&mut self) {
        if self.state == STATE_PAUSED {
            return;
        }
        if self.motor_shock.pattern_idx != PATTERN_OFF
            || self.motor_quake.pattern_idx != PATTERN_OFF
        {
            self.wave_active = false;
        }
        self.camera_shock.pattern_idx = PATTERN_OFF;
        self.motor_shock.pattern_idx = PATTERN_OFF;
        self.camera_shock.current_frame = RESET_FRAME;
        self.motor_shock.current_frame = RESET_FRAME;
        if self.camera_quake.pattern_idx != PATTERN_OFF {
            self.camera_quake.current_frame = 0;
        }
        if self.motor_quake.pattern_idx != PATTERN_OFF {
            self.motor_quake.current_frame = 0;
        }
        self.state = STATE_PAUSED;
    }

    pub fn start_shock(&mut self, pattern_idx: i32, flags: i32, coord: Vec3) -> bool {
        let mut ret = false;
        if flags & MASK_CAMERA_SHAKE != 0 {
            let at = pattern_idx as usize;
            self.camera_shock.pattern_idx = pattern_idx;
            self.camera_shock.current_frame = 0;
            self.camera_shock.flags = flags;
            self.camera_shock.coord = coord;
            self.camera_shock.pattern = CAMERA_SHOCK_PATTERNS[at].pattern;
            self.camera_shock.length = i32::from(CAMERA_SHOCK_PATTERNS[at].length);
            self.camera_shock.rounds = i32::from(CAMERA_SHOCK_PATTERNS[at].rounds);
            ret = true;
        }
        if flags & FLAG_MOTOR_SHAKE != 0 {
            let at = pattern_idx as usize;
            self.motor_shock.pattern_idx = pattern_idx;
            self.motor_shock.current_frame = 0;
            self.motor_shock.pattern = MOTOR_SHOCK_PATTERNS[at].pattern;
            self.motor_shock.length = i32::from(MOTOR_SHOCK_PATTERNS[at].length);
            ret = true;
        }
        ret
    }

    pub fn start_quake(&mut self, pattern_idx: i32, flags: i32, coord: Vec3) -> bool {
        let mut ret = false;
        if flags & MASK_CAMERA_SHAKE != 0 {
            let at = pattern_idx as usize;
            self.camera_quake.pattern_idx = pattern_idx;
            self.camera_quake.current_frame = 0;
            self.camera_quake.flags = flags;
            self.camera_quake.coord = coord;
            self.camera_quake.pattern = CAMERA_QUAKE_PATTERNS[at].pattern;
            self.camera_quake.length = i32::from(CAMERA_QUAKE_PATTERNS[at].length);
            self.camera_quake.rounds = i32::from(CAMERA_QUAKE_PATTERNS[at].rounds);
            ret = true;
        }
        if flags & FLAG_MOTOR_SHAKE != 0 {
            let at = pattern_idx as usize;
            self.motor_quake.pattern_idx = pattern_idx;
            self.motor_quake.current_frame = 0;
            self.motor_quake.pattern = MOTOR_QUAKE_PATTERNS[at].pattern;
            self.motor_quake.length = i32::from(MOTOR_QUAKE_PATTERNS[at].length);
            self.motor_quake.rounds = i32::from(CAMERA_QUAKE_PATTERNS[at].rounds);
            ret = true;
        }
        ret
    }

    pub fn start_quake_pattern(&mut self, pattern: &[u8], rounds: i32, flags: i32, coord: Vec3) -> bool {
        let byte = |at: usize| i32::from(pattern.get(at).copied().unwrap_or(0));
        let pattern_len = (byte(0) << 8) | byte(1);
        let bits = (byte(if pattern_len >= 1 { 2 } else { 0 }) << 24)
            | (byte(if pattern_len >= 9 { 3 } else { 0 }) << 16)
            | (byte(if pattern_len >= 17 { 4 } else { 0 }) << 8)
            | byte(if pattern_len >= 25 { 5 } else { 0 });
        let spread = make_bits(bits as u32, byte(1), 32);
        let mut ret = false;
        if flags & MASK_CAMERA_SHAKE != 0 {
            self.camera_quake.pattern_idx = 0;
            self.camera_quake.current_frame = 0;
            self.camera_quake.flags = flags;
            self.camera_quake.coord = coord;
            self.camera_quake.pattern = spread;
            self.camera_quake.length = 32;
            self.camera_quake.rounds = rounds;
            ret = true;
        }
        if flags & FLAG_MOTOR_SHAKE != 0 {
            self.motor_quake.pattern_idx = 0;
            self.motor_quake.current_frame = 0;
            self.motor_quake.pattern = spread;
            self.motor_quake.length = 32;
            self.motor_quake.rounds = rounds;
            ret = true;
        }
        ret
    }

    pub fn stop_quake(&mut self, flags: i32) -> i32 {
        let mut ret = 0;
        if flags & MASK_CAMERA_SHAKE != 0 {
            self.camera_quake.flags &= !flags;
            if self.camera_quake.flags == 0 {
                self.camera_quake.pattern_idx = PATTERN_OFF;
            }
            self.camera_quake.current_frame = 0;
            ret = 1;
        }
        if flags & FLAG_MOTOR_SHAKE != 0 && self.motor_quake.pattern_idx != PATTERN_OFF {
            self.motor_quake.pattern_idx = PATTERN_OFF;
            self.motor_quake.current_frame = 0;
            ret = 1;
        }
        ret
    }

    fn request_shake(&mut self, length: i32, pattern: u32, flags: i32, coord: Vec3) {
        self.shake.active = true;
        self.shake.length = length;
        self.shake.pattern = pattern;
        self.shake.flags = flags;
        self.shake.coord = coord;
    }

    fn clear_shake(&mut self) {
        self.shake.active = false;
    }

    fn request_wave(&mut self, pattern: u32, length: i32, loop_forever: bool) {
        self.wave_active = true;
        self.wave_pattern = pattern;
        self.wave_length = length;
        self.wave_loop_forever = loop_forever;
        self.wave_rate = MOTOR_WAVE_RATE;
    }

    fn clear_wave(&mut self) {
        self.wave_active = false;
    }

    pub fn run(&mut self) -> i32 {
        self.state = STATE_RUNNING;
        if !self.vibration_enabled {
            self.motor_shock.pattern_idx = PATTERN_OFF;
            self.motor_quake.pattern_idx = PATTERN_OFF;
            self.motor_shock.current_frame = RESET_FRAME;
            self.motor_quake.current_frame = RESET_FRAME;
        }

        if self.camera_shock.current_frame == 0 || self.camera_quake.current_frame == 0 {
            let mut rumble: u32 = 0;
            if self.camera_shock.pattern_idx == PATTERN_OFF {
                self.camera_shock.current_frame = RESET_FRAME;
            } else if self.camera_shock.current_frame >= 0 {
                rumble |= RUMBLE_SHOCK;
            }
            if self.camera_quake.pattern_idx == PATTERN_OFF {
                self.camera_quake.current_frame = RESET_FRAME;
            } else if self.camera_quake.current_frame >= 0 {
                rumble |= RUMBLE_QUAKE;
            }
            if rumble == RUMBLE_SHOCK {
                let length = self.camera_shock.length;
                let mut pattern = self.camera_shock.pattern;
                let rounds = self.camera_shock.rounds;
                pattern |= self.random_bit(rounds, length);
                let flags = self.camera_shock.flags;
                let coord = self.camera_shock.coord;
                self.request_shake(length, pattern, flags, coord.normalize_or_zero());
            } else if rumble == RUMBLE_QUAKE {
                let length = self.camera_quake.length;
                let mut pattern =
                    roll_shift(self.camera_quake.pattern, length, self.frame_idx) as i32;
                let rounds = self.camera_quake.rounds;
                pattern |= self.random_bit(rounds, length) as i32;
                let flags = self.camera_quake.flags;
                let coord = self.camera_quake.coord;
                self.request_shake(length, pattern as u32, flags, coord.normalize_or_zero());
            } else if rumble == RUMBLE_SHOCK | RUMBLE_QUAKE {
                let shock_frame = self.camera_shock.current_frame;
                let mut pattern =
                    (self.camera_shock.pattern as i32).wrapping_shl(shock_frame as u32);
                let length = self.camera_shock.length - shock_frame;
                let bits = make_bits(
                    self.camera_quake.pattern,
                    self.camera_quake.length,
                    length,
                );
                pattern |= roll_shift(bits, length, self.frame_idx) as i32;
                let rounds = self.camera_shock.rounds.max(self.camera_quake.rounds);
                pattern |= self.random_bit(rounds, length) as i32;
                let flags = self.camera_shock.flags | self.camera_quake.flags;
                let coord = self.camera_shock.coord + self.camera_quake.coord;
                self.request_shake(length, pattern as u32, flags, coord.normalize_or_zero());
                self.camera_shock.current_frame = 0;
                self.camera_quake.current_frame = 0;
            } else {
                self.clear_shake();
            }
        }

        if self.motor_quake.current_frame >= MAX_MOTOR_QUAKE_FRAME {
            self.clear_wave();
            self.motor_quake.current_frame = -1;
        }

        if self.motor_shock.current_frame == 0 || self.motor_quake.current_frame == 0 {
            let mut rumble: u32 = 0;
            if self.motor_shock.pattern_idx == PATTERN_OFF {
                self.motor_shock.current_frame = RESET_FRAME;
            } else if self.motor_shock.current_frame >= 0 {
                rumble |= RUMBLE_SHOCK;
            }
            if self.motor_quake.pattern_idx == PATTERN_OFF {
                self.motor_quake.current_frame = RESET_FRAME;
            } else if self.motor_quake.current_frame >= 0 {
                rumble |= RUMBLE_QUAKE;
            }
            if rumble == RUMBLE_SHOCK {
                let length = self.motor_shock.length;
                let mut pattern = self.motor_shock.pattern;
                let rounds = self.motor_shock.rounds;
                pattern |= self.random_bit(rounds, length);
                self.motor_shock.stop_frame = length;
                self.request_wave(pattern, length, false);
            } else if rumble == RUMBLE_QUAKE {
                let length = self.motor_quake.length;
                let mut pattern =
                    roll_shift(self.motor_quake.pattern, length, self.frame_idx) as i32;
                let rounds = self.motor_quake.rounds;
                pattern |= self.random_bit(rounds, length) as i32;
                self.motor_quake.stop_frame = i32::MAX;
                self.request_wave(pattern as u32, length, true);
            } else if rumble == RUMBLE_SHOCK | RUMBLE_QUAKE {
                let shock_frame = self.motor_shock.current_frame;
                let mut pattern = (self.motor_shock.pattern as i32).wrapping_shl(shock_frame as u32);
                let length = self.motor_shock.length - shock_frame;
                let bits = make_bits(
                    self.motor_quake.pattern,
                    self.motor_quake.length,
                    length,
                );
                pattern |= roll_shift(bits, length, self.frame_idx) as i32;
                let rounds = self.motor_shock.rounds.max(self.motor_quake.rounds);
                pattern |= self.random_bit(rounds, length) as i32;
                self.motor_shock.stop_frame = length;
                self.motor_quake.stop_frame = length;
                self.motor_shock.current_frame = 0;
                self.motor_quake.current_frame = 0;
                self.request_wave(pattern as u32, length, false);
            } else {
                self.clear_wave();
                self.motor_shock.stop_frame = RESET_FRAME;
                self.motor_quake.stop_frame = RESET_FRAME;
            }
        }

        if self.camera_shock.current_frame >= -1 {
            self.camera_shock.current_frame += 1;
            if self.camera_shock.current_frame > self.camera_shock.length {
                self.camera_shock.current_frame = 0;
                self.camera_shock.pattern_idx = PATTERN_OFF;
            }
        }
        if self.motor_shock.current_frame >= -1 {
            self.motor_shock.current_frame += 1;
            if self.motor_shock.current_frame > self.motor_shock.stop_frame {
                self.motor_shock.current_frame = 0;
                self.motor_shock.pattern_idx = PATTERN_OFF;
            }
        }
        if self.camera_quake.current_frame >= -1 {
            self.camera_quake.current_frame += 1;
            if self.camera_quake.current_frame > self.camera_quake.length {
                self.camera_quake.current_frame = 0;
            }
        }
        if self.motor_quake.current_frame >= -1 {
            self.motor_quake.current_frame += 1;
            if self.motor_quake.current_frame > self.motor_quake.stop_frame {
                self.motor_quake.current_frame = 0;
            }
        }
        self.frame_idx += 1;
        1
    }
}

pub fn control(mut vibration: ResMut<Vibration>) {
    vibration.run();
}

pub fn init(world: &mut World) {
    world.init_resource::<Vibration>();
}
