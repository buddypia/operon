# Intent: the English and Korean screens show Japanese

- **Status**: draft — needs the translation-sourcing decision below
- **Opened**: 2026-09-04

## Problem

The app ships three languages, but all but two strings are translated only
in the sense that they fall back. Every English or Korean screen today reads
as a Japanese screen with an English or Korean frame around it: 536 of the
538 message ids used at call sites have no row in any translation table.

## Who feels it, and when

Anyone who picks English or Korean in the language picker, on every screen,
from the first launch on. The fallback is silent — nothing says a string is
untranslated, so a reader cannot tell a missing translation from a decision
to leave the Japanese.

## Desired outcome

Picking English or Korean shows an English or Korean screen: every message
id has a row in every table, placeholders survive into both languages, and
the suite asserts the coverage rather than the tables' consistency with each
other.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- User-facing text is Japanese; code, comments, and docs are English.
- The message id is the Japanese source string; translation never renames it.
- Tables are hand-edited, sorted by message id; the drift tests assert the
  order, the per-language coverage, and the placeholder names.
- 95 of the missing ids are templates with named placeholders; a translation
  that renames, drops, or invents one renders the placeholder as literal
  text.

## Systems likely affected

A guess, to be corrected by `spec.md`. The per-language message tables and
the inline suite.

## Open questions

- Who writes 1 072 translations, and what quality bar do they meet? Machine
  translation reviewed by a reader of each language, the owner writing them,
  or staged batches (first-run screens first) are all cheaper than one
  unreviewed bulk commit. Answered by the owner of change 002's migration.
- Does the suite gain a call-site coverage test (every `tr` id has a row),
  closing the hole the current guards leave? Proposed yes; it is what turns
  this from a bulk edit into a guarded property.

## Not in scope

The translation mechanism itself, the fallback behaviour, and the two rows
already present. Change 002's remaining migration questions.
