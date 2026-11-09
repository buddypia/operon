use crate::*;

/// Operon's icon vocabulary. One glyph per action, reused everywhere that
/// action appears, so each symbol only has to be learned once. Every icon-only
/// control carries the wording it replaced as hover text — an icon nobody can
/// decode is worse than the long button it saves space over.
///
/// Every glyph is drawn from Phosphor Icons (MIT), bundled as `phosphor` in the
/// proportional font chain. The previous vocabulary was assorted Unicode
/// symbols — an arrow from one block, a box-drawing mark from another, a
/// fullwidth plus — which no two faces draw at the same weight, stroke, or
/// optical size. One icon family means one stroke width across the whole app,
/// and the licence is one a shipped binary can carry.
///
/// Phosphor's codepoints sit in the private use area, so they resolve from that
/// face alone: a name that is not in `egui_phosphor::regular` renders as a
/// blank box. `every_icon_resolves_from_the_bundled_icon_font` holds that.
pub(crate) const ICON_REFRESH: &str = icons::ARROWS_CLOCKWISE;
pub(crate) const ICON_OPEN_EXTERNAL: &str = icons::ARROW_SQUARE_OUT;
pub(crate) const ICON_SHOW: &str = icons::TERMINAL_WINDOW;
pub(crate) const ICON_COPY: &str = icons::COPY;
pub(crate) const ICON_START: &str = icons::PLAY;
pub(crate) const ICON_STOP: &str = icons::STOP;
pub(crate) const ICON_CLOSE: &str = icons::X;
pub(crate) const ICON_RESUME: &str = icons::ARROW_COUNTER_CLOCKWISE;
pub(crate) const ICON_RESTORE: &str = icons::ARROWS_LEFT_RIGHT;
pub(crate) const ICON_ADD: &str = icons::PLUS;
pub(crate) const ICON_SETTINGS: &str = icons::GEAR;
pub(crate) const ICON_SEARCH: &str = icons::MAGNIFYING_GLASS;
pub(crate) const ICON_PROJECT: &str = icons::FOLDER;
pub(crate) const ICON_BRANCH: &str = icons::GIT_BRANCH;
pub(crate) const ICON_BACK: &str = icons::ARROW_LEFT;
pub(crate) const ICON_DEPENDS_ON: &str = icons::ARROW_LEFT;
pub(crate) const ICON_DISCLOSURE_OPEN: &str = icons::CARET_DOWN;
pub(crate) const ICON_DISCLOSURE_CLOSED: &str = icons::CARET_RIGHT;
pub(crate) const ICON_AVAILABLE: &str = icons::CHECK;
pub(crate) const ICON_UNAVAILABLE: &str = icons::X;
pub(crate) const ICON_SKILL: &str = icons::SPARKLE;
pub(crate) const ICON_NOTICE: &str = icons::INFO;

/// The editor's own marks. A file gets one of three faces — plain text,
/// Markdown, or source — because that is the distinction a person makes when
/// they scan a project tree, and no more: a per-extension icon set would be a
/// vocabulary nobody could finish and nobody asked for.
pub(crate) const ICON_FILE: &str = icons::FILE_TEXT;
pub(crate) const ICON_FILE_MARKDOWN: &str = icons::FILE_MD;
pub(crate) const ICON_FILE_CODE: &str = icons::FILE_CODE;
pub(crate) const ICON_FOLDER_CLOSED: &str = icons::FOLDER;
pub(crate) const ICON_FOLDER_OPEN: &str = icons::FOLDER_OPEN;
pub(crate) const ICON_EDIT: &str = icons::PENCIL_SIMPLE;
pub(crate) const ICON_PREVIEW: &str = icons::EYE;
pub(crate) const ICON_DIFF: &str = icons::GIT_DIFF;
pub(crate) const ICON_SAVE: &str = icons::FLOPPY_DISK;
/// A file the change creates, and one it deletes. A file it merely edits keeps
/// `ICON_FILE`: a modified file is still just a file, and inventing a fourth
/// mark for it would say less than the `+12 −3` already beside the name.
pub(crate) const ICON_FILE_ADDED: &str = icons::FILE_PLUS;
pub(crate) const ICON_FILE_REMOVED: &str = icons::FILE_MINUS;
/// A checklist item in rendered Markdown, done and not done. These are the one
/// place a mark stands in for text the author actually wrote — `[x]` and `[ ]`
/// — and they earn it because a checklist is the shape an agent's plan comes
/// in, and a page of literal brackets is the raw syntax the rendered view
/// exists to replace. A square rather than a circle: a circle is this app's
/// enclosure for a status, and a step somebody has to do is not a status.
pub(crate) const ICON_TASK_DONE: &str = icons::CHECK_SQUARE;
pub(crate) const ICON_TASK_TODO: &str = icons::SQUARE;
/// Edits that are in the editor but not yet on disk. A ring rather than
/// Phosphor's `dot` for the reason the status marks give: at the size a tab is
/// drawn, a dot is a smudge and a ring is a shape.
pub(crate) const ICON_UNSAVED: &str = icons::CIRCLE;
/// A state that needs the person: an agent stopped on a question, or work that
/// cannot start.
pub(crate) const ICON_ATTENTION: &str = icons::WARNING;
pub(crate) const ICON_PENDING: &str = icons::HOURGLASS_MEDIUM;

