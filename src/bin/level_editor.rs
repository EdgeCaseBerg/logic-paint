use logicpaint::netbpm;
use logicpaint::netppm;
use logicpaint::ui;

use egor::{
    app::{App, WindowEvent},
    input::KeyCode,
};

use egor::{
    app::{FrameContext, egui::ComboBox, egui::Slider, egui::Window, egui::TextEdit},
    math::Vec2,
};

use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut level_width = 10;
    let mut level_height = 10;
    let mut current_color = [0., 0., 0., 1.0];
    let mut file_name = String::from("test");

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
                ui.add(Slider::new(&mut level_width, 5..=20).text("Level Width"));
                ui.add(Slider::new(&mut level_height, 5..=20).text("Level Height"));

                ui.separator();

                ui.label("Color: ");
                ui.color_edit_button_rgba_unmultiplied(&mut current_color);

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Filename (no extension)");
                    ui.add(TextEdit::singleline(&mut file_name));
                    if ui.button("Save").clicked() {
                        // TODO: Save current
                    }
                });

                ui.separator();
                ui.label("Current Palette:");

                // TODO: Load color from ppm
            });
        });

    Ok(())
}
