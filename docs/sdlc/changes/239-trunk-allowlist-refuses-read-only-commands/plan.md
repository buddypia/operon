# Plan: the trunk allowlist refuses reads that change nothing

- **Spec**: n/a (`bugfix` route — the bug report is in `./state.yaml`)
- **Approved**: 2026-11-01
- **Status**: approved

## Root cause

`.claude/config/worktree-policy.json#trunk_bash_allowlist` names what runs on
`main`. Since change 173 it has refused 439 commands; about 85 of them read and
change nothing (counts in `state.yaml`). The list never named them, and the two
`git branch` patterns accept `--list` only as the first option, with no `-r` or
`-a` before it and no `--contains`.

## What is let in, and what each one keeps out

| Pattern (regex source, anchored by the guard) | Admits | Still refused |
|---|---|---|
| `git branch` flags pattern gains `--format=\S+` | `git branch -a --format=%(refname:short)` | `git branch x`, `--set-upstream-to=` |
| `git branch` list pattern: `-a`/`-r`/`-v`/`--format=` before `--list`, `--contains`, `--no-contains`, `--merged`, `--no-merged`; names may hold `[` `]` | `git branch -r --list origin/fix/236*`, `git branch --contains e37c065` | a dash after the names (`--force`, `-D`), a bare name |
| `git ls-remote` with `-q`/`-h`/`--heads`/`-t`/`--tags`/`--refs`/`-b`, remote `origin` only | `git ls-remote --heads origin feature/x` | `--upload-pack=`, any other remote or path |
| `git pull` with `-q`, requiring `--ff-only`, then nothing, `origin`, or `origin main` | `git pull -q --ff-only origin main` | `git pull`, `--rebase`, a refspec, another remote |
| `rg` without `--pre`/`--pre-glob`/`--hostname-bin` | `rg -n pattern src` | `rg --pre ./x`; `RIPGREP_CONFIG_PATH=…` in front (guard code, below) |
| `sleep`, `tr`, `dirname`, `readlink`, `paste`, `du`, `strings`, `md5`, `md5sum`, `sha256sum` | as named | a redirect into a file (the guard's own rule) |
| `mktemp`, optionally `-d` and `-q`, no template | `mktemp -d` (in `$TMPDIR`) | `mktemp foo.XXXX`, `mktemp -d ./x.XXXX` |
| `(/usr/bin/)?python3 ~/.claude/hooks/secret-guard.py redact`, exactly | the filter the person's instructions require after `ps` | any other python, any other argument |
| `gh api` without `-X`/`--method`/`-f`/`-F`/`--field`/`--raw-field`/`--input` | `gh api repos/o/r/branches/x --jq .name` | every spelling that sends a body or a method |
| `check-approvals` joins the listed `bash scripts/…` | `bash scripts/check-approvals.sh --list` | — |

Guard code, in `isFetchWithGitOptions` (renamed for what it now covers): git's
own options (`-c core.sshCommand=`, `-C`, `--git-dir`) in front of `pull` and
`ls-remote` are refused as they are for `fetch`, since both start a transport
program; and `rg` with a `RIPGREP_CONFIG_PATH` assignment is refused, since the
file it names can carry `--pre`.

Not changed: awk, sed beyond line or regex prints, `python3`/`node` payloads,
redirects, `git -C <trunk>` in front of a transport. Each was decided by 095 or
131; this change adds no argument against them.

## Files that change

| File | Change |
|---|---|
| `.claude/config/worktree-policy.json` | the patterns above |
| `.cli/hooks/worktree-policy-guard.mjs` | the transport check covers `pull` and `ls-remote`; `rg` refused with `RIPGREP_CONFIG_PATH` |
| `src/tests.rs` | `the_trunk_allowlist_and_the_ownership_check_are_switched_on_here`: each admitted read in `reads_allowed`, each "still refused" spelling in `writes_denied` |
| `docs/sdlc/changes/239-…/` | `state.yaml`, this plan, `approvals.log`, `review.yaml`, `mutations.py` |

## Order of work

1. Add the reads and the refusals to the test; watch it fail on the unedited
   policy (a listed read refused).
2. Edit the policy, then the guard; the test passes.
3. Mutate: drop each exclusion (`--pre`, `--ff-only` requirement, `origin`
   only, the `gh api` method/body lookahead, the transport check for `pull`,
   the `RIPGREP_CONFIG_PATH` check, the `mktemp` template ban) and watch the
   test go red for each.
4. Replay the logged refusals through the new policy (`.tmp/239/replay.mjs`):
   about 85 more admitted, none of them a write.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A new pattern admits a writing or program-starting spelling | a write on `main` the guard let through | `writes_denied` cases per exclusion; the mutations in step 3; review |
| `gh api` defaults to POST when a field is given | a mutation through the API from `main` | `-f`/`-F`/`--field`/`--raw-field`/`--input` all refused |
| `git pull --ff-only` on `main` | moves `main` to `origin/main` | intended: the same move `git merge --ff-only` already makes on the list |

## Proof of completion

- `cargo test --locked the_trunk_allowlist_and_the_ownership_check_are_switched_on_here`
  — fails before step 2, `1 passed` after.
- `node .tmp/239/replay.mjs <worktree> .claude/config/worktree-policy.json 2026-10-17T12:00`
  — `admitted ~90 of 439` (6 before), every admitted line a read.
- `make q.fast` clean; CI green on the branch.

## Departures from the plan

None yet.
