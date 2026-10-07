use bevy_ecs::prelude::*;
use glam::Vec3;

use super::collision::Collision;
use super::combat::{Combat, CutKind};
use super::ground::grounded;
use super::health::Health;
use super::hop::Hop;
use super::intent::{CameraYaw, MoveIntent};
use super::jump::{Direction, Jump, JumpKind, direction};
use super::link_proc::{LinkProc, Proc};
use super::lock::Lock;
use super::player::Player;
use super::roll::Roll;
use super::ship::Ship;
use super::swim::Swim;
use super::walk::{facing_error, target_yaw, to_s16};

/// `mStickDistance > 0.05f`, the gate every stick test in the reference opens with.
pub const STICK_GATE: f32 = 0.05;
/// `cLib_distanceAngleS(m34E8, current.angle.y) > 0x7800`, the reversal the turn states ask for.
pub const REVERSAL_LIMIT: u16 = 0x7800;
/// `std::abs(mNormalSpeed) <= 0.001f`, the still-speed test the wait and move turns open with.
pub const REST_SPEED: f32 = 0.001;
/// `mStickDistance > 0.5f` in the landing dispatcher, the stick hold a rolling landing asks for.
pub const LAND_STICK: f32 = 0.5;
/// `crawl.inc:311`, the ceiling a crawl needs above the player's feet before it will lie down.
pub const CRAWL_CLEARANCE: f32 = 125.0;

/// What a state needs that the port cannot answer. Every guard below asks one question about the
/// tick and reports the one thing its state cannot be entered without, rather than standing in a
/// geometry that is not there.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveGap {
    /// The guard asked for the state and the port answered it, so the state is entered.
    Ready,
    /// The turn-rate word the turn states turn by, which the port has not recovered.
    TurnRate,
    /// The roll-speed and roll-length words the rolling landing runs on.
    RollRate,
    /// The height the player left the ground at, which the landing dispatcher compares against
    /// before it will roll and which nothing in the port keeps.
    TakeoffHeight,
    /// The split between the model's own facing and the body's facing, which the turn and
    /// sidestep states rotate independently and the port's single yaw cannot carry.
    ShapeAngle,
    /// The crouch and guard button, a pad face the port's four intents do not carry.
    CrouchButton,
    /// A low-clearance probe over the player's own joints.
    LowClearance,
    /// The front wall classification the vine states are chosen by, which needs a wall trace and
    /// the wall code of whatever the trace lands on.
    WallType,
    /// A ledge: the wall plane and the ledge point where a wall meets a floor at the player's own
    /// height.
    Ledge,
    /// A ceiling above the player, which the hang states open with.
    Roof,
    /// The special and ground codes of the ground polygon under the player's feet, and the
    /// sliding angle that picks which way the slide faces, which together select a sliding face.
    SlideSurface,
    /// A pushable actor, found through the stage's push and pull callback against the polygon
    /// the player is standing on.
    Pushable,
}

impl MoveGap {
    /// The thing the port is missing, for a reader who has the reference open beside this.
    pub fn reason(self) -> &'static str {
        match self {
            Self::Ready => "nothing",
            Self::TurnRate => "the turn-rate word",
            Self::RollRate => "the roll-speed and roll-length words",
            Self::TakeoffHeight => "the height the player left the ground at",
            Self::ShapeAngle => "the model and body facing split",
            Self::CrouchButton => "the crouch and guard button",
            Self::LowClearance => "a low-clearance probe",
            Self::WallType => "the front wall classification",
            Self::Ledge => "a ledge volume",
            Self::Roof => "a ceiling probe",
            Self::SlideSurface => "the sliding face codes and angle",
            Self::Pushable => "a pushable actor",
        }
    }
}

/// One row per process this module owns: where the reference asks for it, where it leaves through,
/// and the gap that keeps the port from asking. The ladder family is the stage's own module and
/// has no row, and neither do the sword, hammer and boat families.
#[derive(Debug, Clone, Copy)]
pub struct MoveSite {
    /// The process the row describes.
    pub proc: Proc,
    /// Where the reference asks for it, by file and line.
    pub entry: &'static str,
    /// Where the reference leaves it, by file and line.
    pub exit: &'static str,
    /// What the port cannot answer for it.
    pub gap: MoveGap,
}

