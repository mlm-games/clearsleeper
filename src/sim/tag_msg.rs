use bevy_ecs::prelude::*;

use super::actor::Actor;
use super::flags::{EventFlags, Switches};
use super::player::Player;
use super::walk::to_s16;
use crate::assets::MsgArchives;
use crate::text::msgproc::{
    FPC_M_ERROR_PROCESS_ID, MsgGameInfo, MsgProcs, fop_msgm_message_set_actor,
};

pub const TAG_MSG_ACTORS: [&str; 1] = ["TagMsg2"];

pub fn is_tag_msg(name: &str) -> bool {
    TAG_MSG_ACTORS.contains(&name)
}

const MSG_HIDE_LINK: u16 = 0x1902;
const MSG_KEEP_HEIGHT: u16 = 0x836;
const ACTION_WAIT: u8 = 0;
const ACTION_ARRIVAL: u8 = 1;
const ACTION_HUNT: u8 = 2;
const ACTION_EVENT: u8 = 3;

#[derive(Resource, Debug, Clone, Copy)]
pub struct TagMsg {
    pub swbit: u8,
    pub swbit2: u8,
    pub type2: u8,
    pub message: u16,
    pub event_flag: u16,
    pub event_no: u8,
    pub action: u8,
    pub msg_mode: u8,
    pub msg_id: u32,
    pub scale: [f32; 3],
    pub eye_pos: glam::Vec3,
}

impl TagMsg {
    pub fn spawn(actor: &Actor, source: &crate::dzs::StageActor) -> Self {
        let parameters = source.parameters;
        let message = source.angle[0] as u16;
        let lift = if message != MSG_KEEP_HEIGHT { 150.0 } else { 0.0 };
        let pos = actor.pos;
        Self {
            swbit: ((parameters >> 8) & 0xFF) as u8,
            swbit2: ((parameters >> 16) & 0xFF) as u8,
            type2: ((parameters >> 6) & 3) as u8,
            message,
            event_flag: source.angle[2] as u16,
            event_no: ((source.angle[1] >> 8) & 0xFF) as u8,
            action: ACTION_WAIT,
            msg_mode: 0,
            msg_id: FPC_M_ERROR_PROCESS_ID,
            scale: source.scale,
            eye_pos: glam::Vec3::new(pos.x, pos.y + lift, pos.z),
        }
    }
}

fn abs_diff(a: i16, b: i16) -> i16 {
    let diff = a.wrapping_sub(b);
    if diff < 0 {
        diff.wrapping_neg()
    } else {
        diff
    }
}

fn create_action(tag: &TagMsg, switches: &Switches, events: &EventFlags) -> u8 {
    if u32::from(tag.message) == 0x9C5_u32 && events.test(0x0502) {
        ACTION_WAIT
    } else if tag.swbit != 0xFF && switches.test(tag.swbit) {
        ACTION_WAIT
    } else {
        ACTION_ARRIVAL
    }
}

fn arrival_terms(tag: &TagMsg, switches: &Switches, events: &EventFlags) -> bool {
    if tag.swbit2 != 0xFF && !switches.test(tag.swbit2) {
        return false;
    }
    if tag.event_flag != 0xFFFF && !events.test(u32::from(tag.event_flag)) {
        return false;
    }
    true
}

fn range_check(tag: &TagMsg, actor: &Actor, player: &Player) -> bool {
    let mut diff = player.pos - actor.pos;
    if diff.y < 0.0 {
        diff.y = -diff.y;
    }
    let radius = tag.scale[0] * 100.0;
    diff.x * diff.x + diff.z * diff.z < radius * radius && diff.y <= tag.scale[1] * 100.0
}

fn other_check(tag: &TagMsg, actor: &Actor, player: &Player) -> bool {
    if tag.type2 & 1 != 0 {
        return true;
    }
    let target_angle = to_s16((player.pos.x - actor.pos.x).atan2(player.pos.z - actor.pos.z));
    if u32::from(tag.message) == u32::from(MSG_HIDE_LINK) && abs_diff(target_angle, actor.angle[1]) > 0x3000
    {
        return false;
    }
    let facing = to_s16(player.yaw);
    abs_diff(target_angle.wrapping_add(0x7FFF), facing) <= 0x1000
}

pub fn control(
    mut tags: Query<(&mut TagMsg, &Actor)>,
    players: Query<&Player>,
    switches: Res<Switches>,
    events: Res<EventFlags>,
    mut procs: ResMut<MsgProcs>,
    mut info: ResMut<MsgGameInfo>,
    archives: Option<Res<MsgArchives>>,
) {
    let Some(player) = players.iter().next().copied() else {
        return;
    };
    for (mut tag, actor) in tags.iter_mut() {
        if tag.action == ACTION_WAIT {
            let action = create_action(&tag, &switches, &events);
            if action == ACTION_WAIT {
                continue;
            }
            tag.action = ACTION_ARRIVAL;
        }
        if tag.action != ACTION_ARRIVAL {
            continue;
        }
        if !arrival_terms(&tag, &switches, &events) {
            continue;
        }
        if !range_check(&tag, actor, &player) || !other_check(&tag, actor, &player) {
            continue;
        }
        tag.action = ACTION_EVENT;
        tag.msg_mode = 0;
        tag.msg_id = match &archives {
            Some(archive) => fop_msgm_message_set_actor(
                &mut procs,
                &mut info,
                archive.as_ref(),
                0,
                u32::from(tag.message),
                None,
                tag.eye_pos,
            ),
            None => FPC_M_ERROR_PROCESS_ID,
        };
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
    };
    let tag = TagMsg::spawn(&actor, source);
    world.spawn((actor, tag)).id()
}