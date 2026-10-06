use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::Vec3;

use super::actor::Actor;
use super::event::{ActiveEvent, EventList, Stage};
use super::event_manager::EventManager;
use super::events::{SimEvent, SimEvents};
use super::flags::{Collect, EventFlags};
use super::inventory::{self, Equipped};
use super::player::Player;
use super::transition::{NextStage, Target};
use super::vibration::Vibration;
use super::walk;
use crate::assets::MsgArchives;
use crate::text::msgbox::Msg2Box;
use crate::text::msgbox1::Msg1Box;
use crate::text::msgget::MsgArchive;
use crate::text::msgproc::{
    FPC_M_ERROR_PROCESS_ID, FPC_NM_MSG, FPC_NM_MSG2, MsgGameInfo, MsgProcs,
    fop_msgm_message_set_actor,
};
use crate::text::{
    STATUS_BOX_CLOSED, STATUS_MSG_CONTINUES, STATUS_MSG_DESTROYED, STATUS_MSG_DISPLAYED,
    STATUS_MSG_ENDS,
};

pub const OBJ_DOGUU_ACTORS: [&str; 4] = ["Doguu", "MegamiD", "MegamiF", "MegamiN"];

pub fn is_obj_doguu(name: &str) -> bool {
    OBJ_DOGUU_ACTORS.contains(&name)
}

pub const ARC: &str = "Doguu";

pub const STAFF_DOGUU: &str = "Doguu";

pub const EVENT_DOGUU_DEMO1: &str = "DOGUU_DEMO1";
pub const EVENT_DOGUU_DEMO2: &str = "DOGUU_DEMO2";
pub const EVENT_DOGUU_DEMO3: &str = "DOGUU_DEMO3";
pub const EVENT_MEGAMI_DEMO: &str = "MEGAMI_DEMO";

pub const STAFF_GOAL_KEY: &str = "Posion";
pub const STAFF_ANGLE_KEY: &str = "angle";
pub const STAFF_TIMER_KEY: &str = "Timer";

pub const CUT_NAME_TABLE: [&str; 9] = [
    "SETGOAL",
    "SETBALL",
    "SETANGLE",
    "EYEFLASH",
    "MESSAGE_KOKOE",
    "MESSAGE_OITA",
    "QUAKE",
    "JDEMO",
    "SETLIGHT",
];

pub const PARTICLE_GLOW_NAMES: [&str; 3] = [
    "ID_IT_SN_SINJYU_GLOWD00",
    "ID_IT_SN_SINJYU_GLOWF00",
    "ID_IT_SN_SINJYU_GLOWN00",
];

pub const PARTICLE_GLOW_DIN: u32 = 0;
pub const PARTICLE_GLOW_FARORE: u32 = 1;
pub const PARTICLE_GLOW_NAYRU: u32 = 2;

pub const JDEMO_STAGE: &str = "ADMumi";

pub const CUT_SETGOAL: i8 = 0;
pub const CUT_SETBALL: i8 = 1;
pub const CUT_SETANGLE: i8 = 2;
pub const CUT_EYEFLASH: i8 = 3;
pub const CUT_MESSAGE_KOKOE: i8 = 4;
pub const CUT_MESSAGE_OITA: i8 = 5;
pub const CUT_QUAKE: i8 = 6;
pub const CUT_JDEMO: i8 = 7;
pub const CUT_SETLIGHT: i8 = 8;

pub const STATE_WAIT: i32 = 0;
pub const STATE_DEMO1_REQUEST: i32 = 1;
pub const STATE_DEMO1_RUN: i32 = 2;
pub const STATE_DEMO1_END: i32 = 3;
pub const STATE_DEMO2_REQUEST: i32 = 4;
pub const STATE_DEMO2_RUN: i32 = 5;
pub const STATE_DEMO2_END: i32 = 6;
pub const STATE_DEMO3_REQUEST: i32 = 7;
pub const STATE_DEMO3_RUN: i32 = 8;
pub const STATE_FINISH: i32 = 9;
pub const STATE_DONE: i32 = 10;
pub const STATE_MEGAMI_REQUEST: i32 = 11;
pub const STATE_MEGAMI_RUN: i32 = 12;
pub const STATE_RESTING: i32 = 14;

pub const ACT_IDX_NONE: i8 = -1;
pub const STAFF_IDX_NONE: i32 = -1;
pub const EVENT_INFO_NONE: u8 = 0xFF;
pub const EVENT_IDX_NONE: i32 = -1;

pub const KIND_DIN: i32 = 0;
pub const KIND_FARORE: i32 = 1;
pub const KIND_NAYRU: i32 = 2;
pub const BYTE_MASK: u32 = 0xFF;

pub const ARGUMENT_TABLE: [i32; 4] = [0, 1, 2, 3];

pub const EVENT_PLACED_DINS_PEARL: u32 = 0x1480;
pub const EVENT_PLACED_FARORES_PEARL: u32 = 0x1440;
pub const EVENT_PLACED_NAYRUS_PEARL: u32 = 0x1410;
pub const EVENT_UNK_1E40: u32 = 0x1E40;

pub const MSG_STATUS_NONE: u8 = 0xFF;

