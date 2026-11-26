//! The session page's one list (change 285): the column's rows are a
//! project's session tabs, both in one order, and a pinned session stays
//! first and open. The order and the pins live in `session-order.json`.

use crate::*;
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug)]
pub(crate) struct SidebarGroupCount {
    pub(crate) group: SessionStatusGroup,
    pub(crate) count: usize,
    pub(crate) first: Uuid,
}

#[derive(Clone, Debug)]
pub(crate) struct SidebarRowData {
    pub(crate) id: Uuid,
    pub(crate) title: String,
    pub(crate) agent: String,
    pub(crate) kind: SessionStatusKind,
    pub(crate) pinned: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct SidebarProjectData {
    pub(crate) id: Uuid,
    pub(crate) name: String,
    pub(crate) path: PathBuf,
    pub(crate) unfolded: bool,
    pub(crate) group_counts: Vec<SidebarGroupCount>,
    pub(crate) rows: Vec<SidebarRowData>,
}

#[derive(Default, Debug)]
pub(crate) struct SidebarViewData {
    pub(crate) filtering: bool,
    pub(crate) has_any_projects: bool,
    pub(crate) has_matches: bool,
    pub(crate) projects: Vec<SidebarProjectData>,
}

#[derive(Default)]
pub(crate) struct SidebarViewCache {
    fingerprint: Option<u64>,
    data: Arc<SidebarViewData>,
    pub(crate) builds: usize,
    pub(crate) hits: usize,
}

fn hash_workspace_tabs(tabs: &WorkspaceTabs, h: &mut impl Hasher) {
    tabs.focused.hash(h);
    tabs.groups.len().hash(h);
    for g in &tabs.groups {
        g.id.hash(h);
        g.active.hash(h);
        g.tabs.len().hash(h);
        for tab in &g.tabs {
            match tab {
                WorkspaceTab::Session(id) => {
                    0u8.hash(h);
                    id.hash(h);
                }
                WorkspaceTab::Document(doc) => {
                    1u8.hash(h);
                    doc.hash(h);
                }
            }
        }
    }
}

fn hash_session_origin(origin: Option<SessionOrigin>, h: &mut impl Hasher) {
    match origin {
        None => 0u8.hash(h),
        Some(SessionOrigin::Restored { from }) => {
            1u8.hash(h);
            std::mem::discriminant(&from).hash(h);
        }
        Some(SessionOrigin::Resumed { from }) => {
            2u8.hash(h);
            std::mem::discriminant(&from).hash(h);
        }
        Some(SessionOrigin::Adopted) => 3u8.hash(h),
        Some(SessionOrigin::PlainTerminal) => 4u8.hash(h),
    }
}

impl OperonApp {
    fn compute_workspace_sync_fingerprint(&self) -> u64 {
        let mut h = std::hash::DefaultHasher::new();
        self.workspace_project.hash(&mut h);
        self.selected_session.hash(&mut h);
        let mut seen: Vec<Uuid> = self.workspace_seen_sessions.iter().copied().collect();
        seen.sort();
        seen.hash(&mut h);
        self.store.projects.len().hash(&mut h);
        for p in &self.store.projects {
            p.id.hash(&mut h);
        }
        self.store.sessions.len().hash(&mut h);
        for s in &self.store.sessions {
            s.id.hash(&mut h);
            s.project_id.hash(&mut h);
            std::mem::discriminant(&s.status).hash(&mut h);
            s.launched_at.hash(&mut h);
            s.created_at.hash(&mut h);
        }
        self.session_arrangement.projects.hash(&mut h);
        self.session_arrangement.sessions.hash(&mut h);
        self.session_arrangement.pinned.hash(&mut h);
        hash_workspace_tabs(&self.workspace, &mut h);
        self.project_workspaces.len().hash(&mut h);
        let mut pw_keys: Vec<Uuid> = self.project_workspaces.keys().copied().collect();
        pw_keys.sort();
        for key in pw_keys {
            key.hash(&mut h);
            if let Some((tabs, _)) = self.project_workspaces.get(&key) {
                hash_workspace_tabs(tabs, &mut h);
            }
        }
        h.finish()
    }

