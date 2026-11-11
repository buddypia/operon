//! The session page's one list (change 285): the column's rows are a
//! project's session tabs, both in one order, and a pinned session stays
//! first and open. The order and the pins live in `session-order.json`.

use crate::*;

impl OperonApp {
    /// Add newly available live sessions once — every project's, each into
    /// its own workspace, so the column, which lists the tabs, shows them
    /// at once (change 285). Preserve every group's active tab, focused file,
    /// and intentional closes across project switches; then put every
    /// workspace's session tabs in the one order the column draws.
    pub(crate) fn sync_workspace_live_sessions(&mut self) {
        let Some(project) = self.workspace_project else {
            return;
        };
        let (projects, sessions) = (&self.store.projects, &self.store.sessions);
        self.project_workspaces
            .retain(|id, _| projects.iter().any(|p| p.id == *id));
        // The shown workspace drops a deleted session's tab after this sync;
        // the others have no frame of their own to do it in.
        for (tabs, _) in self.project_workspaces.values_mut() {
            tabs.retain(|tab| match tab {
                WorkspaceTab::Session(id) => sessions.iter().any(|s| s.id == *id),
                WorkspaceTab::Document(_) => true,
            });
        }
        self.session_arrangement
            .pinned
            .retain(|id| sessions.iter().any(|s| s.id == *id));
        self.workspace_seen_sessions
            .retain(|id| sessions.iter().any(|s| s.id == *id));
        for id in self.workspace.session_ids() {
            self.workspace_seen_sessions.insert(id);
        }
        for (tabs, _) in self.project_workspaces.values() {
            for id in tabs.session_ids() {
                self.workspace_seen_sessions.insert(id);
            }
        }
        let focus = self.workspace.focused.zip(self.workspace.active().cloned());
        let mut live = sessions
            .iter()
            .filter(|session| {
                matches!(
                    session.status,
                    SessionStatus::Queued
                        | SessionStatus::Starting
                        | SessionStatus::Active
                        | SessionStatus::Unknown
                )
            })
            .collect::<Vec<_>>();
        sort_workspace_sessions(&mut live, &self.session_arrangement);
        let preferred = self
            .selected_session
            .filter(|id| live.iter().any(|s| s.id == *id && s.project_id == project))
            .or_else(|| {
                live.iter()
                    .find(|s| s.project_id == project)
                    .map(|session| session.id)
            });
        let new = live
            .iter()
            .filter_map(|session| {
                self.workspace_seen_sessions
                    .insert(session.id)
                    .then_some((session.project_id, session.id))
            })
            .collect::<Vec<_>>();
        for (owner, id) in new {
            if owner == project {
                self.workspace.open(WorkspaceTab::Session(id));
            } else {
                // A project not shown keeps its front tab; the new one waits
                // beside it.
                let (tabs, _) = self.project_workspaces.entry(owner).or_default();
                let front = tabs.focused.zip(tabs.active().cloned());
                tabs.open(WorkspaceTab::Session(id));
                if let Some((group, tab)) = front {
                    tabs.select(group, &tab);
                }
            }
        }
        let arrangement = &self.session_arrangement;
        let rank = |id: Uuid| {
            sessions
                .iter()
                .find(|s| s.id == id)
                .map(|s| workspace_session_rank(s, arrangement))
        };
        self.workspace.arrange(rank);
        for (tabs, _) in self.project_workspaces.values_mut() {
            tabs.arrange(rank);
        }
        if let Some((group, tab)) = focus {
            self.workspace.select(group, &tab);
        } else if let Some(id) = preferred {
            // Select only a present tab: a closed live session is not reopened.
            if let Some(group) = self
                .workspace
                .groups
                .iter()
                .find(|g| g.tabs.contains(&WorkspaceTab::Session(id)))
                .map(|g| g.id)
            {
                self.workspace.select(group, &WorkspaceTab::Session(id));
            }
        }
    }

    /// 「閉じる」 on a row or a tab (change 286): closes the session's tab
    /// wherever it is. A tab of a project not shown closes in that project's
    /// workspace, and the shown project stays; a pinned one stays open.
    pub(crate) fn close_session_tab(&mut self, id: Uuid) {
        let tab = WorkspaceTab::Session(id);
        if self
            .workspace
            .groups
            .iter()
            .any(|group| group.tabs.contains(&tab))
        {
            self.close_workspace_tab(&tab);
            return;
        }
        if self.is_session_pinned(id) {
            return;
        }
        let mut closed = false;
        for (tabs, _) in self.project_workspaces.values_mut() {
            closed |= tabs.close(&tab);
        }
        if closed {
            self.remember_closed_tab(&tab);
        }
    }

