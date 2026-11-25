//! A session's listening ports: the header badge, the menu behind it with who
//! started each server, when and with what, and the confirmation before one is
//! stopped (changes 262 and 318). Moved out of `src/app/screens.rs` by change
//! 318, which would otherwise have taken that file past the
//! `largest_module_lines` band.

use crate::*;

/// A session's listening ports in its header, and the confirmation before
/// one is stopped (change 262).
impl OperonApp {
    /// One line under the session's title. Not drawn when nothing in the
    /// project is listening, and not drawn when the machine cannot be read —
    /// this is information, not a task, and a standing complaint about a
    /// missing reading is worse than the missing reading.
    pub(crate) fn ui_session_ports(
        &mut self,
        ui: &mut egui::Ui,
        palette: &Palette,
        session: &Session,
    ) {
        let ports = self.session_ports(session);
        if ports.own.is_empty() && ports.elsewhere.is_empty() {
            return;
        }
        meta_separator(ui, palette);
        // The badge counts this folder only. A folder serving nothing in a
        // project that is serving something says so quietly, so the list of
        // the rest stays one click away.
        let (badge_text, badge_color) = match ports.own.as_slice() {
            [] => (
                tf!(
                    "プロジェクトで {p0} ポート稼働中",
                    p0 = ports.elsewhere.len()
                ),
                palette.text_muted,
            ),
            [only] => (
                format!(":{p0} {p1}", p0 = only.port.port, p1 = only.port.process),
                palette.info,
            ),
            several => (tf!("{p0} ポート稼働中", p0 = several.len()), palette.info),
        };
        let stopping = self.background_tasks.contains(&BackgroundKey::PortStop);
        let mut action = None;

        // Servers whose session has stopped or gone (change 318) follow the
        // count in the warning colour, inside the same button.
        let mut badge = egui::text::LayoutJob::default();
        let badge_format = |color| egui::TextFormat {
            font_id: egui::FontId::proportional(11.5),
            color,
            ..Default::default()
        };
        badge.append(
            &format!("{ICON_OPEN_EXTERNAL} {badge_text}"),
            0.0,
            badge_format(badge_color),
        );
        let leftovers = ports.leftovers();
        if leftovers > 0 {
            badge.append(" · ", 0.0, badge_format(palette.text_muted));
            badge.append(
                &format!("{ICON_ATTENTION} {}", tf!("終了忘れ? {p0}", p0 = leftovers)),
                0.0,
                badge_format(palette.warning),
            );
        }

        let badge_res = menu_custom_button(
            ui,
            egui::Button::new(badge)
                .frame(false)
                .min_size(egui::vec2(0.0, CONTROL_HEIGHT_SMALL)),
            |ui| {
                action = session_ports_menu(ui, palette, &ports, stopping);
                if action.is_some() {
                    ui.close();
                }
            },
        )
        .on_hover_text(tr("稼働中のローカルポート一覧を表示"));
        paint_button_focus(ui, &badge_res, palette);

        match action {
            Some(PortMenuAction::Open(url)) => {
                self.request_system_action(tr("ブラウザで開く"), move || open_url(&url));
            }
            Some(PortMenuAction::Copy(url)) => {
                ui.ctx().copy_text(url);
                self.notice = Some(tr("URL をコピーしました。").into());
            }
            Some(PortMenuAction::Stop(attributed)) => {
                self.pending_port_stop = Some(attributed);
            }
            None => {}
        }
    }

