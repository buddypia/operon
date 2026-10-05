# Intent: the automatic answer to a workspace-trust prompt confirms whichever option the CLI happens to have highlighted

- **Status**: approved
- **Opened**: 2026-09-15

## Problem

Operon offers to answer the one-time "do you trust this folder" question the
agent CLIs ask on their first run in a directory, so that opening a session in a
fresh worktree does not stop on a prompt nobody is watching. The answer it sends
is a bare Enter — the key that confirms the option already under the cursor.

Which option is under the cursor is the CLI's choice, not Operon's, and the three
CLIs do not agree. Observed on this machine on 2026-09-15, each launched by hand
in an empty temporary directory:

| CLI | what the prompt offers | under the cursor | a bare Enter |
|---|---|---|---|
| Claude Code | `❯ No, exit` then `Yes, I trust this folder` | **No, exit** | exits |
| Codex | `› 1. Yes, continue` then `2. No, quit` | Yes, continue | continues |
| Antigravity | `> Yes, I trust this folder` then `No, exit` | Yes, I trust this folder | continues |

So on Claude Code the automatic answer says *no*. The agent quits, the pane dies
with status 1, and a session Operon had already written into its store is left
holding a dead terminal. Nothing on screen says a question was answered, let
alone which way.

This is not a new mechanism gone wrong. `is_workspace_trust_prompt` has recognised
Claude Code's wording since it was written, and the test beside it quotes the
wording of the day — `❯ 1. Yes, I trust this folder`, numbered, with yes first.
Claude Code has since dropped the numbering and put `No, exit` first. The
recognition still works; the answer stopped being right, and nothing was watching
the answer.

## Who feels it, and when

Anyone who opens a Claude Code session in a folder Claude Code has not been run
in before: a newly created worktree, a project added today, a fresh checkout on a
second machine. It is invisible in day-to-day use of an existing project, because
a folder already trusted asks nothing — which is why it has gone unnoticed while
being on by default.

It is also what makes two of the six end-to-end restore tests fail. They restore
a conversation into a Claude Code terminal in a temporary directory, which is by
definition untrusted, and the pane dies before the restored conversation renders.

## Desired outcome

1. Opening a session in an untrusted folder with automatic approval on reaches a
   running agent on all three CLIs, not only on the two whose prompt happens to
   put yes first.
2. What Operon sends is derived from the prompt on screen rather than from a
   remembered default, so a CLI that reorders its options is answered correctly
   without a code change.
3. A prompt whose affirmative option cannot be identified gets **no keystroke at
   all**. Today an unrecognised layout still gets a blind Enter; refusing to
   answer leaves the question where a person can see it, which is the safe
   direction for a prompt about trust.
4. The next time a CLI rewords its prompt, that is discoverable by running one
   command rather than by losing a session.
5. `a_saved_codex_conversation_restores_into_a_resumable_claude_terminal` and
   `a_saved_antigravity_conversation_restores_into_a_resumable_claude_terminal`
   pass.

## Constraints this change inherits

- Strings matched against another product's output are matching patterns, not
  copy, and stay exactly as that product emits them — they are not translated.
- The affirmative answer must never be synthesised from a memorised key sequence;
  it is read from the captured pane, or it is not sent.
- macOS only; the prompt is read through `tmux capture-pane`, which is what
  Operon already uses to see a pane.

## Systems likely affected

- `src/tmux.rs` — `is_workspace_trust_prompt` and
  `auto_approve_workspace_trust_prompt`, the recognition and the answer.
- `src/tests.rs` — the guard over the answer, which does not exist yet.
- `scripts/` — a by-hand check that re-observes the three prompts, in the shape
  `scripts/check-transcript-vocabulary.sh` already established for facts that
  only the developer's own machine holds.

## Not in scope

- Whether automatic approval should be on by default. It is a setting with a
  checkbox, and this change is about it doing what it says rather than about
  whether to offer it.
- The other prompts an agent CLI asks. Only the one-time workspace-trust question
  is answered automatically, and that stays true.
