use crate::buffer::GapBuffer;
use crate::terminal::{Key, Terminal};
use std::io::Write;

pub enum Mode {
    Normal,
    Command,
    Insert,
    SaveAs(bool),
}

pub struct Editor {
    filename: String,
    buffer: GapBuffer,
    terminal: Terminal,
    mode: Mode,
    command_buffer: Vec<u8>,
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
            mode: Mode::Normal,
            command_buffer: Vec::new(),
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
        if !self.filename.is_empty() {
            self.open();
        }
        _ = self.terminal.enable_raw_mode();
        loop {
            self.recalc_cursor();
            self.update_scroll();
            self.render();
            if let Some(key) = self.terminal.read_key().ok() {
                match self.mode {
                    Mode::Insert => self.process_insert(key),
                    Mode::Command => {
                        if self.process_command(key) {
                            break;
                        }
                    }
                    Mode::Normal => self.process_normal(key),
                    Mode::SaveAs(quit) => {
                        if self.process_save_as(key, quit) {
                            break;
                        }
                    }
                }
            }
        }
        print!("\x1B[2J\x1B[1;1H");
        _ = self.terminal.disable_raw_mode();
    }

    pub fn process_save_as(&mut self, key: Key, quit: bool) -> bool {
        match key {
            Key::Enter => {
                self.filename = String::from_utf8_lossy(&self.command_buffer)
                    .trim()
                    .to_string();
                self.save();
                self.was_changed = false;
                self.command_buffer.clear();
                self.mode = Mode::Normal;
                quit
            }
            Key::Char(c) => {
                self.command_buffer.push(c);
                false
            }
            Key::Escape => {
                self.mode = Mode::Normal;
                self.command_buffer.clear();
                false
            }
            Key::Backspace => {
                if self.command_buffer.is_empty() {
                    return false;
                }
                self.command_buffer.pop();
                false
            }
            _ => false,
        }
    }

    pub fn execute_command(&mut self) -> bool {
        match String::from_utf8_lossy(&self.command_buffer).trim() {
            "w" => {
                if self.filename.is_empty() {
                    self.mode = Mode::SaveAs(false);
                    self.command_buffer.clear();
                } else {
                    self.save();
                    self.was_changed = false;
                }
                false
            }
            "q" => true,
            "wq" => {
                if self.filename.is_empty() {
                    self.mode = Mode::SaveAs(true);
                    self.command_buffer.clear();
                    return false;
                } else {
                    self.save();
                    self.was_changed = false;
                }
                true
            }
            _ => false,
        }
    }

    pub fn process_command(&mut self, key: Key) -> bool {
        match key {
            Key::Enter => {
                if self.execute_command() {
                    return true;
                }
                self.command_buffer.clear();
            }
            Key::Backspace => {
                self.command_buffer.pop();
            }
            Key::Char(b) => {
                self.command_buffer.push(b);
            }
            Key::Mouse(_, _) => {
                self.mode = Mode::Normal;
                self.command_buffer.clear();
            }

            _ => (),
        }
        false
    }

    pub fn process_normal(&mut self, key: Key) {
        match key {
            Key::Char(b'h') => self.buffer.move_left(),
            Key::Char(b'l') => self.buffer.move_right(),
            Key::Char(b'k') => {
                let index = self
                    .buffer
                    .xy_to_index(self.cursor_x + 1, self.cursor_y.saturating_sub(1));
                self.buffer.move_to(index);
            }
            Key::Char(b'j') => {
                let index = self
                    .buffer
                    .xy_to_index(self.cursor_x + 1, self.cursor_y.saturating_add(1));
                self.buffer.move_to(index);
            }
            Key::Char(b'i') => {
                self.mode = Mode::Insert;
            }
            Key::Char(b'o') => {
                let index = self.buffer.xy_to_index(1, self.cursor_y.saturating_add(1));
                self.buffer.move_to(index);
                self.buffer.insert(b'\n');
                self.mode = Mode::Insert;
            }
            Key::Char(b':') => {
                self.mode = Mode::Command;
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
                let index = self
                    .buffer
                    .xy_to_index(x.saturating_sub(self.gutter_width) + 1, y + self.scroll_y);
                self.buffer.move_to(index);
            }
            _ => {}
        }
    }
    pub fn process_insert(&mut self, key: Key) {
        match key {
            Key::Ctrl(b'c') => {
                self.mode = Mode::Normal;
                self.command_buffer.clear();
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
                let index = self
                    .buffer
                    .xy_to_index(x.saturating_sub(self.gutter_width) + 1, y + self.scroll_y);
                self.buffer.move_to(index);
            }
            _ => {}
        }
    }

    pub fn render(&mut self) {
        let mut screen = String::new();
        print!("\x1B[2J\x1B[1;1H");

        let lines = self.buffer.lines();
        let end = (self.scroll_y + self.height).min(lines.len());
        for (i, line) in lines[self.scroll_y..end].iter().enumerate() {
            screen.push_str(&format!(
                "{:>width$}| {}\r\n",
                i + 1 + self.scroll_y,
                String::from_utf8_lossy(&line[..line.len().min(self.width - 5)]),
                width = self.gutter_width - 2
            ));
        }
        println!("{}", screen);
        match self.mode {
            Mode::Normal => print!("{} --NORMAL", String::from_utf8_lossy(&self.command_buffer)),
            Mode::Command => print!(
                ":{} --COMMAND",
                String::from_utf8_lossy(&self.command_buffer)
            ),
            Mode::Insert => print!(" --INSERT"),
            Mode::SaveAs(_) => print!("Save As: {}", String::from_utf8_lossy(&self.command_buffer)),
        }
        print!("  {}", self.filename);
        print!(
            "\x1B[{};{}H",
            self.cursor_y - self.scroll_y,
            self.cursor_x + self.gutter_width
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
