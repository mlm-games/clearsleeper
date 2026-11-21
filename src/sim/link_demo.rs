//! The player's cutscene animation: which `LkAnm` entry a demo mode plays.
//!
//! `daPy_lk_c::setDemoData` copies the bound demo actor's animation id into the player's
//! `mDemo` mode (`demo_actor_p->getAnmId()`, `d_a_player_main.cpp:10325`), and
//! `changeDemoProc` (`:5379-5400`) dispatches on that `daPy_demo_c::Mode_e` value to the demo
//! process whose init selects one row of `mAnmDataTable` (`d_a_player_main_data.inc:1031`).
//! Each row names two bcks, an under and an upper body, and for the modes below the two are the
//! same entry - which is why one stem answers both halves: the upper-body blend composes the same
//! locals over the same parent as the under animation.
//!
//! The stems are the `LkAnm` archive's own entry names (`bcks/<stem>.bck`), the ones
//! [`crate::sim::event::LINK_ANIM_STEMS`] already reads frame counts for.
//!
//! The modes this table does not answer keep whatever pose the player was drawn with:
//!
//! * the idle, `DEMO_N_WAIT` and `DEMO_INIT_WAIT`, is the standing pair the shell already poses
//!   Link on, so there is nothing to switch;
//! * `DEMO_OPEN_TREASURE` picks between two stems on a demo parameter the mode does not carry:
//!   `dProcOpenTreasure_init` reads `mDemo.getParam0()` and sets `ANM_BOXOPENSHORTLINK` when it is
//!   one, `ANM_BOXOPENLINK` otherwise (`d_a_player_dproc.inc:516-526`);
//! * the bottle's open is two bcks in sequence - `ANM_BINOPENPRE` (`:2155`), then `ANM_BINOPENB`
//!   once the first ends (`:2172-2176`) - and this one-stem table has no second row to give it;
//! * `DEMO_GRAB_PUT`'s other arm, `ANM_GRABWAIT`, needs `checkGrabWear()` (`d_a_player.h:596`),
//!   which is the barrel-on-back lean `field_0x2b0` carries: the port has no barrel-on-back
//!   process, so the value never goes negative and `ANM_GRABUP` is the arm that plays.
//!
//! `DEMO_N_WALK` and `DEMO_N_DASH` are answered only while the demo's stick drives the player:
//! retail picks `ANM_WALK` or `ANM_DASH` from the speed the walk reaches (`d_a_player_main.cpp:3023`,
//! `:3035`), and a walk cut with a centred stick stands in the wait pose like any other.

use bevy_ecs::prelude::*;

/// Every stem [`stem`] can answer, for the layer that loads them.
pub const STEMS: &[&str] = &[
    "boxopenslink",
    "combo_link",
    "comeout",
    "damf",
    "damffup",
    "dash",
    "esamaki",
    "getletter",
    "glad",
    "grabup",
    "grabwait",
    "holdup",
    "itemget",
    "lavadam",
    "lookup",
    "pickup",
    "powup",
    "presentationa",
    "saltation",
    "sethyoinomi",
    "seyyes",
    "slipice",
    "surprised",
    "surprisedb",
    "taktdge",
    "talka",
    "turnback",
    "waitpickup",
    "waitq",
    "walk",
    "walkpush",
    "who",
    "windl",
];

/// `setAnm`'s `f1` (`d_demo.cpp:229`) advances the bck one frame per 60 Hz tick, and this sim
/// steps at 30 Hz ([`crate::SIM_HZ`]), so one step is two bck frames.
const FRAME_RATE: f32 = 2.0;

/// The bck a demo step put the player on, and how far into it the demo is.
///
/// Retail has no such value to read: the demo process selected by `changeDemoProc` owns the
/// morf and `mpCLModel` samples it. This port's player model is posed by the shell, so the
/// selection is carried out of the sim as the stem plus its frame.
#[derive(Resource, Debug, Clone, Copy, Default, PartialEq)]
pub struct PlayerAnm {
    pub stem: Option<&'static str>,
    pub frame: f32,
}

impl PlayerAnm {
    /// The demo actor's animation id as the player's mode: a different id restarts the bck at
    /// frame 0, the same id advances it, and no id - no demo, or a mode [`stem`] does not answer -
    /// leaves the standing pose. `moving` is the demo's stick, which is what decides between the
    /// walk pose and the standing one for the two move modes.
    pub fn apply(&mut self, mode: Option<i32>, moving: bool) {
        let stem = match mode {
            Some(2) if moving => Some("walk"),
            Some(3) if moving => Some("dash"),
            mode => mode.and_then(stem),
        };
        if stem != self.stem {
            self.stem = stem;
            self.frame = 0.0;
        } else if stem.is_some() {
            self.frame += FRAME_RATE;
        }
    }
}

