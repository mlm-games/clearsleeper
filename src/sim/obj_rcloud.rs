use bevy_ecs::prelude::*;

use super::actor::Actor;
use super::event::{ActiveEvent, EventList};
use super::flags::{Collect, EventFlags, SYMBOL_NAYRU};

pub const OBJ_RCLOUD_ACTORS: [&str; 2] = ["Rcloud", "Ocloud"];

pub fn is_obj_rcloud(name: &str) -> bool {
    OBJ_RCLOUD_ACTORS.contains(&name)
}

pub const ARC: &str = "BVkumo";
pub const MODEL_ATTR: u32 = 0x1100_0222;
pub const HEAP: u32 = 0x5a0;

pub const ACT_WAIT: i32 = 0;
pub const ACT_LIFT_WAIT: i32 = 1;
pub const ACT_LIFT: i32 = 2;

pub const DEMO_NAMES: [&str; 2] = ["demo41", "demo41"];

pub const CLOUD_STEP: f32 = -0.008928572;
pub const CLOUD_END: f32 = -1.0;

pub const EVENT_CLOUD_LIFTED: u32 = 0x3908;

pub const NO_RESOURCE: bool = false;
pub const DEMO_FRAME_LIMIT: f32 = 428.0;
pub const DEMO_MANAGER: bool = false;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum RcloudAct {
    Wait,
    LiftWait,
    Lift,
}

impl RcloudAct {
    pub fn index(self) -> i32 {
        match self {
            RcloudAct::Wait => ACT_WAIT,
            RcloudAct::LiftWait => ACT_LIFT_WAIT,
            RcloudAct::Lift => ACT_LIFT,
        }
    }

    pub fn of(index: i32) -> Self {
        match index {
            ACT_LIFT_WAIT => RcloudAct::LiftWait,
            ACT_LIFT => RcloudAct::Lift,
            _ => RcloudAct::Wait,
        }
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Rcloud {
    pub act: RcloudAct,
    pub demo_index: i32,
    pub progress: f32,
    pub entered: bool,
}

impl Rcloud {
    pub fn spawn(parameters: u32) -> Self {
        Self {
            act: RcloudAct::Wait,
            demo_index: (parameters & 0x1) as i32,
            progress: 0.0,
            entered: false,
        }
    }

    pub fn wanted(&self, flags: &EventFlags, collect: &Collect) -> bool {
        match self.demo_index {
            0 => !flags.test(EVENT_CLOUD_LIFTED),
            1 => !collect.is_symbol(SYMBOL_NAYRU),
            _ => NO_RESOURCE,
        }
    }

    pub fn create_init(&mut self, flags: &EventFlags, collect: &Collect) {
        self.entered = true;
        if self.wanted(flags, collect) {
            self.setup_action(ACT_WAIT);
        }
    }

    pub fn setup_action(&mut self, index: i32) {
        self.act = RcloudAct::of(index);
    }

    pub fn demo_slot(&self, list: &EventList) -> Option<usize> {
        let index = usize::try_from(self.demo_index).ok()?;
        let name = DEMO_NAMES.get(index)?;
        list.event_index(name)
    }

    pub fn wait_act_proc(&mut self, running: bool, active: &ActiveEvent, list: &EventList) {
        if !running {
            return;
        }
        if let Some(slot) = self.demo_slot(list)
            && active.start_check(slot)
        {
            self.setup_action(ACT_LIFT_WAIT);
        }
    }

    pub fn lift_wait_act_proc(&mut self, demo_frame: Option<f32>) {
        if DEMO_MANAGER {
            return;
        }
        if let Some(frame) = demo_frame
            && frame > DEMO_FRAME_LIMIT
        {
            self.setup_action(ACT_LIFT);
        }
    }

    pub fn lift_act_proc(&mut self, flags: &mut EventFlags) -> bool {
        self.progress += CLOUD_STEP;
        if self.progress < CLOUD_END {
            self.progress = CLOUD_END;
            flags.set(EVENT_CLOUD_LIFTED);
            return true;
        }
        false
    }

    pub fn execute(
        &mut self,
        running: bool,
        demo_frame: Option<f32>,
        active: &ActiveEvent,
        list: &EventList,
        flags: &mut EventFlags,
    ) -> bool {
        match self.act {
            RcloudAct::Wait => self.wait_act_proc(running, active, list),
            RcloudAct::LiftWait => self.lift_wait_act_proc(demo_frame),
            RcloudAct::Lift => return self.lift_act_proc(flags),
        }
        false
    }
}

pub fn control(
    list: Res<EventList>,
    active: Res<ActiveEvent>,
    mut flags: ResMut<EventFlags>,
    collect: Res<Collect>,
    info: Option<Res<crate::text::msgproc::MsgGameInfo>>,
    mut clouds: Query<(Entity, &mut Rcloud)>,
    mut commands: Commands,
) {
    let running = info.as_deref().is_some_and(|info| info.event_run_check());
    for (entity, mut cloud) in &mut clouds {
        if !cloud.entered {
            cloud.create_init(&flags, &collect);
            continue;
        }
        if cloud.execute(running, None, &active, &list, &mut flags) {
            let _ = commands.entity(entity).despawn();
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
    };
    let cloud = Rcloud::spawn(source.parameters);
    world.spawn((actor, cloud)).id()
}