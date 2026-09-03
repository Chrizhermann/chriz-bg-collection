use std::collections::BTreeMap;
use std::path::PathBuf;

use bg_engine::digest::plan_digest;
use bg_engine::manifest::{
    AcquisitionPolicy, ArchiveKind, ArchiveRootRule, Decision, GameRoot, PeMachine, Phase,
    Postcondition, Readiness, RunArg, SourceKind,
};
use bg_engine::recipe_view::evaluate_preset;
use bg_engine::resolve::Selection;
use bg_engine::validate::validate;
use bg_engine::Manifest;
use serde::Deserialize;

const WEIDU_SHA256: &str = "b156910cbec69359fc2e42f6739aa959d49047d6fd3dc172f6bed88ffad8f927";
const DLCMERGER_SHA256: &str = "ea7584fd7285330cfaa52cfc217677ef4f8cf8151fdec666ea8af199e479db0e";
const EEFIXPACK_SHA256: &str = "7a57e43bd6c30c7e36b7c2b7c5531c7a0834de164da186808da03d24474592bf";
const BG1UB_SHA256: &str = "5a525eb37f68f63706a5120fc4900b68b00b46ff755583fa6ced1fc456cf8104";
const BG1NPC_SHA256: &str = "8e34f397e960b1b990350b5ef101562db066cbe91d8c35eda60baf24f452b1bc";
const EET_SHA256: &str = "9834a53322b7fe9d8923bfde36c0e6bc8bef54d730d32ac29286b80d6f08287e";

const BG1UB_COMPONENTS: &[u32] = &[
    0, 11, 12, 13, 14, 16, 17, 18, 19, 21, 22, 29, 30, 32, 33, 34,
];
const BG1NPC_AUTHORED_COMPONENTS: &[u32] = &[0, 10, 90, 111, 120, 130, 240, 160, 200];
const BG1NPC_RESOLVED_COMPONENTS: &[u32] = &[0, 10, 90, 111, 120, 130, 240, 200];

fn recipe_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../manifest")
}

fn recipe() -> Manifest {
    Manifest::load(&recipe_root()).expect("production recipe should load")
}

