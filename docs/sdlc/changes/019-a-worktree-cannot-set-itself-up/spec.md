# Spec: a repository says once how a new checkout of it is prepared

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. A repository declares its setup by checking in an executable-or-not shell
   script at a fixed path inside its own tree. There is no configuration file
   and no schema: the file is the script.
2. After a worktree is created, Operon reads that path **inside the new
   worktree**, so a branch that changes its own setup is set up its own way.
   A worktree whose tree does not have the file behaves exactly as it does
   today: nothing is drawn and nothing runs.
3. A script is never run without the person having seen those exact contents.
   They are shown the first `SETUP_SCRIPT_PREVIEW_LINES` lines, the total line
   count, and the path, and they answer 実行する or 実行しない.
4. A person may tick 「内容が変わるまで、今後は確認しない」. That records the
   SHA-256 of the contents against that project. A later worktree in the same
   project whose script hashes the same runs without asking; any other hash,
   including a one-character edit, asks again.
5. The approval is checked again at the moment of running, against a fresh read
   of the file. A file that changed between being shown and being run is not the
   file that was approved, and the run is refused with a notice.
6. Running means a terminal session in the new worktree whose command is the
   script, so the person can watch it, scroll it, and kill it with everything
   that already works on a session. It is named for what it is.
7. The path is quoted where it reaches a shell. Every macOS data path contains a
   space, and every worktree path can — this is lesson 013's rule, and its guard
   here is a test that runs the built command under a directory with a space.
8. A setup session that ends with a non-zero status says so once, naming the
   status. A setup session that ends with zero says so once. Neither notice
   repeats, and neither blocks anything.
9. Nothing is read past `SETUP_SCRIPT_MAX_BYTES`. A file larger than that is
   treated as not being a setup script, with a notice, rather than being
   truncated into something whose preview does not match what would run.

## Behaviour

The screen was agreed before this was written. In the Worktrees tab, under the
create field, after a worktree is created and its script has not been approved:

```
┌─ worktree ──────────────────────────────────── 1180px ─┐
│ [ ブランチ名              ] [ ＋ worktree を作成 ]      │
│ 作成した worktree が次のセッションの作業フォルダに…     │
│                                                        │
│ ┌────────────────────────────────────────────────────┐ │
│ │ このリポジトリのセットアップを実行しますか？        │ │
│ │ .operon/setup.sh はリポジトリに入っているスクリプト │ │
│ │ です。tmux ウィンドウ「setup」で実行されます。      │ │
│ │                                                    │ │
│ │   #!/bin/sh                                        │ │
│ │   pnpm install                                     │ │
│ │   pnpm build                                       │ │
│ │   … 全 12 行                                       │ │
│ │                                                    │ │
│ │ ☑ 内容が変わるまで、今後は確認しない                │ │
│ │              [ 実行しない ]  [ 実行する ]           │ │
│ └────────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────┘
```

The block is the same inline group the removal confirmation already uses, in the
same place, so there is one way a confirmation looks on this tab rather than two.

The states that are not the happy one:

- **No script.** Nothing is drawn. This is most projects, and they must not pay
  a row for a feature they do not use.
- **Already approved.** Nothing is asked. The session starts and the notice
  reads 「セットアップを実行しています。」
- **Approved, then edited.** The hash no longer matches, so the block appears
  again with the new contents. This is the case the checkbox is for.
- **Larger than the ceiling.** 「.operon/setup.sh が大きすぎるため実行しません
  （上限 {p0} バイト）。」 Nothing is drawn and nothing runs.
- **Changed between being shown and being run.** 「.operon/setup.sh の内容が
  変わったため実行しませんでした。もう一度確認してください。」
- **Ends non-zero.** 「セットアップが終了コード {p0} で終わりました。セッション
  を開いて確認してください。」 The session stays in the list with its output,
  because `remain-on-exit` is already on for every Operon session.
- **Ends zero.** 「セットアップが完了しました。」
- **tmux missing.** The same refusal every session start already gives.

## Design

`src/git/setup.rs`, a child module of `src/git.rs` in the way `src/tmux/hooks.rs`
is a child of `src/tmux.rs` — because a new top-level module means a line in
`src/main.rs`, and `src/main.rs` is a paused surface.

```rust
pub(crate) struct SetupScript {
    pub(crate) path: PathBuf,
    pub(crate) digest: String,
    pub(crate) preview: Vec<String>,
    pub(crate) total_lines: usize,
}
pub(crate) enum SetupScriptRead { Missing, TooLarge(u64), Found(SetupScript) }
pub(crate) fn read_setup_script(worktree: &Path) -> SetupScriptRead
pub(crate) fn setup_command(script: &Path) -> String
pub(crate) struct SetupApproval { pub(crate) project: Uuid, pub(crate) digest: String }
pub(crate) struct SetupTrust { pub(crate) approvals: Vec<SetupApproval> }
pub(crate) fn load_setup_trust(data_file: &Path) -> SetupTrust
pub(crate) fn save_setup_trust(data_file: &Path, trust: &SetupTrust) -> Result<()>
pub(crate) fn is_setup_approved(trust: &SetupTrust, project: Uuid, digest: &str) -> bool
pub(crate) fn approve_setup(trust: &mut SetupTrust, project: Uuid, digest: &str)
```

