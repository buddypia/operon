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
    /// Every panel, in `index` order.
    pub(crate) const ALL: [Self; 4] = [
        Self::Sessions,
        Self::View(InspectorTab::Files),
        Self::View(InspectorTab::Conversation),
        Self::View(InspectorTab::Changes),
    ];

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

/// The order a person dragged the session column into (change 181): the
/// projects, and every project's sessions. What it does not list — added
/// since the last move — is drawn ahead of what it lists, in the column's own
/// order, so a new project or session is never lost at the bottom.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SessionArrangement {
    #[serde(default)]
    pub(crate) projects: Vec<Uuid>,
    #[serde(default)]
    pub(crate) sessions: Vec<Uuid>,
    /// Projects folded in the session page's column (change 237). A file
    /// written before the field existed unfolds everything.
    #[serde(default)]
    pub(crate) folded: Vec<Uuid>,
}

pub(crate) fn session_arrangement_path(data_file: &Path) -> PathBuf {
    data_file
        .parent()
        .unwrap_or(Path::new("."))
        .join(SESSION_ARRANGEMENT_FILE_NAME)
}

/// A file that is missing, unreadable, unparsable, or past its ceiling is no
/// arrangement: the column falls back to its own order, and the next move
/// writes the file again. The ceiling is counted on the open handle, not read
/// off its metadata, and a file past it is not half-read.
pub(crate) fn load_session_arrangement(data_file: &Path) -> SessionArrangement {
    let Ok(file) = fs::File::open(session_arrangement_path(data_file)) else {
        return SessionArrangement::default();
    };
    let mut raw = Vec::new();
    match file
        .take(SESSION_ARRANGEMENT_FILE_MAX_BYTES + 1)
        .read_to_end(&mut raw)
    {
        Ok(read) if read as u64 <= SESSION_ARRANGEMENT_FILE_MAX_BYTES => {}
        _ => return SessionArrangement::default(),
    }
    serde_json::from_slice(&raw).unwrap_or_default()
}

/// Compact rather than pretty: the file is a list of ids nobody reads by
/// hand, and the ceiling counts its bytes. A write that committed but whose
/// folder was not synced has landed, and is not reported as a failure.
pub(crate) fn save_session_arrangement(
    data_file: &Path,
    arrangement: &SessionArrangement,
) -> Result<()> {
    let mut encoded = serde_json::to_vec(arrangement)?;
    encoded.push(b'\n');
    write_file_atomically(&session_arrangement_path(data_file), &encoded)?;
    Ok(())
}

/// Sorts `items`, already in the column's own order, into `listed`'s: what
/// it does not list first and as it was, then what it lists, in its order.
/// A stable sort keyed by a position lookup, so it allocates nothing and may
/// run every frame.
pub(crate) fn arrange_by<T>(listed: &[Uuid], items: &mut [T], id: impl Fn(&T) -> Uuid) {
    items.sort_by_key(|item| {
        let id = id(item);
        listed
            .iter()
            .position(|listed| *listed == id)
            .map_or((false, 0), |at| (true, at))
    });
}

/// The column's projects: by name, case aside, then as arranged.
pub(crate) fn sort_column_projects<P: std::borrow::Borrow<Project>>(
    projects: &mut [P],
    arrangement: &SessionArrangement,
) {
    projects.sort_by_key(|project| project.borrow().name.to_lowercase());
    arrange_by(&arrangement.projects, projects, |project| {
        project.borrow().id
    });
}

/// One project's sessions in the column: newest first, then as arranged.
pub(crate) fn sort_column_sessions<S: std::borrow::Borrow<Session>>(
    sessions: &mut [S],
    arrangement: &SessionArrangement,
) {
    sessions.sort_by_key(|session| Reverse(session.borrow().created_at));
    arrange_by(&arrangement.sessions, sessions, |session| {
        session.borrow().id
    });
}

/// Whether a release at `slot` among the `visible` rows puts `moved`
/// somewhere else on screen: not into either gap beside where it sits.
pub(crate) fn slot_moves(visible: &[Uuid], moved: Uuid, slot: usize) -> bool {
    visible
        .iter()
        .position(|id| *id == moved)
        .is_some_and(|at| slot != at && slot != at + 1)
}

