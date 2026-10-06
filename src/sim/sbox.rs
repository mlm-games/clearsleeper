use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::event::{ActiveEvent, EventList};
use super::events::{SimEvent, SimEvents};
use super::flags::EventFlags;
use super::ship::Ship;
use super::walk;

/// `OBJNAME("tkrSlv", fpcNm_SBOX_e, 255, 62)` (`d_stage.cpp:502`) is the only `fpcNm_SBOX_e` row
/// in the table. The object archive is `"Dalways"` (`d_a_sbox.cpp:14`), a shared archive this actor
/// does not name in any stage string - RULE 4's mismatch case. Nothing else in `src/sim/` claims
/// `tkrSlv`.
pub const SBOX_ACTORS: [&str; 1] = ["tkrSlv"];

/// `fopAc_cullSizeBox[/* fopAc_CULLBOX_6_e */] = fopAc_cullSizeBox(cXyz(-150.0f, 0.0f, -150.0f),
/// cXyz(150.0f, 300.0f, 150.0f))` (`f_op_actor_mng.cpp:558`) - the profile's `fopAc_CULLBOX_6_e`
/// (`d_a_sbox.cpp:538`).
const CULL_MIN: [f32; 3] = [-150.0, 0.0, -150.0]; // <- f_op_actor_mng.cpp:558
const CULL_MAX: [f32; 3] = [150.0, 300.0, 150.0]; // <- f_op_actor_mng.cpp:558

/// `mDoMtx_stack_c::transM(0.0f, 22.35f, 20.0f)` (`d_a_sbox.cpp:112`) - `mpModel1`'s body-local seat
/// on the ship.
const SEAT_Y: f32 = 22.35; // <- d_a_sbox.cpp:112
const SEAT_Z: f32 = 20.0; // <- d_a_sbox.cpp:112
/// `mDoMtx_stack_c::transM(0.0f, 72.35f, 20.0f)` (`d_a_sbox.cpp:124`) - `mpModel2`'s, i.e. the
/// treasure-flash model's.
const FLASH_Y: f32 = 72.35; // <- d_a_sbox.cpp:124
/// `mDoMtx_stack_c::YrotM(0x7fff)` (`d_a_sbox.cpp:113`) and
/// `current.angle.y = ship->shape_angle.y + 0x7fff` (`:120`). One full turn is `0x10000`, so this is
/// half a turn minus one angle unit.
const HALF_TURN_MINUS_ONE: i16 = 0x7FFF; // <- d_a_sbox.cpp:113

/// `m2F8 = dComIfGp_evmng_getMyStaffId("tkrSlv")` (`d_a_sbox.cpp:395`) - the STAFF name, which is the
/// same string as the stage placement but a different table.
const STAFF_TKRSLV: &str = "tkrSlv"; // <- d_a_sbox.cpp:395
/// `static char* action_table[] = {"WAIT", "OPEN", "DELETE"};` (`d_a_sbox.cpp:388`).
const ACTION_WAIT: &str = "WAIT"; // <- d_a_sbox.cpp:388
const ACTION_OPEN: &str = "OPEN"; // <- d_a_sbox.cpp:388
const ACTION_DELETE: &str = "DELETE"; // <- d_a_sbox.cpp:388
/// `dComIfGp_evmng_getMyIntegerP(m2F8, "Timer")` (`d_a_sbox.cpp:262`).
const KEY_TIMER: &str = "Timer"; // <- d_a_sbox.cpp:262
/// `dComIfGp_evmng_getMyIntegerP(m2F8, "DISP")` (`d_a_sbox.cpp:361`).
const KEY_DISP: &str = "DISP"; // <- d_a_sbox.cpp:361

