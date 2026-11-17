//! `Operon agents list|read|send|wait` (change 309): one agent asks about, and
//! types into, the other agents of its own project.
//!
//! The window already knows every session's state — it merges what the CLIs'
//! hooks report with what is on each screen — so it writes that answer to
//! `AGENTS_SNAPSHOT_FILE` and this command reads it, rather than judging
//! states a second time and disagreeing with the window. The command reaches a
//! terminal directly through tmux; it never asks the window to do anything.
//!
//! Three guardrails keep one agent from typing into the wrong place: only
//! sessions of the caller's own project, only an agent that has finished its
//! turn (the rule broadcast and diff notes already follow), and the screen is
//! looked at again just before typing, because the snapshot may be a second —
//! or, if the window stalled, longer — old. They are guardrails, not a
//! security boundary: the caller names itself through its environment and
//! its directory, and anything running as the same user can already reach
//! tmux. Letting an agent run `agents send` lets it prompt the other agents
//! of the project with their permissions.

use crate::*;

/// What the window publishes. `written_at` is how the command tells a running
/// Operon from one that quit or crashed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct AgentSnapshot {
    pub(crate) written_at: u64,
    pub(crate) projects: Vec<AgentProject>,
    pub(crate) sessions: Vec<AgentEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct AgentProject {
    pub(crate) id: Uuid,
    pub(crate) name: String,
    pub(crate) path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct AgentEntry {
    pub(crate) id: Uuid,
    pub(crate) project_id: Uuid,
    pub(crate) name: String,
    pub(crate) agent: String,
    pub(crate) branch: Option<String>,
    pub(crate) folder: PathBuf,
    pub(crate) tmux_name: String,
    pub(crate) state: String,
}

pub(crate) const AGENT_STATE_IDLE: &str = "idle";
pub(crate) const AGENT_STATE_WAITING: &str = "waiting";
pub(crate) const AGENT_STATE_WORKING: &str = "working";

/// The word a state is published as, or `None` for the two that belong to
/// history rather than to the workspace.
pub(crate) fn agent_state_token(kind: SessionStatusKind) -> Option<&'static str> {
    Some(match kind {
        SessionStatusKind::Stopping => "stopping",
        SessionStatusKind::Starting => "starting",
        SessionStatusKind::Working => AGENT_STATE_WORKING,
        SessionStatusKind::Waiting => AGENT_STATE_WAITING,
        SessionStatusKind::Idle => AGENT_STATE_IDLE,
        SessionStatusKind::Running => "running",
        SessionStatusKind::Blocked => "blocked",
        SessionStatusKind::Ready => "ready",
        SessionStatusKind::Queued => "queued",
        SessionStatusKind::Failed => "failed",
        SessionStatusKind::Lost => "lost",
        SessionStatusKind::Checking => "checking",
        SessionStatusKind::Done | SessionStatusKind::Stopped => return None,
    })
}

pub(crate) fn agents_snapshot_path(data_file: &Path) -> PathBuf {
    data_file
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(AGENTS_SNAPSHOT_FILE)
}

