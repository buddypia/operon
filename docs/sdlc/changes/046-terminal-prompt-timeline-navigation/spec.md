# Spec: Terminal Prompt Timeline Navigation

- **Intent**: ./intent.md
- **Status**: approved

## Requirements

1. Operon tracks user prompt submissions per session, storing for each turn: turn index, prompt text, timestamp, activity status (working or completed), and starting terminal line number.
2. The terminal session view renders a collapsible right-side panel titled "会話履歴" (Conversation History) displaying the turn list.
3. Each turn item shows its sequential number (#1, #2...), relative time (e.g. "3分前" or formatted timestamp), prompt excerpt (truncated to 2 lines), and status badge (実行中 / 完了).
4. Clicking a turn card triggers `scroll_to: Some(line)` on the terminal pane, causing `egui::ScrollArea` to smoothly position the prompt row within view and apply a momentary jump highlight.
5. The panel header provides a "↓ 最新" (Scroll to Bottom) button that immediately jumps to the end of the terminal buffer and marks the active turn.
6. A toggle button in the session header/toolbar allows hiding or expanding the right-side timeline panel so the terminal can take the full width when desired.
7. All user-facing strings are in Japanese and registered in `src/i18n_tables.rs`.

## Behaviour

- **Initial state (no turns)**: When a session starts before any prompt is submitted, the right panel displays an empty hint "プロンプトの入力を待っています…" (Waiting for prompt input...).
- **During a turn**: When `UserPromptSubmit` or CLI execution begins, turn #N appears with status "実行中" (Working). Clicking it scrolls the terminal to the prompt line.
- **Completed turn**: When the CLI stops or reaches `Idle`, turn #N updates to "✓ 完了" (Completed).
- **Navigation click**: Clicking any card immediately sets `scroll_to` to the recorded line index. The terminal viewport jumps to center the prompt line, and a border/selection highlight is painted over the row for visual confirmation.
- **Collapsed panel**: When the user clicks the panel toggle button, the right panel is hidden and the terminal expands to fill 100% of the available workspace width. The toggle state persists during the session.
- **Scrollback eviction**: If terminal output exceeds tmux scrollback limits and the line is no longer available in memory, Operon smoothly scrolls to the earliest available line rather than crashing or overflowing.

## Design

- `src/app.rs`:
  - Introduce `struct PromptTurn { pub turn: usize, pub prompt: String, pub timestamp: Instant, pub line: usize, pub completed: bool }`.
  - Add `pub(crate) session_prompt_turns: HashMap<Uuid, Vec<PromptTurn>>` and `pub(crate) show_prompt_timeline: bool` to `OperonApp`.
  - When handling `HookReading` or `UserPromptSubmit` in hook connection / poll routines, capture prompt text and record the current length of `terminal_layouts` as the turn starting line.
- `src/app/screens.rs`:
  - Update `ui_session` layout: allocate available horizontal space between the terminal container and the right timeline panel if `show_prompt_timeline` is enabled.
  - Draw `ui_prompt_timeline` rendering card items with hover styles, active selection border, status indicators, and click handling.
- `src/ui/terminal.rs`:
  - Enhance `TerminalSearchOverlay` or add highlight drawing for the jumped prompt line when requested.
- `src/i18n_tables.rs`:
  - Register all Japanese strings (`会話履歴`, `最新へ移動`, `プロンプトの入力を待っています…`, `会話パネル切替`, etc.) across ja, en, and ko tables.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | Applies | Uses existing `Palette` semantic roles (`card`, `raised`, `accent`, `text_strong`, `text_muted`, `success`, `border_subtle`) without hardcoded hex literals. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | Applies | Reuses existing glyphs `ICON_CHAT`, `ICON_MORE`, `ICON_CHECK` or standard Phosphor icons registered in glyphs. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | Does not apply | No external configuration keys or environment cross-references are introduced. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | Does not apply | Prompt turns and timeline visibility are session-scoped runtime state held in memory, avoiding store schema changes. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | Does not apply | No new subprocesses or commands are spawned; reads from existing hooks and tmux buffers. |
| Documentation — user-facing docs change in all three languages together | Does not apply | No top-level user guide changes are required for this UI convenience feature. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Applies | 100% local in-memory operation, no external network calls or unsafe blocks. |
| Budgets — any new scan or output path states its byte and item ceiling | Applies | Turns per session are capped at 500 items in memory; prompt excerpts are bounded to 200 characters. |

## Flagged concerns

- **tmux scrollback overflow and line offset drift** — Long-running sessions may drop early lines when tmux scrollback wraps. This is resolved by bounding the jump target to `line.min(total_lines.saturating_sub(1))` and falling back gracefully to the top of the buffer if the line has expired.
- **Prompt truncation and multi-line formatting** — User prompts can contain code blocks or thousands of characters. This is resolved by taking only the first 200 characters of the first line for the timeline summary card while preserving readability.

## Acceptance

- `cargo test --locked` passes, including new tests for prompt turn tracking and line jumping in `src/tests.rs`.
- `cargo clippy --locked -- -D warnings` and `cargo fmt --check` pass without errors.
- Opening a session with multiple turns displays the right-side timeline panel with accurate prompt excerpts.
- Clicking any turn card smoothly scrolls the terminal viewport to the exact prompt line.
- Toggling the timeline button hides and reveals the side panel without layout distortion.

## Rejected alternatives

- Sticky single-line bar above terminal alone was rejected because it only shows one prompt at a time and lacks overall session context navigation.
- Floating modal dialog for history was rejected because it obscures the terminal output while reading.
