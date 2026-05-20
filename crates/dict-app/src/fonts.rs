use std::fs::File;
use std::path::Path;
use std::sync::OnceLock;

use eframe::egui;
use memmap2::Mmap;

/// Avoid loading multi‑tens‑of‑MB system font collections into the heap.
const MAX_FONT_BYTES: u64 = 30_000_000;

const FONT_CANDIDATES: &[&str] = &[
    "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
    "/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf",
    "/System/Library/Fonts/Hiragino Sans GB.ttc",
    "/System/Library/Fonts/Supplemental/Songti.ttc",
    "/System/Library/Fonts/STHeiti Light.ttc",
    "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
    "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
];

static CJK_FONT: OnceLock<Option<egui::FontData>> = OnceLock::new();

fn smallest_font_path() -> Option<&'static str> {
    let mut best: Option<(&str, u64)> = None;
    for path in FONT_CANDIDATES {
        let p = Path::new(path);
        if !p.exists() {
            continue;
        }
        let Ok(meta) = std::fs::metadata(p) else {
            continue;
        };
        let size = meta.len();
        if size > MAX_FONT_BYTES {
            continue;
        }
        if best.is_none_or(|(_, s)| size < s) {
            best = Some((path, size));
        }
    }
    best.map(|(p, _)| p)
}

fn load_font_data() -> Option<egui::FontData> {
    let path = smallest_font_path()?;
    let file = File::open(path).ok()?;
    let mmap = unsafe { Mmap::map(&file).ok()? };
    let leaked = Box::leak(Box::new(mmap));
    Some(egui::FontData::from_static(leaked))
}

pub fn setup_cjk_fonts(ctx: &egui::Context) {
    let Some(font_data) = CJK_FONT.get_or_init(load_font_data).as_ref() else {
        return;
    };

    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "cjk".to_owned(),
        std::sync::Arc::new(font_data.clone()),
    );
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "cjk".to_owned());
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("cjk".to_owned());

    ctx.set_fonts(fonts);
}
