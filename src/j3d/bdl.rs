use glam::Mat4;

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
const NORMAL_STRIDE_S16: usize = 6;
const NORMAL_STRIDE_F32: usize = 12;
const ATTRIBUTE_TERM: u32 = 0xFF;
const ATTRIBUTE_MATRIX: u32 = 0;
const ATTRIBUTE_POSITION: u32 = 9;
const ATTRIBUTE_NORMAL: u32 = 10;
const ATTRIBUTE_COLOR0: u32 = 11;
const ATTRIBUTE_TEX0: u32 = 13;
const DESCRIPTOR_WIDTHS: [usize; 4] = [0, 1, 1, 2];
const TYPE_S16: u32 = 3;
const TYPE_F32: u32 = 4;
const TYPE_RGBA8: u32 = 5;
const COMPONENT_ONE: u32 = 1;
const POSITION_FORM: (u32, u32) = (COMPONENT_ONE, TYPE_F32);
const COLOR_FORM: (u32, u32) = (COMPONENT_ONE, TYPE_RGBA8);
const TEXCOORD_FORM: (u32, u32) = (COMPONENT_ONE, TYPE_S16);
const NORMAL_FORM_S16: (u32, u32) = (0, TYPE_S16);
const NORMAL_FORM_F32: (u32, u32) = (0, TYPE_F32);
const TEXCOORD_STRIDE: usize = 4;
const TEXTURE_STRIDE: usize = 0x20;
const TEXTURE_IMAGE_OFFSET: usize = 0x1C;
const MATERIAL_INIT_STRIDE: usize = 0x14C;
const MATERIAL_TEX_NO: usize = 0x084;
const MATERIAL_COLOR_CHAN_NUM: usize = 0x002;
const MATERIAL_COLOR_CHAN_IDX: usize = 0x00C;
const COLOR_CHAN_STRIDE: usize = 8;
const COLOR_CHAN_MAX: usize = 4;
const MATERIAL_UNTEXTURED: u16 = 0xFFFF;
const MATERIAL_TEX_GEN_NUM: usize = 0x003;
const MATERIAL_TEV_STAGE_NUM: usize = 0x004;
const MATERIAL_TEX_COORD_IDX: usize = 0x028;
const MATERIAL_TEV_ORDER_IDX: usize = 0x0BC;
const TEX_COORD_STRIDE: usize = 4;
const TEV_ORDER_STRIDE: usize = 4;
const TEX_GEN_MAX: u8 = 8;
const TEV_STAGE_MAX: u8 = 16;
const GX_TEXMTX0: u8 = 30;
const GX_IDENTITY: u8 = 60;
const FORMAT_I4: u8 = 0;
const FORMAT_IA4: u8 = 2;
const FORMAT_CMPR: u8 = 0xE;
const WRAP_CLAMP: u8 = 0;
const WRAP_REPEAT: u8 = 1;
const WRAP_MIRROR: u8 = 2;
const BOX_SAMPLES: usize = 16;
pub const LAYER_SIZE: u32 = 512;
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
const TRIG_SHIFT: u32 = 4;
const TRIG_ENTRIES: f64 = 4096.0;
const TRIG_QUARTER: u32 = 1024;
const PARENT_NONE: u16 = 0xFFFF;
const SHAPE_MATRIX_DIRECT: u8 = 0;
const SHAPE_MATRIX_MULTI: u8 = 3;
const MATRIX_INDEX_DIVISOR: u32 = 3;
const MATRIX_UNSET: u16 = 0xFFFF;
const SHAPE_INDEX_TABLE: usize = 0x1C;
const SHAPE_INIT_MATRIX_SLOTS: usize = 0x02;
const SHAPE_INIT_MATRIX_FIRST: usize = 0x04;
const ENVELOPE_TABLE_COUNT: usize = 0x08;
const ENVELOPE_TABLE_MIXES: usize = 0x0C;
const ENVELOPE_TABLE_INDEX: usize = 0x10;
const ENVELOPE_TABLE_WEIGHT: usize = 0x14;
pub const SKIN_INFLUENCES: usize = 4;
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
    Texture,
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
            Self::Texture => "bdl texture format unsupported",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnimBinding {
    pub set: u32,
    pub track: u32,
}

#[derive(Debug, Clone, Default)]
pub struct RoomMesh {
    pub groups: Vec<RoomGroup>,
    pub textures: Vec<RoomTexture>,
    pub uv_placements: Vec<[f32; 4]>,
    pub animations: Vec<crate::j3d::btk::TextureAnimation>,
    pub joints: Vec<[f32; 3]>,
    pub joint_matrices: Vec<Mat4>,
    pub joint_parents: Vec<u16>,
}

#[derive(Debug, Clone, Default)]
pub struct RoomGroup {
    pub positions: Vec<[f32; 3]>,
    pub colors: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
    pub joint_indices: Vec<[u16; SKIN_INFLUENCES]>,
    pub joint_weights: Vec<[f32; SKIN_INFLUENCES]>,
    pub texture_page: Option<u32>,
    pub lit: bool,
    pub anim: Option<AnimBinding>,
}

