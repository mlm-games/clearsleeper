use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::bombflower::{Blasts, BLAST_RADIUS};
use super::events::{SimEvent, SimEvents};
use super::flags::Switches;

/// `d_stage.cpp:916` `OBJNAME("Wall", fpcNm_WALL_e, 255, 0)`, and `c_dylink.cpp:423`
/// `{fpcNm_WALL_e, "d_a_wall"}`.
pub const WALL_ACTOR: &str = "Wall";

/// `mType = fopAcM_GetParam(this) >> 8;` (`_create`, `:140`) carries no mask; the value lands in the
/// `u8 daWall_c::mType` (`d_a_wall.h:48`), so the shift is the only truncation that happens.
const KIND_SHIFT: u32 = 8;

/// `mSwitchNo = fopAcM_GetParam(this) & 0xFF;` (`_create`, `:141`, repeated in `CreateInit`, `:133`).
const SWITCH_MASK: u32 = 0xFF;

/// `_create` (`:143`) refuses a wall whose `mSwitchNo == 0xff`, and `set_effect` (`:302`) latches
/// every other index.
const NO_SWITCH: u8 = 0xFF; // <- d_a_wall.cpp:143, :302

/// `fopAcM_seStart(this, JA_SE_OBJ_BOMB_WALL_BRK, 0)` (`set_se`, `:308`), and
/// `JA_SE_OBJ_BOMB_WALL_BRK = 0x696C` (`include/JAZelAudio/JAZelAudio_SE.h:1890`).
pub const SE_BOMB_WALL_BRK: u32 = 0x696C;

/// `mDst = 200.00f;` (`set_effect`, `:277`) - the value `mode_break` chases down to zero.
const FADE_FROM: f32 = 200.0; // <- d_a_wall.cpp:277

/// `cLib_chaseF(&mDst, 0.0f, 2.2222223f)` (`mode_break`, `:215`).
const FADE_STEP: f32 = 2.2222223; // <- d_a_wall.cpp:215

/// `mBreakCounter > 10` (`mode_break`, `:215`).
const FADE_FRAMES: u8 = 10; // <- d_a_wall.cpp:215

/// `m_tri_vtx[mType]` (`d_a_wall.cpp:25-38`) reduced to the two half-extents of the rectangle it
/// spells out. `set_tri` (`:240-259`) builds `mTri[0] = (v0, v1, v2)` and `mTri[1] = (v0, v2, v3)`
/// from the four corners `(±half_x, ±half_z, 0.0f)`, so the two triangles tile that rectangle
/// exactly, and it transforms them by `transS(current.pos)` then `YrotM(current.angle.y)`
/// (`:244-249`) - which puts the pair in the actor's own XZ plane at the actor's own height.
pub const HALF_EXTENT: [[f32; 2]; 3] = [
    [150.0, 120.0], // <- d_a_wall.cpp:26, :27
    [700.0, 500.0], // <- d_a_wall.cpp:30, :31
    [180.0, 90.0], // <- d_a_wall.cpp:34, :35
];

/// `daWall_c::m_cull_size` (`d_a_wall.cpp:40-44`), the profile's `fopAc_CULLBOX_CUSTOM_e` (`:366`)
/// box as `CreateInit` hands it to `fopAcM_setCullSizeBox(this, m_cull_size[mType][0],
/// m_cull_size[mType][1])` (`:124-126`).
///
/// Distinct from `HALF_EXTENT`: that is the rectangle the two bomb triangles tile, this is the
/// view volume, and neither is derived from the other. Row 0's view box is 180x150x50 while its
/// triangle rectangle is 150x120, so culling to the triangles would pop the wall out early.
pub const CULL_BOX: [[[f32; 3]; 2]; 3] = [
    [[-180.0, -150.0, -50.0], [180.0, 150.0, 50.0]], // <- d_a_wall.cpp:41
    [[-700.0, -600.0, -100.0], [700.0, 600.0, 100.0]], // <- d_a_wall.cpp:42
    [[-200.0, -100.0, -50.0], [200.0, 100.0, 50.0]], // <- d_a_wall.cpp:43
];

/// `mState` (`d_a_wall.h:42`), the `mode_proc[mState]` index of `_execute` (`:178-183`):
/// `0` is `mode_wait`, `1` is `mode_break`.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Wait,
    Break,
}

