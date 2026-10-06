use crate::app::OperonApp;
use crate::files::{build_file_tree, file_scan_warning};
use crate::prelude::*;
use crate::ui::files::{file_tree_rows, git_status_badge, FileTreeAction};
use crate::ui::widgets::{hairline, icon_button};
use crate::*;
use std::collections::BTreeSet;

/// Which view the side panel beside the terminal is showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub(crate) enum InspectorTab {
    Files,
    Conversation,
    Changes,
}

impl InspectorTab {
    pub(crate) fn label(&self) -> &'static str {
        match self {
            Self::Files => tr("ファイル"),
            Self::Conversation => tr("会話"),
            Self::Changes => tr("変更"),
        }
    }
}

/// Which side of the terminal the inspector panel is docked to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub(crate) enum SidebarSide {
    #[default]
    Left,
    Right,
}

impl SidebarSide {
    pub(crate) fn opposite(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

/// Layout of the session view: the terminal, and one side panel beside it.
///
/// There used to be two side columns, a conversation list and a file tree,
/// and between them they took about 480px from the terminal while one of them
/// was usually empty. They are tabs of one panel now.
///
/// Docked right, the panel is a column beside the terminal and this decides
/// its width. Docked left, it is stacked under the session list in that
/// column (change 127) and `stacked_sidebar_heights` splits it instead.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct SessionColumnsLayout {
    pub(crate) show_inspector: bool,
    pub(crate) inspector_w: f32,
    pub(crate) terminal_w: f32,
}

impl SessionColumnsLayout {
    /// Below this width the terminal keeps the whole row.
    pub(crate) const INSPECTOR_MIN_TOTAL: f32 = 640.0;
    pub(crate) const INSPECTOR_MIN_W: f32 = 240.0;
    pub(crate) const INSPECTOR_MAX_W: f32 = 300.0;
    pub(crate) const TERMINAL_MIN_W: f32 = 300.0;

