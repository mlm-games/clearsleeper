use bevy_ecs::prelude::*;

use crate::j2d::Screen;
use crate::text::font::FontBank;
use crate::text::msgbox::TextPrint;
use crate::text::msgdata::MsgLenState;
use crate::text::msgget::{MsgArchive, MsgGet};
use crate::text::msglayout::MsgLayoutState;
use crate::text::raster::Canvas;
use crate::text::{MesgEntry, MsgData};

const TIMER_TABLE: [i8; 22] = [
    7, 6, 7, 6, 6, 7, 9, 8, 9, 6, 7, 5, 8, 6, 8, 7, 7, 9, 10, 10, 7, 7,
];

const MSG_FIRST: u32 = 0x579;

const SLIDE_SLOW: f32 = 0.571_962_6;
const SLIDE_FAST: f32 = 0.474_820_14;
const FADE: f32 = 0.067;
const CROSS: f32 = 0.022_3;
const SCALE_STEP: f32 = 0.000_763_358_8;

const PANE_COUNT: usize = 11;
const D1: usize = 0;
const D2: usize = 1;
const D3: usize = 2;
const D4: usize = 3;
const D42: usize = 4;
const D5: usize = 5;
const D6: usize = 6;
const MAK1: usize = 7;
const MAK2: usize = 8;
const TX1: usize = 9;
const TX2: usize = 10;

const NAMES: [&str; PANE_COUNT] = [
    "d1", "d2", "d3", "d4", "d42", "d5", "d6", "mak1", "mak2", "tx1", "tx2",
];

const BLACK: [u8; 4] = [0, 0, 0, 255];

#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct Pane {
    pub alpha: f32,
    pub offset_x: f32,
    pub offset_y: f32,
    pub scale: f32,
}

#[derive(Resource, Clone, Debug)]
pub struct Text {
    pub phase: u8,
    pub msg_no: u32,
    pub alpha: f32,
    pub timer: i32,
    pub timer_max: i32,
    pub repeat: i32,
    pub next: bool,
    pub data: MsgData,
}

impl Default for Text {
    fn default() -> Self {
        Self {
            phase: 3,
            msg_no: 0,
            alpha: 0.0,
            timer: 0,
            timer_max: 0,
            repeat: 0,
            next: false,
            data: MsgData::new_data_proc(),
        }
    }
}

#[derive(Resource, Clone, Debug)]
pub struct Intro {
    pub state: u8,
    pub timer: i32,
    pub pos_x: f32,
    pub alpha: f32,
    pub scale: f32,
    pub panes: [Pane; PANE_COUNT],
    pub text: Text,
    pub done: bool,
}

impl Default for Intro {
    fn default() -> Self {
        Self {
            state: 0,
            timer: 0,
            pos_x: 0.0,
            alpha: 0.0,
            scale: 1.0,
            panes: [Pane {
                alpha: 0.0,
                offset_x: 0.0,
                offset_y: 0.0,
                scale: 1.0,
            }; PANE_COUNT],
            text: Text::default(),
            done: false,
        }
    }
}

impl Intro {
    pub fn start(&mut self, fonts: &mut FontBank, archive: &dyn MsgArchive) {
        *self = Self::default();
        self.panes[MAK1].alpha = 1.0;
        self.panes[MAK2].alpha = 1.0;
        self.text.set(0, 0, fonts, archive);
    }
}

