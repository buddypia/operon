---
paths:
  - "src/history.rs"
  - "src/transcript.rs"
  - "src/cli.rs"
---

# Before reading another CLI's transcript

Same rule as colour and icons, on a format Operon does not own: the readers in
`src/history.rs` hold no judgement. Every record, payload, role, block, flag, and
wrapper has a row in `TRANSCRIPT_VOCABULARY` (`src/transcript.rs`) giving its
class and the reason.

"Excluded on purpose" and "never seen before" must stay different answers. While
both were `None`, every `thinking` block was dropped for want of a `text` field
and a real conversation restored at 40 %, suite green — the fixtures only held
record types somebody had already thought of. So an unrowed kind is carried
labelled and reported, and reasoning crosses as attributed text, never as native
reasoning (a signed or org-encrypted block cannot survive the crossing).

Seed a row from a real transcript, never from documentation:
`bash scripts/check-transcript-vocabulary.sh` names what the local stores hold
that the table has not met. Run it after updating an agent CLI.

# Which conversation a terminal is in

A stored `native_session_id` is the ID the terminal *launched* with, and `/clear`
moves it to a new conversation under a new ID. Do not infer the current one from
the transcripts — both candidates begin after the terminal and the abandoned one
begins closer, so every ranking picks it. Claude Code publishes the answer under
`CLAUDE_SESSION_REGISTRY_DIRECTORY`; `registered_claude_conversation_in` reads
it, and every use of a stored ID goes through
`corrected_managed_claude_conversation` first. Two terminals it cannot tell
apart is a refusal. Lesson 008.

---

Both halves of this rule are about a store another product writes, which is why
they load together: the three files above are the only ones that read one. The
enforcement is `restore_counts_every_source_record_into_exactly_one_class` and
the guards listed under entries 007 and 008 of `docs/sdlc/lessons.md`.