#[derive(Debug, Clone)]
pub struct RoomTexture {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

impl RoomMesh {
    pub fn append(&mut self, mut part: RoomMesh) {
        let page_base = self.textures.len() as u32;
        let animation_base = self.animations.len() as u32;
        for group in &mut part.groups {
            if let Some(page) = group.texture_page.as_mut() {
                *page += page_base;
            }
            if let Some(binding) = group.anim.as_mut() {
                binding.set += animation_base;
            }
        }
        self.uv_placements.append(&mut part.uv_placements);
        self.animations.append(&mut part.animations);
        self.groups.append(&mut part.groups);
        self.textures.append(&mut part.textures);
        self.joints.append(&mut part.joints);
        self.joint_matrices.append(&mut part.joint_matrices);
        self.joint_parents.append(&mut part.joint_parents);
    }
}

pub fn parse(
    data: &[u8],
    origin: [f32; 2],
    angle: f32,
    animation: Option<&crate::j3d::btk::TextureAnimation>,
) -> Result<RoomMesh, BdlError> {
    let blocks = Blocks::parse(data)?;
    let (worlds, matrices, parents, material_nodes) = read_joints(data, &blocks)?;
    let draw_joints = read_draw_joints(data, &blocks, worlds.len())?;
    let envelopes = read_envelopes(data, &blocks, worlds.len())?;
    let vertices = read_vertex_arrays(data, &blocks)?;
    let materials = read_materials(data, &blocks)?;
    let textures = read_textures(data, &blocks)?;
    let (sin, cos) = angle.sin_cos();
    let context = ShapeContext {
        data,
        vertices: &vertices,
        origin,
        sin,
        cos,
        animation,
    };
    let mut mesh = RoomMesh::default();
    mesh.joints = worlds.clone();
    mesh.joint_matrices = matrices;
    mesh.joint_parents = parents;
    if let Some(animation) = animation {
        mesh.animations.push(animation.clone());
    }
    read_shapes(
        &context,
        &blocks,
        &worlds,
        &draw_joints,
        &envelopes,
        &material_nodes,
        &materials,
        textures.len(),
        &mut mesh,
    )?;
    bake_textures(&mut mesh, &textures);
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

fn trig_entry(index: u32) -> f32 {
    (f64::from(index) * (std::f64::consts::TAU / TRIG_ENTRIES)).sin() as f32
}

fn trig_index(value: i16) -> u32 {
    u32::from(value as u16) >> TRIG_SHIFT
}

fn trig_sin(value: i16) -> f32 {
    trig_entry(trig_index(value))
}

fn trig_cos(value: i16) -> f32 {
    trig_entry(trig_index(value) + TRIG_QUARTER)
}

fn joint_matrix(
    scale: [f32; 3],
    rotation: [i16; 3],
    translation: [f32; 3],
) -> Mat4 {
    let sx = trig_sin(rotation[0]);
    let cx = trig_cos(rotation[0]);
    let sy = trig_sin(rotation[1]);
    let cy = trig_cos(rotation[1]);
    let sz = trig_sin(rotation[2]);
    let cz = trig_cos(rotation[2]);
    let mut columns = [[0.0_f32; 4]; 4];
    columns[2][0] = -sy;
    columns[0][0] = cz * cy;
    columns[1][0] = sz * cy;
    columns[2][1] = cy * sx;
    columns[2][2] = cy * cx;
    let cxsz = cx * sz;
    let sxcz = sx * cz;
    columns[0][1] = sxcz * sy - cxsz;
    columns[1][2] = cxsz * sy - sxcz;
    let sxsz = sx * sz;
    let cxcz = cx * cz;
    columns[0][2] = cxcz * sy + sxsz;
    columns[1][1] = sxsz * sy + cxcz;
    columns[3] = [0.0, 0.0, 0.0, 1.0];
    for column in columns.iter_mut().take(3) {
        column[0] *= scale[0];
        column[1] *= scale[1];
        column[2] *= scale[2];
    }
    columns[0][3] = translation[0];
    columns[1][3] = translation[1];
    columns[2][3] = translation[2];
    Mat4::from_cols_array_2d(&columns)
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

    fn get(&self, tag: &[u8; 4]) -> Option<(usize, usize)> {
        self.entries
            .iter()
            .find(|(candidate, _, _)| candidate == tag)
            .map(|&(_, start, end)| (start, end))
    }

    fn find(&self, tag: &[u8; 4]) -> Result<(usize, usize), BdlError> {
        self.get(tag).ok_or(BdlError::Block)
    }
}

fn read_joints(
    data: &[u8],
    blocks: &Blocks,
) -> Result<(Vec<[f32; 3]>, Vec<Mat4>, Vec<u16>, Vec<(u16, Option<u16>)>), BdlError> {
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
        let translation = [
            read_f32(data, at(init, JOINT_TRANSLATION)?)?,
            read_f32(data, at(init, JOINT_TRANSLATION + 4)?)?,
            read_f32(data, at(init, JOINT_TRANSLATION + 8)?)?,
        ];
        locals.push(joint_matrix(scale, rotation, translation));
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
    let mut material_nodes = Vec::new();
    let mut current_material = None;
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
            HIERARCHY_MATERIAL => current_material = Some(value),
            HIERARCHY_SHAPE => material_nodes.push((value, current_material)),
            _ => return Err(BdlError::Hierarchy),
        }
    }
    if parents.contains(&PARENT_UNSET) {
        return Err(BdlError::Hierarchy);
    }

    let mut resolved_matrices: Vec<Option<Mat4>> = vec![None; joint_count];
    for joint in 0..joint_count {
        if resolved_matrices[joint].is_some() {
            continue;
        }
        let mut chain = Vec::new();
        let mut current = joint;
        let mut world = loop {
            if let Some(resolved) = resolved_matrices[current] {
                break resolved;
            }
            let parent = parents[current];
            if parent == PARENT_ROOT {
                let resolved = locals[current];
                resolved_matrices[current] = Some(resolved);
                break resolved;
            }
            if parent < 0 {
                return Err(BdlError::Hierarchy);
            }
            chain.push(current);
            current = parent as usize;
        };
        for member in chain.iter().rev() {
            world *= locals[*member];
            resolved_matrices[*member] = Some(world);
        }
    }
    let matrices = resolved_matrices
        .into_iter()
        .map(|matrix| matrix.ok_or(BdlError::Hierarchy))
        .collect::<Result<Vec<_>, BdlError>>()?;
    let worlds = matrices
        .iter()
        .map(|matrix| [matrix.w_axis.x, matrix.w_axis.y, matrix.w_axis.z])
        .collect();
    let joint_parents = parents
        .iter()
        .map(|&parent| {
            if parent < 0 {
                PARENT_NONE
            } else {
                parent as u16
            }
        })
        .collect();
    Ok((worlds, matrices, joint_parents, material_nodes))
}

#[derive(Debug, Clone, Copy)]
struct DrawBinding {
    target: u16,
    blended: bool,
}

#[derive(Debug, Clone, Copy, Default)]
struct SkinBinding {
    indices: [u16; SKIN_INFLUENCES],
    weights: [f32; SKIN_INFLUENCES],
}

fn read_draw_joints(
    data: &[u8],
    blocks: &Blocks,
    joint_count: usize,
) -> Result<Vec<DrawBinding>, BdlError> {
    let (start, _) = blocks.find(b"DRW1")?;
    let count = usize::from(read_u16(data, start + 8)?);
    let flags_start = at(start, read_u32(data, at(start, 0x0C)?)? as usize)?;
    let index_start = at(start, read_u32(data, at(start, 0x10)?)? as usize)?;
    let mut joints = Vec::with_capacity(count);
    for slot in 0..count {
        let flag = *data.get(flags_start + slot).ok_or(BdlError::Truncated)?;
        let target = read_u16(data, at(index_start, slot * 2)?)?;
        if flag == 0 && usize::from(target) >= joint_count {
            return Err(BdlError::Range);
        }
        joints.push(DrawBinding {
            target,
            blended: flag != 0,
        });
    }
    Ok(joints)
}

