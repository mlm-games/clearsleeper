use bevy_ecs::prelude::*;

#[derive(Component, Debug, Clone, Copy)]
pub struct Scale(pub glam::Vec3);

use super::actor::Actor;
use super::daynight::{DayNight, TIME_DAY};
use super::flags::{EventFlags, Switches};
use super::ship::Crane;

pub const SALVAGE_ACTORS: [&str; 6] = [
    "SalvFM", "Salvag2", "SalvagE", "SalvagN", "Salvage", "SwSlvg",
];

pub fn is_salvage(name: &str) -> bool {
    SALVAGE_ACTORS.contains(&name)
}

pub const MAX_SEA: usize = 128;
pub const MAX_ROOM: usize = 32;
pub const MAX_TOTAL: usize = MAX_ROOM + MAX_SEA;
pub const OCEAN_GRIDS: usize = 50;
pub const SAVE_SLOT_MAX: u8 = 15;
pub const COLLECT_MAX: u8 = 128;
pub const CMAP_MAX: i32 = 64;
pub const SAVE_NO_ALWAYS: u8 = 31;
pub const ALPHA_FULL: u8 = 0xFF;
pub const ALPHA_GONE: u8 = 0;
pub const NEAR_RANGE: f32 = 25000.0;

pub const DEPTH_TIERS: [f32; 3] = [1000.0, 1500.0, 2000.0];

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Info {
    pub room_no: i8,
    pub pos: glam::Vec3,
    pub scale: glam::Vec3,
    pub save_no: u8,
    pub kind: u8,
    pub item_no: u8,
    pub switch_no: u8,
    pub height: f32,
    pub radius: f32,
    pub alpha: u8,
    pub draw_mode: i32,
    pub flags: u8,
}

impl Info {
    fn reset_flag(&mut self) {
        self.flags = 0;
    }

    fn set_flag(&mut self, flag: u8) {
        self.flags |= flag;
    }

    fn clr_flag(&mut self, flag: u8) {
        self.flags &= !flag;
    }

    fn check_used(&self, flag: u8) -> bool {
        self.flags & flag != 0
    }

    fn check_regist(&self) -> bool {
        self.room_no != -1
    }
}

#[derive(Resource, Debug, Clone)]
pub struct Control {
    pub info: Vec<Info>,
    pub sea_num: usize,
    pub room_num: usize,
    pub ocean_sv: [u16; OCEAN_GRIDS],
    pub random_salvage_point: u8,
    pub collect: [u8; 16],
    pub inside_near: f32,
    pub surface_near: f32,
    pub cmap_near: f32,
    pub disappear_frame: u8,
    pub depth_index: usize,
    pub in_sea: bool,
}

impl Default for Control {
    fn default() -> Self {
        let mut info = vec![Info {
            room_no: -1,
            ..Info::default()
        }; MAX_TOTAL];
        for slot in &mut info {
            slot.room_no = -1;
        }
        Self {
            info,
            sea_num: 0,
            room_num: 0,
            ocean_sv: [0; OCEAN_GRIDS],
            random_salvage_point: 0,
            collect: [0; 16],
            inside_near: 0.0,
            surface_near: 0.0,
            cmap_near: 0.0,
            disappear_frame: 10,
            depth_index: 0,
            in_sea: false,
        }
    }
}

pub struct Params {
    pub kind: u8,
    pub item_no: u8,
    pub room_no_prm: u8,
    pub save_no: u8,
    pub kind_type: u8,
    pub switch_no: u8,
    pub cmap_no: u8,
}

pub fn decode(parameters: u32, angle_z: i16) -> Params {
    Params {
        kind: ((parameters >> 28) & 0xFF) as u8,
        item_no: ((parameters >> 4) & 0xFF) as u8,
        room_no_prm: ((parameters >> 12) & 0xFF) as u8,
        save_no: ((parameters >> 20) & 0xFF) as u8,
        kind_type: (parameters & 0x0F) as u8,
        switch_no: (angle_z & 0xFF) as u8,
        cmap_no: (angle_z & 0x03) as u8,
    }
}

