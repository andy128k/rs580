use std::{thread, time};
use std::convert::TryFrom;
use termion::{clear, cursor, async_stdin};
use termion::raw::IntoRawMode;
use termion::raw::RawTerminal;
use termion::input::TermRead;
use termion::event::Key;
use termion::screen::AlternateScreen;
use std::io::{Write, stdout, Stdout};
use std::sync::Arc;
use std::cell::{RefCell, Cell};

const ROM: [u8; 2048] = *include_bytes!("./RK86-16.rom");
// const ZG: [u8; 2048] = *include_bytes!("./zg.rom");

struct RKDisplay {
    data: [u8; 78 * 30],
    cursor_x: u8,
    cursor_y: u8,
    last_print: time::Instant,
    dirty: bool,
    stdout: AlternateScreen<RawTerminal<Stdout>>,
    indicators: u8,
}

impl RKDisplay {
    pub fn new() -> Result<Self, std::io::Error> {
        // Enter raw mode.
        let stdout = stdout().into_raw_mode()?;
        let stdout = AlternateScreen::from(stdout);
        Ok(Self {
            data: [0; 78 * 30],
            cursor_x: 0,
            cursor_y: 0,
            indicators: 0,
            last_print: time::Instant::now(),
            dirty: true,
            stdout,
        })
    }

    pub fn copy_from_machine(&mut self, machine: &rs580::Machine) {
        // 0x37c2
        let video_ram = machine.memory.get_range(0x36d0, 0x3ff4);
        if &video_ram[..] != &self.data[..] {
            self.data.copy_from_slice(&video_ram);
            self.dirty = true;
        }
        if self.cursor_x != machine.memory.get_u8(0x3602) {
            self.cursor_x = machine.memory.get_u8(0x3602);
            self.dirty = true;
        }
        if self.cursor_y != machine.memory.get_u8(0x3603) {
            self.cursor_y = machine.memory.get_u8(0x3603);
            self.dirty = true;
        }
    }

    pub fn copy_from_keyboard(&mut self, keyboard: &RKKeyboard) {
        if self.indicators != keyboard.get_indicators() {
            self.indicators = keyboard.get_indicators();
            self.dirty = true;
        }
    }

    pub fn print(&mut self) -> Result<(), std::io::Error> {
        let now = time::Instant::now();
        if (now - self.last_print).subsec_millis() % 25 != 0 {
            return Ok(());
        }
        self.last_print = now;

        if self.dirty {
            self.dirty = false;

            write!(self.stdout, "{}{}{}", clear::All, cursor::Hide, cursor::Goto(1, 1))?;

            write!(self.stdout, "+")?;
            for _ in 0..78 {
                write!(self.stdout, "-")?;
            }
            write!(self.stdout, "+\r\n")?;

            for y in 0..30 {
                write!(self.stdout, "|")?;
                for x in 0..78 {
                    let b: u8 = self.data[y * 78 + x];
                    if b == 0 {
                        write!(self.stdout, " ")?;
                    } else if b >= 32 && b < 128 {
                        write!(self.stdout, "{}", b as char)?;
                    } else {
                        write!(self.stdout, "?")?;
                        // write!(self.stdout, "{}", '\u{1f34c}')?;
                    }
                }
                write!(self.stdout, "|\r\n")?;
            }

            write!(self.stdout, "+")?;
            for _ in 0..78 {
                write!(self.stdout, "-")?;
            }
            write!(self.stdout, "+\r\n")?;
            write!(self.stdout, "{:04b}", self.indicators)?;

            write!(self.stdout, "{}{}", cursor::Show, cursor::Goto(self.cursor_x as u16 + 2, self.cursor_y as u16 + 2))?;

            self.stdout.flush()?;
        }

        Ok(())
    }
}

impl std::ops::Drop for RKDisplay {
    fn drop(&mut self) {
        // write!(self.stdout, "{}{}", clear::All, cursor::Goto(1, 1)).unwrap();
    }
}

#[derive(Clone, Copy, Debug)]
struct RKKey {
    pub a: u8,
    pub b: u8,
    pub c: u8,
}

impl TryFrom<Key> for RKKey {
    type Error = ();

