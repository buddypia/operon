use crate::prelude::*;
use crate::*;

#[derive(Clone, Copy, Default)]
pub(crate) struct TerminalTextStyle {
    pub(crate) foreground: Option<Color32>,
    pub(crate) bright: bool,
}

/// The 16 ANSI colours in their standard order — black, red, green, yellow,
/// blue, magenta, cyan, white — mixed for the theme that is on. A CLI paints
/// `SGR 31` because it means "red", so the table has to answer with the theme's
/// red rather than with a fixed one that a light terminal cannot show.
pub(crate) fn terminal_ansi_color(index: u8, bright: bool, palette: &Palette) -> Color32 {
    let table = if bright {
        &palette.ansi_bright
    } else {
        &palette.ansi
    };
    table[usize::from(index.min(7))]
}

pub(crate) fn terminal_ansi_256_color(index: u8, palette: &Palette) -> Color32 {
    match index {
        0..=7 => terminal_ansi_color(index, false, palette),
        8..=15 => terminal_ansi_color(index - 8, true, palette),
        16..=231 => {
            let value = index - 16;
            let red = value / 36;
            let green = (value / 6) % 6;
            let blue = value % 6;
            let level = |component: u8| {
                if component == 0 {
                    0
                } else {
                    55 + component * 40
                }
            };
            Color32::from_rgb(level(red), level(green), level(blue))
        }
        _ => {
            // The 24-step grey ramp is the one part of the 256-colour space
            // that CLIs use for a *relationship* rather than a colour: a high
            // step is "dimmer than my text", which every agent uses for the
            // grey it writes hints and diffs in. Those steps were picked
            // against a black terminal, so on paper they are white on white.
            // Mirroring the ramp keeps "dim" meaning dim. The 216-colour cube
            // above is left exactly as asked, because mirroring a hue would
            // answer a request for red with cyan.
            let gray = 8 + (index - 232) * 10;
            Color32::from_gray(if palette.dark { gray } else { 255 - gray })
        }
    }
}

pub(crate) fn apply_terminal_sgr(
    parameters: &str,
    style: &mut TerminalTextStyle,
    palette: &Palette,
) {
    let parameters = if parameters.is_empty() {
        vec![0]
    } else {
        parameters
            .split(';')
            .filter_map(|value| value.parse::<u16>().ok())
            .collect::<Vec<_>>()
    };
    let mut index = 0;
    while index < parameters.len() {
        match parameters[index] {
            0 => *style = TerminalTextStyle::default(),
            1 => style.bright = true,
            22 => style.bright = false,
            30..=37 => {
                style.foreground = Some(terminal_ansi_color(
                    (parameters[index] - 30) as u8,
                    false,
                    palette,
                ))
            }
            39 => style.foreground = None,
            90..=97 => {
                style.foreground = Some(terminal_ansi_color(
                    (parameters[index] - 90) as u8,
                    true,
                    palette,
                ))
            }
            38 => match parameters.get(index + 1) {
                Some(5) if parameters.get(index + 2).is_some() => {
                    style.foreground = Some(terminal_ansi_256_color(
                        parameters[index + 2] as u8,
                        palette,
                    ));
                    index += 2;
                }
                Some(2) if parameters.len() >= index + 5 => {
                    style.foreground = Some(Color32::from_rgb(
                        parameters[index + 2] as u8,
                        parameters[index + 3] as u8,
                        parameters[index + 4] as u8,
                    ));
                    index += 4;
                }
                _ => {}
            },
            _ => {}
        }
        index += 1;
    }
}

pub(crate) fn append_terminal_text(
    job: &mut LayoutJob,
    text: &str,
    style: TerminalTextStyle,
    palette: &Palette,
) {
    if text.is_empty() {
        return;
    }
    let color = style.foreground.unwrap_or(if style.bright {
        palette.ansi_bright[7]
    } else {
        palette.terminal_fg
    });
    job.append(
        text,
        0.0,
        TextFormat {
            font_id: FontId::monospace(MONOSPACE_SIZE),
            color,
            ..Default::default()
        },
    );
}

/// Lay out one captured line, returning the SGR state the next line starts in.
/// A colour a CLI opens on one line stays open until it resets it, so the style
/// has to survive the line break the same way it does in a real terminal.
pub(crate) fn append_terminal_line(
    job: &mut LayoutJob,
    line: &str,
    mut style: TerminalTextStyle,
    palette: &Palette,
) -> TerminalTextStyle {
    let bytes = line.as_bytes();
    let mut text_start = 0;
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == 0x1b && bytes.get(index + 1) == Some(&b'[') {
            let sequence_start = index + 2;
            let Some(offset) = bytes[sequence_start..]
                .iter()
                .position(|byte| (0x40..=0x7e).contains(byte))
            else {
                break;
            };
            let sequence_end = sequence_start + offset;
            append_terminal_text(job, &line[text_start..index], style, palette);
            if bytes[sequence_end] == b'm' {
                apply_terminal_sgr(&line[sequence_start..sequence_end], &mut style, palette);
            }
            index = sequence_end + 1;
            text_start = index;
            continue;
        }
        let width = line[index..]
            .chars()
            .next()
            .map(char::len_utf8)
            .unwrap_or(1);
        index += width;
    }
    append_terminal_text(job, &line[text_start..], style, palette);
    style
}

