use bevy_ecs::prelude::*;
use glam::{Quat, Vec3};

use super::actor::Actor;
use super::collision::Collision;
use super::enemy::{self, Enemy, Hurt};
use super::events::{SimEvent, SimEvents};
use super::flags::{StoryBits, Switches};
use super::ground;
use super::hookshot::{CAPSULE_RADIUS, Hookshot, Phase, SHOT_STEP};
use super::inventory::Equipped;
use super::pickup::{self, Pickup};
use super::player::Player;
use super::stream;
use super::walk;

pub const HOMEN_ACTORS: [&str; 2] = ["Homen1", "Homen2"];

pub const TYPE_SLAB: u8 = 0;

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
pub const FALL_SPEED: [f32; 2] = [20.0, 12.0];

pub const SWING_FRAMES: i16 = 3;
pub const SWING_SCALE: f32 = 0.5;
pub const SWING_STEP: f32 = 20.0;

pub const ITEM_FRAMES: i16 = 20;
pub const ENEMY_FRAMES: i16 = 6;
pub const SETTLE_FRAMES: i16 = 1;

pub const TILT_RATE: [i16; 2] = [90, 120];
pub const TILT_MAX: i16 = 2000;
pub const TILT_LIMIT: i16 = 0x5000;
pub const SPIN_RANGE: f32 = 0xC00 as f32;
pub const SPIN_SCALE: i16 = 20;
pub const SPIN_FACE_SCALE: i16 = 10;
pub const SPIN_MAX_STEP: i16 = 0x1000;
pub const SPIN_FACE: i16 = 0x8000;

pub const ENEMY_ROWS: usize = 6;
pub const ENEMY_PARAM: [u32; ENEMY_ROWS] = [
    0xFFFF_0000,
    0xFFFF_0020,
    0xFFFF_0040,
    0xFFFF_0060,
    0xFFFF_0080,
    0xFFFF_00A0,
];
pub const ENEMY_LIFT: [Vec3; 2] = [
    Vec3::new(0.0, 300.0, 350.0),
    Vec3::new(0.0, 0.0, 25.0),
];

pub const SE_MASK_S_BREAK: u32 = 0x69FF;
pub const SE_MASK_L_BREAK: u32 = 0x6A00;
pub const SE_RIDDLE: u32 = 0x0806;

pub const EVENT_BOOKED: u16 = 0x3880;
pub const EVENT_DROPPED: u16 = 0x3410;

const ARG0_SHIFT: u32 = 0x08;
const ARG0_MASK: u32 = 0x0F;
const ITEM_SHIFT: u32 = 0x0C;
const ITEM_MASK: u32 = 0x3F;
const ENEMY_SHIFT: u32 = 0x12;
const ENEMY_MASK: u32 = 0x0F;
const SWITCH_SHIFT: u32 = 0x18;
const SWITCH_MASK: u32 = 0xFF;
const TYPE_BIT: u32 = 1;
const KIND_MASK: usize = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Free,
    WaitFalldown,
    Falldown,
    Wait,
    Done,
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct Domen {
    pub kind: u8,
    pub switch_no: u8,
    pub item: u8,
    pub enemy: u8,
    pub step: Step,
    pub land: Vec3,
    pub ground_y: f32,
    pub pitch: i16,
    pub yaw: i16,
    pub roll: i16,
    pub facing: i16,
    pub tilt: i16,
    pub tilt_rate: i16,
    pub swing: i16,
    pub wobble: i16,
    pub settle: i16,
    pub laid: i16,
    pub item_timer: i16,
    pub foe_timer: i16,
    pub speed: Vec3,
    pub speed_f: f32,
    pub gravity: f32,
    pub max_fall: f32,
    pub foe: Option<Entity>,
    pub seed: u32,
}

