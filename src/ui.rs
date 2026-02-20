use crate::gamestate::CellState;
use crate::gamestate::PlayState;
use crate::netbpm::Pbm;
use crate::netppm::Ppm;
use std::fs::read_to_string;

use egor::{
    app::{FrameContext, egui::ComboBox, egui::Slider, egui::Window},
    math::{Rect, Vec2, vec2},
    render::{Color, Graphics},
};

#[derive(Debug, Copy, Clone)]
pub struct ColorPalette {
    pub background: [f32; 4],
    pub grid_even: [f32; 4],
    pub grid_odd: [f32; 4],
    pub cell_filled_in: [f32; 4],
    pub cell_marked_user: [f32; 4],
    pub cell_marked_game: [f32; 4],
    pub cell_highlight: [f32; 4],
    pub cell_incorrect: [f32; 4],
    pub group_highlight: [f32; 4],
}

fn rgba(r: u8, g: u8, b: u8, a: f32) -> [f32; 4] {
    [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a]
}

impl ColorPalette {
    pub fn meeks() -> Self {
        Self {
            background: rgba(61, 123, 123, 1.0),
            grid_even: rgba(255, 227, 212, 1.0),
            grid_odd: rgba(255, 255, 255, 1.0),
            cell_filled_in: rgba(46, 220, 255, 1.0),
            cell_marked_user: rgba(129, 231, 223, 0.8),
            cell_marked_game: rgba(129, 231, 223, 1.0),
            cell_highlight: rgba(252, 255, 172, 1.0),
            cell_incorrect: rgba(255, 0, 0, 1.0),
            group_highlight: rgba(251, 212, 207, 1.0),
        }
    }

    pub fn even_odd_color(&self, i: usize) -> ([f32; 4], [f32; 4]) {
        if i % 2 == 0 {
            (self.grid_even, self.grid_odd)
        } else {
            (self.grid_odd, self.grid_even)
        }
    }
}

#[derive(Debug)]
pub enum Action {
    FillCell,
    MarkCell,
}

#[derive(Debug)]
pub struct PlayerInput {
    // position in world units
    pub position: Vec2,
    pub action: Option<Action>,
}

impl PlayerInput {
    fn can_fill_at(&self, cell: &Rect) -> bool {
        match self.action {
            Some(Action::FillCell) => cell.contains(self.position),
            _ => false,
        }
    }

    fn can_mark_at(&self, cell: &Rect) -> bool {
        match self.action {
            Some(Action::MarkCell) => cell.contains(self.position),
            _ => false,
        }
    }

    fn overlaps(&self, cell: &Rect) -> bool {
        cell.contains(self.position)
    }
}

pub struct PlayArea {
    pub top_left: Vec2,
    pub size: Vec2,
    pub grid_gutter: f32,
    pub palette: ColorPalette,
}

fn draw_x_at(position: Vec2, cell_size: Vec2, color: Color, gfx: &mut Graphics) {
    let thickness = cell_size * 0.2;
    gfx.polygon()
        .at(position + vec2(0., thickness.y))
        .points(&[
            vec2(thickness.x, 0.),
            vec2(thickness.x, cell_size.y),
            vec2(0., cell_size.y),
            vec2(0., 0.),
        ])
        .rotate(-3.145 / 4.)
        .color(color);
    gfx.polygon()
        .at(position + vec2(cell_size.x - thickness.x, thickness.y))
        .points(&[
            vec2(thickness.x, 0.),
            vec2(thickness.x, cell_size.y),
            vec2(0., cell_size.y),
            vec2(0., 0.),
        ])
        .rotate(-7. * 3.145 / 4.)
        .color(color);
}

impl PlayArea {
    fn play_area_gutter(&self) -> Vec2 {
        self.size * 0.4
    }

    fn halfset(&self) -> f32 {
        self.grid_gutter / 2.
    }

    fn anchor(&self) -> Vec2 {
        self.top_left + Vec2::splat(self.halfset())
    }

    fn box_size(&self, number_of_boxes: usize) -> f32 {
        let halfset = self.halfset();
        (self.size.x - (halfset + halfset * number_of_boxes as f32)) / number_of_boxes as f32
    }

