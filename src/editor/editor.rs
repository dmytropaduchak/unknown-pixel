use macroquad::prelude::clear_background;
use macroquad::prelude::draw_line;
use macroquad::prelude::draw_rectangle;
use macroquad::prelude::draw_rectangle_lines;
use macroquad::prelude::draw_texture_ex;
use macroquad::prelude::is_key_pressed;
use macroquad::prelude::is_mouse_button_down;
use macroquad::prelude::is_mouse_button_released;
use macroquad::prelude::load_texture;
use macroquad::prelude::mouse_position;
use macroquad::prelude::next_frame;
use macroquad::prelude::screen_height;
use macroquad::prelude::screen_width;
use macroquad::prelude::vec2;
use macroquad::prelude::Color;
use macroquad::prelude::DrawTextureParams;
use macroquad::prelude::FilterMode;
use macroquad::prelude::KeyCode;
use macroquad::prelude::MouseButton;
use macroquad::prelude::BLACK;
use macroquad::prelude::LIGHTGRAY;
use macroquad::prelude::RED;
use macroquad::prelude::WHITE;
use macroquad::prelude::*;
use crate::editor::EditorState;

const PIXEL_SIZE: f32 = 8.0;
const GRID_WIDTH: usize = 320;
const GRID_HEIGHT: usize = 240;

// - rename const
// - add grid size indicator
// - add pen size indicator (right bottom)
// - drawing simple shapes
// - help
// ? change baground
// ? butoons + (keyboard combination)
// ? export rectangle size
//

pub struct Editor {
    editor_state: EditorState,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            editor_state: EditorState::new(),
        }
    }

    pub async fn run(&mut self) {
        loop {
            clear_background(self.editor_state.background_color);
            self.editor_state.update_window_size();
            self.editor_state.handle_input();
            self.editor_state.draw_blocks();
            self.editor_state.draw_shapes();
            self.editor_state.draw_pixels();
            self.editor_state.draw_cursor();
            self.editor_state.draw_layers();
            next_frame().await;
        }
    }
}
