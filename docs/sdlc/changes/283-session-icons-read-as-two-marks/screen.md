# Screen: one mark per session, its state a dot on it

Tab strip, before and after:

```
Before:  (‖)[✳] セッション5 ×   (✓)[A] セッション2 ×   (⟳)[>_] セッション6 ×
After:   [✳]• セッション5 ×     [A]• セッション2 ×     [>_] セッション6 ×
              idle dot             done dot             running terminal: no dot
```

Left column, before and after:

```
Before                         After
⟳  ターミナル  セッション7       [>_]   セッション7
⟳  ターミナル  セッション6       [>_]   セッション6
(‖) [✳]  セッション5             [✳]•  セッション5
```

- The dot sits on the mark's lower right corner, ringed by the tab's or row's
  fill, in the state's colour (実行中 green, アイドル grey-blue, 要対応 amber,
  failed red, finished grey).
- 要対応: a larger amber dot with 「!」 in it.
- A custom command, which has no mark: its circled state mark, as today.
- Hover: the CLI name, the state word and its explanation.
