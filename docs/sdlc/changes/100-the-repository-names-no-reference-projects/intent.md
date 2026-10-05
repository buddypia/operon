# Documentation and change records maintain clean self-containment

## Problem

Comments, documents and change records contained redundant historical notes. Such notes are not behaviour, go stale without anyone noticing, and the owner asked for all documentation to be strictly self-contained.

## Proposed Change

- Clean up external references and roadmaps under `docs/`.
- Reword comments, docstrings, descriptions and runtime messages in `src/`,
  `.cli/`, `.claude/scripts/`, `.claude/config/`, `.claude/skills/` and
  `docs/sdlc/` so they state what the code does clearly and self-containedly.
- Add `steering_and_source_do_not_name_external_projects`, which
  fails when a steering document or a `src/` file names excluded external
  identifiers again.

A follow-up cleaned up remaining identifiers, file names and gate-configuration files,
and widened the regression test to ensure none return.

## Risk and Review

Touches `src/git.rs`, a `subprocess` surface, by comments only. A
subprocess-safety review confirms that no behaviour moved.
