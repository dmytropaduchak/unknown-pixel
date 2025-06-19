use macroquad::prelude::clear_background;
use macroquad::prelude::draw_line;
use macroquad::prelude::draw_rectangle;
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
use macroquad::prelude::WHITE;

const PIXEL_SIZE: f32 = 8.0;
const GRID_WIDTH: usize = 320;
const GRID_HEIGHT: usize = 240;

pub struct Editor {
    color: Color,
    // texture: Option<Texture2D>,
    // state: EditorState,
}

impl Editor {
    pub fn new() -> Self {
        let color = WHITE;
        // let texture = None;
        // let state = EditorState::new();

        Editor {
            color,
            // state,
            // texture,
        }
    }

    pub async fn run(&mut self) {
        let texture = load_texture("example.png").await.unwrap();
        texture.set_filter(FilterMode::Nearest);

        let mut grid = vec![vec![false; GRID_WIDTH]; GRID_HEIGHT];
        let mut drawing = false;
        // let mut cursor = PIXEL_SIZE;

        loop {
            clear_background(self.color);

            let screen_width = screen_width();
            let screen_height = screen_height();

            draw_texture_ex(
                &texture,
                0.0,
                0.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(screen_width, screen_height)),
                    ..Default::default()
                },
            );
            // Draw grid lines
            for x in 0..=GRID_WIDTH {
                draw_line(
                    x as f32 * PIXEL_SIZE,
                    0.0,
                    x as f32 * PIXEL_SIZE,
                    GRID_HEIGHT as f32 * PIXEL_SIZE,
                    1.0,
                    LIGHTGRAY,
                );
            }
            for y in 0..=GRID_HEIGHT {
                draw_line(
                    0.0,
                    y as f32 * PIXEL_SIZE,
                    GRID_WIDTH as f32 * PIXEL_SIZE,
                    y as f32 * PIXEL_SIZE,
                    1.0,
                    LIGHTGRAY,
                );
            }
            // Draw pixels
            for y in 0..GRID_HEIGHT {
                for x in 0..GRID_WIDTH {
                    if grid[y][x] {
                        draw_rectangle(
                            x as f32 * PIXEL_SIZE,
                            y as f32 * PIXEL_SIZE,
                            PIXEL_SIZE,
                            PIXEL_SIZE,
                            BLACK,
                        );
                    }
                }
            }

            // Input
            if is_mouse_button_down(MouseButton::Left) {
                drawing = true;
            } else if is_mouse_button_released(MouseButton::Left) {
                drawing = false;
            }

            if drawing {
                let (mx, my) = mouse_position();
                let gx = (mx / PIXEL_SIZE).floor() as usize;
                let gy = (my / PIXEL_SIZE).floor() as usize;
                if gx < GRID_WIDTH && gy < GRID_HEIGHT {
                    grid[gy][gx] = true;
                }
            }

            if is_key_pressed(KeyCode::E) {
                self.export_combined(&grid);
            }

            next_frame().await;
        }
    }
    fn export_combined(&self, grid: &[Vec<bool>]) {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        struct Rect {
            x: usize,
            y: usize,
            w: usize,
            h: usize,
        }

        let mut rects = vec![];
        let mut visited = vec![vec![false; GRID_WIDTH]; GRID_HEIGHT];

        // Pass 1: collect all filled pixels
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                if grid[y][x] && !visited[y][x] {
                    // Expand as much as possible
                    let mut w = 1;
                    while x + w < GRID_WIDTH && grid[y][x + w] && !visited[y][x + w] {
                        w += 1;
                    }

                    let mut h = 1;
                    'outer: while y + h < GRID_HEIGHT {
                        for dx in 0..w {
                            if !grid[y + h][x + dx] || visited[y + h][x + dx] {
                                break 'outer;
                            }
                        }
                        h += 1;
                    }

                    // Mark visited
                    for dy in 0..h {
                        for dx in 0..w {
                            visited[y + dy][x + dx] = true;
                        }
                    }

                    rects.push(Rect { x, y, w, h });
                }
            }
        }
        const EXPORT_PIXEL_SIZE: f32 = 2.0;
        // Final pass: generate code
        println!("fn draw_exported_picture(x_offset: f32, y_offset: f32) {{");
        for r in rects {
            println!(
                "    draw_rectangle(x_offset + {:.1}, y_offset + {:.1}, {:.1}, {:.1}, BLUE);",
                r.x as f32 * EXPORT_PIXEL_SIZE,
                r.y as f32 * EXPORT_PIXEL_SIZE,
                r.w as f32 * EXPORT_PIXEL_SIZE,
                r.h as f32 * EXPORT_PIXEL_SIZE,
            );
        }
        println!("}}");
    }
}
