//! Static validation of a loaded executable recipe.

use std::collections::BTreeSet;
use std::fmt;

use crate::error::EngineError;
use crate::manifest::SourceKind;
use crate::Manifest;

/// Placeholder digest used only while authoring an artifact entry.
const UNPINNED_SHA256: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// How serious a [`Finding`] is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// The recipe is usable, but something looks wrong.
    Warning,
    /// The recipe cannot be used as-is.
    Error,
}

/// One stable validation result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// Finding severity.
    pub severity: Severity,
    /// Stable kebab-case rule id.
    pub rule: &'static str,
    /// Human-readable explanation naming the offending recipe element.
    pub message: String,
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let severity = match self.severity {
            Severity::Warning => "warning",
            Severity::Error => "error",
        };
        write!(formatter, "{severity}[{}]: {}", self.rule, self.message)
    }
}

/// Rule requiring component ids to be unique within one installer.
pub const RULE_COMPONENT_IDS_UNIQUE: &str = "component-ids-unique";
/// Rule requiring fetchable source identity to be well formed.
pub const RULE_SOURCES: &str = "sources";
/// Warning emitted for an all-zero authoring digest.
pub const RULE_UNPINNED_SOURCE: &str = "unpinned-source";
/// Warning emitted for legacy stdin that lacks a terminating newline.
pub const RULE_STDIN_NEWLINE: &str = "stdin-newline";

/// Runs the currently defined recipe validation rules in stable order.
pub fn validate(manifest: &Manifest) -> Vec<Finding> {
    let mut findings = Vec::new();
    check_component_ids(manifest, &mut findings);
    check_sources(manifest, &mut findings);
    check_stdin(manifest, &mut findings);
    findings
}

/// Rejects a recipe when any validation finding has error severity.
pub fn check(manifest: &Manifest) -> crate::error::Result<()> {
    let findings = validate(manifest);
    if findings
        .iter()
        .any(|finding| finding.severity == Severity::Error)
    {
        return Err(EngineError::Validation(findings));
    }
    Ok(())
}

fn error(findings: &mut Vec<Finding>, rule: &'static str, message: String) {
    findings.push(Finding {
        severity: Severity::Error,
        rule,
        message,
    });
}

fn warning(findings: &mut Vec<Finding>, rule: &'static str, message: String) {
    findings.push(Finding {
        severity: Severity::Warning,
        rule,
        message,
    });
}

fn check_component_ids(manifest: &Manifest, findings: &mut Vec<Finding>) {
    for (id, mod_file) in &manifest.mods {
        let mut seen = BTreeSet::new();
        for component in &mod_file.components {
            if !seen.insert(component.id) {
                error(
                    findings,
                    RULE_COMPONENT_IDS_UNIQUE,
                    format!("installer {id:?} declares component {} twice", component.id),
                );
            }
        }
    }
}

fn check_sources(manifest: &Manifest, findings: &mut Vec<Finding>) {
    for (id, artifact) in &manifest.artifacts {
        if artifact.source.kind == SourceKind::Manual {
            continue;
        }

        if !artifact.source.url.starts_with("https://") {
            error(
                findings,
                RULE_SOURCES,
                format!(
                    "artifact {id:?} source url {:?} is not https",
                    artifact.source.url
                ),
            );
        }

        let sha256 = artifact.source.sha256.as_str();
        if sha256.len() != 64 || !sha256.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            error(
                findings,
                RULE_SOURCES,
                format!("artifact {id:?} source sha256 {sha256:?} is not 64 hex characters"),
            );
        } else if sha256 == UNPINNED_SHA256 {
            warning(
                findings,
                RULE_UNPINNED_SOURCE,
                format!("artifact {id:?} has an all-zero sha256, so its source is not pinned"),
            );
        }
    }
}

fn check_stdin(manifest: &Manifest, findings: &mut Vec<Finding>) {
    for (id, mod_file) in &manifest.mods {
        for component in &mod_file.components {
            if component
                .stdin
                .as_deref()
                .is_some_and(|stdin| !stdin.ends_with('\n'))
            {
                warning(
                    findings,
                    RULE_STDIN_NEWLINE,
                    format!(
                        "installer {id:?} component {} stdin does not end with a newline",
                        component.id
                    ),
                );
            }
        }
    }
}
