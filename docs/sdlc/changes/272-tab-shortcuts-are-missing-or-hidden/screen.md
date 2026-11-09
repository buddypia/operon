# Screen: Tab keys a Mac user brings with them

- **Status**: approved

No layout, colour, icon or size changes. Two existing lists gain rows drawn by
their existing code; one brief notice is new.

Settings → キー割り当て (the grid draws every `KEYMAP_ACTIONS` row; five appended):
```
検索・操作を開く              ⌘K
新しいセッション              ⌘N
…                             …
サイドパネルを開閉            ⌥⌘B
空のターミナルを開く          ⌘T
タブを閉じる                  ⌘W
閉じたタブを開き直す          ⇧⌘T
次のタブへ                    ⇧⌘]
前のタブへ                    ⇧⌘[
[キー割り当てファイルを開く]
```
Command palette, nothing typed (two rows appended after the existing actions):
```
新しいセッション                       ⌘N
…
キー割り当てファイルを開く
空のターミナルを開く                   ⌘T
閉じたタブを開き直す                   ⇧⌘T
```
A row whose action the keymap file set to `null` draws 「なし」 in Settings and no
chord in the palette, as today.

Sessions workspace, ⌘T (same result as pressing +):
```
│ [● セッション1 ×][● セッション2 ×][◌ ターミナル ×][+]
│                                   ━━━━━━━━━━━━━━━
```
⌘W on 「セッション2」, then ⇧⌘T: the tab returns at the end of the focused pane, selected:
```
│ [● セッション1 ×][◌ ターミナル ×][● セッション2 ×][+]
│                                   ━━━━━━━━━━━━━━━━
```
⇧⌘T with nothing to reopen: the existing toast area shows
「開き直せるタブはありません。」 for `NOTICE_TOAST_SECONDS`, nothing else moves.
