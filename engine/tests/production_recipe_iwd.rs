//! Static contracts of the Icewind Dale recipe `recipes/iwd-current`, with no game installation.
//!
//! The release records under `recipes/iwd-current/releases/` cite this file as the accepted
//! static-test evidence for every run the `chris-recommended` preset resolves.
use std::collections::BTreeSet;
use std::path::PathBuf;

use bg_engine::manifest::{
    AcquisitionPolicy, ArchiveKind, CollectionGame, Decision, InputValue, InvocationMode, Phase,
    PromptAnswer, Readiness, SourceKind,
};
use bg_engine::recipe_view::{evaluate, evaluate_preset, SelectionEvaluation};
use bg_engine::resolve::Selection;
use bg_engine::validate::validate;
use bg_engine::Manifest;

const PRESET: &str = "chris-recommended";
const WEIDU: &str = "weidu-249-amd64";
const WEASEL_DOWNLOADS: &str = "https://downloads.weaselmods.net/download/";

fn recipe_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../recipes/iwd-current")
}

fn recipe() -> Manifest {
    Manifest::load(&recipe_root()).expect("IWD recipe should load")
}

fn recommended(manifest: &Manifest) -> SelectionEvaluation {
    evaluate_preset(manifest, PRESET, "windows").expect("recommended preset should evaluate")
}

fn position(manifest: &Manifest, run_id: &str) -> usize {
    manifest
        .collection
        .runs
        .iter()
        .position(|run| run.run_id == run_id)
        .unwrap_or_else(|| panic!("missing run {run_id}"))
}

/// Every authored run in install order: run id, installer id, phase, and component list.
/// Each run's position is explained by its comment in `collection.toml`.
const RUNS: &[(&str, &str, Phase, &[u32])] = &[
    ("eefixpack-main", "eefixpack", Phase::Main, &[0, 2]),
    (
        "eeex-main",
        "eeex",
        Phase::Main,
        &[0, 1, 9, 2, 3, 4, 5, 6, 7, 8],
    ),
    ("leui-iwdee-main", "leui-iwdee", Phase::Main, &[0]),
    (
        "infinity-ui-main",
        "infinity-ui",
        Phase::Main,
        &[0, 1, 2, 3, 4, 5, 6, 7, 8],
    ),
    ("bubb-spell-menu-main", "bubb-spell-menu", Phase::Main, &[0]),
    ("iwdgo-main", "iwdgo", Phase::Main, &[0]),
    ("ub-iwd-main", "ub-iwd", Phase::Main, &[300, 700, 800]),
    ("willowisp-main", "willowisp", Phase::Main, &[2, 3, 4, 5]),
    ("minervanpc-main", "minervanpc", Phase::Main, &[0]),
    ("karihinpc-main", "karihinpc", Phase::Main, &[0]),
    ("athanasiosnpc-main", "athanasiosnpc", Phase::Main, &[0, 1]),
    ("guldruknpc-main", "guldruknpc", Phase::Main, &[0]),
    ("niriknpc-main", "niriknpc", Phase::Main, &[0]),
    ("cassianpc-main", "cassianpc", Phase::Main, &[0]),
    ("bethany-main", "bethany", Phase::Main, &[0]),
    ("dendjelion-main", "dendjelion", Phase::Main, &[0]),
    ("dusky-main", "dusky", Phase::Main, &[0]),
    ("hommet-main", "hommet", Phase::Main, &[0]),
    ("ina-main", "ina", Phase::Main, &[0]),
    ("lanna-main", "lanna", Phase::Main, &[0]),
    ("oakmaw-main", "oakmaw", Phase::Main, &[0]),
    ("orra-main", "orra", Phase::Main, &[0]),
    ("tviy-main", "tviy", Phase::Main, &[0]),
    ("urchin-main", "urchin", Phase::Main, &[0]),
    ("vito-main", "vito", Phase::Main, &[0]),
    ("innbb-main", "innbb", Phase::Main, &[0]),
    ("blinkingdead-main", "blinkingdead", Phase::Main, &[0]),
    ("snowytoes-main", "snowytoes", Phase::Main, &[0]),
    ("ourlady-main", "ourlady", Phase::Main, &[0]),
    ("skineater-main", "skineater", Phase::Main, &[0]),
    ("kuldahar-main", "kuldahar", Phase::Main, &[0]),
    (
        "thewarofbloomandrot-main",
        "thewarofbloomandrot",
        Phase::Main,
        &[0],
    ),
    (
        "iwdcrossmodpack-main",
        "iwdcrossmodpack",
        Phase::Main,
        &[10, 11, 12, 13, 14, 15, 30],
    ),
    (
        "cdtweaks-main",
        "cdtweaks",
        Phase::Main,
        &[
            10, 50, 60, 72, 90, 110, 130, 160, 170, 171, 180, 181, 182, 190, 191, 192, 193, 194,
            195, 200, 3150, 3151, 2010, 220, 230, 240, 241, 1010, 1100, 1120, 1140, 1141, 1142,
            1280, 1290, 1300, 1360, 1361, 2020, 2030, 2035, 2090, 2151, 2152, 2170, 2190, 2191,
            2192, 2210, 2220, 2250, 2260, 2270, 2280, 2292, 2380, 2410, 2420, 2430, 2500, 2520,
            2590, 2760, 2999, 3000, 3001, 3002, 3008, 3010, 3011, 3012, 3030, 3031, 3040, 3070,
            3071, 3072, 3073, 3080, 3081, 3082, 3083, 3090, 3091, 3092, 3093, 3100, 3101, 3102,
            3103, 3110, 3111, 3112, 3113, 3120, 3121, 3122, 3125, 3130, 3131, 3132, 3191, 3194,
            3195, 3196, 3197, 3198, 3200, 3205, 3240, 3241, 3242, 3290, 3291, 3292, 3293, 3294,
            3295, 3310, 3320, 3340, 3341, 3342, 3343, 3344, 3345, 3346, 3347, 3350, 3351, 3352,
            3353, 3354, 3355, 3356, 3357, 3390, 3500,
        ],
    ),
    (
        "stratagems-main",
        "stratagems",
        Phase::Main,
        &[
            1600, 1610, 2000, 2500, 2510, 2520, 3510, 3540, 3541, 4030, 4115, 4140, 4250, 5000,
            5900, 6020, 6030, 6040, 6100, 6510, 6540, 6550, 6700,
        ],
    ),
    (
        "hidden-gameplay-options-main",
        "hidden-gameplay-options",
        Phase::Main,
        &[
            10, 11, 12, 13, 14, 16, 18, 19, 20, 22, 23, 24, 25, 27, 28, 29, 30, 32, 33, 34, 35, 36,
            37, 38, 39, 200, 300, 301, 303, 304, 101, 102, 103,
        ],
    ),
    (
        "cdtweaks-spell-save-penalties",
        "cdtweaks",
        Phase::Main,
        &[2310, 2311, 2312],
    ),
    ("buffbot-main", "buffbot", Phase::Main, &[1, 0]),
    ("eeexremote-main", "eeexremote", Phase::Main, &[0]),
];

