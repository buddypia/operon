use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub(crate) enum CliProvider {
    #[default]
    Codex,
    Claude,
    Gemini,
}

impl CliProvider {
    pub(crate) fn agent(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Claude => "claude",
            Self::Gemini => "gemini",
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Codex => "Codex CLI",
            Self::Claude => "Claude Code",
            Self::Gemini => "Antigravity CLI",
        }
    }

    pub(crate) fn executable(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Claude => "claude",
            Self::Gemini => ANTIGRAVITY_COMMAND,
        }
    }

    pub(crate) fn native_resume_command(self, session_id: &str) -> String {
        match self {
            Self::Codex => format!("codex resume {session_id}"),
            Self::Claude => format!("claude --resume {session_id}"),
            Self::Gemini => format!("{ANTIGRAVITY_COMMAND} --conversation {session_id}"),
        }
    }
}

/// The full-history restore that is running right now, as a person needs it
/// described while they wait for it.
///
/// This is the one job in the app long enough to be waited on: it reads a whole
/// conversation off disk and rewrites it into the destination CLI's own store,
/// and until that finishes there is no session to show. A line of text at the
/// top of a page is where a *result* goes, so the wait gets the window instead.
#[derive(Debug, Clone)]
pub(crate) struct RestoreProgress {
    pub(crate) import_id: Uuid,
    pub(crate) source: CliProvider,
    /// The conversation being read. Two conversations in one project can open
    /// with the same request, so without the ID a restore of the wrong one is
    /// indistinguishable from a restore that failed to restore.
    pub(crate) source_conversation: String,
    pub(crate) destination: CliProvider,
    pub(crate) last_user_message: String,
    pub(crate) started_at: Instant,
}

#[derive(Debug, Clone)]
pub(crate) struct TranscriptMatch {
    pub(crate) provider: String,
    pub(crate) session_id: String,
    pub(crate) title: Option<String>,
    pub(crate) path: PathBuf,
    pub(crate) snippet: String,
    pub(crate) score: Option<f64>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct CliSession {
    pub(crate) provider: CliProvider,
    /// Stable provider conversation identity. Antigravity resumes with this
    /// provider-owned conversation ID.
    pub(crate) native_id: String,
    /// Current CLI resume argument (the stable ID for Codex/Claude; current
    /// conversation ID for Antigravity).
    pub(crate) id: String,
    pub(crate) title: Option<String>,
    pub(crate) last_user_message: Option<String>,
    pub(crate) branch: Option<String>,
    pub(crate) updated_at: u64,
    pub(crate) path: PathBuf,
}

/// A real destination-provider session created by a full-history import.
/// The destination CLI owns this ID and can resume it normally.
#[derive(Debug, Clone)]
pub(crate) struct ImportedNativeSession {
    pub(crate) managed_session_id: Uuid,
    pub(crate) provider: CliProvider,
    pub(crate) native_session_id: Option<String>,
    pub(crate) native_session_path: PathBuf,
    pub(crate) archive_path: PathBuf,
    pub(crate) imported_records: usize,
    /// What became of every record in the source, so the restore can say what
    /// it carried and what it left behind instead of only naming a total.
    pub(crate) classification: RestoreClassification,
    pub(crate) agent_command: String,
}

/// A local-only pointer to a native CLI conversation. Operon deliberately
/// stores no transcript copy: each provider remains the authority for its own
/// conversation and a missing source is simply rediscovered on the next scan.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct NativeSessionReference {
    pub(crate) project_id: Uuid,
    pub(crate) provider: CliProvider,
    pub(crate) session_id: String,
    pub(crate) source_path: PathBuf,
    pub(crate) last_seen_at: u64,
}

/// An auditable record that a session was intentionally continued in another
/// provider. New records preserve a full local transcript snapshot and point
/// at a real provider-owned destination resume ID.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct CliHandoff {
    pub(crate) id: Uuid,
    pub(crate) project_id: Uuid,
    pub(crate) source_provider: CliProvider,
    pub(crate) source_session_id: String,
    pub(crate) source_path: PathBuf,
    pub(crate) target_agent: String,
    pub(crate) note: String,
    pub(crate) created_at: u64,
    #[serde(default)]
    pub(crate) full_history: bool,
    #[serde(default)]
    pub(crate) archive_path: Option<PathBuf>,
    #[serde(default)]
    pub(crate) destination_session_id: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct CliSessionFileScan {
    pub(crate) session: Option<CliSession>,
    /// When the conversation itself began, from the transcript's own first
    /// timestamped record. Unlike the file's modification time, this does not
    /// move while the conversation is live, so it is the only reliable way to
    /// tell which conversation a managed terminal started.
    pub(crate) started_at: Option<u64>,
    pub(crate) bytes_read: u64,
    pub(crate) lines_read: usize,
    pub(crate) truncated: bool,
    pub(crate) unreadable: bool,
}

#[derive(Default)]
pub(crate) struct AntigravityIndexScan {
    pub(crate) conversations: AntigravityConversations,
    pub(crate) truncated: bool,
    pub(crate) unreadable_entries: usize,
}

/// Conversations of one workspace, keyed by ID, each holding the newest time it
/// was seen at and the title to list it under.
pub(crate) type AntigravityConversations = HashMap<String, (u64, Option<String>)>;

#[derive(Debug, Clone)]
pub(crate) struct TranscriptHit {
    pub(crate) snippet: String,
    pub(crate) title: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct CliSessionScan {
    pub(crate) sessions: Vec<CliSession>,
    pub(crate) truncated: bool,
    pub(crate) unreadable_entries: usize,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct TranscriptSearchSnapshot {
    pub(crate) matches: Vec<TranscriptMatch>,
    pub(crate) truncated: bool,
    pub(crate) unreadable_entries: usize,
    pub(crate) scanned_files: usize,
    pub(crate) scanned_bytes: u64,
    pub(crate) scanned_lines: usize,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct TranscriptSearchLimits {
    pub(crate) visited_entries: usize,
    pub(crate) candidates: usize,
    pub(crate) bytes: u64,
    pub(crate) lines: usize,
}

impl Default for TranscriptSearchLimits {
    fn default() -> Self {
        Self {
            visited_entries: FILE_SCAN_VISIT_LIMIT,
            candidates: TRANSCRIPT_CANDIDATE_LIMIT,
            bytes: TRANSCRIPT_SEARCH_BYTE_LIMIT,
            lines: TRANSCRIPT_SEARCH_LINE_LIMIT,
        }
    }
}

#[derive(Default)]
pub(crate) struct AntigravityLogScan {
    pub(crate) conversation_ids: Vec<String>,
    pub(crate) truncated: bool,
    pub(crate) unreadable_entries: usize,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct TranscriptFileSearch {
    pub(crate) hit: Option<TranscriptHit>,
    pub(crate) first_text: Option<String>,
    pub(crate) title: Option<String>,
    pub(crate) bytes_read: u64,
    pub(crate) lines_read: usize,
    pub(crate) truncated: bool,
    pub(crate) unreadable: bool,
}

pub(crate) fn parse_claude_cli_session_limited(
    path: &Path,
    max_bytes: u64,
    max_lines: usize,
) -> CliSessionFileScan {
    let file = match fs::File::open(path) {
        Ok(file) => file,
        Err(_) => {
            return CliSessionFileScan {
                unreadable: true,
                ..CliSessionFileScan::default()
            };
        }
    };
    let metadata = match file.metadata() {
        Ok(metadata) => metadata,
        Err(_) => {
            return CliSessionFileScan {
                unreadable: true,
                ..CliSessionFileScan::default()
            };
        }
    };
    if metadata.len() > TRANSCRIPT_FILE_MAX_BYTES || metadata.len() > max_bytes {
        return CliSessionFileScan {
            truncated: true,
            ..CliSessionFileScan::default()
        };
    }
    let read_limit = metadata.len();
    let mut reader = BufReader::new(file.take(read_limit));
    let mut line = String::new();
    let mut title = None;
    let mut last_user_message = None;
    let mut branch = None;
    let mut saw_message = false;
    let mut started_at = None;
    let mut lines_read = 0;
    let mut reached_eof = false;
    let mut unreadable = false;
    while lines_read < max_lines {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => {
                reached_eof = true;
                break;
            }
            Ok(_) => lines_read += 1,
            Err(_) => {
                unreadable = true;
                break;
            }
        }
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&line) else {
            continue;
        };
        if started_at.is_none() {
            started_at = value
                .get("timestamp")
                .and_then(|value| value.as_str())
                .and_then(parse_rfc3339_utc_seconds);
        }
        match value.get("type").and_then(|value| value.as_str()) {
            Some("custom-title") => {
                title = value
                    .get("customTitle")
                    .and_then(|value| value.as_str())
                    .map(ToOwned::to_owned)
                    .or(title);
            }
            Some("ai-title") if title.is_none() => {
                title = value
                    .get("aiTitle")
                    .and_then(|value| value.as_str())
                    .map(ToOwned::to_owned);
            }
            Some("user") => {
                saw_message = true;
                if let Some(message) = claude_message_text(&value) {
                    last_user_message = Some(message);
                }
                if let Some(value) = value.get("gitBranch").and_then(|value| value.as_str()) {
                    branch = Some(value.to_owned());
                }
            }
            _ => {}
        }
    }
    let bytes_read = read_limit.saturating_sub(reader.get_ref().limit());
    let grew_while_reading = reader
        .get_ref()
        .get_ref()
        .metadata()
        .map(|latest| latest.len() > read_limit)
        .unwrap_or(true);
    CliSessionFileScan {
        session: saw_message.then(|| CliSession {
            provider: CliProvider::Claude,
            native_id: path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or("session")
                .to_owned(),
            id: path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or("session")
                .to_owned(),
            title,
            last_user_message,
            branch,
            updated_at: metadata
                .modified()
                .ok()
                .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                .map(|duration| duration.as_secs())
                .unwrap_or_default(),
            path: path.to_path_buf(),
        }),
        started_at,
        bytes_read,
        lines_read,
        truncated: !reached_eof || grew_while_reading,
        unreadable,
    }
}

pub(crate) fn scan_gemini_cli_sessions_in(
    history_path: &Path,
    project: &Path,
    limit: usize,
) -> CliSessionScan {
    let store = history_path.parent().unwrap_or(Path::new("."));
    let allowed_paths = project_and_worktree_paths(project);
    let index = read_antigravity_history_index(history_path, &allowed_paths);
    // `agy` writes a conversation's first turn to the index before the
    // conversation has an ID, so a conversation that never received a second
    // turn is missing from the index entirely. Its process log names it.
    let logs = scan_antigravity_log_conversations(
        &store.join(ANTIGRAVITY_LOG_DIRECTORY_NAME),
        &allowed_paths,
    );
    let mut conversations = index.conversations;
    let truncated = index.truncated || logs.truncated;
    let unreadable_entries = index.unreadable_entries + logs.unreadable_entries;
    for conversation_id in logs.conversation_ids {
        if conversations.contains_key(&conversation_id) {
            continue;
        }
        // A conversation the index never named has no time and no title of its
        // own, and without a transcript there is nothing to restore either.
        let transcript = antigravity_conversation_transcript_in(store, &conversation_id);
        let Some(opening) = antigravity_transcript_opening(&transcript) else {
            continue;
        };
        conversations.insert(conversation_id, opening);
    }
    let mut sessions = antigravity_cli_sessions(store, history_path, conversations, limit);
    // The index carries no message text a restore can trust, so read each
    // listed conversation's own last request the way the Claude and Codex scans
    // read theirs. Only the sessions already pointing at a transcript are read;
    // the rest have nothing to restore yet.
    for session in &mut sessions {
        if is_antigravity_conversation_transcript(&session.path) {
            session.last_user_message =
                transcript_last_user_message(&session.path, CliProvider::Gemini);
        }
    }
    CliSessionScan {
        sessions,
        truncated,
        unreadable_entries,
    }
}

pub(crate) fn recent_jsonl_files_from_roots(
    roots: &[(String, PathBuf, usize)],
    cap: usize,
    visit_limit: usize,
) -> RecentJsonlScan {
    let mut newest = BinaryHeap::<Reverse<(SystemTime, String, PathBuf)>>::new();
    let mut directories = roots
        .iter()
        .map(|(provider, directory, max_depth)| {
            (provider.clone(), directory.clone(), 0usize, *max_depth)
        })
        .collect::<VecDeque<_>>();
    let mut visited = 0;
    let mut truncated = false;
    let mut unreadable_entries = 0;
    'scan: while let Some((provider, directory, depth, max_depth)) = directories.pop_front() {
        if visited >= visit_limit {
            truncated = true;
            break;
        }
        let entries = match fs::read_dir(directory) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(_) => {
                unreadable_entries += 1;
                continue;
            }
        };
        for entry in entries {
            if visited >= visit_limit {
                truncated = true;
                break 'scan;
            }
            visited += 1;
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => {
                    unreadable_entries += 1;
                    continue;
                }
            };
            let file_type = match entry.file_type() {
                Ok(file_type) => file_type,
                Err(_) => {
                    unreadable_entries += 1;
                    continue;
                }
            };
            if file_type.is_symlink() {
                continue;
            }
            let path = entry.path();
            if file_type.is_file()
                && path
                    .extension()
                    .is_some_and(|extension| extension == "jsonl")
            {
                let modified = entry
                    .metadata()
                    .and_then(|metadata| metadata.modified())
                    .unwrap_or_else(|_| {
                        unreadable_entries += 1;
                        UNIX_EPOCH
                    });
                if cap == 0 {
                    truncated = true;
                    continue;
                }
                newest.push(Reverse((modified, provider.clone(), path)));
                if newest.len() > cap {
                    newest.pop();
                    truncated = true;
                }
            } else if file_type.is_dir() && depth + 1 < max_depth {
                directories.push_back((provider.clone(), path, depth + 1, max_depth));
            }
        }
    }
    let mut files = newest
        .into_iter()
        .map(|Reverse((modified, provider, path))| (modified, provider, path))
        .collect::<Vec<_>>();
    files.sort_by_key(|(modified, _, _)| Reverse(*modified));
    RecentJsonlScan {
        candidates: files
            .into_iter()
            .map(|(_, provider, path)| (provider, path))
            .collect(),
        truncated,
        unreadable_entries,
    }
}

