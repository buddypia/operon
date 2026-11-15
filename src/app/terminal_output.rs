//! A terminal's captured text, incremental ANSI layout, and event-driven
//! `tmux -C` output stream manager (changes 276, 292).

use crate::*;
use std::hash::{Hash, Hasher};
#[cfg(test)]
use std::sync::atomic::AtomicUsize;
use std::sync::{atomic::AtomicU8, Condvar, Mutex};

/// Rows a capture thread laid out for the text it read, or its word that the
/// text is the one the app already holds.
#[derive(Debug)]
pub(crate) struct LaidOutTerminal {
    /// The theme whose ANSI table coloured `layouts`.
    pub(crate) theme: AppTheme,
    pub(crate) fingerprint: u64,
    /// `None` when the text's fingerprint was the known one. Only a hint:
    /// `set_session_output` still compares the text itself.
    pub(crate) layouts: Option<Vec<LayoutJob>>,
}

#[derive(Clone, Debug)]
pub(crate) struct PrelaidIncremental {
    pub(crate) theme: AppTheme,
    pub(crate) incremental: Arc<IncrementalTerminalLayout>,
    pub(crate) line_ids: Vec<u64>,
    pub(crate) dirty_lines: Option<Vec<usize>>,
}

pub(crate) fn terminal_output_fingerprint(output: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    output.hash(&mut hasher);
    hasher.finish()
}

/// Lays `output` out unless its fingerprint is `known`. Runs off the frame.
pub(crate) fn lay_out_capture(
    output: &str,
    theme: AppTheme,
    known: Option<u64>,
) -> LaidOutTerminal {
    let fingerprint = terminal_output_fingerprint(output);
    LaidOutTerminal {
        theme,
        fingerprint,
        layouts: (known != Some(fingerprint))
            .then(|| terminal_line_layouts(output, theme.palette())),
    }
}

/// One tmux capture with its rows, for a background thread.
pub(crate) fn capture_laid_out(
    tmux_name: &str,
    theme: AppTheme,
    known: Option<u64>,
) -> (UiResult<TerminalCapture>, Option<LaidOutTerminal>) {
    match tmux_capture_with_cursor(tmux_name) {
        Ok(capture) => {
            let laid_out = lay_out_capture(&capture.output, theme, known);
            (Ok(capture), Some(laid_out))
        }
        Err(error) => (Err(error.to_string()), None),
    }
}

/// Pure frame-budget coalescing delay for `tmux -C` output notifications.
pub(crate) fn stream_coalesce_delay(
    last_capture_at: Option<Instant>,
    now: Instant,
    budget: Duration,
) -> Duration {
    match last_capture_at {
        None => Duration::ZERO,
        Some(last) => budget.saturating_sub(now.saturating_duration_since(last)),
    }
}

/// How long until the pane should next read tmux for an echo: zero when it is
/// due, `None` when the last keystroke is `TERMINAL_ECHO_WINDOW` old.
pub(crate) fn terminal_echo_poll_wait(
    last_input: Option<Instant>,
    last_poll: Option<Instant>,
    now: Instant,
) -> Option<Duration> {
    if now.saturating_duration_since(last_input?) >= TERMINAL_ECHO_WINDOW {
        return None;
    }
    Some(last_poll.map_or(Duration::ZERO, |polled| {
        TERMINAL_ECHO_POLL.saturating_sub(now.saturating_duration_since(polled))
    }))
}

#[derive(Clone, Debug)]
pub(crate) enum StreamedTerminalUpdate {
    CursorOnly {
        session_id: Uuid,
        theme: AppTheme,
        fingerprint: u64,
        cursor: Option<TerminalCursor>,
    },
    Full {
        session_id: Uuid,
        capture: TerminalCapture,
        theme: AppTheme,
        fingerprint: u64,
        incremental: Arc<IncrementalTerminalLayout>,
        jobs: Vec<LayoutJob>,
        line_ids: Vec<u64>,
        dirty_lines: Option<Vec<usize>>,
    },
}

