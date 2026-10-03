use clearsleeper::dzs::{self, DzsError, Placement};

fn synthetic() -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(&2_u32.to_be_bytes());
    data.extend_from_slice(b"MULT");
    data.extend_from_slice(&0_u32.to_be_bytes());
    data.extend_from_slice(&0_u32.to_be_bytes());
    data.extend_from_slice(b"RTBL");
    data.extend_from_slice(&0_u32.to_be_bytes());
    data.extend_from_slice(&0_u32.to_be_bytes());

    let mult_at = data.len();
    data.extend_from_slice(&100.0_f32.to_be_bytes());
    data.extend_from_slice(&(-200.0_f32).to_be_bytes());
    data.extend_from_slice(&0x4000_i16.to_be_bytes());
    data.push(7);
    data.push(3);
    data.extend_from_slice(&0.0_f32.to_be_bytes());
    data.extend_from_slice(&0.0_f32.to_be_bytes());
    data.extend_from_slice(&0_i16.to_be_bytes());
    data.push(44);
    data.push(255);

    let entries_at = data.len();
    data.extend_from_slice(&0_u32.to_be_bytes());
    data.extend_from_slice(&0_u32.to_be_bytes());

    let record0 = data.len();
    data.extend_from_slice(&[1, 9, 2, 0]);
    data.extend_from_slice(&0_u32.to_be_bytes());
    let rooms0 = data.len();
    data.push(5 | 0x80);

    let record1 = data.len();
    data.extend_from_slice(&[2, 0, 1, 0]);
    data.extend_from_slice(&0_u32.to_be_bytes());
    let rooms1 = data.len();
    data.push(0x80);
    data.push(44 | 0x80);

    write_u32(&mut data, 8, 2);
    write_u32(&mut data, 12, mult_at as u32);
    write_u32(&mut data, 20, 2);
    write_u32(&mut data, 24, entries_at as u32);
    write_u32(&mut data, entries_at, record0 as u32);
    write_u32(&mut data, entries_at + 4, record1 as u32);
    write_u32(&mut data, record0 + 4, rooms0 as u32);
    write_u32(&mut data, record1 + 4, rooms1 as u32);
    data
}

fn kankyo() -> Vec<u8> {
    let counts = [1_u32, 2, 3, 6, 2, 2];
    let tags: [&[u8; 4]; 6] = [b"STAG", b"EnvR", b"Colo", b"Pale", b"Virt", b"ACTR"];
    let stag_at = 4 + counts.len() * 12;
    let envr_at = stag_at + 0x20;
    let colo_at = envr_at + 2 * 0x08;
    let pale_at = colo_at + 3 * 0x0C;
    let virt_at = pale_at + 6 * 0x2C;
    let actr_at = virt_at + 2 * 0x24;
    let offsets = [stag_at, envr_at, colo_at, pale_at, virt_at, actr_at];
    let mut data = vec![0_u8; actr_at + 2 * 0x20];
    write_u32(&mut data, 0, counts.len() as u32);
    for (index, tag) in tags.into_iter().enumerate() {
        let node = 4 + index * 12;
        data[node..node + 4].copy_from_slice(tag);
        write_u32(&mut data, node + 4, counts[index]);
        write_u32(&mut data, node + 8, offsets[index] as u32);
    }
    data[stag_at + 0x0D] = 3;
    data[envr_at] = 1;
    data[colo_at + 0x0C + 5] = 5;
    let pale = pale_at + 5 * 0x2C;
    data[pale + 0x0C..pale + 0x0F].copy_from_slice(&[255, 255, 255]);
    data[pale + 0x0F..pale + 0x12].copy_from_slice(&[9, 99, 224]);
    data[pale + 0x21] = 1;
    let virt = virt_at + 0x24;
    data[virt + 0x18..virt + 0x1B].copy_from_slice(&[10, 20, 30]);
    data[actr_at..actr_at + 8].copy_from_slice(b"Ship\0\0\0\0");
    data[actr_at + 0x20..actr_at + 0x28].copy_from_slice(b"sea\0\0\0\0\0");
    data
}

fn write_u32(data: &mut [u8], at: usize, value: u32) {
    data[at..at + 4].copy_from_slice(&value.to_be_bytes());
}

fn read_u32(data: &[u8], at: usize) -> u32 {
    u32::from_be_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]])
}

#[test]
fn golden_stage_tables_parse_placements_and_room_groups() {
    let tables = dzs::parse(&synthetic()).unwrap();
    let placements = tables.placements();
    assert_eq!(placements.len(), 2);
    assert_eq!(
        placements[0],
        Placement {
            trans: [100.0, -200.0],
            angle: 0x4000,
            room: 7,
            wave_max: 3,
        }
    );
    assert_eq!(
        placements[1],
        Placement {
            trans: [0.0, 0.0],
            angle: 0,
            room: 44,
            wave_max: 255,
        }
    );
    assert_eq!(tables.room_groups().len(), 2);
    assert_eq!(tables.room_groups()[0], vec![5_u8]);
    assert_eq!(tables.room_groups()[1], vec![0_u8, 44_u8]);
    assert_eq!(tables.placement(7).unwrap().trans, [100.0, -200.0]);
    assert!(tables.placement(9).is_none());
    assert_eq!(tables.load_set(0), &[5_u8]);
    assert_eq!(tables.load_set(1), &[0_u8, 44_u8]);
    assert!(tables.load_set(2).is_empty());
}

