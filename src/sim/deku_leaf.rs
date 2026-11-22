use bevy_ecs::prelude::*;

use super::ground::grounded;
use super::health::Magic;
use super::intent::{DoIntent, UseIntent};
use super::inventory::{Equipped, ITEM_DEKU_LEAF, SLOT_DEKU_LEAF};
use super::item_use::{self, ItemUse};
use super::player::Player;

/// `daPyProc_FAN_SWING_e`/`daPyProc_FAN_GLIDE_e` and the glide's magic clock
/// (`tww/src/d/actor/d_a_player_fan.inc:183-205,329-363`;
/// `wwhd/wwhd_src/d/actor/d_a_player_main_02.cpp:1435-1455`,
/// `wwhd/wwhd_src/d/actor/d_a_player_main_05.cpp:103-133`).
#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FanPhase {
    #[default]
    Idle,
    Swing,
    Glide,
}

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct Fan {
    pub phase: FanPhase,
    pub ticks: u32,
    pub gust_done: bool,
    pub magic_clock: u32,
    press: bool,
}

/// `frameCtrl.getFrame() >= 22.0` in the swing (`tww/src/d/actor/d_a_player_fan.inc:243`,
/// `wwhd/wwhd_src/d/actor/d_a_player_main_07.cpp:3272`) at the
/// port's two frames per tick.
pub const GUST_FRAME: u32 = 11;
pub const MAGIC_PERIOD: u32 = 40;
pub const LIFT: f32 = 15.0;
pub const GRAVITY: f32 = -0.5;
pub const FALL_CAP: f32 = -2.0;
pub const SPEED: f32 = 12.0;

pub fn control(
    use_intent: Res<UseIntent>,
    do_intent: Res<DoIntent>,
    equipped: Res<Equipped>,
    mut magic: ResMut<Magic>,
    mut use_state: ResMut<ItemUse>,
    mut fan: ResMut<Fan>,
    collision: Option<Res<super::collision::Collision>>,
    mut players: Query<&mut Player>,
) {
    let has_leaf = equipped.item(SLOT_DEKU_LEAF) == ITEM_DEKU_LEAF;
    let press = use_intent.0 && !fan.press;
    fan.press = use_intent.0;
    let air = players
        .iter()
        .next()
        .is_some_and(|p| !grounded(collision.as_deref(), p.pos));

    match fan.phase {
        FanPhase::Idle => {
            if press && has_leaf {
                if air && magic.units >= 1 {
                    fan.phase = FanPhase::Glide;
                    fan.ticks = 0;
                    fan.magic_clock = 0;
                } else if !air {
                    fan.phase = FanPhase::Swing;
                    fan.ticks = 0;
                    fan.gust_done = false;
                }
                // A failed airborne glide plays JA_SE_ITEM_TARGET_OUT (`fan.inc:114`),
                // which has no port cue id; it is presentation only and left out.
            }
        }
        FanPhase::Swing => {
            fan.ticks += 1;
            if fan.ticks >= GUST_FRAME && !fan.gust_done {
                fan.gust_done = true;
            }
            if fan.ticks >= item_use::FAN_TICKS {
                fan.phase = FanPhase::Idle;
            }
        }
        FanPhase::Glide => {
            fan.ticks += 1;
            fan.magic_clock += 1;
            if fan.magic_clock >= MAGIC_PERIOD {
                fan.magic_clock = 0;
                magic.units = magic.units.saturating_sub(1);
            }
            if magic.units == 0 || do_intent.0 || !air {
                fan.phase = FanPhase::Idle;
                use_state.stem = None;
            }
        }
    }

    if fan.phase == FanPhase::Glide {
        use_state.stem = Some("usefanb");
    }

    for mut player in &mut players {
        let player = &mut *player;
        if fan.phase == FanPhase::Glide {
            if fan.ticks == 1 {
                player.speed_y = LIFT;
                player.speed = SPEED;
            }
            player.speed_y = (player.speed_y + GRAVITY).max(FALL_CAP);
            player.pos.y += player.speed_y;
        }
    }
}
