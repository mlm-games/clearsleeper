use bevy_ecs::prelude::*;
use glam::Vec2;
use glam::Vec3;
use glam::Vec3Swizzles;

use super::actor::Actor;
use super::bombflower::Blasts;
use super::event_manager::EventManager;
use super::events::{SimEvent, SimEvents};
use super::intent::{DoIntent, MoveIntent};
use super::player::Player;
use super::sea::Sea;
use super::stream::{cos_s, sin_s};
use super::vibration::Vibration;
use super::walk::{add_calc_angle_s, from_s16};

/// `d_a_canon.cpp` is the cannon-minigame cannon, process `fpcNm_Canon_e`, whose OBJNAME row is
/// `"Canon"`. `src/sim/canon.rs` is a DIFFERENT translation unit (`d_a_obj_canon.cpp`, the stage
/// gun emplacement) whose row is `"Ocanon"` (`d_stage.cpp:819`). The rows sit forty-eight lines
/// apart (`d_stage.cpp:819`, `:867`) and no other module claims `"Canon"`.
pub const CANON2_ACTORS: [&str; 1] = ["Canon"]; // <- d_stage.cpp:867

/// `setProcess(&daCanon_c::wait_proc)`.
pub const PROC_WAIT: u8 = 0; // <- d_a_canon.cpp:247
/// `setProcess(&daCanon_c::game_start_proc)`.
pub const PROC_GAME_START: u8 = 1; // <- d_a_canon.cpp:294
/// `setProcess(&daCanon_c::game_proc)`.
pub const PROC_GAME: u8 = 2; // <- d_a_canon.cpp:306
/// `setProcess(&daCanon_c::fire_proc)`.
pub const PROC_FIRE: u8 = 3; // <- d_a_canon.cpp:336
/// `setProcess(&daCanon_c::pause_proc)`.
pub const PROC_PAUSE: u8 = 4; // <- d_a_canon.cpp:396
/// `setProcess(&daCanon_c::end_proc)`.
pub const PROC_END: u8 = 5; // <- d_a_canon.cpp:414

/// `static char* cut_name_tbl[] = {"Canon_game"}` in `wait_proc` (`:252`) and `end_proc` (`:427`).
pub const CUT_NAME: &str = "Canon_game"; // <- d_a_canon.cpp:252
/// `dComIfGp_evmng_getMyStaffId("Canon")` in `wait_proc` (`:256`) and `end_proc` (`:435`).
pub const STAFF_NAME: &str = "Canon"; // <- d_a_canon.cpp:256
/// `getMyActIdx(staff, cut_name_tbl, 1, TRUE, 0)` - `force` bypasses the cached action index.
pub const ACT_FORCE: bool = true; // <- d_a_canon.cpp:258
/// `getMyActIdx(..., nameType = 0)` selects plain `strcmp` rather than the prefix compare.
pub const ACT_NAME_TYPE: i32 = 0; // <- d_a_canon.cpp:258
/// `getMyStaffId` returns `-1` when this actor owns no staff.
pub const NO_STAFF: i32 = -1; // <- d_a_canon.cpp:257
/// `getMyStaffId("Canon")` uses the default `staffType` argument.
pub const STAFF_TYPE: i32 = 0; // <- d_com_inf_game.h:3653
/// `ARRAY_SSIZE(cut_name_tbl)` / `ARRAY_SIZE(cut_name_tbl)` is the one-entry table length.
pub const CUT_COUNT: usize = 1; // <- d_a_canon.cpp:258, :438