#[derive(Default)]
struct WatcherSignal {
    dirty: bool,
    exited: bool,
    stop: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct LastDeliveredSnapshot {
    pub(crate) theme: AppTheme,
    pub(crate) fingerprint: u64,
    pub(crate) output: String,
    pub(crate) cursor: Option<TerminalCursor>,
}

/// Classifies a newly captured terminal snapshot against `last_delivered`:
/// returns `None` when `(theme, fingerprint, output, cursor)` is unchanged (zero
/// repaint, zero channel message), `Some(CursorOnly)` when only `cursor` moved
/// under the same theme and text (zero ANSI re-parse), or `Some(Full)` after
/// incremental line-level ANSI layout when text or theme changed.
pub(crate) fn classify_captured_snapshot(
    session_id: Uuid,
    capture: TerminalCapture,
    theme: AppTheme,
    worker_inc: &mut Option<Arc<IncrementalTerminalLayout>>,
    last_delivered: &mut Option<LastDeliveredSnapshot>,
) -> Option<StreamedTerminalUpdate> {
    let fingerprint = terminal_output_fingerprint(&capture.output);
    let text_unchanged = last_delivered.as_ref().is_some_and(|prev| {
        prev.theme == theme && prev.fingerprint == fingerprint && prev.output == capture.output
    });
    if text_unchanged {
        let prev = last_delivered.as_mut().expect("checked Some");
        if prev.cursor == capture.cursor {
            return None;
        }
        prev.cursor = capture.cursor;
        return Some(StreamedTerminalUpdate::CursorOnly {
            session_id,
            theme,
            fingerprint,
            cursor: capture.cursor,
        });
    }
    let (mut inc, jobs, line_ids) =
        terminal_line_layouts_incremental(&capture.output, theme, worker_inc.as_deref());
    let dirty_lines = inc.take_dirty_lines();
    let inc_arc = Arc::new(inc);
    *worker_inc = Some(Arc::clone(&inc_arc));
    *last_delivered = Some(LastDeliveredSnapshot {
        theme,
        fingerprint,
        output: capture.output.clone(),
        cursor: capture.cursor,
    });
    Some(StreamedTerminalUpdate::Full {
        session_id,
        capture,
        theme,
        fingerprint,
        incremental: inc_arc,
        jobs,
        line_ids,
        dirty_lines: Some(dirty_lines),
    })
}

struct StreamSeed {
    theme: AppTheme,
    output: Option<String>,
    cursor: Option<TerminalCursor>,
    incremental: Option<Arc<IncrementalTerminalLayout>>,
}

struct SessionStreamHandle {
    tmux_name: String,
    signal: Arc<(Mutex<WatcherSignal>, Condvar)>,
    child: Arc<Mutex<StreamingChild>>,
    live: Arc<AtomicBool>,
    done: Arc<AtomicBool>,
    reader_join: Option<thread::JoinHandle<()>>,
    coalescer_join: Option<thread::JoinHandle<()>>,
}

impl SessionStreamHandle {
    fn stop(&mut self, wait_coalescer: bool) {
        self.live.store(false, Ordering::Release);
        self.done.store(true, Ordering::Release);
        if let Ok(mut state) = self.signal.0.lock() {
            state.stop = true;
            self.signal.1.notify_all();
        }
        if let Ok(mut child) = self.child.lock() {
            child.shutdown();
        }
        if let Some(join) = self.reader_join.take() {
            let _ = join.join();
        }
        if wait_coalescer {
            if let Some(join) = self.coalescer_join.take() {
                let _ = join.join();
            }
        }
    }
}

pub(crate) struct TerminalStreamManager {
    entries: HashMap<Uuid, SessionStreamHandle>,
    cooldowns: HashMap<(Uuid, String), Instant>,
    sender: Sender<StreamedTerminalUpdate>,
    receiver: Receiver<StreamedTerminalUpdate>,
    focused: Arc<AtomicBool>,
    theme_code: Arc<AtomicU8>,
    repaint: Arc<Mutex<Option<egui::Context>>>,
    #[cfg(test)]
    total_spawns: usize,
    #[cfg(test)]
    total_captures: Arc<AtomicUsize>,
    #[cfg(test)]
    total_deliveries: Arc<AtomicUsize>,
}

impl Default for TerminalStreamManager {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            entries: HashMap::new(),
            cooldowns: HashMap::new(),
            sender,
            receiver,
            focused: Arc::new(AtomicBool::new(true)),
            theme_code: Arc::new(AtomicU8::new(app_theme_code(AppTheme::default()))),
            repaint: Arc::new(Mutex::new(None)),
            #[cfg(test)]
            total_spawns: 0,
            #[cfg(test)]
            total_captures: Arc::new(AtomicUsize::new(0)),
            #[cfg(test)]
            total_deliveries: Arc::new(AtomicUsize::new(0)),
        }
    }
}

