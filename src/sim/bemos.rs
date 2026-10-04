use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::combat::HitPoints;
use super::events::{SimEvent, SimEvents};
use super::flags::{EventFlags, Switches, ZoneActors};
use super::health::Health;
use super::pickup::Pickup;
use super::player::Player;
use super::stream::add_calc_angle_s2;
use super::walk::{chase_f, from_s16, swept, to_s16};

pub const BEMOS_ACTORS: [&str; 3] = ["Hmos1", "Hmos2", "Hmos3"];

const BEAM_ACTOR: &str = "Ylesr00";
const ITEM_ACTOR: &str = "item";

const BYTE_MASK: u32 = 0xFF;
const SWAY_SHIFT: u32 = 8;
const KIND_SHIFT: u32 = 0x1C;
const KIND_MASK: u32 = 0x3;
const SHAPE_ITEM_MASK: u32 = 0x3F;
const BEAM_PARAM: u32 = 0x1000000;

const KIND_RED: u8 = 1;
const KIND_YELLOW: u8 = 2;
const NO_SWITCH: u8 = 0xFF;

const RANGE_BASE: f32 = 500.0;
const RANGE_STEP: f32 = 100.0;
const RANGE_FIXED: f32 = 1000.0;
const SWAY_STEP: i16 = 250;
const SWAY_POS: i16 = 750;
const SWAY_NEG: i16 = 500;

const BLUE_SIDE: f32 = 400.0;
const BLUE_OPEN: f32 = 0.3;
const BLUE_SHUT: f32 = 0.3;
const BLUE_HOLD_FIRST: i16 = 3;
const BLUE_HOLD_AGAIN: i16 = 7;

const RED_OPEN: f32 = 0.1;
const RED_SHUT: f32 = 0.07;
const RED_HOLD: i16 = 0;
const RED_GRACE: i16 = 30;
const RED_YAW_STEP: i16 = 400;

const EYE_HEIGHT: f32 = 260.0;
const YELLOW_SEAT: f32 = 80.0;
const EYE_LIMIT: f32 = 250.0;
const LOCK_RANGE: f32 = 5.0;
const LOCK_CONE: u16 = 0x1000;
const TURN_CAP: i16 = 0x400;
const TURN_DIV: i16 = 4;
const TURN_DIV_RED: i16 = 2;
const TURN_DIV_YAW: i16 = 1;

const BEAM_TIP: f32 = 60.0;
const BEAM_SEAT: f32 = 40.0;
const BEAM_LENGTH: f32 = 1000.0;
const BEAM_RADIUS: f32 = 25.0;
const BEAM_GROW: f32 = 5.0;
const BEAM_SHRINK: f32 = 4.0;
const CHARGE_FRAMES: f32 = 5.0;
const DROP_OFFSET: f32 = 30.0;