/// `fopAcM_seStart(this, JA_SE_OBJ_TBOX_OPEN_S1, 0)` (`d_a_sbox.cpp:287`) -
/// `JA_SE_OBJ_TBOX_OPEN_S1 = 0x690C` (`JAZelAudio_SE.h:1801`).
const SE_TBOX_OPEN_S1: u32 = 0x690C; // <- JAZelAudio_SE.h:1801
/// `fopAcM_seStart(this, JA_SE_OBJ_TBOX_UNLOCK, 0)` (`d_a_sbox.cpp:288`) -
/// `JA_SE_OBJ_TBOX_UNLOCK = 0x690B` (`JAZelAudio_SE.h:1800`).
const SE_TBOX_UNLOCK: u32 = 0x690B; // <- JAZelAudio_SE.h:1800
/// `fopAcM_seStart(this, JA_SE_OBJ_TBOX_OPEN_S2, 0)` (`d_a_sbox.cpp:327`) -
/// `JA_SE_OBJ_TBOX_OPEN_S2 = 0x690D` (`JAZelAudio_SE.h:1802`).
const SE_TBOX_OPEN_S2: u32 = 0x690D; // <- JAZelAudio_SE.h:1802
/// `dDoAud_subBgmStart(JA_BGM_OPEN_BOX)` (`d_a_sbox.cpp:290`) -
/// `JA_BGM_OPEN_BOX = 0x80000009` (`JAZelAudio_BGM.h:15`).
const BGM_OPEN_BOX: u32 = 0x8000_0009; // <- JAZelAudio_BGM.h:15

/// `setFlag(1)` (`d_a_sbox.cpp:403`) / `clrFlag(1)` (`:329`) / `chkFlag(1)` (`:325`) - the "opening
/// box BCK is rolling" bit.
const FLAG_BCK: u32 = 0x01; // <- d_a_sbox.cpp:403
/// `setFlag(2)` (`:291`) / `chkFlag(2)` (`:325`, `:338`, `:368`) / `clrFlag(2 | 4)` (`:381`) - the
/// "the box is open" state that gates the treasure-flash animation group.
const FLAG_OPEN: u32 = 0x02; // <- d_a_sbox.cpp:291
/// `setFlag(4)` (`:374`) / `chkFlag(4)` (`:466`) / `clrFlag(2 | 4)` (`:381`) - "the flash animation
/// has reached its reveal frame".
const FLAG_FLASH: u32 = 0x04; // <- d_a_sbox.cpp:374
/// `setFlag(8)` (`:362`) / `chkFlag(8)` (`:456`) - "this cut has a DISP integer, draw the box".
const FLAG_DISP: u32 = 0x08; // <- d_a_sbox.cpp:362
/// `setFlag(0x10)` (`:317`) / `chkFlag(0x10)` (`:156`, `:344`) / `clrFlag(0x10)` (`:164`, `:347`) - the
/// global colour-ratio darkening.
const FLAG_DARK: u32 = 0x10; // <- d_a_sbox.cpp:317

/// The Windfall slot-machine prize box - `daSbox_c` (`d_a_sbox.h`), the actor behind `tkrSlv`. It
/// rides the King of Red Lions' deck and, while its `tkrSlv` staff cut is live, waits out a `Timer`
/// integer and then plays the box-opening sequence.
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. No `dCcD_SrcCyl`/`dCcD_SrcSph` is declared
/// anywhere in this translation unit; the profile `Group` is `fopAc_ACTOR_e` (`d_a_sbox.cpp:527`) and
/// `Status` is `fopAcStts_UNK4000_e | fopAcStts_UNK40000_e` (`:536`) - no damage bit. Neither
/// `combat::HitPoints` nor `enemy::Hurt` is attached.
///
/// `daSbox_IsDelete` (`:499-501`) is the bare "accepts a delete request" form of RULE 6b. The real
/// destruction is `fopAcM_delete(this)` inside `actionWait` (`:447`) when no staff cut is live, which
/// is modelled as a despawn rather than a poll.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Sbox {
    /// `fopAc_ac_c::mFlag` - see the `FLAG_*` masks above.
    flag: u32,
    /// `m2F4` (`d_a_sbox.cpp:264`) - the `Timer` countdown, from `demoInitWait`.
    timer: i32,
}

impl Sbox {
    /// `daSbox_c::create` (`d_a_sbox.cpp:244-258`) minus `fopAcM_ct_Retail` / `fopAcM_ct_Demo`, the
    /// `dComIfG_resLoad` round trip and `fopAcM_entrySolidHeap` (`:252-254`), plus `CreateInit`'s
    /// `setAction(0)` (`:238`). `CreateInit` returns TRUE unconditionally (`:240`), so there is no
    /// `cPhs_ERROR_e` path outside the heap guard and no `refused` flag is needed.
    ///
    /// `m2EC` (`setAction(0)`) is not stored: `execute`'s `switch (m2EC)` (`:485-490`) has exactly one
    /// case and no other write, so the arm is unconditional here.
    ///
    /// RULE 2: `dStage_actorInit` never writes `fopAcM_prm_class::scale`; `calcMtx`'s
    /// `mpModel->setBaseScale(scale)` (`:94`, `:99`) is renderer work.
    pub fn spawn() -> Self {
        Self { flag: 0, timer: 0 }
    }

