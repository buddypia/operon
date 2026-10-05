use crate::prelude::*;
use crate::*;

/// How many columns each of the two line-number gutters takes. Five digits
/// covers a hundred-thousand-line file, and a fixed width is what keeps the
/// code in a diff starting at the same x on every row.
pub(crate) const DIFF_GUTTER_COLUMNS: usize = 5;

/// One row of the review pane: the bar that names a file, or a line of one of
/// its hunks. Flattening the two into a single list is what lets the whole
/// review — every file, every hunk — scroll as one virtualised column instead
/// of as a stack of independently scrolling panes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DiffRow {
    File(usize),
    Line(usize, usize),
    /// A note written against the line above it, drawn inside the same
    /// scrolling column so that scrolling can never separate the two.
    Comment(usize, usize),
}

pub(crate) fn diff_pane_rows(
    files: &[DiffFile],
    collapsed: &HashSet<String>,
    annotated: &dyn Fn(&str, usize) -> bool,
) -> Vec<DiffRow> {
    let mut rows = Vec::new();
    for (index, file) in files.iter().enumerate() {
        rows.push(DiffRow::File(index));
        if collapsed.contains(&file.path) {
            continue;
        }
        for (line, content) in file.lines.iter().enumerate() {
            rows.push(DiffRow::Line(index, line));
            if content
                .new_number
                .is_some_and(|number| annotated(&file.path, number))
            {
                rows.push(DiffRow::Comment(index, line));
            }
        }
    }
    rows
}

/// Where every row starts, and where the last one ends.
///
/// The pane used to divide by one row height, because every row was one line
/// tall. A note is taller, so the offsets are accumulated instead and the
/// visible range is found by searching them. Returned as `offsets.len() ==
/// rows.len() + 1`, so the height of row `i` is `offsets[i + 1] - offsets[i]`
/// and the total is the last entry — one shape, no special case for the end.
pub(crate) fn diff_row_offsets(rows: &[DiffRow], line_height: f32) -> Vec<f32> {
    let mut offsets = Vec::with_capacity(rows.len() + 1);
    let mut cursor = 0.0;
    for row in rows {
        offsets.push(cursor);
        cursor += match row {
            DiffRow::Comment(_, _) => line_height * DIFF_COMMENT_ROW_LINES,
            _ => line_height,
        };
    }
    offsets.push(cursor);
    offsets
}

/// The rows a viewport touches, including the ones it only partly covers.
///
/// A row that begins above the viewport and ends inside it is visible, and
/// dropping it is how a note disappears while it is being scrolled through.
/// `partition_point` finds the first row whose *end* is past the top, and the
/// first row whose *start* is at or past the bottom.
pub(crate) fn diff_visible_rows(offsets: &[f32], top: f32, bottom: f32) -> std::ops::Range<usize> {
    if offsets.len() < 2 {
        return 0..0;
    }
    let rows = offsets.len() - 1;
    let first = offsets[1..].partition_point(|end| *end <= top);
    let last = offsets[..rows].partition_point(|start| *start < bottom);
    first.min(rows)..last.max(first.min(rows))
}

/// The label above a file's hunks: the count of what changed, in the two inks
/// that mean added and removed.
pub(crate) fn diff_counts(ui: &mut egui::Ui, added: usize, removed: usize, palette: &Palette) {
    if added > 0 {
        ui.label(
            RichText::new(format!("+{added}"))
                .size(12.0)
                .color(palette.diff_added),
        );
    }
    if removed > 0 {
        // A minus sign rather than a hyphen: this is a count, and it is read
        // next to a `+`.
        ui.label(
            RichText::new(format!("−{removed}"))
                .size(12.0)
                .color(palette.diff_removed),
        );
    }
    if added == 0 && removed == 0 {
        ui.label(
            RichText::new(tr("内容の変更なし"))
                .size(12.0)
                .color(palette.text_faint),
        );
    }
}

