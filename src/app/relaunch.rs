//! What a relaunch puts back (change 270): the workspace's tabs, splits, and
//! front tabs, kept in `workspace-layout.json` beside the store; the sessions
//! that were running at the last quit and whose terminals are gone, started
//! again in place; and the kind a terminal takes from the CLI running in it.

use crate::*;

/// Whether a launch brings its session forward — selects it, shows the
/// session page, and says it is starting — or starts it where it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LaunchFocus {
    Bring,
    Keep,
}

/// What `prepare_native_resume` found: a conversation to reopen, or an ID
/// whose transcript was never written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum NativeResume {
    Ready {
        native_id: String,
        path: Option<PathBuf>,
        /// Bytes cut from a transcript whose last line a crash left torn.
        repaired_bytes: Option<usize>,
    },
    NoTranscript,
}

/// How a session found gone at the first look after a start is started again
/// (change 270): a plain terminal opens a shell, a CLI with a saved
/// conversation reopens it. Anything else — a custom command, a recovered
/// terminal, a CLI with no conversation to reopen — is not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RelaunchKind {
    Shell,
    Resume(CliProvider),
}

pub(crate) fn relaunch_kind(session: &Session) -> Option<RelaunchKind> {
    if session.agent == "terminal" && session.agent_command.is_empty() {
        return Some(RelaunchKind::Shell);
    }
    let provider = cli_provider_for_agent(&session.agent)?;
    session
        .native_session_id
        .as_ref()
        .map(|_| RelaunchKind::Resume(provider))
}

/// Whether `session` began as a terminal rather than as a CLI launch, and
/// which kind of terminal: its recorded origin, or for a record written before
/// origins were, what its name and kind say. A terminal keeps this provenance
/// after it takes the kind of the CLI running in it, which is how a second CLI
/// typed into it later is recognised too (change 270).
pub(crate) fn terminal_provenance(session: &Session) -> Option<SessionOrigin> {
    match session.origin.or_else(|| legacy_session_origin(session)) {
        Some(provenance @ (SessionOrigin::PlainTerminal | SessionOrigin::Adopted)) => {
            Some(provenance)
        }
        _ => None,
    }
}

/// A terminal the person typed a CLI into is a session of that CLI from here
/// on (change 270): the mark, the label, the state reading, and the resume all
/// read the kind, which was otherwise decided once, at creation. A session
/// launched as a CLI keeps its kind. Whether the kind moved.
pub(crate) fn take_cli_kind(session: &mut Session, provider: CliProvider) -> bool {
    let Some(provenance) =
        terminal_provenance(session).filter(|_| session.agent != provider.agent())
    else {
        return false;
    };
    session.agent = provider.agent().to_owned();
    session.goal.clear();
    session.origin = Some(provenance);
    true
}

impl OperonApp {
    /// Puts back the workspace a previous run left (change 270). A project
    /// whose saved tabs cannot be drawn as saved is left to today's start, and
    /// its sessions are not counted as placed, so its live ones come back as
    /// tabs rather than staying hidden.
    pub(crate) fn restore_workspace_layout(&mut self, layout: &WorkspaceLayout) {
        let Some(project) = layout
            .project
            .filter(|id| self.store.projects.iter().any(|project| project.id == *id))
        else {
            return;
        };
        let sessions = &self.store.sessions;
        let mut placed = layout
            .placed
            .iter()
            .copied()
            .filter(|id| sessions.iter().any(|session| session.id == *id))
            .collect::<HashSet<_>>();
        let mut selected = None;
        for saved in &layout.workspaces {
            if !self
                .store
                .projects
                .iter()
                .any(|known| known.id == saved.project)
            {
                continue;
            }
            let in_project = |id: Uuid| {
                sessions
                    .iter()
                    .any(|session| session.id == id && session.project_id == saved.project)
            };
            let Some(tabs) = WorkspaceTabs::from_saved(saved, in_project) else {
                placed.retain(|id| !in_project(*id));
                continue;
            };
            let chosen = saved.selected.filter(|id| in_project(*id));
            if saved.project == project {
                self.workspace = tabs;
                selected = chosen;
            } else {
                self.project_workspaces
                    .insert(saved.project, (tabs, chosen));
            }
        }
        self.workspace_seen_sessions = placed;
        self.workspace_project = Some(project);
        self.selected_project = Some(project);
        self.workspace_initialized = true;
        self.set_terminal_session_context(selected);
    }

