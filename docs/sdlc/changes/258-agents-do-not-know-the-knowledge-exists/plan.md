# Plan: an agent session starts knowing the project's knowledge exists

- **Spec**: `./spec.md`
- **Approved**: 2026-11-08
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/tests.rs` | seven `knowledge_258_*` tests; `HookEnvironment` literals gain `executable` |
| `src/config.rs` | `HOOK_EXECUTABLE_ENV`, `KNOWLEDGE_BRIEF_SUBCOMMAND`, `KNOWLEDGE_BRIEF_ENTRIES`, `KNOWLEDGE_BRIEF_MAX_CHARS`, `KNOWLEDGE_BRIEF_SECONDS` |
| `src/knowledge.rs` | `knowledge_brief`, `knowledge_brief_cli`, `bounded`, routing in `run_cli` |
| `src/tmux/hooks.rs` | `HookEnvironment.executable`, `variables`, the SessionStart branch of `hook_script` |
| `src/app.rs` | `mint_hook_environment` sets the executable |
| `src/app/knowledge.rs` | the opt-in bullet and the enabled-tab line |
| `src/i18n_tables.rs` | the bullet in every table |
| `README.md`, `README.ja.md`, `README.ko.md` | one line in the knowledge bullets |

## Order of work

0. Merge `main` once change 257 has landed, so this branch builds on it.
1. Constants, the `executable` field, and stub functions with today's behaviour
   so the tests compile; write the seven tests and watch each fail for its own
   reason.
2. `knowledge_brief` (Req 1–2).
3. `knowledge_brief_cli`, path-only project match, `bounded`, and the router (Req 3).
4. Hook script and environment (Req 4–5).
5. Opt-in bullet, enabled-tab line, and translations (Req 6); README lines.
6. `mutations.py`, `verification.yaml`, review.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The script prints `{}` and the brief, so Claude reads neither as JSON | context lost, or a parse warning | Req 4 asserts stdout is exactly the brief |
| The brief command swallows the hook payload | status stops updating | Req 4 runs it with a payload and checks it is still read after |
| A failing executable breaks session start | the agent refuses to start | Req 4's failure cases and the existing script safety test |
| The brief creates databases for projects that never opted in | stray files | Req 3 asserts none is created |
| A slow store, database, or filesystem holds the session start | the agent waits before its first prompt | `knowledge_258_a_slow_brief_gives_up_and_says_nothing`; no `git` call in the brief |
| Stray output from the executable reaches the CLI | a parse warning or junk context | Req 4: printed only when it starts with `{"hookSpecificOutput":` |
| A large base makes the context huge | adherence drops | Req 2 at `KNOWLEDGE_MAX_ENTRIES` |
| Codex's trusted hash breaks | Codex asks to trust the hook again | the managed command is unchanged; `hook` tests |

## Proof of completion

- `cargo test --locked knowledge_258_` — `7 passed; 0 failed`.
- `cargo test --locked hook` and `cargo test --locked knowledge_` — all pass.
- `make q.fast` — fmt and clippy clean.
- `python3 docs/sdlc/changes/258-agents-do-not-know-the-knowledge-exists/mutations.py` — every mutation caught.
- CI green on the branch head.

## Departures from the plan

None yet.
