use bevy_ecs::prelude::*;
use glam::{Quat, Vec3};

use super::actor::Actor;
use super::collision::Collision;
use super::enemy::{self, Enemy, Hurt};
use super::event::{ActiveEvent, EventList};
use super::events::{SimEvent, SimEvents};
use super::flags::{StoryBits, Switches};
use super::ground;
use super::hookshot::{CAPSULE_RADIUS, Hookshot, Phase, SHOT_STEP};
use super::inventory::Equipped;
use super::pickup::{ITEM_ACTORS, Pickup};
use super::player::Player;
use super::stream;
use super::walk;

pub const HOMEN_ACTORS: [&str; 2] = ["Homen1", "Homen2"];

pub const HOMEN_EVENT: &str = "homen_down";

pub const TYPE_L: u8 = 0;

pub const NO_SWITCH: u8 = 0xFF;
pub const NO_ITEM: u8 = 0x3F;
pub const NO_ENEMY: u8 = 0xF;

pub const BOOTS_SLOT: usize = 9;
pub const ITEM_IRON_BOOTS: u8 = 0x29;

pub const NORSE: [[Vec3; 2]; 2] = [
    [Vec3::new(0.0, 453.0, 140.0), Vec3::new(0.0, 181.5, 48.0)],
    [Vec3::new(0.0, 453.0, 190.0), Vec3::new(0.0, 181.5, 68.0)],
];

pub const HOOK_RADIUS: [f32; 2] = [105.0, 57.0];

pub const CULL_MIN: [[f32; 3]; 2] = [[-300.0, -30.0, -100.0], [-100.0, -30.0, -100.0]];
pub const CULL_MAX: [[f32; 3]; 2] = [[300.0, 600.0, 300.0], [100.0, 250.0, 120.0]];

pub const HINGE: f32 = 200.0;
pub const LAID: i16 = 1;

pub const GRAVITY: f32 = -2.0;
pub const MAX_FALL: f32 = -36.0;
pub const GROUND_PROBE: f32 = 100.0;
pub const GROUND_MARGIN: f32 = 1.0;
pub const NO_GROUND: f32 = f32::NEG_INFINITY;
pub const FALL_SPEED: [f32; 2] = [20.0, 12.0];

pub const SWING_FRAMES: i16 = 3;
pub const SWING_SCALE: f32 = 0.5;
pub const SWING_STEP: f32 = 20.0;

pub const ITEM_FRAMES: i16 = 20;
pub const ENEMY_FRAMES: i16 = 6;

pub const TILT_RATE: [i16; 2] = [90, 120];
pub const TILT_MAX: i16 = 2000;
pub const TILT_LIMIT: i16 = 0x5000;
pub const SPIN_RANGE: f32 = 3072.0;
pub const SPIN_SCALE: i16 = 20;
pub const SPIN_FACE_SCALE: i16 = 10;
pub const SPIN_MAX_STEP: i16 = 0x1000;
pub const SPIN_FACE: i16 = -0x8000;

pub const ENEMY_ROWS: usize = 11;
pub const ENEMY_CHUCHU: u8 = 6;
pub const ENEMY_PARAM: [u32; ENEMY_ROWS] = [
    0xFFFF_0000,
    0xFFFF_0020,
    0xFFFF_0040,
    0xFFFF_0060,
    0xFFFF_0080,
    0xFFFF_00A0,
    0xFF00_0000,
    0xFF00_0100,
    0xFF00_0200,
    0xFF00_0300,
    0xFF00_0400,
];
pub const ENEMY_LIFT: [Vec3; 2] = [
    Vec3::new(0.0, 300.0, 350.0),
    Vec3::new(0.0, 0.0, 25.0),
];

pub const ITEM_SWITCH: u32 = 0xFF;
pub const ITEM_ACTION_SHIFT: u32 = 26;
pub const ITEM_ACTION: u32 = 1;

pub const SE_MASK_S_BREAK: u32 = 0x69FF;
pub const SE_MASK_L_BREAK: u32 = 0x6A00;
pub const SE_RIDDLE: u32 = 0x0806;

pub const EVENT_BOOKED: u16 = 0x3880;
pub const EVENT_DROPPED: u16 = 0x3410;

