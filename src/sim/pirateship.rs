use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::area::StageName;
use super::events::{SimEvent, SimEvents};
use super::flags::StoryBits;
use super::goal_flag::StartStagePoint;
use super::walk::from_s16;

pub const PIRATE_ACTORS: [&str; 1] = ["Pirates"];

pub const UMIKZ_STAGE: &str = "A_umikz";
pub const SE_CRUISE: u32 = 0x7034;
pub const SE_SAIL: u32 = 0x50BE;

/// `dComIfGs_isEventBit(dSv_event_flag_c::UNK_0310)` (`d_a_obj_pirateship.cpp:431`, `:485`): the
/// tale told. These are SAVE event bits, so they live in [`StoryBits`] - `EventFlags` is the
/// `dEvDtFlag_c` store the event scripts test, a different resource. `aryll.rs:2105` is the write
/// that raises this one when the telescope scene finishes.
pub const STORY_TALE: u16 = 0x0310;
/// `dComIfGs_isEventBit(dSv_event_flag_c::UNK_0001)` (`:433`, `:441`, `:490`): the kidnapping, set
/// by the end of `zelda_fly` (`aryll.rs:1313`). This is the bit that puts the raiding crew on the
/// deck.
pub const STORY_KIDNAPPED: u16 = 0x0001;

pub const KIND_FIRST_ISLAND: u8 = 0;
/// `case 1:` (`:499-504`): the ship that cruises `A_umikz`, the only kind that makes waves.
pub const KIND_UMIKZ: u8 = 1;
/// `case 2:` (`:506-512`): the ship moored at Mail Island when the voyage starts from the point
/// the `arriveMj` table is written for.
pub const KIND_ARRIVAL_MJ: u8 = 2;
pub const KIND_ARRIVAL: u8 = 3;
/// `case 4:` (`:519-523`): the Interloper barge.
pub const KIND_YUUKAIGO: u8 = 4;

/// `dComIfGp_getStartStagePoint() == 0x12` (`:507`): the only stage point that mans this ship.
pub const ARRIVAL_MJ_POINT: i16 = 0x12;

pub const NO_SET_ID: u16 = 0xFFFF;

pub const CATAPULT_OFFSET: Vec3 = Vec3::new(0.0, 700.0, 850.0);

pub const CULL_MIN: Vec3 = Vec3::new(-1400.0, -200.0, -1800.0);
pub const CULL_MAX: Vec3 = Vec3::new(1400.0, 3300.0, 2200.0);

const SAIL_MASK: u32 = 0xF;
const KIND_SHIFT: u32 = 0x18;
const KIND_MASK: u32 = 0xFF;

/// One row of `daObjPirateship::Act_c::p_offset[]` (`d_a_obj_pirateship.cpp:61-139`): the child
/// actor `pirateCreate` (`:194-210`) spawns, its parameter word, and the offset and angle
/// `fopAcM_createChildFromOffset` applies in the ship's frame (`f_op_actor_mng.cpp:259-264`).
/// Each retail row ends with a small index (`0`, `2`, `0xFF`, ...) that `pirateCreate` never
/// reads - only the name, parameter, position and angle reach the create call - so it is not
/// carried here.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PirateOffset {
    pub name: &'static str,
    pub parameters: u32,
    pub pos: Vec3,
    pub angle_y: i16,
}

pub const P_OFFSET: [PirateOffset; 11] = [
    PirateOffset {
        name: "P2a",
        parameters: 0xFFFFFF00,
        pos: Vec3::new(135.5, 2200.0, 198.0),
        angle_y: 0x2900,
    },
    PirateOffset {
        name: "P2c",
        parameters: 0xFFFFFF02,
        pos: Vec3::new(275.0, 400.0, 216.0),
        angle_y: 0x4D00,
    },
    PirateOffset {
        name: "P1b",
        parameters: 0xF0100000,
        pos: Vec3::new(-150.0, 700.0, 950.0),
        angle_y: 0x4000,
    },
    PirateOffset {
        name: "P1a",
        parameters: 0xF0000000,
        pos: Vec3::new(150.0, 700.0, 950.0),
        angle_y: -0x4000,
    },
    PirateOffset {
        name: "Zl1",
        parameters: 0xFFFFFF04,
        pos: Vec3::new(-200.0, 680.0, -875.0),
        angle_y: 0,
    },
    PirateOffset {
        name: "P1b",
        parameters: 0xF0100002,
        pos: Vec3::new(163.0, 700.0, 822.0),
        angle_y: -0x4000,
    },
    PirateOffset {
        name: "P1a",
        parameters: 0xF0000002,
        pos: Vec3::new(0.0, 750.0, -860.0),
        angle_y: 0,
    },
    PirateOffset {
        name: "Zl1",
        parameters: 0xFFFFFF03,
        pos: Vec3::new(125.0, 2200.0, 100.0),
        angle_y: 0,
    },
    PirateOffset {
        name: "P2a",
        parameters: 0xFFFFFF00,
        pos: Vec3::new(-200.0, 680.0, -875.0),
        angle_y: 0,
    },
    PirateOffset {
        name: "P1b",
        parameters: 0xF0100003,
        pos: Vec3::new(215.0, 700.0, 765.0),
        angle_y: 0x3C00,
    },
    PirateOffset {
        name: "P1a",
        parameters: 0xF0000003,
        pos: Vec3::new(275.0, 740.0, -1145.0),
        angle_y: 0x1800,
    },
];

