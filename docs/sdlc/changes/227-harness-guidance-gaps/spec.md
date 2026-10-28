# Spec: Reconstruct a change receipt from committed records

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. `change_receipt_uses_committed_evidence_and_exposes_unknowns` checks a JSON
   report for one change at a resolved commit, including artifact SHA-256 hashes,
   recorded state, approvals and review, policy hashes, and its first parent.
   Dirty worktree edits must not change the report.
2. `change_receipt_rejects_invalid_missing_and_oversized_records` checks fail-closed
   handling for invalid change names, unknown commits, absent state, non-UTF-8
   records, and records over 64 KiB. Optional absent records are null.
3. Receipts must explicitly describe records as historical claims, not fresh
   verification; unavailable model, tools, test counts, cost and correction counts
   are null. The first-parent point restores only the previous snapshot and does
   not promise to undo all commits of a change. Requirement 1 pins these claims.
4. The feature-pilot closing instructions name the command without requiring a
   second mutable ledger; requirement 1 checks that integration.

## Behaviour

`python3 scripts/change-receipt.py CHANGE [REVISION]` prints deterministic JSON
for REVISION (HEAD by default). Errors go to stderr and exit 2, with no partial
JSON. This developer report and its keys are English; it changes no app text.
A root commit has no previous snapshot. Incomplete records stay incomplete.

## Design

EXTEND existing scripts/pipeline-indicators.sh's reporting concern with a separate
small standard-library Python script, not a new application module or database.
All inputs are committed objects from the current git repository. Fixed allowlists
read six change artifacts and seven policy files; no directory or transcript scan.
Git objects are resolved with replacement objects and grafts disabled. Each git
command has a 10-second timeout and at most 64 KiB of accepted output. Blob size is
checked before reading; aggregate report is bounded by thirteen records. Optional
missing files remain null; present invalid or oversized files fail. Paths and
revision arguments are validated, and no shell or network is used. No draw path,
model route, runtime budget, persisted schema, or launch setting changes.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — DESIGN.md | No | No GUI changes or palette literals |
| Icons — src/glyphs.rs | No | No icons |
| Identifier SSOT — src/config.rs | No | Report keys owned by the script, no app identifiers duplicated |
| Durability — durability-invariants skill | No | Reads committed git objects, no app-store or cancellation changes |
| Subprocess safety — src/exec.rs | Developer script | Fixed git argv, timeout, bounded records, no shell; Rust tests use existing bounded wrappers |
| Documentation — CONTRIBUTING.md | Internal only | No public user manual or README change |
| Local-first — CONTRIBUTING.md | Yes | Standard library and local git only; no telemetry or dependency |
| Budgets | Yes | 13 fixed records, 64 KiB each, 10 seconds per git invocation |

## Flagged concerns

None. A receipt is provenance, not an independent certification of its records.

## Acceptance

- `cargo test --locked change_receipt` passes both named tests.
- `make q.fast` and `bash scripts/check-bands.sh` pass.
- New guards are watched failing on meaningful altered evidence or claims.
- Folder 1's source-audit.md is complete before reading folder 2.
- No binary release: no production Rust or Cargo changes.

## Rejected alternatives

- Capture whole agent traces and secrets: unnecessary to join existing change records.
- Invent cost or tests-passed values: unavailable measurements must remain unknown.
- Persist receipts as another ledger: recreate from immutable commit objects instead.
