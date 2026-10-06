use bevy_ecs::prelude::*;

use super::flags::{EventFlags, Switches};

pub const EVSW_ACTORS: [&str; 1] = ["Evsw"];

const SWSAVE_SHIFT: u32 = 0; // <- d_a_tag_evsw.h:14 (PRM_SWSAVE_S)
const SWSAVE_MASK: u32 = 0xFF; // <- d_a_tag_evsw.h:13 (PRM_SWSAVE_W = 8, d_a_obj.h:14)
const EVENTBIT_SHIFT: u32 = 8; // <- d_a_tag_evsw.h:17 (PRM_EVENTBITID_S)
const EVENTBIT_MASK: u32 = 0xFFFF; // <- d_a_tag_evsw.h:16 (PRM_EVENTBITID_W = 16)
const TYPE_SHIFT: u32 = 24; // <- d_a_tag_evsw.h:20 (PRM_TYPE_S)
const TYPE_MASK: u32 = 0x3; // <- d_a_tag_evsw.h:19 (PRM_TYPE_W = 2)
const TYPE_HOLD: u8 = 1; // <- d_a_tag_evsw.cpp:18

/// `daTagEvsw::Act_c` (`d_a_tag_evsw.cpp`). `_create` (`:11-27`) and `_execute` (`:35-48`) are
/// the same body; `_draw` (`:51-53`) returns TRUE with no draw call, so retail renders nothing
/// here, and `Mthd_IsDelete` (`:78-81`) is a bare TRUE - it accepts a delete request, it is not
/// a self-destruct. `_create` never refuses (`:26` is unconditionally `cPhs_COMPLEATE_e`), so
/// there is no spawn-then-despawn path.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Evsw {
    pub sw_save: u8,
    pub event_bit: u16,
    pub kind: u8,
}

impl Evsw {
    pub fn spawn(parameters: u32, flags: &EventFlags, switches: &mut Switches) -> Self {
        let tag = Self {
            sw_save: ((parameters >> SWSAVE_SHIFT) & SWSAVE_MASK) as u8,
            event_bit: ((parameters >> EVENTBIT_SHIFT) & EVENTBIT_MASK) as u16,
            kind: ((parameters >> TYPE_SHIFT) & TYPE_MASK) as u8,
        };
        tag.sync(flags, switches);
        tag
    }

    /// `_create` (`:14-24`) and `_execute` (`:36-46`) share this body verbatim.
    fn sync(&self, flags: &EventFlags, switches: &mut Switches) {
        if flags.test(u32::from(self.event_bit)) {
            switches.set(self.sw_save);
        } else if self.kind != TYPE_HOLD {
            switches.clear(self.sw_save);
        }
    }
}

pub fn is_evsw(name: &str) -> bool {
    EVSW_ACTORS.contains(&name)
}

pub fn control(flags: Res<EventFlags>, mut switches: ResMut<Switches>, tags: Query<&Evsw>) {
    for tag in &tags {
        tag.sync(&flags, &mut switches);
    }
}