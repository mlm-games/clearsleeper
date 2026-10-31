//! The telescope, `daPy_lk_c::procScope_init` / `procScope` (`d_a_player_main.cpp:5904-5933`).
//!
//! The item button's telescope arm enters it (`d_a_player_main.cpp:4086-4088`, `:4247-4249`:
//! `if (checkPhotoBoxItem(mEquipItem) || mEquipItem == dItemNo_TELESCOPE_e) return
//! procScope_init(mEquipItem);`). The init sets `mNormalSpeed` to zero, faces the player along the
//! camera (`current.angle.y = shape_angle.y`), clears `daPyFlg0_SCOPE_CANCEL` and, for the
//! telescope arm's `0x20`, raises `daPyStts0_TELESCOPE_LOOK_e` (`:5905-5921`), which is the flag
//! every telescope consumer reads: the camera's mode choice (`d_camera.cpp:1297-1299`), the
//! scope HUD (`d_scope.cpp:332`), Aryll's scope conversation (`d_a_npc_ls1.cpp:1802`) and the
//! message system's `getScopeMode` (`d_msg.cpp`).
//!
//! The tick only does two things (`:5922-5931`): [`check_scope_end`] leaves for the wait process,
//! and otherwise the player's own yaw is carried onto the camera's (`shape_angle.y =
//! fopCamM_GetAngleY(...)` while `dCamAttnStts_00000010_e` is set), so the body follows the view.
//!
//! The camera half is `dCamera_c`'s mode `0xe`, which the telescope flag selects
//! (`d_camera.cpp:1297-1299`) and which `subjectCamera` serves with its scoped block
//! (`:4840-4870`): the field of view walks between `mWork.subject.m39C` — two thirds of the
//! style's own field of view, `:4890-4893` — and `m3A0` as `mItemScopeWipeScale` grows from one
//! to three, and the viewport is cut to the scope's square (`:4907-4912`). [`camera::track`]'s
//! scope branch publishes that first-person view here; the HUD's ring, arrow and message panes
//! (`d_scope.cpp`) have no port yet.
//!
//! The zoom scale `dComIfGp_getCameraZoomScale(0)` runs from `1.0` to `9.0`
//! (`d_picture_box.cpp:454-507`), and Aryll's conversation reads two thresholds off it: `2.0`
//! for "you can see him" and `9.0` for the focus (`d_a_npc_ls1.cpp:41-42`, `:1848`, `:1858`).
use bevy_ecs::prelude::*;
use glam::{Vec2, Vec3};

use super::combat::Combat;
use super::event::ActiveEvent;
use super::intent::{CStick, MoveIntent, PadButtons, UseIntent};
use super::inventory::{Equipped, NO_ITEM};
use super::player::Player;
use super::ship::Ship;
use super::swim::Swim;
use super::talk::Dialogue;
use crate::text::msgproc::MsgGameInfo;
use crate::text::{STATUS_SCOPE_ACTIVE, STATUS_UNK0};

/// `dItemNo_TELESCOPE_e` (`d_item_data.h:63`-era table, the same `0x20` `aryll.rs` hands over).
pub const TELESCOPE_ITEM: u8 = 0x20;
/// The item screen's telescope slot: `inventory::SLOT_ITEMS` maps `0x20` to slot 0.
pub const TELESCOPE_SLOT: usize = 0;

/// `getHeight`'s player height, the eye the camera's scoped block places
/// (`d_a_player_main.cpp:5905-5921` leaves the position alone; `d_camera.cpp:4840-4870` builds the
/// eye from the player's own).
pub const EYE_HEIGHT: f32 = 125.0;

/// `dComIfGp_getCameraZoomScale(0)`'s range, `d_picture_box.cpp:454-507`: the scale is `1.0` at
/// rest and `9.0` at full zoom.
pub const ZOOM_MIN: f32 = 1.0;
pub const ZOOM_MAX: f32 = 9.0;
/// The scope's zoom gain on the sub-stick. Retail's is `mCamSetup.m030`
/// (`d_cam_param.cpp:316`), the gain the scoped styles' zoom walks at; the port's subject engine
/// does not run the zoom block yet, so the scale itself is walked here.
const ZOOM_RATE: f32 = 0.16;
/// The aim rates on the control stick. Retail feeds the scoped angle through `CalcSubjectAngle`
/// (`d_camera.cpp:4850`-era), whose gain is `mCamSetup.mCurveWeight`
/// (`d_cam_param.cpp:322`) against the style's own range; a free aim rate is this port's own.
const PITCH_RATE: f32 = 0.018;
const YAW_RATE: f32 = 0.022;
/// `cM_deg2s` of the aim limits: `mWork.subject`'s latitude range from the style
/// (`LOCKON_LATITUDE_MAX`, `d_cam_param.h`), which the follow styles fill with about 60 degrees.
const PITCH_LIMIT: f32 = 1.05;

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Scope {
    pub active: bool,
    /// The view's yaw and pitch in radians, carried across the tick the way
    /// `mViewCache.mDirection` carries the camera's own.
    pub yaw: f32,
    pub pitch: f32,
    /// `dComIfGp_getCameraZoomScale(0)`.
    pub zoom: f32,
    /// The use button's level last tick, so the item press is an edge: `mEquipItem`'s arm runs on
    /// the pad's trigger, not its hold (`d_a_player_main.cpp:4083`).
    pressed: bool,
    /// `daPyFlg0_SCOPE_CANCEL` (`d_a_player.h`), which `checkScopeEnd` reads and `procScope`
    /// clears every tick (`d_a_player_main.cpp:5653-5660`, `:5929`). Aryll raises it as the
    /// `zelda_fly` timeline runs out (`d_a_npc_ls1.cpp:1635-1636`) so the telescope comes down
    /// without the player touching the pad.
    cancel: bool,
}

