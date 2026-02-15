mod gamestate;
mod netbpm;
mod screens;
mod ui;

use crate::gamestate::PlayState;
use crate::netbpm::Pbm;
use egor::{
    app::{FrameContext, egui::ComboBox, egui::Slider, egui::Window},
    math::Vec2,
};
use screens::{ScreenAction, Screens};
use std::env;
use std::fs::read_to_string;
use ui::ColorPalette;

use egor::{
    app::{App, WindowEvent},
    input::KeyCode,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments: Vec<String> = env::args().collect();
    if arguments.is_empty() {
        eprintln!("{:?}", "pass the input data as the first argument.");
        return Err("Could not load file".into());
    }

    let mut arguments = arguments.iter();
    arguments.next(); // skip the name of the program being ran
    let filename = match arguments.next() {
        Some(arg) => arg,
        _ => "./assets/P1.pbm",
    };

    let test_pbm = read_to_string(filename)?;
    let test_pbm: netbpm::Pbm = test_pbm.parse()?;
    let mut game_state: gamestate::PlayState = (&test_pbm).into();
    let mut debuggable_stuff = DebugStuff::new();
    let mut palette = ColorPalette::meeks();
    let mut current_screen = Screens::GameScreen;
    let mut wipe_progress = 0.0;

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

            let action = match &current_screen {
                Screens::GameScreen => {
                    screens::play_game_screen(&mut game_state, frame_context, &mut palette)
                }
                Screens::WinScreen => screens::win_screen(
                    &mut game_state,
                    frame_context,
                    &palette,
                    &mut debuggable_stuff,
                ),
                Screens::WipeScreen {
                    from: _,
                    to: _,
                    duration,
                } => screens::wipe_screen(&mut wipe_progress, *duration, frame_context, &palette),
            };
            match action {
                ScreenAction::NoAction => {}
                ScreenAction::ChangeScreen { to } => {
                    wipe_progress = 0.0;
                    current_screen = Screens::WipeScreen {
                        from: Box::new(current_screen.clone()),
                        to: Box::new(to),
                        duration: debuggable_stuff.transition_duration,
                    };
                }
                ScreenAction::WipeLeft => {
                    // TODO
                }
                ScreenAction::WipeRight => {
                    // TODO
                }
                ScreenAction::WipeDone => {
                    let Screens::WipeScreen {
                        from: _,
                        ref to,
                        duration: _,
                    } = current_screen
                    else {
                        panic!("screen was not wipe!{:?}", current_screen)
                    };
                    current_screen = *to.clone();
                }
            };

            debug_window(
                frame_context,
                &mut debuggable_stuff,
                &mut palette,
                &mut game_state,
            );
        });

    Ok(())
}

struct DebugStuff {
    size_x: usize,
    size_y: usize,
    selected_level: usize,
    transition_duration: f32,
}

impl DebugStuff {
    fn new() -> DebugStuff {
        DebugStuff {
            size_x: 200,
            size_y: 200,
            selected_level: 0,
            transition_duration: 2.0,
        }
    }
}

fn debug_window(
    frame_context: &mut FrameContext,
    debuggable_stuff: &mut DebugStuff,
    palette: &mut ColorPalette,
    game_state: &mut PlayState,
) {
    let gfx = &mut (frame_context.gfx);
    let input = &mut (frame_context.input);
    let egui_ctx = frame_context.egui_ctx;
    let timer = frame_context.timer;

    let screen_size = gfx.screen_size();
    let (mx, my) = input.mouse_position();
    let world_xy = gfx.camera().screen_to_world(Vec2::new(mx, my), screen_size);

    let levels = ["./assets/P1.pbm", "./assets/P1-10x10.pbm"];

    Window::new("Debug").show(egui_ctx, |ui| {
        ui.label(format!("FPS: {}", timer.fps));
        ui.label(format!("Mouse x: {} y: {}", mx, my));
        ui.label(format!("World x: {} y: {}", world_xy.x, world_xy.y));
        ui.label(format!("Screensize: {}", screen_size));
        ui.label(format!("Grid complete? {}", game_state.is_complete()));
        ui.add(Slider::new(&mut debuggable_stuff.size_x, 1..=800).text("BG width"));
        ui.add(Slider::new(&mut debuggable_stuff.size_y, 1..=800).text("BG height"));

        ui.separator();
        ui.label("grid even: ");
        ui.color_edit_button_rgba_unmultiplied(&mut palette.grid_even);
        ui.label("grid odd: ");
        ui.color_edit_button_rgba_unmultiplied(&mut palette.grid_odd);
        ui.label("background: ");
        ui.color_edit_button_rgba_unmultiplied(&mut palette.background);
        ui.label("cell_filled_in: ");
        ui.color_edit_button_rgba_unmultiplied(&mut palette.cell_filled_in);
        ui.label("cell_marked_user: ");
        ui.color_edit_button_rgba_unmultiplied(&mut palette.cell_marked_user);
        ui.label("cell_marked_game: ");
        ui.color_edit_button_rgba_unmultiplied(&mut palette.cell_marked_game);
        ui.label("cell_highlight: ");
        ui.color_edit_button_rgba_unmultiplied(&mut palette.cell_highlight);
        ui.label("cell_incorrect: ");
        ui.color_edit_button_rgba_unmultiplied(&mut palette.cell_incorrect);
        ui.label("palette.group_highlight: ");
        ui.color_edit_button_rgba_unmultiplied(&mut palette.group_highlight);

        ui.separator();
        let before_level = debuggable_stuff.selected_level;
        ComboBox::from_label("Load level").show_index(
            ui,
            &mut debuggable_stuff.selected_level,
            levels.len(),
            |i| levels[i],
        );
        if before_level != debuggable_stuff.selected_level {
            let pbm = read_to_string(levels[debuggable_stuff.selected_level])
                .expect("Could not load level");
            let pbm: Pbm = pbm.parse().expect("level not in expected format");
            *game_state = (&pbm).into();
        }
        ui.separator();
        ui.add(Slider::new(&mut debuggable_stuff.transition_duration, 0.5..=5.0).text("Wipe duration"));
    });
}
