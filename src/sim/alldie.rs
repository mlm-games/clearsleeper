use bevy_ecs::prelude::*;

use super::actor::Actor;
use super::enemy::Enemy;
use super::flags::Switches;

/// The stage string is `ALLdie` (`d_stage.cpp:573`, `fpcNm_ALLDIE_e`). There is no `_arcname`:
/// `daAlldie_c` derives straight from `fopAc_ac_c` (`d_a_alldie.h:5`), holds only `mAction` and
/// `mTimer` (`:14-16`), never calls `dComIfG_resLoad`, and the profile's `Parameters` count is 0
/// (`d_a_alldie.cpp:127`).
pub const ALLDIE_ACTORS: [&str; 1] = ["ALLdie"];

/// `enum { ACT_WAIT, ACT_CHECK, ACT_TIMER }` (`d_a_alldie.cpp:11-15`) - the decomp's own order,
/// which `execute`'s `switch (mAction)` (`:52-62`) and every `setActio` rely on.
const ACT_WAIT: u8 = 0; // <- d_a_alldie.cpp:12
const ACT_CHECK: u8 = 1; // <- d_a_alldie.cpp:13
const ACT_TIMER: u8 = 2; // <- d_a_alldie.cpp:14

/// `u8 daAlldie_c::getSwbit() { return fopAcM_GetParam(this) >> 0x8; }` (`d_a_alldie.cpp:19`) -
/// the shift is `:19`, the `u8` return type `:18`. `getSwbit()` is consumed only as
/// `dComIfGs_isSwitch(getSwbit(), fopAcM_GetRoomNo(this))` (`:70`) and
/// `dComIfGs_onSwitch(getSwbit(), fopAcM_GetRoomNo(this))` (`:45`).
///
/// RULE 11b applies verbatim: `dSv_info_c::onSwitch`/`isSwitch` route `i_no >= MEMORY_SWITCH` to
/// the per-zone `dSv_zoneBit_c` arrays (`d_save.h:982`, `d_save.cpp:1539-1560`, `:1598-1601`), so
/// a real index of `0x80` or above reads false and does not `set()` here. The index is never
/// down-shifted.
const SWBIT_SHIFT: u32 = 0x8; // <- d_a_alldie.cpp:19
const SWBIT_MASK: u32 = 0xFF; // <- d_a_alldie.cpp:18-19 (u8 daAlldie_c::getSwbit())

/// `mTimer = 65` (`d_a_alldie.cpp:31`) - the `s16 mTimer` (`d_a_alldie.h:15`) countdown
/// `actionTimer` drains at `:42`.
const TIMER_DELAY: i16 = 65; // <- d_a_alldie.cpp:31

/// `fopAc_CULLBOX_6_e` (`d_a_alldie.cpp:133`) resolves through `l_cullSizeBox[6]`
/// (`f_op_actor_mng.cpp:564`) - a real box, so `cull()` is derivable.
const CULL_MIN: [f32; 3] = [-150.0, 0.0, -150.0]; // <- f_op_actor_mng.cpp:564 (fopAc_CULLBOX_6_e)
const CULL_MAX: [f32; 3] = [150.0, 300.0, 150.0]; // <- f_op_actor_mng.cpp:564 (fopAc_CULLBOX_6_e)

/// "All enemies dead" - `daAlldie_c` (`d_a_alldie.h:5-16`), the actor behind the stage string
/// `ALLdie`. It waits out a 65-tick grace period with no enemy in its room, then latches its
/// switch and parks.
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. `daAlldie_c` owns no `dCcD_Stts`, no
/// `dCcD_Cyl` and no `dCcD_Sph` at all - the class has exactly two members, `mAction` and `mTimer`
/// (`d_a_alldie.h:14-16`) - so it registers no AT, no Tg and no Co collider, and nothing in the
/// decomp ever damages it. The profile `Group` is `fopAc_ACTOR_e` (`:132`). Neither
/// `combat::HitPoints` nor `enemy::Hurt` is attached.
///
/// `daAlldie_Draw` (`:85-87`) is `return TRUE` with no draw call and the class holds no model, so
/// retail renders nothing at all: no `actor_views` branch is derivable.
///
/// `daAlldie_IsDelete` (`:96-98`) is the bare "accepts a delete request" form of RULE 6b, and
/// `create` (`:67-82`) always returns `cPhs_COMPLEATE_e`, so there is no spawn-then-despawn path.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Alldie {
    /// `mAction` (`d_a_alldie.h:14`), the `ACT_*` index `execute` dispatches on (`:52-62`).
    pub action: u8,
    /// `mTimer` (`d_a_alldie.h:15`), loaded by `actionCheck` (`:31`) and drained by `actionTimer`
    /// (`:42`).
    pub timer: i16,
    /// `getSwbit()` (`d_a_alldie.cpp:18-20`), the switch both `isSwitch` (`:70`) and `onSwitch`
    /// (`:45`) address.
    pub swbit: u8,
    /// One-shot guard so the `current.angle` stores of `create` (`:78-79`) happen on the first tick
    /// and not every tick.
    pub entered: bool,
}

