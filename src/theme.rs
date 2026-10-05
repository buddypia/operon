use crate::prelude::*;

use crate::*;

/// The geometry half of the design system. Colour answers to `Palette` and
/// nothing else; these are the numbers everything is measured from, and they
/// are set once for the whole context so no call site can quietly disagree
/// about how tall a button is or how far apart two cards sit. `DESIGN.md`
/// carries the same table under `spacing` and `rounded`.
pub(crate) const SPACE_XS: f32 = 4.0;
pub(crate) const SPACE_SM: f32 = 8.0;
pub(crate) const SPACE_MD: f32 = 12.0;
pub(crate) const SPACE_LG: f32 = 18.0;
pub(crate) const SPACE_XL: f32 = 26.0;
/// One height for everything a person clicks. A row of controls that each pick
/// their own height is the single loudest sign that an interface was assembled
/// rather than drawn.
pub(crate) const CONTROL_HEIGHT: f32 = 28.0;
/// The dense variant, for the actions that sit inside a list row.
pub(crate) const CONTROL_HEIGHT_SMALL: f32 = 24.0;
pub(crate) const RADIUS_CONTROL: u8 = 6;
pub(crate) const RADIUS_CARD: u8 = 10;
pub(crate) const RADIUS_WINDOW: u8 = 12;
/// The height of the toolbar that doubles as the macOS title bar. The traffic
/// lights sit at the top left of it, so it cannot be shorter than they are.
pub(crate) const TOOLBAR_HEIGHT: f32 = 52.0;
/// How much of the toolbar's left edge belongs to the traffic lights.
pub(crate) const TRAFFIC_LIGHT_INSET: f32 = 78.0;

/// The margin between a page's content and the window edge.
pub(crate) const PAGE_GUTTER: f32 = SPACE_XL;

