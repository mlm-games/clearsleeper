use std::collections::HashMap;

use crate::assets::AssetRoot;
use crate::j2d::Screen;
use crate::panes::{self, Assets};
use crate::text::raster::Canvas;
use crate::ui_menu_save::{GAME_OVER_HOLD, GAME_OVER_TAIL, Pane, Particles, SaveEnv, SaveMenu};

pub const ARCHIVE: &str = "files/res/Object/Gover.arc";
pub const SCREEN_FILE: &str = "gameover.blo";
pub const LETTER_TAGS: [&str; 8] = ["txg", "txa", "txm", "txe1", "txo", "txv", "txe2", "txr"];
pub const LETTER_X: [f32; 8] = [-175.0, -130.0, -80.0, -30.0, 32.0, 75.0, 120.0, 160.0];
pub const ROTATE_TABLE: [i16; 6] = [40, 30, 20, -20, -30, -40];
pub const BACKDROP_TAG: &str = "blak";

pub const STATE_SAVE_OPEN: u8 = 3;
pub const STATE_SAVE_MOVE: u8 = 4;
pub const STATE_SAVE_CLOSE: u8 = 5;
pub const STATE_DONE: u8 = 6;

pub const PARTICLE_OPEN_MENU: u16 = 0x2E;
pub const PARTICLE_SMOKE: u16 = 0x2F;
pub const PARTICLE_HA: u16 = 0x30;
pub const SE_EXIT_GAME_OVER: u32 = 0x085A;
pub const LIFE_REFILL: u8 = 12;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Continue,
    Title,
}

pub struct GameOverAssets {
    pub assets: Assets,
}

impl GameOverAssets {
    pub fn load(root: &AssetRoot) -> Option<Self> {
        Assets::load_from(root, ARCHIVE, None, SCREEN_FILE).map(|assets| Self { assets })
    }
}

/// `cM_rnd`'s three lanes (`c_math.cpp:177-182`), seeded the way the machine
/// starts them (`m_Do_machine.cpp:617`).
#[derive(Clone, Debug)]
pub struct Rng {
    pub seed: [i32; 3],
}

impl Default for Rng {
    fn default() -> Self {
        Self { seed: [100; 3] }
    }
}

impl Rng {
    pub fn next(&mut self) -> f32 {
        const MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
        const MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
        const SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
        for lane in 0..3 {
            self.seed[lane] = (self.seed[lane].wrapping_mul(MUL[lane])) % MOD[lane];
        }
        let sum = self.seed[0] as f32 / SCALE[0]
            + self.seed[1] as f32 / SCALE[1]
            + self.seed[2] as f32 / SCALE[2];
        (sum % 1.0).abs()
    }

    pub fn next_f(&mut self, range: f32) -> f32 {
        self.next() * range
    }
}

/// `dDlst_GameOverScrnDraw_c`: the eight letter panes and the `blak` backdrop,
/// with the animation counters `field_0x32c`/`field_0x330` (`d_gameover.h:29-40`).
pub struct Letters {
    pub letter: [Pane; 8],
    pub backdrop: Pane,
    pub count: i32,
    pub delay: i32,
    pub alpha: f32,
    pub rotate: [i16; 8],
    pub rng: Rng,
    pub sounds: Vec<u32>,
}

impl Default for Letters {
    fn default() -> Self {
        Self {
            letter: std::array::from_fn(|index| Pane::new(LETTER_TAGS[index])),
            backdrop: Pane::new(BACKDROP_TAG),
            count: 8,
            delay: 0,
            alpha: 0.0,
            rotate: [0; 8],
            rng: Rng::default(),
            sounds: Vec::new(),
        }
    }
}

impl Letters {
    /// `setScreen`, `d_gameover.cpp:206-225`: the tags are the eight letters,
    /// and `blak` starts at full init alpha so the draw's `mAlpha` scales it.
    pub fn create(&mut self, screen: &Screen) {
        for index in 0..8 {
            if let Some(layout) = screen.find(LETTER_TAGS[index]) {
                self.letter[index].parts_set_file(layout);
            }
        }
        if let Some(layout) = screen.find(BACKDROP_TAG) {
            self.backdrop.parts_set_file(layout);
            self.backdrop.init_alpha = 0xFF;
        }
    }

    /// `setBackAlpha`/`fopMsgM_setNowAlpha(&blak, mAlpha)`, `d_gameover.h:73`
    /// and `d_gameover.cpp:417`.
    pub fn set_back_alpha(&mut self, alpha: f32) {
        self.alpha = alpha.clamp(0.0, 1.0);
        self.backdrop.set_now_alpha(self.alpha);
    }

    /// `valueInit`, `d_gameover.cpp:227-233`.
    pub fn value_init(&mut self) {
        self.count = 8;
        self.delay = 0;
        for pane in &mut self.letter {
            pane.user_area = 0;
        }
    }

