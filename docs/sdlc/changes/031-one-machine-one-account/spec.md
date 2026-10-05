# Spec: an account is a directory, chosen at launch, that the hooks follow into

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. An account is a name and a directory, for one of the two providers that have
   a variable for it. Registering one neither reads nor writes anything inside
   that directory beyond what requirement 4 installs.
2. The launch screen offers the accounts registered for the chosen agent, plus
   the machine's own login, which is what every launch uses today and stays the
   default.
3. Launching with an account sets that provider's variable for the session, and
   nothing else about the launch changes.
4. The managed hooks and the usage reader are installed into every registered
   account's directory as well as the machine's own, so agent state, unread, and
   usage work the same under any account. Turning the hook setting off removes
   them from all of them.
5. Which account a session used is remembered, so reopening its conversation
   reopens it under the same account. A record for a session that is gone is
   dropped rather than kept forever.
6. A directory that is not there, or not a directory, is refused at the moment
   it is registered, with a sentence saying which.
7. Nothing about an account leaves this machine, and no credential is read.

## Behaviour

The launch screen gains one row between the agent and the model:

```
エージェント  [claude      ▾]
アカウント    [このマシン   ▾]
モデル        [opus        ▾]
権限モード    [acceptEdits ▾]
```

The row is drawn only for `codex` and `claude`, because they are the two with a
variable. Its first entry is `このマシン`, which is today's behaviour and is what
a launch uses when nothing is chosen. Registered accounts follow, by name.

Accounts are registered in settings, under the hooks section they affect: a name,
a folder picker, and the list with a remove button. The section says in one line
what an account is, because "a directory the CLI keeps its login in" is not
something to make a person infer:

> アカウントは、CLI がログイン情報を置くフォルダです。Operon はその場所を指すだけで、
> 中身を読むことはありません。Codex は `CODEX_HOME`、Claude Code は
> `CLAUDE_CONFIG_DIR` を見ます。

Not-happy states:

- **The folder is gone when a session is launched.** The launch is refused
  before tmux opens a window, with
  `アカウント「{name}」のフォルダが見つかりません。`, because a window that opens,
  fails, and dies leaves a session record behind for nothing.
- **The folder is gone when hooks are installed.** That account is skipped and
  named in the hook status list, in the same place a missing CLI is named.
- **The same directory registered twice.** Refused: two names for one account is
  two rows that cannot be told apart afterwards.
- **The machine's own login is never a registered account.** It is always
  present, cannot be removed, and is what an empty choice means.

## Design

`src/agents/accounts.rs`, a child of `agents`:

- `AgentAccount { id, provider, name, path }` and `AgentAccounts { accounts }`,
  with the sidecar pair `load_agent_accounts(data_file)` /
  `save_agent_accounts(data_file, &accounts)` following `src/git/setup.rs`'s
  shape, written through `write_file_atomically`.
- `account_variable(provider) -> &'static str` — `CODEX_HOME` for Codex,
  `CLAUDE_CONFIG_DIR` for Claude, `None` for Antigravity. Confirmed against the
  installed binaries: a `claude` run with `CLAUDE_CONFIG_DIR` pointed at an
  empty directory created its state there, and `codex resume --help` names
  `$CODEX_HOME` as where a profile is layered from.
- `account_variables(accounts, id) -> Vec<(&'static str, String)>` — what the
  launch appends to the hook variables it already passes.
- `register_account(accounts, provider, name, path) -> Result<_, String>` — the
  refusals of requirement 6, pure over a path that is passed in.
- `session_accounts` — the session-to-account record of requirement 5, kept in
  the same sidecar and pruned against the store's session ids on every save.

`src/tmux/hooks.rs` gains the shape the accounts need. `claude_settings_path`
today is `home/.claude/settings.json`; `CLAUDE_CONFIG_DIR` names the `.claude`
part directly, so the path helpers split into a root and a file:
`claude_config_root(home)` and `claude_settings_in(root)`, with the existing
function keeping its meaning as the composition of the two. The installer takes
a list of roots rather than one home, and the list is the machine's own root plus
every registered account's directory.

`src/app.rs` holds the chosen account on the launch form and passes it to the
launch; `resume_command_with_launch_options`'s caller looks the source session's
account up and carries it.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | The row uses the launch screen's existing combo styling. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No | No new mark. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | Yes | The two variable names are agreed between the launch and the hook installer. They live once, in `src/config.rs`, and the test asserts the round trip: what the launch sets is the root the installer wrote into. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | Yes | No store change. The sidecar is written through `write_file_atomically`; a corrupt or absent one means no accounts, which is today's behaviour and the direction that fails safe. The session-to-account records are pruned on every save, so the file cannot grow without bound. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | Yes | No new child. The directory reaches tmux as one `-e KEY=VALUE` argv element, never through a shell, and a path that is not an existing directory never reaches a launch. |
| Documentation — user-facing docs change in all three languages together | Yes | One bullet in each README. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Yes | "No accounts" in that rule means no Operon account. This adds no service and no request; it names a local directory that another program already owns. |
| Budgets — any new scan or output path states its byte and item ceiling | Yes | The sidecar is read with the same 64 KiB ceiling the keymap uses, and the hook installer's work is bounded by the number of registered accounts. |

## Flagged concerns

- **Installing into a directory the person named.** Requirement 4 writes a hooks
  entry into a folder that is not this app's. It is the same write the app
  already makes to `~/.claude/settings.json`, under the same setting, removed by
  the same toggle — but it is worth naming, because the number of directories it
  touches is now something a person chooses. Settled by the person on
  2026-09-06: the hooks follow the account, because the alternative is three
  shipped features silently going dark on a switch.

## Acceptance

- `cargo test --locked` passes, including:
  - `an_account_is_one_variable_and_the_launch_carries_it`
  - `the_hooks_follow_every_account_that_is_registered`
  - `a_directory_that_is_not_there_is_refused_before_a_window_opens`
  - `the_same_directory_cannot_be_registered_twice`
  - `a_session_reopens_under_the_account_that_had_it`
  - `an_account_record_for_a_session_that_is_gone_is_dropped`
- In the running app: register a second Claude directory, start a session under
  it, and confirm the status chip moves as the agent works — which is the proof
  that the hooks went in.

## Rejected alternatives

- **A field on `Session`.** The store is paused, and this would be a schema bump.
- **A managed runtime home with `config.toml` mirroring.** It exists so
  the app's hooks do not collide with the user's own in `~/.codex`. Operon installs
  into the directory the person named, so there is nothing to isolate from.
- **A toolbar chip.** Right where a swap
  rewrites a credential pointer. Here a switch is one variable at launch,
  so the launch screen is where it belongs.
- **Leaving the hooks behind.** Ships a switch that silently turns off agent
  state, unread, and usage for any account that is not the default.
