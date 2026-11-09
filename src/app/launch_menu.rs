//! The tab strip's + menu (change 273): what it offers, and starting a plain
//! terminal or an agent CLI in the pane whose + was pressed. An agent starts
//! with its CLI's pinned or last-used options; options that need the danger
//! acknowledgement open the launch sheet instead.
//!
//! A child of `src/app.rs`, like `screens.rs`, so that neither of them grows
//! past the module-size band and `src/main.rs` keeps its module list.

use crate::*;

/// What a tab strip's + offers, top to bottom (change 273): the session agent
/// value of a plain terminal, then the agent CLIs in the order the person
/// named them. Each is also what the row's mark is looked up by.
pub(crate) const WORKSPACE_LAUNCH_CHOICES: [&str; 4] = ["terminal", "claude", "codex", "gemini"];

/// The popup the + of `group`'s pane opens, or the empty workspace's for `None`.
fn workspace_launch_menu_id(group: Option<Uuid>) -> egui::Id {
    egui::Id::new(("workspace-launch-menu", group))
}

/// The + menu's width: the longest name, 「Antigravity」, with its mark and the
/// button padding, so the four rows line up as one column.
const LAUNCH_MENU_WIDTH: f32 = 168.0;

/// What choosing an agent under a pane's + does (change 273), decided before
/// anything is written so that the decision can be checked without starting a
/// CLI.
pub(crate) enum PaneAgentLaunch {
    /// Every check passed: record the session and start it.
    Start(PaneAgentStart),
    /// The remembered options need the danger acknowledgement and nothing
    /// carries it, so the launch sheet asks: opened on `project_id`, in the
    /// pane's folder `path_input`.
    Confirm {
        project_id: Uuid,
        path_input: String,
    },
}

/// A checked launch of one agent CLI into a pane: where, what command, and
/// under which login.
pub(crate) struct PaneAgentStart {
    pub(crate) project: Project,
    pub(crate) worktree_path: PathBuf,
    pub(crate) agent: &'static str,
    pub(crate) command: String,
    pub(crate) account: Option<Uuid>,
}

impl OperonApp {
    /// The project and folder a pane's + starts in: the pane's current tab's,
    /// else the workspace's. The folder is a session tab's worktree, or empty
    /// for the project's own folder. The plus belongs to its pane, not a
    /// possibly stale launch-sheet draft.
    pub(crate) fn workspace_pane_place(&self, group: Option<Uuid>) -> Option<(Project, String)> {
        let tab = group
            .and_then(|id| self.workspace.groups.iter().find(|g| g.id == id))
            .and_then(|g| g.tabs.get(g.active))
            .or_else(|| self.workspace.active());
        let project_id = tab
            .and_then(|tab| self.workspace_tab_project(tab))
            .or(self.workspace_project)
            .or(self.selected_project);
        let project = self
            .store
            .projects
            .iter()
            .find(|p| Some(p.id) == project_id)
            .cloned()?;
        let path = match tab {
            Some(WorkspaceTab::Session(id)) => self
                .store
                .sessions
                .iter()
                .find(|s| s.id == *id)
                .and_then(|s| s.worktree_path.as_ref())
                .map(|path| path.display().to_string())
                .unwrap_or_default(),
            _ => String::new(),
        };
        Some((project, path))
    }

    /// What choosing `agent` under a pane's + would do (change 273), with no
    /// side effect. The options are the ones the launch sheet would open with
    /// for that CLI — the pin when it is for this CLI, else what the sheet last
    /// used — and every refusal is the sentence `launch_session` gives for it.
    ///
    /// The danger acknowledgement is decided before tmux is asked about: the
    /// sheet that asks for it does not need tmux to open, and a combination
    /// that skips permissions must never be one check away from starting.
    pub(crate) fn workspace_agent_launch(
        &self,
        group: Option<Uuid>,
        agent: &str,
    ) -> UiResult<PaneAgentLaunch> {
        let Some(agent) = AGENTS.iter().copied().find(|known| *known == agent) else {
            return Err(tr("エージェントの起動オプションが不正です。").into());
        };
        let Some((project, path_input)) = self.workspace_pane_place(group) else {
            return Err(tr("先にプロジェクトを選んでください。").into());
        };
        if !self.tools.agent_available(agent) {
            return Err(tf!("{selected_agent} が PATH 上に見つかりません。インストールしてから、「設定」で「ツールを再確認」を押してください。", selected_agent = agent));
        }
        let pinned = self
            .recent_agent_settings
            .pinned
            .as_ref()
            .filter(|pin| pin.agent == agent);
        let settings = pinned
            .map(|pin| pin.settings.clone())
            .or_else(|| self.recent_agent_settings.agents.get(agent).cloned())
            .unwrap_or_default();
        let account = pinned
            .and_then(|pin| pin.account)
            .filter(|id| self.accounts.get(*id).is_some())
            .filter(|_| agent_supports_accounts(agent));
        let acknowledged = pinned.is_some_and(|pin| pin.acknowledged);
        if !acknowledged
            && launch_needs_acknowledgement(
                agent,
                &settings.mode,
                &settings.flags,
                "",
                &settings.extra_args,
            )
        {
            return Ok(PaneAgentLaunch::Confirm {
                project_id: project.id,
                path_input,
            });
        }
        if self.worktree_mutation_running(project.id) {
            return Err(tr("worktree 操作の完了を待ってください。").into());
        }
        if !self.tools.tmux {
            return Err(tr("tmux が必要です。「設定」を開いてください。").into());
        }
        account_ready_to_launch(&self.accounts, account)?;
        let command = build_agent_launch_command(
            agent,
            "",
            &settings.model,
            &settings.mode,
            &settings.effort,
            &settings.flags,
            &settings.extra_args,
        )?;
        let worktree_path = resolve_selected_session_path(
            project.id,
            &project.path,
            Some(project.id),
            &path_input,
        )?;
        Ok(PaneAgentLaunch::Start(PaneAgentStart {
            project,
            worktree_path,
            agent,
            command,
            account,
        }))
    }

