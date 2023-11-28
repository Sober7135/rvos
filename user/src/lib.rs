#![no_std]
#![feature(panic_info_message)]
#![feature(allow_internal_unstable)] // `format_args_nl`
#![feature(linkage)]
#![feature(format_args_nl)]
#![allow(internal_features)]

pub mod console;
mod lang_items;
mod syscall;

use syscall::*;

fn clear_bss() {
    extern "C" {
        fn ebss();
        fn sbss();
    }
    (ebss as usize..sbss as usize)
        .for_each(|address| unsafe { (address as *mut u8).write_volatile(0) })
}

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    clear_bss();
    exit(main());
    unreachable!("Unreachable! The program must be terminated")
}

// if user program not define `main`, compiler will link it to this, so called "weak"
#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
    panic!("No main defined");
}

pub fn write(fd: usize, buf: &[u8]) -> isize {
    sys_write(fd, buf)
}

pub fn exit(state: i32) -> isize {
    sys_exit(state)
}

pub fn yield_() -> isize {
    sys_yield()
}
