use clearsleeper::assets;

fn main() {
    let Ok(root) = std::env::var("CS_ASSETS") else {
        eprintln!("CS_ASSETS unset");
        return;
    };
    let _ = assets::AssetRoot::at(root);
    let tables = assets::load_stage_tables(assets::SEA_STAGE).expect("stage tables");
    println!(
        "stage {:?} actors={} events={}",
        tables.name(),
        tables.actors().len(),
        tables.event_names().len()
    );

    let set = assets::start_set(&tables);
    println!("start room set: {set:?}");
    match assets::load_room_set(&tables, &set, |_| 0) {
        Some(room) => {
            println!(
                "room: collision={} mesh_groups={} textures={} actors={}",
                room.collision.is_some(),
                room.mesh.as_ref().map_or(0, |m| m.groups.len()),
                room.mesh.as_ref().map_or(0, |m| m.textures.len()),
                room.actors.len()
            );
            let mut names: Vec<&str> = room.actors.iter().map(|a| a.name.as_str()).collect();
            names.sort_unstable();
            println!("actor names ({}):", names.len());
            for chunk in names.chunks(8) {
                println!("  {}", chunk.join(" "));
            }
            if let Some(c) = room.collision.as_ref() {
                let mut min = [f32::MAX; 3];
                let mut max = [f32::MIN; 3];
                for a in &room.actors {
                    for k in 0..3 {
                        min[k] = min[k].min(a.pos[k]);
                        max[k] = max[k].max(a.pos[k]);
                    }
                }
                println!("actor bounds x {:.1}..{:.1}  y {:.1}..{:.1}  z {:.1}..{:.1}",
                    min[0], max[0], min[1], max[1], min[2], max[2]);
                let _ = c;
            }
            for a in room.actors.iter().filter(|a| a.name == "P1a") {
                println!("player start P1a: pos={:?} angle={:?} params={:#x}", a.pos, a.angle, a.parameters);
            }
            for a in room.actors.iter().filter(|a| a.name == "Dk") {
                println!("dragon Dk: pos={:?} params={:#x}", a.pos, a.parameters);
            }
            for a in room.actors.iter().filter(|a| a.name == "NpcSo" || a.name == "AND_SW2") {
                println!("{}: pos={:?} params={:#x}", a.name, a.pos, a.parameters);
            }
            for id in 0..8u8 {
                if let Some(e) = tables.exit(id) {
                    println!("exit {id}: {:?} start={} room={}", e.stage, e.start, e.room);
                }
            }
        }
        None => println!("room set FAILED to load"),
    }
}