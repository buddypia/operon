//! The editor's comparisons (change 307): the 差分 view's compare bar —
//! which revision, which layout, which change — and the 注釈 view, each line
//! beside the commit that last changed it.

use crate::*;

/// What the 差分 view compares for the document in front, and where it stands
/// among the changes. Belongs to one document: another one starts again from
/// HEAD ↔ local, as an IDE's diff of another file does.
#[derive(Debug, Clone, Default)]
pub(crate) struct EditorCompare {
    pub(crate) document: Option<DocumentId>,
    pub(crate) base: DiffBase,
    /// The two column names, worked out once when the base is chosen rather
    /// than on every frame.
    pub(crate) labels: (String, String),
    /// The change block the person last stepped to.
    pub(crate) cursor: Option<usize>,
    /// The block to scroll to on the next frame.
    pub(crate) scroll_to: Option<usize>,
    /// How many change blocks the last frame drew.
    pub(crate) changes: usize,
}

/// The column names for a comparison: the older side, then the newer.
pub(crate) fn compare_labels(base: &DiffBase) -> (String, String) {
    match base {
        DiffBase::Head => ("HEAD".to_string(), tr("ローカル").to_string()),
        DiffBase::Revision(commit) => {
            (short_commit(commit).to_string(), tr("ローカル").to_string())
        }
        DiffBase::Commit(commit) => {
            let short = short_commit(commit);
            (tf!("{short} の親", short = short), short.to_string())
        }
    }
}

impl OperonApp {
    /// The comparison state for `document`, started afresh when the document
    /// in front is not the one it was kept for.
    pub(crate) fn compare_for(&mut self, document: &DocumentId) -> &mut EditorCompare {
        if self.editor_compare.document.as_ref() != Some(document) {
            self.editor_compare = EditorCompare {
                document: Some(document.clone()),
                labels: compare_labels(&DiffBase::Head),
                ..EditorCompare::default()
            };
        }
        &mut self.editor_compare
    }

    /// Compare `document` against `base` from now on, read afresh.
    pub(crate) fn set_compare_base(
        &mut self,
        project: &Project,
        document: &DocumentId,
        base: DiffBase,
    ) {
        let file = document.path.to_string_lossy().into_owned();
        self.revision_diff_cache
            .remove(&(project.id, file.clone(), base.clone()));
        if base.is_head() {
            self.invalidate_open_editor_diff(project.id);
        }
        let compare = self.compare_for(document);
        compare.labels = compare_labels(&base);
        compare.base = base;
        compare.cursor = None;
        compare.scroll_to = None;
        compare.changes = 0;
        self.editor_view = EditorView::Diff;
    }

    /// Drop what was read about one file — its blame, its history, its
    /// comparisons with past revisions — so the views read it again.
    pub(crate) fn forget_compare_data(&mut self, project_id: Uuid, file: &str) {
        let key = (project_id, file.to_string());
        self.blame_cache.remove(&key);
        self.file_history_cache.remove(&key);
        self.revision_diff_cache
            .retain(|(project, path, _), _| *project != project_id || path != file);
    }

    pub(crate) fn request_blame(&mut self, project: &Project, file: String) {
        let project_id = project.id;
        let path = project.path.clone();
        self.spawn_background(BackgroundKey::Blame(project_id, file.clone()), move || {
            let result = git_blame(&path, &file).map_err(|error| error.to_string());
            BackgroundResult::Blame {
                project_id,
                file,
                result,
            }
        });
    }

    pub(crate) fn request_file_history(&mut self, project: &Project, file: String) {
        let project_id = project.id;
        let path = project.path.clone();
        self.spawn_background(
            BackgroundKey::FileHistory(project_id, file.clone()),
            move || {
                let result = git_file_history(&path, &file).map_err(|error| error.to_string());
                BackgroundResult::FileHistory {
                    project_id,
                    file,
                    result,
                }
            },
        );
    }