/// The snapshot at `path`, or `None` when it is missing, larger than its
/// ceiling, or not a snapshot.
pub(crate) fn read_agent_snapshot(path: &Path) -> Option<AgentSnapshot> {
    use std::io::Read;
    let file = fs::File::open(path).ok()?;
    let mut bytes = Vec::new();
    file.take(AGENTS_SNAPSHOT_MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .ok()?;
    if bytes.len() as u64 > AGENTS_SNAPSHOT_MAX_BYTES {
        return None;
    }
    serde_json::from_slice(&bytes).ok()
}

/// Everything the command needs from outside itself, so the suite can run it
/// against a scripted world.
pub(crate) trait AgentHost {
    /// The snapshot as it is on disk now.
    fn snapshot(&mut self) -> Option<AgentSnapshot>;
    fn now(&mut self) -> u64;
    /// One wait between looks at the snapshot.
    fn pause(&mut self);
    /// The visible screen only: what the agent is showing right now.
    fn visible_screen(&mut self, tmux_name: &str) -> Result<String>;
    /// The last `lines` lines of screen and scrollback.
    fn tail(&mut self, tmux_name: &str, lines: usize) -> Result<String>;
    /// `text` and Enter, typed into the terminal.
    fn send(&mut self, tmux_name: &str, text: &str) -> Result<()>;
}

/// Who is asking: the managed terminal named in `HOOK_SESSION_ENV`, if any,
/// and the directory the command was run in.
pub(crate) struct AgentCaller {
    pub(crate) tmux_name: Option<String>,
    pub(crate) directory: PathBuf,
}

/// The real world: the snapshot beside the store and the tmux server the
/// caller's `TMUX` names (the default one outside a pane).
pub(crate) struct LiveAgentHost {
    pub(crate) snapshot_path: PathBuf,
}

impl AgentHost for LiveAgentHost {
    fn snapshot(&mut self) -> Option<AgentSnapshot> {
        read_agent_snapshot(&self.snapshot_path)
    }

    fn now(&mut self) -> u64 {
        now()
    }

    fn pause(&mut self) {
        thread::sleep(Duration::from_secs(1));
    }

    fn visible_screen(&mut self, tmux_name: &str) -> Result<String> {
        tmux_capture_screen(tmux_name)
    }

    fn tail(&mut self, tmux_name: &str, lines: usize) -> Result<String> {
        let mut command = tmux_command();
        let start = format!("-{lines}");
        command.args(["capture-pane", "-p", "-J", "-S", &start, "-t", tmux_name]);
        let limited = run_command_with_output_limit(
            &mut command,
            Duration::from_secs(10),
            AGENTS_READ_MAX_BYTES,
            COMMAND_ERROR_MAX_BYTES,
        )?;
        if !limited.output.status.success() {
            return Err(anyhow!(String::from_utf8_lossy(&limited.output.stderr)
                .trim()
                .to_owned()));
        }
        Ok(String::from_utf8_lossy(&limited.output.stdout).to_string())
    }

    fn send(&mut self, tmux_name: &str, text: &str) -> Result<()> {
        tmux_send_input(
            tmux_name,
            &[
                TerminalInput::Text(text.to_owned()),
                TerminalInput::Key("Enter".into()),
            ],
        )
    }
}

/// The entry `src/main.rs` calls when the first argument is
/// `AGENTS_CLI_SUBCOMMAND`. Opens no window and takes no lock.
pub(crate) fn run_agents_cli(arguments: &[String]) -> i32 {
    let caller = AgentCaller {
        tmux_name: std::env::var(HOOK_SESSION_ENV)
            .ok()
            .filter(|name| !name.is_empty()),
        directory: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
    };
    let mut host = LiveAgentHost {
        snapshot_path: agents_snapshot_path(&app_data_file()),
    };
    let outcome = agents_cli(arguments, &caller, &mut host);
    // `write!` rather than `print!`: `agents read … | head` closes the pipe
    // early, and a print into a closed pipe panics.
    use std::io::Write;
    let _ = std::io::stdout().write_all(outcome.stdout.as_bytes());
    let _ = std::io::stderr().write_all(outcome.stderr.as_bytes());
    outcome.code
}

pub(crate) fn agents_usage() -> &'static str {
    tr(
        "使い方: Operon agents <list [--json] | read <ID> [--lines N] | send <ID> <内容> | wait <ID> [--until idle|waiting|either] [--timeout 秒]>\nID は list に出る先頭 8 文字（4 文字以上の前方一致）です。",
    )
}

pub(crate) const AGENTS_EXIT_TERMINAL: i32 = 1;
pub(crate) const AGENTS_EXIT_TARGET: i32 = 2;
pub(crate) const AGENTS_EXIT_NOT_RUNNING: i32 = 3;
pub(crate) const AGENTS_EXIT_TIMEOUT: i32 = 4;
pub(crate) const AGENTS_EXIT_BUSY: i32 = 5;
pub(crate) const AGENTS_EXIT_USAGE: i32 = 64;

fn fail(code: i32, message: impl std::fmt::Display) -> CliOutcome {
    CliOutcome {
        code,
        stdout: String::new(),
        stderr: format!("{message}\n"),
    }
}

fn succeed(stdout: impl Into<String>) -> CliOutcome {
    let stdout: String = stdout.into();
    let stdout = if stdout.ends_with('\n') {
        stdout
    } else {
        format!("{stdout}\n")
    };
    CliOutcome {
        code: 0,
        stdout,
        stderr: String::new(),
    }
}

fn short_id(id: Uuid) -> String {
    id.simple().to_string()[..8].to_owned()
}

/// A snapshot a running Operon wrote, or the reason there is none.
fn fresh_snapshot(host: &mut dyn AgentHost) -> Result<AgentSnapshot, CliOutcome> {
    let now = host.now();
    match host.snapshot() {
        Some(snapshot)
            if now.saturating_sub(snapshot.written_at) <= AGENTS_SNAPSHOT_STALE_SECONDS =>
        {
            Ok(snapshot)
        }
        _ => Err(fail(
            AGENTS_EXIT_NOT_RUNNING,
            tr("Operon が起動していないため、エージェントの状態を取得できません。"),
        )),
    }
}

