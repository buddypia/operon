# Bugfix: managed hooks time out on a loaded machine

- **Route**: `bugfix`
- **Skill**: `.claude/skills/root-cause/SKILL.md`

## Cause

`HOOK_ENTRY_TIMEOUT_SECONDS` (10) is written as `timeout` into every managed
hook entry. At load average over 100 with 25 Claude sessions running, starting
`/bin/sh` for `claude-hook.sh` took 11 s on the path that posts nothing, so
Claude killed the hook and reported `timed out after 10s` (`UserPromptSubmit`
and the rest). The script's own post is capped at 1.5 s, so the budget only
ever has to cover process start; 10 s did not.

## Fix

The constant is 60. A healthy run is unchanged: it finishes in well under 2 s.

## Codex trust

Codex hashes the entry, timeout included, into `trusted_hash`. The trust keys
are by index and do not change; the hashes do. `apply_codex_config` already
drops every block whose key it is about to write before appending fresh ones,
and `request_hook_apply` runs at startup, so the first launch of the new build
re-trusts its own entries. No migration code was needed; a test now holds it.

The one exception is the one every reinstall already had: when `config.toml`
holds a table `apply_codex_config` cannot read, it drops nothing, so the old
block stays beside a `hooks.json` now saying 60 and Codex stops running our
hooks — closed and silent. This change makes that path reach every existing
install at once rather than only on a reinstall that changed something.

## Tests, both watched failing

- `hook_entry_timeout_outlasts_a_shell_start_on_a_loaded_machine` — failed at
  10 with `高負荷時のシェル起動 (実測 11 秒) に対して短すぎます: 10`.
- `codex_reinstall_replaces_trust_hashes_written_with_an_older_timeout` —
  seeds an install as an older build left it (timeout 10, its hashes) and
  reinstalls. Failed at 10 (old and new hash equal); with the drop of owned
  blocks in `apply_codex_config` mutated away it failed with
  `古い信頼ハッシュが残っています`.
- `claude_hook_install_is_idempotent_and_keeps_every_foreign_entry` now reads
  the timeout from the constant instead of the literal `10`.
