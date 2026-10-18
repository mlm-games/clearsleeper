use bevy_ecs::prelude::*;
use glam::Vec3;

use super::event::{ActiveEvent, EventList};
use super::events::{SimEvent, SimEvents};
use super::flags::{EventFlags, StoryBits};

pub const VGNFD_ACTORS: [&str; 1] = ["VgnFD"];

pub const ARC_NAME: &str = "VgnFD";
pub const TIMER_KEY: &str = "Timer";

pub const DOOR_COUNT: usize = 4;
pub const MODEL_COUNT: usize = 5;

pub const DOOR_EV: [u16; DOOR_COUNT] = [0x3904, 0x3902, 0x3901, 0x3A80];
pub const BOSS_EV: [u16; DOOR_COUNT] = [0x3240, 0x3220, 0x3210, 0x3208];
pub const FIN_EV: u16 = 0x3204;

pub const DEMO_NAME: [&str; DOOR_COUNT] = ["4_door_dn", "4_door_mr", "4_door_dc", "4_door_kz"];
pub const FIN_EVENT: &str = "4_door_fin";

pub const ACT_NAMES: [&str; 2] = ["wait0", "wait1"];
pub const BREAK_ACT_NAMES: [&str; 6] = ["wait0", "gannon", "vib0", "vib1", "break", "burst"];

pub const ACT_NONE: i8 = -1;
pub const ACT_WAIT0: i8 = 0;
pub const ACT_GANNON: i8 = 1;
pub const ACT_VIB0: i8 = 2;
pub const ACT_VIB1: i8 = 3;
pub const ACT_BREAK: i8 = 4;
pub const ACT_BURST: i8 = 5;

pub const STATE_WAIT: u8 = 0;
pub const STATE_DEMO: u8 = 1;
pub const STATE_DEMO_FIN: u8 = 2;

pub const NO_INDEX: i32 = -1;

pub const MODEL_INTACT: u8 = 0;
pub const MODEL_BROKEN: u8 = 1;
pub const MODEL_BURST: u8 = 2;

pub const BRK_HOLD_RATE: f32 = 0.0;
pub const BRK_PLAY_RATE: f32 = 1.0;
pub const BRK_REVERSE_RATE: f32 = -1.0;
pub const BTK_HOLD_RATE: f32 = 0.0;
pub const BTK_PLAY_RATE: f32 = 1.0;

pub const TIMER_NONE: i32 = 0;

pub const CULL_MIN: [f32; 3] = [-260.0, -10.0, -50.0];
pub const CULL_MAX: [f32; 3] = [260.0, 510.0, 100.0];

pub const SE_DOOR_LIGHT: u32 = 0x6A55;
pub const SE_READ_RIDDLE: u32 = 0x0806;
pub const SE_BRK_1: u32 = 0x6A1D;
pub const SE_BRK_2: u32 = 0x6A1E;
pub const SE_BRK_3: u32 = 0x6A1F;