/// Every installer: id, payload artifact, and TP2 path inside that artifact.
const INSTALLERS: &[(&str, &str, &str)] = &[
    (
        "athanasiosnpc",
        "athanasiosnpc-035784ef834a5ae8d89addd1cab12d98e0089af6",
        "AthanasiosNPC/AthanasiosNPC.tp2",
    ),
    ("bethany", "bethany-1.03", "Bethany/Bethany.tp2"),
    (
        "blinkingdead",
        "blinkingdead-3.1",
        "BlinkingDead/BlinkingDead.tp2",
    ),
    (
        "bubb-spell-menu",
        "bubb-spell-menu-5.2",
        "bubb_spell_menu_extended/bubb_spell_menu_extended.tp2",
    ),
    ("buffbot", "buffbot-1.12.1", "buffbot/setup-buffbot.tp2"),
    (
        "cassianpc",
        "cassianpc-8f9af4ca4e76ec72240f961b18a1d83739acd994",
        "CassiaNPC/Setup-CassiaNPC.tp2",
    ),
    (
        "cdtweaks",
        "cdtweaks-7649ced6cd25865874d787ec1a9abbc67b068729",
        "cdtweaks/setup-cdtweaks.tp2",
    ),
    ("dendjelion", "dendjelion-2.1", "dendjelion/Dendjelion.tp2"),
    ("dusky", "dusky-2.5", "Dusky/Dusky.tp2"),
    ("eeex", "eeex-1.3.1", "EEex/EEex.tp2"),
    ("eeexremote", "eeexremote-0.2.1", "setup-eeexremote.tp2"),
    (
        "eefixpack",
        "eefixpack-beta2",
        "eefixpack/setup-eefixpack.tp2",
    ),
    (
        "guldruknpc",
        "guldruknpc-1522ddbe7bee18e32acc7152aa3aeb8bdbf98924",
        "GuldrukNPC/GuldrukNPC.tp2",
    ),
    (
        "hidden-gameplay-options",
        "hidden-gameplay-options-5.2",
        "HiddenGameplayOptions/HiddenGameplayOptions.tp2",
    ),
    ("hommet", "hommet-1.2", "Hommet/hommet.tp2"),
    ("ina", "ina-2.5", "Ina/Ina.tp2"),
    (
        "infinity-ui",
        "infinity-ui-1.31",
        "infinity_ui/infinity_ui.tp2",
    ),
    ("innbb", "innbb-3.1", "InnBB/InnBB.tp2"),
    (
        "iwdcrossmodpack",
        "iwdcrossmodpack-5",
        "iwdcrossmodpack/iwdcrossmodpack.tp2",
    ),
    (
        "iwdgo",
        "iwdgo-fda64dea40a33336d1e60d54584a4a6c6a4667d6",
        "iwdgo/iwdgo.tp2",
    ),
    ("karihinpc", "karihinpc-1.02", "KarihiNPC/KarihiNPC.tp2"),
    ("kuldahar", "kuldahar-2.6.1", "Kuldahar/kuldahar.tp2"),
    ("lanna", "lanna-1.8", "Lanna/lanna.tp2"),
    ("leui-iwdee", "leui-iwdee-2.0", "LeUI-IWDEE/LeUI-IWDEE.tp2"),
    ("minervanpc", "minervanpc-1.1", "MinervaNPC/MinervaNPC.tp2"),
    (
        "niriknpc",
        "niriknpc-2a1e9a969edd058937fca7dfb5a9f62caca5b1e4",
        "NirikNPC/NirikNPC.tp2",
    ),
    ("oakmaw", "oakmaw-3.3", "OakMaw/OakMaw.tp2"),
    ("orra", "orra-2.7", "Orra/Orra.tp2"),
    ("ourlady", "ourlady-2.7", "OurLady/OurLady.tp2"),
    ("skineater", "skineater-2.6", "Skineater/Skineater.tp2"),
    ("snowytoes", "snowytoes-1.6", "Snowytoes/Snowytoes.tp2"),
    (
        "stratagems",
        "stratagems-35.21",
        "stratagems/setup-stratagems.tp2",
    ),
    (
        "thewarofbloomandrot",
        "thewarofbloomandrot-1.3",
        "TheWarOfBloomAndRot/TheWarOfBloomAndRot.tp2",
    ),
    ("tviy", "tviy-2.4", "Tviy/Tviy.tp2"),
    ("ub-iwd", "ub-iwd-13", "ub_iwd/setup-ub_iwd.tp2"),
    ("urchin", "urchin-2.5", "Urchin/Urchin.tp2"),
    ("vito", "vito-1.1", "Vito/Vito.tp2"),
    ("willowisp", "willowisp-3.6", "willowisp/willowisp.tp2"),
];

