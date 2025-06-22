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
    cursor: f32,
    color: Color,
    grid: Vec<Vec<bool>>,
    draw: bool,
    size: f32,       // grid cell size (zoom)
    pen_size: usize, // pen pixel size
    w: f32,
    h: f32,
    undo_stack: Vec<Vec<Vec<bool>>>,
    redo_stack: Vec<Vec<Vec<bool>>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Rect {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
}

impl Editor {
    pub fn new() -> Self {
        let size = PIXEL_SIZE;
        let cursor = PIXEL_SIZE;
        let color = WHITE;
        let grid = vec![vec![false; GRID_WIDTH]; GRID_HEIGHT];
        let draw = false;
        let w = screen_width();
        let h = screen_height();
        let pen_size = 1;
        let undo_stack = Vec::new();
        let redo_stack = Vec::new();

        Editor {
            cursor,
            color,
            grid,
            draw,
            size,
            pen_size,
            w,
            h,
            undo_stack,
            redo_stack,
        }
    }

    pub async fn run(&mut self) {
        let texture = load_texture("example.png").await.unwrap();
        texture.set_filter(FilterMode::Nearest);

        loop {
            clear_background(self.color);

            self.w = screen_width();
            self.h = screen_height();

            let dest_size = Some(vec2(self.w, self.h));
            let params = DrawTextureParams {
                dest_size,
                ..Default::default()
            };
            draw_texture_ex(&texture, 0.0, 0.0, self.color, params);

            for x in 0..=GRID_WIDTH {
                draw_line(
                    x as f32 * self.size,
                    0.0,
                    x as f32 * self.size,
                    GRID_HEIGHT as f32 * self.size,
                    1.0,
                    LIGHTGRAY,
                );
            }
            for y in 0..=GRID_HEIGHT {
                draw_line(
                    0.0,
                    y as f32 * self.size,
                    GRID_WIDTH as f32 * self.size,
                    y as f32 * self.size,
                    1.0,
                    LIGHTGRAY,
                );
            }

            self.pixels();
            self.pixel();

            if is_mouse_button_down(MouseButton::Left) {
                if !self.draw {
                    self.push_undo();
                }
                self.draw = true;
            }
            if is_mouse_button_released(MouseButton::Left) {
                self.draw = false;
            }

            // Undo (Ctrl+Z)
            if is_key_pressed(KeyCode::Z) && is_key_down(KeyCode::LeftSuper) {
                self.undo();
            }
            // Redo (Ctrl+Y)
            if is_key_pressed(KeyCode::Y) && is_key_down(KeyCode::LeftSuper) {
                self.redo();
            }

            // Export [Ctrl+E]
            if is_key_pressed(KeyCode::E) && is_key_down(KeyCode::LeftSuper) {
                self.export();
            }

            if !is_key_down(KeyCode::LeftSuper) && is_key_pressed(KeyCode::Minus) {
                self.pen_size += 1;
            }
            if !is_key_down(KeyCode::LeftSuper)
                && is_key_pressed(KeyCode::Equal)
                && self.pen_size > 1
            {
                self.pen_size -= 1;
            }

            // Grid size (zoom)
            if is_key_down(KeyCode::LeftSuper) && is_key_pressed(KeyCode::Equal) {
                self.size += 1.0;
            }
            if is_key_down(KeyCode::LeftSuper) && is_key_pressed(KeyCode::Minus) {
                if self.size > 1.0 {
                    self.size -= 1.0;
                }
            }

            self.display();

            let text = format!("GRID: {}x{}", self.size, self.size);
            draw_text(&text, 20.0, 20.0, 24.0, BLACK);

            let text = format!("PEN: {}x{}", self.pen_size, self.pen_size);
            draw_text(&text, 20.0, 40.0, 24.0, BLACK);

            next_frame().await;
        }
    }

    fn pixel(&mut self) {
        if self.draw {
            let (mx, my) = mouse_position();
            let gx = (mx / self.size).floor() as isize;
            let gy = (my / self.size).floor() as isize;
            for dy in 0..self.pen_size as isize {
                for dx in 0..self.pen_size as isize {
                    let x = gx + dx;
                    let y = gy + dy;
                    if x >= 0 && y >= 0 && (x as usize) < GRID_WIDTH && (y as usize) < GRID_HEIGHT {
                        self.grid[y as usize][x as usize] = true;
                    }
                }
            }
        }
    }

    fn push_undo(&mut self) {
        self.undo_stack.push(self.grid.clone());
        self.redo_stack.clear();
    }

    fn undo(&mut self) {
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack.push(self.grid.clone());
            self.grid = prev;
        }
    }

    fn redo(&mut self) {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(self.grid.clone());
            self.grid = next;
        }
    }

    fn pixels(&self) {
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                if self.grid[y][x] {
                    draw_rectangle(
                        x as f32 * self.size,
                        y as f32 * self.size,
                        self.size,
                        self.size,
                        BLACK,
                    );
                }
            }
        }
    }

    fn display(&self) {
        let color = RED.with_alpha(0.3);

        let w = 640.0;
        let h = 480.0;
        let x = self.w / 2.0 - w / 2.0;
        let y = self.h / 2.0 - h / 2.0;
        draw_rectangle_lines(x, y, w, h, 2.0, color);

        let w = 1280.0;
        let h = 720.0;
        let x = self.w / 2.0 - w / 2.0;
        let y = self.h / 2.0 - h / 2.0;
        draw_rectangle_lines(x, y, w, h, 2.0, color);
    }

    fn export(&self) {
        let mut rects = vec![];
        let mut visited = vec![vec![false; GRID_WIDTH]; GRID_HEIGHT];

        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                if self.grid[y][x] && !visited[y][x] {
                    let mut w = 1;
                    while x + w < GRID_WIDTH && self.grid[y][x + w] && !visited[y][x + w] {
                        w += 1;
                    }
                    let mut h = 1;
                    'outer: while y + h < GRID_HEIGHT {
                        for dx in 0..w {
                            if !self.grid[y + h][x + dx] || visited[y + h][x + dx] {
                                break 'outer;
                            }
                        }
                        h += 1;
                    }
                    for dy in 0..h {
                        for dx in 0..w {
                            visited[y + dy][x + dx] = true;
                        }
                    }
                    rects.push(Rect { x, y, w, h });
                }
            }
        }

        println!("fn draw_exported_picture(x_offset: f32, y_offset: f32) {{");
        for r in rects {
            println!(
                "    draw_rectangle(x_offset + {:.1}, y_offset + {:.1}, {:.1}, {:.1}, BLUE);",
                r.x as f32 * self.size,
                r.y as f32 * self.size,
                r.w as f32 * self.size,
                r.h as f32 * self.size,
            );
        }
        println!("}}");
    }
}