pub const SE_CANNON_MOVE: u32 = 0x2051; // <- d_a_canon.cpp:488
pub const SE_CANNON_FIRE: u32 = 0x2852; // <- d_a_canon.cpp:322
pub const SE_CANNON_FLY: u32 = 0x381F; // <- d_a_canon.cpp:361
pub const SE_BOMB_EXPLODE: u32 = 0x6901; // <- d_a_canon.cpp:240
pub const SE_PLAYSPOT_START: u32 = 0x08A8; // <- d_a_canon.cpp:291
pub const SE_PLAYSPOT_DESTROY: u32 = 0x08AA; // <- d_a_canon.cpp:389
pub const SE_PLAYSPOT_END_F: u32 = 0x08AC; // <- d_a_canon.cpp:393
/// `fire_proc`: `m656-- < 0` - the countdown tests before it decrements.
pub const COUNT_UNDER: i16 = 0; // <- d_a_canon.cpp:299, :401

/// `getBulEndPos`: `((home.angle.x + 0x1000) - shape_angle.x) / 0x1000` and
/// `fVar2 * 1500.0f * 7.0f + 10500.0f`.
pub const PITCH_DIV: f32 = 4096.0; // <- d_a_canon.cpp:130
pub const ARC_SPACING: f32 = 1500.0; // <- d_a_canon.cpp:131
pub const ARC_SEGMENTS: f32 = 7.0; // <- d_a_canon.cpp:131
pub const ARC_BASE: f32 = 10500.0; // <- d_a_canon.cpp:131
/// `fire_proc_init`: `m652 = tmp * 8.0f * 7.0f + 64.0f`.
pub const FLIGHT_SCALE: f32 = 8.0; // <- d_a_canon.cpp:331
pub const FLIGHT_TURNS: f32 = 7.0; // <- d_a_canon.cpp:331
pub const FLIGHT_MIN: f32 = 64.0; // <- d_a_canon.cpp:331
/// `game_proc`: the muzzle sits `200.0f` out along the barrel.
pub const MUZZLE: f32 = 200.0; // <- d_a_canon.cpp:315
/// `wait_proc`: the player is stood `200.0f` behind the cannon.
pub const PLAYER_BACK: f32 = 200.0; // <- d_a_canon.cpp:264

/// `game_start_proc_init`: `m656 = 60`.
pub const START_DELAY: i16 = 60; // <- d_a_canon.cpp:286
/// `pause_proc_init`: `m656 = 30`.
pub const PAUSE_DELAY: i16 = 30; // <- d_a_canon.cpp:371
/// `end_proc_init`: `m656 = 60`.
pub const END_DELAY: i16 = 60; // <- d_a_canon.cpp:412
/// `end_proc` breaks the remaining targets when the countdown reaches 30.
pub const BREAK_FRAME: i16 = 30; // <- d_a_canon.cpp:430
/// `wait_proc`: `m65C = 10` shells per round.
pub const ROUNDS: i16 = 10; // <- d_a_canon.cpp:263
/// `createTargetObj` loops while `i != 0` from `i = 5`; `m65A == 5` ends the round.
pub const TARGET_COUNT: i16 = 5; // <- d_a_canon.cpp:174, :380

/// `_create` parks the barrel at `0x2AAA`; `game_start_proc_init` re-parks it.
pub const PITCH_PARKED: i16 = 0x2AAA; // <- d_a_canon.cpp:609, :288
/// `_create` overwrites the placement pitch: `home.angle.x = 0x2000` (`fopAc_ac_c::home` is the
/// placement `actor_place`, `f_op_actor.h:287`), so the pitch band `PadMove` clamps to always
/// starts from `0x2000` no matter what the stage row carries.
pub const PITCH_HOME: i16 = 0x2000; // <- d_a_canon.cpp:610
/// `PadMove` clamps the yaw to `home.angle.y + -0x1200` and `home.angle.y + 0x1200`.
pub const YAW_LIMIT: i16 = 0x1200; // <- d_a_canon.cpp:458, :464
/// `PadMove` clamps the pitch to `home.angle.x + 0x1000`.
pub const PITCH_UP: i16 = 0x1000; // <- d_a_canon.cpp:472
/// `cLib_addCalcAngleS(&shape_angle.y, target1, 4, 0x100, 0x10)`.
pub const AIM_SCALE: i16 = 4; // <- d_a_canon.cpp:484
pub const AIM_MAX: i16 = 0x100; // <- d_a_canon.cpp:484
pub const AIM_MIN: i16 = 0x10; // <- d_a_canon.cpp:484
/// `fire_proc`: `s16 idx = 0x4000 - shape_angle.x`.
pub const HALF_TURN: i16 = 0x4000; // <- d_a_canon.cpp:352
/// `f32 tmp1 = sp54.absXZ() * 0.25f * (cM_ssin(idx) / cM_scos(idx));`.
pub const BALLISTIC_SCALE: f32 = 0.25; // <- d_a_canon.cpp:353
/// `f32 tmp2 = 1.0f - (fVar4 - 0.5f) * (fVar4 - 0.5f) * 4.0f;`.
pub const ARC_SHAPE: f32 = 4.0; // <- d_a_canon.cpp:349
/// `f32 fVar4 = (f32)m652 / m654;` - the shell walks the range in `m654` equal steps.
pub const FLIGHT_DENOM: f32 = 1.0; // <- d_a_canon.cpp:348

