use std::path::PathBuf;

use bg_engine::manifest::{
    AcquisitionPolicy, ArchiveRootRule, Decision, InputValue, Phase, Readiness, SourceKind,
};
use bg_engine::recipe_view::{evaluate, evaluate_preset};
use bg_engine::resolve::Selection;
use bg_engine::Manifest;

const SCS_AUTHORED: &[u32] = &[
    2000, 2500, 2510, 3040, 3041, 3505, 3540, 3541, 4000, 4115, 4210, 4215, 4218, 4240, 4250, 5000,
    5900, 6000, 6010, 6030, 6040, 6100, 6200, 6300, 6310, 6320, 6500, 6510, 6520, 6540, 6550, 6560,
    6570, 6580, 6590, 6800, 6810, 6820, 6830, 6840, 6850, 7000, 7010, 7020, 7030, 7040, 7050, 7060,
    7070, 7080, 7090, 7100, 7110, 7130, 7140, 7150, 7200, 7210, 7220, 7230, 7250, 7900, 8000, 8010,
    8020, 8040, 8050, 8060, 8070, 8080, 8085, 8090, 8100, 8110, 8120, 8130, 8140, 8150, 8160, 8170,
    8180, 8190,
];

const SCS_RECOMMENDED: &[u32] = &[
    2000, 2500, 2510, 3041, 3505, 3540, 4000, 4115, 4215, 4218, 4250, 5900, 6000, 6010, 6030, 6040,
    6100, 6200, 6300, 6310, 6320, 6500, 6510, 6520, 6540, 6550, 6560, 6570, 6580, 6590, 6800, 6810,
    6820, 6830, 6840, 6850, 7000, 7010, 7020, 7030, 7040, 7050, 7060, 7070, 7080, 7090, 7100, 7110,
    7130, 7140, 7150, 7200, 7210, 7220, 7230, 7250, 7900, 8010, 8020, 8040, 8050, 8060, 8070, 8080,
    8085, 8100, 8110, 8120, 8130, 8140, 8150, 8160, 8170, 8180, 8190,
];

const RANDOMISER_AUTHORED: &[u32] = &[
    500, 510, 530, 540, 560, 570, 9000, 10200, 10210, 10300, 1100,
];
const RANDOMISER_RECOMMENDED: &[u32] = &[500, 530, 540, 560, 570, 9000, 10200, 10210, 1100];

fn recipe_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../manifest")
}

fn recipe() -> Manifest {
    Manifest::load(&recipe_root()).expect("production recipe should load")
}

#[test]
fn freezes_the_reviewed_scs_randomiser_and_buffbot_artifacts() {
    let manifest = recipe();
    let expected = [
        (
            "stratagems-35.21",
            "35.21",
            "https://github.com/Gibberlings3/SwordCoastStratagems/releases/download/v35.21/lin-stratagems-35.21.zip",
            "v35.21",
            "lin-stratagems-35.21.zip",
            58_517_195,
            "dd6b0dbe3b9f65cf70e5f57601ff667c791e968a62136254292e7521a6be99cb",
            "stratagems",
            "stratagems/setup-stratagems.tp2",
        ),
        (
            "randomiser-8.1.1",
            "8.1.1",
            "https://github.com/Chrizhermann/chriz-item-randomiser/releases/download/v8.1.1/randomiser-v8.1.1.zip",
            "v8.1.1",
            "randomiser-v8.1.1.zip",
            3_447_259,
            "b5d95e801269ff6e675b6c2c829f897f0769986c8577384f095accc420e7c16b",
            "randomiser",
            "randomiser/randomiser.tp2",
        ),
        (
            "buffbot-1.8.3-alpha",
            "1.8.3-alpha",
            "https://github.com/Chrizhermann/bg-eeex-buffbot/releases/download/v1.8.3-alpha/buffbot-v1.8.3-alpha.zip",
            "v1.8.3-alpha",
            "buffbot-v1.8.3-alpha.zip",
            4_240_743,
            "fe6e0b4cd393a608d8c3df75f26993106f3550e954030a7b66dbfd28956820a7",
            "buffbot",
            "buffbot/setup-buffbot.tp2",
        ),
    ];

    for (id, version, url, reference, filename, length, sha256, root, tp2) in expected {
        let artifact = manifest
            .artifacts
            .get(id)
            .unwrap_or_else(|| panic!("missing artifact {id}"));
        assert_eq!(artifact.version, version);
        assert_eq!(artifact.acquisition, AcquisitionPolicy::FetchOnly);
        assert_eq!(artifact.source.kind, SourceKind::GithubRelease);
        assert_eq!(artifact.source.url, url);
        assert_eq!(artifact.source.reference, reference);
        assert_eq!(artifact.source.expected_filename.as_deref(), Some(filename));
        assert_eq!(artifact.source.expected_length, Some(length));
        assert_eq!(artifact.source.sha256, sha256);
        assert_eq!(artifact.archive.root_rule, ArchiveRootRule::Direct);
        assert_eq!(artifact.archive.publish_roots, [root]);
        assert_eq!(artifact.archive.tp2_paths, [tp2]);
    }
}

