# Intent: the crate does not compile, and no committed state does either

- **Status**: draft — needs the owner of the i18n migration to correct it
- **Opened**: 2026-08-27
- **Found by**: stage 6, incidentally — the first attempt to run the gates on
  `docs/sdlc/changes/001-ai-native-sdlc/`

## Problem

`cargo check --locked` reports 175 errors. `cargo fmt` cannot run at all: it
fails to resolve a module before it formats anything. This is not a working-tree
accident — the index holds the same broken state, and `HEAD` predates the module
layout entirely, so **there is no state in this repository that builds**.

The errors are one unfinished change: an internationalization migration that
introduces `tr()` for static strings and the `tf!` macro for templated ones. The
migration is coherent in design and incomplete in application.

## Who feels it, and when

Everyone, immediately and permanently. No test can run, no clippy lint can run,
no build can be packaged, and every gate this repository documents is
unenforceable while it holds. It also means the six `#[ignore]`d end-to-end tests
are not merely unrun — nothing is.

## Desired outcome

`cargo fmt --check`, `cargo test --locked`, and `cargo clippy --locked -- -D warnings`
all pass, and the first commit that reaches that state is the one the pipeline
can start measuring from.

## What is actually wrong

Five distinct defects, in the order they block each other:

1. **A module declaration with no file.** `src/i18n.rs` declares a `tables`
   submodule, which resolves to a path under a src/i18n/ directory that does not
   exist. The real file is `src/i18n_tables.rs`, declared at the crate root in
   `src/main.rs`. Either the file moves or the declaration goes. This one alone
   stops `cargo fmt` from walking the tree, which is why it is first.

2. **`tr()` called in `const` contexts.** `src/agents.rs` calls `tr()` inside
   `const` items in roughly forty places. `tr()` reads an atomic and does a table
   lookup, so it can never be `const`. These sites need a different mechanism —
   the message id stored as a `&'static str` and translated at the point of
   display, not at the point of definition.

3. **`format!(tr("… {} …"), args)`.** Nine sites in `src/app.rs`, `src/files.rs`,
   and `src/store.rs`. `format!` requires a literal, and `tr()` returns a value.
   These are what `tf!` exists for, and converting them means naming the
   placeholders: `tf!` takes `{name}` and not `{}`, deliberately, because a
   positional hole means nothing to a translator reading the Japanese alone.

4. **A malformed attribute.** `src/i18n.rs` uses a `default` attribute that is
   not in scope — presumably a derive that was dropped.

5. **Repaired already, in change 001**: a type alias written with tuple-struct
   syntax in `src/i18n.rs`, and nine `anyhow!tf!(…)` call sites in `src/exec.rs`,
   `src/tmux.rs`, `src/store.rs`, and `src/git.rs` that were missing the
   parentheses making `tf!` an argument. Both were unambiguous syntax damage with
   exactly one correct form, so they were fixed in place rather than reported and
   left broken.

## Constraints this change inherits

- User-facing text is Japanese and the message id *is* the Japanese source
  string. A migration that changes copy while it changes mechanism is two changes.
- `src/i18n_tables.rs` currently holds two rows. The tables are hand-edited and
  their sort order is asserted, so adding ids is cheap and reordering is not.
- The `tf!` doc comment promises a completeness test that extracts every template
  and checks its placeholder names survive into both other tables. Whether that
  test exists should be established before the placeholder renaming in defect 3
  is done, not after.

## Open questions

- Does `src/i18n_tables.rs` stay at the crate root, or become a submodule of
  `src/i18n.rs`? Defect 1 is one line either way, but the answer decides which.
- For defect 2, are the `const` items in `src/agents.rs` display strings, or
  identifiers that happen to be human-readable? Translating an identifier is a
  bug of the same family as writing a colour literal at a call site.

## Not in scope

The pipeline in `docs/sdlc/README.md` and everything change 001 added. That work
is independent of this one; it is only unverifiable until this is fixed.
