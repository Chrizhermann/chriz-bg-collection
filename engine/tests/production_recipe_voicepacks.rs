use std::path::PathBuf;

use bg_engine::manifest::{
    AcquisitionPolicy, ArchiveKind, ArchiveRootRule, Decision, Phase, SourceKind,
};
use bg_engine::recipe_view::evaluate_preset;
use bg_engine::Manifest;

/// ChrizFader's player voicepacks, release v1.1.0, in release-manifest order:
/// (pack id, TP2 folder, version, archive length, SHA-256).
const PACKS: &[(&str, &str, &str, u64, &str)] = &[
    (
        "neera",
        "cd_neera",
        "1.0.1",
        2_361_220,
        "f2adbfd775f86f9cfbcae37ca1b4aaa1ca9e57876b8952603a56b632aba50fbc",
    ),
    (
        "safana-bg1",
        "cd_safana",
        "1.0.1",
        2_947_151,
        "e5b3e7fa69461c333530d61979bf713dfddc693614e95f6cea854e45ae8a541d",
    ),
    (
        "safana-sod",
        "cd_safsod",
        "1.0.1",
        2_749_003,
        "edd486e85c9521905dd8cba2a5f3fddb846f3ae33b3f36e0308bab89c0432602",
    ),
    (
        "jaheira-bg1",
        "cd_jahebg1",
        "1.0.1",
        2_710_434,
        "d8a3bd683fc1c044eceec662414ff8279ea4992053b6c931486097436ad03806",
    ),
    (
        "jaheira-bg2",
        "cd_jahebg2",
        "1.0.1",
        2_119_330,
        "5efe01db15944ad675eaf115ba0ca6bbcf76656ffa6c0408096e4c82be5ccf07",
    ),
    (
        "yeslick",
        "cd_yeslick",
        "1.0.1",
        2_710_823,
        "c3c1213cc4ec89002b750788c58cd533d5c765d839c9c6b7c463c9db9ec3f22b",
    ),
    (
        "edwin-bg1",
        "cd_edwnbg1",
        "1.0.1",
        2_904_129,
        "6c02995b38edc5a021a54afbf544ae3e3ec3fa8f7cf5750aad22eb853a5e5411",
    ),
    (
        "edwin-bg2",
        "cd_edwnbg2",
        "1.0.1",
        2_454_719,
        "d32e6f5577b95ae73b2113cb2ec83de2952a8c83383b4863d7df5fdcbc6aebc9",
    ),
    (
        "aerie",
        "cd_aerie",
        "1.0.1",
        2_341_205,
        "04ce446df621ad6def59f5ca922c44a4d9a147ca86a236afdb22df1390ba6775",
    ),
    (
        "imoen-bg1",
        "cd_imobg1",
        "1.0.1",
        1_796_373,
        "cda87d6b56a15437025c536e2ea50194330565adf8a5e55f1cbdbbcaf8de79bb",
    ),
    (
        "imoen-bg2",
        "cd_imobg2",
        "1.0.1",
        2_046_524,
        "043d6216b4cf347a8a28f0d464af4286f01624587e55c14e324d0d21f628be60",
    ),
    (
        "sarevok",
        "cd_sarevok",
        "1.0.1",
        2_115_482,
        "e105f9b296d06677d9d0ba67ffdc3583c90ac13a97ea4dd7d45117f721af7937",
    ),
    (
        "amelyssan",
        "cd_amely",
        "1.0.1",
        2_447_749,
        "f67c4e4b87ccafc9272539d212c0211d53e45f8cd76d1a06aabd86239f4b6809",
    ),
    (
        "gromnir-il-khan",
        "cd_gromn",
        "1.0.1",
        1_972_660,
        "7aaad0379a6cbba87c5b2aa772ad3b53b777ae1d1821194567cfa8288811c9f5",
    ),
    (
        "ajantis",
        "cd_ajanti",
        "2.0.0",
        4_057_717,
        "7d2150ba920205e68258b9fc74a35594c1368d8b09e4e66ca9f1569a1baed77b",
    ),
    (
        "viconia-bg1",
        "cd_vicbg1",
        "1.0.0",
        4_120_894,
        "bd56af0ef69e84986061ee53fad36611291fb2038c2dbfd059542eb6d6a0d5f5",
    ),
    (
        "viconia-bg2",
        "cd_vicbg2",
        "1.0.0",
        5_058_181,
        "dde9cfca2692de0b75e1bd9a6bc8dabba85db79608b87da18f49ae685d31420a",
    ),
    (
        "garithos",
        "cd_garith",
        "1.1.0",
        5_117_808,
        "24abfd30cd8c9e5bedc77b2f8c9ab8b1444cc5c4189f5857d2b70def805525c5",
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
fn freezes_the_published_v110_voicepack_archives_without_their_bundled_weidu() {
    for (profile, manifest) in profiles() {
        for (pack, folder, version, length, sha256) in PACKS {
            let artifact_id = format!("voicepack-{pack}-{version}");
            let artifact = &manifest.artifacts[&artifact_id];
            let filename = format!("{pack}-player-voicepack-v{version}.zip");
            assert_eq!(artifact.version, *version, "{profile} {artifact_id}");
            assert_eq!(artifact.acquisition, AcquisitionPolicy::FetchOnly);
            assert_eq!(artifact.source.kind, SourceKind::GithubRelease);
            assert_eq!(artifact.source.reference, "v1.1.0");
            assert_eq!(
                artifact.source.url,
                format!(
                    "https://github.com/Chrizhermann/chriz-bg-voicepacks/releases/download/v1.1.0/{filename}"
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