impl Alldie {
    /// `daAlldie_c::create` (`d_a_alldie.cpp:67-82`), minus `fopAcM_ct` (`:68`), minus the
    /// `shape_angle` zeroing (`:76-77`) - shape angles are not modelled - and minus the
    /// `current.angle` zeroing (`:78-79`), which `create_init` applies on the first tick because
    /// nothing in the sim can change between Create and the first Execute.
    pub fn spawn(parameters: u32, switches: &Switches) -> Self {
        let swbit = ((parameters >> SWBIT_SHIFT) & SWBIT_MASK) as u8;
        Self {
            // `if (!dComIfGs_isSwitch(getSwbit(), fopAcM_GetRoomNo(this))) { setActio(ACT_CHECK); }
            //  else { setActio(ACT_WAIT); }` (:70-74)
            action: if switches.test(swbit) {
                ACT_WAIT
            } else {
                ACT_CHECK
            },
            timer: 0,
            swbit,
            entered: false,
        }
    }

    /// `fopAc_CULLBOX_6_e` (`d_a_alldie.cpp:133`) -> `l_cullSizeBox[6]` (`f_op_actor_mng.cpp:564`).
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// The two surviving stores of `create`: `current.angle.z = 0; current.angle.x = 0;` (`:78-79`).
    /// `Actor::angle` is `current.angle`, so that is `angle[2]` and `angle[0]`.
    fn create_init(&mut self, actor: &mut Actor) {
        actor.angle[2] = 0;
        actor.angle[0] = 0;
        self.entered = true;
    }

    /// `daAlldie_c::actionWait` (`:23-25`) is a bare `return TRUE` - it stores nothing and reads
    /// nothing, which is why `execute` needs no arm body beyond falling through to it.
    fn action_wait(&mut self) {}

    /// `daAlldie_c::actionCheck` (`:28-35`):
    /// `if (!fopAcM_myRoomSearchEnemy(fopAcM_GetRoomNo(this))) { setActio(ACT_TIMER); mTimer = 65; }`
    fn action_check(&mut self, room_enemy: bool) {
        if !room_enemy {
            self.action = ACT_TIMER;
            self.timer = TIMER_DELAY;
        }
    }

    /// `daAlldie_c::actionTimer` (`:38-48`): an enemy reappearing sends it back to `ACT_CHECK`,
    /// otherwise `mTimer` drains and the exhaustion arm latches the switch and parks in
    /// `ACT_WAIT`.
    fn action_timer(&mut self, room_enemy: bool, switches: &mut Switches) {
        if room_enemy {
            self.action = ACT_CHECK;
        } else if self.timer > 0 {
            self.timer -= 1;
        } else {
            self.action = ACT_WAIT;
            switches.set(self.swbit);
        }
    }

    /// `daAlldie_c::execute` (`:51-65`). The decomp's `switch (mAction)` has no `case ACT_WAIT`:
    /// `default:` is `actionWait` (`:59-61`), and `mAction` is a `u8` only ever loaded through
    /// `setActio`, so the fall-through arm is exactly `ACT_WAIT`.
    fn execute(&mut self, room_enemy: bool, switches: &mut Switches) {
        match self.action {
            ACT_CHECK => self.action_check(room_enemy),
            ACT_TIMER => self.action_timer(room_enemy, switches),
            _ => self.action_wait(),
        }
    }

    /// `fopAcM_myRoomSearchEnemy(fopAcM_GetRoomNo(this)) != NULL` (`:29`, `:39`). The room layer
    /// holds an `fopAc_ENEMY_e` actor (`f_op_actor_mng.cpp:1272-1278`, `:1282-1296`); the function's
    /// first arm, the player's grabbed actor (`:1289-1292`), is `catch` state, which RULE 3b lists
    /// among the confirmed-absent inputs. `fopAcM_GetRoomNo` is `current.roomNo`
    /// (`f_op_actor_mng.h:111-113`), which is `Actor::room` here.
    fn room_enemy(actor: &Actor, foes: &[i8]) -> bool {
        foes.contains(&actor.room)
    }
}

pub fn is_alldie(name: &str) -> bool {
    ALLDIE_ACTORS.contains(&name)
}

pub fn control(
    mut switches: ResMut<Switches>,
    // `With<Enemy>` / `Without<Enemy>` is the disjoint pair RULE 9 requires: this system writes
    // `Actor` for the gates and reads it for the foes, and `daAlldie_c` is `fopAc_ACTOR_e`, so no
    // gate ever carries an `Enemy`.
    foes: Query<&Actor, With<Enemy>>,
    mut gates: Query<(&mut Actor, &mut Alldie), Without<Enemy>>,
) {
    let mut rooms: Vec<i8> = Vec::new();
    for actor in &foes {
        if !rooms.contains(&actor.room) {
            rooms.push(actor.room);
        }
    }
    for (mut actor, mut gate) in &mut gates {
        let actor = &mut *actor;
        let gate = &mut *gate;
        if !gate.entered {
            gate.create_init(actor);
        }
        gate.execute(Alldie::room_enemy(actor, &rooms), &mut switches);
    }
}