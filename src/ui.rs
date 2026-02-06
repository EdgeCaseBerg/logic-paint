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

#[derive(Debug)]
pub struct PlayerInput {
    // position in world units
    pub position: Vec2,
    pub action: Option<Action>,
}

impl PlayerInput {
    fn can_fill_at(&self, cell: Rect) -> bool {
        match self.action {
            Some(Action::FillCell) => cell.contains(self.position),
            _ => false,
        }
    }

    fn can_mark_at(&self, cell: Rect) -> bool {
        match self.action {
            Some(Action::MarkCell) => cell.contains(self.position),
            _ => false
        }
    }
}

pub struct PlayArea {
    pub top_left: Vec2,
    pub size: Vec2,
    pub grid_gutter: f32,
}

impl PlayArea {
    pub fn draw_gridarea_background(&self, _play_state: &PlayState, gfx: &mut Graphics) {
        gfx.rect()
            .at(self.top_left)
            .size(self.size)
            .color(Color::BLUE);
    }

    pub fn draw_grid(&self, play_state: &mut PlayState, input: &PlayerInput, gfx: &mut Graphics) {
        let halfset = self.grid_gutter / 2.;
        let anchor = self.top_left + Vec2::splat(halfset as f32);
        let offset = Vec2::splat(halfset);
        let num_boxes = play_state.rows().len();
        let box_size =
            (self.size.x as f32 - (halfset + halfset * num_boxes as f32)) / num_boxes as f32;

        for (r, row) in play_state.rows().into_iter().enumerate() {
            for (c, state) in row.iter().enumerate() {
                let box_size = box_size as f32;
                let position = anchor + vec2(c as f32, r as f32) * (Vec2::splat(box_size) + offset);
                let size = Vec2::splat(box_size);
                let color = match state {
                    CellState::Empty => Color::WHITE,
                    CellState::Filled => Color::GREEN,
                    CellState::Incorrect => Color::RED,
                    CellState::RuledOut => Color::new([0.3, 0.3, 0.3, 1.0]),
                    CellState::UserRuledOut => Color::new([0.5, 0.5, 0.5, 1.0]),
                };
                gfx.rect().at(position).size(size).color(color);
                if input.can_fill_at(Rect::new(position, size)) {
                    play_state.attempt_fill(r, c);
                }
                if input.can_mark_at(Rect::new(position, size)) {
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
                        true => Color::GREEN,
                        false => Color::WHITE,
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
                        true => Color::GREEN,
                        false => Color::WHITE,
                    })
                    .at(screen_position);
            }
        }
    }
}
