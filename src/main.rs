mod gamestate;
mod netbpm;

use std::env;
use std::fs::read_to_string;

use egor::{
    app::{App, FrameContext },
    input::{KeyCode },
    math::{Rect, Vec2, vec2},
    render::Color,
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
    println!("{:?}", test_pbm);

    let state: gamestate::PlayState = (&test_pbm).into();
    println!("{:?}", state);

    let mut position = Vec2::ZERO;

    App::new()
        .title("Egor Stateful Rectangle")
        .run(move |FrameContext { gfx, input, timer, .. } | {
            let dx = input.key_held(KeyCode::ArrowRight) as i8
                - input.key_held(KeyCode::ArrowLeft) as i8;
            let dy =
                input.key_held(KeyCode::ArrowDown) as i8 - input.key_held(KeyCode::ArrowUp) as i8;

            position += vec2(dx as f32, dy as f32) * 100.0 * timer.delta;

            gfx.rect().at(position).color(Color::RED);
        });

    Ok(())
}