/// Draw a parsed diff, laying out only the rows the viewport can show, and the
/// notes a person has written against those lines.
///
/// Nothing here re-wraps a line. Which column a change sits in is part of
/// reading a diff, so the pane scrolls sideways instead — the same bargain the
/// terminal pane makes, for the same reason.
///
/// Rows are no longer all one line tall, so the visible range comes from a
/// search over `diff_row_offsets` rather than from a division. Both that and
/// the search are pure and pinned in `src/tests.rs`: lesson 005 is that only
/// the caller proves the screen, so the arithmetic is kept somewhere a test
/// can reach it.
///
/// Returns whether the set of notes changed and needs writing down.
pub(crate) fn diff_pane(
    ui: &mut egui::Ui,
    files: &[DiffFile],
    collapsed: &mut HashSet<String>,
    palette: &Palette,
    height: f32,
    project: Uuid,
    annotations: &mut DiffAnnotations,
) -> bool {
    if files.is_empty() {
        ui.label(RichText::new(tr("変更はありません。")).color(palette.text_muted));
        return false;
    }
    let cell = monospace_cell_size(ui);
    let rows = diff_pane_rows(files, collapsed, &|file, line| {
        annotations.shows_row(project, file, line)
    });
    let offsets = diff_row_offsets(&rows, cell.y);
    let total_height = offsets.last().copied().unwrap_or(0.0);
    let widest = files.iter().map(|file| file.widest).max().unwrap_or(0);
    // Two gutters, the sign, and a space after it, then the widest line in the
    // diff. The `max` keeps a diff of short lines from shrinking its own rows
    // to less than the pane it is drawn in.
    let content_width =
        ((DIFF_GUTTER_COLUMNS * 2 + 2 + widest) as f32 * cell.x + 16.0).max(ui.available_width());
    let mut toggled = None;
    let mut opened = None;
    let mut changed = false;

    ui.spacing_mut().item_spacing.y = 0.0;
    egui::ScrollArea::both()
        .auto_shrink([false, false])
        .max_height(height)
        .show_viewport(ui, |ui, viewport| {
            // The rows are placed at absolute offsets rather than by advancing
            // a cursor, so the scroll extent has to be claimed up front or the
            // area would believe it holds one row.
            ui.set_min_size(egui::vec2(content_width, total_height));
            let origin = ui.min_rect().min;
            for index in diff_visible_rows(&offsets, viewport.min.y, viewport.max.y) {
                let rect = egui::Rect::from_min_size(
                    egui::pos2(origin.x, origin.y + offsets[index]),
                    egui::vec2(content_width, offsets[index + 1] - offsets[index]),
                );
                match rows[index] {
                    DiffRow::File(file) => {
                        let file = &files[file];
                        let open = !collapsed.contains(&file.path);
                        let response = ui.interact(
                            rect,
                            ui.id().with(("diff-file-row", &file.path)),
                            egui::Sense::click(),
                        );
                        ui.painter().rect_filled(
                            rect,
                            0.0,
                            if response.hovered() {
                                palette.control_hovered
                            } else {
                                palette.raised
                            },
                        );
                        if let Some(stroke) = focus_stroke(response.has_focus(), palette) {
                            ui.painter()
                                .rect_stroke(rect, 0.0, stroke, egui::StrokeKind::Inside);
                        }
                        ui.painter().hline(
                            rect.x_range(),
                            rect.top(),
                            egui::Stroke::new(1.0, palette.border_subtle),
                        );
                        let mut row = ui.new_child(
                            egui::UiBuilder::new()
                                .id_salt(("diff-file", &file.path))
                                .max_rect(rect.shrink2(egui::vec2(6.0, 0.0)))
                                .layout(egui::Layout::left_to_right(egui::Align::Center)),
                        );
                        row.spacing_mut().item_spacing.x = 6.0;
                        row.label(
                            RichText::new(if open {
                                ICON_DISCLOSURE_OPEN
                            } else {
                                ICON_DISCLOSURE_CLOSED
                            })
                            .size(12.0)
                            .color(palette.text_muted),
                        );
                        row.label(
                            RichText::new(match file.change {
                                DiffFileChange::Added => ICON_FILE_ADDED,
                                DiffFileChange::Removed => ICON_FILE_REMOVED,
                                _ => file_tree_icon(&file.path),
                            })
                            .size(13.0)
                            .color(match file.change {
                                DiffFileChange::Added => palette.diff_added,
                                DiffFileChange::Removed => palette.diff_removed,
                                _ => palette.text_muted,
                            }),
                        );
                        // A rename is said in words rather than given a glyph
                        // of its own: "old → new" is the whole fact, and no
                        // mark carries it.
                        let name = match &file.renamed_from {
                            Some(from) => format!("{from} → {}", file.path),
                            None => file.path.clone(),
                        };
                        row.label(RichText::new(name).size(12.5).color(palette.text_strong));
                        diff_counts(&mut row, file.added, file.removed, palette);
                        if response.clicked() {
                            toggled = Some(file.path.clone());
                        }
                    }
                    DiffRow::Line(file, line) => {
                        let path = files[file].path.clone();
                        let line = &files[file].lines[line];
                        let wash = match line.kind {
                            DiffLineKind::Added => Some(palette.diff_added_bg),
                            DiffLineKind::Removed => Some(palette.diff_removed_bg),
                            _ => None,
                        };
                        if let Some(wash) = wash {
                            ui.painter().rect_filled(rect, 0.0, wash);
                        }
                        if line.kind == DiffLineKind::Hunk {
                            // A hairline is the whole of a hunk break. A filled
                            // band would be a fourth colour on a screen whose
                            // colours already all mean something.
                            ui.painter().hline(
                                rect.x_range(),
                                rect.top(),
                                egui::Stroke::new(1.0, palette.border_subtle),
                            );
                        }
                        let job = diff_line_layout(line, cell.x, palette);
                        let galley = ui.fonts(|fonts| fonts.layout_job(job));
                        ui.painter()
                            .galley(rect.min + egui::vec2(8.0, 0.0), galley, palette.text);
                        // Only a line that exists after the change can carry a
                        // note: a removed line has no line number on the side
                        // the agent would go looking at.
                        if let Some(number) = line.new_number {
                            let response = ui.interact(
                                rect,
                                ui.id().with(("diff-line", &path, number)),
                                egui::Sense::click(),
                            );
                            if response.hovered()
                                || response.has_focus()
                                || annotations.shows_row(project, &path, number)
                            {
                                ui.painter().text(
                                    rect.right_center() - egui::vec2(8.0, 0.0),
                                    egui::Align2::RIGHT_CENTER,
                                    ICON_COMMENT,
                                    egui::FontId::proportional(12.0),
                                    if annotations.find(project, &path, number).is_some() {
                                        palette.accent_soft
                                    } else {
                                        palette.text_muted
                                    },
                                );
                            }
                            if let Some(stroke) = focus_stroke(response.has_focus(), palette) {
                                ui.painter().rect_stroke(
                                    rect,
                                    0.0,
                                    stroke,
                                    egui::StrokeKind::Inside,
                                );
                            }
                            if response.clicked() {
                                opened = Some((path, number));
                            }
                        }
                    }
                    DiffRow::Comment(file, line) => {
                        let path = files[file].path.clone();
                        let source = &files[file].lines[line];
                        let Some(number) = source.new_number else {
                            continue;
                        };
                        let stale = annotations
                            .find(project, &path, number)
                            .is_some_and(|comment| comment.anchor != comment_anchor(&source.text));
                        let editing = annotations
                            .editing
                            .as_ref()
                            .is_some_and(|(open, at)| open == &path && *at == number);
                        ui.painter().rect_filled(
                            rect.shrink2(egui::vec2(8.0, 2.0)),
                            4.0,
                            palette.raised,
                        );
                        let mut note = ui.new_child(
                            egui::UiBuilder::new()
                                .id_salt(("diff-comment", &path, number))
                                .max_rect(rect.shrink2(egui::vec2(16.0, 6.0)))
                                .layout(egui::Layout::top_down(egui::Align::LEFT)),
                        );
                        let (comment_sent, comment_resolved) = annotations
                            .find(project, &path, number)
                            .map(|c| (c.sent, c.resolved))
                            .unwrap_or((false, false));
                        note.horizontal(|ui| {
                            ui.label(
                                RichText::new(tf!(
                                    "{ICON_COMMENT} {p0}:{p1}",
                                    ICON_COMMENT = ICON_COMMENT,
                                    p0 = path,
                                    p1 = number
                                ))
                                .size(11.5)
                                .color(palette.text_muted),
                            );
                            if comment_sent {
                                ui.label(
                                    RichText::new(format!("[{}]", tr("送信済み")))
                                        .size(11.0)
                                        .color(palette.accent),
                                );
                            }
                            if comment_resolved {
                                ui.label(
                                    RichText::new(format!("[{}]", tr("解決済み")))
                                        .size(11.0)
                                        .color(palette.success),
                                );
                            }
                            if stale {
                                ui.label(
                                    RichText::new(tr("行が変わっています"))
                                        .size(11.5)
                                        .color(palette.warning),
                                );
                            }
                        });
                        if editing {
                            let editor = note.add(
                                egui::TextEdit::multiline(&mut annotations.draft)
                                    .desired_rows(2)
                                    .desired_width(f32::INFINITY)
                                    .hint_text(tr("この行について書く")),
                            );
                            editor.request_focus();
                            if annotations.draft.chars().count() > DIFF_COMMENT_BODY_MAX_CHARS {
                                annotations.draft = annotations
                                    .draft
                                    .chars()
                                    .take(DIFF_COMMENT_BODY_MAX_CHARS)
                                    .collect();
                            }
                            note.horizontal(|ui| {
                                if ui.button(tr("削除")).clicked() {
                                    changed |= annotations.remove(project, &path, number);
                                }
                                if ui.button(tr("保存")).clicked() {
                                    changed |= annotations.save(
                                        project,
                                        &path,
                                        number,
                                        &comment_anchor(&source.text),
                                    );
                                }
                            });
                        } else if let Some((body, is_resolved)) = annotations
                            .find(project, &path, number)
                            .map(|c| (c.body.clone(), c.resolved))
                        {
                            note.horizontal(|ui| {
                                let body_text = if is_resolved {
                                    RichText::new(body)
                                        .size(12.5)
                                        .strikethrough()
                                        .color(palette.text_muted)
                                } else {
                                    RichText::new(body).size(12.5).color(palette.text)
                                };
                                ui.label(body_text);
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if ui.small_button(tr("削除")).clicked() {
                                            changed |= annotations.remove(project, &path, number);
                                        }
                                        let toggle_label = if is_resolved {
                                            tr("再開")
                                        } else {
                                            tr("解決")
                                        };
                                        if ui.small_button(toggle_label).clicked() {
                                            changed |=
                                                annotations.toggle_resolved(project, &path, number);
                                        }
                                    },
                                );
                            });
                        }
                    }
                }
            }
        });
    if let Some(path) = toggled {
        if !collapsed.remove(&path) {
            collapsed.insert(path);
        }
    }
    if let Some((path, number)) = opened {
        annotations.open(project, &path, number);
    }
    changed
}

