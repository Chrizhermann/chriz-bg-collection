//! Static validation of a loaded [`Manifest`].
//!
//! [`validate`] runs every rule and returns all findings; [`check`] is the
//! gate that turns error-severity findings into an
//! [`EngineError::Validation`].
//!
//! Findings come back in a stable order: the rules run in the order they are
//! defined in this file (`order-refs` first, `stdin-newline` last), and each
//! rule walks the manifest in declaration order — the install order first,
//! then mods by id, then toggles, then choice groups.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::error::EngineError;
use crate::manifest::{ComponentRef, OrderEntry, Phase, SourceKind};
use crate::Manifest;

/// Mod id of the EET merge anchor, if the collection installs it.
const EET_ID: &str = "eet";
/// Mod id of the EET end anchor, if the collection installs it.
const EET_END_ID: &str = "eet_end";
/// Platform names a mod may declare.
const KNOWN_PLATFORMS: [&str; 3] = ["windows", "macos", "linux"];
/// Placeholder digest used while authoring a mod entry before it is pinned.
const UNPINNED_SHA256: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// How serious a [`Finding`] is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// The manifest is usable, but something looks wrong.
    Warning,
    /// The manifest cannot be used as-is; [`check`] rejects it.
    Error,
}

/// One validation result: which rule fired, how serious it is, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// How serious this finding is.
    pub severity: Severity,
    /// Stable kebab-case id of the rule that produced this finding.
    pub rule: &'static str,
    /// Human-readable explanation, naming the offending manifest element.
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

/// Runs every validation rule and returns all findings, worst and mildest
/// alike, in a stable order.
pub fn validate(manifest: &Manifest) -> Vec<Finding> {
    let mut findings = Findings::default();

    check_order_refs(manifest, &mut findings);
    check_component_refs(manifest, &mut findings);
    check_component_ids_unique(manifest, &mut findings);
    check_selector_ids(manifest, &mut findings);
    check_sources(manifest, &mut findings);
    check_phase_order(manifest, &mut findings);
    check_nonempty(manifest, &mut findings);
    check_option_slot(manifest, &mut findings);
    check_stdin_newline(manifest, &mut findings);

    findings.0
}

/// Validates `manifest` and fails if any finding is an [`Severity::Error`].
///
/// The returned [`EngineError::Validation`] carries *all* findings, warnings
/// included, so a caller can print the full diagnosis from the error alone.
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

/// Accumulator that keeps findings in the order the rules emit them.
#[derive(Default)]
struct Findings(Vec<Finding>);

impl Findings {
    fn error(&mut self, rule: &'static str, message: String) {
        self.0.push(Finding {
            severity: Severity::Error,
            rule,
            message,
        });
    }

    fn warning(&mut self, rule: &'static str, message: String) {
        self.0.push(Finding {
            severity: Severity::Warning,
            rule,
            message,
        });
    }
}

/// Rule `order-refs`: the install order and the mod set must agree.
///
/// Every `order[].id` names a declared mod and every declared mod appears in
/// the order at least once. A mod may occupy several slots only if *every* one
/// of its entries lists explicit components and those lists are pairwise
/// disjoint — an entry without `components` means "all declared components",
/// so it must be the mod's only entry. An explicit component list may not name
/// the same component more than once, even when the mod has only one slot.
pub const RULE_ORDER_REFS: &str = "order-refs";

