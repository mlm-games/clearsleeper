use clearsleeper::App;
use clearsleeper::sim::flags::Switches;
use clearsleeper::sim::warship;

const GONE_WITH_AWAY_SWITCH: u32 = 0x0400_0000;

#[test]
fn golden_switch_keeps_a_warship_at_anchor() {
    let mut app = App::new();
    let away = warship::is_switched(
        GONE_WITH_AWAY_SWITCH,
        -1,
        app.sim.world.resource::<Switches>(),
    );
    assert!(away, "a clear away-switch lets it sail");
    app.sim.world.resource_mut::<Switches>().set(4);
    assert!(
        !warship::is_switched(
            GONE_WITH_AWAY_SWITCH,
            -1,
            app.sim.world.resource::<Switches>()
        ),
        "and setting that switch keeps it at anchor"
    );
}
