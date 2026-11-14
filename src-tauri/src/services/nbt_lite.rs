//! Minimal read-only NBT parser — just enough to pull a few known fields out of a world's
//! `level.dat` (gzip-compressed big-endian NBT) for the Worlds manager. Not a general-purpose
//! NBT library: no writing, no SNBT, no region/chunk formats. Hand-rolled rather than pulling in
//! a new crate since `flate2` (gzip) is already a dependency and the format itself is tiny and
//! stable — same "hand-roll the one binary format we actually touch" pattern already used for
//! server pings / crash logs elsewhere in this backend.

use std::collections::HashMap;
use std::io::Read;

#[derive(Debug, Clone)]
pub enum NbtValue {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<i8>),
    String(String),
    List(Vec<NbtValue>),
    Compound(HashMap<String, NbtValue>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

impl NbtValue {
    pub fn as_compound(&self) -> Option<&HashMap<String, NbtValue>> {
        match self {
            NbtValue::Compound(m) => Some(m),
            _ => None,
        }
    }
    pub fn as_str(&self) -> Option<&str> {
        match self {
            NbtValue::String(s) => Some(s),
            _ => None,
        }
    }
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            NbtValue::Long(v) => Some(*v),
            NbtValue::Int(v) => Some(*v as i64),
            NbtValue::Short(v) => Some(*v as i64),
            NbtValue::Byte(v) => Some(*v as i64),
            _ => None,
        }
    }
    pub fn as_i32(&self) -> Option<i32> {
        self.as_i64().map(|v| v as i32)
    }
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            NbtValue::Byte(v) => Some(*v != 0),
            _ => None,
        }
    }
}

/// Convenience: `Data.LevelName` etc, walking nested compounds by key.
pub fn dig<'a>(root: &'a NbtValue, path: &[&str]) -> Option<&'a NbtValue> {
    let mut cur = root;
    for key in path {
        cur = cur.as_compound()?.get(*key)?;
    }
    Some(cur)
}

struct Cursor<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        let end = self.pos.checked_add(n)?;
        if end > self.buf.len() {
            return None;
        }
        let slice = &self.buf[self.pos..end];
        self.pos = end;
        Some(slice)
    }
    fn u8(&mut self) -> Option<u8> {
        self.take(1).map(|b| b[0])
    }
    fn i8(&mut self) -> Option<i8> {
        self.u8().map(|b| b as i8)
    }
    fn i16(&mut self) -> Option<i16> {
        self.take(2).map(|b| i16::from_be_bytes([b[0], b[1]]))
    }
    fn i32(&mut self) -> Option<i32> {
        self.take(4).map(|b| i32::from_be_bytes([b[0], b[1], b[2], b[3]]))
    }
    fn i64(&mut self) -> Option<i64> {
        self.take(8).and_then(|b| b.try_into().ok()).map(i64::from_be_bytes)
    }
    fn f32(&mut self) -> Option<f32> {
        self.take(4).and_then(|b| b.try_into().ok()).map(f32::from_be_bytes)
    }
    fn f64(&mut self) -> Option<f64> {
        self.take(8).and_then(|b| b.try_into().ok()).map(f64::from_be_bytes)
    }
    fn string(&mut self) -> Option<String> {
        let len = self.i16()? as u16 as usize;
        let bytes = self.take(len)?;
        // MUTF-8 vs UTF-8 differ only for the astral/null edge cases we don't hit here
        // (level names are ordinary player-typed text) — lossy UTF-8 is fine.
        Some(String::from_utf8_lossy(bytes).into_owned())
    }

    fn payload(&mut self, tag_type: u8) -> Option<NbtValue> {
        Some(match tag_type {
            1 => NbtValue::Byte(self.i8()?),
            2 => NbtValue::Short(self.i16()?),
            3 => NbtValue::Int(self.i32()?),
            4 => NbtValue::Long(self.i64()?),
            5 => NbtValue::Float(self.f32()?),
            6 => NbtValue::Double(self.f64()?),
            7 => {
                let len = self.i32()?.max(0) as usize;
                let mut v = Vec::with_capacity(len.min(1 << 20));
                for _ in 0..len {
                    v.push(self.i8()?);
                }
                NbtValue::ByteArray(v)
            }
            8 => NbtValue::String(self.string()?),
            9 => {
                let elem_type = self.u8()?;
                let len = self.i32()?.max(0) as usize;
                let mut v = Vec::with_capacity(len.min(1 << 16));
                if elem_type != 0 {
                    for _ in 0..len {
                        v.push(self.payload(elem_type)?);
                    }
                }
                NbtValue::List(v)
            }
            10 => {
                let mut map = HashMap::new();
                loop {
                    let child_type = self.u8()?;
                    if child_type == 0 {
                        break;
                    }
                    let name = self.string()?;
                    let value = self.payload(child_type)?;
                    map.insert(name, value);
                }
                NbtValue::Compound(map)
            }
            11 => {
                let len = self.i32()?.max(0) as usize;
                let mut v = Vec::with_capacity(len.min(1 << 20));
                for _ in 0..len {
                    v.push(self.i32()?);
                }
                NbtValue::IntArray(v)
            }
            12 => {
                let len = self.i32()?.max(0) as usize;
                let mut v = Vec::with_capacity(len.min(1 << 20));
                for _ in 0..len {
                    v.push(self.i64()?);
                }
                NbtValue::LongArray(v)
            }
            _ => return None,
        })
    }
}