/// Every artifact file of the recipe with its exact upstream identity and integrity pin.
const ARTIFACTS: &[Pin] = &[
    Pin {
        id: "athanasiosnpc-035784ef834a5ae8d89addd1cab12d98e0089af6",
        version: "035784ef834a5ae8d89addd1cab12d98e0089af6",
        kind: SourceKind::GithubCommitZip,
        url: "https://codeload.github.com/TheArtisanBG/IWD-Athanasios-NPC/zip/035784ef834a5ae8d89addd1cab12d98e0089af6",
        reference: "035784ef834a5ae8d89addd1cab12d98e0089af6",
        filename: "IWD-Athanasios-NPC-035784ef834a5ae8d89addd1cab12d98e0089af6.zip",
        length: 3_009_450,
        sha256: "80f020b11806a865f8aac00dc64ed58e1110a2cb064d27e8f5b97f06fcc8ff9b",
    },
    Pin {
        id: "bethany-1.03",
        version: "1.03",
        kind: SourceKind::WeaselmodsDownload,
        url: "https://downloads.weaselmods.net/download/bethany/?wpdmdl=751",
        reference: "1.03",
        filename: "Bethany-v1.03.zip",
        length: 3_759_411,
        sha256: "b18ca9501ac37c5d5db6c4e25850a3c877d17cd232545268d0bf9c49684d5f32",
    },
    Pin {
        id: "blinkingdead-3.1",
        version: "3.1",
        kind: SourceKind::WeaselmodsDownload,
        url: "https://downloads.weaselmods.net/download/night-of-the-blinking-dead/?wpdmdl=376",
        reference: "3.1",
        filename: "BlinkingDead-v3.1.zip",
        length: 10_707_885,
        sha256: "2e765e78b83051df0e8734e234caf0eb30661b1b924b5a65be2915c3a1e6c3b7",
    },
    Pin {
        id: "bubb-spell-menu-5.2",
        version: "5.2",
        kind: SourceKind::GithubTagArchive,
        url: "https://codeload.github.com/Bubb13/Bubbs-Spell-Menu-Extended/zip/refs/tags/v5.2",
        reference: "v5.2",
        filename: "Bubbs-Spell-Menu-Extended-5.2.zip",
        length: 1_432_756,
        sha256: "ac5380e99a534dd96200e97fa4358ca22ff2738266570d95195eac85d8f82e12",
    },
    Pin {
        id: "buffbot-1.12.1",
        version: "1.12.1",
        kind: SourceKind::GithubRelease,
        url: "https://github.com/Chrizhermann/bg-eeex-buffbot/releases/download/v1.12.1/buffbot-v1.12.1.zip",
        reference: "v1.12.1",
        filename: "buffbot-v1.12.1.zip",
        length: 5_698_539,
        sha256: "7ab5f040b05d474edfb846b11bff5fda134ee468e3e3a289e6f788f33e34ed22",
    },
    Pin {
        id: "cassianpc-8f9af4ca4e76ec72240f961b18a1d83739acd994",
        version: "8f9af4ca4e76ec72240f961b18a1d83739acd994",
        kind: SourceKind::GithubCommitZip,
        url: "https://codeload.github.com/Arvendor/CassiaNPC/zip/8f9af4ca4e76ec72240f961b18a1d83739acd994",
        reference: "8f9af4ca4e76ec72240f961b18a1d83739acd994",
        filename: "CassiaNPC-8f9af4ca4e76ec72240f961b18a1d83739acd994.zip",
        length: 3_896_221,
        sha256: "3d72034531f1cf7d98f8ed1c6510e624ebeb7aa15faba11a99a6bf98452b452e",
    },
    Pin {
        id: "cdtweaks-7649ced6cd25865874d787ec1a9abbc67b068729",
        version: "7649ced6cd25865874d787ec1a9abbc67b068729",
        kind: SourceKind::GithubCommitZip,
        url: "https://codeload.github.com/Gibberlings3/Tweaks-Anthology/zip/7649ced6cd25865874d787ec1a9abbc67b068729",
        reference: "7649ced6cd25865874d787ec1a9abbc67b068729",
        filename: "Tweaks-Anthology-7649ced6cd25865874d787ec1a9abbc67b068729.zip",
        length: 23_871_463,
        sha256: "117cd79cb5187640a5efd6c1d19181b43bcfa7381161df2f88db055d3af075f7",
    },
    Pin {
        id: "dendjelion-2.1",
        version: "2.1",
        kind: SourceKind::WeaselmodsDownload,
        url: "https://downloads.weaselmods.net/download/dendjelion/?wpdmdl=286",
        reference: "2.1",
        filename: "dendjelion-v2.1.zip",
        length: 12_598_744,
        sha256: "43fbf9c21474aa673a73dbc44c2852afb01217ea3573c9c0c8905340ac011493",
    },
    Pin {
        id: "dusky-2.5",
        version: "2.5",
        kind: SourceKind::WeaselmodsDownload,
        url: "https://downloads.weaselmods.net/download/dusky-iwdee-npc/?wpdmdl=129",
        reference: "2.5",
        filename: "Dusky-v2.5.zip",
        length: 6_209_880,
        sha256: "57fe2c552579c0602e7090068e34c93324e983ab3829bdb6ac12641abf432cb4",
    },
    Pin {
        id: "eeex-1.3.1",
        version: "1.3.1",
        kind: SourceKind::GithubRelease,
        url: "https://github.com/Bubb13/EEex/releases/download/v1.3.1/eeex-v1.3.1.zip",
        reference: "v1.3.1",
        filename: "eeex-v1.3.1.zip",
        length: 10_390_181,
        sha256: "9c8104bd63fe7e7dfff889d89ab2d98e158b6e552846370a924ed7eacea4ecf3",
    },
    Pin {
        id: "eeexremote-0.2.1",
        version: "0.2.1",
        kind: SourceKind::GithubCommitZip,
        url: "https://codeload.github.com/Chrizhermann/eeex-remote-console/zip/3d98990766abe9ed1024c79ff43d609101f37836",
        reference: "3d98990766abe9ed1024c79ff43d609101f37836",
        filename: "eeex-remote-console-3d98990766abe9ed1024c79ff43d609101f37836.zip",
        length: 34_419,
        sha256: "f417cb993fff8fac5f72a7e2c597baa78dd68f7bc51b5951777b5ffd60e70359",
    },
    Pin {
        id: "eefixpack-beta2",
        version: "Beta 2",
        kind: SourceKind::GithubRelease,
        url: "https://github.com/Gibberlings3/EE_Fixpack/releases/download/Beta_2/enhanced-edition-fixpack-beta-2.iemod",
        reference: "Beta_2",
        filename: "enhanced-edition-fixpack-beta-2.iemod",
        length: 130_716_647,
        sha256: "7a57e43bd6c30c7e36b7c2b7c5531c7a0834de164da186808da03d24474592bf",
    },
    Pin {
        id: "guldruknpc-1522ddbe7bee18e32acc7152aa3aeb8bdbf98924",
        version: "1522ddbe7bee18e32acc7152aa3aeb8bdbf98924",
        kind: SourceKind::GithubCommitZip,
        url: "https://codeload.github.com/TheArtisanBG/IWD-Guldruk-NPC/zip/1522ddbe7bee18e32acc7152aa3aeb8bdbf98924",
        reference: "1522ddbe7bee18e32acc7152aa3aeb8bdbf98924",
        filename: "IWD-Guldruk-NPC-1522ddbe7bee18e32acc7152aa3aeb8bdbf98924.zip",
        length: 3_103_020,
        sha256: "7852755da148a5e46d29d314ad9e8aa0c7a36ada7a7fe975f1cdc1bac3639bbf",
    },
    Pin {
        id: "hidden-gameplay-options-5.2",
        version: "5.2",
        kind: SourceKind::GithubRelease,
        url: "https://github.com/Argent77/A7-HiddenGameplayOptions/releases/download/v5.2/win-A7-HiddenGameplayOptions-v5.2.zip",
        reference: "v5.2",
        filename: "win-A7-HiddenGameplayOptions-v5.2.zip",
        length: 2_767_061,
        sha256: "a7b173f811b1d2137e9836e8dec7118e0a1927ff85e26956d211e0d1b53c2029",
    },
    Pin {
        id: "hommet-1.2",
        version: "1.2",
        kind: SourceKind::WeaselmodsDownload,
        url: "https://downloads.weaselmods.net/download/hommet/?wpdmdl=441",
        reference: "1.2",
        filename: "Hommet-v1.2.zip",
        length: 4_373_752,
        sha256: "71ce5655db53d428f90ec62445f757403041647c2ffefdfc67ee21f45c4f47df",
    },
    Pin {
        id: "ina-2.5",
        version: "2.5",
        kind: SourceKind::WeaselmodsDownload,
        url: "https://downloads.weaselmods.net/download/ina/?wpdmdl=288",
        reference: "2.5",
        filename: "Ina-v2.5.zip",
        length: 5_978_882,
        sha256: "000d1c5d05330d254f5d6535ececb26543a375265257cbec61c4bdc913678723",
    },
    Pin {
        id: "infinity-ui-1.31",
        version: "1.31",
        kind: SourceKind::GithubTagArchive,
        url: "https://codeload.github.com/Renegade0/InfinityUI/zip/refs/tags/v1.31",
        reference: "v1.31",
        filename: "InfinityUI-1.31.zip",
        length: 272_024_105,
        sha256: "4541f7f99cbf375ca30d0ff1f37df8df11e07a5e34bb09e158efbbee476b6685",
    },
    Pin {
        id: "innbb-3.1",
        version: "3.1",
        kind: SourceKind::WeaselmodsDownload,
        url: "https://downloads.weaselmods.net/download/below-below-inn/?wpdmdl=369",
        reference: "3.1",
        filename: "InnBB-v3.1.zip",
        length: 13_649_964,
        sha256: "159a82621b39c9eb494fbed3eab5c52570e5b30e8a119eac6b37464da12e8f62",
    },
    Pin {
        id: "iwdcrossmodpack-5",
        version: "5",
        kind: SourceKind::GithubRelease,
        url: "https://github.com/Gibberlings3/iwdcrossmod/releases/download/v5/icewind-dale-crossmod-banter-pack-v5.iemod",
        reference: "v5",
        filename: "icewind-dale-crossmod-banter-pack-v5.iemod",
        length: 667_051,
        sha256: "658f584697784511975099156e3f20dcb557398a7e588ff450c25bed27adc19b",
    },
    Pin {
        id: "iwdgo-fda64dea40a33336d1e60d54584a4a6c6a4667d6",
        version: "fda64dea40a33336d1e60d54584a4a6c6a4667d6",
        kind: SourceKind::GithubCommitZip,
        url: "https://codeload.github.com/Spellhold-Studios/Icewind-Dale-Graphical-Overhaul/zip/fda64dea40a33336d1e60d54584a4a6c6a4667d6",
        reference: "fda64dea40a33336d1e60d54584a4a6c6a4667d6",
        filename: "Icewind-Dale-Graphical-Overhaul-fda64dea40a33336d1e60d54584a4a6c6a4667d6.zip",
        length: 346_739_468,
        sha256: "276a143864118e9f9f6e52e2cce5dba0c7ab2ab5e5c674b2b24bda5ee396b995",
    },
    Pin {
        id: "karihinpc-1.02",
        version: "1.02",
        kind: SourceKind::GithubTagArchive,
        url: "https://codeload.github.com/TheArtisanBG/KarihiNPC/zip/refs/tags/v1.02",
        reference: "v1.02",
        filename: "KarihiNPC-1.02.zip",
        length: 6_785_704,
        sha256: "1491d118bb4100d4c707b1eed0072407e580b63af4f3e426721b8d61ad342722",
    },
    Pin {
        id: "kuldahar-2.6.1",
        version: "2.6.1",
        kind: SourceKind::WeaselmodsDownload,
        url: "https://downloads.weaselmods.net/download/the-rediscovery-of-kuldahar/?wpdmdl=420",
        reference: "2.6.1",
        filename: "Kuldahar-v2.6.1.zip",
        length: 39_452_216,
        sha256: "dd854e7706d16ea487f2a19fed4e5430193ff29062112ec27daecb1254ed5565",
    },
    Pin {
        id: "lanna-1.8",
        version: "1.8",
        kind: SourceKind::WeaselmodsDownload,
        url: "https://downloads.weaselmods.net/download/lanna/?wpdmdl=437",
        reference: "1.8",
        filename: "Lanna v1.8.zip",
        length: 14_379_906,
        sha256: "88eef0c49067a98652efd538650f23068c2ba7f7798bbd1f1f304276d950fdf9",
    },
    Pin {
        id: "leui-iwdee-2.0",
        version: "2.0",
        kind: SourceKind::GithubRelease,
        url: "https://github.com/r-e-d/LeUI-IWDEE/releases/download/v2.0/lefreuts-enhanced-ui-for-iwdee-2.0.iemod",
        reference: "v2.0",
        filename: "lefreuts-enhanced-ui-for-iwdee-2.0.iemod",
        length: 1_506_856,
        sha256: "a0fbdd4dc244777c4ba0f63678047c2c3711ba6cc80b1d8492b3a0069b56c79f",
    },
    Pin {
        id: "minervanpc-1.1",
        version: "1.1",
        kind: SourceKind::GithubTagArchive,
        url: "https://codeload.github.com/TheArtisanBG/MinervaNPC/zip/refs/tags/v1.1",
        reference: "v1.1",
        filename: "MinervaNPC-1.1.zip",
        length: 18_490_079,
        sha256: "c97a938fb981d23d67416f9026b0b418fc17a23530c9f6a8b8ee51dbf44adc6a",
    },
    Pin {
        id: "niriknpc-2a1e9a969edd058937fca7dfb5a9f62caca5b1e4",
        version: "2a1e9a969edd058937fca7dfb5a9f62caca5b1e4",
        kind: SourceKind::GithubCommitZip,
        url: "https://codeload.github.com/TheArtisanBG/IWD-Nirik-NPC/zip/2a1e9a969edd058937fca7dfb5a9f62caca5b1e4",
        reference: "2a1e9a969edd058937fca7dfb5a9f62caca5b1e4",
        filename: "IWD-Nirik-NPC-2a1e9a969edd058937fca7dfb5a9f62caca5b1e4.zip",
        length: 2_759_471,
        sha256: "28e92bce07cbe00222958c7d3bbff5d03aef9c0009954fb9ccefda91fd9d44d9",
    },
    Pin {
        id: "oakmaw-3.3",
        version: "3.3",
        kind: SourceKind::WeaselmodsDownload,
        url: "https://downloads.weaselmods.net/download/oak-maw/?wpdmdl=290",
        reference: "3.3",
        filename: "OakMaw v3.3.zip",
        length: 4_852_691,
        sha256: "8e4f0dbbc33a4d48cc076afef2165d04589888b98cbfcfe6955cc5f6873440b8",
    },
    Pin {
        id: "orra-2.7",
        version: "2.7",
        kind: SourceKind::WeaselmodsDownload,
        url: "https://downloads.weaselmods.net/download/orra/?wpdmdl=389",
        reference: "2.7",
        filename: "Orra-v2.7.zip",
        length: 3_459_689,
        sha256: "ae6e593c99fd720f31d40d6e76213ace6a636d57c7ddae3040b17792908bbef9",
    },
    Pin {
        id: "ourlady-2.7",
        version: "2.7",
        kind: SourceKind::WeaselmodsDownload,
        url: "https://downloads.weaselmods.net/download/tale-of-our-lady-dreamless/?wpdmdl=365",
        reference: "2.7",
        filename: "Tale-of-Our-Lady-Dreamless-v2.7.zip",
        length: 10_717_208,
        sha256: "4f94e07df46d0043e8b746b7091bae6afe5119f04f3f9edd70c311b18acd267a",
    },
    Pin {
        id: "skineater-2.6",
        version: "2.6",
        kind: SourceKind::WeaselmodsDownload,
        url: "https://downloads.weaselmods.net/download/terror-of-the-skineater/?wpdmdl=372",
        reference: "2.6",
        filename: "Skineater-v2.6.zip",
        length: 33_852_186,
        sha256: "077e892cecbd4e99f88c1a5153639620f547839a9614ff8bde47cc3fdcae8983",
    },
    Pin {
        id: "snowytoes-1.6",
        version: "1.6",
        kind: SourceKind::WeaselmodsDownload,
        url: "https://downloads.weaselmods.net/download/snowytoes-hamlet/?wpdmdl=557",
        reference: "1.6",
        filename: "Snowytoes-v1.6.zip",
        length: 17_766_463,
        sha256: "16308c765b77ddc082a70b5c09c44b77afd9d620db0853ebb77f1f9d657db9f6",
    },
    Pin {
        id: "stratagems-35.21",
        version: "35.21",
        kind: SourceKind::GithubRelease,
        url: "https://github.com/Gibberlings3/SwordCoastStratagems/releases/download/v35.21/lin-stratagems-35.21.zip",
        reference: "v35.21",
        filename: "lin-stratagems-35.21.zip",
        length: 58_517_195,
        sha256: "dd6b0dbe3b9f65cf70e5f57601ff667c791e968a62136254292e7521a6be99cb",
    },
    Pin {
        id: "thewarofbloomandrot-1.3",
        version: "1.3",
        kind: SourceKind::WeaselmodsDownload,
        url: "https://downloads.weaselmods.net/download/the-war-of-bloom-and-rot/?wpdmdl=866",
        reference: "1.3",
        filename: "TheWarOfBloomAndRot-v1.3.zip",
        length: 41_160_100,
        sha256: "8cd969ede1757d9b77e4a4a80eda5c0dc3d6ba5522d660dce331afd7a7bbc737",
    },
    Pin {
        id: "tviy-2.4",
        version: "2.4",
        kind: SourceKind::WeaselmodsDownload,
        url: "https://downloads.weaselmods.net/download/tviy/?wpdmdl=309",
        reference: "2.4",
        filename: "Tviy v2.4.zip",
        length: 6_382_769,
        sha256: "457a88611d91ff4e279163b9f1df1824e336f37d9c5c92100df84c9429acdb69",
    },
    Pin {
        id: "ub-iwd-13",
        version: "13",
        kind: SourceKind::GithubRelease,
        url: "https://github.com/Gibberlings3/iwd_unfinished_business/releases/download/v13/unfinished-business-for-icewind-dale-v13.iemod",
        reference: "v13",
        filename: "unfinished-business-for-icewind-dale-v13.iemod",
        length: 1_555_072,
        sha256: "eb12da0f5862d768bbedd7304a5282a1c17af7022df56c7ddac7f36ccb5072c3",
    },
    Pin {
        id: "urchin-2.5",
        version: "2.5",
        kind: SourceKind::WeaselmodsDownload,
        url: "https://downloads.weaselmods.net/download/urchin/?wpdmdl=294",
        reference: "2.5",
        filename: "Urchin v2.5.zip",
        length: 37_333_625,
        sha256: "2c0e4f0f527ba969c0c72b238172c5e23c9a14576c41f8403cb08bb1488d8db3",
    },
    Pin {
        id: "vito-1.1",
        version: "1.1",
        kind: SourceKind::WeaselmodsDownload,
        url: "https://downloads.weaselmods.net/download/vito/?wpdmdl=878",
        reference: "1.1",
        filename: "Vito-v1.1.zip",
        length: 10_748_549,
        sha256: "1ec906634f1d509f5d350148efc11b5ca30ca6d1f3d11aef1a0ba443a455c7f3",
    },
    Pin {
        id: "weidu-249-amd64",
        version: "249.00",
        kind: SourceKind::GithubRelease,
        url: "https://github.com/WeiDUorg/weidu/releases/download/v249.00/WeiDU-Windows-249-amd64.zip",
        reference: "v249.00",
        filename: "WeiDU-Windows-249-amd64.zip",
        length: 2_154_755,
        sha256: "b156910cbec69359fc2e42f6739aa959d49047d6fd3dc172f6bed88ffad8f927",
    },
    Pin {
        id: "willowisp-3.6",
        version: "3.6",
        kind: SourceKind::WeaselmodsDownload,
        url: "https://downloads.weaselmods.net/download/will-of-the-wisps/?wpdmdl=91",
        reference: "3.6",
        filename: "willowisp-v3.6.zip",
        length: 52_337_479,
        sha256: "6643c48ee0374ee291ef2e29a6ab74fc5f8f04e0a4bec6e2b82252a646ef0e89",
    },
];