fn read_envelopes(
    data: &[u8],
    blocks: &Blocks,
    joint_count: usize,
) -> Result<Vec<SkinBinding>, BdlError> {
    let Some((start, end)) = blocks.get(b"EVP1") else {
        return Ok(Vec::new());
    };
    let count = usize::from(read_u16(data, at(start, ENVELOPE_TABLE_COUNT)?)?);
    let mixes_at = at(start, read_u32(data, at(start, ENVELOPE_TABLE_MIXES)?)? as usize)?;
    let index_at = at(start, read_u32(data, at(start, ENVELOPE_TABLE_INDEX)?)? as usize)?;
    let weight_at = at(start, read_u32(data, at(start, ENVELOPE_TABLE_WEIGHT)?)? as usize)?;
    let mut bindings = Vec::with_capacity(count);
    let mut mix = 0_usize;
    for envelope in 0..count {
        let entries = usize::from(
            *data
                .get(at(mixes_at, envelope)?)
                .ok_or(BdlError::Truncated)?,
        );
        if entries == 0 || entries > SKIN_INFLUENCES {
            return Err(BdlError::Envelope);
        }
        let mut binding = SkinBinding::default();
        for slot in 0..entries {
            let joint = at(index_at, (mix + slot) * 2)?;
            let weight = at(weight_at, (mix + slot) * 4)?;
            if at(joint, 2)? > end || at(weight, 4)? > end {
                return Err(BdlError::Block);
            }
            let index = read_u16(data, joint)?;
            if usize::from(index) >= joint_count {
                return Err(BdlError::Range);
            }
            binding.indices[slot] = index;
            binding.weights[slot] = read_f32(data, weight)?;
        }
        mix += entries;
        bindings.push(binding);
    }
    Ok(bindings)
}

fn skin_binding(
    draw: u16,
    draws: &[DrawBinding],
    envelopes: &[SkinBinding],
    joint_count: usize,
) -> Result<SkinBinding, BdlError> {
    let entry = *draws.get(usize::from(draw)).ok_or(BdlError::Range)?;
    if !entry.blended {
        if usize::from(entry.target) >= joint_count {
            return Err(BdlError::Range);
        }
        let mut binding = SkinBinding::default();
        binding.indices[0] = entry.target;
        binding.weights[0] = 1.0;
        return Ok(binding);
    }
    Ok(*envelopes.get(usize::from(entry.target)).ok_or(BdlError::Range)?)
}

struct VertexArrays {
    position_at: usize,
    position_count: usize,
    color_at: usize,
    color_count: usize,
    normal_at: usize,
    normal_count: usize,
    normal_stride: usize,
    normal_frac: u32,
    texcoord_at: usize,
    texcoord_count: usize,
    texcoord_frac: u32,
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
    let normal_rel = read_u32(data, at(start, 0x10)?)?;
    let color_rel = read_u32(data, at(start, 0x18)?)?;
    let texcoord_rel = read_u32(data, at(start, 0x20)?)?;
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

    let (normal_at, normal_bytes) = if normal_rel == 0 {
        (0, 0)
    } else {
        let normal_end = match next_relative(data, start, 0x10)? {
            Some(rel) => at(start, rel as usize)?,
            None => end,
        };
        let normal_at = at(start, normal_rel as usize)?;
        let normal_bytes = normal_end.checked_sub(normal_at).ok_or(BdlError::Block)?;
        (normal_at, normal_bytes)
    };

    let (texcoord_at, texcoord_count) = if texcoord_rel == 0 {
        (0, 0)
    } else {
        let texcoord_end = match next_relative(data, start, 0x20)? {
            Some(rel) => at(start, rel as usize)?,
            None => end,
        };
        let texcoord_at = at(start, texcoord_rel as usize)?;
        let texcoord_bytes = texcoord_end.checked_sub(texcoord_at).ok_or(BdlError::Block)?;
        (texcoord_at, texcoord_bytes / TEXCOORD_STRIDE)
    };

