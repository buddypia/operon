# Plan: Tab keys a Mac user brings with them

- **Spec**: `./spec.md`
- **Approved**: 2026-11-09
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/app/keymap.rs` | five `KeymapAction` rows; `Keymap::action_for(key, modifiers)` with the shifted-bracket key match and the exact-over-lenient preference |
| `src/app.rs` | `closed_tabs` field; `chord_pressed` / `chord_consumed` through `action_for`; tab dispatch in `handle_app_shortcuts`; `step_workspace_tab`, `reopen_closed_tab`; push in `close_workspace_tab` |
| `src/app/screens.rs` | remove the hard-coded ⌘W and ⌃Tab blocks from `ui_tab_workspace`; two palette rows and their dispatch |
| `src/config.rs` | `CLOSED_TAB_LIMIT`; the `KEYMAP_FILE_MAX_BYTES` comment stops counting actions |
| `src/i18n_tables.rs` | 「閉じたタブを開き直す」「次のタブへ」「前のタブへ」「開き直せるタブはありません。」 in EN and KO |
| `docs/MANUAL.md`, `docs/MANUAL.ja.md`, `docs/MANUAL.ko.md` | §8 rows and fixed-key sentence; §4 tab paragraph |
| `src/tests.rs` | the seven `shortcut_272_` tests |

## Order of work

1. Tests for requirements 2 and 5's key matching against `Keymap` alone
   (`shortcut_272_the_most_specific_chord_wins`, the bracket half of
   `shortcut_272_brackets_and_control_tab_step_through_tabs`): they fail to
   compile until `action_for` and the rows exist — watch them fail, then add the
   rows and `action_for`.
2. App-level tests for requirements 1, 3, 4, 5, 6 driving `handle_app_shortcuts`
   inside `ctx.run` with injected `Event::Key`s; watch them fail (⌘T no-op,
   ⇧⌘T no-op, ⌃⇧Tab forward, no palette rows).
3. `src/app.rs`: `chord_pressed` / `chord_consumed` via `action_for`;
   `closed_tabs` + `CLOSED_TAB_LIMIT`; `step_workspace_tab`, `reopen_closed_tab`;
   dispatch; then delete the screens.rs blocks in the same step so ⌘W is never
   handled twice. Compiles after this step.
4. Palette rows and dispatch; i18n rows.
5. Manual test (requirement 7), watched failing, then the three manuals.
6. Tidy; `make q.fast`; `cargo test --locked shortcut_272_` and the three named
   existing tests; mutation sweep; `verification.yaml`.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| ⌘W handled both in `handle_app_shortcuts` and a leftover screens.rs block, closing two tabs | one press closes two tabs | `shortcut_272_close_is_a_keymap_action` counts tabs after one press through a full workspace frame |
| The exact-over-lenient rule changes an existing chord | ⇧⌘K stops opening the palette | `shortcut_272_the_most_specific_chord_wins` asserts ⇧⌘K still opens it |
| `}` from ⇧⌘] not matched on a real keyboard | ⇧⌘] does nothing in the app | test injects `CloseCurlyBracket`; manual check in the packaged app |
| `step_workspace_tab` on an empty group divides by zero | panic, window lost | the empty-group guard; test steps with no tab |
| Reopen of a removed session or a document of a removed project | a tab for nothing, or a panic on lookup | `shortcut_272_reopen_brings_back_the_last_closed_tab` removes one before reopening |
| A refused close (unsaved document) remembered | ⇧⌘T "reopens" a tab still open | same test, unsaved-document case |
| A keymap file that already uses ⌘T or ⇧⌘T for another action | the file is refused whole after updating | existing duplicate sentence; noted in the manual's keymap paragraph is not needed — the sentence names both actions |

## Proof of completion

- `make q.fast` — passes (fmt, clippy `-D warnings`, harness checks).
- `cargo test --locked shortcut_272_` — `test result: ok. 7 passed; 0 failed`.
- `cargo test --locked the_keys_section_lists_every_action`,
  `cargo test --locked the_palette_draws_the_chord_the_handler_listens_for`,
  `cargo test --locked every_default_chord_survives_being_written_and_read_back`,
  `cargo test --locked no_message_spells_a_remappable_chord` — each `ok`.
- `python3 docs/sdlc/changes/272-tab-shortcuts-are-missing-or-hidden/mutations.py` — every mutation caught.
- In the packaged app: ⌘T adds a terminal tab; ⌘W then ⇧⌘T brings it back;
  ⇧⌘] / ⇧⌘[ step; Settings → キー割り当て lists the five rows.

## Departures from the plan

None yet.
