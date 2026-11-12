use std::{fs, path::Path};

use serde_json::Value;

/// Update feeds on each app's own distribution repository: a fixed release tag that carries only
/// `latest.json`, re-pointed to each new version once its assets are checked (see
/// `docs/cebg-release.md` and `docs/iwd-release.md`).
const BG_FEED: &str =
    "https://github.com/Chrizhermann/chriz-easy-bg/releases/download/alpha/latest.json";
const IWD_FEED: &str =
    "https://github.com/Chrizhermann/chriz-easy-iwd/releases/download/updates/latest.json";

/// `tauri.conf.json` with `overlay` merged over it, as the Tauri CLI merges a `--config` file.
fn configuration(overlay: Option<&str>) -> Value {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let read = |name: &str| -> Value {
        serde_json::from_str(
            &fs::read_to_string(crate_root.join(name)).expect("read Tauri configuration"),
        )
        .expect("parse Tauri configuration")
    };
    let mut config = read("tauri.conf.json");
    if let Some(overlay) = overlay {
        merge(&mut config, &read(overlay));
    }
    config
}

/// The configuration of the app this test was compiled as: the CLI hands its overlay to the
/// build as `TAURI_CONFIG`, as `build.rs` describes.
fn compiled_configuration() -> Value {
    let mut config = configuration(None);
    if let Some(patch) = option_env!("TAURI_CONFIG") {
        merge(
            &mut config,
            &serde_json::from_str(patch).expect("TAURI_CONFIG is a JSON configuration patch"),
        );
    }
    config
}

/// RFC 7396 JSON merge patch, the rule the Tauri CLI uses for configuration files.
fn merge(target: &mut Value, patch: &Value) {
    let Value::Object(patch) = patch else {
        *target = patch.clone();
        return;
    };
    if !target.is_object() {
        *target = Value::Object(serde_json::Map::new());
    }
    let Value::Object(target) = target else {
        unreachable!("replaced by an object above")
    };
    for (key, value) in patch {
        if value.is_null() {
            target.remove(key);
        } else {
            merge(target.entry(key.clone()).or_insert(Value::Null), value);
        }
    }
}

/// The minisign key an app's updater checks setups against.
fn updater_public_key(config: &Value) -> minisign::PublicKey {
    use base64::Engine;
    let encoded = config["plugins"]["updater"]["pubkey"]
        .as_str()
        .expect("the updater names its public key");
    let text = String::from_utf8(
        base64::engine::general_purpose::STANDARD
            .decode(encoded.trim())
            .expect("the updater public key is base64"),
    )
    .expect("the updater public key is text");
    minisign::PublicKey::from_box(
        minisign::PublicKeyBox::from_string(&text).expect("the updater public key is a key box"),
    )
    .expect("the updater public key is a minisign key")
}

#[test]
fn product_name_is_safe_for_the_nsis_template() {
    let config_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tauri.conf.json");
    let config: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(config_path).expect("read Tauri configuration"))
            .expect("parse Tauri configuration");

    assert_eq!(config["productName"], "Chriz Easy BG");
    assert_eq!(config["app"]["windows"][0]["title"], "Chriz Easy BG");
    assert_eq!(config["app"]["windows"][0]["width"], 1160);
    assert_eq!(config["app"]["windows"][0]["height"], 760);
}

/// The uninstaller hook asks the app to remove its data with the request the app accepts.
#[test]
fn the_uninstaller_hook_asks_the_app_to_remove_its_application_data() {
    use chriz_bg_app_lib::application_data::REMOVE_APPLICATION_DATA;

    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let config: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(crate_root.join("tauri.conf.json")).expect("read Tauri configuration"),
    )
    .expect("parse Tauri configuration");
    let hooks = config["bundle"]["windows"]["nsis"]["installerHooks"]
        .as_str()
        .expect("NSIS installer hooks are configured");
    let hooks = fs::read_to_string(crate_root.join(hooks)).expect("read NSIS installer hooks");

    assert!(hooks.contains("!macro NSIS_HOOK_PREUNINSTALL"));
    assert!(hooks.contains("!macro NSIS_HOOK_POSTUNINSTALL"));
    assert!(hooks.contains(&format!(
        r#"ExecWait '"$INSTDIR\${{MAINBINARYNAME}}.exe" {REMOVE_APPLICATION_DATA} "${{BUNDLEID}}"' $0"#
    )));
}

#[test]
fn bundle_resources_include_production_game_profiles() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let config: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(crate_root.join("tauri.conf.json")).expect("read Tauri configuration"),
    )
    .expect("parse Tauri configuration");
    let profile_root = crate_root.join("../../manifest/game-builds");

    let resources = config["bundle"]["resources"].as_object().unwrap();
    assert!(!resources.contains_key("../../manifest/"));
    for relative in [
        "collection.toml",
        "release.json",
        "artifacts/",
        "mods/",
        "presets/",
        "game-builds/",
        "releases/",
    ] {
        assert_eq!(
            resources[&format!("../../manifest/{relative}")],
            format!("manifest/{relative}")
        );
    }
    for authoring in [
        "reference/",
        "evidence/",
        "curation-map.toml",
        "install-order.tsv",
        "mod-sources.tsv",
        "own-mods.toml",
    ] {
        assert!(!resources.contains_key(&format!("../../manifest/{authoring}")));
    }
    assert_eq!(
        config["bundle"]["resources"]["../../recipes/curated-full-current/"],
        "recipes/curated-full-current/"
    );
    assert!(config["bundle"]["resources"]
        .get("../../recipes/creator-full-current/")
        .is_none());
    assert!(profile_root
        .join("steam-bgee-sod-2.7.3-en-us.toml")
        .is_file());
    assert!(profile_root.join("steam-bg2ee-2.7.3-en-us.toml").is_file());
}

