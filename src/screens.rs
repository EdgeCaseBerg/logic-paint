use crate::DebugStuff;
use crate::gamestate::PlayState;
use crate::ui::{Action, ColorPalette, PlayArea, PlayerInput};

use egor::{
    app::FrameContext,
    input::MouseButton,
    math::{Vec2, vec2},
    render::Color,
};

pub fn play_game_screen(
    game_state: &mut PlayState,
    frame_context: &mut FrameContext,
    palette: &mut ColorPalette,
) {
    let gfx = &mut (frame_context.gfx);
    let input = &mut (frame_context.input);
    let bg_position = vec2(-163., -209.);
    let bg_size = 525;
    let box_offset = 8.0;

    let screen_size = gfx.screen_size();
    let (mx, my) = input.mouse_position();
    let world_xy = gfx.camera().screen_to_world(Vec2::new(mx, my), screen_size);
    let left_mouse_pressed = input.mouse_pressed(MouseButton::Left);
    let right_mouse_pressed = input.mouse_pressed(MouseButton::Right);

    let play_area = PlayArea {
        top_left: bg_position,
        size: Vec2::splat(bg_size as f32),
        grid_gutter: box_offset,
        palette: *palette,
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
}

pub fn win_screen(
    game_state: &mut PlayState,
    frame_context: &mut FrameContext,
    palette: &ColorPalette,
    debuggable_stuff: &DebugStuff,
) {
    let gfx = &mut (frame_context.gfx);
    let input = &mut (frame_context.input);

    let screen_size = gfx.screen_size();
    gfx.camera().target(screen_size / 2.);

    let (mx, my) = input.mouse_position();
    // let world_xy = gfx.camera().screen_to_world(Vec2::new(mx, my), screen_size);
    // let left_mouse_pressed = input.mouse_pressed(MouseButton::Left);
    // let right_mouse_pressed = input.mouse_pressed(MouseButton::Right);


    gfx.rect()
        .at(vec2(
            75.,
            75.,
        ))
        .color(Color::new(palette.background))
        .size(vec2(
            debuggable_stuff.size_x as f32, //480 is pretty good
            debuggable_stuff.size_y as f32,
        ));

    let num_incorrect = game_state.number_incorrect();
    if num_incorrect == 0 {
        gfx.text(&format!("Perfect!"))
            .size(16.)
            .color(Color::new(palette.group_highlight))
            .at(screen_size / 2.);
    } else {
        gfx.text(&format!("Incorrect {}", num_incorrect))
            .size(16.)
            .color(Color::new(palette.group_highlight))
            .at(screen_size / 2.);
    }

    gfx.rect()
        .at(screen_size /2. + vec2(-50., 100.))
        .color(Color::new(palette.background)) // use cellhighlight for hover color
        .size(vec2(200., 100.));
    gfx.text(&format!("Return to Menu"))
            .size(16.)
            .color(Color::new(palette.group_highlight))  // use bg color for hover color
            .at(screen_size / 2. + vec2(-25., 150.));
    // TODO: interact wit the button
}
