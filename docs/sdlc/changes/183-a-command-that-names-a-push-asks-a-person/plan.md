# Plan: a person is asked only about a push they can read whole

- **Spec**: none — bugfix route; the bug report in `./state.yaml` is the intent.
- **Approved**: 2026-10-21
- **Status**: done

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

1. **Review round 1 found the detection still missed spellings the shell runs
   as a push** (both reviewers, Important): `"git" push`, `git "push"`,
   `git pu""sh`, a backslash-newline, `git -C a\ b push`, `git -C $'a b' push`,
   and a program or subcommand computed at run time (`$(which git) push`,
   `$G push`, `-c alias.p=push`). Each ran unasked on main as well. The hook now
   reads the command a second time as the shell hands git its words (a
   backslash-newline joined, quotes and backslashes dropped), reads an escaped
   space inside an option argument, and counts a computed program as a push
   whenever `push` appears. The denied list gained each spelling, the
   untouched list `git stash push`, `git -C d stash push` and a `$(cat <<EOF)`
   commit message, and the sweep three mutations (11 in all).
2. **Review round 2 found computed spellings in argument position**
   (subprocess-safety-reviewer, Important): `git $(echo push)`, `git ${IFS}push`,
   `P=push; git $P`, `"$x" push`, and `Alias.` in another case. Rather than one
   more pattern per spelling, a git subcommand that is an expansion is now a
   push whatever it expands to, a quote may precede a computed program, and
   `alias.` is matched in any case. Two more mutations (13 in all).
3. The lone shape is an allowlist of word characters, stricter than the list
   of shell syntax in "The rule": a glob, `#`, `!`, `%` or `,` is denied too.
   Whitespace is folded only where bash splits words (space, tab, newline), so
   a carriage return, `\v` or `\f` stays and is denied.
4. **Review round 3 found option arguments the pattern could not read**
   (subprocess-safety-reviewer, Important): `git -c user.name="a b" push -f`,
   `git -c a=$(echo b c) push -f`, ``git -c a=`echo b` push -f`` ran unasked.
   Each round had found one more spelling of a shell word, so the hook stopped
   reading option arguments: once an option follows `git`, any later word
   `push` on the line is a push. The detection is shorter than before, and
   `git -C d stash push` moves from untouched to denied. Replaying the 7,728
   distinct Bash commands in this project's transcripts through the hooks
   measured what else moves, and showed round 2's rule refusing 185 ordinary
   `git -C $W …` lines: an expansion after git's options now counts only when
   the word `push` is on the line, as a computed program does. A Bash call
   keeps no shell variables from the last one, so a `$P` meaning `push` is set
   on the same line, where the word is. The sweep's three argument mutations
   were retargeted and two added for the expansion rule: 15.
5. The lesson is entry 084, not 081: main took 081 to 083 while this was in
   review, and the merge of main renumbered it.
6. `state.yaml`, `approvals.log`, `review.yaml` and `mutations.py` in this
   directory change too; the table lists only the files outside it.