#[cfg(test)]
pub(crate) fn search_transcript_file(path: &Path, words: &[String]) -> Option<String> {
    search_transcript_hit(path, words).map(|hit| hit.snippet)
}

pub(crate) fn read_codex_session_titles(
    path: &Path,
    max_bytes: u64,
    max_lines: usize,
) -> CodexTitleScan {
    let file = match fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return CodexTitleScan::default();
        }
        Err(_) => {
            return CodexTitleScan {
                unreadable: true,
                ..CodexTitleScan::default()
            };
        }
    };
    let metadata = match file.metadata() {
        Ok(metadata) => metadata,
        Err(_) => {
            return CodexTitleScan {
                unreadable: true,
                ..CodexTitleScan::default()
            };
        }
    };
    if metadata.len() > TRANSCRIPT_FILE_MAX_BYTES || metadata.len() > max_bytes {
        return CodexTitleScan {
            truncated: true,
            ..CodexTitleScan::default()
        };
    }
    let read_limit = metadata.len();
    let mut reader = BufReader::new(file.take(read_limit));
    let mut line = String::new();
    let mut scan = CodexTitleScan::default();
    let line_limit = max_lines.min(TRANSCRIPT_FILE_LINE_LIMIT);
    let mut reached_eof = false;
    while scan.scanned_lines < line_limit {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => {
                reached_eof = true;
                break;
            }
            Ok(_) => scan.scanned_lines += 1,
            Err(_) => {
                scan.unreadable = true;
                break;
            }
        }
        let Some((id, title)) = serde_json::from_str::<serde_json::Value>(&line)
            .ok()
            .and_then(|value| {
                Some((
                    value.get("id")?.as_str()?.to_owned(),
                    value.get("thread_name")?.as_str()?.to_owned(),
                ))
            })
        else {
            continue;
        };
        scan.titles.insert(id, title);
    }
    scan.scanned_bytes = read_limit.saturating_sub(reader.get_ref().limit());
    let grew_while_reading = reader
        .get_ref()
        .get_ref()
        .metadata()
        .map(|latest| latest.len() > read_limit)
        .unwrap_or(true);
    scan.truncated = !reached_eof || grew_while_reading;
    scan
}

pub(crate) fn scan_native_cli_sessions(project: &Path, limit: usize) -> CliSessionScan {
    let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else {
        return CliSessionScan::default();
    };
    let scans = [
        scan_codex_cli_sessions_in(
            &home.join(".codex").join("sessions"),
            &home.join(".codex").join("session_index.jsonl"),
            project,
            limit,
        ),
        scan_claude_cli_sessions_in(
            &home
                .join(CLAUDE_STORE_DIRECTORY)
                .join(CLAUDE_PROJECTS_DIRECTORY),
            project,
            limit,
        ),
        scan_gemini_cli_sessions_in(
            &home
                .join(".gemini")
                .join("antigravity-cli")
                .join("history.jsonl"),
            project,
            limit,
        ),
    ];
    let mut sessions = Vec::new();
    let mut truncated = false;
    let mut unreadable_entries = 0;
    for scan in scans {
        sessions.extend(scan.sessions);
        truncated |= scan.truncated;
        unreadable_entries += scan.unreadable_entries;
    }
    sessions.sort_by_key(|session| Reverse(session.updated_at));
    if sessions.len() > limit {
        truncated = true;
    }
    sessions.truncate(limit);
    CliSessionScan {
        sessions,
        truncated,
        unreadable_entries,
    }
}

/// The root of Claude Code's local store, and the two directories under it that
/// Operon reads. Built here so that renaming one reaches every reader at once.
pub(crate) fn claude_store_root() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|home| PathBuf::from(home).join(CLAUDE_STORE_DIRECTORY))
}

pub(crate) fn claude_projects_root() -> Option<PathBuf> {
    claude_store_root().map(|root| root.join(CLAUDE_PROJECTS_DIRECTORY))
}

pub(crate) fn claude_session_registry_root() -> Option<PathBuf> {
    claude_store_root().map(|root| root.join(CLAUDE_SESSION_REGISTRY_DIRECTORY))
}

/// Where Claude Code keeps one conversation's transcript. The registry and the
/// project scan both need this path, so they build it the same way: a workspace
/// is identified by `encode_claude_project_path` and nothing else.
pub(crate) fn claude_transcript_path_in(
    projects_root: &Path,
    workspace: &Path,
    native_session_id: &str,
) -> Option<PathBuf> {
    if !is_safe_cli_session_id(native_session_id) {
        return None;
    }
    let path = projects_root
        .join(encode_claude_project_path(workspace))
        .join(format!("{native_session_id}.jsonl"));
    path.is_file().then_some(path)
}

pub(crate) fn native_session_path_for_project(
    provider: CliProvider,
    project: &Path,
    native_session_id: &str,
) -> Option<PathBuf> {
    if !is_safe_cli_session_id(native_session_id) {
        return None;
    }
    let home = std::env::var_os("HOME").map(PathBuf::from)?;
    match provider {
        CliProvider::Claude => {
            claude_transcript_path_in(&claude_projects_root()?, project, native_session_id)
        }
        CliProvider::Codex => {
            let sessions = scan_codex_cli_sessions_in(
                &home.join(".codex").join("sessions"),
                &home.join(".codex").join("session_index.jsonl"),
                project,
                160,
            );
            sessions
                .sessions
                .into_iter()
                .find(|session| session.native_id == native_session_id)
                .map(|session| session.path)
        }
        CliProvider::Gemini => {
            crate::history::antigravity_conversation_transcript_path(native_session_id)
                .ok()
                .filter(|path| path.is_file())
        }
    }
}

/// The readiness of a local transcript for native session resumption.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TranscriptResumeReadiness {
    /// The transcript exists, contains at least one readable record, and ends cleanly.
    Ready(PathBuf),
    /// The transcript ended with an incomplete or truncated trailing line (e.g. from hard SIGKILL),
    /// which was safely trimmed after backing up the file.
    Sanitized { path: PathBuf, removed_bytes: usize },
    /// The transcript file does not exist, is 0 bytes, or contains 0 valid conversation records.
    EmptyOrMissing,
}

/// Inspect an on-disk transcript file to ensure it can be resumed safely by the target agent CLI.
///
/// An abrupt kill (SIGKILL, reboot) can leave the file ending with a truncated JSON line
/// or leave the file completely empty. If an incomplete trailing line is detected following
/// at least one valid record, it is trimmed and backed up so the CLI parser does not crash.
pub(crate) fn check_transcript_resume_readiness(path: &Path) -> TranscriptResumeReadiness {
    let Ok(metadata) = fs::metadata(path) else {
        return TranscriptResumeReadiness::EmptyOrMissing;
    };
    if !metadata.is_file() || metadata.len() == 0 {
        return TranscriptResumeReadiness::EmptyOrMissing;
    }
    let Ok(content) = fs::read(path) else {
        return TranscriptResumeReadiness::EmptyOrMissing;
    };
    if content.is_empty() {
        return TranscriptResumeReadiness::EmptyOrMissing;
    }

    let Ok(text) = std::str::from_utf8(&content) else {
        return TranscriptResumeReadiness::EmptyOrMissing;
    };

    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.is_empty() {
        return TranscriptResumeReadiness::EmptyOrMissing;
    }

    let mut valid_records = 0usize;
    for &line in &lines[..lines.len() - 1] {
        if serde_json::from_str::<serde_json::Value>(line).is_ok() {
            valid_records += 1;
        }
    }

    let last_line = lines[lines.len() - 1];
    if serde_json::from_str::<serde_json::Value>(last_line).is_ok() {
        return TranscriptResumeReadiness::Ready(path.to_path_buf());
    }

    // The last non-empty line failed JSON parsing. If there are prior valid records,
    // this is a truncated line caused by sudden termination.
    if valid_records > 0 {
        if let Some(sanitized_bytes) = sanitize_transcript_tail_if_needed(path, &content, last_line)
        {
            return TranscriptResumeReadiness::Sanitized {
                path: path.to_path_buf(),
                removed_bytes: sanitized_bytes,
            };
        }
    }

    TranscriptResumeReadiness::EmptyOrMissing
}

/// Safely backup the transcript and trim the unparseable trailing line.
fn sanitize_transcript_tail_if_needed(
    path: &Path,
    content: &[u8],
    last_line: &str,
) -> Option<usize> {
    let last_line_bytes = last_line.as_bytes();
    // Find where the last line starts in content
    let trailing_pos = content
        .windows(last_line_bytes.len())
        .rposition(|window| window == last_line_bytes)?;

    let trimmed = &content[..trailing_pos];
    // Create a backup file with .bak suffix before writing
    let backup_path = path.with_extension("jsonl.bak");
    let _ = fs::write(&backup_path, content);

    // Atomically write the trimmed content
    let temp_path = path.with_extension("jsonl.tmp");
    if fs::write(&temp_path, trimmed).is_err() {
        return None;
    }
    if fs::rename(&temp_path, path).is_err() {
        let _ = fs::remove_file(&temp_path);
        return None;
    }

    let removed_bytes = content.len().saturating_sub(trimmed.len());
    Some(removed_bytes)
}

