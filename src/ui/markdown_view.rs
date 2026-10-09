use crate::prelude::*;
use crate::*;

/// One bounded cache for the one rendered document on screen, shaped like
/// `EditorTextLayoutCache` next door and there for the same reason: parsing and
/// highlighting are derived state, and the rendered view is the default for
/// Markdown, so the pane a person spends the most time in was the one redoing
/// the most work every frame.
///
/// Everything a frame needs that does not depend on the frame is built here:
/// the blocks, the inline runs inside them, and every fence's layout. What is
/// left in the draw path needs `ui` — assembling a `LayoutJob` from runs takes
/// `ui.style()`, and wrapping takes `ui.available_width()`.
#[derive(Default)]
pub(crate) struct MarkdownPreviewCache {
    key: Option<MarkdownPreviewKey>,
    document: RenderedMarkdown,
    #[cfg(test)]
    builds: usize,
}

#[derive(Clone, PartialEq, Eq)]
struct MarkdownPreviewKey {
    text: String,
    colours: SyntaxColours,
}

/// A Markdown document with everything parsed that can be parsed before there
/// is a frame to draw it into.
///
/// A rendered block rather than a `MarkdownBlock` beside a parallel vector of
/// layouts: the two had to be indexed in step, nothing in the types said so,
/// and filtering the blocks would have coloured the wrong fence. Here the
/// pairing cannot come apart. `src/markdown.rs` keeps its own types and stays a
/// parser with no drawing in it, testable without a font.
#[derive(Default)]
pub(crate) struct RenderedMarkdown {
    pub(crate) blocks: Vec<RenderedBlock>,
}

/// One block of a document, with its inline runs already found.
pub(crate) enum RenderedBlock {
    Heading {
        level: u8,
        spans: Vec<MarkdownSpan>,
    },
    Paragraph(Vec<MarkdownSpan>),
    Bullet {
        depth: usize,
        marker: String,
        task: Option<bool>,
        spans: Vec<MarkdownSpan>,
    },
    Quote(Vec<MarkdownSpan>),
    Code {
        /// The info string as written, which is what the corner of the frame
        /// says. `fence_language` decides the colouring; this is the label.
        language: String,
        /// Coloured where the fence named a language, plain monospace where it
        /// did not — one field either way, because the frame draws one thing.
        job: LayoutJob,
    },
    Rule,
    Table {
        header: Vec<String>,
        rows: Vec<Vec<Vec<MarkdownSpan>>>,
    },
}

impl MarkdownPreviewCache {
    pub(crate) fn document_for(&mut self, text: &str, palette: &Palette) -> &RenderedMarkdown {
        let colours = SyntaxColours::from(palette);
        let cache_hit = self
            .key
            .as_ref()
            .is_some_and(|key| key.text == text && key.colours == colours);
        if !cache_hit {
            self.document = render_markdown(text, palette);
            self.key = Some(MarkdownPreviewKey {
                text: text.to_owned(),
                colours,
            });
            #[cfg(test)]
            {
                self.builds += 1;
            }
        }
        &self.document
    }

    #[cfg(test)]
    pub(crate) fn builds(&self) -> usize {
        self.builds
    }
}

/// Parse a document all the way to the point where drawing it needs a frame.
pub(crate) fn render_markdown(text: &str, palette: &Palette) -> RenderedMarkdown {
    let blocks = parse_markdown(text)
        .into_iter()
        .map(|block| match block {
            MarkdownBlock::Heading { level, text } => RenderedBlock::Heading {
                level,
                spans: parse_markdown_spans(&text),
            },
            MarkdownBlock::Paragraph(text) => RenderedBlock::Paragraph(parse_markdown_spans(&text)),
            MarkdownBlock::Bullet {
                depth,
                marker,
                text,
                task,
            } => RenderedBlock::Bullet {
                depth,
                marker,
                task,
                spans: parse_markdown_spans(&text),
            },
            MarkdownBlock::Quote(text) => RenderedBlock::Quote(parse_markdown_spans(&text)),
            MarkdownBlock::Code { language, lines } => RenderedBlock::Code {
                // `Plain` is what `syntax_layout` reads as "leave this alone",
                // and what it returns for it is the monospace body an
                // unlabelled fence had before any of this existed.
                job: syntax_layout(&lines.join("\n"), fence_language(&language), palette),
                language,
            },
            MarkdownBlock::Rule => RenderedBlock::Rule,
            MarkdownBlock::Table { header, rows } => RenderedBlock::Table {
                header,
                rows: rows
                    .into_iter()
                    .map(|row| row.iter().map(|cell| parse_markdown_spans(cell)).collect())
                    .collect(),
            },
        })
        .collect();
    RenderedMarkdown { blocks }
}

