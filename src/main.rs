mod buffer;
mod editor;
mod terminal;
use editor::Editor;
fn main() {
    Editor::new().run();
}
