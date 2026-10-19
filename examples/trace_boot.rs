//! Headless boot: a fresh session, a tape, N ticks and a trace file.
//!
//! The retail comparison's port side: the session the game starts in (no save loaded, the
//! world the boot builds), the pad tape replayed into it, and the per-tick digest the
//! differ reads. No window, no GPU device and no audio device are opened: [`App::new`] is
//! the headless construction, its audio a no-op sink.
//!
//! ```text
//! usage: trace_boot [ticks] [trace-file]
//! ```
//!
//! - `ticks` is the number of sim ticks to run, `CS_TRACE_TICKS` when it is not given and
//!   600 when neither is.
//! - `trace-file` is where the trace goes, `CS_TRACE` when it is not given. The path is the
//!   one the trace hook reads (`CS_TRACE`), so a path given here is installed as that
//!   variable before the session is built.
//!
//! The environment the session reads: `CS_DISC` names the disc image to install, `CS_ASSETS`
//! the extracted tree to fall back on, and `CS_INPUT` the tape to replay.

use std::path::{Path, PathBuf};
use std::time::Duration;

use clearsleeper::input_tape::InputTape;
use clearsleeper::sim::frame::CurrentFrame;
use clearsleeper::{App, SIM_HZ};

const USAGE: &str = "usage: trace_boot [ticks] [trace-file]";
const DEFAULT_TICKS: u64 = 600;

fn main() {
    let mut args = std::env::args().skip(1);
    let ticks = match args.next() {
        Some(text) => match text.parse::<u64>() {
            Ok(ticks) => ticks,
            Err(_) => {
                eprintln!("trace_boot: {text} is not a tick count");
                eprintln!("{USAGE}");
                std::process::exit(2);
            }
        },
        None => env_ticks(),
    };
    let Some(trace) = trace_path(args.next()) else {
        eprintln!("trace_boot: no trace file: set CS_TRACE or name one");
        eprintln!("{USAGE}");
        std::process::exit(2);
    };
    unsafe { std::env::set_var("CS_TRACE", &trace) };

    if let Some(disc) = std::env::var_os("CS_DISC").filter(|value| !value.is_empty()) {
        match clearsleeper::source::install_file(Path::new(&disc)) {
            Ok(summary) => println!("disc: {summary}"),
            Err(error) => {
                eprintln!("trace_boot: {error}");
                std::process::exit(1);
            }
        }
    }

    let tape = match read_tape() {
        Ok(tape) => tape,
        Err(error) => {
            eprintln!("trace_boot: {error}");
            std::process::exit(1);
        }
    };

    let mut app = App::new();
    if app.stage().is_none() {
        eprintln!("trace_boot: no stage data: set CS_DISC or CS_ASSETS");
    }

    let step = Duration::from_secs_f64(1.0 / SIM_HZ);
    for _ in 0..ticks {
        let tick = app.sim.world.resource::<CurrentFrame>().0 + 1;
        let state = tape.state_at(tick);
        app.stage_pad_state(&state);
        app.advance(step);
    }

    let ran = app.sim.world.resource::<CurrentFrame>().0;
    println!("ticks: {ran} (asked {ticks})");
    println!("trace: {}", trace.display());
}

fn env_ticks() -> u64 {
    std::env::var("CS_TRACE_TICKS")
        .ok()
        .and_then(|text| text.parse::<u64>().ok())
        .unwrap_or(DEFAULT_TICKS)
}

fn trace_path(argument: Option<String>) -> Option<PathBuf> {
    let path = match argument {
        Some(path) => PathBuf::from(path),
        None => PathBuf::from(std::env::var_os("CS_TRACE")?),
    };
    if path.as_os_str().is_empty() {
        return None;
    }
    Some(path)
}

fn read_tape() -> Result<InputTape, String> {
    let Some(path) = std::env::var_os("CS_INPUT").filter(|value| !value.is_empty()) else {
        return Ok(InputTape::default());
    };
    let text =
        std::fs::read_to_string(&path).map_err(|error| format!("{}: {error}", path.display()))?;
    InputTape::parse(&text).map_err(|error| format!("{}: {error}", path.display()))
}