    /// Every project's tabs as a relaunch keeps them, in an order that does
    /// not depend on the map's, so two snapshots of one workspace compare
    /// equal. `None` until the workspace has been laid out once.
    pub(crate) fn workspace_layout_snapshot(&self) -> Option<WorkspaceLayout> {
        let project = self.workspace_project?;
        let selected = self.selected_session.filter(|id| {
            self.store
                .sessions
                .iter()
                .any(|session| session.id == *id && session.project_id == project)
        });
        let mut workspaces = vec![self.workspace.to_saved(project, selected)];
        workspaces.extend(
            self.project_workspaces
                .iter()
                .map(|(id, (tabs, selected))| tabs.to_saved(*id, *selected)),
        );
        workspaces.sort_by_key(|saved| saved.project);
        let mut placed = self
            .workspace_seen_sessions
            .iter()
            .copied()
            .collect::<Vec<_>>();
        placed.sort();
        Some(WorkspaceLayout {
            project: Some(project),
            workspaces,
            placed,
        })
    }

    /// Writes the workspace layout when it differs from the last one written.
    /// Called on the two-second tick and at quit. A failed write is tried
    /// again at the next tick, and said once per streak of failures.
    pub(crate) fn save_workspace_layout_if_changed(&mut self) {
        let Some(snapshot) = self.workspace_layout_snapshot() else {
            return;
        };
        if self.saved_workspace_layout.as_ref() == Some(&snapshot) {
            return;
        }
        match save_workspace_layout(&self.data_file, &snapshot) {
            Ok(()) => {
                self.saved_workspace_layout = Some(snapshot);
                self.workspace_layout_save_failed = false;
            }
            Err(error) => {
                if !self.workspace_layout_save_failed {
                    self.notice = Some(tf!(
                        "タブの配置を保存できませんでした: {error}",
                        error = error
                    ));
                }
                self.workspace_layout_save_failed = true;
            }
        }
    }

    /// Whether `id` is outside the per-session automatic-restore cooldown
    /// window (`AUTO_RESTORE_COOLDOWN_SECONDS`).
    pub(crate) fn auto_restore_ready(&self, id: Uuid, now: Instant) -> bool {
        self.last_auto_restore.get(&id).is_none_or(|last| {
            now.saturating_duration_since(*last)
                >= Duration::from_secs(AUTO_RESTORE_COOLDOWN_SECONDS)
        })
    }

    /// Records an automatic restore timestamp for `id` after pruning removed
    /// sessions so `last_auto_restore` stays bounded by `store.sessions`.
    pub(crate) fn record_auto_restore(&mut self, id: Uuid, now: Instant) {
        let sessions = &self.store.sessions;
        self.last_auto_restore
            .retain(|known, _| sessions.iter().any(|session| session.id == *known));
        self.last_auto_restore.insert(id, now);
    }

    /// Clears the automatic-restore cooldown when a person manually retries or
    /// resumes `id`.
    pub(crate) fn clear_auto_restore_cooldown(&mut self, id: Uuid) {
        self.last_auto_restore.remove(&id);
    }