fn check_order_refs(manifest: &Manifest, findings: &mut Findings) {
    let mut slots: BTreeMap<&str, Vec<usize>> = BTreeMap::new();

    for (index, entry) in manifest.collection.order.iter().enumerate() {
        if !manifest.mods.contains_key(&entry.id) {
            findings.error(
                RULE_ORDER_REFS,
                format!("order entry {index} references unknown mod {:?}", entry.id),
            );
            continue;
        }
        slots.entry(entry.id.as_str()).or_default().push(index);
    }

    for id in manifest.mods.keys() {
        if !slots.contains_key(id.as_str()) {
            findings.error(
                RULE_ORDER_REFS,
                format!("mod {id:?} never appears in the install order"),
            );
        }
    }

    for (id, indices) in &slots {
        let mut seen: BTreeMap<u32, usize> = BTreeMap::new();
        for &index in indices {
            let Some(components) = manifest.collection.order[index].components.as_ref() else {
                if indices.len() > 1 {
                    findings.error(
                        RULE_ORDER_REFS,
                        format!(
                            "mod {id:?} occupies {} order entries, so order entry {index} must \
                             list explicit components",
                            indices.len()
                        ),
                    );
                }
                continue;
            };

            for &component in components {
                match seen.insert(component, index) {
                    Some(first) if first == index => findings.error(
                        RULE_ORDER_REFS,
                        format!(
                            "order entry {index} lists component {component} of mod {id:?} twice"
                        ),
                    ),
                    Some(first) => findings.error(
                        RULE_ORDER_REFS,
                        format!(
                            "component {component} of mod {id:?} is listed by both order entry \
                             {first} and order entry {index}"
                        ),
                    ),
                    None => {}
                }
            }
        }
    }
}

/// Rule `component-refs`: every reference names something that exists.
///
/// Covers explicit `order[].components`, the [`ComponentRef`]s in toggles and
/// choice options, and the mod ids in `removes_mods`.
pub const RULE_COMPONENT_REFS: &str = "component-refs";

fn check_component_refs(manifest: &Manifest, findings: &mut Findings) {
    for (index, entry) in manifest.collection.order.iter().enumerate() {
        let (Some(mod_file), Some(components)) =
            (manifest.mods.get(&entry.id), entry.components.as_ref())
        else {
            // An unknown mod id is `order-refs`' business.
            continue;
        };

        for component in components {
            if !mod_file.components.iter().any(|it| it.id == *component) {
                findings.error(
                    RULE_COMPONENT_REFS,
                    format!(
                        "order entry {index} references component {component}, which mod {:?} \
                         does not declare",
                        entry.id
                    ),
                );
            }
        }
    }

    for toggle in &manifest.collection.toggles {
        for mod_id in &toggle.removes_mods {
            if !manifest.mods.contains_key(mod_id) {
                findings.error(
                    RULE_COMPONENT_REFS,
                    format!("toggle {:?} removes unknown mod {mod_id:?}", toggle.id),
                );
            }
        }

        for reference in &toggle.removes_components {
            check_component_ref(
                manifest,
                findings,
                &format!("toggle {:?}", toggle.id),
                reference,
            );
        }
    }

    for group in &manifest.collection.choice_groups {
        for option in &group.options {
            let origin = format!("choice option {:?} of group {:?}", option.id, group.id);
            for reference in option
                .adds_components
                .iter()
                .chain(&option.removes_components)
            {
                check_component_ref(manifest, findings, &origin, reference);
            }
        }
    }
}

fn check_component_ref(
    manifest: &Manifest,
    findings: &mut Findings,
    origin: &str,
    reference: &ComponentRef,
) {
    let Some(mod_file) = manifest.mods.get(&reference.mod_id) else {
        findings.error(
            RULE_COMPONENT_REFS,
            format!("{origin} references unknown mod {:?}", reference.mod_id),
        );
        return;
    };

    if !mod_file
        .components
        .iter()
        .any(|component| component.id == reference.component)
    {
        findings.error(
            RULE_COMPONENT_REFS,
            format!(
                "{origin} references component {}, which mod {:?} does not declare",
                reference.component, reference.mod_id
            ),
        );
    }
}

/// Rule `component-ids-unique`: a mod may not declare the same component
/// (DESIGNATED number) twice.
pub const RULE_COMPONENT_IDS_UNIQUE: &str = "component-ids-unique";

fn check_component_ids_unique(manifest: &Manifest, findings: &mut Findings) {
    for (id, mod_file) in &manifest.mods {
        let mut seen: BTreeSet<u32> = BTreeSet::new();
        for component in &mod_file.components {
            if !seen.insert(component.id) {
                findings.error(
                    RULE_COMPONENT_IDS_UNIQUE,
                    format!("mod {id:?} declares component {} twice", component.id),
                );
            }
        }
    }
}

