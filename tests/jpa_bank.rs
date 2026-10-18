//! The JPA resource bank: the archive walk `JPAEmitterArchiveLoader_v10::load`
//! (`JPAEmitterLoader.cpp:97-186`) performs, the lookup `JPAEmitterResource::getByUserIndex`
//! answers (`JPAResourceManager.cpp:60-67`), and the ids `createSimpleEmitterID`
//! (`JPAEmitterManager.cpp:52-78`) writes onto the emitter it creates.

use clearsleeper::sim::jpa::{Jpa, JpaGap};
use clearsleeper::sim::jpa_bank::JpaResourceRef;
use clearsleeper::sim::jpa_bank::{
    ARCHIVE_HEADER_SIZE, BLOCK_FIELD, BLOCK_KEY, BLOCK_TEX_DATA, JpaBank, JpaBankError,
    PARTICLE_HEADER_SIZE, TEXTURE_MAGIC,
};
use glam::Vec3;

/// One particle header's worth of fixture: the ids `JPAEmitterParticleHeader_v10` carries
/// (`JPAEmitterLoader.cpp:31-41`) and the blocks its `blockNum` walks.
struct Emitter {
    user_index: u16,
    key_num: u8,
    fld_num: u8,
    texture_num: u8,
    blocks: &'static [(u32, u32)],
}

/// A synthetic `JPAC` archive: the `JPAEmitterArchiveData_v10` header, one `JEFF` header and its
/// blocks per emitter, then one `TEX1` entry per texture size. Each block's size covers its own
/// `JPAEmitterBlockHeader_v10`, which is what advances the reference's walk
/// (`JPAEmitterLoader.cpp:171`).
fn archive(emitters: &[Emitter], textures: &[u32]) -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(b"JPAC");
    data.extend_from_slice(b"1-00");
    data.extend_from_slice(&(emitters.len() as u16).to_be_bytes());
    data.extend_from_slice(&(textures.len() as u16).to_be_bytes());
    data.resize(ARCHIVE_HEADER_SIZE, 0);
    for emitter in emitters {
        let start = data.len();
        data.extend_from_slice(b"JEFF");
        data.extend_from_slice(b"jpa1");
        data.extend_from_slice(&0_u32.to_be_bytes());
        data.extend_from_slice(&(emitter.blocks.len() as u32).to_be_bytes());
        data.extend_from_slice(&0_u32.to_be_bytes());
        data.push(emitter.key_num);
        data.push(emitter.fld_num);
        data.push(emitter.texture_num);
        data.push(0);
        data.extend_from_slice(&emitter.user_index.to_be_bytes());
        data.resize(start + PARTICLE_HEADER_SIZE, 0);
        for &(magic, size) in emitter.blocks {
            let block = data.len();
            data.extend_from_slice(&magic.to_be_bytes());
            data.extend_from_slice(&size.to_be_bytes());
            data.extend_from_slice(&0_u32.to_be_bytes());
            data.resize(block + size as usize, 0);
        }
    }
    for &size in textures {
        let start = data.len();
        data.extend_from_slice(&TEXTURE_MAGIC.to_be_bytes());
        data.extend_from_slice(&size.to_be_bytes());
        data.resize(start + size as usize, 0);
    }
    data
}

fn emitter(user_index: u16, blocks: &'static [(u32, u32)]) -> Emitter {
    Emitter {
        user_index,
        key_num: 0,
        fld_num: 0,
        texture_num: 0,
        blocks,
    }
}