impl Text {
    fn set(&mut self, msg_no: u32, repeat: u32, fonts: &mut FontBank, archive: &dyn MsgArchive) {
        let resolved = if msg_no == 0 { MSG_FIRST } else { msg_no };
        self.phase = 0;
        self.msg_no = resolved;
        self.alpha = 0.0;
        self.timer = 0;
        self.repeat = if msg_no != 0 { repeat as i32 } else { 0 };
        self.next = false;
        let index = (resolved - MSG_FIRST) as usize;
        self.timer_max = i32::from(TIMER_TABLE.get(index).copied().unwrap_or(7)) * 30;

        let mut get = MsgGet::default();
        let mut data = MsgData::new_data_proc();
        data.out = [
            Some(String::new()),
            Some(String::new()),
            Some(String::new()),
            Some(String::new()),
        ];
        if let Some(head) = get.get_mesg_header(archive, 0, resolved) {
            if let Some(bytes) = get.get_message(head) {
                let entry: MesgEntry = get.get_mesg_entry(head).unwrap_or_default();
                data.set_bmg(bytes.to_vec());
                data.entry = entry;
                data.line_width = 486;
                data.center_line_width = 486;
                data.font_size = 29;
                data.ruby_font_size = 16;
                data.char_space = 10;
                data.line_space = 28;
                data.send_speed = 2;
                data.space_timer = 0;
                data.string_length(fonts, MsgLenState::default());
                data.string_shift();
            }
        }
        self.data = data;
    }

    fn exec(&mut self, fonts: &mut FontBank) {
        self.data.string_set(fonts, &MsgLayoutState::default());
        match self.phase {
            0 => {
                if self.alpha >= 1.0 {
                    self.alpha = 1.0;
                    self.timer = 0;
                    self.phase = 1;
                } else {
                    self.alpha += 0.1;
                }
            }
            1 => {
                if self.timer >= self.timer_max {
                    self.timer = 0;
                    self.alpha = 1.0;
                    self.phase = 2;
                } else {
                    self.timer += 1;
                }
            }
            2 => {
                if self.alpha <= 0.0 {
                    self.alpha = 0.0;
                    if self.repeat == 0 {
                        self.next = true;
                        self.phase = 3;
                    } else {
                        self.repeat -= 1;
                        self.phase = 4;
                    }
                } else {
                    self.alpha -= 0.1;
                }
            }
            3 => {}
            _ => {
                if self.timer >= 45 {
                    self.msg_no += 1;
                    self.phase = 0;
                    self.timer = 0;
                    self.data = MsgData::new_data_proc();
                    self.data.out = [
                        Some(String::new()),
                        Some(String::new()),
                        Some(String::new()),
                        Some(String::new()),
                    ];
                } else {
                    self.timer += 1;
                }
            }
        }
    }
}

