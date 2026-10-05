use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::events::{SimEvent, SimEvents};
use super::flags::Switches;
use super::flags::StoryBits;
use super::player::Player;
use super::walk;

pub const STATUE_ACTOR: &str = "MknjD";

pub const NO_SWITCH: u8 = 0xFF;

pub const STEP_IDLE: u8 = 0;
pub const STEP_WAKER: u8 = 1;
pub const STEP_CHECK: u8 = 2;
pub const STEP_ERROR_WAIT: u8 = 3;
pub const STEP_ERROR_RUN: u8 = 4;
pub const STEP_ERROR_END: u8 = 5;
pub const STEP_DEMO_WAIT: u8 = 6;
pub const STEP_DEMO_RUN: u8 = 7;
pub const STEP_TALKABLE: u8 = 8;
pub const STEP_TALK: u8 = 9;
pub const STEP_CUT: u8 = 10;
pub const STEP_TALK_HOLD: u8 = 11;
pub const STEP_TALK_RUN: u8 = 12;

pub const CULL_HALF: f32 = 400.0;
pub const CULL_TOP: f32 = 405.0;

pub const TALK_RANGE: f32 = 800.0;
pub const BACK_LIMIT: i16 = 0x4000;

pub const ITEM_EARTH_LYRIC: u8 = 0x70;
pub const ITEM_WIND_ARIA: u8 = 0x71;

pub const FLAG_EARTH: u16 = 0x2920;
pub const FLAG_WIND: u16 = 0x2910;

pub const SE_RIDDLE: u32 = 0x0806;
pub const SE_GATE_CREAK: u32 = 0x69A8;
pub const SE_GATE_LIGHT: u32 = 0x69A9;
pub const SE_GATE_BREAK: u32 = 0x69AA;

pub const BGM_TAKT_MEDRI: u32 = 0x8000_0022;
pub const BGM_TAKT_MAKORE: u32 = 0x8000_0021;
pub const BGM_D_EARTH: u32 = 0x8000_002D;
pub const BGM_D_WIND: u32 = 0x8000_002F;

pub const WAKING_TIMER: i16 = 0x2B;
pub const REFRESH_TIMER: i16 = 0x0E;

pub const SPLIT_FRAMES: i32 = 60;
pub const LIGHT_FRAME: i32 = 30;
pub const SHATTER_FRAME: i32 = 160;
pub const DONE_FRAME: i32 = 255;
pub const FADE_PIVOT: i32 = 0xA0;
pub const FADE_SPAN: f64 = 100.0;
pub const OPAQUE: u8 = 0xFF;
pub const TRANSPARENT: u8 = 0;

const SWITCH_SHIFT: u32 = 0;
const SWITCH_MASK: u32 = 0xFF;
const TYPE_SHIFT: u32 = 16;
const TYPE_MASK: u32 = 0x3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Idle,
    Waker,
    Check,
    ErrorWait,
    ErrorRun,
    ErrorEnd,
    DemoWait,
    DemoRun,
    Talkable,
    Talk,
    Cut,
    TalkHold,
    TalkRun,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Statue {
    pub switch_no: u8,
    pub wind: bool,
    pub step: u8,
    pub break_timer: i32,
    pub alpha: u8,
    pub wakes: i16,
    pub done: bool,
}

impl Statue {
    pub fn spawn(parameters: u32) -> Self {
        let wind = (parameters >> TYPE_SHIFT) & TYPE_MASK != 0;
        Self {
            switch_no: ((parameters >> SWITCH_SHIFT) & SWITCH_MASK) as u8,
            wind,
            step: STEP_IDLE,
            break_timer: 0,
            alpha: OPAQUE,
            wakes: if wind { WAKING_TIMER } else { REFRESH_TIMER },
            done: false,
        }
    }

    pub fn item(&self) -> u8 {
        if self.wind {
            ITEM_WIND_ARIA
        } else {
            ITEM_EARTH_LYRIC
        }
    }

    pub fn flag(&self) -> u16 {
        if self.wind {
            FLAG_WIND
        } else {
            FLAG_EARTH
        }
    }

    pub fn cull(&self) -> (f32, f32) {
        (CULL_HALF, CULL_TOP)
    }

    pub fn song(&self) -> u32 {
        if self.wind {
            BGM_TAKT_MAKORE
        } else {
            BGM_TAKT_MEDRI
        }
    }

