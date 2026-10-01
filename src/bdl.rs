const HEADER_LEN: usize = 0x20;
const BLOCK_LEN: usize = 8;
const MAGIC: [u8; 8] = *b"J3D2bdl4";
const FORM_STRIDE: usize = 0x10;
const DESCRIPTOR_STRIDE: usize = 8;
const JOINT_INIT_STRIDE: usize = 0x40;
const SHAPE_INIT_STRIDE: usize = 0x28;
const SHAPE_GROUP_STRIDE: usize = 8;
const POSITION_STRIDE: usize = 12;
const COLOR_STRIDE: usize = 4;
const ATTRIBUTE_TERM: u32 = 0xFF;
const ATTRIBUTE_POSITION: u32 = 9;
const ATTRIBUTE_COLOR0: u32 = 11;
const TYPE_INDEX16: u32 = 3;
const TYPE_F32: u32 = 4;
const TYPE_RGBA8: u32 = 5;
const COMPONENT_ONE: u32 = 1;
const POSITION_FORM: (u32, u32) = (COMPONENT_ONE, TYPE_F32);
const COLOR_FORM: (u32, u32) = (COMPONENT_ONE, TYPE_RGBA8);
const HIERARCHY_END: u16 = 0;
const HIERARCHY_BEGIN_CHILD: u16 = 1;
const HIERARCHY_END_CHILD: u16 = 2;
const HIERARCHY_JOINT: u16 = 0x10;
const HIERARCHY_MATERIAL: u16 = 0x11;
const HIERARCHY_SHAPE: u16 = 0x12;
const PARENT_UNSET: i32 = -1;
const PARENT_ROOT: i32 = -2;
const JOINT_SCALE: usize = 4;
const JOINT_ROTATION: usize = 0x10;
const JOINT_TRANSLATION: usize = 0x18;
const IDENTITY_SCALE: [f32; 3] = [1.0, 1.0, 1.0];
const SHAPE_MATRIX_DIRECT: u8 = 0;
const COMMAND_NOP: u8 = 0x00;
const COMMAND_QUADS: u8 = 0x80;
const COMMAND_TRIANGLES: u8 = 0x90;
const COMMAND_STRIP: u8 = 0x98;
const COMMAND_FAN: u8 = 0xA0;
const COMMAND_LINES: u8 = 0xA8;
const COMMAND_LINE_STRIP: u8 = 0xB0;
const COMMAND_POINTS: u8 = 0xB8;
const WHITE: [f32; 3] = [1.0, 1.0, 1.0];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BdlError {
    Truncated,
    Magic,
    Block,
    Hierarchy,
    Transform,
    Envelope,
    Descriptor,
    Shape,
    Command,
    Range,
}

