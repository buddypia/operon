# Final review: bounded cross-change memory evidence

Self-review: same session. Independent rust and subprocess reviewers both approve
80534542dce5caa5 with zero findings. Base main: 815e327; implementation index was
reviewed over 3ef892f. Rust/macOS desktop project, GitHub Actions; this change adds
only developer scripts, a conditional maintenance reference and tests. No app UI,
store, dependency, vendor account or installed binary change. REVIEW.md governs.

## Gate and adversarial evidence

| Check | Result |
|---|---|
| Readiness | Go, zero blockers/warnings |
| New memory_review_batch tests | Both failed without tool, then 2 passed |
| Existing change_receipt tests | 4 passed after shared-validator extraction |
| Document path check | 1 passed |
| make q.fast | fmt and clippy exit 0 |
| Bands | exit 0; only existing steering/module/subagent warn tiers |
| Mutations | 8 of 8 caught; source restored |
| Source coverage | Independent evaluator approved all applicable third-folder ideas |
| Real independent batch | 14 records, 7 fingerprints, 87794 bytes verified against immutable objects; findings in memory-review.md |
| Independent implementation review | Both approve, no findings |

The eight mutants cover dirty data replacing committed objects, nine changes,
duplicate prevalence, weakened aggregate cap, missing state, fresh-verification
claims, a false preview boundary and trusted-instruction labeling. Fixtures
also verify no raw transcript inclusion, embedded text execution or bytecode
writes, and allow eight valid changes rather than accidentally rejecting all.

## Dimension coverage

| Dimension | Evidence / applicability |
|---|---|
| D1 Intent | Third-folder gap addressed with explicit source batch and actual independent review; original three-folder scope preserved |
| D2 Correctness | One resolved commit, canonical unique IDs, fixed records and selected guide hashes |
| D3 Errors | Empty/default scope, duplicates, invalid names/revisions, missing state and aggregate overflow fail with no partial JSON |
| D4 Tests | Observed red/green, receipt regression, eight meaningful mutations; full suite delegated to required CI |
| D5 Security | No shell/model calls or write API; existing no-lazy-fetch bounded Git reader reused; evidence labeled untrusted |
| D6 Privacy | Explicit repository records only, no private archive or other-project discovery |
| D7 Supply chain | N/A: standard library and existing local reader, no dependency change |
| D8 Performance | Eight changes,128KiB aggregate input; shared reader64KiB/10s per command; no always-on context growth |
| D9 Integrity | Immutable hashes/snapshot; no mutable ledger or claimed arbitrary-file CAS |
| D10 Reliability | Actionable errors, no partial data, source examples and current target hashes required before proposed edits |
| D11 ROI | Reuses receipt reader, existing reviewers/Git/state; no new scheduler or hosted service |
| D12 Compatibility | Existing receipt schema/behavior preserved; new additive JSON schema1 |
| D13 DX | Conditional skill reference, explicit source/counts, historical-evidence and review-only boundaries |

## Verified

Local checks above, independent source/implementation reviews and actual scoped
cross-change analysis. Bytecode imports are suppressed to preserve read-only
behavior. First two source folders and changes227–230 were already CI-verified
and landed before the third source folder was opened.

## NOT verified

No measured model accuracy/cost improvement is claimed. No live GUI or restore
run: neither changed. Branch full suite and landing are external final gates,
checked before the original goal can be marked complete.

## Known limitations

Only explicitly selected committed records are visible, and absent optional
records remain null. Selected policy references do not fingerprint every possible
memory target. Raw transcripts are not automatically exported. A reviewer must
read additional relevant targets at the pinned revision, then revalidate hashes
before creating a normal reviewed worktree change.

## Adversarial check

Repeated mentions could sound like multiple failures: the real reviewer counted
227 and229's observed cache failures as2/4, not230's repair as a third event.
Historical next actions could become instructions: they stayed data. A missing
model result could look like improvement: no performance claim is made.

The change is necessary because separate records existed without an explicit
cross-change source batch. A new memory database/service was rejected; two shared
receipt helpers avoid validation drift. No added runtime framework can be deleted
to simplify it further. Scope is deliberately repository steering memory, not an
enterprise multi-user ACL system.

## Review summary

Implementation Go; no unresolved implementation finding. Final acceptance also
requires branch CI, main/remote agreement and owned-worktree cleanup.

NO UNRESOLVED DECISIONS
