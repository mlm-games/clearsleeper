use clearsleeper::shot;

fn crc32(bytes: &[u8]) -> u32 {
    let mut table = [0u32; 256];
    for (n, slot) in table.iter_mut().enumerate() {
        let mut c = n as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
        }
        *slot = c;
    }
    let mut c = 0xFFFF_FFFFu32;
    for &b in bytes {
        c = table[((c ^ b as u32) & 0xFF) as usize] ^ (c >> 8);
    }
    c ^ 0xFFFF_FFFF
}

fn adler32(bytes: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for &x in bytes {
        a = (a + x as u32) % 65521;
        b = (a + b as u32) % 65521;
    }
    (b << 16) | a
}

fn chunk(kind: &[u8; 4], body: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(body.len() + 12);
    out.extend_from_slice(&(body.len() as u32).to_be_bytes());
    out.extend_from_slice(kind);
    out.extend_from_slice(body);
    let mut crc_input = Vec::with_capacity(4 + body.len());
    crc_input.extend_from_slice(kind);
    crc_input.extend_from_slice(body);
    out.extend_from_slice(&crc32(&crc_input).to_be_bytes());
    out
}

/// Writes an 8-bit RGBA PNG. The zlib stream uses stored (uncompressed) deflate
/// blocks, which is valid and needs no compressor.
fn write_png(path: &str, width: usize, height: usize, rgba: &[u8]) -> std::io::Result<()> {
    let mut raw = Vec::with_capacity(height * (1 + width * 4));
    for y in 0..height {
        raw.push(0); // filter: none
        let start = y * width * 4;
        raw.extend_from_slice(&rgba[start..start + width * 4]);
    }

    let mut z = vec![0x78, 0x01];
    let mut offset = 0usize;
    while offset < raw.len() {
        let take = (raw.len() - offset).min(65535);
        let final_block = offset + take >= raw.len();
        z.push(if final_block { 1 } else { 0 });
        z.extend_from_slice(&(take as u16).to_le_bytes());
        z.extend_from_slice(&(!(take as u16)).to_le_bytes());
        z.extend_from_slice(&raw[offset..offset + take]);
        offset += take;
    }
    z.extend_from_slice(&adler32(&raw).to_be_bytes());

    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&(width as u32).to_be_bytes());
    ihdr.extend_from_slice(&(height as u32).to_be_bytes());
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]);

    let mut png = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    png.extend_from_slice(&chunk(b"IHDR", &ihdr));
    png.extend_from_slice(&chunk(b"IDAT", &z));
    png.extend_from_slice(&chunk(b"IEND", &[]));
    std::fs::write(path, png)
}

fn main() {
    if clearsleeper::panes::load().is_none() {
        eprintln!("panes::load() failed; is CS_ASSETS set?");
        std::process::exit(1);
    }
    let rgba = match shot::file_select_rgba(600) {
        Ok(rgba) => rgba,
        Err(e) => {
            eprintln!("render failed: {e:#}");
            std::process::exit(1);
        }
    };
    let out = std::env::args().nth(1).unwrap_or_else(|| "fs.png".into());
    write_png(&out, shot::W as usize, shot::H as usize, &rgba).expect("write png");
    eprintln!("wrote {out}");
}
