mod drag;

use eframe::egui::{
    self, vec2, Color32, Frame, Id, Pos2, Rect, RichText, Sense, Stroke, StrokeKind,
    ViewportCommand, WindowLevel,
};

use dict_db::SearchMode;

use crate::app::DictApp;

pub use drag::{handle_native_window_drag, is_plain_click};

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

    /// Compact float icon: drag anywhere on the circle, click without moving to expand.
    fn ui_float_collapsed(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(Frame::NONE.fill(Color32::from_rgb(45, 125, 210)))
            .show(ctx, |ui| {
                let rect = ui.max_rect();
                let response =
                    ui.interact(rect, Id::new("float_icon"), Sense::click_and_drag());

                let center = rect.center();
                let painter = ui.painter();
                painter.circle_filled(center, 22.0, Color32::from_rgb(30, 100, 180));
                painter.circle_stroke(center, 22.0, Stroke::new(1.5, Color32::WHITE));
                paint_dict_search_icon(painter, center, Color32::WHITE);

                handle_native_window_drag(ctx, &response);
                if is_plain_click(&response) {
                    self.expand_float(ctx);
                }
            });
    }

    /// Expanded panel: dedicated drag bar; buttons stay clickable below it.
    fn ui_float_expanded(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui_float_expanded_header(ctx, ui, self);
            ui.separator();

            ui.horizontal(|ui| {
                ui.selectable_value(
                    &mut self.search_mode,
                    SearchMode::ZhToEn,
                    self.i18n.t("mode_zh_en"),
                );
                ui.selectable_value(
                    &mut self.search_mode,
                    SearchMode::EnToCn,
                    self.i18n.t("mode_en_cn"),
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

/// Title row: left = drag handle + title, right = action buttons (no overlap).
fn ui_float_expanded_header(ctx: &egui::Context, ui: &mut egui::Ui, app: &mut DictApp) {
    ui.horizontal(|ui| {
        let full_w = ui.available_width();
        let button_w = 76.0;
        let drag_w = (full_w - button_w).max(80.0);

        let (drag_rect, drag_resp) =
            ui.allocate_exact_size(vec2(drag_w, 28.0), Sense::drag());
        handle_native_window_drag(ctx, &drag_resp);

        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(drag_rect), |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("⠿").weak());
                ui.label(RichText::new(app.i18n.t("app_title")).strong());
            });
        });

        ui.allocate_ui_with_layout(
            vec2(button_w, 28.0),
            egui::Layout::right_to_left(egui::Align::Center),
            |ui| {
                if ui.small_button(app.i18n.t("float_collapse")).clicked() {
                    app.collapse_float(ctx);
                }
                if ui.small_button(app.i18n.t("float_full")).clicked() {
                    app.enter_full_mode(ctx);
                }
            },
        );
    });
}

fn paint_dict_search_icon(painter: &egui::Painter, center: Pos2, color: Color32) {
    let stroke = Stroke::new(2.0, color);
    let book = Rect::from_center_size(center + vec2(-5.0, 3.0), vec2(13.0, 15.0));
    painter.rect_stroke(book, 1.5, stroke, StrokeKind::Middle);
    painter.line_segment(
        [
            book.left_top() + vec2(4.0, 2.0),
            book.left_bottom() - vec2(-4.0, 2.0),
        ],
        stroke,
    );

    let lens = center + vec2(7.0, -5.0);
    painter.circle_stroke(lens, 6.5, stroke);
    let handle = lens + vec2(4.5, 4.5);
    painter.line_segment([handle, handle + vec2(7.0, 7.0)], stroke);
}