pub fn init(world: &mut World) {
    world.init_resource::<PlayerAnm>();
}

/// The `LkAnm` stem a demo mode plays, `None` for a mode this table does not answer - see the
/// module docs for the three groups it leaves alone.
///
/// The mode numbers are `daPy_demo_c::Mode_e`'s; `changeDemoProc` and the demo process inits are
/// what pair each one with its animation.
pub fn stem(mode: i32) -> Option<&'static str> {
    Some(match mode {
        // `DEMO_N_TALK_e`: `dProcTalk_init`'s talk pose, `ANM_TALKA`
        // (`d_a_player_dproc.inc:363`). `DEMO_E_TALK_e` (8) is the standing pair instead, which
        // is the pose the fallback already draws.
        6 => "talka",
        // `DEMO_DAM_ACTION_e` and `DEMO_LDAM_e` / `DEMO_UNK_027_e`.
        7 => "damf",
        9 | 27 => "damffup",
        // `DEMO_GET_ITEM_e`: `dProcGetItem_init`'s `ANM_ITEMGET` (`d_a_player_dproc.inc:678`).
        11 => "itemget",
        // `DEMO_HOLDUP_e`.
        13 => "holdup",
        // `DEMO_LAROUND_e`: `dProcLookAround_init` (`d_a_player_dproc.inc:1145`), `ANM_COMEOUT`.
        15 => "comeout",
        // `DEMO_SALUTE_e`.
        19 => "saltation",
        // `DEMO_L_AROUND2_e`: `dProcLookAround2_init`'s `ANM_WHO` (`d_a_player_dproc.inc:1195`).
        20 => "who",
        // `DEMO_T_PICKUP_e`.
        21 => "pickup",
        // `DEMO_T_WAIT_e`: `dProcTalismanWait_init`'s `ANM_WAITPICKUP`
        // (`d_a_player_dproc.inc:1251`).
        22 => "waitpickup",
        // `DEMO_SURPRISED_e`.
        24 => "surprised",
        // `DEMO_TBACK_e`.
        25 => "turnback",
        // `DEMO_LOOKUP_e`, the mode the prologue's telescope talk leans on.
        26 => "lookup",
        // `DEMO_UNK_028_e`: `dProcQuakeWait_init`'s `ANM_WAITQ` (`d_a_player_dproc.inc:1392`).
        28 => "waitq",
        // `DEMO_UNK_029_e`: `dProcDance_init`'s `ANM_GLAD` (`d_a_player_dproc.inc:1410`).
        29 => "glad",
        // `DEMO_PUSH_e`: `dProcPushMove_init`'s `ANM_WALKPUSH` (`d_a_player_dproc.inc:1514`).
        33 => "walkpush",
        // `DEMO_NOD_e`.
        36 => "seyyes",
        // `DEMO_PRESENT_e`.
        37 => "presentationa",
        // `DEMO_WCHANGE_e` and `DEMO_LTURN_e`.
        38 | 53 => "windl",
        // `DEMO_SIPUT_e`: `dProcStandItemPut_init`'s `ANM_GRABWAIT`
        // (`d_a_player_dproc.inc:1737`).
        40 => "grabwait",
        // `DEMO_PGET_e`.
        46 => "powup",
        // `DEMO_VOLCANO_e`: `dProcVorcanoFail_init`'s `ANM_LAVADAM`
        // (`d_a_player_dproc.inc:860`).
        47 => "lavadam",
        // `DEMO_S_SURP_e`.
        49 => "surprisedb",
        // `DEMO_LET_OPEN_e`.
        54 => "getletter",
        // `DEMO_GRAB_PUT_e`: `procGrabPut_init` (`d_a_player_grab.inc:602-624`) branches on
        // `checkGrabWear()`, and the port has no barrel-on-back (see the module docs).
        56 => "grabup",
        // `DEMO_GET_DANCE_e`.
        59 => "taktdge",
        // `DEMO_OPEN_SBOX_e`.
        63 => "boxopenslink",
        // `DEMO_FOODSET_e`.
        65 => "sethyoinomi",
        // `DEMO_UNK_070_e`.
        70 => "combo_link",
        // `DEMO_FOODTHROW_e`.
        73 => "esamaki",
        // `DEMO_UNK_074_e`.
        74 => "slipice",
        _ => return None,
    })
}
