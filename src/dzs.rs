const CHUNK_LEN: usize = 12;
const MULT_LEN: usize = 0x0C;
const ROOM_GROUP_LEN: usize = 0x08;
const ROOM_INDEX_MASK: u8 = 0x3F;
const SEA_HALF_SPAN: f32 = 450_000.0;
const SEA_CELL_SIZE: f32 = 100_000.0;
const SEA_CHART_MIN: i32 = 1;
const SEA_CHART_MAX: i32 = 7;
const SEA_CHART_SIDE: i32 = 7;
const STAG_LEN: usize = 0x20;
const EXIT_LEN: usize = 0x0C;
const ENVR_LEN: usize = 0x08;
const COLO_LEN: usize = 0x0C;
const PALE_LEN: usize = 0x2C;
const VIRT_LEN: usize = 0x24;
const ACTOR_LEN: usize = 0x20;
const SCALED_ACTOR_LEN: usize = 0x24;
const EVENT_LEN: usize = 0x18;
const EVENT_NAME_LEN: usize = 15;
const ACTOR_NAME_LEN: usize = 8;
const SEA_ACTOR: &str = "sea";
pub const DEFAULT_WAVE_MAX: u8 = 10;
pub const NO_ROOM: i8 = -1;
const BOOT_TIME: f32 = 165.0;
const TIME_H_SCALE: f32 = 15.0;
const SCHEDULE: [(f32, f32, u8); 11] = [
    (0.0, 90.0, 5),
    (90.0, 105.0, 5),
    (105.0, 120.0, 0),
    (120.0, 150.0, 1),
    (150.0, 270.0, 2),
    (270.0, 285.0, 2),
    (285.0, 300.0, 3),
    (300.0, 315.0, 4),
    (315.0, 360.0, 5),
    (360.0, 360.0, 5),
    (360.0, 360.0, 5),
];

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaletteEntry {
    pub bg1_amb: [u8; 3],
    pub bg1_dif: [u8; 3],
    pub virt_idx: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StageActor {
    pub name: String,
    pub parameters: u32,
    pub pos: [f32; 3],
    pub angle: [i16; 3],
    pub set_id: u16,
    pub room: i8,
    pub scale: [f32; 3],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageExit {
    pub stage: String,
    pub start: u8,
    pub room: u8,
    pub wipe: u8,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StageHeader {
    pub time_h: i8,
    pub stage_type: u32,
    pub schbit_start: u8,
    pub schbit_sec: u8,
    pub schbit_enable: u8,
    pub far_plane: f32,
}

pub struct StageTables {
    placements: Vec<Placement>,
    room_groups: Vec<Vec<u8>>,
    stag: Option<StageHeader>,
    envr: Vec<[u8; 8]>,
    colo: Vec<[u8; 8]>,
    palettes: Vec<PaletteEntry>,
    skies: Vec<[u8; 3]>,
    actors: Vec<StageActor>,
    exits: Vec<StageExit>,
    events: Vec<String>,
    name: String,
    has_sea: bool,
}

type ChunkNode = ([u8; 4], u32, u32);

fn chunk_nodes(data: &[u8]) -> Result<Vec<ChunkNode>, DzsError> {
    let chunk_count = read_u32(data, 0)?;
    let nodes_end = table_end(4, chunk_count as usize, CHUNK_LEN)?;
    if nodes_end > data.len() {
        return Err(DzsError::BadTable);
    }
    let mut nodes = Vec::with_capacity(chunk_count as usize);
    for index in 0..chunk_count as usize {
        let at = 4 + index * CHUNK_LEN;
        nodes.push((
            [data[at], data[at + 1], data[at + 2], data[at + 3]],
            read_u32(data, at + 4)?,
            read_u32(data, at + 8)?,
        ));
    }
    Ok(nodes)
}

fn chunk_locate(nodes: &[ChunkNode], tag: &[u8; 4]) -> Option<(u32, u32)> {
    nodes
        .iter()
        .find(|(name, ..)| name == tag)
        .map(|(_, count, offset)| (*count, *offset))
}

fn layer_chunk(layer: u8) -> Option<[u8; 4]> {
    let digit = match layer {
        0..=9 => b'0' + layer,
        10 => b'a',
        11 => b'b',
        _ => return None,
    };
    Some([b'A', b'C', b'T', digit])
}

fn scene_chunk(layer: u8) -> Option<u8> {
    match layer {
        0..=9 => Some(b'0' + layer),
        10 => Some(b'a'),
        11 => Some(b'b'),
        _ => None,
    }
}

pub fn room_actors(data: &[u8], room: i8, layer: u8) -> Result<Vec<StageActor>, DzsError> {
    let nodes = chunk_nodes(data)?;
    let mut actors = Vec::new();
    for tag in [b"ACTR", b"TGOB"] {
        if let Some((count, offset)) = chunk_locate(&nodes, tag) {
            actors.extend(parse_actors(data, count, offset, room, false)?);
        }
    }
    for tag in [b"TGSC", b"DOOR"] {
        if let Some((count, offset)) = chunk_locate(&nodes, tag) {
            actors.extend(parse_actors(data, count, offset, room, true)?);
        }
    }
    if let Some(tag) = layer_chunk(layer) {
        if let Some((count, offset)) = chunk_locate(&nodes, &tag) {
            actors.extend(parse_actors(data, count, offset, room, false)?);
        }
    }
    for tag in [Some([b'S', b'C', b'O', b'B']), scene_chunk(layer).map(|digit| [b'S', b'C', b'O', digit])]
        .into_iter()
        .flatten()
    {
        if let Some((count, offset)) = chunk_locate(&nodes, &tag) {
            actors.extend(parse_actors(data, count, offset, room, true)?);
        }
    }
    if let Some((count, offset)) = chunk_locate(&nodes, b"TGDR") {
        actors.extend(parse_actors(data, count, offset, room, true)?);
    }
    Ok(actors)
}

pub fn parse(data: &[u8]) -> Result<StageTables, DzsError> {
    let nodes = chunk_nodes(data)?;
    let locate = |tag: &[u8; 4]| chunk_locate(&nodes, tag);
    let placements = match locate(b"MULT") {
        Some((count, offset)) => parse_placements(data, count, offset)?,
        None => Vec::new(),
    };
    let room_groups = match locate(b"RTBL") {
        Some((count, offset)) => parse_room_groups(data, count, offset)?,
        None => Vec::new(),
    };
    let stag = match locate(b"STAG") {
        Some((_, offset)) => Some(parse_stag(data, offset)?),
        None => None,
    };
    let envr = match locate(b"EnvR") {
        Some((count, offset)) => {
            parse_records::<[u8; 8]>(data, count, offset, ENVR_LEN, read_array)?
        }
        None => Vec::new(),
    };
    let colo = match locate(b"Colo") {
        Some((count, offset)) => {
            parse_records::<[u8; 8]>(data, count, offset, COLO_LEN, read_array)?
        }
        None => Vec::new(),
    };
    let palettes = match locate(b"Pale") {
        Some((count, offset)) => parse_records(data, count, offset, PALE_LEN, parse_palette)?,
        None => Vec::new(),
    };
    let skies = match locate(b"Virt") {
        Some((count, offset)) => {
            parse_records::<[u8; 3]>(data, count, offset, VIRT_LEN, |data, at| {
                read_array(data, at + 0x18)
            })?
        }
        None => Vec::new(),
    };
    let mut actors = match locate(b"ACTR") {
        Some((count, offset)) => parse_actors(data, count, offset, NO_ROOM, false)?,
        None => Vec::new(),
    };
    for tag in [b"TGSC", b"SCOB", b"DOOR", b"TGDR"] {
        if let Some((count, offset)) = locate(tag) {
            actors.extend(parse_actors(data, count, offset, NO_ROOM, true)?);
        }
    }
    let exits = match locate(b"SCLS") {
        Some((count, offset)) => parse_exits(data, count, offset)?,
        None => Vec::new(),
    };
    let events = match locate(b"EVNT") {
        Some((count, offset)) => parse_events(data, count, offset)?,
        None => Vec::new(),
    };
    let has_sea = actors.iter().any(|actor| actor.name == SEA_ACTOR);
    Ok(StageTables {
        placements,
        room_groups,
        stag,
        envr,
        colo,
        palettes,
        skies,
        actors,
        exits,
        events,
        name: String::new(),
        has_sea,
    })
}

fn parse_events(data: &[u8], count: u32, offset: u32) -> Result<Vec<String>, DzsError> {
    let end = table_end(offset as usize, count as usize, EVENT_LEN)?;
    if end > data.len() {
        return Err(DzsError::BadTable);
    }
    let mut events = Vec::with_capacity(count as usize);
    for index in 0..count as usize {
        let at = offset as usize + index * EVENT_LEN + 1;
        if at + EVENT_NAME_LEN > data.len() {
            return Err(DzsError::BadTable);
        }
        let raw: [u8; EVENT_NAME_LEN] = read_array(data, at)?;
        let stop = raw
            .iter()
            .position(|&byte| byte == 0)
            .unwrap_or(EVENT_NAME_LEN);
        events.push(String::from_utf8_lossy(&raw[..stop]).into_owned());
    }
    Ok(events)
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

fn parse_records<T>(
    data: &[u8],
    count: u32,
    offset: u32,
    stride: usize,
    mut read: impl FnMut(&[u8], usize) -> Result<T, DzsError>,
) -> Result<Vec<T>, DzsError> {
    let end = table_end(offset as usize, count as usize, stride)?;
    if end > data.len() {
        return Err(DzsError::BadTable);
    }
    let mut records = Vec::with_capacity(count as usize);
    for index in 0..count as usize {
        records.push(read(data, offset as usize + index * stride)?);
    }
    Ok(records)
}

fn parse_stag(data: &[u8], offset: u32) -> Result<StageHeader, DzsError> {
    let end = table_end(offset as usize, 1, STAG_LEN)?;
    if end > data.len() {
        return Err(DzsError::BadTable);
    }
    let at = offset as usize;
    let type_and_schbit = read_u32(data, at + 0x0C)?;
    let enable_and_far = read_u32(data, at + 0x10)?;
    Ok(StageHeader {
        time_h: read_u8(data, at + 0x0D)? as i8,
        stage_type: (type_and_schbit >> 16) & 7,
        schbit_start: (type_and_schbit >> 8) & 0xFF,
        schbit_sec: (type_and_schbit & 0xFF) as u8,
        schbit_enable: (enable_and_far >> 16) as u8,
        far_plane: (enable_and_far & 0xFFFF) as u16 as f32,
    })
}

fn parse_palette(data: &[u8], at: usize) -> Result<PaletteEntry, DzsError> {
    Ok(PaletteEntry {
        bg1_amb: read_array(data, at + 0x0C)?,
        bg1_dif: read_array(data, at + 0x0F)?,
        virt_idx: read_u8(data, at + 0x21)?,
    })
}

fn parse_exits(data: &[u8], count: u32, offset: u32) -> Result<Vec<StageExit>, DzsError> {
    let end = table_end(offset as usize, count as usize, EXIT_LEN)?;
    if end > data.len() {
        return Err(DzsError::BadTable);
    }
    let mut exits = Vec::with_capacity(count as usize);
    for index in 0..count as usize {
        let at = offset as usize + index * EXIT_LEN;
        let name: [u8; 8] = read_array(data, at)?;
        let stop = name
            .iter()
            .position(|&byte| byte == 0)
            .unwrap_or(name.len());
        exits.push(StageExit {
            stage: String::from_utf8_lossy(&name[..stop]).into_owned(),
            start: read_u8(data, at + 8)?,
            room: read_u8(data, at + 9)?,
            wipe: read_u8(data, at + 0xA)?,
        });
    }
    Ok(exits)
}

fn parse_actors(
    data: &[u8],
    count: u32,
    offset: u32,
    room: i8,
    scaled: bool,
) -> Result<Vec<StageActor>, DzsError> {
    let stride = if scaled { SCALED_ACTOR_LEN } else { ACTOR_LEN };
    let end = table_end(offset as usize, count as usize, stride)?;
    if end > data.len() {
        return Err(DzsError::BadTable);
    }
    let mut actors = Vec::with_capacity(count as usize);
    for index in 0..count as usize {
        let at = offset as usize + index * stride;
        let name: [u8; ACTOR_NAME_LEN] = read_array(data, at)?;
        let stop = name
            .iter()
            .position(|&byte| byte == 0)
            .unwrap_or(ACTOR_NAME_LEN);
        actors.push(StageActor {
            name: String::from_utf8_lossy(&name[..stop]).into_owned(),
            parameters: read_u32(data, at + 0x08)?,
            pos: [
                read_f32(data, at + 0x0C)?,
                read_f32(data, at + 0x10)?,
                read_f32(data, at + 0x14)?,
            ],
            angle: [
                read_i16(data, at + 0x18)?,
                read_i16(data, at + 0x1A)?,
                read_i16(data, at + 0x1C)?,
            ],
            set_id: read_u16(data, at + 0x1E)?,
            room,
            scale: if scaled {
                [
                    f32::from(data[at + 0x20]) * 0.01,
                    f32::from(data[at + 0x21]) * 0.01,
                    f32::from(data[at + 0x22]) * 0.01,
                ]
            } else {
                [1.0, 1.0, 1.0]
            },
        });
    }
    Ok(actors)
}

impl StageTables {
    pub fn actors(&self) -> &[StageActor] {
        &self.actors
    }

    pub fn event_names(&self) -> &[String] {
        &self.events
    }

    pub fn exit(&self, id: u8) -> Option<&StageExit> {
        self.exits.get(usize::from(id))
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_owned();
        self
    }

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

    pub fn wave_max(&self, room: u8) -> u8 {
        self.placement(room)
            .map_or(DEFAULT_WAVE_MAX, |entry| entry.wave_max)
    }

    pub fn has_sea_actor(&self) -> bool {
        self.has_sea
    }

    pub fn stag(&self) -> Option<StageHeader> {
        self.stag
    }

    pub fn boot_time(&self) -> f32 {
        match self.stag.map(|stag| stag.time_h) {
            Some(hours) if hours >= 0 => hours as f32 * TIME_H_SCALE,
            _ => BOOT_TIME,
        }
    }

    pub fn boot_sky(&self) -> Option<[f32; 4]> {
        let entry = self.boot_palette().and_then(|palette| {
            self.skies
                .get(usize::from(palette.virt_idx))
                .copied()
        })?;
        Some([
            entry[0] as f32 / 255.0,
            entry[1] as f32 / 255.0,
            entry[2] as f32 / 255.0,
            1.0,
        ])
    }

    pub fn boot_sea_colors(&self) -> Option<([f32; 3], [f32; 3])> {
        let palette = self.boot_palette()?;
        Some((
            bytes_to_unit(palette.bg1_amb),
            bytes_to_unit(palette.bg1_dif),
        ))
    }

    fn boot_palette(&self) -> Option<&PaletteEntry> {
        let schedule = SCHEDULE.iter().find(|(begin, end, _)| {
            self.boot_time() >= *begin && self.boot_time() <= *end
        })?;
        let envr = self.envr.first()?;
        let colo = self.colo.get(usize::from(envr[0]))?;
        self.palettes.get(usize::from(colo[usize::from(schedule.2)]))
    }
}

fn bytes_to_unit(bytes: [u8; 3]) -> [f32; 3] {
    [
        bytes[0] as f32 / 255.0,
        bytes[1] as f32 / 255.0,
        bytes[2] as f32 / 255.0,
    ]
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

fn read_u16(data: &[u8], at: usize) -> Result<u16, DzsError> {
    let bytes = data.get(at..at + 2).ok_or(DzsError::Truncated)?;
    Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
}

fn read_u8(data: &[u8], at: usize) -> Result<u8, DzsError> {
    data.get(at).copied().ok_or(DzsError::Truncated)
}

fn read_array<const N: usize>(data: &[u8], at: usize) -> Result<[u8; N], DzsError> {
    let bytes = data.get(at..at + N).ok_or(DzsError::Truncated)?;
    let mut out = [0_u8; N];
    out.copy_from_slice(bytes);
    Ok(out)
}
