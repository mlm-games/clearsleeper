use bevy_ecs::prelude::*;

pub const WBIRD_ACTORS: [&str; 1] = ["WBird"];

/// The stage string is `WBird` (`d_stage.cpp:941`) and the model's `_arcname` is also `WBird`
/// (`d_a_wbird.cpp:13`). Retail never places one: `daPy_lk_c::procTactPlayEnd_init`
/// (`d_a_player_tact.inc:686`) is the only creator, right after the Wind Waker's tune ends.
///
/// Every arm of `daWbird_c::execute` (`:170-187`) is gated on an input this sim does not have:
///
/// * `actionSelect` (`:123-164`) counts `field_0x29E` up and then switches on
///   `dComIfGp_getOperateWind()` (`:131`), i.e. `dComIf_control_c::mOperateWind`
///   (`d_com_inf_game.h:486`), which only `d_operate_wind.cpp:663-665` and `d_meter.cpp:4850`
///   write. Without it neither `case 1` (`setStartPos`, `:134`) nor `case 0` (`:152-157`)
///   is reachable, so the counter and the `JA_SE_SYS_WTAKT_WIND_AMB` loop (`:130`) never
///   resolve; `mOperateWind` also starts at 0 (`d_com_inf_game.cpp:136`), which is `case 0`.
/// * `setStartPos` (`:21-62`) needs `dKyw_get_wind_vec` (`:23`) and the environment light's
///   `mTactWindAngleX/Y` (`:45-47`) - the wind vectors the contract lists as confirmed-absent.
/// * `actionSelect` case 1 needs `dComIfGp_evmng_getEventIdx` (`:141-143`) for `TACT_WINDOW2`
///   / `TACT_WINDOW2_SHIP`, i.e. the stage event-info table, and
///   `player->setPlayerPosAndAngle` / `cancelOriginalDemo` (`:139`, `:147`), which is demo
///   control the sim does not model.
/// * `actionMove` (`:96-120`) needs `dKyw_custom_windpower` (`:109`, `:148`) and
///   `dKyw_tact_wind_set_go` (`:111-113`).
/// * `CreateInit` (`:65-72`) needs `fopAcM_orderChangeEvent(this, "TACT_WINDOW", 0, -1)`
///   (`:68`), which resolves through the same absent event-info table.
///
/// `actionWait` (`:82-83`) is empty and `calcMtx` (`:15-18`) only writes `eyePos` and the
/// attention position, so nothing is left to port: `daWbird_IsDelete` (`:201-203`) is a bare
/// TRUE, an accepted delete request rather than a self-destruct. `draw` (`:166-168`) returns
/// TRUE with no draw call, so the bird renders nothing and needs no `actor_views` branch even
/// though the profile does name `fopAc_CULLBOX_6_e` (`:239`).
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Wbird;

impl Wbird {
    pub fn spawn() -> Self {
        Self
    }
}

pub fn is_wbird(name: &str) -> bool {
    WBIRD_ACTORS.contains(&name)
}

pub fn control(_birds: Query<&Wbird>) {}
