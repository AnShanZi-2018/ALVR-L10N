//! Minimal TrueType/OpenType scanner: locate an embedded font blob inside an exe.

#[derive(Clone, Debug)]
pub struct FontBlob {
    pub offset: usize,
    pub length: usize,
    pub family: String,
}

fn u16_at(data: &[u8], pos: usize) -> Option<u16> {
    let bytes = data.get(pos..pos + 2)?;
    Some(u16::from_be_bytes([bytes[0], bytes[1]]))
}

fn u32_at(data: &[u8], pos: usize) -> Option<u32> {
    let bytes = data.get(pos..pos + 4)?;
    Some(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn read_name_table(data: &[u8], base: usize, offset: usize, length: usize) -> Option<String> {
    let table = base.checked_add(offset)?;
    let table_end = table.checked_add(length)?;
    let format = u16_at(data, table)?;
    let count = u16_at(data, table + 2)?;
    let string_offset = u16_at(data, table + 4)?;
    if format > 1 || count > 300 {
        return None;
    }

    let mut family: Option<String> = None;
    for i in 0..count {
        let record = table + 6 + i as usize * 12;
        let platform = u16_at(data, record)?;
        let name_id = u16_at(data, record + 6)?;
        let name_len = u16_at(data, record + 8)? as usize;
        let name_off = u16_at(data, record + 10)? as usize;
        if name_id != 1 {
            continue;
        }
        let start = table + string_offset as usize + name_off;
        if start + name_len > table_end || start + name_len > data.len() {
            continue;
        }
        let raw = &data[start..start + name_len];
        let text = if platform == 3 {
            let units: Vec<u16> = raw
                .chunks_exact(2)
                .map(|c| u16::from_be_bytes([c[0], c[1]]))
                .collect();
            String::from_utf16_lossy(&units)
        } else {
            raw.iter().map(|&b| b as char).collect()
        };
        if platform == 3 || family.is_none() {
            family = Some(text);
        }
    }
    family
}

/// Find the first embedded font whose family name starts with `wanted`.
pub fn find_blob(data: &[u8], wanted: &str) -> Option<FontBlob> {
    let mut pos = 0usize;
    while pos + 12 < data.len() {
        let signature = &data[pos..pos + 4];
        if signature != [0, 1, 0, 0] {
            pos += 1;
            continue;
        }

        if let Some(blob) = try_parse(data, pos, wanted) {
            return Some(blob);
        }
        pos += 4;
    }
    None
}

fn try_parse(data: &[u8], base: usize, wanted: &str) -> Option<FontBlob> {
    let num_tables = u16_at(data, base + 4)? as usize;
    if !(5..=60).contains(&num_tables) {
        return None;
    }

    let mut has = [false; 5]; // cmap, head, hhea, maxp, name
    let mut max_end = 0usize;
    let mut name_table = None;

    for i in 0..num_tables {
        let record = base + 12 + i * 16;
        let tag = data.get(record..record + 4)?;
        if !tag.iter().all(|b| b.is_ascii_graphic() || *b == b' ') {
            return None;
        }
        let offset = u32_at(data, record + 8)? as usize;
        let length = u32_at(data, record + 12)? as usize;
        if length == 0 || base.checked_add(offset)?.checked_add(length)? > data.len() {
            return None;
        }
        match tag {
            b"cmap" => has[0] = true,
            b"head" => has[1] = true,
            b"hhea" => has[2] = true,
            b"maxp" => has[3] = true,
            b"name" => {
                has[4] = true;
                name_table = Some((offset, length));
            }
            _ => {}
        }
        max_end = max_end.max(offset + length);
    }

    if !has.iter().all(|v| *v) {
        return None;
    }

    let (name_offset, name_length) = name_table?;
    let family = read_name_table(data, base, name_offset, name_length)?;
    if !family.starts_with(wanted) {
        return None;
    }

    Some(FontBlob {
        offset: base,
        length: (max_end + 3) & !3,
        family,
    })
}

