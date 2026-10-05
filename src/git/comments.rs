//! The notes a person writes against a diff while reading it, and the message
//! those notes become when they are handed to the agent.
//!
//! A child of `src/git.rs` beside `src/git/setup.rs`, for the same reason: a
//! new top-level module is a line in `src/main.rs`, which `docs/sdlc/risk.yaml`
//! holds at `paused`.

use crate::*;

/// One note: which project, which file, which line on the new side, what that
/// line said when the note was written, and the note itself.
///
/// The anchor is what the line said rather than where it was. A note whose line
/// has changed is marked, not moved: moving it to the line that now has that
/// number would be a guess, and a guess about which line a review comment
/// belongs to is worse than saying the line changed.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct DiffComment {
    pub(crate) project: Uuid,
    pub(crate) file: String,
    pub(crate) line: usize,
    pub(crate) anchor: String,
    pub(crate) body: String,
    #[serde(default)]
    pub(crate) sent: bool,
    #[serde(default)]
    pub(crate) resolved: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct DiffCommentStore {
    #[serde(default)]
    pub(crate) comments: Vec<DiffComment>,
}

pub(crate) fn diff_comments_path(data_file: &Path) -> PathBuf {
    data_file
        .parent()
        .unwrap_or(Path::new("."))
        .join(DIFF_COMMENTS_FILE_NAME)
}

/// A file that cannot be read or parsed yields no notes, and where its bytes
/// went if they could not be parsed.
///
/// A partial set that looked whole would be worse than none: the person would
/// send a review missing the point they cared about and never know. Yielding
/// nothing was always deliberate. Letting the next save write that nothing over
/// the file was not — a sidecar holding twelve notes that stopped parsing
/// became one note and no copy anywhere, with nothing said. It is moved aside
/// first, the same move `recover_unreadable_store` makes for the store itself,
/// so the empty set is honest rather than destructive.
pub(crate) fn load_diff_comments_reporting(
    data_file: &Path,
) -> (DiffCommentStore, Option<PathBuf>) {
    let path = diff_comments_path(data_file);
    let Ok(raw) = fs::read(&path) else {
        return (DiffCommentStore::default(), None);
    };
    if let Ok(store) = serde_json::from_slice::<DiffCommentStore>(&raw) {
        return (store, None);
    }
    // An empty file is a first save that was interrupted, not a set of notes
    // that stopped parsing. There is nothing in it to rescue.
    if raw.iter().all(u8::is_ascii_whitespace) {
        return (DiffCommentStore::default(), None);
    }
    let rescued = path.with_extension(format!("unreadable-{}.json", Uuid::new_v4().simple()));
    match fs::rename(&path, &rescued) {
        Ok(()) => (DiffCommentStore::default(), Some(rescued)),
        // The move failed, so the bytes are still where they were. Reporting no
        // rescue keeps the caller from claiming a copy exists, and the next
        // save will still overwrite — but saying so is the honest failure, and
        // it is the one the notice above the pane carries.
        Err(_) => (DiffCommentStore::default(), None),
    }
}

pub(crate) fn save_diff_comments(data_file: &Path, store: &DiffCommentStore) -> Result<()> {
    let path = diff_comments_path(data_file);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let encoded = serde_json::to_string_pretty(store)?;
    write_file_atomically(&path, format!("{encoded}\n").as_bytes())?;
    Ok(())
}

/// What a line said, as a fixed-size value. The digest rather than the text
/// because a note file should not grow with the length of the lines somebody
/// commented on, and because the only question ever asked of it is whether it
/// is still the same.
pub(crate) fn comment_anchor(line: &str) -> String {
    sha256_hex(line.trim_end().as_bytes())
}

/// A comment body cannot end its own quoting, be read as two notes, or carry a
/// bare newline into a form whose blocks are newline-separated.
fn escape_comment_body(body: &str) -> String {
    let mut escaped = String::with_capacity(body.len());
    for character in body.chars() {
        match character {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\r' => escaped.push_str("\\r"),
            '\n' => escaped.push_str("\\n"),
            other => escaped.push(other),
        }
    }
    escaped
}

