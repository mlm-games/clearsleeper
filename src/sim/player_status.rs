use bevy_ecs::prelude::*;

use super::intent::{ItemIntent, TargetIntent, UseIntent};
use super::player::Player;
use super::ship::Ship;
use super::swim::Swim;

pub const STTS0_UNK1: u32 = 0x0000_0001;
pub const STTS0_UNK2: u32 = 0x0000_0002;
pub const STTS0_UNK4: u32 = 0x0000_0004;
pub const STTS0_UNK10: u32 = 0x0000_0010;
pub const STTS0_UNK20: u32 = 0x0000_0020;
pub const STTS0_UNK40: u32 = 0x0000_0040;
pub const STTS0_UNK80: u32 = 0x0000_0080;
pub const STTS0_HANG: u32 = 0x0000_0100;
pub const STTS0_UNK200: u32 = 0x0000_0200;
pub const STTS0_UNK400: u32 = 0x0000_0400;
pub const STTS0_UNK800: u32 = 0x0000_0800;
pub const STTS0_BOW_AIM: u32 = 0x0000_1000;
pub const STTS0_SUBJECT: u32 = 0x0000_2000;
pub const STTS0_HOOKSHOT_AIM: u32 = 0x0000_4000;
pub const STTS0_SWORD_SWING: u32 = 0x0000_8000;
pub const STTS0_SHIP_RIDE: u32 = 0x0001_0000;
pub const STTS0_ROPE_AIM: u32 = 0x0002_0000;
pub const STTS0_UNK40000: u32 = 0x0004_0000;
pub const STTS0_BOOMERANG_AIM: u32 = 0x0008_0000;
pub const STTS0_SWIM: u32 = 0x0010_0000;
pub const STTS0_TELESCOPE_LOOK: u32 = 0x0020_0000;
pub const STTS0_BOOMERANG_WAIT: u32 = 0x0040_0000;
pub const STTS0_UNK800000: u32 = 0x0080_0000;
pub const STTS0_UNK1000000: u32 = 0x0100_0000;
pub const STTS0_UNK2000000: u32 = 0x0200_0000;
pub const STTS0_UNK4000000: u32 = 0x0400_0000;
pub const STTS0_CRAWL: u32 = 0x0800_0000;
pub const STTS0_UNK10000000: u32 = 0x1000_0000;
pub const STTS0_UNK20000000: u32 = 0x2000_0000;
pub const STTS0_SPIN_ATTACK: u32 = 0x4000_0000;
pub const STTS0_UNK80000000: u32 = 0x8000_0000;

pub const STTS0_JUDGEMENT: u32 = STTS0_UNK1
    | STTS0_UNK10
    | STTS0_UNK20
    | STTS0_UNK40
    | STTS0_HANG
    | STTS0_UNK200
    | STTS0_SUBJECT
    | STTS0_TELESCOPE_LOOK
    | STTS0_UNK800000
    | STTS0_UNK1000000
    | STTS0_UNK2000000
    | STTS0_UNK4000000
    | STTS0_UNK10000000
    | STTS0_UNK20000000;

pub const STTS1_WIND_WAKER_CONDUCT: u32 = 0x0000_0001;
pub const STTS1_UNK2: u32 = 0x0000_0002;
pub const STTS1_UNK4: u32 = 0x0000_0004;
pub const STTS1_UNK8: u32 = 0x0000_0008;
pub const STTS1_UNK10: u32 = 0x0000_0010;
pub const STTS1_UNK20: u32 = 0x0000_0020;
pub const STTS1_UNK40: u32 = 0x0000_0040;
pub const STTS1_UNK80: u32 = 0x0000_0080;
pub const STTS1_UNK100: u32 = 0x0000_0100;
pub const STTS1_UNK200: u32 = 0x0000_0200;
pub const STTS1_UNK400: u32 = 0x0000_0400;
pub const STTS1_UNK800: u32 = 0x0000_0800;
pub const STTS1_UNK1000: u32 = 0x0000_1000;
pub const STTS1_UNK2000: u32 = 0x0000_2000;
pub const STTS1_UNK4000: u32 = 0x0000_4000;
pub const STTS1_UNK8000: u32 = 0x0000_8000;
pub const STTS1_UNK10000: u32 = 0x0001_0000;
pub const STTS1_UNK20000: u32 = 0x0002_0000;
pub const STTS1_UNK40000: u32 = 0x0004_0000;
pub const STTS1_UNK80000: u32 = 0x0008_0000;
pub const STTS1_UNK100000: u32 = 0x0010_0000;
pub const STTS1_UNK200000: u32 = 0x0020_0000;
pub const STTS1_UNK400000: u32 = 0x0040_0000;
pub const STTS1_UNK800000: u32 = 0x0080_0000;
pub const STTS1_UNK1000000: u32 = 0x0100_0000;
pub const STTS1_UNK2000000: u32 = 0x0200_0000;
pub const STTS1_UNK4000000: u32 = 0x0400_0000;
pub const STTS1_UNK8000000: u32 = 0x0800_0000;
pub const STTS1_UNK10000000: u32 = 0x1000_0000;
pub const STTS1_UNK20000000: u32 = 0x2000_0000;
pub const STTS1_UNK40000000: u32 = 0x4000_0000;
pub const STTS1_UNK80000000: u32 = 0x8000_0000;

#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PlayerStatus {
    pub status0: u32,
    pub status1: u32,
}

impl PlayerStatus {
    pub fn check0(&self, flag: u32) -> bool {
        self.status0 & flag != 0
    }

    pub fn check1(&self, flag: u32) -> bool {
        self.status1 & flag != 0
    }
}

pub fn control(
    players: Query<&Player>,
    swims: Query<&Swim>,
    ships: Query<&Ship>,
    targeting: Res<TargetIntent>,
    item: Res<ItemIntent>,
    use_intent: Res<UseIntent>,
    mut status: ResMut<PlayerStatus>,
    mut info: ResMut<crate::text::msgproc::MsgGameInfo>,
) {
    let Some(player) = players.iter().next() else {
        return;
    };
    let swimming = swims.iter().next().is_some_and(|swim| swim.active);
    let riding = ships.iter().any(|ship| ship.riding);

    let mut bits = 0u32;
    if riding {
        bits |= STTS0_SHIP_RIDE;
    }
    if swimming {
        bits |= STTS0_SWIM;
    }
    if targeting.0 {
        bits |= STTS0_BOOMERANG_AIM;
    }
    if item.0 {
        bits |= STTS0_BOOMERANG_WAIT;
    }
    if use_intent.0 {
        bits |= STTS0_HOOKSHOT_AIM;
    }
    if player.face > 0.9 {
        bits |= STTS0_CRAWL;
    }

    status.status0 = bits;
    info.player_status0 = bits;
}

pub fn init(world: &mut World) {
    world.init_resource::<PlayerStatus>();
}
