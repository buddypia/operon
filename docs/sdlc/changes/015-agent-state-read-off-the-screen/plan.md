# Plan: agent state from the CLIs' own hooks

- **Spec**: `./spec.md`
- **Approved**: 2026-09-05
- **Status**: done

This is the plan produced in plan mode and accepted before any file was edited.
If the implementation departs from it, update this file — an abandoned plan is
worse than no plan, because the next reader trusts it.

## Files that change

| File | Change |
|---|---|
| `src/config.rs` | The hook identifiers: directory and file names under the data directory, the five environment variable names, the Antigravity bundle name, the body ceiling, the staleness window. |
| `src/util.rs` | `sha256` from the FIPS 180-4 description, used for the Codex trust hash. |
| src/tmux/hooks.rs (new, child of `src/tmux.rs`) | Script text and managed command per CLI; install and remove for Claude Code, Codex, and Antigravity; the endpoint file; the Unix-socket listener with its minimal HTTP reader; event normalisation; the per-session `HookStatus` state machine; the hook settings file (the toggle). |
| `src/tmux.rs` | `pub(crate) mod hooks;` plus its re-export; `HookEnvironment` threaded into `launch_stored_session`; `start_tmux_agent_session_in_environment` and `start_empty_tmux_session_in_environment` passing `-e KEY=VALUE` to `new-session` (the three-argument forms delegate with an empty environment so existing callers and tests stand); `set_tmux_environment` before a respawn. |
| `src/app.rs` | Fields `hook_listener`, `hook_events`, `hook_status`, `hook_launch_tokens`, `hook_installs`, `hooks_enabled`; bind the listener on the first frame; drain events each frame; mint a launch token in `start_session`; consult fresh hook state in `apply_session_poll`; pass the tool name into the status hint; install on startup and from the settings toggle; the settings checkbox and three rows. |
| `src/ui/widgets.rs` | `tool_status_row` gains a verdict-token variant so the hook rows can say INSTALLED / NOT INSTALLED / FAILED with the same layout. |
| `src/i18n_tables.rs` | EN and KO rows for every new message id, in sorted position. |
| `src/tests.rs` | The thirteen tests named in the spec's Acceptance. |
| `CLAUDE.md` | The module map gains the child file under the `src/tmux.rs` entry. |
| `README.md`, `README.ko.md`, `README.ja.md` | One bullet under Supervision. |

## Order of work

1. `src/config.rs` constants and `src/util.rs` `sha256` with its vector test. Compiles; the suite is green.
2. src/tmux/hooks.rs: script text, managed commands, the endpoint file, and the three installers and removers over `serde_json::Value` and line-based TOML. Tests against temporary directories seeded with the real shapes seen on this machine (third-party status bundles in Claude, Codex, and Antigravity). Compiles; green.
3. src/tmux/hooks.rs: the listener. Test binds a socket in a temporary directory, connects with `UnixStream`, writes a request, reads the 204, receives the event.
4. src/tmux/hooks.rs: normalisation and `apply_reading`. Table-driven tests.
5. `src/tmux.rs`: the environment-carrying launch functions and `set_tmux_environment`. Live-tmux test reads the variable back with `tmux show-environment`.
6. `src/app.rs` and `src/ui/widgets.rs`: wiring, poll merge, settings. `src/i18n_tables.rs` rows. Compiles; green; clippy clean.
7. `CLAUDE.md` and the three READMEs.
8. Gates, bands, manual round trip with Claude Code, commit.