    let mut position_form = None;
    let mut normal_form = None;
    let mut color_form = None;
    let mut texcoord_form = None;
    let mut normal_frac = 0_u32;
    let mut texcoord_frac = 0_u32;
    let mut cursor = at(start, read_u32(data, at(start, 0x08)?)? as usize)?;
    loop {
        let attribute = read_u32(data, cursor)?;
        let components = read_u32(data, at(cursor, 4)?)?;
        let kind = read_u32(data, at(cursor, 8)?)?;
        if attribute == ATTRIBUTE_TEX0 {
            texcoord_form = Some((components, kind));
            texcoord_frac = u32::from(*data.get(at(cursor, 12)?).ok_or(BdlError::Truncated)?);
        }
        if attribute == ATTRIBUTE_NORMAL {
            normal_form = Some((components, kind));
            normal_frac = u32::from(*data.get(at(cursor, 12)?).ok_or(BdlError::Truncated)?);
        }
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
    if texcoord_rel != 0 && texcoord_form != Some(TEXCOORD_FORM) {
        return Err(BdlError::Descriptor);
    }
    if texcoord_rel != 0 && texcoord_frac > 31 {
        return Err(BdlError::Descriptor);
    }
    if normal_rel != 0
        && normal_form != Some(NORMAL_FORM_S16)
        && normal_form != Some(NORMAL_FORM_F32)
    {
        return Err(BdlError::Descriptor);
    }
    if normal_rel != 0 && normal_form == Some(NORMAL_FORM_S16) && normal_frac > 31 {
        return Err(BdlError::Descriptor);
    }
    let (normal_stride, normal_count) = if normal_rel == 0 {
        (0, 0)
    } else if normal_form == Some(NORMAL_FORM_F32) {
        (NORMAL_STRIDE_F32, normal_bytes / NORMAL_STRIDE_F32)
    } else {
        (NORMAL_STRIDE_S16, normal_bytes / NORMAL_STRIDE_S16)
    };

    Ok(VertexArrays {
        position_at,
        position_count,
        color_at,
        color_count,
        normal_at,
        normal_count,
        normal_stride,
        normal_frac,
        texcoord_at,
        texcoord_count,
        texcoord_frac,
    })
}

#[derive(Debug, Clone)]
struct MaterialBinding {
    texture: Option<u16>,
    lit: bool,
    name: Option<String>,
    coord: Option<u8>,
    init_matrix: Option<u8>,
}

fn material_lit(
    data: &[u8],
    start: usize,
    end: usize,
    init: usize,
    color_chan_num: u32,
    color_chan_info: u32,
) -> Result<bool, BdlError> {
    if color_chan_num == 0 || color_chan_info == 0 {
        return Ok(false);
    }
    let num_table = at(start, color_chan_num as usize)?;
    let info_table = at(start, color_chan_info as usize)?;
    let num_index = usize::from(
        *data
            .get(at(init, MATERIAL_COLOR_CHAN_NUM)?)
            .ok_or(BdlError::Truncated)?,
    );
    let num_at = at(num_table, num_index)?;
    if at(num_at, 1)? > end {
        return Err(BdlError::Block);
    }
    let channel_count = usize::from(*data.get(num_at).ok_or(BdlError::Truncated)?);
    if channel_count > COLOR_CHAN_MAX {
        return Err(BdlError::Descriptor);
    }
    for channel in 0..channel_count {
        let info_index =
            usize::from(read_u16(data, at(init, MATERIAL_COLOR_CHAN_IDX + channel * 2)?)?);
        let info = at(info_table, info_index * COLOR_CHAN_STRIDE)?;
        if at(info, COLOR_CHAN_STRIDE)? > end {
            return Err(BdlError::Block);
        }
        if *data.get(info).ok_or(BdlError::Truncated)? != 0 {
            return Ok(true);
        }
    }
    Ok(false)
}

fn read_material_names(
    data: &[u8],
    start: usize,
    end: usize,
    table: usize,
    count: usize,
) -> Result<Vec<String>, BdlError> {
    let base = at(start, table)?;
    if at(base, 4)? > end {
        return Err(BdlError::Block);
    }
    if usize::from(read_u16(data, base)?) < count {
        return Err(BdlError::Block);
    }
    if at(base, 4 + count * 4)? > end {
        return Err(BdlError::Block);
    }
    let mut names = Vec::with_capacity(count);
    for material in 0..count {
        let offset = usize::from(read_u16(data, at(base, 4 + material * 4 + 2)?)?);
        let string = at(base, offset)?;
        if string >= end {
            return Err(BdlError::Block);
        }
        let bytes = data.get(string..end).ok_or(BdlError::Truncated)?;
        let length = bytes
            .iter()
            .position(|&byte| byte == 0)
            .ok_or(BdlError::Block)?;
        let name = std::str::from_utf8(&bytes[..length]).map_err(|_| BdlError::Block)?;
        names.push(name.to_owned());
    }
    Ok(names)
}

fn block_byte(data: &[u8], base: usize, offset: usize, end: usize) -> Result<u8, BdlError> {
    let position = at(base, offset)?;
    if position >= end {
        return Err(BdlError::Block);
    }
    data.get(position).copied().ok_or(BdlError::Truncated)
}

fn material_coord(
    data: &[u8],
    start: usize,
    end: usize,
    init: usize,
    tex_gen_num_table: u32,
    tex_coord_table: u32,
    tev_order_table: u32,
    tev_stage_table: u32,
) -> Result<(Option<u8>, Option<u8>), BdlError> {
    if tex_gen_num_table == 0
        || tex_coord_table == 0
        || tev_order_table == 0
        || tev_stage_table == 0
    {
        return Ok((None, None));
    }
    let tex_gen_num_index = usize::from(block_byte(
        data,
        init,
        MATERIAL_TEX_GEN_NUM,
        end,
    )?);
    let tex_gen_num_table_at = at(start, tex_gen_num_table as usize)?;
    let tex_gen_num = block_byte(data, tex_gen_num_table_at, tex_gen_num_index, end)?;
    if tex_gen_num > TEX_GEN_MAX {
        return Err(BdlError::Descriptor);
    }
    let stage_num_index = usize::from(block_byte(
        data,
        init,
        MATERIAL_TEV_STAGE_NUM,
        end,
    )?);
    let stage_num_table_at = at(start, tev_stage_table as usize)?;
    let stage_num = block_byte(data, stage_num_table_at, stage_num_index, end)?;
    if stage_num > TEV_STAGE_MAX {
        return Err(BdlError::Descriptor);
    }
    let tev_order_table_at = at(start, tev_order_table as usize)?;
    let mut coord = None;
    for stage in 0..usize::from(stage_num) {
        if at(init, MATERIAL_TEV_ORDER_IDX + stage * 2 + 2)? > end {
            return Err(BdlError::Block);
        }
        let order_index =
            usize::from(read_u16(data, at(init, MATERIAL_TEV_ORDER_IDX + stage * 2)?)?);
        let order = at(tev_order_table_at, order_index * TEV_ORDER_STRIDE)?;
        if at(order, TEV_ORDER_STRIDE)? > end {
            return Err(BdlError::Block);
        }
        if block_byte(data, order, 1, end)? != 0 {
            continue;
        }
        let slot = block_byte(data, order, 0, end)?;
        if usize::from(slot) >= usize::from(tex_gen_num) {
            return Err(BdlError::Descriptor);
        }
        coord = Some(slot);
        break;
    }
    let init_matrix = match coord {
        None => None,
        Some(slot) => {
            if at(init, MATERIAL_TEX_COORD_IDX + usize::from(slot) * 2 + 2)? > end {
                return Err(BdlError::Block);
            }
            let info_index = usize::from(read_u16(
                data,
                at(init, MATERIAL_TEX_COORD_IDX + usize::from(slot) * 2)?,
            )?);
            let info_table_at = at(start, tex_coord_table as usize)?;
            let info = at(info_table_at, info_index * TEX_COORD_STRIDE)?;
            if at(info, TEX_COORD_STRIDE)? > end {
                return Err(BdlError::Block);
            }
            let matrix = block_byte(data, info, 2, end)?;
            match matrix.checked_sub(GX_TEXMTX0) {
                Some(offset) if matrix < GX_IDENTITY && offset % 3 == 0 => {
                    Some((offset / 3) as u8)
                }
                _ => None,
            }
        }
    };
    Ok((coord, init_matrix))
}

fn material_anim(
    animation: &crate::j3d::btk::TextureAnimation,
    binding: &MaterialBinding,
) -> Option<AnimBinding> {
    let name = binding.name.as_deref()?;
    let coord = binding.coord?;
    let slot = if animation.has_track(name, coord) {
        coord
    } else {
        binding.init_matrix?
    };
    let track = animation.track(name, slot)?;
    Some(AnimBinding { set: 0, track })
}

fn read_materials(data: &[u8], blocks: &Blocks) -> Result<Vec<MaterialBinding>, BdlError> {
    let Some((start, end)) = blocks.get(b"MAT3") else {
        return Ok(Vec::new());
    };
    let count = usize::from(read_u16(data, at(start, 8)?)?);
    let init_table = at(start, read_u32(data, at(start, 0x0C)?)? as usize)?;
    let index_table = at(start, read_u32(data, at(start, 0x10)?)? as usize)?;
    let texno_table = at(start, read_u32(data, at(start, 0x48)?)? as usize)?;
    let color_chan_num = read_u32(data, at(start, 0x24)?)?;
    let color_chan_info = read_u32(data, at(start, 0x28)?)?;
    let name_table = read_u32(data, at(start, 0x14)?)?;
    let tex_gen_num_table = read_u32(data, at(start, 0x34)?)?;
    let tex_coord_table = read_u32(data, at(start, 0x38)?)?;
    let tev_order_table = read_u32(data, at(start, 0x4C)?)?;
    let tev_stage_table = read_u32(data, at(start, 0x58)?)?;
    let names = if name_table == 0 {
        Vec::new()
    } else {
        read_material_names(data, start, end, name_table as usize, count)?
    };
    let mut materials = Vec::with_capacity(count);
    for material in 0..count {
        let slot = usize::from(read_u16(data, at(index_table, material * 2)?)?);
        let init = at(init_table, slot * MATERIAL_INIT_STRIDE)?;
        if at(init, MATERIAL_TEX_NO + 2)? > end {
            return Err(BdlError::Block);
        }
        let texno = read_u16(data, at(init, MATERIAL_TEX_NO)?)?;
        let texture = if texno == MATERIAL_UNTEXTURED {
            None
        } else {
            let binding = at(texno_table, usize::from(texno) * 2)?;
            if at(binding, 2)? > end {
                return Err(BdlError::Block);
            }
            Some(read_u16(data, binding)?)
        };
        let lit = material_lit(
            data,
            start,
            end,
            init,
            color_chan_num,
            color_chan_info,
        )?;
        let (coord, init_matrix) = material_coord(
            data,
            start,
            end,
            init,
            tex_gen_num_table,
            tex_coord_table,
            tev_order_table,
            tev_stage_table,
        )?;
        materials.push(MaterialBinding {
            texture,
            lit,
            name: names.get(material).cloned(),
            coord,
            init_matrix,
        });
    }
    Ok(materials)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Wrap {
    Clamp,
    Repeat,
    Mirror,
}

struct NativeTexture {
    width: usize,
    height: usize,
    wrap_s: Wrap,
    wrap_t: Wrap,
    rgba: Vec<u8>,
}

fn read_wrap(value: u8) -> Result<Wrap, BdlError> {
    match value {
        WRAP_CLAMP => Ok(Wrap::Clamp),
        WRAP_REPEAT => Ok(Wrap::Repeat),
        WRAP_MIRROR => Ok(Wrap::Mirror),
        _ => Err(BdlError::Texture),
    }
}

fn texture_size(format: u8, width: usize, height: usize) -> Result<usize, BdlError> {
    if width == 0 || height == 0 {
        return Err(BdlError::Texture);
    }
    let blocks_wide = width.div_ceil(8);
    match format {
        FORMAT_I4 | FORMAT_CMPR => Ok(blocks_wide * height.div_ceil(8) * 32),
        FORMAT_IA4 => Ok(blocks_wide * height.div_ceil(4) * 32),
        _ => Err(BdlError::Texture),
    }
}

fn read_textures(data: &[u8], blocks: &Blocks) -> Result<Vec<NativeTexture>, BdlError> {
    let Some((start, end)) = blocks.get(b"TEX1") else {
        return Ok(Vec::new());
    };
    let count = usize::from(read_u16(data, at(start, 8)?)?);
    let entries = at(start, read_u32(data, at(start, 0x0C)?)? as usize)?;
    let mut textures = Vec::with_capacity(count);
    for index in 0..count {
        let entry = at(entries, index * TEXTURE_STRIDE)?;
        if at(entry, TEXTURE_STRIDE)? > end {
            return Err(BdlError::Block);
        }
        let format = *data.get(entry).ok_or(BdlError::Truncated)?;
        let width = usize::from(read_u16(data, at(entry, 2)?)?);
        let height = usize::from(read_u16(data, at(entry, 4)?)?);
        let wrap_s = read_wrap(*data.get(at(entry, 6)?).ok_or(BdlError::Truncated)?)?;
        let wrap_t = read_wrap(*data.get(at(entry, 7)?).ok_or(BdlError::Truncated)?)?;
        let image = read_u32(data, at(entry, TEXTURE_IMAGE_OFFSET)?)?;
        let size = texture_size(format, width, height)?;
        let image_at = at(entry, if image == 0 { TEXTURE_STRIDE } else { image as usize })?;
        let image_end = at(image_at, size)?;
        if image_end > data.len() {
            return Err(BdlError::Truncated);
        }
        if image_end > end {
            return Err(BdlError::Block);
        }
        textures.push(NativeTexture {
            width,
            height,
            wrap_s,
            wrap_t,
            rgba: decode_texture(format, width, height, &data[image_at..image_end])?,
        });
    }
    Ok(textures)
}

fn decode_texture(
    format: u8,
    width: usize,
    height: usize,
    source: &[u8],
) -> Result<Vec<u8>, BdlError> {
    let mut rgba = vec![0_u8; width * height * 4];
    match format {
        FORMAT_I4 => decode_i4(width, height, source, &mut rgba)?,
        FORMAT_IA4 => decode_ia4(width, height, source, &mut rgba)?,
        FORMAT_CMPR => decode_cmpr(width, height, source, &mut rgba)?,
        _ => return Err(BdlError::Texture),
    }
    Ok(rgba)
}

fn decode_i4(width: usize, height: usize, source: &[u8], rgba: &mut [u8]) -> Result<(), BdlError> {
    let mut cursor = 0_usize;
    for block_y in (0..height).step_by(8) {
        for block_x in (0..width).step_by(8) {
            for row in 0..8 {
                let y = block_y + row;
                for pair in 0..4 {
                    let value = *source.get(cursor).ok_or(BdlError::Truncated)?;
                    cursor += 1;
                    if y >= height {
                        continue;
                    }
                    for half in 0..2 {
                        let x = block_x + pair * 2 + half;
                        if x >= width {
                            continue;
                        }
                        let level = if half == 0 { value >> 4 } else { value & 0xF };
                        let level = (level << 4) | level;
                        let at = (y * width + x) * 4;
                        rgba[at..at + 4].copy_from_slice(&[level, level, level, level]);
                    }
                }
            }
        }
    }
    Ok(())
}

fn decode_ia4(width: usize, height: usize, source: &[u8], rgba: &mut [u8]) -> Result<(), BdlError> {
    let mut cursor = 0_usize;
    for block_y in (0..height).step_by(4) {
        for block_x in (0..width).step_by(8) {
            for row in 0..4 {
                let y = block_y + row;
                for column in 0..8 {
                    let value = *source.get(cursor).ok_or(BdlError::Truncated)?;
                    cursor += 1;
                    if y >= height {
                        continue;
                    }
                    let x = block_x + column;
                    if x >= width {
                        continue;
                    }
                    let alpha = value >> 4;
                    let level = value & 0xF;
                    let alpha = (alpha << 4) | alpha;
                    let level = (level << 4) | level;
                    let at = (y * width + x) * 4;
                    rgba[at..at + 4].copy_from_slice(&[level, level, level, alpha]);
                }
            }
        }
    }
    Ok(())
}

fn decode_cmpr(width: usize, height: usize, source: &[u8], rgba: &mut [u8]) -> Result<(), BdlError> {
    let mut cursor = 0_usize;
    for block_y in (0..height).step_by(8) {
        for block_x in (0..width).step_by(8) {
            for sub in [[0, 0], [4, 0], [0, 4], [4, 4]] {
                let block = source
                    .get(cursor..cursor + 8)
                    .ok_or(BdlError::Truncated)?;
                cursor += 8;
                decode_dxt(
                    block,
                    block_x + sub[0],
                    block_y + sub[1],
                    width,
                    height,
                    rgba,
                );
            }
        }
    }
    Ok(())
}

fn decode_dxt(
    block: &[u8],
    origin_x: usize,
    origin_y: usize,
    width: usize,
    height: usize,
    rgba: &mut [u8],
) {
    let c1 = u16::from_be_bytes([block[0], block[1]]);
    let c2 = u16::from_be_bytes([block[2], block[3]]);
    let r1 = ((c1 >> 11) & 0x1F) as u8;
    let g1 = ((c1 >> 5) & 0x3F) as u8;
    let b1 = (c1 & 0x1F) as u8;
    let r2 = ((c2 >> 11) & 0x1F) as u8;
    let g2 = ((c2 >> 5) & 0x3F) as u8;
    let b2 = (c2 & 0x1F) as u8;
    let r1 = (r1 << 3) | (r1 >> 2);
    let g1 = (g1 << 2) | (g1 >> 4);
    let b1 = (b1 << 3) | (b1 >> 2);
    let r2 = (r2 << 3) | (r2 >> 2);
    let g2 = (g2 << 2) | (g2 >> 4);
    let b2 = (b2 << 3) | (b2 >> 2);
    let blend = |first: u8, second: u8| {
        ((u16::from(first) * 3 + u16::from(second) * 5) >> 3) as u8
    };
    let average = |first: u8, second: u8| ((u16::from(first) + u16::from(second)) / 2) as u8;
    let mut colors = [[0_u8; 4]; 4];
    colors[0] = [r1, g1, b1, 255];
    colors[1] = [r2, g2, b2, 255];
    if c1 > c2 {
        colors[2] = [blend(r2, r1), blend(g2, g1), blend(b2, b1), 255];
        colors[3] = [blend(r1, r2), blend(g1, g2), blend(b1, b2), 255];
    } else {
        colors[2] = [average(r1, r2), average(g1, g2), average(b1, b2), 255];
        colors[3] = [average(r1, r2), average(g1, g2), average(b1, b2), 0];
    }
    for row in 0..4 {
        let bits = block[4 + row];
        for column in 0..4 {
            let x = origin_x + column;
            let y = origin_y + row;
            if x >= width || y >= height {
                continue;
            }
            let index = usize::from((bits >> (6 - column * 2)) & 3);
            let at = (y * width + x) * 4;
            rgba[at..at + 4].copy_from_slice(&colors[index]);
        }
    }
}

fn wrap_index(index: i64, dim: i64, wrap: Wrap) -> i64 {
    match wrap {
        Wrap::Repeat => index.rem_euclid(dim),
        Wrap::Clamp => index.clamp(0, dim - 1),
        Wrap::Mirror => {
            let period = dim * 2;
            let wrapped = index.rem_euclid(period);
            if wrapped >= dim {
                period - 1 - wrapped
            } else {
                wrapped
            }
        }
    }
}

fn axis_samples(lo: f32, hi: f32, footprint: f32, dim: usize, wrap: Wrap) -> Vec<i64> {
    let dim_i = dim as i64;
    if footprint <= 1.0 {
        let center = (lo + hi) * 0.5;
        return vec![wrap_index((center * dim as f32).floor() as i64, dim_i, wrap)];
    }
    let start = (lo * dim as f32).floor() as i64;
    let stop = (hi * dim as f32).ceil() as i64;
    let count = (stop - start).max(1);
    let stride = ((count + BOX_SAMPLES as i64 - 1) / BOX_SAMPLES as i64).max(1);
    let mut samples = Vec::new();
    for step in 0..BOX_SAMPLES as i64 {
        let index = start + step * stride;
        if index >= stop {
            break;
        }
        samples.push(wrap_index(index, dim_i, wrap));
    }
    if samples.is_empty() {
        samples.push(wrap_index(start, dim_i, wrap));
    }
    samples
}

fn span_scale(span: f32) -> f32 {
    if span > 0.0 {
        1.0 / span
    } else {
        f32::INFINITY
    }
}

fn bake_textures(mesh: &mut RoomMesh, native: &[NativeTexture]) {
    let mut ranges = vec![[f32::MAX, f32::MIN, f32::MAX, f32::MIN]; native.len()];
    for group in &mesh.groups {
        let Some(page) = group.texture_page else {
            continue;
        };
        let range = &mut ranges[page as usize];
        for uv in &group.uvs {
            range[0] = range[0].min(uv[0]);
            range[1] = range[1].max(uv[0]);
            range[2] = range[2].min(uv[1]);
            range[3] = range[3].max(uv[1]);
        }
    }
    let layer = LAYER_SIZE as f32;
    let mut textures = Vec::with_capacity(native.len());
    let mut placements = Vec::with_capacity(native.len());
    for (index, texture) in native.iter().enumerate() {
        let range = ranges[index];
        let (u0, u1, v0, v1) = if range[0] <= range[1] && range[2] <= range[3] {
            (range[0], range[1], range[2], range[3])
        } else {
            (0.0, 1.0, 0.0, 1.0)
        };
        let scale_u = ((texture.width as f32) / layer).min(span_scale(u1 - u0));
        let scale_v = ((texture.height as f32) / layer).min(span_scale(v1 - v0));
        let tile_u = scale_u * layer;
        let tile_v = scale_v * layer;
        let width = (((u1 - u0) * tile_u).ceil() as u32 + 1)
            .min(LAYER_SIZE)
            .max(1);
        let height = (((v1 - v0) * tile_v).ceil() as u32 + 1)
            .min(LAYER_SIZE)
            .max(1);
        textures.push(RoomTexture {
            width,
            height,
            rgba: bake_pixels(texture, u0, v0, tile_u, tile_v, width, height),
        });
        placements.push([scale_u, scale_v, u0, v0]);
    }
    for group in &mut mesh.groups {
        let Some(page) = group.texture_page else {
            continue;
        };
        let placement = placements[page as usize];
        for uv in &mut group.uvs {
            uv[0] = (uv[0] - placement[2]) * placement[0];
            uv[1] = (uv[1] - placement[3]) * placement[1];
        }
    }
    mesh.textures = textures;
    mesh.uv_placements = placements;
}

fn bake_pixels(
    texture: &NativeTexture,
    u0: f32,
    v0: f32,
    tile_u: f32,
    tile_v: f32,
    width: u32,
    height: u32,
) -> Vec<u8> {
    let mut rgba = vec![0_u8; width as usize * height as usize * 4];
    let footprint_u = texture.width as f32 / tile_u;
    let footprint_v = texture.height as f32 / tile_v;
    for y in 0..height {
        let lo = v0 + y as f32 / tile_v;
        let hi = v0 + (y + 1) as f32 / tile_v;
        let rows = axis_samples(lo, hi, footprint_v, texture.height, texture.wrap_t);
        for x in 0..width {
            let lo = u0 + x as f32 / tile_u;
            let hi = u0 + (x + 1) as f32 / tile_u;
            let columns = axis_samples(lo, hi, footprint_u, texture.width, texture.wrap_s);
            let mut sum = [0_u32; 4];
            for &row in &rows {
                for &column in &columns {
                    let at = (row as usize * texture.width + column as usize) * 4;
                    for channel in 0..4 {
                        sum[channel] += u32::from(texture.rgba[at + channel]);
                    }
                }
            }
            let samples = (rows.len() * columns.len()) as u32;
            let at = (y as usize * width as usize + x as usize) * 4;
            for channel in 0..4 {
                rgba[at + channel] = ((sum[channel] + samples / 2) / samples) as u8;
            }
        }
    }
    rgba
}

struct ShapeContext<'a> {
    data: &'a [u8],
    vertices: &'a VertexArrays,
    origin: [f32; 2],
    sin: f32,
    cos: f32,
    animation: Option<&'a crate::j3d::btk::TextureAnimation>,
}

struct PrimitiveLayout {
    attributes: Vec<u32>,
    widths: Vec<usize>,
    stride: usize,
    position_field: usize,
    normal_field: Option<usize>,
    color_field: Option<usize>,
    texcoord_field: Option<usize>,
    matrix_field: Option<usize>,
}

fn descriptor_width(kind: u32) -> Result<usize, BdlError> {
    usize::try_from(kind)
        .ok()
        .and_then(|kind| DESCRIPTOR_WIDTHS.get(kind).copied())
        .ok_or(BdlError::Descriptor)
}

impl PrimitiveLayout {
    fn parse(data: &[u8], table: usize, offset: usize) -> Result<Self, BdlError> {
        let mut attributes = Vec::new();
        let mut widths = Vec::new();
        let mut cursor = at(table, offset)?;
        loop {
            let attribute = read_u32(data, cursor)?;
            let kind = read_u32(data, at(cursor, 4)?)?;
            cursor += DESCRIPTOR_STRIDE;
            if attribute == ATTRIBUTE_TERM {
                break;
            }
            widths.push(descriptor_width(kind)?);
            attributes.push(attribute);
        }
        let stride = widths.iter().sum();
        let find = |wanted: u32| attributes.iter().position(|&a| a == wanted);
        let position_field = find(ATTRIBUTE_POSITION).ok_or(BdlError::Descriptor)?;
        let normal_field = find(ATTRIBUTE_NORMAL);
        let color_field = find(ATTRIBUTE_COLOR0);
        let texcoord_field = find(ATTRIBUTE_TEX0);
        let matrix_field = find(ATTRIBUTE_MATRIX);
        Ok(Self {
            attributes,
            widths,
            stride,
            position_field,
            normal_field,
            color_field,
            texcoord_field,
            matrix_field,
        })
    }

