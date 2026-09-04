//! Operator helper for the same add-on path used by the desktop launcher.
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let root = PathBuf::from(
        args.next()
            .ok_or("usage: install_radar MANAGED_ROOT CACHE_ROOT")?,
    );
    let cache = PathBuf::from(
        args.next()
            .ok_or("usage: install_radar MANAGED_ROOT CACHE_ROOT")?,
    );
    if args.next().is_some() {
        return Err("unexpected argument".into());
    }
    let receipt: bg_engine::receipt::InstallReceipt =
        serde_json::from_slice(&std::fs::read(root.join(".chriz/install-receipt.json"))?)?;
    if !matches!(
        receipt.outcome,
        bg_engine::receipt::ReceiptOutcome::Succeeded
    ) {
        return Err("the game installation has not completed successfully".into());
    }
    let release = bg_engine::radar::check_latest()?;
    let installed = bg_engine::radar::install(
        &cache,
        &root,
        root.join("game"),
        &release,
        &bg_engine::events::ConsoleSink,
    )?;
    println!("{}", serde_json::to_string_pretty(&installed)?);
    Ok(())
}
