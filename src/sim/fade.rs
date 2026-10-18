//! The two screen overlays the event and demo machinery drive: the graphic layer's fade
//! (`mDoGph_gInf_c::fadeOut` / `calcFade`, `m_Do_graphic.cpp:153-215`) and the draw list's wipe
//! (`dDlst_list_c::wipeIn` / `wipeOut` / `calcWipe`, `d_drawlist.cpp:2044-2085`).
//!
//! Both are pure numbers here. The sim owns the rate, the speed and the colour and advances them
//! once per fixed step the way the reference advances them once per frame; the shell is what turns
//! them into pixels, which is why neither carries a renderer. `Fade::rate` is the number the
//! DIRECTOR staff's `FADE` action polls (`d_event_data.cpp:718-727`) and `Wipe::rate` the number
//! its `WIPE` action polls (`:741-751`), so the cuts end on the same tick they do in retail.
//!
//! `calcFade`'s brightness tail (`m_Do_graphic.cpp:198-206`) reads `dComIfG_getBrightness`, which
//! this port does not carry: with no fade active the colour is left at rgb 0 / alpha 0 rather than
//! invented from a brightness the sim never holds.

use bevy_ecs::prelude::*;

/// `GXColor`.
pub type Color = [u8; 4];

/// `g_clearColor` (`d_com_inf_game.cpp:29`), the colour `fadeOut(speed)`'s one-argument form
/// passes (`m_Do_graphic.cpp:175-176`).
pub const CLEAR_COLOR: Color = [0x00, 0x00, 0x00, 0x00];
/// `g_blackColor` (`d_com_inf_game.cpp:30`).
pub const BLACK_COLOR: Color = [0x00, 0x00, 0x00, 0xFF];
/// `g_saftyWhiteColor` (`d_com_inf_game.cpp:32`), the "safety white" the knob door's night fade
/// picks (`d_a_knob00.cpp:249`).
pub const SAFTY_WHITE_COLOR: Color = [0xA0, 0xA0, 0xA0, 0xFF];
/// `dDlst_list_c::mWipeColor`'s initial value (`d_drawlist.cpp:2044`).
const WIPE_COLOR: Color = [0x00, 0x00, 0x00, 0x00];

/// `mDoGph_gInf_c::mFade`/`mFadeRate`/`mFadeSpeed`/`mFadeColor` (`m_Do_graphic.cpp:46-48`, `:78`).
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Fade {
    active: bool,
    rate: f32,
    speed: f32,
    color: Color,
}

impl Default for Fade {
    /// `mDoGph_gInf_c::create`'s two stores: `mFade = 0` and `mFadeColor = g_clearColor`
    /// (`m_Do_graphic.cpp:120-122`).
    fn default() -> Self {
        Self {
            active: false,
            rate: 0.0,
            speed: 0.0,
            color: CLEAR_COLOR,
        }
    }
}

impl Fade {
    /// `mDoGph_gInf_c::fadeOut(f32 speed, GXColor& color)` - `m_Do_graphic.cpp:154-158`. A positive
    /// speed clears the screen from transparent to the colour, a negative one from the colour back
    /// to transparent.
    pub fn fade_out(&mut self, speed: f32, color: Color) {
        self.active = true;
        self.speed = speed;
        self.color = color;
        self.rate = if speed >= 0.0 { 0.0 } else { 1.0 };
    }

    /// `mDoGph_gInf_c::fadeIn(f32 rate, GXColor& color)` - `m_Do_graphic.h:86`, an inline that
    /// negates the rate.
    pub fn fade_in(&mut self, rate: f32, color: Color) {
        self.fade_out(-rate, color);
    }

    /// `mDoGph_gInf_c::setFadeRate(f32)` - `m_Do_graphic.h:78`.
    pub fn set_rate(&mut self, rate: f32) {
        self.rate = rate;
    }

    /// `mDoGph_gInf_c::offFade()` - `m_Do_graphic.h:60`.
    pub fn off(&mut self) {
        self.active = false;
    }

    /// `mDoGph_gInf_c::isFade()` - `m_Do_graphic.h:61`, the `u8` the FADE cut's move arm tests
    /// (`d_event_data.cpp:720`).
    pub fn is_fade(&self) -> bool {
        self.active
    }

    /// `mDoGph_gInf_c::getFadeRate()` - `m_Do_graphic.h:79`.
    pub fn rate(&self) -> f32 {
        self.rate
    }

    /// `mDoGph_gInf_c::getFadeColor()` - `m_Do_graphic.h:80`.
    pub fn color(&self) -> Color {
        self.color
    }

    /// `mDoGph_gInf_c::calcFade` - `m_Do_graphic.cpp:181-215`, the rate integrator and the one
    /// place the fade colour's alpha is written. The `< 0` side clears `mFade` and the `> 1` side
    /// does not, which is the reference's own asymmetry and the reason a fade-out ends its cut on
    /// `getFadeRate() >= 1.0f` (`d_event_data.cpp:722`) while a fade-in ends it on `!isFade()`.
    pub fn calc(&mut self) {
        if self.active {
            self.rate += self.speed;
            if self.rate < 0.0 {
                self.rate = 0.0;
                self.active = false;
            } else if self.rate > 1.0 {
                self.rate = 1.0;
            }
            self.color[3] = (self.rate * 255.0) as u8;
        } else {
            self.color = [0, 0, 0, 0];
        }
    }
}

