# Screen: 180 — Handoff destination model and effort, and the reuse notice

- **Status**: approved
- Structure: no new screen, modal, or panel. One row of two existing control
  kinds (the launch sheet's `モデル（任意）` text field and `推論の深さ` picker) is
  added inside each history card under its existing `復元先` row; the session
  menu entries gain a parenthesised suffix; the notice bar gets one more
  sentence. Existing palette roles only.

## 1. `履歴` → `CLI セッション` card (`ui_cli_sessions`, 880px content width)

### Before

```
┌───────────────────────────────────────────────────────────────────────────────────────────────────┐
│ Claude Code  認証エラーの修正  5分前                                                  [↗] [▶]     │
│ 現在の Claude Code セッション ID  4c022664-9f6f-4ea1-9642-2f7cbc96376a  [⧉]                       │
│ ⎇ feature/auth-fix                                                                                │
│ この会話の最後の依頼                                                                              │
│ ログイン失敗時のバリデーションエラーを修正して                                                    │
│ 復元先  [ Codex CLI ▾ ]  [ ⟲ Handoff バンドルで引き継ぐ ]                                         │
└───────────────────────────────────────────────────────────────────────────────────────────────────┘
```

### After

```
┌───────────────────────────────────────────────────────────────────────────────────────────────────┐
│ Claude Code  認証エラーの修正  5分前                                                  [↗] [▶]     │
│ 現在の Claude Code セッション ID  4c022664-9f6f-4ea1-9642-2f7cbc96376a  [⧉]                       │
│ ⎇ feature/auth-fix                                                                                │
│ この会話の最後の依頼                                                                              │
│ ログイン失敗時のバリデーションエラーを修正して                                                    │
│ 復元先  [ Codex CLI ▾ ]  [ ⟲ Handoff バンドルで引き継ぐ ]                                         │
│ モデル（任意） [ gpt-5-mini            ]  推論の深さ [ low                    ▾ ]                 │
└───────────────────────────────────────────────────────────────────────────────────────────────────┘
```

- Empty field shows the hint `既定のモデルを使う`; the picker's empty choice is
  `既定（CLI に任せる）`, followed by the destination's own effort list
  (Codex: `minimal low medium high xhigh`; Claude Code: `low medium high xhigh
  max`; Antigravity: `low medium high`).
- Changing `復元先` to `Claude Code（同じ CLI に複製）` or `Antigravity` shows that
  destination's own saved model and effort in the same row.
- Hover on the model field: `Codex CLI への Handoff で使うモデル。空欄なら Codex CLI の既定のモデルです。`
- Text field 170px and picker 200px, as on the launch sheet.

### Too long

- A model name wider than 170px scrolls inside the field, as on the launch
  sheet; the field accepts at most the settings file's per-field ceiling.

## 2. Session row menu (`render_restore_options`, 220px menu width)

### Before

```
┌──────────────────────── 220px ─┐
│  …                             │
│ ────────────────────────────── │
│  Codex CLI へ Handoff          │
│  Antigravity へ Handoff        │
└────────────────────────────────┘
```

### After (Codex options set, Antigravity none)

```
┌──────────────────────────────────────── 300px ─┐
│  …                                             │
│ ────────────────────────────────────────────── │
│  Codex CLI へ Handoff（gpt-5-mini · low）      │
│  Antigravity へ Handoff                        │
└────────────────────────────────────────────────┘
```

- Only the effort set: `Codex CLI へ Handoff（low）`. The menu widens to fit
  its longest entry, as egui menus do.
- Disabled while a handoff is running, as today.

## 3. Notice bar

### Reused

```
┌─ ⓘ ───────────────────────────────────────────────────────────────────────────────────────── [✕] ─┐
│ Claude Code の会話 4c022664-… の Handoff バンドル（作成: 3分前）を再利用し、Codex CLI の新規     │
│ セッションを起動しています…                                                                        │
└────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

### Fresh (unchanged from change 179)

```
│ Claude Code の会話 4c022664-… から Handoff バンドル（直近 8 ターン）を作成し、Codex CLI の新規 … │
```

### Refused options

```
┌─ ⓘ ───────────────────────────────────────────────────────────────────────────────────────── [✕] ─┐
│ エージェントの起動オプションが不正です。                                                           │
└────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

No modal opens and the card is unchanged.

## 4. Loading and empty states

- The progress modal while a handoff runs is unchanged (change 179); a reuse
  closes it as quickly as a fresh one.
- Before a scan and with no sessions found: unchanged.
