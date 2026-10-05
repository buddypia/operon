//! Agent state as the CLIs themselves report it.
//!
//! Every supported CLI runs a command of the user's choosing at the start of a
//! turn, around each tool, when it needs permission, and when the turn ends.
//! This module writes that command — one small POSIX script per CLI — registers
//! it in the CLI's own settings file, and listens on a Unix socket for what the
//! script posts. `src/tmux.rs` carries the session's identity into the pane's
//! environment so the script can say which session it speaks for.
//!
//! The screen heuristic beside this one is not replaced. It answers for a CLI
//! whose hooks are not installed, for the minutes after a restart before the
//! next event, and whenever a reported state has gone stale — silence is not
//! evidence, so a state nobody has confirmed for half an hour stops outranking
//! what is visibly on the screen.
//!
//! Three rules the CLIs impose, each of which cost somebody a bug before it was
//! written down:
//!
//! - A hook must never make its CLI fail. Every script exits `0` on every path,
//!   reads all of stdin before doing anything else (a hook that exits early
//!   hands the CLI an `EPIPE`), and answers on stdout with whatever that CLI
//!   treats as "no opinion" *before* it reads.
//! - The settings file belongs to the person, not to Operon. Entries that are
//!   not ours survive every install and every removal untouched, and the file is
//!   rewritten only when its parsed content actually changed.
//! - Codex keeps a trust hash beside each hook entry, keyed by the entry's index
//!   in its file. Ours is appended rather than inserted, so no existing entry's
//!   index moves and no already-approved hook needs approving again.

use crate::*;
use std::io::ErrorKind;
use std::os::unix::net::{UnixListener, UnixStream};

/// The identity a managed session carries so that a hook running inside it can
/// name the session it belongs to, and the socket it should reach.
///
/// The launch token is minted per launch rather than per session: a retried or
/// respawned pane gets a new one, so a dying process's last events cannot be
/// mistaken for the new process's first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HookEnvironment {
    pub(crate) session: String,
    pub(crate) launch_token: String,
    pub(crate) endpoint: PathBuf,
    pub(crate) socket: PathBuf,
}

impl HookEnvironment {
    /// The `KEY=VALUE` pairs `tmux new-session -e` is given. The socket is
    /// passed as well as the endpoint file so a hook works before the file has
    /// been read, and the endpoint file is what lets it keep working after
    /// Operon restarts under a new socket.
    pub(crate) fn variables(&self) -> Vec<(&'static str, String)> {
        vec![
            (HOOK_SESSION_ENV, self.session.clone()),
            (HOOK_LAUNCH_ENV, self.launch_token.clone()),
            (HOOK_ENDPOINT_ENV, self.endpoint.display().to_string()),
            (HOOK_SOCKET_ENV, self.socket.display().to_string()),
        ]
    }
}

pub(crate) fn hook_directory(data_file: &Path) -> PathBuf {
    data_file
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(HOOK_DIRECTORY)
}

pub(crate) fn hook_socket_path(data_file: &Path) -> PathBuf {
    hook_directory(data_file).join(HOOK_SOCKET_FILE)
}

pub(crate) fn hook_endpoint_path(data_file: &Path) -> PathBuf {
    hook_directory(data_file).join(HOOK_ENDPOINT_FILE)
}

pub(crate) fn hook_settings_path(data_file: &Path) -> PathBuf {
    hook_directory(data_file).join(HOOK_SETTINGS_FILE)
}

/// The managed script's file name, built from the persisted agent id so that
/// the name and the provider cannot drift apart.
pub(crate) fn hook_script_name(provider: CliProvider) -> String {
    format!("{}-hook.sh", provider.agent())
}

pub(crate) fn hook_script_path(data_file: &Path, provider: CliProvider) -> PathBuf {
    hook_directory(data_file).join(hook_script_name(provider))
}

/// How an installed entry is recognised as ours, in every settings file: the
/// hook directory and the script's own name. Recognising by file name rather
/// than by the whole command is what lets an entry written by an older build,
/// or under a different data directory, still be cleaned up.
pub(crate) fn hook_command_marker(provider: CliProvider) -> String {
    format!("{HOOK_DIRECTORY}/{}", hook_script_name(provider))
}

/// The endpoint file the scripts source on every invocation. Only the socket
/// path is in it: the session and the launch token belong to the pane and must
/// not be shared between panes.
///
/// The value is single-quoted because the path this app is given on macOS
/// contains a space — `Application Support` — and `.` on an unquoted assignment
/// stops at it, leaving a truncated path and a stray command. That was found by
/// running the real script against the real directory, not by reading it.
pub(crate) fn hook_endpoint_contents(socket: &Path) -> String {
    format!(
        "{HOOK_SOCKET_ENV}={}\n",
        shell_quote(&socket.display().to_string())
    )
}

/// The events registered per CLI, with the payload-shape each needs.
///
/// `PreCompact` is deliberately absent from Claude's list: an aborted
/// compaction fires it with no matching completion, which pins a session at
/// "working" for ever. `PostCompact` with a manual trigger is the one signal
/// that a `/compact` finished, and it is treated as a boundary rather than as a
/// finished turn.
pub(crate) fn hook_events(provider: CliProvider) -> &'static [(&'static str, bool)] {
    match provider {
        // (event, needs a `matcher` wrapper)
        CliProvider::Claude => &[
            ("SessionStart", false),
            ("UserPromptSubmit", false),
            ("Stop", false),
            ("StopFailure", false),
            ("PostCompact", false),
            ("PreToolUse", true),
            ("PostToolUse", true),
            ("PostToolUseFailure", true),
            ("PermissionRequest", true),
        ],
        CliProvider::Codex => &[
            ("SessionStart", false),
            ("UserPromptSubmit", false),
            ("Stop", false),
            ("PreToolUse", false),
            ("PostToolUse", false),
            ("PermissionRequest", false),
        ],
        CliProvider::Gemini => &[
            ("PreInvocation", false),
            ("PostInvocation", false),
            ("Stop", false),
            ("PreToolUse", true),
            ("PostToolUse", true),
        ],
    }
}

/// What the CLI must see on stdout for a given event, before the script does
/// anything else.
///
/// Claude treats an empty stdout from a permission-capable hook as a refusal,
/// so `{}` goes out first. Antigravity refuses a tool whose `PreToolUse` hook
/// says nothing, and `allow` there would auto-approve every tool it ever sees —
/// `ask` leaves its own prompt exactly where it was. Codex wants nothing.
pub(crate) fn hook_stdout_answer(provider: CliProvider, event: &str) -> Option<&'static str> {
    match (provider, event) {
        (CliProvider::Claude, _) => Some("{}"),
        (CliProvider::Gemini, "Stop") => Some("{\"decision\":\"\"}"),
        (CliProvider::Gemini, "PreToolUse") => Some("{\"decision\":\"ask\"}"),
        (CliProvider::Gemini, _) => Some("{}"),
        (CliProvider::Codex, _) => None,
    }
}

/// The managed script, one per CLI.
///
/// It answers the CLI first, reads all of stdin, refreshes the socket path from
/// the endpoint file, and posts one JSON envelope. Every branch ends in `exit
/// 0`: a status hook that can fail is a status hook that breaks the agent it
/// was watching.
pub(crate) fn hook_script(provider: CliProvider) -> String {
    let source = provider.agent();
    // Antigravity is the only CLI whose answer depends on the event, and it is
    // also the only one that passes the event in the environment rather than in
    // the payload — so its answer is a `case` and the others are a constant.
    let answer = match provider {
        CliProvider::Gemini => format!(
            "case \"${HOOK_EVENT_ENV}\" in\n  Stop) printf '%s\\n' '{}' ;;\n  PreToolUse) printf '%s\\n' '{}' ;;\n  *) printf '%s\\n' '{}' ;;\nesac\n",
            hook_stdout_answer(provider, "Stop").unwrap_or("{}"),
            hook_stdout_answer(provider, "PreToolUse").unwrap_or("{}"),
            hook_stdout_answer(provider, "other").unwrap_or("{}"),
        ),
        _ => match hook_stdout_answer(provider, "") {
            Some(answer) => format!("printf '%s\\n' '{answer}'\n"),
            None => String::new(),
        },
    };
    // Antigravity fires hooks with nothing on stdin; the others always send a
    // payload, and an empty one from them means the CLI is not talking to us.
    let empty_payload = match provider {
        CliProvider::Gemini => "payload='{}'\n",
        _ => "exit 0\n",
    };
    format!(
        r#"#!/bin/sh
# Operon agent status hook — generated file, rewritten on every launch.
# Reports this pane's agent state to the Operon that started it. Never fails,
# never blocks: every path exits 0 and the post is capped at 1.5 seconds.
{answer}payload=$({{ command -p cat 2>/dev/null || cat; }})
if [ -z "$payload" ]; then
{empty_payload}fi
if [ -n "${HOOK_ENDPOINT_ENV}" ] && [ -r "${HOOK_ENDPOINT_ENV}" ]; then
  . "${HOOK_ENDPOINT_ENV}" 2>/dev/null || :
fi
if [ -z "${HOOK_SOCKET_ENV}" ] || [ -z "${HOOK_SESSION_ENV}" ]; then
  exit 0
fi
if [ ! -S "${HOOK_SOCKET_ENV}" ]; then
  exit 0
fi
printf '{{"source":"{source}","session":"%s","token":"%s","event":"%s","payload":%s}}' \
  "${HOOK_SESSION_ENV}" "${HOOK_LAUNCH_ENV}" "${HOOK_EVENT_ENV}" "$payload" \
  | curl -sS -X POST --unix-socket "${HOOK_SOCKET_ENV}" http://localhost/hook \
      --connect-timeout 0.5 --max-time 1.5 \
      -H 'Content-Type: application/json' \
      --data-binary @- >/dev/null 2>&1 || :
exit 0
"#
    )
}

/// The command written into the CLI's settings file.
///
/// It names the script by absolute path, refuses to run it unless it is there
/// and executable, and falls back to the CLI's own "no opinion" answer when it
/// is not — a settings file that survives Operon being deleted must not break
/// the agent.
pub(crate) fn hook_managed_command(script: &Path, provider: CliProvider, event: &str) -> String {
    let script = script.display();
    let fallback = match hook_stdout_answer(provider, event) {
        Some(answer) => format!(
            "{{ command -p cat 2>/dev/null || cat; }} >/dev/null 2>&1 || :; printf '%s\\n' '{answer}'"
        ),
        None => "{ command -p cat 2>/dev/null || cat; } >/dev/null 2>&1 || :".to_owned(),
    };
    format!(
        "if [ -x '{script}' ]; then {HOOK_EVENT_ENV}='{event}' /bin/sh '{script}'; else {fallback}; fi"
    )
}

// ── what a settings file looks like after an install ────────────────────────

/// One CLI's registration state, as the settings screen reports it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum HookInstallState {
    Installed,
    /// The CLI is not on `PATH`, or the mechanism is switched off. The reason is
    /// shown as written.
    NotInstalled(String),
    Failed(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HookInstall {
    pub(crate) provider: CliProvider,
    pub(crate) state: HookInstallState,
}

/// Operon's own note beside the scripts: whether the mechanism is on, and the
/// exact Codex trust keys the last install wrote, so removal deletes those and
/// nothing a person approved themselves.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct HookSettings {
    #[serde(default = "hooks_enabled_by_default")]
    pub(crate) enabled: bool,
    #[serde(default)]
    pub(crate) codex_trust_keys: Vec<String>,
}

pub(crate) fn hooks_enabled_by_default() -> bool {
    true
}

pub(crate) fn load_hook_settings(data_file: &Path) -> HookSettings {
    fs::read(hook_settings_path(data_file))
        .ok()
        .and_then(|raw| serde_json::from_slice::<HookSettings>(&raw).ok())
        .unwrap_or(HookSettings {
            enabled: hooks_enabled_by_default(),
            codex_trust_keys: Vec::new(),
        })
}

pub(crate) fn save_hook_settings(data_file: &Path, settings: &HookSettings) -> Result<()> {
    let path = hook_settings_path(data_file);
    ensure_hook_directory(&path)?;
    let encoded = serde_json::to_string_pretty(settings)?;
    write_file_atomically(&path, format!("{encoded}\n").as_bytes())?;
    Ok(())
}

fn ensure_hook_directory(path: &Path) -> Result<()> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    fs::create_dir_all(parent)?;
    let _ = fs::set_permissions(parent, fs::Permissions::from_mode(0o700));
    Ok(())
}