struct Pin {
    id: &'static str,
    version: &'static str,
    kind: SourceKind,
    url: &'static str,
    reference: &'static str,
    filename: &'static str,
    length: u64,
    sha256: &'static str,
}

/// The exact install plan of the `chris-recommended` preset: run id and resolved components.
const RECOMMENDED: &[(&str, &[u32])] = &[
    ("eefixpack-main", &[0, 2]),
    ("eeex-main", &[0, 1, 2, 3, 4, 5, 6, 7, 8]),
    ("bubb-spell-menu-main", &[0]),
    ("iwdgo-main", &[0]),
    ("ub-iwd-main", &[300, 700, 800]),
    ("minervanpc-main", &[0]),
    ("karihinpc-main", &[0]),
    ("athanasiosnpc-main", &[0, 1]),
    ("guldruknpc-main", &[0]),
    ("niriknpc-main", &[0]),
    ("cassianpc-main", &[0]),
    ("bethany-main", &[0]),
    ("dendjelion-main", &[0]),
    ("dusky-main", &[0]),
    ("hommet-main", &[0]),
    ("ina-main", &[0]),
    ("lanna-main", &[0]),
    ("oakmaw-main", &[0]),
    ("orra-main", &[0]),
    ("tviy-main", &[0]),
    ("urchin-main", &[0]),
    ("vito-main", &[0]),
    ("iwdcrossmodpack-main", &[13, 30]),
    (
        "cdtweaks-main",
        &[
            60, 110, 130, 170, 180, 2010, 1010, 1120, 1142, 1290, 2090, 2152, 2170, 2192, 2210,
            2250, 2260, 2270, 2280, 2380, 2410, 2500, 2520, 2590, 2999, 3000, 3010, 3040, 3080,
            3090, 3100, 3110, 3121, 3290, 3310, 3354, 3500,
        ],
    ),
    (
        "stratagems-main",
        &[
            1600, 1610, 2000, 2500, 2510, 2520, 3510, 3540, 4030, 4115, 4250, 5900, 6020, 6030,
            6040, 6100, 6510, 6540, 6550, 6700,
        ],
    ),
    (
        "hidden-gameplay-options-main",
        &[
            10, 11, 12, 13, 14, 16, 18, 19, 20, 22, 23, 24, 25, 27, 28, 29, 30, 32, 33, 34, 35, 36,
            37, 38, 39, 300, 301, 103,
        ],
    ),
    ("cdtweaks-spell-save-penalties", &[2312]),
    ("buffbot-main", &[1, 0]),
];

