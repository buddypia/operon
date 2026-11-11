# Screen: a Home session row leads with a mark, not a word

最近のセッション, about 700px, before and after:

```
Before
│ ‖ アイドル  ログイン画面の修正   Claude  operon  ⑂ feature/login     3分前  [開く] ··· │
│ ⟳ 実行中    検索の高速化         Codex   operon  ⑂ feature/search    8分前  [停止] ··· │
│ ! 要対応    とても長いタイトルは途中で切…  Claude  operon           1時間前 [開く] ··· │
│ ✓ 終了      テスト追加           Claude  operon                      昨日   [再開] ··· │

After
│ (‖) ログイン画面の修正   Claude  operon  ⑂ feature/login             3分前  [開く] ··· │
│ (⟳) 検索の高速化         Codex   operon  ⑂ feature/search            8分前  [停止] ··· │
│ (!) とても長いタイトルは途中で切れずに少し長く…  Claude  operon      1時間前 [開く] ··· │
│ (✓) テスト追加           Claude  operon                              昨日   [再開] ··· │

Pointer resting on (‖):
  ┌──────────────────────────────────┐
  │ アイドル                         │
  │ ターンを終え、次の依頼を出せます │
  └──────────────────────────────────┘
```

- The mark is the circled generated state mark the left column uses, in the
  state's colour; the word after it is removed, nothing else moves except
  that the title starts further left and gets the freed width.
- The same row under 要対応 on Home and on a project's page changes the same
  way.
- No list is empty, loading or failing differently: those states draw no row.