#[test]
fn pins_exact_core_artifact_identities_and_archive_contracts() {
    let manifest = recipe();
    let expected = BTreeMap::from([
        (
            "weidu-249-amd64",
            (
                "249.00",
                "https://github.com/WeiDUorg/weidu/releases/download/v249.00/WeiDU-Windows-249-amd64.zip",
                "v249.00",
                "WeiDU-Windows-249-amd64.zip",
                2_154_755,
                WEIDU_SHA256,
                ArchiveKind::Zip,
                ArchiveRootRule::SingleWrapper,
                vec!["weidu.exe"],
                Vec::<&str>::new(),
            ),
        ),
        (
            "dlcmerger-2.1",
            (
                "2.1",
                "https://github.com/Argent77/A7-DlcMerger/releases/download/v2.1/DlcMerger-v2.1.iemod",
                "v2.1",
                "DlcMerger-v2.1.iemod",
                125_691,
                DLCMERGER_SHA256,
                ArchiveKind::Iemod,
                ArchiveRootRule::Direct,
                vec!["DlcMerger"],
                vec!["DlcMerger/DlcMerger.tp2"],
            ),
        ),
        (
            "eefixpack-beta2",
            (
                "Beta 2",
                "https://github.com/Gibberlings3/EE_Fixpack/releases/download/Beta_2/enhanced-edition-fixpack-beta-2.iemod",
                "Beta_2",
                "enhanced-edition-fixpack-beta-2.iemod",
                130_716_647,
                EEFIXPACK_SHA256,
                ArchiveKind::Iemod,
                ArchiveRootRule::Direct,
                vec!["eefixpack"],
                vec!["eefixpack/setup-eefixpack.tp2"],
            ),
        ),
        (
            "bg1ub-17.1",
            (
                "17.1",
                "https://github.com/Pocket-Plane-Group/bg1ub/releases/download/v17.1/bg1-unfinished-business-v17.1.iemod",
                "v17.1",
                "bg1-unfinished-business-v17.1.iemod",
                14_579_031,
                BG1UB_SHA256,
                ArchiveKind::Iemod,
                ArchiveRootRule::Direct,
                vec!["bg1ub"],
                vec!["bg1ub/bg1ub.tp2"],
            ),
        ),
        (
            "bg1npc-32",
            (
                "32",
                "https://github.com/Gibberlings3/BG1NPC/releases/download/v32/the-bg1npc-project-v32.iemod",
                "v32",
                "the-bg1npc-project-v32.iemod",
                27_617_013,
                BG1NPC_SHA256,
                ArchiveKind::Iemod,
                ArchiveRootRule::Direct,
                vec!["bg1npc"],
                vec!["bg1npc/bg1npc.tp2"],
            ),
        ),
        (
            "eet-official-74e91d72bca5d073fa11c1d088b90d7ff0c7105d",
            (
                "74e91d72bca5d073fa11c1d088b90d7ff0c7105d",
                "https://codeload.github.com/Gibberlings3/EET/zip/74e91d72bca5d073fa11c1d088b90d7ff0c7105d",
                "74e91d72bca5d073fa11c1d088b90d7ff0c7105d",
                "EET-74e91d72bca5d073fa11c1d088b90d7ff0c7105d.zip",
                211_137_765,
                EET_SHA256,
                ArchiveKind::Zip,
                ArchiveRootRule::SingleWrapper,
                vec!["EET", "EET_end"],
                vec![
                    "EET/EET.tp2",
                    "EET/other/BGEE_to_EET_mod_checker/BGEE_to_EET_mod_checker/BGEE_to_EET_mod_checker.tp2",
                    "EET/other/EET_modConverter/EET_modConverter/EET_modConverter.tp2",
                    "EET_end/EET_end.tp2",
                ],
            ),
        ),
    ]);

    assert_eq!(manifest.artifacts.len(), expected.len());
    for (id, (version, url, reference, filename, length, sha256, kind, root_rule, roots, tp2s)) in
        expected
    {
        let artifact = &manifest.artifacts[id];
        assert_eq!(artifact.version, version, "{id} version");
        assert_eq!(
            artifact.acquisition,
            AcquisitionPolicy::FetchOnly,
            "{id} policy"
        );
        assert_eq!(
            artifact.source.kind,
            if id.starts_with("eet-") {
                SourceKind::GithubCommitZip
            } else {
                SourceKind::GithubRelease
            },
            "{id} source kind"
        );
        assert_eq!(artifact.source.url, url, "{id} URL");
        assert_eq!(artifact.source.reference, reference, "{id} reference");
        assert_eq!(
            artifact.source.expected_filename.as_deref(),
            Some(filename),
            "{id} filename"
        );
        assert_eq!(artifact.source.expected_length, Some(length), "{id} length");
        assert_eq!(artifact.source.sha256, sha256, "{id} SHA-256");
        assert_eq!(artifact.archive.kind, kind, "{id} archive kind");
        assert_eq!(artifact.archive.root_rule, root_rule, "{id} root rule");
        assert_eq!(artifact.archive.publish_roots, roots, "{id} publish roots");
        assert_eq!(artifact.archive.tp2_paths, tp2s, "{id} TP2 paths");
        assert_eq!(
            artifact.provenance.reviewed_on, "2026-09-03",
            "{id} review date"
        );
    }

    let weidu = &manifest.artifacts["weidu-249-amd64"];
    let tool = weidu
        .tool
        .as_ref()
        .expect("WeiDU must be an executable tool");
    assert_eq!(tool.executable, "weidu.exe");
    assert_eq!(tool.pe_machine, PeMachine::X86_64);
    assert!(
        manifest
            .artifacts
            .values()
            .filter(|artifact| artifact.tool.is_some())
            .count()
            == 1
    );
    assert!(validate(&manifest).is_empty(), "{:#?}", validate(&manifest));
}

