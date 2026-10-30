use crate::prelude::*;
use crate::*;
use std::{str::FromStr, sync::OnceLock};
use syntect::{
    easy::HighlightLines,
    highlighting::{
        Color as SyntectColor, FontStyle, ScopeSelectors, StyleModifier, Theme, ThemeItem,
        ThemeSettings,
    },
    parsing::{SyntaxReference, SyntaxSet},
    util::LinesWithEndings,
};

/// The one active editor gets one bounded cache. It is deliberately not a
/// cache per tab: every open document may be 128 KiB, while only the front tab
/// is laid out each frame. Switching tabs costs one grammar pass; an unchanged
/// frame costs none.
#[derive(Default)]
pub(crate) struct EditorTextLayoutCache {
    key: Option<EditorTextLayoutKey>,
    layout: CachedEditorTextLayout,
    /// The read-only viewer's chunks. Kept here so the workspace's per-group
    /// loan of this cache loans the viewer's with it.
    pub(crate) viewer: ViewerLayoutCache,
    #[cfg(test)]
    builds: usize,
}

#[derive(Default)]
pub(crate) struct CachedEditorTextLayout {
    pub(crate) syntax: LayoutJob,
    pub(crate) line_count: usize,
    pub(crate) widest_line: usize,
    pub(crate) line_numbers: String,
}

#[derive(Clone, PartialEq, Eq)]
struct EditorTextLayoutKey {
    text: String,
    language: EditorLanguage,
    colours: SyntaxColours,
}

/// The roles the syntax theme reads, as a cache key. Shared with the rendered
/// Markdown cache next door rather than written out twice: both caches have to
/// rebuild when the theme changes, and two lists of roles would be two lists to
/// keep in step.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct SyntaxColours {
    text: Color32,
    text_muted: Color32,
    text_faint: Color32,
    accent_text: Color32,
    info: Color32,
    success: Color32,
    warning: Color32,
    sand: Color32,
}

impl From<&Palette> for SyntaxColours {
    fn from(palette: &Palette) -> Self {
        Self {
            text: palette.text,
            text_muted: palette.text_muted,
            text_faint: palette.text_faint,
            accent_text: palette.accent_text,
            info: palette.info,
            success: palette.success,
            warning: palette.warning,
            sand: palette.sand,
        }
    }
}

impl EditorTextLayoutCache {
    pub(crate) fn layout_for(
        &mut self,
        text: &str,
        language: EditorLanguage,
        palette: &Palette,
    ) -> &CachedEditorTextLayout {
        let colours = SyntaxColours::from(palette);
        let cache_hit = self.key.as_ref().is_some_and(|key| {
            key.text == text && key.language == language && key.colours == colours
        });
        if !cache_hit {
            let line_count = text.lines().count().max(1);
            let widest_line = text
                .lines()
                .map(|line| line.chars().count())
                .max()
                .unwrap_or(0);
            let digits = line_count.to_string().len();
            self.layout = CachedEditorTextLayout {
                syntax: syntax_layout(text, language, palette),
                line_count,
                widest_line,
                line_numbers: (1..=line_count)
                    .map(|number| format!("{number:>digits$}"))
                    .collect::<Vec<_>>()
                    .join("\n"),
            };
            self.key = Some(EditorTextLayoutKey {
                text: text.to_owned(),
                language,
                colours,
            });
            #[cfg(test)]
            {
                self.builds += 1;
            }
        }
        &self.layout
    }

    #[cfg(test)]
    pub(crate) fn builds(&self) -> usize {
        self.builds
    }
}

/// The rows of a document too large to edit, split once per read.
///
/// A row is only its start offset: four bytes, because a file of newlines has
/// as many rows as bytes. Its end is the next row's start with the line ending
/// trimmed, and whether it begins a line is whether the byte before it is a
/// newline — both are read off the text, not stored beside it.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct ViewerRows {
    /// Byte offset of each row. A line longer than `VIEWER_ROW_MAX_CHARS`
    /// continues on the rows after its first.
    pub(crate) starts: Vec<u32>,
    /// The 1-based line number of the first row of each chunk of
    /// `VIEWER_CHUNK_ROWS`, so a chunk can number its rows on its own.
    pub(crate) chunk_lines: Vec<u32>,
    pub(crate) lines: usize,
    /// The most characters in any row, which is how far the viewer scrolls
    /// sideways.
    pub(crate) widest: usize,
}

