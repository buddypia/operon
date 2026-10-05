# Spec: the gate stops depending on the table being complete

- **Intent**: `./intent.md`
- **Status**: draft

## Requirements

1. A dashed argument whose name marks it as a permission escape asks for the
   acknowledgement even when no table names it.
2. A table entry wins for the argument it names, in both directions: a flag the
   table calls dangerous asks, and a flag the table calls safe does not, whatever
   its name looks like.
3. Only arguments — tokens written with a leading dash — are matched against the
   flag and alias tables. A value that happens to spell a flag's name is not one.
4. Mode values are still matched as bare values, because that is how they appear
   on a command line, and they go through the same normalisation as everything
   else.
5. `--allowed-tools` asks when the tools it pre-approves include one that runs
   shell commands, and does not otherwise.
6. A document is attributed to the innermost root that contains it, so a
   worktree inside the project folder is the worktree's.
7. Every place that acts on a document's diff asks one shared question about
   whether that document may have one.
8. `OpenDocument::named` composes its string through the translation layer.

## Behaviour

Nothing new is drawn. Every visible consequence is one of the screens agreed for
changes 034 and 036 appearing in a case it already claimed to cover:

- `codex --dangerously-bypass-hook-trust` typed into 起動コマンド now raises
  「警告: 確認もサンドボックスもなしにファイルを変更できます。」 and the
  「理解したうえで起動する」 checkbox, and the launch button stays disabled until
  it is ticked. So does any future flag of any of the three CLIs whose name
  carries `dangerously`.
- `claude --allowed-tools Bash` raises the same. `claude --allowed-tools Read`
  does not.
- `claude --allow-dangerously-skip-permissions` still does not, because
  `CLAUDE_FLAGS` says that flag is not dangerous and the table wins for a flag
  it names. That is the one place the heuristic must not override.
- `aider --preset=yolo` no longer raises it. `codex --yolo` still does.
- A document opened from a worktree at `<project>/.worktrees/fix-login` now
  carries its branch on the tab and is not offered the Diff view, exactly as a
  worktree outside the project folder already was.
- A save notice for such a document reads `fix-login · src/app.rs を保存しました。`
  rather than naming the path alone.

## Design

**`src/agents.rs`**

- `launch_argument_tokens` yields whether each token was written with a leading
  dash alongside the normalised text, so the flag, alias, and heuristic arms can
  require an argument while the mode arm keeps matching a bare value.
- `argument_names_a_permission_escape(token) -> bool` — the heuristic. A
  normalised argument that contains `dangerously`, or that is `yolo`, names one.
  Kept deliberately small: it is a net under the tables, not a second table.
- `command_needs_acknowledgement` asks, in order: does any table name this exact
  argument (and if so, what does it say), then the aliases, then the modes, then
  the heuristic for arguments no table named. The table's answer is final for an
  argument it names, so `--allow-dangerously-skip-permissions` stays safe.
- `PRE_APPROVING_ARGUMENTS` — `allowed-tools` and `allowedTools`, whose *value*
  decides: an entry naming `Bash` pre-approves shell execution.

**`src/app.rs`**

- `document_root_for` picks the longest matching root rather than the first,
  which is the rule `src/git/ports.rs` already uses to attribute a listening
  socket to the deepest worktree containing it.
- `document_may_have_a_diff(&self, id) -> bool` — the one question, called by
  the view switcher, the 差分を見る button, and `invalidate_open_editor_diff`.
- `OpenDocument::named` uses `tf!("{p0} · {p1}")`.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | Nothing new is painted. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No | No new mark. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | Yes | The heuristic reads no flag name that a table also spells; the tables stay the source of truth for the flags they name, and the guard drives both from the tables. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | No | No persisted shape changes. `document_root_for` decides at open, from paths already stored. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | Yes | No new child. The change is entirely inside the gate that decides whether a launch is confirmed, and it only ever adds cases where it asks. |
| Documentation — user-facing docs change in all three languages together | Yes | The three READMEs stop hedging: the sentence narrowed in 036 can say what the gate now does. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Yes | Nothing leaves the machine; no dependency added. |
| Budgets — any new scan or output path states its byte and item ceiling | Yes | No new scan. The heuristic is two string comparisons per argument of a line `is_safe_agent_command` has already bounded, and runs only for the custom agent. |

## Flagged concerns

- **The heuristic can over-ask.** A custom command carrying an argument named
  `--dangerously-something` that is in fact harmless will demand a checkbox.
  **Resolved**: accepted, and bounded — a table entry overrides it for any flag
  a table names, so the only over-asking is on arguments this application has
  never heard of, where asking is the answer.
- **`--allowed-tools` is judged by its value.** `Bash` is the tool that runs
  shell commands; a comma list or a scoped form like `Bash(git:*)` still names
  it. **Resolved**: matched on the value containing `Bash`, stated here so the
  next reader does not think it was a whole-token comparison.
- **Requirement 6 changes which root an already-open document would get.**
  **Resolved**: the root is decided at open and stored, so no open document
  moves; only documents opened after this behave differently.

## Acceptance

- `cargo test --locked` passes, including:
  - `a_permission_escape_no_table_names_still_asks`
  - `a_table_that_calls_a_flag_safe_outranks_the_heuristic`
  - `a_value_that_spells_a_flag_is_not_a_flag`
  - `pre_approving_a_shell_tool_asks_and_pre_approving_a_reader_does_not`
  - `a_document_belongs_to_the_innermost_root_that_holds_it`
  - `one_question_decides_whether_a_document_may_have_a_diff`
  - `a_save_notice_composes_its_branch_through_the_translation_layer`
- Each watched failing before it is watched passing.
- In the running app: type `codex --dangerously-bypass-hook-trust` into
  別のコマンドを使う and the warning appears; make a worktree under the project
  folder, open a file from its session, and the tab carries the branch.

## Rejected alternatives

- **Add the three missing flags to the tables and stop.** That is the fix that
  guarantees a fourth. The reviews found these by running `--help`; nothing in
  this repository can.
- **A test that reads each CLI's `--help` and checks the table.** It needs a live
  agent CLI, so it would be a seventh `#[ignore]` — a number `CLAUDE.md` treats
  as a documented regression, and `gate-commit.sh` refuses a commit that gains
  one.
- **Match any argument containing `dangerously` with no table override.** It
  would fire on `--allow-dangerously-skip-permissions`, which `CLAUDE_FLAGS`
  deliberately calls safe — two sources disagreeing about one flag, which is the
  defect change 036 was about.
- **Treat every `--allowed-tools` as dangerous.** Pre-approving `Read` is not a
  reason to make somebody sign something.
