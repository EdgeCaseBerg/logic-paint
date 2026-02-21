use crate::level_settings::LevelSettings;
use crate::levels::Level;
use crate::netbpm::Pbm;
use crate::netppm::Ppm;
use egor::app::FrameContext;
use egor::app::egui::lerp;
use egor::input::MouseButton;
use egor::math::Rect;
use egor::math::{Vec2, vec2};
use egor::render::Color;
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
    pub fn ui(&mut self, frame_context: &mut FrameContext, level_settings: &mut LevelSettings) {
        let gfx = &mut (frame_context.gfx);
        let input = &mut (frame_context.input);
        let left_mouse_pressed =
            input.mouse_pressed(MouseButton::Left) || input.mouse_held(MouseButton::Left);
        let right_mouse_pressed =
            input.mouse_pressed(MouseButton::Right) || input.mouse_held(MouseButton::Right);
        let (mx, my) = input.mouse_position();
        let screen_size = gfx.screen_size();
        let world_xy = gfx.camera().screen_to_world(Vec2::new(mx, my), screen_size);

        let num_boxes_x = level_settings.width;
        let num_boxes_y = level_settings.height; // TODO: maybe just always have a square
        let gutter = 2.;
        let cell_size = (self.size.x - (gutter + gutter * num_boxes_x as f32)) / num_boxes_x as f32;
        let cell_size = Vec2::splat(cell_size);

        let pbm_anchor = self.top_left;
        gfx.rect()
            .at(pbm_anchor)
            .size(self.size)
            .color(Color::WHITE);

        let pbm_anchor = pbm_anchor + gutter;
        for r in 0..num_boxes_y {
            for c in 0..num_boxes_x {
                let position = pbm_anchor + vec2(c as f32, r as f32) * (cell_size + gutter);

                if Rect::new(position, cell_size).contains(world_xy) && left_mouse_pressed {
                    self.pbm_grid[r][c] = true;
                }
                if Rect::new(position, cell_size).contains(world_xy) && right_mouse_pressed {
                    self.pbm_grid[r][c] = false;
                }
                let color = if self.pbm_grid[r][c] {
                    Color::RED
                } else {
                    Color::BLACK
                };

                gfx.rect().at(position).size(cell_size).color(color);
            }
        }

        let ppm_anchor = pbm_anchor + vec2(400. + 50., 0.);
        gfx.rect()
            .at(ppm_anchor)
            .size(self.size)
            .color(Color::WHITE);

        let ppm_anchor = ppm_anchor + gutter;
        for r in 0..num_boxes_y {
            for c in 0..num_boxes_x {
                let position = ppm_anchor + vec2(c as f32, r as f32) * (cell_size + gutter);

                if Rect::new(position, cell_size).contains(world_xy) && left_mouse_pressed {
                    self.ppm_grid[r][c] = level_settings.current_color;
                }
                if Rect::new(position, cell_size).contains(world_xy) && right_mouse_pressed {
                    self.ppm_grid[r][c] = [0.0, 0.0, 0.0, 1.0];
                }
                let rgb = self.ppm_grid[r][c];

                gfx.rect()
                    .at(position)
                    .size(cell_size)
                    .color(Color::new(rgb));
            }
        }
    }
}

pub fn save_grid_as_level(level_settings: &LevelSettings, grids: &EditorGrids) -> Level {
    let ppm = grids.into();
    let pbm = grids.into();
    let path: PathBuf = ["./levels", &level_settings.filename, ].iter().collect();
    Level {
        info: pbm,
        image: ppm,
        completed: false,
        path: path.with_extension("level"),
    }
}