    /// The profile's `fopAc_CULLBOX_6_e` box (`d_a_sbox.cpp:538` -> `f_op_actor_mng.cpp:558`).
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// Whether `draw()` (`:456`) would draw `mpModel1` at all - `if (!chkFlag(8)) { return TRUE; }`.
    pub fn drawn(&self) -> bool {
        self.flag & FLAG_DISP != 0
    }

    /// `mDoMtx_stack_c::YrotM` (`m_Do_mtx.cpp:136-141`) applied to a vector, i.e. `mDoMtx_YrotS`'s
    /// row 0 `(cos, 0, sin)` / row 2 `(-sin, 0, cos)` (`m_Do_mtx.cpp:116-130`) under
    /// `mDoMtx_multVecZero`. RULE 5b: the angle units come from `super::walk::from_s16` alone.
    fn yrot(angle: i16, v: Vec3) -> Vec3 {
        let (s, c) = walk::from_s16(angle).sin_cos();
        Vec3::new(v.x * c + v.z * s, v.y, -v.x * s + v.z * c)
    }

    /// ```
    /// void daSbox_c::shipMtx() {
    ///     daShip_c* ship = dComIfGp_getShipActor();
    ///     if (ship == NULL) { JUT_ASSERT(220, FALSE); }
    ///     mDoMtx_stack_c::copy(ship->getBodyMtx());
    ///     mDoMtx_stack_c::transM(0.0f, 22.35f, 20.0f);
    ///     mDoMtx_stack_c::YrotM(0x7fff);
    ///     mpModel1->setBaseTRMtx(mDoMtx_stack_c::get());
    ///     mDoMtx_multVecZero(mDoMtx_stack_c::get(), &current.pos);
    ///     attention_info.position = current.pos;
    ///     eyePos = current.pos;
    ///     current.angle.y = ship->shape_angle.y + 0x7fff;
    ///     shape_angle.y = ship->shape_angle.y;
    ///     mDoMtx_stack_c::copy(ship->getBodyMtx());
    ///     mDoMtx_stack_c::transM(0.0f, 72.35f, 20.0f);
    ///     mpModel2->setBaseTRMtx(mDoMtx_stack_c::get());
    /// }
    /// ```
    /// (`:106-126`)
    ///
    /// `ship->getBodyMtx()` is `translate(shipPos) * Yrot(shipYaw)`, so `mDoMtx_multVecZero` on the
    /// composed matrix is a body-local point added to the ship's position, pre-rotated by
    /// `HALF_TURN_MINUS_ONE` exactly as `current.angle.y` shows. The sim reads the ship's body facing
    /// from `Actor::angle[1]` - the same reduction `src/sim/buoybarrel.rs:452` and
    /// `src/sim/ship.rs:210` (`crane.tick_top(actor.pos, actor.angle[1])`) already make, because
    /// `super::ship::Ship` carries no separate `shape_angle`.
    ///
    /// `JUT_ASSERT(220, FALSE)` (`:109`) is compiled out of a retail build, so a missing ship makes
    /// retail dereference NULL on the next line. There is no defined behaviour to transcribe; the
    /// sim leaves the placement untouched and reports it.
    ///
    /// `attention_info` / `eyePos` (`:118-119`) are RULE 3b's attention requests, and both
    /// `setBaseTRMtx` calls are presentation, so only `current.pos` and `current.angle.y` survive.
    fn ship_mtx(&self, ship_pos: Vec3, ship_yaw: i16, out: &mut Actor) {
        let seat = Vec3::new(0.0, SEAT_Y, SEAT_Z);
        let world = Self::yrot(ship_yaw.wrapping_add(HALF_TURN_MINUS_ONE), seat);
        out.pos = ship_pos + world;
        out.angle[1] = ship_yaw.wrapping_add(HALF_TURN_MINUS_ONE);
    }

