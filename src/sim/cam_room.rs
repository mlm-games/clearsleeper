use bevy_ecs::prelude::{Resource, World};
use glam::Vec3;

use super::cam_follow::{SAngle, ViewCache};
use super::cam_type::type_index;
use crate::dzs::{NO_ROOM, StageArrowEntry, StageCameraEntry, StageCameras, StageTables};

/// `dCamera_c::GetCameraTypeFromMapToolID`'s no-type answer, `d_camera.cpp:1489`, `:1494` and
/// `:1507`.
pub const NO_CAM_TYPE: u8 = 0xFF;
/// `dCamera_c::mCurArrowIdx` with no arrow, `d_camera.cpp:1517`.
pub const NO_ARROW: u8 = 0xFF;
/// The value `field_0x11`, `field_0x12`, `field_0x13` and the arrow rows' `field_0x12` carry when
/// the name leaves the field unset, read as `0xFF` at `d_camera.cpp:4239`, `:4469-4471` and
/// `:4559-4569`.
pub const CAM_FIELD_UNSET: u8 = 0xFF;

/// `dCamera_c::GetCameraTypeFromCameraName`, `d_camera.cpp:1522-1540`, and the search half of
/// `GetCameraTypeFromMapToolID` at `d_camera.cpp:1497-1508`: the first row of the camera type
/// table whose name matches, [`NO_CAM_TYPE`] for no row. Retail compares against the current row
/// first, `d_camera.cpp:1524-1526`, which names the row the loop finds anyway, since no two rows
/// share a name.
pub fn camera_type_from_name(name: &str) -> u8 {
    match type_index(name) {
        Some(found) => u8::try_from(found).unwrap_or(NO_CAM_TYPE),
        None => NO_CAM_TYPE,
    }
}

/// `dCamera_c::pushPos`, `d_camera.cpp:1545-1552`: the placement `m084`, `m090`, `m09C` and `m0A0`
/// keep while an event camera runs. Nothing else in the readable text reads them back, so this is
/// the whole of what the original stores.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PushedPos {
    pub center: Vec3,
    pub eye: Vec3,
    pub fovy: f32,
    pub bank: SAngle,
}

impl PushedPos {
    pub fn from_view_cache(cache: &ViewCache) -> Self {
        Self {
            center: cache.center,
            eye: cache.eye,
            fovy: cache.fovy,
            bank: cache.bank,
        }
    }
}

impl Default for PushedPos {
    fn default() -> Self {
        Self::from_view_cache(&ViewCache::default())
    }
}

/// The whole-stage camera pair `dComIfGp_getStage().getCamera()` and `getArrow()` answer with, the
/// pair every loaded room installed on its own `dStage_roomDt_c`, and the row `dCamera_c` last
/// resolved: `mCurRoomCamEntry`, `mCurRoomArrowEntry` and `mCurArrowIdx`, `d_camera.h:419-421`.
#[derive(Resource, Debug, Clone)]
pub struct CamRoom {
    stage: StageCameras,
    rooms: Vec<(u8, StageCameras)>,
    cam_entry: Option<StageCameraEntry>,
    arrow_entry: Option<StageArrowEntry>,
    arrow_idx: u8,
    pushed: PushedPos,
}

impl Default for CamRoom {
    fn default() -> Self {
        Self {
            stage: StageCameras::default(),
            rooms: Vec::new(),
            cam_entry: None,
            arrow_entry: None,
            arrow_idx: NO_ARROW,
            pushed: PushedPos::default(),
        }
    }
}

impl CamRoom {
    /// The whole-stage pair, which is what a `room_no` of `-1` resolves against. A stage brings
    /// its rooms with it, so the loaded rooms go too.
    pub fn set_stage(&mut self, stage: &StageTables) {
        self.stage = stage.cameras().clone();
        self.rooms.clear();
        self.reset_entry();
    }

