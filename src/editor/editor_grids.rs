use crate::base_dir;
use crate::editor::editor_settings::LevelSettings;
use crate::editor::editor_ui_actions::UiActions;
use crate::editor::solver::TheMultiVerseOfLines;
use crate::levels::Level;
use crate::netpbm::Pbm;
use crate::netpbm::Ppm;
use crate::ui::GridLayout;
use crate::ui::draw_centered_text;
use egor::app::FrameContext;
use egor::input::MouseButton;
use egor::math::Rect;
use egor::math::{Vec2, vec2};
use egor::render::Color;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct EditorGrids {
    pub pbm_grid: Vec<Vec<bool>>,
    pub ppm_grid: Vec<Vec<[f32; 4]>>,
    pub size: Vec2,
    pub top_left: Vec2,
}

impl Default for EditorGrids {
    fn default() -> EditorGrids {
        let mut pbm_grid = Vec::with_capacity(20);
        let mut ppm_grid = Vec::with_capacity(20);

        for _ in 0..20 {
            let mut pbm_row = Vec::with_capacity(20);
            let mut ppm_row = Vec::with_capacity(20);
            for _ in 0..20 {
                pbm_row.push(false);
                ppm_row.push([0.0, 0.0, 0.0, 1.0]);
            }
            pbm_grid.push(pbm_row);
            ppm_grid.push(ppm_row);
        }

        EditorGrids {
            pbm_grid,
            ppm_grid,
            size: vec2(400., 400.),
            top_left: vec2(400., 120.), // [ 90 + 500 + 100 + 500 + 90  ]
        }
    }
}

impl EditorGrids {
    pub fn load_level(&mut self, level: &Level) {
        for (r, row) in level.info.rows().into_iter().enumerate() {
            for (c, b) in row.into_iter().enumerate() {
                self.pbm_grid[r][c] = b;
            }
        }
        for (r, row) in level.image.rows().into_iter().enumerate() {
            for (c, triplet) in row.into_iter().enumerate() {
                self.ppm_grid[r][c] = level.image.to_rgba(triplet);
            }
        }
    }

    pub fn unique_colors(&self) -> Vec<[f32; 4]> {
        let mut unique = HashMap::new();
        for row in &self.ppm_grid {
            for &[r, g, b, a] in row {
                // each value is u16, 16 * 4 = 64 and so...
                let key = (percent_to_u16(r, 255.) as u64) << 48
                    | (percent_to_u16(g, 255.) as u64) << 32
                    | (percent_to_u16(b, 255.) as u64) << 16
                    | (percent_to_u16(a, 255.) as u64);
                unique.insert(key, [r, g, b, a]);
            }
        }
        let mut unique: Vec<_> = unique.into_values().collect();
        unique.sort_by(|a, b| {
            // https://doc.rust-lang.org/std/primitive.f32.html#method.total_cmp
            a[0].total_cmp(&b[0])
                .then(a[1].total_cmp(&b[1]))
                .then(a[2].total_cmp(&b[2]))
                .then(a[3].total_cmp(&b[3]))
        });

        unique
    }

