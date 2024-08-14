use std::io::{stdin, stdout, Read, Write};
use std::sync::mpsc;
use std::thread;

/// 屏幕坐标
# [derive(Debug)]
pub struct Position(pub u16, pub u16);
impl Position {
    pub fn new(x: u16, y: u16) -> Self { Position(x, y) }
    pub fn init() -> Self {
        // 关闭他喵的为我好的行缓冲模式
        set_row_cache_mode(false);

        // 线程通信，和gpio一致
        // Master ==tx ==> Slave
        // Master <== rx== Slave
        // revc时阻塞等待
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
                    Err(e) => eprintln!("{e}")
                }
            }
        });

        // 清屏 移动光标到右下 获取位置 初始化坐标
        print!("\x1b[3J\x1b[9999C\x1b[9999B\x1b[6n");
        stdout().flush().unwrap();
        let mut screen = (0, 0);

        // 得到坐标(行列) => 自己计算行列不再依靠单独计算
        // TODO:　这样做不能动态更改行列，之后需要时刻刷新，解藕注册模式
        if let Ok(input) = rx.recv() {
            // println!("你输入的是：{:#x?}", input);
            // test output: 你输入的是：[0x1b,0x5b,0x31,0x32,0x3b,0x32,0x35,0x30,0x52,0xa,]
            // ASCII MODE:  0x52 => `R`, 0xa => `\n`, 0x5b => `[`, X ,0x3b => `;`, Y
            if !input.is_ascii() { unimplemented!("Invalid input. 不正确的输入，获取坐标异常") }
            let mut slice_symbol = 0_u8;
            let mut x = Vec::new();
            let mut y = Vec::new();
            for ascii in input {
                match ascii {
                    b'[' => slice_symbol = 1,
                    b';' => slice_symbol = 2,
                    b'R' => break,
                    height if slice_symbol == 1 => y.push(height),
                    width if slice_symbol == 2 => x.push(width),
                    _ => {}
                }
            }

            let ascii2u16 = |acc, &ascii| acc * 10u16 + (ascii - 0x30) as u16;
            screen = (x.iter().fold(0, ascii2u16), y.iter().fold(0, ascii2u16));
        }

        // 开启他喵的为我好的行缓冲模式
        set_row_cache_mode(true);

        // 移动光标到左上
        print!("\x1b[3J\x1b[H");
        stdout().flush().unwrap();
        Position(screen.0, screen.1)
    }
    fn _auto_update(&mut self) {
        print!("\x1b[6n");
        stdout().flush().unwrap();
        if let Some((x, y)) = self.read_cursor_position() {
            self.0 = x;
            self.1 = y;
        }
    }
    
    #[allow(dead_code)]
    fn read_cursor_position(&self) -> Option<(u16,u16)> { todo!() }
    fn _update(&mut self) { todo!() }
    pub fn clone(&self) -> Self { Position(self.0, self.1) }
}

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

/** 6211\
   0x EA 48 FE 48 EA 44 DA\
   0b EA => 1110 1010      ▀█▀ █ ▀\
   0b 48 => 0100 1000      ▀█▀▀█▀▀\
   0b FE => 1111 1110      ▀█▀ ▀▄▀\
   0b 48 => 0100 1000      ▀▀ ▀▀ ▀\
   0b EA => 1110 1010\
   0b 44 => 0100 0100\
   0b DA => 1101 1010\
   0b 00 => 0000 0000

   0x EA 48 FE 48 EC 4A D4\
   0b EA => 1110 1010      ▀█▀ █ ▀\
   0b 48 => 0100 1000      ▀█▀▀█▀▀\
   0b FE => 1111 1110      ▀█  █▀▄\
   0b 48 => 0100 1000      ▀▀ ▀▀ ▀\
   0b EC => 1100 1100\
   0b 4A => 0100 1010\
   0b D4 => 1101 1010
    **/
pub fn font_dir() {
    // 这可咋整哇，英文的 unicode/ISO10646 规范读不快，好麻烦（
    let font = [0x0_u8; 7];
    let mut fonts = [font; u16::MAX as usize];

    // 文件游标 BITMAP => HEAD, ENDCHAR => EOF
    let mut rows = 0;
    let mut cursor = 0;
    let mut isdata = false;
    
    // TODO: 使用外部
    let bdf = include_str!("JiZhi-bitmap-8.bdf");
    for line in bdf.lines() {
        match line {
            "BITMAP" => {
                isdata = true;
                rows = 0
            },
            "ENDCHAR" => isdata = false,
            data if isdata => {
                // (?s)BITMAP.{0,4}(?=[^0-9A-Fa-f\n]) 我找了半天了 BITMAP <= 0xDATA <= ENDCHAR
                // 中间就没有不能被解析的字符， `Result::unwrap()` on an `Err` value: ParseIntError { kind: InvalidDigit }
                // 冤枉啊！给我个这个
                // 草 我不小心源文件多敲了一个退格
                fonts[cursor][rows] = u8::from_str_radix(data, 16).unwrap();
                if rows < 6 { rows += 1 }
            },
            prefix => if let Some((prefix, i)) = prefix.split_once(' ') {
                // -1的encoding是什么鬼啊？！
                if let "ENCODING" = prefix { cursor = i.parse::<usize>().unwrap_or(0) }
            }
        }
    }
    // 导入完毕
    
    let s = String::from("本编辑器自带字体解析");

    for c in s.chars() {
        let font = fonts[c as usize];
        for j in (0..8).step_by(2) {
            for i in (0..8).rev() {
                let high = (font[j] >> i) & 1;
                let low = if j == 6 { 0 } else { (font[j + 1] >> i) & 1 };
                // 0b 00 => ' ' 10 => '▀' 01 => '▄' 11 => '█'
                match (high, low) {
                    (0, 0) => print!(" "),
                    (0, 1) => print!("▄"),
                    (1, 0) => print!("▀"),
                    (1, 1) => print!("█"),
                    _ => unreachable!("位移错误！")
                }
            }
            print!(" \x1b[9D\x1b[1B");
        }
        print!("\x1b[8C\x1b[9A");
    }

    print!("\x1b[H");
    stdout().flush().unwrap();
}