fn act_of(list: &EventList, cut: usize, names: &[&str]) -> i8 {
    let Some(name) = list.cut_name(cut) else {
        return ACT_NONE;
    };
    names
        .iter()
        .position(|entry| *entry == name)
        .and_then(|index| i8::try_from(index).ok())
        .unwrap_or(ACT_NONE)
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Vgnfd {
    pub state: u8,
    pub demo_idx: i32,
    pub event_idx: Option<usize>,
    pub model: u8,
    pub timer: i32,
    pub brk_open: [bool; MODEL_COUNT],
    pub brk_rate: [f32; MODEL_COUNT],
    pub btk_rate: f32,
    pub entered: bool,
}

impl Vgnfd {
    fn check_ev_init(stories: &StoryBits, index: i32) -> bool {
        usize::try_from(index)
            .ok()
            .and_then(|slot| DOOR_EV.get(slot))
            .is_some_and(|flag| stories.test(*flag))
    }

    fn check_boss(stories: &StoryBits, index: i32) -> bool {
        usize::try_from(index)
            .ok()
            .and_then(|slot| BOSS_EV.get(slot))
            .is_some_and(|flag| stories.test(*flag))
    }

    fn on_door_ev(stories: &mut StoryBits, index: i32) {
        if let Some(flag) = usize::try_from(index)
            .ok()
            .and_then(|slot| DOOR_EV.get(slot))
        {
            stories.set(*flag);
        }
    }

    fn on_fin(stories: &mut StoryBits) {
        stories.set(FIN_EV);
    }

    fn check_fin(stories: &StoryBits) -> bool {
        DOOR_EV.iter().all(|flag| stories.test(*flag))
    }

    fn get_start_demo_idx(stories: &StoryBits) -> i32 {
        for index in 0..DOOR_COUNT as i32 {
            if !Self::check_ev_init(stories, index) && Self::check_boss(stories, index) {
                return index;
            }
        }
        NO_INDEX
    }

    pub fn spawn(stories: &StoryBits) -> Option<Self> {
        if Self::check_fin(stories) {
            return None;
        }
        let demo_idx = Self::get_start_demo_idx(stories);
        let mut brk_open = [false; MODEL_COUNT];
        for slot in 1..MODEL_COUNT {
            brk_open[slot] = Self::check_ev_init(stories, (slot as i32) - 1);
        }
        Some(Self {
            state: if demo_idx != NO_INDEX {
                STATE_DEMO
            } else {
                STATE_WAIT
            },
            demo_idx,
            event_idx: None,
            model: MODEL_INTACT,
            timer: TIMER_NONE,
            brk_open,
            brk_rate: [BRK_HOLD_RATE; MODEL_COUNT],
            btk_rate: BTK_HOLD_RATE,
            entered: false,
        })
    }

    fn create_init(&mut self, list: &EventList) {
        self.event_idx = None;
        if self.state != STATE_DEMO {
            return;
        }
        if let Some(name) = self.demo_name() {
            self.event_idx = list.event_index(name);
        }
    }

    pub fn cull(&self) -> (Vec3, Vec3) {
        (Vec3::from_array(CULL_MIN), Vec3::from_array(CULL_MAX))
    }

    pub fn drawn(&self) -> bool {
        self.model != MODEL_BURST
    }

    fn demo_name(&self) -> Option<&'static str> {
        usize::try_from(self.demo_idx)
            .ok()
            .and_then(|slot| DEMO_NAME.get(slot))
            .copied()
    }

    fn set_timer(&mut self, list: &EventList, cut: usize) {
        self.timer = list.cut_number(cut, TIMER_KEY).unwrap_or(TIMER_NONE);
    }

    fn demo_accepted(&self, active: &ActiveEvent) -> bool {
        self.event_idx.is_some_and(|slot| active.start_check(slot))
    }

    fn order(&self, list: &EventList, active: &mut ActiveEvent, entity: Entity) {
        let Some(slot) = self.event_idx else {
            return;
        };
        if active.stage_of(slot).is_none() {
            active.order(list, slot, entity);
        }
    }

    fn door_rate(&mut self, index: i32, rate: f32) {
        if let Some(slot) = usize::try_from(index)
            .ok()
            .and_then(|slot| self.brk_rate.get_mut(slot))
        {
            *slot = rate;
        }
    }

    fn state_demo(
        &mut self,
        list: &EventList,
        active: &mut ActiveEvent,
        stories: &mut StoryBits,
        ev_flags: &mut EventFlags,
        events: &mut SimEvents,
        entity: Entity,
    ) {
        if !self.demo_accepted(active) {
            self.order(list, active, entity);
            return;
        }
        if let Some(slot) = self.event_idx
            && active.end_check(slot)
        {
            let _ = active.end(slot);
            if Self::check_fin(stories) {
                self.event_idx = list.event_index(FIN_EVENT);
                self.order(list, active, entity);
                self.state = STATE_DEMO_FIN;
            } else {
                self.state = STATE_WAIT;
            }
            return;
        }
        let Some((cut, advance)) = active.staff_cut(list, ARC_NAME) else {
            return;
        };
        let act = act_of(list, cut, &ACT_NAMES);
        if advance && act == ACT_WAIT0 {
            Self::on_door_ev(stories, self.demo_idx);
            self.set_timer(list, cut);
        }
        if act != ACT_WAIT0 {
            return;
        }
        self.timer -= 1;
        if self.timer > 0 {
            return;
        }
        active.end_staff_cut(list, ev_flags, ARC_NAME);
        self.door_rate(self.demo_idx.saturating_add(1), BRK_PLAY_RATE);
        events.list.push(SimEvent::Se(SE_DOOR_LIGHT));
        if Self::check_fin(stories) {
            events.list.push(SimEvent::Se(SE_READ_RIDDLE));
        }
    }

    fn fin_advance(&mut self, act: i8, events: &mut SimEvents) {
        match act {
            ACT_GANNON => {
                self.brk_rate[0] = BRK_PLAY_RATE;
                self.brk_rate[1..].fill(BRK_REVERSE_RATE);
            }
            ACT_VIB0 => events.list.push(SimEvent::Se(SE_BRK_1)),
            ACT_BREAK => {
                self.model = MODEL_BROKEN;
                self.btk_rate = BTK_PLAY_RATE;
                events.list.push(SimEvent::Se(SE_BRK_2));
            }
            ACT_BURST => {
                self.model = MODEL_BURST;
                events.list.push(SimEvent::Se(SE_BRK_3));
            }
            _ => {}
        }
    }

    fn state_demo_fin(
        &mut self,
        list: &EventList,
        active: &mut ActiveEvent,
        stories: &mut StoryBits,
        ev_flags: &mut EventFlags,
        events: &mut SimEvents,
        entity: Entity,
    ) -> bool {
        if !self.demo_accepted(active) {
            self.order(list, active, entity);
            return false;
        }
        if let Some(slot) = self.event_idx
            && active.end_check(slot)
        {
            let _ = active.end(slot);
            Self::on_fin(stories);
            self.state = STATE_WAIT;
            return true;
        }
        let Some((cut, advance)) = active.staff_cut(list, ARC_NAME) else {
            return false;
        };
        let act = act_of(list, cut, &BREAK_ACT_NAMES);
        if advance {
            self.set_timer(list, cut);
            self.fin_advance(act, events);
        }
        if self.timer > 0 {
            self.timer -= 1;
        }
        if matches!(
            act,
            ACT_WAIT0 | ACT_GANNON | ACT_VIB0 | ACT_VIB1 | ACT_BREAK
        ) && self.timer <= 0
        {
            active.end_staff_cut(list, ev_flags, ARC_NAME);
        }
        false
    }

    fn execute(
        &mut self,
        list: &EventList,
        active: &mut ActiveEvent,
        stories: &mut StoryBits,
        ev_flags: &mut EventFlags,
        events: &mut SimEvents,
        entity: Entity,
    ) -> bool {
        match self.state {
            STATE_DEMO => {
                self.state_demo(list, active, stories, ev_flags, events, entity);
                false
            }
            STATE_DEMO_FIN => self.state_demo_fin(list, active, stories, ev_flags, events, entity),
            _ => false,
        }
    }
}

pub fn is_vgnfd(name: &str) -> bool {
    VGNFD_ACTORS.contains(&name)
}

pub fn control(
    list: Res<EventList>,
    mut active: ResMut<ActiveEvent>,
    mut stories: ResMut<StoryBits>,
    mut ev_flags: ResMut<EventFlags>,
    mut events: ResMut<SimEvents>,
    mut doors: Query<(Entity, &mut Vgnfd)>,
    mut commands: Commands,
) {
    for (entity, mut door) in &mut doors {
        let door = &mut *door;
        if !door.entered {
            door.entered = true;
            door.create_init(&list);
        }
        if door.execute(
            &list,
            &mut active,
            &mut stories,
            &mut ev_flags,
            &mut events,
            entity,
        ) {
            let _ = commands.entity(entity).despawn();
        }
    }
}
