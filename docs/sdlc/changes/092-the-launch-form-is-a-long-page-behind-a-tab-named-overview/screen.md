# Screen: the launch sheet, 概要, and 履歴

- **Status**: approved

The owner approved the rendered mock `.tmp/ui-before-after/view-projects-after.html`
(section ③, "全項目 OK、進めて", 2026-09-27). This file is the part of it this
change builds.

## Launch sheet (over any page)

```
┌ 新しいセッション  [design-lookbook ▾]                               ✕ ┐
│ エージェント                                                          │
│ [◉ Codex ●] [◉ Claude Code ●] [◉ Antigravity ●]   別のコマンドを使う   │
│ アカウント [このマシン ▾]            (Codex / Claude only, as today)   │
│ 依頼（任意）                                                          │
│ [例: ログイン画面の不具合を調査して修正して。            ]            │
│ 作業場所  プロジェクト直下                                  ↗         │
│ ▸ 詳しい設定（任意） · モデル opus · …                                │
│───────────────────────────────────────────────────────────────────────│
│ (notices: data leaves the Mac / Claude opens a terminal / danger       │
│  warning and 理解したうえで起動する / blocking reason and 設定を開く)   │
│ ターミナルだけ開く                          [Antigravity を起動  ⌘↩] │
└───────────────────────────────────────────────────────────────────────┘
```

- A scrim covers the page; the sheet is centred, 560 px wide, at most 80 %
  of the window high; its body scrolls and its footer does not.
- Agent choices: agent mark, name, a dot — `ICON_AVAILABLE` in the success
  colour when usable, `ICON_UNAVAILABLE` in `accent_soft` when not (hover:
  「この Mac で使用可能」 / 「未準備（設定で確認）」). The selected one is
  outlined in its agent colour. 「別のコマンドを使う」 is a quiet button; with
  it chosen, 「起動コマンド（必須）」 appears under the row, as today.
- Where the sheet departs from the mock, and why:
  - The account row stays visible (not folded), as today: it names a choice
    already made (`the_account_row_is_on_the_screen_without_unfolding_anything`);
    the mock lists アカウント under 詳しい設定.
  - 作業場所 has no 「既存 worktree ▾」 picker: the launch form never had one; an
    existing worktree is chosen with the worktree tab's
    「この場所で新規セッション」, which now opens this sheet with it set. Nor a
    「新しい worktree」 choice (intent: not in scope).
  - 「別のコマンドを使う」 stays beside the agent row rather than inside
    詳しい設定, which the checklist allowed: it is an agent choice, and the
    required 「起動コマンド」 field it reveals must not be folded away.
- 作業場所: 「プロジェクト直下」 or 「ワークツリー: <branch>」 (a worktree chosen
  on the worktree tab), 「プロジェクト直下に戻す」 when a worktree is chosen,
  and the Finder icon.
- 詳しい設定（任意）: session name, model, the CLI's mode / effort / switches,
  開始順 (ほかのセッションの後に開始). Its header names what is set inside, as
  today.
- The launch button is filled with the agent colour and reads
  「<agent> を起動  ⌘↩」. Disabled with the reason under it, as today.
- ✕, Esc: close without launching; what was typed stays for next time.
  Clicking the scrim does nothing. Opening the sheet puts the cursor in
  依頼, so keys no longer go to a terminal that had focus.
- A refusal from the launch (e.g. an account not ready) is repeated at the top
  of the footer.

## 概要 tab

```
[セッション 2 待機中]  [未コミットの変更 1 ファイル]  [worktree 2]
[＋ 新しいセッション]
このプロジェクトのセッション
○ 待機中  …one-line rows as on Home…
```

- The change and worktree numbers come from the tabs' caches, requested in
  the background when absent; 「—」 while loading or on error.

## 履歴

Under today's session grid, a 「CLI セッション」 section with a project picker,
holding exactly what the 概要 tab held: 「ローカル CLI セッションを検出」, the
remembered-reference line, recent restores, the restore note, and each found
conversation with 元の CLI で再開 / 復元先 / 会話全履歴を復元.

## Non-happy states

| State | Shown |
|---|---|
| No project | 新しいセッション: today's notice and the Projects page; no sheet |
| tmux or agent missing | sheet opens; button disabled; reason and 設定を開く |
| ⌘↩ while not ready | nothing |
| Launch accepted | the sheet closes; the launch progress modal takes over, as today |
