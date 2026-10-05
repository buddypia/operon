# Spec: choose, commit, and push from the screen the change was read on

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. The Changes view lists each changed file with a checkbox. Tracked
   modifications start ticked; untracked files start unticked, because an agent
   that left a scratch file behind should not have it committed by a person who
   did not look.
2. A commit stages exactly the ticked files and nothing else, with
   `git add -- <paths>`, then commits. Files that were staged before and are not
   ticked now are unstaged first, so what is committed is what the list shows.
3. The message is a free-text field. Committing with an empty message is
   refused; git's own refusal is never reached.
4. 「AI に書かせる」drafts the message from the staged change, using whichever of
   Claude Code, Codex, or Antigravity is present, in that order. The one used is
   named beside the button. The draft lands in the field and is edited by the
   person; it never becomes a commit on its own.
5. Drafting runs the CLI **non-interactively and read-only**, with the prompt as
   one argument, through the existing output-limited wrapper with a timeout. It
   cannot edit a file, and it cannot be the reason a commit happens.
6. The prompt carries the branch, the staged file list, and the staged patch,
   each within a stated ceiling; a patch past its ceiling is replaced by a
   sentence saying so rather than truncated mid-hunk.
7. Push refuses when the upstream is ahead: 「リモートが進んでいます。先に pull
   してください。」 There is **no** force push, in any spelling, anywhere in
   this change — a test asserts the string cannot appear in `src/`.
8. A branch with no upstream is pushed with `--set-upstream origin <branch>`;
   one with an upstream is pushed with a plain `git push`.
9. The upstream line says what is true: how far ahead, how far behind, or that
   there is no upstream yet.
10. Every git and CLI invocation added here goes through `src/exec.rs` with a
    timeout, and no argument is ever built by interpolating into a shell string.

## Behaviour

The screen was agreed before this was written:

```
┌─ Git ───────────────────────────── 1180px ─┐
│ 変更   差分   履歴                          ⟳   │
├──────────────────────────────────────┤
│ ☑ M   src/app.rs                      ✎  │
│ ☑ A   src/git/setup.rs                ✎  │
│ ☐ ??  notes.txt                       ✎  │
│                                             │
│ ┌──────────────────────────────────┐  │
│ │ リポジトリの本流から worktree を作る   │  │
│ │                                      │  │
│ └──────────────────────────────────┘  │
│ [ AI に書かせる ]        [ 2 ファイルをコミット ]│
│                                             │
│ main は origin/main より 1 コミット先   [ push ]│
└───────────────────────────────────────┘
```

The states that are not the happy one:

- **No changes.** 「作業ツリーに変更はありません。」 alone, as today. No
  message field, no commit button. The push line still shows, because a branch
  can be ahead with a clean tree.
- **Nothing ticked.** The button reads 「0 ファイルをコミット」 and is disabled.
- **Empty message.** 「コミットメッセージを入力してください。」
- **Drafting.** The button becomes 「書いています…」 and is disabled; the
  request is a background task like every other.
- **No CLI present.** The button is not drawn at all. There is nothing to
  explain and nothing to fix from this screen.
- **The draft fails or times out.** 「コミットメッセージを書けませんでした:
  {error}」 The field is left exactly as it was.
- **Commit refused by git** (a hook, an empty commit): git's stderr is surfaced.
- **Remote is ahead.** 「リモートが進んでいます。先に pull してください。」
- **No upstream.** 「origin に {branch} はまだありません。」 and the push button
  creates it.

## Design

`src/git.rs`:

```rust
pub(crate) struct UpstreamState {
    pub(crate) upstream: Option<String>,
    pub(crate) ahead: usize,
    pub(crate) behind: usize,
}
pub(crate) fn git_upstream_state(path: &Path) -> Result<UpstreamState>
pub(crate) fn git_stage_exactly(path: &Path, files: &[String]) -> Result<()>
pub(crate) fn git_commit(path: &Path, message: &str) -> Result<String>
pub(crate) fn git_push(path: &Path, state: &UpstreamState, branch: &str) -> Result<String>
pub(crate) fn git_staged_summary(path: &Path) -> Result<(String, String)>
pub(crate) fn commit_message_prompt(branch: &str, files: &str, patch: &str) -> String
pub(crate) fn clean_generated_commit_message(raw: &str) -> String
```

`git_upstream_state` is `git rev-parse --abbrev-ref --symbolic-full-name @{upstream}`
followed by `git rev-list --left-right --count <upstream>...HEAD`, which gives
behind and ahead in one line, in that order.

`git_stage_exactly` runs `git reset --quiet -- .` first so that a file unticked
after having been staged by hand does not ride along, then
`git add -- <paths>`. The reset is index-only and touches no working tree file.