/// Moves `moved` within `order` — one level of the column, every item of it,
/// drawn or not — to `slot` among the `visible` ones: before `visible[slot]`,
/// or after the last of them. What is not drawn keeps its place relative to
/// the rest. Whether anything moved.
pub(crate) fn place(order: &mut Vec<Uuid>, moved: Uuid, visible: &[Uuid], slot: usize) -> bool {
    if !slot_moves(visible, moved, slot) {
        return false;
    }
    let (anchor, after) = match visible.get(slot) {
        Some(anchor) => (*anchor, false),
        // `slot_moves` found `moved` among them, so there is a last.
        None => match visible.last() {
            Some(last) => (*last, true),
            None => return false,
        },
    };
    let Some(from) = order.iter().position(|id| *id == moved) else {
        return false;
    };
    order.remove(from);
    let Some(at) = order.iter().position(|id| *id == anchor) else {
        order.insert(from, moved);
        return false;
    };
    order.insert(if after { at + 1 } else { at }, moved);
    true
}

/// Where a release at height `y` lands among rows drawn top to bottom: the
/// index of the first row whose middle is below it, or past the last.
pub(crate) fn drop_slot(y: f32, rects: &[egui::Rect]) -> usize {
    rects
        .iter()
        .position(|rect| y < rect.center().y)
        .unwrap_or(rects.len())
}

/// The height of the line that marks `slot` among `rects`: halfway across
/// the gap between two rows, or `gap / 2` outside the first or the last.
pub(crate) fn slot_line_y(rects: &[egui::Rect], slot: usize, gap: f32) -> Option<f32> {
    let above = slot.checked_sub(1).and_then(|index| rects.get(index));
    match (above, rects.get(slot)) {
        (Some(above), Some(below)) => Some((above.bottom() + below.top()) / 2.0),
        (None, Some(below)) => Some(below.top() - gap / 2.0),
        (Some(above), None) => Some(above.bottom() + gap / 2.0),
        (None, None) => None,
    }
}

/// What is being dragged inside the session column: a project's card, or a
/// session's row, which stays among its own project's.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ColumnDrag {
    Project(Uuid),
    Session { project: Uuid, session: Uuid },
}

impl ColumnDrag {
    /// The project or session being moved.
    pub(crate) fn moved(self) -> Uuid {
        match self {
            Self::Project(project) => project,
            Self::Session { session, .. } => session,
        }
    }
}

/// The items of the dragged level, as this frame drew them: what a release
/// can land among. Filled only while a drag is held.
#[derive(Debug, Default)]
pub(crate) struct ColumnTargets {
    pub(crate) ids: Vec<Uuid>,
    pub(crate) rects: Vec<egui::Rect>,
}

impl ColumnTargets {
    pub(crate) fn push(&mut self, id: Uuid, rect: egui::Rect) {
        self.ids.push(id);
        self.rects.push(rect);
    }

