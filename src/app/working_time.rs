//! How long each agent has been working on its current turn (change 278).
//! A waiting session says how long it has waited; this is the same reading
//! for the state where a stuck agent and a busy one otherwise look alike.
//! Nothing is stored: it is measured from when this run first saw the turn.

use crate::*;

/// The time as a row says it, in the waiting time's units. Nothing under a
/// minute: a count that has not moved yet is not worth a row's width.
pub(crate) fn working_time_label(worked: Duration) -> Option<String> {
    let minutes = worked.as_secs() / 60;
    match minutes {
        0 => None,
        1..=59 => Some(tf!("{p0}分", p0 = minutes)),
        _ => Some(tf!("{p0}時間", p0 = minutes / 60)),
    }
}

impl OperonApp {
    /// Keeps the working clocks in step with the sessions, once per frame
    /// before anything draws, in the shape of `track_attention`: the map is
    /// moved out for the pass, and nothing is allocated when nothing changed.
    pub(crate) fn track_working(&mut self, now: Instant) {
        let mut since = std::mem::take(&mut self.working_since);
        let mut seen = 0;
        for session in &self.store.sessions {
            if self.session_status_kind_for(session) == SessionStatusKind::Working {
                since.entry(session.id).or_insert(now);
                seen += 1;
            } else {
                since.remove(&session.id);
            }
        }
        // Only a session that is gone can be left over.
        if since.len() != seen {
            since.retain(|id, _| self.store.sessions.iter().any(|session| session.id == *id));
        }
        self.working_since = since;
    }

    pub(crate) fn working_time(&self, session_id: Uuid, now: Instant) -> Option<Duration> {
        self.working_since
            .get(&session_id)
            .map(|began| now.saturating_duration_since(*began))
    }

    pub(crate) fn working_time_text(&self, session_id: Uuid, now: Instant) -> Option<String> {
        self.working_time(session_id, now)
            .and_then(working_time_label)
    }

    /// `text` with the session's working time after it, when there is one.
    pub(crate) fn with_working_time(&self, text: &str, session_id: Uuid, now: Instant) -> String {
        match self.working_time_text(session_id, now) {
            Some(time) => tf!("{p0} · {p1}", p0 = text, p1 = time),
            None => text.to_owned(),
        }
    }
}
