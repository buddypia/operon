# Spec: four failures the harness could not stop from recurring

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

Numbered, each one independently checkable.

1. `REVIEW.md` names the mutation-coverage rule: a mutation set that touches a
   sentence a person reads must mutate the **claim**, not only the counters that
   feed it. `the_review_policy_names_every_policy_the_repo_enforces` fails if the
   rule is dropped.
2. `.claude/skills/sdlc/SKILL.md`'s close-the-loop step carries the same rule, so
   a session running the pipeline meets it before review does.
3. `evals/007-*` exercises the rule against a real agent: given a change that
   adds a user-facing sentence and a mutation set that only moves counters, the
   agent adds a mutation of the sentence. `every_eval_*` format checks apply to
   it, and `scripts/pipeline-indicators.sh` with `--lessons` accepts it as the guard for
   lesson 041.
4. `.claude/hooks/gate-stop.sh` compares the tree it resolved against the
   repository its own file belongs to, and when they differ says so as the first
   thing in its output, naming both. When they agree it says nothing extra.
5. Requirement 4 holds for the three resolution paths the hook already has — the
   payload's `cwd`, the inherited repository pointers, and `CLAUDE_PROJECT_DIR` —
   and is asserted against a fixture where hook and tree genuinely differ.
6. `REVIEW.md` distinguishes three reviewer states and gives each a remedy:
   running (wait or stop, not both), finished-and-reported, and
   finished-and-silent. The silent one is re-asked by name; it is not replaced,
   because a replacement produces a second verdict at the same digest.
7. `scripts/package-macos.sh` writes the source commit into the bundle's
   `Info.plist` under a key spelled once, in the script, and read by nothing that
   spells it again.
8. `scripts/check-installed-build.sh` exits 0 when the installed bundle was built
   from a commit at or after the newest commit touching release-binary sources,
   1 when it is behind, and says which commit each side is at. An unstamped
   bundle is reported as unknown, not as stale: every bundle installed before
   this change is unstamped and calling those stale would make the check noise
   from the day it lands.
9. `.claude/hooks/gate-stop.sh` runs requirement 8's check and reports a stale
   installed build alongside the contract items it already reports.
10. `docs/sdlc/templates/state.yaml` carries a `release-binary` contract row, so
    the question "does this change make the installed app stale" is answered
    while the contract is written rather than after the commit.
11. `docs/sdlc/lessons.md` gains one entry covering all four, with a Guard
    paragraph naming what now catches each.

## Behaviour

Nothing in the application changes; every output here is read by an agent or by
a person at a terminal.

**Requirement 4, the mismatched-tree line.** Japanese, like the rest of
`gate-stop.sh`'s refusal, and first in the output because it changes how
everything under it should be read:

> このフックは別のチェックアウトにあります。フック: `<hook repo>` / 対象ツリー:
> `<resolved tree>`。以下の指摘は対象ツリーのものです。フックを直すには、その
> チェックアウトを作業ディレクトリに持つセッションが要ります。

**Requirement 9, the stale-build line.** Also Japanese, appended to the existing
report rather than replacing it:

> インストール済みの `/Applications/Operon.app` は `<installed>` で作られており、
> リリースバイナリに触れた最新のコミットは `<head>` です。`bash
> scripts/package-macos.sh` で作り直して入れ替えてください。

Unstamped is a different line and not a refusal:

> インストール済みバンドルにソースコミットの刻印がありません（この変更より前に
> 作られたバンドルです）。次回のパッケージ化で刻印されます。

`scripts/check-installed-build.sh` run by hand prints the same facts in English,
because it is a script an agent reads.

Not-the-happy-path states, in order of how likely they are here:

- No installed bundle at all — exit 0 and say so. A machine that has never
  installed the app is not behind.
- `git log` finds no commit touching release-binary sources — exit 0. A fresh
  repository has nothing to be stale against.
- The stamped commit is not in this repository's history (installed from another
  branch, or from a commit since rewritten) — exit 1 and say the stamp is
  unknown here rather than pretending to compare.
- The hook runs where `git` is absent or the tree is not a repository — the hook
  already exits 0 in that case and continues to.

## Design

Four independent pieces. None of them shares code with another, which is the
argument for one change: they share a shape, not an implementation, and four
change directories would each carry the same authorisation and the same lesson.

**1 and 2 — steering.** A paragraph in `REVIEW.md`'s passes, a bullet in
`.claude/skills/sdlc/SKILL.md`'s step 10, and a row in
`the_review_policy_names_every_policy_the_repo_enforces`'s required table.

**3 — the eval.** `evals/007-mutate-the-claim-not-the-counter.md`, seeded by
lesson 041. Setup writes a small change directory whose `mutations.py` moves
counters only, against a function whose notice makes a claim. The prompt asks the
agent to verify the guards by mutation. The check requires the resulting
`mutations.py` to contain a mutation whose replaced text lies inside a user-facing
literal.

**4 and 5 — the hook says where it is.** `gate-stop.sh` already resolves
`working_tree`. It gains one more resolution: the repository root of its own
file, via `dirname "$0"` and `git rev-parse --show-toplevel`. Two roots, compared
with `[ "$a" != "$b" ]`, and one printf. `$0` is used rather than
`CLAUDE_PROJECT_DIR` because the variable is the thing under suspicion.

**7, 8 and 9 — the stamp and the check.** `package-macos.sh` gains
`/usr/libexec/PlistBuddy -c "Add :OperonSourceCommit string <sha>"` against the
staged `Info.plist`, with the sha from `git rev-parse HEAD`. The key is spelled
once in that script; `check-installed-build.sh` reads the key name **out of
`package-macos.sh`** rather than repeating it, which is
`.claude/rules/identifiers.md` applied to a shell pair — the same shape as
`gate-stop.sh`'s stamp path being read out of the hook by its test.