    /// Restores interrupted sessions in place at startup (`startup_ids`) and
    /// mid-run (`midrun_ids`), announces whichever group started, and returns
    /// the names of `Failed` sessions whose automatic restore declined so
    /// failure notifications still fire for them (change 291).
    pub(crate) fn relaunch_interrupted_sessions(
        &mut self,
        startup_ids: Vec<(Uuid, Option<String>)>,
        midrun_ids: Vec<(Uuid, Option<String>)>,
        now: Instant,
    ) -> Vec<String> {
        let mut declined_failures = Vec::new();
        let mut saw_error_notice = false;
        let mut startup_started = 0usize;
        for (id, failed_name) in startup_ids {
            let before = self.notice.clone();
            if self.relaunch_lost_session(id) {
                self.record_auto_restore(id, now);
                startup_started += 1;
            } else {
                if self.notice != before && !notice_is_brief(&self.notice, &self.brief_notice) {
                    saw_error_notice = true;
                }
                if let Some(name) = failed_name {
                    declined_failures.push(name);
                }
            }
        }
        let mut midrun_started = 0usize;
        for (id, failed_name) in midrun_ids {
            let before = self.notice.clone();
            if self.relaunch_lost_session(id) {
                self.record_auto_restore(id, now);
                midrun_started += 1;
            } else {
                if self.notice != before && !notice_is_brief(&self.notice, &self.brief_notice) {
                    saw_error_notice = true;
                }
                if let Some(name) = failed_name {
                    declined_failures.push(name);
                }
            }
        }
        if !self.store_retry_pending && !saw_error_notice {
            if midrun_started > 0 {
                self.notice_briefly(tf!(
                    "強制終了されたセッションを {count} 件自動で復元しました。",
                    count = midrun_started
                ));
            } else if startup_started > 0 {
                self.notice_briefly(tf!(
                    "前回開いていたセッションを {count} 件再開しました。",
                    count = startup_started
                ));
            }
        }
        declined_failures
    }

    /// Handles automatic restore after `BackgroundResult::TerminalResized`
    /// observed `TmuxState::Gone` and persisted `SessionStatus::Lost`.
    pub(crate) fn auto_restore_after_resize_gone(&mut self, session_id: Uuid, now: Instant) {
        let first_look = self.relaunch_candidates.remove(&session_id);
        let eligible = !self.setup_sessions.contains(&session_id)
            && self
                .store
                .sessions
                .iter()
                .find(|session| session.id == session_id)
                .is_some_and(|session| relaunch_kind(session).is_some())
            && self.auto_restore_ready(session_id, now);
        if !eligible {
            return;
        }
        if first_look {
            let _ = self.relaunch_interrupted_sessions(vec![(session_id, None)], Vec::new(), now);
        } else {
            let _ = self.relaunch_interrupted_sessions(Vec::new(), vec![(session_id, None)], now);
        }
    }

    /// One session of `relaunch_remembered_sessions`: the same record, the
    /// same tab, the same tmux name. A CLI reopens its conversation through
    /// the checks the manual 「再開」 makes, with the options it was launched
    /// with; a plain terminal opens a shell in its folder. A conversation
    /// whose transcript was never written is not started fresh by itself —
    /// that would send its first request again — and keeps its 「再試行」.
    /// Which sessions come here — Lost or Failed at the first look, never one
    /// the person exited — is `apply_session_poll`'s decision alone.
    pub(crate) fn relaunch_lost_session(&mut self, session_id: Uuid) -> bool {
        let Some(session) = self
            .store
            .sessions
            .iter()
            .find(|session| session.id == session_id)
            .cloned()
        else {
            return false;
        };
        let provider = match relaunch_kind(&session) {
            None => return false,
            Some(RelaunchKind::Shell) => {
                return self.launch_session_record(session_id, LaunchFocus::Keep)
            }
            Some(RelaunchKind::Resume(provider)) => provider,
        };
        let (native_id, path) = match self.prepare_native_resume(&session, provider) {
            Ok(NativeResume::Ready {
                native_id, path, ..
            }) => (native_id, path),
            Ok(NativeResume::NoTranscript) => return false,
            Err(notice) => {
                self.notice = Some(notice);
                return false;
            }
        };
        let previous_store = self.store.clone();
        if let Some(stored) = self
            .store
            .sessions
            .iter_mut()
            .find(|stored| stored.id == session_id)
        {
            stored.agent_command = resume_command_with_launch_options(
                provider,
                &session.agent,
                &session.agent_command,
                &native_id,
            );
            stored.native_session_id = Some(native_id);
            stored.native_session_path = path;
        }
        if !self.persist() {
            self.store = previous_store;
            return false;
        }
        self.launch_session_record(session_id, LaunchFocus::Keep)
    }
}
