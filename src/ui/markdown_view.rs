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

#[derive(Default)]
struct MarkdownBlockLayoutSlot {
    height: f32,
    galley: Option<Arc<egui::Galley>>,
}

#[derive(Default)]
struct MarkdownLayoutCache {
    key: Option<(u32, u32, SyntaxColours)>,
    slots: Vec<Option<MarkdownBlockLayoutSlot>>,
    code_key: Option<(u32, SyntaxColours)>,
    code_galleys: Vec<Option<Arc<egui::Galley>>>,
    galley_builds: usize,
    galley_hits: usize,
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
    layout_cache: std::sync::Mutex<MarkdownLayoutCache>,
}

impl RenderedMarkdown {
    #[cfg(test)]
    pub(crate) fn galley_builds(&self) -> usize {
        self.layout_cache
            .lock()
            .map(|c| c.galley_builds)
            .unwrap_or(0)
    }

    #[cfg(test)]
    pub(crate) fn galley_hits(&self) -> usize {
        self.layout_cache.lock().map(|c| c.galley_hits).unwrap_or(0)
    }
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
    RenderedMarkdown {
        blocks,
        layout_cache: std::sync::Mutex::default(),
    }
}

fn estimated_block_height(block: &RenderedBlock) -> f32 {
    match block {
        RenderedBlock::Heading { level, .. } => {
            if *level <= 2 {
                44.0
            } else {
                32.0
            }
        }
        RenderedBlock::Paragraph(_) => 26.0,
        RenderedBlock::Bullet { .. } | RenderedBlock::Quote(_) => 24.0,
        RenderedBlock::Code { .. } => 64.0,
        RenderedBlock::Rule => 18.0,
        RenderedBlock::Table { rows, .. } => 28.0 + rows.len() as f32 * 24.0,
    }
}

