use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::events::{SimEvent, SimEvents};
use super::flags::Switches;
use super::health::Health;
use super::player::Player;

/// Stage object name (`tww/src/d/d_stage.cpp:793`). `ssk_class` is `fpcNm_SSK_e`, listed
/// `0x0003`, draw prio `fpcDwPi_SSK_e`, status `fopAcStts_UNK40000_e`, cull type
/// `fopAc_CULLBOX_CUSTOM_e` (`d_a_ssk.cpp:435-450`). Its model archive is `"Ssk"`
/// (`d_a_ssk.cpp:366`), which differs from the stage string.
pub const SSTURU_ACTORS: [&str; 1] = ["s_turu"];

/// `fopAcM_SetMin` / `fopAcM_SetMax` (`d_a_ssk.cpp:393-394`).
const CULL_MIN: [f32; 3] = [-800.0, -200.0, -400.0];
const CULL_MAX: [f32; 3] = [500.0, 1000.0, 1000.0];

const VARIANT_SHIFT: u32 = 0x00;
const SIGHT_SHIFT: u32 = 0x08;
const SWITCH_SHIFT: u32 = 0x18;
const BYTE_MASK: u32 = 0xFF;

const NO_SWITCH: u8 = 0xFF;
const ALWAYS_UP: u8 = 0xFF;
const SIGHT_DEFAULT: f32 = 400.0;
const SIGHT_SCALE: f32 = 10.0;
const LEAVE_GAP: f32 = 100.0;

const GROW_TARGET: f32 = 1.0;
const GROW_SCALE: f32 = 0.5;
const GROW_STEP: f32 = 0.3;
const GROW_DONE: f32 = 0.9;
const SINK_SCALE: f32 = 0.5;
const SINK_STEP: f32 = 0.3;
const SINK_DONE: f32 = 0.1;
const LID_SCALE: f32 = 0.2;
const LID_STEP: f32 = 0.5;
const LID_DONE: f32 = 0.1;

/// `mCyl.SetH(300.0f)` / `mCyl.SetR(80.0f)` (`d_a_ssk.cpp:246-247`) against the player's
/// collider cylinder (`d_a_player_main_data.inc:68-69`).
const CYL_HEIGHT: f32 = 300.0;
const CYL_RADIUS: f32 = 80.0;
const PLAYER_RADIUS: f32 = 30.0;
const PLAYER_HEIGHT: f32 = 125.0;

/// `SrcObjAt Atp` of `body_co_cyl` (`d_a_ssk.cpp:338`); `cCcS::CalcTgPlusDmg` passes it to
/// `cCcD_Stts::PlusDmg` (`d_cc_s.cpp:126-133`).
const CONTACT_DAMAGE: u8 = 1;

const SE_JAMA_SHOKU_OUT: u32 = 0x694B;
const SE_JAMA_SHOKU_IN: u32 = 0x694D;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Wait,
    LiftLid,
    Rise,
    Sink,
    Up,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct SsTuru {
    pub variant: u8,
    pub switch_no: u8,
    pub mode: Mode,
    pub at_live: bool,
    pub struck: bool,
    pub lid: f32,
    pub rise: f32,
    pub sight: f32,
    pub refused: bool,
}

fn add_calc2(value: &mut f32, target: f32, scale: f32, max_step: f32) {
    if *value != target {
        let mut step = scale * (target - *value);
        if step > max_step {
            step = max_step;
        } else if step < -max_step {
            step = -max_step;
        }
        *value += step;
    }
}

/// `cLib_addCalc0` (`c_lib.cpp:69-77`): `*value -= clamp(*value * scale, -max, max)`.
fn add_calc0(value: &mut f32, scale: f32, max_step: f32) {
    let mut step = *value * scale;
    if step > max_step {
        step = max_step;
    } else if step < -max_step {
        step = -max_step;
    }
    *value -= step;
}

/// `cCcD_CylAttr::CalcAabBox` (`tww/src/SSystem/SComponent/c_cc_d.cpp:439`) spans a cylinder
/// from `GetCP()->y` to `GetCP()->y + GetH()`.
fn cyl_cross(base: Vec3, at: Vec3, radius: f32, height: f32, other: f32, other_h: f32) -> bool {
    let dx = at.x - base.x;
    let dz = at.z - base.z;
    let span = radius + other;
    if span * span < dx * dx + dz * dz {
        return false;
    }
    base.y < at.y + other_h && at.y < base.y + height
}

