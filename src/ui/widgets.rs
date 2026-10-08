use crate::prelude::*;
use crate::*;

/// The geometry every widget in the app inherits: one control height, one
/// button padding, one type scale. Set on the context rather than at the call
/// sites, because a control that picks its own height is exactly what makes a
/// row of them read as a row of accidents.
///
/// egui's defaults are drawn for a debug inspector — 12.5px body text, 2px of
/// vertical button padding, a scrollbar that takes a column of its own. None of
/// those are wrong there and all of them are wrong here.
pub(crate) fn apply_interface_metrics(ctx: &egui::Context) {
    use egui::{FontFamily, FontId, TextStyle};

    ctx.all_styles_mut(|style| {
        style.spacing.item_spacing = egui::vec2(SPACE_SM, SPACE_SM);
        style.spacing.button_padding = egui::vec2(10.0, 5.0);
        style.spacing.interact_size = egui::vec2(28.0, CONTROL_HEIGHT);
        style.spacing.indent = SPACE_LG;
        style.spacing.icon_width = 16.0;
        style.spacing.icon_width_inner = 9.0;
        style.spacing.icon_spacing = SPACE_SM;
        style.spacing.menu_margin = egui::Margin::same(6);
        style.spacing.window_margin = egui::Margin::same(16);
        style.spacing.combo_width = 160.0;
        style.spacing.text_edit_width = 320.0;
        style.spacing.tooltip_width = 420.0;
        // A scrollbar that reserves a column of its own puts a moving edge on
        // the right of every list. Floating it keeps the column still.
        style.spacing.scroll.floating = true;
        style.spacing.scroll.bar_width = 8.0;
        style.spacing.scroll.floating_allocated_width = 0.0;
        // The disclosure triangles in the project tree and the settings page
        // are not buttons, so they do not get a button's frame or its rule.
        style.visuals.collapsing_header_frame = false;
        style.visuals.indent_has_left_vline = false;
        // The scale `DESIGN.md` documents. egui only carries five slots, so the
        // rest of the scale is named at the call site through `RichText::size`.
        style.text_styles = [
            (
                TextStyle::Heading,
                FontId::new(20.0, FontFamily::Proportional),
            ),
            (TextStyle::Body, FontId::new(14.0, FontFamily::Proportional)),
            (
                TextStyle::Button,
                FontId::new(13.5, FontFamily::Proportional),
            ),
            (
                TextStyle::Small,
                FontId::new(12.0, FontFamily::Proportional),
            ),
            (
                TextStyle::Monospace,
                FontId::new(MONOSPACE_SIZE, FontFamily::Monospace),
            ),
        ]
        .into();
    });
}

/// The surface a group of related things sits on. One radius, one hairline, one
/// padding — a card is one object, and the only thing a call site chooses is
/// what goes inside it.
pub(crate) fn card_frame(palette: &Palette) -> egui::Frame {
    egui::Frame::new()
        .fill(palette.raised)
        .stroke(egui::Stroke::new(1.0, palette.border_subtle))
        .corner_radius(egui::CornerRadius::same(RADIUS_CARD))
        .inner_margin(egui::Margin::same(14))
}

/// A card that is itself the way into the thing it describes. A list row whose
/// only target is one small glyph at the far right makes a person aim at
/// something they cannot see from the name they just read, so the whole surface
/// takes the click and marks itself with the hover outline.
///
/// `UiBuilder::sense` is what makes that safe: it registers this row's rect
/// *behind* everything drawn inside it, so the buttons on the row still take
/// their own clicks. An `ui.interact` over the finished card would sit on top
/// of them and swallow every one.
///
/// Text selection is off inside a card like this. egui's labels sense a click
/// and a drag so their text can be selected, and a row is mostly labels — left
/// on, the title and the branch name would be the two places on the row where
/// tapping it does nothing.
pub(crate) fn clickable_card<R>(
    ui: &mut egui::Ui,
    palette: &Palette,
    id_salt: impl std::hash::Hash,
    clickable: bool,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    let scope = ui.scope_builder(
        egui::UiBuilder::new().id_salt(id_salt).sense(if clickable {
            egui::Sense::click()
        } else {
            egui::Sense::hover()
        }),
        |ui| {
            let hovered = clickable && ui.response().hovered();
            card_frame(palette)
                .stroke(egui::Stroke::new(
                    1.0,
                    if hovered {
                        palette.border
                    } else {
                        palette.border_subtle
                    },
                ))
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    if clickable {
                        ui.style_mut().interaction.selectable_labels = false;
                    }
                    add_contents(ui)
                })
                .inner
        },
    );
    let response = if clickable {
        scope
            .response
            .on_hover_cursor(egui::CursorIcon::PointingHand)
    } else {
        scope.response
    };
    if clickable {
        // Standard buttons take egui's `active` style while focused, which
        // already carries the strong outline. A custom row paints its own
        // frame, so it says the same thing by hand, or keyboard focus lands
        // on it invisibly.
        if let Some(stroke) = focus_stroke(response.has_focus(), palette) {
            ui.painter().rect_stroke(
                response.rect,
                egui::CornerRadius::same(RADIUS_CARD),
                stroke,
                egui::StrokeKind::Outside,
            );
        }
    }
    egui::InnerResponse::new(scope.inner, response)
}

/// The mark keyboard focus leaves on a custom-drawn row or card, or nothing.
///
/// A pure decision so the contrast test can pin the ink and a unit test can
/// pin both arms: `Some` while focused, `None` otherwise.
pub(crate) fn focus_stroke(has_focus: bool, palette: &Palette) -> Option<egui::Stroke> {
    has_focus.then(|| egui::Stroke::new(1.0, palette.border_strong))
}

