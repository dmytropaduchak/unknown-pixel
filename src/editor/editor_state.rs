use macroquad::prelude::*;

const PIXEL_SIZE: f32 = 8.0;
const GRID_WIDTH: usize = 320;
const GRID_HEIGHT: usize = 240;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ShapeTool {
    Pixel,
    Line,
    Rect,
    Circle,
    Hex,
}

pub struct Layer {
    pub grid: Vec<Vec<bool>>,
    pub visible: bool,
    pub name: String,
}

impl Layer {
    pub fn new(name: String) -> Self {
        Self {
            grid: vec![vec![false; GRID_WIDTH]; GRID_HEIGHT],
            visible: true,
            name,
        }
    }
}

pub struct EditorState {
    pub grid: Vec<Vec<bool>>,
    pub draw: bool,
    pub size: f32,
    pub pen_size: usize,
    pub color: Color,
    pub background_color: Color,
    pub w: f32,
    pub h: f32,
    pub undo_stack: Vec<Vec<Vec<bool>>>,
    pub redo_stack: Vec<Vec<Vec<bool>>>,
    pub panel_width: f32,
    pub panel_width_frac: f32,
    pub texture: Option<Texture2D>,
    pub shape_tool: ShapeTool,
    pub layers: Vec<Layer>,
    pub selected_layer: usize,
    pub drag_start: Option<(isize, isize)>,
    pub drag_end: Option<(isize, isize)>,
}

