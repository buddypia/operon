# Intent: A memory review cannot show exactly which sessions informed it

- **Status**: approved
- **Opened**: 2026-10-28

## Problem

The user asked to read each folder under ai-harness sequentially, analyze it
against Operon and apply the relevant ideas. The third folder's talk recommends
an out-of-band review across sessions, with evidence, prevalence, permissions and
proposed updates rather than silent global memory edits. Operon has per-project
background knowledge and versioned development records, but no bounded, explicit
source batch for reviewing the development harness's memory across changes.

## Who feels it, and when

A maintainer reviewing repeated failures across completed changes before adding
another global instruction or changing a skill.

## Desired outcome

A read-only command supplies only explicitly selected committed change records
at one immutable revision, with content hashes and a clear review-only scope.
An independent review can cite source examples and a denominator, reject stale
or injected instructions, and route any useful proposal through the existing
change/review process. No automatic memory rewrite or extra paid run is implied.

## Constraints this change inherits

Local-first; existing state and review ledgers remain authoritative. No raw
private transcript export, new scheduler, app-store migration or vendor service.
The first two source folders are verified and landed before this folder was read.

## Systems likely affected

Existing change receipt reporting and sdlc maintenance references.

## Open questions

None: this is a reversible extension within the user's requested adoption.

## Not in scope

Replacing SQLite with Markdown, enterprise ACLs, or a new agent runtime.
