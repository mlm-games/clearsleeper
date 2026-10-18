use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::arrow::{Arrow, TYPE_FIRE};
use super::event_manager::EventManager;
use super::events::{SimEvent, SimEvents};
use super::flags::Switches;
use super::ground;
use super::stream::{cos_s, sin_s};
use super::walk::within;
use crate::sim::collision::Collision;

/// `d_a_ep.cpp` is `g_profile_EP`, process `fpcNm_EP_e`; the OBJNAME row is `"bonbori"`, NOT the
/// arcname `"Ep"` (`dComIfG_resLoad(&i_this->mPhase, "Ep")`, `d_a_ep.cpp:640`). `src/sim/torch.rs`
/// claims the arcname `"Ep"`, which appears in no OBJNAME row - see the report.
pub const EP_ACTORS: [&str; 1] = ["bonbori"]; // <- d_stage.cpp:481

/// `i_this->mType = fopAcM_GetParam(a_this) & 0x3F;` (`fopAcM_GetParam` returns the whole u32).
pub const TYPE_MASK: u32 = 0x3F; // <- d_a_ep.cpp:643
/// `if (i_this->mType == 0x3F) { i_this->mType = 0; }`
pub const TYPE_ALIAS: u32 = 0x3F; // <- d_a_ep.cpp:644
/// `if (i_this->mType == 0x3F) { i_this->mType = 0; }` - the alias folds back to kind zero.
pub const TYPE_TORCH: u8 = 0; // <- d_a_ep.cpp:645
/// `(mType == 0) || (mType == 3)` gates the model, the post collider and `ga_move`.
pub const TYPE_MODEL: u8 = 3; // <- d_a_ep.cpp:658, :384
/// `(mType == 0) || (mType == 3)` again, in the mode-10 re-arm branch.
pub const TYPE_REBURN: u8 = 3; // <- d_a_ep.cpp:313

/// `m4D0 = 0` in `daEp_CreateInit`, the unlit park.
pub const MODE_IDLE: i32 = 0; // <- d_a_ep.cpp:556
/// `i_this->m4D0 = 3;` when the staff cut reaches "FIRE".
pub const MODE_CATCH: i32 = 3; // <- d_a_ep.cpp:249
/// `case 3:` falls into `case 4:` after `m4D0++`.
pub const MODE_BURN: i32 = 4; // <- d_a_ep.cpp:261
/// `i_this->m4D0 = 10;` when the flame drops below its own anchor, and when it burns out.
pub const MODE_DYING: i32 = 10; // <- d_a_ep.cpp:484

/// `fopAcM_orderOtherEvent(&i_this->actor, "SHOKUDAI_SWITCH")`.
pub const EVENT_SWITCH: &str = "SHOKUDAI_SWITCH"; // <- d_a_ep.cpp:92
/// `dComIfGp_evmng_endCheck("SHOKUDAI_SWITCH")` in `ep_switch_event_end`.
pub const EVENT_SWITCH_END: &str = "SHOKUDAI_SWITCH"; // <- d_a_ep.cpp:81
/// `dComIfGp_evmng_existence("SHOKUDAI_SWITCH")` in `ep_move` case 0.
pub const EVENT_SWITCH_EXISTS: &str = "SHOKUDAI_SWITCH"; // <- d_a_ep.cpp:232
/// `ret = dComIfGp_evmng_getMyStaffId("SHOKUDAI");` in `ep_switch_event_begin`.
pub const STAFF_SWITCH: &str = "SHOKUDAI"; // <- d_a_ep.cpp:95
/// `static char* actions[] = { "WAIT", "FIRE" };`
pub const ACT_WAIT: &str = "WAIT"; // <- d_a_ep.cpp:107
/// `static char* actions[] = { "WAIT", "FIRE" };`
pub const ACT_FIRE: &str = "FIRE"; // <- d_a_ep.cpp:107
/// `getMyActIdx(m7E0, actions, ARRAY_SIZE(actions), FALSE, 0)` - cached, not forced.
pub const ACT_COUNT: usize = 2; // <- d_a_ep.cpp:109
/// `getMyStaffId` returns `-1` when this torch owns no staff.
pub const NO_STAFF: i32 = -1; // <- d_a_ep.cpp:96
/// `i_this->m7E0 = -1;` at the tail of `daEp_Create`.
pub const NO_CUT: i32 = -1; // <- d_a_ep.cpp:695
/// `ret` is `FALSE` when the per-actor demo command branch is taken instead of a staff cut.
pub const BEGIN_NONE: i32 = 0; // <- d_a_ep.cpp:93, :235
/// `getEventIdx(name)` uses `0xFF` for the by-name lookup, not the event-info table.
pub const NO_EVENT_INFO: u8 = 0xFF; // <- d_event_manager.cpp, get_event_idx