/// Every process with no module of its own, ordered by the sequence the reference tries them in:
/// the crouch and crawl pair first, because every other grounded state reaches its button test
/// through the crouch state, and the wall and vine selection last, because the reference runs it
/// after the action dispatch has had its turn.
pub const SITES: &[MoveSite] = &[
    MoveSite { proc: Proc::Crouch, entry: "main.cpp:4285", exit: "main.cpp:6526", gap: MoveGap::CrouchButton },
    MoveSite { proc: Proc::CrouchDefense, entry: "main.cpp:4292", exit: "main.cpp:6409", gap: MoveGap::CrouchButton },
    MoveSite { proc: Proc::CrouchDefenseSlip, entry: "main.cpp:4292", exit: "main.cpp:6490", gap: MoveGap::CrouchButton },
    MoveSite { proc: Proc::CrawlStart, entry: "main.cpp:6530", exit: "crawl.inc:295", gap: MoveGap::LowClearance },
    MoveSite { proc: Proc::CrawlMove, entry: "crawl.inc:297", exit: "crawl.inc:399", gap: MoveGap::LowClearance },
    MoveSite { proc: Proc::CrawlAutoMove, entry: "crawl.inc:473", exit: "crawl.inc:656", gap: MoveGap::LowClearance },
    MoveSite { proc: Proc::CrawlEnd, entry: "crawl.inc:295", exit: "crawl.inc:710", gap: MoveGap::LowClearance },
    MoveSite { proc: Proc::PushPullWait, entry: "main.cpp:4138", exit: "pushpull.inc:104", gap: MoveGap::Pushable },
    MoveSite { proc: Proc::PushMove, entry: "pushpull.inc:108", exit: "pushpull.inc:141", gap: MoveGap::Pushable },
    MoveSite { proc: Proc::PullMove, entry: "pushpull.inc:110", exit: "pushpull.inc:184", gap: MoveGap::Pushable },
    MoveSite { proc: Proc::SlideFront, entry: "main.cpp:4900", exit: "main.cpp:6712", gap: MoveGap::SlideSurface },
    MoveSite { proc: Proc::SlideBack, entry: "main.cpp:4902", exit: "main.cpp:6746", gap: MoveGap::SlideSurface },
    MoveSite { proc: Proc::SlideFrontLand, entry: "main.cpp:6712", exit: "main.cpp:6762", gap: MoveGap::SlideSurface },
    MoveSite { proc: Proc::SlideBackLand, entry: "main.cpp:6746", exit: "main.cpp:6787", gap: MoveGap::SlideSurface },
    MoveSite { proc: Proc::WaitTurn, entry: "main.cpp:4482", exit: "main.cpp:6581", gap: MoveGap::TurnRate },
    MoveSite { proc: Proc::MoveTurn, entry: "main.cpp:4492", exit: "main.cpp:6621", gap: MoveGap::ShapeAngle },
    MoveSite { proc: Proc::SideRoll, entry: "main.cpp:4952", exit: "main.cpp:6959", gap: MoveGap::RollRate },
    MoveSite { proc: Proc::HangStart, entry: "main.cpp:4780", exit: "hang.inc:252", gap: MoveGap::Ledge },
    MoveSite { proc: Proc::HangFallStart, entry: "main.cpp:7470", exit: "hang.inc:364", gap: MoveGap::Ledge },
    MoveSite { proc: Proc::HangUp, entry: "hang.inc:372", exit: "hang.inc:406", gap: MoveGap::Ledge },
    MoveSite { proc: Proc::HangWait, entry: "hang.inc:261", exit: "hang.inc:440", gap: MoveGap::Ledge },
    MoveSite { proc: Proc::HangMove, entry: "hang.inc:411", exit: "hang.inc:478", gap: MoveGap::Ledge },
    MoveSite { proc: Proc::HangClimb, entry: "hang.inc:257", exit: "hang.inc:574", gap: MoveGap::Roof },
    MoveSite { proc: Proc::HangWallCatch, entry: "main.cpp:4152", exit: "hang.inc:620", gap: MoveGap::Ledge },
    MoveSite { proc: Proc::ClimbUpStart, entry: "main.cpp:4835", exit: "climb.inc:266", gap: MoveGap::WallType },
    MoveSite { proc: Proc::ClimbDownStart, entry: "main.cpp:4864", exit: "climb.inc:311", gap: MoveGap::WallType },
    MoveSite { proc: Proc::ClimbMoveUpDown, entry: "climb.inc:90", exit: "climb.inc:372", gap: MoveGap::WallType },
    MoveSite { proc: Proc::ClimbMoveSide, entry: "climb.inc:92", exit: "climb.inc:439", gap: MoveGap::WallType },
];