/// The colour half of `DESIGN.md`, resolved for one theme.
///
/// Every colour this app paints goes through a role on this table instead of a
/// literal at the call site. That is what makes a theme a table swap rather
/// than a sweep through twenty thousand lines, and it is why the light theme
/// can darken a status colour that the dark theme brightens without any
/// drawing code knowing which theme is on.
///
/// Roles are named for the job, never the hue. `warning` is amber on the dark
/// tables and a deep ochre on the light one; both are "the colour that says
/// this needs your attention", which is what every caller actually wants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Palette {
    /// Whether egui should build its own widget defaults from the dark base.
    pub(crate) dark: bool,

    // Surfaces, from the back of the screen forwards.
    /// Fill behind egui windows — the command palette floats on this.
    pub(crate) window: Color32,
    /// The main panel behind every page.
    pub(crate) panel: Color32,
    /// Cards and grouped sections that sit above the panel.
    pub(crate) raised: Color32,
    /// The quieter card used for a choice that is not yet chosen.
    pub(crate) card: Color32,
    /// Text edits and other sunken fields.
    pub(crate) inset: Color32,
    /// The row a person has selected in the session list.
    pub(crate) row_selected: Color32,
    /// Veil drawn over the whole window while a folder is dragged onto it.
    pub(crate) scrim: Color32,
    /// Text drawn on top of `scrim`, which is dark under every theme.
    pub(crate) on_scrim: Color32,

    // Controls.
    pub(crate) control: Color32,
    pub(crate) control_hovered: Color32,
    pub(crate) control_active: Color32,
    pub(crate) selection: Color32,
    pub(crate) selection_border: Color32,

    // Lines.
    /// Separators and the hairline around a group.
    pub(crate) border_subtle: Color32,
    /// The outline that has to be seen — a chosen card, a window edge.
    pub(crate) border: Color32,
    /// The outline of the widget being pressed right now.
    pub(crate) border_strong: Color32,

    // Text, brightest first. Secondary lines still have to be readable:
    // dimming a branch name or the last request of a conversation to egui's
    // "weak" grey hides exactly the details used to tell two sessions apart,
    // so those get `text` and a size of their own rather than `RichText::weak`.
    pub(crate) text_strong: Color32,
    pub(crate) text: Color32,
    pub(crate) text_muted: Color32,
    pub(crate) text_faint: Color32,
    /// Branch names, which are scanned rather than read and earn their own hue.
    pub(crate) branch: Color32,
    pub(crate) link: Color32,
    pub(crate) code_bg: Color32,

    // Brand and status.
    /// The brand orange as a fill — buttons, the mark behind white text.
    pub(crate) accent: Color32,
    /// The brand orange as text or an icon on a surface. On light backgrounds
    /// the fill orange has nowhere near enough contrast to be read, so the two
    /// roles part company there and stay together on the dark tables.
    pub(crate) accent_text: Color32,
    /// The softer brand tone that leads a callout or a section heading.
    pub(crate) accent_soft: Color32,
    /// Text drawn on top of `accent`.
    pub(crate) on_accent: Color32,
    pub(crate) info: Color32,
    pub(crate) success: Color32,
    /// "Finished its turn, waiting for you" — calmer than `success`, because it
    /// is not an achievement, just a state.
    pub(crate) calm: Color32,
    pub(crate) warning: Color32,
    pub(crate) danger: Color32,
    /// Sessions recovered from a terminal nobody in this app started.
    pub(crate) sand: Color32,

    // Diff. Reading a diff is the one place in this app where a colour is the
    // primary carrier of meaning, so it gets two roles per side rather than
    // one: a wash behind the whole row, which is what the eye counts at a
    // glance, and an ink for the sign in front of it, which is what a person
    // who cannot separate the two hues reads instead. Neither reuses `success`
    // or `danger` — an added line is not an achievement and a removed one is
    // not a failure, and a status hue that also meant "diff" would be a status
    // hue nobody could trust.
    /// The wash behind a line the change adds.
    pub(crate) diff_added_bg: Color32,
    /// The ink of that line's `+`, and of an "N added" count.
    pub(crate) diff_added: Color32,
    /// The same pair for a line the change removes.
    pub(crate) diff_removed_bg: Color32,
    pub(crate) diff_removed: Color32,
    // A third role per side, for the words inside a row that actually moved.
    // The wash answers "how much did this agent touch"; this answers "what did
    // it change", and it is a stronger band of the same hue because it is a few
    // characters wide rather than a whole row. It is only ever drawn inside its
    // own wash, so it is read against that and not against the page.
    /// The band behind the words a rewritten line added.
    pub(crate) diff_added_emphasis: Color32,
    /// The band behind the words a rewritten line removed.
    pub(crate) diff_removed_emphasis: Color32,

    // Vendor marks. Each CLI's own colour, pulled towards the surface it is
    // drawn on so the name beside the mark stays readable in both themes.
    pub(crate) agent_codex: Color32,
    pub(crate) agent_claude: Color32,
    pub(crate) agent_antigravity: Color32,
    pub(crate) agent_unknown: Color32,

    // Terminal.
    pub(crate) terminal_bg: Color32,
    pub(crate) terminal_border: Color32,
    /// What a terminal writes when it has not asked for a colour, which is
    /// most of what a terminal writes. Separate from ANSI 7 on purpose: a CLI
    /// that says `SGR 37` is asking for the colour *named* white, while
    /// uncoloured output is asking for "your normal ink", and on paper those
    /// two are not the same request.
    pub(crate) terminal_fg: Color32,
    /// ANSI 30-37 in the standard order: black, red, green, yellow, blue,
    /// magenta, cyan, white. Index 7 doubles as the default foreground.
    pub(crate) ansi: [Color32; 8],
    /// ANSI 90-97, same order.
    pub(crate) ansi_bright: [Color32; 8],
}

