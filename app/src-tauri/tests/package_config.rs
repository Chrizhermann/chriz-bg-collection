use std::{fs, path::Path};

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

/// Tauri's installer never removes files an older version shipped when it updates in place, so
/// the preinstall hook empties each folder the app bundles before the new files are written. Its
/// list must name exactly the folders of `bundle.resources`, and the hooks never use `RMDir /r`,
/// which follows junctions out of a folder. `tools/check-nsis-update-cleanup.ps1` runs a real
/// update install in CI.
#[test]
fn the_installer_empties_exactly_the_folders_the_app_bundles() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let config: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(crate_root.join("tauri.conf.json")).expect("read Tauri configuration"),
    )
    .expect("parse Tauri configuration");
    let hooks = config["bundle"]["windows"]["nsis"]["installerHooks"]
        .as_str()
        .expect("NSIS installer hooks are configured");
    let hooks = fs::read_to_string(crate_root.join(hooks)).expect("read NSIS installer hooks");
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

    let mut cleared = code
        .skip_while(|line| *line != "!macro NSIS_HOOK_PREINSTALL")
        .take_while(|line| *line != "!macroend")
        .filter_map(|line| line.strip_prefix("!insertmacro CebgClearBundledFolder "))
        .map(|folder| folder.trim_matches('"').replace('\\', "/"))
        .collect::<Vec<_>>();
    cleared.sort();
    let mut bundled = config["bundle"]["resources"]
        .as_object()
        .expect("bundle resources")
        .values()
        .filter_map(serde_json::Value::as_str)
        .filter_map(|target| target.strip_suffix('/'))
        .map(str::to_owned)
        .collect::<Vec<_>>();
    bundled.sort();
    assert!(!cleared.is_empty());
    assert_eq!(cleared, bundled);
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

#[test]
#[ignore = "requires CEBG_SIGNED_SETUP from a freshly built local release"]
fn built_update_signature_matches_the_bundled_public_key() {
    use base64::Engine;
    let setup = std::env::var("CEBG_SIGNED_SETUP").expect("set CEBG_SIGNED_SETUP");
    let config: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("tauri.conf.json")).unwrap(),
    )
    .unwrap();
    let decode = |text: &str| {
        String::from_utf8(
            base64::engine::general_purpose::STANDARD
                .decode(text.trim())
                .unwrap(),
        )
        .unwrap()
    };
    let public = decode(config["plugins"]["updater"]["pubkey"].as_str().unwrap());
    let signature = decode(&fs::read_to_string(format!("{setup}.sig")).unwrap());
    let key = minisign::PublicKey::from_box(minisign::PublicKeyBox::from_string(&public).unwrap())
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
