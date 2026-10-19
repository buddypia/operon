# Screen: 179 — Cross-CLI compact Handoff bundle UI

- **Status**: approved
- Structure: existing roles and layout geometry in `履歴` (`ui_cli_sessions`),
  the session row context menu (`render_restore_options`), and the progress modal
  (`ui_restore_modal`) are unchanged; wording inside those existing roles updates
  from full-history store import to compact Handoff bundle per the user's
  request.

## 1. `履歴` → `CLI セッション` (`ui_cli_sessions`, 880px content width)

### Before

```
┌─ CLI セッション  Operon の外で見つかったネイティブセッション  [🔍 ローカル CLI セッションを検出] ─ 880px ─┐
│  ⟲ 会話全履歴を復元: Claude Code → Codex CLI · 3分前                                                     │
│  ▶ 元の CLI で再開 · ⟲ 同じ CLI にも別の CLI にも全履歴を復元（トークン消費なし）                        │
│  ┌─────────────────────────────────────────────────────────────────────────────────────────────────────┐ │
│  │ 復元メモ（任意）                                                                                    │ │
│  └─────────────────────────────────────────────────────────────────────────────────────────────────────┘ │
│  ┌─────────────────────────────────────────────────────────────────────────────────────────────────────┐ │
│  │ Claude Code  認証エラーの修正  5分前                                                  [↗] [▶]       │ │
│  │ 現在の Claude Code セッション ID  4c022664-9f6f-4ea1-9642-2f7cbc96376a  [⧉]                         │ │
│  │ ⎇ feature/auth-fix                                                                                  │ │
│  │ この会話の最後の依頼                                                                                │ │
│  │ ログイン失敗時のバリデーションエラーを修正して                                                      │ │
│  │ 復元先  [ Codex CLI ▾ ]  [ ⟲ 会話全履歴を復元 ]                                                     │ │
│  └─────────────────────────────────────────────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

### After

```
┌─ CLI セッション  Operon の外で見つかったネイティブセッション  [🔍 ローカル CLI セッションを検出] ─ 880px ─┐
│  ⟲ Handoff バンドルで引き継ぎ: Claude Code → Codex CLI · 3分前                                           │
│  ▶ 元の CLI で再開 · ⟲ コンパクトな Handoff バンドルで別 CLI へ引き継ぎ                                  │
│  ┌─────────────────────────────────────────────────────────────────────────────────────────────────────┐ │
│  │ 引き継ぎメモ（任意）                                                                                │ │
│  └─────────────────────────────────────────────────────────────────────────────────────────────────────┘ │
│  ┌─────────────────────────────────────────────────────────────────────────────────────────────────────┐ │
│  │ Claude Code  認証エラーの修正  5分前                                                  [↗] [▶]       │ │
│  │ 現在の Claude Code セッション ID  4c022664-9f6f-4ea1-9642-2f7cbc96376a  [⧉]                         │ │
│  │ ⎇ feature/auth-fix                                                                                  │ │
│  │ この会話の最後の依頼                                                                                │ │
│  │ ログイン失敗時のバリデーションエラーを修正して                                                      │ │
│  │ 引き継ぎ先  [ Codex CLI ▾ ]  [ ⟲ Handoff バンドルで引き継ぐ ]                                       │ │
│  └─────────────────────────────────────────────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

- Same-CLI option in the picker still reads `Claude Code（同じ CLI に複製）`.
- Hover on `[ ⟲ Handoff バンドルで引き継ぐ ]`:
  - Cross-CLI: `{p0} へコンパクトな Handoff バンドルで引き継ぎます`
  - Same-CLI: `同じ {p0} に Handoff バンドルで会話を複製します`
- Existing historical `CliHandoff` entries with `full_history == true` continue
  to render `⟲ 会話全履歴を復元: …`; new entries (`full_history == false`) render
  `⟲ Handoff バンドルで引き継ぎ: …`.

### Empty and pre-scan states (unchanged)

- Before scan:
  ```
  │  Operon の外で起動したネイティブセッションを検出します。                                              │
  ```
- Scan returned 0 sessions:
  ```
  │  このプロジェクトのローカル CLI セッションは見つかりませんでした。                                    │
  ```

## 2. Session row context menu (`render_restore_options`, 220px menu width)

### Before

```
┌──────────────────────── 220px ─┐
│  名前を変更                    │
│  …                             │
│ ────────────────────────────── │
│  Codex CLI へ履歴を復元        │
│  Antigravity へ履歴を復元      │
└────────────────────────────────┘
```

### After

```
┌──────────────────────── 220px ─┐
│  名前を変更                    │
│  …                             │
│ ────────────────────────────── │
│  Codex CLI へ Handoff          │
│  Antigravity へ Handoff        │
└────────────────────────────────┘
```

- Hover on each item: `{p0} へコンパクトな Handoff バンドルで引き継ぎます`.
- Disabled while another background handoff task (`FullHistoryImport`) is running.

## 3. Handoff progress modal (`ui_restore_modal`, 520px width)

### Before

```
┌────────────────────────────────────────────────────────── 520px ─┐
│  ⟳  Claude Code の会話を Codex CLI に復元しています…             │
│                                                                  │
│  復元元の会話 ID    4c022664-9f6f-4ea1-9642-2f7cbc96376a         │
│  復元先            Codex CLI                                     │
│                                                                  │
│  復元元の最後の依頼                                              │
│  ログイン失敗時のバリデーションエラーを修正して                  │
│                                                                  │
│ ──────────────────────────────────────────────────────────────── │
│  ローカル履歴ファイルから抽出しています · 経過 2 秒 · モデルは   │
│  実行しないためトークンは消費しません                            │
│                                      [ バックグラウンドで続ける ] │
└──────────────────────────────────────────────────────────────────┘
```

### After (cross-CLI handoff and same-CLI fork)

```
┌────────────────────────────────────────────────────────── 520px ─┐
│  ⟳  Claude Code の会話を Codex CLI に Handoff しています…        │
│                                                                  │
│  復元元の会話 ID    4c022664-9f6f-4ea1-9642-2f7cbc96376a         │
│  復元先            Codex CLI                                     │
│                                                                  │
│  復元元の最後の依頼                                              │
│  ログイン失敗時のバリデーションエラーを修正して                  │
│                                                                  │
│ ──────────────────────────────────────────────────────────────── │
│  ローカル履歴から Handoff バンドルを組み立てています · 経過 1 秒 │
│                                      [ バックグラウンドで続ける ] │
└──────────────────────────────────────────────────────────────────┘
```

- Same-CLI title remains: `{p0} の会話を同じ CLI に複製しています…`.
- Long `last_user_message` is truncated at 220 chars (`truncate_chars(&progress.last_user_message, 220)`).
- Footer status label and right-aligned `[ バックグラウンドで続ける ]` button stay on
  separate vertical rows via `restore_progress_footer` so their rects never
  overlap at 360px or 520px.
- Error / completion banners appear in the top notice bar:
  - Success: `{source} の会話 {source_conversation} から Handoff バンドル（直近 {records} ターン）を作成し、{destination} セッションを起動しています。{breakdown}`
  - Empty transcript error: `選択した元履歴には、復元できる記録がありません。`
  - Failure notice: `会話全履歴を復元できませんでした。元の CLI セッションは変更されていません: {error}`