/// Writes the managed script, its endpoint file, and the directory that holds
/// them. The script is rewritten unconditionally: it is generated, and a stale
/// copy from an older build is exactly what the file name match cannot catch.
pub(crate) fn write_hook_runtime(
    data_file: &Path,
    socket: &Path,
    providers: &[CliProvider],
) -> Result<()> {
    let endpoint = hook_endpoint_path(data_file);
    ensure_hook_directory(&endpoint)?;
    write_file_atomically(&endpoint, hook_endpoint_contents(socket).as_bytes())?;
    let _ = fs::set_permissions(&endpoint, fs::Permissions::from_mode(0o600));
    for provider in providers {
        let script = hook_script_path(data_file, *provider);
        write_file_atomically(&script, hook_script(*provider).as_bytes())?;
        fs::set_permissions(&script, fs::Permissions::from_mode(0o755))?;
    }
    if providers.contains(&CliProvider::Claude) {
        let reader = statusline_script_path(data_file);
        write_file_atomically(&reader, statusline_script().as_bytes())?;
        fs::set_permissions(&reader, fs::Permissions::from_mode(0o755))?;
    }
    Ok(())
}

// ── editing another product's settings file ─────────────────────────────────

/// Reads a JSON settings file that may not exist yet. A file past the ceiling,
/// or one that is not an object, is an error rather than something to overwrite.
fn read_json_object(path: &Path) -> Result<serde_json::Map<String, serde_json::Value>> {
    match fs::metadata(path) {
        Ok(metadata) => {
            if metadata.len() > HOOK_SETTINGS_MAX_BYTES {
                return Err(anyhow!(tf!(
                    "設定ファイルが大きすぎます（{p0} バイト）",
                    p0 = metadata.len()
                )));
            }
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {
            return Ok(serde_json::Map::new());
        }
        Err(error) => return Err(error.into()),
    }
    let raw = fs::read(path)?;
    if raw.iter().all(u8::is_ascii_whitespace) {
        return Ok(serde_json::Map::new());
    }
    match serde_json::from_slice::<serde_json::Value>(&raw)? {
        serde_json::Value::Object(map) => Ok(map),
        _ => Err(anyhow!(tr(
            "設定ファイルが JSON オブジェクトではありません"
        ))),
    }
}

/// Writes a settings file only when its parsed content changed, keeping the
/// mode it had and leaving one backup behind. The write itself is the store's:
/// temporary file, flush, rename, directory flush.
fn write_json_object(
    path: &Path,
    value: &serde_json::Map<String, serde_json::Value>,
) -> Result<()> {
    let existing = read_json_object(path).ok();
    if existing.as_ref() == Some(value) {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mode = fs::metadata(path)
        .ok()
        .map(|metadata| metadata.permissions().mode());
    if path.exists() {
        let _ = fs::copy(path, path.with_extension("operon.bak"));
    }
    let encoded = serde_json::to_string_pretty(value)?;
    // A settings file this app did not create may be read-only, as
    // Antigravity's is. The rename needs the directory, not the file, and the
    // mode is put back so the file stays as its owner left it.
    if let Some(mode) = mode {
        let _ = fs::set_permissions(path, fs::Permissions::from_mode(mode | 0o200));
    }
    write_file_atomically(path, format!("{encoded}\n").as_bytes())?;
    if let Some(mode) = mode {
        let _ = fs::set_permissions(path, fs::Permissions::from_mode(mode));
    }
    Ok(())
}

fn managed_handler(command: String) -> serde_json::Value {
    serde_json::json!({
        "type": "command",
        "command": command,
        "timeout": HOOK_ENTRY_TIMEOUT_SECONDS,
    })
}

fn group_is_managed(group: &serde_json::Value, marker: &str) -> bool {
    group
        .get("hooks")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|handlers| {
            handlers.iter().any(|handler| {
                handler
                    .get("command")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|command| command.contains(marker))
            })
        })
        || group
            .get("command")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|command| command.contains(marker))
}

/// Drops every group this app owns from one event's array, leaving the rest in
/// the order they were written. An event left with nothing is dropped only when
/// this app is what emptied it: an empty array somebody else wrote is theirs.
fn strip_managed_groups(events: &mut serde_json::Map<String, serde_json::Value>, marker: &str) {
    let names: Vec<String> = events.keys().cloned().collect();
    for name in names {
        let Some(serde_json::Value::Array(groups)) = events.get_mut(&name) else {
            continue;
        };
        let before = groups.len();
        groups.retain(|group| !group_is_managed(group, marker));
        if groups.is_empty() && groups.len() != before {
            events.remove(&name);
        }
    }
}

/// The Claude Code and Codex shape: `hooks.<Event>` is an array of groups, and a
/// group holds handlers. Ours goes on the end, so no existing group's index
/// moves — which matters to Codex, whose trust keys are indexed.
fn install_event_groups(
    settings: &mut serde_json::Map<String, serde_json::Value>,
    provider: CliProvider,
    script: &Path,
    remove_only: bool,
) -> Vec<(String, usize, usize)> {
    let marker = hook_command_marker(provider);
    let mut events = match settings.remove("hooks") {
        Some(serde_json::Value::Object(map)) => map,
        _ => serde_json::Map::new(),
    };
    strip_managed_groups(&mut events, &marker);
    let mut written = Vec::new();
    if !remove_only {
        for (event, needs_matcher) in hook_events(provider) {
            let handler = managed_handler(hook_managed_command(script, provider, event));
            let group = if *needs_matcher {
                serde_json::json!({ "matcher": "*", "hooks": [handler] })
            } else {
                serde_json::json!({ "hooks": [handler] })
            };
            let slot = events
                .entry((*event).to_owned())
                .or_insert_with(|| serde_json::Value::Array(Vec::new()));
            if !slot.is_array() {
                *slot = serde_json::Value::Array(Vec::new());
            }
            let Some(array) = slot.as_array_mut() else {
                continue;
            };
            written.push(((*event).to_owned(), array.len(), 0));
            array.push(group);
        }
    }
    if !events.is_empty() {
        settings.insert("hooks".to_owned(), serde_json::Value::Object(events));
    }
    written
}

/// A CLI's configuration root: the directory that holds its settings, its
/// hooks, and its login.
///
/// The machine's own is under the home folder; a registered account names one
/// directly, which is the shape `CLAUDE_CONFIG_DIR` and `CODEX_HOME` take. The
/// two are one level apart — `home` is the *parent* of `.claude`, and the
/// variable names `.claude` itself — so the two halves are written once here
/// rather than at each call site.
///
/// The wrappers that used to compose them for a home are gone: nothing in
/// production takes a home any more, and a convenience function only the tests
/// call is a second path that can drift from the one that ships.
/// `the_hooks_follow_every_account_that_is_registered` walks the real chain —
/// `account_roots` then `claude_settings_in` — and asserts the machine's own
/// settings file is still exactly where it was.
pub(crate) fn claude_config_root(home: &Path) -> PathBuf {
    home.join(CLAUDE_STORE_DIRECTORY)
}

pub(crate) fn codex_config_root(home: &Path) -> PathBuf {
    home.join(CODEX_STORE_DIRECTORY)
}

pub(crate) fn claude_settings_in(root: &Path) -> PathBuf {
    root.join(CLAUDE_SETTINGS_FILE)
}

pub(crate) fn codex_hooks_in(root: &Path) -> PathBuf {
    root.join(CODEX_HOOKS_FILE)
}

pub(crate) fn codex_config_in(root: &Path) -> PathBuf {
    root.join(CODEX_CONFIG_FILE)
}

/// Every configuration root this agent's hooks belong in: the machine's own,
/// then each registered account's.
///
/// This is the whole of "the hooks follow the account". Without it a session
/// launched under a second login finds no managed hooks in the directory it was
/// pointed at, and goes quiet — the state chip stops moving, a finished turn is
/// never marked unread, and the usage reading disappears.
pub(crate) fn account_roots(
    provider: CliProvider,
    home: &Path,
    accounts: &AgentAccounts,
) -> Vec<PathBuf> {
    let own = match provider {
        CliProvider::Claude => claude_config_root(home),
        CliProvider::Codex => codex_config_root(home),
        CliProvider::Gemini => home.to_path_buf(),
    };
    let mut roots = vec![own];
    roots.extend(
        accounts
            .for_agent(provider.agent())
            .into_iter()
            // A folder that has been moved or deleted since it was registered
            // is skipped rather than created: this application does not own it,
            // and making one would be inventing a login.
            .filter(|account| account.path.is_dir())
            .map(|account| account.path.clone()),
    );
    roots
}

pub(crate) fn antigravity_hooks_path(home: &Path) -> PathBuf {
    home.join(GEMINI_STORE_DIRECTORY)
        .join(GEMINI_CONFIG_DIRECTORY)
        .join(GEMINI_HOOKS_FILE)
}

pub(crate) fn install_claude_hooks(data_file: &Path, root: &Path, remove_only: bool) -> Result<()> {
    let path = claude_settings_in(root);
    if remove_only && !path.exists() {
        return Ok(());
    }
    let script = hook_script_path(data_file, CliProvider::Claude);
    let mut settings = read_json_object(&path)?;
    install_event_groups(&mut settings, CliProvider::Claude, &script, remove_only);
    // The usage reader rides in the same file, in a slot of its own, and is
    // never put back once the person has taken it out.
    let reader = statusline_script_path(data_file);
    let marker = hook_directory(data_file).join(STATUSLINE_INSTALL_MARKER);
    apply_statusline(&mut settings, &reader, marker.exists(), !remove_only);
    if !remove_only && !marker.exists() {
        let _ = write_file_atomically(&marker, b"");
    }
    write_json_object(&path, &settings)
}

/// Codex's own name for an event inside a trust key: the event in snake case.
pub(crate) fn codex_trust_label(event: &str) -> String {
    let mut label = String::with_capacity(event.len() + 4);
    for (index, character) in event.char_indices() {
        if character.is_ascii_uppercase() {
            if index != 0 {
                label.push('_');
            }
            label.push(character.to_ascii_lowercase());
        } else {
            label.push(character);
        }
    }
    label
}

/// The hash Codex stores beside an approved hook: SHA-256 over the entry's
/// canonical JSON, keys sorted at every level, no whitespace.
///
/// Verified against three entries Codex itself wrote on this machine, which is
/// the only way to know an algorithm inferred from a file is the algorithm that
/// produced it.
pub(crate) fn codex_trusted_hash(event: &str, command: &str, matcher: Option<&str>) -> String {
    codex_trusted_hash_with(event, command, matcher, HOOK_ENTRY_TIMEOUT_SECONDS, None)
}

/// The same hash for an entry this app did not write, which is the only way to
/// check the algorithm against one Codex produced itself.
pub(crate) fn codex_trusted_hash_with(
    event: &str,
    command: &str,
    matcher: Option<&str>,
    timeout: u64,
    status_message: Option<&str>,
) -> String {
    let mut handler = serde_json::Map::new();
    handler.insert("async".to_owned(), serde_json::Value::Bool(false));
    handler.insert(
        "command".to_owned(),
        serde_json::Value::String(command.to_owned()),
    );
    if let Some(status_message) = status_message {
        handler.insert(
            "statusMessage".to_owned(),
            serde_json::Value::String(status_message.to_owned()),
        );
    }
    handler.insert(
        "timeout".to_owned(),
        serde_json::Value::from(timeout.max(1)),
    );
    handler.insert(
        "type".to_owned(),
        serde_json::Value::String("command".to_owned()),
    );
    let handler = serde_json::Value::Object(handler);
    let mut identity = serde_json::Map::new();
    identity.insert(
        "event_name".to_owned(),
        serde_json::Value::String(codex_trust_label(event)),
    );
    identity.insert("hooks".to_owned(), serde_json::Value::Array(vec![handler]));
    if let Some(matcher) = matcher {
        identity.insert(
            "matcher".to_owned(),
            serde_json::Value::String(matcher.to_owned()),
        );
    }
    // `serde_json::Map` is a BTreeMap by default, so serializing it is already
    // the canonical form: keys ascending, no spaces.
    let canonical = serde_json::Value::Object(identity).to_string();
    format!("sha256:{}", sha256_hex(canonical.as_bytes()))
}

pub(crate) fn codex_trust_key(
    hooks_path: &Path,
    event: &str,
    group: usize,
    handler: usize,
) -> String {
    format!(
        "{}:{}:{group}:{handler}",
        hooks_path.display(),
        codex_trust_label(event)
    )
}

