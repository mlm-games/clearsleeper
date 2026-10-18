# clearsleeper

The Wind Waker recreation running on repame via repose.

Sim systems land module-by-module in `repame-sim` schedules at a fixed
step; world rendering goes through `repame-view3d` (glTF meshes, skinned
animation, cascaded shadows); UI stays repose views.

Layout: `src/` grows `sim/` (components, resources, systems), `view.rs`
(snapshot producers), `hud.rs`, `audio.rs`, `save.rs`, `input.rs`,
`assets.rs` as strangler slices land. Nothing here depends on bevy, ever.

## Build & run

Desktop (windowed, `CS_WINDOW=1280x720` overrides the size). A disc image is named
directly (`--disc <path>`, or just the path); with no argument, `disc.iso`/`disc.rvz`
next to the executable and under the app's data dir are tried, and only then does
`CS_ASSETS` decide:

    cargo run -- --disc /path/to/GZLE01.rvz
    cargo run                    # default disc image, else CS_ASSETS

Web (trunk dev server, http://127.0.0.1:8080):

    trunk serve

The web build has no extracted tree to read: the page's drop zone takes a GameCube disc
image (an ISO/GCM dump, or an RVZ the `nod` crate unpacks) and installs it as the asset
source, so the same `files/res/...` paths resolve. Anything else fails with the reason on
the page. The image is read into the tab's wasm memory whole, which the threaded build
caps at 4 GB, so an image over 2 GB is refused on the page before it is read.

Android (APK signed with the debug keystore). The app opens the system file picker at
boot; a `disc.iso`/`disc.rvz` pushed into the app's data dir (the external files dir
takes an `adb push`) skips the picker:

    cargo rapk check
    cargo rapk build --lib     # target/debug/apk/cs.apk

Release artifacts: `cargo build --release`, `trunk build --release`,
`cargo rapk build --release --lib`. Release signing reads
`CARGO_RAPK_RELEASE_KEYSTORE` + `CARGO_RAPK_RELEASE_KEYSTORE_PASSWORD`
(or `package.metadata.android.signing.release`); keystores stay
untracked.

Disclaimer: This does not provide any copyrighted assets. You need to bring your own copy of the game.

The disc image is the primary source on every platform: the file is read with the same
reader the web drop and the Android picker feed, and an image that is named or found but
unreadable is a typed failure (stderr on desktop), never a silent empty world.
`CS_ASSETS` stays the development fallback - it points the archive loader at an extracted
game disc root (`files/`, `sys/`) when no image is installed. The data stays outside this
repository. Headless sim tests never touch it.

Mainly uses dusklight as a reference, and the code of tww for the decomp.
