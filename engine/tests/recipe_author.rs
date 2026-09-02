use std::fs;
use std::process::Command;

#[test]
fn capture_log_emits_stable_active_rows_with_provenance_without_writing_inputs() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("WeiDU.log");
    let log = concat!(
        "// Log of Currently Installed WeiDU Mods\r\n",
        "// ~OLD/OLD.TP2~ #0 #9 // Recently Uninstalled evidence\r\n",
        "\r\n",
        "~DLCMERGER\\DLCMERGER.TP2~ #0 #1 // Merge DLC into game\r\n",
        "~BG1UB/BG1UB.TP2~ #0 #11 // Scar and the Sashenstar's Daughter\r\n",
    );
    fs::write(&source, log).unwrap();
    let source_before = fs::read(&source).unwrap();

    let historical_order =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../manifest/install-order.tsv");
    let historical_order_before = fs::read(&historical_order).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_chriz-bg-author"))
        .args([
            "capture-log",
            "--source-label",
            "synthetic BG1 fixture",
            "--log",
        ])
        .arg(&source)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        concat!(
            "position\tsource_label\tsource_line\ttp2\tlanguage\tcomponent\tcomponent_name\n",
            "1\tsynthetic BG1 fixture\t4\tDLCMERGER\\DLCMERGER.TP2\t0\t1\tMerge DLC into game\n",
            "2\tsynthetic BG1 fixture\t5\tBG1UB/BG1UB.TP2\t0\t11\tScar and the Sashenstar's Daughter\n",
        )
    );
    assert_eq!(fs::read(&source).unwrap(), source_before);
    assert_eq!(
        fs::read(&historical_order).unwrap(),
        historical_order_before
    );
}

#[test]
fn capture_log_rejects_malformed_active_evidence_without_partial_stdout() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("WeiDU.log");
    fs::write(&source, "~BROKEN/SETUP-BROKEN.TP2~ #0\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_chriz-bg-author"))
        .args(["capture-log", "--source-label", "broken fixture", "--log"])
        .arg(&source)
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("malformed active WeiDU.log entry"));
}
