use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::damage_fx::DamageFxQueue;
use super::damage_resolve::DamageFx;
use super::events::{SimEvent, SimEvents};
use super::flags::Switches;

/// TRANSLATION UNIT: `tww/src/d/actor/d_a_obj_gryw00.cpp` (366 lines, `Nonmatching == 0`).
/// This is NOT `d_a_obj_Ygush00.cpp`, which is a different actor: the Hyrule spring
/// (`Ygush00`, `Ygush01`, `Ygstp00`, `Ygush02` at `d_stage.cpp:1162-1165`) that plays
/// `JA_SE_OBJ_SPRING` (`d_a_obj_Ygush00.cpp:125`, `:131`) and cross-references
/// `fpcNm_Obj_Gryw00_e` only as a neighbour (`:128`). Those four stage strings are UNPORTED.
pub const GRYW_ACTORS: [&str; 1] = ["Gryw00"]; // <- d_stage.cpp:1141 (fpcNm_Obj_Gryw00_e)

/// `daObjGryw00_c` dispatches through a `ModeFunc` member pointer (`:50`), not an enum, so these
/// indices are an internal encoding of the five `modeFunc` assignments, not decomp values.
const PHASE_WAIT: u8 = 0; // <- d_a_obj_gryw00.cpp:171
const PHASE_SPREAD: u8 = 1; // <- d_a_obj_gryw00.cpp:242
const PHASE_RISE: u8 = 2; // <- d_a_obj_gryw00.cpp:248
const PHASE_LOOP_WAIT: u8 = 3; // <- d_a_obj_gryw00.cpp:257
const PHASE_HIGH_WATER: u8 = 4; // <- d_a_obj_gryw00.cpp:160

const SWITCH_NONE: u8 = 0xFF; // <- d_a_obj_gryw00.cpp:154, :225

const HIGH_WATER_LEVEL: f32 = 510.0; // <- d_a_obj_gryw00.cpp:17 (#define HIGH_WATER_LEVEL 510)
const LOW_WATER_LEVEL: f32 = -5.0; // <- d_a_obj_gryw00.cpp:172-173 (mWaterLv = -5; mWaterMaxLv = -5)
const SPREAD_WATER_LEVEL: f32 = 5.0; // <- d_a_obj_gryw00.cpp:237 (mWaterMaxLv = 5)
const WAVE_AMPLITUDE: f32 = -10.0; // <- d_a_obj_gryw00.cpp:16 (#define WAVE_AMPLITUDE -10)
const WAVE_START: f32 = 0.0; // <- d_a_obj_gryw00.cpp:175, :177 (mWaveAmplitude = 0)
const WATER_WAVE_GROWTH_SPEED: f32 = 0.10526316; // <- d_a_obj_gryw00.cpp:14
const WATER_RISE_SPEED: f32 = 1.6777408; // <- d_a_obj_gryw00.cpp:15
const SPREAD_PHASE_ANIM_LEN: f32 = 95.0; // <- d_a_obj_gryw00.cpp:12 (#define SPREAD_PHASE_ANIM_LEN 95)
const RISE_PHASE_ANIM_LEN: f32 = 299.0; // <- d_a_obj_gryw00.cpp:13 (#define RISE_PHASE_ANIM_LEN 299)
const GEYSER_SOUND_LEN: i32 = 310; // <- d_a_obj_gryw00.cpp:18 (#define GEYSER_SOUND_LEN 310)

/// `mBtk.init(..., J3DFrameCtrl::EMode_NONE, 0.0f, 0, 0x190, FALSE, 0)`
/// (`d_a_obj_gryw00.cpp:140`) - the sixth and seventh `mDoExt_btkAnm::init` arguments are
/// `i_start` and `i_end` (`m_Do_ext.h:59-60`), so the unsolved BTK runs frames 0 ..= 400.
const BTK_END_FRAME: f32 = 400.0; // <- d_a_obj_gryw00.cpp:140 (i_end = 0x190)
const BTK_START_RATE: f32 = 0.0; // <- d_a_obj_gryw00.cpp:140 (i_rate = 0.0f)
const BTK_PLAY_RATE: f32 = 1.0; // <- d_a_obj_gryw00.cpp:65 (i_rate = 1.0f), :231 (setPlaySpeed(1.0f))

