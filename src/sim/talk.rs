use bevy_ecs::prelude::*;
use glam::Vec2;

use super::actor::Actor;
use super::area::StageName;
use super::flags::{Arrivals, EventFlags, StoryBits};
use super::health::Health;
use super::intent::{DoIntent, MoveIntent};
use super::layer::SEA;
use super::player::Player;

pub const TALK_ACTOR: &str = "NpcSo";

const TAG_ACTOR: &str = "TagSo";
const MINIGAME_TYPE_SHIFT: u32 = 16;
const MINIGAME_TYPE_MASK: u32 = 0xFF;
const MINIGAME_TYPE: u32 = 1;
const DRAGON_ROOST: i8 = 13;
const TALK_RANGE: f32 = 400.0;
const DEFAULT_MESSAGE: u16 = 1;
const BLANK_MESSAGE: i16 = 0;
const MISSING_MESSAGE: i16 = -1;

const TALK_END_FLAG: u32 = 0x0008;
const SPOKEN_FLAG: u16 = 0x0901;
const BOW_EVENT_FLAG: u16 = 0x3A10;

const MEET_A: u16 = 0x32CA;
const MEET_B: u16 = 0x32CB;
const MEET_C: u16 = 0x32CC;
const MEET_D: u16 = 0x32CD;
const FIRST_END: u16 = 0x32CE;
const ARRIVE_CHECK: u16 = 0x32D0;
const MAP_OFFER: u16 = 0x32D1;
const MAP_DONE: u16 = 0x32D2;
const MAP_CLOSING: u16 = 0x32D3;
const ARRIVED_RETURN: u16 = 0x32D4;
const BOW_PROMPT: u16 = 0x32D6;
const BOW_DECLINE: u16 = 0x32D7;
const BOW_OFFER: u16 = 0x32D8;
const BOW_REFUSED: u16 = 0x32D9;
const BOW_ACCEPTED: u16 = 0x32DA;
const BOW_EVENT: u16 = 0x32DB;
const BOW_OFFER_AGAIN: u16 = 0x32DC;
const RUPEE_PROMPT: u16 = 0x32DD;
const RUPEE_ACCEPTED: u16 = 0x32DE;
const RUPEE_TALLY_MANY: u16 = 0x32DF;
const RUPEE_TALLY_ONE: u16 = 0x32E0;
const AFTER_BOW: u16 = 0x32E1;
const CLOSING: u16 = 0x32E2;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Talk {
    pub base: u16,
    pub room: i8,
    pub map_done: bool,
    pub bow_done: bool,
    pub rupee_done: bool,
    pub rupee_count: u8,
    pub gone: bool,
}

