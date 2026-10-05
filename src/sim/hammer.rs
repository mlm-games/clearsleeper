use bevy_ecs::prelude::*;
use glam::Vec3;

use super::collision::Collision;
use super::ground;
use super::hop::Hop;
use super::intent::{CameraYaw, DoIntent, MoveIntent};
use super::inventory::{Equipped, HAMMER_SLOT, ITEM_SKULL_HAMMER};
use super::jump::Jump;
use super::lock::Lock;
use super::player::Player;
use super::roll::Roll;
use super::ship::Ship;
use super::walk;

pub const AT_SKULL_HAMMER: u32 = 1 << 16;

pub const SE_ON: u32 = 0x69B2;
pub const SE_OFF: u32 = 0x69B7;
pub const SE_SWING: u32 = 0x69B8;

pub const CUT_NONE: u8 = 0x00;
pub const CUT_HAMMER_SIDESWING: u8 = 0x11;
pub const CUT_HAMMER_FRONTSWING: u8 = 0x12;

pub const AT_POWER: i32 = 4;
pub const AT_RADIUS: f32 = 50.0;

pub const TURN_SCALE: i16 = 30;
pub const TURN_MAX_STEP: i16 = 0x3CDF;
pub const TURN_MIN_STEP: i16 = 0x1F40;

pub const SIDE_RATE: f32 = 1.2;
pub const SIDE_START: f32 = 4.0;
pub const SIDE_END: i16 = 60;
pub const SIDE_OPEN: f32 = 25.0;
pub const SIDE_CLOSE: f32 = 36.0;
pub const SIDE_CANCEL: f32 = 55.0;

pub const READY_RATE: f32 = 1.0;
pub const READY_START: f32 = 2.0;
pub const READY_END: i16 = 28;

pub const FRONT_RATE: f32 = 4.0;
pub const FRONT_START: f32 = 0.0;
pub const FRONT_END: i16 = 35;

pub const END_RATE: f32 = 0.9;
pub const END_START: f32 = 5.0;
pub const END_FRAME: i16 = 12;
pub const END_CANCEL: f32 = 10.0;

pub const STICK_GATE: f32 = 0.05;
pub const DIR_FORWARD: i32 = 0;
pub const DIR_BACKWARD: i32 = 1;
pub const DIR_LEFT: i32 = 2;
pub const DIR_RIGHT: i32 = 3;

pub const DIR_BACKWARD_LIMIT: u16 = 0x6000;
pub const DIR_SIDE_LIMIT: i16 = 0x2000;
pub const DONE: f32 = 0.01;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Swing {
    Idle,
    FrontReady,
    Front,
    FrontEnd,
    Side,
}

#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Cut {
    pub kind: u8,
    pressed: bool,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Hammer {
    pub swing: Swing,
    pub frame: f32,
    pub rate: f32,
    pub end: i16,
    pub heading: i16,
}

impl Hammer {
    pub fn idle() -> Self {
        Self {
            swing: Swing::Idle,
            frame: 0.0,
            rate: 0.0,
            end: 0,
            heading: 0,
        }
    }

    pub fn begin(&mut self, swing: Swing, end: i16, rate: f32, start: f32, heading: i16) {
        self.swing = swing;
        self.end = end;
        self.rate = rate;
        self.frame = start;
        self.heading = heading;
    }

    pub fn halt(&mut self) {
        self.swing = Swing::Idle;
        self.frame = 0.0;
        self.rate = 0.0;
    }

    pub fn advance(&mut self) {
        if self.rate < DONE {
            return;
        }
        self.frame += self.rate;
        if self.frame > (self.end) as f32 {
            self.rate = 0.0;
        }
    }

    pub fn done(&self) -> bool {
        self.rate < DONE
    }

    pub fn cut(&self) -> u8 {
        match self.swing {
            Swing::Side => CUT_HAMMER_SIDESWING,
            Swing::FrontReady | Swing::Front | Swing::FrontEnd => CUT_HAMMER_FRONTSWING,
            Swing::Idle => CUT_NONE,
        }
    }

    pub fn striking(&self) -> bool {
        match self.swing {
            Swing::Side => self.frame >= SIDE_OPEN && self.frame < SIDE_CLOSE,
            Swing::Front => true,
            Swing::FrontReady | Swing::FrontEnd | Swing::Idle => false,
        }
    }

    pub fn swinging(&self) -> bool {
        self.swing != Swing::Idle
    }

