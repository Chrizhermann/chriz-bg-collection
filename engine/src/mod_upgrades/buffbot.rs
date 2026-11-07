//! Recovery footprint for the retired BuffBot 1.8.3 -> 1.8.4 transaction.
//! Kept so existing records, affected-file backups and Undo remain readable.
use crate::patches::files;
use std::collections::BTreeSet;

pub(crate) const OLD_VERSION: &str = "1.8.3-alpha";
pub(crate) const NEW_VERSION: &str = "1.8.4-alpha";
pub(crate) const NEW_HASH: &str =
    "1081abc59511263570603560059ef361f45dd3288933f7faa26166ce0e805ced";
pub(crate) const RUN_ID: &str = "buffbot-bg2";
pub(crate) const UPGRADE_ID: &str = "buffbot-1.8.3-to-1.8.4";
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
fn source_names() -> BTreeSet<String> {
    let mut names: BTreeSet<_> = COPIED.iter().map(|n| format!("buffbot/{n}")).collect();
    names.insert("buffbot/setup-buffbot.tp2".into());
    for language in LANGUAGES {
        names.insert(format!("buffbot/lang/{language}/setup.tra"));
    }
    names
}

fn metadata_name(component: u8, kind: &str) -> String {
    match kind.strip_suffix("-text") {
        Some(base) => format!("{BACKUP}/{component}/{base}.{component}.text"),
        None => format!("{BACKUP}/{component}/{kind}.{component}"),
    }
}

fn installed_names() -> BTreeSet<String> {
    let mut names: BTreeSet<_> = COPIED.iter().map(|n| format!("override/{n}")).collect();
    names.insert("override/bfbot_l10n.tra".into());
    names.insert(STRREF_PATH.into());
    names
}

/// Recorded install-time writes only. Presets, saves, other mods, runtime spells and
/// loader files remain outside the recovery footprint (their preimages are defensive).
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
