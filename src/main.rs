mod editor;
use editor::*;

#[macroquad::main(editor_config)]
async fn main() {
    let mut editor = Editor::new();
    editor.run().await;
}