/// The key and the raw value a `key = value` line assigns, or `None` for a blank
/// line, a table header, or anything else without an assignment. Deliberately
/// not a TOML parse — this is the one shape `apply_codex_config` has to
/// recognise, and the line editor exists so the rest of the file is never
/// interpreted at all.
///
/// A comment needs no case of its own, which is worth saying because the first
/// version had one: `# hooks = true` yields the key `# hooks`, so a commented
/// line can never equal the name being looked for. The explicit check read as
/// carefulness and could not be made to fail — its mutation came back green —
/// so it is gone rather than left standing as a guard nobody can test.
fn assignment_of(line: &str) -> Option<(&str, &str)> {
    let trimmed = line.trim_start();
    if trimmed.starts_with('[') {
        return None;
    }
    let (key, value) = trimmed.split_once('=')?;
    Some((key.trim(), value.trim()))
}

/// Whether a line opens the named table. Not an equality test on the trimmed
/// line: `[features] # my favourites` is the same table, and the whole reason
/// this file is edited line by line rather than re-serialised is that the
/// comments and ordering belong to the person who wrote them. Review found the
/// exact test appending a SECOND `[features]` when the first carried a trailing
/// comment, which is `duplicate key `features` in document root` — the same
/// "Codex cannot read its config at all" this function was fixed for, put back
/// by the fix. A dotted child does not count: `[features.context_management]`
/// keeps its own keys.
///
/// Whitespace inside the brackets is the same table too, and closing only the
/// comment spelling was found to be half a fix in the next round: `[ features ]`
/// is valid TOML and produced the identical duplicate. Every position that can
/// hold space is trimmed rather than the two that had been reported.
///
/// What it does NOT see, because nothing here parses: a `[features]` written
/// inside a multi-line basic string is read as a real header. The input is
/// contrived and the consequence is mild — the flag is judged already set and
/// nothing is written, so Operon's hooks register and never fire — but a line
/// editor's blind spots belong in its documentation rather than in a reader's
/// surprise.
fn opens_table(line: &str, name: &str) -> bool {
    let Some(rest) = line.trim_start().strip_prefix('[') else {
        return false;
    };
    let Some(rest) = rest.trim_start().strip_prefix(name) else {
        return false;
    };
    let Some(rest) = rest.trim_start().strip_prefix(']') else {
        return false;
    };
    let rest = rest.trim_start();
    rest.is_empty() || rest.starts_with('#')
}

/// The text between the brackets of a line SHAPED like a table header, whatever
/// the name turns out to be, and whether the brackets were doubled: brackets
/// that open the line and close it, with nothing after them but a comment.
/// `[[name]]`, an array-of-tables header, counts — it opens a table exactly as
/// much as `[name]` does, and reading only the single-bracket form left
/// `[[profiles]]` invisible to every scan here, which is how the top-level
/// region swallowed one and deleted the `hooks` key inside it.
///
/// The second half of the answer is for the one caller that cannot treat the
/// two alike. `[[hooks]]` binds an ARRAY at `hooks`, so `[hooks.state."K"]`
/// after it lands inside that array's last element instead of in a table of our
/// own — a document that still parses and means something nobody asked for. It
/// is returned from here rather than re-read from the line, because "doubled
/// brackets mean an array of tables" living in two places is the shape every
/// finding in this change has had.
///
/// Deliberately wider than `opens_table` — `["features"]` is a header this file
/// does not recognise as `features`, and still a header — and narrower than "the
/// trim starts with `[`", which a multi-line array's continuation row satisfies.
fn table_header_shape(line: &str) -> Option<(&str, bool)> {
    let trimmed = line.trim();
    let inner = trimmed.strip_prefix('[')?;
    let (inner, closing, array) = match inner.strip_prefix('[') {
        Some(doubled) => (doubled, "]]", true),
        None => (inner, "]", false),
    };
    let (name, rest) = inner.split_once(closing)?;
    let rest = rest.trim_start();
    (rest.is_empty() || rest.starts_with('#')).then_some((name.trim(), array))
}

/// A value that does not end on the line it started on.
///
/// The two nest, and the first version of this enum did not let them: a `"""`
/// opened INSIDE an array was invisible, so its contents were counted as
/// brackets and read as structure. Review reproduced three losses from that one
/// gap — a flag written into the middle of somebody's string, a line deleted
/// out of one, and a person's own `hooks` key deleted because a `]` in their
/// prose closed the array early and hid the header below. All three parse
/// before and after, which is the shape this change keeps meeting.
#[derive(Clone, Copy)]
enum OpenValue {
    /// An array, carrying how many brackets are still to close.
    Array(usize),
    /// A multi-line string, carrying the array depth AROUND it so that the
    /// array is still open when the string closes, and which fence closes it.
    Fenced { depth: usize, literal: bool },
}

/// Which lines of a config are outside any value that began on an earlier line.
///
/// **This is the only thing the line editor remembers, and every scan here has
/// to consult it.** Three separate defects came from reading one line at a
/// time, and they are one defect: whether a line means what it looks like
/// depends on lines already passed.
///
///   - `  [1]` inside a multi-line array is a legal table name read alone. So
///     are `[1.2]`, `["a"]` and `[true]`. Requiring the name to look like a key
///     path closed exactly one spelling — the one with a comma in it — and the
///     class stayed open.
///   - `[features]` inside a multi-line string is not a header, and this
///     module's own doc comment used to claim the consequence was mild. It is
///     not: the flag is inserted under the false header and the document gains
///     a duplicate key.
///   - `hooks = true` inside a multi-line string is not an assignment, and the
///     top-level repair removed the line, silently emptying somebody's note.
///
/// Being wrong here is not safe in one direction, which is why `scan` below
/// bothers with quotes, fences and comments. Believing a line is inside a value
/// hides it, and the scan that must not have a line hidden from it is the one
/// asking whether `[features]` already sets `hooks`: it does not see the key, it
/// inserts a second one, and the document gains a duplicate. So the callers
/// split — the scans that decide WHERE a table begins and ends consult this, and
/// the vetoes beside them deliberately do not. Lesson 030, third application.
///
/// Returns which lines are structural AND whether the document closed
/// everything it opened, because both are the same walk and this file has spent
/// six rounds on what happens when one rule gets two readers. The second answer
/// is what the walk is already holding when it reaches the end; taking it from
/// anywhere else would be a claim that has to agree with this one.
fn structural_lines(lines: &[String]) -> (Vec<bool>, bool) {
    let mut outside = Vec::with_capacity(lines.len());
    let mut open: Option<OpenValue> = None;
    for line in lines {
        outside.push(open.is_none());
        open = advance(open, line);
    }
    (outside, open.is_none())
}

/// A line that opens a table and cannot be READ as one — as opposed to a line
/// that is not a table header to anybody.
///
/// `table_header_shape` finds its closing bracket without respecting quotes,
/// so `[projects."/Users/me/[work]"]` splits inside the key: valid TOML, a
/// folder a person picked in a dialog, and a header to nothing in this file.
/// THREE readers need to know that, for three different reasons, which is why
/// the question is asked in one place and answered in three. Round 16 gave it
/// to two of them and review found the third within the round — the eighth time
/// in this change that one rule reached the readers somebody listed and not the
/// one they did not.
///
///   - `first_table` must STOP here. Everything past it is treated as top
///     level and a `hooks` scalar found there is deleted, so running past a
///     real table takes a key out of somebody's configuration. Stopping early
///     costs nothing, because a top-level scalar precedes every header by
///     definition.
///   - `can_hold_hook_state` must REFUSE here. It cannot know what the
///     document declares, so it cannot know whether the block it is about to
///     append is a second declaration of the same table.
///   - the DROP LOOP must not drop, and this is the reader that made the
///     refusal expensive. It removes Operon's own `[hooks.state."…"]` blocks so
///     the append can put them back — and the append is the thing
///     `can_hold_hook_state` has just decided to refuse. It deleted what it
///     could not re-add: measured as trust blocks 2, 0, 0, 0 across four
///     launches once a person opened a folder named `~/src/repo[1]` and Codex
///     wrote `[projects."/Users/me/src/repo[1]"]` into the file. Not a
///     degradation that heals on the next launch — the working state is gone
///     and the same call declined to write it again. It is the document-level
///     answer here and not the per-line one: the loop must stand still if the
///     document holds such a header ANYWHERE, because that is the condition
///     `can_hold_hook_state` refuses on, and on the ENABLING path dropping is
///     only ever safe when the append that follows it will happen.
///
///     On the REMOVING path there is no append and none is wanted — the drop
///     is the request, and there is no second half for anything to veto. The
///     loop stands still there too, for the wider reason rather than that
///     one: we did not re-derive this data and we cannot re-derive it, so we
///     leave it. Review measured the trade and it is small in both
///     directions, but the two paths reach the same answer by different
///     arguments, and a rule stated without its exception is how this change
///     has been wrong three rounds running.
///
/// The `]` test is the whole point and is not decoration. `[a` closes nothing,
/// so it opens no table, so the `hooks = true` under it is genuinely top-level
/// and genuinely the line an older Operon wrote — treating it as an unreadable
/// header leaves a broken machine broken over a line TOML itself rejects, and
/// round 5 decided that the other way. A `]` present but mis-split is a header
/// we may have misread; a `]` absent is not a header at all.
///
/// Change 062 makes the bracket search quote-aware, and this predicate stops
/// being true of anything.
fn opens_unreadable_table(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with('[') && trimmed.contains(']') && table_header_shape(line).is_none()
}

/// What is still open after `line`, having entered it with `open` open.
///
/// One walk rather than a `starts` and a `continues` pair. The pair could not
/// express a value nested in another value — the very thing a multi-line array
/// of multi-line strings is — and split one rule over two functions that had to
/// agree, which is the shape of half the findings in this change.
fn advance(open: Option<OpenValue>, line: &str) -> Option<OpenValue> {
    let (depth, rest) = match open {
        Some(OpenValue::Fenced { depth, literal }) => {
            let fence = if literal { "'''" } else { "\"\"\"" };
            // Still inside it, or closed here and the line goes on at the depth
            // the string was sitting at. `escaped` does not carry across the
            // newline: TOML's line-ending backslash swallows the newline and the
            // whitespace after it, and what follows is not itself escaped.
            let Some(past) = closing_offset(line, fence, !literal) else {
                return open;
            };
            (depth as isize, &line[past..])
        }
        Some(OpenValue::Array(depth)) => (depth as isize, line),
        None => (0, opening_value_of(line)?),
    };
    scan(depth, rest)
}

/// The value part of a line that could open something, or `None` for a line
/// that could not. A comment is not an assignment: `assignment_of` reads
/// `# roots = [` as the key `# roots`, and letting a bracket in a comment open
/// an array would hide every line after it.
fn opening_value_of(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    if trimmed.starts_with('#') {
        return None;
    }
    assignment_of(trimmed).map(|(_, value)| value)
}

/// Whether a value continues past the line it started on. A line editor that
/// removes a line cannot remove one of these without orphaning the rest of it.
///
/// The same walk as everything else here, which it was NOT for one round: this
/// kept an older `starts_with('[') && !contains(']')`, so `hooks = [[],` was
/// read as a value that ends on its line. The first line was then removed and
/// the rest of the array left unattached — the exact failure the comment at its
/// call site says this helper exists to prevent. Being one expression of the
/// rule instead of a second one is what fixed it, not a better pattern.
///
/// `scripts/check-codex-config-shape.sh` reads this same shape in awk, and the
/// two are held to one acceptance set by
/// `the_config_check_and_the_writer_read_one_file_the_same_way`. Change one and
/// the other changes in the same commit.
fn opens_multiline_value(value: &str) -> bool {
    scan(0, value).is_some()
}

/// Walks the structural characters of `text` from `depth` brackets deep, and
/// says what is still open when the line ends.
///
/// Only the characters that are structure count. A bracket inside a string or
/// past a `#` is text — `paths = ["/a[1]"] # [` closes on its own line — and a
/// reader that counted those would believe an array stayed open and hide the
/// rest of the file from the scans above.
fn scan(mut depth: isize, text: &str) -> Option<OpenValue> {
    const FENCES: [(&str, bool); 2] = [("\"\"\"", false), ("'''", true)];
    let mut index = 0;
    while index < text.len() {
        let rest = &text[index..];
        if let Some((fence, literal)) = FENCES
            .into_iter()
            .find(|(fence, _)| rest.starts_with(fence))
        {
            let body = &rest[fence.len()..];
            let Some(past) = closing_offset(body, fence, !literal) else {
                return Some(OpenValue::Fenced {
                    depth: depth.max(0) as usize,
                    literal,
                });
            };
            index += fence.len() + past;
            continue;
        }
        let Some(character) = rest.chars().next() else {
            break;
        };
        index += character.len_utf8();
        match character {
            // An unterminated one-line string consumes the rest of the line,
            // which is the safe answer: what follows it is text, not structure.
            quote @ ('"' | '\'') => {
                let (delimiter, escapes) = if quote == '"' {
                    ("\"", true)
                } else {
                    ("'", false)
                };
                let body = &text[index..];
                index += closing_offset(body, delimiter, escapes).unwrap_or(body.len());
            }
            '#' => break,
            '[' => depth += 1,
            ']' => depth -= 1,
            _ => {}
        }
    }
    (depth > 0).then_some(OpenValue::Array(depth as usize))
}

