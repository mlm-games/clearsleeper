use bevy_ecs::prelude::*;

pub const ROTEN_ACTORS: [&str; 3] = ["Roten2", "Roten3", "Roten4"];

const TYPE_SHIFT: u32 = 24; // <- d_a_obj_roten.cpp:102
const TYPE_MASK: u32 = 0xFF; // <- d_a_obj_roten.cpp:102 (mType is a u8)
const TYPE_MAX: u8 = 2; // <- d_a_obj_roten.cpp:103 (cLib_minMaxLimit<u8>)
const RESERVE_LAST: u16 = 0xB; // <- d_a_obj_roten.cpp:74
const TYPE0_MIN: i32 = 3; // <- d_a_obj_roten.cpp:83
const TYPE1_MIN: i32 = 6; // <- d_a_obj_roten.cpp:87
const ANY_MIN: i32 = 9; // <- d_a_obj_roten.cpp:91

/// `Roten_create_check` (`d_a_obj_roten.cpp:71-96`). `temp` counts the player's get-bag reserve
/// bits `0..=0xB`; slot 0 counts whether or not its bit is set, exactly as the `else if (i == 0)`
/// at `:78-80` does. The sim keeps those bits in `Zunari::roten_reserve` (`zunari.rs:325`),
/// which `Zunari::exec_item_get` (`zunari.rs:834-842`) sets from `item - ROTEN_FIRST` for each
/// bought shop item - the same twelve items `d_item.cpp:1199-1265` turns into
/// `dComIfGs_onGetItemReserve(0x0 .. 0xB)`.
fn create_check(kind: u8, reserve: u16) -> bool {
    let mut temp = 0;
    for slot in 0..=RESERVE_LAST {
        if (reserve >> slot) & 1 == 1 || slot == 0 {
            temp += 1;
        }
    }
    if kind == 0 && temp > TYPE0_MIN {
        return true;
    }
    if kind == 1 && temp > TYPE1_MIN {
        return true;
    }
    temp > ANY_MIN
}

/// Zunari's shop models - the wooden posts and the curtain frames on top of them. The stage
/// strings are `Roten2`, `Roten3` and `Roten4` (`d_stage.cpp:1084-1086`); the archive is
/// `Roten` (`d_a_obj_roten.cpp:22`) and the three posts/curtains are `ROTEN02..04`
/// (`:38-47`).
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Roten {
    /// `_create` (`:105-111`) returns `cPhs_ERROR_e` when `Roten_create_check` fails, which
    /// cancels the process during Create, so retail never runs `_execute` on that post at all.
    /// See `retag.rs:16-18` for the same shape.
    pub refused: bool,
}

impl Roten {
    /// `_create` (`d_a_obj_roten.cpp:99-130`). Everything after the `Roten_create_check` test
    /// is archive loading, `dBgW` moving-background registration and matrix setup (`:112-127`),
    /// and `_execute` (`:151-156`) is `set_mtx()` plus `mpBgW->Move()` - both renderer-only and
    /// `dBgW`-only, so nothing else in this component is ever read.
    pub fn spawn(parameters: u32, reserve: u16) -> Self {
        let kind = (((parameters >> TYPE_SHIFT) & TYPE_MASK) as u8).min(TYPE_MAX);
        Self {
            refused: !create_check(kind, reserve),
        }
    }
}

pub fn is_roten(name: &str) -> bool {
    ROTEN_ACTORS.contains(&name)
}

pub fn control(posts: Query<(Entity, &Roten)>, mut commands: Commands) {
    for (entity, post) in &posts {
        if post.refused {
            let _ = commands.entity(entity).despawn();
        }
    }
}