    fn field_at(&self, field: usize, record: &[u8]) -> Result<usize, BdlError> {
        let start = self.widths[..field].iter().sum::<usize>();
        match self.widths[field] {
            0 => Ok(0),
            1 => Ok(usize::from(
                *record.get(start).ok_or(BdlError::Truncated)?,
            )),
            _ => Ok(usize::from(read_u16(record, start)?)),
        }
    }
}

fn read_shapes(
    context: &ShapeContext<'_>,
    blocks: &Blocks,
    worlds: &[[f32; 3]],
    draw_joints: &[DrawBinding],
    envelopes: &[SkinBinding],
    material_nodes: &[(u16, Option<u16>)],
    materials: &[MaterialBinding],
    texture_count: usize,
    mesh: &mut RoomMesh,
) -> Result<(), BdlError> {
    let (shape_start, _) = blocks.find(b"SHP1")?;
    let data = context.data;
    let shape_count = usize::from(read_u16(data, shape_start + 8)?);
    let init_table = at(shape_start, read_u32(data, at(shape_start, 0x0C)?)? as usize)?;
    let index_table = at(shape_start, read_u32(data, at(shape_start, 0x10)?)? as usize)?;
    let descriptor_table = at(shape_start, read_u32(data, at(shape_start, 0x18)?)? as usize)?;
    let mtx_table = at(shape_start, read_u32(data, at(shape_start, SHAPE_INDEX_TABLE)?)? as usize)?;
    let display_table = at(shape_start, read_u32(data, at(shape_start, 0x20)?)? as usize)?;
    let matrix_table = at(shape_start, read_u32(data, at(shape_start, 0x24)?)? as usize)?;
    let draw_table = at(shape_start, read_u32(data, at(shape_start, 0x28)?)? as usize)?;
    let mut shape_materials = vec![None; shape_count];
    for &(shape, material) in material_nodes {
        let slot = usize::from(shape);
        if slot >= shape_count || shape_materials[slot].is_some() {
            return Err(BdlError::Hierarchy);
        }
        shape_materials[slot] = material;
    }
    let mut slots: Vec<Option<SkinBinding>> = Vec::new();

    for shape in 0..shape_count {
        let slot = usize::from(read_u16(data, at(index_table, shape * 2)?)?);
        let init = at(init_table, slot * SHAPE_INIT_STRIDE)?;
        let matrix_type = *data.get(init).ok_or(BdlError::Truncated)?;
        if matrix_type != SHAPE_MATRIX_DIRECT && matrix_type != SHAPE_MATRIX_MULTI {
            return Err(BdlError::Shape);
        }
        let group_count = usize::from(read_u16(data, at(init, 2)?)?);
        let descriptor_offset = usize::from(read_u16(data, at(init, 4)?)?);
        let matrix_slot = usize::from(read_u16(data, at(init, 6)?)?);
        let draw_slot = usize::from(read_u16(data, at(init, 8)?)?);

        let layout = PrimitiveLayout::parse(data, descriptor_table, descriptor_offset)?;
        let texcoord_field = layout.texcoord_field;
        let lit = match shape_materials[shape] {
            Some(index) if !materials.is_empty() => {
                materials.get(usize::from(index)).ok_or(BdlError::Range)?.lit
            }
            _ => false,
        };
        let mut texture_page = if materials.is_empty() || texture_count == 0 {
            None
        } else {
            match shape_materials[shape] {
                Some(index) => {
                    let binding = materials.get(usize::from(index)).ok_or(BdlError::Range)?;
                    match binding.texture {
                        Some(texture) => {
                            if usize::from(texture) >= texture_count {
                                return Err(BdlError::Range);
                            }
                            Some(u32::from(texture))
                        }
                        None => None,
                    }
                }
                None => None,
            }
        };
        if texture_page.is_some()
            && (texcoord_field.is_none() || context.vertices.texcoord_count == 0)
        {
            texture_page = None;
        }
        let textured = texture_page.is_some();
        let anim = match (texture_page, context.animation, shape_materials[shape]) {
            (Some(_), Some(animation), Some(index)) if !materials.is_empty() => {
                let binding = materials.get(usize::from(index)).ok_or(BdlError::Range)?;
                material_anim(animation, binding)
            }
            _ => None,
        };
        let mut group = RoomGroup {
            texture_page,
            lit,
            anim,
            ..RoomGroup::default()
        };

        for subgroup in 0..group_count {
            let matrix = at(matrix_table, (matrix_slot + subgroup) * SHAPE_GROUP_STRIDE)?;
            let draw_matrix = usize::from(read_u16(data, matrix)?);
            let (world, bindings) = match layout.matrix_field {
                Some(_) => {
                    let count = usize::from(read_u16(data, at(matrix, SHAPE_INIT_MATRIX_SLOTS)?)?);
                    let first = usize::try_from(read_u32(
                        data,
                        at(matrix, SHAPE_INIT_MATRIX_FIRST)?,
                    )?)
                    .map_err(|_| BdlError::Range)?;
                    if slots.len() < count {
                        slots.resize(count, None);
                    }
                    let mut resolved = Vec::with_capacity(count);
                    for slot in 0..count {
                        let entry = read_u16(data, at(mtx_table, first + slot * 2)?)?;
                        let binding = if entry == MATRIX_UNSET {
                            slots[slot].ok_or(BdlError::Shape)?
                        } else {
                            let binding = skin_binding(
                                entry,
                                draw_joints,
                                envelopes,
                                worlds.len(),
                            )?;
                            slots[slot] = Some(binding);
                            binding
                        };
                        resolved.push(binding);
                    }
                    ([0.0, 0.0, 0.0], resolved)
                }
                None => {
                    let entry = *draw_joints.get(draw_matrix).ok_or(BdlError::Range)?;
                    if entry.blended {
                        return Err(BdlError::Envelope);
                    }
                    let joint = usize::from(entry.target);
                    if joint >= worlds.len() {
                        return Err(BdlError::Range);
                    }
                    (worlds[joint], Vec::new())
                }
            };
            let draw = at(draw_table, (draw_slot + subgroup) * SHAPE_GROUP_STRIDE)?;
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
                &layout,
                &bindings,
                textured,
                world,
                &mut group,
            )?;
        }
        mesh.groups.push(group);
    }
    Ok(())
}