/// One layout job per captured line. The terminal pane draws thousands of
/// lines, and egui lays out a whole job even where it is scrolled out of sight,
/// so the buffer is split at the line the viewport can skip rather than handed
/// over as one long block of text.
pub(crate) fn terminal_line_layouts(output: &str, palette: &Palette) -> Vec<LayoutJob> {
    let output = output.replace('\r', "");
    let mut style = TerminalTextStyle::default();
    output
        .split('\n')
        .map(|line| {
            let mut job = LayoutJob::default();
            style = append_terminal_line(&mut job, line, style, palette);
            job
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TerminalPreeditUpdate {
    Unchanged,
    Set(String),
    Clear,
}

pub(crate) fn terminal_input_events(
    ctx: &egui::Context,
    is_preediting: bool,
) -> (Vec<TerminalInput>, TerminalPreeditUpdate) {
    ctx.input(|input| {
        // One Enter belongs to each conversion the IME confirmed this frame,
        // and no more than one. A frame can hold both the Enter that confirmed
        // a conversion and the Enter that sends the line — pressing Enter twice
        // is how a sentence in Japanese ends — and a flag over the whole frame
        // swallowed the second along with the first.
        //
        // Counting rather than matching on position is what makes this hold
        // whichever way round the platform delivers the pair: the key before
        // the commit, or after it.
        let mut enters_the_ime_owns = input
            .events
            .iter()
            .filter(
                |e| matches!(e, egui::Event::Ime(egui::ImeEvent::Commit(text)) if !text.is_empty()),
            )
            .count();

        let mut preedit_update = TerminalPreeditUpdate::Unchanged;
        for event in &input.events {
            match event {
                egui::Event::Ime(egui::ImeEvent::Preedit(text)) => {
                    if text.is_empty() {
                        preedit_update = TerminalPreeditUpdate::Clear;
                    } else {
                        preedit_update = TerminalPreeditUpdate::Set(text.clone());
                    }
                }
                egui::Event::Ime(egui::ImeEvent::Commit(_))
                | egui::Event::Ime(egui::ImeEvent::Disabled) => {
                    preedit_update = TerminalPreeditUpdate::Clear;
                }
                _ => {}
            }
        }

        let in_preedit = match &preedit_update {
            TerminalPreeditUpdate::Set(_) => true,
            TerminalPreeditUpdate::Clear => false,
            TerminalPreeditUpdate::Unchanged => is_preediting,
        };

        let inputs = input
            .events
            .iter()
            .filter_map(|event| match event {
                egui::Event::Text(text) | egui::Event::Paste(text) if !text.is_empty() => {
                    Some(TerminalInput::Text(text.clone()))
                }
                egui::Event::Ime(egui::ImeEvent::Commit(text)) if !text.is_empty() => {
                    Some(TerminalInput::Text(text.clone()))
                }
                egui::Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } => {
                    if enters_the_ime_owns > 0 && *key == egui::Key::Enter {
                        enters_the_ime_owns -= 1;
                        return None;
                    }
                    if in_preedit {
                        match key {
                            egui::Key::Backspace
                            | egui::Key::ArrowDown
                            | egui::Key::ArrowUp
                            | egui::Key::ArrowLeft
                            | egui::Key::ArrowRight
                            | egui::Key::Escape
                            | egui::Key::Enter => return None,
                            _ => {}
                        }
                    }
                    terminal_key_binding(*key, *modifiers).map(TerminalInput::Key)
                }
                _ => None,
            })
            .collect();

        (inputs, preedit_update)
    })
}

pub(crate) fn terminal_key_binding(key: egui::Key, modifiers: egui::Modifiers) -> Option<String> {
    // Keep macOS application shortcuts under the app's control. Ctrl remains
    // available for terminal conventions such as Ctrl+C and Ctrl+R.
    if modifiers.command {
        return None;
    }
    let named = match key {
        egui::Key::ArrowDown => Some("Down"),
        egui::Key::ArrowLeft => Some("Left"),
        egui::Key::ArrowRight => Some("Right"),
        egui::Key::ArrowUp => Some("Up"),
        egui::Key::Escape => Some("Escape"),
        egui::Key::Tab if modifiers.shift => Some("BTab"),
        egui::Key::Tab => Some("Tab"),
        egui::Key::Backspace => Some("BSpace"),
        egui::Key::Enter => Some("Enter"),
        egui::Key::Insert => Some("IC"),
        egui::Key::Delete => Some("DC"),
        egui::Key::Home => Some("Home"),
        egui::Key::End => Some("End"),
        egui::Key::PageUp => Some("PPage"),
        egui::Key::PageDown => Some("NPage"),
        egui::Key::F1
        | egui::Key::F2
        | egui::Key::F3
        | egui::Key::F4
        | egui::Key::F5
        | egui::Key::F6
        | egui::Key::F7
        | egui::Key::F8
        | egui::Key::F9
        | egui::Key::F10
        | egui::Key::F11
        | egui::Key::F12 => return Some(format!("{key:?}")),
        _ => None,
    };
    if modifiers.ctrl {
        return terminal_control_key(key)
            .or(named)
            .map(|key| format!("C-{key}"));
    }
    if modifiers.alt {
        return named
            .map(|key| format!("M-{key}"))
            .or_else(|| terminal_control_key(key).map(|key| format!("M-{key}")));
    }
    named.map(str::to_owned)
}

pub(crate) fn terminal_control_key(key: egui::Key) -> Option<&'static str> {
    match key {
        egui::Key::A => Some("a"),
        egui::Key::B => Some("b"),
        egui::Key::C => Some("c"),
        egui::Key::D => Some("d"),
        egui::Key::E => Some("e"),
        egui::Key::F => Some("f"),
        egui::Key::G => Some("g"),
        egui::Key::H => Some("h"),
        egui::Key::I => Some("i"),
        egui::Key::J => Some("j"),
        egui::Key::K => Some("k"),
        egui::Key::L => Some("l"),
        egui::Key::M => Some("m"),
        egui::Key::N => Some("n"),
        egui::Key::O => Some("o"),
        egui::Key::P => Some("p"),
        egui::Key::Q => Some("q"),
        egui::Key::R => Some("r"),
        egui::Key::S => Some("s"),
        egui::Key::T => Some("t"),
        egui::Key::U => Some("u"),
        egui::Key::V => Some("v"),
        egui::Key::W => Some("w"),
        egui::Key::X => Some("x"),
        egui::Key::Y => Some("y"),
        egui::Key::Z => Some("z"),
        egui::Key::Num0 => Some("0"),
        egui::Key::Num1 => Some("1"),
        egui::Key::Num2 => Some("2"),
        egui::Key::Num3 => Some("3"),
        egui::Key::Num4 => Some("4"),
        egui::Key::Num5 => Some("5"),
        egui::Key::Num6 => Some("6"),
        egui::Key::Num7 => Some("7"),
        egui::Key::Num8 => Some("8"),
        egui::Key::Num9 => Some("9"),
        egui::Key::Space => Some("Space"),
        egui::Key::Backspace => Some("BSpace"),
        _ => None,
    }
}

/// One terminal cell in the monospace face the person picked in Settings.
/// Every Sarasa variant keeps an exact 1:2 advance ratio between ASCII and CJK
/// characters, so a measured ASCII advance is the cell width and a CJK
/// character occupies precisely two columns.
/// One character cell of the monospaced face: the width of a space and the
/// height of a row. The terminal sizes its grid with it and the editor and the
/// diff pane place their gutters with it.
pub(crate) fn monospace_cell_size(ui: &egui::Ui) -> egui::Vec2 {
    let font = FontId::monospace(MONOSPACE_SIZE);
    ui.fonts(|fonts| egui::vec2(fonts.glyph_width(&font, ' '), fonts.row_height(&font)))
}

pub(crate) fn terminal_grid_size(viewport: egui::Vec2, cell: egui::Vec2) -> (u16, u16) {
    // The pane draws each captured line as one row without re-wrapping it, so
    // the columns tmux wraps at have to be columns this viewport can show.
    // A small safety margin prevents a wrapped right-most glyph from causing
    // TUI status bars to jump between two rows while the window is resized.
    let columns = ((viewport.x - 20.0) / cell.x).floor().clamp(40.0, 400.0) as u16;
    let rows = ((viewport.y - 20.0) / cell.y).floor().clamp(12.0, 160.0) as u16;
    (columns, rows)
}

/// Draw the captured buffer one row per line, laying out only the rows on
/// screen. egui lays out a whole text block even where it is scrolled out of
/// sight, and this buffer is thousands of lines long. tmux has already wrapped
/// each line to the width it was told the pane has, so a captured line is one
/// row: re-wrapping here would split a line the terminal considers finished.
pub(crate) fn terminal_rows(
    ui: &mut egui::Ui,
    lines: &[LayoutJob],
    viewport: egui::Vec2,
    row_height: f32,
    search: TerminalSearchOverlay<'_>,
    links: &mut TerminalLinkOverlay<'_>,
) -> Option<TerminalTargetKind> {
    // Terminal rows sit directly on top of each other. Any spacing here would
    // also be counted twice: `show_rows` reserves height per row from it.
    ui.spacing_mut().item_spacing.y = 0.0;
    let cell = terminal_cell_size(ui);
    // Named, never auto-numbered. Two sibling `allocate_ui_with_layout` children
    // of one row share a `Ui::id` in egui 0.31 — `new_child` salts every
    // anonymous child with the same constant — so two unnamed scroll areas, this
    // one and the prompt timeline's, hash to the same `Id`. One id is one
    // persisted state, and the drag-to-scroll widget that state registers once
    // the content outgrows the viewport is then registered a second time, on
    // top of this pane's own click interaction. egui hands a pointer that lands
    // on a drag-only widget back as no click at all, so the pane stops taking
    // the keyboard the moment an agent fills a screen.
    let mut area = egui::ScrollArea::vertical()
        .id_salt("terminal-rows")
        .auto_shrink([false, false])
        .max_height(viewport.y)
        .stick_to_bottom(search.scroll_to.is_none());
    if let Some(line) = search.scroll_to {
        // Put the row a third of the way down rather than at the very top, so
        // what is above the match is visible too — a match with no context is
        // a match you have to scroll away from to understand.
        let offset = (line as f32 * row_height - viewport.y / 3.0).max(0.0);
        area = area.vertical_scroll_offset(offset);
    }
    let mut under_pointer = None;
    area.show_rows(ui, row_height, lines.len(), |ui, rows| {
        let first = rows.start;
        let top = ui.cursor().top();
        for (offset, line) in lines[rows.clone()].iter().enumerate() {
            // Only the rows the viewport asked for, so the cost of a search is
            // the size of the pane and never the size of the scrollback.
            for (index, found) in search.matches.iter().enumerate() {
                if found.line != first + offset {
                    continue;
                }
                let y = top + offset as f32 * row_height;
                let rect = egui::Rect::from_min_size(
                    egui::pos2(ui.min_rect().left() + found.column as f32 * cell.x, y),
                    egui::vec2(found.columns as f32 * cell.x, row_height),
                );
                ui.painter().rect_filled(
                    rect,
                    1.0,
                    if search.current == Some(index) {
                        search.palette.selection_border
                    } else {
                        search.palette.selection
                    },
                );
            }
            if line.is_empty() {
                // A blank line still owns its row, or every line below it
                // would be drawn one row too high.
                ui.add_space(row_height);
            } else {
                ui.add(egui::Label::new(line.clone()).wrap_mode(egui::TextWrapMode::Extend));
                // Only the rows on screen are read for paths and links, so the
                // cost is the pane rather than the scrollback.
                let row = egui::Rect::from_min_size(
                    egui::pos2(ui.min_rect().left(), top + offset as f32 * row_height),
                    egui::vec2(ui.min_rect().width(), row_height),
                );
                if let Some(kind) = terminal_row_links(ui, &line.text, row, cell, links) {
                    under_pointer = Some(kind);
                }
            }
        }
    });
    under_pointer
}

/// What the pane needs to know about a search in progress. Borrowed rather than
/// owned, and empty when no search is open, so a pane with no search costs
/// nothing to draw.
pub(crate) struct TerminalSearchOverlay<'a> {
    pub(crate) matches: &'a [TerminalMatch],
    pub(crate) current: Option<usize>,
    pub(crate) scroll_to: Option<usize>,
    pub(crate) palette: &'a Palette,
}