#[test]
fn declares_a_standalone_iwdee_collection_with_its_own_data_folder() {
    let manifest = recipe();
    assert_eq!(manifest.collection.game, CollectionGame::Iwdee);
    assert_eq!(
        manifest.collection.application_data_directory.as_deref(),
        Some("Chriz IWD Collection")
    );
    assert_eq!(manifest.collection.game_build, "2.7.3.0");
    assert_eq!(
        manifest
            .presets
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        [PRESET]
    );
    assert_eq!(manifest.collection.features.len(), 301);
    assert!(
        validate(&manifest).is_empty(),
        "authoring validation findings: {:?}",
        validate(&manifest)
    );
}

#[test]
fn pins_the_exact_ordered_runs_and_component_lists() {
    let manifest = recipe();
    let actual = manifest
        .collection
        .runs
        .iter()
        .map(|run| {
            (
                run.run_id.as_str(),
                run.mod_id.as_str(),
                run.phase,
                run.components.as_slice(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(actual, RUNS);
    assert!(manifest
        .collection
        .runs
        .iter()
        .all(|run| run.args.is_empty() && run.postconditions.is_empty()));

    // EE Fixpack readme: directly after the official patches. The Remote Console needs every
    // EEex component at install time and runs last.
    assert_eq!(manifest.collection.runs[0].run_id, "eefixpack-main");
    assert_eq!(
        manifest.collection.runs.last().unwrap().run_id,
        "eeexremote-main"
    );
    // Interface mods before the spell menu that takes their layout; IWDGO before quest mods.
    assert!(position(&manifest, "eeex-main") < position(&manifest, "leui-iwdee-main"));
    assert_eq!(
        position(&manifest, "leui-iwdee-main") + 1,
        position(&manifest, "infinity-ui-main")
    );
    assert!(position(&manifest, "infinity-ui-main") < position(&manifest, "bubb-spell-menu-main"));
    for quest in [
        "innbb-main",
        "blinkingdead-main",
        "snowytoes-main",
        "ourlady-main",
        "skineater-main",
        "kuldahar-main",
        "thewarofbloomandrot-main",
    ] {
        assert!(
            position(&manifest, "iwdgo-main") < position(&manifest, quest),
            "{quest}"
        );
    }
    // Will of the Wisps before Bethany; the banter pack after every companion.
    assert!(position(&manifest, "willowisp-main") < position(&manifest, "bethany-main"));
    for companion in ["minervanpc-main", "cassianpc-main", "vito-main"] {
        assert!(
            position(&manifest, companion) < position(&manifest, "iwdcrossmodpack-main"),
            "{companion}"
        );
    }
    // Tweaks Anthology before SCS; its save penalties after SCS so they cover SCS spells;
    // Hidden Gameplay Options after the Tweaks Anthology and the interface mods.
    assert!(position(&manifest, "cdtweaks-main") < position(&manifest, "stratagems-main"));
    assert!(
        position(&manifest, "stratagems-main")
            < position(&manifest, "cdtweaks-spell-save-penalties")
    );
    assert!(
        position(&manifest, "cdtweaks-main") < position(&manifest, "hidden-gameplay-options-main")
    );
    // BuffBot after EEex, late as in the BG installer.
    assert!(position(&manifest, "eeex-main") < position(&manifest, "buffbot-main"));
    assert_eq!(
        position(&manifest, "buffbot-main") + 1,
        position(&manifest, "eeexremote-main")
    );
}

#[test]
fn pins_every_installer_and_artifact_source() {
    let manifest = recipe();
    assert_eq!(
        manifest
            .mods
            .iter()
            .map(|(id, installer)| (
                id.as_str(),
                installer.artifact_id.as_str(),
                installer.tp2.as_str()
            ))
            .collect::<Vec<_>>(),
        INSTALLERS
    );
    for (id, installer) in &manifest.mods {
        assert_eq!(installer.weidu_artifact_id, WEIDU, "{id} WeiDU");
        assert_eq!(installer.invocation_mode, InvocationMode::SetupName, "{id}");
        assert_eq!(installer.language, 0, "{id} language");
        assert!(
            manifest.artifacts[&installer.artifact_id]
                .archive
                .tp2_paths
                .contains(&installer.tp2),
            "{id} TP2 is published by its artifact"
        );
    }

    assert_eq!(
        manifest
            .artifacts
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>(),
        ARTIFACTS.iter().map(|pin| pin.id).collect::<BTreeSet<_>>()
    );
    for pin in ARTIFACTS {
        let id = pin.id;
        let artifact = &manifest.artifacts[id];
        assert_eq!(artifact.version, pin.version, "{id} version");
        assert_eq!(
            artifact.acquisition,
            AcquisitionPolicy::FetchOnly,
            "{id} policy"
        );
        assert_eq!(artifact.source.kind, pin.kind, "{id} source kind");
        assert_eq!(artifact.source.url, pin.url, "{id} URL");
        assert_eq!(artifact.source.reference, pin.reference, "{id} reference");
        assert_eq!(
            artifact.source.expected_filename.as_deref(),
            Some(pin.filename),
            "{id} filename"
        );
        assert_eq!(
            artifact.source.expected_length,
            Some(pin.length),
            "{id} length"
        );
        assert_eq!(artifact.source.sha256, pin.sha256, "{id} SHA-256");
        match pin.kind {
            SourceKind::WeaselmodsDownload => {
                // The anonymous package link of the reviewed Weasel Mods download page.
                let (page, package) = pin.url.split_once("?wpdmdl=").unwrap();
                assert!(page.starts_with(WEASEL_DOWNLOADS), "{id} Weasel Mods page");
                assert!(page.ends_with('/'), "{id} Weasel Mods page");
                assert!(
                    !package.is_empty() && package.bytes().all(|byte| byte.is_ascii_digit()),
                    "{id} Weasel Mods package id"
                );
                assert_eq!(artifact.provenance.homepage, page, "{id} homepage");
                assert_eq!(artifact.archive.kind, ArchiveKind::Zip, "{id} archive kind");
            }
            SourceKind::GithubCommitZip => {
                assert!(
                    pin.url.starts_with("https://codeload.github.com/")
                        && pin.url.ends_with(&format!("/zip/{}", pin.reference)),
                    "{id} commit archive URL"
                );
                assert_eq!(pin.reference.len(), 40, "{id} commit");
            }
            SourceKind::GithubTagArchive => {
                assert!(
                    pin.url.starts_with("https://codeload.github.com/")
                        && pin
                            .url
                            .ends_with(&format!("/zip/refs/tags/{}", pin.reference)),
                    "{id} tag archive URL"
                );
            }
            SourceKind::GithubRelease => {
                assert!(
                    pin.url.starts_with("https://github.com/")
                        && pin
                            .url
                            .contains(&format!("/releases/download/{}/", pin.reference))
                        && pin.url.ends_with(pin.filename),
                    "{id} release asset URL"
                );
            }
            SourceKind::Manual => panic!("{id} must not need a manual download"),
        }
    }
}

#[test]
fn resolves_the_exact_recommended_plan() {
    let manifest = recipe();
    let evaluation = recommended(&manifest);
    let actual = evaluation
        .plan
        .runs
        .iter()
        .map(|run| (run.run_id.as_str(), run.components.as_slice()))
        .collect::<Vec<_>>();
    assert_eq!(actual, RECOMMENDED);
    for run in &evaluation.plan.runs {
        let authored = &manifest.collection.runs[position(&manifest, &run.run_id)];
        assert_eq!(run.mod_id, authored.mod_id, "{}", run.run_id);
        assert_eq!(run.phase, Phase::Main, "{}", run.run_id);
        assert!(
            run.components
                .iter()
                .all(|component| authored.components.contains(component)),
            "{} resolves only authored components",
            run.run_id
        );
    }
    let components = evaluation
        .plan
        .runs
        .iter()
        .map(|run| run.components.len())
        .sum::<usize>();
    assert_eq!(components, 124);
}

#[test]
fn interface_overhauls_are_alternatives_and_both_off_by_default() {
    let manifest = recipe();
    let evaluation = recommended(&manifest);
    for id in ["mod:leui-iwdee", "mod:infinity-ui"] {
        let control = evaluation.view.control(id).unwrap();
        assert_eq!(control.decision, Decision::Optional, "{id}");
        assert_eq!(
            control.choice_group.as_deref(),
            Some("interface-overhaul"),
            "{id}"
        );
        assert_eq!(
            control.group_label.as_deref(),
            Some("User interface"),
            "{id}"
        );
        assert!(!control.selected, "{id}");
    }
    assert!(evaluation.plan.components_for("leui-iwdee-main").is_none());
    assert!(evaluation.plan.components_for("infinity-ui-main").is_none());

    let mut leui = Selection::defaults("windows");
    leui.set_feature("mod:leui-iwdee", true);
    let leui = evaluate(&manifest, &leui).unwrap();
    assert_eq!(leui.plan.components_for("leui-iwdee-main"), Some(&[0][..]));
    assert!(leui.plan.components_for("infinity-ui-main").is_none());

    // Asking for both never installs two interfaces.
    let mut both = Selection::defaults("windows");
    both.set_feature("mod:leui-iwdee", true);
    both.set_feature("mod:infinity-ui", true);
    let both = evaluate(&manifest, &both).unwrap();
    assert!(
        both.plan.components_for("leui-iwdee-main").is_none()
            || both.plan.components_for("infinity-ui-main").is_none(),
        "{:?}",
        both.plan.runs
    );
}

/// Every choice group offers alternatives: its options exclude each other, so the interface
/// can switch between them and the plan never resolves two options of one group.
#[test]
fn every_choice_group_offers_exclusive_alternatives() {
    let manifest = recipe();
    let mut groups = std::collections::BTreeMap::<&str, Vec<&str>>::new();
    for feature in &manifest.collection.features {
        if let Some(group) = feature.choice_group.as_deref() {
            groups.entry(group).or_default().push(feature.id.as_str());
        }
    }
    assert_eq!(groups.len(), 30);
    let evaluation = recommended(&manifest);
    for (group, members) in &groups {
        assert!(members.len() > 1, "{group}");
        for member in members {
            let control = evaluation.view.control(member).unwrap();
            for other in members.iter().filter(|other| *other != member) {
                assert!(
                    control.conflicts.iter().any(|conflict| conflict == other),
                    "{member} must exclude {other} in {group}"
                );
            }
            assert!(
                control.choice_available.is_some(),
                "{member} must be switchable within {group}"
            );
        }
    }

    // Asking for two quicksave counts at once never installs both.
    let mut selection = Selection::defaults("windows");
    selection.set_feature("mod:infinity-ui", true);
    selection.set_feature("feature:infinity-ui:component-8", true);
    let components = evaluate(&manifest, &selection)
        .unwrap()
        .plan
        .components_for("infinity-ui-main")
        .unwrap()
        .to_vec();
    assert!(
        components
            .iter()
            .filter(|component| (3..=8).contains(*component))
            .count()
            <= 1,
        "{components:?}"
    );
}

#[test]
fn infinity_ui_defaults_to_four_quicksave_slots_and_offers_the_other_counts() {
    let manifest = recipe();
    let evaluation = recommended(&manifest);
    // Components 3 to 8 select 1 to 6 quicksave slots.
    for component in 3..=8 {
        let id = format!("feature:infinity-ui:component-{component}");
        let control = evaluation.view.control(&id).unwrap();
        assert_eq!(
            control.choice_group.as_deref(),
            Some("infinity-ui:select-number-of-quicksave-slots"),
            "{id}"
        );
        assert_eq!(control.parent.as_deref(), Some("mod:infinity-ui"), "{id}");
        assert_eq!(control.readiness, Readiness::Ready, "{id}");
        assert_eq!(
            control.decision,
            if component == 6 {
                Decision::Default
            } else {
                Decision::Optional
            },
            "{id}"
        );
        assert!(control
            .title
            .ends_with(&format!("Quicksave Slots: {}", component - 2)));
    }

    let mut selection = Selection::defaults("windows");
    selection.set_feature("mod:infinity-ui", true);
    assert_eq!(
        evaluate(&manifest, &selection)
            .unwrap()
            .plan
            .components_for("infinity-ui-main"),
        Some(&[0, 6][..])
    );
    selection.set_feature("feature:infinity-ui:component-6", false);
    selection.set_feature("feature:infinity-ui:component-8", true);
    assert_eq!(
        evaluate(&manifest, &selection)
            .unwrap()
            .plan
            .components_for("infinity-ui-main"),
        Some(&[0, 8][..])
    );
}

#[test]
fn cassia_answers_its_portrait_prompt_with_one() {
    let manifest = recipe();
    let component = manifest.mods["cassianpc"]
        .components
        .iter()
        .find(|component| component.id == 0)
        .unwrap();
    assert_eq!(component.prompts.len(), 1);
    assert_eq!(
        component.prompts[0].expected_output,
        "[2] Darker blue by Takeda11"
    );
    assert_eq!(
        component.prompts[0].answer,
        PromptAnswer::Literal(InputValue::Integer(1))
    );
    assert!(component.prompts[0].when_any_features.is_empty());

    let evaluation = recommended(&manifest);
    let run = evaluation
        .plan
        .runs
        .iter()
        .find(|run| run.run_id == "cassianpc-main")
        .unwrap();
    let script = run
        .prompt_scripts
        .iter()
        .find(|script| script.component.component == 0)
        .unwrap();
    assert_eq!(script.steps.len(), 1);
    assert_eq!(
        script.steps[0].expected_output,
        "[2] Darker blue by Takeda11"
    );
    assert_eq!(script.steps[0].answer, "1\n");
}

#[test]
fn remote_console_requires_eeex_and_luajit_and_is_off_by_default() {
    let manifest = recipe();
    let evaluation = recommended(&manifest);
    let parent = evaluation.view.control("mod:eeexremote").unwrap();
    assert_eq!(parent.decision, Decision::Optional);
    assert!(!parent.selected);
    let console = evaluation
        .view
        .control("feature:eeexremote:component-0")
        .unwrap();
    assert_eq!(
        console.requires,
        ["feature:eeex:component-1", "feature:eeex:component-8"]
    );
    assert!(evaluation.plan.components_for("eeexremote-main").is_none());

    let mut selection = Selection::defaults("windows");
    selection.set_feature("mod:eeexremote", true);
    let enabled = evaluate(&manifest, &selection).unwrap();
    assert_eq!(enabled.plan.runs.last().unwrap().run_id, "eeexremote-main");
    assert_eq!(
        enabled.plan.components_for("eeexremote-main"),
        Some(&[0][..])
    );
    let eeex = enabled.plan.components_for("eeex-main").unwrap();
    assert!(eeex.contains(&1) && eeex.contains(&8));
}

#[test]
fn buffbot_is_on_by_default_and_installs_its_luajit_support_first() {
    let manifest = recipe();
    let evaluation = recommended(&manifest);
    let parent = evaluation.view.control("mod:buffbot").unwrap();
    assert_eq!(parent.decision, Decision::Default);
    assert!(parent.selected);
    let luajit = evaluation
        .view
        .control("feature:buffbot:component-1")
        .unwrap();
    let automation = evaluation
        .view
        .control("feature:buffbot:component-0")
        .unwrap();
    for control in [luajit, automation] {
        assert_eq!(control.decision, Decision::Mandatory, "{}", control.id);
        assert_eq!(
            control.parent.as_deref(),
            Some("mod:buffbot"),
            "{}",
            control.id
        );
        assert!(control.selected, "{}", control.id);
    }
    assert_eq!(luajit.requires, ["feature:eeex:component-1"]);
    assert_eq!(automation.requires, ["feature:buffbot:component-1"]);
    assert_eq!(
        evaluation.plan.components_for("buffbot-main"),
        Some(&[1, 0][..])
    );

    let mut selection = Selection::defaults("windows");
    selection.set_feature("mod:buffbot", false);
    let without = evaluate(&manifest, &selection).unwrap();
    assert!(without.plan.components_for("buffbot-main").is_none());
}

#[test]
fn druids_keep_their_own_xp_table_unless_the_player_picks_the_cleric_one() {
    // Christopher, 2026-10-27, as in Chriz Easy BG: on the cleric XP table druids reach
    // level 15 far too early, so the option stays offered but unticked.
    let manifest = recipe();
    let evaluation = recommended(&manifest);
    let cleric_table = evaluation
        .view
        .control("feature:cdtweaks:component-2292")
        .unwrap();
    assert_eq!(cleric_table.decision, Decision::Optional);
    assert!(!cleric_table.selected);
    assert!(!evaluation
        .plan
        .components_for("cdtweaks-main")
        .unwrap()
        .contains(&2292));

    let mut selection = Selection::defaults("windows");
    selection.set_feature("feature:cdtweaks:component-2292", true);
    let picked = evaluate(&manifest, &selection).unwrap();
    assert!(picked
        .plan
        .components_for("cdtweaks-main")
        .unwrap()
        .contains(&2292));
}

#[test]
fn hidden_gameplay_options_is_on_by_default_and_never_offers_component_40() {
    // Christopher, 2026-10-28: the mod's extra in-game options belong to the recommended setup.
    let manifest = recipe();
    let run = &manifest.collection.runs[position(&manifest, "hidden-gameplay-options-main")];
    assert!(!run.components.contains(&40));
    assert!(
        !manifest.collection.features.iter().any(|feature| feature.id
            == "feature:hidden-gameplay-options:component-40"
            || feature
                .components
                .iter()
                .any(|component| { component.run_id == run.run_id && component.component == 40 }))
    );

    let evaluation = recommended(&manifest);
    let parent = evaluation
        .view
        .control("mod:hidden-gameplay-options")
        .unwrap();
    assert_eq!(parent.decision, Decision::Default);
    assert!(parent.selected);
    assert!(evaluation
        .plan
        .components_for("hidden-gameplay-options-main")
        .is_some_and(|options| options.len() == 28));

    // The mod refuses some options beside an interface overhaul; picking one drops just those
    // options, and the overhaul stays.
    let mut selection = Selection::defaults("windows");
    selection.set_feature("mod:infinity-ui", true);
    let overhaul = evaluate(&manifest, &selection).unwrap();
    let options = overhaul
        .plan
        .components_for("hidden-gameplay-options-main")
        .unwrap();
    for refused in [29, 30, 300, 301] {
        assert!(!options.contains(&refused), "{refused}");
        assert!(
            overhaul
                .findings
                .iter()
                .any(|finding| finding.rule == "default-feature-omitted"
                    && finding.feature_id
                        == format!("feature:hidden-gameplay-options:component-{refused}")),
            "{refused}"
        );
    }
    assert!(
        options.contains(&28),
        "only Lefreut's interface refuses option 28"
    );
    assert!(overhaul.plan.components_for("infinity-ui-main").is_some());

    let mut selection = Selection::defaults("windows");
    selection.set_feature("mod:hidden-gameplay-options", false);
    let without = evaluate(&manifest, &selection).unwrap();
    assert!(without
        .plan
        .components_for("hidden-gameplay-options-main")
        .is_none());
}

#[test]
fn scs_installs_all_spell_tweaks_without_the_per_category_choices() {
    let manifest = recipe();
    let run = &manifest.collection.runs[position(&manifest, "stratagems-main")];
    assert!(run.components.contains(&2000));
    let per_category = 2010..=2080;
    assert!(!run
        .components
        .iter()
        .any(|component| per_category.contains(component)));
    assert!(!manifest.collection.features.iter().any(|feature| {
        feature.components.iter().any(|component| {
            component.run_id == run.run_id && per_category.contains(&component.component)
        })
    }));

    let evaluation = recommended(&manifest);
    let all = evaluation
        .view
        .control("feature:stratagems:component-2000")
        .unwrap();
    assert_eq!(all.decision, Decision::Mandatory);
    assert!(all.selected);
    assert!(evaluation
        .plan
        .components_for("stratagems-main")
        .unwrap()
        .contains(&2000));
}

#[test]
fn save_penalties_run_after_scs_and_default_to_arcane_and_divine() {
    let manifest = recipe();
    let run = &manifest.collection.runs[position(&manifest, "cdtweaks-spell-save-penalties")];
    assert_eq!(run.mod_id, "cdtweaks");
    assert_eq!(run.components, [2310, 2311, 2312]);
    assert!(position(&manifest, "stratagems-main") < position(&manifest, run.run_id.as_str()));

    let evaluation = recommended(&manifest);
    let order = evaluation
        .plan
        .runs
        .iter()
        .map(|run| run.run_id.as_str())
        .collect::<Vec<_>>();
    let scs = order
        .iter()
        .position(|id| *id == "stratagems-main")
        .unwrap();
    let penalties = order
        .iter()
        .position(|id| *id == "cdtweaks-spell-save-penalties")
        .unwrap();
    assert!(scs < penalties);
    assert_eq!(
        evaluation
            .plan
            .components_for("cdtweaks-spell-save-penalties"),
        Some(&[2312][..])
    );
}