impl EditorState {
    pub fn new() -> Self {
        let mut layers = vec![Layer::new("Layer 1".to_string())];
        Self {
            grid: vec![vec![false; GRID_WIDTH]; GRID_HEIGHT],
            draw: false,
            size: PIXEL_SIZE,
            pen_size: 1,
            color: WHITE,
            background_color: WHITE,
            w: screen_width(),
            h: screen_height(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            panel_width: screen_width() * 0.2,
            panel_width_frac: 0.2,
            texture: None,
            shape_tool: ShapeTool::Pixel,
            layers,
            selected_layer: 0,
            drag_start: None,
            drag_end: None,
        }
    }

    pub fn update_window_size(&mut self) {
        self.w = screen_width();
        self.h = screen_height();
        self.panel_width = self.w * self.panel_width_frac;
    }

    fn editor_area(&self) -> (f32, f32, f32, f32) {
        let width = GRID_WIDTH as f32 * self.size;
        let height = GRID_HEIGHT as f32 * self.size;
        let x = self.panel_width;
        let y = (self.h - height) / 2.0;
        (x, y, width, height)
    }

    fn mouse_grid_pos(&self) -> Option<(isize, isize)> {
        let (editor_x, editor_y, width, height) = self.editor_area();
        let (mx, my) = mouse_position();
        if mx < editor_x || mx >= editor_x + width || my < editor_y || my >= editor_y + height {
            return None;
        }
        let gx = ((mx - editor_x) / self.size).floor() as isize;
        let gy = ((my - editor_y) / self.size).floor() as isize;
        Some((gx, gy))
    }

    pub fn handle_input(&mut self) {
        let (editor_x, editor_y, width, height) = self.editor_area();
        let (mx, my) = mouse_position();
        let grid_pos = self.mouse_grid_pos();
        
        // Grid size (zoom) buttons - change visual size of grid cells
        let grid_label_y = self.h - 110.0;
        let grid_btn_y = self.h - 90.0;
        if self.button_clicked(20.0, grid_btn_y, 32.0, 32.0) {
            if self.size > 2.0 {
                self.size -= 1.0;
            }
        }
        if self.button_clicked(60.0, grid_btn_y, 32.0, 32.0) {
            self.size += 1.0;
        }
        
        // Pen size buttons - change number of grid cells affected
        let pen_label_y = self.h - 70.0;
        let pen_btn_y = self.h - 50.0;
        if self.button_clicked(20.0, pen_btn_y, 32.0, 32.0) {
            if self.pen_size > 1 {
                self.pen_size -= 1;
            }
        }
        if self.button_clicked(60.0, pen_btn_y, 32.0, 32.0) {
            self.pen_size += 1;
        }
        
        // Undo/redo buttons
        if self.button_clicked(20.0, 20.0, 32.0, 32.0) {
            self.undo();
        }
        if self.button_clicked(60.0, 20.0, 32.0, 32.0) {
            self.redo();
        }
        // Shape tool buttons
        let shape_btns = [ShapeTool::Pixel, ShapeTool::Line, ShapeTool::Rect, ShapeTool::Circle, ShapeTool::Hex];
        for (i, tool) in shape_btns.iter().enumerate() {
            if self.button_clicked(20.0, 70.0 + i as f32 * 40.0, 32.0, 32.0) {
                self.shape_tool = *tool;
            }
        }
        // Layer buttons (add, delete, select, show/hide)
        let mut layer_y = 250.0;
        let mut select_layer: Option<usize> = None;
        let mut toggle_layer: Option<usize> = None;
        let mut delete_layer: Option<usize> = None;
        for (i, _layer) in self.layers.iter().enumerate() {
            if self.button_clicked(20.0, layer_y, 24.0, 24.0) {
                select_layer = Some(i);
            }
            if self.button_clicked(50.0, layer_y, 24.0, 24.0) {
                toggle_layer = Some(i);
            }
            if self.button_clicked(80.0, layer_y, 24.0, 24.0) && self.layers.len() > 1 {
                delete_layer = Some(i);
                break;
            }
            layer_y += 34.0;
        }
        // Apply layer actions after the loop
        if let Some(i) = select_layer {
            self.selected_layer = i;
        }
        if let Some(i) = toggle_layer {
            self.layers[i].visible = !self.layers[i].visible;
        }
        if let Some(i) = delete_layer {
            self.layers.remove(i);
            if self.selected_layer >= self.layers.len() {
                self.selected_layer = self.layers.len() - 1;
            }
        }
        // Add layer button
        if self.button_clicked(20.0, layer_y, 24.0, 24.0) {
            self.layers.push(Layer::new(format!("Layer {}", self.layers.len() + 1)));
        }
        // Drawing
        match self.shape_tool {
            ShapeTool::Pixel => {
                if is_mouse_button_down(MouseButton::Left) {
                    if let Some((gx, gy)) = grid_pos {
                        for dy in 0..self.pen_size as isize {
                            for dx in 0..self.pen_size as isize {
                                let x = gx + dx;
                                let y = gy + dy;
                                if x >= 0 && y >= 0 && (x as usize) < GRID_WIDTH && (y as usize) < GRID_HEIGHT {
                                    self.layers[self.selected_layer].grid[y as usize][x as usize] = true;
                                }
                            }
                        }
                    }
                }
                self.drag_start = None;
                self.drag_end = None;
            }
            _ => {
                // For shapes, use drag logic
                if is_mouse_button_pressed(MouseButton::Left) {
                    if let Some((gx, gy)) = grid_pos {
                        self.drag_start = Some((gx, gy));
                        self.drag_end = None;
                    }
                }
                if is_mouse_button_down(MouseButton::Left) && self.drag_start.is_some() {
                    if let Some((gx, gy)) = grid_pos {
                        self.drag_end = Some((gx, gy));
                    }
                }
                if is_mouse_button_released(MouseButton::Left) {
                    if let (Some((sx, sy)), Some((ex, ey))) = (self.drag_start, self.drag_end) {
                        match self.shape_tool {
                            ShapeTool::Line => self.draw_line_on_grid(sx, sy, ex, ey),
                            ShapeTool::Rect => self.draw_rect_on_grid(sx, sy, ex, ey),
                            ShapeTool::Circle => self.draw_circle_on_grid(sx, sy, ex, ey),
                            ShapeTool::Hex => self.draw_hex_on_grid(sx, sy, ex, ey),
                            _ => {}
                        }
                    }
                    self.drag_start = None;
                    self.drag_end = None;
                }
            }
        }
    }

    fn button_clicked(&self, x: f32, y: f32, w: f32, h: f32) -> bool {
        let (mx, my) = mouse_position();
        is_mouse_button_pressed(MouseButton::Left)
            && mx >= x && mx <= x + w && my >= y && my <= y + h
    }

    pub fn undo(&mut self) {
        // TODO: implement undo for layers
    }
    pub fn redo(&mut self) {
        // TODO: implement redo for layers
    }

    pub fn draw_blocks(&self) {
        // Draw left panel
        draw_rectangle(0.0, 0.0, self.panel_width, self.h, LIGHTGRAY.with_alpha(0.8));
        // Draw main editor area border
        let editor_x = self.panel_width;
        let editor_y = (self.h - (GRID_HEIGHT as f32 * self.size)) / 2.0;
        draw_rectangle_lines(editor_x, editor_y, GRID_WIDTH as f32 * self.size, GRID_HEIGHT as f32 * self.size, 2.0, RED);
        // Draw background image if loaded
        if let Some(texture) = &self.texture {
            let dest_size = Some(vec2(GRID_WIDTH as f32 * self.size, GRID_HEIGHT as f32 * self.size));
            let params = DrawTextureParams {
                dest_size,
                ..Default::default()
            };
            draw_texture_ex(texture, editor_x, editor_y, self.background_color, params);
        }
        
        // Grid Size section
        let grid_label_y = self.h - 110.0;
        let grid_btn_y = self.h - 90.0;
        draw_text("Grid Size", 20.0, grid_label_y, 16.0, BLACK);
        let grid_text = format!("{}px", self.size as i32);
        draw_text(&grid_text, 100.0, grid_label_y, 16.0, BLACK);
        
        // Grid size buttons
        draw_rectangle_lines(20.0, grid_btn_y, 32.0, 32.0, 2.0, BLACK);
        draw_text("-", 32.0, grid_btn_y + 22.0, 20.0, BLACK);
        draw_rectangle_lines(60.0, grid_btn_y, 32.0, 32.0, 2.0, BLACK);
        draw_text("+", 72.0, grid_btn_y + 22.0, 20.0, BLACK);
        
        // Pen Size section
        let pen_label_y = self.h - 70.0;
        let pen_btn_y = self.h - 50.0;
        draw_text("Pen Size", 20.0, pen_label_y, 16.0, BLACK);
        let pen_text = format!("{}x{}", self.pen_size, self.pen_size);
        draw_text(&pen_text, 100.0, pen_label_y, 16.0, BLACK);
        
        // Pen size buttons
        draw_rectangle_lines(20.0, pen_btn_y, 32.0, 32.0, 2.0, BLACK);
        draw_text("-", 32.0, pen_btn_y + 22.0, 20.0, BLACK);
        draw_rectangle_lines(60.0, pen_btn_y, 32.0, 32.0, 2.0, BLACK);
        draw_text("+", 72.0, pen_btn_y + 22.0, 20.0, BLACK);
        
        // Grid size visual indicator (left - small grid showing cell size)
        let grid_indicator_size = 32.0;
        let grid_indicator_x = 100.0;
        let grid_indicator_y = self.h - 110.0;
        draw_rectangle(grid_indicator_x, grid_indicator_y, grid_indicator_size, grid_indicator_size, WHITE);
        draw_rectangle_lines(grid_indicator_x, grid_indicator_y, grid_indicator_size, grid_indicator_size, 1.0, BLACK);
        
        // Draw grid lines in the indicator to show cell size
        let cell_size_in_indicator = grid_indicator_size / self.size;
        for i in 1..(self.size as i32) {
            let line_pos = i as f32 * cell_size_in_indicator;
            if line_pos < grid_indicator_size {
                draw_line(
                    grid_indicator_x + line_pos, grid_indicator_y,
                    grid_indicator_x + line_pos, grid_indicator_y + grid_indicator_size,
                    1.0, LIGHTGRAY,
                );
                draw_line(
                    grid_indicator_x, grid_indicator_y + line_pos,
                    grid_indicator_x + grid_indicator_size, grid_indicator_y + line_pos,
                    1.0, LIGHTGRAY,
                );
            }
        }
        
        // Pen size visual indicator (right - small rectangle)
        let pen_indicator_size = self.pen_size as f32 * 4.0; // 4 pixels per pen cell for visibility
        let pen_indicator_x = 140.0;
        let pen_indicator_y = self.h - 70.0;
        draw_rectangle(pen_indicator_x, pen_indicator_y, pen_indicator_size, pen_indicator_size, BLACK);
        draw_rectangle_lines(pen_indicator_x, pen_indicator_y, pen_indicator_size, pen_indicator_size, 1.0, RED);
    }

    pub fn draw_shapes(&self) {
        // Draw shape tool buttons
        let shape_btns = [ShapeTool::Pixel, ShapeTool::Line, ShapeTool::Rect, ShapeTool::Circle, ShapeTool::Hex];
        for (i, tool) in shape_btns.iter().enumerate() {
            let y = 70.0 + i as f32 * 40.0;
            let color = if *tool == self.shape_tool { RED } else { BLACK };
            draw_rectangle_lines(20.0, y, 32.0, 32.0, 2.0, color);
            let label = match tool {
                ShapeTool::Pixel => "P",
                ShapeTool::Line => "L",
                ShapeTool::Rect => "R",
                ShapeTool::Circle => "C",
                ShapeTool::Hex => "H",
            };
            draw_text(label, 28.0, y + 24.0, 20.0, color);
        }
    }

    pub fn draw_pixels(&self) {
        let editor_x = self.panel_width;
        let editor_y = (self.h - (GRID_HEIGHT as f32 * self.size)) / 2.0;
        // Draw grid
        for grid_x in 0..=GRID_WIDTH {
            draw_line(
                editor_x + grid_x as f32 * self.size,
                editor_y,
                editor_x + grid_x as f32 * self.size,
                editor_y + GRID_HEIGHT as f32 * self.size,
                1.0,
                LIGHTGRAY.with_alpha(0.5),
            );
        }
        for grid_y in 0..=GRID_HEIGHT {
            draw_line(
                editor_x,
                editor_y + grid_y as f32 * self.size,
                editor_x + GRID_WIDTH as f32 * self.size,
                editor_y + grid_y as f32 * self.size,
                1.0,
                LIGHTGRAY.with_alpha(0.5),
            );
        }
        // Draw all visible layers
        for (i, layer) in self.layers.iter().enumerate() {
            if layer.visible {
                for y in 0..GRID_HEIGHT {
                    for x in 0..GRID_WIDTH {
                        if layer.grid[y][x] {
                            draw_rectangle(
                                editor_x + x as f32 * self.size,
                                editor_y + y as f32 * self.size,
                                self.size,
                                self.size,
                                if i == self.selected_layer { BLACK } else { DARKGRAY },
                            );
                        }
                    }
                }
            }
        }
        self.draw_shape_preview();
    }

    pub fn draw_cursor(&self) {
        // Draw pen preview at mouse position
        let (editor_x, editor_y, width, height) = self.editor_area();
        let (mx, my) = mouse_position();
        let grid_pos = self.mouse_grid_pos();
        
        if let Some((gx, gy)) = grid_pos {
            // Draw cursor preview showing pen size in grid cells
            for dy in 0..self.pen_size as isize {
                for dx in 0..self.pen_size as isize {
                    let px = gx + dx;
                    let py = gy + dy;
                    if px >= 0 && py >= 0 && (px as usize) < GRID_WIDTH && (py as usize) < GRID_HEIGHT {
                        draw_rectangle(
                            editor_x + px as f32 * self.size,
                            editor_y + py as f32 * self.size,
                            self.size,
                            self.size,
                            Color::new(0.0, 0.0, 0.0, 0.2),
                        );
                    }
                }
            }
        }
        
        // Pixel preview in left bottom corner - show pen size in grid cells
        let preview_x = 20.0;
        let preview_y = self.h - 40.0;
        let preview_cell_size = 4.0; // Fixed small size for preview
        let preview_total_size = self.pen_size as f32 * preview_cell_size;
        
        // Draw background for preview
        draw_rectangle(preview_x, preview_y, preview_total_size, preview_total_size, WHITE);
        draw_rectangle_lines(preview_x, preview_y, preview_total_size, preview_total_size, 1.0, BLACK);
        
        // Draw grid lines in preview
        for i in 1..self.pen_size {
            let line_pos = i as f32 * preview_cell_size;
            draw_line(
                preview_x + line_pos, preview_y,
                preview_x + line_pos, preview_y + preview_total_size,
                1.0, LIGHTGRAY,
            );
            draw_line(
                preview_x, preview_y + line_pos,
                preview_x + preview_total_size, preview_y + line_pos,
                1.0, LIGHTGRAY,
            );
        }
    }

    pub fn draw_layers(&self) {
        // Draw layers panel in left panel
        let mut y = 250.0;
        for (i, layer) in self.layers.iter().enumerate() {
            let color = if i == self.selected_layer { RED } else { BLACK };
            draw_rectangle_lines(20.0, y, 24.0, 24.0, 2.0, color);
            draw_text(&layer.name, 50.0, y + 18.0, 16.0, color);
            if !layer.visible {
                draw_text("(H)", 120.0, y + 18.0, 16.0, DARKGRAY);
            }
            y += 34.0;
        }
        // Add layer button
        draw_rectangle_lines(20.0, y, 24.0, 24.0, 2.0, BLACK);
        draw_text("+", 28.0, y + 18.0, 16.0, BLACK);
    }

    fn draw_line_on_grid(&mut self, sx: isize, sy: isize, ex: isize, ey: isize) {
        let dx = (ex - sx).abs();
        let dy = -(ey - sy).abs();
        let sx_step = if sx < ex { 1 } else { -1 };
        let sy_step = if sy < ey { 1 } else { -1 };
        let mut err = dx + dy;
        let (mut x, mut y) = (sx, sy);
        loop {
            // Draw pen-sized pixels at each point
            for dy in 0..self.pen_size as isize {
                for dx in 0..self.pen_size as isize {
                    let px = x + dx;
                    let py = y + dy;
                    if px >= 0 && py >= 0 && (px as usize) < GRID_WIDTH && (py as usize) < GRID_HEIGHT {
                        self.layers[self.selected_layer].grid[py as usize][px as usize] = true;
                    }
                }
            }
            if x == ex && y == ey { break; }
            let e2 = 2 * err;
            if e2 >= dy { err += dy; x += sx_step; }
            if e2 <= dx { err += dx; y += sy_step; }
        }
    }
    fn draw_rect_on_grid(&mut self, sx: isize, sy: isize, ex: isize, ey: isize) {
        let (min_x, max_x) = (sx.min(ex), sx.max(ex));
        let (min_y, max_y) = (sy.min(ey), sy.max(ey));
        for x in min_x..=max_x {
            // Draw pen-sized pixels for top and bottom edges
            for dy in 0..self.pen_size as isize {
                for dx in 0..self.pen_size as isize {
                    let px = x + dx;
                    let py = min_y + dy;
                    if px >= 0 && py >= 0 && (px as usize) < GRID_WIDTH && (py as usize) < GRID_HEIGHT {
                        self.layers[self.selected_layer].grid[py as usize][px as usize] = true;
                    }
                    let py = max_y + dy;
                    if px >= 0 && py >= 0 && (px as usize) < GRID_WIDTH && (py as usize) < GRID_HEIGHT {
                        self.layers[self.selected_layer].grid[py as usize][px as usize] = true;
                    }
                }
            }
        }
        for y in min_y..=max_y {
            // Draw pen-sized pixels for left and right edges
            for dy in 0..self.pen_size as isize {
                for dx in 0..self.pen_size as isize {
                    let px = min_x + dx;
                    let py = y + dy;
                    if px >= 0 && py >= 0 && (px as usize) < GRID_WIDTH && (py as usize) < GRID_HEIGHT {
                        self.layers[self.selected_layer].grid[py as usize][px as usize] = true;
                    }
                    let px = max_x + dx;
                    if px >= 0 && py >= 0 && (px as usize) < GRID_WIDTH && (py as usize) < GRID_HEIGHT {
                        self.layers[self.selected_layer].grid[py as usize][px as usize] = true;
                    }
                }
            }
        }
    }
    fn draw_circle_on_grid(&mut self, sx: isize, sy: isize, ex: isize, ey: isize) {
        let rx = (ex - sx).abs();
        let ry = (ey - sy).abs();
        let r = rx.max(ry);
        let (cx, cy) = (sx, sy);
        let mut x = r;
        let mut y = 0;
        let mut err = 0;
        while x >= y {
            for &(dx, dy) in &[
                (x, y), (y, x), (-y, x), (-x, y),
                (-x, -y), (-y, -x), (y, -x), (x, -y),
            ] {
                let px = cx + dx;
                let py = cy + dy;
                // Draw pen-sized pixels at each circle point
                for pdy in 0..self.pen_size as isize {
                    for pdx in 0..self.pen_size as isize {
                        let final_x = px + pdx;
                        let final_y = py + pdy;
                        if final_x >= 0 && final_y >= 0 && (final_x as usize) < GRID_WIDTH && (final_y as usize) < GRID_HEIGHT {
                            self.layers[self.selected_layer].grid[final_y as usize][final_x as usize] = true;
                        }
                    }
                }
            }
            y += 1;
            if err <= 0 {
                err += 2 * y + 1;
            } else {
                x -= 1;
                err -= 2 * x + 1;
            }
        }
    }
    fn draw_hex_on_grid(&mut self, sx: isize, sy: isize, ex: isize, ey: isize) {
        // Draw a regular hexagon inscribed in the rectangle from (sx,sy) to (ex,ey)
        let (cx, cy) = (sx, sy);
        let rx = (ex - sx).abs() as f32;
        let ry = (ey - sy).abs() as f32;
        let r = rx.max(ry) / 2.0;
        let angle_step = std::f32::consts::PI / 3.0;
        let mut points = vec![];
        for i in 0..6 {
            let angle = i as f32 * angle_step;
            let px = cx as f32 + r * angle.cos();
            let py = cy as f32 + r * angle.sin();
            points.push((px, py));
        }
        // Draw lines between points using pen size
        for i in 0..6 {
            let (x0, y0) = points[i];
            let (x1, y1) = points[(i + 1) % 6];
            self.draw_line_on_grid(x0.round() as isize, y0.round() as isize, x1.round() as isize, y1.round() as isize);
        }
    }

    pub fn draw_shape_preview(&self) {
        let editor_x = self.panel_width;
        let editor_y = (self.h - (GRID_HEIGHT as f32 * self.size)) / 2.0;
        if let (Some((sx, sy)), Some((ex, ey))) = (self.drag_start, self.drag_end) {
            let mut preview_pixels = vec![];
            match self.shape_tool {
                ShapeTool::Line => {
                    self.collect_line_pixels(sx, sy, ex, ey, &mut preview_pixels);
                }
                ShapeTool::Rect => {
                    self.collect_rect_pixels(sx, sy, ex, ey, &mut preview_pixels);
                }
                ShapeTool::Circle => {
                    self.collect_circle_pixels(sx, sy, ex, ey, &mut preview_pixels);
                }
                ShapeTool::Hex => {
                    self.collect_hex_pixels(sx, sy, ex, ey, &mut preview_pixels);
                }
                _ => {}
            }
            // For each preview pixel, draw a semi-transparent rectangle using pen size
            for &(x, y) in &preview_pixels {
                for dy in 0..self.pen_size as isize {
                    for dx in 0..self.pen_size as isize {
                        let px = x + dx;
                        let py = y + dy;
                        if px >= 0 && py >= 0 && (px as usize) < GRID_WIDTH && (py as usize) < GRID_HEIGHT {
                            draw_rectangle(
                                editor_x + px as f32 * self.size,
                                editor_y + py as f32 * self.size,
                                self.size,
                                self.size,
                                Color::new(1.0, 0.0, 0.0, 0.3),
                            );
                        }
                    }
                }
            }
        }
    }

    // --- Collect preview pixels for each shape ---
    fn collect_line_pixels(&self, sx: isize, sy: isize, ex: isize, ey: isize, out: &mut Vec<(isize, isize)>) {
        let dx = (ex - sx).abs();
        let dy = -(ey - sy).abs();
        let sx_step = if sx < ex { 1 } else { -1 };
        let sy_step = if sy < ey { 1 } else { -1 };
        let mut err = dx + dy;
        let (mut x, mut y) = (sx, sy);
        loop {
            out.push((x, y));
            if x == ex && y == ey { break; }
            let e2 = 2 * err;
            if e2 >= dy { err += dy; x += sx_step; }
            if e2 <= dx { err += dx; y += sy_step; }
        }
    }
    fn collect_rect_pixels(&self, sx: isize, sy: isize, ex: isize, ey: isize, out: &mut Vec<(isize, isize)>) {
        let (min_x, max_x) = (sx.min(ex), sx.max(ex));
        let (min_y, max_y) = (sy.min(ey), sy.max(ey));
        for x in min_x..=max_x {
            out.push((x, min_y));
            out.push((x, max_y));
        }
        for y in min_y..=max_y {
            out.push((min_x, y));
            out.push((max_x, y));
        }
    }
    fn collect_circle_pixels(&self, sx: isize, sy: isize, ex: isize, ey: isize, out: &mut Vec<(isize, isize)>) {
        let rx = (ex - sx).abs();
        let ry = (ey - sy).abs();
        let r = rx.max(ry);
        let (cx, cy) = (sx, sy);
        let mut x = r;
        let mut y = 0;
        let mut err = 0;
        while x >= y {
            for &(dx, dy) in &[
                (x, y), (y, x), (-y, x), (-x, y),
                (-x, -y), (-y, -x), (y, -x), (x, -y),
            ] {
                out.push((cx + dx, cy + dy));
            }
            y += 1;
            if err <= 0 {
                err += 2 * y + 1;
            } else {
                x -= 1;
                err -= 2 * x + 1;
            }
        }
    }
    fn collect_hex_pixels(&self, sx: isize, sy: isize, ex: isize, ey: isize, out: &mut Vec<(isize, isize)>) {
        let (cx, cy) = (sx, sy);
        let rx = (ex - sx).abs() as f32;
        let ry = (ey - sy).abs() as f32;
        let r = rx.max(ry) / 2.0;
        let angle_step = std::f32::consts::PI / 3.0;
        let mut points = vec![];
        for i in 0..6 {
            let angle = i as f32 * angle_step;
            let px = cx as f32 + r * angle.cos();
            let py = cy as f32 + r * angle.sin();
            points.push((px, py));
        }
        for i in 0..6 {
            let (x0, y0) = points[i];
            let (x1, y1) = points[(i + 1) % 6];
            self.collect_line_pixels(x0.round() as isize, y0.round() as isize, x1.round() as isize, y1.round() as isize, out);
        }
    }
} 