const ARG0_SHIFT: u32 = 0x08;
const ARG0_MASK: u32 = 0x0F;
const ITEM_SHIFT: u32 = 0x0C;
const ITEM_MASK: u32 = 0x3F;
const BIT_SHIFT: u32 = 0x00;
const BIT_MASK: u32 = 0x7F;
const ENEMY_SHIFT: u32 = 0x12;
const ENEMY_MASK: u32 = 0x0F;
const SWITCH_SHIFT: u32 = 0x18;
const SWITCH_MASK: u32 = 0xFF;
const TYPE_BIT: u32 = 1;
const KIND_MASK: usize = 1;

const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
const RND_SEED: i32 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Free,
    WaitFalldown,
    Falldown,
    Wait,
    Done,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Homen {
    pub kind: u8,
    pub switch_no: u8,
    pub item: u8,
    pub item_bit: u8,
    pub enemy: u8,
    pub state: Step,
    pub land: Vec3,
    pub ground_y: f32,
    pub tilt: i16,
    pub tilt_rate: i16,
    pub facing: i16,
    pub swing: i16,
    pub wobble: i16,
    pub settle: i16,
    pub laid: i16,
    pub hookshot: Option<Entity>,
    pub foe: Option<Entity>,
    pub carried: bool,
    pub entered: bool,
    pub item_timer: i16,
    pub foe_timer: i16,
    pub event_id: i16,
    pub event_state: i16,
    pub speed: Vec3,
    pub speed_f: f32,
    pub gravity: f32,
    pub max_fall: f32,
    pub pitch: i16,
    pub yaw: i16,
    pub roll: i16,
    pub seed: [i32; 3],
}

