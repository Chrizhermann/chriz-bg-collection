//! Synthetic fake-game builder for integration tests.
//!
//! Writes a minimal Infinity Engine game directory that `weidu.exe --game <root>`
//! accepts: a `chitin.key` (KEY V1) indexing one uncompressed BIFF V1 with stub
//! resources, a one-string `dialog.tlk` at the root and under `lang/en_US/`, an
//! empty `override/`, and an `engine.lua`.
//!
//! Binary layouts (little-endian, all offsets absolute within the file) were
//! verified against WeiDU 24900; see the `bg-modding` skill reference
//! `weidu-testing.md`, section "Pattern 2: Synthetic fake game".
//!
//! The default resource set contains `OH6000.ARE` so WeiDU's `GAME_IS ~bg2ee~`
//! evaluates to true (game detection keys on that resref's presence in the KEY),
//! plus `SPELL.IDS` and `STATS.IDS` stubs so sibling-component predicates that
//! call `IDS_OF_SYMBOL` do not spam `get_ids_map` load errors.

use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

/// Resource type code for `.SPL` files.
pub const RES_TYPE_SPL: u16 = 1006;
/// Resource type code for `.IDS` files.
pub const RES_TYPE_IDS: u16 = 1008;
/// Resource type code for `.ARE` files.
pub const RES_TYPE_ARE: u16 = 1010;

/// Size of the KEY V1 header.
const KEY_HEADER_LEN: usize = 0x18;
/// Size of one KEY BIF-table entry (excluding the out-of-line name).
const KEY_BIF_ENTRY_LEN: usize = 12;
/// Size of one KEY resource-table entry.
const KEY_RES_ENTRY_LEN: usize = 14;
/// Size of the BIFF V1 header.
const BIF_HEADER_LEN: usize = 0x14;
/// Size of one BIFF variable-resource table entry.
const BIF_VAR_ENTRY_LEN: usize = 0x10;
/// Size of the TLK V1 header.
const TLK_HEADER_LEN: usize = 0x12;
/// Size of one TLK string entry.
const TLK_ENTRY_LEN: usize = 26;
/// BIF name as stored in the KEY (relative to the game root, forward slashes).
const BIF_NAME: &[u8] = b"data/fake.bif";
/// KEY BIF-entry location flag: `<root>/data`.
const BIF_LOCATION_DATA: u16 = 1;
/// Content of the generated `engine.lua`.
const ENGINE_LUA: &str = "engine_name = 'FakeGame'\n";

/// A BIFF-only resource to index in the fake game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resource {
    /// Resource name without extension: 1–8 ASCII bytes, stored uppercase in the KEY.
    pub resref: String,
    /// IE resource type code (see the `RES_TYPE_*` constants).
    pub res_type: u16,
    /// Raw file bytes stored in the BIF.
    pub payload: Vec<u8>,
}

/// A generated fake game directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FakeGame {
    /// The game root that was populated.
    pub root: PathBuf,
}

impl FakeGame {
    /// Build the default fake game (see [`FakeGame::default_resources`]) under `root`.
    ///
    /// `root` is created if missing; if it exists it must be an empty directory,
    /// otherwise `ErrorKind::AlreadyExists` is returned and nothing is written.
    pub fn build(root: &Path) -> io::Result<FakeGame> {
        Self::build_with(root, &[])
    }

    /// Same as [`FakeGame::build`], with `extra` BIFF-only resources appended after
    /// the defaults (KEY/BIF table order is defaults first, then `extra` in order).
    ///
    /// A resref longer than 8 bytes, empty, or non-ASCII yields
    /// `ErrorKind::InvalidInput` before anything is written.
    pub fn build_with(root: &Path, extra: &[Resource]) -> io::Result<FakeGame> {
        let mut resources = Self::default_resources();
        resources.extend_from_slice(extra);
        for res in &resources {
            validate_resref(&res.resref)?;
        }
        let bif = encode_bif(&resources)?;
        let key = encode_key(&resources, bif.len())?;
        let tlk = encode_tlk();

        ensure_empty_root(root)?;
        let game = FakeGame {
            root: root.to_path_buf(),
        };
        fs::create_dir_all(game.root.join("data"))?;
        fs::create_dir_all(game.root.join("lang").join("en_US"))?;
        fs::create_dir_all(game.override_dir())?;
        fs::write(game.bif_path(), &bif)?;
        fs::write(game.key_path(), &key)?;
        fs::write(game.root.join("dialog.tlk"), &tlk)?;
        fs::write(
            game.root.join("lang").join("en_US").join("dialog.tlk"),
            &tlk,
        )?;
        fs::write(game.root.join("engine.lua"), ENGINE_LUA)?;
        Ok(game)
    }