/// The buffer as the pane shows it. Colour is styling, so copying takes the
/// text the escape codes were wrapped around rather than the codes themselves.
/// One place a query occurs, in the coordinates the pane draws in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TerminalMatch {
    pub(crate) line: usize,
    /// Display cells from the left edge, not characters — see
    /// `terminal_display_columns`.
    pub(crate) column: usize,
    pub(crate) columns: usize,
}

/// How many cells a run of text occupies in this pane.
///
/// This is where the 1:2 rule the module already states stops being a comment.
/// Sarasa's CJK advance is exactly twice its ASCII advance, so a position in
/// this pane is a count of cells and never a count of characters — the same
/// assumption the cursor and the preedit are drawn on.
pub(crate) fn terminal_display_columns(text: &str) -> usize {
    text.chars().map(terminal_character_columns).sum()
}

fn terminal_character_columns(character: char) -> usize {
    let code = character as u32;
    let wide = matches!(code,
        0x1100..=0x115F
            | 0x2E80..=0x303E
            | 0x3041..=0x33FF
            | 0x3400..=0x4DBF
            | 0x4E00..=0x9FFF
            | 0xA000..=0xA4CF
            | 0xAC00..=0xD7A3
            | 0xF900..=0xFAFF
            | 0xFE30..=0xFE6F
            | 0xFF00..=0xFF60
            | 0xFFE0..=0xFFE6
            | 0x1F300..=0x1F9FF
            | 0x20000..=0x2FFFD
            | 0x30000..=0x3FFFD);
    if wide {
        2
    } else {
        1
    }
}

