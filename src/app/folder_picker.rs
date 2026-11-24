//! The in-app folder picker (change 317): one sheet in which a project's
//! folder is typed as a path, walked to by clicking, or created when it does
//! not exist yet. The projects page and the launch sheet both open it, and
//! the macOS dialog stays one button away.
//!
//! Nothing here touches the disk in a frame. `list_folder` reads a folder on
//! a background thread and the sheet draws what came back; a folder is made
//! on a background thread too.
//!
//! A child of `src/app.rs`, like `launch_projects.rs`. The choosers for the
//! macOS dialog live here as well, so that file does not grow.

use crate::*;

/// How many rows of folders the sheet shows before it scrolls.
const FOLDER_PICKER_VISIBLE_ROWS: usize = 10;

/// Who opened the picker, and so what opening a folder does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FolderPickerTarget {
    /// The projects page, ⌘O, the palette: the folder is added and opened.
    Projects,
    /// The launch sheet: the folder becomes the sheet's project in place.
    LaunchSheet,
}

/// A typed path, read as the folder to list and the name being typed in it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FolderInput {
    pub(crate) dir: PathBuf,
    pub(crate) partial: String,
}

impl FolderInput {
    /// The path the text names: what 「開く」 opens.
    pub(crate) fn typed(&self) -> PathBuf {
        if self.partial.is_empty() {
            self.dir.clone()
        } else {
            self.dir.join(&self.partial)
        }
    }
}

/// Text ending in `/` lists that folder; otherwise the last component is a
/// name still being typed in its parent. Only `/…`, `~` and `~/…` are paths:
/// `dev/x` has no folder to be relative to, and `~name` is another person's
/// home, which nothing here resolves.
pub(crate) fn split_folder_input(text: &str, home: Option<&Path>) -> Option<FolderInput> {
    let full = if text == "~" {
        format!("{}/", home?.display())
    } else if let Some(rest) = text.strip_prefix("~/") {
        format!("{}/{rest}", home?.display())
    } else if text.starts_with('/') {
        text.to_owned()
    } else {
        return None;
    };
    let cut = full.rfind('/')? + 1;
    Some(FolderInput {
        dir: PathBuf::from(&full[..cut]),
        partial: full[cut..].to_owned(),
    })
}

/// `path` as the field spells a folder: `~` for the home folder, and a
/// trailing `/` so that it lists.
pub(crate) fn folder_input_text(path: &Path, home: Option<&Path>) -> String {
    let mut text = match home.and_then(|home| path.strip_prefix(home).ok()) {
        Some(rest) if rest.as_os_str().is_empty() => "~".to_owned(),
        Some(rest) => format!("~/{}", rest.display()),
        None => path.display().to_string(),
    };
    if !text.ends_with('/') {
        text.push('/');
    }
    text
}

/// Where the picker opens: beside the newest project, unless macOS guards
/// that folder — listing it would have macOS ask the person, in Operon's
/// name, about a folder they did not point at.
pub(crate) fn folder_picker_start(projects: &[Project], home: Option<&Path>) -> String {
    let beside = projects
        .iter()
        .max_by_key(|project| project.added_at)
        .and_then(|project| project.path.parent())
        .filter(|parent| !is_in_protected_folder(parent, home));
    match (beside, home) {
        (Some(parent), _) => folder_input_text(parent, home),
        (None, Some(_)) => "~/".to_owned(),
        (None, None) => "/".to_owned(),
    }
}

/// What the folder being listed turned out to be.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FolderDirState {
    Listed,
    Missing,
    Unreadable,
}

/// What the typed path turned out to be.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FolderTarget {
    Folder,
    Missing,
    NotAFolder,
    /// The disk would not say: a parent that is a file, or one that may not
    /// be looked into. Not offered for creation, which would fail the same way.
    Unreadable,
}

/// One folder read for one text. `input` is the text it was read for, so a
/// listing that arrives late can be told from the one the field wants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FolderListing {
    pub(crate) input: String,
    pub(crate) dir: PathBuf,
    pub(crate) partial: String,
    pub(crate) typed: PathBuf,
    pub(crate) names: Vec<String>,
    pub(crate) dir_state: FolderDirState,
    pub(crate) truncated: bool,
    pub(crate) target: FolderTarget,
}

