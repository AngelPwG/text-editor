mod buffer;
mod editor;
mod terminal;
use editor::Editor;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let filename: String = args.get(1).cloned().unwrap_or_default();
    Editor::new(filename).run();
}
