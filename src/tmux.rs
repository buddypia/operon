use crate::prelude::*;
use crate::*;

pub(crate) mod hooks;
pub(crate) use hooks::*;

/// Every tmux this crate runs starts here, so this is the one place the test
/// build can be kept off the server a person's sessions live on (sdlc 098):
/// `-L` names a socket of the suite's own, and wins over `$TMUX`.
pub(crate) fn tmux_command() -> Command {
    let mut command = Command::new("tmux");
    #[cfg(test)]
    command.args(["-L", "operon_suite"]);
    command.arg("-u");
    if std::env::var_os("LANG").is_none() {
        command.env("LANG", "ja_JP.UTF-8");
    }
    if std::env::var_os("LC_ALL").is_none() {
        command.env("LC_ALL", "ja_JP.UTF-8");
    }
    command
}

#[cfg(test)]
pub(crate) fn tmux_state(name: &str) -> TmuxState {
    tmux_observation(name).state
}

pub(crate) fn tmux_observation(name: &str) -> TmuxObservation {
    let mut command = tmux_command();
    command.args([
        "list-panes",
        "-t",
        name,
        "-F",
        "#{pane_dead}\t#{pane_dead_status}",
    ]);
    let output = run_command_with_timeout(&mut command, Duration::from_secs(5));
    match output {
        Ok(output) if output.status.success() => {
            parse_tmux_observation(&String::from_utf8_lossy(&output.stdout))
        }
        Ok(output) => TmuxObservation {
            state: tmux_error_state(&String::from_utf8_lossy(&output.stderr)),
            exit_status: None,
        },
        Err(_) => TmuxObservation {
            state: TmuxState::Unknown,
            exit_status: None,
        },
    }
}

pub(crate) fn parse_tmux_observation(output: &str) -> TmuxObservation {
    let panes = output
        .lines()
        .filter_map(|line| {
            let (dead, status) = line.trim_end_matches('\r').split_once('\t')?;
            Some((dead == "1", status.parse::<i32>().ok()))
        })
        .collect::<Vec<_>>();
    if panes.is_empty() {
        return TmuxObservation {
            state: TmuxState::Unknown,
            exit_status: None,
        };
    }
    if panes.iter().any(|(dead, _)| !dead) {
        return TmuxObservation {
            state: TmuxState::Alive,
            exit_status: None,
        };
    }
    let exit_status = panes
        .iter()
        .filter_map(|(_, status)| *status)
        .find(|status| *status != 0)
        .or_else(|| {
            panes
                .iter()
                .all(|(_, status)| *status == Some(0))
                .then_some(0)
        });
    TmuxObservation {
        state: TmuxState::Dead,
        exit_status,
    }
}

/// What one tmux observation is allowed to say about a session's state.
///
/// Only an exit status the app actually read is grounds for a verdict: `0` is
/// `Exited` and anything else is `Failed`. A pane that is dead without a status,
/// or a session that is not there at all, ends as `Lost` — the run is over and
/// the outcome was never observed. Calling either of those a failure puts a red
/// mark on work that most likely finished, which is what happens to every
/// session on the list after a reboot takes the tmux server with it.
pub(crate) fn tmux_error_state(stderr: &str) -> TmuxState {
    let error = stderr.to_lowercase();
    if error.contains("permission denied") || error.contains("operation not permitted") {
        return TmuxState::Unknown;
    }
    if [
        "can't find session",
        "can't find window",
        "no server running",
        "no such session",
        "error connecting to",
        "failed to connect to",
        "connection refused",
    ]
    .iter()
    .any(|needle| error.contains(needle))
    {
        TmuxState::Gone
    } else {
        TmuxState::Unknown
    }
}

/// A session that ended without producing what it was asked for can be run
/// again from its own record. That covers the failures and it covers the runs
/// whose terminal disappeared — the second is the one a person most wants back,
/// because nothing about it says the work was finished.
pub(crate) fn can_retry_session(status: &SessionStatus) -> bool {
    matches!(status, SessionStatus::Failed | SessionStatus::Lost)
}

pub(crate) fn retry_tmux_action(observation: TmuxObservation) -> RetryTmuxAction {
    match observation.state {
        TmuxState::Alive => RetryTmuxAction::RefuseAlive,
        TmuxState::Dead => RetryTmuxAction::RemoveDead,
        TmuxState::Gone => RetryTmuxAction::Ready,
        TmuxState::Unknown => RetryTmuxAction::RefuseUnknown,
    }
}

pub(crate) fn tmux_cleanup_failure_result(
    error: String,
    observed_after: TmuxState,
) -> UiResult<()> {
    if observed_after == TmuxState::Gone {
        Ok(())
    } else {
        Err(error)
    }
}

pub(crate) fn close_tmux_session_confirming_gone(name: &str) -> UiResult<()> {
    let mut command = tmux_command();
    command.args(["kill-session", "-t", name]);
    let cleanup = run_command_with_timeout(&mut command, Duration::from_secs(5));
    let error = match cleanup {
        Ok(output)
            if output.status.success()
                || tmux_error_state(&String::from_utf8_lossy(&output.stderr))
                    == TmuxState::Gone =>
        {
            return Ok(());
        }
        Ok(output) => tf!(
            "ターミナルを閉じられませんでした: {p0}",
            p0 = String::from_utf8_lossy(&output.stderr).trim()
        ),
        Err(error) => tf!("tmux を実行できませんでした: {error}", error = error),
    };
    // The terminal may disappear between the initial observation and the
    // kill command, or the command may time out after completing. Re-observe
    // the requested end state before showing a false failure.
    tmux_cleanup_failure_result(error, tmux_observation(name).state)
}