/// Every place `query` occurs in the pane's text, in the order they are read.
///
/// The text searched is what the pane draws, with the escape codes already
/// stripped by `terminal_line_layouts`: the colours are styling, and a query
/// that matched them would be washed at a column where nothing is drawn.
///
/// Folding is ASCII-only and therefore exactly one character in and one
/// character out, which is what keeps a match's column arithmetic true. A
/// Unicode fold can turn one character into several, and the highlight would
/// then be drawn a cell or two away from the thing it is highlighting.
pub(crate) fn terminal_search_matches(lines: &[LayoutJob], query: &str) -> Vec<TerminalMatch> {
    let needle: Vec<char> = query.chars().map(|c| c.to_ascii_lowercase()).collect();
    if needle.is_empty() {
        return Vec::new();
    }
    let mut matches = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let haystack: Vec<char> = line.text.chars().map(|c| c.to_ascii_lowercase()).collect();
        if haystack.len() < needle.len() {
            continue;
        }
        let mut at = 0;
        while at + needle.len() <= haystack.len() {
            if haystack[at..at + needle.len()] != needle[..] {
                at += 1;
                continue;
            }
            let prefix: String = line.text.chars().take(at).collect();
            let found: String = line.text.chars().skip(at).take(needle.len()).collect();
            matches.push(TerminalMatch {
                line: index,
                column: terminal_display_columns(&prefix),
                columns: terminal_display_columns(&found),
            });
            if matches.len() >= TERMINAL_SEARCH_MAX_MATCHES {
                return matches;
            }
            // Occurrences do not overlap: the next one starts after this one.
            at += needle.len();
        }
    }
    matches
}

