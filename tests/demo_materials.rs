//! Real-disc checks for the demo cut model's material half: the btp and brk keys
//! `daDemo00_c::createHeap` inits (`d_a_demo00.cpp:173-221`) landing in the node table's material
//! slots. Skipped when `CS_ASSETS` is unset, like the other real-disc checks.

use clearsleeper::assets::ASSETS_ENV;
use clearsleeper::sim::demo::{PassInit, RoomDemoArchive};

#[test]
fn real_disc_demo_tex_pattern_key_rewrites_the_material_texture() {
    let Ok(_) = std::env::var(ASSETS_ENV) else {
        eprintln!("CS_ASSETS is not set; skipping real-disc demo tex-pattern check");
        return;
    };
    let mut archive = RoomDemoArchive::default();
    archive.name_from_room(Some("Demo07"));
    let mut table = archive.model(0x29).expect("km.bdl is in Demo07");
    let eye = table
        .materials()
        .iter()
        .find(|material| material.name.as_deref() == Some("SC_eye"))
        .expect("km.bdl carries the SC_eye material");
    assert_eq!(
        eye.tex_nos[0],
        Some(8),
        "the MAT3 texture table's own binding for the material's first unit"
    );

    let pass = archive
        .tex_pattern(0x0E)
        .expect("07_km_cut3waitb.btp is a J3D1btp1 this port reads")
        .expect("the key is in Demo07");
    table.set_tex_pattern(Some(pass));
    table.set_pass_frames(0.0);

    let eye = table
        .materials()
        .iter()
        .find(|material| material.name.as_deref() == Some("SC_eye"))
        .expect("km.bdl carries the SC_eye material");
    assert_eq!(
        eye.tex_nos[0],
        Some(4),
        "the key's frame-zero texture, which is not the material's own binding"
    );
}

#[test]
fn real_disc_demo_tev_reg_key_writes_the_cut_material_register() {
    let Ok(_) = std::env::var(ASSETS_ENV) else {
        eprintln!("CS_ASSETS is not set; skipping real-disc demo tev-register check");
        return;
    };
    let mut archive = RoomDemoArchive::default();
    archive.name_from_room(Some("Demo01"));
    let mut table = archive.model(0x18).expect("blackfadebox.bdl is in Demo01");
    assert_eq!(table.materials().len(), 1);
    assert_eq!(table.materials()[0].name.as_deref(), Some("blackFade"));

    let pass = archive
        .tev_reg(0x12)
        .expect("fadebox_invisible.brk is a J3D1brk1 this port reads")
        .expect("the key is in Demo01");
    assert_eq!(pass.c_reg.len(), 1, "one C-register entry");
    assert_eq!(pass.c_reg[0].color_id, 1, "it drives register C1");
    table.set_tev_reg(Some(pass), 0x12, PassInit::Heap);
    table.set_pass_frames(0.0);

    assert_eq!(
        table.materials()[0].tev_colors[1],
        Some([0, 0, 0, 0]),
        "the entry's own value, which is what hides the fade box"
    );
}
