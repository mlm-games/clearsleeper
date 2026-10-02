const CHUNK_LEN: usize = 12;
const MULT_LEN: usize = 0x0C;
const ROOM_GROUP_LEN: usize = 0x08;
const ROOM_INDEX_MASK: u8 = 0x3F;
const SEA_HALF_SPAN: f32 = 450_000.0;
const SEA_CELL_SIZE: f32 = 100_000.0;
const SEA_CHART_MIN: i32 = 1;
const SEA_CHART_MAX: i32 = 7;
const SEA_CHART_SIDE: i32 = 7;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DzsError {
    Truncated,
    BadTable,
    BadRecord,
}

impl DzsError {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Truncated => "dzs truncated",
            Self::BadTable => "dzs table out of range",
            Self::BadRecord => "dzs record out of range",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placement {
    pub trans: [f32; 2],
    pub angle: i16,
    pub room: u8,
    pub wave_max: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StageTables {
    placements: Vec<Placement>,
    room_groups: Vec<Vec<u8>>,
}

pub fn parse(data: &[u8]) -> Result<StageTables, DzsError> {
    let chunk_count = read_u32(data, 0)?;
    let nodes_end = table_end(4, chunk_count as usize, CHUNK_LEN)?;
    if nodes_end > data.len() {
        return Err(DzsError::BadTable);
    }
    let mut mult = None;
    let mut rtbl = None;
    for index in 0..chunk_count as usize {
        let at = 4 + index * CHUNK_LEN;
        let tag = &data[at..at + 4];
        let count = read_u32(data, at + 4)?;
        let offset = read_u32(data, at + 8)?;
        if tag == b"MULT" && mult.is_none() {
            mult = Some((count, offset));
        }
        if tag == b"RTBL" && rtbl.is_none() {
            rtbl = Some((count, offset));
        }
    }
    let placements = match mult {
        Some((count, offset)) => parse_placements(data, count, offset)?,
        None => Vec::new(),
    };
    let room_groups = match rtbl {
        Some((count, offset)) => parse_room_groups(data, count, offset)?,
        None => Vec::new(),
    };
    Ok(StageTables {
        placements,
        room_groups,
    })
}

fn parse_placements(data: &[u8], count: u32, offset: u32) -> Result<Vec<Placement>, DzsError> {
    let end = table_end(offset as usize, count as usize, MULT_LEN)?;
    if end > data.len() {
        return Err(DzsError::BadTable);
    }
    let mut placements = Vec::with_capacity(count as usize);
    for index in 0..count as usize {
        let at = offset as usize + index * MULT_LEN;
        placements.push(Placement {
            trans: [read_f32(data, at)?, read_f32(data, at + 4)?],
            angle: read_i16(data, at + 8)?,
            room: read_u8(data, at + 0x0A)?,
            wave_max: read_u8(data, at + 0x0B)?,
        });
    }
    Ok(placements)
}

fn parse_room_groups(data: &[u8], count: u32, offset: u32) -> Result<Vec<Vec<u8>>, DzsError> {
    let end = table_end(offset as usize, count as usize, 4)?;
    if end > data.len() {
        return Err(DzsError::BadTable);
    }
    let mut room_groups = Vec::with_capacity(count as usize);
    for index in 0..count as usize {
        let at = offset as usize + index * 4;
        let record = read_u32(data, at)? as usize;
        let record_end = record
            .checked_add(ROOM_GROUP_LEN)
            .ok_or(DzsError::BadRecord)?;
        if record_end > data.len() {
            return Err(DzsError::BadRecord);
        }
        let rooms = read_u32(data, record + 4)? as usize;
        let rooms_end = rooms
            .checked_add(usize::from(data[record]))
            .ok_or(DzsError::BadRecord)?;
        if rooms_end > data.len() {
            return Err(DzsError::BadRecord);
        }
        room_groups.push(
            data[rooms..rooms_end]
                .iter()
                .map(|byte| byte & ROOM_INDEX_MASK)
                .collect(),
        );
    }
    Ok(room_groups)
}

impl StageTables {
    pub fn placements(&self) -> &[Placement] {
        &self.placements
    }

    pub fn room_groups(&self) -> &[Vec<u8>] {
        &self.room_groups
    }

    pub fn placement(&self, room: u8) -> Option<&Placement> {
        self.placements.iter().find(|entry| entry.room == room)
    }

    pub fn load_set(&self, room: u8) -> &[u8] {
        self.room_groups
            .get(usize::from(room))
            .map_or(&[], |group| group.as_slice())
    }

    pub fn room_at(&self, x: f32, z: f32) -> u8 {
        if self.placements.is_empty() {
            return 0;
        }
        let cell = |value: f32| ((value + SEA_HALF_SPAN) / SEA_CELL_SIZE).floor() as i32;
        let column = cell(x);
        let row = cell(z);
        if !(SEA_CHART_MIN..=SEA_CHART_MAX).contains(&column)
            || !(SEA_CHART_MIN..=SEA_CHART_MAX).contains(&row)
        {
            return 0;
        }
        ((row - SEA_CHART_MIN) * SEA_CHART_SIDE + (column - SEA_CHART_MIN) + 1) as u8
    }
}

fn table_end(offset: usize, count: usize, stride: usize) -> Result<usize, DzsError> {
    count
        .checked_mul(stride)
        .and_then(|bytes| offset.checked_add(bytes))
        .ok_or(DzsError::BadTable)
}

fn read_u32(data: &[u8], at: usize) -> Result<u32, DzsError> {
    let bytes = data.get(at..at + 4).ok_or(DzsError::Truncated)?;
    Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn read_f32(data: &[u8], at: usize) -> Result<f32, DzsError> {
    let bytes = data.get(at..at + 4).ok_or(DzsError::Truncated)?;
    Ok(f32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn read_i16(data: &[u8], at: usize) -> Result<i16, DzsError> {
    let bytes = data.get(at..at + 2).ok_or(DzsError::Truncated)?;
    Ok(i16::from_be_bytes([bytes[0], bytes[1]]))
}

fn read_u8(data: &[u8], at: usize) -> Result<u8, DzsError> {
    data.get(at).copied().ok_or(DzsError::Truncated)
}