/// Split `text` into viewer rows. The caller holds at most
/// `VIEWER_FILE_MAX_BYTES`, far below `u32::MAX`; past `u32::MAX` the split
/// stops rather than wrapping an offset, and the rows end there.
pub(crate) fn viewer_rows(text: &str) -> ViewerRows {
    let mut rows = ViewerRows::default();
    let push = |rows: &mut ViewerRows, start: usize, line: u32| {
        let Ok(start) = u32::try_from(start) else {
            return false;
        };
        if rows.starts.len() % VIEWER_CHUNK_ROWS == 0 {
            rows.chunk_lines.push(line);
        }
        rows.starts.push(start);
        true
    };
    let mut offset = 0;
    let mut line = 0u32;
    'lines: for raw in text.split_inclusive('\n') {
        line += 1;
        if !push(&mut rows, offset, line) {
            break;
        }
        let content = raw.strip_suffix('\n').unwrap_or(raw);
        let content = content.strip_suffix('\r').unwrap_or(content);
        let mut chars = 0;
        for (index, _) in content.char_indices() {
            if chars == VIEWER_ROW_MAX_CHARS {
                rows.widest = rows.widest.max(chars);
                chars = 0;
                if !push(&mut rows, offset + index, line) {
                    break 'lines;
                }
            }
            chars += 1;
        }
        rows.widest = rows.widest.max(chars);
        offset += raw.len();
    }
    if rows.starts.is_empty() {
        // An empty document still has one, empty, first line.
        push(&mut rows, 0, 1);
        line = 1;
    }
    rows.lines = line as usize;
    rows
}

impl ViewerRows {
    pub(crate) fn len(&self) -> usize {
        self.starts.len()
    }

    fn begins_line(&self, text: &str, row: usize) -> bool {
        let start = self.starts[row] as usize;
        start == 0 || text.as_bytes().get(start - 1) == Some(&b'\n')
    }

    /// The bytes row `row` shows: up to the next row, without a line ending.
    pub(crate) fn row_range(&self, text: &str, row: usize) -> std::ops::Range<usize> {
        let start = self.starts[row] as usize;
        let end = self
            .starts
            .get(row + 1)
            .map_or(text.len(), |next| *next as usize);
        let shown = text.get(start..end).unwrap_or("");
        let shown = shown.strip_suffix('\n').unwrap_or(shown);
        let shown = shown.strip_suffix('\r').unwrap_or(shown);
        start..start + shown.len()
    }

    /// The row that begins 1-based line `line`, or the last row for a line past
    /// the end.
    pub(crate) fn row_of_line(&self, text: &str, line: usize) -> usize {
        let chunk = self
            .chunk_lines
            .partition_point(|first| (*first as usize) < line)
            .saturating_sub(1);
        let mut current = self
            .chunk_lines
            .get(chunk)
            .map_or(1, |first| *first as usize);
        let from = chunk * VIEWER_CHUNK_ROWS;
        for row in from..self.len() {
            if row != from && self.begins_line(text, row) {
                current += 1;
            }
            if current >= line && self.begins_line(text, row) {
                return row;
            }
        }
        self.len().saturating_sub(1)
    }
}

/// One row as the viewer draws it.
///
/// The line number and the text are one layout, drawn as one label: a row of
/// two widgets is as tall as a control, not as a line of text, and
/// `show_rows` places every row at the height of a line.
pub(crate) struct ViewerRowLayout {
    pub(crate) job: LayoutJob,
    /// Where the row's own text starts in `job.text`, after the line number
    /// (padded to the widest one, blank on a continuation row) and the gap.
    /// Only the tests take the row apart again; the frame draws `job` whole.
    #[cfg(test)]
    text_start: usize,
}

/// Between the line number and the text.
const VIEWER_GUTTER_GAP: &str = "  ";

#[cfg(test)]
impl ViewerRowLayout {
    pub(crate) fn gutter(&self) -> &str {
        &self.job.text[..self.text_start - VIEWER_GUTTER_GAP.len()]
    }

