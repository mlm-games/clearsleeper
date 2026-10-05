use bevy_ecs::prelude::*;
use glam::Vec3;

use super::font::{FontBank, ResFont};
use super::fontdata;
use super::msgbox::{Msg2Box, Msg2Ctx, Msg2Env, Msg2Hio, Msg2Screen, Msg2Stick};
use super::msgdata::MsgStickFlags;
use super::msgbox1::{Msg1Box, Msg1Ctx, Msg1Env, Msg1Hio, Msg1Screen, Msg1Stick};
use super::msgdata::MsgLenState;
use super::msgproc::{MeterMsgHio, MsgGameInfo, MsgProcs, FPC_NM_MSG2};
use super::msgget::MsgArchive;
use super::msglayout::MsgLayoutState;

pub const COLOR_TABLE: &str = "color.bmc";

pub const FONT_SLOTS: usize = 2;
pub const MAIN_FONT: usize = 0;
pub const RUBY_FONT: usize = 1;
pub const NAME_MAX: usize = 20;

#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TextEnv {
    pub opt_ruby: bool,
    pub event_bit_2d80: bool,
    pub dif_col: [u8; 3],
    pub meter_mode: u8,
    pub heap_lock: u8,
    pub brightness: u8,
}

impl TextEnv {
    fn msg2(&self) -> Msg2Env {
        Msg2Env {
            opt_ruby: self.opt_ruby,
            event_bit_2d80: self.event_bit_2d80,
            dif_col: self.dif_col,
            meter_mode: self.meter_mode,
            heap_lock_flag: self.heap_lock,
            brightness: self.brightness,
            se: None,
        }
    }

    fn absorb(&mut self, env: Msg2Env) {
        self.meter_mode = env.meter_mode;
        self.brightness = env.brightness;
    }
}

#[derive(Resource, Debug, Clone, Copy, Default, PartialEq)]
pub struct Stick {
    pub pad0_y: f32,
    pub pad3_y: f32,
    pub pad0_x: f32,
    pub trig_a: bool,
    pub trig_b: bool,
    pub trigger: MsgStickFlags,
}

#[derive(Resource, Debug, Clone, Copy, Default, PartialEq)]
pub struct Names {
    pub player: [u8; NAME_MAX],
    pub player_len: usize,
    pub npc: [u8; NAME_MAX],
    pub npc_len: usize,
    pub item: [u8; NAME_MAX],
    pub item_len: usize,
    pub password: [u8; NAME_MAX],
    pub password_len: usize,
}

impl Names {
    fn player(&self) -> &str {
        Self::slice(&self.player, self.player_len)
    }

    fn npc(&self) -> &str {
        Self::slice(&self.npc, self.npc_len)
    }

    fn item(&self) -> &str {
        Self::slice(&self.item, self.item_len)
    }

    fn password(&self) -> &str {
        Self::slice(&self.password, self.password_len)
    }

    fn slice(bytes: &[u8; NAME_MAX], len: usize) -> &str {
        std::str::from_utf8(bytes.get(..len).unwrap_or(&[])).unwrap_or("")
    }
}

#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TextFlags {
    pub clear_count: u8,
    pub pal_language: i32,
    pub next_msg: bool,
}

#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Counters {
    pub event_reg_beff: i32,
    pub event_reg_ba0f: i32,
    pub event_reg_8aff: i32,
    pub event_reg_86ff: i32,
    pub event_reg_c0ff: i32,
    pub message_count: i32,
    pub mini_game_rupee: i32,
    pub fwater_timer: i32,
    pub item_timer: i32,
    pub beast_joy_pendant: i32,
    pub bomb_max: i32,
    pub arrow_max: i32,
    pub buy_item: i32,
    pub pay_rupee: i32,
}

#[derive(Resource, Debug, Clone, Copy, Default, PartialEq)]
pub struct Layout {
    pub npc_name_msg_no: u32,
    pub item_name_msg_no: u32,
    pub password_msg_no: u32,
}

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct TextScreen {
    pub msg2: Msg2Screen,
    pub msg1: Msg1Screen,
}

#[derive(Resource)]
pub struct TextDriver {
    pub out_hio: MeterMsgHio,
    pub msg1_hio: Msg1Hio,
    pub msg2_hio: Msg2Hio,
    pub env: TextEnv,
    pub stick: Stick,
    pub names: Names,
    pub flags: TextFlags,
    pub counters: Counters,
    pub layout: Layout,
    pub screen: TextScreen,
    pub phase1: u8,
    pub phase2: u8,
    pub live_pid: u32,
    pub live: bool,
    pub msg_no: u32,
}

impl Default for TextDriver {
    fn default() -> Self {
        Self {
            out_hio: MeterMsgHio::default(),
            msg1_hio: Msg1Hio::default(),
            msg2_hio: Msg2Hio::default(),
            env: TextEnv::default(),
            stick: Stick::default(),
            names: Names::default(),
            flags: TextFlags::default(),
            counters: Counters::default(),
            layout: Layout::default(),
            screen: TextScreen::default(),
            phase1: 0,
            phase2: 0,
            live_pid: 0,
            live: false,
            msg_no: 0,
        }
    }
}

