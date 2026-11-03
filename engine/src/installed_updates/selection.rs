//! Preserve receipt choices without applying a target recipe's new defaults.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::manifest::{Decision, Feature, InputSpec, InputValue};
use crate::recipe_view::{evaluate, NormalizedSelection, SelectionEvaluation};
use crate::Manifest;

/// A choice that cannot be carried forward without an explicit migration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SelectionMigrationIssue {
    /// Stable machine-readable reason for refusing automatic preservation.
    pub rule: String,
    /// Affected semantic feature, or none for an invalid overall selection.
    pub feature_id: Option<String>,
    /// Explanation suitable for an update preview.
    pub message: String,
}

/// Evaluates a target recipe while preserving a complete installed selection.
///
/// Both desired states and effective states must survive. Newly visible optional
/// features remain off; incompatible feature or input changes require migration.
/// This pure comparison does not establish that installing the resulting plan
/// over an existing game is supported or safe.
pub fn preserve_installed_selection(
    old: &Manifest,
    target: &Manifest,
    installed: &NormalizedSelection,
) -> Result<SelectionEvaluation, Vec<SelectionMigrationIssue>> {
    let baseline = evaluate(old, &installed.to_selection()).map_err(|error| {
        vec![issue(
            "installed-selection-invalid",
            None,
            format!(
                "Your saved choices could not be checked against the collection originally installed: {error}"
            ),
        )]
    })?;
    if baseline.normalized_selection != *installed {
        return Err(vec![issue(
            "installed-selection-incomplete",
            None,
            "The installation record is missing some of your choices or settings, or they disagree with the original setup. CEBG cannot safely guess the missing values.".into(),
        )]);
    }

    let old_features = feature_map(old);
    let target_features = feature_map(target);
    let mut issues = Vec::new();
    let mut selection = installed.to_selection();

    for (id, desired) in &installed.features {
        let title = old_features
            .get(id.as_str())
            .map_or(id.as_str(), |feature| feature.title.as_str());
        let Some(next) = target_features.get(id.as_str()) else {
            issues.push(issue(
                "feature-removed",
                Some(id),
                format!("Your earlier choice for {title} is no longer available in the new collection. CEBG cannot discard or replace it automatically."),
            ));
            continue;
        };
        if next.decision == Decision::Excluded {
            issues.push(issue(
                "feature-excluded",
                Some(id),
                format!("Your earlier choice for {title} is no longer offered by the new collection. An explicit decision is needed to keep your setup."),
            ));
            continue;
        }
        if next.decision == Decision::Mandatory && !desired {
            issues.push(issue(
                "newly-mandatory-feature",
                Some(id),
                format!("{title} is now required, but you previously switched it off. CEBG needs your choice before changing this setting."),
            ));
        }
        // The exact baseline comparison above established these entries.
        if let (Some(previous), Some(values)) =
            (old_features.get(id.as_str()), installed.inputs.get(id))
        {
            compare_inputs(previous, next, values, &mut issues);
            if *desired && adds_dependency(previous, next) {
                issues.push(issue(
                    "new-feature-dependency",
                    Some(id),
                    format!("{title} has new or changed requirements for other options. CEBG needs an explicit decision before changing your setup."),
                ));
            }
        }
    }

    for next in &target.collection.features {
        if next.decision == Decision::Excluded || installed.features.contains_key(&next.id) {
            continue;
        }
        if next.decision == Decision::Mandatory {
            issues.push(issue(
                "new-mandatory-feature",
                Some(&next.id),
                format!(
                    "The new option {} is required. Adding it to your existing setup needs an explicit decision.",
                    next.title
                ),
            ));
        } else {
            selection.set_feature(&next.id, false);
        }
    }
    if !issues.is_empty() {
        return Err(issues);
    }

    let evaluated = evaluate(target, &selection).map_err(|error| {
        vec![issue(
            "target-selection-invalid",
            None,
            format!("The new collection could not keep your saved choices: {error}"),
        )]
    })?;
    for previous in &baseline.view.controls {
        if let Some(next) = evaluated.view.control(&previous.id) {
            if previous.selected != next.selected {
                issues.push(issue(
                    "effective-selection-changed",
                    Some(&previous.id),
                    format!(
                        "{} would be turned {} even though your saved choice has not changed. CEBG needs an explicit decision before changing your setup.",
                        previous.title,
                        if next.selected { "on" } else { "off" },
                    ),
                ));
            }
        }
    }
    if issues.is_empty() {
        Ok(evaluated)
    } else {
        Err(issues)
    }
}