`setup_command` is `format!("bash {}", shell_quote(path))`. It is a function
rather than a `format!` at the call site precisely so the quoting has one place
to be missing from, and one test to catch it.

The trust file sits beside the hook settings file under the data directory,
written with `write_file_atomically` through the same pattern
`load_hook_settings` / `save_hook_settings` established. It is not the store:
the store is a paused surface, and this is a per-machine approval rather than
work anybody would miss.

`src/app.rs` gains three fields — `pending_setup: Option<PendingSetup>`,
`setup_sessions: HashSet<Uuid>`, `setup_trust: SetupTrust` — and four methods:
`offer_setup` (called from the `WorktreeCreated` handler), `start_setup_session`
(the plain-terminal path with a different command and name), `ui_setup_prompt`
(the block above), and the arm in the session poll that turns a finished setup
session into one notice.

`setup_sessions` is in memory, not persisted. A restart forgets which sessions
were setup runs, which costs one notice that nobody is waiting for; persisting
it would mean a store field, and the store is paused. Recorded here so the next
reader knows it was decided rather than missed.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | The block reuses `palette.accent_soft` for its title and `palette.text_muted` for the preview, exactly as the removal confirmation does. No new role. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No | No new mark; the block has none. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | Yes | The script path, the byte ceiling, and the preview line count are `src/config.rs` constants. The launch command is built by `setup_command` and by nothing else. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | Yes | No persisted shape changes. The setup session is an ordinary `SessionOrigin::PlainTerminal` row, so nothing in `src/models.rs` moves. The trust file is written atomically and a corrupt one falls back to "nothing is approved", which fails towards asking. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | Yes | The only new child is a tmux session created by the existing `start_tmux_agent_session`, which is already timed out. The command reaching the shell is one `bash` with one quoted argument. `is_safe_agent_command` deliberately does not gate it: that gate exists for agent binaries chosen from a fixed set, and this is a script the person has just read and approved, which is a different guarantee made a different way. |
| Documentation — user-facing docs change in all three languages together | Yes | One bullet in each README naming the path. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Yes | Everything is local files and tmux. The script's own contents are the repository's business, and it runs on the person's machine with their approval — which is the reason for requirements 3 to 5. |
| Budgets — any new scan or output path states its byte and item ceiling | Yes | `SETUP_SCRIPT_MAX_BYTES` for the read, `SETUP_SCRIPT_PREVIEW_LINES` for what is drawn. One file, one stat, one read, only after a worktree is created. |

## Flagged concerns

- **This runs code out of a repository.** Resolved by requirements 3 to 5: the
  contents are shown before the first run, the approval is per project and per
  content, and the file is re-read and re-hashed at the moment of running so
  that what runs is what was approved. A repository whose script changes gets
  asked again. This is the whole reason the feature has a screen at all.
- **The approval is per machine, and per project.** Deliberate. A digest
  approved in one project does not carry into another, because trusting a
  repository is what is being approved, not trusting a string.

## Acceptance

- `cargo test --locked` passes, including:
  - `reads_a_setup_script_and_its_digest`
  - `treats_a_missing_setup_script_as_nothing_to_do`
  - `refuses_a_setup_script_past_its_byte_ceiling`
  - `previews_only_the_first_lines_and_counts_the_rest`
  - `approves_a_setup_script_per_project_and_per_content`
  - `an_edited_setup_script_is_not_the_one_that_was_approved`
  - `the_setup_command_survives_a_worktree_path_with_a_space`
  - `a_setup_script_runs_end_to_end_in_a_path_with_a_space`
- In the running app: a project with `.operon/setup.sh` shows the block after a
  worktree is created; 実行する starts a session that runs it; ticking the
  checkbox and making a second worktree runs it without asking; editing one
  character asks again.

## Rejected alternatives

- **A YAML project configuration.** Five keys, a schema, `uniqueKeys`, alias limits, and a
  YAML crate — which is a paused surface — to carry one field whose value is a
  shell script. The file being a shell script is the whole design.
- **Run it without asking.** Adding "open a repository, arbitrary code runs" to
  an application that today only runs `git` against a repository is an
  escalation, and it is not one a person can undo after the fact.
- **Approve by path rather than by content.** Then the approval survives an edit
  to the thing that was approved, which is the only thing the approval is about.
- **Run it inside the agent's own session as a second window.** The setup has to
  be able to run before there is an agent, because the reason to run it is that
  the agent should not have to. A session of its own is also killable on its own.
- **Make the agent wait for setup to finish.** Worth doing, and the design
  is known — a marker file and a wrapper. It is a second change, and it
  needs this one to exist first.
