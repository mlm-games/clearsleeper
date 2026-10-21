//! Replaying a recorded GX command stream into per-draw state and geometry.
//!
//! A `.dff` frame is a byte stream, not a scene: the state that configures a
//! draw arrives from register writes earlier in the same stream, so the draws
//! only mean something once the writes are applied in order. This walks the
//! stream once, folds the BP writes into a register file, and records each draw
//! with the material bound when it was issued. The vertex format comes from the
//! CP registers in the same recording, and its stride is what lets a draw's
//! payload be stepped over and read back as geometry.
//!
//! An attribute the vertex descriptor marks `INDEX8`/`INDEX16` carries an index
//! instead of a value, and the value lives in one of the CP's sixteen arrays:
//! `base + index * stride`, in the type and width the same attribute would take
//! direct (`VertexLoader_Position::Pos_ReadIndex` and its siblings address their
//! array as `cached_arraybases[array] + index * array_strides[array]`,
//! `dolphin/Source/Core/VideoCommon/VertexLoader_Position.cpp`). A `.dff` does
//! not carry main memory, only the writes the guest made outside the command
//! stream, so a replay writes those into a recorded-memory image as the walk
//! reaches their FIFO offsets and reads the arrays out of it. A value no
//! recorded write covers is a typed [`ReplayGap`], never a zero.
//!
//! Vertex payloads are the bytes the big-endian guest wrote, so every
//! multi-byte component reads big-endian, which is how gecko reads the same
//! stream (`gecko/crates/gecko/src/flipper/gx/vertex.rs:765`) and how
//! `crate::j3d::bdl` reads the vertex arrays a `bdl4` carries. An array row
//! reads big-endian the same way: the loaders read each component with
//! `Common::FromBigEndian` over the array bytes
//! (`dolphin/Source/Core/VideoCommon/VertexLoaderUtils.h`).

use std::fmt;

use super::fifo::{self, FifoEvent};
use super::vtxfmt::{Element, VertexFormat, component_size};
use crate::dff::{Frame, MemoryUpdate};
use crate::gx_decode::{BpState, GxDecodeError, TexDescriptor};
use repame_view3d::{ShadingError, ShadingModel};

/// Attribute numbers, `GX_VA_*` (`tww/include/dolphin/gx/GXEnum.h:198`).
const GX_VA_POS: u8 = 0x09;
const GX_VA_NRM: u8 = 0x0A;
const GX_VA_CLR0: u8 = 0x0B;
const GX_VA_TEX0: u8 = 0x0D;

/// `GXCompType` codes (`GXEnum.h:247`). A fixed-point value scales by
/// `1 / 2^frac`, with the fraction bits the vertex format carries; `f32` reads
/// as stored.
const GX_U8: u32 = 0;
const GX_S8: u32 = 1;
const GX_U16: u32 = 2;
const GX_S16: u32 = 3;
const GX_F32: u32 = 4;
/// Codes 5..7 have no `GXCompType` name. Dolphin reads them as `f32`: they are
/// "assumed to behave the same as float"
/// (`ComponentFormat::InvalidFloat5..7`,
/// `dolphin/Source/Core/VideoCommon/CPMemory.h`), every loader table binds them
/// to its float reader (`VertexLoader_Position.cpp`, `VertexLoader_Normal.cpp`,
/// `VertexLoader_TextCoord.cpp`) and `GetElementSize` sizes each as four bytes,
/// which is the reading this decoder takes.
const GX_FLOAT5: u32 = 5;
const GX_FLOAT6: u32 = 6;
const GX_FLOAT7: u32 = 7;

/// The two `INDEX*` entry kinds: the value lives in an array the element's
/// index names (`GX_INDEX8`/`GX_INDEX16`, `GXEnum.h:227`).
const GX_INDEX8: u8 = 2;
const GX_INDEX16: u8 = 3;

/// CP register runs the arrays live in: `GX_CP_REG_ARRAYBASE = 0xA0` and
/// `GX_CP_REG_ARRAYSTRIDE = 0xB0` (`tww/include/dolphin/gx/GXEnum.h:955-956`),
/// one register per array slot. `GXSetArray` writes them as
/// `cpAttr | 0xA0` and `cpAttr | 0xB0` with `cpAttr = attr - GX_VA_POS`
/// (`tww/src/dolphin/gx/GXAttr.c:478-479`), which for slots 0..15 is the same
/// as adding.
const CP_ARRAY_BASE: usize = 0xA0;
const CP_ARRAY_STRIDE: usize = 0xB0;
/// The address bits a CP array base keeps: `GXSetArray` masks the pointer with
/// `0x3FFFFFFF` before writing it (`tww/src/dolphin/gx/GXAttr.c:473`), and
/// Dolphin reads the same physical address back out of its CP state
/// (`CPMemory.cpp`, `array_bases[...] = value & GetPhysicalAddressMask(...)`).
const ARRAY_BASE_MASK: u32 = 0x3FFF_FFFF;
/// An array stride is one byte: `array_strides[...] = value & 0xFF`
/// (`dolphin/Source/Core/VideoCommon/CPMemory.cpp`).
const ARRAY_STRIDE_MASK: u32 = 0xFF;
/// Bytes the largest attribute value takes: nine `f32` normal components, a
/// `GX_NRM_NBT` set with `nrmidx3`. [`Element::value_size`] sizes every value,
/// and the `VAT` fields cannot produce more.
const VALUE_MAX: usize = 36;

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
    /// One unit normal per position when the draw's format carries a normal,
    /// empty when it carries none the decoder models.
    pub normals: Vec<[f32; 3]>,
    /// One unit binormal per position when the draw's format carries a
    /// `GX_NRM_NBT` set, empty otherwise. The stream orders the set normal,
    /// binormal, tangent (`tww/include/dolphin/gx/GXEnum.h:234`).
    pub binormals: Vec<[f32; 3]>,
    /// One unit tangent per position for that same set.
    pub tangents: Vec<[f32; 3]>,
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

