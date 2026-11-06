# Session tab state cues

- **Status**: approved

Existing horizontal scrolling tab strip, about 760 logical pixels at the screenshot width.

Before: [○ design-lookbook · Antigravity ×] [◌ design-lookbook · Antigravity ×]
After:  [pause-circle 指示待ち design-lookbook · Antigr… ×]
        [circular-arrows 実行中 design-lookbook · Antigr… ×]
        [check-circle 終了 design-lookbook · Antigravity ×]

The lines above describe adjacent tabs in the same horizontal strip, not stacked rows. Only state within the existing strip changes. State text is 12.5px in the current status tone, title remains 160px and truncates, hover keeps its explanation.

Empty: no tabs. Loading: power 実行中 or question 実行中 according to the existing state mapping. Error: cross-circle 終了; hover 異常終了。出力を確認してください. Long: the state remains intact before a truncated title. Permission question: attention 要対応. Interrupted: stop 終了. Unknown result: minus-circle 終了, with the current missing-terminal hover.