pub(crate) fn validate_session_transcript_for_resume(
    provider: CliProvider,
    workspace: &Path,
    session_id: &str,
    recorded_path: Option<&Path>,
) -> TranscriptResumeReadiness {
    let path = recorded_path
        .filter(|p| p.is_file())
        .map(Path::to_path_buf)
        .or_else(|| native_session_path_for_project(provider, workspace, session_id));
    let Some(path) = path else {
        return TranscriptResumeReadiness::EmptyOrMissing;
    };
    check_transcript_resume_readiness(&path)
}

/// Older managed Claude terminals were launched without a caller-chosen
/// `--session-id`. Their full transcript still exists locally, so recover it by
/// the time the conversation *began*, which is the only property that ties a
/// transcript to the terminal that created it. Ranking by file modification
/// time cannot work: the live conversation keeps being written, so it drifts
/// away from the terminal's start while an abandoned older transcript stays
/// frozen next to it and wins. A tie is deliberately rejected.
pub(crate) fn recover_managed_claude_session(
    workspace: &Path,
    created_at: u64,
    goal: &str,
) -> UiResult<Option<CliSession>> {
    let home = std::env::var_os("HOME").map(PathBuf::from).ok_or_else(|| {
        tr("ホームディレクトリを取得できないため、Claude の履歴を読めません。").to_owned()
    })?;
    recover_managed_claude_session_in(
        &home
            .join(CLAUDE_STORE_DIRECTORY)
            .join(CLAUDE_PROJECTS_DIRECTORY),
        workspace,
        created_at,
        goal,
    )
}

pub(crate) fn recover_managed_claude_session_in(
    projects_root: &Path,
    workspace: &Path,
    created_at: u64,
    goal: &str,
) -> UiResult<Option<CliSession>> {
    let task = session_display_goal(goal);
    if task.is_empty() {
        return Ok(None);
    }
    let directory = projects_root.join(encode_claude_project_path(workspace));
    let entries = match fs::read_dir(&directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(tf!(
                "Claude 履歴 {p0} の読み取り: {error}",
                error = error,
                p0 = directory.display()
            ))
        }
    };
    let mut candidates = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if entry.file_type().ok().is_none_or(|kind| !kind.is_file())
            || path
                .extension()
                .is_none_or(|extension| extension != "jsonl")
            || path
                .file_stem()
                .is_some_and(|stem| stem.to_string_lossy().starts_with("agent-"))
        {
            continue;
        }
        let Ok(modified) = entry.metadata().and_then(|metadata| metadata.modified()) else {
            continue;
        };
        let updated_at = modified
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or_default();
        // A transcript untouched since before this terminal existed cannot hold
        // its conversation. Dropping those first keeps the full parse below
        // bounded without ranking by a time that moves while a session is live.
        if updated_at.saturating_add(MANAGED_SESSION_CLOCK_SKEW_SECONDS) < created_at {
            continue;
        }
        candidates.push((updated_at, path));
    }
    candidates.sort_by_key(|(updated_at, _)| *updated_at);
    let mut matches = Vec::new();
    let mut remaining_bytes = TRANSCRIPT_SEARCH_BYTE_LIMIT;
    let mut remaining_lines = TRANSCRIPT_SEARCH_LINE_LIMIT;
    for (updated_at, path) in candidates.into_iter().take(24) {
        if remaining_bytes == 0 || remaining_lines == 0 {
            break;
        }
        let scan = parse_claude_cli_session_limited(
            &path,
            remaining_bytes,
            remaining_lines.min(TRANSCRIPT_FILE_LINE_LIMIT),
        );
        remaining_bytes = remaining_bytes.saturating_sub(scan.bytes_read);
        remaining_lines = remaining_lines.saturating_sub(scan.lines_read);
        // A transcript with no timestamped record of its own can only be dated
        // by the file itself.
        let started_at = scan.started_at.unwrap_or(updated_at);
        if started_at.saturating_add(MANAGED_SESSION_CLOCK_SKEW_SECONDS) < created_at {
            continue;
        }
        if let Some(session) = scan.session {
            let resumes_the_task = session
                .last_user_message
                .as_deref()
                .is_some_and(|message| message.trim() == task);
            matches.push((
                started_at.saturating_sub(created_at),
                resumes_the_task,
                session,
            ));
        }
    }
    // The launch task confirms a candidate but cannot be required: a terminal
    // that has been used since is no longer sitting on its first request.
    if matches
        .iter()
        .any(|(_, resumes_the_task, _)| *resumes_the_task)
    {
        matches.retain(|(_, resumes_the_task, _)| *resumes_the_task);
    }
    matches.sort_by_key(|(distance, _, _)| *distance);
    match matches.as_slice() {
        [] => Ok(None),
        [(_, _, session)] => Ok(Some(session.clone())),
        [(first_distance, _, session), (second_distance, _, _), ..]
            if first_distance < second_distance => Ok(Some(session.clone())),
        _ => Err(tr("この管理セッションに一致する Claude 履歴が複数あります。誤った会話を復元しないよう、中止します").into()),
    }
}

/// Operon records a managed session before its terminal starts writing, and
/// the two clocks are the same machine's, so only a small allowance is needed.
pub(crate) const MANAGED_SESSION_CLOCK_SKEW_SECONDS: u64 = 5;

/// Codex names its own conversation, so a managed launch is correlated by the
/// tracking token carried in its first prompt. A terminal started without a
/// goal sends no prompt at all, so that resolver can never match it and the
/// record keeps no conversation ID. The conversation itself is still on disk,
/// so recover it the same way a legacy Claude terminal is recovered: by when
/// the conversation *began*, which is the only property that ties a transcript
/// to the terminal that created it. File modification time cannot rank
/// candidates, because a live conversation keeps being written while an
/// abandoned older one stays frozen next to the terminal's start. A tie is
/// deliberately rejected rather than restoring the wrong conversation.
pub(crate) fn recover_managed_codex_session(
    workspace: &Path,
    created_at: u64,
) -> UiResult<Option<CliSession>> {
    let home = std::env::var_os("HOME").map(PathBuf::from).ok_or_else(|| {
        tr("ホームディレクトリを取得できないため、Codex の履歴を読めません。").to_owned()
    })?;
    recover_managed_codex_session_in(
        &home.join(".codex").join("sessions"),
        &home.join(".codex").join("session_index.jsonl"),
        workspace,
        created_at,
    )
}

pub(crate) fn recover_managed_codex_session_in(
    sessions_root: &Path,
    title_index: &Path,
    workspace: &Path,
    created_at: u64,
) -> UiResult<Option<CliSession>> {
    let limits = TranscriptSearchLimits::default();
    let titles = read_codex_session_titles(title_index, limits.bytes, limits.lines);
    let recent = recent_jsonl_files_from_roots(
        &[("Codex".to_owned(), sessions_root.to_path_buf(), 5)],
        limits.candidates,
        limits.visited_entries,
    );
    let allowed_paths = project_and_worktree_paths(workspace);
    let mut matches = Vec::new();
    let mut remaining_bytes = limits.bytes.saturating_sub(titles.scanned_bytes);
    let mut remaining_lines = limits.lines.saturating_sub(titles.scanned_lines);
    for (_, path) in recent.candidates {
        if remaining_bytes == 0 || remaining_lines == 0 {
            break;
        }
        // A transcript untouched since before this terminal existed cannot hold
        // its conversation. Dropping those first keeps the full parse below
        // bounded without ranking by a time that moves while a session is live.
        let updated_at = fs::metadata(&path)
            .and_then(|metadata| metadata.modified())
            .ok()
            .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs())
            .unwrap_or_default();
        if updated_at.saturating_add(MANAGED_SESSION_CLOCK_SKEW_SECONDS) < created_at {
            continue;
        }
        let scan = parse_codex_cli_session_limited(
            &path,
            &titles.titles,
            &allowed_paths,
            remaining_bytes,
            remaining_lines.min(TRANSCRIPT_FILE_LINE_LIMIT),
        );
        remaining_bytes = remaining_bytes.saturating_sub(scan.bytes_read);
        remaining_lines = remaining_lines.saturating_sub(scan.lines_read);
        let Some(session) = scan.session else {
            continue;
        };
        // A transcript with no timestamped record of its own can only be dated
        // by the file itself.
        let started_at = scan.started_at.unwrap_or(updated_at);
        if started_at.saturating_add(MANAGED_SESSION_CLOCK_SKEW_SECONDS) < created_at {
            continue;
        }
        matches.push((started_at.saturating_sub(created_at), session));
    }
    matches.sort_by_key(|(distance, _)| *distance);
    match matches.as_slice() {
        [] => Ok(None),
        [(_, session)] => Ok(Some(session.clone())),
        // A terminal that has existed for a while is normally outlived by later
        // conversations of the same project, so the nearest start wins. Only an
        // unresolvable tie is refused.
        [(first_distance, session), (second_distance, _), ..]
            if first_distance < second_distance =>
        {
            Ok(Some(session.clone()))
        }
        _ => Err(tr("この管理セッションに一致する Codex 履歴が複数あります。誤った会話を復元しないよう、中止します").into()),
    }
}

/// Antigravity names a conversation only once its first turn is sent, and a
/// prompt passed on the command line is never recorded, so a managed launch
/// leaves nothing to identify it with at launch time. What the launch does
/// leave from its first moment is a process log naming the workspace it opened,
/// and that log names every conversation the terminal attaches to. The terminal
/// is the `agy` process Operon started for it, which is the first one to
/// begin at or after the launch, and what is on its screen is the last
/// conversation it attached to.
///
/// Verified against `agy` 2026-08-20: launching writes no history entry at all,
/// the first typed turn writes one that names no conversation, and later turns
/// carry the ID — so a terminal whose conversation received a single turn is
/// absent from the history index entirely and exists only in the log. Across
/// this Mac's 686 stored logs, `Streaming conversation` is a strict superset of
/// `Created conversation` (46 logs only reopen conversations, none only create
/// them) and its last occurrence always names the same conversation the process
/// last switched to and redrew.
pub(crate) fn recover_managed_antigravity_session(
    workspace: &Path,
    launched_at: u64,
) -> UiResult<Option<CliSession>> {
    recover_managed_antigravity_session_in(
        &antigravity_history_index_path()?,
        workspace,
        launched_at,
    )
}

pub(crate) fn recover_managed_antigravity_session_in(
    history_path: &Path,
    workspace: &Path,
    launched_at: u64,
) -> UiResult<Option<CliSession>> {
    let store = history_path.parent().unwrap_or(Path::new("."));
    let allowed_paths = project_and_worktree_paths(workspace);
    if let Some(conversation) = antigravity_terminal_conversation(
        &store.join(ANTIGRAVITY_LOG_DIRECTORY_NAME),
        &allowed_paths,
        launched_at,
    ) {
        let transcript = antigravity_conversation_transcript_in(store, &conversation);
        // Without a transcript there is nothing to restore, and the log alone
        // cannot supply one.
        if let Some((updated_at, title)) = antigravity_transcript_opening(&transcript) {
            return Ok(Some(CliSession {
                provider: CliProvider::Gemini,
                native_id: conversation.clone(),
                id: conversation,
                title,
                last_user_message: None,
                branch: None,
                updated_at,
                path: transcript,
            }));
        }
    }
    // Logs `agy` has since rotated away leave only the history index, which
    // names a conversation from its second typed turn on. Date each candidate
    // by when its own transcript began: the index entry's time is the time of a
    // typed turn, which says nothing about when the conversation started, so an
    // older conversation the user returned to must not win.
    let index = read_antigravity_history_index(history_path, &allowed_paths);
    let mut matches = Vec::new();
    for session in antigravity_cli_sessions(store, history_path, index.conversations, 96) {
        if !is_antigravity_conversation_transcript(&session.path) {
            continue;
        }
        let Some(started_at) = antigravity_transcript_started_at(&session.path) else {
            continue;
        };
        if started_at.saturating_add(MANAGED_SESSION_CLOCK_SKEW_SECONDS) < launched_at {
            continue;
        }
        matches.push((started_at.saturating_sub(launched_at), session));
    }
    matches.sort_by_key(|(distance, _)| *distance);
    match matches.as_slice() {
        [] => Ok(None),
        [(_, session)] => Ok(Some(session.clone())),
        [(first_distance, session), (second_distance, _), ..]
            if first_distance < second_distance =>
        {
            Ok(Some(session.clone()))
        }
        _ => Err(tr("この管理セッションに一致する Antigravity の会話が複数あります。誤った会話を復元しないよう、中止します").into()),
    }
}

