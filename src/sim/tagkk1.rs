use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::player::Player;
use super::walk;

pub const TAGKK1_ACTORS: [&str; 1] = ["TagKk1"];

const HORIZONTAL: f32 = 350.0; // <- d_a_tag_kk1.cpp:35 (l_HIO.prm.mHorizontalDistance)
const VERTICAL: f32 = 30.0; // <- d_a_tag_kk1.cpp:36 (l_HIO.prm.mVerticalDistance)
const ANGLE_LIMIT: u16 = 0x1000; // <- d_a_tag_kk1.cpp:61

/// Poor Muuru (Mila) chase startup tag. `daTag_Kk1_c::_execute` (`d_a_tag_kk1.cpp:54-66`)
/// sets `mTagSet` every tick from the HIO ranges in `daTag_Kk1_HIO_c`'s constructor table
/// (`:34-38`); nothing in the decomp ever reads `mTagSet`, so `tag_set` is this actor's
/// whole observable state.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TagKk1 {
    pub tag_set: bool,
}

impl TagKk1 {
    pub fn spawn() -> Self {
        Self { tag_set: false }
    }

    fn facing(&self, pos: Vec3, yaw: i16, player: &Player) -> bool {
        if (player.pos - pos).length() >= HORIZONTAL {
            return false;
        }
        if player.pos.y - pos.y >= VERTICAL {
            return false;
        }
        walk::to_s16(player.face)
            .wrapping_sub(yaw)
            .unsigned_abs()
            < ANGLE_LIMIT
    }
}

pub fn is_tagkk1(name: &str) -> bool {
    TAGKK1_ACTORS.contains(&name)
}

pub fn control(players: Query<&Player>, mut tags: Query<(&Actor, &mut TagKk1)>) {
    let Some(player) = players.iter().next() else {
        return;
    };
    for (actor, mut tag) in &mut tags {
        let set = tag.facing(actor.pos, actor.angle[1], player);
        tag.tag_set = set;
    }
}