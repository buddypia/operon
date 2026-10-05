use crate::*;

pub(crate) mod comments;
pub(crate) use comments::*;
pub(crate) mod ports;
pub(crate) use ports::*;
pub(crate) mod setup;
pub(crate) use setup::*;

/// A `git` aimed at the repository the caller names, and at no other.
///
/// `run_command_with_output_limit` drops `INHERITED_REPOSITORY_POINTERS` for
/// every child, so production git is covered by going through `src/exec.rs` at
/// all. This exists for the commands that do not: a `Command` handed straight
/// to `.status()` or `.output()`, which is how the tests build theirs. Both
/// read the same constant, so there is one list and not two. The four further
/// variables are change 058's: they name an index, an object store, a prefix,
/// and a common directory, and each one alone redirects a git that is otherwise
/// pointed at the right tree.
///
/// `no_git_command_in_the_crate_can_inherit_the_session_repository` fails on a
/// raw git `Command` built anywhere under `src/` but here.
pub(crate) fn git_command(path: &Path) -> Command {
    let mut command = Command::new("git");
    forget_inherited_repository(&mut command).current_dir(path);
    command
}

/// Strips every variable that would point a git — this one, or one a child
/// process spawns — at the session's repository instead of the one it was
/// given. One list: `git_command` and anything that runs a git-spawning script
/// against a fixture share it, so a variable added here reaches both.
pub(crate) fn forget_inherited_repository(command: &mut Command) -> &mut Command {
    for pointer in INHERITED_REPOSITORY_POINTERS {
        command.env_remove(pointer);
    }
    command
        .env_remove("GIT_INDEX_FILE")
        .env_remove("GIT_OBJECT_DIRECTORY")
        .env_remove("GIT_PREFIX")
        .env_remove("GIT_COMMON_DIR")
}

pub(crate) fn git_output_allowing_limited(
    path: &Path,
    arguments: &[&str],
    allowed_codes: &[i32],
    max_bytes: usize,
) -> Result<(String, bool)> {
    if max_bytes == 0 {
        return Ok((String::new(), true));
    }
    let mut command = git_command(path);
    command.args(arguments);
    let limited = run_command_with_output_limit(
        &mut command,
        Duration::from_secs(20),
        max_bytes,
        COMMAND_ERROR_MAX_BYTES,
    )?;
    let accepted = limited.output.status.success()
        || limited
            .output
            .status
            .code()
            .is_some_and(|code| allowed_codes.contains(&code));
    if accepted || limited.stdout_truncated {
        Ok((
            String::from_utf8_lossy(&limited.output.stdout).to_string(),
            limited.stdout_truncated,
        ))
    } else {
        Err(anyhow!(
            String::from_utf8_lossy(&limited.output.stderr).to_string()
        ))
    }
}

pub(crate) fn parse_porcelain_changed_files(output: &str) -> Vec<ChangedFile> {
    let entries = output.split('\0').collect::<Vec<_>>();
    let mut files = Vec::new();
    let mut index = 0;
    while index < entries.len() {
        let entry = entries[index];
        if entry.len() < 4 {
            index += 1;
            continue;
        }
        let status = &entry[..2];
        let path = &entry[3..];
        files.push(ChangedFile {
            status: status.trim().to_owned(),
            path: path.to_owned(),
        });
        if status.contains('R') || status.contains('C') {
            index += 1;
        }
        index += 1;
    }
    files
}

