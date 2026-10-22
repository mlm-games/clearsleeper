use bevy_ecs::prelude::*;
use glam::Vec3;

use super::events::{SimEvent, SimEvents};
use super::intent::PadButtons;
use super::player::Player;

pub const TITLE_ACTOR: &str = "title";

pub fn is_title(name: &str) -> bool {
    TITLE_ACTORS.contains(&name)
}

pub const TITLE_ACTORS: [&str; 1] = [TITLE_ACTOR];

pub const ARCHIVE: &str = "TlogoE";
pub const SCREEN_FILE: &str = "title_logo_e.blo";

pub const TAG_PRES: u32 = u32::from_be_bytes(*b"pres");
pub const TAG_NINT: u32 = u32::from_be_bytes(*b"nint");
pub const TAG_ZELD: u32 = u32::from_be_bytes(*b"zeld");
pub const TAG_ZELJ: u32 = u32::from_be_bytes(*b"zelj");
pub const TAG_EFT1: u32 = u32::from_be_bytes(*b"eft1");
pub const TAG_EFT2: u32 = u32::from_be_bytes(*b"eft2");

pub const PANE_COUNT: usize = 6;
pub const PANE_ZELD: usize = 0;
pub const PANE_ZELJ: usize = 1;
pub const PANE_PRES: usize = 2;
pub const PANE_NINT: usize = 3;
pub const PANE_EFT1: usize = 4;
pub const PANE_EFT2: usize = 5;

pub const PANE_TAG: [u32; PANE_COUNT] =
    [TAG_ZELD, TAG_ZELJ, TAG_PRES, TAG_NINT, TAG_EFT1, TAG_EFT2];

/// The panes the title screen puts on the screen. `zelj` is the Japanese-only subtitle logo,
/// which the decomp holds at zero outside Japan (`d_a_title.cpp:380`), `eft1` and `eft2` are
/// the 8x8 dummies whose alpha the decomp never commits (`d_a_title.cpp:516-518`), and
/// `pres` is left out because the port's owner asked for PRESS START to be ignored; retail
/// fades it in over frames 160-180 and blinks it once the title is entered
/// (`d_a_title.cpp:413-419`, `:444-454`).
pub const PANE_DRAWN: &[usize] = &[PANE_ZELD, PANE_NINT];

pub const PANE_ORIGIN: [[f32; 2]; PANE_COUNT] = [
    [150.0, 108.0],
    [134.0, 247.0],
    [181.0, 336.0],
    [162.0, 415.0],
    [261.0, 246.0],
    [477.0, 246.0],
];

pub const PANE_INIT_ALPHA: [u8; PANE_COUNT] = [0xFF, 0xFF, 0xFF, 0xFF, 0, 0];

pub const SCREEN_HALF_W: f32 = 320.0;
pub const SCREEN_HALF_H: f32 = 240.0;

pub const ATTR_0X00: f32 = 0.0;
pub const ATTR_0X04: f32 = 0.0;
pub const ATTR_0X08: f32 = 0.9;
pub const ATTR_0X0C: f32 = 0.1;
pub const ATTR_0X10: f32 = -57.0;
pub const ATTR_0X14: f32 = -3.0;
pub const ATTR_0X18: f32 = 1.0;
pub const ATTR_0X1C: f32 = 1.0;
pub const ATTR_0X20: f32 = 120.0;
pub const ATTR_0X24: f32 = 10.0;
pub const ATTR_0X28: f32 = 120.0;
pub const ATTR_0X2C: f32 = 10.0;
pub const ATTR_0X30: f32 = 85.0;
pub const ATTR_0X34: f32 = 5.0;

pub const EMODE_NONE: u8 = 0;
pub const EMODE_RESET: u8 = 1;
pub const EMODE_LOOP: u8 = 2;
pub const EMODE_REVERSE: u8 = 3;
pub const EMODE_LOOP_REVERSE: u8 = 4;

pub const STATE_STOP: u8 = 1;
pub const STATE_LOOP: u8 = 2;

