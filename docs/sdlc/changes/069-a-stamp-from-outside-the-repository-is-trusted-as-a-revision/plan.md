# Plan: a stamp from outside the repository is trusted as a revision

- **Spec**: none. `bugfix` skips stage 2, and the bug reports are the terminal
  sessions below — see `./state.yaml`.
- **Approved**: 2026-09-22
- **Status**: in progress

Change 068 landed at `3f5c2ae` with `approve 0 0`. These four were found after
it, by reading its own output adversarially rather than by reading its diff —
which is the distinction worth keeping: every one of them is visible only by
running the thing with a value nobody had tried.

## The four causes, each reproduced before anything was edited

**A — the stamp is spent as a git revision and never checked to be one.**
`scripts/check-installed-build.sh` reads `installed_commit` out of
`$bundle/Contents/Info.plist` — a file under `/Applications`, which anyone who
can write there chooses — and hands it to `git cat-file -e "${it}^{commit}"` and
`git merge-base --is-ancestor`. Git's revision grammar is much wider than "a
commit id". Measured against `3f5c2ae`, with a fixture repository and a bundle
stamped with the four characters `HEAD`:

```
installed build: CURRENT — /tmp/adv068b/Operon.app was built from HEAD, which
  includes the newest release-binary commit 5dae14ef…
  >> exit 0
```

That is the check answering "yes" to its own question for any bundle at all,
including one that is genuinely years behind — and `.claude/hooks/gate-stop.sh`
reports only on exit 1, so the answer is silence. `main`, `@`, `HEAD@{1}` and
`:/message` are the same family.

This is the class change 068 fixed for `$1` and did not look for on the one
input that actually crosses a trust boundary.

**B — a hyphen-leading stamp, and why it is not a separate requirement.**
`--help` already produced `UNKNOWN COMMIT` before the fix, because `^{commit}`
is appended and the result is no longer option-shaped. True by accident rather
than by design; A's validation makes it true on purpose. Recorded so the next
reader does not count it as a second defect.

**C — a question that cannot be asked exits 1.** With the packager missing:

```
sed: /tmp/adv068b/tree/scripts/package-macos.sh: No such file or directory
  >> exit 1
```

`set -e` ends the script at `sed`. The file's own header defines exit 1 as "the
installed bundle is behind, or carries a stamp this repository has never heard
of", and exit 0 as the code for every question that cannot be asked — including,
in the same sentence, "no key declared". So a checkout without a packager
answered through a code nobody chose, with a raw `sed:` line as its message.

**D — WITHDRAWN.** `sed … | head -1` was predicted to abort under `pipefail`
when the packager declares `source_commit_key=` twice. It does not: the probe
ran and the script answered `CURRENT` correctly. Kept in the record because a
finding that did not survive its own test is evidence about the method.

**E — `merge-base --is-ancestor` has three answers and the caller saw two.**
0 is yes, 1 is no, and 128 is "I could not tell you" — measured:

```
$ git merge-base --is-ancestor <head> 000…000
fatal: Not a valid commit name 000…000
  >> merge-base exit 128
```

Read as a plain `if`, the third answer printed `STALE` and exited 1. A transient
git failure — a corrupt object, a permission problem, a resource limit — became
a positive claim that the bundle was behind, and the Stop gate turned it into a
refusal. The path is narrow, because `cat-file -e` upstream screens most causes;
what it removes is a false claim in the cases it does not.

**F — a `finally` does not run on SIGTERM.** `docs/sdlc/templates/mutations.py`
restores the mutated file in a `finally`, which covers a normal exit and Ctrl-C.
SIGTERM raises nothing: the interpreter dies where it stands. Measured, with a
sweep whose subject file holds `kept`:

```
mid-run content : BROKEN
after SIGTERM   : BROKEN
```

The tree keeps a file somebody wrote to be wrong, and the next `git add -A`
commits it. In a real sweep that file is `src/history.rs`. The running check was
left behind too.

## Files that change

| File | Change |
|---|---|
| `scripts/check-installed-build.sh` | A: the stamp must look like an object name; C: guard the packager read; E: three-way on `merge-base` |
| `docs/sdlc/templates/mutations.py` | F: signal handlers, atomic restore, restore verified before exit, the check's group killed on interrupt |
| `src/tests.rs` | three new tests, one per behaviour that can be run |
| `docs/sdlc/lessons.md` | lesson 043, added in review round 1 |
| `docs/sdlc/changes/069-…/` | `state.yaml`, this file, `mutations.py`, `review.yaml` |

