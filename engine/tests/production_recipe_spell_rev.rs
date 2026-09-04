use std::path::PathBuf;

use bg_engine::manifest::{
    AcquisitionPolicy, ArchiveRootRule, Decision, Phase, Readiness, SourceKind,
};
use bg_engine::recipe_view::{evaluate, evaluate_preset};
use bg_engine::resolve::Selection;
use bg_engine::Manifest;

const SPELL_REV_MENU_COMPONENTS: &[u32] = &[0, 10, 20, 30, 55, 60, 65, 70];
const SPELL_REV_EARLY_COMPONENTS: &[u32] = &[0, 10, 20, 30, 55, 65];
const SPELL_REV_LATE_COMPONENTS: &[u32] = &[60];

fn recipe_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../manifest")
}

fn recipe() -> Manifest {
    Manifest::load(&recipe_root()).expect("production recipe should load")
}

#[test]
fn freezes_the_reviewed_spell_revisions_successor_release() {
    let manifest = recipe();
    let artifact = &manifest.artifacts["spell-rev-4.21-chriz.3"];

    assert_eq!(artifact.version, "4.21-chriz.3");
    assert_eq!(artifact.acquisition, AcquisitionPolicy::FetchOnly);
    assert_eq!(artifact.source.kind, SourceKind::GithubRelease);
    assert_eq!(artifact.source.reference, "v4.21-chriz.3");
    assert_eq!(
        artifact.source.url,
        "https://github.com/Chrizhermann/chriz-spell-revisions-patch/releases/download/v4.21-chriz.3/spell_rev-v4-21-chriz-3.zip"
    );
    assert_eq!(
        artifact.source.expected_filename.as_deref(),
        Some("spell_rev-v4-21-chriz-3.zip")
    );
    assert_eq!(artifact.source.expected_length, Some(7_581_145));
    assert_eq!(
        artifact.source.sha256,
        "14e3801fc88e079a56fc7d37da5c86e29bedc32d192b69b50a2ceaac51756abe"
    );
    assert_eq!(artifact.archive.root_rule, ArchiveRootRule::Direct);
    assert_eq!(artifact.archive.publish_roots, ["spell_rev"]);
    assert_eq!(
        artifact.archive.tp2_paths,
        ["spell_rev/setup-spell_rev.tp2"]
    );
    assert!(artifact
        .archive
        .publish_roots
        .iter()
        .all(|root| !root.to_ascii_lowercase().ends_with(".exe")));
}

#[test]
fn authors_the_exact_spell_revisions_menu_and_split_run_order() {
    let manifest = recipe();
    let installer = &manifest.mods["spell-rev"];
    assert_eq!(installer.artifact_id, "spell-rev-4.21-chriz.3");
    assert_eq!(installer.tp2, "spell_rev/setup-spell_rev.tp2");
    assert_eq!(installer.language, 0);
    assert_eq!(installer.weidu_artifact_id, "weidu-249-amd64");
    assert_eq!(
        installer
            .components
            .iter()
            .map(|component| component.id)
            .collect::<Vec<_>>(),
        SPELL_REV_MENU_COMPONENTS
    );

    let runs = &manifest.collection.runs;
    let position = |id: &str| {
        runs.iter()
            .position(|run| run.run_id == id)
            .unwrap_or_else(|| panic!("missing run {id}"))
    };
    let ascension = position("ascension-bg2");
    let early = position("spell-rev-core-bg2");
    let modpack = position("chriz-bg-modpack-bg2");
    let late = position("spell-rev-npc-spellbooks-bg2");
    let buffbot = position("buffbot-bg2");

    assert_eq!(ascension + 1, early);
    assert_eq!(modpack + 1, late);
    assert_eq!(late + 1, buffbot);
    assert_eq!(buffbot, runs.len() - 1);
    assert_eq!(runs[early].phase, Phase::Main);
    assert_eq!(runs[early].components, SPELL_REV_EARLY_COMPONENTS);
    assert_eq!(runs[late].phase, Phase::PostEetEnd);
    assert_eq!(runs[late].components, SPELL_REV_LATE_COMPONENTS);
    assert!(runs
        .iter()
        .filter(|run| run.mod_id == "spell-rev")
        .all(|run| !run.components.contains(&70)));
}