/// Split a unified diff into one entry per file, classifying and numbering
/// every line on the way past.
///
/// This is deliberately tolerant. The text it is handed is whatever `git diff`
/// wrote, plus the `--no-index` diffs this app synthesises for untracked files,
/// plus the truncation notice appended when the capture budget runs out — and a
/// review screen that renders nothing because one header was in a shape the
/// parser did not expect is worse than one that renders the odd line as a note.
/// Anything unrecognised therefore lands in `DiffLineKind::Note` and is still
/// shown; nothing is ever dropped.
pub(crate) fn parse_unified_diff(diff: &str) -> Vec<DiffFile> {
    /// The header lines that carry no content of their own. Each one either
    /// starts a file, names it, or describes a change git chose not to show,
    /// and all of them are consumed into `DiffFile` rather than drawn.
    fn header_prefix(line: &str) -> bool {
        [
            "index ",
            "old mode ",
            "new mode ",
            "new file mode ",
            "deleted file mode ",
            "similarity index ",
            "dissimilarity index ",
            "copy from ",
            "copy to ",
        ]
        .iter()
        .any(|prefix| line.starts_with(prefix))
    }

    let mut files: Vec<DiffFile> = Vec::new();
    let mut old_number = 0usize;
    let mut new_number = 0usize;
    // Line numbers only mean something inside a hunk. Before the first `@@` of
    // a file, git is describing the file rather than quoting it.
    let mut in_hunk = false;

    // A diff that opens with a hunk — a fragment pasted in, or a `git diff`
    // whose header this app trimmed — still has to render, so the first line
    // that needs a file gets an unnamed one.
    fn current(files: &mut Vec<DiffFile>) -> &mut DiffFile {
        if files.is_empty() {
            files.push(DiffFile {
                path: String::new(),
                renamed_from: None,
                change: DiffFileChange::Modified,
                lines: Vec::new(),
                added: 0,
                removed: 0,
                widest: 0,
                note: None,
            });
        }
        files.last_mut().expect("just pushed when empty")
    }

    for line in diff.split('\n') {
        // A carriage return is part of the file's bytes rather than of the
        // diff, and egui would draw it as a missing glyph at the end of every
        // row of a CRLF file. The terminal pane strips it for the same reason.
        let line = line.strip_suffix('\r').unwrap_or(line);

        if let Some(paths) = line.strip_prefix("diff --git ") {
            let (old, new) = split_diff_git_paths(paths);
            files.push(DiffFile {
                path: new.clone().unwrap_or_default(),
                renamed_from: None,
                change: DiffFileChange::Modified,
                lines: Vec::new(),
                added: 0,
                removed: 0,
                widest: 0,
                note: None,
            });
            // Keep the old path only until `---`/`+++` confirm it; a rename is
            // recognised from `rename from`, not from the two differing here.
            if let (Some(old), true) = (old, new.is_none()) {
                files.last_mut().expect("just pushed").path = old;
            }
            in_hunk = false;
            continue;
        }
        if let Some(rest) = line.strip_prefix("rename from ") {
            let file = current(&mut files);
            file.renamed_from = Some(rest.to_owned());
            file.change = DiffFileChange::Renamed;
            continue;
        }
        if let Some(rest) = line.strip_prefix("rename to ") {
            let file = current(&mut files);
            file.path = rest.to_owned();
            file.change = DiffFileChange::Renamed;
            continue;
        }
        if line.starts_with("new file mode ") {
            current(&mut files).change = DiffFileChange::Added;
            continue;
        }
        if line.starts_with("deleted file mode ") {
            current(&mut files).change = DiffFileChange::Removed;
            continue;
        }
        if line.starts_with("Binary files ") || line.starts_with("GIT binary patch") {
            current(&mut files).note =
                Some(tr("バイナリファイルのため差分は表示できません。").into());
            continue;
        }
        if header_prefix(line) {
            continue;
        }
        // `---` and `+++` have to be tested before `-` and `+`, or the two
        // lines that name the file would be counted as a removal and an
        // addition of it.
        if let Some(rest) = line.strip_prefix("--- ") {
            if let Some(path) = strip_diff_path_prefix(rest) {
                let file = current(&mut files);
                if file.path.is_empty() {
                    file.path = path;
                }
            } else if rest == "/dev/null" && !files.is_empty() {
                let file = current(&mut files);
                if file.change == DiffFileChange::Modified {
                    file.change = DiffFileChange::Added;
                }
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix("+++ ") {
            if let Some(path) = strip_diff_path_prefix(rest) {
                current(&mut files).path = path;
            } else if rest == "/dev/null" {
                let file = current(&mut files);
                if file.change == DiffFileChange::Modified {
                    file.change = DiffFileChange::Removed;
                }
            }
            continue;
        }
        if line.starts_with("@@") {
            if let Some((old_start, new_start)) = parse_hunk_header(line) {
                old_number = old_start;
                new_number = new_start;
                in_hunk = true;
            } else {
                // A combined diff from a merge conflict (`@@@`) numbers three
                // sides at once. Rather than mis-number it, stop numbering and
                // show the hunk as written.
                in_hunk = false;
            }
            current(&mut files).lines.push(DiffLine {
                kind: DiffLineKind::Hunk,
                old_number: None,
                new_number: None,
                text: line.to_owned(),
                highlights: Vec::new(),
            });
            continue;
        }
        if !in_hunk {
            // Text between files that git did not label: the truncation notice
            // this app appends, or a blank line separating two `--no-index`
            // diffs. A blank one carries nothing and would otherwise open an
            // unnamed file at the end of the list.
            if line.trim().is_empty() {
                continue;
            }
            current(&mut files).lines.push(DiffLine {
                kind: DiffLineKind::Note,
                old_number: None,
                new_number: None,
                text: line.to_owned(),
                highlights: Vec::new(),
            });
            continue;
        }
        let (kind, text) = match line.as_bytes().first() {
            Some(b'+') => (DiffLineKind::Added, &line[1..]),
            Some(b'-') => (DiffLineKind::Removed, &line[1..]),
            Some(b' ') => (DiffLineKind::Context, &line[1..]),
            // `\ No newline at end of file`.
            Some(b'\\') => (DiffLineKind::Note, line),
            // The last line of a diff that ends in a newline splits into an
            // empty trailing piece, and git also writes a bare empty line for
            // an empty context line.
            _ => (DiffLineKind::Context, line),
        };
        let file = current(&mut files);
        let (old, new) = match kind {
            DiffLineKind::Added => {
                file.added += 1;
                let at = new_number;
                new_number += 1;
                (None, Some(at))
            }
            DiffLineKind::Removed => {
                file.removed += 1;
                let at = old_number;
                old_number += 1;
                (Some(at), None)
            }
            DiffLineKind::Context => {
                let (old, new) = (old_number, new_number);
                old_number += 1;
                new_number += 1;
                (Some(old), Some(new))
            }
            DiffLineKind::Hunk | DiffLineKind::Note => (None, None),
        };
        file.lines.push(DiffLine {
            kind,
            old_number: old,
            new_number: new,
            text: text.to_owned(),

            highlights: Vec::new(),
        });
    }

    // A diff whose last line ends in `\n` splits into a trailing empty piece,
    // which the loop above turned into an empty context line at the end of the
    // final file. It is not in the file, so it is not shown.
    if let Some(file) = files.last_mut() {
        if file
            .lines
            .last()
            .is_some_and(|line| line.kind == DiffLineKind::Context && line.text.is_empty())
        {
            file.lines.pop();
        }
    }
    files.retain(|file| !file.lines.is_empty() || file.note.is_some() || !file.path.is_empty());
    for file in &mut files {
        mark_rewritten_words(&mut file.lines);
        file.widest = file
            .lines
            .iter()
            .map(|line| line.text.chars().count())
            .max()
            .unwrap_or(0);
    }
    files
}

/// How many tokens on one side of a pair still earn a full comparison.
///
/// The comparison below is quadratic in tokens. It runs once per parse rather
/// than once per frame, but a minified bundle is one line of a hundred thousand
/// tokens and squaring that is not a cost anyone agreed to pay. Past this
/// ceiling the pair keeps the prefix and suffix it can find in linear time and
/// marks everything between them as one span — still the truth, just a blunter
/// one, on a line nobody was going to read word by word anyway.
pub(crate) const DIFF_WORD_TOKEN_LIMIT: usize = 128;

/// How much of each side has to survive unmarked for the marks to be drawn.
///
/// Below this the two lines are not one line rewritten, they are two different
/// lines that happen to be adjacent, and banding three quarters of both rows
/// says nothing the wash did not already say. Dropping the marks there is what
/// makes the cheap positional pairing safe: a mispaired line has almost nothing
/// in common with its partner, so the failure mode is a mark that is missing,
/// never one that points at the wrong words.
const DIFF_WORD_MIN_COMMON: f32 = 0.25;

/// Mark the words that moved, for each line of this file that is one half of a
/// rewrite.
///
/// A replacement block is a run of removed lines immediately followed by a run
/// of added lines, which is the shape git writes when an edit lands in place.
/// The two runs are paired by position — the first removed line against the
/// first added one — and the surplus of the longer run is left alone, because
/// a line with no counterpart was inserted or deleted rather than rewritten.
fn mark_rewritten_words(lines: &mut [DiffLine]) {
    let mut at = 0;
    while at < lines.len() {
        if lines[at].kind != DiffLineKind::Removed {
            at += 1;
            continue;
        }
        let removed_start = at;
        while at < lines.len() && lines[at].kind == DiffLineKind::Removed {
            at += 1;
        }
        let added_start = at;
        while at < lines.len() && lines[at].kind == DiffLineKind::Added {
            at += 1;
        }
        let removed = removed_start..added_start;
        let added = added_start..at;
        for offset in 0..removed.len().min(added.len()) {
            let (before, after) = lines.split_at_mut(added_start);
            let old = &mut before[removed_start + offset];
            let new = &mut after[offset];
            if let Some((old_spans, new_spans)) = changed_spans(&old.text, &new.text) {
                old.highlights = old_spans;
                new.highlights = new_spans;
            }
        }
    }
}

/// Byte ranges into one line, in order and never overlapping — the same shape
/// `DiffLine::highlights` holds.
type WordSpans = Vec<std::ops::Range<usize>>;

/// The spans that differ between two versions of one line, or `None` where the
/// two have too little in common to be called the same line.
fn changed_spans(old: &str, new: &str) -> Option<(WordSpans, WordSpans)> {
    if old == new || old.is_empty() || new.is_empty() {
        return None;
    }
    let old_tokens = word_tokens(old);
    let new_tokens = word_tokens(new);
    let token = |text: &str, range: &std::ops::Range<usize>| text[range.clone()].to_owned();

    // Trim what the two share at each end first. It is linear, it is what
    // makes the common case — one identifier changed in the middle of a long
    // line — cost almost nothing, and it is the whole answer past the ceiling.
    let mut prefix = 0;
    while prefix < old_tokens.len()
        && prefix < new_tokens.len()
        && token(old, &old_tokens[prefix]) == token(new, &new_tokens[prefix])
    {
        prefix += 1;
    }
    let mut suffix = 0;
    while suffix < old_tokens.len() - prefix
        && suffix < new_tokens.len() - prefix
        && token(old, &old_tokens[old_tokens.len() - 1 - suffix])
            == token(new, &new_tokens[new_tokens.len() - 1 - suffix])
    {
        suffix += 1;
    }
    let old_middle = prefix..old_tokens.len() - suffix;
    let new_middle = prefix..new_tokens.len() - suffix;

    let (old_changed, new_changed) =
        if old_middle.len() > DIFF_WORD_TOKEN_LIMIT || new_middle.len() > DIFF_WORD_TOKEN_LIMIT {
            (
                old_middle.clone().collect::<Vec<_>>(),
                new_middle.clone().collect::<Vec<_>>(),
            )
        } else {
            let old_words = old_middle
                .clone()
                .map(|index| token(old, &old_tokens[index]))
                .collect::<Vec<_>>();
            let new_words = new_middle
                .clone()
                .map(|index| token(new, &new_tokens[index]))
                .collect::<Vec<_>>();
            let common = longest_common_subsequence(&old_words, &new_words);
            let mut old_changed = Vec::new();
            let mut new_changed = Vec::new();
            let (mut old_at, mut new_at) = (0, 0);
            for (old_index, new_index) in common
                .iter()
                .copied()
                .chain([(old_words.len(), new_words.len())])
            {
                old_changed.extend((old_at..old_index).map(|index| old_middle.start + index));
                new_changed.extend((new_at..new_index).map(|index| new_middle.start + index));
                old_at = old_index + 1;
                new_at = new_index + 1;
            }
            (old_changed, new_changed)
        };

    let old_spans = merge_token_spans(&old_tokens, &old_changed);
    let new_spans = merge_token_spans(&new_tokens, &new_changed);
    if old_spans.is_empty() && new_spans.is_empty() {
        return None;
    }
    let marked = |text: &str, spans: &[std::ops::Range<usize>]| {
        spans.iter().map(|span| span.len()).sum::<usize>() as f32 / text.len().max(1) as f32
    };
    if marked(old, &old_spans) > 1.0 - DIFF_WORD_MIN_COMMON
        || marked(new, &new_spans) > 1.0 - DIFF_WORD_MIN_COMMON
    {
        return None;
    }
    Some((old_spans, new_spans))
}

/// Split a line into the units a mark is allowed to fall on: a run of word
/// characters, a run of whitespace, one character of a script written without
/// spaces, or one character of anything else.
///
/// Marking single letters instead would turn a rename into a scatter of
/// one-letter bands, which is harder to read than no marks at all. The
/// exception is the scripts that do not delimit their words: this app's own
/// interface is Japanese, and `セッションを復元しました。` is one run of
/// alphanumeric characters, so a rule that only knew about runs would make a
/// whole Japanese sentence one token and then mark all of it or none of it.
/// There the character is the unit, which is also how a reader of those
/// scripts sees the change; adjacent marked characters merge back into one
/// band afterwards.
fn word_tokens(text: &str) -> WordSpans {
    #[derive(PartialEq, Eq, Clone, Copy)]
    enum Class {
        Word,
        Space,
        /// One character wide: a script with no space between its words, or
        /// punctuation, both of which are their own unit.
        Single,
    }
    let class = |ch: char| {
        if writes_without_spaces(ch) {
            Class::Single
        } else if ch.is_alphanumeric() || ch == '_' {
            Class::Word
        } else if ch.is_whitespace() {
            Class::Space
        } else {
            Class::Single
        }
    };
    let mut tokens = Vec::new();
    let mut characters = text.char_indices().peekable();
    while let Some((start, first)) = characters.next() {
        let mut end = start + first.len_utf8();
        let first_class = class(first);
        if first_class != Class::Single {
            while let Some(&(next_start, next)) = characters.peek() {
                if class(next) != first_class {
                    break;
                }
                end = next_start + next.len_utf8();
                characters.next();
            }
        }
        tokens.push(start..end);
    }
    tokens
}

/// Whether this character belongs to a script that puts no space between its
/// words. Named by block rather than by a general Unicode property because
/// this is the exact question — "would a run of these be a whole sentence" —
/// and no single property answers it.
fn writes_without_spaces(ch: char) -> bool {
    matches!(
        ch as u32,
        // Hiragana and katakana, including the prolonged sound mark.
        0x3040..=0x30FF
        // CJK ideographs: extension A, the unified block, compatibility.
        | 0x3400..=0x4DBF
        | 0x4E00..=0x9FFF
        | 0xF900..=0xFAFF
        // Halfwidth katakana.
        | 0xFF66..=0xFF9F
        // Hangul syllables.
        | 0xAC00..=0xD7AF
    )
}

/// Fold a list of changed token indices into the fewest byte ranges that cover
/// them, so touching tokens are drawn as one band rather than as several.
fn merge_token_spans(tokens: &[std::ops::Range<usize>], changed: &[usize]) -> WordSpans {
    let mut spans: Vec<std::ops::Range<usize>> = Vec::new();
    for &index in changed {
        let token = tokens[index].clone();
        match spans.last_mut() {
            Some(last) if last.end == token.start => last.end = token.end,
            _ => spans.push(token),
        }
    }
    spans
}

/// The indices of a longest common subsequence of two token lists.
///
/// Bounded by `DIFF_WORD_TOKEN_LIMIT` at the only call site, which is what
/// keeps the table below small enough to build for every changed line of a
/// large diff.
fn longest_common_subsequence(left: &[String], right: &[String]) -> Vec<(usize, usize)> {
    let (rows, columns) = (left.len(), right.len());
    let mut table = vec![0u32; (rows + 1) * (columns + 1)];
    for row in (0..rows).rev() {
        for column in (0..columns).rev() {
            let at = row * (columns + 1) + column;
            table[at] = if left[row] == right[column] {
                table[at + columns + 2] + 1
            } else {
                table[at + 1].max(table[at + columns + 1])
            };
        }
    }
    let mut pairs = Vec::new();
    let (mut row, mut column) = (0, 0);
    while row < rows && column < columns {
        let at = row * (columns + 1) + column;
        if left[row] == right[column] {
            pairs.push((row, column));
            row += 1;
            column += 1;
        } else if table[at + columns + 1] >= table[at + 1] {
            row += 1;
        } else {
            column += 1;
        }
    }
    pairs
}

/// The two paths on a `diff --git a/old b/new` line.
///
/// git separates them with a space and does not escape one inside a path, so a
/// file with a space in its name is genuinely ambiguous here. The `a/` and `b/`
/// prefixes are what disambiguate it: the split is taken at the space that has
/// ` b/` after it, and where no such split exists the line is left unnamed for
/// `+++` to fill in.
pub(crate) fn split_diff_git_paths(paths: &str) -> (Option<String>, Option<String>) {
    let mut search = 0;
    while let Some(offset) = paths[search..].find(" b/") {
        let at = search + offset;
        let (old, new) = (&paths[..at], &paths[at + 1..]);
        if let (Some(old), Some(new)) = (
            strip_diff_path_prefix(old.trim_matches('"')),
            strip_diff_path_prefix(new.trim_matches('"')),
        ) {
            return (Some(old), Some(new));
        }
        search = at + 1;
    }
    (None, None)
}

/// A path as it appears in a diff header, with the `a/` or `b/` git puts in
/// front of it taken off. `/dev/null` is not a path and returns `None`, which
/// is how a file that the change creates or deletes is recognised.
pub(crate) fn strip_diff_path_prefix(path: &str) -> Option<String> {
    // `--- a/x.rs\t2026-01-01` — `--no-index` against a file outside the
    // repository writes a timestamp after the path, separated by a tab.
    let path = path.split('\t').next().unwrap_or(path);
    let path = path.trim_matches('"');
    if path == "/dev/null" {
        return None;
    }
    path.strip_prefix("a/")
        .or_else(|| path.strip_prefix("b/"))
        .map(str::to_owned)
        // A `--no-index` diff of a path this app passed in whole has no
        // prefix to strip, and the path is still the path.
        .or_else(|| (!path.is_empty()).then(|| path.to_owned()))
}

/// The first line number on each side of an `@@ -a,b +c,d @@` header. A hunk
/// that covers one line writes `-a` with no count, which is why the comma is
/// optional here.
pub(crate) fn parse_hunk_header(line: &str) -> Option<(usize, usize)> {
    let body = line.strip_prefix("@@ ")?;
    let ranges = body.split(" @@").next()?;
    let (old, new) = ranges.split_once(' ')?;
    let start = |range: &str, sign: char| -> Option<usize> {
        range
            .strip_prefix(sign)?
            .split(',')
            .next()?
            .parse::<usize>()
            .ok()
    };
    Some((start(old, '-')?, start(new, '+')?))
}

/// The lines added and removed across a whole diff, for the summary above it.
pub(crate) fn diff_totals(files: &[DiffFile]) -> (usize, usize) {
    files.iter().fold((0, 0), |(added, removed), file| {
        (added + file.added, removed + file.removed)
    })
}

pub(crate) fn git_has_head(path: &Path) -> bool {
    git_output(path, &["rev-parse", "--verify", "HEAD"]).is_ok()
}

pub(crate) fn git_untracked_files(path: &Path) -> Result<Vec<String>> {
    let output = git_output(
        path,
        &["status", "--porcelain=v1", "-z", "--untracked-files=all"],
    )?;
    Ok(parse_porcelain_changed_files(&output)
        .into_iter()
        .filter(|file| file.status == "??")
        .map(|file| file.path)
        .collect())
}

pub(crate) fn git_working_tree_diff(path: &Path, file: Option<&str>) -> Result<String> {
    let mut arguments = vec!["diff", "--no-ext-diff", "--unified=3"];
    if git_has_head(path) {
        arguments.push("HEAD");
    } else {
        arguments.push("--cached");
    }
    if let Some(file) = file {
        arguments.extend(["--", file]);
    }
    let (tracked, mut truncated) =
        git_output_allowing_limited(path, &arguments, &[], GIT_DIFF_MAX_BYTES)?;
    let mut result = String::new();
    truncated |= append_diff_section(&mut result, &tracked, GIT_DIFF_MAX_BYTES);
    let untracked = if let Some(file) = file {
        git_untracked_files(path)?
            .into_iter()
            .filter(|candidate| candidate == file)
            .collect::<Vec<_>>()
    } else {
        git_untracked_files(path)?
    };
    for untracked_file in untracked {
        if truncated {
            break;
        }
        let remaining = GIT_DIFF_MAX_BYTES.saturating_sub(result.len());
        let (diff, output_truncated) = git_output_allowing_limited(
            path,
            &[
                "diff",
                "--no-ext-diff",
                "--no-index",
                "--unified=3",
                "--",
                "/dev/null",
                &untracked_file,
            ],
            &[1],
            remaining,
        )?;
        truncated = output_truncated || append_diff_section(&mut result, &diff, GIT_DIFF_MAX_BYTES);
    }
    if truncated {
        result.push_str(&tf!(
            "\n\n… 差分を {p0} KiB で切り詰めました",
            p0 = GIT_DIFF_MAX_BYTES / 1024
        ));
    }
    Ok(result)
}

pub(crate) fn append_diff_section(result: &mut String, section: &str, max_bytes: usize) -> bool {
    if section.trim().is_empty() {
        return false;
    }
    if !result.is_empty() {
        if result.len() >= max_bytes {
            return true;
        }
        result.push('\n');
    }
    let remaining = max_bytes.saturating_sub(result.len());
    if section.len() <= remaining {
        result.push_str(section);
        return false;
    }
    let mut end = remaining.min(section.len());
    while !section.is_char_boundary(end) {
        end -= 1;
    }
    result.push_str(&section[..end]);
    true
}
pub(crate) fn truncate_chars(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        text.to_owned()
    } else {
        format!("{}…", text.chars().take(max_chars).collect::<String>())
    }
}
/// Turn a path the UI holds relative to a project into an absolute one, having
/// established that it is still inside that project.
///
/// Both halves matter. The relative path comes from a scan of the project, but
/// it reaches this function by way of a `PathBuf` that lived in the UI across
/// an arbitrary number of frames, and the symlink it points through may have
/// been repointed since. Canonicalising both sides and comparing is what makes
/// "inside the project" a fact rather than an assumption — and every read and
/// every write goes through here so that there is exactly one copy of it.
pub(crate) fn resolve_project_file(root: &Path, relative: &Path) -> Result<PathBuf> {
    let root = root
        .canonicalize()
        .with_context(|| tf!("プロジェクトルート {p0} の解決", p0 = root.display()))?;
    let path = root
        .join(relative)
        .canonicalize()
        .with_context(|| format!("resolving file {}", relative.display()))?;
    if !path.starts_with(&root) {
        return Err(anyhow!(tf!("ファイルが選択中のプロジェクトの外にあります")));
    }
    Ok(path)
}

/// Read a file for the editor, saying why where it cannot be edited rather than
/// failing. A binary file and an oversized one are both ordinary things to
/// click on in a project tree, and neither is an error worth a red message.
pub(crate) fn read_file_for_editing(root: &Path, relative: &Path) -> Result<FileOpenOutcome> {
    let path = resolve_project_file(root, relative)?;
    let metadata = fs::metadata(&path)?;
    if is_image(relative) {
        if metadata.len() > EDITOR_IMAGE_MAX_BYTES as u64 {
            return Ok(FileOpenOutcome::Unopenable(tf!(
                "{p0} MiB の画像ファイルです。プレビューできるのは {p1} MiB までです。",
                p0 = metadata.len() / (1024 * 1024),
                p1 = EDITOR_IMAGE_MAX_BYTES / (1024 * 1024)
            )));
        }
        let bytes = fs::read(&path)?;
        return match image::load_from_memory(&bytes) {
            Ok(img) => Ok(FileOpenOutcome::Image {
                bytes,
                width: img.width(),
                height: img.height(),
            }),
            Err(_) => Ok(FileOpenOutcome::Unopenable(
                tr("画像ファイルをデコードできませんでした。").into(),
            )),
        };
    }
    if metadata.len() > EDITOR_FILE_MAX_BYTES as u64 {
        return Ok(FileOpenOutcome::Unopenable(tf!(
            "{p0} KiB のファイルです。エディタで開けるのは {p1} KiB までです。",
            p0 = metadata.len() / 1024,
            p1 = EDITOR_FILE_MAX_BYTES / 1024
        )));
    }
    let bytes = fs::read(&path)?;
    if bytes.contains(&0) {
        return Ok(FileOpenOutcome::Unopenable(
            tr("バイナリファイルはエディタで開けません。").into(),
        ));
    }
    match String::from_utf8(bytes) {
        Ok(text) => Ok(FileOpenOutcome::Text(text)),
        Err(_) => Ok(FileOpenOutcome::Unopenable(
            tr("UTF-8 ではないため、エディタで開けません。").into(),
        )),
    }
}

/// Read a file the editor already has open, and say nothing where it still
/// holds what the editor read.
///
/// This is `read_file_for_editing` plus one comparison, and it is deliberately
/// that rather than a reader of its own: the size ceiling, the binary check,
/// the UTF-8 check, and the path resolution are decisions that must not be made
/// twice, and a second reader would be a second place for them to drift.
///
/// The comparison is against the bytes rather than against a modification time.
/// A stamp says a write happened; the bytes say the file is different, and an
/// agent that rewrites a file with the content it already had would otherwise
/// raise a warning over nothing. See the first flagged concern in
/// `docs/sdlc/changes/010-the-editor-does-not-keep-up-with-what-agents-write/spec.md`
/// for the rest of that argument, including what it costs.
pub(crate) fn read_file_for_editing_if_changed(
    root: &Path,
    relative: &Path,
    known: &str,
) -> Result<Option<FileOpenOutcome>> {
    let outcome = read_file_for_editing(root, relative)?;
    // A file that became binary or oversized while it was open *is* a change,
    // and reaches the caller as one: it has something to say that the buffer on
    // screen does not.
    if matches!(&outcome, FileOpenOutcome::Text(text) if text == known) {
        return Ok(None);
    }
    Ok(Some(outcome))
}

/// Write the editor's buffer back, refusing where the file no longer holds what
/// the editor last read.
///
/// This app exists to run agents against the working tree a person is looking
/// at, so a file changing underneath an open editor is the normal case here,
/// not the exceptional one. A save that overwrote the newer bytes would delete
/// an agent's work silently, which is the one failure this pane must not have;
/// the person is told instead, and can reload and re-apply.
///
/// The write is atomic for the same reason the store's is: a crash between
/// truncating a source file and filling it back in loses the file, and the file
/// is somebody's work.
pub(crate) fn save_project_file(
    root: &Path,
    relative: &Path,
    expected: &str,
    contents: &str,
) -> Result<()> {
    let path = resolve_project_file(root, relative)?;
    let current =
        fs::read(&path).with_context(|| tf!("{p0} の読み込み", p0 = relative.display()))?;
    if current.as_slice() != expected.as_bytes() {
        return Err(anyhow!(tf!("このファイルは開いたあとに変更されています。上書きすると、その変更が失われます。再読み込みしてください。")));
    }
    let mode = fs::metadata(&path)
        .ok()
        .map(|metadata| metadata.permissions());
    write_project_file_atomically(&path, contents.as_bytes(), mode)?;
    Ok(())
}

/// The store's atomic write, with the original file's permissions carried onto
/// the replacement.
///
/// Kept separate from `write_file_atomically` on purpose: that one writes this
/// app's own store, where a fresh file with default permissions is exactly
/// right, and it is covered by the durability tests as it stands. A project
/// file is somebody else's — a hook, a shell script — and arriving back on disk
/// without its executable bit would be a silent breakage. The permissions go on
/// the temporary file before the rename, so the file at `path` is never
/// momentarily the wrong mode.
pub(crate) fn write_project_file_atomically(
    path: &Path,
    contents: &[u8],
    permissions: Option<fs::Permissions>,
) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!(tf!("保存先の親ディレクトリがありません")))?;
    let temporary = parent.join(format!(
        ".{}.tmp-{}",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("file"),
        Uuid::new_v4()
    ));
    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        file.write_all(contents)?;
        if let Some(permissions) = permissions {
            file.set_permissions(permissions)?;
        }
        file.sync_all()?;
        drop(file);
        fs::rename(&temporary, path)?;
        // A directory sync that fails leaves the rename committed but not
        // durable, which for a file the person can simply save again is worth
        // no notice of its own.
        let _ = fs::File::open(parent).and_then(|directory| directory.sync_all());
        Ok::<(), std::io::Error>(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result.map_err(Into::into)
}

