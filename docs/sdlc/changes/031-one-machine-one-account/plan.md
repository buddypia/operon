# Plan: an account is a directory the hooks follow into

- **Spec**: `./spec.md`

## Files that change

- `src/config.rs` — the two variable names and the sidecar's file name.
- `src/agents/accounts.rs` — new. The whole model and every refusal.
- `src/agents.rs` — one `mod` line, and the re-export.
- `src/tmux/hooks.rs` — the path helpers split into a root and a file; the
  installer takes roots rather than one home.
- `src/app.rs` — the launch form's field, the launch, the resume, and the prune.
- `src/app/screens.rs` — the launch row and the settings section.
- `src/i18n_tables.rs`, three READMEs, the roadmap.

## Order of work

1. **`src/config.rs`**, so nothing else writes either variable name.
2. **`src/agents/accounts.rs`**, pure, with no caller. Everything except the two
   filesystem functions is decided over values that are passed in.
3. **Tests, watched failing by mutation.** Six, all without a window.
4. **`src/tmux/hooks.rs`.** The riskiest step, because it changes a path that
   already works. `claude_settings_path(home)` keeps its exact meaning as
   `claude_settings_in(claude_config_root(home))`, and a test asserts that
   equality so the split cannot move where the machine's own hooks go.
5. **`src/app.rs`**, then `src/app/screens.rs`.
6. Text, documents, roadmap.

## Risks

- **The path split moves the default install.** `CLAUDE_CONFIG_DIR` names the
  `.claude` directory itself while `home` names its parent, so the two shapes are
  one level apart and easy to conflate. Caught by
  `the_hooks_follow_every_account_that_is_registered`, which asserts the machine's
  own settings file is still at `home/.claude/settings.json` while an account's
  is at `<account>/settings.json`.
- **A launch under an account whose folder has been deleted.** tmux opens a
  window, the CLI fails on its first line, and a session record is left behind
  for a run that never started. Refused before the window, and
  `a_directory_that_is_not_there_is_refused_before_a_window_opens` is the guard.
- **A resume that forgets the account.** Change 027 made a resume carry the
  options it was launched with; an account is one of those, and dropping it would
  reopen a work conversation under a personal login. Caught by
  `a_session_reopens_under_the_account_that_had_it`.
- **A sidecar that only grows.** One record per session ever launched under an
  account. Pruned against the store's ids on every save, and
  `an_account_record_for_a_session_that_is_gone_is_dropped` watches it.
- **Work in a draw path.** The launch row lists accounts from a `Vec` already in
  memory; the sidecar is read at startup and after a registration, never per
  frame.
- **Writing into a directory that is not this application's.** Bounded by the
  same setting that already governs the write to `~/.claude/settings.json`, and
  removed by the same toggle. Named in `spec.md` rather than left implicit.

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. N passed; 0 failed; 6 ignored`, N six
  higher.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `bash scripts/check-bands.sh` — nothing newly breached. `steering_bytes` is
  already at `propose` and change 030 holds that.
- Each of the six new tests watched failing under a mutation aimed at what it
  guards, with the mutation confirmed to have landed where it was aimed.
- In the running app: register a second Claude directory, start a session under
  it, and watch the status chip move as the agent works.

## Departures from the plan

- **The three `*_path(home)` wrappers were deleted rather than kept.** The plan
  said `claude_settings_path(home)` would keep its exact meaning as the
  composition of the two halves, and that a test would assert the equality.
  After the installer took a root, nothing in production called any of the three
  — they existed only for the tests. A convenience function only the tests call
  is a second path that can drift from the one that ships, which is lesson 005's
  shape. The guard now walks the chain production takes, `account_roots` then
  `claude_settings_in`, and asserts the machine's own settings file is still at
  `home/.claude/settings.json`. That is strictly stronger than the equality the
  plan asked for.
- **Two existing tests broke, exactly where the plan said the risk was.**
  `claude_hook_install_is_idempotent_and_keeps_every_foreign_entry` and
  `codex_hook_install_writes_the_feature_flag_and_trust_entries_once` call the
  installers directly and were passing a home where a root is now wanted, so
  they wrote one level too high. Fixed by composing the root at the call site,
  not by changing what the installer means.
