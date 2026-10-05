use bevy_ecs::prelude::*;
use bevy_ecs::schedule::Schedule;

use clearsleeper::assets;
use clearsleeper::text::{driver, font::FontBank, msgbox::Msg2Box, msgbox1::Msg1Box, msgproc};

fn main() {
    let Ok(root) = std::env::var("CS_ASSETS") else {
        eprintln!("CS_ASSETS unset");
        return;
    };
    let root = assets::AssetRoot::at(root);
    let archives = assets::MsgArchives::load(&root);

    let mut world = World::new();
    world.insert_resource(archives);
    world.insert_resource(msgproc::MsgProcs::default());
    world.insert_resource(msgproc::MsgGameInfo::default());
    world.insert_resource(driver::TextDriver::default());
    world.insert_resource(driver::TextEnv::default());
    world.insert_resource(driver::Stick::default());
    world.insert_resource(driver::Names::default());
    world.insert_resource(driver::TextFlags::default());
    world.insert_resource(driver::Counters::default());
    world.insert_resource(driver::Layout::default());
    world.insert_resource(driver::TextScreen::default());
    world.insert_resource(Msg1Box::default());
    world.insert_resource(Msg2Box::default());
    let mut fonts = FontBank::new();
    driver::load_fonts(&mut fonts);
    world.insert_resource(fonts);

    let archive = world.remove_resource::<assets::MsgArchives>().unwrap();
    {
        let mut procs = world.remove_resource::<msgproc::MsgProcs>().unwrap();
        let mut info = world.remove_resource::<msgproc::MsgGameInfo>().unwrap();
        driver::spawn(&mut procs, &mut info, &archive, 0, 0x58E);
        println!("procs after spawn: {}", procs.procs().len());
        world.insert_resource(procs);
        world.insert_resource(info);
    }
    world.insert_resource(archive);

    let mut schedule = Schedule::default();
    schedule.add_systems(driver::control);

    for step in 0..300 {
        schedule.run(&mut world);
        if step % 30 == 29 {
            let one = world.resource::<Msg1Box>();
            println!("step {step}: text_len={} {:?}", one.output_text.len(), one.output_text);
        }
    }

    let one = world.resource::<Msg1Box>();
    println!("msg1 text {:?} select {:?}", one.output_text, one.select_text);
    let screen = world.resource::<driver::TextScreen>();
    for (i, pane) in screen.msg1.text.iter().enumerate() {
        match pane {
            Some(p) => println!(
                "pane {i}: bounds={:?} alpha={} visible={}",
                p.bounds, p.alpha, p.visible
            ),
            None => println!("pane {i}: none"),
        }
    }
    let f = world.resource::<FontBank>();
    let font = f.get(0);
    println!(
        "font valid={} page={}B",
        font.is_some_and(|x| x.valid),
        font.map_or(0, |x| x.tex_page().len())
    );

    let overlay = clearsleeper::text::overlay::render(&mut world, 640, 480);
    println!(
        "overlay live={} lit={}",
        overlay.live,
        overlay.canvas.pixels.chunks_exact(4).filter(|p| p[3] > 0).count()
    );
    std::fs::write("/tmp/opencode/overlay.rgba", &overlay.canvas.pixels).unwrap();
}