/// Reads the folder `text` points into: its child folders whose names start
/// with the partial, ignoring case. At most `limit` entries are read, counted
/// on the open handle, and a listing that stopped there says so. Whether the
/// typed path is itself a folder is asked separately, so one past the ceiling
/// still opens. Runs on a background thread.
pub(crate) fn list_folder(text: &str, home: Option<&Path>, limit: usize) -> Option<FolderListing> {
    let input = split_folder_input(text, home)?;
    let typed = input.typed();
    let target = match fs::metadata(&typed) {
        Ok(metadata) if metadata.is_dir() => FolderTarget::Folder,
        Ok(_) => FolderTarget::NotAFolder,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => FolderTarget::Missing,
        Err(_) => FolderTarget::Unreadable,
    };
    let needle = input.partial.to_lowercase();
    let mut names = Vec::new();
    let mut truncated = false;
    let dir_state = match fs::read_dir(&input.dir) {
        Ok(entries) => {
            for (read, entry) in entries.enumerate() {
                if read == limit {
                    truncated = true;
                    break;
                }
                let Ok(entry) = entry else {
                    continue;
                };
                let name = entry.file_name().to_string_lossy().into_owned();
                if name.starts_with('.') && !needle.starts_with('.') {
                    continue;
                }
                if !name.to_lowercase().starts_with(&needle) {
                    continue;
                }
                let is_folder = entry.file_type().is_ok_and(|kind| {
                    kind.is_dir()
                        || (kind.is_symlink()
                            && fs::metadata(entry.path()).is_ok_and(|target| target.is_dir()))
                });
                if is_folder {
                    names.push(name);
                }
            }
            names.sort_by_cached_key(|name| name.to_lowercase());
            FolderDirState::Listed
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => FolderDirState::Missing,
        Err(_) => FolderDirState::Unreadable,
    };
    Some(FolderListing {
        input: text.to_owned(),
        dir: input.dir,
        partial: input.partial,
        typed,
        names,
        dir_state,
        truncated,
        target,
    })
}

/// What Tab makes of the partial: the one name left, closed with `/`, or as
/// much as every matching name shares. Nothing when that adds nothing.
pub(crate) fn folder_completion(names: &[String], partial: &str) -> Option<String> {
    let first = names.first()?;
    if names.len() == 1 {
        return Some(format!("{first}/"));
    }
    let shared = names[1..]
        .iter()
        .map(|name| {
            first
                .chars()
                .zip(name.chars())
                .take_while(|(a, b)| a.to_lowercase().eq(b.to_lowercase()))
                .count()
        })
        .min()
        .unwrap_or(0);
    (shared > partial.chars().count()).then(|| first.chars().take(shared).collect())
}

/// The sheet's state. Everything the frame draws is here already: `rows`,
/// `location`, `note` and `question` are built when the text changes or a
/// listing arrives, never while drawing.
pub(crate) struct FolderPicker {
    pub(crate) target: FolderPickerTarget,
    pub(crate) input: String,
    pub(crate) home: Option<PathBuf>,
    /// The listing for `input`, or nothing while it is being read.
    pub(crate) listing: Option<FolderListing>,
    pub(crate) rows: Vec<String>,
    /// The folder being listed, as the location row shows it.
    pub(crate) location: String,
    /// The text is not a path: no listing is asked for it.
    pub(crate) refused: bool,
    pub(crate) at_root: bool,
    /// Shown under a listing that stopped at the ceiling.
    pub(crate) note: Option<String>,
    /// The confirmation's question, while a missing folder waits for it.
    pub(crate) question: Option<String>,
    /// The folder is being made; the sheet waits for the answer.
    pub(crate) creating: bool,
    /// A conversion is open in the field, so Enter, Esc and Tab are the IME's.
    composing: bool,
    focus_field: bool,
    caret_to_end: bool,
}

impl FolderPicker {
    fn target(&self) -> Option<FolderTarget> {
        self.listing.as_ref().map(|listing| listing.target)
    }

    /// The line under the field: what is wrong with the text, or that 「開く」
    /// will make a folder.
    fn status_line(&self) -> Option<&'static str> {
        if self.refused {
            return Some(tr("/ または ~ で始まるパスを入力してください。"));
        }
        match self.target()? {
            FolderTarget::Folder => None,
            FolderTarget::Missing => Some(tr("このフォルダはまだありません。作成して開けます。")),
            FolderTarget::NotAFolder => Some(tr("これはフォルダではありません。")),
            FolderTarget::Unreadable => Some(tr("このパスは確認できません。")),
        }
    }

    /// What the list says when it has no row.
    fn empty_line(&self) -> Option<&'static str> {
        if self.refused {
            return None;
        }
        let Some(listing) = &self.listing else {
            return Some(tr("読み込み中…"));
        };
        Some(match listing.dir_state {
            FolderDirState::Missing => tr("この場所はまだありません。"),
            FolderDirState::Unreadable => tr("このフォルダは読み取れません。"),
            FolderDirState::Listed if listing.partial.is_empty() => tr("フォルダはありません。"),
            FolderDirState::Listed => tr("一致するフォルダはありません。"),
        })
    }
}

