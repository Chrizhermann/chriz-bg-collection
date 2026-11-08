use std::path::PathBuf;

use bg_engine::manifest::{
    AcquisitionPolicy, ArchiveKind, ArchiveRootRule, Decision, Phase, SourceKind,
};
use bg_engine::recipe_view::evaluate_preset;
use bg_engine::Manifest;

/// ChrizFader's player voicepacks, release v1.2.0, in release-manifest order:
/// (pack id, TP2 folder, version, archive length, SHA-256).
const PACKS: &[(&str, &str, &str, u64, &str)] = &[
    (
        "neera",
        "cd_neera",
        "1.0.2",
        2_363_513,
        "b7d029f839062b5e7763de61f541a4922e2cb73d82b6e6c035d8fcdcff3eac94",
    ),
    (
        "safana-bg1",
        "cd_safana",
        "1.0.2",
        2_949_451,
        "daf45b3d58901509a05f570269e417ad137dcd920cc4cbfc14fbeafe6272f276",
    ),
    (
        "safana-sod",
        "cd_safsod",
        "1.0.2",
        2_751_308,
        "eacccb9187a1abaa28d8d8e6171e3437459f1280b04bec9c9c4199e7f33a2be2",
    ),
    (
        "jaheira-bg1",
        "cd_jahebg1",
        "1.0.2",
        2_712_742,
        "7167f278446bc9d63485aebf54cefe281d52d6659a496a2c070417fc734322e8",
    ),
    (
        "jaheira-bg2",
        "cd_jahebg2",
        "1.0.2",
        2_121_640,
        "1c629463c66b0cc377e35356e560e1d26868eab61124fa704c26d61843df8bcf",
    ),
    (
        "yeslick",
        "cd_yeslick",
        "1.0.2",
        2_713_135,
        "b4118fecf98786bd092056f9039ee6d8591d5419aa09cdb5b8807ba85bd2c7d6",
    ),
    (
        "edwin-bg1",
        "cd_edwnbg1",
        "1.0.2",
        2_906_433,
        "478b58972a4f247e00c046e02ab6c2c87f3fb520d3b10f037b730ba855061a5b",
    ),
    (
        "edwin-bg2",
        "cd_edwnbg2",
        "1.0.2",
        2_457_030,
        "131d4db31326c377ddfde2f73009d1bc18e8d62ff0f053d66389d6be684b8e3d",
    ),
    (
        "aerie",
        "cd_aerie",
        "1.0.2",
        2_343_511,
        "a6c0d61b87803df550dd97cdd9e56e443ee268a06e5751e9b6b4cb2cbba93f18",
    ),
    (
        "imoen-bg1",
        "cd_imobg1",
        "1.0.2",
        1_798_679,
        "7bac3a24dbd5ecdc70afb2b82210c2e6e39061fcb4f2b7803aeb9c911452d098",
    ),
    (
        "imoen-bg2",
        "cd_imobg2",
        "1.0.2",
        2_048_827,
        "98125ee8e3159ec7c968649e769fa1de0073c0d9dc8c41675c62247586ebe3d7",
    ),
    (
        "sarevok",
        "cd_sarevok",
        "1.0.2",
        2_117_790,
        "29589c43eb14191124155b246979270c137af2f1303a70e8df245173d1c31882",
    ),
    (
        "amelyssan",
        "cd_amely",
        "1.0.2",
        2_450_051,
        "d16ed2b2b773b5e6caf1b67597eab4c1717ea0014ddbf90e969cc1d808b2693a",
    ),
    (
        "gromnir-il-khan",
        "cd_gromn",
        "1.0.2",
        1_974_964,
        "ce709ab49aea5db3e6ac1419995301d66c64c9bf13e79f55be1968926a363e0b",
    ),
    (
        "ajantis",
        "cd_ajanti",
        "2.0.1",
        4_059_707,
        "1c9bb4056e8266b2868a223c22fe9675407763bf8c2796e96a4c1590dc6e73f3",
    ),
    (
        "viconia-bg1",
        "cd_vicbg1",
        "1.0.1",
        4_122_902,
        "a1f9b57cd841861cc87eec665b862c215089ae51548994b1fc193f45904708f6",
    ),
    (
        "viconia-bg2",
        "cd_vicbg2",
        "1.0.1",
        5_060_194,
        "68fbfca869d7d4a5ef795a7b2d8573b886448dd724da873a24d53ced1b74b56f",
    ),
    (
        "garithos",
        "cd_garith",
        "1.1.1",
        5_119_782,
        "84d153f3e7608243ab57b32a3f57d1f34301b602a0196566da48fa7ce646f2a8",
    ),
];

