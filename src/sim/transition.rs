use bevy_ecs::prelude::*;

use super::collision::{Collision, NO_EXIT};
use super::ground::grounded;
use super::health::Health;
use super::player::Player;

/// `dComIfGp_setNextStage`'s `i_layer = -1` default (`d_com_inf_game.h:2245`): the
/// layer is left to `dComIfG_play_c::getLayerNo` (`d_com_inf_game.cpp:184-216`), which
/// derives it from the story bits and the clock. Any other value is the reference's own
/// "start the stage on this layer" arm and is returned as it stands.
pub const LAYER_NONE: i8 = -1;

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct NextStage {
    pub exit: Option<u8>,
    pub target: Option<Target>,
    /// `dStage_nextStage_c::mLayer` (`d_stage.h:945-952`), the fourth argument of the
    /// only other call that reads it, `dComIfGp_setNextStage`
    /// (`d_com_inf_game.h:2245`). [`super::layer::no`] models the `getLayerNo` arm for
    /// [`LAYER_NONE`]; a start layer from here wins over it for the whole stage
    /// session, which is what `dComIfGp_getStartStageLayer` gives the reference.
    pub layer: i8,
    /// `dLib_setNextStageBySclsNum(i_sclsnum, room_no)` (`d_lib.cpp:237-256`): the scene number
    /// and the room whose SCLS table resolves it (`room_no == -1` means the stage's own table,
    /// `:242-246`). The reference reads that table on the spot; the sim cannot - the tables are
    /// parsed from the stage and room archives, which the shell owns - so the request is
    /// published here and [`App::apply_exit`](crate::App::apply_exit) resolves it into a
    /// [`Target`]. A scene number the table does not carry resolves to nothing, which is the
    /// reference's own out-of-range arm (`:250-256`) and not a stand-in.
    pub scls: Option<(u8, i8)>,
}

impl Default for NextStage {
    fn default() -> Self {
        Self {
            exit: None,
            target: None,
            layer: LAYER_NONE,
            scls: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub stage: String,
    pub room: u8,
    pub start: u8,
}

impl NextStage {
    /// `dComIfGp_setNextStage(name, point, room, layer, 0.0f, 0, 1, 0)`: the whole
    /// request, for the callers that name a layer. The trailing arguments this seam does not
    /// carry, each with its own reader named:
    ///
    /// * `i_lastSpeed` and `i_lastMode` are `dSv_restart_c::setLastSceneInfo`'s pair
    ///   (`d_com_inf_game.cpp:692`). `dComIfGp_getLastMode` (`d_com_inf_game.h:1437`) and
    ///   `getLastSpeedF` (`:1433`) have no caller anywhere in the checkout, so those two save
    ///   fields have no reader to model.
    /// * `i_setPoint` is `dComIfGs_setStartPoint(i_point)` (`:693-694`), the save's start point;
    ///   this port's save record has no start-point field.
    /// * `i_wipe` is `dStage_nextStage_c::mWipe` (`d_stage.h:975`), which `dScnPly_Draw` maps
    ///   through `l_wipeType` to a scene overlap and a fade colour (`d_s_play.cpp:307-341`). This
    ///   port's stage swap is immediate and has no overlap channel.
    ///
    /// The `Hour` a cut can name beside these is NOT one of them: it is `dKy_set_nexttime`
    /// (`d_event_data.cpp:45-46`) into the pending slot
    /// [`DayNight::next_time`](super::daynight::DayNight::next_time), which the stage mount
    /// consumes.
    pub fn set(&mut self, stage: &str, room: u8, start: u8, layer: i8) {
        self.target = Some(Target {
            stage: stage.to_owned(),
            room,
            start,
        });
        self.layer = layer;
    }

    /// `dLib_setNextStageBySclsNum(i_sclsnum, room_no)` (`d_lib.cpp:237-256`), requested rather
    /// than resolved - see [`NextStage::scls`]. The reference hands
    /// `dComIfGp_setNextStage(entry->mStage, entry->mStart, entry->mRoom, -1, 0.0f, 0, TRUE,
    /// wipe)` (`:253-255`), whose trailing arguments carry the same boundaries
    /// [`NextStage::set`] names, and `dStage_sclsInfo_getWipe`'s `& 0xF` (`d_stage.h:1086-1088`)
    /// only feeds the `wipe == 0xFF ? 0 : wipe` the shell has no channel for.
    pub fn set_by_scls(&mut self, scls: u8, room: i8) {
        self.scls = Some((scls, room));
    }
}

pub fn check(
    collision: Option<Res<Collision>>,
    health: Res<Health>,
    mut next: ResMut<NextStage>,
    players: Query<&Player>,
) {
    for player in &players {
        if health.units == 0 {
            next.exit = None;
            continue;
        }
        let Some(collision) = collision.as_deref() else {
            next.exit = None;
            continue;
        };
        if !grounded(Some(collision), player.pos) {
            next.exit = None;
            continue;
        }
        next.exit = match collision.exit_at(player.pos.x, player.pos.z) {
            Some(exit) if exit != NO_EXIT => Some(exit),
            _ => None,
        };
    }
}