    fn try_from(key: Key) -> Result<Self, ()> {
        match key {
            Key::Char('x')  => Ok(RKKey { a: 0b_0111_1111, b: 0b_1111_1110, c: 0 }),
            Key::Char('p')  => Ok(RKKey { a: 0b_1011_1111, b: 0b_1111_1110, c: 0 }),
            Key::Char('h')  => Ok(RKKey { a: 0b_1101_1111, b: 0b_1111_1110, c: 0 }),
            Key::Char('@')  => Ok(RKKey { a: 0b_1110_1111, b: 0b_1111_1110, c: 0 }),
            Key::Char('8')  => Ok(RKKey { a: 0b_1111_0111, b: 0b_1111_1110, c: 0 }),
            Key::Char('0')  => Ok(RKKey { a: 0b_1111_1011, b: 0b_1111_1110, c: 0 }),
            Key::Char('\t') => Ok(RKKey { a: 0b_1111_1101, b: 0b_1111_1110, c: 0 }),
            Key::Home       => Ok(RKKey { a: 0b_1111_1110, b: 0b_1111_1110, c: 0 }),

            Key::Char('y')  => Ok(RKKey { a: 0b_0111_1111, b: 0b_1111_1101, c: 0 }),
            Key::Char('q')  => Ok(RKKey { a: 0b_1011_1111, b: 0b_1111_1101, c: 0 }),
            Key::Char('i')  => Ok(RKKey { a: 0b_1101_1111, b: 0b_1111_1101, c: 0 }),
            Key::Char('a')  => Ok(RKKey { a: 0b_1110_1111, b: 0b_1111_1101, c: 0 }),
            Key::Char('9')  => Ok(RKKey { a: 0b_1111_0111, b: 0b_1111_1101, c: 0 }),
            Key::Char('1')  => Ok(RKKey { a: 0b_1111_1011, b: 0b_1111_1101, c: 0 }),
            Key::Char('\r') => Ok(RKKey { a: 0b_1111_1101, b: 0b_1111_1101, c: 0 }), // ПС
            // Key::???     => Ok(RKKey { a: 0b_1111_1110, b: 0b_1111_1101, c: 0 }), // СТР

            Key::Char('z')  => Ok(RKKey { a: 0b_0111_1111, b: 0b_1111_1011, c: 0 }),
            Key::Char('r')  => Ok(RKKey { a: 0b_1011_1111, b: 0b_1111_1011, c: 0 }),
            Key::Char('j')  => Ok(RKKey { a: 0b_1101_1111, b: 0b_1111_1011, c: 0 }),
            Key::Char('b')  => Ok(RKKey { a: 0b_1110_1111, b: 0b_1111_1011, c: 0 }),
            Key::Char(':')  => Ok(RKKey { a: 0b_1111_0111, b: 0b_1111_1011, c: 0 }),
            Key::Char('2')  => Ok(RKKey { a: 0b_1111_1011, b: 0b_1111_1011, c: 0 }),
            Key::Char('\n') => Ok(RKKey { a: 0b_1111_1101, b: 0b_1111_1011, c: 0 }), // ВК
            // Key::???     => Ok(RKKey { a: 0b_1111_1110, b: 0b_1111_1011, c: 0 }), // АР2

            Key::Char('[')  => Ok(RKKey { a: 0b_0111_1111, b: 0b_1111_0111, c: 0 }),
            Key::Char('s')  => Ok(RKKey { a: 0b_1011_1111, b: 0b_1111_0111, c: 0 }),
            Key::Char('k')  => Ok(RKKey { a: 0b_1101_1111, b: 0b_1111_0111, c: 0 }),
            Key::Char('c')  => Ok(RKKey { a: 0b_1110_1111, b: 0b_1111_0111, c: 0 }),
            Key::Char(';')  => Ok(RKKey { a: 0b_1111_0111, b: 0b_1111_0111, c: 0 }),
            Key::Char('3')  => Ok(RKKey { a: 0b_1111_1011, b: 0b_1111_0111, c: 0 }),
            Key::Backspace  => Ok(RKKey { a: 0b_1111_1101, b: 0b_1111_0111, c: 0 }),
            Key::F(1)       => Ok(RKKey { a: 0b_1111_1110, b: 0b_1111_0111, c: 0 }),

            Key::Char('\\') => Ok(RKKey { a: 0b_0111_1111, b: 0b_1110_1111, c: 0 }),
            Key::Char('t')  => Ok(RKKey { a: 0b_1011_1111, b: 0b_1110_1111, c: 0 }),
            Key::Char('l')  => Ok(RKKey { a: 0b_1101_1111, b: 0b_1110_1111, c: 0 }),
            Key::Char('d')  => Ok(RKKey { a: 0b_1110_1111, b: 0b_1110_1111, c: 0 }),
            Key::Char('<')  => Ok(RKKey { a: 0b_1111_0111, b: 0b_1110_1111, c: 0 }),
            Key::Char('4')  => Ok(RKKey { a: 0b_1111_1011, b: 0b_1110_1111, c: 0 }),
            Key::Left       => Ok(RKKey { a: 0b_1111_1101, b: 0b_1110_1111, c: 0 }),
            Key::F(2)       => Ok(RKKey { a: 0b_1111_1110, b: 0b_1110_1111, c: 0 }),

            Key::Char(']')  => Ok(RKKey { a: 0b_0111_1111, b: 0b_1101_1111, c: 0 }),
            Key::Char('u')  => Ok(RKKey { a: 0b_1011_1111, b: 0b_1101_1111, c: 0 }),
            Key::Char('m')  => Ok(RKKey { a: 0b_1101_1111, b: 0b_1101_1111, c: 0 }),
            Key::Char('e')  => Ok(RKKey { a: 0b_1110_1111, b: 0b_1101_1111, c: 0 }),
            Key::Char('-')  => Ok(RKKey { a: 0b_1111_0111, b: 0b_1101_1111, c: 0 }),
            Key::Char('5')  => Ok(RKKey { a: 0b_1111_1011, b: 0b_1101_1111, c: 0 }),
            Key::Up         => Ok(RKKey { a: 0b_1111_1101, b: 0b_1101_1111, c: 0 }),
            Key::F(3)       => Ok(RKKey { a: 0b_1111_1110, b: 0b_1101_1111, c: 0 }),

            Key::Char('^')  => Ok(RKKey { a: 0b_0111_1111, b: 0b_1011_1111, c: 0 }),
            Key::Char('v')  => Ok(RKKey { a: 0b_1011_1111, b: 0b_1011_1111, c: 0 }),
            Key::Char('n')  => Ok(RKKey { a: 0b_1101_1111, b: 0b_1011_1111, c: 0 }),
            Key::Char('f')  => Ok(RKKey { a: 0b_1110_1111, b: 0b_1011_1111, c: 0 }),
            Key::Char('>')  => Ok(RKKey { a: 0b_1111_0111, b: 0b_1011_1111, c: 0 }),
            Key::Char('6')  => Ok(RKKey { a: 0b_1111_1011, b: 0b_1011_1111, c: 0 }),
            Key::Right      => Ok(RKKey { a: 0b_1111_1101, b: 0b_1011_1111, c: 0 }),
            Key::F(4)       => Ok(RKKey { a: 0b_1111_1110, b: 0b_1011_1111, c: 0 }),

            Key::Char(' ')  => Ok(RKKey { a: 0b_0111_1111, b: 0b_0111_1111, c: 0 }),
            Key::Char('w')  => Ok(RKKey { a: 0b_1011_1111, b: 0b_0111_1111, c: 0 }),
            Key::Char('o')  => Ok(RKKey { a: 0b_1101_1111, b: 0b_0111_1111, c: 0 }),
            Key::Char('g')  => Ok(RKKey { a: 0b_1110_1111, b: 0b_0111_1111, c: 0 }),
            Key::Char('/')  => Ok(RKKey { a: 0b_1111_0111, b: 0b_0111_1111, c: 0 }),
            Key::Char('7')  => Ok(RKKey { a: 0b_1111_1011, b: 0b_0111_1111, c: 0 }),
            Key::Down       => Ok(RKKey { a: 0b_1111_1101, b: 0b_0111_1111, c: 0 }),
            // ?            => Ok(RKKey { a: 0b_1111_1110, b: 0b_0111_1111, c: 0 }),

            Key::Ctrl('a')  => Ok(RKKey { a: 0, b: 0, c: 0b_0110_0000 }), // РУС/ЛАТ
            Key::Ctrl('u')  => Ok(RKKey { a: 0, b: 0, c: 0b_1100_0000 }), // УС
            Key::Ctrl('s')  => Ok(RKKey { a: 0, b: 0, c: 0b_1100_0000 }), // СС

            _ => Err(()),
        }
    }
}