pub(crate) fn terminal_plain_text(lines: &[LayoutJob]) -> String {
    lines
        .iter()
        .map(|line| line.text.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

/// One terminal cell in the monospace face the person picked in Settings.
/// Every Sarasa variant keeps an exact 1:2 advance ratio between ASCII and CJK
/// characters, so a measured ASCII advance is the cell width and a CJK
/// character occupies precisely two columns.
pub(crate) fn terminal_cell_size(ui: &egui::Ui) -> egui::Vec2 {
    let font = FontId::monospace(15.0);
    ui.fonts(|fonts| egui::vec2(fonts.glyph_width(&font, ' '), fonts.row_height(&font)))
}

pub(crate) fn terminal_cursor_rect(
    terminal_rect: egui::Rect,
    cursor: (u16, u16),
    cell: egui::Vec2,
) -> egui::Rect {
    let padding = 8.0;
    let min_x = terminal_rect.min.x + padding + (cursor.0 as f32 * cell.x);
    let min_y = terminal_rect.min.y + padding + (cursor.1 as f32 * cell.y);
    let max_x = (min_x + cell.x).min(terminal_rect.max.x - padding);
    let max_y = (min_y + cell.y).min(terminal_rect.max.y - padding);
    egui::Rect::from_min_max(
        egui::pos2(min_x.min(max_x), min_y.min(max_y)),
        egui::pos2(max_x.max(min_x), max_y.max(min_y)),
    )
}

pub(crate) fn draw_terminal_cursor(
    ui: &egui::Ui,
    terminal_rect: egui::Rect,
    cursor: (u16, u16),
    cell: egui::Vec2,
    palette: &Palette,
) {
    let padding = 8.0;
    let min_x = terminal_rect.min.x + padding + (cursor.0 as f32 * cell.x);
    let min_y = terminal_rect.min.y + padding + (cursor.1 as f32 * cell.y);
    if min_x >= terminal_rect.max.x - padding || min_y >= terminal_rect.max.y - padding {
        return;
    }
    let rect = egui::Rect::from_min_size(egui::pos2(min_x, min_y), cell);
    let stroke_color = palette.terminal_fg.gamma_multiply(0.6);
    ui.painter().rect_stroke(
        rect,
        0.0,
        egui::Stroke::new(1.5, stroke_color),
        egui::StrokeKind::Inside,
    );
}

pub(crate) fn draw_terminal_preedit(
    ui: &egui::Ui,
    terminal_rect: egui::Rect,
    cursor: (u16, u16),
    cell: egui::Vec2,
    preedit: &str,
    palette: &Palette,
) {
    if preedit.is_empty() {
        return;
    }
    let padding = 8.0;
    let pos = egui::pos2(
        terminal_rect.min.x + padding + (cursor.0 as f32 * cell.x),
        terminal_rect.min.y + padding + (cursor.1 as f32 * cell.y),
    );
    if pos.x >= terminal_rect.max.x - padding || pos.y >= terminal_rect.max.y - padding {
        return;
    }
    let font_id = FontId::monospace(MONOSPACE_SIZE);
    let galley = ui
        .painter()
        .layout_no_wrap(preedit.to_string(), font_id, palette.terminal_fg);
    let width = galley.size().x;
    let preedit_rect = egui::Rect::from_min_size(pos, egui::vec2(width, cell.y));

    // Under-layer highlight
    ui.painter()
        .rect_filled(preedit_rect, 1.0, palette.selection);

    // Text
    ui.painter().galley(pos, galley, palette.terminal_fg);

    // High-visibility underline
    let underline_y = pos.y + cell.y - 1.0;
    ui.painter().line_segment(
        [
            egui::pos2(pos.x, underline_y),
            egui::pos2(pos.x + width, underline_y),
        ],
        egui::Stroke::new(2.0, palette.accent_text),
    );
}

/// Something in the output that can be opened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TerminalTargetKind {
    /// A path, and the line inside it the output named. Relative paths are
    /// resolved against the session's own folder, which is what makes
    /// a stack trace's `<file>:<line>` mean anything at all.
    Path {
        path: String,
        line: Option<usize>,
    },
    Url(String),
}

/// Where it is on the row, in the cells the pane draws in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TerminalTarget {
    pub(crate) column: usize,
    pub(crate) columns: usize,
    pub(crate) kind: TerminalTargetKind,
}

