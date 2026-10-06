use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::intent::DoIntent;
use super::msg::MsgBox;
use super::player::Player;
use crate::assets::MsgArchives;
use crate::text::msgproc::{
    FPC_M_ERROR_PROCESS_ID, MsgGameInfo, MsgProcs, fop_msgm_message_set_actor,
};
use crate::text::{STATUS_BOX_CLOSED, STATUS_MSG_DESTROYED};

pub const OBJ_PAPER_ACTORS: [&str; 3] = ["Paper", "Ppos", "Piwa"];

pub fn is_obj_paper(name: &str) -> bool {
    OBJ_PAPER_ACTORS.contains(&name)
}

pub const TYPE_OPAPER: u8 = 0;
pub const TYPE_PPOS: u8 = 1;
pub const TYPE_PIWA: u8 = 2;

pub const MODE_WAIT: u8 = 0;
pub const MODE_TALK_BEGIN: u8 = 1;
pub const MODE_GET_MSG: u8 = 2;
pub const MODE_TALK_WAIT: u8 = 3;

pub const MODEL_ATTR: u32 = 0x1100_0022;
pub const MSG_NO_MASK: u32 = 0xFFFF;
pub const TYPE_SHIFT: u32 = 0x10;
pub const TYPE_WIDTH: u32 = 0x4;

pub const NO_MSG: u32 = FPC_M_ERROR_PROCESS_ID;

pub const TALK_REACH: f32 = 400.0;
pub const PIWA_STATUS: u32 = 0x18;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Attr {
    pub heap: u32,
    pub eye_offset: i16,
    pub attention_offset: i16,
    pub cull_radius: i16,
    pub cull_y: i16,
    pub talk_dist: i8,
    pub speak_dist: i8,
    pub tev_actor: bool,
    pub cyl_radius: i16,
    pub cyl_height: i16,
}

pub const ATTR: [Attr; 3] = [
    Attr {
        heap: 0x04C0,
        eye_offset: 0x00,
        attention_offset: 0x28,
        cull_radius: 0x28,
        cull_y: 0x00,
        talk_dist: 0x1D,
        speak_dist: 0x1E,
        tev_actor: true,
        cyl_radius: 0x00,
        cyl_height: 0x00,
    },
    Attr {
        heap: 0x04C0,
        eye_offset: 0x00,
        attention_offset: 0x32,
        cull_radius: 0x3C,
        cull_y: 0x00,
        talk_dist: 0x1F,
        speak_dist: 0x20,
        tev_actor: false,
        cyl_radius: 0x00,
        cyl_height: 0x00,
    },
    Attr {
        heap: 0x04C0,
        eye_offset: 0x3C,
        attention_offset: 0x82,
        cull_radius: 0x50,
        cull_y: 0x3C,
        talk_dist: 0x1D,
        speak_dist: 0x1E,
        tev_actor: false,
        cyl_radius: 0x37,
        cyl_height: 0x73,
    },
];