/// The caller's project: its own session's, else the nearest registered
/// folder at or above where it stands.
fn caller_project(caller: &AgentCaller, snapshot: &AgentSnapshot) -> Option<Uuid> {
    if let Some(entry) = caller.tmux_name.as_ref().and_then(|name| {
        snapshot
            .sessions
            .iter()
            .find(|entry| &entry.tmux_name == name)
    }) {
        return Some(entry.project_id);
    }
    let projects: Vec<Project> = snapshot
        .projects
        .iter()
        .map(|project| Project {
            id: project.id,
            path: project.path.clone(),
            name: project.name.clone(),
            added_at: 0,
        })
        .collect();
    project_for_directory(&caller.directory, &projects).map(|project| project.id)
}

/// The one session of `project` that `prefix` names.
fn find_target<'a>(
    snapshot: &'a AgentSnapshot,
    project: Uuid,
    prefix: &str,
) -> Result<&'a AgentEntry, CliOutcome> {
    let wanted = prefix
        .chars()
        .filter(|character| *character != '-')
        .collect::<String>()
        .to_ascii_lowercase();
    if wanted.chars().count() < AGENTS_ID_PREFIX_MIN {
        return Err(fail(
            AGENTS_EXIT_TARGET,
            tr("セッション ID は 4 文字以上で指定してください。"),
        ));
    }
    let matches: Vec<&AgentEntry> = snapshot
        .sessions
        .iter()
        .filter(|entry| entry.project_id == project)
        .filter(|entry| entry.id.simple().to_string().starts_with(&wanted))
        .collect();
    match matches.as_slice() {
        [entry] if is_safe_tmux_name(&entry.tmux_name) => Ok(entry),
        [_] => Err(fail(
            AGENTS_EXIT_TARGET,
            tf!("{id} のターミナル名が不正です。", id = prefix),
        )),
        [] => Err(fail(
            AGENTS_EXIT_TARGET,
            tf!("{id} に一致するセッションがありません。", id = prefix),
        )),
        many => Err(fail(
            AGENTS_EXIT_TARGET,
            tf!(
                "{id} は複数のセッションに一致します: {list}",
                id = prefix,
                list = many
                    .iter()
                    .map(|entry| short_id(entry.id))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        )),
    }
}

pub(crate) fn agents_cli(
    arguments: &[String],
    caller: &AgentCaller,
    host: &mut dyn AgentHost,
) -> CliOutcome {
    let Some((verb, rest)) = arguments.split_first() else {
        return succeed(agents_usage());
    };
    match verb.as_str() {
        "help" | "--help" | "-h" => return succeed(agents_usage()),
        "list" | "read" | "send" | "wait" => {}
        _ => return fail(AGENTS_EXIT_USAGE, agents_usage()),
    }
    let snapshot = match fresh_snapshot(host) {
        Ok(snapshot) => snapshot,
        Err(outcome) => return outcome,
    };
    let Some(project) = caller_project(caller, &snapshot) else {
        return fail(
            AGENTS_EXIT_TARGET,
            tf!(
                "{path} は Operon に登録されたプロジェクトではありません。",
                path = caller.directory.display()
            ),
        );
    };
    match verb.as_str() {
        "list" => agents_list(rest, caller, &snapshot, project),
        "read" => agents_read(rest, &snapshot, project, host),
        "send" => agents_send(rest, caller, &snapshot, project, host),
        _ => agents_wait(rest, &snapshot, project, host),
    }
}

fn agents_list(
    rest: &[String],
    caller: &AgentCaller,
    snapshot: &AgentSnapshot,
    project: Uuid,
) -> CliOutcome {
    let json = match rest {
        [] => false,
        [flag] if flag == "--json" => true,
        _ => return fail(AGENTS_EXIT_USAGE, agents_usage()),
    };
    let is_self = |entry: &AgentEntry| caller.tmux_name.as_deref() == Some(&entry.tmux_name);
    let sessions: Vec<&AgentEntry> = snapshot
        .sessions
        .iter()
        .filter(|entry| entry.project_id == project)
        .collect();
    if json {
        let rows: Vec<serde_json::Value> = sessions
            .iter()
            .map(|entry| {
                serde_json::json!({
                    "id": short_id(entry.id),
                    "name": entry.name,
                    "agent": entry.agent,
                    "state": entry.state,
                    "branch": entry.branch,
                    "folder": entry.folder,
                    "self": is_self(entry),
                })
            })
            .collect();
        let name = snapshot
            .projects
            .iter()
            .find(|candidate| candidate.id == project)
            .map(|candidate| candidate.name.as_str())
            .unwrap_or_default();
        return succeed(serde_json::json!({ "project": name, "sessions": rows }).to_string());
    }
    let mut stdout: String = sessions
        .iter()
        .map(|entry| agents_list_row(entry, is_self(entry)))
        .collect();
    if sessions.iter().all(|entry| is_self(entry)) {
        stdout.push_str(tr("このプロジェクトに他のセッションはありません。"));
    }
    succeed(stdout)
}