/// The byte offset just past the closing `delimiter` in `text`, or `None` if
/// the text runs out before it closes.
///
/// One walk for all four quoted things TOML has. The delimiter is a `&str`
/// rather than a `char` because the only difference between the one-line kinds
/// and the multi-line ones is its length, and `escapes` is a parameter because
/// the two LITERAL kinds have none at all — inside `'` or `'''` a backslash is
/// a backslash, which is why Windows paths are written that way.
///
/// This rule used to live here for the one-line kinds and as a plain
/// `find(fence)` for the multi-line ones, and the `find` did not know about
/// escapes. That is not a corner: `\"""` is the ONLY way TOML lets somebody
/// write a literal `"""` inside a `"""` string, so it is what a note explaining
/// Operon's own config format contains. Review reproduced three losses through
/// it — the flag written into the note, a line deleted out of one, and the
/// block-dropping loop eating the closing fence for an `Unterminated string`.
///
/// So this is a MOVE and not a copy, on purpose. Four rounds of this change
/// have now failed the same way: a rule with a second expression somewhere
/// else, and the gap exactly where the two meet.
fn closing_offset(text: &str, delimiter: &str, escapes: bool) -> Option<usize> {
    let mut escaped = false;
    let mut index = 0;
    while index < text.len() {
        let rest = &text[index..];
        if !escaped && rest.starts_with(delimiter) {
            return Some(index + delimiter.len());
        }
        let character = rest.chars().next()?;
        escaped = !escaped && escapes && character == '\\';
        index += character.len_utf8();
    }
    None
}

/// The name of a line that is a table header *and* carries a name TOML would
/// accept. The strict reader.
fn table_header_name(line: &str) -> Option<&str> {
    table_header_shape(line)
        .map(|(name, _)| name)
        .filter(|name| is_key_path(name))
}

/// Whether a string is the dotted key path a TOML table header holds: segments
/// joined by dots **outside quotes**, each segment either quoted or made of
/// bare-key characters.
///
/// Splitting on every dot is the bug this signature exists to describe.
/// `profiles."gpt-5.6"` is ONE table with two segments; split naively it
/// becomes `profiles`, `"gpt-5`, `6"`, the last two of which are neither quoted
/// nor bare, so the line stops being a header at all. Operon writes
/// `[hooks.state."…/hooks.json:stop:1:0"]` itself — dots inside the quotes,
/// hundreds of them on a working machine — so this is its own output, not an
/// exotic input.
fn is_key_path(name: &str) -> bool {
    fn segment_is_valid(segment: &str) -> bool {
        let trimmed = segment.trim();
        let quoted = (trimmed.starts_with('"') && trimmed.ends_with('"'))
            || (trimmed.starts_with('\'') && trimmed.ends_with('\''));
        (quoted && trimmed.len() >= 2)
            || (!trimmed.is_empty()
                && trimmed.chars().all(|character| {
                    character.is_ascii_alphanumeric() || matches!(character, '_' | '-')
                }))
    }

    let Some(segments) = key_segments(name) else {
        // An unterminated quote is not a name anybody meant to write, and
        // treating it as one is how a line editor walks into the middle of a
        // string.
        return false;
    };
    segments.iter().all(|segment| segment_is_valid(segment))
}

/// A dotted key path split into its segments, or `None` if a quote never
/// closes.
///
/// The walk carries each segment's START, not its characters, so every segment
/// is a slice of `name` — cheaper, and one less thing that can disagree with
/// the string it was copied from.
///
/// This exists as a function because it was about to be written a fourth time.
/// `is_key_path` had it, `might_bind` had it again for the first
/// segment, and `can_hold_hook_state` needs the first TWO — and every finding
/// in this change that was not a misread was one rule with a second expression
/// somewhere else. Splitting on every dot is the bug the walk exists to avoid:
/// `profiles."gpt-5.6"` is ONE table with two segments, and Operon writes
/// `[hooks.state."…/hooks.json:stop:1:0"]` itself, hundreds of times on a
/// working machine.
fn key_segments(name: &str) -> Option<Vec<&str>> {
    let mut segments = Vec::new();
    let mut start = 0;
    let mut quote: Option<char> = None;
    for (index, character) in name.char_indices() {
        match quote {
            Some(open) if character == open => quote = None,
            Some(_) => {}
            None if character == '"' || character == '\'' => quote = Some(character),
            None if character == '.' => {
                segments.push(&name[start..index]);
                start = index + character.len_utf8();
            }
            None => {}
        }
    }
    quote.is_none().then(|| {
        segments.push(&name[start..]);
        segments
    })
}

/// A key-path segment with the space around it and its quotes taken off, the
/// way every reader in this file compares one.
///
/// It does NOT trim inside the quotes, and the removal filter in
/// `apply_codex_config` states why at length: `" hooks "` is a key TOML calls
/// ` hooks `, a different key from `hooks`. This function used to trim twice,
/// which made the two readers of that one question disagree — and the one that
/// disagreed was the one that DELETES, so `[hooks.state."<our key> "]`, a table
/// TOML says is not ours, was dropped with its contents. Unreachable from
/// Operon's own keys, which begin with `/` and end with a digit; fixed anyway,
/// because the file holding both answers is the shape that has produced a
/// finding in six of the last seven rounds.
fn unquote_segment(segment: &str) -> &str {
    segment.trim().trim_matches(['"', '\''])
}

/// Whether a line is a table header by NAME as well as by shape. The strict
/// reader, used where reaching too far is the cheap mistake.
fn is_table_header(line: &str) -> bool {
    table_header_name(line).is_some()
}

/// Whether a line is a table header by SHAPE alone. The generous reader, used
/// where reaching too far is the expensive mistake.
///
/// The two exist because a scan's boundary has a direction that costs more, and
/// this file's two scans point opposite ways:
///
///   `end`, the [features] body       reaching too far only makes the veto
///                                    decline to write — a flag that is not
///                                    set. So: the STRICT reader, which
///                                    recognises fewer lines as headers and
///                                    therefore runs long when unsure.
///   `first_table`, the top-level     reaching too far DELETES a key belonging
///                                    to another table. So: the GENEROUS
///                                    reader, which recognises more lines as
///                                    headers and therefore stops early when
///                                    unsure.
///
/// This is `set_features_hooks`'s asymmetry applied to where a range stops
/// rather than to whether a line is written. Review found the cost of getting
/// it backwards: with one reader for both, a dot inside a quoted name made
/// `first_table` run to end-of-file and take somebody's `hooks = true` with it,
/// and the output still parsed.
fn looks_like_table_header(line: &str) -> bool {
    table_header_shape(line).is_some()
}

/// Whether a line might open the table called `name`, without claiming to
/// decide that it does. The deliberately generous half of the pair with
/// `opens_table`: it answers "could I be about to duplicate something?", where
/// a false yes costs a flag that is not set and a false no costs a config Codex
/// cannot load.
///
/// Generous about the spelling of the name — quotes come off, so `["features"]`
/// is the same table TOML says it is — and strict about which table it is. A
/// dotted child is a DIFFERENT table and `[features]` may be appended beside it
/// legally, so `[features.context_management]` has to answer no. The first
/// version asked whether the header text merely contained the name, which said
/// yes to that child and stopped the ordinary config from ever getting its
/// flag. Caught by `the_config_check_and_the_writer_read_one_file_the_same_way`
/// on the case it already carried.
fn might_open_table(line: &str, name: &str) -> bool {
    // Shape, not name: a false no here appends a second `[features]` beside one
    // this file failed to parse the name of, which is the expensive direction.
    table_header_shape(line).is_some_and(|(found, _)| unquote_segment(found) == name)
}

/// Whether a line might bind something AT `name` — the name itself, or
/// anything underneath it. Only the FIRST segment of the key path is compared,
/// quote-aware, so `hooks = true`, `hooks = { x = 1 }`, `hooks.state.inner =
/// true` and `"hooks".x = 1` are one answer and `hooks_enabled = true` is not.
///
/// Both vetoes in `set_features_hooks` ask this, and for two rounds they asked
/// it with two different functions:
///
///   - at the root, a dotted key or an inline table DEFINES `features` without
///     a header, and appending `[features]` beside either is `Cannot declare
///     features twice`;
///   - inside `[features]`, a dotted key binds `hooks` as a TABLE, and
///     inserting `hooks = true` beside it is `dotted key hooks attempted to
///     extend non-table type (boolean)`.
///
/// Same rule, same failure, and the second call site kept a `might_assign` that
/// compared the WHOLE key path until review round 12. Two lines of valid TOML
/// — `[features]` and `hooks.state.inner = true` — produced a config Codex
/// cannot load, and the shape script agreed with the writer while both were
/// wrong, so nobody was told. That is the sixth time in this change that one
/// rule had a second expression and the defect was in the copy nobody revisited;
/// the whole-path reader is deleted rather than corrected, because a corrected
/// duplicate is still a duplicate.
///
/// The spelling that matters most is the dotted one, because it is the shape
/// this change TEACHES: a person who reads "the flag lives at `features.hooks`"
/// and writes that line by hand used to lose their whole Codex configuration to
/// Operon for it, on the first frame after launch.
///
/// The name says "might" for the reason `might_open_table` does: a false yes
/// costs a flag that is not set, a false no costs a machine whose Codex will
/// not start, and this is the reader that gets to be generous.
fn might_bind(line: &str, name: &str) -> bool {
    assignment_of(line)
        .and_then(|(key, _)| key_segments(key))
        .is_some_and(|segments| unquote_segment(segments[0]) == name)
}

/// The tables `[hooks.state."K"]` needs on its way to the key. Both are created
/// implicitly by that header, and TOML will only create one where nothing else
/// has already bound the name.
const HOOK_STATE_PATH: &[&str] = &["hooks", "state"];

/// The full path one trust block declares: `hooks.state.<key>`, unquoted the
/// way every reader here compares a segment.
///
/// One function because three places now need this path and the file's whole
/// history is one rule written twice. The drop loop asks "is this header the
/// block I am about to replace?", the veto asks "is anything standing where
/// that block goes?", and the append writes it.
fn hook_state_path(key: &str) -> Vec<&str> {
    let mut path: Vec<&str> = HOOK_STATE_PATH.to_vec();
    path.push(key);
    path
}

/// Whether an assignment stands where `target` has to be declared.
///
/// `bound` is where the assignment lands — the open table, then its own dotted
/// key — and `table_depth` is how much of that came from the table header. The
/// split matters, because the two ways to be in the way are not symmetric and
/// TOML treats them differently:
///
///   - **Binding a prefix.** `hooks.state = "x"` puts a value where we needed a
///     table. Always fatal, wherever the line is.
///   - **Reaching past it with a DOTTED KEY.** `hooks.state."K".note = "x"`
///     creates `hooks.state.K` implicitly, and a later `[hooks.state."K"]` may
///     not redefine it — `Cannot declare … twice`. A table HEADER that reaches
///     past it does NOT do this: `[hooks.state."K".sub]` is legal above our
///     block, because defining a super-table after its sub-table is allowed.
///     Both were measured; an earlier draft of this function had one rule for
///     both and refused the legal one, costing the blocks for nothing.
///
/// So the second test asks whether the boundary falls inside the line's own key
/// path rather than in the header above it. Neither test firing is the allowed
/// case, and it is the one the earlier rounds kept over-reaching on:
/// `hooks.state.inner`, `hooks.state."other"` and `hooks.state."Kx"` all differ
/// from ours at a segment, so they are other tables and nothing is in the way.
fn stands_in_the_way(bound: &[&str], target: &[&str], table_depth: usize) -> bool {
    target.starts_with(bound) || (bound.starts_with(target) && target.len() > table_depth)
}

