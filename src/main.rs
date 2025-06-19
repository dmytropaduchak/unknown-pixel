mod config;
use config::*;

mod editor;
use editor::*;

#[macroquad::main(default)]
async fn main() {
    let mut editor = Editor::new();
    editor.run().await;
}
