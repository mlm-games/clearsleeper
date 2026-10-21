use bevy_ecs::prelude::*;

use crate::archive::Archive;
use crate::bti::{self, Image};
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
const MESSAGE_FONT_SIZE: i32 = 23;
const MESSAGE_LINE_SPACE: i32 = 28;
const TX2: usize = 10;

const NAMES: [&str; PANE_COUNT] = [
    "d1", "d2", "d3", "d4", "d42", "d5", "d6", "mak1", "mak2", "tx1", "tx2",
];

/// The order `J2DScreen` draws its hierarchy in: `J2DPane::draw` walks children
/// first to last and a later sibling blends over an earlier one
/// (`J2DPane.cpp:200-202`). Opening.blo stores them BACK, d6, d5, d4, d42,
/// d3, d2, d1, mak1, mak2, tx1, tx2; `dScnOpen_proc_c` draws the screen once
/// (`d_s_open_sub.cpp:562`).
const DRAW_ORDER: [usize; PANE_COUNT] = [D6, D5, D4, D42, D3, D2, D1, MAK1, MAK2, TX1, TX2];

/// The Opening.arc `.bti` entries, decoded once. A pane's texture is the TIMG
/// reference inside its own `PIC1` stream (`J2DPicture.cpp:38`), which
/// `j2d::parse` keeps in `ScreenPane::texture`; `dScnOpen_proc_c` binds the
/// panes by tag (`d_s_open_sub.cpp:233-243`). Opening.blo's references are
/// d1 demo_1.bti, d2 demo_2.bti, d3 demo_3.bti, d4 demo_4.bti,
/// d42 demo_4_2.bti, d5 demo_5.bti, d6 demo_6.bti, mak1/mak2 black.bti;
/// tx1/tx2 are text boxes and have no TIMG.
#[derive(Default)]
pub struct PaneImages {
    entries: Vec<(String, Image)>,
}

impl PaneImages {
    pub fn load(archive: &Archive) -> Self {
        let mut images = Self::default();
        for name in archive.entries() {
            if !name.ends_with(".bti") {
                continue;
            }
            if let Ok(bytes) = archive.read(name) {
                images.add(name, bytes);
            }
        }
        images
    }

    /// Decode one named entry; `false` when the bytes are not a BTI this port reads.
    pub fn add(&mut self, name: &str, bytes: &[u8]) -> bool {
        let Some(image) = bti::decode(bytes) else {
            return false;
        };
        self.entries.push((name.to_string(), image));
        true
    }

    pub fn get(&self, name: &str) -> Option<&Image> {
        self.entries
            .iter()
            .find(|(entry, _)| entry == name)
            .map(|(_, image)| image)
    }
}

/// One opening pane's `J2DPicture`, at the rect and opacity the state machine
/// gives it: the texture the blo entry names, the pane's uv window and corner
/// colours, and `alpha` for `fopMsgM_setAlpha` (`f_op_msg_mng.cpp:9900-9902`).
pub fn draw_pane(
    canvas: &mut Canvas,
    screen: &Screen,
    images: &PaneImages,
    name: &str,
    bounds: &[f32; 4],
    alpha: f32,
) {
    let Some(node) = screen.find(name) else {
        return;
    };
    let Some(texture) = node.texture.as_deref() else {
        return;
    };
    let Some(image) = images.get(texture) else {
        return;
    };
    crate::panes::draw_picture_image(canvas, image, node, *bounds, node.rotation, alpha);
}

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
        // `set_message(0, 0)` leaves the machine idle (`mProc = 4`, `d_s_open_sub.cpp:26-31`):
        // message 0x579 is only shown when state 8 asks for it.
        self.phase = if msg_no == 0 { 3 } else { 0 };
        self.msg_no = resolved;
        self.alpha = 0.0;
        self.timer = 0;
        // `if (param_1 != 0) { field_0x22ec = param_1 - 1; }` (`d_s_open_sub.cpp:37-39`):
        // the repeat count is written only when the parameter is non-zero, so the chain
        // `set_message(msgNo + 1, 0)` (`d_s_open_sub.cpp:158`) keeps the count it had and
        // the next message repeats with it.
        if repeat != 0 {
            self.repeat = i32::try_from(repeat.saturating_sub(1)).unwrap_or(i32::MAX);
        }
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
                // `setLineWidth(tTextBox->getWidth())` is the USA box's 503, and
                // `setCenterLineWidth(486)` (`d_s_open_sub.cpp:89-90`).
                data.line_width = 503;
                data.center_line_width = 486;
                // `g_msgHIO.field_0x70` is 23 on USA (`d_meter.cpp:430`), the box's line
                // space is overridden to 28 (`d_s_open_sub.cpp:56-57`) and its char space is
                // the box's 0 (`:79`).
                data.font_size = MESSAGE_FONT_SIZE;
                data.ruby_font_size = 16;
                data.char_space = 0;
                data.line_space = MESSAGE_LINE_SPACE;
                data.send_speed = 2;
                data.space_timer = 0;
                data.string_length(fonts, MsgLenState::default());
                data.string_shift();
            }
        }
        self.data = data;
    }

    /// One frame of the message machine (`d_s_open_sub.cpp:117-168`); `true` when the delay
    /// before the next message has run out and the caller should load `msg_no + 1`.
    fn exec(&mut self, fonts: &mut FontBank) -> bool {
        self.data.string_set(fonts, &MsgLayoutState::default());
        let mut advance = false;
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
                    self.timer = 0;
                    advance = true;
                } else {
                    self.timer += 1;
                }
            }
        }
        advance
    }
}

