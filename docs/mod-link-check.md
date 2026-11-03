# Mod link check

A nightly early warning: does every pinned mod download link still serve the file we reviewed?
When a link serves something else, the installer asks the player (the download-drift handling
of #69 and #70). This check tells us first.

It uses no AI. It is [`tools/check_mod_links.py`](../tools/check_mod_links.py) (Python standard
library only) and the scheduled workflow
[`.github/workflows/mod-link-check.yml`](../.github/workflows/mod-link-check.yml). Nothing runs on
anyone's PC.

## What it checks

- **Which links.** The downloads that the shipped recipes' mods use: Chriz Easy BG
  (`manifest` and `recipes/curated-full-current`, from the next-release branch) and Chriz Easy
  IWD (`recipes/iwd-current`, from `claude/iwdee-app`). An artifact file that no mod uses (a
  superseded pin kept as history) is not a download anyone makes and is left out, and so is the
  historical `recipes/creator-full-current`. A download that two recipes share is checked once and
  names both recipes. Manual downloads (`manual-user-supplied`) cannot be fetched by link and are
  counted as skipped.
- **How, cheapest first.** One request at a time, a short pause between links, a user agent that
  names this project, 20 second timeouts.
  - GitHub release assets: the REST API publishes each asset's size and SHA-256 digest. That
    settles the link without downloading anything.
  - Other links (GitHub archives, Weasel Mods): a `HEAD` request for the final URL and
    `Content-Length`; a `Range: bytes=0-0` request when `HEAD` is refused. Weasel Mods links
    also compare the served file name.
  - A file is downloaded and hashed only when that is the only way to know: its size differs
    from the pin (and no digest is published), or the host does not tell its size (GitHub's
    generated archives; codeload sends an `ETag` instead). The result is remembered with the
    size or `ETag`, so the same file is not downloaded again. The nightly run hashes files up to
    512 MiB; a larger one that cannot be confirmed any other way is counted as not verifiable.
- **The verdict.**

| Verdict | Meaning |
|---|---|
| same | The size, and the digest or hash where there is one, match the pin. |
| changed | The size, the SHA-256 or (Weasel Mods) the file name differs. |
| gone | HTTP 404 or 410, or the release or asset no longer exists. |
| moved | The link ends on a host that is neither the pin's own, nor GitHub's own hosts, nor one of the pin's reviewed `redirect_hosts`. |
| unreachable | A timeout, a connection or TLS failure, 5xx, 429, or another refusal, still there after one retry. |

An unreachable link is reported only from the second night in a row; the first night it is only
remembered. Servers have bad nights.

## What Christopher sees

- **Nothing**, while every link is the same.
- When something is not: one issue, **Mod link check**, with a table: mod, recipe, what changed
  (and since when), link. The next night updates the same issue; a comment lists links that are
  new or no longer reported.
- When everything is the same again, the issue gets a closing comment and closes itself.
- If the run itself fails (the next-release variable is missing, a branch is gone, a recipe
  folder is empty), the workflow run fails and GitHub sends its usual failed-run mail. An
  incomplete checkout never counts as "all links are the same".

The run's summary page lists the same findings, the number of downloads, and notes such as a
recipe folder that has artifacts but is not on the checked list.

## Setting

- Repository variable `CEBG_NEXT_RELEASE_BRANCH` (already used by the own-mod pin job) names the
  branch with Chriz Easy BG's recipes. `CEBG_IWD_BRANCH` names Chriz Easy IWD's; it defaults to
  `claude/iwdee-app`. Set it when that branch is renamed or merged into the release branch.
- Permissions: `contents: read` and `issues: write`. No secrets; the job's own token only
  raises the GitHub API rate limit and goes to `api.github.com` alone.
- What earlier nights learned (which links failed, hashes of archives) lives in the Actions
  cache, restored and saved each night. If it is lost, the unreachable rule starts over and the
  archives are hashed once more.
- A new recipe folder that ships in an installer has to be added to the `--installer` lists in
  the workflow; the run notes a folder with artifacts that is not on them.

## Run it locally

Read-only; it writes nothing into the game or the recipes. Put the recipes of each branch in a
folder of its own (no checkout needed), then:

```powershell
$env:GH_TOKEN = (gh auth token --user Chrizhermann)   # optional: more GitHub API requests
git archive origin/<next-release-branch> manifest recipes | tar -x -C $scratch\bg
git archive origin/claude/iwdee-app manifest recipes | tar -x -C $scratch\iwd
python -I tools/check_mod_links.py check `
  --installer "Chriz Easy BG" $scratch\bg manifest recipes/curated-full-current `
  --installer "Chriz Easy IWD" $scratch\iwd recipes/iwd-current `
  --state $scratch\state.json --report $scratch\report.json --max-download-mb 100
```

`--max-download-mb` keeps a local run from fetching the biggest archives (the nightly default is
512). `--only <text>` checks the links whose URL or mod name contains the text. The same
`--state` file makes a second run skip the downloads of the first. `issue --report <file>
--repo <owner/name> --dry-run` shows what the issue step would do.

Tests: `python -m unittest discover -s tools/tests`. They answer from recorded responses and
touch the network never.

## Related checks

- The own-mod pin check and daily job (#53, #55, #72, #73) look for a **newer release** of our
  own mods and prepare pins. This check asks whether the **pinned file** is still served, for
  every mod, ours included. They share the `CEBG_NEXT_RELEASE_BRANCH` variable and nothing else.
  They stay separate workflows: the pin job needs write access to contents, pull requests and
  actions and a Windows runner; this one needs only `issues: write` and runs on Ubuntu.
