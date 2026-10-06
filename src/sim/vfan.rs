use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::boko::Boko;
use super::event::{ActiveEvent, EventList};
use super::events::{SimEvent, SimEvents};
use super::flags::{EventFlags, Switches};
use super::phantom_ganon::BOKO_SWORD;

pub const VFAN_ACTORS: [&str; 1] = ["Vfan"];

const SWSAVE_MASK: u32 = 0xFF; // <- d_a_obj_vfan.h:16-17 (PRM_SWSAVE_W = 8, PRM_SWSAVE_S = 0)
const NO_SWITCH: u8 = 0xFF; // <- d_a_obj_vfan.cpp:91 (the switch Mthd_Create tests is a real index)
const CYL_RADIUS: f32 = 100.0; // <- d_a_obj_vfan.cpp:44 (dCcD_SrcCyl Radius)
const CYL_HEIGHT: f32 = 300.0; // <- d_a_obj_vfan.cpp:45 (dCcD_SrcCyl Height)
const CPS_R_PGANON_SWORD: f32 = 50.0; // <- d_a_boko_static.cpp:45 (daBoko_c::m_cps_r[Type_PGANON_SWORD_e])
const STATE_WAIT: u8 = 0; // <- d_a_obj_vfan.cpp:78 (mState = 0), :154
const STATE_DEMO: u8 = 1; // <- d_a_obj_vfan.cpp:163
const STATE_BREAK: u8 = 2; // <- d_a_obj_vfan.cpp:175
const BREAK_TICKS: i32 = 150; // <- d_a_obj_vfan.cpp:183 (mBreakTimer == 150)
const EVENT_VFAN: &str = "Vfan"; // <- d_a_obj_vfan.cpp:80 (dComIfGp_evmng_getEventIdx("Vfan"))
const EVENT_BIT: u32 = 0x3A08; // <- d_a_obj_vfan.cpp:192 (dSv_event_flag_c::UNK_3A08)
const SE_READ_RIDDLE_1: u32 = 0x0806; // <- JAZelAudio_SE.h:18
const SE_OBJ_GN_SW_DR_LIGHT: u32 = 0x6A20; // <- JAZelAudio_SE.h:2055
const SE_OBJ_GN_SW_DR_BREAK: u32 = 0x6A21; // <- JAZelAudio_SE.h:2056

pub const CULL_MIN: [f32; 3] = [-4000.0, -500.0, -4000.0]; // <- d_a_obj_vfan.cpp:69
pub const CULL_MAX: [f32; 3] = [4000.0, 500.0, 4000.0]; // <- d_a_obj_vfan.cpp:69

/// `daObjVfan::Act_c` (`d_a_obj_vfan.h:13-44`): Ganon's Tower - Phantom Ganon door. The stage string
/// is `Vfan` (`d_stage.cpp:1096`) and the archive is `Vfan` (`d_a_obj_vfan.cpp:17`).
///
/// `Mthd_Create` (`:86-102`) returns `cPhs_STOP_e` when `prm_get_swSave` is already set (`:90-91`);
/// `cPhs_STOP_e` and `cPhs_ERROR_e` both reach `fpcCtRq_Cancel`
/// (`f_pc_create_req.cpp:107-108`, `c_phase.h:9`), so the actor is cancelled inside Create and never
/// reaches `Execute` - it is spawned and dropped on its first tick, not self-destructed.
///
/// `cyl_check_src` (`:19-47`) is a `dCcD_SrcCyl` of radius 100 and height 300 whose Tg type is
/// `AT_TYPE_PGANON_SWORD` (`:26`, `d_cc_d.h:41` = `1 << 29`) with
/// `cCcD_TgSPrm_Set_e | cCcD_TgSPrm_IsOther_e` (`:27`). That bit is registered by exactly one
/// actor, `daBoko_c` with `Type_PGANON_SWORD_e` (param 5): `d_a_boko.cpp:998` applies it with
/// `mCps.SetAtType(getAtType())` and `daBoko_c::m_at_type[5]` is `AT_TYPE_PGANON_SWORD`
/// (`d_a_boko_static.cpp:61`). Its AT capsule runs `m2DC` -> `m2D0` with radius
/// `m_cps_r[5] = 50.0f` (`d_a_boko_static.cpp:45`, `d_a_boko.cpp:607`, `:1001`) - `super::boko`'s
/// `root` and `top`. `super::phantom_ganon` spawns that stick with `BOKO_SWORD = 5`
/// (`phantom_ganon.rs:2724`). So this actor does have an inbound damage path, and it is a hit, not
/// damage: no `HitPoints` is attached, exactly as for the other six.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Vfan {
    pub switch_no: u8,
    /// `mState` (`d_a_obj_vfan.h:41`).
    pub state: u8,
    /// `mIsAlive` (`d_a_obj_vfan.h:42`); `Draw` (`:211`) skips its draw call when false.
    pub alive: bool,
    /// `mBreakTimer` (`d_a_obj_vfan.h:43`).
    pub break_timer: i32,
    /// `m_evid` (`d_a_obj_vfan.cpp:13`), the `Vfan` event index; `None` is `getEventIdx`'s -1.
    pub event_slot: Option<usize>,
    /// `Mthd_Create`'s `cPhs_STOP_e` arm, which cancels the actor inside Create.
    pub refused: bool,
    pub entered: bool,
}