pub(crate) fn git_branch(path: &Path) -> Result<String> {
    git_output(path, &["rev-parse", "--abbrev-ref", "HEAD"]).map(|branch| branch.trim().to_owned())
}
pub(crate) fn is_valid_branch_name(branch: &str) -> bool {
    // A question about a string, not a repository, so it is asked from a
    // directory that is never one.
    let mut command = git_command(Path::new("/"));
    command.args(["check-ref-format", "--branch", branch]);
    run_command_with_timeout(&mut command, Duration::from_secs(5))
        .is_ok_and(|output| output.status.success())
}
pub(crate) fn has_git_metadata(path: &Path) -> bool {
    path.join(".git").exists()
}
pub(crate) fn scan_git_repositories(workspace: &Path, max_depth: usize) -> (Vec<PathBuf>, bool) {
    scan_git_repositories_with_limit(workspace, max_depth, FILE_SCAN_VISIT_LIMIT)
}

pub(crate) fn scan_git_repositories_with_limit(
    workspace: &Path,
    max_depth: usize,
    visit_limit: usize,
) -> (Vec<PathBuf>, bool) {
    fn scan(
        current: &Path,
        depth: usize,
        max_depth: usize,
        visit_limit: usize,
        repositories: &mut Vec<PathBuf>,
        visited: &mut usize,
        truncated: &mut bool,
    ) {
        if *visited >= visit_limit {
            *truncated = true;
            return;
        }
        if has_git_metadata(current) {
            repositories.push(current.to_path_buf());
            return;
        }
        if depth >= max_depth {
            return;
        }
        let Ok(entries) = fs::read_dir(current) else {
            return;
        };
        for entry in entries.flatten() {
            if *visited >= visit_limit {
                *truncated = true;
                break;
            }
            *visited += 1;
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if file_type.is_symlink() {
                continue;
            }
            let path = entry.path();
            if !file_type.is_dir() {
                continue;
            }
            let name = entry.file_name();
            if matches!(
                name.to_str(),
                Some(".git" | "node_modules" | "target" | ".next")
            ) {
                continue;
            }
            scan(
                &path,
                depth + 1,
                max_depth,
                visit_limit,
                repositories,
                visited,
                truncated,
            );
        }
    }
    let mut repositories = Vec::new();
    let mut visited = 0;
    let mut truncated = false;
    scan(
        workspace,
        0,
        max_depth,
        visit_limit,
        &mut repositories,
        &mut visited,
        &mut truncated,
    );
    repositories.sort();
    (repositories, truncated)
}