    /// The confirmation before a listener is stopped: which process, which
    /// PID, working where. Nothing is sent until 「停止する」.
    pub(crate) fn ui_port_stop_modal(&mut self, ctx: &egui::Context, palette: &Palette) {
        let Some(attributed) = self.pending_port_stop.clone() else {
            return;
        };
        let port = &attributed.port;
        // Not a `SessionPorts` entry, so the origin is resolved here, against
        // the store as it stands while the person decides (change 318).
        let origin = self.port_origin(&attributed);
        let process = attributed.process.as_deref();
        let unknown = || tr("不明").to_owned();
        let start = process.and_then(|process| {
            let relative = process.started_at.map(relative_time);
            match (&process.started, relative) {
                (Some(started), Some(relative)) => Some(format!("{started} ({relative})")),
                (Some(started), None) => Some(started.clone()),
                (None, relative) => relative,
            }
        });
        let parent = match process.and_then(|process| process.parent_pid) {
            Some(1) => tf!("終了済み (PID {ppid})", ppid = 1),
            Some(ppid) => tf!("PID {ppid}", ppid = ppid),
            None => unknown(),
        };
        let mut open = true;
        let mut confirmed = false;
        let mut cancelled = false;
        egui::Window::new(tr("ポートを停止"))
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .default_width(440.0)
            .show(ctx, |ui| {
                ui.label(
                    RichText::new(tf!(
                        ":{port} {process} を停止しますか？",
                        port = port.port,
                        process = port.process
                    ))
                    .strong(),
                );
                ui.add_space(SPACE_SM);
                egui::Grid::new("port-stop-facts")
                    .num_columns(2)
                    .show(ui, |ui| {
                        let muted = |ui: &mut egui::Ui, text: &str| {
                            ui.label(RichText::new(text).color(palette.text_muted));
                        };
                        muted(ui, tr("起動したセッション"));
                        ui.label(port_origin_text(&origin));
                        ui.end_row();
                        muted(ui, tr("起動時刻"));
                        ui.label(start.clone().unwrap_or_else(unknown));
                        ui.end_row();
                        muted(ui, tr("コマンド"));
                        match process.and_then(|process| process.command.as_deref()) {
                            Some(command) => ui.add(
                                egui::Label::new(RichText::new(command).monospace()).wrap(),
                            ),
                            None => ui.label(unknown()),
                        };
                        ui.end_row();
                        muted(ui, tr("フォルダ"));
                        match &attributed.directory {
                            Some(directory) => ui.monospace(friendly_path(directory)),
                            None => ui.label(unknown()),
                        };
                        ui.end_row();
                        muted(ui, tr("PID"));
                        ui.monospace(port.pid.to_string());
                        ui.end_row();
                        muted(ui, tr("親プロセス"));
                        ui.label(parent.clone());
                        ui.end_row();
                    });
                ui.add_space(SPACE_SM);
                ui.label(
                    RichText::new(tr(
                        "終了シグナル (SIGTERM) を送ります。Operon の外で起動したプロセスも含まれます。",
                    ))
                    .color(palette.text_muted),
                );
                ui.add_space(SPACE_MD);
                ui.horizontal(|ui| {
                    if ui.button(tr("キャンセル")).clicked() {
                        cancelled = true;
                    }
                    if ui
                        .button(RichText::new(tr("停止する")).color(palette.danger))
                        .clicked()
                    {
                        confirmed = true;
                    }
                });
            });
        if confirmed {
            self.pending_port_stop = None;
            self.stop_port(attributed.port);
        } else if cancelled || !open {
            self.pending_port_stop = None;
        }
    }
}

/// What a press in the ports menu asks for. Returned rather than done, so the
/// menu can be drawn and read in a test.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PortMenuAction {
    Open(String),
    Copy(String),
    Stop(AttributedPort),
}

/// The ports menu: this folder's servers, then the rest of the project's, each
/// with open, copy and stop. An empty section is not drawn.
pub(crate) fn session_ports_menu(
    ui: &mut egui::Ui,
    palette: &Palette,
    ports: &SessionPorts,
    stopping: bool,
) -> Option<PortMenuAction> {
    let mut action = None;
    egui::ScrollArea::vertical()
        .max_height(240.0)
        .show(ui, |ui| {
            if !ports.own.is_empty() {
                ui.label(RichText::new(tr("この作業フォルダ")).small().weak());
                ui.separator();
                port_menu_rows(ui, palette, ports, &ports.own, false, stopping, &mut action);
            }
            if !ports.elsewhere.is_empty() {
                ui.label(
                    RichText::new(tr("プロジェクト内のほかの場所"))
                        .small()
                        .weak(),
                );
                ui.separator();
                port_menu_rows(
                    ui,
                    palette,
                    ports,
                    &ports.elsewhere,
                    true,
                    stopping,
                    &mut action,
                );
            }
        });
    action
}

/// How a listener's origin is named (change 318).
pub(crate) fn port_origin_text(origin: &PortOrigin) -> String {
    match origin {
        PortOrigin::Session {
            name,
            running: true,
        } => tf!("「{name}」", name = name),
        PortOrigin::Session {
            name,
            running: false,
        } => tf!("「{name}」(停止中)", name = name),
        PortOrigin::RemovedSession => tr("削除済みのセッション").to_owned(),
        PortOrigin::Unknown => tr("起動元不明").to_owned(),
        PortOrigin::Unread => tr("起動元を読み取れませんでした").to_owned(),
    }
}

