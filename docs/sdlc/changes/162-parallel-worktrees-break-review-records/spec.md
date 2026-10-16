# Spec: a review survives unrelated work landing on main, and change numbers stop colliding

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. The digest `scripts/check-review.sh` binds a verdict to ignores the two
   things in a diff that describe the rest of the file rather than the change:
   the line numbers in each hunk header (`@@ -a,b +c,d @@` reads as `@@`) and the
   whole-file blob ids on each `index` line (`index 1a2b..3c4d 100644` reads as
   `index 100644`). The same edit staged against two bases that differ only by
   lines added elsewhere in the same file yields one digest. Test:
   `a_review_survives_main_moving_the_lines_above_it`. Measured on change 158:
   its reviewed diff and its landing diff after taking in 159 and 160 differ in
   exactly one hunk header's numbers and one `index` line; both normalise to
   `03555c43b78c3415`.
2. Everything else in the diff still moves the digest: a different edit, an
   edit on main to a line inside the reviewed hunk's context, and an edit that
   follows a line holding bytes that are not UTF-8 each yield a different
   digest. The normalising `sed` runs under `LC_ALL=C`: under the person's
   `ja_JP.UTF-8`, BSD `sed` stops at the first invalid byte sequence with
   `RE error: illegal byte sequence` and prints only what came before, and the
   pipeline ignores its status, so everything after such a line would stop
   moving the digest (found by the evaluator, measured). Test: the same
   `a_review_survives_main_moving_the_lines_above_it`.
3. A record holding the digest as computed before this change — the sha256 of
   the diff byte for byte — still passes. Records being written in other
   worktrees when this lands are not sent back. Test:
   `a_review_recorded_before_change_162_still_counts`.
4. `the_review_gate_has_no_bypass` keeps pinning the digest path line by line:
   the new `digest=` line with its one `LC_ALL=C sed`, the `legacy_digest=`
   line, and the staleness comparison accepting exactly those two, each pinned
   exactly. No other line may assign a variable from `run_diff` through
   `shasum`. Test: `the_review_gate_has_no_bypass` (amended).
5. `scripts/check-change-number.sh <branch>` refuses a branch `<kind>/NNN-slug`
   when `NNN` is already used with another slug: by a directory under
   `docs/sdlc/changes/` on `main` or `origin/main`, or by any local or remote
   branch. The refusal names the holder and the next free number — the largest
   number found in any source, plus one. A branch with no number passes. A
   number held only under the same slug passes, the slug compared alone, so
   `feature/158-x` and `fix/158-x` are one change. A remote branch is read
   with its remote name stripped (`origin/fix/161-y` holds 161 as `161-y`).
   `make wt.new` runs it before creating anything. Test:
   `a_new_change_cannot_take_a_number_already_in_use`, one arm per rule.
   Collisions already on `main` (059, 060, 098, 128 each name two directories)
   are history: the check judges only a new branch, so they block nothing,
   though a new branch reusing one of those numbers is refused.
6. The pipeline says what to do when main moved during review: merge it, run
   `bash scripts/check-review.sh --against origin/main` before committing the
   merge, and if the verdict is stale, re-confirm it and stage the refreshed
   `review.yaml` in that same merge commit. `.claude/skills/feature-pilot/SKILL.md`
   says it where it already says to merge main; `docs/sdlc/templates/review.yaml`
   names the normalisation in its digest recipe. Test:
   `harness_documents_only_name_paths_that_exist` (existing) keeps the named
   paths real.

## Behaviour

Who reads what:

- A session landing a change after main moved elsewhere in a shared file sees
  `✅ SHIP` instead of 「判定が古い diff のものです」.
- When main changed a reviewed line, the refusal is unchanged:
  「判定が古い diff のものです / 判定は diff X のもの、現在の diff は Y。」.
- `make wt.new BR=fix/161-x` with 161 taken prints, and creates nothing:
  「❌ 変更番号 161 は使用中です: fix/161-new-terminal-opens-with-a-stray-ctrl-d。
  次の空き番号は 163 です。」 (exit 1).
- `git fetch` failing inside the number check (offline): one warning line,
  「⚠ origin を取得できませんでした。手元の情報だけで番号を確かめます。」, and the
  check goes on with the refs it has.

## Design

Placement: `EXTEND` of the review gate; `NEW_IN_MODULE` for the number check,
which joins the other `scripts/check-*.sh`.

