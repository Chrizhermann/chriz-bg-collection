//! Pure eligibility and postconditions for the reviewed Artisan description-link repair.
//!
//! The caller supplies effective override tables and the installation's own English TLK.
//! This module neither accesses files nor executes the owner-maintained WeiDU adapter.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::weidu::log::parse_active_entries;

/// The complete override-table set accepted by this repair.
pub const TABLES: [&str; 3] = ["BGCLATXT.2DA", "CLASTEXT.2DA", "SODCLTXT.2DA"];

// Metadata from the reviewed AKCB_KIT_DESCRIPTIONS adapter, not mod source code.
// Both Dragon Disciple alternatives and both Monk Revisions symbols are intentional.
const TARGETS: [(&str, u32); 25] = [
    ("BERSERKER", 1003),
    ("WIZARD_SLAYER", 1006),
    ("KENSAI", 1004),
    ("BARBARIAN", 1005),
    ("DWARVEN_DEFENDER", 1007),
    ("FERALAN", 2010),
    ("STALKER", 2011),
    ("BEASTMASTER", 2012),
    ("CAVALIER", 3010),
    ("INQUISITOR", 3003),
    ("UNDEAD_HUNTER", 3011),
    ("BLACKGUARD", 3004),
    ("C0ILM", 3005),
    ("TOTEMIC_DRUID", 5300),
    ("SHAPESHIFTER", 5100),
    ("BEAST_FRIEND", 5200),
    ("C0HIVE", 5002),
    ("ASSASIN", 7004),
    ("BOUNTY_HUNTER", 7007),
    ("SWASHBUCKLER", 7006),
    ("SHADOWDANCER", 7008),
    ("DRAGON_DISCIPLE", 8002),
    ("DRAGON_DISCIPLE", 8003),
    ("DARK_MOON", 10001),
    ("SUN_SOUL", 10001),
];

/// One intended DESCSTR cell replacement, retaining the original row spelling.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DescriptionChange {
    /// Canonical override-table filename.
    pub file: String,
    /// Class row label as found in the input table.
    pub row: String,
    /// Original DESCSTR token.
    pub before: String,
    /// Existing, validated KITLIST HELP token.
    pub after: String,
}

/// A validated, installation-local description repair and its complete cell postconditions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DescriptionPlan {
    /// Intended replacements; empty means the accepted tables already have these links.
    pub changes: Vec<DescriptionChange>,
    /// Every expected header and cell, including all unrelated content.
    pub expected: BTreeMap<String, Vec<Vec<String>>>,
    /// Selected KITLIST symbols and validated local HELP references.
    pub selected: BTreeMap<String, u32>,
}