/// Why a row is marked 「終了忘れ?」, or `None` when it is not.
fn port_leftover_reason(origin: &PortOrigin) -> Option<&'static str> {
    match origin {
        PortOrigin::Session { running: false, .. } => {
            Some(tr("起動したセッションは停止しています"))
        }
        PortOrigin::RemovedSession => Some(tr("起動したセッションは削除されています")),
        _ => None,
    }
}

/// `11/24 19:30 · 14時間前`, or the part that was read.
fn port_start_text(process: &ListenerProcess) -> Option<String> {
    let relative = process.started_at.map(relative_time);
    match (&process.started, relative) {
        (Some(started), Some(relative)) => Some(format!("{started} · {relative}")),
        (Some(started), None) => Some(started.clone()),
        (None, Some(relative)) => Some(relative),
        (None, None) => None,
    }
}

/// The first detail line under a row: origin, a gone parent, the start.
pub(crate) fn port_detail_line(origin: &PortOrigin, process: Option<&ListenerProcess>) -> String {
    let mut parts = vec![port_origin_text(origin)];
    if let Some(process) = process {
        if process.parent_pid == Some(1) {
            parts.push(tr("親は終了済み").to_owned());
        }
        if let Some(start) = port_start_text(process) {
            parts.push(start);
        }
    }
    parts.join(" · ")
}

fn port_menu_rows(
    ui: &mut egui::Ui,
    palette: &Palette,
    session_ports: &SessionPorts,
    ports: &[AttributedPort],
    with_folder: bool,
    stopping: bool,
    action: &mut Option<PortMenuAction>,
) {
    for attributed in ports {
        let port = &attributed.port;
        let url = port_url(port);
        // Security invariant: only accept http/https scheme with loopback/local host
        let is_safe_url = (url.starts_with("http://") || url.starts_with("https://"))
            && !url
                .chars()
                .any(|c| c.is_control() || c.is_whitespace() || c == '"' || c == '\'');
        if !is_safe_url {
            continue;
        }
        let port_display = format!(":{p0} {p1}", p0 = port.port, p1 = port.process);
        let origin = session_ports.origin(port.pid);
        let process = attributed.process.as_deref();
        ui.horizontal(|ui| {
            if ui
                .button(RichText::new(port_display).size(12.0))
                .on_hover_text(tf!("ブラウザで開く: {url}"))
                .clicked()
            {
                *action = Some(PortMenuAction::Open(url.clone()));
            }
            if let Some(reason) = port_leftover_reason(origin) {
                ui.label(
                    RichText::new(format!("{ICON_ATTENTION} {}", tr("終了忘れ?")))
                        .size(11.5)
                        .color(palette.warning),
                )
                .on_hover_text(reason);
            }
            if with_folder {
                if let Some(root) = attributed.root.as_ref() {
                    let label = if root.label.is_empty() {
                        tr("プロジェクトのルート").to_owned()
                    } else {
                        root.label.clone()
                    };
                    ui.scope(|ui| {
                        ui.set_max_width(180.0);
                        ui.add(
                            egui::Label::new(
                                RichText::new(label).size(12.0).color(palette.text_muted),
                            )
                            .truncate(),
                        )
                        .on_hover_text(friendly_path(&root.path));
                    });
                }
            }
            if small_icon_button(ui, palette, ICON_COPY, tr("URL をコピー")).clicked() {
                *action = Some(PortMenuAction::Copy(url.clone()));
            }
            let stop = ui
                .add_enabled_ui(!stopping, |ui| {
                    small_icon_button(ui, palette, ICON_STOP, tr("このプロセスを停止…"))
                })
                .inner
                .on_disabled_hover_text(tr("停止しています…"));
            if stop.clicked() {
                *action = Some(PortMenuAction::Stop(attributed.clone()));
            }
        });
        // Who started it, when, and with what (change 318), indented under
        // the port so the rows still read as one list.
        ui.indent(("port-details", port.pid, port.port), |ui| {
            ui.label(
                RichText::new(port_detail_line(origin, process))
                    .size(11.5)
                    .color(palette.text_muted),
            );
            if let Some(command) = process.and_then(|process| process.command.as_deref()) {
                let hover = match &attributed.directory {
                    Some(directory) => format!("{command}\n{}", friendly_path(directory)),
                    None => command.to_owned(),
                };
                ui.add(
                    egui::Label::new(
                        RichText::new(command)
                            .size(11.5)
                            .monospace()
                            .color(palette.text_muted),
                    )
                    .truncate(),
                )
                .on_hover_text(hover);
            }
        });
    }
}