    /// The panel is drawn when it is open and the row is at least
    /// `INSPECTOR_MIN_TOTAL` wide; it takes 28% of the row, clamped to
    /// 240–300, and the terminal takes the rest, at least 300.
    pub(crate) fn compute(total_avail_w: f32, show_session_inspector: bool) -> Self {
        let show_inspector = show_session_inspector && total_avail_w >= Self::INSPECTOR_MIN_TOTAL;
        let inspector_w = if show_inspector {
            (total_avail_w * 0.28).clamp(Self::INSPECTOR_MIN_W, Self::INSPECTOR_MAX_W)
        } else {
            0.0
        };
        let terminal_w = if show_inspector {
            (total_avail_w - inspector_w - SPACE_SM).max(Self::TERMINAL_MIN_W)
        } else {
            total_avail_w
        };
        Self {
            show_inspector,
            inspector_w,
            terminal_w,
        }
    }
}

/// Where a side-panel tab dragged from `origin` and released at `pos` sends
/// the panel: the other side when the release is inside `area` and past its
/// middle by more than a 24px dead zone, otherwise nowhere.
pub(crate) fn sidebar_drop_side(
    origin: SidebarSide,
    pos: egui::Pos2,
    area: egui::Rect,
) -> Option<SidebarSide> {
    const DEADZONE_PX: f32 = 24.0;
    if !area.contains(pos) {
        return None;
    }
    let center_x = area.center().x;
    let past_middle = match origin {
        SidebarSide::Left => pos.x > center_x + DEADZONE_PX,
        SidebarSide::Right => pos.x < center_x - DEADZONE_PX,
    };
    past_middle.then(|| origin.opposite())
}

/// Heights of the session list and the side panel stacked under it.
///
/// `split` is the list's share of `total_h`. The list keeps
/// `SIDEBAR_SECTION_MIN_H` and the panel `SIDEBAR_PANEL_MIN_H` while the
/// column has room for both; a column too short for that is halved. A share
/// that is not finite is the default.
pub(crate) fn stacked_sidebar_heights(total_h: f32, split: f32) -> (f32, f32) {
    let total_h = total_h.max(0.0);
    if total_h < SIDEBAR_SECTION_MIN_H + SIDEBAR_PANEL_MIN_H {
        return (total_h / 2.0, total_h / 2.0);
    }
    let split = if split.is_finite() {
        split
    } else {
        SIDEBAR_SPLIT_DEFAULT
    };
    let top = (total_h * split).clamp(SIDEBAR_SECTION_MIN_H, total_h - SIDEBAR_PANEL_MIN_H);
    (top, total_h - top)
}

/// The top level of a tree, split into what is shown and the entries whose
/// name starts with a dot. A repository's root is where the dot folders
/// gather — `.git`, `.claude`, `.github`, `.worktrees` — and listed first they
/// push every source folder below the fold.
pub(crate) fn split_hidden_entries(
    nodes: Vec<FileTreeNode>,
) -> (Vec<FileTreeNode>, Vec<FileTreeNode>) {
    nodes
        .into_iter()
        .partition(|node| !node.name.starts_with('.'))
}

/// The files a terminal has mentioned that resolved to a file under `root`,
/// relative to it, in path order and at most `limit`. A path that was looked
/// for and is not a file (`None`) is not one.
pub(crate) fn touched_files(
    resolved: &HashMap<String, Option<PathBuf>>,
    root: &Path,
    limit: usize,
) -> Vec<PathBuf> {
    resolved
        .values()
        .flatten()
        .filter_map(|path| path.strip_prefix(root).ok())
        .filter(|relative| !relative.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .take(limit)
        .collect()
}

/// The scanned paths whose text contains `query`, ignoring case, in scan
/// order and at most `limit`.
pub(crate) fn filter_scanned_paths(paths: &[PathBuf], query: &str, limit: usize) -> Vec<PathBuf> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return Vec::new();
    }
    paths
        .iter()
        .filter(|path| path.to_string_lossy().to_lowercase().contains(&query))
        .take(limit)
        .cloned()
        .collect()
}