pub(crate) fn worktree_destination(project: &Path, branch: &str) -> PathBuf {
    let project_name = project
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("project");
    let safe_branch = branch
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.') {
                character
            } else {
                '-'
            }
        })
        .collect::<String>();
    project
        .parent()
        .unwrap_or(project)
        .join(format!("{project_name}-worktrees"))
        .join(safe_branch)
}
pub(crate) fn list_worktrees(project: &Path) -> Result<Vec<Worktree>> {
    let output = git_output(project, &["worktree", "list", "--porcelain"])?;
    let main = project
        .canonicalize()
        .unwrap_or_else(|_| project.to_path_buf());
    let mut worktrees = Vec::new();
    for section in output.split("\n\n") {
        let mut path = None;
        let mut branch = None;
        for line in section.lines() {
            if let Some(value) = line.strip_prefix("worktree ") {
                path = Some(PathBuf::from(value));
            }
            if let Some(value) = line.strip_prefix("branch refs/heads/") {
                branch = Some(value.to_owned());
            }
        }
        if let Some(path) = path {
            let normalized = path.canonicalize().unwrap_or(path);
            worktrees.push(Worktree {
                is_main: normalized == main,
                path: normalized,
                branch,
            });
        }
    }
    Ok(worktrees)
}
/// Where a new worktree's branch starts. `qualified` is what git is given, so a
/// tag sharing the short name cannot win the lookup; `display` is what the
/// person is told they branched from.
pub(crate) struct WorktreeBase {
    pub(crate) qualified: String,
    pub(crate) display: String,
}

/// A worktree that now exists, described by the three things the person needs
/// told: where it is, what the branch ended up being called, and what it
/// started from.
#[derive(Debug)]
pub(crate) struct CreatedWorktree {
    pub(crate) destination: PathBuf,
    pub(crate) branch: String,
    pub(crate) base: String,
}

/// The project's mainline, or nothing. Ordered: what the remote says its own
/// default is, then the two names a remote uses when it has not said, then the
/// two a repository with no remote uses. Nothing is invented past those five —
/// a repository with none of them has no mainline, and branching from whatever
/// happens to be checked out is the defect this exists to replace.
pub(crate) fn detect_worktree_base(project: &Path) -> Option<WorktreeBase> {
    const FALLBACKS: [&str; 4] = [
        "refs/remotes/origin/main",
        "refs/remotes/origin/master",
        "refs/heads/main",
        "refs/heads/master",
    ];
    let symbolic = command_output_in_dir(
        "git",
        ["symbolic-ref", "--quiet", "refs/remotes/origin/HEAD"],
        project,
    )
    .ok()
    .map(|target| target.trim().to_owned())
    .filter(|target| !target.is_empty());
    symbolic
        .into_iter()
        .chain(FALLBACKS.iter().map(|candidate| (*candidate).to_owned()))
        .find(|candidate| git_ref_resolves(project, candidate))
        .map(|qualified| WorktreeBase {
            display: worktree_base_display(&qualified),
            qualified,
        })
}