    /// ```
    /// void daSbox_c::demoInitWait() {
    ///     int* puVar1 = dComIfGp_evmng_getMyIntegerP(m2F8, "Timer");
    ///     if (puVar1 != NULL) { m2F4 = *puVar1; } else { m2F4 = 0; }
    /// }
    /// ```
    /// (`:261-268`)
    fn demo_init_wait(&mut self, cut: usize, list: &EventList) {
        self.timer = list.cut_number(cut, KEY_TIMER).unwrap_or(0);
    }

    /// ```
    /// BOOL daSbox_c::demoProcWait() {
    ///     if (m2F4 > 0) { m2F4--; } else { dComIfGp_evmng_cutEnd(m2F8); }
    ///     return FALSE;
    /// }
    /// ```
    /// (`:271-278`)
    ///
    /// The `return FALSE` is `demoProc`'s own return value, not a second signal - `actionWait` only
    /// tests it (`:446`).
    fn demo_proc_wait(&mut self, ended: &mut bool) {
        if self.timer > 0 {
            self.timer -= 1;
        } else {
            // `dComIfGp_evmng_cutEnd(m2F8);` (:275)
            *ended = true;
        }
    }

    /// ```
    /// void daSbox_c::demoInitOpen() {
    ///     mBck1.setPlaySpeed(1.0f); mBck2.setPlaySpeed(1.0f);
    ///     mBtk.setPlaySpeed(1.0f); mBrk.setPlaySpeed(1.0f);
    ///     fopAcM_seStart(this, JA_SE_OBJ_TBOX_OPEN_S1, 0);
    ///     fopAcM_seStart(this, JA_SE_OBJ_TBOX_UNLOCK, 0);
    ///     dDoAud_subBgmStart(JA_BGM_OPEN_BOX);
    ///     setFlag(2);
    ///     dComIfGp_particle_set(... x6 ...);
    ///     setFlag(0x10);
    ///     m304 = 0.4f; m308 = 0;
    ///     lightInit();
    /// }
    /// ```
    /// (`:281-321`)
    ///
    /// Dropped: the four `setPlaySpeed` calls (`:282-285`), the six `dComIfGp_particle_set` calls
    /// (`:293-315`, RULE 3b's particle system), and `lightInit` (`:320`, `dKy_plight_priority_set`
    /// / `dKy_efplight_set` - RULE 3b's light source). `m304` / `m308` are read only by `darkProc`,
    /// whose one effect is `dKy_set_allcol_ratio` (`:163`, `:171`, `:345`) - the global colour ratio,
    /// which has no resource here - so they are not stored; `FLAG_DARK` is still set because
    /// `darkProc`'s early return at `:156` is what it gates.
    fn demo_init_open(&mut self, events: &mut SimEvents) {
        events.list.push(SimEvent::Se(SE_TBOX_OPEN_S1));
        events.list.push(SimEvent::Se(SE_TBOX_UNLOCK));
        events.list.push(SimEvent::Bgm(BGM_OPEN_BOX));
        // `setFlag(2);` (:291)
        self.flag |= FLAG_OPEN;
        // `setFlag(0x10);` (:317)
        self.flag |= FLAG_DARK;
    }

    /// ```
    /// void daSbox_c::demoProcOpen() {
    ///     if (chkFlag(1)) {
    ///         if (mBck1.play()) {
    ///             fopAcM_seStart(this, JA_SE_OBJ_TBOX_OPEN_S2, 0);
    ///             dComIfGp_evmng_cutEnd(m2F8);
    ///             clrFlag(1);
    ///         }
    ///     } else { dComIfGp_evmng_cutEnd(m2F8); }
    /// }
    /// ```
    /// (`:324-334`)
    ///
    /// `mBck1.play()` returns "the animation reached its end", which reads BCK archive frame data
    /// that does not exist in this checkout (BUG CLASS 2) - no stand-in frame count is invented, so
    /// the whole `chkFlag(1)` body is dropped and only the `else` arm survives. The consequence is
    /// that `FLAG_BCK` stays set and the cut never ends from here; see the report.
    fn demo_proc_open(&mut self, ended: &mut bool) {
        if self.flag & FLAG_BCK == 0 {
            // `} else { dComIfGp_evmng_cutEnd(m2F8); }` (:332-334)
            *ended = true;
        }
    }

