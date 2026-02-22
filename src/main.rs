use logicpaint::base_dir;
use logicpaint::gamestate;
use logicpaint::levels;
use logicpaint::netbpm;
use logicpaint::netppm;
use logicpaint::pop_up::PopUp;
use logicpaint::screens;
use logicpaint::ui;
use logicpaint::ui::DebugStuff;
use logicpaint::ui::LoadedPpms;
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

use std::fs::read_to_string;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let exe_dir = base_dir();
    let assets = exe_dir.join("assets");
    let level_dir_path = exe_dir.join("levels");

    let loaded_ppms = LoadedPpms::load(assets)?;

    let mut current_screen = Screens::ChooseLevelScreen { page: 0 };
    let mut levels = levels::load_levels_from_dir(&level_dir_path)?;
    let mut palette = ColorPalette::meeks();

    // TODO: Refactor this to be one struct passed around
    let mut win_image = levels[0].image.clone();
    let mut current_level = levels[0].path.clone();
    let mut game_state: gamestate::PlayState = (&levels[0].info).into();

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
                    &loaded_ppms,
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