pub const SE_TORCH_IGNITION: u32 = 0x6902; // <- d_a_ep.cpp:244
pub const SE_TORCH_OFF: u32 = 0x6904; // <- d_a_ep.cpp:295
pub const SE_TORCH_BURNING: u32 = 0x6103; // <- d_a_ep.cpp:297

/// `i_this->m505 = fopAcM_GetParam(a_this) >> 8;` stored into a `u8`.
pub const HOLD_SHIFT: u32 = 8; // <- d_a_ep.cpp:650
/// `i_this->m507 = fopAcM_GetParam(a_this) >> 0x10;`
pub const GUARD_SHIFT: u32 = 16; // <- d_a_ep.cpp:651
/// `i_this->mOnSwitchNo = fopAcM_GetParam(a_this) >> 0x18;`
pub const SWITCH_SHIFT: u32 = 24; // <- d_a_ep.cpp:652
/// The three parameter bytes are all narrowed into `u8`.
pub const BYTE_MASK: u32 = 0xFF; // <- d_a_ep.cpp:650-652

/// `if (i_this->m505 != 0xff) { i_this->mTimers[3] = i_this->m505 * 10 + 1; }`
pub const HOLD_SCALE: i16 = 10; // <- d_a_ep.cpp:224
/// `i_this->mTimers[3] = i_this->m505 * 10 + 1;`
pub const HOLD_BIAS: i16 = 1; // <- d_a_ep.cpp:224
/// `i_this->mTimers[3] = 20000;` in `daEp_Create`.
pub const HOLD_INIT: i16 = 20000; // <- d_a_ep.cpp:690
/// `if ((i_this->mTimers[3] != 0) && (i_this->mTimers[3] < 10000)) { i_this->mTimers[3]--; }`
pub const HOLD_CAP: i16 = 10000; // <- d_a_ep.cpp:431
/// `if (i_this->mTimers[3] == 1 && ...)`
pub const HOLD_DUE: i16 = 1; // <- d_a_ep.cpp:286

/// `m508` is `0` while disarmed and `mOnSwitchNo + 1` while armed; `0xFF + 1` wraps to `0`.
pub const ARMED_NONE: u8 = 0; // <- d_a_ep.cpp:245
/// `dComIfGs_onSwitch(i_this->m508 - 1, ...)` and `i_this->m508 = i_this->mOnSwitchNo + 1`.
pub const ARMED_BIAS: u8 = 1; // <- d_a_ep.cpp:226, :293
/// `if (i_this->mOnSwitchNo != 0xFF)` - the "this torch has no switch" sentinel.
pub const NO_SWITCH: u8 = 0xFF; // <- d_a_ep.cpp:286, :686

/// `cLib_addCalc0(&i_this->mLightPower, 0.5f, 0.05f)` in mode 0.
pub const IDLE_SCALE: f32 = 0.5; // <- d_a_ep.cpp:217
pub const IDLE_MAX: f32 = 0.05; // <- d_a_ep.cpp:217
/// `cLib_addCalc2(&i_this->mLightPower, i_this->m4E0, 0.5f, 0.2f)` in modes 3 and 4.
pub const BURN_SCALE: f32 = 0.5; // <- d_a_ep.cpp:262
pub const BURN_MAX: f32 = 0.2; // <- d_a_ep.cpp:262
/// `cLib_addCalc0(&i_this->mLightPower, 1.0f, 0.10000000149011612f)` in mode 10.
pub const DIE_SCALE: f32 = 1.0; // <- d_a_ep.cpp:311
pub const DIE_MAX: f32 = 0.10000000149011612; // <- d_a_ep.cpp:311
/// `if (i_this->mLightPower < 0.05f)`
pub const DIE_LIMIT: f32 = 0.05; // <- d_a_ep.cpp:312
/// `i_this->m4E0 = i_this->actor.scale.x;` - stage-placed scale is exactly 1.0 (RULE 2).
pub const FULL_POWER: f32 = 1.0; // <- d_a_ep.cpp:250