    /// ```
    /// void daSbox_c::demoInitDelete() {
    ///     if (chkFlag(2)) {
    ///         if (mEmitter != NULL) { mEmitter->becomeInvalidEmitter(); mEmitter = NULL; }
    ///         if (chkFlag(0x10)) { dKy_set_allcol_ratio(1.0f); clrFlag(0x10); }
    ///     }
    ///     dKy_plight_cut(&m30C);
    ///     dKy_efplight_cut(&m32C);
    /// }
    /// ```
    /// (`:337-352`)
    ///
    /// Dropped: the emitter teardown (`:340-342`, particle system) and the two light cuts (`:350-351`,
    /// RULE 3b's light source). `dKy_set_allcol_ratio(1.0f)` (`:345`) is the global colour ratio with
    /// no resource here, but the `clrFlag(0x10)` beside it is a plain flag write and is kept.
    fn demo_init_delete(&mut self) {
        if self.flag & FLAG_OPEN != 0 && self.flag & FLAG_DARK != 0 {
            self.flag &= !FLAG_DARK;
        }
    }

    /// ```
    /// void daSbox_c::demoInitCom() {
    ///     if (dComIfGp_evmng_getMyIntegerP(m2F8, "DISP") != NULL) { setFlag(8); }
    /// }
    /// ```
    /// (`:360-365`)
    fn demo_init_com(&mut self, cut: usize, list: &EventList) {
        if list.cut_number(cut, KEY_DISP).is_some() {
            self.flag |= FLAG_DISP;
        }
    }

    /// ```
    /// BOOL daSbox_c::demoProc() {
    ///     m2F8 = dComIfGp_evmng_getMyStaffId("tkrSlv");
    ///     if (m2F8 == -1) { return FALSE; }
    ///     s32 iVar1 = getNowEventAction();
    ///     if (dComIfGp_evmng_getIsAddvance(m2F8)) {
    ///         setFlag(1);
    ///         demoInitCom();
    ///         switch (iVar1) { case 0: demoInitWait(); case 1: demoInitOpen(); case 2: demoInitDelete(); }
    ///     }
    ///     switch (iVar1) {
    ///         case 0: demoProcWait(); break;
    ///         case 1: demoProcOpen(); break;
    ///         case 2: demoProcDelete(); break;
    ///         default: dComIfGp_evmng_cutEnd(m2F8); break;
    ///     }
    ///     demoProcCom();
    ///     return TRUE;
    /// }
    /// ```
    /// (`:394-441`)
    ///
    /// `getNowEventAction` (`:387-391`) is
    /// `dComIfGp_evmng_getMyActIdx(m2F8, action_table, 3, FALSE, 1)`; both branches of that function
    /// compare the current cut NAME for EQUALITY (`d_event_manager.cpp:472-481`), so `nameType = 1`
    /// only selects `dEvmng_strcmp` over `strcmp`. An unrecognised cut name yields -1, which lands in
    /// the `default` arm. Retail also caches the answer in `staff->mCurActionIdx`
    /// (`d_event_manager.cpp:466-468`, `:474`, `:484`); the sim's `ActiveEvent::staff_cut` re-reads the
    /// cut name each tick, which gives the same answer while the cut only advances on an advance tick.
    ///
    /// `demoProcCom` (`:367-384`) is OMITTED: every one of its six inputs is unavailable - `mBck2.play()`
    /// / `mBtk.play()` / `mBrk.play()` (BCK frame data), `(s32)mBck2.getFrame() == 36` (BCK frame
    /// data), `volmProc` (`mEmitter`, particle system), `darkProc` (`dKy_set_allcol_ratio`) and
    /// `lightProc` (`dKy_plight_*`).
    ///
    /// The two ways `demoProc` (`:394-441`) reports back: `m2F8 == -1` returns FALSE, which makes
    /// `actionWait` (`:447`) delete the actor, while TRUE keeps it alive - but the cut itself may
    /// still have been ended, which is a separate `dComIfGp_evmng_cutEnd(m2F8)` call.
    fn demo_proc(
        &mut self,
        cut: Option<(usize, bool)>,
        list: &EventList,
        events: &mut SimEvents,
    ) -> Step {
        let Some((cut, advance)) = cut else {
            return Step::Delete;
        };
        let name = list.cut_name(cut).unwrap_or("");
        let mut ended = false;
        if advance {
            // `setFlag(1);` (:403)
            self.flag |= FLAG_BCK;
            self.demo_init_com(cut, list);
            match name {
                ACTION_WAIT => self.demo_init_wait(cut, list),
                ACTION_OPEN => self.demo_init_open(events),
                ACTION_DELETE => self.demo_init_delete(),
                _ => {}
            }
        }
        match name {
            ACTION_WAIT => self.demo_proc_wait(&mut ended),
            ACTION_OPEN => self.demo_proc_open(&mut ended),
            // `void daSbox_c::demoProcDelete() { dComIfGp_evmng_cutEnd(m2F8); }` (:355-357)
            ACTION_DELETE => ended = true,
            // `default: dComIfGp_evmng_cutEnd(m2F8); break;` (:434-436)
            _ => ended = true,
        }
        Step::Live { end_cut: ended }
    }