/// The marks a status chip leads with. A status is not an action, so none of
/// these is a bare action: a circle or circular arrows give the mark its
/// status enclosure, with pause, check, cross or minus distinguishing states.
/// That keeps a chip from reading as a button in the same row, and it survives being the wrong colour for a person who cannot use the
/// colour to tell running from idle.
///
/// Phosphor's own `dot` is not one of these: its ink is three pixels across at
/// the size a chip is drawn, which is a smudge rather than a state.
pub(crate) const ICON_STATUS_RUNNING: &str = icons::ARROWS_CLOCKWISE;
pub(crate) const ICON_STATUS_IDLE: &str = icons::PAUSE_CIRCLE;
pub(crate) const ICON_STATUS_STARTING: &str = icons::POWER;
pub(crate) const ICON_STATUS_QUEUED: &str = icons::PLAY_CIRCLE;
pub(crate) const ICON_STATUS_DONE: &str = icons::CHECK_CIRCLE;
pub(crate) const ICON_STATUS_FAILED: &str = icons::X_CIRCLE;
pub(crate) const ICON_STATUS_LOST: &str = icons::MINUS_CIRCLE;
pub(crate) const ICON_STATUS_UNKNOWN: &str = icons::QUESTION;
/// The main working tree and the worktrees added beside it.
pub(crate) const ICON_TREE_MAIN: &str = icons::HOUSE;
pub(crate) const ICON_TREE_EXTRA: &str = icons::FOLDER_SIMPLE;
/// The rest of what a row can do. Every action behind it is either rare or
/// irreversible, which is the only reason to spend a second click on one — the
/// actions a person reaches for while scanning stay on the row itself.
pub(crate) const ICON_MORE: &str = icons::DOTS_THREE;

/// Drag handle grip on draggable tabs and panels.
pub(crate) const ICON_DRAG_HANDLE: &str = icons::DOTS_SIX_VERTICAL;

/// A note a person wrote against a line of a diff. It marks the line in the
/// review pane and counts the set in the footer, which is two places, which is
/// why it is a constant rather than a glyph at a call site.
pub(crate) const ICON_COMMENT: &str = icons::CHAT_TEXT;

/// Stepping backwards and forwards through the places a search found. Two
/// marks rather than words, because they sit on a bar that is one row.
pub(crate) const ICON_PREVIOUS: &str = icons::CARET_UP;
pub(crate) const ICON_NEXT: &str = icons::CARET_DOWN;
/// The ナレッジ tab, and a knowledge entry kept by hand (change 157).
pub(crate) const ICON_KNOWLEDGE: &str = icons::LIGHTBULB;
pub(crate) const ICON_PIN: &str = icons::PUSH_PIN;
/// 「全体を表示」 on the knowledge map: fit every point in view.
pub(crate) const ICON_FIT: &str = icons::ARROWS_OUT;

