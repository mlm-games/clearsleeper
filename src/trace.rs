//! Per-tick simulation digests for cross-build equivalence.
//!
//! Set `CS_TRACE` to a file path on a native build and every simulated tick appends exactly
//! one canonical line describing that tick's fully-advanced state. `lib.rs`'s `reset_world`
//! calls [`install`] once the world is built and the room set is mounted, so each world
//! construction registers the observer afresh; a second world (a new-game reset) reopens the
//! same path in append mode while the sim's own counter restarts from one. [`install`]
//! returns `false` unless `CS_TRACE` names a non-empty path, and the untraced build then runs
//! with no observer system, no sink resource and no file, so the disabled path costs zero
//! work per tick. The observer is a system over a read-only `&World` chained after the tick
//! pipeline's last system (`sim::ba1::control_draw`, which the whole `sim::build` chain
//! reaches last), so the line reports a state every tick system has already passed and no
//! existing system's order or behaviour is touched.
//!
//! On `wasm32` the emission is a documented no-op: [`install`] returns `false` without
//! touching the variable, because the trace tool diffs the byte stream on disk, which no web
//! host grants the page. The gate lives here rather than at the call site, so whatever the
//! variable holds the web build neither opens a file nor panics; it is a target fact, not a
//! runtime value the host can see.
//!
//! # Line grammar
//!
//! One line per tick, `\n` terminated. The pairs are space separated, every key appears
//! exactly once, and the order is fixed, so a reader can compare positionally or by key.
//! Written in order, with `-` standing in for an absent string value:
//!
//! ```text
//! tick=<u64> stage=<name|-> room=<i8>
//!     px=<f> py=<f> pz=<f>
//!     yaw=<u32> face=<u32> speed=<f> speed_y=<f> proc=<name>
//!     hp=<u8> hpmax=<u8> rupees=<u32>
//!     sw=0x<hex> ef=0x<hex> sb=0x<hex>
//!     evt=<n|-> evtst=<name|-> talk=<0|1> rng=0x<hex8>
//!     a<proc>:kind=<n>,x=<f>,y=<f>,z=<f>,yaw=<i16>,hp=<i32>   (once per actor, ascending)
//! ```
//!
//! - `tick` is `sim::frame::CurrentFrame`, the sim's own counter, read after `frame::advance`
//!   ran: the 1-based index of the tick the line reports. The observer's own invocation count
//!   would agree tick for tick and stays unused because the canonical counter exists. The
//!   player token holds `sim::player::Player`; without a player entity it falls back to
//!   `sim::area::Area`'s spawn and yaw with zero speed, the same fallback `App::snapshot`
//!   uses.
//! - `px`, `py`, `pz`, `speed` and `speed_y` are exact `f32` bits, lowercase hex of
//!   `f32::to_bits` with all eight nibbles (`0x3f800000`). `yaw` and `face` print the same
//!   `f32::to_bits` as plain unsigned decimal. No float is ever written as text, so `nan` and
//!   `inf` cannot appear.
//! - `stage` is `sim::area::StageName` verbatim, asset stage codes carrying no whitespace;
//!   the empty string plays `-`. `room` is `sim::door::StayRoom`, the status-room id the
//!   transition tables keep, `-1` outside a room.
//! - `proc` is `{:?}` of the player's `sim::link_proc::LinkProc::current`, the stable
//!   `sim::link_proc::Proc` variant name (`Wait`, `CutA`, ...).
//! - `hp` and `hpmax` are `sim::health::Health::units/max`, `rupees` is
//!   `sim::pickup::Rupees::count`.
//! - `sw`, `ef` and `sb` are registers rendered as one hex number each: `sim::flags::Switches`
//!   as its four words, each in little-endian byte order, words kept in word order,
//!   `sim::flags::EventFlags` and `sim::flags::StoryBits` as their stored byte rows. Leading
//!   zero bytes are trimmed; the first remaining byte keeps both its nibbles and every byte
//!   after it prints, so no non-leading zero is ever trimmed and an all-zero register prints
//!   `0x0`.
//! - `evt` and `evtst` are the head of the event manager's runs in execution order: the run's
//!   0-based `sim::event::EventList` position and the `sim::event::Stage` variant name
//!   (`Order`, `Play`, `Close`). No run prints `-` for both.
//! - `talk` is `sim::talk::Dialogue::open` as `0` or `1`. `rng` is `0x00000000`: the port owns
//!   no central RNG stream, every draw site carries its own state, so the key stays total with
//!   a fixed word until the sim gets a central register and a native reference tool can mirror
//!   the same slot.
//! - One actor token per live `sim::actor::Actor` registered in the
//!   `sim::registry::ActorRegistry`, ordered ascending by `proc_id`, the registry's stable
//!   process id (the reference's own `fopAc` id, not the bevy `Entity`). `kind` is FNV-1a-32
//!   of the actor's name bytes; `x`, `y`, `z` are `actor.pos`'s f32 bits, `yaw` is
//!   `actor.angle[1]` as decimal and `hp` is `sim::combat::HitPoints` (`0` when the actor has
//!   none). A registry entry whose actor no longer holds an `Actor` prints no token.
//!
//! Writes open the file in append mode at the first line and flush once per line. A failed
//! open or write warns once through `log::warn!` and then goes silent for the rest of the
//! session, so tracing can never turn a failing disk into a failed game.
//!
//! # Reading a trace
//!
//! [`parse_text`] turns a whole file into lines in file order; [`parse_line`] reads one
//! hand-written line. Tokens split on single spaces and each token splits on its first `=`,
//! so a token without `=` is [`TraceError::BadPair`] and a file with no lines is
//! [`TraceError::Empty`]: typed errors, never panics. [`diff_report`] compares two parsed
//! traces index by index and field by field and reports the first divergence as
//!
//! ```text
//! first difference at tick 2
//!   sw: 0x0 vs 0x1
//! 1 further differing ticks
//! ```
//!
//! where the tick label is the left line's `tick=` value falling back to the right side's and
//! then to the 1-based line index, and a field missing on either side prints `absent`.

