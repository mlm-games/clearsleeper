use glam::{Mat4, Vec2, Vec3};
use repame_view3d::{
    BatchDesc, CullMode, Frame3d, MeshGroup, OrbitCamera, Rgb, SceneFilter, SceneUpload,
    SkinnedMesh,
};

use crate::j3d::bdl::{LAYER_SIZE, RoomGroup, RoomMesh, SKIN_INFLUENCES};

pub const LINK_ARCHIVE: &str = "Link";

pub const SKY: [f32; 4] = [0.44, 0.70, 0.94, 1.0];
const GROUND: Rgb = [0.24, 0.42, 0.22];
const WALL: Rgb = [0.48, 0.20, 0.20];
const WALL_HEIGHT: f32 = 30.0;
const WALL_THICKNESS: f32 = 8.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActorKind {
    Enemy,
    Pickup,
    Talker,
    Ship,
    Bomb,
    Arrow,
    Boomerang,
    Spear,
    Balloon,
    Warship,
    Cannon,
    Fire,
    Gull,
    Scenery,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ActorView {
    pub pos: Vec3,
    pub kind: ActorKind,
    pub item: u8,
    pub height: f32,
    pub half: f32,
    /// The BCK an actor's own frame cursor is on. One field for every actor so the view can
    /// skin any of them the same way it skins the player ([`PlayerAnm`]).
    pub anm: Option<ActorAnm>,
    /// Whether this actor is drawn from a real model instead of a box.
    pub modeled: bool,
}

/// The animation an actor's sim cursor is on: the object archive and resource id its own anm
/// table named, then the frame and wrap the cursor reached (`dNpc_setAnmIDRes`, `d_npc.cpp:546-563`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ActorAnm {
    pub archive: &'static str,
    pub bck: i32,
    pub frame: f32,
    pub looping: bool,
}

/// The bck a demo step put the player on, and the frame the demo has reached in it
/// ([`crate::sim::link_demo::PlayerAnm`]).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerAnm {
    pub stem: &'static str,
    pub frame: f32,
    /// Walk and dash loop; a cutscene morf and a sword cut hold their last frame.
    pub looping: bool,
}

/// The equipped sword and shield item ids, `dComIfGs_getSelectEquip(0/1)`
/// (`d_save.h:46`). `0xFF` means none (`inventory::NO_ITEM`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PlayerEquipment {
    pub sword: u8,
    pub shield: u8,
}

/// A parts model attached at one of the body's joints, the port's `setBaseTRMtx` target
/// (`tww/src/d/actor/d_a_player_main.cpp:1150-1159`,
/// `wwhd/wwhd_src/d/actor/d_a_player_main_04.cpp:3174-3181`).
#[derive(Clone, Copy)]
pub struct AttachedPart<'a> {
    pub mesh: &'a RoomMesh,
    pub body_joint: usize,
    /// A part-local BCK pose and the frame to sample it at. `None` uses the model's bind pose.
    pub anim: Option<(&'a crate::j3d::bck::Morph, f32)>,
    /// Joint-local offset applied before the body joint matrix: the back-mount transform for a
    /// shield that is not being wielded (`tww/src/d/actor/d_a_player_main.cpp:1312-1319`,
    /// `wwhd/wwhd_src/d/actor/d_a_player_main_04.cpp:3231-3234`).
    pub local: Mat4,
    /// Yaw, in model space about the part's own position, added while the mirror shield is stowed
    /// and Link walks or dashes (`tww/src/d/actor/d_a_player_main.cpp:1320-1337`,
    /// `wwhd/wwhd_src/d/actor/d_a_player_main_04.cpp:3235-3255`).
    pub sway: f32,
    /// A material slot and vertex-colour multiplier from a BRK, applied to the matching shape
    /// (`tww/src/d/actor/d_a_player_main.cpp:1222-1254`,
    /// `wwhd/wwhd_src/d/actor/d_a_player_main_04.cpp:3141-3169`).
    pub color: Option<(u16, [f32; 3])>,
    /// A material slot and texture matrix from a BTK, applied to that material's UVs
    /// (`tww/src/d/actor/d_a_player_main.cpp:1261-1268,1283-1294`).
    pub uv: Option<(u16, crate::j3d::btk::UvMatrix)>,
}

impl<'a> AttachedPart<'a> {
    pub fn hand(mesh: &'a RoomMesh, body_joint: usize) -> Self {
        Self {
            mesh,
            body_joint,
            anim: None,
            local: Mat4::IDENTITY,
            sway: 0.0,
            color: None,
            uv: None,
        }
    }

    pub fn back(mesh: &'a RoomMesh, body_joint: usize, mirror: bool) -> Self {
        Self {
            mesh,
            body_joint,
            anim: None,
            local: Mat4::from_translation(Vec3::new(15.5, 4.75, if mirror { -0.2 } else { 0.0 }))
                * Mat4::from_rotation_z(-std::f32::consts::FRAC_PI_2),
            sway: 0.0,
            color: None,
            uv: None,
        }
    }

    /// The sword's scabbard transform on `podA`, used when another item is selected
    /// (`tww/src/d/actor/d_a_player_main.cpp:1272-1277`,
    /// `wwhd/wwhd_src/d/actor/d_a_player_main_04.cpp:3180-3186`).
    pub fn sword_back(mesh: &'a RoomMesh, body_joint: usize) -> Self {
        Self {
            mesh,
            body_joint,
            anim: None,
            local: Mat4::from_translation(Vec3::new(-11.25, 4.5, 0.45))
                * Mat4::from_rotation_z(-0x1C71 as f32 * std::f32::consts::TAU / 65536.0),
            sway: 0.0,
            color: None,
            uv: None,
        }
    }
}