/// Whether a fully qualified ref names a commit in this repository. The
/// `^{commit}` suffix is what makes it an answer about history rather than
/// about a name: a symref left dangling by a renamed default branch parses and
/// resolves to nothing.
fn git_ref_resolves(project: &Path, reference: &str) -> bool {
    let target = format!("{reference}^{{commit}}");
    command_output_in_dir(
        "git",
        ["rev-parse", "--verify", "--quiet", target.as_str()],
        project,
    )
    .is_ok_and(|output| !output.trim().is_empty())
}

fn worktree_base_display(reference: &str) -> String {
    reference
        .strip_prefix("refs/remotes/")
        .or_else(|| reference.strip_prefix("refs/heads/"))
        .unwrap_or(reference)
        .to_owned()
}

/// The name for one attempt. The first is what was asked for; the rest are
/// numbered from two, because a person counting their attempts at a task calls
/// the second one two.
pub(crate) fn worktree_name_candidate(name: &str, attempt: usize) -> String {
    if attempt == 0 {
        name.to_owned()
    } else {
        format!("{name}-{}", attempt + 1)
    }
}

/// Whether a name is free: no branch holds it, no directory sits at its
/// destination, and no registered worktree already claims that path. Three
/// positive checks rather than "the add failed", so that a real refusal from
/// git is reported to the person instead of being retried under another name.
pub(crate) fn is_worktree_name_available(project: &Path, branch: &str, destination: &Path) -> bool {
    if destination.exists() {
        return false;
    }
    if git_ref_resolves(project, &format!("refs/heads/{branch}")) {
        return false;
    }
    !list_worktrees(project).is_ok_and(|worktrees| {
        worktrees
            .iter()
            .any(|worktree| worktree.path == *destination)
    })
}

/// Record where the branch came from, and make its first push work.
///
/// `branch.<name>.base` is the durable answer to "what should this be compared
/// with", written where the next reader of the branch looks: git's own config,
/// not this application's store. `push.autoSetupRemote` is what makes the
/// `--no-track` on the add a safety rather than a chore — without tracking, a
/// first `git push` otherwise needs `--set-upstream`; with tracking, it would
/// aim at the mainline. Neither is worth failing a worktree that already
/// exists, so nothing is returned: a caller cannot mistake this for the result
/// of the creation.
pub(crate) fn configure_created_worktree(project: &Path, branch: &str, base: &str) {
    let key = format!("branch.{branch}.base");
    let _ = git_output(
        project,
        &["config", "--local", "--replace-all", key.as_str(), base],
    );
    let already_set = git_output(project, &["config", "--get", "push.autoSetupRemote"])
        .is_ok_and(|value| !value.trim().is_empty());
    if !already_set {
        let _ = git_output(
            project,
            &["config", "--local", "push.autoSetupRemote", "true"],
        );
    }
}

/// Create a worktree for `name`, starting from the project's mainline.
///
/// The name is a request, not a promise: when it is taken the next number is
/// used, which is what turns "run this task again" into a second worktree
/// beside the first instead of a refusal.
pub(crate) fn create_worktree_from_mainline(
    project: &Path,
    name: &str,
) -> std::result::Result<CreatedWorktree, String> {
    if !is_valid_branch_name(name) {
        return Err(tr("有効な Git ブランチ名ではありません。").into());
    }
    let Some(base) = detect_worktree_base(project) else {
        return Err(tr(
            "このリポジトリの起点ブランチを判定できませんでした。origin/HEAD、origin/main、main のいずれかが必要です。",
        )
        .into());
    };
    let mut last_candidate = name.to_owned();
    for attempt in 0..WORKTREE_NAME_MAX_ATTEMPTS {
        let branch = worktree_name_candidate(name, attempt);
        let destination = worktree_destination(project, &branch);
        last_candidate = branch.clone();
        if !is_worktree_name_available(project, &branch, &destination) {
            continue;
        }
        let mut command = git_command(project);
        command
            .args(["-C"])
            .arg(project)
            .args(["worktree", "add", "--no-track", "-b", &branch])
            .arg(&destination)
            .arg(&base.qualified);
        let output = run_command_with_timeout(&mut command, Duration::from_secs(30))
            .map_err(|error| tf!("Git を実行できませんでした: {error}", error = error))?;
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).trim().to_owned());
        }
        configure_created_worktree(project, &branch, &base.qualified);
        return Ok(CreatedWorktree {
            destination,
            branch,
            base: base.display,
        });
    }
    Err(tf!(
        "空いている worktree 名が見つかりませんでした（最後に試した名前: {p0}）。",
        p0 = last_candidate
    ))
}

pub(crate) fn is_git_worktree_for_project(project: &Path, candidate: &Path) -> bool {
    list_worktrees(project).is_ok_and(|worktrees| {
        worktrees
            .into_iter()
            .any(|worktree| worktree.path == candidate)
    })
}

pub(crate) fn is_allowed_session_path(project: &Path, candidate: &Path) -> bool {
    let normalized_project = project
        .canonicalize()
        .unwrap_or_else(|_| project.to_path_buf());
    let normalized_candidate = candidate
        .canonicalize()
        .unwrap_or_else(|_| candidate.to_path_buf());
    normalized_candidate == normalized_project
        || is_git_worktree_for_project(&normalized_project, &normalized_candidate)
}

/// Where this branch stands against the branch it pushes to.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct UpstreamState {
    /// `None` when the branch has never been pushed.
    pub(crate) upstream: Option<String>,
    pub(crate) ahead: usize,
    pub(crate) behind: usize,
}

/// Read the upstream and the two counts.
///
/// `--left-right --count` prints the two sides of a symmetric difference on one
/// line, left first: what the upstream has and this branch does not, then what
/// this branch has and the upstream does not. Left is behind and right is
/// ahead, and reading those the wrong way round would turn "you are behind,
/// pull first" into a push over somebody else's work.
pub(crate) fn git_upstream_state(path: &Path) -> Result<UpstreamState> {
    let Ok(upstream) = command_output_in_dir(
        "git",
        [
            "rev-parse",
            "--abbrev-ref",
            "--symbolic-full-name",
            "@{upstream}",
        ],
        path,
    ) else {
        return Ok(UpstreamState::default());
    };
    let upstream = upstream.trim().to_owned();
    if upstream.is_empty() {
        return Ok(UpstreamState::default());
    }
    let range = format!("{upstream}...HEAD");
    let counts = command_output_in_dir(
        "git",
        ["rev-list", "--left-right", "--count", range.as_str()],
        path,
    )?;
    let mut parts = counts.split_whitespace();
    let behind = parts
        .next()
        .and_then(|value| value.parse().ok())
        .unwrap_or(0);
    let ahead = parts
        .next()
        .and_then(|value| value.parse().ok())
        .unwrap_or(0);
    Ok(UpstreamState {
        upstream: Some(upstream),
        ahead,
        behind,
    })
}

/// The files whose staged content is neither `HEAD` nor the working tree.
///
/// That is what `git add -p` leaves behind: part of a file's changes in the
/// index and part not. It is a selection this application cannot express — it
/// stages whole files — so it is a selection this application must not silently
/// throw away. A file appearing in both the staged and the unstaged diff is
/// exactly that state, and Operon's own staging never produces it, which is
/// what makes this usable as a refusal rather than "is anything staged", a
/// check that would refuse forever after one failed commit.
pub(crate) fn partially_staged_files(path: &Path) -> Result<Vec<String>> {
    let names = |arguments: &[&str]| -> Result<Vec<String>> {
        let mut command = git_command(path);
        command.args(["-C"]).arg(path).args(arguments);
        let output = run_command_with_timeout(&mut command, Duration::from_secs(30))?;
        if !output.status.success() {
            return Err(anyhow!(String::from_utf8_lossy(&output.stderr)
                .trim()
                .to_owned()));
        }
        Ok(String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(str::to_owned)
            .filter(|line| !line.is_empty())
            .collect())
    };
    if !git_has_head(path) {
        return Ok(Vec::new());
    }
    let staged = names(&["diff", "--cached", "--name-only"])?;
    let unstaged = names(&["diff", "--name-only"])?;
    Ok(staged
        .into_iter()
        .filter(|file| unstaged.contains(file))
        .collect())
}

/// Make the index hold exactly these files and nothing else.
///
/// The reset comes first because a file staged by hand, and then unticked here,
/// would otherwise ride along in a change whose own list said it would not. It
/// is index-only: `git reset` without `--hard` never touches a working tree
/// file, which is worth saying because this is the line in this module that
/// looks most dangerous and is not.
///
/// It refuses outright on a hunk selection made outside this application. That
/// selection lives only in the index — the tree is never written to the object
/// store — so overwriting it destroys work with nothing to recover it from,
/// which is worse than not staging at all.
pub(crate) fn git_stage_exactly(path: &Path, files: &[String]) -> Result<()> {
    let partial = partially_staged_files(path)?;
    if !partial.is_empty() {
        return Err(anyhow!(tf!(
            "{files} は一部の変更だけがステージされています。Operon はファイル単位でしかステージできないため、その選択を上書きしてしまいます。ターミナルでコミットするか、`git reset` してからやり直してください。",
            files = partial.join(tr("、"))
        )));
    }
    if git_has_head(path) {
        let mut reset = git_command(path);
        reset
            .args(["-C"])
            .arg(path)
            .args(["reset", "--quiet", "--", "."]);
        // The status was dropped here, so a reset that never ran looked exactly
        // like one that did — and the `add` below would then succeed and carry
        // a hand-staged, unticked file into the commit. That is the one thing
        // the reset above exists to prevent.
        let output = run_command_with_timeout(&mut reset, Duration::from_secs(30))?;
        if !output.status.success() {
            return Err(anyhow!(String::from_utf8_lossy(&output.stderr)
                .trim()
                .to_owned()));
        }
    }
    if files.is_empty() {
        return Ok(());
    }
    let mut add = git_command(path);
    add.args(["-C"]).arg(path).args(["add", "--"]).args(files);
    let output = run_command_with_timeout(&mut add, Duration::from_secs(30))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(anyhow!(String::from_utf8_lossy(&output.stderr)
            .trim()
            .to_owned()))
    }
}

