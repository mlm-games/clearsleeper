use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::flags::Switches;
use super::health::Health;
use super::player::Player;
use super::walk::{add_calc_angle_s, from_s16, to_s16};

pub const SHIP_ACTOR: &str = "Oship";

const SWITCH_SHIFT: u32 = 24;
const HOME_SWITCH_MASK: u16 = 0xFF;
/// The three bands the ship's own table names (`d_a_oship.cpp:65-67`):
/// `mDistRangeA`, `mDistRangeB` and `mDistRangeC`.
const RANGE_A: f32 = 2500.0;
const RANGE_B: f32 = 6000.0;
const RANGE_C: f32 = 12000.0;
/// `mAttackDelayA = 0x1e` (`d_a_oship.cpp:68`) and `mAttackDelayB = 200` (`:69`).
const ATTACK_DELAY_A: u32 = 30;
const ATTACK_DELAY_B: u32 = 200;
const CANNON_OFFSET: f32 = 450.0;
const CANNON_SPEED: f32 = 85.0;
const CANNON_GRAVITY: f32 = -2.0;
const CANNON_FREE: u32 = 35;
const CANNON_LIFE: u32 = 240;
const CANNON_REACH: f32 = 40.0;
const CANNON_POWER: u8 = 4;
const SAIL_SPEED: f32 = 20.0;
const SAIL_SCALE: f32 = 0.25;
const TURN_SCALE: i16 = 2;
const TURN_MAX: i16 = 0x100;
const AIM_SCALE: i16 = 6;
const AIM_MAX: i16 = 0x300;
const BODY_HEIGHT: f32 = 300.0;
const BODY_HALF: f32 = 300.0;

pub const BODY: (f32, f32) = (BODY_HEIGHT, BODY_HALF);

