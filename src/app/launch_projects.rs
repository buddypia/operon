//! The launch sheet's project (change 294): the list it is picked from —
//! recent first, narrowed by name — and adding a folder as a project from
//! inside the sheet, by the folder picker or a drop, without leaving the page.
//! With no project at all the sheet asks for a folder instead of closing.
//!
//! A child of `src/app.rs`, like `screens.rs`, so that neither of them grows
//! past the module-size band.

use crate::*;

/// From this many projects the sheet's list shows a filter field.
pub(crate) const LAUNCH_PROJECT_FILTER_MIN: usize = 8;

/// A folder that is a project after `register_project_folder`.
pub(crate) enum RegisteredProject {
    New(Uuid),
    Existing(Uuid),
}

impl RegisteredProject {
    fn id(&self) -> Uuid {
        match self {
            RegisteredProject::New(id) | RegisteredProject::Existing(id) => *id,
        }
    }
}

/// What the person did in the sheet's project list this frame, acted on by
/// `apply_launch_project_pick` once the sheet has drawn.
pub(crate) enum LaunchProjectPick {
    Switch(Uuid),
    NewFolder,
}

/// The sheet's list, in the order it is shown: projects whose latest session
/// started most recently first, then the never-used ones by when they were
/// added, newest first. `filter` keeps names containing it, ignoring case.
pub(crate) fn launch_project_choices(
    projects: &[Project],
    sessions: &[Session],
    filter: &str,
) -> Vec<(Uuid, String)> {
    let mut last_used: HashMap<Uuid, u64> = HashMap::new();
    for session in sessions {
        let started = session.launched_at.unwrap_or(session.created_at);
        let entry = last_used.entry(session.project_id).or_insert(started);
        *entry = (*entry).max(started);
    }
    let needle = filter.trim().to_lowercase();
    let mut choices: Vec<(Option<u64>, u64, Uuid, String)> = projects
        .iter()
        .filter(|project| needle.is_empty() || project.name.to_lowercase().contains(&needle))
        .map(|project| {
            (
                last_used.get(&project.id).copied(),
                project.added_at,
                project.id,
                project.name.clone(),
            )
        })
        .collect();
    // `Some` sorts above `None`, so used projects lead; a stable sort keeps
    // the store's order between equals.
    choices.sort_by_key(|choice| std::cmp::Reverse((choice.0, choice.1)));
    choices
        .into_iter()
        .map(|(_, _, id, name)| (id, name))
        .collect()
}

impl OperonApp {
    /// Makes `path` a project, or finds the project it already is. Navigates
    /// nowhere: the projects page and the launch sheet each decide that.
    pub(crate) fn register_project_folder(
        &mut self,
        path: PathBuf,
    ) -> Result<RegisteredProject, ProjectAddition> {
        if !path.is_dir() {
            self.notice = Some(tr("既存のフォルダを選んでください。").into());
            return Err(ProjectAddition::Declined);
        }
        let path = path.canonicalize().unwrap_or(path);
        if let Some(existing) = self
            .store
            .projects
            .iter()
            .find(|project| project.path == path)
        {
            return Ok(RegisteredProject::Existing(existing.id));
        }
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Project")
            .to_owned();
        let project = Project {
            id: Uuid::new_v4(),
            path,
            name,
            added_at: now(),
        };
        let previous_store = self.store.clone();
        let project_id = project.id;
        self.store.projects.push(project);
        if !self.persist() {
            self.store = previous_store;
            return Err(ProjectAddition::NotSaved);
        }
        Ok(RegisteredProject::New(project_id))
    }

    /// The sheet's 「新しいプロジェクト…」 and 「フォルダを選択…」: the folder
    /// picker, called after the sheet has drawn, never inside it.
    pub(crate) fn choose_launch_project_folder(&mut self) {
        #[cfg(not(test))]
        {
            let selection = rfd::FileDialog::new()
                .set_title(tr("プロジェクトフォルダを選択"))
                .pick_folder();
            self.apply_launch_project_folder(selection);
        }
        #[cfg(test)]
        {
            self.notice = Some("choose_launch_project_folder_invoked".into());
        }
    }

    /// A cancelled picker changes nothing and says nothing.
    pub(crate) fn apply_launch_project_folder(&mut self, selection: Option<PathBuf>) {
        let Some(path) = selection else {
            return;
        };
        let Ok(registered) = self.register_project_folder(path) else {
            return;
        };
        self.point_launch_sheet_at(registered.id());
        self.notice_briefly(match registered {
            RegisteredProject::New(_) => tr("プロジェクトを追加しました。"),
            RegisteredProject::Existing(_) => tr("登録済みのプロジェクトに切り替えました。"),
        });
    }

    /// Whether the sheet is on screen: open, and not waiting under a progress
    /// modal, which owns the window — its Esc, its ⌘↩ and its drops — while
    /// it shows.
    pub(crate) fn launch_sheet_shows(&self) -> bool {
        self.launch_sheet_open
            && self.launch_progress.as_ref().is_none_or(|p| p.dismissed)
            && self.restore_progress.is_none()
    }

