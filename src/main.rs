//! Entry: desktop window plus Android NativeActivity (`cargo rapk`).

#[cfg(not(target_os = "android"))]
fn main() -> anyhow::Result<()> {
    use std::time::{Duration, Instant};

    let size = std::env::var("CS_WINDOW")
        .ok()
        .and_then(|v| {
            let (a, b) = v.split_once('x')?;
            Some((a.parse().ok()?, b.parse().ok()?))
        })
        .unwrap_or((1280, 720));
    let mut app = clearsleeper::App::new();
    let mut last = Instant::now();
    repame_shell::run_desktop("clearsleeper", size, move |sched, ctx| {
        let now = Instant::now();
        let dt = now.duration_since(last).min(Duration::from_secs_f32(0.25));
        last = now;
        app.advance(dt);
        clearsleeper::root_view(sched, ctx)
    })
}

#[cfg(target_os = "android")]
fn main() {}