use std::fmt::Write as _;

use bevy_ecs::prelude::World;

use crate::sim::{
    actor, area, combat, door, event, flags, frame, health, link_proc, pickup, player, registry,
    talk,
};

#[cfg(target_arch = "wasm32")]
pub fn install(_sim: &mut repame_sim::Sim) -> bool {
    false
}

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use std::fs;
    use std::io::Write as _;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Mutex, MutexGuard};

    use bevy_ecs::prelude::*;
    use bevy_ecs::schedule::IntoScheduleConfigs;
    use repame_sim::Sim;

    use super::line_of;
    use crate::sim::ba1;

    #[derive(Resource)]
    struct Sink {
        path: PathBuf,
        file: Mutex<Option<fs::File>>,
        failed: AtomicBool,
    }

    impl Sink {
        fn append(&self, line: &str) {
            if self.failed.load(Ordering::Relaxed) {
                return;
            }
            let mut slot: MutexGuard<Option<fs::File>> = match self.file.lock() {
                Ok(slot) => slot,
                Err(poisoned) => poisoned.into_inner(),
            };
            if slot.is_none() {
                match fs::OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(&self.path)
                {
                    Ok(file) => *slot = Some(file),
                    Err(error) => {
                        self.failed.store(true, Ordering::Relaxed);
                        log::warn!("clearsleeper: trace open failed ({error})");
                        return;
                    }
                }
            }
            let Some(file) = slot.as_mut() else {
                return;
            };
            if let Err(error) = file.write_all(line.as_bytes()).and_then(|()| file.flush()) {
                self.failed.store(true, Ordering::Relaxed);
                log::warn!("clearsleeper: trace write failed ({error})");
            }
        }
    }

    fn observe(world: &World) {
        let Some(sink) = world.get_resource::<Sink>() else {
            return;
        };
        if sink.failed.load(Ordering::Relaxed) {
            return;
        }
        let mut line = line_of(world);
        line.push('\n');
        sink.append(&line);
    }

    pub fn install(sim: &mut Sim) -> bool {
        let Some(path) = std::env::var_os("CS_TRACE")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
        else {
            return false;
        };
        sim.world.insert_resource(Sink {
            path,
            file: Mutex::new(None),
            failed: AtomicBool::new(false),
        });
        sim.add_chained_systems(observe.after(ba1::control_draw));
        true
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use native::install;

pub fn parse_line(line: &str) -> Result<TickLine, TraceError> {
    parse_pair(line.trim_end_matches(['\n', '\r']), 1)
}

pub fn parse_text(text: &str) -> Result<Vec<TickLine>, TraceError> {
    if text.is_empty() {
        return Err(TraceError::Empty);
    }
    let mut rows = Vec::new();
    for (at, line) in text.lines().enumerate() {
        rows.push(parse_pair(line, at + 1)?);
    }
    Ok(rows)
}

fn parse_pair(line: &str, number: usize) -> Result<TickLine, TraceError> {
    if line.is_empty() {
        return Err(TraceError::BadPair {
            line: number,
            text: line.to_string(),
        });
    }
    let mut fields = Vec::new();
    for token in line.split(' ') {
        let Some((key, value)) = token.split_once('=') else {
            return Err(TraceError::BadPair {
                line: number,
                text: line.to_string(),
            });
        };
        fields.push(Field {
            key: key.to_string(),
            value: value.to_string(),
        });
    }
    Ok(TickLine { fields })
}

pub struct TickLine {
    fields: Vec<Field>,
}

impl TickLine {
    pub fn field(&self, key: &str) -> Option<&str> {
        self.fields
            .iter()
            .find(|field| field.key == key)
            .map(|field| field.value.as_str())
    }

    pub fn tick(&self) -> Option<&str> {
        self.field("tick")
    }

    pub fn to_line(&self) -> String {
        let mut text = String::new();
        for (at, field) in self.fields.iter().enumerate() {
            if at > 0 {
                text.push(' ');
            }
            text.push_str(&field.key);
            text.push('=');
            text.push_str(&field.value);
        }
        text
    }
}

struct Field {
    key: String,
    value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraceError {
    Empty,
    BadPair { line: usize, text: String },
}

impl std::fmt::Display for TraceError {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TraceError::Empty => write!(out, "empty trace"),
            TraceError::BadPair { line, text } => {
                write!(out, "line {line} has no key=value pair: {text:?}")
            }
        }
    }
}

