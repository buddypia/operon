# Intent: the local machine runs the whole test suite several times per change

- **Status**: approved
- **Opened**: 2026-10-06

## Problem

After code is written, the person's Mac runs the full test suite (about 80
seconds, with real git repositories and tmux servers) again and again for one
change: once when verifying, once at every commit, once when packaging, and
twice more before the installed app is replaced. GitHub Actions already runs the
same suite on every push to `main` (change 126), but nothing waits for it: work
lands by a local merge and is pushed only afterwards, so the CI run is a report
nobody reads before the change is in.

## Who feels it, and when

The person, on every change an agent carries through the pipeline: the machine
is busy while the agent repeats a suite whose answer does not change between the
runs, and other sessions sharing the machine slow down.

## Desired outcome

Quoting the person's request (2026-10-06): 「コード生成後、ローカルでPRレビューや
検証しているのをGithub ActionsのCI/CDに任せてローカルの負荷を軽減したい。」
Decided with the person the same day:

- Landing flow: push the change's branch, wait for CI to pass, then merge
  locally and push `main`.
- Reviews by the reviewer agents stay local (they cost tokens, not CPU).

Observable afterwards: a change carried from first commit to installed app runs
the full suite on the local machine zero times when the network is up; a branch
whose CI failed, or never ran, still cannot be merged into `main`.

## Constraints this change inherits

- The protection stays: no code reaches `main` without the full suite having
  passed on that tree somewhere. When CI cannot be asked (offline, `gh` not
  logged in), the suite runs locally as before.
- `every_document_that_names_the_gates_names_the_same_three` and the other
  invariant tests keep holding.
- Reviews and `scripts/check-review.sh` are unchanged.

## Not in scope

- Moving the reviewer agents into CI (`claude-review.yml`).
- Pull requests on GitHub; landing stays a local `--no-ff` merge.
- The `#[ignore]`d live-CLI tests: still run locally when restore moves.
