# Screen: Session workspace terminal pane (`Page::Sessions`)

- **Spec**: `./spec.md`
- **Status**: approved

## Overview

This change alters no visual chrome, colours, typography, spacing, or Japanese strings on `Page::Sessions`. `docs/sdlc/risk.yaml` marks `drawing` (`src/app.rs`, `src/app/screens.rs`, `src/ui/terminal.rs`) as `screen: yes`; this layout records the preserved visual structure across normal, split, empty/loading, error, and long/flooded states, alongside the frame-update behaviour before and after.

## Active TUI Session (1280px window, sidebar 268px + workspace 1012px)

### Visual layout (unchanged before and after)

```text
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│ Operon   ホーム   セッション   プロジェクト   ナレッジ   設定                       ⌘K コマンド │
├────────────────────────┬─────────────────────────────────────────────────────────────────────────┤
│ セッション         [+] │ [● Claude: API 修正 ×] [○ Codex: テスト ×]                    [分割 ⫿] │
│ [すべて][Claude][Codex]├─────────────────────────────────────────────────────────────────────────┤
│                        │ ╭─ Claude Code v2.x ──────────────────────────────────────────────────╮ │
│ ▾ 要対応 (0)           │ │ > src/ui/terminal.rs の再レンダリングを抑止してください             │ │
│                        │ │                                                                     │ │
│ ▾ 実行中 (1)           │ │ ⠋ Reading src/ui/terminal.rs (1,507 lines)…                        │ │
│ ● API 修正      12s    │ │   ✓ Checked TerminalPaneRenderCache                                 │ │
│   operon · main        │ │   ✓ Checked tmux control-mode stream                                │ │
│                        │ │                                                                     │ │
│ ▾ アイドル (1)         │ │ ─────────────────────────────────────────────────────────────────── │ │
│ ○ テスト               │ │ > ▊                                                                 │ │
│   operon · feat/test   │ │   ⏎ 送信 · Esc 中断                                                 │ │
│                        │ ╰─────────────────────────────────────────────────────────────────────╯ │
└────────────────────────┴─────────────────────────────────────────────────────────────────────────┘
```

### Frame & rendering behaviour across states

```text
State                              Before                                    After
─────────────────────────────────  ────────────────────────────────────────  ────────────────────────────────────────
Idle TUI (no output, no input)     2s poll spawns 2 tmux procs + wakes UI    0 stream captures, 0 unchanged wakeups,
                                                                             0 ANSI parses, 0 Galley builds
1-line spinner tick (5,000 lines)  Updates every 2s (0.5 FPS); copies full   Updates in ~16ms (60 FPS); parses 1
                                   buffer, parses 5,000 lines, shapes 50     dirty line, shapes 1 dirty row, reuses
                                   visible Galleys                           4,999 lines + 49 row Galleys
Keystroke echo                     30 frames x 33ms blind timer + 60 tmux    0 blind timers when watched; 1 atomic
                                   procs over 1s                             capture on %output (~10ms), 1 repaint
Scroll by 1 row (50-row viewport)  Shapes all 50 visible Galleys             Shapes at most 1 newly visible row,
                                                                             reuses 49 cached row Galleys
Pointer resting on "v2.x" while    Wipes None path cache every capture ->    Keeps negative path entry for 2s TTL ->
spinner ticks at 60Hz              re-probes disk + request_repaint loop     0 disk re-probes, 0 extra repaints
```

## Empty / Loading State (pane ~740px wide, before first capture lands)

```text
┌──────────────────────────────────────────────────────────────────────────┐
│ [● 新しいセッション ×]                                                   │
├──────────────────────────────────────────────────────────────────────────┤
│ ┌──────────────────────────────────────────────────────────────────────┐ │
│ │ ターミナル出力を待っています…                                        │ │
│ │                                                                      │ │
│ │                                                                      │ │
│ └──────────────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────────┘
```

## Error State (capture failure toast / notice banner, ~740px wide)

```text
┌──────────────────────────────────────────────────────────────────────────┐
│ ⚠ ターミナル出力を読み取れませんでした: tmux capture-pane timed out     │
├──────────────────────────────────────────────────────────────────────────┤
│ [● Claude: API 修正 ×]                                                   │
├──────────────────────────────────────────────────────────────────────────┤
│ ┌──────────────────────────────────────────────────────────────────────┐ │
│ │ (直前に取得できた最後のターミナル表示をそのまま保持)                 │ │
│ └──────────────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────────┘
```

## Too-Long / Split 4-Pane / Flooded Output State (narrow split pane ~490px wide)

```text
┌────────────────────────────────────────────────┬────────────────────────────────────────────────┐
│ [● Claude: 実装と検証を並行実行中… ×]          │ [● Terminal: ログ監視 ×]                       │
├────────────────────────────────────────────────┼────────────────────────────────────────────────┤
│ ┌────────────────────────────────────────────┐ │ ┌────────────────────────────────────────────┐ │
│ │ Very long unwrapped compiler diagnostic li…│ │ │ [03:02:11] line 4998 of 5000 (16ms batch)  │ │
│ │   --> src/ui/terminal.rs:681:18            │ │ │ [03:02:11] line 4999 of 5000               │ │
│ │ ▊                                          │ │ │ ▊                                          │ │
│ └────────────────────────────────────────────┘ │ └────────────────────────────────────────────┘ │
└────────────────────────────────────────────────┴────────────────────────────────────────────────┘
```

- Unwrapped long lines continue to clip cleanly at the pane's right inner edge (`pane_width`, `TextWrapMode::Extend`).
- Long tab titles continue to truncate with `…` via single-pass `Galley` measurement.
- Output floods coalesce to `16ms` batches (`MAX_CAPTURE_PANE_BYTES` = `8 MiB`, `5000` lines max).
