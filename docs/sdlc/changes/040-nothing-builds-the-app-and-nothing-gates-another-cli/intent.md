# Intent: nothing builds the app, and outside Claude Code nothing gates anything

- **Status**: draft
- **Opened**: 2026-09-10

## Problem

Two gaps with one root: what this repository checks is a property of the profile
cargo happened to compile and of the CLI the person happened to open — not a
property of the repository.

**The app is never built.** The three gates are `cargo fmt --check`,
`cargo test --locked`, and `cargo clippy --locked -- -D warnings`. All three
compile the debug profile. `Cargo.toml` sets the release profile to `lto = true`,
`codegen-units = 1`, `strip = true`: a whole-program link-time optimisation over
a single codegen unit, which is a different compile and a different link from
anything the gates run. The only place it happens is inside
`scripts/package-macos.sh`, by hand, at release time. `AGENTS.md` says "After
every change, package the app and atomically replace /Applications/Operon.app".
That is a promise in prose, and `.claude/hooks/gate-commit.sh` opens by
explaining why a promise in prose is not enough — "a green suite the agent never
ran looks exactly like a green suite it did" — about these same three gates. The
sentence applies to the packaging line it sits beside.

The size of it, honestly: `cargo test --locked` does compile the `operon` binary
target, so an ordinary compile error cannot reach a commit. What is unchecked is
the release profile itself — an LTO link failure, a `debug_assertions`
difference, an arithmetic overflow check that is on in debug and off in release —
and everything `scripts/package-macos.sh` does around it. A narrow gap, and the
kind that surfaces at the worst moment, which is when someone is trying to ship.

**Outside Claude Code, nothing runs.** `.claude/settings.json` is Claude Code's
file. `.claude/hooks/guard-write.sh`, `.claude/hooks/guard-bash.sh`,
`.claude/hooks/gate-commit.sh` and `.claude/hooks/gate-stop.sh` fire only there.
A Codex CLI session that edits Rust and commits reaches exactly one gate:
`.githooks/reference-transaction`, which judges the commit's record against its
diff and runs no cargo at all. From Codex, fmt, test, clippy and the test-erosion
count are not skipped — they were never installed.

And that one remaining gate is itself installed by Claude Code.
`.claude/hooks/gate-commit.sh` copies `.githooks/installed/reference-transaction`
into the repository's hooks directory before every commit, and its header says a
fresh clone gets the trampoline "at its first commit through Claude Code, or by
hand". A clone only ever worked in from Codex has no review gate either, and
nothing anywhere reports that it is missing.

## Who feels it, and when

- Whoever runs `.claude/skills/ship/SKILL.md`. The release build is the first
  time the release profile has been compiled since the previous release, so an
  LTO failure introduced weeks earlier surfaces during the release rather than
  during the change that caused it.
- Whoever opens this repository in Codex CLI. Every gate thirty-nine changes went
  into building is off, silently, and the session cannot notice: a transcript
  with no gates in it looks exactly like a transcript whose gates all passed.
- The owner, who currently has to know which CLI a change was made in before
  knowing what was checked.

## Desired outcome

- A session that changed Rust cannot finish without the release build having
  succeeded on that tree, and a failure is reported rather than passed over.
- A session that changed only documents, translations, or comments pays none of
  it.
- What the gates check does not depend on which CLI opened the repository. A
  commit from Codex CLI, from a plain terminal, or from a CLI nobody has written
  yet passes the same checks a Claude Code commit does, or is refused.
- Which gate ran at which boundary, and what it skipped, is visible afterwards
  rather than inferred from the transcript.

## Constraints this change inherits

- `.claude/settings.json`, `.claude/hooks`, and `.githooks` are the
  `gate-configuration` surface in `docs/sdlc/risk.yaml`: high risk, `paused`. A
  change here can switch off every other row.