impl Domen {
    pub fn spawn(parameters: u32, switches: &Switches, angle: [i16; 3]) -> Option<Self> {
        let switch_no = ((parameters >> SWITCH_SHIFT) & SWITCH_MASK) as u8;
        if switch_no != NO_SWITCH && switches.test(switch_no) {
            return None;
        }
        Some(Self {
            kind: (((parameters >> ARG0_SHIFT) & ARG0_MASK) & TYPE_BIT) as u8,
            switch_no,
            item: ((parameters >> ITEM_SHIFT) & ITEM_MASK) as u8,
            enemy: ((parameters >> ENEMY_SHIFT) & ENEMY_MASK) as u8,
            step: Step::Free,
            land: Vec3::ZERO,
            ground_y: 0.0,
            pitch: angle[0],
            yaw: angle[1],
            roll: angle[2],
            facing: 0,
            tilt: 0,
            tilt_rate: 0,
            swing: 0,
            wobble: 0,
            settle: 0,
            laid: 0,
            item_timer: 0,
            foe_timer: 0,
            speed: Vec3::ZERO,
            speed_f: 0.0,
            gravity: 0.0,
            max_fall: 0.0,
            foe: None,
            seed: 0x2540_F491,
        })
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN[self.slot()], CULL_MAX[self.slot()])
    }

    pub fn drawn(&self) -> bool {
        !matches!(self.step, Step::Wait | Step::Done)
    }

    pub fn break_sound(&self) -> u32 {
        if self.kind == TYPE_SLAB {
            SE_MASK_S_BREAK
        } else {
            SE_MASK_L_BREAK
        }
    }

    fn slot(&self) -> usize {
        usize::from(self.kind) & KIND_MASK
    }

    fn norse(&self, set: usize) -> Vec3 {
        NORSE[set & KIND_MASK][self.slot()]
    }

    fn reach(&self) -> f32 {
        HOOK_RADIUS[self.slot()] + CAPSULE_RADIUS
    }

    fn placed(&self, offset: Vec3, pos: Vec3) -> Vec3 {
        let turn = Quat::from_rotation_z(walk::from_s16(self.roll))
            * Quat::from_rotation_y(walk::from_s16(self.yaw))
            * Quat::from_rotation_x(walk::from_s16(self.pitch));
        let hinge = Vec3::new(0.0, 0.0, HINGE);
        if self.kind == TYPE_SLAB && self.laid == LAID {
            pos + turn * (offset - hinge) + hinge
        } else {
            pos + turn * offset
        }
    }

    fn strike(
        &mut self,
        pos: Vec3,
        hooks: &Query<&Hookshot>,
        facing: i16,
        stories: &mut StoryBits,
    ) -> Option<Vec3> {
        let centre = self.placed(self.norse(0), pos);
        let head = hooks.iter().find(|hook| {
            hook.phase == Phase::Shot
                && walk::swept(
                    hook.pos,
                    hook.pos + hook.heading * SHOT_STEP,
                    centre,
                    self.reach(),
                )
        })?;
        self.facing = facing;
        if self.kind == TYPE_SLAB {
            stories.set(EVENT_BOOKED);
        }
        Some(head.pos)
    }

    fn summon(&mut self, actor: &Actor, commands: &mut Commands) {
        if self.item_timer > 0 {
            self.item_timer -= 1;
            if self.item_timer == 0 && self.item != NO_ITEM {
                let parameter = u32::from(self.item);
                commands.spawn((
                    fresh(actor, pickup::ITEM_ACTORS[0], actor.pos, parameter),
                    Pickup::spawn(parameter),
                ));
            }
        }
        if self.foe_timer > 0 {
            self.foe_timer -= 1;
            if self.foe_timer == 0 {
                self.hire(actor, commands);
            }
        }
    }

    fn hire(&mut self, actor: &Actor, commands: &mut Commands) {
        if self.enemy == NO_ENEMY {
            return;
        }
        let Some(parameter) = ENEMY_PARAM.get(usize::from(self.enemy)).copied() else {
            return;
        };
        let lift = self.placed(ENEMY_LIFT[self.slot()], actor.pos);
        let (state, hit_points) = enemy::spawn(parameter, 0);
        self.foe = Some(
            commands
                .spawn((
                    fresh(actor, enemy::ENEMY_ACTOR, lift, parameter),
                    state,
                    hit_points,
                    Hurt::default(),
                ))
                .id(),
        );
    }

    fn drop_in(&mut self, pos: Vec3, head: Option<Vec3>, stories: &mut StoryBits) {
        self.foe_timer = ENEMY_FRAMES;
        self.item_timer = ITEM_FRAMES;
        self.swing = 0;
        if self.kind != TYPE_SLAB {
            self.land = head.map_or(pos, |head| head - turned_y(self.norse(1), self.yaw));
        }
        if self.kind == TYPE_SLAB {
            stories.set(EVENT_DROPPED);
        }
        self.step = Step::WaitFalldown;
    }

    fn begin_fall(&mut self, pos: Vec3, solid: Option<&Collision>) {
        self.gravity = GRAVITY;
        self.max_fall = MAX_FALL;
        let probe = Vec3::new(pos.x, pos.y + GROUND_PROBE, pos.z);
        self.ground_y = ground::surface(solid, probe).unwrap_or(0.0);
        self.tilt_rate = TILT_RATE[self.slot()];
        self.wobble = self.spin_target();
    }

    fn spin_target(&mut self) -> i16 {
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 17;
        self.seed ^= self.seed << 5;
        let draw = (self.seed >> 8) as f32 / 16777216.0;
        (draw * 2.0 * SPIN_RANGE - SPIN_RANGE) as i16
    }

    fn tumbled(&mut self, centre: Vec3, base: Vec3) -> bool {
        if self.kind == TYPE_SLAB && self.laid != LAID {
            if base.y - GROUND_MARGIN <= self.ground_y {
                self.laid = LAID;
                self.stall();
            }
        } else if self.kind != TYPE_SLAB && base.y < self.ground_y {
            self.stall();
        }
        if centre.y < self.ground_y {
            self.tilt_rate = 0;
            self.tilt = 0;
            return true;
        }
        if self.laid != LAID {
            stream::add_calc_angle_s2(&mut self.roll, self.wobble, SPIN_SCALE, SPIN_MAX_STEP);
            stream::add_calc_angle_s2(
                &mut self.yaw,
                self.facing.wrapping_add(SPIN_FACE),
                SPIN_FACE_SCALE,
                SPIN_MAX_STEP,
            );
        }
        if self.gravity != 0.0 {
            self.speed_f = FALL_SPEED[self.slot()];
        }
        false
    }

    fn stall(&mut self) {
        self.gravity = 0.0;
        self.speed = Vec3::ZERO;
        self.speed_f = 0.0;
    }

    fn lean(&mut self) {
        self.tilt = (self.tilt + self.tilt_rate).min(TILT_MAX);
        self.pitch = self.pitch.wrapping_add(self.tilt);
        if self.pitch >= TILT_LIMIT {
            self.pitch = TILT_LIMIT;
            self.tilt = 0;
            self.tilt_rate = 0;
        }
    }

    fn fall(&mut self) -> Vec3 {
        let (speed, speed_f) = stream::calc_speed(
            self.speed_f,
            self.yaw,
            self.speed.y,
            self.gravity,
            self.max_fall,
        );
        self.speed = speed;
        self.speed_f = speed_f;
        speed
    }

    fn angle(&self) -> [i16; 3] {
        [self.pitch, self.yaw, self.roll]
    }
}

