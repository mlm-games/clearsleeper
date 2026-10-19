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

## Retail comparison

The port can be run headless against a retail oracle: a fresh session, a scripted pad
tape, and a per-tick digest of the world.

    cargo run --example trace_boot -- 600 /tmp/port.trace
    cargo run --example trace-diff -- /tmp/port.trace /tmp/retail.trace

`trace_boot` opens no window, no GPU device and no audio device. `CS_DISC` names the disc
image to install and `CS_ASSETS` the extracted tree to fall back on, `CS_INPUT` the tape to
replay, and `CS_TRACE` (or the second argument) where the digest goes; the tick count is
the first argument, `CS_TRACE_TICKS`, or 600. The digest's grammar and the field each key
comes from in the retail game are in `src/trace.rs`.

The tape is one line per retrace, the pad state the guest is given:

    retrace=<N> button=0x<hex> stick=<x>,<y> c=<x>,<y> l=<L> r=<R>

`button` is the GameCube pad word (`A` 0x0100, `B` 0x0200, `X` 0x0400, `Y` 0x0800, `Z`
0x0010, `L` 0x0040, `R` 0x0020, `START` 0x1000, the D-pad 0x0001-0x0008), `stick` and `c`
are the two sticks as the centred bytes the pad sent, and `l` and `r` are the analog
triggers. A retrace with no line holds the state of the line before it.

Disclaimer: This does not provide any copyrighted assets. You need to bring your own copy of the game.

The disc image is the primary source on every platform: the file is read with the same
reader the web drop and the Android picker feed, and an image that is named or found but
unreadable is a typed failure (stderr on desktop), never a silent empty world.
`CS_ASSETS` stays the development fallback - it points the archive loader at an extracted
game disc root (`files/`, `sys/`) when no image is installed. The data stays outside this
repository. Headless sim tests never touch it.

Mainly uses dusklight as a reference, and the code of tww for the decomp.
