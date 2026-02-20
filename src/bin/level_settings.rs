use egor::app::egui::Rgba;
use egor::app::{egui::Slider, egui::TextEdit, egui::Ui, egui::widgets::Button};

pub struct LevelSettings {
    width: usize,
    height: usize,
    filename: String,
    current_color: [f32; 4],
    lru_colors: Vec<[f32; 4]>,
    max_colors: usize,
}

impl Default for LevelSettings {
    fn default() -> LevelSettings {
        LevelSettings {
            width: 10,
            height: 10,
            filename: String::from("test"),
            current_color: [0., 0., 0., 1.0],
            lru_colors: vec![[0., 0., 0., 1.0], [1., 0., 0., 1.0], [0., 1., 1., 1.0]],
            max_colors: 12,
        }
    }
}

impl LevelSettings {
    pub fn ui(&mut self, ui: &mut Ui) {
        ui.add(Slider::new(&mut self.width, 5..=20).text("Level Width"));
        ui.add(Slider::new(&mut self.height, 5..=20).text("Level Height"));

        ui.separator();

        ui.label("Color: ");
        ui.color_edit_button_rgba_unmultiplied(&mut self.current_color);

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Filename (no extension)");
            ui.add(TextEdit::singleline(&mut self.filename));
            if ui.button("Save").clicked() {
                // TODO: Save current
            }
        });

        ui.separator();
        ui.label("Current Palette:");
        for color in &mut self.lru_colors {
            ui.horizontal(|ui| {
                // ui.color_edit_button_rgba_unmultiplied(&mut color);
                ui.scope(|ui| {
                    //https://github.com/emilk/egui/discussions/3356
                    // https://docs.rs/egui/latest/egui/style/struct.WidgetVisuals.html
                    ui.style_mut().visuals.widgets.inactive.weak_bg_fill =
                        Rgba::from_rgba_unmultiplied(color[0], color[1], color[2], color[3]).into();
                    if ui.button("Select").clicked() {
                        self.current_color = color.clone();
                    }
                })
            });
        }
    }
}