/// `Dark` — the default. Ink-black surfaces with a single warm accent.
pub(crate) const DARK_PALETTE: Palette = Palette {
    dark: true,
    window: Color32::from_rgb(27, 27, 27),
    panel: Color32::from_rgb(27, 27, 27),
    raised: Color32::from_rgb(25, 25, 25),
    card: Color32::from_rgb(30, 30, 30),
    inset: Color32::from_rgb(10, 10, 10),
    row_selected: Color32::from_rgb(54, 39, 29),
    scrim: Color32::from_black_alpha(220),
    on_scrim: Color32::from_rgb(255, 190, 90),

    control: Color32::from_rgb(60, 60, 60),
    control_hovered: Color32::from_rgb(70, 70, 70),
    control_active: Color32::from_rgb(55, 55, 55),
    selection: Color32::from_rgb(0, 92, 128),
    selection_border: Color32::from_rgb(192, 222, 255),

    border_subtle: Color32::from_rgb(60, 60, 60),
    border: Color32::from_rgb(118, 118, 118),
    border_strong: Color32::from_rgb(255, 255, 255),

    text_strong: Color32::from_rgb(242, 244, 247),
    text: Color32::from_rgb(201, 205, 211),
    text_muted: Color32::from_rgb(154, 160, 168),
    text_faint: Color32::from_rgb(147, 153, 162),
    branch: Color32::from_rgb(150, 190, 255),
    link: Color32::from_rgb(90, 170, 255),
    code_bg: Color32::from_rgb(64, 64, 64),

    accent: Color32::from_rgb(255, 148, 56),
    accent_text: Color32::from_rgb(255, 148, 56),
    accent_soft: Color32::from_rgb(255, 190, 90),
    on_accent: Color32::from_rgb(26, 16, 4),
    info: Color32::from_rgb(120, 175, 255),
    success: Color32::from_rgb(55, 210, 161),
    calm: Color32::from_rgb(126, 196, 176),
    warning: Color32::from_rgb(246, 187, 55),
    danger: Color32::from_rgb(238, 102, 102),
    sand: Color32::from_rgb(180, 150, 120),

    diff_added_bg: Color32::from_rgb(16, 40, 28),
    diff_added: Color32::from_rgb(90, 215, 155),
    diff_removed_bg: Color32::from_rgb(46, 22, 24),
    diff_removed: Color32::from_rgb(255, 138, 133),
    diff_added_emphasis: Color32::from_rgb(26, 81, 56),
    diff_removed_emphasis: Color32::from_rgb(114, 45, 49),

    agent_codex: Color32::from_rgb(48, 184, 167),
    agent_claude: Color32::from_rgb(222, 119, 72),
    agent_antigravity: Color32::from_rgb(105, 150, 244),
    agent_unknown: Color32::from_rgb(146, 146, 158),

    terminal_bg: Color32::from_rgb(10, 12, 16),
    terminal_border: Color32::from_rgb(45, 49, 58),
    terminal_fg: Color32::from_rgb(206, 212, 222),
    ansi: [
        Color32::from_rgb(123, 132, 148),
        Color32::from_rgb(238, 102, 102),
        Color32::from_rgb(55, 210, 161),
        Color32::from_rgb(246, 187, 55),
        Color32::from_rgb(120, 175, 255),
        Color32::from_rgb(193, 132, 252),
        Color32::from_rgb(58, 205, 222),
        Color32::from_rgb(206, 212, 222),
    ],
    ansi_bright: [
        Color32::from_rgb(166, 174, 188),
        Color32::from_rgb(255, 128, 128),
        Color32::from_rgb(104, 232, 177),
        Color32::from_rgb(255, 210, 97),
        Color32::from_rgb(151, 196, 255),
        Color32::from_rgb(213, 168, 255),
        Color32::from_rgb(100, 225, 239),
        Color32::from_rgb(245, 247, 250),
    ],
};