impl Drop for TerminalStreamManager {
    fn drop(&mut self) {
        self.shutdown_all();
    }
}

impl TerminalStreamManager {
    pub(crate) fn is_watching(&self, session_id: Uuid) -> bool {
        self.entries.get(&session_id).is_some_and(|entry| {
            entry.live.load(Ordering::Acquire) && !entry.done.load(Ordering::Acquire)
        })
    }

    pub(crate) fn notify_theme_changed(&self, theme: AppTheme) {
        self.theme_code
            .store(app_theme_code(theme), Ordering::Release);
        for entry in self.entries.values() {
            if let Ok(mut state) = entry.signal.0.lock() {
                state.dirty = true;
                entry.signal.1.notify_one();
            }
        }
    }

    pub(crate) fn shutdown_all(&mut self) {
        for (_, mut entry) in self.entries.drain() {
            entry.stop(true);
        }
    }

    #[cfg(test)]
    pub(crate) fn active_watcher_count(&self) -> usize {
        self.entries
            .values()
            .filter(|entry| {
                entry.live.load(Ordering::Acquire) && !entry.done.load(Ordering::Acquire)
            })
            .count()
    }

    #[cfg(test)]
    pub(crate) fn has_active_entry(&self, session_id: Uuid) -> bool {
        self.entries
            .get(&session_id)
            .is_some_and(|entry| !entry.done.load(Ordering::Acquire))
    }

    #[cfg(test)]
    pub(crate) fn total_spawns(&self) -> usize {
        self.total_spawns
    }

    #[cfg(test)]
    pub(crate) fn total_captures(&self) -> usize {
        self.total_captures.load(Ordering::Acquire)
    }

    #[cfg(test)]
    pub(crate) fn total_deliveries(&self) -> usize {
        self.total_deliveries.load(Ordering::Acquire)
    }

    #[cfg(test)]
    pub(crate) fn insert_live_watcher_for_test(
        &mut self,
        session_id: Uuid,
        tmux_name: String,
        streaming: StreamingChild,
    ) {
        let signal = Arc::new((Mutex::new(WatcherSignal::default()), Condvar::new()));
        let child = Arc::new(Mutex::new(streaming));
        let live = Arc::new(AtomicBool::new(true));
        let done = Arc::new(AtomicBool::new(false));
        self.entries.insert(
            session_id,
            SessionStreamHandle {
                tmux_name,
                signal,
                child,
                live,
                done,
                reader_join: None,
                coalescer_join: None,
            },
        );
    }

    #[cfg(test)]
    pub(crate) fn send_stream_update_for_test(&self, update: StreamedTerminalUpdate) {
        let _ = self.sender.send(update);
    }