/// The keyboard focus mark on a button: the same 1px ring at `RADIUS_CONTROL`
/// for every button, painted only while the button holds focus. It adds an
/// outline and nothing else, so the fill the button is drawn with stays as it is.
pub(crate) fn paint_button_focus(ui: &egui::Ui, response: &egui::Response, palette: &Palette) {
    if let Some(stroke) = focus_stroke(response.has_focus(), palette) {
        ui.painter().rect_stroke(
            response.rect,
            egui::CornerRadius::same(RADIUS_CONTROL),
            stroke,
            egui::StrokeKind::Inside,
        );
    }
}

/// One row of the ⌘K palette. The group is what the row is filed under and is
/// searched along with the label, so typing a project's name finds it whether
/// the person was thinking "project" or thinking of the name.
/// What a status means, rather than what it looks like. Naming the meaning
/// here keeps this function free of any theme: the palette in force when the
/// chip is drawn decides whether "needs you" is amber or ochre.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StatusTone {
    /// On its way somewhere, with nothing to do about it yet.
    Info,
    /// Doing work right now.
    Running,
    /// Finished its turn and ready for the next request.
    Idle,
    /// Waiting on the person reading the screen.
    Attention,
    /// Ended badly.
    Failed,
    /// Over, and nothing came of it either way.
    Neutral,
    /// Picked up from a terminal this app did not start, or stopped by hand.
    Recovered,
}

/// A path as a person writes it rather than as the filesystem stores it. Every
/// path in this app is under the home directory, so spelling that prefix out on
/// every row spends the width that the part telling two projects apart needs.
pub(crate) fn friendly_path(path: &Path) -> String {
    let display = path.display().to_string();
    match directories::UserDirs::new().map(|dirs| dirs.home_dir().display().to_string()) {
        Some(home) if !home.is_empty() && display.starts_with(&home) => {
            format!("~{}", &display[home.len()..])
        }
        _ => display,
    }
}

/// The 1px rule that separates two bands of content. Thinner and quieter than
/// egui's own separator, which is drawn for a debug panel.
pub(crate) fn hairline(ui: &mut egui::Ui, palette: &Palette) {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 1.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 0.0, palette.border_subtle);
}

/// How wide a page draws itself inside the window it was given.
///
/// The page takes the window less a gutter each side. A window somebody dragged
/// wider was dragged wider to hold more — a column of a fixed width answers that
/// by growing its own margins, which is the app declining to use the screen it
/// was handed. Nothing on these pages is prose read line after line: they are
/// cards, rows, and counts, and each of those reads better with room than
/// squeezed against its neighbour.
///
/// The gutter is a fraction of a narrow window rather than a constant, because
/// 26px off each side of a 400px window is a page that is mostly margin.
pub(crate) fn page_column_width(available_width: f32) -> f32 {
    let gutter = PAGE_GUTTER.min(available_width * 0.04).max(0.0);
    (available_width - gutter * 2.0).max(0.0)
}

/// A page's own header: what this page is, on the left, and what a person can
/// do to it, on the right, on one baseline. Every page uses it, so no page has
/// to invent where its title goes or how far the content below it starts.
pub(crate) fn page_header(
    ui: &mut egui::Ui,
    palette: &Palette,
    title: &str,
    subtitle: Option<&str>,
    actions: impl FnOnce(&mut egui::Ui),
) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.add_space(2.0);
            ui.label(
                RichText::new(title)
                    .size(20.0)
                    .strong()
                    .color(palette.text_strong),
            );
            if let Some(subtitle) = subtitle {
                ui.label(RichText::new(subtitle).size(12.5).color(palette.text_muted));
            }
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), actions);
    });
    ui.add_space(SPACE_MD);
    hairline(ui, palette);
    ui.add_space(SPACE_LG);
}

/// The one button on a page that starts work. There is never a second one:
/// two filled buttons in a row is two things claiming to be the answer.
pub(crate) fn primary_button(
    ui: &mut egui::Ui,
    palette: &Palette,
    icon: &str,
    label: &str,
) -> egui::Response {
    let response = ui.add(
        egui::Button::new(
            RichText::new(format!("{icon}  {label}"))
                .strong()
                .color(palette.on_accent),
        )
        .fill(palette.accent)
        .stroke(egui::Stroke::NONE)
        .min_size(egui::vec2(0.0, CONTROL_HEIGHT)),
    );
    paint_button_focus(ui, &response, palette);
    response
}

/// Everything that is not the primary action: a hairline at rest, a fill under
/// the pointer. This is the app's ordinary button and it is what `ui.button`
/// already draws, wrapped only to hold the shared height.
pub(crate) fn secondary_button(ui: &mut egui::Ui, label: impl Into<String>) -> egui::Response {
    ui.add(egui::Button::new(label.into()).min_size(egui::vec2(0.0, CONTROL_HEIGHT)))
}

/// The quietest control there is: wording alone until the pointer reaches it.
/// For the actions that belong beside a heading rather than under it.
pub(crate) fn quiet_button(
    ui: &mut egui::Ui,
    palette: &Palette,
    label: impl Into<String>,
) -> egui::Response {
    let response = ui.add(
        egui::Button::new(RichText::new(label.into()).color(palette.text_muted))
            .stroke(egui::Stroke::NONE)
            .min_size(egui::vec2(0.0, CONTROL_HEIGHT)),
    );
    paint_button_focus(ui, &response, palette);
    response
}

