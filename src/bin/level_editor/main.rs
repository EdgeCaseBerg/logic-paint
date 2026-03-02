use logicpaint::editor_grids::{EditorGrids, save_grid_as_level};
use logicpaint::editor_settings::LevelSettings;
use logicpaint::pop_up::PopUp;
use logicpaint::ui_actions::UiActions;

use std::future::Future;
use std::thread;
use std::sync::mpsc::{ Receiver, Sender, channel };
use std::path::PathBuf;
use rfd::FileHandle;
use rfd::FileDialog;

use egor::{
    app::{App, WindowEvent},
    input::KeyCode,
};

use egor::{
    app::{egui::Align2, egui::Window},
    render::Color,
};

struct IoThreadFacade {
    open_file_channel: (Sender<PathBuf>, Receiver<PathBuf>),
    file_to_load: Option<PathBuf>
}

fn execute<F: Future<Output = ()> + Send + 'static>(f: F) {
    // this is stupid... use any executor of your choice instead
    eprintln!("execute ");
    std::thread::spawn(move || async {
        eprintln!("in async");
        f.await
    });
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut level_settings = LevelSettings::default();
    let mut grids = EditorGrids::default();
    let mut save_pop_up: Option<PopUp> = None;

    let mut io: (Sender<PathBuf>, Receiver<PathBuf>) = channel();

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

            if let Ok(to_open) = io.1.try_recv() {
                eprintln!("Open plz {:?}", to_open);
            }


            let gfx = &mut (frame_context.gfx);
            let egui_ctx = frame_context.egui_ctx;

            gfx.clear(Color::new([0.5, 0.5, 0.5, 1.0]));

            grids.ui(frame_context, &mut level_settings);
            Window::new("Settings")
                .anchor(Align2::LEFT_TOP, egor::app::egui::Vec2::ZERO)
                .default_size([100.0, 500.0])
                .show(egui_ctx, |ui| {
                    match level_settings.ui(ui) {
                        UiActions::Nothing => {}
                        UiActions::RecomputePalette => {
                            level_settings.refresh_palette_with(grids.unique_colors());
                        }
                        UiActions::OpenLevel => {
                            let sender = io.0.clone();
                            std::thread::spawn(move || {
                                let files = FileDialog::new()
                                    .add_filter("level", &["level"])
                                    .set_directory("/") // TODO set to startup area
                                    .pick_file();
                                if let Some(file) = files {
                                    let _ = sender.send(file);
                                }
                            });
                            
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
                                }
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
