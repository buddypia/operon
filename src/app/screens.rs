//! One contiguous run lifted out of `src/app.rs`, from the toolbar through the
//! terminal workspace: the pages, the session lists, the editor, settings, the
//! terminal, and the handful of helpers only that run calls — the palette's
//! ranking and key handling among them, which decide rather than draw but have
//! no caller outside this file.
//!
//! It is a line range, not a rule. Three drawing methods stayed in
//! `src/app.rs` — `ui_restore_modal`, `ui_setup_prompt`, and
//! `ui_diff_comment_footer` — because they sit in impl blocks with the state
//! they read, and moving them would have made this something other than a pure
//! relocation. Say "this run moved" rather than "drawing lives here": the
//! second is a rule a reader would follow to the wrong file.
//!
//! Why at all: `largest_module_lines` had been recorded as breached in three
//! commits in a row, at 8862, 9078, and 9269 against a warn of 8800, and three
//! records in a row is where a record stops being information.
//!
//! A child of `src/app.rs` rather than a module of its own, because a new
//! top-level module is a line in `src/main.rs`, which `docs/sdlc/risk.yaml`
//! holds at `paused`.
//!
//! Nothing here was rewritten when it moved. Every line is byte-identical to
//! the line it replaced.

use crate::*;

impl OperonApp {
    /// The toolbar, which on macOS is also the window's title bar.
    ///
    /// Left to right: the mark, the segmented navigation, and — pushed to the
    /// far right — search, the one filled button on the screen, and settings.
    /// Nothing else. A toolbar is the one strip a person's eye returns to
    /// between tasks, so anything in it that is not a way somewhere else is
    /// noise; the sentence about where metadata lives moved to Settings, where
    /// somebody asking that question would go looking for it.
    pub(crate) fn ui_topbar(&mut self, ctx: &egui::Context) {
        let palette = self.store.theme.palette();
        egui::TopBottomPanel::top("topbar")
            .resizable(false)
            .exact_height(TOOLBAR_HEIGHT)
            .show_separator_line(false)
            .frame(
                egui::Frame::new()
                    .fill(palette.raised)
                    .inner_margin(egui::Margin::symmetric(SPACE_MD as i8, 0)),
            )
            .show(ctx, |ui| {
                let panel_bounds = ui.clip_rect();
                let draggable_rect = egui::Rect::from_min_max(
                    egui::pos2(panel_bounds.min.x + TRAFFIC_LIGHT_INSET, panel_bounds.min.y),
                    panel_bounds.max,
                );
                let title_bar_response = ui.interact(
                    draggable_rect,
                    ui.id().with("topbar_drag"),
                    egui::Sense::click_and_drag(),
                );

                // Zero-allocation stack buffer for topbar control exclusion bounds.
                // 16 entries comfortably cover the traffic-lights inset, all navigation tabs,
                // brand mark, action buttons, and rate limit indicators without heap allocations.
                let mut control_rects = [egui::Rect::NOTHING; 16];
                let mut control_count = 0;
                let mut register_control = |rect: egui::Rect| {
                    if control_count < control_rects.len() {
                        control_rects[control_count] = rect;
                        control_count += 1;
                    }
                };

                // Exclude macOS window buttons (traffic lights inset).
                register_control(egui::Rect::from_min_max(
                    panel_bounds.min,
                    egui::pos2(panel_bounds.min.x + TRAFFIC_LIGHT_INSET, panel_bounds.max.y),
                ));

                ui.horizontal_centered(|ui| {
                    ui.spacing_mut().item_spacing.x = SPACE_XS;
                    // The three window buttons live at the top left of this
                    // strip now, so the toolbar's own content starts after them.
                    ui.add_space(TRAFFIC_LIGHT_INSET - SPACE_MD);
                    let wordmark = RichText::new("Operon")
                        .size(14.0)
                        .strong()
                        .color(palette.accent_text);
                    // Mark and word are one control, so there is one hover
                    // target and one click target rather than two widgets that
                    // happen to sit together. A build whose bundled mark will
                    // not decode still gets a button, with the word alone.
                    let brand = match brand_mark_image(ui.ctx(), 18.0) {
                        Some(mark) => egui::Button::image_and_text(mark, wordmark),
                        None => egui::Button::new(wordmark),
                    };
                    let brand_res = ui.add(
                        brand
                            .stroke(egui::Stroke::NONE)
                            .min_size(egui::vec2(0.0, CONTROL_HEIGHT)),
                    );
                    paint_button_focus(ui, &brand_res, palette);
                    register_control(brand_res.rect);
                    if brand_res.on_hover_text(tr("ホーム")).clicked() {
                        self.page = Page::Home;
                    }
                    ui.add_space(SPACE_MD);
                    for (page, icon, label) in [
                        (Page::Home, ICON_TREE_MAIN, tr("ホーム")),
                        (Page::Projects, ICON_PROJECT, tr("プロジェクト")),
                        (Page::Sessions, ICON_SHOW, tr("セッション")),
                    ] {
                        let tab_res = nav_tab(ui, palette, icon, label, self.page == page);
                        register_control(tab_res.rect);
                        if tab_res.clicked() {
                            // Clicking the tab you are already on returns to
                            // that tab's root, which is what a nav tab does
                            // everywhere else — the breadcrumb above the
                            // project detail is the other way back.
                            if page == Page::Projects {
                                self.select_project(None);
                            }
                            self.page = page;
                        }
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let settings_res = icon_button(ui, palette, ICON_SETTINGS, tr("設定"));
                        register_control(settings_res.rect);
                        if settings_res.on_hover_text(tr("設定")).clicked() {
                            self.page = Page::Settings;
                        }
                        // One "+" in the window, and it starts a session: the
                        // project picker it used to be is on the Projects
                        // page, ⌘O, and a drop. The chord is read from the
                        // keymap, so a rebound key is the key it shows.
                        let new_session_label = match self.keymap.chord_for("session.new") {
                            Some(chord) => {
                                tf!("新しいセッション  {chord}", chord = chord_label(chord))
                            }
                            None => tr("新しいセッション").to_owned(),
                        };
                        let new_session_res =
                            primary_button(ui, palette, ICON_ADD, &new_session_label);
                        register_control(new_session_res.rect);
                        if new_session_res.clicked() {
                            self.open_new_session();
                        }
                        let search_label = match self.keymap.chord_for("palette.open") {
                            Some(chord) => tf!("検索・操作  {chord}", chord = chord_label(chord)),
                            None => tr("検索・操作").to_owned(),
                        };
                        let search_res = icon_text_button(ui, ICON_SEARCH, &search_label);
                        register_control(search_res.rect);
                        if search_res.clicked() {
                            self.open_command_palette();
                        }
                        // Left of the three controls in this right-to-left row:
                        // a fact rather than a way somewhere, so it sits before
                        // the things that are.
                        ui.add_space(SPACE_SM);
                        if let Some(rate_limits_rect) = self.ui_rate_limits(ui, palette) {
                            register_control(rate_limits_rect);
                        }
                    });
                });

                // Synchronize OS-reported viewport maximize state in a single query.
                if let Some(maximized) = ctx.input(|i| i.viewport().maximized) {
                    self.window_maximized = maximized;
                }

                // Defensive hit testing: coordinates must lie strictly inside the draggable
                // toolbar region, must not collide with any child control, and must fail closed.
                let is_over_control = |pos: egui::Pos2| -> bool {
                    !draggable_rect.contains(pos)
                        || control_rects[..control_count]
                            .iter()
                            .any(|rect| rect.contains(pos))
                };

                if title_bar_response.double_clicked() {
                    let pos =
                        ctx.input(|i| i.pointer.interact_pos().or_else(|| i.pointer.latest_pos()));
                    if let Some(pos) = pos {
                        if !is_over_control(pos) {
                            let new_state = !self.window_maximized;
                            self.window_maximized = new_state;
                            ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(new_state));
                        }
                    }
                } else if title_bar_response.drag_started_by(egui::PointerButton::Primary) {
                    let origin = ctx.input(|i| {
                        i.pointer
                            .press_origin()
                            .or_else(|| i.pointer.interact_pos())
                    });
                    if let Some(origin) = origin {
                        if !is_over_control(origin) {
                            ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                        }
                    }
                }
            });
        // The toolbar and the page below it are two surfaces of nearly the same
        // value, so the hairline is what says where one ends. There are no
        // shadows in the content area.
        egui::TopBottomPanel::top("topbar-rule")
            .exact_height(1.0)
            .show_separator_line(false)
            .frame(egui::Frame::new().fill(palette.border_subtle))
            .show(ctx, |_| {});
    }

    /// The fixed actions: the key the dispatch matches, the label a person reads
    /// and searches, and the keymap id whose chord is drawn beside it.
    ///
    /// The four rows that have a chord take their label from the keymap
    /// registry rather than writing it a second time, and none of them has a
    /// shortcut typed into the label any more. That was a place the application
    /// told a person which key to press and was under no obligation to be
    /// right; the chord is now read from the map the frame loop listens with.
    pub(crate) fn palette_actions() -> [(&'static str, &'static str, &'static str); 8] {
        [
            (
                "New session",
                tr(action_label("session.new")),
                "session.new",
            ),
            ("Open history", tr("履歴を開く"), ""),
            (
                "Open Projects",
                tr(action_label("page.projects")),
                "page.projects",
            ),
            (
                "Open Sessions",
                tr(action_label("page.sessions")),
                "page.sessions",
            ),
            (
                "Open Settings",
                tr(action_label("page.settings")),
                "page.settings",
            ),
            (
                "Add a project",
                tr(action_label("project.add")),
                "project.add",
            ),
            ("Scan a workspace", tr("ワークスペースをスキャン…"), ""),
            ("Open the keymap file", tr("キー割り当てファイルを開く"), ""),
        ]
    }

    /// Every row the palette should draw for the query it currently holds.
    ///
    /// With nothing typed this is the actions and nothing else: a palette
    /// that opens onto every session in the store is a list, and the point of
    /// the field is that it is not one.
    pub(crate) fn palette_entries(&self) -> Vec<PaletteEntry> {
        let query = self.command_search.trim().to_lowercase();
        let mut entries: Vec<(PaletteEntry, u64)> = Vec::new();
        for (action, label, keymap_id) in Self::palette_actions() {
            let score = if query.is_empty() {
                Some(PaletteScore::Name)
            } else {
                palette_rank(label, label, &query)
            };
            if let Some(score) = score {
                entries.push((
                    PaletteEntry {
                        kind: PaletteKind::Action(action),
                        label: label.to_owned(),
                        detail: String::new(),
                        chord: self.keymap.label_for(keymap_id),
                        score,
                    },
                    u64::MAX,
                ));
            }
        }
        if !query.is_empty() {
            for session in &self.store.sessions {
                let project = self
                    .store
                    .projects
                    .iter()
                    .find(|project| project.id == session.project_id);
                let project_name = project.map(|project| project.name.as_str()).unwrap_or("");
                let title = session_title(session);
                let agent = agent_short_label(&session.agent);
                let branch = session.branch.as_deref().unwrap_or_default();
                let haystack = format!("{title} {} {agent} {branch} {project_name}", session.goal);
                let Some(score) = palette_rank(&title, &haystack, &query) else {
                    continue;
                };
                // Only a row that will be drawn pays for how it is drawn. The
                // state view allocates, and computing it for every session in
                // the store on every frame the palette is open is the shape of
                // per-frame work `.claude/rules/rust.md` is about.
                let status = self.status_view(session);
                entries.push((
                    PaletteEntry {
                        kind: PaletteKind::Session(session.id),
                        label: title,
                        detail: format!("{} · {agent} · {project_name}", status.label),
                        chord: String::new(),
                        score,
                    },
                    session.launched_at.unwrap_or(session.created_at),
                ));
            }
            for project in &self.store.projects {
                let path = project.path.display().to_string();
                let haystack = format!("{} {path}", project.name);
                let Some(score) = palette_rank(&project.name, &haystack, &query) else {
                    continue;
                };
                entries.push((
                    PaletteEntry {
                        kind: PaletteKind::Project(project.id),
                        label: project.name.clone(),
                        detail: friendly_path(&project.path),
                        chord: String::new(),
                        score,
                    },
                    project.added_at,
                ));
            }
        }
        entries.sort_by(|(left, left_time), (right, right_time)| {
            left.score
                .cmp(&right.score)
                .then(left.kind.rank().cmp(&right.kind.rank()))
                .then(right_time.cmp(left_time))
        });
        entries.into_iter().map(|(entry, _)| entry).collect()
    }

    /// The rows that fit, and how many were left over. One pass over the store
    /// serves both, because the palette is drawn every frame it is open.
    pub(crate) fn palette_visible_entries(&self) -> (Vec<PaletteEntry>, usize) {
        let mut entries = self.palette_entries();
        let overflow = entries.len().saturating_sub(PALETTE_RESULT_LIMIT);
        entries.truncate(PALETTE_RESULT_LIMIT);
        (entries, overflow)
    }

    /// Opens one row. The click path and the `Enter` path both come here, so a
    /// row cannot behave differently depending on how it was chosen.
    pub(crate) fn activate_palette_entry(&mut self, kind: &PaletteKind) {
        match kind {
            PaletteKind::Action("New session") => self.open_new_session(),
            PaletteKind::Action("Open history") => {
                self.page = Page::Sessions;
                self.session_library_open = true;
            }
            PaletteKind::Action("Open Projects") => self.page = Page::Projects,
            PaletteKind::Action("Open Sessions") => self.page = Page::Sessions,
            PaletteKind::Action("Open Settings") => self.page = Page::Settings,
            PaletteKind::Action("Add a project") => self.choose_project_folder(),
            PaletteKind::Action("Scan a workspace") => self.choose_workspace_folder(),
            PaletteKind::Action("Open the keymap file") => self.open_keymap_file(),
            PaletteKind::Action(_) => {}
            PaletteKind::Session(session_id) => self.open_session_in_terminal(*session_id),
            PaletteKind::Project(project_id) => {
                self.select_project(Some(*project_id));
                self.page = Page::Projects;
            }
        }
        self.close_command_palette();
    }

    /// Moves the selection and, on `Enter`, opens what it points at.
    ///
    /// Handled here rather than inside the window so that the keys never reach
    /// the text field — an arrow that moved the caret instead of the selection
    /// is the whole reason this is not drawn-and-handled in one place.
    pub(crate) fn handle_palette_keys(&mut self, ctx: &egui::Context) {
        if !self.command_palette_open {
            return;
        }
        let (entries, _) = self.palette_visible_entries();
        if entries.is_empty() {
            self.command_selection = 0;
            return;
        }
        // The results change as the query does, so the selection is put back
        // inside them every frame rather than only when a key moves it.
        self.command_selection = self.command_selection.min(entries.len() - 1);
        let (down, up, enter) = ctx.input_mut(|input| {
            (
                input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown),
                input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp),
                input.consume_key(egui::Modifiers::NONE, egui::Key::Enter),
            )
        });
        if down {
            self.command_selection = (self.command_selection + 1).min(entries.len() - 1);
        }
        if up {
            self.command_selection = self.command_selection.saturating_sub(1);
        }
        if enter {
            let kind = entries[self.command_selection].kind.clone();
            self.activate_palette_entry(&kind);
        }
    }

    pub(crate) fn ui_command_palette(&mut self, ctx: &egui::Context) {
        if !self.command_palette_open {
            return;
        }
        let mut open = true;
        egui::Window::new(tr("検索・操作"))
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .default_width(420.0)
            .show(ctx, |ui| {
                let search = ui.add(
                    egui::TextEdit::singleline(&mut self.command_search)
                        .hint_text(tr("操作・セッション・プロジェクトを検索"))
                        .desired_width(f32::INFINITY),
                );
                if self.command_palette_needs_focus {
                    search.request_focus();
                    self.command_palette_needs_focus = false;
                }
                let palette = self.store.theme.palette();
                let (entries, overflow) = self.palette_visible_entries();
                if entries.is_empty() {
                    ui.add_space(SPACE_SM);
                    ui.label(RichText::new(tr("一致する項目がありません")).weak());
                }
                let mut chosen = None;
                for (index, entry) in entries.iter().enumerate() {
                    let selected = index == self.command_selection;
                    // The chord sits at the right of the row it belongs to,
                    // drawn from the same map the frame loop listens with, so
                    // the palette cannot name a key the application ignores.
                    let row = ui
                        .horizontal(|ui| {
                            let row = ui.selectable_label(
                                selected,
                                RichText::new(&entry.label).size(14.0).color(palette.text),
                            );
                            if !entry.chord.is_empty() {
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.label(
                                            RichText::new(&entry.chord)
                                                .size(12.0)
                                                .color(palette.text_muted),
                                        );
                                    },
                                );
                            }
                            row
                        })
                        .inner;
                    if !entry.detail.is_empty() {
                        ui.label(
                            RichText::new(&entry.detail)
                                .size(11.5)
                                .color(palette.text_muted),
                        );
                    }
                    if row.clicked() {
                        chosen = Some(entry.kind.clone());
                    }
                    if selected {
                        row.scroll_to_me(None);
                    }
                }
                // The cap is not a scrollbar: a query is how the list gets
                // shorter, so say what is being left out rather than hiding it.
                if overflow > 0 {
                    ui.add_space(SPACE_XS);
                    ui.label(
                        RichText::new(tf!(
                            "ほかに {p0} 件あります。検索語を足してください。",
                            p0 = overflow
                        ))
                        .small()
                        .weak(),
                    );
                }
                if let Some(kind) = chosen {
                    self.activate_palette_entry(&kind);
                }
            });
        if !open {
            self.command_palette_open = false;
            self.command_search.clear();
        }
    }

    pub(crate) fn ui_home(&mut self, ui: &mut egui::Ui) {
        let palette = self.store.theme.palette();
        let project_count = self.store.projects.len();
        // The sidebar's count, so a tile and the chip it opens agree.
        let group_counts = self.status_group_counts();

        let awaiting_input_count = self.attention_count();

        // No start button here: the toolbar's 「新しいセッション」 is on every
        // page, and a second primary action beside it would be a second way to
        // do one thing that did it differently.
        page_header(
            ui,
            palette,
            tr("ホーム"),
            Some(tr("いま何が動いていて、どれがあなたを待っているか。")),
            |_| {},
        );

        // The one state worth interrupting for goes above the counts, because
        // a number nobody has to act on should never be read before the one
        // somebody does. It is drawn with nothing waiting too: an empty line
        // that says so is how "nothing waits" differs from "not looked".
        if awaiting_input_count > 0 {
            let mut waiting: Vec<&Session> = self
                .store
                .sessions
                .iter()
                .filter(|session| {
                    self.session_in_status_group(session, SessionStatusGroup::NeedsYou)
                })
                .collect();
            waiting.sort_by_key(|session| std::cmp::Reverse(session.created_at));
            // Owned only for the five drawn: a row takes `&mut self`.
            let waiting: Vec<Session> = waiting.into_iter().take(5).cloned().collect();
            card_frame(palette)
                .fill(palette.row_selected)
                .stroke(egui::Stroke::new(1.0, palette.warning))
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(ICON_ATTENTION)
                                .size(16.0)
                                .color(palette.warning),
                        );
                        // Not "waiting for an answer": this counts the 要対応
                        // group, and a session blocked on a prerequisite that
                        // failed has no terminal to answer in.
                        ui.label(
                            RichText::new(tf!(
                                "{awaiting_input_count} 件のセッションがあなたを待っています",
                                awaiting_input_count = awaiting_input_count
                            ))
                            .color(palette.warning)
                            .strong(),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if secondary_button(ui, tr("要対応のセッションを見る")).clicked()
                            {
                                self.show_sessions_needing_you();
                            }
                        });
                    });
                    ui.add_space(SPACE_SM);
                    for session in &waiting {
                        self.home_session_row(ui, session, "waiting");
                    }
                });
        } else {
            card_frame(palette).show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(ICON_STATUS_DONE)
                            .size(16.0)
                            .color(palette.success),
                    );
                    ui.label(
                        RichText::new(tr("あなたを待っているセッションはありません"))
                            .color(palette.text_strong)
                            .strong(),
                    );
                    ui.label(
                        RichText::new(tr("要対応になるとここに先頭表示されます"))
                            .size(12.0)
                            .color(palette.text_muted),
                    );
                });
            });
        }
        ui.add_space(SPACE_MD);

        // One tile per group, in the chips' order, named with the chips' words:
        // Home and the session list describe a session the same way.
        let mut go = None;
        ui.columns(4, |columns| {
            for (column, group) in columns.iter_mut().zip(SessionStatusGroup::GROUPS) {
                let icon = match group {
                    SessionStatusGroup::NeedsYou => ICON_ATTENTION,
                    SessionStatusGroup::Busy => ICON_STATUS_RUNNING,
                    SessionStatusGroup::Free => ICON_STATUS_IDLE,
                    SessionStatusGroup::Finished => ICON_STATUS_DONE,
                };
                let tile = metric_tile(
                    column,
                    palette,
                    icon,
                    tr(group.message_id()),
                    group_counts[group.index()],
                    palette.status(group.tone()),
                );
                if tile.clicked() {
                    go = Some(group);
                }
            }
        });
        if let Some(group) = go {
            self.show_status_group(group);
        }

        ui.add_space(SPACE_XL);
        // The recent list and, beside it, one way into each project. Under
        // 720 px the column would squeeze the rows onto two lines, so it
        // goes underneath instead.
        const PROJECT_COLUMN: f32 = 300.0;
        if project_count == 0 {
            self.home_recent_sessions(ui);
        } else if ui.available_width() < 720.0 {
            self.home_recent_sessions(ui);
            ui.add_space(SPACE_XL);
            self.home_project_starts(ui);
        } else {
            let height = ui.available_height();
            let recent_width = ui.available_width() - PROJECT_COLUMN - SPACE_LG;
            ui.horizontal_top(|ui| {
                ui.allocate_ui_with_layout(
                    egui::vec2(recent_width, height),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| self.home_recent_sessions(ui),
                );
                ui.add_space(SPACE_LG);
                ui.allocate_ui_with_layout(
                    egui::vec2(PROJECT_COLUMN, height),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| self.home_project_starts(ui),
                );
            });
        }
    }

    fn home_recent_sessions(&mut self, ui: &mut egui::Ui) {
        let palette = self.store.theme.palette();
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(tr("最近のセッション"))
                    .size(15.0)
                    .strong()
                    .color(palette.text_strong),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if quiet_button(ui, palette, tr("すべて表示")).clicked() {
                    self.page = Page::Sessions;
                }
            });
        });
        ui.add_space(SPACE_SM);

        let mut recent: Vec<&Session> = self.store.sessions.iter().collect();
        recent.sort_by_key(|session| std::cmp::Reverse(session.created_at));
        let recent: Vec<Session> = recent.into_iter().take(6).cloned().collect();
        if recent.is_empty() {
            let message = if self.store.projects.is_empty() {
                tr("セッションはまだありません。プロジェクトを追加すると、ここに履歴が並びます。")
            } else {
                tr("セッションはまだありません。ツールバーの「新しいセッション」から始められます。")
            };
            empty_state(ui, palette, ICON_SHOW, message);
            return;
        }
        for session in &recent {
            self.home_session_row(ui, session, "recent");
        }
    }

    /// One start per project, so a session in a project other than the one
    /// the toolbar would pick is one click from Home rather than a trip
    /// through Projects.
    fn home_project_starts(&mut self, ui: &mut egui::Ui) {
        let palette = self.store.theme.palette();
        ui.set_width(ui.available_width());
        ui.label(
            RichText::new(tr("プロジェクトから始める"))
                .size(15.0)
                .strong()
                .color(palette.text_strong),
        );
        ui.add_space(SPACE_SM);
        let mut start = None;
        for project in &self.store.projects {
            card_frame(palette)
                .inner_margin(egui::Margin::symmetric(12, 8))
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    // The button first, so a long name is cut short rather
                    // than drawn under it.
                    // `horizontal` holds the row to one line's height.
                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if icon_text_button(ui, ICON_ADD, tr("セッション")).clicked() {
                                start = Some(project.id);
                            }
                            ui.with_layout(
                                egui::Layout::left_to_right(egui::Align::Center),
                                |ui| {
                                    ui.label(RichText::new(ICON_PROJECT).color(palette.accent));
                                    ui.add(
                                        egui::Label::new(
                                            RichText::new(&project.name)
                                                .strong()
                                                .color(palette.text_strong),
                                        )
                                        .truncate(),
                                    );
                                },
                            );
                        });
                    });
                });
            ui.add_space(SPACE_XS);
        }
        if let Some(project_id) = start {
            self.open_project_session_setup(project_id);
        }
    }

    /// A session on one line: what it is doing, what it is, where, and the
    /// one thing to do with it. The details the old card spelled out under
    /// every session — tmux name, working directory, the resume command —
    /// are how a person attaches from a shell, which is rare, so they sit in
    /// the ··· menu with the rare actions.
    ///
    /// `list` salts the ids: a waiting session is usually a recent one too,
    /// and is drawn in both lists on the same frame.
    fn home_session_row(&mut self, ui: &mut egui::Ui, session: &Session, list: &'static str) {
        let palette = self.store.theme.palette();
        let status = self.status_view(session);
        let title = session_title(session);
        let project_name = self
            .store
            .projects
            .iter()
            .find(|project| project.id == session.project_id)
            .map(|project| project.name.as_str())
            .unwrap_or_else(|| tr("登録されていないプロジェクト"))
            .to_owned();
        let verb = session_verb(session, self.cancellation_pending(session.id));
        let mut open_requested = false;
        let mut verb_requested = None;
        let mut close_requested = false;
        let mut removal_requested = false;
        let row = clickable_card(ui, palette, ("home-row", list, session.id), true, |ui| {
            ui.horizontal(|ui| {
                status_chip(ui, &status, 13.5, palette);
                // Right to left first, so the action and the menu keep their
                // place and the title takes whatever width is left.
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let _ = ui
                        .menu_button(RichText::new(ICON_MORE).size(14.0), |ui| {
                            if let Some(stop) = verb
                                .as_ref()
                                .filter(|verb| matches!(verb.action, SessionVerbAction::Stop))
                            {
                                if ui.button(stop.label).clicked() {
                                    verb_requested = Some(SessionVerbAction::Stop);
                                    ui.close_menu();
                                }
                            }
                            if matches!(
                                session.status,
                                SessionStatus::Exited | SessionStatus::Failed
                            ) && ui
                                .button(if session.status == SessionStatus::Failed {
                                    tr("残ったターミナルを閉じる")
                                } else {
                                    tr("ターミナルを閉じる")
                                })
                                .clicked()
                            {
                                close_requested = true;
                                ui.close_menu();
                            }
                            if ui.button(tr("セッションを削除")).clicked() {
                                removal_requested = true;
                                ui.close_menu();
                            }
                            ui.separator();
                            self.home_session_details(ui, session, &title);
                        })
                        .response
                        .on_hover_text(tr("その他の操作"));
                    match verb
                        .as_ref()
                        .filter(|verb| !matches!(verb.action, SessionVerbAction::Stop))
                    {
                        Some(verb) => {
                            if verb_button(ui, palette, verb.icon, verb.label, verb.danger)
                                .on_hover_text(&verb.hint)
                                .clicked()
                            {
                                verb_requested = Some(verb.action);
                            }
                        }
                        None => {
                            if secondary_button(ui, tr("開く")).clicked() {
                                open_requested = true;
                            }
                        }
                    }
                    ui.label(
                        RichText::new(relative_time(session.created_at))
                            .size(12.0)
                            .color(palette.text_muted),
                    );
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        // A truncated label takes all the width it is offered,
                        // so each one is offered its share: a long title is cut
                        // short instead of pushing the agent, project, and
                        // branch over the time and the action.
                        let shared = (ui.available_width() - 60.0).max(0.0);
                        capped_label(
                            ui,
                            shared * 0.5,
                            RichText::new(&title).strong().color(palette.text_strong),
                        );
                        agent_chip(ui, &session.agent, 12.0, palette);
                        capped_label(
                            ui,
                            shared * 0.25,
                            RichText::new(&project_name)
                                .size(12.0)
                                .color(palette.text_muted),
                        );
                        if let Some(branch) = &session.branch {
                            capped_label(
                                ui,
                                shared * 0.25,
                                RichText::new(format!("{ICON_BRANCH} {branch}"))
                                    .size(12.0)
                                    .color(palette.branch),
                            );
                        }
                    });
                });
            });
            self.session_removal_confirmation(ui, session.id);
        });
        if removal_requested {
            self.request_session_removal(session.id);
        } else if close_requested {
            self.close_completed_terminal(session.id);
        } else if let Some(action) = verb_requested {
            match action {
                SessionVerbAction::Stop => self.stop_session(session.id),
                SessionVerbAction::Start => self.start_session(session.id),
                SessionVerbAction::ResumeNative => self.resume_managed_native_session(session.id),
            }
        } else if open_requested || row.response.clicked() {
            self.open_session_in_terminal(session.id);
        }
        ui.add_space(SPACE_XS);
    }

    /// What the old card printed under every session, drawn only while the
    /// row's menu is open.
    fn home_session_details(&self, ui: &mut egui::Ui, session: &Session, title: &str) {
        let palette = self.store.theme.palette();
        let goal = session_display_goal(&session.goal);
        if goal != title && !goal.trim().is_empty() {
            ui.label(RichText::new(goal).small().color(palette.text));
        }
        ui.label(
            RichText::new(format!("tmux: {}", session.tmux_name))
                .small()
                .weak(),
        );
        if let Some(path) = &session.worktree_path {
            ui.label(
                RichText::new(format!("cwd: {}", path.display()))
                    .small()
                    .weak(),
            );
        }
        if let (Some(provider), Some(native_session_id)) = (
            cli_provider_for_agent(&session.agent),
            &session.native_session_id,
        ) {
            // The same command the card printed and the resume verb runs,
            // launch options included; see `session_card`.
            ui.label(
                RichText::new(format!(
                    "native resume: {}",
                    resume_command_with_launch_options(
                        provider,
                        &session.agent,
                        &session.agent_command,
                        native_session_id,
                    )
                ))
                .small()
                .weak(),
            );
        } else if cli_provider_for_agent(&session.agent).is_some() {
            ui.label(
                RichText::new(tr("ネイティブ再開 ID を待機中"))
                    .small()
                    .weak(),
            );
        }
        if !session.depends_on.is_empty() {
            ui.label(
                RichText::new(tf!(
                    "{p0} 件のセッションに依存",
                    p0 = session.depends_on.len()
                ))
                .small()
                .weak(),
            );
        }
        if let Some(parent) = blocking_dependency_label(&self.store.sessions, session) {
            ui.label(
                RichText::new(tf!(
                    "ブロック中: 前提 “{parent}” が完了していません。",
                    parent = parent
                ))
                .small()
                .color(palette.accent_soft),
            );
        }
    }

    pub(crate) fn ui_projects(&mut self, ui: &mut egui::Ui) {
        let palette = self.store.theme.palette();
        if self.selected_project().is_none() {
            let first_run = self.store.projects.is_empty();
            if first_run {
                ui.add_space(26.0);
                ui.vertical_centered(|ui| {
                    // A build whose bundled mark will not decode draws no mark
                    // and keeps its heading: an empty state that cannot say
                    // what to do next is worse than one without a logo.
                    if let Some(mark) = brand_mark_image(ui.ctx(), 48.0) {
                        ui.add(mark);
                    }
                    ui.heading(tr("まずはプロジェクトを開いてください"));
                    ui.label(
                        RichText::new(tr("フォルダを選ぶと、その中で AI を起動できます。"))
                            .color(palette.text),
                    );
                    ui.add_space(14.0);
                    if ui
                        .add_sized(
                            [240.0, CONTROL_HEIGHT],
                            egui::Button::new(
                                RichText::new(tr("プロジェクトフォルダを選択…")).strong(),
                            ),
                        )
                        .clicked()
                    {
                        self.choose_project_folder();
                    }
                    ui.add_space(4.0);
                    if ui
                        .add_sized(
                            [240.0, CONTROL_HEIGHT],
                            egui::Button::new(tr("ワークスペースをスキャン…")),
                        )
                        .clicked()
                    {
                        self.choose_workspace_folder();
                    }
                    ui.add_space(10.0);
                    ui.label(
                        RichText::new(tr(
                            "Finder からフォルダをここにドラッグすることもできます · ⌘O",
                        ))
                        .small()
                        .color(palette.text_muted),
                    );
                });
                ui.add_space(24.0);
                ui.vertical_centered(|ui| {
                    egui::Frame::group(ui.style()).show(ui, |ui| {
                        ui.set_min_width(380.0);
                        ui.label(RichText::new(tr("最初のセッションを始める前に")).strong());
                        readiness_row(
                            ui,
                            self.tools.tmux,
                            "tmux",
                            tr("セッション実行に必須"),
                            palette,
                        );
                        readiness_row(
                            ui,
                            self.tools.available_agent_count() > 0,
                            tr("コーディングエージェント"),
                            if self.tools.available_agent_count() > 0 {
                                tr("1 つ以上インストール済み")
                            } else {
                                tr("Codex・Claude・Antigravity のいずれかが必要です")
                            },
                            palette,
                        );
                        if (!self.tools.tmux || self.tools.available_agent_count() == 0)
                            && ui.button(tr("セットアップ手順を開く")).clicked()
                        {
                            self.open_settings(SettingsSection::Agents);
                        }
                        ui.label(
                            RichText::new(tr("信頼できるフォルダでのみ起動してください。"))
                                .small()
                                .color(palette.accent_soft),
                        );
                    });
                });
            } else {
                let mut choose_folder = false;
                let mut scan_workspace = false;
                page_header(
                    ui,
                    palette,
                    tr("プロジェクト"),
                    Some(tr(
                        "開くか、フォルダをこのウィンドウにドロップしてください。",
                    )),
                    |ui| {
                        // Secondary: the toolbar's 「新しいセッション」 is this
                        // page's one primary action too.
                        if icon_text_button(ui, ICON_ADD, tr("フォルダを選択…")).clicked()
                        {
                            choose_folder = true;
                        }
                        if secondary_button(ui, tr("ワークスペースをスキャン…")).clicked()
                        {
                            scan_workspace = true;
                        }
                    },
                );
                if choose_folder {
                    self.choose_project_folder();
                }
                if scan_workspace {
                    self.choose_workspace_folder();
                }
                let mut projects = self.store.projects.clone();
                projects.sort_by_key(|project| std::cmp::Reverse(project.added_at));
                for project in projects {
                    if self.pending_project_removal == Some(project.id) {
                        card_frame(palette)
                            .stroke(egui::Stroke::new(1.0, palette.danger))
                            .show(ui, |ui| {
                                ui.set_min_width(ui.available_width());
                                ui.label(
                                    RichText::new(tf!(
                                        "「{name}」を Operon から削除しますか？",
                                        name = &project.name
                                    ))
                                    .strong()
                                    .color(palette.text_strong),
                                );
                                ui.label(
                                    RichText::new(tr(
                                        "記録だけを削除します。ファイル・ブランチ・worktree は残ります。",
                                    ))
                                    .size(12.5)
                                    .color(palette.text_muted),
                                );
                                ui.add_space(SPACE_SM);
                                ui.horizontal(|ui| {
                                    let remove_res = ui.add(
                                        egui::Button::new(
                                            RichText::new(tr("Operon から削除"))
                                                .color(readable_text_on(palette.danger, palette)),
                                        )
                                        .fill(palette.danger)
                                        .stroke(egui::Stroke::NONE)
                                        .min_size(egui::vec2(0.0, CONTROL_HEIGHT)),
                                    );
                                    paint_button_focus(ui, &remove_res, palette);
                                    if remove_res.clicked() {
                                        self.remove_project(project.id);
                                    }
                                    if quiet_button(ui, palette, tr("キャンセル")).clicked() {
                                        self.pending_project_removal = None;
                                    }
                                });
                            });
                        ui.add_space(SPACE_SM);
                        continue;
                    }

                    let mut removal_requested = false;
                    let mut reveal_requested = false;
                    let row =
                        clickable_card(ui, palette, ("project-row", project.id), true, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(ICON_PROJECT)
                                        .size(20.0)
                                        .color(palette.accent_text),
                                );
                                ui.add_space(SPACE_XS);
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if small_icon_button(
                                            ui,
                                            palette,
                                            ICON_CLOSE,
                                            tr("プロジェクトを削除"),
                                        )
                                        .clicked()
                                        {
                                            removal_requested = true;
                                        }
                                        if small_icon_button(
                                            ui,
                                            palette,
                                            ICON_OPEN_EXTERNAL,
                                            tr("Finder で表示"),
                                        )
                                        .clicked()
                                        {
                                            reveal_requested = true;
                                        }
                                        ui.with_layout(
                                            egui::Layout::top_down(egui::Align::LEFT),
                                            |ui| {
                                                ui.label(
                                                    RichText::new(&project.name)
                                                        .size(14.0)
                                                        .strong()
                                                        .color(palette.text_strong),
                                                );
                                                ui.add(
                                                    egui::Label::new(
                                                        RichText::new(friendly_path(&project.path))
                                                            .size(12.0)
                                                            .color(palette.text_muted),
                                                    )
                                                    .truncate(),
                                                );
                                            },
                                        );
                                    },
                                );
                            });
                        });
                    let mut menu_remove = false;
                    let mut menu_reveal = false;
                    row.response.context_menu(|ui| {
                        if ui.button(tr("Finder で表示")).clicked() {
                            menu_reveal = true;
                            ui.close_menu();
                        }
                        if ui.button(tr("プロジェクトを削除")).clicked() {
                            menu_remove = true;
                            ui.close_menu();
                        }
                    });
                    if removal_requested || menu_remove {
                        self.pending_project_removal = Some(project.id);
                    } else if reveal_requested || menu_reveal {
                        let path = project.path.clone();
                        self.request_system_action(tr("Finder で表示"), move || {
                            reveal_path(&path)
                        });
                    } else if row.response.clicked() {
                        self.select_project(Some(project.id));
                        self.project_tab = ProjectTab::Overview;
                    }
                    ui.add_space(SPACE_SM);
                }
            }

            ui.add_space(14.0);
            let manual_label = if self.show_manual_project_entry {
                tf!(
                    "{ICON_DISCLOSURE_OPEN} パスを手入力",
                    ICON_DISCLOSURE_OPEN = ICON_DISCLOSURE_OPEN
                )
            } else {
                tf!(
                    "{ICON_DISCLOSURE_CLOSED} パスを手入力",
                    ICON_DISCLOSURE_CLOSED = ICON_DISCLOSURE_CLOSED
                )
            };
            let mut manual_clicked = false;
            if first_run {
                ui.vertical_centered(|ui| {
                    manual_clicked = secondary_button(ui, manual_label).clicked();
                });
            } else {
                manual_clicked = secondary_button(ui, manual_label).clicked();
            }
            if manual_clicked {
                self.show_manual_project_entry = !self.show_manual_project_entry;
            }
            if self.show_manual_project_entry {
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    ui.label(tr("プロジェクトフォルダ"));
                    ui.horizontal(|ui| {
                        let response = ui.add(
                            egui::TextEdit::singleline(&mut self.project_path_input)
                                .hint_text("/absolute/path/to/project")
                                .desired_width(520.0),
                        );
                        let submit = response.lost_focus()
                            && ui.input(|input| input.key_pressed(egui::Key::Enter));
                        if ui.button(tr("追加")).clicked() || submit {
                            self.add_project();
                        }
                    });
                    ui.label(tr("Git リポジトリを含むワークスペース（スキャン深さ: 3）"));
                    ui.horizontal(|ui| {
                        let response = ui.add(
                            egui::TextEdit::singleline(&mut self.workspace_scan_input)
                                .hint_text("/absolute/path/to/workspace")
                                .desired_width(520.0),
                        );
                        let submit = response.lost_focus()
                            && ui.input(|input| input.key_pressed(egui::Key::Enter));
                        if ui.button(tr("スキャン")).clicked() || submit {
                            self.import_workspace();
                        }
                    });
                });
            }
            return;
        }
        let project = self.selected_project().cloned().unwrap();
        // A breadcrumb rather than a back button: it says where this page sits
        // as well as how to leave it, which a lone arrow does not.
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = SPACE_XS;
            if quiet_button(
                ui,
                palette,
                tf!("{ICON_BACK}  プロジェクト", ICON_BACK = ICON_BACK),
            )
            .clicked()
            {
                self.select_project(None);
            }
            ui.label(RichText::new("/").size(12.5).color(palette.text_faint));
            ui.label(
                RichText::new(&project.name)
                    .size(12.5)
                    .color(palette.text_muted),
            );
        });
        ui.add_space(SPACE_XS);
        let mut remove_requested = false;
        page_header(
            ui,
            palette,
            &project.name,
            Some(&friendly_path(&project.path)),
            |ui| {
                if secondary_button(ui, tr("削除…"))
                    .on_hover_text(tr("Operon の記録からこのプロジェクトを外します"))
                    .clicked()
                {
                    remove_requested = true;
                }
                if icon_button(ui, palette, ICON_OPEN_EXTERNAL, tr("Finder で表示")).clicked() {
                    let path = project.path.clone();
                    self.request_system_action(tr("Finder で表示"), move || reveal_path(&path));
                }
            },
        );
        if remove_requested {
            self.pending_project_removal = Some(project.id);
        }
        if self.pending_project_removal == Some(project.id) {
            card_frame(palette)
                .stroke(egui::Stroke::new(1.0, palette.danger))
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.label(
                        RichText::new(tr("このプロジェクトを Operon から削除しますか？"))
                            .strong()
                            .color(palette.text_strong),
                    );
                    ui.label(
                        RichText::new(tr(
                            "記録だけを削除します。ファイル・ブランチ・worktree は残ります。",
                        ))
                        .size(12.5)
                        .color(palette.text_muted),
                    );
                    ui.add_space(SPACE_SM);
                    ui.horizontal(|ui| {
                        let remove_res = ui.add(
                            egui::Button::new(
                                RichText::new(tr("Operon から削除"))
                                    .color(readable_text_on(palette.danger, palette)),
                            )
                            .fill(palette.danger)
                            .stroke(egui::Stroke::NONE)
                            .min_size(egui::vec2(0.0, CONTROL_HEIGHT)),
                        );
                        paint_button_focus(ui, &remove_res, palette);
                        if remove_res.clicked() {
                            self.remove_project(project.id);
                        }
                        if quiet_button(ui, palette, tr("キャンセル")).clicked() {
                            self.pending_project_removal = None;
                        }
                    });
                });
            ui.add_space(SPACE_MD);
        }
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = SPACE_XS;
            for tab in ProjectTab::all() {
                let count = self.project_tab_count(tab, project.id);
                // Only uncommitted changes ask to be looked at; a worktree or
                // PR count is information, not a call to act.
                let emphasise = tab == ProjectTab::Git;
                if tab_item_with_count(
                    ui,
                    palette,
                    tab.label(),
                    count,
                    emphasise,
                    self.project_tab == tab,
                )
                .clicked()
                {
                    self.project_tab = tab;
                }
            }
        });
        tab_bar_rule(ui, palette);
        match self.project_tab {
            ProjectTab::Overview => self.ui_overview(ui, &project),
            ProjectTab::Git => self.ui_git(ui, &project),
            ProjectTab::Worktrees => self.ui_worktrees(ui, &project),
            ProjectTab::PullRequests => self.ui_pull_requests(ui, &project),
            ProjectTab::Files => self.ui_files(ui, &project),
            ProjectTab::AgentSettings => self.ui_agent_settings(ui, &project),
        }
    }

    /// The files the agents read about this project: its skills, then its
    /// instruction files (change 093: they were two tabs of one list each).
    pub(crate) fn ui_agent_settings(&mut self, ui: &mut egui::Ui, project: &Project) {
        self.ui_skills(ui, project);
        ui.add_space(SPACE_LG);
        self.ui_rules(ui, project);
    }

    pub(crate) fn select_agent(&mut self, agent: &str) {
        if self.selected_agent == agent {
            return;
        }
        self.record_current_agent_settings();
        self.selected_agent = agent.to_owned();
        self.apply_agent_launch_settings(agent);
        if let Err(error) = save_recent_agent_settings(&self.data_file, &self.recent_agent_settings)
        {
            eprintln!("Operon: failed to save recent agent settings: {error}");
        }
        // Consent given for one CLI's combination is not consent for another's,
        // even in the unlikely case the two spell out the same switches.
        self.acknowledged_launch = None;
    }

    /// What the launch form is asking to run, in the terms consent is given in.
    pub(crate) fn pending_launch(&self) -> LaunchAcknowledgement {
        let mut flags = self.agent_flag_inputs.clone();
        flags.sort();
        LaunchAcknowledgement {
            agent: self.selected_agent.clone(),
            mode: self.agent_mode_input.clone(),
            flags,
            custom_command: self.custom_command.clone(),
        }
    }

    pub(crate) fn launch_acknowledged(&self) -> bool {
        self.acknowledged_launch
            .as_ref()
            .is_some_and(|acknowledged| *acknowledged == self.pending_launch())
    }

    /// One agent on the sheet's single row: its mark, its name, and whether
    /// this Mac can run it. The whole frame is the button.
    pub(crate) fn ui_agent_choice(&mut self, ui: &mut egui::Ui, agent: &str) {
        let palette = self.store.theme.palette();
        let selected = self.selected_agent == agent;
        let available = self.tools.agent_available(agent);
        let (title, _) = agent_choice_copy(agent);
        let accent = palette.agent_accent(agent);
        let (fill, stroke) = if selected {
            (accent.gamma_multiply(0.18), egui::Stroke::new(1.5, accent))
        } else {
            (palette.card, egui::Stroke::new(1.0, palette.border))
        };
        let (dot, dot_color, dot_hint) = if available {
            (ICON_AVAILABLE, palette.success, tr("この Mac で使用可能"))
        } else {
            (
                ICON_UNAVAILABLE,
                palette.accent_soft,
                tr("未準備（設定で確認）"),
            )
        };
        let response = ui
            .scope_builder(
                egui::UiBuilder::new()
                    .id_salt(("agent-choice", agent))
                    .sense(egui::Sense::click()),
                |ui| {
                    ui.style_mut().interaction.selectable_labels = false;
                    egui::Frame::new()
                        .fill(fill)
                        .stroke(stroke)
                        .corner_radius(egui::CornerRadius::same(RADIUS_CONTROL))
                        .inner_margin(egui::Margin::symmetric(10, 6))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = 6.0;
                                agent_icon(ui, agent, 16.0);
                                ui.label(RichText::new(title).strong());
                                ui.label(RichText::new(dot).size(12.0).color(dot_color));
                            });
                        });
                },
            )
            .response
            .on_hover_text(dot_hint)
            .on_hover_cursor(egui::CursorIcon::PointingHand);
        if response.clicked() {
            self.select_agent(agent);
        }
    }

    /// The mode, the reasoning effort, and the switches the chosen CLI accepts.
    ///
    /// Change 012 removed a launch-option editor that nothing could reach, and
    /// was right to: a control nobody can open is a second answer to every
    /// question about where a launch is configured. What it left behind was the
    /// other half — `agent_mode_input`, `agent_effort_input`, and
    /// `agent_flag_inputs` stayed as state, `build_agent_launch_command` kept
    /// reading them, and the danger warning below kept asking a question about
    /// three fields nothing could fill. So the tables were complete, validated
    /// and tested, and the only way to ask this application for an unguarded
    /// run was to type one into 起動コマンド, where nothing asked anything.
    ///
    /// Every value here comes out of `src/agents.rs`. This function writes no
    /// flag and no mode of its own, which is what keeps a CLI's rename to one
    /// edit in one table.
    pub(crate) fn ui_launch_options(&mut self, ui: &mut egui::Ui, palette: &Palette) {
        // `&'static` every one of them, so nothing here borrows `self` and the
        // closures below are free to take it mutably.
        let modes = agent_mode_options(&self.selected_agent);
        let efforts = agent_effort_options(&self.selected_agent);
        let switches = agent_flag_options(&self.selected_agent);
        let mode_label = agent_mode_label(&self.selected_agent);
        if !modes.is_empty() {
            ui.horizontal(|ui| {
                // The CLI's own word for the setting. "モード" over Codex's
                // sandbox picker would leave a person guessing which of its two
                // approval-shaped settings they were looking at.
                ui.label(mode_label);
                // Through a local and then a setter, rather than straight into
                // the field: a mode can invalidate a switch that was ticked
                // while there was no mode, and a `selectable_value` writing the
                // field directly leaves that switch on, greyed out so it cannot
                // be unticked, and the launch button enabled over a combination
                // the CLI will refuse.
                let mut chosen = self.agent_mode_input.clone();
                egui::ComboBox::from_id_salt("launch-mode")
                    .selected_text(if chosen.is_empty() {
                        tr("既定（CLI に任せる）")
                    } else {
                        &chosen
                    })
                    .width(200.0)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut chosen, String::new(), tr("既定（CLI に任せる）"));
                        for mode in modes {
                            ui.selectable_value(&mut chosen, (*mode).to_owned(), *mode);
                        }
                    });
                if chosen != self.agent_mode_input {
                    self.set_launch_mode(chosen);
                }
                if is_dangerous_agent_mode(&self.agent_mode_input) {
                    ui.label(RichText::new(tr("危険")).small().color(palette.accent_soft));
                }
            });
        }
        if !efforts.is_empty() {
            ui.horizontal(|ui| {
                // Straight into the field, unlike the mode above. Effort carries
                // no exclusivity with any switch and no CLI treats it as
                // dangerous, so there is nothing for a setter to withdraw. The
                // next valued option added here should check that of itself
                // rather than copy whichever neighbour it sits beside.
                ui.label(tr("推論の深さ"));
                let mut effort_chosen = self.agent_effort_input.clone();
                egui::ComboBox::from_id_salt("launch-effort")
                    .selected_text(if effort_chosen.is_empty() {
                        tr("既定（CLI に任せる）")
                    } else {
                        &effort_chosen
                    })
                    .width(200.0)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut effort_chosen,
                            String::new(),
                            tr("既定（CLI に任せる）"),
                        );
                        for effort in efforts {
                            ui.selectable_value(&mut effort_chosen, (*effort).to_owned(), *effort);
                        }
                    });
                if effort_chosen != self.agent_effort_input {
                    self.agent_effort_input = effort_chosen;
                    self.persist_recent_agent_settings();
                }
            });
        }
        for flag in switches {
            let mut on = self.agent_flag_inputs.iter().any(|id| id == flag.id);
            // The CLI refuses this switch beside its mode flag, so it is not
            // offered while a mode is set. Refusing here rather than at launch
            // is the difference between a disabled checkbox and a window that
            // opens, prints a usage error, and dies with a session record
            // already written.
            let blocked_by_mode = flag.conflicts_with_mode && !self.agent_mode_input.is_empty();
            let label = if flag.dangerous {
                RichText::new(tf!("{p0}（危険）", p0 = tr(flag.label))).color(palette.accent_soft)
            } else {
                RichText::new(tr(flag.label))
            };
            let mut response =
                ui.add_enabled(!blocked_by_mode, egui::Checkbox::new(&mut on, label));
            response = response.on_hover_text(tr(flag.detail));
            if blocked_by_mode {
                response.on_disabled_hover_text(tf!(
                    "「{label}」は{p1}の指定と同時に使えません。",
                    label = tr(flag.label),
                    p1 = mode_label
                ));
                continue;
            }
            if response.changed() {
                self.toggle_launch_flag(flag, on);
            }
        }
    }

    /// Choose the mode, dropping whatever the mode has just invalidated.
    ///
    /// The mirror of `toggle_launch_flag`: a switch marked `conflicts_with_mode`
    /// is offered only while there is no mode, so setting one has to clear it.
    /// Without this the checkbox stayed ticked and disabled — unreachable to
    /// untick — while the launch button stayed enabled over a pair the CLI
    /// refuses, and the only refusal came from `resolve_agent_flags` after the
    /// click. A screen that offers a launch it will always reject is worse than
    /// one that never offered it.
    pub(crate) fn set_launch_mode(&mut self, mode: String) {
        let agent = self.selected_agent.clone();
        if !mode.is_empty() {
            self.agent_flag_inputs
                .retain(|id| agent_flag(&agent, id).is_some_and(|flag| !flag.conflicts_with_mode));
        }
        self.agent_mode_input = mode;
        self.persist_recent_agent_settings();
        // Consent was given for the combination that was on screen when it was
        // given. This is now a different one.
        self.acknowledged_launch = None;
    }

    /// Turn one switch on or off, keeping the CLI's own exclusivity.
    ///
    /// A named method rather than eight lines inside the loop above, because
    /// the rule it keeps — at most one switch per group, because the CLI takes
    /// at most one — is the thing worth a test, and a test that re-derives it
    /// from `group` is a second copy of the rule rather than a check on the
    /// first (lesson 005).
    pub(crate) fn toggle_launch_flag(&mut self, flag: &AgentFlag, on: bool) {
        let agent = self.selected_agent.clone();
        // An id this agent's catalogue does not know is the one input
        // `resolve_agent_flags` refuses outright, so it never survives a change
        // here: the postcondition of this method is a set that resolver accepts.
        self.agent_flag_inputs
            .retain(|id| agent_flag(&agent, id).is_some());
        if on {
            if !flag.group.is_empty() {
                self.agent_flag_inputs.retain(|id| {
                    agent_flag(&agent, id).is_some_and(|other| other.group != flag.group)
                });
            }
            if !self.agent_flag_inputs.iter().any(|id| id == flag.id) {
                self.agent_flag_inputs.push(flag.id.to_owned());
            }
        } else {
            self.agent_flag_inputs.retain(|id| id != flag.id);
        }
        self.persist_recent_agent_settings();
        // Consent was given for the combination that was on screen when it was
        // given. This is now a different one.
        self.acknowledged_launch = None;
    }

    /// What the project looks like: its sessions, its uncommitted changes, and
    /// its worktrees. The counts come from the caches the 変更 and worktree
    /// tabs fill in the background; the first visit asks for them.
    pub(crate) fn ui_overview(&mut self, ui: &mut egui::Ui, project: &Project) {
        let palette = self.store.theme.palette();
        if !self.git_changes_cache.contains_key(&project.id) {
            self.request_git_changes(project);
        }
        if !self.worktree_cache.contains_key(&project.id) {
            self.request_worktrees(project);
        }
        let session_count = self
            .store
            .sessions
            .iter()
            .filter(|session| session.project_id == project.id)
            .count();
        let changes = match self.git_changes_cache.get(&project.id) {
            Some(Ok(snapshot)) => snapshot.files.len().to_string(),
            _ => "—".to_owned(),
        };
        let worktrees = match self.worktree_cache.get(&project.id) {
            Some(Ok(worktrees)) => worktrees.len().to_string(),
            _ => "—".to_owned(),
        };
        ui.add_space(SPACE_MD);
        ui.columns(3, |columns| {
            overview_tile(
                &mut columns[0],
                palette,
                tr("セッション"),
                &session_count.to_string(),
            );
            overview_tile(&mut columns[1], palette, tr("未コミットの変更"), &changes);
            overview_tile(&mut columns[2], palette, "worktree", &worktrees);
        });
        ui.add_space(SPACE_MD);
        if icon_text_button(ui, ICON_ADD, tr("新しいセッション")).clicked() {
            self.open_project_session_setup(project.id);
        }
        ui.add_space(SPACE_LG);
        ui.label(
            RichText::new(tr("このプロジェクトのセッション"))
                .size(15.0)
                .strong()
                .color(palette.text_strong),
        );
        ui.add_space(SPACE_SM);
        let mut sessions: Vec<&Session> = self
            .store
            .sessions
            .iter()
            .filter(|session| session.project_id == project.id)
            .collect();
        sessions.sort_by_key(|session| std::cmp::Reverse(session.created_at));
        let sessions: Vec<Session> = sessions.into_iter().take(8).cloned().collect();
        if sessions.is_empty() {
            ui.label(RichText::new(tr("セッションはまだありません。")).weak());
        }
        for session in &sessions {
            self.home_session_row(ui, session, "project");
        }
    }

    /// The launch sheet's body: agent, request, where to work, and the folded
    /// detail. The footer — notices, the acknowledgement, the launch button —
    /// is `ui_launch_footer`, outside the sheet's scroll area.
    pub(crate) fn ui_launch_form(&mut self, ui: &mut egui::Ui, project: &Project) {
        let palette = self.store.theme.palette();
        ui.label(
            RichText::new(tr("エージェント"))
                .strong()
                .color(palette.text_strong),
        );
        ui.horizontal_wrapped(|ui| {
            for agent in ["codex", "claude", "gemini"] {
                self.ui_agent_choice(ui, agent);
            }
            if quiet_button(ui, palette, tr("別のコマンドを使う")).clicked() {
                self.select_agent("custom");
            }
        });
        if self.selected_agent == "custom" {
            ui.horizontal(|ui| {
                ui.label(tr("起動コマンド（必須）"));
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.custom_command)
                        .hint_text(tr("例: aider --model ..."))
                        .desired_width(280.0),
                );
                if response.lost_focus() {
                    self.persist_recent_agent_settings();
                }
            });
        }
        // Which login this session runs under. Beside the agent rather
        // than under the optional detail below, because it is the same
        // question as the agent — which CLI, as whom — and because a
        // person with two logins has to answer it every time. It spent
        // one release folded into 「3. 詳しい設定（任意）」, where a
        // choice somebody had already made was invisible until they
        // opened a section labelled optional.
        //
        // Drawn only for the two CLIs that have a variable for it: an
        // account row for an agent that cannot be pointed anywhere
        // would be a choice with no effect.
        if agent_supports_accounts(&self.selected_agent) {
            let registered = self
                .accounts
                .for_agent(&self.selected_agent)
                .into_iter()
                .map(|account| (account.id, account.name.clone()))
                .collect::<Vec<_>>();
            ui.horizontal(|ui| {
                ui.label(tr("アカウント"));
                let chosen = self
                    .agent_account_input
                    .and_then(|id| self.accounts.get(id))
                    .map(|account| account.name.clone())
                    .unwrap_or_else(|| tr("このマシン").to_owned());
                egui::ComboBox::from_id_salt("launch-account")
                    .selected_text(chosen)
                    .width(170.0)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.agent_account_input, None, tr("このマシン"));
                        for (id, name) in &registered {
                            ui.selectable_value(&mut self.agent_account_input, Some(*id), name);
                        }
                    });
                if registered.is_empty() {
                    ui.label(RichText::new(tr("「設定」で追加できます")).small().weak());
                }
            });
        }
        ui.add_space(SPACE_MD);
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(tr("依頼（任意）"))
                    .strong()
                    .color(palette.text_strong),
            );
            ui.menu_button(tr("テンプレートを挿入"), |ui| {
                let templates = list_prompt_templates(Some(&project.path));
                for t in templates {
                    if ui.button(&t.title).clicked() {
                        if self.goal_input.is_empty() {
                            self.goal_input = t.content;
                        } else {
                            self.goal_input = format!("{}\n\n{}", self.goal_input, t.content);
                        }
                        ui.close_menu();
                    }
                }
            });
            if self.tools.gh && ui.button(tr("GitHub Issue から取得…")).clicked() {
                self.fetch_github_issues(project);
            }
        });
        if let Some(issues) = self.cached_github_issues.get(&project.id) {
            if !issues.is_empty() {
                egui::CollapsingHeader::new(tf!("GitHub Issues ({count})", count = issues.len()))
                    .id_salt("launch-github-issues")
                    .default_open(false)
                    .show(ui, |ui| {
                        for issue in issues.iter().take(10) {
                            ui.horizontal(|ui| {
                                ui.monospace(format!("#{}", issue.number));
                                if ui.button(&issue.title).clicked() {
                                    self.session_name_input = format!("issue-{}", issue.number);
                                    self.goal_input = format!(
                                        "Fix #{}: {}\n\n{}",
                                        issue.number, issue.title, issue.body
                                    );
                                }
                            });
                        }
                    });
            }
        }
        let goal = ui.add(
            egui::TextEdit::multiline(&mut self.goal_input)
                .id_salt("launch-goal")
                .hint_text(tr(
                    "例: ログイン画面の不具合を調査して修正して。関連するテストも実行して。",
                ))
                .desired_rows(3)
                .desired_width(f32::INFINITY),
        );
        // Opening the sheet moves the keyboard here, away from a terminal
        // pane that forwards every key it holds focus for.
        if self.launch_sheet_needs_focus {
            goal.request_focus();
            self.launch_sheet_needs_focus = false;
        }
        ui.add_space(SPACE_MD);
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(tr("作業場所"))
                    .strong()
                    .color(palette.text_strong),
            );
            let worktree_active = !self.session_path_input.trim().is_empty();
            let target_path = if worktree_active {
                PathBuf::from(self.session_path_input.trim())
            } else {
                project.path.clone()
            };
            let folder_chip = if worktree_active {
                // From the worktree tab's cache, never from `git`: this row is
                // drawn every frame the sheet is open, and a child process here
                // would block the window on every keystroke typed into it.
                let branch = self
                    .worktree_cache
                    .get(&project.id)
                    .and_then(|worktrees| worktrees.as_ref().ok())
                    .and_then(|worktrees| {
                        worktrees
                            .iter()
                            .find(|worktree| worktree.path == target_path)
                            .and_then(|worktree| worktree.branch.clone())
                    })
                    .unwrap_or_default();
                if branch.is_empty() {
                    "worktree".to_owned()
                } else {
                    tf!("ワークツリー: {branch}", branch = branch)
                }
            } else {
                tr("プロジェクト直下").to_owned()
            };
            ui.label(RichText::new(folder_chip).color(palette.accent_text));
            if worktree_active && quiet_button(ui, palette, tr("プロジェクト直下に戻す")).clicked()
            {
                self.session_path_input.clear();
                self.session_path_project = None;
            }
            if small_icon_button(
                ui,
                palette,
                ICON_OPEN_EXTERNAL,
                tr("作業フォルダを Finder で表示"),
            )
            .clicked()
            {
                let reveal_target = target_path.clone();
                self.request_system_action(tr("Finder で表示"), move || {
                    reveal_path(&reveal_target)
                });
            }
        });
        ui.add_space(SPACE_SM);
        egui::CollapsingHeader::new(self.launch_detail_title())
            .id_salt("launch-detail")
            .default_open(false)
            .show(ui, |ui| {
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.label(tr("セッション名（任意）"));
                    ui.add(
                        egui::TextEdit::singleline(&mut self.session_name_input)
                            .hint_text(tr("例: ログイン不具合の修正"))
                            .desired_width(240.0),
                    );
                });
                if self.selected_agent != "custom" {
                    ui.horizontal(|ui| {
                        ui.label(tr("モデル（任意）"));
                        let response = ui.add(
                            egui::TextEdit::singleline(&mut self.agent_model_input)
                                .hint_text(tr("既定のモデルを使う"))
                                .desired_width(170.0),
                        );
                        if response.lost_focus() {
                            self.persist_recent_agent_settings();
                        }
                    });
                }
                self.ui_launch_options(ui, palette);
                let dependency_choices = self
                    .store
                    .sessions
                    .iter()
                    .filter(|session| {
                        session.project_id == project.id
                            && !matches!(
                                session.status,
                                SessionStatus::Failed
                                    | SessionStatus::Cancelled
                                    | SessionStatus::Lost
                            )
                            && !self.cancellation_pending(session.id)
                    })
                    .map(|session| {
                        // The same word the sidebar says for this session.
                        let label = format!(
                            "{} ({})",
                            session_title(session),
                            self.status_view(session).label
                        );
                        (session.id, label)
                    })
                    .collect::<Vec<_>>();
                ui.horizontal(|ui| {
                    ui.label(tr("ほかのセッションの後に開始"));
                    let selected_label = self
                        .depends_on_input
                        .and_then(|id| dependency_choices.iter().find(|(choice, _)| *choice == id))
                        .map(|(_, label)| label.clone())
                        .unwrap_or_else(|| tr("すぐ開始する（既定）").into());
                    egui::ComboBox::from_id_salt("depends-on")
                        .selected_text(selected_label)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.depends_on_input,
                                None,
                                tr("すぐ開始する（既定）"),
                            );
                            for (session_id, label) in &dependency_choices {
                                ui.selectable_value(
                                    &mut self.depends_on_input,
                                    Some(*session_id),
                                    label.as_str(),
                                );
                            }
                        });
                });
            });
    }

    /// Under the sheet's scroll area: what the launch will do, what stops it,
    /// and the two ways to start. Returns (launch, terminal only).
    fn ui_launch_footer(&mut self, ui: &mut egui::Ui, project: &Project) -> (bool, bool) {
        let palette = self.store.theme.palette();
        let worktree_mutation_running = self.worktree_mutation_running(project.id);
        // A refusal from `launch_session` is set as the notice, which the
        // central panel draws under the scrim. Repeated here, where it is read.
        if let Some(notice) = &self.notice {
            ui.label(
                RichText::new(notice.as_str())
                    .small()
                    .color(palette.accent_soft),
            );
        }
        let dangerous_mode = launch_needs_acknowledgement(
            &self.selected_agent,
            &self.agent_mode_input,
            &self.agent_flag_inputs,
            &self.custom_command,
        );
        ui.label(RichText::new(tr("選んだ AI は外部ツールとして実行され、プロジェクトの内容を提供元へ送信することがあります。")).small().weak());
        if agent_requires_visible_terminal(&self.selected_agent) {
            ui.label(
                RichText::new(tr("Claude Code は起動後にアプリ内ターミナルで開きます。"))
                    .small()
                    .weak(),
            );
        }
        if dangerous_mode {
            ui.label(
                RichText::new(tr(
                    "警告: 確認もサンドボックスもなしにファイルを変更できます。",
                ))
                .small()
                .color(palette.accent_soft),
            );
            let mut acknowledged = self.launch_acknowledged();
            if ui
                .checkbox(&mut acknowledged, tr("理解したうえで起動する"))
                .changed()
            {
                let pending = self.pending_launch();
                self.acknowledged_launch = acknowledged.then_some(pending);
            }
        }
        let launch_ready = self.launch_ready(project);
        if !launch_ready {
            let agent_ready = self.tools.agent_available(&self.selected_agent);
            let custom_command_ready =
                self.selected_agent != "custom" || !self.custom_command.trim().is_empty();
            let reason = if !custom_command_ready {
                tr("起動コマンドを入力してください")
            } else if !self.tools.tmux {
                tr("tmux が必要です")
            } else if !agent_ready {
                tr("選んだ AI が見つかりません")
            } else if worktree_mutation_running {
                tr("worktree 操作の完了待ち")
            } else {
                tr("上の確認にチェックを入れてください")
            };
            ui.label(RichText::new(reason).small().color(palette.accent_soft));
            if (!self.tools.tmux || !agent_ready)
                && icon_text_button(ui, ICON_SETTINGS, tr("設定を開く")).clicked()
            {
                self.open_settings(SettingsSection::Agents);
                self.launch_sheet_open = false;
            }
        }
        if !self.launch_sheet_is_git {
            ui.label(
                RichText::new(tr(
                    "Git リポジトリではありません。AI の起動とファイル閲覧は使えます。",
                ))
                .small()
                .color(palette.accent_soft),
            );
        }
        if worktree_mutation_running {
            ui.label(
                RichText::new(tr("worktree 操作中です。完了後に起動できます。"))
                    .small()
                    .color(palette.accent_soft),
            );
        }
        ui.add_space(SPACE_SM);
        let mut launch = false;
        let mut terminal_only = false;
        ui.horizontal(|ui| {
            ui.add_enabled_ui(self.tools.tmux && !worktree_mutation_running, |ui| {
                if quiet_button(ui, palette, tr("ターミナルだけ開く")).clicked() {
                    terminal_only = true;
                }
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let accent = palette.agent_accent(&self.selected_agent);
                let label = tf!(
                    "{p0} を起動  ⌘↩",
                    p0 = agent_choice_copy(&self.selected_agent).0
                );
                if ui
                    .add_enabled(
                        launch_ready,
                        egui::Button::new(
                            RichText::new(label)
                                .strong()
                                .color(readable_text_on(accent, palette)),
                        )
                        .fill(accent)
                        .min_size(egui::vec2(0.0, CONTROL_HEIGHT)),
                    )
                    .clicked()
                {
                    launch = true;
                }
            });
        });
        (launch, terminal_only)
    }

    /// The launch sheet (change 092): the form over whatever page is showing,
    /// so starting a session never navigates away. Closed by ✕ or Esc; ⌘↩
    /// presses its button. Drawn before the launch-progress modal, which a
    /// launch hands over to.
    pub(crate) fn ui_launch_sheet(&mut self, ctx: &egui::Context) {
        if !self.launch_sheet_open {
            return;
        }
        // A progress modal owns the window while it shows: the sheet waits
        // under it rather than taking Esc and ⌘↩ from it.
        let progress_showing = self.launch_progress.as_ref().is_some_and(|p| !p.dismissed)
            || self.restore_progress.is_some();
        if progress_showing {
            return;
        }
        let Some(project) = self.selected_project().cloned() else {
            self.launch_sheet_open = false;
            return;
        };
        let palette = self.store.theme.palette();
        let screen = ctx.screen_rect();
        egui::Area::new(egui::Id::new("launch-sheet-scrim"))
            .order(egui::Order::Middle)
            .fixed_pos(screen.min)
            .show(ctx, |ui| {
                // Takes the click and does nothing: a stray click must not
                // throw away a request somebody was typing.
                let (rect, _) = ui.allocate_exact_size(screen.size(), egui::Sense::click());
                ui.painter().rect_filled(rect, 0.0, palette.scrim);
            });
        if !self.command_palette_open
            && ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
        {
            self.launch_sheet_open = false;
            return;
        }
        let command_enter = !self.command_palette_open
            && ctx.input_mut(|input| input.consume_key(egui::Modifiers::COMMAND, egui::Key::Enter));
        let projects: Vec<(Uuid, String)> = self
            .store
            .projects
            .iter()
            .map(|project| (project.id, project.name.clone()))
            .collect();
        let sessions_before = self.store.sessions.len();
        let mut close = false;
        let mut switch_to = None;
        let mut launch = command_enter && self.launch_ready(&project);
        let mut terminal_only = false;
        const WIDTH: f32 = 560.0;
        egui::Area::new(egui::Id::new("launch-sheet"))
            .order(egui::Order::Foreground)
            .fixed_pos(egui::pos2(
                screen.center().x - WIDTH / 2.0 - 18.0,
                screen.top() + screen.height() * 0.1,
            ))
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(palette.raised)
                    .stroke(egui::Stroke::new(1.0, palette.border))
                    .corner_radius(egui::CornerRadius::same(RADIUS_WINDOW))
                    .inner_margin(egui::Margin::same(18))
                    .shadow(ui.style().visuals.popup_shadow)
                    .show(ui, |ui| {
                        ui.set_width(WIDTH);
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(tr("新しいセッション"))
                                    .size(16.0)
                                    .strong()
                                    .color(palette.text_strong),
                            );
                            egui::ComboBox::from_id_salt("launch-sheet-project")
                                .selected_text(&project.name)
                                .show_ui(ui, |ui| {
                                    for (id, name) in &projects {
                                        if ui.selectable_label(*id == project.id, name).clicked() {
                                            switch_to = Some(*id);
                                        }
                                    }
                                });
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if small_icon_button(ui, palette, ICON_CLOSE, tr("閉じる"))
                                        .clicked()
                                    {
                                        close = true;
                                    }
                                },
                            );
                        });
                        ui.add_space(SPACE_SM);
                        egui::ScrollArea::vertical()
                            .id_salt("launch-sheet-body")
                            .max_height((screen.height() * 0.8 - 200.0).max(160.0))
                            .auto_shrink([false, true])
                            .show(ui, |ui| self.ui_launch_form(ui, &project));
                        ui.separator();
                        let (pressed, empty) = self.ui_launch_footer(ui, &project);
                        launch |= pressed;
                        terminal_only = empty;
                    });
            });
        if close {
            self.launch_sheet_open = false;
            return;
        }
        if let Some(project_id) = switch_to {
            self.switch_launch_project(project_id);
            return;
        }
        // Asked again after the form drew: a click in the same frame can have
        // changed the agent or a switch since `launch` was first computed.
        if launch && self.launch_ready(&project) {
            self.launch_session();
        } else if terminal_only {
            self.launch_empty_session();
        }
        // A launch that was accepted wrote a session record; a refusal wrote
        // a notice instead, and the sheet stays to show it.
        if self.store.sessions.len() > sessions_before {
            self.launch_sheet_open = false;
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
        self.launch_sheet_is_git = self
            .selected_project()
            .is_some_and(|project| has_git_metadata(&project.path));
    }

    /// Past CLI conversations, found and restored. It lived under the launch
    /// form and moved to 履歴 (change 092): starting a session and importing
    /// an old one are two jobs.
    pub(crate) fn ui_history_cli_section(&mut self, ui: &mut egui::Ui) {
        let project_id = self
            .history_project
            .filter(|id| self.store.projects.iter().any(|project| project.id == *id))
            .or(self.selected_project)
            .or_else(|| self.store.projects.first().map(|project| project.id));
        let Some(project) = project_id
            .and_then(|id| self.store.projects.iter().find(|project| project.id == id))
            .cloned()
        else {
            return;
        };
        let projects: Vec<(Uuid, String)> = self
            .store
            .projects
            .iter()
            .map(|project| (project.id, project.name.clone()))
            .collect();
        egui::CollapsingHeader::new(tr("CLI セッション（過去の会話の検出と復元）"))
            .id_salt("history-cli")
            .default_open(false)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(tr("プロジェクト"));
                    egui::ComboBox::from_id_salt("history-project")
                        .selected_text(&project.name)
                        .show_ui(ui, |ui| {
                            for (id, name) in &projects {
                                if ui.selectable_label(*id == project.id, name).clicked()
                                    && *id != project.id
                                {
                                    self.history_project = Some(*id);
                                    // A note is written for one project's
                                    // conversation; it does not follow.
                                    self.handoff_note_input.clear();
                                }
                            }
                        });
                });
                self.ui_cli_sessions(ui, &project);
            });
        ui.separator();
    }

    pub(crate) fn ui_cli_sessions(&mut self, ui: &mut egui::Ui, project: &Project) {
        let palette = self.store.theme.palette();
        ui.horizontal(|ui| {
            ui.label(RichText::new(tr("CLI セッション")).strong());
            ui.label(
                RichText::new(tr("Operon の外で見つかったネイティブセッション"))
                    .small()
                    .weak(),
            );
            if icon_text_button(ui, ICON_SEARCH, tr("ローカル CLI セッションを検出"))
                .on_hover_text(tr("このプロジェクトのフォルダで会話を探す"))
                .clicked()
            {
                self.scan_cli_sessions(project);
            }
        });
        let remembered_count = self
            .store
            .native_session_references
            .iter()
            .filter(|reference| reference.project_id == project.id)
            .count();
        let recent_handoffs = self
            .store
            .cli_handoffs
            .iter()
            .filter(|handoff| handoff.project_id == project.id)
            .rev()
            .take(3)
            .cloned()
            .collect::<Vec<_>>();
        if remembered_count > 0 && !self.cli_sessions.contains_key(&project.id) {
            ui.label(
                RichText::new(tf!("ローカル参照 {remembered_count} 件を保存済み。更新すると最新の履歴が優先されます。", remembered_count = remembered_count))
                .small()
                .weak(),
            );
        }
        for handoff in recent_handoffs {
            ui.label(
                RichText::new(format!(
                    "{ICON_RESTORE} {}: {} → {} · {}",
                    if handoff.full_history {
                        tr("会話全履歴を復元")
                    } else {
                        tr("従来の handoff")
                    },
                    handoff.source_provider.label(),
                    agent_choice_copy(&handoff.target_agent).0,
                    relative_time(handoff.created_at)
                ))
                .size(12.0)
                .color(palette.text_muted),
            );
        }
        if let Some(cli_sessions) = self.cli_sessions.get(&project.id).cloned() {
            if cli_sessions.is_empty() {
                ui.label(
                    RichText::new(tr(
                        "このプロジェクトのローカル CLI セッションは見つかりませんでした。",
                    ))
                    .weak(),
                );
            }
            if !cli_sessions.is_empty() {
                ui.label(
                    RichText::new(tf!("{ICON_RESUME} 元の CLI で再開 · {ICON_RESTORE} 同じ CLI にも別の CLI にも全履歴を復元（トークン消費なし）", ICON_RESTORE = ICON_RESTORE, ICON_RESUME = ICON_RESUME))
                    .size(12.0)
                    .color(palette.text),
                );
                ui.add(
                    egui::TextEdit::multiline(&mut self.handoff_note_input)
                        .hint_text(tr("復元メモ（任意）"))
                        .desired_rows(2)
                        .desired_width(f32::INFINITY),
                );
            }
            for cli_session in cli_sessions.into_iter().take(12) {
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(cli_session.provider.label()).small().weak());
                        ui.label(
                            RichText::new(cli_session.title.as_deref().unwrap_or(&cli_session.id))
                                .strong(),
                        );
                        ui.label(
                            RichText::new(relative_time(cli_session.updated_at))
                                .small()
                                .weak(),
                        );
                        if icon_button(ui, palette, ICON_OPEN_EXTERNAL, tr("Finder で表示"))
                            .clicked()
                        {
                            let path = cli_session.path.clone();
                            self.request_system_action(tr("Finder で表示"), move || {
                                open_path(&path)
                            });
                        }
                        if is_safe_cli_session_id(&cli_session.id)
                            && icon_button(ui, palette, ICON_RESUME, tr("元の CLI で再開"))
                                .clicked()
                        {
                            self.resume_cli_session(project, &cli_session);
                        }
                    });
                    // The ID the CLI itself resumes with. A titled card used to
                    // hide it, which left the two conversations a person is
                    // choosing between looking identical — and it is the ID
                    // that the restore names back to them.
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = SPACE_XS;
                        ui.label(
                            RichText::new(tf!(
                                "現在の {p0} セッション ID",
                                p0 = cli_session.provider.label()
                            ))
                            .size(11.5)
                            .color(palette.text_faint),
                        );
                        ui.add(
                            egui::Label::new(
                                RichText::new(&cli_session.id)
                                    .size(11.5)
                                    .monospace()
                                    .color(palette.text_faint),
                            )
                            .truncate(),
                        );
                        resume_copy_button(ui, palette, cli_session.provider, &cli_session.id);
                    });
                    if let Some(branch) = &cli_session.branch {
                        ui.label(
                            RichText::new(format!("{ICON_BRANCH} {branch}"))
                                .size(12.5)
                                .color(palette.branch),
                        );
                    }
                    if let Some(message) = &cli_session.last_user_message {
                        ui.label(
                            RichText::new(tr("この会話の最後の依頼"))
                                .size(11.5)
                                .color(palette.text_faint),
                        );
                        ui.label(
                            RichText::new(truncate_chars(message, 160))
                                .size(12.5)
                                .color(palette.text),
                        );
                    }
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(tr("復元先"))
                                .size(12.0)
                                .color(palette.text_muted),
                        );
                        // Every card offers its own picker: the listed sessions
                        // can come from different CLIs, and one shared choice
                        // would keep flipping as their cards render.
                        let mut target = self
                            .handoff_target_inputs
                            .get(&cli_session.id)
                            .cloned()
                            .unwrap_or_else(|| {
                                default_handoff_target(cli_session.provider, &self.tools).to_owned()
                            });
                        egui::ComboBox::from_id_salt(("handoff-target", &cli_session.id))
                            .selected_text(restore_target_label(&target, cli_session.provider))
                            .show_ui(ui, |ui| {
                                // The CLI this conversation already lives in is
                                // on the list too: restoring there forks it into
                                // a second conversation of that CLI's own, which
                                // is how a person keeps a branch of the work
                                // without abandoning the one they are in.
                                for agent in FULL_HISTORY_RESTORE_TARGETS.iter().copied() {
                                    ui.selectable_value(
                                        &mut target,
                                        agent.to_owned(),
                                        restore_target_label(agent, cli_session.provider),
                                    );
                                }
                            });
                        // The picker beside it already names the destination, so
                        // the button says what happens rather than repeating it.
                        if is_safe_cli_session_id(&cli_session.id)
                            && icon_text_button(ui, ICON_RESTORE, tr("会話全履歴を復元"))
                                .on_hover_text(if target == cli_session.provider.agent() {
                                    tf!(
                                        "同じ {p0} に会話を複製します（トークン消費なし）",
                                        p0 = cli_session.provider.label()
                                    )
                                } else {
                                    tf!(
                                        "{p0} へ全履歴を移植します（トークン消費なし）",
                                        p0 = agent_choice_copy(&target).0
                                    )
                                })
                                .clicked()
                        {
                            self.handoff_cli_session(project, &cli_session, &target);
                        }
                        self.handoff_target_inputs
                            .insert(cli_session.id.clone(), target);
                    });
                });
                ui.add_space(4.0);
            }
        } else {
            ui.label(
                RichText::new(tr(
                    "Operon の外で起動したネイティブセッションを検出します。",
                ))
                .weak(),
            );
        }
    }

    pub(crate) fn session_card(&mut self, ui: &mut egui::Ui, session: &Session) {
        let palette = self.store.theme.palette();
        let failed_dependency = blocking_dependency_label(&self.store.sessions, session);
        let project_name = self
            .store
            .projects
            .iter()
            .find(|project| project.id == session.project_id)
            .map(|project| project.name.clone())
            .unwrap_or_else(|| tr("登録されていないプロジェクト").into());
        let status = self.status_view(session);
        let title = session_title(session);
        let mut removal_requested = false;
        let row = clickable_card(ui, palette, ("session-row", session.id), true, |ui| {
            ui.horizontal(|ui| {
                status_chip(ui, &status, 13.5, palette);
                ui.label(RichText::new(&title).strong());
                agent_chip(ui, &session.agent, 12.0, palette);
                ui.label(
                    RichText::new(relative_time(session.created_at))
                        .size(12.0)
                        .color(palette.text_muted),
                );
                ui.label(
                    RichText::new(format!("{ICON_PROJECT} {project_name}"))
                        .size(12.0)
                        .color(palette.text_muted),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if small_icon_button(ui, palette, ICON_CLOSE, tr("セッションを削除")).clicked()
                    {
                        removal_requested = true;
                    }
                });
            });
            ui.horizontal_wrapped(|ui| {
                // This row is read while deciding which session to look at, so
                // it carries one way in. Attaching from Terminal.app is a
                // choice about a session already open, and lives there.
                if icon_button(ui, palette, ICON_SHOW, tr("ターミナルを表示")).clicked() {
                    self.open_session_in_terminal(session.id);
                }
                if matches!(
                    session.status,
                    SessionStatus::Exited | SessionStatus::Failed
                ) && icon_button(
                    ui,
                    palette,
                    ICON_CLOSE,
                    if session.status == SessionStatus::Failed {
                        tr("残ったターミナルを閉じる")
                    } else {
                        tr("ターミナルを閉じる")
                    },
                )
                .clicked()
                {
                    self.close_completed_terminal(session.id);
                }
                // The state verb closes the row: same word, same place, on
                // every surface that acts on a session.
                if let Some(verb) = session_verb(session, self.cancellation_pending(session.id)) {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let response = verb_button(ui, palette, verb.icon, verb.label, verb.danger)
                            .on_hover_text(verb.hint);
                        if response.clicked() {
                            match verb.action {
                                SessionVerbAction::Stop => self.stop_session(session.id),
                                SessionVerbAction::Start => self.start_session(session.id),
                                SessionVerbAction::ResumeNative => {
                                    self.resume_managed_native_session(session.id)
                                }
                            }
                        }
                    });
                }
            });
            let goal = session_display_goal(&session.goal);
            if goal != title && !goal.trim().is_empty() {
                ui.label(RichText::new(goal).color(palette.text));
            }
            if let Some(branch) = &session.branch {
                ui.label(
                    RichText::new(format!("{ICON_BRANCH} {branch}"))
                        .size(12.5)
                        .color(palette.branch),
                );
            }
            ui.label(
                RichText::new(format!("tmux: {}", session.tmux_name))
                    .small()
                    .weak(),
            );
            if let Some(path) = &session.worktree_path {
                ui.label(
                    RichText::new(format!("cwd: {}", path.display()))
                        .small()
                        .weak(),
                );
            }
            if let (Some(provider), Some(native_session_id)) = (
                cli_provider_for_agent(&session.agent),
                &session.native_session_id,
            ) {
                // The command the 会話 ID から再開 button above will run, not a
                // shorter one that resembles it. `native_resume_display` answers
                // a different question — "how would I resume this from a shell",
                // which is what the two call sites for a conversation Operon did
                // not launch are asking — and it cannot carry launch options
                // because those sessions have no launch command.
                //
                // This one does, and printing it without them meant the line
                // that looks most like evidence was the one that dropped the
                // switch a person had acknowledged in writing: the hover said
                // 権限確認をすべてスキップする would be carried and the line
                // underneath spelled a command without it.
                ui.label(
                    RichText::new(format!(
                        "native resume: {}",
                        resume_command_with_launch_options(
                            provider,
                            &session.agent,
                            &session.agent_command,
                            native_session_id,
                        )
                    ))
                    .small()
                    .weak(),
                );
            } else if cli_provider_for_agent(&session.agent).is_some() {
                ui.label(
                    RichText::new(tr("ネイティブ再開 ID を待機中"))
                        .small()
                        .weak(),
                );
            }
            if !session.depends_on.is_empty() {
                ui.label(
                    RichText::new(tf!(
                        "{p0} 件のセッションに依存",
                        p0 = session.depends_on.len()
                    ))
                    .small()
                    .weak(),
                );
            }
            if let Some(parent) = &failed_dependency {
                ui.label(
                    RichText::new(tf!(
                        "ブロック中: 前提 “{parent}” が完了していません。",
                        parent = parent
                    ))
                    .small()
                    .color(palette.accent_soft),
                );
            }
            self.session_removal_confirmation(ui, session.id);
        });
        if removal_requested {
            self.request_session_removal(session.id);
        } else if row.response.clicked() {
            self.open_session_in_terminal(session.id);
        }
        ui.add_space(6.0);
    }

    pub(crate) fn ui_git(&mut self, ui: &mut egui::Ui, project: &Project) {
        let palette = self.store.theme.palette();
        ui.horizontal(|ui| {
            ui.heading("Git");
            for view in GitView::all() {
                if ui
                    .selectable_label(self.git_view == view, view.label())
                    .clicked()
                {
                    self.git_view = view;
                }
            }
            if icon_button(ui, palette, ICON_REFRESH, tr("更新")).clicked() {
                match self.git_view {
                    GitView::Changes => self.request_git_changes(project),
                    GitView::Diff => self.request_git_diff(project, self.git_selected_file.clone()),
                    GitView::History => self.request_git_history(project),
                }
            }
        });
        ui.separator();
        match self.git_view {
            GitView::Changes => {
                if !self.git_changes_cache.contains_key(&project.id) {
                    self.request_git_changes(project);
                }
                if self
                    .background_tasks
                    .contains(&BackgroundKey::GitChanges(project.id))
                {
                    ui.label(RichText::new(tr("Git の状態を更新しています…")).weak());
                }
                match self.git_changes_cache.get(&project.id).cloned() {
                    Some(Ok(snapshot)) => {
                        self.ensure_staged_selection(project.id, &snapshot.files);
                        ui.label(RichText::new(tr("変更されたファイル")).strong());
                        if snapshot.files.is_empty() {
                            ui.label(RichText::new(tr("作業ツリーに変更はありません。")).weak());
                        }
                        for file in &snapshot.files {
                            ui.horizontal(|ui| {
                                let mut ticked = self
                                    .git_staged_files
                                    .get(&project.id)
                                    .is_some_and(|selected| selected.contains(&file.path));
                                if ui
                                    .checkbox(&mut ticked, "")
                                    .on_hover_text(tr("次のコミットに含めます"))
                                    .changed()
                                {
                                    self.toggle_staged_file(project.id, &file.path, ticked);
                                }
                                let (mark, colour) = git_status_badge(&file.status, palette);
                                ui.label(RichText::new(mark).size(12.0).color(colour))
                                    .on_hover_text(git_status_wording(&file.status));
                                if ui
                                    .selectable_label(
                                        self.git_selected_file.as_deref()
                                            == Some(file.path.as_str()),
                                        &file.path,
                                    )
                                    .on_hover_text(tr("このファイルの差分だけを表示します"))
                                    .clicked()
                                {
                                    self.git_selected_file = Some(file.path.clone());
                                    self.git_view = GitView::Diff;
                                }
                                // Reviewing a change is usually the step before
                                // editing it, so the file is one click from the
                                // editor rather than a hunt through the tree.
                                if icon_button(ui, palette, ICON_EDIT, tr("エディタで開く"))
                                    .clicked()
                                {
                                    self.open_document(project, PathBuf::from(&file.path));
                                }
                            });
                        }
                        if !snapshot.files.is_empty() {
                            ui.add_space(SPACE_SM);
                            self.ui_commit_box(ui, palette, project);
                        }
                        ui.add_space(SPACE_SM);
                        self.ui_push_row(ui, palette, project);
                    }
                    Some(Err(error)) => {
                        ui.label(tf!("Git の状態を取得できません: {error}", error = error));
                    }
                    None => {}
                }
            }
            GitView::Diff => {
                let file = self.git_selected_file.clone();
                self.ensure_diff_parsed(project, file.clone());
                let key = (project.id, file.clone());
                let totals = self
                    .diff_file_cache
                    .get(&key)
                    .map(|files| (files.len(), diff_totals(files)));
                ui.horizontal(|ui| {
                    ui.label(RichText::new(tr("作業ツリーの差分")).strong());
                    if let Some((count, (added, removed))) = totals {
                        ui.label(
                            RichText::new(tf!("{count} ファイル", count = count))
                                .size(12.0)
                                .color(palette.text_muted),
                        );
                        diff_counts(ui, added, removed, palette);
                    }
                    if file.is_some() && ui.button(tr("すべて表示")).clicked() {
                        self.git_selected_file = None;
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let reviewing = self
                            .background_tasks
                            .contains(&BackgroundKey::AiDiffReview(project.id));
                        if reviewing {
                            ui.add_enabled(false, egui::Button::new(tr("AIレビュー中…")));
                        } else if ui.button(tr("AIレビュー")).clicked() {
                            self.request_ai_diff_review(project);
                        }
                    });
                });
                if let Some(review) = self.diff_ai_review.get(&project.id).cloned() {
                    ui.add_space(SPACE_XS);
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(tr("AIコードレビュー評価"))
                                    .strong()
                                    .color(palette.accent),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.button(tr("閉じる")).clicked() {
                                        self.diff_ai_review.remove(&project.id);
                                    }
                                    if ui.button(tr("コピー")).clicked() {
                                        ui.ctx().copy_text(review.clone());
                                        self.notice = Some(
                                            tr("AIレビュー結果をクリップボードにコピーしました。")
                                                .into(),
                                        );
                                    }
                                },
                            );
                        });
                        egui::ScrollArea::vertical()
                            .id_salt("ai_diff_review_scroll")
                            .max_height(140.0)
                            .show(ui, |ui| {
                                code_block(ui, &review);
                            });
                    });
                }
                if self
                    .background_tasks
                    .contains(&BackgroundKey::GitDiff(project.id, file))
                {
                    ui.label(RichText::new(tr("差分を読み込んでいます…")).weak());
                }
                if let Some(Err(error)) = self.git_diff_cache.get(&key) {
                    ui.label(tf!("Git の差分を取得できません: {error}", error = error));
                    return;
                }
                if let Some(files) = self.diff_file_cache.get(&key) {
                    // The footer takes a row when there is anything to say, so
                    // the pane above it is measured with that row already spent
                    // rather than being pushed off the bottom by it.
                    let footer = if self.diff_annotations.count(project.id) > 0 {
                        34.0
                    } else {
                        0.0
                    };
                    let height = (ui.available_height() - 6.0 - footer).max(160.0);
                    if diff_pane(
                        ui,
                        files,
                        &mut self.collapsed_diff_files,
                        palette,
                        height,
                        project.id,
                        &mut self.diff_annotations,
                    ) {
                        self.diff_comments_dirty = true;
                    }
                }
                self.ui_diff_comment_footer(ui, palette, project.id);
            }
            GitView::History => {
                ui.label(RichText::new(tr("最近のコミット")).strong());
                if !self.git_history_cache.contains_key(&project.id) {
                    self.request_git_history(project);
                }
                if self
                    .background_tasks
                    .contains(&BackgroundKey::GitHistory(project.id))
                {
                    ui.label(RichText::new(tr("履歴を読み込んでいます…")).weak());
                }
                match self.git_history_cache.get(&project.id).cloned() {
                    Some(Ok(history)) => code_block(ui, &history),
                    Some(Err(error)) => {
                        ui.label(tf!("Git の履歴を取得できません: {error}", error = error));
                    }
                    None => {}
                }
            }
        }
    }

    pub(crate) fn ui_worktrees(&mut self, ui: &mut egui::Ui, project: &Project) {
        let palette = self.store.theme.palette();
        let mutation_running = self.worktree_mutation_running(project.id);
        ui.horizontal(|ui| {
            ui.heading("worktree");
            if icon_button(ui, palette, ICON_REFRESH, tr("更新")).clicked() {
                self.request_worktrees(project);
            }
        });
        ui.label(tr(
            "別ブランチの作業フォルダで、複数のセッションを並行できます。",
        ));
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.worktree_branch_input)
                    .hint_text(tr("新しいブランチ名（例: feature/my-change）"))
                    .desired_width(340.0),
            );
            if ui
                .add_enabled(
                    !mutation_running,
                    egui::Button::new(tf!("{ICON_ADD} worktree を作成", ICON_ADD = ICON_ADD)),
                )
                .on_disabled_hover_text(tr("worktree 操作の完了待ち"))
                .clicked()
            {
                self.create_worktree();
            }
        });
        ui.label(
            RichText::new(tr(
                "作成した worktree が次のセッションの作業フォルダになります。",
            ))
            .small()
            .weak(),
        );
        ui.add_space(10.0);
        if !self.worktree_cache.contains_key(&project.id) {
            self.request_worktrees(project);
        }
        if self
            .background_tasks
            .contains(&BackgroundKey::Worktrees(project.id))
        {
            ui.label(RichText::new(tr("worktree を読み込んでいます…")).weak());
        }
        match self.worktree_cache.get(&project.id).cloned() {
            Some(Ok(worktrees)) => {
                for worktree in worktrees {
                    egui::Frame::group(ui.style()).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(if worktree.is_main {
                                tf!(
                                    "{ICON_TREE_MAIN} メインの作業ツリー",
                                    ICON_TREE_MAIN = ICON_TREE_MAIN
                                )
                            } else {
                                tf!(
                                    "{ICON_TREE_EXTRA} 追加の worktree",
                                    ICON_TREE_EXTRA = ICON_TREE_EXTRA
                                )
                            });
                            ui.label(
                                RichText::new(format!(
                                    "{ICON_BRANCH} {}",
                                    worktree
                                        .branch
                                        .as_deref()
                                        .unwrap_or(tr("detached HEAD（ブランチ未設定）"))
                                ))
                                .strong()
                                .size(13.5)
                                .color(palette.branch),
                            );
                            if icon_text_button(ui, ICON_ADD, tr("この場所で新規セッション"))
                                .clicked()
                            {
                                // The project first: selecting a different one
                                // clears the chosen path.
                                self.open_project_session_setup(project.id);
                                if worktree.is_main {
                                    self.session_path_input.clear();
                                    self.session_path_project = None;
                                } else {
                                    self.session_path_input = worktree.path.display().to_string();
                                    self.session_path_project = Some(project.id);
                                }
                            }
                            if icon_button(ui, palette, ICON_OPEN_EXTERNAL, tr("Finder で表示"))
                                .clicked()
                            {
                                let path = worktree.path.clone();
                                self.request_system_action(tr("Finder で表示"), move || {
                                    reveal_path(&path)
                                });
                            }
                            ui.menu_button(tr("エディタで開く"), |ui| {
                                if ui.button("VS Code").clicked() {
                                    let _ = open_in_external_editor(
                                        &worktree.path,
                                        ExternalEditor::VsCode,
                                    );
                                    ui.close_menu();
                                }
                                if ui.button("Cursor").clicked() {
                                    let _ = open_in_external_editor(
                                        &worktree.path,
                                        ExternalEditor::Cursor,
                                    );
                                    ui.close_menu();
                                }
                                if ui.button("Zed").clicked() {
                                    let _ = open_in_external_editor(
                                        &worktree.path,
                                        ExternalEditor::Zed,
                                    );
                                    ui.close_menu();
                                }
                                if ui.button("Finder").clicked() {
                                    let _ = open_in_external_editor(
                                        &worktree.path,
                                        ExternalEditor::Finder,
                                    );
                                    ui.close_menu();
                                }
                            });
                            if !worktree.is_main {
                                if let Some(ref branch) = worktree.branch {
                                    if ui.button(tr("main にマージ…")).clicked() {
                                        self.pending_land_worktree = Some((
                                            project.id,
                                            worktree.path.clone(),
                                            branch.clone(),
                                        ));
                                    }
                                }
                                if ui.button(tr("削除…")).clicked() {
                                    self.pending_worktree_removal = Some(worktree.path.clone());
                                }
                            }
                        });
                        ui.monospace(worktree.path.display().to_string());
                    });
                    ui.add_space(6.0);
                }
            }
            Some(Err(error)) => {
                ui.label(tf!("worktree を取得できません: {error}", error = error));
            }
            None => {}
        }
        self.ui_setup_prompt(ui, palette);
        if let Some(path) = self.pending_worktree_removal.clone() {
            ui.add_space(10.0);
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.label(
                    RichText::new(tr("この worktree を削除しますか？"))
                        .strong()
                        .color(palette.accent_soft),
                );
                ui.label(tr(
                    "未コミットの変更があると Git が拒否します。ブランチは残ります。",
                ));
                ui.monospace(path.display().to_string());
                ui.horizontal(|ui| {
                    if ui.button(tr("キャンセル")).clicked() {
                        self.pending_worktree_removal = None;
                    }
                    if ui
                        .button(RichText::new(tr("worktree を削除")).color(palette.danger))
                        .clicked()
                    {
                        self.remove_worktree(project, &path);
                    }
                });
            });
        }
    }

    pub(crate) fn ui_pull_requests(&mut self, ui: &mut egui::Ui, project: &Project) {
        let palette = self.store.theme.palette();
        ui.horizontal(|ui| {
            ui.heading(tr("プルリクエスト"));
            ui.label(RichText::new(tr("認証済み GitHub CLI 経由")).small().weak());
            if icon_button(ui, palette, ICON_REFRESH, tr("更新")).clicked() {
                self.request_pull_requests(project);
            }
        });
        if !self.tools.gh {
            ui.label(tr("GitHub CLI を利用できません。次のコマンドでインストールしてください: brew install gh"));
            return;
        }
        if !self.pull_request_cache.contains_key(&project.id) {
            self.request_pull_requests(project);
        }
        if self
            .background_tasks
            .contains(&BackgroundKey::PullRequests(project.id))
        {
            ui.label(RichText::new(tr("プルリクエストを読み込んでいます…")).weak());
        }
        match self.pull_request_cache.get(&project.id).cloned() {
            Some(Ok(pull_requests)) if pull_requests.is_empty() => {
                ui.label(RichText::new(tr("オープン中のプルリクエストはありません。")).weak());
            }
            Some(Ok(pull_requests)) => {
                for pull_request in pull_requests {
                    egui::Frame::group(ui.style()).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(format!("#{}", pull_request.number)).strong());
                            ui.label(&pull_request.title);
                            if pull_request.is_draft {
                                ui.label(
                                    RichText::new(tr("下書き"))
                                        .size(12.0)
                                        .color(palette.text_muted),
                                );
                            }
                            if icon_button(ui, palette, ICON_OPEN_EXTERNAL, tr("ブラウザで開く"))
                                .clicked()
                            {
                                let url = pull_request.url.clone();
                                self.request_system_action(
                                    tr("ブラウザで開く"),
                                    move || open_url(&url),
                                );
                            }
                        });
                        let checks = summarize_checks(&pull_request.checks);
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!(
                                    "{ICON_BRANCH} {}",
                                    pull_request.head_ref_name
                                ))
                                .size(12.5)
                                .color(palette.branch),
                            );
                            ui.label(
                                RichText::new(tf!(
                                    "レビュー: {p0} · チェック: {checks}",
                                    checks = checks,
                                    p0 = review_decision_label(
                                        pull_request.review_decision.as_deref()
                                    )
                                ))
                                .size(12.5)
                                .color(palette.text),
                            );
                        });
                    });
                    ui.add_space(6.0);
                }
            }
            Some(Err(error)) => {
                ui.label(tf!(
                    "プルリクエストを読み込めませんでした: {error}",
                    error = error
                ));
            }
            None => {}
        }
    }

    /// The editor: a project tree down the left, and whatever is in the front
    /// tab on the right, shown as text, as rendered Markdown, or as its diff.
    pub(crate) fn ui_files(&mut self, ui: &mut egui::Ui, project: &Project) {
        let palette = self.store.theme.palette();
        // Read the save shortcut here rather than beside the other global keys,
        // so that ⌘S only means "save" while there is something open to save.
        if ui.input_mut(|input| {
            input.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::S,
            ))
        }) {
            self.save_active_document();
        }
        if !self.file_cache.contains_key(&project.id) {
            self.request_files(project);
        }
        // The badges in the tree come from the same snapshot the Git tab reads,
        // so opening the editor is a reason to have one.
        self.refresh_editor_git_views(project);
        // On the same tick and for the same reason: an agent's write is the
        // event both of these exist to notice.
        self.watch_active_document(project);
        let height = (ui.available_height() - 6.0).max(240.0);
        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(268.0, height),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| self.ui_file_tree(ui, project, palette, height),
            );
            ui.separator();
            let width = ui.available_width();
            ui.allocate_ui_with_layout(
                egui::vec2(width, height),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| self.ui_editor(ui, project, palette, height),
            );
        });
    }

    pub(crate) fn ui_file_tree(
        &mut self,
        ui: &mut egui::Ui,
        project: &Project,
        palette: &'static Palette,
        height: f32,
    ) {
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.file_search)
                    .hint_text(tr("ファイルを絞り込む"))
                    .desired_width(196.0),
            );
            if icon_button(
                ui,
                palette,
                ICON_REFRESH,
                tr("ファイル一覧と Git の状態を読み直す"),
            )
            .clicked()
            {
                self.request_files(project);
                self.invalidate_project_file_views(project.id);
                self.file_tree_cache = None;
            }
        });
        if self
            .background_tasks
            .contains(&BackgroundKey::Files(project.id))
            && !self.file_cache.contains_key(&project.id)
        {
            ui.label(
                RichText::new(tr("プロジェクトを読み込んでいます…"))
                    .size(12.0)
                    .color(palette.text_muted),
            );
        }
        let Some(scan) = self.file_cache.get(&project.id) else {
            return;
        };
        if let Some(warning) = file_scan_warning(scan) {
            ui.label(RichText::new(warning).size(11.5).color(palette.accent_soft));
        }
        let filter = self.file_search.to_lowercase();
        let stale = match &self.file_tree_cache {
            Some((cached, cached_filter, _)) => *cached != project.id || *cached_filter != filter,
            None => true,
        };
        if stale {
            let matching = if filter.is_empty() {
                scan.paths.clone()
            } else {
                scan.paths
                    .iter()
                    .filter(|path| path.to_string_lossy().to_lowercase().contains(&filter))
                    .cloned()
                    .collect()
            };
            self.file_tree_cache = Some((project.id, filter.clone(), build_file_tree(&matching)));
        }
        let Some((_, _, tree)) = &self.file_tree_cache else {
            return;
        };
        if tree.is_empty() {
            ui.label(
                RichText::new(if filter.is_empty() {
                    tr("読み取り可能なファイルはありません。")
                } else {
                    tr("一致するファイルはありません。")
                })
                .size(12.0)
                .color(palette.text_muted),
            );
            return;
        }
        let badges = self.git_status_badges(project.id);
        let mut actions = Vec::new();
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .max_height(height - 44.0)
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 1.0;
                file_tree_rows(
                    ui,
                    tree,
                    project.id,
                    &self.expanded_directories,
                    &badges,
                    // The tree shows the project, so a document rooted in a
                    // worktree highlights nothing here — it has the same
                    // relative path and is a different file.
                    self.tree_highlighted_document(),
                    // While a filter is on, every directory is open: a person
                    // filtering has asked to see the matches, not to go and
                    // find which folders they are in.
                    !filter.is_empty(),
                    0,
                    palette,
                    &mut actions,
                );
            });
        for action in actions {
            match action {
                FileTreeAction::Open(path) => self.open_document(project, path),
                FileTreeAction::Toggle(path) => {
                    let key = (project.id, path);
                    if !self.expanded_directories.remove(&key) {
                        self.expanded_directories.insert(key);
                    }
                }
            }
        }
    }

    pub(crate) fn ui_editor(
        &mut self,
        ui: &mut egui::Ui,
        project: &Project,
        palette: &'static Palette,
        height: f32,
    ) {
        self.ui_editor_tabs(ui, project, palette);
        let Some(document) = self.active_document.clone() else {
            ui.add_space(28.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new(ICON_FILE)
                        .size(28.0)
                        .color(palette.text_faint),
                );
                ui.label(
                    RichText::new(tr("左のツリーからファイルを開いてください。"))
                        .color(palette.text_muted),
                );
                ui.label(
                    RichText::new(tr(
                        "Markdown はプレビュー、変更されたファイルは差分で開けます。",
                    ))
                    .size(12.0)
                    .color(palette.text_faint),
                );
            });
            return;
        };
        if document.project != project.id {
            // The tab belongs to another project. Selecting a project already
            // switches the tree; the editor follows on the next click.
            ui.label(
                RichText::new(tr("開いているファイルは別のプロジェクトのものです。"))
                    .color(palette.text_muted),
            );
            return;
        }
        let Some(index) = self.document_index(&document) else {
            return;
        };
        let path = document.path.clone();
        let markdown = is_markdown(&path);
        let is_img = is_image(&path) || self.open_documents[index].image_data.is_some();
        let loading = self
            .background_tasks
            .contains(&BackgroundKey::FileOpen(document.clone()));
        let saving = self
            .background_tasks
            .contains(&BackgroundKey::FileSave(document.clone()));
        let modified = self.open_documents[index].modified();
        let read_only = self.open_documents[index].read_only.clone();
        let save_error = self.open_documents[index].save_error.clone();
        let external_change = self.open_documents[index].external_change;

        // An image document is rendered visually in Preview mode and cannot be edited.
        if is_img && self.editor_view != EditorView::Preview {
            self.editor_view = EditorView::Preview;
        }

        // The diff is read from the project's working tree, and its cache is
        // keyed by the project and the relative path. A document rooted in a
        // worktree has the same relative path as the project's own copy and
        // different contents, so a diff drawn for it would be the project's
        // file — usually "変更されていません" for a file the agent just wrote,
        // and with the two sharing one cache entry either way. Change 034 made
        // such a document reachable, so it is refused here rather than shown
        // wrong; threading the root through would also have to re-key change
        // 020's notes, which is its own change.
        let project_rooted = self.document_may_have_a_diff(index);
        if !project_rooted && self.editor_view == EditorView::Diff {
            self.editor_view = EditorView::Edit;
        }
        ui.horizontal(|ui| {
            for view in [EditorView::Edit, EditorView::Preview, EditorView::Diff] {
                // A file that is not Markdown and not an image has nothing to preview: the
                // rendered form and the source would be the same text.
                if view == EditorView::Preview && !markdown && !is_img {
                    continue;
                }
                // An image cannot be edited as text or shown as unified text diff.
                if is_img && view != EditorView::Preview {
                    continue;
                }
                if view == EditorView::Diff && !project_rooted {
                    continue;
                }
                if ui
                    .selectable_label(
                        self.editor_view == view,
                        format!("{}  {}", view.icon(), view.label()),
                    )
                    .clicked()
                {
                    self.editor_view = view;
                    if view == EditorView::Diff {
                        self.invalidate_open_editor_diff(project.id);
                    }
                }
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if icon_button(ui, palette, ICON_OPEN_EXTERNAL, tr("既定のアプリで開く")).clicked()
                {
                    let full = self.open_documents[index].absolute();
                    self.request_system_action(tr("ファイルを開く"), move || {
                        open_path(&full)
                    });
                }
                if icon_button(
                    ui,
                    palette,
                    ICON_REFRESH,
                    tr("ディスクから読み直す（編集内容は破棄されます）"),
                )
                .clicked()
                {
                    self.reload_document(&document);
                }
                if saving {
                    ui.label(
                        RichText::new(tr("保存しています…"))
                            .size(12.0)
                            .color(palette.text_muted),
                    );
                } else if read_only.is_none()
                    && ui
                        .add_enabled(
                            modified,
                            egui::Button::new(tf!("{ICON_SAVE}  保存", ICON_SAVE = ICON_SAVE)),
                        )
                        .on_hover_text("⌘S")
                        .clicked()
                {
                    self.save_active_document();
                }
            });
        });
        if let Some(error) = &save_error {
            egui::Frame::group(ui.style())
                .fill(palette.diff_removed_bg)
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new(ICON_ATTENTION).color(palette.diff_removed));
                        ui.label(RichText::new(error).color(palette.text));
                    });
                });
        }
        // Under the view switch and above the content, in the frame the save
        // error already uses: the two are the same kind of statement about the
        // same document, and when both are true both are shown — the save
        // error first, because it is about something the person just tried.
        if external_change {
            egui::Frame::group(ui.style())
                .fill(palette.card)
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new(ICON_ATTENTION).color(palette.warning));
                        ui.label(
                            RichText::new(tr(
                                "このファイルはエディタの外で変更されました。保存すると、その変更を上書きします。",
                            ))
                            .color(palette.text),
                        );
                        if ui.button(tr("読み直す")).clicked() {
                            self.reload_document(&document);
                        }
                        // The diff of the working tree against the last commit,
                        // which is where the write that raised this bar is —
                        // for a document rooted at the project. For one rooted
                        // in a worktree it would be somebody else's file, so
                        // the way in is not offered at all.
                        if project_rooted && ui.button(tr("差分を見る")).clicked() {
                            self.editor_view = EditorView::Diff;
                            self.invalidate_open_editor_diff(project.id);
                        }
                    });
                });
        }
        if loading {
            ui.label(
                RichText::new(tr("読み込んでいます…"))
                    .size(12.0)
                    .color(palette.text_muted),
            );
            return;
        }
        if let Some(reason) = read_only {
            ui.add_space(12.0);
            ui.label(RichText::new(reason).color(palette.text_muted));
            return;
        }
        let remaining = (height - 84.0).max(120.0);
        match self.editor_view {
            EditorView::Edit => self.ui_editor_text(ui, index, palette, remaining),
            EditorView::Preview => {
                if let Some(image_data) = self.open_documents[index].image_data.clone() {
                    self.ui_editor_image_preview(ui, &document, &image_data, palette, remaining);
                } else {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .max_height(remaining)
                        .show(ui, |ui| {
                            let document = self
                                .markdown_preview_cache
                                .document_for(&self.open_documents[index].buffer, palette);
                            markdown_view(ui, document, palette);
                        });
                }
            }
            EditorView::Diff => {
                let file = Some(path.to_string_lossy().into_owned());
                self.ensure_diff_parsed(project, file.clone());
                let key = (project.id, file);
                if let Some(Err(error)) = self.git_diff_cache.get(&key) {
                    ui.label(tf!("差分を取得できません: {error}", error = error));
                    return;
                }
                if let Some(files) = self.diff_file_cache.get(&key) {
                    if files.is_empty() {
                        ui.add_space(12.0);
                        ui.label(
                            RichText::new(tr(
                                "このファイルは、最後のコミットから変更されていません。",
                            ))
                            .color(palette.text_muted),
                        );
                        return;
                    }
                    // The footer takes a row when there is anything to say, so
                    // the pane above it is measured with that row already spent.
                    let footer = if self.diff_annotations.count(project.id) > 0 {
                        34.0
                    } else {
                        0.0
                    };
                    if diff_pane(
                        ui,
                        files,
                        &mut self.collapsed_diff_files,
                        palette,
                        (remaining - footer).max(120.0),
                        project.id,
                        &mut self.diff_annotations,
                    ) {
                        self.diff_comments_dirty = true;
                    }
                    // Every pane that can take a note carries the way to send
                    // one. A note written here and sendable only from the Git
                    // tab is a note somebody writes and then cannot find.
                    self.ui_diff_comment_footer(ui, palette, project.id);
                } else {
                    ui.label(
                        RichText::new(tr("差分を読み込んでいます…"))
                            .size(12.0)
                            .color(palette.text_muted),
                    );
                }
            }
        }
    }

    pub(crate) fn ui_editor_image_preview(
        &mut self,
        ui: &mut egui::Ui,
        document: &DocumentId,
        image_data: &ImageDocumentData,
        palette: &Palette,
        height: f32,
    ) {
        let Some(texture) = document_image_texture(ui.ctx(), document, &image_data.bytes) else {
            ui.add_space(20.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new(tr("画像データを読み込めませんでした。"))
                        .size(13.5)
                        .color(palette.text_muted),
                );
            });
            return;
        };

        let orig_w = image_data.width as f32;
        let orig_h = image_data.height as f32;
        let avail_w = (ui.available_width() - 32.0).max(100.0);
        let avail_h = (height - 48.0).max(100.0);

        let scale = if orig_w > 0.0 && orig_h > 0.0 {
            (avail_w / orig_w).min(avail_h / orig_h).min(1.0)
        } else {
            1.0
        };
        let display_w = (orig_w * scale).max(1.0);
        let display_h = (orig_h * scale).max(1.0);

        egui::ScrollArea::both()
            .id_salt(("editor-image-preview-scroll", document))
            .auto_shrink([false, false])
            .max_height(height)
            .show(ui, |ui| {
                ui.add_space(8.0);
                ui.vertical_centered(|ui| {
                    egui::Frame::default()
                        .fill(palette.raised)
                        .stroke(egui::Stroke::new(1.0, palette.border_subtle))
                        .corner_radius(egui::CornerRadius::same(RADIUS_CARD))
                        .inner_margin(egui::Margin::same(12))
                        .show(ui, |ui| {
                            ui.add(egui::Image::new((
                                texture.id(),
                                egui::vec2(display_w, display_h),
                            )));
                        });
                    ui.add_space(8.0);
                    let zoom_pct = (scale * 100.0).round() as i32;
                    let size_kb = image_data.bytes.len().div_ceil(1024);
                    ui.label(
                        RichText::new(format!(
                            "{} × {} px ({zoom_pct}%) · {} KiB",
                            image_data.width, image_data.height, size_kb
                        ))
                        .size(11.5)
                        .color(palette.text_muted),
                    );
                });
            });
    }

    /// The editable pane: a gutter of line numbers beside the text itself.
    ///
    /// Neither side wraps. A wrapped line would put the numbers out of step
    /// with the lines they count, and a line number that points at the wrong
    /// line is worse than none — so the pane scrolls sideways, as an editor
    /// with line numbers has to.
    pub(crate) fn ui_editor_text(
        &mut self,
        ui: &mut egui::Ui,
        index: usize,
        palette: &'static Palette,
        height: f32,
    ) {
        let language = editor_language(&self.open_documents[index].path);
        let (lines, widest, numbers) = {
            let layout = self.editor_text_layout_cache.layout_for(
                &self.open_documents[index].buffer,
                language,
                palette,
            );
            (
                layout.line_count,
                layout.widest_line,
                layout.line_numbers.clone(),
            )
        };
        let cell = monospace_cell_size(ui);
        // How far the text scrolls sideways, and how wide the gutter has to be
        // for its largest number. A monospaced digit is exactly as wide as a
        // space, so padding each number to the same width right-aligns the
        // column without a layout having to.
        let digits = lines.to_string().len();
        let gutter = cell.x * digits as f32 + 8.0;
        let _text_width = (widest as f32 * cell.x + 12.0).max(ui.available_width() - gutter - 24.0);
        let mut layouter = |ui: &egui::Ui, text: &str, _width: f32| {
            let job = self
                .editor_text_layout_cache
                .layout_for(text, language, palette)
                .syntax
                .clone();
            ui.fonts(|fonts| fonts.layout_job(job))
        };
        // A jump asked for by a click on a stack trace. Spent by this frame, so
        // scrolling by hand afterwards is not undone on the next one; a third
        // of the way down rather than at the very top, so the lines above the
        // named one are readable.
        let jump = self
            .editor_jump_line
            .take()
            .map(|line| (line.saturating_sub(1) as f32 * cell.y - height / 3.0).max(0.0));
        egui::Frame::new()
            .fill(palette.inset)
            .stroke(egui::Stroke::new(1.0, palette.border_subtle))
            .inner_margin(egui::Margin::same(4))
            .show(ui, |ui| {
                let mut area = egui::ScrollArea::both()
                    .auto_shrink([false, false])
                    .max_height(height);
                if let Some(offset) = jump {
                    area = area.vertical_scroll_offset(offset);
                }
                area.show(ui, |ui| {
                    ui.horizontal_top(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        ui.allocate_ui_with_layout(
                            egui::vec2(gutter, cell.y * lines as f32),
                            egui::Layout::top_down(egui::Align::RIGHT),
                            |ui| {
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(numbers)
                                            .monospace()
                                            .size(MONOSPACE_SIZE)
                                            .color(palette.text_muted),
                                    )
                                    .wrap_mode(egui::TextWrapMode::Extend),
                                );
                            },
                        );
                        ui.add_space(8.0);
                        ui.add(
                            egui::TextEdit::multiline(&mut self.open_documents[index].buffer)
                                .code_editor()
                                .frame(false)
                                .margin(egui::Margin::ZERO)
                                .desired_width(f32::INFINITY)
                                .desired_rows(lines)
                                .layouter(&mut layouter),
                        );
                    });
                });
            });
    }

    /// The tab strip. Tabs are per project, so switching projects does not
    /// leave another project's files sitting above this one's tree.
    pub(crate) fn ui_editor_tabs(
        &mut self,
        ui: &mut egui::Ui,
        project: &Project,
        palette: &'static Palette,
    ) {
        let documents = self
            .open_documents
            .iter()
            .filter(|document| document.project == project.id)
            .map(|document| {
                let id = document.id();
                (
                    id.clone(),
                    document.modified(),
                    self.background_tasks.contains(&BackgroundKey::FileSave(id)),
                    // Decided when the document was opened, not here: this
                    // row draws every frame.
                    document.branch.clone(),
                )
            })
            .collect::<Vec<_>>();
        if documents.is_empty() {
            return;
        }
        let mut close = None;
        let mut select = None;
        egui::ScrollArea::horizontal()
            .auto_shrink([false, true])
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    for (document, modified, saving, branch) in &documents {
                        let active = self.active_document.as_ref() == Some(document);
                        let path = &document.path;
                        let name = path
                            .file_name()
                            .map(|name| name.to_string_lossy().into_owned())
                            .unwrap_or_else(|| path.display().to_string());
                        egui::Frame::new()
                            .fill(if active { palette.raised } else { palette.card })
                            .stroke(egui::Stroke::new(
                                1.0,
                                if active {
                                    palette.accent
                                } else {
                                    palette.border_subtle
                                },
                            ))
                            .inner_margin(egui::Margin::symmetric(8, 4))
                            .show(ui, |ui| {
                                ui.spacing_mut().item_spacing.x = 5.0;
                                let label = ui.add(
                                    egui::Label::new(
                                        RichText::new(format!("{}  {name}", file_tree_icon(&name)))
                                            .size(12.5)
                                            .color(if active {
                                                palette.text_strong
                                            } else {
                                                palette.text
                                            }),
                                    )
                                    .sense(egui::Sense::click()),
                                );
                                if label.on_hover_text(document.absolute_display()).clicked() {
                                    select = Some(document.clone());
                                }
                                // Which copy this is. Drawn only for a document
                                // that is not the project's own, so an ordinary
                                // tab looks exactly as it did.
                                if let Some(branch) = branch {
                                    ui.label(
                                        RichText::new(branch).size(11.5).color(palette.text_muted),
                                    );
                                }
                                if *saving {
                                    ui.label(
                                        RichText::new(ICON_PENDING)
                                            .size(11.5)
                                            .color(palette.text_muted),
                                    );
                                } else if *modified {
                                    ui.label(
                                        RichText::new(ICON_UNSAVED)
                                            .size(11.5)
                                            .color(palette.accent_text),
                                    )
                                    .on_hover_text(tr("未保存の変更があります"));
                                }
                                if small_icon_button(ui, palette, ICON_CLOSE, tr("タブを閉じる"))
                                    .clicked()
                                {
                                    close = Some(document.clone());
                                }
                            });
                    }
                });
            });
        if let Some(document) = select {
            self.active_document = Some(document);
        }
        if let Some(document) = close {
            // Closing a tab with unsaved edits would throw them away without
            // asking, so it does not: the tab stays and says why.
            if self
                .document_index(&document)
                .is_some_and(|index| self.open_documents[index].modified())
            {
                self.active_document = Some(document);
                self.notice = Some(
                    tr("未保存の変更があります。保存するか、ディスクから読み直してから閉じてください。")
                        .into(),
                );
            } else {
                self.close_document(&document);
            }
        }
    }

    pub(crate) fn ui_skills(&mut self, ui: &mut egui::Ui, project: &Project) {
        let palette = self.store.theme.palette();
        ui.horizontal(|ui| {
            ui.heading(tr("スキル"));
            if icon_button(ui, palette, ICON_REFRESH, tr("更新")).clicked() {
                self.request_skills(project);
            }
        });
        ui.label(tr(
            "プロジェクトと各エージェントのフォルダで見つかった SKILL.md です。",
        ));
        if !self.skill_cache.contains_key(&project.id) {
            self.request_skills(project);
        }
        if self
            .background_tasks
            .contains(&BackgroundKey::Skills(project.id))
        {
            ui.label(RichText::new(tr("スキルを検索しています…")).weak());
        }
        let skills = self.skill_cache.get(&project.id).cloned();
        if let Some(warning) = skills.as_ref().and_then(file_scan_warning) {
            ui.label(RichText::new(warning).small().color(palette.accent_soft));
        }
        if skills.as_ref().is_some_and(|scan| scan.paths.is_empty()) {
            ui.label(RichText::new(tr("SKILL.md は見つかりませんでした。")).weak());
        }
        for skill in skills.unwrap_or_default().paths {
            ui.horizontal(|ui| {
                ui.label(ICON_SKILL);
                ui.monospace(skill.display().to_string());
                if icon_button(ui, palette, ICON_SHOW, tr("「ファイル」タブで開く")).clicked()
                {
                    self.selected_file = Some(skill);
                    self.project_tab = ProjectTab::Files;
                }
            });
        }
    }

    pub(crate) fn ui_rules(&mut self, ui: &mut egui::Ui, project: &Project) {
        let palette = self.store.theme.palette();
        ui.horizontal(|ui| {
            ui.heading(tr("ルールと指示"));
            if icon_button(ui, palette, ICON_REFRESH, tr("更新")).clicked() {
                self.request_rules(project);
            }
        });
        if !self.rule_cache.contains_key(&project.id) {
            self.request_rules(project);
        }
        if self
            .background_tasks
            .contains(&BackgroundKey::Rules(project.id))
        {
            ui.label(RichText::new(tr("指示ファイルを検索しています…")).weak());
        }
        let rules = self.rule_cache.get(&project.id).cloned();
        if let Some(warning) = rules.as_ref().and_then(file_scan_warning) {
            ui.label(RichText::new(warning).small().color(palette.accent_soft));
        }
        if rules.as_ref().is_some_and(|scan| scan.paths.is_empty()) {
            ui.label(
                RichText::new(tr("プロジェクトの指示ファイルは見つかりませんでした。")).weak(),
            );
        }
        for rule in rules.unwrap_or_default().paths {
            ui.horizontal(|ui| {
                ui.label("≡");
                ui.monospace(rule.display().to_string());
                if icon_button(ui, palette, ICON_SHOW, tr("「ファイル」タブで開く")).clicked()
                {
                    self.selected_file = Some(rule);
                    self.project_tab = ProjectTab::Files;
                }
            });
        }
    }

    fn ui_sessions(&mut self, ui: &mut egui::Ui) {
        let palette = self.store.theme.palette();
        ui.horizontal(|ui| {
            ui.heading(tr("セッション"));
            if icon_button(ui, palette, ICON_REFRESH, tr("更新")).clicked() {
                self.request_session_poll();
            }
            for (view, label) in [
                (SessionView::List, tr("プロジェクト別")),
                (SessionView::Grid, tr("グリッド")),
            ] {
                if ui
                    .selectable_label(self.session_view == view, label)
                    .clicked()
                {
                    self.session_view = view;
                }
            }
            ui.add(
                egui::TextEdit::singleline(&mut self.search)
                    .hint_text(tr("セッションを検索"))
                    .desired_width(260.0),
            );
        });
        ui.horizontal(|ui| {
            ui.label(tr("ローカル履歴"));
            let response = ui.add(
                egui::TextEdit::singleline(&mut self.transcript_search_input)
                    .hint_text(tr("Codex と Claude の履歴を検索"))
                    .desired_width(330.0),
            );
            let submit =
                response.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));
            if icon_text_button(ui, ICON_SEARCH, tr("履歴を検索"))
                .on_hover_text(tr("ローカル履歴を全文検索"))
                .clicked()
                || submit
            {
                self.search_local_transcripts();
            }
        });
        if !self.transcript_matches.is_empty() {
            ui.add_space(8.0);
            ui.label(RichText::new(tr("履歴の一致")).strong());
            let transcript_matches = self.transcript_matches.clone();
            egui::ScrollArea::vertical()
                .max_height(220.0)
                .show(ui, |ui| {
                    for transcript in &transcript_matches {
                        egui::Frame::group(ui.style()).show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(&transcript.provider).small().weak());
                                if let Some(score) = transcript.score {
                                    if score < 0.999 {
                                        let pct = (score * 100.0).round() as u64;
                                        ui.label(RichText::new(format!("({pct}%)")).small().weak());
                                    }
                                }
                                ui.label(
                                    RichText::new(
                                        transcript
                                            .title
                                            .as_deref()
                                            .unwrap_or(&transcript.session_id),
                                    )
                                    .strong(),
                                );
                                if icon_button(ui, palette, ICON_OPEN_EXTERNAL, tr("Finder で表示"))
                                    .clicked()
                                {
                                    let path = transcript.path.clone();
                                    self.request_system_action(tr("Finder で表示"), move || {
                                        open_path(&path)
                                    });
                                }
                            });
                            if transcript.title.is_some() {
                                ui.label(RichText::new(&transcript.session_id).small().weak());
                            }
                            ui.label(&transcript.snippet);
                        });
                        ui.add_space(4.0);
                    }
                });
        }
        ui.separator();
        if self.session_view == SessionView::Grid {
            self.ui_session_grid(ui);
            return;
        }
        self.ui_project_session_list(ui);
    }

    fn ui_project_session_list(&mut self, ui: &mut egui::Ui) {
        let palette = self.store.theme.palette();
        let projects = self.store.projects.clone();
        let sessions = self.store.sessions.clone();
        let needle = self.search.to_lowercase();
        let mut matched_sessions = 0;
        if projects.is_empty() {
            ui.label(
                RichText::new(tr(
                    "プロジェクトがありません。先にプロジェクトを追加してください。",
                ))
                .weak(),
            );
            return;
        }
        egui::ScrollArea::vertical().show(ui, |ui| {
            for project in projects {
                let mut project_sessions = sessions
                    .iter()
                    .filter(|session| {
                        session.project_id == project.id
                            && (needle.is_empty()
                                || format!("{} {} {}", session.name, session.goal, session.agent)
                                    .to_lowercase()
                                    .contains(&needle))
                    })
                    .cloned()
                    .collect::<Vec<_>>();
                project_sessions.sort_by_key(|session| Reverse(session.created_at));
                matched_sessions += project_sessions.len();
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(ICON_PROJECT).color(palette.accent_text));
                        ui.label(RichText::new(&project.name).strong());
                        ui.label(
                            RichText::new(tf!("{p0} 件", p0 = project_sessions.len()))
                                .small()
                                .weak(),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if small_icon_button(ui, palette, ICON_SHOW, tr("プロジェクトを開く"))
                                .clicked()
                            {
                                self.open_project_session_setup(project.id);
                            }
                        });
                    });
                    ui.label(
                        RichText::new(project.path.display().to_string())
                            .small()
                            .weak(),
                    );
                    ui.add_space(4.0);
                    if project_sessions.is_empty() {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(tr("セッションはまだありません。"))
                                    .small()
                                    .weak(),
                            );
                            if small_icon_button(ui, palette, ICON_ADD, tr("セッションを開始"))
                                .clicked()
                            {
                                self.open_project_session_setup(project.id);
                            }
                        });
                    } else {
                        for session in project_sessions {
                            self.session_card(ui, &session);
                        }
                    }
                });
                ui.add_space(8.0);
            }
            if matched_sessions == 0 && !needle.is_empty() {
                ui.label(RichText::new(tr("検索条件に一致するセッションはありません。")).weak());
            }
        });
    }

    fn ui_session_grid(&mut self, ui: &mut egui::Ui) {
        let palette = self.store.theme.palette();
        ui.label(tr("エージェント一覧"));
        let needle = self.search.to_lowercase();
        let projects = self.store.projects.clone();
        let sessions = self.store.sessions.clone();
        let session_names = sessions
            .iter()
            .map(|session| (session.id, session.name.clone()))
            .collect::<HashMap<_, _>>();
        if projects.is_empty() {
            ui.label(RichText::new(tr("プロジェクトはまだありません。")).weak());
            return;
        }
        egui::ScrollArea::vertical().show(ui, |ui| {
            for project in projects {
                let mut project_sessions = sessions
                    .iter()
                    .filter(|session| {
                        session.project_id == project.id
                            && (needle.is_empty()
                                || format!("{} {} {}", session.name, session.goal, session.agent)
                                    .to_lowercase()
                                    .contains(&needle))
                    })
                    .cloned()
                    .collect::<Vec<_>>();
                project_sessions.sort_by_key(|session| session.created_at);
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(&project.name).strong());
                        ui.label(
                            RichText::new(project.path.display().to_string())
                                .small()
                                .weak(),
                        );
                        if icon_button(ui, palette, ICON_SHOW, tr("プロジェクトを開く")).clicked()
                        {
                            self.select_project(Some(project.id));
                            self.page = Page::Projects;
                        }
                    });
                    ui.add_space(4.0);
                    if project_sessions.is_empty() {
                        ui.label(
                            RichText::new(if needle.is_empty() {
                                tr("セッションはまだありません。")
                            } else {
                                tr("一致するセッションはありません。")
                            })
                            .small()
                            .weak(),
                        );
                    } else {
                        ui.horizontal_wrapped(|ui| {
                            for session in project_sessions {
                                self.session_grid_node(ui, &session, &session_names);
                            }
                        });
                    }
                });
                ui.add_space(8.0);
            }
        });
    }

    fn session_grid_node(
        &mut self,
        ui: &mut egui::Ui,
        session: &Session,
        session_names: &HashMap<Uuid, String>,
    ) {
        let palette = self.store.theme.palette();
        let status = self.status_view(session);
        let failed_dependency = blocking_dependency_label(&self.store.sessions, session);
        let mut removal_requested = false;
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.set_min_width(205.0);
            ui.horizontal(|ui| {
                status_chip(ui, &status, 13.5, palette);
                ui.label(RichText::new(session_title(session)).strong());
                agent_chip(ui, &session.agent, 12.0, palette);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if small_icon_button(ui, palette, ICON_CLOSE, tr("セッションを削除")).clicked()
                    {
                        removal_requested = true;
                    }
                });
            });
            if session.depends_on.is_empty() {
                ui.label(RichText::new(tr("単独で開始します")).small().weak());
            } else {
                let parents = session
                    .depends_on
                    .iter()
                    .filter_map(|id| session_names.get(id))
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ");
                ui.label(
                    RichText::new(format!("{ICON_DEPENDS_ON} {parents}"))
                        .size(12.0)
                        .color(palette.text),
                );
            }
            if let Some(parent) = &failed_dependency {
                ui.label(
                    RichText::new(tf!("{parent} によりブロック中", parent = parent))
                        .small()
                        .color(palette.accent_soft),
                );
            }
            ui.horizontal(|ui| {
                if icon_button(ui, palette, ICON_SHOW, tr("ターミナルを表示")).clicked() {
                    self.open_session_in_terminal(session.id);
                }
                if matches!(
                    session.status,
                    SessionStatus::Exited | SessionStatus::Failed
                ) && icon_button(
                    ui,
                    palette,
                    ICON_CLOSE,
                    if session.status == SessionStatus::Failed {
                        tr("残ったターミナルを閉じる")
                    } else {
                        tr("ターミナルを閉じる")
                    },
                )
                .clicked()
                {
                    self.close_completed_terminal(session.id);
                }
                if let Some(verb) = session_verb(session, self.cancellation_pending(session.id)) {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let response = verb_button(ui, palette, verb.icon, verb.label, verb.danger)
                            .on_hover_text(verb.hint);
                        if response.clicked() {
                            match verb.action {
                                SessionVerbAction::Stop => self.stop_session(session.id),
                                SessionVerbAction::Start => self.start_session(session.id),
                                SessionVerbAction::ResumeNative => {
                                    self.resume_managed_native_session(session.id)
                                }
                            }
                        }
                    });
                }
            });
            self.session_removal_confirmation(ui, session.id);
        });
        if removal_requested {
            self.request_session_removal(session.id);
        }
    }

    /// Settings as a list of five sections and the chosen one beside it
    /// (change 094). It was one page, and a person scrolled past the fonts to
    /// find the tool that was missing.
    pub(crate) fn ui_settings(&mut self, ui: &mut egui::Ui) {
        let palette = self.store.theme.palette();
        let ready = self.tools.tmux && self.tools.available_agent_count() > 0;
        ui.horizontal_top(|ui| {
            ui.vertical(|ui| {
                ui.set_width(200.0);
                ui.label(
                    RichText::new(tr("設定"))
                        .size(20.0)
                        .strong()
                        .color(palette.text_strong),
                );
                ui.add_space(SPACE_SM);
                for section in SettingsSection::all() {
                    ui.horizontal(|ui| {
                        let chosen = self.settings_section == section;
                        if nav_tab(ui, palette, "", section.label(), chosen).clicked() {
                            self.settings_section = section;
                        }
                        if section == SettingsSection::Agents {
                            let (word, ink) = if ready {
                                (tr("準備完了"), palette.success)
                            } else {
                                (tr("要対応"), palette.accent_soft)
                            };
                            ui.label(RichText::new(word).size(12.0).color(ink));
                        }
                    });
                }
            });
            ui.add_space(SPACE_LG);
            ui.vertical(|ui| {
                let section = self.settings_section;
                page_header(
                    ui,
                    palette,
                    section.label(),
                    Some(section.summary()),
                    |_| {},
                );
                match section {
                    SettingsSection::Appearance => self.ui_settings_appearance(ui, palette),
                    SettingsSection::Agents => self.ui_settings_agents(ui, palette),
                    SettingsSection::Notifications => self.ui_settings_notifications(ui),
                    SettingsSection::Data => self.ui_settings_data(ui),
                    SettingsSection::Keys => self.ui_settings_keys(ui, palette),
                }
            });
        });
    }

    fn ui_settings_appearance(&mut self, ui: &mut egui::Ui, palette: &Palette) {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            let previous_theme = self.store.theme;
            let mut theme_changed = false;
            ui.horizontal(|ui| {
                ui.label(tr("テーマ"));
                egui::ComboBox::from_id_salt("app-theme")
                    .selected_text(self.store.theme.label())
                    .show_ui(ui, |ui| {
                        for theme in AppTheme::all() {
                            theme_changed |= ui
                                .selectable_value(&mut self.store.theme, theme, theme.label())
                                .changed();
                        }
                    });
                theme_swatches(ui, self.store.theme.palette());
            });
            ui.label(
                RichText::new(self.store.theme.detail())
                    .small()
                    .color(palette.text_muted),
            );
            if theme_changed && !self.persist() {
                self.store.theme = previous_theme;
            }

            let previous_terminal_font = self.store.terminal_font;
            let mut font_changed = false;
            ui.horizontal(|ui| {
                ui.label(tr("ターミナルとコードのフォント"));
                egui::ComboBox::from_id_salt("terminal-font")
                    .selected_text(self.store.terminal_font.label())
                    .show_ui(ui, |ui| {
                        for font in [
                            TerminalFont::SarasaMonoJ,
                            TerminalFont::SarasaMonoK,
                            TerminalFont::SarasaTermJ,
                            TerminalFont::SarasaFixedJ,
                        ] {
                            font_changed |= ui
                                .selectable_value(&mut self.store.terminal_font, font, font.label())
                                .changed();
                        }
                    });
            });
            // Every Sarasa face is 1:2, so the cell rule belongs to the picker
            // rather than to each of the four entries in it.
            ui.label(
                RichText::new(tf!(
                    "{p0} · ASCII は 1 セル、日本語／韓国語は 2 セル",
                    p0 = self.store.terminal_font.detail()
                ))
                .small()
                .weak(),
            );
            if font_changed && !self.persist() {
                self.store.terminal_font = previous_terminal_font;
            }

            let previous_language = self.store.language;
            let mut language_changed = false;
            ui.horizontal(|ui| {
                ui.label(tr("言語"));
                egui::ComboBox::from_id_salt("app-language")
                    .selected_text(self.store.language.native_label())
                    .show_ui(ui, |ui| {
                        for language in Language::all() {
                            language_changed |= ui
                                .selectable_value(
                                    &mut self.store.language,
                                    language,
                                    language.native_label(),
                                )
                                .changed();
                            if language_changed && language != previous_language {
                                set_active_language(language);
                            }
                        }
                    });
            });
            ui.label(
                RichText::new(self.store.language.detail())
                    .small()
                    .color(palette.text_muted),
            );
            if language_changed && !self.persist() {
                self.store.language = previous_language;
                set_active_language(previous_language);
            }
        });
    }

    /// Each tool on one row — there or not, its hook, its accounts, and what
    /// to do when it is missing — then what happens when a session starts.
    /// These were three places on the old page, with English verdicts.
    fn ui_settings_agents(&mut self, ui: &mut egui::Ui, palette: &Palette) {
        let tools = self.tools.clone();
        let present = [
            tools.tmux,
            tools.codex,
            tools.claude,
            tools.antigravity,
            tools.git,
            tools.gh,
        ]
        .into_iter()
        .filter(|present| *present)
        .count();
        let mut open_setup: Option<(&'static str, &'static str)> = None;
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(tr("ツール")).strong());
                ui.label(
                    RichText::new(tf!("{p0}/6 使用可能", p0 = present))
                        .size(12.5)
                        .color(if present == 6 {
                            palette.success
                        } else {
                            palette.accent_soft
                        }),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if icon_text_button(ui, ICON_REFRESH, tr("ツールを再確認"))
                        .on_hover_text(tr("PATH を再確認"))
                        .clicked()
                    {
                        self.request_tool_status();
                    }
                });
            });
            for (agent, label, available, setup) in [
                (
                    "codex",
                    "Codex CLI",
                    tools.codex,
                    (
                        tr("Codex のセットアップを開く"),
                        "https://developers.openai.com/codex/cli/",
                    ),
                ),
                (
                    "claude",
                    "Claude Code",
                    tools.claude,
                    (
                        tr("Claude Code のセットアップを開く"),
                        "https://docs.anthropic.com/en/docs/claude-code/getting-started",
                    ),
                ),
                (
                    "gemini",
                    "Antigravity CLI (agy)",
                    tools.antigravity,
                    (
                        tr("Antigravity のセットアップを開く"),
                        "https://antigravity.google/",
                    ),
                ),
            ] {
                let hook = self
                    .hooks_enabled
                    .then(|| {
                        self.hook_installs
                            .iter()
                            .find(|install| install.provider.agent() == agent)
                            .map(|install| &install.state)
                    })
                    .flatten();
                let accounts = agent_supports_accounts(agent).then(|| {
                    self.accounts
                        .accounts
                        .iter()
                        .filter(|account| account.agent == agent)
                        .count()
                });
                tool_status_row(
                    ui,
                    Some(agent),
                    label,
                    available,
                    tr("コーディングエージェント"),
                    palette,
                    |ui| {
                        // Drawn right to left, beside the verdict.
                        if !available && quiet_button(ui, palette, tr("手順")).clicked() {
                            open_setup = Some(setup);
                        }
                        if let Some(count) = accounts {
                            ui.label(
                                RichText::new(tf!("アカウント {p0}", p0 = count))
                                    .size(12.0)
                                    .color(palette.text_muted),
                            );
                        }
                        if let Some(state) = hook {
                            hook_cell(ui, state, palette);
                        }
                    },
                );
            }
            tool_status_row(
                ui,
                None,
                "tmux",
                tools.tmux,
                tr("セッションの実行に必須"),
                palette,
                |ui| {
                    if !tools.tmux {
                        ui.monospace("brew install tmux");
                    }
                },
            );
            tool_status_row(
                ui,
                None,
                "Git",
                tools.git,
                tr("worktree・変更内容・履歴の表示"),
                palette,
                |_| {},
            );
            tool_status_row(
                ui,
                None,
                "GitHub CLI",
                tools.gh,
                tr("任意: プルリクエストの詳細"),
                palette,
                |_| {},
            );
        });
        if let Some((label, url)) = open_setup {
            self.request_system_action(label, move || open_url(url));
        }
        if !tools.tmux || tools.available_agent_count() == 0 {
            ui.label(
                RichText::new(
                    tr("Finder から起動したアプリは Homebrew・~/.local/bin・~/.cargo/bin も参照します。"),
                )
                .small()
                .weak(),
            );
        }
        ui.add_space(SPACE_MD);
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.label(RichText::new(tr("起動時の動作")).strong());
            ui.add_space(SPACE_XS);
            if ui
                .checkbox(
                    &mut self.hooks_enabled,
                    tr("エージェント CLI のフックで状態を検知する"),
                )
                .changed()
            {
                let enabled = self.hooks_enabled;
                if enabled {
                    let ctx = ui.ctx().clone();
                    self.bind_hook_listener(&ctx);
                }
                self.request_hook_apply(enabled);
            }
            ui.label(
                RichText::new(tr(
                    "各 CLI の設定ファイルに Operon 用の項目を追加し、ターンの開始と完了をその CLI 自身に報告させます。オフにすると項目を取り除き、画面の読み取りだけで判定します。",
                ))
                .small()
                .weak(),
            );
            let previous_auto_approve_workspace_prompts = self.store.auto_approve_workspace_prompts;
            ui.horizontal(|ui| {
                if ui
                    .checkbox(
                        &mut self.store.auto_approve_workspace_prompts,
                        tr("初回起動時のフォルダ信頼プロンプトを自動承認する"),
                    )
                    .changed()
                    && !self.persist()
                {
                    self.store.auto_approve_workspace_prompts =
                        previous_auto_approve_workspace_prompts;
                }
                ui.label(
                    RichText::new(tr("フォルダ信頼のみ"))
                        .size(12.0)
                        .color(palette.text_faint),
                );
            });
        });
        ui.add_space(SPACE_LG);
        self.ui_agent_accounts(ui, palette);
    }

    fn ui_settings_notifications(&mut self, ui: &mut egui::Ui) {
        let previous_notifications_enabled = self.store.notifications_enabled;
        if ui
            .checkbox(
                &mut self.store.notifications_enabled,
                tr("ターンの完了・入力待ち・終了を通知する"),
            )
            .changed()
        {
            if !self.persist() {
                self.store.notifications_enabled = previous_notifications_enabled;
            } else if self.store.notifications_enabled {
                request_notification_authorization();
            }
        }
    }

    fn ui_settings_data(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new(tr("ローカルデータ")).strong());
        ui.label(tr(
            "メタデータはこの Mac に留まります。ソースコードは送信しません。",
        ));
        ui.monospace(self.data_file.display().to_string());
        ui.add_space(SPACE_SM);
        if icon_text_button(ui, ICON_OPEN_EXTERNAL, tr("データフォルダを表示"))
            .on_hover_text(tr("Finder で開く"))
            .clicked()
        {
            if let Some(parent) = self.data_file.parent() {
                let path = parent.to_path_buf();
                self.request_system_action(tr("Finder で表示"), move || reveal_path(&path));
            }
        }
        ui.add_space(18.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new(tr("ターミナル復旧")).strong());
            if icon_text_button(ui, ICON_SEARCH, tr("tmux をスキャン"))
                .on_hover_text(tr("記録が失われた tmux セッションを探す"))
                .clicked()
            {
                self.scan_orphaned_tmux_sessions();
            }
        });
        let orphaned = self.orphaned_tmux_sessions.clone();
        for orphan in orphaned {
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.label(RichText::new(&orphan.name).strong());
                ui.label(
                    RichText::new(orphan.cwd.display().to_string())
                        .small()
                        .weak(),
                );
                if !self.store.projects.is_empty() {
                    let checking = self
                        .background_tasks
                        .contains(&BackgroundKey::RecoveryAdopt(orphan.name.clone()));
                    if ui
                        .add_enabled(!checking, egui::Button::new(tr("セッションを引き継ぐ")))
                        .clicked()
                    {
                        self.adopt_orphaned_tmux_session(&orphan);
                    }
                    if checking {
                        ui.label(
                            RichText::new(tr("プロジェクトを確認しています…"))
                                .small()
                                .weak(),
                        );
                    }
                } else {
                    ui.label(
                        RichText::new(tr("引き継ぐ前に、このプロジェクトを登録してください。"))
                            .small()
                            .weak(),
                    );
                }
            });
        }
    }

    /// Every chord the window answers to, as the palette draws it. The file is
    /// how they change, so the one action here opens it.
    fn ui_settings_keys(&mut self, ui: &mut egui::Ui, palette: &Palette) {
        egui::Grid::new("settings-keys")
            .num_columns(2)
            .spacing(egui::vec2(SPACE_LG, SPACE_SM))
            .show(ui, |ui| {
                for action in KEYMAP_ACTIONS {
                    ui.label(tr(action.label));
                    let chord = self.keymap.label_for(action.id);
                    if chord.is_empty() {
                        ui.label(RichText::new(tr("なし")).color(palette.text_faint));
                    } else {
                        ui.monospace(chord);
                    }
                    ui.end_row();
                }
            });
        ui.add_space(SPACE_MD);
        if icon_text_button(ui, ICON_OPEN_EXTERNAL, tr("キー割り当てファイルを開く")).clicked()
        {
            self.open_keymap_file();
        }
    }

    /// Registering a login. It sits under the hook setting because that is what
    /// it changes the reach of: every account added here gets the managed hooks
    /// too, which is what keeps the state chip moving for a session that is not
    /// running as the machine's own login.
    pub(crate) fn ui_agent_accounts(&mut self, ui: &mut egui::Ui, palette: &Palette) {
        ui.label(RichText::new(tr("エージェントのアカウント")).strong());
        ui.label(
            RichText::new(tr(
                "アカウントは、CLI がログイン情報を置くフォルダです。Operon はその場所を指すだけで、中身を読むことはありません。追加したフォルダには、上のフック設定と同じ項目が入ります。",
            ))
            .small()
            .weak(),
        );
        ui.add_space(6.0);
        let registered = self.accounts.accounts.clone();
        for account in &registered {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(agent_short_label(&account.agent))
                        .size(12.0)
                        .color(palette.text_muted),
                );
                ui.label(RichText::new(&account.name).strong());
                ui.add(
                    egui::Label::new(
                        RichText::new(account.path.display().to_string())
                            .small()
                            .color(palette.text_muted),
                    )
                    .truncate(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if small_icon_button(ui, palette, ICON_CLOSE, tr("このアカウントを削除"))
                        .clicked()
                    {
                        self.forget_agent_account(account.id);
                    }
                });
            });
        }
        if registered.is_empty() {
            ui.label(RichText::new(tr("まだ登録されていません")).small().weak());
        }
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("account-agent")
                .selected_text(agent_short_label(&self.account_agent_input))
                .width(120.0)
                .show_ui(ui, |ui| {
                    for agent in AGENTS.iter().filter(|agent| agent_supports_accounts(agent)) {
                        ui.selectable_value(
                            &mut self.account_agent_input,
                            (*agent).to_owned(),
                            agent_short_label(agent),
                        );
                    }
                });
            ui.add(
                egui::TextEdit::singleline(&mut self.account_name_input)
                    .hint_text(tr("例: 仕事"))
                    .desired_width(140.0),
            );
            if icon_text_button(ui, ICON_ADD, tr("フォルダを選んで追加")).clicked() {
                self.choose_agent_account_folder();
            }
        });
    }

    pub(crate) fn ui_terminal_session_tabs(&mut self, ui: &mut egui::Ui) {
        let palette = self.store.theme.palette();
        let mut projects = self.store.projects.clone();
        projects.sort_by_key(|project| project.name.to_lowercase());
        let sessions = self.store.sessions.clone();
        // One pass over the sessions for all four numbers. `status_view` would
        // have built two `String`s per session to tell us the same thing, in a
        // path that runs sixty times a second.
        let group_counts = self.status_group_counts();
        ui.horizontal(|ui| {
            ui.label(RichText::new(tr("プロジェクト別セッション")).strong());
            ui.label(
                RichText::new(tf!("{p0} 件", p0 = sessions.len()))
                    .small()
                    .weak(),
            );
            // New sessions start from the toolbar and search is ⌘K; what the
            // magnifier here used to open is the history, and says so.
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if quiet_button(ui, palette, tr("履歴")).clicked() {
                    self.session_library_open = true;
                }
            });
        });
        // The four things a person can do about a session, as four independent
        // toggles on one row, behind a 「すべて」 that says no filter is on and
        // clears one that is. The icon each group used to carry is the status
        // dot on every row below, so the chip says the word and, when there
        // is anything to count, the count — a row of zeros read as four more
        // things to look at.
        //
        // The count that used to sit beside the title as `⚠ n` is the 要対応
        // chip now: two counts of "needs me" side by side, defined differently,
        // is worse than one.
        let filtering = self.status_filter.iter().any(|selected| *selected);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            if ui
                .selectable_label(
                    !filtering,
                    RichText::new(tf!("すべて {count}", count = sessions.len()))
                        .size(11.5)
                        .color(palette.text),
                )
                .clicked()
            {
                self.clear_status_filter();
            }
            for group in SessionStatusGroup::GROUPS {
                let index = group.index();
                let count = group_counts[index];
                let selected = self.status_filter[index];
                let ink = if count == 0 {
                    palette.text_faint
                } else {
                    palette.status(group.tone())
                };
                let label = if count == 0 {
                    tr(group.message_id()).to_owned()
                } else {
                    format!("{} {count}", tr(group.message_id()))
                };
                let chip = ui
                    .selectable_label(selected, RichText::new(label).size(11.5).color(ink))
                    .on_hover_text(tr(group.hint_id()));
                if chip.clicked() {
                    self.toggle_status_filter(group);
                }
            }
        });
        ui.separator();
        let mut shown = 0usize;
        egui::ScrollArea::vertical().show(ui, |ui| {
            for project in projects {
                let mut project_sessions = sessions
                    .iter()
                    .filter(|session| session.project_id == project.id)
                    .filter(|session| self.session_matches_status_filter(session))
                    .cloned()
                    .collect::<Vec<_>>();
                project_sessions.sort_by_key(|session| Reverse(session.created_at));
                shown += project_sessions.len();
                // A project with nothing to show contributes nothing: not its
                // name, not its path, not its empty line. Four lines each, and
                // the whole point of the filter is the length of this column.
                if filtering && project_sessions.is_empty() {
                    continue;
                }
                // One line. The full path is what tells two checkouts of the
                // same name apart, and that is a question asked rarely enough
                // to live under the pointer rather than on every group.
                ui.horizontal(|ui| {
                    ui.label(RichText::new(ICON_PROJECT).color(palette.accent_text));
                    ui.label(RichText::new(&project.name).small().strong())
                        .on_hover_ui(|ui| {
                            ui.label(project.path.display().to_string());
                        });
                    ui.label(
                        RichText::new(tf!("{p0} 件", p0 = project_sessions.len()))
                            .small()
                            .weak(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if small_icon_button(ui, palette, ICON_ADD, tr("セッションを開始")).clicked() {
                            self.open_project_session_setup(project.id);
                        }
                    });
                });
                if project_sessions.is_empty() {
                    ui.horizontal(|ui| {
                        ui.add_space(8.0);
                        ui.label(RichText::new(tr("セッションはまだありません")).small().weak());
                    });
                    ui.add_space(8.0);
                    continue;
                }
                let is_importing = self.background_tasks.iter().any(|key| {
                    matches!(key, BackgroundKey::FullHistoryImport(_))
                });
                for candidate in project_sessions {
                    let is_selected = self.selected_session == Some(candidate.id);
                    let is_unread = self.is_session_unread(candidate.id);
                    let mut select_requested = false;
                    let mut unread_requested = false;
                    let mut requested_handoff = None;
                    let mut close_requested = false;
                    let mut worktree_requested = false;
                    let status = self.status_view(&candidate);
                    let title = session_list_title(
                        &candidate,
                        self.session_prompt_turns
                            .get(&candidate.id)
                            .and_then(|turns| turns.first())
                            .map(|turn| turn.prompt.as_str()),
                    );
                    let renaming = self.renaming_session == Some(candidate.id);
                    let mut rename_requested = false;
                    let mut rename_finished = None;
                    let render_restore_options = |ui: &mut egui::Ui, handoff_target: &mut Option<String>| {
                        if cli_provider_for_agent(&candidate.agent).is_some() {
                            ui.separator();
                            for agent in FULL_HISTORY_RESTORE_TARGETS.iter().copied() {
                                if agent != candidate.agent.as_str() {
                                    let button = egui::Button::new(tf!("{p0} へ履歴を復元", p0 = agent_choice_copy(agent).0));
                                    if ui
                                        .add_enabled(!is_importing, button)
                                        .on_hover_text(tf!("{p0} へ全履歴を復元します", p0 = agent_choice_copy(agent).0))
                                        .clicked()
                                    {
                                        *handoff_target = Some(agent.to_owned());
                                        ui.close_menu();
                                    }
                                }
                            }
                        }
                    };
                    // The row's `✕` stands where its time does while the
                    // pointer is on the row or the row is open, so a list of
                    // idle sessions reads as names and ages rather than as a
                    // column of delete buttons. `···` stays put: a menu whose
                    // button vanished under it would close as it opened.
                    let row_active =
                        is_selected || self.hovered_session_row == Some(candidate.id);
                    let session_card = ui.scope_builder(
                        egui::UiBuilder::new()
                            .id_salt(("terminal-session-card", candidate.id))
                            .sense(egui::Sense::click()),
                        |ui| {
                            egui::Frame::default()
                                .fill(if is_selected {
                                    palette.row_selected
                                } else {
                                    Color32::TRANSPARENT
                                })
                                .corner_radius(egui::CornerRadius::same(RADIUS_CONTROL))
                                .inner_margin(egui::Margin::symmetric(6, 5))
                                .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.spacing_mut().item_spacing.y = 2.0;
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = SPACE_XS + 2.0;
                                // The status as a dot in its own colour. The
                                // word is on the line below; the dot is what a
                                // glance down the column reads.
                                let (dot, _) = ui.allocate_exact_size(
                                    egui::vec2(8.0, 8.0),
                                    egui::Sense::hover(),
                                );
                                ui.painter().circle_filled(
                                    dot.center(),
                                    4.0,
                                    palette.status(status.tone),
                                );
                                // What is left once the corner has its room:
                                // the time, or `···` and `✕`.
                                let title_width = (ui.available_width() - 64.0).max(40.0);
                                ui.allocate_ui_with_layout(
                                    egui::vec2(title_width, 20.0),
                                    egui::Layout::left_to_right(egui::Align::Center),
                                    |ui| {
                                if renaming {
                                    let editor = ui.add(
                                        egui::TextEdit::singleline(&mut self.rename_input)
                                            .hint_text(tr("セッション名"))
                                            .desired_width(title_width),
                                    );
                                    if self.rename_needs_focus {
                                        editor.request_focus();
                                        self.rename_needs_focus = false;
                                    }
                                    let cancelled = ui
                                        .input(|input| input.key_pressed(egui::Key::Escape));
                                    if cancelled {
                                        rename_finished = Some(false);
                                    } else if editor.lost_focus() {
                                        // Committing on blur as well as on Enter
                                        // keeps a typed name from being lost by
                                        // clicking away from the editor.
                                        rename_finished = Some(true);
                                    }
                                } else {
                                    let title = ui
                                        .add(
                                            egui::Label::new(
                                                RichText::new(&title)
                                                .strong()
                                                .size(13.5)
                                                // Every title on this list is
                                                // already bold, so an unread one
                                                // is told apart by ink rather
                                                // than by weight: the row a
                                                // person has not read yet is the
                                                // one written in the strongest
                                                // ink the theme has.
                                                .color(if is_unread {
                                                    palette.text_strong
                                                } else {
                                                    palette.text
                                                }),
                                            )
                                            .truncate()
                                            .sense(egui::Sense::click()),
                                        )
                                        .on_hover_text(if is_unread {
                                            tf!("{title}\n\n未読 · ダブルクリックで名前を変更", title = title)
                                        } else {
                                            tf!("{title}\n\nダブルクリックで名前を変更", title = title)
                                        });
                                    if title.double_clicked() {
                                        rename_requested = true;
                                    }
                                }
                                    },
                                );
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.spacing_mut().item_spacing.x = 0.0;
                                        if row_active {
                                            let close_res = ui.add(
                                                egui::Button::new(
                                                    RichText::new(ICON_CLOSE).size(15.0),
                                                )
                                                .frame(false)
                                                .min_size(egui::vec2(CONTROL_HEIGHT_SMALL, CONTROL_HEIGHT_SMALL)),
                                            );
                                            paint_button_focus(ui, &close_res, palette);
                                            if close_res
                                                .on_hover_text(tr("セッションを削除"))
                                                .clicked()
                                            {
                                                close_requested = true;
                                            }
                                        } else {
                                            ui.label(
                                                RichText::new(relative_time(candidate.created_at))
                                                    .size(11.5)
                                                    .color(palette.text_faint),
                                            );
                                        }
                                        // The rare actions sit behind the dots so
                                        // the row's one destructive corner control
                                        // keeps its neighbours predictable.
                                        let menu_res = egui::menu::menu_custom_button(
                                            ui,
                                            egui::Button::new(
                                                RichText::new(ICON_MORE)
                                                    .size(14.0)
                                                    .color(palette.text_muted),
                                            )
                                            .frame(false)
                                            .min_size(egui::vec2(CONTROL_HEIGHT_SMALL, CONTROL_HEIGHT_SMALL)),
                                            |ui| {
                                                if ui.button("worktree").clicked() {
                                                    worktree_requested = true;
                                                    ui.close_menu();
                                                }
                                                if ui.button(tr("名前を変更")).clicked() {
                                                    rename_requested = true;
                                                    ui.close_menu();
                                                }
                                                if ui.button(tr("未読にする")).clicked() {
                                                    unread_requested = true;
                                                    ui.close_menu();
                                                }
                                                render_restore_options(ui, &mut requested_handoff);
                                            },
                                        )
                                        .response
                                        .on_hover_text(tr("worktree · 名前を変更 · 復元 · 未読にする"));
                                        paint_button_focus(ui, &menu_res, palette);
                                    },
                                );
                            });
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = SPACE_XS;
                                ui.add_space(14.0);
                                agent_chip(ui, &candidate.agent, 10.5, palette);
                                ui.label(
                                    RichText::new(agent_short_label(&candidate.agent))
                                        .size(11.5)
                                        .color(palette.text_muted),
                                );
                                ui.label(RichText::new("·").size(11.5).color(palette.text_faint));
                                // Four words for fourteen states: which one it
                                // is, is the hover's to say.
                                ui.label(
                                    RichText::new(&status.label)
                                        .size(11.5)
                                        .color(palette.status(status.tone)),
                                )
                                .on_hover_text(&status.hint);
                                // The branch only when the session runs in a
                                // worktree: in the project's own checkout it is
                                // the project row's branch and says nothing new.
                                if let Some(path) = &candidate.worktree_path {
                                    let location = candidate
                                        .branch
                                        .as_deref()
                                        .map(str::to_owned)
                                        .or_else(|| {
                                            path.file_name()
                                                .and_then(|name| name.to_str())
                                                .map(str::to_owned)
                                        })
                                        .unwrap_or_else(|| path.display().to_string());
                                    ui.add(
                                        egui::Label::new(
                                            RichText::new(format!("{ICON_BRANCH} {location}"))
                                                .size(11.5)
                                                .color(palette.branch),
                                        )
                                        .truncate(),
                                    );
                                }
                            });
                            self.session_removal_confirmation(ui, candidate.id);
                                });
                        },
                    );
                    // Geometric, not `hovered()`: the pointer on the row's own
                    // buttons is still on the row.
                    if ui.rect_contains_pointer(session_card.response.rect) {
                        self.hovered_session_row = Some(candidate.id);
                    } else if self.hovered_session_row == Some(candidate.id) {
                        self.hovered_session_row = None;
                    }
                    let session_card_response = session_card.response.on_hover_text(
                        tr("クリックでターミナル表示 · ダブルクリックで名前を変更"),
                    );
                    // A double click is easy to miss, and the title it lands on
                    // is small. The same actions are on the card's context menu so
                    // they can be found without knowing the gesture.
                    let mut menu_used = false;
                    session_card_response.context_menu(|ui| {
                        menu_used = true;
                        if ui.button(tr("名前を変更")).clicked() {
                            rename_requested = true;
                            ui.close_menu();
                        }
                        if ui.button("worktree").clicked() {
                            worktree_requested = true;
                            ui.close_menu();
                        }
                        if ui.button(tr("未読にする")).clicked() {
                            unread_requested = true;
                            ui.close_menu();
                        }
                        render_restore_options(ui, &mut requested_handoff);
                    });
                    if session_card_response.clicked()
                        && !close_requested
                        && !worktree_requested
                        && !rename_requested
                        && !menu_used
                        && requested_handoff.is_none()
                    {
                        select_requested = true;
                    }
                    match rename_finished {
                        Some(true) => self.commit_session_rename(),
                        Some(false) => self.cancel_session_rename(),
                        None => {}
                    }
                    if rename_requested {
                        self.begin_session_rename(&candidate);
                    }
                    if unread_requested {
                        self.mark_session_unread(candidate.id);
                    }
                    if select_requested {
                        self.open_session_in_terminal(candidate.id);
                    }
                    if let Some(target) = requested_handoff {
                        self.handoff_managed_session(candidate.id, &target);
                    }
                    if close_requested {
                        self.request_session_removal(candidate.id);
                    }
                    if worktree_requested {
                        self.open_project_worktrees(project.id);
                    }
                    ui.add_space(4.0);
                }
                ui.add_space(6.0);
            }
            if sessions.is_empty() {
                // A person with no sessions is not being filtered, so this
                // wins over anything the filter would have said.
                ui.label(RichText::new(tr("セッションはまだありません。プロジェクトから起動してください。")).weak());
            } else if filtering {
                // A list that hides things says so, and says how to stop.
                // Anything less is a list that has silently lost sessions.
                if shown == 0 {
                    ui.label(
                        RichText::new(tr("選んだ状態のセッションはありません"))
                            .small()
                            .color(palette.text_muted),
                    );
                    if ui.button(tr("フィルタを解除")).clicked() {
                        self.clear_status_filter();
                    }
                } else {
                    ui.label(
                        RichText::new(tf!("{p0} 件を非表示中", p0 = sessions.len().saturating_sub(shown)))
                            .size(11.5)
                            .color(palette.text_faint),
                    );
                }
            }
        });
    }

    pub(crate) fn ui_terminal_panel(&mut self, ui: &mut egui::Ui, session: &Session) {
        let palette = self.store.theme.palette();
        let session_id = session.id;
        let cancellation_pending = self.cancellation_pending(session_id);
        let project = self
            .store
            .projects
            .iter()
            .find(|project| project.id == session.project_id)
            .cloned();
        let project_name = project
            .as_ref()
            .map(|project| project.name.clone())
            .unwrap_or_else(|| tr("登録されていないプロジェクト").into());
        let status = self.status_view(session);
        let title = session_list_title(
            session,
            self.session_prompt_turns
                .get(&session_id)
                .and_then(|turns| turns.first())
                .map(|turn| turn.prompt.as_str()),
        );
        let activity = self.session_activity_for(session_id);
        // The pane's own header, on one line: what you are looking at on the
        // left, what you can do to it pinned to the right. The actions never
        // move as the status changes, because a control that walks along the
        // row as a session runs is a control that has to be found again each
        // time. The agent and project used to be said twice, in the title and
        // again on a line of their own; the title is the conversation now, so
        // the rest follows it once.
        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                // The state verb owns the corner. Its word is visible outright,
                // and the slot does not move as the status changes — the verb
                // that lives there is what changes.
                if let Some(verb) = session_verb(session, cancellation_pending) {
                    let loud = verb_is_loud(&verb, &session.status, cancellation_pending, activity);
                    let response = verb_button(ui, palette, verb.icon, verb.label, loud)
                        .on_hover_text(verb.hint);
                    if response.clicked() {
                        match verb.action {
                            SessionVerbAction::Stop => self.stop_session(session_id),
                            SessionVerbAction::Start => self.start_session(session_id),
                            SessionVerbAction::ResumeNative => {
                                self.resume_managed_native_session(session_id)
                            }
                        }
                    }
                }
                let _ = ui
                    .menu_button(RichText::new(ICON_MORE).size(16.0), |ui| {
                        // The pane covers reading and typing. What is left is
                        // what only a real terminal window can do, and the item
                        // says so rather than naming the app twice next to
                        // "ターミナルを表示".
                        if ui
                            .button(tr("Terminal.app で開く（マウス操作・tmux 操作用）"))
                            .clicked()
                        {
                            let tmux_name = session.tmux_name.clone();
                            self.request_system_action(tr("ターミナルを開く"), move || {
                                open_tmux(&tmux_name)
                            });
                            ui.close_menu();
                        }
                        // Selecting text covers what is on screen. Everything
                        // tmux still holds is more than a drag can reach, so it
                        // gets its own item.
                        if ui.button(tr("表示中のログをすべてコピー")).clicked() {
                            if let Some(lines) = self.terminal_layouts.get(&session_id) {
                                let text = terminal_plain_text(lines);
                                ui.ctx().copy_text(text);
                                self.notice_briefly(tr("ターミナルのログをコピーしました。"));
                            }
                            ui.close_menu();
                        }
                        if ui.button(tr("今すぐ同期")).clicked() {
                            self.refresh_session_output(session_id);
                            ui.close_menu();
                        }
                        if let Some(ref wt) = session.worktree_path {
                            ui.separator();
                            ui.label(RichText::new(tr("外部エディタで開く")).small().weak());
                            if ui.button("VS Code").clicked() {
                                let _ = open_in_external_editor(wt, ExternalEditor::VsCode);
                                ui.close_menu();
                            }
                            if ui.button("Cursor").clicked() {
                                let _ = open_in_external_editor(wt, ExternalEditor::Cursor);
                                ui.close_menu();
                            }
                            if ui.button("Zed").clicked() {
                                let _ = open_in_external_editor(wt, ExternalEditor::Zed);
                                ui.close_menu();
                            }
                            if ui.button("Finder").clicked() {
                                let _ = open_in_external_editor(wt, ExternalEditor::Finder);
                                ui.close_menu();
                            }
                        }
                        if let (Some(project), Some(ref wt), Some(ref branch)) =
                            (&project, &session.worktree_path, &session.branch)
                        {
                            ui.separator();
                            if ui.button(tr("worktree を main にマージ…")).clicked() {
                                self.pending_land_worktree =
                                    Some((project.id, wt.clone(), branch.clone()));
                                ui.close_menu();
                            }
                            if self.tools.gh && ui.button(tr("プルリクエストを作成…")).clicked()
                            {
                                self.pending_pr_modal = Some((
                                    project.id,
                                    wt.clone(),
                                    format!("feat: changes from {branch}"),
                                    String::new(),
                                    false,
                                ));
                                ui.close_menu();
                            }
                        }
                        if let Some(project) = &project {
                            let wt_to_check =
                                session.worktree_path.as_deref().unwrap_or(&project.path);
                            if let SetupScriptRead::Found(script) = read_run_script(wt_to_check) {
                                ui.separator();
                                if ui
                                    .button(tr("実行スクリプト (.operon/run.sh) を起動…"))
                                    .clicked()
                                {
                                    self.pending_run_script = Some(PendingRunScript {
                                        project: project.id,
                                        worktree: wt_to_check.to_path_buf(),
                                        script,
                                    });
                                    ui.close_menu();
                                }
                            }
                        }
                        if let Some(ref _wt) = session.worktree_path {
                            ui.separator();
                            ui.label(RichText::new(tr("チェックポイント")).small().weak());
                            if ui.button(tr("手動チェックポイントを作成")).clicked() {
                                self.capture_checkpoint(session_id, "手動作成");
                                self.notice_briefly(tr("チェックポイントを作成しました。"));
                                ui.close_menu();
                            }
                            let checkpoints = self
                                .session_checkpoints
                                .get(&session_id)
                                .cloned()
                                .unwrap_or_default();
                            if !checkpoints.is_empty() {
                                let mut restore_commit = None;
                                let mut copy_diff_commit = None;
                                ui.menu_button(
                                    tr("チェックポイントへロールバック"),
                                    |ui| {
                                        for cp in checkpoints.iter().rev().take(5) {
                                            let label =
                                                format!("{} (Turn {})", cp.summary, cp.turn_index);
                                            ui.horizontal(|ui| {
                                                if ui.button(&label).clicked() {
                                                    restore_commit = Some(cp.commit.clone());
                                                    ui.close_menu();
                                                }
                                                if ui.button(tr("差分")).clicked() {
                                                    copy_diff_commit = Some(cp.commit.clone());
                                                    ui.close_menu();
                                                }
                                            });
                                        }
                                    },
                                );
                                if let Some(commit) = restore_commit {
                                    self.restore_checkpoint(session_id, &commit);
                                }
                                if let Some(commit) = copy_diff_commit {
                                    if let Some(ref wt) = session.worktree_path {
                                        if let Ok(diff) = git_diff_checkpoint(wt, &commit) {
                                            ui.ctx().copy_text(diff);
                                            self.notice_briefly(tr("差分をコピーしました。"));
                                        }
                                    }
                                }
                            }
                        }
                    })
                    .response
                    .on_hover_text(tr("その他の操作"));
                // A folded side panel is reopened from here; an open one is
                // folded from its own tab row, next to what it hides.
                if !self.show_session_inspector
                    && icon_button(ui, palette, ICON_FOLDER_OPEN, tr("サイドパネルを表示"))
                        .clicked()
                {
                    self.show_session_inspector = true;
                }
                ui.add_space(SPACE_SM);
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = SPACE_XS;
                    agent_chip(ui, &session.agent, 12.0, palette);
                    let goal = session_display_goal(&session.goal);
                    ui.add(
                        egui::Label::new(
                            RichText::new(&title)
                                .size(15.0)
                                .strong()
                                .color(palette.text_strong),
                        )
                        .truncate(),
                    )
                    .on_hover_ui(|ui| {
                        ui.label(if goal.trim().is_empty() {
                            &title
                        } else {
                            &goal
                        });
                    });
                    status_chip(ui, &status, 12.5, palette);
                    meta_separator(ui, palette);
                    ui.label(
                        RichText::new(&project_name)
                            .size(12.0)
                            .color(palette.text_muted),
                    );
                    if let Some(branch) = &session.branch {
                        ui.label(RichText::new("/").size(12.0).color(palette.text_faint));
                        ui.label(
                            RichText::new(format!("{ICON_BRANCH} {branch}"))
                                .size(12.0)
                                .color(palette.branch),
                        );
                    }
                    self.ui_session_ports(ui, palette, session);
                });
            });
        });
        ui.add_space(SPACE_SM);
        hairline(ui, palette);
        ui.add_space(SPACE_SM);
        self.ensure_initial_prompt_turn(session_id);
        let total_row_w = ui.available_width();
        let layout = SessionColumnsLayout::compute(total_row_w, self.show_session_inspector);
        // Docked left, the panel is stacked under the session list in the
        // workspace's own column (change 127); only docked right does it take
        // width from this row.
        let show_inspector =
            layout.show_inspector && self.session_inspector_side == SidebarSide::Right;
        let max_inspector_w = (total_row_w - SessionColumnsLayout::TERMINAL_MIN_W)
            .clamp(SIDEBAR_MIN_W, SIDEBAR_MAX_W);
        let inspector_w = self
            .session_inspector_w
            .unwrap_or(layout.inspector_w)
            .clamp(SIDEBAR_MIN_W, max_inspector_w);
        let terminal_w = if show_inspector {
            (total_row_w - inspector_w - 6.0).max(SessionColumnsLayout::TERMINAL_MIN_W)
        } else {
            total_row_w
        };

        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(terminal_w, ui.available_height()),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    // This frame deliberately consumes every remaining pixel in the right
                    // workspace. It is the terminal viewport itself, not a preview card.
                    // Named from the session alone, not derived from this `Ui`:
                    // the pane and the timeline beside it are anonymous siblings
                    // and therefore share `ui.id`, so anything hashed off it is
                    // one careless key away from colliding with the other pane.
                    let terminal_id = egui::Id::new(("terminal-pane", session_id));
                    let terminal_height = ui.available_height().max(320.0);
                    ui.set_min_width(terminal_w);
                    let cell = terminal_cell_size(ui);
                    if self.resolved_paths_session.is_some()
                        && self.resolved_paths_session != Some(session_id)
                    {
                        self.resolved_paths.clear();
                        self.resolved_paths_generation += 1;
                        self.unresolved_paths.clear();
                    }
                    self.resolved_paths_session = Some(session_id);
                    if self.selected_session.is_none() {
                        self.selected_session = Some(session_id);
                    }
                    let (terminal_rect, (hovered_target, unresolved)) = {
                        let lines = self.terminal_layouts.get(&session_id);
                        let terminal = egui::Frame::default()
                            .fill(palette.terminal_bg)
                            .stroke(egui::Stroke::new(1.0, palette.terminal_border))
                            .inner_margin(egui::Margin::same(8))
                            .show(ui, |ui| {
                                let viewport = egui::vec2(
                                    ui.available_width(),
                                    (terminal_height - 18.0).max(1.0),
                                );
                                ui.set_min_size(viewport);
                                match lines {
                                    Some(lines) => {
                                        let mut unresolved = Vec::new();
                                        let hovered = terminal_rows(
                                            ui,
                                            lines,
                                            viewport,
                                            cell.y,
                                            self.terminal_search_overlay(session_id, palette),
                                            &mut TerminalLinkOverlay {
                                                resolved: &self.resolved_paths,
                                                unresolved: &mut unresolved,
                                                palette,
                                            },
                                        );
                                        (hovered, unresolved)
                                    }
                                    None => {
                                        ui.label(
                                            RichText::new(tr("ターミナル出力を待っています…"))
                                                .color(palette.text_muted),
                                        );
                                        (None, Vec::new())
                                    }
                                }
                            });
                        (terminal.response.rect, terminal.inner)
                    };
                    // Once the pane has jumped, the request is spent: keeping it would undo
                    // the next scroll a person makes by hand.
                    if let Some(search) = self.terminal_search.get_mut(&session_id) {
                        search.scroll_to = None;
                    }
                    self.ui_terminal_search(ui, palette, session_id);
                    self.request_terminal_resize(
                        session_id,
                        terminal_grid_size(terminal_rect.size(), cell),
                    );
                    self.unresolved_paths.extend(unresolved);
                    let terminal_response =
                        ui.interact(terminal_rect, terminal_id, egui::Sense::click());
                    let modifiers = ui.input(|input| input.modifiers);
                    let has_open_modifier = is_terminal_open_modifier(&modifiers);

                    // The pane owns typing, so an ordinary click still only takes focus: an
                    // agent being talked to must not lose the keyboard because a stack
                    // trace happened to be under the pointer. Holding ⌘ (Command) or ⌥ (Option)
                    // says "open this", which is the gesture the screen was agreed on.
                    if let Some(kind) = hovered_target.clone() {
                        if has_open_modifier {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        if terminal_response.clicked() && has_open_modifier {
                            if is_terminal_cmd_modifier(&modifiers) {
                                self.open_terminal_target_external(kind.clone());
                            } else {
                                self.open_terminal_target(kind.clone());
                            }
                        }
                        if terminal_response.secondary_clicked() {
                            if let TerminalTargetKind::Path { path, .. } = &kind {
                                if let Some(resolved) = self.resolve_session_terminal_file(path) {
                                    self.terminal_path_menu = Some((session_id, resolved));
                                }
                            }
                        }
                    }
                    if terminal_response.clicked()
                        && !has_open_modifier
                        && session.status == SessionStatus::Active
                    {
                        terminal_response.request_focus();
                    }
                    self.ui_terminal_path_menu(ui, session_id);

                    let cursor = self
                        .terminal_cursors
                        .get(&session_id)
                        .copied()
                        .unwrap_or((0, 0));
                    let cursor_rect = terminal_cursor_rect(terminal_rect, cursor, cell);
                    let preedit = self
                        .terminal_preedits
                        .get(&session_id)
                        .cloned()
                        .unwrap_or_default();

                    if session.status == SessionStatus::Active {
                        if !preedit.is_empty() {
                            draw_terminal_preedit(
                                ui,
                                terminal_rect,
                                cursor,
                                cell,
                                &preedit,
                                palette,
                            );
                        } else if terminal_response.has_focus() {
                            draw_terminal_cursor(ui, terminal_rect, cursor, cell, palette);
                        }
                    }

                    if terminal_response.has_focus() {
                        // Unlike a hidden TextEdit, the terminal pane owns IME input too.
                        // This keeps Japanese text committed by macOS available to tmux.
                        ui.ctx()
                            .send_viewport_cmd(egui::ViewportCommand::IMEAllowed(true));
                        ui.ctx()
                            .send_viewport_cmd(egui::ViewportCommand::IMEPurpose(
                                egui::viewport::IMEPurpose::Terminal,
                            ));
                        ui.ctx()
                            .send_viewport_cmd(egui::ViewportCommand::IMERect(cursor_rect));
                        ui.memory_mut(|memory| {
                            memory.set_focus_lock_filter(
                                terminal_id,
                                egui::EventFilter {
                                    tab: true,
                                    horizontal_arrows: true,
                                    vertical_arrows: true,
                                    escape: true,
                                },
                            );
                        });
                        let is_preediting = !preedit.is_empty();
                        let (inputs, preedit_update) =
                            terminal_input_events(ui.ctx(), is_preediting);
                        match preedit_update {
                            TerminalPreeditUpdate::Set(text) => {
                                self.terminal_preedits.insert(session_id, text);
                            }
                            TerminalPreeditUpdate::Clear => {
                                self.terminal_preedits.remove(&session_id);
                            }
                            TerminalPreeditUpdate::Unchanged => {}
                        }
                        self.queue_terminal_input(session_id, inputs);
                    }
                    // The jump to the newest line belongs to the pane it
                    // scrolls, in the corner a reader's eye ends at.
                    let jump = egui::Rect::from_min_size(
                        terminal_rect.right_bottom() - egui::vec2(96.0, 36.0),
                        egui::vec2(84.0, CONTROL_HEIGHT_SMALL),
                    );
                    if ui
                        .put(
                            jump,
                            egui::Button::new(
                                RichText::new(format!("{ICON_NEXT} {}", tr("最新へ")))
                                    .size(11.5)
                                    .color(palette.text_muted),
                            )
                            .fill(palette.raised),
                        )
                        .on_hover_text(tr("ターミナルの最新行へ移動"))
                        .clicked()
                    {
                        let total_lines = self
                            .terminal_layouts
                            .get(&session_id)
                            .map(|lines| lines.len())
                            .unwrap_or(0);
                        let search = self.terminal_search.entry(session_id).or_default();
                        search.scroll_to = Some(total_lines.saturating_sub(1));
                    }
                },
            );

            if show_inspector {
                let height = ui.available_height();
                let splitter =
                    ui.allocate_response(egui::vec2(6.0, height), egui::Sense::click_and_drag());
                if splitter.hovered() || splitter.dragged() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                }
                if splitter.dragged() {
                    let delta = splitter.drag_delta().x;
                    if delta.is_finite() {
                        self.session_inspector_w =
                            Some((inspector_w - delta).clamp(SIDEBAR_MIN_W, max_inspector_w));
                    }
                }
                ui.painter().line_segment(
                    [
                        egui::pos2(splitter.rect.center().x, splitter.rect.top()),
                        egui::pos2(splitter.rect.center().x, splitter.rect.bottom()),
                    ],
                    egui::Stroke::new(1.0, palette.border_subtle),
                );
                ui.allocate_ui_with_layout(
                    egui::vec2(inspector_w, height),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        self.ui_session_inspector(
                            ui,
                            session,
                            project.as_ref(),
                            palette,
                            inspector_w,
                            height,
                        );
                    },
                );
            }
        });
    }

    pub(crate) fn select_default_terminal_session(&mut self) {
        let selected_is_available = self.selected_session.is_some_and(|session_id| {
            self.store
                .sessions
                .iter()
                .any(|session| session.id == session_id)
        });
        if selected_is_available {
            return;
        }
        // A terminal workspace must not open to an empty state while a
        // managed agent is running. Prefer the newest live session, then the
        // newest remaining record when all sessions are finished.
        let session_id = self
            .store
            .sessions
            .iter()
            .rev()
            .find(|session| {
                matches!(
                    session.status,
                    SessionStatus::Starting | SessionStatus::Active | SessionStatus::Unknown
                )
            })
            .or_else(|| self.store.sessions.last())
            .map(|session| session.id);
        if self.selected_session != session_id {
            self.resolved_paths.clear();
            self.resolved_paths_generation += 1;
            self.unresolved_paths.clear();
            self.resolved_paths_session = session_id;
            self.selected_session = session_id;
        }
        if let Some(session_id) = session_id {
            self.refresh_session_output(session_id);
        }
    }

    pub(crate) fn ui_terminal_workspace(&mut self, ui: &mut egui::Ui) {
        self.select_default_terminal_session();
        let workspace_size = ui.available_size();
        let workspace = ui.allocate_ui_with_layout(
            workspace_size,
            egui::Layout::left_to_right(egui::Align::TOP),
            |ui| {
                ui.set_min_size(workspace_size);
                self.ui_session_column(ui, workspace_size.y);
                let sep_res = ui.allocate_response(
                    egui::vec2(6.0, workspace_size.y),
                    egui::Sense::click_and_drag(),
                );
                if sep_res.hovered() || sep_res.dragged() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                }
                if sep_res.dragged() {
                    let delta = sep_res.drag_delta().x;
                    self.session_list_w =
                        (self.session_list_w + delta).clamp(SIDEBAR_MIN_W, SIDEBAR_MAX_W);
                }
                ui.separator();
                ui.vertical(|ui| {
                    ui.set_min_width(ui.available_width());
                    ui.set_min_height(workspace_size.y);
                    if self.session_library_open {
                        ui.horizontal(|ui| {
                            ui.heading(tr("履歴"));
                            if ui.button(tr("ターミナルに戻る")).clicked() {
                                self.session_library_open = false;
                            }
                        });
                        ui.separator();
                        self.ui_history_cli_section(ui);
                        self.ui_sessions(ui);
                    } else {
                        let selected = self
                            .selected_session
                            .and_then(|id| {
                                self.store.sessions.iter().find(|session| session.id == id)
                            })
                            .cloned();
                        if let Some(session) = selected {
                            self.ui_terminal_panel(ui, &session);
                        } else {
                            ui.add_space(36.0);
                            ui.heading(tr("セッションを選択"));
                            ui.label(tr("左の一覧からセッションを選んでください。"));
                        }
                    }
                });
            },
        );
        let palette = self.store.theme.palette();
        self.ui_sidebar_tab_drop(ui, workspace.response.rect, palette);
    }

    /// The workspace's left column: the session list, and — docked left,
    /// open, and with a session to describe — the side panel stacked under
    /// it, split by a draggable rule (change 127).
    fn ui_session_column(&mut self, ui: &mut egui::Ui, height: f32) {
        let width = self.session_list_w;
        let selected = (!self.session_library_open
            && self.show_session_inspector
            && self.session_inspector_side == SidebarSide::Left)
            .then(|| {
                self.selected_session.and_then(|id| {
                    self.store
                        .sessions
                        .iter()
                        .find(|session| session.id == id)
                        .cloned()
                })
            })
            .flatten();
        ui.vertical(|ui| {
            ui.set_width(width);
            ui.set_min_height(height);
            // One id for the list stacked or not, so folding the panel keeps
            // its scroll position.
            let Some(session) = selected else {
                ui.push_id("session-column-list", |ui| {
                    self.ui_terminal_session_tabs(ui);
                });
                return;
            };
            const RULE_H: f32 = 6.0;
            let (list_h, panel_h) =
                stacked_sidebar_heights(height - RULE_H, self.session_list_split);
            ui.allocate_ui_with_layout(
                egui::vec2(width, list_h),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    ui.set_min_height(list_h);
                    ui.push_id("session-column-list", |ui| {
                        self.ui_terminal_session_tabs(ui);
                    });
                },
            );
            let palette = self.store.theme.palette();
            let rule = ui.allocate_response(egui::vec2(width, RULE_H), egui::Sense::drag());
            if rule.hovered() || rule.dragged() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
            }
            if rule.dragged() {
                let delta = rule.drag_delta().y;
                let usable = height - RULE_H;
                // A column too short for both minimums is halved whatever the
                // split says; a drag there would only overwrite the split.
                if delta.is_finite() && usable >= SIDEBAR_SECTION_MIN_H + SIDEBAR_PANEL_MIN_H {
                    self.session_list_split = ((list_h + delta) / usable).clamp(0.0, 1.0);
                }
            }
            ui.painter().line_segment(
                [
                    egui::pos2(rule.rect.left(), rule.rect.center().y),
                    egui::pos2(rule.rect.right(), rule.rect.center().y),
                ],
                egui::Stroke::new(1.0, palette.border_subtle),
            );
            let project = self
                .store
                .projects
                .iter()
                .find(|project| project.id == session.project_id)
                .cloned();
            self.ensure_initial_prompt_turn(session.id);
            ui.allocate_ui_with_layout(
                egui::vec2(width, panel_h),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    ui.push_id("session-column-panel", |ui| {
                        self.ui_session_inspector(
                            ui,
                            &session,
                            project.as_ref(),
                            palette,
                            width,
                            panel_h,
                        );
                    });
                },
            );
        });
    }

    /// A side-panel tab being dragged: the hint beside the pointer, and on
    /// release past the middle of `area` (outside a dead zone), the panel
    /// moves to that side.
    fn ui_sidebar_tab_drop(&mut self, ui: &egui::Ui, area: egui::Rect, palette: &Palette) {
        if let Some((tab, origin_side)) = self.dragging_sidebar_tab {
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.dragging_sidebar_tab = None;
            } else {
                ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
                let target_side = origin_side.opposite();
                let hint = match target_side {
                    SidebarSide::Left => tr("サイドバーを左側に移動"),
                    SidebarSide::Right => tr("サイドバーを右側に移動"),
                };
                egui::show_tooltip(
                    ui.ctx(),
                    ui.layer_id(),
                    egui::Id::new("sidebar_tab_drag_tooltip"),
                    |ui| {
                        ui.label(
                            RichText::new(tf!("{tab} · {hint}", tab = tab.label(), hint = hint))
                                .color(palette.text_strong),
                        );
                    },
                );

                if ui.input(|i| i.pointer.button_released(egui::PointerButton::Primary)) {
                    let pos = ui.input(|i| i.pointer.hover_pos());
                    if let Some(side) =
                        pos.and_then(|pos| sidebar_drop_side(origin_side, pos, area))
                    {
                        self.session_inspector_side = side;
                        let notice = match side {
                            SidebarSide::Left => tr("サイドバーを左側に移動しました"),
                            SidebarSide::Right => tr("サイドバーを右側に移動しました"),
                        };
                        self.notice_briefly(notice);
                    }
                    self.dragging_sidebar_tab = None;
                }
            }
        }
    }
}

