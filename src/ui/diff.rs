use crate::prelude::*;
use crate::*;
use std::hash::{Hash, Hasher};

#[derive(Default)]
struct DiffRowsState {
    rows_fp: Option<u64>,
    rows: Vec<DiffRow>,
    change_rows: Vec<usize>,
    offsets: Vec<f32>,
    total_height: f32,
    widest: usize,
    row_builds: usize,
}

#[derive(Default)]
struct DiffGalleysState {
    style_key: Option<(u64, SyntaxColours, u32, u32)>,
    galleys: HashMap<(usize, usize, i8), Arc<egui::Galley>>,
    galley_builds: usize,
    galley_hits: usize,
}

/// Cached diff pane row list, virtualized offsets, and bounded shaped row galleys
/// (change 322).
#[derive(Default)]
pub(crate) struct DiffRenderCache {
    rows: DiffRowsState,
    galleys: DiffGalleysState,
}

impl DiffRenderCache {
    #[cfg(test)]
    pub(crate) fn row_builds(&self) -> usize {
        self.rows.row_builds
    }

    #[cfg(test)]
    pub(crate) fn galley_builds(&self) -> usize {
        self.galleys.galley_builds
    }

    #[cfg(test)]
    pub(crate) fn galley_hits(&self) -> usize {
        self.galleys.galley_hits
    }

    #[cfg(test)]
    pub(crate) fn cached_galley_len(&self) -> usize {
        self.galleys.galleys.len()
    }
}

fn diff_files_fingerprint(
    files: &[DiffFile],
    collapsed: &HashSet<String>,
    project: Uuid,
    annotations: &DiffAnnotations,
    side_by_side: bool,
    editable: bool,
    cell_y_bits: u32,
) -> (u64, u64) {
    let mut fh = std::hash::DefaultHasher::new();
    files.len().hash(&mut fh);
    for file in files {
        file.path.hash(&mut fh);
        std::mem::discriminant(&file.change).hash(&mut fh);
        file.added.hash(&mut fh);
        file.removed.hash(&mut fh);
        file.widest.hash(&mut fh);
        file.lines.len().hash(&mut fh);
        for line in &file.lines {
            std::mem::discriminant(&line.kind).hash(&mut fh);
            line.old_number.hash(&mut fh);
            line.new_number.hash(&mut fh);
            line.text.hash(&mut fh);
            line.highlights.len().hash(&mut fh);
            for span in &line.highlights {
                span.start.hash(&mut fh);
                span.end.hash(&mut fh);
            }
        }
    }
    let files_fp = fh.finish();

    let mut rh = std::hash::DefaultHasher::new();
    files_fp.hash(&mut rh);
    project.hash(&mut rh);
    side_by_side.hash(&mut rh);
    editable.hash(&mut rh);
    cell_y_bits.hash(&mut rh);
    let mut collapsed_sorted: Vec<&str> = collapsed.iter().map(String::as_str).collect();
    collapsed_sorted.sort_unstable();
    collapsed_sorted.hash(&mut rh);
    annotations.editing.hash(&mut rh);
    annotations.comments.len().hash(&mut rh);
    for c in &annotations.comments {
        if c.project == project {
            c.file.hash(&mut rh);
            c.line.hash(&mut rh);
            c.body.hash(&mut rh);
            c.sent.hash(&mut rh);
            c.resolved.hash(&mut rh);
        }
    }
    annotations.note_heights.len().hash(&mut rh);
    if let Some((editing_file, editing_line)) = &annotations.editing {
        if let Some(h) = annotations
            .note_heights
            .get(editing_file.as_str())
            .and_then(|m| m.get(&(project, *editing_line)))
        {
            h.to_bits().hash(&mut rh);
        }
    }
    let mut note_entries: Vec<(&str, usize, u32)> = annotations
        .note_heights
        .iter()
        .flat_map(|(path, heights)| {
            heights
                .iter()
                .filter(|((p, _), _)| *p == project)
                .map(|((_, line), h)| (path.as_str(), *line, h.to_bits()))
        })
        .collect();
    note_entries.sort_unstable();
    note_entries.hash(&mut rh);
    (files_fp, rh.finish())
}

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
    /// One side-by-side row of a file: the line in the left column and the
    /// one in the right (change 307), as `side_by_side_rows` pairs them.
    Pair(usize, Option<usize>, Option<usize>),
}

/// One row of the side-by-side layout: the line drawn in the left column and
/// the one in the right (change 307). A context line, a hunk header, and a
/// note name the same line on both sides — the header and the note span the
/// whole row — and a side with nothing facing it is `None`, drawn as filler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SidePair {
    pub(crate) left: Option<usize>,
    pub(crate) right: Option<usize>,
}

