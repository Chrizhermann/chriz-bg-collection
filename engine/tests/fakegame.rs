//! Self-test of the synthetic fake-game builder in `support::fakegame`.
//!
//! Every test builds into its own unique directory under the OS temp dir and
//! removes it again when the test ends (including on assertion failure).

mod support;

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use support::fakegame::{FakeGame, Resource, RES_TYPE_ARE, RES_TYPE_IDS, RES_TYPE_SPL};

/// A unique scratch directory that is removed on drop.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!(
            "bg-engine-fakegame-{name}-{}-{nanos}",
            std::process::id()
        ));
        Scratch(dir)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn u16_at(bytes: &[u8], off: usize) -> u16 {
    u16::from_le_bytes([bytes[off], bytes[off + 1]])
}

fn u32_at(bytes: &[u8], off: usize) -> u32 {
    u32::from_le_bytes([bytes[off], bytes[off + 1], bytes[off + 2], bytes[off + 3]])
}

#[test]
fn builds_expected_tree() {
    let scratch = Scratch::new("tree");
    let game = FakeGame::build(scratch.path()).unwrap();

    assert_eq!(game.root, scratch.path());
    assert!(game.key_path().is_file(), "chitin.key missing");
    assert_eq!(game.key_path(), scratch.path().join("chitin.key"));
    assert!(game.bif_path().is_file(), "data/fake.bif missing");
    assert_eq!(
        game.bif_path(),
        scratch.path().join("data").join("fake.bif")
    );
    assert!(scratch.path().join("data").is_dir());
    assert!(scratch.path().join("lang").join("en_US").is_dir());

    let override_dir = game.override_dir();
    assert_eq!(override_dir, scratch.path().join("override"));
    assert!(override_dir.is_dir(), "override/ missing");
    assert_eq!(fs::read_dir(&override_dir).unwrap().count(), 0);

    let lua = fs::read_to_string(scratch.path().join("engine.lua")).unwrap();
    assert_eq!(lua, "engine_name = 'FakeGame'\n");

    let root_tlk = fs::read(scratch.path().join("dialog.tlk")).unwrap();
    let lang_tlk = fs::read(scratch.path().join("lang").join("en_US").join("dialog.tlk")).unwrap();
    assert_eq!(root_tlk.len(), 0x2C);
    assert_eq!(root_tlk, lang_tlk);
    assert_eq!(&root_tlk[0..8], b"TLK V1  ");
    assert_eq!(u16_at(&root_tlk, 8), 0, "lang id");
    assert_eq!(u32_at(&root_tlk, 10), 1, "num strings");
    assert_eq!(u32_at(&root_tlk, 14), 0x2C, "string offset");
    assert!(
        root_tlk[18..].iter().all(|&b| b == 0),
        "entry must be all zeros"
    );
}

#[test]
fn key_header_and_tables_are_consistent() {
    let scratch = Scratch::new("key");
    let game = FakeGame::build(scratch.path()).unwrap();
    let key = fs::read(game.key_path()).unwrap();

    assert_eq!(&key[0..8], b"KEY V1  ");
    let bif_count = u32_at(&key, 8);
    let res_count = u32_at(&key, 12);
    let bif_table_off = u32_at(&key, 16) as usize;
    let res_table_off = u32_at(&key, 20) as usize;
    assert_eq!(bif_count, 1);
    assert_eq!(res_count, 3);
    assert_eq!(bif_table_off, 0x18);

    let bif_file_length = u32_at(&key, bif_table_off) as u64;
    let name_off = u32_at(&key, bif_table_off + 4) as usize;
    let name_len = u16_at(&key, bif_table_off + 8) as usize;
    let location = u16_at(&key, bif_table_off + 10);
    assert_eq!(name_off, 0x18 + 12);
    assert_eq!(name_len, b"data/fake.bif\0".len());
    assert_eq!(&key[name_off..name_off + name_len], b"data/fake.bif\0");
    assert_eq!(location, 1);
    assert_eq!(res_table_off, 0x18 + 12 + name_len);
    assert_eq!(key.len(), res_table_off + 14 * res_count as usize);

    let bif_size = fs::metadata(game.bif_path()).unwrap().len();
    assert_eq!(bif_file_length, bif_size);

    let resources = game.read_key_resources().unwrap();
    assert_eq!(
        resources,
        vec![
            ("OH6000".to_owned(), RES_TYPE_ARE, 0),
            ("SPELL".to_owned(), RES_TYPE_IDS, 1),
            ("STATS".to_owned(), RES_TYPE_IDS, 2),
        ]
    );
}