fn emit(group: &mut RoomGroup, base: u32, a: usize, b: usize, c: usize) {
    group
        .indices
        .extend([base + a as u32, base + c as u32, base + b as u32]);
}

fn normalize(vector: [f32; 3]) -> [f32; 3] {
    let length_squared =
        vector[0] * vector[0] + vector[1] * vector[1] + vector[2] * vector[2];
    if length_squared <= 0.0 {
        return vector;
    }
    let inverse = length_squared.sqrt().recip();
    [
        vector[0] * inverse,
        vector[1] * inverse,
        vector[2] * inverse,
    ]
}

fn decode_display_list(
    context: &ShapeContext<'_>,
    display: &[u8],
    layout: &PrimitiveLayout,
    bindings: &[SkinBinding],
    textured: bool,
    world: [f32; 3],
    group: &mut RoomGroup,
) -> Result<(), BdlError> {
    let data = context.data;
    let stride = layout.stride;
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
        let base = group.positions.len() as u32;
        for _ in 0..count {
            let bytes = display
                .get(cursor..cursor + stride)
                .ok_or(BdlError::Truncated)?;
            cursor += stride;
            let mut position_index = None;
            let mut normal_index = None;
            let mut color_index = None;
            let mut texcoord_index = None;
            let mut binding = SkinBinding::default();
            for field in 0..layout.attributes.len() {
                let index = layout.field_at(field, bytes)?;
                if field == layout.position_field {
                    position_index = Some(index);
                }
                if layout.normal_field == Some(field) {
                    normal_index = Some(index);
                }
                if layout.color_field == Some(field) {
                    color_index = Some(index);
                }
                if layout.texcoord_field == Some(field) {
                    texcoord_index = Some(index);
                }
                if layout.matrix_field == Some(field) {
                    binding = *bindings
                        .get(index / MATRIX_INDEX_DIVISOR as usize)
                        .ok_or(BdlError::Range)?;
                }
            }
            let position_index = position_index.ok_or(BdlError::Descriptor)?;
            if position_index >= context.vertices.position_count {
                return Err(BdlError::Range);
            }
            let color_index = match layout.color_field {
                Some(_) => {
                    let index = color_index.ok_or(BdlError::Descriptor)?;
                    if index >= context.vertices.color_count {
                        return Err(BdlError::Range);
                    }
                    Some(index)
                }
                None => None,
            };
            let uv = if textured {
                let index = texcoord_index.ok_or(BdlError::Descriptor)?;
                if index >= context.vertices.texcoord_count {
                    return Err(BdlError::Range);
                }
                let texcoord_at = context.vertices.texcoord_at + index * TEXCOORD_STRIDE;
                let scale = 1.0 / ((1_u32 << context.vertices.texcoord_frac) as f32);
                Some([
                    f32::from(read_i16(data, texcoord_at)?) * scale,
                    f32::from(read_i16(data, at(texcoord_at, TEXCOORD_STRIDE - 2)?)?) * scale,
                ])
            } else {
                None
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
            group.positions.push([
                lx * context.cos + lz * context.sin + context.origin[0],
                ly,
                -lx * context.sin + lz * context.cos + context.origin[1],
            ]);
            group.colors.push(match color_index {
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
            if layout.matrix_field.is_some() {
                group.joint_indices.push(binding.indices);
                group.joint_weights.push(binding.weights);
            }
            if layout.normal_field.is_some() {
                let index = normal_index.ok_or(BdlError::Descriptor)?;
                if index >= context.vertices.normal_count {
                    return Err(BdlError::Range);
                }
                let normal_at =
                    context.vertices.normal_at + index * context.vertices.normal_stride;
                let normal = if context.vertices.normal_stride == NORMAL_STRIDE_F32 {
                    [
                        read_f32(data, normal_at)?,
                        read_f32(data, normal_at + 4)?,
                        read_f32(data, normal_at + 8)?,
                    ]
                } else {
                    let scale = 1.0 / ((1_u32 << context.vertices.normal_frac) as f32);
                    [
                        f32::from(read_i16(data, normal_at)?) * scale,
                        f32::from(read_i16(data, normal_at + 2)?) * scale,
                        f32::from(read_i16(data, normal_at + 4)?) * scale,
                    ]
                };
                group.normals.push(normalize([
                    normal[0] * context.cos + normal[2] * context.sin,
                    normal[1],
                    -normal[0] * context.sin + normal[2] * context.cos,
                ]));
            }
            if let Some(uv) = uv {
                group.uvs.push(uv);
            }
        }
        if !indexed {
            continue;
        }
        match command {
            COMMAND_QUADS => {
                for vertex in (0..count).step_by(4) {
                    emit(group, base, vertex, vertex + 1, vertex + 2);
                    emit(group, base, vertex, vertex + 2, vertex + 3);
                }
            }
            COMMAND_TRIANGLES => {
                for vertex in (0..count).step_by(3) {
                    emit(group, base, vertex, vertex + 1, vertex + 2);
                }
            }
            COMMAND_STRIP => {
                for vertex in 0..count.saturating_sub(2) {
                    if vertex % 2 == 0 {
                        emit(group, base, vertex, vertex + 1, vertex + 2);
                    } else {
                        emit(group, base, vertex + 1, vertex, vertex + 2);
                    }
                }
            }
            COMMAND_FAN => {
                for vertex in 1..count.saturating_sub(1) {
                    emit(group, base, 0, vertex, vertex + 1);
                }
            }
            _ => return Err(BdlError::Command),
        }
    }
    Ok(())
}