    /// The session page column's rows for `project` (change 285): exactly
    /// its session tabs, every group's, in the order the tabs are put in.
    pub(crate) fn project_tab_sessions(&self, project: Uuid) -> Vec<&Session> {
        let tabs = if self.workspace_project == Some(project) {
            Some(&self.workspace)
        } else {
            self.project_workspaces.get(&project).map(|(tabs, _)| tabs)
        };
        let Some(tabs) = tabs else {
            return Vec::new();
        };
        let mut rows = tabs
            .session_ids()
            .filter_map(|id| {
                self.store
                    .sessions
                    .iter()
                    .find(|s| s.id == id && s.project_id == project)
            })
            .collect::<Vec<_>>();
        sort_workspace_sessions(&mut rows, &self.session_arrangement);
        rows
    }

    pub(crate) fn is_session_pinned(&self, session_id: Uuid) -> bool {
        self.session_arrangement.pinned.contains(&session_id)
    }

    /// Pins or unpins a session from its row or its tab, and saves it with
    /// the column's order. A failed write leaves it in effect for this run
    /// and says so.
    pub(crate) fn toggle_session_pin(&mut self, session_id: Uuid) {
        let mut arrangement = self.session_arrangement.clone();
        if let Some(at) = arrangement.pinned.iter().position(|id| *id == session_id) {
            arrangement.pinned.remove(at);
        } else {
            arrangement.pinned.push(session_id);
        }
        self.commit_session_arrangement(arrangement);
    }

    /// After a tab was dropped among `group`'s tabs: the order that group now
    /// shows becomes the column's order, so the rows follow (change 285).
    /// The whole level is written, as a row drag writes it.
    pub(crate) fn follow_tab_drop(&mut self, group: Uuid) {
        let Some(project) = self.workspace_project else {
            return;
        };
        let Some(sequence) = self
            .workspace
            .groups
            .iter()
            .find(|candidate| candidate.id == group)
            .map(|group| {
                group
                    .tabs
                    .iter()
                    .filter_map(|tab| match tab {
                        WorkspaceTab::Session(id) => Some(*id),
                        WorkspaceTab::Document(_) => None,
                    })
                    .collect::<Vec<_>>()
            })
        else {
            return;
        };
        let mut level = self
            .store
            .sessions
            .iter()
            .filter(|session| session.project_id == project)
            .collect::<Vec<_>>();
        sort_workspace_sessions(&mut level, &self.session_arrangement);
        let mut order = level.iter().map(|session| session.id).collect::<Vec<_>>();
        follow(&mut order, &sequence);
        let mut arrangement = self.session_arrangement.clone();
        arrangement.sessions.retain(|id| !order.contains(id));
        arrangement.sessions.extend(order);
        self.commit_session_arrangement(arrangement);
    }

    /// Whether any project's workspace, shown or not, holds a tab for it.
    pub(crate) fn session_has_a_tab(&self, session_id: Uuid) -> bool {
        self.workspace.session_ids().any(|id| id == session_id)
            || self
                .project_workspaces
                .values()
                .any(|(tabs, _)| tabs.session_ids().any(|id| id == session_id))
    }

    /// Keeps `arrangement` and writes it: only what the store still holds,
    /// so the file is bounded by it. A failed write leaves it in effect for
    /// this run and says so.
    pub(crate) fn commit_session_arrangement(&mut self, mut arrangement: SessionArrangement) {
        let (projects, sessions) = (&self.store.projects, &self.store.sessions);
        arrangement
            .projects
            .retain(|id| projects.iter().any(|project| project.id == *id));
        arrangement
            .sessions
            .retain(|id| sessions.iter().any(|session| session.id == *id));
        arrangement
            .folded
            .retain(|id| projects.iter().any(|project| project.id == *id));
        arrangement
            .pinned
            .retain(|id| sessions.iter().any(|session| session.id == *id));
        self.session_arrangement = arrangement;
        if let Err(error) = save_session_arrangement(&self.data_file, &self.session_arrangement) {
            self.notice = Some(tf!("並び順を保存できませんでした: {error}", error = error));
        }
    }
}
