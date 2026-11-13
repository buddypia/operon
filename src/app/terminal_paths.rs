//! Opening a path named in terminal output (change 293): which file a bare name
//! means, and the right-click menu that offers where to open it.
//!
//! ⌘- and ⌥-click open a path inside Operon; everywhere else a file can go is
//! one item in this menu, so a gesture has one meaning and the rest are named.

use crate::*;

/// What a row of the terminal path menu does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TerminalPathMenuChoice {
    OpenInOperon,
    OpenWithDefaultApp,
    RevealInFinder,
    CopyPath,
}

/// The menu's rows, in the order they are drawn. Drawing and tests read this
/// one list, so the two cannot disagree about what the menu offers.
pub(crate) const TERMINAL_PATH_MENU: [TerminalPathMenuChoice; 4] = [
    TerminalPathMenuChoice::OpenInOperon,
    TerminalPathMenuChoice::OpenWithDefaultApp,
    TerminalPathMenuChoice::RevealInFinder,
    TerminalPathMenuChoice::CopyPath,
];

impl TerminalPathMenuChoice {
    /// The row's words, in the active language.
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::OpenInOperon => tr("Operon で開く"),
            Self::OpenWithDefaultApp => tr("既定のアプリで開く"),
            Self::RevealInFinder => tr("Finder で表示"),
            Self::CopyPath => tr("パスをコピー"),
        }
    }
}

/// Whether the menu goes away this frame: a row was chosen, Escape was
/// pressed, or the primary button went down somewhere other than the menu.
pub(crate) fn terminal_path_menu_closes(
    chosen: bool,
    escape_pressed: bool,
    pressed_outside: bool,
) -> bool {
    chosen || escape_pressed || pressed_outside
}

/// The file a name printed in a terminal means: the usual resolution against
/// the worktree and the project, and then — for a name with no folder in it,
/// which is how agents tend to report what they wrote — the one file of that
/// name in the scan of the session's folder. Several files of that name mean
/// nothing: underlining the wrong one is worse than underlining none.
pub(crate) fn resolve_terminal_path_with_scan(
    worktree: Option<&Path>,
    project_root: &Path,
    scan: Option<&FileScanSnapshot>,
    path: &str,
) -> Option<PathBuf> {
    if let Some(resolved) = resolve_terminal_session_path(worktree, project_root, path) {
        return Some(resolved);
    }
    if path.contains('/') {
        return None;
    }
    let relative = unique_scanned_file(scan?, path)?;
    let root = worktree.unwrap_or(project_root);
    // The scan may be older than the file system; only a file still there
    // counts. Canonical, like every other resolution, so one file is one tab.
    root.join(relative)
        .canonicalize()
        .ok()
        .filter(|candidate| candidate.is_file())
}

impl OperonApp {
    /// The scan of the folder `session` works in, when one has been read: the
    /// worktree's for a worktree session, the project's otherwise — the same
    /// one the files tab beside the terminal shows.
    pub(crate) fn terminal_session_scan(
        &self,
        project_id: Uuid,
        project_root: &Path,
        worktree: Option<&Path>,
    ) -> Option<&FileScanSnapshot> {
        match worktree.filter(|worktree| *worktree != project_root) {
            Some(worktree) => self.session_file_cache.get(worktree),
            None => self.file_cache.get(&project_id),
        }
    }

    /// Ask for the scan a bare name needs, when it is not there yet. Called off
    /// the draw path, from the probe drain; `spawn_background` drops a second
    /// request for a scan already running.
    pub(crate) fn request_terminal_session_scan(
        &mut self,
        project: &Project,
        worktree: Option<&Path>,
    ) {
        match worktree.filter(|worktree| *worktree != project.path.as_path()) {
            Some(worktree) => {
                if !self.session_file_cache.contains_key(worktree) {
                    self.request_session_files(worktree);
                }
            }
            None => {
                if !self.file_cache.contains_key(&project.id) {
                    self.request_files(project);
                }
            }
        }
    }

