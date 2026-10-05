# The readiness gate

Read before entering stage 3, or when a No-Go needs interpreting.

```sh
bash scripts/check-readiness.sh docs/sdlc/changes/NNN-slug
```

`0` Go, `2` Conditional Go, `1` No-Go. Record the verdict in `state.yaml` as
`readiness`, and do not begin stage 3 on a No-Go.

## The question it answers, and the one it does not

> Can this be implemented by reading only these documents, without coming back
> to ask?

That is the right framing. But the script answers a
narrower version: **is the document filled in?** It cannot tell whether the design
is right, whether the requirements are the ones worth having, or whether the
approach will work. A Go is a floor, not an endorsement, and the script says so in
its own output for a reason — a checker mistaken for a quality verdict is worse
than no checker, because it ends the argument early.

The part it cannot do is the part to do yourself, with the spec open:

- Is each requirement independently checkable? A requirement nobody can fail is
  not a requirement.
- Does the **Behaviour** section cover the states that are not the happy one —
  empty, absent, interrupted, denied, too large, concurrent?
- Would the **Design** section let someone else write `plan.md` without asking?
- Is a **Rejected alternative** missing, such that the same argument will be had
  again in review?

## What it blocks on

| Finding | Why it blocks |
|---|---|
| a template placeholder in angle brackets | the section was never written |
| `TODO`, `TBD`, `FIXME`, `XXX` | a decision was deferred past the stage that owns it |
| `Status:` still carrying the template's `draft \| approved \| superseded` | nobody chose |
| an empty cell in the **Policy conformance** table | the template ships every row empty, so an untouched table reads, in a diff, exactly like a table |
| a **flagged concern** with no body | the pipeline's rule is that concerns are answered before stage 3, not carried into it |
| a missing mandatory section | |
| no numbered requirements, or an empty **Acceptance** | nothing says what done means |
| no `intent.md`, or an empty **Desired outcome** | there is no upstream to check the spec against |

## Backticks are stripped first

Committed specs legitimately contain `Vec<Range<usize>>`, `<uuid>`,
`<provider>/unparsable-line`, and `<stages> <attempts> <gate>` — all code or
shapes being *discussed*. The checker removes every backticked span and fenced
block before it looks for anything. A name being discussed is not a name being
pointed at; lesson 003 established that rule for paths and it holds here.

If a real placeholder slips through because it was written inside backticks, that
is the trade, and it is the same trade `harness_documents_only_name_paths_that_exist`
makes.

## Reading a No-Go

Fix it in the spec. The failure mode this gate exists to prevent is answering a
document problem in the implementation, discovering at stage 4 that the answer
was wrong, and having no record of the question.

If a finding is wrong — the checker is over-firing on legitimate prose — that is a
defect in `scripts/check-readiness.sh` and it gets fixed there, with the corpus of
committed specs as the test. The first cut of the flagged-concern check demanded
the literal word "Answer" and rejected four already-committed specs that answer
their concerns in ordinary prose. A checker that is wrong about the documents it
ships with is a checker nobody runs.

## Conditional Go

Warnings only. Proceed, and carry the warning into `state.yaml`'s `resume` so it
is not rediscovered. The common one is a missing `state.yaml`, which is a warning
rather than a block because it costs the *next* session, not this one.
