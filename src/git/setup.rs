//! What a repository says about preparing a fresh checkout of itself, and
//! whether the person has agreed to run it.
//!
//! A child of `src/git.rs` rather than a module of its own, because a new
//! top-level module is a line in `src/main.rs` and `src/main.rs` is a paused
//! surface in `docs/sdlc/risk.yaml`.
//!
//! The whole of the format is: a shell script, at `SETUP_SCRIPT_RELATIVE_PATH`,
//! inside the worktree. A schema and a YAML parser would add nothing: the only
//! key that earns its place holds a shell script, and a file that already is one
//! needs no parser.

use crate::*;

/// A setup script that exists and is small enough to run, described by what the
/// person is shown and what the approval is taken over.
#[derive(Debug, Clone)]
pub(crate) struct SetupScript {
    pub(crate) path: PathBuf,
    /// SHA-256 of the exact bytes the preview was cut from. The approval is of
    /// these contents, so this is what an approval remembers.
    pub(crate) digest: String,
    pub(crate) preview: Vec<String>,
    pub(crate) total_lines: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct PendingRunScript {
    pub(crate) project: Uuid,
    pub(crate) worktree: PathBuf,
    pub(crate) script: SetupScript,
}

#[derive(Debug, Clone)]
pub(crate) enum SetupScriptRead {
    /// Most projects. Nothing is drawn and nothing runs.
    Missing,
    /// Reported rather than truncated: a preview that does not match what would
    /// run is worse than no preview at all.
    TooLarge(u64),
    Found(SetupScript),
}

pub(crate) fn setup_script_path(worktree: &Path) -> PathBuf {
    worktree.join(SETUP_SCRIPT_RELATIVE_PATH)
}

pub(crate) fn read_setup_script(worktree: &Path) -> SetupScriptRead {
    let path = setup_script_path(worktree);
    let Ok(metadata) = fs::metadata(&path) else {
        return SetupScriptRead::Missing;
    };
    if !metadata.is_file() {
        return SetupScriptRead::Missing;
    }
    if metadata.len() > SETUP_SCRIPT_MAX_BYTES {
        return SetupScriptRead::TooLarge(metadata.len());
    }
    let Ok(bytes) = fs::read(&path) else {
        return SetupScriptRead::Missing;
    };
    SetupScriptRead::Found(describe_setup_script(path, &bytes))
}

/// The preview and the digest are cut from one read of one buffer. Reading
/// twice would let the file change between them, and then what is shown is not
/// what is approved.
fn describe_setup_script(path: PathBuf, bytes: &[u8]) -> SetupScript {
    let text = String::from_utf8_lossy(bytes);
    let lines: Vec<&str> = text.lines().collect();
    SetupScript {
        path,
        digest: sha256_hex(bytes),
        preview: lines
            .iter()
            .take(SETUP_SCRIPT_PREVIEW_LINES)
            .map(|line| (*line).to_owned())
            .collect(),
        total_lines: lines.len(),
    }
}

/// What the pane runs. A function rather than a `format!` at the call site so
/// that the quoting has exactly one place to be missing from — entry 013 in
/// `docs/sdlc/lessons.md` is what happens when a path with a space reaches a
/// shell unquoted, and every macOS worktree path may contain one.
pub(crate) fn setup_command(script: &Path) -> String {
    format!("bash {}", shell_quote(&script.display().to_string()))
}

/// One approval: these contents, in this project. Not the path — an approval
/// that survives an edit to the thing approved is an approval of nothing — and
/// not the contents alone, because what is being trusted is a repository.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct SetupApproval {
    pub(crate) project: Uuid,
    pub(crate) digest: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct SetupTrust {
    #[serde(default)]
    pub(crate) approvals: Vec<SetupApproval>,
}

pub(crate) fn setup_trust_path(data_file: &Path) -> PathBuf {
    data_file
        .parent()
        .unwrap_or(Path::new("."))
        .join(SETUP_TRUST_FILE_NAME)
}

/// A file that cannot be read or cannot be parsed means nothing is approved,
/// which fails towards asking. The other direction would be a corrupt file that
/// silently agreed to run a script.
pub(crate) fn load_setup_trust(data_file: &Path) -> SetupTrust {
    fs::read(setup_trust_path(data_file))
        .ok()
        .and_then(|raw| serde_json::from_slice::<SetupTrust>(&raw).ok())
        .unwrap_or_default()
}

pub(crate) fn save_setup_trust(data_file: &Path, trust: &SetupTrust) -> Result<()> {
    let path = setup_trust_path(data_file);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let encoded = serde_json::to_string_pretty(trust)?;
    write_file_atomically(&path, format!("{encoded}\n").as_bytes())?;
    Ok(())
}

pub(crate) fn is_setup_approved(trust: &SetupTrust, project: Uuid, digest: &str) -> bool {
    trust
        .approvals
        .iter()
        .any(|approval| approval.project == project && approval.digest == digest)
}

pub(crate) fn approve_setup(trust: &mut SetupTrust, project: Uuid, digest: &str) {
    if is_setup_approved(trust, project, digest) {
        return;
    }
    trust.approvals.push(SetupApproval {
        project,
        digest: digest.to_owned(),
    });
}

/// A worktree that has just been made and whose setup script is waiting on the
/// person. Held rather than run, because the whole point of the block it draws
/// is that a repository does not get to decide to run code on this machine.
#[derive(Debug, Clone)]
pub(crate) struct PendingSetup {
    pub(crate) project: Uuid,
    pub(crate) worktree: PathBuf,
    pub(crate) script: SetupScript,
    /// The checkbox: whether approving also remembers these contents.
    pub(crate) remember: bool,
}

/// What should happen when the person presses 実行する.
#[derive(Debug, Clone)]
pub(crate) enum SetupRunDecision {
    /// The file on disk is still the one that was shown.
    Run(SetupScript),
    /// It is not. The new contents come back so the person can be asked about
    /// what is actually there, rather than about what used to be.
    Changed(SetupScript),
    /// It went away, or grew past the ceiling, between being shown and being
    /// run.
    Unreadable,
}

/// Decide against a fresh read, never against the digest carried from the
/// moment the script was shown. Those two are the same value almost always, and
/// the case where they differ is the only case this check exists for.
pub(crate) fn decide_setup_run(worktree: &Path, shown_digest: &str) -> SetupRunDecision {
    match read_setup_script(worktree) {
        SetupScriptRead::Found(script) if script.digest == shown_digest => {
            SetupRunDecision::Run(script)
        }
        SetupScriptRead::Found(script) => SetupRunDecision::Changed(script),
        SetupScriptRead::Missing | SetupScriptRead::TooLarge(_) => SetupRunDecision::Unreadable,
    }
}

pub(crate) fn run_script_path(worktree: &Path) -> PathBuf {
    worktree.join(RUN_SCRIPT_RELATIVE_PATH)
}

pub(crate) fn read_run_script(worktree: &Path) -> SetupScriptRead {
    let path = run_script_path(worktree);
    let Ok(metadata) = fs::metadata(&path) else {
        return SetupScriptRead::Missing;
    };
    if !metadata.is_file() {
        return SetupScriptRead::Missing;
    }
    if metadata.len() > SETUP_SCRIPT_MAX_BYTES {
        return SetupScriptRead::TooLarge(metadata.len());
    }
    let Ok(bytes) = fs::read(&path) else {
        return SetupScriptRead::Missing;
    };
    SetupScriptRead::Found(describe_setup_script(path, &bytes))
}

pub(crate) fn decide_run_script_run(worktree: &Path, shown_digest: &str) -> SetupRunDecision {
    match read_run_script(worktree) {
        SetupScriptRead::Found(script) if script.digest == shown_digest => {
            SetupRunDecision::Run(script)
        }
        SetupScriptRead::Found(script) => SetupRunDecision::Changed(script),
        SetupScriptRead::Missing | SetupScriptRead::TooLarge(_) => SetupRunDecision::Unreadable,
    }
}
