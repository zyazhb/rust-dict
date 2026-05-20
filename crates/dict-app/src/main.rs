mod app;
mod fonts;
mod i18n;

use app::DictApp;
use eframe::egui;

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
            cc.egui_ctx.options_mut(|o| {
                o.max_passes = std::num::NonZeroUsize::new(1).unwrap();
            });
            let app = DictApp::new(cc);
            cc.egui_ctx
                .send_viewport_cmd(egui::ViewportCommand::Title(app.window_title()));
            Ok(Box::new(app))
        }),
    )
}