/// `J3DFrameCtrl::checkPass` and `J3DFrameCtrl::play` are not in this checkout
/// (`include/JSystem/JAnimation.h` is absent), so `mDoExt_baseAnm::checkFrame`
/// (`m_Do_ext.h:32`) cannot be transcribed. `btk_check_frame` and the clamp in `btk_play` are a
/// stand-in for it. `BTK_FRAME_EPS` is NOT a retail constant - it only keeps the clamp from
/// re-triggering on the next tick.
const BTK_FRAME_EPS: f32 = 0.001;

/// Models `J3DFrameCtrl::STATE_STOP_E`, the flag `mDoExt_baseAnm::isStop` tests
/// (`m_Do_ext.h:40-42`). Not a decomp literal.
const BTK_STATE_STOP: u8 = 0x01;

const SE_W_OUT: u32 = 0x704B; // <- JAZelAudio_SE.h:2157 (JA_SE_OBJ_RDUN_MAE_W_OUT)
const SE_GEYSER: u32 = 0x704C; // <- JAZelAudio_SE.h:2158 (JA_SE_OBJ_RDUN_MAE_GEYSER)
const SE_W_UP: u32 = 0x704D; // <- JAZelAudio_SE.h:2159 (JA_SE_OBJ_RDUN_MAE_W_UP)

/// The four `dComIfGp_particle_set` ids `particle_set` (`d_a_obj_gryw00.cpp:122-127`) creates at
/// `home.pos`: `dPa_name::ID_AK_SN_RYUDANMAESPLASH00`, `ID_AK_SN_RYUDANMAESTEAM00`,
/// `ID_AK_SN_RYUDANMAESPLASH01` and `ID_AK_SN_RYUDANMAEHAMON00`. The last two are the ones the
/// class keeps in `mpEmitters[0]` / `mpEmitters[1]`; the first two are fire-and-forget.
const ID_RYUDANMAESPLASH00: u16 = 0x8295; // <- d/d_particle_name.h:783
const ID_RYUDANMAESPLASH01: u16 = 0x8296; // <- d/d_particle_name.h:784
const ID_RYUDANMAESTEAM00: u16 = 0x8297; // <- d/d_particle_name.h:785
const ID_RYUDANMAEHAMON00: u16 = 0x8298; // <- d/d_particle_name.h:786

const SWSAVE_SHIFT: u32 = 0x00; // <- d_a_obj_gryw00.h:16 (PRM_SWSAVE_S = 0x00)
const SWSAVE_WIDTH: u32 = 0x08; // <- d_a_obj_gryw00.h:15 (PRM_SWSAVE_W = 0x08)
const SWSAVE_MASK: u32 = (1 << SWSAVE_WIDTH) - 1;

/// `fopAc_CULLBOX_CUSTOM_e` (`d_a_obj_gryw00.cpp:365`) with no `fopAcM_setCullSizeBox` call
/// anywhere in the translation unit, so retail has NO cull box for this actor and neither does
/// this module.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Gryw {
    switch_no: u8,
    mode: u8,
    water_lv: f32,
    water_max: f32,
    water_inc: f32,
    wave: f32,
    wave_max: f32,
    wave_inc: f32,
    should_play_se: bool,
    geyser_se: i32,
    btk_end: f32,
    btk_rate: f32,
    btk_frame: f32,
    btk_state: u8,
}

impl Gryw {
    pub fn water_lv(&self) -> f32 {
        self.water_lv
    }

    pub fn wave(&self) -> f32 {
        self.wave
    }
}

impl Gryw {
    /// `Create` (`d_a_obj_gryw00.cpp:151-190`): `param_get_swSave` reads bits 0..8, and a switch
    /// that is already set at load time takes the "puzzle already solved" branch - static
    /// high-water BTK (`:156`), `high_water_level_act_proc` (`:160`) and the water/wave levels
    /// pinned high (`:162-166`). Otherwise the actor waits hidden (`:169-177`).
    pub fn spawn(parameters: u32, switches: &Switches) -> Self {
        let switch_no = ((parameters >> SWSAVE_SHIFT) & SWSAVE_MASK) as u8;
        let solved = switch_no != SWITCH_NONE && switches.test(switch_no);
        Self {
            switch_no,
            mode: if solved { PHASE_HIGH_WATER } else { PHASE_WAIT },
            water_lv: if solved {
                HIGH_WATER_LEVEL
            } else {
                LOW_WATER_LEVEL
            },
            water_max: if solved {
                HIGH_WATER_LEVEL
            } else {
                LOW_WATER_LEVEL
            },
            water_inc: 0.0,
            wave: if solved { WAVE_AMPLITUDE } else { WAVE_START },
            wave_max: if solved { WAVE_AMPLITUDE } else { WAVE_START },
            wave_inc: 0.0,
            should_play_se: false,
            geyser_se: 0,
            btk_end: BTK_END_FRAME,
            btk_rate: if solved {
                BTK_PLAY_RATE
            } else {
                BTK_START_RATE
            },
            btk_frame: 0.0,
            btk_state: 0,
        }
    }

