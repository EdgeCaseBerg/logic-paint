mod gamestate;
mod netbpm;

use std::env;
use std::fs::read_to_string;

use egor::{
    app::{App, FrameContext, WindowEvent, egui::Window},
    input::{KeyCode, MouseButton},
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

    let mut game_over = false;

    App::new().title("Egor Stateful Rectangle").run(
        move |FrameContext {
                  gfx,
                  input,
                  timer,
                  egui_ctx,
                  events,
              }| {
            for event in events {
                match event {
                    WindowEvent::CloseRequested => {
                        println!("Shutting down");
                        game_over = true;
                    }
                    _ => {}
                }
            }

            if game_over {
                return;
            }

            if input.key_pressed(KeyCode::Escape) {
                game_over = true;
            }

            let screen_size = gfx.screen_size();
            let (mx, my) = input.mouse_position();
            let world_xy = gfx.camera().screen_to_world(Vec2::new(mx, my), screen_size);

            gfx.rect().at(world_xy).color(Color::RED);

            Window::new("Debug").show(egui_ctx, |ui| {
                ui.label(format!("FPS: {}", timer.fps));
                ui.label(format!("Mouse x: {} y: {}", mx, my));
                ui.label(format!("World x: {} y: {}", world_xy.x, world_xy.y));
                ui.label(format!("Screensize: {}", screen_size));
            });
        },
    );

    Ok(())
}