/// Rule `selector-ids`: user-facing selectors are addressable unambiguously.
///
/// Toggle ids and choice-group ids share one namespace and must be unique
/// across both sets; option ids are unique within their group, and a group's
/// `default` names one of its own options.
pub const RULE_SELECTOR_IDS: &str = "selector-ids";

fn check_selector_ids(manifest: &Manifest, findings: &mut Findings) {
    let mut seen: BTreeSet<&str> = BTreeSet::new();

    for toggle in &manifest.collection.toggles {
        if !seen.insert(toggle.id.as_str()) {
            findings.error(
                RULE_SELECTOR_IDS,
                format!("selector id {:?} is declared more than once", toggle.id),
            );
        }
    }

    for group in &manifest.collection.choice_groups {
        if !seen.insert(group.id.as_str()) {
            findings.error(
                RULE_SELECTOR_IDS,
                format!("selector id {:?} is declared more than once", group.id),
            );
        }
    }

    for group in &manifest.collection.choice_groups {
        let mut options: BTreeSet<&str> = BTreeSet::new();
        for option in &group.options {
            if !options.insert(option.id.as_str()) {
                findings.error(
                    RULE_SELECTOR_IDS,
                    format!(
                        "choice group {:?} declares option {:?} more than once",
                        group.id, option.id
                    ),
                );
            }
        }

        if !options.contains(group.default.as_str()) {
            findings.error(
                RULE_SELECTOR_IDS,
                format!(
                    "choice group {:?} defaults to {:?}, which is not one of its options",
                    group.id, group.default
                ),
            );
        }
    }
}

/// Rule `sources`: a mod's platforms and download are well-formed.
///
/// `platforms` holds known names without duplicates. For non-manual sources,
/// `source.url` is https and `source.sha256` is exactly 64 hexadecimal
/// characters. Manual sources intentionally allow authoring placeholders: a
/// later acquire stage asks the user for the archive and records its digest.
pub const RULE_SOURCES: &str = "sources";

/// Rule `unpinned-source`: a source whose sha256 is all zeroes is a
/// placeholder from pre-hash authoring, not a real pin. Warning only, because
/// a later `--allow-unpinned` run is the intended way to obtain the digest.
pub const RULE_UNPINNED_SOURCE: &str = "unpinned-source";

fn check_sources(manifest: &Manifest, findings: &mut Findings) {
    for (id, mod_file) in &manifest.mods {
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        for platform in &mod_file.platforms {
            if !KNOWN_PLATFORMS.contains(&platform.as_str()) {
                findings.error(
                    RULE_SOURCES,
                    format!(
                        "mod {id:?} declares unknown platform {platform:?}; known platforms are {}",
                        KNOWN_PLATFORMS.join(", ")
                    ),
                );
            } else if !seen.insert(platform.as_str()) {
                findings.error(
                    RULE_SOURCES,
                    format!("mod {id:?} lists platform {platform:?} more than once"),
                );
            }
        }

        if mod_file.source.kind == SourceKind::Manual {
            continue;
        }

        if !mod_file.source.url.starts_with("https://") {
            findings.error(
                RULE_SOURCES,
                format!(
                    "mod {id:?} source url {:?} is not https",
                    mod_file.source.url
                ),
            );
        }

        let sha256 = mod_file.source.sha256.as_str();
        if sha256.len() != 64 || !sha256.bytes().all(is_hex) {
            findings.error(
                RULE_SOURCES,
                format!("mod {id:?} source sha256 {sha256:?} is not 64 hex characters"),
            );
        } else if sha256 == UNPINNED_SHA256 {
            findings.warning(
                RULE_UNPINNED_SOURCE,
                format!("mod {id:?} has an all-zero sha256, so its source is not pinned"),
            );
        }
    }
}

fn is_hex(byte: u8) -> bool {
    byte.is_ascii_hexdigit()
}