/// Record the staged change with this message.
///
/// The message travels as one argument in a vector, never through a shell, so a
/// message holding a quote, a newline, or a command substitution is a message.
pub(crate) fn git_record_staged(path: &Path, message: &str) -> Result<String> {
    let mut command = git_command(path);
    command
        .args(["-C"])
        .arg(path)
        .args(["commit", "-m", message]);
    let output = run_command_with_timeout(&mut command, Duration::from_secs(60))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
    } else {
        Err(anyhow!(String::from_utf8_lossy(&output.stderr)
            .trim()
            .to_owned()))
    }
}

/// Push, and never rewrite anything.
///
/// There is no force here in any spelling, and `no_source_file_can_force_a_push`
/// in `src/tests.rs` is what keeps it that way. This is the one action in
/// Operon that can destroy work belonging to somebody who is not at this
/// keyboard.
pub(crate) fn git_push(path: &Path, state: &UpstreamState, branch: &str) -> Result<String> {
    if state.behind > 0 {
        return Err(anyhow!(tr(
            "リモートが進んでいます。先に pull してください。"
        )));
    }
    let mut command = git_command(path);
    command.args(["-C"]).arg(path).arg("push");
    if state.upstream.is_none() {
        command.args(["--set-upstream", "origin", branch]);
    }
    let output = run_command_with_timeout(&mut command, Duration::from_secs(120))?;
    if output.status.success() {
        // git says everything useful about a push on stderr.
        Ok(String::from_utf8_lossy(&output.stderr).trim().to_owned())
    } else {
        Err(anyhow!(String::from_utf8_lossy(&output.stderr)
            .trim()
            .to_owned()))
    }
}

/// The staged change, as the two things a message is written from: which files,
/// and what happened inside them.
pub(crate) fn git_staged_summary(path: &Path) -> Result<(String, String)> {
    let (files, _) = git_output_allowing_limited(
        path,
        &["diff", "--cached", "--name-status"],
        &[],
        COMMIT_PROMPT_FILES_MAX_BYTES,
    )?;
    let (patch, truncated) = git_output_allowing_limited(
        path,
        &["diff", "--cached", "--no-ext-diff", "--unified=3"],
        &[],
        COMMIT_PROMPT_PATCH_MAX_BYTES,
    )?;
    // A patch cut off mid-hunk reads as a change that ends there, which is
    // worse than no patch at all: the model would describe half the work, and
    // describe it confidently.
    let patch = if truncated {
        String::from(
            "(diff omitted — too large to read; infer the change from the staged file list above)",
        )
    } else {
        patch
    };
    Ok((files, patch))
}

/// The prompt a message is drafted from.
///
/// The rules in it are what stop a model wrapping the one line that was wanted
/// in a paragraph of preamble, and they were arrived at by use rather than by
/// reasoning. The text is English because
/// it addresses a model and not a person: `tr` is for what people read.
pub(crate) fn commit_message_prompt(branch: &str, files: &str, patch: &str) -> String {
    format!(
        "You are generating a single git commit message.
Return only the commit message text. Do not include a preamble, quotes, or code fences.

Rules:
- First line: imperative mood, <= 72 chars, no trailing period.
- Optional body: blank line, then short wrapped prose explaining WHY.
- Capture the primary user-visible or developer-visible change.
- Use only the staged changes below as context.
- Do not include trailers such as Co-authored-by.

Branch: {branch}

Staged files:
{files}

Staged patch:
```diff
{patch}
```
"
    )
}

/// Strip what a model wraps a message in when it did not quite obey.
///
/// A fenced block, a fenced block with a language on it, the whole thing inside
/// double quotes, and leading blank lines: each has been seen from at least one
/// of the three CLIs. Stripping them is cheaper and more reliable than a
/// sterner prompt.
pub(crate) fn clean_generated_commit_message(raw: &str) -> String {
    let mut text = raw.trim().to_owned();
    if text.starts_with("```") {
        let after_fence = text.find('\n').map_or(text.len(), |at| at + 1);
        text = text[after_fence..].to_owned();
        if let Some(at) = text.rfind("```") {
            text.truncate(at);
        }
        text = text.trim().to_owned();
    }
    if text.len() >= 2 && text.starts_with('"') && text.ends_with('"') {
        text = text[1..text.len() - 1].trim().to_owned();
    }
    text
}

/// Strip ANSI escape codes so errors and outputs are clean for model prompts.
pub(crate) fn strip_ansi_codes(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            if chars.peek() == Some(&'[') {
                chars.next();
                while let Some(&next) = chars.peek() {
                    chars.next();
                    if ('@'..='~').contains(&next) || next == 'm' {
                        break;
                    }
                }
            }
        } else {
            out.push(ch);
        }
    }
    out
}

/// Summarize git commit failure (e.g. hook or linter failure) into a human-readable title.
pub(crate) fn summarize_commit_failure(raw: &str) -> String {
    let clean = strip_ansi_codes(raw);
    let lines: Vec<&str> = clean
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();
    if lines.is_empty() {
        return tr("コミットフック（pre-commit）の検証に失敗しました").into();
    }
    for line in &lines {
        let lower = line.to_lowercase();
        if lower.contains("clippy") {
            return tr("Cargo clippy でエラーが発生しました").into();
        }
        if lower.contains("cargo test")
            || lower.contains("test failed")
            || lower.contains("failed test")
        {
            return tr("テストの実行に失敗しました").into();
        }
        if lower.contains("pre-commit") || lower.contains("hook") {
            return tr("コミットフック（pre-commit）の検証に失敗しました").into();
        }
        if lower.contains("format") || lower.contains("cargo fmt") {
            return tr("コードフォーマットの検証に失敗しました").into();
        }
    }
    lines[0].to_string()
}

