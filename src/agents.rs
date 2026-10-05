use crate::*;

mod accounts;
mod settings;

// Which login a session runs under is part of how it is launched, so it lives
// beside the launch and is reachable by symbol name like everything else.
pub(crate) use accounts::*;
pub(crate) use settings::*;

pub(crate) fn agent_shell_command(agent: &str, goal: &str) -> String {
    let launched = suppress_codex_startup_update_prompt(agent);
    if is_native_resume_command(agent) || goal.trim().is_empty() {
        // A native resume reopens the existing interactive conversation. Adding
        // a prompt would execute that request a second time. A blank goal also
        // starts the CLI interactively so the user can enter its first request.
        return launched;
    }
    let quoted = shell_quote(goal);
    match agent.split_whitespace().next().unwrap_or_default() {
        // `-p` is Claude's non-interactive print mode. Sessions need the interactive
        // terminal so their agent can receive follow-up messages from the inspector.
        "agy" => format!("{launched} --prompt-interactive {quoted}"),
        _ => format!("{launched} {quoted}"),
    }
}

/// Codex asks about a new release before it opens its session, and the selected
/// option on that prompt installs the update. A terminal Operon just started
/// has nobody at the keyboard, so the prompt sits there and the session looks
/// like it opened empty — a restored conversation in particular reads as "the
/// restore did nothing". Turn the startup check off for launches Operon
/// makes; the user's own `codex` runs are untouched.
pub(crate) fn suppress_codex_startup_update_prompt(command: &str) -> String {
    let mut parts = command.splitn(2, ' ');
    if parts.next() != Some("codex") {
        return command.to_owned();
    }
    match parts.next() {
        Some(rest) => format!("codex {CODEX_STARTUP_OPTIONS} {rest}"),
        None => format!("codex {CODEX_STARTUP_OPTIONS}"),
    }
}

/// The sentence that introduces the tracking token in a Codex launch prompt.
/// It is stored verbatim in the session goal, so the wording a build before the
/// rename wrote is still on disk and stays recognized.
pub(crate) const NATIVE_TRACKING_TOKEN_PREAMBLES: [&str; 2] = [
    "\n\nOperon tracking token (ignore; do not repeat):",
    "\n\nXirp Copy tracking token (ignore; do not repeat):",
];

pub(crate) fn native_tracking_token(session_id: Uuid) -> String {
    format!("OPERON-NATIVE-SESSION:{session_id}")
}

/// Codex allocates its native ID internally. Put a per-launch opaque marker in
/// its first prompt so local history can be correlated without trusting a
/// timestamp or a duplicate user task.
pub(crate) fn goal_with_native_tracking_token(
    agent: &str,
    goal: String,
    session_id: Uuid,
) -> String {
    if goal.trim().is_empty() {
        return String::new();
    }
    if agent == "codex" {
        format!(
            "{goal}{} {}",
            NATIVE_TRACKING_TOKEN_PREAMBLES[0],
            native_tracking_token(session_id)
        )
    } else {
        goal
    }
}

/// Claude supports allocating a caller-chosen UUID for a new persisted
/// interactive session. Storing it before launch avoids a later transcript
/// scan and lets the very first managed Claude session be resumed by ID.
/// Codex allocates its own ID, which is captured from its local session file
/// immediately after launch instead.
pub(crate) fn command_with_managed_native_session_id(
    agent: &str,
    command: String,
    session_id: Uuid,
) -> (String, Option<String>) {
    if agent == "claude" {
        let session_id = session_id.to_string();
        (
            format!("{command} --session-id {session_id}"),
            Some(session_id),
        )
    } else {
        (command, None)
    }
}

/// Recover a provider-owned ID from a durable managed launch command. This is
/// needed for records written by builds that persisted the command but not its
/// duplicate native-session metadata.
pub(crate) fn native_session_id_from_agent_command(
    provider: CliProvider,
    command: &str,
) -> Option<String> {
    let parts = command.split_whitespace().collect::<Vec<_>>();
    let flagged = |flag: &str| {
        parts
            .windows(2)
            .find_map(|pair| (pair[0] == flag).then_some(pair[1]))
    };
    let candidate = match provider {
        // `codex resume` takes its id as a positional, so it closes the line
        // whatever options sit in front of it.
        CliProvider::Codex => {
            (parts.first() == Some(&"codex") && parts.get(1) == Some(&"resume") && parts.len() > 2)
                .then(|| parts[parts.len() - 1])
        }
        // A launch allocates the id with `--session-id`; a resume names the
        // same id with `--resume`. Both are this terminal's conversation.
        CliProvider::Claude => flagged("--session-id").or_else(|| flagged("--resume")),
        CliProvider::Gemini => flagged("--conversation"),
    }?;
    is_safe_cli_session_id(candidate).then(|| candidate.to_owned())
}

pub(crate) fn is_native_resume_command(command: &str) -> bool {
    // Only reachable through session records written by the removed Gemini
    // `--session-file` importer. Such a record must still run without an extra
    // positional prompt, otherwise reopening it would replay a user turn.
    if command.starts_with("gemini --session-file '") && command.ends_with('\'') {
        return true;
    }
    // A resume may carry the options its launch was started with, so the match
    // is on the resume's own shape rather than on the whole line. Answering
    // yes too readily is the safe direction: all this decides is whether a goal
    // is appended, and appending one to a resume replays a turn.
    let parts = command.split_whitespace().collect::<Vec<_>>();
    let Some(binary) = parts.first() else {
        return false;
    };
    let flagged = |flag: &str| {
        parts
            .windows(2)
            .any(|pair| pair[0] == flag && is_safe_cli_session_id(pair[1]))
    };
    match *binary {
        "codex" => {
            parts.get(1) == Some(&"resume")
                && parts.len() > 2
                && parts
                    .last()
                    .is_some_and(|id| !id.starts_with('-') && is_safe_cli_session_id(id))
        }
        "claude" => flagged("--resume"),
        ANTIGRAVITY_COMMAND => flagged("--conversation"),
        _ => false,
    }
}

/// What the value after a launch option means, which is what lets a resume
/// name the option in the same words the launch screen used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LaunchOptionRole {
    Model,
    Mode,
    Effort,
    /// Carrying this into a resume would contradict the resume. Claude's
    /// `--session-id` allocates a conversation, and asking one command to
    /// allocate and to resume is asking for two different conversations;
    /// Antigravity's `--new-project` says in its own help that it starts a new
    /// project, which is the conversation being thrown away.
    RefusedByResume,
}

/// One option a launch command may carry, its arity, and its meaning.
///
/// The names are written here a second time, which the identifier rule
/// normally forbids: `build_agent_launch_command` above already spells them,
/// and `command_with_managed_native_session_id` spells `--session-id`. The
/// guard is `every_launch_option_the_pickers_offer_survives_a_resume`, which
/// builds a launch through those functions for every model, mode, effort, and
/// flag the pickers offer and asserts each token reaches the resume. A rename
/// in the builder fails that test rather than silently dropping an option.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LaunchOptionShape {
    pub(crate) name: &'static str,
    pub(crate) takes_value: bool,
    pub(crate) role: LaunchOptionRole,
}