pub const MSG_GREAT_FAIRY: u32 = 3821;
pub const MSG_FIRST: u32 = 3822;
pub const MSG_SECOND: u32 = 3823;
pub const MSG_THIRD: u32 = 3824;
pub const MSG_SECOND_CHANCE: u32 = 3825;
pub const MSG_LAST_CHANCE: u32 = 3826;

pub const SE_LK_PUT_PEARL: u32 = 0x288C;
pub const SE_OBJ_SDOLL_EYE_PULSE: u32 = 0x6A31;
pub const BGM_GET_PEARL: u32 = 0x8000_004F;

pub const CYL_RADIUS: f32 = 200.0;
pub const CYL_RADIUS_HELD: f32 = 50.0;
pub const CYL_RADIUS_DONE: f32 = 30.0;
pub const CYL_HEIGHT: f32 = 200.0;

pub const CO_SET: u32 = 0x01;
pub const CO_IS_OTHER: u32 = 0x08;
pub const CO_VS_ENEMY: u32 = 0x10;
pub const CO_SRC_SPRM: u32 = CO_SET | CO_IS_OTHER | CO_VS_ENEMY;

pub const TG_IS_PLAYER: u32 = 0x04;
pub const TG_SRC_SPRM: u32 = TG_IS_PLAYER;

pub const STTS_WEIGHT: i32 = 0xFF;
pub const STTS_PARAM: i32 = 0xFF;

pub const LIGHT_COLOR: [[u16; 3]; 3] = [
    [0x00C8, 0x0032, 0x0032],
    [0x0032, 0x00C8, 0x0032],
    [0x0032, 0x0032, 0x00C8],
];

pub const LIGHT_RND_BASE: f32 = 1.0;
pub const LIGHT_RND_RANGE: f32 = 0.1;
pub const LIGHT_SCALE: f32 = 0.5;
pub const LIGHT_MAX_STEP: f32 = 0.05;
pub const LIGHT_POWER_SCALE: f32 = 300.0;
pub const LIGHT_FLUCTUATION: f32 = 100.0;

pub const EYE_OFFSET: f32 = 125.0;

pub const CULL_MIN: [f32; 3] = [-600.0, -0.0, -600.0];
pub const CULL_MAX: [f32; 3] = [600.0, 400.0, 600.0];
pub const CULL_FAR: f32 = 1.0;

pub const SETLIGHT_END: i32 = 200;
pub const SETLIGHT_SEG1: i32 = 21;
pub const SETLIGHT_SEG1_SLOPE: f32 = 0.01875;
pub const SETLIGHT_SEG1_BASE: f32 = 1.0;
pub const SETLIGHT_SEG2: i32 = 41;
pub const SETLIGHT_SEG2_SLOPE: f32 = 0.0125;
pub const SETLIGHT_SEG2_BASE: f32 = 0.625;
pub const SETLIGHT_SEG3: i32 = 61;
pub const SETLIGHT_SEG3_SLOPE: f32 = 0.00625;
pub const SETLIGHT_SEG3_BASE: f32 = 0.375;
pub const SETLIGHT_SEG4_LO: i32 = 130;
pub const SETLIGHT_SEG4_HI: i32 = 150;
pub const SETLIGHT_SEG4_SLOPE: f32 = 0.00625;
pub const SETLIGHT_SEG4_BASE: f32 = 0.25;
pub const SETLIGHT_SEG5_LO: i32 = 150;
pub const SETLIGHT_SEG5_HI: i32 = 170;
pub const SETLIGHT_SEG5_SLOPE: f32 = 0.0125;
pub const SETLIGHT_SEG5_BASE: f32 = 0.375;
pub const SETLIGHT_SEG6_LO: i32 = 170;
pub const SETLIGHT_SEG6_HI: i32 = 190;
pub const SETLIGHT_SEG6_SLOPE: f32 = 0.01875;
pub const SETLIGHT_SEG6_BASE: f32 = 0.625;
pub const SETLIGHT_PARTICLE_TICK: i32 = 40;
pub const SETLIGHT_GLOW_TICK: i32 = 125;

pub const QUAKE_PATTERN: u32 = 0x0010_FFEE;
pub const QUAKE_FLAGS: i32 = 63;
pub const QUAKE_COORD: Vec3 = Vec3::new(0.0, 1.0, 0.0);

pub const ORDER_HIND: u16 = 0xFFFF;
pub const ORDER_FLAG: u16 = 0;

pub const JDEMO_LAYER: i8 = 8;
pub const JDEMO_ROOM: u8 = 0;

pub const BRK_STEP: f32 = 1.0;
pub const BRK_PULSE_FRAME: f32 = 1.0;

pub const CRYSTAL_JOINT: u32 = 1;

pub const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
pub const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
pub const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];

#[derive(Resource, Debug, Clone, Copy, Default, PartialEq)]
pub struct LightInfluence {
    pub pos: Vec3,
    pub r: u16,
    pub g: u16,
    pub b: u16,
    pub power: f32,
    pub fluctuation: f32,
}

