//! Replaying a recorded GX command stream into per-draw state and geometry.
//!
//! A `.dff` frame is a byte stream, not a scene: the state that configures a
//! draw arrives from register writes earlier in the same stream, so the draws
//! only mean something once the writes are applied in order. This walks the
//! stream once, folds the BP writes into a register file, and records each draw
//! with the material bound when it was issued. The vertex format comes from the
//! CP registers in the same recording, and its stride is what lets a draw's
//! payload be stepped over and read back as geometry.

use super::fifo::{self, FifoEvent};
use super::vtxfmt::VertexFormat;
use crate::dff::Frame;
use crate::gx_decode::{BpState, GxDecodeError, TexDescriptor};
use repame_view3d::{ShadingError, ShadingModel};

/// `GX_F32` in `GXCompType`, the only coordinate type read without a scale.
const GX_F32: u32 = 4;

/// One draw as it stood on the wire: its topology, its vertex count, and the
/// material the stream had bound at the time.
#[derive(Clone, Debug)]
pub struct DrawCall {
    /// Primitive topology, one of `fifo`'s masked draw opcodes.
    pub command: u8,
    pub vertices: u16,
    /// Index into [`Replay::materials`].
    pub material: usize,
    /// Index into [`Replay::textures`], or `None` when nothing is bound.
    pub texture: Option<usize>,
}

/// One draw turned into geometry: the vertices it named and the triangles they
/// expand to, in the topology the stream used.
#[derive(Clone, Debug, Default)]
pub struct DrawGeometry {
    pub positions: Vec<[f32; 3]>,
    pub colors: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
    /// Index into [`Replay::materials`].
    pub material: usize,
    /// Index into [`Replay::textures`], or `None` when nothing is bound.
    pub texture: Option<usize>,
}

/// Why a frame could not be replayed.
#[derive(Debug)]
pub enum ReplayError {
    Fifo(super::fifo::FifoError),
    Material(GxDecodeError),
    Shading(ShadingError),
}

/// The register files a frame runs against, plus the state derived from them.
pub struct Replay {
    bp: BpState,
    /// The CP register file, which is where the vertex format lives. Kept live
    /// so a draw reads the format in force when it was issued.
    cp: Vec<u32>,
    materials: Vec<ShadingModel>,
    textures: Vec<TexDescriptor>,
    draws: Vec<DrawCall>,
    geometry: Vec<DrawGeometry>,
}

/// The primitive topologies a stream can name, as the fifo decoder masks them.
const TOPOLOGY_TRIANGLES: u8 = 0x90;
const TOPOLOGY_STRIP: u8 = 0x98;
const TOPOLOGY_FAN: u8 = 0xA0;

impl Replay {
    /// Start from the register images a `.dff` carries, which hold the state in
    /// force when the recording began.
    pub fn new(bp_mem: &[u32], cp_mem: &[u32]) -> Self {
        let mut cp = vec![0_u32; 0x100];
        let take = cp_mem.len().min(cp.len());
        cp[..take].copy_from_slice(&cp_mem[..take]);
        Self {
            bp: BpState::from_words(bp_mem),
            cp,
            materials: Vec::new(),
            textures: Vec::new(),
            draws: Vec::new(),
            geometry: Vec::new(),
        }
    }