impl TextDriver {
    fn len_state(&self) -> MsgLenState<'_> {
        MsgLenState {
            player_name: self.names.player(),
            npc_name: self.names.npc(),
            item_name: self.names.item(),
            pirate_password: self.names.password(),
            pal_language: self.flags.pal_language,
            clear_count: self.flags.clear_count,
            event_reg_beff: self.counters.event_reg_beff,
            message_count_number: self.counters.message_count,
            mini_game_rupee: self.counters.mini_game_rupee,
            event_reg_8aff: self.counters.event_reg_8aff,
            fwater_timer: self.counters.fwater_timer,
            event_reg_86ff: self.counters.event_reg_86ff,
            beast_joy_pendant: self.counters.beast_joy_pendant,
            event_reg_c0ff: self.counters.event_reg_c0ff,
            item_timer: self.counters.item_timer,
            bomb_max: self.counters.bomb_max,
            arrow_max: self.counters.arrow_max,
            buy_item: self.counters.buy_item,
            pay_rupee: self.counters.pay_rupee,
        }
    }

    fn layout_state<'a>(
        &'a self,
        npc: &'a [u8],
        item: &'a [u8],
        password: &'a [u8],
        clt1: &'a [u8],
    ) -> MsgLayoutState<'a> {
        MsgLayoutState {
            player_name: self.names.player(),
            npc_name_msg_no: self.layout.npc_name_msg_no,
            npc_name_msg: npc,
            item_name_msg_no: self.layout.item_name_msg_no,
            item_name_msg: item,
            pirate_password_msg_no: self.layout.password_msg_no,
            pirate_password_msg: password,
            pal_language: self.flags.pal_language,
            clear_count: self.flags.clear_count,
            next_msg_flag: self.flags.next_msg,
            msg_hio_0x6c: 0,
            color_clt1: clt1,
            event_reg_beff: self.counters.event_reg_beff,
            event_reg_ba0f: self.counters.event_reg_ba0f,
            event_reg_8aff: self.counters.event_reg_8aff,
            event_reg_86ff: self.counters.event_reg_86ff,
            event_reg_c0ff: self.counters.event_reg_c0ff,
            message_count_number: self.counters.message_count,
            mini_game_rupee: self.counters.mini_game_rupee,
            fwater_timer: self.counters.fwater_timer,
            item_timer: self.counters.item_timer,
            beast_joy_pendant: self.counters.beast_joy_pendant,
            bomb_max: self.counters.bomb_max,
            arrow_max: self.counters.arrow_max,
            buy_item: self.counters.buy_item,
            pay_rupee: self.counters.pay_rupee,
        }
    }

    pub fn set_player_name(&mut self, name: &str) {
        let take = name.len().min(NAME_MAX);
        self.names.player.fill(0);
        self.names.player_len = take;
        if let Some(slot) = self.names.player.get_mut(..take) {
            slot.copy_from_slice(name.as_bytes().get(..take).unwrap_or(&[]));
        }
    }

    pub fn set_npc_name(&mut self, name: &str) {
        let take = name.len().min(NAME_MAX);
        self.names.npc.fill(0);
        self.names.npc_len = take;
        if let Some(slot) = self.names.npc.get_mut(..take) {
            slot.copy_from_slice(name.as_bytes().get(..take).unwrap_or(&[]));
        }
    }

}

pub fn load_fonts(fonts: &mut FontBank) {
    if fonts.len() >= FONT_SLOTS {
        return;
    }
    let disc = crate::assets::load_font();
    while fonts.len() < FONT_SLOTS {
        let font = match &disc {
            Some(bytes) => ResFont::from_bytes(bytes),
            None => ResFont::from_bytes(fontdata::ASCENT_FIX12),
        };
        fonts.insert(font);
    }
}

