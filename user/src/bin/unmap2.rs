#![no_std]
#![no_main]

extern crate user_lib;

use user_lib::{mmap, munmap};

/*
 * expected: return 0
*/

#[no_mangle]
fn main() -> i32 {
    let start: usize = 0x10000000;
    let len: usize = 4096;
    let prot: usize = 3;
    assert_eq!(0, mmap(start, len, prot));
    assert_eq!(munmap(start, len + 1), -1);
    assert_eq!(munmap(start + 1, len - 1), -1);
    0
}