/// Decompresses (gzip) and parses a full NBT document, returning the root compound's payload
/// (the root tag's own name, usually empty, is discarded).
pub fn parse_gzipped(bytes: &[u8]) -> Option<NbtValue> {
    let mut decoder = flate2::read::GzDecoder::new(bytes);
    let mut raw = Vec::new();
    decoder.read_to_end(&mut raw).ok()?;

    let mut cursor = Cursor { buf: &raw, pos: 0 };
    let root_type = cursor.u8()?;
    if root_type != 10 {
        return None;
    }
    let _root_name = cursor.string()?;
    cursor.payload(10)
}

fn tag_id(v: &NbtValue) -> u8 {
    match v {
        NbtValue::Byte(_) => 1,
        NbtValue::Short(_) => 2,
        NbtValue::Int(_) => 3,
        NbtValue::Long(_) => 4,
        NbtValue::Float(_) => 5,
        NbtValue::Double(_) => 6,
        NbtValue::ByteArray(_) => 7,
        NbtValue::String(_) => 8,
        NbtValue::List(_) => 9,
        NbtValue::Compound(_) => 10,
        NbtValue::IntArray(_) => 11,
        NbtValue::LongArray(_) => 12,
    }
}

fn write_string(out: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    out.extend_from_slice(&(bytes.len() as u16).to_be_bytes());
    out.extend_from_slice(bytes);
}

fn write_payload(out: &mut Vec<u8>, v: &NbtValue) {
    match v {
        NbtValue::Byte(b) => out.push(*b as u8),
        NbtValue::Short(s) => out.extend_from_slice(&s.to_be_bytes()),
        NbtValue::Int(i) => out.extend_from_slice(&i.to_be_bytes()),
        NbtValue::Long(l) => out.extend_from_slice(&l.to_be_bytes()),
        NbtValue::Float(f) => out.extend_from_slice(&f.to_be_bytes()),
        NbtValue::Double(d) => out.extend_from_slice(&d.to_be_bytes()),
        NbtValue::ByteArray(a) => {
            out.extend_from_slice(&(a.len() as i32).to_be_bytes());
            for b in a {
                out.push(*b as u8);
            }
        }
        NbtValue::String(s) => write_string(out, s),
        NbtValue::List(items) => {
            let elem_type = items.first().map(tag_id).unwrap_or(0);
            out.push(elem_type);
            out.extend_from_slice(&(items.len() as i32).to_be_bytes());
            for item in items {
                write_payload(out, item);
            }
        }
        NbtValue::Compound(map) => {
            for (name, value) in map {
                out.push(tag_id(value));
                write_string(out, name);
                write_payload(out, value);
            }
            out.push(0); // TAG_End
        }
        NbtValue::IntArray(a) => {
            out.extend_from_slice(&(a.len() as i32).to_be_bytes());
            for i in a {
                out.extend_from_slice(&i.to_be_bytes());
            }
        }
        NbtValue::LongArray(a) => {
            out.extend_from_slice(&(a.len() as i32).to_be_bytes());
            for l in a {
                out.extend_from_slice(&l.to_be_bytes());
            }
        }
    }
}

/// Re-serializes a root compound (as returned by `parse_gzipped`) and gzips it back —
/// round-trips vanilla `level.dat` files fine since we never drop unknown fields, only mutate
/// the couple of values the Worlds manager actually edits (see `worlds_service::rename_world`).
pub fn write_gzipped(root: &NbtValue) -> Option<Vec<u8>> {
    let mut raw = Vec::new();
    raw.push(10u8); // root is always TAG_Compound
    write_string(&mut raw, ""); // root tag is conventionally unnamed
    write_payload(&mut raw, root);

    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    std::io::Write::write_all(&mut encoder, &raw).ok()?;
    encoder.finish().ok()
}
