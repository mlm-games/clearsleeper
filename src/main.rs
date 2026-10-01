//! Entry: desktop window plus Android NativeActivity (`cargo rapk`).

#[cfg(not(target_os = "android"))]
fn main() -> anyhow::Result<()> {
    let size = std::env::var("CS_WINDOW")
        .ok()
        .and_then(|v| {
            let (a, b) = v.split_once('x')?;
            Some((a.parse().ok()?, b.parse().ok()?))
        })
        .unwrap_or((1280, 720));
    repame_shell::run_desktop("clearsleeper", size, clearsleeper::root_view)
}

#[cfg(target_os = "android")]
fn main() {}
