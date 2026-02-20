mod level_settings;

use level_settings::LevelSettings;
use logicpaint::netbpm;
use logicpaint::netppm;
use logicpaint::ui;

use egor::{
    app::{App, WindowEvent},
    input::KeyCode,
};

use egor::{
    app::{
        FrameContext, egui::Align2, egui::ComboBox, egui::Slider, egui::TextEdit, egui::Ui,
        egui::Window, egui::widgets::Button,
    },
    math::{Vec2, vec2},
    render::{Color, Graphics},
};

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

struct EditorGrids {
    pbm_grid: Vec<Vec<bool>>,
    ppm_grid: Vec<Vec<[f32; 4]>>,
    size: Vec2,
    top_left: Vec2,
}

impl Default for EditorGrids {
    fn default() -> EditorGrids {
        let mut pbm_grid = Vec::with_capacity(20);
        let mut ppm_grid = Vec::with_capacity(20);

        for _ in 0..20 {
            let mut pbm_row = Vec::with_capacity(20);
            let mut ppm_row = Vec::with_capacity(20);
            for _ in 0..20 {
                pbm_row.push(false);
                ppm_row.push([0.0, 0.0, 0.0, 1.0]);
            }
            pbm_grid.push(pbm_row);
            ppm_grid.push(ppm_row);
        }

        EditorGrids {
            pbm_grid,
            ppm_grid,
            size: vec2(400., 400.),
            top_left: vec2(400., 120.), // [ 90 + 500 + 100 + 500 + 90  ]
        }
    }
}