/// `Light` — warm paper rather than white, so a full-screen window at midday
/// does not glare. Every status hue is re-mixed dark enough to be read as text
/// on paper; the same hue at the dark theme's brightness would vanish.
pub(crate) const LIGHT_PALETTE: Palette = Palette {
    dark: false,
    window: Color32::from_rgb(250, 249, 247),
    panel: Color32::from_rgb(247, 245, 242),
    raised: Color32::from_rgb(255, 255, 255),
    card: Color32::from_rgb(239, 236, 230),
    inset: Color32::from_rgb(255, 255, 255),
    row_selected: Color32::from_rgb(251, 230, 210),
    scrim: Color32::from_black_alpha(200),
    on_scrim: Color32::from_rgb(255, 205, 130),

    control: Color32::from_rgb(230, 226, 219),
    control_hovered: Color32::from_rgb(220, 215, 207),
    control_active: Color32::from_rgb(207, 201, 192),
    selection: Color32::from_rgb(180, 214, 250),
    selection_border: Color32::from_rgb(0, 83, 125),

    border_subtle: Color32::from_rgb(220, 215, 207),
    border: Color32::from_rgb(135, 127, 114),
    border_strong: Color32::from_rgb(26, 28, 30),

    text_strong: Color32::from_rgb(15, 17, 19),
    text: Color32::from_rgb(36, 40, 44),
    text_muted: Color32::from_rgb(86, 92, 99),
    text_faint: Color32::from_rgb(95, 101, 108),
    branch: Color32::from_rgb(29, 78, 152),
    link: Color32::from_rgb(31, 95, 191),
    code_bg: Color32::from_rgb(236, 232, 225),

    accent: Color32::from_rgb(225, 112, 26),
    accent_text: Color32::from_rgb(169, 73, 15),
    accent_soft: Color32::from_rgb(138, 83, 16),
    on_accent: Color32::from_rgb(255, 252, 248),
    info: Color32::from_rgb(31, 95, 191),
    success: Color32::from_rgb(11, 112, 79),
    calm: Color32::from_rgb(46, 106, 90),
    warning: Color32::from_rgb(133, 92, 0),
    danger: Color32::from_rgb(179, 38, 30),
    sand: Color32::from_rgb(122, 90, 52),

    diff_added_bg: Color32::from_rgb(221, 243, 228),
    diff_added: Color32::from_rgb(11, 110, 63),
    diff_removed_bg: Color32::from_rgb(251, 227, 226),
    diff_removed: Color32::from_rgb(166, 32, 25),
    diff_added_emphasis: Color32::from_rgb(133, 207, 157),
    diff_removed_emphasis: Color32::from_rgb(243, 165, 160),

    agent_codex: Color32::from_rgb(13, 110, 100),
    agent_claude: Color32::from_rgb(166, 68, 27),
    agent_antigravity: Color32::from_rgb(43, 78, 186),
    agent_unknown: Color32::from_rgb(84, 84, 96),

    terminal_bg: Color32::from_rgb(252, 251, 248),
    terminal_border: Color32::from_rgb(213, 207, 198),
    terminal_fg: Color32::from_rgb(36, 40, 44),
    ansi: [
        Color32::from_rgb(43, 48, 54),
        Color32::from_rgb(179, 38, 30),
        Color32::from_rgb(11, 112, 79),
        Color32::from_rgb(133, 92, 0),
        Color32::from_rgb(31, 95, 191),
        Color32::from_rgb(120, 61, 158),
        Color32::from_rgb(13, 105, 117),
        Color32::from_rgb(86, 92, 99),
    ],
    ansi_bright: [
        Color32::from_rgb(74, 80, 87),
        Color32::from_rgb(155, 30, 24),
        Color32::from_rgb(9, 95, 67),
        Color32::from_rgb(113, 78, 0),
        Color32::from_rgb(25, 80, 163),
        Color32::from_rgb(102, 50, 136),
        Color32::from_rgb(11, 90, 100),
        Color32::from_rgb(26, 28, 30),
    ],
};

