# Screen: change 226

- **Intent**: `./intent.md`
- **Status**: draft

The terminal area of the session workspace (the centre of `ui_terminal_workspace`),
about 900px wide at the default window with the session list on the left and the
views on the right.

## Before

```
┌─ terminal area ─────────────────────────────────── 900px ─┐
│ ✦ ログイン画面の修正  ● 実行中 · operon / ⑂ fix/login  ⋯ ■停止 │
│ ───────────────────────────────────────────────────────── │
│ ┌───────────────────────────────────────────────────────┐ │
│ │ $ claude                                              │ │
│ │ > ログイン画面のバリデーションを直して                    │ │
│ │                                                       │ │
│ │                                             [↓ 最新へ] │ │
│ └───────────────────────────────────────────────────────┘ │
└───────────────────────────────────────────────────────────┘
```

## After — dragging a session onto a terminal

A row of the session list — or, once two terminals are showing, a terminal's
header grip `⠿` — is dragged over a terminal. The part the session would take is filled with the accent and framed;
a hint follows the pointer. Each target terminal is cut like VS Code's: within
30% of an edge is that edge, the rest is the middle.

Pointer near the right edge → 「右に分割」:

```
┌─ terminal area ─────────────────────────────────── 900px ─┐
│ ✦ ログイン画面の修正  ● 実行中 · operon / ⑂ fix/login  ⋯ ■停止 │
│ ───────────────────────────────────────────────────────── │
│ ┌──────────────────────────┏━━━━━━━━━━━━━━━━━━━━━━━━━━━┓ │
│ │ $ claude                 ┃░░░░░░░░░░░░░░░░░░░░░░░░░░░┃ │
│ │ > ログイン画面のバリデ…   ┃░░░░░░░░░░░░░ ☚ 右に分割 ░░┃ │
│ │                          ┃░░░░░░░░░░░░░░░░░░░░░░░░░░░┃ │
│ └──────────────────────────┗━━━━━━━━━━━━━━━━━━━━━━━━━━━┛ │
└───────────────────────────────────────────────────────────┘
```

Pointer near the bottom edge → 「下に分割」; top 「上に分割」, left 「左に分割」:

```
│ ┌───────────────────────────────────────────────────────┐ │
│ │ $ claude                                              │ │
│ │ > ログイン画面のバリデーションを直して                    │ │
│ ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓ │
│ ┃░░░░░░░░░░░░░░░░░░░░░ ☚ 下に分割 ░░░░░░░░░░░░░░░░░░░░░░┃ │
│ ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛ │
```

Pointer in the middle → 「ここに表示」: the whole terminal is framed, and the
release replaces what it shows (the replaced session keeps running and stays in
the list). A terminal dragged by its grip onto another's middle trades places
with it instead, so nothing leaves the screen.

```
│ ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓ │
│ ┃░░░░░░░░░░░░░░░░░░░░ ☚ ここに表示 ░░░░░░░░░░░░░░░░░░░░░┃ │
│ ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛ │
```

## After — two terminals side by side

After the release on 「右に分割」. Each terminal keeps its own header. The one
holding the keyboard has an accent frame and drives the views on the right; a
click anywhere in the other moves both. The rule between them drags to resize.
`✕` (「画面から外す」) takes a terminal off the screen without stopping it, and
appears only while there are two or more.

```
┌─ terminal area ─────────────────────────────────── 900px ─┐
│ ⠿ ✦ ログイン画面の修正 ● 実行中 ⋯ ✕ ■ │ ⠿ ◆ レビュー ● 待機中 ⋯ ✕ ■ │
│ ─────────────────────────────── │ ─────────────────────── │
│ ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓ ┃ ┌─────────────────────┐ │
│ ┃ $ claude                    ┃ ┃ │ $ codex             │ │
│ ┃ > ログイン画面のバリデ…      ┃ ┃ │ > 差分をレビューして  │ │
│ ┃                             ┃ ┃ │                     │ │
│ ┃                  [↓ 最新へ] ┃ ┃ │         [↓ 最新へ]  │ │
│ ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛ ┃ └─────────────────────┘ │
└───────────────────────────────────────────────────────────┘
```

Four at most, as a 2×2 or any mix of rows and columns:

```
┌──────────────────────────┬────────────────────────────────┐
│ ⠿ ✦ 実装 ● 実行中  ⋯ ✕ ■   │ ⠿ ◆ レビュー ● 待機中  ⋯ ✕ ■     │
│ ┏━━━━━━━━━━━━━━━━━━━━━━━━┓ │ ┌────────────────────────────┐ │
│ ┃                        ┃ │ │                            │ │
│ ┗━━━━━━━━━━━━━━━━━━━━━━━━┛ │ └────────────────────────────┘ │
├──────────────────────────┼────────────────────────────────┤
│ ⠿ ▸ dev サーバー ● 実行中 ⋯ ✕ ■│ ⠿ ✦ テスト ○ 停止済み  ⋯ ✕ ▶     │
│ ┌────────────────────────┐ │ ┌────────────────────────────┐ │
│ │                        │ │ │                            │ │
│ └────────────────────────┘ │ └────────────────────────────┘ │
└──────────────────────────┴────────────────────────────────┘
```

## States

- **One terminal** (the default, and after every other terminal is taken off):
  exactly the Before screen — no `⠿`, no `✕`, no accent frame.
- **Clicking a session in the list**: it appears in the terminal holding the
  keyboard, replacing what that one shows — unless it is already on screen, in
  which case that terminal takes the keyboard instead.
- **Refused drop** — a fifth terminal, or a split that would leave a terminal
  narrower than 320px or shorter than 200px: the zone is framed in the warning
  colour, the hint says 「これ以上分割できません」, and a release changes nothing.
- **Dropping a terminal on itself, or a session on the terminal already showing
  it**: no zone is drawn for that terminal and the release changes nothing.
- **Dragging a session already shown elsewhere**: it moves; it is never shown twice.
- **Escape, or the pointer released outside the terminal area**: the drag ends,
  nothing moves. Released in the session list it still reorders the list as
  today.
- **A session removed from the list**: its terminal leaves the screen and its
  neighbour takes the room.
- **Too long**: titles truncate with `…` inside each header; the status chip,
  `⋯`, `✕` and the state verb keep their place on the right.
- **History open** (「履歴」): replaces the whole terminal area as today; the
  arrangement returns when 「ターミナルに戻る」 is pressed.
- **After a restart**: one terminal again, as the docked panels are not kept
  either.
