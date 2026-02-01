use crate::gamestate::PlayState;

use egor::{
    math::Vec2,
    render::{Color, Graphics},
    time::FrameTimer,
};

#[derive(Debug)]
enum MouseState {
    LeftClick { x: f32, y: f32 },
    RightClick { x: f32, y: f32 },
}

#[derive(Debug)]
struct Mouse {
    // position in world units
    position: Vec2,
    click_state: Option<MouseState>,
}

pub struct PlayArea {
    pub top_left: Vec2,
    pub size: Vec2,
}

impl PlayArea {
    pub fn draw(&self, play_state: &PlayState, _timer: &FrameTimer, gfx: &mut Graphics) {
        gfx.rect()
            .at(self.top_left)
            .size(self.size)
            .color(Color::BLUE);
    }
}