pub fn attr(kind: u8) -> Attr {
    ATTR.get(usize::from(kind)).copied().unwrap_or(ATTR[0])
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Paper {
    pub kind: u8,
    pub mode: u8,
    pub msg_id: u32,
    pub msg_bound: bool,
    pub has_collision: bool,
    pub attended: bool,
    pub eye: Vec3,
    pub cyl_radius: f32,
    pub cyl_height: f32,
    pub entered: bool,
}

impl Paper {
    pub fn spawn(parameters: u32, pos: Vec3) -> Self {
        let kind = ((parameters >> TYPE_SHIFT) & ((1 << TYPE_WIDTH) - 1)) as u8;
        let info = attr(kind);
        Self {
            kind,
            mode: MODE_WAIT,
            msg_id: NO_MSG,
            msg_bound: false,
            has_collision: info.cyl_radius != 0,
            attended: false,
            eye: Vec3::new(pos.x, pos.y + f32::from(info.eye_offset), pos.z),
            cyl_radius: f32::from(info.cyl_radius),
            cyl_height: f32::from(info.cyl_height),
            entered: false,
        }
    }

    pub fn msg_no(parameters: u32) -> u32 {
        parameters & MSG_NO_MASK
    }

    pub fn solid_status(&self) -> u32 {
        if self.kind == TYPE_PIWA { PIWA_STATUS } else { 0 }
    }

    pub fn cull(&self) -> (f32, f32, f32) {
        let info = attr(self.kind);
        (
            0.0,
            f32::from(info.cull_y),
            f32::from(info.cull_radius),
        )
    }

    pub fn attention(&self, pos: Vec3) -> Vec3 {
        let info = attr(self.kind);
        Vec3::new(
            pos.x,
            pos.y + f32::from(info.attention_offset),
            pos.z,
        )
    }

    pub fn create_init(&mut self) {
        self.entered = true;
        self.mode = MODE_WAIT;
        self.msg_id = NO_MSG;
        self.msg_bound = false;
        self.attended = false;
    }

    pub fn in_reach(&self, actor: &Actor, player: &Player) -> bool {
        let dx = player.pos.x - actor.pos.x;
        let dz = player.pos.z - actor.pos.z;
        dx * dx + dz * dz <= TALK_REACH * TALK_REACH
    }

    pub fn mode_wait_init(&mut self) {
        self.mode = MODE_WAIT;
    }

    pub fn mode_wait(&mut self, asked: bool) {
        if asked {
            self.mode_talk0_init();
        }
    }

    pub fn mode_talk0_init(&mut self) {
        self.msg_id = NO_MSG;
        self.msg_bound = false;
        self.attended = true;
        self.mode = MODE_TALK_BEGIN;
    }

    pub fn mode_talk0(
        &mut self,
        procs: &mut MsgProcs,
        info: &mut MsgGameInfo,
        archive: Option<&MsgArchives>,
        msg_no: u32,
    ) {
        if self.msg_id != NO_MSG || !self.attended {
            return;
        }
        self.attended = false;
        self.msg_id = match archive {
            Some(archive) => fop_msgm_message_set_actor(
                procs,
                info,
                archive,
                0,
                msg_no,
                None,
                self.eye,
            ),
            None => NO_MSG,
        };
        self.mode = MODE_GET_MSG;
    }

    pub fn mode_talk1(&mut self, procs: &MsgProcs) {
        if self.msg_id != NO_MSG && procs.search_by_id(self.msg_id).is_some() {
            self.msg_bound = true;
            self.mode = MODE_TALK_WAIT;
        }
    }

    pub fn mode_talk2(&mut self, procs: &mut MsgProcs, box_: &mut MsgBox) {
        if !self.msg_bound || box_.status() != STATUS_BOX_CLOSED {
            return;
        }
        if self.msg_id != NO_MSG {
            procs.delete(self.msg_id);
        }
        box_.set_status(STATUS_MSG_DESTROYED);
        self.msg_id = NO_MSG;
        self.msg_bound = false;
        self.mode_wait_init();
    }
}

pub fn control(
    do_intent: Res<DoIntent>,
    mut procs: ResMut<MsgProcs>,
    mut info: ResMut<MsgGameInfo>,
    mut box_: ResMut<super::msg::MsgBox>,
    archives: Option<Res<MsgArchives>>,
    mut papers: Query<(&mut Actor, &mut Paper)>,
    players: Query<&Player>,
    mut asked: Local<bool>,
) {
    let pressed = do_intent.0 && !*asked;
    *asked = do_intent.0;
    let archive = archives.as_deref();
    let player = players.iter().next().copied();
    for (actor, mut paper) in &mut papers {
        let msg_no = Paper::msg_no(actor.parameters);
        if !paper.entered {
            paper.create_init();
        }
        let ask = pressed
            && player.is_some_and(|player| paper.in_reach(&actor, &player))
            && !box_.active();
        match paper.mode {
            MODE_WAIT => paper.mode_wait(ask),
            MODE_TALK_BEGIN => paper.mode_talk0(&mut procs, &mut info, archive, msg_no),
            MODE_GET_MSG => paper.mode_talk1(&procs),
            _ => paper.mode_talk2(&mut procs, &mut box_),
        }
    }
}

pub fn spawn(world: &mut World, source: &crate::dzs::StageActor) -> Entity {
    let actor = Actor {
        name: source.name.clone(),
        parameters: source.parameters,
        pos: glam::Vec3::from_array(source.pos),
        angle: source.angle,
        set_id: source.set_id,
        room: source.room,
        ..Default::default()
    };
    let paper = Paper::spawn(source.parameters, actor.pos);
    world.spawn((actor, paper)).id()
}