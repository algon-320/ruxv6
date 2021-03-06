use super::x86;

// from: https://os.phil-opp.com/vga-text-mode/

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    pub const fn new(fg: Color, bg: Color) -> Self {
        ColorCode((bg as u8) << 4 | (fg as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenCell {
    ascii: u8,
    color: ColorCode,
}

const HEIGHT: usize = 25;
const WIDTH: usize = 80;
const CRTPORT: u16 = 0x3D4;

#[repr(transparent)]
struct Buffer {
    cells: [[ScreenCell; WIDTH]; HEIGHT],
}

impl Buffer {
    fn write(&mut self, row: usize, col: usize, cell: ScreenCell) {
        let ptr = &mut self.cells[row][col] as *mut ScreenCell;
        unsafe {
            core::ptr::write_volatile(ptr, cell);
        }
    }
    fn read(&self, row: usize, col: usize) -> ScreenCell {
        let ptr = &self.cells[row][col] as *const ScreenCell;
        unsafe { core::ptr::read_volatile(ptr) }
    }
}

use core::fmt;

pub struct Writer {
    column_position: usize,
    row_position: usize,
    color: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= WIDTH {
                    self.new_line();
                }
                self.buffer.write(
                    self.row_position,
                    self.column_position,
                    ScreenCell {
                        ascii: byte,
                        color: self.color,
                    },
                );
                self.column_position += 1;
                self.update_cursor();
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7E | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xFE),
            }
        }
    }

    pub fn clear_screen(&mut self) {
        for r in 0..HEIGHT {
            self.clear_row(r);
        }
    }

    fn new_line(&mut self) {
        if self.row_position == HEIGHT - 1 {
            // scroll rows
            for row in 1..HEIGHT {
                for col in 0..WIDTH {
                    let cell = self.buffer.read(row, col);
                    self.buffer.write(row - 1, col, cell);
                }
            }
            self.clear_row(HEIGHT - 1);
        } else {
            self.row_position += 1;
        }
        self.column_position = 0;
        self.update_cursor();
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenCell {
            ascii: b' ',
            color: self.color,
        };
        for col in 0..WIDTH {
            self.buffer.write(row, col, blank);
        }
    }

    fn update_cursor(&self) {
        // from https://wiki.osdev.org/Text_Mode_Cursor
        let pos = self.row_position * WIDTH + self.column_position;
        x86::outb(CRTPORT + 0, trunc8!(0x0F));
        x86::outb(CRTPORT + 1, trunc8!(pos & 0xFF));
        x86::outb(CRTPORT + 0, trunc8!(0x0E));
        x86::outb(CRTPORT + 1, trunc8!((pos >> 8) & 0xFF));
    }

    fn change_color(&mut self, color: ColorCode) {
        self.color = color;
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

use core::fmt::Write;
use spin::Mutex;

pub const DEFAULT_COLOR: ColorCode = ColorCode::new(Color::White, Color::Black);
pub const ERROR_COLOR: ColorCode = ColorCode::new(Color::LightRed, Color::Black);
pub const WARNING_COLOR: ColorCode = ColorCode::new(Color::Yellow, Color::Black);
pub const INFO_COLOR: ColorCode = ColorCode::new(Color::LightCyan, Color::Black);
pub const DEBUG_COLOR: ColorCode = ColorCode::new(Color::LightGreen, Color::Black);

lazy_static! {
    pub static ref VGA_WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        row_position: 0,
        color: DEFAULT_COLOR,
        buffer: unsafe { &mut *(0x800B8000 as *mut Buffer) },
    });
}

#[macro_export]
macro_rules! print {
    ($color:expr;$($arg:tt)*) => (crate::vga_buffer::_print_with_color($color, format_args!($($arg)*)));
    ($($arg:tt)*) => (crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => (crate::print!("\n"));
    ($color:expr;$($arg:tt)*) => (crate::print!($color;"{}\n", format_args!($($arg)*)));
    ($($arg:tt)*) => (crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! debug_println {
    () => (crate::print!("\n"));
    ($($arg:tt)*) => (crate::print!(crate::vga_buffer::DEBUG_COLOR;"{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    let mut writer = VGA_WRITER.lock();
    writer.write_fmt(args).unwrap();
}
#[doc(hidden)]
pub fn _print_with_color(color: ColorCode, args: fmt::Arguments) {
    let mut writer = VGA_WRITER.lock();
    writer.change_color(color);
    writer.write_fmt(args).unwrap();
    writer.change_color(DEFAULT_COLOR);
}