    /// Records a checked pane launch as a new session in that pane and
    /// returns its ID; the caller starts it. The launch sheet's draft and the
    /// remembered options are not read or written here.
    pub(crate) fn open_workspace_agent_session(
        &mut self,
        group: Option<Uuid>,
        start: &PaneAgentStart,
    ) -> Option<Uuid> {
        let id = Uuid::new_v4();
        let (agent_command, native_session_id) =
            command_with_managed_native_session_id(start.agent, start.command.clone(), id);
        let goal = goal_with_native_tracking_token(start.agent, String::new(), id);
        let name = self.next_session_name(start.project.id);
        let previous_store = self.store.clone();
        self.store.sessions.push(Session {
            id,
            project_id: start.project.id,
            name,
            goal,
            agent: start.agent.to_owned(),
            tmux_name: managed_tmux_name(id),
            created_at: now(),
            launched_at: None,
            status: SessionStatus::Starting,
            worktree_path: Some(start.worktree_path.clone()),
            branch: None,
            agent_command: agent_command.clone(),
            native_session_id,
            native_session_path: None,
            origin: None,
            depends_on: Vec::new(),
        });
        if !self.persist() {
            self.store = previous_store;
            return None;
        }
        self.remember_session_account(id, start.account);
        if let Some(group) = group {
            self.workspace.focused = Some(group);
        }
        self.launch_progress = Some(LaunchProgress {
            session_id: id,
            project_name: start.project.name.clone(),
            worktree_path: start.worktree_path.clone(),
            agent: start.agent.to_owned(),
            agent_command,
            goal: String::new(),
            started_at: Instant::now(),
            failure: None,
            dismissed: true,
        });
        self.select_session(id);
        self.page = Page::Sessions;
        self.session_library_open = false;
        Some(id)
    }

    /// The agent rows of a pane's + menu (change 273): start the CLI in that
    /// pane, or, when its options need the danger acknowledgement, open the
    /// launch sheet on it in the pane's folder.
    pub(crate) fn launch_workspace_agent(&mut self, group: Option<Uuid>, agent: &str) {
        match self.workspace_agent_launch(group, agent) {
            Err(reason) => self.notice = Some(reason),
            Ok(PaneAgentLaunch::Start(start)) => {
                if let Some(id) = self.open_workspace_agent_session(group, &start) {
                    self.start_session(id);
                }
            }
            Ok(PaneAgentLaunch::Confirm {
                project_id,
                path_input,
            }) => {
                self.open_project_session_setup(project_id);
                // Switched by hand, not through `select_agent`: that records
                // the form into the remembered options and writes the file,
                // and this path only reads them.
                if self.selected_agent != agent {
                    self.selected_agent = agent.to_owned();
                    self.apply_agent_launch_settings(agent);
                    self.acknowledged_launch = None;
                    self.resume_conversation_input.clear();
                    self.agent_account_input = None;
                }
                self.session_path_project = Some(project_id);
                self.session_path_input = path_input;
            }
        }
    }

    /// The menu under a tab strip's + (change 273): a plain terminal or one
    /// of the agent CLIs, started in `group`'s pane. Pressing the + opens and
    /// closes it; a choice, a click elsewhere or Escape closes it. The choice
    /// is acted on after the popup is drawn, so the launch sees a closed menu.
    pub(crate) fn ui_workspace_launch_menu(
        &mut self,
        ui: &mut egui::Ui,
        add: &egui::Response,
        group: Option<Uuid>,
    ) {
        let popup = workspace_launch_menu_id(group);
        if add.clicked() {
            ui.memory_mut(|memory| memory.toggle_popup(popup));
        }
        let tools = &self.tools;
        let chosen = egui::popup_below_widget(
            ui,
            popup,
            add,
            egui::PopupCloseBehavior::CloseOnClick,
            |ui| {
                ui.set_min_width(LAUNCH_MENU_WIDTH);
                ui.spacing_mut().item_spacing.y = SPACE_XS;
                let mut chosen = None;
                for choice in WORKSPACE_LAUNCH_CHOICES {
                    let (label, enabled) = if choice == "terminal" {
                        (agent_short_label(choice), true)
                    } else {
                        (agent_choice_copy(choice).0, tools.agent_available(choice))
                    };
                    let mark = session_mark_texture(ui.ctx(), choice);
                    if launch_choice_button(ui, mark, label, enabled, tr("未準備（設定で確認）"))
                        .clicked()
                    {
                        chosen = Some(choice);
                    }
                }
                chosen
            },
        )
        .flatten();
        match chosen {
            Some("terminal") => self.launch_workspace_terminal(group),
            Some(agent) => self.launch_workspace_agent(group, agent),
            None => {}
        }
    }

    /// An Escape that closes a + menu belongs to the menu. Called before any
    /// pane is drawn, so no terminal — this pane's or another's — sends it on
    /// and interrupts its agent's turn. Other popups, such as the launch
    /// sheet's lists, keep their Escape: only the + menus' ids are asked about.
    pub(crate) fn take_launch_menu_escape(&self, ui: &mut egui::Ui) {
        let launch_menu_open = std::iter::once(None)
            .chain(self.workspace.groups.iter().map(|g| Some(g.id)))
            .any(|group| ui.memory(|m| m.is_popup_open(workspace_launch_menu_id(group))));
        if launch_menu_open
            && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
        {
            ui.memory_mut(|m| m.close_popup());
        }
    }
}