/// The restore's live status and its dismissal action occupy separate rows.
///
/// A long label followed by a right-to-left child in one horizontal layout can
/// spend the same pixels twice: the label claims the row before the child puts
/// its button at the right edge. Because the restore spinner requests constant
/// repaints, the doubled glyphs then visibly flicker. Keeping the paint rects
/// vertically disjoint makes the footer stable at every supported width.
pub(crate) fn restore_progress_footer(
    ui: &mut egui::Ui,
    palette: &Palette,
    elapsed_seconds: u64,
) -> (egui::Response, egui::Response) {
    let status = ui.add(
        egui::Label::new(
            RichText::new(tf!("ローカル履歴ファイルから抽出しています · 経過 {elapsed_seconds} 秒 · モデルは実行しないためトークンは消費しません", elapsed_seconds = elapsed_seconds))
            .size(11.5)
            .color(palette.text_muted),
        )
        .wrap(),
    );
    ui.add_space(SPACE_XS);
    let action = ui
        .with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            quiet_button(ui, palette, tr("バックグラウンドで続ける"))
        })
        .inner;
    (status, action)
}

/// The progress and dismiss action row for starting an agent CLI or terminal.
/// Separating the status label and right-aligned button across vertical rows
/// prevents pixel collision and text flicker under continuous spinner repainting.
pub(crate) fn launch_progress_footer(
    ui: &mut egui::Ui,
    palette: &Palette,
    elapsed_seconds: u64,
) -> (egui::Response, egui::Response) {
    let status = ui.add(
        egui::Label::new(
            RichText::new(tf!(
                "tmux セッションを準備しています · 経過 {elapsed_seconds} 秒 · 完了すると自動でターミナルが開きます",
                elapsed_seconds = elapsed_seconds
            ))
            .size(11.5)
            .color(palette.text_muted),
        )
        .wrap(),
    );
    ui.add_space(SPACE_XS);
    let action = ui
        .with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            quiet_button(ui, palette, tr("バックグラウンドで続ける"))
        })
        .inner;
    (status, action)
}

/// One tab of the toolbar's segmented navigation. The chosen one is marked by
/// the accent-tinted row surface and the accent ink, which is the same pairing
/// a selected session row uses — egui's own selection blue belongs to selected
/// *text*, and a page is not text.
pub(crate) fn nav_tab(
    ui: &mut egui::Ui,
    palette: &Palette,
    icon: &str,
    label: &str,
    selected: bool,
) -> egui::Response {
    let ink = if selected {
        palette.accent_text
    } else {
        palette.text_muted
    };
    let text = if icon.is_empty() {
        label.to_owned()
    } else {
        format!("{icon}  {label}")
    };
    let mut button = egui::Button::new(RichText::new(text).color(ink))
        .stroke(egui::Stroke::NONE)
        .min_size(egui::vec2(0.0, CONTROL_HEIGHT));
    if selected {
        button = button.fill(palette.row_selected);
    }
    let response = ui.add(button);
    paint_button_focus(ui, &response, palette);
    response
}

