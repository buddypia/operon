# Screen: セッション page — the two search fields

No layout moves and no control is added. One line of weak text under
「ローカル履歴」 changes when it appears, and gains a "searching" wording. The
page is drawn at the workspace's width; 900px here.

## 1. 「セッションを検索」 — which rows stay

The session 「operon · Claude Code」 was launched without a name; its first
prompt was 「ログイン画面の余白を直して」, so the list titles it by that line.

```
Before — typing a word of the listed title
┌──────────────────────────────────────────────────────────── 900px ─┐
│ ⟳  [プロジェクト別] グリッド   [ログイン                  ]        │
│ ローカル履歴 [Codex と Claude の履歴を検索      ] 🔍 履歴を検索    │
│ ────────────────────────────────────────────────────────────────── │
│  operon                                                         ＋ │
│   一致するセッションはありません。                                  │
└────────────────────────────────────────────────────────────────────┘

After
┌──────────────────────────────────────────────────────────── 900px ─┐
│ ⟳  [プロジェクト別] グリッド   [ログイン                  ]        │
│ ローカル履歴 [Codex と Claude の履歴を検索      ] 🔍 履歴を検索    │
│ ────────────────────────────────────────────────────────────────── │
│  operon                                                         ＋ │
│   Ⓒ‖ ログイン画面の余白を直して              feature/login   3分前 │
└────────────────────────────────────────────────────────────────────┘
```

The same row stays for 「operon」 (its project), 「feature/login」 (its branch),
「claude code」 (its agent as written), and 「ログイン 」 with a trailing space —
in 「グリッド」 as in 「プロジェクト別」.

- **Empty**: no text — every row, as today.
- **Nothing matches**: 「一致するセッションはありません。」 under each project,
  as today.
- **Too long**: the field scrolls its text as today; nothing else is drawn.

## 2. 「ローカル履歴」 — the line under the field

```
Before — typed, Enter not pressed          After — typed, Enter not pressed
│ ローカル履歴 [余白        ] 🔍 履歴を検索 │ ローカル履歴 [余白        ] 🔍 履歴を検索
│ 一致する履歴はありませんでした。          │
│ ───────────────────────────────────────  │ ───────────────────────────────────────

Before — searching                         After — searching
│ ローカル履歴 [余白        ] 🔍 履歴を検索 │ ローカル履歴 [余白        ] 🔍 履歴を検索
│ 一致する履歴はありませんでした。          │ 履歴を検索しています…
│ ───────────────────────────────────────  │ ───────────────────────────────────────

Finished, nothing found — unchanged
│ ローカル履歴 [余白        ] 🔍 履歴を検索
│ 一致する履歴はありませんでした。
│ 💡 意味検索を有効にすると表記揺れも検索できます。 設定で有効化する
│ ───────────────────────────────────────
```

- **Finished, matches**: the 「履歴の一致」 / 「キーワードで一致」 list, as today.
- **Edited after "none"**: the line goes away until the next search finishes.
- **Field emptied while a search runs**: nothing is drawn under the field.
- **Error / cut short**: the corner notice reports a scan that hit its limits,
  as today; the panel reads "none" or lists what was found.
- 「履歴を検索しています…」 is weak text on the line the "none" sentence uses;
  it is shorter than that sentence and cannot wrap where that one does not.
