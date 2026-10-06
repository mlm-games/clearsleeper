use bevy_ecs::prelude::*;

use super::actor::Actor;
use super::player::Player;
use super::tsubo::Tsubo;

pub const ACORN_LEAF_ACTORS: [&str; 1] = ["VigaH"];

pub fn is_acorn_leaf(name: &str) -> bool {
    ACORN_LEAF_ACTORS.contains(&name)
}

const ACORN_OFFSET_Y: f32 = 15.0;
const RESPAWN_FRAMES: i32 = 60;
const SPIN_TRIGGER: i32 = 69;
const SPIN_END: i32 = 70;
const PLAYER_NEAR: f32 = 50.0;
const APPROACH_STEP: f32 = 3.0;
const HEIGHT_TOLERANCE: f32 = 5.0;
const GRAB_NEAR: f32 = 200.0;
const GRAB_TARGET_NEAR: f32 = 70.0;
const ITEM_NO_MASK: u32 = 0x3F;
const ITEM_BIT_SHIFT: u32 = 6;
const ITEM_BIT_MASK: u32 = 0x7F;
const ACORN_TRUE: i32 = 0x4;
const ACORN_FALSE: i32 = 0x3F;
const ACORN_KIND_SHIFT: u32 = 8;
const ACORN_BIT_SHIFT: u32 = 16;
const ACORN_MARK: u32 = 7 << 24;
const ACORN_TOP: u32 = 1 << 31;

pub fn make_acorn_params(hanging: bool, item_no: u32, bit_no: u32) -> u32 {
    let kind = if hanging { ACORN_TRUE } else { ACORN_FALSE };
    ((bit_no & ITEM_BIT_MASK) << ACORN_BIT_SHIFT)
        | ((item_no & ITEM_NO_MASK) | ((kind as u32) << ACORN_KIND_SHIFT))
        | ACORN_MARK
        | ACORN_TOP
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Leaf {
    #[default]
    Idle,
    Gone,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct AcornLeaf {
    pub leaf: Leaf,
    pub item_no: u32,
    pub item_bit_no: u32,
    pub cooldown: i32,
    pub spin: i32,
    pub spinning: bool,
    pub held: bool,
    pub grabbed: u32,
    pub last_distance: f32,
}

impl AcornLeaf {
    pub fn spawn(parameters: u32) -> Self {
        Self {
            leaf: Leaf::Idle,
            item_no: parameters & ITEM_NO_MASK,
            item_bit_no: (parameters >> ITEM_BIT_SHIFT) & ITEM_BIT_MASK,
            cooldown: 0,
            spin: 0,
            spinning: false,
            held: false,
            grabbed: 0xFFFF_FFFF,
            last_distance: 0.0,
        }
    }
}

pub fn control(
    mut leaves: Query<(&Actor, &mut AcornLeaf)>,
    players: Query<&Player>,
    items: Query<&Tsubo>,
) {
    let Some(player) = players.iter().next().copied() else {
        return;
    };
    let grabbed: u32 = 0xFFFF_FFFF;
    let live_items = items.iter().count();
    for (actor, mut leaf) in leaves.iter_mut() {
        if leaf.leaf == Leaf::Gone {
            continue;
        }
        if leaf.cooldown == 0 && live_items == 0 {
            leaf.held = true;
            leaf.cooldown = RESPAWN_FRAMES;
        }
        if leaf.cooldown > 0 {
            leaf.cooldown -= 1;
        }

        let distance = horizontal_distance(player.pos, actor.pos);
        if distance < PLAYER_NEAR
            && (leaf.last_distance - distance) > APPROACH_STEP
            && (player.pos.y - actor.pos.y).abs() < HEIGHT_TOLERANCE
        {
            leaf.spinning = true;
        }
        if distance < GRAB_NEAR && leaf.grabbed != grabbed {
            leaf.spinning = true;
        }
        if live_items > 0 {
            if leaf.spin < SPIN_TRIGGER {
                leaf.spin += 1;
            }
            if leaf.spin == SPIN_TRIGGER {
                leaf.held = false;
                leaf.spin = SPIN_END;
            }
        }
        if leaf.spinning && leaf.spin == SPIN_END {
            leaf.spinning = false;
        }
        leaf.grabbed = grabbed;
        leaf.last_distance = distance;
    }
}

fn horizontal_distance(from: glam::Vec3, to: glam::Vec3) -> f32 {
    let dx = from.x - to.x;
    let dz = from.z - to.z;
    (dx * dx + dz * dz).sqrt()
}

pub fn spawn_offset() -> glam::Vec3 {
    glam::Vec3::new(0.0, ACORN_OFFSET_Y, 0.0)
}

pub fn spawn(world: &mut World, source: &crate::dzs::StageActor) -> Entity {
    let actor = Actor {
        name: source.name.clone(),
        parameters: source.parameters,
        pos: glam::Vec3::from_array(source.pos),
        angle: source.angle,
        set_id: source.set_id,
        room: source.room,
        ..Default::default()
    };
    let leaf = AcornLeaf::spawn(source.parameters);
    world.spawn((actor, leaf)).id()
}