#[test]
fn golden_archive_header_and_table_parse_exact_ids() {
    let data = archive(
        &[
            Emitter {
                user_index: 0xA06A,
                key_num: 1,
                fld_num: 0,
                texture_num: 2,
                blocks: &[(BLOCK_KEY, 0x14), (BLOCK_TEX_DATA, 0x10)],
            },
            Emitter {
                user_index: 0x0001,
                key_num: 0,
                fld_num: 1,
                texture_num: 1,
                blocks: &[(BLOCK_FIELD, 0x18)],
            },
        ],
        &[0x30],
    );

    let mut bank = JpaBank::default();
    bank.register(1, &data).expect("archive is well formed");
    let manager = bank.manager(1).expect("manager 1 is mounted");

    assert_eq!(manager.resources().len(), 2);
    assert_eq!(manager.resources()[0].user_index, 0xA06A);
    assert_eq!(manager.resources()[0].key_num, 1);
    assert_eq!(manager.resources()[0].fld_num, 0);
    assert_eq!(manager.resources()[0].texture_num, 2);
    assert_eq!(manager.resources()[1].user_index, 0x0001);
    assert_eq!(manager.resources()[1].fld_num, 1);

    let first = ARCHIVE_HEADER_SIZE + PARTICLE_HEADER_SIZE;
    assert_eq!(manager.resources()[0].blocks.len(), 2);
    assert_eq!(manager.resources()[0].blocks[0].magic, BLOCK_KEY);
    assert_eq!(manager.resources()[0].blocks[0].size, 0x14);
    assert_eq!(manager.resources()[0].blocks[0].offset, first);
    assert_eq!(manager.resources()[0].blocks[1].magic, BLOCK_TEX_DATA);
    assert_eq!(manager.resources()[0].blocks[1].offset, first + 0x14);

    let second = first + 0x14 + 0x10;
    assert_eq!(
        manager.resources()[1].blocks[0].offset,
        second + PARTICLE_HEADER_SIZE
    );
    assert_eq!(manager.resources()[1].blocks[0].size, 0x18);

    let texture = second + PARTICLE_HEADER_SIZE + 0x18;
    assert_eq!(manager.textures().len(), 1);
    assert_eq!(manager.textures()[0].magic, TEXTURE_MAGIC);
    assert_eq!(manager.textures()[0].size, 0x30);
    assert_eq!(manager.textures()[0].offset, texture);
    assert_eq!(data.len(), texture + 0x30);
}

#[test]
fn lookup_hits_in_registration_order_and_misses_by_manager() {
    let data = archive(
        &[
            emitter(0x8001, &[]),
            emitter(0x8001, &[]),
            emitter(0x8002, &[]),
        ],
        &[],
    );
    let mut bank = JpaBank::default();
    bank.register(1, &data).expect("archive is well formed");

    let hit = bank
        .get_by_user_index(1, 0x8001)
        .expect("first id is carried");
    assert_eq!(hit.user_index, 0x8001);
    assert_eq!(bank.registration_index(1, 0x8001), Some(0));
    assert_eq!(
        bank.get_by_user_index(1, 0x8002).map(|r| r.user_index),
        Some(0x8002)
    );
    assert_eq!(bank.get_by_user_index(1, 0x8003), None);
    assert!(bank.check_user_index_duplication(1, 0x8002));
    assert!(!bank.check_user_index_duplication(1, 0x8003));

    assert_eq!(bank.get_by_user_index(0, 0x8001), None);
    assert!(bank.manager(0).is_none());
}

#[test]
fn emitter_from_bank_resource_carries_retail_ids() {
    let data = archive(&[emitter(0xA06A, &[(BLOCK_KEY, 0x14)])], &[]);
    let mut jpa = Jpa::default();
    jpa.load_manager(1, &data).expect("archive is well formed");
    let pos = Vec3::new(1.5, -2.0, 3.25);

    let slot = jpa
        .create_simple_emitter_id(pos, 0xA06A, 2, 1)
        .expect("the id is in the mounted manager");
    let created = &jpa.emitters[slot];
    assert!(created.active);
    assert_eq!(created.res_id, 0xA06A);
    assert_eq!(created.res_mgr_id, 1);
    assert_eq!(created.group_id, 2);
    assert_eq!(created.global_translation, pos);
    assert_eq!(
        created.resource,
        Some(JpaResourceRef {
            rm_id: 1,
            index: 0,
            user_index: 0xA06A,
        })
    );
    assert!(jpa.take_gaps().is_empty());

    let packed = (2_u32 << 24) | (1_u32 << 16) | 0xA06A;
    let slot = jpa
        .create_simple_emitter_packed_id(pos, packed)
        .expect("the packed id decodes to the same resource");
    let created = &jpa.emitters[slot];
    assert_eq!(created.res_id, 0xA06A);
    assert_eq!(created.res_mgr_id, 1);
    assert_eq!(created.group_id, 2);
}

