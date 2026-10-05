# Plan: read the registry, stop guessing

- **Spec**: `./spec.md`
- **Approved**: 2026-08-28
- **Status**: done

## Files that change

| File | Change |
|---|---|
| `src/config.rs` | the three Claude store directory names and the two registry read budgets |
| `src/cli.rs` | `ClaudeRegisteredSession`, `RegisteredConversation`, `read_claude_session_registry_in`, `registered_claude_conversation_in`, `claude_transcript_path_in`; `corrected_managed_claude_conversation_in` holds the decision and `corrected_managed_claude_conversation` becomes its `HOME` wrapper |
| `src/tests.rs` | six guards, all through the whole decision |
| `CLAUDE.md` | the identifier-SSOT section gains the Claude store directories; a line on what identifies a terminal's conversation |
| `docs/sdlc/lessons.md` | entry 008 |

## Order of work

1. `src/config.rs`, and route the five call sites that spelled
   Claude Code's projects directory inline through it. Tree compiles, suite unchanged.
2. The registry reader and the correlation, with no caller. Tree compiles.
3. `corrected_managed_claude_conversation` consults it. The existing fallback is
   left exactly as it is.
4. The six guards, each through `corrected_managed_claude_conversation_in`
   rather than the lookup it is assembled from.
5. Verify against the reported terminal: the registry entry for
   `/Users/someone/dev/buddypia/writing` names `7ee22502`, not the stored
   `f5cb1562`.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The correlation window is too tight on a cold start | the registry finds nothing and the old behaviour returns | by construction — a miss is the previous behaviour, not a new failure |
| The correlation window is too loose | a sibling terminal's conversation is restored | `two_claude_terminals_started_together_are_told_apart_or_refused`; the window is the existing constant and is not widened |
| A refusal blocks reopening as well as restoring | two terminals launched together in one workspace both stop opening | the same guard: the stored ID resolves the case where either terminal has not rotated, so only a genuine double-rotation refuses |
| The decision stops consulting the registry | every terminal silently returns to restoring its launch conversation | four guards routed through the decision. Verified by mutation: deleting the consult fails 5 tests, reordering it fails 1 |
| A future Claude Code stops writing the registry | every terminal falls back to today's ranking, silently | not caught automatically. Named here because requirement 5 makes the fallback deliberate: the failure returns, it does not become a new one |
| A sixth call site spells the projects directory inline | the next rename moves five of six | `the_claude_store_directories_are_spelled_in_exactly_one_place` |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. 288 passed; 0 failed; 6 ignored`, on
  three consecutive full runs.
- **Mutation-tested after an adversarial pass.** The first four guards all
  passed with the registry consult *deleted from the decision entirely* — the
  shape of lesson 004, where the check was covered and its caller was not. The
  guards were rewritten to go through `corrected_managed_claude_conversation_in`,
  and four mutations of the decision now fail: deleting the consult (5
  failures), moving it after the resume guard (1), dropping the stored-ID
  disambiguation (1), and letting the start-time check overrule a registry
  confirmation (1). Two further mutations confirm the SSOT guard: inlining
  either the projects or the registry directory name is caught, and Codex's own
  `sessions` is not.
- `cargo clippy --locked --all-targets -- -D warnings` — nothing past the
  compile lines.
- `bash scripts/check-transcript-vocabulary.sh` — `66 kinds observed locally,
  all classified`.
- `bash scripts/check-bands.sh` — one **warn** breach: `always_loaded_bytes`
  12375 against a warn tier of 11000, from the `CLAUDE.md` section this change
  adds on top of the one change 003 added. It reached the **diagnose** tier at
  13062 on the first draft; the section was cut to the rule and the account of
  why moved to lesson 008, which is not always-loaded. The band was not moved.
- The reported terminal, against the real store:
  `registered_claude_conversation_in` returns
  `7ee22502-a406-4c12-bce5-947aa6d8e971` where the record holds
  `f5cb1562-c38f-44ae-b804-0b7208a4066d`, and the correction resolves to that
  conversation's transcript.
- One intermittent failure seen in
  `starts_an_agent_command_in_a_retainable_tmux_session`, once in four full
  runs, passing in isolation and on the three runs after. It creates a real
  tmux session and touches nothing this change modifies; recorded as a
  pre-existing timing sensitivity rather than dismissed.
- `cargo test --locked -- --ignored`: 3 of 6 pass, unchanged by this change.
  The other 3 fail on the pre-existing defect named in change 003's plan and
  still unfixed: `auto_approve_workspace_trust_prompt` in `src/tmux.rs` answers
  the workspace-trust prompt with a bare `Enter`, and the current CLIs open that
  prompt with the cursor on `No, exit`, so the pane exits 1 before any restored
  content is rendered. It needs its own intent; it is also what keeps half the
  end-to-end restore coverage from running, which is why it is named here twice.