    fn btk_play(&mut self) {
        self.btk_state = 0;
        self.btk_frame += self.btk_rate;
        if self.btk_frame < 0.0 {
            self.btk_frame = 0.0;
            self.btk_rate = 0.0;
            self.btk_state |= BTK_STATE_STOP;
        }
        if self.btk_frame >= self.btk_end {
            self.btk_frame = self.btk_end - BTK_FRAME_EPS;
            self.btk_rate = 0.0;
            self.btk_state |= BTK_STATE_STOP;
        }
    }

    fn btk_check_frame(&self, pass: f32) -> bool {
        let cur = self.btk_frame;
        let mut next = cur + self.btk_rate;
        if next < 0.0 {
            next = 0.0;
        }
        if next >= self.btk_end {
            next = self.btk_end - BTK_FRAME_EPS;
        }
        if cur <= next {
            cur <= pass && pass < next
        } else {
            next <= pass && pass < cur
        }
    }

    /// `mDoExt_baseAnm::isStop` (`m_Do_ext.h:40-42`) is
    /// `checkState(STATE_STOP_E) || getRate() == 0.0f`.
    fn btk_stop(&self) -> bool {
        self.btk_state & BTK_STATE_STOP != 0 || self.btk_rate == 0.0
    }

    /// `set_se` (`d_a_obj_gryw00.cpp:107-126`). Note retail clears `mShouldPlaySe` at
    /// `:111-113` but does NOT return, so the two remaining sounds still fire on that tick.
    fn set_se(&mut self, events: &mut SimEvents) {
        if !self.should_play_se {
            return;
        }
        if self.water_lv >= HIGH_WATER_LEVEL {
            self.should_play_se = false;
        }
        if self.geyser_se > 0 {
            self.geyser_se -= 1;
            events.list.push(SimEvent::Se(SE_GEYSER));
        }
        events.list.push(SimEvent::Se(SE_W_OUT));
        events.list.push(SimEvent::Se(SE_W_UP));
    }

    /// `switch_wait_act_proc` (`:224-243`). The BCK write (`:232`) is renderer-only: `mBck` drives
    /// the model's joint animation and nothing in the translation unit reads a frame or a rate
    /// back off it, so the port's `j3d` BCK reader has no consumer here. The four emitters
    /// (`:238`) go through [`particle_set`](Self::particle_set).
    fn switch_wait_act_proc(&mut self, pos: Vec3, switches: &Switches, fx: &mut DamageFxQueue) {
        if self.switch_no == SWITCH_NONE {
            return;
        }
        if !switches.test(self.switch_no) {
            return;
        }
        self.btk_rate = BTK_PLAY_RATE;
        self.wave_inc = -WATER_WAVE_GROWTH_SPEED;
        self.wave_max = WAVE_AMPLITUDE;
        self.water_inc = WATER_WAVE_GROWTH_SPEED;
        self.water_max = SPREAD_WATER_LEVEL;
        self.particle_set(pos, fx); // <- :238
        self.geyser_se = GEYSER_SOUND_LEN;
        self.should_play_se = true;
        self.mode = PHASE_SPREAD;
    }