- `scripts/check-review.sh`: one `LC_ALL=C sed -E` with two expressions between
  `run_diff` and `shasum` on the `digest=` line —
  `s/^@@ -[0-9]+(,[0-9]+)? \+[0-9]+(,[0-9]+)? @@/@@/` and
  `s/^index [0-9a-f]+\.\.[0-9a-f]+/index/`. In `git diff` output a content line
  begins with a space, `+`, `-`, or `\`, so a line beginning `@@` is a hunk
  header and a line beginning `index` is a file header. `legacy_digest=$(run_diff | shasum -a 256 | cut -c1-16)`
  beside it, and the staleness test accepts either. Every message keeps printing
  `$digest`, so every new record is the normalised one.
- `scripts/check-change-number.sh`: bash; `git fetch --quiet --prune origin`
  under `scripts/run-bounded.sh 20` — wider than `worktree-new.mjs`'s
  `git fetch origin <base>`: every remote branch, with deleted ones pruned, so
  the check sees other sessions' pushed branches; it updates remote-tracking
  refs and writes no file. Numbers from
  `git ls-tree --name-only <ref>:docs/sdlc/changes` for `main` and
  `origin/main`, and from `git for-each-ref --format='%(refname:lstrip=2)'
  refs/heads refs/remotes` with a leading `origin/` removed.
- `Makefile` `wt.new`: `bash scripts/check-change-number.sh $(BR)` before the
  node script.
- `src/tests.rs`: two fixture tests on `ReviewGateFixture`; one fixture test of
  the number check in a temporary repository with a bare `origin`.
- Docs: `.claude/skills/feature-pilot/SKILL.md` (step 0 numbering; the
  merge-main recipe), `.claude/skills/sdlc/SKILL.md` §3 numbering line,
  `docs/sdlc/templates/review.yaml` recipe, `docs/sdlc/lessons.md` entry.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | Nothing is drawn. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No | Nothing is drawn. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | No | The digest recipe lives in `scripts/check-review.sh` only; the template describes it and says to read it off the script. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | No | No store, sidecar, or migration is touched. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | Yes | No Rust spawn is added outside `src/tests.rs`. The new script runs `git` and `scripts/run-bounded.sh` with fixed arguments; the branch name is matched by `[[ =~ ]]` and never evaluated. |
| Documentation — user-facing docs change in all three languages together | No | Only agent-facing docs change; `README*` do not describe the gate. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Yes | The number check fetches every branch of `origin`, the user's own remote (`worktree-new.mjs` fetches only `origin <base>`); it fails open offline. |
| Budgets — any new scan or output path states its byte and item ceiling | Yes | The fetch is bounded at 20 s; the scan reads ref names and one tree listing per ref, no file contents. |

## Flagged concerns

- **Is ignoring hunk line numbers and blob ids a weaker gate?** It drops two
  facts: where in the file a hunk sits, and what the rest of the file holds.
  Every added and removed line, its three lines of context, the enclosing
  function text git prints after `@@`, the file names, and mode changes are
  still hashed. The rest of a touched file is then treated as every untouched
  file already is: a change on main to code the reviewed diff does not touch
  does not void the review. A hunk can only keep its digest while moving if the
  same lines, with the same three lines either side, now sit elsewhere in the
  same file. Answered: accepted, because the alternative is that every change
  landing on main voids every review in flight in that file. The person asked
  for the permanent fix and delegated the call (quoted in the intent).
- **Accepting the legacy digest forever.** A legacy match means the diff is
  byte-identical to the one reviewed, which is strictly stronger than the
  normalised match. Answered: kept, so no in-flight record is invalidated; it
  costs one line.
- **The merge of main that brings no change directory.** Such a merge commit is
  judged on its own diff and cannot carry a refreshed record. Answered: out of
  scope (intent); it needs main to have taken a commit with no change directory
  that also overlaps a reviewed hunk, and the existing refusal still names it.

## Acceptance

- `cargo test --locked a_review_survives_main_moving_the_lines_above_it
  a_review_recorded_before_change_162_still_counts
  a_new_change_cannot_take_a_number_already_in_use the_review_gate`
  passes, and each new test was watched failing before the script changed.
- `make q.fast` passes; `bash scripts/check-bands.sh` within bands.
- Change 158 lands after merging this from main, with its refreshed record in
  that merge commit and no further review round.

## Rejected alternatives

- Allow a commit that only updates `review.yaml` when its digest matches the
  branch's diff against main — wider: it lets a record move without any code
  or merge to vouch for it, and the gate would need a second base.
- Hash with `-U0` (no context) — drops the context lines too, so main editing a
  line next to the reviewed one would no longer send it back.
- Rebase branches onto main instead of merging — rewrites pushed history, which
  the destructive-git guard refuses.
- Pick numbers from a shared counter file — another file every session writes,
  which is the collision moved, not removed.