/// `sp3C.set(i_this->mPosTop.x, i_this->mPosTop.y + 30.0f, i_this->mPosTop.z);`
pub const FLAME_LIFT: f32 = 30.0; // <- d_a_ep.cpp:341
/// `sph_src`: `Radius 40.0f` - the sphere the fire arrows are tested against.
pub const FLAME_RADIUS: f32 = 40.0; // <- d_a_ep.cpp:604
/// `sph_src` `SrcObjTg Type` is `AT_TYPE_FIRE | AT_TYPE_UNK20000 | AT_TYPE_FIRE_ARROW |
/// AT_TYPE_WIND | AT_TYPE_UNK400000`, so the fire arrow is the only hitter this crate carries.
pub const FIRE_ARROW_AT: u32 = super::barricade::AT_FIRE_ARROW; // <- d_a_ep.cpp:586

/// `sp08.y = REG0_F(0) + 140.0f;` in `daEp_set_mtx`; `REG0_F` folds to zero.
pub const TOP_LIFT: f32 = 140.0; // <- d_a_ep.cpp:394
/// `cXyz sp08 = i_this->mPosTop; sp08.y += 200.0f;` before `GroundCross`.
pub const GROUND_PROBE: f32 = 200.0; // <- d_a_ep.cpp:478
/// `if ((i_this->mGroundCheckTimer & 0xF) == 0)`
pub const GROUND_MASK: u32 = 0xF; // <- d_a_ep.cpp:474
/// `if (i_this->m4D0 > 0) { ... if (i_this->m4D0 < 10) { ... } }`
pub const MODE_BURN_FLOOR: i32 = 0; // <- d_a_ep.cpp:471
pub const MODE_BURN_CEIL: i32 = 10; // <- d_a_ep.cpp:472

/// `fopAcM_SetMin(a_this, -160.0f, -160.0f, -160.0f)` with the matching `SetMax` at `:561` is what
/// gives `fopAc_CULLBOX_CUSTOM_e` a box; `g_profile_EP` selects CUSTOM at `d_a_ep.cpp:725`.
pub const CULL_MIN: f32 = -160.0; // <- d_a_ep.cpp:560
pub const CULL_MAX: f32 = 160.0; // <- d_a_ep.cpp:561

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Ep {
    /// `mType`, the six-bit kind decoded out of the parameter word.
    pub kind: u8,
    /// `m505`, the parameter byte that sets the flame hold timer.
    pub hold: u8,
    /// `m507`, the parameter byte guarding the extinguish.
    pub guard: u8,
    /// `mOnSwitchNo`, the switch this torch obeys.
    pub on_switch: u8,
    /// `m508`, zero while disarmed and `mOnSwitchNo + 1` while armed.
    pub armed: u8,
    /// `m4D0`, the lit state.
    pub mode: i32,
    /// `mLightPower`, eased toward zero or toward `full`.
    pub power: f32,
    /// `m4E0`, the power the current ignition eases toward.
    pub full: f32,
    /// `mTimers[3]`, the hold countdown.
    pub wait: i16,
    /// `m7E0`, the `"SHOKUDAI"` staff cut index, or `-1`.
    pub cut: i32,
    /// `mGroundCheckTimer`, the `& 0xF` gate on the ground probe.
    pub probe: u8,
    /// `mPosTop`, the flame anchor recomputed from the actor transform.
    pub top: Vec3,
    /// Set once `ep_move` reaches `fopAcM_delete(&i_this->actor)` in mode 10.
    pub gone: bool,
}

impl Ep {
    pub fn spawn(parameters: u32, switches: &Switches) -> Self {
        let raw = parameters & TYPE_MASK;
        let on_switch = ((parameters >> SWITCH_SHIFT) & BYTE_MASK) as u8;
        let mut torch = Self {
            kind: if raw == TYPE_ALIAS {
                TYPE_TORCH
            } else {
                raw as u8
            },
            hold: ((parameters >> HOLD_SHIFT) & BYTE_MASK) as u8,
            guard: ((parameters >> GUARD_SHIFT) & BYTE_MASK) as u8,
            on_switch,
            armed: ARMED_NONE,
            mode: MODE_IDLE,
            power: 0.0,
            full: 0.0,
            wait: HOLD_INIT,
            cut: NO_CUT,
            probe: 0,
            top: Vec3::ZERO,
            gone: false,
        };
        if on_switch != NO_SWITCH && !switches.test(on_switch) {
            torch.armed = on_switch + ARMED_BIAS;
        }
        torch
    }

