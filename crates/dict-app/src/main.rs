mod app;
mod fonts;
mod i18n;

use app::DictApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([960.0, 640.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Eng Dict",
        options,
        Box::new(|cc| {
            fonts::setup_cjk_fonts(&cc.egui_ctx);
            Ok(Box::new(DictApp::new(cc)))
        }),
    )
}
