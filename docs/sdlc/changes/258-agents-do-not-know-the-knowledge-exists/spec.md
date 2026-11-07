# Spec: an agent session starts knowing the project's knowledge exists

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. `knowledge_brief(entries, executable)` returns `None` for no entries, and
   otherwise a text of at most `KNOWLEDGE_BRIEF_MAX_CHARS` (1,500) characters
   that says: the project keeps knowledge distilled from earlier agent
   conversations here, with the number of entries; that it exists so an agent
   does not re-derive what an earlier session found; to search it before
   investigating a cause, choosing an approach, or answering how the project
   works, with the exact command — the executable shell-quoted, then
   `knowledge search "<what you need>" --json`; that 検証済み entries were
   confirmed by a test, command output, or the person and 未確認 ones should be
   checked before relying on them; and then the titles of the entries worth
   knowing now, one per line as `- [kind・status] title`. Bodies are never
   included.
   Test: `knowledge_258_brief_says_what_the_knowledge_is_for_and_how_to_search`.
2. The listed entries are at most `KNOWLEDGE_BRIEF_ENTRIES` (10): pinned first,
   then 検証済み, then by freshness; and the whole text stays inside the ceiling
   with `KNOWLEDGE_MAX_ENTRIES` entries at their longest titles.
   Test: `knowledge_258_brief_lists_pinned_then_verified_then_freshest`.
3. `knowledge_brief_cli(directory, projects, root, executable)` prints, for a
   directory inside a registered project's folder (matched by path alone, with no
   `git` call, so a worktree outside the project's folder gets no brief) whose
   knowledge is on and non-empty,
   one JSON object `{"hookSpecificOutput": {"hookEventName": "SessionStart",
   "additionalContext": "…"}}`, whose `additionalContext` is the brief, and exits 0. For a directory outside every
   project, a project without a database, a disabled project, or an empty one it
   prints nothing and exits 0. It needs no embedding model, counts no
   reference, and creates no database. `Operon knowledge brief` runs it with the
   working directory and the running executable, and gives up after
   `KNOWLEDGE_BRIEF_SECONDS` (3): the work runs on a thread, and when it has not
   answered by then the command prints nothing and exits 0.
   Test: `knowledge_258_brief_cli_answers_session_start_or_says_nothing`, and
   `knowledge_258_a_slow_brief_gives_up_and_says_nothing` for the bound
   (`bounded(seconds, work)` returns `None` for work that outlives it).
4. The managed hook script for Claude Code and for Codex, run with
   `OPERON_HOOK_EVENT=SessionStart` and `OPERON_EXECUTABLE` naming an executable
   that prints a brief, prints exactly that brief on stdout and nothing else
   before reading the payload — only when what it printed starts with
   `{"hookSpecificOutput":`, so stray output is never handed to the CLI; the brief command gets `/dev/null` as stdin, so
   the payload still reaches the post. For any other event, or when the
   executable is unset, missing, fails, prints nothing, or prints anything
   else, Claude Code's script
   prints `{}` and Codex's prints nothing, as today. Antigravity's script is
   unchanged.
   Test: `knowledge_258_session_start_hook_hands_the_agent_the_brief`.
5. `HookEnvironment` carries the Operon executable, and its variables include
   `OPERON_EXECUTABLE` (a constant in `src/config.rs`) with that path; a launch
   sets it to the running executable.
   Test: `knowledge_258_hook_environment_names_the_operon_executable`.
6. Before knowledge is turned on, プロジェクト › ナレッジ adds one bullet:
   「・Claude Code と Codex のセッション開始時に、ピン留め・検証済み・最近使われたものを優先してナレッジの題名 (最大 10 件) と検索方法をその CLI に渡します。他の CLI との会話から作られた題名も含まれます。」
   Once it is on, the same tab shows under its status line, in the muted small
   style of that line:
   「Claude Code と Codex のセッション開始時に、ナレッジの題名 (最大 10 件) と検索方法をその CLI に渡しています。」
   so a project turned on before this change says so too.
   Test: `knowledge_258_opt_in_says_what_agents_are_given`.

## Behaviour

- A Claude Code or Codex session started by Operon in an opted-in project with
  entries begins with the brief as context (SessionStart fires on start,
  resume, clear, compact, and fork). The person sees nothing new in the terminal
  beyond what the CLI shows for hook context.
- Projects without knowledge, sessions with hooks off, and Antigravity sessions
  start as today.
- Failure states: a missing or failing executable, a locked database, or a store
  that cannot be read all print nothing, and the hook answers as today. The
  command gives up after 3 seconds (Req 3), so a start is never held longer
  than that, whatever is slow underneath.
- Opt-in card (disabled state) gains the bullet in Req 6, beside the existing
  three; the enabled tab gains the one muted line. Translations in every table.

## Design