    /// `daEp_Draw` renders `mpModel` only for kinds 0 and 3.
    pub fn drawn(&self) -> bool {
        self.kind == TYPE_TORCH || self.kind == TYPE_MODEL
    }

    /// The CUSTOM box as `(base, height, half)`, matching `canon::hull`.
    pub fn hull(&self) -> (f32, f32, f32) {
        (CULL_MIN, CULL_MAX - CULL_MIN, CULL_MAX)
    }

    /// `daEp_set_mtx`: with a model, `mPosTop` is `140.0f` up the actor's own rotation; without one
    /// it is the actor origin.
    fn top_of(&self, pos: Vec3, angle: [i16; 3]) -> Vec3 {
        if !self.drawn() {
            return pos;
        }
        let lift = sin_s(angle[0]) * TOP_LIFT;
        Vec3::new(
            pos.x + lift * sin_s(angle[1]),
            pos.y + cos_s(angle[0]) * TOP_LIFT,
            pos.z + lift * cos_s(angle[1]),
        )
    }

    /// `mSph1.ChkTgHit()` followed by `!hitObj->ChkAtType(0x600000)`. `0x600000` is
    /// `AT_TYPE_WIND | AT_TYPE_UNK400000`, and no collider in this crate carries either bit, so any
    /// overlap qualifies. A fire arrow is the only registered hitter the crate has.
    fn flame_hit(&self, arrows: &[Vec3]) -> bool {
        let centre = self.top + Vec3::Y * FLAME_LIFT;
        arrows
            .iter()
            .any(|shot| within(*shot, centre, FLAME_RADIUS))
    }

    /// `ep_switch_event_end`: `dComIfGp_evmng_endCheck("SHOKUDAI_SWITCH")`.
    fn switch_end(&self, manager: &EventManager) -> bool {
        manager.end_check_old(EVENT_SWITCH_END)
    }

    /// `ep_switch_event_begin`: order the switch event, then take the `"SHOKUDAI"` staff cut.
    fn switch_begin(&mut self, manager: &mut EventManager) -> i32 {
        let index = manager.get_event_idx(Some(EVENT_SWITCH), NO_EVENT_INFO);
        let _ = manager.order(index);
        let staff = manager.get_my_staff_id(STAFF_SWITCH, None, 0);
        if staff == NO_STAFF {
            self.switch_end(manager);
        }
        staff
    }

    /// `ep_switch_event_move`: `ret` is `TRUE` only for the `"FIRE"` action.
    fn switch_move(&self, manager: &mut EventManager, events: &mut SimEvents) -> bool {
        if !manager.get_is_advance(self.cut) {
            return false;
        }
        let action = manager.get_my_act_idx(self.cut, &[ACT_WAIT, ACT_FIRE][..ACT_COUNT], false, 0);
        if action == 1 {
            events.list.push(SimEvent::Se(SE_TORCH_IGNITION));
            true
        } else {
            manager.cut_end(self.cut);
            false
        }
    }

    /// `case 0:` - armed torches fade, unlit torches wait on the staff cut.
    fn idle(
        &mut self,
        manager: &mut EventManager,
        switches: &mut Switches,
        arrows: &[Vec3],
        events: &mut SimEvents,
    ) {
        if self.armed != ARMED_NONE {
            add_calc0(&mut self.power, IDLE_SCALE, IDLE_MAX);
            if self.flame_hit(arrows) {
                if self.hold != NO_SWITCH {
                    self.wait = i16::from(self.hold) * HOLD_SCALE + HOLD_BIAS;
                }
                switches.set(self.armed - ARMED_BIAS);
            }
            if switches.test(self.armed - ARMED_BIAS) {
                if self.kind != TYPE_MODEL {
                    if manager.existence_name(EVENT_SWITCH_EXISTS) {
                        let cut = self.switch_begin(manager);
                        self.cut = cut;
                        if cut != BEGIN_NONE {
                            self.armed = ARMED_NONE;
                            if cut != NO_CUT {
                                self.switch_move(manager, events);
                            }
                        }
                        return;
                    }
                }
                events.list.push(SimEvent::Se(SE_TORCH_IGNITION));
                self.armed = ARMED_NONE;
            }
        } else if self.kind != TYPE_MODEL && self.cut != NO_CUT {
            if self.switch_move(manager, events) {
                self.mode = MODE_CATCH;
                self.full = FULL_POWER;
            }
        } else {
            self.mode = MODE_CATCH;
            self.full = FULL_POWER;
        }
    }