fn fresh(actor: &Actor, name: &str, pos: Vec3, parameters: u32) -> Actor {
    Actor {
        name: name.to_owned(),
        parameters,
        pos,
        angle: [0; 3],
        set_id: 0,
        room: actor.room,
    }
}

fn turned_y(offset: Vec3, yaw: i16) -> Vec3 {
    Quat::from_rotation_y(walk::from_s16(yaw)) * offset
}

pub fn is_domen(name: &str) -> bool {
    HOMEN_ACTORS.contains(&name)
}

pub fn control(
    collision: Option<Res<Collision>>,
    equipped: Res<Equipped>,
    players: Query<&Player>,
    hooks: Query<&Hookshot>,
    foes: Query<(), With<Enemy>>,
    mut stories: ResMut<StoryBits>,
    mut switches: ResMut<Switches>,
    mut events: ResMut<SimEvents>,
    mut masks: Query<(Entity, &mut Actor, &mut Domen)>,
    mut commands: Commands,
) {
    let solid = collision.as_deref();
    let player = players.iter().next().copied();
    let booted = equipped.item(BOOTS_SLOT) == ITEM_IRON_BOOTS;
    for (entity, mut actor, mut mask) in &mut masks {
        let struck = match mask.step {
            Step::Free | Step::WaitFalldown => mask.strike(
                actor.pos,
                &hooks,
                player.map_or(0, |player| walk::to_s16(player.face)),
                &mut stories,
            ),
            _ => None,
        };
        mask.summon(&actor, &mut commands);
        match mask.step {
            Step::Free => {
                if struck.is_some() && booted {
                    mask.drop_in(actor.pos, struck, &mut stories);
                }
            }
            Step::WaitFalldown => {
                if mask.swing < SWING_FRAMES {
                    let land = mask.land;
                    walk::add_calc(&mut actor.pos.x, land.x, SWING_SCALE, SWING_STEP, 0.0);
                    walk::add_calc(&mut actor.pos.y, land.y, SWING_SCALE, SWING_STEP, 0.0);
                    walk::add_calc(&mut actor.pos.z, land.z, SWING_SCALE, SWING_STEP, 0.0);
                    mask.swing += 1;
                } else {
                    mask.step = Step::Falldown;
                    mask.begin_fall(actor.pos, solid);
                }
            }
            Step::Falldown => {
                let base = mask.placed(Vec3::ZERO, actor.pos);
                let centre = mask.placed(mask.norse(0), actor.pos);
                if mask.tumbled(centre, base) {
                    mask.step = Step::Wait;
                    mask.settle = SETTLE_FRAMES;
                    if mask.switch_no != NO_SWITCH {
                        switches.set(mask.switch_no);
                    }
                    events.list.push(SimEvent::Se(mask.break_sound()));
                }
            }
            Step::Wait => {
                if mask.settle > 0 {
                    mask.settle -= 1;
                    if mask.settle == 0 {
                        mask.step = Step::Done;
                        if mask.kind == TYPE_SLAB {
                            events.list.push(SimEvent::Se(SE_RIDDLE));
                        }
                    }
                }
            }
            Step::Done => {
                if mask.foe.is_some_and(|foe| !foes.contains(foe)) {
                    let _ = commands.entity(entity).despawn();
                    continue;
                }
            }
        }
        mask.lean();
        actor.pos += mask.fall();
        actor.angle = mask.angle();
    }
}