/// Read when an Antigravity conversation began and what it opened with, from
/// its transcript's own first record. The store keeps no creation time of its
/// own, and the directory's birth time is not portable across copies of the
/// store. The opening request is the only title a conversation the history
/// index never named can be listed under.
pub(crate) fn antigravity_transcript_opening(path: &Path) -> Option<(u64, Option<String>)> {
    let file = fs::File::open(path).ok()?;
    let mut reader = BufReader::new(file.take(TRANSCRIPT_FILE_MAX_BYTES));
    let mut line = String::new();
    for _ in 0..TRANSCRIPT_FILE_LINE_LIMIT {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) | Err(_) => return None,
            Ok(_) => {}
        }
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&line) else {
            continue;
        };
        let Some(started_at) = value
            .get("created_at")
            .and_then(|value| value.as_str())
            .and_then(parse_rfc3339_utc_seconds)
        else {
            continue;
        };
        let title = transcript_value_turn(&value, CliProvider::Gemini)
            .filter(|(role, _)| *role == "user")
            .map(|(_, request)| truncate_chars(&request, ANTIGRAVITY_TRANSCRIPT_TITLE_MAX_CHARS));
        return Some((started_at, title));
    }
    None
}

pub(crate) fn antigravity_transcript_started_at(path: &Path) -> Option<u64> {
    antigravity_transcript_opening(path).map(|(started_at, _)| started_at)
}

/// One entry of Claude Code's session registry: the conversation a CLI process
/// is on right now, the workspace it opened, and when the process started.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ClaudeRegisteredSession {
    pub(crate) session_id: String,
    pub(crate) workspace: PathBuf,
    pub(crate) started_at: u64,
}

/// Read every entry Claude Code has published. An unreadable or absent registry
/// is silence rather than an error: a machine running a Claude Code that
/// predates the registry must keep the behaviour it had, not gain a failure.
pub(crate) fn read_claude_session_registry_in(
    registry_root: &Path,
) -> Vec<ClaudeRegisteredSession> {
    let Ok(entries) = fs::read_dir(registry_root) else {
        return Vec::new();
    };
    let mut sessions = Vec::new();
    for entry in entries.flatten().take(CLAUDE_SESSION_REGISTRY_VISIT_LIMIT) {
        let path = entry.path();
        if path.extension().is_none_or(|extension| extension != "json") {
            continue;
        }
        if entry
            .metadata()
            .is_ok_and(|metadata| metadata.len() > CLAUDE_SESSION_REGISTRY_FILE_MAX_BYTES)
        {
            continue;
        }
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
            continue;
        };
        let Some(session_id) = value
            .get("sessionId")
            .and_then(|value| value.as_str())
            .filter(|id| is_safe_cli_session_id(id))
        else {
            continue;
        };
        let Some(workspace) = value.get("cwd").and_then(|value| value.as_str()) else {
            continue;
        };
        // Claude Code writes the process start as epoch milliseconds; every
        // other time in this module is seconds.
        let Some(started_at) = value
            .get("startedAt")
            .and_then(|value| value.as_u64())
            .map(|millis| millis / 1_000)
        else {
            continue;
        };
        sessions.push(ClaudeRegisteredSession {
            session_id: session_id.to_owned(),
            workspace: PathBuf::from(workspace),
            started_at,
        });
    }
    sessions
}

/// What Claude Code's registry has to say about a managed terminal.
///
/// `Unknown` and `Named` are deliberately different answers, for the reason
/// `TRANSCRIPT_VOCABULARY` separates "excluded on purpose" from "never seen":
/// a registry that names the stored conversation has *positively confirmed* the
/// stored identity, and no weaker inference may then overrule it. A registry
/// with nothing to say leaves the older checks to run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RegisteredConversation {
    Unknown,
    Named(String),
}

/// The conversation Claude Code says this terminal is on.
///
/// This cannot be inferred from the transcripts. `/clear` starts a new
/// conversation in the same terminal, and both the abandoned one and the live
/// one begin *after* the terminal was created — the abandoned one closest to it,
/// so every ranking by start time picks the wrong file, and file modification
/// time picks the wrong file the other way. The registry is rewritten when a
/// terminal rotates, so it is the only local record that is not a guess. On the
/// machine this was written for, 13 of 30 live entries named a conversation that
/// began after the process did, by as much as thirteen hours, and two named one
/// that began before it.
///
/// The correlation is by workspace and by when the CLI process started, against
/// the same allowance already used to date a transcript against its terminal.
/// A workspace is identified by `encode_claude_project_path`, the normalization
/// the transcript directory itself is keyed by, so the registry and the
/// transcripts cannot disagree about which workspace an entry belongs to — and
/// because it canonicalizes, a workspace reached through a symlink still
/// matches the path Claude Code resolved.
pub(crate) fn registered_claude_conversation_in(
    registry_root: &Path,
    workspace: &Path,
    launched_at: u64,
    recorded_id: Option<&str>,
) -> UiResult<RegisteredConversation> {
    let encoded = encode_claude_project_path(workspace);
    let mut named: Vec<String> = read_claude_session_registry_in(registry_root)
        .into_iter()
        .filter(|entry| encode_claude_project_path(&entry.workspace) == encoded)
        .filter(|entry| {
            entry.started_at.abs_diff(launched_at) <= MANAGED_SESSION_CLOCK_SKEW_SECONDS
        })
        .map(|entry| entry.session_id)
        .collect();
    named.sort();
    named.dedup();
    match named.len() {
        0 => Ok(RegisteredConversation::Unknown),
        1 => Ok(RegisteredConversation::Named(named.remove(0))),
        // Two terminals were opened in this workspace at the same moment. If one
        // of them is still on the conversation this record was launched with,
        // that entry is this terminal and the other belongs to its neighbour:
        // a terminal that has not rotated goes on publishing its launch ID. That
        // is evidence, not a tie-break, and without it two terminals started
        // together would both become unresumable.
        _ => match recorded_id.filter(|id| named.iter().any(|named| named == id)) {
            Some(recorded_id) => Ok(RegisteredConversation::Named(recorded_id.to_owned())),
            // Both have moved on. Choosing between them is the mistake this
            // whole function exists to stop.
            None => Err(tr("このワークスペースで同時に開始された Claude ターミナルが複数あり、どの会話がこのターミナルのものか判別できません。誤った会話を復元しないよう、中止します。").into()),
        },
    }
}

/// Correct a stored Claude identity that is not the conversation this terminal
/// is in, and return `None` when the stored one is still right.
///
/// The stored identity is the `--session-id` the terminal was launched with, and
/// a terminal does not stay on one conversation: `/clear` moves it to a new one,
/// under a new ID, in a new file, and the stored ID goes on naming the first.
/// So the terminal's own registry entry is consulted first and outranks
/// everything else, including for a terminal launched as a resume — a resumed
/// terminal rotates too. Only when the registry has nothing to say does the
/// older check apply: that a transcript which began before the terminal existed
/// cannot be its.
pub(crate) fn corrected_managed_claude_conversation(
    session: &Session,
    workspace: &Path,
    recorded_transcript: Option<&Path>,
) -> UiResult<Option<CliSession>> {
    let (Some(registry_root), Some(projects_root)) =
        (claude_session_registry_root(), claude_projects_root())
    else {
        return Ok(None);
    };
    corrected_managed_claude_conversation_in(
        &registry_root,
        &projects_root,
        session,
        workspace,
        recorded_transcript,
    )
}

/// `corrected_managed_claude_conversation` rooted at the two directories rather
/// than at `HOME`, so that a guard can build one and exercise the whole
/// decision — which evidence is consulted, and in which order — rather than the
/// pieces it is assembled from. Testing the pieces is how the rename in lesson
/// 004 passed a green suite: the check was covered and its caller was not.
pub(crate) fn corrected_managed_claude_conversation_in(
    registry_root: &Path,
    projects_root: &Path,
    session: &Session,
    workspace: &Path,
    recorded_transcript: Option<&Path>,
) -> UiResult<Option<CliSession>> {
    if cli_provider_for_agent(&session.agent) != Some(CliProvider::Claude) {
        return Ok(None);
    }
    let recorded_id = session.native_session_id.as_deref();
    // A record written before the launch time was kept can only be dated by
    // when it was created, which is within the same allowance of the launch.
    let launched_at = session.launched_at.unwrap_or(session.created_at);
    match registered_claude_conversation_in(registry_root, workspace, launched_at, recorded_id)? {
        // The registry named the conversation this record already holds. That
        // is a positive confirmation of the stored identity, so the weaker
        // checks below do not get to overrule it: a terminal moved onto an
        // older conversation from inside Claude Code has a stored ID that
        // began before the terminal did, and the start-time check would send a
        // confirmed identity off to be "recovered".
        RegisteredConversation::Named(registered) if Some(registered.as_str()) == recorded_id => {
            return Ok(None);
        }
        RegisteredConversation::Named(registered) => {
            let Some(path) = claude_transcript_path_in(projects_root, workspace, &registered)
            else {
                return Err(tr("このターミナルは新しい会話を開始したばかりで、まだ会話ログが書かれていません。1 度発言してから、もう一度お試しください。").into());
            };
            return Ok(Some(claude_cli_session_at(&path)));
        }
        // Claude Code before the registry existed, or a terminal it has no
        // entry for. Everything below is the behaviour such a machine had.
        RegisteredConversation::Unknown => {}
    }
    // A terminal launched as a resume legitimately reopens a conversation that
    // began long before its own record, so the start-time check below cannot
    // judge it. The registry above can, and already has.
    if is_native_resume_command(&session.agent_command) {
        return Ok(None);
    }
    let Some(recorded_transcript) = recorded_transcript else {
        return Ok(None);
    };
    if claude_transcript_can_belong_to_managed_session(recorded_transcript, session.created_at) {
        return Ok(None);
    }
    match recover_managed_claude_session_in(
        projects_root,
        workspace,
        session.created_at,
        &session.goal,
    )? {
        Some(recovered) => Ok(Some(recovered)),
        None => Err(
            tr("このターミナルに記録されていた Claude 会話は、ターミナルが作られる前に終わった別の会話でした。実際の会話ログを特定できません。Discover local CLI sessions から該当する会話を選んでください。")
                .into(),
        ),
    }
}

/// Describe one Claude transcript as the session a restore consumes. A file the
/// terminal has only just opened holds no turn to read a title or a last request
/// from, and is still the conversation that terminal is in.
fn claude_cli_session_at(path: &Path) -> CliSession {
    let scan = parse_claude_cli_session_limited(
        path,
        TRANSCRIPT_FILE_MAX_BYTES,
        TRANSCRIPT_FILE_LINE_LIMIT,
    );
    let native_id = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or_default()
        .to_owned();
    scan.session.unwrap_or_else(|| CliSession {
        provider: CliProvider::Claude,
        native_id: native_id.clone(),
        id: native_id,
        title: None,
        last_user_message: None,
        branch: None,
        updated_at: fs::metadata(path)
            .and_then(|metadata| metadata.modified())
            .ok()
            .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs())
            .unwrap_or_default(),
        path: path.to_path_buf(),
    })
}

