# Plan: Event-driven, damage-tracked terminal TUI pipeline for session panes

- **Spec**: `./spec.md`
- **Approved**: 2026-11-13
- **Status**: done

This is the plan produced in plan mode and accepted before any file was edited.
If the implementation departs from it, update this file — an abandoned plan is
worse than no plan, because the next reader trusts it.

## Files that change

| File | Change |
|---|---|
| `src/config.rs` | Add `TERMINAL_STREAM_FRAME_BUDGET` (`16ms`), `TERMINAL_STREAM_UNFOCUSED_BUDGET` (`250ms`), `MAX_CACHED_TERMINAL_ROW_GALLEYS` (`2048`), and `TERMINAL_NEGATIVE_PATH_TTL` (`2s`). |
| `src/exec.rs` | Add `StreamingChild` and `spawn_piped_streaming(command: &mut Command) -> Result<StreamingChild>` stripping `INHERITED_REPOSITORY_POINTERS`, configuring piped `stdin`/`stdout` and null `stderr`, and implementing safe `shutdown()` (`stdin` close, `try_wait` up to `25ms`, then `Child::kill()` + `Child::wait()`) and `Drop` with zero `unsafe`. |
| `src/tmux.rs` | Keep existing single-process `tmux_capture_with_cursor` intact; add `tmux_spawn_output_watcher` built via `tmux_command()` + `is_safe_tmux_name` (`-C attach-session -t <name> -f read-only,ignore-size`) and bounded `O(1)` control-line chunk scanner `tmux_control_chunk_has_activity`. |
| `src/ui/terminal.rs` | Add `IncrementalTerminalLayout` / `terminal_line_layouts_incremental` (per-line `\r` stripping without whole-buffer copy, exact `(theme, style_in, raw)` line reuse across in-place edits and scrollback shifts, monotonic `line_id`s, and dirty-line tracking), bounded row-level `Galley` cache inside `TerminalPaneRenderCache` (`row_galley_builds` / `row_galley_hits`), binary-searched `search.matches`, and single-pass `hover_pos()` link check per pane frame. |
| `src/app/terminal_output.rs` | Add `TerminalStreamManager` (per-visible-pane event-driven coalesced capture via `tmux_capture_with_cursor`, `SESSION_POLL_INTERVAL` respawn cooldown, `is_watching` liveness flag, zero-repaint on unchanged `(theme, fingerprint, cursor)`, cursor-only fast path, incremental background layout), dirty-line-aware negative path-cache TTL pruning, and relocate terminal input/echo/resize helpers from `src/app.rs`. |
| `src/app/screens.rs` | Remove per-frame `self.store.projects.clone()` in `ui_workspace_projects`, avoid double tab-title `Galley` shaping in `ui_tab_workspace`, and pass `line_ids` into `TerminalPaneContext` while keeping `src/app/screens.rs` under `10000` lines. |
| `src/app.rs` | Wire `TerminalStreamManager` (`sync_terminal_streams`, `drain_terminal_streams`, `on_exit` shutdown), skip duplicate `on_screen` captures in `request_session_poll` and speculative post-`send-keys` capture when `is_watching` is true, store and clean up `terminal_incremental_layouts` / `terminal_line_ids` / bounded `negative_path_probed_at` across `apply_appearance`, session switches, and `remove_session_record`, and remove relocated terminal methods so `src/app.rs` shrinks below `9979` lines. |
| `src/tests.rs` | Add failing-first empirical tests `tui_perf_292_*` for R1–R6 (using `tmux_command()`'s `operon_suite` socket) and exhaustive `formal-model` test `model_check_tui_292_incremental_ansi_and_coalescer_equivalence` via `exhaustive_check`. |

## Order of work

1. **Constants & subprocess primitives (`src/config.rs`, `src/exec.rs`, `src/tmux.rs`)**:
   - Add the four constants in `src/config.rs`.
   - Implement `StreamingChild` and `spawn_piped_streaming(command: &mut Command)` in `src/exec.rs` stripping `INHERITED_REPOSITORY_POINTERS` so all child spawning stays inside `src/exec.rs` (`unwrapped_spawns` stays at `2`, `unsafe_blocks` stays at `3`).
   - Implement `tmux_spawn_output_watcher` (via `tmux_command()` and `is_safe_tmux_name`) and `tmux_control_chunk_has_activity` in `src/tmux.rs`, keeping `tmux_capture_with_cursor` intact. Tree compiles cleanly.
2. **Incremental ANSI layout & row-level `Galley` cache (`src/ui/terminal.rs`)**:
   - Implement `CachedTerminalLine`, `IncrementalTerminalLayout`, and `terminal_line_layouts_incremental` with exact `(theme, style_in, raw)` matching, scrollback shift detection, dirty-line reporting, and `#[cfg(test)]` parsed-line counters.
   - Extend `TerminalPaneRenderCache` with bounded `row_galleys` (`MAX_CACHED_TERMINAL_ROW_GALLEYS`), single `ctx.fonts` lock per miss, `row_galley_builds()` / `row_galley_hits()` counters, single `hover_pos()` query per pane frame, and `partition_point` lookup on `search.matches`. Tree compiles cleanly.
3. **Stream manager, dirty-line-aware negative-path TTL, & workspace draw-path cleanup (`src/app/terminal_output.rs`, `src/app/screens.rs`, `src/app.rs`)**:
   - Relocate terminal input, echo, drag, and resize methods from `src/app.rs` into `src/app/terminal_output.rs`.
   - Implement `TerminalStreamManager` (with `is_watching` liveness and `2s` respawn cooldown) and pure `stream_coalesce_delay`, wire `sync_terminal_streams` + `drain_terminal_streams` into `OperonApp::update` and `on_exit`, suppress redundant watched captures in `request_session_poll` / `poll_terminal_echo` / `TerminalInputSent`, clean up incremental/line-id/path-timestamp state in `apply_appearance` and `remove_session_record`, and retain timestamped negative `resolved_paths` entries within `TERMINAL_NEGATIVE_PATH_TTL` unless mentioned on a dirty line.
   - Remove per-frame `projects.clone()` in `ui_workspace_projects` and reuse tab-title `Galley`s in `ui_tab_workspace`. Tree compiles cleanly.
4. **Tests, model check, mutation sweep, & band verification (`src/tests.rs`)**:
   - Add `tui_perf_292_*` (R1–R6, driving `tmux_command()`'s `operon_suite` socket) and `model_check_tui_292_incremental_ansi_and_coalescer_equivalence`.
   - Run `make q.fast`, named test suites (including `test_set_session_output_*`, `terminal_276_*`, `adversarial_*`, `every_tmux_the_crate_runs_goes_through_tmux_command`), `mutations.py`, `bash scripts/check-bands.sh`, `bash scripts/check-review.sh --index`, and independent verifiers.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Orphaned `tmux -C` client or respawn storm on exited/missing session | Extra client in `tmux list-clients` or repeated `tmux -C` spawns per frame | `StreamingChild::Drop` + `on_exit` + `SESSION_POLL_INTERVAL` respawn cooldown + `tui_perf_292_control_stream_coalesces_bursts_and_suppresses_blind_echo_polls` |
| Multi-line ANSI colour state drift or stale theme colours on incremental line reuse | Line `k + 1` keeps old colour or theme switch fails to re-colour | `(theme, style_in, raw)` predicate + `apply_appearance` cache clear + `terminal_276_*` + `model_check_tui_292_incremental_ansi_and_coalescer_equivalence` |
| Breaking dynamic-file negative-cache eviction or existing `TerminalPaneRenderCache` / `BackgroundKey` tests | Failure in `test_set_session_output_evicts_negative_cache_for_dynamic_files`, `adversarial_*`, or `empirical_stress_test_all_55_*` | Dirty-line-aware negative eviction + running all existing path/cache/wake tests alongside `tui_perf_292_*` |
| `src/app.rs` or `src/app/screens.rs` exceeding `10000`-line `propose` band | `bash scripts/check-bands.sh` reports `propose` | Moving terminal methods into `src/app/terminal_output.rs`, compact edits in `src/app/screens.rs`, and asserting module line budgets in `tui_perf_292_session_workspace_steady_frame_and_module_budget` |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. N passed; 0 failed; 6 ignored`.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `cargo test --locked tui_perf_292_` — all 6 requirement tests pass.
- `cargo test --locked model_check_tui_292_incremental_ansi_and_coalescer_equivalence` — passes exhaustive state walk.
- `python3 docs/sdlc/changes/292-terminal-tui-rerenders-and-stalls/mutations.py` — catches every mutation.
- `bash scripts/check-bands.sh` — 0 `propose` breaches, `unsafe_blocks` at `3`, `unwrapped_spawns` at `2`, `src/app.rs` < `9979` lines, `src/app/screens.rs` < `10000` lines.
- `bash scripts/check-review.sh --index` — exits `0`.
- In `/Applications/Operon.app`, typing and running TUIs in `Page::Sessions` updates at 60Hz when active and performs zero captures/repaints when idle.

## Departures from the plan

- **Control-mode client flags (`-f ignore-size` without `read-only`)**: On `tmux 3.8`, attaching with `-f read-only,ignore-size` marks the session's sole client `CLIENT_READONLY`, causing `tmux send-keys` (`tmux_send_input`) to fail with `client is read-only`. Passing `-f ignore-size` while never writing to `StreamingChild`'s `stdin` (closing `stdin` only on detach) preserves exact window geometry and clean detach while allowing `tmux_send_input` to type normally. `spec.md` was updated accordingly and re-approved.
- **Testable pure helpers (`tmux_output_watcher_args`, `classify_captured_snapshot`) and first-frame hover width**: Extracted `tmux_output_watcher_args` in `src/tmux.rs` and `classify_captured_snapshot` in `src/app/terminal_output.rs` for deterministic unit/model/mutation testing, and bounded `row_rect` width by `.max(pane_width)` in `terminal_rows` (`src/ui/terminal.rs`) so hover link detection works on the first frame before `ScrollArea` caches content width.
- **Existing `unsafe_blocks` count on `main`**: `main` already has `4` `unsafe` blocks (`warn` tier in `check-bands.sh`); this change adds `0` `unsafe` blocks, leaving `unsafe_blocks` at `4`.
