# Agreeing the screen before the code exists

Read when `docs/sdlc/risk.yaml` puts the change on a `screen yes` surface — it
touches `src/app.rs`, `src/ui/`, `src/theme.rs`, or `src/glyphs.rs`. Set
`screen: pending` in `state.yaml` and do not enter stage 3 until it reads
`approved`.

## Why this gate and not a test

Operon is an immediate-mode GUI, and its layout bugs are the ones tests are worst
at. Lesson 005: a footer widget was written to fix an overlap, had a test pinning
its two rects apart at two widths, and the modal never called it. The widget test
was green for the entire time the overlap was shipping. **A widget test proves a
widget; only the caller proves the screen, and only an eye on the rendered
screen proves the layout** — a person's for a new structure, the evaluator's for
a small change to one already agreed. A layout agreed in ASCII costs a message;
discovered wrong after the code, it costs the code, roles, tests, and strings.

## What to present

An ASCII layout of the screen at a realistic width, with the states that are not
the happy one. `AskUserQuestion`'s `preview` field renders monospace and puts two
options side by side, which is what a Before/After wants.

```
┌─ セッション復元 ────────────────────── 520px ─┐
│                                               │
│  Claude Code → Codex                          │
│  532 / 1 165 件を復元しました                  │
│                                               │
│  ▸ 分類されなかった 12 件を表示                │
│                                               │
│              [ バックグラウンドで続ける ]      │
└───────────────────────────────────────────────┘
```

Include, because these are where layout goes wrong:

- **The real width.** A modal is 520px here; a sentence that fits at 900 claims
  the row at 520. That is lesson 005's actual mechanism.
- **The Japanese strings**, not English placeholders. Japanese runs shorter than
  English and wraps differently, and the string is what occupies the row.
- **Empty, loading, error, and too-long** as separate frames. The happy state is
  the one that was already imagined.
- **Before and After** when the change modifies an existing screen. A diff of the
  layout is easier to judge than the layout.

## Who decides, and the three-way verdict

Commit the layout as `screen.md` in the change directory, so a verdict can be
bound to its digest. Then `references/approval.md` decides who judges it: a new
screen or modal, a re-arranged layout, or a feature removed or moved goes to a
person; wording, spacing, and colour or state inside existing roles goes to the
evaluator, which after the build judges a screenshot against `screen.md` too.
Either way the verdict is a line in `approvals.log`, and one of:

- **approve** — set `screen: approved`, proceed to stage 3.
- **revise** — take the note, redraw, ask again. **Three revisions maximum**, then
  `awaiting-user`: the disagreement is about what the screen is for, which is a
  spec question.
- **reject** — set `status: blocked` and record why in `resume`.

Silence is not approval. The colour and glyph rules and the geometry tests still
apply while the code is written; the drawing surface is `low` risk because a
wrong layout costs rework, not corruption.
