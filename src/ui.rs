use crate::gamestate::PlayState;

use egor::{
    math::Vec2,
    render::{Color, Graphics},
    time::FrameTimer,
};

// #[derive(Debug)]
// enum Screen {
// 	TitleScreen,
// 	PlayScreen,
// }

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

pub trait UiComponent {
    fn draw(&self, timer: &FrameTimer, gfx: &mut Graphics);
}

pub struct PlayArea<'play_state_lifetime> {
    play_state: &'play_state_lifetime mut PlayState,
    top_left: Vec2,
    size: Vec2,
}

impl<'play_state_lifetime> PlayArea<'play_state_lifetime> {
    pub fn new(
        play_state: &'play_state_lifetime mut PlayState,
        at: Vec2,
    ) -> PlayArea<'play_state_lifetime> {
        let size = Vec2::splat(525.);
        PlayArea {
            play_state,
            top_left: at,
            size,
        }
    }
}

impl<'play_state_lifetime> UiComponent for PlayArea<'play_state_lifetime> {
    fn draw(&self, _timer: &FrameTimer, gfx: &mut Graphics) {
        gfx.rect()
            .at(self.top_left)
            .size(self.size)
            .color(Color::BLUE);
    }
}