pub fn control(
    archive: Option<Res<crate::assets::MsgArchives>>,
    mut procs: ResMut<MsgProcs>,
    mut info: ResMut<MsgGameInfo>,
    mut driver: ResMut<TextDriver>,
    mut one: ResMut<Msg1Box>,
    mut two: ResMut<Msg2Box>,
    mut fonts: ResMut<FontBank>,
) {
    load_fonts(&mut fonts);
    let pending = procs.procs().last().and_then(|proc| {
        let msg_no = match &proc.append {
            super::msgproc::MsgAppend::Class(params) => params.m_msg_no,
            _ => proc.msg_no,
        };
        (msg_no != 0).then_some((proc.pid, proc.name, msg_no))
    });
    let Some((pid, name, msg_no)) = pending else {
        driver.live = false;
        driver.phase1 = 0;
        driver.phase2 = 0;
        driver.live_pid = 0;
        return;
    };
    driver.live = true;
    driver.msg_no = msg_no;
    // a different process became the newest one: the live box belongs to the old pid,
    // so create the new one and remember which pid it is, or the old box finishing
    // would delete the wrong process
    if driver.live_pid != pid {
        if driver.live_pid != super::msgproc::FPC_M_ERROR_PROCESS_ID {
            procs.delete(driver.live_pid);
        }
        driver.live_pid = pid;
        driver.phase1 = 0;
        driver.phase2 = 0;
    }
    let Some(archive) = archive else {
        return;
    };
    let is_two = name == FPC_NM_MSG2;
    if is_two {
        if driver.phase2 == 0 {
            two.msg_no = msg_no;
            let mut env = driver.env.msg2();
            {
                let len = driver.len_state();
                let layout = driver.layout_state(
                    driver.names.npc().as_bytes(),
                    driver.names.item().as_bytes(),
                    driver.names.password().as_bytes(),
                    archive.resource(COLOR_TABLE).unwrap_or_default(),
                );
                let mut ctx = Msg2Ctx {
                    hio: &driver.msg2_hio,
                    out_hio: &driver.out_hio,
                    env: &mut env,
                    archive: archive.as_ref(),
                    fonts: &fonts,
                    mx_font: MAIN_FONT,
                    rx_font: RUBY_FONT,
                    screen: &driver.screen.msg2,
                    len: &len,
                    layout: &layout,
                    stick: Msg2Stick {
                        pad0_y: driver.stick.pad0_y,
                        pad3_y: driver.stick.pad3_y,
                        trig_a: driver.stick.trig_a,
                    },
                };
                two.create(&mut ctx, &mut info, msg_no);
            }
            driver.env.absorb(env);
            driver.phase2 = 1;
        }
        let mut env = driver.env.msg2();
        let outcome = {
            let len = driver.len_state();
            let layout = driver.layout_state(
                driver.names.npc().as_bytes(),
                driver.names.item().as_bytes(),
                driver.names.password().as_bytes(),
                archive.resource(COLOR_TABLE).unwrap_or_default(),
            );
            let mut ctx = Msg2Ctx {
                hio: &driver.msg2_hio,
                out_hio: &driver.out_hio,
                env: &mut env,
                archive: archive.as_ref(),
                fonts: &fonts,
                mx_font: MAIN_FONT,
                rx_font: RUBY_FONT,
                screen: &driver.screen.msg2,
                len: &len,
                layout: &layout,
                stick: Msg2Stick {
                    pad0_y: driver.stick.pad0_y,
                    pad3_y: driver.stick.pad3_y,
                    trig_a: driver.stick.trig_a,
                },
            };
            two.execute(&mut ctx, &mut info)
        };
        driver.env.absorb(env);
        if !outcome {
            driver.phase2 = 0;
            driver.live = false;
            procs.delete(pid);
        }
        return;
    }
    if driver.phase1 == 0 {
        one.msg_no = msg_no;
        let mut env = Msg1Env::default();
        {
            let len = driver.len_state();
            let layout = driver.layout_state(
                driver.names.npc().as_bytes(),
                driver.names.item().as_bytes(),
                driver.names.password().as_bytes(),
                archive.resource(COLOR_TABLE).unwrap_or_default(),
            );
            let mut ctx = Msg1Ctx {
                hio: &driver.msg1_hio,
                env: &mut env,
                archive: archive.as_ref(),
                fonts: &fonts,
                mx_font: MAIN_FONT,
                rx_font: RUBY_FONT,
                screen: &driver.screen.msg1,
                len: &len,
                layout: &layout,
                out_hio: &driver.out_hio,
                stick: Msg1Stick {
                    stick_x: driver.stick.pad0_x,
                    stick_y: driver.stick.pad0_y,
                    trig_a: driver.stick.trig_a,
                    trig_b: driver.stick.trig_b,
                    trigger: driver.stick.trigger,
                },
            };
            one.create(&mut ctx, &mut info);
        }
        driver.phase1 = 1;
    }
    let len = driver.len_state();
    let layout = driver.layout_state(
        driver.names.npc().as_bytes(),
        driver.names.item().as_bytes(),
        driver.names.password().as_bytes(),
        archive.resource(COLOR_TABLE).unwrap_or_default(),
    );
    let mut env = Msg1Env::default();
    let mut ctx = Msg1Ctx {
        hio: &driver.msg1_hio,
        env: &mut env,
        archive: archive.as_ref(),
        fonts: &fonts,
        mx_font: MAIN_FONT,
        rx_font: RUBY_FONT,
        screen: &driver.screen.msg1,
        len: &len,
        layout: &layout,
        out_hio: &driver.out_hio,
        stick: Msg1Stick {
            stick_x: driver.stick.pad0_x,
            stick_y: driver.stick.pad0_y,
            trig_a: driver.stick.trig_a,
            trig_b: driver.stick.trig_b,
            trigger: driver.stick.trigger,
        },
    };
    if !one.execute(&mut ctx, &mut info) {
        driver.phase1 = 0;
        driver.live = false;
        procs.delete(pid);
    }
}

pub fn spawn(
    procs: &mut MsgProcs,
    info: &mut MsgGameInfo,
    archive: &dyn super::msgget::MsgArchive,
    clear_count: u8,
    msg_no: u32,
) -> u32 {
    super::msgproc::fop_msgm_message_set(
        procs,
        info,
        archive,
        clear_count,
        msg_no,
        None,
        Vec3::ZERO,
    )
}