    fn spawn_watcher(
        &mut self,
        session_id: Uuid,
        tmux_name: String,
        seed: StreamSeed,
        visible_sessions: Arc<std::sync::RwLock<HashSet<Uuid>>>,
        now: Instant,
    ) {
        #[cfg(test)]
        {
            self.total_spawns += 1;
        }
        let mut streaming = match tmux_spawn_output_watcher(&tmux_name) {
            Ok(streaming) => streaming,
            Err(_) => {
                self.cooldowns.insert((session_id, tmux_name), now);
                return;
            }
        };
        let Some(mut stdout) = streaming.take_stdout() else {
            streaming.shutdown();
            self.cooldowns.insert((session_id, tmux_name), now);
            return;
        };

        let signal = Arc::new((Mutex::new(WatcherSignal::default()), Condvar::new()));
        let child = Arc::new(Mutex::new(streaming));
        let live = Arc::new(AtomicBool::new(false));
        let done = Arc::new(AtomicBool::new(false));

        let reader_signal = Arc::clone(&signal);
        let reader_child = Arc::clone(&child);
        let reader_live = Arc::clone(&live);
        let reader_done = Arc::clone(&done);
        let reader_join = thread::spawn(move || {
            let mut buf = [0_u8; 8192];
            let mut carry = ControlLineState::default();
            let mut attached = false;
            while let Ok(count) = stdout.read(&mut buf) {
                if count == 0 {
                    break;
                }
                let chunk_signal = tmux_control_chunk_has_activity(&mut carry, &buf[..count]);
                if carry.is_attached() && !attached {
                    attached = true;
                    if let Ok(mut state) = reader_signal.0.lock() {
                        if !state.stop {
                            reader_live.store(true, Ordering::Release);
                            state.dirty = true;
                            reader_signal.1.notify_one();
                        }
                    }
                }
                match chunk_signal {
                    ControlChunkSignal::Activity => {
                        if let Ok(mut state) = reader_signal.0.lock() {
                            if state.stop {
                                break;
                            }
                            state.dirty = true;
                            reader_signal.1.notify_one();
                        }
                    }
                    ControlChunkSignal::Exited => {
                        if let Ok(mut state) = reader_signal.0.lock() {
                            if carry.had_activity_in_last_chunk() && !state.stop {
                                state.dirty = true;
                            }
                            state.exited = true;
                            reader_signal.1.notify_all();
                        }
                        break;
                    }
                    ControlChunkSignal::None => {}
                }
            }
            reader_live.store(false, Ordering::Release);
            reader_done.store(true, Ordering::Release);
            if let Ok(mut state) = reader_signal.0.lock() {
                state.exited = true;
                reader_signal.1.notify_all();
            }
            if let Ok(mut child) = reader_child.lock() {
                child.shutdown();
            }
        });

        let coalescer_signal = Arc::clone(&signal);
        let coalescer_focused = Arc::clone(&self.focused);
        let coalescer_theme = Arc::clone(&self.theme_code);
        let coalescer_repaint = Arc::clone(&self.repaint);
        let coalescer_sender = self.sender.clone();
        let coalescer_name = tmux_name.clone();
        #[cfg(test)]
        let coalescer_captures = Arc::clone(&self.total_captures);
        #[cfg(test)]
        let coalescer_deliveries = Arc::clone(&self.total_deliveries);

        let coalescer_join = thread::spawn(move || {
            let mut last_capture_at: Option<Instant> = None;
            let mut worker_inc = seed.incremental;
            let mut last_delivered = seed.output.map(|output| LastDeliveredSnapshot {
                theme: seed.theme,
                fingerprint: terminal_output_fingerprint(&output),
                output,
                cursor: seed.cursor,
            });

            loop {
                let exiting_after = {
                    let Ok(mut guard) = coalescer_signal.0.lock() else {
                        return;
                    };
                    while !guard.dirty && !guard.exited && !guard.stop {
                        guard = match coalescer_signal.1.wait(guard) {
                            Ok(guard) => guard,
                            Err(_) => return,
                        };
                    }
                    if guard.stop || (!guard.dirty && guard.exited) {
                        return;
                    }
                    if !guard.exited {
                        let budget = if coalescer_focused.load(Ordering::Acquire) {
                            TERMINAL_STREAM_FRAME_BUDGET
                        } else {
                            TERMINAL_STREAM_UNFOCUSED_BUDGET
                        };
                        let delay = stream_coalesce_delay(last_capture_at, Instant::now(), budget);
                        if !delay.is_zero() {
                            guard =
                                match coalescer_signal
                                    .1
                                    .wait_timeout_while(guard, delay, |st| !st.stop && !st.exited)
                                {
                                    Ok((guard, _)) => guard,
                                    Err(_) => return,
                                };
                            if guard.stop {
                                return;
                            }
                        }
                    }
                    let exiting = guard.exited;
                    guard.dirty = false;
                    exiting
                };

                last_capture_at = Some(Instant::now());
                #[cfg(test)]
                coalescer_captures.fetch_add(1, Ordering::Relaxed);

                let Ok(capture) = tmux_capture_with_cursor(&coalescer_name) else {
                    if exiting_after {
                        return;
                    }
                    continue;
                };

                if coalescer_signal
                    .0
                    .lock()
                    .map(|state| state.stop)
                    .unwrap_or(true)
                {
                    return;
                }

                let theme = app_theme_from_code(coalescer_theme.load(Ordering::Acquire));
                if let Some(update) = classify_captured_snapshot(
                    session_id,
                    capture,
                    theme,
                    &mut worker_inc,
                    &mut last_delivered,
                ) {
                    if coalescer_sender.send(update).is_ok() {
                        #[cfg(test)]
                        coalescer_deliveries.fetch_add(1, Ordering::Relaxed);
                        wake_visible_session(&visible_sessions, &coalescer_repaint, session_id);
                    }
                }

                if exiting_after {
                    return;
                }
            }
        });

        self.entries.insert(
            session_id,
            SessionStreamHandle {
                tmux_name,
                signal,
                child,
                live,
                done,
                reader_join: Some(reader_join),
                coalescer_join: Some(coalescer_join),
            },
        );
    }
}