impl SsTuru {
    pub fn spawn(parameters: u32, switches: &Switches) -> Self {
        let variant = ((parameters >> VARIANT_SHIFT) & BYTE_MASK) as u8;
        let mut sight_band = ((parameters >> SIGHT_SHIFT) & BYTE_MASK) as u8;
        if sight_band == NO_SWITCH {
            sight_band = 0;
        }
        let sight = f32::from(sight_band) * SIGHT_SCALE;
        let switch_no = ((parameters >> SWITCH_SHIFT) & BYTE_MASK) as u8;
        let always_up = variant == ALWAYS_UP;
        Self {
            variant,
            switch_no,
            mode: if always_up { Mode::Up } else { Mode::Wait },
            at_live: always_up,
            struck: false,
            lid: if always_up { GROW_TARGET } else { 0.0 },
            rise: if always_up { GROW_TARGET } else { 0.0 },
            sight: if sight == 0.0 { SIGHT_DEFAULT } else { sight },
            refused: switch_no != NO_SWITCH && switches.test(switch_no),
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `draw_sub` (`d_a_ssk.cpp:41-64`) scales `mpMorf2` by `m350` and `mpMorf1` by
    /// `actor.scale`, and `createInit` sets both to zero for a statue that has not woken
    /// (`:418-419`), so the pair only draws once either ramp leaves zero.
    pub fn drawn(&self) -> bool {
        self.lid > 0.0 || self.rise > 0.0
    }

    /// `daSsk_Execute` recomputes `m2C4 = m2C0 + 100.0f` (`d_a_ssk.cpp:252`).
    fn leave_range(&self) -> f32 {
        self.sight + LEAVE_GAP
    }

    fn switched(&self, switches: &Switches) -> bool {
        self.switch_no != NO_SWITCH && switches.test(self.switch_no)
    }

    /// `nomal_move` (`d_a_ssk.cpp:84-159`), `case 0:`.
    fn wait(&mut self, actor: &Actor, player: &Player) {
        if (player.pos - actor.pos).length() < self.sight {
            self.at_live = true;
            self.mode = Mode::LiftLid;
        }
    }

    /// `case 1:` - `m350` is the second model's scale, which gates the state change.
    fn lift(&mut self, events: &mut SimEvents) {
        add_calc2(&mut self.lid, GROW_TARGET, GROW_SCALE, GROW_STEP);
        if self.lid > GROW_DONE {
            self.lid = GROW_TARGET;
            self.rise = 0.0;
            self.mode = Mode::Rise;
            events.list.push(SimEvent::Se(SE_JAMA_SHOKU_OUT));
        }
    }

    /// `case 2:` - `actor.scale.y` is the rock's own rise height.
    fn rise_step(&mut self) {
        add_calc2(&mut self.rise, GROW_TARGET, GROW_SCALE, GROW_STEP);
        if self.rise > GROW_DONE {
            self.rise = GROW_TARGET;
            self.mode = Mode::Up;
        }
    }

    /// `case 3:` - returns true when `fopAcM_delete` runs.
    fn sink(&mut self, switches: &Switches) -> bool {
        add_calc0(&mut self.rise, SINK_SCALE, SINK_STEP);
        if self.rise >= SINK_DONE {
            return false;
        }
        self.rise = 0.0;
        add_calc0(&mut self.lid, LID_SCALE, LID_STEP);
        if self.lid >= LID_DONE {
            return false;
        }
        self.lid = 0.0;
        self.mode = Mode::Wait;
        self.switched(switches)
    }

    /// `case 10:`.
    fn up(
        &mut self,
        actor: &Actor,
        player: &Player,
        switches: &Switches,
        events: &mut SimEvents,
    ) {
        if self.variant == 1 && (player.pos - actor.pos).length() > self.leave_range() {
            self.at_live = false;
            self.mode = Mode::Sink;
        }
        if self.switched(switches) {
            self.at_live = false;
            self.mode = Mode::Sink;
            events.list.push(SimEvent::Se(SE_JAMA_SHOKU_IN));
        }
    }

    fn execute(
        &mut self,
        actor: &Actor,
        player: &Player,
        switches: &Switches,
        events: &mut SimEvents,
    ) -> bool {
        match self.mode {
            Mode::Wait => self.wait(actor, player),
            Mode::LiftLid => self.lift(events),
            Mode::Rise => self.rise_step(),
            Mode::Sink => return self.sink(switches),
            Mode::Up => self.up(actor, player, switches, events),
        }
        false
    }
}

pub fn is_ss_turu(name: &str) -> bool {
    SSTURU_ACTORS.contains(&name)
}

pub fn control(
    switches: Res<Switches>,
    mut health: ResMut<Health>,
    players: Query<&Player>,
    mut events: ResMut<SimEvents>,
    mut statues: Query<(Entity, &Actor, &mut SsTuru)>,
    mut commands: Commands,
) {
    let Some(player) = players.iter().next().copied() else {
        return;
    };
    let mut gone: Vec<Entity> = Vec::new();
    for (entity, actor, mut statue) in &mut statues {
        if statue.refused {
            gone.push(entity);
            continue;
        }
        if statue.execute(actor, &player, &switches, &mut events) {
            gone.push(entity);
            continue;
        }
        let touch = cyl_cross(
            actor.pos,
            player.pos,
            CYL_RADIUS,
            CYL_HEIGHT,
            PLAYER_RADIUS,
            PLAYER_HEIGHT,
        );
        if statue.at_live && touch && !statue.struck {
            statue.struck = true;
            health.damage(CONTACT_DAMAGE);
        } else if !touch {
            statue.struck = false;
        }
    }
    for entity in gone {
        let _ = commands.entity(entity).despawn();
    }
}