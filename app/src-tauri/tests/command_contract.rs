use std::fs;
use std::path::{Path, PathBuf};

use bg_engine::games::{
    Eligibility, FindingKind, GameCandidate, GameFinding, GameRole, Storefront,
};
use bg_engine::recipe_view::NormalizedSelection;
use chriz_bg_app_lib::bridge::NativeBridge;
use chriz_bg_app_lib::error::CommandError;
use serde_json::json;
use tempfile::TempDir;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("canonical workspace root")
}

fn copy_tree(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).expect("create fixture directory");
    let mut entries = fs::read_dir(source)
        .expect("read fixture directory")
        .collect::<Result<Vec<_>, _>>()
        .expect("read fixture entries");
    entries.sort_by_key(|entry| entry.file_name().to_string_lossy().to_ascii_lowercase());
    for entry in entries {
        let target = destination.join(entry.file_name());
        if entry.file_type().expect("read fixture entry type").is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).expect("copy fixture file");
        }
    }
}

fn recipe_with_profiles() -> (TempDir, PathBuf) {
    let temp = tempfile::tempdir().expect("create bridge fixture root");
    let recipe = temp.path().join("recipe");
    copy_tree(
        &workspace_root().join("engine/tests/fixtures/manifest"),
        &recipe,
    );
    copy_tree(
        &workspace_root().join("engine/tests/fixtures/games/profiles"),
        &recipe.join("game-builds"),
    );
    (temp, recipe)
}

fn candidate(
    role: GameRole,
    storefront: Storefront,
    root: &str,
    eligibility: Eligibility,
    kind: FindingKind,
    message: &str,
) -> GameCandidate {
    GameCandidate {
        role,
        storefront,
        root: PathBuf::from(root),
        build: Some("2.7.3.0".to_owned()),
        eligibility,
        findings: vec![GameFinding {
            kind,
            message: message.to_owned(),
            paths: Vec::new(),
        }],
        fingerprint: Some("fixture-fingerprint".to_owned()),
    }
}

#[test]
fn command_errors_have_one_stable_serialized_shape() {
    let error = CommandError::new(
        "invalid_selection",
        "That combination is unavailable.",
        "Review the highlighted options and try again.",
        "engine detail",
    );

    assert_eq!(
        serde_json::to_value(error).expect("serialize command error"),
        json!({
            "code": "invalid_selection",
            "message": "That combination is unavailable.",
            "recovery_action": "Review the highlighted options and try again.",
            "technical_detail": "engine detail"
        })
    );
}

#[test]
fn bootstrap_validates_the_recipe_without_claiming_an_unsigned_version() {
    let bridge = NativeBridge::new(workspace_root().join("manifest"), "chris-recommended");

    let status = bridge.bootstrap().expect("bootstrap production recipe");

    assert_eq!(status.mode, "native");
    assert_eq!(status.engine_version, "0.1.0");
    assert_eq!(status.recipe_version, None);
}

#[test]
fn production_bridge_loads_only_the_recipe_bundled_below_the_resource_directory() {
    let resources = tempfile::tempdir().expect("create packaged resource fixture");
    copy_tree(
        &workspace_root().join("manifest"),
        &resources.path().join("manifest"),
    );
    let bridge = NativeBridge::from_resource_dir(resources.path());

    let status = bridge.bootstrap().expect("bootstrap packaged recipe");

    assert_eq!(status.mode, "native");
    assert_eq!(status.recipe_version, None);
}

#[test]
fn discovery_groups_roles_and_selects_each_candidate_independently() {
    let bridge = NativeBridge::with_discoverer(
        workspace_root().join("engine/tests/fixtures/manifest"),
        "recommended",
        |_| {
            Ok(vec![
                candidate(
                    GameRole::Bg2ee,
                    Storefront::Gog,
                    r"C:\Fixture\BG2EE",
                    Eligibility::Experimental,
                    FindingKind::UnverifiedStorefront,
                    "GOG still needs release acceptance.",
                ),
                candidate(
                    GameRole::BgeeSod,
                    Storefront::Steam,
                    r"C:\Fixture\BGEE",
                    Eligibility::Eligible,
                    FindingKind::Fresh,
                    "Clean supported installation.",
                ),
            ])
        },
    );

    let discovery = bridge.discover_games().expect("project discovery");

    assert_eq!(discovery.bg1_candidates.len(), 1);
    assert_eq!(discovery.bg2_candidates.len(), 1);
    assert_eq!(discovery.selected_bg1_id, discovery.bg1_candidates[0].id);
    assert_eq!(discovery.selected_bg2_id, discovery.bg2_candidates[0].id);
    assert_ne!(discovery.selected_bg1_id, discovery.selected_bg2_id);
    assert!(discovery.bg1_candidates[0].eligible);
    assert_eq!(discovery.bg1_candidates[0].storefront, "steam");
    assert_eq!(
        discovery.bg1_candidates[0].build.as_deref(),
        Some("2.7.3.0")
    );
    assert!(!discovery.bg2_candidates[0].eligible);
}

#[test]
fn explicit_game_paths_are_inspected_by_the_engine_before_projection() {
    let (_temp, recipe) = recipe_with_profiles();
    let bridge = NativeBridge::new(recipe, "recommended");
    let game = workspace_root().join("engine/tests/fixtures/games/content/steam-bgee-sod");

    let inspected = bridge
        .inspect_game_path(GameRole::BgeeSod, &game)
        .expect("inspect fixture path");

    assert_eq!(Path::new(&inspected.path), game.canonicalize().unwrap());
    assert!(!inspected.eligible);
    assert!(!inspected.findings.is_empty());
}

#[test]
fn evaluate_build_returns_only_semantic_ui_projection_and_phase_summaries() {
    let bridge = NativeBridge::new(workspace_root().join("manifest"), "chris-recommended");

    let evaluation = bridge
        .evaluate_build(&NormalizedSelection {
            platform: "windows".to_owned(),
            features: Default::default(),
            inputs: Default::default(),
        })
        .expect("evaluate recommended build");

    assert!(!evaluation.view.controls.is_empty());
    assert!(evaluation.selected_choice_count > 0);
    assert!(evaluation
        .plan
        .phases
        .iter()
        .any(|phase| phase.id == "main"));
    let serialized = serde_json::to_string(&evaluation).expect("serialize evaluation");
    assert!(!serialized.contains("\"components\":"));
}