    /// `dStage_dt_c_roomLoader` hands every room that loads its own pair, `d_stage.cpp:2180-2198`,
    /// and `dComIfGp_getRoomCamera` answers by room number, `d_com_inf_game.cpp:644-653`, so a
    /// room keeps its pair after it leaves the loaded set. A room file without `RCAM`/`RARO`
    /// installs nothing and reads as no camera at all.
    pub fn set_rooms(&mut self, rooms: impl IntoIterator<Item = (u8, StageCameras)>) {
        for (room_no, tables) in rooms {
            match self.rooms.iter_mut().find(|(no, _)| *no == room_no) {
                Some(entry) => entry.1 = tables,
                None => self.rooms.push((room_no, tables)),
            }
        }
        self.reset_entry();
    }

    /// The rooms whose pairs are known, by room number.
    pub fn rooms(&self) -> &[(u8, StageCameras)] {
        &self.rooms
    }

    /// `dCamera_c::mCurRoomCamEntry`, the row behind the type the last lookup answered.
    pub fn cam_entry(&self) -> Option<&StageCameraEntry> {
        self.cam_entry.as_ref()
    }

    /// `dCamera_c::mCurArrowIdx`, [`NO_ARROW`] when the last lookup took no arrow.
    pub fn arrow_idx(&self) -> u8 {
        self.arrow_idx
    }

    /// `dCamera_c::mCurRoomArrowEntry` for a camera that has one. Retail leaves the row in place
    /// when the arrow index fails its test at `d_camera.cpp:1516-1518`, but every reader gates
    /// on `mCurArrowIdx` first, `d_camera.cpp:4239`, `:4443` and `:4549`, so an absent arrow
    /// reads as none here.
    pub fn arrow_entry(&self) -> Option<&StageArrowEntry> {
        if self.arrow_idx == NO_ARROW {
            return None;
        }
        self.arrow_entry.as_ref()
    }

    /// `dCamera_c::GetCameraTypeFromMapToolID`, `d_camera.cpp:1473-1520`. A `room_no` of `-1`
    /// resolves against the whole-stage pair, any other room against that room's own pair, whose
    /// absence answers [`NO_CAM_TYPE`] the way the original's null camera does. The answer is
    /// `cam_type::TYPE_KEEP` for a row that keeps the camera where it is, which
    /// `d_camera.cpp:1350-1359` turns into the unchanged type, and every other row into the next
    /// one.
    pub fn type_from_map_tool_id(&mut self, idx: i32, room_no: i32) -> u8 {
        let tables = if room_no == i32::from(NO_ROOM) {
            &self.stage
        } else {
            let Some(wanted) = u8::try_from(room_no).ok() else {
                return NO_CAM_TYPE;
            };
            match self.rooms.iter().find(|(room, _)| *room == wanted) {
                Some((_, tables)) => tables,
                None => return NO_CAM_TYPE,
            }
        };
        let Some(entry) = usize::try_from(idx)
            .ok()
            .and_then(|index| tables.cameras.get(index))
        else {
            return NO_CAM_TYPE;
        };
        let found = camera_type_from_name(&entry.cam_type);
        if found == NO_CAM_TYPE {
            return NO_CAM_TYPE;
        }
        self.cam_entry = Some(entry.clone());
        let arrow_idx = entry.arrow_idx;
        if arrow_idx != NO_ARROW && usize::from(arrow_idx) < tables.arrows.len() {
            self.arrow_idx = arrow_idx;
            self.arrow_entry = tables.arrows.get(usize::from(arrow_idx)).copied();
        } else {
            self.arrow_idx = NO_ARROW;
            self.arrow_entry = None;
        }
        found
    }

    /// `dCamera_c::pushPos`.
    pub fn push_pos(&mut self, view: &ViewCache) {
        self.pushed = PushedPos::from_view_cache(view);
    }

    /// `dCamera_c::m084`, `m090`, `m09C` and `m0A0`.
    pub fn pushed(&self) -> PushedPos {
        self.pushed
    }

    fn reset_entry(&mut self) {
        self.cam_entry = None;
        self.arrow_entry = None;
        self.arrow_idx = NO_ARROW;
    }
}

pub fn init(world: &mut World) {
    world.init_resource::<CamRoom>();
}