/// Something a draw asked for that the replay could not read.
///
/// Every one names the single thing the recording does not carry, and the
/// attribute it belongs to is left out or its vertex skipped instead: a gap
/// never becomes a zeroed, partial or invented value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayGap {
    /// The format reads an attribute through an index and no array is declared
    /// for it: the CP file has base and stride both zero, the state `GXInit`
    /// leaves every slot in (`tww/src/dolphin/gx/GXInit.c:272-274` clears them
    /// with `GXSetArray((GXAttr)i, gx, 0)`).
    MissingArray { draw: usize, attr: u8 },
    /// An index that resolves to bytes no recorded write covers. `address` is
    /// what `base + index * stride` resolved to.
    ArrayValue {
        draw: usize,
        attr: u8,
        index: u32,
        address: u32,
    },
    /// A `VAT` type code, or an element shape, with no reading in this decoder.
    Unsupported { draw: usize, attr: u8, ctype: u32 },
    /// A `LOAD_INDX_A..D` load of array `array` whose bytes no recorded write
    /// covers: the CP reads `count` dwords at `base + index * stride` for the
    /// transform unit (`FifoRecorder::OnIndexedLoad`,
    /// `dolphin/Source/Core/Core/FifoPlayer/FifoRecorder.cpp`).
    IndexedXf { array: u8, index: u32, address: u32 },
}

impl fmt::Display for ReplayGap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::MissingArray { draw, attr } => write!(
                f,
                "draw {draw} attribute {attr:#04x} is indexed with no declared array"
            ),
            Self::ArrayValue {
                draw,
                attr,
                index,
                address,
            } => write!(
                f,
                "draw {draw} attribute {attr:#04x} index {index} names {address:#010x}, \
                 which no recorded write covers"
            ),
            Self::Unsupported { draw, attr, ctype } => write!(
                f,
                "draw {draw} attribute {attr:#04x} has no reading for type {ctype}"
            ),
            Self::IndexedXf {
                array,
                index,
                address,
            } => write!(
                f,
                "indexed xf load of array {array} index {index} names {address:#010x}, \
                 which no recorded write covers"
            ),
        }
    }
}

/// Why one attribute of one vertex has no value to read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Miss {
    /// The element is not in the payload, which is shorter than the draw's own
    /// vertex count claims.
    Absent,
    /// An `INDEX8`/`INDEX16` position whose index is all ones: the position
    /// loader marks the vertex skipped and reads nothing for it
    /// (`Pos_ReadIndex`, `m_vertexSkip = index == std::numeric_limits<I>::max()`,
    /// `dolphin/Source/Core/VideoCommon/VertexLoader_Position.cpp`), which is
    /// why the recorder leaves those indices out of the array it saves
    /// (`FifoRecorder::ProcessVertexComponent`, "0xff skips the vertex").
    Skipped,
    /// An indexed element whose array slot nothing declared: base and stride
    /// are both still the zero `GXInit` left.
    MissingArray,
    /// An index that resolves to bytes no recorded write covers.
    ArrayValue { index: u32, address: u32 },
    /// A type code, or an element shape, with no reading in this decoder.
    Unsupported { ctype: u32 },
}

/// The gap a miss becomes for one draw's attribute, where it is one: a payload
/// that ran out and a skipped position are not gaps in the recording.
fn replay_gap(miss: Miss, draw: usize, attr: u8) -> Option<ReplayGap> {
    match miss {
        Miss::Absent | Miss::Skipped => None,
        Miss::MissingArray => Some(ReplayGap::MissingArray { draw, attr }),
        Miss::ArrayValue { index, address } => Some(ReplayGap::ArrayValue {
            draw,
            attr,
            index,
            address,
        }),
        Miss::Unsupported { ctype } => Some(ReplayGap::Unsupported { draw, attr, ctype }),
    }
}

/// The main-memory bytes a recording carries: one run per recorded write, in
/// the order the frames wrote them, so a later run shadows an earlier one.
///
/// A `.dff` does not store main memory. What it stores is the writes the guest
/// made outside the command stream, each with the FIFO offset it landed at
/// ([`MemoryUpdate`]), which is what `FifoRecorder::UseMemory` recorded to keep
/// a replay self-contained
/// (`dolphin/Source/Core/Core/FifoPlayer/FifoRecorder.cpp`); a fifolog player
/// writes them back at their positions
/// (`gecko/crates/fifoplayer/src/playback.rs:136`). Only the recorded bytes
/// exist: a read any byte of which was never written is a hole the replay
/// reports rather than a zero.
#[derive(Default)]
struct RecordedMemory {
    runs: Vec<(u32, Vec<u8>)>,
}

