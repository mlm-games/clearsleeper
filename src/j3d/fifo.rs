//! The live GX FIFO command stream: register writes interleaved with draws.
//!
//! [`crate::j3d::bdl`] walks the display list embedded in a `bdl4`, where the vertex
//! format comes from the file descriptor. A FIFO stream carries no descriptor: every
//! command has to be long enough to be read off its own header, and the CP/VAT writes
//! in the same stream are what configure the draws that follow. So this decoder
//! reports byte offsets with every event and stops at the vertex payload, leaving the
//! stride to the caller.
//!
//! `GX_NOP` is skipped rather than reported, because it carries no data and
//! [`crate::j3d::bdl`] already agrees on that.
//!
//! Opcodes are `_GXFifoCmd` and `_GXPrimitive` from the decomp headers; the payload
//! shapes come from the inline writers next to them.

/// One decoded FIFO event, with its byte offset into the stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FifoEvent {
    /// A BP register write: the register id and its 32-bit value. Both halves of
    /// the wire dword are kept — the id is its top byte, so the value is what the
    /// opcode's single `u32` carried.
    Bp { id: u8, value: u32, at: usize },
    /// A CP register write.
    Cp { id: u8, value: u32, at: usize },
    /// An XF register write: first register index and the dwords written.
    Xf {
        first: u16,
        dwords: Vec<u32>,
        at: usize,
    },
    /// An `INDX_A`..`INDX_D` write: the memory index, the first XF register and how
    /// many registers the address word covers. No dwords follow on the wire.
    Indx {
        command: u8,
        index: u16,
        first: u16,
        count: u8,
        at: usize,
    },
    /// A draw call: the primitive topology and the vertex count, plus the offset of
    /// the opcode byte itself.
    Draw {
        command: u8,
        vertices: u16,
        at: usize,
    },
    /// The byte offset of the vertex payload that follows a draw, so the caller can
    /// decode it once the vertex format is known. `len` is how much the stream still
    /// holds at `at`, an upper bound on the payload.
    VertexPayload { at: usize, len: usize },
    /// A `GX_FIFO_CMD_CALL_DL` packet: the nested display list and its length in
    /// bytes, neither of which this decoder follows.
    CallDisplayList { address: u32, len: u32, at: usize },
    /// `GX_FIFO_CMD_INVAL_VTX`, a one-byte packet.
    InvalidateVertexCache { at: usize },
}