/// What the owned processes are doing and what is stopping them. The reference keeps this in its
/// per-process scratch words; the port keeps the one part of it a reader needs, which is the gap.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct MoveState {
    /// The process whose entry guard asked this tick, if any did.
    pub asked: Option<Proc>,
    /// What the port could not answer for `asked`.
    pub gap: MoveGap,
}

impl Default for MoveState {
    fn default() -> Self {
        Self {
            asked: None,
            gap: MoveGap::Ready,
        }
    }
}

impl MoveState {
    /// The row naming `proc`, if this module owns it.
    pub fn site(proc: Proc) -> Option<&'static MoveSite> {
        SITES.iter().find(|site| site.proc == proc)
    }

    /// Whether this module owns `proc`.
    pub fn owns(proc: Proc) -> bool {
        SITES.iter().any(|site| site.proc == proc)
    }

    /// The gap standing between `proc` and the port, or [`MoveGap::Ready`] for a process no row
    /// names.
    pub fn gap_of(proc: Proc) -> MoveGap {
        Self::site(proc).map_or(MoveGap::Ready, |site| site.gap)
    }
}

/// Everything a guard may read, taken once so every guard sees the same tick. The five queries at
/// the end answer `None` because the stage's collision has no such thing to answer with: it holds
/// a ground height, a ground normal and an exit byte per polygon, and nothing else. They are the
/// only place a real implementation plugs in.
struct Probe<'a> {
    player: &'a Player,
    camera: &'a CameraYaw,
    collision: Option<&'a Collision>,
    swim: &'a Swim,
    jump: &'a Jump,
    roll: &'a Roll,
    hop: &'a Hop,
    combat: &'a Combat,
    lock: &'a Lock,
    intent: &'a MoveIntent,
    dead: bool,
    riding: bool,
}

impl Probe<'_> {
    fn stick(&self) -> f32 {
        self.intent.axis.length().min(1.0)
    }

    /// `m34E8`, the heading the stick is asking for.
    fn wanted(&self) -> i16 {
        target_yaw(self.intent, self.camera)
    }

    /// `cLib_distanceAngleS(m34E8, current.angle.y)`, the stick against the body's facing.
    fn body_error(&self) -> u16 {
        facing_error(self.intent, self.camera, self.player.yaw)
    }

    /// `getDirectionFromShapeAngle`, the stick read against the model's own facing.
    fn shape_direction(&self) -> Direction {
        direction(self.wanted(), to_s16(self.player.face))
    }

    /// `getDirectionFromCurrentAngle`, the stick read against the body's own facing.
    fn body_direction(&self) -> Direction {
        direction(self.wanted(), to_s16(self.player.yaw))
    }

    /// Whether a module above already owns the player's motion, which is what the reference reads
    /// off its own process word when it asks for a new one.
    fn busy(&self) -> bool {
        self.swim.active
            || self.jump.active
            || self.roll.active
            || self.hop.active
            || self.combat.active
            || self.lock.target.is_some()
    }

    /// `spActionButton()`, the crouch and guard button: a pad face the port's four intents do not
    /// carry, so `None` is the honest answer rather than a stand-in for one of them.
    fn crouch_button(&self) -> Option<bool> {
        None
    }

    /// `mFrontWallType`, chosen from a wall trace and the wall code of the triangle it lands on.
    fn front_wall(&self) -> Option<u8> {
        None
    }

    /// `m3724`, the ledge point where a wall meets a floor at the player's own height.
    fn ledge(&self) -> Option<Vec3> {
        None
    }

    /// The ceiling above the player, which the hang states open with.
    fn roof(&self) -> Option<f32> {
        None
    }

    /// The sliding face the ground polygon presents, true when the slide runs downhill. It comes
    /// from the polygon's special and ground codes and from the angle between its normal and the
    /// model's own facing.
    fn slide_face(&self) -> Option<bool> {
        None
    }

    /// A pushable actor standing in the player's grab reach.
    fn pushable(&self) -> Option<Entity> {
        None
    }
}