    pub fn awakening(&self) -> u32 {
        if self.wind {
            BGM_D_WIND
        } else {
            BGM_D_EARTH
        }
    }

    pub fn enter(&mut self, step: Step) {
        self.step = match step {
            Step::Idle => STEP_IDLE,
            Step::Waker => STEP_WAKER,
            Step::Check => STEP_CHECK,
            Step::ErrorWait => STEP_ERROR_WAIT,
            Step::ErrorRun => STEP_ERROR_RUN,
            Step::ErrorEnd => STEP_ERROR_END,
            Step::DemoWait => STEP_DEMO_WAIT,
            Step::DemoRun => STEP_DEMO_RUN,
            Step::Talkable => STEP_TALKABLE,
            Step::Talk => STEP_TALK,
            Step::Cut => STEP_CUT,
            Step::TalkHold => STEP_TALK_HOLD,
            Step::TalkRun => STEP_TALK_RUN,
        };
    }

    pub fn resting(&self) -> bool {
        self.step == STEP_IDLE
    }

    pub fn granted(&self, stories: &StoryBits) -> bool {
        stories.test(self.flag())
    }

    pub fn shatter(&mut self) -> bool {
        self.break_timer += 1;
        self.fade();
        self.break_timer == DONE_FRAME
    }

    fn fade(&mut self) {
        let frames = self.break_timer;
        self.alpha = if frames < SPLIT_FRAMES {
            OPAQUE
        } else if frames < SHATTER_FRAME {
            let scaled = f64::from(FADE_PIVOT - frames) * 255.0 / FADE_SPAN;
            if scaled >= 255.0 {
                OPAQUE
            } else {
                scaled as u8
            }
        } else {
            TRANSPARENT
        };
    }

    pub fn cracking(&self) -> bool {
        self.break_timer > 0 && !self.done
    }

    pub fn cue(&self) -> Option<u32> {
        match self.break_timer {
            1 => Some(SE_GATE_CREAK),
            LIGHT_FRAME => Some(SE_GATE_LIGHT),
            SHATTER_FRAME => Some(SE_GATE_BREAK),
            _ => None,
        }
    }
}

pub fn is_statue(name: &str) -> bool {
    name == STATUE_ACTOR
}

pub fn in_reach(base: Vec3, player: &Player) -> bool {
    let gap = base - player.pos;
    gap.x * gap.x + gap.z * gap.z < TALK_RANGE * TALK_RANGE
}

pub fn behind(base: Vec3, facing: i16, player: &Player) -> bool {
    let want = walk::to_s16((base.x - player.pos.x).atan2(base.z - player.pos.z));
    let diff = want.wrapping_sub(facing);
    diff < -BACK_LIMIT || diff > BACK_LIMIT
}

pub fn latched(statue: &Statue, switches: &Switches) -> bool {
    statue.switch_no != NO_SWITCH && switches.test(statue.switch_no)
}

pub fn control(
    mut stories: ResMut<StoryBits>,
    players: Query<&Player>,
    mut statues: Query<(Entity, &Actor, &mut Statue)>,
    mut switches: ResMut<Switches>,
    mut events: ResMut<SimEvents>,
    mut commands: Commands,
) {
    let player = players.iter().next().copied();
    for (entity, actor, mut statue) in &mut statues {
        if statue.cracking() {
            if statue.shatter() {
                statue.done = true;
                events.list.push(SimEvent::Bgm(statue.awakening()));
            }
            if let Some(cue) = statue.cue() {
                events.list.push(SimEvent::Se(cue));
            }
            continue;
        }
        if statue.done {
            let _ = commands.entity(entity).despawn();
            continue;
        }
        if latched(&*statue, &switches) {
            statue.enter(Step::Cut);
            statue.shatter();
            events.list.push(SimEvent::Se(SE_RIDDLE));
            continue;
        }
        if statue.wakes > 1 {
            statue.wakes -= 1;
        }
        if statue.wakes == 1 {
            statue.wakes = 0;
            events.list.push(SimEvent::Bgm(statue.song()));
        }
        if !statue.resting() || statue.granted(&stories) {
            continue;
        }
        let Some(player) = player else {
            continue;
        };
        if in_reach(actor.pos, &player)
            && behind(actor.pos, walk::to_s16(player.face), &player)
        {
            stories.set(statue.flag());
            statue.enter(Step::Talkable);
        }
    }
}