impl OperonApp {
    /// The side panel: its tab row and whichever tab is showing.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn ui_session_inspector(
        &mut self,
        ui: &mut egui::Ui,
        session: &Session,
        project: Option<&Project>,
        palette: &Palette,
        width: f32,
        height: f32,
    ) {
        // Files and changes are a project's; a session without one has only
        // its conversation to show. Decided here rather than stored, so the
        // tab chosen for other sessions survives visiting this one.
        let shown = if project.is_some() {
            self.session_inspector_tab
        } else {
            InspectorTab::Conversation
        };
        let is_worktree = match (project, &session.worktree_path) {
            (Some(project), Some(path)) => path != &project.path,
            _ => false,
        };
        let turns = self
            .session_prompt_turns
            .get(&session.id)
            .map_or(0, Vec::len);
        let changes = match project {
            Some(project) if !is_worktree => match self.git_changes_cache.get(&project.id) {
                Some(Ok(snapshot)) => snapshot.files.len(),
                _ => 0,
            },
            _ => 0,
        };
        egui::Frame::default()
            .fill(palette.raised)
            .stroke(egui::Stroke::new(1.0, palette.border_subtle))
            .inner_margin(egui::Margin::same(8))
            .corner_radius(egui::CornerRadius::same(RADIUS_CARD))
            .show(ui, |ui| {
                ui.set_width((width - 16.0).max(100.0));
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = SPACE_XS;
                    let mut tab = |ui: &mut egui::Ui, which: InspectorTab, label: String| {
                        let selected = shown == which;
                        let text = RichText::new(label).size(12.5).color(if selected {
                            palette.text_strong
                        } else {
                            palette.text_muted
                        });
                        let response = ui
                            .add(
                                egui::Button::new(text)
                                    .selected(selected)
                                    .sense(egui::Sense::click_and_drag()),
                            )
                            .on_hover_text(tr("ドラッグして移動"));
                        if response.clicked() {
                            self.session_inspector_tab = which;
                        }
                        if response.drag_started() {
                            self.dragging_sidebar_tab = Some((which, self.session_inspector_side));
                        }
                    };
                    if project.is_some() {
                        tab(ui, InspectorTab::Files, tr("ファイル").to_owned());
                    }
                    tab(
                        ui,
                        InspectorTab::Conversation,
                        if turns > 0 {
                            tf!("会話 {count}", count = turns)
                        } else {
                            tr("会話").to_owned()
                        },
                    );
                    if project.is_some() {
                        tab(
                            ui,
                            InspectorTab::Changes,
                            if changes > 0 {
                                tf!("変更 {count}", count = changes)
                            } else {
                                tr("変更").to_owned()
                            },
                        );
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if icon_button(ui, ICON_DISCLOSURE_CLOSED, tr("サイドパネルを畳む"))
                            .clicked()
                        {
                            self.show_session_inspector = false;
                        }
                        let (dock_icon, dock_hint) = match self.session_inspector_side {
                            SidebarSide::Left => (ICON_RESTORE, tr("サイドバーを右側に移動")),
                            SidebarSide::Right => (ICON_RESTORE, tr("サイドバーを左側に移動")),
                        };
                        if icon_button(ui, dock_icon, dock_hint).clicked() {
                            self.session_inspector_side = self.session_inspector_side.opposite();
                            let notice = match self.session_inspector_side {
                                SidebarSide::Left => tr("サイドバーを左側に移動しました"),
                                SidebarSide::Right => tr("サイドバーを右側に移動しました"),
                            };
                            self.notice_briefly(notice);
                        }
                    });
                });
                ui.add_space(SPACE_XS);
                hairline(ui, palette);
                ui.add_space(SPACE_XS);
                let body_height = (height - 56.0).max(100.0);
                match (shown, project) {
                    (InspectorTab::Files, Some(project)) => {
                        self.ui_session_files_tab(ui, session, project, palette, body_height)
                    }
                    (InspectorTab::Changes, Some(project)) => self.ui_session_changes_tab(
                        ui,
                        session,
                        project,
                        palette,
                        is_worktree,
                        body_height,
                    ),
                    _ => self.ui_prompt_timeline(ui, palette, session.id),
                }
            });
    }

    /// The files tab: a filter, the files the terminal mentioned, the tree
    /// with its dot entries folded, and the scan limit in one line.
    pub(crate) fn ui_session_files_tab(
        &mut self,
        ui: &mut egui::Ui,
        session: &Session,
        project: &Project,
        palette: &Palette,
        height: f32,
    ) {
        let root = session
            .worktree_path
            .clone()
            .unwrap_or_else(|| project.path.clone());
        let is_worktree = session
            .worktree_path
            .as_ref()
            .is_some_and(|p| p != &project.path);

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = SPACE_XS;
            ui.add(
                egui::TextEdit::singleline(&mut self.session_file_filter)
                    .hint_text(tr("ファイルを絞り込む"))
                    .desired_width((ui.available_width() - 30.0).max(60.0)),
            );
            if icon_button(ui, ICON_REFRESH, tr("更新")).clicked() {
                if is_worktree {
                    self.session_file_cache.remove(&root);
                    self.request_session_files(&root);
                } else {
                    self.file_cache.remove(&project.id);
                    self.request_files(project);
                }
                self.session_file_tree_cache = None;
                self.session_filter_results = None;
            }
        });
        ui.add_space(SPACE_XS);

        if is_worktree {
            if !self.session_file_cache.contains_key(&root) {
                self.request_session_files(&root);
            }
        } else if !self.file_cache.contains_key(&project.id) {
            self.request_files(project);
        }
        let is_scanning = if is_worktree {
            self.background_tasks
                .contains(&BackgroundKey::WorktreeFiles(root.clone()))
                && !self.session_file_cache.contains_key(&root)
        } else {
            self.background_tasks
                .contains(&BackgroundKey::Files(project.id))
                && !self.file_cache.contains_key(&project.id)
        };
        if is_scanning {
            ui.label(
                RichText::new(tr("プロジェクトを読み込んでいます…"))
                    .size(12.0)
                    .color(palette.text_muted),
            );
        }
        let scan = if is_worktree {
            self.session_file_cache.get(&root)
        } else {
            self.file_cache.get(&project.id)
        };
        let scan_warning = scan.and_then(file_scan_warning);

        // Built once per root, not per frame: the scan can be twelve
        // thousand paths.
        let needs_rebuild = !matches!(
            &self.session_file_tree_cache,
            Some((cached_root, _, _)) if cached_root == &root
        );
        if needs_rebuild {
            self.session_file_tree_cache = scan.map(|scan| {
                let (visible, hidden) = split_hidden_entries(build_file_tree(&scan.paths));
                (root.clone(), visible, hidden)
            });
        }
        let query = self.session_file_filter.trim().to_owned();
        let filter_stale = !matches!(
            &self.session_filter_results,
            Some((cached_root, cached_query, _)) if cached_root == &root && cached_query == &query
        );
        // Not cached while the scan is still loading: an empty result saved
        // then would outlive the scan that arrives a moment later.
        if let Some(scan) = scan.filter(|_| !query.is_empty() && filter_stale) {
            self.session_filter_results = Some((
                root.clone(),
                query.clone(),
                filter_scanned_paths(&scan.paths, &query, FILE_FILTER_RESULT_LIMIT),
            ));
        }
        let touched_stale = !matches!(
            &self.touched_files_cache,
            Some((id, generation, cached_root, _))
                if *id == session.id
                    && *generation == self.resolved_paths_generation
                    && cached_root == &root
        );
        if touched_stale {
            self.touched_files_cache = Some((
                session.id,
                self.resolved_paths_generation,
                root.clone(),
                touched_files(&self.resolved_paths, &root, TOUCHED_FILES_LIMIT),
            ));
        }

        let badges = if is_worktree {
            HashMap::new()
        } else {
            self.git_status_badges(project.id)
        };
        let active_doc = self.active_document.as_ref().filter(|doc| doc.root == root);
        let mut actions = Vec::new();
        let mut toggle_hidden = false;
        let scroll_height = (height - if scan_warning.is_some() { 64.0 } else { 40.0 }).max(80.0);
        egui::ScrollArea::vertical()
            .id_salt(("session-file-tree-scroll", session.id))
            .auto_shrink([false, false])
            .max_height(scroll_height)
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 1.0;
                if !query.is_empty() {
                    let results = self
                        .session_filter_results
                        .as_ref()
                        .map(|(_, _, results)| results.as_slice())
                        .unwrap_or(&[]);
                    if results.is_empty() {
                        ui.label(
                            RichText::new(tr("一致するファイルはありません"))
                                .size(12.0)
                                .color(palette.text_muted),
                        );
                    }
                    for path in results {
                        if path_row(ui, path, palette).clicked() {
                            actions.push(FileTreeAction::Open(path.clone()));
                        }
                    }
                    return;
                }
                if let Some((_, _, _, touched)) = &self.touched_files_cache {
                    if !touched.is_empty() {
                        ui.label(
                            RichText::new(tr("ターミナルに出たファイル"))
                                .size(11.0)
                                .color(palette.text_faint),
                        );
                        for path in touched {
                            if path_row(ui, path, palette).clicked() {
                                actions.push(FileTreeAction::Open(path.clone()));
                            }
                        }
                        ui.add_space(SPACE_XS);
                        hairline(ui, palette);
                        ui.add_space(SPACE_XS);
                    }
                }
                let Some((cached_root, visible, hidden)) = &self.session_file_tree_cache else {
                    return;
                };
                if cached_root != &root {
                    return;
                }
                if visible.is_empty() && hidden.is_empty() {
                    ui.label(
                        RichText::new(tr("読み取り可能なファイルはありません。"))
                            .size(12.0)
                            .color(palette.text_muted),
                    );
                    return;
                }
                file_tree_rows(
                    ui,
                    visible,
                    project.id,
                    &self.expanded_directories,
                    &badges,
                    active_doc,
                    false,
                    0,
                    palette,
                    &mut actions,
                );
                if !hidden.is_empty() {
                    let caret = if self.session_hidden_expanded {
                        ICON_DISCLOSURE_OPEN
                    } else {
                        ICON_DISCLOSURE_CLOSED
                    };
                    let row = ui.add(
                        egui::Label::new(
                            RichText::new(tf!(
                                "{caret} 隠しファイル・フォルダ {count} 件",
                                caret = caret,
                                count = hidden.len()
                            ))
                            .size(12.0)
                            .color(palette.text_faint),
                        )
                        .sense(egui::Sense::click()),
                    );
                    if row.clicked() {
                        toggle_hidden = true;
                    }
                    if self.session_hidden_expanded {
                        file_tree_rows(
                            ui,
                            hidden,
                            project.id,
                            &self.expanded_directories,
                            &badges,
                            active_doc,
                            false,
                            0,
                            palette,
                            &mut actions,
                        );
                    }
                }
            });
        if let Some(warning) = scan_warning {
            ui.add_space(SPACE_XS);
            ui.label(
                RichText::new(format!(
                    "{ICON_NOTICE} {}",
                    tr("一部のファイルを省略しています")
                ))
                .size(11.0)
                .color(palette.text_faint),
            )
            .on_hover_text(warning);
        }
        if toggle_hidden {
            self.session_hidden_expanded = !self.session_hidden_expanded;
        }
        for action in actions {
            match action {
                FileTreeAction::Open(path) => {
                    self.navigate_to_file(project, root.clone(), path, None);
                }
                FileTreeAction::Toggle(path) => {
                    let key = (project.id, path);
                    if !self.expanded_directories.remove(&key) {
                        self.expanded_directories.insert(key);
                    }
                }
            }
        }
    }

    /// The changes tab: the project's changed files, read from the cache the
    /// Git tab fills. Git never runs in the draw path.
    pub(crate) fn ui_session_changes_tab(
        &mut self,
        ui: &mut egui::Ui,
        session: &Session,
        project: &Project,
        palette: &Palette,
        is_worktree: bool,
        height: f32,
    ) {
        if is_worktree {
            ui.label(
                RichText::new(tr(
                    "worktree の変更はプロジェクトの「変更」タブで確認できます。",
                ))
                .size(12.0)
                .color(palette.text_muted),
            );
            return;
        }
        if !self.git_changes_cache.contains_key(&project.id) {
            self.request_git_changes(project);
        }
        let mut open = None;
        match self.git_changes_cache.get(&project.id) {
            None => {
                ui.label(
                    RichText::new(tr("変更を読み込んでいます…"))
                        .size(12.0)
                        .color(palette.text_muted),
                );
            }
            Some(Err(error)) => {
                ui.label(
                    RichText::new(tf!("変更を読み取れませんでした: {error}", error = error))
                        .size(12.0)
                        .color(palette.danger),
                );
            }
            Some(Ok(snapshot)) if snapshot.files.is_empty() => {
                ui.label(
                    RichText::new(tr("変更はありません"))
                        .size(12.0)
                        .color(palette.text_muted),
                );
            }
            Some(Ok(snapshot)) => {
                egui::ScrollArea::vertical()
                    .id_salt(("session-changes-scroll", session.id))
                    .auto_shrink([false, false])
                    .max_height(height)
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing.y = 1.0;
                        for file in &snapshot.files {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = SPACE_XS;
                                let (mark, colour) = git_status_badge(&file.status, palette);
                                ui.label(RichText::new(mark).size(11.0).color(colour));
                                let row = ui.add(
                                    egui::Label::new(
                                        RichText::new(&file.path).size(12.5).color(palette.text),
                                    )
                                    .truncate()
                                    .sense(egui::Sense::click()),
                                );
                                if row.clicked() {
                                    open = Some(PathBuf::from(&file.path));
                                }
                            });
                        }
                    });
            }
        }
        if let Some(path) = open {
            self.navigate_to_file(project, project.path.clone(), path, None);
        }
    }
}