impl Homen {
    pub fn spawn(parameters: u32) -> Self {
        Self {
            kind: (((parameters >> ARG0_SHIFT) & ARG0_MASK) & TYPE_BIT) as u8,
            switch_no: ((parameters >> SWITCH_SHIFT) & SWITCH_MASK) as u8,
            item: ((parameters >> ITEM_SHIFT) & ITEM_MASK) as u8,
            item_bit: ((parameters >> BIT_SHIFT) & BIT_MASK) as u8,
            enemy: ((parameters >> ENEMY_SHIFT) & ENEMY_MASK) as u8,
            state: Step::Free,
            land: Vec3::ZERO,
            ground_y: 0.0,
            tilt: 0,
            tilt_rate: 0,
            facing: 0,
            swing: 0,
            wobble: 0,
            settle: 0,
            laid: 0,
            hookshot: None,
            foe: None,
            carried: false,
            entered: false,
            item_timer: 0,
            foe_timer: 0,
            event_id: 0,
            event_state: 0,
            speed: Vec3::ZERO,
            speed_f: 0.0,
            gravity: 0.0,
            max_fall: 0.0,
            pitch: 0,
            yaw: 0,
            roll: 0,
            seed: [RND_SEED; 3],
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        let index = self.slot();
        (
            CULL_MIN.get(index).copied().unwrap_or(CULL_MIN[0]),
            CULL_MAX.get(index).copied().unwrap_or(CULL_MAX[0]),
        )
    }

    pub fn drawn(&self) -> bool {
        self.state != Step::Wait && self.state != Step::Done
    }

    pub fn latched(&self, switches: &Switches) -> bool {
        self.switch_no != NO_SWITCH && switches.test(self.switch_no)
    }

    pub fn break_sound(&self) -> u32 {
        if self.kind == TYPE_L {
            SE_MASK_S_BREAK
        } else {
            SE_MASK_L_BREAK
        }
    }

    fn slot(&self) -> usize {
        usize::from(self.kind) & KIND_MASK
    }

    fn hook_reach(&self) -> f32 {
        HOOK_RADIUS
            .get(self.slot())
            .copied()
            .unwrap_or(HOOK_RADIUS[0])
            + CAPSULE_RADIUS
    }

    pub fn norse_offset(&self, set: usize) -> Vec3 {
        NORSE
            .get(set & KIND_MASK)
            .and_then(|row| row.get(self.slot()))
            .copied()
            .unwrap_or(Vec3::ZERO)
    }

    pub fn norse_point(&self, set: usize, pos: Vec3) -> Vec3 {
        self.placed(self.norse_offset(set), pos)
    }

    pub fn placed(&self, offset: Vec3, pos: Vec3) -> Vec3 {
        let turn = Quat::from_rotation_z(walk::from_s16(self.roll))
            * Quat::from_rotation_y(walk::from_s16(self.yaw))
            * Quat::from_rotation_x(walk::from_s16(self.pitch));
        let hinge = Vec3::new(0.0, 0.0, HINGE);
        if self.kind == TYPE_L && self.laid == LAID {
            pos + turn * (offset - hinge) + hinge
        } else {
            pos + turn * offset
        }
    }

    pub fn exe_event(&mut self, entity: Entity, list: &EventList, active: &mut ActiveEvent) {
        match self.event_state {
            1 => {
                if self.event_id >= 0 {
                    active.order(list, usize::from(self.event_id as u16), entity);
                }
            }
            2 => {
                if self.event_id >= 0 && active.end_check(usize::from(self.event_id as u16)) {
                    self.event_id = -1;
                    self.event_state = 0;
                }
            }
            _ => {}
        }
    }

    pub fn item_parameter(&self) -> u32 {
        u32::from(self.item)
            | (u32::from(self.item_bit) << 8)
            | (ITEM_SWITCH << 16)
            | (ITEM_ACTION << ITEM_ACTION_SHIFT)
    }

    pub fn make_item(&self, actor: &Actor, commands: &mut Commands) {
        if self.item == NO_ITEM {
            return;
        }
        let parameters = self.item_parameter();
        commands.spawn((
            Actor {
                name: ITEM_ACTORS[0].to_owned(),
                parameters,
                pos: actor.pos,
                angle: [0; 3],
                set_id: 0,
                room: actor.room,
            },
            Pickup::spawn(parameters),
        ));
    }

    pub fn make_enemy(&mut self, actor: &Actor, commands: &mut Commands) {
        if self.enemy == NO_ENEMY || self.enemy >= ENEMY_CHUCHU {
            return;
        }
        let Some(parameter) = ENEMY_PARAM.get(usize::from(self.enemy)).copied() else {
            return;
        };
        let lift = ENEMY_LIFT.get(self.slot()).copied().unwrap_or(ENEMY_LIFT[0]);
        let (state, hit_points) = enemy::spawn(parameter, 0);
        self.foe = Some(
            commands
                .spawn((
                    Actor {
                        name: enemy::ENEMY_ACTOR.to_owned(),
                        parameters: parameter,
                        pos: self.placed(lift, actor.pos),
                        angle: [0; 3],
                        set_id: 0,
                        room: actor.room,
                    },
                    state,
                    hit_points,
                    Hurt::default(),
                ))
                .id(),
        );
    }

    pub fn manage_item_timer(&mut self, actor: &Actor, commands: &mut Commands) {
        if self.item_timer <= 0 {
            return;
        }
        self.item_timer -= 1;
        if self.item_timer == 0 {
            self.make_item(actor, commands);
        }
    }

    pub fn manage_enemy_timer(&mut self, actor: &Actor, commands: &mut Commands) {
        if self.foe_timer <= 0 {
            return;
        }
        self.foe_timer -= 1;
        if self.foe_timer == 0 {
            self.make_enemy(actor, commands);
        }
    }

    pub fn process_init(&mut self, state: Step) -> bool {
        self.state = state;
        true
    }

    pub fn process_free_init(&mut self) -> bool {
        self.foe = None;
        true
    }

    pub fn process_free_main(
        &mut self,
        entity: Entity,
        pos: Vec3,
        player: Option<&Player>,
        booted: bool,
        hooks: &mut Query<(Entity, &mut Hookshot)>,
        stories: &mut StoryBits,
    ) {
        if self.carried {
            self.process_wait_falldown_init(pos, hooks, stories);
            self.process_init(Step::WaitFalldown);
        }
        self.set_co_collision(entity, pos, player, booted, hooks, stories);
    }

    pub fn process_wait_falldown_init(
        &mut self,
        pos: Vec3,
        hooks: &mut Query<(Entity, &mut Hookshot)>,
        stories: &mut StoryBits,
    ) -> bool {
        self.carried = false;
        self.foe_timer = ENEMY_FRAMES;
        self.item_timer = ITEM_FRAMES;
        self.swing = 0;
        if self.kind != TYPE_L {
            self.land = pos;
            if let Some(shot) = self.hookshot
                && let Ok((_, hook)) = hooks.get(shot)
            {
                self.land = hook.pos - turned_y(self.norse_offset(1), self.yaw);
            }
        }
        if self.kind == TYPE_L {
            stories.set(EVENT_DROPPED);
        }
        true
    }

    pub fn process_wait_falldown_main(
        &mut self,
        entity: Entity,
        actor: &mut Actor,
        solid: Option<&Collision>,
        player: Option<&Player>,
        booted: bool,
        hooks: &mut Query<(Entity, &mut Hookshot)>,
        stories: &mut StoryBits,
    ) {
        if self.swing < SWING_FRAMES {
            let land = self.land;
            add_calc2(&mut actor.pos.x, land.x, SWING_SCALE, SWING_STEP);
            add_calc2(&mut actor.pos.y, land.y, SWING_SCALE, SWING_STEP);
            add_calc2(&mut actor.pos.z, land.z, SWING_SCALE, SWING_STEP);
            self.swing += 1;
        } else {
            self.process_falldown_init(actor.pos, solid);
            self.process_init(Step::Falldown);
        }
        self.set_co_collision(entity, actor.pos, player, booted, hooks, stories);
    }

    pub fn process_falldown_init(&mut self, pos: Vec3, solid: Option<&Collision>) -> bool {
        self.gravity = GRAVITY;
        self.max_fall = MAX_FALL;
        let probe = Vec3::new(pos.x, pos.y + GROUND_PROBE, pos.z);
        self.ground_y = ground::surface(solid, probe).unwrap_or(NO_GROUND);
        self.tilt_rate = TILT_RATE.get(self.slot()).copied().unwrap_or(TILT_RATE[0]);
        self.wobble = rnd_fx(&mut self.seed, SPIN_RANGE) as i16;
        true
    }

    pub fn process_falldown_main(
        &mut self,
        pos: Vec3,
        switches: &mut Switches,
        events: &mut SimEvents,
    ) {
        let sp34 = self.norse_point(0, pos);
        if self.kind == TYPE_L && self.laid != LAID {
            let sp1c = self.placed(Vec3::ZERO, pos);
            if sp1c.y - GROUND_MARGIN <= self.ground_y {
                self.laid = LAID;
                self.stall();
            }
        } else if pos.y < self.ground_y {
            self.stall();
        }
        if sp34.y < self.ground_y {
            self.stall();
            self.tilt_rate = 0;
            self.tilt = 0;
            self.process_wait_init(switches, events);
            self.process_init(Step::Wait);
        } else if self.laid != LAID {
            stream::add_calc_angle_s2(&mut self.roll, self.wobble, SPIN_SCALE, SPIN_MAX_STEP);
            stream::add_calc_angle_s2(
                &mut self.yaw,
                self.facing.wrapping_add(SPIN_FACE),
                SPIN_FACE_SCALE,
                SPIN_MAX_STEP,
            );
        }
        if self.gravity != 0.0 {
            self.speed_f = FALL_SPEED.get(self.slot()).copied().unwrap_or(FALL_SPEED[0]);
        }
    }

    pub fn process_wait_init(&mut self, switches: &mut Switches, events: &mut SimEvents) -> bool {
        switches.set(self.switch_no);
        self.settle = 1;
        events.list.push(SimEvent::Se(self.break_sound()));
        true
    }

    pub fn process_wait_main(&mut self, events: &mut SimEvents) {
        if self.settle > 0 {
            self.settle -= 1;
            if self.settle == 0 {
                self.process_none_init(events);
                self.process_init(Step::Done);
            }
        }
    }

    pub fn process_none_init(&mut self, events: &mut SimEvents) -> bool {
        if self.kind == TYPE_L {
            events.list.push(SimEvent::Se(SE_RIDDLE));
        }
        true
    }

    pub fn process_none_main(&self, foes: &Query<(), With<Enemy>>) -> bool {
        match self.foe {
            Some(foe) => !foes.contains(foe),
            None => false,
        }
    }

    pub fn adjust_hookshot(
        &mut self,
        entity: Entity,
        pos: Vec3,
        booted: bool,
        hooks: &mut Query<(Entity, &mut Hookshot)>,
    ) {
        if booted {
            return;
        }
        let Some(shot) = self.hookshot else {
            return;
        };
        let Ok((_, mut hook)) = hooks.get_mut(shot) else {
            return;
        };
        let offset = turned_y(self.norse_offset(1), self.yaw);
        hook.anchored(entity, pos + offset);
        hook.carry_offset = offset;
        self.hookshot = None;
    }

    pub fn set_co_collision(
        &mut self,
        entity: Entity,
        pos: Vec3,
        player: Option<&Player>,
        booted: bool,
        hooks: &mut Query<(Entity, &mut Hookshot)>,
        stories: &mut StoryBits,
    ) {
        let centre = self.norse_point(0, pos);
        let reach = self.hook_reach();
        let mut struck = None;
        for (shot, hook) in hooks.iter() {
            if hook.phase == Phase::Shot
                && walk::swept(
                    hook.pos,
                    hook.pos + hook.heading * SHOT_STEP,
                    centre,
                    reach,
                )
            {
                struck = Some((shot, hook.pos));
                break;
            }
        }
        let Some((shot, tip)) = struck else {
            return;
        };
        self.facing = player.map_or(self.facing, |player| walk::to_s16(player.face));
        self.hookshot = Some(shot);
        if self.kind == TYPE_L {
            stories.set(EVENT_BOOKED);
        }
        if booted {
            if let Ok((_, mut hook)) = hooks.get_mut(shot) {
                hook.hooked(entity, pos - tip);
            }
            self.carried = true;
        }
    }

    fn stall(&mut self) {
        self.gravity = 0.0;
        self.speed = Vec3::ZERO;
        self.speed_f = 0.0;
    }

    fn lean(&mut self) {
        self.tilt = self.tilt.wrapping_add(self.tilt_rate).min(TILT_MAX);
        self.pitch = self.pitch.wrapping_add(self.tilt);
        if self.pitch >= TILT_LIMIT {
            self.pitch = TILT_LIMIT;
            self.tilt = 0;
            self.tilt_rate = 0;
        }
    }

    fn fall(&mut self) -> Vec3 {
        let (speed, _) = stream::calc_speed(
            self.speed_f,
            self.yaw,
            self.speed.y,
            self.gravity,
            self.max_fall,
        );
        self.speed = speed;
        speed
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

fn rnd_fx(seed: &mut [i32; 3], range: f32) -> f32 {
    range * (rnd(seed) - 0.5) * 2.0
}

fn add_calc2(value: &mut f32, target: f32, scale: f32, max_step: f32) {
    if *value != target {
        let step = (scale * (target - *value)).clamp(-max_step, max_step);
        *value += step;
    }
}

fn turned_y(offset: Vec3, yaw: i16) -> Vec3 {
    Quat::from_rotation_y(walk::from_s16(yaw)) * offset
}

pub fn is_homen(name: &str) -> bool {
    HOMEN_ACTORS.contains(&name)
}

pub fn control(
    collision: Option<Res<Collision>>,
    equipped: Res<Equipped>,
    lists: Res<EventList>,
    mut active: ResMut<ActiveEvent>,
    mut stories: ResMut<StoryBits>,
    mut switches: ResMut<Switches>,
    mut events: ResMut<SimEvents>,
    players: Query<&Player>,
    mut hooks: Query<(Entity, &mut Hookshot)>,
    foes: Query<(), With<Enemy>>,
    mut masks: Query<(Entity, &mut Actor, &mut Homen), Without<Player>>,
    mut commands: Commands,
) {
    let solid = collision.as_deref();
    let player = players.iter().next().copied();
    let booted = equipped.item(BOOTS_SLOT) == ITEM_IRON_BOOTS;
    for (entity, mut actor, mut mask) in &mut masks {
        let actor = &mut *actor;
        let mask = &mut *mask;
        if !mask.entered {
            mask.entered = true;
            if mask.latched(&switches) {
                let _ = commands.entity(entity).despawn();
                continue;
            }
            mask.pitch = actor.angle[0];
            mask.yaw = actor.angle[1];
            mask.roll = actor.angle[2];
            mask.process_free_init();
            mask.process_init(Step::Free);
        }
        mask.adjust_hookshot(entity, actor.pos, booted, &mut hooks);
        mask.exe_event(entity, &lists, &mut active);
        mask.manage_item_timer(actor, &mut commands);
        mask.manage_enemy_timer(actor, &mut commands);
        let mut doomed = false;
        match mask.state {
            Step::Free => {
                mask.process_free_main(
                    entity,
                    actor.pos,
                    player.as_ref(),
                    booted,
                    &mut hooks,
                    &mut stories,
                );
            }
            Step::WaitFalldown => {
                mask.process_wait_falldown_main(
                    entity,
                    actor,
                    solid,
                    player.as_ref(),
                    booted,
                    &mut hooks,
                    &mut stories,
                );
            }
            Step::Falldown => {
                mask.process_falldown_main(actor.pos, &mut switches, &mut events);
            }
            Step::Wait => {
                mask.process_wait_main(&mut events);
            }
            Step::Done => {
                doomed = mask.process_none_main(&foes);
            }
        }
        mask.lean();
        actor.pos += mask.fall();
        actor.angle = [mask.pitch, mask.yaw, mask.roll];
        if doomed {
            let _ = commands.entity(entity).despawn();
        }
    }
}