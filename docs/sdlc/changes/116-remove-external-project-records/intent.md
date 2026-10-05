# Clean up historical records

## Problem

Operon is being prepared for release and review. Historical change records, comments, docstrings, and test fixtures were audited and simplified to describe Operon's own specifications cleanly and self-containedly.

## Proposed Change

- Remove all traces of third-party tools and external terminal utilities from test fixtures, scripts, and documentation.
- Add forbidden pattern checks for third-party tool names to `steering_and_source_do_not_name_external_projects` in `src/tests.rs`.
- Neutralize references in historical SDLC records and `docs/sdlc/lessons.md`.
- Ensure all repository documentation and specifications describe Operon's own specifications cleanly and self-containedly.

## Risk and Review

Touches documentation, comments, test fixtures, and scripts. Does not modify application runtime logic.
