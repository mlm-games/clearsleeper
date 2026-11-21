use bevy_ecs::prelude::*;

use super::boomerang::BoomCatch;
use super::combat::{Combat, CutKind};
use super::death::GameOver;
use super::events::{SimEvent, SimEvents};
use super::hammer::{Hammer, Swing};
use super::hookshot::Hookshot;
use super::item_use::ItemUse;
use super::jump::Jump;
use super::link_proc::{LinkProc, Proc};
use super::player::Player;
use super::roll::Roll;
use super::swim::Swim;

/// The bck the player's body is on during normal play, separate from the cutscene morf so a demo
/// can keep its own one-shot animation without either writer clearing the other.
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct PlayerPose {
    pub stem: Option<&'static str>,
    pub frame: f32,
}

fn advance(pose: &mut PlayerPose, stem: &'static str) {
    if pose.stem != Some(stem) {
        pose.stem = Some(stem);
        pose.frame = 0.0;
    } else {
        pose.frame += 2.0;
    }
}

const HURT_FRAMES: u32 = 12;

/// Flinch frames set from the frame's `Hurt` event and read by the body pose on the next tick.
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct HurtReaction {
    pub frames: u32,
}

pub fn hurt_reaction(events: Res<SimEvents>, mut reaction: ResMut<HurtReaction>) {
    if events.list.contains(&SimEvent::Hurt) {
        reaction.frames = HURT_FRAMES;
    }
}

