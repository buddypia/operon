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

/// Interactive terminal and agent kinds whose pane should stay usable as a
/// login shell when their top-level command exits cleanly with status 0
/// (change 288). One-shot `.operon/run.sh` runs (`RUN_SCRIPT_AGENT`) and
/// synthetic test fixtures are excluded.
pub(crate) fn is_interactive_terminal_session(session: &Session) -> bool {
    matches!(
        session.agent.as_str(),
        "terminal" | "existing terminal" | "claude" | "codex" | "gemini" | "custom"
    )
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

    /// Queues interactive sessions that an older build saved as `Exited` while
    /// their workspace tab was still open so the first poll reopens them as
    /// live login shells instead of leaving `Pane is dead` on screen
    /// (change 288). Closed historical sessions stay `Exited`.
    pub(crate) fn reopen_exited_workspace_tabs_on_load(&mut self) {
        if !self.tools.tmux {
            return;
        }
        let mut open_tabs = self.workspace.session_ids().collect::<HashSet<_>>();
        for (tabs, _) in self.project_workspaces.values() {
            open_tabs.extend(tabs.session_ids());
        }
        if open_tabs.is_empty() {
            return;
        }
        let candidates = self
            .store
            .sessions
            .iter()
            .filter(|session| {
                session.status == SessionStatus::Exited
                    && open_tabs.contains(&session.id)
                    && !self.store.pending_cancellations.contains(&session.id)
                    && is_interactive_terminal_session(session)
                    && queued_dependency_count(&self.store.sessions, session.id) == 0
            })
            .map(|session| session.id)
            .collect::<Vec<_>>();
        self.exited_tabs_on_load.extend(candidates);
    }

    /// Converts a cleanly exited interactive session in place into a plain
    /// login shell in the same tab and tmux session, keeping its terminal
    /// provenance and scrollback while clearing finished CLI conversation
    /// pointers so future retries or typed CLIs start clean (change 288).
    /// Rolls back the whole record if launching the shell is refused or cannot
    /// be persisted.
    pub(crate) fn reopen_exited_session_as_shell(&mut self, session_id: Uuid) -> bool {
        if self.session_launch_running(session_id) {
            return false;
        }
        let Some(existing) = self
            .store
            .sessions
            .iter()
            .find(|session| session.id == session_id)
            .cloned()
        else {
            return false;
        };
        let origin = terminal_provenance(&existing).unwrap_or(SessionOrigin::PlainTerminal);
        let previous_store = self.store.clone();
        if let Some(stored) = self
            .store
            .sessions
            .iter_mut()
            .find(|stored| stored.id == session_id)
        {
            stored.agent = "terminal".into();
            stored.agent_command.clear();
            stored.goal.clear();
            stored.native_session_id = None;
            stored.native_session_path = None;
            stored.origin = Some(origin);
        }
        if !self.launch_session_record(session_id, LaunchFocus::Keep) {
            self.store = previous_store;
            return false;
        }
        self.hook_status.remove(&session_id);
        self.background_tasks
            .remove(&BackgroundKey::NativeSessionResolution(session_id));
        true
    }

    /// Whether enough time has passed since the last automatic restoration of
    /// `session_id` to allow another automatic recovery without crash-looping.
    pub(crate) fn auto_restore_ready(&self, session_id: Uuid, now: Instant) -> bool {
        self.last_auto_restore
            .get(&session_id)
            .is_none_or(|last| now.saturating_duration_since(*last) >= AUTO_RESTORE_COOLDOWN)
    }

    /// Clears the automatic-restore cooldown when the person explicitly starts
    /// or resumes a session.
    pub(crate) fn clear_auto_restore_cooldown(&mut self, session_id: Uuid) {
        self.last_auto_restore.remove(&session_id);
    }

    /// Starts again, in place, the sessions the first look found gone that
    /// were running when Operon last closed, and says how many it started
    /// unless a persistent notice was raised during the same poll.
    pub(crate) fn relaunch_remembered_sessions(
        &mut self,
        ids: Vec<Uuid>,
        now: Instant,
        persistent_notice: &mut bool,
    ) -> HashSet<Uuid> {
        let mut started = HashSet::new();
        for id in ids {
            let saved_notice = self.notice.take();
            let launched = self.relaunch_lost_session(id);
            if self.notice.is_some() {
                self.brief_notice = None;
                *persistent_notice = true;
            } else {
                self.notice = saved_notice;
            }
            if launched {
                self.last_auto_restore.insert(id, now);
                started.insert(id);
            }
        }
        if !started.is_empty() && !*persistent_notice {
            self.notice_briefly(tf!(
                "前回開いていたセッションを {count} 件再開しました。",
                count = started.len()
            ));
        }
        started
    }

    /// Automatically restores sessions that were force-terminated or crashed
    /// while Operon was running, stamps their cooldown, and shows a brief
    /// Japanese toast naming the session (or the count when more than one)
    /// unless a persistent error banner was raised during the same poll.
    pub(crate) fn restore_forced_sessions(
        &mut self,
        sessions: Vec<(Uuid, String)>,
        now: Instant,
        persistent_notice: &mut bool,
    ) -> HashSet<Uuid> {
        let mut restored_ids = HashSet::new();
        let mut restored_names = Vec::new();
        for (id, name) in sessions {
            let saved_notice = self.notice.take();
            let launched = self.relaunch_lost_session(id);
            if self.notice.is_some() {
                self.brief_notice = None;
                *persistent_notice = true;
            } else {
                self.notice = saved_notice;
            }
            if launched {
                self.last_auto_restore.insert(id, now);
                restored_ids.insert(id);
                restored_names.push(name);
            }
        }
        if !*persistent_notice {
            match restored_names.as_slice() {
                [] => {}
                [name] => self.notice_briefly(tf!(
                    "強制終了されたセッションを自動で復元しました: {name}",
                    name = name
                )),
                many => self.notice_briefly(tf!(
                    "強制終了されたセッションを {count} 件自動で復元しました。",
                    count = many.len()
                )),
            }
        }
        restored_ids
    }

    /// Runs the post-persist side effects of `apply_session_poll`: sidecar
    /// sync, setup banners, clean-exit shell transitions, startup relaunches,
    /// mid-run forced restorations, and ambient notifications.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn finish_session_poll_transitions(
        &mut self,
        changed: bool,
        completed: Vec<String>,
        failed: Vec<(Uuid, String)>,
        finished_setups: Vec<(Uuid, Option<i32>)>,
        reopen_shells: Vec<Uuid>,
        relaunch: Vec<Uuid>,
        forced_restore: Vec<(Uuid, String)>,
        now: Instant,
    ) {
        let mut persistent_notice = false;
        if changed {
            match save_cancellation_intents(&self.data_file, &self.store.pending_cancellations) {
                Ok(WriteOutcome::Durable) => {}
                Ok(WriteOutcome::CommittedButNotSynced(error)) => {
                    self.store_retry_pending = true;
                    let warning = tf!("キャンセル復旧用メタデータは書き込まれましたが、ディスクに保存されたことを確認できませんでした。Operon が自動的に再試行します。エラー: {error}", error = error);
                    self.durability_warning = Some(warning);
                }
                Err(error) => {
                    self.store_retry_pending = true;
                    self.brief_notice = None;
                    self.notice = Some(tf!("セッション状態は保存しましたが、キャンセル復旧用メタデータが保留中です。Operon が自動的に再試行します。エラー: {error}", error = error));
                    persistent_notice = true;
                }
            }
        }
        for (id, status) in finished_setups {
            self.setup_sessions.remove(&id);
            self.brief_notice = None;
            self.notice = Some(match status {
                Some(0) => tr("セットアップが完了しました。").into(),
                Some(code) => tf!(
                    "セットアップが終了コード {p0} で終わりました。セッションを開いて確認してください。",
                    p0 = code
                ),
                None => tr(
                    "セットアップがどう終わったか読み取れませんでした。セッションを開いて確認してください。",
                )
                .into(),
            });
            persistent_notice = true;
        }
        for id in reopen_shells {
            let saved_notice = self.notice.take();
            self.reopen_exited_session_as_shell(id);
            if self.notice.is_some() {
                self.brief_notice = None;
                persistent_notice = true;
            } else {
                self.notice = saved_notice;
            }
        }
        let mut relaunched_ids =
            self.relaunch_remembered_sessions(relaunch, now, &mut persistent_notice);
        relaunched_ids.extend(self.restore_forced_sessions(
            forced_restore,
            now,
            &mut persistent_notice,
        ));
        if changed && self.store.notifications_enabled {
            for name in completed {
                self.send_notification(tf!("セッションが完了しました: {name}", name = name));
            }
            for (id, name) in failed {
                // A crash actually undone by auto-relaunch is not news; one
                // whose relaunch could not proceed (no transcript, busy
                // worktree, broken store) still notifies the person.
                if !relaunched_ids.contains(&id) {
                    self.send_notification(tf!("セッションが失敗しました: {name}", name = name));
                }
            }
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
