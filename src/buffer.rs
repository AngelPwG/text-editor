use std::slice::ChunksExact;

pub struct GapBuffer {
    buffer: Vec<u8>,
    gap_start: usize,
    gap_end: usize,
}
impl GapBuffer {
    pub fn new() -> Self {
        let new_buffer = vec![0u8; 100];
        GapBuffer {
            buffer: new_buffer,
            gap_start: 0,
            gap_end: 99,
        }
    }
    pub fn insert(&mut self, c: u8) {
        self.buffer[self.gap_start] = c;
        self.gap_start += 1;
        if self.gap_start > self.gap_end {
            let new_gap = vec![0u8; 100];
            self.buffer.splice(self.gap_end..self.gap_end, new_gap);
            self.gap_end = self.gap_start + 99;
        }
    }
    pub fn move_left(&mut self) {
        if self.gap_start == 0 {
            return;
        }
        self.buffer[self.gap_end] = self.buffer[self.gap_start - 1];
        self.gap_start -= 1;
        self.gap_end -= 1;
    }
    pub fn move_right(&mut self) {
        if self.gap_end == self.buffer.len() - 1 {
            return;
        }
        self.buffer[self.gap_start] = self.buffer[self.gap_end + 1];
        self.gap_start += 1;
        self.gap_end += 1;
    }
    pub fn delete(&mut self) {
        if self.gap_start == 0 {
            return;
        }
        self.gap_start -= 1;
    }
    pub fn recalc_cursor(&mut self) -> (usize, usize) {
        let mut cursor_x = 1;
        let mut cursor_y = 1;
        let iter = self.buffer[..self.gap_start].iter().enumerate();
        for (i, &byte) in iter {
            if byte == b'\n' {
                cursor_x = 1;
                cursor_y += 1;
            } else {
                cursor_x += 1;
            }
        }
        (cursor_x, cursor_y)
    }
    pub fn xy_to_index(&self, x: usize, y: usize) -> usize {
        let mut cursor_x: usize = 1;
        let mut cursor_y: usize = 1;
        let iter = self.buffer[..self.gap_start].iter().enumerate();
        for (i, &byte) in iter {
            if byte == b'\n' {
                cursor_x = 1;
                if cursor_y == y {
                    return i;
                }
                cursor_y = cursor_y.saturating_add(1);
            } else {
                cursor_x = cursor_x.saturating_add(1);
            }
            if cursor_x == x && cursor_y == y {
                return i;
            }
        }
        if self.gap_end + 1 == self.buffer.len() {
            return self.gap_start - 1;
        }
        let iter2 = self.buffer[self.gap_end + 1..].iter().enumerate();
        let mut it: usize = 0;
        for (i, &byte) in iter2 {
            if byte == b'\n' {
                cursor_x = 1;
                if cursor_y == y {
                    return self.gap_start + i;
                }
                cursor_y = cursor_y.saturating_add(1);
            } else {
                cursor_x = cursor_x.saturating_add(1);
            }
            if cursor_x == x && cursor_y == y {
                return self.gap_start + i;
            }
            it = i;
        }
        self.gap_start + it
    }
    pub fn move_to(&mut self, index: usize) {
        if index >= self.buffer.len() - (self.gap_end - self.gap_start + 1) {
            return;
        }
        if index < self.gap_start {
            while self.gap_start != index {
                self.move_left();
            }
        } else {
            while self.gap_start != index {
                self.move_right();
            }
        }
    }
    pub fn lines(&self) -> Vec<Vec<u8>> {
        let mut lines = Vec::new();
        let mut current_line = Vec::new();

        if self.gap_end == self.buffer.len() - 1 {
            let bytes_iter = self.buffer[..self.gap_start].iter();
            for &byte in bytes_iter {
                if byte == b'\n' {
                    lines.push(current_line);
                    current_line = Vec::new();
                } else {
                    current_line.push(byte);
                }
            }
            if !current_line.is_empty() {
                lines.push(current_line);
            }
        } else {
            let bytes_iter = self.buffer[..self.gap_start]
                .iter()
                .chain(self.buffer[self.gap_end + 1..].iter());
            for &byte in bytes_iter {
                if byte == b'\n' {
                    lines.push(current_line);
                    current_line = Vec::new();
                } else {
                    current_line.push(byte);
                }
            }
            if !current_line.is_empty() {
                lines.push(current_line);
            }
        }
        lines
    }
}