    /// Dropped folders: the sheet's while it shows, the projects page's
    /// otherwise.
    pub(crate) fn route_dropped_folders(&mut self, dropped: Vec<egui::DroppedFileHandle>) {
        if !self.launch_sheet_shows() {
            self.add_dropped_folders(dropped);
            return;
        }
        let (folders, _) = dropped_project_folders(&dropped);
        if folders.is_empty() {
            self.notice = Some(tr("Finder からフォルダをドロップすると追加できます。").into());
            return;
        }
        if folders.len() == 1 {
            self.apply_launch_project_folder(folders.into_iter().next());
            return;
        }
        let (mut first, mut added, mut failed) = (None, 0, false);
        for path in folders {
            match self.register_project_folder(path) {
                Ok(registered) => {
                    added += usize::from(matches!(registered, RegisteredProject::New(_)));
                    first.get_or_insert(registered.id());
                }
                Err(_) => failed = true,
            }
        }
        let Some(first) = first else {
            return;
        };
        self.point_launch_sheet_at(first);
        if failed {
            // The failure's notice stays: it is the one to read.
            return;
        }
        if added == 0 {
            self.notice_briefly(tr("登録済みのプロジェクトに切り替えました。"));
            return;
        }
        let name = self
            .selected_project()
            .map(|project| project.name.clone())
            .unwrap_or_default();
        self.notice_briefly(tf!(
            "{count} 件のフォルダを追加し、{name} を選びました。",
            count = added,
            name = name
        ));
    }

    /// The sheet takes `project_id`: switched to when it already shows a
    /// project, opened on it when it was asking for a folder.
    /// Pointing it at the project it shows keeps the form as it is.
    fn point_launch_sheet_at(&mut self, project_id: Uuid) {
        if self.selected_project == Some(project_id) {
            return;
        }
        if self.selected_project().is_some() {
            self.switch_launch_project(project_id);
        } else {
            self.open_project_session_setup(project_id);
        }
    }

    /// The sheet's project picker. The request and the name were typed for
    /// this launch, not for the old project, and the session behind the sheet
    /// is not the sheet's to close: both survive the switch. Everything tied
    /// to the old project's folder — the 作業場所, the dependency, the danger
    /// acknowledgement — `select_project` still clears.
    pub(crate) fn switch_launch_project(&mut self, project_id: Uuid) {
        let goal = std::mem::take(&mut self.goal_input);
        let name = std::mem::take(&mut self.session_name_input);
        let session = self.selected_session;
        let library_open = self.session_library_open;
        self.select_project(Some(project_id));
        self.goal_input = goal;
        self.session_name_input = name;
        self.selected_session = session;
        self.session_library_open = library_open;
        self.launch_project_filter.clear();
        self.launch_sheet_is_git = self
            .selected_project()
            .is_some_and(|project| has_git_metadata(&project.path));
        if let Some(project) = self.selected_project().cloned() {
            if !self.file_cache.contains_key(&project.id) {
                self.request_files(&project);
            }
        }
    }

    /// The combo in the sheet's title row, and what was picked in it.
    pub(crate) fn ui_launch_sheet_project_picker(
        &mut self,
        ui: &mut egui::Ui,
        project: &Project,
    ) -> Option<LaunchProjectPick> {
        let mut pick = None;
        let mut list_open = false;
        egui::ComboBox::from_id_salt("launch-sheet-project")
            .selected_text(&project.name)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
            .show_ui(ui, |ui| {
                list_open = true;
                if self.store.projects.len() >= LAUNCH_PROJECT_FILTER_MIN {
                    let field = ui.add(
                        egui::TextEdit::singleline(&mut self.launch_project_filter)
                            .hint_text(tr("プロジェクトを絞り込み"))
                            .desired_width(f32::INFINITY),
                    );
                    if ui.memory(|memory| memory.focused().is_none()) {
                        field.request_focus();
                    }
                }
                // Built only while the list is open.
                let choices = launch_project_choices(
                    &self.store.projects,
                    &self.store.sessions,
                    &self.launch_project_filter,
                );
                if choices.is_empty() {
                    ui.label(RichText::new(tr("一致するプロジェクトはありません。")).weak());
                }
                for (id, name) in &choices {
                    if ui.selectable_label(*id == project.id, name).clicked() {
                        pick = Some(LaunchProjectPick::Switch(*id));
                    }
                }
                ui.separator();
                let new_project = tf!("{icon}  新しいプロジェクト…", icon = ICON_ADD);
                if ui.selectable_label(false, new_project).clicked() {
                    pick = Some(LaunchProjectPick::NewFolder);
                }
                if pick.is_some() {
                    egui::Popup::close_all(ui);
                }
            });
        // A list closed without a choice does not reopen narrowed.
        if !list_open && !self.launch_project_filter.is_empty() {
            self.launch_project_filter.clear();
        }
        pick
    }

    pub(crate) fn apply_launch_project_pick(&mut self, pick: LaunchProjectPick) {
        match pick {
            LaunchProjectPick::Switch(id) => self.switch_launch_project(id),
            LaunchProjectPick::NewFolder => self.choose_launch_project_folder(),
        }
    }

    /// The sheet's body when there is no project yet. True when 「フォルダを
    /// 選択…」 was pressed.
    pub(crate) fn ui_launch_sheet_without_project(
        &self,
        ui: &mut egui::Ui,
        palette: &Palette,
    ) -> bool {
        ui.add_space(SPACE_SM);
        ui.label(RichText::new(tr("まだプロジェクトがありません。")).color(palette.text_strong));
        ui.label(tr("セッションを始めるフォルダを選んでください。"));
        ui.add_space(SPACE_SM);
        let chosen = primary_button(ui, palette, ICON_FOLDER_OPEN, tr("フォルダを選択…")).clicked();
        ui.label(
            RichText::new(tr(
                "Finder からフォルダをこのウィンドウにドロップしても追加できます。",
            ))
            .small()
            .color(palette.text_muted),
        );
        chosen
    }
}
