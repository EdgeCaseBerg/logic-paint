use crate::gamestate::CellState;
use crate::gamestate::PlayState;

use egor::{
    math::{Rect, Vec2, vec2},
    render::{Color, Graphics},
};

#[derive(Debug)]
pub enum Action {
    FillCell,
    MarkCell,
}

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

    fn can_highlight_at(&self, cell: &Rect) -> bool {
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

    pub fn draw_gridarea_background(&self, play_state: &PlayState, gfx: &mut Graphics) {
        gfx.rect()
            .at(self.top_left)
            .size(self.size)
            .color(Color::new(self.palette.background));
    }

    pub fn draw_grid(&self, play_state: &mut PlayState, input: &PlayerInput, gfx: &mut Graphics) {
        let halfset = self.grid_gutter / 2.;
        let anchor = self.top_left + Vec2::splat(halfset as f32);
        let offset = Vec2::splat(halfset);
        let num_boxes = play_state.rows().len();
        let box_size =
            (self.size.x as f32 - (halfset + halfset * num_boxes as f32)) / num_boxes as f32;

        let side_areas_size = self.play_area_gutter();
        for (r, row) in play_state.rows().into_iter().enumerate() {
            let cell_size = Vec2::splat(box_size);
            let y_offset = r as f32 * (halfset + box_size);
            gfx.rect()
                .at(anchor - vec2(side_areas_size.x, -y_offset))
                .color(Color::new(self.palette.group_highlight))
                .size(vec2(side_areas_size.x, box_size));

            for (c, state) in row.iter().enumerate() {
                let position = anchor + vec2(c as f32, r as f32) * (Vec2::splat(box_size) + offset);
                let even_odd_bg_color = if r % 2 == 0 {
                    self.palette.grid_even
                } else {
                    self.palette.grid_odd
                };
                let color = match state {
                    CellState::Empty => Color::new(even_odd_bg_color),
                    CellState::Filled => Color::new(self.palette.cell_filled_in),
                    CellState::Incorrect => Color::new(self.palette.cell_incorrect),
                    CellState::RuledOut => Color::new(self.palette.cell_marked_game),
                    CellState::UserRuledOut => Color::new(self.palette.cell_marked_user),
                };
                let cell_rect = Rect::new(position, cell_size);
                if input.can_highlight_at(&cell_rect) {
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

    pub fn draw_row_groups(
        &self,
        play_state: &PlayState,
        _input: &PlayerInput,
        gfx: &mut Graphics,
    ) {
        // _input for background
        let halfset = self.grid_gutter / 2.;
        let num_boxes = play_state.rows().len();
        let box_size =
            (self.size.x as f32 - (halfset + halfset * num_boxes as f32)) / num_boxes as f32;
        let padding = self.grid_gutter / 2. - box_size / 2.;
        let offset = Vec2::splat(halfset);
        let scaler = vec2(0.5, 1.);
        let anchor = self.top_left + Vec2::splat(halfset);
        let anchor = anchor - padding;
        for (r, groups) in play_state.row_groups.iter().enumerate() {
            let number_of_groups = groups.iter().len();
            for i in 0..number_of_groups {
                let grid_offset = vec2(-(i as f32) - 2., r as f32);
                let grid_cell_size = Vec2::splat(box_size) + offset;
                let position = anchor + grid_offset * grid_cell_size * scaler;
                let screen_size = gfx.screen_size();
                let screen_position = gfx.camera().world_to_screen(position, screen_size);
                // write out the numbers from the right outward for alignment
                let g = number_of_groups - i - 1;
                gfx.text(&format!("{}", groups[g].num_cells))
                    .size(0.5 * box_size as f32)
                    .color(match groups[g].filled {
                        true => Color::new(self.palette.cell_filled_in),
                        false => Color::new(self.palette.background),
                    })
                    .at(screen_position);
            }
        }
    }

    pub fn draw_column_groups(
        &self,
        play_state: &PlayState,
        _input: &PlayerInput,
        gfx: &mut Graphics,
    ) {
        let halfset = self.grid_gutter / 2.;
        let anchor = self.top_left + Vec2::splat(halfset as f32);
        let offset = Vec2::splat(halfset);
        let num_boxes = play_state.cols().len();
        let box_size =
            (self.size.x as f32 - (halfset + halfset * num_boxes as f32)) / num_boxes as f32;

        let padding = offset.y / 2. - box_size / 2.;
        let anchor = anchor - padding;

        let scaler = vec2(1., 0.5);
        let anchor = anchor - offset;
        for (c, groups) in play_state.column_groups.iter().enumerate() {
            let number_of_groups = groups.iter().len();
            for i in 0..number_of_groups {
                let grid_offset = vec2(c as f32, -(i as f32) - 2.);
                let grid_cell_size = Vec2::splat(box_size) + offset;
                let position = anchor + grid_offset * grid_cell_size * scaler;
                let screen_size = gfx.screen_size();
                let screen_position = gfx.camera().world_to_screen(position, screen_size);
                // render the bottom number closest to the top of the grid, then go up for alignment
                let g = number_of_groups - i - 1;
                gfx.text(&format!("{}", groups[g].num_cells))
                    .size(0.5 * box_size as f32)
                    .color(match groups[g].filled {
                        true => Color::new(self.palette.cell_filled_in),
                        false => Color::new(self.palette.background),
                    })
                    .at(screen_position);
            }
        }
    }
}
