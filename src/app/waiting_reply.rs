//! The 要対応 section and the reply a person can send from one of its rows
//! without opening the session (change 279).

use crate::*;

impl OperonApp {
    /// The 要対応 section at the head of the session list: the queue, longest
    /// waiting first, each row saying how long and — when the hooks said it —
    /// what the agent last said. Returns how many rows it drew.
    pub(crate) fn ui_pinned_sessions(&mut self, ui: &mut egui::Ui, sessions: &[Session]) -> usize {
        let palette = self.store.theme.palette();
        let ink = palette.status(SessionStatusGroup::NeedsYou.tone());
        let now = Instant::now();
        let count = self
            .attention_order
            .iter()
            .filter(|id| {
                sessions
                    .iter()
                    .any(|session| session.id == **id && self.session_matches_filters(session))
            })
            .count();
        if count == 0 {
            return 0;
        }
        let mut open = None;
        let mut reply = None;
        // Out of `self` for the pass, as `track_attention` takes its map: the
        // rows borrow the app while a field edits its draft.
        let mut drafts = std::mem::take(&mut self.waiting_reply_drafts);
        // A card of its own, like each project below it (change 163).
        card_frame(palette)
            .inner_margin(egui::Margin::same(SPACE_SM as i8))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = SPACE_XS;
                    ui.label(RichText::new(tr("要対応")).size(11.5).strong().color(ink));
                    ui.label(
                        RichText::new(count.to_string())
                            .size(11.5)
                            .color(palette.text_muted),
                    );
                });
                // By index: the queue is read while rows borrow the app, and copying
                // it to iterate would allocate every frame.
                for index in 0..self.attention_order.len() {
                    let session_id = self.attention_order[index];
                    let Some(session) = sessions.iter().find(|session| {
                        session.id == session_id && self.session_matches_filters(session)
                    }) else {
                        continue;
                    };
                    let title = session_list_title(
                        session,
                        self.session_prompt_turns
                            .get(&session_id)
                            .and_then(|turns| turns.first())
                            .map(|turn| turn.prompt.as_str()),
                    );
                    let project = self
                        .store
                        .projects
                        .iter()
                        .find(|project| project.id == session.project_id)
                        .map(|project| project.name.as_str())
                        .unwrap_or_default();
                    let meta = match session.branch.as_deref() {
                        Some(branch) => format!("{project} · {ICON_BRANCH} {branch}"),
                        None => format!("{ICON_PROJECT} {project}"),
                    };
                    let wait = self
                        .attention_wait(session_id, now)
                        .map(attention_wait_label)
                        .unwrap_or_default();
                    let line = self.session_attention_line(session_id);
                    let selected = self.selected_session == Some(session_id);
                    let row = ui.scope_builder(
                        egui::UiBuilder::new()
                            .id_salt(("pinned-session", session_id))
                            .sense(egui::Sense::click()),
                        |ui| {
                            // A selectable label takes the click for itself, and the
                            // row would then open only from its padding.
                            ui.style_mut().interaction.selectable_labels = false;
                            egui::Frame::default()
                                .fill(if selected {
                                    palette.row_selected
                                } else {
                                    Color32::TRANSPARENT
                                })
                                .corner_radius(egui::CornerRadius::same(RADIUS_CONTROL))
                                .inner_margin(egui::Margin::symmetric(6, 5))
                                .show(ui, |ui| {
                                    ui.set_min_width(ui.available_width());
                                    ui.spacing_mut().item_spacing.y = 2.0;
                                    ui.horizontal(|ui| {
                                        ui.spacing_mut().item_spacing.x = SPACE_XS + 2.0;
                                        let (dot, _) = ui.allocate_exact_size(
                                            egui::vec2(8.0, 8.0),
                                            egui::Sense::hover(),
                                        );
                                        ui.painter().circle_filled(dot.center(), 4.0, ink);
                                        // Ringed on the selected row, for the
                                        // same reason as the session column's
                                        // dot (change 246).
                                        if selected {
                                            ui.painter().circle_stroke(
                                                dot.center(),
                                                4.0,
                                                egui::Stroke::new(1.0, palette.selected_ink()),
                                            );
                                        }
                                        agent_chip(ui, &session.agent, 12.0, palette);
                                        let title_width = (ui.available_width() - 56.0).max(40.0);
                                        ui.allocate_ui_with_layout(
                                            egui::vec2(title_width, 20.0),
                                            egui::Layout::left_to_right(egui::Align::Center),
                                            |ui| {
                                                ui.add(
                                                    egui::Label::new(
                                                        RichText::new(&title)
                                                            .strong()
                                                            .size(13.5)
                                                            .color(if selected {
                                                                palette.selected_ink()
                                                            } else {
                                                                palette.text_strong
                                                            }),
                                                    )
                                                    .truncate(),
                                                );
                                            },
                                        );
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                ui.label(RichText::new(wait).size(11.5).color(
                                                    if selected {
                                                        palette.selected_ink()
                                                    } else {
                                                        ink
                                                    },
                                                ));
                                            },
                                        );
                                    });
                                    ui.add(
                                        egui::Label::new(RichText::new(meta).size(12.0).color(
                                            if selected {
                                                palette.selected_secondary_ink()
                                            } else {
                                                palette.text_muted
                                            },
                                        ))
                                        .truncate(),
                                    );
                                    if let Some(line) = line {
                                        let mut job = egui::text::LayoutJob::single_section(
                                            line.to_owned(),
                                            egui::TextFormat {
                                                font_id: egui::FontId::proportional(12.0),
                                                color: if selected {
                                                    palette.selected_secondary_ink()
                                                } else {
                                                    palette.text
                                                },
                                                ..Default::default()
                                            },
                                        );
                                        job.wrap = egui::text::TextWrapping {
                                            max_width: ui.available_width(),
                                            max_rows: 2,
                                            break_anywhere: true,
                                            overflow_character: Some('…'),
                                        };
                                        ui.add(egui::Label::new(job));
                                    }
                                    if session.status == SessionStatus::Active
                                        && reply_field(ui, drafts.entry(session_id).or_default())
                                    {
                                        reply = Some(session_id);
                                    }
                                });
                        },
                    );
                    if row.response.clicked() {
                        open = Some(session_id);
                    }
                }
            });
        if let Some(session_id) = reply {
            if let Some(text) = drafts.remove(&session_id) {
                self.send_waiting_reply(session_id, &text);
            }
        }
        self.waiting_reply_drafts = drafts;
        if let Some(session_id) = open {
            self.open_waiting_session(session_id);
        }
        ui.add_space(SPACE_SM);
        count
    }

    /// Types `text` and Enter into a waiting session's terminal, as its pane
    /// would. Says so only when there was something to send and a running
    /// terminal to send it to.
    pub(crate) fn send_waiting_reply(&mut self, session_id: Uuid, text: &str) -> bool {
        let text = text.trim();
        let running = self
            .store
            .sessions
            .iter()
            .any(|session| session.id == session_id && session.status == SessionStatus::Active);
        if text.is_empty() || !running {
            return false;
        }
        self.queue_terminal_input(
            session_id,
            vec![
                TerminalInput::Text(text.to_owned()),
                TerminalInput::Key("Enter".into()),
            ],
        );
        self.notice_briefly(tr("返信を送りました"));
        true
    }
}

