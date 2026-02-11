mod gamestate;
mod netbpm;
mod screens;
mod ui;

use std::env;
use std::fs::read_to_string;

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

            screens::play_game_screen(&mut game_state, frame_context);
        });

    Ok(())
}
