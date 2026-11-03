# Fork versions and upstream bases

September 29 user decision: use the **upstream version followed by our revision**
for fork releases. Christopher rejected continuing Artisan's independent
`chriz-v1.x` naming. This supersedes the earlier suggestion to present future
releases as "Chriz fork 1.6.0".

## Naming rule for the next releases

- Preferred form: `<upstream-version>-chriz.<revision>`. Existing upstream-based
  suffixes such as Bardic's `-balance.N` already satisfy the same principle.
- The first part identifies the upstream base; the suffix identifies our patch
  revision. If only our changes advance, increment the suffix. If upstream
  advances with our patch layer unchanged, change the upstream part only. If
  adapting our changes to upstream changes the patch layer, increment both.
- Do not reset our revision merely because the upstream version changes. Record
  the exact upstream commit with every release, including upstream-only updates.
- Untagged upstream snapshots must be identified honestly, for example with a
  `-dev.<short-upstream-commit>` base qualifier. Do not label hundreds of post-tag
  commits as the exact older tag, invent an upstream version, or reuse a released
  version/archive identity for different contents.
- Apply the scheme to Artisan's next release, including all three TP2 VERSION
  fields, archive/release metadata, tests and CEBG pins together. Record the
  migration from legacy `chriz-v1.6.0`; do not rename or replace that published
  tag/archive or relabel already-installed copies.

### Artisan's verified base and migration target

As of September 29, upstream's latest named release is
[v4.81a](https://github.com/TheArtisanBG/The-Artisan-s-Kitpack/releases/tag/v4.81a)
at `a4b469988a568f8f7b943d67512297e05da03eeb`. The included upstream/master
`85928f964f6965fd37980bf679d7eeb9af410256` is **297 commits later**, with no
upstream TP2 VERSION directive or newer development label. It is not the exact
v4.81a release. Use **`v4.81a-dev.85928f9-chriz.1`** for the next prepared owner
release: accurate upstream snapshot first, our new suffix sequence second.
Starting the suffix sequence at 1 is the migration convention, not an upstream
fact or a renaming of old releases. Subsequent snapshot changes update the base
hash even when our patch revision is unchanged.

The Artisan owner published this identity on September 29, from merge
`0798dc4ee3d09b5d97c56b69c80d9d2ac2317911` (PR #11). All three TP2 versions
match, 12 focused checks passed, and all 2,705 downloaded archive files match the
release commit. The new archive is 89,631,846 bytes, SHA-256
`6d954fab27b86c4a94b8fc06b2f8b5a34b9ae0b3f09cb03ce38de1d5ce80a73b`.
CEBG integration is now in progress. Published legacy `chriz-v1.6.0` tags and
assets remain unchanged; old archive bytes are not relabeled.

## Existing releases (historical identities)

| Fork release approved for CEBG | Meaning | Included upstream base checked September 29 |
|---|---|---|
| Artisan `chriz-v1.6.0` | Legacy independent numbering; migrate the next release to the rule above | `TheArtisanBG/The-Artisan-s-Kitpack` commit `85928f964f6965fd37980bf679d7eeb9af410256` |
| Bardic Wonders `v2.9c-balance.6` | Upstream-derived 2.9c naming plus our sixth balance revision | `TheArtisanBG/Bardic-Wonders` commit `4e72d9739d6728d688484618281ed6c3a7428b4c` |
| Spell Revisions `v4.21-chriz.5` | Upstream 4.21 naming plus our fifth fork revision | `Gibberlings3/SpellRevisions` upstream master / v4.21 `af1dedb6` |

These upstream bases have no incoming commits according to the completed owner
reviews. That is a source-history result, not combined-install or gameplay testing.
CEBG's September 29 source refresh has pinned and verified the Artisan and Bardic
archives above; SR was already pinned. This does not mean a new installer package
has been published.

## Recording and displaying versions

- Keep already-published tags and archives unchanged. Rewriting a tag to improve
  its name would break provenance and make existing installations harder to audit.
- Preserve the exact fork version in artifact metadata, checksums, manifests and
  WeiDU logs. All three Artisan installers belong to the same fork release.
- In human-facing release notes, lead with the upstream version and our suffix,
  not an unrelated independent version. Label historical Artisan 1.6 explicitly
  as the legacy fork version when it must be mentioned; never as upstream 1.6.
- For every new fork release, record upstream repository, exact base commit,
  upstream release label when applicable, and our patch revision separately.
- Before each source refresh, compare the chosen fork with the current upstream
  history. A newer fork version alone does not prove upstream is incorporated.

This records version identity and release-writing guidance. The new owner release
implements the migration; it does not rename historical tags, update installed
games or broaden CEBG's existing-game update support.