fn feature_map(manifest: &Manifest) -> BTreeMap<&str, &Feature> {
    manifest
        .collection
        .features
        .iter()
        .map(|feature| (feature.id.as_str(), feature))
        .collect()
}

fn adds_dependency(old: &Feature, target: &Feature) -> bool {
    let old_alternatives = old.requires_any.iter().collect::<BTreeSet<_>>();
    let target_alternatives = target.requires_any.iter().collect::<BTreeSet<_>>();
    (target.parent.is_some() && target.parent != old.parent)
        || target.requires.iter().any(|id| !old.requires.contains(id))
        || (!target_alternatives.is_empty() && target_alternatives != old_alternatives)
}

fn compare_inputs(
    old: &Feature,
    target: &Feature,
    values: &BTreeMap<String, InputValue>,
    issues: &mut Vec<SelectionMigrationIssue>,
) {
    let old_ids = old
        .inputs
        .iter()
        .map(InputSpec::id)
        .collect::<BTreeSet<_>>();
    let target_ids = target
        .inputs
        .iter()
        .map(InputSpec::id)
        .collect::<BTreeSet<_>>();
    for id in old_ids.symmetric_difference(&target_ids) {
        issues.push(issue(
            "input-schema-changed",
            Some(&old.id),
            format!(
                "The {id} setting for {} was added or removed. Its value cannot be chosen automatically.",
                old.title
            ),
        ));
    }
    for previous in &old.inputs {
        let Some(next) = target
            .inputs
            .iter()
            .find(|input| input.id() == previous.id())
        else {
            continue;
        };
        let Some(value) = values.get(previous.id()) else {
            continue;
        };
        let failure = match (previous, next, value) {
            (InputSpec::Boolean { .. }, InputSpec::Boolean { .. }, InputValue::Boolean(_)) => None,
            (
                InputSpec::Integer { .. },
                InputSpec::Integer { min, max, .. },
                InputValue::Integer(value),
            ) => {
                if (*min..=*max).contains(value) {
                    None
                } else {
                    Some((
                        "input-value-invalid",
                        "your saved number is outside the newly allowed range",
                    ))
                }
            }
            (
                InputSpec::Choice {
                    options: old_options,
                    ..
                },
                InputSpec::Choice {
                    options: new_options,
                    ..
                },
                InputValue::Choice(value),
            ) => match (
                old_options.iter().find(|option| option.id == *value),
                new_options.iter().find(|option| option.id == *value),
            ) {
                (Some(previous), Some(next)) if previous.answer == next.answer => None,
                (Some(_), Some(_)) => Some((
                    "input-meaning-changed",
                    "your saved option now means something different",
                )),
                _ => Some((
                    "input-value-invalid",
                    "your saved option is no longer available",
                )),
            },
            _ => Some(("input-schema-changed", "the kind of setting has changed")),
        };
        if let Some((rule, reason)) = failure {
            issues.push(issue(
                rule,
                Some(&old.id),
                format!(
                    "The {} setting for {} needs a decision: {reason}.",
                    previous.id(),
                    old.title
                ),
            ));
        }
    }
}

fn issue(rule: &str, feature_id: Option<&str>, message: String) -> SelectionMigrationIssue {
    SelectionMigrationIssue {
        rule: rule.into(),
        feature_id: feature_id.map(str::to_owned),
        message,
    }
}