/// Read when a Claude conversation began, from the transcript's own records.
pub(crate) fn claude_transcript_started_at(path: &Path) -> Option<u64> {
    parse_claude_cli_session_limited(path, TRANSCRIPT_FILE_MAX_BYTES, TRANSCRIPT_FILE_LINE_LIMIT)
        .started_at
}

/// A conversation that began before its terminal existed cannot be the
/// conversation that terminal is showing. Older builds matched managed Claude
/// terminals to transcripts by file modification time, which reliably picked an
/// abandoned older conversation over the live one, so a stored identity has to
/// be re-checked before it is used to restore anything.
pub(crate) fn claude_transcript_can_belong_to_managed_session(
    path: &Path,
    created_at: u64,
) -> bool {
    match claude_transcript_started_at(path) {
        Some(started_at) => {
            started_at.saturating_add(MANAGED_SESSION_CLOCK_SKEW_SECONDS) >= created_at
        }
        // Nothing in the transcript contradicts the stored identity.
        None => true,
    }
}

pub(crate) fn shared_session_archive_directory(store_path: &Path) -> PathBuf {
    store_path
        .parent()
        .unwrap_or_else(|| Path::new(".operon"))
        .join("shared-sessions")
}

/// The archive holds whole transcripts — prompts, tool output, source code — so
/// the owner is the only account that may read them. `create_dir_all` applies
/// the process umask, which on a stock Mac leaves `0o755`: every other local
/// account can walk in. Narrowing the directory is what protects the files
/// inside it, including `manifest.json` and `restored-conversation.md`, which
/// other modules write here through `write_file_atomically`.
pub(crate) const SHARED_SESSION_DIRECTORY_MODE: u32 = 0o700;
pub(crate) const SHARED_SESSION_FILE_MODE: u32 = 0o600;

fn create_private_directory(directory: &Path) -> std::io::Result<()> {
    fs::create_dir_all(directory)?;
    fs::set_permissions(
        directory,
        fs::Permissions::from_mode(SHARED_SESSION_DIRECTORY_MODE),
    )
}

/// Copy a provider transcript before importing it. The copy is the durable,
/// provider-neutral source of truth; provider session formats may change or be
/// cleaned up independently after the destination thread is created.
pub(crate) fn snapshot_native_transcript(
    source: &Path,
    archive_root: &Path,
    import_id: Uuid,
) -> Result<PathBuf> {
    let source_file = fs::File::open(source)
        .with_context(|| tf!("元履歴 {p0} を開く処理", p0 = source.display()))?;
    let import_directory = archive_root.join(import_id.to_string());
    // The root is narrowed too: an archive created by an earlier build already
    // exists at the umask default, and a per-import directory inside a
    // world-traversable root is still reachable by path.
    create_private_directory(archive_root).with_context(|| {
        tf!(
            "共有セッションディレクトリ {p0} の作成",
            p0 = archive_root.display()
        )
    })?;
    create_private_directory(&import_directory).with_context(|| {
        tf!(
            "共有セッションディレクトリ {p0} の作成",
            p0 = import_directory.display()
        )
    })?;
    let destination = import_directory.join("source.jsonl");
    let temporary = import_directory.join(format!(".source.jsonl.tmp-{}", Uuid::new_v4()));
    let mut destination_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(SHARED_SESSION_FILE_MODE)
        .open(&temporary)
        .with_context(|| tf!("履歴スナップショット {p0} の作成", p0 = temporary.display()))?;
    let mut source_reader = BufReader::new(source_file);
    std::io::copy(&mut source_reader, &mut destination_file)
        .with_context(|| tf!("元履歴 {p0} の複製", p0 = source.display()))?;
    destination_file
        .sync_all()
        .with_context(|| tf!("履歴スナップショット {p0} の同期", p0 = temporary.display()))?;
    drop(destination_file);
    fs::rename(&temporary, &destination).with_context(|| {
        tf!(
            "履歴スナップショット {p0} の確定",
            p0 = destination.display()
        )
    })?;
    fs::File::open(&import_directory)
        .and_then(|directory| directory.sync_all())
        .with_context(|| {
            tf!(
                "共有セッションディレクトリ {p0} の同期",
                p0 = import_directory.display()
            )
        })?;
    Ok(destination)
}

pub(crate) fn scan_codex_cli_sessions_for_resolution(project: &Path) -> CliSessionScan {
    let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else {
        return CliSessionScan::default();
    };
    scan_codex_cli_sessions_in_with_limits(
        &home.join(".codex").join("sessions"),
        &home.join(".codex").join("session_index.jsonl"),
        project,
        96,
        TranscriptSearchLimits {
            visited_entries: NATIVE_RESOLUTION_SCAN_VISITED_ENTRIES,
            candidates: 96,
            bytes: NATIVE_RESOLUTION_SCAN_BYTES,
            lines: NATIVE_RESOLUTION_SCAN_LINES,
        },
    )
}

pub(crate) fn project_and_worktree_paths(project: &Path) -> HashSet<PathBuf> {
    let mut paths = HashSet::from([project
        .canonicalize()
        .unwrap_or_else(|_| project.to_path_buf())]);
    if let Ok(worktrees) = list_worktrees(project) {
        paths.extend(
            worktrees
                .into_iter()
                .map(|worktree| worktree.path.canonicalize().unwrap_or(worktree.path)),
        );
    }
    paths
}

pub(crate) fn scan_codex_cli_sessions_in(
    sessions_root: &Path,
    title_index: &Path,
    project: &Path,
    limit: usize,
) -> CliSessionScan {
    scan_codex_cli_sessions_in_with_limits(
        sessions_root,
        title_index,
        project,
        limit,
        TranscriptSearchLimits::default(),
    )
}

pub(crate) fn scan_codex_cli_sessions_in_with_limits(
    sessions_root: &Path,
    title_index: &Path,
    project: &Path,
    limit: usize,
    limits: TranscriptSearchLimits,
) -> CliSessionScan {
    let titles = read_codex_session_titles(title_index, limits.bytes, limits.lines);
    let recent = recent_jsonl_files_from_roots(
        &[("Codex".to_owned(), sessions_root.to_path_buf(), 5)],
        limits.candidates,
        limits.visited_entries,
    );
    let allowed_paths = project_and_worktree_paths(project);
    let mut sessions = Vec::new();
    let mut truncated = titles.truncated || recent.truncated;
    let mut unreadable_entries = recent.unreadable_entries + usize::from(titles.unreadable);
    let mut remaining_bytes = limits.bytes.saturating_sub(titles.scanned_bytes);
    let mut remaining_lines = limits.lines.saturating_sub(titles.scanned_lines);
    for (_, path) in recent.candidates {
        if remaining_bytes == 0 || remaining_lines == 0 {
            truncated = true;
            break;
        }
        let scan = parse_codex_cli_session_limited(
            &path,
            &titles.titles,
            &allowed_paths,
            remaining_bytes,
            remaining_lines.min(TRANSCRIPT_FILE_LINE_LIMIT),
        );
        remaining_bytes = remaining_bytes.saturating_sub(scan.bytes_read);
        remaining_lines = remaining_lines.saturating_sub(scan.lines_read);
        truncated |= scan.truncated;
        unreadable_entries += usize::from(scan.unreadable);
        if let Some(session) = scan.session {
            sessions.push(session);
        }
    }
    sessions.sort_by_key(|session| Reverse(session.updated_at));
    if sessions.len() > limit {
        truncated = true;
    }
    sessions.truncate(limit);
    CliSessionScan {
        sessions,
        truncated,
        unreadable_entries,
    }
}

pub(crate) fn parse_codex_cli_session_limited(
    path: &Path,
    titles: &HashMap<String, String>,
    allowed_paths: &HashSet<PathBuf>,
    max_bytes: u64,
    max_lines: usize,
) -> CliSessionFileScan {
    let file = match fs::File::open(path) {
        Ok(file) => file,
        Err(_) => {
            return CliSessionFileScan {
                unreadable: true,
                ..CliSessionFileScan::default()
            }
        }
    };
    let metadata = match file.metadata() {
        Ok(metadata) => metadata,
        Err(_) => {
            return CliSessionFileScan {
                unreadable: true,
                ..CliSessionFileScan::default()
            }
        }
    };
    if metadata.len() > TRANSCRIPT_FILE_MAX_BYTES || metadata.len() > max_bytes {
        return CliSessionFileScan {
            truncated: true,
            ..CliSessionFileScan::default()
        };
    }
    let read_limit = metadata.len();
    let mut reader = BufReader::new(file.take(read_limit));
    let mut line = String::new();
    let mut session_id = None;
    let mut cwd = None;
    let mut last_user_message = None;
    let mut started_at = None;
    let mut lines_read = 0;
    let mut reached_eof = false;
    let mut unreadable = false;
    while lines_read < max_lines {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => {
                reached_eof = true;
                break;
            }
            Ok(_) => lines_read += 1,
            Err(_) => {
                unreadable = true;
                break;
            }
        }
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&line) else {
            continue;
        };
        if started_at.is_none() {
            started_at = value
                .get("timestamp")
                .and_then(|value| value.as_str())
                .and_then(parse_rfc3339_utc_seconds);
        }
        let payload = value.get("payload");
        if value.get("type").and_then(|value| value.as_str()) == Some("session_meta") {
            session_id = payload
                .and_then(|payload| payload.get("session_id").or_else(|| payload.get("id")))
                .and_then(|value| value.as_str())
                .map(ToOwned::to_owned)
                .or(session_id);
            cwd = payload
                .and_then(|payload| payload.get("cwd"))
                .and_then(|value| value.as_str())
                .map(PathBuf::from)
                .or(cwd);
        }
        if value.get("type").and_then(|value| value.as_str()) == Some("event_msg") {
            if let Some(goal) = payload
                .and_then(|payload| payload.get("goal"))
                .and_then(|value| value.as_str())
                .map(ToOwned::to_owned)
            {
                last_user_message = Some(goal);
            }
        }
        if let Some(message) = codex_user_message_text(&value) {
            last_user_message = Some(message);
        }
    }
    let bytes_read = read_limit.saturating_sub(reader.get_ref().limit());
    let grew_while_reading = reader
        .get_ref()
        .get_ref()
        .metadata()
        .map(|latest| latest.len() > read_limit)
        .unwrap_or(true);
    let session = session_id.and_then(|id| {
        let cwd = cwd?;
        let canonical_cwd = cwd.canonicalize().unwrap_or(cwd);
        allowed_paths.contains(&canonical_cwd).then(|| CliSession {
            title: titles.get(&id).cloned(),
            provider: CliProvider::Codex,
            native_id: id.clone(),
            id,
            last_user_message,
            branch: git_branch(&canonical_cwd).ok(),
            updated_at: metadata
                .modified()
                .ok()
                .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                .map(|duration| duration.as_secs())
                .unwrap_or_default(),
            path: path.to_path_buf(),
        })
    });
    CliSessionFileScan {
        session,
        started_at,
        bytes_read,
        lines_read,
        truncated: !reached_eof || grew_while_reading,
        unreadable,
    }
}

pub(crate) fn codex_user_message_text(value: &serde_json::Value) -> Option<String> {
    let payload = value.get("payload")?;
    if value.get("type")?.as_str()? != "response_item"
        || payload.get("type")?.as_str()? != "message"
        || payload.get("role")?.as_str()? != "user"
    {
        return None;
    }
    payload
        .get("content")?
        .as_array()?
        .iter()
        .filter(|part| {
            matches!(
                part.get("type").and_then(|value| value.as_str()),
                Some("input_text" | "text")
            )
        })
        .filter_map(|part| part.get("text").and_then(|value| value.as_str()))
        .map(ToOwned::to_owned)
        .reduce(|mut left, right| {
            left.push(' ');
            left.push_str(&right);
            left
        })
}