/// `dDlst_list_c::mWipe`/`mWipeRate`/`mWipeSpeed`/`mWipeColor` (`d_drawlist.cpp:2043-2046`).
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Wipe {
    active: bool,
    rate: f32,
    speed: f32,
    color: Color,
}

impl Default for Wipe {
    fn default() -> Self {
        Self {
            active: false,
            rate: 0.0,
            speed: 0.0,
            color: WIPE_COLOR,
        }
    }
}

impl Wipe {
    /// `dDlst_list_c::wipeIn(f32 speed, GXColor& color)` - `d_drawlist.cpp:2051-2065`.
    pub fn wipe_in(&mut self, speed: f32, color: Color) {
        self.active = true;
        self.speed = speed;
        self.color = color;
        self.rate = if speed >= 0.0 { 0.0 } else { 1.0 };
    }

    /// `dDlst_list_c::wipeOut(f32 rate, GXColor& color)` - `d_drawlist.h:525`, the inline the
    /// DIRECTOR staff's `WIPE` action calls with its own `Color` key (`d_event_data.cpp:688`).
    pub fn wipe_out(&mut self, rate: f32, color: Color) {
        self.wipe_in(-rate, color);
    }

    /// `dDlst_list_c::wipeOut(f32 rate)` - `d_drawlist.h:524`: `wipeIn(-rate)`, which the
    /// one-argument `wipeIn` (`d_drawlist.cpp:2065`) resolves to the two-argument form with
    /// `g_blackColor`.
    pub fn wipe_out_default(&mut self, rate: f32) {
        self.wipe_in(rate, BLACK_COLOR);
    }

    /// `dDlst_list_c::getWipeRate()` - `d_drawlist.h:643`.
    pub fn rate(&self) -> f32 {
        self.rate
    }

    /// `dDlst_list_c::mWipe`.
    pub fn is_wipe(&self) -> bool {
        self.active
    }

    pub fn color(&self) -> Color {
        self.color
    }

    /// `dDlst_list_c::calcWipe` - `d_drawlist.cpp:2069-2083`. As in `calcFade`, only the `> 1` side
    /// clears the flag; a wipe pulled back down to 0 stays live at rate 0.
    pub fn calc(&mut self) {
        if !self.active {
            return;
        }
        self.rate += self.speed;
        if self.rate < 0.0 {
            self.rate = 0.0;
        } else if self.rate > 1.0 {
            self.rate = 1.0;
            self.active = false;
        }
    }
}

/// `mDoGph_gInf_c::calcFade`, `dDlst_list_c::calcWipe`, `JUTFader::control` and
/// `mDoGph_gInf_c::calcMonotone`, once per fixed step.
///
/// `dScnPly_Execute` runs the event manager and then the frame's draw, whose `mDoGph_Painter`
/// calls `calcFade` (`m_Do_graphic.cpp:181`) - so the fade, the wipe and the monotone chase advance
/// *after* the cuts that read them on any given tick, which is why this system sits behind
/// `event::control` in the `Execute` chain.
pub fn control(
    mut fade: ResMut<Fade>,
    mut wipe: ResMut<Wipe>,
    mut fader: ResMut<Fader>,
    mut monotone: ResMut<Monotone>,
) {
    fade.calc();
    wipe.calc();
    fader.control();
    monotone.calc();
}

/// `JUTFader` - `JUTFader.h:10-36`, `JUTFader.cpp:9-89`. This is the *other* fade: the display's
/// whole-screen fader, driven by a frame count, which `daDemo00_c`'s parameter ids 9 and 10 start
/// (`d_a_demo00.cpp:776-783`). It is not the `mDoGph_gInf_c` rate fade above, and the two are kept
/// apart for that reason. The box and the drawing are the shell's.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Fader {
    status: i32,
    fade_time: u16,
    timer: u16,
    color: Color,
    delay_timer: i32,
    delay_status: i32,
}

/// `JUTFader::EStatus` - `JUTFader.h:11-16`.
pub const FADER_WAIT_OUT: i32 = 0;
pub const FADER_WAIT_IN: i32 = 1;
pub const FADER_FADE_IN: i32 = 2;
pub const FADER_FADE_OUT: i32 = 3;

impl Default for Fader {
    /// `JUTFader::JUTFader`'s five stores (`JUTFader.cpp:10-15`).
    fn default() -> Self {
        Self {
            status: FADER_WAIT_OUT,
            fade_time: 0,
            timer: 0,
            color: CLEAR_COLOR,
            delay_timer: -1,
            delay_status: 0,
        }
    }
}

impl Fader {
    pub fn status(&self) -> i32 {
        self.status
    }

    pub fn color(&self) -> Color {
        self.color
    }