/// One non-Link actor drawn from its own object archive instead of a box: positioned with the
/// actor's own transform, posed by the BCK its sim cursor published (`dNpc_setAnmIDRes`,
/// `tww/src/d/d_npc.cpp:546-563`, HD `d_npc.cpp:58-69`).
#[derive(Clone, Copy)]
pub struct NpcDraw<'a> {
    pub mesh: &'a RoomMesh,
    pub anim: Option<(&'a crate::j3d::bck::Morph, f32)>,
    pub pos: Vec3,
    pub yaw: f32,
    /// Head Y and backbone X deltas post-multiplied onto the already-posed world matrices
    /// (`tww/src/d/actor/d_a_npc_ba1.cpp:97-113`, HD `d_a_npc_ba1.cpp:205-235`).
    pub joint_control: Option<(u16, f32, u16, f32)>,
    /// The cape model hung off the `footL` joint (`d_a_npc_ba1.cpp:2034-2040,294-296`).
    pub cloth: Option<(u16, &'a RoomMesh)>,
}

/// A blade-tip ribbon sampled from the cut's BCK at past frames, drawn with `blur.bti` the way
/// `copySwBlur` feeds `drawSwBlur` (`tww/src/d/actor/d_a_player_particle.inc:193-309`).
pub struct SwordBlurDraw<'a> {
    pub points: Vec<Vec3>,
    pub rgba: &'a [u8],
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SceneSnapshot {
    pub player_pos: Vec3,
    pub player_yaw: f32,
    pub player_face: f32,
    /// Linear ground speed, for the stowed mirror shield's walk/dash sway
    /// (`tww/src/d/actor/d_a_player_main.cpp:1331`).
    pub player_speed: f32,
    /// Whether a non-grab item animation is playing, i.e. another item is selected, which sends
    /// the sword to its back scabbard (`tww/src/d/actor/d_a_player_main.cpp:1269-1276`).
    pub item_in_use: bool,
    pub player_anm: Option<PlayerAnm>,
    /// `checkPlayerNoDraw()` (`d_a_player_main.h:1980`), the gate the player's own draw takes
    /// before it submits its model: something raised `daPyFlg0_NO_DRAW`, or the camera carries
    /// `dCamAttnStts_SUBJECT_e` ([`crate::sim::camera::Camera::publish_attention`]).
    pub player_no_draw: bool,
    pub equipment: PlayerEquipment,
    /// Whether the shield sits on the back (`podA`) rather than the hand: retail places it in
    /// hand while wielding the sword or guarding (tww `d_a_player_main.cpp:1301-1319`).
    pub shield_on_back: bool,
    pub bounds_min: Vec2,
    pub bounds_max: Vec2,
    pub ground_y: f32,
    pub walls: bool,
    pub tick: u64,
    pub sky: [f32; 4],
    pub sea: Option<SeaView>,
    pub actors: Vec<ActorView>,
    pub particles: Vec<ParticleView>,
    pub billboards: Vec<BillboardView>,
    pub oriented_weather: Vec<OrientedWeatherView>,
}

/// One live emitter as the draw layer needs it: where it is, how big, what
/// colour it has faded to and how far it has rolled.
///
/// `repame-view3d` has no billboard or sprite primitive — its mesh vocabulary
/// is indexed triangles, one texture page and one group alpha — so this is the
/// render seam. The quad below is built from these fields and the camera eye;
/// nothing here pretends to be the retail sprite, which is a `J2DParticle`
/// textured quad whose texel this port has no page for.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParticleView {
    pub pos: Vec3,
    /// Half-extents in the quad's own plane, x across and y up.
    pub half: Vec3,
    pub color: Rgb,
    pub alpha: f32,
    pub roll: f32,
}

/// One camera-facing weather quad, built from the already-simulated rain/sun state
/// (`tww/src/d/d_kankyo_rain.cpp:3946-4117`,
/// `wwhd/wwhd_src/d/d_kankyo_rain_drawRain.cpp:9-156`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BillboardView {
    pub pos: Vec3,
    pub half: Vec2,
    pub color: Rgb,
    pub alpha: f32,
    pub roll: f32,
    /// Index into `App::weather_textures`; stars and flare rays are untextured.
    pub page: usize,
    pub textured: bool,
    pub depth_test: bool,
}

