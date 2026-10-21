//! Turning a replayed `.dff` frame into engine mesh groups.
//!
//! A replay gives back positions, colours, normals, texture coordinates and the
//! material a draw was bound to, which is what the render path takes. What it
//! does not give back is a frame number: the texture memory a recording carries
//! is a snapshot, so a draw's page is uploaded once per recording rather than
//! once per frame, and the page index a group names is the recording's texture
//! slot.
//!
//! GX is Y-up right-handed and so is the engine's world space, and the vertex
//! positions the replay reads back are already world units, so no coordinate
//! conversion happens here.

use super::replay::{DrawGeometry, Replay};
use crate::dff::DffFile;
use crate::gx_decode::decode_tmem_texture;
use glam::Vec3;
use repame_view3d::{Frame3d, MeshGroup, OrbitCamera, SceneUpload, ShadingModel};

/// A replay laid out for the render path.
#[derive(Clone, Debug)]
pub struct Presented {
    /// One group per drawn draw, in draw order.
    pub groups: Vec<MeshGroup>,
    /// The material programs named by each group's `material_index`.
    pub materials: Vec<ShadingModel>,
    /// One page per texture slot the replay bound, in slot order.
    pub uploads: Vec<SceneUpload>,
}

/// A texture the recording carries could not be turned into a page.
#[derive(Clone, Debug)]
pub enum PresentError {
    /// The texture memory behind this slot could not be decoded, either because
    /// the recording carries too few bytes for the descriptor's format or
    /// because the format itself is not one [`decode_tmem_texture`] reads.
    Decode { slot: usize },
}

/// Build the render path's view of one replay of one recording.
///
/// `dff` is the recording the replay came from, and it is where the texture
/// memory comes from.
pub fn present(dff: &DffFile, replay: &Replay) -> Result<Presented, PresentError> {
    let uploads = replay
        .textures()
        .iter()
        .enumerate()
        .map(|(slot, desc)| {
            let rgba =
                decode_tmem_texture(&dff.tex_mem, desc).ok_or(PresentError::Decode { slot })?;
            Ok(SceneUpload {
                page: slot as u32,
                x: 0,
                y: 0,
                w: desc.width.into(),
                h: desc.height.into(),
                rgba,
            })
        })
        .collect::<Result<Vec<SceneUpload>, PresentError>>()?;

    let groups = replay
        .geometry()
        .iter()
        .map(|geometry| group(geometry, uploads.len()))
        .collect();

    Ok(Presented {
        groups,
        materials: replay.materials().to_vec(),
        uploads,
    })
}

/// A whole recording as one frame.
pub fn frame(presented: &Presented, cam: OrbitCamera) -> Frame3d {
    let mut frame = Frame3d {
        cam,
        groups: presented.groups.clone(),
        materials: presented.materials.clone(),
        uploads: presented.uploads.clone(),
        // See `view::build_frame_with_actor`: no occlusion consumer, retail has no HiZ.
        occlusion_feed: None,
        ..Frame3d::default()
    };
    if let Some((low, high)) = frame.groups.iter().find_map(|g| g.bounds()) {
        let mid = Vec3::new(
            (low[0] + high[0]) * 0.5,
            (low[1] + high[1]) * 0.5,
            (low[2] + high[2]) * 0.5,
        );
        let reach = (high[0] - low[0])
            .max(high[1] - low[1])
            .max(high[2] - low[2])
            .max(1.0);
        frame.cam.target = mid;
        frame.cam.dist = reach * 2.5;
    }
    frame
}

/// Load, replay and present a recording in one step, for the `CS_DFF` view.
pub fn load(path: &std::path::Path) -> Presented {
    let dff = DffFile::load(path).expect("dff load");
    let mut replay = Replay::new(&dff.bp_mem, &dff.cp_mem);
    for frame in &dff.frames {
        replay.frame(frame).expect("dff replay");
    }
    present(&dff, &replay).expect("dff present")
}

/// One draw's geometry as a group. A page with no upload behind it samples
/// empty texels, so an unresolved page takes the group untextured. Normals pass
/// through as the replay decoded them, one unit normal per position or none at
/// all, the shape the engine's lit path takes.
fn group(geometry: &DrawGeometry, page_count: usize) -> MeshGroup {
    let page = geometry
        .texture
        .filter(|slot| *slot < page_count)
        .unwrap_or(0);
    let uvs = if geometry.uvs.is_empty() {
        Vec::new()
    } else {
        geometry.uvs.clone()
    };
    MeshGroup {
        positions: geometry.positions.clone(),
        colors: geometry.colors.clone(),
        normals: geometry.normals.clone(),
        uvs,
        indices: geometry.indices.clone(),
        texture_page: u32::try_from(page).unwrap_or(0),
        material_index: geometry.material,
        ..MeshGroup::default()
    }
}
