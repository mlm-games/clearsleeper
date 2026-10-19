use clearsleeper::App;
use clearsleeper::input_tape::{
    BUTTON_A, BUTTON_B, BUTTON_L, BUTTON_R, BUTTON_START, BUTTON_X, BUTTON_Y, BUTTON_Z, InputTape,
    PadState, TapeError,
};
use clearsleeper::sim::intent::{CStick, MoveIntent, PadButtons};

fn line(retrace: u64, button: u16, stick: (i8, i8), c: (i8, i8)) -> String {
    format!(
        "retrace={retrace} button=0x{button:04x} stick={},{} c={},{} l=0 r=0",
        stick.0, stick.1, c.0, c.1
    )
}

fn tape(lines: &[String]) -> InputTape {
    InputTape::parse(&lines.join("\n")).expect("tape parses")
}

#[test]
fn a_line_applies_at_its_retrace() {
    let tape = tape(&[
        line(1, BUTTON_A, (0, 0), (0, 0)),
        line(5, BUTTON_B, (10, -20), (0, 0)),
    ]);
    assert_eq!(tape.len(), 2);
    assert_eq!(tape.state_at(1).buttons, BUTTON_A);
    assert_eq!(tape.state_at(5).buttons, BUTTON_B);
    assert_eq!(tape.state_at(5).stick_x, 10);
    assert_eq!(tape.state_at(5).stick_y, -20);
    assert_eq!(tape.state_at(99).buttons, BUTTON_B);
}

#[test]
fn a_gap_holds_the_previous_state() {
    let tape = tape(&[
        line(1, BUTTON_A, (0, 0), (0, 0)),
        line(5, BUTTON_B, (10, -20), (0, 0)),
    ]);
    for retrace in 1..5 {
        let state = tape.state_at(retrace);
        assert_eq!(state.buttons, BUTTON_A, "retrace {retrace}");
        assert_eq!(state.stick_x, 0, "retrace {retrace}");
    }
    let before = tape.state_at(0);
    assert_eq!(
        before.buttons, 0,
        "a tick before the first line runs released"
    );
    assert_eq!(before.stick_x, 0);
}

#[test]
fn a_malformed_line_is_a_typed_error() {
    assert!(matches!(
        InputTape::parse("retrace=1 button=0x0100 stick=0,0 c=0,0 l=0 r=0 nonsense"),
        Err(TapeError::BadPair { line: 1, .. })
    ));
    assert!(matches!(
        InputTape::parse("retrace=1 button=0x0100 stick=0,0 c=0,0 l=0"),
        Err(TapeError::Missing { line: 1, key: "r" })
    ));
    assert!(matches!(
        InputTape::parse("retrace=one button=0x0100 stick=0,0 c=0,0 l=0 r=0"),
        Err(TapeError::BadValue { line: 1, .. })
    ));
    assert!(matches!(
        InputTape::parse("retrace=1 button=0xzzzz stick=0,0 c=0,0 l=0 r=0"),
        Err(TapeError::BadValue { line: 1, .. })
    ));
    assert!(matches!(
        InputTape::parse("retrace=1 button=0x0100 stick=0 c=0,0 l=0 r=0"),
        Err(TapeError::BadValue { line: 1, .. })
    ));
    assert!(matches!(
        InputTape::parse("retrace=1 button=0x0100 stick=0,0 c=0,0 l=300 r=0"),
        Err(TapeError::BadValue { line: 1, .. })
    ));
}

#[test]
fn a_comment_and_a_blank_line_are_skipped() {
    let text = format!("# the pad trace\n\n{}\n", line(2, BUTTON_A, (0, 0), (0, 0)));
    let tape = InputTape::parse(&text).expect("tape parses");
    assert_eq!(tape.len(), 1);
    assert_eq!(tape.state_at(2).buttons, BUTTON_A);
}

#[test]
fn each_button_bit_yields_its_intent() {
    type Read = fn(&clearsleeper::input::ButtonBits) -> bool;
    let bits: [(u16, Read); 8] = [
        (BUTTON_A, |held| held.a),
        (BUTTON_B, |held| held.b),
        (BUTTON_X, |held| held.x),
        (BUTTON_Y, |held| held.y),
        (BUTTON_Z, |held| held.z),
        (BUTTON_L, |held| held.l),
        (BUTTON_R, |held| held.r),
        (BUTTON_START, |held| held.start),
    ];
    for (bit, read) in bits {
        let state = PadState {
            buttons: bit,
            ..PadState::default()
        };
        let held = state.hold();
        assert!(read(&held), "bit 0x{bit:04x} must reach its own intent");
        let others: Vec<bool> = bits
            .iter()
            .filter(|(other, _)| *other != bit)
            .map(|(_, other)| other(&held))
            .collect();
        assert!(
            others.iter().all(|set| !set),
            "bit 0x{bit:04x} must not raise another intent"
        );
    }
}

#[test]
fn the_stick_becomes_the_retail_position() {
    let centred = PadState::default();
    assert_eq!(centred.move_axis(), glam::Vec2::ZERO);
    assert_eq!(centred.c_axis(), glam::Vec2::ZERO);

    let dead = PadState {
        stick_y: 15,
        ..PadState::default()
    };
    assert_eq!(dead.move_axis(), glam::Vec2::ZERO, "the pad's dead zone");

    let forward = PadState {
        stick_y: 69,
        ..PadState::default()
    };
    let axis = forward.move_axis();
    assert!((axis.y - 1.0).abs() < 1e-6, "axis = {axis:?}");
    assert!(axis.x.abs() < 1e-6);

    let c_forward = PadState {
        c_stick_y: 57,
        ..PadState::default()
    };
    let c_axis = c_forward.c_axis();
    assert!((c_axis.y - 1.0).abs() < 1e-6, "c axis = {c_axis:?}");

    let full = PadState {
        stick_x: 127,
        ..PadState::default()
    };
    let axis = full.move_axis();
    assert!((axis.x - 1.0).abs() < 1e-6, "axis = {axis:?}");
    assert!(axis.y.abs() < 1e-6);
}

#[test]
fn the_trigger_reads_as_the_retail_fraction() {
    let state = PadState {
        trigger_left: 70,
        trigger_right: 255,
        ..PadState::default()
    };
    assert!((state.trigger_left() - 0.5).abs() < 1e-6);
    assert!((state.trigger_right() - 1.0).abs() < 1e-6);
}

#[test]
fn a_staged_pad_state_reaches_the_intent_resources() {
    let mut app = App::new();
    let pressed = PadState {
        buttons: BUTTON_A,
        stick_y: 69,
        c_stick_y: 57,
        ..PadState::default()
    };
    app.stage_pad_state(&pressed);
    let axis = app.sim.world.resource::<MoveIntent>().axis;
    assert!((axis.y - 1.0).abs() < 1e-6, "axis = {axis:?}");
    let pad = *app.sim.world.resource::<PadButtons>();
    assert!(pad.hold.a, "the hold word carries the button");
    assert!(pad.trig.a, "the first press is an edge");
    let c = *app.sim.world.resource::<CStick>();
    assert!((c.axis.y - 1.0).abs() < 1e-6, "c axis = {:?}", c.axis);

    app.stage_pad_state(&pressed);
    let pad = *app.sim.world.resource::<PadButtons>();
    assert!(pad.hold.a, "the button is still down");
    assert!(!pad.trig.a, "a held button is not a fresh press");

    app.stage_pad_state(&PadState::default());
    let pad = *app.sim.world.resource::<PadButtons>();
    assert!(!pad.hold.a, "the button is released");
}
