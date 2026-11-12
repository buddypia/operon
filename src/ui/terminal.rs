use crate::prelude::*;
use crate::*;

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub(crate) struct TerminalTextStyle {
    pub(crate) foreground: Option<Color32>,
    pub(crate) background: Option<Color32>,
    pub(crate) bright: bool,
    pub(crate) inverse: bool,
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

fn parse_extended_sgr_color(
    parameters: &[u16],
    index: usize,
    palette: &Palette,
) -> (Option<Color32>, usize) {
    let last = parameters.len().saturating_sub(1);
    match parameters.get(index + 1).copied() {
        Some(5) => {
            let color = parameters
                .get(index + 2)
                .map(|&value| terminal_ansi_256_color(value.min(255) as u8, palette));
            (color, (index + 2).min(last))
        }
        Some(2) => {
            let color = match (
                parameters.get(index + 2),
                parameters.get(index + 3),
                parameters.get(index + 4),
            ) {
                (Some(&red), Some(&green), Some(&blue)) => Some(Color32::from_rgb(
                    red.min(255) as u8,
                    green.min(255) as u8,
                    blue.min(255) as u8,
                )),
                _ => None,
            };
            (color, (index + 4).min(last))
        }
        _ => (None, index),
    }
}

fn apply_colon_sgr_subparameters(token: &str, style: &mut TerminalTextStyle, palette: &Palette) {
    let parts: Vec<Option<u16>> = token
        .split(':')
        .map(|part| part.parse::<u16>().ok())
        .collect();
    let parse_color = |parts: &[Option<u16>]| -> Option<Color32> {
        match parts.get(1).copied().flatten() {
            Some(5) => parts
                .get(2)
                .copied()
                .flatten()
                .map(|value| terminal_ansi_256_color(value.min(255) as u8, palette)),
            Some(2) => {
                let rgb = if parts.len() >= 6 {
                    (parts.get(3), parts.get(4), parts.get(5))
                } else {
                    (parts.get(2), parts.get(3), parts.get(4))
                };
                match rgb {
                    (Some(Some(r)), Some(Some(g)), Some(Some(b))) => Some(Color32::from_rgb(
                        (*r).min(255) as u8,
                        (*g).min(255) as u8,
                        (*b).min(255) as u8,
                    )),
                    _ => None,
                }
            }
            _ => None,
        }
    };
    match parts.first().copied().flatten() {
        Some(0) | None => *style = TerminalTextStyle::default(),
        Some(38) => {
            if let Some(color) = parse_color(&parts) {
                style.foreground = Some(color);
            }
        }
        Some(48) => {
            if let Some(color) = parse_color(&parts) {
                style.background = Some(color);
            }
        }
        _ => {}
    }
}

pub(crate) fn apply_terminal_sgr(
    parameters: &str,
    style: &mut TerminalTextStyle,
    palette: &Palette,
) {
    if parameters.is_empty() {
        *style = TerminalTextStyle::default();
        return;
    }
    let mut numeric = Vec::new();
    for token in parameters.split(';') {
        if token.contains(':') {
            apply_colon_sgr_subparameters(token, style, palette);
            continue;
        }
        if token.is_empty() {
            numeric.push(0);
        } else if let Ok(value) = token.parse::<u16>() {
            numeric.push(value);
        }
    }
    let mut index = 0;
    while index < numeric.len() {
        match numeric[index] {
            0 => *style = TerminalTextStyle::default(),
            1 => style.bright = true,
            7 => style.inverse = true,
            22 => style.bright = false,
            27 => style.inverse = false,
            30..=37 => {
                style.foreground = Some(terminal_ansi_color(
                    (numeric[index] - 30) as u8,
                    false,
                    palette,
                ))
            }
            38 => {
                let (color, next) = parse_extended_sgr_color(&numeric, index, palette);
                if let Some(color) = color {
                    style.foreground = Some(color);
                }
                index = next;
            }
            39 => style.foreground = None,
            40..=47 => {
                style.background = Some(terminal_ansi_color(
                    (numeric[index] - 40) as u8,
                    false,
                    palette,
                ))
            }
            48 => {
                let (color, next) = parse_extended_sgr_color(&numeric, index, palette);
                if let Some(color) = color {
                    style.background = Some(color);
                }
                index = next;
            }
            49 => style.background = None,
            58 => {
                let (_, next) = parse_extended_sgr_color(&numeric, index, palette);
                index = next;
            }
            90..=97 => {
                style.foreground = Some(terminal_ansi_color(
                    (numeric[index] - 90) as u8,
                    true,
                    palette,
                ))
            }
            100..=107 => {
                style.background = Some(terminal_ansi_color(
                    (numeric[index] - 100) as u8,
                    true,
                    palette,
                ))
            }
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
    let foreground = style.foreground.unwrap_or(if style.bright {
        palette.ansi_bright[7]
    } else {
        palette.terminal_fg
    });
    let (color, background) = if style.inverse {
        (style.background.unwrap_or(palette.terminal_bg), foreground)
    } else {
        (foreground, style.background.unwrap_or(Color32::TRANSPARENT))
    };
    job.append(
        text,
        0.0,
        TextFormat {
            font_id: FontId::monospace(MONOSPACE_SIZE),
            color,
            background,
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

static NEXT_TERMINAL_LINE_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

fn next_terminal_line_id() -> u64 {
    NEXT_TERMINAL_LINE_ID.fetch_add(1, Ordering::Relaxed)
}

fn clean_terminal_line(raw_line: &str) -> Cow<'_, str> {
    if !raw_line.contains('\r') {
        Cow::Borrowed(raw_line)
    } else if let Some(stripped) = raw_line.strip_suffix('\r') {
        if !stripped.contains('\r') {
            Cow::Borrowed(stripped)
        } else {
            Cow::Owned(stripped.replace('\r', ""))
        }
    } else {
        Cow::Owned(raw_line.replace('\r', ""))
    }
}

fn hash_clean_line(line: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for &byte in line.as_bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}

#[derive(Clone, Debug)]
pub(crate) struct CachedTerminalLine {
    pub(crate) line_id: u64,
    pub(crate) style_in: TerminalTextStyle,
    pub(crate) style_out: TerminalTextStyle,
    pub(crate) raw_hash: u64,
    pub(crate) raw: Box<str>,
    pub(crate) job: LayoutJob,
}

#[derive(Clone, Debug)]
pub(crate) struct IncrementalTerminalLayout {
    pub(crate) theme: AppTheme,
    pub(crate) lines: Vec<CachedTerminalLine>,
    #[cfg(test)]
    parsed_lines: usize,
    dirty_lines: Vec<usize>,
}

pub(crate) fn app_theme_code(theme: AppTheme) -> u8 {
    match theme {
        AppTheme::Dark => 0,
        AppTheme::Light => 1,
        AppTheme::HighContrast => 2,
    }
}

pub(crate) fn app_theme_from_code(code: u8) -> AppTheme {
    match code {
        1 => AppTheme::Light,
        2 => AppTheme::HighContrast,
        _ => AppTheme::Dark,
    }
}

impl IncrementalTerminalLayout {
    #[cfg(test)]
    pub(crate) fn parsed_lines(&self) -> usize {
        self.parsed_lines
    }

    pub(crate) fn take_dirty_lines(&mut self) -> Vec<usize> {
        let mut out = std::mem::take(&mut self.dirty_lines);
        out.shrink_to_fit();
        out
    }
}

/// Incremental line-level ANSI layout (change 292).
/// Strips `\r` per line without allocating a whole-buffer copy and reuses
/// cached `LayoutJob`s and monotonic `line_id`s whenever `(theme, style_in, raw)`
/// matches across in-place TUI updates or scrollback shifts.
pub(crate) fn terminal_line_layouts_incremental(
    output: &str,
    theme: AppTheme,
    previous: Option<&IncrementalTerminalLayout>,
) -> (IncrementalTerminalLayout, Vec<LayoutJob>, Vec<u64>) {
    let palette = theme.palette();
    let prev_lines = previous
        .filter(|prev| prev.theme == theme)
        .map(|prev| prev.lines.as_slice());

    let cleaned: Vec<(Cow<'_, str>, u64)> = output
        .split('\n')
        .map(|raw_line| {
            let clean = clean_terminal_line(raw_line);
            let hash = hash_clean_line(&clean);
            (clean, hash)
        })
        .collect();

    let mut shift: usize = 0;
    if let Some(prev) = prev_lines {
        if let Some((anchor_idx, (anchor_str, anchor_hash))) = cleaned
            .iter()
            .take(32)
            .enumerate()
            .find(|(_, (line, _))| !line.is_empty())
            .or_else(|| cleaned.first().map(|entry| (0, entry)))
        {
            if !prev.get(anchor_idx).is_some_and(|entry| {
                entry.raw_hash == *anchor_hash && entry.raw.as_ref() == anchor_str.as_ref()
            }) {
                let search_end = prev.len().min(anchor_idx.saturating_add(1024));
                for (cand, entry) in prev
                    .iter()
                    .enumerate()
                    .take(search_end)
                    .skip(anchor_idx + 1)
                {
                    if entry.raw_hash == *anchor_hash && entry.raw.as_ref() == anchor_str.as_ref() {
                        shift = cand - anchor_idx;
                        break;
                    }
                }
            }
        }
    }

    let mut next_lines = Vec::with_capacity(cleaned.len());
    let mut jobs = Vec::with_capacity(cleaned.len());
    let mut line_ids = Vec::with_capacity(cleaned.len());
    let mut dirty_lines = Vec::new();
    #[cfg(test)]
    let mut parsed_lines = 0;
    let mut style = TerminalTextStyle::default();
    let mut running_offset = shift as isize;

    for (idx, (clean_line, raw_hash)) in cleaned.into_iter().enumerate() {
        let same_shifted_slot = prev_lines
            .and_then(|prev| prev.get(idx.saturating_add(shift)))
            .is_some_and(|entry| {
                entry.raw_hash == raw_hash && entry.raw.as_ref() == clean_line.as_ref()
            });

        let matched = prev_lines.and_then(|prev| {
            let matches_entry = |entry: &CachedTerminalLine| {
                entry.style_in == style
                    && entry.raw_hash == raw_hash
                    && entry.raw.as_ref() == clean_line.as_ref()
            };
            let shifted_idx = idx.checked_add_signed(running_offset);
            if let Some(cand) = shifted_idx {
                if let Some(entry) = prev.get(cand).filter(|e| matches_entry(e)) {
                    return Some((cand, entry));
                }
            }
            if running_offset != 0 {
                if let Some(entry) = prev.get(idx).filter(|e| matches_entry(e)) {
                    return Some((idx, entry));
                }
            }
            let base = shifted_idx.unwrap_or(idx);
            for delta in 1..=8_usize {
                if let Some(cand) = base.checked_add(delta) {
                    if let Some(entry) = prev.get(cand).filter(|e| matches_entry(e)) {
                        return Some((cand, entry));
                    }
                }
                if let Some(cand) = base.checked_sub(delta) {
                    if let Some(entry) = prev.get(cand).filter(|e| matches_entry(e)) {
                        return Some((cand, entry));
                    }
                }
            }
            None
        });

        if let Some((matched_idx, entry)) = matched {
            running_offset = matched_idx as isize - idx as isize;
            if !same_shifted_slot {
                dirty_lines.push(idx);
            }
            style = entry.style_out;
            line_ids.push(entry.line_id);
            jobs.push(entry.job.clone());
            next_lines.push(entry.clone());
        } else {
            let style_in = style;
            let mut job = LayoutJob::default();
            let style_out = append_terminal_line(&mut job, &clean_line, style_in, palette);
            style = style_out;
            let line_id = next_terminal_line_id();
            #[cfg(test)]
            {
                parsed_lines += 1;
            }
            dirty_lines.push(idx);
            line_ids.push(line_id);
            jobs.push(job.clone());
            next_lines.push(CachedTerminalLine {
                line_id,
                style_in,
                style_out,
                raw_hash,
                raw: clean_line.into_owned().into_boxed_str(),
                job,
            });
        }
    }

    (
        IncrementalTerminalLayout {
            theme,
            lines: next_lines,
            #[cfg(test)]
            parsed_lines,
            dirty_lines,
        },
        jobs,
        line_ids,
    )
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

/// Which way the find bar steps this frame — `Some(true)` for Enter,
/// `Some(false)` for Shift+Enter — and whether a conversion is still open
/// after it.
///
/// The same reading as `terminal_input_events`, for a field rather than a
/// pane: each confirmed conversion owns one Enter wherever it falls in the
/// frame (lesson 016), and an Enter while a preedit is showing is the IME's.
pub(crate) fn find_bar_step(events: &[egui::Event], composing: bool) -> (Option<bool>, bool) {
    let mut enters_the_ime_owns = events
        .iter()
        .filter(|e| matches!(e, egui::Event::Ime(egui::ImeEvent::Commit(text)) if !text.is_empty()))
        .count();
    let mut composing = composing;
    let mut step = None;
    for event in events {
        match event {
            egui::Event::Ime(egui::ImeEvent::Preedit(text)) => composing = !text.is_empty(),
            egui::Event::Ime(egui::ImeEvent::Commit(_) | egui::ImeEvent::Disabled) => {
                composing = false;
            }
            egui::Event::Key {
                key: egui::Key::Enter,
                pressed: true,
                modifiers,
                ..
            } => {
                if enters_the_ime_owns > 0 {
                    enters_the_ime_owns -= 1;
                    continue;
                }
                if composing {
                    continue;
                }
                if modifiers.is_none() {
                    step = Some(true);
                } else if modifiers.shift_only() {
                    step = Some(false);
                }
            }
            _ => {}
        }
    }
    (step, composing)
}

pub(crate) fn terminal_key_binding(key: egui::Key, modifiers: egui::Modifiers) -> Option<String> {
    // Keep macOS application shortcuts under the app's control. Ctrl remains
    // available for terminal conventions such as Ctrl+C and Ctrl+R.
    if modifiers.command {
        return None;
    }
    // Shift+Enter is a newline in an agent's input, Enter a submit. tmux
    // writes `\r` for `S-Enter` as for `Enter`, so the Shift cannot ride on
    // the Enter; send what Ctrl+J sends instead — the LF a terminal set up for
    // Shift+Enter sends, which Claude Code, Codex, and Antigravity CLI read as
    // a newline (change 264). Exactly Shift: Ctrl or Option keep their chords.
    if key == egui::Key::Enter && modifiers.matches_exact(egui::Modifiers::SHIFT) {
        return terminal_key_binding(egui::Key::J, egui::Modifiers::CTRL);
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
#[allow(clippy::too_many_arguments)]
pub(crate) fn terminal_rows(
    ui: &mut egui::Ui,
    lines: &[LayoutJob],
    viewport: egui::Vec2,
    row_height: f32,
    search: TerminalSearchOverlay<'_>,
    links: &mut TerminalLinkOverlay<'_>,
    cursor: Option<TerminalCursor>,
    context: Option<TerminalPaneContext<'_>>,
) -> TerminalRowsResult {
    // Terminal rows sit directly on top of each other. Any spacing here would
    // also be counted twice: `show_rows` reserves height per row from it.
    ui.spacing_mut().item_spacing.y = 0.0;
    let cell = terminal_cell_size(ui);
    // The pane's caret lives in the coordinate system the rows are drawn in,
    // not in the one the viewport measures from (change 155). The capture is
    // scrollback taller than the pane's grid, so the grid row has to be
    // renumbered into a row of the buffer before a rectangle can anchor to it.
    let caret_row: Option<Option<(usize, u16)>> = cursor.map(|cursor| {
        terminal_cursor_row(lines.len(), cursor.pane_rows, cursor.y).map(|row| (row, cursor.x))
    });
    // Named and salted per session. Two sibling `allocate_ui_with_layout` children
    // of one row share a `Ui::id` in egui 0.31 — `new_child` salts every
    // anonymous child with the same constant — so two unnamed scroll areas, this
    // one and the prompt timeline's, hash to the same `Id`. Salting by session_id
    // isolates persistent scroll positions between workspace tabs.
    let session_salt = context.as_ref().map(|c| c.session_id);
    let mut area = egui::ScrollArea::vertical()
        .id_salt(("terminal-rows", session_salt))
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
    // The pane's content width, measured on the outer ui: once the scroll
    // area is inside its closure the clip is measured from the laid-out
    // content, so it offers no bound against a stale capture that came from a
    // wider grid.
    let pane_width = ui.available_width();
    let mut under_pointer = None;
    let mut caret = None;
    area.show_rows(ui, row_height, lines.len(), |ui, rows| {
        let first = rows.start;
        let top = ui.cursor().top();
        // The caret is a row of this buffer now, so it anchors exactly where a
        // highlighted match anchors — and `show_rows` only ever shows rows that
        // are visible, so a caret the person scrolled away reports none.
        let computed_caret = if let Some(Some((row, column))) = caret_row {
            // A caret scrolled above the viewport has its row before the band's
            // start (egui's end-of-content clamp can put a full-screen TUI's
            // cursor there): offset_rows must abort rather than underflow.
            let rect = row.checked_sub(first).map(|offset_rows| {
                egui::Rect::from_min_size(
                    egui::pos2(ui.min_rect().left() + column as f32 * cell.x, top),
                    cell,
                )
                .translate(egui::vec2(0.0, offset_rows as f32 * row_height))
            });
            // The box belongs on a cell the viewer can see, all of it. egui
            // serves one spare row past the viewport's bottom (`show_rows`
            // adds +1 to the served range), and the band's y clip already
            // includes the space the scrollbar reserve animates away — so a
            // caret a half cell below the viewport still passes the clip.
            // Full containment against that clip is the bound. Horizontally
            // the clip overlays the pane's whole width (minus the bar
            // reserve), so the pane's own reserved width, measured on the
            // outer ui, is what stops a capture from before a resize from
            // painting past the frame. The tolerance is sub-pixel slack for
            // float noise between the two measurements: half a pixel, about a
            // sixteenth of a cell.
            if let Some(rect) = rect {
                let clip = ui.clip_rect();
                let inner_right = ui.min_rect().left() + pane_width;
                let slack = 0.5;
                if rect.min.y >= clip.min.y
                    && rect.max.y <= clip.max.y
                    && rect.min.x >= clip.min.x
                    && rect.max.x <= inner_right + slack
                {
                    Some(rect)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        let egui_ctx = ui.ctx().clone();
        let pixels_per_point_bits = egui_ctx.pixels_per_point().to_bits();
        let cached_galleys_storage = context.map(|ctx| {
            let key = TerminalPaneKey {
                session_id: ctx.session_id,
                visible_rows: (rows.start, rows.end),
                output_generation: ctx.output_generation,
                theme: ctx.theme,
                pixels_per_point_bits,
            };
            ctx.cache
                .get_or_render_rows(&egui_ctx, ctx.session_id, &key, lines, ctx.line_ids)
        });

        // Placed for this frame's pane, never cached: the pane moves and
        // resizes without its rows changing.
        caret = computed_caret;
        let hover_pos = ui.input(|input| input.pointer.hover_pos());

        for (offset, line) in lines[rows.clone()].iter().enumerate() {
            let line_idx = first + offset;
            let y = top + offset as f32 * row_height;
            let row_pos = egui::pos2(ui.min_rect().left(), y);
            let row_rect = egui::Rect::from_min_size(
                row_pos,
                egui::vec2(ui.min_rect().width().max(pane_width), row_height),
            );

            // Only the rows the viewport asked for, so the cost of a search is
            // the size of the pane and never the size of the scrollback.
            if !search.matches.is_empty() {
                let start_match = search
                    .matches
                    .partition_point(|found| found.line < line_idx);
                for (rel_idx, found) in search.matches[start_match..].iter().enumerate() {
                    if found.line != line_idx {
                        break;
                    }
                    let index = start_match + rel_idx;
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
            }

            if let Some(galleys) = cached_galleys_storage {
                if let Some(Some(galley)) = galleys.get(offset) {
                    ui.painter()
                        .galley(row_pos, Arc::clone(galley), search.palette.terminal_fg);
                }
                ui.add_space(row_height);
            } else if line.is_empty() {
                // A blank line still owns its row, or every line below it
                // would be drawn one row too high.
                ui.add_space(row_height);
            } else {
                ui.add(egui::Label::new(line.clone()).wrap_mode(egui::TextWrapMode::Extend));
            }

            if !line.is_empty() && hover_pos.is_some_and(|position| row_rect.contains(position)) {
                // Only the hovered row on screen is read for paths and links.
                if let Some(kind) =
                    terminal_row_links_at_pointer(ui, hover_pos, &line.text, row_rect, cell, links)
                {
                    under_pointer = Some(kind);
                }
            }
        }
    });
    TerminalRowsResult {
        hovered: under_pointer,
        caret,
    }
}

/// What `terminal_rows` saw while it drew.
pub(crate) struct TerminalRowsResult {
    pub(crate) hovered: Option<TerminalTargetKind>,
    /// The caret's cell in screen coordinates, when the cursor's row is among
    /// the rows this frame drew. Scrolled away is nothing — a box that floats
    /// past the pane's edge is worse than no box.
    pub(crate) caret: Option<egui::Rect>,
}

/// Dirty-checking key for a pane's laid-out rows: exactly the inputs of a
/// row's galley. The pane's size is not one — rows are laid out unwrapped —
/// nor are the caret and search highlights, which are painted every frame, so
/// a window resize with unchanged output reuses every row (change 255).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TerminalPaneKey {
    /// Identifies the session.
    pub(crate) session_id: Uuid,
    /// Range of visible rows (start, end) served by `ScrollArea::show_rows`.
    pub(crate) visible_rows: (usize, usize),
    /// Monotonically increasing counter bumped only when tmux outputs new content.
    pub(crate) output_generation: u64,
    /// Active theme (Dark/Light).
    pub(crate) theme: AppTheme,
    /// DPI scale factor bits (`pixels_per_point`).
    pub(crate) pixels_per_point_bits: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct TerminalRowGalleyKey {
    line_id: u64,
    font_size_bits: u32,
    theme_code: u8,
    pixels_per_point_bits: u32,
}

/// Cached rendered representation of one terminal pane.
pub(crate) struct CachedTerminalPane {
    /// The key against which this cached rendering was generated.
    pub(crate) key: TerminalPaneKey,
    /// Shaped galleys for the visible rows. `None` represents an empty row (blank line).
    pub(crate) galleys: Vec<Option<Arc<egui::Galley>>>,
}

/// Retained galley & dirty-checked rendering cache for terminal panes.
/// Eliminates per-frame LayoutJob cloning and egui::Label allocations on unchanged frames,
/// and reuses row-level galleys across scrolling and partial TUI updates (change 292).
#[derive(Default)]
pub(crate) struct TerminalPaneRenderCache {
    /// Memoized pane layouts keyed by session ID.
    entries: HashMap<Uuid, CachedTerminalPane>,
    /// Row-level shaped galleys keyed by `(line_id, theme, pixels_per_point)`.
    row_galleys: HashMap<TerminalRowGalleyKey, Arc<egui::Galley>>,
    /// Per-session count of builds (cache misses).
    session_builds: HashMap<Uuid, usize>,
    /// Total number of cache misses (pane-level rebuilds).
    builds: usize,
    /// Total number of cache hits (reused pane slices).
    hits: usize,
    /// Total number of individual row `fonts.layout_job` calls.
    row_galley_builds: usize,
    /// Total number of individual row galley hits during pane misses.
    row_galley_hits: usize,
}

impl TerminalPaneRenderCache {
    /// Query the cache. On a pane-key hit, returns cached visible galleys immediately.
    /// On a miss, reuses any matching row galleys from `row_galleys` and locks
    /// `egui_ctx.fonts` at most once to shape only dirty or newly visible non-empty rows.
    pub(crate) fn get_or_render_rows(
        &mut self,
        egui_ctx: &egui::Context,
        session_id: Uuid,
        key: &TerminalPaneKey,
        lines: &[LayoutJob],
        line_ids: Option<&[u64]>,
    ) -> &[Option<Arc<egui::Galley>>] {
        let is_hit = self
            .entries
            .get(&session_id)
            .is_some_and(|entry| entry.key == *key);
        if is_hit {
            self.hits += 1;
        } else {
            let end = key.visible_rows.1.min(lines.len());
            let start = key.visible_rows.0.min(end);
            let range = start..end;
            let mut galleys = Vec::with_capacity(range.len());
            let mut missing_indices: Vec<(usize, usize, Option<TerminalRowGalleyKey>)> = Vec::new();

            for (slot, row_idx) in range.enumerate() {
                let line = &lines[row_idx];
                if line.is_empty() {
                    galleys.push(None);
                    continue;
                }
                let font_size_bits = line
                    .sections
                    .first()
                    .map_or(0, |section| section.format.font_id.size.to_bits());
                let row_key = line_ids
                    .and_then(|ids| ids.get(row_idx).copied())
                    .filter(|&id| id != 0)
                    .map(|line_id| TerminalRowGalleyKey {
                        line_id,
                        font_size_bits,
                        theme_code: app_theme_code(key.theme),
                        pixels_per_point_bits: key.pixels_per_point_bits,
                    });
                if let Some(rk) = row_key {
                    if let Some(cached) = self.row_galleys.get(&rk) {
                        self.row_galley_hits += 1;
                        galleys.push(Some(Arc::clone(cached)));
                        continue;
                    }
                }
                galleys.push(None);
                missing_indices.push((slot, row_idx, row_key));
            }

            if !missing_indices.is_empty() {
                let shaped: Vec<Arc<egui::Galley>> = egui_ctx.fonts(|fonts| {
                    missing_indices
                        .iter()
                        .map(|&(_, row_idx, _)| fonts.layout_job(lines[row_idx].clone()))
                        .collect()
                });
                self.row_galley_builds += shaped.len();
                let cacheable_new = missing_indices
                    .iter()
                    .filter(|(_, _, rk)| rk.is_some())
                    .count();
                if cacheable_new > 0
                    && self.row_galleys.len() + cacheable_new > MAX_CACHED_TERMINAL_ROW_GALLEYS
                {
                    self.row_galleys.clear();
                }
                for ((slot, _, row_key), galley) in missing_indices.into_iter().zip(shaped) {
                    if let Some(rk) = row_key {
                        if self.row_galleys.len() < MAX_CACHED_TERMINAL_ROW_GALLEYS {
                            self.row_galleys.insert(rk, Arc::clone(&galley));
                        }
                    }
                    galleys[slot] = Some(galley);
                }
            }

            self.entries.insert(
                session_id,
                CachedTerminalPane {
                    key: key.clone(),
                    galleys,
                },
            );
            self.builds += 1;
            *self.session_builds.entry(session_id).or_default() += 1;
        }
        let entry = self.entries.get(&session_id).expect("entry must exist");
        &entry.galleys
    }

    /// Number of full rebuilds (cache misses).
    #[cfg(test)]
    pub(crate) fn builds(&self) -> usize {
        self.builds
    }

    /// Number of rebuilds for a specific session.
    #[cfg(test)]
    pub(crate) fn session_builds(&self, session_id: Uuid) -> usize {
        self.session_builds.get(&session_id).copied().unwrap_or(0)
    }

    /// Number of cache hits.
    #[cfg(test)]
    pub(crate) fn hits(&self) -> usize {
        self.hits
    }

    /// Number of individual row `fonts.layout_job` calls across all misses.
    #[cfg(test)]
    pub(crate) fn row_galley_builds(&self) -> usize {
        self.row_galley_builds
    }

    /// Number of individual row galley cache hits during pane misses.
    #[cfg(test)]
    pub(crate) fn row_galley_hits(&self) -> usize {
        self.row_galley_hits
    }

    /// Evict a specific session (e.g. when closed or deleted).
    pub(crate) fn remove(&mut self, session_id: &Uuid) -> Option<CachedTerminalPane> {
        self.session_builds.remove(session_id);
        self.entries.remove(session_id)
    }

    /// Clear all cached entries (e.g. on theme or font change).
    pub(crate) fn clear(&mut self) {
        self.entries.clear();
        self.row_galleys.clear();
        self.session_builds.clear();
    }

    /// Retain only active sessions to prevent unbounded memory growth.
    pub(crate) fn retain(&mut self, mut f: impl FnMut(&Uuid, &mut CachedTerminalPane) -> bool) {
        self.entries.retain(|id, pane| f(id, pane));
        self.session_builds
            .retain(|id, _| self.entries.contains_key(id));
    }
}

/// Context passed into `terminal_rows` containing identity, dirty generation, and cache reference.
pub(crate) struct TerminalPaneContext<'a> {
    pub(crate) session_id: Uuid,
    pub(crate) output_generation: u64,
    pub(crate) theme: AppTheme,
    pub(crate) line_ids: Option<&'a [u64]>,
    pub(crate) cache: &'a mut TerminalPaneRenderCache,
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

/// A row of the pane's grid, numbered as a row of the capture instead.
///
/// The capture's split always leaves one phantom entry after the final newline,
/// so the rows the buffer shows are `lines.len() - 1`, the last `pane_rows` of
/// which are the screen. tmux pads the capture to the grid's height — a fresh
/// 20-row pane captures exactly 20 lines — and a read that hits the output
/// limit errors out instead of delivering short stdout, so through
/// `tmux_capture_with_cursor` the mapping always stands: `body - pane + y`.
/// `#{pane_height}` absent or a cursor at and below the pane reports no row.
///
/// Short captures are still defended: whatever producer ever shows the pane
/// one cut from its head, the rows that survive are the buffer's tail, no row
/// of which is the cursor's row — so report none rather than a box drawn on
/// the wrong line (change 155).
pub(crate) fn terminal_cursor_row(lines_len: usize, pane_rows: u16, y: u16) -> Option<usize> {
    let pane_rows = usize::from(pane_rows);
    let body_rows = lines_len.checked_sub(1)?;
    if pane_rows == 0 || usize::from(y) >= pane_rows || body_rows < pane_rows {
        return None;
    }
    Some(body_rows - pane_rows + usize::from(y))
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
    let needle_columns: usize = needle.iter().copied().map(terminal_character_columns).sum();
    let mut matches = Vec::new();
    let mut haystack = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        if line.text.len() < needle.len() {
            continue;
        }
        haystack.clear();
        haystack.extend(line.text.chars().map(|c| c.to_ascii_lowercase()));
        if haystack.len() < needle.len() {
            continue;
        }
        let mut at = 0;
        let mut column = 0;
        while at + needle.len() <= haystack.len() {
            if haystack[at..at + needle.len()] != needle[..] {
                column += terminal_character_columns(haystack[at]);
                at += 1;
                continue;
            }
            matches.push(TerminalMatch {
                line: index,
                column,
                columns: needle_columns,
            });
            if matches.len() >= TERMINAL_SEARCH_MAX_MATCHES {
                return matches;
            }
            // Occurrences do not overlap: the next one starts after this one.
            column += needle_columns;
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

pub(crate) fn draw_terminal_cursor(ui: &egui::Ui, caret: egui::Rect, palette: &Palette) {
    let stroke_color = palette.terminal_fg.gamma_multiply(0.6);
    ui.painter().rect_stroke(
        caret,
        0.0,
        egui::Stroke::new(1.5, stroke_color),
        egui::StrokeKind::Inside,
    );
}

pub(crate) fn draw_terminal_preedit(
    ui: &egui::Ui,
    caret: egui::Rect,
    preedit: &str,
    palette: &Palette,
) {
    if preedit.is_empty() {
        return;
    }
    let pos = caret.min;
    let font_id = FontId::monospace(MONOSPACE_SIZE);
    let galley = ui
        .painter()
        .layout_no_wrap(preedit.to_string(), font_id, palette.terminal_fg);
    let width = galley.size().x;
    let preedit_rect = egui::Rect::from_min_size(pos, egui::vec2(width, caret.height()));

    // Under-layer highlight
    ui.painter()
        .rect_filled(preedit_rect, 1.0, palette.selection);

    // Text
    ui.painter().galley(pos, galley, palette.terminal_fg);

    // High-visibility underline
    let underline_y = pos.y + caret.height() - 1.0;
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
    resolve_candidate_path_under(base_dir, path, home_directory().as_deref())
}

/// `resolve_candidate_path` with the home folder given, so a guard can supply
/// one that is not the person's.
///
/// A path under a folder macOS guards (`PROTECTED_HOME_FOLDERS`,
/// `PROTECTED_VOLUMES_ROOT`) is refused as written, before anything touches the
/// disk, unless it is inside `base_dir`: looking it up would have macOS ask the
/// person, in Operon's name, about a file only an agent's output pointed at
/// (change 265).
pub(crate) fn resolve_candidate_path_under(
    base_dir: &Path,
    path: &str,
    home: Option<&Path>,
) -> Option<PathBuf> {
    let clean = path.strip_prefix("file://").unwrap_or(path);
    let candidate = if let Some(stripped) = clean.strip_prefix("~/") {
        home?.join(stripped)
    } else if clean == "~" {
        home?.to_path_buf()
    } else if clean.starts_with('/') {
        PathBuf::from(clean)
    } else {
        base_dir.join(clean)
    };
    let candidate = lexically_normal(&candidate);
    if is_in_protected_folder(&candidate, home) && !is_inside_folder(&candidate, base_dir) {
        return None;
    }
    let resolved = candidate.canonicalize().ok()?;
    if !resolved.is_file() {
        return None;
    }
    Some(resolved)
}

/// `path` with `.` and `..` taken out by reading it rather than by asking the
/// disk. The candidates here are absolute, so a `..` at the root stays there.
fn lexically_normal(path: &Path) -> PathBuf {
    let mut normal = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normal.pop();
            }
            other => normal.push(other),
        }
    }
    normal
}

/// Whether macOS asks before an app looks at `path`: under the home folder's
/// Desktop, Documents or Downloads, or on a mounted volume. Compared without
/// case, because APFS finds `~/downloads` as readily as `~/Downloads`, and
/// without the data volume's prefix, which reaches the same home folder.
pub(crate) fn is_in_protected_folder(path: &Path, home: Option<&Path>) -> bool {
    let folded = |path: &Path| PathBuf::from(path.to_string_lossy().to_lowercase());
    let path = folded(path);
    let data_volume = folded(Path::new(SYSTEM_DATA_VOLUME_ROOT));
    let path = match path.strip_prefix(&data_volume) {
        Ok(rest) => Path::new("/").join(rest),
        Err(_) => path,
    };
    let under = |root: &Path| path.starts_with(folded(root));
    under(Path::new(PROTECTED_VOLUMES_ROOT))
        || home.is_some_and(|home| {
            let home = lexically_normal(home);
            PROTECTED_HOME_FOLDERS
                .iter()
                .any(|folder| under(&home.join(folder)))
        })
}

/// Whether `path` is inside `folder` as given or as canonicalized. Only the
/// folder is resolved, and it is one the person registered.
fn is_inside_folder(path: &Path, folder: &Path) -> bool {
    path.starts_with(lexically_normal(folder))
        || folder
            .canonicalize()
            .is_ok_and(|canonical| path.starts_with(canonical))
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
#[cfg(test)]
pub(crate) fn terminal_row_links(
    ui: &egui::Ui,
    text: &str,
    rect: egui::Rect,
    cell: egui::Vec2,
    overlay: &mut TerminalLinkOverlay<'_>,
) -> Option<TerminalTargetKind> {
    let hover_pos = ui.input(|input| input.pointer.hover_pos());
    terminal_row_links_at_pointer(ui, hover_pos, text, rect, cell, overlay)
}

fn terminal_row_links_at_pointer(
    ui: &egui::Ui,
    hover_pos: Option<egui::Pos2>,
    text: &str,
    rect: egui::Rect,
    cell: egui::Vec2,
    overlay: &mut TerminalLinkOverlay<'_>,
) -> Option<TerminalTargetKind> {
    // Only the row under the pointer is read. The underline is drawn on hover
    // and nowhere else, so reading every visible row on every frame bought
    // nothing (change 208); a path is probed the first time it is hovered.
    let pointer = hover_pos.filter(|position| rect.contains(*position))?;
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
                        // Probed at the start of the next frame; ask for it
                        // rather than wait for the pointer to move.
                        ui.ctx().request_repaint();
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
        if span.contains(pointer) {
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
