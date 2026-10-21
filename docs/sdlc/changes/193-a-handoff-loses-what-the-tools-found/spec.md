# Spec: A handoff carries the latest tool results, shortened, and never as conversation

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. A handoff bundle ends its excerpt with a section holding the last
   `HANDOFF_TOOL_RESULTS_MAX` (3) tool results of the source conversation,
   oldest first; an older one is not carried. Test:
   `a_handoff_carries_the_latest_tool_results_shortened`.
2. A tool result longer than `HANDOFF_TOOL_RESULT_MAX_CHARS` (1 000) is
   shortened by rule: a run of identical consecutive lines becomes one line
   followed by `[repeated N times]`; then, if still too long, the first
   `HANDOFF_TOOL_RESULT_HEAD_LINES` (3) and last
   `HANDOFF_TOOL_RESULT_TAIL_LINES` (5) lines are kept, together with the
   latest `HANDOFF_TOOL_RESULT_SIGNAL_LINES` (6) lines between them that
   contain a word of `HANDOFF_TOOL_RESULT_SIGNAL_WORDS` (`error`, `fail`,
   `panic`, `fatal`, `exception`, `traceback`, case-insensitive); every gap
   becomes one `[N lines omitted by Operon; the full output is in the transcript snapshot]`
   line; a kept line over `HANDOFF_TOOL_RESULT_LINE_MAX_CHARS` (160) is cut and
   ends `[line shortened by Operon]`. Signal lines give way earliest-first when
   the result is still too long, and as a last resort the result is cut with
   `HANDOFF_SHORTENED_MARKER`. Test:
   `a_shortened_tool_result_keeps_its_error_lines_and_counts_repeats`.
3. No part of a `[Tool call: …]`, `[Tool result]`, or `[Thinking]` block —
   including paragraphs after a blank line inside it — appears in the excerpt
   as something the user or the assistant said; reasoning appears nowhere in
   the bundle. Test: `a_tool_result_with_blank_lines_never_reads_as_conversation`.
4. Over the byte budget, what gives way is, in order: older excerpt turns (down
   to the last one), tool results oldest-first, then the existing sequence
   (the last turn shortened, the excerpt, the task, the note). The bundle never
   exceeds `HANDOFF_BUNDLE_MAX_BYTES`. Test:
   `a_handoff_gives_up_tool_results_before_the_last_turn_task_or_note`.
5. A bundle from a conversation with no tool call, tool result, or reasoning
   block is byte-for-byte what it was before this change. Test: the existing handoff tests
   (`a_long_handoff_keeps_the_persons_instruction_whole`,
   `a_handoff_says_where_operon_shortened_the_text`, and the rest of
   `cargo test --locked handoff`) pass unchanged.

## Behaviour

Nothing a person reads in the app changes: no new string, notice, or control.
The bundle is English text handed to the destination agent as its first
prompt, like the rest of the bundle. The new section, after the excerpt and
before the person's instructions:

```
Latest tool results from the source session (oldest first; the full output of each is in the transcript snapshot):
### Tool result
<shortened output>
### Tool result
<shortened output>
```

States:

- **No tool result** in the source: no section; the bundle is as before.
- **A tool result that is empty** after trimming: not carried, and not counted
  toward the three.
- **Too large**: a result past 1 000 chars is shortened as Requirement 2 says;
  a bundle past 6 000 bytes gives way as Requirement 4 says.
- **Interrupted / error**: no new failure path. Shortening is pure string work
  on text already read; it cannot fail.

## Design

**Placement: `EXTEND`.** The behaviour lives in the handoff bundle builder,
`build_compact_handoff_bundle` and `compact_handoff_turn_text` in
`src/history.rs`, which already decide what of a turn is carried
(`git grep -n compact_handoff_turn_text`). Route `modify`.

- `src/history.rs`
  - A block splitter over a turn's text: chunks separated by a blank line; a
    chunk starting with `[Tool call:`, `[Tool result]`, or
    `RESTORED_REASONING_LABEL` opens a block of that kind; a chunk starting
    with none of them continues the previous block when that block is a tool or
    reasoning block, and is said text otherwise. This is what stops the leak:
    today each chunk is judged alone. Antigravity's reader writes a record's
    said text before its reasoning and tool calls, and Claude Code and Codex
    write one block per record in practice, so said text after a tool block is
    rare; where a record does hold one (a user record with a tool result and
    then an interruption note), that text is carried in the tool section under
    the wrong heading — still shown, never dropped. The opposite mistake, tool
    output read as said text, is the one this change removes.
  - `compact_handoff_turn_text` keeps said blocks only.
  - `handoff_tool_results(turns)`: the bodies of the last three non-empty
    `[Tool result]` blocks across the turns, each through
    `shorten_tool_result`.
  - `shorten_tool_result(text) -> String`: Requirement 2. Signal candidates are
    capped at six before any combination is tried, so the work is linear in
    the result's length.
  - `build_compact_handoff_bundle`'s `render` gains the tool slice; its give-way
    loop becomes the order of Requirement 4. The return value
    `(bundle, included_turns)` and the manifest are unchanged.
