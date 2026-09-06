# EET compatibility excerpt attribution

Verified on 2026-09-07 against the public, immutable EET source used by CEBG. This
closes the missing-attribution item from the initial public-source audit. CEBG's
own source remains MIT; the notices preserve upstream attribution and rights.

## Exact material and origin

`engine/src/weidu/eet_compat.rs` contains a three-line Windows batch match string,
`ORIGINAL_BATCH`, and a short corrected replacement. The match string is **265
UTF-8 bytes** without a trailing newline and occurs exactly once in the pinned
[`EET/lib/macros.tph`](https://github.com/Gibberlings3/EET/blob/74e91d72bca5d073fa11c1d088b90d7ff0c7105d/EET/lib/macros.tph#L15).
The upstream file's LF SHA-256 is
`f7c6fc721f05d7bb38149f8f935153f80f6f4b0040df5ea637a5d7b955287168`, matching CEBG's
existing full-file guard. The entire macro file and EET archive are not bundled
into CEBG source or application resources.

Public history identifies **Argent77** as the contributor of `GET_USER_DIRECTORY`
and these batch lines in [commit
`38db44c2dffb3066e695c74b2c9992cf72fd246d`](https://github.com/Gibberlings3/EET/commit/38db44c2dffb3066e695c74b2c9992cf72fd246d),
dated 2024-03-26. The pinned EET readme identifies **K4thos** as project author and
documents **GNU General Public License version 3** for the author's code, preserving
other contributors' rights. [Author](https://github.com/Gibberlings3/EET/blob/74e91d72bca5d073fa11c1d088b90d7ff0c7105d/EET/readme-EET.html#L32),
[credits/license statement](https://github.com/Gibberlings3/EET/blob/74e91d72bca5d073fa11c1d088b90d7ff0c7105d/EET/readme-EET.html#L370),
[GPLv3 text](https://www.gnu.org/licenses/gpl-3.0.html).

The helper has no separate fragment-specific copyright notice. Attribution names
the evidenced contributor and project without inventing an exclusive copyright
owner or a copyright-year notice. It does not replace the upstream statement with
an MIT declaration.

## Publication disposition

- Add the evidenced contributor, exact source/commit and upstream license links to
  `THIRD_PARTY_NOTICES.md`; the notices generator maintains that entry.
- Add attribution comments immediately above `ORIGINAL_BATCH` so the source points
  directly to the notice and this record.
- Identify the 2026-09-07 CEBG change: retaining a complete spaced Documents path,
  quoting shell values/output and preserving punctuation/environment expansion.
  The original and replacement strings stay visible in source. Their bytes,
  detection guards and runtime behavior are unchanged by this attribution work.
- Retain MIT for CEBG-owned material and retain upstream rights for the excerpt.
  The notice records provenance; it neither relicenses upstream nor purports to
  decide whether a limited functional fragment imposes whole-program obligations.
  This bounded attribution resolution introduces no additional publication gate.

A source snapshot containing the same code would have the same attribution. There
is no reason to discard private Git history or disable the EET compatibility fix
to address this item.

## Verification

Read the exact pinned macro/readme via public HTTPS and inspected the introducing
commit through GitHub's public API. Compared the macro's normalized bytes and
match-string location/length against the local implementation in memory. No mod
archive was downloaded, copied into the repository or executed; no game, cache,
running installer or updater was accessed. The local Rust edit adds five comment
lines only, and no build/runtime test is needed to establish that textual scope.
This record is an attribution review, not a legal opinion or new runtime acceptance.
