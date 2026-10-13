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

impl InspectorTab {
    /// The three views, in the order their tabs are drawn.
    pub(crate) const ALL: [Self; 3] = [Self::Files, Self::Conversation, Self::Changes];
}

/// An edge of the session workspace a panel can be docked on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum DockEdge {
    Left,
    Right,
    Top,
    Bottom,
}

impl DockEdge {
    pub(crate) const ALL: [Self; 4] = [Self::Left, Self::Right, Self::Top, Self::Bottom];

    pub(crate) fn index(self) -> usize {
        self as usize
    }

    /// Top and bottom run across the workspace, so what shares one of them
    /// sits side by side; on the left and right it is stacked.
    pub(crate) fn is_horizontal(self) -> bool {
        matches!(self, Self::Top | Self::Bottom)
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Left => tr("左側"),
            Self::Right => tr("右側"),
            Self::Top => tr("上部"),
            Self::Bottom => tr("下部"),
        }
    }

    /// The menu entry that sends a panel here.
    pub(crate) fn move_label(self) -> &'static str {
        match self {
            Self::Left => tr("左側に移動"),
            Self::Right => tr("右側に移動"),
            Self::Top => tr("上部に移動"),
            Self::Bottom => tr("下部に移動"),
        }
    }
}

/// Something in the session workspace that can be dragged to an edge: the
/// session list, or one of the views about the selected session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum DockPanel {
    Sessions,
    View(InspectorTab),
}

impl DockPanel {
    fn index(self) -> usize {
        match self {
            Self::Sessions => 0,
            Self::View(InspectorTab::Files) => 1,
            Self::View(InspectorTab::Conversation) => 2,
            Self::View(InspectorTab::Changes) => 3,
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Sessions => tr("セッション一覧"),
            Self::View(tab) => tab.label(),
        }
    }
}

/// Where each panel of the session workspace is docked, which view each edge
/// is showing, and how big each edge is.
///
/// The views on one edge are tabs of one group. The session list is not a
/// tab: on an edge it shares with a group it sits beside it, split by a rule
/// (change 127's stacked column, generalised to every edge).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct DockLayout {
    /// By `DockPanel::index`.
    edges: [DockEdge; 4],
    /// By `DockEdge::index`: the view last chosen there.
    showing: [Option<InspectorTab>; 4],
    /// By `DockEdge::index`: width for left and right, height for top and
    /// bottom.
    pub(crate) sizes: [f32; 4],
    /// By `DockEdge::index`: the session list's share of an edge it shares.
    pub(crate) splits: [f32; 4],
}

impl Default for DockLayout {
    /// The list on the left and the three views on the right, files showing
    /// (change 139).
    fn default() -> Self {
        Self {
            edges: [
                DockEdge::Left,
                DockEdge::Right,
                DockEdge::Right,
                DockEdge::Right,
            ],
            showing: [None, Some(InspectorTab::Files), None, None],
            sizes: [
                SIDEBAR_DEFAULT_W,
                SIDEBAR_DEFAULT_W,
                DOCK_DEFAULT_H,
                DOCK_DEFAULT_H,
            ],
            splits: [SIDEBAR_SPLIT_DEFAULT; 4],
        }
    }
}

impl DockLayout {
    pub(crate) fn edge_of(&self, panel: DockPanel) -> DockEdge {
        self.edges[panel.index()]
    }

    /// Sends `panel` to `edge`; a view arrives as that edge's showing tab.
    /// False when it was already there.
    pub(crate) fn move_panel(&mut self, panel: DockPanel, edge: DockEdge) -> bool {
        let from = self.edge_of(panel);
        if from == edge {
            return false;
        }
        self.edges[panel.index()] = edge;
        if let DockPanel::View(tab) = panel {
            if self.showing[from.index()] == Some(tab) {
                self.showing[from.index()] = None;
            }
            self.showing[edge.index()] = Some(tab);
        }
        true
    }