/// Rule `phase-order`: the install order respects the EET merge.
///
/// Phases along the order are non-decreasing in [`Phase`]'s ordering
/// (`bg1-pre-merge` < `main` < `post-eet-end`). When the collection installs
/// `eet` it must be a `main`-phase mod and hold the first `main` slot;
/// likewise `eet_end` must be `main` and hold the last one.
pub const RULE_PHASE_ORDER: &str = "phase-order";

fn check_phase_order(manifest: &Manifest, findings: &mut Findings) {
    let slots: Vec<(usize, &str, Phase)> = manifest
        .collection
        .order
        .iter()
        .enumerate()
        .filter_map(|(index, entry)| {
            manifest
                .mods
                .get(&entry.id)
                .map(|mod_file| (index, entry.id.as_str(), mod_file.phase))
        })
        .collect();

    let mut highest: Option<(usize, &str, Phase)> = None;
    for &(index, id, phase) in &slots {
        if let Some((high_index, high_id, high_phase)) = highest {
            if phase < high_phase {
                findings.error(
                    RULE_PHASE_ORDER,
                    format!(
                        "order entry {index} (mod {id:?}) has phase {}, but order entry \
                         {high_index} (mod {high_id:?}) already has the later phase {}",
                        phase_name(phase),
                        phase_name(high_phase)
                    ),
                );
                continue;
            }
        }
        highest = Some((index, id, phase));
    }

    check_main_anchor(manifest, &slots, findings, EET_ID, true);
    check_main_anchor(manifest, &slots, findings, EET_END_ID, false);
    check_eet_end_block(manifest, &slots, findings);
}

fn check_eet_end_block(
    manifest: &Manifest,
    slots: &[(usize, &str, Phase)],
    findings: &mut Findings,
) {
    if !manifest.mods.contains_key(EET_END_ID) {
        return;
    }

    let last_main_is_eet_end = slots
        .iter()
        .rfind(|(_, _, phase)| *phase == Phase::Main)
        .is_some_and(|(_, id, _)| *id == EET_END_ID);
    if !last_main_is_eet_end {
        // `check_main_anchor` already emits the precise last-main diagnostic.
        return;
    }

    let Some(first_eet_end) = slots.iter().position(|(_, id, _)| *id == EET_END_ID) else {
        return;
    };

    if let Some(&(index, id, _)) = slots[first_eet_end + 1..]
        .iter()
        .find(|(_, id, phase)| *phase == Phase::Main && *id != EET_END_ID)
    {
        findings.error(
            RULE_PHASE_ORDER,
            format!(
                "order entry {index} (mod {id:?}) splits the final contiguous block of mod \
                 {EET_END_ID:?}"
            ),
        );
    }
}

fn check_main_anchor(
    manifest: &Manifest,
    slots: &[(usize, &str, Phase)],
    findings: &mut Findings,
    anchor_id: &str,
    first: bool,
) {
    let Some(mod_file) = manifest.mods.get(anchor_id) else {
        return;
    };

    if mod_file.phase != Phase::Main {
        findings.error(
            RULE_PHASE_ORDER,
            format!(
                "mod {anchor_id:?} must have phase main, but declares {}",
                phase_name(mod_file.phase)
            ),
        );
    }

    let position = if first { "first" } else { "last" };
    let mut mains = slots.iter().filter(|(_, _, phase)| *phase == Phase::Main);
    let anchor = if first {
        mains.next()
    } else {
        mains.next_back()
    };

    match anchor {
        None => findings.error(
            RULE_PHASE_ORDER,
            format!(
                "mod {anchor_id:?} must be the {position} main-phase entry, but the install order \
                 has no main-phase entry"
            ),
        ),
        Some(&(index, id, _)) if id != anchor_id => findings.error(
            RULE_PHASE_ORDER,
            format!(
                "mod {anchor_id:?} must be the {position} main-phase entry, but order entry \
                 {index} (mod {id:?}) is"
            ),
        ),
        Some(_) => {}
    }
}