/// `daWall_c` (`d_a_wall.h:10-49`) - the bombable wall.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Wall {
    /// `mType` - `m_arcname[mType]` (`:47`, `:102`), `m_tri_vtx[mType]` (`:241`), and the
    /// `switch (mType)` selectors of `mode_break` (`:211`) and `set_effect` (`:279`).
    pub kind: u8,
    /// `mSwitchNo` - the `dComIfGs_isSwitch` index of `_create` (`:142`) and the
    /// `dComIfGs_onSwitch` index of `set_effect` (`:303`).
    pub switch_no: u8,
    /// `mState` - `_execute` (`:183`).
    pub state: State,
    /// `mBreakCounter` (`:209-215`, `:299`). Zero until `set_effect` writes one, and the
    /// `mode_break` entry guard.
    pub counter: u8,
    /// `mDst` (`:215`, `:277`) - the smoke emitter's global alpha.
    pub fade: f32,
    /// The inverse of `_create`'s `cPhs_ERROR_e` (`:142-145`). It starts `false` and `control`
    /// resolves it on the first tick: `Mthd_Create` needs the live `Switches` and `spawn` is not
    /// handed them. Until it resolves, retail's `_execute` cannot run at all, because the
    /// `cPhs_ERROR_e` return cancels the process during Create.
    pub appears: bool,
}

impl Wall {
    /// `_create` (`:138-157`) and `CreateInit` (`:122-135`), minus the archive load (`:147`), the
    /// solid heap (`:150`), the model, the moving background, the two triangles (`:129`) and the
    /// cull box (`:126`). What survives is the two parameter reads, `mState = false` (`:134`) and
    /// the create-time refusal.
    pub fn spawn(parameters: u32) -> Self {
        Self {
            kind: (parameters >> KIND_SHIFT) as u8,
            switch_no: (parameters & SWITCH_MASK) as u8,
            state: State::Wait,
            // `mDst` is written by `set_effect` before `mode_break` can read it, so its value here
            // is never observed.
            counter: 0,
            fade: 0.0,
            appears: false,
        }
    }

    /// `m_tri_vtx` (`:25-38`) and `m_cull_size` (`:40-44`) are three-entry tables indexed by
    /// `mType`, which `fopAcM_GetParam(this) >> 8` leaves unmasked. The `switch (mType)` arms of
    /// `mode_break` (`:211`) and `set_effect` (`:279`) both send every value past 2 to `default`;
    /// the reads below are clamped so an out-of-range parameter cannot index the array.
    fn row(&self) -> usize {
        usize::from(self.kind).min(HALF_EXTENT.len() - 1)
    }

