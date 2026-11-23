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