/// `create_idx_tbl_1stIsland[] = {0, 1, 2, 3, -1}` (`:140`): the crew the ship spawns at create
/// once both story bits are already up, i.e. when the Outset raid is loaded from a save that has
/// already seen it.
const CREATE_FIRST_ISLAND: [usize; 4] = [0, 1, 2, 3];
/// `create_idx_tbl_1stIsland_demo[] = {2, -1}` (`:141`): the single pirate `pirateCreate` spawns
/// the tick the kidnapping bit rises, the crew the raid cutscene brings up on deck.
const CREATE_FIRST_ISLAND_DEMO: [usize; 1] = [2];

/// `create_idx_tbl_umikz[] = {0, 1, 4, 5, 6, -1}` (`:142`): the cruising ship's full crew - two
/// `P2`s, Zelda at the wheel and two more pirates on the forecastle.
const CREATE_UMIKZ: [usize; 5] = [0, 1, 4, 5, 6];
/// `create_idx_tbl_arriveMj[] = {1, 7, 8, 9, 10, -1}` (`:143`): who stands the Mail Island arrival.
const CREATE_ARRIVE_MJ: [usize; 5] = [1, 7, 8, 9, 10];
/// `create_idx_tbl_Yuukaigo[] = {0, 1, 5, -1}` (`:144`): the barge's three.
const CREATE_YUUKAIGO: [usize; 3] = [0, 1, 5];

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Pirate {
    pub kind: u8,
    pub sail: bool,
    /// `m2CC` (`d_a_obj_pirateship.cpp:430-440`): the ship's drawn/active latch, raised by
    /// `UNK_0310` on the first execute. `Draw` returns early while it is clear (`:371-373`).
    pub visible: bool,
    /// `m2CD` (`:196`, `:442`, `:487-494`): the latch that says the crew has been created, so the
    /// demo table fires once and only once.
    pub crew: bool,
    /// The create-time crew pass. Retail runs it inside `_create` (`:484-497`), before any
    /// execute; this flag is the port's mark that the first tick has already served as that
    /// pass, so the create-time table and the demo table cannot both fire.
    pub created: bool,
    pub catapult: Option<Entity>,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Catapult {
    pub ship: Entity,
}

impl Pirate {
    pub fn spawn(parameters: u32) -> Self {
        let kind = ((parameters >> KIND_SHIFT) & KIND_MASK) as u8;
        Self {
            kind,
            sail: parameters & SAIL_MASK != 0,
            visible: kind != KIND_FIRST_ISLAND,
            crew: false,
            created: false,
            catapult: None,
        }
    }

    pub fn cull(&self) -> (Vec3, Vec3) {
        (CULL_MIN, CULL_MAX)
    }
}

fn mounted(angle: [i16; 3]) -> Vec3 {
    let (sx, cx) = from_s16(angle[0]).sin_cos();
    let (sy, cy) = from_s16(angle[1]).sin_cos();
    let (sz, cz) = from_s16(angle[2]).sin_cos();
    let off = CATAPULT_OFFSET;
    Vec3::new(
        cz * cy * off.x + (cz * sy * sx - sz * cx) * off.y + (cz * sy * cx + sz * sx) * off.z,
        sz * cy * off.x + (sz * sy * sx + cz * cx) * off.y + (sz * sy * cx - cz * sx) * off.z,
        -sy * off.x + cy * sx * off.y + cy * cx * off.z,
    )
}

pub fn is_pirateship(name: &str) -> bool {
    PIRATE_ACTORS.contains(&name)
}

/// `daObjPirateship::Act_c::pirateCreate` (`d_a_obj_pirateship.cpp:194-210`). `m2CD = 1` is set
/// before the loop in retail (`:196`); the caller owns that write so both entry points keep the
/// same order. The position and angle are the offset form of
/// `fopAcM_createChildFromOffset`'s string overload (`f_op_actor_mng.cpp:257-264`): only the
/// ship's Y angle turns the offset, and the row's angle is added to it.
fn pirate_create(table: &[usize], ship: &Actor, commands: &mut Commands) {
    let (sin, cos) = from_s16(ship.angle[1]).sin_cos();
    for &index in table {
        let row = &P_OFFSET[index];
        let pos = Vec3::new(
            ship.pos.x + row.pos.z * sin + row.pos.x * cos,
            ship.pos.y + row.pos.y,
            ship.pos.z + row.pos.z * cos - row.pos.x * sin,
        );
        let angle = ship.angle[1].wrapping_add(row.angle_y);
        let child = Actor {
            name: row.name.to_owned(),
            parameters: row.parameters,
            pos,
            angle: [0, angle, 0],
            set_id: NO_SET_ID,
            room: ship.room,
            ..Default::default()
        };
        let mut spawned = commands.spawn(child);
        // The crew never passes through `spawn_actor`, so each row's actor module has to come with
        // it - the `1stIsland` table is two `P2`s, a `P1b` and a `P1a`, and the other tables add
        // `Zl1`. Without the module the row lands as a bare `Actor` that animates nothing.
        if super::npc_p2::is_npc_p2(row.name) {
            spawned.insert(super::npc_p2::NpcP2::spawn(row.parameters, pos, angle));
        } else if super::npc_p1::is_npc_p1(row.name) {
            spawned.insert(super::npc_p1::NpcP1::spawn(row.parameters));
        } else if row.name == super::zelda::ZELDA_ACTOR {
            spawned.insert(super::zelda::Zelda::spawn(row.parameters));
        }
    }
}

pub fn control(
    stage: Res<StageName>,
    stories: Res<StoryBits>,
    start_point: Res<StartStagePoint>,
    mut events: ResMut<SimEvents>,
    mut ships: Query<(Entity, &Actor, &mut Pirate), With<Pirate>>,
    mut guns: Query<(&mut Actor, &Catapult), Without<Pirate>>,
    mut commands: Commands,
) {
    for (entity, actor, mut pirate) in &mut ships {
        // `_create`'s switch on the parameter's top byte (`:483-528`). `m2CD` says the crew pass
        // has run, so every arm except `case 0:` raises it even when it spawns nobody.
        if !pirate.created {
            pirate.created = true;
            match pirate.kind {
                // `case 0:` (`:484-497`): with the tale told, a ship loaded from a save that
                // already has the kidnapping spawns the whole `1stIsland` crew up front.
                KIND_FIRST_ISLAND => {
                    if stories.test(STORY_TALE) && stories.test(STORY_KIDNAPPED) {
                        pirate_create(&CREATE_FIRST_ISLAND, actor, &mut commands);
                        pirate.crew = true;
                    }
                }
                // `case 1:` (`:499-504`): `CreateWave()` here is the particle emitter the port
                // never creates, so there is no wake to raise on this tick either.
                KIND_UMIKZ => {
                    pirate_create(&CREATE_UMIKZ, actor, &mut commands);
                    pirate.crew = true;
                }
                // `case 2:` (`:506-512`): the crew is there only for the `0x12` arrival point;
                // the ship itself is manned either way.
                KIND_ARRIVAL_MJ => {
                    if start_point.0 == ARRIVAL_MJ_POINT {
                        pirate_create(&CREATE_ARRIVE_MJ, actor, &mut commands);
                    }
                    pirate.crew = true;
                }
                // `case 4:` (`:519-523`); `case 3:` and `default:` (`:514-517`, `:525-527`) crew
                // nobody at all.
                KIND_YUUKAIGO => {
                    pirate_create(&CREATE_YUUKAIGO, actor, &mut commands);
                    pirate.crew = true;
                }
                _ => pirate.crew = true,
            }
        }
        if pirate.catapult.is_none() && pirate.kind != KIND_ARRIVAL {
            let child = Actor {
                name: super::touseki::TOUSEKI_ACTORS[0].to_owned(),
                parameters: 0,
                pos: actor.pos + mounted(actor.angle),
                angle: actor.angle,
                set_id: NO_SET_ID,
                room: actor.room,
                ..Default::default()
            };
            pirate.catapult = Some(commands.spawn((child, Catapult { ship: entity })).id());
        }
        for (mut mount, gun) in &mut guns {
            if gun.ship != entity {
                continue;
            }
            mount.pos = actor.pos + mounted(actor.angle);
            mount.angle = actor.angle;
        }
        // `if (m2CC == 0) { if (UNK_0310) m2CC = 1; else return false; }` (`:430-440`): the ship
        // stays inert until the tale bit is up, and the wake tick itself never runs the crew
        // branch below - retail chains it as an `else if`.
        if !pirate.visible {
            if !stories.test(STORY_TALE) {
                continue;
            }
            pirate.visible = true;
        // `else if (m2CD == 0 && dComIfGs_isEventBit(UNK_0001))` (`:441-446`): the raid's crew
        // arrives the tick the kidnapping bit rises. `DeleteWave()` (`:445`) has no subject here:
        // `CreateWave` is a particle emitter the port never creates.
        } else if pirate.kind == KIND_FIRST_ISLAND && !pirate.crew && stories.test(STORY_KIDNAPPED)
        {
            pirate.crew = true;
            pirate_create(&CREATE_FIRST_ISLAND_DEMO, actor, &mut commands);
        }
        if pirate.kind == KIND_FIRST_ISLAND {
            continue;
        }
        if stage.0 == UMIKZ_STAGE {
            events.list.push(SimEvent::Se(SE_CRUISE));
        }
        if pirate.sail {
            events.list.push(SimEvent::Se(SE_SAIL));
        }
    }
}