    pub fn ui(
        &mut self,
        frame_context: &mut FrameContext,
        level_settings: &mut LevelSettings,
        last_known_solve: &TheMultiVerseOfLines,
    ) -> UiActions {
        let mut action = UiActions::Nothing;

        let gfx = &mut (frame_context.gfx);
        let input = &mut (frame_context.input);
        let left_mouse_pressed =
            input.mouse_pressed(MouseButton::Left) || input.mouse_held(MouseButton::Left);
        let right_mouse_pressed =
            input.mouse_pressed(MouseButton::Right) || input.mouse_held(MouseButton::Right);
        let (mx, my) = input.mouse_position();
        let world_xy = gfx.camera().screen_to_world(Vec2::new(mx, my));

        let num_boxes_x = level_settings.width;
        let num_boxes_y = level_settings.height; // TODO: maybe just always have a square
        let gutter = 2.;
        let cell_size_x =
            (self.size.x - (gutter + gutter * num_boxes_x as f32)) / num_boxes_x as f32;
        let cell_size_y =
            (self.size.y - (gutter + gutter * num_boxes_y as f32)) / num_boxes_y as f32;
        let cell_size = vec2(cell_size_x, cell_size_y);

        let pbm_anchor = self.top_left;

        let layout = GridLayout {
            area: Rect {
                position: self.top_left,
                size: self.size,
            },
            rows: level_settings.height,
            columns: level_settings.width,
            cell_gap: 2.,
        };

        // Draw the background, highlighting bad cells as needed
        let mut bg_layout = layout.shifted_by(Vec2::splat(-layout.cell_gap));
        bg_layout.cell_gap = 0.0;
        for (r, c, rect) in bg_layout.iter_cells() {
            // When we resize bad things can happen, so skip until the user is done.
            if r >= last_known_solve.rows.len() {
                continue;
            }
            if c >= last_known_solve.columns.len() {
                continue;
            }
            let potential_row_patterns = last_known_solve.rows[r].len();
            let potential_column_patterns = last_known_solve.columns[c].len();
            let color = match (potential_row_patterns, potential_column_patterns) {
                (1, 1) => Color::GREEN,
                (0, 0) => Color::RED,
                (1, _) => Color::new([1.0, 0.5, 0.0, 1.0]),
                (_, 1) => Color::new([1.0, 0.5, 0.0, 1.0]),
                _ => Color::new([1.0, 1.0, 0.0, 1.0]),
            };
            gfx.rect().at(rect.position).size(rect.size).color(color);
        }

        // List how many potential options there are next to the side
        let row_potential_listing = layout.shifted_by(Vec2::new(-layout.cell_size().x, 0.));
        for (r, c, rect) in row_potential_listing.iter_cells() {
            // When we resize bad things can happen, so skip until the user is done.
            // also, we don't care about columns beyond
            if r >= last_known_solve.rows.len() || c > 0 {
                continue;
            }
            let potential_patterns = last_known_solve.rows[r].len();
            if potential_patterns == 1 {
                continue;
            }
            draw_centered_text(
                gfx,
                &format!("{}", potential_patterns),
                rect.position + rect.size * 0.5,
                rect.size.y,
                Color::RED,
            );
        }

        // List how many potential options there are next to the top
        let column_potential_listing = layout.shifted_by(Vec2::new(0., -layout.cell_size().y));
        for (r, c, rect) in column_potential_listing.iter_cells() {
            // When we resize bad things can happen, so skip until the user is done.
            // also, we don't care about rows beyond the first
            if c >= last_known_solve.columns.len() || r > 0 {
                continue;
            }
            let potential_patterns = last_known_solve.columns[c].len();
            if potential_patterns == 1 {
                continue;
            }
            draw_centered_text(
                gfx,
                &format!("{}", potential_patterns),
                rect.position + rect.size * 0.5,
                rect.size.y,
                Color::RED,
            );
        }

        for (r, c, rect) in layout.iter_cells() {
            if rect.contains(world_xy) && left_mouse_pressed {
                action = UiActions::LevelGridUpdated;
                self.pbm_grid[r][c] = true;
            }
            if rect.contains(world_xy) && right_mouse_pressed {
                action = UiActions::LevelGridUpdated;
                self.pbm_grid[r][c] = false;
            }
            let color = if self.pbm_grid[r][c] {
                Color::WHITE
            } else {
                Color::BLACK
            };

            gfx.rect().at(rect.position).size(rect.size).color(color);
        }

        let layout = layout.shifted_by(vec2(400. + 50., 0.));
        gfx.rect()
            .at(layout.area.position)
            .size(layout.area.size)
            .color(Color::WHITE);

        for (r, c, rect) in layout.iter_cells() {
            if rect.contains(world_xy) && left_mouse_pressed {
                self.ppm_grid[r][c] = level_settings.current_color;
            }
            if rect.contains(world_xy) && right_mouse_pressed {
                self.ppm_grid[r][c] = [0.0, 0.0, 0.0, 1.0];
            }
            let rgb = self.ppm_grid[r][c];

            gfx.rect()
                .at(rect.position)
                .size(rect.size)
                .color(Color::new(rgb));
        }
        action
    }
}

pub fn save_grid_as_level(level_settings: &LevelSettings, grids: &EditorGrids) -> Level {
    let ppm = (level_settings, grids).into();
    let pbm = (level_settings, grids).into();
    let base = base_dir();
    let path: PathBuf = ["levels", &level_settings.filename].iter().collect();
    let path = base.join(path);
    Level {
        info: pbm,
        image: ppm,
        completed: false,
        path: path.with_extension("level"),
    }
}

fn percent_to_u16(t: f32, max_value: f32) -> u16 {
    (t.clamp(0.0, 1.0) * max_value).round() as u16
}

impl From<(&LevelSettings, &EditorGrids)> for Ppm {
    fn from(tuple: (&LevelSettings, &EditorGrids)) -> Ppm {
        let (level_settings, grids) = tuple;
        let width = level_settings.width;
        let height = level_settings.height;
        let mut cells = Vec::with_capacity(height * width);
        let max_value = 255.;

        for r in 0..height {
            for c in 0..width {
                // [0.15354905, 0.13828914, 0.6661099, 1.0]
                let rgba = grids.ppm_grid[r][c];
                let r: u16 = percent_to_u16(rgba[0], max_value);
                let g: u16 = percent_to_u16(rgba[1], max_value);
                let b: u16 = percent_to_u16(rgba[2], max_value);
                cells.push([r, g, b]);
            }
        }

        Ppm {
            width,
            height,
            max_value: max_value as u16,
            cells,
        }
    }
}

impl From<(&LevelSettings, &EditorGrids)> for Pbm {
    fn from(tuple: (&LevelSettings, &EditorGrids)) -> Pbm {
        let (level_settings, grids) = tuple;
        let width = level_settings.width;
        let height = level_settings.height;

        let mut cells = Vec::with_capacity(height * width);
        for r in 0..height {
            for c in 0..width {
                cells.push(grids.pbm_grid[r][c]);
            }
        }

        Pbm {
            width,
            height,
            cells,
        }
    }
}
