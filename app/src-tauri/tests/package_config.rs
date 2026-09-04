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
