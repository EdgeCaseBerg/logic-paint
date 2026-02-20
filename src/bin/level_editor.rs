mod level_settings;

use level_settings::LevelSettings;
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
        FrameContext, egui::ComboBox, egui::Slider, egui::TextEdit, egui::Ui, egui::Window,
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

            Window::new("Settings").show(egui_ctx, |ui| level_settings.ui(ui));
        });

    Ok(())
}