/// What the person did in the sheet this frame, acted on once it has drawn.
enum FolderPickerAct {
    Cancel,
    Submit,
    Withdraw,
    Complete,
    Parent,
    Descend(usize),
    Finder,
}

fn picker_home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

impl OperonApp {
    /// Opens the picker, unless a progress modal owns the window. The palette
    /// that asked for it closes first.
    pub(crate) fn open_folder_picker(&mut self, target: FolderPickerTarget) {
        let launching = self
            .launch_progress
            .as_ref()
            .is_some_and(|progress| !progress.dismissed);
        if launching || self.restore_progress.is_some() || self.folder_picker.is_some() {
            return;
        }
        if self.command_palette_open {
            self.close_command_palette();
        }
        let home = picker_home();
        let input = folder_picker_start(&self.store.projects, home.as_deref());
        self.folder_picker = Some(FolderPicker {
            target,
            input,
            home,
            listing: None,
            rows: Vec::new(),
            location: String::new(),
            refused: false,
            at_root: false,
            note: None,
            question: None,
            creating: false,
            composing: false,
            focus_field: true,
            caret_to_end: true,
        });
        self.folder_picker_input_changed();
    }

    /// Puts `text` in the field, as a click, Tab or a drop does.
    pub(crate) fn set_folder_picker_input(&mut self, text: String) {
        let Some(picker) = self.folder_picker.as_mut() else {
            return;
        };
        picker.input = text;
        picker.caret_to_end = true;
        // A click on a row took the keyboard; typing goes on in the field.
        picker.focus_field = true;
        self.folder_picker_input_changed();
    }

    /// The field's text is new: what was known about the old text goes, a
    /// pending confirmation is withdrawn, and the new text's folder is asked.
    pub(crate) fn folder_picker_input_changed(&mut self) {
        let Some(picker) = self.folder_picker.as_mut() else {
            return;
        };
        picker.listing = None;
        picker.rows.clear();
        picker.note = None;
        picker.question = None;
        match split_folder_input(&picker.input, picker.home.as_deref()) {
            Some(input) => {
                picker.refused = false;
                picker.at_root = input.dir.parent().is_none();
                // Rebuilt from its components, which drops the text's trailing `/`.
                picker.location = friendly_path(&input.dir.components().collect::<PathBuf>());
            }
            None => {
                picker.refused = true;
                picker.at_root = true;
                picker.location.clear();
            }
        }
        self.request_folder_listing();
    }

    /// Asks for the listing of the field's text. One listing is read at a
    /// time; a text that changed meanwhile is asked when that one is back.
    pub(crate) fn request_folder_listing(&mut self) {
        let Some(picker) = self.folder_picker.as_ref() else {
            return;
        };
        if picker.refused || picker.listing.is_some() {
            return;
        }
        let text = picker.input.clone();
        let home = picker.home.clone();
        self.spawn_background(BackgroundKey::FolderListing, move || {
            BackgroundResult::FolderListing(list_folder(
                &text,
                home.as_deref(),
                FOLDER_PICKER_ENTRY_LIMIT,
            ))
        });
    }

