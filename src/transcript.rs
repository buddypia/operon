use crate::*;

/// What a restore does with one piece of another CLI's transcript.
///
/// The fourth outcome — the vocabulary has no row for this kind at all — is
/// deliberately *not* in this enum. It is `None` from `transcript_class`, and
/// the reader turns it into a counted, reported, still-restored fragment. That
/// asymmetry is the whole point: "excluded on purpose" and "never seen before"
/// used to be the same `None` inside the readers, so a record type Claude Code
/// invented after this code was written vanished without a trace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TranscriptClass {
    /// Something one of the two participants said, did, or was shown. It is
    /// carried across.
    Conversation,
    /// The model's own reasoning. Carried across as attributed text, never as
    /// native reasoning — see `RESTORED_REASONING_LABEL`.
    Reasoning,
    /// The CLI talking to itself: its harness, its bookkeeping, its injected
    /// context. Excluded, and the exclusion is declared here rather than
    /// implied by a missing match arm.
    Operational,
}

/// Which layer of a provider's transcript a kind names. A provider can reuse a
/// word between layers — Codex has both a `message` payload and `input_text`
/// blocks inside it — so a kind is only unique once the layer is known.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum TranscriptLayer {
    /// The `type` of a whole record. Antigravity has no single field for this,
    /// so its records are keyed `SOURCE/TYPE`.
    Record,
    /// The `type` of the payload inside a record that carries one. Codex wraps
    /// every conversation item in a `response_item` record.
    Payload,
    /// The `role` a message declares. A conversation has two participants; the
    /// other roles a provider uses are the harness addressing the model.
    Role,
    /// The `type` of one content block inside a message, or the name of a field
    /// that carries content of its own.
    Block,
    /// A boolean a record sets on itself to say what kind of record it is.
    /// A `Operational` flag excludes the record it is set on.
    Flag,
    /// A `<tag>` a CLI wraps around text inside a message it sends itself.
    /// `Operational` means the block is stripped; `Conversation` means the text
    /// inside it is the person's and is kept.
    Wrapper,
}

pub(crate) struct TranscriptKind {
    /// `CliProvider::agent()`, so the table and `AGENTS` cannot drift.
    pub(crate) provider: &'static str,
    pub(crate) layer: TranscriptLayer,
    pub(crate) kind: &'static str,
    pub(crate) class: TranscriptClass,
    /// Why it is classified this way. The table is the documentation: a row
    /// whose reason cannot be written is a row nobody understood.
    ///
    /// Nothing at runtime reads it, which is the point — it is addressed to the
    /// next person deciding whether a classification is still right.
    /// `the_transcript_vocabulary_is_well_formed` rejects an empty one.
    #[allow(dead_code)]
    pub(crate) reason: &'static str,
}

use TranscriptClass::{Conversation, Operational, Reasoning};
use TranscriptLayer::{Block, Flag, Payload, Record, Role, Wrapper};

const fn kind(
    provider: &'static str,
    layer: TranscriptLayer,
    kind: &'static str,
    class: TranscriptClass,
    reason: &'static str,
) -> TranscriptKind {
    TranscriptKind {
        provider,
        layer,
        kind,
        class,
        reason,
    }
}

