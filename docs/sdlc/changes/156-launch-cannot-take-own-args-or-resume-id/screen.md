# Screen: launch sheet, 「詳しい設定（任意）」

Inside the existing folded section of the launch sheet (about 560px wide). No new
screen, no re-arrangement: two rows appended after the switches, above
「ほかのセッションの後に開始」.

## Before

```
▾ 詳しい設定（任意） · 権限確認をすべてスキップする
  セッション名（任意） [ 例: ログイン不具合の修正          ]
  モデル（任意）       [ 既定のモデルを使う      ]
  権限モード           [ 既定（CLI に任せる）  ▾]
  推論の深さ           [ 既定（CLI に任せる）  ▾]
  ☑ 権限確認をすべてスキップする（危険）
  ☐ 後から「権限確認をすべてスキップ」を選べるようにする
  ☐ IDE に自動接続する
  ☐ 詳細ログを出す
  ほかのセッションの後に開始 [ すぐ開始する（既定） ▾]
```

## After — happy state (claude, resuming unattended)

```
▾ 詳しい設定（任意） · 権限確認をすべてスキップする · 追加の引数 --add-dir=../shared · 再開する会話 0f3c2a9e-1b7d-4c55-9e…
  セッション名（任意） [ 例: ログイン不具合の修正          ]
  モデル（任意）       [ 既定のモデルを使う      ]
  権限モード           [ 既定（CLI に任せる）  ▾]
  推論の深さ           [ 既定（CLI に任せる）  ▾]
  ☑ 権限確認をすべてスキップする（危険）
  ☐ 後から「権限確認をすべてスキップ」を選べるようにする
  ☐ IDE に自動接続する
  ☐ 詳細ログを出す
  追加の引数（任意）     [ --add-dir=../shared                  ]
    チェックした項目の後ろに付けて起動します。値は --name=value の形で書きます。   (weak, small)
  再開する会話 ID（任意） [ 0f3c2a9e-…                          ]
  ほかのセッションの後に開始 [ すぐ開始する（既定） ▾]
─────────────────────────────── footer (unchanged layout)
  警告: 確認もサンドボックスもなしにファイルを変更できます。
  ☑ 理解したうえで起動する
  ☐ この設定を毎回使う
  [ターミナルだけ開く]                          [ Claude Code を起動  ⌘↩ ]
```

## Empty state

Both fields empty, hints shown: 「例: --add-dir=../shared --verbose」 and
「空なら新しい会話を始めます」. The command is unchanged from today.

## Error states (footer reason line, accent_soft, button disabled)

```
  会話を再開するときは依頼を空にしてください。再開後にターミナルで入力できます。
  追加の引数は - で始まるスイッチだけです（値は --name=value の形で書きます）: ../shared
  --model はこの画面の項目で指定してください。
  --dangerously-skip-permissions はこの画面の項目で指定してください。
  会話 ID には英数字とハイフンだけを、64 文字以内で入力してください。
  「新しいプロジェクトとして開始する」は会話の再開と同時に指定できません。
```

## Too long

A typed line wider than the field scrolls inside the single-line field. In the
header each value is cut to 24 characters with `…`
(`LAUNCH_DETAIL_SUMMARY_VALUE_MAX`), so a 1 024-character line is still a short
entry; `LAUNCH_DETAIL_SUMMARY_MAX` (3) caps the number of entries and folds a
fourth into 「ほか N 件」:

```
▾ 詳しい設定（任意） · 権限確認をすべてスキップする · 追加の引数 --add-dir=../shared --… · 再開する会話 0f3c2a9e-1b7d-4c55-9e… · ほか 1 件
```

## Custom agent

Neither row is drawn for 「別のコマンドを使う」, and switching agent clears the
conversation ID, so no hidden ID can block the launch.
