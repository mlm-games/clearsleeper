use bevy_ecs::prelude::*;

use super::actor::Actor;
use super::collision::Collision;
use super::flags::EventFlags;
use super::ground;
use super::stream;
use super::walk;

pub const SPCITEM_ACTORS: [&str; 1] = ["SPitem"];

pub const ITEM_JOY_PENDANT: u8 = 0x1F;
pub const ITEM_SHIELD: u8 = 0x3B;

pub const EVENT_SHIELD_REMOVED: u32 = 0x0E20;

pub const GRAVITY: f32 = -4.0;
pub const SHIELD_GRAVITY: f32 = 0.0;
pub const MAX_FALL: f32 = -100.0;
pub const GROUND_REFLECT: f32 = 0.62;
pub const BOUNCE_GATE: f32 = 0.5;

pub const SPIN_OFFSET: i16 = 0x2000;
pub const SPIN_STEP: i16 = 0x2000;

pub const CULL_MIN: [f32; 3] = [-40.0, 0.0, -40.0];
pub const CULL_MAX: [f32; 3] = [40.0, 125.0, 40.0];

const ITEM_MASK: u32 = 0xFF;
const RUPEES: [u8; 7] = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x0F];

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct SpcItem {
    pub item: u8,
    pub refused: bool,
    pub fall: f32,
    pub spin: i16,
    pub landings: u8,
    pub speed_y: f32,
    pub down: bool,
}

impl SpcItem {
    pub fn spawn(parameters: u32, flags: &EventFlags) -> Self {
        let item = (parameters & ITEM_MASK) as u8;
        Self {
            item,
            refused: item == ITEM_SHIELD && flags.test(EVENT_SHIELD_REMOVED),
            fall: 0.0,
            spin: 0,
            landings: 0,
            speed_y: 0.0,
            down: false,
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    fn gravity(&self) -> f32 {
        if self.item == ITEM_SHIELD {
            SHIELD_GRAVITY
        } else {
            GRAVITY
        }
    }

    fn rupee(&self) -> bool {
        RUPEES.contains(&self.item)
    }

    /// `daSpcItem01_c::move` (`d_a_spc_item01.cpp:173-205`). `fopAcM_posMoveF` is
    /// `stream::calc_speed` + `fopAcM_posMove` (`tww/src/f_op/f_op_actor_mng.cpp:452-484`), and
    /// `mStts.GetCCMoveP()` is the zeroed `cCcD_Stts::m_cc_move` because this translation unit
    /// never calls `cCcD_Stts::PlusCcMove`. `mAcch.CrrPos` is reduced to the merged-DZB ground
    /// test, keeping `dBgS_Acch::GroundCheck`'s "was not already resting" guard so a settled item
    /// reports a landing only every other tick (`tww/src/d/d_bg_s_acch.cpp:115,137-148`).
    fn step(&mut self, actor: &mut Actor, solid: Option<&Collision>) {
        let (travel, _) =
            stream::calc_speed(0.0, actor.angle[1], self.speed_y, self.gravity(), MAX_FALL);
        self.speed_y = travel.y;
        let step = travel;
        actor.pos += step;
        let was_down = self.down;
        let mut touched = false;
        if let Some(floor) = ground::surface(solid, actor.pos)
            && actor.pos.y <= floor
        {
            actor.pos.y = floor;
            self.speed_y = 0.0;
            self.down = true;
            touched = true;
        }
        if touched && !was_down {
            match self.item {
                ITEM_SHIELD => {}
                ITEM_JOY_PENDANT => self.speed_y = 0.0,
                _ => {
                    self.landings = self.landings.wrapping_add(1);
                    let bounce = self.fall * GROUND_REFLECT;
                    if bounce <= self.gravity() - BOUNCE_GATE {
                        self.speed_y = -bounce;
                    }
                }
            }
        }
        if self.speed_y != 0.0 {
            self.fall = self.speed_y;
        }
    }

    /// `daSpcItem01_c::rotate_item` (`d_a_spc_item01.cpp:208-217`).
    fn rotate(&mut self, actor: &mut Actor) {
        if !self.rupee() {
            return;
        }
        self.spin = if self.landings == 0 {
            actor.angle[0].wrapping_add(SPIN_OFFSET)
        } else {
            0
        };
        walk::chase_angle_s(&mut actor.angle[0], self.spin, SPIN_STEP);
    }
}

pub fn is_spcitem(name: &str) -> bool {
    SPCITEM_ACTORS.contains(&name)
}

pub fn control(
    collision: Option<Res<Collision>>,
    mut items: Query<(Entity, &mut Actor, &mut SpcItem)>,
    mut commands: Commands,
) {
    let solid = collision.as_deref();
    let mut gone: Vec<Entity> = Vec::new();
    for (entity, mut actor, mut item) in &mut items {
        if item.refused {
            gone.push(entity);
            continue;
        }
        item.step(&mut actor, solid);
        item.rotate(&mut actor);
    }
    for entity in gone {
        let _ = commands.entity(entity).despawn();
    }
}