/// Every record, payload, block, flag, and wrapper the three supported CLIs are
/// known to write, and what a restore does with each.
///
/// This is the only place that decides. The readers in `src/history.rs` ask
/// this table and act on the answer; they never carry a judgement of their own,
/// because a judgement written as a match arm is one no test can enumerate.
///
/// **Seeded from real local transcripts, not from documentation** — 5 524
/// Claude Code sessions, 631 Codex sessions, and 445 Antigravity conversations
/// were scanned for every distinct kind they contain. Nothing is listed here
/// that was not actually observed: an invented row is a row that claims
/// coverage the table does not have.
///
/// Sorted by `(provider, layer, kind)` so `transcript_class` is a binary search
/// and `the_transcript_vocabulary_is_well_formed` can assert the order. A row
/// out of place makes its own kind unreachable rather than wrong, which is the
/// same failure the `i18n_tables` ordering exists to prevent.
pub(crate) const TRANSCRIPT_VOCABULARY: &[TranscriptKind] = &[
    // ---- Claude Code: records ------------------------------------------------
    kind("claude", Record, "agent-name", Operational, "which subagent produced the next records; names the harness, not a turn"),
    kind("claude", Record, "ai-title", Operational, "the conversation title Claude Code generated for its own picker"),
    kind("claude", Record, "assistant", Conversation, "a turn the model took"),
    kind("claude", Record, "atis-latch", Operational, "internal latch state; carries no message"),
    kind("claude", Record, "attachment", Operational, "context Claude Code injected into its own prompt — hook output, file contents, skill listings. Restoring it puts a third CLI's internals into the destination conversation"),
    kind("claude", Record, "cost-state", Operational, "accumulated spend for this session"),
    kind("claude", Record, "custom-title", Operational, "the title the person gave the session, not something they said in it"),
    kind("claude", Record, "file-history-delta", Operational, "Claude Code's own undo history for files it edited"),
    kind("claude", Record, "file-history-snapshot", Operational, "as file-history-delta"),
    kind("claude", Record, "last-prompt", Operational, "a duplicate of the most recent user record, kept for the resume picker"),
    kind("claude", Record, "mode", Operational, "which mode the session was in"),
    kind("claude", Record, "permission-mode", Operational, "which permission mode the session was in"),
    kind("claude", Record, "pr-link", Operational, "a pull request Claude Code opened; bookkeeping about the turn, not the turn"),
    kind("claude", Record, "progress", Operational, "hook and subagent progress events, carrying a `data` payload rather than a message"),
    kind("claude", Record, "queue-operation", Operational, "the person queueing or dequeueing a prompt they had not sent yet"),
    kind("claude", Record, "relocated", Operational, "records that the project directory moved"),
    kind("claude", Record, "summary", Conversation, "the summary that stands in for turns a compaction or a resume replaced. It IS the conversation for everything before it, and it carries its text in `summary` rather than in a message"),
    kind("claude", Record, "system", Operational, "Claude Code reporting its own state to the transcript"),
    kind("claude", Record, "user", Conversation, "a turn the person took"),
    kind("claude", Record, "worktree-state", Operational, "which git worktree the session was attached to"),
    // ---- Claude Code: content blocks ----------------------------------------
    kind("claude", Block, "fallback", Operational, "Claude Code recording that it rerouted to another model. A fact about the request, not about the conversation"),
    kind("claude", Block, "image", Conversation, "the person or a tool put an image in the conversation. It cannot cross to another CLI, so a placeholder naming it does — losing the fact that an image was there is worse than losing the pixels"),
    kind("claude", Block, "redacted_thinking", Reasoning, "reasoning the provider returned encrypted; only the fact that it happened can cross"),
    kind("claude", Block, "text", Conversation, "what was actually said"),
    kind("claude", Block, "thinking", Reasoning, "the model's extended thinking. Roughly as numerous as text blocks in real transcripts, so dropping it loses about half of what the assistant produced"),
    kind("claude", Block, "tool_reference", Conversation, "names a tool the turn referred to"),
    kind("claude", Block, "tool_result", Conversation, "the evidence a decision in the conversation was made on"),
    kind("claude", Block, "tool_use", Conversation, "the action the model took, which the next turns respond to"),
    // ---- Claude Code: record flags ------------------------------------------
    kind("claude", Flag, "isAbortedMidStream", Conversation, "the person interrupted the turn. What the model had already said is still on the record and still what the next turns respond to, so the flag marks the turn rather than excluding it"),
    kind("claude", Flag, "isApiErrorMessage", Operational, "the CLI reporting a request failure to itself"),
    kind("claude", Flag, "isCompactSummary", Conversation, "the summary that replaced compacted turns. It IS the conversation for everything before the compaction, so it is kept"),
    kind("claude", Flag, "isMeta", Operational, "something Claude Code told itself"),
    kind("claude", Flag, "isSidechain", Operational, "a subagent's own conversation, which has its own transcript and its own restore"),
    kind("claude", Flag, "isSnapshotUpdate", Operational, "file-history bookkeeping; only ever set on records already excluded at the record layer"),
    kind("claude", Flag, "isVisibleInTranscriptOnly", Conversation, "shown to the person but withheld from the model; still something they read"),
    // ---- Claude Code: wrappers inside user messages --------------------------
    kind("claude", Wrapper, "command-args", Conversation, "the arguments the person typed after a slash command"),
    kind("claude", Wrapper, "command-message", Operational, "the CLI's own echo of the command"),
    kind("claude", Wrapper, "command-name", Conversation, "the slash command the person typed"),
    kind("claude", Wrapper, "local-command-caveat", Operational, "boilerplate Claude Code prepends to locally-run commands"),
    kind("claude", Wrapper, "local-command-stdout", Operational, "the output of a command the person ran in the CLI, not something they said"),
    kind("claude", Wrapper, "system-reminder", Operational, "instructions the harness injected into the person's message"),
    kind("claude", Wrapper, "task-notification", Operational, "a subagent finishing, reported into the person's next message"),
    // ---- Codex: records ------------------------------------------------------
    kind("codex", Record, "compacted", Operational, "marks where the thread was compacted"),
    kind("codex", Record, "event_msg", Operational, "Codex's replay track. Every conversation item already appears as a response_item, so reading both would restore the conversation twice"),
    kind("codex", Record, "inter_agent_communication_metadata", Operational, "routing between collaborating agents; the messages themselves are response_items"),
    kind("codex", Record, "response_item", Conversation, "the record that carries conversation items; its payload decides"),
    kind("codex", Record, "session_meta", Operational, "the session's own identity and configuration"),
    kind("codex", Record, "token_usage_record", Operational, "token counts for a turn and for the thread, keyed by thread/turn/response id. The same thing claude/cost-state is, and seeded from a real rollout: 32 of this machine's own carry it, 2385 records in one thread"),
    kind("codex", Record, "turn_context", Operational, "the model and settings a turn ran under"),
    kind("codex", Record, "world_state", Operational, "Codex's snapshot of the workspace"),
    // ---- Codex: response_item payloads ---------------------------------------
    kind("codex", Payload, "agent_message", Conversation, "a message between collaborating agents; model-written conversation"),
    kind("codex", Payload, "custom_tool_call", Conversation, "an action the model took"),
    kind("codex", Payload, "custom_tool_call_output", Conversation, "the evidence that action produced"),
    kind("codex", Payload, "function_call", Conversation, "an action the model took"),
    kind("codex", Payload, "function_call_output", Conversation, "the evidence that action produced"),
    kind("codex", Payload, "image_generation_call", Conversation, "an image the model produced; carried as a placeholder naming it"),
    kind("codex", Payload, "message", Conversation, "a turn, by either participant"),
    kind("codex", Payload, "reasoning", Reasoning, "the model's reasoning. The most numerous response_item in real Codex sessions — about three times as many as message"),
    kind("codex", Payload, "tool_search_call", Conversation, "the model looking up which tools it has. Carries its query in `arguments` and no `name`"),
    kind("codex", Payload, "tool_search_output", Conversation, "the tools that search returned, in `tools` rather than `output`"),
    kind("codex", Payload, "web_search_call", Conversation, "a search the model ran; its result informs later turns"),
    // ---- Codex: message roles ------------------------------------------------
    kind("codex", Role, "assistant", Conversation, "the model"),
    kind("codex", Role, "developer", Operational, "the harness instructing the model; the destination loads its own"),
    kind("codex", Role, "system", Operational, "as developer"),
    kind("codex", Role, "user", Conversation, "the person"),
    // ---- Codex: content blocks -----------------------------------------------
    kind("codex", Block, "encrypted_content", Operational, "reasoning encrypted under OpenAI's organisation-scoped key. It cannot be decrypted anywhere else, so moving the bytes would restore nothing while looking like it restored something"),
    kind("codex", Block, "input_image", Conversation, "an image the person supplied; carried as a placeholder naming it"),
    kind("codex", Block, "input_text", Conversation, "what the person said"),
    kind("codex", Block, "output_text", Conversation, "what the model said"),
    // ---- Codex: wrappers inside user messages --------------------------------
    kind("codex", Wrapper, "environment_context", Operational, "the machine the session ran on"),
    kind("codex", Wrapper, "hook_prompt", Operational, "output of a hook, sent to the model as if the person said it"),
    kind("codex", Wrapper, "objective", Conversation, "the standing objective the person wrote, inside the scaffolding Codex wraps around it"),
    kind("codex", Wrapper, "recommended_plugins", Operational, "the harness describing itself"),
    kind("codex", Wrapper, "skill", Operational, "a skill Codex loaded into its own prompt"),
    kind("codex", Wrapper, "turn_aborted", Operational, "the CLI recording an interruption"),
    kind("codex", Wrapper, "user_instructions", Operational, "the project's standing instructions, which the destination loads for itself"),
    // ---- Antigravity: records, keyed SOURCE/TYPE ------------------------------
    kind("gemini", Record, "MODEL/GENERIC", Conversation, "tool results — generated images, fetched pages, written files. The same evidence the other two providers' tool_result records carry"),
    kind("gemini", Record, "MODEL/PLANNER_RESPONSE", Conversation, "a turn the model took"),
    kind("gemini", Record, "MODEL/RUN_COMMAND", Operational, "command execution trace"),
    kind("gemini", Record, "SYSTEM/CHECKPOINT", Operational, "a restore point agy wrote for itself"),
    kind("gemini", Record, "SYSTEM/ERROR_MESSAGE", Operational, "the CLI reporting a failure"),
    kind("gemini", Record, "SYSTEM/SYSTEM_MESSAGE", Operational, "the CLI talking to itself"),
    kind("gemini", Record, "SYSTEM_SDK/EPHEMERAL_MESSAGE", Operational, "a transient notice that was never part of the conversation"),
    kind("gemini", Record, "USER_EXPLICIT/USER_INPUT", Conversation, "a turn the person took"),
    // ---- Antigravity: fields carrying their own content ----------------------
    kind("gemini", Block, "thinking", Reasoning, "the model's reasoning, on the record that produced it"),
    kind("gemini", Block, "tool_calls", Conversation, "the actions a planner response took"),
    // ---- Antigravity: wrappers inside a submitted request --------------------
    kind("gemini", Wrapper, "USER_REQUEST", Conversation, "the request itself, inside the metadata agy appends to it"),
    kind("gemini", Wrapper, "USER_SETTINGS_CHANGE", Operational, "the local environment of a session that is not being resumed"),
];