/// Render Markdown the way a person reads it rather than the way it is stored.
///
/// This is why the app has an editor at all: agents write their plans, notes,
/// and reviews as Markdown, and a plan read as raw syntax is a plan read in the
/// wrong format. It renders what agents write — headings, lists, checklists,
/// fences, quotes, tables, links — and leaves anything else as the text it was.
pub(crate) fn markdown_view(ui: &mut egui::Ui, document: &RenderedMarkdown, palette: &Palette) {
    for block in &document.blocks {
        match block {
            RenderedBlock::Heading { level, spans } => {
                ui.add_space(if *level <= 2 { 12.0 } else { 8.0 });
                // Weight carries the hierarchy and size only widens the top of
                // it, the way the rest of the app does it: six sizes for six
                // heading levels would make an outline out of a document.
                let size = match level {
                    1 => 22.0,
                    2 => 18.0,
                    3 => 16.0,
                    _ => 14.0,
                };
                markdown_line(
                    ui,
                    spans,
                    palette,
                    MarkdownInk {
                        size,
                        strong: true,
                        color: palette.text_strong,
                    },
                );
                if *level <= 2 {
                    ui.add_space(2.0);
                    ui.separator();
                }
            }
            RenderedBlock::Paragraph(spans) => {
                ui.add_space(6.0);
                markdown_line(ui, spans, palette, MarkdownInk::body(palette));
            }
            RenderedBlock::Bullet {
                depth,
                marker,
                task,
                spans,
            } => {
                ui.horizontal_top(|ui| {
                    ui.add_space(12.0 + *depth as f32 * 16.0);
                    // An ordered checklist keeps its number: `1. [x] …` is a
                    // step that is both first and done, and dropping the `1.`
                    // to make room for the box loses the half a reader was
                    // counting with.
                    if let Some(done) = task {
                        if marker != "•" {
                            ui.label(RichText::new(marker).size(14.0).color(palette.accent_soft));
                        }
                        // The words beside either box stay body ink. A finished
                        // plan drawn in grey is a finished plan nobody can
                        // read, and the box already says which is which.
                        ui.label(
                            RichText::new(if *done {
                                ICON_TASK_DONE
                            } else {
                                ICON_TASK_TODO
                            })
                            .size(14.0)
                            .color(if *done {
                                palette.success
                            } else {
                                palette.text_faint
                            }),
                        );
                    } else {
                        ui.label(RichText::new(marker).size(14.0).color(palette.accent_soft));
                    }
                    markdown_line(ui, spans, palette, MarkdownInk::body(palette));
                });
            }
            RenderedBlock::Quote(spans) => {
                ui.add_space(4.0);
                egui::Frame::new()
                    .fill(palette.card)
                    .inner_margin(egui::Margin::symmetric(10, 6))
                    .show(ui, |ui| {
                        markdown_line(
                            ui,
                            spans,
                            palette,
                            MarkdownInk {
                                size: 14.0,
                                strong: false,
                                color: palette.text_muted,
                            },
                        );
                    });
            }
            RenderedBlock::Code { language, job } => {
                ui.add_space(6.0);
                egui::Frame::new()
                    .fill(palette.inset)
                    .stroke(egui::Stroke::new(1.0, palette.border_subtle))
                    .inner_margin(egui::Margin::symmetric(10, 8))
                    .show(ui, |ui| {
                        if !language.is_empty() {
                            ui.label(RichText::new(language).size(11.5).color(palette.text_faint));
                        }
                        // One clone of one fence's layout — its text and its
                        // sections, not a re-highlight. `egui::Label` takes the
                        // job by value and there is no borrowing form, and
                        // egui's galley cache keys on the job, so the layout
                        // itself is not redone. The parse and the grammar pass,
                        // which is where the cost was, are hoisted.
                        ui.add(
                            egui::Label::new(egui::WidgetText::from(job.clone()))
                                .wrap_mode(egui::TextWrapMode::Extend),
                        );
                    });
            }
            RenderedBlock::Rule => {
                ui.add_space(8.0);
                ui.separator();
            }
            RenderedBlock::Table { header, rows } => {
                ui.add_space(8.0);
                egui::Grid::new(ui.next_auto_id())
                    .striped(true)
                    .spacing(egui::vec2(14.0, 5.0))
                    .show(ui, |ui| {
                        for cell in header {
                            ui.label(RichText::new(cell).size(13.5).strong());
                        }
                        ui.end_row();
                        for row in rows {
                            for cell in row {
                                markdown_line(ui, cell, palette, MarkdownInk::body(palette));
                            }
                            ui.end_row();
                        }
                    });
            }
        }
    }
}