impl Talk {
    pub fn new(base: u16, room: i8) -> Self {
        Self {
            base,
            room,
            map_done: false,
            bow_done: false,
            rupee_done: false,
            rupee_count: 0,
            gone: false,
        }
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dialogue {
    pub open: bool,
    pub message: u16,
    pub speaker: Option<Entity>,
    held: bool,
}

impl Default for Dialogue {
    fn default() -> Self {
        Self {
            open: false,
            message: 0,
            speaker: None,
            held: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scene {
    MapOpen,
    FirstEnd,
    Bow,
    GetRupee,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Next(u16),
    Done(Option<Scene>),
}

#[derive(Debug, Clone, Copy)]
pub struct Context {
    pub stories: StoryBits,
    pub arrivals: Arrivals,
    pub sea: bool,
    pub has_bow: bool,
    pub minigame_tag: bool,
    pub timer_active: bool,
    pub accepted: bool,
}

pub fn message(angle_x: i16) -> u16 {
    if angle_x == BLANK_MESSAGE || angle_x == MISSING_MESSAGE {
        DEFAULT_MESSAGE
    } else {
        angle_x as u16
    }
}

pub fn opening(talk: &Talk, ctx: &Context) -> u16 {
    if talk.bow_done {
        if talk.rupee_count >= 10 {
            if talk.rupee_done {
                CLOSING
            } else {
                RUPEE_PROMPT
            }
        } else if talk.rupee_count == 0 {
            AFTER_BOW
        } else if talk.rupee_count == 1 {
            RUPEE_TALLY_ONE
        } else {
            RUPEE_TALLY_MANY
        }
    } else if !ctx.stories.test(SPOKEN_FLAG) && ctx.sea && talk.room == DRAGON_ROOST {
        if talk.map_done { talk.base } else { MEET_A }
    } else if talk.map_done {
        talk.base
    } else {
        ARRIVE_CHECK
    }
}

pub fn advance(message: u16, talk: &Talk, ctx: &Context) -> Step {
    if message == talk.base {
        if talk.map_done {
            if !ctx.stories.test(SPOKEN_FLAG) && ctx.sea && talk.room == DRAGON_ROOST {
                return Step::Next(FIRST_END);
            }
            return Step::Next(MAP_DONE);
        }
        return Step::Next(BOW_PROMPT);
    }
    match message {
        MEET_A => Step::Next(MEET_B),
        MEET_B => Step::Next(MEET_C),
        MEET_C => Step::Next(MEET_D),
        MEET_D => Step::Done(Some(Scene::MapOpen)),
        FIRST_END => Step::Done(Some(Scene::FirstEnd)),
        ARRIVE_CHECK => {
            let grid = i32::from(talk.room) - 1;
            if grid >= 0 && ctx.arrivals.is_arrived(grid as usize) {
                Step::Next(ARRIVED_RETURN)
            } else {
                Step::Next(MAP_OFFER)
            }
        }
        MAP_OFFER => Step::Done(Some(Scene::MapOpen)),
        ARRIVED_RETURN => Step::Next(talk.base),
        MAP_DONE => Step::Next(MAP_CLOSING),
        BOW_PROMPT => {
            if ctx.minigame_tag && ctx.has_bow && !ctx.timer_active {
                if ctx.stories.test(BOW_EVENT_FLAG) {
                    Step::Next(BOW_OFFER_AGAIN)
                } else {
                    Step::Next(BOW_OFFER)
                }
            } else {
                Step::Next(BOW_DECLINE)
            }
        }
        BOW_OFFER => {
            if ctx.accepted {
                Step::Next(BOW_ACCEPTED)
            } else {
                Step::Next(BOW_REFUSED)
            }
        }
        BOW_OFFER_AGAIN => {
            if ctx.accepted {
                Step::Next(BOW_EVENT)
            } else {
                Step::Next(BOW_REFUSED)
            }
        }
        BOW_ACCEPTED => Step::Next(BOW_EVENT),
        BOW_EVENT => Step::Done(Some(Scene::Bow)),
        BOW_REFUSED => Step::Next(BOW_DECLINE),
        RUPEE_PROMPT => Step::Next(RUPEE_ACCEPTED),
        RUPEE_ACCEPTED => Step::Done(Some(Scene::GetRupee)),
        RUPEE_TALLY_MANY | RUPEE_TALLY_ONE => Step::Next(CLOSING),
        AFTER_BOW => Step::Next(CLOSING),
        _ => Step::Done(None),
    }
}

fn vanish(talk: &mut Talk, stories: &mut StoryBits, event_flags: &mut EventFlags) {
    event_flags.set(TALK_END_FLAG);
    stories.set(SPOKEN_FLAG);
    talk.map_done = false;
    talk.bow_done = false;
    talk.rupee_done = false;
    talk.rupee_count = 0;
    talk.gone = true;
}

fn talking(player: &Player, actor: &Actor) -> bool {
    let dx = player.pos.x - actor.pos.x;
    let dz = player.pos.z - actor.pos.z;
    dx * dx + dz * dz <= TALK_RANGE * TALK_RANGE
}

fn any_minigame_tag(actors: &Query<&Actor>) -> bool {
    actors.iter().any(|actor| {
        actor.name == TAG_ACTOR
            && ((actor.parameters >> MINIGAME_TYPE_SHIFT) & MINIGAME_TYPE_MASK) == MINIGAME_TYPE
    })
}

pub fn control(
    do_intent: Res<DoIntent>,
    health: Res<Health>,
    mut intent: ResMut<MoveIntent>,
    mut dialogue: ResMut<Dialogue>,
    mut stories: ResMut<StoryBits>,
    mut event_flags: ResMut<EventFlags>,
    arrivals: Res<Arrivals>,
    stage: Res<StageName>,
    mut talks: Query<(Entity, &mut Talk, &Actor)>,
    actors: Query<&Actor>,
    mut players: Query<&mut Player>,
) {
    let pressed = do_intent.0 && !dialogue.held;
    dialogue.held = do_intent.0;
    if pressed {
        let ctx = Context {
            stories: *stories,
            arrivals: *arrivals,
            sea: stage.0 == SEA,
            has_bow: false,
            timer_active: false,
            minigame_tag: any_minigame_tag(&actors),
            accepted: false,
        };
        if dialogue.open {
            let Some((_, mut talk, _)) = dialogue
                .speaker
                .and_then(|speaker| talks.get_mut(speaker).ok())
            else {
                dialogue.open = false;
                dialogue.speaker = None;
                return;
            };
            let step = advance(dialogue.message, &talk, &ctx);
            match step {
                Step::Next(message) => dialogue.message = message,
                Step::Done(scene) => match scene {
                    Some(Scene::MapOpen) => {
                        talk.map_done = true;
                        event_flags.set(TALK_END_FLAG);
                        dialogue.message = opening(&talk, &ctx);
                    }
                    Some(Scene::Bow) => {
                        talk.bow_done = true;
                        event_flags.set(TALK_END_FLAG);
                        let message = opening(&talk, &ctx);
                        stories.set(BOW_EVENT_FLAG);
                        dialogue.message = message;
                    }
                    Some(Scene::GetRupee) => {
                        talk.rupee_done = true;
                        event_flags.set(TALK_END_FLAG);
                        dialogue.message = opening(&talk, &ctx);
                    }
                    Some(Scene::FirstEnd) | None => {
                        dialogue.open = false;
                        dialogue.speaker = None;
                        vanish(&mut talk, &mut stories, &mut event_flags);
                    }
                },
            }
        } else if health.units > 0 {
            let Some(player) = players.iter().next() else {
                return;
            };
            for (entity, talk, actor) in &talks {
                if !talk.gone && talking(player, actor) {
                    dialogue.open = true;
                    dialogue.speaker = Some(entity);
                    dialogue.message = opening(&talk, &ctx);
                    break;
                }
            }
        }
    }
    if !dialogue.open {
        return;
    }
    intent.axis = Vec2::ZERO;
    for mut player in &mut players {
        player.speed = 0.0;
    }
}