    /// The slot a release at `pointer` would put `moved` in, when that moves
    /// it at all.
    pub(crate) fn slot(&self, moved: Uuid, pointer: Option<egui::Pos2>) -> Option<usize> {
        let slot = drop_slot(pointer?.y, &self.rects);
        slot_moves(&self.ids, moved, slot).then_some(slot)
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

    /// The workspace divided for this layout. An edge is occupied by the
    /// session list, and — while the views are open, `Some(has_project)` — by
    /// a view it shows. Drawing and the move check both read this, so what is
    /// checked is what is drawn.
    pub(crate) fn rects(&self, area: egui::Rect, views: Option<bool>) -> DockRects {
        let sessions = self.edge_of(DockPanel::Sessions);
        let occupied = DockEdge::ALL.map(|edge| {
            edge == sessions
                || views.is_some_and(|has_project| self.shown_view(edge, has_project).is_some())
        });
        dock_rects(area, occupied, self.sizes, sessions)
    }

    /// Which panels `area` has room for with every view open, in
    /// `DockPanel::ALL` order. Judged open so that a move made while the views
    /// are folded, or while a session without a project offers only its
    /// conversation, cannot hide them the moment they are shown.
    fn with_room(&self, area: egui::Rect) -> [bool; 4] {
        let rects = self.rects(area, Some(true));
        DockPanel::ALL.map(|panel| rects.edges[self.edge_of(panel).index()].is_some())
    }

    /// Whether `after` keeps every panel this layout has room for, and gives
    /// room to every panel it sends somewhere else. A small window gives up an
    /// edge to keep the terminal its minimum; a move onto that edge, or one
    /// that takes the room another edge had, is refused (lesson 070).
    pub(crate) fn keeps_on_screen(&self, after: &DockLayout, area: egui::Rect) -> bool {
        let (before, now) = (self.with_room(area), after.with_room(area));
        DockPanel::ALL
            .into_iter()
            .zip(before)
            .zip(now)
            .all(|((panel, had), has)| has || (!had && self.edge_of(panel) == after.edge_of(panel)))
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

/// What a refused move says, wherever it is refused: the drop zone's hint,
/// the greyed menu entry, and the notice after a release.
pub(crate) fn dock_no_room(edge: DockEdge) -> String {
    tf!(
        "ウィンドウが小さいため{edge}には置けません",
        edge = edge.label()
    )
}

/// What releasing `panel`, docked on `from`, at `at` does: the move, the
/// refusal when it does not `fits`, or — in the middle and on its own edge
/// alike — nothing.
pub(crate) fn dock_drop_hint(
    panel: DockPanel,
    from: DockEdge,
    at: Option<DockEdge>,
    fits: bool,
) -> String {
    match at.filter(|edge| *edge != from) {
        Some(edge) if fits => tf!(
            "{panel}を{edge}に移動",
            panel = panel.label(),
            edge = edge.label()
        ),
        Some(edge) => dock_no_room(edge),
        None => tr("ここで離すとキャンセル").to_owned(),
    }
}

/// One "move to `to`" entry of a dock menu: greyed when the move changes
/// nothing, or — saying why on hover — when it does not `fits`. True when
/// clicked.
fn dock_menu_entry(ui: &mut egui::Ui, to: DockEdge, moves: bool, fits: bool) -> bool {
    let entry = ui.add_enabled(moves && fits, egui::Button::new(to.move_label()));
    let entry = if moves && !fits {
        entry.on_disabled_hover_ui(|ui| {
            ui.label(dock_no_room(to));
        })
    } else {
        entry
    };
    entry.clicked()
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

/// Where on a terminal a dragged session lands (change 226): beside it on one
/// side, or in its place — as an editor tab is dropped on an editor group.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PaneDrop {
    Left,
    Right,
    Top,
    Bottom,
    Center,
}

impl PaneDrop {
    fn label(self) -> &'static str {
        match self {
            Self::Left => tr("左に分割"),
            Self::Right => tr("右に分割"),
            Self::Top => tr("上に分割"),
            Self::Bottom => tr("下に分割"),
            Self::Center => tr("ここに表示"),
        }
    }
}

/// One terminal, or two arrangements sharing a rect: side by side or stacked,
/// the first taking `share` of it.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum PaneNode {
    Leaf(Uuid),
    Split {
        side_by_side: bool,
        share: f32,
        first: Box<PaneNode>,
        second: Box<PaneNode>,
    },
}

impl PaneNode {
    fn leaves(&self, out: &mut Vec<Uuid>) {
        match self {
            Self::Leaf(id) => out.push(*id),
            Self::Split { first, second, .. } => {
                first.leaves(out);
                second.leaves(out);
            }
        }
    }

    fn for_each_leaf_mut(&mut self, f: &mut impl FnMut(&mut Uuid)) {
        match self {
            Self::Leaf(id) => f(id),
            Self::Split { first, second, .. } => {
                first.for_each_leaf_mut(f);
                second.for_each_leaf_mut(f);
            }
        }
    }

    /// This arrangement without the terminals `keep` refuses; a split that
    /// loses one side becomes the other.
    fn retained(self, keep: &impl Fn(Uuid) -> bool) -> Option<Self> {
        match self {
            Self::Leaf(id) => keep(id).then_some(self),
            Self::Split {
                side_by_side,
                share,
                first,
                second,
            } => match (first.retained(keep), second.retained(keep)) {
                (Some(first), Some(second)) => Some(Self::Split {
                    side_by_side,
                    share,
                    first: Box::new(first),
                    second: Box::new(second),
                }),
                (Some(only), None) | (None, Some(only)) => Some(only),
                (None, None) => None,
            },
        }
    }

    /// Puts `moved` beside the terminal showing `target`, on `drop`'s side.
    fn split_at(&mut self, target: Uuid, moved: Uuid, drop: PaneDrop) -> bool {
        match self {
            Self::Leaf(id) if *id == target => {
                let (side_by_side, moved_first) = match drop {
                    PaneDrop::Left => (true, true),
                    PaneDrop::Right => (true, false),
                    PaneDrop::Top => (false, true),
                    PaneDrop::Bottom | PaneDrop::Center => (false, false),
                };
                let (first, second) = if moved_first {
                    (moved, target)
                } else {
                    (target, moved)
                };
                *self = Self::Split {
                    side_by_side,
                    share: 0.5,
                    first: Box::new(Self::Leaf(first)),
                    second: Box::new(Self::Leaf(second)),
                };
                true
            }
            Self::Leaf(_) => false,
            Self::Split { first, second, .. } => {
                first.split_at(target, moved, drop) || second.split_at(target, moved, drop)
            }
        }
    }