#[derive(Resource, Debug, Default)]
pub struct PointLights {
    pub live: Vec<LightInfluence>,
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct AllColRatio {
    pub ratio: f32,
}

impl Default for AllColRatio {
    fn default() -> Self {
        Self { ratio: 1.0 }
    }
}

#[derive(Resource, Debug, Default)]
pub struct ParticleSets {
    pub live: Vec<(u32, Vec3, [i16; 3])>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cyl {
    pub centre: Vec3,
    pub radius: f32,
    pub height: f32,
    pub co_sprm: u32,
    pub tg_sprm: u32,
    pub at_type: u32,
    pub weight: i32,
    pub param: i32,
    pub registered: bool,
    pub co_hit: bool,
    pub co_hit_obj: bool,
}

impl Default for Cyl {
    fn default() -> Self {
        Self::SRC
    }
}

impl Cyl {
    const SRC: Cyl = Cyl {
        centre: Vec3::ZERO,
        radius: CYL_RADIUS,
        height: CYL_HEIGHT,
        co_sprm: CO_SRC_SPRM,
        tg_sprm: TG_SRC_SPRM,
        at_type: 0,
        weight: STTS_WEIGHT,
        param: STTS_PARAM,
        registered: false,
        co_hit: false,
        co_hit_obj: false,
    };

    pub fn set(&mut self) {
        *self = Cyl::SRC;
    }

    pub fn set_r(&mut self, radius: f32) {
        self.radius = radius;
    }

    pub fn set_c(&mut self, centre: Vec3) {
        self.centre = centre;
    }

    pub fn stts_init(&mut self) {
        self.weight = STTS_WEIGHT;
        self.param = STTS_PARAM;
    }

    pub fn co_set(&self) -> bool {
        self.co_sprm & CO_SET != 0
    }

    pub fn co_is_other(&self) -> bool {
        self.co_sprm & CO_IS_OTHER != 0
    }

    pub fn co_vs_enemy(&self) -> bool {
        self.co_sprm & CO_VS_ENEMY != 0
    }

    pub fn tg_is_player(&self) -> bool {
        self.tg_sprm & TG_IS_PLAYER != 0
    }

    pub fn chk_co_hit(&self) -> bool {
        self.co_hit
    }

    pub fn get_co_hit_obj(&self) -> bool {
        self.co_hit_obj
    }

    pub fn register(&mut self) {
        self.registered = true;
    }

    pub fn covers(&self, point: Vec3) -> bool {
        let dx = point.x - self.centre.x;
        let dz = point.z - self.centre.z;
        if dx * dx + dz * dz > self.radius * self.radius {
            return false;
        }
        (point.y - self.centre.y).abs() <= self.height * 0.5
    }

