//! Audited BuffBot 1.8.3 -> 1.8.4 adapter. This is not a generic WeiDU updater.
//!
//! Source inventory/digests come from owner tags 2c3c7e9de49d95e0118d3396a9f90fd341e7bebf
//! and 9ed6ba0736bdae365b33d473b5fc2c03c9b247db. No mod contents are bundled here.
//! The pilot only accepts a fresh, English install whose helper did not change the
//! loader. Other uninstall metadata needs a separately reviewed adapter.
use crate::patches::files;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

pub(crate) const OLD_VERSION: &str = "1.8.3-alpha";
pub(crate) const NEW_VERSION: &str = "1.8.4-alpha";
pub(crate) const OLD_HASH: &str =
    "fe6e0b4cd393a608d8c3df75f26993106f3550e954030a7b66dbfd28956820a7";
pub(crate) const NEW_HASH: &str =
    "1081abc59511263570603560059ef361f45dd3288933f7faa26166ce0e805ced";
pub(crate) const RUN_ID: &str = "buffbot-bg2";
pub(crate) const UPGRADE_ID: &str = "buffbot-1.8.3-to-1.8.4";
// Fingerprints of the published ZIP payloads (OLD_HASH / NEW_HASH), using
// validate_source's lowercase, ordinal path order and binary u64 lengths.
// Verify these with published_archives_match_reviewed_sources before packaging.
const OLD_TREE: &str = "0d3a2210e1787d26493eaa75a5099a7a1d4f6037bff9ee028e8b70a74bc76710";
const NEW_TREE: &str = "24091c2b6b42f1eace4b49dcbb8fc0c8237c4b78ecf6649a850b068c4300c490";
const FILE_LIMIT: u64 = 16 * 1024 * 1024;
const TLK_LIMIT: u64 = 256 * 1024 * 1024;
pub(crate) const TLK_PATH: &str = "lang/en_us/dialog.tlk";
pub(crate) const STRREF_PATH: &str = "override/bfbot_strrefs.txt";
const BACKUP: &str = "weidu_external/backup/buffbot";

// These are literal COPY inputs, not prefix globs. The selected catalog and
// generated strref file bring the installed override-file count to 33.
const COPIED: &[&str] = &[
    "m_bfbot.lua",
    "bfbotcor.lua",
    "bfbotloc.lua",
    "bfbotcls.lua",
    "bfbotscn.lua",
    "bfbotexe.lua",
    "bfbotmp.lua",
    "bfbotper.lua",
    "bfbotinn.lua",
    "bfbotui.lua",
    "bfbotthm.lua",
    "bfbottst.lua",
    "buffbot.menu",
    "bfbotib.bam",
    "bfbotbg.mos",
    "mos9900.pvrz",
    "mos9901.pvrz",
    "mos9902.pvrz",
    "mos9903.pvrz",
    "bfbotfr.pvrz",
    "bfbotab.bam",
    "mos9910.pvrz",
    "mos9911.pvrz",
    "mos9912.pvrz",
    "mos9913.pvrz",
    "bfbotfr2.pvrz",
    "mos9920.pvrz",
    "mos9921.pvrz",
    "mos9922.pvrz",
    "mos9923.pvrz",
    "bfbotfr3.pvrz",
];
const LANGUAGES: &[&str] = &[
    "brazilian_portuguese",
    "english",
    "french",
    "german",
    "italian",
    "polish",
    "russian",
    "schinese",
    "spanish",
];
const METADATA: &[&str] = &[
    "args",
    "args-text",
    "mappings",
    "move",
    "other",
    "readln",
    "readln-text",
    "tlkpath",
    "uninstall",
];
const EMPTY_LIST: &[u8] = &[
    0x84, 0x95, 0xa6, 0xbe, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x40,
];

fn source_names() -> BTreeSet<String> {
    let mut names: BTreeSet<_> = COPIED.iter().map(|n| format!("buffbot/{n}")).collect();
    names.insert("buffbot/setup-buffbot.tp2".into());
    for language in LANGUAGES {
        names.insert(format!("buffbot/lang/{language}/setup.tra"));
    }
    names
}

/// Source arguments are extraction roots containing `buffbot/`, not the mod folder.
pub(crate) fn validate_sources(old: &Path, new: &Path) -> Result<(), String> {
    validate_source(old, OLD_TREE)?;
    validate_source(new, NEW_TREE)
}

