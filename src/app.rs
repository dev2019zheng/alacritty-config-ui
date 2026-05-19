#![allow(deprecated)]

use std::path::PathBuf;

use anyhow::{Context, Result};
use eframe::egui::{self, Color32, RichText};
use font_kit::source::SystemSource;

use crate::config_graph::{ConfigGraph, ThemeDocumentConfig};
use crate::config_types::{
    CellColor, CursorBlinking, CursorShape, Decorations, OptionAsAlt, Rgb, ViModeStyle,
};
use crate::preview;
use crate::theme_catalog::{ThemeCatalog, ThemeEntry};
use crate::ui::{self, EditorPanel};

pub struct AlacrittyConfigApp {
    graph: Option<ConfigGraph>,
    catalog: ThemeCatalog,
    fonts: Vec<String>,
    selected_panel: EditorPanel,
    theme_search: String,
    detached_preview: bool,
    status: String,
    error: Option<String>,
}

impl AlacrittyConfigApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let fonts = load_system_fonts();
        let mut app = Self {
            graph: None,
            catalog: ThemeCatalog::default(),
            fonts,
            selected_panel: EditorPanel::ThemeBrowser,
            theme_search: String::new(),
            detached_preview: false,
            status: "Loading ~/.config/alacritty/alacritty.toml".to_owned(),
            error: None,
        };
        if let Err(err) = app.load_graph(None) {
            app.error = Some(err.to_string());
            app.status = "Failed to load the default config".to_owned();
        }
        app
    }

    fn load_graph(&mut self, path: Option<PathBuf>) -> Result<()> {
        let graph = match path {
            Some(path) => ConfigGraph::load(path)?,
            None => ConfigGraph::load_default()?,
        };
        self.catalog = ThemeCatalog::load(&graph.config_dir())?;
        self.status = format!("Loaded {} theme entries", self.catalog.entries.len());
        self.error = None;
        self.graph = Some(graph);
        Ok(())
    }

    fn reload_graph(&mut self) {
        let result = if let Some(graph) = &mut self.graph {
            graph.reload().and_then(|_| {
                self.catalog = ThemeCatalog::load(&graph.config_dir())?;
                Ok(())
            })
        } else {
            self.load_graph(None)
        };

        match result {
            Ok(()) => {
                self.error = None;
                self.status = "Reloaded configuration from disk".to_owned();
            }
            Err(err) => self.error = Some(err.to_string()),
        }
    }

    fn save_graph(&mut self) {
        let Some(graph) = &mut self.graph else {
            return;
        };

        match graph
            .save()
            .and_then(|_| ThemeCatalog::load(&graph.config_dir()))
        {
            Ok(catalog) => {
                self.catalog = catalog;
                self.error = None;
                self.status = format!("Saved {}", graph.root_path.display());
            }
            Err(err) => self.error = Some(err.to_string()),
        }
    }

    fn open_config_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Open Alacritty root config")
            .add_filter("TOML", &["toml"])
            .pick_file()
            && let Err(err) = self.load_graph(Some(path))
        {
            self.error = Some(err.to_string());
        }
    }

    fn open_base_import_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Select base config import")
            .add_filter("TOML", &["toml"])
            .pick_file()
            && let Some(graph) = &mut self.graph
        {
            graph.base_path = Some(path.clone());
            graph.merged.general.import = graph.current_imports();
            graph.dirty = true;
            self.error = None;
            self.status = format!("Staged new base import {}", path.display());
        }
    }

    fn open_theme_import_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Select active theme import")
            .add_filter("TOML", &["toml"])
            .pick_file()
        {
            match std::fs::read_to_string(&path)
                .with_context(|| format!("failed to read {}", path.display()))
                .and_then(|raw| {
                    toml::from_str::<ThemeDocumentConfig>(&raw)
                        .with_context(|| format!("failed to parse {}", path.display()))
                }) {
                Ok(fragment) => {
                    if let Some(graph) = &mut self.graph {
                        graph.activate_theme_fragment(path.clone(), fragment);
                        self.error = None;
                        self.status = format!("Staged new active theme import {}", path.display());
                    }
                }
                Err(err) => self.error = Some(err.to_string()),
            }
        }
    }

    fn apply_theme_entry(&mut self, entry: ThemeEntry) {
        let Some(graph) = &mut self.graph else {
            return;
        };

        let mut fragment = graph.current_theme_fragment();
        if let Some(window) = entry.fragment.window {
            fragment.window = Some(window);
        }
        if let Some(font) = entry.fragment.font {
            fragment.font = Some(font);
        }
        if let Some(colors) = entry.fragment.colors {
            fragment.colors = Some(colors);
        }

        let target_path = if entry.source.is_preset() {
            self.catalog
                .suggested_custom_path(&graph.config_dir(), &entry.name)
        } else {
            entry.path.clone()
        };

        graph.activate_theme_fragment(target_path.clone(), fragment);
        self.selected_panel = EditorPanel::Colors;
        self.error = None;
        self.status = if entry.source.is_preset() {
            format!(
                "Preset '{}' staged into {}. Save to write the custom theme and activate it.",
                entry.name,
                target_path.display()
            )
        } else {
            format!(
                "Activated '{}' from {}. Save to persist the import.",
                entry.name,
                target_path.display()
            )
        };
    }

    fn show_top_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                if ui.button("Open config…").clicked() {
                    self.open_config_dialog();
                }
                if ui.button("Reload").clicked() {
                    self.reload_graph();
                }
                if ui.button("Save").clicked() {
                    self.save_graph();
                }
                ui.checkbox(&mut self.detached_preview, "Detached preview");
                ui.separator();
                ui.label(&self.status);
            });

            if let Some(graph) = &self.graph {
                ui.small(format!(
                    "root: {}{}",
                    graph.root_path.display(),
                    if graph.dirty {
                        " • unsaved changes"
                    } else {
                        ""
                    }
                ));
            }

            if let Some(error) = &self.error {
                ui.colored_label(Color32::from_rgb(0xFF, 0x8B, 0x8B), error);
            }
        });
    }

    fn show_main_ui(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("sidebar")
            .resizable(false)
            .default_width(180.0)
            .show(ctx, |ui| ui::sidebar(ui, &mut self.selected_panel));

        egui::SidePanel::right("preview")
            .default_width(420.0)
            .show(ctx, |ui| {
                ui.heading("Live preview");
                ui.separator();
                if let Some(graph) = &self.graph {
                    preview::render_preview(ui, &graph.merged);
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.graph.is_none() {
                ui.heading("No config loaded");
                return;
            }

            match self.selected_panel {
                EditorPanel::ThemeBrowser => self.render_theme_browser(ui),
                EditorPanel::General => self.render_general_panel(ui),
                EditorPanel::Window => self.render_window_panel(ui),
                EditorPanel::Font => self.render_font_panel(ui),
                EditorPanel::Colors => self.render_colors_panel(ui),
                EditorPanel::Cursor => self.render_cursor_panel(ui),
                EditorPanel::Selection => self.render_selection_panel(ui),
                EditorPanel::Scrolling => self.render_scrolling_panel(ui),
            }
        });
    }

    fn maybe_show_detached_preview(&mut self, ctx: &egui::Context) {
        if !self.detached_preview {
            return;
        }
        let Some(graph) = &self.graph else {
            return;
        };
        let preview_config = graph.merged.clone();
        ctx.show_viewport_deferred(
            egui::ViewportId::from_hash_of("alacritty-preview-window"),
            egui::ViewportBuilder::default()
                .with_title("Alacritty Preview")
                .with_inner_size([640.0, 420.0]),
            move |ctx, _class| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    preview::render_preview(ui, &preview_config);
                });
            },
        );
    }

    fn graph_mut(&mut self) -> &mut ConfigGraph {
        self.graph.as_mut().expect("graph checked before use")
    }

    fn render_general_panel(&mut self, ui: &mut egui::Ui) {
        let mut choose_base = false;
        let mut choose_theme = false;

        {
            let graph = self.graph_mut();
            ui.heading("General");
            ui.separator();
            graph.dirty |= ui
                .checkbox(
                    &mut graph.merged.general.live_config_reload,
                    "Enable live_config_reload",
                )
                .changed();

            let base_display = graph
                .base_path
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "<none>".to_owned());
            let theme_display = graph
                .active_theme_path
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "<none>".to_owned());

            ui.label("Import slots");
            ui.horizontal(|ui| {
                ui.label("Base");
                ui.monospace(base_display);
                if ui.button("Choose…").clicked() {
                    choose_base = true;
                }
            });
            ui.horizontal(|ui| {
                ui.label("Theme");
                ui.monospace(theme_display);
                if ui.button("Choose…").clicked() {
                    choose_theme = true;
                }
            });
        }

        if choose_base {
            self.open_base_import_dialog();
        }
        if choose_theme {
            self.open_theme_import_dialog();
        }
    }

    fn render_window_panel(&mut self, ui: &mut egui::Ui) {
        let graph = self.graph_mut();
        let window = &mut graph.merged.window;
        ui.heading("Window");
        ui.separator();
        graph.dirty |= ui
            .add(egui::Slider::new(&mut window.padding.x, 0..=64).text("Padding X"))
            .changed();
        graph.dirty |= ui
            .add(egui::Slider::new(&mut window.padding.y, 0..=64).text("Padding Y"))
            .changed();
        graph.dirty |= ui
            .checkbox(&mut window.dynamic_padding, "Dynamic padding")
            .changed();
        graph.dirty |= ui.checkbox(&mut window.blur, "Blur").changed();
        graph.dirty |= ui
            .checkbox(&mut window.dynamic_title, "Dynamic title")
            .changed();
        graph.dirty |= ui
            .add(egui::Slider::new(&mut window.opacity, 0.0..=1.0).text("Opacity"))
            .changed();

        let current_decorations = window.decorations;
        egui::ComboBox::from_label("Decorations")
            .selected_text(format!("{:?}", window.decorations))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut window.decorations, Decorations::Full, "Full");
                ui.selectable_value(
                    &mut window.decorations,
                    Decorations::Transparent,
                    "Transparent",
                );
                ui.selectable_value(
                    &mut window.decorations,
                    Decorations::Buttonless,
                    "Buttonless",
                );
                ui.selectable_value(&mut window.decorations, Decorations::None, "None");
            });
        graph.dirty |= current_decorations != window.decorations;

        let current_option_as_alt = window.option_as_alt;
        egui::ComboBox::from_label("Option as Alt")
            .selected_text(format!("{:?}", window.option_as_alt))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut window.option_as_alt, OptionAsAlt::OnlyLeft, "OnlyLeft");
                ui.selectable_value(
                    &mut window.option_as_alt,
                    OptionAsAlt::OnlyRight,
                    "OnlyRight",
                );
                ui.selectable_value(&mut window.option_as_alt, OptionAsAlt::Both, "Both");
                ui.selectable_value(&mut window.option_as_alt, OptionAsAlt::None, "None");
            });
        graph.dirty |= current_option_as_alt != window.option_as_alt;
    }

    fn render_font_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Font");
        ui.separator();
        let fonts = self.fonts.clone();
        let graph = self.graph_mut();
        graph.dirty |= edit_font_face(ui, "Normal", &mut graph.merged.font.normal, &fonts);
        graph.dirty |= edit_font_face(ui, "Bold", &mut graph.merged.font.bold, &fonts);
        graph.dirty |= edit_font_face(ui, "Italic", &mut graph.merged.font.italic, &fonts);
        graph.dirty |= edit_font_face(
            ui,
            "Bold Italic",
            &mut graph.merged.font.bold_italic,
            &fonts,
        );
        graph.dirty |= ui
            .add(egui::Slider::new(&mut graph.merged.font.size, 8.0..=28.0).text("Size"))
            .changed();
        graph.dirty |= ui
            .checkbox(
                &mut graph.merged.font.builtin_box_drawing,
                "Builtin box drawing",
            )
            .changed();
    }

    fn render_colors_panel(&mut self, ui: &mut egui::Ui) {
        let graph = self.graph_mut();
        ui.heading("Colors");
        ui.separator();
        graph.dirty |= ui
            .checkbox(
                &mut graph.merged.colors.draw_bold_text_with_bright_colors,
                "Draw bold text with bright colors",
            )
            .changed();

        ui.collapsing("Primary", |ui| {
            graph.dirty |= edit_rgb(
                ui,
                "Background",
                &mut graph.merged.colors.primary.background,
            );
            graph.dirty |= edit_rgb(
                ui,
                "Foreground",
                &mut graph.merged.colors.primary.foreground,
            );
            if let Some(dim) = &mut graph.merged.colors.primary.dim_foreground {
                graph.dirty |= edit_rgb(ui, "Dim foreground", dim);
            }
            if let Some(bright) = &mut graph.merged.colors.primary.bright_foreground {
                graph.dirty |= edit_rgb(ui, "Bright foreground", bright);
            }
        });

        ui.collapsing("Cursor", |ui| {
            graph.dirty |= edit_cell_color(ui, "Text", &mut graph.merged.colors.cursor.text);
            graph.dirty |= edit_cell_color(ui, "Cursor", &mut graph.merged.colors.cursor.cursor);
        });

        ui.collapsing("Selection", |ui| {
            graph.dirty |= edit_cell_color(ui, "Text", &mut graph.merged.colors.selection.text);
            graph.dirty |= edit_cell_color(
                ui,
                "Background",
                &mut graph.merged.colors.selection.background,
            );
        });

        ui.collapsing("Normal ANSI", |ui| {
            graph.dirty |= edit_named_colors(ui, &mut graph.merged.colors.normal);
        });
        ui.collapsing("Bright ANSI", |ui| {
            graph.dirty |= edit_named_colors(ui, &mut graph.merged.colors.bright);
        });
    }

    fn render_cursor_panel(&mut self, ui: &mut egui::Ui) {
        let graph = self.graph_mut();
        ui.heading("Cursor");
        ui.separator();
        let cursor = &mut graph.merged.cursor;
        let current_shape = cursor.style.shape;
        egui::ComboBox::from_label("Shape")
            .selected_text(format!("{:?}", cursor.style.shape))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut cursor.style.shape, CursorShape::Block, "Block");
                ui.selectable_value(&mut cursor.style.shape, CursorShape::Underline, "Underline");
                ui.selectable_value(&mut cursor.style.shape, CursorShape::Beam, "Beam");
            });
        graph.dirty |= current_shape != cursor.style.shape;

        let current_blinking = cursor.style.blinking;
        egui::ComboBox::from_label("Blinking")
            .selected_text(format!("{:?}", cursor.style.blinking))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut cursor.style.blinking, CursorBlinking::Never, "Never");
                ui.selectable_value(&mut cursor.style.blinking, CursorBlinking::Off, "Off");
                ui.selectable_value(&mut cursor.style.blinking, CursorBlinking::On, "On");
                ui.selectable_value(&mut cursor.style.blinking, CursorBlinking::Always, "Always");
            });
        graph.dirty |= current_blinking != cursor.style.blinking;

        graph.dirty |= ui
            .checkbox(&mut cursor.unfocused_hollow, "Unfocused hollow")
            .changed();
        graph.dirty |= ui
            .add(egui::Slider::new(&mut cursor.thickness, 0.05..=0.5).text("Thickness"))
            .changed();

        let current_vi_mode = cursor.vi_mode_style.clone();
        let mut vi_none = matches!(cursor.vi_mode_style, ViModeStyle::None);
        if ui.checkbox(&mut vi_none, "Disable vi_mode_style").changed() {
            cursor.vi_mode_style = if vi_none {
                ViModeStyle::None
            } else {
                ViModeStyle::Style(cursor.style.clone())
            };
        }
        graph.dirty |= current_vi_mode != cursor.vi_mode_style;
    }

    fn render_selection_panel(&mut self, ui: &mut egui::Ui) {
        let graph = self.graph_mut();
        ui.heading("Selection");
        ui.separator();
        graph.dirty |= ui
            .text_edit_singleline(&mut graph.merged.selection.semantic_escape_chars)
            .changed();
        graph.dirty |= ui
            .checkbox(
                &mut graph.merged.selection.save_to_clipboard,
                "Save to clipboard",
            )
            .changed();
    }

    fn render_scrolling_panel(&mut self, ui: &mut egui::Ui) {
        let graph = self.graph_mut();
        ui.heading("Scrolling");
        ui.separator();
        graph.dirty |= ui
            .add(
                egui::Slider::new(&mut graph.merged.scrolling.history, 0..=100_000).text("History"),
            )
            .changed();
        graph.dirty |= ui
            .add(
                egui::Slider::new(&mut graph.merged.scrolling.multiplier, 1..=20)
                    .text("Multiplier"),
            )
            .changed();
    }

    fn render_theme_browser(&mut self, ui: &mut egui::Ui) {
        ui.heading("Theme browser");
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Search");
            ui.text_edit_singleline(&mut self.theme_search);
        });
        ui.add_space(8.0);

        let active_theme = self
            .graph
            .as_ref()
            .and_then(|graph| graph.active_theme_path.clone());
        let filter = self.theme_search.trim().to_lowercase();
        let mut pending = None;

        egui::ScrollArea::vertical().show(ui, |ui| {
            for entry in &self.catalog.entries {
                if !filter.is_empty() && !entry.name.to_lowercase().contains(&filter) {
                    continue;
                }

                egui::Frame::group(ui.style()).show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new(&entry.name).strong());
                        ui.small(entry.source.label());
                        if active_theme.as_ref() == Some(&entry.path) {
                            ui.small("active");
                        }
                    });
                    ui.small(entry.path.display().to_string());

                    if let Some(colors) = &entry.fragment.colors {
                        ui.horizontal_wrapped(|ui| {
                            for rgb in [
                                colors.normal.red,
                                colors.normal.green,
                                colors.normal.blue,
                                colors.normal.magenta,
                                colors.normal.cyan,
                                colors.normal.yellow,
                            ] {
                                let (rect, _) = ui.allocate_exact_size(
                                    egui::vec2(20.0, 14.0),
                                    egui::Sense::hover(),
                                );
                                ui.painter().rect_filled(
                                    rect,
                                    3.0,
                                    Color32::from_rgb(rgb.r, rgb.g, rgb.b),
                                );
                            }
                        });
                    }

                    ui.horizontal(|ui| {
                        let label = if entry.source.is_preset() {
                            "Copy & Customize"
                        } else {
                            "Activate"
                        };
                        if ui.button(label).clicked() {
                            pending = Some(entry.clone());
                        }
                    });
                });
                ui.add_space(6.0);
            }
        });

        if let Some(entry) = pending {
            self.apply_theme_entry(entry);
        }
    }
}

