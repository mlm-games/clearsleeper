//! Hearts HUD: health state → heart slots drawn from layout primitives.

use std::f32::consts::{FRAC_PI_4, SQRT_2};

use repose_core::{Color, Modifier, Transform, UnitExt, View};
use repose_ui::{Box as UiBox, Row, ViewExt, ZStack};

use crate::sim::health::Health;

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
    vec![
        diamond,
        lobe_at(0.0),
        lobe_at(size - lobe),
    ]
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
    let cells = heart_states(&health)
        .into_iter()
        .map(|state| heart(state, HEART_SIZE))
        .collect::<Vec<_>>();
    Row(
        Modifier::new()
            .absolute()
            .offset(Some(HUD_PAD.dp()), Some(HUD_PAD.dp()), None, None)
            .gap(HEART_GAP.dp())
            .hit_passthrough(),
    )
    .child(cells)
}
