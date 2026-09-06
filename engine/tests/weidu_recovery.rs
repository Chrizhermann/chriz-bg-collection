use bg_engine::weidu::recovery::{
    plan_partial_tail_recovery, verify_rollback, ExpectedInstall, RecoveryAction,
};

const PREFIX: &str = "~BASE/SETUP-BASE.TP2~ #0 #1 // Base v1\n\
~BALANCE/SETUP-BALANCE.TP2~ #0 #2 // Balance 2\n";

fn expected() -> ExpectedInstall {
    ExpectedInstall {
        tp2: "chriz-bg-modpack/setup-chriz-bg-modpack.tp2".to_owned(),
        language: 0,
        components: vec![
            110, 130, 140, 170, 190, 192, 193, 194, 195, 196, 197, 198, 410, 430, 440, 450,
        ],
    }
}

fn row(component: u32) -> String {
    format!(
        "~CHRIZ-BG-MODPACK/SETUP-CHRIZ-BG-MODPACK.TP2~ #0 #{component} // component {component}\n"
    )
}

fn statuses(successful: &[u32], install: &ExpectedInstall) -> String {
    install
        .components
        .iter()
        .map(|component| {
            if successful.contains(component) {
                format!("SUCCESSFULLY INSTALLED component {component}\n")
            } else {
                format!("NOT INSTALLED DUE TO ERRORS component {component}\n")
            }
        })
        .collect()
}

#[test]
fn plans_real_fourteen_of_sixteen_as_top_tail_rollback() {
    let install = expected();
    let installed = install
        .components
        .iter()
        .copied()
        .filter(|component| !matches!(component, 170 | 192))
        .collect::<Vec<_>>();
    let after = format!(
        "{PREFIX}{}",
        installed
            .iter()
            .map(|component| row(*component))
            .collect::<String>()
    );

    let plan =
        plan_partial_tail_recovery(PREFIX, &after, &statuses(&installed, &install), &install)
            .expect("ordered non-prefix subset is a recoverable top tail");

    assert_eq!(
        plan.action,
        RecoveryAction::RollbackAndReinstall {
            installed_to_uninstall: installed.iter().rev().copied().collect(),
            reinstall: install.components.clone(),
        }
    );
    verify_rollback(&plan, PREFIX).expect("exact historical prefix is restored");
}

#[test]
fn plans_sod_thirty_one_of_thirty_two_with_component_900_gap_as_top_tail_rollback() {
    let install = ExpectedInstall {
        tp2: "chriz-sod-remix/setup-chriz-sod-remix.tp2".to_owned(),
        language: 0,
        components: vec![
            100, 110, 120, 130, 140, 150, 145, 160, 170, 180, 175, 185, 190, 195, 210, 197, 187,
            200, 215, 220, 225, 245, 230, 240, 250, 255, 260, 270, 280, 290, 900, 910,
        ],
    };
    let installed = install
        .components
        .iter()
        .copied()
        .filter(|component| *component != 900)
        .collect::<Vec<_>>();
    let after = format!(
        "{PREFIX}{}",
        installed
            .iter()
            .map(|component| format!(
                "~CHRIZ-SOD-REMIX/SETUP-CHRIZ-SOD-REMIX.TP2~ #0 #{component} // component {component}\n"
            ))
            .collect::<String>()
    );

    let plan =
        plan_partial_tail_recovery(PREFIX, &after, &statuses(&installed, &install), &install)
            .expect("the ordered subset around the component 900 gap is recoverable");

    assert_eq!(
        plan.action,
        RecoveryAction::RollbackAndReinstall {
            installed_to_uninstall: installed.iter().rev().copied().collect(),
            reinstall: install.components.clone(),
        }
    );
    verify_rollback(&plan, PREFIX).expect("the unrelated historical prefix is preserved");
}

#[test]
fn prefix_only_completion_uses_normal_remaining_suffix() {
    let install = expected();
    let installed = install.components[..3].to_vec();
    let after = format!(
        "{PREFIX}{}",
        installed
            .iter()
            .map(|component| row(*component))
            .collect::<String>()
    );

    assert_eq!(
        plan_partial_tail_recovery(PREFIX, &after, &statuses(&installed, &install), &install)
            .unwrap()
            .action,
        RecoveryAction::ContinueSuffix {
            remaining: install.components[3..].to_vec(),
        }
    );
}

#[test]
fn rejects_foreign_suffix() {
    let install = expected();
    let after = format!(
        "{PREFIX}{}~OTHER/SETUP-OTHER.TP2~ #0 #7 // foreign\n",
        row(110)
    );
    assert!(
        plan_partial_tail_recovery(PREFIX, &after, &statuses(&[110], &install), &install).is_err()
    );
}

#[test]
fn rejects_changed_older_entry() {
    let install = expected();
    let changed = PREFIX.replace("Balance 2", "Balance 3");
    let after = format!("{changed}{}", row(110));
    assert!(
        plan_partial_tail_recovery(PREFIX, &after, &statuses(&[110], &install), &install).is_err()
    );
}

#[test]
fn rejects_status_mismatch_and_duplicate_tail_rows() {
    let install = expected();
    let after = format!("{PREFIX}{}", row(110));
    assert!(
        plan_partial_tail_recovery(PREFIX, &after, &statuses(&[], &install), &install).is_err()
    );

    let duplicate = format!("{PREFIX}{}{}", row(110), row(110));
    assert!(
        plan_partial_tail_recovery(PREFIX, &duplicate, &statuses(&[110], &install), &install)
            .is_err()
    );
}

#[test]
fn rollback_verification_allows_comments_but_rejects_leftover_rows() {
    let install = expected();
    let installed = vec![110, 140];
    let after = format!("{PREFIX}{}{}", row(110), row(140));
    let plan =
        plan_partial_tail_recovery(PREFIX, &after, &statuses(&installed, &install), &install)
            .unwrap();

    verify_rollback(
        &plan,
        &format!("{PREFIX}// Recently Uninstalled: component 110\n"),
    )
    .unwrap();
    assert!(verify_rollback(&plan, &format!("{PREFIX}{}", row(110))).is_err());
}
