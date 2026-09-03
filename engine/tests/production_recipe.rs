use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Command;

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
use zip::ZipArchive;

const WEIDU_SHA256: &str = "b156910cbec69359fc2e42f6739aa959d49047d6fd3dc172f6bed88ffad8f927";
const WEIDU_EXE_SHA256: &str = "ad70f5897a6d0ba4b0d226f845a9b14cf345f56cc9697ca8d05cac9fe4932c1a";
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

    let expected_limits = BTreeMap::from([
        ("weidu-249-amd64", (8, 64, 8 << 20, 16 << 20, 100)),
        ("dlcmerger-2.1", (8, 128, 1 << 20, 4 << 20, 100)),
        ("eefixpack-beta2", (16, 8_192, 16 << 20, 256 << 20, 100)),
        ("bg1ub-17.1", (16, 2_048, 16 << 20, 64 << 20, 100)),
        ("bg1npc-32", (16, 4_096, 8 << 20, 128 << 20, 1_000)),
        (
            "eet-official-74e91d72bca5d073fa11c1d088b90d7ff0c7105d",
            (16, 2_048, 64 << 20, 512 << 20, 100),
        ),
    ]);
    for (id, expected) in expected_limits {
        let limits = &manifest.artifacts[id].archive.limits;
        assert_eq!(
            (
                limits.max_depth,
                limits.max_entries,
                limits.max_entry_uncompressed_bytes,
                limits.max_total_uncompressed_bytes,
                limits.max_compression_ratio,
            ),
            expected,
            "{id} archive limits"
        );
    }

    let weidu = &manifest.artifacts["weidu-249-amd64"];
    let tool = weidu
        .tool
        .as_ref()
        .expect("WeiDU must be an executable tool");
    assert_eq!(tool.executable, "weidu.exe");
    assert_eq!(tool.expected_length, 1_364_992);
    assert_eq!(tool.sha256, WEIDU_EXE_SHA256);
    assert_eq!(tool.weidu_version, "24900");
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

    let core_runs = expected
        .iter()
        .map(|(run_id, ..)| {
            manifest
                .collection
                .runs
                .iter()
                .position(|run| run.run_id == *run_id)
                .unwrap_or_else(|| panic!("missing core run {run_id}"))
        })
        .collect::<Vec<_>>();
    assert!(
        core_runs.windows(2).all(|pair| pair[0] < pair[1]),
        "core runs must retain their relative order: {core_runs:?}"
    );
    for ((run_id, mod_id, phase, components), index) in expected.into_iter().zip(core_runs) {
        let run = &manifest.collection.runs[index];
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

    let implementation_plan = std::fs::read_to_string(
        recipe_root().join("../docs/plans/2026-09-02-installer-v0-real-alpha-implementation.md"),
    )
    .unwrap();
    assert!(implementation_plan.contains("--args-list p <canonical staged BG1 root>"));
    assert!(!implementation_plan.contains("--args-list sp <canonical staged BG1 root>"));
}