pub(crate) fn prepare_tmux_for_retry(name: &str) -> UiResult<()> {
    match retry_tmux_action(tmux_observation(name)) {
        RetryTmuxAction::RefuseAlive => Err(
            tr("そのターミナルはまだ実行中です。「古いターミナルを停止」を実行してから、もう一度「再試行」を選んでください。")
                .into(),
        ),
        RetryTmuxAction::RemoveDead => {
            let mut command = tmux_command();
            command.args(["kill-session", "-t", name]);
            let cleanup = run_command_with_timeout(&mut command, Duration::from_secs(5));
            let cleanup_error = match cleanup {
                Ok(output)
                    if output.status.success()
                        || tmux_error_state(&String::from_utf8_lossy(&output.stderr))
                            == TmuxState::Gone =>
                {
                    return Ok(());
                }
                Ok(output) => tf!("古いターミナルを停止できませんでした: {p0}", p0 = String::from_utf8_lossy(&output.stderr).trim()),
                Err(error) => tf!("古いターミナルを停止できませんでした: {error}", error = error),
            };
            // A timeout or non-zero result can arrive after tmux already
            // removed the session. Re-observe before refusing the retry so
            // externally completed cleanup is not reported as a failure.
            if retry_tmux_action(tmux_observation(name)) == RetryTmuxAction::Ready {
                Ok(())
            } else {
                Err(cleanup_error)
            }
        }
        RetryTmuxAction::Ready => Ok(()),
        RetryTmuxAction::RefuseUnknown => {
            Err(tr("古いターミナルがまだ開いているか判定できませんでした。").into())
        }
    }
}