/// Plans the repair using exact installed version declarations and local string references.
///
/// All three canonical table names are required, with no extras. Only the installed
/// components in the reviewed mapping may select kits. Language zero and a version
/// explicitly present in `supported_versions` are required even for an already-fixed table.
pub fn plan(
    kitlist: &[u8],
    tables: &BTreeMap<String, Vec<u8>>,
    log: &[u8],
    tlk: &[u8],
    supported_versions: &[String],
) -> Result<DescriptionPlan, String> {
    require_tables(tables)?;
    let kits = parse_table(kitlist, "KITLIST.2DA")?;
    if !kits[2][0].eq_ignore_ascii_case("ROWNAME") || !kits[2][3].eq_ignore_ascii_case("HELP") {
        return Err("Unsupported KITLIST column layout".into());
    }

    let log = std::str::from_utf8(log).map_err(|_| "WeiDU.log is not UTF-8".to_owned())?;
    let log = log.strip_prefix('\u{feff}').unwrap_or(log);
    // The shared parser ignores non-entry prose. The pilot deliberately accepts only
    // blank/comment lines and complete active entries, so preserve that stricter gate.
    for (index, line) in log.lines().enumerate() {
        let line = line.trim();
        if !line.is_empty() && !line.starts_with("//") && !line.starts_with('~') {
            return Err(format!("Malformed WeiDU.log entry on line {}", index + 1));
        }
    }
    let mut installed = BTreeMap::new();
    for entry in parse_active_entries(log).map_err(|error| error.to_string())? {
        if entry.tp2_key == "artisanskitpack/artisanskitpack.tp2" {
            let component = entry.component;
            if installed.insert(component, entry).is_some() {
                return Err(format!("Duplicate Artisan component: {component}"));
            }
        }
    }

    let tlk = Tlk::parse(tlk)?;
    let mut selected = BTreeMap::new();
    let mut values = BTreeMap::new();
    for (symbol, component) in TARGETS {
        let Some(entry) = installed.get(&component) else {
            continue;
        };
        let version = entry
            .annotation
            .as_deref()
            .and_then(|annotation| annotation.rsplit(": ").next());
        if entry.language != 0
            || !version.is_some_and(|version| {
                !version.is_empty() && supported_versions.iter().any(|value| value == version)
            })
        {
            return Err(format!(
                "Component {component}: unsupported language/version"
            ));
        }
        let mut matches = kits[3..]
            .iter()
            .filter(|row| row[1].eq_ignore_ascii_case(symbol));
        let row = matches
            .next()
            .ok_or_else(|| format!("Missing/ambiguous KITLIST symbol: {symbol}"))?;
        if matches.next().is_some() {
            return Err(format!("Missing/ambiguous KITLIST symbol: {symbol}"));
        }
        let value = &row[4];
        if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(format!("Invalid HELP reference: {symbol}"));
        }
        let reference: u32 = value
            .parse()
            .map_err(|_| format!("Invalid HELP reference: {symbol}"))?;
        tlk.validate_reference(reference)
            .map_err(|reason| format!("{reason}: {symbol} ({reference})"))?;
        selected.insert(symbol.to_owned(), reference);
        values.insert(symbol, value);
    }
    if selected.is_empty() {
        return Err("No installed supported Artisan components".into());
    }

    let mut changes = Vec::new();
    let mut expected = BTreeMap::new();
    for (filename, data) in tables {
        let mut rows = parse_table(data, filename)?;
        if !rows[2][3].eq_ignore_ascii_case("DESCSTR") {
            return Err(format!("Unsupported description columns: {filename}"));
        }
        let mut names = BTreeSet::new();
        for row in &mut rows[3..] {
            let name = row[0].to_ascii_uppercase();
            if !names.insert(name.clone()) {
                return Err(format!("Duplicate class row: {filename}: {}", row[0]));
            }
            for (symbol, value) in &values {
                if (name == *symbol || name == alias(symbol)) && row[4] != **value {
                    changes.push(DescriptionChange {
                        file: filename.clone(),
                        row: row[0].clone(),
                        before: row[4].clone(),
                        after: (*value).clone(),
                    });
                    row[4] = (*value).clone();
                }
            }
        }
        expected.insert(filename.clone(), rows);
    }
    Ok(DescriptionPlan {
        changes,
        expected,
        selected,
    })
}

/// Verifies all expected table cells, permitting whitespace and comment-only differences.
///
/// Added/removed tables or any changed header, row, or unrelated cell fail verification.
pub fn verify(plan: &DescriptionPlan, tables: &BTreeMap<String, Vec<u8>>) -> Result<(), String> {
    require_tables(&plan.expected)?;
    if !tables.keys().eq(plan.expected.keys()) {
        return Err("Postcondition table set changed".into());
    }
    for (name, data) in tables {
        let rows = parse_table(data, name)?;
        if plan.expected.get(name) != Some(&rows) {
            return Err(format!("Unexpected table edit: {name}"));
        }
    }
    Ok(())
}

fn require_tables<T>(tables: &BTreeMap<String, T>) -> Result<(), String> {
    if tables.len() != TABLES.len() || TABLES.iter().any(|name| !tables.contains_key(*name)) {
        return Err("Exactly BGCLATXT.2DA, CLASTEXT.2DA and SODCLTXT.2DA are required".into());
    }
    Ok(())
}

fn alias(symbol: &str) -> &str {
    match symbol {
        "ASSASIN" => "ASSASSIN",
        "FERALAN" => "ARCHER",
        "BEASTMASTER" => "BEAST_MASTER",
        "BEAST_FRIEND" => "AVENGER",
        _ => symbol,
    }
}

fn parse_table(data: &[u8], name: &str) -> Result<Vec<Vec<String>>, String> {
    if !data.is_ascii() {
        return Err(format!("Non-ASCII 2DA: {name}"));
    }
    let text = std::str::from_utf8(data).map_err(|_| format!("Non-ASCII 2DA: {name}"))?;
    let rows: Vec<Vec<String>> = text
        .lines()
        .map(|line| line.split("//").next().unwrap_or_default())
        .map(|line| line.split_ascii_whitespace().map(str::to_owned).collect())
        .filter(|row: &Vec<String>| !row.is_empty())
        .collect();
    if rows.len() < 4
        || rows[0].len() != 2
        || !rows[0][0].eq_ignore_ascii_case("2DA")
        || !rows[0][1].eq_ignore_ascii_case("V1.0")
        || rows[1].len() != 1
    {
        return Err(format!("Invalid 2DA header: {name}"));
    }
    let width = rows[2]
        .len()
        .checked_add(1)
        .ok_or_else(|| format!("Invalid 2DA width: {name}"))?;
    if width < 5 || rows[3..].iter().any(|row| row.len() != width) {
        return Err(format!("Unsupported/ragged 2DA layout: {name}"));
    }
    Ok(rows)
}

