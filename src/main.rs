mod app;
mod config_graph;
mod config_types;
mod preview;
mod theme_catalog;
mod ui;

use eframe::egui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Alacritty Config UI")
            .with_inner_size([1440.0, 920.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Alacritty Config UI",
        options,
        Box::new(|cc| Ok(Box::new(app::AlacrittyConfigApp::new(cc)))),
    )
}