pub(crate) fn launch_option_shapes(agent: &str) -> &'static [LaunchOptionShape] {
    match agent {
        "codex" => &[
            LaunchOptionShape {
                name: "--model",
                takes_value: true,
                role: LaunchOptionRole::Model,
            },
            LaunchOptionShape {
                name: "-c",
                takes_value: true,
                role: LaunchOptionRole::Effort,
            },
            LaunchOptionShape {
                name: "--sandbox",
                takes_value: true,
                role: LaunchOptionRole::Mode,
            },
        ],
        "claude" => &[
            LaunchOptionShape {
                name: "--model",
                takes_value: true,
                role: LaunchOptionRole::Model,
            },
            LaunchOptionShape {
                name: "--effort",
                takes_value: true,
                role: LaunchOptionRole::Effort,
            },
            LaunchOptionShape {
                name: "--permission-mode",
                takes_value: true,
                role: LaunchOptionRole::Mode,
            },
            LaunchOptionShape {
                name: "--session-id",
                takes_value: true,
                role: LaunchOptionRole::RefusedByResume,
            },
            // A session reopened twice reads its own first resume as the launch
            // it carries forward. Without this the second command would name
            // two conversations.
            LaunchOptionShape {
                name: "--resume",
                takes_value: true,
                role: LaunchOptionRole::RefusedByResume,
            },
        ],
        "gemini" => &[
            LaunchOptionShape {
                name: "--model",
                takes_value: true,
                role: LaunchOptionRole::Model,
            },
            LaunchOptionShape {
                name: "--effort",
                takes_value: true,
                role: LaunchOptionRole::Effort,
            },
            LaunchOptionShape {
                name: "--mode",
                takes_value: true,
                role: LaunchOptionRole::Mode,
            },
            LaunchOptionShape {
                name: "--new-project",
                takes_value: false,
                role: LaunchOptionRole::RefusedByResume,
            },
            LaunchOptionShape {
                name: "--conversation",
                takes_value: true,
                role: LaunchOptionRole::RefusedByResume,
            },
        ],
        _ => &[],
    }
}

/// The binary a managed launch of this agent runs. The store keeps the agent
/// kind `"gemini"` for a CLI that is spelled `agy`, so the two are not the same
/// string and reading the command means knowing which one to expect.
fn agent_binary(agent: &str) -> &str {
    if agent == "gemini" {
        ANTIGRAVITY_COMMAND
    } else {
        agent
    }
}

/// The tokens of a recorded launch command that a resume may carry with it.
///
/// Empty when the command is not one this build wrote — a different binary, or
/// nothing at all. That is the honest answer for a record from a build that
/// stored something else, and it degrades to the bare resume every build before
/// this one ran.
pub(crate) fn carried_launch_tokens(agent: &str, launch_command: &str) -> Vec<String> {
    let tokens = launch_command.split_whitespace().collect::<Vec<_>>();
    if tokens.first() != Some(&agent_binary(agent)) {
        return Vec::new();
    }
    let flags = agent_flag_options(agent);
    let shapes = launch_option_shapes(agent);
    let mut carried = Vec::new();
    let mut index = 1;
    while index < tokens.len() {
        let token = tokens[index];
        let shape = shapes.iter().find(|shape| shape.name == token);
        // The refusal is read before the catalogue, because an option can be
        // both: Antigravity's `--new-project` is a switch the launch screen
        // offers *and* the one thing a resume must not repeat. Matching it as a
        // flag first would carry it.
        if let Some(shape) = shape.filter(|shape| shape.role == LaunchOptionRole::RefusedByResume) {
            index += if shape.takes_value { 2 } else { 1 };
            continue;
        }
        // A flag from the catalogue travels as its whole argument sequence, so
        // a valued switch such as `--ask-for-approval on-request` cannot be
        // split in half by the walk that carries it.
        if let Some(flag) = flags
            .iter()
            .find(|flag| !flag.args.is_empty() && tokens[index..].starts_with(flag.args))
        {
            carried.extend(flag.args.iter().map(|argument| (*argument).to_owned()));
            index += flag.args.len();
            continue;
        }
        if let Some(shape) = shape {
            let width = if shape.takes_value { 2 } else { 1 };
            if index + width <= tokens.len() {
                carried.extend(tokens[index..index + width].iter().map(|t| (*t).to_owned()));
            }
            index += width;
            continue;
        }
        // A bare word in a stored launch command would reach the CLI as a
        // prompt and replay a turn. Only switches travel.
        if token.starts_with('-') {
            carried.push(token.to_owned());
        }
        index += 1;
    }
    carried
}

/// The command that reopens this terminal's conversation, running the agent the
/// way it was launched.
///
/// The resume itself is never respelled here: it comes from
/// `CliProvider::native_resume_command`, and the options are inserted at the
/// seam after any subcommand and before the resume's own flag or its positional
/// id. Falls back to the bare resume whenever the result would not survive the
/// gates every other launch passes.
pub(crate) fn resume_command_with_launch_options(
    provider: CliProvider,
    agent: &str,
    launch_command: &str,
    native_session_id: &str,
) -> String {
    let bare = provider.native_resume_command(native_session_id);
    let carried = carried_launch_tokens(agent, launch_command);
    if carried.is_empty() {
        return bare;
    }
    let mut parts = bare
        .split_whitespace()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if parts.len() < 2 {
        return bare;
    }
    let seam = parts
        .iter()
        .position(|token| token.starts_with('-'))
        .unwrap_or(parts.len() - 1);
    parts.splice(seam..seam, carried);
    let command = parts.join(" ");
    if is_safe_agent_command(&command) && is_native_resume_command(&command) {
        command
    } else {
        bare
    }
}

/// The launch options a resume will carry, named the way the launch screen
/// named them. This is the second line of the resume verb's hover text, and it
/// is empty when there is nothing to say.
pub(crate) fn carried_launch_option_labels(agent: &str, launch_command: &str) -> Vec<String> {
    let carried = carried_launch_tokens(agent, launch_command);
    let flags = agent_flag_options(agent);
    let shapes = launch_option_shapes(agent);
    let mut labels = Vec::new();
    let mut index = 0;
    while index < carried.len() {
        if let Some(flag) = flags.iter().find(|flag| {
            !flag.args.is_empty()
                && carried.len() >= index + flag.args.len()
                && carried[index..index + flag.args.len()]
                    .iter()
                    .zip(flag.args)
                    .all(|(carried, expected)| carried == expected)
        }) {
            labels.push(tr(flag.label).to_owned());
            index += flag.args.len();
            continue;
        }
        if let Some(shape) = shapes
            .iter()
            .find(|shape| shape.name == carried[index] && shape.takes_value)
        {
            if let Some(value) = carried.get(index + 1) {
                // Codex spells its effort as one `-c key=value` pair, so the
                // value a person recognises is the half after the `=`.
                let value = value.rsplit('=').next().unwrap_or(value.as_str());
                let named = match shape.role {
                    LaunchOptionRole::Model => Some(tf!("モデル {value}", value = value)),
                    LaunchOptionRole::Mode => Some(tf!(
                        "{label} {value}",
                        label = agent_mode_label(agent),
                        value = value
                    )),
                    LaunchOptionRole::Effort => Some(tf!("effort {value}", value = value)),
                    // Unreachable by construction: a refused option never
                    // reaches the carried list. Named rather than panicked, so
                    // that a future role added to the table cannot turn this
                    // walk into a crash on a hover.
                    LaunchOptionRole::RefusedByResume => None,
                };
                labels.extend(named);
            }
            index += 2;
            continue;
        }
        index += 1;
    }
    labels
}