    /// A listing is back. One for text that is no longer in the field is
    /// dropped, and the field's text is asked instead.
    pub(crate) fn finish_folder_listing(&mut self, listing: Option<FolderListing>) {
        self.background_tasks.remove(&BackgroundKey::FolderListing);
        let Some(picker) = self.folder_picker.as_mut() else {
            return;
        };
        match listing {
            Some(listing) if listing.input == picker.input => {
                picker.rows = listing
                    .names
                    .iter()
                    .map(|name| format!("{ICON_FOLDER_CLOSED}  {name}"))
                    .collect();
                picker.note = listing.truncated.then(|| {
                    tf!(
                        "項目が多いため、先頭の {count} 件だけを読みました。パスを最後まで入力すれば開けます。",
                        count = FOLDER_PICKER_ENTRY_LIMIT
                    )
                });
                picker.listing = Some(listing);
            }
            _ => self.request_folder_listing(),
        }
    }

    /// 「開く」, 「フォルダを作成…」 and Enter. An existing folder opens; a
    /// missing one is asked about first; anything else does nothing.
    pub(crate) fn submit_folder_picker(&mut self) {
        let Some(picker) = self.folder_picker.as_mut() else {
            return;
        };
        if picker.creating {
            return;
        }
        if picker.question.is_some() {
            self.confirm_folder_creation();
            return;
        }
        let Some(listing) = &picker.listing else {
            return;
        };
        match listing.target {
            FolderTarget::Folder => {
                let path = listing.typed.clone();
                self.open_folder_from_picker(path);
            }
            FolderTarget::Missing => {
                picker.question = Some(tf!(
                    "「{path}」を作成しますか？",
                    path = friendly_path(&listing.typed)
                ));
            }
            FolderTarget::NotAFolder | FolderTarget::Unreadable => {}
        }
    }

    /// 「作成して開く」: the folder and its missing parents are made on a
    /// background thread, and the sheet waits.
    pub(crate) fn confirm_folder_creation(&mut self) {
        let Some(picker) = self.folder_picker.as_mut() else {
            return;
        };
        let missing = picker
            .listing
            .as_ref()
            .filter(|listing| listing.target == FolderTarget::Missing);
        let (Some(listing), Some(_), false) = (missing, &picker.question, picker.creating) else {
            return;
        };
        let path = listing.typed.clone();
        picker.creating = true;
        self.spawn_background(BackgroundKey::FolderCreate, move || {
            let error = fs::create_dir_all(&path)
                .err()
                .map(|error| error.to_string());
            BackgroundResult::FolderCreated { path, error }
        });
    }

    /// The folder was made, or could not be. Made, it opens as any folder
    /// does; otherwise the sheet stays with its footer back.
    pub(crate) fn finish_folder_creation(&mut self, path: PathBuf, error: Option<String>) {
        self.background_tasks.remove(&BackgroundKey::FolderCreate);
        let Some(picker) = self.folder_picker.as_mut() else {
            return;
        };
        picker.creating = false;
        match error {
            None => self.open_folder_from_picker(path),
            Some(error) => {
                picker.question = None;
                self.notice = Some(tf!(
                    "「{path}」を作成できませんでした: {error}",
                    path = friendly_path(&path),
                    error = error
                ));
            }
        }
    }

    /// Closes the picker and hands `path` to whoever opened it. What they say
    /// about it — added, already there, not a folder — is their notice.
    fn open_folder_from_picker(&mut self, path: PathBuf) {
        let Some(picker) = self.folder_picker.take() else {
            return;
        };
        match picker.target {
            FolderPickerTarget::Projects => {
                self.add_project_path(path);
            }
            FolderPickerTarget::LaunchSheet => self.apply_launch_project_folder(Some(path)),
        }
    }

    /// 「キャンセル」, ✕ and Esc. Says nothing and changes nothing; a folder
    /// being made is waited for.
    pub(crate) fn close_folder_picker(&mut self) {
        if self
            .folder_picker
            .as_ref()
            .is_some_and(|picker| !picker.creating)
        {
            self.folder_picker = None;
        }
    }

