use std::{fs, path::Path};

#[test]
fn product_name_is_safe_for_the_nsis_template() {
    let config_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tauri.conf.json");
    let config: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(config_path).expect("read Tauri configuration"))
            .expect("parse Tauri configuration");

    assert_eq!(config["productName"], "Chriz BG Collection");
}