    fn compute_sidebar_view_fingerprint(&self) -> u64 {
        let mut h = std::hash::DefaultHasher::new();
        std::mem::discriminant(&crate::i18n::active_language()).hash(&mut h);
        std::mem::discriminant(&self.store.language).hash(&mut h);
        self.workspace_project.hash(&mut h);
        self.selected_project.hash(&mut h);
        self.selected_session.hash(&mut h);
        self.status_filter.hash(&mut h);
        self.agent_filter.hash(&mut h);
        self.search.hash(&mut h);
        self.session_arrangement.projects.hash(&mut h);
        self.session_arrangement.sessions.hash(&mut h);
        self.session_arrangement.pinned.hash(&mut h);
        let mut unfolded: Vec<Uuid> = self.session_arrangement.unfolded.to_vec();
        unfolded.sort();
        unfolded.hash(&mut h);
        self.store.projects.len().hash(&mut h);
        for p in &self.store.projects {
            p.id.hash(&mut h);
            p.name.hash(&mut h);
            p.path.hash(&mut h);
        }
        self.store.sessions.len().hash(&mut h);
        for s in &self.store.sessions {
            s.id.hash(&mut h);
            s.project_id.hash(&mut h);
            s.name.hash(&mut h);
            s.goal.hash(&mut h);
            s.agent.hash(&mut h);
            hash_session_origin(s.origin, &mut h);
            std::mem::discriminant(&s.status).hash(&mut h);
            s.worktree_path.hash(&mut h);
            s.created_at.hash(&mut h);
            s.launched_at.hash(&mut h);
            std::mem::discriminant(&self.session_status_kind_for(s)).hash(&mut h);
            self.is_session_unread(s.id).hash(&mut h);
            self.session_prompt_turns
                .get(&s.id)
                .and_then(|turns| turns.first())
                .map(|turn| turn.prompt.as_str())
                .hash(&mut h);
        }
        self.workspace.groups.len().hash(&mut h);
        for g in &self.workspace.groups {
            for tab in &g.tabs {
                if let WorkspaceTab::Session(id) = tab {
                    id.hash(&mut h);
                }
            }
        }
        let mut pw_keys: Vec<Uuid> = self.project_workspaces.keys().copied().collect();
        pw_keys.sort();
        for owner in pw_keys {
            owner.hash(&mut h);
            if let Some((tabs, _)) = self.project_workspaces.get(&owner) {
                for id in tabs.session_ids() {
                    id.hash(&mut h);
                }
            }
        }
        h.finish()
    }

    pub(crate) fn ensure_sidebar_view_cache(&mut self) -> Arc<SidebarViewData> {
        let fp = self.compute_sidebar_view_fingerprint();
        if self.sidebar_view_cache.fingerprint == Some(fp) {
            self.sidebar_view_cache.hits += 1;
            return Arc::clone(&self.sidebar_view_cache.data);
        }
        let mut refs: Vec<&Project> = self.store.projects.iter().collect();
        sort_column_projects(&mut refs, &self.session_arrangement);
        let filtering = self.has_session_filters();
        let mut pinned_set: HashSet<Uuid> =
            self.session_arrangement.pinned.iter().copied().collect();
        let _ = &pinned_set;
        let has_any_projects = !refs.is_empty();
        let mut has_matches = false;
        let mut projects = Vec::with_capacity(refs.len());
        for project in refs {
            let tab_sessions = self.project_tab_sessions(project.id);
            let mut rows = Vec::new();
            let mut group_buckets: [Option<(usize, Uuid)>; 3] = [None; 3];
            for session in tab_sessions {
                if !self.session_matches_filters(session) {
                    continue;
                }
                has_matches = true;
                let kind = self.session_status_kind_for(session);
                let group = if self.is_session_unread(session.id) {
                    Some(SessionStatusGroup::NeedsYou)
                } else {
                    kind.group()
                };
                if let Some(group) = group {
                    let entry = &mut group_buckets[group.index()];
                    match entry {
                        Some((count, _)) => *count += 1,
                        None => *entry = Some((1, session.id)),
                    }
                }
                let title = session_list_title(
                    session,
                    self.session_prompt_turns
                        .get(&session.id)
                        .and_then(|turns| turns.first())
                        .map(|turn| turn.prompt.as_str()),
                );
                rows.push(SidebarRowData {
                    id: session.id,
                    title,
                    agent: session.agent.clone(),
                    kind,
                    pinned: self.is_session_pinned(session.id),
                });
            }
            if filtering && rows.is_empty() {
                continue;
            }
            let mut group_counts = Vec::new();
            for group in [
                SessionStatusGroup::Free,
                SessionStatusGroup::Busy,
                SessionStatusGroup::NeedsYou,
            ] {
                if let Some((count, first)) = group_buckets[group.index()] {
                    group_counts.push(SidebarGroupCount {
                        group,
                        count,
                        first,
                    });
                }
            }
            let unfolded = filtering || self.session_arrangement.unfolded.contains(&project.id);
            projects.push(SidebarProjectData {
                id: project.id,
                name: project.name.clone(),
                path: project.path.clone(),
                unfolded,
                group_counts,
                rows,
            });
        }
        pinned_set.clear();
        let data = Arc::new(SidebarViewData {
            filtering,
            has_any_projects,
            has_matches,
            projects,
        });
        self.sidebar_view_cache.fingerprint = Some(fp);
        self.sidebar_view_cache.data = Arc::clone(&data);
        self.sidebar_view_cache.builds += 1;
        data
    }