    pub fn draw_backgrounds(
        &self,
        play_state: &PlayState,
        input: &PlayerInput,
        gfx: &mut Graphics,
    ) {
        let num_boxes = play_state.rows().len();
        let halfset = self.halfset();
        let anchor = self.anchor();
        let box_size = self.box_size(num_boxes);
        let cell_and_gutter_size = halfset + box_size;
        let cell_and_gutter_size_v = Vec2::splat(cell_and_gutter_size);
        let side_areas_size = self.play_area_gutter();
        let row_group_bg_size = vec2(side_areas_size.x, box_size);
        let column_group_size = vec2(box_size, side_areas_size.y);

        for (r, row) in play_state.rows().into_iter().enumerate() {
            let y_offset = r as f32 * cell_and_gutter_size;
            let row_group_bg_position = anchor - vec2(side_areas_size.x, -y_offset);
            let (even_odd_bg_color, odd_even_bg_color) = self.palette.even_odd_color(r);
            let mouse_within_row_range = true
                && row_group_bg_position.y <= input.position.y
                && input.position.y <= row_group_bg_position.y + box_size;

            let row_group_bg = if mouse_within_row_range {
                self.palette.group_highlight
            } else {
                odd_even_bg_color
            };

            // backgrounds behind the row numbers
            gfx.rect()
                .at(row_group_bg_position)
                .color(Color::new(row_group_bg))
                .size(row_group_bg_size);

            if r != 0 {
                continue;
            }

            // backgrounds behind the column numbers
            let colors = [even_odd_bg_color, odd_even_bg_color];
            for (c, _) in row.iter().enumerate() {
                let position = anchor + vec2(c as f32, r as f32) * cell_and_gutter_size_v;
                let column_group_position = position + vec2(0., -side_areas_size.y);
                let mouse_within_column_range = true
                    && column_group_position.x <= input.position.x
                    && input.position.x <= column_group_position.x + column_group_size.x;
                let column_group_bg_color = if mouse_within_column_range {
                    self.palette.group_highlight
                } else {
                    colors[c % 2]
                };
                gfx.rect()
                    .at(column_group_position)
                    .color(Color::new(column_group_bg_color))
                    .size(column_group_size);
            }
        }

        //The play area behind the grid
        gfx.rect()
            .at(self.top_left)
            .size(self.size)
            .color(Color::new(self.palette.background));
    }

    pub fn draw_grid(&self, play_state: &mut PlayState, input: &PlayerInput, gfx: &mut Graphics) {
        let halfset = self.halfset();
        let anchor = self.anchor();
        let offset = Vec2::splat(halfset);
        let num_boxes = play_state.rows().len();
        let box_size = self.box_size(num_boxes);
        let cell_size = Vec2::splat(box_size);
        let side_areas_size = self.play_area_gutter();

        for (r, row) in play_state.rows().into_iter().enumerate() {
            let (even_odd_bg_color, odd_even_bg_color) = self.palette.even_odd_color(r);

            let y_offset = r as f32 * (halfset + box_size);
            let row_group_bg_position = anchor - vec2(side_areas_size.x, -y_offset);
            let row_group_bg = if row_group_bg_position.y <= input.position.y
                && input.position.y <= row_group_bg_position.y + box_size
            {
                self.palette.group_highlight
            } else {
                odd_even_bg_color
            };
            gfx.rect()
                .at(row_group_bg_position)
                .color(Color::new(row_group_bg))
                .size(vec2(side_areas_size.x, box_size));

            for (c, state) in row.iter().enumerate() {
                let position = anchor + vec2(c as f32, r as f32) * (Vec2::splat(box_size) + offset);
                let color = match state {
                    CellState::Empty => Color::new(even_odd_bg_color),
                    CellState::Filled => Color::new(self.palette.cell_filled_in),
                    CellState::Incorrect => Color::new(self.palette.cell_incorrect),
                    CellState::RuledOut => Color::new(self.palette.cell_marked_game),
                    CellState::UserRuledOut => Color::new(self.palette.cell_marked_user),
                };
                let cell_rect = Rect::new(position, cell_size);
                if input.overlaps(&cell_rect) {
                    gfx.rect()
                        .at(position - offset)
                        .size(cell_size + offset * 2.)
                        .color(Color::new(self.palette.cell_highlight));
                }

                match state {
                    CellState::Empty | CellState::Filled => {
                        gfx.rect().at(position).size(cell_size).color(color);
                    }
                    _ => {
                        gfx.rect()
                            .at(position)
                            .size(cell_size)
                            .color(Color::new(even_odd_bg_color));
                        draw_x_at(position, cell_size, color, gfx);
                    }
                };

                if input.can_fill_at(&cell_rect) {
                    play_state.attempt_fill(r, c);
                }
                if input.can_mark_at(&cell_rect) {
                    play_state.mark_cell(r, c);
                }
            }
        }
    }