impl Intro {
    /// One frame of `dScnOpen_proc_c::proc_execute` (`d_s_open_sub.cpp:282-555`).
    ///
    /// The decomp's `switch` falls out of a case into the next one wherever the case has
    /// no `break` (`d_s_open_sub.cpp:287-544`), so one frame runs several cases: state 2
    /// runs 3, 4 and 5 with it, state 12 runs 13, 14 and 15. The cases that end a frame are
    /// 1, 5, 7, 9, 11, 15, 17, 21, 23, 25, 27, 31, 33, 37, 39, 41, 43 and 44.
    #[allow(clippy::too_many_lines)]
    pub fn step(&mut self, fonts: &mut FontBank, archive: &dyn MsgArchive) {
        if self.done {
            return;
        }
        if self.text.exec(fonts) {
            self.text.set(self.text.msg_no + 1, 0, fonts, archive);
        }

        macro_rules! hold {
            ($limit:expr, $to:expr) => {{
                if self.timer >= $limit {
                    self.state = $to;
                } else {
                    self.timer += 1;
                }
            }};
        }

        let mut case = self.state;
        loop {
            match case {
                0 => {
                    self.timer = 0;
                    self.state = 1;
                }
                1 => {
                    hold!(90, 2);
                    break;
                }
                2 => {
                    self.text.set(0x58E, 1, fonts, archive);
                    self.panes[TX1].offset_y = -190.0;
                    self.panes[TX2].offset_y = -190.0;
                    self.state = 3;
                }
                3 => {
                    if self.text.next {
                        self.state = 4;
                    }
                }
                4 => {
                    self.panes[TX1].offset_y = 0.0;
                    self.panes[TX2].offset_y = 0.0;
                    self.timer = 0;
                    self.state = 5;
                }
                5 => {
                    hold!(60, 6);
                    break;
                }
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
                    break;
                }
                8 => {
                    self.text.set(0x579, 2, fonts, archive);
                    self.panes[D1].alpha = 1.0;
                    self.state = 9;
                }
                9 => {
                    self.pos_x -= SLIDE_SLOW;
                    if self.text.next {
                        self.state = 10;
                    }
                    self.panes[D1].offset_y = self.pos_x;
                    break;
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
                    break;
                }
                12 => {
                    self.text.set(0x57B, 3, fonts, archive);
                    self.state = 13;
                }
                13 => {
                    if self.text.next {
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
                    break;
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
                    break;
                }
                18 => {
                    self.text.set(0x57E, 2, fonts, archive);
                    self.panes[D3].alpha = 1.0;
                    self.state = 19;
                }
                19 => {
                    if self.text.next {
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
                    break;
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
                    break;
                }
                24 => {
                    self.text.set(0x580, 5, fonts, archive);
                    self.panes[D4].alpha = 1.0;
                    self.panes[D42].alpha = 1.0;
                    self.state = 25;
                }
                25 => {
                    self.pos_x -= SLIDE_FAST;
                    if self.text.next {
                        self.state = 26;
                    }
                    self.panes[D4].offset_x = self.pos_x;
                    break;
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
                    break;
                }
                28 => {
                    self.text.set(0x585, 4, fonts, archive);
                    self.panes[D5].alpha = 1.0;
                    self.state = 29;
                }
                29 => {
                    if self.text.next {
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
                    break;
                }
                32 => {
                    self.timer = 0;
                    self.state = 33;
                }
                33 => {
                    hold!(90, 34);
                    break;
                }
                34 => {
                    self.text.set(0x589, 1, fonts, archive);
                    self.panes[TX1].offset_y = -190.0;
                    self.panes[TX2].offset_y = -190.0;
                    self.state = 35;
                }
                35 => {
                    if self.text.next {
                        self.state = 36;
                    }
                }
                36 => {
                    self.panes[TX1].offset_y = 0.0;
                    self.panes[TX2].offset_y = 0.0;
                    self.timer = 0;
                    self.state = 37;
                }
                37 => {
                    hold!(60, 38);
                    break;
                }
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
                    break;
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
                    if self.text.next {
                        self.state = 42;
                    }
                    break;
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
                    break;
                }
                44 => break,
                _ => break,
            }
            case += 1;
        }

        // The message's alpha reaches the text panes at the end of the frame
        // (`d_s_open_sub.cpp:547-549`), after the state machine has run.
        let message_alpha = self.text.alpha;
        self.panes[TX1].alpha = message_alpha;
        self.panes[TX2].alpha = message_alpha;

        // `if (mpProc->mState == 44) changeGameScene();` (`d_s_open.cpp:70-71`).
        if self.state == 44 {
            self.done = true;
        }
    }
}

/// `pane2d[PANE_tx1]->move(..., 376.0f)` on USA (`d_s_open_sub.cpp:250-252`): the
/// opening's text sits in the lower letterbox bar, not at the blo's y = 392.
pub fn usa_text_box(screen: &mut Screen) {
    if let Some(pane) = screen.panes.iter_mut().find(|pane| pane.name == "tx1") {
        pane.bounds.y = 376.0;
    }
}

pub fn render(
    canvas: &mut Canvas,
    screen: &Screen,
    intro: &Intro,
    fonts: &mut FontBank,
    draw_image: &dyn Fn(&mut Canvas, &str, &[f32; 4], f32),
) {
    // The 2D opaque list is drawn over the frame cleared to the back colour, black
    // (`d_drawlist.cpp:1570-1572`), so the opening's letterbox is black.
    canvas.pixels.fill(0);
    for pixel in canvas.pixels.chunks_exact_mut(4) {
        pixel[3] = 0xFF;
    }
    for index in DRAW_ORDER {
        let name = NAMES[index];
        let pane = &intro.panes[index];
        if pane.alpha <= 0.0 {
            continue;
        }
        let Some(layout) = screen.find(name) else {
            continue;
        };
        let Some(bounds) = pane_bounds(screen, intro, name) else {
            continue;
        };
        // The ruby box is hidden on USA (`rTextBox->hide()`, `d_s_open_sub.cpp:117`), and
        // both boxes carry the same `msg1`, so drawing it would strike the line twice.
        if index == TX2 {
            continue;
        }
        if index == TX1 {
            let text = intro.text.data.out[0].clone().unwrap_or_default();
            if text.is_empty() {
                continue;
            }
            let mut print = TextPrint::default();
            print.private_initiate(
                fonts,
                Some(0),
                0.0,
                MESSAGE_LINE_SPACE as f32,
                0xFFFF_FFFF,
                0xFFFF_FFFF,
                0,
                0xFFFF_FFFF,
                false,
            );
            print.set_font_size(MESSAGE_FONT_SIZE as f32, MESSAGE_FONT_SIZE as f32);
            // `shiftSet(0, ((mTextLines - lineCount) - 1) * lineSpace / 2)` centers a short
            // message in the box (`d_s_open_sub.cpp:96-101`); the first line starts at the
            // font's ascent below the box top, as `J2DTextBox::draw` does.
            let entry = intro.text.data.entry;
            let shift = ((i32::from(entry.field_0x16) - intro.text.data.line_count) - 1)
                * (MESSAGE_LINE_SPACE / 2);
            let ascent = fonts.get(0).map_or(0.0, |font| font.ascent() as f32);
            print.locate(0.0, ascent + shift as f32);
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
    }
}

/// A child is drawn at its parent's global origin plus its own local bounds
/// (`J2DPane.cpp:167-168`). Opening.blo nests d42 under d4, so the slide that
/// translates d4 in states 23-27 carries d42 in from the right; every other
/// opening pane hangs off ROOT at (0, 0).
pub fn pane_bounds(screen: &Screen, intro: &Intro, name: &str) -> Option<[f32; 4]> {
    let index = NAMES.iter().position(|entry| *entry == name)?;
    let layout = screen.find(name)?;
    let pane = &intro.panes[index];
    let origin = pane_origin(screen, intro, index, layout);
    Some([
        origin[0],
        origin[1],
        layout.bounds.w * pane.scale,
        layout.bounds.h * pane.scale,
    ])
}

fn pane_origin(
    screen: &Screen,
    intro: &Intro,
    index: usize,
    layout: &crate::j2d::ScreenPane,
) -> [f32; 2] {
    let pane = &intro.panes[index];
    let local = [
        layout.bounds.x + pane.offset_x,
        layout.bounds.y + pane.offset_y,
    ];
    let Some(parent_index) = layout.parent else {
        return local;
    };
    let Some(parent) = screen.panes.get(parent_index) else {
        return local;
    };
    match NAMES.iter().position(|name| *name == parent.name) {
        Some(parent) => {
            let parent_origin = pane_origin(screen, intro, parent, &screen.panes[parent_index]);
            [parent_origin[0] + local[0], parent_origin[1] + local[1]]
        }
        None => [parent.bounds.x + local[0], parent.bounds.y + local[1]],
    }
}