/// One tab of an in-page tab bar, marked by an accent rule along its bottom
/// edge that lands on the hairline under the row.
///
/// A filled chip would be a third kind of selected-thing in an app that already
/// has two — the toolbar's tinted nav and a selected list row — and a filled
/// chip beside a hairline button reads as a button that is stuck down. A rule
/// under a word is the mark every tool of this kind uses, and it is the only
/// one that says "this is where you are" rather than "this is pressed".
///
/// A tab can also say how much is behind it: 「変更 3」. The number is a plain
/// figure after the label, not a pill — the accent rule is already this row's
/// one mark — and it takes the accent colour only when `emphasise` asks and
/// there is something to count, so 「変更 0」 never calls for attention.
pub(crate) fn tab_item_with_count(
    ui: &mut egui::Ui,
    palette: &Palette,
    label: &str,
    count: Option<usize>,
    emphasise: bool,
    selected: bool,
) -> egui::Response {
    let ink = if selected {
        palette.text_strong
    } else {
        palette.text_muted
    };
    let text: egui::WidgetText = match count {
        None => RichText::new(label).color(ink).into(),
        Some(count) => {
            // A laid-out job keeps the fonts written into it; `Button` gives
            // its own `TextStyle::Button` only to plain text. Name it here, or
            // a tab with a count is drawn a size larger than one without.
            let style = ui.style();
            let mut job = egui::text::LayoutJob::default();
            RichText::new(label).color(ink).append_to(
                &mut job,
                style,
                egui::FontSelection::Style(egui::TextStyle::Button),
                egui::Align::Center,
            );
            let figure = if emphasise && count > 0 {
                palette.accent
            } else {
                ink
            };
            RichText::new(format!(" {count}")).color(figure).append_to(
                &mut job,
                style,
                egui::FontSelection::Style(egui::TextStyle::Button),
                egui::Align::Center,
            );
            job.into()
        }
    };
    let response = ui.add(
        egui::Button::new(text)
            .stroke(egui::Stroke::NONE)
            .min_size(egui::vec2(0.0, CONTROL_HEIGHT)),
    );
    paint_button_focus(ui, &response, palette);
    if selected {
        let rect = response.rect;
        let y = rect.bottom() + 2.5;
        ui.painter().line_segment(
            [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
            egui::Stroke::new(2.0, palette.accent),
        );
    }
    response
}

/// The rule a tab bar's marks sit on, and the space after it. Kept here so
/// every tab bar in the app ends the same distance above its content.
pub(crate) fn tab_bar_rule(ui: &mut egui::Ui, palette: &Palette) {
    ui.add_space(2.0);
    hairline(ui, palette);
    ui.add_space(SPACE_MD);
}

/// An icon-only action. The wording the icon replaces becomes its hover text,
/// so the button stays discoverable for anyone who does not recognise the glyph.
/// Square, frameless, and the same size everywhere — a toolbar is a row of
/// equal targets or it is a row of buttons that happen to be near each other.
pub(crate) fn icon_button(
    ui: &mut egui::Ui,
    palette: &Palette,
    icon: &str,
    tooltip: &str,
) -> egui::Response {
    let response = ui
        .add(
            egui::Button::new(RichText::new(icon).size(16.0))
                .stroke(egui::Stroke::NONE)
                .min_size(egui::vec2(CONTROL_HEIGHT, CONTROL_HEIGHT)),
        )
        .on_hover_text(tooltip);
    paint_button_focus(ui, &response, palette);
    response
}

/// The same control at the size dense surfaces use — session rows, list rows.
pub(crate) fn small_icon_button(
    ui: &mut egui::Ui,
    palette: &Palette,
    icon: &str,
    tooltip: &str,
) -> egui::Response {
    let response = ui
        .add(
            egui::Button::new(RichText::new(icon).size(14.0))
                .stroke(egui::Stroke::NONE)
                .min_size(egui::vec2(CONTROL_HEIGHT_SMALL, CONTROL_HEIGHT_SMALL)),
        )
        .on_hover_text(tooltip);
    paint_button_focus(ui, &response, palette);
    response
}

/// A button that keeps its wording but leads with the shared icon, for the
/// primary actions that are worth reading in full.
pub(crate) fn icon_text_button(ui: &mut egui::Ui, icon: &str, label: &str) -> egui::Response {
    ui.add(egui::Button::new(format!("{icon}  {label}")).min_size(egui::vec2(0.0, CONTROL_HEIGHT)))
}

/// The verb that changes a session's state — 停止, 再試行, 今すぐ開始 — with
/// its word visible outright. A session's state verb used to be a bare
/// pictogram in a row of utility glyphs, so the one action that does something
/// irreversible-shaped was the one that said nothing until hovered. The word is
/// the whole point; the icon only keeps the verb recognisable at a glance.
///
/// `danger` verbs keep the hairline at rest and take their colour only under
/// the pointer, so an idle screen stays quiet and the fill still marks exactly
/// one control: the one under the pointer.
pub(crate) fn verb_button(
    ui: &mut egui::Ui,
    palette: &Palette,
    icon: &str,
    label: &str,
    danger: bool,
) -> egui::Response {
    ui.scope(|ui| {
        if danger {
            // The wash is derived from the role rather than written as a
            // literal, the same way the vendor-tinted selection is.
            let wash = palette.danger.gamma_multiply(0.16);
            let pressed = palette.danger.gamma_multiply(0.28);
            let visuals = ui.visuals_mut();
            visuals.widgets.hovered.weak_bg_fill = wash;
            visuals.widgets.hovered.bg_fill = wash;
            visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, palette.danger);
            visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.5, palette.danger);
            visuals.widgets.active.weak_bg_fill = pressed;
            visuals.widgets.active.bg_fill = pressed;
            visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, palette.danger);
            visuals.widgets.active.fg_stroke = egui::Stroke::new(1.5, palette.danger);
        }
        let response = ui.add(
            egui::Button::new(
                RichText::new(format!("{icon}  {label}"))
                    .size(13.0)
                    .strong(),
            )
            .corner_radius(egui::CornerRadius::same(RADIUS_CONTROL))
            .min_size(egui::vec2(0.0, CONTROL_HEIGHT)),
        );
        // A plain verb already takes egui's own focus outline, which is the
        // same ring. A danger verb overwrites that outline with the danger
        // stroke above, so the shared ring is painted again over it.
        if danger {
            paint_button_focus(ui, &response, palette);
        }
        response
    })
    .inner
}

/// Render a status as icon + label in its own colour, with the explanation of
/// what that state means on hover.
pub(crate) fn status_chip(
    ui: &mut egui::Ui,
    view: &SessionStatusView,
    size: f32,
    palette: &Palette,
) {
    ui.add(egui::Label::new(
        RichText::new(format!("{} {}", view.icon, view.label))
            .size(size)
            .strong()
            .color(palette.status(view.tone)),
    ))
    .on_hover_text(&view.hint);
}

/// The dot between two pieces of metadata on the same line. Punctuation inside
/// a phrase rather than a pictogram standing in for one, so it comes from the
/// text face at the weight of the words beside it — the same rule the arrow in
/// `old → new` answers to.
pub(crate) fn meta_separator(ui: &mut egui::Ui, palette: &Palette) {
    ui.label(RichText::new("·").size(12.0).color(palette.text_faint));
}

/// One line of the machinery underneath a session: what it is called, and the
/// exact string a person would paste into a shell. The value is monospaced and
/// selectable, because the only reason to read it is to copy it.
/// The one click that turns a row into a command line.
///
/// The conversation ID beside this button is the whole identity of a CLI
/// session, and it is the thing a person compares two rows by — so the row shows
/// the ID. But an ID on its own is not something anybody can run: each CLI
/// resumes with its own verb and its own flag, and remembering that Codex takes
/// `resume` while Claude Code takes `--resume` is exactly the kind of thing this
/// app exists so that nobody has to. So the button hands over the command, with
/// the command as its hover text — a copy button that will not say what it
/// copied is one you have to paste somewhere else to read.
pub(crate) fn resume_copy_button(
    ui: &mut egui::Ui,
    palette: &Palette,
    provider: CliProvider,
    native_session_id: &str,
) {
    let command = native_resume_display(provider, native_session_id);
    if small_icon_button(
        ui,
        palette,
        ICON_COPY,
        &tf!("「{command}」をコピー", command = command),
    )
    .clicked()
    {
        ui.ctx().copy_text(command);
    }
}

pub(crate) fn plumbing_row(ui: &mut egui::Ui, palette: &Palette, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.add_sized(
            egui::vec2(110.0, 18.0),
            egui::Label::new(RichText::new(label).size(11.5).color(palette.text_faint)),
        );
        ui.add(
            egui::Label::new(
                RichText::new(value)
                    .size(11.5)
                    .monospace()
                    .color(palette.text_muted),
            )
            .truncate(),
        )
        .on_hover_text(value);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if small_icon_button(ui, palette, ICON_COPY, tr("コピー")).clicked() {
                ui.ctx().copy_text(value.to_owned());
            }
        });
    });
}