    /// The least size this arrangement is drawn at without a terminal under
    /// `PANE_MIN_W` × `PANE_MIN_H`.
    fn min_size(&self) -> egui::Vec2 {
        match self {
            Self::Leaf(_) => egui::vec2(PANE_MIN_W, PANE_MIN_H),
            Self::Split {
                side_by_side,
                first,
                second,
                ..
            } => {
                let (a, b) = (first.min_size(), second.min_size());
                if *side_by_side {
                    egui::vec2(a.x + DOCK_GAP + b.x, a.y.max(b.y))
                } else {
                    egui::vec2(a.x.max(b.x), a.y + DOCK_GAP + b.y)
                }
            }
        }
    }

    fn layout(&self, rect: egui::Rect, out: &mut PaneRects) {
        match self {
            Self::Leaf(id) => out.panes.push((*id, rect)),
            Self::Split {
                side_by_side,
                share,
                first,
                second,
            } => {
                let index = out.rules.len();
                let (min_first, min_second) = (first.min_size(), second.min_size());
                let (first_rect, rule, second_rect) = if *side_by_side {
                    let (a, _) =
                        split_lengths(rect.width() - DOCK_GAP, *share, min_first.x, min_second.x);
                    let rule_left = rect.left() + a;
                    (
                        rect.with_max_x(rule_left),
                        rect.with_min_x(rule_left).with_max_x(rule_left + DOCK_GAP),
                        rect.with_min_x(rule_left + DOCK_GAP),
                    )
                } else {
                    let (a, _) =
                        split_lengths(rect.height() - DOCK_GAP, *share, min_first.y, min_second.y);
                    let rule_top = rect.top() + a;
                    (
                        rect.with_max_y(rule_top),
                        rect.with_min_y(rule_top).with_max_y(rule_top + DOCK_GAP),
                        rect.with_min_y(rule_top + DOCK_GAP),
                    )
                };
                out.rules.push(PaneRule {
                    index,
                    rect: rule,
                    span: rect,
                    side_by_side: *side_by_side,
                });
                first.layout(first_rect, out);
                second.layout(second_rect, out);
            }
        }
    }

    /// Sets the share of the split numbered `index` in `layout`'s order, and
    /// counts the splits passed on the way.
    fn set_share(&mut self, index: usize, value: f32, seen: &mut usize) -> bool {
        let Self::Split {
            share,
            first,
            second,
            ..
        } = self
        else {
            return false;
        };
        if *seen == index {
            *share = value;
            return true;
        }
        *seen += 1;
        first.set_share(index, value, seen) || second.set_share(index, value, seen)
    }
}

/// The rule between the two sides of one split, as drawn.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PaneRule {
    /// The split's number, for `PaneLayout::set_share`.
    pub(crate) index: usize,
    pub(crate) rect: egui::Rect,
    /// The whole rect the split divides, which a drag's share is read against.
    pub(crate) span: egui::Rect,
    pub(crate) side_by_side: bool,
}

/// The terminal area divided: each terminal's rect in reading order, and the
/// rules between them.
#[derive(Debug, Default)]
pub(crate) struct PaneRects {
    pub(crate) panes: Vec<(Uuid, egui::Rect)>,
    pub(crate) rules: Vec<PaneRule>,
}

/// The terminals the session workspace shows, and how they share it
/// (change 226). Empty or a single terminal means the workspace shows the
/// selected session, as it did before; only a split is a layout. In memory,
/// as `DockLayout` is.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct PaneLayout {
    root: Option<PaneNode>,
}

impl PaneLayout {
    pub(crate) fn is_split(&self) -> bool {
        matches!(self.root, Some(PaneNode::Split { .. }))
    }

    /// The one terminal left when the layout is not a split.
    pub(crate) fn single(&self) -> Option<Uuid> {
        match self.root {
            Some(PaneNode::Leaf(id)) => Some(id),
            _ => None,
        }
    }

    /// The sessions on screen in reading order: left before right, top
    /// before bottom.
    pub(crate) fn sessions(&self) -> Vec<Uuid> {
        let mut out = Vec::new();
        if let Some(root) = &self.root {
            root.leaves(&mut out);
        }
        out
    }

