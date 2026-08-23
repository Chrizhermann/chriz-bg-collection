use bg_engine::manifest::{ModFile, Phase, SourceKind};

fn fixture(name: &str) -> String {
    std::fs::read_to_string(format!(
        "{}/tests/fixtures/manifest/mods/{name}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
}

#[test]
fn parses_mod_file() {
    let m: ModFile = toml::from_str(&fixture("testmod.toml")).unwrap();
    assert_eq!(m.id, "testmod");
    assert_eq!(m.weidu, "249.00");
    assert_eq!(m.phase, Phase::Main);
    assert_eq!(m.source.kind, SourceKind::GithubRelease);
    assert_eq!(m.components.len(), 2);
    assert_eq!(m.components[1].stdin.as_deref(), Some("1\n"));
    assert_eq!(m.platforms, vec!["windows", "macos", "linux"]);
}

#[test]
fn parses_commit_zip_source_and_defaults() {
    let m: ModFile = toml::from_str(&fixture("eet.toml")).unwrap();
    assert_eq!(m.source.kind, SourceKind::GithubCommitZip);
    assert_eq!(m.language, 0); // defaulted
    assert!(m.source.manual_page.is_none());
}