fn cut_proc(kind: CutKind) -> Proc {
    match kind {
        CutKind::A => Proc::CutA,
        CutKind::F => Proc::CutF,
        CutKind::R => Proc::CutR,
        CutKind::L => Proc::CutL,
        CutKind::EA => Proc::CutEa,
        CutKind::EB => Proc::CutEb,
    }
}

fn jump_proc(kind: JumpKind) -> Proc {
    match kind {
        JumpKind::Back => Proc::BackJump,
        JumpKind::Left | JumpKind::Right => Proc::SideStep,
    }
}

fn named(probe: &Probe) -> Proc {
    if probe.swim.active {
        return if probe.swim.surfaced {
            Proc::SwimWait
        } else {
            Proc::SwimMove
        };
    }
    if probe.combat.active {
        return cut_proc(probe.combat.kind);
    }
    if probe.hop.active {
        return Proc::AutoJump;
    }
    if probe.roll.active {
        return Proc::FrontRoll;
    }
    if probe.jump.active {
        return jump_proc(probe.jump.kind);
    }
    if probe.lock.target.is_some() {
        return if probe.player.speed.abs() > REST_SPEED {
            Proc::AtnActorMove
        } else {
            Proc::AtnActorWait
        };
    }
    if probe.player.speed.abs() > REST_SPEED {
        return Proc::Move;
    }
    Proc::Wait
}

fn want_crouch(probe: &Probe) -> Option<(Proc, MoveGap)> {
    if probe.dead || probe.riding || probe.busy() {
        return None;
    }
    match probe.crouch_button() {
        Some(false) => None,
        Some(true) | None => Some((Proc::Crouch, MoveGap::CrouchButton)),
    }
}

fn want_push_pull(probe: &Probe) -> Option<(Proc, MoveGap)> {
    if probe.dead || probe.riding || probe.busy() {
        return None;
    }
    match probe.pushable() {
        Some(_) => Some((Proc::PushPullWait, MoveGap::Ready)),
        None => Some((Proc::PushPullWait, MoveGap::Pushable)),
    }
}

fn want_crawl(probe: &Probe) -> Option<(Proc, MoveGap)> {
    if probe.dead || probe.riding {
        return None;
    }
    if !grounded(probe.collision, probe.player.pos) || probe.stick() <= STICK_GATE {
        return None;
    }
    let Some(roof) = probe.roof() else {
        return Some((Proc::CrawlStart, MoveGap::LowClearance));
    };
    if roof - probe.player.pos.y > CRAWL_CLEARANCE {
        return None;
    }
    Some((Proc::CrawlStart, MoveGap::Ready))
}

fn want_slide(probe: &Probe) -> Option<(Proc, MoveGap)> {
    if probe.dead || probe.riding || probe.busy() {
        return None;
    }
    if !grounded(probe.collision, probe.player.pos) {
        return None;
    }
    Some(match probe.slide_face() {
        Some(true) => (Proc::SlideFront, MoveGap::Ready),
        Some(false) => (Proc::SlideBack, MoveGap::Ready),
        None => (Proc::SlideFront, MoveGap::SlideSurface),
    })
}

fn want_wait_turn(probe: &Probe) -> Option<(Proc, MoveGap)> {
    if probe.dead || probe.riding || probe.busy() {
        return None;
    }
    if probe.player.speed.abs() > REST_SPEED {
        return None;
    }
    if probe.body_error() <= REVERSAL_LIMIT || probe.stick() <= STICK_GATE {
        return None;
    }
    Some((Proc::WaitTurn, MoveGap::TurnRate))
}

