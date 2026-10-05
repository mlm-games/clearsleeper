use bevy_ecs::prelude::*;

use super::flags::{Collect, SYMBOL_DIN};

pub const KYTAG05_ACTORS: [&str; 1] = ["kytag5"];

const MUFUU_TIMER: [i16; 4] = [10, 10, 0, 90]; // <- d_a_kytag05.cpp:29-34
const FUU_TIMER: [i16; 4] = [150, 150, 150, 150]; // <- d_a_kytag05.cpp:35-40
const WIND_COUNT: usize = 4; // <- d_a_kytag05.cpp:77

/// Weather tag 5. `daKytag05_Create` (`d_a_kytag05.cpp:122-145`) returns `cPhs_STOP_e` on the
/// Din symbol, and STOP cancels the create request just like ERROR
/// (`f_pc_create_req.cpp:106-108`), so `refused` records that. `mUnknownParam` (`:133`) is
/// written and never read, so it is not stored.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Kytag05 {
    pub refused: bool,
    index: u8,
    timer: i32,
}

impl Kytag05 {
    /// `mIndex`/`mTimer` come straight from `daKytag05_Create` (`d_a_kytag05.cpp:131-132`).
    pub fn spawn(collect: &Collect) -> Self {
        Self {
            refused: collect.is_symbol(SYMBOL_DIN),
            index: 0,
            timer: 0,
        }
    }

    /// The wind cycle of `daKytag05_Execute` (`d_a_kytag05.cpp:66-87`). Its whole output is
    /// `dKyw_evt_wind_set(0, wind_table[mIndex >> 1])` (`:80-81`) and the two
    /// `g_env_light.mWind.mEvtWindSet` writes (`:70`, `:83`); wind vectors and the
    /// environment light have no home in this sim, so the cycle itself is all that advances.
    fn gust(&mut self) {
        let slot = usize::from(self.index >> 1);
        if self.index & 1 == 0 {
            let Some(limit) = FUU_TIMER.get(slot) else {
                return;
            };
            if self.timer >= i32::from(*limit) {
                self.timer = 0;
                self.index = self.index.wrapping_add(1);
            } else {
                self.timer += 1;
            }
            return;
        }
        let Some(limit) = MUFUU_TIMER.get(slot) else {
            return;
        };
        if self.timer >= i32::from(*limit) {
            self.index = self.index.wrapping_add(1);
            if usize::from(self.index >> 1) >= WIND_COUNT {
                self.index = 0;
            }
            self.timer = 0;
        } else {
            self.timer += 1;
        }
    }
}

pub fn is_kytag05(name: &str) -> bool {
    KYTAG05_ACTORS.contains(&name)
}

pub fn control(mut tags: Query<(Entity, &mut Kytag05)>, mut commands: Commands) {
    for (entity, mut tag) in &mut tags {
        if tag.refused {
            let _ = commands.entity(entity).despawn();
            continue;
        }
        tag.gust();
    }
}