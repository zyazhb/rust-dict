use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use dict_core::{
    dto_to_ranked, HttpOnlineProvider, MockOnlineProvider, OnlineProvider, QueryRouter,
    RankBadge, RankedCandidate,
};
use dict_db::{AppSettings, CedictDb, HistoryRecord, SavedWord, SearchMode, UserDb};
use eframe::egui;

use crate::float::{FloatState, UiMode};
use crate::i18n::{I18n, Locale};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AppTab {
    Search,
    History,
    Saved,
    Settings,
}

pub struct DictApp {
    pub(crate) router: QueryRouter,
    pub(crate) cedict: Option<CedictDb>,
    pub(crate) user: UserDb,
    pub(crate) settings: AppSettings,
    pub(crate) tab: AppTab,
    pub(crate) query: String,
    pub(crate) pinyin_mode: bool,
    pub(crate) search_mode: SearchMode,
    pub(crate) results: Vec<RankedCandidate>,
    pub(crate) status: String,
    pub(crate) last_search_at: Option<Instant>,
    pub(crate) pending_search: bool,
    pub(crate) history: Vec<HistoryRecord>,
    pub(crate) saved: Vec<SavedWord>,
    pub(crate) saved_filter: String,
    pub(crate) note_edit_id: Option<i64>,
    pub(crate) note_edit_text: String,
    online_rx: Option<mpsc::Receiver<OnlineSearchResult>>,
    pub(crate) force_online_next: bool,
    pub(crate) cedict_path_edit: String,
    pub(crate) i18n: I18n,
    pub(crate) locale_edit: Locale,
    pub(crate) window_title: String,
    pub(crate) ui_mode: UiMode,
    pub(crate) last_viewport: Option<crate::float::ViewportLayout>,
}

struct OnlineSearchResult {
    query: String,
    results: Vec<RankedCandidate>,
    error: Option<String>,
}