fn validate_source(root: &Path, expected: &str) -> Result<(), String> {
    let inventory = file_paths(root, "buffbot")?;
    if inventory.keys().cloned().collect::<BTreeSet<_>>() != source_names() {
        return Err("BuffBot source files do not match the supported release inventory.".into());
    }
    let mut digest = Sha256::new();
    for (name, path) in inventory {
        let bytes = files::read(&path, FILE_LIMIT)?;
        digest.update(name.as_bytes());
        digest.update(b"\n");
        digest.update((bytes.len() as u64).to_le_bytes());
        digest.update(bytes);
    }
    if hex::encode(digest.finalize()) != expected {
        return Err("BuffBot source files differ from the reviewed release. Re-download the verified archive.".into());
    }
    Ok(())
}

/// Require both archive provenance and a coherent, supported old installed state.
/// The caller separately proves receipt identity, exact final WeiDU rows [1, 0],
/// game/mod language, no active process and a held installation lock.
pub(crate) fn validate_before(game: &Path, old: &Path) -> Result<(), String> {
    validate_source(old, OLD_TREE)?;
    validate_source(game, OLD_TREE)?;
    validate_installed_files(game, old, true)?;
    validate_loader(game)?;
    validate_backup(game)?;
    if find_optional(game, "lang/en_us/dialogf.tlk")?.is_some() {
        return Err("This BuffBot upgrade does not yet support installations with a separate female dialog.tlk.".into());
    }
    let tlk = read_named(game, TLK_PATH, TLK_LIMIT)?;
    let refs = read_named(game, STRREF_PATH, 4096)?;
    validate_strrefs(&tlk, &refs)
}

/// Verify actual installed output, not just a successful process exit or log row.
/// This version pair must preserve every existing TLK entry and all eight strrefs.
pub(crate) fn verify_after(
    game: &Path,
    new: &Path,
    before_tlk: &[u8],
    before_strrefs: &[u8],
) -> Result<(), String> {
    validate_source(new, NEW_TREE)?;
    validate_source(game, NEW_TREE)?;
    validate_installed_files(game, new, false)?;
    validate_loader(game)?;
    validate_backup(game)?;
    if find_optional(game, "lang/en_us/dialogf.tlk")?.is_some() {
        return Err("An unexpected female dialog.tlk appeared during the upgrade.".into());
    }
    let after = read_named(game, TLK_PATH, TLK_LIMIT)?;
    let refs = read_named(game, STRREF_PATH, 4096)?;
    if parse_strrefs(&refs)? != parse_strrefs(before_strrefs)? {
        return Err(
            "BuffBot's existing ability text references changed during the upgrade.".into(),
        );
    }
    validate_strrefs(&after, &refs)?;
    verify_tlk_unchanged(before_tlk, &after)
}

fn validate_installed_files(game: &Path, source: &Path, before: bool) -> Result<(), String> {
    for name in COPIED {
        let actual = read_named(game, &format!("override/{name}"), FILE_LIMIT)?;
        // The UI legitimately regenerates this shipped file to fit the screen.
        // Before reinstall, accept its bounded MOS structure; after reinstall,
        // require the exact shipped bytes because the game has not run yet.
        if before && *name == "bfbotbg.mos" {
            validate_mos(&actual)?;
        } else if actual != read_named(source, &format!("buffbot/{name}"), FILE_LIMIT)? {
            return Err(format!(
                "Installed BuffBot file differs from the reviewed version: override/{name}"
            ));
        }
    }
    if read_named(game, "override/bfbot_l10n.tra", FILE_LIMIT)?
        != read_named(source, "buffbot/lang/english/setup.tra", FILE_LIMIT)?
    {
        return Err(
            "The installed BuffBot language catalog is not the supported English catalog.".into(),
        );
    }
    Ok(())
}

fn validate_loader(game: &Path) -> Result<(), String> {
    read_named(game, "override/m___eeex.lua", FILE_LIMIT)?;
    let capable = |folder: &str| -> Result<bool, String> {
        let sprite = find_optional(game, &format!("{folder}/eeex_sprite.lua"))?;
        let options = find_optional(game, &format!("{folder}/eeex_options.lua"))?;
        if let (Some(sprite), Some(options)) = (sprite, options) {
            files::regular(&options)?;
            return Ok(files::read(&sprite, FILE_LIMIT)?
                .windows(b"EEex_Sprite_GetCharacterIndex".len())
                .any(|w| w == b"EEex_Sprite_GetCharacterIndex"));
        }
        Ok(false)
    };
    let legacy = capable("override")?;
    let modern = capable("eeex_scripts")?;
    if legacy == modern {
        return Err("BuffBot requires one supported EEex script layout; missing or mixed layouts cannot be upgraded.".into());
    }
    for name in ["lua51.dll", "luaprovider.dll"] {
        if read_named(game, name, FILE_LIMIT)?.is_empty() {
            return Err(format!("The LuaJIT loader file is empty: {name}"));
        }
    }
    let bytes = read_named(game, "infinityloader.ini", FILE_LIMIT)?;
    validate_loader_ini(&bytes, if legacy { "5.1" } else { "5.1-LuaJIT" })
}