impl FifoEvent {
    /// The byte offset of the command this event was read from.
    ///
    /// A recorded memory write at that offset belongs before the command: the
    /// fifo recorder analyses a command, recording the bytes it reads or writes
    /// at the command's own offset, before it appends the command to the frame
    /// (`FifoRecorder::WriteGPCommand`,
    /// `dolphin/Source/Core/Core/FifoPlayer/FifoRecorder.cpp`).
    pub fn at(&self) -> usize {
        match *self {
            Self::Bp { at, .. }
            | Self::Cp { at, .. }
            | Self::Xf { at, .. }
            | Self::Indx { at, .. }
            | Self::Draw { at, .. }
            | Self::VertexPayload { at, .. }
            | Self::CallDisplayList { at, .. }
            | Self::InvalidateVertexCache { at } => at,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FifoError {
    Truncated,
    UnknownCommand(u8),
}

impl FifoError {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Truncated => "fifo stream truncated",
            Self::UnknownCommand(_) => "fifo command unsupported",
        }
    }
}

// include/dolphin/gx/GXEnum.h:1016, repeated as GX_NOP in include/dolphin/gx/GX.h:52
const COMMAND_NOP: u8 = 0x00;
// include/dolphin/gx/GXEnum.h:1018, repeated as GX_LOAD_BP_REG in
// include/dolphin/gd/GDBase.h:17 and include/dolphin/gx/GX.h:51
const COMMAND_LOAD_BP: u8 = 0x61;
// include/dolphin/gx/GXEnum.h:1019, repeated as GX_LOAD_CP_REG in
// include/dolphin/gd/GDBase.h:11
const COMMAND_LOAD_CP: u8 = 0x08;
// include/dolphin/gx/GXEnum.h:1020, repeated as GX_LOAD_XF_REG in
// include/dolphin/gd/GDBase.h:12
const COMMAND_LOAD_XF: u8 = 0x10;
// include/dolphin/gx/GXEnum.h:1022, repeated as GX_LOAD_INDX_A in
// include/dolphin/gd/GDBase.h:13
const COMMAND_LOAD_INDX_A: u8 = 0x20;
// include/dolphin/gx/GXEnum.h:1023, repeated as GX_LOAD_INDX_B in
// include/dolphin/gd/GDBase.h:14
const COMMAND_LOAD_INDX_B: u8 = 0x28;
// include/dolphin/gx/GXEnum.h:1024, repeated as GX_LOAD_INDX_C in
// include/dolphin/gd/GDBase.h:15
const COMMAND_LOAD_INDX_C: u8 = 0x30;
// include/dolphin/gx/GXEnum.h:1025, repeated as GX_LOAD_INDX_D in
// include/dolphin/gd/GDBase.h:16
const COMMAND_LOAD_INDX_D: u8 = 0x38;
// include/dolphin/gx/GXEnum.h:1027
const COMMAND_CALL_DL: u8 = 0x40;
// include/dolphin/gx/GXEnum.h:1028
const COMMAND_INVAL_VTX: u8 = 0x48;

// include/dolphin/gx/GXEnum.h:7
const DRAW_QUADS: u8 = 0x80;
// include/dolphin/gx/GXEnum.h:8
const DRAW_TRIANGLES: u8 = 0x90;
// include/dolphin/gx/GXEnum.h:9
const DRAW_STRIP: u8 = 0x98;
// include/dolphin/gx/GXEnum.h:10
const DRAW_FAN: u8 = 0xA0;
// include/dolphin/gx/GXEnum.h:11
const DRAW_LINES: u8 = 0xA8;
// include/dolphin/gx/GXEnum.h:12
const DRAW_LINE_STRIP: u8 = 0xB0;
// include/dolphin/gx/GXEnum.h:13
const DRAW_POINTS: u8 = 0xB8;
const DRAW_COMMANDS: [u8; 7] = [
    DRAW_QUADS,
    DRAW_TRIANGLES,
    DRAW_STRIP,
    DRAW_FAN,
    DRAW_LINES,
    DRAW_LINE_STRIP,
    DRAW_POINTS,
];

// `GX_VTXFMT0`..`GX_VTXFMT7` are 0..7 (include/dolphin/gx/GXEnum.h:186) and
// `GXBegin` writes `fmt | type` (src/dolphin/gx/GXGeometry.c:75), so the vertex
// format rides in the low three bits of a draw opcode and the primitive tag is the
// rest.
const VERTEX_FORMAT_MASK: u8 = 0x07;
const PRIMITIVE_MASK: u8 = !VERTEX_FORMAT_MASK;
// A BP write is one dword with the register address in bits 24..31
// (src/dolphin/gx/GXGeometry.c:98, src/dolphin/gx/GXFrameBuf.c:634).
const BP_REGISTER_SHIFT: u8 = 24;
// `GX_XF_LOAD_REGS` packs `(len - 1) << 16 | addr` into one big-endian dword
// (include/dolphin/gx/GX.h:99), so the count-1 sits in the high half of the
// two `u16` that follow the opcode.
// `GDWriteXFIndxACmd` packs `(len - 1) << 12 | addr`
// (include/dolphin/gd/GDBase.h:113).
const INDX_COUNT_SHIFT: u16 = 12;
const INDX_FIRST_MASK: u16 = 0x0FFF;
const DWORD: usize = 4;

/// Walk a FIFO byte stream, emitting typed events.
/// `vertex_stride` is the byte size of one vertex. `None` (or 0) means the
/// caller cannot size a payload, so the walk stops at the first draw and reports
/// the rest of the stream as its payload. `Some(n)` lets it skip `vertices * n`
/// and keep going, which is what a multi-draw frame needs.
pub fn decode_with_stride(
    data: &[u8],
    vertex_stride: Option<usize>,
) -> Result<Vec<FifoEvent>, FifoError> {
    let mut events = Vec::new();
    let mut cursor = 0;
    while cursor < data.len() {
        let command = data[cursor];
        let at = cursor;
        cursor += 1;
        match command {
            COMMAND_NOP => {}
            COMMAND_LOAD_BP => {
                let value = be_u32(data, cursor)?;
                cursor += DWORD;
                events.push(FifoEvent::Bp {
                    id: (value >> BP_REGISTER_SHIFT) as u8,
                    value,
                    at,
                });
            }
            COMMAND_LOAD_CP => {
                let id = *data.get(cursor).ok_or(FifoError::Truncated)?;
                cursor += 1;
                let value = be_u32(data, cursor)?;
                cursor += DWORD;
                events.push(FifoEvent::Cp { id, value, at });
            }
            COMMAND_LOAD_XF => {
                let count = usize::from(be_u16(data, cursor)?) + 1;
                cursor += 2;
                let first = be_u16(data, cursor)?;
                cursor += 2;
                let dwords = take_dwords(data, &mut cursor, count)?;
                events.push(FifoEvent::Xf { first, dwords, at });
            }
            COMMAND_LOAD_INDX_A | COMMAND_LOAD_INDX_B | COMMAND_LOAD_INDX_C
            | COMMAND_LOAD_INDX_D => {
                let index = be_u16(data, cursor)?;
                cursor += 2;
                let word = be_u16(data, cursor)?;
                cursor += 2;
                events.push(FifoEvent::Indx {
                    command,
                    index,
                    first: word & INDX_FIRST_MASK,
                    count: ((word >> INDX_COUNT_SHIFT) + 1) as u8,
                    at,
                });
            }
            COMMAND_CALL_DL => {
                let address = be_u32(data, cursor)?;
                cursor += DWORD;
                let len = be_u32(data, cursor)?;
                cursor += DWORD;
                events.push(FifoEvent::CallDisplayList { address, len, at });
            }
            COMMAND_INVAL_VTX => events.push(FifoEvent::InvalidateVertexCache { at }),
            _ => {
                let topology = command & PRIMITIVE_MASK;
                if !DRAW_COMMANDS.contains(&topology) {
                    return Err(FifoError::UnknownCommand(command));
                }
                let vertices = be_u16(data, cursor)?;
                cursor += 2;
                events.push(FifoEvent::Draw {
                    command: topology,
                    vertices,
                    at,
                });
                match vertex_stride.filter(|n| *n > 0) {
                    Some(stride) => {
                        let skip = usize::from(vertices)
                            .checked_mul(stride)
                            .ok_or(FifoError::Truncated)?;
                        let end = cursor.checked_add(skip).ok_or(FifoError::Truncated)?;
                        if end > data.len() {
                            return Err(FifoError::Truncated);
                        }
                        events.push(FifoEvent::VertexPayload {
                            at: cursor,
                            len: skip,
                        });
                        cursor = end;
                    }
                    None => {
                        events.push(FifoEvent::VertexPayload {
                            at: cursor,
                            len: data.len() - cursor,
                        });
                        // Without a stride the walk cannot tell where the next
                        // command begins, and reading vertex bytes as opcodes
                        // would invent commands that were never sent.
                        return Ok(events);
                    }
                }
            }
        }
    }
    Ok(events)
}

fn be_u16(data: &[u8], at: usize) -> Result<u16, FifoError> {
    let bytes: [u8; 2] = data
        .get(at..)
        .and_then(|tail| tail.first_chunk())
        .copied()
        .ok_or(FifoError::Truncated)?;
    Ok(u16::from_be_bytes(bytes))
}

fn be_u32(data: &[u8], at: usize) -> Result<u32, FifoError> {
    let bytes: [u8; 4] = data
        .get(at..)
        .and_then(|tail| tail.first_chunk())
        .copied()
        .ok_or(FifoError::Truncated)?;
    Ok(u32::from_be_bytes(bytes))
}

fn take_dwords(data: &[u8], cursor: &mut usize, count: usize) -> Result<Vec<u32>, FifoError> {
    let start = *cursor;
    let end = start + count * DWORD;
    let bytes = data.get(start..end).ok_or(FifoError::Truncated)?;
    *cursor = end;
    Ok(bytes
        .chunks_exact(DWORD)
        .map(|dword| u32::from_be_bytes([dword[0], dword[1], dword[2], dword[3]]))
        .collect())
}
