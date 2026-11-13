//! One instruction to every idle agent of a project (change 282): the sheet
//! the command palette opens, who it will reach, and the send.

use crate::app::waiting_reply::reply_enter_sends;
use crate::*;

/// The open sheet: what has been typed, and whether its field still has to
/// take the keyboard.
#[derive(Default)]
pub(crate) struct BroadcastSheet {
    pub(crate) text: String,
    pub(crate) focus: bool,
}

/// How many recipients the sheet names before it counts the rest.
pub(crate) const BROADCAST_NAMED: usize = 6;

impl OperonApp {
    pub(crate) fn open_broadcast_sheet(&mut self) {
        self.broadcast_sheet = Some(BroadcastSheet {
            text: String::new(),
            focus: true,
        });
    }

    /// The project the sheet speaks to: the workspace's, else the selected.
    pub(crate) fn broadcast_project(&self) -> Option<&Project> {
        let id = self.workspace_project.or(self.selected_project)?;
        self.store.projects.iter().find(|project| project.id == id)
    }

    /// Who an instruction would reach: that project's sessions whose agent
    /// has finished its turn, which only a running terminal's can have. An
    /// agent that works, waits on a prompt, or has ended is left alone.
    pub(crate) fn broadcast_targets(&self) -> Vec<Uuid> {
        let Some(project) = self.broadcast_project().map(|project| project.id) else {
            return Vec::new();
        };
        self.store
            .sessions
            .iter()
            .filter(|session| {
                session.project_id == project
                    && self.session_status_kind_for(session) == SessionStatusKind::Idle
            })
            .map(|session| session.id)
            .collect()
    }

    /// Types `text` and Enter into every target's terminal and says how many
    /// it reached. Nothing is sent, and nothing said, for an empty text or
    /// with nobody idle.
    pub(crate) fn send_broadcast(&mut self, text: &str) -> usize {
        let text = text.trim();
        if text.is_empty() {
            return 0;
        }
        let targets = self.broadcast_targets();
        for session_id in &targets {
            self.queue_terminal_input(
                *session_id,
                vec![
                    TerminalInput::Text(text.to_owned()),
                    TerminalInput::Key("Enter".into()),
                ],
            );
        }
        if !targets.is_empty() {
            self.notice_briefly(tf!(
                "{p0} 件のエージェントに送信しました",
                p0 = targets.len()
            ));
        }
        targets.len()
    }

    pub(crate) fn ui_broadcast_sheet(&mut self, ctx: &egui::Context) {
        let Some(mut sheet) = self.broadcast_sheet.take() else {
            return;
        };
        let palette = self.store.theme.palette();
        let screen = ctx.content_rect();
        egui::Area::new(egui::Id::unique("broadcast-sheet-scrim"))
            .order(egui::Order::Middle)
            .fixed_pos(screen.min)
            .show(ctx, |ui| {
                // Takes the click and does nothing: a stray click must not
                // throw away what somebody was typing.
                let (rect, _) = ui.allocate_exact_size(screen.size(), egui::Sense::click());
                ui.painter().rect_filled(rect, 0.0, palette.scrim);
            });
        if ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
            return;
        }
        let targets = self.broadcast_targets();
        let project = self
            .broadcast_project()
            .map(|project| project.name.clone())
            .unwrap_or_default();
        let mut close = false;
        let mut send = false;
        const WIDTH: f32 = 460.0;
        egui::Area::new(egui::Id::unique("broadcast-sheet"))
            .order(egui::Order::Foreground)
            .fixed_pos(egui::pos2(
                screen.center().x - WIDTH / 2.0 - SPACE_LG,
                screen.top() + screen.height() * 0.18,
            ))
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(palette.raised)
                    .stroke(egui::Stroke::new(1.0_f32, palette.border))
                    .corner_radius(egui::CornerRadius::same(RADIUS_WINDOW))
                    .inner_margin(egui::Margin::same(SPACE_LG as i8))
                    .shadow(ui.style().visuals.popup_shadow)
                    .show(ui, |ui| {
                        ui.set_width(WIDTH);
                        ui.label(
                            RichText::new(tr("アイドル中のエージェントへ送信"))
                                .size(16.0)
                                .strong()
                                .color(palette.text_strong),
                        );
                        ui.add_space(SPACE_XS);
                        if targets.is_empty() {
                            ui.label(
                                RichText::new(tr("アイドル中のエージェントはいません"))
                                    .color(palette.text_muted),
                            );
                        } else {
                            ui.label(
                                RichText::new(tf!(
                                    "{p0} · 送信先 {p1} 件",
                                    p0 = project,
                                    p1 = targets.len()
                                ))
                                .color(palette.text_muted),
                            );
                        }
                        for session in targets.iter().take(BROADCAST_NAMED).filter_map(|id| {
                            self.store.sessions.iter().find(|session| session.id == *id)
                        }) {
                            ui.horizontal(|ui| {
                                agent_chip(ui, &session.agent, 12.0, palette);
                                ui.add(
                                    egui::Label::new(session_list_title(
                                        session,
                                        self.session_prompt_turns
                                            .get(&session.id)
                                            .and_then(|turns| turns.first())
                                            .map(|turn| turn.prompt.as_str()),
                                    ))
                                    .truncate(),
                                );
                            });
                        }
                        if targets.len() > BROADCAST_NAMED {
                            ui.label(
                                RichText::new(tf!(
                                    "ほか {p0} 件",
                                    p0 = targets.len() - BROADCAST_NAMED
                                ))
                                .color(palette.text_muted),
                            );
                        }
                        ui.add_space(SPACE_SM);
                        let field = ui.add_enabled(
                            !targets.is_empty(),
                            egui::TextEdit::singleline(&mut sheet.text)
                                .hint_text(tr("全員に送る内容を入力"))
                                .desired_width(f32::INFINITY),
                        );
                        if std::mem::take(&mut sheet.focus) {
                            field.request_focus();
                        }
                        if field.lost_focus()
                            && ui.input(|input| input.key_pressed(egui::Key::Enter))
                        {
                            if ui.input(|input| reply_enter_sends(&input.events)) {
                                send = true;
                            } else {
                                // The Enter confirmed a conversion.
                                field.request_focus();
                            }
                        }
                        ui.add_space(SPACE_SM);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let ready = !targets.is_empty() && !sheet.text.trim().is_empty();
                            ui.add_enabled_ui(ready, |ui| {
                                send |= primary_button(
                                    ui,
                                    palette,
                                    ICON_START,
                                    &tf!("{p0} 件に送信", p0 = targets.len()),
                                )
                                .clicked();
                            });
                            close = secondary_button(ui, tr("キャンセル")).clicked();
                        });
                    });
            });
        if close {
            return;
        }
        // The sheet stays, with its text, when there was nothing to send.
        if send && self.send_broadcast(&sheet.text) > 0 {
            return;
        }
        self.broadcast_sheet = Some(sheet);
    }
}
