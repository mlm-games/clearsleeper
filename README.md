# clearsleeper

The Wind Waker recreation running on repame via repose.

Sim systems land module-by-module in `repame-sim` schedules at a fixed
step; world rendering goes through `repame-view3d` (glTF meshes, skinned
animation, cascaded shadows); UI stays repose views.

Layout: `src/` grows `sim/` (components, resources, systems),
`render/` (snapshot producers), `ui/`, `audio.rs`, `save.rs`, `input.rs`
as strangler slices land. Nothing here depends on bevy, ever.