    /// Read and parse a comparison with a past revision off the frame.
    pub(crate) fn request_revision_diff(
        &mut self,
        project: &Project,
        file: String,
        base: DiffBase,
    ) {
        let project_id = project.id;
        let path = project.path.clone();
        self.spawn_background(
            BackgroundKey::RevisionDiff(project_id, file.clone(), base.clone()),
            move || {
                let result = git_revision_diff(&path, &file, &base)
                    .map(|diff| parse_unified_diff(&diff))
                    .map_err(|error| error.to_string());
                BackgroundResult::RevisionDiff {
                    project_id,
                    file,
                    base,
                    result,
                }
            },
        );
    }

    /// The 差分 view: the compare bar, then the comparison it names.
    pub(crate) fn ui_editor_diff(
        &mut self,
        ui: &mut egui::Ui,
        project: &Project,
        document: &DocumentId,
        palette: &'static Palette,
        remaining: f32,
    ) {
        let file = document.path.to_string_lossy().into_owned();
        let base = self.compare_for(document).base.clone();
        let mut step = self.ui_editor_compare_bar(ui, project, document, &file, &base, palette);
        // F7 / ⇧F7, an IDE's keys for the next and previous change, while no
        // text field is taking the keyboard.
        if step.is_none() && !ui.ctx().egui_wants_keyboard_input() {
            step = ui.input_mut(|input| {
                if input.consume_key(egui::Modifiers::SHIFT, egui::Key::F7) {
                    Some(false)
                } else if input.consume_key(egui::Modifiers::NONE, egui::Key::F7) {
                    Some(true)
                } else {
                    None
                }
            });
        }
        {
            let compare = self.compare_for(document);
            if let Some(forward) = step {
                compare.cursor = step_change(compare.changes, compare.cursor, forward);
                compare.scroll_to = compare.cursor;
            } else if compare.cursor.is_none() && compare.changes > 0 {
                // Opening a comparison lands on its first change.
                compare.cursor = Some(0);
                compare.scroll_to = Some(0);
            }
        }
        ui.add_space(SPACE_XS);
        let footer = if base.is_head() && self.diff_annotations.count(project.id) > 0 {
            34.0
        } else {
            0.0
        };
        let height = (remaining - 34.0 - footer).max(120.0);
        // Read first, while `self` is free to start a read; the pane below
        // then borrows only the fields it draws from.
        let head_key = (project.id, Some(file.clone()));
        let revision_key = (project.id, file.clone(), base.clone());
        if base.is_head() {
            self.ensure_diff_parsed(project, Some(file.clone()));
        } else if !self.revision_diff_cache.contains_key(&revision_key)
            && !self.background_tasks.contains(&BackgroundKey::RevisionDiff(
                project.id,
                file.clone(),
                base.clone(),
            ))
        {
            self.request_revision_diff(project, file.clone(), base.clone());
        }
        let read = if base.is_head() {
            match self.git_diff_cache.get(&head_key) {
                Some(Err(error)) => Some(Err(error)),
                _ => self.diff_file_cache.get(&head_key).map(Ok),
            }
        } else {
            self.revision_diff_cache
                .get(&revision_key)
                .map(|result| result.as_ref())
        };
        let files = match read {
            None => {
                ui.label(
                    RichText::new(tr("差分を読み込んでいます…"))
                        .size(12.0)
                        .color(palette.text_muted),
                );
                return;
            }
            Some(Err(error)) => {
                ui.label(tf!("差分を取得できません: {error}", error = error));
                return;
            }
            Some(Ok(files)) => files,
        };
        if files.is_empty() {
            ui.add_space(12.0);
            let empty = match base {
                DiffBase::Head => tr("このファイルは、最後のコミットから変更されていません。"),
                DiffBase::Revision(_) => tr("このリビジョンとローカルに違いはありません。"),
                DiffBase::Commit(_) => tr("このコミットはこのファイルを変更していません。"),
            };
            ui.label(RichText::new(empty).color(palette.text_muted));
            return;
        }
        let compare = &self.editor_compare;
        let view = DiffPaneView {
            layout: self.store.diff_layout,
            labels: (&compare.labels.0, &compare.labels.1),
            // Hunk actions and notes act on the working copy against HEAD;
            // against any other revision they would patch the wrong text.
            editable: base.is_head(),
            scroll_to_change: compare.scroll_to,
        };
        let mut hunk_action = None;
        let outcome = diff_pane_with_view(
            ui,
            files,
            &mut self.collapsed_diff_files,
            palette,
            height,
            project.id,
            &mut self.diff_annotations,
            &mut hunk_action,
            &view,
        );
        if outcome.notes_changed {
            self.diff_comments_dirty = true;
        }
        if let Some(action) = hunk_action.filter(|_| base.is_head()) {
            self.apply_diff_hunk_action(project, &head_key, action);
        }
        // Every pane that can take a note carries the way to send one.
        if base.is_head() {
            self.ui_diff_comment_footer(ui, palette, project.id);
        }
        let compare = self.compare_for(document);
        compare.changes = outcome.changes;
        compare.scroll_to = None;
        if compare.cursor.is_some_and(|at| at >= outcome.changes) {
            compare.cursor = outcome.changes.checked_sub(1);
        }
    }

