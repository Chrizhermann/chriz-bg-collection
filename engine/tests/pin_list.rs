//! Keeps the pin tables in `docs/pin-list-2.7.md` in step with the recipes.
//!
//! The tables between the markers are generated from each recipe's `mods/` and `artifacts/`
//! files. Set `CEBG_UPDATE_PIN_LIST=1` to rewrite them after a pin change.

use std::collections::BTreeMap;
use std::path::PathBuf;

use bg_engine::manifest::SourceKind;
use bg_engine::Manifest;

const DOC: &str = "docs/pin-list-2.7.md";
const BEGIN: &str = "<!-- BEGIN GENERATED PINS -->";
const END: &str = "<!-- END GENERATED PINS -->";
const UPDATE: &str = "CEBG_UPDATE_PIN_LIST";

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn cell(text: &str) -> String {
    text.replace('|', "\\|")
}

fn source_label(kind: &SourceKind) -> &'static str {
    match kind {
        SourceKind::GithubRelease => "release asset",
        SourceKind::GithubTagArchive => "tag archive",
        SourceKind::GithubCommitZip => "commit archive",
        SourceKind::Manual => "manual download",
    }
}

/// One table of every artifact an installer of the recipe references.
fn recipe_table(recipe: &str, title: &str) -> String {
    let manifest = Manifest::load(&repo_root().join(recipe))
        .unwrap_or_else(|error| panic!("load recipe {recipe}: {error}"));
    let mut payload_users: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    let mut weidu_users: BTreeMap<&str, usize> = BTreeMap::new();
    for installer in manifest.mods.values() {
        payload_users
            .entry(installer.artifact_id.as_str())
            .or_default()
            .push(installer.id.as_str());
        *weidu_users
            .entry(installer.weidu_artifact_id.as_str())
            .or_default() += 1;
    }

    let mut ids: Vec<&str> = payload_users.keys().copied().collect();
    ids.extend(weidu_users.keys().copied());
    ids.sort_unstable();
    ids.dedup();

    let mut out = format!(
        "### {title}: `{recipe}/`\n\n{} archives for {} installers.\n\n\
         | Archive | Version | Used by | SHA-256 | Source |\n|---|---|---|---|---|\n",
        ids.len(),
        manifest.mods.len()
    );
    for id in ids {
        let artifact = manifest
            .artifacts
            .get(id)
            .unwrap_or_else(|| panic!("{recipe}: installer references missing artifact {id}"));
        let mut used_by: Vec<String> = payload_users
            .get(id)
            .map(|users| users.iter().map(|user| format!("`{user}`")).collect())
            .unwrap_or_default();
        if let Some(count) = weidu_users.get(id) {
            used_by.push(if *count == manifest.mods.len() {
                "WeiDU for every installer".to_owned()
            } else {
                format!("WeiDU for {count} installers")
            });
        }
        let digest = artifact.source.sha256.to_ascii_lowercase();
        out.push_str(&format!(
            "| {} | {} | {} | `{}…` | [{}]({}) |\n",
            cell(&artifact.name),
            cell(&artifact.version),
            used_by.join(", "),
            &digest[..12],
            source_label(&artifact.source.kind),
            artifact.source.url
        ));
    }
    out
}

fn generated_block() -> String {
    format!(
        "{BEGIN}\n{}\n{}{END}",
        recipe_table("recipes/curated-full-current", "Shipped recipe"),
        recipe_table("manifest", "Base recipe")
    )
}

#[test]
fn pin_list_tables_match_the_recipes() {
    let path = repo_root().join(DOC);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {DOC}: {error}"))
        .replace("\r\n", "\n");
    let start = text
        .find(BEGIN)
        .unwrap_or_else(|| panic!("{DOC} lacks {BEGIN}"));
    let end = text
        .find(END)
        .unwrap_or_else(|| panic!("{DOC} lacks {END}"))
        + END.len();
    let expected = generated_block();
    if text[start..end] == expected {
        return;
    }
    if std::env::var_os(UPDATE).is_some() {
        let updated = format!("{}{}{}", &text[..start], expected, &text[end..]);
        std::fs::write(&path, updated).unwrap_or_else(|error| panic!("write {DOC}: {error}"));
        return;
    }
    panic!(
        "{DOC} no longer matches the recipes. Regenerate it with \
         `{UPDATE}=1 cargo test -p chriz-bg-engine --test pin_list` and review the diff."
    );
}
