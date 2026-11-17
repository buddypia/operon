# Plan: `Operon agents`

- **Spec**: `./spec.md`
- **Approved**: 2026-11-17
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/tmux/control.rs` | new: snapshot types, `agents_cli`, `agents_wait`, `AgentTerminal` + tmux impl, `run_agents_cli` |
| `src/tmux.rs` | `mod control; pub(crate) use control::*;` |
| `src/app/agent_snapshot.rs` | new: `agent_snapshot`, `track_agent_snapshot`, removal on exit |
| `src/app.rs` | `mod agent_snapshot;`, two fields, one call in `logic`, one in `on_exit` |
| `src/config.rs` | constants |
| `src/main.rs` | dispatch `agents` beside `knowledge` |
| `src/i18n_tables.rs` | strings in every table |
| `src/tests.rs` | `agents_309_*` |
| `CLAUDE.md` | map lines for the two new files |
| `README*.md`, `docs/MANUAL*.md` | section in three languages |

## Order of work

1. Constants and snapshot types; contract test of the round trip (R1 first half).
2. Failing tests R2–R9 against a fake `AgentTerminal` and an in-memory snapshot; then `agents_cli` / `agents_wait`.
3. App side: `agent_snapshot` and the due rule (R1), wired in `logic` / `on_exit`.
4. `main.rs` dispatch, strings, docs, CLAUDE.md lines.
5. `make q.fast`, `cargo test --locked agents_309_`, a live check from a tmux session.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Snapshot built every frame | stutter | the due check runs at most once a second; `agents_309_snapshot_is_due_on_change_or_heartbeat_only` |
| Text typed into a working agent | prompt cut in half | R6 test; mutation `send-ignores-state` |
| Another project's session reached | leak across projects | R4/R5 tests; mutation `other-projects-listed` |
| Stale file after a crash believed | send into a closed app's guess | R3 staleness test |
| A `;`-ending text truncated by tmux | wrong text | reuse `tmux_send_input_arguments` (change 280) |

## Proof of completion

- `make q.fast` — fmt, clippy clean.
- `cargo test --locked agents_309_` — all pass.
- Live: two tmux sessions under a running build; `agents list/read/send/wait` behave as the spec says.

## Departures from the plan