    /// The bar above a comparison: what is compared, the layout, and the
    /// change navigation. Returns a step the person asked for: `Some(true)`
    /// for the next change, `Some(false)` for the previous one.
    fn ui_editor_compare_bar(
        &mut self,
        ui: &mut egui::Ui,
        project: &Project,
        document: &DocumentId,
        file: &str,
        base: &DiffBase,
        palette: &'static Palette,
    ) -> Option<bool> {
        let mut step = None;
        let mut chosen = None;
        ui.horizontal(|ui| {
            ui.label(RichText::new(tr("比較:")).color(palette.text_muted));
            match base {
                DiffBase::Commit(commit) => {
                    let short = short_commit(commit);
                    ui.label(tf!("{short} の変更（親 ↔ {short}）", short = short));
                    if ui.button(tr("HEAD と比較に戻る")).clicked() {
                        chosen = Some(DiffBase::Head);
                    }
                }
                DiffBase::Head | DiffBase::Revision(_) => {
                    let current = match base {
                        DiffBase::Revision(commit) => short_commit(commit),
                        _ => tr("HEAD（最新コミット）"),
                    };
                    ui.menu_button(current, |ui| {
                        chosen = self.ui_revision_picker(ui, project, file, base, palette);
                    });
                    ui.label(tr("↔ ローカル"));
                }
            }
            ui.separator();
            let mut layout = self.store.diff_layout;
            ui.selectable_value(&mut layout, DiffLayout::SideBySide, tr("左右"));
            ui.selectable_value(&mut layout, DiffLayout::Unified, tr("統合"));
            if layout != self.store.diff_layout {
                let previous = self.store.diff_layout;
                self.store.diff_layout = layout;
                if !self.persist() {
                    self.store.diff_layout = previous;
                }
            }
            ui.separator();
            if icon_button(ui, palette, ICON_CHANGE_PREVIOUS, tr("前の変更 (⇧F7)")).clicked()
            {
                step = Some(false);
            }
            if icon_button(ui, palette, ICON_CHANGE_NEXT, tr("次の変更 (F7)")).clicked() {
                step = Some(true);
            }
            let compare = &self.editor_compare;
            let position = match (compare.changes, compare.cursor) {
                (0, _) => tr("変更なし").to_string(),
                (total, cursor) => tf!(
                    "{n} / {total} 変更",
                    n = cursor.map_or(0, |at| at + 1),
                    total = total
                ),
            };
            ui.label(RichText::new(position).size(12.0).color(palette.text_muted));
        });
        if let Some(base) = chosen {
            self.set_compare_base(project, document, base);
        }
        step
    }

