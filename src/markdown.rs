/// A block of Markdown, in as much of the syntax as an agent's notes actually
/// use. This is not a CommonMark implementation and does not try to be: it
/// covers headings, lists, quotes, fences, rules, and tables, and anything it
/// does not recognise stays a paragraph rather than disappearing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum MarkdownBlock {
    Heading {
        /// 1 to 6, as written.
        level: u8,
        text: String,
    },
    Paragraph(String),
    Bullet {
        /// How many levels in, counted from the indent two spaces at a time.
        depth: usize,
        /// The bullet or the number, kept as written so an ordered list keeps
        /// its numbering.
        marker: String,
        text: String,
        /// Whether this item is a checklist item, and if so whether it is
        /// ticked. `None` for an ordinary bullet, which is most of them.
        task: Option<bool>,
    },
    Quote(String),
    Code {
        /// The word after the opening fence, empty where there was none.
        language: String,
        lines: Vec<String>,
    },
    Rule,
    /// A whole table. Held as one block rather than a row at a time so the
    /// columns can be laid out against each other.
    Table {
        header: Vec<String>,
        rows: Vec<Vec<String>>,
    },
}

/// A run of text inside a block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum MarkdownSpan {
    Text(String),
    Strong(String),
    Emphasis(String),
    Strike(String),
    Code(String),
    Link { text: String, url: String },
}

