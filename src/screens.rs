use crate::gamestate::PlayState;
use crate::netbpm::Pbm;
use crate::ui::{Action, ColorPalette, PlayArea, PlayerInput};

use std::fs::read_to_string;

use egor::{
    app::{FrameContext, egui::ComboBox, egui::Slider, egui::Window},
    input::MouseButton,
    math::{Vec2, vec2},
    render::Color,
};

pub fn play_game_screen(game_state: &mut PlayState, frame_context: &mut FrameContext) {
    let gfx = &mut (frame_context.gfx);
    let input = &mut (frame_context.input);
    let egui_ctx = frame_context.egui_ctx;
    let timer = frame_context.timer;
    let bg_position = vec2(-163., -209.);
    let mut bg_size = 525;
    let mut box_offset = 8.0;
    let mut draw_text_at = vec2(-160., -300.);
    let levels = ["./assets/P1.pbm", "./assets/P1-10x10.pbm"];
    let mut selected_level = 0;
    let mut palette = ColorPalette::meeks();

    let screen_size = gfx.screen_size();
    let (mx, my) = input.mouse_position();
    let world_xy = gfx.camera().screen_to_world(Vec2::new(mx, my), screen_size);
    let left_mouse_pressed = input.mouse_pressed(MouseButton::Left);
    let right_mouse_pressed = input.mouse_pressed(MouseButton::Right);

    let play_area = PlayArea {
        top_left: bg_position,
        size: Vec2::splat(bg_size as f32),
        grid_gutter: box_offset,
        palette,
    };

    let player_input = PlayerInput {
        position: world_xy,
        action: {
            match (left_mouse_pressed, right_mouse_pressed) {
                (false, false) => None,
                (true, false) => Some(Action::FillCell),
                (_, true) => Some(Action::MarkCell),
            }
        },
    };

    play_area.draw_backgrounds(&game_state, &player_input, gfx);
    play_area.draw_grid(game_state, &player_input, gfx);
    play_area.draw_row_groups(&game_state, gfx);
    play_area.draw_column_groups(&game_state, gfx);

    game_state.update_groups();

    let left_mouse_held = input.mouse_held(MouseButton::Left);
    let left_mouse_released = input.mouse_released(MouseButton::Left);
    let right_mouse_held = input.mouse_held(MouseButton::Right);
    let right_mouse_released = input.mouse_released(MouseButton::Right);

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
        ui.add(Slider::new(&mut box_offset, 2.0..=20.0).text("Box Offset"));
        let before_level = selected_level;
        ComboBox::from_label("Load level")
            .show_index(ui, &mut selected_level, levels.len(), |i| levels[i]);
        if before_level != selected_level {
            let pbm = read_to_string(levels[selected_level]).expect("Could not load level");
            let pbm: Pbm = pbm.parse().expect("level not in expected format");
            *game_state = (&pbm).into();
        }
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
    });
}


pub fn win_screen(game_state: &mut PlayState, frame_context: &mut FrameContext) {
    let gfx = &mut (frame_context.gfx);
    let input = &mut (frame_context.input);
    let egui_ctx = frame_context.egui_ctx;
    let timer = frame_context.timer;
    let mut palette = ColorPalette::meeks();
    let mut size_x = 200;
    let mut size_y = 200;

    let screen_size = gfx.screen_size();
    let (mx, my) = input.mouse_position();
    let world_xy = gfx.camera().screen_to_world(Vec2::new(mx, my), screen_size);
    let left_mouse_pressed = input.mouse_pressed(MouseButton::Left);
    let right_mouse_pressed = input.mouse_pressed(MouseButton::Right);

    gfx.rect()
        .at(world_xy)
        .color(Color::new(palette.background))
        .size(vec2(size_x as f32, size_y as f32));


    Window::new("Debug").show(egui_ctx, |ui| {
        ui.label(format!("FPS: {}", timer.fps));
        ui.label(format!("Mouse x: {} y: {}", mx, my));
        ui.label(format!("World x: {} y: {}", world_xy.x, world_xy.y));
        ui.label(format!("Screensize: {}", screen_size));
        ui.add(Slider::new(&mut size_x, 1..=800).text("BG width"));
        ui.add(Slider::new(&mut size_y, 1..=800).text("BG height"));
    });

}