impl DictApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("eng-dict");
        std::fs::create_dir_all(&data_dir).ok();
        let user_path = data_dir.join("user.db");
        let user = UserDb::open(user_path.to_str().unwrap()).expect("user db");
        let mut settings = user.get_settings().unwrap_or_default();
        if settings.cedict_path.is_empty() {
            settings.cedict_path = default_cedict_path();
        }
        let cedict_path_edit = settings.cedict_path.clone();
        let cedict = CedictDb::open_readonly(&settings.cedict_path).ok();
        let locale = if settings.locale.is_empty() {
            Locale::Zh
        } else {
            Locale::parse(&settings.locale)
        };
        if settings.locale.is_empty() {
            settings.locale = locale.as_str().to_string();
        }
        let i18n = I18n::new(locale);
        let window_title = i18n.t("app_title").to_string();
        let locale_edit = locale;
        let status = if cedict.is_some() {
            i18n.status_ready()
        } else {
            i18n.status_dict_not_found(&settings.cedict_path)
        };
        Self {
            router: QueryRouter::default(),
            cedict,
            user,
            settings,
            tab: AppTab::Search,
            query: String::new(),
            pinyin_mode: false,
            search_mode: SearchMode::ZhToEn, // default 中文→英文
            results: vec![],
            status,
            last_search_at: None,
            pending_search: false,
            history: vec![],
            saved: vec![],
            saved_filter: String::new(),
            note_edit_id: None,
            note_edit_text: String::new(),
            online_rx: None,
            force_online_next: false,
            cedict_path_edit,
            i18n,
            locale_edit,
            window_title,
            ui_mode: UiMode::Float(FloatState::Collapsed),
            last_viewport: None,
        }
    }

    pub fn window_title(&self) -> String {
        self.window_title.clone()
    }

    fn apply_locale(&mut self) {
        self.settings.locale = self.locale_edit.as_str().to_string();
        self.i18n = I18n::new(self.locale_edit);
        self.window_title = self.i18n.t("app_title").to_string();
    }

    /// Schedule repaints only when the UI is actively changing.
    fn schedule_repaint_if_needed(&self, ctx: &egui::Context) {
        const DEBOUNCE: Duration = Duration::from_millis(300);
        let mut wait = None;

        if self.pending_search {
            if let Some(started) = self.last_search_at {
                wait = Some(DEBOUNCE.saturating_sub(started.elapsed()));
            }
        }
        if self.online_rx.is_some() {
            wait = Some(match wait {
                Some(d) => d.min(Duration::from_millis(250)),
                None => Duration::from_millis(250),
            });
        }

        if let Some(delay) = wait {
            ctx.request_repaint_after(delay.max(Duration::from_millis(32)));
        }
    }

    fn reload_cedict(&mut self) {
        self.settings.cedict_path = self.cedict_path_edit.clone();
        let _ = self.user.save_settings(&self.settings);
        self.cedict = CedictDb::open_readonly(&self.settings.cedict_path).ok();
        self.status = if self.cedict.is_some() {
            self.i18n.status_dict_loaded()
        } else {
            self.i18n.status_cannot_open(&self.settings.cedict_path)
        };
    }

    fn refresh_history(&mut self) {
        if let Ok(h) = self.user.list_history(100) {
            self.history = h;
        }
    }

    fn refresh_saved(&mut self) {
        if let Ok(s) = self.user.list_saved() {
            self.saved = s;
        }
    }

    fn run_search(&mut self, force_online: bool) {
        let Some(cedict) = self.cedict.as_ref() else {
            self.status = self.i18n.status_load_dict_first();
            return;
        };
        let q = self.query.trim().to_string();
        if q.is_empty() {
            self.results.clear();
            return;
        }

        let online_enabled = self.settings.online_enabled;
        let threshold = self.settings.online_score_threshold;
        let use_trad = self.settings.show_traditional;

        if online_enabled && (force_online || self.settings.online_api_url.is_empty()) {
            let local = self
                .router
                .search_local(
                    cedict,
                    &self.user,
                    &q,
                    self.search_mode,
                    self.pinyin_mode,
                    use_trad,
                )
                .unwrap_or_default();

            if force_online
                || self
                    .router
                    .needs_online_fallback(&local, threshold, force_online)
            {
                self.start_online_search(q.clone(), local);
                return;
            }
        }

        match self.router.search_with_online(
            cedict,
            &self.user,
            &q,
            self.search_mode,
            self.pinyin_mode,
            use_trad,
            self.online_provider().as_ref().map(|p| p.as_ref()),
            force_online,
            threshold,
        ) {
            Ok(results) => {
                self.results = results;
                let _ = self
                    .user
                    .add_history(&q, self.search_mode, self.results.first().map(|r| r.entry.id));
                self.status = self.i18n.status_results(self.results.len());
                self.refresh_history();
            }
            Err(e) => self.status = e.to_string(),
        }
    }

    fn online_provider(&self) -> Option<Box<dyn OnlineProvider>> {
        if !self.settings.online_enabled {
            return None;
        }
        if self.settings.online_api_url.is_empty() {
            Some(Box::new(MockOnlineProvider))
        } else {
            Some(Box::new(HttpOnlineProvider {
                base_url: self.settings.online_api_url.clone(),
                api_key: self.settings.online_api_key.clone(),
            }))
        }
    }

    fn start_online_search(&mut self, query: String, mut local: Vec<RankedCandidate>) {
        let provider = self.online_provider();
        let user_path = self.user_db_path();
        let (tx, rx) = mpsc::channel();
        self.online_rx = Some(rx);
        self.status = self.i18n.status_searching_online();

        thread::spawn(move || {
            let result = {
                let user = UserDb::open(&user_path).ok();
                let provider = provider.unwrap_or_else(|| Box::new(MockOnlineProvider));
                match provider.search_zh_to_en(&query) {
                    Ok(dtos) => {
                        if let Some(ref u) = user {
                            if let Ok(json) = serde_json::to_string(&dtos) {
                                let hash = format!("{:x}", simple_hash(&query));
                                let _ = u.set_online_cache(&hash, &json);
                            }
                        }
                        let online = dto_to_ranked(dtos, &query);
                        local.extend(online);
                        local.sort_by(|a, b| {
                            b.score
                                .partial_cmp(&a.score)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        });
                        local.truncate(50);
                        Ok(local)
                    }
                    Err(e) => Err(e.to_string()),
                }
            };
            let msg = match result {
                Ok(results) => OnlineSearchResult {
                    query,
                    results,
                    error: None,
                },
                Err(e) => OnlineSearchResult {
                    query,
                    results: vec![],
                    error: Some(e),
                },
            };
            let _ = tx.send(msg);
        });
    }

    fn user_db_path(&self) -> String {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("eng-dict")
            .join("user.db")
            .to_string_lossy()
            .into_owned()
    }

    fn poll_online(&mut self) {
        let Some(rx) = &self.online_rx else {
            return;
        };
        if let Ok(msg) = rx.try_recv() {
            self.online_rx = None;
            if let Some(err) = msg.error {
                self.status = err;
            } else {
                self.results = msg.results;
                let _ = self.user.add_history(
                    &msg.query,
                    self.search_mode,
                    self.results.first().map(|r| r.entry.id),
                );
                self.status = self.i18n.status_results_online(self.results.len());
                self.refresh_history();
            }
        }
    }

    pub(crate) fn schedule_debounced_search(&mut self) {
        self.pending_search = true;
        self.last_search_at = Some(Instant::now());
    }
}