    pub(crate) fn contains(&self, id: Uuid) -> bool {
        self.sessions().contains(&id)
    }

    fn replace(&mut self, from: Uuid, to: Uuid) {
        if let Some(root) = &mut self.root {
            root.for_each_leaf_mut(&mut |id| {
                if *id == from {
                    *id = to;
                }
            });
        }
    }

    /// The layout after `moved` is dropped on the terminal showing `target`
    /// — which, while the layout is not a split, is the one terminal on
    /// screen. `None` when the drop changes nothing. Whether the result has
    /// room is `fits`'s question.
    pub(crate) fn placed(&self, moved: Uuid, target: Uuid, drop: PaneDrop) -> Option<Self> {
        if moved == target {
            return None;
        }
        let mut after = self.clone();
        if !after.is_split() {
            after.root = Some(PaneNode::Leaf(target));
        }
        if !after.contains(target) {
            return None;
        }
        if drop == PaneDrop::Center {
            if after.contains(moved) {
                if let Some(root) = &mut after.root {
                    root.for_each_leaf_mut(&mut |id| {
                        if *id == moved {
                            *id = target;
                        } else if *id == target {
                            *id = moved;
                        }
                    });
                }
            } else {
                after.replace(target, moved);
            }
            return Some(after);
        }
        after.remove(moved);
        after.root.as_mut()?.split_at(target, moved, drop);
        Some(after)
    }

    pub(crate) fn remove(&mut self, id: Uuid) {
        self.retain(|candidate| candidate != id);
    }

    pub(crate) fn retain(&mut self, keep: impl Fn(Uuid) -> bool) {
        self.root = self.root.take().and_then(|root| root.retained(&keep));
    }

    pub(crate) fn rects(&self, area: egui::Rect) -> PaneRects {
        let mut out = PaneRects::default();
        if let Some(root) = &self.root {
            root.layout(area, &mut out);
        }
        out
    }

    /// Sets the first side's share of the split `PaneRule::index` names.
    pub(crate) fn set_share(&mut self, index: usize, share: f32) {
        if let Some(root) = &mut self.root {
            root.set_share(index, share.clamp(0.0, 1.0), &mut 0);
        }
    }

    /// Whether `area` holds this layout: at most `PANE_MAX` terminals, none
    /// under its minimum size.
    pub(crate) fn fits(&self, area: egui::Rect) -> bool {
        let Some(root) = &self.root else {
            return true;
        };
        let min = root.min_size();
        self.sessions().len() <= PANE_MAX && area.width() >= min.x && area.height() >= min.y
    }
}

/// A view in the central workspace. Documents keep their exact worktree identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum WorkspaceTab {
    Session(Uuid),
    Document(DocumentId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WorkspaceGroup {
    pub(crate) id: Uuid,
    pub(crate) tabs: Vec<WorkspaceTab>,
    pub(crate) active: usize,
}

/// Ordered tabs in independently selectable groups; closing a view never owns
/// the lifetime of its session or document buffer.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct WorkspaceTabs {
    pub(crate) groups: Vec<WorkspaceGroup>,
    pub(crate) focused: Option<Uuid>,
    pub(crate) layout: PaneLayout,
}

impl WorkspaceTabs {
    pub(crate) fn open(&mut self, tab: WorkspaceTab) -> Uuid {
        if let Some(group) = self
            .groups
            .iter_mut()
            .find(|group| group.tabs.contains(&tab))
        {
            group.active = group
                .tabs
                .iter()
                .position(|candidate| candidate == &tab)
                .unwrap_or(0);
            self.focused = Some(group.id);
            return group.id;
        }
        let index = self
            .focused
            .and_then(|id| self.groups.iter().position(|group| group.id == id));
        let index = index.unwrap_or_else(|| {
            if self.groups.is_empty() {
                let id = Uuid::new_v4();
                self.groups.push(WorkspaceGroup {
                    id,
                    tabs: Vec::new(),
                    active: 0,
                });
                self.layout.root = Some(PaneNode::Leaf(id));
            }
            0
        });
        let group = &mut self.groups[index];
        group.tabs.push(tab);
        group.active = group.tabs.len() - 1;
        self.focused = Some(group.id);
        group.id
    }

    pub(crate) fn select(&mut self, id: Uuid, tab: &WorkspaceTab) -> bool {
        let Some(group) = self.groups.iter_mut().find(|group| group.id == id) else {
            return false;
        };
        let Some(index) = group.tabs.iter().position(|candidate| candidate == tab) else {
            return false;
        };
        group.active = index;
        self.focused = Some(id);
        true
    }