/// Whether `[hooks.state."…"]` can be appended to these lines at all.
///
/// The trust blocks are the last thing `apply_codex_config` writes, and for ten
/// rounds they were written unconditionally — so every shape the function
/// carefully decided to PRESERVE above was a shape the append then broke. Not
/// appending costs a hook that is registered and not trusted, so it does not
/// fire. Appending into a name TOML cannot extend costs the whole file: Codex
/// loads NO configuration, which is the defect this change exists to stop.
///
/// Round 11 asked this of the first path segment only, which is the same
/// "closed the spelling, not the shape" mistake the change had already been
/// caught at four times, one segment further in. Round 12 widened it to every
/// proper prefix of `hooks.state` — and round 13 found the LEAF still unasked
/// about by anybody, which is the same gap once more at the only end of the
/// path nobody had looked at. A document may bind `hooks.state.inner`, or
/// `hooks.state."someone else's key"`, or open `[hooks.state]`, and all three
/// leave our table free; but `hooks.state."K" = "x"`, `[[hooks.state."K"]]`
/// and `hooks.state."K".note = "x"` each stand exactly where the block goes.
///
/// So the question is asked of the WHOLE path now, per key, with
/// `stands_in_the_way` carrying the rule in one place. Ten shapes were run
/// against `tomllib` before
/// this was written; nine break and the tenth — `"K" = "x"` under a `[hooks]`
/// header, which is `hooks.K` and not `hooks.state.K` — does not, and is
/// allowed here for that reason rather than by accident.
///
/// All-or-nothing across `entries` rather than per key. A half-written trust
/// set is a state nothing else in this module has a reading for, the entries
/// come from one `hooks.json` and are trusted as a unit, and the cost of
/// refusing is the cheap one either way: registered, untrusted, silent.
///
/// It re-walks rather than reusing the caller's `outside`, because the caller
/// has removed and inserted lines since it asked, and an index from before an
/// edit is a claim about a different document.
fn can_hold_hook_state(lines: &[String], entries: &[(String, String)]) -> bool {
    // A key this file cannot write and read back as the same string is a key it
    // does not write at all.
    //
    // The header is built with `format!("[hooks.state.\"{key}\"]")` and the key
    // is a filesystem path — `<hooks.json>:<event>:<group>:<handler>` — so the
    // characters a TOML basic string gives meaning to arrive from outside. All
    // three were measured: a `"` in the path makes the header unparseable, a
    // newline likewise, and `\b` is the quiet one — TOML reads it as a
    // BACKSPACE, so the key Codex stores is not the key Operon later looks for,
    // the hook is registered, never trusted, never fires, and nothing says so.
    //
    // Refusing rather than escaping, and that is a deliberately smaller fix
    // than the one available. Writing `\"` would make this the only place in
    // the file that ENCODES a quoted key while five readers decode one by
    // stripping quotes and not unescaping — a new seam between a writer and its
    // readers, which is the exact shape that has produced a finding in five of
    // the last six rounds. Change 062 already holds the shared-reader work and
    // holds this with it. Until then the cost is the cheap one this function
    // already pays everywhere: registered, untrusted, silent.
    let writable = |key: &str| {
        !key.chars()
            .any(|character| character.is_control() || character == '"' || character == '\\')
    };
    if !entries.iter().all(|(key, _)| writable(key)) {
        return false;
    }
    let targets: Vec<Vec<&str>> = entries
        .iter()
        .map(|(key, _)| hook_state_path(key))
        .collect();
    let (outside, _) = structural_lines(lines);
    // The table every following assignment hangs off, innermost last. Empty is
    // the document root.
    let mut table: Vec<&str> = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        if !outside[index] {
            continue;
        }
        if let Some((name, array)) = table_header_shape(line) {
            // A name whose quote never closes was misread, and the rest of this
            // walk would be attributing keys to a table it invented. Refusing
            // is the cheap direction, the same one the early return takes.
            let Some(segments) = key_segments(name) else {
                return false;
            };
            table = segments.into_iter().map(unquote_segment).collect();
            // Only an array of tables, and only one at or above our path. A
            // plain header DEFINES a table, which is what we want; an array
            // binds a list, and TOML then hangs our block off its last element
            // — valid, and Operon's trust state filed inside somebody's data.
            // `[[hooks.state."K".sub]]` is BELOW us and measured harmless, so
            // the prefix test runs one way here and both ways for assignments.
            if array && targets.iter().any(|target| target.starts_with(&table)) {
                return false;
            }
            continue;
        }
        // A header this cannot read means this walk does not know what the
        // document declares, so it cannot know whether the block it is about
        // to append is a second declaration of the same table. The branch
        // above says the same thing for a name whose quotes never close.
        //
        // Round 15 put this here and nowhere else, which was the right answer
        // in one of the four places that needed it: `first_table` above runs
        // past the same line and DELETES under it. `opens_unreadable_table`
        // is the shared question now; the two answers stay separate because
        // they are separate decisions, one a boundary and one a veto, which is
        // lesson 030's rule for every other pair of readers in this file.
        if opens_unreadable_table(line) {
            return false;
        }
        let Some((key, _)) = assignment_of(line) else {
            continue;
        };
        let Some(segments) = key_segments(key) else {
            return false;
        };
        // Where this assignment actually lands: the open table, then the dotted
        // key path written on the line. `state = "x"` under `[hooks]` and
        // `hooks.state = "x"` at the root are one document to TOML, and were two
        // separate misses before they were one expression.
        let bound: Vec<&str> = table
            .iter()
            .copied()
            .chain(segments.into_iter().map(unquote_segment))
            .collect();
        if targets
            .iter()
            .any(|target| stands_in_the_way(&bound, target, table.len()))
        {
            return false;
        }
    }
    true
}

/// Sets `hooks = true` inside `[features]`, adding the table if it is absent and
/// leaving an existing setting — another tool's, with its own markers around it
/// — exactly where it is. Writing a second one would be a duplicate key, which
/// is the same class of invalid file this function was fixed for.
///
/// **When recognition fails, this writes nothing.** That is the rule, and it is
/// here because three review rounds were spent closing one reported spelling at
/// a time — a trailing comment, then space inside the brackets, then a quoted
/// name and a multi-line array — each fix reproducing the failure it fixed. The
/// fourth round named the shape instead of the count, and it is a question of
/// which way the code falls when it does not understand the file:
///
///   nothing written  the flag is not set, so Operon's hooks are registered and
///                    never fire. Reversible, visible, and
///                    `scripts/check-codex-config-shape.sh` says
///                    `features.hooks: not set` in so many words.
///   written anyway   a duplicate table or a duplicate key, and Codex loads NO
///                    configuration at all — every session on the machine,
///                    whether or not Operon launched it, on the first frame
///                    after launch because `ensure_hooks_started` applies then.
///
/// Those are not comparable costs, so the generous readers above get the last
/// word over the strict one. A spelling nobody has thought of yet lands in the
/// first row rather than the second, which is the only part of this that keeps
/// working without another review round.
fn set_features_hooks(lines: &mut Vec<String>) {
    // Two kinds of question are asked below, and only one of them consults
    // `structural_lines`. Deciding WHERE the table is must ignore a `[features]`
    // that is really text inside a multi-line string, because inserting the flag
    // under it is a duplicate key and the failure this whole change exists to
    // stop. The two VETOES must not: a `hooks = true` sitting inside somebody's
    // note is not an assignment, but treating it as one only declines to write,
    // while overlooking a real one writes the duplicate. So the vetoes read
    // every line, and the boundaries read the structural ones.
    //
    // This choice was recorded once with a false sentence beside it — that the
    // cost is not observable in valid TOML. It is. A `[features]` body holding
    // a multi-line string with a `hooks = …`-shaped line in it makes the veto
    // fire, and the flag is never written: the document parses, the person's
    // configuration is untouched, and Operon's hooks are registered and
    // silently never run. The trade is still the right way round — nothing
    // written beats a duplicate key — but it is a real cost paid by a real
    // file, and the only reason it stays invisible is that nothing tells the
    // person. Change 063 is that notice, and it covers this case as well as the
    // "could not read the file" one: "read it and decided not to write" is the
    // same sentence to whoever is looking at a hook that never fires.
    let (outside, _) = structural_lines(lines);
    let structural = |index: &usize| outside[*index];
    let Some(header) = (0..lines.len())
        .filter(structural)
        .find(|index| opens_table(&lines[*index], "features"))
    else {
        // No header recognised. Before appending one, ask the generous readers
        // whether this file already has something TOML would call the same
        // table: a header it could not parse the name of (`["features"]`), or
        // either of the two header-less ways of defining one (`features.hooks =`
        // and `features = { … }`). Appending beside any of them is `Cannot
        // declare features twice`.
        //
        // The tradeoff is chosen rather than inherited: reading EVERY
        // line means a `features.hooks = true` that sits under `[other]` — a
        // different table, which `[features]` could legally be appended beside —
        // also vetoes, and that file does not get its flag. That is the cheap
        // direction and the one the other two vetoes already take; the
        // expensive direction is `Cannot declare features twice`.
        if lines
            .iter()
            .any(|line| might_open_table(line, "features") || might_bind(line, "features"))
        {
            return;
        }
        while lines.last().is_some_and(|line| line.trim().is_empty()) {
            lines.pop();
        }
        if !lines.is_empty() {
            lines.push(String::new());
        }
        lines.push("[features]".to_owned());
        lines.push("hooks = true".to_owned());
        return;
    };
    // One scan, and it deliberately ends at a real table header rather than at
    // the first line starting with `[`. A multi-line array's continuation row
    // satisfies the latter, so the key could be sitting just past a
    // `matrix = [ … ]` and be invisible — which is how a second `hooks = true`
    // got inserted into a table that already had one.
    //
    // There were two scans here for one round, a strict one over the narrow
    // range and this one after it. The strict one could not be made to fail:
    // the veto tests the same key identity over a superset of the lines, so it
    // answered yes wherever the strict scan did, and the mutation for it came
    // back green. Subsumed, so removed — a guard whose mutation cannot fail is
    // not a guard, and two readers of one thing is how this file got into
    // review three times.
    let end = (header + 1..lines.len())
        .filter(structural)
        .find(|index| is_table_header(&lines[*index]))
        .unwrap_or(lines.len());
    // `might_bind`, not a whole-key comparison, and this is the call site that
    // kept one until round 12. `hooks.state.inner = true` under `[features]`
    // binds `hooks` as a TABLE; inserting `hooks = true` beside it is `dotted
    // key hooks attempted to extend non-table type (boolean)`. Two lines of
    // valid TOML in, a Codex that will not start out — and the shape script
    // reported `features.hooks: not set` and exited 0 about the same file, so
    // the two implementations agreed and nobody was told.
    if lines[header + 1..end]
        .iter()
        .any(|line| might_bind(line, "hooks"))
    {
        return;
    }
    lines.insert(header + 1, "hooks = true".to_owned());
}