fn phase_name(phase: Phase) -> &'static str {
    match phase {
        Phase::Bg1PreMerge => "bg1-pre-merge",
        Phase::Main => "main",
        Phase::PostEetEnd => "post-eet-end",
    }
}

/// Rule `nonempty`: lists that would silently install nothing are rejected.
///
/// Checked in manifest order: `collection.order`, then every explicit
/// `order[].components`, then each mod's `components` and `platforms`.
pub const RULE_NONEMPTY: &str = "nonempty";

fn check_nonempty(manifest: &Manifest, findings: &mut Findings) {
    if manifest.collection.order.is_empty() {
        findings.error(RULE_NONEMPTY, "the install order is empty".to_owned());
    }

    for (index, entry) in manifest.collection.order.iter().enumerate() {
        if entry.components.as_ref().is_some_and(Vec::is_empty) {
            findings.error(
                RULE_NONEMPTY,
                format!(
                    "order entry {index} (mod {:?}) lists an empty component set; omit \
                     `components` to install all of them",
                    entry.id
                ),
            );
        }
    }

    for (id, mod_file) in &manifest.mods {
        if mod_file.components.is_empty() {
            findings.error(RULE_NONEMPTY, format!("mod {id:?} declares no components"));
        }
        if mod_file.platforms.is_empty() {
            findings.error(RULE_NONEMPTY, format!("mod {id:?} declares no platforms"));
        }
    }
}

/// Rule `option-slot`: a choice option's `adds_components` has exactly one
/// place to go.
///
/// The referenced mod must occupy the install order, and when it is split
/// across several order entries the component must be listed by exactly one of
/// them — that is the slot the resolver re-enables it in. It never invents a
/// slot. A mod with a single order entry is always fine.
pub const RULE_OPTION_SLOT: &str = "option-slot";

fn check_option_slot(manifest: &Manifest, findings: &mut Findings) {
    for group in &manifest.collection.choice_groups {
        for option in &group.options {
            for reference in &option.adds_components {
                // Unknown mods and undeclared components are `component-refs`'
                // business; only placement is judged here.
                let Some(mod_file) = manifest.mods.get(&reference.mod_id) else {
                    continue;
                };
                if !mod_file
                    .components
                    .iter()
                    .any(|component| component.id == reference.component)
                {
                    continue;
                }

                let origin = format!(
                    "choice option {:?} of group {:?} adds component {} of mod {:?}",
                    option.id, group.id, reference.component, reference.mod_id
                );
                let slots: Vec<&OrderEntry> = manifest
                    .collection
                    .order
                    .iter()
                    .filter(|entry| entry.id == reference.mod_id)
                    .collect();

                if slots.is_empty() {
                    findings.error(
                        RULE_OPTION_SLOT,
                        format!("{origin}, but that mod is not in the install order"),
                    );
                    continue;
                }
                if slots.len() == 1 {
                    continue;
                }

                let holders = slots
                    .iter()
                    .filter(|entry| match &entry.components {
                        None => true,
                        Some(components) => components.contains(&reference.component),
                    })
                    .count();
                if holders != 1 {
                    findings.error(
                        RULE_OPTION_SLOT,
                        format!(
                            "{origin}, which is split across {} order entries, but {holders} of \
                             them list that component; exactly one must",
                            slots.len()
                        ),
                    );
                }
            }
        }
    }
}

/// Rule `stdin-newline`: scripted `stdin` answers end with a newline.
///
/// Warning only, but a missing terminator makes the runner glue the next
/// answer onto this one when it concatenates them for WeiDU's READLN prompts.
pub const RULE_STDIN_NEWLINE: &str = "stdin-newline";

fn check_stdin_newline(manifest: &Manifest, findings: &mut Findings) {
    for (id, mod_file) in &manifest.mods {
        for component in &mod_file.components {
            if let Some(stdin) = component.stdin.as_ref() {
                if !stdin.ends_with('\n') {
                    findings.warning(
                        RULE_STDIN_NEWLINE,
                        format!(
                            "component {} of mod {id:?} has stdin that does not end with a newline",
                            component.id
                        ),
                    );
                }
            }
        }
    }
}