pub fn control(
    combat: Res<Combat>,
    proc: Option<Res<LinkProc>>,
    item_use: Res<ItemUse>,
    swim: Res<Swim>,
    roll: Res<Roll>,
    jump: Res<Jump>,
    game_over: Res<GameOver>,
    mut hurt: ResMut<HurtReaction>,
    mut boom_catch: ResMut<BoomCatch>,
    players: Query<(&Player, Option<&Hammer>)>,
    shots: Query<&Hookshot>,
    mut pose: ResMut<PlayerPose>,
) {
    let hurt_left = hurt.frames;
    if hurt.frames > 0 {
        hurt.frames -= 1;
    }
    let catch_left = boom_catch.0;
    if boom_catch.0 > 0 {
        boom_catch.0 -= 1;
    }
    let (speed, hammer) = players
        .iter()
        .next()
        .map_or((0.0, None), |(player, hammer)| (player.speed.abs(), hammer));
    let current = proc.as_deref().map(|proc| proc.current);
    let proc_frame = proc.as_ref().map_or(0, |proc| proc.frame) as f32 * 2.0;
    // tww/src/d/actor/d_a_player_main_data.inc:1718-1738 and d_a_player_dproc.inc:1015-1028;
    // wwhd/wwhd_src/d/actor/d_a_player_main.cpp:111-113,
    // wwhd/wwhd_src/d/actor/d_a_player_main_03.cpp:2612-2629.
    if game_over.active {
        advance(&mut pose, if swim.active { "swimdie" } else { "dielong" });
        return;
    }
    // The player's named() dispatch checks swim before combat and the airborne states before the
    // move states (`link_move.rs:967-986`); mirror that order so one state cannot erase another.
    if swim.active {
        // tww/src/d/actor/d_a_player_swim.inc:343,422;
        // wwhd/wwhd_src/d/actor/d_a_player_main_03.cpp:3071,3033.
        advance(&mut pose, if swim.surfaced { "swimwait" } else { "swimp" });
        return;
    }
    // tww/src/d/actor/d_a_player_main_data.inc:1620-1710, d_a_player_main.cpp:7525-7554;
    // wwhd/wwhd_src/d/actor/d_a_player_main.cpp:97-110,
    // wwhd/wwhd_src/d/actor/d_a_player_main_03.cpp:3365-3376.
    if hurt_left > 0 {
        pose.stem = Some("damf");
        pose.frame = (HURT_FRAMES - hurt_left) as f32 * 2.0;
        return;
    }
    // Each sword process installs one of these body animations for its whole run:
    // tww/src/d/actor/d_a_player_sword.inc:561-562,663-664,765-766,866-867,965-966,1049-1050;
    // wwhd/wwhd_src/d/actor/d_a_player_main_02.cpp:905-1064.
    if combat.active {
        let stem = match combat.kind {
            CutKind::A => "cuta",
            CutKind::F => "cutf",
            CutKind::R => "cutr",
            CutKind::L => "cutl",
            CutKind::EA => "cutea",
            CutKind::EB => "cuteb",
        };
        pose.stem = Some(stem);
        // The cut's own tick counter is its frame control; the bck advances two frames per tick
        // at this sim rate, starting from the table's frame 4.
        pose.frame = 4.0 + combat.frame as f32 * 2.0;
        return;
    }
    // tww/src/d/actor/d_a_player_main_data.inc:2222-2250 and d_a_player_hammer.inc:171-177,241-247,
    // 288-294,388-394; wwhd/wwhd_src/d/actor/d_a_player_main.cpp:183-186,
    // wwhd/wwhd_src/d/actor/d_a_player_main_03.cpp:237,255.
    if let Some(hammer) = hammer.filter(|hammer| hammer.swing != Swing::Idle) {
        let stem = match hammer.swing {
            Swing::Side => "hamswinga",
            Swing::FrontReady => "hamswingbpre",
            Swing::Front => "hamswingbhit",
            Swing::FrontEnd => "hamswingbend",
            Swing::Idle => unreachable!(),
        };
        pose.stem = Some(stem);
        pose.frame = hammer.frame;
        return;
    }
    if let Some(stem) = item_use.stem {
        pose.stem = Some(stem);
        pose.frame = item_use.frame as f32 * 2.0;
        return;
    }
    // tww/src/d/actor/d_a_player_main_data.inc:1900-1906, d_a_player_boomerang.inc:240-256;
    // wwhd/wwhd_src/d/actor/d_a_player_main.cpp:137,
    // wwhd/wwhd_src/d/actor/d_a_player_main_02.cpp:377.
    if catch_left > 0 {
        pose.stem = Some("boomcatch");
        pose.frame = (super::boomerang::BOOMCATCH_FRAMES - catch_left) as f32 * 2.0;
        return;
    }
    // tww/src/d/actor/d_a_player_main_data.inc:2194-2200 and d_a_player_hook.inc:304-325;
    // wwhd/wwhd_src/d/actor/d_a_player_main.cpp:179,
    // wwhd/wwhd_src/d/actor/d_a_player_main_02.cpp:2333.
    if shots.iter().any(|shot| {
        matches!(
            shot.phase,
            super::hookshot::Phase::Shot
                | super::hookshot::Phase::Pull
                | super::hookshot::Phase::Return
        )
    }) {
        advance(&mut pose, "hookshotjmp");
        return;
    }
    if roll.active || matches!(current, Some(Proc::FrontRoll | Proc::FrontRollCrash)) {
        // tww/src/d/actor/d_a_player_main.cpp:6807,6880;
        // wwhd/wwhd_src/d/actor/d_a_player_main_03.cpp:415,
        // wwhd/wwhd_src/d/actor/d_a_player_main_05.cpp:235,550.
        let crash = current == Some(Proc::FrontRollCrash);
        pose.stem = Some(if crash { "rollfmis" } else { "rollf" });
        pose.frame = if crash { proc_frame } else { roll.frame };
        return;
    }
    if matches!(
        current,
        Some(Proc::AutoJump | Proc::BackJump | Proc::SmallJump | Proc::VerticalJump)
    ) {
        // tww/src/d/actor/d_a_player_main.cpp:7099,7422;
        // wwhd/wwhd_src/d/actor/d_a_player_main_03.cpp:2681.
        pose.stem = Some("jmpst");
        pose.frame = proc_frame;
        return;
    }
    // tww/src/d/actor/d_a_player_main_data.inc:1368-1381, d_a_player_main.cpp:7258,7285;
    // wwhd/wwhd_src/d/actor/d_a_player_main.cpp:61,62,
    // wwhd/wwhd_src/d/actor/d_a_player_main_05.cpp:838,851.
    if current == Some(Proc::LandDamage) {
        let stem = if proc.as_deref().is_some_and(|proc| proc.land_stand) {
            "landdamast"
        } else {
            "landdama"
        };
        advance(&mut pose, stem);
        return;
    }
    // Fall and land reuse the same paired bck as the jump's airborne phase in both trees:
    // tww/src/d/actor/d_a_player_main.cpp:7181,7325;
    // wwhd/wwhd_src/d/actor/d_a_player_main_03.cpp:2237,2681.
    if jump.active
        || matches!(
            current,
            Some(
                Proc::Fall | Proc::Land | Proc::BackJumpLand | Proc::SideStepLand | Proc::SlowFall
            )
        )
    {
        advance(&mut pose, "jmped");
        return;
    }
    // tww/src/d/actor/d_a_player_main.cpp:6508,6390 and d_a_player_crawl.inc:350;
    // wwhd/wwhd_src/d/actor/d_a_player_main_03.cpp:294,
    // wwhd/wwhd_src/d/actor/d_a_player_main_04b.cpp:675.
    match current {
        Some(Proc::CrouchDefense | Proc::CrouchDefenseSlip) => {
            advance(&mut pose, "difence");
            return;
        }
        Some(Proc::CrawlMove | Proc::CrawlAutoMove) => {
            advance(&mut pose, "lieforward");
            return;
        }
        Some(Proc::Crouch | Proc::CrawlStart | Proc::CrawlEnd) => {
            advance(&mut pose, "crouch");
            return;
        }
        _ => {}
    }
    // tww/src/d/actor/d_a_player_hang.inc:425 and d_a_player_ladder.inc:94,207;
    // wwhd/wwhd_src/d/actor/d_a_player_main_04b.cpp:1150,1158.
    match current {
        Some(Proc::HangUp | Proc::HangClimb) => {
            advance(&mut pose, "hangup");
            return;
        }
        Some(
            Proc::HangStart
            | Proc::HangFallStart
            | Proc::HangWait
            | Proc::HangMove
            | Proc::HangWallCatch,
        ) => {
            advance(&mut pose, "hanging");
            return;
        }
        Some(
            Proc::LadderUpStart
            | Proc::LadderUpEnd
            | Proc::LadderMove
            | Proc::ClimbUpStart
            | Proc::ClimbMoveUpDown
            | Proc::ClimbMoveSide,
        ) => {
            advance(&mut pose, "ladderupst");
            return;
        }
        Some(Proc::LadderDownStart | Proc::LadderDownEnd | Proc::ClimbDownStart) => {
            advance(&mut pose, "ladderdwst");
            return;
        }
        _ => {}
    }
    // tww/src/d/actor/d_a_player_main_data.inc:1921-1934 and d_a_player_pushpull.inc:129,172;
    // wwhd/wwhd_src/d/actor/d_a_player_main.cpp:140,141,
    // wwhd/wwhd_src/d/actor/d_a_player_main_07.cpp:4349,4361.
    match current {
        Some(Proc::PushMove) => {
            advance(&mut pose, "walkpush");
            return;
        }
        Some(Proc::PullMove) => {
            advance(&mut pose, "walkpull");
            return;
        }
        _ => {}
    }
    // tww/src/d/actor/d_a_player_main_data.inc:1403-1429, d_a_player_main.cpp:6693,6727,6756,6782;
    // wwhd/wwhd_src/d/actor/d_a_player_main.cpp:66-69,
    // wwhd/wwhd_src/d/actor/d_a_player_main_02.cpp:506,521.
    match current {
        Some(Proc::SlideFront) => {
            advance(&mut pose, "slidef");
            return;
        }
        Some(Proc::SlideFrontLand) => {
            advance(&mut pose, "slidefland");
            return;
        }
        Some(Proc::SlideBack) => {
            advance(&mut pose, "slideb");
            return;
        }
        Some(Proc::SlideBackLand) => {
            advance(&mut pose, "slidebland");
            return;
        }
        _ => {}
    }
    // A moving body is WALK below the dash band and DASH above it; a standing body keeps its
    // WAITS/WAIT pair. tww/src/d/actor/d_a_player_main.cpp:2968-3045;
    // wwhd/wwhd_src/d/actor/d_a_player_main_02.cpp:3567-3765.
    let moving = speed > 0.05
        && proc.as_deref().is_some_and(|proc| {
            matches!(
                proc.current,
                Proc::Move | Proc::AtnMove | Proc::AtnActorMove | Proc::SideStep
            )
        });
    if moving {
        let stem = if speed > 12.0 { "dash" } else { "walk" };
        advance(&mut pose, stem);
    } else {
        pose.stem = None;
        pose.frame = 0.0;
    }
}