    pub(crate) fn active(&self) -> Option<&WorkspaceTab> {
        let group = self
            .groups
            .iter()
            .find(|group| Some(group.id) == self.focused)?;
        group.tabs.get(group.active)
    }

    pub(crate) fn visible(&self) -> Vec<WorkspaceTab> {
        self.layout
            .sessions()
            .into_iter()
            .filter_map(|id| {
                let group = self.groups.iter().find(|group| group.id == id)?;
                group.tabs.get(group.active).cloned()
            })
            .collect()
    }

    pub(crate) fn rects(&self, area: egui::Rect) -> PaneRects {
        self.layout.rects(area)
    }

    pub(crate) fn close(&mut self, tab: &WorkspaceTab) -> bool {
        let Some(index) = self
            .groups
            .iter()
            .position(|group| group.tabs.contains(tab))
        else {
            return false;
        };
        let group = &mut self.groups[index];
        let old_active = group.tabs.get(group.active).cloned();
        group.tabs.retain(|candidate| candidate != tab);
        group.active = old_active
            .and_then(|active| group.tabs.iter().position(|candidate| *candidate == active))
            .unwrap_or_else(|| group.active.min(group.tabs.len().saturating_sub(1)));
        self.settle(index);
        true
    }

    fn settle(&mut self, old_index: usize) {
        self.groups.retain(|group| !group.tabs.is_empty());
        self.layout
            .retain(|id| self.groups.iter().any(|group| group.id == id));
        if !self
            .groups
            .iter()
            .any(|group| Some(group.id) == self.focused)
        {
            self.focused = self
                .groups
                .get(old_index.min(self.groups.len().saturating_sub(1)))
                .map(|group| group.id);
        }
        // A singleton still needs a rect, unlike the legacy terminal-only
        // arrangement whose caller drew the selected session separately.
        if self.groups.len() == 1 && self.layout.single().is_none() {
            self.layout.root = Some(PaneNode::Leaf(self.groups[0].id));
        }
    }

    /// Move to a final tab index, preserving every other group's selection.
    pub(crate) fn insert_tab(&mut self, tab: &WorkspaceTab, target: Uuid, index: usize) -> bool {
        if !self.groups.iter().any(|group| group.id == target) {
            return false;
        }
        let Some(source) = self
            .groups
            .iter()
            .position(|group| group.tabs.contains(tab))
        else {
            return false;
        };
        if self.groups[source].id == target {
            let group = &mut self.groups[source];
            let Some(from) = group.tabs.iter().position(|candidate| candidate == tab) else {
                return false;
            };
            let value = group.tabs.remove(from);
            let index = index.min(group.tabs.len());
            group.tabs.insert(index, value);
            group.active = index;
            self.focused = Some(target);
            return from != index;
        }
        self.close(tab);
        let Some(group) = self.groups.iter_mut().find(|group| group.id == target) else {
            return false;
        };
        let index = index.min(group.tabs.len());
        group.tabs.insert(index, tab.clone());
        group.active = index;
        self.focused = Some(target);
        true
    }

    /// Check the complete resulting arrangement before accepting a drop.
    /// Refused drops leave focus, order and geometry untouched.
    pub(crate) fn move_tab(
        &mut self,
        tab: &WorkspaceTab,
        target: Uuid,
        drop: PaneDrop,
        area: egui::Rect,
    ) -> bool {
        let Some(source) = self.groups.iter().find(|group| group.tabs.contains(tab)) else {
            return false;
        };
        let Some(destination) = self.groups.iter().find(|group| group.id == target) else {
            return false;
        };
        if drop == PaneDrop::Center {
            let index = destination.tabs.len();
            return self.insert_tab(tab, target, index);
        }
        if source.id == target && source.tabs.len() == 1 {
            return false;
        }
        let mut after = self.clone();
        after.close(tab);
        let id = Uuid::new_v4();
        let Some(layout) = after.layout.placed(id, target, drop) else {
            return false;
        };
        if !layout.fits(area) {
            return false;
        }
        after.layout = layout;
        after.groups.push(WorkspaceGroup {
            id,
            tabs: vec![tab.clone()],
            active: 0,
        });
        after.focused = Some(id);
        *self = after;
        true
    }