/// `text` with every control character dropped: a name or a screen is
/// another program's text, and a stray escape or carriage return in it would
/// redraw the caller's terminal or forge a row.
fn without_controls(text: &str) -> String {
    text.chars()
        .filter(|character| !character.is_control())
        .collect()
}

fn agents_list_row(entry: &AgentEntry, own: bool) -> String {
    let mark = if own { tr(" (自分)") } else { "" };
    format!(
        "{}  {:<8}  {:<8}  {}  {}{}\n",
        short_id(entry.id),
        entry.state,
        entry.agent,
        without_controls(entry.branch.as_deref().unwrap_or("-")),
        without_controls(&entry.name),
        mark
    )
}

fn terminal_failure(error: &anyhow::Error) -> CliOutcome {
    fail(
        AGENTS_EXIT_TERMINAL,
        tf!("ターミナルを操作できませんでした: {error}", error = error),
    )
}

fn agents_read(
    rest: &[String],
    snapshot: &AgentSnapshot,
    project: Uuid,
    host: &mut dyn AgentHost,
) -> CliOutcome {
    let (target, lines) = match rest {
        [target] => (target, AGENTS_READ_LINES_DEFAULT),
        [target, flag, count] if flag == "--lines" => match count.parse::<usize>() {
            Ok(count) if (1..=AGENTS_READ_LINES_MAX).contains(&count) => (target, count),
            _ => {
                return fail(
                    AGENTS_EXIT_USAGE,
                    tf!(
                        "--lines は 1〜{max} で指定してください。",
                        max = AGENTS_READ_LINES_MAX
                    ),
                );
            }
        },
        _ => return fail(AGENTS_EXIT_USAGE, agents_usage()),
    };
    let entry = match find_target(snapshot, project, target) {
        Ok(entry) => entry,
        Err(outcome) => return outcome,
    };
    match host.tail(&entry.tmux_name, lines) {
        Ok(text) => {
            let text: String = strip_terminal_escapes(&text)
                .chars()
                .filter(|character| !character.is_control() || matches!(character, '\n' | '\t'))
                .collect();
            let kept: Vec<&str> = text.trim_end_matches('\n').lines().collect();
            let start = kept.len().saturating_sub(lines);
            succeed(kept[start..].join("\n"))
        }
        Err(error) => terminal_failure(&error),
    }
}

