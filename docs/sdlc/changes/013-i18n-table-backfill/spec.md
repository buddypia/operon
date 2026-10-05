# Spec: backfill the translation tables, priority screens first

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

Numbered, each one independently checkable. A requirement nobody can fail is not
a requirement.

1. Every batch-1 message id (101 ids: top bar, command palette, home,
   first-run project flow, settings, startup dialogs, language names, shared
   widget chrome) has a row in `EN_TABLE` and in `KO_TABLE`.
2. Both tables stay sorted by message id, so the binary search keeps finding
   every row (`every_translation_table_is_sorted_by_message_id`).
3. Every translated row keeps exactly the placeholder names its message id
   declares (`every_translation_keeps_the_placeholder_names_of_its_message_id`).
4. No message id at any call site is renamed: translation never changes copy.
5. The suite stays green throughout; no test is lost.
6. The remaining ~586 ids are left for later batches under this same spec, and
   the last batch adds the call-site coverage test from the Design section.

## Behaviour

What the user sees and does, including the states that are not the happy one:
empty, loading, interrupted, denied, and too large. Japanese strings for anything
a person reads; describe them here so review does not discover them in the diff.

With English active, the top bar, command palette, home, first-run project
screens, settings, and startup dialogs read in English; with Korean active,
in Korean. Japanese screens are unchanged — the message id is the Japanese
copy, so the Japanese path never touches the tables.

Non-batch-1 screens still fall back to Japanese under English/Korean. That is
the current behaviour, unchanged by this batch: a partial backfill must never
be presented as a complete one, so the language picker gains no new promise
and no notice changes.

Placeholders render as values, not as literal `{name}` text, in both
languages — requirement 3 is what makes a renamed hole visible before a
person ever sees it.

## Design

The shape of the solution: which modules gain what, which types change, what
crosses a process boundary. Enough that `plan.md` is a matter of ordering.

Only `src/i18n_tables.rs` changes: new `(id, translation)` rows in the two
tables, inserted in byte order. No type changes, no call-site changes, no
process boundary crossed.

Inventory correction: `intent.md` says 536 of 538 ids, counted with a
single-line pattern. A lexer that understands comments, raw strings, char
literals, and multi-line invocations finds 687 real call-site ids, of which 2
have rows — 685 missing, 224 of them templates with placeholders. The spec
number governs; the intent's number is superseded by it.

Batch 1 is 101 ids (top bar, palette, home, first-run, settings, startup
dialogs, language names, widget chrome). Later batches follow the same shape;
the last one additionally lands the coverage guard: a test that lexes
`src/**/*.rs` (excluding `src/tests.rs` and `src/i18n_tables.rs`) for
`tr("…")` / `tf!("…")` ids and asserts every one is in the catalog. The guard
waits for the last batch because landing it now would fail for the reason it
names — 586 ids still missing — and a red suite is not a guard.

## Policy conformance

Answer each line, or state that it does not apply. These are the policies this
repository enforces with tests, so "does not apply" is a claim the suite checks.

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | No drawing changes; no colour literal added |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No | No glyph added or removed |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | No | Message ids live at the call site by i18n design; translations are data, not identifiers |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | No | `src/store.rs` untouched; no persisted shape changes |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | No | No child process spawned |
| Documentation — user-facing docs change in all three languages together | No | No `README.md` / docs change; translated strings are app copy, not documents |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Yes | No dependency, no network, no `unsafe`; translations are hand-written table rows |
| Budgets — any new scan or output path states its byte and item ceiling | No | No new scan path; the deferred coverage test reads the bounded source tree at test time only |

## Flagged concerns

Things this spec cannot settle on its own. Each needs an answer before stage 3
begins; record the answer inline rather than in a separate thread.

- **Who writes the translations, and to what bar?** — Answered: the working
  session hand-writes batch 1 (101 ids, priority screens), English and Korean,
  keeping every placeholder name. Machine bulk translation was rejected for
  quality; owner-written follow-ups remain possible for later batches. A native
  read-through of batch 1 is recommended but not blocking — the placeholder and
  sort tests, plus English/Korean eyeballing in the running app, are the gates.
- **Does the suite gain call-site coverage now?** — Answered: no, in the last
  batch (see Design). Landing it now fails on the 586 ids still missing, which
  is the test working as designed and the suite going red, both at once.

## Acceptance

How the finished change is judged, as commands and observations:

- `cargo fmt --check` — no output.
- `cargo test --locked` — green, same count (no test added or lost in batch 1).
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- In the running app with English active: top bar, palette, home, first-run,
  and settings read in English; with Korean active, in Korean. Placeholders
  (e.g. the elapsed-seconds extraction notice) render values, not `{name}`.
- `git diff --stat` touches only `src/i18n_tables.rs` plus the change directory.

## Rejected alternatives

One line each, with the reason. This is the section that stops the same argument
from being had twice.

- Machine-translate all 685 at once: unreviewed bulk copy in two languages ships errors a person reads as product quality.
- Leave the fallback and record it as intended: defeats the language picker, which promises three languages.
- Rename message ids while translating: a migration that changes copy while changing mechanism is two changes.
- Land the coverage guard in batch 1: it would fail for the 586 ids still missing, and a red suite is not a guard.
