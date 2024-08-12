use std::fs::OpenOptions;
use std::io::{stdin, stdout, Read, Write};
use std::process::exit;
use std::sync::mpsc;
use std::thread;

fn set_row_cache_mode(enable: bool) {
    let fd = libc::STDIN_FILENO;
    unsafe {
        let mut termios = std::mem::zeroed::<libc::termios>();
        libc::tcgetattr(fd, &mut termios);
        if !enable { termios.c_lflag &= !(libc::ICANON | libc::ECHO); }
        else { termios.c_lflag |= libc::ICANON | libc::ECHO; }
        libc::tcsetattr(fd, libc::TCSANOW, &termios);
    }
}

# [derive(Debug)]
struct Position(u16,u16);
impl Position {
    fn new(x: u16, y: u16) -> Self { Position(x, y) }
    fn init() -> Self { Position(0, 0) }
    fn auto_update(&mut self) {
        print!("\x1b[6n");
        stdout().flush().unwrap();
        if let Some((x, y)) = self.read_cursor_position() {
            self.0 = x;
            self.1 = y;
        }
    }
    fn read_cursor_position(&self) -> Option<(u16,u16)> { todo!() }
    fn update(&mut self) { todo!() }
}

fn main() {
    set_row_cache_mode(false);
    let mut position = Position::init();
    let mut screen = Position::init();
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut buffer = Vec::new();
        for byte in stdin().bytes() {
            match byte {
                Ok(ascii@ b'R') => {
                    buffer.push(ascii);
                    tx.send(buffer).unwrap();
                    break
                },
                Ok(ascii) => buffer.push(ascii),
                Err(e) => panic!("{}", e)
            }
        }
    });

    print!("\x1b[3J\x1b[9999C\x1b[9999B\x1b[6n");
    stdout().flush().unwrap();

    if let Ok(input) = rx.recv() {
        if !input.is_ascii() { unimplemented!("Invalid input. 不正确的输入，获取坐标异常") }
        let mut slice_symbol = 0_u8;
        let mut x = Vec::new();
        let mut y = Vec::new();
        for ascii in input {
            match ascii {
                b'[' => slice_symbol = 1,
                b';' => slice_symbol = 2,
                b'R' => break,
                Y if slice_symbol == 1 => y.push(Y),
                X if slice_symbol == 2 => x.push(X),
                _ => {}
            }
        }

        let ascii2u16 = |acc, &ascii| acc * 10u16 + (ascii - 0x30) as u16;
        screen.0 = x.iter().fold(0, ascii2u16);
        screen.1 = y.iter().fold(0, ascii2u16);
    }
    set_row_cache_mode(true);

    print!("\x1b[H");
    stdout().flush().unwrap();

    let mut file = OpenOptions::new().read(true).open("/dev/input/event7").unwrap();
    loop {

        let mut packet = [0u8; 24];
        file.read_exact(&mut packet).unwrap();

        let evtype = u16::from_ne_bytes(packet[16..18].try_into().unwrap());
        let code = u16::from_ne_bytes(packet[18..20].try_into().unwrap());
        let value = i32::from_ne_bytes(packet[20..].try_into().unwrap());

        let mouse_key = || {
            match code {
                0x110_u16 => { /* BTN_LEFT */ },
                0x111 => { /* BTN_RIGHT */ },
                0x112 => { /* BTN_MIDDLE */ },
                0x113 => { /* BTN_SIDE */ },
                0x114 => { /* BTN_EXTRA */
                    print!("\x1b[3J\x1b[H");
                    exit(0)
                },
                _ => unimplemented!()
            }
        };

        let mut mouse_wheel = || {
            match code {
                0 => { /* REL_X */
                    if value < 0 { // 左
                        if position.0 > 1 {
                            position.0 -= 1;
                            print!("\x1b[1D")
                        }
                    } else { // 右
                        if position.0 < screen.0 {
                            position.0 += 1;
                            print!("\x1b[1C")
                        }
                    }
                },
                1 => { /* REL_Y */
                    if value.abs() > 1 {
                        if value < 0 { // 上
                            if position.1 > 1 {
                                position.1 -= 1;
                                print!("\x1b[1A")
                            }
                        } else { // 下
                            if position.1 < screen.1 {
                                position.1 += 1;
                                print!("\x1b[1B")
                            }
                        }
                    }
                },
                8 => { /* REL_WHEEL */ },
                11 => { /* REL_WHEEL_HI_RES */ },
                _ => unimplemented!()
            }
        };

        match evtype {
            0x00_u16 => { /* EOF */ },
            0x01 => mouse_key(),
            0x02 => mouse_wheel(),
            0x03 => unimplemented!(),
            0x04 => { /* SYNC */ },
            _ => unimplemented!()
        }

        stdout().flush().unwrap();
    }
}