use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::arrow::{Arrow, Phase, TYPE_FIRE};
use super::barricade::{AT_BOMB, AT_FIRE_ARROW, AT_SKULL_HAMMER};
use super::events::{SimEvent, SimEvents};
use super::flags::Switches;
use super::hammer::{self, Cut, Hammer};
use super::item::Bomb;
use super::pickup::{ITEM_ACTORS, Pickup};
use super::player::Player;
use super::walk;

pub const LEAF_ACTORS: [&str; 1] = ["Vochi"];

pub const NO_SWITCH: u8 = 0xFF;

pub const SPH_RADIUS: f32 = 150.0;
pub const SPH_DROP: f32 = 115.0;
pub const HIT_REACH: f32 = 110.0;

pub const CULL_HALF: f32 = 75.0;
pub const CULL_TOP: f32 = 150.0;

pub const ITEM_NONE: u8 = 0x3F;
pub const TABLE_LOW: u8 = 0x20;
pub const TABLE_HIGH: u8 = 0x3E;
pub const ITEM_DROP: f32 = 30.0;
pub const ITEM_SWITCH: u32 = 0xFF;
pub const ITEM_ACTION_SHIFT: u32 = 26;
pub const ITEM_ACTION: u32 = 1;

pub const OPAQUE: f32 = 255.0;
pub const ITEM_TOP: f32 = 200.0;
pub const TRANSPARENT: f32 = 0.0;

pub const BLOW_FADE: f32 = -12.0;
pub const BLOW_WAIT: i32 = 0;
pub const BLOW_LIFE: i32 = 1;

pub const BURN_FADE: f32 = -3.0;
pub const BURN_WAIT: i32 = 60;
pub const BURN_LIFE: i32 = 120;

pub const SE_BLOW: u32 = 0x69EF;
pub const SE_BURN: u32 = 0x69F0;

const ITEM_SHIFT: u32 = 0;
const ITEM_MASK: u32 = 0x3F;
const BIT_SHIFT: u32 = 6;
const BIT_MASK: u32 = 0x7F;
const SAVE_SHIFT: u32 = 13;
const SAVE_MASK: u32 = 0xFF;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Wait,
    Alpha,
    Item,
    Retire,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Leaves {
    pub appear: bool,
    pub switch_no: u8,
    pub item: u8,
    pub item_bit: u8,
    pub step: Step,
    pub alpha: f32,
    pub fade: f32,
    pub wait: i32,
    pub life: i32,
}

impl Leaves {
    pub fn spawn(parameters: u32, switches: &Switches) -> Self {
        let switch_no = ((parameters >> SAVE_SHIFT) & SAVE_MASK) as u8;
        Self {
            appear: switch_no == NO_SWITCH || !switches.test(switch_no),
            switch_no,
            item: ((parameters >> ITEM_SHIFT) & ITEM_MASK) as u8,
            item_bit: ((parameters >> BIT_SHIFT) & BIT_MASK) as u8,
            step: Step::Wait,
            alpha: OPAQUE,
            fade: 0.0,
            wait: 0,
            life: 0,
        }
    }

    pub fn centre(&self, pos: Vec3) -> Vec3 {
        Vec3::new(pos.x, pos.y - SPH_DROP, pos.z)
    }

    pub fn cull(&self) -> (f32, f32) {
        (CULL_HALF, CULL_TOP)
    }

    pub fn drawn(&self) -> bool {
        self.appear && self.alpha as u8 != 0
    }

    pub fn latch(&self, switches: &mut Switches) {
        if self.switch_no != NO_SWITCH {
            switches.set(self.switch_no);
        }
    }

    fn ignite(&mut self, burning: bool) {
        let (fade, wait, life) = if burning {
            (BURN_FADE, BURN_WAIT, BURN_LIFE)
        } else {
            (BLOW_FADE, BLOW_WAIT, BLOW_LIFE)
        };
        self.fade = fade;
        self.wait = wait;
        self.life = life;
        self.step = Step::Alpha;
    }

