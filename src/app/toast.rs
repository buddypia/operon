//! The corner overlay: brief toasts on every page, and on the Sessions page
//! the persistent notice and the durability warning too, so neither takes a
//! row above the terminal workspace and moves it (change 289).

use crate::*;

impl OperonApp {
    /// Corner overlay for transient toasts everywhere, and for persistent
    /// notices and durability warnings on `Page::Sessions` so they never shift
    /// the terminal workspace vertically.
    pub(crate) fn ui_brief_notice_toast(&mut self, ctx: &egui::Context, palette: &Palette) {
        let show_warning = self.page == Page::Sessions && self.durability_warning.is_some();
        let brief = notice_is_brief(&self.notice, &self.brief_notice);
        let show_notice = self.notice.is_some() && (brief || self.page == Page::Sessions);
        if !show_warning && !show_notice {
            return;
        }
        let card_max_w = (ctx.content_rect().width() - SPACE_LG * 2.0).clamp(200.0, 420.0);
        let warn_text_w = (card_max_w - 52.0).max(80.0);
        let notice_text_w = (card_max_w - 52.0 - CONTROL_HEIGHT_SMALL - SPACE_SM * 2.0).max(80.0);
        let shadow = ctx.global_style().visuals.popup_shadow;
        let mut dismissed = false;
        egui::Area::new(egui::Id::unique("brief-notice-toast"))
            .order(egui::Order::Foreground)
            .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-SPACE_LG, -SPACE_LG))
            .interactable(true)
            .show(ctx, |ui| {
                if show_warning {
                    if let Some(warning) = self.durability_warning.as_deref() {
                        egui::Frame::new()
                            .fill(palette.raised)
                            .stroke(egui::Stroke::new(1.0, palette.warning))
                            .corner_radius(egui::CornerRadius::same(RADIUS_CARD))
                            .inner_margin(egui::Margin::symmetric(12, 8))
                            .shadow(shadow)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = SPACE_SM;
                                    ui.label(
                                        RichText::new(ICON_ATTENTION)
                                            .size(14.0)
                                            .color(palette.warning),
                                    );
                                    ui.scope(|ui| {
                                        ui.set_max_width(warn_text_w);
                                        ui.add(
                                            egui::Label::new(
                                                RichText::new(warning)
                                                    .size(13.5)
                                                    .color(palette.text),
                                            )
                                            .wrap(),
                                        );
                                    });
                                });
                            });
                        if show_notice {
                            ui.add_space(SPACE_SM);
                        }
                    }
                }
                if show_notice {
                    if let Some(text) = self.notice.as_deref() {
                        let (icon, icon_color, border) = if brief {
                            (ICON_STATUS_DONE, palette.success, palette.border_subtle)
                        } else {
                            (ICON_NOTICE, palette.info, palette.info)
                        };
                        egui::Frame::new()
                            .fill(palette.raised)
                            .stroke(egui::Stroke::new(1.0, border))
                            .corner_radius(egui::CornerRadius::same(RADIUS_CARD))
                            .inner_margin(egui::Margin::symmetric(12, 8))
                            .shadow(shadow)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = SPACE_SM;
                                    ui.label(RichText::new(icon).size(14.0).color(icon_color));
                                    ui.scope(|ui| {
                                        ui.set_max_width(notice_text_w);
                                        ui.add(
                                            egui::Label::new(
                                                RichText::new(text).size(13.5).color(palette.text),
                                            )
                                            .wrap(),
                                        );
                                    });
                                    if small_icon_button(
                                        ui,
                                        palette,
                                        ICON_CLOSE,
                                        tr("この通知を閉じる"),
                                    )
                                    .clicked()
                                    {
                                        dismissed = true;
                                    }
                                });
                            });
                    }
                }
            });
        if dismissed {
            self.notice = None;
            self.brief_notice = None;
        }
    }
}