#[test]
fn authors_the_tail_in_dependency_order_and_keeps_buffbot_absolute_last() {
    let manifest = recipe();
    let run_ids = manifest
        .collection
        .runs
        .iter()
        .map(|run| run.run_id.as_str())
        .collect::<Vec<_>>();
    let scs = run_ids
        .iter()
        .position(|id| *id == "stratagems-bg2")
        .unwrap();
    let randomiser = run_ids
        .iter()
        .position(|id| *id == "randomiser-bg2")
        .unwrap();
    let eet_end = run_ids.iter().position(|id| *id == "eet-end-bg2").unwrap();
    let buffbot = run_ids.iter().position(|id| *id == "buffbot-bg2").unwrap();
    assert!(scs < randomiser && randomiser < eet_end && eet_end < buffbot);
    assert_eq!(buffbot, run_ids.len() - 1);

    let runs = &manifest.collection.runs;
    assert_eq!(runs[scs].phase, Phase::Main);
    assert_eq!(runs[scs].components, SCS_AUTHORED);
    assert_eq!(runs[randomiser].phase, Phase::Main);
    assert_eq!(runs[randomiser].components, RANDOMISER_AUTHORED);
    assert_eq!(runs[eet_end].phase, Phase::EetFinalization);
    assert_eq!(runs[buffbot].phase, Phase::PostEetEnd);
    assert_eq!(runs[buffbot].components, [1, 0]);
}

#[test]
fn recommended_preset_resolves_exact_curated_defaults_and_blocks_the_duplicate_fix() {
    let manifest = recipe();
    let evaluation = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
    assert_eq!(
        evaluation.plan.components_for("stratagems-bg2"),
        Some(SCS_RECOMMENDED)
    );
    assert_eq!(
        evaluation.plan.components_for("randomiser-bg2"),
        Some(RANDOMISER_RECOMMENDED)
    );
    assert_eq!(
        evaluation.plan.components_for("buffbot-bg2"),
        Some(&[1, 0][..])
    );

    let duplicate = evaluation
        .view
        .control("feature:randomiser:component-10300")
        .expect("Randomiser duplicate-fix control");
    assert_eq!(duplicate.decision, Decision::Default);
    assert_eq!(duplicate.readiness, Readiness::Ready);
    assert!(!duplicate.selected);
    assert!(!duplicate.interactive);
    assert_eq!(
        duplicate.unavailable_reason.as_deref(),
        Some("Already provided by SCS component 8040 (Improved random spawns).")
    );
}

#[test]
fn randomiser_percentage_is_typed_and_scs_spell_revision_conflict_is_authored() {
    let manifest = recipe();
    let randomiser = manifest
        .collection
        .features
        .iter()
        .find(|feature| feature.id == "feature:randomiser:component-510")
        .expect("Randomiser percentage feature");
    let input = randomiser
        .inputs
        .first()
        .expect("Randomiser percentage input");
    let serialized = toml::to_string(input).unwrap();
    assert!(serialized.contains("kind = \"integer\""));
    assert!(serialized.contains("min = 0"));
    assert!(serialized.contains("max = 100"));

    let component = manifest.mods["randomiser"]
        .components
        .iter()
        .find(|component| component.id == 510)
        .expect("Randomiser component 510");
    assert_eq!(component.prompts.len(), 1);
    assert_eq!(
        component.prompts[0].expected_output,
        "Please enter the chance for items to randomly not be randomised as a integet number (e.g. 10 for 10%)"
    );

    let scs_hla = manifest
        .collection
        .features
        .iter()
        .find(|feature| feature.id == "feature:stratagems:component-4240")
        .expect("SCS HLA feature");
    assert!(scs_hla
        .conflicts
        .iter()
        .any(|conflict| conflict.feature_id == "mod:spell-rev"
            && conflict.reason == "Unavailable with Spell Revisions."));
}

#[test]
fn tail_toggles_and_randomiser_input_boundaries_resolve_semantically() {
    let manifest = recipe();
    let mut selection = Selection::defaults("windows");
    selection.set_feature("feature:stratagems:component-3041", false);
    selection.set_feature("feature:stratagems:component-3040", true);
    selection.set_feature("feature:stratagems:component-3540", false);
    selection.set_feature("feature:stratagems:component-3541", true);
    selection.set_feature("feature:randomiser:component-510", true);
    selection.set_input(
        "feature:randomiser:component-510",
        "chance-percent",
        InputValue::Integer(0),
    );
    let evaluation = evaluate(&manifest, &selection).unwrap();
    let scs = evaluation.plan.components_for("stratagems-bg2").unwrap();
    assert!(scs.contains(&3040));
    assert!(!scs.contains(&3041));
    assert!(scs.contains(&3541));
    assert!(!scs.contains(&3540));
    let randomiser = evaluation
        .plan
        .runs
        .iter()
        .find(|run| run.run_id == "randomiser-bg2")
        .unwrap();
    assert!(randomiser.components.contains(&510));
    let prompt = randomiser
        .prompt_scripts
        .iter()
        .find(|script| script.component.component == 510)
        .unwrap();
    assert_eq!(prompt.steps[0].answer, "0\n");

    selection.set_input(
        "feature:randomiser:component-510",
        "chance-percent",
        InputValue::Integer(100),
    );
    let evaluation = evaluate(&manifest, &selection).unwrap();
    let prompt = evaluation
        .plan
        .runs
        .iter()
        .find(|run| run.run_id == "randomiser-bg2")
        .unwrap()
        .prompt_scripts
        .iter()
        .find(|script| script.component.component == 510)
        .unwrap();
    assert_eq!(prompt.steps[0].answer, "100\n");

    selection.set_input(
        "feature:randomiser:component-510",
        "chance-percent",
        InputValue::Integer(101),
    );
    assert!(evaluate(&manifest, &selection).is_err());
}