    /// `animeOpen`, `d_gameover.cpp:236-286`.
    pub fn anime_open(&mut self, particles: &mut Particles) -> bool {
        let mut active = false;
        let mut started: i16 = -1;

        if self.count >= 0 {
            if self.delay != 0 {
                self.delay -= 1;
            } else {
                let mut rand = self.rng.next_f(self.count as f32) as i32;
                if rand >= self.count - 1 {
                    rand = self.count - 1;
                }
                for index in 0..8 {
                    if self.letter[index].user_area == 0 {
                        if rand == 0 {
                            self.anime1(index);
                            started = index as i16;
                            break;
                        }
                        rand -= 1;
                    }
                }
                self.count -= 1;
                self.delay = (self.count * self.count) / 5;
            }
        }

        for index in 0..8 {
            if (0..9).contains(&self.letter[index].user_area) {
                if self.letter[index].user_area >= 1 && started != index as i16 {
                    self.anime1(index);
                    if self.letter[index].user_area == 5 {
                        particles.set_2d_fore(PARTICLE_OPEN_MENU, LETTER_X[index], 50.0, 0.0);
                    }
                }
                active = true;
            }
        }

        !active
    }

    /// `animeClose`, `d_gameover.cpp:289-335`.
    pub fn anime_close(&mut self) -> bool {
        let mut active = false;
        let mut started: i16 = -1;

        if self.count >= 0 {
            if self.delay != 0 {
                self.delay = 0;
            } else {
                let mut rand = self.rng.next_f(self.count as f32) as i32;
                if rand >= self.count - 1 {
                    rand = self.count - 1;
                }
                for index in 0..8 {
                    if self.letter[index].user_area == 0 {
                        if rand == 0 {
                            self.anime2(index);
                            started = index as i16;
                            break;
                        }
                        rand -= 1;
                    }
                }
                self.delay = self.rng.next_f(3.0) as i32 + 1;
                if self.delay >= 3 {
                    self.delay = 3;
                }
                self.count -= 1;
            }
        }

        for index in 0..8 {
            if (0..7).contains(&self.letter[index].user_area) {
                if self.letter[index].user_area >= 1 && started != index as i16 {
                    self.anime2(index);
                }
                active = true;
            }
        }

        !active
    }

    /// `anime1`, `d_gameover.cpp:348-371`: the pre-ramp drops from -288 to 0,
    /// the two-step dip to -9, then the settle.
    pub fn anime1(&mut self, index: usize) -> bool {
        let pane = &mut self.letter[index];
        if pane.user_area < 5 {
            if pane.user_area == 0 {
                pane.set_init_alpha();
            }
            pane.user_area += 1;
            pane.pane_trans(0.0, (1.0 - acc(5, pane.user_area, 0)) * -288.0);
        } else if pane.user_area < 7 {
            pane.user_area += 1;
            pane.pane_trans(0.0, acc(2, pane.user_area - 5, 0) * -9.0);
        } else if pane.user_area < 9 {
            pane.user_area += 1;
            pane.pane_trans(0.0, (1.0 - acc(2, pane.user_area - 7, 0)) * -9.0);
            if pane.user_area == 9 {
                self.sounds.push(SE_EXIT_GAME_OVER);
            }
        }
        pane.user_area == 9
    }

    /// `anime2`, `d_gameover.cpp:374-406`: the letter falls 244 down while its
    /// random tilt unwinds, and vanishes on the seventh step.
    pub fn anime2(&mut self, index: usize) -> bool {
        let pane = &mut self.letter[index];
        if pane.user_area < 7 {
            if pane.user_area == 0 {
                let rand = (self.rng.next_f(6.0) as i32).min(5);
                self.rotate[index] = ROTATE_TABLE[rand as usize];
            }
            pane.user_area += 1;
            let step = f32::from(pane.user_area);
            pane.rotate(
                pane.size.x / 2.0,
                pane.size.y / 2.0,
                f32::from(self.rotate[index]) * (1.0 - step / 7.0),
            );
            pane.pane_trans(0.0, step * step / 49.0 * 244.0);
        }
        if pane.user_area == 7 {
            pane.set_now_alpha_zero();
            true
        } else {
            false
        }
    }
}

fn acc(max: i16, value: i16, base: i16) -> f32 {
    let span = f32::from(max - base);
    if span == 0.0 {
        return 0.0;
    }
    let value = f32::from(value - base);
    value * value / (span * span)
}

pub struct GameOver {
    pub letters: Letters,
    pub save: SaveMenu,
    pub emitters: [Option<usize>; 2],
    pub hold: i16,
    pub tail: i16,
    pub state: u8,
    pub anime_start: bool,
    pub menu_open: bool,
    pub status: u8,
    pub reset: bool,
}

impl Default for GameOver {
    fn default() -> Self {
        Self {
            letters: Letters::default(),
            save: SaveMenu::default(),
            emitters: [None, None],
            hold: GAME_OVER_HOLD,
            tail: GAME_OVER_TAIL,
            state: 0,
            anime_start: false,
            menu_open: false,
            status: 0,
            reset: false,
        }
    }
}