/// A message the app owes the person about something that just happened, or is
/// still happening. The edge carries the tone and the mark repeats it, so the
/// difference between "done" and "look at this" survives being the wrong colour
/// for the reader.
pub(crate) fn banner(
    ui: &mut egui::Ui,
    palette: &Palette,
    icon: &str,
    tone: Color32,
    add_contents: impl FnOnce(&mut egui::Ui),
) {
    card_frame(palette)
        .inner_margin(egui::Margin::symmetric(12, 10))
        .stroke(egui::Stroke::new(1.0, tone))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = SPACE_SM;
                ui.label(RichText::new(icon).size(15.0).color(tone));
                add_contents(ui);
            });
        });
}

/// What a list says when it is empty. A bare sentence in dimmed text reads as
/// something that failed to load; a mark and a centred line reads as a place
/// that is waiting for its first item.
pub(crate) fn empty_state(ui: &mut egui::Ui, palette: &Palette, icon: &str, message: &str) {
    card_frame(palette)
        .inner_margin(egui::Margin::symmetric(14, 30))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.vertical_centered(|ui| {
                ui.label(RichText::new(icon).size(26.0).color(palette.text_faint));
                ui.add_space(SPACE_SM);
                ui.label(RichText::new(message).color(palette.text_muted));
            });
        });
}

/// One number worth watching, with the word for what it counts above it. The
/// colour is spent only on a count that is not zero: an amber nought is an
/// alarm about nothing, and a page of them teaches a person to ignore amber.
pub(crate) fn metric_tile(
    ui: &mut egui::Ui,
    palette: &Palette,
    icon: &str,
    label: &str,
    value: usize,
    tone: Color32,
) -> egui::Response {
    let response = card_frame(palette)
        .inner_margin(egui::Margin::symmetric(14, 12))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.label(
                RichText::new(format!("{icon}  {label}"))
                    .size(12.0)
                    .color(palette.text_muted),
            );
            ui.add_space(2.0);
            ui.label(
                RichText::new(value.to_string())
                    .size(28.0)
                    .color(if value == 0 { palette.text_faint } else { tone }),
            );
        })
        .response;
    let response = response.interact(egui::Sense::click());
    if let Some(stroke) = focus_stroke(response.has_focus(), palette) {
        ui.painter().rect_stroke(
            response.rect,
            egui::CornerRadius::same(RADIUS_CARD),
            stroke,
            egui::StrokeKind::Outside,
        );
    }
    response
}

/// The theme picker's own preview: the roles a person is actually choosing
/// between, drawn in the theme they name rather than described in words. A
/// swatch says what "ハイコントラスト" does to a status colour in a way that a
/// sentence about contrast ratios cannot.
pub(crate) fn theme_swatches(ui: &mut egui::Ui, palette: &Palette) {
    let swatches = [
        (tr("背景"), palette.panel),
        (tr("カード"), palette.raised),
        (tr("文字"), palette.text),
        (tr("アクセント"), palette.accent),
        (tr("実行中"), palette.success),
        (tr("応答待ち"), palette.warning),
        (tr("失敗"), palette.danger),
    ];
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 3.0;
        for (name, colour) in swatches {
            let (rect, response) =
                ui.allocate_exact_size(egui::vec2(20.0, 14.0), egui::Sense::hover());
            ui.painter().rect(
                rect,
                2.0,
                colour,
                egui::Stroke::new(1.0, palette.border_subtle),
                egui::StrokeKind::Inside,
            );
            response.on_hover_text(name);
        }
    });
}

pub(crate) fn dropped_project_folders(dropped: &[egui::DroppedFile]) -> (Vec<PathBuf>, usize) {
    let folders = dropped
        .iter()
        .filter_map(|file| file.path.as_ref())
        .filter(|path| path.is_dir())
        .cloned()
        .collect::<Vec<_>>();
    let rejected = dropped.len().saturating_sub(folders.len());
    (folders, rejected)
}

pub(crate) fn readiness_row(
    ui: &mut egui::Ui,
    available: bool,
    label: &str,
    detail: &str,
    palette: &Palette,
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = SPACE_SM;
        ui.label(
            RichText::new(if available {
                ICON_AVAILABLE
            } else {
                ICON_UNAVAILABLE
            })
            .size(13.0)
            .strong()
            .color(if available {
                palette.success
            } else {
                palette.warning
            }),
        );
        ui.label(RichText::new(label).size(13.0).color(palette.text));
        ui.label(RichText::new(detail).size(12.0).color(palette.text_muted));
    });
}

/// One line of the settings list, led by the CLI's own mark where there is one.
/// This list is where a person works out which of three similarly worded rows
/// is the tool they meant, so it is where the marks earn the most — but tmux
/// and git have no mark, and their box is reserved anyway so every row on the
/// list keeps one left edge.
///
/// `extra` draws between the name and the verdict, right to left: what else
/// this tool's row has to say — its hook, its accounts, what to do when it is
/// missing — so none of it is somewhere else on the page (change 094).
pub(crate) fn tool_status_row(
    ui: &mut egui::Ui,
    agent: Option<&str>,
    label: &str,
    available: bool,
    purpose: &str,
    palette: &Palette,
    extra: impl FnOnce(&mut egui::Ui),
) {
    // The name and what it is for on the left, whether it is there on the
    // right: the column a person scans down is the answer, not the question.
    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let (icon, word, ink) = if available {
                (ICON_AVAILABLE, tr("使用可能"), palette.success)
            } else {
                (ICON_UNAVAILABLE, tr("未検出"), palette.warning)
            };
            ui.label(
                RichText::new(format!("{icon}  {word}"))
                    .size(12.5)
                    .strong()
                    .color(ink),
            );
            extra(ui);
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                agent_icon_slot(ui, agent, 16.0);
                ui.vertical(|ui| {
                    ui.label(RichText::new(label).size(13.5).color(palette.text));
                    ui.label(RichText::new(purpose).size(12.0).color(palette.text_muted));
                });
            });
        });
    });
}