impl Control {
    pub fn ocean_bit(&self, grid: u8, bit: u16) -> bool {
        self.ocean_sv
            .get(usize::from(grid))
            .is_some_and(|word| word & (1 << bit) != 0)
    }

    fn invalid(&self, no: usize, room_no: i8, cmap_no: i32, kind: u8, save_no: u8, switch_no: i32) -> bool {
        if room_no != 0 && self.room_num > MAX_ROOM {
            return true;
        }
        if room_no == 0 && self.sea_num > MAX_SEA {
            return true;
        }
        if cmap_no <= 0 || cmap_no > CMAP_MAX {
            return true;
        }
        if (kind == 2 || kind == 3) && save_no > SAVE_SLOT_MAX {
            return true;
        }
        if kind == 0 && !(COLLECT_MAX >= save_no && save_no >= 1) {
            return true;
        }
        if kind == 2 && switch_no == 0xFF {
            return true;
        }
        no == usize::MAX
    }

    pub fn entry(
        &mut self,
        actor: &Actor,
        scale: glam::Vec3,
        switches: &Switches,
        events: &EventFlags,
        day: &DayNight,
    ) {
        let _ = events;
        let params = decode(actor.parameters, actor.angle[2]);
        let room_no = actor.room;
        let (no, cmap_no) = if room_no != 0 {
            (self.room_num + MAX_SEA, i32::from(room_no))
        } else {
            (self.sea_num, i32::from(params.room_no_prm))
        };
        if no >= MAX_TOTAL
            || self.invalid(no, room_no, cmap_no, params.kind, params.save_no, i32::from(params.switch_no))
        {
            return;
        }
        let grid = cmap_no.clamp(0, OCEAN_GRIDS as i32 - 1) as u8;

        let mut show = false;
        match params.kind {
            0 => {
                if self.collect_byte(params.save_no) != 0 {
                    return;
                }
                if params.cmap_no != self.random_salvage_point {
                    return;
                }
                show = true;
            }
            2 => {
                if !switches.test(params.switch_no) {
                    show = true;
                }
                if params.save_no != SAVE_NO_ALWAYS && self.ocean_bit(grid, u16::from(params.save_no)) {
                    return;
                }
            }
            3 => {
                if params.save_no != SAVE_NO_ALWAYS && self.ocean_bit(grid, u16::from(params.save_no)) {
                    return;
                }
            }
            4 => {
                if params.save_no != SAVE_NO_ALWAYS && self.ocean_bit(grid, u16::from(params.save_no)) {
                    return;
                }
                if day.daynight() == TIME_DAY {
                    show = true;
                }
            }
            _ => {}
        }

        let slot = &mut self.info[no];
        slot.reset_flag();
        if show {
            slot.set_flag(1);
        }
        slot.room_no = room_no;
        slot.pos = actor.pos;
        slot.scale = scale;
        slot.save_no = params.save_no;
        slot.kind = params.kind;
        slot.item_no = params.item_no;
        slot.switch_no = params.switch_no;
        let (height, radius) = match params.kind {
            0 => (slot.scale.y * 1000.0, slot.scale.x * 700.0),
            5 => (slot.scale.y * 1000.0, slot.scale.x * 500.0),
            _ => (slot.scale.y * 500.0, slot.scale.x * 400.0),
        };
        slot.height = height;
        slot.radius = radius;

        if room_no != 0 {
            self.room_num += 1;
        } else {
            self.sea_num += 1;
        }
    }

    fn collect_byte(&self, save_no: u8) -> u8 {
        self.collect
            .get(usize::from(save_no / 8))
            .map_or(0, |byte| byte >> (save_no % 8) & 1)
    }

    pub fn clear_room(&mut self) {
        for slot in self.info.iter_mut().skip(MAX_SEA) {
            init_slot(slot);
        }
        self.room_num = 0;
    }

