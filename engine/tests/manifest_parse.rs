use bg_engine::manifest::{
    AcquisitionPolicy, Artifact, Collection, GameRoot, InvocationMode, ModFile, Phase,
    Postcondition, RunArg, SourceKind,
};

fn collection_fixture() -> String {
    std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/manifest/collection.toml"
    ))
    .unwrap()
}

fn mod_fixture(name: &str) -> String {
    std::fs::read_to_string(format!(
        "{}/tests/fixtures/manifest/mods/{name}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
}

fn artifact_fixture(name: &str) -> String {
    std::fs::read_to_string(format!(
        "{}/tests/fixtures/manifest/artifacts/{name}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
}

fn remove_first_line(text: &str, line_to_remove: &str) -> String {
    let mut removed = false;
    text.lines()
        .filter(|line| {
            if !removed && *line == line_to_remove {
                removed = true;
                false
            } else {
                true
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn installer(id: &str, artifact_id: &str, tp2: &str) -> String {
    format!(
        r#"
id = "{id}"
artifact_id = "{artifact_id}"
name = "{id}"
tp2 = "{tp2}"
language = 0
weidu_artifact_id = "weidu"
invocation_mode = "explicit-tp2"

[[components]]
id = 0
name = "Core"
"#
    )
}

#[test]
fn parses_explicit_runs_and_typed_arguments() {
    let collection: Collection = toml::from_str(&collection_fixture()).unwrap();

    assert_eq!(collection.schema, 2);
    assert_eq!(collection.game_build, "2.7.3.0");
    assert_eq!(collection.runs[0].run_id, "eefixpack-bg1");
    assert_eq!(collection.runs[0].phase, Phase::Bg1Preparation);
    assert_eq!(collection.runs[0].components, vec![0, 2]);
    assert_eq!(
        collection.runs[0].args,
        vec![RunArg::StagedRoot(GameRoot::Bg1)]
    );
    assert_eq!(collection.runs[1].run_id, "eefixpack-bg2");
    assert_eq!(collection.runs[1].phase, Phase::Bg2Preparation);
    assert_eq!(Phase::Bg1Preparation.game_root(), GameRoot::Bg1);
    assert_eq!(Phase::EetInitialization.game_root(), GameRoot::Bg2);
    assert_eq!(Phase::Main.game_root(), GameRoot::Bg2);
}

#[test]
fn parses_typed_run_postconditions() {
    let text = collection_fixture().replacen(
        "args = [{ kind = \"staged-root\", value = \"bg1\" }]",
        concat!(
            "args = [{ kind = \"staged-root\", value = \"bg1\" }]\n",
            "postconditions = [{ kind = \"text-file-markers\", path = \"weidu.conf\", ",
            "required = [\"lang_dir = en_US\"], forbidden = [\"lang_dir = ko_KR\"], ",
            "max_bytes = 4096 }]"
        ),
        1,
    );

    let collection: Collection = toml::from_str(&text).unwrap();
    assert_eq!(
        collection.runs[0].postconditions,
        vec![Postcondition::TextFileMarkers {
            path: "weidu.conf".to_owned(),
            required: vec!["lang_dir = en_US".to_owned()],
            forbidden: vec!["lang_dir = ko_KR".to_owned()],
            max_bytes: 4096,
        }]
    );
}

#[test]
fn parses_installer_referencing_separate_mod_and_weidu_artifacts() {
    let installer: ModFile = toml::from_str(&mod_fixture("eefixpack.toml")).unwrap();

    assert_eq!(installer.id, "eefixpack");
    assert_eq!(installer.artifact_id, "eefixpack");
    assert_eq!(installer.weidu_artifact_id, "weidu");
    assert_eq!(installer.invocation_mode, InvocationMode::ExplicitTp2);
    assert_eq!(installer.components.len(), 2);
    assert_eq!(installer.components[1].stdin.as_deref(), Some("1\n"));
}

#[test]
fn parses_artifact_acquisition_and_provenance_independently() {
    let artifact: Artifact = toml::from_str(&artifact_fixture("eefixpack.toml")).unwrap();

    assert_eq!(artifact.source.kind, SourceKind::GithubRelease);
    assert_eq!(artifact.acquisition, AcquisitionPolicy::FetchOnly);
    assert_eq!(artifact.archive.publish_roots, ["EE_Fixpack"]);
    assert_eq!(artifact.provenance.license, "MIT");
}

#[test]
fn eet_and_eet_end_can_share_one_artifact() {
    let eet: ModFile = toml::from_str(&installer("eet", "eet", "EET/EET.tp2")).unwrap();
    let eet_end: ModFile = toml::from_str(&installer("eet_end", "eet", "EET/EET_end.tp2")).unwrap();

    assert_eq!(eet.artifact_id, eet_end.artifact_id);
    assert_ne!(eet.tp2, eet_end.tp2);
}

#[test]
fn artisan_installers_can_share_one_artifact_and_keep_distinct_tp2_paths() {
    let main: ModFile = toml::from_str(&installer(
        "artisan-kitpack",
        "artisan-kitpack",
        "ArtisansKitpack/ArtisansKitpack.tp2",
    ))
    .unwrap();
    let npcs: ModFile = toml::from_str(&installer(
        "artisan-npcs",
        "artisan-kitpack",
        "ArtisansKitpack/ArtisansKitpack-NPCs.tp2",
    ))
    .unwrap();
    let tweaks: ModFile = toml::from_str(&installer(
        "artisan-tweaks",
        "artisan-kitpack",
        "ArtisansKitpack/ArtisansKitpack-Tweaks.tp2",
    ))
    .unwrap();

    assert_eq!(main.artifact_id, npcs.artifact_id);
    assert_eq!(npcs.artifact_id, tweaks.artifact_id);
    assert_ne!(main.tp2, npcs.tp2);
    assert_ne!(npcs.tp2, tweaks.tp2);
}

#[test]
fn rejects_unknown_collection_field() {
    let text = collection_fixture().replace("game_build", "gamebuild");
    let error = toml::from_str::<Collection>(&text).unwrap_err();

    assert!(error.to_string().contains("gamebuild"), "{error}");
}

#[test]
fn rejects_unknown_installer_field() {
    let text = mod_fixture("eefixpack.toml").replace("tp2 =", "tp22 =");
    let error = toml::from_str::<ModFile>(&text).unwrap_err();

    assert!(error.to_string().contains("tp22"), "{error}");
}

#[test]
fn installer_language_is_required() {
    let text = remove_first_line(&mod_fixture("eefixpack.toml"), "language = 0");
    let error = toml::from_str::<ModFile>(&text).unwrap_err();

    assert!(error.to_string().contains("language"), "{error}");
}

#[test]
fn installer_components_are_required() {
    let text = installer("test", "eefixpack", "test/test.tp2");
    let without_components = text.split("[[components]]").next().unwrap();
    let error = toml::from_str::<ModFile>(without_components).unwrap_err();

    assert!(error.to_string().contains("components"), "{error}");
}

#[test]
fn run_components_are_required() {
    let text = remove_first_line(&collection_fixture(), "components = [0, 2]");
    let error = toml::from_str::<Collection>(&text).unwrap_err();

    assert!(error.to_string().contains("components"), "{error}");
}

#[test]
fn run_arguments_are_required() {
    let text = remove_first_line(
        &collection_fixture(),
        "args = [{ kind = \"staged-root\", value = \"bg1\" }]",
    );
    let error = toml::from_str::<Collection>(&text).unwrap_err();

    assert!(error.to_string().contains("args"), "{error}");
}

#[test]
fn collection_runs_are_required() {
    let error = toml::from_str::<Collection>("schema = 2\ngame_build = \"2.7.3.0\"\n").unwrap_err();

    assert!(error.to_string().contains("runs"), "{error}");
}

#[test]
fn typed_run_argument_rejects_unknown_fields() {
    let text = collection_fixture().replacen(
        "{ kind = \"staged-root\", value = \"bg1\" }",
        "{ kind = \"staged-root\", value = \"bg1\", extra = true }",
        1,
    );
    let error = toml::from_str::<Collection>(&text).unwrap_err();

    assert!(error.to_string().contains("extra"), "{error}");
}

#[test]
fn rejects_unknown_enum_value() {
    let text = mod_fixture("eefixpack.toml").replace("explicit-tp2", "implicit-tp2");
    let error = toml::from_str::<ModFile>(&text).unwrap_err();

    assert!(error.to_string().contains("implicit-tp2"), "{error}");
}