/// Whether a resume would carry an option the launch screen made the person
/// acknowledge in writing. The verb takes its danger treatment from this, so
/// that a run without permission prompts is never resumed silently.
pub(crate) fn carried_launch_is_dangerous(agent: &str, launch_command: &str) -> bool {
    let carried = carried_launch_tokens(agent, launch_command);
    // The same question the launch screen asks, asked of the tokens the resume
    // will actually carry. Reading only this agent's flag table missed the mode:
    // a session launched with `--sandbox danger-full-access` is one the launch
    // screen made a person sign for, the resume does carry it, and the restart
    // chip drew it as ordinary.
    //
    // Not the custom agent, which has no chip: `carried_launch_tokens` returns
    // nothing unless the line starts with this agent's binary, so a custom
    // command carries nothing — and correctly, because `resume_command_with_
    // launch_options` falls back to the bare resume for the same reason. There
    // is nothing to be dangerous about. `cli_provider_for_agent("custom")` is
    // `None` anyway, so no custom session is ever drawn a resume verb.
    arguments_need_acknowledgement(carried.iter().map(String::as_str))
}

pub(crate) fn resolved_native_resume_command(
    command: &str,
    native_session_id: Option<&str>,
    native_session_path: Option<&Path>,
    project: &Path,
) -> UiResult<String> {
    let parts = command.split_whitespace().collect::<Vec<_>>();
    if !matches!(parts.as_slice(), ["agy", "--conversation", _]) {
        return Ok(command.to_owned());
    }
    let Some(native_session_id) = native_session_id else {
        return Err(tr("Antigravity の会話 ID がありません。").into());
    };
    let _ = native_session_path;
    let _ = project;
    Ok(CliProvider::Gemini.native_resume_command(native_session_id))
}

pub(crate) fn native_resume_display(provider: CliProvider, native_session_id: &str) -> String {
    match provider {
        CliProvider::Gemini => CliProvider::Gemini.native_resume_command(native_session_id),
        _ => provider.native_resume_command(native_session_id),
    }
}

/// Never preselect the CLI the conversation already lives in. That target is
/// offered — restoring into the same CLI forks the conversation into a second
/// one of its own — but it is the rarer intent, and a picker that started there
/// would make the ordinary case, moving the work to another tool, the one that
/// needs a change.
/// Preselects a destination the person can actually launch. A fixed preference
/// points at one CLI regardless of what is installed, so the mismatch only
/// surfaces as an error after they press the restore button.
pub(crate) fn default_handoff_target(source: CliProvider, tools: &ToolStatus) -> &'static str {
    let preferred = match source {
        CliProvider::Claude | CliProvider::Gemini => "codex",
        CliProvider::Codex => "claude",
    };
    if tools.agent_available(preferred) {
        return preferred;
    }
    FULL_HISTORY_RESTORE_TARGETS
        .iter()
        .copied()
        .find(|agent| *agent != source.agent() && tools.agent_available(agent))
        .unwrap_or(preferred)
}

/// A restore destination as a picker or a menu names it. Every supported CLI is
/// a destination, including the one the conversation is already in — but that
/// entry does something different from the rest, so it has to read differently:
/// it forks the conversation rather than moving it to another tool.
pub(crate) fn restore_target_label(agent: &str, source: CliProvider) -> String {
    let name = agent_choice_copy(agent).0;
    if agent == source.agent() {
        tf!("{name}（同じ CLI に複製）", name = name)
    } else {
        name.to_owned()
    }
}

/// What a restore is doing, said the way the destination decides. Calling a
/// same-CLI restore "復元" would name an operation that appears to change
/// nothing; what it produces is a second conversation in the same tool.
pub(crate) fn restore_progress_title(progress: &RestoreProgress) -> String {
    if progress.source == progress.destination {
        tf!(
            "{p0} の会話を同じ CLI に複製しています…",
            p0 = progress.source.label()
        )
    } else {
        tf!(
            "{p0} の会話を {p1} に復元しています…",
            p0 = progress.source.label(),
            p1 = progress.destination.label()
        )
    }
}

pub(crate) fn cli_provider_for_agent(agent: &str) -> Option<CliProvider> {
    match agent {
        "codex" => Some(CliProvider::Codex),
        "claude" => Some(CliProvider::Claude),
        "gemini" => Some(CliProvider::Gemini),
        _ => None,
    }
}

/// A managed session created by an older Operon build can itself contain a
/// handoff packet as its launch goal. Passing that packet verbatim again makes
/// every subsequent continuation nest another transcript and eventually hides
/// the real work. Peel those envelopes until the actual task remains.
pub(crate) fn original_task_from_session_goal(goal: &str) -> String {
    let mut task = goal.trim();
    for _ in 0..8 {
        let envelope = [
            (
                "Original user request:\n",
                "\n\nRecent terminal conversation",
            ),
            ("Task to continue:\n", "\n\nLatest terminal context"),
            (
                "Last user request from the source session:\n",
                "\n\nBefore changing files",
            ),
        ]
        .into_iter()
        .filter_map(|(start, end)| task.find(start).map(|offset| (offset, start, end)))
        .min_by_key(|(offset, _, _)| *offset);
        let Some((start, marker, end_marker)) = envelope else {
            break;
        };
        let nested = &task[start + marker.len()..];
        let nested = nested.split(end_marker).next().unwrap_or(nested).trim();
        if nested.is_empty() || nested == task {
            break;
        }
        task = nested;
    }
    for preamble in NATIVE_TRACKING_TOKEN_PREAMBLES {
        if let Some((before, _)) = task.split_once(preamble) {
            task = before;
            break;
        }
    }
    task.trim().to_owned()
}

/// Older stores retain the previous internal label. Present it as a clear
/// continuation action without rewriting the user's durable session history.
pub(crate) fn session_display_name(name: &str) -> String {
    let Some(legacy) = name.strip_prefix("Handoff: ") else {
        return name.to_owned();
    };
    let Some((source, target)) = legacy.split_once(" → ") else {
        return name.to_owned();
    };
    tf!(
        "{target} で続ける（{source} から）",
        source = source,
        target = target
    )
}

pub(crate) fn session_display_goal(goal: &str) -> String {
    original_task_from_session_goal(goal)
}

/// A record written before titles carried the work names the mechanism that
/// created it instead — "Antigravity で会話全履歴を復元（Codex CLI から）". Every
/// restored session reads the same way, and the sentence is longer than the
/// space a list gives it. These records cannot be rewritten in place (the name
/// is the user's own durable data), so read the mechanism out of the name and
/// present it the way a new record is written: the work in the title, the
/// mechanism left out.
pub(crate) fn is_mechanism_session_name(name: &str) -> bool {
    let name = name.trim();
    name.contains("会話全履歴を復元（")
        || name.starts_with("Handoff: ")
        || name.starts_with("Resume: ")
        || name.starts_with("Untitled ")
        || name == "Untitled terminal"
        || name == "復旧したターミナルセッション"
}

/// The provenance a legacy name still carries, for records saved before
/// `Session::origin` existed.
pub(crate) fn legacy_session_origin(session: &Session) -> Option<SessionOrigin> {
    let name = session.name.trim();
    if let Some((_, rest)) = name.split_once(" で会話全履歴を復元（") {
        let source = rest.strip_suffix(" から）")?;
        return provider_from_label(source).map(|from| SessionOrigin::Restored { from });
    }
    if let Some(legacy) = name.strip_prefix("Handoff: ") {
        let (source, _) = legacy.split_once(" → ")?;
        return provider_from_label(source).map(|from| SessionOrigin::Restored { from });
    }
    if name.starts_with("Resume: ") {
        // The record does not name the source: a resume reopens the terminal's
        // own conversation, so the CLI it runs is the CLI it came from.
        return cli_provider_for_agent(&session.agent).map(|from| SessionOrigin::Resumed { from });
    }
    if name == "Untitled terminal" || session.agent == "terminal" {
        return Some(SessionOrigin::PlainTerminal);
    }
    if name == "復旧したターミナルセッション" || session.agent == "existing terminal"
    {
        return Some(SessionOrigin::Adopted);
    }
    None
}