/// One CLI's hook registration, as a cell in its tool row. Three verdicts
/// rather than two: "not registered" and "failed" are the same absence to a
/// reader who is only told whether it worked, and only one of them is
/// something they can act on — so both carry their reason on hover.
pub(crate) fn hook_cell(ui: &mut egui::Ui, state: &HookInstallState, palette: &Palette) {
    let (word, ink, reason) = match state {
        HookInstallState::Installed => (tr("フック 導入済み"), palette.success, None),
        HookInstallState::NotInstalled(reason) => {
            (tr("フック 未導入"), palette.text_faint, Some(reason))
        }
        HookInstallState::Failed(error) => (tr("フック 失敗"), palette.danger, Some(error)),
    };
    let response = ui.label(RichText::new(word).size(12.0).color(ink));
    if let Some(reason) = reason {
        response.on_hover_text(reason);
    }
}

/// Which state a session is in, as a value rather than as the word that gets
/// drawn. Run state alone is not the whole story — a terminal whose CLI is
/// blocked on a permission prompt is "running" exactly like one that is
/// mid-turn, and only one of them is waiting on the person reading the list —
/// so this is what the three inputs collapse to, and the mark, the token, the
/// colour, and the filter group all hang off it.
///
/// It exists as its own type so that the session list can count how many
/// sessions are in each filter group on every frame without building the two
/// `String`s a `SessionStatusView` carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SessionStatusKind {
    Stopping,
    Starting,
    Working,
    Waiting,
    Idle,
    Running,
    Blocked,
    Ready,
    Queued,
    Done,
    Failed,
    Stopped,
    Lost,
    Checking,
}

impl SessionStatusKind {
    /// Every variant, for the guards that check the mapping is total. Nothing
    /// the app draws iterates the states — a session is in one of them — so
    /// this list exists for the tests and is compiled only for them.
    #[cfg(test)]
    pub(crate) const ALL: [SessionStatusKind; 14] = [
        SessionStatusKind::Stopping,
        SessionStatusKind::Starting,
        SessionStatusKind::Working,
        SessionStatusKind::Waiting,
        SessionStatusKind::Idle,
        SessionStatusKind::Running,
        SessionStatusKind::Blocked,
        SessionStatusKind::Ready,
        SessionStatusKind::Queued,
        SessionStatusKind::Done,
        SessionStatusKind::Failed,
        SessionStatusKind::Stopped,
        SessionStatusKind::Lost,
        SessionStatusKind::Checking,
    ];

    pub(crate) fn icon(self) -> &'static str {
        match self {
            SessionStatusKind::Stopping => ICON_STOP,
            SessionStatusKind::Starting => ICON_STATUS_STARTING,
            SessionStatusKind::Working => ICON_STATUS_RUNNING,
            SessionStatusKind::Waiting => ICON_ATTENTION,
            SessionStatusKind::Idle => ICON_STATUS_IDLE,
            SessionStatusKind::Running => ICON_STATUS_RUNNING,
            SessionStatusKind::Blocked => ICON_ATTENTION,
            SessionStatusKind::Ready => ICON_STATUS_QUEUED,
            SessionStatusKind::Queued => ICON_PENDING,
            SessionStatusKind::Done => ICON_STATUS_DONE,
            SessionStatusKind::Failed => ICON_STATUS_FAILED,
            SessionStatusKind::Stopped => ICON_STOP,
            SessionStatusKind::Lost => ICON_STATUS_LOST,
            SessionStatusKind::Checking => ICON_STATUS_UNKNOWN,
        }
    }

    /// The word a person reads for this state: its group's, a message id.
    /// Fourteen states used to have fourteen English tokens beside four
    /// Japanese chip words, so one session had two names depending on where it
    /// was read. The state is still told apart by its colour and by the hint,
    /// which says which of the fourteen it is.
    pub(crate) fn label(self) -> &'static str {
        self.group().message_id()
    }

    pub(crate) fn tone(self) -> StatusTone {
        match self {
            SessionStatusKind::Stopping => StatusTone::Attention,
            SessionStatusKind::Starting => StatusTone::Info,
            SessionStatusKind::Working => StatusTone::Running,
            SessionStatusKind::Waiting => StatusTone::Attention,
            SessionStatusKind::Idle => StatusTone::Idle,
            SessionStatusKind::Running => StatusTone::Running,
            SessionStatusKind::Blocked => StatusTone::Attention,
            SessionStatusKind::Ready => StatusTone::Info,
            SessionStatusKind::Queued => StatusTone::Attention,
            SessionStatusKind::Done => StatusTone::Neutral,
            SessionStatusKind::Failed => StatusTone::Failed,
            SessionStatusKind::Stopped => StatusTone::Recovered,
            SessionStatusKind::Lost => StatusTone::Recovered,
            SessionStatusKind::Checking => StatusTone::Neutral,
        }
    }

    /// Which of the four things a person can do about this session. The match
    /// is exhaustive on purpose: a new state does not compile until someone
    /// says what a reader is supposed to do with it.
    pub(crate) fn group(self) -> SessionStatusGroup {
        match self {
            // Stopped on a question, or stopped for good on a prerequisite
            // that will not arrive. Either way nothing moves until a person
            // does something.
            SessionStatusKind::Waiting | SessionStatusKind::Blocked => SessionStatusGroup::NeedsYou,
            // Live, with nothing to do. The one this filter exists for.
            SessionStatusKind::Idle => SessionStatusGroup::Free,
            // In flight, including the not-yet-started: a queued session is
            // going to run without anyone's help, so it is no more actionable
            // than one that is mid-turn.
            SessionStatusKind::Starting
            | SessionStatusKind::Working
            | SessionStatusKind::Running
            | SessionStatusKind::Ready
            | SessionStatusKind::Queued
            | SessionStatusKind::Stopping
            | SessionStatusKind::Checking => SessionStatusGroup::Busy,
            // The terminal is gone. Its output may still be worth reading, but
            // no instruction can be given to it.
            SessionStatusKind::Done
            | SessionStatusKind::Failed
            | SessionStatusKind::Stopped
            | SessionStatusKind::Lost => SessionStatusGroup::Finished,
        }
    }
}