/// `HighContrast` — the dark table pushed apart. Surfaces go further towards
/// black, text and every status hue go further towards white, and the borders
/// stop being hairlines you have to hunt for.
pub(crate) const HIGH_CONTRAST_PALETTE: Palette = Palette {
    dark: true,
    window: Color32::from_rgb(12, 13, 15),
    panel: Color32::from_rgb(20, 20, 22),
    raised: Color32::from_rgb(28, 29, 33),
    card: Color32::from_rgb(34, 35, 40),
    inset: Color32::from_rgb(4, 5, 7),
    row_selected: Color32::from_rgb(74, 52, 35),
    scrim: Color32::from_black_alpha(235),
    on_scrim: Color32::from_rgb(255, 208, 138),

    control: Color32::from_rgb(51, 54, 61),
    control_hovered: Color32::from_rgb(62, 66, 75),
    control_active: Color32::from_rgb(42, 45, 52),
    selection: Color32::from_rgb(10, 110, 150),
    selection_border: Color32::from_rgb(220, 238, 255),

    border_subtle: Color32::from_rgb(74, 79, 90),
    border: Color32::from_rgb(122, 130, 144),
    border_strong: Color32::from_rgb(255, 255, 255),

    text_strong: Color32::from_rgb(255, 255, 255),
    text: Color32::from_rgb(245, 247, 250),
    text_muted: Color32::from_rgb(214, 218, 224),
    text_faint: Color32::from_rgb(169, 175, 184),
    branch: Color32::from_rgb(180, 210, 255),
    link: Color32::from_rgb(140, 196, 255),
    code_bg: Color32::from_rgb(42, 46, 54),

    accent: Color32::from_rgb(255, 164, 92),
    accent_text: Color32::from_rgb(255, 178, 110),
    accent_soft: Color32::from_rgb(255, 208, 138),
    on_accent: Color32::from_rgb(16, 10, 2),
    info: Color32::from_rgb(166, 200, 255),
    success: Color32::from_rgb(91, 232, 190),
    calm: Color32::from_rgb(160, 216, 200),
    warning: Color32::from_rgb(255, 211, 92),
    danger: Color32::from_rgb(255, 138, 138),
    sand: Color32::from_rgb(214, 186, 152),

    diff_added_bg: Color32::from_rgb(11, 51, 36),
    diff_added: Color32::from_rgb(126, 238, 188),
    diff_removed_bg: Color32::from_rgb(58, 26, 28),
    diff_removed: Color32::from_rgb(255, 168, 164),
    diff_added_emphasis: Color32::from_rgb(19, 110, 74),
    diff_removed_emphasis: Color32::from_rgb(125, 50, 54),

    agent_codex: Color32::from_rgb(94, 220, 204),
    agent_claude: Color32::from_rgb(255, 158, 110),
    agent_antigravity: Color32::from_rgb(150, 184, 255),
    agent_unknown: Color32::from_rgb(190, 190, 202),

    terminal_bg: Color32::from_rgb(4, 5, 7),
    terminal_border: Color32::from_rgb(96, 104, 118),
    terminal_fg: Color32::from_rgb(245, 247, 250),
    ansi: [
        Color32::from_rgb(126, 134, 150),
        Color32::from_rgb(255, 138, 138),
        Color32::from_rgb(91, 232, 190),
        Color32::from_rgb(255, 211, 92),
        Color32::from_rgb(166, 200, 255),
        Color32::from_rgb(214, 168, 255),
        Color32::from_rgb(120, 226, 240),
        Color32::from_rgb(245, 247, 250),
    ],
    ansi_bright: [
        Color32::from_rgb(168, 176, 190),
        Color32::from_rgb(255, 180, 180),
        Color32::from_rgb(150, 245, 214),
        Color32::from_rgb(255, 228, 148),
        Color32::from_rgb(200, 222, 255),
        Color32::from_rgb(232, 202, 255),
        Color32::from_rgb(170, 240, 250),
        Color32::from_rgb(255, 255, 255),
    ],
};