/// The message field and the two buttons under the file list.
impl OperonApp {
    pub(crate) fn ui_commit_box(
        &mut self,
        ui: &mut egui::Ui,
        palette: &Palette,
        project: &Project,
    ) {
        if let Some((_msg, _files, error)) = self.last_commit_failure.get(&project.id).cloned() {
            let summary = summarize_commit_failure(&error);
            ui.add_space(SPACE_XS);
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!(
                            "{} {}",
                            ICON_ATTENTION,
                            tf!("コミット失敗: {summary}", summary = summary)
                        ))
                        .color(palette.warning)
                        .strong(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(tr("閉じる")).clicked() {
                            self.last_commit_failure.remove(&project.id);
                        }
                        if ui.button(tr("AIで修正")).clicked() {
                            self.fix_commit_failure_with_ai(project, ui.ctx());
                        }
                    });
                });
            });
            ui.add_space(SPACE_XS);
        }
        let selected = self.staged_selection(project.id).len();
        ui.add(
            egui::TextEdit::multiline(&mut self.commit_message_input)
                .desired_rows(3)
                .desired_width(f32::INFINITY)
                .hint_text(tr("コミットメッセージ")),
        );
        ui.horizontal(|ui| {
            let drafting = self
                .background_tasks
                .contains(&BackgroundKey::CommitMessage(project.id));
            // Both buttons write the same index, so both go dark while either
            // is working. The word on the button still says which one it is.
            let busy = OperonApp::git_worktree_busy(&self.background_tasks, project.id);
            // Drawn only when there is something to draw it with. There is
            // nothing to explain from this screen and nothing to fix here.
            if let Some(agent) = commit_message_agent(&self.tools) {
                if ui
                    .add_enabled(
                        !busy,
                        egui::Button::new(if drafting {
                            tr("書いています…")
                        } else {
                            tr("AI に書かせる")
                        }),
                    )
                    .clicked()
                {
                    self.draft_commit_message(project);
                }
                ui.label(
                    RichText::new(agent_short_label(agent))
                        .size(11.5)
                        .color(palette.text_muted),
                );
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add_enabled(
                        selected > 0 && !busy,
                        egui::Button::new(tf!("{p0} ファイルをコミット", p0 = selected)),
                    )
                    .clicked()
                {
                    self.record_staged_change(project);
                }
            });
        });
    }

    /// Where this branch stands against the remote, and the one button that
    /// changes it. Drawn even with a clean tree, because a branch can be ahead
    /// with nothing left to commit — which is exactly when somebody forgets.
    pub(crate) fn ui_push_row(&mut self, ui: &mut egui::Ui, palette: &Palette, project: &Project) {
        if !self.upstream_cache.contains_key(&project.id) {
            self.request_upstream_state(project);
        }
        let Some(state) = self.upstream_cache.get(&project.id).cloned() else {
            return;
        };
        ui.horizontal(|ui| {
            let sentence = match (&state.upstream, state.ahead, state.behind) {
                (None, _, _) => tr("このブランチは origin にまだありません。").into(),
                (Some(upstream), 0, 0) => {
                    tf!("{p0} と同じ状態です。", p0 = upstream)
                }
                (Some(upstream), ahead, 0) => tf!(
                    "{p0} より {p1} コミット先行しています。",
                    p0 = upstream,
                    p1 = ahead
                ),
                (Some(upstream), 0, behind) => tf!(
                    "{p0} より {p1} コミット遅れています。",
                    p0 = upstream,
                    p1 = behind
                ),
                (Some(upstream), ahead, behind) => tf!(
                    "{p0} より {p1} コミット先行・{p2} コミット遅れています。",
                    p0 = upstream,
                    p1 = ahead,
                    p2 = behind
                ),
            };
            ui.label(RichText::new(sentence).size(12.0).color(palette.text_muted));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let running = OperonApp::git_worktree_busy(&self.background_tasks, project.id);
                let pushable = state.behind == 0 && (state.upstream.is_none() || state.ahead > 0);
                if ui
                    .add_enabled(pushable && !running, egui::Button::new(tr("push")))
                    .clicked()
                {
                    self.push_project(project);
                }
                if self.tools.gh && ui.button(tr("PR を作成…")).clicked() {
                    let branch = state.upstream.as_deref().unwrap_or("HEAD");
                    self.pending_pr_modal = Some((
                        project.id,
                        project.path.clone(),
                        format!("feat: changes from {branch}"),
                        String::new(),
                        false,
                    ));
                }
            });
        });
    }

    pub(crate) fn request_upstream_state(&mut self, project: &Project) {
        let project_id = project.id;
        let path = project.path.clone();
        self.spawn_background(BackgroundKey::Upstream(project_id), move || {
            BackgroundResult::UpstreamRead {
                project_id,
                result: git_upstream_state(&path).map_err(|error| error.to_string()),
            }
        });
    }

    pub(crate) fn ui_prompt_timeline(
        &mut self,
        ui: &mut egui::Ui,
        palette: &Palette,
        session_id: Uuid,
    ) {
        let turns = self
            .session_prompt_turns
            .get(&session_id)
            .cloned()
            .unwrap_or_default();
        let turns_count = turns.len();

        // The frame, the title, and the count are the side panel's tab now,
        // and the jump to the newest line is on the terminal it scrolls.

        if turns.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(SPACE_LG);
                ui.label(
                    RichText::new(tr("プロンプトの入力を待っています…"))
                        .color(palette.text_muted)
                        .size(12.0),
                );
            });
            return;
        }

        // The other half of the id the terminal pane names in
        // `terminal_rows`: these two scroll areas are siblings under one
        // `horizontal_top`, and an unnamed pair there shares an id.
        egui::ScrollArea::vertical()
            .id_salt("prompt-timeline")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = SPACE_SM;
                for turn in turns.iter().rev() {
                    let is_latest = turn.turn == turns_count;
                    let card_fill = if is_latest && !turn.completed {
                        palette.card
                    } else {
                        palette.panel
                    };
                    let card_stroke = if is_latest && !turn.completed {
                        egui::Stroke::new(1.0, palette.accent)
                    } else {
                        egui::Stroke::new(1.0, palette.border_subtle)
                    };

                    let mut jump_target = None;
                    let card_response = egui::Frame::default()
                        .fill(card_fill)
                        .stroke(card_stroke)
                        .inner_margin(egui::Margin::symmetric(10, 8))
                        .corner_radius(egui::CornerRadius::same(RADIUS_CONTROL))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.horizontal(|ui| {
                                let badge_text = tf!("#{p0} ターン", p0 = turn.turn);
                                ui.label(RichText::new(badge_text).strong().size(12.0).color(
                                    if is_latest {
                                        palette.accent
                                    } else {
                                        palette.text_muted
                                    },
                                ));
                                let elapsed_secs = turn.timestamp.elapsed().as_secs();
                                let time_str = if elapsed_secs < 60 {
                                    format!("{elapsed_secs}s")
                                } else if elapsed_secs < 3600 {
                                    format!("{}m", elapsed_secs / 60)
                                } else {
                                    format!("{}h", elapsed_secs / 3600)
                                };
                                ui.label(
                                    RichText::new(time_str).size(11.5).color(palette.text_faint),
                                );

                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if turn.completed {
                                            ui.label(
                                                RichText::new("✓ 完了")
                                                    .size(11.5)
                                                    .color(palette.success),
                                            );
                                        } else {
                                            ui.label(
                                                RichText::new("● 実行中")
                                                    .size(11.5)
                                                    .color(palette.accent),
                                            );
                                        }
                                    },
                                );
                            });

                            ui.add_space(2.0);
                            let first_line = turn.prompt.lines().next().unwrap_or(&turn.prompt);
                            let excerpt: String = first_line.chars().take(80).collect();
                            ui.add(
                                egui::Label::new(
                                    RichText::new(excerpt).size(12.0).color(palette.text_strong),
                                )
                                .truncate(),
                            );
                        });

                    let card_interact = ui.interact(
                        card_response.response.rect,
                        ui.make_persistent_id(("turn-card", session_id, turn.turn)),
                        egui::Sense::click(),
                    );
                    if card_interact.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    if card_interact.clicked() {
                        jump_target = Some(turn.line);
                    }

                    if let Some(target_line) = jump_target {
                        let total_lines = self
                            .terminal_layouts
                            .get(&session_id)
                            .map(|l| l.len())
                            .unwrap_or(0);
                        let line = target_line.min(total_lines.saturating_sub(1));
                        let search = self.terminal_search.entry(session_id).or_default();
                        search.scroll_to = Some(line);
                    }
                }
            });
    }
}