    /// Tab: the partial grows to what the listed names share.
    pub(crate) fn folder_picker_complete(&mut self) {
        let Some(picker) = self.folder_picker.as_ref() else {
            return;
        };
        let Some(listing) = &picker.listing else {
            return;
        };
        // A listing cut at the ceiling does not know every match.
        if listing.truncated {
            return;
        }
        let Some(completed) = folder_completion(&listing.names, &listing.partial) else {
            return;
        };
        let Some(stem) = picker.input.strip_suffix(listing.partial.as_str()) else {
            return;
        };
        // A bare `~` lists the home folder, and a name goes under it.
        let slash = if stem.ends_with('/') { "" } else { "/" };
        self.set_folder_picker_input(format!("{stem}{slash}{completed}"));
    }

    /// A row was clicked: that folder becomes the text, and is listed.
    pub(crate) fn folder_picker_descend(&mut self, row: usize) {
        let Some(picker) = self.folder_picker.as_ref() else {
            return;
        };
        let Some(listing) = &picker.listing else {
            return;
        };
        let Some(name) = listing.names.get(row) else {
            return;
        };
        let text = folder_input_text(&listing.dir.join(name), picker.home.as_deref());
        self.set_folder_picker_input(text);
    }

    /// 「上のフォルダへ」: the folder above the one being listed.
    pub(crate) fn folder_picker_parent(&mut self) {
        let Some(picker) = self.folder_picker.as_ref() else {
            return;
        };
        let home = picker.home.as_deref();
        let Some(input) = split_folder_input(&picker.input, home) else {
            return;
        };
        let Some(parent) = input.dir.parent() else {
            return;
        };
        let text = folder_input_text(parent, home);
        self.set_folder_picker_input(text);
    }

    /// 「Finder で選ぶ…」: the macOS dialog the entry opened before change
    /// 317, with the picker out of its way.
    pub(crate) fn choose_folder_in_finder(&mut self) {
        let Some(picker) = self.folder_picker.take() else {
            return;
        };
        #[cfg(not(test))]
        {
            let selection = rfd::FileDialog::new()
                .set_title(tr("プロジェクトフォルダを選択"))
                .pick_folder();
            match picker.target {
                FolderPickerTarget::Projects => self.apply_project_folder_choice(selection),
                FolderPickerTarget::LaunchSheet => self.apply_launch_project_folder(selection),
            }
        }
        #[cfg(test)]
        {
            self.notice = Some(format!(
                "choose_folder_in_finder_invoked {:?}",
                picker.target
            ));
        }
    }

    /// A folder dropped while the picker shows is typed for the person, not
    /// opened: they still press 「開く」.
    pub(crate) fn fill_folder_picker_from_drop(&mut self, dropped: &[egui::DroppedFileHandle]) {
        let Some(picker) = self.folder_picker.as_ref() else {
            return;
        };
        if picker.creating {
            return;
        }
        let (folders, _) = dropped_project_folders(dropped);
        if let Some(folder) = folders.first() {
            let text = folder_input_text(folder, picker.home.as_deref());
            self.set_folder_picker_input(text);
        }
    }

    pub(crate) fn add_project(&mut self) {
        let path = PathBuf::from(self.project_path_input.trim());
        self.add_project_path(path);
    }

    /// 「フォルダを選択…」, ⌘O and the palette's project action.
    pub(crate) fn choose_project_folder(&mut self) {
        self.open_folder_picker(FolderPickerTarget::Projects);
    }

    pub(crate) fn apply_project_folder_choice(&mut self, selection: Option<PathBuf>) {
        if let Some(path) = selection {
            self.add_project_path(path);
        } else {
            self.notice = Some(tr("プロジェクトの選択をキャンセルしました。").into());
        }
    }

    pub(crate) fn choose_workspace_folder(&mut self) {
        let selection = rfd::FileDialog::new()
            .set_title(tr("スキャンするワークスペースを選択"))
            .pick_folder();
        self.apply_workspace_folder_choice(selection);
    }

    pub(crate) fn apply_workspace_folder_choice(&mut self, selection: Option<PathBuf>) {
        if let Some(path) = selection {
            self.workspace_scan_input = path.display().to_string();
            self.import_workspace();
        } else {
            self.notice = Some(tr("ワークスペースの選択をキャンセルしました。").into());
        }
    }

