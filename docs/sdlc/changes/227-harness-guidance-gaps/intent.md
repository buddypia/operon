# Intent: A finished change has no compact evidence receipt

- **Status**: approved
- **Opened**: 2026-10-28

## Problem

The user asked: “/Users/a13973/Downloads/ai-harness 이 폴더의 내용을 폴더를 반드시 1개씩 차례대로 읽고 차례대로 내 프로젝트에 분석후에 웬만하면 다 적용해줘.”
The first folder describes a receipt linking an accepted artifact to its policy,
verification, retries, and rollback point. Existing records are separate, so a
person inspecting a finished change must reconstruct those links.

## Who feels it, and when

A person or resumed agent checking how one committed change was produced.

## Desired outcome

One local command emits a reproducible receipt from a named commit, with explicit
unknowns for information that was never captured and no claim of fresh verification.

## Constraints this change inherits

Local-first; no new dependencies or second mutable record of pipeline state.
Read and apply the three supplied folders sequentially.

## Systems likely affected

Existing pipeline reporting scripts, feature-pilot's closing instructions, tests.

## Open questions

None: the user authorized applying relevant ideas; an additive local report is reversible.

## Not in scope

Running agents, collecting transcripts or secrets, or inventing model/cost figures.
Later folders remain unread until this folder's application is verified.