    pub(crate) fn retain(&mut self, keep: impl Fn(&WorkspaceTab) -> bool) {
        for group in &mut self.groups {
            let old_active = group.tabs.get(group.active).cloned();
            group.tabs.retain(&keep);
            group.active = old_active
                .and_then(|active| group.tabs.iter().position(|candidate| *candidate == active))
                .unwrap_or_else(|| group.active.min(group.tabs.len().saturating_sub(1)));
        }
        self.settle(0);
    }
}

/// Where a session released at `pos` over the terminal in `rect` goes: the
/// nearer edge within `DOCK_DROP_BAND` of it, else the middle; outside the
/// terminal, nowhere.
pub(crate) fn pane_drop(pos: egui::Pos2, rect: egui::Rect) -> Option<PaneDrop> {
    if !rect.contains(pos) || rect.width() <= 0.0 || rect.height() <= 0.0 {
        return None;
    }
    Some(
        [
            (PaneDrop::Left, (pos.x - rect.left()) / rect.width()),
            (PaneDrop::Right, (rect.right() - pos.x) / rect.width()),
            (PaneDrop::Top, (pos.y - rect.top()) / rect.height()),
            (PaneDrop::Bottom, (rect.bottom() - pos.y) / rect.height()),
        ]
        .into_iter()
        .filter(|(_, share)| *share <= DOCK_DROP_BAND)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map_or(PaneDrop::Center, |(drop, _)| drop),
    )
}

/// The part of the terminal in `rect` a session dropped at `drop` takes.
pub(crate) fn pane_drop_zone(rect: egui::Rect, drop: PaneDrop) -> egui::Rect {
    let center = rect.center();
    match drop {
        PaneDrop::Left => rect.with_max_x(center.x),
        PaneDrop::Right => rect.with_min_x(center.x),
        PaneDrop::Top => rect.with_max_y(center.y),
        PaneDrop::Bottom => rect.with_min_y(center.y),
        PaneDrop::Center => rect,
    }
}