    /// A scan landed: every name that missed may now mean a file, so the
    /// misses are forgotten and the next hover asks again.
    pub(crate) fn forget_terminal_path_misses(&mut self) {
        let before = self.resolved_paths.len();
        self.resolved_paths.retain(|_, resolved| resolved.is_some());
        for cache in self.terminal_path_caches.values_mut() {
            cache.retain(|_, resolved| resolved.is_some());
        }
        self.negative_path_probed_at.clear();
        if self.resolved_paths.len() != before {
            self.resolved_paths_generation += 1;
        }
    }

    /// A primary click on an openable target. ⌘ and ⌥ both open it inside
    /// Operon; a plain click opens nothing, because the pane owns typing.
    pub(crate) fn open_clicked_terminal_target(
        &mut self,
        modifiers: &egui::Modifiers,
        kind: TerminalTargetKind,
    ) {
        if super::screens::is_terminal_open_modifier(modifiers) {
            self.open_terminal_target(kind);
        }
    }

    /// A secondary click on a target: a path that resolves gets the menu,
    /// placed at `pointer`. A URL gets nothing new.
    pub(crate) fn open_terminal_path_menu(
        &mut self,
        session_id: Uuid,
        kind: &TerminalTargetKind,
        pointer: Option<egui::Pos2>,
    ) {
        let TerminalTargetKind::Path { path, .. } = kind else {
            return;
        };
        if let Some(resolved) = self.resolve_session_terminal_file(path) {
            self.terminal_path_menu = Some((session_id, resolved));
            self.terminal_path_menu_at = pointer;
        }
    }

    /// Do what a row of the path menu says with `path`.
    pub(crate) fn apply_terminal_path_menu_choice(
        &mut self,
        ctx: &egui::Context,
        choice: TerminalPathMenuChoice,
        path: PathBuf,
    ) {
        match choice {
            TerminalPathMenuChoice::OpenInOperon => {
                self.open_terminal_target(TerminalTargetKind::Path {
                    path: path.display().to_string(),
                    line: None,
                });
            }
            TerminalPathMenuChoice::OpenWithDefaultApp => {
                self.open_terminal_target_external(TerminalTargetKind::Path {
                    path: path.display().to_string(),
                    line: None,
                });
            }
            TerminalPathMenuChoice::RevealInFinder => {
                self.request_system_action(tr("Finder で表示"), move || reveal_path(&path));
            }
            TerminalPathMenuChoice::CopyPath => {
                ctx.copy_text(path.display().to_string());
                self.notice = Some(tr("パスをコピーしました。").into());
            }
        }
    }

    /// The menu a right-click on a path opens, at the pointer.
    pub(crate) fn ui_terminal_path_menu(&mut self, ui: &mut egui::Ui, session_id: Uuid) {
        let Some((owner, path)) = self.terminal_path_menu.clone() else {
            return;
        };
        if owner != session_id {
            return;
        }
        let muted = self.store.theme.palette().text_muted;
        let mut chosen = None;
        let mut window = egui::Window::new(tr("パスの操作"))
            .collapsible(false)
            .resizable(false)
            .title_bar(false);
        if let Some(at) = self.terminal_path_menu_at {
            window = window.fixed_pos(at);
        }
        let shown = window.show(ui.ctx(), |ui| {
            ui.label(
                RichText::new(path.display().to_string())
                    .size(12.0)
                    .color(muted),
            );
            for choice in TERMINAL_PATH_MENU {
                if ui.button(choice.label()).clicked() {
                    chosen = Some(choice);
                }
            }
        });
        let menu_rect = shown.map(|shown| shown.response.rect);
        let (escape_pressed, pressed_outside) = ui.input(|input| {
            let pressed_outside = input.pointer.primary_pressed()
                && input
                    .pointer
                    .interact_pos()
                    .is_some_and(|at| menu_rect.is_some_and(|rect| !rect.contains(at)));
            (input.key_pressed(egui::Key::Escape), pressed_outside)
        });
        if let Some(choice) = chosen {
            self.apply_terminal_path_menu_choice(ui.ctx(), choice, path);
        }
        if terminal_path_menu_closes(chosen.is_some(), escape_pressed, pressed_outside) {
            self.terminal_path_menu = None;
            self.terminal_path_menu_at = None;
        }
    }
}