impl Intro {
    #[allow(clippy::too_many_lines)]
    pub fn step(&mut self, fonts: &mut FontBank, archive: &dyn MsgArchive) {
        if self.done {
            return;
        }
        self.text.exec(fonts);
        let message_alpha = self.text.alpha;
        let next = self.text.next;
        self.panes[TX1].alpha = message_alpha;
        self.panes[TX2].alpha = message_alpha;

        macro_rules! hold {
            ($limit:expr, $to:expr) => {{
                if self.timer >= $limit {
                    self.state = $to;
                } else {
                    self.timer += 1;
                }
            }};
        }

        match self.state {
            0 => {
                self.timer = 0;
                self.state = 1;
            }
            1 => hold!(90, 2),
            2 => {
                self.text.set(0x58E, 1, fonts, archive);
                self.panes[TX1].offset_y = -190.0;
                self.panes[TX2].offset_y = -190.0;
                self.state = 3;
            }
            3 => {
                if next {
                    self.state = 4;
                }
            }
            4 => {
                self.panes[TX1].offset_y = 0.0;
                self.panes[TX2].offset_y = 0.0;
                self.timer = 0;
                self.state = 5;
            }
            5 => hold!(60, 6),
            6 => {
                self.panes[D1].alpha = 0.0;
                self.pos_x = 0.0;
                self.alpha = 0.0;
                self.state = 7;
            }
            7 => {
                self.pos_x -= SLIDE_SLOW;
                self.alpha += FADE;
                if self.alpha >= 1.0 {
                    self.alpha = 1.0;
                    self.state = 8;
                }
                self.panes[D1].offset_y = self.pos_x;
                self.panes[D1].alpha = self.alpha;
            }
            8 => {
                self.text.set(0x579, 2, fonts, archive);
                self.panes[D1].alpha = 1.0;
                self.state = 9;
            }
            9 => {
                self.pos_x -= SLIDE_SLOW;
                if next {
                    self.state = 10;
                }
                self.panes[D1].offset_y = self.pos_x;
            }
            10 => {
                self.panes[D1].alpha = 1.0;
                self.panes[D2].alpha = 0.0;
                self.alpha = 0.0;
                self.state = 11;
            }
            11 => {
                self.pos_x -= SLIDE_SLOW;
                self.alpha += CROSS;
                if self.alpha >= 1.0 {
                    self.alpha = 1.0;
                    self.state = 12;
                }
                self.panes[D1].offset_y = self.pos_x;
                self.panes[D1].alpha = 1.0 - self.alpha;
                self.panes[D2].alpha = self.alpha;
            }
            12 => {
                self.text.set(0x57B, 3, fonts, archive);
                self.state = 13;
            }
            13 => {
                if next {
                    self.state = 14;
                }
            }
            14 => {
                self.alpha = 1.0;
                self.state = 15;
            }
            15 => {
                self.alpha -= FADE;
                if self.alpha <= 0.0 {
                    self.alpha = 0.0;
                    self.state = 16;
                }
                self.panes[D2].alpha = self.alpha;
            }
            16 => {
                self.alpha = 0.0;
                self.state = 17;
            }
            17 => {
                self.alpha += FADE;
                if self.alpha >= 1.0 {
                    self.alpha = 1.0;
                    self.state = 18;
                }
                self.panes[D3].alpha = self.alpha;
            }
            18 => {
                self.text.set(0x57E, 2, fonts, archive);
                self.panes[D3].alpha = 1.0;
                self.state = 19;
            }
            19 => {
                if next {
                    self.state = 20;
                }
            }
            20 => {
                self.alpha = 1.0;
                self.state = 21;
            }
            21 => {
                self.alpha -= FADE;
                if self.alpha <= 0.0 {
                    self.alpha = 0.0;
                    self.state = 22;
                }
                self.panes[D3].alpha = self.alpha;
            }
            22 => {
                self.pos_x = 0.0;
                self.alpha = 0.0;
                self.state = 23;
            }
            23 => {
                self.pos_x -= SLIDE_FAST;
                self.alpha += FADE;
                if self.alpha >= 1.0 {
                    self.alpha = 1.0;
                    self.state = 24;
                }
                self.panes[D4].offset_x = self.pos_x;
                self.panes[D4].alpha = self.alpha;
                self.panes[D42].alpha = self.alpha;
            }
            24 => {
                self.text.set(0x580, 5, fonts, archive);
                self.panes[D4].alpha = 1.0;
                self.panes[D42].alpha = 1.0;
                self.state = 25;
            }
            25 => {
                self.pos_x -= SLIDE_FAST;
                if next {
                    self.state = 26;
                }
                self.panes[D4].offset_x = self.pos_x;
            }
            26 => {
                self.panes[D4].alpha = 1.0;
                self.panes[D42].alpha = 1.0;
                self.panes[D5].alpha = 0.0;
                self.alpha = 0.0;
                self.state = 27;
            }
            27 => {
                self.pos_x -= SLIDE_FAST;
                self.alpha += CROSS;
                if self.alpha >= 1.0 {
                    self.alpha = 1.0;
                    self.state = 28;
                }
                self.panes[D4].offset_x = self.pos_x;
                self.panes[D4].alpha = 1.0 - self.alpha;
                self.panes[D42].alpha = 1.0 - self.alpha;
                self.panes[D5].alpha = self.alpha;
            }
            28 => {
                self.text.set(0x585, 4, fonts, archive);
                self.panes[D5].alpha = 1.0;
                self.state = 29;
            }
            29 => {
                if next {
                    self.state = 30;
                }
            }
            30 => {
                self.alpha = 1.0;
                self.state = 31;
            }
            31 => {
                self.alpha -= FADE;
                if self.alpha <= 0.0 {
                    self.alpha = 0.0;
                    self.state = 32;
                }
                self.panes[D5].alpha = self.alpha;
            }
            32 => {
                self.timer = 0;
                self.state = 33;
            }
            33 => hold!(90, 34),
            34 => {
                self.text.set(0x589, 1, fonts, archive);
                self.panes[TX1].offset_y = -190.0;
                self.panes[TX2].offset_y = -190.0;
                self.state = 35;
            }
            35 => {
                if next {
                    self.state = 36;
                }
            }
            36 => {
                self.panes[TX1].offset_y = 0.0;
                self.panes[TX2].offset_y = 0.0;
                self.timer = 0;
                self.state = 37;
            }
            37 => hold!(60, 38),
            38 => {
                self.scale = 2.0;
                self.alpha = 0.0;
                self.state = 39;
            }
            39 => {
                self.scale -= SCALE_STEP;
                self.alpha += FADE;
                if self.alpha >= 1.0 {
                    self.alpha = 1.0;
                    self.state = 40;
                }
                self.panes[D6].alpha = self.alpha;
                self.panes[D6].scale = self.scale;
            }
            40 => {
                self.text.set(0x58A, 4, fonts, archive);
                self.panes[D6].alpha = 1.0;
                self.state = 41;
            }
            41 => {
                self.scale -= SCALE_STEP;
                if self.scale < 1.0 {
                    self.scale = 1.0;
                }
                self.panes[D6].scale = self.scale;
                if next {
                    self.state = 42;
                }
            }
            42 => {
                self.alpha = 1.0;
                self.state = 43;
            }
            43 => {
                self.alpha -= FADE;
                if self.alpha <= 0.0 {
                    self.alpha = 0.0;
                    self.state = 44;
                }
                self.panes[D6].alpha = self.alpha;
            }
            44 => {
                self.done = true;
            }
            _ => {}
        }
    }
}

