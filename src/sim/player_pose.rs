use bevy_ecs::prelude::*;

use super::combat::{Combat, CutKind};
use super::link_proc::{LinkProc, Proc};
use super::player::Player;

/// The bck the player's body is on during normal play, separate from the cutscene morf so a demo
/// can keep its own one-shot animation without either writer clearing the other.
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct PlayerPose {
    pub stem: Option<&'static str>,
    pub frame: f32,
}

pub fn control(
    combat: Res<Combat>,
    proc: Option<Res<LinkProc>>,
    players: Query<&Player>,
    mut pose: ResMut<PlayerPose>,
) {
    let speed = players
        .iter()
        .next()
        .map_or(0.0, |player| player.speed.abs());
    // tww/src/d/actor/d_a_player_sword.inc:561-562,663-664,765-766,866-867,965-966,1049-1050:
    // each sword process installs one of these body animations for its whole run.
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
    // tww/src/d/actor/d_a_player_main.cpp:2968-3045: a moving body is WALK below the dash band
    // and DASH above it; a standing body keeps its WAITS/WAIT pair.
    let moving = speed > 0.05
        && proc.as_deref().is_some_and(|proc| {
            matches!(
                proc.current,
                Proc::Move
                    | Proc::AtnMove
                    | Proc::AtnActorMove
                    | Proc::CrawlMove
                    | Proc::CrawlAutoMove
                    | Proc::SideStep
            )
        });
    if moving {
        let stem = if speed > 12.0 { "dash" } else { "walk" };
        if pose.stem != Some(stem) {
            pose.stem = Some(stem);
            pose.frame = 0.0;
        } else {
            pose.frame += 2.0;
        }
    } else {
        pose.stem = None;
        pose.frame = 0.0;
    }
}