    /// ```
    /// BOOL daSbox_c::actionWait() {
    ///     shipMtx();
    ///     if (!demoProc()) { fopAcM_delete(this); }
    ///     return TRUE;
    /// }
    /// ```
    /// (`:444-450`)
    fn action_wait(
        &mut self,
        cut: Option<(usize, bool)>,
        list: &EventList,
        events: &mut SimEvents,
        ship: Option<(Vec3, i16)>,
        actor: &mut Actor,
    ) -> Step {
        if let Some((ship_pos, ship_yaw)) = ship {
            self.ship_mtx(ship_pos, ship_yaw, actor);
        }
        self.demo_proc(cut, list, events)
    }
}

/// What one `actionWait` (`:444-450`) decided.
enum Step {
    /// `demoProc` returned FALSE - `fopAcM_delete(this)` (`:447`).
    Delete,
    /// `demoProc` returned TRUE; `end_cut` says whether a branch also called
    /// `dComIfGp_evmng_cutEnd(m2F8)`.
    Live { end_cut: bool },
}

pub fn is_sbox(name: &str) -> bool {
    SBOX_ACTORS.contains(&name)
}

/// Access set per parameter, for RULE 9(b):
/// * `list` - `Res<EventList>`, shared read.
/// * `active` - `ResMut<ActiveEvent>`, exclusive.
/// * `ev_flags` - `ResMut<EventFlags>`, exclusive (`end_staff_cut` writes the cut's flag id).
/// * `events` - `ResMut<SimEvents>`, exclusive.
/// * `ships` - `Query<(&Actor, &Ship), Without<Sbox>>`, shared read. Disjoint from `boxes` through
///   the `With<Sbox>` / `Without<Sbox>` pair.
/// * `boxes` - `Query<(Entity, &mut Actor, &mut Sbox), With<Sbox>>`, writes `Actor` + `Sbox`.
/// * `commands` - `Commands`.
pub fn control(
    list: Res<EventList>,
    mut active: ResMut<ActiveEvent>,
    mut ev_flags: ResMut<EventFlags>,
    mut events: ResMut<SimEvents>,
    ships: Query<(&Actor, &Ship), Without<Sbox>>,
    mut boxes: Query<(Entity, &mut Actor, &mut Sbox), With<Sbox>>,
    mut commands: Commands,
) {
    // `dComIfGp_getShipActor()` - the one live `daShip_c`; `d_stage.cpp:650` is the only
    // `fpcNm_SHIP_e` row and `src/sim/ship.rs:12` claims it.
    let ship = ships.iter().next().map(|(hull, _)| (hull.pos, hull.angle[1]));
    for (entity, mut actor, mut box_) in &mut boxes {
        // `m2F8 = dComIfGp_evmng_getMyStaffId("tkrSlv")` (:395). `ActiveEvent::staff_cut` answers the
        // staff id AND `dComIfGp_evmng_getIsAddvance` in one lookup, which is the whole of `m2F8`'s
        // two reads in this translation unit.
        let cut = active.staff_cut(&list, STAFF_TKRSLV);
        // `execute()` -> `case 0: actionWait();` (:485-489) - `m2EC` has no other write.
        match box_.action_wait(cut, &list, &mut events, ship, &mut actor) {
            // `if (!demoProc()) { fopAcM_delete(this); }` (:447) - a genuine self-destruction, not
            // the RULE 6b delete-request form.
            Step::Delete => {
                let _ = commands.entity(entity).despawn();
            }
            Step::Live { end_cut: true } => {
                // `dComIfGp_evmng_cutEnd(m2F8)` (`:275`, `:332`, `:356`, `:435`).
                active.end_staff_cut(&list, &mut ev_flags, STAFF_TKRSLV);
            }
            Step::Live { end_cut: false } => {}
        }
    }
}