pub struct Diff {
    pub tick: String,
    pub fields: Vec<(String, String, String)>,
    pub further: usize,
}

const ABSENT: &str = "absent";

impl Diff {
    pub fn message(&self) -> String {
        let mut text = format!("first difference at tick {}\n", self.tick);
        for (key, left, right) in &self.fields {
            let _ = write!(text, "  {key}: {left} vs {right}\n");
        }
        let _ = write!(text, "{} further differing ticks", self.further);
        text
    }
}

pub fn diff_report(left: &[TickLine], right: &[TickLine]) -> Option<Diff> {
    let mut first: Option<Diff> = None;
    let mut further = 0usize;
    for at in 0..left.len().max(right.len()) {
        let fields = match (left.get(at), right.get(at)) {
            (Some(left), Some(right)) => field_diffs(left, right),
            (Some(left), None) => left
                .fields
                .iter()
                .map(|field| (field.key.clone(), field.value.clone(), ABSENT.to_string()))
                .collect(),
            (None, Some(right)) => right
                .fields
                .iter()
                .map(|field| (field.key.clone(), ABSENT.to_string(), field.value.clone()))
                .collect(),
            (None, None) => Vec::new(),
        };
        if fields.is_empty() {
            continue;
        }
        if first.is_none() {
            let (line, other) = (left.get(at), right.get(at));
            let tick = tick_label(line, other, at);
            first = Some(Diff {
                tick,
                fields,
                further: 0,
            });
        } else {
            further += 1;
        }
    }
    let mut first = first?;
    first.further = further;
    Some(first)
}

fn tick_label(left: Option<&TickLine>, right: Option<&TickLine>, at: usize) -> String {
    let probe = match (left, right) {
        (Some(line), Some(other)) => line.tick().or_else(|| other.tick()),
        (Some(line), None) => line.tick(),
        (None, Some(line)) => line.tick(),
        (None, None) => None,
    };
    let fallback = at + 1;
    probe.map_or_else(|| fallback.to_string(), |tick| tick.to_string())
}

fn field_diffs(left: &TickLine, right: &TickLine) -> Vec<(String, String, String)> {
    let mut out = Vec::new();
    for (at, field) in left.fields.iter().enumerate() {
        match right.fields.get(at) {
            Some(other) if other.key != field.key || other.value != field.value => {
                out.push((field.key.clone(), field.value.clone(), other.value.clone()));
            }
            Some(_) => {}
            None => out.push((field.key.clone(), field.value.clone(), ABSENT.to_string())),
        }
    }
    for other in right.fields.iter().skip(left.fields.len()) {
        out.push((other.key.clone(), ABSENT.to_string(), other.value.clone()));
    }
    out
}

