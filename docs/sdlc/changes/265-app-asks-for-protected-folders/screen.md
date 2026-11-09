# Screen: what Operon shows about the person's own folders

- **Status**: approved

Two existing surfaces change wording or state; no layout moves, no new control.

## Launch sheet footer, a dangerous switch ticked (sheet ~560px)

Before:
```
│ 選んだ AI は外部ツールとして実行され、プロジェクトの内容を提供元へ送信することがあります。 │
│ 警告: 確認もサンドボックスもなしにファイルを変更できます。                               │
│ [ ] 理解したうえで起動する                                                             │
```
After (same `accent_soft` small label; wraps to a second line at this width):
```
│ 選んだ AI は外部ツールとして実行され、プロジェクトの内容を提供元へ送信することがあります。 │
│ 警告: 確認もサンドボックスもなしに、プロジェクトの外（書類・ダウンロード・デスクトップ   │
│ など）のファイルも読み書きできます。                                                   │
│ [ ] 理解したうえで起動する                                                             │
```
No dangerous switch: the line is absent, as today.

## Terminal pane, agent output naming files (pane 900px)

Before — every existing file is underlined and clickable:
```
│ Read(src/app.rs)                       ← underlined, opens in the Files tab   │
│ Read(~/.claude/settings.json)          ← underlined, opens outside            │
│ Found ~/Downloads/brief.pptx           ← underlined (Operon stat'ed it)       │
```
After — a file under Desktop, Documents, Downloads or /Volumes that
is not inside this session's folders is plain text and is never looked up:
```
│ Read(src/app.rs)                       ← underlined, opens in the Files tab   │
│ Read(~/.claude/settings.json)          ← underlined, opens outside            │
│ Found ~/Downloads/brief.pptx           ← plain text                           │
```
A project registered at `~/Documents/proj`: its own files stay underlined;
`~/Documents/other.txt` is plain text.

## The macOS folder prompt (system dialog, after packaging)

Before:
```
「Operon」が"ダウンロード"フォルダ内のファイルへのアクセスを求めています。
                                  [許可しない] [許可]
```
After (the system draws the title and buttons; the second paragraph is ours):
```
「Operon」が"ダウンロード"フォルダ内のファイルへのアクセスを求めています。
多くの場合、Operon のターミナルで動いているエージェントやコマンドがこの
フォルダを使おうとしています（macOS は Operon が起動したプロセスの操作を
Operon の名前で尋ねます）。心当たりがなければ「許可しない」を選んでください。
このフォルダにプロジェクトを置いていなければ、Operon の動作には影響しません。
                                  [許可しない] [許可]
```
