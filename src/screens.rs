use crate::DebugStuff;
use crate::gamestate::PlayState;
use crate::levels::Level;
use crate::netppm::Ppm;
use crate::ui::{Action, ColorPalette, PlayArea, PlayerInput, draw_ppm_at};

use egor::{
    app::FrameContext,
    app::egui::lerp,
    input::MouseButton,
    math::{Vec2, vec2},
    render::Color,
};

#[derive(Debug, Clone)]
pub enum Screens {
    GameScreen,
    WinScreen,
    WipeScreen {
        from: Box<Screens>,
        to: Box<Screens>,
        duration: f32,
    },
    ChooseLevelScreen,
}

#[derive(Debug, Clone)]
pub enum ScreenAction {
    ChangeScreen { to: Screens },
    NoAction,
    WipeLeft,
    WipeRight,
    WipeDone,
}

pub fn play_game_screen(
    game_state: &mut PlayState,
    frame_context: &mut FrameContext,
    palette: &mut ColorPalette,
) -> ScreenAction {
    let gfx = &mut (frame_context.gfx);
    let input = &mut (frame_context.input);
    let bg_position = vec2(1280. / 2. - 163., 720. / 2. - 209.);
    let bg_size = 525;
    let box_offset = 8.0;

    let screen_size = gfx.screen_size();
    gfx.camera().target(screen_size / 2.);
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

    if game_state.is_complete() {
        ScreenAction::ChangeScreen {
            to: Screens::WinScreen,
        }
    } else {
        ScreenAction::NoAction
    }
}

pub fn win_screen(
    game_state: &mut PlayState,
    ppm: &Ppm,
    frame_context: &mut FrameContext,
    palette: &ColorPalette,
    _debuggable_stuff: &DebugStuff,
) -> ScreenAction {
    let gfx = &mut (frame_context.gfx);
    let input = &mut (frame_context.input);

    let screen_size = gfx.screen_size();
    gfx.camera().target(screen_size / 2.);

    let (mx, my) = input.mouse_position();
    let world_xy = gfx.camera().screen_to_world(Vec2::new(mx, my), screen_size);
    let left_mouse_pressed = input.mouse_pressed(MouseButton::Left);
    // TODO: add in held so that one can drag easily.

    draw_ppm_at(ppm, vec2(75., 75.), vec2(450., 450.), gfx);

    let num_incorrect = game_state.number_incorrect();
    if num_incorrect == 0 {
        gfx.text(&format!("Perfect!"))
            .size(78.)
            .color(Color::new(palette.group_highlight))
            .at(screen_size / 2.);
    } else {
        gfx.text(&format!("Incorrect {}", num_incorrect))
            .size(16.)
            .color(Color::new(palette.group_highlight))
            .at(screen_size / 2.);
    }

    let button_bg_pos = screen_size / 2. + vec2(-50., 100.);
    let button_size = vec2(200., 100.);
    let rect = egor::math::Rect::new(button_bg_pos, button_size);
    let should_highlight = rect.contains(world_xy);
    let (btn_color, font_color) = if should_highlight {
        (
            Color::new(palette.group_highlight),
            Color::new(palette.background),
        )
    } else {
        (
            Color::new(palette.background),
            Color::new(palette.group_highlight),
        )
    };
    gfx.rect()
        .at(button_bg_pos)
        .color(btn_color)
        .size(button_size);
    draw_centered_text(
        gfx,
        &format!("Return to Menu"),
        button_bg_pos + button_size * 0.5,
        16.,
        font_color,
    );
    if left_mouse_pressed && should_highlight {
        // Tmp go back to game screen for now
        ScreenAction::ChangeScreen {
            to: Screens::GameScreen,
        }
    } else {
        ScreenAction::NoAction
    }
}

// TODO move to ui module
fn draw_centered_text(
    gfx: &mut egor::render::Graphics,
    text: &str,
    center: Vec2,
    size: f32,
    color: Color,
) {
    // average font width is 0.53
    let w = text.len() as f32 * size * 0.53;
    let h = size;

    let pos = center - vec2(w * 0.5, h * 0.5 - size / 2.);

    gfx.text(text).size(size).color(color).at(pos);
}

pub fn wipe_screen(
    wipe_progress: &mut f32,
    duration: f32,
    frame_context: &mut FrameContext,
    palette: &ColorPalette,
) -> ScreenAction {
    *wipe_progress += frame_context.timer.delta;

    let gfx = &mut (frame_context.gfx);
    let screen_size = gfx.screen_size();
    let pos = screen_size / 2.;

    gfx.camera().target(screen_size / 2.);

    draw_centered_text(
        gfx,
        &format!("{}%", *wipe_progress / duration),
        pos,
        20.,
        Color::new(palette.group_highlight),
    );

    let box_size = 60.0;
    let num_boxes = screen_size / box_size;
    let box_progress = *wipe_progress / duration;
    let sin = lerp(0.0..=std::f32::consts::PI, box_progress);
    let max_boxes_to_draw_x = 1 + lerp(0.0..=num_boxes.x, sin.sin()) as usize;
    let max_boxes_to_draw_y = 1 + lerp(0.0..=num_boxes.y, sin.sin()) as usize;
    let (even, odd) = palette.even_odd_color(0);
    let in_first_half_of_animation = *wipe_progress < duration * 0.5;

    for x in 0..max_boxes_to_draw_x {
        for y in 0..max_boxes_to_draw_y {
            let offset = if in_first_half_of_animation {
                vec2(x as f32, y as f32)
            } else {
                vec2(num_boxes.x - x as f32, num_boxes.y - y as f32)
            };
            let pos = offset * box_size;
            let color = if x % 2 == 0 { even } else { odd };
            gfx.rect()
                .color(Color::new(color))
                .size(vec2(box_size, box_size))
                .at(pos);
        }
    }

    if in_first_half_of_animation {
        ScreenAction::WipeLeft
    } else if *wipe_progress > duration {
        ScreenAction::WipeDone
    } else {
        ScreenAction::WipeRight
    }
}

pub fn level_select_screen(
    _levels: &[Level],
    _frame_context: &mut FrameContext,
    _current_level: &mut PlayState,
) -> ScreenAction {
    ScreenAction::NoAction
}
