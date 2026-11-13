# Plan: ⌘-click opens a terminal path in Operon; right-click offers where to open it; bare names resolve

- **Spec**: `./spec.md`
- **Approved**: 2026-11-13
- **Status**: done

## Files that change

| File | Change |
|---|---|
| `src/files.rs` | `unique_scanned_file(scan, name) -> Option<&Path>` |
| `src/app.rs` | `terminal_path_menu_at` field; `mod terminal_paths;`; probe drain and `resolve_session_terminal_file` call `resolve_terminal_path_with_scan`; drain requests a missing scan; `Files`/`WorktreeFiles` handlers call `forget_terminal_path_misses` |
| module `app::terminal_paths` (new) | `ui_terminal_path_menu` moved from `screens.rs`; `TERMINAL_PATH_MENU`, `TerminalPathMenuChoice`, `apply_terminal_path_menu_choice`, `terminal_path_menu_closes`, `resolve_terminal_path_with_scan`, `forget_terminal_path_misses` |
| `src/app/screens.rs` | click dispatch (⌘/⌥ → `open_terminal_target`), secondary click stores pointer; menu moved out; `is_terminal_cmd_modifier` stays (used by `is_terminal_open_modifier` and its tests) |
| `CLAUDE.md` | map line for module `app::terminal_paths` |
| `src/i18n_tables.rs` | 「Operon で開く」 in EN and KO tables |
| `src/tests.rs` | nine `terminal_293_` tests (R1 1, R2 2, R3 1, R4 2, R5 2, R6 1); update tests that pinned ⌘ → external |
| `docs/MANUAL.md`, `docs/MANUAL.ja.md`, `docs/MANUAL.ko.md` | ⌘/⌥ open in editor, right-click menu, bare names |

## Order of work

1. Tests for R1–R6 written against the new symbols; watch them fail to compile/fail.
2. `unique_scanned_file` and `resolve_terminal_path_with_scan`; wire both callers (R4); scan request from the drain and miss-forgetting in both scan handlers (R5).
3. Move `ui_terminal_path_menu` to the new module; menu constant, choice enum, `apply_terminal_path_menu_choice`, `terminal_path_menu_closes`, drawing at the pointer (R2, R3, R6).
4. Click dispatch (R1); update tests that pinned ⌘ → external.
5. Strings, manuals.
6. `make q.fast`, `cargo test --locked terminal_` (named filter), screenshot check if the app can be run.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Bare-name lookup in the draw path | frame stutter | lookup only in probe drain / click handler; rust-reviewer |
| Menu never closes | stuck window | `terminal_293_menu_closes_on_escape_or_outside_click`; existing removal tests |
| Bare name never resolves when the files tab was not opened | permanent miss | R5 tests: drain requests the scan; scan landing forgets misses |
| Bare name matches the wrong file | wrong tab opens | unique-match only; two-match test |
| Old tests asserting ⌘ → external | CI red | updated in step 4 |
| `screens.rs` crosses 10000 lines | bands propose | menu moved out; `check-bands.sh` |

## Proof of completion

- `make q.fast` — passes.
- `cargo test --locked terminal_293_` — 9 passed.
- CI `cargo test --locked` green on the branch.

## Departures from the plan

- Review (rust-reviewer): a bare name resolved through the scan now also has to
  stay inside the root, like every other resolution, so a symlink out of it is
  never underlined (`terminal_293_bare_name_symlinked_outside_the_root_stays_unresolved`).
- Empirical verifier, reproduced and fixed: F1 — the drain asked for a project
  scan even when the bare name had already resolved at the root; it now asks
  only after a miss. F2 — a truncated scan was taken as proof that a name was
  unique; a truncated scan now resolves no bare name. Both are tests and
  mutations in this change.
