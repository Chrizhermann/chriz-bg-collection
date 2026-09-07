fn main() {
    tauri_build::build();

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
