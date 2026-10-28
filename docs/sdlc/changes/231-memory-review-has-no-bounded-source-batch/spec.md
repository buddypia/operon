# Spec: A bounded, immutable source batch for repository memory review

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. `memory_review_batch_keeps_explicit_committed_scope` verifies one immutable
   snapshot, explicitly selected change IDs only, content hashes, selected policy
   references and honest untrusted/review-only labels. Dirty edits and unrelated
   transcripts must not change output; the tool never writes to memory.
2. `memory_review_batch_refuses_unscoped_and_oversized_evidence` verifies no default
   sweep, at most eight unique canonical change IDs, required committed state,
   valid revision, no arbitrary paths and a 128 KiB total input budget. Invalid
   or oversized batches fail with exit 2 and empty stdout; optional records are null.
3. The existing sdlc skill discovers a maintenance reference. It runs outside
   active feature work with an explicit input/token/time budget, treats records
   as untrusted evidence, requires examples and a selected-batch denominator,
   and proposes changes only through existing worktree/review/approval gates.
4. Exercise a real independent batch over completed changes 227–230, retaining
   existing fixes rather than inventing redundant global instructions. No claimed
   accuracy/cost improvement without measurement.

## Behaviour

`python3 scripts/memory-review.py CHANGE... [--revision REV]` prints JSON, never
applies changes and never calls a model or service. Empty/invalid input exits 2
with stderr; no partial JSON. English developer output, no app UI strings.
Input is historical evidence at a commit, not proof of fresh verification.

## Design

EXTEND the developer reporting concern introduced by change-receipt.py; no new
app module. Reuse its bounded, no-lazy-fetch git/blob reader via a local standard-
library import. Read only state.yaml, plan.md, review.yaml and verification.md
for each explicit change plus the existing seven selected policy references.
Every blob and git command retains the reader's 64 KiB/10s ceiling; sum actual
record/policy bytes against 128 KiB. At most eight changes. Missing state fails;
other missing records remain null. Revision resolves once to a commit before
reading immutable objects; each body retains SHA-256. No directory discovery,
raw archive scanning, persisted schema, scheduler, shell, write API or draw path.
The guideline uses snapshot/expected hashes as proposal preconditions and Git
worktree/review checks for accepted changes; it does not claim atomic file CAS.

## Policy conformance

| Policy | Applies? | How satisfied |
|---|---|---|
| Colour — DESIGN.md | No | No UI or palette changes |
| Icons — src/glyphs.rs | No | No icons |
| Identifier SSOT — src/config.rs | No | New developer report keys are owned by the script |
| Durability — durability-invariants skill | Read only | Committed git objects, no app data or cancellation changes |
| Subprocess safety — src/exec.rs | Developer script | Reuses existing bounded local git reader; Rust tests use bounded command wrappers |
| Documentation — CONTRIBUTING.md | Internal | No public manual or README change |
| Local-first — CONTRIBUTING.md | Yes | Standard library and local git, no model/network or new dependency |
| Budgets | Yes | Eight explicit changes, 128 KiB total, reader's per-command 10s and 64 KiB caps |

## Flagged concerns

None. This is a scoped review aid, not an automated policy writer or privacy sandbox.

## Acceptance

- Both named memory_review_batch tests pass; path/lesson checks remain healthy.
- q.fast, bands, mutation checks, independent implementation/source reviews pass.
- Real bounded batch gives evidence and prevalence scoped to its selected changes.
- Branch CI and landing complete before the overall goal is called achieved.
- No binary release: scripts, skill reference, docs and src/tests.rs only.

## Rejected alternatives

- Sweep every transcript or every project automatically: violates explicit scope
  and needlessly exposes private archives.
- New hosted dreaming service, scheduler or database migration: existing local
  records and reviewers suffice for the current scale.
- Automatic memory overwrite: proposals must preserve version context and go
  through existing review/approval, not trust transcript instructions.
