# Plan: the custom-command gate admits a newline

- **Spec**: n/a (`bugfix` route — the bug report is in `./state.yaml`)
- **Approved**: 2026-11-04
- **Status**: approved

## Root cause

`is_safe_agent_command` in `src/agents.rs` admits any character for which
`char::is_ascii_whitespace` holds: space, tab, `\n`, form feed, `\r`. Only
space and tab are word separators to a shell; `\n` and `\r` end a command the
way `;` does, and the command reaches tmux as one shell string
(`respawn_tmux_pane`). Leading and trailing whitespace is trimmed first, so
only an interior line break gets through, which is exactly the injection.

## Fix

Admit `' '` and `'\t'` only. Nothing else in the predicate changes.

## Files that change

| File | Change |
|---|---|
| `src/agents.rs` | the whitespace arm of `is_safe_agent_command` |
| `src/tests.rs` | regression test with an embedded `\n`, `\r`, form feed refused and a tab admitted; `model_check_safe_agent_command_admits_no_shell_metacharacter`, the 242 model with its command arm restored |
| `docs/sdlc/changes/241-…/` | `state.yaml`, this plan, `approvals.log`, `verification.yaml`, `mutations.py` |

## Order of work

1. Add both tests; watch them fail on the unedited gate (`"a\nb"` admitted;
   the model reports the two-step counterexample).
2. Edit the gate; both pass, and the existing command tests stay green.
3. Mutate: put `is_ascii_whitespace` back, admit `\n` alone, admit `\r`
   alone; each must turn a test red.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A remembered command holding a tab or other whitespace now refused | a launch refused that used to run | tab stays admitted; only line breaks and form feed are dropped, which never separated words for the person who typed them |
| The model's alphabet misses a separator | a line-ending character still admitted | the alphabet holds `\n`, `\r`, form feed, vertical tab, tab and a non-ASCII space |
