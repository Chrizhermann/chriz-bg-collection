# Installation diagnostics checkpoint — 2026-09-06

Christopher will run a normal UI installation himself. No additional full test install
was started for this investigation, and no game folders or BuffBot files were changed.
The supplied community diagnostics now establish that SoD Remix 900 failed before
the BuffBot run was reached. See the [confirmed cause and recovery assessment](issues/sod900-community-recovery-2026-09-06.md).
A successful local run alone would not have established what happened on the tester's PC.

## What an empty BuffBot attempts list establishes

`runs[].attempts: []` means no finalized per-run attempt record was embedded when that
terminal receipt was assembled. It does not prove that the process never launched:
process evidence is written before reconciliation produces the consolidated record.
An earlier failure, interruption before reconciliation, or an older terminal receipt
from before a resume can explain the fragment. It does not establish user error.

The receipt validator requires all planned components to be accounted for on success.
A validated successful receipt cannot have empty attempts for BuffBot's planned `[1, 0]`.
Read the top-level outcome and attempt IDs before interpreting a single run fragment.

## Existing evidence and the small readability improvement

The installer already retains versions, selected options, source identities, durable
step transitions, invocation/process results and output, and before/after WeiDU logs.
New diagnostic ZIPs include `START-HERE.txt` with the selected receipt's outcome,
stopped step/reason, versions, and per-run finalized attempt/log-addition counts.
The summary uses the existing path/secret redaction and does not infer success from
exit code zero, a copied folder, or an absent record. Original evidence is unchanged.

Useful local paths inside the managed installation:

- `.chriz/ledger/`: durable step transitions, including later resume progress.
- `.chriz/attempts/<attempt>/receipt.json`: immutable terminal receipt.
- `.chriz/attempts/<evidence-attempt>/steps/<step>/attempt-NNNN/`: raw invocation,
  process output/results, debug output, and before/after WeiDU logs when available.
- `.chriz/evidence/runs/<run>/attempt-NNNN.json`: consolidated per-run records.

In exported ZIPs the evidence is under `receipt/`, `ledger/`, and `logs/steps/`.
The selected receipt is a terminal snapshot, not a live progress indicator. Older
failure receipts can coexist with a later successful installation.

## Limits and tonight's test

A crash before the first terminal receipt currently prevents diagnostics ZIP export;
raw ledger/attempt evidence can still be inspected. Preserve the installation folder
on failure, including `.chriz`, and capture evidence before manually installing mods
or cleaning up. Missing files may locate where evidence stopped but cannot by
themselves distinguish an OS crash, power loss, or forced termination.

The prior published app already writes the underlying evidence. The summary and
preceding clarity/support changes are included in app alpha.11; release verification
is tracked in [patch acceptance](patch-acceptance-alpha11-2026-09-06.md). No install
needs to be repeated merely to gain this summary. JSON/JSONL exports now redact
structured values without breaking JSON syntax; original evidence is unchanged.

Verification: the two new summary regressions failed before implementation and passed
after it; all 9 diagnostics tests and 10 receipt tests passed from PowerShell. Tests
cover empty attempts, partial log additions despite exit zero, and failure-detail
redaction. No new live installation or native-window acceptance was performed.
