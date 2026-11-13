# Plan: the 新しいセッション sheet adds a project in place

- **Spec**: `./spec.md`
- **Approved**: 2026-11-13
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/app/launch_projects.rs` | new: registration without navigation, sheet adoption (picker, drop), choices + filter, header picker, no-project body, `switch_launch_project` (moved) |
| `src/app.rs` | `mod launch_projects;`; `try_add_project_path` delegates; `open_new_session` opens the sheet; drop routing and overlay text; `launch_project_filter` field |
| `src/app/screens.rs` | `ui_launch_sheet` draws the picker / no-project body from the new module; `switch_launch_project` moves out |
| `src/i18n_tables.rs` | ten new rows per table; 「先にプロジェクトを追加してください。」 rows removed |
| `src/tests.rs` | eight `launch_294_` tests; three tests rewritten for the no-project sheet |
| `CLAUDE.md` | map line for the module; two lines trimmed to hold `always_loaded_bytes` |

## Order of work

1. Write the eight tests and rewrite the three; watch them fail (compile errors
   count for the new symbols; the rewritten ones fail on behaviour).
2. Module with `register_project_folder`; `try_add_project_path` delegates
   (existing project-add tests stay green).
3. Adoption, choices, filter, picker, no-project body; wire into
   `ui_launch_sheet`, `open_new_session`, `update`.
4. Strings in all three tables; `CLAUDE.md` map line.
5. `make q.fast`; `cargo test --locked launch_294_` and the rewritten and
   neighbouring sheet / project tests; check-bands.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Filter click closes the combo | cannot type a filter | `CloseOnClickOutside`; manual check |
| Row click no longer closes the combo | popup stays after a choice | explicit `close_popup`; R1 test sees the sheet switch |
| Sheet with no project closes itself on the first frame (old early return) | 新しいセッション does nothing | R5 renders two frames and asserts it stays open |
| Moving registration changes project-page add | duplicate notice / page lost | existing `add_project_path` tests |
| Picker called inside the draw closure | nested modal during layout | called after the `Area`; review |
| Module/doc bands | propose at 10000 lines / 8000 bytes | check-bands.sh |

## Proof of completion

- `make q.fast` — passes.
- `cargo test --locked launch_294_` — 8 passed.
- `cargo test --locked sheet` and the rewritten tests — passed.
- `bash scripts/check-bands.sh` — no metric at propose.
- In the app: 新しいセッション → 「＋ 新しいプロジェクト…」 → pick → the sheet shows it.

## Departures from the plan
