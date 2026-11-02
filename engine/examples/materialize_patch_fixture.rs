//! Acceptance helper for the Updates game-fix controls in a packaged app.
//!
//! Creates one disposable, registered synthetic installation below a new scratch
//! directory, mirroring the engine patch-test fixture. Launch the packaged app
//! with `LOCALAPPDATA=<scratch>\localappdata` so its registry, locks and patch
//! backups stay isolated. Nothing outside the scratch directory is written, no
//! game or mod is installed, and the synthetic `Baldur.exe` is never executable.
use bg_engine::{
    digest::{plan_digest, selection_digest, sha256_bytes},
    manifest::GameRoot,
    patches::description::TABLES,
    receipt::{
        FinalLogReceipt, FinalReceiptState, InstallReceipt, LogComponentReceipt, ReceiptOutcome,
        ReceiptVersions, RECEIPT_SCHEMA_VERSION,
    },
    recipe_view::NormalizedSelection,
    registry::{ManagedInstallRecord, ManagedInstallRegistry, REGISTRY_SCHEMA_VERSION},
    resolve::InstallPlan,
    session::{CampaignCreated, SessionEvent, SessionStore, SourceGameFingerprints},
};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

#[allow(dead_code)]
#[path = "../tests/support/fakegame.rs"]
mod fakegame;

const INSTALL_ID: &str = "install-patch-acceptance";
const ENGINE_NAME: &str = "CEBG patch acceptance fixture";
const LOG: &[u8] = b"// Synthetic component evidence\r\n~ArtisansKitpack/ArtisansKitpack.TP2~ #0 #7004 // Assassin: chriz-v1.3.0\r\n";
const KITS: &[u8] = b"2DA V1.0\r\n*\r\nROWNAME LOWER MIXED HELP\r\n0 ASSASIN 10 11 1\r\n";
const CLASSES: &[u8] = b"2DA V1.0\r\n*\r\nLOWER MIXED OTHER DESCSTR EXTRA\r\nASSASSIN 10 11 12 0 unchanged\r\nMAGE 20 21 22 0 unrelated\r\n";
const TP2: &[u8] = b"BACKUP ~ArtisansKitpack/backup~\nAUTHOR ~Synthetic test fixture~\nVERSION ~chriz-v1.3.0~\nBEGIN ~Assassin~ DESIGNATED 7004\nPRINT ~Synthetic metadata only~\n";

fn write(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    fs::create_dir_all(path.parent().expect("fixture paths have parents"))?;
    fs::write(path, bytes)
}

