# The readiness gate

Read before entering stage 3, or when interpreting a No-Go:

```sh
bash scripts/check-readiness.sh docs/sdlc/changes/NNN-slug
```

`0` Go, `2` Conditional Go, `1` No-Go. Record in `state.yaml` as `readiness`, and
do not enter stage 3 on a No-Go.

## What the script checks vs. what you check

The script checks that the document is filled in; you check that the design is
sound:

- Each requirement is independently checkable and falsifiable.
- **Behaviour** covers empty, absent, interrupted, denied, too-large, and
  concurrent states.
- **Design** is complete enough to write `plan.md` without asking.
- **Rejected alternatives** are recorded so review does not re-argue them.

## What it blocks on

| Finding | Why it blocks |
|---|---|
| angle-bracket template placeholder | section was never written |
| `TODO`, `TBD`, `FIXME`, `XXX` | decision deferred past its stage |
| `Status:` still `draft \| approved \| superseded` | nobody chose |
| empty cell in **Policy conformance** | table row left unchecked |
| **flagged concern** with no body | concerns must be answered before stage 3 |
| missing mandatory section, requirements, **Acceptance**, or `intent.md` | upstream/completion contract missing |

Backticked spans and fenced code blocks are stripped before scanning so code
shapes like `Vec<Range<usize>>` do not trip placeholder detection.

## Reading a No-Go or Conditional Go

- **No-Go**: fix `spec.md` (or fix `scripts/check-readiness.sh` if it over-fired
  on valid prose, tested against committed specs).
- **Conditional Go**: warnings only; record the warning in `state.yaml`'s
  `resume` and proceed.