#[test]
fn bif_payloads_round_trip() {
    let scratch = Scratch::new("bif");
    let game = FakeGame::build(scratch.path()).unwrap();
    let bif = fs::read(game.bif_path()).unwrap();
    let defaults = FakeGame::default_resources();
    let key_entries = game.read_key_resources().unwrap();

    assert_eq!(&bif[0..8], b"BIFFV1  ");
    let var_count = u32_at(&bif, 8) as usize;
    let tileset_count = u32_at(&bif, 12);
    let table_off = u32_at(&bif, 16) as usize;
    assert_eq!(var_count, defaults.len());
    assert_eq!(var_count, key_entries.len());
    assert_eq!(tileset_count, 0);
    assert_eq!(table_off, 0x14);

    let mut expected_offset = table_off + 0x10 * var_count;
    for (i, expected) in defaults.iter().enumerate() {
        let entry = table_off + 0x10 * i;
        let locator = u32_at(&bif, entry);
        let offset = u32_at(&bif, entry + 4) as usize;
        let size = u32_at(&bif, entry + 8) as usize;
        let res_type = u16_at(&bif, entry + 12);
        let reserved = u16_at(&bif, entry + 14);

        assert_eq!(locator, key_entries[i].2, "locator of entry {i}");
        assert_eq!(locator, i as u32, "bif index 0 => locator == var index");
        assert_eq!(offset, expected_offset, "payload offset of entry {i}");
        assert_eq!(size, expected.payload.len(), "size of entry {i}");
        assert_eq!(res_type, expected.res_type, "type of entry {i}");
        assert_eq!(res_type, key_entries[i].1);
        assert_eq!(reserved, 0);
        assert_eq!(&bif[offset..offset + size], &expected.payload[..]);
        assert_eq!(key_entries[i].0, expected.resref);
        expected_offset += size;
    }
    assert_eq!(
        bif.len(),
        expected_offset,
        "payloads must be contiguous to EOF"
    );

    assert_eq!(&defaults[0].payload[0..8], b"AREAV1.0");
    assert_eq!(defaults[0].payload.len(), 8 + 0x11c);
    assert_eq!(&defaults[1].payload[..], b"IDS V1.0\r\n0 NONE\r\n");
}

#[test]
fn build_with_appends_extra_resources() {
    let scratch = Scratch::new("extra");
    let extra = Resource {
        resref: "TESTSPL".to_owned(),
        res_type: RES_TYPE_SPL,
        payload: b"SPL V1  ...".to_vec(),
    };
    let game = FakeGame::build_with(scratch.path(), &[extra]).unwrap();

    let resources = game.read_key_resources().unwrap();
    assert_eq!(resources.len(), 4);
    assert_eq!(resources[3], ("TESTSPL".to_owned(), RES_TYPE_SPL, 3));

    let bif = fs::read(game.bif_path()).unwrap();
    assert_eq!(u32_at(&bif, 8), 4, "bif var count");
    let entry = 0x14 + 0x10 * 3;
    let offset = u32_at(&bif, entry + 4) as usize;
    let size = u32_at(&bif, entry + 8) as usize;
    assert_eq!(u32_at(&bif, entry), 3);
    assert_eq!(u16_at(&bif, entry + 12), RES_TYPE_SPL);
    assert_eq!(&bif[offset..offset + size], b"SPL V1  ...");

    let key = fs::read(game.key_path()).unwrap();
    assert_eq!(
        u32_at(&key, 0x18) as u64,
        fs::metadata(game.bif_path()).unwrap().len()
    );
}

#[test]
fn refuses_nonempty_root() {
    let scratch = Scratch::new("nonempty");
    fs::create_dir_all(scratch.path()).unwrap();
    fs::write(scratch.path().join("stale.txt"), b"x").unwrap();

    let err = FakeGame::build(scratch.path()).unwrap_err();
    assert_eq!(err.kind(), ErrorKind::AlreadyExists);
    assert!(!scratch.path().join("chitin.key").exists());
}

#[test]
fn rejects_invalid_resrefs() {
    for (resref, why) in [
        ("TOOLONG12", "9 bytes"),
        ("\u{c4}RGER", "non-ASCII"),
        ("", "empty"),
    ] {
        let scratch = Scratch::new("badresref");
        let extra = Resource {
            resref: resref.to_owned(),
            res_type: RES_TYPE_SPL,
            payload: vec![0],
        };
        let err = FakeGame::build_with(scratch.path(), &[extra]).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidInput, "{why}");
        assert!(!scratch.path().join("chitin.key").exists(), "{why}");
    }
}