    pub fn get_distance(&self, no: usize, at: glam::Vec3) -> Option<f32> {
        let slot = self.info.get(no)?;
        if !slot.check_regist() {
            return None;
        }
        Some((slot.pos - at).length())
    }

    pub fn calc_alpha(&mut self, player: glam::Vec3, event_running: bool) {
        let near = if self.in_sea {
            self.surface_near
        } else {
            self.inside_near
        };
        let step = f32::from(ALPHA_FULL) / f32::from(self.disappear_frame.max(1));
        for index in 0..MAX_TOTAL {
            let Some(distance) = self.get_distance(index, player) else {
                continue;
            };
            let slot = &mut self.info[index];
            let mut threshold = near;
            if slot.kind == 0 {
                threshold = self.cmap_near;
            }
            let in_range = event_running
                || distance > threshold * 100.0
                || slot.kind == 2
                    && distance <= NEAR_RANGE;
            if slot.check_used(1) && distance <= NEAR_RANGE && in_range {
                chase(&mut slot.alpha, ALPHA_FULL, step);
                slot.draw_mode = 1;
            } else {
                if chase(&mut slot.alpha, ALPHA_GONE, step) {
                    slot.draw_mode = 0;
                } else {
                    slot.draw_mode = 1;
                }
            }
        }
    }

    pub fn check_area(&self, crane: glam::Vec3, no: usize) -> bool {
        let Some(slot) = self.info.get(no) else {
            return false;
        };
        if !slot.check_regist() {
            return false;
        }
        let flat = ((crane - slot.pos).x * (crane - slot.pos).x
            + (crane - slot.pos).z * (crane - slot.pos).z)
            .sqrt();
        if flat < slot.radius {
            let surface = if self.in_sea {
                self.surface_near
            } else {
                self.inside_near
            };
            if crane.y < surface - DEPTH_TIERS[self.depth_index.min(2)] {
                return true;
            }
        }
        false
    }
}

fn chase(value: &mut u8, target: u8, step: f32) -> bool {
    if *value == target {
        return target == ALPHA_GONE;
    }
    if *value < target {
        let next = f32::from(*value) + step;
        *value = if next >= f32::from(target) { target } else { next as u8 };
    } else {
        let next = f32::from(*value) - step;
        *value = if next <= f32::from(target) { target } else { next as u8 };
    }
    *value == target
}

fn init_slot(slot: &mut Info) {
    slot.room_no = -1;
    slot.pos = glam::Vec3::ZERO;
    slot.scale = glam::Vec3::ZERO;
    slot.save_no = 0;
    slot.kind = 0;
    slot.item_no = 0;
    slot.switch_no = 0;
    slot.alpha = 0;
    slot.draw_mode = 0;
    slot.reset_flag();
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    WaitInit,
    Wait,
    SalvageInit,
    Salvage,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Salvage {
    pub phase: Phase,
    pub live: bool,
    pub spawn_box: bool,
    pub timer: i32,
}

impl Default for Salvage {
    fn default() -> Self {
        Self {
            phase: Phase::WaitInit,
            live: false,
            spawn_box: false,
            timer: 0,
        }
    }
}

pub fn control(
    tags: Query<(&Actor, &Salvage, &Scale)>,
    players: Query<&super::player::Player>,
    cranes: Query<&Crane>,
    switches: Res<Switches>,
    events: Res<EventFlags>,
    day: Res<DayNight>,
    mut control: ResMut<Control>,
) {
    let Some(player) = players.iter().next().copied() else {
        return;
    };
    if cranes.iter().any(|crane| crane.crane_top().is_some()) {
        control.in_sea = true;
    }
    for (actor, tag, scale) in tags.iter() {
        if tag.phase == Phase::Salvage {
            control.entry(actor, scale.0, &switches, &events, &day);
        }
    }
    control.calc_alpha(player.pos, false);
}

pub fn init(world: &mut World) {
    world.init_resource::<Control>();
}