    /// Sends every view on `from` to `to`, keeping the one that was showing.
    /// The session list stays where it is.
    pub(crate) fn move_views(&mut self, from: DockEdge, to: DockEdge) -> bool {
        let showing = self.shown_view(from, true);
        let mut moved = false;
        for tab in InspectorTab::ALL {
            if self.edge_of(DockPanel::View(tab)) == from {
                moved |= self.move_panel(DockPanel::View(tab), to);
            }
        }
        if let Some(tab) = showing.filter(|_| moved) {
            self.select(tab);
        }
        moved
    }

    /// Makes `tab` the showing view of whichever edge it is on.
    pub(crate) fn select(&mut self, tab: InspectorTab) {
        let edge = self.edge_of(DockPanel::View(tab));
        self.showing[edge.index()] = Some(tab);
    }

    /// The views docked on `edge`, in tab order.
    pub(crate) fn views_at(&self, edge: DockEdge) -> impl Iterator<Item = InspectorTab> + '_ {
        InspectorTab::ALL
            .into_iter()
            .filter(move |tab| self.edge_of(DockPanel::View(*tab)) == edge)
    }

    /// The view `edge` shows: the one last chosen there if it is still there
    /// and offered, else the first offered. A session with no project offers
    /// only its conversation, since files and changes are a project's.
    pub(crate) fn shown_view(&self, edge: DockEdge, has_project: bool) -> Option<InspectorTab> {
        let offered = |tab: &InspectorTab| has_project || *tab == InspectorTab::Conversation;
        self.showing[edge.index()]
            .filter(|tab| self.edge_of(DockPanel::View(*tab)) == edge && offered(tab))
            .or_else(|| self.views_at(edge).find(offered))
    }
}

/// Where a panel released at `pos` goes: the nearest edge of `area` when the
/// release is within `DOCK_DROP_BAND` of it, otherwise — the middle, or
/// outside the workspace — nowhere.
pub(crate) fn dock_drop_edge(pos: egui::Pos2, area: egui::Rect) -> Option<DockEdge> {
    if !area.contains(pos) || area.width() <= 0.0 || area.height() <= 0.0 {
        return None;
    }
    [
        (DockEdge::Left, (pos.x - area.left()) / area.width()),
        (DockEdge::Right, (area.right() - pos.x) / area.width()),
        (DockEdge::Top, (pos.y - area.top()) / area.height()),
        (DockEdge::Bottom, (area.bottom() - pos.y) / area.height()),
    ]
    .into_iter()
    .filter(|(_, share)| *share <= DOCK_DROP_BAND)
    .min_by(|a, b| a.1.total_cmp(&b.1))
    .map(|(edge, _)| edge)
}

/// The workspace divided among its occupied edges and the terminal.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct DockRects {
    /// By `DockEdge::index`; `None` for an edge with nothing on it or no room.
    pub(crate) edges: [Option<egui::Rect>; 4],
    pub(crate) center: egui::Rect,
}

