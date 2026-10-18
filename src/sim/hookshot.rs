use bevy_ecs::prelude::*;

use glam::Vec3;

use super::actor::Actor;
use super::carry::Carryable;
use super::collision::Collision;
use super::ground;
use super::player::Player;
use super::registry::ActorRegistry;

pub const HOOKSHOT_ACTOR: &str = "HShot";

pub const CAPSULE_RADIUS: f32 = 5.0;
pub const SHOT_SPEED: f32 = 7.0;
pub const SHOT_STEP: f32 = 15.0;
pub const REACH: f32 = 1500.0;
pub const LINK: f32 = 7.0;
pub const SPOOL: f32 = 9.0;
pub const SPOOL_PULL: f32 = SPOOL * LINK - LINK;
pub const CHAIN_MAX: f32 = 18.0;
pub const LIFT: f32 = 100.0;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Wait,
    Shot,
    Return,
    Pull,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Hookshot {
    pub phase: Phase,
    pub pos: Vec3,
    pub root: Vec3,
    pub heading: Vec3,
    pub carry: Option<Entity>,
    pub carry_offset: Vec3,
    pub chain: f32,
}

impl Hookshot {
    pub fn nocked(root: Vec3) -> Self {
        Self {
            phase: Phase::Wait,
            pos: root,
            root,
            heading: Vec3::Z,
            carry: None,
            carry_offset: Vec3::ZERO,
            chain: 0.0,
        }
    }

    pub fn launch(&mut self, pitch: f32, yaw: f32) {
        self.phase = Phase::Shot;
        let x_cos = pitch.cos();
        self.heading = Vec3::new(yaw.sin() * x_cos, -pitch.sin(), yaw.cos() * x_cos) * SHOT_SPEED;
        self.measure();
    }

    pub fn stow(&mut self, root: Vec3) {
        self.phase = Phase::Wait;
        self.pos = root;
        self.carry = None;
        self.carry_offset = Vec3::ZERO;
        self.chain = 0.0;
    }

    pub fn measure(&mut self) {
        self.chain = (self.pos - self.root).length() / LINK;
    }

    pub fn spent(&self) -> bool {
        self.phase == Phase::Wait
    }

    pub fn over_extended(&self) -> bool {
        self.chain > CHAIN_MAX
    }

    pub fn beyond_reach(&self) -> bool {
        (self.pos - self.root).length() >= REACH
    }

    pub fn hooked(&mut self, target: Entity, offset: Vec3) {
        self.phase = Phase::Pull;
        self.carry = Some(target);
        self.carry_offset = offset;
    }

    pub fn anchored(&mut self, target: Entity, pos: Vec3) {
        self.phase = Phase::Pull;
        self.pos = pos;
        self.carry = Some(target);
        self.carry_offset = Vec3::ZERO;
        self.heading = Vec3::ZERO;
    }

    pub fn reel(&mut self) {
        self.phase = Phase::Return;
    }

    pub fn step(&mut self) -> Vec3 {
        let next = self.pos + self.heading * SHOT_STEP;
        self.pos = next;
        next
    }

    pub fn spool(&mut self) -> bool {
        let delta = self.pos - self.root;
        let reach = delta.length();
        if self.chain <= SPOOL || reach / LINK <= SPOOL {
            return true;
        }
        self.chain -= SPOOL;
        self.pos -= delta * (SPOOL_PULL / reach);
        false
    }

    pub fn haul(&mut self) -> Vec3 {
        let delta = self.pos - self.root;
        let reach = delta.length();
        let links = (reach / LINK) as i32;
        if links < 1 {
            self.chain = 0.0;
            return Vec3::ZERO;
        }
        let slack = (links as f32 - self.chain).max(0.0);
        let taken = slack
            + if self.chain <= SPOOL {
                self.chain
            } else {
                SPOOL
            };
        self.chain -= taken;
        let move_by = delta * ((taken * LINK) / reach);
        self.heading = move_by;
        move_by
    }
}

pub fn is_hookshot(name: &str) -> bool {
    name == HOOKSHOT_ACTOR
}

pub fn control(
    collision: Option<Res<Collision>>,
    registry: Option<ResMut<ActorRegistry>>,
    mut hooks: Query<(Entity, &mut Hookshot)>,
    carryables: Query<(Entity, &Actor, &Carryable)>,
    targets: Query<(Entity, &Actor)>,
    mut players: Query<&mut Player>,
    mut commands: Commands,
) {
    let solid = collision.as_deref();
    let Some(mut player) = players.iter_mut().next() else {
        return;
    };
    let root = player.pos + Vec3::new(0.0, 40.0, 0.0);
    let mut spent: Vec<Entity> = Vec::new();
    let mut haul = Vec3::ZERO;
    let mut latched: Option<Entity> = None;
    let mut released: Vec<Entity> = Vec::new();
    for (entity, mut hook) in &mut hooks {
        hook.root = root;
        match hook.phase {
            Phase::Wait => hook.pos = root,
            Phase::Shot => {
                hook.measure();
                let target = carryables
                    .iter()
                    .find(|(_, actor, _)| {
                        swept(
                            hook.pos,
                            hook.pos + hook.heading * SHOT_STEP,
                            actor.pos,
                            CAPSULE_RADIUS,
                        )
                    })
                    .map(|(target, actor, _)| (target, actor.pos));
                match target {
                    Some((target, pos)) => {
                        let offset = pos - hook.pos;
                        hook.hooked(target, offset);
                        latched = Some(target);
                    }
                    None if hook.over_extended() || hook.beyond_reach() => hook.reel(),
                    None => {
                        hook.step();
                    }
                }
            }
            Phase::Pull => {
                hook.measure();
                if hook.chain <= 0.0 {
                    hook.stow(root);
                    continue;
                }
                if let Some(target) = hook.carry
                    && let Ok((_, actor)) = targets.get(target)
                {
                    hook.pos = actor.pos + hook.carry_offset;
                }
                haul = hook.haul();
            }
            Phase::Return => {
                if hook.spool() {
                    hook.stow(root);
                    continue;
                }
                if let Some(target) = hook.carry
                    && let Ok((_, actor)) = targets.get(target)
                {
                    let carried = hook.pos + hook.carry_offset;
                    let mut probe = carried;
                    probe.y += LIFT * 2.0;
                    if let Some(floor) = ground::surface(solid, probe)
                        && floor > carried.y
                    {
                        hook.carry_offset.y = floor - hook.pos.y;
                    }
                    let _ = actor;
                }
            }
        }
        if hook.spent() {
            if let Some(pin) = hook.carry {
                released.push(pin);
            }
            spent.push(entity);
        }
    }
    player.pos += haul;
    if let Some(mut registry) = registry {
        if let Some(target) = latched
            && let Some(proc_id) = registry.proc_id(target)
        {
            registry.set_hook_carry_now(proc_id);
        }
        for target in released {
            if let Some(proc_id) = registry.proc_id(target) {
                registry.cancel_hook_carry_now(proc_id);
            }
        }
    }
    for entity in spent {
        let _ = commands.entity(entity).despawn();
    }
}

pub fn swept(from: Vec3, to: Vec3, target: Vec3, radius: f32) -> bool {
    let travel = to - from;
    let length_squared = travel.length_squared();
    if length_squared < 1e-8 {
        return (target - from).length_squared() <= radius * radius;
    }
    let along = ((target - from).dot(travel) / length_squared).clamp(0.0, 1.0);
    (from + travel * along - target).length_squared() <= radius * radius
}