pub(crate) fn provider_from_label(label: &str) -> Option<CliProvider> {
    match label.trim() {
        "Codex CLI" | "Codex" => Some(CliProvider::Codex),
        "Claude Code" | "Claude" => Some(CliProvider::Claude),
        "Antigravity CLI" | "Antigravity" => Some(CliProvider::Gemini),
        _ => None,
    }
}

/// What this session is shown as. Mechanism names give way to the conversation's
/// own subject, which is the only part that tells two of them apart.
pub(crate) fn session_title(session: &Session) -> String {
    if !is_mechanism_session_name(&session.name) {
        return session_display_name(&session.name);
    }
    conversation_topic_name(
        nonempty_transcript_text(&session_display_goal(&session.goal)).as_deref(),
        None,
        || short_mechanism_title(session),
    )
}

/// What the session list calls a session. A session launched without a name
/// or a request is named `<workspace> · <agent>`, which every such session on
/// the same project shares; once a prompt has been typed into it, its first
/// line tells them apart. Anything a person named, and anything launched with a
/// request, keeps `session_title`.
pub(crate) fn session_list_title(session: &Session, first_prompt: Option<&str>) -> String {
    let title = session_title(session);
    let launch_fallback = session.goal.trim().is_empty()
        && session
            .name
            .rsplit_once(" · ")
            .is_some_and(|(_, agent)| agent == agent_choice_copy(&session.agent).0);
    if !launch_fallback {
        return title;
    }
    first_prompt
        .and_then(|prompt| prompt.lines().map(str::trim).find(|line| !line.is_empty()))
        .map(str::to_owned)
        .unwrap_or(title)
}

/// The compact stand-in for a mechanism name whose conversation has no request
/// to quote — still shorter, and specific to this session rather than shared by
/// every restored one.
pub(crate) fn short_mechanism_title(session: &Session) -> String {
    match session.origin.or_else(|| legacy_session_origin(session)) {
        Some(SessionOrigin::Restored { from }) => {
            tf!("{p0} から復元", p0 = agent_short_label(from.agent()))
        }
        Some(SessionOrigin::Resumed { from }) => {
            tf!("{p0} の再開", p0 = agent_short_label(from.agent()))
        }
        Some(SessionOrigin::Adopted) => tr("復旧ターミナル").to_owned(),
        Some(SessionOrigin::PlainTerminal) => tr("ターミナル").to_owned(),
        None => tf!("{p0} のセッション", p0 = agent_short_label(&session.agent)),
    }
}

/// How many characters of a conversation's opening request a session title
/// keeps. Long enough to tell two pieces of work apart in the sidebar, short
/// enough that the name still fits beside its buttons.
pub(crate) const SESSION_TOPIC_NAME_MAX_CHARS: usize = 34;

/// Name a session after what its conversation is about.
///
/// "Antigravity で会話全履歴を復元（Codex CLI から）" describes the mechanism, so
/// every restored session reads the same way in the list — the same failure as
/// "Untitled". The mechanism is not what tells two sessions apart, so the title
/// carries the one thing that does: the work itself.
pub(crate) fn conversation_topic_name(
    message: Option<&str>,
    title: Option<&str>,
    fallback: impl FnOnce() -> String,
) -> String {
    [message, title]
        .into_iter()
        .flatten()
        .find_map(topic_from_text)
        .unwrap_or_else(fallback)
}

/// Reduce a request to the one line worth showing as a title.
pub(crate) fn topic_from_text(text: &str) -> Option<String> {
    let line = text.lines().map(str::trim).find(|line| !line.is_empty())?;
    let line = line
        .trim_start_matches(['#', '-', '*', '>', '・', '•', ' '])
        .trim();
    let line = line.split_whitespace().collect::<Vec<_>>().join(" ");
    if line.is_empty() || looks_like_conversation_id(&line) {
        return None;
    }
    Some(truncate_chars(&line, SESSION_TOPIC_NAME_MAX_CHARS))
}

/// A provider's conversation ID is not a name. Listing one as the title only
/// repeats an identifier that is already shown beside it.
pub(crate) fn looks_like_conversation_id(text: &str) -> bool {
    text.len() >= 12
        && text
            .chars()
            .all(|character| character.is_ascii_hexdigit() || character == '-')
}

/// The title for a terminal that carries a restored or resumed conversation.
pub(crate) fn restored_session_name(source: &CliSession) -> String {
    conversation_topic_name(
        source.last_user_message.as_deref(),
        source.title.as_deref(),
        || tf!("{p0} から引き継いだ会話", p0 = source.provider.label()),
    )
}

/// The folder a session works in, as a person refers to it.
pub(crate) fn workspace_display_name(workspace: &Path) -> String {
    workspace
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| workspace.display().to_string())
}

/// The title for a terminal opened without an agent. Naming it after the folder
/// it opened in beats "Untitled": that is the only thing that distinguishes one
/// plain terminal from the next.
pub(crate) fn plain_terminal_name(workspace: &Path) -> String {
    tf!("{p0} のターミナル", p0 = workspace_display_name(workspace))
}

pub(crate) fn agent_choice_copy(agent: &str) -> (&'static str, &'static str) {
    match agent {
        "codex" => ("Codex CLI", tr("OpenAI のコーディングエージェント")),
        "claude" => ("Claude Code", tr("Anthropic のコーディングエージェント")),
        "gemini" => (
            "Antigravity",
            tr("Google のコーディングエージェント（agy）"),
        ),
        "custom" => (tr("別のコマンド"), tr("任意の対応 CLI を指定します")),
        _ => (tr("不明な AI"), tr("利用できません")),
    }
}

/// The CLI's name as a chip beside a session's status. The stored agent ID
/// ("gemini") is not what the CLI is called, so a card that shows the raw value
/// names a product that does not exist.
pub(crate) fn agent_short_label(agent: &str) -> &str {
    match agent {
        "codex" => "Codex",
        "claude" => "Claude",
        "gemini" => "Antigravity",
        "terminal" => tr("ターミナル"),
        "existing terminal" => tr("復旧ターミナル"),
        "custom" => tr("カスタム CLI"),
        // An older record can name a command that is not one of the known
        // agents. Showing it verbatim beats hiding it behind "unknown".
        other => other,
    }
}

/// The mark for `agent`, paired with the name egui caches the upload under.
/// Rows with no vendor behind them — a plain terminal, a custom command — get
/// `None` and keep showing their label alone.
///
/// Every mark is the vendor's macOS app icon, so one asset per agent covers
/// all three themes and every row shares the same rounded square. Codex is the
/// one vendor that ships a light and a dark plate; the light one is taken,
/// because a dark plate is the mark that sinks into the panel it sits on.
pub(crate) fn agent_icon_asset(agent: &str) -> Option<(&'static str, &'static [u8])> {
    match agent {
        "claude" => Some(("claude-code", AGENT_ICON_CLAUDE)),
        "gemini" => Some(("antigravity", AGENT_ICON_ANTIGRAVITY)),
        "codex" => Some(("codex", AGENT_ICON_CODEX)),
        _ => None,
    }
}

