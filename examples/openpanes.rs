use clearsleeper::assets;
use clearsleeper::j2d;

fn main() {
    let Ok(root) = std::env::var("CS_ASSETS") else {
        eprintln!("CS_ASSETS unset");
        return;
    };
    let root = assets::AssetRoot::at(root);
    for arc_name in ["files/res/Object/Opening.arc", "files/res/Object/Open1.arc"] {
        let Ok(arc) = root.read_archive(arc_name) else {
            println!("{arc_name}: missing");
            continue;
        };
        for entry in arc.entries() {
            if !entry.ends_with(".blo") {
                continue;
            }
            let bytes = arc.read(entry).unwrap_or_default();
            let Some(screen) = j2d::parse(bytes) else {
                println!("{arc_name}/{entry}: parse failed ({} bytes)", bytes.len());
                continue;
            };
            println!(
                "{arc_name}/{entry}: {}x{} color={:08x} panes={}",
                screen.width,
                screen.height,
                screen.color,
                screen.panes.len()
            );
            for pane in &screen.panes {
                println!(
                    "  {:<6} {:?} ({:>7.1},{:>6.1}) {:>6.1}x{:<6.1} a={:3} vis={} tex={:?} font={:?} fs={}x{} cs={} ls={} txt={:?}",
                    pane.name,
                    pane.kind,
                    pane.bounds.x,
                    pane.bounds.y,
                    pane.bounds.w,
                    pane.bounds.h,
                    pane.alpha,
                    u8::from(pane.visible),
                    pane.texture,
                    pane.font,
                    pane.font_size_x,
                    pane.font_size_y,
                    pane.char_space,
                    pane.line_space,
                    pane.text.chars().take(24).collect::<String>(),
                );
            }
        }
    }
}