/// What the hint beside the pointer says over `drop`.
pub(crate) fn pane_drop_hint(drop: PaneDrop, refused: bool) -> &'static str {
    if refused {
        tr("これ以上分割できません")
    } else {
        drop.label()
    }
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
        palette: &Palette,
        edge: DockEdge,
        shown: InspectorTab,
        width: f32,
        height: f32,
    ) {
        let project_id = self
            .store
            .projects
            .iter()
            .find(|project| project.id == session.project_id)
            .map(|project| project.id);
        let is_worktree = self
            .store
            .projects
            .iter()
            .find(|project| project.id == session.project_id)
            .zip(session.worktree_path.as_ref())
            .is_some_and(|(project, path)| path != &project.path);
        // Files and changes are a project's; a session without one offers
        // only its conversation. Decided here rather than stored, so the tab
        // chosen for other sessions survives visiting this one.
        let offered = InspectorTab::ALL.map(|tab| {
            (self.dock.edge_of(DockPanel::View(tab)) == edge
                && (project_id.is_some() || tab == InspectorTab::Conversation))
                .then_some(tab)
        });
        let turns = self
            .session_prompt_turns
            .get(&session.id)
            .map_or(0, Vec::len);
        let changes = match project_id {
            Some(id) if !is_worktree => match self.git_changes_cache.get(&id) {
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
                        // Drawn as `icon_button` draws the fold button beside it.
                        let placement = egui::menu::menu_custom_button(
                            ui,
                            egui::Button::new(RichText::new(ICON_RESTORE).size(16.0))
                                .stroke(egui::Stroke::NONE)
                                .min_size(egui::vec2(CONTROL_HEIGHT, CONTROL_HEIGHT)),
                            |ui| {
                                for to in DockEdge::ALL {
                                    let mut after = self.dock;
                                    let moves = after.move_views(edge, to);
                                    if dock_menu_entry(ui, to, moves, self.dock_fits(&after)) {
                                        self.dock_views_to(edge, to);
                                        ui.close_menu();
                                    }
                                }
                            },
                        )
                        .response
                        .on_hover_text(tr("パネルの配置を変更"));
                        paint_button_focus(ui, &placement, palette);
                    });
                });
                ui.add_space(SPACE_XS);
                hairline(ui, palette);
                ui.add_space(SPACE_XS);
                let body_height = (height - 56.0).max(100.0);
                match (shown, project_id) {
                    (InspectorTab::Files, Some(project_id)) => self.ui_session_files_tab(
                        ui,
                        session,
                        project_id,
                        is_worktree,
                        palette,
                        body_height,
                    ),
                    (InspectorTab::Changes, Some(project_id)) => self.ui_session_changes_tab(
                        ui,
                        session,
                        project_id,
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
        grip.context_menu(|ui| {
            for to in DockEdge::ALL {
                let mut after = self.dock;
                let moves = after.move_panel(panel, to);
                if dock_menu_entry(ui, to, moves, self.dock_fits(&after)) {
                    self.dock_panel_to(panel, to);
                    ui.close_menu();
                }
            }
        });
    }

    /// Whether `after` may replace the layout: it keeps on screen every panel
    /// the workspace has room for. Before the workspace is first drawn there
    /// is nothing on screen to lose.
    pub(crate) fn dock_fits(&self, after: &DockLayout) -> bool {
        self.dock_area
            .is_none_or(|area| self.dock.keeps_on_screen(after, area))
    }

    /// Puts `after` in place and says `done`, or — when it would take a panel
    /// off the screen — leaves the layout as it is and says why. Every move
    /// passes through here.
    fn dock_commit(&mut self, after: DockLayout, edge: DockEdge, done: String) {
        if self.dock_fits(&after) {
            self.dock = after;
            self.notice_briefly(done);
        } else {
            self.notice_briefly(dock_no_room(edge));
        }
    }

    /// Sends `panel` to `edge` and says so; nothing when it is already there.
    pub(crate) fn dock_panel_to(&mut self, panel: DockPanel, edge: DockEdge) {
        let mut after = self.dock;
        if after.move_panel(panel, edge) {
            let done = tf!(
                "{panel}を{edge}に移動しました",
                panel = panel.label(),
                edge = edge.label()
            );
            self.dock_commit(after, edge, done);
        }
    }

    /// Sends every view on `from` to `to`, as the placement menu does.
    pub(crate) fn dock_views_to(&mut self, from: DockEdge, to: DockEdge) {
        let mut after = self.dock;
        if after.move_views(from, to) {
            let done = tf!("パネルを{edge}に移動しました", edge = to.label());
            self.dock_commit(after, to, done);
        }
    }

    /// The files tab: a filter, the files the terminal mentioned, the tree
    /// with its dot entries folded, and the scan limit in one line.
    pub(crate) fn ui_session_files_tab(
        &mut self,
        ui: &mut egui::Ui,
        session: &Session,
        project_id: Uuid,
        is_worktree: bool,
        palette: &Palette,
        height: f32,
    ) {
        let Some(root) = session.worktree_path.clone().or_else(|| {
            self.store
                .projects
                .iter()
                .find(|project| project.id == project_id)
                .map(|project| project.path.clone())
        }) else {
            return;
        };

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
                } else if let Some(project) = self
                    .store
                    .projects
                    .iter()
                    .find(|project| project.id == project_id)
                    .cloned()
                {
                    self.file_cache.remove(&project.id);
                    self.request_files(&project);
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
        } else if !self.file_cache.contains_key(&project_id) {
            if let Some(project) = self
                .store
                .projects
                .iter()
                .find(|project| project.id == project_id)
                .cloned()
            {
                self.request_files(&project);
            }
        }
        let is_scanning = if is_worktree {
            self.background_tasks
                .contains(&BackgroundKey::WorktreeFiles(root.clone()))
                && !self.session_file_cache.contains_key(&root)
        } else {
            self.background_tasks
                .contains(&BackgroundKey::Files(project_id))
                && !self.file_cache.contains_key(&project_id)
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
            self.file_cache.get(&project_id)
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

        let badges = self.git_status_badge_map(if is_worktree { Uuid::nil() } else { project_id });
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
                    project_id,
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
                            project_id,
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
                    if let Some(project) = self
                        .store
                        .projects
                        .iter()
                        .find(|project| project.id == project_id)
                        .cloned()
                    {
                        self.navigate_to_file(&project, root.clone(), path, None);
                    }
                }
                FileTreeAction::Toggle(path) => {
                    let key = (project_id, path);
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
        project_id: Uuid,
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
        if !self.git_changes_cache.contains_key(&project_id) {
            if let Some(project) = self
                .store
                .projects
                .iter()
                .find(|project| project.id == project_id)
                .cloned()
            {
                self.request_git_changes(&project);
            }
        }
        let mut open = None;
        match self.git_changes_cache.get(&project_id) {
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
            if let Some(project) = self
                .store
                .projects
                .iter()
                .find(|project| project.id == project_id)
                .cloned()
            {
                let root = project.path.clone();
                self.navigate_to_file(&project, root, path, None);
            }
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