pub const BCK_TITLE_SHIP_FRAME_MAX: f32 = 360.0;
pub const BPK_TITLE_SHIP_FRAME_MAX: f32 = -1.0;
pub const BTK_SUBTITLE_START_FRAME_MAX: f32 = 45.0;
pub const BTK_SUBTITLE_KIRARI_FRAME_MAX: f32 = 20.0;

pub const START_DELAY: i32 = 120;
pub const SMOKE_RISE: i32 = -50;
pub const ENTER_FRAME: i32 = 200;

pub const ZELD_HOLD: i32 = 30;
pub const ZELD_RAMP: i32 = 80;
pub const ZELD_SPAN: f32 = 50.0;
pub const KIRAKIRA_FRAME: i32 = 80;
pub const KIRAKIRA_LAST: i32 = 115;
pub const KIRAKIRA_STEP: f32 = 35.0;
pub const NINT_HOLD: i32 = 150;
pub const NINT_RAMP: i32 = 170;
pub const NINT_SPAN: f32 = 20.0;
pub const PRES_HOLD: i32 = 160;
pub const PRES_RAMP: i32 = 180;
pub const PRES_SPAN: f32 = 20.0;
pub const PRES_PERIOD: i32 = 100;
pub const PRES_HALF: i32 = 50;
pub const PRES_SPAN_HALF: f32 = 50.0;
pub const ALPHA_SET_COUNT: usize = 4;

pub const WIND_FRAME: f32 = 120.0;
pub const WIND_BASE: f32 = 10.0;
pub const WIND_YAW: f32 = 32768.0;
pub const WIND_TILT: f32 = 4000.0;

pub const KIRARI_SPAN: i32 = 20;
pub const KIRARI_FRAME: f32 = 120.0;
pub const KIRARI_BASE: f32 = 10.0;
pub const KIRARI_BASE_CSUM: f32 = 130.0;

pub const SHIP_FRAME_BASE: f32 = 100.0;
pub const SHIP_FRAME_STEP: i32 = 2;

pub const SHIP_Z: f32 = 1000.0;
pub const SHIP_ROTATE_Y: i16 = 0x4000;
pub const SUBTITLE_Z: f32 = -10000.0;
pub const KIRARI_Z: f32 = -10010.0;
pub const SUBTITLE_ROTATE_Y: i16 = -0x8000;

pub const PA_TITLESMOKE: u32 = 0x83F9;
pub const PA_TITLEWIND: u32 = 0x83FA;
pub const PA_TITLEKIRAKIRA: u32 = 0x83FB;

pub const SE_TITLE_KIRA: u32 = 0x0909;
pub const SE_TITLE_WIND: u32 = 0x090B;
pub const SE_OP_ENTER_GAME: u32 = 0x082F;

pub const CULL_MIN: [f32; 3] = [-40.0, 0.0, -40.0];
pub const CULL_MAX: [f32; 3] = [40.0, 125.0, 40.0];

pub const FPC_NM_PLAY_SCENE: u16 = 7;
pub const FPC_NM_OPENING_SCENE: u16 = 8;
pub const FPC_NM_TITLE_SCENE: u16 = 10;
pub const FPC_NM_NAME_SCENE: u16 = 12;
pub const FPC_NM_OPEN2_SCENE: u16 = 15;
pub const FPC_NM_TITLE: u16 = 0x01C1;

pub const SCENE_REQ_CHANGE: u32 = 2;
pub const FADE_PEEK_TIME: u16 = 5;
pub const FADE_PROC_NAME: i16 = 0;
pub const FADE_PROC_OPENING: i16 = 1;

pub const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
pub const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
pub const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
pub const RND_SEED: i32 = 100;

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TitleKirakira {
    pub sound_flag: bool,
}

