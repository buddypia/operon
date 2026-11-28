# docs/features — the product's description, for sessions

What Operon is, written so a session can change it without reading all of
`src/` first, and drawn by `python3 scripts/change-map.py` as the product map.

| File | Holds |
|---|---|
| `index.yaml` | every feature: id, title, screen, domain, summary |
| `<id>/brief.md`, `<id>/spec.md` | a feature for a person (Japanese) and for a session (English) |
| `domains.yaml` | the domains, what each owns, and its modules |
| `glossary.yaml` | the words the product uses, and the words it avoids |
| `flow.yaml` | every screen and the ways between them |
| `../images/screens/` | a picture of each screen, and `screens.json` |

## The map's views

The page opens on 現況 and keeps a list of every feature down the left of every
view (`j`/`k` step through it on a feature's page). The bar, in order:

| View | Shows |
|---|---|
| 現況 | features by state, where every open change is among the stages, what waits for the person, what is under way and its next step, each domain, each branch |
| 機能 | the features as tiles, narrowed by chips (状態, テスト, ドメイン) whose choice is in the link |
| 画面 | the pictures of the screens, walked by the ways between them; 全体図 draws them all |
| ドメイン | what each domain owns and excludes, its code, features and screens |
| 用語集 | the words the product uses and avoids |
| 要件 | every requirement line as a card, narrowed by domain, feature and test |
| 作業 | the backlog, what a session holds, the changes under way, what waits for the person, what landed lately |
| 変更 | every change record |
| 進め方 | the pipeline: routes across the stages, the files that make a session stop and ask, the evidence methods |
| カタログ | the application's colour roles in the three themes, its marks and its keys |

A feature's page carries its rail (概要, 仕様, 画面写真, 要件, テスト, 実機確認),
its sections, its code by layer and the evidence its landed changes rest on,
and a bar for a verdict (OK, 要修正, 保留) and a note. The basket, the verdicts
and the theme are kept in the browser only, under names beginning
`operon-map-`; the copied texts are what reach a session.

Two options:

- `--backlog` — also read the backlog (change 323) through
  `scripts/backlog.py`, within `--backlog-seconds` (default 30); without it the
  作業 view says the backlog was not read.
- `--serve` — serve the page on `127.0.0.1` only, at `--port` (default 8765, 0
  for any free one), built again from the documents on every load; Ctrl-C
  stops it.

## Pictures of the screens

The map's 画面 and 全体図 views show each screen as the application draws it.
The pictures are taken from the running application by
`scripts/capture-screens.py` and committed (change 316).

In `flow.yaml`, each screen carries exactly one of:

- `reach` — the words to press, `;`-separated, from a fresh launch with the
  demo project (`verify-demo`), that bring the screen up; `-` for the first
  window. Each word is pressed by the rule of `scripts/verify-app.py`: the
  control whose name is exactly the words, else the only one containing them.
- `noshot` — a Japanese reason the screen cannot be photographed (it is not a
  window, or nothing a press reaches opens it).

And each way may carry:

- `press` — the words of the control the way presses, when they differ from
  its `label`; `words #n` names the n-th of several controls answering to the
  same words. The map places a pressable area on the picture where the
  control is.
- `unseen` — a Japanese reason the way has no control on the picture (it shows
  only in a state the demo project does not have, or its button has no name).

### Retaking them

A change that alters what a screen shows, adds a screen, or adds or renames a
way updates `reach`, `press` or `unseen` in `flow.yaml` and retakes the
pictures in the same branch:

```
python3 scripts/capture-screens.py --plan          # what would be pressed; starts nothing
python3 scripts/capture-screens.py                 # every screen (about 10 s each)
python3 scripts/capture-screens.py --only home palette --no-build
```

It builds `target/debug/operon`, launches it apart from the person's home and
tmux once per screen, presses the words, and writes `<id>.jpg` and its entry in
`screens.json` (window size, controls and their bounds, words, time, commit).
The window shows on the screen while it runs; the pointer is not moved. It
needs the accessibility and screen-recording permissions `agent-desktop`
asks for. Exit 0 is every screen taken, 1 is some screen not taken (its earlier
picture is kept, and the screen and word are named), 2 is that it could not
start. Then `python3 scripts/change-map.py` and look at 画面.