    pub fn probe_co(&mut self, player: Option<Vec3>) {
        let accepts = self.co_set() && (self.co_is_other() || self.co_vs_enemy());
        let targetable = self.tg_is_player();
        let hit = match player {
            Some(point) => accepts && targetable && self.covers(point),
            None => false,
        };
        self.co_hit = hit;
        self.co_hit_obj = hit;
    }
}

fn rnd(seed: &mut [i32; 3]) -> f32 {
    for lane in 0..3 {
        seed[lane] = (seed[lane] * RND_MUL[lane]) % RND_MOD[lane];
    }
    let sum = (seed[0]) as f32 / RND_SCALE[0]
        + (seed[1]) as f32 / RND_SCALE[1]
        + (seed[2]) as f32 / RND_SCALE[2];
    (sum % 1.0).abs()
}

fn add_calc2(value: &mut f32, target: f32, scale: f32, max_step: f32) {
    if *value != target {
        let mut step = scale * (target - *value);
        if step > max_step {
            step = max_step;
        } else if step < -max_step {
            step = -max_step;
        }
        *value += step;
    }
}

fn rotate_y(angle_y: i16, point: Vec3) -> Vec3 {
    let rad = walk::from_s16(angle_y);
    let (sin, cos) = rad.sin_cos();
    Vec3::new(
        point.x * cos + point.z * sin,
        point.y,
        point.z * cos - point.x * sin,
    )
}

fn pearl_of(kind: i32) -> u8 {
    if kind == KIND_DIN {
        inventory::ITEM_PEARL_DIN
    } else if kind == KIND_FARORE {
        inventory::ITEM_PEARL_FARORE
    } else {
        inventory::ITEM_PEARL_NAYRU
    }
}

fn pearl_event_of(kind: i32) -> u32 {
    if kind == KIND_DIN {
        EVENT_PLACED_DINS_PEARL
    } else if kind == KIND_FARORE {
        EVENT_PLACED_FARORES_PEARL
    } else {
        EVENT_PLACED_NAYRUS_PEARL
    }
}

fn light_row_of(kind: i32) -> [u16; 3] {
    match kind {
        KIND_DIN => LIGHT_COLOR[0],
        KIND_FARORE => LIGHT_COLOR[1],
        KIND_NAYRU => LIGHT_COLOR[2],
        _ => [0; 3],
    }
}

fn glow_of(kind: i32) -> u32 {
    if kind == KIND_DIN {
        PARTICLE_GLOW_DIN
    } else if kind == KIND_FARORE {
        PARTICLE_GLOW_FARORE
    } else {
        PARTICLE_GLOW_NAYRU
    }
}

fn event_slot(index: i32) -> usize {
    usize::try_from(index).unwrap_or(0)
}

fn has_event(index: i32) -> bool {
    index != EVENT_IDX_NONE && index >= 0
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct ObjDoguu {
    pub health: i32,
    pub max_health: i32,
    pub argument: i32,
    pub kind: i32,
    pub item_no: u8,
    pub entered: bool,
    pub head1_jnt_no: u32,
    pub crystal_joint: u32,
    pub has_crystal: bool,
    pub body: bool,
    pub crystal_on: bool,
    pub glow: bool,
    pub demo1: i32,
    pub demo2: i32,
    pub demo3: i32,
    pub megami: i32,
    pub state: i32,
    pub act: i8,
    pub goal: Vec3,
    pub timer: i32,
    pub fade: i32,
    pub all_col_ratio: f32,
    pub shape: u32,
    pub glow_pos: Vec3,
    pub influence: LightInfluence,
    pub light_power: f32,
    pub cyl: Cyl,
    pub brk_frame: f32,
    pub bck_frame: f32,
    pub msg_no: u32,
    pub msg_pid: u32,
    pub msg_found: bool,
    pub eye_pos: Vec3,
    pub scale: f32,
    pub seed: [i32; 3],
    pub demo_actor_id: u32,
    pub demo_shape: u32,
    pub demo_angle: [i16; 3],
    pub demo_pos: Vec3,
    pub demo_frame: f32,
}

impl ObjDoguu {
    pub fn spawn(parameters: u32) -> Self {
        let _ = parameters & BYTE_MASK;
        Self {
            health: 0,
            max_health: 0,
            argument: EVENT_IDX_NONE,
            kind: 0,
            item_no: inventory::ITEM_PEARL_DIN,
            entered: false,
            head1_jnt_no: 0,
            crystal_joint: 0,
            has_crystal: false,
            body: false,
            crystal_on: false,
            glow: false,
            demo1: EVENT_IDX_NONE,
            demo2: EVENT_IDX_NONE,
            demo3: EVENT_IDX_NONE,
            megami: EVENT_IDX_NONE,
            state: STATE_WAIT,
            act: ACT_IDX_NONE,
            goal: Vec3::ZERO,
            timer: 0,
            fade: 0,
            all_col_ratio: 1.0,
            shape: 0,
            glow_pos: Vec3::ZERO,
            influence: LightInfluence::default(),
            light_power: 1.0,
            cyl: Cyl::SRC,
            brk_frame: 0.0,
            bck_frame: 0.0,
            msg_no: 0,
            msg_pid: FPC_M_ERROR_PROCESS_ID,
            msg_found: false,
            eye_pos: Vec3::ZERO,
            scale: 1.0,
            seed: [0; 3],
            demo_actor_id: 0,
            demo_shape: 0,
            demo_angle: [0; 3],
            demo_pos: Vec3::ZERO,
            demo_frame: 0.0,
        }
    }

    pub fn argument_of(name: &str) -> i32 {
        let slot = OBJ_DOGUU_ACTORS.iter().position(|entry| *entry == name);
        match slot {
            Some(slot) => ARGUMENT_TABLE.get(slot).copied().unwrap_or(0),
            None => 0,
        }
    }

    pub fn create(&mut self, argument: i32, parameters: u32, pos: Vec3, scale: f32) {
        self.argument = argument;
        if argument >= 1 {
            self.kind = argument - 1;
        } else {
            self.kind = (parameters & BYTE_MASK) as i32;
        }
        self.scale = scale;
        self.eye_pos = Vec3::new(pos.x, pos.y + EYE_OFFSET, pos.z);
        self.crystal_joint = CRYSTAL_JOINT;
        self.entered = true;
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn cull_far(&self) -> f32 {
        CULL_FAR
    }

    pub fn drawn(&self) -> bool {
        true
    }

    pub fn create_init(&mut self, has_pearl: bool, flags: &EventFlags, manager: &mut EventManager) {
        self.item_no = pearl_of(self.kind);
        self.cyl.stts_init();
        self.cyl.set();
        if !has_pearl {
            self.cyl.set_r(CYL_RADIUS_HELD);
        }
        if self.argument >= 1 {
            self.bck_frame = 0.0;
            self.has_crystal = true;
            self.body = false;
            self.crystal_on = true;
            self.state = STATE_RESTING;
        } else if flags.test(EVENT_PLACED_DINS_PEARL)
            && flags.test(EVENT_PLACED_FARORES_PEARL)
            && flags.test(EVENT_PLACED_NAYRUS_PEARL)
        {
            self.bck_frame = 0.0;
            self.has_crystal = true;
            self.body = true;
            self.crystal_on = false;
            self.state = STATE_RESTING;
        } else {
            self.body = false;
            let placed = flags.test(pearl_event_of(self.kind));
            if placed {
                self.crystal_on = true;
                self.has_crystal = true;
                self.state = STATE_RESTING;
            } else {
                self.crystal_on = false;
                self.has_crystal = false;
                self.state = 0;
            }
        }
        self.demo1 = manager.get_event_idx(Some(EVENT_DOGUU_DEMO1), EVENT_INFO_NONE);
        self.demo2 = manager.get_event_idx(Some(EVENT_DOGUU_DEMO2), EVENT_INFO_NONE);
        self.demo3 = manager.get_event_idx(Some(EVENT_DOGUU_DEMO3), EVENT_INFO_NONE);
        self.megami = manager.get_event_idx(Some(EVENT_MEGAMI_DEMO), EVENT_INFO_NONE);
        self.light_power = 1.0;
        if self.has_crystal {
            self.glow = true;
        } else {
            self.glow = false;
        }
        self.all_col_ratio = 1.0;
    }

    pub fn set_point_light(&mut self) {
        add_calc2(
            &mut self.light_power,
            rnd(&mut self.seed) * LIGHT_RND_RANGE + LIGHT_RND_BASE,
            LIGHT_SCALE,
            LIGHT_MAX_STEP,
        );
        self.influence.pos = self.glow_pos;
        let row = light_row_of(self.kind);
        self.influence.r = row[0];
        self.influence.g = row[1];
        self.influence.b = row[2];
        self.influence.power = (self.light_power * LIGHT_POWER_SCALE) as i16 as f32;
        self.influence.fluctuation = LIGHT_FLUCTUATION;
    }

    pub fn get_finish_event_count(&self, flags: &EventFlags) -> i32 {
        let mut count = 0;
        if flags.test(EVENT_PLACED_DINS_PEARL) {
            count += 1;
        }
        if flags.test(EVENT_PLACED_FARORES_PEARL) {
            count += 1;
        }
        if flags.test(EVENT_PLACED_NAYRUS_PEARL) {
            count += 1;
        }
        count
    }

    pub fn set_finish_my_event(&self, flags: &mut EventFlags) {
        flags.set(pearl_event_of(self.kind));
    }

    pub fn get_msg(&self) -> u32 {
        if self.act == CUT_MESSAGE_KOKOE {
            MSG_GREAT_FAIRY
        } else if self.kind == KIND_DIN {
            MSG_FIRST
        } else if self.kind == KIND_FARORE {
            MSG_SECOND
        } else {
            MSG_THIRD
        }
    }

    pub fn next_msg_status(&mut self, flags: &EventFlags) -> u8 {
        let mut message_status = STATUS_MSG_CONTINUES;
        match self.msg_no {
            MSG_FIRST | MSG_SECOND | MSG_THIRD => {
                let finished = self.get_finish_event_count(flags);
                if finished == 0 {
                    self.msg_no = MSG_SECOND_CHANCE;
                } else if finished == 1 {
                    self.msg_no = MSG_LAST_CHANCE;
                } else {
                    message_status = STATUS_MSG_ENDS;
                }
            }
            _ => {
                message_status = STATUS_MSG_ENDS;
            }
        }
        message_status
    }

    fn talk(&mut self, text: &mut Text<'_>, flags: &EventFlags) -> u8 {
        let mut status = MSG_STATUS_NONE;
        if self.msg_pid == FPC_M_ERROR_PROCESS_ID {
            self.msg_no = self.get_msg();
            self.msg_pid = text.message_set_actor(self.msg_no, self.eye_pos);
            self.msg_found = false;
        } else if self.msg_found {
            status = text.status(self.msg_pid);
            if status == STATUS_MSG_DISPLAYED {
                let next = self.next_msg_status(flags);
                text.set_status(self.msg_pid, next);
                if next == STATUS_MSG_CONTINUES {
                    self.message_set(text);
                }
            } else if status == STATUS_BOX_CLOSED {
                text.set_status(self.msg_pid, STATUS_MSG_DESTROYED);
                self.msg_pid = FPC_M_ERROR_PROCESS_ID;
            }
        } else {
            self.msg_found = text.alive(self.msg_pid);
        }
        status
    }

    fn message_set(&mut self, text: &mut Text<'_>) {
        self.msg_pid = text.message_set(self.msg_no);
    }

    fn set_goal(&mut self, staff_pos: Vec3, angle_y: i16, manager: &mut EventManager) {
        let goal = rotate_y(angle_y, staff_pos);
        self.goal = goal;
        manager.goal = goal;
    }

    fn set_player_angle(&self, angle_y: i16, staff_angle: i32, player: &mut Player) {
        player.yaw = walk::from_s16(angle_y.wrapping_add(staff_angle as i16));
    }

    fn set_jdemo(&self, next: &mut NextStage) {
        next.exit = None;
        next.target = Some(Target {
            stage: JDEMO_STAGE.to_owned(),
            room: JDEMO_ROOM,
            start: self.kind as u8,
        });
    }

    fn fade_ratio(&mut self) {
        let mut i_var2 = self.fade;
        if i_var2 >= SETLIGHT_END {
            self.all_col_ratio = 1.0;
        } else {
            self.fade += 1;
        }
        i_var2 = self.fade;
        if i_var2 < SETLIGHT_SEG1 {
            self.all_col_ratio = SETLIGHT_SEG1_BASE - i_var2 as f32 * SETLIGHT_SEG1_SLOPE;
        } else if i_var2 < SETLIGHT_SEG2 {
            self.all_col_ratio = SETLIGHT_SEG2_BASE - (i_var2 + -20) as f32 * SETLIGHT_SEG2_SLOPE;
        } else if i_var2 < SETLIGHT_SEG3 {
            self.all_col_ratio = SETLIGHT_SEG3_BASE - (i_var2 + -40) as f32 * SETLIGHT_SEG3_SLOPE;
        } else if (i_var2 >= SETLIGHT_SEG4_LO) && (i_var2 < SETLIGHT_SEG4_HI) {
            self.all_col_ratio =
                (i_var2 - SETLIGHT_SEG4_LO) as f32 * SETLIGHT_SEG4_SLOPE + SETLIGHT_SEG4_BASE;
        } else if (self.fade >= SETLIGHT_SEG5_LO) && (self.fade < SETLIGHT_SEG5_HI) {
            self.all_col_ratio =
                (i_var2 - SETLIGHT_SEG5_LO) as f32 * SETLIGHT_SEG5_SLOPE + SETLIGHT_SEG5_BASE;
        } else if (i_var2 >= SETLIGHT_SEG6_LO) && (i_var2 <= SETLIGHT_SEG6_HI) {
            self.all_col_ratio =
                (i_var2 - SETLIGHT_SEG6_LO) as f32 * SETLIGHT_SEG6_SLOPE + SETLIGHT_SEG6_BASE;
        }
    }
}

pub struct Text<'a> {
    pub procs: &'a mut MsgProcs,
    pub info: &'a mut MsgGameInfo,
    pub one: &'a mut Msg1Box,
    pub two: &'a mut Msg2Box,
    pub archive: Option<&'a dyn MsgArchive>,
}

impl Text<'_> {
    fn box_name(&self, pid: u32) -> Option<u16> {
        self.procs.search_by_id(pid).map(|proc| proc.name)
    }

    pub fn alive(&self, pid: u32) -> bool {
        self.box_name(pid).is_some()
    }

    pub fn status(&mut self, pid: u32) -> u8 {
        match self.box_name(pid) {
            Some(FPC_NM_MSG) => self.one.status,
            Some(FPC_NM_MSG2) => self.two.status,
            _ => MSG_STATUS_NONE,
        }
    }

    pub fn set_status(&mut self, pid: u32, status: u8) {
        match self.box_name(pid) {
            Some(FPC_NM_MSG) => self.one.status = status,
            Some(FPC_NM_MSG2) => self.two.status = status,
            _ => {}
        }
    }

    pub fn message_set(&mut self, msg_no: u32) -> u32 {
        let Some(archive) = self.archive else {
            return FPC_M_ERROR_PROCESS_ID;
        };
        fop_msgm_message_set_actor(self.procs, self.info, archive, 0, msg_no, None, Vec3::ZERO)
    }

    pub fn message_set_actor(&mut self, msg_no: u32, eye_pos: Vec3) -> u32 {
        let Some(archive) = self.archive else {
            return FPC_M_ERROR_PROCESS_ID;
        };
        fop_msgm_message_set_actor(
            self.procs,
            self.info,
            archive,
            0,
            msg_no,
            None,
            eye_pos,
        )
    }
}

pub struct Ctx<'a> {
    pub entity: Entity,
    pub pos: Vec3,
    pub angle: [i16; 3],
    pub has_pearl: bool,
    pub flags: &'a mut EventFlags,
    pub list: &'a EventList,
    pub active: &'a mut ActiveEvent,
    pub manager: &'a mut EventManager,
    pub events: &'a mut SimEvents,
    pub vibration: &'a mut Vibration,
    pub next: &'a mut NextStage,
    pub allcol: &'a mut AllColRatio,
    pub particles: &'a mut ParticleSets,
    pub player: Option<&'a mut Player>,
    pub text: Text<'a>,
}

impl Ctx<'_> {
    fn demo_accrpt(&self, index: i32) -> bool {
        if !has_event(index) {
            return false;
        }
        matches!(
            self.active.stage_of(event_slot(index)),
            Some(Stage::Play) | Some(Stage::Close)
        )
    }

