use logicpaint::gamestate;
use logicpaint::levels;
use logicpaint::netbpm;
use logicpaint::netppm;
use logicpaint::pop_up::PopUp;
use logicpaint::screens;
use logicpaint::ui;
use logicpaint::ui::DebugStuff;
use logicpaint::ui::debug_window;

use crate::gamestate::PlayState;

use egor::{
    app::{App, WindowEvent},
    input::KeyCode,
};
use egor::{
    app::{FrameContext, egui::ComboBox, egui::Slider, egui::Window},
    math::Vec2,
};

use crate::screens::{ScreenAction, Screens};
use crate::ui::ColorPalette;

use std::env;
use std::fs::read_to_string;
use std::path::Path;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments: Vec<String> = env::args().collect();
    if arguments.is_empty() {
        eprintln!("{:?}", "pass the pbm data as the first argument.");
        eprintln!("{:?}", "pass the ppm data as the second argument.");
        return Err("Could not load file".into());
    }
    let exe_dir = base_dir();

    let assets = exe_dir.join("assets");
    let level_dir_path = exe_dir.join("levels");

    let mut arguments = arguments.iter();
    arguments.next(); // skip the name of the program being ran
    let filename_pbm = match arguments.next() {
        Some(arg) => arg,
        _ => "./assets/P1.pbm",
    };

    arguments.next(); // skip the name of the program being ran
    let filename_ppm = match arguments.next() {
        Some(arg) => arg,
        _ => "./assets/P3.ppm",
    };

    // todo use options and whatnot to load things up properly and such
    let unknown_ppm = read_to_string(assets.join("unsolved.ppm"))?;
    let unknown_ppm: netppm::Ppm = unknown_ppm.parse()?;
    let test_pbm = read_to_string(filename_pbm)?;
    let test_pbm: netbpm::Pbm = test_pbm.parse()?;
    let win_image = read_to_string(filename_ppm)?;
    let mut win_image: netppm::Ppm = win_image.parse()?;
    let mut game_state: gamestate::PlayState = (&test_pbm).into();
    let mut levels = levels::load_levels_from_dir(&level_dir_path)?;
    let mut palette = ColorPalette::meeks();
    let mut current_screen = Screens::ChooseLevelScreen { page: 0 };
    let mut current_level = levels[0].path.clone();
    let mut wipe_progress = 0.0;
    let mut show_wipe = false;
    let mut last_action = ScreenAction::NoAction;
    let mut debuggable_stuff = DebugStuff::new();
    let mut maybe_popup: Option<PopUp> = None;

    App::new()
        .window_size(1280, 720)
        .title("Logic Brush")
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

            let screen_to_draw = if show_wipe {
                let Screens::WipeScreen {
                    ref from,
                    ref to,
                    duration: _,
                } = current_screen
                else {
                    panic!("screen was not wipe!{:?}", current_screen)
                };
                match last_action {
                    ScreenAction::WipeLeft => from,
                    ScreenAction::WipeRight | ScreenAction::WipeDone => to,
                    _ => &current_screen,
                }
            } else {
                &current_screen
            };

            let mut action = match screen_to_draw {
                Screens::GameScreen => {
                    screens::play_game_screen(&mut game_state, frame_context, &mut palette)
                }
                Screens::WinScreen => screens::win_screen(
                    &mut game_state,
                    &mut win_image,
                    frame_context,
                    &palette,
                    &mut debuggable_stuff,
                ),
                Screens::ChooseLevelScreen { page } => screens::level_select_screen(
                    &levels,
                    *page,
                    frame_context,
                    &mut game_state,
                    &mut win_image,
                    &mut current_level,
                    &unknown_ppm,
                ),
                _ => ScreenAction::NoAction,
            };
            if show_wipe {
                action = screens::wipe_screen(
                    &mut wipe_progress,
                    debuggable_stuff.transition_duration,
                    frame_context,
                    &palette,
                );
            }
            match action {
                ScreenAction::NoAction => {}
                ScreenAction::ChangeScreen { ref to } => {
                    wipe_progress = 0.0;
                    show_wipe = true;
                    current_screen = Screens::WipeScreen {
                        from: Box::new(current_screen.clone()),
                        to: Box::new(to.clone()),
                        duration: debuggable_stuff.transition_duration,
                    };
                }
                ScreenAction::WipeDone => {
                    let Screens::WipeScreen {
                        from: _,
                        ref to,
                        duration,
                    } = current_screen
                    else {
                        panic!("screen was not wipe!{:?}", current_screen)
                    };
                    debuggable_stuff.transition_duration = duration;
                    show_wipe = false;
                    current_screen = *to.clone();
                }
                ScreenAction::NextPage => {
                    let Screens::ChooseLevelScreen { page } = current_screen else {
                        panic!("screen was not level select {:?}", current_screen);
                    };
                    current_screen = Screens::ChooseLevelScreen { page: page + 1 };
                }
                ScreenAction::PreviousPage => {
                    let Screens::ChooseLevelScreen { page } = current_screen else {
                        panic!("screen was not level select {:?}", current_screen);
                    };
                    current_screen = Screens::ChooseLevelScreen { page: page - 1 };
                }
                ScreenAction::MarkLevelComplete => {
                    let found_level = levels.iter_mut().find(|level| level.path == current_level);
                    if let Some(played_level) = found_level {
                        if !played_level.completed {
                            match played_level.mark_completed() {
                                Ok(_) => {}
                                Err(error) => {
                                    maybe_popup = Some(PopUp {
                                        heading: "Error".to_owned(),
                                        msg: format!("There was a problem {}", error).to_owned(),
                                        visible: true,
                                    });
                                }
                            }
                        }
                    }
                }
                _ => {}
            };
            last_action = action;

            if let Some(popup) = maybe_popup.as_mut() {
                let egui_ctx = frame_context.egui_ctx;
                Window::new("!").show(egui_ctx, |ui| {
                    popup.ui(ui);
                });
                if !popup.visible {
                    maybe_popup = None;
                }
            }

            debug_window(
                frame_context,
                &mut debuggable_stuff,
                &mut palette,
                &mut game_state,
            );
        });

    Ok(())
}

pub fn base_dir() -> PathBuf {
    let mut dir = env::current_exe()
        .expect("failed to get current_exe")
        .parent()
        .unwrap()
        .to_path_buf();

    // To avoid a bit of friction with cargo run versus cargo build --release + run the binary
    // we can find out if we're in "dev mode" by looking for the tell tale signs of development
    // aka: there's a toml file hanging out somewhere above wherever we are.
    let mut probe = dir.clone();
    while probe.pop() {
        if probe.join("Cargo.toml").exists() {
            return probe;
        }
    }

    dir
}