The tree compiles between every step.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A hook command that hangs or errors makes the CLI slower or fail closed | Claude prints a hook error; a turn stalls for ten seconds | The script's first line is the CLI's expected stdout; `curl --max-time 1.5`; every path exits 0. Test: the script text is parsed by `/bin/sh -n` and the fallback branch of the managed command prints `{}`. |
| Rewriting a settings file loses a foreign entry | Another tool's hook stops firing | Install and remove tests seed foreign entries and assert them equal after each pass; the file is written only when the parsed value changed. |
| A stale process after respawn reports the old turn | A finished chip flips back to WORKING | Launch tokens are rotated before every respawn; the stale-token test. |
| `SessionStart` after resume notifies "finished" | A phantom notification after `/clear` | Boundary readings carry `boundary: true`; the never-notifies test. |
| The hook state stays authoritative after Operon lost the plot | A crashed CLI shows WORKING for ever | Freshness is thirty minutes and requires an alive pane; the outranks-then-yields test; the screen heuristic resumes. |
| Socket path over the `sun_path` limit | Bind fails on a long user name | Explicit length check with a Japanese notice; hooks stay uninstalled that start. |
| The `-e` flag is missing on an old tmux | Launch fails on tmux < 3.2 | The error text from tmux is surfaced as the launch error; tmux 3.7 is what Homebrew ships. |
| `subprocess_sites` or `modules` band moves | `scripts/check-bands.sh` warns | No new `Command::new`; the child module adds no `mod` line to `src/main.rs`. |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. N passed; 0 failed; 6 ignored` with N thirteen higher than before.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `bash scripts/check-bands.sh` — no breach beyond warn.
- Each new test watched failing by mutation: the sha256 vector with one round constant changed; the installer with the preservation step removed; the boundary test with `boundary` ignored; the stale-token test with the comparison removed.
- In the running app: a Claude Code session launched from Operon shows WORKING within a second of a prompt, IDLE when the answer ends, one notification; `/clear` shows IDLE and no notification; Settings shows 「Claude Code: 登録済み」; turning the toggle off leaves `~/.claude/settings.json` without any `agent-hooks/` entry and with every other entry intact.

## Departures from the plan

- **No `set_tmux_environment`.** Every launch, including a retry, creates the
  tmux session afresh, so `new-session -e` already carries the current token and
  `respawn-pane` inherits it. Verified against tmux 3.7 before the code was
  written: a value set with `-e` and then replaced with `set-environment` both
  reach the respawned pane.
- **Codex entries are appended, not prepended.** Prepending puts the group
  first so a slow user hook cannot delay the state. But Codex keys each trust
  hash by the entry's index in its file, so inserting at the front invalidates
  every trust entry the person already approved. Appending costs a little
  latency behind a slow hook and costs nobody an approval prompt.
- **The event name is passed in the environment for all three CLIs.** Only
  Antigravity requires it; doing it everywhere means the listener routes on one
  field rather than on three different payload shapes.
- **The metadata travels in the JSON envelope, not in HTTP headers.** The script
  wraps the CLI's payload in one object naming the source, the session, the
  token, and the event. One parse, no header vocabulary, and nothing to escape:
  the three added values are a session name, a UUID, and a fixed event name.
- **`tool_status_row` was left alone; `hook_status_row` was added beside it.**
  The existing row is a boolean, and this state has three values — "not
  registered" and "failed" are the same absence to a reader who is only told
  whether it worked, and only one of them is actionable.
- **`claude_md_maps_every_module_that_exists` was generalised.** It excluded
  `src/ui/` by name, so the first child module outside that directory failed it.
  It now excludes any path with a directory component, which is the shape of a
  child module rather than a list of the ones that exist — a list would have to
  be edited for each new one, which is how a guard gets edited into silence.
- **One more test than the plan named, and a lesson.** The endpoint file wrote
  the socket path unquoted, so sourcing it stopped at the space in `Application
  Support` and nothing was ever posted — while all nine tests passed, because a
  temporary directory has no space in its name. Found by running the real script
  against the real directory. `an_installed_hook_command_posts_a_reading_end_to_end`
  now runs the installed command through the script, the endpoint, the socket,
  and into a reading, under a path with a space; entry 013 in
  `docs/sdlc/lessons.md` records the rule.
- **A plain terminal carries the hook environment too.** It has no agent to
  report for at launch, but a person who then types `claude` into it by hand
  gets the same reporting as a launched session, for no extra code.
