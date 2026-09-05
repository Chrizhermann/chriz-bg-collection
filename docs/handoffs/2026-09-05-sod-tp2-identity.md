# R4: successful SoD install rejected by the recipe's TP2 alias

At ledger 216, r4 (`install-83f1bf87d5eb83425ed7`, frozen alpha.7) became permanently
`fresh_copy_required`: `WeiDU.log changed outside the exact frozen component suffix`.
This is a collection authoring mistake, not a new SoD v0.6.5 mod failure.

## Bounded evidence

Attempt evidence lives under r4 `.chriz/attempts/attempt-06284c52f0b43b3c6f2d/steps/`
`0107-c656b8d09bdc1ad9/attempt-0001/`. Sanitized diagnostics were exported to
`target/curated-r4-sod-path-mismatch-20260905` (one 1,753,257-byte bundle, 553 entries).

- Process exited 0; exactly 30 `SUCCESSFULLY INSTALLED` markers, including 120 and 225.
- BG2 active log grew from 316 to 346 entries; all preceding lines were unchanged.
- All appended component IDs, their order and language 0 match the invocation.
- Every appended TP2 identity is `chriz-sod-remix/setup-chriz-sod-remix.tp2`.
- Frozen metadata instead expected `setup-chriz-sod-remix.tp2` at the game root.
- Both TP2 copies are shipped by the verified artifact and are byte-identical. WeiDU's
  setup-name resolution selected/logged the nested copy. That full path, not a basename
  alias, must be the expected identity.
- `engine/src/weidu/verify.rs` intentionally compares full slash/case-normalized paths.
  Final snapshot verification would reject the same mismatch too; bypassing one check
  or removing the terminal seal would not be a proper fix.

## Correction and verification

Commit `b14c50a` changes only the authored SoD TP2 identity, advances generated recipe
version to alpha.8, and adds regression coverage. 28 Python tool tests, four production
Chriz recipe tests, and 11 WeiDU verification tests pass; format/diff checks pass.
The synthetic check proves nested identity succeeds while the root alias still fails.
Stale production test assertions for v0.6.4 were updated to already-pinned v0.6.5 metadata.
No engine validation was weakened and no mod source was edited.

Generated recipe commit `c4d607e` changes only the TP2, recipe version and static evidence
identity. The real engine still resolves 43 runs / 430 components, with exactly the same
selection and plan hashes. The remaining modpack/rebalance root installers were checked:
their artifacts publish only the root TP2 and do not have this duplicate nested copy.

R4 remains failed and unchanged. A corrected isolated r5 was started with the existing
verified cache; see the current acceptance record for identity/process/checkpoint. Never
describe the r4 partial install as a successful full collection or resume its sealed attempt.

## Cleanup outcome

The proposed old r2/r3 removal was rejected by the tool before execution. Both remain.
Read-only checks found the expected IDs, no reparse points or in-tree save directories;
diagnostic bundles are preserved, but these checks do not override the tool rejection.
Do not retry either target by another tool or shell. No files were deleted in this turn.

The optional SoD-skip work is separate: owner reports local prototypes and 23 synthetic
tests, not a released/installable candidate. Native ground-pile and effective EET import
checks remain; do not include it in r5 or treat it as an installer release blocker.
