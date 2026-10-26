fn main() {
    tauri_build::build();
    select_flavor();

    // Tauri's mock runtime also needs Common Controls v6 at process startup.
    // The normal app resource does not supply a manifest to integration tests.
    // https://github.com/tauri-apps/tauri/issues/13419
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc")
    {
        let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/windows-test.manifest");
        println!("cargo:rerun-if-changed={}", manifest.display());
        // Cargo scopes these arguments to test executables, leaving app linking unchanged.
        println!("cargo:rustc-link-arg-tests=/MANIFEST:EMBED");
        println!(
            "cargo:rustc-link-arg-tests=/MANIFESTINPUT:{}",
            manifest.display()
        );
    }
}

/// Builds the Icewind Dale app when the Tauri CLI passes `tauri.iwd.conf.json` (`--config`),
/// which it hands to the build as the JSON patch in `TAURI_CONFIG`. Without a patch, or with
/// the Baldur's Gate identifier, this is the Baldur's Gate app. An unknown identifier stops the
/// build rather than shipping one app under another's name; see `src/flavor.rs`.
fn select_flavor() {
    println!("cargo:rustc-check-cfg=cfg(chriz_flavor_iwd)");
    println!("cargo:rerun-if-env-changed=TAURI_CONFIG");
    let Ok(patch) = std::env::var("TAURI_CONFIG") else {
        return;
    };
    let patch: serde_json::Value =
        serde_json::from_str(&patch).expect("TAURI_CONFIG must be a JSON configuration patch");
    match patch.get("identifier").and_then(serde_json::Value::as_str) {
        None | Some("dev.chrizhermann.bgcollection") => {}
        Some("dev.chrizhermann.iwdcollection") => {
            println!("cargo:rustc-cfg=chriz_flavor_iwd");
        }
        Some(other) => panic!("no app flavor uses the bundle identifier {other:?}"),
    }
}
