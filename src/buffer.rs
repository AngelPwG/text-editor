pub struct GapBuffer {
    buffer: Vec<u8>,
    gap_start: usize,
    gap_end: usize,
}
impl GapBuffer {
    pub fn new() -> Self{
        let new_buffer = vec![0u8; 100];
        GapBuffer { buffer: new_buffer, gap_start: 0, gap_end: 99 }
    }
    pub fn insert(&mut self, c: u8) {
        self.buffer[self.gap_start] = c;
        self.gap_start += 1;
        if self.gap_start > self.gap_end {
            let new_gap = vec![0u8; 100];
            self.buffer.splice(self.gap_end..self.gap_end, new_gap);
            self.gap_end = self.gap_start + 100;
        }
    }
    pub fn move_left(&mut self) {
        if self.gap_start == 0 {
            return
        }
        self.buffer[self.gap_end] = self.buffer[self.gap_start - 1];
        self.gap_start -= 1;
        self.gap_end -= 1;
    }
    pub fn move_right(&mut self) {
        if self.gap_end == self.buffer.len() - 1 {
            return
        }
        self.buffer[self.gap_start] = self.buffer[self.gap_end + 1];
        self.gap_start += 1;
        self.gap_end += 1;
    }
    pub fn delete(&mut self) {
        if self.gap_start == 0{
            return
        }
        self.gap_start -= 1;
    }
    pub fn cursor_position(&self) -> u8 {
        self.gap_start
    }
    pub fn move_to(&mut self, index: usize){
        if index >= self.buffer.len() - (self.gap_end - self.gap_start + 1) {
            return
        }
        if index < self.gap_start{
            while self.gap_start != index {
                self.move_left();
            }
        }else{
            while self.gap_start != index {
                self.move_right();
            }
        }
    }
    pub fn get_text(&self) -> Vec<u8> {
        [&self.buffer[..self.gap_start], &self.buffer[self.gap_end + 1..]].concat()
    }
}
