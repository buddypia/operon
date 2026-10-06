# Bugfix: friction that recurred while landing 127 and 128

- **Route**: `bugfix` — each item below was observed in this repository on
  2026-10-06; the observation is the report.
- **Skill**: `.claude/skills/root-cause/SKILL.md`

The person asked for the problems met on the way to be fixed permanently
(「他にも問題を恒久対応して」). Four were in this repository.

| # | Observed | Cause | Fix | Guard (watched failing) |
|---|---|---|---|---|
| 1 | Packaging stopped twice on tests that pass alone | assertions on elapsed time and a 60 s node bound, on a machine shared with another session's suite | assert that the check ran (a marker), not how long the stop took; `NODE_HOOK_TEST_TIMEOUT` = 180 s | marker assertion, by mutation (lesson 064) |
| 2 | A later 「停止」 could delete a session without saying so | the notice after a failed stop did not mention the pending delete kept by change 128 | its own notice when a delete is pending, with EN/KO rows | `failed_stop_keeps_the_requested_delete_for_the_retry`, by restoring the old notice |
| 3 | Two branches numbered 127, two numbered 129, and two 128s landed on main | numbers are "next free" with no check across concurrent sessions; 059/060/098 already collided | test that a number names one change; 059/060/098/128 grandfathered; this change is 130, not 129 | `change_directory_numbers_are_unique` (lesson 063) |
| 4 | Following the commit guard's advice was refused by the commit gate | commit-guard advised `git -C <worktree> commit`, which gate-commit refuses | messages say `cd` then `git commit`; stale `/create-pr` line removed | `the_commit_guard_never_advises_a_commit_the_commit_gate_refuses` (lesson 065) |

## Not done here, and why

- **The trunk allowlist refuses commands that never touch the repository**
  (`ps`, `uptime`, a script in `$HOME`) while the session's project directory is
  the main checkout. Exempting them would loosen a guard, which
  `.claude/skills/sdlc/references/approval.md` gives to a person. Proposed to the
  person, not changed.
- **The Supabase access token in a process's arguments** was outside this
  repository: `nichenext/.envrc` held it, with two other secrets, in plain text.
  Moved to the macOS Keychain the same day; rotation needs the person's
  Supabase and Cloudflare logins.