    /// The list inside the revision picker: HEAD, then the file's history.
    /// Returns the base the person picked.
    fn ui_revision_picker(
        &mut self,
        ui: &mut egui::Ui,
        project: &Project,
        file: &str,
        base: &DiffBase,
        palette: &'static Palette,
    ) -> Option<DiffBase> {
        let mut chosen = None;
        ui.label(
            RichText::new(tr("比較するリビジョン"))
                .size(12.0)
                .color(palette.text_muted),
        );
        if ui
            .selectable_label(base.is_head(), tr("HEAD（最新コミット）"))
            .clicked()
        {
            chosen = Some(DiffBase::Head);
            ui.close();
        }
        ui.separator();
        let key = (project.id, file.to_string());
        let loading = self
            .background_tasks
            .contains(&BackgroundKey::FileHistory(project.id, file.to_string()));
        if !self.file_history_cache.contains_key(&key) && !loading {
            self.request_file_history(project, file.to_string());
        }
        match self.file_history_cache.get(&key) {
            None => {
                ui.label(
                    RichText::new(tr("履歴を読み込んでいます…"))
                        .size(12.0)
                        .color(palette.text_muted),
                );
            }
            Some(Err(error)) => {
                ui.label(tf!("履歴を取得できません: {error}", error = error));
            }
            Some(Ok(revisions)) => {
                egui::ScrollArea::vertical()
                    .id_salt("revision-picker")
                    .max_height(360.0)
                    .show(ui, |ui| {
                        for revision in revisions {
                            let selected =
                                matches!(base, DiffBase::Revision(commit) if *commit == revision.commit);
                            let date = revision.date.get(..10).unwrap_or(&revision.date);
                            let row = format!(
                                "{}  {}  {}  {}",
                                revision.short, date, revision.author, revision.summary
                            );
                            if ui.selectable_label(selected, row).clicked() {
                                chosen = Some(DiffBase::Revision(revision.commit.clone()));
                                ui.close();
                            }
                        }
                    });
            }
        }
        chosen
    }