struct Tlk<'a> {
    data: &'a [u8],
    entry_count: u32,
    text_start: usize,
}

impl<'a> Tlk<'a> {
    fn parse(data: &'a [u8]) -> Result<Self, String> {
        if data.len() < 18 || data.get(..8) != Some(b"TLK V1  ") {
            return Err("Invalid TLK header".into());
        }
        let entry_count = read_u32(data, 10)?;
        let text_start = usize::try_from(read_u32(data, 14)?)
            .map_err(|_| "Invalid TLK table bounds".to_owned())?;
        let table_end = usize::try_from(entry_count)
            .ok()
            .and_then(|count| count.checked_mul(26))
            .and_then(|size| size.checked_add(18))
            .ok_or_else(|| "Invalid TLK table bounds".to_owned())?;
        if table_end > text_start || text_start > data.len() {
            return Err("Invalid TLK table bounds".into());
        }
        Ok(Self {
            data,
            entry_count,
            text_start,
        })
    }

    fn validate_reference(&self, reference: u32) -> Result<(), String> {
        if reference >= self.entry_count {
            return Err("Invalid HELP reference".into());
        }
        let entry = usize::try_from(reference)
            .ok()
            .and_then(|reference| reference.checked_mul(26))
            .and_then(|offset| offset.checked_add(18))
            .ok_or_else(|| "Invalid HELP entry bounds".to_owned())?;
        let fields = entry
            .checked_add(18)
            .and_then(|start| start.checked_add(8).map(|end| (start, end)))
            .and_then(|(start, end)| self.data.get(start..end))
            .ok_or_else(|| "Invalid HELP entry bounds".to_owned())?;
        let offset = usize::try_from(read_u32(fields, 0)?)
            .map_err(|_| "Invalid HELP text bounds".to_owned())?;
        let length = usize::try_from(read_u32(fields, 4)?)
            .map_err(|_| "Invalid HELP text bounds".to_owned())?;
        let end = self
            .text_start
            .checked_add(offset)
            .and_then(|start| start.checked_add(length));
        if length == 0 || !end.is_some_and(|end| end <= self.data.len()) {
            return Err("Invalid/empty HELP text".into());
        }
        Ok(())
    }
}