    /// The resources every fake game contains, in KEY/BIF table order:
    /// `OH6000.ARE`, `SPELL.IDS`, `STATS.IDS`.
    pub fn default_resources() -> Vec<Resource> {
        let mut area = b"AREAV1.0".to_vec();
        area.resize(area.len() + 0x11c, 0);
        vec![
            Resource {
                resref: "OH6000".to_owned(),
                res_type: RES_TYPE_ARE,
                payload: area,
            },
            Resource {
                resref: "SPELL".to_owned(),
                res_type: RES_TYPE_IDS,
                payload: b"IDS V1.0\r\n0 NONE\r\n".to_vec(),
            },
            Resource {
                resref: "STATS".to_owned(),
                res_type: RES_TYPE_IDS,
                payload: b"IDS V1.0\r\n0 NONE\r\n".to_vec(),
            },
        ]
    }

    /// `<root>/chitin.key`.
    pub fn key_path(&self) -> PathBuf {
        self.root.join("chitin.key")
    }

    /// `<root>/data/fake.bif`.
    pub fn bif_path(&self) -> PathBuf {
        self.root.join("data").join("fake.bif")
    }

    /// `<root>/override`.
    pub fn override_dir(&self) -> PathBuf {
        self.root.join("override")
    }

    /// Parse `<root>/chitin.key` and return `(resref, type, locator)` triples in
    /// resource-table order. The resref is returned without NUL padding.
    pub fn read_key_resources(&self) -> io::Result<Vec<(String, u16, u32)>> {
        let key = fs::read(self.key_path())?;
        parse_key_resources(&key)
    }
}

/// Reject resrefs the KEY cannot represent: empty, longer than 8 bytes, or non-ASCII.
fn validate_resref(resref: &str) -> io::Result<()> {
    if resref.is_empty() || resref.len() > 8 || !resref.is_ascii() {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            format!("resref {resref:?} must be 1-8 ASCII bytes"),
        ));
    }
    Ok(())
}

/// Ensure `root` is either absent (then create it) or an existing empty directory.
fn ensure_empty_root(root: &Path) -> io::Result<()> {
    match fs::read_dir(root) {
        Ok(mut entries) => {
            if entries.next().is_some() {
                return Err(io::Error::new(
                    ErrorKind::AlreadyExists,
                    format!("fake game root {} is not empty", root.display()),
                ));
            }
            Ok(())
        }
        Err(err) if err.kind() == ErrorKind::NotFound => fs::create_dir_all(root),
        Err(err) => Err(err),
    }
}

fn invalid_data(msg: impl Into<String>) -> io::Error {
    io::Error::new(ErrorKind::InvalidData, msg.into())
}

fn to_u32(value: usize, what: &str) -> io::Result<u32> {
    u32::try_from(value).map_err(|_| invalid_data(format!("{what} {value} exceeds u32")))
}

fn to_u16(value: usize, what: &str) -> io::Result<u16> {
    u16::try_from(value).map_err(|_| invalid_data(format!("{what} {value} exceeds u16")))
}