struct RKKeyboardInternal {
    key_stream: RefCell<termion::input::Keys<termion::AsyncReader>>,
    current_key: Cell<(RKKey, time::Instant)>,
    current_line: Cell<u8>,
    state: Cell<u8>,
}

#[derive(Clone)]
struct RKKeyboard(Arc<RKKeyboardInternal>);

impl RKKeyboard {
    pub fn new() -> Self {
        RKKeyboard(Arc::new(RKKeyboardInternal {
            key_stream: RefCell::new(async_stdin().keys()),
            current_key: Cell::new((
                RKKey {
                    a: 0,
                    b: 0,
                    c: 0,
                },
                time::Instant::now() - time::Duration::from_secs(100)
            )),
            current_line: Cell::new(0),
            state: Cell::new(0),
        }))
    }

    pub fn process_key(&self) -> bool {
        let b = self.0.key_stream.borrow_mut().next();
        if let Some(Ok(k)) = b {
            if k == Key::Ctrl('c') || k == Key::Ctrl('q') {
                return true;
            }
            if let Ok(key) = RKKey::try_from(k) {
                self.0.current_key.set((key, time::Instant::now()));
            }
        }
        false
    }

    fn get_current_key(&self) -> Option<RKKey> {
        let (key, when) = self.0.current_key.get();
        if time::Instant::now().duration_since(when) > time::Duration::from_secs(1) {
            None
        } else {
            Some(key)
        }
    }