const HIT_POWER: u8 = 1;
const SE_BREAK: u32 = 0x6807;
const SE_HIT: u32 = 0x2853;
const EVENT_LIT: u32 = 0x1010;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Eye {
    Wait,
    Charge,
    Search,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Bemos {
    pub kind: u8,
    pub eye: Vec3,
    pub reach: f32,
    pub body: i16,
    pub state: Eye,
    pub ray: Option<Entity>,
    pub open: f32,
    pub charge: f32,
    pub hold: i16,
    pub gate: i16,
    pub sway: i16,
    pub grace: i16,
    pub pitch: i16,
    pub yaw: i16,
    pub lock: Option<Vec3>,
    pub seen: bool,
    pub switch_no: u8,
    pub latched: bool,
    pub item: u8,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Beam {
    pub grow: f32,
    pub shrink: f32,
    pub lit: bool,
}

fn aim(pitch: i16, yaw: i16) -> Vec3 {
    let (tilt, level) = from_s16(pitch).sin_cos();
    let (sin, cos) = from_s16(yaw).sin_cos();
    Vec3::new(level * sin, -tilt, level * cos)
}

fn ramp_up(beam: &mut Beam) {
    if beam.lit {
        beam.shrink = 0.0;
        beam.grow = BEAM_GROW;
        return;
    }
    beam.shrink = 0.0;
    if beam.grow < BEAM_GROW {
        beam.grow += 1.0;
    } else {
        beam.grow = BEAM_GROW;
        beam.lit = true;
    }
}

fn ramp_down(beam: &mut Beam) -> bool {
    if !beam.lit {
        beam.grow = 0.0;
        beam.shrink = 0.0;
        return true;
    }
    if beam.grow < BEAM_GROW {
        beam.grow += 1.0;
    }
    if beam.shrink < BEAM_SHRINK {
        beam.shrink += 1.0;
        false
    } else {
        beam.grow = 0.0;
        beam.shrink = 0.0;
        beam.lit = false;
        true
    }
}

fn blue_range(bemos: &mut Bemos, player: &Player, pos: Vec3, lift: f32) -> Option<i16> {
    let delta = player.pos - pos;
    let (sin, cos) = from_s16(bemos.body).sin_cos();
    let ahead = delta.x * sin + delta.z * cos;
    let across = delta.x * cos - delta.z * sin;
    let flat = (ahead * ahead + across * across).sqrt();
    if ahead > 0.0
        && ahead < bemos.reach
        && across.abs() < BLUE_SIDE
        && delta.y.abs() < EYE_LIMIT
    {
        bemos.seen = true;
        Some(to_s16((delta.y + lift).atan2(flat)))
    } else {
        bemos.seen = false;
        None
    }
}

fn red_range(
    bemos: &mut Bemos,
    player: &Player,
    pos: Vec3,
    lift: f32,
) -> Option<(i16, i16)> {
    let delta = player.pos - pos;
    let flat = (delta.x * delta.x + delta.z * delta.z).sqrt();
    let bearing = to_s16(delta.x.atan2(delta.z));
    let relative = bearing.wrapping_sub(bemos.body);
    let error = relative.wrapping_sub(bemos.yaw).unsigned_abs();
    if let Some(anchor) = bemos.lock {
        if (player.pos - anchor).length() <= LOCK_RANGE && error <= LOCK_CONE {
            bemos.seen = false;
            return None;
        }
        bemos.lock = None;
    }
    if (error < LOCK_CONE || bemos.seen) && flat < bemos.reach {
        bemos.seen = true;
        Some((to_s16((lift - delta.y).atan2(flat)), relative))
    } else {
        bemos.seen = false;
        None
    }
}

fn spawn_ray(commands: &mut Commands, actor: &Actor, beamos: &Bemos) -> Entity {
    let yaw = beamos.body.wrapping_add(beamos.yaw);
    let seat = if beamos.kind == KIND_YELLOW { BEAM_SEAT } else { 0.0 };
    let ray = Actor {
        name: BEAM_ACTOR.to_owned(),
        parameters: BEAM_PARAM,
        pos: beamos.eye + aim(beamos.pitch, yaw) * seat,
        angle: [beamos.pitch, yaw, 0],
        set_id: actor.set_id,
        room: actor.room,
    };
    commands
        .spawn((
            ray,
            Beam {
                grow: 0.0,
                shrink: 0.0,
                lit: false,
            },
        ))
        .id()
}

fn shatter(actor: &Actor, beamos: &Bemos, events: &mut SimEvents, commands: &mut Commands) {
    let yaw = beamos.body.wrapping_add(beamos.yaw);
    let (sin, cos) = from_s16(yaw).sin_cos();
    let pos = beamos.eye + Vec3::new(sin * DROP_OFFSET, 0.0, cos * DROP_OFFSET);
    events.list.push(SimEvent::Se(SE_BREAK));
    commands.spawn((
        Actor {
            name: ITEM_ACTOR.to_owned(),
            parameters: u32::from(beamos.item),
            pos,
            angle: [beamos.pitch, yaw, 0],
            set_id: actor.set_id,
            room: actor.room,
        },
        Pickup::spawn(u32::from(beamos.item)),
    ));
}

impl Bemos {
    pub fn spawn(
        pos: Vec3,
        parameters: u32,
        shape: [i16; 3],
        switches: &Switches,
    ) -> (Self, HitPoints) {
        let kind = ((parameters >> KIND_SHIFT) & KIND_MASK) as u8;
        let low = (parameters & BYTE_MASK) as u8;
        let band = ((parameters >> SWAY_SHIFT) & BYTE_MASK) as i8;
        let (eye, body) = match kind {
            KIND_YELLOW => (pos + aim(shape[0], shape[1]) * YELLOW_SEAT, 0),
            _ => (pos + Vec3::new(0.0, EYE_HEIGHT, 0.0), shape[1]),
        };
        let reach = match kind {
            KIND_YELLOW => RANGE_FIXED,
            _ => {
                let scale = if low == NO_SWITCH { 0.0 } else { f32::from(low) };
                scale * RANGE_STEP + RANGE_BASE
            }
        };
        let sway = match kind {
            KIND_RED if band >= 0 => i16::from(band) * SWAY_STEP + SWAY_POS,
            KIND_RED => i16::from(band) * SWAY_STEP - SWAY_NEG,
            _ => 0,
        };
        let latched = kind == KIND_YELLOW && low != NO_SWITCH && switches.test(low);
        (
            Self {
                kind,
                eye,
                reach,
                body,
                state: Eye::Wait,
                ray: None,
                open: 0.0,
                charge: 0.0,
                hold: 0,
                gate: 0,
                sway,
                grace: 0,
                pitch: 0,
                yaw: 0,
                lock: None,
                seen: false,
                switch_no: low,
                latched,
                item: ((shape[2] as u32) & SHAPE_ITEM_MASK) as u8,
            },
            HitPoints(super::damage::KILL_POWER),
        )
    }
}

pub fn is_bemos(name: &str) -> bool {
    BEMOS_ACTORS.contains(&name)
}

pub fn control(
    players: Query<&Player>,
    mut beamoses: Query<(Entity, &Actor, &mut Bemos, &HitPoints), (With<Bemos>, Without<Beam>)>,
    mut rays: Query<(Entity, &mut Actor, &mut Beam), With<Beam>>,
    mut health: ResMut<Health>,
    mut events: ResMut<SimEvents>,
    switches: Res<Switches>,
    mut lit: ResMut<EventFlags>,
    mut zones: ResMut<ZoneActors>,
    mut commands: Commands,
) {
    if let Some(player) = players.iter().next().copied() {
        for (_, actor, ray, _) in &rays {
            if !ray.lit {
                continue;
            }
            let dir = aim(actor.angle[0], actor.angle[1]);
            if swept(
                actor.pos,
                actor.pos + dir * BEAM_LENGTH,
                player.pos,
                BEAM_RADIUS,
            ) {
                health.damage(HIT_POWER);
                events.list.push(SimEvent::Se(SE_HIT));
            }
        }
    }
    let Some(player) = players.iter().next().copied() else {
        return;
    };
    for (entity, actor, mut beamos, hit_points) in &mut beamoses {
        if hit_points.0 <= 0 {
            if beamos.kind != KIND_YELLOW {
                shatter(actor, &bemos, &mut events, &mut commands);
            }
            if let Some(ray) = beamos.ray {
                let _ = commands.entity(ray).despawn();
            }
            zones.mark_done(actor.room, actor.set_id);
            let _ = commands.entity(entity).despawn();
            continue;
        }
        if bemos.ray.is_none() {
            let ray = spawn_ray(&mut commands, actor, &bemos);
            bemos.ray = Some(ray);
        }
        let mut ray = match bemos.ray {
            Some(id) => rays.get_mut(id).ok().map(|(_, body, beam)| (body, beam)),
            None => None,
        };
        let pos = actor.pos;
        let lift = bemos.eye.y - pos.y;
        let tip = bemos.eye + aim(bemos.pitch, bemos.body.wrapping_add(bemos.yaw)) * BEAM_TIP;
        match bemos.state {
            Eye::Wait => match bemos.kind {
                KIND_YELLOW => {
                    if bemos.switch_no != NO_SWITCH && switches.test(bemos.switch_no) {
                        if let Some((_, beam)) = ray.as_mut()
                            && beam.lit
                        {
                            ramp_down(&mut *beam);
                        }
                    } else {
                        bemos.state = Eye::Search;
                        bemos.charge = 0.0;
                    }
                }
                KIND_RED => {
                    if red_range(&mut bemos, &player, pos, lift).is_some() {
                        bemos.state = Eye::Charge;
                        bemos.charge = 0.0;
                        bemos.hold = 0;
                        bemos.gate = RED_HOLD;
                    } else {
                        add_calc_angle_s2(&mut bemos.pitch, 0, TURN_DIV, TURN_CAP);
                        bemos.yaw = bemos.yaw.wrapping_add(bemos.sway);
                    }
                }
                _ => {
                    if blue_range(&mut bemos, &player, pos, lift).is_some() {
                        bemos.state = Eye::Charge;
                        bemos.charge = 0.0;
                        bemos.hold = 0;
                        bemos.gate = BLUE_HOLD_FIRST;
                    }
                }
            },
            Eye::Charge => {
                if bemos.kind == KIND_RED {
                    if let Some((pitch, _)) = red_range(&mut bemos, &player, pos, lift) {
                        chase_f(&mut bemos.open, 1.0, RED_OPEN);
                        if bemos.hold > bemos.gate {
                            add_calc_angle_s2(&mut bemos.pitch, pitch, TURN_DIV_RED, TURN_CAP);
                            if bemos.open == 1.0 {
                                bemos.charge += 1.0;
                                if bemos.charge >= CHARGE_FRAMES {
                                    bemos.state = Eye::Search;
                                    bemos.grace = RED_GRACE;
                                }
                            }
                        } else {
                            bemos.hold = bemos.hold.wrapping_add(1);
                        }
                    } else if bemos.hold <= 0 {
                        chase_f(&mut bemos.open, 0.0, RED_SHUT);
                        if bemos.open == 0.0 {
                            bemos.state = Eye::Wait;
                        }
                        add_calc_angle_s2(&mut bemos.pitch, 0, TURN_DIV, TURN_CAP);
                        if bemos.charge > 0.0 {
                            bemos.charge -= 1.0;
                        }
                    } else {
                        bemos.hold = bemos.hold.wrapping_sub(1);
                    }
                } else if let Some(pitch) = blue_range(&mut bemos, &player, pos, lift) {
                    chase_f(&mut bemos.open, 1.0, BLUE_OPEN);
                    if bemos.hold > bemos.gate {
                        if bemos.open == 1.0 {
                            bemos.charge += 1.0;
                            if bemos.charge >= CHARGE_FRAMES {
                                bemos.charge = 0.0;
                                bemos.state = Eye::Search;
                            }
                        }
                    } else {
                        bemos.hold = bemos.hold.wrapping_add(1);
                    }
                    add_calc_angle_s2(&mut bemos.pitch, pitch, TURN_DIV, TURN_CAP);
                } else if bemos.hold <= 0 {
                    chase_f(&mut bemos.open, 0.0, BLUE_SHUT);
                    if bemos.open == 0.0 {
                        bemos.state = Eye::Wait;
                    }
                    add_calc_angle_s2(&mut bemos.pitch, 0, TURN_DIV, TURN_CAP);
                    if bemos.charge > 0.0 {
                        bemos.charge -= 1.0;
                    }
                } else {
                    bemos.hold = bemos.hold.wrapping_sub(1);
                }
            }
            Eye::Search => match bemos.kind {
                KIND_YELLOW => {
                    let armed = bemos.switch_no == NO_SWITCH || !switches.test(bemos.switch_no);
                    if armed {
                        if let Some((_, beam)) = ray.as_mut() {
                            ramp_up(&mut *beam);
                        }
                    } else if ray.as_mut()
                        .map_or(true, |(_, beam)| ramp_down(&mut *beam))
                    {
                        bemos.state = Eye::Wait;
                    }
                }
                KIND_RED => {
                    if let Some((pitch, yaw)) = red_range(&mut bemos, &player, pos, lift) {
                        add_calc_angle_s2(&mut bemos.pitch, pitch, TURN_DIV, TURN_CAP);
                        add_calc_angle_s2(&mut bemos.yaw, yaw, TURN_DIV_YAW, RED_YAW_STEP);
                        if let Some((body, beam)) = ray.as_mut() {
                            body.pos = tip;
                            body.angle = [bemos.pitch, bemos.body.wrapping_add(bemos.yaw), 0];
                            ramp_up(&mut *beam);
                        }
                        if bemos.grace > 0 {
                            bemos.grace -= 1;
                        } else {
                            bemos.lock = Some(player.pos);
                        }
                    } else if ray.as_mut()
                        .map_or(true, |(_, beam)| ramp_down(&mut *beam))
                    {
                        bemos.state = Eye::Charge;
                        bemos.hold = RED_HOLD;
                        bemos.gate = RED_HOLD;
                    }
                }
                _ => {
                    if let Some(pitch) = blue_range(&mut bemos, &player, pos, lift) {
                        add_calc_angle_s2(&mut bemos.pitch, pitch, TURN_DIV, TURN_CAP);
                        if let Some((body, beam)) = ray.as_mut() {
                            body.pos = tip;
                            body.angle = [bemos.pitch, bemos.body.wrapping_add(bemos.yaw), 0];
                            ramp_up(&mut *beam);
                        }
                    } else if ray.as_mut()
                        .map_or(true, |(_, beam)| ramp_down(&mut *beam))
                    {
                        bemos.state = Eye::Charge;
                        bemos.hold = BLUE_HOLD_AGAIN;
                        bemos.gate = BLUE_HOLD_AGAIN;
                    }
                }
            },
        }
        if bemos.kind == KIND_YELLOW && bemos.switch_no != NO_SWITCH {
            let on = switches.test(bemos.switch_no);
            if on != bemos.latched && on && !lit.test(EVENT_LIT) {
                lit.set(EVENT_LIT);
            }
            bemos.latched = on;
        }
    }
}