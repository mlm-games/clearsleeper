use bevy_ecs::prelude::*;

use glam::Vec3;

use super::actor::Actor;
use super::combat::{HitPoints, actor_status};
use super::damage;
use super::damage_resolve::{
    AT_SE_ARROW, AT_SPL_UNK0, AT_TYPE_FIRE_ARROW, AT_TYPE_ICE_ARROW, AT_TYPE_LIGHT_ARROW,
    AT_TYPE_NORMAL_ARROW, Atk, AtkType, Attacker, CcAtInfo, CUT_TYPE_NONE, HitAudio,
    HitStop, NAME_ARROW, PlayerCut, Target, cc_at_check,
};
use super::enemy::Hurt;
use super::ground;
use super::player::Player;
use super::registry::ActorRegistry;
use super::ship::Ship;
use super::walk::{swept, to_s16};

pub const ARROW_ACTOR: &str = "Arrow";

pub const PARAM_NOCKED: u32 = 0;
pub const PARAM_LOOSED: u32 = 1;
pub const PARAM_WATER: u32 = 4;

pub const TYPE_NORMAL: u8 = 0;
pub const TYPE_FIRE: u8 = 1;
pub const TYPE_ICE: u8 = 2;
pub const TYPE_LIGHT: u8 = 3;

pub const MUZZLE_SPEED: f32 = 200.0;
pub const CAPSULE_RADIUS: f32 = 5.0;
pub const CAPSULE_STRETCH: f32 = 1.25;
pub const BODY_HALF: f32 = 30.0;
pub const POWER: i32 = damage::ARROW;
pub const FLINCH: u32 = 10;
pub const STUCK_FRAMES: i32 = 300;
pub const BLINK_FRAMES: i32 = 60;
pub const FIRE_WATER_FRAMES: i32 = 1;
pub const ICE_WATER_FRAMES: i32 = 10 * 30;
pub const MAGIC_COST: [i16; 4] = [0, 1, 1, 2];
pub const ROUNDS: u8 = 5;

/// `SrcObjAt Atp` for the fire and ice arrows in the reference's arrow attack table.
const ATP_FIRE_ICE: u8 = 4;
/// `SrcObjAt Atp` for the light arrow. The charge reads the damage as a signed byte, so this value
/// is negative and the light arrow takes no health off anything it strikes.
const ATP_LIGHT: u8 = 0xFF;
/// `dCcG_At_Spl_UNKB`, the special parameter a loosed light arrow's AT object takes.
const AT_SPL_UNKB: u8 = 0xB;

