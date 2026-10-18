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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dff::{DffFile, Frame};
    use crate::j3d::replay::Replay;
    use repame_view3d::OrbitCamera;

    const GX_DIRECT: u32 = 1;
    const GX_S16: u32 = 3;
    const GX_F32: u32 = 4;
    const GX_RGBA8: u32 = 5;

    fn vertex(x: f32, y: f32, z: f32, r: u8) -> Vec<u8> {
        let mut out = Vec::new();
        for value in [x, y, z] {
            out.extend_from_slice(&value.to_be_bytes());
        }
        out.extend_from_slice(&[r, r, r, 255]);
        out
    }

    /// A recording whose frame draws four lit vertices under the default
    /// material, with no texture bound.
    fn recording() -> DffFile {
        let mut file = DffFile::default();
        // Position direct, three f32s, twelve bytes (`VCD_lo` bits 9..10,
        // `VAT_A` position bits 0..8).
        file.cp_mem[0x50] = GX_DIRECT << 9;
        file.cp_mem[0x70] = 1 | (GX_F32 << 1);
        // Diffuse colour direct, `GX_RGBA8` with the RGBA count, four bytes
        // (`VCD_lo` bits 13..14, `VAT_A` colour bits 13..16).
        file.cp_mem[0x50] |= GX_DIRECT << 13;
        file.cp_mem[0x70] |= (1 << 13) | (GX_RGBA8 << 14);

        let mut stream = vec![0x98_u8, 0x00, 0x04];
        let corners = [(1.0_f32, 0.0_f32), (0.0, 1.0), (1.0, 1.0), (0.0, 0.0)];
        for (x, y) in corners {
            stream.extend_from_slice(&vertex(x, y, 0.0, 255_u8));
        }
        file.frames.push(Frame {
            fifo_data: stream,
            fifo_start: 0,
            fifo_end: 0,
            memory_updates: Vec::new(),
        });
        file
    }

    #[test]
    fn a_replayed_draw_becomes_a_group_the_frame_can_take() {
        let file = recording();
        let path = std::path::Path::new("/tmp/opencode/present_basic.dff");
        file.save(path).expect("save");
        let loaded = DffFile::load(path).expect("load");

        let mut replay = Replay::new(&loaded.bp_mem, &loaded.cp_mem);
        replay.frame(&loaded.frames[0]).expect("replay");
        let presented = present(&loaded, &replay).expect("present");

        assert_eq!(presented.groups.len(), 1);
        let group = &presented.groups[0];
        assert_eq!(group.positions.len(), 4, "the payload reads back");
        assert_eq!(
            group.indices.len(),
            6,
            "a four-vertex strip is two triangles"
        );
        assert_eq!(group.material_index, 0, "the draw points at its material");
        assert!(presented.materials.len() >= 1);
        // The synthetic recording's stage samples slot 0, so exactly one page
        // is uploaded and the group names it.
        assert_eq!(presented.uploads.len(), 1);
        assert_eq!(presented.uploads[0].page, 0);
        assert_eq!(group.texture_page, 0);
        assert!(group.uvs.is_empty(), "no coordinate set was bound");
    }

    #[test]
    fn a_recordings_vertex_colour_reaches_the_group() {
        let file = recording();
        let path = std::path::Path::new("/tmp/opencode/present_color.dff");
        file.save(path).expect("save");
        let loaded = DffFile::load(path).expect("load");

        let mut replay = Replay::new(&loaded.bp_mem, &loaded.cp_mem);
        replay.frame(&loaded.frames[0]).expect("replay");
        let presented = present(&loaded, &replay).expect("present");

        let colors = &presented.groups[0].colors;
        assert_eq!(colors.len(), 4, "one colour per position");
        assert_eq!(colors[0], [1.0, 1.0, 1.0], "full white stays full white");
    }

    #[test]
    fn a_recordings_bounds_aim_the_camera() {
        let file = recording();
        let path = std::path::Path::new("/tmp/opencode/present_frame.dff");
        file.save(path).expect("save");
        let loaded = DffFile::load(path).expect("load");

        let mut replay = Replay::new(&loaded.bp_mem, &loaded.cp_mem);
        replay.frame(&loaded.frames[0]).expect("replay");
        let presented = present(&loaded, &replay).expect("present");

        let frame = frame(&presented, OrbitCamera::default());
        assert_eq!(frame.groups.len(), 1);
        assert_eq!(frame.materials.len(), presented.materials.len());
        assert_eq!(frame.uploads.len(), presented.uploads.len());
        // The recording spans one unit, so the camera pulls back to two and a
        // half times that and looks at the middle of it.
        assert_eq!(frame.cam.dist, 2.5);
        assert_eq!(frame.cam.target.x, 0.5);
    }

    /// Normals and fixed-point texture coordinates on the way to a group: a
    /// `GX_S16` normal carries 14 fraction bits by its type
    /// (`FracAdjust`, `dolphin/Source/Core/VideoCommon/VertexLoader_Normal.cpp`)
    /// and is normalized for the render path, a `GX_S16` texture coordinate
    /// scales by the `VAT` shift it was given, 8 here.
    #[test]
    fn a_recordings_normal_and_uv_reach_the_group() {
        let mut file = DffFile::default();
        file.cp_mem[0x50] = (GX_DIRECT << 9) | (GX_DIRECT << 11);
        file.cp_mem[0x60] = GX_DIRECT;
        file.cp_mem[0x70] =
            1 | (GX_F32 << 1) | (GX_S16 << 10) | (1 << 21) | (GX_S16 << 22) | (8 << 25);

        let mut stream = vec![0x98_u8, 0x00, 0x04];
        let corners = [
            (0.0_f32, 0.0_f32, 0_u16, 0_u16),
            (1.0, 0.0, 256, 0),
            (1.0, 1.0, 256, 256),
            (0.0, 1.0, 0, 256),
        ];
        for (x, y, u, v) in corners {
            for value in [x, y, 0.0] {
                stream.extend_from_slice(&value.to_be_bytes());
            }
            // A +Z normal: `2^-14` units, so 16384 is 1.0.
            for _ in 0..3 {
                stream.extend_from_slice(&16384_i16.to_be_bytes());
            }
            // `2^-8` units, so 256 is 1.0.
            stream.extend_from_slice(&(u as i16).to_be_bytes());
            stream.extend_from_slice(&(v as i16).to_be_bytes());
        }
        file.frames.push(Frame {
            fifo_data: stream,
            fifo_start: 0,
            fifo_end: 0,
            memory_updates: Vec::new(),
        });

        let path = std::path::Path::new("/tmp/opencode/present_normal_uv.dff");
        file.save(path).expect("save");
        let loaded = DffFile::load(path).expect("load");

        let mut replay = Replay::new(&loaded.bp_mem, &loaded.cp_mem);
        replay.frame(&loaded.frames[0]).expect("replay");
        let presented = present(&loaded, &replay).expect("present");

        let group = &presented.groups[0];
        assert_eq!(
            group.normals.len(),
            group.positions.len(),
            "one normal per position"
        );
        assert_eq!(group.normals[0], [0.0, 0.0, 1.0]);
        assert_eq!(
            group.uvs,
            vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]
        );
    }
}
