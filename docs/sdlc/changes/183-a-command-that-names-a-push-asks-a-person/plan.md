# Plan: a person is asked only about a push they can read whole

- **Spec**: none — bugfix route; the bug report in `./state.yaml` is the intent.
- **Approved**: 2026-10-21
- **Status**: planning

## The rule

The publishing gate has three outcomes, decided on the command as a whole:

- **allow** — the command is one line that is exactly
  `git push [-u|--set-upstream|-q|--quiet]... origin <branch>`, whitespace
  collapsed.
- **ask** — the command is one line that is a lone `git push` whose words hold
  no shell syntax (no quote, `$`, `;`, `&`, `|`, `<`, `>`, parenthesis or
  backslash), and whose destination or refspec is a decision: a force, a `+` or
  `:` refspec, `--mirror`, `--delete`, `--all`, another remote. The person sees
  a command that is only that push.
- **deny** — anything else that names a push: joined, piped, redirected, inside
  `bash -c`, written with options before `push` (`git -C dir push`,
  `git -c k=v push`, `git --no-pager push`), with the remote and branch
  implicit, or only text in a commit message, a search or a file body. The
  reason names the form that runs and says how to keep text out of the command.
  Nothing denied runs, so a force joined to a commit still reaches a person, as
  the lone push the session reruns.

A push is detected as `git`, any global options (an option word, optionally
followed by one argument, quoted or not), then `push`, with any whitespace
between — so `git -C dir push --force` and `git<TAB>push --force` are pushes.

## What this changes from change 182

| Command | 182 | 183 |
|---|---|---|
| `git commit -m "… git push …"`, `grep "git push" …` | ask | deny, with how to keep text out |
| `git -C dir push --force origin main`, `git -c k=v push …`, `git --no-pager push -f …`, `git<TAB>push --force …` | runs unasked | deny (options) or ask (tab, lone) |
| a decision joined to other commands (`git commit … && git push --force origin main`, `bash -c "git push …"`, `… &`) | ask | deny; asked when rerun alone |
| `git push origin x > /tmp/log` | ask | deny |

Nothing moves to allow that did not run before: `git<TAB>push origin x` ran
unasked because the gate never saw it, and still does.

## Files that change

| File | Change |
|---|---|
| `.claude/hooks/guard-bash.sh` | push detection admits global options and any whitespace; the per-segment split is replaced by the lone/plain/else decision above |
| `src/tests.rs` | `a_plain_push_to_origin_is_not_asked_about`: text mentions, global options, redirects and joined decisions are denied; tab and `HEAD:main` lone pushes asked; `git commit -m "push it"` and `git log --grep push` untouched |
| `docs/sdlc/lessons.md` | entry 081, with its Guard |

## Order of work

1. Commit this plan, `state.yaml`, and `approvals.log`.
2. Test cases; watch them fail on the unedited hook.
3. The hook. Green.
4. Mutations: global options no longer detected; the lone check admits a quote;
   a non-lone push asks instead of being denied; the plain shape loses `-q`;
   the refusal no longer says how to keep text out of the command (the claim).
5. Gates, lesson, review (subprocess-safety-reviewer for gate-configuration,
   rust-reviewer for the test).

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The lone check admits shell syntax, so an ask shows a person more than the push | a joined command reaches a person | denied list holding `&`, `>`, quotes and `bash -c` |
| Detection widens onto commands that are not pushes | `git commit -m "push it"` refused | the untouched list |
| Detection misses a push written another way | a force runs unasked | the `-C`, `-c`, `--no-pager`, quoted `-C` and tab cases |
| `deny` loops on a command that only mentions a push | the same refusal twice | the reason names `git commit -F <file>`, the Write tool and `git[[:space:]]push`, none of which the gate detects |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo clippy --locked -- -D warnings` — clean.
- `cargo test --locked a_plain_push_to_origin_is_not_asked_about` — fails on the
  unedited hook, passes after.
- The whole suite in CI on the push.

## Departures from the plan