fn validate_loader_ini(bytes: &[u8], version: &str) -> Result<(), String> {
    let text = std::str::from_utf8(bytes).map_err(files::err)?;
    let mut section = "";
    let mut sections = 0;
    let mut general = 0;
    let expected = [
        ("LuaPatchMode", "REPLACE_INTERNAL_WITH_EXTERNAL"),
        ("LuaLibrary", "lua51.dll"),
        ("LuaVersionExternal", version),
    ];
    let mut seen = BTreeSet::new();
    for line in text.lines() {
        if line.starts_with('[') {
            sections += 1;
            section = line;
            if line == "[General]" {
                general += 1;
                if sections != 1 {
                    return Err("Loader [General] must be its first section.".into());
                }
            }
        }
        for (key, value) in expected {
            if line.starts_with(&format!("{key}="))
                && (section != "[General]" || line != format!("{key}={value}") || !seen.insert(key))
            {
                return Err("The LuaJIT loader settings are ambiguous or unsupported.".into());
            }
        }
    }
    if general != 1 || seen.len() != expected.len() {
        return Err("BuffBot's exact LuaJIT loader settings are not active.".into());
    }
    Ok(())
}

fn metadata_name(component: u8, kind: &str) -> String {
    match kind.strip_suffix("-text") {
        Some(base) => format!("{BACKUP}/{component}/{base}.{component}.text"),
        None => format!("{BACKUP}/{component}/{kind}.{component}"),
    }
}

fn validate_backup(game: &Path) -> Result<(), String> {
    let actual = file_paths(game, BACKUP)?;
    let expected: BTreeSet<_> = [0, 1]
        .into_iter()
        .flat_map(|component| {
            METADATA
                .iter()
                .map(move |kind| metadata_name(component, kind))
        })
        .collect();
    if actual.keys().cloned().collect::<BTreeSet<_>>() != expected {
        return Err("This BuffBot installation has unsupported uninstall backups. The first upgrade supports fresh CEBG installs only.".into());
    }
    // Reject even empty extra directories: no unknown metadata replay surfaces.
    let backup = find_required(game, BACKUP)?;
    let directories: BTreeSet<_> = walkdir::WalkDir::new(&backup)
        .min_depth(1)
        .into_iter()
        .map(|entry| entry.map_err(files::err))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|e| e.file_type().is_dir())
        .map(|e| {
            e.path()
                .strip_prefix(&backup)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/")
                .to_ascii_lowercase()
        })
        .collect();
    if directories != BTreeSet::from(["0".to_string(), "1".to_string()]) {
        return Err("BuffBot uninstall backup directories have an unsupported layout.".into());
    }
    for component in [0, 1] {
        for kind in METADATA {
            let name = metadata_name(component, kind);
            let bytes = files::read(&actual[&name], FILE_LIMIT)?;
            let valid = match *kind {
                "args" | "readln" => bytes == EMPTY_LIST,
                "tlkpath" => validate_tlkpath(&bytes),
                "uninstall" => validate_uninstall(&bytes, component)?,
                _ => bytes.is_empty(),
            };
            if !valid {
                return Err(format!("Unsupported BuffBot uninstall record: {name}. This upgrade supports fresh English installs with an unchanged EEex-owned loader only."));
            }
        }
    }
    Ok(())
}

fn validate_tlkpath(bytes: &[u8]) -> bool {
    let Ok(text) = std::str::from_utf8(bytes) else {
        return false;
    };
    let normalized = text
        .trim_end_matches(['\r', '\n'])
        .replace('\\', "/")
        .to_ascii_lowercase();
    normalized == "~./lang/en_us/dialog.tlk~" || normalized == "~lang/en_us/dialog.tlk~"
}

fn validate_uninstall(bytes: &[u8], component: u8) -> Result<bool, String> {
    let text = std::str::from_utf8(bytes).map_err(files::err)?;
    let names: Vec<_> = text
        .lines()
        .map(|line| line.replace('\\', "/").to_ascii_lowercase())
        .collect();
    if component == 1 {
        return Ok(names.is_empty());
    }
    let expected = installed_names();
    let actual: BTreeSet<_> = names.iter().cloned().collect();
    Ok(names.len() == actual.len() && actual == expected)
}

