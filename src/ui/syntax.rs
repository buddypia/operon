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

/// Build a semantic-colour layout from Syntect's bundled syntax definitions.
/// Parsing is constrained by the editor's 128 KiB input limit. The caller owns
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
