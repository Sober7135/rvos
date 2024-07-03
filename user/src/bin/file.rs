#![no_main]
#![no_std]

use user_lib::{close, open, read, write, OpenFlags};

#[macro_use]
extern crate user_lib;

#[no_mangle]
pub fn main() -> i32 {
    let test_str = "Hello, World!";
    let fname = "fname\0";
    let fd = open(&fname, OpenFlags::CREATE | OpenFlags::WRONLY);
    assert!(fd > 0);

    let fd = fd as usize;
    let writed = write(fd, test_str.as_bytes());
    assert_eq!(writed as usize, test_str.as_bytes().len());
    close(fd);

    let fd = open(fname, OpenFlags::RDONLY);
    assert!(fd > 0);
    let fd = fd as usize;
    let mut buffer = [0u8; 100];
    let read_len = read(fd, &mut buffer) as usize;
    println!("read {} bytes", read_len);
    close(fd);

    assert_eq!(test_str, core::str::from_utf8(&buffer[..read_len]).unwrap());
    0
}