#[test]
fn exposes_ready_curated_features_and_keeps_component_70_excluded() {
    let manifest = recipe();
    let parent = manifest
        .collection
        .features
        .iter()
        .find(|feature| feature.id == "mod:spell-rev")
        .expect("Spell Revisions parent");
    assert_eq!(parent.decision, Decision::Default);
    assert_eq!(parent.readiness, Readiness::Ready);

    let mandatory = manifest
        .collection
        .features
        .iter()
        .find(|feature| feature.id == "feature:spell-rev:mandatory-components")
        .expect("Spell Revisions mandatory core");
    assert_eq!(mandatory.decision, Decision::Mandatory);
    assert_eq!(mandatory.readiness, Readiness::Ready);
    assert_eq!(mandatory.parent.as_deref(), Some("mod:spell-rev"));
    assert_eq!(mandatory.components.len(), 1);
    assert_eq!(mandatory.components[0].run_id, "spell-rev-core-bg2");
    assert_eq!(mandatory.components[0].component, 0);

    for (component, run_id) in [
        (10, "spell-rev-core-bg2"),
        (20, "spell-rev-core-bg2"),
        (30, "spell-rev-core-bg2"),
        (55, "spell-rev-core-bg2"),
        (65, "spell-rev-core-bg2"),
        (60, "spell-rev-npc-spellbooks-bg2"),
    ] {
        let id = format!("feature:spell-rev:component-{component}");
        let feature = manifest
            .collection
            .features
            .iter()
            .find(|feature| feature.id == id)
            .unwrap_or_else(|| panic!("missing feature {id}"));
        assert_eq!(feature.decision, Decision::Default, "{id}");
        assert_eq!(feature.readiness, Readiness::Ready, "{id}");
        assert_eq!(feature.parent.as_deref(), Some("mod:spell-rev"), "{id}");
        assert_eq!(feature.components.len(), 1, "{id}");
        assert_eq!(feature.components[0].run_id, run_id, "{id}");
        assert_eq!(feature.components[0].component, component, "{id}");
    }

    assert!(manifest
        .collection
        .features
        .iter()
        .all(|feature| feature.id != "feature:spell-rev:component-70"));
}

#[test]
fn recommended_preset_selects_both_runs_and_parent_disable_removes_both() {
    let manifest = recipe();
    let recommended = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
    assert_eq!(
        recommended.plan.components_for("spell-rev-core-bg2"),
        Some(SPELL_REV_EARLY_COMPONENTS)
    );
    assert_eq!(
        recommended
            .plan
            .components_for("spell-rev-npc-spellbooks-bg2"),
        Some(SPELL_REV_LATE_COMPONENTS)
    );
    assert!(recommended.plan.components_for("rr-bg2").is_some());

    let mut without_spell_revisions = Selection::defaults("windows");
    without_spell_revisions.set_feature("mod:spell-rev", false);
    let plan = evaluate(&manifest, &without_spell_revisions).unwrap().plan;
    assert_eq!(plan.components_for("spell-rev-core-bg2"), None);
    assert_eq!(plan.components_for("spell-rev-npc-spellbooks-bg2"), None);
}

#[test]
fn preserves_the_scs_conflict_without_blocking_deferred_rr_compatibility() {
    let manifest = recipe();
    let scs_hlas = manifest
        .collection
        .features
        .iter()
        .find(|feature| feature.id == "feature:stratagems:component-4240")
        .expect("SCS innate HLA feature");
    assert!(scs_hlas.conflicts.iter().any(|conflict| {
        conflict.feature_id == "mod:spell-rev"
            && conflict.reason == "Unavailable with Spell Revisions."
    }));

    let spell_revisions = manifest
        .collection
        .features
        .iter()
        .find(|feature| feature.id == "mod:spell-rev")
        .expect("Spell Revisions parent");
    assert!(spell_revisions
        .conflicts
        .iter()
        .all(|conflict| !conflict.feature_id.starts_with("feature:rr:")));
    assert!(manifest
        .collection
        .features
        .iter()
        .filter(|feature| feature.id.starts_with("feature:rr:"))
        .flat_map(|feature| &feature.conflicts)
        .all(|conflict| conflict.feature_id != "mod:spell-rev"));

    for feature_id in [
        "feature:artisanskitpack:component-8101",
        "feature:artisanskitpack-npc:component-5102",
        "feature:artisanskitpack-npc:component-10004",
    ] {
        let feature = manifest
            .collection
            .features
            .iter()
            .find(|feature| feature.id == feature_id)
            .unwrap_or_else(|| panic!("missing Artisan compatibility feature {feature_id}"));
        assert!(feature.conflicts.iter().any(|conflict| {
            conflict.feature_id == "feature:spell-rev:mandatory-components"
                && conflict.reason == "Unavailable with Spell Revisions."
        }));
    }
}