/// The four modes `changeModeByRange` picks between (`d_a_oship.cpp:204-221`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Band {
    /// `MODE_RANGE_A` - inside `mDistRangeA`.
    A,
    /// `MODE_RANGE_B` - from `mDistRangeA` up to `mDistRangeB`.
    B,
    /// `MODE_RANGE_C` - from `mDistRangeB` up to `mDistRangeC`.
    C,
    /// `MODE_RANGE_D` - past `mDistRangeC`; the mode that never arms the cannon.
    D,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Warship {
    pub home_yaw: i16,
    pub aim: i16,
    pub timer: u32,
    /// The mode the ship is in, `None` until the first frame has read the distance to
    /// the player.
    pub band: Option<Band>,
    /// `mSubMode = prms & 0xFF` (`d_a_oship.cpp:996`).
    pub submode: u8,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Cannon {
    pub pos: Vec3,
    pub vel: Vec3,
    pub free: u32,
    pub life: u32,
}

/// `daOship_c::_create` (`d_a_oship.cpp:1021-1027`): the away switch (`prms >> 24`, `:1000`) set
/// refuses the ship, and a home switch (`home.angle.x & 0xFF`, `:1004`) that is present yet clear
/// refuses it too, so a clear away switch and an absent home switch are what let it sail.
pub fn is_switched(parameters: u32, home_x: i16, switches: &Switches) -> bool {
    let away = ((parameters >> SWITCH_SHIFT) & 0xFF) as u8;
    let home = (home_x as u16 & HOME_SWITCH_MASK) as u8;
    if away != 0xFF && switches.test(away) {
        return false;
    }
    home == 0xFF || switches.test(home)
}

pub fn spawn(home_yaw: i16, parameters: u32) -> Warship {
    Warship {
        home_yaw,
        aim: home_yaw,
        timer: 0,
        band: None,
        submode: (parameters & 0xFF) as u8,
    }
}

/// The band the distance to the player falls in (`changeModeByRange`,
/// `d_a_oship.cpp:204-221`), out of the three the ship's own table names
/// (`d_a_oship.cpp:65-67`).
fn band_for(dist: f32) -> Band {
    if dist < RANGE_A {
        Band::A
    } else if dist < RANGE_B {
        Band::B
    } else if dist < RANGE_C {
        Band::C
    } else {
        Band::D
    }
}

/// The delay a band's init arms the attack timer to: `modeRangeAInit` rests it on
/// `mAttackDelayA` (`d_a_oship.cpp:68`, `:516-518`), `modeRangeBInit` on the same
/// when the ship's sub-mode is zero and on `mAttackDelayB` otherwise (`:540-547`),
/// and `modeRangeCInit` on `mAttackDelayB` (`:569-571`). `modeRangeDInit`
/// (`:592-595`) is empty, so the outer band arms nothing.
fn arm(band: Band, submode: u8) -> Option<u32> {
    match band {
        Band::A => Some(ATTACK_DELAY_A),
        Band::B => Some(if submode == 0 {
            ATTACK_DELAY_A
        } else {
            ATTACK_DELAY_B
        }),
        Band::C => Some(ATTACK_DELAY_B),
        Band::D => None,
    }
}

fn fire(ship: &Actor, warship: &Warship, target: Vec3, commands: &mut Commands) {
    let yaw = from_s16(warship.aim);
    let flat = Vec3::new(target.x - ship.pos.x, 0.0, target.z - ship.pos.z);
    let range = flat.length().max(1.0);
    // `modeRangeA` aims at the player's `current.pos` (`d_a_oship.cpp:527`, `:556`, `:580`, `:632`)
    // and `attackCannon` launches along that aim (`:457-459`), so the shot flies at the feet.
    let pitch = (target.y - ship.pos.y).atan2(range);
    let (sin, cos) = pitch.sin_cos();
    let pos = ship.pos + Vec3::new(CANNON_OFFSET * yaw.sin(), 0.0, CANNON_OFFSET * yaw.cos());
    commands.spawn(Cannon {
        pos,
        vel: Vec3::new(
            CANNON_SPEED * cos * yaw.sin(),
            CANNON_SPEED * sin,
            CANNON_SPEED * cos * yaw.cos(),
        ),
        free: CANNON_FREE,
        life: CANNON_LIFE,
    });
}

pub fn control(
    mut health: ResMut<Health>,
    players: Query<&Player>,
    mut ships: Query<(&mut Actor, &mut Warship)>,
    mut shots: Query<(Entity, &mut Cannon)>,
    mut commands: Commands,
) {
    let Some(player) = players.iter().next().copied() else {
        return;
    };
    for (mut ship, mut warship) in &mut ships {
        let dx = player.pos.x - ship.pos.x;
        let dz = player.pos.z - ship.pos.z;
        let dist = (dx * dx + dz * dz).sqrt();
        ship.angle[1] = add_calc_angle_s(ship.angle[1], warship.home_yaw, TURN_SCALE, TURN_MAX, 0);
        if dist < RANGE_C {
            warship.aim =
                add_calc_angle_s(warship.aim, to_s16(dx.atan2(dz)), AIM_SCALE, AIM_MAX, 0);
        }
        let yaw = from_s16(ship.angle[1]);
        ship.pos.x += SAIL_SPEED * SAIL_SCALE * yaw.sin();
        ship.pos.z += SAIL_SPEED * SAIL_SCALE * yaw.cos();
        let band = band_for(dist);
        let delay = arm(band, warship.submode);
        match warship.band {
            // The first frame only reads the band. The port's cannon starts loaded - the
            // timer is zero - where the reference waits out `mAttackDelayA` in its first
            // mode (`d_a_oship.cpp:516-518`).
            None => warship.band = Some(band),
            Some(current) if current != band => {
                // `changeModeByRange` re-initialises the mode whenever the band changes
                // (`d_a_oship.cpp:218-219`), and each band's init arms the attack timer
                // (`:516-518`, `:540-547`, `:569-571`).
                warship.band = Some(band);
                if let Some(delay) = delay {
                    warship.timer = delay;
                }
            }
            Some(_) => {}
        }
        let Some(delay) = delay else {
            // `modeRangeD` (`:598-607`) only watches the player: it neither counts the
            // attack timer down nor arms the cannon.
            continue;
        };
        if warship.timer > 0 {
            warship.timer -= 1;
            continue;
        }
        fire(&ship, &warship, player.pos, &mut commands);
        warship.timer = delay;
    }
    for (entity, mut shot) in &mut shots {
        if shot.free > 0 {
            shot.free -= 1;
        } else {
            shot.vel.y += CANNON_GRAVITY;
        }
        let step = shot.vel;
        shot.pos += step;
        shot.life = shot.life.saturating_sub(1);
        if shot.pos.distance(player.pos) <= CANNON_REACH {
            health.damage(CANNON_POWER);
            let _ = commands.entity(entity).despawn();
            continue;
        }
        if shot.life == 0 || shot.pos.y < player.pos.y - 400.0 {
            let _ = commands.entity(entity).despawn();
        }
    }
}