    fn order_other(&mut self, index: i32) {
        if !has_event(index) {
            return;
        }
        let entity = self.entity;
        let slot = event_slot(index);
        self.active.order(self.list, slot, entity);
    }

    fn end_check(&self, index: i32) -> bool {
        has_event(index) && self.active.end_check(event_slot(index))
    }

    fn event_reset(&mut self) {
        self.manager.exception.init();
    }

    fn player_pos(&self) -> Option<Vec3> {
        self.player.as_ref().map(|player| player.pos)
    }
}

impl ObjDoguu {
    fn private_cut(&mut self, ctx: &mut Ctx<'_>) {
        let staff = ctx.manager.get_my_staff_id(STAFF_DOGUU, None, 0);
        if staff == STAFF_IDX_NONE {
            return;
        }
        let act = ctx.manager.get_my_act_idx(staff, &CUT_NAME_TABLE, true, 0);
        self.act = if act == -1 {
            ACT_IDX_NONE
        } else {
            act as i8
        };
        if self.act == ACT_IDX_NONE {
            ctx.manager.cut_end(staff);
            return;
        }
        let mut do_cut_end = false;
        if ctx.manager.get_is_advance(staff) {
            match self.act {
                CUT_SETGOAL => {
                    let staff_pos = ctx.manager.get_my_vec(staff, STAFF_GOAL_KEY);
                    let angle_y = ctx.angle[1];
                    if let Some(pos) = staff_pos {
                        self.set_goal(pos, angle_y, ctx.manager);
                    }
                }
                CUT_SETBALL => {
                    self.has_crystal = true;
                    ctx.events.list.push(SimEvent::Se(SE_LK_PUT_PEARL));
                }
                CUT_SETANGLE => {
                    let staff_angle = ctx.manager.get_my_int(staff, STAFF_ANGLE_KEY);
                    let angle_y = ctx.angle[1];
                    if let (Some(angle), Some(player)) = (staff_angle, ctx.player.as_deref_mut()) {
                        self.set_player_angle(angle_y, angle, player);
                    }
                }
                CUT_EYEFLASH => {
                    self.crystal_on = true;
                }
                CUT_MESSAGE_KOKOE => {}
                CUT_MESSAGE_OITA => {
                    ctx.events.list.push(SimEvent::Bgm(BGM_GET_PEARL));
                }
                CUT_QUAKE => {
                    ctx.vibration.start_quake_pattern(
                        &QUAKE_PATTERN.to_be_bytes(),
                        0,
                        QUAKE_FLAGS,
                        QUAKE_COORD,
                    );
                    self.timer = ctx
                        .manager
                        .get_my_int(staff, STAFF_TIMER_KEY)
                        .unwrap_or(self.timer);
                }
                CUT_JDEMO => {
                    self.set_jdemo(ctx.next);
                }
                CUT_SETLIGHT => {
                    self.fade = 0;
                }
                _ => {}
            }
        }
        match self.act {
            CUT_SETGOAL => {
                do_cut_end = true;
            }
            CUT_SETBALL => {
                do_cut_end = true;
            }
            CUT_SETANGLE => {
                do_cut_end = true;
            }
            CUT_EYEFLASH => {
                do_cut_end = true;
            }
            CUT_MESSAGE_KOKOE => {
                if self.talk(&mut ctx.text, ctx.flags) == STATUS_BOX_CLOSED {
                    do_cut_end = true;
                }
            }
            CUT_MESSAGE_OITA => {
                if self.talk(&mut ctx.text, ctx.flags) == STATUS_BOX_CLOSED {
                    do_cut_end = true;
                }
            }
            CUT_QUAKE => {
                self.timer -= 1;
                if self.timer == 0 {
                    do_cut_end = true;
                    ctx.vibration.stop_quake(QUAKE_FLAGS);
                }
            }
            CUT_JDEMO => {
                do_cut_end = true;
            }
            CUT_SETLIGHT => {
                self.fade_ratio();
                if self.fade == SETLIGHT_PARTICLE_TICK {
                    let kind = self.kind;
                    let pos = self.glow_pos;
                    let angle = ctx.angle;
                    ctx.particles.live.push((glow_of(kind), pos, angle));
                } else if self.fade == SETLIGHT_GLOW_TICK {
                    self.glow = true;
                }
            }
            _ => {
                do_cut_end = true;
            }
        }
        if do_cut_end {
            ctx.manager.cut_end(staff);
        }
    }