    pub fn struck(&mut self, at: u32, hit: Vec3, pos: Vec3) -> Option<u32> {
        if self.step != Step::Wait {
            return None;
        }
        match at {
            AT_BOMB => {
                if !walk::within(hit, self.centre(pos), SPH_RADIUS) {
                    return None;
                }
                self.ignite(false);
                Some(SE_BLOW)
            }
            AT_FIRE_ARROW => {
                if !walk::within(hit, self.centre(pos), SPH_RADIUS) {
                    return None;
                }
                self.ignite(true);
                Some(SE_BURN)
            }
            AT_SKULL_HAMMER => {
                if !xz_within(hit, pos, HIT_REACH) {
                    return None;
                }
                Some(SE_BLOW)
            }
            _ => None,
        }
    }

    fn advance(&mut self) {
        self.wait = self.wait.saturating_sub(1);
        self.life = self.life.saturating_sub(1);
    }

    fn item_parameter(&self) -> u32 {
        u32::from(self.item)
            | (u32::from(self.item_bit) << 8)
            | (ITEM_SWITCH << 16)
            | (ITEM_ACTION << ITEM_ACTION_SHIFT)
    }

    fn release(&self, actor: &Actor, commands: &mut Commands) {
        if self.item == ITEM_NONE || (TABLE_LOW..=TABLE_HIGH).contains(&self.item) {
            return;
        }
        let parameters = self.item_parameter();
        commands.spawn((
            Actor {
                name: ITEM_ACTORS[0].to_owned(),
                parameters,
                pos: actor.pos + Vec3::new(0.0, -ITEM_DROP, 0.0),
                angle: [0; 3],
                set_id: 0,
                room: actor.room,
            },
            Pickup::spawn(parameters),
        ));
    }
}

fn xz_within(from: Vec3, to: Vec3, reach: f32) -> bool {
    let dx = to.x - from.x;
    let dz = to.z - from.z;
    dx * dx + dz * dz <= reach * reach
}

pub fn is_leaves(name: &str) -> bool {
    LEAF_ACTORS.contains(&name)
}

pub fn control(
    players: Query<&Player>,
    hammers: Query<&Hammer>,
    cut: Res<Cut>,
    arrows: Query<&Arrow>,
    blasts: Query<&Bomb>,
    mut piles: Query<(Entity, &Actor, &mut Leaves)>,
    mut switches: ResMut<Switches>,
    mut events: ResMut<SimEvents>,
    mut commands: Commands,
) {
    let player = players.iter().next().copied();
    let striking = cut.kind != hammer::CUT_NONE && hammers.iter().any(|hammer| hammer.striking());
    let head = player.map(|player| hammer::strike_origin(&player));
    for (entity, actor, mut pile) in &mut piles {
        if !pile.appear {
            let _ = commands.entity(entity).despawn();
            continue;
        }
        let mut sound = None;
        if pile.step == Step::Wait {
            for arrow in &arrows {
                if arrow.kind != TYPE_FIRE || arrow.phase != Phase::Flying {
                    continue;
                }
                let travel = arrow.pos - arrow.speed;
                if walk::swept(travel, arrow.pos, pile.centre(actor.pos), SPH_RADIUS) {
                    sound = pile.struck(AT_FIRE_ARROW, arrow.pos, actor.pos);
                    break;
                }
            }
            if sound.is_none() {
                for blast in &blasts {
                    if blast.fuse != 0 {
                        continue;
                    }
                    sound = pile.struck(AT_BOMB, blast.pos, actor.pos);
                    break;
                }
            }
            if sound.is_none() && striking && let Some(head) = head {
                sound = pile.struck(AT_SKULL_HAMMER, head, actor.pos);
            }
            if let Some(sound) = sound {
                events.list.push(SimEvent::Se(sound));
                pile.latch(&mut switches);
            }
        }
        pile.advance();
        match pile.step {
            Step::Wait => {}
            Step::Alpha => {
                if pile.wait <= 0 {
                    pile.step = Step::Item;
                }
            }
            Step::Item => {
                pile.alpha += pile.fade;
                if pile.alpha <= ITEM_TOP {
                    pile.release(actor, &mut commands);
                    pile.step = Step::Retire;
                }
            }
            Step::Retire => {
                pile.alpha += pile.fade;
                if pile.alpha <= TRANSPARENT {
                    pile.alpha = TRANSPARENT;
                    if pile.life <= 0 {
                        let _ = commands.entity(entity).despawn();
                    }
                }
            }
        }
    }
}
