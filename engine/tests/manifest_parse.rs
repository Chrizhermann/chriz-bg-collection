use bg_engine::manifest::{Collection, ComponentRef, ModFile, Phase, SourceKind};

fn fixture(name: &str) -> String {
    std::fs::read_to_string(format!(
        "{}/tests/fixtures/manifest/mods/{name}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
}

fn collection_fixture() -> String {
    std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/manifest/collection.toml"
    ))
    .unwrap()
}

#[test]
fn parses_collection() {
    let collection: Collection = toml::from_str(&collection_fixture()).unwrap();
    assert_eq!(collection.schema, 1);
    assert_eq!(collection.game_build, "2.7.3.0");
    assert_eq!(collection.order.len(), 3);
    assert_eq!(collection.order[1].components, Some(vec![0]));
    assert!(collection.order[0].components.is_none());
    assert_eq!(
        collection.toggles[0].removes_components[0],
        ComponentRef {
            mod_id: "testmod".to_owned(),
            component: 10,
        }
    );
    assert_eq!(collection.choice_groups[0].default, "plain");
    assert_eq!(
        collection.choice_groups[0].options[1].adds_components.len(),
        1
    );
}

#[test]
fn rejects_unknown_collection_field() {
    let text = collection_fixture().replace("game_build", "gamebuild");
    let err = toml::from_str::<Collection>(&text).unwrap_err();
    assert!(err.to_string().contains("gamebuild"), "{err}");
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
}

#[test]
fn rejects_unknown_field() {
    let text = fixture("testmod.toml").replace("weidu = \"249.00\"", "wiedu = \"249.00\"");
    let err = toml::from_str::<ModFile>(&text).unwrap_err();
    assert!(err.to_string().contains("wiedu"), "{err}");
}

#[test]
fn rejects_unknown_enum_value() {
    let text = fixture("testmod.toml").replace("phase = \"main\"", "phase = \"mian\"");
    let err = toml::from_str::<ModFile>(&text).unwrap_err();
    assert!(err.to_string().contains("mian"), "{err}");
}