/// What the vocabulary says about one kind, or `None` when it has never been
/// seen before. `None` is not "exclude"; see `TranscriptClass`.
pub(crate) fn transcript_class(
    provider: CliProvider,
    layer: TranscriptLayer,
    kind: &str,
) -> Option<TranscriptClass> {
    let key = (provider.agent(), layer, kind);
    TRANSCRIPT_VOCABULARY
        .binary_search_by(|entry| (entry.provider, entry.layer, entry.kind).cmp(&key))
        .ok()
        .map(|index| TRANSCRIPT_VOCABULARY[index].class)
}

/// Every wrapper tag of one class, for the readers that strip or extract them.
/// The list lives in the table so a tag cannot be added to the stripper and
/// forgotten in the vocabulary, or the reverse.
pub(crate) fn transcript_wrappers(
    provider: CliProvider,
    class: TranscriptClass,
) -> impl Iterator<Item = &'static str> {
    TRANSCRIPT_VOCABULARY
        .iter()
        .filter(move |entry| {
            entry.provider == provider.agent()
                && entry.layer == TranscriptLayer::Wrapper
                && entry.class == class
        })
        .map(|entry| entry.kind)
}

/// How a carried-over reasoning fragment is labelled in the destination.
///
/// It is never written back as native reasoning. Anthropic signs each thinking
/// block and rejects a request whose blocks were modified; OpenAI encrypts its
/// reasoning under an organisation-scoped key. Neither can be reproduced by
/// Operon, and a forged one would fail inside the destination CLI long after
/// the restore reported success. Attributed text is the only form of the
/// model's reasoning that actually survives the crossing.
pub(crate) const RESTORED_REASONING_LABEL: &str = "[Thinking]";

/// How a fragment whose kind the vocabulary does not know is labelled.
///
/// It is restored rather than dropped: an optional field a reader does not
/// understand is safe to ignore for parsing but must be passed through rather
/// than silently stripped. The label names the kind so the reader of the
/// restored conversation — and `TRANSCRIPT_VOCABULARY` — can be told what it is.
///
/// English, like the `[Tool call: …]` and `[Tool result]` markers it sits
/// beside in a restored transcript. These are annotations on carried-over data,
/// not app copy; the counts a person reads afterwards are the Japanese notice
/// `restore_classification_notice` builds.
pub(crate) fn unclassified_label(provider: CliProvider, kind: &str) -> String {
    format!("[Unclassified: {}/{}]", provider.agent(), kind)
}
