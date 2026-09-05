# R5 modpack failure: Xan 170 and Viconia 192

Checked 2026-09-06 KST. This is owning-repo work, not a reason to drop approved
components or relax the collection's exact WeiDU verification.

## Outcome and boundaries

R5 stopped at modpack installation with WeiDU exit 2. Components **170 and 192
failed**; the other 14 requested components succeeded. The previous 383 WeiDU
rows are byte-for-byte unchanged; the final log has 397 rows. This is non-prefix
partial completion, not the earlier SoD TP2 alias mismatch. Ledger 222 correctly
records `fresh_copy_required`: `WeiDU.log changed outside the exact frozen
component suffix`. Do not resume, remove the seal, edit the frozen recipe, or
omit either component to make it pass. SoD Remix and BG Rebalance completed.

Keep this failed copy read-only until the modpack owner has captured small
reproduction fixtures. No full rebuild before candidate fixes pass focused
tests against these actual input shapes. Steam, C:\Games references, stream
installation and user saves remain untouched.

## Exact evidence

- Root: `C:\Users\chris\Games\CEBG-Curated-20260905-r5`.
- Install: `install-e6325c7c98451ad4901e`.
- Attempt: `attempt-aba3cbb32e1956c690b8`.
- Terminal: `terminal-0000000222-277ba8822bcb84fe`.
- Frozen recipe: `0.1.0-alpha.8`, payload SHA-256
  `cb220e5de749a779ec0e87337534fd803c11805375011e4b2856984120b4da34`.
- Small step evidence relative to root:
  `.chriz\attempts\attempt-aba3cbb32e1956c690b8\steps\0110-22791e178261b9e3\attempt-0001`.
  Read `process-output.log`, `invocation.json`, `process-result.json`,
  `before.log` and `after.log`, rather than the enormous full-install log.
- Terminal receipt:
  `.chriz\attempts\terminal-0000000222-277ba8822bcb84fe\receipt.json`.
- Local diagnostics archive in the active collection worktree:
  `target/curated-r5-modpack-failure-20260906.zip` (not uploaded).
- Requested components:
  `110,130,140,170,190,192,193,194,195,196,197,198,410,430,440,450`.
  Component 400 was not selected; it is not a third failure.
- Former worker PID 45792 has exited. R5 has no successful final-state receipt.

## 170: wrong Enchanter symbol in the released lookup

`chriz-bg-modpack/lib/cbm_xan_ek_fix.tpa` lines 28–35 requests
`IDS_OF_SYMBOL(~kit~ ~ENCHANTER~)` and rejects a nonpositive result.
Actual `game/override/KIT.IDS` contains:

```text
0x0200 MAGESCHOOL_ENCHANTER
0x4033 C0EK
```

It does not contain `ENCHANTER`; C0EK itself is present. The existing synthetic
fixture in `tests/test_priority_npc_fixes.py` supplies `0x0200 ENCHANTER`, hiding
this integration mismatch. Support the real canonical school symbol and cover
it with a realistic fixture; preserve the intended CRE kit encoding
`0x02000000`. Do not change the approved Xan conversion policy.

## 192: VICONI4 memorization metadata is already inconsistent

The range guard in `cbm_viconia_cleric_thief.tpa` lines 473–475 rejects the actual
pre-component creature. In the rolled-back `game/override/VICONI4.CRE`:

- File size: 3,668 bytes.
- Memorization-info offset: `0x424`, 17 rows.
- Memorized-spell offset: `0x534`, header count 7.
- Priest spell-level index 0: first 0, count 4 (in bounds).
- Priest spell-level index 1: first 4, count 4 (would need 8 records).
- Later empty rows have first 8, count 0, also beyond the header count.
- Actual seven records: `SPPR103` three times, `SPPR108`, `SPPR208`,
  `SPPR213`, `SPPR208`.

This is an installation CRE, not evidence of save corruption. The earlier mod
that produced the stale metadata has **not** been attributed. Existing clean,
tiled spellbook fixtures do not represent this input. Determine a bounded,
justified normalization/reconciliation policy and regression-test this shape;
do not merely delete the guard or invent memorized spells. Preserve the approved
class, skill, proficiency and ability policy. Temporary memorized-spell choices
are not grounds for another sprawling save-repair project.

## Owning repo and next candidate

Both library blobs are unchanged across public modpack alpha.1 through
**v0.2.0-alpha.4** and current public main `9280222`. Alpha.4 was published
2026-09-05 14:13 UTC with Yoshimo/Hexxat 220–223, but does not fix either issue.
The installed r5 libraries match the released alpha.1 blobs.

Owning task: **Add Yoshimo and Hexxat kits**,
`01a071a1-76a7-7ca3-acdf-6c79da027550` (local). Ask it to preserve the unfinished
Imoen work in the dirty repository root and use its own isolated worktree for
these two compatibility fixes. Capture only small necessary resources from r5
into local fixtures; avoid publishing proprietary game data. Use focused real
WeiDU checks in disposable fixture directories, not the stream game or r5.
Any publishing follows the owner's existing release authority, not a new grant
from this handoff. Return tested candidate/release identity and evidence limits.

Collection follow-up: expose the actual failed component names/reasons in the
UI alongside the recovery requirement; the current suffix-mismatch message is
accurate but insufficiently helpful. Keep the strict verification contract.

## Why the existing playthrough is not contradictory evidence

Read-only check of `C:\BG-EET-RC-20260903\game\WeiDU.log` found no modpack
entries: neither 170 nor 192 ran there. Xan has the separate Xan v19 component 1
and Artisan chriz-v1.3.0 NPC component 20002. That game's KIT.IDS also has
`MAGESCHOOL_ENCHANTER`, not `ENCHANTER`. The separate current-save migration plan
in the modpack `save-npc-migration` worktree documents copy-only embedded-Viconia
repair, not execution of fresh-install component 192. The checked documentation
does not itself establish final save-repair acceptance. Do not infer a broken
current save or a general class/kit engine problem from these installer failures.