impl BdlError {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Truncated => "bdl data truncated",
            Self::Magic => "bdl magic mismatch",
            Self::Block => "bdl block table invalid",
            Self::Hierarchy => "bdl joint hierarchy invalid",
            Self::Transform => "bdl joint transform unsupported",
            Self::Envelope => "bdl envelope matrix unsupported",
            Self::Descriptor => "bdl vertex descriptor unsupported",
            Self::Shape => "bdl shape matrix unsupported",
            Self::Command => "bdl display list command unsupported",
            Self::Range => "bdl index out of range",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RoomMesh {
    pub positions: Vec<[f32; 3]>,
    pub colors: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

impl RoomMesh {
    pub fn append(&mut self, mut part: RoomMesh) {
        let base = self.positions.len() as u32;
        self.positions.append(&mut part.positions);
        self.colors.append(&mut part.colors);
        self.indices
            .extend(part.indices.iter().map(|index| index + base));
    }
}

pub fn parse(data: &[u8], origin: [f32; 2], angle: f32) -> Result<RoomMesh, BdlError> {
    let blocks = Blocks::parse(data)?;
    let worlds = read_joints(data, &blocks)?;
    let draw_joints = read_draw_joints(data, &blocks, worlds.len())?;
    let vertices = read_vertex_arrays(data, &blocks)?;
    let (sin, cos) = angle.sin_cos();
    let context = ShapeContext {
        data,
        vertices: &vertices,
        origin,
        sin,
        cos,
    };
    let mut mesh = RoomMesh::default();
    read_shapes(&context, &blocks, &worlds, &draw_joints, &mut mesh)?;
    Ok(mesh)
}

fn at(base: usize, offset: usize) -> Result<usize, BdlError> {
    base.checked_add(offset).ok_or(BdlError::Truncated)
}

fn read_u16(data: &[u8], start: usize) -> Result<u16, BdlError> {
    let end = start.checked_add(2).ok_or(BdlError::Truncated)?;
    let bytes = data.get(start..end).ok_or(BdlError::Truncated)?;
    Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
}

fn read_i16(data: &[u8], start: usize) -> Result<i16, BdlError> {
    let end = start.checked_add(2).ok_or(BdlError::Truncated)?;
    let bytes = data.get(start..end).ok_or(BdlError::Truncated)?;
    Ok(i16::from_be_bytes([bytes[0], bytes[1]]))
}

fn read_u32(data: &[u8], start: usize) -> Result<u32, BdlError> {
    let end = start.checked_add(4).ok_or(BdlError::Truncated)?;
    let bytes = data.get(start..end).ok_or(BdlError::Truncated)?;
    Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn read_f32(data: &[u8], start: usize) -> Result<f32, BdlError> {
    let end = start.checked_add(4).ok_or(BdlError::Truncated)?;
    let bytes = data.get(start..end).ok_or(BdlError::Truncated)?;
    Ok(f32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

struct Blocks {
    entries: Vec<([u8; 4], usize, usize)>,
}

impl Blocks {
    fn parse(data: &[u8]) -> Result<Self, BdlError> {
        if data.len() < HEADER_LEN {
            return Err(BdlError::Truncated);
        }
        if data[..8] != MAGIC {
            return Err(BdlError::Magic);
        }
        let count = read_u32(data, 12)? as usize;
        if count > data.len() / BLOCK_LEN {
            return Err(BdlError::Block);
        }
        let mut entries = Vec::with_capacity(count);
        let mut cursor = HEADER_LEN;
        for _ in 0..count {
            let tag_bytes = data.get(cursor..cursor + 4).ok_or(BdlError::Block)?;
            let mut tag = [0_u8; 4];
            tag.copy_from_slice(tag_bytes);
            let size = read_u32(data, cursor + 4)? as usize;
            if size < BLOCK_LEN {
                return Err(BdlError::Block);
            }
            let end = cursor.checked_add(size).ok_or(BdlError::Block)?;
            if end > data.len() {
                return Err(BdlError::Block);
            }
            entries.push((tag, cursor, end));
            cursor = end;
        }
        Ok(Self { entries })
    }

    fn find(&self, tag: &[u8; 4]) -> Result<(usize, usize), BdlError> {
        self.entries
            .iter()
            .find(|(candidate, _, _)| candidate == tag)
            .map(|&(_, start, end)| (start, end))
            .ok_or(BdlError::Block)
    }
}

fn read_joints(data: &[u8], blocks: &Blocks) -> Result<Vec<[f32; 3]>, BdlError> {
    let (joint_start, _) = blocks.find(b"JNT1")?;
    let joint_count = usize::from(read_u16(data, joint_start + 8)?);
    let init_start = at(joint_start, read_u32(data, at(joint_start, 0x0C)?)? as usize)?;
    let index_start = at(joint_start, read_u32(data, at(joint_start, 0x10)?)? as usize)?;
    let init_len = index_start.checked_sub(init_start).ok_or(BdlError::Block)?;
    if init_len != joint_count * JOINT_INIT_STRIDE {
        return Err(BdlError::Block);
    }
    let mut locals = Vec::new();
    for joint in 0..joint_count {
        let slot = usize::from(read_u16(data, at(index_start, joint * 2)?)?);
        if slot >= joint_count {
            return Err(BdlError::Range);
        }
        let init = at(init_start, slot * JOINT_INIT_STRIDE)?;
        let scale = [
            read_f32(data, at(init, JOINT_SCALE)?)?,
            read_f32(data, at(init, JOINT_SCALE + 4)?)?,
            read_f32(data, at(init, JOINT_SCALE + 8)?)?,
        ];
        let rotation = [
            read_i16(data, at(init, JOINT_ROTATION)?)?,
            read_i16(data, at(init, JOINT_ROTATION + 2)?)?,
            read_i16(data, at(init, JOINT_ROTATION + 4)?)?,
        ];
        if scale != IDENTITY_SCALE || rotation != [0, 0, 0] {
            return Err(BdlError::Transform);
        }
        locals.push([
            read_f32(data, at(init, JOINT_TRANSLATION)?)?,
            read_f32(data, at(init, JOINT_TRANSLATION + 4)?)?,
            read_f32(data, at(init, JOINT_TRANSLATION + 8)?)?,
        ]);
    }

    let (hierarchy_start, hierarchy_end) = blocks.find(b"INF1")?;
    let mut cursor = at(
        hierarchy_start,
        read_u32(data, at(hierarchy_start, 0x14)?)? as usize,
    )?;
    if cursor > hierarchy_end {
        return Err(BdlError::Hierarchy);
    }
    let mut parents = vec![PARENT_UNSET; joint_count];
    let mut stack = vec![(PARENT_ROOT, PARENT_ROOT)];
    loop {
        if cursor + 4 > hierarchy_end {
            return Err(BdlError::Hierarchy);
        }
        let kind = read_u16(data, cursor)?;
        let value = read_u16(data, cursor + 2)?;
        cursor += 4;
        match kind {
            HIERARCHY_END => break,
            HIERARCHY_BEGIN_CHILD => {
                let current = stack.last().copied().ok_or(BdlError::Hierarchy)?.1;
                stack.push((current, current));
            }
            HIERARCHY_END_CHILD => {
                if stack.len() <= 1 {
                    return Err(BdlError::Hierarchy);
                }
                stack.pop();
            }
            HIERARCHY_JOINT => {
                let (parent, _) = stack.last().copied().ok_or(BdlError::Hierarchy)?;
                let joint = usize::from(value);
                if joint >= joint_count || parents[joint] != PARENT_UNSET {
                    return Err(BdlError::Hierarchy);
                }
                parents[joint] = parent;
                let top = stack.last_mut().ok_or(BdlError::Hierarchy)?;
                top.1 = joint as i32;
            }
            HIERARCHY_MATERIAL | HIERARCHY_SHAPE => {}
            _ => return Err(BdlError::Hierarchy),
        }
    }
    if parents.contains(&PARENT_UNSET) {
        return Err(BdlError::Hierarchy);
    }

    let mut worlds: Vec<Option<[f32; 3]>> = vec![None; joint_count];
    for joint in 0..joint_count {
        if worlds[joint].is_some() {
            continue;
        }
        let mut chain = Vec::new();
        let mut current = joint;
        let mut world = loop {
            if let Some(resolved) = worlds[current] {
                break resolved;
            }
            let parent = parents[current];
            if parent == PARENT_ROOT {
                let resolved = locals[current];
                worlds[current] = Some(resolved);
                break resolved;
            }
            if parent < 0 {
                return Err(BdlError::Hierarchy);
            }
            chain.push(current);
            current = parent as usize;
        };
        for member in chain.iter().rev() {
            let local = locals[*member];
            world = [
                world[0] + local[0],
                world[1] + local[1],
                world[2] + local[2],
            ];
            worlds[*member] = Some(world);
        }
    }
    worlds
        .into_iter()
        .map(|world| world.ok_or(BdlError::Hierarchy))
        .collect()
}

fn read_draw_joints(
    data: &[u8],
    blocks: &Blocks,
    joint_count: usize,
) -> Result<Vec<usize>, BdlError> {
    let (start, _) = blocks.find(b"DRW1")?;
    let count = usize::from(read_u16(data, start + 8)?);
    let flags_start = at(start, read_u32(data, at(start, 0x0C)?)? as usize)?;
    let index_start = at(start, read_u32(data, at(start, 0x10)?)? as usize)?;
    let mut joints = Vec::new();
    for slot in 0..count {
        let flag = *data.get(flags_start + slot).ok_or(BdlError::Truncated)?;
        if flag != 0 {
            return Err(BdlError::Envelope);
        }
        let joint = usize::from(read_u16(data, at(index_start, slot * 2)?)?);
        if joint >= joint_count {
            return Err(BdlError::Range);
        }
        joints.push(joint);
    }
    Ok(joints)
}

struct VertexArrays {
    position_at: usize,
    position_count: usize,
    color_at: usize,
    color_count: usize,
}

fn next_relative(data: &[u8], start: usize, after: u32) -> Result<Option<u32>, BdlError> {
    let mut smallest: Option<u32> = None;
    let mut field = after + 4;
    while field < 0x40 {
        let value = read_u32(data, at(start, field as usize)?)?;
        if value != 0 {
            smallest = Some(smallest.map_or(value, |current| current.min(value)));
        }
        field += 4;
    }
    Ok(smallest)
}

fn read_vertex_arrays(data: &[u8], blocks: &Blocks) -> Result<VertexArrays, BdlError> {
    let (start, end) = blocks.find(b"VTX1")?;
    let position_rel = read_u32(data, at(start, 0x0C)?)?;
    let color_rel = read_u32(data, at(start, 0x18)?)?;
    if position_rel == 0 {
        return Err(BdlError::Descriptor);
    }
    let position_end = match next_relative(data, start, 0x0C)? {
        Some(rel) => at(start, rel as usize)?,
        None => end,
    };
    let position_at = at(start, position_rel as usize)?;
    let position_bytes = position_end
        .checked_sub(position_at)
        .ok_or(BdlError::Block)?;
    let position_count = position_bytes / POSITION_STRIDE;

    let (color_at, color_count) = if color_rel == 0 {
        (0, 0)
    } else {
        let color_end = match next_relative(data, start, 0x18)? {
            Some(rel) => at(start, rel as usize)?,
            None => end,
        };
        let color_at = at(start, color_rel as usize)?;
        let color_bytes = color_end.checked_sub(color_at).ok_or(BdlError::Block)?;
        (color_at, color_bytes / COLOR_STRIDE)
    };

    let mut position_form = None;
    let mut color_form = None;
    let mut cursor = at(start, read_u32(data, at(start, 0x08)?)? as usize)?;
    loop {
        let attribute = read_u32(data, cursor)?;
        let components = read_u32(data, at(cursor, 4)?)?;
        let kind = read_u32(data, at(cursor, 8)?)?;
        cursor += FORM_STRIDE;
        if attribute == ATTRIBUTE_TERM {
            break;
        }
        if attribute == ATTRIBUTE_POSITION {
            position_form = Some((components, kind));
        }
        if attribute == ATTRIBUTE_COLOR0 {
            color_form = Some((components, kind));
        }
    }
    if position_form != Some(POSITION_FORM) {
        return Err(BdlError::Descriptor);
    }
    if color_rel != 0 && color_form != Some(COLOR_FORM) {
        return Err(BdlError::Descriptor);
    }

    Ok(VertexArrays {
        position_at,
        position_count,
        color_at,
        color_count,
    })
}

struct ShapeContext<'a> {
    data: &'a [u8],
    vertices: &'a VertexArrays,
    origin: [f32; 2],
    sin: f32,
    cos: f32,
}

fn read_shapes(
    context: &ShapeContext<'_>,
    blocks: &Blocks,
    worlds: &[[f32; 3]],
    draw_joints: &[usize],
    mesh: &mut RoomMesh,
) -> Result<(), BdlError> {
    let (shape_start, _) = blocks.find(b"SHP1")?;
    let data = context.data;
    let shape_count = usize::from(read_u16(data, shape_start + 8)?);
    let init_table = at(shape_start, read_u32(data, at(shape_start, 0x0C)?)? as usize)?;
    let index_table = at(shape_start, read_u32(data, at(shape_start, 0x10)?)? as usize)?;
    let descriptor_table = at(shape_start, read_u32(data, at(shape_start, 0x18)?)? as usize)?;
    let display_table = at(shape_start, read_u32(data, at(shape_start, 0x20)?)? as usize)?;
    let matrix_table = at(shape_start, read_u32(data, at(shape_start, 0x24)?)? as usize)?;
    let draw_table = at(shape_start, read_u32(data, at(shape_start, 0x28)?)? as usize)?;

    for shape in 0..shape_count {
        let slot = usize::from(read_u16(data, at(index_table, shape * 2)?)?);
        let init = at(init_table, slot * SHAPE_INIT_STRIDE)?;
        let matrix_type = *data.get(init).ok_or(BdlError::Truncated)?;
        if matrix_type != SHAPE_MATRIX_DIRECT {
            return Err(BdlError::Shape);
        }
        let group_count = usize::from(read_u16(data, at(init, 2)?)?);
        let descriptor_offset = usize::from(read_u16(data, at(init, 4)?)?);
        let matrix_slot = usize::from(read_u16(data, at(init, 6)?)?);
        let draw_slot = usize::from(read_u16(data, at(init, 8)?)?);

        let mut descriptor = Vec::new();
        let mut cursor = at(descriptor_table, descriptor_offset)?;
        loop {
            let attribute = read_u32(data, cursor)?;
            let kind = read_u32(data, at(cursor, 4)?)?;
            cursor += DESCRIPTOR_STRIDE;
            if attribute == ATTRIBUTE_TERM {
                break;
            }
            if kind != TYPE_INDEX16 {
                return Err(BdlError::Descriptor);
            }
            descriptor.push(attribute);
        }
        let position_field = descriptor
            .iter()
            .position(|&attribute| attribute == ATTRIBUTE_POSITION)
            .ok_or(BdlError::Descriptor)?;
        let color_field = descriptor
            .iter()
            .position(|&attribute| attribute == ATTRIBUTE_COLOR0);

        for group in 0..group_count {
            let matrix = at(matrix_table, (matrix_slot + group) * SHAPE_GROUP_STRIDE)?;
            let draw_matrix = usize::from(read_u16(data, matrix)?);
            if draw_matrix >= draw_joints.len() {
                return Err(BdlError::Range);
            }
            let world = worlds[draw_joints[draw_matrix]];
            let draw = at(draw_table, (draw_slot + group) * SHAPE_GROUP_STRIDE)?;
            let display_size = read_u32(data, draw)? as usize;
            let display_start = at(display_table, read_u32(data, at(draw, 4)?)? as usize)?;
            let display_end = display_start
                .checked_add(display_size)
                .ok_or(BdlError::Truncated)?;
            let display = data
                .get(display_start..display_end)
                .ok_or(BdlError::Truncated)?;
            decode_display_list(
                context,
                display,
                &descriptor,
                position_field,
                color_field,
                world,
                mesh,
            )?;
        }
    }
    Ok(())
}

fn emit(mesh: &mut RoomMesh, base: u32, a: usize, b: usize, c: usize) {
    mesh.indices
        .extend([base + a as u32, base + c as u32, base + b as u32]);
}

fn decode_display_list(
    context: &ShapeContext<'_>,
    display: &[u8],
    descriptor: &[u32],
    position_field: usize,
    color_field: Option<usize>,
    world: [f32; 3],
    mesh: &mut RoomMesh,
) -> Result<(), BdlError> {
    let data = context.data;
    let stride = descriptor.len() * 2;
    let mut cursor = 0_usize;
    while cursor < display.len() {
        let command = display[cursor];
        if command == COMMAND_NOP {
            cursor += 1;
            continue;
        }
        let count = match command {
            COMMAND_QUADS
            | COMMAND_TRIANGLES
            | COMMAND_STRIP
            | COMMAND_FAN
            | COMMAND_LINES
            | COMMAND_LINE_STRIP
            | COMMAND_POINTS => {
                let bytes = display
                    .get(cursor + 1..cursor + 3)
                    .ok_or(BdlError::Truncated)?;
                cursor += 3;
                usize::from(u16::from_be_bytes([bytes[0], bytes[1]]))
            }
            _ => return Err(BdlError::Command),
        };
        if command == COMMAND_QUADS && count % 4 != 0 {
            return Err(BdlError::Command);
        }
        if command == COMMAND_TRIANGLES && count % 3 != 0 {
            return Err(BdlError::Command);
        }
        let indexed =
            !matches!(command, COMMAND_LINES | COMMAND_LINE_STRIP | COMMAND_POINTS);
        let base = mesh.positions.len() as u32;
        for _ in 0..count {
            let bytes = display
                .get(cursor..cursor + stride)
                .ok_or(BdlError::Truncated)?;
            cursor += stride;
            let mut position_index = None;
            let mut color_index = None;
            for field in 0..descriptor.len() {
                let index = usize::from(u16::from_be_bytes([
                    bytes[field * 2],
                    bytes[field * 2 + 1],
                ]));
                if field == position_field {
                    position_index = Some(index);
                }
                if color_field == Some(field) {
                    color_index = Some(index);
                }
            }
            let position_index = position_index.ok_or(BdlError::Descriptor)?;
            if position_index >= context.vertices.position_count {
                return Err(BdlError::Range);
            }
            let color_index = match color_field {
                Some(_) => {
                    let index = color_index.ok_or(BdlError::Descriptor)?;
                    if index >= context.vertices.color_count {
                        return Err(BdlError::Range);
                    }
                    Some(index)
                }
                None => None,
            };
            if !indexed {
                continue;
            }
            let position_at = context.vertices.position_at + position_index * POSITION_STRIDE;
            let local = [
                read_f32(data, position_at)?,
                read_f32(data, position_at + 4)?,
                read_f32(data, position_at + 8)?,
            ];
            let lx = local[0] + world[0];
            let ly = local[1] + world[1];
            let lz = local[2] + world[2];
            mesh.positions.push([
                lx * context.cos + lz * context.sin + context.origin[0],
                ly,
                -lx * context.sin + lz * context.cos + context.origin[1],
            ]);
            mesh.colors.push(match color_index {
                Some(index) => {
                    let color_at = context.vertices.color_at + index * COLOR_STRIDE;
                    let bytes = data
                        .get(color_at..color_at + 3)
                        .ok_or(BdlError::Truncated)?;
                    [
                        f32::from(bytes[0]) / 255.0,
                        f32::from(bytes[1]) / 255.0,
                        f32::from(bytes[2]) / 255.0,
                    ]
                }
                None => WHITE,
            });
        }
        if !indexed {
            continue;
        }
        match command {
            COMMAND_QUADS => {
                for vertex in (0..count).step_by(4) {
                    emit(mesh, base, vertex, vertex + 1, vertex + 2);
                    emit(mesh, base, vertex, vertex + 2, vertex + 3);
                }
            }
            COMMAND_TRIANGLES => {
                for vertex in (0..count).step_by(3) {
                    emit(mesh, base, vertex, vertex + 1, vertex + 2);
                }
            }
            COMMAND_STRIP => {
                for vertex in 0..count.saturating_sub(2) {
                    if vertex % 2 == 0 {
                        emit(mesh, base, vertex, vertex + 1, vertex + 2);
                    } else {
                        emit(mesh, base, vertex + 1, vertex, vertex + 2);
                    }
                }
            }
            COMMAND_FAN => {
                for vertex in 1..count.saturating_sub(1) {
                    emit(mesh, base, 0, vertex, vertex + 1);
                }
            }
            _ => return Err(BdlError::Command),
        }
    }
    Ok(())
}