impl RecordedMemory {
    fn write(&mut self, address: u32, data: &[u8]) {
        self.runs.push((address, data.to_vec()));
    }

    /// Copy `out.len()` bytes from `address`, the newest write of each byte
    /// winning. `false` when any byte was never written.
    fn read(&self, address: u32, out: &mut [u8]) -> bool {
        for (offset, byte) in out.iter_mut().enumerate() {
            let Some(address) = address.checked_add(offset as u32) else {
                return false;
            };
            let Some(value) =
                self.runs.iter().rev().find_map(|(start, data)| {
                    data.get(address.checked_sub(*start)? as usize).copied()
                })
            else {
                return false;
            };
            *byte = value;
        }
        true
    }
}

/// The recorded writes of the frame being walked, in fifo order.
struct RecordedWrites<'a> {
    updates: Vec<(usize, &'a MemoryUpdate)>,
    next: usize,
}

impl<'a> RecordedWrites<'a> {
    /// Sort the frame's writes by their FIFO offset, stably, so two writes at
    /// one offset keep the file's order, which is the order the fifolog player
    /// applies them in (`gecko/crates/fifoplayer/src/main.rs:83`).
    fn new(frame: &'a Frame) -> Self {
        let mut updates: Vec<(usize, &MemoryUpdate)> = frame
            .memory_updates
            .iter()
            .map(|update| (update.fifo_position as usize, update))
            .collect();
        updates.sort_by_key(|(position, _)| *position);
        Self { updates, next: 0 }
    }

    /// Write every recorded write the stream has reached by byte offset `at`.
    /// A write whose offset equals a command's own offset lands before that
    /// command: the recorder analyses a command before appending it, so the
    /// array a draw reads carries the draw's own offset
    /// (`FifoRecorder::WriteGPCommand`).
    fn through(&mut self, memory: &mut RecordedMemory, at: usize) {
        while let Some((position, update)) = self.updates.get(self.next).copied() {
            if position > at {
                break;
            }
            memory.write(update.address, &update.data);
            self.next += 1;
        }
    }

    /// Write everything left: those land after the frame's last command, and
    /// the recording's later frames read them, exactly as the fifolog player
    /// applies every update of a frame.
    fn rest(&mut self, memory: &mut RecordedMemory) {
        for (_, update) in &self.updates[self.next..] {
            memory.write(update.address, &update.data);
        }
        self.next = self.updates.len();
    }
}