/// Enables the hooks feature under `[features]`, replaces the trust blocks whose
/// keys we own, removes a top-level `hooks` SCALAR an older Operon wrote — and
/// only a scalar, because an inline table there is somebody's working
/// configuration — and leaves everything else byte-for-byte.
///
/// Line-based rather than parsed: `config.toml` is a large hand-edited file with
/// comments and ordering the person chose, and a round trip through a TOML
/// serializer would rewrite all of it. The only edits are an inserted line and
/// whole blocks appended or replaced by exact key. That argument is about the
/// input, and for a long time it was also allowed to excuse never reading the
/// output back: `codex_hook_install_writes_a_config_codex_can_load` now parses
/// what this returns, with the `toml` dev-dependency added for no other purpose.
///
/// **`hooks` at the top level is a table, not a flag.** Codex declares it as
/// `HooksToml`, so the scalar this function used to insert at line 0 was the
/// wrong type for the schema and Codex refused to load the file at all — two
/// different ways, depending on what else was present: `cannot extend value of
/// type boolean with a dotted key` once this function's own
/// `[hooks.state."…"]` blocks followed it, and `invalid type: boolean `true`,
/// expected struct HooksToml` when they did not. The flag lives at
/// `features.hooks`, which is where every other tool writing this file puts it.
/// Removing the bad line is part of the fix and not tidying: Operon wrote it, so
/// a machine already broken by it stays broken until Operon takes it back out.
pub(crate) fn apply_codex_config(
    existing: &str,
    enable_hooks: bool,
    remove_keys: &[String],
    entries: &[(String, String)],
) -> String {
    let mut lines: Vec<String> = existing.lines().map(str::to_owned).collect();
    // **One rule, before anything is touched: a file this cannot read to the end
    // is a file it does not edit.** A document that never closes what it opened
    // was misread somewhere, and every edit below assumes the reading was right
    // — the appended `[features]` would land inside whatever is still open, the
    // appended blocks after it, and a removal would take a line out of text
    // nobody can see the extent of.
    //
    // This began as a filter on the removal alone, and review found the rule
    // stated more widely in the comment than in the code twice running. It is
    // one early return now because that is the only shape that cannot drift
    // from its own description.
    let (outside, understood) = structural_lines(&lines);
    if !understood {
        return existing.to_owned();
    }
    // The other way to not understand a document, asked once for the whole
    // file because that is the grain two of its three readers need. See
    // `opens_unreadable_table`: `first_table` wants the POSITION and asks per
    // line, the drop loop and `can_hold_hook_state` want to know whether the
    // document holds one at all, and they have to agree — one of them deletes
    // on the strength of the other one appending.
    //
    // This is deliberately not a second early return. Round 16 built that and
    // measured what it costs: it declines to take out the top-level `hooks`
    // scalar, which is the broken machine this whole change exists to repair.
    // Reading a document imperfectly is a reason to write less, not a reason
    // to write nothing.
    let holds_unreadable_table = lines
        .iter()
        .zip(&outside)
        .any(|(line, outside)| *outside && opens_unreadable_table(line));
    // Drop every block whose header names a key we are replacing or removing.
    //
    // This loop asked `trimmed.starts_with('[')` and consulted nothing else for
    // six rounds, while every other scan in the file learned to. Review reached
    // it with a `[hooks.state."K"]` that was somebody's PROSE — a person pasting
    // Operon's own output into a `note = """ … """` to explain it — and the loop
    // dropped from there to the next `[` line, taking the rest of their text and
    // the closing fence with it. Valid TOML in, `invalid multiline basic string`
    // out. There are two paths in this function that delete a line, and only one
    // of them had been given the six rounds of care.
    //
    // Round 13 found the OTHER half of that same loop still unreformed: it
    // compared the header as raw text against `[hooks.state."{key}"]`, so
    // Operon's own block, reformatted, was a block it no longer recognised.
    // `[ hooks.state."K" ]`, `[hooks . state . "K"]` and `[hooks.state.'K']`
    // are the same table to TOML, were dropped by nobody, vetoed by nobody, and
    // the append then declared the table a second time — `Cannot declare
    // ('hooks','state','K') twice`, and Codex loads no configuration at all.
    // Those are `set_features_hooks`'s first three findings exactly: space
    // inside the brackets, a differently quoted name, whitespace around the
    // structure. Six rounds closed them there and this reader never had one.
    //
    // A DOUBLE-bracket header is deliberately not dropped. `[[hooks.state."K"]]`
    // is an array of tables and not a block this ever wrote, so deleting it
    // would be taking somebody's data; `can_hold_hook_state` refuses the append
    // instead. Same split as everywhere else here — the reader that deletes is
    // strict, the reader that vetoes is generous.
    //
    // Round 17: and the strict reader now also asks the generous one's
    // question, because on the ENABLING path the two are one transaction.
    // Dropping is the first half of a replacement whose second half
    // `can_hold_hook_state` can veto, so every reason IT refuses for is a
    // reason this must not start. Deleting beside a veto is somebody's trust
    // state gone, and the same call that deleted it declined to write it back.
    //
    // The REMOVING path stands still too, and it needs its own argument,
    // because none of the above applies to it: nothing is appended, nothing
    // vetoes, and the drop is the whole request. The argument is the wider
    // one — we did not re-derive this data and we cannot, so we leave it.
    //
    // What is left behind then is NOT cleaned up on a later launch, and an
    // earlier draft of this comment claimed it was. It is not: with
    // `remove_only`, `install_event_groups` writes nothing, so `keys` is empty
    // and the caller stores `codex_trust_keys: []` — the same call that
    // decided to keep the blocks forgets their names, so `owned` cannot reach
    // them again however readable the document becomes. They stay until hooks
    // are turned back ON, which regenerates the keys from `entries`. Review
    // measured six blocks surviving a launch that had the header readable
    // again, against a control that cleaned up when the keys were remembered.
    //
    // Put as a cost rather than as a mechanism: the removal path gets exactly
    // one attempt, and this gate spends it. That is the sentence to weigh the
    // trade against, and it is why the argument below has to be about the
    // data rather than about a later launch putting it right.
    //
    // That is acceptable, for reasons that are about the data and not about a
    // future repair: what remains is inert. The registration in `hooks.json`
    // is gone, so Codex never matches these entries, and a stale
    // `trusted_hash` can only ever fail closed. Nobody's own configuration is
    // touched. Leaving inert rows beats deleting live ones on a guess.
    let owned: Vec<Vec<&str>> = remove_keys
        .iter()
        .chain(entries.iter().map(|(key, _)| key))
        .map(|key| hook_state_path(key))
        .collect();
    let mut kept: Vec<String> = Vec::with_capacity(lines.len());
    let mut dropping = false;
    for (index, line) in lines.drain(..).enumerate() {
        if outside[index] && line.trim_start().starts_with('[') {
            dropping = !holds_unreadable_table
                && table_header_shape(&line).is_some_and(|(name, array)| {
                    !array
                        && key_segments(name).is_some_and(|segments| {
                            let path: Vec<&str> =
                                segments.into_iter().map(unquote_segment).collect();
                            owned.contains(&path)
                        })
                });
        }
        if !dropping {
            kept.push(line);
        }
    }
    lines = kept;
    // Whatever else happens, the SCALAR an older Operon put above the first
    // table header goes. It is invalid against Codex's schema whether or not
    // this call is enabling anything, so it is removed on the remove-only path
    // too — that path is how a person turns Operon's hooks off, and it would be
    // the one moment to leave the machine unable to start Codex.
    //
    // A value opening with `{` is an inline table, which is exactly the shape
    // Codex's `HooksToml` expects, so it is somebody's working configuration and
    // not the line this change exists to take back out. Review found the first
    // version deleting it: the doc comment said "scalar" and the code asked only
    // for the key, so a person who had configured Codex's hooks by hand lost
    // them by starting Operon — `ensure_hooks_started` applies on the first
    // frame, so it needed no action from them at all. Repairing our own mistake
    // is the licence here, and it does not extend one line past it.
    // The GENEROUS reader, and this is the one boundary in the file where that
    // is the right choice. Everything below this index is treated as top-level
    // and a `hooks` scalar found there is deleted, so a scan that reaches too
    // far deletes a key belonging to somebody's table — the shape review found
    // when a dot inside a quoted name (`[profiles."gpt-5.6"]`, and the
    // `[hooks.state."…"]` blocks Operon writes itself) stopped counting as a
    // header and this ran to end-of-file. Stopping too EARLY only means a
    // top-level scalar goes unrepaired, which is the mild half.
    //
    // Its neighbour in `set_features_hooks` takes the strict reader for the
    // mirror-image reason. Neither is "the right reader"; the direction the
    // mistake costs is what picks one.
    //
    // Read again rather than reused: the drop loop above removed lines, so the
    // indices the first walk answered for are not these lines' indices. Dropping
    // whole blocks cannot open a value that was closed, so the second answer is
    // still `understood` and is not re-asked.
    let (outside, _) = structural_lines(&lines);
    // …and generous includes a header it cannot READ. `looks_like_table_header`
    // answers no for `[projects."/Users/me/[work]"]`, because the bracket
    // search stops at the `]` inside the quoted name, and this scan then ran to
    // end-of-file and took the person's `hooks` key out of their projects
    // entry. Same shape as the dotted name above, one character apart, and the
    // dotted one was fixed rounds ago. Stopping at a line this cannot read
    // costs nothing that matters: a top-level scalar precedes every header by
    // definition, so the region that needs repairing is always above it.
    let first_table = (0..lines.len())
        .find(|index| {
            outside[*index]
                && (looks_like_table_header(&lines[*index])
                    || opens_unreadable_table(&lines[*index]))
        })
        .unwrap_or(lines.len());
    // A value that opens a multi-line construct is left alone for the same
    // reason, one step further out: this removes whole LINES, so taking the
    // first line of a `hooks = [` … `]` or a `hooks = """` leaves the rest of it
    // unattached and turns a file Codex rejects by type into one it rejects by
    // syntax. Neither loads, so nothing is gained by reaching for it, and the
    // direction that costs least when the reader is wrong is to stay out —
    // which is the same rule `set_features_hooks` follows.
    //
    // The removal's other veto is the early return at the top of this function:
    // it gets no licence at all in a file that was not read to the end. For six
    // rounds it had neither, while `set_features_hooks` had two, and review's
    // three reproductions landed two of their losses on this side — the side
    // that deletes.
    let removing: Vec<usize> = (0..first_table)
        .filter(|index| outside[*index])
        .filter(|index| {
            assignment_of(&lines[*index]).is_some_and(|(key, value)| {
                // Unquoted, so `"hooks" = true` is the same key. Comparing the
                // raw text made that a spelling the repair walked past —
                // benign, because Operon never wrote it, but two readers of one
                // question is how most of this file's findings started.
                //
                // This is deliberately NOT `might_bind`, which is the one place
                // in this file where the WHOLE key path is the right
                // comparison. `might_bind` is generous because its callers
                // decline to write; this branch DELETES a line, and a dotted
                // `hooks.enabled = true` is somebody's real table that the
                // trust blocks go on to extend. Generous here would take it
                // out. The direction the mistake costs picks the reader, as it
                // does at every other boundary in this file.
                //
                // It does not re-trim after unquoting either. `assignment_of`
                // already trimmed the key, so a second `.trim()` could only
                // change the answer for `" hooks "`, which TOML says is a key
                // named ` hooks ` and not this one. The redundancy also hid a
                // mutation: with both trims in place, taking the one out of
                // `assignment_of` changed nothing anywhere and came back green.
                key.trim_matches(['"', '\'']) == "hooks"
                    // A value opening with `{` is an inline table — the shape
                    // Codex's own schema asks for, somebody's working
                    // configuration — and a value that runs past its line cannot
                    // be taken out a line at a time without orphaning the rest
                    // of it. Both stay.
                    && !value.starts_with('{')
                    && !opens_multiline_value(value)
            })
        })
        .collect();
    for index in removing.into_iter().rev() {
        lines.remove(index);
    }
    if enable_hooks {
        set_features_hooks(&mut lines);
    }
    while lines.last().is_some_and(|line| line.trim().is_empty()) {
        lines.pop();
    }
    // Asked of the lines as they now stand, not as they arrived: the removal
    // above may have taken the only thing that was in the way, and
    // `set_features_hooks` may have added a table. The whole argument for the
    // question is on `can_hold_hook_state`.
    if can_hold_hook_state(&lines, entries) {
        for (key, hash) in entries {
            lines.push(String::new());
            lines.push(format!("[hooks.state.\"{key}\"]"));
            lines.push("enabled = true".to_owned());
            lines.push(format!("trusted_hash = \"{hash}\""));
        }
    }
    let mut text = lines.join("\n");
    text.push('\n');
    text
}

pub(crate) fn install_codex_hooks(
    data_file: &Path,
    root: &Path,
    previous_keys: &[String],
    remove_only: bool,
) -> Result<Vec<String>> {
    let hooks_path = codex_hooks_in(root);
    if remove_only && !hooks_path.exists() && !codex_config_in(root).exists() {
        return Ok(Vec::new());
    }
    let script = hook_script_path(data_file, CliProvider::Codex);
    let mut settings = read_json_object(&hooks_path)?;
    let written = install_event_groups(&mut settings, CliProvider::Codex, &script, remove_only);
    write_json_object(&hooks_path, &settings)?;

    let mut keys = Vec::new();
    let mut entries = Vec::new();
    for (event, group, handler) in &written {
        let key = codex_trust_key(&hooks_path, event, *group, *handler);
        let hash = codex_trusted_hash(
            event,
            &hook_managed_command(&script, CliProvider::Codex, event),
            None,
        );
        keys.push(key.clone());
        entries.push((key, hash));
    }
    let config_path = codex_config_in(root);
    let existing = fs::read_to_string(&config_path).unwrap_or_default();
    let updated = apply_codex_config(&existing, !remove_only, previous_keys, &entries);
    if updated != existing {
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mode = fs::metadata(&config_path)
            .ok()
            .map(|metadata| metadata.permissions().mode());
        if config_path.exists() {
            let _ = fs::copy(&config_path, config_path.with_extension("toml.operon.bak"));
        }
        write_file_atomically(&config_path, updated.as_bytes())?;
        if let Some(mode) = mode {
            let _ = fs::set_permissions(&config_path, fs::Permissions::from_mode(mode));
        } else {
            let _ = fs::set_permissions(&config_path, fs::Permissions::from_mode(0o600));
        }
    }
    Ok(keys)
}