impl Default for Scope {
    fn default() -> Self {
        Self {
            active: false,
            yaw: 0.0,
            pitch: 0.0,
            zoom: ZOOM_MIN,
            pressed: false,
            cancel: false,
        }
    }
}

impl Scope {
    /// `daPy_getPlayerLinkActorClass()->onNoResetFlg0(daPy_py_c::daPyFlg0_SCOPE_CANCEL)`
    /// (`d_a_npc_ls1.cpp:1636`).
    pub fn request_cancel(&mut self) {
        self.cancel = true;
    }

    /// The scoped view's eye.
    pub fn eye(&self, pos: Vec3) -> Vec3 {
        Vec3::new(pos.x, pos.y + EYE_HEIGHT, pos.z)
    }

    /// The scoped view's forward vector from the aim pair.
    pub fn forward(&self) -> Vec3 {
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();
        Vec3::new(sin_yaw * cos_pitch, sin_pitch, cos_yaw * cos_pitch)
    }
}

/// `daPy_lk_c::checkScopeEnd` (`d_a_player_main.cpp:5651-5663`): while the scope message status
/// is `SCOPE_ACTIVE`, the pad's cancel trigger or the no-reset `SCOPE_CANCEL` flag ends the
/// scope. A running event is *not* an exit: the telescope stays up through the `zelda_fly`
/// timeline, whose own end raises the flag (`d_a_npc_ls1.cpp:1635-1636`).
fn check_scope_end(pad: &PadButtons, cancel: bool) -> bool {
    pad.trig.b || pad.hold.b || cancel
}

/// Whether the item screen's telescope is the selected item. `dComIfGp_getSelectItem(dItemBtn_Z_e)`
/// is what the item button hands `mEquipItem`, and an unarmed button answers `0xFF`
/// (`d_com_inf_game.cpp`'s select table), so a collected telescope with nothing else armed is
/// what the scope can come up for.
fn telescope_ready(equipped: &Equipped) -> bool {
    equipped.item(TELESCOPE_SLOT) == TELESCOPE_ITEM && equipped.item(TELESCOPE_SLOT) != NO_ITEM
}

pub fn control(
    intent: Res<UseIntent>,
    pad: Option<Res<PadButtons>>,
    sub: Res<CStick>,
    mut main: ResMut<MoveIntent>,
    equipped: Res<Equipped>,
    dialogue: Res<Dialogue>,
    combat: Res<Combat>,
    swim: Res<Swim>,
    ships: Query<&Ship>,
    active: Option<Res<ActiveEvent>>,
    mut players: Query<&mut Player>,
    mut info: ResMut<MsgGameInfo>,
    mut scope: ResMut<Scope>,
) {
    let Some(mut player) = players.iter_mut().next() else {
        return;
    };
    let pad = pad.as_deref().copied().unwrap_or_default();
    let pressed = intent.0 && !scope.pressed;
    scope.pressed = intent.0;
    let running = active
        .as_ref()
        .is_some_and(|active| active.running_front().is_some());
    let riding = ships.iter().any(|ship| ship.riding);

    if scope.active {
        if check_scope_end(&pad, scope.cancel) {
            scope.active = false;
            // `d_scope.cpp`'s close leaves the scope message status at `MSG_UNK0`, which is the
            // state Aryll's `zelda_fly` end waits for before it releases the scope
            // (`d_a_npc_ls1.cpp:1618-1621`).
            if info.scope_mesg_status != STATUS_UNK0 {
                info.scope_mesg_status = STATUS_UNK0;
            }
            info.mesg_status = STATUS_UNK0;
            return;
        }
        scope.pitch = (scope.pitch + main.axis.y * PITCH_RATE).clamp(-PITCH_LIMIT, PITCH_LIMIT);
        scope.yaw += main.axis.x * YAW_RATE;
        if sub.value() > 0.2 {
            scope.zoom = (scope.zoom + sub.axis.y * ZOOM_RATE).clamp(ZOOM_MIN, ZOOM_MAX);
        }
        // `offNoResetFlg0(daPyFlg0_SCOPE_CANCEL)` at the end of `procScope`
        // (`d_a_player_main.cpp:5929`).
        scope.cancel = false;
        // `procScope`'s `shape_angle.y = fopCamM_GetAngleY(dComIfGp_getCamera(0))`
        // (`d_a_player_main.cpp:5925-5927`): the body turns with the view, so the telescope and
        // the face point where the eye does.
        player.yaw = scope.yaw;
        player.face = scope.yaw;
        // `procScope_init`'s `mNormalSpeed = 0.0f` (`d_a_player_main.cpp:5907`): the stick aims
        // the telescope, it does not walk Link. The other intents are cleared with it so the same
        // stick does not swing, grab or fire while the eye is up.
        main.axis = Vec2::ZERO;
        return;
    }

    let free = !dialogue.open
        && !combat.active
        && !swim.active
        && !riding
        && !running
        && telescope_ready(&equipped);
    if !pressed || !free {
        return;
    }
    scope.active = true;
    scope.cancel = false;
    scope.yaw = player.yaw;
    scope.pitch = 0.0;
    scope.zoom = ZOOM_MIN;
    // `procScope_init`'s `daPyStts0_TELESCOPE_LOOK_e` (`d_a_player_main.cpp:5915`) is published by
    // `player_status::control` from [`Scope::active`]; the message system's scope mode starts from
    // `SCOPE_ACTIVE`, which `getScopeMode` turns into `SCOPE_WAIT` on its first ask
    // (`d_msg.cpp`'s scope block; `text/msgproc.rs:400-413`).
    info.scope_mesg_status = STATUS_SCOPE_ACTIVE;
}