fn profiles() -> [(&'static str, Manifest); 2] {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    ["manifest", "recipes/curated-full-current"].map(|profile| {
        let manifest = Manifest::load(&root.join(profile))
            .unwrap_or_else(|error| panic!("{profile} recipe should load: {error}"));
        (profile, manifest)
    })
}

#[test]
fn freezes_the_published_v120_voicepack_archives_without_their_bundled_weidu() {
    for (profile, manifest) in profiles() {
        for (pack, folder, version, length, sha256) in PACKS {
            let artifact_id = format!("voicepack-{pack}-{version}");
            let artifact = &manifest.artifacts[&artifact_id];
            let filename = format!("{pack}-player-voicepack-v{version}.zip");
            assert_eq!(artifact.version, *version, "{profile} {artifact_id}");
            assert_eq!(artifact.acquisition, AcquisitionPolicy::FetchOnly);
            assert_eq!(artifact.source.kind, SourceKind::GithubRelease);
            assert_eq!(artifact.source.reference, "v1.2.0");
            assert_eq!(
                artifact.source.url,
                format!(
                    "https://github.com/Chrizhermann/chriz-bg-voicepacks/releases/download/v1.2.0/{filename}"
                )
            );
            assert_eq!(
                artifact.source.expected_filename.as_deref(),
                Some(filename.as_str())
            );
            assert_eq!(artifact.source.expected_length, Some(*length));
            assert_eq!(artifact.source.sha256, *sha256);
            assert_eq!(artifact.archive.kind, ArchiveKind::Zip);
            assert_eq!(artifact.archive.root_rule, ArchiveRootRule::Direct);
            // Only the mod folder is published: never setup-cd_*.exe or Garithos's Extras.
            assert_eq!(artifact.archive.publish_roots, [*folder]);
            assert_eq!(
                artifact.archive.tp2_paths,
                [format!("{folder}/{folder}.tp2")]
            );

            let installer = &manifest.mods[&format!("voicepack-{pack}")];
            assert_eq!(installer.artifact_id, artifact_id);
            assert_eq!(installer.tp2, format!("{folder}/{folder}.tp2"));
            assert_eq!(installer.language, 0);
            assert_eq!(installer.weidu_artifact_id, "weidu-249-amd64");
            assert_eq!(installer.components.len(), 1);
            assert_eq!(installer.components[0].id, 0);
            assert!(installer.components[0].prompts.is_empty());
        }
    }
}

#[test]
fn installs_every_voicepack_by_default_after_eet_end_and_before_buffbot() {
    for (profile, manifest) in profiles() {
        let runs = &manifest.collection.runs;
        let position = |id: &str| {
            runs.iter()
                .position(|run| run.run_id == id)
                .unwrap_or_else(|| panic!("{profile}: missing run {id}"))
        };
        let first = position("voicepack-neera-bg2");
        for (offset, (pack, ..)) in PACKS.iter().enumerate() {
            let run_id = format!("voicepack-{pack}-bg2");
            let run = &runs[first + offset];
            assert_eq!(run.run_id, run_id, "{profile}: release order");
            assert_eq!(run.mod_id, format!("voicepack-{pack}"));
            assert_eq!(run.phase, Phase::PostEetEnd);
            assert_eq!(run.components, [0]);
        }
        let last = first + PACKS.len() - 1;
        assert!(position("eet-end-bg2") < first);
        assert!(position("chriz-bg-modpack-baeloth-bg2") < first);
        assert!(last < position("buffbot-bg2"));
        // Scroll policy 430 stays the final run, right after BuffBot.
        let (tail, rest) = runs.split_last().unwrap();
        assert_eq!(tail.run_id, "chriz-bg-modpack-scrolls-bg2");
        assert_eq!(rest.last().unwrap().run_id, "buffbot-bg2");

        let evaluation = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
        for (pack, ..) in PACKS {
            let feature_id = format!("feature:voicepack-{pack}:component-0");
            let feature = manifest
                .collection
                .features
                .iter()
                .find(|feature| feature.id == feature_id)
                .unwrap_or_else(|| panic!("{profile}: missing {feature_id}"));
            assert_eq!(feature.decision, Decision::Default);
            assert!(feature.requires.is_empty());
            assert_eq!(
                evaluation
                    .plan
                    .components_for(&format!("voicepack-{pack}-bg2")),
                Some(&[0][..]),
                "{profile}: {feature_id}"
            );
        }
    }
}