fn installed_names() -> BTreeSet<String> {
    let mut names: BTreeSet<_> = COPIED.iter().map(|n| format!("override/{n}")).collect();
    names.insert("override/bfbot_l10n.tra".into());
    names.insert(STRREF_PATH.into());
    names
}

/// Exact install-time writes only. Presets, saves, other mods, runtime spells and
/// loader files are not writable in this pilot (loader preimages are defensive).
pub(crate) fn writable(name: &str) -> bool {
    if files::safe_relative(name).is_err() {
        return false;
    }
    let name = name.to_ascii_lowercase();
    installed_names().contains(&name)
        || source_names().contains(&name)
        || matches!(
            name.as_str(),
            "weidu.log" | "weidu.conf" | "setup-buffbot.debug"
        )
        || name == TLK_PATH
        || [0, 1].into_iter().any(|component| {
            METADATA
                .iter()
                .any(|kind| metadata_name(component, kind) == name)
        })
        || source_directory(&name)
        || matches!(
            name.as_str(),
            BACKUP
                | "weidu_external/backup/buffbot/0"
                | "weidu_external/backup/buffbot/1"
                | "override/bfbot_presets"
        )
}

fn source_directory(name: &str) -> bool {
    matches!(name, "buffbot" | "buffbot/lang")
        || LANGUAGES
            .iter()
            .any(|language| name == format!("buffbot/lang/{language}"))
}

/// Only the owner's known generated resources may differ after playing before Undo.
pub(crate) fn runtime_mutable(name: &str) -> bool {
    if files::safe_relative(name).is_err() {
        return false;
    }
    let name = name.to_ascii_lowercase();
    if matches!(
        name.as_str(),
        "override/bfbotbg.mos"
            | "override/bfbotbg2.mos"
            | "override/bfbotbg3.mos"
            | "override/bfbtch.spl"
            | "override/bfbtcr.spl"
            | "override/bfbtrm.spl"
    ) {
        return true;
    }
    let Some(stem) = name
        .strip_prefix("override/bfbt")
        .and_then(|s| s.strip_suffix(".spl"))
    else {
        return false;
    };
    let bytes = stem.as_bytes();
    bytes.len() == 2 && (b'0'..=b'5').contains(&bytes[0]) && (b'1'..=b'8').contains(&bytes[1])
}

fn parse_strrefs(bytes: &[u8]) -> Result<[u32; 8], String> {
    let text = std::str::from_utf8(bytes).map_err(files::err)?;
    let refs: Vec<u32> = text
        .lines()
        .map(|line| {
            if line.is_empty() || !line.bytes().all(|b| b.is_ascii_digit()) {
                return Err("Invalid BuffBot ability text reference file.".to_string());
            }
            line.parse().map_err(files::err)
        })
        .collect::<Result<_, _>>()?;
    if refs.iter().copied().collect::<BTreeSet<_>>().len() != 8 {
        return Err("BuffBot needs eight distinct ability text references.".into());
    }
    refs.try_into()
        .map_err(|_| "BuffBot needs exactly eight ability text references.".into())
}

fn validate_strrefs(tlk: &[u8], refs: &[u8]) -> Result<(), String> {
    let table = Tlk::parse(tlk)?;
    for (i, index) in parse_strrefs(refs)?.into_iter().enumerate() {
        let entry = table.entry(index as usize)?;
        if entry.1 != format!("BuffBot {}", i + 1).as_bytes() {
            return Err(
                "BuffBot's recorded text references do not resolve to its English ability names."
                    .into(),
            );
        }
    }
    Ok(())
}

struct Tlk<'a> {
    bytes: &'a [u8],
    count: usize,
    data: usize,
}
impl<'a> Tlk<'a> {
    fn parse(bytes: &'a [u8]) -> Result<Self, String> {
        if bytes.get(..8) != Some(b"TLK V1  ") {
            return Err("Unsupported dialog.tlk format.".into());
        }
        let count = word32(bytes, 10)? as usize;
        let data = word32(bytes, 14)? as usize;
        let end = count
            .checked_mul(26)
            .and_then(|n| n.checked_add(18))
            .ok_or("TLK table overflow")?;
        if end > data || data > bytes.len() {
            return Err("Malformed dialog.tlk entry table.".into());
        }
        let result = Self { bytes, count, data };
        for i in 0..count {
            result.entry(i)?;
        }
        Ok(result)
    }
    fn entry(&self, index: usize) -> Result<(&'a [u8], &'a [u8]), String> {
        if index >= self.count {
            return Err("A BuffBot text reference is outside dialog.tlk.".into());
        }
        let base = 18 + index * 26;
        let offset = word32(self.bytes, base + 18)? as usize;
        let length = word32(self.bytes, base + 22)? as usize;
        let start = self
            .data
            .checked_add(offset)
            .ok_or("TLK string offset overflow")?;
        let end = start
            .checked_add(length)
            .ok_or("TLK string length overflow")?;
        let text = self
            .bytes
            .get(start..end)
            .ok_or("Malformed dialog.tlk string bounds")?;
        Ok((&self.bytes[base..base + 18], text))
    }
}

