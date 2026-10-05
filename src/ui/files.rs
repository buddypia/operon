use crate::prelude::*;
use crate::*;

/// What a click in the project tree asked for.
///
/// Collected rather than acted on, because walking the tree borrows both the
/// cached tree and the set of open directories out of the app, and either
/// change would invalidate the walk that is still running.
pub(crate) enum FileTreeAction {
    Open(PathBuf),
    Toggle(PathBuf),
}

/// Draw one level of the project tree and recurse into the directories a person
/// has opened.
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
    for node in nodes {
        let open = force_open || expanded.contains(&(project, node.path.clone()));
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            ui.add_space(4.0 + depth as f32 * 13.0);
            let mut icon_clicked = false;
            if node.is_directory() {
                let chevron_res = ui.add(
                    egui::Label::new(
                        RichText::new(if open {
                            ICON_DISCLOSURE_OPEN
                        } else {
                            ICON_DISCLOSURE_CLOSED
                        })
                        .size(11.0)
                        .color(palette.text_faint),
                    )
                    .sense(egui::Sense::click()),
                );
                let folder_res = ui.add(
                    egui::Label::new(
                        RichText::new(if open {
                            ICON_FOLDER_OPEN
                        } else {
                            ICON_FOLDER_CLOSED
                        })
                        .size(13.0)
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
                    egui::Label::new(
                        RichText::new(file_tree_icon(&node.name))
                            .size(13.0)
                            .color(palette.text_muted),
                    )
                    .sense(egui::Sense::click()),
                );
                if file_res.clicked() {
                    icon_clicked = true;
                }
            }
            let selected = !node.is_directory()
                && active
                    .is_some_and(|active| active.project == project && active.path == node.path);
            let response = ui.selectable_label(
                selected,
                RichText::new(&node.name).size(12.5).color(if selected {
                    palette.text_strong
                } else {
                    palette.text
                }),
            );
            if response.clicked() || icon_clicked {
                actions.push(if node.is_directory() {
                    FileTreeAction::Toggle(node.path.clone())
                } else {
                    FileTreeAction::Open(node.path.clone())
                });
            }
            // The badge is the whole reason the editor knows about git: it is
            // how a person spots, without opening anything, which files the
            // agent that just finished actually touched.
            if let Some(status) = badges.get(node.path.to_string_lossy().as_ref()) {
                let (mark, colour) = git_status_badge(status, palette);
                ui.label(RichText::new(mark).size(11.0).color(colour))
                    .on_hover_text(git_status_wording(status));
            }
        });
        if node.is_directory() && open {
            file_tree_rows(
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