Placement verdict: **EXTEND** — `src/knowledge.rs` owns the knowledge CLI,
`src/tmux/hooks.rs` the hook script and its environment, `src/app/knowledge.rs`
the opt-in card (`git grep -n "fn hook_script\|fn run_cli\|fn ui_knowledge_disabled" src`).

- `src/config.rs`: `HOOK_EXECUTABLE_ENV = "OPERON_EXECUTABLE"`,
  `KNOWLEDGE_BRIEF_SUBCOMMAND = "brief"`, `KNOWLEDGE_BRIEF_ENTRIES = 10`,
  `KNOWLEDGE_BRIEF_MAX_CHARS = 1_500`, `KNOWLEDGE_BRIEF_SECONDS = 3` (1,500
  characters stays under Codex's ~2,500-token cap even at one token per
  Japanese character, and under Claude
  Code's 10,000-character hook-context limits, measured from their docs
  2026-11-08).
- `src/knowledge.rs`: `knowledge_brief` (pure), `knowledge_brief_cli` (picks the
  project whose folder contains the directory by path alone, opens with
  `open_existing`, reads `enabled` and `entries`), and `bounded(seconds, work)`
  (runs work on a thread, waits with `recv_timeout`), routed in `run_cli` before
  the embedder is looked up; on timeout the process exits without waiting for
  the thread. The text is agent-facing, in English like the judge
  prompt; entry labels go through `tr` as in the search output.
- `src/tmux/hooks.rs`: `HookEnvironment.executable: Option<PathBuf>`, added to
  `variables` when present. `hook_script` for Claude Code and Codex starts with
  a `brief` variable filled only on `SessionStart` by
  `"$OPERON_EXECUTABLE" knowledge brief </dev/null 2>/dev/null || brief=''`,
  then prints it when non-empty, else the provider's answer as today. The
  managed command in the CLI settings is unchanged, so Codex's trusted hashes
  stay valid.
- `src/app.rs`: `mint_hook_environment` sets
  `executable: std::env::current_exe().ok()` — a launch-time call, not the draw
  path.
- `src/app/knowledge.rs`: one more bullet label in `ui_knowledge_disabled`, and
  one muted line under the enabled tab's status line (static text, no I/O).
- External command: the agent CLI runs the hook script, which runs Operon's own
  binary with fixed arguments `knowledge brief`, stdin `/dev/null`, stderr
  discarded; no path from the payload reaches it. Operon's Rust code spawns
  nothing new.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | no, read | the bullet reuses the card's existing label style |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | no | none |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | yes, read | `OPERON_EXECUTABLE` and `brief` are constants used by both the environment / router and the script; Req 4 runs the script with the variable `variables()` produces, Req 3 routes the subcommand constant |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | read | no schema change; the brief reads with `open_existing` and never creates a database or writes a reference |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | yes | no new child of Operon; the hook script's new child is Operon's binary with constant arguments, stdin from `/dev/null`, every failure path ending in the old answer and `exit 0` (`every_managed_hook_script_is_valid_shell_that_cannot_fail` still holds); the launch command is unchanged |
| Documentation — user-facing docs change in all three languages together | yes | README knowledge bullets in three languages gain one line each |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | yes | the brief goes only to the agent CLI the person is already running in that project, titles only, opted-in projects only |
| Budgets — any new scan or output path states its byte and item ceiling | yes | ≤ 10 titles, ≤ 1,500 characters (Req 2); one read of the store file and of the entries table, bounded to 3 seconds (Req 3) |

## Flagged concerns

- **Titles from one CLI's conversation reach another CLI's provider.** Resolved:
  `knowledge search` already returns whole entries to any agent in the project,
  and the judge already summarises across CLIs (change 225); the brief sends
  less (titles only) and only for projects the person opted in, and Req 6 says
  so before opting in.
- **A slow brief delays every session start.** Resolved: no model load, no
  `git` call, and a hard 3-second bound in the command itself (Req 3); any
  failure is silent.
- **Projects turned on before this change.** Their owners never saw the new
  bullet, because the opt-in card is gone once knowledge is on. Resolved: the
  enabled tab carries a standing line saying the CLIs are given the titles
  (Req 6), and turning knowledge off stops the brief, since a disabled project
  prints nothing (Req 3).

## Acceptance

- `cargo test --locked knowledge_258_` passes all seven tests, each watched failing
  first.
- `cargo test --locked hook` and `cargo test --locked knowledge_` pass.
- `make q.fast` clean.

## Rejected alternatives

- `--append-system-prompt` on the launch command: changes the launch gates and
  only covers sessions Operon launches fresh, not resume or compact.
- Writing the index into `CLAUDE.md` / `AGENTS.md` in the project: puts private
  knowledge in the working tree.
- Including entry bodies: the top layer must stay small, and wrong notes spread
  from it; search returns bodies on demand.