impl eframe::App for AlacrittyConfigApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if ctx.input(|input| input.key_pressed(egui::Key::S) && input.modifiers.command) {
            self.save_graph();
        }
        self.show_top_bar(&ctx);
        self.show_main_ui(&ctx);
        self.maybe_show_detached_preview(&ctx);
    }
}

fn load_system_fonts() -> Vec<String> {
    let mut families = SystemSource::new().all_families().unwrap_or_default();
    families.sort();
    families.dedup();
    families
}

fn edit_font_face(
    ui: &mut egui::Ui,
    label: &str,
    face: &mut crate::config_types::FontFace,
    fonts: &[String],
) -> bool {
    let mut changed = false;
    ui.group(|ui| {
        ui.label(RichText::new(label).strong());
        ui.horizontal(|ui| {
            ui.label("Family");
            egui::ComboBox::from_id_salt(format!("font-family-{label}"))
                .selected_text(&face.family)
                .width(260.0)
                .show_ui(ui, |ui| {
                    for family in fonts {
                        changed |= ui
                            .selectable_value(&mut face.family, family.clone(), family)
                            .changed();
                    }
                });
        });
        ui.horizontal(|ui| {
            ui.label("Style");
            changed |= ui.text_edit_singleline(&mut face.style).changed();
        });
    });
    changed
}

