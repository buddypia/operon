use crate::*;
use std::hash::{Hash, Hasher};

/// What a click in the project tree asked for.
///
/// Collected rather than acted on, because walking the tree borrows both the
/// cached tree and the set of open directories out of the app, and either
/// change would invalidate the walk that is still running.
pub(crate) enum FileTreeAction {
    Open(PathBuf),
    Toggle(PathBuf),
}

struct CachedTreeEntry {
    name: String,
    path: PathBuf,
    is_dir: bool,
    open: bool,
    depth: usize,
    icon: &'static str,
    badge: Option<String>,
}

/// Cached flattened visible rows for the project file tree so steady frames
/// avoid re-traversing expanded directories and rebuilding per-row keys.
#[derive(Default)]
pub(crate) struct FileTreeRowCache {
    fingerprint: Option<u64>,
    rows: Vec<CachedTreeEntry>,
    builds: usize,
    hits: usize,
}

impl FileTreeRowCache {
    #[cfg(test)]
    pub(crate) fn builds(&self) -> usize {
        self.builds
    }

    #[cfg(test)]
    pub(crate) fn hits(&self) -> usize {
        self.hits
    }

    fn push_nodes(
        nodes: &[FileTreeNode],
        expanded: &HashSet<&Path>,
        badges: &HashMap<String, String>,
        force_open: bool,
        depth: usize,
        out: &mut Vec<CachedTreeEntry>,
    ) {
        for node in nodes {
            let is_dir = node.is_directory();
            let open = is_dir && (force_open || expanded.contains(node.path.as_path()));
            let icon = if is_dir {
                if open {
                    ICON_DISCLOSURE_OPEN
                } else {
                    ICON_DISCLOSURE_CLOSED
                }
            } else {
                file_tree_icon(&node.name)
            };
            let badge = badges.get(node.path.to_string_lossy().as_ref()).cloned();
            out.push(CachedTreeEntry {
                name: node.name.clone(),
                path: node.path.clone(),
                is_dir,
                open,
                depth,
                icon,
                badge,
            });
            if is_dir && open {
                Self::push_nodes(&node.children, expanded, badges, force_open, depth + 1, out);
            }
        }
    }

    fn hash_tree_nodes(nodes: &[FileTreeNode], h: &mut impl Hasher) {
        nodes.len().hash(h);
        for node in nodes {
            node.name.hash(h);
            node.path.hash(h);
            Self::hash_tree_nodes(&node.children, h);
        }
    }

    fn ensure(
        &mut self,
        nodes: &[FileTreeNode],
        project: Uuid,
        expanded: &HashSet<(Uuid, PathBuf)>,
        badges: &HashMap<String, String>,
        force_open: bool,
    ) {
        let mut h = std::hash::DefaultHasher::new();
        project.hash(&mut h);
        force_open.hash(&mut h);
        Self::hash_tree_nodes(nodes, &mut h);
        let mut expanded_paths: Vec<&Path> = expanded
            .iter()
            .filter_map(|(pid, path)| (*pid == project).then_some(path.as_path()))
            .collect();
        expanded_paths.sort();
        expanded_paths.hash(&mut h);
        let mut sorted_badges: Vec<(&str, &str)> = badges
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        sorted_badges.sort_unstable();
        sorted_badges.hash(&mut h);
        let fp = h.finish();
        if self.fingerprint == Some(fp) {
            self.hits += 1;
            return;
        }
        let expanded_set: HashSet<&Path> = expanded_paths.into_iter().collect();
        self.rows.clear();
        Self::push_nodes(nodes, &expanded_set, badges, force_open, 0, &mut self.rows);
        self.fingerprint = Some(fp);
        self.builds += 1;
    }
}