    /// `particle_set` (`:122-127`): four `dComIfGp_particle_set(id, &homePos, NULL, NULL, 0xff,
    /// NULL, -1, NULL, &mColorK0, NULL)` calls, all at `this->home.pos`, with the third and fourth
    /// ids kept in `mpEmitters[0]` / `mpEmitters[1]`. The port's actor-facing particle channel is
    /// [`DamageFxQueue`](super::damage_fx::DamageFxQueue) - the one `fgmahou.rs` and `shand.rs`
    /// push their own `dComIfGp_particle_set` calls through, and the one `App::pump_damage_fx`
    /// drains into the renderer's emitter pool - so the four requests are queued at the position
    /// the reference creates them.
    ///
    /// `particle_move` (`:129-138`) then follows the two held emitters - `setGlobalTranslation` at
    /// `home.pos + (0, mWaterLv, 0)` - and `particle_delete` (`:140-150`) retires them. The queue
    /// carries one-shot requests and no emitter handle, so neither call has a destination: the two
    /// held effects are placed once, where retail creates them.
    fn particle_set(&self, pos: Vec3, fx: &mut DamageFxQueue) {
        for id in [
            ID_RYUDANMAESPLASH00,
            ID_RYUDANMAESTEAM00,
            ID_RYUDANMAESPLASH01,
            ID_RYUDANMAEHAMON00,
        ] {
            fx.push(DamageFx::Particle {
                id,
                pos,
                angle: None,
                scale: None,
            });
        }
    }

    /// `spread_water_face_act_proc` (`:246-250`).
    fn spread_water_face_act_proc(&mut self) {
        if self.btk_check_frame(SPREAD_PHASE_ANIM_LEN) {
            self.mode = PHASE_RISE;
        }
    }

    /// `water_level_move_wait_act_proc` (`:253-261`). The BCK hold (`:258-259`,
    /// `mBck.setPlaySpeed(0); mBck.setFrame(RISE_PHASE_ANIM_LEN);`) is renderer-only, as the BCK
    /// write in [`switch_wait_act_proc`](Self::switch_wait_act_proc) is: only the model draw reads
    /// a BCK frame back.
    fn water_level_move_wait_act_proc(&mut self) {
        if self.btk_check_frame(RISE_PHASE_ANIM_LEN) {
            self.water_inc = WATER_RISE_SPEED;
            self.water_max = HIGH_WATER_LEVEL;
            self.mode = PHASE_LOOP_WAIT;
        }
    }

    /// `anime_loop_start_wait_act_proc` (`:264-270`).
    fn anime_loop_start_wait_act_proc(&mut self) {
        if self.btk_stop() {
            self.mode = PHASE_HIGH_WATER;
        }
    }

    /// `high_water_level_act_proc` (`:272-273`) is empty.
    fn high_water_level_act_proc(&mut self) {}

    fn act_proc(&mut self, pos: Vec3, switches: &Switches, fx: &mut DamageFxQueue) {
        match self.mode {
            PHASE_WAIT => self.switch_wait_act_proc(pos, switches, fx),
            PHASE_SPREAD => self.spread_water_face_act_proc(),
            PHASE_RISE => self.water_level_move_wait_act_proc(),
            PHASE_LOOP_WAIT => self.anime_loop_start_wait_act_proc(),
            PHASE_HIGH_WATER => self.high_water_level_act_proc(),
            _ => {}
        }
    }

    /// `Execute` (`:276-304`), minus the renderer-only matrix writes (`:288-295`) and the BCK play
    /// (`:300`). `particle_move` (`:297`) is `particle_set`'s note. Retail order is water,
    /// `set_se`, BTK, mode.
    fn execute(
        &mut self,
        pos: Vec3,
        switches: &Switches,
        events: &mut SimEvents,
        fx: &mut DamageFxQueue,
    ) {
        self.wave += self.wave_inc;
        if self.wave < self.wave_max {
            self.wave = self.wave_max;
            self.wave_inc = 0.0;
        }
        self.water_lv += self.water_inc;
        if self.water_lv > self.water_max {
            self.water_lv = self.water_max;
            self.water_inc = 0.0;
        }
        self.set_se(events);
        self.btk_play();
        self.act_proc(pos, switches, fx);
    }
}

pub fn is_gryw(name: &str) -> bool {
    GRYW_ACTORS.contains(&name)
}

pub fn control(
    switches: Res<Switches>,
    mut events: ResMut<SimEvents>,
    mut fx: ResMut<DamageFxQueue>,
    mut pools: Query<(&Actor, &mut Gryw)>,
) {
    for (actor, mut pool) in &mut pools {
        pool.execute(actor.pos, &switches, &mut events, &mut fx);
    }
}