/// The AT type and the power one arrow kind carries, the two rows of the reference's arrow attack
/// table: the normal arrow is `POWER`, the fire and ice arrows `ATP_FIRE_ICE`, the light arrow
/// `ATP_LIGHT`.
fn arrow_attack(kind: u8) -> (AtkType, u8) {
    match kind {
        TYPE_FIRE => (AT_TYPE_FIRE_ARROW, ATP_FIRE_ICE),
        TYPE_ICE => (AT_TYPE_ICE_ARROW, ATP_FIRE_ICE),
        TYPE_LIGHT => (AT_TYPE_LIGHT_ARROW, ATP_LIGHT),
        _ => (AT_TYPE_NORMAL_ARROW, POWER as u8),
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Nocked,
    Flying,
    StuckActor,
    StuckGround,
    Drowning,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Arrow {
    pub pos: Vec3,
    pub parameter: u32,
    pub kind: u8,
    pub phase: Phase,
    pub speed: Vec3,
    pub pitch: f32,
    pub yaw: f32,
    pub round: u8,
    pub frames: i32,
    pub reflected: bool,
    pub visible: bool,
    pub hit_joint: Option<i32>,
}

impl Arrow {
    pub fn nocked(kind: u8, round: u8) -> Self {
        Self {
            pos: Vec3::ZERO,
            parameter: PARAM_NOCKED,
            kind,
            phase: Phase::Nocked,
            speed: Vec3::ZERO,
            pitch: 0.0,
            yaw: 0.0,
            round,
            frames: 0,
            reflected: false,
            visible: true,
            hit_joint: None,
        }
    }

    pub fn shoot(&mut self, pitch: f32, yaw: f32) {
        self.parameter = PARAM_LOOSED;
        self.phase = Phase::Flying;
        self.pitch = pitch;
        self.yaw = yaw;
        let x_cos = pitch.cos();
        self.speed = Vec3::new(
            MUZZLE_SPEED * yaw.sin() * x_cos,
            MUZZLE_SPEED * pitch.sin(),
            MUZZLE_SPEED * yaw.cos() * x_cos,
        );
    }

    pub fn water_frames(&self) -> i32 {
        match self.kind {
            TYPE_FIRE => FIRE_WATER_FRAMES,
            TYPE_ICE => ICE_WATER_FRAMES,
            _ => 0,
        }
    }

    pub fn drown(&mut self) {
        self.parameter = PARAM_WATER;
        self.phase = Phase::Drowning;
        self.frames = self.water_frames();
        self.visible = self.frames > 0;
    }

    pub fn stick(&mut self, phase: Phase) {
        self.phase = phase;
        self.frames = STUCK_FRAMES;
        self.speed = Vec3::ZERO;
    }

    pub fn blink(&mut self) -> bool {
        self.visible = self.frames > BLINK_FRAMES || self.frames % 2 == 0;
        self.visible
    }

}

pub fn is_arrow(name: &str) -> bool {
    name == ARROW_ACTOR
}

pub fn control(
    collision: Option<Res<super::collision::Collision>>,
    wet: Option<Res<super::water::Water>>,
    registry: Res<ActorRegistry>,
    mut arrows: Query<(Entity, &mut Arrow)>,
    mut foes: Query<
        (
            Entity,
            &Actor,
            Option<&super::jnt_hit::JntHit>,
            &mut HitPoints,
            Option<&mut Hurt>,
        ),
        Without<Ship>,
    >,
    players: Query<&Player>,
    mut flags: ResMut<super::flags::ZoneActors>,
    mut stop: ResMut<HitStop>,
    mut audio: ResMut<HitAudio>,
    mut events: ResMut<super::events::SimEvents>,
    mut fx: ResMut<super::damage_fx::DamageFxQueue>,
    mut commands: Commands,
) {
    let solid = collision.as_deref();
    let soaked = wet.as_deref().is_some_and(|water| water.in_area);
    let shooter = PlayerCut {
        cut_type: CUT_TYPE_NONE,
        cut_count: 0,
        pos: players
            .iter()
            .next()
            .map_or(Vec3::ZERO, |player| player.pos),
    };
    let mut spent: Vec<Entity> = Vec::new();
    let mut slain: Vec<(i8, u16)> = Vec::new();
    for (entity, mut arrow) in &mut arrows {
        match arrow.phase {
            Phase::Nocked => continue,
            Phase::Flying => {
                let from = arrow.pos;
                let step = arrow.speed;
                arrow.pos += step;
                let tip = arrow.pos + step * CAPSULE_STRETCH;
                let mut struck: Option<(Entity, Option<i32>, Vec3, i8, u16)> = None;
                let mut nearest = f32::INFINITY;
                for (foe, actor, colliders, _, _) in &foes {
                    if !swept(from, tip, actor.pos, CAPSULE_RADIUS + BODY_HALF) {
                        continue;
                    }
                    let distance = (actor.pos - arrow.pos).length();
                    if distance >= nearest {
                        continue;
                    }
                    // `_atHit` ignores any actor without a JntHit, then `procMove` refines
                    // the strike through `searchJntHitPosAngleOffset` to pick the joint.
                    let angle = [to_s16(arrow.pitch), to_s16(arrow.yaw), 0];
                    let contact = colliders.map(|colliders| colliders.search(angle, actor, arrow.pos));
                    let joint = contact.and_then(|(joint, _)| (joint >= 0).then_some(joint));
                    nearest = distance;
                    struck = Some((
                        foe,
                        joint,
                        contact.map_or(actor.pos, |(_, pos)| pos),
                        actor.room,
                        actor.set_id,
                    ));
                }
                let Some((foe, joint, contact, room, set_id)) = struck else {
                    if soaked && arrow.kind != TYPE_LIGHT {
                        arrow.drown();
                        continue;
                    }
                    if ground::surface(solid, arrow.pos).is_some_and(|floor| arrow.pos.y <= floor) {
                        arrow.stick(Phase::StuckGround);
                    }
                    continue;
                };
                if let Ok((_, actor, _, mut hp, mut hurt)) = foes.get_mut(foe) {
                    let (at_type, at_atp) = arrow_attack(arrow.kind);
                    let mut target = Target {
                        name: actor.name.clone(),
                        pos: actor.pos,
                        room: actor.room,
                        status: actor_status(&registry, foe),
                        health: *hp,
                        steal_item_left: actor.steal_item_left,
                        steal_item_bit_no: actor.steal_item_bit_no,
                        item_table_idx: actor.item_table_idx,
                        ..Default::default()
                    };
                    let mut info = CcAtInfo {
                        mp_obj: Some(Atk {
                            at_type,
                            at_se: AT_SE_ARROW,
                            at_spl: if arrow.kind == TYPE_LIGHT {
                                AT_SPL_UNKB
                            } else {
                                AT_SPL_UNK0
                            },
                            at_atp,
                            ac: Some(Attacker {
                                name: NAME_ARROW.to_owned(),
                                pos: arrow.pos,
                                speed: arrow.speed,
                                bomb_state: 0,
                                tsubo_type: 0,
                            }),
                        }),
                        p_particle_pos: Some(contact),
                        ..Default::default()
                    };
                    cc_at_check(
                        &mut target,
                        &mut info,
                        &shooter,
                        &mut events,
                        &mut stop,
                        &mut audio,
                        &mut fx.items,
                    );
                    hp.0 = target.health.0;
                    if let Some(hurt) = hurt.as_deref_mut() {
                        hurt.0 = FLINCH;
                    }
                    if hp.0 <= 0 {
                        slain.push((room, set_id));
                    }
                }
                arrow.hit_joint = joint;
                arrow.stick(Phase::StuckActor);
            }
            Phase::StuckActor | Phase::StuckGround => {
                arrow.frames -= 1;
                arrow.blink();
                if arrow.frames <= 0 {
                    spent.push(entity);
                }
            }
            Phase::Drowning => {
                arrow.frames -= 1;
                arrow.visible = arrow.frames > 0;
                if arrow.frames <= 0 {
                    spent.push(entity);
                }
            }
        }
    }
    for (room, set_id) in slain {
        flags.mark_done(room, set_id);
    }
    for entity in spent {
        let _ = commands.entity(entity).despawn();
    }
}