/// Build a structured recovery prompt when git commit fails.
pub(crate) fn build_fix_commit_failure_prompt(
    branch: &str,
    files: &[String],
    commit_message: &str,
    error: &str,
) -> String {
    let files_list = if files.is_empty() {
        "- (No files explicitly staged; check git status)".to_string()
    } else {
        files
            .iter()
            .map(|f| format!("- {f}"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let clean_error = strip_ansi_codes(error);
    let bounded_error = if clean_error.len() > 8000 {
        let head = &clean_error[..3000];
        let tail = &clean_error[clean_error.len() - 4000..];
        format!("{head}\n\n[...truncated...]\n\n{tail}")
    } else {
        clean_error
    };

    format!(
        "Fix this git commit failure and ensure all checks pass.

- Branch: {branch}
- Intended commit message: \"{commit_message}\"
- Staged files:
{files_list}

Failure output:
```
{bounded_error}
```

Rules:
- Identify the root cause from the failure output above (e.g. failing test, linter/compiler warning, or pre-commit hook).
- Inspect and edit the necessary files to fix the issue cleanly.
- Run relevant validation commands (e.g. cargo test, cargo clippy, or npm test) to confirm the fix.
- Ensure the git index and working tree are in a clean, commit-ready state.
- Reply with the root cause, what files you changed, and the validation results."
    )
}

/// Generate a prompt for automated AI evaluation of the diff.
pub(crate) fn ai_diff_review_prompt(branch: &str, patch: &str) -> String {
    format!(
        "You are an expert code reviewer evaluating an AI-generated code change.
Provide a concise, high-signal, objective review of the diff below.

Structure your review with these sections:
1. Summary: 1-2 sentences on what this diff does.
2. Correctness & Potential Bugs: edge cases, off-by-one errors, null/unhandled states, or logic flaws.
3. Safety & Regressions: breaking changes, security concerns, unhandled errors.
4. Test & Verification Coverage: whether tests cover new paths or if critical tests are missing.

Rules:
- Be constructive, precise, and specific to the lines changed.
- If the diff looks clean with no significant concerns, explicitly say so under the relevant sections.
- Do not include conversational greetings.

Branch: {branch}

Diff patch:
```diff
{patch}
```
"
    )
}

/// Strip any redundant markdown wrapping or code fences around an AI review if present.
pub(crate) fn clean_generated_ai_review(raw: &str) -> String {
    let mut text = raw.trim().to_owned();
    if text.starts_with("```") {
        let after_fence = text.find('\n').map_or(text.len(), |at| at + 1);
        text = text[after_fence..].to_owned();
        if let Some(at) = text.rfind("```") {
            text.truncate(at);
        }
        text = text.trim().to_owned();
    }
    text
}

pub(crate) fn list_pull_requests(project: &Path) -> Result<Vec<PullRequest>> {
    let output = command_output_in_dir(
        "gh",
        [
            "pr",
            "list",
            "--state",
            "open",
            "--limit",
            "30",
            "--json",
            "number,title,headRefName,isDraft,url,reviewDecision,statusCheckRollup",
        ],
        project,
    )?;
    serde_json::from_str(&output).context(tr("GitHub CLI のプルリクエストデータの解析"))
}
pub(crate) fn summarize_checks(checks: &[PrCheck]) -> String {
    if checks.is_empty() {
        return tr("なし").into();
    }
    let failed = checks
        .iter()
        .filter(|check| {
            matches!(
                check.conclusion.as_deref(),
                Some("FAILURE" | "CANCELLED" | "TIMED_OUT" | "ACTION_REQUIRED")
            )
        })
        .count();
    let pending = checks
        .iter()
        .filter(|check| {
            check
                .status
                .as_deref()
                .is_some_and(|status| status != "COMPLETED")
        })
        .count();
    if failed > 0 {
        tf!("{failed} 件が失敗", failed = failed)
    } else if pending > 0 {
        tf!("{pending} 件が実行中", pending = pending)
    } else {
        tf!("{p0} 件すべて成功", p0 = checks.len())
    }
}

/// GitHub reports its review state as an API constant. Show what it means
/// instead of the constant: this line is read at a glance, not parsed.
pub(crate) fn review_decision_label(decision: Option<&str>) -> &str {
    match decision {
        Some("APPROVED") => tr("承認済み"),
        Some("CHANGES_REQUESTED") => tr("変更を要求"),
        Some("REVIEW_REQUIRED") => tr("レビュー待ち"),
        None => tr("レビュー未設定"),
        Some(other) => other,
    }
}

/// Land a worktree branch back into the mainline branch.
///
/// Merges `branch` into mainline using `--no-ff`.
/// Rejects if worktree has uncommitted changes, if main checkout is dirty,
/// or if there are 0 commits ahead. On conflict, aborts merge and returns error.
/// Optionally removes worktree and deletes merged branch with `git branch -d`.
pub(crate) fn is_safe_ref_name(name: &str) -> bool {
    if name.is_empty() || name.starts_with('-') || name.contains(['\0', '\n', '\r', ' ']) {
        return false;
    }
    !name.contains("..")
        && !name.starts_with('/')
        && !name.ends_with('/')
        && !name.ends_with(".lock")
}

/// Merges `branch` into mainline using `--no-ff`.
/// Rejects if worktree has uncommitted changes, if main checkout is dirty or not on mainline,
/// or if there are 0 commits ahead. On conflict, aborts merge and returns error.
/// Optionally removes worktree and deletes merged branch with `git branch -d`.
pub(crate) fn git_land_worktree(
    project: &Path,
    worktree: &Path,
    branch: &str,
    auto_cleanup: bool,
) -> Result<LandResult> {
    if !is_safe_ref_name(branch) {
        anyhow::bail!(tf!(
            "無効または危険なブランチ名が指定されました: {branch}",
            branch = branch
        ));
    }

    let wt_status = git_output(worktree, &["status", "--porcelain"])?;
    if !wt_status.trim().is_empty() {
        anyhow::bail!(tr("worktree に未コミットの変更があるためマージできません。すべての変更をコミットまたは退避してください。"));
    }

    let base = detect_worktree_base(project).ok_or_else(|| {
        anyhow!(tr(
            "プロジェクトのメインラインブランチを検出できませんでした。"
        ))
    })?;
    let local_mainline = base
        .display
        .strip_prefix("origin/")
        .unwrap_or(&base.display);
    let mainline = local_mainline;

    if branch == mainline {
        anyhow::bail!(tf!(
            "マージ元とマージ先が同じブランチです ({mainline})。",
            mainline = mainline
        ));
    }

    let current_branch_raw = git_output(project, &["rev-parse", "--abbrev-ref", "HEAD"])?;
    let current_branch = current_branch_raw.trim();
    if current_branch != mainline {
        anyhow::bail!(tf!(
            "メイン作業ツリーが {mainline} ではなく {current} にあります。メイン作業ツリーで {mainline} をチェックアウトしてからマージしてください。",
            mainline = mainline,
            current = current_branch
        ));
    }

    // Ahead of the dirty check, because a merge whose resolution equals `HEAD`
    // leaves the status clean and only `MERGE_HEAD` says a person is in the
    // middle of it. Running our merge into theirs would fail, and undoing ours
    // would undo theirs.
    if merge_head_exists(project)? {
        anyhow::bail!(tr("メイン作業ツリーで別のマージが進行中のため取り込めません。そのマージを完了するか中止してから、もう一度お試しください。"));
    }

    let proj_status = git_output(project, &["status", "--porcelain"])?;
    if !proj_status.trim().is_empty() {
        anyhow::bail!(tr("メインの作業ツリーに未コミットの変更があるためマージできません。メイン作業ツリーをクリーンにしてください。"));
    }

    let range = format!("{mainline}..{branch}");
    let count_str = git_output(project, &["rev-list", "--count", &range, "--"])?;
    let commits_merged: usize = count_str.trim().parse().unwrap_or(0);
    if commits_merged == 0 {
        anyhow::bail!(tf!(
            "マージするコミットがありません（{branch} は {mainline} より進んでいません）。",
            branch = branch,
            mainline = mainline
        ));
    }

    let before = git_output(project, &["rev-parse", "HEAD"])?
        .trim()
        .to_owned();
    let mut merge_cmd = git_command(project);
    merge_cmd.args(["merge", "--no-ff", "--no-edit", "--", branch]);
    // A timeout or a spawn error is a failed merge too: either can leave
    // `MERGE_HEAD` behind, so it takes the same undo as a non-zero exit.
    let failure = match run_command_with_timeout(&mut merge_cmd, Duration::from_secs(60)) {
        Ok(output) if output.status.success() => None,
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
            Some(if stderr.is_empty() {
                String::from_utf8_lossy(&output.stdout).trim().to_owned()
            } else {
                stderr
            })
        }
        Err(error) => Some(error.to_string()),
    };
    if let Some(reason) = failure {
        anyhow::bail!(undo_failed_landing(project, mainline, &before, &reason));
    }

    let mut cleaned_up_worktree = false;
    let mut deleted_branch = false;
    if auto_cleanup {
        let mut rm_cmd = git_command(project);
        rm_cmd.args(["worktree", "remove", "--"]).arg(worktree);
        if let Ok(rm_out) = run_command_with_timeout(&mut rm_cmd, Duration::from_secs(30)) {
            if rm_out.status.success() {
                cleaned_up_worktree = true;
                let mut br_cmd = git_command(project);
                br_cmd.args(["branch", "-d", "--", branch]);
                if let Ok(br_out) = run_command_with_timeout(&mut br_cmd, Duration::from_secs(30)) {
                    deleted_branch = br_out.status.success();
                }
            }
        }
    }

    Ok(LandResult {
        mainline_branch: mainline.to_owned(),
        worktree_branch: branch.to_owned(),
        commits_merged,
        cleaned_up_worktree,
        deleted_branch,
    })
}

/// Whether the checkout at `path` is in the middle of a merge.
fn merge_head_exists(path: &Path) -> Result<bool> {
    // `--verify -q` exits 1 with no output when the ref is absent.
    let (output, _) = git_output_allowing_limited(
        path,
        &["rev-parse", "-q", "--verify", "MERGE_HEAD"],
        &[1],
        4096,
    )?;
    Ok(!output.trim().is_empty())
}

/// How many conflicting paths a landing notice names before it counts the rest.
const LANDING_CONFLICTS_SHOWN: usize = 10;

/// Put the main checkout back after a landing's merge stopped, and return the
/// sentence that says what happened — claiming only what was verified.
///
/// `merge --abort` runs only when `MERGE_HEAD` is present. `git_land_worktree`
/// refuses a merge already in progress, so any `MERGE_HEAD` here is this
/// landing's; the condition is defence in depth, not something a test can see.
/// Whether or not an abort ran, the checkout counts as unchanged only when
/// `HEAD` is the commit recorded before the merge, `MERGE_HEAD` is gone, and the
/// status is clean — which it was on entry, because `git_land_worktree` refuses
/// a dirty checkout. A merge killed after it began writing the tree but before
/// it wrote `MERGE_HEAD` leaves the first two true and only the third false.
fn undo_failed_landing(project: &Path, mainline: &str, before: &str, reason: &str) -> String {
    // An unreadable probe is treated as "started": an abort with nothing to
    // abort fails harmlessly, and the verification below still decides.
    let started = merge_head_exists(project).unwrap_or(true);
    let mut conflicts = Vec::new();
    let mut abort_failure = None;
    if started {
        if let Ok((output, _)) = git_output_allowing_limited(
            project,
            &["diff", "--name-only", "--diff-filter=U", "-z"],
            &[],
            64 * 1024,
        ) {
            conflicts = output
                .split('\0')
                .filter(|path| !path.is_empty())
                .map(str::to_owned)
                .collect::<Vec<_>>();
        }
        let mut abort = git_command(project);
        abort.args(["merge", "--abort"]);
        match run_command_with_timeout(&mut abort, Duration::from_secs(30)) {
            Ok(output) if output.status.success() => {}
            Ok(output) => {
                abort_failure = Some(String::from_utf8_lossy(&output.stderr).trim().to_owned());
            }
            Err(error) => abort_failure = Some(error.to_string()),
        }
    }

    let head_unchanged =
        git_output(project, &["rev-parse", "HEAD"]).is_ok_and(|head| head.trim() == before);
    let merge_gone = merge_head_exists(project).is_ok_and(|exists| !exists);
    // A stale `index.lock` makes `status` fail, and that is not clean either.
    let tree_clean = git_output(project, &["status", "--porcelain"])
        .is_ok_and(|status| status.trim().is_empty());
    if !(head_unchanged && merge_gone && tree_clean) {
        return tf!(
            "マージを取り消せませんでした。メイン作業ツリーにマージ途中の状態が残っています: {err}",
            err = abort_failure
                .filter(|text| !text.is_empty())
                .unwrap_or_else(|| reason.to_owned())
        );
    }
    if !started {
        return tf!(
            "マージを開始できませんでした。{mainline} は変更されていません: {err}",
            mainline = mainline,
            err = reason
        );
    }
    if conflicts.is_empty() {
        return tf!(
            "マージを完了できなかったため取り込みを取り消し、{mainline} を元の状態に戻しました: {err}",
            mainline = mainline,
            err = reason
        );
    }
    let mut files = conflicts
        .iter()
        .take(LANDING_CONFLICTS_SHOWN)
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join("、");
    if conflicts.len() > LANDING_CONFLICTS_SHOWN {
        files.push(' ');
        files.push_str(&tf!(
            "ほか {p0} 件",
            p0 = conflicts.len() - LANDING_CONFLICTS_SHOWN
        ));
    }
    tf!(
        "競合が発生したため取り込みを取り消し、{mainline} を元の状態に戻しました。競合したファイル: {files}",
        mainline = mainline,
        files = files
    )
}

/// Create a GitHub pull request using gh CLI.
pub(crate) fn git_create_pull_request(
    path: &Path,
    title: &str,
    body: &str,
    draft: bool,
) -> Result<String> {
    let mut args = vec!["pr", "create", "--title", title, "--body", body];
    if draft {
        args.push("--draft");
    }
    let mut cmd = Command::new("gh");
    forget_inherited_repository(&mut cmd);
    cmd.current_dir(path).args(&args);
    let output =
        run_command_with_timeout(&mut cmd, Duration::from_secs(60)).map_err(|e| anyhow!("{e}"))?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        anyhow::bail!("{err}");
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

/// Create a snapshot checkpoint of the worktree state at the current turn.
/// Uses an isolated temporary index file to capture all working tree edits
/// without modifying the user's active .git/index staging area.
pub(crate) fn git_create_checkpoint(
    worktree: &Path,
    session_id: Uuid,
    turn_index: usize,
    summary: &str,
) -> Result<Checkpoint> {
    let safe_summary: String = summary
        .chars()
        .filter(|c| !c.is_control() || *c == ' ')
        .take(200)
        .collect();

    let temp_index_path =
        std::env::temp_dir().join(format!("operon_checkpoint_idx_{session_id}_{turn_index}"));

    struct TempIndexGuard<'a>(&'a Path);
    impl<'a> Drop for TempIndexGuard<'a> {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(self.0);
        }
    }
    let _guard = TempIndexGuard(&temp_index_path);

    let mut add_cmd = git_command(worktree);
    add_cmd.env("GIT_INDEX_FILE", &temp_index_path);
    add_cmd.args(["add", "-A", "--", "."]);
    let add_out = run_command_with_timeout(&mut add_cmd, Duration::from_secs(30))
        .map_err(|e| anyhow!("{e}"))?;
    if !add_out.status.success() {
        let err = String::from_utf8_lossy(&add_out.stderr).trim().to_owned();
        anyhow::bail!("Failed to index working tree for checkpoint: {err}");
    }

    let mut write_cmd = git_command(worktree);
    write_cmd.env("GIT_INDEX_FILE", &temp_index_path);
    write_cmd.args(["write-tree"]);
    let write_out = run_command_with_timeout(&mut write_cmd, Duration::from_secs(30))
        .map_err(|e| anyhow!("{e}"))?;
    if !write_out.status.success() {
        let err = String::from_utf8_lossy(&write_out.stderr).trim().to_owned();
        anyhow::bail!("Failed to write tree for checkpoint: {err}");
    }
    let tree_sha = String::from_utf8_lossy(&write_out.stdout).trim().to_owned();

    let commit_msg = if safe_summary.is_empty() {
        format!("Checkpoint turn {turn_index}")
    } else {
        format!("Checkpoint turn {turn_index}: {safe_summary}")
    };
    let mut commit_cmd = git_command(worktree);
    commit_cmd.args(["commit-tree", &tree_sha, "-p", "HEAD", "-m", &commit_msg]);
    let commit_out = run_command_with_timeout(&mut commit_cmd, Duration::from_secs(30))
        .map_err(|e| anyhow!("{e}"))?;
    if !commit_out.status.success() {
        let err = String::from_utf8_lossy(&commit_out.stderr)
            .trim()
            .to_owned();
        anyhow::bail!("Failed to commit tree for checkpoint: {err}");
    }
    let commit_sha = String::from_utf8_lossy(&commit_out.stdout)
        .trim()
        .to_owned();

    let ref_name = format!("{CHECKPOINTS_REF_PREFIX}{session_id}/{turn_index}");
    let mut ref_cmd = git_command(worktree);
    ref_cmd.args(["update-ref", "--", &ref_name, &commit_sha]);
    let ref_out = run_command_with_timeout(&mut ref_cmd, Duration::from_secs(30))
        .map_err(|e| anyhow!("{e}"))?;
    if !ref_out.status.success() {
        let err = String::from_utf8_lossy(&ref_out.stderr).trim().to_owned();
        anyhow::bail!("Failed to update ref for checkpoint: {err}");
    }

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    Ok(Checkpoint {
        commit: commit_sha,
        turn_index,
        timestamp,
        summary: safe_summary,
    })
}

/// List all checkpoints recorded for a session.
pub(crate) fn git_list_checkpoints(worktree: &Path, session_id: Uuid) -> Result<Vec<Checkpoint>> {
    let prefix = format!("{CHECKPOINTS_REF_PREFIX}{session_id}/");
    let output = git_output(
        worktree,
        &[
            "for-each-ref",
            "--format=%(objectname) %(refname) %(contents:subject)",
            "--",
            &prefix,
        ],
    )?;
    let mut checkpoints = Vec::new();
    for line in output.lines() {
        let parts: Vec<&str> = line.splitn(3, ' ').collect();
        if parts.len() >= 2 {
            let commit = parts[0].to_owned();
            let refname = parts[1];
            let summary = parts.get(2).copied().unwrap_or("").to_owned();
            let turn_index = refname
                .strip_prefix(&prefix)
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            checkpoints.push(Checkpoint {
                commit,
                turn_index,
                timestamp: 0,
                summary,
            });
        }
    }
    checkpoints.sort_by_key(|c| c.turn_index);
    Ok(checkpoints)
}

/// Restore files in the worktree to match the state at a checkpoint.
pub(crate) fn git_restore_checkpoint(worktree: &Path, commit_sha: &str) -> Result<()> {
    if !commit_sha.chars().all(|c| c.is_ascii_hexdigit()) {
        anyhow::bail!("無効なコミット SHA です");
    }
    let mut cmd = git_command(worktree);
    cmd.args(["checkout", commit_sha, "--", "."]);
    let output =
        run_command_with_timeout(&mut cmd, Duration::from_secs(30)).map_err(|e| anyhow!("{e}"))?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        anyhow::bail!("{err}");
    }
    Ok(())
}