/// The vendor mark for `agent`, decoded and uploaded once. The decode, the
/// upload, and the frame-to-frame cache are `png_texture` in `src/ui/widgets.rs`
/// — shared with Operon's own mark, which needs exactly the same treatment and
/// is not an agent.
pub(crate) fn agent_icon_texture(ctx: &egui::Context, agent: &str) -> Option<egui::TextureHandle> {
    let (name, bytes) = agent_icon_asset(agent)?;
    png_texture(ctx, name, bytes)
}

/// Draws the agent's mark as a `size`-point square. An agent without a mark
/// draws nothing and reserves no space, so its label keeps the layout it had.
pub(crate) fn agent_icon(ui: &mut egui::Ui, agent: &str, size: f32) {
    let Some(texture) = agent_icon_texture(ui.ctx(), agent) else {
        return;
    };
    ui.add(egui::Image::new((texture.id(), egui::vec2(size, size))));
}

/// The same square, but held open even when there is no mark to put in it. A
/// list that mixes marked and unmarked rows would otherwise step its labels in
/// and out by the width of an icon.
pub(crate) fn agent_icon_slot(ui: &mut egui::Ui, agent: Option<&str>, size: f32) {
    match agent.and_then(|agent| agent_icon_texture(ui.ctx(), agent)) {
        Some(texture) => {
            ui.add(egui::Image::new((texture.id(), egui::vec2(size, size))));
        }
        None => {
            ui.allocate_space(egui::vec2(size, size));
        }
    }
}

/// The CLI beside a session's status. A vendor mark says which CLI this is
/// without spelling it out, so the marked agents draw the mark alone and keep
/// the name in the tooltip. A plain terminal or a custom command has no mark,
/// and falls back to its label rather than showing nothing. `size` is the text
/// size; the mark is drawn slightly larger so both read at the same height.
pub(crate) fn agent_chip(ui: &mut egui::Ui, agent: &str, size: f32, palette: &Palette) {
    let label = agent_short_label(agent);
    let response = match agent_icon_texture(ui.ctx(), agent) {
        Some(texture) => ui.add(egui::Image::new((
            texture.id(),
            egui::vec2(size + 4.0, size + 4.0),
        ))),
        None => ui.label(
            RichText::new(label)
                .size(size)
                .color(palette.agent_accent(agent)),
        ),
    };
    response.on_hover_text(label);
}

pub(crate) fn agent_requires_visible_terminal(agent: &str) -> bool {
    // Claude Code asks a safety question before its first use in a folder. A
    // detached tmux session cannot surface that answer, so open the terminal
    // after successful launch instead of making an active session look stuck.
    agent == "claude"
}

pub(crate) fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

pub(crate) fn is_safe_agent_command(command: &str) -> bool {
    let command = command.trim();
    !command.is_empty()
        && command.chars().all(|character| {
            character.is_ascii_alphanumeric()
                || character.is_ascii_whitespace()
                || matches!(character, '-' | '_' | '.' | '/' | '=' | ':' | '@' | '+')
        })
}

/// The one setting each CLI exposes as a single choice, and nothing else. Each
/// list is exactly what that CLI's own flag accepts — Codex's `--sandbox`,
/// Claude Code's `--permission-mode`, Antigravity's `--mode` — so a value here
/// is never a value the CLI would reject.
///
/// Anything a CLI lets you combine with that choice is a flag rather than a
/// mode (see `agent_flag_options`). Folding Codex's approval policy into this
/// list would have made `--sandbox workspace-write` and
/// `--ask-for-approval never` two options a person has to pick between, when
/// the CLI takes both at once and the pair is the setting people actually want.
pub(crate) fn agent_mode_options(agent: &str) -> &'static [&'static str] {
    match agent {
        "codex" => &["read-only", "workspace-write", "danger-full-access"],
        "claude" => &[
            "acceptEdits",
            "plan",
            "auto",
            "manual",
            "dontAsk",
            "bypassPermissions",
        ],
        "gemini" => &["accept-edits", "plan"],
        _ => &[],
    }
}

/// What that choice is called on screen. Three CLIs spell the same idea three
/// ways, and "モード" over Codex's sandbox picker would leave a person guessing
/// which of its two approval-shaped settings they were looking at.
pub(crate) fn agent_mode_label(agent: &str) -> &'static str {
    match agent {
        "codex" => tr("サンドボックス"),
        "claude" => tr("権限モード"),
        "gemini" => tr("実行モード"),
        _ => tr("モード"),
    }
}

pub(crate) fn agent_effort_options(agent: &str) -> &'static [&'static str] {
    match agent {
        "codex" => &["minimal", "low", "medium", "high", "xhigh"],
        "claude" => &["low", "medium", "high", "xhigh", "max"],
        "gemini" => &["low", "medium", "high"],
        _ => &[],
    }
}

/// One switch a person can add to a launch command beside the model, the mode,
/// and the effort. Every entry names the exact arguments it appends, so drawing
/// code never writes a flag and a saved preset holds an ID rather than a
/// command line — a CLI that renames a flag is one edit here.
///
/// `group` and `conflicts_with_mode` are the CLI's own exclusivity, checked
/// before tmux opens a window rather than after. Codex rejects
/// `--approve-for-me` beside `--ask-for-approval`, beside
/// `--dangerously-bypass-approvals-and-sandbox`, and beside `--sandbox`; all
/// four of those live in the `approval` group, and the one that also refuses a
/// sandbox says so. Without this, a bad pair would be a window that opens,
/// prints a usage error, and dies — with a session record already written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AgentFlag {
    pub(crate) id: &'static str,
    pub(crate) args: &'static [&'static str],
    pub(crate) label: &'static str,
    pub(crate) detail: &'static str,
    /// Non-empty means at most one flag carrying this name may be on.
    pub(crate) group: &'static str,
    pub(crate) dangerous: bool,
    /// The CLI refuses this flag beside its mode flag.
    pub(crate) conflicts_with_mode: bool,
}

pub(crate) fn agent_flag(agent: &str, id: &str) -> Option<&'static AgentFlag> {
    agent_flag_options(agent).iter().find(|flag| flag.id == id)
}

/// The selected flags in catalogue order, with duplicates collapsed and every
/// exclusivity the CLI declares already checked. Ordering by the catalogue
/// rather than by click order is what makes the same set of switches produce
/// the same command line every time — a preset that reorders itself between
/// launches is one nobody can compare against what actually ran.
pub(crate) fn resolve_agent_flags(
    agent: &str,
    mode: &str,
    flags: &[String],
) -> std::result::Result<Vec<&'static AgentFlag>, String> {
    let catalog = agent_flag_options(agent);
    let mut seen: Vec<&'static AgentFlag> = Vec::new();
    for id in flags {
        let Some(flag) = catalog.iter().find(|flag| flag.id == id) else {
            return Err(tf!(
                "'{id}' は {agent} でサポートされていない起動オプションです。",
                agent = agent,
                id = id
            ));
        };
        if seen.iter().any(|already| already.id == flag.id) {
            continue;
        }
        if !flag.group.is_empty() {
            if let Some(conflict) = seen.iter().find(|already| already.group == flag.group) {
                return Err(tf!(
                    "「{label}」と「{label_2}」は同時に指定できません。",
                    label = tr(conflict.label),
                    label_2 = tr(flag.label)
                ));
            }
        }
        if flag.conflicts_with_mode && !mode.is_empty() {
            return Err(tf!(
                "「{label}」は{p1}の指定と同時に使えません。",
                label = tr(flag.label),
                p1 = agent_mode_label(agent)
            ));
        }
        seen.push(flag);
    }
    // Walking the catalogue rather than sorting what was collected is what
    // gives the ordering for free: the table is already in the order the
    // command should read.
    Ok(catalog
        .iter()
        .filter(|flag| seen.iter().any(|chosen| chosen.id == flag.id))
        .collect())
}