fn want_move_turn(probe: &Probe) -> Option<(Proc, MoveGap)> {
    if probe.dead || probe.riding || probe.busy() {
        return None;
    }
    if probe.player.speed.abs() <= REST_SPEED {
        return None;
    }
    if probe.body_error() <= REVERSAL_LIMIT || probe.stick() <= STICK_GATE {
        return None;
    }
    Some((Proc::MoveTurn, MoveGap::ShapeAngle))
}

fn want_side_roll(probe: &Probe) -> Option<(Proc, MoveGap)> {
    if probe.dead || probe.riding {
        return None;
    }
    if probe.stick() <= LAND_STICK {
        return None;
    }
    if matches!(probe.body_direction(), Direction::Back)
        || matches!(probe.shape_direction(), Direction::Back)
    {
        return None;
    }
    Some((Proc::SideRoll, MoveGap::TakeoffHeight))
}

fn want_climb(probe: &Probe) -> Option<(Proc, MoveGap)> {
    if probe.dead || probe.riding || probe.busy() {
        return None;
    }
    if !grounded(probe.collision, probe.player.pos) {
        return None;
    }
    match probe.front_wall() {
        Some(3) => Some((Proc::ClimbUpStart, MoveGap::Ready)),
        Some(_) => None,
        None => Some((Proc::ClimbUpStart, MoveGap::WallType)),
    }
}

fn want_hang(probe: &Probe) -> Option<(Proc, MoveGap)> {
    if probe.dead || probe.riding || probe.busy() {
        return None;
    }
    match probe.ledge() {
        Some(_) => Some((Proc::HangStart, MoveGap::Ready)),
        None => Some((Proc::HangStart, MoveGap::Ledge)),
    }
}

const ASKS: &[for<'a> fn(&Probe<'a>) -> Option<(Proc, MoveGap)>] = &[
    want_crouch,
    want_push_pull,
    want_crawl,
    want_slide,
    want_wait_turn,
    want_move_turn,
    want_side_roll,
    want_climb,
    want_hang,
];

fn resolve(probe: &Probe, current: Proc) -> Option<(Proc, MoveGap)> {
    if MoveState::owns(current) {
        return None;
    }
    ASKS.iter().find_map(|ask| ask(probe))
}

/// Name the process the modules above already are, then ask the reference's own guards for the
/// ones no module owns. The port carries no animation clock and no frame controls, so every state
/// in [`SITES`] reports its gap and stays out; the process word still names what the modules above
/// are doing, which is the half that bites today.
pub fn control(
    intent: Res<MoveIntent>,
    camera: Res<CameraYaw>,
    collision: Option<Res<Collision>>,
    health: Res<Health>,
    swim: Res<Swim>,
    jump: Res<Jump>,
    roll: Res<Roll>,
    hop: Res<Hop>,
    combat: Res<Combat>,
    lock: Res<Lock>,
    ships: Query<&Ship>,
    mut proc: ResMut<LinkProc>,
    mut state: ResMut<MoveState>,
    players: Query<&Player>,
) {
    let riding = ships.iter().any(|ship| ship.riding);
    let Some(player) = players.iter().next() else {
        return;
    };
    let probe = Probe {
        player: &player,
        camera: &camera,
        collision: collision.as_deref(),
        swim: &swim,
        jump: &jump,
        roll: &roll,
        hop: &hop,
        combat: &combat,
        lock: &lock,
        intent: &intent,
        dead: health.units == 0,
        riding,
    };
    proc.advance();
    let current = proc.current;
    let ask = resolve(&probe, current);
    state.asked = ask.map(|(want, _)| want);
    match ask {
        Some((want, MoveGap::Ready)) => {
            state.gap = MoveGap::Ready;
            proc.enter(want);
        }
        Some((_, gap)) => state.gap = gap,
        None if MoveState::owns(current) => state.gap = MoveState::gap_of(current),
        None => {
            state.gap = MoveGap::Ready;
            proc.set_proc(named(&probe));
        }
    }
}