fn wake_visible_session(
    visible_sessions: &Arc<std::sync::RwLock<HashSet<Uuid>>>,
    repaint: &Arc<Mutex<Option<egui::Context>>>,
    session_id: Uuid,
) {
    if visible_sessions
        .read()
        .is_ok_and(|visible| visible.contains(&session_id))
    {
        if let Ok(guard) = repaint.lock() {
            if let Some(ctx) = guard.as_ref() {
                ctx.request_repaint();
            }
        }
    }
}

impl OperonApp {
    pub(crate) fn sync_terminal_streams(&mut self, focused: bool) {
        self.terminal_streams
            .focused
            .store(focused, Ordering::Release);
        self.terminal_streams
            .theme_code
            .store(app_theme_code(self.store.theme), Ordering::Release);
        if let Ok(mut slot) = self.terminal_streams.repaint.lock() {
            *slot = self.repaint.clone();
        }

        let now = Instant::now();
        let desired: Vec<(Uuid, String)> = if self.tools.tmux {
            self.terminals_on_screen()
                .into_iter()
                .take(PANE_MAX)
                .filter_map(|id| {
                    let session = self.store.sessions.iter().find(|s| s.id == id)?;
                    (session.status == SessionStatus::Active
                        && is_safe_tmux_name(&session.tmux_name))
                    .then(|| (session.id, session.tmux_name.clone()))
                })
                .collect()
        } else {
            Vec::new()
        };

        let stale_ids: Vec<(Uuid, bool)> = self
            .terminal_streams
            .entries
            .iter()
            .filter_map(|(id, entry)| {
                let worker_done = entry.done.load(Ordering::Acquire);
                let still_wanted = desired
                    .iter()
                    .any(|(did, dname)| did == id && dname == &entry.tmux_name);
                if worker_done || !still_wanted {
                    Some((*id, worker_done))
                } else {
                    None
                }
            })
            .collect();

        for (id, record_cooldown) in stale_ids {
            if let Some(mut entry) = self.terminal_streams.entries.remove(&id) {
                let name = entry.tmux_name.clone();
                entry.stop(false);
                if record_cooldown {
                    self.terminal_streams.cooldowns.insert((id, name), now);
                }
            }
        }

        self.terminal_streams
            .cooldowns
            .retain(|_, started| now.saturating_duration_since(*started) < SESSION_POLL_INTERVAL);

        for (session_id, tmux_name) in desired {
            if self.terminal_streams.entries.contains_key(&session_id) {
                continue;
            }
            if self
                .terminal_streams
                .cooldowns
                .get(&(session_id, tmux_name.clone()))
                .is_some_and(|at| now.saturating_duration_since(*at) < SESSION_POLL_INTERVAL)
            {
                continue;
            }
            let has_current_rows = self.terminal_layouts.contains_key(&session_id);
            let seed = StreamSeed {
                theme: self.store.theme,
                output: has_current_rows
                    .then(|| self.session_output.get(&session_id).cloned())
                    .flatten(),
                cursor: self.terminal_cursors.get(&session_id).copied(),
                incremental: has_current_rows
                    .then(|| self.terminal_incremental_layouts.get(&session_id).cloned())
                    .flatten(),
            };
            let visible = Arc::clone(&self.visible_sessions);
            self.terminal_streams
                .spawn_watcher(session_id, tmux_name, seed, visible, now);
        }
    }