/// The register files a frame runs against, plus the state derived from them.
pub struct Replay {
    bp: BpState,
    /// The CP register file, which is where the vertex format lives. Kept live
    /// so a draw reads the format in force when it was issued, and so a draw's
    /// indexed attributes read the arrays declared when it was issued.
    cp: Vec<u32>,
    /// The recorded main memory, which is where an indexed attribute's value
    /// and an `INDX_A..D` load's dwords live.
    ram: RecordedMemory,
    materials: Vec<ShadingModel>,
    textures: Vec<TexDescriptor>,
    draws: Vec<DrawCall>,
    geometry: Vec<DrawGeometry>,
    gaps: Vec<ReplayGap>,
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
            ram: RecordedMemory::default(),
            materials: Vec::new(),
            textures: Vec::new(),
            draws: Vec::new(),
            geometry: Vec::new(),
            gaps: Vec::new(),
        }
    }

    /// Walk one frame's stream in order, applying register writes and the
    /// frame's recorded memory writes, recording each draw with the material
    /// then bound, and reading back its geometry.
    pub fn frame(&mut self, frame: &Frame) -> Result<(), ReplayError> {
        // The vertex format in force at the start of the frame gives the stride
        // for the whole walk. A stream that changes it mid-frame would need a
        // second pass, which this does not do.
        let stride = usize::from(VertexFormat::from_cp(&self.cp).stride);
        let events =
            fifo::decode_with_stride(&frame.fifo_data, Some(stride)).map_err(ReplayError::Fifo)?;

        let mut recorded = RecordedWrites::new(frame);
        let mut pending: Option<(usize, u16)> = None;
        for event in &events {
            recorded.through(&mut self.ram, event.at());
            match *event {
                FifoEvent::Bp { id, value, .. } => self.write_bp(id, value),
                FifoEvent::Cp { id, value, .. } => self.write_cp(id, value),
                FifoEvent::Indx {
                    command,
                    index,
                    count,
                    ..
                } => self.indexed_xf_load(command, index, count),
                FifoEvent::Draw {
                    command,
                    vertices,
                    at,
                    ..
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
        recorded.rest(&mut self.ram);
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

    /// One CP register write. The register file is where the vertex format
    /// (`GX_CP_REG_VCD_LO`..`VAT_GRP2`, 0x50..0x9F) and the sixteen arrays
    /// (`GX_CP_REG_ARRAYBASE`/`ARRAYSTRIDE`, 0xA0..0xBF) are declared, and both
    /// are read back out of it at the draw.
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
        Ok(
            match self.materials.iter().position(|known| *known == model) {
                Some(slot) => slot,
                None => {
                    self.materials.push(model);
                    self.materials.len() - 1
                }
            },
        )
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
    /// force. The payload sits directly after the draw command. Every attribute
    /// is read at the vertex its position came from, so a vertex the position
    /// read skips skips its other attributes too. An attribute the draw cannot
    /// read at all (a missing array, an index the recording does not cover, a
    /// type this decoder has no reading for) is left out and reported as one
    /// [`ReplayGap`] for the draw.
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
        let draw = self.draws.len().saturating_sub(1);
        let mut positions = Vec::with_capacity(usize::from(vertices));
        let mut read = Vec::with_capacity(usize::from(vertices));
        let mut position_gap = false;
        for vertex in 0..usize::from(vertices) {
            let position = position_at(&self.cp, &format, payload, vertex, &self.ram);
            match position {
                Ok(xyz) => {
                    positions.push(xyz);
                    read.push(vertex);
                }
                // The payload holds no such vertex, or its index is the skip
                // mark: neither is a gap in the recording.
                Err(Miss::Absent | Miss::Skipped) => {}
                Err(miss) => {
                    // One report per draw: the later vertices of a draw whose
                    // position cannot be read name the same array.
                    if !position_gap {
                        position_gap = true;
                        self.push_gap(miss, draw, GX_VA_POS);
                    }
                }
            }
        }
        let colors = decode_colors(&self.cp, &format, payload, &read, &self.ram);
        let colors = self.decoded(colors, draw, GX_VA_CLR0);
        let normals = match decode_normals(&self.cp, &format, payload, &read, &self.ram) {
            Ok(normals) => normals,
            Err(miss) => {
                self.push_gap(miss, draw, GX_VA_NRM);
                DecodedNormals::default()
            }
        };
        let uvs = decode_uvs(&self.cp, &format, payload, &read, &self.ram);
        let uvs = self.decoded(uvs, draw, GX_VA_TEX0);
        let material = self.draws.len().saturating_sub(1);
        self.geometry.push(DrawGeometry {
            indices: triangles(self.last_topology(), positions.len()),
            positions,
            colors,
            normals: normals.normals,
            binormals: normals.binormals,
            tangents: normals.tangents,
            uvs,
            material,
            texture: None,
        });
    }

    /// One attribute's decode, with the gap a miss is reported and the
    /// attribute left out of the draw.
    fn decoded<T>(&mut self, decoded: Result<Vec<T>, Miss>, draw: usize, attr: u8) -> Vec<T> {
        match decoded {
            Ok(values) => values,
            Err(miss) => {
                self.push_gap(miss, draw, attr);
                Vec::new()
            }
        }
    }

    /// Report one gap: kept for the caller to read back, and logged once.
    fn push_gap(&mut self, miss: Miss, draw: usize, attr: u8) {
        if let Some(gap) = replay_gap(miss, draw, attr) {
            log::warn!("dff replay: {gap}");
            self.gaps.push(gap);
        }
    }

    /// One `LOAD_INDX_A..D`: the CP reads `count` dwords of array `XF_A..XF_D`
    /// from `base + index * stride` and writes them to XF registers from the
    /// command's own address on.
    ///
    /// The command byte names the array: `GX_LOAD_INDX_A` is 0x20, and Dolphin
    /// maps it to `CPArray::XF_A` as `cmd / 8 + 8`
    /// (`dolphin/Source/Core/VideoCommon/OpcodeDecoding.h`, "GX_LOAD_INDX_A
    /// (32 = 8*4) . CPArray::XF_A (4+8 = 12)"); the recorder saves exactly
    /// `array_bases[array] + array_strides[array] * index` for `size * sizeof(u32)`
    /// bytes (`FifoRecorder::OnIndexedLoad`). Nothing in this replay reads XF
    /// state (`FifoEvent::Xf` writes are dropped the same way), so what is
    /// resolved here is the array read: a recording that does not carry the
    /// bytes is a gap, one that does is the load the transform unit would have
    /// seen.
    fn indexed_xf_load(&mut self, command: u8, index: u16, count: u8) {
        let slot = command / 8 + 8;
        let (base, stride) = array(&self.cp, slot).unwrap_or((0, 0));
        let address = base.wrapping_add(u32::from(index).wrapping_mul(stride));
        let mut bytes = [0_u8; 16 * 4];
        let len = usize::from(count) * 4;
        if !self.ram.read(address, &mut bytes[..len]) {
            let gap = ReplayGap::IndexedXf {
                array: slot,
                index: u32::from(index),
                address,
            };
            log::warn!("dff replay: {gap}");
            self.gaps.push(gap);
        }
    }

    fn last_topology(&self) -> u8 {
        self.draws
            .last()
            .map(|d| d.command)
            .unwrap_or(TOPOLOGY_TRIANGLES)
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

    /// Everything the replay could not read, in the order the draws named it.
    pub fn gaps(&self) -> &[ReplayGap] {
        &self.gaps
    }
}

/// One array slot's base and stride, out of the CP register file: base at
/// `GX_CP_REG_ARRAYBASE + slot`, stride at `GX_CP_REG_ARRAYSTRIDE + slot`.
/// `None` while both are still zero, the state `GXInit` leaves every slot in
/// (`tww/src/dolphin/gx/GXInit.c:272-274` clears them with
/// `GXSetArray((GXAttr)i, gx, 0)`), so a draw that never declared its array
/// names address zero at zero stride.
fn array(cp: &[u32], slot: u8) -> Option<(u32, u32)> {
    let base = cp
        .get(CP_ARRAY_BASE + usize::from(slot))
        .copied()
        .unwrap_or(0)
        & ARRAY_BASE_MASK;
    let stride = cp
        .get(CP_ARRAY_STRIDE + usize::from(slot))
        .copied()
        .unwrap_or(0)
        & ARRAY_STRIDE_MASK;
    (base != 0 || stride != 0).then_some((base, stride))
}

/// The index an `INDEX8`/`INDEX16` element carries, read big-endian out of the
/// element's own bytes: every loader's `*_ReadIndex` takes the index with
/// `DataRead<I>` and addresses its array with
/// `cached_arraybases[array] + index * array_strides[array]`
/// (`dolphin/Source/Core/VideoCommon/VertexLoader_Position.cpp`,
/// `VertexLoader_Normal.cpp`, `VertexLoader_TextCoord.cpp`,
/// `VertexLoader_Color.cpp`). `None` when the element is not an index or has no
/// room for one.
fn index_of(bytes: &[u8], kind: u8) -> Option<u32> {
    match kind {
        GX_INDEX8 => Some(u32::from(*bytes.first()?)),
        GX_INDEX16 => Some(u32::from(u16::from_be_bytes([
            *bytes.first()?,
            *bytes.get(1)?,
        ]))),
        _ => None,
    }
}

/// The index that marks a skipped vertex: `0xFF`/`0xFFFF`
/// (`Pos_ReadIndex`, `m_vertexSkip = index == std::numeric_limits<I>::max()`).
fn skip_index(kind: u8) -> u32 {
    if kind == GX_INDEX8 { 0xFF } else { 0xFFFF }
}

/// One attribute value's bytes for one vertex: the payload's own for a direct
/// element, or the bytes the vertex's index names, copied out of the recorded
/// memory into `scratch` (an array row sits nowhere near the payload). `Err` is
/// the typed reason there are none.
fn value_bytes<'a>(
    cp: &[u32],
    format: &VertexFormat,
    attr: u8,
    payload: &'a [u8],
    vertex: usize,
    memory: &RecordedMemory,
    scratch: &'a mut [u8; VALUE_MAX],
) -> Result<&'a [u8], Miss> {
    let Some(element) = format.element(attr) else {
        return Err(Miss::Absent);
    };
    let Some(raw) = format.read(attr, payload, vertex) else {
        return Err(Miss::Absent);
    };
    if !element.is_indexed() {
        return Ok(raw);
    }
    let Some(index) = index_of(raw, element.kind) else {
        return Err(Miss::Absent);
    };
    let Some(slot) = element.array_slot() else {
        return Err(Miss::Unsupported {
            ctype: element.ctype,
        });
    };
    let Some((base, stride)) = array(cp, slot) else {
        return Err(Miss::MissingArray);
    };
    let address = base.wrapping_add(index.wrapping_mul(stride));
    let size = usize::from(element.value_size());
    let Some(out) = scratch.get_mut(..size) else {
        return Err(Miss::Unsupported {
            ctype: element.ctype,
        });
    };
    if !memory.read(address, out) {
        return Err(Miss::ArrayValue { index, address });
    }
    Ok(out)
}

/// One component of an attribute value by its component index, in the stream's
/// byte order and scaled: a fixed-point type by `raw / 2^frac`, `f32` (and the
/// assumed-float codes 5..7) as stored
/// (`dolphin/Source/Core/VideoCommon/VertexLoaderUtils.h` reads every component
/// big-endian; `TCScale<float>`, `PosScale<float>` and `FracAdjust<float>`
/// ignore the fraction).
fn component_at(bytes: &[u8], index: usize, ctype: u32, scale: f32) -> Option<f32> {
    let width = usize::from(component_size(ctype));
    if width == 0 {
        return None;
    }
    let at = index * width;
    Some(match ctype {
        GX_U8 => f32::from(*bytes.get(at)?) * scale,
        GX_S8 => f32::from(*bytes.get(at)? as i8) * scale,
        GX_U16 => f32::from(be_u16(bytes, at)?) * scale,
        GX_S16 => f32::from(be_i16(bytes, at)?) * scale,
        GX_F32 | GX_FLOAT5 | GX_FLOAT6 | GX_FLOAT7 => be_f32(bytes, at)?,
        _ => return None,
    })
}

/// `count` components of an attribute value, straight from the bytes that
/// carry them.
fn value_components<const N: usize>(bytes: &[u8], ctype: u32, scale: f32) -> Option<[f32; N]> {
    let mut out = [0.0_f32; N];
    for (index, value) in out.iter_mut().enumerate() {
        *value = component_at(bytes, index, ctype, scale)?;
    }
    Some(out)
}

/// The `raw / 2^frac` a fixed-point value scales by. A float ignores it.
fn fixed_scale(frac: u8) -> f32 {
    1.0 / ((1_u32 << frac) as f32)
}

/// One vertex's position in three components, wherever its format keeps them.
///
/// A `GX_POS_XY` position carries x and y only. The SDK's `GXPosition2*`
/// writers send exactly two values (`tww/include/dolphin/gx/GX.h:129-146`), and
/// the hardware feeds z = 0 for the missing component, which is how Dolphin's
/// vertex shader builds the position: `float4 rawpos = float4(i.pos0, i.pos1,
/// 0.0f, 1.0f)` (`dolphin/Source/Core/VideoCommon/VertexShaderGen.cpp`). The
/// all-ones index is the position loader's skip mark: the vertex is skipped,
/// not a gap.
fn position_at(
    cp: &[u32],
    format: &VertexFormat,
    payload: &[u8],
    vertex: usize,
    memory: &RecordedMemory,
) -> Result<[f32; 3], Miss> {
    let Some(element) = format.element(GX_VA_POS) else {
        return Err(Miss::Absent);
    };
    if element.is_indexed() {
        let raw = format
            .read(GX_VA_POS, payload, vertex)
            .ok_or(Miss::Absent)?;
        if index_of(raw, element.kind) == Some(skip_index(element.kind)) {
            return Err(Miss::Skipped);
        }
    }
    let components = element.components();
    if components != 2 && components != 3 {
        return Err(Miss::Unsupported {
            ctype: element.ctype,
        });
    }
    let mut scratch = [0_u8; VALUE_MAX];
    let bytes = value_bytes(cp, format, GX_VA_POS, payload, vertex, memory, &mut scratch)?;
    if components == 2 {
        let [x, y] = value_components::<2>(bytes, element.ctype, fixed_scale(element.frac)).ok_or(
            Miss::Unsupported {
                ctype: element.ctype,
            },
        )?;
        return Ok([x, y, 0.0]);
    }
    value_components::<3>(bytes, element.ctype, fixed_scale(element.frac)).ok_or(
        Miss::Unsupported {
            ctype: element.ctype,
        },
    )
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
                    indices.extend_from_slice(&[
                        vertex as u32 - 2,
                        vertex as u32 - 1,
                        vertex as u32,
                    ]);
                } else {
                    indices.extend_from_slice(&[
                        vertex as u32 - 1,
                        vertex as u32 - 2,
                        vertex as u32,
                    ]);
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

/// Vertex colours, one per position read, from the payload or the array the
/// vertex's index names.
///
/// A format carrying no colour leaves the vertex uncoloured rather than
/// guessing: the byte width and the channel layout of a `GXColorType` are the
/// one in `VertexLoader_Color.cpp` (`Read24`/`Read32` for the direct path,
/// `Color_ReadIndex_*` for the array path, which read the same bytes at
/// `base + index * stride`). An element whose type has no reading here is a
/// miss, so the draw is uncoloured and the gap reported instead of a white or
/// half-decoded colour.
fn decode_colors(
    cp: &[u32],
    format: &VertexFormat,
    payload: &[u8],
    vertices: &[usize],
    memory: &RecordedMemory,
) -> Result<Vec<[f32; 3]>, Miss> {
    let Some(element) = format.element(GX_VA_CLR0) else {
        return Ok(Vec::new());
    };
    let ctype = element.ctype;
    // `VertexLoader_Color::GetFunction` rejects a format past `RGBA8888`; the
    // same ceiling leaves this decoder without a reading.
    if ctype > 5 {
        return Err(Miss::Unsupported { ctype });
    }
    let mut scratch = [0_u8; VALUE_MAX];
    let mut colors = Vec::with_capacity(vertices.len());
    for &vertex in vertices {
        let bytes = value_bytes(
            cp,
            format,
            GX_VA_CLR0,
            payload,
            vertex,
            memory,
            &mut scratch,
        )?;
        let Some(color) = color_from_bytes(bytes, ctype) else {
            return Err(Miss::Absent);
        };
        colors.push(color);
    }
    Ok(colors)
}

/// One vertex colour's three channels, straight from the bytes that carry them,
/// in the stored `GXColorType` (`GXEnum.h:254`). The packing is
/// `VertexLoader_Color.cpp`: `Read24` for the three-byte formats, and
/// `SetCol565`/`SetCol4444`/`SetCol6666` for the packed ones, each of which
/// spreads a channel across a byte with its top bits repeated. `None` when the
/// type is not one of them or the slice is short.
fn color_from_bytes(bytes: &[u8], ctype: u32) -> Option<[f32; 3]> {
    Some(match ctype {
        // `RGB8`/`RGBX8`/`RGBA8`: a three-byte channel, one byte each
        // (`Read24`; the fourth byte of the 32-bit formats is not a channel
        // this geometry carries).
        1 | 2 | 5 => [
            f32::from(*bytes.first()?) / 255.0,
            f32::from(*bytes.get(1)?) / 255.0,
            f32::from(*bytes.get(2)?) / 255.0,
        ],
        // `RGBA4`: four bits a channel in one halfword, blue in the high
        // nibble (`GXColorType`'s 16-bit order is BGRa).
        3 => {
            let word = be_u16(bytes, 0)?;
            [
                f32::from((word >> 12) & 0xF) / 15.0,
                f32::from((word >> 8) & 0xF) / 15.0,
                f32::from((word >> 4) & 0xF) / 15.0,
            ]
        }
        // `RGB565`: five red, six green and five blue bits in one halfword.
        0 => {
            let word = be_u16(bytes, 0)?;
            [
                f32::from((word >> 11) & 0x1F) / 31.0,
                f32::from((word >> 5) & 0x3F) / 63.0,
                f32::from(word & 0x1F) / 31.0,
            ]
        }
        // `RGBA6`: six bits a channel over three bytes, red in the high one.
        4 => {
            let packed = u32::from_be_bytes([0, *bytes.first()?, *bytes.get(1)?, *bytes.get(2)?]);
            let channel = |shift: u32| {
                let six = ((packed >> shift) & 0x3F) as u8;
                f32::from((six << 2) | (six >> 4)) / 255.0
            };
            [channel(18), channel(12), channel(6)]
        }
        _ => return None,
    })
}

/// Texture coordinates, one per position read, from the payload or the array
/// the vertex's index names.
///
/// A component scales as `raw / 2^frac`, the fraction bits the vertex format
/// carries in `Element::frac` (`VAT` texture shift); `f32` reads as stored
/// (`VertexLoader::VertexLoader` builds `m_tcScale[i] = 1 / (1 << TexFrac)`
/// and `TCScale` applies it to every integer type and to no float,
/// `dolphin/Source/Core/VideoCommon/VertexLoader.cpp`,
/// `VertexLoader_TextCoord.cpp`).
///
/// The hardware has no T input for a one-component texture coordinate: the
/// missing component is zero, which is how Dolphin fills it
/// (`dolphin/Source/Core/VideoCommon/VertexShaderGen.cpp`, "case 1: float3
/// rawtex = float3(i.tex, 0.0f, 0.0f)"). Only texture coordinate set 0 is
/// read: the draw path carries one set, and the `TevOrder` texmap that would
/// pick another per stage is not modelled yet.
fn decode_uvs(
    cp: &[u32],
    format: &VertexFormat,
    payload: &[u8],
    vertices: &[usize],
    memory: &RecordedMemory,
) -> Result<Vec<[f32; 2]>, Miss> {
    let Some(element) = format.element(GX_VA_TEX0) else {
        return Ok(Vec::new());
    };
    let components = element.components();
    if components != 1 && components != 2 {
        return Err(Miss::Unsupported {
            ctype: element.ctype,
        });
    }
    let scale = fixed_scale(element.frac);
    let mut scratch = [0_u8; VALUE_MAX];
    let mut uvs = Vec::with_capacity(vertices.len());
    for &vertex in vertices {
        let bytes = value_bytes(
            cp,
            format,
            GX_VA_TEX0,
            payload,
            vertex,
            memory,
            &mut scratch,
        )?;
        let uv = if components == 1 {
            value_components::<1>(bytes, element.ctype, scale).map(|[s]| [s, 0.0])
        } else {
            value_components::<2>(bytes, element.ctype, scale)
        };
        let Some(uv) = uv else {
            return Err(Miss::Unsupported {
                ctype: element.ctype,
            });
        };
        uvs.push(uv);
    }
    Ok(uvs)
}

/// The vectors a normal attribute carries: a `GX_NRM_XYZ` set fills only
/// `normals`, the single-normal path; a `GX_NRM_NBT` set fills all three in the
/// stream's order, normal, binormal and tangent
/// (`tww/include/dolphin/gx/GXEnum.h:234`).
#[derive(Default)]
struct DecodedNormals {
    normals: Vec<[f32; 3]>,
    binormals: Vec<[f32; 3]>,
    tangents: Vec<[f32; 3]>,
}

/// Normals, one vector per position read, from the payload or the array the
/// vertex's index names.
///
/// A component scales as `raw / 2^frac` with the fraction its type carries
/// (`Element::frac`: `FracAdjust` in
/// `dolphin/Source/Core/VideoCommon/VertexLoader_Normal.cpp`); `f32` reads as
/// stored. A `GX_NRM_NBT` value is nine components, three vectors in the order
/// normal, binormal, tangent (`tww/include/dolphin/gx/GXEnum.h:234`). With
/// `nrmidx3` the element carries one index per vector and each vector comes
/// from its own row at the vector's own triple
/// (`Normal_ReadIndex_Indices3`, `VertexLoader_Normal.cpp`, which reads
/// components 0..3 from the first row, 3..6 from the second and 6..9 from the
/// third). The vectors name directions but need not be unit length on the wire,
/// so each is normalized for the render path, which takes unit vectors only. A
/// degenerate vector has no direction; it leaves the draw without normal data
/// rather than a zeroed or scaled one.
fn decode_normals(
    cp: &[u32],
    format: &VertexFormat,
    payload: &[u8],
    vertices: &[usize],
    memory: &RecordedMemory,
) -> Result<DecodedNormals, Miss> {
    let Some(element) = format.element(GX_VA_NRM) else {
        return Ok(DecodedNormals::default());
    };
    let components = element.components();
    if components != 3 && components != 9 {
        return Err(Miss::Unsupported {
            ctype: element.ctype,
        });
    }
    let scale = fixed_scale(element.frac);
    let index3 = element.index_count() == 3;
    let mut scratch = [0_u8; VALUE_MAX];
    let mut decoded = DecodedNormals {
        normals: Vec::with_capacity(vertices.len()),
        binormals: Vec::with_capacity(if components == 9 { vertices.len() } else { 0 }),
        tangents: Vec::with_capacity(if components == 9 { vertices.len() } else { 0 }),
    };
    for &vertex in vertices {
        let vectors = if index3 {
            let mut vectors = [[0.0_f32; 3]; 3];
            for (vector, out) in vectors.iter_mut().enumerate() {
                *out = index3_vector(
                    cp,
                    format,
                    element,
                    payload,
                    vertex,
                    memory,
                    vector,
                    &mut scratch,
                )?;
            }
            vectors
        } else {
            let bytes = value_bytes(cp, format, GX_VA_NRM, payload, vertex, memory, &mut scratch)?;
            if components == 3 {
                let normal = value_components::<3>(bytes, element.ctype, scale).ok_or(
                    Miss::Unsupported {
                        ctype: element.ctype,
                    },
                )?;
                [normal, [0.0; 3], [0.0; 3]]
            } else {
                let values = value_components::<9>(bytes, element.ctype, scale).ok_or(
                    Miss::Unsupported {
                        ctype: element.ctype,
                    },
                )?;
                [
                    [values[0], values[1], values[2]],
                    [values[3], values[4], values[5]],
                    [values[6], values[7], values[8]],
                ]
            }
        };

        let Some(normal) = unit_normal(vectors[0]) else {
            return Ok(DecodedNormals::default());
        };
        decoded.normals.push(normal);
        if components == 9 {
            let Some(binormal) = unit_normal(vectors[1]) else {
                return Ok(DecodedNormals::default());
            };
            let Some(tangent) = unit_normal(vectors[2]) else {
                return Ok(DecodedNormals::default());
            };
            decoded.binormals.push(binormal);
            decoded.tangents.push(tangent);
        }
    }
    Ok(decoded)
}

/// One vector of an indexed `GX_NRM_NBT` set with `nrmidx3`: the `vector`th
/// index names a row and the vector is that row's `vector`th triple, which is
/// the offset Dolphin's three `Normal_ReadIndex_Offset` calls take
/// (`Normal_ReadIndex_Indices3`, `VertexLoader_Normal.cpp`).
#[allow(clippy::too_many_arguments)]
fn index3_vector(
    cp: &[u32],
    format: &VertexFormat,
    element: &Element,
    payload: &[u8],
    vertex: usize,
    memory: &RecordedMemory,
    vector: usize,
    scratch: &mut [u8; VALUE_MAX],
) -> Result<[f32; 3], Miss> {
    let raw = format
        .read(GX_VA_NRM, payload, vertex)
        .ok_or(Miss::Absent)?;
    let width = if element.kind == GX_INDEX8 { 1 } else { 2 };
    let index = index_of(raw.get(vector * width..).ok_or(Miss::Absent)?, element.kind)
        .ok_or(Miss::Absent)?;
    let Some(slot) = element.array_slot() else {
        return Err(Miss::Unsupported {
            ctype: element.ctype,
        });
    };
    let Some((base, stride)) = array(cp, slot) else {
        return Err(Miss::MissingArray);
    };
    let component = usize::from(component_size(element.ctype));
    if component == 0 {
        return Err(Miss::Unsupported {
            ctype: element.ctype,
        });
    }
    let address = base.wrapping_add(index.wrapping_mul(stride));
    let at = address.wrapping_add((vector * 3 * component) as u32);
    let len = 3 * component;
    let Some(out) = scratch.get_mut(..len) else {
        return Err(Miss::Unsupported {
            ctype: element.ctype,
        });
    };
    if !memory.read(at, out) {
        return Err(Miss::ArrayValue { index, address });
    }
    value_components::<3>(out, element.ctype, fixed_scale(element.frac)).ok_or(Miss::Unsupported {
        ctype: element.ctype,
    })
}

/// A unit-length normal, or `None` for one with no length or no finite
/// components: the render path takes unit normals, and there is no direction to
/// present for those.
fn unit_normal(vector: [f32; 3]) -> Option<[f32; 3]> {
    let length = (vector[0] * vector[0] + vector[1] * vector[1] + vector[2] * vector[2]).sqrt();
    if !length.is_finite() || length <= 0.0 {
        return None;
    }
    Some([vector[0] / length, vector[1] / length, vector[2] / length])
}

fn be_u16(bytes: &[u8], at: usize) -> Option<u16> {
    let chunk = bytes.get(at..at + 2)?;
    Some(u16::from_be_bytes([chunk[0], chunk[1]]))
}

fn be_i16(bytes: &[u8], at: usize) -> Option<i16> {
    Some(be_u16(bytes, at)? as i16)
}

fn be_f32(bytes: &[u8], at: usize) -> Option<f32> {
    let chunk = bytes.get(at..at + 4)?;
    Some(f32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
}