/// Lay out one diff row: the two line numbers, the sign, and the line.
///
/// The numbers are padded to a fixed width rather than right-aligned by a
/// layout, because this is one galley drawn at one position and a monospaced
/// space is exactly as wide as a digit.
///
/// A row that is half of a rewrite carries its `highlights` as section
/// backgrounds rather than as a second painting pass, so a marked row still
/// costs exactly one galley and the pane keeps virtualising its rows.
pub(crate) fn diff_line_layout(line: &DiffLine, _cell_width: f32, palette: &Palette) -> LayoutJob {
    let mut job = LayoutJob::default();
    let font = FontId::monospace(MONOSPACE_SIZE);
    let mut append = |text: &str, color: Color32, background: Color32| {
        job.append(
            text,
            0.0,
            TextFormat {
                font_id: font.clone(),
                color,
                background,
                ..Default::default()
            },
        );
    };
    let plain = Color32::TRANSPARENT;
    match line.kind {
        DiffLineKind::Hunk | DiffLineKind::Note => {
            // Neither is a line of the file, so neither claims a line number.
            append(
                &" ".repeat(DIFF_GUTTER_COLUMNS * 2 + 2),
                palette.text_faint,
                plain,
            );
            append(
                &line.text,
                if line.kind == DiffLineKind::Hunk {
                    palette.text_muted
                } else {
                    palette.text_faint
                },
                plain,
            );
            return job;
        }
        _ => {}
    }
    let number = |value: Option<usize>| match value {
        Some(value) => format!("{value:>DIFF_GUTTER_COLUMNS$}"),
        None => " ".repeat(DIFF_GUTTER_COLUMNS),
    };
    append(&number(line.old_number), palette.text_muted, plain);
    append(&number(line.new_number), palette.text_muted, plain);
    let (sign, ink, mark) = match line.kind {
        DiffLineKind::Added => ("+", palette.diff_added, palette.diff_added_emphasis),
        DiffLineKind::Removed => ("-", palette.diff_removed, palette.diff_removed_emphasis),
        _ => (" ", palette.text_faint, plain),
    };
    append(&format!(" {sign}"), ink, plain);
    let mut at = 0;
    for span in &line.highlights {
        append(&line.text[at..span.start], palette.text, plain);
        append(&line.text[span.clone()], palette.text, mark);
        at = span.end;
    }
    append(&line.text[at..], palette.text, plain);
    job
}