pub fn render(
    canvas: &mut Canvas,
    screen: &Screen,
    intro: &Intro,
    fonts: &mut FontBank,
    draw_image: &dyn Fn(&mut Canvas, &str, &[f32; 4], f32),
) {
    for (index, name) in NAMES.iter().enumerate() {
        let pane = &intro.panes[index];
        if pane.alpha <= 0.0 {
            continue;
        }
        let Some(layout) = screen.find(name) else {
            continue;
        };
        let bounds = [
            layout.bounds.x + pane.offset_x,
            layout.bounds.y + pane.offset_y,
            layout.bounds.w * pane.scale,
            layout.bounds.h * pane.scale,
        ];
        if index == TX1 || index == TX2 {
            let text = intro.text.data.out[0].clone().unwrap_or_default();
            if text.is_empty() {
                continue;
            }
            let mut print = TextPrint::default();
            print.private_initiate(
                fonts,
                Some(0),
                0.0,
                0.0,
                0xFFFF_FFFF,
                0xFFFF_FFFF,
                0,
                0xFFFF_FFFF,
                false,
            );
            print.set_font_size(layout.font_size_x as f32, layout.font_size_y as f32);
            print.locate(0.0, layout.bounds.h);
            let text_layout = print.print_return(
                fonts,
                text.as_bytes(),
                layout.bounds.w,
                layout.bounds.h,
                0,
                0,
                0.0,
                0.0,
                0xFF,
            );
            if let Some(font) = fonts.get(0) {
                canvas.draw_layout(
                    &text_layout,
                    font,
                    [bounds[0], bounds[1]],
                    (255.0 * pane.alpha.clamp(0.0, 1.0)) as u8,
                );
            }
            continue;
        }
        if layout.kind == crate::j2d::Kind::TextBox {
            continue;
        }
        draw_image(canvas, name, &bounds, pane.alpha);
        if *name == "mak1" || *name == "mak2" {
            canvas.fill_rect(
                bounds,
                [
                    BLACK[0],
                    BLACK[1],
                    BLACK[2],
                    (255.0 * pane.alpha.clamp(0.0, 1.0)) as u8,
                ],
            );
        }
    }
}
