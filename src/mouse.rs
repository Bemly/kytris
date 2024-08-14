use std::fs::OpenOptions;
use std::io::{stdout, Read, Write};
use std::os::fd::AsRawFd;
use std::process::exit;
use libc::{c_ulong, ioctl};
use crate::print::{font_dir, Position};

pub fn listener(path: &str, screen: Position) {
    let mut file = OpenOptions::new().read(true).open(path).expect("请用 Root 权限运行本程序");
    let mut position = Position(0, 0);

    const EVIOCGRAB: c_ulong = 0b1000000000001000100010110010000;
    // 使用ioctl来抓取设备
    unsafe {
        if ioctl(file.as_raw_fd(), EVIOCGRAB, 1) != 0 {
            eprintln!("Failed to grab the device");
            return;
        }
    }

    print!("窗口大小：({},{})", position.0, position.1);
    stdout().flush().unwrap();

    for i in 0..screen.1 {
        if i == screen.1 - 1 {
            print!("{i} || ▄▀█");
            stdout().flush().unwrap();
        } else {
            println!("{i} || ▄▀█");
        }
    }
    print!("\x1b[H");
    stdout().flush().unwrap();
    
    font_dir();

    loop {

        // timestamp-int              | timestamp-float         | type  | code  | value
        // 0x 00 00 00 00 00 00 00 00 | 00 00 00 00 00 00 00 00 | 00 00 | 00 00 | 00 00 00 00
        let mut packet = [0u8; 24];
        file.read_exact(&mut packet).unwrap();

        /* TODO:
            根据时间戳求鼠标加速度
            鼠标加速度 = 鼠标速度 / 鼠标时间间隔
            求鼠标采样率（频率相反）
            鼠标采样率 = 鼠标采样次数 / 鼠标时间间隔
            争取把timestamp搞出来
        */
        let _tv_sec = u64::from_ne_bytes(packet[..8].try_into().unwrap());
        let _tv_usec = u64::from_ne_bytes(packet[8..16].try_into().unwrap());
        let evtype = u16::from_ne_bytes(packet[16..18].try_into().unwrap());
        let code = u16::from_ne_bytes(packet[18..20].try_into().unwrap());
        let value = i32::from_ne_bytes(packet[20..].try_into().unwrap());

        // println!("Time: {}.{} type:{:#x} code:{:#x} value:{:#x}", tv_sec, tv_usec, evtype, code, value);

        // 监听鼠标事件
        let mouse_key = || {
            match code {
                0x110_u16 => { /* BTN_LEFT */
                    if let 0x1_i32 = value {
                        // BTN_DOWN
                    } else {
                        // BTN_UP
                    }
                },
                0x111 => { /* BTN_RIGHT */
                    if let 0x1_i32 = value {
                        // print!("\x1b[;46mW");
                        stdout().flush().unwrap();
                    } else {
                        // print!("\x1b[0m");
                        stdout().flush().unwrap();
                    }
                },
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
                        if position.0 > 0 {
                            position.0 -= 1;
                            print!("\x1b[1D")
                        } else {
                            print!("\x07")
                        }
                    } else { // 右
                        if position.0 < screen.0 {
                            position.0 += 1;
                            print!("\x1b[1C")
                        } else {
                            print!("\x07")
                        }
                    }
                },
                1 => { /* REL_Y */
                    if value.abs() > 1 {
                        if value < 0 { // 上
                            if position.1 > 0 {
                                position.1 -= 1;
                                print!("\x1b[1A")
                            } else {
                                print!("\x07")
                            }
                        } else { // 下
                            if position.1 < screen.1 {
                                position.1 += 1;
                                print!("\x1b[1B")
                            } else {
                                print!("\x07")
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

        // print!("\x07");
        stdout().flush().unwrap();
    }
}