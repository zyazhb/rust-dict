use dict_core::RankBadge;
use dict_db::SearchMode;
use eframe::egui::{self, Id, RichText};

use crate::app::DictApp;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ResultsStyle {
    Full,
    Compact,
}

pub struct QueryFieldOpts {
    pub id: Option<Id>,
    pub request_focus: bool,
    pub full_width: bool,
}

impl QueryFieldOpts {
    pub fn full_mode() -> Self {
        Self {
            id: None,
            request_focus: false,
            full_width: false,
        }
    }

    pub fn float_mode(request_focus: bool) -> Self {
        Self {
            id: Some(Id::new("float_query")),
            request_focus,
            full_width: true,
        }
    }
}

pub fn ui_search_mode_controls(app: &mut DictApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.selectable_value(
            &mut app.search_mode,
            SearchMode::ZhToEn,
            app.i18n.t("mode_zh_en"),
        );
        ui.selectable_value(
            &mut app.search_mode,
            SearchMode::EnToCn,
            app.i18n.t("mode_en_cn"),
        );
        if app.search_mode == SearchMode::ZhToEn {
            ui.checkbox(&mut app.pinyin_mode, app.i18n.t("pinyin"));
        }
    });
}

pub fn ui_query_field(app: &mut DictApp, ui: &mut egui::Ui, opts: QueryFieldOpts) -> bool {
    let mut edit = egui::TextEdit::singleline(&mut app.query).hint_text(app.i18n.t("query_hint"));
    if let Some(id) = opts.id {
        edit = edit.id(id);
    }
    if opts.full_width {
        edit = edit.desired_width(f32::INFINITY);
    }
    let resp = ui.add(edit);
    if opts.request_focus {
        resp.request_focus();
        if resp.has_focus() {
            app.float_focus_search = false;
        }
    }
    resp.changed()
}

pub fn ui_search_actions(app: &mut DictApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        if ui.button(app.i18n.t("search_now")).clicked() {
            app.run_search(false);
        }
        if ui.button(app.i18n.t("search_online")).clicked() {
            app.force_online_next = true;
            app.run_search(true);
        }
    });
}

pub fn ui_search_results(
    app: &mut DictApp,
    ui: &mut egui::Ui,
    style: ResultsStyle,
) -> Option<usize> {
    let mut save_at = None;
    let n = app.results.len();
    let scroll = egui::ScrollArea::vertical();
    let scroll = if style == ResultsStyle::Compact {
        scroll.auto_shrink([false, false])
    } else {
        scroll
    };
    scroll.show(ui, |ui| {
        for i in 0..n {
            let c = &app.results[i];
            let chinese = if app.settings.show_traditional {
                c.entry.trad.as_str()
            } else {
                c.entry.simp.as_str()
            };
            ui.group(|ui| match style {
                ResultsStyle::Full => {
                    ui.horizontal(|ui| {
                        ui.heading(&c.english);
                        ui.label(format!("{chinese} [{}]", c.entry.pinyin));
                        if ui.button(app.i18n.t("save")).clicked() {
                            save_at = Some(i);
                        }
                    });
                    ui.label(&c.sense);
                    let badges: Vec<_> = c
                        .badges
                        .iter()
                        .map(|b| match b {
                            RankBadge::ExactMatch => app.i18n.t("badge_exact"),
                            RankBadge::CommonWord => app.i18n.t("badge_common"),
                            RankBadge::SavedBefore => app.i18n.t("badge_saved"),
                            RankBadge::Online => app.i18n.t("badge_online"),
                        })
                        .collect();
                    if !badges.is_empty() {
                        ui.label(format!(
                            "[{}] {} {:.2}",
                            badges.join(", "),
                            app.i18n.t("score"),
                            c.score
                        ));
                    }
                }
                ResultsStyle::Compact => {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(RichText::new(&c.english).strong());
                            ui.label(format!("{chinese}  {}", c.entry.pinyin));
                        });
                        if ui.small_button(app.i18n.t("save")).clicked() {
                            save_at = Some(i);
                        }
                    });
                }
            });
            if style == ResultsStyle::Full && i + 1 < n {
                ui.separator();
            }
        }
    });
    save_at
}

pub fn save_result_at(app: &mut DictApp, index: usize) {
    let c = &app.results[index];
    let chinese = if app.settings.show_traditional {
        c.entry.trad.as_str()
    } else {
        c.entry.simp.as_str()
    };
    let _ = app.user.save_word(
        &c.english,
        chinese,
        &c.entry.pinyin,
        &c.sense,
        "",
    );
    app.status = app.i18n.status_saved();
    app.refresh_saved();
}
