use logicpaint::editor_grids::{EditorGrids, save_grid_as_level};
use logicpaint::level_settings::LevelSettings;
use logicpaint::levels::Level;
use logicpaint::netbpm::Pbm;
use logicpaint::netppm::Ppm;
use logicpaint::ui_actions::UiActions;
use logicpaint::pop_up::PopUp;

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
    let mut save_pop_up: Option<PopUp> = None;

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

            gfx.clear(Color::new([0.5, 0.5, 0.5, 1.0]));
            let screen_size = gfx.screen_size();
            gfx.camera().target(screen_size / 2.);


            let (mx, my) = input.mouse_position();
            let world_xy = gfx.camera().screen_to_world(Vec2::new(mx, my), screen_size);

            Window::new("Settings")
                .anchor(Align2::LEFT_TOP, egor::app::egui::Vec2::ZERO)
                .default_size([100.0, 500.0])
                .show(egui_ctx, |ui| {
                    grids.ui(frame_context, &mut level_settings);
                    match level_settings.ui(ui) {
                        UiActions::Nothing => {}
                        UiActions::RecomputePalette => {
                            level_settings.refresh_palette_with(grids.unique_colors());
                        }
                        UiActions::SaveLevel => {
                            let level = save_grid_as_level(&level_settings, &grids);
                            match level.save() {
                                Ok(_) => {
                                    save_pop_up = Some(PopUp {
                                        heading: "Saved".to_owned(),
                                        msg: "Your level has been saved".to_owned(),
                                        visible: true,
                                    });
                                },
                                Err(error) => {
                                    save_pop_up = Some(PopUp {
                                        heading: "Error".to_owned(),
                                        msg: format!("Error: {error}").to_owned(),
                                        visible: true,
                                    });
                                }
                            }
                        }
                    }
                    if let Some(popup) = save_pop_up.as_mut() {
                        popup.ui(ui);
                    }
                });
        });

    Ok(())
}
