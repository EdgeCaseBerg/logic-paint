use logicpaint::netbpm;
use logicpaint::netppm;
use logicpaint::ui;

use egor::{
    app::{App, WindowEvent},
    input::KeyCode,
};

use egor::app::egui::Rgba;
use egor::{
    app::{
        FrameContext, egui::ComboBox, egui::Slider, egui::TextEdit, egui::Window,
        egui::widgets::Button,
    },
    math::Vec2,
};

use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut level_settings = LevelSettings::default();

    App::new()
        .window_size(1280, 720)
        .title("Logic Brush Level Editor")
        .run(move |frame_context| {
            for event in &frame_context.events {
                match event {
                    WindowEvent::CloseRequested => {
                        std::process::exit(0);
                    }
                    _ => {}
                }
            }
            if frame_context.input.key_pressed(KeyCode::Escape) {
                std::process::exit(0);
            }

            let gfx = &mut (frame_context.gfx);
            let input = &mut (frame_context.input);
            let egui_ctx = frame_context.egui_ctx;

            let screen_size = gfx.screen_size();
            let (mx, my) = input.mouse_position();
            let world_xy = gfx.camera().screen_to_world(Vec2::new(mx, my), screen_size);

            Window::new("Settings").show(egui_ctx, |ui| {
                ui.add(Slider::new(&mut level_settings.width, 5..=20).text("Level Width"));
                ui.add(Slider::new(&mut level_settings.height, 5..=20).text("Level Height"));

                ui.separator();

                ui.label("Color: ");
                ui.color_edit_button_rgba_unmultiplied(&mut level_settings.current_color);

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Filename (no extension)");
                    ui.add(TextEdit::singleline(&mut level_settings.filename));
                    if ui.button("Save").clicked() {
                        // TODO: Save current
                    }
                });

                ui.separator();
                ui.label("Current Palette:");
                for mut color in &mut level_settings.lru_colors {
                    ui.horizontal(|ui| {
                        // ui.color_edit_button_rgba_unmultiplied(&mut color);
                        ui.scope(|ui| {
                            //https://github.com/emilk/egui/discussions/3356
                            // https://docs.rs/egui/latest/egui/style/struct.WidgetVisuals.html
                            ui.style_mut().visuals.widgets.inactive.weak_bg_fill =
                                Rgba::from_rgba_unmultiplied(
                                    color[0], color[1], color[2], color[3],
                                )
                                .into();
                            // TODO: handle on click
                            ui.button("Select")
                        })
                    });
                }
            });
        });

    Ok(())
}

struct LevelSettings {
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