pub(crate) fn launch_stored_session(
    session: &Session,
    project_path: &Path,
    retrying: bool,
    auto_approve_workspace_prompts: bool,
    hooks: Option<&HookEnvironment>,
    // `account` is which login this session runs under, already resolved to
    // `KEY=VALUE` pairs by the caller. Empty is the machine's own, which is the
    // launch every build before change 031 made. It is resolved outside this
    // module so the tmux layer stays a layer that starts sessions rather than
    // one that knows what an account is.
    account: &[(&'static str, String)],
) -> UiResult<Option<String>> {
    let worktree_path = session
        .worktree_path
        .clone()
        .unwrap_or_else(|| project_path.to_path_buf());
    if !worktree_path.is_dir() {
        return Err(tr("セッションの作業ディレクトリが存在しません。").into());
    }
    if !is_allowed_session_path(project_path, &worktree_path) {
        return Err(
            tr("作業ディレクトリが、このプロジェクトのメインフォルダでも Git worktree でもありません。").into(),
        );
    }
    if !tool_available("tmux") {
        return Err(tr(
            "tmux が必要です。次のコマンドでインストールしてください: brew install tmux",
        )
        .into());
    }
    if retrying {
        prepare_tmux_for_retry(&session.tmux_name)?;
    }
    if session.agent == "terminal" && session.agent_command.is_empty() {
        start_empty_tmux_session(
            &session.tmux_name,
            &worktree_path,
            &session_variables(hooks, account),
        )
        .map_err(|error| error.to_string())?;
    } else {
        let command = if session.agent_command.is_empty() {
            &session.agent
        } else {
            &session.agent_command
        };
        let command = resolved_native_resume_command(
            command,
            session.native_session_id.as_deref(),
            session.native_session_path.as_deref(),
            project_path,
        )?;
        let executable = command.split_whitespace().next().unwrap_or_default();
        if !tool_available(executable) {
            return Err(tf!(
                "'{executable}' が PATH 上に見つかりません。",
                executable = executable
            ));
        }
        start_tmux_agent_session(
            &session.tmux_name,
            &worktree_path,
            &agent_shell_command(&command, &session.goal),
            &session_variables(hooks, account),
        )
        .map_err(|error| error.to_string())?;
        if auto_approve_workspace_prompts {
            auto_approve_workspace_trust_prompt(&session.tmux_name)
                .map_err(|error| error.to_string())?;
        }
    }
    Ok(git_branch(&worktree_path).ok())
}

/// The `-e KEY=VALUE` arguments a managed session is created with, or nothing
/// when the status hooks are switched off. Set on the session rather than on
/// the pane, because `respawn-pane` builds the new process's environment from
/// the session's — which is how a retried launch keeps its identity.
pub(crate) fn hook_variables(hooks: Option<&HookEnvironment>) -> Vec<(&'static str, String)> {
    hooks.map(HookEnvironment::variables).unwrap_or_default()
}

/// Everything a managed session is created with: the hook identity, and the
/// login it runs under. One function so that a launch path cannot pick up one
/// and forget the other.
pub(crate) fn session_variables(
    hooks: Option<&HookEnvironment>,
    account: &[(&'static str, String)],
) -> Vec<(&'static str, String)> {
    let mut variables = hook_variables(hooks);
    variables.extend(account.iter().cloned());
    variables
}

pub(crate) fn start_tmux_agent_session(
    name: &str,
    cwd: &Path,
    command: &str,
    environment: &[(&'static str, String)],
) -> Result<()> {
    // tmux destroys a window whose command exits while retain-on-exit is still
    // off, taking the agent's exit status and its error output with it. An
    // agent that fails on its first line is exactly the case worth reporting,
    // so hold the window open with a placeholder that cannot exit, configure
    // the session, and only then hand the pane to the agent command.
    let mut tmux = tmux_command();
    tmux.args(["new-session", "-d", "-s", name, "-c"])
        .arg(cwd)
        .args(
            environment
                .iter()
                .flat_map(|(key, value)| ["-e".to_owned(), format!("{key}={value}")]),
        )
        // `cat` blocks on the pane's own tty forever and writes nothing, so it
        // neither races the configuration below nor leaves output behind.
        .arg("/bin/cat");
    let created = run_command_with_timeout(&mut tmux, Duration::from_secs(5))?;
    if !created.status.success() {
        return Err(anyhow!(String::from_utf8_lossy(&created.stderr).to_string()));
    }
    if let Err(error) =
        configure_tmux_session(name).and_then(|()| respawn_tmux_pane(name, cwd, command))
    {
        kill_tmux_session(name);
        return Err(error);
    }
    Ok(())
}

/// Replaces whatever the pane is running with `command`, keeping the window and
/// its already-installed options.
pub(crate) fn respawn_tmux_pane(name: &str, cwd: &Path, command: &str) -> Result<()> {
    let mut tmux = tmux_command();
    tmux.args(["respawn-pane", "-k", "-t", name, "-c"])
        .arg(cwd)
        .arg(command);
    let respawned = run_command_with_timeout(&mut tmux, Duration::from_secs(5))?;
    if !respawned.status.success() {
        return Err(anyhow!(
            String::from_utf8_lossy(&respawned.stderr).to_string()
        ));
    }
    Ok(())
}

pub(crate) fn start_empty_tmux_session(
    name: &str,
    cwd: &Path,
    environment: &[(&'static str, String)],
) -> Result<()> {
    let mut command = tmux_command();
    command
        .args(["new-session", "-d", "-s", name, "-c"])
        .arg(cwd)
        .args(
            environment
                .iter()
                .flat_map(|(key, value)| ["-e".to_owned(), format!("{key}={value}")]),
        );
    let created = run_command_with_timeout(&mut command, Duration::from_secs(5))?;
    if !created.status.success() {
        return Err(anyhow!(String::from_utf8_lossy(&created.stderr).to_string()));
    }
    if let Err(error) = configure_tmux_session(name) {
        kill_tmux_session(name);
        return Err(error);
    }
    Ok(())
}

pub(crate) fn configure_tmux_session(name: &str) -> Result<()> {
    let mut command = tmux_command();
    command.args(["set-option", "-t", name, "remain-on-exit", "on"]);
    let configured = run_command_with_timeout(&mut command, Duration::from_secs(5))?;
    if !configured.status.success() {
        return Err(anyhow!(
            String::from_utf8_lossy(&configured.stderr).to_string()
        ));
    }
    let mut command = tmux_command();
    command.args(["set-option", "-t", name, "history-limit", "50000"]);
    let history = run_command_with_timeout(&mut command, Duration::from_secs(5))?;
    if !history.status.success() {
        return Err(anyhow!(String::from_utf8_lossy(&history.stderr).to_string()));
    }
    Ok(())
}

pub(crate) fn kill_tmux_session(name: &str) {
    let mut command = tmux_command();
    command.args(["kill-session", "-t", name]);
    let _ = run_command_with_timeout(&mut command, Duration::from_secs(5));
}

pub(crate) fn list_operon_tmux_sessions() -> Result<Vec<OrphanedTmuxSession>> {
    let mut command = tmux_command();
    command.args([
        "list-sessions",
        "-F",
        "#{session_name}\t#{session_created}\t#{pane_current_path}",
    ]);
    let output = run_command_with_timeout(&mut command, Duration::from_secs(5))?;
    tmux_session_listing_result(
        output.status.success(),
        &String::from_utf8_lossy(&output.stdout),
        &String::from_utf8_lossy(&output.stderr),
    )
}

pub(crate) fn tmux_session_listing_result(
    success: bool,
    stdout: &str,
    stderr: &str,
) -> Result<Vec<OrphanedTmuxSession>> {
    if success {
        Ok(parse_operon_tmux_sessions(stdout))
    } else if tmux_error_state(stderr) == TmuxState::Gone {
        Ok(Vec::new())
    } else {
        Err(anyhow!(stderr.trim().to_owned()))
    }
}

pub(crate) fn parse_operon_tmux_sessions(output: &str) -> Vec<OrphanedTmuxSession> {
    output
        .lines()
        .filter_map(|line| {
            let mut fields = line.splitn(3, '\t');
            let name = fields.next()?.to_owned();
            let created_at = fields.next()?.parse().ok()?;
            let cwd = PathBuf::from(fields.next()?);
            (is_safe_tmux_name(&name) && cwd.is_dir()).then_some(OrphanedTmuxSession {
                name,
                created_at,
                cwd: cwd.canonicalize().unwrap_or(cwd),
            })
        })
        .collect()
}
pub(crate) const OPERON_CURSOR_MARKER: &str = "__OPERON_CURSOR__:";

pub(crate) fn parse_tmux_cursor_coordinate(text: &str) -> Result<(u16, u16)> {
    let mut parts = text.split(',');
    let x = parts
        .next()
        .ok_or_else(|| anyhow!("missing x coordinate"))?
        .trim()
        .parse::<u16>()?;
    let y = parts
        .next()
        .ok_or_else(|| anyhow!("missing y coordinate"))?
        .trim()
        .parse::<u16>()?;
    Ok((x, y))
}

pub(crate) fn parse_tmux_capture_with_cursor(raw: &str) -> TerminalCapture {
    if let Some(pos) = raw.rfind(OPERON_CURSOR_MARKER) {
        let (body, marker_part) = raw.split_at(pos);
        let cursor_str = &marker_part[OPERON_CURSOR_MARKER.len()..];
        let cursor_line = cursor_str.lines().next().unwrap_or("").trim();
        let cursor = parse_tmux_cursor_coordinate(cursor_line).ok();
        TerminalCapture {
            output: body.to_string(),
            cursor,
        }
    } else {
        TerminalCapture {
            output: raw.to_string(),
            cursor: None,
        }
    }
}

pub(crate) fn tmux_capture_with_cursor(name: &str) -> Result<TerminalCapture> {
    let scrollback = format!("-{TERMINAL_SCROLLBACK_LINES}");
    let marker_arg = format!("{OPERON_CURSOR_MARKER}#{{cursor_x}},#{{cursor_y}}");
    let mut command = tmux_command();
    command.args([
        "capture-pane",
        "-e",
        "-p",
        "-S",
        scrollback.as_str(),
        "-t",
        name,
        ";",
        "display-message",
        "-p",
        "-t",
        name,
        &marker_arg,
    ]);
    let output = run_command_with_timeout(&mut command, Duration::from_secs(5))?;
    if !output.status.success() {
        return Err(anyhow!(String::from_utf8_lossy(&output.stderr).to_string()));
    }
    let raw = String::from_utf8_lossy(&output.stdout);
    Ok(parse_tmux_capture_with_cursor(&raw))
}

/// Scrollback and all, escapes kept, cursor position dropped.
///
/// `#[cfg(test)]` since change 053, and the reason is worth keeping: the only
/// production caller was the trust-prompt poll, which now reads the visible
/// screen instead. The terminal pane goes through `tmux_capture_with_cursor`
/// directly because it needs the cursor. So what is left is the tests' idea of
/// "everything the pane has ever shown", which is the right question for a test
/// waiting on output and the wrong one for any decision.
#[cfg(test)]
pub(crate) fn tmux_capture(name: &str) -> Result<String> {
    // `-e` preserves terminal SGR sequences. The in-app renderer turns the
    // safe color subset into egui text formatting instead of displaying escape
    // codes, so Claude's headings, prompts, and warnings retain their cues.
    tmux_capture_with_cursor(name).map(|capture| capture.output)
}

#[cfg(test)]
pub(crate) fn tmux_cursor_position(name: &str) -> Result<(u16, u16)> {
    let mut command = tmux_command();
    command.args([
        "display-message",
        "-p",
        "-t",
        name,
        "#{cursor_x},#{cursor_y}",
    ]);
    let output = run_command_with_timeout(&mut command, Duration::from_secs(5))?;
    if !output.status.success() {
        return Err(anyhow!(String::from_utf8_lossy(&output.stderr).to_string()));
    }
    parse_tmux_cursor_coordinate(&String::from_utf8_lossy(&output.stdout))
}

/// The visible screen, with its escapes, and nothing from the scrollback.
///
/// The third capture in this file, and the distinction each one makes is the
/// point. `tmux_capture` keeps 5 000 lines of history because the terminal pane
/// is something a person scrolls. `tmux_capture_screen` drops the escapes
/// because a run-state marker is a word. This keeps the escapes and drops the
/// history, which is what a *decision about what is on screen right now* needs:
/// a question that has scrolled away is not being asked, and answering it
/// presses a key into whatever replaced it.
///
/// It is also exactly the form the `--save` option of
/// `scripts/check-trust-prompts.sh` writes, so the fixtures in `src/tests.rs`
/// and the bytes the caller passes are the same bytes.
pub(crate) fn tmux_capture_visible_screen(name: &str) -> Result<String> {
    let mut command = tmux_command();
    command.args(["capture-pane", "-e", "-p", "-t", name]);
    let output = run_command_with_timeout(&mut command, Duration::from_secs(5))?;
    if !output.status.success() {
        return Err(anyhow!(String::from_utf8_lossy(&output.stderr).to_string()));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub(crate) fn tmux_capture_screen(name: &str) -> Result<String> {
    // The visible screen only. Scrollback would keep an interrupt hint from a
    // turn that ended minutes ago on record, so every later poll would still
    // read as a running turn and the session would never report finishing.
    let mut command = tmux_command();
    command.args(["capture-pane", "-p", "-t", name]);
    let output = run_command_with_timeout(&mut command, Duration::from_secs(20))?;
    if !output.status.success() {
        return Err(anyhow!(String::from_utf8_lossy(&output.stderr).to_string()));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Read the agent's run state from the visible tmux screen.
///
/// The markers are the ones the supported CLIs print today: Codex CLI and
/// Claude Code offer `esc to interrupt` while a turn runs, Antigravity CLI
/// offers `esc to cancel`. A screen that is neither running nor blocked on a
/// prompt counts as idle, so a CLI that renames its hint goes quiet instead of
/// announcing completions that did not happen.
pub(crate) fn detect_agent_activity(screen: &str) -> AgentActivity {
    let tail = agent_status_tail(screen);
    // Running wins over the prompt shapes: a CLI drops its interrupt hint
    // before it can be waiting on an answer, so an agent that merely printed a
    // numbered list mid-turn is still working rather than asking for input.
    if is_agent_working(&tail) {
        AgentActivity::Working
    } else if is_agent_input_prompt(&tail) {
        AgentActivity::AwaitingInput
    } else {
        AgentActivity::Idle
    }
}

/// The bottom lines of the screen, where every supported CLI draws its footer.
pub(crate) fn agent_status_tail(screen: &str) -> String {
    let lines = screen
        .lines()
        .map(|line| line.trim_end())
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>();
    lines[lines.len().saturating_sub(ACTIVITY_STATUS_TAIL_LINES)..].join("\n")
}

pub(crate) fn is_agent_working(tail: &str) -> bool {
    let lowercase = tail.to_ascii_lowercase();
    let interrupt_hint = [
        // Codex CLI: `• Working (7s • esc to interrupt)`.
        "esc to interrupt",
        // Antigravity CLI replaces `? for shortcuts` with this while working.
        "esc to cancel",
        "ctrl+c to interrupt",
    ]
    .iter()
    .any(|marker| lowercase.contains(marker));
    interrupt_hint || has_running_turn_timer(tail)
}

/// Claude Code runs a turn under `✻ Moonwalking… (7s · thinking with xhigh
/// effort)` and offers no interrupt hint there. The verb is picked at random
/// per turn and the suffix varies, so match the one stable part: an ellipsis
/// followed by an elapsed-seconds counter. Its finished line — `✻ Crunched for
/// 21s` — has no parenthesized counter and so does not match.
pub(crate) fn has_running_turn_timer(tail: &str) -> bool {
    tail.lines().any(|line| {
        line.split("… (").skip(1).any(|rest| {
            let seconds = rest
                .chars()
                .take_while(|character| character.is_ascii_digit())
                .count();
            seconds > 0 && rest[seconds..].starts_with('s')
        })
    })
}

/// Whether the CLI is blocked on a question only a person can answer.
///
/// Approval wording differs per CLI and per release, so this recognizes the
/// shared shapes — the workspace-trust prompt and a numbered yes/no list —
/// rather than trying to enumerate every prompt.
pub(crate) fn is_agent_input_prompt(tail: &str) -> bool {
    if is_workspace_trust_prompt(tail) {
        return true;
    }
    let tail = tail.to_ascii_lowercase();
    [
        "do you want to proceed",
        "yes, and don't ask again",
        "yes, allow always",
        "1. yes",
        "(y/n)",
    ]
    .iter()
    .any(|marker| tail.contains(marker))
}

/// The last exchange in a managed terminal, as a session row quotes it.
///
/// Either half may be missing and the row still says something worth reading: a
/// long answer pushes the request off the visible screen, and a turn that has
/// only just started has no answer yet.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ConversationPreview {
    pub(crate) request: Option<String>,
    pub(crate) reply: Option<String>,
}

/// How one CLI marks up its own conversation on screen.
///
/// Like the tmux error strings elsewhere, these are matching patterns against
/// another program's output rather than this app's copy: they are exactly the
/// glyphs the CLI prints today. A CLI that restyles its transcript stops being
/// quoted, which is the failure worth having — the alternative is a row that
/// confidently quotes the wrong half of the screen.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ScreenGrammar {
    /// In front of a line the person typed.
    pub(crate) request: char,
    /// In front of a line the agent wrote. Antigravity CLI marks its tool calls
    /// and leaves its prose unmarked, so it has none and its answer is found by
    /// where it sits instead.
    pub(crate) reply: Option<char>,
}

pub(crate) fn screen_grammar(provider: CliProvider) -> ScreenGrammar {
    match provider {
        CliProvider::Codex => ScreenGrammar {
            request: '›',
            reply: Some('•'),
        },
        CliProvider::Claude => ScreenGrammar {
            request: '❯',
            reply: Some('⏺'),
        },
        CliProvider::Gemini => ScreenGrammar {
            request: '>',
            reply: None,
        },
    }
}

/// Glyphs that start a line nobody said: tool calls and their results, warnings,
/// spinner frames, and the frames the CLIs draw their banners in.
pub(crate) const SCREEN_NOISE_MARKERS: &[char] = &[
    '⎿', '└', '├', '│', '╭', '╮', '╰', '╯', '┌', '┐', '┘', '┬', '┴', '┼', '⚠', '✻', '✽', '·', '●',
];

pub(crate) fn is_screen_noise_marker(marker: char) -> bool {
    // Two whole blocks are never prose here: braille, which is how all three
    // CLIs animate a spinner, and block elements, which is what every one of
    // them draws its startup banner out of. A banner matters because it is the
    // one thing on screen before anybody has said anything, and a row that
    // quoted it would open every session by reciting a logo.
    ('\u{2800}'..='\u{28FF}').contains(&marker)
        || ('\u{2580}'..='\u{259F}').contains(&marker)
        || SCREEN_NOISE_MARKERS.contains(&marker)
}

/// A horizontal rule, which is where a CLI separates one region of its screen
/// from the next.
pub(crate) fn is_screen_rule(line: &str) -> bool {
    !line.is_empty() && line.chars().all(|character| matches!(character, '─' | '━'))
}

/// `1. Yes, continue` and friends: the CLI offering choices, not the person
/// making one. Codex and Claude Code both draw those behind the same glyph they
/// echo a typed request with.
pub(crate) fn is_numbered_choice(text: &str) -> bool {
    let digits = text
        .chars()
        .take_while(|character| character.is_ascii_digit())
        .count();
    digits > 0 && text[digits..].starts_with('.')
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScreenRole {
    Request,
    Reply,
}

pub(crate) enum ScreenLine<'a> {
    /// This line opens a message, and carries its first fragment.
    Opens(ScreenRole, &'a str),
    /// The pane wrapped the message being read; this is the rest of it.
    Continues(&'a str),
    /// Nothing quotable, and the end of whatever message was being read.
    Ends,
}

/// Which part of a conversation, if any, one screen line is.
///
/// `open` says whether a message is already being read, because the same
/// indented line means different things either side of that: inside a message
/// it is the pane's wrapping, and outside one it is either an unmarked answer
/// (Antigravity CLI) or a tool's own output (everywhere else).
pub(crate) fn classify_screen_line<'a>(
    line: &'a str,
    grammar: ScreenGrammar,
    open: bool,
) -> ScreenLine<'a> {
    let indented = line.starts_with([' ', '\t']);
    let trimmed = line.trim();
    if trimmed.is_empty() || is_screen_rule(trimmed) {
        return ScreenLine::Ends;
    }
    let mut characters = trimmed.chars();
    let Some(marker) = characters.next() else {
        return ScreenLine::Ends;
    };
    let rest = characters.as_str().trim_start();
    if marker == grammar.request {
        if rest.is_empty() || is_numbered_choice(rest) {
            return ScreenLine::Ends;
        }
        return ScreenLine::Opens(ScreenRole::Request, rest);
    }
    if Some(marker) == grammar.reply {
        // Codex CLI draws its run-state line behind the same bullet as its
        // prose — `• Working (4s • esc to interrupt)` is the CLI talking about
        // itself, not an answer.
        if rest.is_empty() || is_agent_working(trimmed) {
            return ScreenLine::Ends;
        }
        return ScreenLine::Opens(ScreenRole::Reply, rest);
    }
    if is_screen_noise_marker(marker) || !indented {
        return ScreenLine::Ends;
    }
    if open {
        return ScreenLine::Continues(trimmed);
    }
    match grammar.reply {
        Some(_) => ScreenLine::Ends,
        // Antigravity CLI leaves its prose unmarked, so an indented run that
        // follows something else is the only shape its answer has.
        None => ScreenLine::Opens(ScreenRole::Reply, trimmed),
    }
}

/// Where the composer starts, so the empty prompt waiting at the bottom is not
/// read as something the person just said.
///
/// Every supported CLI draws its composer last and behind the same glyph it
/// echoes a request with, so the bottom-most one is always the composer. Claude
/// Code and Antigravity CLI box theirs in rules, and the rule above the prompt
/// belongs to the composer too.
pub(crate) fn composer_start(lines: &[&str], grammar: ScreenGrammar) -> usize {
    let Some(prompt) = lines
        .iter()
        .rposition(|line| line.trim_start().starts_with(grammar.request))
    else {
        return lines.len();
    };
    match prompt.checked_sub(1) {
        Some(above) if is_screen_rule(lines[above].trim()) => above,
        _ => prompt,
    }
}

/// Quote the last thing each side said, from the visible tmux screen.
///
/// The screen is what the poller already reads for run state, so quoting it
/// costs no further work and covers every CLI the same way. The price is that
/// this is the CLI's drawing rather than its transcript: a request that scrolled
/// away is simply gone, and what Claude Code last "said" may be the sentence it
/// puts in front of a tool call, because that is what the screen last showed.
pub(crate) fn read_conversation_preview(agent: &str, screen: &str) -> ConversationPreview {
    let Some(grammar) = cli_provider_for_agent(agent).map(screen_grammar) else {
        return ConversationPreview::default();
    };
    let lines = screen
        .lines()
        .map(|line| line.trim_end())
        .collect::<Vec<_>>();
    let mut messages: Vec<(ScreenRole, String)> = Vec::new();
    let mut open = false;
    for line in &lines[..composer_start(&lines, grammar)] {
        match classify_screen_line(line, grammar, open) {
            ScreenLine::Opens(role, text) => {
                messages.push((role, text.to_owned()));
                open = true;
            }
            ScreenLine::Continues(text) => {
                if let Some((_, body)) = messages.last_mut() {
                    append_wrapped_fragment(body, text);
                }
            }
            ScreenLine::Ends => open = false,
        }
    }
    let last = |wanted: ScreenRole| {
        messages
            .iter()
            .rev()
            .find(|(role, _)| *role == wanted)
            .and_then(|(_, body)| trim_conversation_preview(body))
    };
    ConversationPreview {
        request: last(ScreenRole::Request),
        reply: last(ScreenRole::Reply),
    }
}

/// Rejoin a line the pane wrapped.
///
/// A pane wraps wherever the column runs out. In English that is at a space the
/// pane then swallowed, so the space has to go back; in Japanese it is
/// mid-sentence with no space involved, and putting one there would insert a
/// character the person never typed.
///
/// Nothing on screen says which of the two happened, so this reads the seam and
/// takes the safer error. A Latin word that wrapped away from Japanese around it
/// rejoins without its space — `beta.txt を読んで` comes back as
/// `beta.txtを読んで`, which is ordinary Japanese. The error in the other
/// direction is a space dropped into the middle of a Japanese word, which is
/// not.
pub(crate) fn append_wrapped_fragment(body: &mut String, fragment: &str) {
    let seam_is_unspaced = body.chars().next_back().is_some_and(is_unspaced_script)
        || fragment.chars().next().is_some_and(is_unspaced_script);
    if !body.is_empty() && !seam_is_unspaced {
        body.push(' ');
    }
    body.push_str(fragment);
}

/// Scripts written without spaces between words.
pub(crate) fn is_unspaced_script(character: char) -> bool {
    matches!(character as u32,
        0x3000..=0x303F     // CJK punctuation
        | 0x3040..=0x30FF   // Hiragana and Katakana
        | 0x3400..=0x4DBF   // CJK ideographs, extension A
        | 0x4E00..=0x9FFF   // CJK unified ideographs
        | 0xAC00..=0xD7AF   // Hangul syllables
        | 0xF900..=0xFAFF   // CJK compatibility ideographs
        | 0xFF00..=0xFF60   // Fullwidth forms
        | 0xFFE0..=0xFFE6)
}

/// A row quotes one line, so a whole answer is more than it can hold. The cap is
/// on the string rather than only on the drawn width: an agent that pastes a
/// file into its reply would otherwise have every character of it laid out on
/// every frame, to be thrown away at the row's right edge.
pub(crate) const CONVERSATION_PREVIEW_MAX_CHARS: usize = 200;

pub(crate) fn trim_conversation_preview(body: &str) -> Option<String> {
    let body = body.trim();
    if body.is_empty() {
        return None;
    }
    let mut preview = body
        .chars()
        .take(CONVERSATION_PREVIEW_MAX_CHARS)
        .collect::<String>();
    if body.chars().count() > CONVERSATION_PREVIEW_MAX_CHARS {
        preview.push('…');
    }
    Some(preview)
}

/// Fold one observation into a session's tracker, reporting the change worth
/// telling the user about.
///
/// A visible interrupt hint is positive evidence and counts immediately; its
/// absence is not, so leaving `Working` needs repeated agreement. That keeps a
/// single redraw caught mid-frame from announcing a turn that is still running.
pub(crate) fn observe_agent_activity(
    tracker: &mut ActivityTracker,
    observed: AgentActivity,
) -> Option<ActivityNotice> {
    let required = match observed {
        AgentActivity::Working => 1,
        AgentActivity::AwaitingInput | AgentActivity::Idle => ACTIVITY_CONFIRMATIONS,
    };
    let agreed = match tracker.candidate {
        Some((candidate, seen)) if candidate == observed => seen.saturating_add(1),
        _ => 1,
    };
    tracker.candidate = Some((observed, agreed));
    if agreed < required {
        return None;
    }
    let previous = tracker.settled.replace(observed);
    activity_notice(previous, observed)
}

pub(crate) fn activity_notification_body(
    notice: ActivityNotice,
    session: &str,
    agent: &str,
) -> String {
    // Deliberately not `agent_choice_copy`: that copy is picker text describing
    // a choice ("別のコマンド"), while a notification has to name the CLI that
    // produced it.
    let agent = match agent {
        "codex" => "Codex CLI",
        "claude" => "Claude Code",
        "gemini" => "Antigravity CLI",
        _ => tr("カスタム CLI"),
    };
    match notice {
        ActivityNotice::Finished => tf!(
            "{agent} が完了しました: {session}",
            agent = agent,
            session = session
        ),
        ActivityNotice::NeedsInput => tf!(
            "{agent} が入力を待っています: {session}",
            agent = agent,
            session = session
        ),
    }
}

pub(crate) fn activity_notice(
    previous: Option<AgentActivity>,
    next: AgentActivity,
) -> Option<ActivityNotice> {
    match (previous, next) {
        // The first settled reading is a baseline: a session that was already
        // idle when Operon started did not just finish anything.
        (None, _) => None,
        (Some(AgentActivity::Working), AgentActivity::Idle) => Some(ActivityNotice::Finished),
        (Some(AgentActivity::Working | AgentActivity::Idle), AgentActivity::AwaitingInput) => {
            Some(ActivityNotice::NeedsInput)
        }
        _ => None,
    }
}

/// A phrase one of the agent CLIs prints, and the day somebody watched it print
/// it — `YYYY-MM-DD`, or `UNOBSERVED` for a phrase this repository carries
/// without a capture behind it.
///
/// The second field is not decoration. This function used to answer the trust
/// prompt with a blind `Enter` chosen from a guess about the layout, and the
/// guess was wrong for the CLI it was written for. A phrase nobody has watched
/// a CLI print is a guess wearing the same clothes as evidence, so it says so,
/// and `scripts/check-trust-prompts.sh` re-observes the rows on demand.
pub(crate) type TrustPhrase = (&'static str, &'static str);

/// Wording that identifies the one-time workspace-trust question.
///
/// Every row names trust, and `every_trust_marker_names_trust` requires it.
/// That is not a tidiness rule. An earlier cut of this searched
/// `TRUST_PROMPT_YES` as well, to avoid spelling `yes, i trust this folder` in
/// two lists — and it made `yes, continue` a reason to believe a screen was
/// about trust. Codex words an ordinary approval with that option, so Operon
/// would have pressed Enter on `Codex wants to run rm -rf build`. The option
/// half says which key; only the question says whether to press one.
///
/// No row here is also a `TRUST_PROMPT_YES` row, and
/// `no_trust_marker_is_also_an_option` holds that. Sharing even one phrase
/// would leave the separation true of every phrase except the shared one,
/// which is the same hole a size smaller: a menu offering
/// `Yes, I trust this folder` about something other than a folder would be
/// recognised by its option alone.
///
/// Lowercase throughout: the screen is lowercased before it is searched.
pub(crate) const TRUST_PROMPT_MARKERS: [TrustPhrase; 6] = [
    (
        "quick safety check: is this a project you created or one you trust",
        "2026-09-17",
    ),
    // Codex, on this machine, on this date.
    ("do you trust the contents of this directory", "2026-09-17"),
    // Antigravity's `agy`, likewise. Its question and Codex's differ by one
    // word, which is exactly why neither is written from memory.
    ("do you trust the contents of this project", "2026-09-17"),
    // The three below are inherited. They were in this list before anything in
    // the repository recorded where a phrase came from, and no run of
    // `scripts/check-trust-prompts.sh` has yet put one on a screen. They stay
    // because dropping a marker can only narrow what is recognised, and a
    // narrower marker list means a prompt left unanswered — but they are
    // labelled so the next reader does not mistake them for measurements.
    ("do you trust this folder", "UNOBSERVED"),
    ("do you trust the files in this folder", "UNOBSERVED"),
    ("trust this workspace", "UNOBSERVED"),
];

/// The affirmative option, as each CLI prints it on its own line.
pub(crate) const TRUST_PROMPT_YES: [TrustPhrase; 3] = [
    // Claude Code and Antigravity's `agy` word it identically.
    ("yes, i trust this folder", "2026-09-17"),
    // Codex. Note it carries no "trust": the option half cannot stand in for
    // the recognition half.
    ("yes, continue", "2026-09-17"),
    ("yes, i trust the authors", "UNOBSERVED"),
];

/// The refusing option. Needed even though it is never chosen: the option lines
/// have to be found in screen order before the cursor's distance from the
/// affirmative one can be counted, and on Claude Code the refusing option is
/// the one the cursor starts on.
pub(crate) const TRUST_PROMPT_NO: [TrustPhrase; 3] = [
    ("no, exit", "2026-09-17"),
    ("no, quit", "2026-09-17"),
    ("no, don't trust", "UNOBSERVED"),
];

/// The glyph each CLI parks at the start of the selected line. Three CLIs,
/// three different marks, and `>` is ASCII while the other two are not.
///
/// These are not Operon's icons and do not belong in `ICON_VOCABULARY`: nothing
/// draws them. They are matching patterns for another product's output, in the
/// same category as the tmux error strings elsewhere in this file.
pub(crate) const TRUST_PROMPT_CURSORS: [char; 3] = ['❯', '›', '>'];

/// How many option lines a trust prompt is allowed to have before this stops
/// believing it is looking at one.
///
/// Every prompt any of the three CLIs draws has two. The number is not a limit
/// on menus — it is the point past which a count of matching lines is better
/// explained by the screen having been misread than by a CLI having grown
/// options, and the answer to a misread screen is to press nothing. Without it
/// the movement keys are bounded only by the pane height.
const TRUST_PROMPT_MAX_OPTIONS: usize = 6;

/// The screen as this module searches it: escapes gone, ASCII lowercased.
///
/// Stripped and not merely lowercased, because the callers pass a
/// `capture-pane -e` capture, and a CLI is free to colour a word in the middle
/// of its question exactly the way Claude Code colours the words in its
/// options — at which point a marker stops being a substring and the question
/// goes unrecognised while plainly on screen.
fn as_searched(screen: &str) -> String {
    strip_terminal_escapes(screen).to_ascii_lowercase()
}

pub(crate) fn is_workspace_trust_prompt(output: &str) -> bool {
    holds_a_trust_question(&as_searched(output))
}

/// The recognition half, over an already-normalised screen, so the one caller
/// that has normalised it does not pay for a second copy of a whole pane.
fn holds_a_trust_question(searched: &str) -> bool {
    // Whitespace collapsed across line ends before the search, because a marker
    // is a sentence and a sentence wraps. Claude Code's is 66 characters and is
    // the only one it has: in an 80-column pane it clears the edge by a single
    // word, and at 60 it breaks in half and the prompt goes wholly
    // unrecognised. Measured, not supposed: the probe in
    // scripts/check-trust-prompts.sh was pointed at a 60-column pane and
    // reported Claude Code showing no trust prompt at all.
    // `start_tmux_agent_session` passes no `-x`, so the width is tmux's
    // `default-size` and belongs to whoever configured tmux.
    //
    // Only this half needs it. An option is two or three words and stays on its
    // line, and the line is where the cursor has to be found anyway.
    let unwrapped = searched.split_whitespace().collect::<Vec<_>>().join(" ");
    TRUST_PROMPT_MARKERS
        .iter()
        .any(|(phrase, _)| unwrapped.contains(phrase))
}

/// Which keys answer the workspace-trust prompt on screen, in the order they
/// must be sent — or `None`, meaning send nothing.
///
/// `None` covers two cases on purpose: the screen is not a trust prompt, and it
/// is one drawn in a shape this cannot read. Both leave the prompt up for a
/// person, which is the safe direction for a question about trust. The
/// alternative is what this replaces — a blind `Enter` that confirms whichever
/// option the cursor happens to be sitting on, which on Claude Code is
/// `No, exit`, so the session Operon had just opened exited instead.
pub(crate) fn workspace_trust_answer_keys(screen: &str) -> Option<Vec<&'static str>> {
    let screen = as_searched(screen);
    if !holds_a_trust_question(&screen) {
        return None;
    }

    // The option lines in screen order, each with whether it is the affirmative
    // one and whether the cursor is parked on it. Distance is counted in
    // options and not in rows: the three CLIs space their lists differently,
    // and one blank row between two options would otherwise cost a keypress.
    let mut options = Vec::new();
    for line in screen.lines() {
        let says_yes = TRUST_PROMPT_YES
            .iter()
            .any(|(phrase, _)| line.contains(phrase));
        let says_no = TRUST_PROMPT_NO
            .iter()
            .any(|(phrase, _)| line.contains(phrase));
        match (says_yes, says_no) {
            (false, false) => continue,
            // Both on one line is a layout nobody has seen, and guessing which
            // half the cursor would confirm is how this function got its bug.
            (true, true) => return None,
            _ => {}
        }
        // At the start of the line, after its indent. A cursor glyph anywhere
        // else belongs to the option's own text — `>` in particular is a
        // character an option is allowed to contain.
        let selected = line.trim_start().starts_with(&TRUST_PROMPT_CURSORS[..]);
        options.push((says_yes, selected));
        if options.len() > TRUST_PROMPT_MAX_OPTIONS {
            return None;
        }
    }

    // Exactly one affirmative option and exactly one cursor, or this is not a
    // list this can read. Half-drawn — body rendered, options not yet — lands
    // here too, and the caller's poll loop looks again in 160ms.
    let affirmative = single_position(options.iter().map(|(says_yes, _)| *says_yes))?;
    let cursor = single_position(options.iter().map(|(_, selected)| *selected))?;

    let step = if affirmative >= cursor { "Down" } else { "Up" };
    let mut keys = vec![step; affirmative.abs_diff(cursor)];
    keys.push("Enter");
    Some(keys)
}

/// The index of the one `true`, or `None` for none and for more than one.
fn single_position(flags: impl Iterator<Item = bool>) -> Option<usize> {
    let mut found = None;
    for (index, flag) in flags.enumerate() {
        if flag {
            if found.is_some() {
                return None;
            }
            found = Some(index);
        }
    }
    found
}

pub(crate) fn auto_approve_workspace_trust_prompt(name: &str) -> Result<bool> {
    // Do not blindly send Enter after launch: agents can subsequently ask
    // about tools, file changes, or privileged actions. This one-time, bounded
    // handshake only advances a recognized workspace-trust prompt, and it sends
    // what `workspace_trust_answer_keys` read off the screen rather than a key
    // chosen when this function was written. Which option `Enter` lands on is a
    // property of the screen, so it is decided by looking at the screen.
    //
    // The *visible* screen, through `tmux_capture_visible_screen` rather than
    // the `tmux_capture` the terminal pane uses. That one carries 5 000 lines of
    // scrollback, and this is also the resume path: `claude --resume` replays a
    // whole prior conversation into the pane before the first look, so a
    // restored message that merely *quotes* a trust prompt — a pasted
    // transcript, a diff of this file — would put a marker, a yes line and a
    // cursor into history, in a folder that was trusted long ago. The keys
    // would then land in a live agent's composer.
    let deadline = Instant::now() + Duration::from_secs(8);
    while Instant::now() < deadline {
        if let Ok(output) = tmux_capture_visible_screen(name) {
            if let Some(keys) = workspace_trust_answer_keys(&output) {
                // Through `tmux_send_input` rather than a second `send-keys`
                // loop beside it: it is already the one place that knows how a
                // run of keys reaches a pane in a single tmux invocation, which
                // is what keeps the move and the confirm from being split by a
                // redraw in between.
                tmux_send_input(
                    name,
                    &keys
                        .into_iter()
                        .map(|key| TerminalInput::Key(key.to_owned()))
                        .collect::<Vec<_>>(),
                )?;
                return Ok(true);
            }
        }
        thread::sleep(Duration::from_millis(160));
    }
    Ok(false)
}

pub(crate) fn tmux_send_input(name: &str, input: &[TerminalInput]) -> Result<()> {
    if input.is_empty() {
        return Ok(());
    }
    // A single tmux invocation preserves the exact order of text and control
    // keys. This lets full-screen TUIs receive ordinary typing, cursor keys,
    // Enter, and control chords exactly as if the pane were attached.
    let mut command = tmux_command();
    for (index, event) in input.iter().enumerate() {
        command.args(["send-keys", "-t", name]);
        match event {
            TerminalInput::Text(text) => {
                command.args(["-l", text]);
            }
            TerminalInput::Key(key) => {
                command.arg(key);
            }
        }
        if index + 1 < input.len() {
            command.arg(";");
        }
    }
    let submitted = run_command_with_timeout(&mut command, Duration::from_secs(5))?;
    if submitted.status.success() {
        Ok(())
    } else {
        Err(anyhow!(
            String::from_utf8_lossy(&submitted.stderr).to_string()
        ))
    }
}

pub(crate) fn tmux_resize(name: &str, (columns, rows): (u16, u16)) -> Result<()> {
    if !is_safe_tmux_name(name) {
        return Err(anyhow!(tf!("管理ターミナル名が不正です")));
    }
    let mut command = tmux_command();
    command.args([
        "resize-window",
        "-t",
        name,
        "-x",
        &columns.to_string(),
        "-y",
        &rows.to_string(),
    ]);
    let resized = run_command_with_timeout(&mut command, Duration::from_secs(5))?;
    if resized.status.success() {
        Ok(())
    } else {
        Err(anyhow!(String::from_utf8_lossy(&resized.stderr).to_string()))
    }
}

pub(crate) fn open_tmux(name: &str) -> Result<()> {
    if !is_safe_tmux_name(name) {
        return Err(anyhow!(tf!("管理ターミナル名が不正です")));
    }
    let script = format!(
        "tell application \"Terminal\" to do script \"tmux attach -t {}\"",
        name.replace('"', "\\\"")
    );
    let mut command = Command::new("osascript");
    command.args(["-e", &script]);
    run_system_command(&mut command).context(tr("ターミナルに tmux への接続を依頼"))
}
/// The only place a managed tmux session gets its name. It sat inline at five
/// call sites, which is how the name the app wrote and the prefix
/// `is_safe_tmux_name` recognizes drifted apart without a test noticing: the
/// gate was only ever handed literals, never a name the app had built.
pub(crate) fn managed_tmux_name(id: Uuid) -> String {
    format!("{MANAGED_TMUX_PREFIX}{}", &id.simple().to_string()[..8])
}

pub(crate) fn is_safe_tmux_name(name: &str) -> bool {
    (name.starts_with(MANAGED_TMUX_PREFIX) || name.starts_with(MANAGED_TMUX_LEGACY_PREFIX))
        && name.len() <= 96
        && name
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '-')
}

impl ConversationPreview {
    pub(crate) fn is_empty(&self) -> bool {
        self.request.is_none() && self.reply.is_none()
    }
}
