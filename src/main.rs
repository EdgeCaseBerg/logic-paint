mod gamestate;
mod netbpm;

use std::env;
use std::fs::read_to_string;

use egor::{
    app::{App, FrameContext, WindowEvent, egui::Window, egui::Slider},
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
    let mut bg_size = 550;
    let mut bg_position = vec2(-163., -229.);
    let mut grid_step = 10;
    let mut num_boxes = 10;
    let mut box_offset = 10.0;

    App::new().window_size(1280, 720).title("Logic Brush").run(
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
            let left_mouse_pressed = input.mouse_pressed(MouseButton::Left);
            let left_mouse_held = input.mouse_held(MouseButton::Left);
            let left_mouse_released = input.mouse_released(MouseButton::Left);
            let right_mouse_pressed = input.mouse_pressed(MouseButton::Right);
            let right_mouse_held = input.mouse_held(MouseButton::Right);
            let right_mouse_released = input.mouse_released(MouseButton::Right);

            if right_mouse_pressed {
                bg_position = world_xy;
                println!("{:?}", bg_position);
            }

            gfx.rect().at(bg_position).size(Vec2::splat(bg_size as f32)).color(Color::BLUE);

            
            for r_step in (0..bg_size).step_by(bg_size / num_boxes) {
                for c_step in (0..bg_size).step_by(bg_size / num_boxes) {
                    let position = bg_position + Vec2::splat(box_offset/2.) + vec2(r_step as f32, c_step as f32);
                    gfx.rect().at(position).size(Vec2::splat((bg_size / num_boxes) as f32) - box_offset/2.).color(Color::WHITE);
                }
            }

            Window::new("Debug").show(egui_ctx, |ui| {
                ui.label(format!("FPS: {}", timer.fps));
                ui.label(format!("Mouse x: {} y: {}", mx, my));
                ui.label(format!("World x: {} y: {}", world_xy.x, world_xy.y));
                ui.label(format!("Screensize: {}", screen_size));

                ui.label(format!("Mouse state: left_mouse_pressed: {}", left_mouse_pressed));
                ui.label(format!("Mouse state: left_mouse_held: {}", left_mouse_held));
                ui.label(format!("Mouse state: left_mouse_released: {}", left_mouse_released));
                ui.label(format!("Mouse state: right_mouse_pressed: {}", right_mouse_pressed));
                ui.label(format!("Mouse state: right_mouse_held: {}", right_mouse_held));
                ui.label(format!("Mouse state: right_mouse_released: {}", right_mouse_released));

                ui.add(Slider::new(&mut bg_size, 1..=800).text("BG size"));
                ui.add(Slider::new(&mut grid_step, 1..=100).text("Grid step"));
                ui.add(Slider::new(&mut num_boxes, 10..=20).step_by(5.).text("Grid step"));
                ui.add(Slider::new(&mut box_offset, 2.0..=20.0).text("Box Offset"));
                
                
            });
        },
    );

    Ok(())
}