/// Read one line of output for things worth opening.
///
/// Hand-written rather than a pattern crate: a dependency is a paused surface,
/// and the two shapes wanted here are narrow enough to say out loud. A URL is a
/// scheme followed by non-space; a path is a run of path characters containing
/// a dot and an extension, optionally followed by `:line` or `:line:column`.
///
/// Deliberately narrow. A false positive underlines a word that is not a file
/// and opens nothing when clicked, which is worse than missing one — so a token
/// with no extension is not a path here, even though plenty of real paths have
/// none.
pub(crate) fn terminal_targets(text: &str) -> Vec<TerminalTarget> {
    let characters: Vec<char> = text.chars().collect();
    let mut targets = Vec::new();
    let mut index = 0;
    while index < characters.len() {
        if let Some(end) = url_end(&characters, index) {
            let raw: String = characters[index..end].iter().collect();
            let trimmed = trim_trailing_punctuation(&raw);
            if !trimmed.is_empty() {
                if let Some(file_path) = trimmed.strip_prefix("file://") {
                    let token = file_path.trim_end_matches([':', '.', ',', ';']);
                    let (path, line) =
                        split_path_and_line(token).unwrap_or((token.to_owned(), None));
                    targets.push(TerminalTarget {
                        column: terminal_display_columns(
                            &characters[..index].iter().collect::<String>(),
                        ),
                        columns: terminal_display_columns(trimmed),
                        kind: TerminalTargetKind::Path { path, line },
                    });
                } else {
                    targets.push(TerminalTarget {
                        column: terminal_display_columns(
                            &characters[..index].iter().collect::<String>(),
                        ),
                        columns: terminal_display_columns(trimmed),
                        kind: TerminalTargetKind::Url(trimmed.to_owned()),
                    });
                }
            }
            index = end;
            continue;
        }
        if let Some((end, path, line)) = tool_call_target(&characters, index) {
            targets.push(TerminalTarget {
                column: terminal_display_columns(&characters[..index].iter().collect::<String>()),
                columns: terminal_display_columns(
                    &characters[index..end].iter().collect::<String>(),
                ),
                kind: TerminalTargetKind::Path { path, line },
            });
            index = end;
            continue;
        }
        if !is_path_character(characters[index])
            || (index > 0 && is_path_character(characters[index - 1]))
        {
            index += 1;
            continue;
        }
        let mut end = index;
        while end < characters.len() && is_path_character(characters[end]) {
            end += 1;
        }
        let raw: String = characters[index..end].iter().collect();
        // A panic prints the location followed by a colon, and that last
        // colon belongs to the
        // sentence rather than to the path.
        let token = raw.trim_end_matches([':', '.', ',', ';']);
        if let Some((path, line)) = split_path_and_line(token) {
            targets.push(TerminalTarget {
                column: terminal_display_columns(&characters[..index].iter().collect::<String>()),
                columns: terminal_display_columns(token),
                kind: TerminalTargetKind::Path { path, line },
            });
        }
        index = end;
    }
    targets
}