    /// The 注釈 view: the working copy, read-only, each line beside the commit
    /// that last changed it.
    pub(crate) fn ui_editor_annotate(
        &mut self,
        ui: &mut egui::Ui,
        project: &Project,
        document: &DocumentId,
        palette: &'static Palette,
        remaining: f32,
    ) {
        let file = document.path.to_string_lossy().into_owned();
        let key = (project.id, file.clone());
        let loading = self
            .background_tasks
            .contains(&BackgroundKey::Blame(project.id, file.clone()));
        if !self.blame_cache.contains_key(&key) && !loading {
            self.request_blame(project, file.clone());
        }
        let lines = match self.blame_cache.get(&key) {
            None => {
                ui.label(
                    RichText::new(tr("注釈を読み込んでいます…"))
                        .size(12.0)
                        .color(palette.text_muted),
                );
                return;
            }
            Some(Err(error)) => {
                ui.label(tf!("注釈を取得できません: {error}", error = error));
                return;
            }
            Some(Ok(BlameOutcome::Untracked)) => {
                ui.add_space(12.0);
                ui.label(
                    RichText::new(tr("このファイルはまだコミットされていません。"))
                        .color(palette.text_muted),
                );
                return;
            }
            Some(Ok(BlameOutcome::TooLarge)) => {
                ui.add_space(12.0);
                ui.label(
                    RichText::new(tf!(
                        "大きすぎるため注釈を表示できません（{limit} KiB まで）。",
                        limit = GIT_BLAME_MAX_BYTES / 1024
                    ))
                    .color(palette.text_muted),
                );
                return;
            }
            Some(Ok(BlameOutcome::Lines { lines, widest })) => (Arc::clone(lines), *widest),
        };
        let (lines, widest) = lines;
        let cell = monospace_cell_size(ui);
        let digits = lines.len().checked_ilog10().unwrap_or(0) as usize + 1;
        let gutter = cell.x * BLAME_GUTTER_COLUMNS as f32 + 12.0;
        let width = gutter + cell.x * (digits + 3 + widest) as f32;
        let mut action = None;
        egui::Frame::new()
            .fill(palette.inset)
            .stroke(egui::Stroke::new(1.0_f32, palette.border_subtle))
            .inner_margin(egui::Margin::same(4))
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                egui::ScrollArea::both()
                    .id_salt("annotate-scroll")
                    .auto_shrink([false, false])
                    .max_height(remaining.max(120.0))
                    .show_rows(ui, cell.y, lines.len(), |ui, range| {
                        ui.set_min_width(width);
                        for index in range {
                            let line = &lines[index];
                            let (rect, _) = ui.allocate_exact_size(
                                egui::vec2(width.max(ui.available_width()), cell.y),
                                egui::Sense::hover(),
                            );
                            let gutter_rect =
                                egui::Rect::from_min_size(rect.min, egui::vec2(gutter, cell.y));
                            let response = ui.interact(
                                gutter_rect,
                                ui.scope_id().with(("annotate-gutter", index)),
                                egui::Sense::click(),
                            );
                            ui.painter().rect_filled(
                                gutter_rect,
                                0.0,
                                if response.hovered() {
                                    palette.control_hovered
                                } else {
                                    palette.raised
                                },
                            );
                            if line.starts_run {
                                let text = if line.uncommitted {
                                    tr("未コミット").to_string()
                                } else {
                                    blame_gutter_text(line)
                                };
                                ui.painter().with_clip_rect(gutter_rect).text(
                                    gutter_rect.left_center() + egui::vec2(6.0, 0.0),
                                    egui::Align2::LEFT_CENTER,
                                    text,
                                    egui::FontId::monospace(MONOSPACE_SIZE),
                                    if line.uncommitted {
                                        palette.accent
                                    } else {
                                        palette.text_muted
                                    },
                                );
                            }
                            ui.painter().text(
                                egui::pos2(gutter_rect.right() + 6.0, rect.center().y),
                                egui::Align2::LEFT_CENTER,
                                format!("{:>digits$}  {}", index + 1, line.text),
                                egui::FontId::monospace(MONOSPACE_SIZE),
                                palette.text,
                            );
                            let response = if line.uncommitted {
                                response
                            } else {
                                response.on_hover_text(format!(
                                    "{} · {} · {} · {}",
                                    line.short, line.author, line.date, line.summary
                                ))
                            };
                            egui::Popup::menu(&response).show(|ui| {
                                if line.uncommitted {
                                    if ui.button(tr("HEAD とローカルを比較")).clicked() {
                                        action = Some(BlameAction::Compare(DiffBase::Head));
                                    }
                                    return;
                                }
                                if ui.button(tr("このコミットの変更を表示")).clicked() {
                                    action = Some(BlameAction::Compare(DiffBase::Commit(
                                        line.commit.clone(),
                                    )));
                                }
                                if ui.button(tr("このリビジョンとローカルを比較")).clicked()
                                {
                                    action = Some(BlameAction::Compare(DiffBase::Revision(
                                        line.commit.clone(),
                                    )));
                                }
                                if ui.button(tr("コミットハッシュをコピー")).clicked() {
                                    action = Some(BlameAction::Copy(line.commit.clone()));
                                }
                            });
                        }
                    });
            });
        match action {
            Some(BlameAction::Compare(base)) => self.set_compare_base(project, document, base),
            Some(BlameAction::Copy(commit)) => {
                ui.ctx().copy_text(commit);
                self.notice_briefly(tr("コミットハッシュをコピーしました。"));
            }
            None => {}
        }
    }
}

/// What the person picked from a blame line's menu.
enum BlameAction {
    Compare(DiffBase),
    Copy(String),
}

/// How many monospace columns the blame gutter takes: a short hash, a date,
/// and an author cut to fit.
pub(crate) const BLAME_GUTTER_COLUMNS: usize = 7 + 1 + 10 + 1 + BLAME_AUTHOR_COLUMNS;
const BLAME_AUTHOR_COLUMNS: usize = 12;

/// `a1b2c3d 2026-11-15 buddypia`, the author cut to the gutter.
pub(crate) fn blame_gutter_text(line: &BlameLine) -> String {
    let date = line.date.get(..10).unwrap_or(&line.date);
    let author: String = line.author.chars().take(BLAME_AUTHOR_COLUMNS).collect();
    format!("{} {date} {author}", line.short)
}