/// Conversations of one workspace, keyed by ID, each holding the newest time it
/// was seen at and the title to list it under.
/// The index holds one line per submitted turn, not per conversation, so point
/// each session at the transcript `agy` writes for that conversation whenever it
/// exists. A conversation discovered before its first transcript flush keeps the
/// index as its source location.
pub(crate) fn antigravity_cli_sessions(
    store: &Path,
    history_path: &Path,
    conversations: AntigravityConversations,
    limit: usize,
) -> Vec<CliSession> {
    let mut candidates = conversations.into_iter().collect::<Vec<_>>();
    // Conversations arrive from a map, so break ties by ID rather than let the
    // list reorder itself between scans.
    candidates.sort_by(|left, right| {
        Reverse(left.1 .0)
            .cmp(&Reverse(right.1 .0))
            .then_with(|| left.0.cmp(&right.0))
    });
    candidates
        .into_iter()
        .take(limit)
        .map(|(conversation_id, (updated_at, title))| {
            let transcript = antigravity_conversation_transcript_in(store, &conversation_id);
            CliSession {
                provider: CliProvider::Gemini,
                native_id: conversation_id.clone(),
                id: conversation_id.clone(),
                title: title.or(Some(tf!(
                    "Antigravity の会話 {conversation_id}",
                    conversation_id = conversation_id
                ))),
                last_user_message: None,
                branch: None,
                updated_at,
                path: if transcript.is_file() {
                    transcript
                } else {
                    history_path.to_path_buf()
                },
            }
        })
        .collect()
}

/// Read the conversations `agy`'s shared history index attributes to this
/// workspace. Every entry is one submitted turn, and only turns from the second
/// one on carry the conversation they belong to.
pub(crate) fn read_antigravity_history_index(
    history_path: &Path,
    allowed_paths: &HashSet<PathBuf>,
) -> AntigravityIndexScan {
    let file = match fs::File::open(history_path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return AntigravityIndexScan::default()
        }
        Err(_) => {
            return AntigravityIndexScan {
                unreadable_entries: 1,
                ..AntigravityIndexScan::default()
            }
        }
    };
    let metadata = match file.metadata() {
        Ok(metadata) => metadata,
        Err(_) => {
            return AntigravityIndexScan {
                unreadable_entries: 1,
                ..AntigravityIndexScan::default()
            }
        }
    };
    if metadata.len() > TRANSCRIPT_FILE_MAX_BYTES {
        return AntigravityIndexScan {
            truncated: true,
            ..AntigravityIndexScan::default()
        };
    }
    let mut scan = AntigravityIndexScan::default();
    let mut reader = BufReader::new(file);
    let mut line = String::new();
    for line_number in 0..TRANSCRIPT_FILE_LINE_LIMIT {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {}
            Err(_) => {
                scan.unreadable_entries += 1;
                break;
            }
        }
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&line) else {
            continue;
        };
        let Some(workspace) = value.get("workspace").and_then(|value| value.as_str()) else {
            continue;
        };
        let workspace = PathBuf::from(workspace);
        if !allowed_paths.contains(&workspace.canonicalize().unwrap_or(workspace)) {
            continue;
        }
        let timestamp = value
            .get("timestamp")
            .and_then(|value| value.as_u64())
            .unwrap_or_default();
        let Some(conversation_id) = value
            .get("conversationId")
            .and_then(|value| value.as_str())
            .map(ToOwned::to_owned)
        else {
            continue;
        };
        let timestamp = if timestamp > 100_000_000_000 {
            timestamp / 1_000
        } else {
            timestamp
        };
        let title = value
            .get("display")
            .and_then(|value| value.as_str())
            .map(ToOwned::to_owned);
        let replace = scan
            .conversations
            .get(&conversation_id)
            .is_none_or(|(previous, _)| timestamp >= *previous);
        if replace {
            scan.conversations
                .insert(conversation_id, (timestamp, title));
        }
        if line_number + 1 == TRANSCRIPT_FILE_LINE_LIMIT {
            scan.truncated = true;
        }
    }
    scan
}

/// List `agy`'s per-process logs, newest process first, with the moment each
/// process started.
///
/// That moment is the file's own creation time. `agy` names the file after the
/// local wall clock, which cannot be compared with a stored timestamp without
/// knowing the timezone, while the creation time is the same instant already in
/// epoch seconds; verified across this Mac's 686 logs, the two never differ by
/// more than a second. A log whose creation time the filesystem does not keep is
/// dropped rather than guessed at.
pub(crate) fn antigravity_process_logs(
    log_directory: &Path,
    scan: &mut AntigravityLogScan,
) -> Vec<(u64, PathBuf)> {
    let entries = match fs::read_dir(log_directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Vec::new(),
        Err(_) => {
            scan.unreadable_entries += 1;
            return Vec::new();
        }
    };
    let mut logs = Vec::new();
    for (visited, entry) in entries.enumerate() {
        if visited >= ANTIGRAVITY_LOG_VISIT_LIMIT {
            scan.truncated = true;
            break;
        }
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => {
                scan.unreadable_entries += 1;
                continue;
            }
        };
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        if !name.starts_with(ANTIGRAVITY_LOG_FILE_PREFIX)
            || !name.ends_with(ANTIGRAVITY_LOG_FILE_SUFFIX)
        {
            continue;
        }
        let metadata = match entry.metadata() {
            Ok(metadata) => metadata,
            Err(_) => {
                scan.unreadable_entries += 1;
                continue;
            }
        };
        // `agy` keeps a `cli.log` symlink to the log of its newest process;
        // following it would read that log a second time.
        if metadata.is_symlink() || !metadata.is_file() {
            continue;
        }
        let Some(started_at) = metadata
            .created()
            .ok()
            .and_then(|created| created.duration_since(UNIX_EPOCH).ok())
            .map(|since_epoch| since_epoch.as_secs())
        else {
            scan.unreadable_entries += 1;
            continue;
        };
        logs.push((started_at, entry.path()));
    }
    logs.sort_by(|left, right| right.cmp(left));
    logs
}

/// Collect every conversation this workspace's `agy` processes attached to,
/// from their own logs.
///
/// The history index cannot answer this alone: `agy` appends a conversation's
/// first turn to it before the conversation has an ID, so a conversation that
/// never received a second turn leaves only an entry naming no conversation and
/// is invisible to discovery. The process log is not: the CLI server names the
/// workspace it opened as it starts, and names every conversation it attaches
/// to afterwards.
pub(crate) fn scan_antigravity_log_conversations(
    log_directory: &Path,
    allowed_paths: &HashSet<PathBuf>,
) -> AntigravityLogScan {
    let mut scan = AntigravityLogScan::default();
    let mut budget = ANTIGRAVITY_LOG_SCAN_BYTE_LIMIT;
    let mut seen = HashSet::new();
    for (_, log) in antigravity_process_logs(log_directory, &mut scan) {
        if budget == 0 {
            scan.truncated = true;
            break;
        }
        let Some(read) = read_antigravity_log_conversations(&log, allowed_paths, &mut budget)
        else {
            scan.unreadable_entries += 1;
            continue;
        };
        scan.truncated |= read.truncated;
        for conversation_id in read.conversation_ids {
            if seen.insert(conversation_id.clone()) {
                scan.conversation_ids.push(conversation_id);
            }
        }
    }
    scan
}

/// Identify the conversation on the screen of the managed terminal launched at
/// `launched_at`, from the log of the `agy` process Operon started for it.
///
/// That process is the first one to open this workspace at or after the launch,
/// and what its terminal shows is the last conversation it attached to: `/clear`
/// leaves the earlier conversation behind. A launch that failed before creating
/// one is skipped rather than treated as the answer, which is what lets a retry
/// resolve to the conversation of its own attempt.
pub(crate) fn antigravity_terminal_conversation(
    log_directory: &Path,
    allowed_paths: &HashSet<PathBuf>,
    launched_at: u64,
) -> Option<String> {
    let mut scan = AntigravityLogScan::default();
    let mut logs = antigravity_process_logs(log_directory, &mut scan);
    logs.retain(|(started_at, _)| {
        started_at.saturating_add(MANAGED_SESSION_CLOCK_SKEW_SECONDS) >= launched_at
    });
    logs.reverse();
    let mut budget = ANTIGRAVITY_LOG_SCAN_BYTE_LIMIT;
    for (_, log) in logs {
        if budget == 0 {
            return None;
        }
        let Some(read) = read_antigravity_log_conversations(&log, allowed_paths, &mut budget)
        else {
            continue;
        };
        if let Some(conversation_id) = read.conversation_ids.last() {
            return Some(conversation_id.clone());
        }
    }
    None
}

/// Read one `agy` process log, and only as far as this scan needs it: a log of
/// another workspace is dropped at the line that names the workspace, and a log
/// that names none within its header is not an `agy` process log at all.
/// Returns `None` when the log cannot be opened.
pub(crate) fn read_antigravity_log_conversations(
    path: &Path,
    allowed_paths: &HashSet<PathBuf>,
    budget: &mut u64,
) -> Option<AntigravityLogScan> {
    let file = fs::File::open(path).ok()?;
    let readable = (*budget).min(TRANSCRIPT_FILE_MAX_BYTES);
    let mut reader = BufReader::new(file.take(readable));
    let mut line = String::new();
    let mut scan = AntigravityLogScan::default();
    let mut workspace_matched = false;
    let mut read_bytes = 0;
    for line_number in 0..TRANSCRIPT_FILE_LINE_LIMIT {
        line.clear();
        // A log line that is not UTF-8 leaves the reader at an unknown offset;
        // the workspace header and the conversation lines are plain ASCII, so
        // stopping there costs nothing this scan needs.
        let Ok(count) = reader.read_line(&mut line) else {
            break;
        };
        if count == 0 {
            break;
        }
        read_bytes += count as u64;
        if !workspace_matched {
            if let Some(workspaces) = antigravity_log_workspaces(&line) {
                if !antigravity_log_workspace_matches(workspaces, allowed_paths) {
                    break;
                }
                workspace_matched = true;
                continue;
            }
            if line_number + 1 >= ANTIGRAVITY_LOG_HEAD_LINE_LIMIT {
                break;
            }
            continue;
        }
        if let Some(conversation_id) = antigravity_log_streamed_conversation(&line) {
            scan.conversation_ids.push(conversation_id.to_owned());
        }
        // A log of this workspace that outruns either budget may still name
        // conversations further on, so say so rather than present a partial
        // list as the whole one.
        if read_bytes >= readable || line_number + 1 == TRANSCRIPT_FILE_LINE_LIMIT {
            scan.truncated = true;
        }
    }
    *budget = budget.saturating_sub(read_bytes);
    Some(scan)
}

pub(crate) fn antigravity_log_workspaces(line: &str) -> Option<&str> {
    line.split_once("workspaceDirs=[")?
        .1
        .split_once(']')
        .map(|(workspaces, _)| workspaces)
}

/// `agy` logs its workspace list the way Go prints a slice, so the bracketed
/// value is either one path or several separated by spaces. A single path may
/// itself contain spaces, so accept the whole value as one path as well as each
/// space-separated part of it.
pub(crate) fn antigravity_log_workspace_matches(
    workspaces: &str,
    allowed_paths: &HashSet<PathBuf>,
) -> bool {
    std::iter::once(workspaces)
        .chain(workspaces.split(' '))
        .any(|candidate| {
            let candidate = PathBuf::from(candidate);
            allowed_paths.contains(&candidate.canonicalize().unwrap_or(candidate))
        })
}

