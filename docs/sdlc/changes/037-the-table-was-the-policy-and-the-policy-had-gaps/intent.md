# Intent: a table that decides safety, kept by hand, and provably behind

- **Status**: draft
- **Opened**: 2026-09-08

## Problem

Change 036 made the acknowledgement gate read the command that will run rather
than the picker a person used. The set of arguments that count as "runs the
agent without asking" is a table this repository keeps by hand, and the review
of 036 found three entries missing from it — two of them by running the CLIs and
reading their own help text, which is the only way any of them could have been
found. Two were fixed inside that change. The third was left as a decision:
`codex --dangerously-bypass-hook-trust`, which the CLI itself labels DANGEROUS,
and which runs a repository's own hooks without the trust step that normally
gates them. A fourth, `claude --allowed-tools`, pre-approves a tool for the
whole session and is dangerous or harmless depending on which tool.

Adding three more strings is not the fix. A hand-kept table that decides whether
a person is asked before an agent is let loose will be behind again the next
time any of the three CLIs ships a flag, and nothing in the repository will
notice — the tests walk the table, so a flag the table does not name is a flag
no test can look for. The mechanism has to be able to be wrong about a specific
flag and still ask.

Two smaller versions of the same shape came out of the same reviews. The
matcher's own documentation says its normalisation is applied to both sides of
the comparison, and the mode arm is not normalised at all; it is harmless today
only because the two dangerous mode values happen to contain no dash and no
equals sign, which nothing states. And because the matcher now ignores leading
dashes, a bare word can match a flag: `aider --preset=yolo` asks for a dangerous
launch confirmation, which is the safe direction but is also how people learn to
tick the box without reading it.

Separately, the review of change 034 left two defects that were correctly judged
non-blocking and are still defects. A git worktree created *inside* the project
folder is treated as the project's own, so the tab carries no branch badge, the
Diff view is offered when it cannot be drawn correctly, and a file the agent has
just written reports itself unchanged. And the one place that decides to refresh
an editor diff does not ask whether the document may have one, so it discards a
cache and runs `git diff` one frame before the view refuses to draw.

## Who feels it, and when

The first, the day any of the three CLIs adds a way to skip its own permission
prompt — which is the situation the gate exists for and the one where it will be
silently absent.

The worktree defect, whenever somebody keeps worktrees under the project folder,
which is a common layout and the one `.worktrees/` implies.

## Desired outcome

- An argument that names itself as a permission escape is asked about even when
  no table has heard of it, and a table entry that says a flag is *not*
  dangerous still wins for that flag.
- A confirmation is asked for arguments, not for words that happen to appear in
  a value.
- The matcher's documentation describes what the matcher does.
- A document under any root is attributed to the root that actually contains it,
  and every place that acts on a document's diff asks the same question about
  whether it may have one.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- User-facing text is Japanese; code, comments, and docs are English.
- The suite runs six `#[ignore]`d tests and a seventh is a documented
  regression, so nothing here may depend on a live agent CLI.
- The screens agreed for changes 034 and 036 do not change. This makes them true
  in cases they already claimed to cover.

## Systems likely affected

`src/agents.rs` for the matcher, `src/app.rs` for the document root and the
diff-refresh question, `src/app/screens.rs`, `src/tests.rs`, and the three
`README`s, whose sentence about the gate can stop hedging once the gate stops
depending only on the table.

## Open questions

None that a person must answer. `--allowed-tools` is resolved by reading the
value rather than by policy: it is a pre-approval, and pre-approving a tool that
runs shell commands is the case worth asking about.

## Not in scope

Two features are not built: sorting the sessions that need a person to the top,
and fanning one prompt across several worktrees. Both have screens of their own.
`steering_bytes`, which change 030 owns and which is awaiting a decision.