impl GameOver {
    /// `_create`, `d_gameover.cpp:67-105`, and `setUseType(2)` at `:90-91`.
    pub fn create(&mut self, screen: &Screen) {
        self.letters.create(screen);
        self.letters.value_init();
        self.save.set_use_type(2);
        self.hold = GAME_OVER_HOLD;
        self.tail = GAME_OVER_TAIL;
        self.anime_start = false;
    }

    /// `dMenu_save_c::_create` (`d_menu_save.cpp:49-73`) binds `menu_save.blo`
    /// from `saveres.arc`; the error panel's `errorres.arc` is not ported, so
    /// an empty screen keeps its binding path alive without drawing it.
    pub fn create_save(&mut self, screen: Screen) {
        let error = Screen {
            width: 640,
            height: 480,
            color: 0,
            panes: Vec::new(),
        };
        self.save.create(screen, error);
    }

    pub fn anime_start(&mut self) {
        self.anime_start = true;
    }

    pub fn set_emitter0(&mut self, particles: &mut Particles, x: f32, y: f32, z: f32) {
        self.emitters[0] = Some(particles.set_2d_fore(PARTICLE_SMOKE, x, y, z));
    }

    pub fn set_emitter1(&mut self, particles: &mut Particles, x: f32, y: f32, z: f32) {
        self.emitters[1] = Some(particles.set_2d_menu_fore(PARTICLE_HA, x, y, z));
    }

    fn stop_emitter0(&mut self, particles: &mut Particles) {
        if let Some(index) = self.emitters[0] {
            particles.stop(index);
        }
    }

    fn stop_emitter1(&mut self, particles: &mut Particles) {
        if let Some(index) = self.emitters[1] {
            particles.stop(index);
        }
    }

    /// `dGameover_c::_execute`, `d_gameover.cpp:109-166`. The letter branch is
    /// `:141-161`, the open is `:142-148`, the 90 frame hold `:149-152`, the
    /// close and `field_0x116` tail `:153-160`.
    pub fn step(&mut self, env: &mut SaveEnv<'_>) -> Option<Outcome> {
        if self.state == STATE_SAVE_OPEN {
            // `:118-121`: open, then set life to 12.
            if self.save.open(env) {
                self.state = STATE_SAVE_MOVE;
                let life = env.health.max.min(LIFE_REFILL);
                env.health.units = life;
            }
        } else if self.state == STATE_SAVE_MOVE {
            // `:123-126`: move, then finish when the save menu reports status 3.
            self.save.move_step(env);
            if env.system.is_reset() {
                return Some(Outcome::Title);
            }
            if self.save.get_save_status() == 3 {
                self.state = STATE_SAVE_CLOSE;
            }
        } else if self.state == STATE_SAVE_CLOSE {
            // `:127-140`: close, then end status 0 resets to the title and 1 continues.
            if self.save.close(env) {
                let outcome = match self.save.get_end_status() {
                    0 => {
                        self.status = 3;
                        self.menu_open = false;
                        self.reset = true;
                        Some(Outcome::Title)
                    }
                    1 => {
                        self.status = 2;
                        self.menu_open = false;
                        Some(Outcome::Continue)
                    }
                    _ => None,
                };
                self.stop_emitter1(env.particles);
                self.state = STATE_DONE;
                return outcome;
            }
        } else if self.state != STATE_DONE && self.anime_start {
            if self.hold == GAME_OVER_HOLD {
                if self.letters.anime_open(env.particles) {
                    self.set_emitter0(env.particles, 0.0, 0.0, 0.0);
                    self.set_emitter1(env.particles, 0.0, 0.0, 0.0);
                    self.hold -= 1;
                }
            } else if self.hold > 0 {
                self.hold -= 1;
                if self.hold == 0 {
                    self.letters.value_init();
                }
            } else if self.letters.anime_close() {
                // `:154-159`: `field_0x116-- <= 0` is the check before the decrement.
                self.stop_emitter0(env.particles);
                if self.tail <= 0 {
                    self.state = STATE_SAVE_OPEN;
                    self.menu_open = true;
                } else {
                    self.tail -= 1;
                }
            }
        }
        None
    }
}

/// `dDlst_GameOverScrnDraw_c::draw`, `d_gameover.cpp:413-423`: every letter at
/// its own `mNowAlpha`, then `blak` at `mInitAlpha * mAlpha`, all through the
/// screen's own draw order. The live view stays behind this transparent layer;
/// the retail framebuffer capture (`:13-65`) is not reproduced.
pub fn render(canvas: &mut Canvas, assets: &GameOverAssets, state: &GameOver) {
    canvas.pixels.fill(0);
    let mut live: HashMap<&str, &Pane> = HashMap::new();
    for pane in &state.letters.letter {
        live.insert(pane.tag, pane);
    }
    live.insert(state.letters.backdrop.tag, &state.letters.backdrop);
    panes::draw_screen(canvas, &assets.assets, &live);
}
