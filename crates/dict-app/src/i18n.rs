#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Locale {
    #[default]
    En,
    Zh,
}

impl Locale {
    pub fn parse(s: &str) -> Self {
        match s {
            "zh" | "zh-CN" | "zh-TW" => Locale::Zh,
            _ => Locale::En,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Locale::En => "en",
            Locale::Zh => "zh",
        }
    }
}

pub struct I18n {
    pub locale: Locale,
}

impl I18n {
    pub fn new(locale: Locale) -> Self {
        Self { locale }
    }

    pub fn t(&self, key: &'static str) -> &'static str {
        match (self.locale, key) {
            // App
            (Locale::En, "app_title") => "Eng Dict",
            (Locale::Zh, "app_title") => "英语词典",

            // Tabs
            (Locale::En, "tab_search") => "Search",
            (Locale::Zh, "tab_search") => "搜索",
            (Locale::En, "tab_history") => "History",
            (Locale::Zh, "tab_history") => "历史",
            (Locale::En, "tab_saved") => "Saved",
            (Locale::Zh, "tab_saved") => "收藏",
            (Locale::En, "tab_settings") => "Settings",
            (Locale::Zh, "tab_settings") => "设置",

            // Search
            (Locale::En, "search_heading") => "Search",
            (Locale::Zh, "search_heading") => "搜索",
            (Locale::En, "mode_zh_en") => "中文→英文",
            (Locale::Zh, "mode_zh_en") => "中文→英文",
            (Locale::En, "mode_en_en") => "英文→英文",
            (Locale::Zh, "mode_en_en") => "英文→英文",
            (Locale::En, "pinyin") => "Pinyin",
            (Locale::Zh, "pinyin") => "拼音",
            (Locale::En, "search_now") => "Search now",
            (Locale::Zh, "search_now") => "立即搜索",
            (Locale::En, "search_online") => "Search online",
            (Locale::Zh, "search_online") => "在线搜索",
            (Locale::En, "save") => "Save",
            (Locale::Zh, "save") => "收藏",
            (Locale::En, "query_hint") => "Enter Chinese or English…",
            (Locale::Zh, "query_hint") => "输入中文或英文…",

            // History / Saved
            (Locale::En, "history_heading") => "History",
            (Locale::Zh, "history_heading") => "搜索历史",
            (Locale::En, "saved_heading") => "Saved words",
            (Locale::Zh, "saved_heading") => "收藏的单词",
            (Locale::En, "filter_hint") => "Filter…",
            (Locale::Zh, "filter_hint") => "筛选…",
            (Locale::En, "edit_note") => "Edit note",
            (Locale::Zh, "edit_note") => "编辑笔记",
            (Locale::En, "save_note") => "Save note",
            (Locale::Zh, "save_note") => "保存笔记",
            (Locale::En, "delete") => "Delete",
            (Locale::Zh, "delete") => "删除",

            // Settings
            (Locale::En, "settings_heading") => "Settings",
            (Locale::Zh, "settings_heading") => "设置",
            (Locale::En, "language") => "Interface language",
            (Locale::Zh, "language") => "界面语言",
            (Locale::En, "lang_en") => "English",
            (Locale::Zh, "lang_en") => "English",
            (Locale::En, "lang_zh") => "中文",
            (Locale::Zh, "lang_zh") => "中文",
            (Locale::En, "cedict_path") => "CC-CEDICT database path:",
            (Locale::Zh, "cedict_path") => "词典数据库路径：",
            (Locale::En, "reload_dict") => "Reload dictionary",
            (Locale::Zh, "reload_dict") => "重新加载词典",
            (Locale::En, "show_trad") => "Show traditional characters",
            (Locale::Zh, "show_trad") => "显示繁体字",
            (Locale::En, "online_enable") => "Enable online fallback",
            (Locale::Zh, "online_enable") => "启用在线查询",
            (Locale::En, "api_url") => "API URL (empty = mock provider):",
            (Locale::Zh, "api_url") => "API 地址（留空则使用模拟）：",
            (Locale::En, "api_key") => "API key:",
            (Locale::Zh, "api_key") => "API 密钥：",
            (Locale::En, "online_threshold") => "Online threshold",
            (Locale::Zh, "online_threshold") => "在线查询阈值",
            (Locale::En, "save_settings") => "Save settings",
            (Locale::Zh, "save_settings") => "保存设置",
            (Locale::En, "license") => "CC-CEDICT © MDBG / CC BY-SA 3.0",
            (Locale::Zh, "license") => "CC-CEDICT © MDBG / CC BY-SA 3.0",
            (Locale::En, "float_collapse") => "−",
            (Locale::Zh, "float_collapse") => "收起",
            (Locale::En, "float_full") => "Full",
            (Locale::Zh, "float_full") => "完整",
            (Locale::En, "float_open") => "Float",
            (Locale::Zh, "float_open") => "悬浮",

            // Badges
            (Locale::En, "badge_exact") => "exact",
            (Locale::Zh, "badge_exact") => "精确",
            (Locale::En, "badge_common") => "common",
            (Locale::Zh, "badge_common") => "常用",
            (Locale::En, "badge_saved") => "saved",
            (Locale::Zh, "badge_saved") => "已收藏",
            (Locale::En, "badge_online") => "online",
            (Locale::Zh, "badge_online") => "在线",
            (Locale::En, "score") => "score",
            (Locale::Zh, "score") => "评分",

            // Mode labels in history
            (Locale::En, "hist_zh_en") => "ZH→EN",
            (Locale::Zh, "hist_zh_en") => "中→英",
            (Locale::En, "hist_en_en") => "EN→EN",
            (Locale::Zh, "hist_en_en") => "英→英",

            _ => key,
        }
    }

    pub fn status_ready(&self) -> String {
        match self.locale {
            Locale::En => "Ready".into(),
            Locale::Zh => "就绪".into(),
        }
    }

    pub fn status_dict_not_found(&self, path: &str) -> String {
        match self.locale {
            Locale::En => format!("Dictionary not found at {path}"),
            Locale::Zh => format!("未找到词典：{path}"),
        }
    }

    pub fn status_dict_loaded(&self) -> String {
        match self.locale {
            Locale::En => "Dictionary loaded".into(),
            Locale::Zh => "词典已加载".into(),
        }
    }

    pub fn status_cannot_open(&self, path: &str) -> String {
        match self.locale {
            Locale::En => format!("Cannot open {path}"),
            Locale::Zh => format!("无法打开 {path}"),
        }
    }

    pub fn status_load_dict_first(&self) -> String {
        match self.locale {
            Locale::En => "Load a dictionary in Settings first".into(),
            Locale::Zh => "请先在设置中加载词典".into(),
        }
    }

    pub fn status_results(&self, n: usize) -> String {
        match self.locale {
            Locale::En => format!("{n} results"),
            Locale::Zh => format!("{n} 条结果"),
        }
    }

    pub fn status_results_online(&self, n: usize) -> String {
        match self.locale {
            Locale::En => format!("{n} results (incl. online)"),
            Locale::Zh => format!("{n} 条结果（含在线）"),
        }
    }

    pub fn status_searching_online(&self) -> String {
        match self.locale {
            Locale::En => "Searching online…".into(),
            Locale::Zh => "正在在线搜索…".into(),
        }
    }

    pub fn status_saved(&self) -> String {
        match self.locale {
            Locale::En => "Saved".into(),
            Locale::Zh => "已收藏".into(),
        }
    }

    pub fn status_settings_saved(&self) -> String {
        match self.locale {
            Locale::En => "Settings saved".into(),
            Locale::Zh => "设置已保存".into(),
        }
    }

}
