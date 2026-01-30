mod gamestate;
mod netbpm;

use std::env;
use std::fs::read_to_string;

use egor::{
    app::{App, FrameContext, WindowEvent, egui::Slider, egui::Window},
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

    let mut game_state: gamestate::PlayState = (&test_pbm).into();
    println!("{:?}", game_state);

    let mut game_over = false;
    let mut bg_size = 550;
    let mut bg_position = vec2(-163., -229.);
    // let mut box_size = 50;
    let mut num_boxes = 10;
    let mut box_offset = 8.0;
    let mut draw_text_at = vec2(-160., -300.);

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

            let screen_size = gfx.screen_size();

            if game_over {
                gfx.text("GAME OVER")
                    .size(50.)
                    .color(Color::RED)
                    .at(vec2(screen_size.x / 2. - 40., screen_size.y / 2.));
                return;
            }

            if input.key_pressed(KeyCode::Escape) {
                game_over = true;
            }

            let (mx, my) = input.mouse_position();
            let world_xy = gfx.camera().screen_to_world(Vec2::new(mx, my), screen_size);
            let left_mouse_pressed = input.mouse_pressed(MouseButton::Left);
            let left_mouse_held = input.mouse_held(MouseButton::Left);
            let left_mouse_released = input.mouse_released(MouseButton::Left);
            let right_mouse_pressed = input.mouse_pressed(MouseButton::Right);
            let right_mouse_held = input.mouse_held(MouseButton::Right);
            let right_mouse_released = input.mouse_released(MouseButton::Right);

            if right_mouse_pressed {
                draw_text_at = world_xy;
                eprintln!("{:?}", draw_text_at);
            }

            gfx.rect()
                .at(bg_position)
                .size(Vec2::splat(bg_size as f32))
                .color(Color::BLUE);

            let halfset = box_offset / 2.;
            let anchor = bg_position + Vec2::splat(halfset as f32);
            let offset = Vec2::splat(halfset);
            let num_boxes = game_state.rows().len();
            let box_size =
                (bg_size as f32 - (halfset + halfset * num_boxes as f32)) / num_boxes as f32;

            for (r, row) in game_state.rows().into_iter().enumerate() {
                for (c, state) in row.iter().enumerate() {
                    let box_size = box_size as f32;
                    let position =
                        anchor + vec2(c as f32, r as f32) * (Vec2::splat(box_size) + offset);
                    let size = Vec2::splat(box_size);
                    let color = match state {
                        gamestate::CellState::Empty => Color::WHITE,
                        gamestate::CellState::Filled => Color::GREEN,
                        gamestate::CellState::Incorrect => Color::RED,
                        gamestate::CellState::RuledOut => Color::new([0.5, 0.5, 0.5, 1.0]),
                        gamestate::CellState::UserRuledOut => Color::new([0.5, 0.5, 0.5, 1.0]),
                        _ => Color::BLACK,
                    };
                    gfx.rect().at(position).size(size).color(color);
                    if Rect::new(position, size).contains(world_xy) && left_mouse_pressed {
                        println!("attempt file at r:{} c:{}", r, c);
                        game_state.attempt_fill(r, c);
                    }
                }
            }

            let padding = offset.y / 2. - box_size / 2.;
            let scaler = vec2(0.5, 1.);
            let anchor = anchor - padding;
            for (r, groups) in game_state.row_groups.iter().enumerate() {
                let number_of_groups = groups.iter().len();
                // let anchor = anchor - vec2((box_size + offset.x) * number_of_groups as f32, 0.);
                for i in 0..number_of_groups {
                    let position =
                        anchor + vec2(-(i as f32) - 2., r as f32) * (Vec2::splat(box_size) + offset) * scaler;
                    let tp = gfx.camera().world_to_screen(position, screen_size);
                    let g = number_of_groups - i - 1; // right aligned.
                    gfx.text(&format!("{}", groups[g].num_cells))
                        .size(0.5 * box_size as f32)
                        .color(match groups[g].filled {
                            true => Color::GREEN,
                            false => Color::WHITE,
                        })
                        .at(tp);
                }
            }

            if right_mouse_pressed {
                eprintln!(
                    "{} -> {}",
                    draw_text_at,
                    gfx.camera().world_to_screen(draw_text_at, screen_size)
                );
            }
            //Vec2(416.1604, 291.32635)
            gfx.text("?")
                .size(box_size as f32)
                .color(Color::RED)
                .at(draw_text_at);

            let foo = gfx.camera().world_to_screen(draw_text_at, screen_size);
            gfx.text("?")
                .size(box_size as f32)
                .color(Color::WHITE)
                .at(foo);

            game_state.update_groups();

            Window::new("Debug").show(egui_ctx, |ui| {
                ui.label(format!("FPS: {}", timer.fps));
                ui.label(format!("Mouse x: {} y: {}", mx, my));
                ui.label(format!("World x: {} y: {}", world_xy.x, world_xy.y));
                ui.label(format!("Screensize: {}", screen_size));

                ui.label(format!(
                    "Mouse state: left_mouse_pressed: {}",
                    left_mouse_pressed
                ));
                ui.label(format!("Mouse state: left_mouse_held: {}", left_mouse_held));
                ui.label(format!(
                    "Mouse state: left_mouse_released: {}",
                    left_mouse_released
                ));
                ui.label(format!(
                    "Mouse state: right_mouse_pressed: {}",
                    right_mouse_pressed
                ));
                ui.label(format!(
                    "Mouse state: right_mouse_held: {}",
                    right_mouse_held
                ));
                ui.label(format!(
                    "Mouse state: right_mouse_released: {}",
                    right_mouse_released
                ));
                ui.label(format!("Grid complete? {}", game_state.is_complete()));

                ui.add(Slider::new(&mut bg_size, 1..=800).text("BG size"));
                // ui.add(Slider::new(&mut box_size, 1..=100).text("Box size"));
                // ui.add(
                //     Slider::new(&mut num_boxes, 10..=20)
                //         .step_by(5.)
                //         .text("# boxes"),
                // );
                ui.add(Slider::new(&mut box_offset, 2.0..=20.0).text("Box Offset"));
            });
        },
    );

    Ok(())
}
