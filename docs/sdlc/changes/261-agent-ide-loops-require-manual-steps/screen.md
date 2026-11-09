# Screen: Hunk actions, editor/terminal agent handoff, `@` mentions, external IDEs, parallel arena, and `⌘K` file/symbol jump

- **Status**: approved

1. Diff pane hunk header row (`src/ui/diff.rs`):
```
▼  src/git.rs  +14 −3
────────────────────────────────────────────────────────────────────────
          @@ -120,6 +120,17 @@ pub(crate) fn parse_unified_diff  [ステージ] [破棄]
  120  120   let mut files = Vec::new();
       121 + let mut hunk_count = 0;
```

2. Built-in editor header (`src/app/screens.rs`):
```
[✎ 編集] [⇄ 差分]          [エージェントに送る] [外部エディタで開く ▾] [↗] [↻] [💾 保存]
                                                  ├─ VS Code
                                                  ├─ Cursor
                                                  ├─ Zed
                                                  ├─ Windsurf
                                                  ├─ Antigravity
                                                  ├─ Void
                                                  └─ Finder
```

3. Session terminal `⋯` menu (`src/app/screens.rs`):
```
┌────────────────────────────────────────────┐
│ Terminal.app で開く（マウス操作・tmux 操作用） │
│ 表示中のログをすべてコピー                   │
│ 直近のエラーをエージェントに送る             │
│ 今すぐ同期                                 │
├────────────────────────────────────────────┤
│ 外部エディタで開く                          │
│   VS Code / Cursor / Zed / Windsurf / ...  │
└────────────────────────────────────────────┘
```

4. Launch sheet (`⌘N`) — parallel arena & `@` context mentions (`src/app/screens.rs`):
```
エージェント
  [Codex] [Claude Code] [Antigravity] [別のコマンドを使う]
  複数エージェントで並走（別 worktree で比較）: [x] Codex  [x] Claude Code  [ ] Antigravity

依頼（任意）  [テンプレートを挿入 ▾] [@ コンテキストを挿入 ▾] [GitHub Issue から取得…]
┌──────────────────────────────────────────────────────────────────────┐
│ @src/git.rs のハンク適用処理を確認して @diff                         │
└──────────────────────────────────────────────────────────────────────┘
  候補: [+ @diff (未コミットの変更差分)] [+ @src/git.rs] [+ @src/ui/diff.rs]
```

5. Command palette (`⌘K`) — file and symbol rows (`src/app/screens.rs`):
```
┌──────────────────────────────────────────────────────────────────────┐
│ 🔍 parse_unified_diff                                                │
├──────────────────────────────────────────────────────────────────────┤
│ fn parse_unified_diff                       シンボル · src/git.rs:113 │
│ src/git.rs                                  ファイル · operon         │
└──────────────────────────────────────────────────────────────────────┘
```

All controls use existing palette roles (`palette.raised`, `palette.card`, `palette.text`, `palette.text_muted`, `palette.accent_soft`, `palette.danger`) and existing Phosphor icons.
