# Plan: remove the unreachable launch-preset interface

- **Spec**: none (refactor route skips intent and spec; `intent.md` is the problem record)
- **Approved**: 2026-09-04
- **Status**: in progress

This is the plan produced before any source file was edited. Departures go in
the section at the bottom, not by rewriting the sections above.

## Files that change

| File | Change |
|---|---|
| `src/app.rs` | remove dead preset UI + helpers + state + `FilePreview` variant/arm |
| `src/agents.rs` | remove dead preset catalogue + launch-control helpers |
| `src/ui/widgets.rs` | remove dead widgets (`settings_section`, `field_row`, `field_separator` go too: only dead callers) |
| `src/glyphs.rs` | remove `ICON_RULE`, `ICON_TIME`, `ICON_REQUEST`, `ICON_REPLY` + vocabulary rows |
| `src/theme.rs` | remove `STATUS_COLUMN_WIDTH` |
| `src/git.rs` | remove `launch_root_for_session` |
| `src/store.rs` | untouched — `AgentLaunchPreset` records stay, no schema change |
| `src/tests.rs` | remove the tests that pin the removed code, openly (see Risks) |

## Order of work

Numbered, so that the tree compiles between steps wherever it can.

1. `src/theme.rs`, `src/glyphs.rs`, `src/git.rs`: leaf constants/helpers first.
   `cargo check` stays green throughout (removals only widen dead code).
2. `src/ui/widgets.rs`: dead widgets. `cargo check` green.
3. `src/agents.rs`: preset catalogue + control helpers. `cargo check` green.
4. `src/app.rs`: dead UI regions, state fields, `FilePreview` variant and its
   match arm. `cargo check` green — `#[allow(dead_code)]` on the removed items
   means no new warnings appear.
5. `src/tests.rs`: remove the now-uncompilable tests. The tree does NOT compile
   between step 4 and this step; that is the one place it cannot.
6. Full gates: `cargo fmt`, `cargo test --locked`, `cargo clippy --locked -- -D warnings`.
7. Commit with `OPERON_ALLOW_TEST_REMOVAL=1`, naming the behaviour that stopped
   existing (unreachable preset editor + its widgets).

## Risks

What could break that the compiler will not catch. For each one, what makes it
visible.

| Risk | How it shows up | What catches it |
|---|---|---|
| A removed widget was reached from a live path the audit missed | screen loses a control | `cargo test` suite (322 tests draw every live view) + manual launch of the app |
| A removed test pinned live behaviour, not dead code | suite shrinks over live logic | each removed test named in the commit message against the removed item |
| Persisted presets orphaned | old preset records linger unread | accepted: data stays by design; `store.rs` untouched, schema version unchanged |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. N passed; 0 failed; 6 ignored` with N smaller by exactly the removed tests.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `grep -rn allow(dead_code) src/app.rs src/agents.rs src/ui/widgets.rs` — only live-code allows remain, if any.
- `bash scripts/check-bands.sh` — `largest_module_lines` falls with `src/app.rs`.

## Departures from the plan

Filled in during implementation. What changed, and why the plan was wrong.

- Steps 2–4 do not compile independently: `field_row`, `field_separator`,
  and `settings_section` are called from inside the dead regions removed in
  steps 3–4, so step 2 alone breaks the build with 12 `E0425` errors. The
  atomic unit is steps 2+3+4+5 together; compilation is re-verified after
  step 5 instead of between steps.