/// Draw the project tree using `cache` on steady frames when provided, or
/// falling back to direct zero-clone traversal when uncached.
#[allow(clippy::too_many_arguments)]
pub(crate) fn file_tree_rows(
    ui: &mut egui::Ui,
    nodes: &[FileTreeNode],
    project: Uuid,
    expanded: &HashSet<(Uuid, PathBuf)>,
    badges: &HashMap<String, String>,
    active: Option<&DocumentId>,
    force_open: bool,
    depth: usize,
    palette: &Palette,
    actions: &mut Vec<FileTreeAction>,
) {
    let expanded_paths: HashSet<&Path> = expanded
        .iter()
        .filter_map(|(pid, path)| (*pid == project).then_some(path.as_path()))
        .collect();
    file_tree_rows_borrowed(
        ui,
        nodes,
        project,
        &expanded_paths,
        badges,
        active,
        force_open,
        depth,
        palette,
        actions,
    );
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn file_tree_rows_cached(
    ui: &mut egui::Ui,
    nodes: &[FileTreeNode],
    project: Uuid,
    expanded: &HashSet<(Uuid, PathBuf)>,
    badges: &HashMap<String, String>,
    active: Option<&DocumentId>,
    force_open: bool,
    palette: &Palette,
    actions: &mut Vec<FileTreeAction>,
    cache: Option<&mut FileTreeRowCache>,
) {
    let Some(cache) = cache else {
        file_tree_rows(
            ui, nodes, project, expanded, badges, active, force_open, 0, palette, actions,
        );
        return;
    };
    cache.ensure(nodes, project, expanded, badges, force_open);
    let row_height = CONTROL_HEIGHT_SMALL;
    for entry in &cache.rows {
        let row_rect = egui::Rect::from_min_size(
            ui.cursor().min,
            egui::vec2(ui.available_width(), row_height),
        );
        if ui.is_rect_visible(row_rect) {
            paint_file_tree_entry(
                ui,
                entry.is_dir,
                entry.open,
                entry.depth,
                &entry.name,
                &entry.path,
                entry.icon,
                entry.badge.as_deref(),
                project,
                active,
                palette,
                actions,
            );
        } else {
            ui.allocate_exact_size(
                egui::vec2(ui.available_width(), row_height),
                egui::Sense::hover(),
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn file_tree_rows_borrowed(
    ui: &mut egui::Ui,
    nodes: &[FileTreeNode],
    project: Uuid,
    expanded: &HashSet<&Path>,
    badges: &HashMap<String, String>,
    active: Option<&DocumentId>,
    force_open: bool,
    depth: usize,
    palette: &Palette,
    actions: &mut Vec<FileTreeAction>,
) {
    let row_height = CONTROL_HEIGHT_SMALL;
    for node in nodes {
        let is_dir = node.is_directory();
        let open = is_dir && (force_open || expanded.contains(node.path.as_path()));
        let row_rect = egui::Rect::from_min_size(
            ui.cursor().min,
            egui::vec2(ui.available_width(), row_height),
        );
        if ui.is_rect_visible(row_rect) {
            let icon = if is_dir {
                if open {
                    ICON_DISCLOSURE_OPEN
                } else {
                    ICON_DISCLOSURE_CLOSED
                }
            } else {
                file_tree_icon(&node.name)
            };
            let badge = badges
                .get(node.path.to_string_lossy().as_ref())
                .map(String::as_str);
            paint_file_tree_entry(
                ui, is_dir, open, depth, &node.name, &node.path, icon, badge, project, active,
                palette, actions,
            );
        } else {
            ui.allocate_exact_size(
                egui::vec2(ui.available_width(), row_height),
                egui::Sense::hover(),
            );
        }

        if is_dir && open {
            file_tree_rows_borrowed(
                ui,
                &node.children,
                project,
                expanded,
                badges,
                active,
                force_open,
                depth + 1,
                palette,
                actions,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn paint_file_tree_entry(
    ui: &mut egui::Ui,
    is_dir: bool,
    open: bool,
    depth: usize,
    name: &str,
    path: &Path,
    icon: &str,
    badge: Option<&str>,
    project: Uuid,
    active: Option<&DocumentId>,
    palette: &Palette,
    actions: &mut Vec<FileTreeAction>,
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        ui.add_space(4.0 + depth as f32 * 13.0);
        let mut icon_clicked = false;
        if is_dir {
            let chevron_res = ui.add(
                egui::Label::new(RichText::new(icon).size(11.5).color(palette.text_faint))
                    .sense(egui::Sense::click()),
            );
            let folder_res = ui.add(
                egui::Label::new(
                    RichText::new(if open {
                        ICON_FOLDER_OPEN
                    } else {
                        ICON_FOLDER_CLOSED
                    })
                    .size(13.5)
                    .color(palette.text_muted),
                )
                .sense(egui::Sense::click()),
            );
            if chevron_res.clicked() || folder_res.clicked() {
                icon_clicked = true;
            }
        } else {
            // A file's row lines up with the name of a directory beside it
            // rather than with its caret, so the column of names stays a
            // column.
            ui.add_space(13.0);
            let file_res = ui.add(
                egui::Label::new(RichText::new(icon).size(13.5).color(palette.text_muted))
                    .sense(egui::Sense::click()),
            );
            if file_res.clicked() {
                icon_clicked = true;
            }
        }
        let selected = !is_dir
            && active.is_some_and(|active| active.project == project && active.path == path);
        let response = ui.selectable_label(
            selected,
            RichText::new(name).size(12.5).color(if selected {
                palette.text_strong
            } else {
                palette.text
            }),
        );
        if response.clicked() || icon_clicked {
            actions.push(if is_dir {
                FileTreeAction::Toggle(path.to_path_buf())
            } else {
                FileTreeAction::Open(path.to_path_buf())
            });
        }
        // The badge is the whole reason the editor knows about git: it is
        // how a person spots, without opening anything, which files the
        // agent that just finished actually touched.
        if let Some(status) = badge {
            let (mark, colour) = git_status_badge(status, palette);
            ui.label(RichText::new(mark).size(11.5).color(colour))
                .on_hover_text(git_status_wording(status));
        }
    });
}

/// The one letter beside a changed file, and the ink it is drawn in. Letters
/// rather than glyphs: `M`, `A`, `D` are what git itself prints and what every
/// tool that shows this has taught people to read.
pub(crate) fn git_status_badge(status: &str, palette: &Palette) -> (&'static str, Color32) {
    match status {
        "??" | "A" | "AM" => ("A", palette.diff_added),
        "D" | "AD" | " D" => ("D", palette.diff_removed),
        status if status.contains('R') => ("R", palette.info),
        status if status.contains('U') => ("U", palette.warning),
        _ => ("M", palette.accent_text),
    }
}

pub(crate) fn git_status_wording(status: &str) -> &'static str {
    match status {
        "??" => tr("追跡されていない新しいファイルです"),
        "A" | "AM" => tr("追加されたファイルです"),
        "D" | "AD" | " D" => tr("削除されたファイルです"),
        status if status.contains('R') => tr("名前が変更されたファイルです"),
        status if status.contains('U') => tr("コンフリクトしているファイルです"),
        _ => tr("変更されたファイルです"),
    }
}