#[test]
fn recommended_defaults_resolve_the_core_spine_but_keep_bg1npc_160_visible_and_omitted() {
    let manifest = recipe();
    let evaluation = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
    let plan = &evaluation.plan;

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

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ArtifactVerificationEvidence {
    schema: u32,
    evidence_kind: String,
    verified_on: String,
    cache_namespace: String,
    extraction_marker_version: u32,
    generated_by: EvidenceGenerator,
    artifacts: Vec<VerifiedArtifactEvidence>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EvidenceGenerator {
    command: String,
    version: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct VerifiedArtifactEvidence {
    artifact_id: String,
    immutable_source_url: String,
    source_reference: String,
    observed_final_host: String,
    actual_length: u64,
    actual_sha256: String,
    wrapper_kind: String,
    #[serde(default)]
    wrapper_directory: Option<String>,
    materialized_publish_roots: Vec<String>,
    declared_tp2_paths: Vec<String>,
    fresh_download: bool,
    fresh_extraction: bool,
    verified_limits: VerifiedLimitsEvidence,
    observed_shape: ObservedArchiveShape,
    #[serde(default)]
    pe: Option<PeEvidence>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct VerifiedLimitsEvidence {
    max_depth: usize,
    max_entries: usize,
    max_entry_uncompressed_bytes: u64,
    max_total_uncompressed_bytes: u64,
    max_compression_ratio: u64,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ObservedArchiveShape {
    max_depth: usize,
    entry_count: usize,
    max_entry_uncompressed_bytes: u64,
    total_uncompressed_bytes: u64,
    max_compression_ratio: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PeEvidence {
    executable: String,
    length: u64,
    sha256: String,
    weidu_version: String,
    version_command: String,
    machine: String,
}

fn artifact_verification_evidence() -> ArtifactVerificationEvidence {
    let evidence_path = recipe_root().join("evidence/artifact-verification.toml");
    toml::from_str(
        &std::fs::read_to_string(&evidence_path)
            .expect("production artifact verification evidence should exist"),
    )
    .expect("production artifact verification evidence should parse")
}

#[test]
fn committed_real_verification_evidence_matches_every_core_artifact_contract() {
    let manifest = recipe();
    let evidence = artifact_verification_evidence();
    let expected_observations = BTreeMap::from([
        (
            "weidu-249-amd64",
            (
                "release-assets.githubusercontent.com",
                Some("WeiDU-Windows"),
                ObservedArchiveShape {
                    max_depth: 5,
                    entry_count: 56,
                    max_entry_uncompressed_bytes: 1_364_992,
                    total_uncompressed_bytes: 3_101_972,
                    max_compression_ratio: 18,
                },
            ),
        ),
        (
            "dlcmerger-2.1",
            (
                "release-assets.githubusercontent.com",
                None,
                ObservedArchiveShape {
                    max_depth: 4,
                    entry_count: 27,
                    max_entry_uncompressed_bytes: 204_800,
                    total_uncompressed_bytes: 261_719,
                    max_compression_ratio: 5,
                },
            ),
        ),
        (
            "eefixpack-beta2",
            (
                "release-assets.githubusercontent.com",
                None,
                ObservedArchiveShape {
                    max_depth: 6,
                    entry_count: 5_234,
                    max_entry_uncompressed_bytes: 6_124_066,
                    total_uncompressed_bytes: 137_694_750,
                    max_compression_ratio: 25,
                },
            ),
        ),
        (
            "bg1ub-17.1",
            (
                "release-assets.githubusercontent.com",
                None,
                ObservedArchiveShape {
                    max_depth: 4,
                    entry_count: 917,
                    max_entry_uncompressed_bytes: 5_779_952,
                    total_uncompressed_bytes: 20_508_513,
                    max_compression_ratio: 13,
                },
            ),
        ),
        (
            "bg1npc-32",
            (
                "release-assets.githubusercontent.com",
                None,
                ObservedArchiveShape {
                    max_depth: 6,
                    entry_count: 1_931,
                    max_entry_uncompressed_bytes: 1_487_545,
                    total_uncompressed_bytes: 55_174_685,
                    max_compression_ratio: 615,
                },
            ),
        ),
        (
            "eet-official-74e91d72bca5d073fa11c1d088b90d7ff0c7105d",
            (
                "codeload.github.com",
                Some("EET-74e91d72bca5d073fa11c1d088b90d7ff0c7105d"),
                ObservedArchiveShape {
                    max_depth: 7,
                    entry_count: 793,
                    max_entry_uncompressed_bytes: 26_266_375,
                    total_uncompressed_bytes: 229_893_521,
                    max_compression_ratio: 25,
                },
            ),
        ),
    ]);

    assert_eq!(evidence.schema, 1);
    assert_eq!(evidence.evidence_kind, "production-artifact-verification");
    assert_eq!(evidence.verified_on, "2026-09-03");
    assert_eq!(evidence.cache_namespace, "sha256-v2");
    assert_eq!(evidence.extraction_marker_version, 2);
    assert_eq!(
        evidence.generated_by.command,
        "chriz-bg-author artifact verify <artifact.toml> --cache-root <new-empty-scratch-dir>"
    );
    assert_eq!(evidence.generated_by.version, "chriz-bg-engine 0.1.0");
    assert_eq!(evidence.artifacts.len(), expected_observations.len());

    let mut seen = BTreeSet::new();
    for record in &evidence.artifacts {
        assert!(
            seen.insert(record.artifact_id.as_str()),
            "duplicate evidence for {}",
            record.artifact_id
        );
        let artifact = manifest
            .artifacts
            .get(&record.artifact_id)
            .unwrap_or_else(|| panic!("unknown evidence artifact {}", record.artifact_id));
        let (expected_host, expected_wrapper, expected_shape) =
            &expected_observations[record.artifact_id.as_str()];

        assert_eq!(record.immutable_source_url, artifact.source.url);
        assert_eq!(record.source_reference, artifact.source.reference);
        assert!(!record.immutable_source_url.contains('?'));
        assert_eq!(&record.observed_final_host, expected_host);
        assert!(!record.observed_final_host.contains(['/', '?', '#']));
        assert_eq!(
            record.actual_length,
            artifact.source.expected_length.unwrap()
        );
        assert_eq!(record.actual_sha256, artifact.source.sha256);
        assert_eq!(
            record.wrapper_kind,
            match artifact.archive.root_rule {
                ArchiveRootRule::Direct => "direct",
                ArchiveRootRule::SingleWrapper => "single-wrapper",
                ArchiveRootRule::DirectOrSingleWrapper => "direct-or-single-wrapper",
            }
        );
        assert_eq!(record.wrapper_directory.as_deref(), *expected_wrapper);
        assert_eq!(
            record.materialized_publish_roots,
            artifact.archive.publish_roots
        );
        assert_eq!(record.declared_tp2_paths, artifact.archive.tp2_paths);
        assert!(record.fresh_download);
        assert!(record.fresh_extraction);
        assert_eq!(
            record.verified_limits,
            VerifiedLimitsEvidence {
                max_depth: artifact.archive.limits.max_depth,
                max_entries: artifact.archive.limits.max_entries,
                max_entry_uncompressed_bytes: artifact.archive.limits.max_entry_uncompressed_bytes,
                max_total_uncompressed_bytes: artifact.archive.limits.max_total_uncompressed_bytes,
                max_compression_ratio: artifact.archive.limits.max_compression_ratio,
            }
        );
        assert_eq!(&record.observed_shape, expected_shape);
        assert!(record.observed_shape.max_depth <= record.verified_limits.max_depth);
        assert!(record.observed_shape.entry_count <= record.verified_limits.max_entries);
        assert!(
            record.observed_shape.max_entry_uncompressed_bytes
                <= record.verified_limits.max_entry_uncompressed_bytes
        );
        assert!(
            record.observed_shape.total_uncompressed_bytes
                <= record.verified_limits.max_total_uncompressed_bytes
        );
        assert!(
            record.observed_shape.max_compression_ratio
                <= record.verified_limits.max_compression_ratio
        );

        match (&artifact.tool, &record.pe) {
            (Some(tool), Some(pe)) => {
                assert_eq!(pe.executable, tool.executable);
                assert_eq!(pe.length, tool.expected_length);
                assert_eq!(pe.sha256, tool.sha256);
                assert_eq!(pe.weidu_version, tool.weidu_version);
                assert_eq!(pe.version_command, "weidu.exe --version");
                assert_eq!(pe.machine, "x86-64");
                assert_eq!(tool.pe_machine, PeMachine::X86_64);
            }
            (None, None) => {}
            _ => panic!("{} PE evidence does not match tool contract", artifact.id),
        }
    }
    assert_eq!(seen, expected_observations.keys().copied().collect());
}

fn measured_archive_shape(path: &Path) -> ObservedArchiveShape {
    let mut archive = ZipArchive::new(File::open(path).expect("open verified archive"))
        .expect("verified archive should remain ZIP-compatible");
    let mut shape = ObservedArchiveShape {
        max_depth: 0,
        entry_count: archive.len(),
        max_entry_uncompressed_bytes: 0,
        total_uncompressed_bytes: 0,
        max_compression_ratio: 0,
    };
    for index in 0..archive.len() {
        let entry = archive.by_index_raw(index).expect("read central directory");
        shape.max_depth = shape.max_depth.max(entry.name().split('/').count());
        if !entry.is_dir() {
            shape.max_entry_uncompressed_bytes =
                shape.max_entry_uncompressed_bytes.max(entry.size());
            shape.total_uncompressed_bytes = shape
                .total_uncompressed_bytes
                .checked_add(entry.size())
                .expect("archive size should fit u64");
            let compressed = entry.compressed_size().max(1);
            let ratio = entry.size().div_ceil(compressed);
            shape.max_compression_ratio = shape.max_compression_ratio.max(ratio);
        }
    }
    shape
}

#[test]
#[ignore = "requires official HTTPS or a prepopulated external artifact cache"]
fn real_artifact_verification_matches_committed_evidence() {
    let evidence = artifact_verification_evidence();
    let cache_root = PathBuf::from(
        std::env::var_os("CHRIZ_BG_REAL_ARTIFACT_CACHE")
            .expect("set CHRIZ_BG_REAL_ARTIFACT_CACHE to an external scratch directory"),
    );
    assert!(
        cache_root.is_absolute(),
        "real artifact cache must be absolute"
    );
    let repository_root = recipe_root()
        .parent()
        .expect("manifest should have a repository parent")
        .to_path_buf();
    assert!(
        !cache_root.starts_with(repository_root),
        "real artifact cache must remain outside the repository"
    );
    let entire_cache_was_fresh = !cache_root.exists();

    for record in &evidence.artifacts {
        let artifact_path = recipe_root()
            .join("artifacts")
            .join(format!("{}.toml", record.artifact_id));
        let digest = &record.actual_sha256;
        let extraction_marker = cache_root
            .join("production-extracted/sha256-v2")
            .join(&digest[..2])
            .join(digest)
            .join(".chriz-bg-extraction.json");
        let extraction_was_fresh = !extraction_marker.exists();
        let output = Command::new(env!("CARGO_BIN_EXE_chriz-bg-author"))
            .args(["artifact", "verify"])
            .arg(&artifact_path)
            .arg("--cache-root")
            .arg(&cache_root)
            .output()
            .unwrap_or_else(|error| panic!("verify {}: {error}", record.artifact_id));
        assert!(
            output.status.success(),
            "verify {} failed: {}",
            record.artifact_id,
            String::from_utf8_lossy(&output.stderr)
        );
        let report: serde_json::Value =
            serde_json::from_slice(&output.stdout).expect("parse verification report");
        assert_eq!(report["verified"], true);
        assert_eq!(report["artifact_id"], record.artifact_id);
        assert_eq!(report["length"], record.actual_length);
        assert_eq!(report["sha256"], record.actual_sha256);
        let final_url = report["final_url"].as_str().expect("final URL");
        let final_host = url::Url::parse(final_url)
            .expect("valid final URL")
            .host_str()
            .expect("final URL host")
            .to_owned();
        assert_eq!(final_host, record.observed_final_host);
        assert_eq!(
            report["archive"]["wrapper_directory"].as_str(),
            record.wrapper_directory.as_deref()
        );
        assert_eq!(
            serde_json::from_value::<Vec<String>>(report["archive"]["publish_roots"].clone())
                .expect("publish roots"),
            record.materialized_publish_roots
        );
        assert_eq!(
            serde_json::from_value::<Vec<String>>(report["archive"]["tp2_paths"].clone())
                .expect("TP2 paths"),
            record.declared_tp2_paths
        );
        assert_eq!(
            report["pe_machine"].as_str(),
            record.pe.as_ref().map(|pe| pe.machine.as_str())
        );
        assert_eq!(
            report["tool"]["length"].as_u64(),
            record.pe.as_ref().map(|pe| pe.length)
        );
        assert_eq!(
            report["tool"]["sha256"].as_str(),
            record.pe.as_ref().map(|pe| pe.sha256.as_str())
        );
        assert_eq!(
            report["tool"]["weidu_version"].as_str(),
            record.pe.as_ref().map(|pe| pe.weidu_version.as_str())
        );

        let archive_path = cache_root
            .join("production/sha256")
            .join(&digest[..2])
            .join(format!("{digest}.archive"));
        assert_eq!(measured_archive_shape(&archive_path), record.observed_shape);
        let marker: serde_json::Value = serde_json::from_slice(
            &std::fs::read(&extraction_marker).expect("read v2 extraction marker"),
        )
        .expect("parse v2 extraction marker");
        assert_eq!(
            marker["version"],
            serde_json::Value::from(evidence.extraction_marker_version)
        );
        if entire_cache_was_fresh {
            assert_eq!(report["cache_disposition"], "downloaded");
            assert!(extraction_was_fresh);
            assert!(record.fresh_download);
            assert!(record.fresh_extraction);
        }
    }
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
