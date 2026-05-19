use eframe::egui;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditorPanel {
    ThemeBrowser,
    General,
    Window,
    Font,
    Colors,
    Cursor,
    Selection,
    Scrolling,
}

impl EditorPanel {
    pub const ALL: [Self; 8] = [
        Self::ThemeBrowser,
        Self::General,
        Self::Window,
        Self::Font,
        Self::Colors,
        Self::Cursor,
        Self::Selection,
        Self::Scrolling,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::ThemeBrowser => "Themes",
            Self::General => "General",
            Self::Window => "Window",
            Self::Font => "Font",
            Self::Colors => "Colors",
            Self::Cursor => "Cursor",
            Self::Selection => "Selection",
            Self::Scrolling => "Scrolling",
        }
    }
}

pub fn sidebar(ui: &mut egui::Ui, selected: &mut EditorPanel) {
    ui.heading("Panels");
    ui.separator();
    for panel in EditorPanel::ALL {
        if ui
            .selectable_label(*selected == panel, panel.label())
            .clicked()
        {
            *selected = panel;
        }
    }
}
