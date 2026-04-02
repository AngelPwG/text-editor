use crate::buffer::GapBuffer;
use crate::terminal::{Key, Terminal};
use std::io::Write;

pub struct Editor {
    filename: String,
    buffer: GapBuffer,
    terminal: Terminal,
    width: usize,
    height: usize,
    cursor_x: usize,
    cursor_y: usize,
    scroll_y: usize,
    gutter_width: usize,
    was_changed: bool,
}

impl Editor {
    pub fn new(filename: String) -> Self {
        let terminal = Terminal::new();
        let (width, height) = terminal.get_size().unwrap_or((80, 24));
        Self {
            filename,
            buffer: GapBuffer::new(),
            terminal,
            width,
            height,
            cursor_x: 1,
            cursor_y: 1,
            scroll_y: 0,
            gutter_width: 2,
            was_changed: false,
        }
    }

    pub fn run(&mut self) {
        self.open();
        _ = self.terminal.enable_raw_mode();
        loop {
            self.recalc_cursor();
            self.update_scroll();
            self.render();
            if let Some(key) = self.terminal.read_key().ok() {
                match key {
                    Key::Ctrl(b'q') => break,
                    Key::Ctrl(b's') => {
                        if self.was_changed {
                            self.save();
                            self.was_changed = false;
                        }
                    }
                    Key::Char(b) => {
                        self.buffer.insert(b);
                        self.was_changed = true;
                    }
                    Key::Backspace => {
                        self.buffer.delete();
                        self.was_changed = true;
                    }
                    Key::Enter => {
                        self.buffer.insert(b'\n');
                        self.was_changed = true;
                    }
                    Key::ArrowUp => {
                        if self.cursor_y != 1 {
                            let index = self
                                .buffer
                                .xy_to_index(self.cursor_x + 1, self.cursor_y.saturating_sub(1));
                            self.buffer.move_to(index);
                        }
                    }
                    Key::ArrowDown => {
                        let index = self
                            .buffer
                            .xy_to_index(self.cursor_x + 1, self.cursor_y.saturating_add(1));
                        self.buffer.move_to(index);
                    }
                    Key::ArrowLeft => self.buffer.move_left(),
                    Key::ArrowRight => self.buffer.move_right(),
                    Key::Mouse(x, y) => {
                        let index = self.buffer.xy_to_index(
                            x.saturating_sub(self.gutter_width) + 1,
                            y + self.scroll_y,
                        );
                        self.buffer.move_to(index);
                    }
                    _ => {}
                }
            }
        }
        print!("\x1B[2J\x1B[1;1H");
        _ = self.terminal.disable_raw_mode();
    }

    pub fn render(&mut self) {
        let mut screen = String::new();
        print!("\x1B[2J\x1B[1;1H");

        let lines = self.buffer.lines();
        let max_line = lines.len();
        let gutter = max_line.to_string().len() + 2;
        let end = (self.scroll_y + self.height).min(lines.len());
        for (i, line) in lines[self.scroll_y..end].iter().enumerate() {
            screen.push_str(&format!(
                "{:>width$}| {}\r\n",
                i + 1 + self.scroll_y,
                String::from_utf8_lossy(&line[..line.len().min(self.width - 5)]),
                width = gutter - 2
            ));
        }
        print!("{}", screen);
        print!(
            "\x1B[{};{}H",
            self.cursor_y - self.scroll_y,
            self.cursor_x + gutter
        );
        std::io::stdout().flush().unwrap();
    }
    fn update_scroll(&mut self) {
        if self.cursor_y >= self.scroll_y + self.height {
            self.scroll_y += 1;
        } else if self.cursor_y <= self.scroll_y {
            self.scroll_y = self.scroll_y.saturating_sub(1);
        }
    }

    fn recalc_cursor(&mut self) {
        self.gutter_width = self.buffer.lines().len().to_string().len() + 2;
        let (cursor_x, cursor_y) = self.buffer.recalc_cursor();
        self.cursor_x = cursor_x;
        self.cursor_y = cursor_y;
    }

    fn open(&mut self) {
        self.buffer
            .load(std::fs::read(&self.filename).unwrap_or_default());
        self.recalc_cursor();
    }
    fn save(&mut self) {
        std::fs::write(&self.filename, self.buffer.to_bytes()).unwrap();
    }
}