    pub(crate) fn text(&self) -> &str {
        &self.job.text[self.text_start..]
    }
}

/// Highlighted rows of the read-only viewer, one chunk at a time and only for
/// chunks that are drawn.
///
/// Keyed by the `Arc<ViewerRows>` of the read it was built for. The cache holds
/// a clone of it, so the allocation cannot be freed and handed to the next read
/// while the key is alive: pointer equality is identity here, and the frame
/// never compares megabytes of text to find out the document is unchanged.
#[derive(Default)]
pub(crate) struct ViewerLayoutCache {
    key: Option<(Arc<ViewerRows>, EditorLanguage, SyntaxColours)>,
    chunks: HashMap<usize, Vec<ViewerRowLayout>>,
    #[cfg(test)]
    builds: usize,
}

/// More chunks than any screen shows; past it the cache starts over rather
/// than growing with every chunk a long scroll passes through.
const VIEWER_CACHED_CHUNKS: usize = 64;

impl ViewerLayoutCache {
    pub(crate) fn row(
        &mut self,
        rows: &Arc<ViewerRows>,
        text: &str,
        language: EditorLanguage,
        palette: &Palette,
        row: usize,
    ) -> Option<&ViewerRowLayout> {
        if row >= rows.len() {
            return None;
        }
        let colours = SyntaxColours::from(palette);
        let current = self
            .key
            .as_ref()
            .is_some_and(|(key, key_language, key_colours)| {
                Arc::ptr_eq(key, rows) && *key_language == language && *key_colours == colours
            });
        if !current {
            self.chunks.clear();
            self.key = Some((Arc::clone(rows), language, colours));
        }
        let chunk = row / VIEWER_CHUNK_ROWS;
        if !self.chunks.contains_key(&chunk) {
            if self.chunks.len() >= VIEWER_CACHED_CHUNKS {
                self.chunks.clear();
            }
            self.chunks.insert(
                chunk,
                viewer_chunk_layout(rows, text, chunk, language, palette),
            );
            #[cfg(test)]
            {
                self.builds += 1;
            }
        }
        self.chunks
            .get(&chunk)
            .and_then(|layouts| layouts.get(row % VIEWER_CHUNK_ROWS))
    }

    #[cfg(test)]
    pub(crate) fn builds(&self) -> usize {
        self.builds
    }
}

/// Highlight one chunk as one piece of text and cut it at its rows.
///
/// The chunk starts from a fresh parser state, so a construct opened in an
/// earlier chunk — a `<script>` body, a block comment — may be coloured as its
/// surroundings until the grammar finds its footing again. That is the price of
/// not parsing megabytes up front, and it costs colour, never text.
pub(crate) fn viewer_chunk_layout(
    rows: &ViewerRows,
    text: &str,
    chunk: usize,
    language: EditorLanguage,
    palette: &Palette,
) -> Vec<ViewerRowLayout> {
    let first = chunk * VIEWER_CHUNK_ROWS;
    let last = (first + VIEWER_CHUNK_ROWS).min(rows.len());
    if first >= last {
        return Vec::new();
    }
    let ranges = (first..last)
        .map(|row| rows.row_range(text, row))
        .collect::<Vec<_>>();
    let base = ranges[0].start;
    let end = ranges[ranges.len() - 1].end;
    let job = syntax_layout(text.get(base..end).unwrap_or(""), language, palette);
    let digits = rows.lines.to_string().len();
    let mut line = rows
        .chunk_lines
        .get(chunk)
        .map_or(1, |first| *first as usize);
    ranges
        .iter()
        .enumerate()
        .map(|(offset, range)| {
            let begins = rows.begins_line(text, first + offset);
            if offset > 0 && begins {
                line += 1;
            }
            let gutter = if begins {
                format!("{line:>digits$}{VIEWER_GUTTER_GAP}")
            } else {
                " ".repeat(digits) + VIEWER_GUTTER_GAP
            };
            let mut row = LayoutJob::default();
            row.append(
                &gutter,
                0.0,
                TextFormat {
                    font_id: FontId::monospace(MONOSPACE_SIZE),
                    color: palette.text_muted,
                    ..Default::default()
                },
            );
            append_slice(&mut row, &job, range.start - base..range.end - base);
            ViewerRowLayout {
                job: row,
                #[cfg(test)]
                text_start: gutter.len(),
            }
        })
        .collect()
}

