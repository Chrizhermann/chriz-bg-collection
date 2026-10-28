//! The product a build is: the Baldur's Gate app or the Icewind Dale app.
//!
//! One source tree builds both apps. `build.rs` reads the bundle identifier of the Tauri
//! configuration overlay and sets `chriz_flavor_iwd` for the Icewind Dale build, so the product
//! words, folders and feature switches below are fixed when the app is compiled. The bridge keeps
//! the flavor as data, which lets tests exercise either app in one build.

use bg_engine::games::GameRole;
use bg_engine::manifest::CollectionGame;
use bg_engine::registry::names::{DEFAULT_APPLICATION_DATA_DIRECTORY, DEFAULT_INSTALLATION_NAME};

/// Product identity and the features one app offers.
#[derive(Debug, PartialEq, Eq)]
pub struct AppFlavor {
    /// Stable key the interface compares with the flavor it was built for.
    pub key: &'static str,
    /// Full product name.
    pub product_name: &'static str,
    /// Product name used inside sentences.
    pub short_name: &'static str,
    /// Bundle identifier of the Tauri configuration; it names the app's own data folders.
    pub identifier: &'static str,
    /// Game every bundled collection installs.
    pub game: CollectionGame,
    /// Folder below `LOCALAPPDATA` that keeps the installation registry.
    pub application_data_directory: &'static str,
    /// Suggested name of a new installation and of its folder below `Games`.
    pub default_installation_name: &'static str,
    /// The one bundled collection of a standalone app. The Baldur's Gate app instead offers
    /// its curated setup and the smaller downloadable alpha.
    pub standalone: Option<StandaloneCollection>,
    /// BG Radar Overlay: its installation, updates and launch option.
    pub radar: bool,
    /// Signed game fixes for completed installations.
    pub install_patches: bool,
    /// Whole-mod updates of installed mods, such as BuffBot.
    pub mod_upgrades: bool,
    /// Signed application updates through the Tauri updater.
    pub app_updates: bool,
    /// Hotpatches for completed installations: the catalog folder below the packaged resources.
    pub hotpatches: Option<&'static str>,
}

/// The single collection a standalone app bundles and installs from one source game.
#[derive(Debug, PartialEq, Eq)]
pub struct StandaloneCollection {
    /// Recipe folder below the packaged resources.
    pub recipe: &'static str,
    /// Setup id reported to the interface.
    pub profile_id: &'static str,
    pub profile_label: &'static str,
    pub profile_description: &'static str,
    /// Role of the one source game.
    pub source_role: GameRole,
    /// The game's own executable, which a completed installation launches when it has no
    /// InfinityLoader because EEex was left out.
    pub game_executable: &'static str,
}

impl AppFlavor {
    /// Role of the source that fills the `bg2` slot: BG2EE, or a standalone app's one game.
    pub fn primary_role(&self) -> GameRole {
        self.standalone
            .as_ref()
            .map_or(GameRole::Bg2ee, |collection| collection.source_role)
    }
}

/// Chriz Easy BG: the Enhanced Edition Trilogy from a BG:EE + SoD source and a BG2:EE source.
pub const BG: AppFlavor = AppFlavor {
    key: "bg",
    product_name: "Chriz Easy BG",
    short_name: "CEBG",
    identifier: "dev.chrizhermann.bgcollection",
    game: CollectionGame::Eet,
    application_data_directory: DEFAULT_APPLICATION_DATA_DIRECTORY,
    default_installation_name: DEFAULT_INSTALLATION_NAME,
    standalone: None,
    radar: true,
    install_patches: true,
    mod_upgrades: true,
    app_updates: true,
    hotpatches: None,
};

/// Chriz Easy IWD: Icewind Dale: Enhanced Edition from one source game.
pub const IWD: AppFlavor = AppFlavor {
    key: "iwd",
    product_name: "Chriz Easy IWD",
    short_name: "Chriz Easy IWD",
    identifier: "dev.chrizhermann.iwdcollection",
    game: CollectionGame::Iwdee,
    application_data_directory: "Chriz IWD Collection",
    default_installation_name: "Chriz Easy IWD",
    standalone: Some(StandaloneCollection {
        recipe: "recipes/iwd-current",
        profile_id: "iwd-current",
        profile_label: "Recommended setup",
        profile_description:
            "The curated Chriz Easy IWD collection for Icewind Dale: Enhanced Edition.",
        source_role: GameRole::Iwdee,
        game_executable: "icewind.exe",
    }),
    // The overlay finds Icewind Dale: Enhanced Edition by its process name as well.
    radar: true,
    install_patches: false,
    mod_upgrades: false,
    app_updates: false,
    hotpatches: Some("hotpatches/iwd"),
};

