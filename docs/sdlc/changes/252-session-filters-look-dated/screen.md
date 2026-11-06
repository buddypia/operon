# Screen: Left sidebar filters, proposal A

- **Status**: approved

The person compared the current sidebar and three proposals as interactive HTML mockups (`.tmp/session-filters/`, not committed) and chose A, with 「承認・回答待ち」 folded into 「要対応」. Default column width 268px.

Before:
```text
プロジェクト                       [履歴]
CLI
[すべて] [Codex] [Claude] [AG] [その他]
ステータス
[すべて] [承認・回答待ち] [要対応]
[実行中] [指示待ち] [終了]
──────────────────────────────────
▾ design-lookbook                ⏸2 ＋
```

After, nothing filtered:
```text
プロジェクト                       [履歴]
┌⚠ 要対応        1┐┌⟳ 実行中        2┐
┌⏸ 指示待ち      3┐┌✓ 終了          1┐
(Codex|Claude|AG|⋯)
──────────────────────────────────
▾ operon                         ⚠1 ＋
```

After, 要対応 and 実行中 selected:
```text
┃⚠ 要対応        1┃┃⟳ 実行中        2┃   ← tone wash + tone hairline, tone count
┌⏸ 指示待ち      3┐┌✓ 終了          1┐
(Codex|Claude|AG|⋯)           3 件  ✕ クリア
```

- Tiles are hairlines at rest, filled under the pointer, and washed in their state's colour when selected. A zero count draws faint but stays clickable.
- Counts follow the CLI choice, so a number never promises rows the list will not show.
- CLI marks sit in one hairline segment; when any is selected the unselected marks fade. Hover and accessibility keep full names; ⋯ is 「その他」.
- 「n 件」 and 「クリア」 appear only while filtering; クリア clears both dimensions.
- 要対応's hover adds 「承認・回答待ち n 件」 under its existing description.
- Narrow column (below two readable tiles): tiles stack in one column.
- No match: 「一致するセッションはありません。」 as today. Empty store: 「プロジェクトがありません」 as today.
