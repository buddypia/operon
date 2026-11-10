//! The Dock icon's badge: how many sessions wait for the person, so the
//! number is in view while another application is in front (change 281).

use crate::*;

/// What the badge reads for `waiting` sessions; nothing when nobody waits.
pub(crate) fn dock_badge_label(waiting: usize) -> Option<String> {
    (waiting > 0).then(|| waiting.to_string())
}

impl OperonApp {
    /// The badge to show, when it differs from the one last asked for: the
    /// Dock is told on a change, not every frame.
    pub(crate) fn dock_badge_change(&mut self) -> Option<Option<String>> {
        let waiting = self.attention_order.len();
        if self.dock_badge_shown == waiting {
            return None;
        }
        self.dock_badge_shown = waiting;
        Some(dock_badge_label(waiting))
    }

    /// Once per frame, after `track_attention` has settled the queue.
    pub(crate) fn track_dock_badge(&mut self) {
        if let Some(label) = self.dock_badge_change() {
            set_dock_badge(label.as_deref());
        }
    }
}