/// Two-entry English TLK: the fixture's KITLIST HELP reference (1) is valid.
fn tiny_tlk() -> Vec<u8> {
    let mut data = vec![0_u8; 18 + 2 * 26];
    data[..8].copy_from_slice(b"TLK V1  ");
    data[10..14].copy_from_slice(&2_u32.to_le_bytes());
    data[14..18].copy_from_slice(&70_u32.to_le_bytes());
    for index in 0..2 {
        let at = 18 + index * 26;
        data[at..at + 2].copy_from_slice(&1_u16.to_le_bytes());
        data[at + 18..at + 22].copy_from_slice(&(index as u32).to_le_bytes());
        data[at + 22..at + 26].copy_from_slice(&1_u32.to_le_bytes());
    }
    data.extend_from_slice(b"AB");
    data
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments: Vec<_> = std::env::args_os().skip(1).collect();
    let [scratch] = arguments.as_slice() else {
        return Err("usage: materialize_patch_fixture <new-empty-scratch-directory>".into());
    };
    let scratch = PathBuf::from(scratch);
    if !scratch.is_absolute() {
        return Err("the scratch directory must be an absolute path".into());
    }
    if scratch.exists() && fs::read_dir(&scratch)?.next().is_some() {
        return Err(format!("refusing to reuse non-empty {}", scratch.display()).into());
    }
    fs::create_dir_all(&scratch)?;
    let scratch = fs::canonicalize(&scratch)?;
    if scratch
        .to_string_lossy()
        .to_ascii_lowercase()
        .contains(r"\games\")
    {
        return Err("refusing a scratch path below a Games folder".into());
    }

    let root = scratch.join("managed");
    fs::create_dir(&root)?;
    let root = fs::canonicalize(root)?;
    let game = root.join("game");
    fakegame::FakeGame::build(&game)?;
    write(
        &game.join("Baldur.exe"),
        b"MZ synthetic marker; never execute",
    )?;
    write(
        &game.join("engine.lua"),
        format!("engine_name = '{ENGINE_NAME}'\n").as_bytes(),
    )?;
    write(&game.join("weidu.conf"), b"lang_dir = en_US\r\n")?;
    write(&game.join("WeiDU.log"), LOG)?;
    write(&game.join("override/KITLIST.2DA"), KITS)?;
    for table in TABLES {
        write(&game.join("override").join(table), CLASSES)?;
    }
    write(&game.join("ArtisansKitpack/ArtisansKitpack.TP2"), TP2)?;
    let tlk = tiny_tlk();
    write(&game.join("lang/en_US/dialog.tlk"), &tlk)?;
    write(&game.join("dialog.tlk"), &tlk)?;
    let save_root = root.join("profile");
    write(
        &save_root.join("save/00001-test/BALDUR.gam"),
        b"synthetic save; immutable",
    )?;

    // Every real installation owns a frozen campaign; diagnostics export opens it.
    let cache = scratch.join("download-cache");
    fs::create_dir(&cache)?;
    let selection = NormalizedSelection {
        platform: "windows".into(),
        features: BTreeMap::new(),
        inputs: BTreeMap::new(),
    };
    let payload = b"PK\x03\x04synthetic patch acceptance recipe".to_vec();
    let envelope = br#"{"kind":"patch-acceptance-fixture"}"#.to_vec();
    let install_plan = InstallPlan { runs: Vec::new() };
    SessionStore::create(
        &root,
        SessionEvent::Created(Box::new(CampaignCreated {
            install_id: INSTALL_ID.into(),
            attempt_id: "attempt-complete".into(),
            managed_root: root.clone(),
            cache_root: fs::canonicalize(&cache)?,
            recipe_payload_sha256: sha256_bytes(&payload),
            recipe_payload: payload.clone(),
            recipe_envelope_sha256: sha256_bytes(&envelope),
            recipe_envelope: envelope.clone(),
            selection_sha256: selection_digest(&selection)?,
            normalized_selection: selection.clone(),
            plan_sha256: plan_digest(&install_plan)?,
            source_games: SourceGameFingerprints {
                bg1: "11".repeat(32),
                bg2: "22".repeat(32),
            },
            artifact_identities: Vec::new(),
            tool_identities: Vec::new(),
            staged_bg1: root.join("bg1"),
            staged_bg2: game.clone(),
        })),
    )?;

    let recipe = "0.1.0-alpha.fixture";
    let receipt = InstallReceipt {
        schema_version: RECEIPT_SCHEMA_VERSION,
        install_id: INSTALL_ID.into(),
        attempt_id: "attempt-complete".into(),
        evidence_attempt_id: "attempt-complete".into(),
        managed_root: root.clone(),
        staged_bg1: root.join("bg1"),
        staged_bg2: game.clone(),
        started_at_millis: 1,
        completed_at_millis: 2,
        outcome: ReceiptOutcome::Succeeded,
        versions: ReceiptVersions {
            application: "0.1.0-alpha.18".into(),
            engine: "0.1.0".into(),
            manifest_schema: 2,
            recipe: recipe.into(),
        },
        source_games: Vec::new(),
        recipe_payload_sha256: sha256_bytes(&payload),
        recipe_envelope_sha256: sha256_bytes(&envelope),
        selection_sha256: selection_digest(&selection)?,
        normalized_selection: selection,
        plan_sha256: plan_digest(&install_plan)?,
        plan: install_plan,
        artifacts: Vec::new(),
        weidu_tools: Vec::new(),
        runs: Vec::new(),
        final_state: Some(FinalReceiptState {
            logs: vec![FinalLogReceipt {
                target: GameRoot::Bg2,
                sha256: sha256_bytes(LOG),
                components: vec![LogComponentReceipt {
                    // Receipts record the normalized TP2 key, as real installations do.
                    tp2: "artisanskitpack/artisanskitpack.tp2".into(),
                    language: 0,
                    component: 7004,
                }],
            }],
            bg1_engine_name: ENGINE_NAME.into(),
            bg2_engine_name: ENGINE_NAME.into(),
            managed_save_root: save_root.clone(),
            launch_path: game.join("Baldur.exe"),
            verification_summary: "synthetic completed installation".into(),
        }),
    };
    let receipt = serde_json::to_vec_pretty(&receipt)?;
    write(&root.join(".chriz/install-receipt.json"), &receipt)?;

    let local_app_data = scratch.join("localappdata");
    let app_data = local_app_data.join("Chriz BG Collection");
    fs::create_dir_all(&app_data)?;
    ManagedInstallRegistry::open_or_create(&app_data)?.publish(&ManagedInstallRecord {
        schema_version: REGISTRY_SCHEMA_VERSION,
        install_id: INSTALL_ID.into(),
        display_name: "Patch acceptance fixture".into(),
        managed_root: root.clone(),
        recipe_version: recipe.into(),
        recipe_sha256: sha256_bytes(&payload),
        engine_name: ENGINE_NAME.into(),
        managed_save_root: save_root,
        launch_path: game.join("Baldur.exe"),
        receipt_sha256: sha256_bytes(&receipt),
        completed_at_millis: 2,
    })?;

    println!("LOCALAPPDATA={}", local_app_data.display());
    println!("MANAGED_ROOT={}", root.display());
    println!("INSTALL_ID={INSTALL_ID}");
    Ok(())
}