/// Lays the occupied edges around the terminal. Top and bottom span `area`;
/// left and right fill the band between them, with a `DOCK_GAP` rule between
/// each edge and the centre. Each axis is fitted on its own so the centre keeps
/// `DOCK_CENTER_MIN_W` × `DOCK_CENTER_MIN_H`: edges shrink to their minimum,
/// then bottom and then top, right and then left, are dropped — never `keep`,
/// the edge the session list is on.
pub(crate) fn dock_rects(
    area: egui::Rect,
    occupied: [bool; 4],
    sizes: [f32; 4],
    keep: DockEdge,
) -> DockRects {
    let size = |edge: DockEdge| {
        let (default, min, max) = if edge.is_horizontal() {
            (DOCK_DEFAULT_H, DOCK_MIN_H, DOCK_MAX_H)
        } else {
            (SIDEBAR_DEFAULT_W, SIDEBAR_MIN_W, SIDEBAR_MAX_W)
        };
        let size = sizes[edge.index()];
        let size = if size.is_finite() { size } else { default };
        occupied[edge.index()].then_some(size.clamp(min, max))
    };
    // Top and bottom first: they span the workspace, and what they leave is
    // the band the left and right edges and the terminal share.
    let [top, bottom] = fit_axis(
        area.height(),
        DOCK_CENTER_MIN_H,
        DOCK_MIN_H,
        [size(DockEdge::Top), size(DockEdge::Bottom)],
        [keep == DockEdge::Top, keep == DockEdge::Bottom],
    );
    let [left, right] = fit_axis(
        area.width(),
        DOCK_CENTER_MIN_W,
        SIDEBAR_MIN_W,
        [size(DockEdge::Left), size(DockEdge::Right)],
        [keep == DockEdge::Left, keep == DockEdge::Right],
    );
    let taken = |length: Option<f32>| length.map_or(0.0, |length| length + DOCK_GAP);
    let band_top = area.top() + taken(top);
    let band_bottom = (area.bottom() - taken(bottom)).max(band_top);
    let center_left = area.left() + taken(left);
    let center_right = (area.right() - taken(right)).max(center_left);
    let mut edges = [None; 4];
    edges[DockEdge::Top.index()] =
        top.map(|h| egui::Rect::from_min_size(area.min, egui::vec2(area.width(), h)));
    edges[DockEdge::Bottom.index()] = bottom
        .map(|h| egui::Rect::from_min_max(egui::pos2(area.left(), area.bottom() - h), area.max));
    edges[DockEdge::Left.index()] = left.map(|w| {
        egui::Rect::from_min_max(
            egui::pos2(area.left(), band_top),
            egui::pos2(area.left() + w, band_bottom),
        )
    });
    edges[DockEdge::Right.index()] = right.map(|w| {
        egui::Rect::from_min_max(
            egui::pos2(area.right() - w, band_top),
            egui::pos2(area.right(), band_bottom),
        )
    });
    DockRects {
        edges,
        center: egui::Rect::from_min_max(
            egui::pos2(center_left, band_top),
            egui::pos2(center_right, band_bottom),
        ),
    }
}

/// The two opposite edges of one axis, fitted around a centre that keeps
/// `center_min` of `total`: each shrinks toward `min`, the second before the
/// first, and while that is not enough the second and then the first is
/// dropped — never one that `keep` marks.
fn fit_axis(
    total: f32,
    center_min: f32,
    min: f32,
    wanted: [Option<f32>; 2],
    keep: [bool; 2],
) -> [Option<f32>; 2] {
    let mut present = wanted;
    loop {
        let mut lengths = present;
        let used: f32 = lengths
            .iter()
            .flatten()
            .map(|length| length + DOCK_GAP)
            .sum();
        let mut over = center_min - (total - used);
        for length in lengths.iter_mut().rev().flatten() {
            let cut = (*length - min).clamp(0.0, over.max(0.0));
            *length -= cut;
            over -= cut;
        }
        if over <= 0.0 {
            return lengths;
        }
        match (0..2).rev().find(|&i| present[i].is_some() && !keep[i]) {
            Some(i) => present[i] = None,
            None => return lengths,
        }
    }
}

/// Two sections sharing `total`: the first takes `share` of it, each keeps
/// its minimum while there is room for both, and a length too short for both
/// is halved. A share that is not finite is the default.
pub(crate) fn split_lengths(total: f32, share: f32, min_first: f32, min_second: f32) -> (f32, f32) {
    let total = total.max(0.0);
    if total < min_first + min_second {
        return (total / 2.0, total / 2.0);
    }
    let share = if share.is_finite() {
        share
    } else {
        SIDEBAR_SPLIT_DEFAULT
    };
    let first = (total * share).clamp(min_first, total - min_second);
    (first, total - first)
}