    /// `case 3:` falls through to `case 4:` and then to `default:` in one tick.
    fn burn(
        &mut self,
        manager: &mut EventManager,
        switches: &mut Switches,
        events: &mut SimEvents,
    ) {
        if self.mode == MODE_CATCH {
            self.mode = MODE_BURN;
        }
        add_calc2(&mut self.power, self.full, BURN_SCALE, BURN_MAX);
        if self.wait == HOLD_DUE && (self.guard == NO_SWITCH || !switches.test(self.guard)) {
            switches.clear(self.on_switch);
        }
        if self.on_switch != NO_SWITCH && !switches.test(self.on_switch) {
            self.mode = MODE_IDLE;
            self.armed = self.on_switch + ARMED_BIAS;
            events.list.push(SimEvent::Se(SE_TORCH_OFF));
        } else {
            events.list.push(SimEvent::Se(SE_TORCH_BURNING));
        }
        if self.kind != TYPE_MODEL && self.cut != NO_CUT {
            if self.switch_end(manager) {
                self.cut = NO_CUT;
            } else {
                manager.cut_end(self.cut);
            }
        }
    }

    /// `case 10:` - the flame collapses; anything that is not kind 0 or 3 is deleted outright.
    fn dying(&mut self, switches: &mut Switches) {
        add_calc0(&mut self.power, DIE_SCALE, DIE_MAX);
        if self.power >= DIE_LIMIT {
            return;
        }
        if self.kind == TYPE_TORCH || self.kind == TYPE_REBURN {
            self.mode = MODE_IDLE;
            if self.on_switch != NO_SWITCH {
                switches.clear(self.on_switch);
                self.armed = self.on_switch + ARMED_BIAS;
            }
        } else {
            self.gone = true;
        }
    }

    fn execute(
        &mut self,
        pos: Vec3,
        angle: [i16; 3],
        arrows: &[Vec3],
        manager: &mut EventManager,
        switches: &mut Switches,
        collision: Option<&Collision>,
        events: &mut SimEvents,
    ) {
        self.top = self.top_of(pos, angle);
        if self.wait != 0 && self.wait < HOLD_CAP {
            self.wait -= 1;
        }
        match self.mode {
            MODE_IDLE => self.idle(manager, switches, arrows, events),
            MODE_CATCH | MODE_BURN => self.burn(manager, switches, events),
            MODE_DYING => self.dying(switches),
            _ => {}
        }
        if self.mode > MODE_BURN_FLOOR && self.mode < MODE_BURN_CEIL {
            self.probe = self.probe.wrapping_add(1);
            if u32::from(self.probe) & GROUND_MASK == 0 {
                let below = self.top + Vec3::Y * GROUND_PROBE;
                if let Some(height) = ground::surface(collision, below) {
                    if self.top.y < height {
                        self.mode = MODE_DYING;
                    }
                }
            }
        }
    }
}

fn add_calc0(value: &mut f32, scale: f32, max_step: f32) {
    let step = (*value * scale).clamp(-max_step, max_step);
    *value -= step;
}

fn add_calc2(value: &mut f32, target: f32, scale: f32, max_step: f32) {
    if *value != target {
        let step = (scale * (target - *value)).clamp(-max_step, max_step);
        *value += step;
    }
}

pub fn is_ep(name: &str) -> bool {
    EP_ACTORS.contains(&name)
}

pub fn control(
    collision: Option<Res<Collision>>,
    mut manager: ResMut<EventManager>,
    mut switches: ResMut<Switches>,
    mut events: ResMut<SimEvents>,
    arrows: Query<&Arrow>,
    mut commands: Commands,
    mut torches: Query<(Entity, &Actor, &mut Ep)>,
) {
    let solid = collision.as_deref();
    let shots: Vec<Vec3> = arrows
        .iter()
        .filter(|shot| shot.kind == TYPE_FIRE)
        .map(|shot| shot.pos)
        .collect();
    for (entity, actor, mut torch) in &mut torches {
        torch.execute(
            actor.pos,
            actor.angle,
            &shots,
            &mut manager,
            &mut switches,
            solid,
            &mut events,
        );
        if torch.gone {
            let _ = commands.entity(entity).despawn();
        }
    }
}