Release-binary sources are `src/` excluding `src/tests.rs`, plus `Cargo.toml` and
`Cargo.lock`. The exclusion is the whole reason 065 and 066 did not need
repackaging, and it is spelled once, in `check-installed-build.sh`.

**10 — the contract row.** One row in `docs/sdlc/templates/state.yaml`, with the
closed set it is checked against unchanged.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | Nothing paints. No `src/theme.rs`, no `src/ui/`, no drawing path is touched. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No | No glyph is added. The two new hook lines are prose. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | **Yes** | `OperonSourceCommit` is spelled in `scripts/package-macos.sh` only; `scripts/check-installed-build.sh` reads it out of that script, and the test reads it out of the script too rather than typing it. `src/config.rs` is for strings the crate spells; this pair is two shell scripts, so the rule applies in its general form — the value lives in one place and the readers refer to it. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | **Partly** | No store shape changes and no migration is needed. `scripts/package-macos.sh` is on the `bundle-swap` surface the durability reviewer covers, so the diff goes to that reviewer: the stamp is written into the staged bundle before the atomic swap, never into the installed one, so a failed package leaves the installed app untouched. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | **Partly** | No Rust child processes are added. The new shell script spawns `git` and `/usr/libexec/PlistBuddy`; the new test spawns the script under `/bin/bash`, as the existing hook tests do, and collects results before removing its fixture so a red does not leak a repository into `$TMPDIR`. |
| Documentation — user-facing docs change in all three languages together | No | `README.md` and its two translations are untouched. `REVIEW.md` and the skills are agent-facing and single-language by policy. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | **Yes** | Everything reads the local repository and the local bundle. The eval calls a model, like the six evals already here, and only when `scripts/run-evals.sh` is invoked by hand or by CI with a key. No new dependency. |
| Budgets — any new scan or output path states its byte and item ceiling | **Yes** | `check-installed-build.sh` runs two bounded `git` commands and one `PlistBuddy` read; it does not scan a tree. The hook's added output is two fixed lines. `steering_bytes` will rise and is addressed under Acceptance. |

## Flagged concerns

- **`steering_bytes` is already in the `diagnose` band and this change adds to
  it.** Resolved, and the resolution is to do nothing to the band: `bands.yaml`
  says a baseline edited to match today is not a baseline, and that when the
  band is reached by lessons alone the answer is a format change rather than a
  re-base. This change records the new reading and the investigation, as change
  061 did. If it crosses `propose` (316000) the band's own prescription applies
  and that is a separate change with its own `intent.md`.
- **Requirement 4's fix does not take effect for any session until main
  merges.** Not resolvable here and not a reason to withhold it: the same was
  true of change 065, which is the change that discovered the defect. Recorded
  in `/tmp/operon-merge-handoff.md`.
- **The eval cannot be run in this session.** `scripts/run-evals.sh` needs an
  API key. Resolved by stating it plainly rather than claiming a pass: the eval
  is format-checked by the suite, and its Check block is verified by hand
  against a tree that satisfies it and one that does not.

## Acceptance

- `cargo fmt --check` silent, `cargo clippy --locked -- -D warnings` silent.
- `cargo test --locked` passes, including
  `the_review_policy_names_every_policy_the_repo_enforces` with its new rows,
  `the_stop_gate_says_when_it_is_reading_another_checkout`, and
  `the_installed_build_check_reads_its_stamp_key_out_of_the_packager`.
- `bash scripts/check-installed-build.sh` exits 0 and reports the installed
  bundle as unstamped, because the bundle on this machine predates the stamp.
- `bash .claude/hooks/gate-stop.sh` run by hand from this worktree prints the
  mismatched-tree line when given a hook path from another checkout, and does
  not print it when hook and tree agree.
- `bash scripts/pipeline-indicators.sh --lessons` reports every entry with a
  guard, including the new one, whose Guard names `evals/007-*` among others.
- `bash scripts/check-bands.sh` runs; the `steering_bytes` reading is recorded
  in `state.yaml` with what moved.
- Each new guard watched failing, via `mutations.py` in this change directory.

## Rejected alternatives

- **Four change directories, one per failure.** Same route, same authorisation,
  same lesson, and four copies of each. The four share a shape and no code; one
  directory records the shape once.
- **Enforcing the mutation rule in `.claude/hooks/gate-commit.sh`.** The gate
  would have to read the diff and the mutation script together and decide what
  counts as a claim. A wrong answer there refuses commits for a heuristic, on
  the surface that can switch off every other gate.
- **A `mutations.py` header field declaring which claims were mutated.**
  Bookkeeping inside the script that is skipped exactly when it matters, and it
  would need backfilling into five existing scripts that were correct without it.
- **Making the Stop gate refuse when it is reading another tree.** It would
  refuse every worktree session on this machine until the merge lands, which is
  a worse failure than the one being fixed. Saying so is enough; the whole cost
  was not knowing.
- **Stamping the version instead of the commit.** `CFBundleShortVersionString`
  is `0.1.0` and has not moved; a stamp that does not move cannot answer
  "is this build behind".
- **Comparing the installed binary's bytes against a fresh release build.**
  Exact, and it costs a full optimised build inside a Stop hook.
- **Fixing `scripts/check-readiness.sh`'s No-Go for every bugfix here.** It is
  the gate this change's own route runs through. Recorded in change 061's
  `state.yaml` and left for its own change.
