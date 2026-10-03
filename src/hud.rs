//! Hearts HUD: health state → heart slots drawn from layout primitives.

use std::f32::consts::{FRAC_PI_4, SQRT_2};

use repose_core::{Color, Modifier, Sp, Transform, UnitExt, View};
use repose_ui::{Box as UiBox, Row, Text, TextStyle, ViewExt, ZStack};

use crate::sim::health::Health;
use crate::sim::item::{Held, Kind};

const HEART_SIZE: f32 = 24.0;
const HEART_GAP: f32 = 5.0;
const HUD_PAD: f32 = 18.0;
/// Placeholder red until user-supplied art lands.
const FULL_HEX: &str = "#E8453C";
/// Placeholder empty-container tone (dark warm grey).
const EMPTY_HEX: &str = "#453033";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeartState {
    Full,
    ThreeQuarters,
    Half,
    Quarter,
    Empty,
}

/// One entry per heart container, left to right: 4 units fill a slot and the
/// remainder shows its quarter. Health beyond `max` cannot occur (clamped at the source).
pub fn heart_states(health: &Health) -> Vec<HeartState> {
    let slots = health.max.div_ceil(4) as usize;
    let units = health.units as usize;
    (0..slots)
        .map(|slot| match units.saturating_sub(slot * 4) {
            0 => HeartState::Empty,
            1 => HeartState::Quarter,
            2 => HeartState::Half,
            3 => HeartState::ThreeQuarters,
            _ => HeartState::Full,
        })
        .collect()
}

/// Heart silhouette from primitives: rotated square (bottom point) behind
/// two clipped circles (lobes).
fn heart_shapes(color: Color, size: f32) -> Vec<View> {
    let lobe = size * 0.6;
    let side = size * 0.58;
    let diamond = UiBox(
        Modifier::new()
            .size(side.dp(), side.dp())
            .background(color)
            .transform(Transform {
                rotate: FRAC_PI_4,
                ..Transform::identity()
            })
            .absolute()
            .offset(
                Some(((size - side) / 2.0).dp()),
                Some((size - side * SQRT_2).dp()),
                None,
                None,
            ),
    );
    let lobe_at = |left: f32| {
        UiBox(
            Modifier::new()
                .size(lobe.dp(), lobe.dp())
                .background(color)
                .clip_rounded((lobe / 2.0).dp())
                .absolute()
                .offset(Some(left.dp()), Some(0.0.dp()), None, None),
        )
    };
    vec![diamond, lobe_at(0.0), lobe_at(size - lobe)]
}

fn heart(state: HeartState, size: f32) -> View {
    let frame = || Modifier::new().size(size.dp(), size.dp());
    let full = Color::from_hex(FULL_HEX);
    let empty = Color::from_hex(EMPTY_HEX);
    let filled_to = |width: f32| {
        ZStack(frame()).child([
            ZStack(frame()).child(heart_shapes(empty, size)),
            UiBox(
                Modifier::new()
                    .absolute()
                    .offset(Some(0.0.dp()), Some(0.0.dp()), None, None)
                    .size(width.dp(), size.dp())
                    .clip_rounded(0.0.dp()),
            )
            .child(heart_shapes(full, size)),
        ])
    };
    match state {
        HeartState::Full => ZStack(frame()).child(heart_shapes(full, size)),
        HeartState::Empty => ZStack(frame()).child(heart_shapes(empty, size)),
        HeartState::Quarter => filled_to(size / 4.0),
        HeartState::Half => filled_to(size / 2.0),
        HeartState::ThreeQuarters => filled_to(size * 3.0 / 4.0),
    }
}

/// Top-left heart row; passthrough keeps orbit drags alive underneath.
pub fn hearts_view(health: Health) -> View {
    Row(Modifier::new()
        .absolute()
        .offset(Some(HUD_PAD.dp()), Some(HUD_PAD.dp()), None, None)
        .gap(HEART_GAP.dp())
        .hit_passthrough())
    .child(heart_cells(health, HEART_SIZE))
}

pub fn hearts_strip(health: Health, size: f32) -> View {
    Row(Modifier::new().gap(HEART_GAP.dp()).hit_passthrough()).child(heart_cells(health, size))
}

const PURSE_HEX: &str = "#3FA94B";

pub fn status_view(health: Health, rupees: u32, held: Option<Held>) -> View {
    let mut stack = vec![hearts_view(health), purse_view(rupees)];
    if let Some(held) = held {
        stack.push(held_view(held));
    }
    ZStack(Modifier::new().fill_max_size().hit_passthrough()).child(stack)
}

fn purse_view(rupees: u32) -> View {
    Row(Modifier::new()
        .absolute()
        .offset(
            Some(HUD_PAD.dp()),
            Some((HUD_PAD + HEART_SIZE + HEART_GAP).dp()),
            None,
            None,
        )
        .gap(4.0.dp())
        .hit_passthrough())
    .child([
        rupee_gem(HEART_SIZE * 0.75),
        Text(format!("{rupees}"))
            .size(Sp(20.0))
            .color(Color::from_hex("#F2EDE3")),
    ])
}

fn held_view(held: Held) -> View {
    let hex = match held.kind() {
        Some(Kind::Bomb) => "#2E3138",
        Some(Kind::Arrow) => "#C9C4B4",
        Some(Kind::Boomerang) => "#B84C85",
        None => "#2E3138",
    };
    let label = held.label();
    Row(Modifier::new()
        .absolute()
        .offset(
            Some(HUD_PAD.dp()),
            Some((HUD_PAD + HEART_SIZE + HEART_GAP + 28.0).dp()),
            None,
            None,
        )
        .gap(4.0.dp())
        .hit_passthrough())
    .child([
        UiBox(
            Modifier::new()
                .size(14.0.dp(), 14.0.dp())
                .background(Color::from_hex(hex)),
        ),
        Text(format!("{label} {}", held.count))
            .size(Sp(16.0))
            .color(Color::from_hex("#F2EDE3")),
    ])
}

fn rupee_gem(size: f32) -> View {
    UiBox(
        Modifier::new()
            .size(size.dp(), size.dp())
            .background(Color::from_hex(PURSE_HEX))
            .transform(Transform {
                rotate: FRAC_PI_4,
                ..Transform::identity()
            }),
    )
}

fn heart_cells(health: Health, size: f32) -> Vec<View> {
    heart_states(&health)
        .into_iter()
        .map(|state| heart(state, size))
        .collect()
}
