# Screen: settings in five sections

- **Status**: approved

The owner approved the rendered mock `/tmp/operon-ui-before-after/view-settings-after.html`
(a copy sits in the worktree's `.tmp/ui-before-after/`)
(section ④, "全項目 OK、進めて", 2026-09-27). This file is what this change
builds from it.

## Layout

```
設定                 │ エージェント
 外観                │ 起動に必要なツールと、状態の検知方法
▌エージェント 準備完了 │ ───────────────────────────────────────────
 通知                │ ┌ ツール  ✓ 5/6 使用可能                ↻ 再確認 ┐
 データと復旧          │ │ ◉ Codex CLI    フック 導入済み   ✓ 使用可能  アカウント 1 │
 キー割り当て          │ │ ◉ Claude Code  フック 導入済み   ✓ 使用可能  アカウント 0 │
                     │ │ ◉ Antigravity  フック 未導入     ✓ 使用可能            │
                     │ │   tmux  セッションの実行に必須    ✓ 使用可能            │
                     │ │   Git   worktree・変更内容・履歴  ✓ 使用可能            │
                     │ │   GitHub CLI 任意: PR の詳細    ✗ 未検出             │
                     │ └──────────────────────────────────────────┘
                     │ ┌ 起動時の動作 ────────────────────────────────┐
                     │ │ ☑ エージェント CLI のフックで状態を検知する          │
                     │ │ ☐ 初回起動時のフォルダ信頼プロンプトを自動承認する  フォルダ信頼のみ │
                     │ └──────────────────────────────────────────┘
                     │ エージェントのアカウント (today's list and add row)
```

- Left list: 200 px, 「設定」 at its top, five rows. The chosen row looks like
  the toolbar's current page (`nav_tab`: `row_selected` fill, `accent_text`). エージェント's row ends with 「準備完了」 in the success colour
  when tmux and at least one agent are present, else 「要対応」 in
  `accent_soft`.
- Right: `page_header` with the section name and one line; the section below.
- ⚙, the palette's 「設定」, and ⌘, show the section chosen last (外観 the
  first time). Each 「設定を開く」 / 「セットアップ手順を開く」 reached from a
  missing tool opens エージェント.

## Sections

- **外観** — 「テーマ、フォント、表示言語」: today's theme, font, and language
  rows, unchanged.
- **エージェント** — 「起動に必要なツールと、状態の検知方法」:
  - ツール card: 「ツール」, 「{n}/6 使用可能」 (success when all, else
    `accent_soft`), 「再確認」 (today's ↻ ツールを再確認). One row per tool:
    mark, name, purpose, and on the right 「使用可能」 (success) or 「未検出」
    (warning, as today). An agent's row also names its hook — 「フック
    導入済み」 (success) / 「フック 未導入」 (`text_faint`, as today) /
    「フック 失敗」 (danger), the reason on hover for the last two — only while
    hooks are on and only once that CLI's hook state is known — and 「アカウント {n}」 for agents that take accounts. A missing
    agent's row has 「手順」, opening today's setup page for that CLI; a
    missing tmux shows 「brew install tmux」.
  - 起動時の動作 card: today's hooks checkbox with its explanation, today's
    auto-approve checkbox relabelled 「初回起動時のフォルダ信頼プロンプトを自動
    承認する」 with 「フォルダ信頼のみ」 beside it.
  - Today's エージェントのアカウント list and add row, unchanged.
  - The PATH note (「Finder から起動したアプリは…」) under the card when
    something is missing, as today.
- **通知** — 「セッションの状態が変わったときの通知」: today's notification
  checkbox.
- **データと復旧** — 「この Mac に保存されるデータと、失われた端末の復旧」: today's
  ローカルデータ text, path, and 「データフォルダを表示」; today's ターミナル
  復旧 row and its orphan cards.
- **キー割り当て** — 「ショートカットの一覧。変更はファイルで行います」: one row per
  keymap action, its label and its chord as the palette draws it (「なし」 when
  unbound); then 「キー割り当てファイルを開く」, the palette's existing action
  (writes the defaults if there is no file, then reveals its folder).

## Where this departs from the mock, and why

- Checkboxes, not toggle switches: egui has none, and a custom widget is out
  of scope (intent). No person decision needed.
- No 「導入する」 link in a hook cell: hooks are registered for every CLI at
  once by the 起動時の動作 checkbox, which sits right below.
- 「アカウント {n}」 is text, not a button: the account list sits in the same
  section just below. Antigravity has no account cell: its CLI takes no
  account folder (`agent_supports_accounts`).
- The five nav rows carry no icon: there is no bell, palette, or keyboard mark
  in the bundled set, and the labels name the sections alone.
- 「手順」 is offered for a missing agent (a missing tmux gets the brew line
  instead), not for Git or GitHub CLI: those have no setup page today, and
  GitHub CLI is optional.
- Colours stay today's: 未検出 in warning, hook 未導入 in `text_faint` (the mock
  draws the first faint and the second in the attention colour).

## Non-happy states

| State | Shown |
|---|---|
| tmux missing | エージェント badge 要対応; tmux row 未検出 and brew line |
| No agent | badge 要対応; each agent row 未検出 with 手順 |
| Hooks off | agent rows without a hook cell |
| Keymap file absent | the defaults listed; the button creates the file |