    /// `setColor(JUtility::TColor)` - `JUTFader.h:34`.
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    /// `startFadeIn(int)` - `JUTFader.cpp:60-70`: it only starts from `WaitOut`.
    pub fn start_fade_in(&mut self, fade_time: i32) -> bool {
        let ready = self.status == FADER_WAIT_OUT;
        if ready {
            self.status = FADER_FADE_IN;
            self.timer = 0;
            self.fade_time = fade_time as u16;
        }
        ready
    }

    /// `startFadeOut(int)` - `JUTFader.cpp:73-83`: it only starts from `WaitIn`.
    pub fn start_fade_out(&mut self, fade_time: i32) -> bool {
        let ready = self.status == FADER_WAIT_IN;
        if ready {
            self.status = FADER_FADE_OUT;
            self.timer = 0;
            self.fade_time = fade_time as u16;
        }
        ready
    }

    /// `control()` - `JUTFader.cpp:17-41`.
    ///
    /// The reference divides by `mFadeTime` in both fade arms (`:32`, `:36`), which is a divide by
    /// zero whenever a fade was started with a time of `0` - and `d_ovlp_fade.cpp:77` does start
    /// `startFadeOut(0)`. The port writes the alpha the fade's own end state has (the arm the next
    /// `control` would take) and moves on, rather than leaving an undefined value in the field.
    pub fn control(&mut self) {
        if self.delay_timer >= 0 {
            if self.delay_timer == 0 {
                self.status = self.delay_status;
            }
            self.delay_timer -= 1;
        }
        if self.status == FADER_WAIT_IN {
            return;
        }
        match self.status {
            FADER_WAIT_OUT => self.color[3] = 0xFF,
            FADER_FADE_IN => {
                self.timer = self.timer.wrapping_add(1);
                self.color[3] = if self.fade_time == 0 {
                    0x00
                } else {
                    0xFF - ((u32::from(self.timer) * 0xFF) / u32::from(self.fade_time)) as u8
                };
                if self.timer >= self.fade_time {
                    self.status = FADER_WAIT_IN;
                }
            }
            FADER_FADE_OUT => {
                self.timer = self.timer.wrapping_add(1);
                self.color[3] = if self.fade_time == 0 {
                    0xFF
                } else {
                    ((u32::from(self.timer) * 0xFF) / u32::from(self.fade_time)) as u8
                };
                if self.timer >= self.fade_time {
                    self.status = FADER_WAIT_OUT;
                }
            }
            _ => {}
        }
    }

    /// `mDelayTimer`/`mDelayStatus` - the two fields `control`'s opening lines read
    /// (`JUTFader.cpp:18-20`), written by `d_ovlp_fade2`-style layers; exposed so one can.
    pub fn set_delay(&mut self, timer: i32, status: i32) {
        self.delay_timer = timer;
        self.delay_status = status;
    }
}

/// `mDoGph_gInf_c`'s monotone chase - `m_Do_graphic.cpp:242-260`. The demo's parameter id 6 writes
/// only the speed (`d_a_demo00.cpp:743`); `onMonotone` (`:243`) is what raises the effect, and the
/// chase's own completion is what lowers it again (`:258`).
#[derive(Resource, Debug, Clone, Copy, PartialEq, Default)]
pub struct Monotone {
    active: bool,
    rate: i16,
    rate_speed: i16,
}

impl Monotone {
    /// `setMonotoneRateSpeed(s16)` - `m_Do_graphic.h:76`.
    pub fn set_rate_speed(&mut self, speed: i16) {
        self.rate_speed = speed;
    }

    pub fn rate_speed(&self) -> i16 {
        self.rate_speed
    }

    pub fn rate(&self) -> i16 {
        self.rate
    }

    pub fn is_monotone(&self) -> bool {
        self.active
    }

    /// `onMonotone()` - `m_Do_graphic.cpp:243-247`.
    pub fn on(&mut self) {
        self.active = true;
    }

    /// `offMonotone()` - `m_Do_graphic.cpp:249-254`.
    pub fn off(&mut self) {
        self.active = false;
    }

    /// `calcMonotone()` - `m_Do_graphic.cpp:256-260`, over `cLib_chaseS` (`c_lib.cpp:259-272`).
    pub fn calc(&mut self) {
        let target: i16 = if self.rate_speed < 0 { 400 } else { -600 };
        // `abs(-32768)` is undefined in the reference; the step is clamped rather than wrapped.
        let step = i32::from(self.rate_speed)
            .unsigned_abs()
            .min(u32::from(i16::MAX as u16)) as i16;
        let done = self.chase(target, step);
        if done && self.rate_speed > 0 {
            self.off();
        }
    }

    /// `cLib_chaseS(s16* pValue, s16 target, s16 step)` - `c_lib.cpp:259-272`, whose `1` means
    /// "reached".
    fn chase(&mut self, target: i16, step: i16) -> bool {
        if step != 0 {
            let step = if self.rate > target { -step } else { step };
            self.rate += step;
            if i32::from(step) * i32::from(self.rate - target) >= 0 {
                self.rate = target;
                return true;
            }
        } else if self.rate == target {
            return true;
        }
        false
    }
}