fn put_u16(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

/// KEY locator for a variable resource: `(bif_index << 20) | (tileset_index << 14) | var_index`.
/// With a single BIF (`bif_index == 0`) and no tilesets this is just the var index.
fn locator(var_index: usize) -> io::Result<u32> {
    let index = to_u32(var_index, "var index")?;
    if index >= 1 << 14 {
        return Err(invalid_data(format!(
            "var index {index} does not fit the 14-bit locator field"
        )));
    }
    Ok(index)
}

/// Encode an uncompressed BIFF V1 holding `resources` as variable entries.
///
/// Header (0x14): `BIFF`, `V1  `, u32 var count, u32 tileset count (0), u32 table
/// offset (0x14). Var entry (0x10): u32 locator, u32 payload offset, u32 size,
/// u16 type, u16 0. Payloads are laid out contiguously after the table.
fn encode_bif(resources: &[Resource]) -> io::Result<Vec<u8>> {
    let table_len = BIF_VAR_ENTRY_LEN * resources.len();
    let total: usize =
        BIF_HEADER_LEN + table_len + resources.iter().map(|r| r.payload.len()).sum::<usize>();
    let mut out = Vec::with_capacity(total);

    out.extend_from_slice(b"BIFF");
    out.extend_from_slice(b"V1  ");
    put_u32(&mut out, to_u32(resources.len(), "resource count")?);
    put_u32(&mut out, 0);
    put_u32(&mut out, to_u32(BIF_HEADER_LEN, "table offset")?);

    let mut payload_offset = BIF_HEADER_LEN + table_len;
    for (index, res) in resources.iter().enumerate() {
        put_u32(&mut out, locator(index)?);
        put_u32(&mut out, to_u32(payload_offset, "payload offset")?);
        put_u32(&mut out, to_u32(res.payload.len(), "payload size")?);
        put_u16(&mut out, res.res_type);
        put_u16(&mut out, 0);
        payload_offset += res.payload.len();
    }
    for res in resources {
        out.extend_from_slice(&res.payload);
    }
    Ok(out)
}

/// Encode a KEY V1 indexing a single BIF (`data/fake.bif`, `bif_len` bytes long)
/// that holds `resources` at var indices `0..n`.
///
/// Header (0x18): `KEY `, `V1  `, u32 BIF count (1), u32 resource count, u32
/// BIF-table offset (0x18), u32 resource-table offset. BIF entry (12): u32 BIF
/// file length, u32 name offset, u16 name length (incl. NUL), u16 location (1 =
/// `<root>/data`). The NUL-terminated name follows the BIF table immediately;
/// the resource table follows the name. Resource entry (14): 8-byte NUL-padded
/// uppercase resref, u16 type, u32 locator.
fn encode_key(resources: &[Resource], bif_len: usize) -> io::Result<Vec<u8>> {
    let name_len = BIF_NAME.len() + 1;
    let name_off = KEY_HEADER_LEN + KEY_BIF_ENTRY_LEN;
    let res_table_off = name_off + name_len;
    let mut out = Vec::with_capacity(res_table_off + KEY_RES_ENTRY_LEN * resources.len());

    out.extend_from_slice(b"KEY ");
    out.extend_from_slice(b"V1  ");
    put_u32(&mut out, 1);
    put_u32(&mut out, to_u32(resources.len(), "resource count")?);
    put_u32(&mut out, to_u32(KEY_HEADER_LEN, "bif table offset")?);
    put_u32(&mut out, to_u32(res_table_off, "resource table offset")?);

    put_u32(&mut out, to_u32(bif_len, "bif file length")?);
    put_u32(&mut out, to_u32(name_off, "bif name offset")?);
    put_u16(&mut out, to_u16(name_len, "bif name length")?);
    put_u16(&mut out, BIF_LOCATION_DATA);
    out.extend_from_slice(BIF_NAME);
    out.push(0);

    for (index, res) in resources.iter().enumerate() {
        let mut resref = [0u8; 8];
        for (dst, src) in resref.iter_mut().zip(res.resref.bytes()) {
            *dst = src.to_ascii_uppercase();
        }
        out.extend_from_slice(&resref);
        put_u16(&mut out, res.res_type);
        put_u32(&mut out, locator(index)?);
    }
    Ok(out)
}

/// Encode a one-empty-string TLK V1 (0x2C bytes).
///
/// Header (0x12): `TLK `, `V1  `, u16 language id (0), u32 string count (1), u32
/// string-data offset (0x2C). Entry (26): u16 flags, 8-byte sound resref, u32
/// volume, u32 pitch, u32 offset, u32 length — all zero.
fn encode_tlk() -> Vec<u8> {
    let mut out = Vec::with_capacity(TLK_HEADER_LEN + TLK_ENTRY_LEN);
    out.extend_from_slice(b"TLK ");
    out.extend_from_slice(b"V1  ");
    put_u16(&mut out, 0);
    put_u32(&mut out, 1);
    put_u32(&mut out, (TLK_HEADER_LEN + TLK_ENTRY_LEN) as u32);
    out.resize(TLK_HEADER_LEN + TLK_ENTRY_LEN, 0);
    out
}

fn read_u16(bytes: &[u8], off: usize, what: &str) -> io::Result<u16> {
    let raw = bytes
        .get(off..off + 2)
        .ok_or_else(|| invalid_data(format!("KEY truncated reading {what} at {off:#x}")))?;
    Ok(u16::from_le_bytes([raw[0], raw[1]]))
}

fn read_u32(bytes: &[u8], off: usize, what: &str) -> io::Result<u32> {
    let raw = bytes
        .get(off..off + 4)
        .ok_or_else(|| invalid_data(format!("KEY truncated reading {what} at {off:#x}")))?;
    Ok(u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
}

/// Parse the resource table of a KEY V1 image into `(resref, type, locator)` triples.
fn parse_key_resources(key: &[u8]) -> io::Result<Vec<(String, u16, u32)>> {
    if key.get(0..8) != Some(b"KEY V1  ") {
        return Err(invalid_data("not a KEY V1 file"));
    }
    let res_count = read_u32(key, 12, "resource count")? as usize;
    let res_table_off = read_u32(key, 20, "resource table offset")? as usize;

    let mut resources = Vec::with_capacity(res_count);
    for index in 0..res_count {
        let entry = res_table_off + KEY_RES_ENTRY_LEN * index;
        let resref_raw = key
            .get(entry..entry + 8)
            .ok_or_else(|| invalid_data(format!("KEY truncated at resource entry {index}")))?;
        let resref_len = resref_raw.iter().position(|&b| b == 0).unwrap_or(8);
        let resref = std::str::from_utf8(&resref_raw[..resref_len])
            .map_err(|_| invalid_data(format!("resource entry {index}: resref is not UTF-8")))?
            .to_owned();
        let res_type = read_u16(key, entry + 8, "resource type")?;
        let locator = read_u32(key, entry + 10, "resource locator")?;
        resources.push((resref, res_type, locator));
    }
    Ok(resources)
}