/// The whole vocabulary, so a test can check it rather than trusting that every
/// name above happens to be spelled the way Phosphor spells it. A glyph missing
/// from the font is a blank box at runtime and nothing at compile time.
#[cfg(test)]
pub(crate) const ICON_VOCABULARY: &[(&str, &str)] = &[
    ("ICON_COMMENT", ICON_COMMENT),
    ("ICON_PREVIOUS", ICON_PREVIOUS),
    ("ICON_NEXT", ICON_NEXT),
    ("ICON_REFRESH", ICON_REFRESH),
    ("ICON_OPEN_EXTERNAL", ICON_OPEN_EXTERNAL),
    ("ICON_SHOW", ICON_SHOW),
    ("ICON_COPY", ICON_COPY),
    ("ICON_START", ICON_START),
    ("ICON_STOP", ICON_STOP),
    ("ICON_CLOSE", ICON_CLOSE),
    ("ICON_RESUME", ICON_RESUME),
    ("ICON_RESTORE", ICON_RESTORE),
    ("ICON_ADD", ICON_ADD),
    ("ICON_SETTINGS", ICON_SETTINGS),
    ("ICON_SEARCH", ICON_SEARCH),
    ("ICON_PROJECT", ICON_PROJECT),
    ("ICON_BRANCH", ICON_BRANCH),
    ("ICON_BACK", ICON_BACK),
    ("ICON_DEPENDS_ON", ICON_DEPENDS_ON),
    ("ICON_DISCLOSURE_OPEN", ICON_DISCLOSURE_OPEN),
    ("ICON_DISCLOSURE_CLOSED", ICON_DISCLOSURE_CLOSED),
    ("ICON_AVAILABLE", ICON_AVAILABLE),
    ("ICON_UNAVAILABLE", ICON_UNAVAILABLE),
    ("ICON_SKILL", ICON_SKILL),
    ("ICON_NOTICE", ICON_NOTICE),
    ("ICON_FILE", ICON_FILE),
    ("ICON_FILE_MARKDOWN", ICON_FILE_MARKDOWN),
    ("ICON_FILE_CODE", ICON_FILE_CODE),
    ("ICON_FOLDER_CLOSED", ICON_FOLDER_CLOSED),
    ("ICON_FOLDER_OPEN", ICON_FOLDER_OPEN),
    ("ICON_EDIT", ICON_EDIT),
    ("ICON_PREVIEW", ICON_PREVIEW),
    ("ICON_DIFF", ICON_DIFF),
    ("ICON_SAVE", ICON_SAVE),
    ("ICON_FILE_ADDED", ICON_FILE_ADDED),
    ("ICON_FILE_REMOVED", ICON_FILE_REMOVED),
    ("ICON_TASK_DONE", ICON_TASK_DONE),
    ("ICON_TASK_TODO", ICON_TASK_TODO),
    ("ICON_UNSAVED", ICON_UNSAVED),
    ("ICON_ATTENTION", ICON_ATTENTION),
    ("ICON_PENDING", ICON_PENDING),
    ("ICON_STATUS_RUNNING", ICON_STATUS_RUNNING),
    ("ICON_STATUS_IDLE", ICON_STATUS_IDLE),
    ("ICON_STATUS_STARTING", ICON_STATUS_STARTING),
    ("ICON_STATUS_QUEUED", ICON_STATUS_QUEUED),
    ("ICON_STATUS_DONE", ICON_STATUS_DONE),
    ("ICON_STATUS_FAILED", ICON_STATUS_FAILED),
    ("ICON_STATUS_LOST", ICON_STATUS_LOST),
    ("ICON_STATUS_UNKNOWN", ICON_STATUS_UNKNOWN),
    ("ICON_TREE_MAIN", ICON_TREE_MAIN),
    ("ICON_TREE_EXTRA", ICON_TREE_EXTRA),
    ("ICON_MORE", ICON_MORE),
    ("ICON_DRAG_HANDLE", ICON_DRAG_HANDLE),
    ("ICON_KNOWLEDGE", ICON_KNOWLEDGE),
    ("ICON_PIN", ICON_PIN),
    ("ICON_FIT", ICON_FIT),
];

/// Each CLI's own product mark, so a card names an agent the way its vendor
/// does instead of asking a person to learn which colour means which tool. The
/// text stays beside every mark: a logo alone only names a product to someone
/// who already recognises it.
///
/// All three are 128x128 PNGs of the icon macOS shows for that vendor's app,
/// cropped to the rounded square itself. One silhouette for all three is what
/// stops the row reading as three unrelated shapes.
/// `assets/agent-icons/SOURCES.md` records where each one came from.
pub(crate) const AGENT_ICON_CLAUDE: &[u8] = include_bytes!("../assets/agent-icons/claude-code.png");
pub(crate) const AGENT_ICON_CODEX: &[u8] = include_bytes!("../assets/agent-icons/codex.png");
pub(crate) const AGENT_ICON_ANTIGRAVITY: &[u8] =
    include_bytes!("../assets/agent-icons/antigravity.png");

/// Operon's own mark, drawn the same way the vendor marks are: as artwork, not
/// as a glyph. A logo is not part of the icon vocabulary above — that vocabulary
/// is one stroke weight of outlines standing for actions, and a mark set in it
/// reads as a character rather than as a product, which is what this replaced.
///
/// Derived from `assets/operon-icon-1024.png`, the bundle master, by keying its
/// dark plate out of the alpha channel and resampling to 128x128 — the size the
/// three vendor marks already ship at. The plate is dropped rather than kept so
/// the mark carries no tile of its own onto the light theme.
pub(crate) const BRAND_MARK: &[u8] = include_bytes!("../assets/operon-mark-128.png");