fn edit_named_colors(ui: &mut egui::Ui, colors: &mut crate::config_types::NamedColors) -> bool {
    let mut changed = false;
    changed |= edit_rgb(ui, "Black", &mut colors.black);
    changed |= edit_rgb(ui, "Red", &mut colors.red);
    changed |= edit_rgb(ui, "Green", &mut colors.green);
    changed |= edit_rgb(ui, "Yellow", &mut colors.yellow);
    changed |= edit_rgb(ui, "Blue", &mut colors.blue);
    changed |= edit_rgb(ui, "Magenta", &mut colors.magenta);
    changed |= edit_rgb(ui, "Cyan", &mut colors.cyan);
    changed |= edit_rgb(ui, "White", &mut colors.white);
    changed
}

fn edit_rgb(ui: &mut egui::Ui, label: &str, rgb: &mut Rgb) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(label);
        let mut color = Color32::from_rgb(rgb.r, rgb.g, rgb.b);
        changed |= ui.color_edit_button_srgba(&mut color).changed();
        if changed {
            *rgb = Rgb::new(color.r(), color.g(), color.b());
        }
        ui.monospace(rgb.to_hex());
    });
    changed
}

fn edit_cell_color(ui: &mut egui::Ui, label: &str, value: &mut CellColor) -> bool {
    let mut changed = false;
    ui.group(|ui| {
        ui.label(RichText::new(label).strong());
        let mut mode = match value {
            CellColor::CellBackground => 0,
            CellColor::CellForeground => 1,
            CellColor::Rgb(_) => 2,
        };
        egui::ComboBox::from_id_salt(format!("cell-color-{label}"))
            .selected_text(match mode {
                0 => "CellBackground",
                1 => "CellForeground",
                _ => "Custom",
            })
            .show_ui(ui, |ui| {
                changed |= ui
                    .selectable_value(&mut mode, 0, "CellBackground")
                    .changed();
                changed |= ui
                    .selectable_value(&mut mode, 1, "CellForeground")
                    .changed();
                changed |= ui.selectable_value(&mut mode, 2, "Custom").changed();
            });

        if changed {
            *value = match mode {
                0 => CellColor::CellBackground,
                1 => CellColor::CellForeground,
                _ => match *value {
                    CellColor::Rgb(rgb) => CellColor::Rgb(rgb),
                    _ => CellColor::Rgb(Rgb::new(0x89, 0xDD, 0xFF)),
                },
            };
        }

        if let CellColor::Rgb(rgb) = value {
            changed |= edit_rgb(ui, "Custom", rgb);
        }
    });
    changed
}