/// The message the agent receives. It names the file and the line before the
/// comment, has no preamble, and separates notes by a blank line. An agent reading it does not
/// have to work out which of eighty changed lines "the null check" meant, and a
/// preamble would only be a sentence the model has to decide whether to obey.
pub(crate) fn diff_comment_message(comments: &[(DiffComment, bool)]) -> String {
    comments
        .iter()
        .map(|(comment, stale)| {
            let mut block = format!("File: {}\nLine: {}\n", comment.file, comment.line);
            if *stale {
                block.push_str("Note: the line has changed since this comment was written.\n");
            }
            block.push_str(&format!(
                "User comment: \"{}\"",
                escape_comment_body(&comment.body)
            ));
            block
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// Review notes formatted as structured Markdown, suitable for copying to clipboard
/// or reviewing across tools.
pub(crate) fn format_diff_comments_markdown(comments: &[(DiffComment, bool)]) -> String {
    if comments.is_empty() {
        return String::new();
    }
    let mut out = String::from("# Review Notes\n\n");
    let mut current_file: Option<&str> = None;
    for (comment, stale) in comments {
        if current_file != Some(&comment.file) {
            current_file = Some(&comment.file);
            out.push_str(&format!("## {}\n\n", comment.file));
        }
        let mut status_flags = Vec::new();
        if *stale {
            status_flags.push("stale");
        }
        if comment.sent {
            status_flags.push("sent");
        }
        if comment.resolved {
            status_flags.push("resolved");
        }
        let flag_str = if status_flags.is_empty() {
            String::new()
        } else {
            format!(" [{}]", status_flags.join(", "))
        };
        out.push_str(&format!(
            "- Line {}{}:\n  > {}\n\n",
            comment.line,
            flag_str,
            escape_comment_body(&comment.body).replace('\n', "\n  > ")
        ));
    }
    out.trim_end().to_owned()
}

/// The notes on the diff currently being drawn, and which one is open.
///
/// Held by the application and lent to the pane, because the pane is redrawn
/// from nothing every frame and a draft half-typed into it has to outlive that.
#[derive(Debug, Clone, Default)]
pub(crate) struct DiffAnnotations {
    pub(crate) comments: Vec<DiffComment>,
    /// The file and new-side line number of the note being written, if any.
    pub(crate) editing: Option<(String, usize)>,
    pub(crate) draft: String,
}

impl DiffAnnotations {
    /// Every project's notes live in one list, because they are written to one
    /// file. The project is therefore part of every lookup rather than a second
    /// copy of the list that has to be kept in step with the first.
    pub(crate) fn find(&self, project: Uuid, file: &str, line: usize) -> Option<&DiffComment> {
        self.comments.iter().find(|comment| {
            comment.project == project && comment.file == file && comment.line == line
        })
    }

    pub(crate) fn count(&self, project: Uuid) -> usize {
        self.comments
            .iter()
            .filter(|comment| comment.project == project)
            .count()
    }

    pub(crate) fn count_unresolved(&self, project: Uuid) -> usize {
        self.comments
            .iter()
            .filter(|comment| comment.project == project && !comment.resolved)
            .count()
    }

    pub(crate) fn count_sent(&self, project: Uuid) -> usize {
        self.comments
            .iter()
            .filter(|comment| comment.project == project && comment.sent && !comment.resolved)
            .count()
    }

    pub(crate) fn count_resolved(&self, project: Uuid) -> usize {
        self.comments
            .iter()
            .filter(|comment| comment.project == project && comment.resolved)
            .count()
    }

    pub(crate) fn mark_sent(&mut self, project: Uuid) {
        for comment in &mut self.comments {
            if comment.project == project && !comment.resolved {
                comment.sent = true;
            }
        }
    }

    pub(crate) fn toggle_resolved(&mut self, project: Uuid, file: &str, line: usize) -> bool {
        if let Some(comment) = self
            .comments
            .iter_mut()
            .find(|c| c.project == project && c.file == file && c.line == line)
        {
            comment.resolved = !comment.resolved;
            true
        } else {
            false
        }
    }

    pub(crate) fn clear_resolved(&mut self, project: Uuid) -> usize {
        let before = self.comments.len();
        self.comments
            .retain(|comment| !(comment.project == project && comment.resolved));
        before - self.comments.len()
    }

    pub(crate) fn clear_project(&mut self, project: Uuid) -> usize {
        let before = self.comments.len();
        self.comments.retain(|comment| comment.project != project);
        before - self.comments.len()
    }

    /// Whether a row needs a note under it: it has one, or one is being written
    /// against it. The second half is why an empty note still takes up space —
    /// a box that appeared only once it had text in it would have nowhere for
    /// the first character to go.
    pub(crate) fn shows_row(&self, project: Uuid, file: &str, line: usize) -> bool {
        self.find(project, file, line).is_some()
            || self
                .editing
                .as_ref()
                .is_some_and(|(open_file, open_line)| open_file == file && *open_line == line)
    }

    pub(crate) fn open(&mut self, project: Uuid, file: &str, line: usize) {
        self.draft = self
            .find(project, file, line)
            .map(|comment| comment.body.clone())
            .unwrap_or_default();
        self.editing = Some((file.to_owned(), line));
    }

    /// Writing an empty note is how a note is deleted, so that there is one
    /// gesture rather than two for "never mind".
    pub(crate) fn save(&mut self, project: Uuid, file: &str, line: usize, anchor: &str) -> bool {
        let body = self.draft.trim().to_owned();
        self.editing = None;
        self.draft.clear();
        let existing = self.comments.iter().position(|comment| {
            comment.project == project && comment.file == file && comment.line == line
        });
        match (existing, body.is_empty()) {
            (Some(index), true) => {
                self.comments.remove(index);
                true
            }
            (Some(index), false) => {
                if self.comments[index].body == body {
                    return false;
                }
                self.comments[index].body = body;
                true
            }
            (None, true) => false,
            (None, false) => {
                self.comments.push(DiffComment {
                    project,
                    file: file.to_owned(),
                    line,
                    anchor: anchor.to_owned(),
                    body,
                    sent: false,
                    resolved: false,
                });
                true
            }
        }
    }

    pub(crate) fn remove(&mut self, project: Uuid, file: &str, line: usize) -> bool {
        self.editing = None;
        self.draft.clear();
        let before = self.comments.len();
        self.comments.retain(|comment| {
            !(comment.project == project && comment.file == file && comment.line == line)
        });
        self.comments.len() != before
    }
}