/// Lay one file's lines out in two columns. A run of removals followed by a
/// run of additions is a rewrite, so the two runs are paired line by line and
/// the longer one's remainder faces filler. One pass, no lookups: the pane
/// does this every frame for the file rows it builds.
pub(crate) fn side_by_side_rows(lines: &[DiffLine]) -> Vec<SidePair> {
    let mut pairs = Vec::with_capacity(lines.len());
    let mut index = 0;
    while index < lines.len() {
        match lines[index].kind {
            DiffLineKind::Removed | DiffLineKind::Added => {
                let removed_start = index;
                while index < lines.len() && lines[index].kind == DiffLineKind::Removed {
                    index += 1;
                }
                let added_start = index;
                while index < lines.len() && lines[index].kind == DiffLineKind::Added {
                    index += 1;
                }
                let removed = added_start - removed_start;
                let added = index - added_start;
                for offset in 0..removed.max(added) {
                    pairs.push(SidePair {
                        left: (offset < removed).then_some(removed_start + offset),
                        right: (offset < added).then_some(added_start + offset),
                    });
                }
            }
            _ => {
                pairs.push(SidePair {
                    left: Some(index),
                    right: Some(index),
                });
                index += 1;
            }
        }
    }
    pairs
}

/// Whether a side-by-side row shows a change: either side added or removed.
pub(crate) fn pair_is_change(lines: &[DiffLine], pair: &SidePair) -> bool {
    [pair.left, pair.right].into_iter().flatten().any(|index| {
        matches!(
            lines[index].kind,
            DiffLineKind::Added | DiffLineKind::Removed
        )
    })
}

/// The rows where a block of changes begins: a changed row whose previous row
/// is not one. These are what 前の変更 / 次の変更 move between.
pub(crate) fn change_block_starts(changed: &[bool]) -> Vec<usize> {
    changed
        .iter()
        .enumerate()
        .filter(|&(index, &is)| is && (index == 0 || !changed[index - 1]))
        .map(|(index, _)| index)
        .collect()
}

/// The block 次の変更 (`forward`) or 前の変更 moves to from `current`, stopping
/// at both ends. Before any step, either direction goes to the first block.
pub(crate) fn step_change(count: usize, current: Option<usize>, forward: bool) -> Option<usize> {
    let last = count.checked_sub(1)?;
    Some(match current {
        None => 0,
        Some(at) if forward => (at + 1).min(last),
        Some(at) => at.saturating_sub(1).min(last),
    })
}

/// The layout the pane actually draws: two columns need room to be read.
pub(crate) fn effective_diff_layout(chosen: DiffLayout, width: f32) -> DiffLayout {
    if chosen == DiffLayout::SideBySide && width < DIFF_SIDE_BY_SIDE_MIN_WIDTH {
        DiffLayout::Unified
    } else {
        chosen
    }
}

pub(crate) fn diff_pane_rows(
    files: &[DiffFile],
    collapsed: &HashSet<String>,
    annotated: &dyn Fn(&str, usize) -> bool,
) -> Vec<DiffRow> {
    let capacity: usize = files
        .iter()
        .map(|file| {
            1 + if collapsed.contains(&file.path) {
                0
            } else {
                file.lines.len()
            }
        })
        .sum();
    let mut rows = Vec::with_capacity(capacity);
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

/// The pane's rows in the side-by-side layout: each file's bar, then its
/// lines paired into two columns, with a note under the row whose right-hand
/// line carries it.
pub(crate) fn diff_pane_side_by_side_rows(
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
        for pair in side_by_side_rows(&file.lines) {
            rows.push(DiffRow::Pair(index, pair.left, pair.right));
            if let Some(right) = pair.right {
                if file.lines[right]
                    .new_number
                    .is_some_and(|number| annotated(&file.path, number))
                {
                    rows.push(DiffRow::Comment(index, right));
                }
            }
        }
    }
    rows
}

/// Whether a pane row shows a change, for 前の変更 / 次の変更.
pub(crate) fn diff_row_is_change(files: &[DiffFile], row: &DiffRow) -> bool {
    match *row {
        DiffRow::Line(file, line) => matches!(
            files[file].lines[line].kind,
            DiffLineKind::Added | DiffLineKind::Removed
        ),
        DiffRow::Pair(file, left, right) => {
            pair_is_change(&files[file].lines, &SidePair { left, right })
        }
        DiffRow::File(_) | DiffRow::Comment(..) => false,
    }
}