    /// The sheet. Reads `FolderPicker` and nothing on disk.
    pub(crate) fn ui_folder_picker(&mut self, ctx: &egui::Context) {
        let Some(mut picker) = self.folder_picker.take() else {
            return;
        };
        let palette = self.store.theme.palette();
        let screen = ctx.content_rect();
        egui::Area::new(egui::Id::unique("folder-picker-scrim"))
            .order(egui::Order::Middle)
            .fixed_pos(screen.min)
            .show(ctx, |ui| {
                // Takes the click and does nothing: a stray click must not
                // throw away a path somebody was typing.
                let (rect, _) = ui.allocate_exact_size(screen.size(), egui::Sense::click());
                ui.painter().rect_filled(rect, 0.0, palette.scrim);
            });
        // Read before the field draws, so that none of the three reaches it:
        // Tab completes rather than leaving the field, Enter opens. While a
        // conversion is open they are the IME's, and the Enter that confirms
        // one is not a request to open (lesson 016).
        let (step, composing) = ctx.input(|input| find_bar_step(&input.events, picker.composing));
        let conversion = picker.composing
            || composing
            || ctx.input(|input| {
                input
                    .events
                    .iter()
                    .any(|event| matches!(event, egui::Event::Ime(_)))
            });
        picker.composing = composing;
        let (escape, tab) = ctx.input_mut(|input| {
            // Taken from the field either way: a single-line field gives up
            // the keyboard on Enter.
            input.consume_key(egui::Modifiers::NONE, egui::Key::Enter);
            if conversion {
                return (false, false);
            }
            (
                input.consume_key(egui::Modifiers::NONE, egui::Key::Escape),
                input.consume_key(egui::Modifiers::NONE, egui::Key::Tab),
            )
        });
        let enter = step == Some(true);
        if conversion && ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
            // egui takes the keyboard from a field on any Esc; it asks for it back.
            picker.focus_field = true;
        }
        let asking = picker.question.is_some();
        let mut act = None;
        if !picker.creating {
            if escape {
                act = Some(if asking {
                    FolderPickerAct::Withdraw
                } else {
                    FolderPickerAct::Cancel
                });
            } else if enter {
                act = Some(FolderPickerAct::Submit);
            } else if tab {
                act = Some(FolderPickerAct::Complete);
            }
        }
        let field_id = egui::Id::unique("folder-picker-path");
        if picker.caret_to_end {
            picker.caret_to_end = false;
            let mut state = egui::TextEdit::load_state(ctx, field_id).unwrap_or_default();
            let end = egui::text::CCursor::new(picker.input.chars().count());
            state
                .cursor
                .set_char_range(Some(egui::text::CCursorRange::one(end)));
            egui::TextEdit::store_state(ctx, field_id, state);
        }
        let mut edited = false;
        const WIDTH: f32 = 640.0;
        egui::Area::new(egui::Id::unique("folder-picker"))
            .order(egui::Order::Foreground)
            .fixed_pos(egui::pos2(
                screen.center().x - WIDTH / 2.0 - SPACE_LG,
                screen.top() + screen.height() * 0.1,
            ))
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(palette.raised)
                    .stroke(egui::Stroke::new(1.0_f32, palette.border))
                    .corner_radius(egui::CornerRadius::same(RADIUS_WINDOW))
                    .inner_margin(egui::Margin::same(SPACE_LG as i8))
                    .shadow(ui.style().visuals.popup_shadow)
                    .show(ui, |ui| {
                        ui.set_width(WIDTH);
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(tr("プロジェクトフォルダを開く"))
                                    .size(16.0)
                                    .strong()
                                    .color(palette.text_strong),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if small_icon_button(ui, palette, ICON_CLOSE, tr("閉じる"))
                                        .clicked()
                                    {
                                        act = Some(FolderPickerAct::Cancel);
                                    }
                                },
                            );
                        });
                        ui.add_space(SPACE_SM);
                        let field = ui.add_enabled(
                            !picker.creating,
                            egui::TextEdit::singleline(&mut picker.input)
                                .id(field_id)
                                .hint_text("~/path/to/project")
                                .desired_width(f32::INFINITY)
                                .lock_focus(true),
                        );
                        if picker.focus_field {
                            picker.focus_field = false;
                            field.request_focus();
                        }
                        edited = field.changed();
                        if let Some(line) = picker.status_line() {
                            ui.label(RichText::new(line).color(palette.text_muted));
                        }
                        ui.add_space(SPACE_XS);
                        ui.horizontal(|ui| {
                            ui.add_enabled_ui(!picker.at_root && !picker.creating, |ui| {
                                if small_icon_button(
                                    ui,
                                    palette,
                                    ICON_PARENT_FOLDER,
                                    tr("上のフォルダへ"),
                                )
                                .clicked()
                                {
                                    act = Some(FolderPickerAct::Parent);
                                }
                            });
                            ui.add(
                                egui::Label::new(
                                    RichText::new(&picker.location).color(palette.text_muted),
                                )
                                .truncate(),
                            );
                        });
                        let row = ui.spacing().interact_size.y;
                        card_frame(palette).show(ui, |ui| {
                            ui.add_enabled_ui(!picker.creating, |ui| {
                                egui::ScrollArea::vertical()
                                    .id_salt("folder-picker-rows")
                                    .auto_shrink([false, false])
                                    .max_height(row * FOLDER_PICKER_VISIBLE_ROWS as f32)
                                    .show_rows(ui, row, picker.rows.len(), |ui, visible| {
                                        ui.style_mut().wrap_mode =
                                            Some(egui::TextWrapMode::Truncate);
                                        if let (true, Some(line)) =
                                            (picker.rows.is_empty(), picker.empty_line())
                                        {
                                            ui.label(RichText::new(line).color(palette.text_muted));
                                        }
                                        for index in visible {
                                            if ui
                                                .selectable_label(false, &picker.rows[index])
                                                .clicked()
                                            {
                                                act = Some(FolderPickerAct::Descend(index));
                                            }
                                        }
                                    });
                            });
                        });
                        if let Some(note) = &picker.note {
                            ui.label(RichText::new(note).small().color(palette.text_muted));
                        }
                        ui.add_space(SPACE_SM);
                        if picker.creating {
                            ui.label(RichText::new(tr("作成中…")).color(palette.text_muted));
                        } else if let Some(question) = &picker.question {
                            ui.label(RichText::new(question).color(palette.text_strong));
                            ui.label(
                                RichText::new(tr("途中のフォルダも、まだなければ作成します。"))
                                    .color(palette.text_muted),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if primary_button(ui, palette, ICON_ADD, tr("作成して開く"))
                                        .clicked()
                                    {
                                        act = Some(FolderPickerAct::Submit);
                                    }
                                    if secondary_button(ui, tr("やめる")).clicked() {
                                        act = Some(FolderPickerAct::Withdraw);
                                    }
                                },
                            );
                        } else {
                            let target = picker.target();
                            ui.horizontal(|ui| {
                                if quiet_button(ui, palette, tr("Finder で選ぶ…")).clicked() {
                                    act = Some(FolderPickerAct::Finder);
                                }
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        let (icon, label) = if target == Some(FolderTarget::Missing)
                                        {
                                            (ICON_ADD, tr("フォルダを作成…"))
                                        } else {
                                            (ICON_FOLDER_OPEN, tr("開く"))
                                        };
                                        let ready = matches!(
                                            target,
                                            Some(FolderTarget::Folder | FolderTarget::Missing)
                                        );
                                        ui.add_enabled_ui(ready, |ui| {
                                            if primary_button(ui, palette, icon, label).clicked() {
                                                act = Some(FolderPickerAct::Submit);
                                            }
                                        });
                                        if secondary_button(ui, tr("キャンセル")).clicked() {
                                            act = Some(FolderPickerAct::Cancel);
                                        }
                                    },
                                );
                            });
                        }
                    });
            });
        self.folder_picker = Some(picker);
        if edited {
            self.folder_picker_input_changed();
        }
        match act {
            Some(FolderPickerAct::Cancel) => self.close_folder_picker(),
            Some(FolderPickerAct::Submit) => self.submit_folder_picker(),
            Some(FolderPickerAct::Withdraw) => {
                if let Some(picker) = self.folder_picker.as_mut() {
                    picker.question = None;
                }
            }
            Some(FolderPickerAct::Complete) => self.folder_picker_complete(),
            Some(FolderPickerAct::Parent) => self.folder_picker_parent(),
            Some(FolderPickerAct::Descend(row)) => self.folder_picker_descend(row),
            Some(FolderPickerAct::Finder) => self.choose_folder_in_finder(),
            None => {}
        }
    }
}