/// The four answers to "what can I do with this session right now", which is
/// the question the session list is filtered by. Fourteen states is the right
/// number to *report* and the wrong number to *choose between*.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SessionStatusGroup {
    NeedsYou,
    Free,
    Busy,
    Finished,
}

impl SessionStatusGroup {
    /// In the order the chips and the Home tiles are drawn, most actionable
    /// first: what needs you, what is moving, what is free, what is over.
    pub(crate) const GROUPS: [SessionStatusGroup; 4] = [
        SessionStatusGroup::NeedsYou,
        SessionStatusGroup::Busy,
        SessionStatusGroup::Free,
        SessionStatusGroup::Finished,
    ];

    /// Its slot in the `[bool; 4]` of selected groups.
    pub(crate) fn index(self) -> usize {
        match self {
            SessionStatusGroup::NeedsYou => 0,
            SessionStatusGroup::Busy => 1,
            SessionStatusGroup::Free => 2,
            SessionStatusGroup::Finished => 3,
        }
    }

    /// The chip's word. A message id, so the drawing code translates it — a
    /// `const` cannot call `tr`.
    pub(crate) fn message_id(self) -> &'static str {
        match self {
            SessionStatusGroup::NeedsYou => "要対応",
            SessionStatusGroup::Free => "待機中",
            SessionStatusGroup::Busy => "実行中",
            SessionStatusGroup::Finished => "終了",
        }
    }

    pub(crate) fn tone(self) -> StatusTone {
        match self {
            SessionStatusGroup::NeedsYou => StatusTone::Attention,
            SessionStatusGroup::Free => StatusTone::Idle,
            SessionStatusGroup::Busy => StatusTone::Running,
            SessionStatusGroup::Finished => StatusTone::Neutral,
        }
    }

    /// What the chip says on hover: which states it will show.
    pub(crate) fn hint_id(self) -> &'static str {
        match self {
            SessionStatusGroup::NeedsYou => "回答待ち・前提が壊れて止まっているセッション",
            SessionStatusGroup::Free => "起動していて、次の依頼を待っているセッション",
            SessionStatusGroup::Busy => "実行中・起動中・順番待ちのセッション",
            SessionStatusGroup::Finished => "ターミナルが終了したセッション",
        }
    }
}

/// Everything one session's state needs at a glance: a glyph, a short label,
/// the colour they share, and the sentence that says what the state means.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SessionStatusView {
    pub(crate) kind: SessionStatusKind,
    pub(crate) icon: &'static str,
    pub(crate) label: String,
    pub(crate) hint: String,
    pub(crate) tone: StatusTone,
}

/// Which state a session is in, and nothing else. No `String` is built here,
/// which is what lets the list count its filter groups every frame.
pub(crate) fn session_status_kind(
    sessions: &[Session],
    session: &Session,
    cancellation_pending: bool,
    activity: Option<AgentActivity>,
) -> SessionStatusKind {
    if cancellation_pending {
        return SessionStatusKind::Stopping;
    }
    match session.status {
        SessionStatus::Starting => SessionStatusKind::Starting,
        SessionStatus::Active => match activity {
            Some(AgentActivity::Working) => SessionStatusKind::Working,
            Some(AgentActivity::AwaitingInput) => SessionStatusKind::Waiting,
            // The CLI is up and has nothing to do: the state every agent
            // terminal calls idle. IDLE names the session's own state,
            // leaving the hint to describe what the reader may do next.
            Some(AgentActivity::Idle) => SessionStatusKind::Idle,
            None => SessionStatusKind::Running,
        },
        SessionStatus::Queued => {
            if blocking_dependency(sessions, session).is_some() {
                SessionStatusKind::Blocked
            } else if dependencies_finished(sessions, &session.depends_on) {
                SessionStatusKind::Ready
            } else {
                SessionStatusKind::Queued
            }
        }
        SessionStatus::Exited => SessionStatusKind::Done,
        SessionStatus::Failed => SessionStatusKind::Failed,
        SessionStatus::Cancelled => SessionStatusKind::Stopped,
        SessionStatus::Lost => SessionStatusKind::Lost,
        SessionStatus::Unknown => SessionStatusKind::Checking,
    }
}

