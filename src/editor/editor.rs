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
                    LIGHTGRAY.with_alpha(0.5),
                );
            }
            for y in 0..=GRID_HEIGHT {
                draw_line(
                    0.0,
                    y as f32 * self.size,
                    GRID_WIDTH as f32 * self.size,
                    y as f32 * self.size,
                    1.0,
                    LIGHTGRAY.with_alpha(0.5),
                );
            }

            self.pixels();
            self.pixel();

            if is_mouse_button_down(MouseButton::Left) {
                if !self.draw {
                    self.undo_stack.push(self.grid.clone());
                    self.redo_stack.clear();
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

            if !is_key_down(KeyCode::LeftSuper) && is_key_pressed(KeyCode::Equal) {
                self.pen_size += 1;
            }
            if !is_key_down(KeyCode::LeftSuper)
                && is_key_pressed(KeyCode::Minus)
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

            self.actions();
            // Show actual pixel size of pen (as rectangle in screen units)
            let pen_pixel_size = self.pen_size as f32 * self.size;
            let x = self.w - pen_pixel_size - 20.0;
            let y = self.h - pen_pixel_size - 20.0;
            draw_rectangle(x, y, pen_pixel_size, pen_pixel_size, BLACK);

            // println!("{}", self.draw);
            // println!("{}", self.draw);

            // let x = self.w - 100.0 + (self.pen_size as f32 / 2.0);
            // let y = self.h - 100.0 + (self.pen_size as f32 / 2.0);
            // draw_rectangle(x, y, self.pen_size as f32, self.pen_size as f32, BLACK);

            let text = format!("GRID: {}x{}", self.size, self.size);
            draw_text(&text, 20.0, 20.0, 24.0, BLACK);

            // let text = format!("PEN: {}x{}", self.pen_size, self.pen_size);
            // draw_text(&text, 20.0, 40.0, 24.0, BLACK);

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
        let w = 640.0;
        let h = 480.0;
        let x = self.w / 2.0 - w / 2.0;
        let y = self.h / 2.0 - h / 2.0;
        draw_rectangle_lines(x, y, w, h, 2.0, RED);

        let w = 1280.0;
        let h = 720.0;
        let x = self.w / 2.0 - w / 2.0;
        let y = self.h / 2.0 - h / 2.0;
        draw_rectangle_lines(x, y, w, h, 2.0, RED);
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

    fn actions(&mut self) {
        if is_key_down(KeyCode::H) {
            let help_items = [
                ("HELP", ""),
                ("[CMD+Z]", "Undo the last action"),
                ("[CMD+Y]", "Redo the undone action"),
                ("[CMD+E]", "Export to drawing function (console)"),
                // ("[CMD+S]", "Toggle snap mode, align to nearby points"),
                // ("[CMD+G]", "Toggle background grid visibility"),
                ("[H]", "Show or hide this help overlay"),
            ];
            let text_size = 20.0;
            let spacing = 6.0;
            let line_height = text_size + spacing;
            let total_height = help_items.len() as f32 * line_height;

            let start_y = screen_height() / 2.0 - total_height / 2.0;
            let padding = 20.0;

            for (i, (shortcut, description)) in help_items.iter().enumerate() {
                let y = start_y + i as f32 * line_height;

                if description.is_empty() {
                    draw_text(shortcut, padding, y, text_size, RED);
                } else {
                    draw_text(shortcut, padding, y, text_size, RED);
                    draw_text(description, padding + 80.0, y, text_size, RED);
                }
            }
        }
    }
}