/// Whether modifiers include Command (⌘) to trigger opening a terminal target externally.
pub(crate) fn is_terminal_cmd_modifier(modifiers: &egui::Modifiers) -> bool {
    modifiers.command || modifiers.mac_cmd
}

/// Whether modifiers include Command (⌘) or Option (⌥) to trigger opening a terminal target.
pub(crate) fn is_terminal_open_modifier(modifiers: &egui::Modifiers) -> bool {
    is_terminal_cmd_modifier(modifiers) || modifiers.alt
}

/// A one-line label cut short at `max_width` rather than at whatever width is
/// left, for a row that has more than one label to fit.
fn capped_label(ui: &mut egui::Ui, max_width: f32, text: RichText) {
    ui.scope(|ui| {
        ui.set_max_width(max_width);
        ui.add(egui::Label::new(text).truncate());
    });
}

/// One number on the 概要 tab, under what it counts. A string, because a
/// count still loading is 「—」 rather than a zero that is not true yet.
fn overview_tile(ui: &mut egui::Ui, palette: &Palette, title: &str, value: &str) {
    card_frame(palette).show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        ui.label(RichText::new(title).size(12.0).color(palette.text_muted));
        ui.label(
            RichText::new(value)
                .size(20.0)
                .strong()
                .color(palette.text_strong),
        );
    });
}