    pub(crate) fn drain_terminal_streams(&mut self) {
        let mut folded: HashMap<Uuid, StreamedTerminalUpdate> = HashMap::new();
        while let Ok(update) = self.terminal_streams.receiver.try_recv() {
            let session_id = match &update {
                StreamedTerminalUpdate::CursorOnly { session_id, .. }
                | StreamedTerminalUpdate::Full { session_id, .. } => *session_id,
            };
            if !self.store.sessions.iter().any(|s| s.id == session_id) {
                continue;
            }
            match (folded.remove(&session_id), update) {
                (None, incoming) => {
                    folded.insert(session_id, incoming);
                }
                (
                    Some(StreamedTerminalUpdate::Full {
                        mut capture,
                        theme: prev_theme,
                        fingerprint: prev_fp,
                        incremental,
                        jobs,
                        line_ids,
                        dirty_lines,
                        ..
                    }),
                    StreamedTerminalUpdate::CursorOnly {
                        theme,
                        fingerprint,
                        cursor,
                        ..
                    },
                ) => {
                    if prev_theme == theme && prev_fp == fingerprint {
                        capture.cursor = cursor;
                    }
                    folded.insert(
                        session_id,
                        StreamedTerminalUpdate::Full {
                            session_id,
                            capture,
                            theme: prev_theme,
                            fingerprint: prev_fp,
                            incremental,
                            jobs,
                            line_ids,
                            dirty_lines,
                        },
                    );
                }
                (
                    Some(StreamedTerminalUpdate::Full { .. }),
                    StreamedTerminalUpdate::Full {
                        capture,
                        theme,
                        fingerprint,
                        incremental,
                        jobs,
                        line_ids,
                        ..
                    },
                ) => {
                    folded.insert(
                        session_id,
                        StreamedTerminalUpdate::Full {
                            session_id,
                            capture,
                            theme,
                            fingerprint,
                            incremental,
                            jobs,
                            line_ids,
                            dirty_lines: None,
                        },
                    );
                }
                (Some(StreamedTerminalUpdate::CursorOnly { .. }), incoming) => {
                    folded.insert(session_id, incoming);
                }
            }
        }
        for (_, update) in folded {
            match update {
                StreamedTerminalUpdate::CursorOnly {
                    session_id,
                    theme,
                    fingerprint,
                    cursor,
                } => {
                    let matches_held =
                        self.terminal_output_fingerprints.get(&session_id) == Some(&fingerprint)
                            || self.session_output.get(&session_id).is_some_and(|held| {
                                terminal_output_fingerprint(held) == fingerprint
                            });
                    if self.store.theme == theme
                        && matches_held
                        && self.terminal_layouts.contains_key(&session_id)
                    {
                        self.terminal_output_fingerprints
                            .insert(session_id, fingerprint);
                        if let Some(cursor) = cursor {
                            self.terminal_cursors.insert(session_id, cursor);
                        } else {
                            self.terminal_cursors.remove(&session_id);
                        }
                    } else if self.session_output.contains_key(&session_id)
                        && !self.terminal_layouts.contains_key(&session_id)
                    {
                        self.request_session_output(session_id);
                    }
                }
                StreamedTerminalUpdate::Full {
                    session_id,
                    capture,
                    theme,
                    fingerprint,
                    incremental,
                    jobs,
                    line_ids,
                    dirty_lines,
                } => {
                    if self.store.theme == theme {
                        self.terminal_prelaid.insert(
                            session_id,
                            LaidOutTerminal {
                                theme,
                                fingerprint,
                                layouts: Some(jobs),
                            },
                        );
                        self.terminal_prelaid_incremental.insert(
                            session_id,
                            PrelaidIncremental {
                                theme,
                                incremental,
                                line_ids,
                                dirty_lines,
                            },
                        );
                    }
                    self.set_session_output(session_id, capture.output, capture.cursor);
                    self.terminal_prelaid.remove(&session_id);
                    self.terminal_prelaid_incremental.remove(&session_id);
                }
            }
        }
    }

    pub(crate) fn request_session_output(&mut self, session_id: Uuid) {
        let Some(session) = self
            .store
            .sessions
            .iter()
            .find(|session| session.id == session_id)
        else {
            return;
        };
        if self
            .background_tasks
            .contains(&BackgroundKey::SessionOutput(session_id))
        {
            return;
        }
        let tmux_name = session.tmux_name.clone();
        let theme = self.store.theme;
        let known = self.terminal_output_fingerprints.get(&session_id).copied();
        self.spawn_background(BackgroundKey::SessionOutput(session_id), move || {
            let (result, laid_out) = capture_laid_out(&tmux_name, theme, known);
            BackgroundResult::SessionOutput {
                session_id,
                result,
                laid_out,
            }
        });
    }

