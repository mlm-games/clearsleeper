use bevy_ecs::prelude::*;

use super::actor::Actor;
use super::collision::Collision;
use super::event::{ActiveEvent, EventList};
use super::events::{SimEvent, SimEvents};
use super::flags::{EventFlags, EventReg, StoryBits, Switches};
use super::ground;
use super::player::Player;
use super::stream;

/// Stage object names (`tww/src/d/d_stage.cpp:605-609`, which also lists `"Warpt"` at `:605`).
/// `daObj_Warpt_c` is `fpcNm_OBJ_WARPT_e`, listed `0x0003`, draw prio
/// `fpcDwPi_OBJ_WARPT_e`, status
/// `0x17 | fopAcStts_SHOWMAP_e | fopAcStts_CULL_e | fopAcStts_UNK40000_e`, cull type
/// `fopAc_CULLBOX_CUSTOM_e` (`d_a_obj_warpt.cpp:883-902`). Its archive is `"ltubw"`
/// (`d_a_obj_warpt.cpp:36`), which differs from every stage string.
pub const WARPT_ACTORS: [&str; 5] = ["Warpt", "Warpnt", "Warpts1", "Warpts2", "Warpts3"];

/// `fopAcM_setCullSizeBox(this, -80.0f, -0.0f, -80.0f, 80.0f, 160.0f, 80.0f)`
/// (`d_a_obj_warpt.cpp:772`).
const CULL_MIN: [f32; 3] = [-80.0, 0.0, -80.0];
const CULL_MAX: [f32; 3] = [80.0, 160.0, 80.0];

const SUB_MASK: u32 = 0xF;
const SLOT_SHIFT: u32 = 0x04;
const SCLS_SHIFT: u32 = 0x0C;
const WARP_A_SHIFT: u32 = 0x08;
const WARP_B_SHIFT: u32 = 0x10;
const WARP_C_SHIFT: u32 = 0x18;
const BYTE_MASK: u32 = 0xFF;

const NO_SWITCH: u8 = 0xFF;
const SUB_SWITCHED: u8 = 1;
const SUB_SP_2: u8 = 2;
const SUB_SP_3: u8 = 3;
const SUB_SP_4: u8 = 4;

const MODE_OPEN: u8 = 0;
const MODE_BREAK_FIRE: u8 = 2;

const GRAVITY: f32 = -6.5;
const MAX_FALL: f32 = -100.0;
const PLAYER_RADIUS: f32 = 50.0;
const PLAYER_HEIGHT: f32 = 100.0;
const TIMER_NEAR: i32 = 30;
const TIMER_FAR: i32 = 5;
const BURN_FRAMES: i32 = 60;
const BIT_1: u8 = 1;
const BIT_2: u8 = 2;
const BIT_4: u8 = 4;

/// `m_event_reg[]` (`d_a_obj_warpt.cpp:37-44`), the `dSv_event_flag_c` ids of
/// `d_save_event_flag.inc:536-541`. Byte-indexed per `d_save.h:722`.
const EVENT_REG: [u16; 6] = [0xA207, 0xA107, 0xA007, 0x9F07, 0xA307, 0xA407];

const SE_READ_RIDDLE_1: u32 = 0x0806;
const SE_OTOSHIANA_SUS: u32 = 0x3019;
const SE_OTOSHIANA_OPEN: u32 = 0x6934;
const SE_WARP_POT_CV_BURN: u32 = 0x69EC;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Open,
    Close,
    BreakFire,
    EventWarp,
    EventOpen,
}