/// Heights of the session list and the side panel stacked under it.
pub(crate) fn stacked_sidebar_heights(total_h: f32, split: f32) -> (f32, f32) {
    split_lengths(total_h, split, SIDEBAR_SECTION_MIN_H, SIDEBAR_PANEL_MIN_H)
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
    /// One edge's group of views: its tab row and the view `shown`.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn ui_session_inspector(
        &mut self,
        ui: &mut egui::Ui,
        session: &Session,
        project: Option<&Project>,
        palette: &Palette,
        edge: DockEdge,
        shown: InspectorTab,
        width: f32,
        height: f32,
    ) {
        // Files and changes are a project's; a session without one offers
        // only its conversation. Decided here rather than stored, so the tab
        // chosen for other sessions survives visiting this one.
        let offered = InspectorTab::ALL.map(|tab| {
            (self.dock.edge_of(DockPanel::View(tab)) == edge
                && (project.is_some() || tab == InspectorTab::Conversation))
                .then_some(tab)
        });
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
                    for which in offered.into_iter().flatten() {
                        let label = match which {
                            InspectorTab::Conversation if turns > 0 => {
                                tf!("会話 {count}", count = turns)
                            }
                            InspectorTab::Changes if changes > 0 => {
                                tf!("変更 {count}", count = changes)
                            }
                            _ => which.label().to_owned(),
                        };
                        let selected = shown == which;
                        let text = RichText::new(format!("{ICON_DRAG_HANDLE} {label}"))
                            .size(12.5)
                            .color(if selected {
                                palette.text_strong
                            } else {
                                palette.text_muted
                            });
                        let response = ui
                            .add(
                                egui::Button::new(text)
                                    .selected(selected)
                                    .corner_radius(egui::CornerRadius::same(RADIUS_CONTROL))
                                    .min_size(egui::vec2(0.0, CONTROL_HEIGHT_SMALL))
                                    .sense(egui::Sense::click_and_drag()),
                            )
                            .on_hover_text(tr("ドラッグして移動"));
                        paint_button_focus(ui, &response, palette);
                        if response.clicked() {
                            self.dock.select(which);
                        }
                        self.dock_grip(ui, &response, DockPanel::View(which));
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if icon_button(
                            ui,
                            palette,
                            ICON_DISCLOSURE_CLOSED,
                            tr("サイドパネルを畳む"),
                        )
                        .clicked()
                        {
                            self.show_session_inspector = false;
                        }
                        ui.menu_button(RichText::new(ICON_RESTORE).size(16.0), |ui| {
                            for to in DockEdge::ALL {
                                if ui
                                    .add_enabled(to != edge, egui::Button::new(to.move_label()))
                                    .clicked()
                                {
                                    if self.dock.move_views(edge, to) {
                                        self.notice_briefly(tf!(
                                            "パネルを{edge}に移動しました",
                                            edge = to.label()
                                        ));
                                    }
                                    ui.close_menu();
                                }
                            }
                        })
                        .response
                        .on_hover_text(tr("パネルの配置を変更"));
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

    /// What every panel's grip does beyond its own click: the open and closed
    /// hand, starting a drag of `panel`, and a right-click menu that sends it
    /// to an edge without dragging.
    pub(crate) fn dock_grip(&mut self, ui: &egui::Ui, grip: &egui::Response, panel: DockPanel) {
        if grip.dragged() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
        } else if grip.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
        }
        if grip.drag_started() {
            self.dock_drag = Some(panel);
        }
        let from = self.dock.edge_of(panel);
        grip.context_menu(|ui| {
            for to in DockEdge::ALL {
                if ui
                    .add_enabled(to != from, egui::Button::new(to.move_label()))
                    .clicked()
                {
                    self.dock_panel_to(panel, to);
                    ui.close_menu();
                }
            }
        });
    }

    /// Sends `panel` to `edge` and says so; nothing when it is already there.
    pub(crate) fn dock_panel_to(&mut self, panel: DockPanel, edge: DockEdge) {
        if self.dock.move_panel(panel, edge) {
            self.notice_briefly(tf!(
                "{panel}を{edge}に移動しました",
                panel = panel.label(),
                edge = edge.label()
            ));
        }
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
            if icon_button(ui, palette, ICON_REFRESH, tr("更新")).clicked() {
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
        let cached_matches = matches!(
            &self.session_file_tree_cache,
            Some((cached_root, _, _)) if cached_root == &root
        );
        if !cached_matches {
            if let Some(scan) = scan {
                let (visible, hidden) = split_hidden_entries(build_file_tree(&scan.paths));
                self.session_file_tree_cache = Some((root.clone(), visible, hidden));
            } else {
                self.session_file_tree_cache = None;
            }
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

        let badges = self.git_status_badge_map(if is_worktree { Uuid::nil() } else { project.id });
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
                                .size(11.5)
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
                .size(11.5)
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
                                ui.label(RichText::new(mark).size(11.5).color(colour));
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
                .size(13.5)
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
