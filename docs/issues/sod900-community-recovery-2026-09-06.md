# SoD component 900 community failure and bounded recovery

## Scope and source

This note records a read-only assessment of the user-supplied diagnostic ZIP for one
public app alpha.10 / recipe alpha.11 installation. The ZIP was read directly without
extraction. No game, installation, or diagnostic file was changed, and no private raw
log or user path is reproduced here.

The exported attempt receipt contains redaction placeholders and is not valid JSON as
exported. Removing only standalone placeholder lines in memory made its non-sensitive
plan metadata inspectable, but that transformed text is not cryptographic evidence and
must not be used to accept a recovery. Any supervised recovery must use the preserved,
unredacted `.chriz` evidence in the original managed installation.

## Established failure

The failed run requested the exact 32-component SoD sequence ending
`280, 290, 900, 910`. Its process exited with code 2. Component 900 inspected
`BD1000.ARE` before its `COPY` committed, found `Container009` with count 3 and first
item `SW1H01`, and rejected the input because v0.6.7 requires exactly one item,
`SW1H01`. WeiDU then reported zero files uninstalled for component 900, marked it not
installed, continued, and installed component 910 successfully.

The active WeiDU stack had 310 rows before the attempt and 341 afterward. All 310
earlier active identities are unchanged. The 31 appended rows use the expected nested
SoD TP2 identity and language 0, and are the authored 32-component sequence with only
900 absent; 910 occupies the final appended row. This is an ordered subset, not an exact
prefix. The normal resume verifier therefore correctly refuses suffix continuation:
installing 900 now would produce the wrong `..., 910, 900` order.

This proves a real released-component compatibility failure for this installation. It does not
yet identify which earlier component produced the two additional chest items, because
the diagnostic records only the count and first item, not the other entries or their
resource provenance. One occurrence also does not prove that every alpha.11 install has
the same input bytes. The owning mod fix should accept the compatible effective chest
shape without discarding foreign items and add a fixture matching the observed
three-item input.

## Work that remains after the failed run

The frozen plan contains six unstarted runs after SoD, totaling 62 components:

1. `hiddengameplayoptions-bg2` — 28 components
2. `chriz-bg-rebalance-bg2` — 10 components
3. `chriz-bg-modpack-bg2` — 20 components
4. `cdtweaks-spell-save-penalties-bg2` — component 2312
5. `spell-rev-npc-spellbooks-bg2` — component 60
6. `buffbot-bg2` — components 1 and 0

Their six unique archives were already cache hits in the receipt and total exactly
39,105,508 bytes. This explains BuffBot's absence: its run is last and was never
started. These archive sizes do not estimate repair duration or prove the cached files
still exist unchanged.

## Supported supervised recovery shape

The existing partial-tail planner recognizes this exact 31-of-32 ordered subset as
`RollbackAndReinstall`. A bounded repair need not rebuild the earlier 310-row stack:

1. preserve the failed campaign and all original `.chriz` evidence unchanged;
2. independently verify and stage a corrected SoD replacement release;
3. under a supervised worker, uninstall all 31 active rows from the failed SoD run in
   reverse stack order;
4. prove the active log returned to the exact historical 310-row prefix;
5. install the complete corrected 32-component SoD run in authored order; and
6. run and verify the six remaining frozen runs through BuffBot.

The original terminal failure remains immutable. The separate supervised-recovery
acceptance path can validate a replacement artifact, exact rollback, complete repaired
run, remaining tail, final logs, postconditions, and a separate completion receipt. It
does not execute WeiDU or expose automatic repair in the app.

## Evidence still required from the remote installation

The diagnostic ZIP is sufficient to plan, but not to perform or accept, a repair. The
following must still be available and independently verified:

- the original managed installation folder, including its unredacted hash-chained
  ledger, attempt evidence, current game files, WeiDU backups, and current `WeiDU.log`;
- a released corrected SoD archive with recorded length and SHA-256, plus verified
  extracted payload and the same pinned WeiDU tool identity;
- hashed intent, process result, before/after log, stdout, stderr, and debug sidecars for
  the rollback, corrected SoD install, and each of the six remaining runs;
- proof that rollback restored the exact 310-row historical prefix before reinstall;
- exact-prefix/log/status reconciliation and postconditions for every repaired or
  remaining run, followed by final-log and launcher/save-identity validation; and
- confirmation that the target stayed untouched between diagnostic export, assessment,
  and supervised execution.

Until those checks exist, the safe player instruction is to keep the folder unchanged
and export/preserve diagnostics. No automatic repair UI or new recovery executor is
shipped by this note.