/// Where every row starts, and where the last one ends.
///
/// The pane used to divide by one row height, because every row was one line
/// tall. A note is taller, so the offsets are accumulated instead and the
/// visible range is found by searching them. Returned as `offsets.len() ==
/// rows.len() + 1`, so the height of row `i` is `offsets[i + 1] - offsets[i]`
/// and the total is the last entry — one shape, no special case for the end.
///
/// A note is as tall as it drew last frame (`note_height`), and
/// `DIFF_COMMENT_ROW_LINES` lines before it has been drawn once.
pub(crate) fn diff_row_offsets(
    rows: &[DiffRow],
    line_height: f32,
    note_height: &dyn Fn(usize, usize) -> Option<f32>,
) -> Vec<f32> {
    let mut offsets = Vec::with_capacity(rows.len() + 1);
    let mut cursor = 0.0;
    for row in rows {
        offsets.push(cursor);
        cursor += match *row {
            DiffRow::Comment(file, line) => {
                note_height(file, line).unwrap_or(line_height * DIFF_COMMENT_ROW_LINES)
            }
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

/// User action triggered on a single hunk header row inside the diff pane.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DiffHunkAction {
    Stage {
        file_index: usize,
        hunk_line_index: usize,
    },
    Discard {
        file_index: usize,
        hunk_line_index: usize,
    },
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
#[cfg(test)]
pub(crate) fn diff_pane(
    ui: &mut egui::Ui,
    files: &[DiffFile],
    collapsed: &mut HashSet<String>,
    palette: &Palette,
    height: f32,
    project: Uuid,
    annotations: &mut DiffAnnotations,
) -> bool {
    let mut hunk_action = None;
    diff_pane_with_actions(
        ui,
        files,
        collapsed,
        palette,
        height,
        project,
        annotations,
        &mut hunk_action,
    )
}

/// How the pane shows a diff (change 307).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct DiffPaneView<'a> {
    /// What the person chose; a pane too narrow draws `Unified` anyway.
    pub(crate) layout: DiffLayout,
    /// The two columns' names in the side-by-side layout: the older side, then
    /// the newer.
    pub(crate) labels: (&'a str, &'a str),
    /// Whether hunk actions and notes are offered. Only HEAD ↔ local is a diff
    /// they can act on.
    pub(crate) editable: bool,
    /// The change block to scroll to on this frame.
    pub(crate) scroll_to_change: Option<usize>,
}

impl DiffPaneView<'static> {
    /// The pane as it always was: one column, actions and notes on.
    pub(crate) fn working_tree() -> Self {
        Self {
            layout: DiffLayout::Unified,
            labels: ("", ""),
            editable: true,
            scroll_to_change: None,
        }
    }
}

/// What a frame of the pane came to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct DiffPaneOutcome {
    /// The set of notes changed and needs writing down.
    pub(crate) notes_changed: bool,
    /// How many change blocks the drawn rows hold.
    pub(crate) changes: usize,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn diff_pane_with_actions(
    ui: &mut egui::Ui,
    files: &[DiffFile],
    collapsed: &mut HashSet<String>,
    palette: &Palette,
    height: f32,
    project: Uuid,
    annotations: &mut DiffAnnotations,
    hunk_action: &mut Option<DiffHunkAction>,
) -> bool {
    diff_pane_with_view(
        ui,
        files,
        collapsed,
        palette,
        height,
        project,
        annotations,
        hunk_action,
        &DiffPaneView::working_tree(),
    )
    .notes_changed
}

#[cfg(test)]
#[allow(clippy::too_many_arguments)]
pub(crate) fn diff_pane_with_cache(
    ui: &mut egui::Ui,
    files: &[DiffFile],
    collapsed: &mut HashSet<String>,
    palette: &Palette,
    height: f32,
    project: Uuid,
    annotations: &mut DiffAnnotations,
    hunk_action: &mut Option<DiffHunkAction>,
    editable: bool,
    cache: Option<&mut DiffRenderCache>,
) -> DiffPaneOutcome {
    diff_pane_with_view_cached(
        ui,
        files,
        collapsed,
        palette,
        height,
        project,
        annotations,
        hunk_action,
        &DiffPaneView {
            editable,
            ..DiffPaneView::working_tree()
        },
        cache,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn diff_pane_with_view(
    ui: &mut egui::Ui,
    files: &[DiffFile],
    collapsed: &mut HashSet<String>,
    palette: &Palette,
    height: f32,
    project: Uuid,
    annotations: &mut DiffAnnotations,
    hunk_action: &mut Option<DiffHunkAction>,
    view: &DiffPaneView,
) -> DiffPaneOutcome {
    if files.is_empty() {
        ui.label(RichText::new(tr("変更はありません。")).color(palette.text_muted));
        return DiffPaneOutcome::default();
    }
    let cache_id = ui.scope_id().with(("diff-render-cache", project));
    let cache_arc: Arc<std::sync::Mutex<DiffRenderCache>> = ui.ctx().data_mut(|data| {
        data.get_temp_mut_or_insert_with(cache_id, || {
            Arc::new(std::sync::Mutex::new(DiffRenderCache::default()))
        })
        .clone()
    });
    let outcome = if let Ok(mut guard) = cache_arc.lock() {
        diff_pane_with_view_cached(
            ui,
            files,
            collapsed,
            palette,
            height,
            project,
            annotations,
            hunk_action,
            view,
            Some(&mut guard),
        )
    } else {
        diff_pane_with_view_cached(
            ui,
            files,
            collapsed,
            palette,
            height,
            project,
            annotations,
            hunk_action,
            view,
            None,
        )
    };
    outcome
}

#[allow(clippy::too_many_arguments)]
fn diff_pane_with_view_cached(
    ui: &mut egui::Ui,
    files: &[DiffFile],
    collapsed: &mut HashSet<String>,
    palette: &Palette,
    height: f32,
    project: Uuid,
    annotations: &mut DiffAnnotations,
    hunk_action: &mut Option<DiffHunkAction>,
    view: &DiffPaneView,
    cache: Option<&mut DiffRenderCache>,
) -> DiffPaneOutcome {
    if files.is_empty() {
        ui.label(RichText::new(tr("変更はありません。")).color(palette.text_muted));
        return DiffPaneOutcome::default();
    }
    let cell = monospace_cell_size(ui);
    let side_by_side =
        effective_diff_layout(view.layout, ui.available_width()) == DiffLayout::SideBySide;
    let editable = view.editable;
    let (files_fp, rows_fp) = diff_files_fingerprint(
        files,
        collapsed,
        project,
        annotations,
        side_by_side,
        editable,
        cell.y.to_bits(),
    );

    let local_rows;
    let local_change_rows;
    let local_offsets;
    let style_key = (
        files_fp,
        SyntaxColours::from(palette),
        ui.ctx().pixels_per_point().to_bits(),
        cell.x.to_bits(),
    );
    let (rows_cache, galley_cache) = match cache {
        Some(c) => (Some(&mut c.rows), Some(&mut c.galleys)),
        None => (None, None),
    };
    let (rows, change_rows, offsets, total_height, widest): (
        &[DiffRow],
        &[usize],
        &[f32],
        f32,
        usize,
    ) = if let Some(rc) = rows_cache {
        if rc.rows_fp != Some(rows_fp) {
            let annotated =
                |file: &str, line: usize| editable && annotations.shows_row(project, file, line);
            rc.rows = if side_by_side {
                diff_pane_side_by_side_rows(files, collapsed, &annotated)
            } else {
                diff_pane_rows(files, collapsed, &annotated)
            };
            let changed_rows: Vec<bool> = rc
                .rows
                .iter()
                .map(|row| diff_row_is_change(files, row))
                .collect();
            rc.change_rows = change_block_starts(&changed_rows);
            rc.offsets = diff_row_offsets(&rc.rows, cell.y, &|file, line| {
                let number = files[file].lines[line].new_number?;
                annotations
                    .note_heights
                    .get(files[file].path.as_str())?
                    .get(&(project, number))
                    .copied()
            });
            rc.total_height = rc.offsets.last().copied().unwrap_or(0.0);
            rc.widest = files.iter().map(|file| file.widest).max().unwrap_or(0);
            rc.rows_fp = Some(rows_fp);
            rc.row_builds += 1;
        }
        (
            &rc.rows,
            &rc.change_rows,
            &rc.offsets,
            rc.total_height,
            rc.widest,
        )
    } else {
        let annotated =
            |file: &str, line: usize| editable && annotations.shows_row(project, file, line);
        local_rows = if side_by_side {
            diff_pane_side_by_side_rows(files, collapsed, &annotated)
        } else {
            diff_pane_rows(files, collapsed, &annotated)
        };
        let changed_rows: Vec<bool> = local_rows
            .iter()
            .map(|row| diff_row_is_change(files, row))
            .collect();
        local_change_rows = change_block_starts(&changed_rows);
        local_offsets = diff_row_offsets(&local_rows, cell.y, &|file, line| {
            let number = files[file].lines[line].new_number?;
            annotations
                .note_heights
                .get(files[file].path.as_str())?
                .get(&(project, number))
                .copied()
        });
        let total_h = local_offsets.last().copied().unwrap_or(0.0);
        let max_w = files.iter().map(|file| file.widest).max().unwrap_or(0);
        (
            &local_rows,
            &local_change_rows,
            &local_offsets,
            total_h,
            max_w,
        )
    };

    let galley_cache = galley_cache.map(|gc| {
        if gc.style_key != Some(style_key) {
            gc.galleys.clear();
            gc.style_key = Some(style_key);
        }
        (&mut gc.galleys, &mut gc.galley_builds, &mut gc.galley_hits)
    });
    // Two gutters, the sign, and a space after it, then the widest line in the
    // diff. The `max` keeps a diff of short lines from shrinking its own rows
    // to less than the pane it is drawn in.
    //
    // Side by side, the two columns share the pane's width and scroll sideways
    // together by one offset, as an IDE's do: one horizontal scroll bar over
    // two columns would carry the right column out of sight.
    let mut height = height;
    let mut column_shift = 0.0;
    let half = (ui.available_width() / 2.0).floor();
    let content_width = if side_by_side {
        let label_strip = cell.y + 6.0;
        let (strip, _) =
            ui.allocate_exact_size(egui::vec2(half * 2.0, label_strip), egui::Sense::hover());
        ui.painter().rect_filled(strip, 0.0, palette.raised);
        for (x, label) in [
            (strip.left(), view.labels.0),
            (strip.left() + half, view.labels.1),
        ] {
            ui.painter().text(
                egui::pos2(x + 8.0, strip.center().y),
                egui::Align2::LEFT_CENTER,
                label,
                egui::FontId::proportional(12.0),
                palette.text_muted,
            );
        }
        ui.painter().vline(
            strip.left() + half,
            strip.y_range(),
            egui::Stroke::new(1.0_f32, palette.border_subtle),
        );
        height = (height - label_strip).max(cell.y);
        let needed = (DIFF_GUTTER_COLUMNS + 2 + widest) as f32 * cell.x + 16.0;
        let id = ui.scope_id().with("diff-side-by-side-shift");
        let pane = egui::Rect::from_min_size(ui.cursor().min, egui::vec2(half * 2.0, height));
        let delta = if ui.rect_contains_pointer(pane) {
            ui.input(|input| input.smooth_scroll_delta.x)
        } else {
            0.0
        };
        let stored: f32 = ui.data(|data| data.get_temp(id)).unwrap_or(0.0);
        column_shift = (stored - delta).clamp(0.0, (needed - half).max(0.0));
        if column_shift != stored {
            ui.data_mut(|data| data.insert_temp(id, column_shift));
        }
        half * 2.0
    } else {
        ((DIFF_GUTTER_COLUMNS * 2 + 2 + widest) as f32 * cell.x + 16.0).max(ui.available_width())
    };
    let mut toggled = None;
    let mut opened = None;
    let mut changed = false;

    ui.spacing_mut().item_spacing.y = 0.0;
    let area = if side_by_side {
        egui::ScrollArea::vertical()
    } else {
        egui::ScrollArea::both()
    };
    // A step to a change puts its first row a few lines below the top, so the
    // context above it stays in view.
    let area = match view
        .scroll_to_change
        .and_then(|block| change_rows.get(block))
    {
        Some(&row) => area.vertical_scroll_offset((offsets[row] - cell.y * 3.0).max(0.0)),
        None => area,
    };
    area.auto_shrink([false, false])
        .max_height(height)
        .show_viewport(ui, |ui, viewport| {
            // The rows are placed at absolute offsets rather than by advancing
            // a cursor, so the scroll extent has to be claimed up front or the
            // area would believe it holds one row.
            ui.set_min_size(egui::vec2(content_width, total_height));
            let origin = ui.min_rect().min;
            let visible = diff_visible_rows(offsets, viewport.min.y, viewport.max.y);
            let mut local_galleys: HashMap<(usize, usize, i8), Arc<egui::Galley>> = HashMap::new();
            let (galleys_map, builds_ctr, hits_ctr) = match galley_cache {
                Some((m, b, h)) => (m, Some(b), Some(h)),
                None => (&mut local_galleys, None, None),
            };
            let mut jobs: Vec<((usize, usize, i8), LayoutJob)> = Vec::new();
            for index in visible.clone() {
                match rows[index] {
                    DiffRow::Line(file_idx, line_idx) => {
                        let key = (file_idx, line_idx, 0i8);
                        if !galleys_map.contains_key(&key) {
                            let line = &files[file_idx].lines[line_idx];
                            jobs.push((key, diff_line_layout(line, cell.x, palette)));
                        }
                    }
                    DiffRow::Pair(file_idx, left, right) => {
                        let file = &files[file_idx];
                        if let (Some(at), true) = (
                            left,
                            left == right
                                && matches!(
                                    left.map(|at| file.lines[at].kind),
                                    Some(DiffLineKind::Hunk | DiffLineKind::Note)
                                ),
                        ) {
                            let key = (file_idx, at, -1i8);
                            if !galleys_map.contains_key(&key) {
                                jobs.push((key, diff_half_layout(&file.lines[at], false, palette)));
                            }
                        } else {
                            if let Some(at) = left {
                                let key = (file_idx, at, 1i8);
                                if !galleys_map.contains_key(&key) {
                                    jobs.push((
                                        key,
                                        diff_half_layout(&file.lines[at], false, palette),
                                    ));
                                }
                            }
                            if let Some(at) = right {
                                let key = (file_idx, at, 2i8);
                                if !galleys_map.contains_key(&key) {
                                    jobs.push((
                                        key,
                                        diff_half_layout(&file.lines[at], true, palette),
                                    ));
                                }
                            }
                        }
                    }
                    DiffRow::File(_) | DiffRow::Comment(_, _) => {}
                }
            }
            let mut overflow_galleys: HashMap<(usize, usize, i8), Arc<egui::Galley>> =
                HashMap::new();
            if !jobs.is_empty() {
                let shaped: Vec<((usize, usize, i8), Arc<egui::Galley>)> = ui.fonts_mut(|fonts| {
                    jobs.into_iter()
                        .map(|(k, job)| (k, fonts.layout_job(job)))
                        .collect()
                });
                if let Some(b) = builds_ctr {
                    *b += shaped.len();
                }
                if galleys_map.len() + shaped.len() > MAX_CACHED_DIFF_ROW_GALLEYS {
                    galleys_map.clear();
                }
                for (k, g) in shaped {
                    if galleys_map.len() < MAX_CACHED_DIFF_ROW_GALLEYS {
                        galleys_map.insert(k, g);
                    } else {
                        overflow_galleys.insert(k, g);
                    }
                }
            } else if let Some(h) = hits_ctr {
                *h += visible.len();
            }

            let lookup_galley = |key: (usize, usize, i8)| {
                galleys_map
                    .get(&key)
                    .or_else(|| overflow_galleys.get(&key))
                    .cloned()
            };

            for index in visible {
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
                            ui.scope_id().with(("diff-file-row", &file.path)),
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
                            egui::Stroke::new(1.0_f32, palette.border_subtle),
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
                            .size(13.5)
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
                    DiffRow::Line(file_idx, line_idx) => {
                        let path = &files[file_idx].path;
                        let line = &files[file_idx].lines[line_idx];
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
                                egui::Stroke::new(1.0_f32, palette.border_subtle),
                            );
                        }
                        if let Some(galley) = lookup_galley((file_idx, line_idx, 0i8)) {
                            ui.painter().galley(
                                rect.min + egui::vec2(8.0, 0.0),
                                galley,
                                palette.text,
                            );
                        }
                        if editable && line.kind == DiffLineKind::Hunk && !path.is_empty() {
                            diff_hunk_buttons(
                                ui,
                                rect,
                                palette,
                                path,
                                file_idx,
                                line_idx,
                                hunk_action,
                            );
                        }
                        // Only a line that exists after the change can carry a
                        // note: a removed line has no line number on the side
                        // the agent would go looking at.
                        if let Some(number) = line.new_number.filter(|_| editable) {
                            let response = ui.interact(
                                rect,
                                ui.scope_id().with(("diff-line", path, number)),
                                egui::Sense::click(),
                            );
                            if response.hovered()
                                || response.has_focus()
                                || annotations.shows_row(project, path, number)
                            {
                                ui.painter().text(
                                    rect.right_center() - egui::vec2(8.0, 0.0),
                                    egui::Align2::RIGHT_CENTER,
                                    ICON_COMMENT,
                                    egui::FontId::proportional(12.0),
                                    if annotations.find(project, path, number).is_some() {
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
                                opened = Some((path.clone(), number));
                            }
                        }
                    }
                    DiffRow::Pair(file_idx, left, right) => {
                        let file = &files[file_idx];
                        // A hunk header or a note spans both columns.
                        if let (Some(at), true) = (
                            left,
                            left == right
                                && matches!(
                                    left.map(|at| file.lines[at].kind),
                                    Some(DiffLineKind::Hunk | DiffLineKind::Note)
                                ),
                        ) {
                            let line = &file.lines[at];
                            if line.kind == DiffLineKind::Hunk {
                                ui.painter().hline(
                                    rect.x_range(),
                                    rect.top(),
                                    egui::Stroke::new(1.0_f32, palette.border_subtle),
                                );
                            }
                            if let Some(galley) = lookup_galley((file_idx, at, -1i8)) {
                                ui.painter().galley(
                                    rect.min + egui::vec2(8.0, 0.0),
                                    galley,
                                    palette.text,
                                );
                            }
                            if editable && line.kind == DiffLineKind::Hunk {
                                diff_hunk_buttons(
                                    ui,
                                    rect,
                                    palette,
                                    &file.path,
                                    file_idx,
                                    at,
                                    hunk_action,
                                );
                            }
                            continue;
                        }
                        let halves = [
                            (
                                egui::Rect::from_min_max(
                                    rect.min,
                                    egui::pos2(rect.left() + half, rect.bottom()),
                                ),
                                left,
                                1i8,
                            ),
                            (
                                egui::Rect::from_min_max(
                                    egui::pos2(rect.left() + half, rect.top()),
                                    rect.max,
                                ),
                                right,
                                2i8,
                            ),
                        ];
                        for (side, at, side_tag) in halves {
                            let Some(at) = at else {
                                ui.painter().rect_filled(side, 0.0, palette.raised);
                                continue;
                            };
                            let line = &file.lines[at];
                            match line.kind {
                                DiffLineKind::Added => {
                                    ui.painter().rect_filled(side, 0.0, palette.diff_added_bg);
                                }
                                DiffLineKind::Removed => {
                                    ui.painter().rect_filled(side, 0.0, palette.diff_removed_bg);
                                }
                                _ => {}
                            }
                            if let Some(galley) = lookup_galley((file_idx, at, side_tag)) {
                                ui.painter()
                                    .with_clip_rect(side.intersect(ui.clip_rect()))
                                    .galley(
                                        side.min + egui::vec2(8.0 - column_shift, 0.0),
                                        galley,
                                        palette.text,
                                    );
                            }
                        }
                        ui.painter().vline(
                            rect.left() + half,
                            rect.y_range(),
                            egui::Stroke::new(1.0_f32, palette.border_subtle),
                        );
                        // A note belongs to the line as it is now: the right
                        // column's.
                        let number = right
                            .and_then(|at| file.lines[at].new_number)
                            .filter(|_| editable);
                        if let Some(number) = number {
                            let side = halves[1].0;
                            let response = ui.interact(
                                side,
                                ui.scope_id().with(("diff-line", &file.path, number)),
                                egui::Sense::click(),
                            );
                            if response.hovered()
                                || response.has_focus()
                                || annotations.shows_row(project, &file.path, number)
                            {
                                ui.painter().text(
                                    side.right_center() - egui::vec2(8.0, 0.0),
                                    egui::Align2::RIGHT_CENTER,
                                    ICON_COMMENT,
                                    egui::FontId::proportional(12.0),
                                    if annotations.find(project, &file.path, number).is_some() {
                                        palette.accent_soft
                                    } else {
                                        palette.text_muted
                                    },
                                );
                            }
                            if let Some(stroke) = focus_stroke(response.has_focus(), palette) {
                                ui.painter().rect_stroke(
                                    side,
                                    0.0,
                                    stroke,
                                    egui::StrokeKind::Inside,
                                );
                            }
                            if response.clicked() {
                                opened = Some((file.path.clone(), number));
                            }
                        }
                    }
                    DiffRow::Comment(file, line) => {
                        let path = &files[file].path;
                        let source = &files[file].lines[line];
                        let Some(number) = source.new_number else {
                            continue;
                        };
                        let stale = annotations
                            .find(project, path, number)
                            .is_some_and(|comment| comment.anchor != comment_anchor(&source.text));
                        let editing = annotations
                            .editing
                            .as_ref()
                            .is_some_and(|(open, at)| open == path && *at == number);
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
                        // Until the row has its measured height, what does not
                        // fit is cut rather than painted over the next line.
                        note.set_clip_rect(rect.intersect(ui.clip_rect()));
                        let (comment_sent, comment_resolved) = annotations
                            .find(project, path, number)
                            .map(|c| (c.sent, c.resolved))
                            .unwrap_or((false, false));
                        let reading = if editing {
                            None
                        } else {
                            annotations
                                .find(project, path, number)
                                .map(|c| (c.body.clone(), c.resolved))
                        };
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
                                        .size(11.5)
                                        .color(palette.accent),
                                );
                            }
                            if comment_resolved {
                                ui.label(
                                    RichText::new(format!("[{}]", tr("解決済み")))
                                        .size(11.5)
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
                            // The note's buttons sit on its header line, so the
                            // body below can take the whole width and wrap.
                            if let Some((_, is_resolved)) = &reading {
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if secondary_button(ui, tr("削除")).clicked() {
                                            changed |= annotations.remove(project, path, number);
                                        }
                                        let toggle_label = if *is_resolved {
                                            tr("再開")
                                        } else {
                                            tr("解決")
                                        };
                                        if secondary_button(ui, toggle_label).clicked() {
                                            changed |=
                                                annotations.toggle_resolved(project, path, number);
                                        }
                                    },
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
                            if std::mem::take(&mut annotations.focus_pending) {
                                editor.request_focus();
                            }
                            if annotations.draft.chars().count() > DIFF_COMMENT_BODY_MAX_CHARS {
                                annotations.draft = annotations
                                    .draft
                                    .chars()
                                    .take(DIFF_COMMENT_BODY_MAX_CHARS)
                                    .collect();
                            }
                            note.horizontal(|ui| {
                                if ui.button(tr("削除")).clicked() {
                                    changed |= annotations.remove(project, path, number);
                                }
                                if ui.button(tr("保存")).clicked() {
                                    changed |= annotations.save(
                                        project,
                                        path,
                                        number,
                                        &comment_anchor(&source.text),
                                    );
                                }
                            });
                        } else if let Some((body, is_resolved)) = reading {
                            let body_text = if is_resolved {
                                RichText::new(body)
                                    .size(12.5)
                                    .strikethrough()
                                    .color(palette.text_muted)
                            } else {
                                RichText::new(body).size(12.5).color(palette.text)
                            };
                            note.label(body_text);
                        }
                        // The row is as tall as the note drew, margins included;
                        // a change is laid out again on the next frame.
                        let drawn = note.min_rect().height() + 12.0;
                        if annotations
                            .note_heights
                            .get(path.as_str())
                            .and_then(|heights| heights.get(&(project, number)))
                            .is_none_or(|height| (height - drawn).abs() > 0.5)
                        {
                            annotations
                                .note_heights
                                .entry(path.clone())
                                .or_default()
                                .insert((project, number), drawn);
                            ui.ctx().request_repaint();
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
    DiffPaneOutcome {
        notes_changed: changed,
        changes: change_rows.len(),
    }
}

/// The stage and discard buttons at the right end of a hunk header row.
fn diff_hunk_buttons(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    palette: &Palette,
    path: &str,
    file_index: usize,
    hunk_line_index: usize,
    hunk_action: &mut Option<DiffHunkAction>,
) {
    let mut hunk_ui = ui.new_child(
        egui::UiBuilder::new()
            .id_salt(("diff-hunk-actions", path, hunk_line_index))
            .max_rect(rect.shrink2(egui::vec2(6.0, 1.0)))
            .layout(egui::Layout::right_to_left(egui::Align::Center)),
    );
    hunk_ui.spacing_mut().item_spacing.x = 6.0;
    if hunk_ui
        .add(
            egui::Button::new(RichText::new(tr("破棄")).size(11.5).color(palette.danger))
                .fill(palette.raised)
                .stroke(egui::Stroke::new(1.0_f32, palette.border_subtle)),
        )
        .on_hover_text(tr("このハンクの変更を破棄します"))
        .clicked()
    {
        *hunk_action = Some(DiffHunkAction::Discard {
            file_index,
            hunk_line_index,
        });
    }
    if hunk_ui
        .add(
            egui::Button::new(
                RichText::new(tr("ステージ"))
                    .size(11.5)
                    .color(palette.accent),
            )
            .fill(palette.raised)
            .stroke(egui::Stroke::new(1.0_f32, palette.border_subtle)),
        )
        .on_hover_text(tr("このハンクをステージします"))
        .clicked()
    {
        *hunk_action = Some(DiffHunkAction::Stage {
            file_index,
            hunk_line_index,
        });
    }
}

/// Lay out one side of a side-by-side row: that side's line number and the
/// line, its rewritten words marked. A hunk header or a note keeps its text
/// after a blank gutter, as in the interleaved layout.
pub(crate) fn diff_half_layout(line: &DiffLine, newer: bool, palette: &Palette) -> LayoutJob {
    let mut job = LayoutJob::default();
    let font = FontId::monospace(MONOSPACE_SIZE);
    let plain = Color32::TRANSPARENT;
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
    match line.kind {
        DiffLineKind::Hunk | DiffLineKind::Note => {
            append(&" ".repeat(DIFF_GUTTER_COLUMNS + 2), plain, plain);
            let ink = if line.kind == DiffLineKind::Hunk {
                palette.text_muted
            } else {
                palette.text_faint
            };
            append(&line.text, ink, plain);
            return job;
        }
        _ => {}
    }
    let number = if newer {
        line.new_number
    } else {
        line.old_number
    };
    match number {
        Some(value) => append(
            &format!("{value:>DIFF_GUTTER_COLUMNS$}  "),
            palette.text_muted,
            plain,
        ),
        None => append(&" ".repeat(DIFF_GUTTER_COLUMNS + 2), plain, plain),
    }
    let mark = match line.kind {
        DiffLineKind::Added => palette.diff_added_emphasis,
        DiffLineKind::Removed => palette.diff_removed_emphasis,
        _ => plain,
    };
    let mut at = 0;
    for span in &line.highlights {
        append(&line.text[at..span.start], palette.text, plain);
        append(&line.text[span.clone()], palette.text, mark);
        at = span.end;
    }
    append(&line.text[at..], palette.text, plain);
    job
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