/// The mode values that run without a guard. Both are values a mode table
/// still offers — Antigravity's `dangerously-skip-permissions` left this list
/// when it became a switch, and a value no picker can produce would be a rule
/// nobody could reach.
pub(crate) fn is_dangerous_agent_mode(mode: &str) -> bool {
    matches!(mode, "danger-full-access" | "bypassPermissions")
}

/// Whether the command line itself asks for an unguarded run.
///
/// The tables above already name the exact arguments that mean it, per CLI, so
/// this reads them rather than looking for the word "dangerously" in a string a
/// person typed. A substring rule would fire on a path that happens to contain
/// the text and would miss the day a CLI renames its switch; matching whole
/// arguments does neither.
///
/// Split on `=` as well as on whitespace, because `--sandbox=danger-full-access`
/// and `--sandbox danger-full-access` are the same instruction to every CLI here
/// — clap and commander both take the joined form — and `is_safe_agent_command`
/// permits `=` on purpose. Matching only the spaced form would have left the
/// joined one an unguarded launch, which is the hole this function exists to
/// close rather than a second spelling of it.
///
/// This exists because the launch form is not the only way a command reaches
/// tmux. The custom agent has no mode and no switches — the line a person typed
/// is the whole of what will run, and until this was here it ran without the
/// acknowledgement every other route to the same behaviour requires.
pub(crate) fn command_needs_acknowledgement(command: &str) -> bool {
    arguments_need_acknowledgement(command.split_whitespace())
}

/// The same question, for a caller that already has the arguments apart.
///
/// `carried_launch_is_dangerous` does: it holds the tokens a resume will carry,
/// and joining them into a line for this function to split again was a `String`
/// per session row per frame, bought for nothing.
pub(crate) fn arguments_need_acknowledgement<'a>(arguments: impl Iterator<Item = &'a str>) -> bool {
    let tokens = launch_argument_tokens(arguments);
    // Whether the command carries this sequence, with each element required to
    // have been written as an argument exactly when the table wrote it as one.
    //
    // Reading `wanted.dashed` rather than taking a flag: an entry like
    // `["--config", "danger=true"]` is one argument and one value, and a rule
    // that demanded a dash of both would make it permanently unmatchable — the
    // trap the previous shape of this had, in a new spelling. It also removes
    // the mode arm's exception, because a mode is written in the table as a bare
    // value and so asks nothing of the token it meets.
    let carries = |sequence: &[LaunchToken<'_>]| {
        !sequence.is_empty()
            && tokens.len() >= sequence.len()
            && tokens.windows(sequence.len()).any(|window| {
                window.iter().zip(sequence).all(|(seen, wanted)| {
                    seen.text == wanted.text && (!wanted.dashed || seen.dashed)
                })
            })
    };
    let carries_arguments =
        |arguments: &[&str]| carries(&launch_argument_tokens(arguments.iter().copied()));

    if AGENTS.iter().any(|agent| {
        agent_flag_options(agent)
            .iter()
            .any(|flag| flag.dangerous && carries_arguments(flag.args))
    }) || DANGEROUS_ARGUMENT_ALIASES
        .iter()
        .any(|alias| carries_arguments(alias))
    {
        return true;
    }
    // Modes are matched as bare values, because that is how they reach a command
    // line: `--sandbox danger-full-access` puts the value in its own token with
    // no dash of its own.
    if AGENTS.iter().any(|agent| {
        agent_mode_options(agent).iter().any(|mode| {
            is_dangerous_agent_mode(mode)
                && (carries(&launch_argument_tokens(std::iter::once(*mode)))
                    // clap takes `-sdanger-full-access` as readily as
                    // `--sandbox danger-full-access`; confirmed against the
                    // binary, which reports only the *next* argument as
                    // unexpected. Without this the value never becomes a token
                    // of its own and the mode arm never sees it.
                    || tokens
                        .iter()
                        .filter_map(LaunchToken::attached_value)
                        .any(|value| value == *mode))
        })
    }) {
        return true;
    }
    // Every value up to the next argument, not the one token after the flag.
    // `--allowed-tools` takes a list, and the spelling Claude Code's own help
    // gives first is space-separated — so `--allowed-tools Edit Bash` handed a
    // shell over while this looked only at `Edit`.
    if tokens.iter().enumerate().any(|(at, token)| {
        token.dashed
            && PRE_APPROVING_ARGUMENTS.contains(&token.text)
            && tokens[at + 1..]
                .iter()
                .take_while(|value| !value.dashed)
                .any(|value| pre_approves_a_shell(value.text))
    }) {
        return true;
    }
    // The net under the tables, and the one place their answer is final: a flag
    // a table names has already been judged above, so reaching it here means the
    // table called it safe. `--allow-dangerously-skip-permissions` only lets a
    // person choose the escape later, and asking about it would be this
    // application disagreeing with its own table about one flag — the shape
    // change 036 was about. Everything else, this application has never heard
    // of, and asking is the answer.
    tokens.iter().any(|token| {
        token.dashed
            && argument_names_a_permission_escape(token.text)
            && !any_table_names(token.text)
    })
}

/// Whether some flag table spells this argument. Read out of the tables rather
/// than listed, so a flag that stops being dangerous, or starts, moves the net
/// with it.
fn any_table_names(argument: &str) -> bool {
    AGENTS.iter().any(|agent| {
        agent_flag_options(agent)
            .iter()
            // Only an entry that calls a flag *safe* may suppress the net. A
            // dangerous entry has already been judged by the arm above, and if
            // that arm missed it — a two-token dangerous flag whose first
            // argument appears alone — suppressing here would hide it twice.
            //
            // No test can watch this: every dangerous entry in the tables today
            // is a single argument, so the arm above and this filter agree on
            // all of them and removing the filter changes nothing observable.
            // Written for the entry that is not here yet, and said plainly so
            // that nobody reads the guards as covering it.
            .filter(|flag| !flag.dangerous)
            .any(|flag| {
                // Arguments only, and without collecting: the net asks about a
                // dashed token, so a table's *value* — `never` in
                // `--ask-for-approval never` — is not an answer to that question. A
                // value that one day reads `dangerously-something` would otherwise
                // silently suppress the net for a real flag of the same name.
                launch_argument_tokens(flag.args.iter().copied())
                    .into_iter()
                    .any(|token| token.dashed && token.text == argument)
            })
    })
}

/// Whether an argument no table has heard of names itself as a way to run the
/// agent without being asked.
///
/// This exists because the tables are kept by hand and were provably behind: the
/// review of change 036 found `codex --dangerously-bypass-hook-trust` by reading
/// the CLI's own help, which is the only way it could have been found. Nothing in
/// this repository can do that — a test that ran `--help` would need a live agent
/// CLI, and a seventh `#[ignore]` is a documented regression here.
///
/// So the tables stay the source of truth for the flags they name, and this is
/// the net under them: deliberately two rules rather than a second table, because
/// a second table would need maintaining and would be behind for the same reason
/// the first one was. All three CLIs mark their escapes the same way, in the flag
/// name, and clap's one hidden alias is the exception the alias table holds.
fn argument_names_a_permission_escape(argument: &str) -> bool {
    argument.contains("dangerously") || argument == "yolo"
}