    fn state_move(&mut self, ctx: &mut Ctx<'_>) {
        match self.state {
            STATE_WAIT => {
                if ctx.has_pearl && self.cyl.chk_co_hit() {
                    let hit_obj = self.cyl.get_co_hit_obj();
                    if hit_obj {
                        self.cyl.set_r(CYL_RADIUS_HELD);
                        let index = self.demo1;
                        ctx.order_other(index);
                        self.state = STATE_DEMO1_REQUEST;
                    }
                }
            }
            STATE_DEMO1_REQUEST => {
                let index = self.demo1;
                if ctx.demo_accrpt(index) {
                    self.state = STATE_DEMO1_RUN;
                } else {
                    ctx.order_other(index);
                }
            }
            STATE_DEMO1_RUN => {
                self.private_cut(ctx);
                let index = self.demo1;
                if ctx.end_check(index) {
                    self.state = STATE_DEMO1_END;
                }
            }
            STATE_DEMO1_END => {
                if self.get_finish_event_count(ctx.flags) == 0 {
                    let index = self.demo2;
                    ctx.order_other(index);
                    self.state = STATE_DEMO2_REQUEST;
                } else {
                    let index = self.demo3;
                    ctx.order_other(index);
                    self.state = STATE_DEMO3_REQUEST;
                }
            }
            STATE_DEMO2_REQUEST => {
                let index = self.demo2;
                if ctx.demo_accrpt(index) {
                    self.state = STATE_DEMO2_RUN;
                }
            }
            STATE_DEMO2_RUN => {
                self.private_cut(ctx);
                let index = self.demo2;
                if ctx.end_check(index) {
                    self.state = STATE_DEMO2_END;
                }
            }
            STATE_DEMO2_END => {
                let index = self.demo3;
                ctx.order_other(index);
                self.state = STATE_DEMO3_REQUEST;
            }
            STATE_DEMO3_REQUEST => {
                let index = self.demo3;
                if ctx.demo_accrpt(index) {
                    self.state = STATE_DEMO3_RUN;
                }
            }
            STATE_DEMO3_RUN => {
                self.private_cut(ctx);
                let index = self.demo3;
                if ctx.end_check(index) {
                    self.state = STATE_FINISH;
                }
            }
            STATE_FINISH => {
                self.set_finish_my_event(ctx.flags);
                if self.get_finish_event_count(ctx.flags) >= 3 {
                    ctx.flags.set(EVENT_UNK_1E40);
                    let index = self.megami;
                    ctx.order_other(index);
                    self.state = STATE_MEGAMI_REQUEST;
                } else {
                    self.state = STATE_DONE;
                    ctx.event_reset();
                }
            }
            STATE_DONE => {
                self.cyl.set_r(CYL_RADIUS_DONE);
            }
            STATE_MEGAMI_REQUEST => {
                let index = self.megami;
                if ctx.demo_accrpt(index) {
                    self.state = STATE_MEGAMI_RUN;
                }
            }
            STATE_MEGAMI_RUN => {
                self.private_cut(ctx);
                let index = self.megami;
                if ctx.end_check(index) {
                    self.state = STATE_RESTING;
                }
            }
            STATE_RESTING => {
                self.cyl.set_r(CYL_RADIUS_DONE);
            }
            _ => {}
        }
    }

