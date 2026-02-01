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
    fn is_fill_at(&self, cell: Rect) -> bool {
        match self.action {
            Some(Action::FillCell) => cell.contains(self.position),
            _ => false,
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
        // I suppose the background for the row/group highlights should be here?
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
                    CellState::RuledOut => Color::new([0.5, 0.5, 0.5, 1.0]),
                    CellState::UserRuledOut => Color::new([0.5, 0.5, 0.5, 1.0]),
                };
                gfx.rect().at(position).size(size).color(color);
                if input.is_fill_at(Rect::new(position, size)) {
                    play_state.attempt_fill(r, c);
                }
            }
        }
    }
}
