use logicpaint::editor_grids::{EditorGrids, save_grid_as_level};
use logicpaint::level_settings::LevelSettings;
use logicpaint::levels::Level;
use logicpaint::netbpm::Pbm;
use logicpaint::netppm::Ppm;
use logicpaint::ui_actions::UiActions;

use egor::{
    app::{App, WindowEvent},
    input::KeyCode,
    input::MouseButton,
};

use egor::{
    app::{FrameContext, egui::Align2, egui::Window},
    math::{Rect, Vec2, vec2},
    render::Color,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut level_settings = LevelSettings::default();
    let mut grids = EditorGrids::default();

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
            gfx.camera().target(screen_size / 2.);

            let (mx, my) = input.mouse_position();
            let world_xy = gfx.camera().screen_to_world(Vec2::new(mx, my), screen_size);

            Window::new("Settings")
                .anchor(Align2::LEFT_TOP, egor::app::egui::Vec2::ZERO)
                .default_size([100.0, 500.0])
                .show(egui_ctx, |ui| {
                    ui.label(format!("{} {} {}", mx, my, world_xy));
                    grids.ui(frame_context, &mut level_settings);
                    match level_settings.ui(ui) {
                        UiActions::Nothing => {}
                        UiActions::SaveLevel => {
                            // TODO: 
                        }
                    }
                });
        });

    Ok(())
}

