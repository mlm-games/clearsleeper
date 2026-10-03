use std::time::Duration;

use clearsleeper::sim::area::Area;
use clearsleeper::sim::player::Player;
use clearsleeper::App;

#[test]
fn golden_boot_spawns_one_player_in_area() {
    let mut app = App::new();
    let area = *app.sim.world.resource::<Area>();
    let mut query = app.sim.world.query::<&Player>();
    let players: Vec<Player> = query.iter(&app.sim.world).copied().collect();
    assert_eq!(players.len(), 1);
    assert_eq!(players[0].pos, area.spawn);
    assert!(players[0].pos.x >= area.bounds_min.x && players[0].pos.x <= area.bounds_max.x);
    assert!(players[0].pos.z >= area.bounds_min.y && players[0].pos.z <= area.bounds_max.y);

    app.advance(Duration::from_millis(100));
    let players: Vec<Player> = query.iter(&app.sim.world).copied().collect();
    assert_eq!(players.len(), 1);
}

#[test]
fn golden_boot_spawns_start_room_actors() {
    let Ok(_) = std::env::var(clearsleeper::assets::ASSETS_ENV) else {
        eprintln!("CS_ASSETS is not set; skipping start room actor check");
        return;
    };
    let mut app = App::new();
    let mut query = app.sim.world.query::<&clearsleeper::sim::actor::Actor>();
    let names: Vec<String> = query
        .iter(&app.sim.world)
        .map(|actor| actor.name.clone())
        .collect();
    assert!(
        names.iter().any(|name| name == "NpcSo"),
        "the start island room spawns its soldier"
    );

    app.advance(Duration::from_millis(100));
    let names: Vec<String> = query
        .iter(&app.sim.world)
        .map(|actor| actor.name.clone())
        .collect();
    assert!(
        names.iter().any(|name| name == "NpcSo"),
        "room actors survive the first ticks"
    );
}