/// Whether one value of a pre-approving argument hands the agent a tool that
/// runs shell commands before anybody is asked.
///
/// `claude --allowed-tools Bash` pre-approves shell execution for the session,
/// which is the same grant the permission prompt exists to take;
/// `--allowed-tools Read` is not. So the value decides.
///
/// Containment rather than equality because Claude Code's help spells its
/// entries scoped — `Bash(git *)`. Note that only the bare form can reach here
/// from a typed command: `is_safe_agent_command` refuses `,` `(` `)` and `*`, so
/// `Bash,Read` and `Bash(git:*)` are spellings this gate will never be handed.
/// They are covered anyway rather than pruned, because the filter is not this
/// function's to depend on and a value list is what the flag takes.
fn pre_approves_a_shell(value: &str) -> bool {
    // The entry, not a substring of it. `BashOutput` reads an existing shell's
    // output and runs nothing, and a gate that asked about it would be teaching
    // people to tick the box without reading it — the same argument that keeps
    // `/opt/--dangerously-skip-permissions-notes/run.sh` ungated.
    //
    // What this cannot see: a tool that runs commands under a name that does not
    // say so, `mcp__shell__execute_command` among them. Nothing about the name
    // of an MCP tool tells this application what it does, and guessing would be
    // a gate that is confidently wrong rather than honestly narrow.
    value.split([',', '(']).any(|entry| entry.trim() == "Bash")
}

/// The arguments whose value is a pre-approval. Both spellings Claude Code
/// accepts; no picker draws either, and neither is in a flag table because
/// whether it is dangerous depends on what it is given.
pub(crate) const PRE_APPROVING_ARGUMENTS: &[&str] = &["allowed-tools", "allowedTools"];

/// One argument of a command line, normalised, and whether it was written as an
/// argument at all.
///
/// The flag and alias tables spell arguments, so they are matched only against
/// tokens that had a leading dash. Without that, dropping the dashes to catch
/// `agy -dangerously-skip-permissions` also made `aider --preset=yolo` match
/// `--yolo` and demand a confirmation for a launch that needed none — safe in
/// direction, and exactly how a person learns to tick the box without reading
/// it. Modes are the deliberate exception: they are values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LaunchToken<'a> {
    text: &'a str,
    dashed: bool,
    /// Written with exactly one dash. clap lets a short option carry its value
    /// with no space — `codex -sdanger-full-access` is `--sandbox
    /// danger-full-access` — so for these the value may still be inside the
    /// token, one character in.
    short: bool,
}

impl LaunchToken<'_> {
    /// The value a short option carried against itself, if it could have.
    /// `-sdanger-full-access` yields `danger-full-access`. A double dash is not
    /// `short`, so `--sandbox` yields nothing at all; the spelling that yields
    /// the harmless `andbox` is the single-dashed `-sandbox`, which `agy`
    /// accepts. Either way it matches nothing, so asking this of every token is
    /// safe.
    fn attached_value(&self) -> Option<&str> {
        self.short.then(|| self.text.get(1..)).flatten()
    }
}

/// One command line's arguments, in the spelling every CLI here would read them
/// as the same thing.
///
/// Three normalisations, each for a spelling that reached tmux unguarded:
///
///   - **Split on `=`.** `--sandbox=danger-full-access` is what clap and
///     commander both take, and `is_safe_agent_command` permits `=` on purpose.
///     Only the first part of a split keeps the dash it was written with; the
///     value after `=` was never an argument.
///   - **Strip the leading dashes.** Antigravity's `agy` is Go's `flag` package,
///     which accepts `-dangerously-skip-permissions` as readily as the double
///     dash the table spells. Codex and Claude Code read a single dash as a
///     bundle of short flags instead, so treating the two the same there can
///     only ask for a confirmation nobody needed — the safe direction.
///   - **Drop what is left empty**, so a bare `--` separator matches nothing.
///
/// Applied to every side of every comparison, including the mode arm: the two
/// dangerous mode values happen to contain neither a dash nor an equals sign, so
/// passing them through changes nothing today, and leaving them out would have
/// been an exception nothing stated.
fn launch_argument_tokens<'a>(arguments: impl Iterator<Item = &'a str>) -> Vec<LaunchToken<'a>> {
    arguments
        .flat_map(|token| {
            let dashed = token.starts_with('-');
            let short = dashed && !token.starts_with("--");
            token
                .split('=')
                .enumerate()
                .map(move |(part, text)| LaunchToken {
                    text: text.trim_start_matches('-'),
                    dashed: dashed && part == 0,
                    short: short && part == 0,
                })
        })
        .filter(|token| !token.text.is_empty())
        .collect()
}

pub(crate) const DANGEROUS_ARGUMENT_ALIASES: &[&[&str]] = &[&["--yolo"]];

/// Whether this combination is one the app makes a person confirm in writing.
/// The mode picker is no longer the only way to ask for an unguarded run, so
/// the check has to read the switches beside it too — and, for the one agent
/// that has neither, the command.
///
/// Both halves rather than only the command: for the three agents with tables,
/// the command is built *from* the mode and the switches, so reading the tables
/// is the same answer for less work. `custom` has no tables, and reading its
/// command is the only way to see what it asked for.
///
/// This runs in a draw path. The table walk is over four `&'static` slices and
/// the token split is over one line `is_safe_agent_command` has already
/// bounded — the same order of work as the `tr()` calls beside it. The custom
/// clause below is the only part of *this* function that costs anything for the
/// other three agents, but `command_needs_acknowledgement` has a second caller
/// now: `carried_launch_is_dangerous`, once per session row per frame, for every
/// agent.
pub(crate) fn launch_needs_acknowledgement(
    agent: &str,
    mode: &str,
    flags: &[String],
    custom_command: &str,
) -> bool {
    is_dangerous_agent_mode(mode)
        || flags
            .iter()
            .any(|id| agent_flag(agent, id).is_some_and(|flag| flag.dangerous))
        || (agent == "custom" && command_needs_acknowledgement(custom_command))
}

pub(crate) fn is_safe_agent_option(value: &str) -> bool {
    value.is_empty()
        || value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
        })
}

pub(crate) fn build_agent_launch_command(
    agent: &str,
    custom_command: &str,
    model: &str,
    mode: &str,
    effort: &str,
    flags: &[String],
) -> std::result::Result<String, String> {
    if agent == "custom" {
        if !flags.is_empty() {
            return Err(
                tr("カスタムエージェントの引数は起動コマンドにそのまま書いてください。").into(),
            );
        }
        let command = custom_command.trim();
        return if command.is_empty() {
            Err(tr("カスタムエージェントのコマンドを入力してください。").into())
        } else if !is_safe_agent_command(command) {
            Err(tr("カスタムコマンドに指定できるのは実行ファイルと通常のフラグだけです。シェル演算子は使えません。").into())
        } else {
            Ok(command.to_owned())
        };
    }
    if !AGENTS.contains(&agent)
        || !is_safe_agent_option(model)
        || !is_safe_agent_option(mode)
        || !is_safe_agent_option(effort)
    {
        return Err(tr("エージェントの起動オプションが不正です。").into());
    }
    if !mode.is_empty() && !agent_mode_options(agent).contains(&mode) {
        return Err(tf!(
            "'{mode}' は {agent} でサポートされていないモードです。",
            agent = agent,
            mode = mode
        ));
    }
    if !effort.is_empty() && !agent_effort_options(agent).contains(&effort) {
        return Err(tf!(
            "'{effort}' は {agent} でサポートされていない effort です。",
            agent = agent,
            effort = effort
        ));
    }
    let selected_flags = resolve_agent_flags(agent, mode, flags)?;
    let mut parts = vec![if agent == "gemini" {
        ANTIGRAVITY_COMMAND.to_owned()
    } else {
        agent.to_owned()
    }];
    match agent {
        "codex" => {
            if !model.is_empty() {
                parts.extend(["--model".into(), model.to_owned()]);
            }
            if !effort.is_empty() {
                parts.extend(["-c".into(), format!("model_reasoning_effort={effort}")]);
            }
            if !mode.is_empty() {
                parts.extend(["--sandbox".into(), mode.to_owned()]);
            }
        }
        "claude" => {
            if !model.is_empty() {
                parts.extend(["--model".into(), model.to_owned()]);
            }
            if !effort.is_empty() {
                parts.extend(["--effort".into(), effort.to_owned()]);
            }
            if !mode.is_empty() {
                parts.extend(["--permission-mode".into(), mode.to_owned()]);
            }
        }
        "gemini" => {
            if !model.is_empty() {
                parts.extend(["--model".into(), model.to_owned()]);
            }
            if !effort.is_empty() {
                parts.extend(["--effort".into(), effort.to_owned()]);
            }
            if !mode.is_empty() {
                parts.extend(["--mode".into(), mode.to_owned()]);
            }
        }
        _ => {}
    }
    for flag in selected_flags {
        parts.extend(flag.args.iter().map(|argument| (*argument).to_owned()));
    }
    Ok(parts.join(" "))
}

