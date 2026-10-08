use std::time::Duration;

use clearsleeper::SIM_HZ;
use clearsleeper::dzs::{self, StageActor};
use clearsleeper::sim;
use clearsleeper::sim::actor::Actor;
use glam::Vec3;
use repame_sim::Sim;

fn actr_stage() -> Vec<u8> {
    let actr_at = 4 + 12;
    let mut data = vec![0_u8; actr_at + 2 * 0x20];
    write_u32(&mut data, 0, 1);
    data[4..8].copy_from_slice(b"ACTR");
    write_u32(&mut data, 8, 2);
    write_u32(&mut data, 12, actr_at as u32);

    let ship = actr_at;
    data[ship..ship + 8].copy_from_slice(b"Ship\0\0\0\0");
    write_u32(&mut data, ship + 0x08, 0x0000_FF00);
    data[ship + 0x0C..ship + 0x10].copy_from_slice(&19200.0_f32.to_be_bytes());
    data[ship + 0x10..ship + 0x14].copy_from_slice(&0.0_f32.to_be_bytes());
    data[ship + 0x14..ship + 0x18].copy_from_slice(&(-2200.0_f32).to_be_bytes());
    data[ship + 0x1E..ship + 0x20].copy_from_slice(&0xFFFF_u16.to_be_bytes());

    let sea = actr_at + 0x20;
    data[sea..sea + 8].copy_from_slice(b"sea\0\0\0\0\0");
    write_u32(&mut data, sea + 0x08, 0x0001_0000);
    data[sea + 0x1A..sea + 0x1C].copy_from_slice(&(-3094_i16).to_be_bytes());
    data[sea + 0x1E..sea + 0x20].copy_from_slice(&0xFFFF_u16.to_be_bytes());
    data
}

fn write_u32(data: &mut [u8], at: usize, value: u32) {
    data[at..at + 4].copy_from_slice(&value.to_be_bytes());
}

fn fresh() -> Sim {
    Sim::new(Duration::from_secs_f64(1.0 / SIM_HZ))
}

#[test]
fn golden_stage_actor_table_parses_records() {
    let tables = dzs::parse(&actr_stage()).expect("stage tables");
    let actors = tables.actors();
    assert_eq!(actors.len(), 2);
    assert_eq!(
        actors[0],
        StageActor {
            name: "Ship".to_string(),
            parameters: 0x0000_FF00,
            pos: [19200.0, 0.0, -2200.0],
            angle: [0, 0, 0],
            set_id: 0xFFFF,
            room: dzs::NO_ROOM,
            scale: [1.0, 1.0, 1.0],
        }
    );
    assert_eq!(
        actors[1],
        StageActor {
            name: "sea".to_string(),
            parameters: 0x0001_0000,
            pos: [0.0, 0.0, 0.0],
            angle: [0, -3094, 0],
            set_id: 0xFFFF,
            room: dzs::NO_ROOM,
            scale: [1.0, 1.0, 1.0],
        }
    );
    assert!(
        actors.iter().all(|actor| actor.room == dzs::NO_ROOM),
        "stage table actors belong to no room"
    );
    assert!(tables.has_sea_actor());
}

#[test]
fn golden_sim_spawns_stage_actors() {
    let tables = dzs::parse(&actr_stage()).expect("stage tables");
    let mut built = fresh();
    sim::build(&mut built, None, Some(&tables));
    let mut query = built.world.query::<&Actor>();
    let mut spawned: Vec<Actor> = query
        .iter(&built.world)
        .map(|actor| actor.clone())
        .collect();
    spawned.sort_by(|left, right| left.name.cmp(&right.name));
    assert_eq!(spawned.len(), 2);
    assert_eq!(
        spawned[0],
        Actor {
            name: "Ship".to_string(),
            parameters: 0x0000_FF00,
            pos: Vec3::new(19200.0, 0.0, -2200.0),
            angle: [0, 0, 0],
            set_id: 0xFFFF,
            room: -1,
            weight_uc: 0xF0,
            ..Default::default()
        }
    );
    assert_eq!(
        spawned[1],
        Actor {
            name: "sea".to_string(),
            parameters: 0x0001_0000,
            pos: Vec3::ZERO,
            angle: [0, -3094, 0],
            set_id: 0xFFFF,
            room: -1,
            ..Default::default()
        }
    );

    let mut bare = fresh();
    sim::build(&mut bare, None, None);
    let mut query = bare.world.query::<&Actor>();
    assert_eq!(query.iter(&bare.world).count(), 0);
}