#[test]
fn authors_the_exact_seven_run_core_spine_and_eet_boundaries() {
    let manifest = recipe();
    let expected = [
        (
            "dlcmerger-bg1",
            "dlcmerger",
            Phase::Bg1Preparation,
            &[1][..],
        ),
        (
            "eefixpack-bg1",
            "eefixpack",
            Phase::Bg1Preparation,
            &[0, 2][..],
        ),
        (
            "bg1ub-bg1",
            "bg1ub",
            Phase::Bg1Preparation,
            BG1UB_COMPONENTS,
        ),
        (
            "bg1npc-bg1",
            "bg1npc",
            Phase::Bg1Preparation,
            BG1NPC_AUTHORED_COMPONENTS,
        ),
        (
            "eefixpack-bg2",
            "eefixpack",
            Phase::Bg2Preparation,
            &[0, 2][..],
        ),
        (
            "eet-initialize-bg2",
            "eet",
            Phase::EetInitialization,
            &[0, 100][..],
        ),
        ("eet-end-bg2", "eet-end", Phase::EetFinalization, &[0][..]),
    ];

    assert_eq!(manifest.collection.runs.len(), expected.len());
    for (run, (run_id, mod_id, phase, components)) in manifest.collection.runs.iter().zip(expected)
    {
        assert_eq!(run.run_id, run_id);
        assert_eq!(run.mod_id, mod_id);
        assert_eq!(run.phase, phase);
        assert_eq!(
            run.phase.game_root(),
            if phase == Phase::Bg1Preparation {
                GameRoot::Bg1
            } else {
                GameRoot::Bg2
            }
        );
        assert_eq!(run.components, components);
    }

    let dlc = &manifest.collection.runs[0];
    assert_eq!(
        dlc.postconditions,
        vec![Postcondition::TextFileMarkers {
            path: "override/ui.menu".to_owned(),
            required: vec!["Infinity_GetFileExists(".to_owned()],
            forbidden: vec!["Infinity_GetFileExistsInZip(".to_owned()],
            max_bytes: 16 * 1024 * 1024,
        }]
    );
    let eet = &manifest.collection.runs[5];
    assert_eq!(
        eet.args,
        vec![
            RunArg::Literal("--args-list".to_owned()),
            RunArg::Literal("p".to_owned()),
            RunArg::StagedRoot(GameRoot::Bg1),
        ]
    );
    assert_eq!(
        manifest.collection.runs.last().unwrap().run_id,
        "eet-end-bg2"
    );
}

#[test]
fn recommended_defaults_resolve_the_core_spine_but_keep_bg1npc_160_visible_and_omitted() {
    let manifest = recipe();
    let evaluation = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
    let plan = &evaluation.plan;

    assert_eq!(plan.runs.len(), 7);
    assert_eq!(plan.components_for("dlcmerger-bg1"), Some(&[1][..]));
    assert_eq!(plan.components_for("eefixpack-bg1"), Some(&[0, 2][..]));
    assert_eq!(plan.components_for("bg1ub-bg1"), Some(BG1UB_COMPONENTS));
    assert_eq!(
        plan.components_for("bg1npc-bg1"),
        Some(BG1NPC_RESOLVED_COMPONENTS)
    );
    assert_eq!(plan.components_for("eefixpack-bg2"), Some(&[0, 2][..]));
    assert_eq!(
        plan.components_for("eet-initialize-bg2"),
        Some(&[0, 100][..])
    );
    assert_eq!(plan.components_for("eet-end-bg2"), Some(&[0][..]));

    let blocked = evaluation
        .view
        .control("feature:bg1npc:component-160")
        .unwrap();
    assert_eq!(blocked.decision, Decision::Default);
    assert_eq!(blocked.readiness, Readiness::Blocked);
    assert!(!blocked.selected);
    assert!(!blocked.interactive);
    assert!(blocked
        .unavailable_reason
        .as_deref()
        .unwrap()
        .contains("copyright"));
    assert!(evaluation.findings.iter().any(|finding| {
        finding.rule == "blocked-default-omitted"
            && finding.feature_id == "feature:bg1npc:component-160"
    }));

    let first_digest = plan_digest(plan).unwrap();
    let second = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
    assert_eq!(first_digest, plan_digest(&second.plan).unwrap());
}