    pub(crate) fn set_session_output(
        &mut self,
        session_id: Uuid,
        output: String,
        cursor: Option<TerminalCursor>,
    ) {
        let prelaid = self.terminal_prelaid.remove(&session_id);
        let prelaid_inc = self.terminal_prelaid_incremental.remove(&session_id);
        if let Some(cursor) = cursor {
            self.terminal_cursors.insert(session_id, cursor);
        } else {
            self.terminal_cursors.remove(&session_id);
        }
        if self
            .session_output
            .get(&session_id)
            .is_some_and(|existing| *existing == output)
            && self.terminal_layouts.contains_key(&session_id)
        {
            if let Some(laid_out) = prelaid.filter(|laid_out| laid_out.theme == self.store.theme) {
                self.terminal_output_fingerprints
                    .insert(session_id, laid_out.fingerprint);
            }
            return;
        }

        let theme = self.store.theme;
        let dirty_lines: Option<Vec<usize>>;
        let layouts = match prelaid {
            Some(LaidOutTerminal {
                theme: laid_out_under,
                fingerprint,
                layouts: Some(layouts),
            }) if laid_out_under == theme => {
                self.terminal_output_fingerprints
                    .insert(session_id, fingerprint);
                if let Some(inc) = prelaid_inc.filter(|inc| inc.theme == theme) {
                    self.terminal_incremental_layouts
                        .insert(session_id, inc.incremental);
                    self.terminal_line_ids.insert(session_id, inc.line_ids);
                    dirty_lines = inc.dirty_lines;
                } else {
                    self.terminal_incremental_layouts.remove(&session_id);
                    self.terminal_line_ids.remove(&session_id);
                    dirty_lines = None;
                }
                layouts
            }
            _ => {
                self.terminal_output_fingerprints.remove(&session_id);
                let prev = self
                    .terminal_incremental_layouts
                    .get(&session_id)
                    .map(Arc::as_ref);
                let (mut inc, jobs, line_ids) =
                    terminal_line_layouts_incremental(&output, theme, prev);
                let dirty = inc.take_dirty_lines();
                self.terminal_incremental_layouts
                    .insert(session_id, Arc::new(inc));
                self.terminal_line_ids.insert(session_id, line_ids);
                dirty_lines = Some(dirty);
                jobs
            }
        };

        self.terminal_fallback_layouts.remove(&session_id);
        self.session_output.insert(session_id, output);
        let gen = self
            .terminal_output_generations
            .entry(session_id)
            .or_insert(0);
        *gen = gen.saturating_add(1);

        let now = Instant::now();
        let is_selected = self.selected_session == Some(session_id);
        let retain_negative_path = |path: &String,
                                    probed_map: &HashMap<String, Instant>,
                                    layouts_ref: &[LayoutJob],
                                    dirty_opt: Option<&[usize]>|
         -> bool {
            if !is_selected {
                return false;
            }
            let Some(&probed_at) = probed_map.get(path) else {
                return false;
            };
            if now.saturating_duration_since(probed_at) >= TERMINAL_NEGATIVE_PATH_TTL {
                return false;
            }
            match dirty_opt {
                Some(indices) => !indices.iter().any(|&idx| {
                    layouts_ref
                        .get(idx)
                        .is_some_and(|job| job.text.contains(path.as_str()))
                }),
                None => false,
            }
        };

        let mut evicted_negative: Vec<String> = Vec::new();
        if let Some(cache) = self.terminal_path_caches.get_mut(&session_id) {
            let probed_ref = &self.negative_path_probed_at;
            let dirty_slice = dirty_lines.as_deref();
            cache.retain(|path, resolved| {
                if resolved.is_some()
                    || retain_negative_path(path, probed_ref, &layouts, dirty_slice)
                {
                    true
                } else {
                    evicted_negative.push(path.clone());
                    false
                }
            });
        }
        if self.resolved_paths_session == Some(session_id) {
            let probed_ref = &self.negative_path_probed_at;
            let dirty_slice = dirty_lines.as_deref();
            self.resolved_paths.retain(|path, resolved| {
                if resolved.is_some()
                    || retain_negative_path(path, probed_ref, &layouts, dirty_slice)
                {
                    true
                } else {
                    evicted_negative.push(path.clone());
                    false
                }
            });
            self.resolved_paths_generation += 1;
        }
        for path in evicted_negative {
            self.negative_path_probed_at.remove(&path);
        }

        self.terminal_layouts.insert(session_id, layouts);
        self.refresh_terminal_search(session_id);
    }