#[test]
fn unknown_resource_records_a_typed_gap_and_creates_nothing() {
    let mut jpa = Jpa::default();
    let pos = Vec3::new(0.0, 0.0, 0.0);

    assert_eq!(jpa.create_simple_emitter_id(pos, 0xA06A, 2, 1), None);
    assert_eq!(
        jpa.take_gaps(),
        vec![JpaGap::ManagerNotMounted {
            group_id: 2,
            rm_id: 1,
            user_id: 0xA06A,
        }]
    );

    let data = archive(&[emitter(0x8001, &[])], &[]);
    jpa.load_manager(1, &data).expect("archive is well formed");
    assert_eq!(jpa.create_simple_emitter_id(pos, 0xA06A, 2, 1), None);
    assert_eq!(
        jpa.take_gaps(),
        vec![JpaGap::UnknownResource {
            group_id: 2,
            rm_id: 1,
            user_id: 0xA06A,
        }]
    );
    assert!(jpa.emitters.iter().all(|emitter| !emitter.active));
}

#[test]
fn clear_resource_manager_force_deletes_that_slot_s_emitters() {
    let data = archive(&[emitter(0x8001, &[])], &[]);
    let mut jpa = Jpa::default();
    jpa.load_manager(1, &data).expect("archive is well formed");
    jpa.load_manager(2, &data).expect("archive is well formed");
    let pos = Vec3::ZERO;
    let swept = jpa
        .create_simple_emitter_id(pos, 0x8001, 0, 1)
        .expect("manager 1 carries the id");
    let kept = jpa
        .create_simple_emitter_id(pos, 0x8001, 0, 2)
        .expect("manager 2 carries the id");

    jpa.clear_resource_manager(1);

    assert!(
        !jpa.emitters[swept].active,
        "the slot's emitter goes back to the pool"
    );
    assert_eq!(jpa.emitters[swept], clearsleeper::sim::jpa::Emitter::default());
    assert!(jpa.emitters[kept].active, "the other slot's emitter stays");
    assert!(jpa.bank.manager(1).is_none());
    assert!(jpa.bank.manager(2).is_some());

    let reused = jpa
        .create_simple_emitter_id(pos, 0x8001, 0, 2)
        .expect("manager 2 is still mounted");
    assert_eq!(reused, swept, "the freed slot is the pool's first");
}

#[test]
fn unreadable_archives_are_typed_errors() {
    let mut bank = JpaBank::default();
    assert_eq!(
        bank.register(0, b"NOPE"),
        Err(JpaBankError::BadArchiveMagic {
            magic: u32::from_be_bytes(*b"NOPE"),
        })
    );
    assert_eq!(
        bank.register(0, b"JPAC2-00"),
        Err(JpaBankError::BadArchiveVersion {
            version: u32::from_be_bytes(*b"2-00"),
        })
    );
    assert_eq!(
        bank.register(0, b"JPAC"),
        Err(JpaBankError::Truncated {
            at: 4,
            need: 4,
            len: 4,
        })
    );
    assert_eq!(
        bank.register(8, &archive(&[], &[])),
        Err(JpaBankError::ManagerOutOfRange { rm_id: 8 })
    );
    assert!(bank.manager(0).is_none());

    let mut damaged = archive(&[emitter(0x8001, &[(BLOCK_KEY, 0x14)])], &[]);
    let size_at = ARCHIVE_HEADER_SIZE + PARTICLE_HEADER_SIZE + 4;
    damaged[size_at..size_at + 4].copy_from_slice(&0_u32.to_be_bytes());
    assert_eq!(
        bank.register(0, &damaged),
        Err(JpaBankError::ZeroSize {
            at: ARCHIVE_HEADER_SIZE + PARTICLE_HEADER_SIZE,
        })
    );
    damaged[size_at..size_at + 4].copy_from_slice(&0x1000_u32.to_be_bytes());
    assert_eq!(
        bank.register(0, &damaged),
        Err(JpaBankError::OutOfRange {
            at: ARCHIVE_HEADER_SIZE + PARTICLE_HEADER_SIZE,
            size: 0x1000,
            len: damaged.len(),
        })
    );
    assert!(bank.manager(0).is_none());
}
