# Handoff: <what is not finished>

- **Route**: <the route from `docs/sdlc/routes.yaml`>
- **Attempts spent**: <n> of <the route's ceiling>
- **Written**: YYYY-MM-DD
- **Status**: open | resolved | abandoned

Written when a route's attempt ceiling in `docs/sdlc/routes.yaml` is reached.
This file is how the search survives a context window ending. Delete none of the
sections: a handoff with an empty **What was ruled out** is the failure it exists
to prevent, because the next attempt then repeats the first.

## The failure, exactly

The command, and its output. Not a paraphrase — the bytes. If it is a test, the
test name and the assertion message. If it is a gate, which of the three and the
line it stopped on.

```
```

## What was tried, in order

One line per attempt: what was changed, what was expected, what happened. The
value here is the *shape* of the search, so an attempt that seemed obviously
wrong in hindsight still belongs on the list.

1.
2.
3.

## What was ruled out

The hypotheses that are now dead, each with the observation that killed it. This
is the section the next attempt reads first, and the only one that makes the
handoff cheaper than starting over.

| Hypothesis | Ruled out by |
|---|---|
| | |

## Narrowest remaining hypothesis

One sentence, stated so that it is falsifiable, plus the single observation that
would confirm or kill it. If there is no such sentence, say that instead of
inventing one — "no hypothesis survives" is a real and useful answer.

## What a person has to decide

The question that cannot be answered from inside the repository. Name it plainly.
This is why the session stopped rather than continued.

## State of the tree

- Branch, and whether the work is committed.
- Which files are modified and not committed.
- Whether the three gates pass as the tree stands: `cargo fmt --check`,
  `cargo test --locked`, `cargo clippy --locked -- -D warnings`.
- Anything left behind that must be cleaned up — a stray fixture, a `dbg!`, an
  `#[ignore]` added to see the rest of the suite run.

## If this becomes a lesson

Name the guard here once it is written, and set the status to `resolved`.
Step 6 of `.claude/skills/sdlc/SKILL.md` owns the rest.