/// Antigravity groups its hooks by bundle name at the top level of its file.
/// Ours is one key beside anybody else's, so installing and removing is setting
/// and clearing that one key.
pub(crate) fn install_antigravity_hooks(
    data_file: &Path,
    home: &Path,
    remove_only: bool,
) -> Result<()> {
    let path = antigravity_hooks_path(home);
    if remove_only && !path.exists() {
        return Ok(());
    }
    let script = hook_script_path(data_file, CliProvider::Gemini);
    let mut settings = read_json_object(&path)?;
    settings.remove(HOOK_ANTIGRAVITY_BUNDLE);
    if !remove_only {
        let mut bundle = serde_json::Map::new();
        for (event, needs_matcher) in hook_events(CliProvider::Gemini) {
            let handler =
                managed_handler(hook_managed_command(&script, CliProvider::Gemini, event));
            let entry = if *needs_matcher {
                serde_json::json!([{ "matcher": "*", "hooks": [handler] }])
            } else {
                // Antigravity's non-tool events put the command directly in the
                // entry rather than inside a `hooks` array.
                serde_json::json!([handler])
            };
            bundle.insert((*event).to_owned(), entry);
        }
        settings.insert(
            HOOK_ANTIGRAVITY_BUNDLE.to_owned(),
            serde_json::Value::Object(bundle),
        );
    }
    write_json_object(&path, &settings)
}

/// Registers, or removes, the managed hooks for every CLI that is installed.
///
/// Every CLI is attempted independently: one unreadable settings file must not
/// stop the other two, and the failure is what the settings screen shows for
/// that row.
pub(crate) fn apply_hooks(
    data_file: &Path,
    home: &Path,
    socket: &Path,
    tools: &ToolStatus,
    enabled: bool,
    accounts: &AgentAccounts,
) -> (Vec<HookInstall>, Vec<String>) {
    let providers = [CliProvider::Codex, CliProvider::Claude, CliProvider::Gemini];
    let present: Vec<CliProvider> = providers
        .into_iter()
        .filter(|provider| tools.agent_available(provider.agent()))
        .collect();
    let settings = load_hook_settings(data_file);
    if enabled {
        if let Err(error) = write_hook_runtime(data_file, socket, &present) {
            return (
                providers
                    .into_iter()
                    .map(|provider| HookInstall {
                        provider,
                        state: HookInstallState::Failed(tf!(
                            "フックスクリプトを書き込めませんでした: {error}",
                            error = error
                        )),
                    })
                    .collect(),
                settings.codex_trust_keys,
            );
        }
    }
    let mut installs = Vec::new();
    let mut codex_keys = Vec::new();
    for provider in providers {
        let available = present.contains(&provider);
        let remove_only = !enabled || !available;
        // Every configuration root this provider's hooks belong in: the
        // machine's own, then each registered account's. Installing into only
        // the first is a switch that silently turns the state chip off.
        let mut outcome = Ok(());
        for root in account_roots(provider, home, accounts) {
            let attempt = match provider {
                CliProvider::Claude => install_claude_hooks(data_file, &root, remove_only),
                CliProvider::Gemini => install_antigravity_hooks(data_file, &root, remove_only),
                CliProvider::Codex => {
                    match install_codex_hooks(
                        data_file,
                        &root,
                        &settings.codex_trust_keys,
                        remove_only,
                    ) {
                        // Trust keys accumulate across roots: each is a key for
                        // one file, and Codex checks the file it is reading.
                        Ok(keys) => {
                            codex_keys.extend(keys);
                            Ok(())
                        }
                        Err(error) => Err(error),
                    }
                }
            };
            // The first failure is what the row reports, and the remaining
            // roots are still attempted: one unreadable settings file must not
            // leave another account without its hooks.
            if let Err(error) = attempt {
                if outcome.is_ok() {
                    outcome = Err(error);
                }
            }
        }
        let state = match outcome {
            Err(error) => HookInstallState::Failed(error.to_string()),
            Ok(()) if remove_only && !enabled => {
                HookInstallState::NotInstalled(tr("フック連携がオフです").to_owned())
            }
            Ok(()) if remove_only => HookInstallState::NotInstalled(tf!(
                "{p0} が PATH 上に見つかりません",
                p0 = provider.executable()
            )),
            Ok(()) => HookInstallState::Installed,
        };
        installs.push(HookInstall { provider, state });
    }
    let _ = save_hook_settings(
        data_file,
        &HookSettings {
            enabled,
            codex_trust_keys: codex_keys.clone(),
        },
    );
    (installs, codex_keys)
}

// ── receiving what the scripts post ─────────────────────────────────────────

/// One posted hook event, after the envelope has been read and before anything
/// has been decided about it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HookEvent {
    pub(crate) source: CliProvider,
    pub(crate) session: String,
    pub(crate) launch_token: String,
    pub(crate) event: String,
    pub(crate) payload: serde_json::Value,
}

pub(crate) fn parse_hook_envelope(body: &[u8]) -> Option<HookEvent> {
    let value: serde_json::Value = serde_json::from_slice(body).ok()?;
    let source = value.get("source")?.as_str()?;
    let source = [CliProvider::Codex, CliProvider::Claude, CliProvider::Gemini]
        .into_iter()
        .find(|provider| provider.agent() == source)?;
    let session = value.get("session")?.as_str()?.to_owned();
    if session.is_empty() {
        return None;
    }
    Some(HookEvent {
        source,
        session,
        launch_token: value
            .get("token")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        event: value
            .get("event")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        payload: value
            .get("payload")
            .cloned()
            .unwrap_or(serde_json::Value::Null),
    })
}

/// Reads one request off a connection and answers it.
///
/// A deliberately small HTTP reader: one method, one path, a `Content-Length`
/// body, and three status codes. It exists to be talked to by the script beside
/// it and by nothing else, and the socket it listens on is readable only by the
/// account that created it.
pub(crate) fn serve_hook_connection(stream: &mut UnixStream) -> Option<HookEvent> {
    stream.set_read_timeout(Some(Duration::from_secs(2))).ok()?;
    stream
        .set_write_timeout(Some(Duration::from_secs(2)))
        .ok()?;
    let mut buffer = Vec::new();
    let mut chunk = [0u8; 4096];
    let header_end = loop {
        if let Some(position) = buffer.windows(4).position(|window| window == b"\r\n\r\n") {
            break position;
        }
        if buffer.len() > HOOK_HEADER_MAX_BYTES {
            let _ = stream.write_all(
                b"HTTP/1.1 431 Request Header Fields Too Large\r\nContent-Length: 0\r\n\r\n",
            );
            return None;
        }
        match stream.read(&mut chunk) {
            Ok(0) => return None,
            Ok(read) => buffer.extend_from_slice(&chunk[..read]),
            Err(_) => return None,
        }
    };
    let head = String::from_utf8_lossy(&buffer[..header_end]).to_string();
    let mut length = 0usize;
    for line in head.lines().skip(1) {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        if name.trim().eq_ignore_ascii_case("content-length") {
            length = value.trim().parse().unwrap_or(0);
        }
    }
    if length > HOOK_BODY_MAX_BYTES {
        let _ = stream.write_all(b"HTTP/1.1 413 Payload Too Large\r\nContent-Length: 0\r\n\r\n");
        return None;
    }
    let mut body = buffer[header_end + 4..].to_vec();
    while body.len() < length {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(read) => body.extend_from_slice(&chunk[..read]),
            Err(_) => break,
        }
    }
    body.truncate(length);
    let event = parse_hook_envelope(&body);
    let response: &[u8] = if event.is_some() {
        b"HTTP/1.1 204 No Content\r\nContent-Length: 0\r\n\r\n"
    } else {
        b"HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n"
    };
    let _ = stream.write_all(response);
    let _ = stream.flush();
    event
}

/// The listening socket, owned for as long as the app runs. Dropping it stops
/// the thread and removes the socket file, so a second launch binds cleanly.
pub(crate) struct HookListener {
    socket: PathBuf,
    stopping: Arc<AtomicBool>,
}

impl HookListener {
    pub(crate) fn bind(
        socket: &Path,
        repaint: Option<egui::Context>,
    ) -> Result<(Self, Receiver<HookEvent>)> {
        // macOS gives `sockaddr_un` a 104-byte `sun_path`, one of which is the
        // terminator, and a longer path fails inside libc with an error nobody
        // can act on. Say so here instead. The real path — the data directory
        // plus `agent-hooks/hook.sock` — is about eighty.
        if socket.as_os_str().len() > 103 {
            return Err(anyhow!(tf!(
                "ソケットのパスが長すぎます: {p0}",
                p0 = socket.display()
            )));
        }
        ensure_hook_directory(socket)?;
        // A socket left behind by a process that is gone would refuse the bind.
        // Only this app's own directory is touched.
        let _ = fs::remove_file(socket);
        let listener = UnixListener::bind(socket)?;
        fs::set_permissions(socket, fs::Permissions::from_mode(0o600))?;
        let (sender, receiver) = mpsc::channel();
        let stopping = Arc::new(AtomicBool::new(false));
        let thread_stopping = Arc::clone(&stopping);
        thread::spawn(move || {
            for stream in listener.incoming() {
                if thread_stopping.load(Ordering::Acquire) {
                    break;
                }
                let Ok(mut stream) = stream else { continue };
                if let Some(event) = serve_hook_connection(&mut stream) {
                    if sender.send(event).is_err() {
                        break;
                    }
                    if let Some(repaint) = &repaint {
                        repaint.request_repaint();
                    }
                }
            }
        });
        Ok((
            Self {
                socket: socket.to_path_buf(),
                stopping,
            },
            receiver,
        ))
    }
}

impl Drop for HookListener {
    fn drop(&mut self) {
        self.stopping.store(true, Ordering::Release);
        // Unblock the accept so the thread can see the flag and end.
        let _ = UnixStream::connect(&self.socket);
        let _ = fs::remove_file(&self.socket);
    }
}

// ── what an event means ─────────────────────────────────────────────────────

/// What one hook event says about the agent, once the provider's own vocabulary
/// has been read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HookReading {
    pub(crate) source: CliProvider,
    pub(crate) activity: AgentActivity,
    /// A settled idle that is the end of a connect, a resume, a `/clear`, or a
    /// manual compaction rather than the end of a turn. It shows as idle and
    /// never notifies: nothing was finished.
    pub(crate) boundary: bool,
    pub(crate) tool: Option<String>,
    pub(crate) last_message: Option<String>,
    pub(crate) session_id: Option<String>,
    pub(crate) transcript_path: Option<String>,
}

fn payload_string(payload: &serde_json::Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        payload
            .get(*key)
            .and_then(serde_json::Value::as_str)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
    })
}

/// Whether a tool name is the one a CLI uses to put a question to the person.
/// Compared with the punctuation removed, because the same tool is spelled
/// `AskUserQuestion`, `ask_user_question`, and `askUserQuestion` across releases.
pub(crate) fn is_question_tool(tool: Option<&str>, names: &[&str]) -> bool {
    let Some(tool) = tool else { return false };
    let flattened: String = tool
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|character| character.to_ascii_lowercase())
        .collect();
    names.contains(&flattened.as_str())
}

/// Claude reruns the whole prompt through `UserPromptSubmit` when a session is
/// continued, which is not a turn a person started.
const CLAUDE_CONTINUATION_PREFIX: &str =
    "This session is being continued from a previous conversation";

