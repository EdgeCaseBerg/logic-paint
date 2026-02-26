use crate::netppm::Ppm;
use crate::playstate::CellState;
use crate::playstate::PlayState;
use crate::screens::ScreenAction;
use std::fs::read_to_string;
use std::path::PathBuf;

use egor::{
    input::{Input, MouseButton},
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
    pub group_font: [f32; 4],
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
            group_font: rgba(0, 0, 0, 1.0),
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
    pub fn from(input: &Input, gfx: &mut Graphics) -> PlayerInput {
        let screen_size = gfx.screen_size();
        let (mx, my) = input.mouse_position();
        let world_xy = gfx.camera().screen_to_world(Vec2::new(mx, my), screen_size);
        let left_mouse_pressed =
            input.mouse_pressed(MouseButton::Left) || input.mouse_held(MouseButton::Left);
        let right_mouse_pressed =
            input.mouse_pressed(MouseButton::Right) || input.mouse_held(MouseButton::Right);

        PlayerInput {
            position: world_xy,
            action: {
                match (left_mouse_pressed, right_mouse_pressed) {
                    (false, false) => None,
                    (true, false) => Some(Action::FillCell),
                    (_, true) => Some(Action::MarkCell),
                }
            },
        }
    }

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

pub fn draw_x_at(position: Vec2, cell_size: Vec2, color: Color, gfx: &mut Graphics) {
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

struct GridLayout {
    pub area: Rect,
    pub rows: usize,
    pub columns: usize,
    pub cell_gap: f32,
}

impl GridLayout {
    pub fn cell_size(&self) -> Vec2 {
        let total = self.area.size;

        // [gap [cell] gap [cell] gap ]
        let gaps = vec2(
            (self.columns as f32 + 1.0) * self.cell_gap,
            (self.rows as f32 + 1.0) * self.cell_gap,
        );

        // (max(vec)) to avoid negative space
        let space_for_cells = (total - gaps).max(vec2(0.0, 0.0));

        space_for_cells / vec2(self.columns as f32, self.rows as f32)
    }

    // Return the top left of where the cells start within the grid layout (offset by the gap)
    pub fn origin(&self) -> Vec2 {
        self.area.min() + vec2(self.cell_gap, self.cell_gap)
    }

    pub fn cell_rect(&self, r: usize, c: usize) -> Rect {
        let cell = self.cell_size();

        let origin = self.origin();

        let offset = vec2(
            c as f32 * (cell.x + self.cell_gap),
            r as f32 * (cell.y + self.cell_gap),
        );

        Rect {
            position: origin + offset,
            size: cell,
        }
    }

    // For me in 3 months: '_ is an anonymous lifetime tied to self.
    // it just means the caller can't let the iterator last longer than this layout
    pub fn iter_cells(&self) -> impl Iterator<Item = (usize, usize, Rect)> + '_ {
        let cell_size = self.cell_size();
        let origin = self.origin();
        let step = cell_size + Vec2::splat(self.cell_gap);

        (0..self.rows).flat_map(move |r| {
            (0..self.columns).map(move |c| {
                let top_left = origin + vec2(c as f32, r as f32) * step;
                let cell = Rect {
                    position: top_left,
                    size: cell_size,
                };
                (r, c, cell)
            })
        })
    }
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

    fn full_layout(&self, play_state: &PlayState) -> (usize, usize, GridLayout) {
        let max_row_groups = play_state
            .row_groups
            .iter()
            .map(|g| g.len())
            .max()
            .unwrap_or(1); // always include at least a bit of gutter
        let max_column_groups = play_state
            .column_groups
            .iter()
            .map(|g| g.len())
            .max()
            .unwrap_or(1); // always include at least a bit of gutter

        let rows = play_state.rows().len() + max_row_groups;
        let columns = play_state.cols().len() + max_column_groups;
        let layout = GridLayout {
            area: Rect {
                position: self.top_left - self.play_area_gutter(),
                size: self.size + self.play_area_gutter(),
            },
            rows,
            columns,
            cell_gap: self.grid_gutter,
        };
        (max_row_groups, max_column_groups, layout)
    }

    pub fn draw_backgrounds(
        &self,
        play_state: &PlayState,
        input: &PlayerInput,
        gfx: &mut Graphics,
    ) {
        let (origin_x, origin_y, layout) = self.full_layout(&play_state);
        for (r, c, rect) in layout.iter_cells() {
            let (even_odd_bg_color, odd_even_bg_color) = self.palette.even_odd_color(r);
            match (r < origin_x, c < origin_y) {
                (true, true) => {
                    // Draw nothing here in the space in the corner. Though we could
                    // put a cute character smiling or something if we wanted to.
                }
                (false, false) => {
                    // The Grid. https://www.youtube.com/watch?v=lWu40FA3eZk
                    gfx.rect()
                        .at(rect.min())
                        .size(rect.size)
                        .color(Color::new(self.palette.background));
                }
                (true, false) => {
                    // Top gutter (column groups)
                    let colors = [even_odd_bg_color, odd_even_bg_color];
                    let mouse_within_column_range = true
                        && rect.position.x <= input.position.x
                        && input.position.x <= rect.position.x + rect.size.x;
                    let color = if mouse_within_column_range {
                        self.palette.group_highlight
                    } else {
                        colors[c % 2]
                    };
                    gfx.rect()
                        .at(rect.min())
                        .color(Color::new(color))
                        .size(rect.size + vec2(0., layout.cell_gap));
                }
                (false, true) => {
                    // Left gutter (row groups)
                    let mouse_within_row_range = true
                        && rect.position.y <= input.position.y
                        && input.position.y <= rect.position.y + rect.size.y;

                    let color = if mouse_within_row_range {
                        self.palette.group_highlight
                    } else {
                        odd_even_bg_color
                    };
                    // we + gap to join the squares together
                    gfx.rect()
                        .at(rect.min())
                        .color(Color::new(color))
                        .size(rect.size + vec2(layout.cell_gap, 0.));
                }
            }
        }
    }

    pub fn draw_grid(&self, play_state: &mut PlayState, input: &PlayerInput, gfx: &mut Graphics) {
        let (origin_y, origin_x, layout) = self.full_layout(&play_state);
        let state_by_rows = play_state.rows();
        for (r, row) in state_by_rows.iter().enumerate() {
            let (even_odd_bg_color, odd_even_bg_color) = self.palette.even_odd_color(r);
            for (c, cell) in row.iter().enumerate() {
                // + origin because the full layout includes gutters and we need to skip em
                let cell_rect = layout.cell_rect(origin_y + r, origin_x + c);

                let color = match cell {
                    CellState::Empty => Color::new(even_odd_bg_color),
                    CellState::Filled => Color::new(self.palette.cell_filled_in),
                    CellState::Incorrect => Color::new(self.palette.cell_incorrect),
                    CellState::RuledOut => Color::new(self.palette.cell_marked_game),
                    CellState::UserRuledOut => Color::new(self.palette.cell_marked_user),
                };
                if input.overlaps(&cell_rect) {
                    gfx.rect()
                        .at(cell_rect.min() - self.grid_gutter / 2.)
                        .size(cell_rect.size + self.grid_gutter)
                        .color(Color::new(self.palette.cell_highlight));
                }

                match cell {
                    CellState::Empty | CellState::Filled => {
                        gfx.rect()
                            .at(cell_rect.min())
                            .size(cell_rect.size)
                            .color(color);
                    }
                    _ => {
                        gfx.rect()
                            .at(cell_rect.min())
                            .size(cell_rect.size)
                            .color(Color::new(even_odd_bg_color));
                        draw_x_at(cell_rect.min(), cell_rect.size, color, gfx);
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
        let num_boxes = play_state.rows().len();
        let screen_size = gfx.screen_size();
        let side_areas_size = self.play_area_gutter();
        // Because fonts are rendered in screen position, compute their grid layout
        // with respect to that rather than raw world units
        let screen_position = gfx
            .camera()
            .world_to_screen(self.top_left - vec2(side_areas_size.x, 0.), screen_size);

        let layout = GridLayout {
            area: Rect {
                position: screen_position,
                size: vec2(side_areas_size.x, self.size.y),
            },
            rows: num_boxes,
            columns: num_boxes - num_boxes / 2, // If 5 cells, room for 3 numbers [x_x_x] and similar
            cell_gap: self.grid_gutter,
        };

        for (r, groups) in play_state.row_groups.iter().enumerate() {
            let groups_in_row = groups.len();
            let start_col = layout.columns - groups_in_row;

            for (i, group) in groups.iter().enumerate() {
                let column = start_col + i;
                let rect = layout.cell_rect(r, column);
                // for some reason fonts position their _center_ at the position we tell
                // them to be. So just add half in to get the real placement location
                let position = rect.min() + rect.size / 2.;
                let text = &format!("{}", group.num_cells);
                let font_color = match group.filled {
                    true => Color::new(self.palette.cell_filled_in),
                    false => Color::new(self.palette.group_font),
                };

                crate::screens::draw_centered_text(gfx, text, position, rect.size.y, font_color);
            }
        }
    }

    pub fn draw_column_groups(&self, play_state: &PlayState, gfx: &mut Graphics) {
        let num_boxes = play_state.cols().len();
        let screen_size = gfx.screen_size();
        let gutter = self.play_area_gutter();
        let grid_corner = self.top_left - vec2(0., gutter.y);
        let screen_position = gfx.camera().world_to_screen(grid_corner, screen_size);
        let layout = GridLayout {
            area: Rect {
                position: screen_position,
                size: vec2(self.size.x, gutter.y),
            },
            rows: num_boxes - num_boxes / 2,
            columns: num_boxes,
            cell_gap: self.grid_gutter,
        };

        for (c, groups) in play_state.column_groups.iter().enumerate() {
            let number_of_groups = groups.iter().len();
            let start_row = layout.rows - number_of_groups;
            for (i, group) in groups.iter().enumerate() {
                let r = start_row + i;
                let rect = layout.cell_rect(r, c);
                // for some reason fonts position their _center_ at the position we tell
                // them to be. So just add half in to get the real placement location
                let position = rect.min() + rect.size / 2.;
                let text = &format!("{}", group.num_cells);
                let font_color = match group.filled {
                    true => Color::new(self.palette.cell_filled_in),
                    false => Color::new(self.palette.group_font),
                };

                crate::screens::draw_centered_text(gfx, text, position, rect.size.y, font_color);
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

pub struct LoadedPpms {
    pub unknown_level: Ppm,
    pub quit: Ppm,
    pub mouse_left: Ppm,
    pub mouse_right: Ppm,
}

impl LoadedPpms {
    pub fn load(assets: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let unknown_level = read_to_string(assets.join("unsolved.ppm"))?;
        let unknown_level: Ppm = unknown_level.parse()?;

        let quit = read_to_string(assets.join("quit.ppm"))?;
        let quit: Ppm = quit.parse()?;

        let mouse_left = read_to_string(assets.join("mouse-left.ppm"))?;
        let mouse_left: Ppm = mouse_left.parse()?;

        let mouse_right = read_to_string(assets.join("mouse-right.ppm"))?;
        let mouse_right: Ppm = mouse_right.parse()?;

        Ok(LoadedPpms {
            unknown_level,
            quit,
            mouse_left,
            mouse_right,
        })
    }
}

pub fn draw_quit_button(
    position: Vec2,
    size: Vec2,
    ppm: &Ppm,
    palette: &ColorPalette,
    input: &PlayerInput,
    gfx: &mut Graphics,
) -> Option<ScreenAction> {
    let mut action = None;
    let rect = Rect::new(position, size);
    let highlight_color = if input.overlaps(&rect) {
        if input.can_fill_at(&rect) {
            action = Some(ScreenAction::QuitGame);
        }
        Color::new(palette.cell_incorrect)
    } else {
        Color::new(palette.group_highlight)
    };
    gfx.rect()
        .color(highlight_color)
        .at(position - 4.)
        .size(size + 4.);
    draw_ppm_at(&ppm, position, size, gfx);
    return action;
}

// Returns a tuple that you can use to compute
// x,y locations in world units. Assuming a screen
// of 32 by 18 for placement.
pub fn world_unit_size() -> Vec2 {
    let x_unit = 1280. / 32.;
    let y_unit = 720. / 18.;
    vec2(x_unit, y_unit)
}
