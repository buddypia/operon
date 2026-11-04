# Intent: a review verdict is an opinion anyone can type, not evidence anyone can re-run

- **Status**: approved
- **Opened**: 2026-11-04

The person's request, verbatim: 「적대적 리뷰 대신에 1. 경험적 검증 (Empirical
Verification) 2. 형식적 검증 (Formal Verification)으로 리뷰하고 개선하도록 내
프로젝트의 매커니즘을 근본적인 해결책으로 개선해줘.」 — replace adversarial review
with empirical and formal verification, as a fundamental fix to the mechanism.

## Problem

A change lands when a reviewer agent's opinion — approve, approve with nits, do
not approve — is written into the change's record at the diff's fingerprint. The
opinion is the evidence. Three things follow, all measured on the 167 reviewed
changes in this repository on 2026-11-04:

- **It cannot be checked.** The gate's own documentation says it establishes
  that a verdict line exists and is fresh, "not that the session ran that
  reviewer rather than writing the line". A verdict is as strong as the
  session's honesty.
- **It does not converge.** 92 of 167 changes needed two or more rounds; one
  needed 20. A second opinion on the same diff produces new opinions, and every
  edit voids the previous ones. The round ceiling and the "never take a nit"
  rule are patches over this, not a cure.
- **It does not settle.** 517 findings were carried rather than fixed or
  refuted. A carried finding is neither shown wrong nor shown right; it is an
  opinion parked in a file.

An argument between agents ends when one stops; a failing test ends it when it
goes green, and a proof ends it when it checks.

## Who feels it, and when

Every session that lands a change: it spends rounds re-asking reviewers about a
diff whose behaviour did not change, and it reports parked findings to the
person that nobody established were real. The person feels it as changes that
take many cycles to land, and as a list of unconfirmed worries at the end of
each one.

## Desired outcome

- A change lands because a machine re-ran its evidence on the exact diff and the
  evidence held — not because a line says someone approved it.
- Every claim a change makes about its behaviour is backed by one of two kinds
  of evidence: empirical (a test, a mutation sweep, a measurement, a
  reproduction that can be re-run) or formal (an exhaustive check of every state
  of a finite model, or a machine-checked proof).
- A worry about a change is either turned into a failing reproduction — and then
  it is a defect that must be fixed — or it is not a finding at all. Nothing is
  parked as "unconfirmed".
- The number of review rounds stops being a thing a session manages.

## Constraints this change inherits

- The judge of a commit is `main`'s committed gate, not this branch's. This
  change is itself judged by the opinion-based gate it replaces, and must keep
  every table that judge reads readable to it.
- History judged before this change must still be judged by the rule it was
  made under.
- Local-first: no service that sends the code anywhere.
- Gate configuration is a `paused` surface: removing a check is a person's call.

## Systems likely affected

The review gate and its record shape, the CI workflow, the reviewer agent
definitions, the review policy, the pipeline documents that describe stage 5,
the change templates, and the suite that pins all of these.

## Open questions

- Which formal tool — answered by default (the person did not reply within ten
  minutes): exhaustive finite-state model checking in the existing test suite
  first, and a measured feasibility probe of a bounded model checker for Rust as
  a follow-up. Recorded in `state.yaml`.
- Whether agents keep a role — answered by default: yes, as verifiers that write
  falsification experiments and proof harnesses, never as judges.

## Not in scope

- Writing the formal models for specific modules (the hook state machine, store
  migration). This change makes them required and checkable; the first models
  are follow-up changes.
- Removing the old reviewer definitions. They must exist for this change to be
  judged by `main`'s gate; their removal is a follow-up once this has landed.
