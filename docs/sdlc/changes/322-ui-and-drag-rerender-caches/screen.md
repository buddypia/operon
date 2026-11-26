# Screen: Cached UI & Zero-Rerender Drag Pipeline (`Page::Sessions`, `Page::Projects`, `⌘K`)

- **Spec**: `./spec.md`
- **Status**: approved

## Overview

This change alters zero visual chrome, colours, typography, spacing, icons, or Japanese strings across Operon. `docs/sdlc/risk.yaml` marks `drawing` (`src/app.rs`, `src/app/screens.rs`, `src/ui`) as `screen: yes`; this document records the preserved layouts across active workspace, drag-and-drop drop zones, diff, editor/Markdown preview, and `⌘K` palette states alongside Before vs After frame work.

## Active Workspace & Drag States (1280px window, sidebar 268px + workspace 1012px)

### Visual layout (unchanged before and after)

```text
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│ Operon   ホーム   セッション   プロジェクト   ナレッジ   設定                       ⌘K 検索・操作 │
├────────────────────────┬─────────────────────────────────────────────────────────────────────────┤
│ セッション一覧   + 履歴│ [● API 修正 ×] [○ テスト ×]                                         [+] │
│ [要対応][実行中][アイドル]├─────────────────────────────────────────────────────────────────────────┤
│ [Codex][Claude][AG][他]│ ╭─ 作業フォルダ / API 修正 ──────────────────── [停止] [サイドパネル] ╮ │
│ ▾ Operon    ●1 ○1  [+] │ │                                                                     │ │
│   [pin] [C] API 修正   │ │      ░░░░░░░░░░░░░░░░░ ここに表示 / 右に分割 ░░░░░░░░░░░░░░░░░░░░░░░  │ │
│         [G] テスト     │ │      (D&D プレビュー枠 & ツールチップ表示中も背面キャッシュ保持)     │ │
│                        │ └─────────────────────────────────────────────────────────────────────┘ │
└────────────────────────┴─────────────────────────────────────────────────────────────────────────┘
```

### Frame & D&D behaviour across states

```text
State                              Before                                    After
─────────────────────────────────  ────────────────────────────────────────  ────────────────────────────────────────
Dock border / split divider drag   Spawns tmux resize every 150ms mid-drag;  0 tmux resizes during drag; 1 settled
                                   dock drags missing from drag guard;       resize on release; 0 workspace syncs,
                                   re-sorts/clones sidebar + projects        0 sidebar rebuilds, 0 poll spawns
Tab drag over split pane drop zone Deep-clones entire WorkspaceTabs +        0 Group/Tab/PathBuf clones (can_move_tab
                                   all PathBufs every pointer frame          checks 4-leaf PaneLayout only)
Steady workspace frame             Runs sync_workspace_live_sessions +       0 syncs (fingerprint hit); 1x visible-
(sidebar + 1..4 panes)             writes visible_sessions & sync_streams    session write-on-change; SidebarViewCache
                                   2x per frame; O(S^2) / O(S*T) scans       reused directly (0 clones/string builds)
Diff pane scroll / steady frame    Rebuilds all rows + offsets every frame;  Cached row table + offsets + bounded
                                   shapes LayoutJob + locks fonts per row    row Galley cache (0 locks on hit)
Editor / Viewer / Markdown frame   Clones 128 KiB LayoutJob + line_numbers   Reuses Arc<Galley> for syntax, gutter,
                                   every frame; allocates cursor prefix      viewer rows, Markdown blocks; culls
                                   String; rebuilds all Markdown blocks      off-screen Markdown blocks past 32
⌘K Palette open / ↑↓ navigation    Evaluates 12k files + symbols 2x/frame    Caches entries by query/state hash (0
                                   and again on every arrow keystroke        re-ranks on same frame or arrow keys)
```

## Empty, Loading, Error & Narrow Split States (Preserved)

- **Empty workspace (`workspace.panes.is_empty()`)**: Preserves `「セッションを選択」`, `「一覧からセッションを選んでください。」`, and the disabled/enabled `「＋」` launch menu button unchanged.
- **Empty diff / binary diff / error**: Preserves `「変更はありません。」`, `「内容の変更なし」`, and error notices unchanged.
- **Loading / unopenable editor**: Preserves `「読み込んでいます…」`, `「バイナリファイルはエディタで開けません。」`, and size-limit notices unchanged.
- **Narrow pane (`< DIFF_SIDE_BY_SIDE_MIN_WIDTH`)**: Side-by-side diff falls back to `Unified` automatically, matching before.
- **Long text / split 4 panes**: Truncated project headings, session titles, tabs, and unwrapped viewer/diff/editor rows match pixel-for-pixel.
