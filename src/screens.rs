use crate::gamestate::PlayState;
use crate::levels::Level;
use crate::netppm::Ppm;
use crate::ui::{Action, ColorPalette, LoadedPpms, PlayArea, PlayerInput, draw_ppm_at};
use std::path::PathBuf;

use egor::math::Rect;
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
    ChooseLevelScreen {
        page: usize,
    },
}

#[derive(Debug, Clone)]
pub enum ScreenAction {
    ChangeScreen { to: Screens },
    NoAction,
    WipeLeft,
    WipeRight,
    WipeDone,
    PreviousPage,
    NextPage,
    MarkLevelComplete,
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
    let left_mouse_pressed =
        input.mouse_pressed(MouseButton::Left) || input.mouse_held(MouseButton::Left);
    let right_mouse_pressed =
        input.mouse_pressed(MouseButton::Right) || input.mouse_held(MouseButton::Right);

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
        ScreenAction::ChangeScreen {
            to: Screens::ChooseLevelScreen { page: 0 },
        }
    } else {
        ScreenAction::MarkLevelComplete
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
    levels: &[Level],
    page: usize,
    frame_context: &mut FrameContext,
    current_level: &mut PlayState,
    current_win_image: &mut Ppm,
    current_path: &mut PathBuf,
    loaded_ppms: &LoadedPpms,
) -> ScreenAction {
    let levels_per_page = 15;
    let levels_per_row = 5;
    let rows = levels_per_page / levels_per_row;
    let levels_to_show: Vec<&Level> = levels
        .iter()
        .skip(levels_per_page * page)
        .take(levels_per_page)
        .collect();

    let gfx = &mut (frame_context.gfx);
    let screen_size = gfx.screen_size();
    let center = screen_size / 2.;
    gfx.camera().target(center);

    let x_unit = 1280. / 32.;
    let y_unit = 720. / 18.;
    let level_bg_size = vec2(16. * x_unit, 10. * y_unit);
    let title_text_position = vec2(8. * x_unit, 5. * y_unit) + vec2(level_bg_size.x / 2., 0.);
    let level_bg_position = vec2(8. * x_unit, 6. * y_unit);
    let quit_position = vec2(28. * x_unit, 1. * y_unit);
    let quit_btn_size = vec2(3. * x_unit, 3. * y_unit);

    draw_centered_text(
        gfx,
        "Logic Paint",
        title_text_position,
        36.,
        Color::WHITE, // TODO: bring in the palette
    );

    gfx.rect()
        .at(level_bg_position)
        .size(level_bg_size)
        .color(Color::BLUE); // TODO: bring in the palette

    draw_ppm_at(&loaded_ppms.quit_ppm, quit_position, quit_btn_size, gfx);

    let input = &mut (frame_context.input);
    let (mx, my) = input.mouse_position();
    let world_xy = gfx.camera().screen_to_world(Vec2::new(mx, my), screen_size);
    let left_mouse_pressed = input.mouse_pressed(MouseButton::Left);

    // TODO: move to ui method
    let padding = vec2(10., 10.);
    let level_tile_height = (level_bg_size.y - padding.y * 2.) / rows as f32 - padding.y * 2.;
    let level_tile_size = vec2(level_tile_height, level_tile_height);
    let centering_x_offset =
        (level_bg_size.x - (level_tile_height + padding.x) * levels_per_row as f32) / 2.;
    let centering_y_offset = (level_bg_size.y - (level_tile_height + padding.y) * rows as f32) / 2.;

    let mut action = ScreenAction::NoAction;

    let anchor = level_bg_position + padding + vec2(centering_x_offset, centering_y_offset);
    for (r, levels_in_row) in levels_to_show.chunks(levels_per_row).enumerate() {
        for (c, level) in levels_in_row.into_iter().enumerate() {
            let pos = anchor + vec2(c as f32, r as f32) * (level_tile_size + padding);
            let rect = Rect::new(pos, level_tile_size);
            let highlight_color = if rect.contains(world_xy) {
                Color::BLACK
            } else {
                Color::WHITE
            };
            gfx.rect()
                .color(highlight_color)
                .at(pos - padding / 4.)
                .size(level_tile_size + padding / 4.);
            if level.completed {
                draw_ppm_at(&level.image, pos, level_tile_size, gfx);
            } else {
                draw_ppm_at(&loaded_ppms.unknown_ppm, pos, level_tile_size, gfx);
            }
            if rect.contains(world_xy) && left_mouse_pressed {
                action = ScreenAction::ChangeScreen {
                    to: Screens::GameScreen,
                };
                let to_load: PlayState = (&level.info).into();
                *current_level = to_load;
                *current_win_image = level.image.clone();
                *current_path = level.path.clone();
            }
        }
    }

    // draw a long tall < and > for the page buttons.
    let btn_width = 30.;
    let btn_size = vec2(btn_width, level_bg_size.y);
    let previous_btn_position = level_bg_position - vec2(btn_width, 0.) - vec2(padding.x, 0.);
    let next_btn_position = level_bg_position + vec2(level_bg_size.x, 0.) + vec2(padding.x, 0.);

    if page > 0 {
        let rect = Rect::new(previous_btn_position, btn_size);
        let (bg, fg) = if rect.contains(world_xy) {
            (Color::WHITE, Color::BLUE)
        } else {
            (Color::BLUE, Color::WHITE)
        };
        if rect.contains(world_xy) && left_mouse_pressed {
            action = ScreenAction::PreviousPage;
        }
        gfx.rect()
            .color(bg)
            .size(btn_size)
            .at(previous_btn_position);
        gfx.polygon()
            .at(previous_btn_position + vec2(btn_width - btn_width / 5., 10.))
            .points(&[
                vec2(0., 0.),
                vec2(-btn_width + btn_width / 4., btn_size.y / 2.),
                vec2(0., btn_size.y - 10.),
                vec2(0., 0.),
            ])
            .color(fg);
    }

    if levels.iter().skip(levels_per_page * (page + 1)).len() > 0 {
        let rect = Rect::new(next_btn_position, btn_size);
        let (bg, fg) = if rect.contains(world_xy) {
            (Color::WHITE, Color::BLUE)
        } else {
            (Color::BLUE, Color::WHITE)
        };
        if rect.contains(world_xy) && left_mouse_pressed {
            action = ScreenAction::NextPage;
        }
        gfx.rect().color(bg).size(btn_size).at(next_btn_position);
        gfx.polygon()
            .at(next_btn_position + vec2(btn_width / 5., 10.))
            .points(&[
                vec2(0., 0.),
                vec2(btn_width - btn_width / 4., btn_size.y / 2.),
                vec2(0., btn_size.y - 10.),
                vec2(0., 0.),
            ])
            .color(fg);
    }

    action
}
