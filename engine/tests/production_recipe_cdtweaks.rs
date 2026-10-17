//! Tweaks Anthology choices in the curated recipe that need more than a component number.

use std::path::PathBuf;

use bg_engine::validate::validate;
use bg_engine::Manifest;

/// The reference setup's `cdtweaks/cdtweaks.txt` for 3347: party movement at 125 percent, no
/// faster casting, portrait icons on (`docs/curation/components/CDTWEAKS.md`).
const PARTY_SPEED_CONFIG: &str = concat!(
    "OUTER_SET party_movement_speed_enabled = 1\n",
    "OUTER_SET party_movement_speed_value = 125\n",
    "OUTER_SET party_casting_speed_enabled = 0\n",
    "OUTER_SET party_movement_speed_display_icons = 1\n",
);

fn curated() -> Manifest {
    Manifest::load(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../recipes/curated-full-current"),
    )
    .expect("curated recipe should load")
}

#[test]
fn custom_party_speed_ships_the_reference_configuration_file() {
    let manifest = curated();
    let declared = manifest
        .mods
        .values()
        .flat_map(|installer| {
            installer.components.iter().flat_map(move |component| {
                component.config_files.iter().map(move |file| {
                    (
                        installer.id.as_str(),
                        component.id,
                        file.path.as_str(),
                        file.content.as_str(),
                    )
                })
            })
        })
        .collect::<Vec<_>>();

    assert_eq!(
        declared,
        [(
            "cdtweaks",
            3347,
            "cdtweaks/cdtweaks.txt",
            PARTY_SPEED_CONFIG
        )]
    );
    assert!(manifest
        .collection
        .runs
        .iter()
        .any(|run| run.mod_id == "cdtweaks" && run.components.contains(&3347)));
    assert!(
        !validate(&manifest)
            .iter()
            .any(|finding| finding.rule == "component-config-files"),
        "{:#?}",
        validate(&manifest)
    );
}