/// The mark a plain or recovered terminal shows where an agent session shows
/// its vendor's (change 267). A shell has no vendor, so this one is Operon's
/// own artwork, covered by the repository's licence and kept out of
/// `assets/agent-icons/`, which holds only third-party marks. It takes their
/// shape — a full-bleed rounded square at 128x128 — so a terminal tab beside
/// an Antigravity tab reads as the same kind of thing. The light plate holds a
/// dark screen so the mark keeps an edge on the dark theme and a body on the
/// light one. Rendered from `assets/terminal-mark.svg` with
/// `rsvg-convert -w 1024` and a Lanczos resize to 128.
pub(crate) const TERMINAL_MARK: &[u8] = include_bytes!("../assets/terminal-mark-128.png");

/// How large a session's CLI mark is drawn beside its 12-point tab title: the
/// size the left column already gives a vendor mark beside 12-point text.
pub(crate) const SESSION_MARK_SIZE: f32 = 16.0;

/// Generated vector vocabulary for compact session status badges. All marks
/// share an enclosure, so they read as status indicators rather than actions.
/// The 18-point grid uses geometry instead of a font or scaled raster asset.
pub(crate) const STATUS_ICON_SIZE: f32 = 18.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StatusMark {
    Progress,
    Pause,
    Attention,
    Power,
    Play,
    Hourglass,
    Check,
    Cross,
    Stop,
    Minus,
    Question,
}

impl StatusMark {
    pub(crate) fn paint(self, painter: &egui::Painter, rect: egui::Rect, ink: Color32) {
        let center = rect.center();
        let scale = rect.width().min(rect.height()) / STATUS_ICON_SIZE;
        let stroke = egui::Stroke::new(1.5 * scale, ink);
        let point = |[x, y]: [f32; 2]| center + egui::vec2(x, y) * scale;
        let line = |a, b| {
            painter.line_segment([point(a), point(b)], stroke);
        };
        let path = |points: &[[f32; 2]]| {
            for edge in points.windows(2) {
                line(edge[0], edge[1]);
            }
        };
        painter.circle_stroke(center, 8.0 * scale, stroke);
        match self {
            Self::Progress => {
                path(&[
                    [-3.8, -0.8],
                    [-2.7, -3.0],
                    [0.0, -4.0],
                    [2.8, -3.0],
                    [3.8, -0.8],
                ]);
                path(&[[1.6, -1.7], [3.8, -0.8], [4.4, -3.0]]);
                path(&[[3.8, 0.8], [2.7, 3.0], [0.0, 4.0], [-2.8, 3.0], [-3.8, 0.8]]);
                path(&[[-1.6, 1.7], [-3.8, 0.8], [-4.4, 3.0]]);
            }
            Self::Pause => {
                line([-2.0, -3.5], [-2.0, 3.5]);
                line([2.0, -3.5], [2.0, 3.5]);
            }
            Self::Attention => {
                line([0.0, -4.0], [0.0, 0.5]);
                painter.circle_filled(point([0.0, 3.3]), 0.9 * scale, ink);
            }
            Self::Power => {
                path(&[
                    [-3.0, -2.0],
                    [-3.8, 0.0],
                    [-2.8, 2.8],
                    [0.0, 3.8],
                    [2.8, 2.8],
                    [3.8, 0.0],
                    [3.0, -2.0],
                ]);
                line([0.0, -4.2], [0.0, 0.0]);
            }
            Self::Play => path(&[[-2.3, -3.8], [3.5, 0.0], [-2.3, 3.8], [-2.3, -3.8]]),
            Self::Hourglass => path(&[
                [-3.0, -3.5],
                [3.0, -3.5],
                [2.5, -2.0],
                [-2.5, 2.0],
                [-3.0, 3.5],
                [3.0, 3.5],
                [2.5, 2.0],
                [-2.5, -2.0],
                [-3.0, -3.5],
            ]),
            Self::Check => path(&[[-3.8, -0.2], [-1.0, 2.8], [4.0, -3.0]]),
            Self::Cross => {
                line([-3.0, -3.0], [3.0, 3.0]);
                line([3.0, -3.0], [-3.0, 3.0]);
            }
            Self::Stop => {
                painter.rect_filled(
                    egui::Rect::from_center_size(center, egui::vec2(6.0, 6.0) * scale),
                    0.8,
                    ink,
                );
            }
            Self::Minus => line([-3.5, 0.0], [3.5, 0.0]),
            Self::Question => {
                path(&[
                    [-2.6, -2.8],
                    [-1.5, -4.0],
                    [1.5, -4.0],
                    [2.6, -2.8],
                    [2.6, -1.3],
                    [0.0, 0.3],
                    [0.0, 1.0],
                ]);
                painter.circle_filled(point([0.0, 3.5]), 0.9 * scale, ink);
            }
        }
    }
}
