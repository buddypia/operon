# Plan: the release gate checks its three conditions instead of asking about them

- **Spec**: `./spec.md`
- **Approved**: 2026-09-06
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `scripts/check-release-preconditions.sh` | new. The four conditions, fail-closed, output capped |
| `.claude/hooks/guard-bash.sh` | the `replace-macos-bundle.sh` branch consults the script; `decide ask` keeps its two-space indent so `gated_commands` still counts it |
| `docs/sdlc/README.md` | stage table row 5 and the environment-tiers table say the gate checks; the paragraph claiming automation cannot cross this boundary is replaced by what it now does |
| `src/tests.rs` | `the_release_gate_consults_the_preconditions_script`, `the_release_preconditions_script_has_no_bypass` |
| `docs/sdlc/lessons.md` | one entry, with its Guard column named |

`scripts/package-macos.sh` and `scripts/replace-macos-bundle.sh` are **not**
edited. The swap transaction and its rollback are unchanged.

## Order of work

The tree compiles between every step; only step 4 touches Rust.

1. Write `scripts/check-release-preconditions.sh` and drive it by hand from the
   shell — success first, then each of the four failures — before any hook reads
   it. A gate is written against observed refusals, not against its own author's
   confidence.
2. Rewire the `replace-macos-bundle.sh` branch in `.claude/hooks/guard-bash.sh`,
   and confirm by hand that `bash .claude/hooks/guard-bash.sh <<< '<payload>'`
   emits `allow` on the good tree and `ask` on each mutation.
3. Update `docs/sdlc/README.md`. Two places state this policy and both move
   together, or the repository claims something it does not do — an Important
   finding under `REVIEW.md`.
4. Add the two tests to `src/tests.rs`, watch each fail by inverting what it
   guards, then restore.
5. `docs/sdlc/lessons.md`, then the three gates, then `check-bands.sh`.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Packaging turns out not to be byte-reproducible on some future toolchain | the gate refuses a correct release with a `diff -r` listing | fails closed — the operator gets the old prompt and the diff naming the file, so the failure is legible rather than silent |
| The command parser accepts a swap whose arguments are not the canonical triple | a bundle other than `dist/Operon.app` reaches `/Applications` under an `allow` | condition 1 requires an exact token match against a derived triple; anything else is `ask`. Watched failing with a wrong staged path |
| `pgrep` misses a running Operon launched some other way | the swap proceeds under a live single-instance lock | the app's own lock still refuses the second instance, as it does today. The gate is one layer, not the only one |
| The hook's 25 s becomes minutes on a cold cache | the session appears to hang at the swap | the hook's timeout is 600 s and the swap is rare; `gate_seconds` is banded at 420 |
| `decide ask` loses its two-space indent in a later edit | `gated_commands` silently falls to 5 and the band never fires because the boundary "still exists" in prose | `scripts/check-bands.sh` reads the counter, and the new test names the branch |
| The script grows a bypass later | the gate becomes decorative | `the_release_preconditions_script_has_no_bypass` |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. 428 passed; 0 failed; 6 ignored`.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `bash scripts/check-bands.sh` — `bands: N metrics within their bands`, with
  `gated_commands` still `6`.
- `the_release_gate_consults_the_preconditions_script` and
  `the_release_preconditions_script_has_no_bypass` both watched failing first.
- Four watched refusals, by hand, each printing the condition it stopped on: a
  hand-edited file inside `dist/Operon.app`; a source file changed after
  packaging; a deliberately failing test; Operon running.
- One watched pass: a clean tree, a freshly packaged bundle, nothing running —
  the hook emits `permissionDecision: "allow"`.

## Departures from the plan

**Condition 3 checks the artifact, not the tree — and the plan's probe for it was
not a probe.** "A source file changed so the rebuilt bundle differs" was watched
and did *not* refuse: a comment-only edit to `src/util.rs` rebuilt the release
binary byte-identically, so `diff -r` matched and the check correctly passed. The
mutation mutated the source and not the thing under test. Re-watched against the
artifact instead — a byte appended to `dist/Operon.app/Contents/MacOS/Operon` —
and against `Contents/Resources/Operon.icns`, where it refused and named the file.

The distinction is worth keeping rather than papering over: condition 3 establishes
that the staged bundle is what this tree *packages*, which is not the same claim as
that the tree has not moved. Tree state is condition 2's job. A source edit that
changes no compiled byte leaves a bundle that is genuinely still correct, and
refusing it would be wrong.

**Another document stated the old policy.** The plan named only
`docs/sdlc/README.md`. `.claude/skills/ship/SKILL.md` also described the gate as
asking a human, and a document that claims something the repository does not do
is an Important finding under `REVIEW.md`. Corrected.

**`scripts/harness-metrics.sh` was edited, and the plan did not list it.** Its
`invariant_tests` regex is a list of name prefixes; without `the_release_` the two
new guards would exist and not be counted. It is a `harness` surface — medium,
supervised — and changing the instrument alongside the thing it measures is called
out here because it makes this reading less comparable than the next one.

**The README section was written twice, and cut in half.** `check-bands.sh` shows
`steering_bytes` at its `propose` threshold — a pre-existing breach that change 030
already owns, `awaiting-user` — and `docs/sdlc/*.md` is inside the counted glob.
The first draft cost +1 685 bytes on that metric; the committed one costs +660. The
reasoning it dropped is in `spec.md`, which is not counted.

**Two refusals were watched that the plan did not ask for.** An absent
`dist/Operon.app` refuses with "does not exist. Build it with…", and condition 4
fired for real, with a live pid, when another session launched the installed app
mid-run.

**Timing.** The plan quoted ~25 s from warm measurements. The watched end-to-end
pass took **2 m 05 s**, because the release build was cold after another session's
commits. Well inside the 600 s hook timeout, but the number was removed from
`.claude/skills/ship/SKILL.md` rather than published as a promise.

**The two new tests landed in someone else's commit.** They were unstaged in
`src/tests.rs` when another session staged that file whole and committed it as
`a6b13eb` (sdlc 033). `the_release_gate_consults_the_preconditions_script` and
`the_release_preconditions_script_has_no_bypass` belong to this change; 033's
`report.md` carries the same pointer the other way. Not reverted — pulling hunks
out of a landed commit to re-land them is more churn in the file that caused the
problem.

**The three gates could not be run at the end.** Another session was mid-edit in
`src/app.rs`, `src/app/screens.rs`, and `src/ui/files.rs` in the same working tree
and it does not compile. The two new tests were watched green and watched failing
under four mutations before that, and they are committed; the closing gate run is
recorded `pending` in `state.yaml` rather than claimed.

**No `lessons.md` entry.** The probe that was not a probe was caught during
verification rather than reaching the tree, and it has happened once. Recorded here
instead, where the implementation reality belongs. `docs/sdlc/lessons.md` is also
the largest single contributor to the `steering_bytes` breach above, so a
first-occurrence entry is a cost 030 should not have to absorb.
