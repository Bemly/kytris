//! kytris - 强大的支持鼠标输入的终端编辑器
//!
//! 启动方法：
//! ```
//! ./kytris
//! ```
//! 
//! 使用外部字体库(支持.bf/.bdf文件解析):\
//!     .bf     BemlyFont                   蓝莓现代位图字体文件\
//!     .bdf    BitmapDistributionFormat   Adobe位图字库文件(itch.io像素小游戏常用)
//! ```
//! ./kytris -f .bf/.bdf文件路径
//! ./kytris --font .bf/.bdf文件路径
//! ```
//! 
//! 指定绑定鼠标输入设备：
//! ```
//! ./kytris -k </dev/input/event*> -m </dev/input/event*>
//! ./kytris --key </dev/input/event*> ---mouse </dev/input/event*>
//! ```
//! 
//! 编译二进制文件：
//! ```
//! cargo run --package kytris --bin kytris
//! ```
mod mouse;
mod print;
mod keyboard;

use crate::print::Position;
use std::thread;

const MOUSE_EVENT_PATH: &str = "/dev/input/event7";
const KEYBOARD_EVENT_PATH: &str = "/dev/input/event6";

fn main() {
    let mouse_screen = Position::init();
    let key_screen = mouse_screen.clone();

    // 创建鼠标监听线程
    let mouse_thread = thread::spawn(move || {
        mouse::listener(MOUSE_EVENT_PATH, mouse_screen);
    });

    // 创建键盘监听线程
    let keyboard_thread = thread::spawn(move || {
        keyboard::listener(KEYBOARD_EVENT_PATH, key_screen);
    });

    // 等待鼠标监听线程结束
    mouse_thread.join().unwrap();

    // 等待键盘监听线程结束
    keyboard_thread.join().unwrap();
}