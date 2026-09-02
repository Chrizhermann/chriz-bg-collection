use serde::Deserialize;

const APPROVED: &str = include_str!("../../manifest/reference/bg1-premerge-approved.toml");

#[derive(Debug, Deserialize)]
struct Fragment {
    schema_version: u32,
    status: String,
    source_capture: String,
    runs: Vec<Run>,
}

#[derive(Debug, Deserialize)]
struct Run {
    id: String,
    artifact_id: String,
    target: String,
    phase: String,
    components: Vec<u32>,
    #[serde(default)]
    staged_bg1: bool,
}

#[test]
fn approved_fragment_has_the_required_premerge_boundary_and_one_fixpack_artifact() {
    let fragment: Fragment = toml::from_str(APPROVED).unwrap();

    assert_eq!(fragment.schema_version, 1);
    assert_eq!(fragment.status, "alpha-default");
    assert_eq!(
        fragment.source_capture,
        "manifest/reference/bg1-premerge.tsv"
    );

    let ids: Vec<_> = fragment.runs.iter().map(|run| run.id.as_str()).collect();
    assert_eq!(
        ids,
        [
            "dlcmerger-bg1",
            "eefixpack-bg1",
            "bg1ub-bg1",
            "bg1npc-bg1",
            "eefixpack-bg2",
            "eet-initialize-bg2",
        ]
    );

    assert_run(
        &fragment.runs[0],
        "dlcmerger-2.1",
        "bg1",
        "bg1-preparation",
        &[1],
    );
    assert_run(
        &fragment.runs[1],
        "eefixpack-beta2",
        "bg1",
        "bg1-preparation",
        &[0, 2],
    );
    assert_run(
        &fragment.runs[2],
        "bg1ub-17.1",
        "bg1",
        "bg1-only",
        &[
            0, 11, 12, 13, 14, 16, 17, 18, 19, 21, 22, 29, 30, 32, 33, 34,
        ],
    );
    assert_run(
        &fragment.runs[3],
        "bg1npc-32",
        "bg1",
        "bg1-only",
        &[0, 10, 90, 111, 120, 130, 240, 160, 200],
    );
    assert_run(
        &fragment.runs[4],
        "eefixpack-beta2",
        "bg2",
        "bg2-preparation",
        &[0, 2],
    );
    assert_run(
        &fragment.runs[5],
        "eet-official-74e91d72bca5d073fa11c1d088b90d7ff0c7105d",
        "bg2",
        "eet-initialization",
        &[0, 100],
    );
    assert!(fragment.runs[5].staged_bg1);

    let fixpack_artifacts: Vec<_> = fragment
        .runs
        .iter()
        .filter(|run| run.id.starts_with("eefixpack-"))
        .map(|run| run.artifact_id.as_str())
        .collect();
    assert_eq!(fixpack_artifacts, ["eefixpack-beta2", "eefixpack-beta2"]);
}

fn assert_run(run: &Run, artifact: &str, target: &str, phase: &str, components: &[u32]) {
    assert_eq!(run.artifact_id, artifact);
    assert_eq!(run.target, target);
    assert_eq!(run.phase, phase);
    assert_eq!(run.components, components);
}

#[test]
fn capture_contains_the_exact_verified_twenty_eight_active_entries() {
    let capture = include_str!("../../manifest/reference/bg1-premerge.tsv");
    let rows: Vec<_> = capture.lines().skip(1).collect();

    assert_eq!(rows.len(), 28);
    let actual: Vec<_> = rows
        .iter()
        .map(|row| {
            let fields: Vec<_> = row.split('\t').collect();
            assert_eq!(fields.len(), 7, "malformed capture row: {row}");
            assert_eq!(
                fields[1],
                "BG1 premerge release candidate, verified 2026-09-03"
            );
            (
                fields[3],
                fields[4].parse::<u32>().unwrap(),
                fields[5].parse::<u32>().unwrap(),
            )
        })
        .collect();
    let expected = [
        ("DLCMERGER\\DLCMERGER.TP2", 0, 1),
        ("EEFIXPACK\\SETUP-EEFIXPACK.TP2", 0, 0),
        ("EEFIXPACK\\SETUP-EEFIXPACK.TP2", 0, 2),
        ("BG1UB/BG1UB.TP2", 0, 0),
        ("BG1UB/BG1UB.TP2", 0, 11),
        ("BG1UB/BG1UB.TP2", 0, 12),
        ("BG1UB/BG1UB.TP2", 0, 13),
        ("BG1UB/BG1UB.TP2", 0, 14),
        ("BG1UB/BG1UB.TP2", 0, 16),
        ("BG1UB/BG1UB.TP2", 0, 17),
        ("BG1UB/BG1UB.TP2", 0, 18),
        ("BG1UB/BG1UB.TP2", 0, 19),
        ("BG1UB/BG1UB.TP2", 0, 21),
        ("BG1UB/BG1UB.TP2", 0, 22),
        ("BG1UB/BG1UB.TP2", 0, 29),
        ("BG1UB/BG1UB.TP2", 0, 30),
        ("BG1UB/BG1UB.TP2", 0, 32),
        ("BG1UB/BG1UB.TP2", 0, 33),
        ("BG1UB/BG1UB.TP2", 0, 34),
        ("BG1NPC/BG1NPC.TP2", 0, 0),
        ("BG1NPC/BG1NPC.TP2", 0, 10),
        ("BG1NPC/BG1NPC.TP2", 0, 90),
        ("BG1NPC/BG1NPC.TP2", 0, 111),
        ("BG1NPC/BG1NPC.TP2", 0, 120),
        ("BG1NPC/BG1NPC.TP2", 0, 130),
        ("BG1NPC/BG1NPC.TP2", 0, 240),
        ("BG1NPC/BG1NPC.TP2", 0, 160),
        ("BG1NPC/BG1NPC.TP2", 0, 200),
    ];
    assert_eq!(actual, expected);
}