    fn execute(&mut self, ctx: &mut Ctx<'_>) {
        if self.glow {
            self.set_point_light();
        }
        self.cyl.set_c(ctx.pos);
        self.cyl.register();
        let player_pos = ctx.player_pos();
        self.cyl.probe_co(player_pos);
        if self.all_col_ratio != 1.0 {
            ctx.allcol.ratio = self.all_col_ratio;
        }
        self.state_move(ctx);
        if !self.body && self.crystal_on {
            if self.brk_frame < BRK_PULSE_FRAME {
                ctx.events.list.push(SimEvent::Se(SE_OBJ_SDOLL_EYE_PULSE));
            }
            self.brk_frame += BRK_STEP;
        }
        if self.demo_actor_id != 0 {
            if self.shape == 0 {
                self.body = false;
            } else {
                self.body = true;
            }
        }
    }
}

#[derive(SystemParam)]
pub struct World<'w> {
    pub list: Res<'w, EventList>,
    pub active: ResMut<'w, ActiveEvent>,
    pub manager: ResMut<'w, EventManager>,
    pub flags: ResMut<'w, EventFlags>,
    pub collect: Res<'w, Collect>,
    pub equipped: Res<'w, Equipped>,
    pub events: ResMut<'w, SimEvents>,
    pub vibration: ResMut<'w, Vibration>,
    pub next: ResMut<'w, NextStage>,
    pub lights: ResMut<'w, PointLights>,
    pub allcol: ResMut<'w, AllColRatio>,
    pub particles: ResMut<'w, ParticleSets>,
    pub procs: ResMut<'w, MsgProcs>,
    pub info: ResMut<'w, MsgGameInfo>,
    pub one: ResMut<'w, Msg1Box>,
    pub two: ResMut<'w, Msg2Box>,
    pub archive: Option<Res<'w, MsgArchives>>,
}