pub fn line_of(world: &World) -> String {
    let mut line = String::new();
    let _ = write!(
        line,
        "tick={} stage={} room={}",
        world
            .get_resource::<frame::CurrentFrame>()
            .map_or(0, |tick| tick.0),
        stage_label(world),
        world
            .get_resource::<door::StayRoom>()
            .map_or(0, |room| room.0),
    );

    let area = world.get_resource::<area::Area>();
    let held = world
        .iter_entities()
        .find_map(|entity| entity.get::<player::Player>().copied());
    let (pos, yaw, face, speed, speed_y) = match held {
        Some(held) => (held.pos, held.yaw, held.face, held.speed, held.speed_y),
        None => (
            area.map_or(glam::Vec3::ZERO, |area| area.spawn),
            area.map_or(0.0, |area| area.yaw),
            area.map_or(0.0, |area| area.yaw),
            0.0,
            0.0,
        ),
    };
    push_bits(&mut line, "px", pos.x);
    push_bits(&mut line, "py", pos.y);
    push_bits(&mut line, "pz", pos.z);
    let _ = write!(line, " yaw={} face={}", yaw.to_bits(), face.to_bits());
    push_bits(&mut line, "speed", speed);
    push_bits(&mut line, "speed_y", speed_y);

    let proc = world
        .get_resource::<link_proc::LinkProc>()
        .map_or(link_proc::Proc::Wait, |proc| proc.current);
    let _ = write!(line, " proc={proc:?}");

    let (units, max) = world
        .get_resource::<health::Health>()
        .map_or((0, 0), |health| (health.units, health.max));
    let _ = write!(line, " hp={units} hpmax={max}");
    let _ = write!(
        line,
        " rupees={}",
        world
            .get_resource::<pickup::Rupees>()
            .map_or(0, |rupees| rupees.count)
    );

    push_reg(&mut line, "sw", &switches_bytes(world));
    push_reg(&mut line, "ef", event_bytes(world));
    push_reg(&mut line, "sb", story_bytes(world));

    match world
        .get_resource::<event::ActiveEvent>()
        .and_then(event::ActiveEvent::running_front)
    {
        Some((index, stage)) => {
            let _ = write!(line, " evt={index} evtst={stage:?}");
        }
        None => line.push_str(" evt=- evtst=-"),
    }
    let talk_open = world
        .get_resource::<talk::Dialogue>()
        .map_or(false, |talk| talk.open);
    let _ = write!(line, " talk={} rng=0x00000000", u8::from(talk_open));

    let Some(sheet) = world.get_resource::<registry::ActorRegistry>() else {
        return line;
    };
    let mut tokens: Vec<(u32, String)> = Vec::new();
    for entry in sheet.iter() {
        let Some(data) = world.get::<actor::Actor>(entry.entity) else {
            continue;
        };
        let hp = world
            .get::<combat::HitPoints>(entry.entity)
            .map_or(0, |hp| hp.0);
        tokens.push((entry.proc_id, actor_token(data, hp)));
    }
    tokens.sort_by_key(|token| token.0);
    for (proc_id, token) in tokens {
        let _ = write!(line, " a{proc_id}:{token}");
    }
    line
}

fn stage_label(world: &World) -> &str {
    let name = world
        .get_resource::<area::StageName>()
        .map_or("", |stage| stage.0.as_str());
    if name.is_empty() { "-" } else { name }
}

fn switches_bytes(world: &World) -> [u8; 16] {
    let Some(switches) = world.get_resource::<flags::Switches>() else {
        return [0; 16];
    };
    let mut bytes = [0; 16];
    for (at, word) in switches.words().iter().enumerate() {
        bytes[at * 4..at * 4 + 4].copy_from_slice(&word.to_le_bytes());
    }
    bytes
}

fn event_bytes(world: &World) -> &[u8] {
    match world.get_resource::<flags::EventFlags>() {
        Some(list) => &list.bytes,
        None => &[],
    }
}

fn story_bytes(world: &World) -> &[u8] {
    match world.get_resource::<flags::StoryBits>() {
        Some(bits) => &bits.bytes,
        None => &[],
    }
}

fn actor_kind(name: &str) -> u32 {
    let mut hash = 2166136261;
    for byte in name.as_bytes() {
        hash ^= u32::from(*byte);
        hash = hash.wrapping_mul(16777619);
    }
    hash
}

fn actor_token(data: &actor::Actor, hp: i32) -> String {
    format!(
        "kind={},x=0x{:08x},y=0x{:08x},z=0x{:08x},yaw={},hp={hp}",
        actor_kind(&data.name),
        data.pos.x.to_bits(),
        data.pos.y.to_bits(),
        data.pos.z.to_bits(),
        data.angle[1],
    )
}

fn push_bits(line: &mut String, key: &str, value: f32) {
    let _ = write!(line, " {key}=0x{:08x}", value.to_bits());
}

fn push_reg(line: &mut String, key: &str, bytes: &[u8]) {
    line.push(' ');
    line.push_str(key);
    line.push_str("=0x");
    match bytes.iter().position(|byte| *byte != 0) {
        Some(first) => {
            for byte in &bytes[first..] {
                let _ = write!(line, "{byte:02x}");
            }
        }
        None => line.push('0'),
    }
}