#[test]
fn bundle_resources_include_application_and_decoder_notices() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let config: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(crate_root.join("tauri.conf.json")).unwrap())
            .unwrap();
    for (source, target) in [
        ("../../LICENSE", "LICENSE"),
        ("../../THIRD_PARTY_NOTICES.md", "THIRD_PARTY_NOTICES.md"),
    ] {
        assert_eq!(config["bundle"]["resources"][source], target);
        assert!(crate_root.join(source).is_file());
    }
}

/// Chriz Easy IWD has its own icon in the "Glacier" look (Christopher, 2026-10-28), with the
/// same sizes as the Chriz Easy BG icon, and its setup file shows it too; `icon.svg` beside it
/// is the source.
#[test]
fn the_iwd_app_has_its_own_icon() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let config = |name: &str| -> serde_json::Value {
        serde_json::from_str(&fs::read_to_string(crate_root.join(name)).unwrap()).unwrap()
    };
    let sizes = |path: &str| -> Vec<u32> {
        let ico = fs::read(crate_root.join(path)).unwrap();
        assert_eq!(ico[..4], [0, 0, 1, 0], "{path} is an icon file");
        let count = usize::from(u16::from_le_bytes([ico[4], ico[5]]));
        let mut sizes = (0..count)
            .map(|index| match ico[6 + 16 * index] {
                0 => 256,
                size => u32::from(size),
            })
            .collect::<Vec<_>>();
        sizes.sort_unstable();
        sizes
    };

    assert_eq!(
        config("tauri.conf.json")["bundle"]["icon"],
        serde_json::json!(["icons/icon.ico"])
    );
    let iwd = config("tauri.iwd.conf.json");
    assert_eq!(
        iwd["bundle"]["icon"],
        serde_json::json!(["icons/iwd/icon.ico"])
    );
    assert_eq!(
        iwd["bundle"]["windows"]["nsis"]["installerIcon"],
        "icons/iwd/icon.ico"
    );
    assert!(crate_root.join("icons/iwd/icon.svg").is_file());
    assert_ne!(
        fs::read(crate_root.join("icons/iwd/icon.ico")).unwrap(),
        fs::read(crate_root.join("icons/icon.ico")).unwrap()
    );
    assert_eq!(sizes("icons/iwd/icon.ico"), sizes("icons/icon.ico"));

    let check_bmp = |path: &str, expected_width: i32, expected_height: i32| {
        let bmp = fs::read(crate_root.join(path)).unwrap();
        assert_eq!(&bmp[0..2], b"BM", "{path} starts with BM header");
        let dib_size = u32::from_le_bytes(bmp[14..18].try_into().unwrap());
        assert_eq!(dib_size, 40, "{path} uses 40-byte BITMAPINFOHEADER");
        let width = i32::from_le_bytes(bmp[18..22].try_into().unwrap());
        let height = i32::from_le_bytes(bmp[22..26].try_into().unwrap());
        let bpp = u16::from_le_bytes(bmp[28..30].try_into().unwrap());
        let compression = u32::from_le_bytes(bmp[30..34].try_into().unwrap());
        assert_eq!((width, height), (expected_width, expected_height), "{path} dimensions");
        assert_eq!(bpp, 24, "{path} is 24-bit RGB");
        assert_eq!(compression, 0, "{path} is uncompressed BI_RGB");
        bmp
    };
    let bg_nsis = &config("tauri.conf.json")["bundle"]["windows"]["nsis"];
    let iwd_nsis = &iwd["bundle"]["windows"]["nsis"];
    assert_eq!(bg_nsis["sidebarImage"], "windows/bg-sidebar.bmp");
    assert_eq!(bg_nsis["headerImage"], "windows/bg-header.bmp");
    assert_eq!(iwd_nsis["sidebarImage"], "windows/iwd-sidebar.bmp");
    assert_eq!(iwd_nsis["headerImage"], "windows/iwd-header.bmp");
    assert_ne!(
        check_bmp("windows/bg-sidebar.bmp", 164, 314),
        check_bmp("windows/iwd-sidebar.bmp", 164, 314)
    );
    assert_ne!(
        check_bmp("windows/bg-header.bmp", 150, 57),
        check_bmp("windows/iwd-header.bmp", 150, 57)
    );
}

