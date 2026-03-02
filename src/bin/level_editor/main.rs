use logicpaint::base_dir;
use logicpaint::editor_grids::{EditorGrids, save_grid_as_level};
use logicpaint::editor_settings::LevelSettings;
use logicpaint::pop_up::PopUp;
use logicpaint::ui_actions::UiActions;
use logicpaint::ui_actions::{IOWorkerRequest, IOWorkerResponse};

use rfd::FileDialog;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;

use egor::{
    app::{App, WindowEvent},
    input::KeyCode,
};

use egor::{
    app::{egui::Align2, egui::Window},
    render::Color,
};

fn spawn_io_worker(
    start_directory: PathBuf,
) -> (Sender<IOWorkerRequest>, Receiver<IOWorkerResponse>) {
    let (main_thread_sender, main_thread_reciever) = channel::<IOWorkerRequest>();
    let (worker_thread_sender, worker_thread_reciever) = channel::<IOWorkerResponse>();
    thread::spawn(move || {
        while let Ok(request) = main_thread_reciever.recv() {
            match request {
                IOWorkerRequest::Shutdown => break,
                IOWorkerRequest::OpenFileDialog => {
                    let selected_file = FileDialog::new()
                        .add_filter("level", &["level"])
                        .set_directory(&start_directory)
                        .pick_file();
                    if let Some(file) = selected_file {
                        match worker_thread_sender.send(IOWorkerResponse::IoOpenChoice(file)) {
                            Ok(something) => {
                                eprintln!("OK: {something:?}");
                                // TODO dont break thing
                                break;
                            }
                            Err(error) => {
                                // TODO
                                break;
                            }
                        }
                    }
                }
            }
        }
    });
    (main_thread_sender, worker_thread_reciever)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut level_settings = LevelSettings::default();
    let mut grids = EditorGrids::default();
    let mut save_pop_up: Option<PopUp> = None;

    let base = base_dir();
    let (io_sender, io_reciever) = spawn_io_worker(base);

    App::new()
        .window_size(1280, 720)
        .title("Logic Brush Level Editor")
        .run(move |frame_context| {
            for event in &frame_context.events {
                match event {
                    WindowEvent::CloseRequested => {
                        let _ = io_sender.send(IOWorkerRequest::Shutdown);
                        std::process::exit(0);
                    }
                    _ => {}
                }
            }
            if frame_context.input.key_pressed(KeyCode::Escape) {
                let _ = io_sender.send(IOWorkerRequest::Shutdown);
                std::process::exit(0);
            }

            if let Ok(io_response) = io_reciever.try_recv() {
                match io_response {
                    IOWorkerResponse::IoOpenChoice(to_open) => {
                        eprintln!("Open plz {:?}", to_open);
                    }
                }
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
                            match io_sender.send(IOWorkerRequest::OpenFileDialog) {
                                Ok(_) => {}
                                Err(errr) => {
                                    eprintln!("Uh oh {:?}", errr)
                                }
                            }
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