    pub fn draw_row_groups(&self, play_state: &PlayState, gfx: &mut Graphics) {
        let offset = self.halfset();
        let num_boxes = play_state.rows().len();
        let box_size = self.box_size(num_boxes);
        let padding = self.grid_gutter / 2. - box_size / 2.;
        let offset = Vec2::splat(offset);
        let grid_cell_size = Vec2::splat(box_size) + offset;
        let scaler = vec2(0.5, 1.);
        let anchor = self.anchor();
        let anchor = anchor - padding;
        let screen_size = gfx.screen_size();

        for (r, groups) in play_state.row_groups.iter().enumerate() {
            let number_of_groups = groups.iter().len();
            for i in 0..number_of_groups {
                let grid_offset = vec2(-(i as f32) - 2., r as f32);
                let position = anchor + grid_offset * grid_cell_size * scaler;
                let screen_position = gfx.camera().world_to_screen(position, screen_size);
                // write out the numbers from the right outward for alignment
                let g = number_of_groups - i - 1;
                gfx.text(&format!("{}", groups[g].num_cells))
                    .size(0.5 * box_size)
                    .color(match groups[g].filled {
                        true => Color::new(self.palette.cell_filled_in),
                        false => Color::new(self.palette.background),
                    })
                    .at(screen_position);
            }
        }
    }

    pub fn draw_column_groups(&self, play_state: &PlayState, gfx: &mut Graphics) {
        let offset = self.halfset();
        let anchor = self.anchor();
        let offset = Vec2::splat(offset);
        let num_boxes = play_state.cols().len();
        let box_size = self.box_size(num_boxes);
        let grid_cell_size = Vec2::splat(box_size) + offset;
        let screen_size = gfx.screen_size();

        let padding = offset.y / 2. - box_size / 2.;
        let anchor = anchor - padding;

        let scaler = vec2(1., 0.5);
        let anchor = anchor - offset;
        for (c, groups) in play_state.column_groups.iter().enumerate() {
            let number_of_groups = groups.iter().len();
            for i in 0..number_of_groups {
                let grid_offset = vec2(c as f32, -(i as f32) - 2.);
                let position = anchor + grid_offset * grid_cell_size * scaler;
                let screen_position = gfx.camera().world_to_screen(position, screen_size);
                // render the bottom number closest to the top of the grid, then go up for alignment
                let g = number_of_groups - i - 1;
                gfx.text(&format!("{}", groups[g].num_cells))
                    .size(0.5 * box_size)
                    .color(match groups[g].filled {
                        true => Color::new(self.palette.cell_filled_in),
                        false => Color::new(self.palette.background),
                    })
                    .at(screen_position);
            }
        }
    }
}

pub fn draw_ppm_at(ppm: &Ppm, top_left: Vec2, size: Vec2, gfx: &mut Graphics) {
    let num_boxes = ppm.rows().len();
    let gutter = 2.;
    let cell_size = (size.x - (gutter + gutter * num_boxes as f32)) / num_boxes as f32;

    for (r, row) in ppm.rows().into_iter().enumerate() {
        for (c, rgb) in row.iter().enumerate() {
            let position = top_left + vec2(c as f32, r as f32) * (Vec2::splat(cell_size) + gutter);
            gfx.rect()
                .at(position - gutter)
                .size(Vec2::splat(cell_size + gutter * 2.))
                .color(Color::new(ppm.to_rgba(*rgb)));
        }
    }
}

pub struct DebugStuff {
    pub size_x: usize,
    pub size_y: usize,
    pub selected_level: usize,
    pub transition_duration: f32,
}

impl DebugStuff {
    pub fn new() -> DebugStuff {
        DebugStuff {
            size_x: 200,
            size_y: 200,
            selected_level: 0,
            transition_duration: 0.8,
        }
    }
}

pub fn debug_window(
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
        ui.add(
            Slider::new(&mut debuggable_stuff.transition_duration, 0.5..=5.0).text("Wipe duration"),
        );
    });
}
