# clearsleeper

The Wind Waker recreation running on repame via repose.

Sim systems land module-by-module in `repame-sim` schedules at a fixed
step; world rendering goes through `repame-view3d` (glTF meshes, skinned
animation, cascaded shadows); UI stays repose views.

Layout: `src/` grows `sim/` (components, resources, systems), `view.rs`
(snapshot producers), `hud.rs`, `audio.rs`, `save.rs`, `input.rs`,
`assets.rs` as strangler slices land. Nothing here depends on bevy, ever.

## Build & run

Desktop (windowed, `CS_WINDOW=1280x720` overrides the size):

    cargo run

Web (trunk dev server, http://127.0.0.1:8080):

    trunk serve

Android (APK signed with the debug keystore):

    cargo rapk check
    cargo rapk build --lib     # target/debug/apk/cs.apk

Release artifacts: `cargo build --release`, `trunk build --release`,
`cargo rapk build --release --lib`. Release signing reads
`CARGO_RAPK_RELEASE_KEYSTORE` + `CARGO_RAPK_RELEASE_KEYSTORE_PASSWORD`
(or `package.metadata.android.signing.release`); keystores stay
untracked.

Disclaimer: This does not provide any copyrighted assets. You need to bring your own copy of the game.

`CS_ASSETS` points the archive loader at an extracted game disc root
(`files/`, `sys/`). the data stays outside this repository. Headless sim
tests never touch it.

Mainly uses dusklight as a reference, and the code of tww for the decomp.