/// `PadMove` dead-zones both sticks at `0.5f`.
pub const STICK_GATE: f32 = 0.5; // <- d_a_canon.cpp:456
/// `target1 += (s16)((0.5f - stickX) * 0x400);`
pub const YAW_GAIN: f32 = 1024.0; // <- d_a_canon.cpp:457
/// `target2 += (s16)((stickY - 0.5f) * 512.0f);`
pub const PITCH_GAIN: f32 = 512.0; // <- d_a_canon.cpp:471
/// `dComIfGp_getVibration().StartShock(7, -0x21, cXyz(0.0f, 1.0f, 0.0f))` in `makeFireEffect`.
pub const SHOCK_PATTERN: i32 = 7; // <- d_a_canon.cpp:241
pub const SHOCK_FLAGS: i32 = -0x21; // <- d_a_canon.cpp:241
pub const SHOCK_COORD: Vec3 = Vec3::new(0.0, 1.0, 0.0); // <- d_a_canon.cpp:241

/// `fopAc_CULLBOX_0_e` resolves to `cXyz(-40,0,-40) .. cXyz(40,125,40)` at
/// `f_op_actor_mng.cpp:558`; `g_profile_Canon` selects it at `d_a_canon.cpp:720`.
pub const CULL_BASE: f32 = 0.0; // <- f_op_actor_mng.cpp:558
pub const CULL_TOP: f32 = 125.0; // <- f_op_actor_mng.cpp:558
pub const CULL_HALF: f32 = 40.0; // <- f_op_actor_mng.cpp:558

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Canon2 {
    /// The process pointer `m290` selects; the six `setProcess` targets are the only values.
    pub process: u8,
    /// `shape_angle`, the barrel pitch/yaw the actor turns under `PadMove`.
    pub shape_angle: [i16; 2],
    /// `m656`, the countdown shared by the start, pause and end processes.
    pub timer: i16,
    /// `m65C`, shells remaining in this round; `fire_proc_init` decrements it.
    pub rounds: i16,
    /// `m65A`, targets destroyed this round. Never increments here: `createTargetObj` is omitted.
    pub hits: i16,
    /// `m614`, the muzzle the shell leaves from.
    pub muzzle: Vec3,
    /// `m620`, `getBulEndPos(shape_angle.y, shape_angle.x)`, the far end of the parabola.
    pub land: Vec3,
    /// `m62C`, the shell's live position this tick.
    pub shell: Vec3,
    /// `m652`, the remaining flight steps.
    pub flight: i16,
    /// `m654`, the step count the flight is divided by.
    pub flight_max: i16,
}

impl Canon2 {
    pub fn spawn() -> Self {
        Self {
            process: PROC_WAIT,
            shape_angle: [PITCH_PARKED, 0],
            timer: 0,
            rounds: 0,
            hits: 0,
            muzzle: Vec3::ZERO,
            land: Vec3::ZERO,
            shell: Vec3::ZERO,
            flight: 0,
            flight_max: FLIGHT_MIN as i16,
        }
    }

