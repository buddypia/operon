# Plan: a resume runs the agent the way it was launched

- **Spec**: `./spec.md`
- **Route**: `modify` — the verb, the button, and the place are unchanged; what
  the button does is corrected. Recorded here rather than left implicit, because
  the route decides whether a person must read the diff and choosing the lighter
  one quietly would be the wrong way to reach that answer.

## Order

1. **`src/agents.rs` — the grammar first, with no caller.** Add
   `resume_command_with_launch_options`, `carried_launch_options`, and
   `carried_launch_is_dangerous`. Widen `is_native_resume_command` and
   `native_session_id_from_agent_command` to the same grammar in the same commit,
   because a widened builder with a narrow recogniser is the failure in
   requirement 3: the reopened command would gain the session's goal as a
   positional prompt and replay a user turn.
2. **Tests before the call site.** Every function above is pure and takes
   strings, so the whole mechanism is testable without a window, a tmux server,
   or a CLI. This is the commit where the guards are watched failing.
3. **`src/app.rs` — two call sites.** `resume_managed_native_session` builds
   `agent_command` through the new function; `session_verb` reads the two hint
   functions and sets `danger`.
4. **`src/i18n_tables.rs`** — the new message ids in all three languages.
5. **README in three languages**, one bullet.
6. **Roadmap** — mark row 10's restart half, and write the argument against
   hibernation rather than leaving the row looking merely unfinished.

## What breaks, and how it is caught

- **The recogniser and the builder disagree.** A resume carrying options is not
  recognised, `agent_shell_command` appends the goal, and the agent redoes the
  first request of the conversation on every restart. Caught by
  `a_resume_carrying_options_is_still_a_native_resume`, which asserts the round
  trip through both functions rather than a literal.
- **`--session-id` is carried into a Claude resume.** Claude is asked to both
  allocate and resume a conversation. Caught by
  `a_resume_refuses_the_options_that_contradict_it`, which builds the launch
  command through `command_with_managed_native_session_id` rather than typing
  `--session-id`, so a rename of that flag moves the test with it.
- **`--new-project` is carried into an Antigravity resume**, which its own help
  says starts a new project — the resume would silently discard the
  conversation. Same test.
- **Codex options land before the subcommand.** `codex --search resume <id>` is
  a usage error, and the window opens, prints it, and dies with a session record
  already written. The builder puts `resume` immediately after the binary, and
  the test asserts token order, not a substring.
- **An option order that changes between builds.** The launch command's own
  order is preserved rather than re-derived, so a resume reads as a superset of
  the launch it came from.
- **Work in a draw path.** `session_verb` runs every frame for every visible
  session. Both hint functions are a `split_whitespace` over one short validated
  line plus a walk of a `&'static` table — the same order as the `tr()` calls
  and the `String` hint the function already builds per frame, and no new
  allocation beyond that hint. Stated here so the reviewer checks the claim
  rather than rediscovering the question.

## Assumptions, and what would falsify them

- **The three CLIs accept their launch options beside their resume flag.**
  Checked on 2026-09-06 against the installed binaries: `codex resume --help`
  lists `--model`, `--sandbox`, `--ask-for-approval`, `--approve-for-me`,
  `--dangerously-bypass-approvals-and-sandbox`, `--search`, and `-c`;
  `claude --help` shows `claude [options] [command] [prompt]` with the flags
  global; `agy --help` is one flat set containing `--conversation` beside
  `--model`, `--mode`, `--effort`, `--sandbox`, and
  `--dangerously-skip-permissions`. Falsified by a CLI moving a flag under a
  subcommand, which would show as a usage error in a reopened window — which is
  why the unparsable case degrades to a bare resume rather than refusing.
- **`agent_command` holds the launch without the goal.** `agent_shell_command`
  appends the quoted goal at run time and does not store it, so the record is
  the options alone. Falsified by a positional token appearing in a stored
  command, which the safety gate would then carry into the resume as a prompt.
  Guarded: only tokens beginning with `-`, and the values immediately following
  a valued option, are carried.

## Not done here

- Hibernation. Argued against in the roadmap: a detached tmux session costs
  almost nothing to leave running, and killing one destroys the scrollback that
  is the reason to go back to it.
- Carrying an environment. There is no per-session private environment to
  capture; Operon's managed launches build theirs from the hook identity.