    pub fn get_indicators(&self) -> u8 {
        self.0.state.get() & 0x0F
    }
}

impl rs580::Memory for RKKeyboard {
    #[inline]
    fn get_u8(&self, addr: u16) -> u8 {
        if addr == 1 {
            let current_key = self.get_current_key();
            if self.0.current_line.get() == 0 {
                if current_key.is_some() {
                    return 0; // <> ff
                } else {
                    return 0xFF;
                }
            }
            if let Some(RKKey {a, b, ..}) = current_key {
                if self.0.current_line.get() == a {
                    return b;
                } else {
                    return 0xFF;
                }
            }
        } else if addr == 2 {
            match self.get_current_key() {
                Some(RKKey {a: 0, b: 0, c}) => {
                    return c & 0xF0 | (self.0.state.get() & 0x0F);
                },
                _ => {},
            }
        }

        0xFF
    }

    #[inline]
    fn set_u8(&mut self, addr: u16, value: u8) {
        if addr == 0 {
            self.0.current_line.set(value);
        }

        if addr == 2 {
            self.0.state.set(value & 0x0F);
        }

        if addr == 3 && (value & 0x80) == 0 {
            let bit = (value >> 1) & 7;
            let mask = 1 << bit;
            let state = self.0.state.get();
            if value & 1 == 1 {
                self.0.state.set(state | mask);
            } else {
                self.0.state.set(state & !mask);
            }
        }
    }
}

fn main() {
    let keyboard = RKKeyboard::new();
    let mut display = RKDisplay::new().unwrap();

    let memory = rs580::SegmentedMemory::new()
        .add(0x0000, 0x4000, Box::new(rs580::RAM::default()))
        .add(0x8000, 0xA000, Box::new(keyboard.clone()))
        .add(0xF800, 0x10000, Box::new(rs580::ROM::new(&ROM)));

    let mut machine = rs580::Machine::new(Box::new(memory));
    machine.registers.pc = 0xF800;

    loop {
        display.copy_from_machine(&machine);
        display.copy_from_keyboard(&keyboard);
        display.print().unwrap();
        if keyboard.process_key() {
            break;
        }

        machine.step();
        if machine.halted {
            println!("HALT");
            break;
        }

        thread::sleep(time::Duration::from_micros(10));
    }
}