impl Default for TitleKirakira {
    fn default() -> Self {
        Self { sound_flag: true }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SceneRequest {
    pub req_type: u32,
    pub scene_proc: u16,
    pub user: u32,
    pub fade_proc: i16,
    pub fade_peek_time: u16,
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TitleScene {
    pub overlay_peek: bool,
    pub is_reset: bool,
    pub enable_next_stage: bool,
    pub room_control_proc: u16,
    pub request: Option<SceneRequest>,
}

impl Default for TitleScene {
    fn default() -> Self {
        Self {
            overlay_peek: false,
            is_reset: false,
            enable_next_stage: false,
            room_control_proc: FPC_NM_PLAY_SCENE,
            request: None,
        }
    }
}

impl TitleScene {
    pub fn change_req(&mut self, scene_proc: u16, fade_proc: i16) -> bool {
        if self.request.is_some() {
            return false;
        }
        self.request = Some(SceneRequest {
            req_type: SCENE_REQ_CHANGE,
            scene_proc,
            user: 0,
            fade_proc,
            fade_peek_time: FADE_PEEK_TIME,
        });
        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Anm {
    pub attribute: u8,
    pub state: u8,
    pub start: f32,
    pub end: f32,
    pub loop_point: f32,
    pub rate: f32,
    pub frame: f32,
}

impl Anm {
    pub fn init(frame_max: f32, attribute: u8) -> Self {
        let mut anm = Self {
            attribute: EMODE_LOOP,
            state: 0,
            start: 0.0,
            end: frame_max,
            loop_point: 0.0,
            rate: 1.0,
            frame: 0.0,
        };
        anm.attribute = attribute;
        anm.frame = anm.start;
        anm.loop_point = anm.frame;
        anm
    }

    pub fn set_frame(&mut self, frame: f32) {
        self.frame = frame;
    }

    pub fn set_play_speed(&mut self, rate: f32) {
        self.rate = rate;
    }

    pub fn end_frame(&self) -> f32 {
        self.end
    }

    pub fn play(&mut self) {
        self.state = 0;
        self.frame += self.rate;
        match self.attribute {
            EMODE_NONE => {
                if self.frame < self.start {
                    self.frame = self.start;
                    self.rate = 0.0;
                    self.state |= STATE_STOP;
                }
                if self.frame >= self.end {
                    self.frame = self.end - 0.001;
                    self.rate = 0.0;
                    self.state |= STATE_STOP;
                }
            }
            EMODE_RESET => {
                if self.frame < self.start {
                    self.frame = self.start;
                    self.rate = 0.0;
                    self.state |= STATE_STOP;
                }
                if self.frame >= self.end {
                    self.frame = self.start;
                    self.rate = 0.0;
                    self.state |= STATE_STOP;
                }
            }
            EMODE_LOOP => {
                while self.frame < self.start {
                    self.state |= STATE_LOOP;
                    if self.loop_point - self.start <= 0.0 {
                        break;
                    }
                    self.frame += self.loop_point - self.start;
                }
                while self.frame >= self.end {
                    self.state |= STATE_LOOP;
                    if self.end - self.loop_point <= 0.0 {
                        break;
                    }
                    self.frame -= self.end - self.loop_point;
                }
            }
            EMODE_REVERSE => {
                if self.frame >= self.end {
                    self.frame = self.end - 0.001;
                    self.rate = -self.rate;
                }
                if self.frame < self.start {
                    self.frame = self.start;
                    self.rate = 0.0;
                    self.state |= STATE_STOP;
                }
            }
            EMODE_LOOP_REVERSE => {
                while self.frame < self.start {
                    self.state |= STATE_LOOP;
                    if self.loop_point - self.start <= 0.0 {
                        break;
                    }
                    self.frame += self.loop_point - self.start;
                }
                while self.frame >= self.end {
                    self.state |= STATE_LOOP;
                    if self.end - self.loop_point <= 0.0 {
                        break;
                    }
                    self.frame -= self.end - self.loop_point;
                }
                if self.frame < self.start {
                    self.frame = self.end - 0.001;
                    self.rate = -self.rate;
                }
            }
            _ => {}
        }
    }

    pub fn is_stop(&self) -> bool {
        self.state & STATE_STOP != 0 || self.rate == 0.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pane {
    pub origin: [f32; 2],
    pub init_alpha: u8,
    pub now_alpha: u8,
    pub alpha: u8,
}

impl Pane {
    pub fn of(index: usize) -> Self {
        let origin = PANE_ORIGIN.get(index).copied().unwrap_or([0.0, 0.0]);
        let init_alpha = PANE_INIT_ALPHA.get(index).copied().unwrap_or(0);
        Self {
            origin,
            init_alpha,
            now_alpha: 0,
            alpha: 0,
        }
    }

    pub fn set_now_alpha(&mut self, value: f32) {
        self.now_alpha = (f32::from(self.init_alpha) * value) as u8;
    }

    pub fn set_alpha(&mut self) {
        self.alpha = self.now_alpha;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Emitter {
    pub live: bool,
    pub id: u32,
    pub pos: Vec3,
    pub angle: [i16; 3],
}

impl Emitter {
    pub const fn none() -> Self {
        Self {
            live: false,
            id: 0,
            pos: Vec3::ZERO,
            angle: [0; 3],
        }
    }

    pub fn spawn(id: u32, pos: Vec3) -> Self {
        Self {
            live: true,
            id,
            pos,
            angle: [0; 3],
        }
    }

    pub fn spawn_angled(id: u32, pos: Vec3, angle: [i16; 3]) -> Self {
        Self {
            live: true,
            id,
            pos,
            angle,
        }
    }

    pub fn translate(&mut self, pos: Vec3) {
        self.pos = pos;
    }

    pub fn become_invalid(&mut self) {
        self.live = false;
        self.id = 0;
        self.pos = Vec3::ZERO;
        self.angle = [0; 3];
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Title {
    pub created: bool,
    pub m00c: Vec3,
    pub m018: i32,
    pub m01c: i32,
    pub m020: i32,
    pub m024: i32,
    pub m028: i32,
    pub m02c: i32,
    pub m090: u8,
    pub m094: f32,
    pub m098: i32,
    pub enter_mode: i32,
    pub m29c: bool,
    pub emitter: Emitter,
    pub emitter2: Emitter,
    pub wind: Emitter,
    pub bck_ship: Anm,
    pub bpk_ship: Anm,
    pub btk_sub: Anm,
    pub btk_kirari: Anm,
    pub pane: [Pane; PANE_COUNT],
    pub ship_base: Vec3,
    pub ship_scale: Vec3,
    pub ship_rotate_y: i16,
    pub subtitle_base: Vec3,
    pub subtitle_scale: Vec3,
    pub subtitle_rotate_y: i16,
    pub kirari_base: Vec3,
    pub seed: [i32; 3],
}

impl Title {
    pub fn spawn(parameters: u32) -> Self {
        let _ = parameters;
        Self {
            created: false,
            m00c: Vec3::ZERO,
            m018: 0,
            m01c: START_DELAY,
            m020: 0,
            m024: 0,
            m028: 0,
            m02c: 0,
            m090: 0,
            m094: 0.0,
            m098: SMOKE_RISE,
            enter_mode: 0,
            m29c: false,
            emitter: Emitter::none(),
            emitter2: Emitter::none(),
            wind: Emitter::none(),
            bck_ship: Anm::init(BCK_TITLE_SHIP_FRAME_MAX, EMODE_LOOP),
            bpk_ship: Anm::init(BPK_TITLE_SHIP_FRAME_MAX, EMODE_LOOP),
            btk_sub: Anm::init(BTK_SUBTITLE_START_FRAME_MAX, EMODE_NONE),
            btk_kirari: Anm::init(BTK_SUBTITLE_KIRARI_FRAME_MAX, EMODE_LOOP),
            pane: [
                Pane::of(PANE_ZELD),
                Pane::of(PANE_ZELJ),
                Pane::of(PANE_PRES),
                Pane::of(PANE_NINT),
                Pane::of(PANE_EFT1),
                Pane::of(PANE_EFT2),
            ],
            ship_base: Vec3::ZERO,
            ship_scale: Vec3::ZERO,
            ship_rotate_y: SHIP_ROTATE_Y,
            subtitle_base: Vec3::ZERO,
            subtitle_scale: Vec3::ZERO,
            subtitle_rotate_y: SUBTITLE_ROTATE_Y,
            kirari_base: Vec3::ZERO,
            seed: [RND_SEED; 3],
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        self.created
    }

    pub fn rnd(&mut self) -> f32 {
        for lane in 0..3 {
            self.seed[lane] = (self.seed[lane] * RND_MUL[lane]) % RND_MOD[lane];
        }
        let sum = (self.seed[0]) as f32 / RND_SCALE[0]
            + (self.seed[1]) as f32 / RND_SCALE[1]
            + (self.seed[2]) as f32 / RND_SCALE[2];
        (sum % 1.0).abs()
    }

    pub fn rnd_f(&mut self, range: f32) -> f32 {
        self.rnd() * range
    }

    pub fn rnd_fx(&mut self, range: f32) -> f32 {
        range * (self.rnd() - 0.5) * 2.0
    }

    pub fn enter_mode(&self) -> i32 {
        self.enter_mode
    }

    pub fn set_enter_mode(&mut self) {
        if self.enter_mode == 1 {
            self.enter_mode = 2;
        }
    }

    pub fn proc_init2d(&mut self) {
        for pane in self.pane.iter_mut() {
            pane.set_now_alpha(0.0);
            pane.set_alpha();
        }
    }

    pub fn proc_init3d(&mut self) {
        self.bpk_ship.set_frame(0.0);
        self.bpk_ship.set_play_speed(1.0);
    }

    pub fn create(&mut self) -> bool {
        self.m01c = START_DELAY;
        self.enter_mode = 0;
        self.m098 = SMOKE_RISE;
        self.m094 = (self.m098 * self.m098) as f32 * -ATTR_0X0C;
        self.m020 = (self.rnd_f(ATTR_0X28) + ATTR_0X2C) as i32;
        self.m024 = (self.rnd_f(ATTR_0X20) + ATTR_0X24 + KIRARI_BASE_CSUM) as i32;
        self.m02c = 0;
        self.m018 = 0;
        self.m090 = 0;
        self.emitter2 = Emitter::none();
        self.emitter = Emitter::none();
        self.proc_init2d();
        self.proc_init3d();
        self.created = true;
        self.m29c = false;
        true
    }

    pub fn set_mtx(&mut self) {
        self.ship_base = Vec3::new(self.m094 + ATTR_0X00, ATTR_0X04, SHIP_Z);
        self.ship_scale = Vec3::splat(ATTR_0X08);
        self.ship_rotate_y = SHIP_ROTATE_Y;
        self.subtitle_base = Vec3::new(ATTR_0X10, ATTR_0X14, SUBTITLE_Z);
        self.kirari_base = Vec3::new(ATTR_0X10, ATTR_0X14, KIRARI_Z);
        self.subtitle_scale = Vec3::new(ATTR_0X18, ATTR_0X1C, 1.0);
        self.subtitle_rotate_y = SUBTITLE_ROTATE_Y;
    }

    fn ship_frame(&mut self) {
        self.m094 = if self.m098 <= 0 {
            (self.m098 * self.m098) as f32 * -ATTR_0X0C
        } else {
            (self.m098 * self.m098) as f32 * ATTR_0X0C
        };
        self.bpk_ship.set_frame(if self.m098 <= 0 {
            SHIP_FRAME_BASE + (self.m098 * SHIP_FRAME_STEP) as f32
        } else {
            SHIP_FRAME_BASE - (self.m098 * SHIP_FRAME_STEP) as f32
        });
    }

    fn eft1(&self) -> [f32; 2] {
        self.pane[PANE_EFT1].origin
    }

    fn eft2(&self) -> [f32; 2] {
        self.pane[PANE_EFT2].origin
    }

    fn smoke_offset(&self) -> Vec3 {
        let origin = self.eft1();
        Vec3::new(
            (origin[0] - SCREEN_HALF_W) - self.m094 + ATTR_0X30,
            (origin[1] - SCREEN_HALF_H) + ATTR_0X34,
            0.0,
        )
    }

    fn smoke_plain(&self) -> Vec3 {
        let origin = self.eft1();
        Vec3::new(origin[0] - SCREEN_HALF_W, origin[1] - SCREEN_HALF_H, 0.0)
    }

    pub fn calc_2d_alpha(&mut self, kirakira: &mut TitleKirakira, events: &mut SimEvents) {
        self.m018 += 1;
        if self.m018 >= ENTER_FRAME && self.enter_mode == 0 {
            self.enter_mode = 1;
        }

        if self.enter_mode == 0 {
            if self.m098 < 0 {
                self.m098 += 1;
            }

            if !self.emitter.live {
                self.emitter = Emitter::spawn(PA_TITLESMOKE, self.smoke_offset());
            } else {
                self.emitter.translate(self.smoke_offset());
            }

            if self.m018 <= ZELD_HOLD {
                self.pane[PANE_ZELD].set_now_alpha(0.0);
            } else if self.m018 <= ZELD_RAMP {
                self.pane[PANE_ZELD].set_now_alpha((self.m018 - ZELD_HOLD) as f32 / ZELD_SPAN);
            } else {
                self.pane[PANE_ZELD].set_now_alpha(1.0);
            }

            self.pane[PANE_ZELJ].set_now_alpha(0.0);

            if self.m018 == KIRAKIRA_FRAME {
                if kirakira.sound_flag {
                    events.list.push(SimEvent::Se(SE_TITLE_KIRA));
                    kirakira.sound_flag = false;
                }
                self.m00c = self.smoke_plain();
                self.emitter2 = Emitter::spawn(PA_TITLEKIRAKIRA, self.m00c);
            } else if self.m018 > KIRAKIRA_FRAME && self.m018 <= KIRAKIRA_LAST && self.emitter2.live
            {
                let eft1 = self.eft1();
                let eft2 = self.eft2();
                self.m00c.x += (eft2[0] - eft1[0]) / KIRAKIRA_STEP;
                self.emitter2.translate(self.m00c);
            }

            if self.m018 >= KIRAKIRA_FRAME {
                self.btk_sub.play();
            }

            if self.m018 <= NINT_HOLD {
                self.pane[PANE_NINT].set_now_alpha(0.0);
            } else if self.m018 <= NINT_RAMP {
                self.pane[PANE_NINT].set_now_alpha((self.m018 - NINT_HOLD) as f32 / NINT_SPAN);
            } else {
                self.pane[PANE_NINT].set_now_alpha(1.0);
            }

            if self.m018 <= PRES_HOLD {
                self.pane[PANE_PRES].set_now_alpha(0.0);
            } else if self.m018 <= PRES_RAMP {
                self.pane[PANE_PRES].set_now_alpha((self.m018 - PRES_HOLD) as f32 / PRES_SPAN);
            } else {
                self.pane[PANE_PRES].set_now_alpha(1.0);
            }
        } else {
            if !self.emitter.live {
                self.emitter = Emitter::spawn(PA_TITLESMOKE, self.smoke_plain());
            } else {
                self.emitter.translate(self.smoke_offset());
            }

            self.pane[PANE_ZELD].set_now_alpha(1.0);
            self.pane[PANE_ZELJ].set_now_alpha(0.0);

            if self.emitter2.live {
                self.emitter2.become_invalid();
            }

            self.btk_sub.set_frame(self.btk_sub.end_frame());
            self.pane[PANE_NINT].set_now_alpha(1.0);
            if self.m028 >= PRES_PERIOD {
                self.m028 = 0;
            } else {
                self.m028 += 1;
            }

            if self.m028 >= PRES_HALF {
                self.pane[PANE_PRES].set_now_alpha((self.m028 - PRES_HALF) as f32 / PRES_SPAN_HALF);
            } else {
                self.pane[PANE_PRES].set_now_alpha((PRES_HALF - self.m028) as f32 / PRES_SPAN_HALF);
            }
        }

        if self.m020 == 0 {
            self.m020 = (self.rnd_f(WIND_FRAME) + WIND_BASE) as i32;
            let angle_x = self.rnd_fx(WIND_YAW) as i16;
            let angle_z = self.rnd_fx(WIND_TILT) as i16;
            self.wind =
                Emitter::spawn_angled(PA_TITLEWIND, self.smoke_plain(), [angle_x, 0, angle_z]);
        } else {
            self.m020 -= 1;
        }

        if self.m024 == 0 {
            self.m02c += 1;
            if self.m02c <= KIRARI_SPAN {
                self.btk_kirari.set_frame(self.m02c as f32);
            } else {
                self.m02c = 0;
                self.btk_kirari.set_frame(0.0);
                self.m024 = (self.rnd_f(KIRARI_FRAME) + KIRARI_BASE) as i32;
            }
        } else {
            self.m024 -= 1;
        }

        self.ship_frame();
    }

    pub fn proc_execute(
        &mut self,
        pressed: bool,
        kirakira: &mut TitleKirakira,
        events: &mut SimEvents,
    ) {
        if self.m01c > 0 {
            self.m01c -= 1;
            if self.m01c == 0 {
                events.list.push(SimEvent::Se(SE_TITLE_WIND));
            }
        } else {
            self.calc_2d_alpha(kirakira, events);
        }

        if pressed && self.enter_mode == 0 {
            self.enter_mode = 1;
            self.m01c = 0;
            self.m098 = 0;
        }

        for pane in self.pane.iter_mut().take(ALPHA_SET_COUNT) {
            pane.set_alpha();
        }

        if self.enter_mode == 2 {
            self.enter_mode = 3;
        } else if self.enter_mode == 3 {
            self.m098 += 1;
        }

        self.bck_ship.play();
        self.set_mtx();
    }

    pub fn execute(
        &mut self,
        pressed: bool,
        scene: &mut TitleScene,
        kirakira: &mut TitleKirakira,
        events: &mut SimEvents,
    ) {
        if !scene.overlay_peek {
            if pressed && self.enter_mode() == 1 {
                self.set_enter_mode();
            } else if self.enter_mode() == 3 {
                if !self.m29c && scene.change_req(FPC_NM_NAME_SCENE, FADE_PROC_NAME) {
                    events.list.push(SimEvent::Se(SE_OP_ENTER_GAME));
                    self.m29c = true;
                }
            } else if !scene.is_reset && scene.enable_next_stage {
                if !self.m29c {
                    let proc_name = if scene.room_control_proc == FPC_NM_OPENING_SCENE {
                        FPC_NM_OPEN2_SCENE
                    } else {
                        FPC_NM_TITLE_SCENE
                    };
                    scene.change_req(proc_name, FADE_PROC_OPENING);
                    self.m29c = true;
                }
            }
        }

        self.proc_execute(pressed, kirakira, events);
    }
}

pub fn control(
    pad: Option<Res<PadButtons>>,
    mut kirakira: ResMut<TitleKirakira>,
    mut scene: ResMut<TitleScene>,
    mut events: ResMut<SimEvents>,
    mut titles: Query<&mut Title, (With<Title>, Without<Player>)>,
) {
    let (trig_a, trig_b, trig_start) = pad.map_or((false, false, false), |pad| {
        (pad.trig.a, pad.trig.b, pad.trig.start)
    });
    let pressed = trig_a || trig_b || trig_start;

    for mut title in &mut titles {
        title.execute(pressed, &mut scene, &mut kirakira, &mut events);
    }
}
