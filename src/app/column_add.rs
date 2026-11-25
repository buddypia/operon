//! What the session page's column can add (change 319): a new session, or a
//! project, from the + beside the column's heading, a right-click on the
//! column's empty space, or a right-click on a project's heading. A project
//! added here stays on the session page, unfolded in the column and chosen as
//! the workspace's project, rather than taking the window to the projects
//! page as the other entries do.
//!
//! A child of `src/app.rs`, like `launch_menu.rs`, so that neither `app.rs`
//! nor `screens.rs` grows past its size band.

use crate::*;

/// One thing the column offers to add.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ColumnAdd {
    /// 「新しいセッション…」: the 新しいセッション sheet, as ⌘N opens it.
    Session,
    /// 「プロジェクトを追加…」: the folder picker, for the column.
    Project,
}

impl ColumnAdd {
    pub(crate) fn label(self) -> &'static str {
        match self {
            ColumnAdd::Session => tr("新しいセッション…"),
            ColumnAdd::Project => tr("プロジェクトを追加…"),
        }
    }
}

/// The popup under the column heading's +, so that an Escape closing it is
/// not passed on to a terminal (`take_launch_menu_escape`).
pub(crate) fn column_add_menu_id() -> egui::Id {
    egui::Id::unique("column-add-menu")
}

/// What the + and the empty space offer, top to bottom.
pub(crate) const COLUMN_ADD_CHOICES: [ColumnAdd; 2] = [ColumnAdd::Session, ColumnAdd::Project];

impl OperonApp {
    /// Acts on a choice from any of the column's menus.
    pub(crate) fn apply_column_add(&mut self, choice: ColumnAdd) {
        match choice {
            ColumnAdd::Session => self.open_new_session(),
            ColumnAdd::Project => self.open_folder_picker(FolderPickerTarget::SessionColumn),
        }
    }

    /// The + popup's and the empty-space menu's rows. Returns the choice; the
    /// caller acts on it after the menu is drawn, so the act sees it closed.
    pub(crate) fn ui_column_add_items(ui: &mut egui::Ui, palette: &Palette) -> Option<ColumnAdd> {
        let mut chosen = None;
        for choice in COLUMN_ADD_CHOICES {
            if quiet_button(ui, palette, choice.label()).clicked() {
                chosen = Some(choice);
                ui.close();
            }
        }
        chosen
    }

    /// 「プロジェクトを追加…」 alone, for the heading menu.
    pub(crate) fn ui_column_add_project_item(ui: &mut egui::Ui, palette: &Palette) -> bool {
        let clicked = quiet_button(ui, palette, ColumnAdd::Project.label()).clicked();
        if clicked {
            ui.close();
        }
        clicked
    }

    /// The folder the picker opened for the column (or `None` when the macOS
    /// dialog was cancelled): registered as anywhere else, then kept on the
    /// session page, unfolded and chosen. A folder already registered is
    /// unfolded and chosen the same way.
    pub(crate) fn apply_column_project_folder(&mut self, selection: Option<PathBuf>) {
        let Some(path) = selection else {
            self.notice = Some(tr("プロジェクトの選択をキャンセルしました。").into());
            return;
        };
        let (project_id, notice) = match self.register_project_folder(path) {
            Ok(RegisteredProject::New(id)) => (id, tr("プロジェクトを追加しました。")),
            Ok(RegisteredProject::Existing(id)) => {
                (id, tr("そのプロジェクトは追加済みのため、開きました。"))
            }
            // register_project_folder has said why.
            Err(_) => return,
        };
        self.page = Page::Sessions;
        self.select_workspace_project(project_id);
        self.notice = Some(notice.into());
        if !self.session_arrangement.unfolded.contains(&project_id) {
            // Says so itself when the order file cannot be written.
            self.toggle_project_fold(project_id);
        }
    }
}
