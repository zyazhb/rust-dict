use eframe::egui::{self, vec2, Color32, Frame, Id, RichText, Sense, Stroke, ViewportCommand, WindowLevel};

use dict_db::SearchMode;

use crate::app::DictApp;

const ICON_SIZE: f32 = 52.0;
const EXPANDED_SIZE: egui::Vec2 = egui::vec2(320.0, 400.0);
const FULL_SIZE: egui::Vec2 = egui::vec2(960.0, 640.0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatState {
    Collapsed,
    Expanded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiMode {
    Float(FloatState),
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ViewportLayout {
    FloatCollapsed,
    FloatExpanded,
    Full,
}

impl DictApp {
    pub fn update_float(&mut self, ctx: &egui::Context) {
        self.sync_viewport(ctx);
        match self.ui_mode {
            UiMode::Float(FloatState::Collapsed) => self.ui_float_collapsed(ctx),
            UiMode::Float(FloatState::Expanded) => self.ui_float_expanded(ctx),
            UiMode::Full => {}
        }
    }

    pub fn expand_float(&mut self, ctx: &egui::Context) {
        self.ui_mode = UiMode::Float(FloatState::Expanded);
        self.sync_viewport(ctx);
        ctx.send_viewport_cmd(ViewportCommand::Focus);
        ctx.request_repaint();
    }

    pub fn collapse_float(&mut self, ctx: &egui::Context) {
        self.ui_mode = UiMode::Float(FloatState::Collapsed);
        self.sync_viewport(ctx);
        ctx.request_repaint();
    }

    pub fn enter_full_mode(&mut self, ctx: &egui::Context) {
        self.ui_mode = UiMode::Full;
        self.sync_viewport(ctx);
        ctx.send_viewport_cmd(ViewportCommand::Title(self.window_title()));
        ctx.request_repaint();
    }

    fn sync_viewport(&mut self, ctx: &egui::Context) {
        let layout = match self.ui_mode {
            UiMode::Float(FloatState::Collapsed) => ViewportLayout::FloatCollapsed,
            UiMode::Float(FloatState::Expanded) => ViewportLayout::FloatExpanded,
            UiMode::Full => ViewportLayout::Full,
        };
        if self.last_viewport == Some(layout) {
            return;
        }
        self.last_viewport = Some(layout);

        match layout {
            ViewportLayout::FloatCollapsed => {
                ctx.send_viewport_cmd(ViewportCommand::WindowLevel(WindowLevel::AlwaysOnTop));
                ctx.send_viewport_cmd(ViewportCommand::Decorations(false));
                ctx.send_viewport_cmd(ViewportCommand::Resizable(false));
                ctx.send_viewport_cmd(ViewportCommand::InnerSize(vec2(ICON_SIZE, ICON_SIZE)));
                ctx.send_viewport_cmd(ViewportCommand::MinInnerSize(vec2(ICON_SIZE, ICON_SIZE)));
                ctx.send_viewport_cmd(ViewportCommand::MaxInnerSize(vec2(ICON_SIZE, ICON_SIZE)));
            }
            ViewportLayout::FloatExpanded => {
                ctx.send_viewport_cmd(ViewportCommand::WindowLevel(WindowLevel::AlwaysOnTop));
                ctx.send_viewport_cmd(ViewportCommand::Decorations(false));
                ctx.send_viewport_cmd(ViewportCommand::Resizable(true));
                ctx.send_viewport_cmd(ViewportCommand::InnerSize(EXPANDED_SIZE));
                ctx.send_viewport_cmd(ViewportCommand::MinInnerSize(vec2(280.0, 200.0)));
                ctx.send_viewport_cmd(ViewportCommand::MaxInnerSize(vec2(480.0, 720.0)));
            }
            ViewportLayout::Full => {
                ctx.send_viewport_cmd(ViewportCommand::WindowLevel(WindowLevel::Normal));
                ctx.send_viewport_cmd(ViewportCommand::Decorations(true));
                ctx.send_viewport_cmd(ViewportCommand::Resizable(true));
                ctx.send_viewport_cmd(ViewportCommand::InnerSize(FULL_SIZE));
                ctx.send_viewport_cmd(ViewportCommand::MinInnerSize(vec2(640.0, 480.0)));
                ctx.send_viewport_cmd(ViewportCommand::MaxInnerSize(vec2(1920.0, 1080.0)));
            }
        }
    }

    fn ui_float_collapsed(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(Frame::NONE.fill(Color32::from_rgb(45, 125, 210)))
            .show(ctx, |ui| {
                ui.centered_and_justified(|ui| {
                    let size = vec2(ICON_SIZE - 4.0, ICON_SIZE - 4.0);
                    let (rect, response) = ui.allocate_exact_size(size, Sense::click_and_drag());
                    let painter = ui.painter();
                    painter.circle_filled(rect.center(), 22.0, Color32::from_rgb(30, 100, 180));
                    painter.circle_stroke(rect.center(), 22.0, Stroke::new(1.5, Color32::WHITE));
                    painter.text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "查",
                        egui::FontId::proportional(22.0),
                        Color32::WHITE,
                    );

                    if response.drag_started() {
                        ctx.send_viewport_cmd(ViewportCommand::StartDrag);
                    }
                    if response.clicked() {
                        self.expand_float(ctx);
                    }
                });
            });
    }

    fn ui_float_expanded(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                let header = egui::Area::new(Id::new("float_drag"))
                    .show(ui.ctx(), |ui| {
                        ui.horizontal(|ui| {
                            let (r, resp) =
                                ui.allocate_exact_size(vec2(24.0, 20.0), Sense::click_and_drag());
                            ui.painter().text(
                                r.center(),
                                egui::Align2::CENTER_CENTER,
                                "⠿",
                                egui::FontId::proportional(14.0),
                                ui.visuals().weak_text_color(),
                            );
                            if resp.drag_started() {
                                ctx.send_viewport_cmd(ViewportCommand::StartDrag);
                            }
                            ui.label(RichText::new(self.i18n.t("app_title")).strong());
                        });
                    })
                    .response;
                let _ = header;
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button(self.i18n.t("float_full")).clicked() {
                        self.enter_full_mode(ctx);
                    }
                    if ui.small_button(self.i18n.t("float_collapse")).clicked() {
                        self.collapse_float(ctx);
                    }
                });
            });
            ui.separator();

            ui.horizontal(|ui| {
                ui.selectable_value(
                    &mut self.search_mode,
                    SearchMode::ZhToEn,
                    self.i18n.t("mode_zh_en"),
                );
                ui.selectable_value(
                    &mut self.search_mode,
                    SearchMode::EnToEn,
                    self.i18n.t("mode_en_en"),
                );
                if self.search_mode == SearchMode::ZhToEn {
                    ui.checkbox(&mut self.pinyin_mode, self.i18n.t("pinyin"));
                }
            });

            let changed = ui
                .add(
                    egui::TextEdit::singleline(&mut self.query)
                        .hint_text(self.i18n.t("query_hint"))
                        .desired_width(f32::INFINITY),
                )
                .changed();
            if changed {
                self.schedule_debounced_search();
            }

            ui.label(egui::RichText::new(&self.status).small().weak());

            let mut save_at: Option<usize> = None;
            let n = self.results.len();
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for i in 0..n {
                        let c = &self.results[i];
                        let chinese = if self.settings.show_traditional {
                            c.entry.trad.as_str()
                        } else {
                            c.entry.simp.as_str()
                        };
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.label(RichText::new(&c.english).strong());
                                    ui.label(format!("{chinese}  {}", c.entry.pinyin));
                                });
                                if ui.small_button(self.i18n.t("save")).clicked() {
                                    save_at = Some(i);
                                }
                            });
                        });
                    }
                });

            if let Some(i) = save_at {
                let c = &self.results[i];
                let chinese = if self.settings.show_traditional {
                    c.entry.trad.as_str()
                } else {
                    c.entry.simp.as_str()
                };
                let _ = self.user.save_word(
                    &c.english,
                    chinese,
                    &c.entry.pinyin,
                    &c.sense,
                    "",
                );
                self.status = self.i18n.status_saved();
            }
        });
    }
}
