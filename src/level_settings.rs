use egor::app::egui::Rgba;
use egor::app::{egui::Slider, egui::TextEdit, egui::Ui};

use crate::ui_actions::UiActions;

pub struct LevelSettings {
    pub width: usize,
    pub height: usize,
    pub filename: String,
    pub current_color: [f32; 4],
    pub palette: Vec<[f32; 4]>,
    pub max_colors: usize,
}

impl Default for LevelSettings {
    fn default() -> LevelSettings {
        LevelSettings {
            width: 10,
            height: 10,
            filename: String::from("test"),
            current_color: [0., 0., 0., 1.0],
            palette: vec![[0., 0., 0., 1.0]],
            max_colors: 12,
        }
    }
}

impl LevelSettings {
    pub fn ui(&mut self, ui: &mut Ui) -> UiActions {
        let mut result = UiActions::Nothing;
        ui.heading("Instructions");
        ui.label("Left click to apply changes to the grids, right click to remove");
        ui.label("The PBM grid defines the cells to fill for the puzzle");
        ui.label("The PPM grid defines the pixel art reward. ");
        ui.separator();
        ui.add(Slider::new(&mut self.width, 5..=20).text("Level Width"));
        ui.add(Slider::new(&mut self.height, 5..=20).text("Level Height"));

        ui.separator();

        ui.label("Color: ");
        let previous_color = self.current_color.clone();
        ui.color_edit_button_rgba_unmultiplied(&mut self.current_color);
        if previous_color != self.current_color {
            result = UiActions::RecomputePalette;
        }

        ui.separator();

        ui.heading("Warning:");
        ui.label("Saving will overwrite any file with the same name");
        ui.horizontal(|ui| {
            ui.label("Filename (do not enter an extension)");
            ui.add(TextEdit::singleline(&mut self.filename).desired_width(100.));
            if ui.button("Save").clicked() {
                result = UiActions::SaveLevel;
            }
        });

        ui.separator();
        ui.label("Current Palette:");
        for color in &mut self.palette {
            ui.horizontal(|ui| {
                ui.scope(|ui| {
                    ui.style_mut().visuals.widgets.inactive.weak_bg_fill =
                        Rgba::from_rgba_unmultiplied(color[0], color[1], color[2], color[3]).into();

                    if ui.button("Select").clicked() {
                        self.current_color = color.clone();
                        result = UiActions::RecomputePalette;
                    }
                })
            });
        }
        result
    }

    pub fn refresh_palette_with(&mut self, unique_colors: Vec<[f32; 4]>) {
        self.palette = unique_colors;
    }
}