pub(crate) fn session_status_view(
    sessions: &[Session],
    session: &Session,
    cancellation_pending: bool,
    activity: Option<AgentActivity>,
) -> SessionStatusView {
    let kind = session_status_kind(sessions, session, cancellation_pending, activity);
    let hint = match kind {
        SessionStatusKind::Stopping => tr("ターミナルの終了待ち").to_owned(),
        SessionStatusKind::Starting => tr("tmux と CLI を準備中").to_owned(),
        SessionStatusKind::Working => tr("応答を生成中").to_owned(),
        SessionStatusKind::Waiting => tr("ターミナルを開いて回答してください").to_owned(),
        SessionStatusKind::Idle => tr("次の依頼を待っています").to_owned(),
        SessionStatusKind::Running => tr("ターミナル稼働中").to_owned(),
        // The only hint that is not a constant: it names the prerequisite,
        // which is data, and is why `hint` lives on the view and not on the
        // kind.
        SessionStatusKind::Blocked => match blocking_dependency_label(sessions, session) {
            Some(parent) => tf!("前提「{parent}」が完了しませんでした", parent = parent),
            None => tr("前提の完了待ち").to_owned(),
        },
        SessionStatusKind::Ready => tr("前提が完了しました").to_owned(),
        SessionStatusKind::Queued => tr("前提の完了待ち").to_owned(),
        SessionStatusKind::Done => tr("正常終了").to_owned(),
        SessionStatusKind::Failed => tr("異常終了。出力を確認してください").to_owned(),
        SessionStatusKind::Stopped => tr("停止操作による終了").to_owned(),
        SessionStatusKind::Lost => tr("ターミナルが残っておらず、結果を確認できません").to_owned(),
        SessionStatusKind::Checking => tr("tmux を確認中").to_owned(),
    };
    SessionStatusView {
        kind,
        icon: kind.icon(),
        // Translated once here, so every surface that draws the view — chip,
        // row, card, palette — reads the same word in the same language.
        label: tr(kind.label()).to_owned(),
        hint,
        tone: kind.tone(),
    }
}

/// Decodes one of the PNGs bundled into the binary. Every mark the app draws as
/// a picture rather than as a glyph comes through here: the three vendor marks
/// and Operon's own.
pub(crate) fn decode_png_mark(bytes: &[u8]) -> Option<egui::ColorImage> {
    let decoded = image::load_from_memory_with_format(bytes, image::ImageFormat::Png)
        .ok()?
        .into_rgba8();
    let size = [decoded.width() as usize, decoded.height() as usize];
    Some(egui::ColorImage::from_rgba_unmultiplied(
        size,
        decoded.as_raw(),
    ))
}

/// Decodes and uploads each mark once, then keeps the handle in egui's own
/// frame-to-frame store. `load_texture` hands a fresh copy to the GPU on every
/// call, so doing this inline would upload one texture per drawn mark per
/// frame. A 128px mark drawn at 12pt needs mipmaps to stay readable; without
/// them the minified sample picks single texels and the mark sparkles.
///
/// `name` is both the cache key and the name egui uploads under, so it has to
/// be unique per asset — not per place the asset is drawn.
pub(crate) fn png_texture(
    ctx: &egui::Context,
    name: &'static str,
    bytes: &'static [u8],
) -> Option<egui::TextureHandle> {
    let id = egui::Id::new(("png-mark", name));
    if let Some(texture) = ctx.data(|data| data.get_temp::<egui::TextureHandle>(id)) {
        return Some(texture);
    }
    let texture = ctx.load_texture(
        name,
        decode_png_mark(bytes)?,
        egui::TextureOptions::LINEAR.with_mipmap_mode(Some(egui::TextureFilter::Linear)),
    );
    ctx.data_mut(|data| data.insert_temp(id, texture.clone()));
    Some(texture)
}

/// Decode an arbitrary supported image (PNG, JPEG, GIF, WebP, BMP, ICO) into ColorImage.
pub(crate) fn decode_image_bytes(bytes: &[u8]) -> Option<egui::ColorImage> {
    let decoded = image::load_from_memory(bytes).ok()?.into_rgba8();
    let size = [decoded.width() as usize, decoded.height() as usize];
    Some(egui::ColorImage::from_rgba_unmultiplied(
        size,
        decoded.as_raw(),
    ))
}

/// Get or load an egui texture for an open document image, keyed by document ID and byte size.
pub(crate) fn document_image_texture(
    ctx: &egui::Context,
    doc_id: &DocumentId,
    bytes: &[u8],
) -> Option<egui::TextureHandle> {
    let id = egui::Id::new(("doc-image-texture", doc_id, bytes.len()));
    if let Some(texture) = ctx.data(|data| data.get_temp::<egui::TextureHandle>(id)) {
        return Some(texture);
    }
    let color_image = decode_image_bytes(bytes)?;
    let texture = ctx.load_texture(
        format!("doc-img-{}", doc_id.path.display()),
        color_image,
        egui::TextureOptions::LINEAR,
    );
    ctx.data_mut(|data| data.insert_temp(id, texture.clone()));
    Some(texture)
}

/// Operon's own mark, as a `size`-point square. This is the artwork the bundle
/// icon draws — the same ring the Dock shows — with the icon's dark plate keyed
/// out, so it sits on the surface it is drawn on instead of putting a tile of
/// its own on a light theme.
///
/// It is a picture and not an `ICON_*` constant on purpose: the glyph
/// vocabulary is for actions, drawn in one stroke weight and recoloured per
/// theme, and a logo is neither. Returns `None` only if the bundled PNG will
/// not decode, which every caller has to answer for rather than draw a gap.
pub(crate) fn brand_mark_image(ctx: &egui::Context, size: f32) -> Option<egui::Image<'static>> {
    // The key names the asset, not the product: a literal opening with
    // `MANAGED_TMUX_PREFIX` is refused outside `src/config.rs` by
    // `the_session_prefix_is_written_in_exactly_one_place`, which does not care
    // that this one could never name a tmux session.
    let texture = png_texture(ctx, "brand-mark", BRAND_MARK)?;
    Some(egui::Image::new((texture.id(), egui::vec2(size, size))))
}

pub(crate) fn code_block(ui: &mut egui::Ui, text: &str) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.add(
            egui::Label::new(
                RichText::new(if text.trim().is_empty() {
                    tr("作業ツリーに変更はありません。")
                } else {
                    text
                })
                .text_style(TextStyle::Monospace),
            )
            .wrap(),
        );
    });
}