impl eframe::App for DictApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_online();

        if self.pending_search {
            if let Some(t) = self.last_search_at {
                if t.elapsed() >= Duration::from_millis(300) {
                    let force = self.force_online_next;
                    self.force_online_next = false;
                    self.run_search(force);
                    self.pending_search = false;
                }
            }
        }

        if matches!(self.ui_mode, UiMode::Float(_)) {
            self.update_float(ctx);
            self.schedule_repaint_if_needed(ctx);
            return;
        }

        let side_w = if self.i18n.locale == Locale::Zh { 130.0 } else { 120.0 };
        egui::SidePanel::left("nav")
            .resizable(false)
            .default_width(side_w)
            .show(ctx, |ui| {
                ui.heading(self.i18n.t("app_title"));
                ui.separator();
                if ui
                    .selectable_label(self.tab == AppTab::Search, self.i18n.t("tab_search"))
                    .clicked()
                {
                    self.tab = AppTab::Search;
                }
                if ui
                    .selectable_label(self.tab == AppTab::History, self.i18n.t("tab_history"))
                    .clicked()
                {
                    self.tab = AppTab::History;
                    self.refresh_history();
                }
                if ui
                    .selectable_label(self.tab == AppTab::Saved, self.i18n.t("tab_saved"))
                    .clicked()
                {
                    self.tab = AppTab::Saved;
                    self.refresh_saved();
                }
                if ui
                    .selectable_label(self.tab == AppTab::Settings, self.i18n.t("tab_settings"))
                    .clicked()
                {
                    self.tab = AppTab::Settings;
                }
                ui.separator();
                if ui.button(self.i18n.t("float_open")).clicked() {
                    self.ui_mode = UiMode::Float(FloatState::Collapsed);
                    self.last_viewport = None;
                    self.collapse_float(ctx);
                }
                ui.label(egui::RichText::new(&self.status).small().weak());
            });

        egui::CentralPanel::default().show(ctx, |ui| match self.tab {
            AppTab::Search => self.ui_search(ui),
            AppTab::History => self.ui_history(ui),
            AppTab::Saved => self.ui_saved(ui),
            AppTab::Settings => self.ui_settings(ui),
        });

        self.schedule_repaint_if_needed(ctx);
    }
}

impl DictApp {
    fn ui_search(&mut self, ui: &mut egui::Ui) {
        ui.heading(self.i18n.t("search_heading"));
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
                    .hint_text(self.i18n.t("query_hint")),
            )
            .changed();
        ui.horizontal(|ui| {
            if ui.button(self.i18n.t("search_now")).clicked() {
                self.run_search(false);
            }
            if ui.button(self.i18n.t("search_online")).clicked() {
                self.force_online_next = true;
                self.run_search(true);
            }
        });
        if changed {
            self.schedule_debounced_search();
        }