/// Split Markdown into blocks.
///
/// Fences win over everything: text inside one is never re-read as Markdown,
/// which is what stops a `#` comment in a shell snippet from becoming a
/// heading. A fence that is never closed runs to the end of the file, the same
/// way every renderer treats it.
pub(crate) fn parse_markdown(text: &str) -> Vec<MarkdownBlock> {
    let mut blocks = Vec::new();
    let mut paragraph: Vec<String> = Vec::new();
    let mut lines = text
        .split('\n')
        .map(|line| line.trim_end_matches('\r'))
        .peekable();

    fn flush(paragraph: &mut Vec<String>, blocks: &mut Vec<MarkdownBlock>) {
        if !paragraph.is_empty() {
            blocks.push(MarkdownBlock::Paragraph(paragraph.join("\n")));
            paragraph.clear();
        }
    }

    while let Some(line) = lines.next() {
        let trimmed = line.trim_start();
        let indent = line.len() - trimmed.len();

        if let Some(fence) = trimmed
            .strip_prefix("```")
            .or_else(|| trimmed.strip_prefix("~~~"))
        {
            flush(&mut paragraph, &mut blocks);
            let closing = if trimmed.starts_with("```") {
                "```"
            } else {
                "~~~"
            };
            let mut body = Vec::new();
            for line in lines.by_ref() {
                if line.trim_start().starts_with(closing) {
                    break;
                }
                body.push(line.to_owned());
            }
            blocks.push(MarkdownBlock::Code {
                language: fence.trim().to_owned(),
                lines: body,
            });
            continue;
        }
        if trimmed.is_empty() {
            flush(&mut paragraph, &mut blocks);
            continue;
        }
        // `---` under a line of text is a setext heading rather than a rule,
        // and `***`/`___` are rules too.
        if is_thematic_break(trimmed) {
            if trimmed.starts_with('-') && !paragraph.is_empty() {
                let text = paragraph.join(" ");
                paragraph.clear();
                blocks.push(MarkdownBlock::Heading { level: 2, text });
                continue;
            }
            flush(&mut paragraph, &mut blocks);
            blocks.push(MarkdownBlock::Rule);
            continue;
        }
        if trimmed.starts_with('=')
            && trimmed.chars().all(|mark| mark == '=')
            && !paragraph.is_empty()
        {
            let text = paragraph.join(" ");
            paragraph.clear();
            blocks.push(MarkdownBlock::Heading { level: 1, text });
            continue;
        }
        if let Some((level, text)) = parse_atx_heading(trimmed) {
            flush(&mut paragraph, &mut blocks);
            blocks.push(MarkdownBlock::Heading { level, text });
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("> ").or(trimmed.strip_prefix(">")) {
            flush(&mut paragraph, &mut blocks);
            blocks.push(MarkdownBlock::Quote(rest.trim_start().to_owned()));
            continue;
        }
        if let Some((marker, rest)) = parse_list_marker(trimmed) {
            flush(&mut paragraph, &mut blocks);
            let (task, text) = split_task_marker(&rest);
            blocks.push(MarkdownBlock::Bullet {
                depth: indent / 2,
                marker,
                text: text.to_owned(),
                task,
            });
            continue;
        }
        // A table is a `|`-delimited line whose next line is the `---|---`
        // separator. Without that separator it is just a line with pipes in
        // it, which is a great deal more common than a table.
        if trimmed.starts_with('|') && lines.peek().is_some_and(|next| is_table_separator(next)) {
            flush(&mut paragraph, &mut blocks);
            let header = parse_table_row(trimmed);
            lines.next();
            let mut rows = Vec::new();
            while let Some(next) = lines.peek() {
                if !next.trim_start().starts_with('|') {
                    break;
                }
                rows.push(parse_table_row(next.trim_start()));
                lines.next();
            }
            blocks.push(MarkdownBlock::Table { header, rows });
            continue;
        }
        paragraph.push(trimmed.to_owned());
    }
    flush(&mut paragraph, &mut blocks);
    blocks
}

pub(crate) fn is_thematic_break(line: &str) -> bool {
    let stripped = line.replace(' ', "");
    stripped.len() >= 3
        && (stripped.chars().all(|mark| mark == '-')
            || stripped.chars().all(|mark| mark == '*')
            || stripped.chars().all(|mark| mark == '_'))
}

pub(crate) fn parse_atx_heading(line: &str) -> Option<(u8, String)> {
    let hashes = line.chars().take_while(|mark| *mark == '#').count();
    if hashes == 0 || hashes > 6 {
        return None;
    }
    let rest = &line[hashes..];
    // `#tag` is a word, not a heading. A heading has a space after its hashes.
    let text = rest.strip_prefix(' ')?;
    Some((
        hashes as u8,
        text.trim().trim_end_matches('#').trim().to_owned(),
    ))
}

/// The `-`, `*`, or `1.` at the head of a list item, with the text after it.
pub(crate) fn parse_list_marker(line: &str) -> Option<(String, String)> {
    for marker in ["- ", "* ", "+ "] {
        if let Some(rest) = line.strip_prefix(marker) {
            // A bullet, and deliberately not an `ICON_*`: it is punctuation
            // inside a line of prose rather than a pictogram standing in for a
            // word, so it has to come from the text face at the weight of the
            // words beside it. `the_typographic_marks_in_text_come_from_the_
            // text_face` is what holds that.
            //
            // A task list's `[ ]` is left in `rest` for `split_task_marker` to
            // take off, which is the one place that decision is made. This used
            // to argue that a checkbox is content rather than syntax and leave
            // the brackets in the text; the renderer had no box to draw at the
            // time, and that is the whole of the argument it was making.
            return Some(("•".to_owned(), rest.to_owned()));
        }
    }
    let digits = line.chars().take_while(char::is_ascii_digit).count();
    if digits == 0 {
        return None;
    }
    let rest = &line[digits..];
    let rest = rest
        .strip_prefix(". ")
        .or_else(|| rest.strip_prefix(") "))?;
    Some((format!("{}.", &line[..digits]), rest.to_owned()))
}

/// The `[ ]` or `[x]` at the head of a checklist item, and the text after it.
///
/// Agents write checklists constantly — a plan is a list of steps and a review
/// is a list of findings — so this is the one Markdown mark in this parser that
/// exists because of who writes the documents rather than because CommonMark
/// says so. It is deliberately strict: the brackets have to be at the head of
/// the item, hold exactly one of a space, an `x`, or an `X`, and be followed by
/// a space or by nothing. `[z] pick one` is a list item that starts with a
/// bracket, and `see [x](url)` is a link.
pub(crate) fn split_task_marker(text: &str) -> (Option<bool>, &str) {
    let Some(rest) = text.strip_prefix('[') else {
        return (None, text);
    };
    let mut marks = rest.chars();
    let ticked = match marks.next() {
        Some(' ') => false,
        Some('x' | 'X') => true,
        _ => return (None, text),
    };
    if marks.next() != Some(']') {
        return (None, text);
    }
    // Both marks consumed above are one byte, so the box is exactly two.
    let after = &rest[2..];
    match after.strip_prefix(' ') {
        Some(body) => (Some(ticked), body),
        // A box with nothing after it is still a box. An agent writing a
        // checklist a line at a time produces one every time.
        None if after.is_empty() => (Some(ticked), after),
        None => (None, text),
    }
}

pub(crate) fn is_table_separator(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with('|')
        && trimmed.contains('-')
        && trimmed
            .chars()
            .all(|mark| matches!(mark, '|' | '-' | ':' | ' '))
}

pub(crate) fn parse_table_row(line: &str) -> Vec<String> {
    line.trim()
        .trim_start_matches('|')
        .trim_end_matches('|')
        .split('|')
        .map(|cell| cell.trim().to_owned())
        .collect()
}

/// Split a line into its inline runs.
///
/// Markers only count where they close: an unmatched `*` in prose is an
/// asterisk, and a lone backtick in a sentence about backticks stays one.
pub(crate) fn parse_markdown_spans(text: &str) -> Vec<MarkdownSpan> {
    let mut spans = Vec::new();
    let mut plain = String::new();
    let bytes = text.as_bytes();
    let mut index = 0;

    fn flush(plain: &mut String, spans: &mut Vec<MarkdownSpan>) {
        if !plain.is_empty() {
            spans.push(MarkdownSpan::Text(std::mem::take(plain)));
        }
    }

    while index < bytes.len() {
        let rest = &text[index..];
        if let Some(body) = rest.strip_prefix("**") {
            if let Some(end) = body.find("**") {
                flush(&mut plain, &mut spans);
                spans.push(MarkdownSpan::Strong(body[..end].to_owned()));
                index += 2 + end + 2;
                continue;
            }
        }
        if let Some(body) = rest.strip_prefix("~~") {
            if let Some(end) = body.find("~~") {
                flush(&mut plain, &mut spans);
                spans.push(MarkdownSpan::Strike(body[..end].to_owned()));
                index += 2 + end + 2;
                continue;
            }
        }
        if let Some(body) = rest.strip_prefix('`') {
            if let Some(end) = body.find('`') {
                flush(&mut plain, &mut spans);
                spans.push(MarkdownSpan::Code(body[..end].to_owned()));
                index += 1 + end + 1;
                continue;
            }
        }
        if let Some(body) = rest.strip_prefix('*').or_else(|| rest.strip_prefix('_')) {
            let marker = &rest[..1];
            if let Some(end) = body.find(marker) {
                if end > 0 {
                    flush(&mut plain, &mut spans);
                    spans.push(MarkdownSpan::Emphasis(body[..end].to_owned()));
                    index += 1 + end + 1;
                    continue;
                }
            }
        }
        if let Some(body) = rest.strip_prefix('[') {
            if let Some(close) = body.find("](") {
                if let Some(end) = body[close + 2..].find(')') {
                    flush(&mut plain, &mut spans);
                    spans.push(MarkdownSpan::Link {
                        text: body[..close].to_owned(),
                        url: body[close + 2..close + 2 + end].to_owned(),
                    });
                    index += 1 + close + 2 + end + 1;
                    continue;
                }
            }
        }
        let width = rest.chars().next().map(char::len_utf8).unwrap_or(1);
        plain.push_str(&rest[..width]);
        index += width;
    }
    flush(&mut plain, &mut spans);
    spans
}