    /// `_draw` renders `mpModel1` on every tick, so the actor is always present.
    pub fn drawn(&self) -> bool {
        true
    }

    /// The `fopAc_CULLBOX_0_e` box as `(base, height, half)`, matching `canon::hull`.
    pub fn hull(&self) -> (f32, f32, f32) {
        (CULL_BASE, CULL_TOP - CULL_BASE, CULL_HALF)
    }

    /// `getBulEndPos(short arg1, short arg2)` called as `(shape_angle.y, shape_angle.x)`.
    fn get_bul_end_pos(&self, pos: Vec3, sea: Option<&Sea>) -> Vec3 {
        let span = (f32::from(PITCH_HOME) + PITCH_DIV - f32::from(self.shape_angle[0])) / PITCH_DIV;
        let reach = span * ARC_SPACING * ARC_SEGMENTS + ARC_BASE;
        let yaw = self.shape_angle[1];
        let mut at = Vec3::new(pos.x + reach * sin_s(yaw), 0.0, pos.z + reach * cos_s(yaw));
        at.y = match sea {
            Some(sea) => sea.height_at(at.x, at.z),
            None => 0.0,
        };
        at
    }

    /// `game_proc` builds `sp08` from the barrel direction, `200.0f` out.
    fn muzzle_of(&self, pos: Vec3) -> Vec3 {
        let pitch = self.shape_angle[0];
        let yaw = self.shape_angle[1];
        let reach = sin_s(pitch) * MUZZLE;
        Vec3::new(
            pos.x + reach * sin_s(yaw),
            pos.y + cos_s(pitch) * MUZZLE,
            pos.z + reach * cos_s(yaw),
        )
    }

    /// The seat `wait_proc` moves the player to: `200.0f` behind the cannon on the home yaw.
    fn seat(&self, pos: Vec3, home_yaw: i16) -> Vec3 {
        Vec3::new(
            pos.x - sin_s(home_yaw) * PLAYER_BACK,
            pos.y,
            pos.z - cos_s(home_yaw) * PLAYER_BACK,
        )
    }

    /// `PadMove`: the left stick walks the yaw and the right stick the pitch, both clamped to a
    /// `+-0x1200` yaw band and a `[home.angle.x, home.angle.x + 0x1000]` pitch band, then eased
    /// by `cLib_addCalcAngleS(..., 4, 0x100, 0x10)`. Returns the retail `sVar2 != 0 || sVar4 != 0`.
    fn pad_move(&mut self, home_yaw: i16, stick: Vec2) -> bool {
        let mut yaw_goal = self.shape_angle[1];
        let mut pitch_goal = self.shape_angle[0];
        if stick.x > STICK_GATE {
            yaw_goal = yaw_goal.wrapping_add(units((STICK_GATE - stick.x) * YAW_GAIN));
            let low = home_yaw.wrapping_add(-YAW_LIMIT);
            if self.shape_angle[1] >= low && yaw_goal < low {
                yaw_goal = low;
            }
        } else if stick.x < -STICK_GATE {
            yaw_goal = yaw_goal.wrapping_add(units((-STICK_GATE - stick.x) * YAW_GAIN));
            let high = home_yaw.wrapping_add(YAW_LIMIT);
            if self.shape_angle[1] <= high && yaw_goal > high {
                yaw_goal = high;
            }
        }
        if stick.y > STICK_GATE {
            pitch_goal = pitch_goal.wrapping_add(units((stick.y - STICK_GATE) * PITCH_GAIN));
            let high = PITCH_HOME.wrapping_add(PITCH_UP);
            if self.shape_angle[0] <= high && pitch_goal > high {
                pitch_goal = high;
            }
        } else if stick.y < -STICK_GATE {
            pitch_goal = pitch_goal.wrapping_add(units((stick.y + STICK_GATE) * PITCH_GAIN));
            let low = PITCH_HOME;
            if self.shape_angle[0] >= low && pitch_goal < low {
                pitch_goal = low;
            }
        }
        let yaw = add_calc_angle_s(self.shape_angle[1], yaw_goal, AIM_SCALE, AIM_MAX, AIM_MIN);
        let pitch = add_calc_angle_s(self.shape_angle[0], pitch_goal, AIM_SCALE, AIM_MAX, AIM_MIN);
        self.shape_angle[1] = yaw;
        self.shape_angle[0] = pitch;
        yaw != yaw_goal || pitch != pitch_goal
    }