/// `agy` logs this line every time its terminal attaches to a conversation, so
/// it names both the conversations a process creates and the ones it reopens.
/// Verified across this Mac's 686 logs: every log that names a created
/// conversation also streams it, 46 logs only stream conversations they did not
/// create, and the last streamed conversation is always the one the process
/// last switched to.
pub(crate) fn antigravity_log_streamed_conversation(line: &str) -> Option<&str> {
    let conversation_id = line
        .split_once("Streaming conversation ")?
        .1
        .split_whitespace()
        .next()?;
    is_safe_cli_session_id(conversation_id).then_some(conversation_id)
}

pub(crate) fn scan_claude_cli_sessions_in(
    root: &Path,
    project: &Path,
    limit: usize,
) -> CliSessionScan {
    scan_claude_cli_sessions_in_with_limits(root, project, limit, TranscriptSearchLimits::default())
}

pub(crate) fn scan_claude_cli_sessions_in_with_limits(
    root: &Path,
    project: &Path,
    limit: usize,
    limits: TranscriptSearchLimits,
) -> CliSessionScan {
    let encoded = encode_claude_project_path(project);
    let directory = root.join(encoded);
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return CliSessionScan::default();
        }
        Err(_) => {
            return CliSessionScan {
                unreadable_entries: 1,
                ..CliSessionScan::default()
            };
        }
    };
    let candidate_limit = limits.candidates;
    let mut newest = BinaryHeap::<Reverse<(SystemTime, PathBuf)>>::new();
    let mut truncated = false;
    let mut unreadable_entries = 0;
    for (visited, entry) in entries.enumerate() {
        if visited >= limits.visited_entries {
            truncated = true;
            break;
        }
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => {
                unreadable_entries += 1;
                continue;
            }
        };
        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(_) => {
                unreadable_entries += 1;
                continue;
            }
        };
        if file_type.is_symlink() || !file_type.is_file() {
            continue;
        }
        let path = entry.path();
        if path
            .extension()
            .is_none_or(|extension| extension != "jsonl")
            || path
                .file_stem()
                .is_some_and(|stem| stem.to_string_lossy().starts_with("agent-"))
        {
            continue;
        }
        let modified = match entry.metadata().and_then(|metadata| metadata.modified()) {
            Ok(modified) => modified,
            Err(_) => {
                unreadable_entries += 1;
                continue;
            }
        };
        if candidate_limit == 0 {
            truncated = true;
            continue;
        }
        newest.push(Reverse((modified, path)));
        if newest.len() > candidate_limit {
            newest.pop();
            truncated = true;
        }
    }
    let mut candidates = newest
        .into_iter()
        .map(|Reverse((modified, path))| (modified, path))
        .collect::<Vec<_>>();
    candidates.sort_by_key(|(modified, _)| Reverse(*modified));
    let mut sessions = Vec::new();
    let mut remaining_bytes = limits.bytes;
    let mut remaining_lines = limits.lines;
    for (_, path) in candidates {
        if remaining_bytes == 0 || remaining_lines == 0 {
            truncated = true;
            break;
        }
        let file_scan = parse_claude_cli_session_limited(
            &path,
            remaining_bytes,
            remaining_lines.min(TRANSCRIPT_FILE_LINE_LIMIT),
        );
        remaining_bytes = remaining_bytes.saturating_sub(file_scan.bytes_read);
        remaining_lines = remaining_lines.saturating_sub(file_scan.lines_read);
        truncated |= file_scan.truncated;
        if file_scan.unreadable {
            unreadable_entries += 1;
        }
        if let Some(session) = file_scan.session {
            sessions.push(session);
        }
    }
    sessions.sort_by_key(|session| std::cmp::Reverse(session.updated_at));
    if sessions.len() > limit {
        truncated = true;
    }
    sessions.truncate(limit);
    CliSessionScan {
        sessions,
        truncated,
        unreadable_entries,
    }
}

/// Claude Code files a conversation under the working directory its process
/// actually resolved, so a project reached through a symlink belongs to the
/// resolved path. Writing or looking for the conversation under the unresolved
/// path silently misses it; an unresolvable path is used as given, which is
/// what discovery did before and what tests exercise.
pub(crate) fn claude_conversation_workspace(project: &Path) -> PathBuf {
    fs::canonicalize(project).unwrap_or_else(|_| project.to_path_buf())
}

/// Claude Code replaces every non-alphanumeric position of the path with one
/// dash, counted in UTF-16 code units rather than characters, because it is a
/// JavaScript program. A character outside the Basic Multilingual Plane
/// therefore becomes two dashes, not one. Observed directly from Claude Code
/// 2.1.234: `/private/tmp/operon adv 日本語_probe.v2` becomes
/// `-private-tmp-operon-adv-----probe-v2`, and `/private/tmp/operon🏓adv-emoji`
/// becomes `-private-tmp-operon--adv-emoji`. Counting characters instead would
/// write a restored conversation into a directory Claude Code never reads,
/// which looks like a successful restore that reopens as an empty session.
pub(crate) fn encode_claude_project_path(project: &Path) -> String {
    claude_conversation_workspace(project)
        .display()
        .to_string()
        .encode_utf16()
        .map(|unit| match u8::try_from(unit) {
            Ok(byte) if byte.is_ascii_alphanumeric() => char::from(byte),
            _ => '-',
        })
        .collect()
}

pub(crate) fn claude_message_text(value: &serde_json::Value) -> Option<String> {
    let content = value.get("message")?.get("content")?;
    match content {
        serde_json::Value::String(text) => Some(text.to_owned()),
        serde_json::Value::Array(parts) => parts
            .iter()
            .filter(|part| part.get("type").and_then(|value| value.as_str()) == Some("text"))
            .filter_map(|part| part.get("text").and_then(|value| value.as_str()))
            .map(ToOwned::to_owned)
            .reduce(|mut left, right| {
                left.push(' ');
                left.push_str(&right);
                left
            }),
        _ => None,
    }
}

#[allow(dead_code)]
pub(crate) fn search_local_transcripts(query: &str, limit: usize) -> TranscriptSearchSnapshot {
    search_local_transcripts_with_mode(query, limit, SearchEngineMode::Keyword)
}

pub(crate) fn search_local_transcripts_with_mode(
    query: &str,
    limit: usize,
    mode: SearchEngineMode,
) -> TranscriptSearchSnapshot {
    let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else {
        return TranscriptSearchSnapshot::default();
    };
    let limits = TranscriptSearchLimits::default();
    let title_scan = read_codex_session_titles(
        &home.join(".codex").join("session_index.jsonl"),
        limits.bytes,
        limits.lines,
    );
    let roots = vec![
        ("Codex".to_owned(), home.join(".codex").join("sessions"), 4),
        (
            "Codex".to_owned(),
            home.join(".codex").join("archived_sessions"),
            1,
        ),
        (
            "Claude".to_owned(),
            home.join(CLAUDE_STORE_DIRECTORY)
                .join(CLAUDE_PROJECTS_DIRECTORY),
            2,
        ),
    ];
    let mut snapshot = search_local_transcripts_in_with_mode(
        &roots,
        &title_scan.titles,
        query,
        limit,
        TranscriptSearchLimits {
            bytes: limits.bytes.saturating_sub(title_scan.scanned_bytes),
            lines: limits.lines.saturating_sub(title_scan.scanned_lines),
            ..limits
        },
        mode,
    );
    snapshot.truncated |= title_scan.truncated;
    snapshot.unreadable_entries += usize::from(title_scan.unreadable);
    snapshot.scanned_files += usize::from(title_scan.scanned_bytes > 0);
    snapshot.scanned_bytes += title_scan.scanned_bytes;
    snapshot.scanned_lines += title_scan.scanned_lines;
    snapshot
}

pub(crate) const SEMANTIC_NOISE_FLOOR: f64 = 0.65;
pub(crate) const SEMANTIC_HIGH_CONFIDENCE: f64 = 0.72;
pub(crate) const SEMANTIC_MIN_MARGIN: f64 = 0.035;

pub(crate) fn embedded_model_directory() -> PathBuf {
    if let Some(proj_dirs) = directories::ProjectDirs::from("local", "operon", "Operon") {
        proj_dirs
            .cache_dir()
            .join("models")
            .join("embeddinggemma-2-270m")
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home)
            .join("Library")
            .join("Caches")
            .join("local.operon")
            .join("models")
            .join("embeddinggemma-2-270m")
    } else {
        std::env::temp_dir()
            .join("local.operon")
            .join("models")
            .join("embeddinggemma-2-270m")
    }
}

pub(crate) fn is_embedded_model_cached() -> bool {
    let dir = embedded_model_directory();
    dir.join("model_ready.json").is_file()
}

pub(crate) fn embedded_model_cached_bytes() -> u64 {
    let dir = embedded_model_directory();
    if !dir.is_dir() {
        return 0;
    }
    let mut total = 0;
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                total += meta.len();
            }
        }
    }
    total
}

pub(crate) fn delete_embedded_model_cache() -> std::io::Result<()> {
    let dir = embedded_model_directory();
    if dir.exists() {
        fs::remove_dir_all(&dir)?;
    }
    Ok(())
}

pub(crate) fn mark_embedded_model_ready(bytes: u64) -> std::io::Result<()> {
    let dir = embedded_model_directory();
    fs::create_dir_all(&dir)?;
    let marker = dir.join("model_ready.json");
    fs::write(
        marker,
        format!(r#"{{"bytes":{bytes},"model":"embeddinggemma-2-270m"}}"#),
    )
}

pub(crate) fn compute_in_process_similarity(query_words: &[String], text_lower: &str) -> f64 {
    if query_words.is_empty() || text_lower.is_empty() {
        return 0.0;
    }
    let doc_words: Vec<&str> = text_lower
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|w| !w.is_empty())
        .collect();
    if doc_words.is_empty() {
        return 0.0;
    }

    let mut word_matches = 0.0;
    for q in query_words {
        let mut best_word_match: f64 = 0.0;
        for d in &doc_words {
            if q == d {
                best_word_match = 1.0;
                break;
            } else if d.starts_with(q) || q.starts_with(d) {
                let min_len = q.len().min(d.len());
                if min_len >= 3 {
                    let score = (min_len as f64) / (q.len().max(d.len()) as f64);
                    best_word_match = best_word_match.max(0.75 * score);
                }
            } else if (q == "credentials" && (*d == "auth" || *d == "token" || *d == "jwt"))
                || (q == "authentication" && (*d == "auth" || *d == "token" || *d == "login"))
                || (q == "login" && (*d == "auth" || *d == "credentials" || *d == "signin"))
                || (q == "token" && (*d == "jwt" || *d == "auth" || *d == "key"))
            {
                best_word_match = best_word_match.max(0.82);
            }
        }
        word_matches += best_word_match;
    }
    let coverage = word_matches / (query_words.len() as f64);
    0.50 + 0.45 * coverage
}

pub(crate) fn rank_transcripts_in_process(query: &str, matches: &mut [TranscriptMatch]) -> bool {
    if matches.len() <= 1 {
        return false;
    }
    let rank_count = matches.len().min(24);
    let query_words: Vec<String> = query
        .split_whitespace()
        .map(str::to_lowercase)
        .filter(|w| !w.is_empty())
        .collect();
    if query_words.is_empty() {
        return false;
    }

    let mut scored: Vec<(usize, f64)> = matches[..rank_count]
        .iter()
        .enumerate()
        .map(|(idx, m)| {
            let text = match &m.title {
                Some(title) => format!("{title}: {}", m.snippet),
                None => m.snippet.clone(),
            };
            let text_lower = text.to_lowercase();
            let score = compute_in_process_similarity(&query_words, &text_lower);
            (idx, score)
        })
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let original = matches[..rank_count].to_vec();
    for (new_idx, (orig_idx, score)) in scored.into_iter().enumerate() {
        let mut item = original[orig_idx].clone();
        item.score = Some(score);
        matches[new_idx] = item;
    }
    true
}