/// Diff changes between a checkpoint and the current working tree.
/// Output is bounded to 1 MB to prevent memory exhaustion.
pub(crate) fn git_diff_checkpoint(worktree: &Path, commit_sha: &str) -> Result<String> {
    if !commit_sha.chars().all(|c| c.is_ascii_hexdigit()) {
        anyhow::bail!("無効なコミット SHA です");
    }
    let (diff_text, truncated) =
        git_output_allowing_limited(worktree, &["diff", commit_sha, "--"], &[0, 1], 1024 * 1024)?;
    if truncated {
        Ok(format!(
            "{diff_text}\n\n[差分が 1MB を超えたため切り詰められました]"
        ))
    } else {
        Ok(diff_text)
    }
}

/// Query open GitHub issues for a repository using gh CLI.
pub(crate) fn list_github_issues(path: &Path) -> Result<Vec<GitHubIssue>> {
    let mut cmd = Command::new("gh");
    forget_inherited_repository(&mut cmd);
    cmd.current_dir(path).args([
        "issue",
        "list",
        "--state",
        "open",
        "--limit",
        "30",
        "--json",
        "number,title,body,labels",
    ]);
    let output =
        run_command_with_timeout(&mut cmd, Duration::from_secs(30)).map_err(|e| anyhow!("{e}"))?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        anyhow::bail!("{err}");
    }
    let json_text = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&json_text).map_err(|e| anyhow!("{e}"))
}

/// Return built-in and project-specific prompt templates.
pub(crate) fn list_prompt_templates(project_path: Option<&Path>) -> Vec<PromptTemplate> {
    let mut templates = vec![
        PromptTemplate {
            id: "review-diff".into(),
            title: tr("この差分をレビューして").into(),
            content: tr("これまでの変更差分（diff）を確認し、バグや品質上の問題がないかコードレビューしてください。").into(),
            is_custom: false,
        },
        PromptTemplate {
            id: "fix-ci".into(),
            title: tr("CIエラーを解決して").into(),
            content: tr("失敗している CI チェックやテストのエラーログを調査し、根本原因を特定して修正してください。").into(),
            is_custom: false,
        },
        PromptTemplate {
            id: "resolve-conflicts".into(),
            title: tr("マージコンフリクトを解消して").into(),
            content: tr("main ブランチとの競合（コンフリクト）を特定し、意図を壊さないよう安全に解決してください。").into(),
            is_custom: false,
        },
        PromptTemplate {
            id: "write-tests".into(),
            title: tr("テストを追加して").into(),
            content: tr("最近追加・変更された機能の境界条件や異常系をカバーする単体テストを作成してください。").into(),
            is_custom: false,
        },
    ];

    if let Some(project) = project_path {
        let custom_dir = project.join(PROMPT_TEMPLATES_RELATIVE_DIR);
        if let Ok(entries) = std::fs::read_dir(custom_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "md") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        let file_stem = path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("custom");
                        templates.push(PromptTemplate {
                            id: format!("custom-{file_stem}"),
                            title: file_stem.replace('-', " "),
                            content: content.trim().to_owned(),
                            is_custom: true,
                        });
                    }
                }
            }
        }
    }

    templates
}