pub fn control(
    mut statues: Query<(Entity, &mut Actor, &mut ObjDoguu), Without<Player>>,
    mut players: Query<&mut Player>,
    world: World,
) {
    let World {
        list,
        mut active,
        mut manager,
        mut flags,
        collect,
        equipped,
        mut events,
        mut vibration,
        mut next,
        mut lights,
        mut allcol,
        mut particles,
        mut procs,
        mut info,
        mut one,
        mut two,
        archive,
    } = world;
    let archive = archive.as_deref().map(|held| held as &dyn MsgArchive);
    allcol.ratio = 1.0;
    let mut live: Vec<LightInfluence> = Vec::new();
    for (entity, actor, mut statue) in &mut statues {
        if !statue.entered {
            let argument = ObjDoguu::argument_of(&actor.name);
            statue.create(argument, actor.parameters, actor.pos, actor.scale.x);
            let kind = statue.kind;
            let held = inventory::check_get_item(&equipped, &collect, pearl_of(kind));
            statue.create_init(held, &flags, &mut manager);
        }
        let pos = actor.pos;
        let angle = actor.angle;
        let has_pearl = inventory::check_get_item(&equipped, &collect, statue.item_no);
        let mut player = players.iter_mut().next();
        {
            let text = Text {
                procs: &mut procs,
                info: &mut info,
                one: &mut one,
                two: &mut two,
                archive,
            };
            statue.execute(&mut Ctx {
                entity,
                pos,
                angle,
                has_pearl,
                flags: &mut flags,
                list: &list,
                active: &mut active,
                manager: &mut manager,
                events: &mut events,
                vibration: &mut vibration,
                next: &mut next,
                allcol: &mut allcol,
                particles: &mut particles,
                player: player.as_deref_mut(),
                text,
            });
        }
        if statue.glow {
            live.push(statue.influence);
        }
        let _ = actor.pos;
    }
    lights.live = live;
}