impl OperonApp {
    pub(crate) fn ui_land_worktree_modal(&mut self, ctx: &egui::Context, palette: &Palette) {
        let Some((project_id, worktree_path, branch)) = self.pending_land_worktree.clone() else {
            return;
        };
        let Some(project) = self
            .store
            .projects
            .iter()
            .find(|p| p.id == project_id)
            .cloned()
        else {
            self.pending_land_worktree = None;
            return;
        };

        let mut open = true;
        egui::Window::new(tr("worktree を main にマージ"))
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .fixed_size([520.0, 240.0])
            .show(ctx, |ui| {
                ui.label(
                    RichText::new(tr("現在の worktree の変更を main ブランチにマージします。"))
                        .strong(),
                );
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label(tr("マージ元:"));
                    ui.monospace(&branch);
                });
                ui.horizontal(|ui| {
                    ui.label(tr("マージ先:"));
                    let base_name = detect_worktree_base(&project.path)
                        .map(|b| {
                            b.display
                                .strip_prefix("origin/")
                                .unwrap_or(&b.display)
                                .to_owned()
                        })
                        .unwrap_or_else(|| "main".into());
                    ui.monospace(base_name);
                });
                ui.add_space(8.0);
                ui.checkbox(
                    &mut self.land_auto_cleanup,
                    tr("マージ成功後に worktree とブランチを自動で削除する"),
                );
                ui.add_space(14.0);
                ui.horizontal(|ui| {
                    if ui.button(tr("キャンセル")).clicked() {
                        self.pending_land_worktree = None;
                    }
                    if ui
                        .button(
                            RichText::new(tr("マージを実行する"))
                                .strong()
                                .color(palette.accent_soft),
                        )
                        .clicked()
                    {
                        self.land_worktree(
                            &project,
                            &worktree_path,
                            &branch,
                            self.land_auto_cleanup,
                        );
                    }
                });
            });
        if !open {
            self.pending_land_worktree = None;
        }
    }

    pub(crate) fn ui_create_pr_modal(&mut self, ctx: &egui::Context, palette: &Palette) {
        let Some((project_id, worktree_path, mut title, mut body, mut draft)) =
            self.pending_pr_modal.clone()
        else {
            return;
        };
        let Some(project) = self
            .store
            .projects
            .iter()
            .find(|p| p.id == project_id)
            .cloned()
        else {
            self.pending_pr_modal = None;
            return;
        };

        let mut open = true;
        egui::Window::new(tr("プルリクエストを作成"))
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .fixed_size([560.0, 320.0])
            .show(ctx, |ui| {
                ui.label(tr("タイトル:"));
                ui.add(egui::TextEdit::singleline(&mut title).desired_width(f32::INFINITY));
                ui.add_space(6.0);
                ui.label(tr("説明:"));
                ui.add(
                    egui::TextEdit::multiline(&mut body)
                        .desired_rows(6)
                        .desired_width(f32::INFINITY),
                );
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button(tr("AI でドラフト生成")).clicked() {
                        if let Ok(diff) = git_working_tree_diff(&worktree_path, None) {
                            if !diff.is_empty() {
                                body = tf!(
                                    "## 概要\n\nAIエージェントによる変更内容:\n\n{diff_summary}",
                                    diff_summary = truncate_chars(&diff, 500)
                                );
                            }
                        }
                    }
                    ui.checkbox(&mut draft, tr("下書き (Draft PR) として作成"));
                });
                ui.add_space(14.0);
                ui.horizontal(|ui| {
                    if ui.button(tr("キャンセル")).clicked() {
                        self.pending_pr_modal = None;
                    }
                    if ui
                        .button(
                            RichText::new(tr("PR を作成する"))
                                .strong()
                                .color(palette.accent_soft),
                        )
                        .clicked()
                    {
                        self.create_pull_request(&project, &worktree_path, &title, &body, draft);
                    }
                });
            });
        if !open {
            self.pending_pr_modal = None;
        } else if let Some(ref mut modal) = self.pending_pr_modal {
            modal.2 = title;
            modal.3 = body;
            modal.4 = draft;
        }
    }

    pub(crate) fn ui_run_script_modal(&mut self, ctx: &egui::Context, palette: &Palette) {
        let Some(pending) = self.pending_run_script.clone() else {
            return;
        };
        let mut open = true;
        egui::Window::new(tr("実行スクリプトの確認"))
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .fixed_size([520.0, 300.0])
            .show(ctx, |ui| {
                ui.label(RichText::new(tr(".operon/run.sh を実行しようとしています。")).strong());
                ui.label(tr("内容を確認して実行を承認してください:"));
                ui.add_space(6.0);
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(140.0)
                        .show(ui, |ui| {
                            for line in &pending.script.preview {
                                ui.monospace(line);
                            }
                        });
                });
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button(tr("キャンセル")).clicked() {
                        self.pending_run_script = None;
                    }
                    if ui
                        .button(
                            RichText::new(tr("承認して実行"))
                                .strong()
                                .color(palette.accent_soft),
                        )
                        .clicked()
                    {
                        match crate::git::setup::decide_run_script_run(&pending.worktree, &pending.script.digest) {
                            crate::git::setup::SetupRunDecision::Run(_) => {
                                let script_path = pending.worktree.join(RUN_SCRIPT_RELATIVE_PATH);
                                let cmd_str =
                                    format!("bash {}", shell_quote(&script_path.display().to_string()));
                                let session_name = format!(
                                    "run-{}",
                                    pending.script.digest.chars().take(8).collect::<String>()
                                );
                                self.launch_custom_script_session(
                                    pending.project,
                                    &pending.worktree,
                                    &session_name,
                                    &cmd_str,
                                );
                                self.pending_run_script = None;
                            }
                            crate::git::setup::SetupRunDecision::Changed(script) => {
                                self.notice = Some(tf!(
                                    "{p0} の内容が変わったため実行しませんでした。もう一度確認してください。",
                                    p0 = RUN_SCRIPT_RELATIVE_PATH
                                ));
                                self.pending_run_script = Some(PendingRunScript {
                                    script,
                                    ..pending
                                });
                            }
                            crate::git::setup::SetupRunDecision::Unreadable => {
                                self.notice = Some(tf!(
                                    "{p0} を読み直せませんでした。実行していません。",
                                    p0 = RUN_SCRIPT_RELATIVE_PATH
                                ));
                                self.pending_run_script = None;
                            }
                        }
                    }
                });
            });
        if !open {
            self.pending_run_script = None;
        }
    }
}