/// How a run of Markdown text is drawn before its own markers have their say.
#[derive(Clone, Copy)]
pub(crate) struct MarkdownInk {
    pub(crate) size: f32,
    pub(crate) strong: bool,
    pub(crate) color: Color32,
}

/// One line of Markdown with its inline markers applied.
///
/// egui lays a paragraph out as one widget, so a line that mixes bold, code,
/// and a link has to be one job rather than a row of labels — a row would let
/// the line break between the pieces instead of inside them. A link is the one
/// exception: it has to be its own widget to be clickable.
///
/// Takes runs rather than text: finding them is a parse, the parse does not
/// depend on the frame, and this is called once per block per frame.
pub(crate) fn markdown_line(
    ui: &mut egui::Ui,
    spans: &[MarkdownSpan],
    palette: &Palette,
    ink: MarkdownInk,
) {
    if spans
        .iter()
        .any(|span| matches!(span, MarkdownSpan::Link { .. }))
    {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            for span in spans {
                match span {
                    MarkdownSpan::Link { text, url } => {
                        ui.hyperlink_to(
                            RichText::new(text).size(ink.size).color(palette.link),
                            url,
                        );
                    }
                    other => {
                        ui.label(markdown_span_text(other, palette, ink));
                    }
                }
            }
        });
        return;
    }
    let mut job = LayoutJob::default();
    for span in spans {
        let rich = markdown_span_text(span, palette, ink);
        rich.append_to(
            &mut job,
            ui.style(),
            egui::FontSelection::Default,
            egui::Align::LEFT,
        );
    }
    job.wrap.max_width = ui.available_width();
    ui.add(egui::Label::new(job).wrap());
}

pub(crate) fn markdown_span_text(
    span: &MarkdownSpan,
    palette: &Palette,
    ink: MarkdownInk,
) -> RichText {
    match span {
        MarkdownSpan::Text(text) => {
            let rich = RichText::new(text).size(ink.size).color(ink.color);
            if ink.strong {
                rich.strong()
            } else {
                rich
            }
        }
        MarkdownSpan::Strong(text) => RichText::new(text)
            .size(ink.size)
            .color(palette.text_strong)
            .strong(),
        MarkdownSpan::Emphasis(text) => RichText::new(text)
            .size(ink.size)
            .color(ink.color)
            .italics(),
        // Struck-out text is text the author withdrew, so it is drawn at the
        // weight of a withdrawal: the line says it, and the fainter ink keeps a
        // page of revisions from being a page of noise.
        MarkdownSpan::Strike(text) => RichText::new(text)
            .size(ink.size)
            .color(palette.text_faint)
            .strikethrough(),
        MarkdownSpan::Code(text) => RichText::new(text)
            .monospace()
            .size(ink.size)
            .color(palette.accent_soft)
            .background_color(palette.code_bg),
        // Handled by the caller, which needs a widget rather than a run.
        MarkdownSpan::Link { text, .. } => RichText::new(text).size(ink.size).color(palette.link),
    }
}

impl MarkdownInk {
    fn body(palette: &Palette) -> Self {
        Self {
            size: 14.0,
            strong: false,
            color: palette.text,
        }
    }
}
