use bevy_ecs::prelude::*;

use super::flags::Collect;
use super::intent::{CStick, PadButtons, UseIntent};
use super::inventory::{Equipped, ITEM_WIND_WAKER, WAKER_SLOT};
use super::weather::{self, Weather};

/// `daPy_lk_c`'s tact processes and the `dOperate_wind` direction wheel, flattened into one
/// state machine. The port has no note judgement, so a performance completes on an explicit
/// confirm instead (`tww/src/d/actor/d_a_player_tact.inc:149-156`,
/// `wwhd/wwhd_src/d/actor/d_a_player_main_07.cpp:3440-3463`).
#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TactPhase {
    #[default]
    Idle,
    Select,
    Play,
    Direction,
}

/// `getTactMusic()`'s value held while performing (`tww/src/d/actor/d_a_player_tact.inc:149-156`):
/// the song index during Play, `-1` outside it so a Tact plate cannot fire.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Tact {
    pub phase: TactPhase,
    pub selected: u8,
    pub music: i32,
    pub deg: i16,
    played: u32,
    pressed: bool,
    cycle: bool,
}

impl Default for Tact {
    fn default() -> Self {
        Self {
            phase: TactPhase::Idle,
            selected: 0,
            music: -1,
            deg: 0,
            played: 0,
            pressed: false,
            cycle: false,
        }
    }
}

const SONG_COUNT: u8 = 6;
const PLAY_TICKS: u32 = 24;

/// `dKyw_tact_wind_set`'s direction table, `d_operate_wind.cpp:462-488`. HD computes it
/// continuously (`wwhd/wwhd_src/d/d_operate_wind.cpp:123-127`); the port mirrors the GameCube
/// setter (`src/sim/weather.rs:1575`), so the GameCube table is authoritative here.
const WIND_ANGLES: [(i16, i16); 8] = [
    (0, -0x8000),
    (45, 0x6000),
    (90, 0x4000),
    (135, 0x2000),
    (180, 0),
    (225, -0x2000),
    (270, -0x4000),
    (315, -0x6000),
];

fn first_song(collect: &Collect) -> u8 {
    (0..SONG_COUNT)
        .find(|&song| collect.is_tact(song))
        .unwrap_or(0)
}

fn next_song(collect: &Collect, from: u8, step: i32) -> u8 {
    for offset in 1..=i32::from(SONG_COUNT) {
        let song = (i32::from(from) + step * offset).rem_euclid(i32::from(SONG_COUNT)) as u8;
        if collect.is_tact(song) {
            return song;
        }
    }
    from
}

fn bucket(angle: i16) -> (i16, i16) {
    let degrees = super::walk::from_s16(angle).to_degrees().rem_euclid(360.0);
    let nearest = (0..8)
        .map(|slot| (slot * 45) as f32)
        .min_by(|a, b| {
            let da = (*a - degrees).abs().min(360.0 - (*a - degrees).abs());
            let db = (*b - degrees).abs().min(360.0 - (*b - degrees).abs());
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap_or(0.0) as usize;
    WIND_ANGLES[nearest]
}

pub fn control(
    use_intent: Res<UseIntent>,
    pad: Res<PadButtons>,
    cstick: Res<CStick>,
    equipped: Res<Equipped>,
    collect: Res<Collect>,
    mut tact: ResMut<Tact>,
    mut weather: ResMut<Weather>,
) {
    let waker = equipped.item(WAKER_SLOT) == ITEM_WIND_WAKER;
    let use_edge = use_intent.0 && !tact.pressed;
    tact.pressed = use_intent.0;
    match tact.phase {
        TactPhase::Idle => {
            if use_edge && waker {
                tact.phase = TactPhase::Select;
                tact.selected = first_song(&collect);
                tact.cycle = false;
            }
        }
        TactPhase::Select => {
            let dir = if cstick.axis.x > 0.5 {
                1
            } else if cstick.axis.x < -0.5 {
                -1
            } else {
                0
            };
            if dir != 0 && !tact.cycle {
                tact.selected = next_song(&collect, tact.selected, dir);
            }
            tact.cycle = dir != 0;
            if pad.trig.b || (use_edge && waker) {
                tact.phase = TactPhase::Idle;
            } else if pad.trig.a && collect.is_tact(tact.selected) {
                tact.phase = TactPhase::Play;
                tact.music = i32::from(tact.selected);
                tact.played = 0;
            }
        }
        TactPhase::Play => {
            tact.played += 1;
            if pad.trig.b || tact.played >= PLAY_TICKS {
                tact.music = -1;
                if tact.selected == 0 {
                    // Wind's Requiem hands over to the direction wheel (`:624-628,676-683`).
                    tact.phase = TactPhase::Direction;
                    let (deg, _) = bucket(cstick.angle);
                    tact.deg = deg;
                } else {
                    tact.phase = TactPhase::Idle;
                }
            }
        }
        TactPhase::Direction => {
            let (deg, y) = bucket(cstick.angle);
            tact.deg = deg;
            if pad.trig.b {
                tact.phase = TactPhase::Idle;
            } else if pad.trig.a {
                weather::tact_wind_set(&mut weather, 0, y);
                weather::tact_wind_set_go(&mut weather);
                tact.phase = TactPhase::Idle;
            }
        }
    }
}