fn verify_tlk_unchanged(before: &[u8], after: &[u8]) -> Result<(), String> {
    let old = Tlk::parse(before)?;
    let new = Tlk::parse(after)?;
    if old.count != new.count || before[8..10] != after[8..10] {
        return Err(
            "The upgrade unexpectedly added/removed TLK entries or changed its language.".into(),
        );
    }
    for i in 0..old.count {
        if old.entry(i)? != new.entry(i)? {
            return Err(format!(
                "Existing dialog.tlk entry {i} changed during the BuffBot upgrade."
            ));
        }
    }
    Ok(())
}

fn word32(bytes: &[u8], offset: usize) -> Result<u32, String> {
    let value = bytes
        .get(offset..offset + 4)
        .ok_or("Truncated binary header")?;
    Ok(u32::from_le_bytes(value.try_into().unwrap()))
}

fn validate_mos(bytes: &[u8]) -> Result<(), String> {
    if bytes.get(..8) != Some(b"MOS V2  ") {
        return Err("BuffBot's generated background is not a supported MOS V2 file.".into());
    }
    let count = word32(bytes, 16)? as usize;
    let offset = word32(bytes, 20)? as usize;
    let end = count
        .checked_mul(28)
        .and_then(|n| n.checked_add(offset))
        .ok_or("MOS size overflow")?;
    if offset != 24
        || count == 0
        || end != bytes.len()
        || word32(bytes, 8)? == 0
        || word32(bytes, 12)? == 0
    {
        return Err("BuffBot's generated background is malformed.".into());
    }
    Ok(())
}

fn file_paths(root: &Path, subdir: &str) -> Result<BTreeMap<String, PathBuf>, String> {
    let folder = find_required(root, subdir)?;
    files::direct(&folder)?;
    if !folder.is_dir() {
        return Err("Expected a direct mod directory.".into());
    }
    let mut files_out = BTreeMap::new();
    let mut seen = BTreeSet::new();
    for entry in walkdir::WalkDir::new(&folder)
        .min_depth(1)
        .follow_links(false)
    {
        let entry = entry.map_err(files::err)?;
        let path = entry.path();
        files::direct(path)?;
        let relative = path
            .strip_prefix(root)
            .map_err(files::err)?
            .to_string_lossy()
            .replace('\\', "/")
            .to_ascii_lowercase();
        files::safe_relative(&relative)?;
        if !seen.insert(relative.clone()) {
            return Err("Case-colliding mod files are not supported.".into());
        }
        if entry.file_type().is_file() {
            files::regular(path)?;
            files_out.insert(relative, path.into());
        } else if !entry.file_type().is_dir() {
            return Err("Special mod files are not supported.".into());
        }
    }
    Ok(files_out)
}