fn read_u32(data: &[u8], offset: usize) -> Result<u32, String> {
    let bytes = offset
        .checked_add(4)
        .and_then(|end| data.get(offset..end))
        .and_then(|bytes| <[u8; 4]>::try_from(bytes).ok())
        .ok_or_else(|| "Truncated TLK field".to_owned())?;
    Ok(u32::from_le_bytes(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    const KITS: &[u8] = b"2DA V1.0\n*\nROWNAME LOWER MIXED HELP EXTRA\n0 ASSASIN 11 12 1 keep\n1 FERALAN 21 22 2 unchanged\n";
    const CLASSES: &[u8] = b"2DA V1.0\n*\nLOWER MIXED OTHER DESCSTR EXTRA\nAssassin 11 12 13 0 keep\nARCHER 21 22 23 0 untouched\nMAGE 31 32 33 999 unrelated\n";

    fn tables(data: &[u8]) -> BTreeMap<String, Vec<u8>> {
        TABLES.map(|name| (name.into(), data.to_vec())).into()
    }

    fn log(component: u32, language: u32, version: &str) -> Vec<u8> {
        format!("~ArtisansKitpack\\ArtisansKitpack.TP2~ #{language} #{component} // Kit: detail: {version}\n").into_bytes()
    }

    fn tlk() -> Vec<u8> {
        let mut data = vec![0_u8; 18 + 3 * 26];
        data[..8].copy_from_slice(b"TLK V1  ");
        data[10..14].copy_from_slice(&3_u32.to_le_bytes());
        data[14..18].copy_from_slice(&96_u32.to_le_bytes());
        for index in 0..3 {
            let entry = 18 + index * 26;
            data[entry + 18..entry + 22].copy_from_slice(&(index as u32).to_le_bytes());
            data[entry + 22..entry + 26].copy_from_slice(&1_u32.to_le_bytes());
        }
        data.extend_from_slice(b"ABC");
        data
    }

    fn supported() -> Vec<String> {
        vec!["chriz-v1.3.0".into()]
    }

    fn normal_plan() -> DescriptionPlan {
        plan(
            KITS,
            &tables(CLASSES),
            &log(7004, 0, "chriz-v1.3.0"),
            &tlk(),
            &supported(),
        )
        .unwrap()
    }

    fn rendered(plan: &DescriptionPlan) -> BTreeMap<String, Vec<u8>> {
        plan.expected
            .iter()
            .map(|(name, rows)| {
                let text = rows
                    .iter()
                    .map(|row| row.join("\t"))
                    .collect::<Vec<_>>()
                    .join("\r\n");
                (
                    name.clone(),
                    format!("// formatting is not a cell\n{text}\n").into_bytes(),
                )
            })
            .collect()
    }

    #[test]
    fn repairs_only_selected_description_cells_and_preserves_unrelated_cells() {
        let result = normal_plan();
        assert_eq!(result.selected, BTreeMap::from([("ASSASIN".into(), 1)]));
        assert_eq!(result.changes.len(), 3);
        for name in TABLES {
            assert!(result.changes.contains(&DescriptionChange {
                file: name.into(),
                row: "Assassin".into(),
                before: "0".into(),
                after: "1".into(),
            }));
            let original = parse_table(CLASSES, name).unwrap();
            let mut expected = original;
            expected[3][4] = "1".into();
            assert_eq!(result.expected[name], expected);
        }
        assert!(verify(&result, &tables(CLASSES)).is_err());
        assert!(verify(&result, &rendered(&result)).is_ok());
    }

    #[test]
    fn repeats_as_noop_but_still_rejects_unsupported_versions() {
        let repaired = rendered(&normal_plan());
        let result = plan(
            KITS,
            &repaired,
            &log(7004, 0, "chriz-v1.3.0"),
            &tlk(),
            &supported(),
        )
        .unwrap();
        assert!(result.changes.is_empty());
        assert!(plan(
            KITS,
            &repaired,
            &log(7004, 0, "chriz-v1.3.0-unreviewed"),
            &tlk(),
            &supported()
        )
        .is_err());
        assert!(plan(
            KITS,
            &repaired,
            &log(7004, 1, "chriz-v1.3.0"),
            &tlk(),
            &supported()
        )
        .is_err());
    }

    #[test]
    fn version_allowlist_is_exact_and_checks_every_corresponding_component() {
        let versions = vec!["chriz-v1.3.0".into(), "reviewed-next".into()];
        assert!(plan(
            KITS,
            &tables(CLASSES),
            &log(7004, 0, "reviewed-next"),
            &tlk(),
            &versions
        )
        .is_ok());
        let mut mixed = log(7004, 0, "chriz-v1.3.0");
        mixed.extend(log(2010, 0, "unknown"));
        assert!(plan(KITS, &tables(CLASSES), &mixed, &tlk(), &versions).is_err());
        assert!(plan(
            KITS,
            &tables(CLASSES),
            &log(7004, 0, "chriz-v1.3.0"),
            &tlk(),
            &[]
        )
        .is_err());
    }

    #[test]
    fn rejects_duplicate_missing_and_malformed_log_evidence() {
        let entry = log(7004, 0, "chriz-v1.3.0");
        let cases = [
            [entry.clone(), entry.clone()].concat(),
            b"~ArtisansKitpack/ArtisansKitpack.TP2~ #0 #7004\n".to_vec(),
            b"~ArtisansKitpack/ArtisansKitpack.TP2~ #0 broken\n".to_vec(),
            [entry.clone(), b"malformed entry\n".to_vec()].concat(),
            log(1, 0, "chriz-v1.3.0"),
            b"// No active components\n".to_vec(),
            vec![0xff],
        ];
        for input in cases {
            assert!(plan(KITS, &tables(CLASSES), &input, &tlk(), &supported()).is_err());
        }
        let bom = [b"\xef\xbb\xbf// header\n".to_vec(), entry].concat();
        assert!(plan(KITS, &tables(CLASSES), &bom, &tlk(), &supported()).is_ok());
    }

    #[test]
    fn requires_all_three_tables_and_no_extra_tables() {
        let result = normal_plan();
        let mut incomplete = rendered(&result);
        incomplete.remove(TABLES[0]);
        assert!(plan(
            KITS,
            &incomplete,
            &log(7004, 0, "chriz-v1.3.0"),
            &tlk(),
            &supported()
        )
        .is_err());
        assert!(verify(&result, &incomplete).is_err());
        let mut extra = rendered(&result);
        extra.insert("OTHER.2DA".into(), CLASSES.to_vec());
        assert!(verify(&result, &extra).is_err());
        assert!(plan(
            KITS,
            &extra,
            &log(7004, 0, "chriz-v1.3.0"),
            &tlk(),
            &supported()
        )
        .is_err());
    }

    #[test]
    fn rejects_bad_table_encoding_shape_columns_and_duplicate_rows() {
        let text = String::from_utf8(CLASSES.to_vec()).unwrap();
        let cases = [
            text.replace("2DA V1.0", "2DA V2.0"),
            text.replace("0 keep", "0"),
            text.replace("DESCSTR", "HELP"),
            text.replace("MAGE", "assassin"),
            text.replace("unrelated", "unrelatéd"),
            "2DA V1.0\n*\nA B C\nrow 1 2 3\n".into(),
        ];
        for input in cases {
            assert!(
                plan(
                    KITS,
                    &tables(input.as_bytes()),
                    &log(7004, 0, "chriz-v1.3.0"),
                    &tlk(),
                    &supported()
                )
                .is_err(),
                "{input}"
            );
        }
        let kits = String::from_utf8(KITS.to_vec()).unwrap();
        for input in [
            kits.replace("HELP", "DESCSTR"),
            kits.replace("FERALAN", "ASSASIN"),
            kits.replace("ASSASIN", "OTHER"),
        ] {
            assert!(plan(
                input.as_bytes(),
                &tables(CLASSES),
                &log(7004, 0, "chriz-v1.3.0"),
                &tlk(),
                &supported()
            )
            .is_err());
        }
    }

    #[test]
    fn rejects_nonnumeric_out_of_range_and_empty_help_references() {
        let kits = String::from_utf8(KITS.to_vec()).unwrap();
        for value in ["-1", "+1", "0x1", "4294967296", "3"] {
            let input = kits.replace("12 1 keep", &format!("12 {value} keep"));
            assert!(plan(
                input.as_bytes(),
                &tables(CLASSES),
                &log(7004, 0, "chriz-v1.3.0"),
                &tlk(),
                &supported()
            )
            .is_err());
        }
        let mut empty = tlk();
        empty[66..70].copy_from_slice(&0_u32.to_le_bytes());
        assert!(plan(
            KITS,
            &tables(CLASSES),
            &log(7004, 0, "chriz-v1.3.0"),
            &empty,
            &supported()
        )
        .is_err());
    }

    #[test]
    fn rejects_truncated_and_overflowing_tlk_ranges() {
        let valid = tlk();
        let mut cases = vec![valid[..17].to_vec(), valid[..97].to_vec()];
        for (offset, value) in [
            (10, u32::MAX),
            (14, 95),
            (14, u32::MAX),
            (62, u32::MAX),
            (66, u32::MAX),
        ] {
            let mut bad = valid.clone();
            bad[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
            cases.push(bad);
        }
        let mut bad_signature = valid;
        bad_signature[4] = b'2';
        cases.push(bad_signature);
        for input in cases {
            assert!(plan(
                KITS,
                &tables(CLASSES),
                &log(7004, 0, "chriz-v1.3.0"),
                &input,
                &supported()
            )
            .is_err());
        }
    }

    #[test]
    fn verify_rejects_unrelated_cell_edits_and_row_reordering() {
        let result = normal_plan();
        let mut altered = result.clone();
        altered.expected.get_mut(TABLES[0]).unwrap()[5][2] = "tampered".into();
        assert!(verify(&result, &rendered(&altered)).is_err());
        let rows = altered.expected.get_mut(TABLES[0]).unwrap();
        rows.swap(3, 4);
        assert!(verify(&result, &rendered(&altered)).is_err());
    }

    #[test]
    fn all_reviewed_mappings_and_aliases_select_the_local_help_reference() {
        for (symbol, component) in TARGETS {
            let other_monk = if symbol == "DARK_MOON" {
                "SUN_SOUL"
            } else {
                "DARK_MOON"
            };
            let mut kits = format!("2DA V1.0\n*\nROWNAME LOWER MIXED HELP\n0 {symbol} 10 11 1\n");
            if component == 10001 {
                kits.push_str(&format!("1 {other_monk} 10 11 2\n"));
            }
            let classes = format!(
                "2DA V1.0\n*\nLOWER MIXED OTHER DESCSTR\n{} 10 11 12 0\n",
                alias(symbol)
            );
            let result = plan(
                kits.as_bytes(),
                &tables(classes.as_bytes()),
                &log(component, 0, "chriz-v1.3.0"),
                &tlk(),
                &supported(),
            )
            .unwrap();
            assert_eq!(result.selected[symbol], 1, "{symbol}/{component}");
            assert_eq!(result.changes.len(), 3, "{symbol}/{component}");
        }
    }
}
