//! A terminal's captured text and the rows laid out from it (change 276).
//! Parsing 5 000 lines of ANSI takes milliseconds, so the thread that reads
//! tmux does it and the frame only stores the result.

use crate::*;
use std::hash::{Hash, Hasher};

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

impl OperonApp {
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
        // Rows that came with this text, when the handler stashed any.
        let prelaid = self.terminal_prelaid.remove(&session_id);
        if let Some(cursor) = cursor {
            self.terminal_cursors.insert(session_id, cursor);
        } else {
            // A capture without the marker line is the pane saying it does not
            // know where the caret is (tmux exits 0 on the lost-target path
            // with no marker at all). Keeping the previous one here would map
            // old pane rows onto every new buffer length — rows of scrollback
            // get a box forever, exactly the drift change 155 removed. Clear
            // before the identical-output early return below, so the stale
            // entry cannot survive on the dedup path either.
            self.terminal_cursors.remove(&session_id);
        }
        // ANSI parsing is noticeably more expensive than drawing a button.
        // Parse only when tmux has supplied fresh output (or when a theme change
        // took `terminal_layouts` away), rather than once per poll while the
        // terminal pane is idle.
        if self
            .session_output
            .get(&session_id)
            .is_some_and(|existing| *existing == output)
            && self.terminal_layouts.contains_key(&session_id)
        {
            // The rows held are this text's, under the theme in use: its
            // fingerprint lets the next capture skip laying it out, also
            // when it was the frame that laid these rows out.
            if let Some(laid_out) = prelaid.filter(|laid_out| laid_out.theme == self.store.theme) {
                self.terminal_output_fingerprints
                    .insert(session_id, laid_out.fingerprint);
            }
            return;
        }
        // The capture thread's rows when it laid this text out under the theme
        // still in use; otherwise here, as before change 276. The fingerprint
        // is kept only with rows that came from it, so the next capture is
        // not told "unchanged" about text this frame laid out itself.
        let theme = self.store.theme;
        let layouts = match prelaid {
            Some(LaidOutTerminal {
                theme: laid_out_under,
                fingerprint,
                layouts: Some(layouts),
            }) if laid_out_under == theme => {
                self.terminal_output_fingerprints
                    .insert(session_id, fingerprint);
                layouts
            }
            _ => {
                self.terminal_output_fingerprints.remove(&session_id);
                terminal_line_layouts(&output, theme.palette())
            }
        };
        // The fallback holds rows only between a theme switch and the capture
        // that replaces them: this is that capture.
        self.terminal_fallback_layouts.remove(&session_id);
        self.terminal_layouts.insert(session_id, layouts);
        self.session_output.insert(session_id, output);
        let gen = self
            .terminal_output_generations
            .entry(session_id)
            .or_insert(0);
        *gen = gen.saturating_add(1);
        // The buffer moved, so where the query occurs moved with it. Here
        // rather than per frame: this walks every line, and the lines only
        // change when tmux has said something new.
        self.refresh_terminal_search(session_id);
        if let Some(cache) = self.terminal_path_caches.get_mut(&session_id) {
            cache.retain(|_, resolved| resolved.is_some());
        }
        if self.resolved_paths_session == Some(session_id) {
            self.resolved_paths.retain(|_, resolved| resolved.is_some());
            self.resolved_paths_generation += 1;
        }
    }
}
