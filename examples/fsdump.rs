use clearsleeper::archive::Archive;
use clearsleeper::assets::AssetRoot;
use clearsleeper::j2d;
use std::path::PathBuf;

fn main() {
    let Ok(root) = std::env::var("CS_ASSETS") else {
        eprintln!("CS_ASSETS unset");
        return;
    };
    let root = AssetRoot::at(root);
    let Ok(stage) = root.read_archive("files/res/Stage/Name/Stage.arc") else {
        eprintln!("no Stage.arc");
        return;
    };
    let Ok(bytes) = stage.read("dat/file_select.arc") else {
        eprintln!("no file_select.arc");
        return;
    };
    let inner = Archive::parse(bytes.to_vec(), PathBuf::from("dat/file_select.arc")).expect("arc");
    let screen = j2d::parse(inner.read("file_select.blo").expect("blo")).expect("screen");
    for pane in &screen.panes {
        let Some(font) = pane.font.as_deref() else {
            continue;
        };
        println!(
            "{:>6} font={font:<24} size=({},{}) bind=({},{}) space=({},{}) text={:?}",
            pane.name,
            pane.font_size_x,
            pane.font_size_y,
            pane.binding_h,
            pane.binding_v,
            pane.char_space,
            pane.line_space,
            pane.text
        );
    }
}