pub(crate) fn normalise_hook_event(event: &HookEvent) -> Option<HookReading> {
    let payload = &event.payload;
    // A subagent's or teammate's events carry the child's id; the lead's never
    // do. A child must not own the row its parent is drawn on.
    if payload.get("agent_id").is_some_and(|id| !id.is_null()) {
        return None;
    }
    let tool = payload_string(payload, &["tool_name", "name"]).or_else(|| {
        payload
            .get("toolCall")
            .and_then(|call| payload_string(call, &["name", "toolName", "tool_name"]))
    });
    let session_id = payload_string(
        payload,
        &[
            "session_id",
            "sessionId",
            "conversationId",
            "conversation_id",
        ],
    );
    let transcript_path = payload_string(payload, &["transcript_path", "transcriptPath"]);
    let last_message = payload_string(payload, &["last_assistant_message", "lastAssistantMessage"])
        .map(|message| message.chars().take(HOOK_LAST_MESSAGE_MAX_CHARS).collect());
    let mut boundary = false;
    let activity = match event.source {
        CliProvider::Claude => match event.event.as_str() {
            "UserPromptSubmit" => {
                let prompt = payload_string(payload, &["prompt"]).unwrap_or_default();
                if prompt.starts_with(CLAUDE_CONTINUATION_PREFIX) {
                    return None;
                }
                AgentActivity::Working
            }
            "PostToolUse" | "PostToolUseFailure" => AgentActivity::Working,
            "PreToolUse" => {
                if is_question_tool(tool.as_deref(), &["askuserquestion", "requestuserinput"]) {
                    AgentActivity::AwaitingInput
                } else {
                    AgentActivity::Working
                }
            }
            "PermissionRequest" => AgentActivity::AwaitingInput,
            "Stop" | "StopFailure" => AgentActivity::Idle,
            "SessionStart" => {
                let source = payload_string(payload, &["source"]).unwrap_or_default();
                if !matches!(source.as_str(), "startup" | "resume" | "clear") {
                    return None;
                }
                boundary = true;
                AgentActivity::Idle
            }
            "PostCompact" => {
                if payload_string(payload, &["trigger"]).as_deref() != Some("manual") {
                    return None;
                }
                boundary = true;
                AgentActivity::Idle
            }
            _ => return None,
        },
        CliProvider::Codex => match event.event.as_str() {
            "UserPromptSubmit" | "PostToolUse" => AgentActivity::Working,
            "PreToolUse" => {
                if is_question_tool(tool.as_deref(), &["requestuserinput", "askuserquestion"]) {
                    AgentActivity::AwaitingInput
                } else {
                    AgentActivity::Working
                }
            }
            "PermissionRequest" => AgentActivity::AwaitingInput,
            "Stop" => AgentActivity::Idle,
            "SessionStart" => {
                boundary = true;
                AgentActivity::Idle
            }
            _ => return None,
        },
        CliProvider::Gemini => match event.event.as_str() {
            "PreInvocation" | "PostInvocation" | "PostToolUse" => AgentActivity::Working,
            "PreToolUse" => {
                if is_question_tool(tool.as_deref(), &["askquestion", "askpermission"]) {
                    AgentActivity::AwaitingInput
                } else {
                    AgentActivity::Working
                }
            }
            // `agy` reports a Stop between its own tool steps, and says so with
            // this flag. Only a fully idle Stop is the end of a turn.
            "Stop" => {
                if payload
                    .get("fullyIdle")
                    .or_else(|| payload.get("fully_idle"))
                    .and_then(serde_json::Value::as_bool)
                    == Some(false)
                {
                    AgentActivity::Working
                } else {
                    AgentActivity::Idle
                }
            }
            _ => return None,
        },
    };
    Some(HookReading {
        source: event.source,
        activity,
        boundary,
        tool: tool.filter(|_| activity == AgentActivity::Working),
        last_message,
        session_id,
        transcript_path,
    })
}

/// One managed session's state as its hooks last reported it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HookStatus {
    pub(crate) activity: AgentActivity,
    pub(crate) boundary: bool,
    pub(crate) tool: Option<String>,
    pub(crate) last_message: Option<String>,
    /// When this activity began, which is not when the last event arrived. Tool
    /// events inside one turn keep the turn's start, so a notification is keyed
    /// to the turn rather than to whichever tool ran last.
    pub(crate) started_at: Instant,
    pub(crate) received_at: Instant,
    /// The transcript `agy` said it had finished. A bookkeeping tool event
    /// arriving after that Stop must not turn the row yellow again.
    pub(crate) completed_transcript: Option<String>,
}

pub(crate) fn apply_hook_reading(
    previous: Option<&HookStatus>,
    reading: HookReading,
    now: Instant,
) -> (HookStatus, Option<ActivityNotice>) {
    // `agy` emits a tool event after the Stop that ended a turn. Recognised by
    // the transcript it names, and only for `agy`: for the others the same
    // transcript is simply the next turn of the same conversation.
    if let Some(settled) = previous.filter(|status| {
        reading.source == CliProvider::Gemini
            && reading.activity == AgentActivity::Working
            && reading.transcript_path.is_some()
            && status.completed_transcript == reading.transcript_path
    }) {
        let mut unchanged = settled.clone();
        unchanged.received_at = now;
        return (unchanged, None);
    }
    let same_activity = previous.is_some_and(|status| status.activity == reading.activity);
    let started_at = match previous {
        Some(status) if same_activity => status.started_at,
        _ => now,
    };
    let notice = if reading.boundary {
        None
    } else {
        activity_notice(previous.map(|status| status.activity), reading.activity)
    };
    let completed_transcript = if reading.activity == AgentActivity::Idle {
        reading.transcript_path.clone()
    } else {
        previous.and_then(|status| status.completed_transcript.clone())
    };
    (
        HookStatus {
            activity: reading.activity,
            boundary: reading.boundary,
            tool: reading.tool.or_else(|| {
                previous
                    .filter(|_| same_activity)
                    .and_then(|status| status.tool.clone())
            }),
            last_message: reading
                .last_message
                .or_else(|| previous.and_then(|status| status.last_message.clone())),
            started_at,
            received_at: now,
            completed_transcript,
        },
        notice,
    )
}

/// Whether a reported state still outranks what the screen says.
pub(crate) fn hook_status_is_fresh(status: &HookStatus, now: Instant) -> bool {
    now.saturating_duration_since(status.received_at)
        < Duration::from_secs(HOOK_STALE_AFTER_SECONDS)
}

/// One of Claude's usage windows, as its status line reports it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct UsageWindow {
    /// How much of the window is spent, 0 to 100.
    pub(crate) used_percent: f32,
    /// When it refills, in epoch seconds. `None` when the CLI did not say.
    pub(crate) resets_at: Option<u64>,
}

/// What Claude Code has said about the account's limits.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct RateLimits {
    pub(crate) five_hour: Option<UsageWindow>,
    pub(crate) seven_day: Option<UsageWindow>,
}

/// Read the limits out of a status-line payload.
///
/// Claude Code pipes this object to the `statusLine` command on every turn,
/// piggybacked on a response it already made — so reading it costs no request
/// of its own and no network access from here at all. Only a subscriber session
/// emits `rate_limits`, and only after its first response, so its absence is
/// ordinary and means "nothing said" rather than "nothing left".
///
/// Both spellings are accepted: `used_percentage` is what the payloads on this
/// machine carry, and `utilization` is the older name that some versions still
/// send. A window that says neither is not a reading.
pub(crate) fn parse_rate_limits(payload: &serde_json::Value) -> Option<RateLimits> {
    let limits = payload.get("rate_limits")?;
    let five_hour = parse_usage_window(limits.get("five_hour"));
    let seven_day = parse_usage_window(limits.get("seven_day"));
    if five_hour.is_none() && seven_day.is_none() {
        return None;
    }
    Some(RateLimits {
        five_hour,
        seven_day,
    })
}

fn parse_usage_window(window: Option<&serde_json::Value>) -> Option<UsageWindow> {
    let window = window?;
    let used = window
        .get("used_percentage")
        .or_else(|| window.get("utilization"))
        .and_then(serde_json::Value::as_f64)?;
    // `resets_at` is epoch seconds today and was a string in older versions.
    let resets_at = window.get("resets_at").and_then(|value| {
        value
            .as_u64()
            .or_else(|| value.as_str().and_then(|text| text.parse().ok()))
    });
    Some(UsageWindow {
        used_percent: used.clamp(0.0, 100.0) as f32,
        resets_at,
    })
}

/// The script Claude Code runs as its status line.
///
/// It writes nothing to stdout, so the line inside the terminal looks exactly
/// as it did before — this is a way of reading what the CLI already knows, not
/// a place to put a second status line.
///
/// The throttle is the interesting part. A status line runs several times a
/// second while a response streams, and posting each tick would be thousands of
/// posts for a number that moves once a turn. The script throttles on a stamp
/// file and takes its clock from the payload's own `total_duration_ms` rather
/// than spawning `date`, using POSIX parameter expansion alone, so the throttle
/// costs no process at all.
pub(crate) fn statusline_script() -> String {
    format!(
        r#"#!/bin/sh
# Operon usage reader — generated file, rewritten on every launch.
# Reads the rate limits Claude Code already reports and forwards them to the
# Operon that started this pane. Prints nothing, never fails, never blocks.
payload=$({{ command -p cat 2>/dev/null || cat; }})
if [ -z "$payload" ]; then exit 0; fi
case "$payload" in
  *'"rate_limits"'*) : ;;
  *) exit 0 ;;
esac
if [ -n "${HOOK_ENDPOINT_ENV}" ] && [ -r "${HOOK_ENDPOINT_ENV}" ]; then
  . "${HOOK_ENDPOINT_ENV}" 2>/dev/null || :
fi
if [ -z "${HOOK_SOCKET_ENV}" ] || [ -z "${HOOK_SESSION_ENV}" ]; then
  exit 0
fi
if [ ! -S "${HOOK_SOCKET_ENV}" ]; then
  exit 0
fi
# The clock comes out of the payload, so the throttle spawns nothing.
now=${{payload#*\"total_duration_ms\":}}
if [ "$now" = "$payload" ]; then
  now=0
else
  now=${{now%%[!0-9]*}}
  [ -n "$now" ] || now=0
fi
stamp="${{TMPDIR:-/tmp}}/operon-statusline-${HOOK_SESSION_ENV}"
last=0
if [ -r "$stamp" ]; then
  # `read` returns non-zero at end of file even when it assigned the value.
  # Two things stop that discarding the stamp: it is written with a trailing
  # newline, and the result is judged by what landed in the variable rather
  # than by the exit status. Either alone is enough; both together is the
  # margin, because the first version of this had neither and throttled
  # nothing — a status line ticking three times a second posted every tick.
  read -r last < "$stamp" 2>/dev/null
  case "$last" in ''|*[!0-9]*) last=0 ;; esac
fi
if [ "$now" -gt 0 ] && [ "$last" -gt 0 ] && [ $((now - last)) -lt {STATUSLINE_MIN_POST_INTERVAL_MS} ]; then
  exit 0
fi
printf '%s\n' "$now" > "$stamp" 2>/dev/null || :
printf '{{"source":"claude","session":"%s","token":"%s","event":"{STATUSLINE_EVENT}","payload":%s}}' \
  "${HOOK_SESSION_ENV}" "${HOOK_LAUNCH_ENV}" "$payload" \
  | curl -sS -X POST --unix-socket "${HOOK_SOCKET_ENV}" http://localhost/hook \
      --connect-timeout 0.5 --max-time 1.5 \
      -H 'Content-Type: application/json' \
      --data-binary @- >/dev/null 2>&1 || :
exit 0
"#,
    )
}

pub(crate) fn statusline_script_path(data_file: &Path) -> PathBuf {
    hook_directory(data_file).join(STATUSLINE_SCRIPT_FILE_NAME)
}

/// Whether Operon may put its reader in the `statusLine` slot.
///
/// Three answers, and the third is the one worth having: a slot holding
/// somebody else's command is never taken, and an **empty** slot that Operon
/// has filled before means the person emptied it deliberately. Without that
/// second fact, every launch would put back what they had just removed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StatusLineSlot {
    Free,
    Ours,
    Theirs,
}

pub(crate) fn statusline_slot(
    settings: &serde_json::Map<String, serde_json::Value>,
    script: &Path,
    installed_before: bool,
) -> StatusLineSlot {
    let Some(entry) = settings.get("statusLine") else {
        return if installed_before {
            StatusLineSlot::Theirs
        } else {
            StatusLineSlot::Free
        };
    };
    let command = entry.get("command").and_then(serde_json::Value::as_str);
    match command {
        Some(command) if command.contains(&script.display().to_string()) => StatusLineSlot::Ours,
        Some(_) => StatusLineSlot::Theirs,
        None => {
            if installed_before {
                StatusLineSlot::Theirs
            } else {
                StatusLineSlot::Free
            }
        }
    }
}

/// Put the reader in the slot, or take it out again. Returns whether the
/// settings changed, so a file that would be rewritten identically is left
/// alone.
pub(crate) fn apply_statusline(
    settings: &mut serde_json::Map<String, serde_json::Value>,
    script: &Path,
    installed_before: bool,
    enable: bool,
) -> bool {
    let slot = statusline_slot(settings, script, installed_before);
    match (slot, enable) {
        (StatusLineSlot::Theirs, _) => false,
        (StatusLineSlot::Ours, false) => settings.remove("statusLine").is_some(),
        (StatusLineSlot::Ours, true) => false,
        (StatusLineSlot::Free, false) => false,
        (StatusLineSlot::Free, true) => {
            settings.insert(
                "statusLine".to_owned(),
                serde_json::json!({
                    "type": "command",
                    "command": script.display().to_string(),
                }),
            );
            true
        }
    }
}