## Order of work

The tree compiles between every step; no signature changes.

1. `the_installed_build_check_refuses_a_stamp_it_cannot_verify` — a fixture
   repository and bundles stamped `HEAD`, `main`, and a real sha as the control.
   Watched failing, and the failure must say `CURRENT`, which is the defect.
2. The script's object-name check. Re-run.
3. `the_installed_build_check_answers_zero_when_it_cannot_ask` — the packager
   removed. Watched failing, and the failure must show the `sed:` line and
   exit 1.
4. The script's packager guard. Re-run.
5. `the_installed_build_check_does_not_call_a_git_failure_stale` — a `git` shim
   first on `PATH` that passes everything through except
   `merge-base --is-ancestor`, which exits 128. Watched failing on `STALE`.
6. The script's three-way branch. Re-run.
7. `the_mutation_sweep_template_puts_the_tree_back_when_it_is_killed` — spawn
   the sweep, wait until the subject file is mutated, send a real SIGTERM with
   `/bin/kill -TERM`, require the file restored and no child left. Watched
   failing on `BROKEN`.
8. The template's signal handling and atomic restore. Re-run.
9. Three gates, bands, the sweep, review.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The object-name check refuses a stamp a real packager writes | `/Applications/Operon.app` starts reading UNKNOWN COMMIT on this machine | `scripts/package-macos.sh` stamps `git rev-parse HEAD`, which is a full hex id. Checked on the real bundle after the change, not assumed |
| sha256 repositories write 64-hex ids | A future repository format reads as "not an object name" | Both widths accepted, 40 and 64 |
| The `git` shim leaks into other tests | Unrelated tests see a broken git | The shim lives in the test's own temporary directory and is put on `PATH` for that one `Command`, never for the process |
| SIGTERM arrives before the sweep has mutated anything | The test asserts a restore that never needed to happen, and passes vacuously | The test polls until the subject file actually reads the mutated value, and fails if it never does |
| `os.replace` across filesystems | The restore raises and the file stays mutated | The temporary file is created in the target's own directory, so the rename is within one filesystem |
| Signal handler runs while `write_atomically` is mid-rename | A half-written temporary is left behind | `os.replace` is atomic and the handler restores from memory, not from disk. The temporary is *not* unlinked by the `except` on that path — `os._exit` runs no `except` and no `finally` — so the handler unlinks the live temporaries itself, from a module-level set written before the write and cleared after it. Corrected in review round 1; the row previously claimed the `except` covered it |
| A second signal arrives while the handler is inside its bounded wait | The handler runs inside itself: the inner `os._exit` abandons the outer call's temporary and `communicate()` is called twice on one `Popen` | The handler installs `SIG_IGN` for all three signals before it does anything else, and a `handling` flag covers a signal already flagged when the handler was replaced. Found in review round 1 |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — `ok`, `6 ignored`, three above 503.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `bash scripts/check-bands.sh` — the same two breaches as HEAD and no third.
- `bash scripts/check-installed-build.sh` — still `CURRENT` on this machine.
- The four probes in `/tmp/adversarial-068b.sh` and
  `/tmp/adversarial-068-sweep.sh` re-run against the fixed files.
- `python3 docs/sdlc/changes/069-…/mutations.py` — every mutation caught.
- `bash scripts/check-review.sh` — ✅ SHIP.

## Departures from the plan

**The refactor was written before the plan was committed, and was unwritten to
fix that.** The four causes were found and the hardened files produced in one
adversarial pass, which put source ahead of `plan.md` — the one ordering
`.claude/skills/sdlc/SKILL.md` step 7 calls out by name, because a plan committed
afterwards is a description. Both files were copied to
`/tmp/operon-069-refactor/`, `git checkout --`-ed back to `3f5c2ae`, and the plan
committed as `6611c1f`; then steps 1–8 above were run in order, each guard
watched failing against the old code before its fix was applied. The recovery is
recorded rather than hidden because the failure is a process one and the next
reader's temptation is the same.