#[test]
fn mandatory_core_groups_cannot_be_disabled() {
    let manifest = recipe();
    for feature_id in ["mod:dlcmerger", "mod:eefixpack", "mod:eet", "mod:eet-end"] {
        let mut selection = Selection::defaults("windows");
        selection.set_feature(feature_id, false);
        let evaluation = bg_engine::recipe_view::evaluate(&manifest, &selection).unwrap();
        let control = evaluation.view.control(feature_id).unwrap();
        assert_eq!(control.decision, Decision::Mandatory);
        assert!(control.selected);
        assert!(!control.interactive);
    }
}

#[test]
fn eet_and_eet_end_share_one_payload_and_every_run_uses_the_pinned_x64_tool() {
    let manifest = recipe();
    let eet_artifact = &manifest.artifacts["eet-official-74e91d72bca5d073fa11c1d088b90d7ff0c7105d"];
    assert_eq!(
        manifest.mods["eet"].artifact_id,
        manifest.mods["eet-end"].artifact_id
    );
    assert_eq!(
        manifest.mods["eet"].artifact_id,
        "eet-official-74e91d72bca5d073fa11c1d088b90d7ff0c7105d"
    );
    assert!(manifest
        .mods
        .values()
        .all(|mod_file| mod_file.weidu_artifact_id == "weidu-249-amd64"));
    assert!(!eet_artifact
        .archive
        .publish_roots
        .iter()
        .any(|root| root.eq_ignore_ascii_case("EET_gui")));
    for auxiliary in [
        "EET/other/BGEE_to_EET_mod_checker/BGEE_to_EET_mod_checker/BGEE_to_EET_mod_checker.tp2",
        "EET/other/EET_modConverter/EET_modConverter/EET_modConverter.tp2",
    ] {
        assert!(eet_artifact
            .archive
            .tp2_paths
            .iter()
            .any(|tp2| tp2 == auxiliary));
        assert!(manifest
            .mods
            .values()
            .all(|mod_file| mod_file.tp2 != auxiliary));
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NestedToolEvidence {
    schema: u32,
    evidence_kind: String,
    artifact_id: String,
    artifact_sha256: String,
    path: String,
    length: u64,
    sha256: String,
    pe_machine: String,
    equals_artifact_id: String,
    equals_path: String,
    equals_sha256: String,
    verified_on: String,
}

#[test]
fn records_exact_eet_nested_x64_weidu_equality_without_making_eet_the_installer_tool() {
    let manifest = recipe();
    let evidence_path = recipe_root().join("evidence/eet-nested-weidu.toml");
    let evidence: NestedToolEvidence = toml::from_str(
        &std::fs::read_to_string(&evidence_path).expect("nested WeiDU evidence should exist"),
    )
    .expect("nested WeiDU evidence should parse");

    assert_eq!(evidence.schema, 1);
    assert_eq!(evidence.evidence_kind, "nested-tool-byte-equality");
    assert_eq!(
        evidence.artifact_id,
        "eet-official-74e91d72bca5d073fa11c1d088b90d7ff0c7105d"
    );
    assert_eq!(evidence.artifact_sha256, EET_SHA256);
    assert_eq!(evidence.path, "EET/bin/win32/x86_64/weidu.exe");
    assert_eq!(evidence.length, 1_364_992);
    assert_eq!(
        evidence.sha256,
        "ad70f5897a6d0ba4b0d226f845a9b14cf345f56cc9697ca8d05cac9fe4932c1a"
    );
    assert_eq!(evidence.pe_machine, "x86-64");
    assert_eq!(evidence.equals_artifact_id, "weidu-249-amd64");
    assert_eq!(evidence.equals_path, "weidu.exe");
    assert_eq!(evidence.equals_sha256, evidence.sha256);
    assert_eq!(evidence.verified_on, "2026-09-03");
    assert!(manifest.artifacts[&evidence.artifact_id].tool.is_none());
}