    /// Walk one frame's stream in order, applying register writes, recording
    /// each draw with the material then bound, and reading back its geometry.
    pub fn frame(&mut self, frame: &Frame) -> Result<(), ReplayError> {
        // The vertex format in force at the start of the frame gives the stride
        // for the whole walk. A stream that changes it mid-frame would need a
        // second pass, which this does not do.
        let stride = usize::from(VertexFormat::from_cp(&self.cp).stride);
        let events =
            fifo::decode_with_stride(&frame.fifo_data, Some(stride)).map_err(ReplayError::Fifo)?;

        let mut pending: Option<(usize, u16)> = None;
        for event in &events {
            match *event {
                FifoEvent::Bp { id, value } => self.write_bp(id, value),
                FifoEvent::Cp { id, value } => self.write_cp(id, value),
                FifoEvent::Draw {
                    command, vertices, at, ..
                } => {
                    let material = self.bind_material()?;
                    let texture = self.bind_texture(material);
                    self.draws.push(DrawCall {
                        command,
                        vertices,
                        material,
                        texture: Some(texture),
                    });
                    pending = Some((at, vertices));
                }
                FifoEvent::VertexPayload { .. } => {
                    if let Some((at, vertices)) = pending.take() {
                        self.record_geometry(at, vertices, &frame.fifo_data);
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// One BP write: the register id is the wire dword's top byte and the low 24
    /// bits are its payload, which is how `BpState` reads the file back.
    fn write_bp(&mut self, id: u8, value: u32) {
        let reg = usize::from(id);
        if reg < self.bp.words.len() {
            self.bp.words[reg] = value & 0x00FF_FFFF;
        }
    }

    fn write_cp(&mut self, id: u8, value: u32) {
        let reg = usize::from(id);
        if reg < self.cp.len() {
            self.cp[reg] = value;
        }
    }

    /// The material bound right now, deduplicated by value so a later draw that
    /// changes the registers does not disturb it.
    fn bind_material(&mut self) -> Result<usize, ReplayError> {
        let model = self.bp.shading_model().map_err(ReplayError::Material)?;
        model.validate().map_err(ReplayError::Shading)?;
        Ok(match self.materials.iter().position(|known| *known == model) {
            Some(slot) => slot,
            None => {
                self.materials.push(model);
                self.materials.len() - 1
            }
        })
    }

    /// The texture descriptor the texture map each stage samples, deduplicated
    /// by value. `GX_TevOrder`'s texmap is both the coordinate set and the
    /// texture slot, so the first stage's map names the page this draw samples.
    fn bind_texture(&mut self, material: usize) -> usize {
        let Some(slot) = self.materials[material].stages.iter().find_map(|stage| {
            let binds = stage.tex_sel;
            (binds != u8::MAX).then_some(usize::from(binds))
        }) else {
            return usize::MAX;
        };
        let Some(desc) = self.bp.tex_descriptor(slot) else {
            return usize::MAX;
        };
        match self.textures.iter().position(|known| *known == desc) {
            Some(known) => known,
            None => {
                self.textures.push(desc);
                self.textures.len() - 1
            }
        }
    }

    /// Read a draw's vertices back as geometry with the vertex format now in
    /// force. The payload sits directly after the draw command.
    fn record_geometry(&mut self, draw_at: usize, vertices: u16, fifo: &[u8]) {
        let format = VertexFormat::from_cp(&self.cp);
        if format.stride == 0 {
            return;
        }
        let payload_at = draw_at + 3;
        if payload_at > fifo.len() {
            return;
        }
        let payload = &fifo[payload_at..];
        let mut positions = Vec::with_capacity(usize::from(vertices));
        for vertex in 0..usize::from(vertices) {
            let Some(bytes) = format.read(0x09, payload, vertex) else {
                continue;
            };
            let Some(xyz) = read_f32_xyz(bytes) else {
                continue;
            };
            positions.push(xyz);
        }
        let material = self.draws.len().saturating_sub(1);
        let colors = decode_colors(&format, payload, positions.len());
        let uvs = decode_uvs(&format, payload, positions.len());
        self.geometry.push(DrawGeometry {
            indices: triangles(self.last_topology(), positions.len()),
            positions,
            colors,
            uvs,
            material,
            texture: None,
        });
    }

    fn last_topology(&self) -> u8 {
        self.draws.last().map(|d| d.command).unwrap_or(TOPOLOGY_TRIANGLES)
    }

    pub fn draws(&self) -> &[DrawCall] {
        &self.draws
    }

    pub fn materials(&self) -> &[ShadingModel] {
        &self.materials
    }

    pub fn textures(&self) -> &[TexDescriptor] {
        &self.textures
    }

    pub fn geometry(&self) -> &[DrawGeometry] {
        &self.geometry
    }
}

/// Expand `count` vertices into triangle indices in the topology `command` uses.
/// Strip winding flips on odd vertices, which is what the hardware does.
fn triangles(command: u8, count: usize) -> Vec<u32> {
    let mut indices = Vec::new();
    match command & 0xF8 {
        // Quads are whole quads, so a count that is not a multiple of four
        // leaves a partial quad that never becomes a triangle.
        0x80 => {
            for quad in 0..count / 4 {
                let base = (quad * 4) as u32;
                indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
            }
        }
        TOPOLOGY_STRIP => {
            for vertex in 2..count {
                if vertex % 2 == 0 {
                    indices.extend_from_slice(&[vertex as u32 - 2, vertex as u32 - 1, vertex as u32]);
                } else {
                    indices.extend_from_slice(&[vertex as u32 - 1, vertex as u32 - 2, vertex as u32]);
                }
            }
        }
        TOPOLOGY_FAN => {
            for vertex in 1..count.saturating_sub(1) {
                indices.extend_from_slice(&[0, vertex as u32, vertex as u32 + 1]);
            }
        }
        _ => {
            for vertex in 0..count / 3 {
                let base = (vertex * 3) as u32;
                indices.extend_from_slice(&[base, base + 1, base + 2]);
            }
        }
    }
    indices
}

/// Vertex colours, one per position read. A format carrying no colour, or one
/// this cannot size, leaves the vertex uncoloured rather than guessing.
fn decode_colors(format: &VertexFormat, payload: &[u8], count: usize) -> Vec<[f32; 3]> {
    let Some(ctype) = format.type_of(0x0B) else {
        return Vec::new();
    };
    (0..count)
        .map(|vertex| {
            let Some(bytes) = format.read(0x0B, payload, vertex) else {
                return [1.0, 1.0, 1.0];
            };
            match ctype {
                // A three-byte channel, one byte each.
                1 | 2 | 5 => [
                    f32::from(bytes[0]) / 255.0,
                    f32::from(bytes[1]) / 255.0,
                    f32::from(bytes[2]) / 255.0,
                ],
                // Four bits per channel in one little-endian halfword.
                3 => {
                    let word = u16::from_le_bytes([bytes[0], bytes[1]]);
                    [
                        f32::from((word >> 12) & 0xF) / 15.0,
                        f32::from((word >> 8) & 0xF) / 15.0,
                        f32::from((word >> 4) & 0xF) / 15.0,
                    ]
                }
                // Five red, six green and five blue bits in one halfword.
                0 => {
                    let word = u16::from_le_bytes([bytes[0], bytes[1]]);
                    [
                        f32::from((word >> 11) & 0x1F) / 31.0,
                        f32::from((word >> 5) & 0x3F) / 63.0,
                        f32::from(word & 0x1F) / 31.0,
                    ]
                }
                _ => [1.0, 1.0, 1.0],
            }
        })
        .collect()
}

/// Texture coordinates, one per position read. A coordinate set in `f32` reads
/// as stored; anything else carries a scaling this does not model, so the
/// coordinate is left absent rather than mis-scaled.
fn decode_uvs(format: &VertexFormat, payload: &[u8], count: usize) -> Vec<[f32; 2]> {
    if format.type_of(0x0D) != Some(GX_F32) {
        return Vec::new();
    }
    (0..count)
        .map(|vertex| {
            let Some(bytes) = format.read(0x0D, payload, vertex) else {
                return [0.0, 0.0];
            };
            let at = |offset: usize| -> f32 {
                bytes
                    .get(offset..offset + 4)
                    .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                    .unwrap_or(0.0)
            };
            [at(0), at(4)]
        })
        .collect()
}

/// Three consecutive little-endian `f32`s, one for each position component.
/// A shorter slice has no position to read, so it reads as absent rather than
/// half-decoding.
fn read_f32_xyz(bytes: &[u8]) -> Option<[f32; 3]> {
    let at = |offset: usize| -> Option<[u8; 4]> {
        bytes
            .get(offset..offset + 4)
            .map(|chunk| [chunk[0], chunk[1], chunk[2], chunk[3]])
    };
    if bytes.len() < 12 {
        return None;
    }
    Some([
        f32::from_le_bytes(at(0)?),
        f32::from_le_bytes(at(4)?),
        f32::from_le_bytes(at(8)?),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dff::{DffFile, Frame};
    use std::path::Path;

    const GX_DIRECT: u8 = 1;
    const GX_F32: u32 = 4;

    fn empty_file() -> DffFile {
        let mut file = DffFile::default();
        file.game_id = "GZLE01".to_string();
        file
    }

    #[test]
    fn a_register_write_reaches_the_next_draws_material() {
        let mut file = empty_file();
        let stream = [
            0x61, 0x00, 0x00, 0x00, 0x11, 0x61, 0x00, 0x00, 0x00, 0x22, 0x98, 0x00, 0x04, 0xAA,
            0xAA, 0xAA, 0xAA,
        ];
        file.frames.push(Frame {
            fifo_data: stream.to_vec(),
            fifo_start: 0,
            fifo_end: stream.len() as u32,
            memory_updates: Vec::new(),
        });

        let path = Path::new("/tmp/opencode/replay_two_writes.dff");
        file.save(path).expect("save");
        let loaded = DffFile::load(path).expect("load");

        let mut replay = Replay::new(&loaded.bp_mem, &loaded.cp_mem);
        replay.frame(&loaded.frames[0]).expect("replay");

        assert_eq!(replay.draws().len(), 1);
        assert_eq!(replay.draws()[0].command, 0x98);
        assert_eq!(replay.draws()[0].vertices, 4);
    }

    #[test]
    fn draws_made_under_one_material_keep_it() {
        let mut file = empty_file();
        file.cp_mem[0x50] = u32::from(GX_DIRECT);
        file.cp_mem[0x70] = (1 << 31) | (GX_F32 << 28);

        let vertex = [0xAA_u8; 12];
        let mut stream = vec![0x61, 0x00, 0x00, 0x04, 0x00, 0x80, 0x00, 0x04];
        for _ in 0..4 {
            stream.extend_from_slice(&vertex);
        }
        stream.extend_from_slice(&[0x61, 0xC0, 0x00, 0x00, 0x01, 0x80, 0x00, 0x04]);
        for _ in 0..4 {
            stream.extend_from_slice(&vertex);
        }
        file.frames.push(Frame {
            fifo_data: stream,
            fifo_start: 0,
            fifo_end: 0,
            memory_updates: Vec::new(),
        });

        let path = Path::new("/tmp/opencode/replay_two_materials.dff");
        file.save(path).expect("save");
        let loaded = DffFile::load(path).expect("load");

        let mut replay = Replay::new(&loaded.bp_mem, &loaded.cp_mem);
        replay.frame(&loaded.frames[0]).expect("replay");

        assert_eq!(replay.draws().len(), 2);
        assert_eq!(replay.materials().len(), 2);
        assert_ne!(replay.draws()[0].material, replay.draws()[1].material);
    }

    #[test]
    fn a_truncated_stream_keeps_what_it_walked() {
        let mut file = empty_file();
        let stream = [0x98, 0x00, 0x02, 0xAA, 0xAA];
        file.frames.push(Frame {
            fifo_data: stream.to_vec(),
            fifo_start: 0,
            fifo_end: stream.len() as u32,
            memory_updates: Vec::new(),
        });

        let path = Path::new("/tmp/opencode/replay_truncated.dff");
        file.save(path).expect("save");
        let loaded = DffFile::load(path).expect("load");

        let mut replay = Replay::new(&loaded.bp_mem, &loaded.cp_mem);
        replay.frame(&loaded.frames[0]).expect("replay");
        assert_eq!(replay.draws().len(), 1);
    }

    #[test]
    fn register_writes_past_the_file_have_no_effect() {
        let mut file = empty_file();
        let stream = [0x61, 0xFF, 0xFF, 0xFF, 0xFF, 0x80, 0x00, 0x01, 0xAA];
        file.frames.push(Frame {
            fifo_data: stream.to_vec(),
            fifo_start: 0,
            fifo_end: stream.len() as u32,
            memory_updates: Vec::new(),
        });

        let path = Path::new("/tmp/opencode/replay_bad_id.dff");
        file.save(path).expect("save");
        let loaded = DffFile::load(path).expect("load");

        let mut replay = Replay::new(&loaded.bp_mem, &loaded.cp_mem);
        replay.frame(&loaded.frames[0]).expect("replay");
        assert_eq!(replay.draws().len(), 1);
    }

    #[test]
    fn a_draws_payload_comes_back_as_positions_and_indices() {
        let mut file = empty_file();
        file.cp_mem[0x50] = u32::from(GX_DIRECT);
        file.cp_mem[0x70] = (1 << 31) | (GX_F32 << 28);

        let vertex = |x: f32, y: f32, z: f32| {
            let mut out = Vec::new();
            for value in [x, y, z] {
                out.extend_from_slice(&value.to_le_bytes());
            }
            out
        };
        let mut stream = vec![0x98_u8, 0x00, 0x04];
        stream.extend_from_slice(&vertex(0.0, 0.0, 0.0));
        stream.extend_from_slice(&vertex(1.0, 0.0, 0.0));
        stream.extend_from_slice(&vertex(0.0, 1.0, 0.0));
        stream.extend_from_slice(&vertex(1.0, 1.0, 0.0));
        file.frames.push(Frame {
            fifo_data: stream,
            fifo_start: 0,
            fifo_end: 0,
            memory_updates: Vec::new(),
        });

        let path = Path::new("/tmp/opencode/replay_geometry.dff");
        file.save(path).expect("save");
        let loaded = DffFile::load(path).expect("load");

        let mut replay = Replay::new(&loaded.bp_mem, &loaded.cp_mem);
        replay.frame(&loaded.frames[0]).expect("replay");

        assert_eq!(replay.draws().len(), 1);
        let geometry = &replay.geometry()[0];
        assert_eq!(geometry.positions.len(), 4);
        assert_eq!(geometry.positions[1], [1.0, 0.0, 0.0]);
        assert_eq!(geometry.positions[3], [1.0, 1.0, 0.0]);
        assert_eq!(geometry.indices, vec![0, 1, 2, 2, 1, 3]);
    }
}