fn find_optional(root: &Path, relative: &str) -> Result<Option<PathBuf>, String> {
    files::safe_relative(relative)?;
    files::direct(root)?;
    // The supported Windows game filesystem resolves names case-insensitively.
    // Avoid enumerating the potentially huge override directory for every file.
    // The enclosing transaction inventory rejects case-colliding paths as well.
    #[cfg(windows)]
    {
        let path = root.join(relative);
        match fs::symlink_metadata(&path) {
            Ok(_) => {
                files::direct(&path)?;
                Ok(Some(path))
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(files::err(error)),
        }
    }
    #[cfg(not(windows))]
    {
        let mut path = root.to_path_buf();
        for part in relative.split('/') {
            if !path.is_dir() {
                return Err("Expected a directory while resolving mod files.".into());
            }
            let matches: Vec<_> = fs::read_dir(&path)
                .map_err(files::err)?
                .map(|entry| entry.map_err(files::err))
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .filter(|entry| {
                    entry
                        .file_name()
                        .to_string_lossy()
                        .eq_ignore_ascii_case(part)
                })
                .collect();
            if matches.len() > 1 {
                return Err("Case-colliding mod paths are not supported.".into());
            }
            let Some(entry) = matches.first() else {
                return Ok(None);
            };
            path = entry.path();
            files::direct(&path)?;
        }
        Ok(Some(path))
    }
}
fn find_required(root: &Path, relative: &str) -> Result<PathBuf, String> {
    find_optional(root, relative)?
        .ok_or_else(|| format!("Required update file is missing: {relative}"))
}
fn read_named(root: &Path, relative: &str, limit: u64) -> Result<Vec<u8>, String> {
    files::read(&find_required(root, relative)?, limit)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tlk(strings: &[&str]) -> Vec<u8> {
        let data = 18 + strings.len() * 26;
        let mut out = vec![0; data];
        out[..8].copy_from_slice(b"TLK V1  ");
        out[10..14].copy_from_slice(&(strings.len() as u32).to_le_bytes());
        out[14..18].copy_from_slice(&(data as u32).to_le_bytes());
        for (i, text) in strings.iter().enumerate() {
            let base = 18 + i * 26;
            let offset = (out.len() - data) as u32;
            out[base] = 1;
            out[base + 18..base + 22].copy_from_slice(&offset.to_le_bytes());
            out[base + 22..base + 26].copy_from_slice(&(text.len() as u32).to_le_bytes());
            out.extend_from_slice(text.as_bytes());
        }
        out
    }
    fn ini() -> Vec<u8> {
        b"[General]\r\nLuaPatchMode=REPLACE_INTERNAL_WITH_EXTERNAL\r\nLuaLibrary=lua51.dll\r\nLuaVersionExternal=5.1-LuaJIT\r\n[Other]\r\n".to_vec()
    }
    #[test]
    fn allowlist_is_exact_and_excludes_player_data() {
        assert_eq!(source_names().len(), 41);
        assert_eq!(installed_names().len(), 33);
        for good in [
            "Override/BfBotScn.lua",
            "buffbot/lang/english/setup.tra",
            "weidu_external/backup/buffbot/0/UNINSTALL.0",
            TLK_PATH,
        ] {
            assert!(writable(good), "{good}");
        }
        for bad in [
            "override/bfbot_presets/mine.json",
            "save/0001/baldur.gam",
            "Baldur.lua",
            "override/bfbot_extra.lua",
            "buffbot/unknown.tp2",
            "lang/fr_FR/dialog.tlk",
            "lua51.dll",
            "weidu_external/backup/buffbot/2/UNINSTALL.2",
            "override/../Baldur.exe",
            "override\\BfBotScn.lua",
        ] {
            assert!(!writable(bad), "{bad}");
        }
    }
    #[test]
    fn runtime_mutations_are_only_known_resource_names() {
        for slot in 0..=5 {
            for preset in 1..=8 {
                assert!(runtime_mutable(&format!("override/BFBT{slot}{preset}.SPL")));
            }
        }
        for name in [
            "override/BFBTCH.SPL",
            "override/BFBTCR.SPL",
            "override/BFBTRM.SPL",
            "override/BFBOTBG.MOS",
            "override/BFBOTBG2.MOS",
            "override/BFBOTBG3.MOS",
        ] {
            assert!(runtime_mutable(name));
        }
        for name in [
            "override/BFBT69.SPL",
            "override/BFBT00.SPL",
            "override/BFBT011.SPL",
            "override/BFBOTBG4.MOS",
            "override/BfBotScn.lua",
            "override/../BFBT01.SPL",
        ] {
            assert!(!runtime_mutable(name));
        }
        assert!(!writable("override/BFBT01.SPL"));
    }
    #[test]
    fn loader_requires_unique_exact_general_keys() {
        validate_loader_ini(&ini(), "5.1-LuaJIT").unwrap();
        assert!(validate_loader_ini(&ini(), "5.1").is_err());
        for suffix in [
            "LuaLibrary=other.dll\n",
            "LuaLibrary=lua51.dll\n",
            "[General]\n",
        ] {
            let mut value = ini();
            value.extend_from_slice(suffix.as_bytes());
            assert!(validate_loader_ini(&value, "5.1-LuaJIT").is_err());
        }
        assert!(
            validate_loader_ini(b"[Other]\n;LuaLibrary=lua51.dll\n[General]\n", "5.1-LuaJIT")
                .is_err()
        );
    }
    #[test]
    fn loader_capability_accepts_one_family_and_rejects_mixed_or_missing_state() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir(temp.path().join("override")).unwrap();
        fs::create_dir(temp.path().join("eeex_scripts")).unwrap();
        fs::write(temp.path().join("override/m___eeex.lua"), b"bootstrap").unwrap();
        fs::write(
            temp.path().join("eeex_scripts/eeex_sprite.lua"),
            b"EEex_Sprite_GetCharacterIndex",
        )
        .unwrap();
        fs::write(
            temp.path().join("eeex_scripts/eeex_options.lua"),
            b"options",
        )
        .unwrap();
        fs::write(temp.path().join("lua51.dll"), b"fixture dll").unwrap();
        fs::write(temp.path().join("luaprovider.dll"), b"fixture dll").unwrap();
        fs::write(temp.path().join("infinityloader.ini"), ini()).unwrap();
        validate_loader(temp.path()).unwrap();
        fs::write(
            temp.path().join("override/eeex_sprite.lua"),
            b"EEex_Sprite_GetCharacterIndex",
        )
        .unwrap();
        fs::write(temp.path().join("override/eeex_options.lua"), b"options").unwrap();
        assert!(validate_loader(temp.path()).is_err());
        fs::remove_file(temp.path().join("override/eeex_sprite.lua")).unwrap();
        fs::write(temp.path().join("lua51.dll"), b"").unwrap();
        assert!(validate_loader(temp.path()).is_err());
    }
    #[test]
    fn exact_uninstall_records_never_replay_external_paths() {
        let valid = installed_names()
            .into_iter()
            .collect::<Vec<_>>()
            .join("\r\n")
            + "\r\n";
        assert!(validate_uninstall(valid.as_bytes(), 0).unwrap());
        assert!(validate_uninstall(b"", 1).unwrap());
        assert!(!validate_uninstall(valid.as_bytes(), 1).unwrap());
        assert!(!validate_uninstall(format!("{valid}../outside\n").as_bytes(), 0).unwrap());
        assert!(
            !validate_uninstall(format!("{valid}override/BfBotScn.lua\n").as_bytes(), 0).unwrap()
        );
        assert!(validate_tlkpath(b"~.\\lang\\en_us\\dialog.tlk~\r\n"));
        for bad in [
            "~C:/external/dialog.tlk~",
            "~../lang/en_us/dialog.tlk~",
            "~./lang/fr_fr/dialog.tlk~",
            "~./lang/en_us/dialog.tlk~\n~x~",
        ] {
            assert!(!validate_tlkpath(bad.as_bytes()));
        }
    }
    #[test]
    fn backup_preflight_requires_complete_proven_metadata_shape() {
        let temp = tempfile::tempdir().unwrap();
        for component in [0, 1] {
            fs::create_dir_all(temp.path().join(format!("{BACKUP}/{component}"))).unwrap();
            for kind in METADATA {
                let bytes = match *kind {
                    "args" | "readln" => EMPTY_LIST.to_vec(),
                    "tlkpath" => b"~.\\lang\\en_us\\dialog.tlk~\r\n".to_vec(),
                    "uninstall" if component == 0 => installed_names()
                        .into_iter()
                        .collect::<Vec<_>>()
                        .join("\r\n")
                        .into_bytes(),
                    _ => vec![],
                };
                fs::write(temp.path().join(metadata_name(component, kind)), bytes).unwrap();
            }
        }
        validate_backup(temp.path()).unwrap();
        let mappings = temp.path().join(metadata_name(0, "mappings"));
        fs::write(&mappings, b"outside mapping").unwrap();
        assert!(validate_backup(temp.path()).is_err());
        fs::write(&mappings, b"").unwrap();
        let helper = temp.path().join(metadata_name(1, "uninstall"));
        fs::write(&helper, b"lua51.dll\r\n").unwrap();
        assert!(validate_backup(temp.path()).is_err());
        fs::write(&helper, b"").unwrap();
        let extra = temp.path().join(format!("{BACKUP}/0/extra"));
        fs::create_dir(&extra).unwrap();
        assert!(validate_backup(temp.path()).is_err());
        fs::remove_dir(&extra).unwrap();
        fs::remove_file(temp.path().join(metadata_name(1, "readln"))).unwrap();
        assert!(validate_backup(temp.path()).is_err());
    }
    #[test]
    fn tlk_validation_preserves_all_existing_content_and_reference_numbers() {
        let labels: Vec<_> = (1..=8).map(|i| format!("BuffBot {i}")).collect();
        let texts: Vec<_> = labels.iter().map(String::as_str).collect();
        let before = tlk(&texts);
        validate_strrefs(&before, b"0\n1\n2\n3\n4\n5\n6\n7\n").unwrap();
        assert!(validate_strrefs(&before, b"0\n1\n2\n3\n4\n5\n6\n6\n").is_err());
        assert!(validate_strrefs(&before, b"1\n2\n3\n4\n5\n6\n7\n8\n").is_err());
        let mut changed = before.clone();
        *changed.last_mut().unwrap() = b'x';
        assert!(verify_tlk_unchanged(&before, &changed).is_err());
        let mut metadata_changed = before.clone();
        metadata_changed[18] ^= 2;
        assert!(verify_tlk_unchanged(&before, &metadata_changed).is_err());
        assert!(verify_tlk_unchanged(&before, &tlk(&["new"])).is_err());
        verify_tlk_unchanged(&before, &before).unwrap();
    }
    #[test]
    fn tlk_repacking_is_safe_but_truncated_or_overlapping_tables_are_not() {
        let before = tlk(&["first", "second"]);
        let mut after = before.clone();
        let data = word32(&after, 14).unwrap() as usize;
        after.splice(data..data, [0, 0]);
        after[14..18].copy_from_slice(&((data + 2) as u32).to_le_bytes());
        verify_tlk_unchanged(&before, &after).unwrap();
        for length in [0, 7, 17, 18, before.len() - 1] {
            assert!(Tlk::parse(&before[..length]).is_err());
        }
        after[14..18].copy_from_slice(&18_u32.to_le_bytes());
        assert!(Tlk::parse(&after).is_err());
    }
    #[test]
    fn generated_background_can_resize_without_becoming_source_drift() {
        let mut mos = Vec::from(&b"MOS V2  "[..]);
        for value in [4096_u32, 2304, 16, 24] {
            mos.extend_from_slice(&value.to_le_bytes());
        }
        mos.resize(24 + 16 * 28, 0);
        validate_mos(&mos).unwrap();
        mos.pop();
        assert!(validate_mos(&mos).is_err());
    }
    #[test]
    fn source_tree_rejects_unknown_or_modified_inputs() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir(temp.path().join("buffbot")).unwrap();
        fs::write(
            temp.path().join("buffbot/setup-buffbot.tp2"),
            b"arbitrary executable source",
        )
        .unwrap();
        assert!(validate_source(temp.path(), OLD_TREE).is_err());
        assert!(find_optional(temp.path(), "../escape").is_err());
        fs::hard_link(
            temp.path().join("buffbot/setup-buffbot.tp2"),
            temp.path().join("linked"),
        )
        .unwrap();
        assert!(file_paths(temp.path(), "buffbot").is_err());
    }

    /// Release acceptance uses the actual pinned ZIPs, not synthetic mod files.
    /// No download, mod execution, or installed-game access occurs here.
    #[test]
    #[ignore = "requires CEBG_BUFFBOT_OLD_ARCHIVE and CEBG_BUFFBOT_NEW_ARCHIVE; no downloads"]
    fn published_archives_match_reviewed_sources() {
        use crate::acquire::{
            extract_archive, ArchiveFormat, ArchiveLimits, ArchiveMode, ArchiveRequirements,
        };

        let temp = tempfile::tempdir().unwrap();
        let mut roots = Vec::new();
        for (variable, archive_hash, tree_hash) in [
            ("CEBG_BUFFBOT_OLD_ARCHIVE", OLD_HASH, OLD_TREE),
            ("CEBG_BUFFBOT_NEW_ARCHIVE", NEW_HASH, NEW_TREE),
        ] {
            let archive = PathBuf::from(std::env::var_os(variable).expect(variable));
            assert_eq!(files::hash(&archive).unwrap(), archive_hash, "{variable}");
            let extracted = extract_archive(
                &archive,
                temp.path(),
                &ArchiveRequirements {
                    artifact_sha256: archive_hash.into(),
                    format: ArchiveFormat::Zip,
                    expected_roots: vec!["buffbot".into()],
                    expected_tp2_paths: vec!["buffbot/setup-buffbot.tp2".into()],
                    limits: ArchiveLimits::default(),
                    mode: ArchiveMode::Public,
                },
            )
            .unwrap();
            validate_source(&extracted.root, tree_hash).expect(variable);
            roots.push(extracted.root);
        }
        validate_sources(&roots[0], &roots[1]).unwrap();

        // Correcting a reviewed digest must not admit changed or extra source.
        for (root, expected) in roots.iter().zip([OLD_TREE, NEW_TREE]) {
            let tp2 = find_required(root, "buffbot/setup-buffbot.tp2").unwrap();
            let original = fs::read(&tp2).unwrap();
            let mut modified = original.clone();
            modified[0] ^= 1;
            fs::write(&tp2, modified).unwrap();
            assert!(validate_source(root, expected).is_err());
            fs::write(&tp2, original).unwrap();
            validate_source(root, expected).unwrap();
            fs::write(root.join("buffbot/unreviewed.lua"), b"unexpected").unwrap();
            assert!(validate_source(root, expected).is_err());
        }
    }
}
