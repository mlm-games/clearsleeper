use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::bombflower::Blasts;
use super::events::{SimEvent, SimEvents};

pub const PLANT_ACTORS: [&str; 1] = ["Plant"];

const CYL_RADIUS: f32 = 30.0; // <- d_a_obj_plant.cpp:37
const CYL_HEIGHT: f32 = 200.0; // <- d_a_obj_plant.cpp:38
const BOMB_RADIUS: f32 = 30.0; // <- d_a_bomb2.cpp:326 (Act_c::M_sph_src)
const REHIT_TIMER: i16 = 0x38; // <- d_a_obj_plant.cpp:85
const SWING_END: i16 = 0x100; // <- d_a_obj_plant.cpp:111
const SE_SWING: u32 = 0x69E6; // <- JAZelAudio_SE.h:1998 (JA_SE_OBJ_TREE_SWING_S)

pub const CULL_MIN: [f32; 3] = [-600.0, 0.0, -600.0]; // <- d_a_obj_plant.cpp:154
pub const CULL_MAX: [f32; 3] = [600.0, 900.0, 600.0]; // <- d_a_obj_plant.cpp:154

/// The stage string is `Plant` (`d_stage.cpp:988`) and the archive `Create` loads is also
/// `Plant` (`d_a_obj_plant.cpp:48`).
///
/// The tree has no AT collider of its own (`l_cyl_src` sets `SrcObjAt Type` to 0,
/// `:17`) - only a Tg of type `AT_TYPE_BOMB` (`:19`), which a thrown bomb's `AT_TYPE_BOMB`
/// sphere (radius 30, `d_a_bomb2.cpp:322`) crosses. `_execute` never applies damage: it only
/// starts a swing, so no `HitPoints` and no `Hurt` are attached - attaching them would let
/// `combat::control` delete a tree retail cannot cut.
///
/// `field_0x40E` (`:92`), `field_0x408` (`:104`, `:107`, `:114`) and `field_0x40A` (`:95`,
/// `:105`, `:108`) exist only to be read back by `nodeCallBack` (`:166-182`), which rebuilds
/// the `joint2` animation matrix. They are omitted rather than stored.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Plant {
    pub hit_timer: i16,
    pub swinging: bool,
}

impl Plant {
    /// `fopAcM_ct(this, daObjPlant_c)` (`:46`) expands to `new (ptr) daObjPlant_c();`, which
    /// value-initialises the term, so `mHitTimer` (`d_a_obj_plant.h:24`) and `field_0x410`
    /// (`:26`) both start at zero.
    pub fn spawn() -> Self {
        Self {
            hit_timer: 0,
            swinging: false,
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `cM3d_Cross_CylSph` (`c_m3d.cpp:1377-1420`) is the test `ProcAtTgHit` runs for the
    /// bomb sphere against the plant's `cM3dGCylS`: the horizontal span is `cyl.r + sph.r`,
    /// then the sphere must straddle the cylinder's `y .. y + h` interval.
    fn bomb_hits(centre: Vec3, origin: Vec3) -> bool {
        let span = CYL_RADIUS + BOMB_RADIUS;
        let dx = origin.x - centre.x;
        let dz = origin.z - centre.z;
        dx * dx + dz * dz <= span * span
            && origin.y + BOMB_RADIUS >= centre.y
            && origin.y - BOMB_RADIUS <= centre.y + CYL_HEIGHT
    }

    /// `daObjPlant_c::_execute` (`:81-121`) minus `mCyl.SetC` / `dComIfG_Ccsp()->Set(&mCyl)`
    /// (`:82-83`), the three swing-curve fields listed above and `set_mtx` (`:118`). Returns
    /// true where retail calls `fopAcM_seStart`.
    fn execute(&mut self, centre: Vec3, blasts: &[Vec3]) -> bool {
        let swung = (self.hit_timer > REHIT_TIMER || !self.swinging)
            && blasts.iter().any(|origin| Self::bomb_hits(centre, *origin));
        if swung {
            self.swinging = true;
            self.hit_timer = 0;
        }
        if self.swinging {
            self.hit_timer += 1;
            if self.hit_timer > SWING_END {
                self.swinging = false;
                self.hit_timer = 0;
            }
        }
        swung
    }
}

pub fn is_plant(name: &str) -> bool {
    PLANT_ACTORS.contains(&name)
}

pub fn control(
    published: Res<Blasts>,
    mut events: ResMut<SimEvents>,
    mut plants: Query<(&Actor, &mut Plant)>,
) {
    let blasts: Vec<Vec3> = published.list.clone();
    for (actor, mut plant) in &mut plants {
        if plant.execute(actor.pos, &blasts) {
            events.list.push(SimEvent::Se(SE_SWING));
        }
    }
}