/// The flavor this binary was built as.
pub const CURRENT: &AppFlavor = if cfg!(chriz_flavor_iwd) { &IWD } else { &BG };

/// Version of this app, from the configuration it was built with: Chriz Easy IWD numbers its
/// releases on its own, Chriz Easy BG uses the crate version.
pub const APP_VERSION: &str = env!("CHRIZ_APP_VERSION");

/// Every flavor, for checks that must cover each app.
pub const ALL: [&AppFlavor; 2] = [&BG, &IWD];

#[cfg(test)]
mod tests {
    use super::*;

    fn configuration(name: &str) -> serde_json::Value {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(name);
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap())
            .unwrap_or_else(|error| panic!("parse {}: {error}", path.display()))
    }

    /// `build.rs` picks the flavor by these identifiers; each configuration names its own.
    #[test]
    fn each_configuration_names_its_flavor() {
        let base = configuration("tauri.conf.json");
        assert_eq!(base["identifier"], BG.identifier);
        assert_eq!(base["productName"], BG.product_name);
        let iwd = configuration("tauri.iwd.conf.json");
        assert_eq!(iwd["identifier"], IWD.identifier);
        assert_eq!(iwd["productName"], IWD.product_name);
        assert_eq!(iwd["app"]["windows"][0]["title"], IWD.product_name);
        let keys = ALL.iter().map(|flavor| flavor.key).collect::<Vec<_>>();
        assert_eq!(keys, ["bg", "iwd"]);
    }

    /// The version the app reports is the one its installer carries.
    #[test]
    fn the_app_reports_the_version_of_its_configuration() {
        let name = if cfg!(chriz_flavor_iwd) {
            "tauri.iwd.conf.json"
        } else {
            "tauri.conf.json"
        };
        assert_eq!(configuration(name)["version"], APP_VERSION);
    }

    /// `build.rs` turns the identifier of the configuration patch into the compiled flavor.
    #[test]
    fn the_compiled_flavor_follows_the_tauri_configuration() {
        let identifier = option_env!("TAURI_CONFIG").and_then(|patch| {
            serde_json::from_str::<serde_json::Value>(patch).unwrap()["identifier"]
                .as_str()
                .map(str::to_owned)
        });
        let expected = match identifier.as_deref() {
            Some(identifier) if identifier == IWD.identifier => &IWD,
            _ => &BG,
        };
        assert_eq!(CURRENT, expected);
    }

    /// The Icewind Dale app bundles exactly the recipe it installs, and the flavor agrees with it.
    #[test]
    fn the_iwd_flavor_matches_its_bundled_recipe() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let collection = IWD.standalone.as_ref().unwrap();
        let manifest = bg_engine::Manifest::load(&root.join(collection.recipe)).unwrap();
        assert_eq!(manifest.collection.game, IWD.game);
        assert_eq!(
            manifest.collection.application_data_directory.as_deref(),
            Some(IWD.application_data_directory)
        );
        assert_eq!(
            manifest.collection.game.standalone_role(),
            Some(collection.source_role)
        );
        let profiles =
            bg_engine::games::GameProfiles::load(root.join(collection.recipe).join("game-builds"))
                .unwrap();
        assert!(!profiles.is_empty());
        assert!(profiles
            .iter()
            .all(|profile| profile.role == collection.source_role
                && profile
                    .executable
                    .eq_ignore_ascii_case(collection.game_executable)));
        assert!(manifest.presets.contains_key("chris-recommended"));

        let resources = configuration("tauri.iwd.conf.json")["bundle"]["resources"].clone();
        assert_eq!(
            resources[format!("../../{}/", collection.recipe)],
            format!("{}/", collection.recipe)
        );
        // The hotpatch catalog ships where the bridge reads it, and fits this game.
        let hotpatches = IWD.hotpatches.unwrap();
        assert_eq!(
            resources[format!("../../{hotpatches}/")],
            format!("{hotpatches}/")
        );
        let catalog = bg_engine::hotpatch::catalog::load(&root.join(hotpatches)).unwrap();
        assert_eq!(catalog.game, IWD.game);
        assert!(BG.hotpatches.is_none());
    }
}