#[test]
fn golden_room_at_maps_sea_chart_cells() {
    let tables = dzs::parse(&synthetic()).unwrap();
    assert_eq!(tables.room_at(0.0, 0.0), 25);
    assert_eq!(tables.room_at(-194_845.4, 314_242.8), 44);
    assert_eq!(tables.room_at(-200_000.0, 300_000.0), 44);
    assert_eq!(tables.room_at(-150_000.0, 300_000.0), 45);
    assert_eq!(tables.room_at(-150_000.1, 300_000.0), 44);
    assert_eq!(tables.room_at(-350_000.0, -350_000.0), 1);
    assert_eq!(tables.room_at(349_999.0, -350_000.0), 7);
    assert_eq!(tables.room_at(-400_000.0, 0.0), 0);
    assert_eq!(tables.room_at(400_000.0, 0.0), 0);
    assert_eq!(tables.room_at(0.0, 400_000.0), 0);

    let empty = dzs::parse(&0_u32.to_be_bytes()).unwrap();
    assert_eq!(empty.room_at(-200_000.0, 300_000.0), 0);
}

#[test]
fn golden_stage_tables_kankyo_chain_and_wave_max() {
    let tables = dzs::parse(&kankyo()).unwrap();
    assert_eq!(tables.boot_time(), 45.0);
    assert!(tables.has_sea_actor());
    assert_eq!(
        tables.boot_sky(),
        Some([10.0 / 255.0, 20.0 / 255.0, 30.0 / 255.0, 1.0])
    );
    assert_eq!(
        tables.boot_sea_colors(),
        Some((
            [1.0, 1.0, 1.0],
            [9.0 / 255.0, 99.0 / 255.0, 224.0 / 255.0],
        ))
    );

    let stage = dzs::parse(&synthetic()).unwrap();
    assert_eq!(stage.boot_time(), 165.0);
    assert!(!stage.has_sea_actor());
    assert_eq!(stage.boot_sky(), None);
    assert_eq!(stage.boot_sea_colors(), None);
    assert_eq!(stage.wave_max(7), 3);
    assert_eq!(stage.wave_max(44), 255);
    assert_eq!(stage.wave_max(9), 10);
}

#[test]
fn rejects_truncated_and_malformed_stage_streams() {
    assert_eq!(dzs::parse(&[]).unwrap_err(), DzsError::Truncated);

    let huge_count = u32::MAX.to_be_bytes().to_vec();
    assert_eq!(dzs::parse(&huge_count).unwrap_err(), DzsError::BadTable);

    let mut short_node = 1_u32.to_be_bytes().to_vec();
    short_node.extend_from_slice(b"MULT");
    assert_eq!(dzs::parse(&short_node).unwrap_err(), DzsError::BadTable);

    let mut bad_mult = synthetic();
    write_u32(&mut bad_mult, 12, 0xFFFF_0000);
    assert_eq!(dzs::parse(&bad_mult).unwrap_err(), DzsError::BadTable);

    let mut bad_rtbl = synthetic();
    write_u32(&mut bad_rtbl, 24, 0xFFFF_0000);
    assert_eq!(dzs::parse(&bad_rtbl).unwrap_err(), DzsError::BadTable);

    let mut bad_record = synthetic();
    let entries_at = read_u32(&bad_record, 24) as usize;
    write_u32(&mut bad_record, entries_at, 0xFFFF_FFF0);
    assert_eq!(dzs::parse(&bad_record).unwrap_err(), DzsError::BadRecord);

    let mut bad_rooms = synthetic();
    let record0 = read_u32(&bad_rooms, entries_at) as usize;
    write_u32(&mut bad_rooms, record0 + 4, 0xFFFF_FFF0);
    assert_eq!(dzs::parse(&bad_rooms).unwrap_err(), DzsError::BadRecord);

    let mut bad_stag = kankyo();
    write_u32(&mut bad_stag, 12, 0xFFFF_0000);
    assert_eq!(dzs::parse(&bad_stag).unwrap_err(), DzsError::BadTable);

    let mut bad_actr = kankyo();
    write_u32(&mut bad_actr, 72, 0xFFFF_0000);
    assert_eq!(dzs::parse(&bad_actr).unwrap_err(), DzsError::BadTable);
}

fn exits() -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(&1_u32.to_be_bytes());
    data.extend_from_slice(b"SCLS");
    data.extend_from_slice(&2_u32.to_be_bytes());
    data.extend_from_slice(&16_u32.to_be_bytes());
    data.extend_from_slice(b"House01\0");
    data.push(3);
    data.push(0);
    data.push(1);
    data.push(0);
    data.extend_from_slice(b"sea\0\0\0\0\0");
    data.push(128);
    data.push(5);
    data.push(15);
    data.push(0);
    data
}

#[test]
fn golden_scls_exit_table_parses_destinations() {
    let tables = dzs::parse(&exits()).unwrap();
    let house = tables.exit(0).expect("first exit resolves");
    assert_eq!(house.stage, "House01");
    assert_eq!(house.start, 3);
    assert_eq!(house.room, 0);
    assert_eq!(house.wipe, 1);
    let sea = tables.exit(1).expect("second exit resolves");
    assert_eq!(sea.stage, "sea");
    assert_eq!(sea.start, 128);
    assert_eq!(sea.room, 5);
    assert_eq!(sea.wipe, 15);
    assert!(
        tables.exit(2).is_none(),
        "ids past the table do not resolve"
    );
    assert_eq!(tables.with_name("House01").name(), "House01");
}