/// One path as a clickable row: its file icon and the path relative to the
/// root it was listed from.
fn path_row(ui: &mut egui::Ui, path: &Path, palette: &Palette) -> egui::Response {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = SPACE_XS;
        let name = path
            .file_name()
            .map(|name| name.to_string_lossy())
            .unwrap_or_default();
        ui.label(
            RichText::new(file_tree_icon(&name))
                .size(13.0)
                .color(palette.text_muted),
        );
        ui.add(
            egui::Label::new(
                RichText::new(path.to_string_lossy())
                    .size(12.5)
                    .color(palette.text),
            )
            .truncate()
            .sense(egui::Sense::click()),
        )
    })
    .inner
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_columns_layout_thresholds() {
        // Below 640 the terminal keeps the row, whether or not the panel is open.
        let l639 = SessionColumnsLayout::compute(639.0, true);
        assert!(!l639.show_inspector);
        assert_eq!(l639.inspector_w, 0.0);
        assert_eq!(l639.terminal_w, 639.0);

        // At 640 the panel appears at its minimum width.
        let l640 = SessionColumnsLayout::compute(640.0, true);
        assert!(l640.show_inspector);
        assert_eq!(l640.inspector_w, 240.0);
        assert_eq!(l640.terminal_w, 640.0 - 240.0 - SPACE_SM);

        // 28% of the row, inside the clamp.
        let l1000 = SessionColumnsLayout::compute(1000.0, true);
        assert_eq!(l1000.inspector_w, 280.0);

        // Folded: nothing is taken from the terminal at any width.
        let folded = SessionColumnsLayout::compute(1400.0, false);
        assert!(!folded.show_inspector);
        assert_eq!(folded.inspector_w, 0.0);
        assert_eq!(folded.terminal_w, 1400.0);
    }

    #[test]
    fn test_session_columns_layout_adversarial_boundaries() {
        let zero = SessionColumnsLayout::compute(0.0, true);
        assert!(!zero.show_inspector);
        assert_eq!(zero.terminal_w, 0.0);

        let neg = SessionColumnsLayout::compute(-100.0, true);
        assert!(!neg.show_inspector);
        assert_eq!(neg.inspector_w, 0.0);
        assert_eq!(neg.terminal_w, -100.0);

        // Huge width: the panel stops at its maximum.
        let huge = SessionColumnsLayout::compute(10_000.0, true);
        assert_eq!(huge.inspector_w, 300.0);
        assert!(huge.terminal_w > 9000.0);

        let sub = SessionColumnsLayout::compute(639.99, true);
        assert!(!sub.show_inspector);

        // Non-finite widths never panic.
        let nan = SessionColumnsLayout::compute(f32::NAN, true);
        assert!(!nan.show_inspector);
        assert_eq!(nan.inspector_w, 0.0);
        assert!(nan.terminal_w.is_nan());

        let inf = SessionColumnsLayout::compute(f32::INFINITY, true);
        assert!(inf.show_inspector);
        assert_eq!(inf.inspector_w, 300.0);
        assert!(inf.terminal_w.is_infinite());

        let neg_inf = SessionColumnsLayout::compute(f32::NEG_INFINITY, true);
        assert!(!neg_inf.show_inspector);
        assert_eq!(neg_inf.inspector_w, 0.0);
        assert!(neg_inf.terminal_w.is_infinite());
    }
}