    /// Add newly available live sessions once — every project's, each into
    /// its own workspace, so the column, which lists the tabs, shows them
    /// at once (change 285). Preserve every group's active tab, focused file,
    /// and intentional closes across project switches; then put every
    /// workspace's session tabs in the one order the column draws.
    pub(crate) fn sync_workspace_live_sessions(&mut self) {
        let Some(project) = self.workspace_project else {
            return;
        };
        let fp = self.compute_workspace_sync_fingerprint();
        if self.workspace_sync_fingerprint == Some(fp) {
            return;
        }
        let (projects, sessions) = (&self.store.projects, &self.store.sessions);
        let project_ids: HashSet<Uuid> = projects.iter().map(|p| p.id).collect();
        let session_ids: HashSet<Uuid> = sessions.iter().map(|s| s.id).collect();
        self.project_workspaces
            .retain(|id, _| project_ids.contains(id));
        // The shown workspace drops a deleted session's tab after this sync;
        // the others have no frame of their own to do it in.
        for (tabs, _) in self.project_workspaces.values_mut() {
            tabs.retain(|tab| match tab {
                WorkspaceTab::Session(id) => session_ids.contains(id),
                WorkspaceTab::Document(_) => true,
            });
        }
        self.session_arrangement
            .pinned
            .retain(|id| session_ids.contains(id));
        self.workspace_seen_sessions
            .retain(|id| session_ids.contains(id));
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
        let ranks: HashMap<Uuid, _> = sessions
            .iter()
            .map(|s| (s.id, workspace_session_rank(s, arrangement)))
            .collect();
        let rank = |id: Uuid| ranks.get(&id).copied();
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
        self.workspace_sync_generation += 1;
        self.workspace_sync_fingerprint = Some(self.compute_workspace_sync_fingerprint());
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

    /// Closes an unpinned session tab automatically when the session finishes
    /// cleanly or is explicitly stopped, moving it to History without flashing
    /// a refusal banner on pinned tabs or clearing unseen neighbour unread
    /// marks while off-screen (change 291).
    pub(crate) fn close_finished_session_tab(&mut self, id: Uuid) {
        if self.is_session_pinned(id) {
            return;
        }
        let tab = WorkspaceTab::Session(id);
        if self
            .workspace
            .groups
            .iter()
            .any(|group| group.tabs.contains(&tab))
        {
            if self.workspace.close(&tab) {
                self.remember_closed_tab(&tab);
            }
            if self.page == Page::Sessions && !self.session_library_open {
                if let Some(next) = self.workspace.active().cloned() {
                    self.activate_workspace_tab(&next);
                }
                if self.selected_session == Some(id) {
                    self.set_terminal_session_context(None);
                }
            } else {
                match self.workspace.active().cloned() {
                    Some(WorkspaceTab::Session(next_id)) => {
                        self.set_terminal_session_context(Some(next_id));
                    }
                    _ => {
                        if self.selected_session == Some(id) {
                            self.set_terminal_session_context(None);
                        }
                    }
                }
            }
            return;
        }
        let mut closed = false;
        for (tabs, selected) in self.project_workspaces.values_mut() {
            if tabs.close(&tab) {
                closed = true;
                if *selected == Some(id) {
                    *selected = match tabs.active() {
                        Some(WorkspaceTab::Session(next_id)) => Some(*next_id),
                        _ => None,
                    };
                }
            }
        }
        if closed {
            self.remember_closed_tab(&tab);
            if self.selected_session == Some(id) {
                self.set_terminal_session_context(None);
            }
        }
    }

    /// Explicit cancellation clears unread marks before closing the unpinned
    /// tab so a session stopped by the person does not linger under 要対応
    /// (change 291).
    pub(crate) fn clear_cancelled_unread_and_close_tab(&mut self, id: Uuid) {
        self.unread_sessions.remove(&id);
        if self.unread_hold == Some(id) {
            self.unread_hold = None;
        }
        self.close_finished_session_tab(id);
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
            .unfolded
            .retain(|id| projects.iter().any(|project| project.id == *id));
        arrangement
            .pinned
            .retain(|id| sessions.iter().any(|session| session.id == *id));
        self.session_arrangement = arrangement;
        if let Err(error) = save_session_arrangement(&self.data_file, &self.session_arrangement) {
            self.notice = Some(tf!("並び順を保存できませんでした: {error}", error = error));
        }
    }

    /// How many sessions want the person: the ones stopped on a question, the
    /// ones stopped on a prerequisite that will never arrive, and the ones
    /// that said something while nobody was reading them.
    pub(crate) fn attention_count(&self) -> usize {
        self.store
            .sessions
            .iter()
            .filter(|session| self.session_in_status_group(session, SessionStatusGroup::NeedsYou))
            .count()
    }

    /// Which state a session is in, without the words.
    pub(crate) fn session_status_kind_for(&self, session: &Session) -> SessionStatusKind {
        session_status_kind(
            &self.store.sessions,
            session,
            self.cancellation_pending(session.id),
            self.session_activity_for(session.id),
            self.awaiting_first_request.contains(&session.id),
        )
    }

    /// Whether a session belongs in a group, for the purpose of the filter.
    pub(crate) fn session_in_status_group(
        &self,
        session: &Session,
        group: SessionStatusGroup,
    ) -> bool {
        self.kind_in_status_group(self.session_status_kind_for(session), session, group)
    }

    /// The same question asked of a kind already in hand.
    pub(crate) fn kind_in_status_group(
        &self,
        kind: SessionStatusKind,
        session: &Session,
        group: SessionStatusGroup,
    ) -> bool {
        if group == SessionStatusGroup::NeedsYou && self.is_session_unread(session.id) {
            return true;
        }
        if matches!(kind, SessionStatusKind::Failed | SessionStatusKind::Lost) {
            return group == SessionStatusGroup::NeedsYou && self.session_has_a_tab(session.id);
        }
        kind.group() == Some(group)
    }

    /// How many sessions are in each group right now, indexed by
    /// `SessionStatusGroup::index`. One pass, no allocation.
    pub(crate) fn status_group_counts(&self) -> [usize; 3] {
        self.status_group_counts_where(|_| true)
    }

    /// The sidebar's tile counts: what each group would show if it were chosen
    /// alone with the current CLI choice, so a number never promises rows the
    /// column will not draw — and the column draws only tabs (change 285).
    pub(crate) fn status_group_counts_for_chosen_agents(&self) -> [usize; 3] {
        let mut tab_ids: HashSet<Uuid> = self.workspace.session_ids().collect();
        for (tabs, _) in self.project_workspaces.values() {
            tab_ids.extend(tabs.session_ids());
        }
        self.status_group_counts_where(|session| {
            self.session_matches_agent_filter(session) && tab_ids.contains(&session.id)
        })
    }

    fn status_group_counts_where(&self, keep: impl Fn(&Session) -> bool) -> [usize; 3] {
        let mut counts = [0usize; 3];
        for session in self.store.sessions.iter().filter(|session| keep(session)) {
            let kind = self.session_status_kind_for(session);
            for group in SessionStatusGroup::GROUPS {
                if self.kind_in_status_group(kind, session, group) {
                    counts[group.index()] += 1;
                }
            }
        }
        counts
    }

    /// Keeps the 要対応 queue in step with the sessions, once per frame before
    /// anything draws.
    pub(crate) fn track_attention(&mut self, now: Instant) {
        let by_id: HashMap<Uuid, &Session> =
            self.store.sessions.iter().map(|s| (s.id, s)).collect();
        let mut since = std::mem::take(&mut self.attention_since);
        let before = since.len();
        since.retain(|id, _| {
            by_id.get(id).is_some_and(|session| {
                self.session_in_status_group(session, SessionStatusGroup::NeedsYou)
            })
        });
        let mut changed = since.len() != before;
        for session in &self.store.sessions {
            if !since.contains_key(&session.id)
                && self.session_in_status_group(session, SessionStatusGroup::NeedsYou)
            {
                since.insert(session.id, now);
                changed = true;
            }
        }
        self.attention_since = since;
        if changed {
            let mut order: Vec<(Instant, Reverse<u64>, Uuid)> = self
                .attention_since
                .iter()
                .map(|(id, entered)| {
                    let launched = by_id
                        .get(id)
                        .map(|session| session.launched_at.unwrap_or(session.created_at))
                        .unwrap_or_default();
                    (*entered, Reverse(launched), *id)
                })
                .collect();
            order.sort();
            self.attention_order = order.into_iter().map(|(_, _, id)| id).collect();
            self.waiting_reply_drafts
                .retain(|id, _| self.attention_since.contains_key(id));
        }
    }

    /// ⌘J: the queued session after the one on screen, wrapping, opened with
    /// the keyboard in its terminal when there is a live pane to type into.
    pub(crate) fn open_next_waiting_session(&mut self) {
        let Some(next) = next_in_attention_queue(&self.attention_order, self.selected_session)
        else {
            self.notice_briefly(tr("返事を待っているセッションはありません"));
            return;
        };
        self.open_waiting_session(next);
    }

    /// Opens a session someone is waiting on, ⌘J's way or a pinned row's: the
    /// keyboard goes to its pane when there is a live one to type into.
    pub(crate) fn open_waiting_session(&mut self, session_id: Uuid) {
        self.open_session_in_terminal(session_id);
        let running = self
            .store
            .sessions
            .iter()
            .any(|session| session.id == session_id && session.status == SessionStatus::Active);
        self.terminal_focus_request = running.then_some(session_id);
    }

    /// What a pinned row says under its project line: the words that ended the
    /// agent's turn, or that it is waiting on a permission or an answer.
    pub(crate) fn session_attention_line(&self, session_id: Uuid) -> Option<&str> {
        let status = self.fresh_hook_status(session_id)?;
        match status.activity {
            AgentActivity::AwaitingInput => Some(tr("許可または回答を待っています")),
            AgentActivity::Idle => status
                .last_message
                .as_deref()
                .map(str::trim)
                .filter(|words| !words.is_empty()),
            AgentActivity::Working => None,
        }
    }

    /// How long a queued session has wanted a person, or `None` when it does
    /// not.
    pub(crate) fn attention_wait(&self, session_id: Uuid, now: Instant) -> Option<Duration> {
        self.attention_since
            .get(&session_id)
            .map(|entered| now.saturating_duration_since(*entered))
    }

    /// Whether the session list should show this session.
    pub(crate) fn session_matches_status_filter(&self, session: &Session) -> bool {
        if !self.status_filter.iter().any(|selected| *selected) {
            return true;
        }
        let kind = self.session_status_kind_for(session);
        SessionStatusGroup::GROUPS.iter().any(|group| {
            self.status_filter[group.index()] && self.kind_in_status_group(kind, session, *group)
        })
    }

    pub(crate) fn session_matches_filters(&self, session: &Session) -> bool {
        self.session_matches_agent_filter(session) && self.session_matches_status_filter(session)
    }

    pub(crate) fn session_matches_agent_filter(&self, session: &Session) -> bool {
        !self.agent_filter.iter().any(|selected| *selected)
            || self.agent_filter[session_agent_filter_index(&session.agent)]
    }

    pub(crate) fn has_session_filters(&self) -> bool {
        self.status_filter.iter().any(|selected| *selected)
            || self.agent_filter.iter().any(|selected| *selected)
    }

    /// How many sessions the column shows while a filter is on.
    pub(crate) fn matching_session_count(&self) -> usize {
        self.store
            .sessions
            .iter()
            .filter(|session| self.session_matches_filters(session))
            .count()
    }

    /// How many of 要対応's sessions, under the current CLI choice, are
    /// waiting on a permission or an answer.
    pub(crate) fn waiting_session_count(&self) -> usize {
        self.store
            .sessions
            .iter()
            .filter(|session| {
                self.session_matches_agent_filter(session)
                    && self.session_status_kind_for(session) == SessionStatusKind::Waiting
            })
            .count()
    }

    pub(crate) fn toggle_agent_filter(&mut self, index: usize) {
        if let Some(selected) = self.agent_filter.get_mut(index) {
            *selected = !*selected;
            self.forget_pending_session_close();
        }
    }

    pub(crate) fn clear_agent_filter(&mut self) {
        self.agent_filter = [false; 4];
        self.forget_pending_session_close();
    }

    pub(crate) fn clear_session_filters(&mut self) {
        self.clear_agent_filter();
        self.clear_status_filter();
    }

    /// Turn one chip on or off.
    pub(crate) fn toggle_status_filter(&mut self, group: SessionStatusGroup) {
        let index = group.index();
        self.status_filter[index] = !self.status_filter[index];
        self.forget_pending_session_close();
    }

    pub(crate) fn clear_status_filter(&mut self) {
        self.status_filter = [false; 3];
        self.forget_pending_session_close();
    }

    /// The one move the home screen's attention banner offers: the session
    /// list, already narrowed to the sessions it was counting.
    pub(crate) fn show_sessions_needing_you(&mut self) {
        self.show_status_group(SessionStatusGroup::NeedsYou);
    }

    /// The session list narrowed to exactly one group — what a Home tile opens.
    pub(crate) fn show_status_group(&mut self, group: SessionStatusGroup) {
        self.page = Page::Sessions;
        self.session_library_open = true;
        self.session_view = SessionView::List;
        self.search.clear();
        self.pending_history_open = None;
        self.clear_session_filters();
        self.status_filter[group.index()] = true;
        self.forget_pending_session_close();
    }

    pub(crate) fn forget_pending_session_close(&mut self) {
        self.pending_session_close = None;
        self.pending_history_open = None;
    }

    /// The state to show for a session, including the live agent activity the
    /// poller has settled on.
    pub(crate) fn status_view(&self, session: &Session) -> SessionStatusView {
        let mut view = session_status_view(
            &self.store.sessions,
            session,
            self.cancellation_pending(session.id),
            self.session_activity_for(session.id),
            self.awaiting_first_request.contains(&session.id),
        );
        if let Some(tool) = self
            .fresh_hook_status(session.id)
            .filter(|status| status.activity == AgentActivity::Working)
            .and_then(|status| status.tool.as_deref())
        {
            view.hint = tf!("{p0} を実行中", p0 = tool);
        }
        view.hint = self.with_working_time(&view.hint, session.id, Instant::now());
        view
    }

    pub(crate) fn open_session_in_terminal(&mut self, session_id: Uuid) {
        if !self
            .store
            .sessions
            .iter()
            .any(|session| session.id == session_id)
        {
            return;
        }
        self.pending_history_open = None;
        self.terminal_capture_missing.remove(&session_id);
        self.select_session(session_id);
        self.page = Page::Sessions;
        self.session_library_open = false;
        // Fetch immediately instead of making the newly selected terminal wait
        // for the periodic two-second session poll.
        self.refresh_session_output(session_id);
    }

    pub(crate) fn workspace_tab_project(&self, tab: &WorkspaceTab) -> Option<Uuid> {
        match tab {
            WorkspaceTab::Session(id) => self
                .store
                .sessions
                .iter()
                .find(|session| session.id == *id)
                .map(|session| session.project_id),
            WorkspaceTab::Document(id) => Some(id.project),
        }
    }

    /// Swap only view state. Document buffers remain in open_documents, including
    /// unsaved edits; this does not write or change a session's lifetime.
    pub(crate) fn switch_workspace_project(&mut self, project: Uuid) {
        let previous_project = self.workspace_project.or_else(|| {
            self.workspace
                .active()
                .and_then(|tab| self.workspace_tab_project(tab))
        });
        if previous_project != Some(project) {
            let (next, context) = self.project_workspaces.remove(&project).unwrap_or_default();
            let context = context.or_else(|| {
                previous_project
                    .is_none()
                    .then_some(self.selected_session)
                    .flatten()
            });
            let previous = std::mem::replace(&mut self.workspace, next);
            if let Some(previous_project) = previous_project {
                self.project_workspaces
                    .insert(previous_project, (previous, self.selected_session));
            }
            let context = context.filter(|id| {
                self.store
                    .sessions
                    .iter()
                    .any(|session| session.id == *id && session.project_id == project)
            });
            self.set_terminal_session_context(context);
            self.workspace_drag = None;
            self.terminal_focus_request = None;
            self.active_document = match self.workspace.active() {
                Some(WorkspaceTab::Document(id)) => Some(id.clone()),
                _ => None,
            };
        }
        self.workspace_project = Some(project);
        self.selected_project = Some(project);
        self.workspace_initialized = true;
    }

    pub(crate) fn select_workspace_project(&mut self, project: Uuid) {
        self.pending_history_open = None;
        self.session_library_open = false;
        self.switch_workspace_project(project);
        self.sync_workspace_live_sessions();
        if let Some(tab) = self.workspace.active().cloned() {
            self.activate_workspace_tab(&tab);
        } else {
            self.set_terminal_session_context(None);
            self.active_document = None;
        }
    }

    /// The one place terminal context changes, also used when switching to an
    /// empty project. Merely restoring a file must not mark its session read.
    pub(crate) fn set_terminal_session_context(&mut self, session_id: Option<Uuid>) {
        if self
            .unread_hold
            .is_some_and(|held| Some(held) != session_id)
        {
            self.unread_hold = None;
        }
        if self
            .terminal_focus_request
            .is_some_and(|wanted| Some(wanted) != session_id)
        {
            self.terminal_focus_request = None;
        }
        if self.selected_session != session_id {
            if let Some(prev_session) = self.selected_session {
                if !self.resolved_paths.is_empty() {
                    self.terminal_path_caches
                        .insert(prev_session, std::mem::take(&mut self.resolved_paths));
                }
            } else {
                self.resolved_paths.clear();
            }

            if let Some(new_session) = session_id {
                if let Some(cached) = self.terminal_path_caches.get(&new_session) {
                    self.resolved_paths = cached.clone();
                } else {
                    self.resolved_paths = HashMap::new();
                }
            } else {
                self.resolved_paths = HashMap::new();
            }

            self.negative_path_probed_at.clear();
            self.resolved_paths_generation += 1;
            self.unresolved_paths.clear();
            self.resolved_paths_session = session_id;
        }
        self.selected_session = session_id;
    }

    /// Opening a session selects its project's workspace and marks that visible
    /// session read. History may explicitly open a finished record here.
    pub(crate) fn select_session(&mut self, session_id: Uuid) {
        let focus_requested = self.terminal_focus_request == Some(session_id);
        if let Some(project) = self
            .store
            .sessions
            .iter()
            .find(|s| s.id == session_id)
            .map(|s| s.project_id)
        {
            self.switch_workspace_project(project);
        }
        self.set_terminal_session_context(Some(session_id));
        if focus_requested {
            self.terminal_focus_request = Some(session_id);
        }
        self.workspace.open(WorkspaceTab::Session(session_id));
        self.workspace_initialized = true;
        if self.unread_hold != Some(session_id) {
            self.unread_sessions.remove(&session_id);
        }
    }

    /// Marks a session unread by hand, and holds it there until the person
    /// looks at something else.
    pub(crate) fn mark_session_unread(&mut self, session_id: Uuid) {
        self.unread_sessions.insert(session_id);
        self.unread_hold = Some(session_id);
    }

    pub(crate) fn is_session_unread(&self, session_id: Uuid) -> bool {
        self.unread_sessions.contains(&session_id)
    }

    /// Whether a session is the one a person is actually looking at.
    pub(crate) fn is_session_on_screen(&self, session_id: Uuid) -> bool {
        self.page == Page::Sessions
            && !self.session_library_open
            && self.terminals_on_screen().contains(&session_id)
    }

    /// The sessions the workspace draws a terminal for: every terminal of a
    /// split, else the selected one.
    pub(crate) fn terminals_on_screen(&self) -> Vec<Uuid> {
        if !self.workspace.groups.is_empty() || self.workspace_initialized {
            self.workspace
                .visible()
                .into_iter()
                .filter_map(|tab| match tab {
                    WorkspaceTab::Session(id) => Some(id),
                    WorkspaceTab::Document(_) => None,
                })
                .collect()
        } else {
            self.selected_session.into_iter().collect()
        }
    }

    /// Takes a terminal off the screen; its session keeps running.
    pub(crate) fn close_pane(&mut self, session_id: Uuid) {
        self.close_workspace_tab(&WorkspaceTab::Session(session_id));
    }

    pub(crate) fn activate_workspace_tab(&mut self, tab: &WorkspaceTab) {
        match tab {
            WorkspaceTab::Session(id) => self.select_session(*id),
            WorkspaceTab::Document(id) => {
                self.switch_workspace_project(id.project);
                self.workspace.open(WorkspaceTab::Document(id.clone()));
                self.active_document = Some(id.clone());
                self.terminal_focus_request = None;
            }
        }
    }

    pub(crate) fn close_workspace_tab(&mut self, tab: &WorkspaceTab) {
        if let WorkspaceTab::Session(id) = tab {
            if self.is_session_pinned(*id) {
                self.notice_briefly(tr(
                    "ピン留め中のセッションは閉じられません。ピン留めを外してから閉じてください。",
                ));
                return;
            }
        }
        if let WorkspaceTab::Document(id) = tab {
            if self
                .document_index(id)
                .is_some_and(|i| self.open_documents[i].modified())
            {
                self.notice = Some(tr("未保存の変更があります。保存するか、ディスクから読み直してから閉じてください。").into());
                return;
            }
            self.close_document(id);
            self.workspace_editor_views.remove(id);
            self.workspace_editor_jumps.remove(id);
        }
        if self.workspace.close(tab) {
            self.remember_closed_tab(tab);
        }
        if let Some(next) = self.workspace.active().cloned() {
            self.activate_workspace_tab(&next);
        }
    }

    /// Keeps a closed tab for ⇧⌘T, newest last, within `CLOSED_TAB_LIMIT`.
    pub(crate) fn remember_closed_tab(&mut self, tab: &WorkspaceTab) {
        self.closed_tabs.retain(|closed| closed != tab);
        self.closed_tabs.push(tab.clone());
        let excess = self.closed_tabs.len().saturating_sub(CLOSED_TAB_LIMIT);
        self.closed_tabs.drain(..excess);
    }

    /// ⇧⌘T: the newest closed tab that can still be shown, shown again.
    pub(crate) fn reopen_closed_tab(&mut self) {
        while let Some(tab) = self.closed_tabs.pop() {
            if self
                .workspace
                .groups
                .iter()
                .any(|group| group.tabs.contains(&tab))
            {
                continue;
            }
            match &tab {
                WorkspaceTab::Session(id) => {
                    if !self.store.sessions.iter().any(|session| session.id == *id) {
                        continue;
                    }
                    self.page = Page::Sessions;
                    self.session_library_open = false;
                    self.select_session(*id);
                }
                WorkspaceTab::Document(id) => {
                    let Some(project) = self
                        .store
                        .projects
                        .iter()
                        .find(|project| project.id == id.project)
                        .cloned()
                    else {
                        continue;
                    };
                    self.page = Page::Sessions;
                    self.session_library_open = false;
                    self.open_document_at(&project, id.root.clone(), id.path.clone());
                }
            }
            return;
        }
        self.notice_briefly(tr("開き直せるタブはありません。"));
    }

    /// ⇧⌘] / ⇧⌘[ and ⌃Tab / ⌃⇧Tab: the focused pane's next or previous tab,
    /// wrapping round.
    pub(crate) fn step_workspace_tab(&mut self, forward: bool) {
        let Some(group) = self
            .workspace
            .groups
            .iter()
            .find(|group| Some(group.id) == self.workspace.focused)
        else {
            return;
        };
        let count = group.tabs.len();
        if count == 0 {
            return;
        }
        let index = if forward {
            (group.active + 1) % count
        } else {
            (group.active + count - 1) % count
        };
        let id = group.id;
        let tab = group.tabs[index].clone();
        self.workspace.select(id, &tab);
        self.activate_workspace_tab(&tab);
    }

    /// Both paths that decide a session changed state call this, at the point
    /// each already decided the change was worth a notification. A change a
    /// person watched happen is not unread.
    pub(crate) fn note_session_activity_change(&mut self, session_id: Uuid) {
        if !self.is_session_on_screen(session_id) {
            self.unread_sessions.insert(session_id);
        }
    }

    /// The sidebar's filters (change 252, updated in change 291).
    pub(crate) fn ui_sidebar_session_filters(&mut self, ui: &mut egui::Ui) {
        let palette = self.store.theme.palette();
        let counts = self.status_group_counts_for_chosen_agents();
        let gap = SPACE_XS;
        let available = ui.available_width();
        let columns = if available >= 3.0 * STATUS_TILE_MIN_W + 2.0 * gap {
            3
        } else if available >= 2.0 * STATUS_TILE_MIN_W + gap {
            2
        } else {
            1
        };
        let mut toggled = None;
        ui.scope(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(gap, gap);
            for row in SessionStatusGroup::GROUPS.chunks(columns) {
                let row_width = (available - gap * (row.len() - 1) as f32) / row.len() as f32;
                ui.horizontal(|ui| {
                    for &group in row {
                        let tile = status_filter_tile(
                            ui,
                            palette,
                            group,
                            counts[group.index()],
                            self.status_filter[group.index()],
                            row_width,
                        )
                        .on_hover_ui(|ui| {
                            ui.label(tr(group.hint_id()));
                            if group == SessionStatusGroup::NeedsYou {
                                ui.label(tf!(
                                    "承認・回答待ち {count} 件",
                                    count = self.waiting_session_count()
                                ));
                            }
                        });
                        if tile.clicked() {
                            toggled = Some(group);
                        }
                    }
                });
            }
        });
        if let Some(group) = toggled {
            self.toggle_status_filter(group);
        }
        let filtering = self.has_session_filters();
        let mut summarised = false;
        ui.horizontal(|ui| {
            self.ui_sidebar_agent_segment(ui, palette);
            if filtering && ui.available_width() >= STATUS_TILE_MIN_W {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    self.ui_sidebar_filter_summary(ui, palette);
                });
                summarised = true;
            }
        });
        if filtering && !summarised {
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    self.ui_sidebar_filter_summary(ui, palette);
                });
            });
        }
        hairline(ui, palette);
    }

    /// Codex, Claude Code, Antigravity and every other CLI, as marks in one
    /// hairline segment; the marks not chosen fade while any is.
    pub(crate) fn ui_sidebar_agent_segment(&mut self, ui: &mut egui::Ui, palette: &Palette) {
        let narrowed = self.agent_filter.iter().any(|selected| *selected);
        let mut toggled = None;
        egui::Frame::NONE
            .stroke(egui::Stroke::new(1.0_f32, palette.border_subtle))
            .corner_radius(egui::CornerRadius::same(RADIUS_CONTROL))
            .inner_margin(egui::Margin::same(1))
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(2.0, 0.0);
                for agent in ["codex", "claude", "gemini", "other"] {
                    let index = session_agent_filter_index(agent);
                    let (label, mark) = if index == 3 {
                        (tr("その他"), None)
                    } else {
                        (agent_choice_copy(agent).0, Some(agent))
                    };
                    let selected = self.agent_filter[index];
                    if agent_filter_toggle(ui, palette, label, mark, ICON_MORE, selected, narrowed)
                        .clicked()
                    {
                        toggled = Some(index);
                    }
                }
            });
        if let Some(index) = toggled {
            self.toggle_agent_filter(index);
        }
    }

    /// Right to left: 「クリア」, then how many sessions the column shows.
    pub(crate) fn ui_sidebar_filter_summary(&mut self, ui: &mut egui::Ui, palette: &Palette) {
        if quiet_button(ui, palette, tr("クリア")).clicked() {
            self.clear_session_filters();
        }
        ui.label(
            RichText::new(tf!("{count} 件", count = self.matching_session_count()))
                .size(11.5)
                .color(palette.text_muted),
        );
    }

    /// One level of a move inside the session column, once its targets are
    /// drawn.
    pub(crate) fn ui_column_drop(
        &mut self,
        ui: &egui::Ui,
        drag: ColumnDrag,
        targets: &ColumnTargets,
        slot: Option<usize>,
        span: egui::Rangef,
        gap: f32,
    ) {
        let Some(slot) = slot else {
            return;
        };
        if let Some(y) = slot_line_y(&targets.rects, slot, gap) {
            let palette = self.store.theme.palette();
            ui.painter()
                .hline(span, y, egui::Stroke::new(2.0_f32, palette.accent));
        }
        if ui.input(|i| i.pointer.primary_released()) {
            self.finish_column_drag(drag, &targets.ids, slot);
        }
    }

    /// Ends a move inside the session column.
    pub(crate) fn finish_column_drag(&mut self, drag: ColumnDrag, visible: &[Uuid], slot: usize) {
        let mut arrangement = self.session_arrangement.clone();
        match drag {
            ColumnDrag::Project(moved) => {
                let mut projects = self.store.projects.iter().collect::<Vec<_>>();
                sort_column_projects(&mut projects, &arrangement);
                let mut order = projects
                    .iter()
                    .map(|project| project.id)
                    .collect::<Vec<_>>();
                if !place(&mut order, moved, visible, slot) {
                    return;
                }
                arrangement.projects = order;
            }
            ColumnDrag::Session { project, session } => {
                let mut sessions = self
                    .store
                    .sessions
                    .iter()
                    .filter(|candidate| candidate.project_id == project)
                    .collect::<Vec<_>>();
                sort_column_sessions(&mut sessions, &arrangement);
                let mut order = sessions
                    .iter()
                    .map(|candidate| candidate.id)
                    .collect::<Vec<_>>();
                if !place(&mut order, session, visible, slot) {
                    return;
                }
                arrangement.sessions.retain(|id| !order.contains(id));
                arrangement.sessions.extend(order);
            }
        }
        self.commit_session_arrangement(arrangement);
    }
}
