---
paths:
  - "src/config.rs"
  - "src/tmux.rs"
  - "src/agents.rs"
  - "src/cli.rs"
  - "src/history.rs"
---

# Before writing any identifier two places have to agree on

Colour, icon, and naming follow one rule: the string lives in `src/config.rs`
and everything else refers to the constant. `MANAGED_TMUX_PREFIX` is the case
that proves why. It was written inline at five call sites, the rename moved the
gate that recognizes it and not the five, and opening a terminal, resizing a
pane, and recovering an unregistered session all silently refused every session
this app created. The whole suite stayed green: it only ever handed the gate a
literal it had typed itself.

So: build the value through the constant or its helper (`managed_tmux_name`),
and test the **round trip** — the value the app produces put through the check
that consumes it — not the check against a literal.
`the_session_prefix_is_written_in_exactly_one_place` fails on a second literal.

The rule covers names Operon does not own: the directories inside Claude Code's
store are `CLAUDE_STORE_DIRECTORY`, `CLAUDE_PROJECTS_DIRECTORY`, and
`CLAUDE_SESSION_REGISTRY_DIRECTORY`, reached through `claude_projects_root` and
`claude_session_registry_root`.

## The same rule applied to a guard

Lesson 004 is this rule one level up, and it is the part that gets skipped. The
guard written *because of* the prefix bug searched for the hardcoded strings
`"operon-` and `"xirp-copy-`, so renaming the constant would have left it looking
for a prefix nothing used — passing, having checked nothing, on exactly the change
it existed to survive.

A guard that restates the value it guards is not a guard. Read the literal out of
the constant. This applies to shell scripts too: `.claude/hooks/gate-stop.sh`
spells its stamp path once, and the test that checks it reads the path out of the
hook.

---

Enforcement: `the_session_prefix_is_written_in_exactly_one_place`,
`every_name_the_app_gives_a_session_passes_the_gate_that_opens_it`,
`the_claude_store_directories_are_spelled_in_exactly_one_place`, and
`evals/003-identifier-ssot.md`. Entries 001 and 004 in `docs/sdlc/lessons.md`.
