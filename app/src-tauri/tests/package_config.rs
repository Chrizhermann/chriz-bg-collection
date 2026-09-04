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

#[test]
fn bundle_resources_include_production_game_profiles() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let config: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(crate_root.join("tauri.conf.json")).expect("read Tauri configuration"),
    )
    .expect("parse Tauri configuration");
    let profile_root = crate_root.join("../../manifest/game-builds");

    assert_eq!(
        config["bundle"]["resources"]["../../manifest/"],
        "manifest/"
    );
    assert!(profile_root
        .join("steam-bgee-sod-2.7.3-en-us.toml")
        .is_file());
    assert!(profile_root.join("steam-bg2ee-2.7.3-en-us.toml").is_file());
}
