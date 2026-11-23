# Agreeing the screen before the code exists

Read when `docs/sdlc/risk.yaml` puts the change on a `screen yes` surface
(`src/app.rs`, `src/ui/`, `src/theme.rs`, or `src/glyphs.rs`). Set
`screen: pending` in `state.yaml` and do not enter stage 3 until `approved`.

## Why this gate

Operon is an immediate-mode GUI: a widget test proves a widget, only the caller
proves the screen, and only an eye on the rendered layout proves the geometry
(lesson 005). Agreeing the layout in ASCII before writing code avoids reworking
roles, tests, and strings.

## What to present

Commit an ASCII layout at a realistic width as `screen.md` in the change
directory:

```
┌─ セッション復元 ────────────────────── 520px ─┐
│  Claude Code → Codex                          │
│  532 / 1 165 件を復元しました                  │
│  ▸ 分類されなかった 12 件を表示                │
│              [ バックグラウンドで続ける ]      │
└───────────────────────────────────────────────┘
```

Include:
- **Real width** (e.g., 520px for a modal).
- **Japanese strings**, not English placeholders.
- **Empty, loading, error, and too-long** states alongside the happy state.
- **Before and After** when modifying an existing screen.

## Who decides

Per `.claude/skills/sdlc/references/approval.md`: a new screen/modal, layout
re-arrangement, or removed/moved feature goes to a person; wording, spacing, or
state inside existing roles goes to the evaluator. Record the verdict in
`approvals.log`:

- **approve** — set `screen: approved`, proceed to stage 3.
- **revise** — redraw and re-evaluate (maximum three revisions, then `awaiting-user`).
- **reject** — set `status: blocked` and record why in `resume`.

## After it is built

The drawing agreed above is a promise about a screen nobody had seen. Once the
code exists, run the feature in the built application and compare the pictures
with `screen.md`: `.claude/skills/feature-pilot/references/verify-in-the-app.md`.
A picture that departs from the agreed layout is a finding, fixed or taken
back to whoever approved the screen.