/// Relative luminance per WCAG 2.1, which is what both `readable_text_on` and
/// the contrast tests are built out of.
///
/// `Color32` is premultiplied, so this is only meaningful for an opaque
/// colour: a translucent one would be measured as the fraction of itself it
/// contributes rather than as the colour a person ends up seeing over a
/// surface. Composite it first if you need to measure a veil.
pub(crate) fn relative_luminance(color: Color32) -> f32 {
    debug_assert!(
        color.a() == 255,
        "contrast is only defined against an opaque colour"
    );
    let channel = |value: u8| {
        let value = f32::from(value) / 255.0;
        if value <= 0.039_28 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(color.r()) + 0.7152 * channel(color.g()) + 0.0722 * channel(color.b())
}

/// WCAG 2.1 contrast ratio, from 1.0 (identical) to 21.0 (black on white).
pub(crate) fn contrast_ratio(foreground: Color32, background: Color32) -> f32 {
    let (first, second) = (
        relative_luminance(foreground),
        relative_luminance(background),
    );
    let (lighter, darker) = if first >= second {
        (first, second)
    } else {
        (second, first)
    };
    (lighter + 0.05) / (darker + 0.05)
}

/// Ink for a button whose fill is a vendor's colour rather than the theme's.
/// A Codex teal and a Claude terracotta cannot share one label colour, and
/// neither can the same teal across a light and a dark theme.
pub(crate) fn readable_text_on(fill: Color32, palette: &Palette) -> Color32 {
    if contrast_ratio(palette.on_accent, fill) >= contrast_ratio(palette.text_strong, fill) {
        palette.on_accent
    } else {
        palette.text_strong
    }
}

impl Palette {
    /// Hands egui the same table the drawing code reads from, so a plain
    /// `ui.button` and a hand-coloured `RichText` cannot disagree about what
    /// the current theme looks like.
    pub(crate) fn visuals(&self) -> egui::Visuals {
        let mut visuals = if self.dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        visuals.window_fill = self.window;
        visuals.panel_fill = self.panel;
        visuals.extreme_bg_color = self.inset;
        visuals.code_bg_color = self.code_bg;
        visuals.hyperlink_color = self.link;
        visuals.warn_fg_color = self.warning;
        visuals.error_fg_color = self.danger;
        visuals.window_stroke = egui::Stroke::new(1.0, self.border_subtle);
        visuals.window_corner_radius = egui::CornerRadius::same(RADIUS_WINDOW);
        visuals.menu_corner_radius = egui::CornerRadius::same(RADIUS_CONTROL);
        visuals.selection.bg_fill = self.selection;
        visuals.selection.stroke = egui::Stroke::new(1.0, self.selection_border);
        visuals.text_cursor.stroke = egui::Stroke::new(2.0, self.accent_text);
        visuals.faint_bg_color = self.card;

        let widgets = &mut visuals.widgets;
        widgets.noninteractive.bg_fill = self.panel;
        widgets.noninteractive.weak_bg_fill = self.panel;
        widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, self.border_subtle);
        // egui reads this one as "the colour of an ordinary label", so it sets
        // the reading contrast of most of the app.
        widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, self.text);
        // A control at rest is a hairline, not a slab. `weak_bg_fill` is what a
        // button paints and it is allowed to be transparent; `bg_fill` is what
        // a checkbox or slider paints and is not, so the two part ways here.
        // Twenty filled grey rectangles on one page read as twenty unrelated
        // objects — the fill is what marks the one under the pointer, which is
        // the only one that needs marking.
        widgets.inactive.bg_fill = self.control;
        widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
        widgets.inactive.bg_stroke = egui::Stroke::new(1.0, self.border_subtle);
        widgets.inactive.fg_stroke = egui::Stroke::new(1.0, self.text);
        widgets.hovered.bg_fill = self.control_hovered;
        widgets.hovered.weak_bg_fill = self.control;
        widgets.hovered.bg_stroke = egui::Stroke::new(1.0, self.border);
        widgets.hovered.fg_stroke = egui::Stroke::new(1.0, self.text_strong);
        widgets.active.bg_fill = self.control_active;
        widgets.active.weak_bg_fill = self.control_hovered;
        widgets.active.bg_stroke = egui::Stroke::new(1.0, self.border_strong);
        widgets.active.fg_stroke = egui::Stroke::new(1.0, self.text_strong);
        widgets.open.bg_fill = self.control;
        widgets.open.weak_bg_fill = self.control;
        widgets.open.bg_stroke = egui::Stroke::new(1.0, self.border);
        widgets.open.fg_stroke = egui::Stroke::new(1.0, self.text_strong);
        // egui thickens `fg_stroke` on hover by default, which makes a label
        // appear to gain weight under the pointer. Hierarchy is carried by the
        // ink, not by a stroke the text face never asked for.
        for widget in [
            &mut widgets.noninteractive,
            &mut widgets.inactive,
            &mut widgets.hovered,
            &mut widgets.active,
            &mut widgets.open,
        ] {
            widget.corner_radius = egui::CornerRadius::same(RADIUS_CONTROL);
            widget.expansion = 0.0;
        }
        visuals
    }

    /// Every role on this table under the name `DESIGN.md` documents it by.
    /// This is the list the drift test walks, which is what keeps the document
    /// a source of truth rather than a description of an older version.
    #[cfg(test)]
    pub(crate) fn tokens(&self) -> Vec<(String, Color32)> {
        let ansi_names = [
            "black", "red", "green", "yellow", "blue", "magenta", "cyan", "white",
        ];
        let tokens = vec![
            ("window", self.window),
            ("panel", self.panel),
            ("raised", self.raised),
            ("card", self.card),
            ("inset", self.inset),
            ("row-selected", self.row_selected),
            ("scrim", self.scrim),
            ("on-scrim", self.on_scrim),
            ("control", self.control),
            ("control-hovered", self.control_hovered),
            ("control-active", self.control_active),
            ("selection", self.selection),
            ("selection-border", self.selection_border),
            ("border-subtle", self.border_subtle),
            ("border", self.border),
            ("border-strong", self.border_strong),
            ("text-strong", self.text_strong),
            ("text", self.text),
            ("text-muted", self.text_muted),
            ("text-faint", self.text_faint),
            ("branch", self.branch),
            ("link", self.link),
            ("code-bg", self.code_bg),
            ("accent", self.accent),
            ("accent-text", self.accent_text),
            ("accent-soft", self.accent_soft),
            ("on-accent", self.on_accent),
            ("info", self.info),
            ("success", self.success),
            ("calm", self.calm),
            ("warning", self.warning),
            ("danger", self.danger),
            ("sand", self.sand),
            ("diff-added-bg", self.diff_added_bg),
            ("diff-added", self.diff_added),
            ("diff-removed-bg", self.diff_removed_bg),
            ("diff-removed", self.diff_removed),
            ("diff-added-emphasis", self.diff_added_emphasis),
            ("diff-removed-emphasis", self.diff_removed_emphasis),
            ("agent-codex", self.agent_codex),
            ("agent-claude", self.agent_claude),
            ("agent-antigravity", self.agent_antigravity),
            ("agent-unknown", self.agent_unknown),
            ("terminal-bg", self.terminal_bg),
            ("terminal-border", self.terminal_border),
            ("terminal-fg", self.terminal_fg),
        ];
        let mut tokens: Vec<(String, Color32)> = tokens
            .into_iter()
            .map(|(name, colour)| (name.to_owned(), colour))
            .collect();
        for (index, name) in ansi_names.iter().enumerate() {
            tokens.push((format!("ansi-{name}"), self.ansi[index]));
            tokens.push((format!("ansi-bright-{name}"), self.ansi_bright[index]));
        }
        tokens
    }

    /// The vendor mark's colour for `agent`, already pulled towards the
    /// current surface.
    pub(crate) fn agent_accent(&self, agent: &str) -> Color32 {
        match agent {
            "codex" => self.agent_codex,
            "claude" => self.agent_claude,
            "gemini" => self.agent_antigravity,
            _ => self.agent_unknown,
        }
    }

    /// The colour a status is drawn in, resolved from the role the status
    /// carries. `session_status_view` names the role and stays theme-free; the
    /// table decides what that role looks like today.
    pub(crate) fn status(&self, tone: StatusTone) -> Color32 {
        match tone {
            StatusTone::Info => self.info,
            StatusTone::Running => self.success,
            StatusTone::Idle => self.calm,
            StatusTone::Attention => self.warning,
            StatusTone::Failed => self.danger,
            StatusTone::Neutral => self.text_faint,
            StatusTone::Recovered => self.sand,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub(crate) enum AppTheme {
    #[default]
    Dark,
    Light,
    HighContrast,
}

impl AppTheme {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Dark => tr("ダーク"),
            Self::Light => tr("ライト"),
            Self::HighContrast => tr("ハイコントラスト（ダーク）"),
        }
    }

    pub(crate) fn detail(self) -> &'static str {
        match self {
            Self::Dark => tr("既定。暗い画面に暖色のアクセント"),
            Self::Light => tr("紙のような明るい配色"),
            Self::HighContrast => tr("ダークの明暗差を広げた配色"),
        }
    }

    pub(crate) fn all() -> [Self; 3] {
        [Self::Dark, Self::Light, Self::HighContrast]
    }

    pub(crate) fn palette(self) -> &'static Palette {
        match self {
            Self::Dark => &DARK_PALETTE,
            Self::Light => &LIGHT_PALETTE,
            Self::HighContrast => &HIGH_CONTRAST_PALETTE,
        }
    }

    pub(crate) fn visuals(self) -> egui::Visuals {
        let mut visuals = self.palette().visuals();
        if self == Self::HighContrast {
            // The one theme that refuses to let a caller dim anything: an
            // uncoloured label is forced to the brightest ink on the table.
            visuals.override_text_color = Some(HIGH_CONTRAST_PALETTE.text);
        }
        visuals
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub(crate) enum TerminalFont {
    #[default]
    #[serde(alias = "NotoSansMonoCjkJp")]
    SarasaMonoJ,
    #[serde(alias = "NotoSansMonoCjkKr")]
    SarasaMonoK,
    SarasaTermJ,
    SarasaFixedJ,
}

impl TerminalFont {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::SarasaMonoJ => "Sarasa Mono J",
            Self::SarasaMonoK => "Sarasa Mono K",
            Self::SarasaTermJ => "Sarasa Term J",
            Self::SarasaFixedJ => "Sarasa Fixed J",
        }
    }

    pub(crate) fn detail(self) -> &'static str {
        match self {
            Self::SarasaMonoJ => tr("日本語優先"),
            Self::SarasaMonoK => tr("韓国語優先"),
            Self::SarasaTermJ => tr("日本語優先・ターミナル向け（合字あり）"),
            Self::SarasaFixedJ => tr("日本語優先・プログラミング向け（合字なし）"),
        }
    }

    pub(crate) fn font_name(self) -> &'static str {
        match self {
            Self::SarasaMonoJ => "sarasa-mono-j",
            Self::SarasaMonoK => "sarasa-mono-k",
            Self::SarasaTermJ => "sarasa-term-j",
            Self::SarasaFixedJ => "sarasa-fixed-j",
        }
    }

    pub(crate) fn fallback_font_name(self) -> &'static str {
        match self {
            Self::SarasaMonoJ => "sarasa-mono-k",
            Self::SarasaMonoK => "sarasa-mono-j",
            Self::SarasaTermJ | Self::SarasaFixedJ => "sarasa-mono-j",
        }
    }
}
