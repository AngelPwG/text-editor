use termios::*;

pub struct Terminal {
    original_termios: Termios,
    raw_termios: Termios,
}
pub enum Key {
    Char(u8),
    ArrowUp,
    ArrowDown,
    ArrowRight,
    Backspace,
    Enter,
    Ctrl(u8),
    Escape,
}
impl Terminal {
    pub fn new() -> Self {
        let mut original_termios = Termios::from_fd(0).unwrap();
        let mut new_raw = original_termios.clone();
        Termios::cfmakeraw(&mut new_raw);
        Terminal {original_termios: original_termios, raw_termios: new_raw }
    }
    pub fn enable_raw_mode(&self) -> Result<(), std::io::Error> {
        tcsetattr(STDIN_FILENO, TCSANOW, &self.raw_termios)?;
        Ok(())
    } 
    pub fn disable_raw_mode(&self) -> Result<(), std::io::Error>{
        tcsetattr(STDIN_FILENO, TCSANOW, &self.original_termios)?;
        Ok(())
    }
    pub fn read_key(&self) -> Result<Key, std::io::Error>{
        
    }
}