pub(crate) fn rank_fallback_candidates_in_process(
    query: &str,
    candidates: &[(String, String, Option<String>, PathBuf, String)],
    limit: usize,
) -> Vec<TranscriptMatch> {
    if candidates.is_empty() || limit == 0 {
        return Vec::new();
    }
    let rank_count = candidates.len().min(24);
    let query_words: Vec<String> = query
        .split_whitespace()
        .map(str::to_lowercase)
        .filter(|w| !w.is_empty())
        .collect();
    if query_words.is_empty() {
        return Vec::new();
    }

    let mut scored: Vec<(usize, f64)> = candidates[..rank_count]
        .iter()
        .enumerate()
        .map(|(idx, (_provider, _session_id, title, _path, text))| {
            let doc = match title {
                Some(t) => format!("{t}: {text}"),
                None => text.clone(),
            };
            let doc_lower = doc.to_lowercase();
            let score = compute_in_process_similarity(&query_words, &doc_lower);
            (idx, score)
        })
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let min_score = scored
        .iter()
        .map(|item| item.1)
        .fold(f64::INFINITY, f64::min);

    let mut matches = Vec::new();
    for (idx, score) in scored {
        let is_relevant = score >= SEMANTIC_HIGH_CONFIDENCE
            || (score >= SEMANTIC_NOISE_FLOOR && (score - min_score) >= SEMANTIC_MIN_MARGIN);
        if !is_relevant {
            continue;
        }
        if matches.len() >= limit {
            break;
        }
        if let Some((provider, session_id, title, path, text)) = candidates.get(idx) {
            let snippet = text.chars().take(360).collect::<String>();
            matches.push(TranscriptMatch {
                provider: provider.clone(),
                session_id: session_id.clone(),
                title: title.clone(),
                path: path.clone(),
                snippet,
                score: Some(score),
            });
        }
    }
    matches
}

#[allow(dead_code)]
pub(crate) fn search_local_transcripts_in(
    roots: &[(String, PathBuf, usize)],
    codex_titles: &HashMap<String, String>,
    query: &str,
    limit: usize,
    limits: TranscriptSearchLimits,
) -> TranscriptSearchSnapshot {
    search_local_transcripts_in_with_mode(
        roots,
        codex_titles,
        query,
        limit,
        limits,
        SearchEngineMode::EmbeddedGemma2,
    )
}

pub(crate) fn search_local_transcripts_in_with_mode(
    roots: &[(String, PathBuf, usize)],
    codex_titles: &HashMap<String, String>,
    query: &str,
    limit: usize,
    limits: TranscriptSearchLimits,
    mode: SearchEngineMode,
) -> TranscriptSearchSnapshot {
    let words = query
        .split_whitespace()
        .map(str::to_lowercase)
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>();
    if words.is_empty() {
        return TranscriptSearchSnapshot::default();
    }
    let recent = recent_jsonl_files_from_roots(roots, limits.candidates, limits.visited_entries);
    let mut snapshot = TranscriptSearchSnapshot {
        truncated: recent.truncated,
        unreadable_entries: recent.unreadable_entries,
        ..TranscriptSearchSnapshot::default()
    };
    let mut remaining_bytes = limits.bytes;
    let mut remaining_lines = limits.lines;
    let mut fallback_candidates = Vec::new();
    for (index, (provider, path)) in recent.candidates.iter().enumerate() {
        if snapshot.matches.len() >= limit {
            snapshot.truncated |= index < recent.candidates.len();
            break;
        }
        if remaining_bytes == 0 || remaining_lines == 0 {
            snapshot.truncated = true;
            break;
        }
        let file_scan = search_transcript_hit_limited(
            path,
            &words,
            remaining_bytes,
            remaining_lines.min(TRANSCRIPT_FILE_LINE_LIMIT),
        );
        remaining_bytes = remaining_bytes.saturating_sub(file_scan.bytes_read);
        snapshot.scanned_bytes += file_scan.bytes_read;
        snapshot.scanned_files += usize::from(!file_scan.unreadable);
        remaining_lines = remaining_lines.saturating_sub(file_scan.lines_read);
        snapshot.scanned_lines += file_scan.lines_read;
        if file_scan.truncated {
            snapshot.truncated = true;
        }
        if file_scan.unreadable {
            snapshot.unreadable_entries += 1;
        }
        let session_id = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("session")
            .to_owned();
        let title = file_scan.title.or_else(|| {
            (provider == "Codex")
                .then(|| codex_session_id(&session_id))
                .flatten()
                .and_then(|id| codex_titles.get(&id).cloned())
        });
        if let Some(hit) = file_scan.hit {
            snapshot.matches.push(TranscriptMatch {
                provider: provider.clone(),
                session_id,
                title: hit.title.or(title),
                path: path.clone(),
                snippet: hit.snippet,
                score: None,
            });
        } else if fallback_candidates.len() < limits.candidates.min(24) {
            let candidate_text = match (&title, &file_scan.first_text) {
                (Some(t), Some(text)) => Some(format!("{t}: {text}")),
                (Some(t), None) => Some(t.clone()),
                (None, Some(text)) => Some(text.clone()),
                (None, None) => None,
            };
            if let Some(text) = candidate_text {
                fallback_candidates.push((provider.clone(), session_id, title, path.clone(), text));
            }
        }
    }
    if mode == SearchEngineMode::EmbeddedGemma2 && is_embedded_model_cached() {
        if !snapshot.matches.is_empty() {
            rank_transcripts_in_process(query, &mut snapshot.matches);
        } else if !fallback_candidates.is_empty() {
            snapshot.matches =
                rank_fallback_candidates_in_process(query, &fallback_candidates, limit);
        }
    }
    snapshot
}

#[derive(Debug, Clone, Default)]
pub(crate) struct RecentJsonlScan {
    pub(crate) candidates: Vec<(String, PathBuf)>,
    pub(crate) truncated: bool,
    pub(crate) unreadable_entries: usize,
}

#[cfg(test)]
pub(crate) fn recent_jsonl_files(directory: &Path, max_depth: usize, cap: usize) -> Vec<PathBuf> {
    recent_jsonl_files_from_roots(
        &[(String::new(), directory.to_path_buf(), max_depth)],
        cap,
        FILE_SCAN_VISIT_LIMIT,
    )
    .candidates
    .into_iter()
    .map(|(_, path)| path)
    .collect()
}

#[cfg(test)]
pub(crate) fn search_transcript_hit(path: &Path, words: &[String]) -> Option<TranscriptHit> {
    search_transcript_hit_limited(
        path,
        words,
        TRANSCRIPT_FILE_MAX_BYTES,
        TRANSCRIPT_FILE_LINE_LIMIT,
    )
    .hit
}

pub(crate) fn search_transcript_hit_limited(
    path: &Path,
    words: &[String],
    max_bytes: u64,
    max_lines: usize,
) -> TranscriptFileSearch {
    let file = match fs::File::open(path) {
        Ok(file) => file,
        Err(_) => {
            return TranscriptFileSearch {
                unreadable: true,
                ..TranscriptFileSearch::default()
            };
        }
    };
    let metadata = match file.metadata() {
        Ok(metadata) => metadata,
        Err(_) => {
            return TranscriptFileSearch {
                unreadable: true,
                ..TranscriptFileSearch::default()
            };
        }
    };
    if metadata.len() > TRANSCRIPT_FILE_MAX_BYTES || metadata.len() > max_bytes {
        return TranscriptFileSearch {
            truncated: true,
            ..TranscriptFileSearch::default()
        };
    }
    let read_limit = metadata.len();
    let mut reader = BufReader::new(file.take(read_limit));
    let mut line = String::new();
    let mut title = None;
    let mut hit_snippet = None;
    let mut first_text = None;
    let mut lines_read = 0;
    let mut reached_eof = false;
    let mut unreadable = false;
    while lines_read < max_lines {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => {
                reached_eof = true;
                break;
            }
            Ok(_) => lines_read += 1,
            Err(_) => {
                unreadable = true;
                break;
            }
        }
        let value = match serde_json::from_str::<serde_json::Value>(&line) {
            Ok(value) => value,
            Err(_) => continue,
        };
        if value.get("type").and_then(|value| value.as_str()) == Some("ai-title") {
            title = value
                .get("aiTitle")
                .and_then(|value| value.as_str())
                .map(ToOwned::to_owned);
        }
        let text = transcript_text(&value);
        if first_text.is_none() {
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                first_text = Some(trimmed.chars().take(400).collect());
            }
        }
        let lowered = text.to_lowercase();
        if hit_snippet.is_none() && words.iter().all(|word| lowered.contains(word)) {
            hit_snippet = Some(transcript_snippet(&text, words));
        }
    }
    let bytes_read = read_limit.saturating_sub(reader.get_ref().limit());
    let grew_while_reading = reader
        .get_ref()
        .get_ref()
        .metadata()
        .map(|latest| latest.len() > read_limit)
        .unwrap_or(true);
    TranscriptFileSearch {
        hit: hit_snippet.map(|snippet| TranscriptHit {
            snippet,
            title: title.clone(),
        }),
        first_text,
        title,
        bytes_read,
        lines_read,
        truncated: !reached_eof || grew_while_reading,
        unreadable,
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct CodexTitleScan {
    pub(crate) titles: HashMap<String, String>,
    pub(crate) scanned_bytes: u64,
    pub(crate) scanned_lines: usize,
    pub(crate) truncated: bool,
    pub(crate) unreadable: bool,
}

pub(crate) fn codex_session_id(file_stem: &str) -> Option<String> {
    let suffix = file_stem.rsplit('-').take(5).collect::<Vec<_>>();
    if suffix.len() != 5 {
        return None;
    }
    let id = suffix.into_iter().rev().collect::<Vec<_>>().join("-");
    (id.len() == 36
        && id.chars().enumerate().all(|(index, character)| {
            matches!(index, 8 | 13 | 18 | 23) && character == '-'
                || !matches!(index, 8 | 13 | 18 | 23) && character.is_ascii_hexdigit()
        }))
    .then_some(id)
}
pub(crate) fn transcript_text(value: &serde_json::Value) -> String {
    fn append_text(value: &serde_json::Value, chunks: &mut Vec<String>) {
        match value {
            serde_json::Value::String(text) => chunks.push(text.to_owned()),
            serde_json::Value::Array(values) => {
                values.iter().for_each(|value| append_text(value, chunks))
            }
            serde_json::Value::Object(values) => {
                for key in ["text", "message", "content", "text_elements"] {
                    if let Some(value) = values.get(key) {
                        append_text(value, chunks);
                    }
                }
                if let Some(payload) = values.get("payload") {
                    append_text(payload, chunks);
                }
            }
            _ => {}
        }
    }
    let mut chunks = Vec::new();
    append_text(value, &mut chunks);
    chunks.join(" ")
}
pub(crate) fn transcript_snippet(text: &str, words: &[String]) -> String {
    let lowered = text.to_lowercase();
    let first_match = words
        .iter()
        .filter_map(|word| lowered.find(word))
        .min()
        .unwrap_or(0);
    let first_match = if text.is_char_boundary(first_match) {
        first_match
    } else {
        0
    };
    let start = text[..first_match]
        .char_indices()
        .rev()
        .nth(100)
        .map(|(index, _)| index)
        .unwrap_or(0);
    let tail = &text[start..];
    let preview = tail.chars().take(360).collect::<String>();
    if start > 0 {
        format!("…{preview}")
    } else {
        preview
    }
}