impl Mode {
    fn of(index: u8) -> Self {
        match index {
            MODE_OPEN => Self::Open,
            1 => Self::Close,
            MODE_BREAK_FIRE => Self::BreakFire,
            3 => Self::EventWarp,
            _ => Self::EventOpen,
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Warpt {
    pub sub: u8,
    pub event_slot: u8,
    pub open_switch: u8,
    pub warp_switch: u8,
    pub scls: u8,
    pub warp_a: u8,
    pub warp_b: u8,
    pub warp_c: u8,
    pub riddle: u8,
    pub mark: u8,
    pub warp_bit: u8,
    pub lid: bool,
    pub mode: Mode,
    pub timer: i32,
    pub burn: i32,
    pub speed_y: f32,
    pub placed: bool,
}

fn timer(value: &mut i32) -> i32 {
    if *value != 0 {
        *value -= 1;
    }
    *value
}

/// `getArg`'s `(uVar6 >> N) & 0xff` on the two `home.angle` shorts
/// (`d_a_obj_warpt.cpp:714-715, 727-730, 749-752`).
fn byte_at(value: i16, shift: u32) -> u8 {
    ((u32::from(value as u16) >> shift) & BYTE_MASK) as u8
}

fn byte_param(parameters: u32, shift: u32) -> u8 {
    ((parameters >> shift) & BYTE_MASK) as u8
}

fn reg_id(slot: u8) -> u16 {
    EVENT_REG.get(usize::from(slot)).copied().unwrap_or(0)
}

impl Warpt {
    /// `getArg` (`d_a_obj_warpt.cpp:705-764`) followed by `createInit`'s opening mode
    /// (`:782-786`). `m298` and `m2C5` are written and never read, so they are not kept.
    /// `m2B8` only indexes `m_event_reg`, and `m2AC`/`m2B0` are the room-scoped
    /// `dComIfGs_*Switch` pair that `super::flags::Switches` models flat. `getArg` writes switch
    /// `m2AC` when `m2B4 == 1`, hence the `&mut` borrow.
    pub fn spawn(
        parameters: u32,
        angle: [i16; 3],
        switches: &mut Switches,
        regs: &mut EventReg,
    ) -> Self {
        let sub = (parameters & SUB_MASK) as u8;
        let open_switch = byte_at(angle[0], 0);
        let warp_switch = byte_at(angle[0], 0x08);
        if matches!(sub, SUB_SP_2 | SUB_SP_3 | SUB_SP_4) {
            let event_slot = ((parameters >> SLOT_SHIFT) & SUB_MASK) as u8;
            let warp_bit = match sub {
                SUB_SP_2 => BIT_1,
                SUB_SP_3 => BIT_2,
                _ => BIT_4,
            };
            let id = reg_id(event_slot);
            if byte_at(angle[2], 0x08) != NO_SWITCH {
                let value = regs.reg(id) | warp_bit;
                regs.set_reg(id, value);
            }
            let lid = regs.reg(id) & warp_bit == 0;
            return Self {
                sub,
                event_slot,
                open_switch,
                warp_switch,
                scls: 0,
                warp_a: byte_param(parameters, WARP_A_SHIFT),
                warp_b: byte_param(parameters, WARP_B_SHIFT),
                warp_c: byte_param(parameters, WARP_C_SHIFT),
                riddle: byte_at(angle[2], 0),
                mark: byte_at(angle[2], 0x08),
                warp_bit,
                lid,
                mode: if lid { Mode::Close } else { Mode::Open },
                timer: 0,
                burn: 0,
                speed_y: 0.0,
                placed: false,
            };
        }
        if sub == SUB_SWITCHED {
            switches.set(open_switch);
        }
        let lid = open_switch == NO_SWITCH || !switches.test(open_switch);
        Self {
            sub,
            event_slot: 0,
            open_switch,
            warp_switch,
            scls: ((parameters >> SCLS_SHIFT) & BYTE_MASK) as u8,
            warp_a: 0,
            warp_b: 0,
            warp_c: 0,
            riddle: 0,
            mark: 0,
            warp_bit: 0,
            lid,
            mode: if lid { Mode::Close } else { Mode::Open },
            timer: 0,
            burn: 0,
            speed_y: 0.0,
            placed: false,
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `isSp` (`d_a_obj_warpt.cpp:262-264`).
    fn is_sp(&self) -> bool {
        matches!(self.sub, SUB_SP_2 | SUB_SP_3 | SUB_SP_4)
    }

    /// `isWarpBit` (`d_a_obj_warpt.cpp:292-295`).
    fn is_warp_bit(&self, regs: &EventReg, bit: u8) -> bool {
        regs.reg(reg_id(self.event_slot)) & bit != 0
    }

    /// `onWarpBit` (`d_a_obj_warpt.cpp:285-289`).
    fn on_warp_bit(&self, regs: &mut EventReg) {
        let id = reg_id(self.event_slot);
        let value = regs.reg(id) | self.warp_bit;
        regs.set_reg(id, value);
    }

    /// `warp` (`d_a_obj_warpt.cpp:277-282`). `dLib_setNextStageBySclsNum` (`d_lib.cpp:237-256`)
    /// needs the `.scls` table, which this sim does not load, so only the sound remains.
    fn warp(&mut self, scls: u8, events: &mut SimEvents) {
        events.list.push(SimEvent::Se(SE_OTOSHIANA_OPEN));
        let _ = scls;
    }

    /// `openHuta` (`d_a_obj_warpt.cpp:389-399`).
    fn open_lid(&mut self, switches: &mut Switches, regs: &mut EventReg, events: &mut SimEvents) {
        if self.is_sp() {
            if self.riddle == NO_SWITCH {
                events.list.push(SimEvent::Se(SE_READ_RIDDLE_1));
            }
            self.on_warp_bit(regs);
        } else if self.open_switch != NO_SWITCH {
            switches.set(self.open_switch);
        }
        self.lid = false;
    }

    /// `normalWarp` (`d_a_obj_warpt.cpp:543-551`).
    fn normal_warp(&mut self, switches: &Switches, events: &mut SimEvents) -> bool {
        if self.warp_switch == NO_SWITCH || !switches.test(self.warp_switch) {
            return false;
        }
        let scls = self.scls;
        self.warp(scls, events);
        true
    }

    /// `spWarp` (`d_a_obj_warpt.cpp:554-582`).
    fn sp_warp(&mut self, regs: &EventReg, events: &mut SimEvents) -> bool {
        let picked = match self.sub {
            SUB_SP_2 => {
                if self.is_warp_bit(regs, self.warp_bit) {
                    Some(self.warp_a)
                } else if self.is_warp_bit(regs, BIT_4) {
                    Some(self.warp_b)
                } else {
                    None
                }
            }
            SUB_SP_3 => {
                if self.is_warp_bit(regs, BIT_4) {
                    Some(self.warp_b)
                } else if self.is_warp_bit(regs, BIT_1) {
                    Some(self.warp_a)
                } else {
                    None
                }
            }
            SUB_SP_4 => {
                if self.is_warp_bit(regs, BIT_1) {
                    Some(self.warp_a)
                } else if self.is_warp_bit(regs, self.warp_bit) {
                    Some(self.warp_b)
                } else {
                    None
                }
            }
            _ => None,
        };
        match picked {
            Some(scls) => {
                self.warp(scls, events);
                true
            }
            None => false,
        }
    }

    /// `modeProc(PROC_INIT_e, index)` (`d_a_obj_warpt.cpp:597-639`) plus
    /// `modeBreakFireInit`'s sound and `m83C = 60` (`:479-480`).
    fn mode_proc(&mut self, index: u8, events: &mut SimEvents) {
        if index == MODE_BREAK_FIRE {
            events.list.push(SimEvent::Se(SE_WARP_POT_CV_BURN));
            self.burn = BURN_FRAMES;
        }
        self.mode = Mode::of(index);
    }

    /// `fopAcM_posMoveF(this, mStts.GetCCMoveP())` (`d_a_obj_warpt.cpp:669`) is
    /// `fopAcM_calcSpeed` + `fopAcM_posMove` (`f_op_actor_mng.cpp:454-484`). `GetCCMoveP()` is the
    /// zeroed `cCcD_Stts::m_cc_move` because this translation unit never calls `PlusCcMove`, and
    /// `mAcch.CrrPos` reduces to the merged-DZB ground test.
    fn integrate(&mut self, actor: &mut Actor, solid: Option<&Collision>) {
        let (travel, _) = stream::calc_speed(0.0, actor.angle[1], self.speed_y, GRAVITY, MAX_FALL);
        self.speed_y = travel.y;
        actor.pos += travel;
        if let Some(floor) = ground::surface(solid, actor.pos)
            && actor.pos.y < floor
        {
            actor.pos.y = floor;
            self.speed_y = 0.0;
        }
    }

    /// `createInit`'s one-shot prologue (`d_a_obj_warpt.cpp:798-806`):
    /// `fopAcM_SetGravity(this, -6.5f)`, `fopAcM_posMoveF(this, NULL)` and
    /// `dLib_checkPlayerInCircle(current.pos, 50.0f, 100.0f)` setting `m840`.
    fn place(&mut self, actor: &mut Actor, player: Option<&Player>, solid: Option<&Collision>) {
        self.placed = true;
        self.integrate(actor, solid);
        let near = player.is_some_and(|player| {
            let dx = player.pos.x - actor.pos.x;
            let dz = player.pos.z - actor.pos.z;
            let dy = (player.pos.y - actor.pos.y).abs();
            dx * dx + dz * dz < PLAYER_RADIUS * PLAYER_RADIUS && dy < PLAYER_HEIGHT
        });
        self.timer = if near { TIMER_NEAR } else { TIMER_FAR };
    }

    /// `modeOpen` (`d_a_obj_warpt.cpp:444-446`).
    fn mode_open(&mut self, events: &mut SimEvents) {
        events.list.push(SimEvent::Se(SE_OTOSHIANA_SUS));
    }

    /// `modeClose` (`d_a_obj_warpt.cpp:454-475`). The `mCyl2` Tg arm needs the lid collider that
    /// only `isRealHuta()` registers (`:346-349`); the sim has no lid collider, so only the
    /// switch arm remains. `modeCloseInit`'s particle is omitted.
    fn mode_close(&mut self, switches: &mut Switches, regs: &mut EventReg, events: &mut SimEvents) {
        if self.open_switch != NO_SWITCH && switches.test(self.open_switch) {
            self.open_lid(switches, regs, events);
            self.mode_proc(MODE_OPEN, events);
        }
    }

    /// `modeBreakFire` (`d_a_obj_warpt.cpp:484-496`); `mLidBrk.play()` is renderer work.
    fn mode_break_fire(
        &mut self,
        switches: &mut Switches,
        regs: &mut EventReg,
        events: &mut SimEvents,
    ) {
        if timer(&mut self.burn) == 0 {
            self.open_lid(switches, regs, events);
            self.mode_proc(MODE_OPEN, events);
        }
    }

    /// `modeEventWarp` (`d_a_obj_warpt.cpp:503-540`). `eventInfo.checkCommandDemoAccrpt()` is
    /// `mCommand == dEvtCmd_INDEMO_e` (`f_op_actor.h:177`), which the event manager raises when
    /// the ordered event starts; `ActiveEvent::start_check` is this sim's model of that. The
    /// player's `onNoResetFlg0`/`onNoResetFlg1` calls are player control and are omitted.
    fn mode_event_warp(
        &mut self,
        accepted: bool,
        cut: Option<usize>,
        active: &mut ActiveEvent,
        list: &EventList,
        flags: &mut EventFlags,
        switches: &Switches,
        regs: &EventReg,
        events: &mut SimEvents,
    ) {
        if !accepted {
            return;
        }
        let mut taken = true;
        if cut.is_some_and(|cut| list.cut_name(cut) == Some("WARP")) {
            taken = if self.is_sp() {
                self.sp_warp(regs, events)
            } else {
                self.normal_warp(switches, events)
            };
            active.end_staff_cut(list, flags, "Warpt");
        }
        if taken {
            return;
        }
        let Some(index) = list.event_index("DEFAULT_WARP") else {
            return;
        };
        if active.end_check(index) {
            self.timer = TIMER_NEAR;
            active.end(index);
            self.mode_proc(MODE_OPEN, events);
        }
    }

    /// `modeEventOpen` (`d_a_obj_warpt.cpp:589-594`). Nothing in the decomp ever calls
    /// `modeProc(PROC_INIT_e, 4)`, so retail never enters this mode.
    fn mode_event_open(
        &mut self,
        accepted: bool,
        active: &mut ActiveEvent,
        list: &EventList,
        events: &mut SimEvents,
    ) {
        if !accepted {
            return;
        }
        let Some(index) = list.event_index("WARPT_OPEN") else {
            return;
        };
        if active.end_check(index) {
            active.end(index);
            self.mode_proc(MODE_OPEN, events);
        }
    }
}

pub fn is_warpt(name: &str) -> bool {
    WARPT_ACTORS.contains(&name)
}

pub fn control(
    collision: Option<Res<Collision>>,
    mut switches: ResMut<Switches>,
    mut regs: ResMut<StoryBits>,
    list: Res<EventList>,
    mut active: ResMut<ActiveEvent>,
    mut flags: ResMut<EventFlags>,
    players: Query<&Player>,
    mut events: ResMut<SimEvents>,
    mut pots: Query<(Entity, &mut Actor, &mut Warpt)>,
) {
    let solid = collision.as_deref();
    let player = players.iter().next().copied();
    let default_warp = list.event_index("DEFAULT_WARP");
    for (entity, mut actor, mut pot) in &mut pots {
        let actor = &mut *actor;
        let pot = &mut *pot;
        if !pot.placed {
            pot.place(actor, player.as_ref(), solid);
        }
        let accepted = default_warp.is_some_and(|index| active.start_check(index));
        if !accepted && matches!(pot.mode, Mode::EventWarp | Mode::EventOpen) {
            if let Some(index) = default_warp {
                active.order(&list, index, entity);
            }
        }
        match pot.mode {
            Mode::Open => pot.mode_open(&mut events),
            Mode::Close => pot.mode_close(&mut switches, &mut regs, &mut events),
            Mode::BreakFire => pot.mode_break_fire(&mut switches, &mut regs, &mut events),
            Mode::EventWarp => pot.mode_event_warp(
                accepted,
                active.staff_cut(&list, "Warpt").map(|(cut, _)| cut),
                &mut active,
                &list,
                &mut flags,
                &switches,
                &regs,
                &mut events,
            ),
            Mode::EventOpen => pot.mode_event_open(accepted, &mut active, &list, &mut events),
        }
        pot.integrate(actor, solid);
        timer(&mut pot.timer);
    }
}