/// Each app updates itself only from its own feed, checks setups against its own key, and its
/// release build writes the updater signature. The keys differ, so a setup signed for one app is
/// never installed as an update of the other.
#[test]
fn each_app_updates_itself_from_its_own_signed_feed() {
    use chriz_bg_app_lib::flavor::{ALL, BG, IWD};

    let bg = configuration(None);
    let iwd = configuration(Some("tauri.iwd.conf.json"));
    // The app loads the updater exactly when its configuration has a feed (`lib.rs`).
    for flavor in ALL {
        let config = match flavor.key {
            "bg" => &bg,
            "iwd" => &iwd,
            other => panic!("no configuration for the {other} app"),
        };
        assert_eq!(
            config["plugins"]["updater"].is_object(),
            flavor.app_updates,
            "{}",
            flavor.key
        );
        assert_eq!(
            config["bundle"]["createUpdaterArtifacts"], flavor.app_updates,
            "{}",
            flavor.key
        );
    }
    for (flavor, config, feed) in [(&BG, &bg, BG_FEED), (&IWD, &iwd, IWD_FEED)] {
        assert_eq!(config["identifier"], flavor.identifier);
        let updater = &config["plugins"]["updater"];
        assert_eq!(
            updater["endpoints"],
            serde_json::json!([feed]),
            "{}",
            flavor.key
        );
        assert_eq!(
            updater["windows"]["installMode"], "passive",
            "{}",
            flavor.key
        );
    }
    assert_ne!(updater_public_key(&bg), updater_public_key(&iwd));
}

/// Tauri's installer never removes files an older version shipped when it updates in place, so
/// the preinstall hook empties each folder an app bundles before the new files are written. Its
/// list for each app must name exactly the folders of that app's `bundle.resources`, and the
/// hooks never use `RMDir /r`, which follows junctions out of a folder.
/// `tools/check-nsis-update-cleanup.ps1` runs a real update install in CI.
#[test]
fn the_installer_empties_exactly_the_folders_each_app_bundles() {
    use chriz_bg_app_lib::flavor::IWD;

    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let bg = configuration(None);
    let iwd = configuration(Some("tauri.iwd.conf.json"));
    for config in [&bg, &iwd] {
        assert_eq!(
            config["bundle"]["windows"]["nsis"]["installerHooks"],
            "windows/hooks.nsh"
        );
    }
    let hooks = fs::read_to_string(crate_root.join("windows/hooks.nsh"))
        .expect("read NSIS installer hooks");
    let code = hooks
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with(';'));
    for line in code.clone() {
        let mut words = line.split_whitespace();
        let command = words.next().unwrap_or_default();
        assert!(
            !(command.eq_ignore_ascii_case("RMDir")
                && words
                    .next()
                    .is_some_and(|flag| flag.eq_ignore_ascii_case("/r"))),
            "the hooks must not follow junctions: {line}"
        );
    }

    let mut lists = [Vec::new(), Vec::new()];
    let mut current = None;
    let mut inside = false;
    for line in code {
        match line {
            "!macro NSIS_HOOK_PREINSTALL" => inside = true,
            "!macroend" => inside = false,
            _ if !inside => {}
            "!else" => current = Some(0),
            "!endif" => current = None,
            _ if line.starts_with("!if ") => {
                assert_eq!(
                    line,
                    format!("!if \"${{BUNDLEID}}\" == \"{}\"", IWD.identifier)
                );
                current = Some(1);
            }
            _ => {
                if let Some(folder) = line.strip_prefix("!insertmacro CebgClearBundledFolder ") {
                    let index = current.expect("each folder belongs to one app's list");
                    lists[index].push(folder.trim_matches('"').replace('\\', "/"));
                }
            }
        }
    }
    let bundled = |config: &Value| {
        let mut folders = config["bundle"]["resources"]
            .as_object()
            .expect("bundle resources")
            .values()
            .filter_map(Value::as_str)
            .filter_map(|target| target.strip_suffix('/'))
            .map(str::to_owned)
            .collect::<Vec<_>>();
        folders.sort();
        folders
    };
    for list in &mut lists {
        list.sort();
    }
    let [bg_list, iwd_list] = lists;
    assert_eq!(bg_list, bundled(&bg));
    assert_eq!(iwd_list, bundled(&iwd));
    assert!(!bg_list.is_empty() && !iwd_list.is_empty());
}

#[test]
#[ignore = "requires CEBG_SIGNED_SETUP from a freshly built local release"]
fn built_update_signature_matches_the_bundled_public_key() {
    use base64::Engine;
    let setup = std::env::var("CEBG_SIGNED_SETUP").expect("set CEBG_SIGNED_SETUP");
    // Chriz Easy IWD is checked against its own key when built with its configuration.
    let key = updater_public_key(&compiled_configuration());
    let signature = String::from_utf8(
        base64::engine::general_purpose::STANDARD
            .decode(fs::read_to_string(format!("{setup}.sig")).unwrap().trim())
            .unwrap(),
    )
    .unwrap();
    let signature = minisign::SignatureBox::from_string(&signature).unwrap();
    minisign::verify(
        &key,
        &signature,
        fs::File::open(&setup).unwrap(),
        true,
        false,
        false,
    )
    .expect("signed setup matches embedded updater key");
}