        let mut save_at: Option<usize> = None;
        let n = self.results.len();
        egui::ScrollArea::vertical().show(ui, |ui| {
            for i in 0..n {
                let c = &self.results[i];
                let chinese = if self.settings.show_traditional {
                    c.entry.trad.as_str()
                } else {
                    c.entry.simp.as_str()
                };
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.heading(&c.english);
                        ui.label(format!("{chinese} [{}]", c.entry.pinyin));
                        if ui.button(self.i18n.t("save")).clicked() {
                            save_at = Some(i);
                        }
                    });
                    ui.label(&c.sense);
                    let badges: Vec<_> = c
                        .badges
                        .iter()
                        .map(|b| match b {
                            RankBadge::ExactMatch => self.i18n.t("badge_exact"),
                            RankBadge::CommonWord => self.i18n.t("badge_common"),
                            RankBadge::SavedBefore => self.i18n.t("badge_saved"),
                            RankBadge::Online => self.i18n.t("badge_online"),
                        })
                        .collect();
                    if !badges.is_empty() {
                        ui.label(format!(
                            "[{}] {} {:.2}",
                            badges.join(", "),
                            self.i18n.t("score"),
                            c.score
                        ));
                    }
                });
                if i + 1 < n {
                    ui.separator();
                }
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
            self.refresh_saved();
        }
    }

    fn ui_history(&mut self, ui: &mut egui::Ui) {
        ui.heading(self.i18n.t("history_heading"));
        let mut replay: Option<(String, SearchMode)> = None;
        egui::ScrollArea::vertical().show(ui, |ui| {
            for h in &self.history {
                let mode_label = if h.mode == SearchMode::ZhToEn {
                    self.i18n.t("hist_zh_en")
                } else {
                    self.i18n.t("hist_en_cn")
                };
                let label = format!("{} — {}", h.query, mode_label);
                if ui.button(label).clicked() {
                    replay = Some((h.query.clone(), h.mode));
                }
            }
        });
        if let Some((query, mode)) = replay {
            self.query = query;
            self.search_mode = mode;
            self.tab = AppTab::Search;
            self.run_search(false);
        }
    }

    fn ui_saved(&mut self, ui: &mut egui::Ui) {
        ui.heading(self.i18n.t("saved_heading"));
        ui.add(
            egui::TextEdit::singleline(&mut self.saved_filter)
                .hint_text(self.i18n.t("filter_hint")),
        );
        let filter = self.saved_filter.to_lowercase();
        let mut save_note_id: Option<i64> = None;
        let mut edit_note: Option<(i64, String)> = None;
        let mut delete_id: Option<i64> = None;
        egui::ScrollArea::vertical().show(ui, |ui| {
            for w in &self.saved {
                if !filter.is_empty()
                    && !w.english.to_lowercase().contains(&filter)
                    && !w.chinese.contains(&filter)
                {
                    continue;
                }
                ui.group(|ui| {
                    ui.heading(format!("{} — {}", w.english, w.chinese));
                    if !w.pinyin.is_empty() {
                        ui.label(&w.pinyin);
                    }
                    ui.label(&w.definition);
                    if self.note_edit_id == Some(w.id) {
                        ui.text_edit_multiline(&mut self.note_edit_text);
                        if ui.button(self.i18n.t("save_note")).clicked() {
                            save_note_id = Some(w.id);
                        }
                    } else if ui.button(self.i18n.t("edit_note")).clicked() {
                        edit_note = Some((w.id, w.note.clone()));
                    }
                    if ui.button(self.i18n.t("delete")).clicked() {
                        delete_id = Some(w.id);
                    }
                });
            }
        });
        if let Some(id) = save_note_id {
            let _ = self.user.update_note(id, &self.note_edit_text);
            self.note_edit_id = None;
            self.refresh_saved();
        }
        if let Some((id, note)) = edit_note {
            self.note_edit_id = Some(id);
            self.note_edit_text = note;
        }
        if let Some(id) = delete_id {
            let _ = self.user.delete_saved(id);
            self.refresh_saved();
        }
    }

    fn ui_settings(&mut self, ui: &mut egui::Ui) {
        ui.heading(self.i18n.t("settings_heading"));
        ui.label(self.i18n.t("language"));
        let prev_locale = self.locale_edit;
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.locale_edit, Locale::En, "English");
            ui.selectable_value(&mut self.locale_edit, Locale::Zh, "中文");
        });
        if self.locale_edit != prev_locale {
            self.apply_locale();
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::Title(self.window_title.clone()));
        }
        ui.separator();
        ui.label(self.i18n.t("cedict_path"));
        ui.text_edit_singleline(&mut self.cedict_path_edit);
        if ui.button(self.i18n.t("reload_dict")).clicked() {
            self.reload_cedict();
        }
        ui.checkbox(
            &mut self.settings.show_traditional,
            self.i18n.t("show_trad"),
        );
        ui.separator();
        ui.checkbox(
            &mut self.settings.online_enabled,
            self.i18n.t("online_enable"),
        );
        ui.label(self.i18n.t("api_url"));
        ui.text_edit_singleline(&mut self.settings.online_api_url);
        ui.label(self.i18n.t("api_key"));
        ui.add(egui::TextEdit::singleline(&mut self.settings.online_api_key).password(true));
        ui.add(
            egui::Slider::new(&mut self.settings.online_score_threshold, 0.0..=1.0)
                .text(self.i18n.t("online_threshold")),
        );
        if ui.button(self.i18n.t("save_settings")).clicked() {
            self.apply_locale();
            self.settings.cedict_path = self.cedict_path_edit.clone();
            let _ = self.user.save_settings(&self.settings);
            self.status = self.i18n.status_settings_saved();
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Title(self.window_title.clone()));
        }
        ui.separator();
        ui.label(self.i18n.t("license"));
    }
}

fn default_cedict_path() -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../data/cedict.db")
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from("data/cedict.db"))
        .to_string_lossy()
        .into_owned()
}

fn simple_hash(s: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}