/// Append the part of `job` that covers `range` to `into`, its sections cut
/// to fit.
fn append_slice(into: &mut LayoutJob, job: &LayoutJob, range: std::ops::Range<usize>) {
    let offset = into.text.len();
    into.text
        .push_str(job.text.get(range.clone()).unwrap_or(""));
    for section in &job.sections {
        let start = section.byte_range.start.max(range.start);
        let end = section.byte_range.end.min(range.end);
        if start < end {
            into.sections.push(egui::text::LayoutSection {
                leading_space: 0.0,
                byte_range: offset + start - range.start..offset + end - range.start,
                format: section.format.clone(),
            });
        }
    }
    into.wrap.max_width = f32::INFINITY;
}

/// Build a semantic-colour layout from Syntect's bundled syntax definitions.
/// Parsing is constrained by the editor's 128 KiB input limit, and by the
/// read-only viewer's chunk of `VIEWER_CHUNK_ROWS` rows. The caller owns
/// the cache because it knows which document is visible; this pure function is
/// also used by tests to prove malformed text still has a safe fallback.
pub(crate) fn syntax_layout(text: &str, language: EditorLanguage, palette: &Palette) -> LayoutJob {
    if language == EditorLanguage::Plain {
        return plain_layout(text, palette);
    }
    let syntax_set = syntax_set();
    let syntax =
        syntax_for(syntax_set, language).unwrap_or_else(|| syntax_set.find_syntax_plain_text());
    let theme = semantic_theme(palette);
    let mut highlighter = HighlightLines::new(syntax, &theme);
    let mut layout = LayoutJob {
        text: text.to_owned(),
        ..Default::default()
    };

    for line in LinesWithEndings::from(text) {
        let Ok(ranges) = highlighter.highlight_line(line, syntax_set) else {
            return plain_layout(text, palette);
        };
        for (style, range) in ranges {
            let colour = Color32::from_rgba_unmultiplied(
                style.foreground.r,
                style.foreground.g,
                style.foreground.b,
                style.foreground.a,
            );
            layout.sections.push(egui::text::LayoutSection {
                leading_space: 0.0,
                byte_range: range_in(text, range),
                format: TextFormat {
                    font_id: FontId::monospace(MONOSPACE_SIZE),
                    color: colour,
                    italics: style.font_style.contains(FontStyle::ITALIC),
                    underline: if style.font_style.contains(FontStyle::UNDERLINE) {
                        egui::Stroke::new(1.0, colour)
                    } else {
                        egui::Stroke::NONE
                    },
                    ..Default::default()
                },
            });
        }
    }
    layout.wrap.max_width = f32::INFINITY;
    layout
}

fn plain_layout(text: &str, palette: &Palette) -> LayoutJob {
    LayoutJob::simple(
        text.to_owned(),
        FontId::monospace(MONOSPACE_SIZE),
        palette.text,
        f32::INFINITY,
    )
}

fn syntax_set() -> &'static SyntaxSet {
    static SYNTAX_SET: OnceLock<SyntaxSet> = OnceLock::new();
    SYNTAX_SET.get_or_init(SyntaxSet::load_defaults_newlines)
}