    /// The `fopAc_CULLBOX_CUSTOM_e` volume for this wall's `mType`, from `m_cull_size` (`:40-44`).
    /// `src/lib.rs`'s `actor_views` branch reads this instead of a literal extent.
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        let [min, max] = CULL_BOX[usize::from(self.kind).min(CULL_BOX.len() - 1)];
        (min, max)
    }

    /// `mode_wait` (`:189-200`): `mTri[i].ChkTgHit()` for `i` in `0..2`, and on a non-NULL hit
    /// object whose `ChkAtType(AT_TYPE_BOMB)` holds (`:192`) the pair calls `set_effect()` then
    /// `set_se()` and breaks out of the loop.
    ///
    /// Collider registration, not just declaration: `l_tri_src` (`:54-82`) carries
    /// `SrcObjAt Type = 0` (`:58`) and `SrcObjTg Type = AT_TYPE_BOMB` (`:61`), and
    /// `dComIfG_Ccsp()->Set(&mTri[i])` runs for both triangles at the end of every `mode_wait`
    /// (`:202-204`), so the Tg arm is live while `mState == 0` and inert once `mState == 1`.
    /// Because the AT type is 0 the wall deals no outbound damage, and because the Tg type is
    /// `AT_TYPE_BOMB` a non-bomb hit object cannot reach this arm at all, which is why the
    /// `ClrTgHit()` else-branch (`:196-198`) is unreachable and is not ported.
    ///
    /// The two triangles tile `HALF_EXTENT[row]`, so a bomb blast overlaps the pair exactly when it
    /// comes within `BLAST_RADIUS` of that rectangle.
    fn struck(&self, pos: Vec3, yaw: i16, blasts: &[Vec3]) -> bool {
        let facing = super::walk::from_s16(yaw);
        let (sin, cos) = facing.sin_cos();
        let [half_x, half_z] = HALF_EXTENT[self.row()];
        blasts.iter().any(|origin| {
            let gap = *origin - pos;
            // `mDoMtx_YrotS` (`src/m_Do/m_Do_mtx.cpp:116-134`) maps a local `(x, z)` to world
            // `(x*cos + z*sin, -x*sin + z*cos)`; the arm above is its inverse.
            let local_x = gap.x * cos - gap.z * sin;
            let local_z = gap.x * sin + gap.z * cos;
            let over_x = (local_x.abs() - half_x).max(0.0);
            let over_z = (local_z.abs() - half_z).max(0.0);
            over_x * over_x + over_z * over_z <= BLAST_RADIUS * BLAST_RADIUS
        })
    }

    /// `set_effect` (`:263-304`), minus the two projections and the emitter (`:283-293`), the
    /// moving-background release (`:301`) and the `local_28` particle angle (`:275-276`).
    fn set_effect(&mut self, switches: &mut Switches) {
        self.fade = FADE_FROM; // mDst = 200.00f (:277)
        self.counter = 1; // mBreakCounter = 1 (:299)
        self.state = State::Break; // mState = 1 (:300)
        // if (mSwitchNo != 0xFF) dComIfGs_onSwitch(mSwitchNo, fopAcM_GetHomeRoomNo(this)); (:302-303)
        if self.switch_no != NO_SWITCH {
            switches.set(self.switch_no);
        }
    }

    /// `mode_break` (`:208-231`). `true` means one of the two `fopAcM_delete(this)` arms fired.
    /// The trailing `fopAcM_offDraw(this)` (`:230`) runs on every `mode_break` tick regardless,
    /// which is what `src/lib.rs` reads through `State::Break`.
    fn decay(&mut self) -> bool {
        if self.counter == 0 {
            return false;
        }
        self.counter = self.counter.wrapping_add(1); // mBreakCounter++ (:210), a u8 field
        match self.kind {
            // case 0: case 1: case 2: (:212-224)
            0..=2 => {
                if self.counter > FADE_FRAMES && super::walk::chase_f(&mut self.fade, 0.0, FADE_STEP)
                {
                    return true;
                }
                // JPABaseEmitter::setGlobalAlpha(mDst) (:219-222) - the particle system is one of
                // the confirmed-absent subsystems, so only the alpha bookkeeping above survives.
                false
            }
            // default: fopAcM_delete(this); (:225-227)
            _ => true,
        }
    }
}

pub fn control(
    blasts: Res<Blasts>,
    mut switches: ResMut<Switches>,
    mut events: ResMut<SimEvents>,
    mut walls: Query<(Entity, &Actor, &mut Wall)>,
    mut commands: Commands,
) {
    let fired: Vec<Vec3> = blasts.list.clone();
    for (entity, actor, mut wall) in &mut walls {
        let wall = &mut *wall;
        // `_create` (`:142-145`): `bool isSwitch = dComIfGs_isSwitch(mSwitchNo, GetHomeRoomNo(this));
        // if (isSwitch || mSwitchNo == 0xff) return cPhs_ERROR_e;`. `Switches::test` models the
        // in-memory 0..128 array, so an index of 0x80 or above reads false here (RULE 11b) - the real
        // index is passed through unchanged. RULE 6b: the returned `cPhs_ERROR_e` cancels the
        // process before `Create` runs, so the wall never exists - spawn it, despawn on tick one.
        if !wall.appears {
            wall.appears = wall.switch_no != NO_SWITCH && !switches.test(wall.switch_no);
            if !wall.appears {
                let _ = commands.entity(entity).despawn();
                continue;
            }
        }
        match wall.state {
            // `mode_proc[mState]` (`:183`) -> `mode_wait` (`:188-205`).
            State::Wait => {
                if wall.struck(actor.pos, actor.angle[1], &fired) {
                    wall.set_effect(&mut switches);
                    events.list.push(SimEvent::Se(SE_BOMB_WALL_BRK)); // set_se() (:307-309)
                }
            }
            // `mode_proc[mState]` (`:183`) -> `mode_break` (`:208-231`).
            State::Break => {
                if wall.decay() {
                    let _ = commands.entity(entity).despawn();
                }
            }
        }
    }
}