fn agents_send(
    rest: &[String],
    caller: &AgentCaller,
    snapshot: &AgentSnapshot,
    project: Uuid,
    host: &mut dyn AgentHost,
) -> CliOutcome {
    let Some((target, words)) = rest.split_first() else {
        return fail(AGENTS_EXIT_USAGE, agents_usage());
    };
    let joined = words.join(" ");
    let text = joined.trim();
    if text.is_empty() {
        return fail(AGENTS_EXIT_USAGE, tr("送信する内容が空です。"));
    }
    if text.chars().count() > AGENTS_SEND_MAX_CHARS {
        return fail(
            AGENTS_EXIT_USAGE,
            tf!(
                "送信する内容が長すぎます（最大 {max} 文字）。",
                max = AGENTS_SEND_MAX_CHARS
            ),
        );
    }
    if text.chars().any(char::is_control) {
        return fail(
            AGENTS_EXIT_USAGE,
            tr("改行や制御文字を含む内容は送信できません。1 行にまとめてください。"),
        );
    }
    let entry = match find_target(snapshot, project, target) {
        Ok(entry) => entry,
        Err(outcome) => return outcome,
    };
    if caller.tmux_name.as_deref() == Some(entry.tmux_name.as_str()) {
        return fail(AGENTS_EXIT_TARGET, tr("自分自身には送信できません。"));
    }
    let busy = |state: &str| {
        fail(
            AGENTS_EXIT_BUSY,
            tf!(
                "{name} は作業中のため送信しません（状態: {state}）。",
                name = entry.name,
                state = state
            ),
        )
    };
    if entry.state != AGENT_STATE_IDLE {
        return busy(&entry.state);
    }
    // The snapshot can be a second old, or older if the window stalled: the
    // screen right now has the last word on whether a turn has begun. It can
    // only withhold a send, never grant one the snapshot did not.
    match host.visible_screen(&entry.tmux_name) {
        Ok(screen) => match detect_agent_activity(&screen) {
            AgentActivity::Working => return busy(AGENT_STATE_WORKING),
            AgentActivity::AwaitingInput => return busy(AGENT_STATE_WAITING),
            AgentActivity::Idle => {}
        },
        Err(error) => return terminal_failure(&error),
    }
    match host.send(&entry.tmux_name, text) {
        Ok(()) => succeed(tf!("{name} に送信しました。", name = entry.name)),
        Err(error) => terminal_failure(&error),
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum WaitFor {
    Idle,
    Waiting,
    Either,
}

impl WaitFor {
    fn reached(self, state: &str) -> bool {
        match self {
            WaitFor::Idle => state == AGENT_STATE_IDLE,
            WaitFor::Waiting => state == AGENT_STATE_WAITING,
            WaitFor::Either => state == AGENT_STATE_IDLE || state == AGENT_STATE_WAITING,
        }
    }
}

fn agents_wait(
    rest: &[String],
    snapshot: &AgentSnapshot,
    project: Uuid,
    host: &mut dyn AgentHost,
) -> CliOutcome {
    let Some((target, mut options)) = rest.split_first() else {
        return fail(AGENTS_EXIT_USAGE, agents_usage());
    };
    let mut until = WaitFor::Either;
    let mut timeout = AGENTS_WAIT_SECONDS_DEFAULT;
    while let [flag, value, tail @ ..] = options {
        match flag.as_str() {
            "--until" => {
                until = match value.as_str() {
                    "idle" => WaitFor::Idle,
                    "waiting" => WaitFor::Waiting,
                    "either" => WaitFor::Either,
                    _ => return fail(AGENTS_EXIT_USAGE, agents_usage()),
                };
            }
            "--timeout" => match value.parse::<u64>() {
                Ok(seconds) if (1..=AGENTS_WAIT_SECONDS_MAX).contains(&seconds) => {
                    timeout = seconds;
                }
                _ => {
                    return fail(
                        AGENTS_EXIT_USAGE,
                        tf!(
                            "--timeout は 1〜{max} 秒で指定してください。",
                            max = AGENTS_WAIT_SECONDS_MAX
                        ),
                    );
                }
            },
            _ => return fail(AGENTS_EXIT_USAGE, agents_usage()),
        }
        options = tail;
    }
    if !options.is_empty() {
        return fail(AGENTS_EXIT_USAGE, agents_usage());
    }
    let mut current = match find_target(snapshot, project, target) {
        Ok(entry) => entry.clone(),
        Err(outcome) => return outcome,
    };
    let started = host.now();
    let deadline = started.saturating_add(timeout);
    // Right after a `send`, the snapshot still says `idle` until the turn it
    // started is reported. So a state that was already the one asked for
    // counts only once it has been left, or once it has held for
    // `AGENTS_WAIT_SETTLE_SECONDS` — long enough for a turn to show.
    let mut left = !until.reached(&current.state);
    loop {
        if !until.reached(&current.state) {
            left = true;
        } else if left || host.now().saturating_sub(started) >= AGENTS_WAIT_SETTLE_SECONDS {
            return succeed(tf!(
                "{name} は {state} になりました。",
                name = current.name,
                state = current.state
            ));
        }
        if matches!(current.state.as_str(), "failed" | "lost") {
            return fail(
                AGENTS_EXIT_TARGET,
                tf!("{name} は終了しました。", name = current.name),
            );
        }
        if host.now() >= deadline {
            if until.reached(&current.state) {
                // It has held for as long as the caller would wait.
                return succeed(tf!(
                    "{name} は {state} になりました。",
                    name = current.name,
                    state = current.state
                ));
            }
            return fail(
                AGENTS_EXIT_TIMEOUT,
                tf!(
                    "{secs} 秒待ちましたが {name} は {state} のままです。",
                    secs = timeout,
                    name = current.name,
                    state = current.state
                ),
            );
        }
        host.pause();
        let snapshot = match fresh_snapshot(host) {
            Ok(snapshot) => snapshot,
            Err(outcome) => return outcome,
        };
        let id = current.id;
        match snapshot.sessions.into_iter().find(|next| next.id == id) {
            Some(next) => current = next,
            None => {
                return fail(
                    AGENTS_EXIT_TARGET,
                    tf!("{name} は終了しました。", name = current.name),
                );
            }
        }
    }
}
