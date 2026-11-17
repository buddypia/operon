//! One instruction to the idle agents of a project that the person ticks
//! (changes 282 and 306): the sheet the command palette opens, who may be
//! chosen, and the send.

use crate::app::waiting_reply::reply_enter_sends;
use crate::*;

/// The open sheet: what has been typed, whether its field still has to take
/// the keyboard, and the sessions ticked. Nobody is ticked on opening (change
/// 306): an agent that has only just started reads as idle, and a line it was
/// never meant for would become its first instruction.
#[derive(Default)]
pub(crate) struct BroadcastSheet {
    pub(crate) text: String,
    pub(crate) focus: bool,
    pub(crate) chosen: Vec<Uuid>,
}

/// How many recipient rows show before the list scrolls.
pub(crate) const BROADCAST_VISIBLE_ROWS: usize = 6;

impl OperonApp {
    pub(crate) fn open_broadcast_sheet(&mut self) {
        self.broadcast_sheet = Some(BroadcastSheet {
            text: String::new(),
            focus: true,
            chosen: Vec::new(),
        });
    }

    /// The project the sheet speaks to: the workspace's, else the selected.
    pub(crate) fn broadcast_project(&self) -> Option<&Project> {
        let id = self.workspace_project.or(self.selected_project)?;
        self.store.projects.iter().find(|project| project.id == id)
    }

    /// Who may be chosen: that project's sessions in アイドル — an agent that
    /// has finished its turn or has not been asked anything yet (change 308) —
    /// which only a running terminal's can be. An agent that
    /// works, waits on a prompt, or has ended is left alone.
    pub(crate) fn broadcast_targets(&self) -> Vec<Uuid> {
        let Some(project) = self.broadcast_project().map(|project| project.id) else {
            return Vec::new();
        };
        self.store
            .sessions
            .iter()
            .filter(|session| {
                session.project_id == project
                    && self.session_status_kind_for(session).group()
                        == Some(SessionStatusGroup::Free)
            })
            .map(|session| session.id)
            .collect()
    }

    /// Who an instruction reaches: the targets that were ticked, in list
    /// order. A ticked session that has started working since is not one.
    pub(crate) fn broadcast_recipients(&self, chosen: &[Uuid]) -> Vec<Uuid> {
        let mut targets = self.broadcast_targets();
        targets.retain(|id| chosen.contains(id));
        targets
    }

    /// Types `text` and Enter into every recipient's terminal and says how
    /// many it reached. Nothing is sent, and nothing said, for an empty text
    /// or with nobody chosen and idle.
    pub(crate) fn send_broadcast(&mut self, text: &str, chosen: &[Uuid]) -> usize {
        let text = text.trim();
        if text.is_empty() {
            return 0;
        }
        let targets = self.broadcast_recipients(chosen);
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
        // `broadcast_recipients` without a second pass over the sessions.
        let recipients: Vec<Uuid> = targets
            .iter()
            .copied()
            .filter(|id| sheet.chosen.contains(id))
            .collect();
        let every_chosen = !targets.is_empty() && recipients.len() == targets.len();
        let project = self
            .broadcast_project()
            .map(|project| project.name.clone())
            .unwrap_or_default();
        let mut close = false;
        let mut send = false;
        // Applied after the rows are drawn, which borrow the sessions.
        let mut toggled = None;
        let mut choose_all = None;
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
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(tf!(
                                        "{p0} · 選択 {p1} / {p2} 件",
                                        p0 = project,
                                        p1 = recipients.len(),
                                        p2 = targets.len()
                                    ))
                                    .color(palette.text_muted),
                                );
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        let label = if every_chosen {
                                            tr("選択を解除")
                                        } else {
                                            tr("すべて選択")
                                        };
                                        if quiet_button(ui, palette, label).clicked() {
                                            choose_all = Some(!every_chosen);
                                        }
                                    },
                                );
                            });
                        }
                        let row = ui.spacing().interact_size.y + ui.spacing().item_spacing.y;
                        egui::ScrollArea::vertical()
                            .id_salt("broadcast-recipients")
                            .auto_shrink([false, true])
                            .max_height(row * BROADCAST_VISIBLE_ROWS as f32)
                            .show(ui, |ui| {
                                for session in targets.iter().filter_map(|id| {
                                    self.store.sessions.iter().find(|session| session.id == *id)
                                }) {
                                    ui.horizontal(|ui| {
                                        let mut ticked = sheet.chosen.contains(&session.id);
                                        let boxed = ui.checkbox(&mut ticked, "").changed();
                                        agent_chip(ui, &session.agent, 12.0, palette);
                                        let title = ui
                                            .add(
                                                egui::Label::new(session_list_title(
                                                    session,
                                                    self.session_prompt_turns
                                                        .get(&session.id)
                                                        .and_then(|turns| turns.first())
                                                        .map(|turn| turn.prompt.as_str()),
                                                ))
                                                .truncate()
                                                .sense(egui::Sense::click()),
                                            )
                                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                                            .clicked();
                                        if boxed || title {
                                            toggled = Some(session.id);
                                        }
                                    });
                                }
                            });
                        ui.add_space(SPACE_SM);
                        let field = ui.add_enabled(
                            !targets.is_empty(),
                            egui::TextEdit::singleline(&mut sheet.text)
                                .hint_text(tr("選んだ相手に送る内容を入力"))
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
                            let ready = !recipients.is_empty() && !sheet.text.trim().is_empty();
                            ui.add_enabled_ui(ready, |ui| {
                                send |= primary_button(
                                    ui,
                                    palette,
                                    ICON_START,
                                    &tf!("{p0} 件に送信", p0 = recipients.len()),
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
        if let Some(id) = toggled {
            if sheet.chosen.contains(&id) {
                sheet.chosen.retain(|chosen| *chosen != id);
            } else {
                sheet.chosen.push(id);
            }
        }
        match choose_all {
            Some(true) => sheet.chosen = targets,
            Some(false) => sheet.chosen.clear(),
            None => {}
        }
        // The sheet stays, with its text, when there was nothing to send —
        // and the field keeps the keyboard, which the Enter took from it.
        if send {
            if self.send_broadcast(&sheet.text, &sheet.chosen) > 0 {
                return;
            }
            sheet.focus = true;
        }
        self.broadcast_sheet = Some(sheet);
    }
}
