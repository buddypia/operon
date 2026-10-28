# Final review: committed change receipts

Self-review: same session; independent index reviews by rust-reviewer and
subprocess-safety-reviewer separately approved digest cd1ffcb3f6a34acb (round 2).
Base: b8d5fff. Reviewed implementation: staged index over 12907ec.

## Project profile

Rust macOS desktop application with local persistence and agent CLI launch paths.
This change adds only a local Python developer report and Rust tests; no app UI,
store, auth, payment, launch configuration, dependency, or bundle changes.
CI: GitHub Actions. House rules: REVIEW.md Important/Nit distinction, independent
review, no nit edits during review, named tests locally and full suite in CI.

## Scope and intent

Join existing committed change records to selected policy hashes and a previous
snapshot without inventing verification or run telemetry. This is the first
folder's receipt application, not completion of the whole three-folder request.
The independent source audit identified a separate eval timeout gap still to fix
before reading the second folder.

## Gate

| Command or observation | Result |
|---|---|
| check-readiness for this change | Go, zero blockers/warnings |
| cargo test --locked change_receipt | 4 passed, zero failed |
| cargo test --locked harness_documents_only_name_paths_that_exist | 1 passed |
| make q.fast | fmt and clippy exit 0 |
| check-bands | exit 0; existing steering/module/subagent warn tiers only |
| mutations.py | 7 of 7 caught; source restored |
| check-review | SHIP, zero findings, round 2 of 2 at cd1ffcb3f6a34acb |
| Committed change 224 smoke report | Correct snapshot, previous snapshot, policy hash; unavailable telemetry null |

## Findings

The partial-clone automatic-fetch issue from round 1 is fixed and pinned by a
transport probe watched failing before the fix. No remaining receipt findings.
The source audit retains the separate pre-existing eval timeout gap explicitly.

## Dimension coverage

| Dimension | Evidence / applicability |
|---|---|
| D1 Intent | Requirements map to four fixture tests; full source request remains active |
| D2 Correctness | Fixed record allowlists; immutable objects; actual hashes, parent and unknowns |
| D3 Errors | Invalid/missing revisions and paths, oversized/invalid UTF-8 records, empty state, root commit |
| D4 Tests | Missing-script and lazy-fetch failures observed; meaningful claim/evidence mutations |
| D5 Security | No shell interpolation; fixed git argv and validated change name; replacement/graft/lazy-fetch disabled |
| D6 Privacy | Only committed pipeline records, no transcript/environment dumping; no network or persistence |
| D7 Supply chain | N/A: standard library only, no manifest or CI change |
| D8 Performance | Thirteen records, 64 KiB each, 10-second git deadline; streaming and cleanup probes |
| D9 Concurrency/integrity | Read-only immutable snapshot, no mutable shared report/cache |
| D10 Reliability | Exit 2 and empty stdout on failure; explicit historical-evidence and rollback scope |
| D11 ROI | Reuses existing ledger rather than adding a runtime; no paid calls |
| D12 Compatibility | Additive developer CLI and schema version 1, null optional records, Python standard library |
| D13 DX | Discoverable feature-pilot close step, JSON stdout/errors stderr, no GUI change |

## Verified

Named tests, static checks, independent index review, realistic partial clone,
stream overflow, deadline cleanup and committed-history smoke output.

## NOT verified

No GUI or live agent restore run: neither production UI nor restore changed.
Full suite is delegated to branch CI as repository policy requires.

## Known limitations

Historical records are claims, not new verification. Selected policy files are
fingerprinted, not the user's entire global environment. A previous snapshot
is not a promise to revert a multi-commit change. Telemetry was never collected.

## Adversarial check

A green historical state could mislead someone into believing the current tree
was tested: the report explicitly labels historical evidence, pins its commit,
preserves pending contract text, and tests dirty worktree substitution.

The change is needed to join scattered evidence. Extending the existing all-change
indicators script was considered; a small bounded report keeps that costly history
scan out. No independent ledger/framework is introduced. The untested premise was
Git's default behavior in partial clones; review challenged it and a real filtered
clone now pins the fix. No added production code can be removed without losing a
specified receipt property or its bounded execution.

## Review summary

Implementation approved; mutation completion and branch CI are recorded in
state.yaml before landing. No unresolved product decisions.

NO UNRESOLVED DECISIONS
