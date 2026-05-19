use eframe::egui::{self, Color32, RichText, Stroke};

use crate::config_types::{CellColor, CursorShape, EditorConfig, Rgb};

pub fn render_preview(ui: &mut egui::Ui, config: &EditorConfig) {
    let background = color32(config.colors.primary.background);
    let foreground = color32(config.colors.primary.foreground);
    let border = color32(config.colors.normal.black);

    egui::Frame::canvas(ui.style())
        .fill(background)
        .stroke(Stroke::new(1.0, border))
        .show(ui, |ui| {
            ui.visuals_mut().override_text_color = Some(foreground);
            ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);

            render_title_bar(ui, config);
            ui.add_space(f32::from(config.window.padding.y));

            ui.label(RichText::new("Preview").heading().color(foreground));
            sample_line(
                ui,
                color32(config.colors.normal.cyan),
                foreground,
                "$",
                "cargo test",
            );
            sample_line(
                ui,
                color32(config.colors.normal.green),
                foreground,
                "✓",
                "3 tests passed",
            );
            sample_line(
                ui,
                color32(config.colors.normal.magenta),
                foreground,
                "λ",
                "font = \"JetBrainsMono Nerd Font Mono\"",
            );

            ui.separator();
            ui.label(RichText::new("ANSI palette").strong());
            render_swatches(ui, &config.colors.normal, &config.colors.bright);

            ui.separator();
            render_selection_example(ui, config);
            render_cursor_example(ui, config);
            render_simulated_cards(ui, config);
        });
}

fn render_title_bar(ui: &mut egui::Ui, config: &EditorConfig) {
    ui.horizontal(|ui| {
        dot(ui, Color32::from_rgb(0xFF, 0x5F, 0x57));
        dot(ui, Color32::from_rgb(0xFE, 0xBC, 0x2E));
        dot(ui, Color32::from_rgb(0x28, 0xC8, 0x40));
        ui.add_space(8.0);
        ui.label(
            RichText::new(format!(
                "alacritty • {} • opacity {:.2}",
                config.font.normal.family, config.window.opacity
            ))
            .strong(),
        );
    });
}

fn render_swatches(
    ui: &mut egui::Ui,
    normal: &crate::config_types::NamedColors,
    bright: &crate::config_types::NamedColors,
) {
    let normal_values = [
        normal.black,
        normal.red,
        normal.green,
        normal.yellow,
        normal.blue,
        normal.magenta,
        normal.cyan,
        normal.white,
    ];
    let bright_values = [
        bright.black,
        bright.red,
        bright.green,
        bright.yellow,
        bright.blue,
        bright.magenta,
        bright.cyan,
        bright.white,
    ];

    ui.horizontal_wrapped(|ui| {
        for value in normal_values.into_iter().chain(bright_values) {
            let (rect, _) = ui.allocate_exact_size(egui::vec2(28.0, 18.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, 4.0, color32(value));
        }
    });
}

fn render_selection_example(ui: &mut egui::Ui, config: &EditorConfig) {
    let selection_background = resolve_cell_color(config.colors.selection.background, config);
    let selection_foreground = resolve_cell_color(config.colors.selection.text, config);
    egui::Frame::canvas(ui.style())
        .fill(selection_background)
        .stroke(Stroke::NONE)
        .show(ui, |ui| {
            ui.label(
                RichText::new("selected text example")
                    .color(selection_foreground)
                    .strong(),
            );
        });
}

fn render_cursor_example(ui: &mut egui::Ui, config: &EditorConfig) {
    ui.label(RichText::new("Cursor").strong());
    let cursor_color = resolve_cell_color(config.colors.cursor.cursor, config);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(120.0, 28.0), egui::Sense::hover());
    ui.painter().rect_stroke(
        rect,
        4.0,
        Stroke::new(1.0, cursor_color),
        egui::StrokeKind::Middle,
    );

    match config.cursor.style.shape {
        CursorShape::Block => {
            let block =
                egui::Rect::from_min_size(rect.min + egui::vec2(8.0, 6.0), egui::vec2(18.0, 16.0));
            ui.painter().rect_filled(block, 2.0, cursor_color);
        }
        CursorShape::Underline => {
            ui.painter().line_segment(
                [
                    rect.left_bottom() + egui::vec2(8.0, -6.0),
                    rect.left_bottom() + egui::vec2(28.0, -6.0),
                ],
                Stroke::new((config.cursor.thickness * 10.0).max(1.0), cursor_color),
            );
        }
        CursorShape::Beam => {
            ui.painter().line_segment(
                [
                    rect.left_top() + egui::vec2(12.0, 5.0),
                    rect.left_bottom() + egui::vec2(12.0, -5.0),
                ],
                Stroke::new((config.cursor.thickness * 10.0).max(1.0), cursor_color),
            );
        }
    }

    let blinking = format!("blinking: {:?}", config.cursor.style.blinking);
    ui.label(blinking);
}

fn render_simulated_cards(ui: &mut egui::Ui, config: &EditorConfig) {
    ui.separator();
    ui.label(RichText::new("Simulated preview").strong());
    ui.horizontal_wrapped(|ui| {
        info_card(
            ui,
            "Window blur",
            if config.window.blur {
                "Enabled"
            } else {
                "Disabled"
            },
        );
        info_card(
            ui,
            "Decorations",
            &format!("{:?}", config.window.decorations),
        );
        info_card(
            ui,
            "Option as Alt",
            &format!("{:?}", config.window.option_as_alt),
        );
        info_card(ui, "Scroll history", &config.scrolling.history.to_string());
        info_card(
            ui,
            "Scroll multiplier",
            &config.scrolling.multiplier.to_string(),
        );
    });
}

fn info_card(ui: &mut egui::Ui, title: &str, value: &str) {
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.set_min_width(110.0);
        ui.label(RichText::new(title).strong());
        ui.label(value);
    });
}

fn sample_line(ui: &mut egui::Ui, accent: Color32, foreground: Color32, prefix: &str, body: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(prefix).color(accent).strong());
        ui.label(RichText::new(body).color(foreground));
    });
}

fn dot(ui: &mut egui::Ui, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
    ui.painter().circle_filled(rect.center(), 4.0, color);
}

fn resolve_cell_color(color: CellColor, config: &EditorConfig) -> Color32 {
    match color {
        CellColor::CellBackground => color32(config.colors.primary.background),
        CellColor::CellForeground => color32(config.colors.primary.foreground),
        CellColor::Rgb(rgb) => color32(rgb),
    }
}

fn color32(rgb: Rgb) -> Color32 {
    Color32::from_rgb(rgb.r, rgb.g, rgb.b)
}