    /// `game_proc` on `CPad_CHECK_TRIG_A(0)`: `makeFireEffect`, `m614 = sp08`, then `fire_proc_init`.
    fn fire(&mut self, pos: Vec3, events: &mut SimEvents) {
        self.muzzle = self.muzzle_of(pos);
        events.list.push(SimEvent::Se(SE_CANNON_FIRE));
        events.list.push(SimEvent::Se(SE_BOMB_EXPLODE));
        let span = (f32::from(PITCH_HOME) + PITCH_DIV - f32::from(self.shape_angle[0])) / PITCH_DIV;
        self.flight = (span * FLIGHT_SCALE * FLIGHT_TURNS + FLIGHT_MIN) as i16;
        self.flight_max = self.flight;
        self.rounds -= 1;
        self.process = PROC_FIRE;
    }

    /// `fire_proc`: `m652-- != 0` walks `m652 / m654` from the muzzle to `m620`, bow-shaped by
    /// `1 - (t - 0.5)^2 * 4` and lifted by the barrel tangent scaled on the chord length. Each
    /// tick the live shell centre is published as the `AT_TYPE_BOMB` origin, which is what
    /// `mCyl2.SetC(m62C); dComIfG_Ccsp()->Set(&mCyl2)` (`:358-359`) registers in retail.
    fn fly(&mut self, pos: Vec3, sea: Option<&Sea>, blasts: &mut Blasts, events: &mut SimEvents) {
        self.land = self.get_bul_end_pos(pos, sea);
        let running = self.flight != COUNT_UNDER;
        if running {
            self.flight -= 1;
            let ratio = f32::from(self.flight) * FLIGHT_DENOM / f32::from(self.flight_max);
            let bow = 1.0 - (ratio - 0.5) * (ratio - 0.5) * ARC_SHAPE;
            let chord = self.land - self.muzzle;
            let tilt = HALF_TURN.wrapping_sub(self.shape_angle[0]);
            let lift = chord.xz().length() * BALLISTIC_SCALE * (sin_s(tilt) / cos_s(tilt));
            self.shell = self.muzzle * ratio + self.land * (1.0 - ratio);
            self.shell.y += lift * bow;
            blasts.push(self.shell);
            events.list.push(SimEvent::Se(SE_CANNON_FLY));
        } else {
            self.timer = PAUSE_DELAY;
            self.process = PROC_PAUSE;
        }
    }

    /// `pause_proc`: `m656-- < 0` then either straight back into `game_proc` or `end_proc`.
    fn pause(&mut self) {
        if self.timer < COUNT_UNDER {
            if self.rounds > 0 && self.hits < TARGET_COUNT {
                self.process = PROC_GAME;
            } else {
                self.timer = END_DELAY;
                self.process = PROC_END;
            }
            return;
        }
        self.timer -= 1;
    }

    /// `game_start_proc`: `m656-- < 0` hands over to `game_proc_init`.
    fn game_start(&mut self) {
        if self.timer < COUNT_UNDER {
            self.process = PROC_GAME;
            return;
        }
        self.timer -= 1;
    }