pub(crate) fn is_safe_cli_session_id(session_id: &str) -> bool {
    !session_id.is_empty()
        && session_id.len() <= 64
        && session_id
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '-')
}

pub(crate) const CODEX_FLAGS: &[AgentFlag] = &[
    AgentFlag {
        id: "approval-on-request",
        args: &["--ask-for-approval", "on-request"],
        label: "必要なときだけ承認を求める",
        detail: "AI が判断して確認を出します。",
        group: "approval",
        dangerous: false,
        conflicts_with_mode: false,
    },
    AgentFlag {
        id: "approval-never",
        args: &["--ask-for-approval", "never"],
        label: "承認を求めない",
        detail: "サンドボックスの範囲内で確認なしに進みます。",
        group: "approval",
        dangerous: false,
        conflicts_with_mode: false,
    },
    AgentFlag {
        id: "approve-for-me",
        args: &["--approve-for-me"],
        label: "承認を AI に任せる",
        detail: "サンドボックスは workspace-write に固定されます。",
        group: "approval",
        dangerous: false,
        conflicts_with_mode: true,
    },
    AgentFlag {
        id: "bypass-approvals",
        args: &["--dangerously-bypass-approvals-and-sandbox"],
        label: "承認とサンドボックスを無効にする",
        detail: "承認もサンドボックスもなしでコマンドを実行します。",
        group: "approval",
        dangerous: true,
        conflicts_with_mode: false,
    },
    AgentFlag {
        id: "search",
        args: &["--search"],
        label: "Web 検索を許可する",
        detail: "調査を伴う依頼で有効です。",
        group: "",
        dangerous: false,
        conflicts_with_mode: false,
    },
];
pub(crate) const CLAUDE_FLAGS: &[AgentFlag] = &[
    AgentFlag {
        id: "skip-permissions",
        args: &["--dangerously-skip-permissions"],
        label: "権限確認をすべてスキップする",
        detail: "確認なしにファイルとコマンドを実行します。",
        group: "permission-escape",
        dangerous: true,
        conflicts_with_mode: false,
    },
    AgentFlag {
        id: "allow-skip-permissions",
        args: &["--allow-dangerously-skip-permissions"],
        label: "後から「権限確認をすべてスキップ」を選べるようにする",
        detail: "起動時には適用されません。",
        group: "permission-escape",
        dangerous: false,
        conflicts_with_mode: false,
    },
    AgentFlag {
        id: "ide",
        args: &["--ide"],
        label: "IDE に自動接続する",
        detail: "対象の IDE が 1 つだけ見つかったときに接続します。",
        group: "",
        dangerous: false,
        conflicts_with_mode: false,
    },
    AgentFlag {
        id: "verbose",
        args: &["--verbose"],
        label: "詳細ログを出す",
        detail: "設定ファイルの verbose を上書きします。",
        group: "",
        dangerous: false,
        conflicts_with_mode: false,
    },
];
pub(crate) const ANTIGRAVITY_FLAGS: &[AgentFlag] = &[
    AgentFlag {
        id: "sandbox",
        args: &["--sandbox"],
        label: "サンドボックスで実行する",
        detail: "ターミナル操作を制限します。",
        group: "",
        dangerous: false,
        conflicts_with_mode: false,
    },
    AgentFlag {
        id: "skip-permissions",
        args: &["--dangerously-skip-permissions"],
        label: "権限確認をすべてスキップする",
        detail: "すべてのツール実行を自動承認します。",
        group: "",
        dangerous: true,
        conflicts_with_mode: false,
    },
    AgentFlag {
        id: "new-project",
        args: &["--new-project"],
        label: "新しいプロジェクトとして開始する",
        detail: "既存の会話履歴を引き継ぎません。",
        group: "",
        dangerous: false,
        conflicts_with_mode: false,
    },
];
pub(crate) fn agent_flag_options(agent: &str) -> &'static [AgentFlag] {
    match agent {
        "codex" => CODEX_FLAGS,
        "claude" => CLAUDE_FLAGS,
        // The persisted ID stayed "gemini" while the binary became Antigravity.
        "gemini" => ANTIGRAVITY_FLAGS,
        _ => &[],
    }
}

pub(crate) fn setup_session_name(workspace: &Path) -> String {
    tf!(
        "{p0} のセットアップ",
        p0 = workspace_display_name(workspace)
    )
}

/// Which CLI drafts a commit message: the first of the three that is on this
/// machine, in this order.
///
/// No setting. The choice has a right answer on nearly every machine, and the
/// one piece of information a setting would have carried — which one was used —
/// is drawn beside the button instead.
pub(crate) fn commit_message_agent(tools: &ToolStatus) -> Option<&'static str> {
    [
        ("claude", tools.claude),
        ("codex", tools.codex),
        ("gemini", tools.antigravity),
    ]
    .into_iter()
    .find_map(|(agent, present)| present.then_some(agent))
}

/// One CLI's own non-interactive, read-only form, plus the prompt.
///
/// Read-only is the point: this button drafts a sentence, and a drafting run
/// that could edit a file would be a button that changed the change it was
/// describing. Each set of flags is the mode that CLI documents for exactly
/// this, and they live here rather than at the call site so there is one place
/// to check them against a CLI that moved.
pub(crate) fn commit_message_command(agent: &str, prompt: &str) -> Option<(String, Vec<String>)> {
    let (program, mut arguments) = match agent {
        "claude" => (
            "claude",
            vec![
                "-p".to_owned(),
                "--output-format".to_owned(),
                "text".to_owned(),
                "--permission-mode".to_owned(),
                "plan".to_owned(),
            ],
        ),
        "codex" => (
            "codex",
            vec![
                "exec".to_owned(),
                "--skip-git-repo-check".to_owned(),
                "-s".to_owned(),
                "read-only".to_owned(),
            ],
        ),
        "gemini" => (
            ANTIGRAVITY_COMMAND,
            vec!["--print".to_owned(), "--sandbox".to_owned()],
        ),
        _ => return None,
    };
    arguments.push(prompt.to_owned());
    Some((program.to_owned(), arguments))
}
