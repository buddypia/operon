# Session workspace layout

- **Status**: approved

The user already requested this structural change in the active task:
“セッションページの中央ウィンドウに新しいタブとして表示されるべき” and
“左プロジェクトのみ表示されてセッションはセッションタブを中央ウィンドウに追加”.
This records that authorization rather than an independent screen approval.

Before, 1280px window:
```text
┌─ セッション ─────┬─ 選択したセッション ───────────┬─ ファイル ──────┐
│ 会話 A          │ ターミナル                    │ src/app.rs      │
│ 会話 B          │                               │ クリック→別ページ│
└─────────────────┴───────────────────────────────┴─────────────────┘
```

After, 1280px window, approximately 200px / 800px / 280px:
```text
┌─ プロジェクト ──┬─[会話 A ×][会話 B ×][app.rs ×]─┬─ ファイル ──────┐
│ operon          │ 選択したタブのターミナル/エディタ│ src/app.rs      │
│ another-project │                                │ src/tests.rs    │
└─────────────────┴────────────────────────────────┴─────────────────┘
```

A tab dragged to the right or bottom edge initially uses half the center:
```text
┌─[会話 A ×]──────────┬─[app.rs ×][会話 B ×]────────┐
│ ターミナル           │ エディタ                    │
│                     │                             │
└─────────────────────┴─────────────────────────────┘
┌─[会話 A ×][会話 B ×]───────────────────────────────┐
│ ターミナル                                          │
├─[app.rs ×]─────────────────────────────────────────┤
│ エディタ                                            │
└────────────────────────────────────────────────────┘
```

The divider is draggable. Existing minimum pane dimensions and maximum count
prevent unusable splits, with これ以上分割できません when refused. A center drop
moves a tab into the group; dropping over the tab strip reorders it.
Long names truncate, hover reveals identity, and the tab strip scrolls.

Empty: セッションを選択, with projects still accessible.
Loading: 読み込んでいます… in the editor content.
Denied/too large/error: the existing editor reason or save-error UI in that tab.
Dirty-close: retain the tab and show the existing unsaved-change notice.
Interrupted drag: Escape or release outside groups keeps the original layout.