    /// Re-read the pane while its echo may still be on the way, and say how
    /// long until the next look; `None` when a `tmux -C` watcher is live, once
    /// nobody has typed for `TERMINAL_ECHO_WINDOW`, or while a capture is in flight.
    pub(crate) fn poll_terminal_echo(
        &mut self,
        session_id: Uuid,
        now: Instant,
    ) -> Option<Duration> {
        if self.terminal_streams.is_watching(session_id) {
            return None;
        }
        let last_poll = self
            .last_output_poll
            .filter(|(polled, _)| *polled == session_id)
            .map(|(_, at)| at);
        let wait = terminal_echo_poll_wait(self.last_terminal_input, last_poll, now)?;
        if !wait.is_zero() {
            return Some(wait);
        }
        if self
            .background_tasks
            .contains(&BackgroundKey::SessionOutput(session_id))
        {
            return None;
        }
        self.last_output_poll = Some((session_id, now));
        self.request_session_output(session_id);
        Some(TERMINAL_ECHO_POLL)
    }

    pub(crate) fn flush_terminal_input(&mut self, session_id: Uuid) {
        if self
            .background_tasks
            .contains(&BackgroundKey::TerminalInput(session_id))
        {
            return;
        }
        let input = self
            .terminal_input_queue
            .get_mut(&session_id)
            .map(|queue| queue.drain(..).collect::<Vec<_>>())
            .unwrap_or_default();
        if input.is_empty() {
            return;
        }
        let Some(tmux_name) = self
            .store
            .sessions
            .iter()
            .find(|session| session.id == session_id && session.status == SessionStatus::Active)
            .map(|session| session.tmux_name.clone())
        else {
            return;
        };
        self.spawn_background(BackgroundKey::TerminalInput(session_id), move || {
            BackgroundResult::TerminalInputSent {
                session_id,
                result: tmux_send_input(&tmux_name, &input).map_err(|error| error.to_string()),
            }
        });
    }

    pub(crate) fn is_layout_dragging(&self) -> bool {
        self.workspace_divider_drag || self.workspace_drag.is_some()
    }

    pub(crate) fn finish_layout_drag(&mut self) {
        for session_id in self.terminals_on_screen() {
            if let Some(&requested) = self.terminal_resize_requested.get(&session_id) {
                if self.terminal_resize_applied.get(&session_id) != Some(&requested) {
                    self.terminal_resize_changed_at.remove(&session_id);
                    self.flush_terminal_resize(session_id);
                } else if !self.terminal_streams.is_watching(session_id) {
                    self.request_session_output(session_id);
                }
            }
        }
    }

    pub(crate) fn request_terminal_resize(
        &mut self,
        session_id: Uuid,
        size: (u16, u16),
    ) -> Option<Duration> {
        if self.terminal_resize_requested.get(&session_id) != Some(&size) {
            self.terminal_resize_requested.insert(session_id, size);
            self.terminal_resize_changed_at
                .insert(session_id, Instant::now());
        }
        self.flush_terminal_resize(session_id)
    }

    pub(crate) fn flush_terminal_resize(&mut self, session_id: Uuid) -> Option<Duration> {
        if self
            .background_tasks
            .contains(&BackgroundKey::TerminalResize(session_id))
        {
            return None;
        }
        let size = self.terminal_resize_requested.get(&session_id).copied()?;
        let applied = self.terminal_resize_applied.get(&session_id).copied();
        if applied == Some(size) {
            return None;
        }
        if applied.is_some() {
            if let Some(changed) = self.terminal_resize_changed_at.get(&session_id) {
                let held = changed.elapsed();
                if held < TERMINAL_RESIZE_SETTLE {
                    return Some(TERMINAL_RESIZE_SETTLE - held);
                }
            }
        }
        let tmux_name = self
            .store
            .sessions
            .iter()
            .find(|session| session.id == session_id && session.status == SessionStatus::Active)
            .map(|session| session.tmux_name.clone())?;
        self.spawn_background(BackgroundKey::TerminalResize(session_id), move || {
            BackgroundResult::TerminalResized {
                session_id,
                size,
                result: tmux_resize(&tmux_name, size).map_err(|error| error.to_string()),
            }
        });
        None
    }
}