/// Weather drawn on a world-space plane rather than camera-facing (`d_kankyo_rain.cpp:4220-4234`
/// for sibuki's XZ splash, `:5332-5370` for wave's sheared crest).
#[derive(Debug, Clone, PartialEq)]
pub struct OrientedWeatherView {
    pub corners: [[f32; 3]; 4],
    pub color: Rgb,
    pub alpha: f32,
    pub page: usize,
    pub textured: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeaView {
    pub origin: Vec2,
    pub side: usize,
    pub heights: Vec<f32>,
    pub color: [f32; 3],
}

pub fn sim_yaw_for_camera(cam_yaw: f32) -> f32 {
    (-cam_yaw.cos()).atan2(-cam_yaw.sin())
}

/// The room's texture pages, as array layers.
///
/// `repame-view3d` creates the batch's pages as one square D2 array texture, and
/// `wgpu-hal`'s GL backend picks the GL texture target from the layer count alone:
/// square D2 textures whose layer count is a multiple of six become cube or cube
/// array textures (`TextureDescriptor::is_cube_compatible`, wgpu-hal issues #1614
/// and #1574), and a single layer becomes a plain 2D texture. The batch still asks
/// for an array view, and WebGL2 has no cube array sampler, so on the web every
/// page of a twelve-page room fails to upload. One unused layer keeps the target a
/// 2D array; no page indexes the padding.
fn array_layers(pages: usize) -> u32 {
    let pages = pages.max(1);
    if pages == 1 || pages % 6 == 0 {
        (pages + 1) as u32
    } else {
        pages as u32
    }
}

pub fn batch_desc(room: Option<&RoomMesh>) -> BatchDesc {
    match room {
        Some(room) => BatchDesc {
            layer_size: LAYER_SIZE,
            layers: array_layers(room.textures.len()),
            filter: SceneFilter::Linear,
            mips: 1,
        },
        None => BatchDesc::default(),
    }
}

/// The batch's page count when the player model's textures share the frame with the room's:
/// the actor's pages sit after the room's, the same order [`build_frame_with_actor`] uploads
/// them in.
pub fn batch_desc_with_actor(
    room: Option<&RoomMesh>,
    actor: Option<&RoomMesh>,
    hair: Option<&RoomMesh>,
    hands: Option<&RoomMesh>,
    held: &[AttachedPart<'_>],
    npcs: &[NpcDraw<'_>],
    blur: bool,
    weather_pages: usize,
) -> BatchDesc {
    match (room, actor, hair, hands) {
        (None, None, None, None)
            if held.is_empty() && npcs.is_empty() && !blur && weather_pages == 0 =>
        {
            BatchDesc::default()
        }
        _ => BatchDesc {
            layer_size: LAYER_SIZE,
            layers: array_layers(
                room.map_or(0, |room| room.textures.len())
                    + actor.map_or(0, |actor| used_pages(actor).len())
                    + hair.map_or(0, |hair| used_pages(hair).len())
                    + hands.map_or(0, |hands| used_pages(hands).len())
                    + held
                        .iter()
                        .map(|part| used_pages(part.mesh).len())
                        .sum::<usize>()
                    + npcs
                        .iter()
                        .map(|npc| {
                            used_pages(npc.mesh).len()
                                + npc.cloth.map_or(0, |(_, mesh)| used_pages(mesh).len())
                        })
                        .sum::<usize>()
                    + usize::from(blur)
                    + weather_pages,
            ),
            filter: SceneFilter::Linear,
            mips: 1,
        },
    }
}

/// The actor pages its groups actually name, packed in page order. A BDL archive carries one
/// texture per part (`cl.bdl` has 41, the body uses five), and uploading the unused ones would
/// cost a page each in the frame's texture array.
fn used_pages(actor: &RoomMesh) -> Vec<u32> {
    let mut pages: Vec<u32> = actor
        .groups
        .iter()
        .filter_map(|group| group.texture_page)
        .collect();
    pages.sort_unstable();
    pages.dedup();
    pages
}

/// `J3DMaterial::load` sets the GX cull mode from the material (`J3DMaterial.cpp:194`), and
/// `GXCullMode` is {none 0, front 1, back 2, all 3} (`GXEnum.h:16-21`). The group keeps the
/// material's own face selection and an all-cull material draws nothing.
fn group_indices(group: &crate::j3d::bdl::RoomGroup) -> (Vec<u32>, CullMode) {
    match group.cull {
        Some(0) => (group.indices.clone(), CullMode::None),
        Some(1) => (group.indices.clone(), CullMode::Front),
        Some(3) => (Vec::new(), CullMode::Back),
        _ => (group.indices.clone(), CullMode::Back),
    }
}

/// The material's alpha test as the renderer's single cutoff. `J3DAlphaComp::load`
/// (`J3DMatBlock.h:745-747`) compares the fragment's alpha against two references and
/// combines them; the renderer discards below one cutoff (`render.rs:322`), so the tests
/// that name a lower bound map onto it and the rest keep every fragment.
fn alpha_cutoff(test: Option<&crate::j3d::bdl::AlphaTest>) -> f32 {
    const NEVER: u8 = 0;
    const LEQUAL: u8 = 3;
    const GREATER: u8 = 4;
    const GEQUAL: u8 = 6;
    const ALWAYS: u8 = 7;
    const OP_AND: u8 = 0;
    const OP_OR: u8 = 1;
    let Some(test) = test else {
        return 0.0;
    };
    if test.comp0 == ALWAYS {
        return 0.0;
    }
    let lower = |comp: u8, reference: u8| -> Option<f32> {
        match comp {
            GREATER => Some((f32::from(reference) + 1.0) / 255.0),
            GEQUAL => Some(f32::from(reference) / 255.0),
            ALWAYS => Some(0.0),
            _ => None,
        }
    };
    if let (Some(first), Some(second)) =
        (lower(test.comp0, test.ref0), lower(test.comp1, test.ref1))
    {
        return match test.op {
            OP_AND => first.max(second),
            OP_OR => first.min(second),
            _ => first.min(second),
        };
    }
    match test.comp0 {
        GEQUAL if test.comp1 == ALWAYS || (test.comp1 == LEQUAL && test.ref1 == u8::MAX) => {
            f32::from(test.ref0) / 255.0
        }
        GREATER if test.comp1 == ALWAYS => (f32::from(test.ref0) + 1.0) / 255.0,
        NEVER => 1.0,
        _ => 0.0,
    }
}

/// `J3DMaterial`'s blend mode and `J3DZModeInfo::mUpdateEnable` (`J3DStruct.h:230-259`). The
/// renderer's transparent pass is the only way to turn depth writes off, so a material that does
/// not update Z joins it even when its blend mode is NONE; with alpha 1 that is the same replace
/// the GX pass does.
fn transparent(group: &crate::j3d::bdl::RoomGroup) -> bool {
    group.blend != crate::j3d::bdl::GX_BM_NONE || !group.z_update
}

fn actor_page(group: &crate::j3d::bdl::RoomGroup, pages: &[u32], page_base: u32) -> u32 {
    group
        .texture_page
        .and_then(|page| pages.iter().position(|candidate| *candidate == page))
        .map_or(page_base, |slot| page_base + slot as u32)
}

pub fn build_frame(snapshot: &SceneSnapshot, cam: OrbitCamera, room: Option<&RoomMesh>) -> Frame3d {
    let cam = OrbitCamera {
        target: Vec3::new(
            snapshot.player_pos.x,
            snapshot.ground_y,
            snapshot.player_pos.z,
        ),
        ..cam
    };
    build_frame_with_actor(
        snapshot,
        cam,
        room,
        None,
        None,
        None,
        None,
        &[],
        &[],
        None,
        &[],
    )
}

pub fn build_frame_with_actor(
    snapshot: &SceneSnapshot,
    cam: OrbitCamera,
    room: Option<&RoomMesh>,
    actor: Option<&RoomMesh>,
    actor_anim: Option<&crate::assets::PlayerIdle>,
    hair: Option<&RoomMesh>,
    hands: Option<&RoomMesh>,
    held: &[AttachedPart<'_>],
    npcs: &[NpcDraw<'_>],
    blur: Option<&SwordBlurDraw<'_>>,
    weather_textures: &[Option<(Vec<u8>, u32, u32)>],
) -> Frame3d {
    let mut frame = Frame3d {
        cam,
        background: Some(snapshot.sky),
        desc: batch_desc_with_actor(
            room,
            actor,
            hair,
            hands,
            held,
            npcs,
            blur.is_some(),
            if snapshot.billboards.is_empty() && snapshot.oriented_weather.is_empty() {
                0
            } else {
                weather_textures.len()
            },
        ),
        // Retail culls by the view frustum (`J3DShape` / `J3DViewMtx`); the GameCube has
        // no hierarchical-Z test, so the engine's armed-by-default occlusion feed would
        // both cull groups this port never asked it to and append a depth pass + readback
        // whose pyramid nobody reads.
        occlusion_feed: None,
        ..Frame3d::default()
    };
    let eye = frame.cam.eye();
    match room {
        Some(room) => {
            for group in &room.groups {
                frame.push(room_group(room, group, snapshot.tick));
            }
            frame
                .uploads
                .extend(
                    room.textures
                        .iter()
                        .enumerate()
                        .map(|(page, texture)| SceneUpload {
                            page: page as u32,
                            x: 0,
                            y: 0,
                            w: texture.width,
                            h: texture.height,
                            rgba: texture.rgba.clone(),
                        }),
                );
        }
        None => frame.push(ground_group(snapshot)),
    }
    if let Some(sea) = &snapshot.sea {
        frame.push(sea_group(sea, eye));
    }
    if snapshot.walls {
        frame.push(walls_group(snapshot, eye));
    }
    if !snapshot.actors.is_empty() {
        frame.push(actors_group(snapshot, eye));
    }
    for particle in particle_groups(snapshot, eye) {
        frame.push(particle);
    }
    // `checkPlayerNoDraw()` (`d_a_player_main.h:1980`): a raised `daPyFlg0_NO_DRAW` keeps the
    // player's model out of the frame.
    if let Some(mesh) = actor.filter(|_| !snapshot.player_no_draw) {
        let page_base = room.map_or(0, |room| room.textures.len() as u32);
        let pages = used_pages(mesh);
        for (slot, page) in pages.iter().enumerate() {
            let Some(texture) = mesh.textures.get(*page as usize) else {
                continue;
            };
            frame.uploads.push(SceneUpload {
                page: page_base + slot as u32,
                x: 0,
                y: 0,
                w: texture.width,
                h: texture.height,
                rgba: texture.rgba.clone(),
            });
        }
        let pose = match actor_anim {
            Some(anim) => anim.pose_named(
                snapshot
                    .player_anm
                    .map(|anm| (anm.stem, anm.frame, anm.looping)),
                // The bck tracks are keyed at 60 Hz and this sim steps at 30 Hz
                // ([`crate::SIM_HZ`]), so a sim tick is two animation frames.
                snapshot.tick as f32 * 2.0,
            ),
            None => mesh.joint_matrices.clone(),
        };
        let placed = place_actor(pose.clone(), snapshot);
        // When hands.bdl supplies a hand, hide cl.bdl's own hand shapes so two hands do not
        // overlap (tww/src/d/actor/d_a_player_main.cpp:1392-1393,1469-1484). Those shapes are
        // direct-matrix, so only the static pass needs the filter.
        let hand_joints: Vec<u16> = if hands.is_some() {
            [
                mesh.find_node_id(b"cl_LhandA").unwrap_or(8),
                mesh.find_node_id(b"cl_RhandA").unwrap_or(12),
            ]
            .into_iter()
            .map(|j| j as u16)
            .collect()
        } else {
            Vec::new()
        };
        for group in skinned_groups(mesh, page_base, &pages, &placed) {
            frame.push(group);
        }
        for group in static_actor_groups(mesh, snapshot, page_base, &pages, &pose, &hand_joints) {
            frame.push(group);
        }
        if let Some(hair) = hair {
            let hair_pages = used_pages(hair);
            let hair_page_base = page_base + pages.len() as u32;
            for (slot, page) in hair_pages.iter().enumerate() {
                let Some(texture) = hair.textures.get(*page as usize) else {
                    continue;
                };
                frame.uploads.push(SceneUpload {
                    page: hair_page_base + slot as u32,
                    x: 0,
                    y: 0,
                    w: texture.width,
                    h: texture.height,
                    rgba: texture.rgba.clone(),
                });
            }
            let head = mesh
                .joint_names
                .iter()
                .position(|name| name == "head_jnt")
                .unwrap_or(15);
            for group in hair_groups(hair, snapshot, hair_page_base, &hair_pages, &pose, head) {
                frame.push(group);
            }
        }
        if let Some(hands) = hands {
            let hand_pages = used_pages(hands);
            let hand_page_base =
                page_base + pages.len() as u32 + hair.map_or(0, |h| used_pages(h).len()) as u32;
            for (slot, page) in hand_pages.iter().enumerate() {
                let Some(texture) = hands.textures.get(*page as usize) else {
                    continue;
                };
                frame.uploads.push(SceneUpload {
                    page: hand_page_base + slot as u32,
                    x: 0,
                    y: 0,
                    w: texture.width,
                    h: texture.height,
                    rgba: texture.rgba.clone(),
                });
            }
            // Retail overwrites the pose joint of the chosen hands shape with the body's
            // `cl_LhandA`/`cl_RhandA` world matrix (`d_a_player_main.cpp:1471-1483`;
            // `wwhd/wwhd_src/d/actor/d_a_player_main_01.cpp:3371-3406`). Bind-pose shape
            // selection stands in for the reference's per-animation hand pick.
            let l_src = mesh.find_node_id(b"cl_LhandA").unwrap_or(8) as usize;
            let r_src = mesh.find_node_id(b"cl_RhandA").unwrap_or(12) as usize;
            let l_hand = hands.find_node_id(b"cl_LhandD");
            let r_hand = hands.find_node_id(b"cl_RhandC");
            let mut selected = Vec::new();
            if let Some(joint) = l_hand {
                selected.push((joint as u16, l_src));
            }
            if let Some(joint) = r_hand {
                selected.push((joint as u16, r_src));
            }
            for group in hands_groups(
                hands,
                snapshot,
                hand_page_base,
                &hand_pages,
                &pose,
                &selected,
            ) {
                frame.push(group);
            }
        }
        // Held weapon/shield parts, each attached to one body hand joint the way retail
        // sets the part model's base matrix (tww/src/d/actor/d_a_player_main.cpp:1150-1159,
        // :1274, :1311-1315). Page slots follow hands in the same order batch_desc_with_actor
        // budgeted them.
        let mut held_page = page_base
            + pages.len() as u32
            + hair.map_or(0, |h| used_pages(h).len()) as u32
            + hands.map_or(0, |h| used_pages(h).len()) as u32;
        for part in held {
            let part_pages = used_pages(part.mesh);
            for (slot, page) in part_pages.iter().enumerate() {
                let Some(texture) = part.mesh.textures.get(*page as usize) else {
                    continue;
                };
                frame.uploads.push(SceneUpload {
                    page: held_page + slot as u32,
                    x: 0,
                    y: 0,
                    w: texture.width,
                    h: texture.height,
                    rgba: texture.rgba.clone(),
                });
            }
            if let Some(body) = pose.get(part.body_joint).copied() {
                let locals: Vec<Mat4> = part.anim.map_or_else(
                    || part.mesh.joint_matrices.clone(),
                    |(m, f)| m.joint_matrices(f),
                );
                let mut body = body * part.local;
                // Retail yaws the stowed mirror shield about its own origin and restores the
                // matrix's translation columns (`d_a_player_main.cpp:1325-1337`). Conjugating by
                // that translation is the same operation in glam's column-vector convention.
                if part.sway != 0.0 {
                    let origin = body.w_axis.truncate();
                    body = Mat4::from_translation(origin)
                        * Mat4::from_rotation_y(part.sway)
                        * Mat4::from_translation(-origin)
                        * body;
                }
                for group in attached_groups(
                    part.mesh,
                    snapshot,
                    held_page,
                    &part_pages,
                    body,
                    &locals,
                    part.color,
                    part.uv,
                ) {
                    frame.push(group);
                }
            }
            held_page += part_pages.len() as u32;
        }
    }
    // NPC pages sit after every player page, in the same order the batch counted them.
    let mut npc_page = room.map_or(0, |room| room.textures.len() as u32)
        + actor.map_or(0, |mesh| used_pages(mesh).len() as u32)
        + hair.map_or(0, |mesh| used_pages(mesh).len() as u32)
        + hands.map_or(0, |mesh| used_pages(mesh).len() as u32)
        + held
            .iter()
            .map(|part| used_pages(part.mesh).len() as u32)
            .sum::<u32>();
    for npc in npcs {
        let npc_page_base = npc_page;
        let pages = used_pages(npc.mesh);
        npc_page += pages.len() as u32;
        for (slot, page) in pages.iter().enumerate() {
            let Some(texture) = npc.mesh.textures.get(*page as usize) else {
                continue;
            };
            frame.uploads.push(SceneUpload {
                page: npc_page_base + slot as u32,
                x: 0,
                y: 0,
                w: texture.width,
                h: texture.height,
                rgba: texture.rgba.clone(),
            });
        }
        let pose = npc.anim.map_or_else(
            || npc.mesh.joint_matrices.clone(),
            |(morph, frame)| morph.joint_matrices(frame),
        );
        let to_world = Mat4::from_translation(npc.pos) * Mat4::from_rotation_y(npc.yaw);
        let mut placed: Vec<Mat4> = pose.iter().map(|joint| to_world * *joint).collect();
        if let Some((head, head_y, backbone, backbone_x)) = npc.joint_control {
            if let Some(matrix) = placed.get_mut(head as usize) {
                *matrix = *matrix * Mat4::from_rotation_y(head_y);
            }
            if let Some(matrix) = placed.get_mut(backbone as usize) {
                *matrix = *matrix * Mat4::from_rotation_x(backbone_x);
            }
        }
        for group in skinned_groups(npc.mesh, npc_page_base, &pages, &placed) {
            frame.push(group);
        }
        if let Some((joint, mesh)) = npc.cloth {
            let cloth_base = npc_page;
            let cloth_pages = used_pages(mesh);
            npc_page += cloth_pages.len() as u32;
            for (slot, page) in cloth_pages.iter().enumerate() {
                let Some(texture) = mesh.textures.get(*page as usize) else {
                    continue;
                };
                frame.uploads.push(SceneUpload {
                    page: cloth_base + slot as u32,
                    x: 0,
                    y: 0,
                    w: texture.width,
                    h: texture.height,
                    rgba: texture.rgba.clone(),
                });
            }
            if let Some(body) = placed.get(joint as usize).copied() {
                for group in attached_groups(
                    mesh,
                    snapshot,
                    cloth_base,
                    &cloth_pages,
                    body,
                    &mesh.joint_matrices,
                    None,
                    None,
                ) {
                    frame.push(group);
                }
            }
        }
    }
    if let Some(blur) = blur {
        if blur.points.len() >= 2 {
            frame.uploads.push(SceneUpload {
                page: npc_page,
                x: 0,
                y: 0,
                w: blur.width,
                h: blur.height,
                rgba: blur.rgba.to_vec(),
            });
            let mut positions = Vec::new();
            let mut uvs = Vec::new();
            for (index, point) in blur.points.iter().enumerate() {
                let direction = blur.points.get(index + 1).unwrap_or(point).clone()
                    - blur
                        .points
                        .get(index.wrapping_sub(1))
                        .unwrap_or(point)
                        .clone();
                let side = direction.cross(Vec3::Y);
                let side = if side.length_squared() < 1e-6 {
                    Vec3::X
                } else {
                    side.normalize()
                } * 3.0;
                positions.extend([
                    [point.x - side.x, point.y - side.y, point.z - side.z],
                    [point.x + side.x, point.y + side.y, point.z + side.z],
                ]);
                let v = index as f32 / (blur.points.len() - 1) as f32;
                uvs.extend([[0.0, v], [1.0, v]]);
            }
            let indices = (0..blur.points.len() - 1)
                .flat_map(|i| {
                    let base = (i * 2) as u32;
                    [base, base + 1, base + 2, base + 1, base + 3, base + 2]
                })
                .collect();
            frame.push(MeshGroup {
                positions,
                uvs,
                indices,
                colors: vec![[1.0, 1.0, 1.0]; blur.points.len() * 2],
                texture_page: npc_page,
                alpha_cutoff: 0.02,
                transparent: true,
                depth_test: false,
                ..MeshGroup::default()
            });
        }
    }
    if !snapshot.billboards.is_empty() || !snapshot.oriented_weather.is_empty() {
        let weather_base = npc_page + u32::from(blur.is_some());
        for (index, texture) in weather_textures.iter().enumerate() {
            if let Some((rgba, width, height)) = texture {
                frame.uploads.push(SceneUpload {
                    page: weather_base + index as u32,
                    x: 0,
                    y: 0,
                    w: *width,
                    h: *height,
                    rgba: rgba.clone(),
                });
            }
        }
        let eye = cam.eye();
        for board in &snapshot.billboards {
            let forward = (eye - board.pos).normalize_or(Vec3::Z);
            let right = Vec3::Y.cross(forward);
            let right = if right.length_squared() < 1e-6 {
                Vec3::X
            } else {
                right.normalize()
            };
            let up = forward.cross(right);
            let (sin, cos) = board.roll.sin_cos();
            let axis_u = right * cos + up * sin;
            let axis_v = up * cos - right * sin;
            let a = board.pos - axis_u * board.half.x - axis_v * board.half.y;
            let b = board.pos + axis_u * board.half.x - axis_v * board.half.y;
            let c = board.pos + axis_u * board.half.x + axis_v * board.half.y;
            let d = board.pos - axis_u * board.half.x + axis_v * board.half.y;
            let mut group = MeshGroup {
                alpha: board.alpha,
                texture_page: weather_base + board.page as u32,
                transparent: true,
                depth_test: board.depth_test,
                cull_mode: CullMode::None,
                ..MeshGroup::default()
            };
            if board.textured {
                group.push_quad_textured(
                    [a.x, a.y, a.z],
                    [b.x, b.y, b.z],
                    [c.x, c.y, c.z],
                    [d.x, d.y, d.z],
                    board.color,
                    [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
                );
            } else {
                group.push_quad(
                    [a.x, a.y, a.z],
                    [b.x, b.y, b.z],
                    [c.x, c.y, c.z],
                    [d.x, d.y, d.z],
                    board.color,
                );
            }
            frame.push(group);
        }
        for quad in &snapshot.oriented_weather {
            let mut group = MeshGroup {
                alpha: quad.alpha,
                texture_page: weather_base + quad.page as u32,
                transparent: true,
                depth_test: true,
                cull_mode: CullMode::None,
                ..MeshGroup::default()
            };
            if quad.textured {
                group.push_quad_textured(
                    quad.corners[0],
                    quad.corners[1],
                    quad.corners[2],
                    quad.corners[3],
                    quad.color,
                    [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
                );
            } else {
                group.push_quad(
                    quad.corners[0],
                    quad.corners[1],
                    quad.corners[2],
                    quad.corners[3],
                    quad.color,
                );
            }
            frame.push(group);
        }
    }
    frame
}

/// `mpKatsuraModel->setBaseTRMtx(mpCLModel->getAnmMtx(CL_JNT_HEAD_JNT_e))`
/// (`d_a_player_main.cpp:11598`): the hair model's vertices are in head-local space, not in
/// `katsura.bdl`'s own root, and the head's animated matrix places them.
fn hair_groups(
    hair: &RoomMesh,
    snapshot: &SceneSnapshot,
    page_base: u32,
    pages: &[u32],
    pose: &[Mat4],
    head: usize,
) -> Vec<MeshGroup> {
    let Some(head) = pose.get(head) else {
        return Vec::new();
    };
    let origin = Vec3::new(
        snapshot.player_pos.x,
        snapshot.player_pos.y,
        snapshot.player_pos.z,
    );
    let transform =
        Mat4::from_translation(origin) * Mat4::from_rotation_y(-snapshot.player_yaw) * *head;
    hair.groups
        .iter()
        .filter(|group| group.joint_indices.is_empty())
        .map(|group| {
            let positions = group
                .positions
                .iter()
                .map(|position| {
                    let point = transform.transform_point3(Vec3::from(*position));
                    [point.x, point.y, point.z]
                })
                .collect();
            let normals = if group.lit {
                group
                    .normals
                    .iter()
                    .map(|normal| {
                        let normal = transform
                            .transform_vector3(Vec3::from(*normal))
                            .normalize_or_zero();
                        [normal.x, normal.y, normal.z]
                    })
                    .collect()
            } else {
                Vec::new()
            };
            let (indices, cull_mode) = group_indices(group);
            MeshGroup {
                positions,
                colors: group.colors.clone(),
                normals,
                uvs: group.uvs.clone(),
                indices,
                texture_page: actor_page(group, pages, page_base),
                alpha_cutoff: alpha_cutoff(group.alpha_test.as_ref()),
                depth_test: group.z_compare,
                transparent: transparent(group),
                cull_mode,
                ..MeshGroup::default()
            }
        })
        .collect()
}

/// The selected `hands.bdl` shapes, each placed by the body joint matrix the reference copies
/// into it (`tww/src/d/actor/d_a_player_main.cpp:1471-1483`,
/// `wwhd/wwhd_src/d/actor/d_a_player_main_01.cpp:3371-3406`). Every hands
/// joint bind local is identity, so its baked vertices need only `to_world * pose[body_joint]`.
fn hands_groups(
    hands: &RoomMesh,
    snapshot: &SceneSnapshot,
    page_base: u32,
    pages: &[u32],
    pose: &[Mat4],
    selected: &[(u16, usize)],
) -> Vec<MeshGroup> {
    let origin = Vec3::new(
        snapshot.player_pos.x,
        snapshot.player_pos.y,
        snapshot.player_pos.z,
    );
    let to_world = Mat4::from_translation(origin) * Mat4::from_rotation_y(-snapshot.player_yaw);
    hands
        .groups
        .iter()
        .filter_map(|group| {
            let joint = group.direct_joint?;
            let (_, src) = selected.iter().find(|(j, _)| *j == joint)?;
            let transform = to_world * *pose.get(*src)?;
            let positions = group
                .positions
                .iter()
                .map(|position| {
                    let point = transform.transform_point3(Vec3::from(*position));
                    [point.x, point.y, point.z]
                })
                .collect();
            let normals = if group.lit {
                group
                    .normals
                    .iter()
                    .map(|normal| {
                        let normal = transform
                            .transform_vector3(Vec3::from(*normal))
                            .normalize_or_zero();
                        [normal.x, normal.y, normal.z]
                    })
                    .collect()
            } else {
                Vec::new()
            };
            let (indices, cull_mode) = group_indices(group);
            Some(MeshGroup {
                positions,
                colors: group.colors.clone(),
                normals,
                uvs: group.uvs.clone(),
                indices,
                texture_page: actor_page(group, pages, page_base),
                alpha_cutoff: alpha_cutoff(group.alpha_test.as_ref()),
                depth_test: group.z_compare,
                transparent: transparent(group),
                cull_mode,
                ..MeshGroup::default()
            })
        })
        .collect()
}

/// A parts model placed at one body joint, the port's `setBaseTRMtx(lHandA_jnt_mtx)` /
/// `(rHandA_jnt_mtx)` (`tww/src/d/actor/d_a_player_main.cpp:1150-1159,1274,1311-1315`;
/// `wwhd/wwhd_src/d/actor/d_a_player_main_04.cpp:3174-3181,3220-3224`). `body` is the joint's
/// animated matrix in player-model space. Skinned groups use the part's own EVP1 inverse bind
/// (which is not `inverse(JNT1 bind world)`, `j3d/bdl.rs:191-194`); direct groups cancel the
/// part's own bind matrix so the part rests in the body joint's pose.
fn tinted(group: &RoomGroup, tint: Option<(u16, [f32; 3])>) -> Vec<[f32; 3]> {
    let Some((slot, multiplier)) = tint.filter(|(slot, _)| group.material == Some(*slot)) else {
        return group.colors.clone();
    };
    group
        .colors
        .iter()
        .map(|c| {
            [
                c[0] * multiplier[0],
                c[1] * multiplier[1],
                c[2] * multiplier[2],
            ]
        })
        .collect()
}

fn btk_uvs(
    part: &RoomMesh,
    group: &RoomGroup,
    anim: Option<(u16, crate::j3d::btk::UvMatrix)>,
) -> Vec<[f32; 2]> {
    let Some((slot, matrix)) = anim.filter(|(slot, _)| group.material == Some(*slot)) else {
        return group.uvs.clone();
    };
    let placement = group
        .texture_page
        .and_then(|page| part.uv_placements.get(page as usize));
    group
        .uvs
        .iter()
        .map(|uv| {
            let (u, v) = match placement {
                Some(p) => (uv[0] / p[0] + p[2], uv[1] / p[1] + p[3]),
                None => (uv[0], uv[1]),
            };
            let mapped_u = matrix.a * u + matrix.b * v + matrix.tx;
            let mapped_v = matrix.c * u + matrix.d * v + matrix.ty;
            match placement {
                Some(p) => [(mapped_u - p[2]) * p[0], (mapped_v - p[3]) * p[1]],
                None => [mapped_u, mapped_v],
            }
        })
        .collect()
}

fn attached_groups(
    part: &RoomMesh,
    snapshot: &SceneSnapshot,
    page_base: u32,
    pages: &[u32],
    body: Mat4,
    locals: &[Mat4],
    tint: Option<(u16, [f32; 3])>,
    uv_anim: Option<(u16, crate::j3d::btk::UvMatrix)>,
) -> Vec<MeshGroup> {
    if locals.len() != part.joints.len() {
        return Vec::new();
    }
    let origin = Vec3::new(
        snapshot.player_pos.x,
        snapshot.player_pos.y,
        snapshot.player_pos.z,
    );
    let to_world = Mat4::from_translation(origin) * Mat4::from_rotation_y(-snapshot.player_yaw);
    let base = to_world * body;
    let part_pose: Vec<Mat4> = locals.iter().map(|local| base * *local).collect();
    let inverse_bind: Vec<Mat4> = if part.inverse_bind.len() == part.joint_matrices.len() {
        part.inverse_bind.clone()
    } else {
        part.joint_matrices
            .iter()
            .map(|m| m.try_inverse().unwrap_or(Mat4::IDENTITY))
            .collect()
    };

    let mut out = Vec::new();
    for group in &part.groups {
        if !group.joint_indices.is_empty() {
            if group.joint_indices.len() != group.positions.len() {
                continue;
            }
            let (indices, cull_mode) = group_indices(group);
            let mesh = SkinnedMesh {
                positions: group.positions.clone(),
                normals: group.normals.clone(),
                colors: tinted(group, tint),
                uvs: btk_uvs(part, group, uv_anim),
                indices,
                joints: group.joint_indices.clone(),
                weights: group.joint_weights.clone(),
                inverse_bind: inverse_bind.clone(),
                joint_nodes: (0..part.joints.len()).collect(),
                texture_page: actor_page(group, pages, page_base),
                alpha_cutoff: alpha_cutoff(group.alpha_test.as_ref()),
                depth_test: group.z_compare,
                transparent: transparent(group),
                ..SkinnedMesh::default()
            };
            let mut baked = mesh.pose(&part_pose);
            baked.cull_mode = cull_mode;
            out.push(baked);
            continue;
        }
        let transform = match group.direct_joint {
            Some(joint) => {
                let (Some(bind), Some(posed)) = (
                    part.joint_matrices.get(joint as usize),
                    part_pose.get(joint as usize),
                ) else {
                    continue;
                };
                let Some(inverse) = bind.try_inverse() else {
                    continue;
                };
                *posed * inverse
            }
            None => base,
        };
        let positions = group
            .positions
            .iter()
            .map(|p| {
                let v = transform.transform_point3(Vec3::from(*p));
                [v.x, v.y, v.z]
            })
            .collect();
        let normals = if group.lit {
            group
                .normals
                .iter()
                .map(|n| {
                    let v = transform
                        .transform_vector3(Vec3::from(*n))
                        .normalize_or_zero();
                    [v.x, v.y, v.z]
                })
                .collect()
        } else {
            Vec::new()
        };
        let (indices, cull_mode) = group_indices(group);
        out.push(MeshGroup {
            positions,
            colors: tinted(group, tint),
            normals,
            uvs: btk_uvs(part, group, uv_anim),
            indices,
            texture_page: actor_page(group, pages, page_base),
            alpha_cutoff: alpha_cutoff(group.alpha_test.as_ref()),
            depth_test: group.z_compare,
            transparent: transparent(group),
            cull_mode,
            ..MeshGroup::default()
        });
    }
    out
}

fn skinned_groups(
    actor: &RoomMesh,
    page_base: u32,
    pages: &[u32],
    placed: &[Mat4],
) -> Vec<MeshGroup> {
    if actor.joint_matrices.len() != actor.joints.len() {
        return Vec::new();
    }
    let inverse_bind: Vec<Mat4> = if actor.inverse_bind.len() == actor.joint_matrices.len() {
        actor.inverse_bind.clone()
    } else {
        actor
            .joint_matrices
            .iter()
            .map(|matrix| matrix.try_inverse().unwrap_or(Mat4::IDENTITY))
            .collect()
    };
    actor
        .groups
        .iter()
        .enumerate()
        .filter(|(_, group)| {
            !group.joint_indices.is_empty() && group.joint_indices.len() == group.positions.len()
        })
        .map(|(_, group)| {
            let (indices, cull_mode) = group_indices(group);
            let mut mesh = SkinnedMesh {
                positions: group.positions.clone(),
                normals: group.normals.clone(),
                uvs: group.uvs.clone(),
                colors: group.colors.clone(),
                indices,
                joints: group.joint_indices.clone(),
                weights: group.joint_weights.clone(),
                inverse_bind: inverse_bind.clone(),
                joint_nodes: (0..actor.joints.len()).collect(),
                texture_page: actor_page(group, pages, page_base),
                alpha_cutoff: alpha_cutoff(group.alpha_test.as_ref()),
                depth_test: group.z_compare,
                transparent: transparent(group),
                ..SkinnedMesh::default()
            };
            mesh.name = "cl".to_string();
            let mut baked = mesh.pose(placed);
            baked.cull_mode = cull_mode;
            baked
        })
        .collect()
}

/// Direct-matrix shapes: the BDL draw has no skin weights, so its vertices are the joint's bind
/// world (`read_shapes` adds `worlds[joint]`). The joint's animated delta carries the whole shape.
fn static_actor_groups(
    actor: &RoomMesh,
    snapshot: &SceneSnapshot,
    page_base: u32,
    pages: &[u32],
    pose: &[Mat4],
    hidden: &[u16],
) -> Vec<MeshGroup> {
    let origin = Vec3::new(
        snapshot.player_pos.x,
        snapshot.player_pos.y,
        snapshot.player_pos.z,
    );
    let to_world = Mat4::from_translation(origin) * Mat4::from_rotation_y(-snapshot.player_yaw);
    actor
        .groups
        .iter()
        .filter(|group| group.joint_indices.is_empty())
        .filter(|group| {
            !group
                .direct_joint
                .is_some_and(|joint| hidden.contains(&joint))
        })
        .filter_map(|group| {
            let transform = match group.direct_joint {
                Some(joint) => {
                    let bind = actor.joint_matrices.get(joint as usize)?;
                    let animated = pose.get(joint as usize)?;
                    to_world * *animated * bind.try_inverse()?
                }
                None => to_world,
            };
            let positions = group
                .positions
                .iter()
                .map(|position| {
                    let point = transform.transform_point3(Vec3::from(*position));
                    [point.x, point.y, point.z]
                })
                .collect();
            let normals = if group.lit {
                group
                    .normals
                    .iter()
                    .map(|normal| {
                        let normal = transform
                            .transform_vector3(Vec3::from(*normal))
                            .normalize_or_zero();
                        [normal.x, normal.y, normal.z]
                    })
                    .collect()
            } else {
                Vec::new()
            };
            let (indices, cull_mode) = group_indices(group);
            Some(MeshGroup {
                positions,
                colors: group.colors.clone(),
                normals,
                uvs: group.uvs.clone(),
                indices,
                texture_page: actor_page(group, pages, page_base),
                alpha_cutoff: alpha_cutoff(group.alpha_test.as_ref()),
                depth_test: group.z_compare,
                transparent: transparent(group),
                cull_mode,
                ..MeshGroup::default()
            })
        })
        .collect()
}

/// Places a posed actor at the player, in world space.
///
/// `pose` holds one animated **world** matrix per joint: the bind chain from
/// `RoomMesh::joint_matrices` with the animated locals applied on the right of
/// each one. `SkinnedMesh::pose` then multiplies each by that joint's
/// `inverse_bind`, so the bind chain cancels and what survives is the animated
/// delta — which is the identity at rest, and therefore the bind pose on screen.
///
/// Passing local rotations here instead would leave `bindWorld * inverse(bindWorld)`
/// unreduced on the right of every joint and displace the actor by the negated
/// bind translations.
fn place_actor(pose: Vec<Mat4>, snapshot: &SceneSnapshot) -> Vec<Mat4> {
    let origin = Vec3::new(
        snapshot.player_pos.x,
        snapshot.player_pos.y,
        snapshot.player_pos.z,
    );
    let to_world = Mat4::from_translation(origin) * Mat4::from_rotation_y(-snapshot.player_yaw);
    pose.into_iter().map(|joint| to_world * joint).collect()
}

pub fn skinned_influences() -> usize {
    SKIN_INFLUENCES
}

/// One camera-facing quad per live emitter.
///
/// `repame-view3d` exposes no billboard or sprite primitive, so the quad is
/// built here from the emitter's half-extents and the camera eye: the right
/// vector is the world up crossed with the view ray, the up vector closes the
/// frame, and the roll turns the pair in the quad's own plane. Each group is
/// its own alpha blend so one emitter's fade never dims another's, which is
/// also what makes the batch sort them back to front.
fn particle_groups(snapshot: &SceneSnapshot, eye: Vec3) -> Vec<MeshGroup> {
    let mut groups = Vec::with_capacity(snapshot.particles.len());
    for particle in &snapshot.particles {
        let ray = eye - particle.pos;
        let forward = ray.normalize_or(Vec3::Z);
        let mut right = Vec3::Y.cross(forward);
        if right.length_squared() < 1e-8 {
            right = Vec3::X.cross(forward);
        }
        let right = right.normalize();
        let up = forward.cross(right);
        let (sin, cos) = particle.roll.sin_cos();
        let across = right * cos - up * sin;
        let above = right * sin + up * cos;
        let (rx, ry) = (across * particle.half.x, above * particle.half.y);
        let mut group = MeshGroup {
            transparent: true,
            alpha: particle.alpha.clamp(0.0, 1.0),
            depth_test: true,
            ..MeshGroup::default()
        };
        group.push_quad(
            (particle.pos - rx - ry).into(),
            (particle.pos + rx - ry).into(),
            (particle.pos + rx + ry).into(),
            (particle.pos - rx + ry).into(),
            particle.color,
        );
        groups.push(group);
    }
    groups
}

const ENEMY: Rgb = [0.55, 0.26, 0.16];
const TALKER: Rgb = [0.32, 0.42, 0.82];
const SHIP: Rgb = [0.58, 0.44, 0.24];
const BOMB: Rgb = [0.18, 0.18, 0.22];
const ARROW: Rgb = [0.86, 0.82, 0.66];
const BOOM: Rgb = [0.72, 0.30, 0.52];
const BALLOON: Rgb = [0.86, 0.44, 0.52];
const WARSHIP: Rgb = [0.30, 0.26, 0.32];
const EMBER: Rgb = [0.94, 0.52, 0.18];
const SEAGULL: Rgb = [0.94, 0.94, 0.90];
const SCENERY: Rgb = [0.44, 0.62, 0.34];
const SPEAR: Rgb = [0.70, 0.66, 0.52];

pub fn item_hex(kind: crate::sim::item::Kind) -> &'static str {
    match kind {
        crate::sim::item::Kind::Bomb => "#2E3138",
        crate::sim::item::Kind::Arrow => "#C9C4B4",
        crate::sim::item::Kind::Boomerang => "#B84C85",
        crate::sim::item::Kind::Hookshot => "#7C8C9C",
    }
}
const HEART: Rgb = [0.86, 0.20, 0.22];
const RUPEE: Rgb = [0.24, 0.72, 0.32];
const RELIC: Rgb = [0.90, 0.52, 0.86];
const PIECE_SIZES: [f32; 2] = [14.0, 20.0];

fn pickup_look(item: u8) -> (Rgb, f32) {
    match item {
        0 => (HEART, 10.0),
        0x01..=0x06 | 0x0F => (RUPEE, 10.0),
        0x07 => (RELIC, PIECE_SIZES[0]),
        0x08 => (RELIC, PIECE_SIZES[1]),
        0x0B..=0x0E => (BOMB, 12.0),
        0x10..=0x12 => (ARROW, 10.0),
        0x2D => (BOOM, 14.0),
        _ => (RELIC, 10.0),
    }
}

fn actor_look(actor: &ActorView) -> Rgb {
    match actor.kind {
        ActorKind::Enemy => ENEMY,
        ActorKind::Talker => TALKER,
        ActorKind::Ship => SHIP,
        ActorKind::Bomb => BOMB,
        ActorKind::Arrow => ARROW,
        ActorKind::Boomerang => BOOM,
        ActorKind::Spear => SPEAR,
        ActorKind::Balloon => BALLOON,
        ActorKind::Warship => WARSHIP,
        ActorKind::Cannon => BOMB,
        ActorKind::Fire => EMBER,
        ActorKind::Gull => SEAGULL,
        ActorKind::Pickup => pickup_look(actor.item).0,
        ActorKind::Scenery => SCENERY,
    }
}

fn actors_group(snapshot: &SceneSnapshot, eye: Vec3) -> MeshGroup {
    let mut group = MeshGroup::default();
    for actor in &snapshot.actors {
        // Modelled actors are drawn from their own BDL/BMD instead of a box. `modeled` is set
        // only when a publisher found both a loaded model and a matching BCK morph, so an actor
        // whose model failed to load keeps its box and never disappears entirely.
        if actor.modeled {
            continue;
        }
        group.push_box(
            actor.pos.x,
            actor.pos.y,
            actor.pos.z,
            actor.half * 2.0,
            actor.height,
            actor.half * 2.0,
            actor_look(actor),
            eye,
        );
    }
    group.depth_test = true;
    group
}

fn room_group(room: &RoomMesh, group: &RoomGroup, tick: u64) -> MeshGroup {
    let (indices, cull_mode) = group_indices(group);
    MeshGroup {
        positions: group.positions.clone(),
        colors: group.colors.clone(),
        normals: if group.lit {
            group.normals.clone()
        } else {
            Vec::new()
        },
        uvs: animated_uvs(room, group, tick),
        indices,
        texture_page: group.texture_page.unwrap_or(0),
        alpha_cutoff: alpha_cutoff(group.alpha_test.as_ref()),
        depth_test: group.z_compare,
        transparent: transparent(group),
        cull_mode,
        ..MeshGroup::default()
    }
}

fn sea_group(sea: &SeaView, eye: Vec3) -> MeshGroup {
    use crate::sim::sea::GRID_SIZE;
    let reach = repame_view3d::FAR + GRID_SIZE;
    let reach = reach * reach;
    let mut group = MeshGroup::default();
    for z in 0..sea.side.saturating_sub(1) {
        for x in 0..sea.side.saturating_sub(1) {
            let x0 = sea.origin.x + x as f32 * GRID_SIZE;
            let z0 = sea.origin.y + z as f32 * GRID_SIZE;
            let x1 = x0 + GRID_SIZE;
            let z1 = z0 + GRID_SIZE;
            let dx = (x0 + x1) * 0.5 - eye.x;
            let dz = (z0 + z1) * 0.5 - eye.z;
            if dx * dx + dz * dz > reach {
                continue;
            }
            let corners = [
                [x0, sea.heights[(z + 1) * sea.side + x], z1],
                [x1, sea.heights[(z + 1) * sea.side + x + 1], z1],
                [x1, sea.heights[z * sea.side + x + 1], z0],
                [x0, sea.heights[z * sea.side + x], z0],
            ];
            let first = [
                corners[1][0] - corners[0][0],
                corners[1][1] - corners[0][1],
                corners[1][2] - corners[0][2],
            ];
            let second = [
                corners[3][0] - corners[0][0],
                corners[3][1] - corners[0][1],
                corners[3][2] - corners[0][2],
            ];
            let normal = [
                first[1] * second[2] - first[2] * second[1],
                first[2] * second[0] - first[0] * second[2],
                first[0] * second[1] - first[1] * second[0],
            ];
            let length =
                (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
            let normal = [normal[0] / length, normal[1] / length, normal[2] / length];
            let base = group.positions.len() as u32;
            group.positions.extend(corners);
            group.colors.extend([sea.color; 4]);
            group.normals.extend([normal; 4]);
            group
                .indices
                .extend([base, base + 1, base + 2, base, base + 2, base + 3]);
        }
    }
    group.depth_test = true;
    group
}

fn animated_uvs(room: &RoomMesh, group: &RoomGroup, tick: u64) -> Vec<[f32; 2]> {
    let (Some(binding), Some(page)) = (group.anim, group.texture_page) else {
        return group.uvs.clone();
    };
    let (Some(animation), Some(&placement)) = (
        room.animations.get(binding.set as usize),
        room.uv_placements.get(page as usize),
    ) else {
        return group.uvs.clone();
    };
    let matrix = match animation.uv_transform(binding.track, animation.frame_at(tick)) {
        Some(matrix) => matrix,
        None => return group.uvs.clone(),
    };
    group
        .uvs
        .iter()
        .map(|uv| {
            let u = (uv[0] / placement[0]) + placement[2];
            let v = (uv[1] / placement[1]) + placement[3];
            let mapped_u = matrix.a * u + matrix.b * v + matrix.tx;
            let mapped_v = matrix.c * u + matrix.d * v + matrix.ty;
            [
                (mapped_u - placement[2]) * placement[0],
                (mapped_v - placement[3]) * placement[1],
            ]
        })
        .collect()
}

fn ground_group(snapshot: &SceneSnapshot) -> MeshGroup {
    let mut group = MeshGroup::default();
    let (x0, z0) = (snapshot.bounds_min.x, snapshot.bounds_min.y);
    let (x1, z1) = (snapshot.bounds_max.x, snapshot.bounds_max.y);
    let y = snapshot.ground_y;
    group.push_quad([x0, y, z1], [x1, y, z1], [x1, y, z0], [x0, y, z0], GROUND);
    group.depth_test = true;
    group
}

fn walls_group(snapshot: &SceneSnapshot, eye: Vec3) -> MeshGroup {
    let mut group = MeshGroup::default();
    let (x0, z0) = (snapshot.bounds_min.x, snapshot.bounds_min.y);
    let (x1, z1) = (snapshot.bounds_max.x, snapshot.bounds_max.y);
    let (cx, cz) = ((x0 + x1) * 0.5, (z0 + z1) * 0.5);
    let (width, depth) = (x1 - x0, z1 - z0);
    let y = snapshot.ground_y;
    group.push_box(
        cx,
        y,
        z0,
        width + WALL_THICKNESS,
        WALL_HEIGHT,
        WALL_THICKNESS,
        WALL,
        eye,
    );
    group.push_box(
        cx,
        y,
        z1,
        width + WALL_THICKNESS,
        WALL_HEIGHT,
        WALL_THICKNESS,
        WALL,
        eye,
    );
    group.push_box(
        x0,
        y,
        cz,
        WALL_THICKNESS,
        WALL_HEIGHT,
        depth + WALL_THICKNESS,
        WALL,
        eye,
    );
    group.push_box(
        x1,
        y,
        cz,
        WALL_THICKNESS,
        WALL_HEIGHT,
        depth + WALL_THICKNESS,
        WALL,
        eye,
    );
    group.depth_test = true;
    group
}