/// Render Markdown the way a person reads it rather than the way it is stored.
///
/// This is why the app has an editor at all: agents write their plans, notes,
/// and reviews as Markdown, and a plan read as raw syntax is a plan read in the
/// wrong format. It renders what agents write — headings, lists, checklists,
/// fences, quotes, tables, links — and leaves anything else as the text it was.
pub(crate) fn markdown_view(ui: &mut egui::Ui, document: &RenderedMarkdown, palette: &Palette) {
    let avail_w = ui.available_width().max(1.0);
    let width_key = (avail_w.round() as u32).max(1);
    let ppp_bits = ui.ctx().pixels_per_point().to_bits();
    let colours = SyntaxColours::from(palette);
    let key = (width_key, ppp_bits, colours);
    let cull = document.blocks.len() > MARKDOWN_VIEWPORT_CULL_THRESHOLD;
    let clip = ui.clip_rect();

    let code_key = (ppp_bits, colours);
    if let Ok(mut cache) = document.layout_cache.lock() {
        if cache.key.as_ref() != Some(&key) || cache.slots.len() != document.blocks.len() {
            cache.key = Some(key);
            cache.slots = (0..document.blocks.len()).map(|_| None).collect();
        }
        if cache.code_key.as_ref() != Some(&code_key)
            || cache.code_galleys.len() != document.blocks.len()
        {
            cache.code_key = Some(code_key);
            cache.code_galleys = vec![None; document.blocks.len()];
        }
    }

    for (idx, block) in document.blocks.iter().enumerate() {
        let (cached, cached_code) = document
            .layout_cache
            .lock()
            .ok()
            .map(|cache| {
                (
                    cache
                        .slots
                        .get(idx)
                        .and_then(|s| s.as_ref())
                        .map(|s| (s.height, s.galley.clone())),
                    cache.code_galleys.get(idx).and_then(|g| g.clone()),
                )
            })
            .unwrap_or((None, None));

        let y = ui.cursor().top();
        if cull {
            if let Some((height, _)) = &cached {
                if y + *height < clip.top() - 300.0 || y > clip.bottom() + 300.0 {
                    ui.allocate_space(egui::vec2(avail_w, *height));
                    continue;
                }
            } else if y > clip.bottom() + 300.0 {
                ui.allocate_space(egui::vec2(avail_w, estimated_block_height(block)));
                continue;
            }
        }

        let y_before = ui.cursor().top();
        let mut shaped_galley: Option<Arc<egui::Galley>> = None;
        match block {
            RenderedBlock::Heading { level, spans } => {
                ui.add_space(if *level <= 2 { 12.0 } else { 8.0 });
                let size = match level {
                    1 => 22.0,
                    2 => 18.0,
                    3 => 16.0,
                    _ => 14.0,
                };
                shaped_galley = markdown_line_cached(
                    ui,
                    spans,
                    palette,
                    MarkdownInk {
                        size,
                        strong: true,
                        color: palette.text_strong,
                    },
                    cached.as_ref().and_then(|(_, g)| g.as_ref()),
                    &document.layout_cache,
                );
                if *level <= 2 {
                    ui.add_space(2.0);
                    ui.separator();
                }
            }
            RenderedBlock::Paragraph(spans) => {
                ui.add_space(6.0);
                shaped_galley = markdown_line_cached(
                    ui,
                    spans,
                    palette,
                    MarkdownInk::body(palette),
                    cached.as_ref().and_then(|(_, g)| g.as_ref()),
                    &document.layout_cache,
                );
            }
            RenderedBlock::Bullet {
                depth,
                marker,
                task,
                spans,
            } => {
                ui.horizontal_top(|ui| {
                    ui.add_space(12.0 + *depth as f32 * 16.0);
                    if let Some(done) = task {
                        if marker != "•" {
                            ui.label(RichText::new(marker).size(14.0).color(palette.accent_soft));
                        }
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
                    .stroke(egui::Stroke::new(1.0_f32, palette.border_subtle))
                    .inner_margin(egui::Margin::symmetric(10, 8))
                    .show(ui, |ui| {
                        if !language.is_empty() {
                            ui.label(RichText::new(language).size(11.5).color(palette.text_faint));
                        }
                        if let Some(galley) = cached_code.as_ref() {
                            if let Ok(mut cache) = document.layout_cache.lock() {
                                cache.galley_hits += 1;
                            }
                            let (rect, _) =
                                ui.allocate_exact_size(galley.size(), egui::Sense::hover());
                            ui.painter()
                                .galley(rect.min, Arc::clone(galley), palette.text);
                            shaped_galley = Some(Arc::clone(galley));
                        } else {
                            let galley = ui.fonts_mut(|fonts| fonts.layout_job(job.clone()));
                            if let Ok(mut cache) = document.layout_cache.lock() {
                                cache.galley_builds += 1;
                                if cache.code_key.as_ref() == Some(&code_key) {
                                    if let Some(slot) = cache.code_galleys.get_mut(idx) {
                                        *slot = Some(Arc::clone(&galley));
                                    }
                                }
                            }
                            let (rect, _) =
                                ui.allocate_exact_size(galley.size(), egui::Sense::hover());
                            ui.painter()
                                .galley(rect.min, Arc::clone(&galley), palette.text);
                            shaped_galley = Some(galley);
                        }
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
        let delta = (ui.cursor().top() - y_before - ui.spacing().item_spacing.y).max(0.0);
        if let Ok(mut cache) = document.layout_cache.lock() {
            if cache.key.as_ref() == Some(&key) {
                if let Some(slot) = cache.slots.get_mut(idx) {
                    *slot = Some(MarkdownBlockLayoutSlot {
                        height: delta,
                        galley: shaped_galley.or_else(|| cached.and_then(|(_, g)| g)),
                    });
                }
            }
        }
    }
}

fn markdown_line_cached(
    ui: &mut egui::Ui,
    spans: &[MarkdownSpan],
    palette: &Palette,
    ink: MarkdownInk,
    cached_galley: Option<&Arc<egui::Galley>>,
    cache_lock: &std::sync::Mutex<MarkdownLayoutCache>,
) -> Option<Arc<egui::Galley>> {
    if spans
        .iter()
        .any(|span| matches!(span, MarkdownSpan::Link { .. }))
    {
        markdown_line(ui, spans, palette, ink);
        return None;
    }
    let avail_w = ui.available_width();
    if let Some(galley) = cached_galley {
        if let Ok(mut cache) = cache_lock.lock() {
            cache.galley_hits += 1;
        }
        let (rect, _) = ui.allocate_exact_size(galley.size(), egui::Sense::hover());
        ui.painter().galley(rect.min, Arc::clone(galley), ink.color);
        return Some(Arc::clone(galley));
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
    job.wrap.max_width = avail_w;
    let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
    if let Ok(mut cache) = cache_lock.lock() {
        cache.galley_builds += 1;
    }
    let (rect, _) = ui.allocate_exact_size(galley.size(), egui::Sense::hover());
    ui.painter()
        .galley(rect.min, Arc::clone(&galley), ink.color);
    Some(galley)
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