**Five tests, not three.** "Order of work" says "three new tests, one per
behaviour that can be run" and then lists four steps that each add one. The
fourth is `the_mutation_sweep_template_puts_the_tree_back_when_it_is_killed`,
which the table under "Files that change" also implies; the count in the table
row was simply wrong. The fifth,
`the_mutation_sweep_template_restores_the_mode_it_found`, came out of review
round 1 — see nit 4 below. `cargo test --locked` goes 503 → 508.

**A fixture helper came out of the first test rather than being planned.**
`installed_build_fixture` and `write_stamped_bundle` in `src/tests.rs` are shared
by all three script tests. Writing the third one by copy would have been the
fourth copy of the same temporary repository, and 068's review had already
carried the opposite shape — fixtures structurally simpler than the thing they
stand for — as its deepest finding.

**Finding B is not a requirement and has no guard.** The plan already says why —
`--help` was refused by accident before the fix, and A makes it refused on
purpose — but the consequence was decided during the build: no mutation exists
for it, because a mutation that removed A's check would be
`stamp-used-as-a-revision` twice.

**Change 068's committed sweep now reports `SETUP FAILED`.** This change
refactored the text `bound-reaches-only-the-direct-child` replaces, so 068's
`mutations.py` can no longer find its target. It was left alone deliberately:
`scripts/check-review.sh` refuses a diff touching more than one change directory,
and a closed change's sweep is the record of a run its `state.yaml` reports on —
its "9 of 9" stands for the code at `3f5c2ae`. Recorded at more length in
`./state.yaml`.

**Review round 1 returned `approve-with-nits 0 4`, and all four were taken here
rather than carried.** That is a departure from `REVIEW.md`'s default — a nit is
carried — and it is deliberate: change 068 is titled "the nits two reviews
carried are still defects", and three of these four are the same shape.

1. The signal handler could run inside itself, because it blocks for up to
   `KILL_GRACE_SECONDS` and a Python-level handler runs in the eval loop.
   `SIG_IGN` for all three signals is now the first thing it does.
2. The same re-entry called `communicate()` twice on one `Popen`. Closed by 1.
3. `the_mutation_sweep_template_puts_the_tree_back_when_it_is_killed` asserted
   its way out while the sweep it spawned was still running, and a `Child` does
   not kill on drop — a failing test leaked a ten-minute `sleep`. `REVIEW.md`
   lists "it leaks a child process" as an Important example and the reviewer
   left the severity open; taking it settles the question.
4. **`scripts/check-installed-build.sh` had lost its executable bit**, `100755 →
   100644` and `600` on disk. The reviewer reported it as an unexplained diff
   header. It is worse than that, and the difference is the point.

**Nit 4 was not a stray `cp`. It was a defect this change introduced, and it is
recorded as the fifth cause.** `chmod 755`, re-run the sweep, and the file is
`600` again — reproduced twice. `write_atomically`, added here for F, writes a
`mkstemp` sibling and renames it over the target; `mkstemp` creates at `0600`
and `os.replace` swaps the whole inode, so the restore carried the temporary's
permissions onto the file. The `path.write_text` it replaced wrote through the
existing inode and could not do this. So the hardening that made the restore
atomic made it lossy, in the one dimension nothing checks: the bytes came back
exactly, `git diff` was empty, and `restore_all`'s read-back compares contents.
The only trace in the whole repository was a mode line in
`git diff --cached --summary`.

A sweep's subject is whatever the change touched, and this repository's changes
touch `.claude/hooks/*.sh` and `scripts/*.sh`. The failure this was one sweep
away from is a hook restored without its executable bit — a gate that stops
running, in a tree whose `git status` is clean.

Fixed by carrying the target's mode onto the temporary before the rename, and
guarded by `the_mutation_sweep_template_restores_the_mode_it_found`, which
sweeps a `755` fixture and requires `755` afterwards. Watched failing through
the sweep's own `restore-drops-the-file-mode` mutation. `cargo test --locked`
goes 507 → 508 and the sweep 7 of 7 → 8 of 8.

Nits 1 and 2 have no mutation. A sweep entry would have to win a race between
two signals to prove anything, and a mutation that cannot fail reliably reports
the unmutated tree as caught — which is rule 1 of the template it would live in.
Said here rather than papered over with an entry that passes vacuously.

**`KILL_GRACE_SECONDS` was added beyond the plan.** The plan's F row asked for
"the check's group killed on interrupt"; the first version then waited
unboundedly on `communicate()` for the killed group's pipes, which is the code
that exists to stop a hang becoming one. The wait is bounded at 10 seconds.