fn syntax_for(syntax_set: &SyntaxSet, language: EditorLanguage) -> Option<&SyntaxReference> {
    let tokens: &[&str] = match language {
        EditorLanguage::Plain => return None,
        EditorLanguage::Markdown => &["md", "Markdown"],
        EditorLanguage::Rust => &["rs", "Rust"],
        EditorLanguage::JavaScript => &["js", "JavaScript"],
        EditorLanguage::TypeScript => &["ts", "TypeScript", "js", "JavaScript"],
        EditorLanguage::Python => &["py", "Python"],
        EditorLanguage::Go => &["go", "Go"],
        EditorLanguage::Ruby => &["rb", "Ruby"],
        EditorLanguage::Shell => &["sh", "Shell Script"],
        EditorLanguage::C => &["c", "C"],
        EditorLanguage::Cpp => &["cpp", "C++"],
        EditorLanguage::Swift => &["swift", "Swift", "c", "C"],
        EditorLanguage::Kotlin => &["kt", "Kotlin", "java", "Java"],
        EditorLanguage::Java => &["java", "Java"],
        EditorLanguage::CSharp => &["cs", "C#", "cpp", "C++"],
        EditorLanguage::ObjectiveC => &["m", "Objective-C", "c", "C"],
        EditorLanguage::Php => &["php", "PHP", "html", "HTML"],
        // The bundled set does not promise a Vue grammar. HTML remains an
        // honest structural fallback and carries comments across lines.
        EditorLanguage::Vue | EditorLanguage::Html => &["html", "HTML"],
        EditorLanguage::Xml => &["xml", "XML"],
        EditorLanguage::Css => &["css", "CSS"],
        EditorLanguage::Sql => &["sql", "SQL"],
        EditorLanguage::Json => &["json", "JSON"],
        // The bundled set has no TOML definition. YAML's indentation and
        // comment grammar is the closest structural fallback, while still
        // leaving malformed TOML editable.
        EditorLanguage::Toml => &["toml", "TOML", "yaml", "YAML"],
        EditorLanguage::Yaml => &["yaml", "YAML"],
        EditorLanguage::Ini => &["ini", "INI", "yaml", "YAML"],
        EditorLanguage::Dockerfile => &["Dockerfile", "docker", "sh", "Shell Script"],
        EditorLanguage::Makefile => &["Makefile", "make", "sh", "Shell Script"],
        EditorLanguage::Lua => &["lua", "Lua"],
    };
    tokens
        .iter()
        .find_map(|token| syntax_set.find_syntax_by_token(token))
}

fn semantic_theme(palette: &Palette) -> Theme {
    Theme {
        name: Some("Operon semantic palette".into()),
        author: None,
        settings: ThemeSettings {
            foreground: Some(syntect_colour(palette.text)),
            background: Some(syntect_colour(palette.inset)),
            ..Default::default()
        },
        scopes: vec![
            theme_item("comment", palette.text_muted, Some(FontStyle::ITALIC)),
            theme_item("string", palette.success, None),
            theme_item("string.quoted.double.json", palette.info, None),
            theme_item("constant.numeric", palette.sand, None),
            theme_item("constant.language", palette.warning, None),
            theme_item("keyword", palette.accent_text, None),
            theme_item("storage", palette.accent_text, None),
            theme_item("meta.preprocessor", palette.accent_text, None),
            theme_item("entity.name", palette.info, None),
            theme_item("support.type", palette.info, None),
            theme_item("entity.name.tag", palette.info, None),
            theme_item("markup.heading", palette.accent_text, Some(FontStyle::BOLD)),
            theme_item("markup.bold", palette.accent_text, Some(FontStyle::BOLD)),
            theme_item("markup.italic", palette.success, Some(FontStyle::ITALIC)),
            theme_item("markup.raw", palette.success, None),
            theme_item(
                "markup.underline.link",
                palette.info,
                Some(FontStyle::UNDERLINE),
            ),
            theme_item("punctuation", palette.text_faint, None),
        ],
    }
}

fn theme_item(selector: &str, colour: Color32, font_style: Option<FontStyle>) -> ThemeItem {
    ThemeItem {
        scope: ScopeSelectors::from_str(selector).expect("static scope selector is valid"),
        style: StyleModifier {
            foreground: Some(syntect_colour(colour)),
            background: None,
            font_style,
        },
    }
}

fn syntect_colour(colour: Color32) -> SyntectColor {
    SyntectColor {
        r: colour.r(),
        g: colour.g(),
        b: colour.b(),
        a: colour.a(),
    }
}

fn range_in(whole: &str, range: &str) -> std::ops::Range<usize> {
    let whole_start = whole.as_ptr() as usize;
    let range_start = range.as_ptr() as usize;
    let start = range_start
        .checked_sub(whole_start)
        .expect("syntax range belongs to its source text");
    start..start + range.len()
}