/// Detect a tool invocation like `Read(path)` or `Edit("path:line")` common in agent CLIs.
fn tool_call_target(characters: &[char], index: usize) -> Option<(usize, String, Option<usize>)> {
    if !characters[index].is_ascii_alphabetic()
        || (index > 0
            && (characters[index - 1].is_ascii_alphanumeric() || characters[index - 1] == '_'))
    {
        return None;
    }
    let mut end = index;
    while end < characters.len()
        && (characters[end].is_ascii_alphanumeric() || characters[end] == '_')
    {
        end += 1;
    }
    let tool_len = end - index;
    if tool_len == 0 || tool_len > 32 {
        return None;
    }
    if end >= characters.len() || characters[end] != '(' {
        return None;
    }
    let paren_start = end;
    let mut paren_end = paren_start + 1;
    let mut depth = 1;
    while paren_end < characters.len() {
        match characters[paren_end] {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            '\n' | '\r' => return None,
            _ => {}
        }
        paren_end += 1;
    }
    if paren_end >= characters.len() || depth != 0 {
        return None;
    }
    let inner: String = characters[paren_start + 1..paren_end].iter().collect();
    let raw_arg = inner.split(',').next()?.trim();
    let unquoted = raw_arg
        .trim_matches(|c| matches!(c, '"' | '\'' | '`'))
        .trim();
    let token = unquoted.trim_end_matches([':', '.', ',', ';']);
    let (path, line) = split_path_and_line(token)?;
    Some((paren_end + 1, path, line))
}

fn is_path_character(character: char) -> bool {
    character.is_ascii_alphanumeric()
        || matches!(character, '.' | '/' | '_' | '-' | ':' | '~' | '+')
}

fn url_end(characters: &[char], index: usize) -> Option<usize> {
    for scheme in ["https://", "http://", "file://"] {
        let scheme: Vec<char> = scheme.chars().collect();
        if characters.len() < index + scheme.len()
            || characters[index..index + scheme.len()] != scheme[..]
        {
            continue;
        }
        // A bracket or a quote in front is punctuation. A path character in
        // front means the scheme is inside some other token, and that is not a
        // link at all.
        if index > 0 && is_path_character(characters[index - 1]) {
            continue;
        }
        let mut end = index + scheme.len();
        while end < characters.len() && !characters[end].is_whitespace() {
            end += 1;
        }
        return Some(end);
    }
    None
}

/// A URL at the end of a sentence carries the sentence's punctuation, and a
/// closing bracket belongs to whatever opened it.
fn trim_trailing_punctuation(url: &str) -> &str {
    url.trim_end_matches(|character| {
        matches!(
            character,
            '.' | ',' | ';' | ':' | ')' | ']' | '>' | '"' | '\''
        )
    })
}

/// Split a `<file>:<line>:<column>` token into the path and the line.
///
/// The column is read and dropped: the editor opens at a line, and a column
/// nobody can act on is a number that only makes the row longer.
fn split_path_and_line(token: &str) -> Option<(String, Option<usize>)> {
    let mut parts = token.split(':');
    let path = parts.next()?;
    if !looks_like_a_path(path) {
        return None;
    }
    let line = parts.next().and_then(|value| value.parse::<usize>().ok());
    Some((path.to_owned(), line.filter(|line| *line > 0)))
}

/// A dot with an extension after it, and something before the dot. `.` and
/// `..` are not files, a trailing dot is punctuation, and a bare word is a
/// word.
fn looks_like_a_path(path: &str) -> bool {
    let clean = path.strip_prefix("file://").unwrap_or(path);
    let Some((stem, extension)) = clean.rsplit_once('.') else {
        return false;
    };
    let name = stem.rsplit('/').next().unwrap_or(stem);
    !name.is_empty()
        && !extension.is_empty()
        && extension.len() <= TERMINAL_PATH_MAX_EXTENSION
        && extension
            .chars()
            .all(|character| character.is_ascii_alphanumeric())
        // A version number is not a file. An extension made only of digits has
        // never been one either, and output is full of `1.2.3`.
        && extension
            .chars()
            .any(|character| character.is_ascii_alphabetic())
}

/// Return the home directory if available from environment or system.
pub(crate) fn home_directory() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .or_else(|| directories::UserDirs::new().map(|dirs| dirs.home_dir().to_path_buf()))
}

