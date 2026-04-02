use libc::STDIN_FILENO;
use std::io::{Read, Write};
use terminal_size::{Height, Width, terminal_size};
use termios::*;

pub struct Terminal {
    original_termios: Termios,
    raw_termios: Termios,
    stdin: std::io::Stdin,
}

pub enum Key {
    Char(u8),
    ArrowUp,
    ArrowDown,
    ArrowRight,
    ArrowLeft,
    Backspace,
    Enter,
    Ctrl(u8),
    Escape,
    Mouse(usize, usize),
    Unknown(u8),
}

impl Terminal {
    pub fn new() -> Self {
        let original_termios = Termios::from_fd(STDIN_FILENO).unwrap();
        let mut new_raw = original_termios.clone();
        cfmakeraw(&mut new_raw);
        Terminal {
            original_termios,
            raw_termios: new_raw,
            stdin: std::io::stdin(),
        }
    }

    pub fn enable_raw_mode(&self) -> Result<(), std::io::Error> {
        tcsetattr(STDIN_FILENO, TCSANOW, &self.raw_termios)?;
        print!("\x1B[?1000h");
        std::io::stdout().flush()?;
        Ok(())
    }

    pub fn disable_raw_mode(&self) -> Result<(), std::io::Error> {
        tcsetattr(STDIN_FILENO, TCSANOW, &self.original_termios)?;
        print!("\x1B[?1000l");
        std::io::stdout().flush()?;
        Ok(())
    }

    pub fn read_key(&mut self) -> Result<Key, std::io::Error> {
        let mut buf = [0u8; 1];
        if self.stdin.read(&mut buf)? == 0 {
            return Ok(Key::Unknown(0));
        }
        if buf[0] == 27 {
            let mut buf_arrows = [0u8; 2];
            if self.stdin.read(&mut buf_arrows)? == 0 {
                return Ok(Key::Escape);
            }
            match buf_arrows {
                [91, 65] => return Ok(Key::ArrowUp),
                [91, 66] => return Ok(Key::ArrowDown),
                [91, 67] => return Ok(Key::ArrowRight),
                [91, 68] => return Ok(Key::ArrowLeft),
                [91, 77] => {
                    let mut buf_mouse = [0u8; 3];
                    if self.stdin.read(&mut buf_mouse)? == 0 {
                        return Ok(Key::Unknown(0));
                    }
                    return Ok(Key::Mouse(
                        (buf_mouse[1] as usize).saturating_sub(32),
                        (buf_mouse[2] as usize).saturating_sub(32),
                    ));
                }
                _ => return Ok(Key::Unknown(buf_arrows[1])),
            }
        } else if buf[0] == 13 {
            return Ok(Key::Enter);
        } else if buf[0] == 127 {
            return Ok(Key::Backspace);
        } else if buf[0] < 32 {
            return Ok(Key::Ctrl(buf[0] + 96));
        }
        Ok(Key::Char(buf[0]))
    }
    pub fn get_size(&self) -> Option<(usize, usize)> {
        if let Some((Width(w), Height(h))) = terminal_size() {
            Some((w as usize, h as usize))
        } else {
            None
        }
    }
}