    pub fn cancelling(&self) -> bool {
        match self.swing {
            Swing::Side => self.frame > SIDE_CANCEL,
            Swing::FrontEnd => self.frame > END_CANCEL,
            _ => false,
        }
    }
}

pub fn direction_of(angle: i16) -> i32 {
    if angle.unsigned_abs() > DIR_BACKWARD_LIMIT {
        DIR_BACKWARD
    } else if angle >= DIR_SIDE_LIMIT {
        DIR_LEFT
    } else if angle <= -DIR_SIDE_LIMIT {
        DIR_RIGHT
    } else {
        DIR_FORWARD
    }
}

pub fn stick_direction(intent: &MoveIntent, camera: &CameraYaw, facing: i16) -> i32 {
    direction_of(walk::target_yaw(intent, camera).wrapping_sub(facing))
}

pub fn side_direction(direction: i32) -> bool {
    direction == DIR_LEFT || direction == DIR_RIGHT
}

pub fn strike_origin(player: &Player) -> Vec3 {
    Vec3::new(
        player.pos.x + AT_RADIUS * player.yaw.sin(),
        player.pos.y,
        player.pos.z + AT_RADIUS * player.yaw.cos(),
    )
}

pub fn turn(player: &Player, intent: &MoveIntent, camera: &CameraYaw, locked: bool) -> i16 {
    let held = walk::to_s16(player.face);
    if locked {
        return held;
    }
    let target = if intent.axis.length() <= STICK_GATE {
        held
    } else {
        walk::target_yaw(intent, camera)
    };
    walk::add_calc_angle_s(held, target, TURN_SCALE, TURN_MAX_STEP, TURN_MIN_STEP)
}

fn launch(
    hammer: &mut Hammer,
    player: &mut Player,
    intent: &MoveIntent,
    camera: &CameraYaw,
    locked: bool,
) {
    let heading = turn(player, intent, camera, locked);
    player.face = walk::from_s16(heading);
    player.yaw = player.face;
    let side = side_direction(stick_direction(intent, camera, heading));
    if side {
        hammer.begin(Swing::Side, SIDE_END, SIDE_RATE, SIDE_START, heading);
    } else {
        hammer.begin(Swing::FrontReady, READY_END, READY_RATE, READY_START, heading);
    }
}

pub fn control(
    collision: Option<Res<Collision>>,
    equipped: Res<Equipped>,
    intent: Res<MoveIntent>,
    camera: Res<CameraYaw>,
    do_intent: Res<DoIntent>,
    lock: Res<Lock>,
    roll: Res<Roll>,
    hop: Res<Hop>,
    jump: Res<Jump>,
    ships: Query<&Ship>,
    mut cut: ResMut<Cut>,
    mut players: Query<(&mut Player, &mut Hammer)>,
) {
    let solid = collision.as_deref();
    let tapped = do_intent.0 && !cut.pressed;
    cut.pressed = do_intent.0;
    cut.kind = CUT_NONE;
    let armed = equipped.item(HAMMER_SLOT) == ITEM_SKULL_HAMMER
        && !roll.active
        && !hop.active
        && !jump.active
        && !ships.iter().any(|ship| ship.riding);
    let locked = lock.target.is_some();
    for (mut player, mut hammer) in &mut players {
        if !armed {
            hammer.halt();
            cut.kind = CUT_NONE;
            continue;
        }
        if hammer.swinging() {
            player.speed = 0.0;
            let heading = turn(&player, &intent, &camera, locked);
            player.face = walk::from_s16(heading);
            player.yaw = player.face;
        }
        match hammer.swing {
            Swing::Idle => {
                if tapped && ground::grounded(solid, player.pos) {
                    launch(&mut hammer, &mut player, &intent, &camera, locked);
                }
            }
            Swing::FrontReady => {
                hammer.advance();
                if hammer.done() {
                    hammer.begin(
                        Swing::Front,
                        FRONT_END,
                        FRONT_RATE,
                        FRONT_START,
                        hammer.heading,
                    );
                }
            }
            Swing::Side => {
                hammer.advance();
                if hammer.done() || hammer.cancelling() {
                    hammer.halt();
                }
            }
            Swing::Front => {
                hammer.advance();
                if hammer.done() {
                    hammer.begin(
                        Swing::FrontEnd,
                        END_FRAME,
                        END_RATE,
                        END_START,
                        hammer.heading,
                    );
                }
            }
            Swing::FrontEnd => {
                hammer.advance();
                if hammer.done() || hammer.cancelling() {
                    hammer.halt();
                }
            }
        }
        cut.kind = hammer.cut();
    }
}