/// Resolve a candidate path string (handling `file://`, `~`, `~/`, absolute, and relative paths)
/// into a canonicalized PathBuf if it exists on disk as a file.
pub(crate) fn resolve_candidate_path(base_dir: &Path, path: &str) -> Option<PathBuf> {
    let clean = path.strip_prefix("file://").unwrap_or(path);
    let candidate = if let Some(stripped) = clean.strip_prefix("~/") {
        home_directory()?.join(stripped)
    } else if clean == "~" {
        home_directory()?
    } else if clean.starts_with('/') {
        PathBuf::from(clean)
    } else {
        base_dir.join(clean)
    };
    let resolved = candidate.canonicalize().ok()?;
    if !resolved.is_file() {
        return None;
    }
    Some(resolved)
}

/// Where a path found in output actually is, or nothing.
///
/// Relative paths are resolved against the session's folder and absolute ones
/// are taken as they are; either way the result has to be inside the folder,
/// so a stack trace naming somebody's home directory cannot turn a click into
/// a way of reading files this session has nothing to do with.
pub(crate) fn resolve_terminal_path(worktree: &Path, path: &str) -> Option<PathBuf> {
    let resolved = resolve_candidate_path(worktree, path)?;
    let root = worktree
        .canonicalize()
        .unwrap_or_else(|_| worktree.to_path_buf());
    resolved.starts_with(&root).then_some(resolved)
}

/// Where a path found in terminal output is, resolving first against the session's
/// active worktree (if any) and falling back to the project root.
/// For absolute and tilde paths, also resolves if the file exists on disk outside project root.
pub(crate) fn resolve_terminal_session_path(
    worktree: Option<&Path>,
    project_root: &Path,
    path: &str,
) -> Option<PathBuf> {
    if let Some(wt) = worktree.filter(|wt| *wt != project_root) {
        if let Some(resolved) = resolve_terminal_path(wt, path) {
            return Some(resolved);
        }
    }
    if let Some(resolved) = resolve_terminal_path(project_root, path) {
        return Some(resolved);
    }
    let clean = path.strip_prefix("file://").unwrap_or(path);
    if clean.starts_with('/') || clean.starts_with('~') {
        if let Some(resolved) = resolve_candidate_path(project_root, path) {
            return Some(resolved);
        }
    }
    None
}

/// What the pane needs in order to draw the openable things in the rows it can
/// see, and to say which one was clicked.
pub(crate) struct TerminalLinkOverlay<'a> {
    /// Resolved paths, from the application's cache. A path that is not in here
    /// yet has not been probed; one that is here as `None` is not a file. Only
    /// a resolved one is drawn as openable, so the pane never underlines
    /// something a click could not open.
    pub(crate) resolved: &'a HashMap<String, Option<PathBuf>>,
    /// Candidates found in visible rows that the cache has never heard of.
    /// Filled while drawing and drained outside it — a `fs::metadata` call in a
    /// draw path is what `.claude/rules/rust.md` forbids.
    pub(crate) unresolved: &'a mut Vec<String>,
    pub(crate) palette: &'a Palette,
}

/// Draw the underline under everything openable on this row, and answer which
/// one the pointer is on.
///
/// Returns the target under the pointer so the caller can decide what a click
/// means: the pane owns focus and typing, and stealing every click would take
/// the keyboard away from an agent that is being talked to.
pub(crate) fn terminal_row_links(
    ui: &egui::Ui,
    text: &str,
    rect: egui::Rect,
    cell: egui::Vec2,
    overlay: &mut TerminalLinkOverlay<'_>,
) -> Option<TerminalTargetKind> {
    let pointer = ui.input(|input| input.pointer.hover_pos());
    let mut under_pointer = None;
    for target in terminal_targets(text) {
        let openable = match &target.kind {
            TerminalTargetKind::Url(_) => true,
            TerminalTargetKind::Path { path, .. } => match overlay.resolved.get(path) {
                Some(resolved) => resolved.is_some(),
                None => {
                    if overlay.unresolved.len() < TERMINAL_PATH_PROBE_LIMIT
                        && !overlay.unresolved.iter().any(|seen| seen == path)
                    {
                        overlay.unresolved.push(path.clone());
                    }
                    false
                }
            },
        };
        if !openable {
            continue;
        }
        let span = egui::Rect::from_min_size(
            egui::pos2(rect.left() + target.column as f32 * cell.x, rect.top()),
            egui::vec2(target.columns as f32 * cell.x, cell.y),
        );
        let hovered = pointer.is_some_and(|position| span.contains(position));
        if hovered {
            // Only on hover. A pane of permanently underlined words is a pane
            // that is harder to read than the plain text it replaced.
            ui.painter().hline(
                span.x_range(),
                span.bottom() - 1.0,
                egui::Stroke::new(1.0, overlay.palette.accent_text),
            );
            under_pointer = Some(target.kind.clone());
        }
    }
    under_pointer
}
