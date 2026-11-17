//! The snapshot `Operon agents` reads (change 309): every live session and
//! the state the window shows for it, written beside the store whenever that
//! changes, and its time refreshed on a heartbeat by the writer thread.

use crate::*;
use std::sync::mpsc;

/// What the writer thread is asked to do, in the order it was asked.
pub(crate) enum AgentSnapshotWrite {
    Write(AgentSnapshot),
    Remove,
}

/// One thread owns the file, so a removal at quit cannot be overtaken by a
/// write that was still in flight.
pub(crate) struct AgentSnapshotWriter {
    sender: mpsc::Sender<AgentSnapshotWrite>,
    handle: thread::JoinHandle<()>,
}

impl AgentSnapshotWriter {
    pub(crate) fn start(path: PathBuf) -> Self {
        let (sender, receiver) = mpsc::channel();
        let handle = thread::spawn(move || {
            apply_agent_snapshot_writes(
                &path,
                receiver,
                Duration::from_secs(AGENTS_SNAPSHOT_HEARTBEAT_SECONDS),
                &mut now,
            );
        });
        Self { sender, handle }
    }

    pub(crate) fn write(&self, snapshot: AgentSnapshot) {
        let _ = self.sender.send(AgentSnapshotWrite::Write(snapshot));
    }

    /// Removes the file after every earlier write and waits for that to be
    /// done.
    pub(crate) fn finish(self) {
        let _ = self.sender.send(AgentSnapshotWrite::Remove);
        drop(self.sender);
        let _ = self.handle.join();
    }
}

/// The writer thread's whole life: each request in turn, until the sender is
/// gone. With nothing asked for `heartbeat`, the last snapshot is written again
/// under a new time, so its age says Operon is still running even while the
/// window is not drawing. A removal forgets it, so nothing brings the file
/// back. A failed write is left for the next one; the file is derived.
pub(crate) fn apply_agent_snapshot_writes(
    path: &Path,
    receiver: mpsc::Receiver<AgentSnapshotWrite>,
    heartbeat: Duration,
    clock: &mut dyn FnMut() -> u64,
) {
    let mut last: Option<AgentSnapshot> = None;
    loop {
        match receiver.recv_timeout(heartbeat) {
            Ok(AgentSnapshotWrite::Write(snapshot)) => {
                if let Ok(bytes) = serde_json::to_vec(&snapshot) {
                    let _ = write_file_atomically(path, &bytes);
                }
                last = Some(snapshot);
            }
            Ok(AgentSnapshotWrite::Remove) => {
                last = None;
                let _ = fs::remove_file(path);
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if let Some(snapshot) = last.as_mut() {
                    snapshot.written_at = clock();
                    if let Ok(bytes) = serde_json::to_vec(snapshot) {
                        let _ = write_file_atomically(path, &bytes);
                    }
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => return,
        }
    }
}

impl OperonApp {
    /// Every session the workspace still holds, with the state the window
    /// shows for it.
    pub(crate) fn agent_snapshot(&self, written_at: u64) -> AgentSnapshot {
        let projects = self
            .store
            .projects
            .iter()
            .map(|project| AgentProject {
                id: project.id,
                name: project.name.clone(),
                path: project.path.clone(),
            })
            .collect();
        let sessions = self
            .store
            .sessions
            .iter()
            .filter_map(|session| {
                let state = agent_state_token(self.session_status_kind_for(session))?;
                let folder = session.worktree_path.clone().or_else(|| {
                    self.store
                        .projects
                        .iter()
                        .find(|project| project.id == session.project_id)
                        .map(|project| project.path.clone())
                })?;
                Some(AgentEntry {
                    id: session.id,
                    project_id: session.project_id,
                    name: session_list_title(
                        session,
                        self.session_prompt_turns
                            .get(&session.id)
                            .and_then(|turns| turns.first())
                            .map(|turn| turn.prompt.as_str()),
                    ),
                    agent: session.agent.clone(),
                    branch: session.branch.clone(),
                    folder,
                    tmux_name: session.tmux_name.clone(),
                    state: state.to_owned(),
                })
            })
            .collect();
        AgentSnapshot {
            written_at,
            projects,
            sessions,
        }
    }

    /// From `logic`: at most once a second, hand a changed snapshot to the
    /// writer thread. Nothing here touches the disk; the heartbeat is the
    /// writer's.
    pub(crate) fn track_agent_snapshot(&mut self, now: Instant) {
        if self.agent_snapshot_checked.is_some_and(|at| {
            now.duration_since(at) < Duration::from_secs(AGENTS_SNAPSHOT_CHECK_SECONDS)
        }) {
            return;
        }
        self.agent_snapshot_checked = Some(now);
        let mut snapshot = self.agent_snapshot(0);
        let Ok(content) = serde_json::to_string(&snapshot) else {
            return;
        };
        if self.agent_snapshot_written.as_deref() == Some(content.as_str()) {
            return;
        }
        snapshot.written_at = crate::now();
        self.agent_snapshot_writer
            .get_or_insert_with(|| {
                AgentSnapshotWriter::start(agents_snapshot_path(&self.data_file))
            })
            .write(snapshot);
        self.agent_snapshot_written = Some(content);
    }

    /// From `on_exit`: the file goes with the window.
    pub(crate) fn finish_agent_snapshot(&mut self) {
        if let Some(writer) = self.agent_snapshot_writer.take() {
            writer.finish();
        }
    }
}