- `src/config.rs`: the seven constants named in Requirements 1–2, beside the
  existing `HANDOFF_BUNDLE_*` ones. The omission and line-cut wordings are
  constants there too, since the tests read them back.
- No persisted shape changes. The reused-archive path already rebuilds the
  bundle from the snapshot, so an archive written before this change yields
  the new bundle on reuse, and the manifest schema version stays 6.
- No external command, no background thread, no draw-path work: the builder
  already runs on the handoff's background task.
- `README.md`, `README.ja.md`, `README.ko.md`: the handoff bullet says the
  latest tool results cross shortened.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | Nothing is drawn; the bundle is text for an agent. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No | No glyph. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | Yes | The omission line, the line-cut suffix, the repeat suffix, and the signal words are constants in `src/config.rs`; the tests assert on those constants, not on retyped literals. The tool markers the splitter reads are the ones the readers write in this file; `RESTORED_REASONING_LABEL` is used by name. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | Yes (read) | No store, sidecar, migration, or archive-shape change, so no version bump. The snapshot stays byte-for-byte; the bundle is still written through `write_file_atomically`. The bundle's byte budget is unchanged and still enforced on every path (Requirement 4). |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | No | No child process, no launch-path change; the bundle reaches the destination through the existing `agent_shell_command` quoting. |
| Documentation — user-facing docs change in all three languages together | Yes | The handoff bullet in `README.md`, `README.ja.md`, and `README.ko.md` changes together. The manuals do not describe the bundle's contents. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Yes | Rule-based string work on this machine; no model, network, or `unsafe`. |
| Budgets — any new scan or output path states its byte and item ceiling | Yes | At most three results carried, each at most 1 000 chars, lines at most 160 chars, at most six signal lines considered; the whole bundle stays within the existing 6 000-byte `HANDOFF_BUNDLE_MAX_BYTES`. Input is already bounded by `TRANSCRIPT_FILE_MAX_BYTES`. |

## Flagged concerns

- **Tool output can carry secrets into another CLI.** A result that printed an
  environment or a token now crosses to the destination, where before it was
  dropped. Answered: the same text is already in the transcript snapshot the
  bundle names and in the source CLI's own store on the same machine, and the
  destination is a CLI the person chose on the same machine; nothing leaves
  the machine that a handoff did not already hand over. The shortening also
  keeps at most 1 000 chars of any one result.
- **Older turns now give way to tool results.** A bundle that fitted eight
  turns may carry fewer turns plus tool results. Answered: the intent ranks
  the latest tool results above older chatter but below the last turn, the
  task, and the note; the full conversation is in the snapshot.

## Acceptance

- `cargo test --locked handoff` passes, including
  `a_handoff_carries_the_latest_tool_results_shortened`,
  `a_shortened_tool_result_keeps_its_error_lines_and_counts_repeats`,
  `a_tool_result_with_blank_lines_never_reads_as_conversation`, and
  `a_handoff_gives_up_tool_results_before_the_last_turn_task_or_note`; each of
  the four fails on main's builder.
- `cargo test --locked -- --ignored` (six live-CLI tests) passes, since the
  handoff moved.
- In the running app, handing off a conversation whose last tool run failed:
  the destination session's first prompt shows a `### Tool result` section
  with the failing lines.

## Rejected alternatives

- **Carry tool calls too, paired with their results.** The reader's text loses
  the call ids, so parallel calls pair wrongly; a wrong label is worse than none.
- **Compress structured JSON results item by item.** Most results the three
  CLIs record are line-oriented text; a JSON array is still shortened by lines,
  and the snapshot holds the rest.
- **Have a model summarise the tool output.** A second model call per handoff,
  with its latency and cost, for text a rule shortens well; and it would invent.
- **Raise the bundle budget to make room.** The budget is what keeps the first
  prompt small; the give-way order spends it instead.