- Change 035 holds that surface. It is `blocked` at attempts 22/2 with reviewers
  reading a fixed digest, and 038 and 039 are both `awaiting-user` behind it.
  Nothing in this change may edit `.claude/hooks`, `.githooks`,
  `.claude/settings.json` or `scripts/check-review.sh` until 035 lands or fails.
  `docs/sdlc/changes` is excluded from the review gate's digest, which is what
  makes this directory safe to write now.
- A release build of this crate is minutes, not seconds. The commit gate's
  existing timeout in `.claude/settings.json` is 600 seconds for fmt, test and
  clippy together, and `.claude/hooks/gate-commit.sh` already warns that a cold
  cache can exceed it.
- Codex CLI reads `AGENTS.md` and has no per-tool hook mechanism of Claude Code's
  kind. Git's own hooks are the only enforcement point the two CLIs share.
- Local-first, and this repository has no remote: `.github/workflows/ci.yml` has
  never run and there is no push to gate at. The boundaries that exist are edit,
  commit, and session end — and session end exists only in Claude Code.

## Systems likely affected

`.claude/hooks/gate-stop.sh` and `.claude/hooks/gate-commit.sh`;
`.githooks/reference-transaction` and the trampoline in
`.githooks/installed/reference-transaction`; a new script under `scripts`;
`.claude/settings.json` for the timeout; `AGENTS.md` and `CLAUDE.md` for the
table of healthy output; `docs/sdlc/routes.yaml` and `REVIEW.md` if the list of
gates changes; and `src/tests.rs` for the guards — including
`every_document_that_names_the_gates_names_the_same_three`, whose name stops
being true if a fourth gate joins the sentence.

## Open questions

The owner answered four at the start, recorded in `state.yaml`. These are what
`spec.md` has to settle:

1. **Two of those answers are in tension.** The build gate is to run at session
   end, and coverage is to be CLI-agnostic. Claude Code's `Stop` has no
   counterpart in Codex, so a Stop-only build gate covers exactly the CLI that is
   already covered. The likely resolution is that `cargo build --release --locked`
   is idempotent — the cost is one build per tree state, not one per boundary — so
   one script can serve both the Stop boundary and the git boundary and the second
   run costs seconds. Whether that holds for this crate needs measuring rather
   than assuming, given `lto = true` and `codegen-units = 1`. **Answered by**:
   `spec.md`, with a measurement.
2. **Which git hook.** `pre-commit` is the natural place, and `--no-verify` skips
   it. `.githooks/reference-transaction` cannot be skipped that way, which is why
   the review gate lives there — but it runs inside the ref update, for every ref
   update including a fetch or a reset, and a multi-minute build there is a
   different proposition from a text check. **Answered by**: `spec.md`.
3. **What counts as "not code".** `.claude/hooks/gate-commit.sh` already carries a
   classifier for the cargo gates. Whether the build gate reuses it unchanged, and
   whether the assets and plist inputs `scripts/package-macos.sh` reads belong on
   the build side of the line. **Answered by**: `spec.md`.
4. **Whether "three gates" becomes "four".** Five documents name the same three,
   and `src/tests.rs` has a guard keeping them in agreement. A gate that runs at a
   different boundary from the other three may not belong in that sentence at all.
   **Answered by**: `spec.md`.
5. **What a refused Codex session is actually told.** A Claude Code hook returns a
   message into the transcript. A git hook writes to stderr, and whether Codex
   surfaces it legibly is an observation, not a deduction. **Answered by**: a
   person at the machine.

## Not in scope

- Packaging and the `/Applications` swap. The owner scoped the build gate to
  `cargo build --release --locked`; `scripts/package-macos.sh` and
  `scripts/replace-macos-bundle.sh` stay where they are, run by
  `.claude/skills/ship/SKILL.md`.
- Making `.github/workflows/ci.yml` run. That is the open question in change 038.
- Any change to what the three existing gates check. This adds a boundary and a
  fourth check; it does not touch the three.
- Codex-specific configuration. This change adds nothing under a Codex directory —
  the shared enforcement point is git, and `AGENTS.md` is the shared document.