`git_commit` passes the message as one `-m` argument. Arguments are a vector,
never a shell string, so a message containing quotes, newlines, or `$(…)` is a
message.

`clean_generated_commit_message` strips a leading and trailing code fence, the
surrounding quotes some models add, and any leading blank lines — for the
same reason as the other generators, because every model does at least one of
these at least sometimes.

`src/agents.rs`:

```rust
pub(crate) fn commit_message_agent(tools: &ToolStatus) -> Option<&'static str>
pub(crate) fn commit_message_command(agent: &str, prompt: &str) -> Option<(String, Vec<String>)>
```

The invocations, each the CLI's own documented non-interactive, read-only form:

| Agent | Arguments before the prompt |
|---|---|
| `claude` | `-p --output-format text --permission-mode plan` |
| `codex` | `exec --skip-git-repo-check -s read-only` |
| `gemini` (`agy`) | `--print --sandbox` |

`src/app.rs` gains `git_staged_files: HashMap<Uuid, HashSet<String>>`,
`commit_message_input`, a `BackgroundKey::CommitMessage` and
`BackgroundKey::GitMutation`, and the three actions. `src/app/screens.rs` draws
the list with checkboxes, the field, and the two buttons.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | The commit button is the existing filled-button treatment; the upstream line uses `palette.text_muted`. No new role. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No | Checkboxes are `egui`'s; the two buttons are text. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | Yes | The prompt ceilings are `src/config.rs` constants. Each CLI's non-interactive form exists once, in `commit_message_command`, and the round trip — that every agent Operon can launch also has a draft form — is what gets tested. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | Yes | No persisted shape changes. The tick set is per project and in memory: it is a selection, not work, and it is re-derived from `git status` on every refresh. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | Yes | Every call is `run_command_with_timeout` or `run_command_with_output_limit`. Arguments are vectors. The drafting CLIs run in their documented read-only modes. A test asserts no `--force`, no `-f` on a push, and no `+` refspec anywhere in `src/`. |
| Documentation — user-facing docs change in all three languages together | Yes | One bullet in each README. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Yes | `git push` talks to the person's own remote because they pressed push. The drafting CLI is the same binary Operon already launches into tmux, run locally. |
| Budgets — any new scan or output path states its byte and item ceiling | Yes | `COMMIT_PROMPT_PATCH_MAX_BYTES` and `COMMIT_PROMPT_FILES_MAX_BYTES` bound the prompt; `COMMIT_MESSAGE_MAX_BYTES` bounds what is read back; `COMMIT_MESSAGE_TIMEOUT_SECONDS` bounds the wait. |

## Flagged concerns

- **A commit is not reversible from this screen.** Deliberate: amend, reset, and
  force are all out of scope, and a person who wants them has a terminal. The
  irreversible thing this change adds is a commit, which git itself makes
  recoverable.
- **A model writes text that becomes a commit message.** It lands in a field the
  person edits and presses a separate button to use. It is never committed
  without that press, which requirement 4 states and the interface enforces by
  having two buttons rather than one.

## Acceptance

- `cargo test --locked` passes, including:
  - `reads_how_far_a_branch_is_from_its_upstream`
  - `stages_exactly_the_files_that_were_ticked`
  - `commits_a_message_that_would_break_a_shell`
  - `refuses_to_push_over_a_remote_that_moved`
  - `pushes_a_new_branch_with_its_upstream_and_never_with_force`
  - `no_source_file_can_force_a_push`
  - `builds_a_read_only_draft_command_for_every_agent_operon_launches`
  - `strips_the_fences_and_quotes_a_model_wraps_a_message_in`
- In the running app: tick two of three files, draft a message, edit it, commit,
  and push; the third file is still uncommitted and the branch is on the remote.

## Rejected alternatives

- **Hunk and line staging.** A second diff pane with its own selection model.
  Per-file is the granularity that matches the problem — an agent touched a file
  you did not want — and hunk staging is a change of its own.
- **Commit everything tracked.** The other option on the agreed screen. It
  cannot leave out the file the agent should not have touched, which is the case
  this exists for, and it cannot add a new file either.
- **A setting for which CLI drafts the message.** A settings row for a choice
  that has a right answer on almost every machine. The one used is named beside
  the button, which is the information the setting would have carried.
- **Force-with-lease as "the safe force".** It is safe against a remote that
  moved and not against a colleague who is mid-review. Out of scope, and the
  test makes adding it accidentally impossible.
- **Amend.** The single most common way to lose a commit that was already
  pushed. Its own change, with its own warning.