/// Whether an Enter among `events` is the person's to send with. One Enter
/// belongs to each conversion the IME confirmed in the frame, as in
/// `terminal_input_events`: the Enter that ends 「はい」's conversion is not
/// the one that sends it.
pub(crate) fn reply_enter_sends(events: &[egui::Event]) -> bool {
    let confirmed = events
        .iter()
        .filter(|event| {
            matches!(event, egui::Event::Ime(egui::ImeEvent::Commit(text)) if !text.is_empty())
        })
        .count();
    let pressed = events
        .iter()
        .filter(|event| {
            matches!(
                event,
                egui::Event::Key {
                    key: egui::Key::Enter,
                    pressed: true,
                    ..
                }
            )
        })
        .count();
    pressed > confirmed
}

/// The row's one-line reply. True on the frame its Enter sends.
fn reply_field(ui: &mut egui::Ui, draft: &mut String) -> bool {
    let response = ui.add(
        egui::TextEdit::singleline(draft)
            .hint_text(tr("返信を入力して Enter"))
            .font(egui::FontId::proportional(12.0))
            .desired_width(f32::INFINITY),
    );
    if !response.lost_focus() || !ui.input(|input| input.key_pressed(egui::Key::Enter)) {
        return false;
    }
    if ui.input(|input| reply_enter_sends(&input.events)) {
        return true;
    }
    // The Enter confirmed a conversion: the person is still typing.
    response.request_focus();
    false
}
