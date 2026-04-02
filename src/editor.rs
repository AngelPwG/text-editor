use crate::buffer::GapBuffer;
use crate::terminal::{Key, Terminal};
use std::io::Write;

pub struct Editor {
    buffer: GapBuffer,
    terminal: Terminal,
    width: usize,
    height: usize,
    cursor_x: usize,
    cursor_y: usize,
}

impl Editor {
    pub fn new() -> Self {
        let terminal = Terminal::new();
        let (width, height) = terminal.get_size().unwrap_or((80, 24));
        Self {
            buffer: GapBuffer::new(),
            terminal,
            width,
            height,
            cursor_x: 1,
            cursor_y: 1,
        }
    }

    pub fn run(&mut self) {
        _ = self.terminal.enable_raw_mode();
        loop {
            self.render();
            if let Some(key) = self.terminal.read_key().ok() {
                match key {
                    Key::Ctrl(b'q') => break,
                    Key::Char(b) => {
                        self.buffer.insert(b);
                    }
                    Key::Backspace => {
                        self.buffer.delete();
                    }
                    Key::Enter => {
                        self.buffer.insert(b'\n');
                    }
                    Key::ArrowUp => self.cursor_y = self.cursor_y.saturating_sub(1),
                    Key::ArrowDown => self.cursor_y = self.cursor_y.saturating_add(1),
                    Key::ArrowLeft => self.buffer.move_left(),
                    Key::ArrowRight => self.buffer.move_right(),
                    Key::Mouse(x, y) => {
                        let index = self.buffer.xy_to_index(x, y);
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
        self.recalc_cursor();
        let lines = self.buffer.lines();
        for line in lines {
            screen.push_str(&format!("{}\r\n", String::from_utf8_lossy(&line)));
        }
        print!("{}", screen);
        print!("\x1B[{};{}H", self.cursor_y, self.cursor_x);
        std::io::stdout().flush().unwrap();
    }

    fn recalc_cursor(&mut self) {
        let (cursor_x, cursor_y) = self.buffer.recalc_cursor();
        self.cursor_x = cursor_x;
        self.cursor_y = cursor_y;
    }
}