    /// `end_proc`: counts `m656` down, breaking the targets at 30, then retires the staff cut.
    fn end_game(&mut self, manager: &mut EventManager, events: &mut SimEvents) {
        if self.timer > 0 {
            if self.timer == BREAK_FRAME {
                events.list.push(SimEvent::Se(SE_PLAYSPOT_DESTROY));
            }
            self.timer -= 1;
            return;
        }
        let staff = manager.get_my_staff_id(STAFF_NAME, None, STAFF_TYPE);
        if staff == NO_STAFF {
            self.process = PROC_WAIT;
            return;
        }
        if manager.get_my_act_idx(staff, &[CUT_NAME], ACT_FORCE, ACT_NAME_TYPE) != -1 {
            manager.cut_end(staff);
            events.list.push(SimEvent::Se(SE_PLAYSPOT_END_F));
        }
        self.process = PROC_WAIT;
    }

    /// `wait_proc`: fire when the live cut is `"Canon_game"`, otherwise release the cut.
    fn wait(
        &mut self,
        manager: &mut EventManager,
        pos: Vec3,
        home_yaw: i16,
        player: Option<&mut Player>,
        events: &mut SimEvents,
    ) {
        let staff = manager.get_my_staff_id(STAFF_NAME, None, STAFF_TYPE);
        if staff == NO_STAFF {
            return;
        }
        let action =
            manager.get_my_act_idx(staff, &[CUT_NAME][..CUT_COUNT], ACT_FORCE, ACT_NAME_TYPE);
        if action == -1 {
            manager.cut_end(staff);
            return;
        }
        self.hits = 0;
        self.rounds = ROUNDS;
        self.timer = START_DELAY;
        self.shape_angle[1] = home_yaw;
        self.shape_angle[0] = PITCH_PARKED;
        self.process = PROC_GAME_START;
        events.list.push(SimEvent::Se(SE_PLAYSPOT_START));
        if let Some(player) = player {
            player.pos = self.seat(pos, home_yaw);
            player.yaw = from_s16(home_yaw);
        }
    }

    fn execute(
        &mut self,
        pos: Vec3,
        home_yaw: i16,
        stick: Vec2,
        trig_a: bool,
        manager: &mut EventManager,
        player: Option<&mut Player>,
        sea: Option<&Sea>,
        blasts: &mut Blasts,
        events: &mut SimEvents,
        vibration: &mut Vibration,
    ) {
        match self.process {
            PROC_WAIT => self.wait(manager, pos, home_yaw, player, events),
            PROC_GAME_START => self.game_start(),
            PROC_GAME => {
                if self.pad_move(home_yaw, stick) {
                    events.list.push(SimEvent::Se(SE_CANNON_MOVE));
                }
                if trig_a {
                    self.fire(pos, events);
                    let _ = vibration.start_shock(SHOCK_PATTERN, SHOCK_FLAGS, SHOCK_COORD);
                }
            }
            PROC_FIRE => self.fly(pos, sea, blasts, events),
            PROC_PAUSE => self.pause(),
            PROC_END => self.end_game(manager, events),
            _ => {}
        }
    }
}

fn units(value: f32) -> i16 {
    value as i16
}

pub fn is_canon2(name: &str) -> bool {
    CANON2_ACTORS.contains(&name)
}

pub fn control(
    intent: Res<MoveIntent>,
    trig_a: Res<DoIntent>,
    mut manager: ResMut<EventManager>,
    sea: Option<Res<Sea>>,
    mut players: Query<&mut Player>,
    mut blasts: ResMut<Blasts>,
    mut events: ResMut<SimEvents>,
    mut vibration: ResMut<Vibration>,
    mut canons: Query<(&Actor, &mut Canon2)>,
) {
    let sea_ref = sea.as_deref();
    let stick = intent.axis;
    let fire = trig_a.0;
    let mut player = players.iter_mut().next();
    for (actor, mut canon) in &mut canons {
        canon.execute(
            actor.pos,
            actor.angle[1],
            stick,
            fire,
            &mut manager,
            player.as_deref_mut(),
            sea_ref,
            &mut blasts,
            &mut events,
            &mut vibration,
        );
    }
}