impl Vfan {
    /// `Create` (`d_a_obj_vfan.cpp:64-83`) with `Mthd_Create`'s `cPhs_STOP_e` arm (`:90-91`),
    /// minus `fopAcM_SetMtx`, `init_mtx`, `fopAcM_setCullSizeBox`, `mStts.Init`, `mCyl.Set`,
    /// `mCyl.SetC`, `mCyl.SetStts` and the `m_evid` lookup, which needs the stage event list and so
    /// happens on the first tick. The `mBreakTimer = 0` and `mState = 0` of `:77-78` are the
    /// initialiser values.
    pub fn spawn(parameters: u32, switches: &Switches) -> Self {
        let switch_no = (parameters & SWSAVE_MASK) as u8;
        Self {
            switch_no,
            state: STATE_WAIT,
            alive: true,
            break_timer: 0,
            event_slot: None,
            refused: switch_no != NO_SWITCH && switches.test(switch_no),
            entered: false,
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        self.alive
    }

    /// The `m_evid` lookup of `Create` (`:80`). `getEventIdx` is called with the default
    /// `evNo = -1` (`d_com_inf_game.h:3611`), which is not the `0xff` of the stage event-info
    /// redirect at `:347`, so the lookup is by name over the event list alone.
    fn create_init(&mut self, list: &EventList) {
        self.event_slot = list.event_index(EVENT_VFAN);
    }

    /// `fopAcM_orderOtherEventId(this, m_evid)` (`:162`, `:177`) -> `ActiveEvent::order`. The
    /// `active.stage_of(slot).is_none()` guard is the sim's stand-in for
    /// `dEvent_manager_c::order`'s `getEventData(eventIdx) == NULL` bail
    /// (`d_event_manager.cpp:362-364`): re-ordering an event that is already ORDER/PLAY/CLOSE
    /// rewrites `mEventState` to ORDER (`:367`), which in the sim would reset `Stage::Order` on
    /// every tick and livelock the demo before `start_check` could ever hold. This is the guard
    /// `vgnfd.rs:189` uses.
    fn order(&self, list: &EventList, active: &mut ActiveEvent, entity: Entity) {
        let Some(slot) = self.event_slot else {
            return;
        };
        if active.stage_of(slot).is_none() {
            let _ = active.order(list, slot, entity);
        }
    }

    /// `Execute` (`d_a_obj_vfan.cpp:152-202`), minus `set_mtx` (`:199`) and the `*mtx` store
    /// (`:200`). The switch has no `default` arm and `mState` only ever holds 0..=2, so the
    /// fall-through arm is exactly `case 2`. Returns true when `fopAcM_delete(this)` (`:194`)
    /// fires; the `ParticleSet` of `:173` is particles only.
    fn execute(
        &mut self,
        actor: &Actor,
        swords: &Query<&Boko>,
        list: &EventList,
        active: &mut ActiveEvent,
        switches: &mut Switches,
        ev_flags: &mut EventFlags,
        events: &mut SimEvents,
        entity: Entity,
    ) -> bool {
        match self.state {
            STATE_WAIT => {
                if Self::sword_touches(actor, swords) {
                    self.order(list, active, entity);
                    self.state = STATE_DEMO;
                }
            }
            STATE_DEMO => {
                if self
                    .event_slot
                    .is_some_and(|slot| active.start_check(slot))
                {
                    events.list.push(SimEvent::Se(SE_READ_RIDDLE_1));
                    events.list.push(SimEvent::Se(SE_OBJ_GN_SW_DR_LIGHT));
                    self.break_timer = 0;
                    self.state = STATE_BREAK;
                } else {
                    self.order(list, active, entity);
                }
            }
            STATE_BREAK => {
                if self.break_timer == BREAK_TICKS {
                    self.alive = false;
                    events.list.push(SimEvent::Se(SE_OBJ_GN_SW_DR_BREAK));
                }
                self.break_timer += 1;
                if self
                    .event_slot
                    .is_some_and(|slot| active.end_check(slot))
                {
                    switches.set(self.switch_no);
                    ev_flags.set(EVENT_BIT);
                    if let Some(slot) = self.event_slot {
                        let _ = active.end(slot);
                    }
                    return true;
                }
            }
            _ => {}
        }
        false
    }

    /// `mCyl.ChkTgHit()` (`:156`) for the door's own `AT_TYPE_PGANON_SWORD` Tg cylinder, which was
    /// centred once on `current.pos` by `Create` (`:73`) and re-registered every tick at `:155`.
    ///
    /// `cM3d_Cross_CpsCyl` (`c_m3d.cpp:1963-2062`) asks exactly whether the two solids overlap, and
    /// its general case is `dist(seg(capsule), seg(cylinder axis)) < cps.R + cyl.R` - the test at
    /// `:2015` / `:2058`. The cylinder axis is the `Center` raised by `cyl.h`.
    fn sword_touches(actor: &Actor, swords: &Query<&Boko>) -> bool {
        let base = actor.pos;
        let axis = (base, base + Vec3::new(0.0, CYL_HEIGHT, 0.0));
        swords.iter().any(|stick| {
            stick.kind == BOKO_SWORD as u8
                && seg_distance((stick.root, stick.top), axis) < CYL_RADIUS + CPS_R_PGANON_SWORD
        })
    }
}

/// `cM3d_Len3dSqPntAndSegLine` / `cM3d_Check_LinLin` (`c_m3d.cpp:1952-2062`) reduced to the shortest
/// distance between the two segments, so the strict `<` of `:2015` / `:2058` is preserved.
fn seg_distance(a: (Vec3, Vec3), b: (Vec3, Vec3)) -> f32 {
    let (s, t) = seg_params(a.0, a.1, b.0, b.1);
    ((a.0 + s * (a.1 - a.0)) - (b.0 + t * (b.1 - b.0))).length()
}

/// The `cM3d_Check_LinLin` barycentric pair: the parameters of the closest points of two segments.
fn seg_params(p0: Vec3, p1: Vec3, q0: Vec3, q1: Vec3) -> (f32, f32) {
    let u = p1 - p0;
    let v = q1 - q0;
    let r = p0 - q0;
    let a = u.dot(u);
    let e = v.dot(v);
    let f = v.dot(r);
    let clamp = |x: f32| x.max(0.0).min(1.0);
    if a <= 0.0 && e <= 0.0 {
        return (0.0, 0.0);
    }
    if a <= 0.0 {
        return (0.0, clamp(f / e));
    }
    let c = u.dot(r);
    if e <= 0.0 {
        return (clamp(-c / a), 0.0);
    }
    let b = u.dot(v);
    let denom = a * e - b * b;
    let s = if denom > 0.0 {
        clamp((b * f - c * e) / denom)
    } else {
        0.0
    };
    let t = (b * s + f) / e;
    if t < 0.0 {
        (clamp(-c / a), 0.0)
    } else if t > 1.0 {
        (clamp((b - c) / a), 1.0)
    } else {
        (s, t)
    }
}

pub fn is_vfan(name: &str) -> bool {
    VFAN_ACTORS.contains(&name)
}

pub fn control(
    swords: Query<&Boko>,
    list: Res<EventList>,
    mut active: ResMut<ActiveEvent>,
    mut switches: ResMut<Switches>,
    mut ev_flags: ResMut<EventFlags>,
    mut events: ResMut<SimEvents>,
    mut doors: Query<(Entity, &Actor, &mut Vfan)>,
    mut commands: Commands,
) {
    for (entity, actor, mut door) in &mut doors {
        let door = &mut *door;
        if door.refused {
            let _ = commands.entity(entity).despawn();
            continue;
        }
        if !door.entered {
            door.entered = true;
            door.create_init(&list);
        }
        if door.execute(
            actor,
            &swords,
            &list,
            &mut active,
            &mut switches,
            &mut ev_flags,
            &mut events,
            entity,
        ) {
            let _ = commands.entity(entity).despawn();
        }
    }
}