use std::fs::OpenOptions;
use std::io::{stdout, Read, Write};
use std::os::fd::AsRawFd;
use std::process::exit;
use std::thread::sleep;
use std::time::Duration;
use libc::{c_ulong, ioctl};
use crate::print::Position;

/// EVIOCGRAB calculates to:\
/// EVIOCGRAB 计算方式
/// ```
/// #define _IOC_NRBITS   0b1000
/// #define _IOC_TYPEBITS 0b1000
/// #define _IOC_SIZEBITS 0b1110
/// #define _IOC_DIRBITS  0b0010
///
/// #define _IOC_NRSHIFT  0
/// #define _IOC_TYPESHIFT (_IOC_NRSHIFT + _IOC_NRBITS)
/// #define _IOC_SIZESHIFT (_IOC_TYPESHIFT + _IOC_TYPEBITS)
/// #define _IOC_DIRSHIFT (_IOC_SIZESHIFT + _IOC_SIZEBITS)
///
/// #define _IOC(dir,type,nr,size) 
///     (((dir)  << _IOC_DIRSHIFT) | 
///      ((type) << _IOC_TYPESHIFT) | 
///      ((nr)   << _IOC_NRSHIFT) | 
///      ((size) << _IOC_SIZESHIFT))
///
/// #define _IOW(type,nr,size) _IOC(_IOC_WRITE,(type),(nr),(size))
/// ```
/// 即：
/// ```
/// EVIOCGRAB = _IOW('E', 0b10010000, int)
///            = _IOC(_IOC_WRITE, 'E', 0b10010000, sizeof(int))
///            = (_IOC_WRITE << _IOC_DIRSHIFT) |
///              ('E' << _IOC_TYPESHIFT) |
///              (0b10010000 << _IOC_NRSHIFT) |
///              (sizeof(int) << _IOC_SIZESHIFT)
/// ```
const EVIOCGRAB: c_ulong = 0b1000000000001000100010110010000;

pub fn listener(path: &str, _screen: Position) {
    let mut file = OpenOptions::new().read(true).open(path).expect("请用 Root 权限运行本程序");
    let mut isctrl = false;
    
    // 抢占图形化界面之前先让图形化界面释放键盘按压时间
    sleep(Duration::new(0, 100_000_000));
    
    // 使用ioctl来抓取设备
    unsafe {
        if ioctl(file.as_raw_fd(), EVIOCGRAB, 1) != 0 {
            eprintln!("Failed to grab the device");
            return;
        }
    }
    
    // 初始化一个缓冲区来存储读取的数据
    let mut buffer = [0u8; 24];  // 输入事件结构体的大小是24字节

    loop {
        // 从文件读取24字节的数据
        file.read_exact(&mut buffer).unwrap();

        // 解析时间戳（tv_sec和tv_usec）
        // let tv_sec = u64::from_ne_bytes(buffer[..8].try_into().unwrap());
        // let tv_usec = u64::from_ne_bytes(buffer[8..16].try_into().unwrap());

        // 解析事件类型、代码和值
        let evtype = u16::from_ne_bytes(buffer[16..18].try_into().unwrap());
        let code = u16::from_ne_bytes(buffer[18..20].try_into().unwrap());
        let value = i32::from_ne_bytes(buffer[20..24].try_into().unwrap());

        // 只处理键盘事件（evtype == 1 表示键盘事件）
        if evtype == 1 {
            // println!("Key event: code = {}, value = {}", code, value);
            match code {
                0x1D_u16 | 0x61 => isctrl = value == 1,
                0x1C if value == 2 => {
                    print!("\x1b[H");
                    stdout().flush().unwrap();
                },
                0x2E if isctrl => {
                    // 当程序退出时，释放设备
                    unsafe {
                        ioctl(file.as_raw_fd(), EVIOCGRAB, 0);
                    }
                    println!("\x07");
                    exit(0)
                }
                _ => {}
            }
        }
    }
}