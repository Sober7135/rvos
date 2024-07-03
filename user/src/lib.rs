#![no_std]
#![feature(panic_info_message)]
#![feature(allow_internal_unstable)] // `format_args_nl`
#![feature(linkage)]
#![feature(format_args_nl)]
#![allow(internal_features)]
#![feature(alloc_error_handler)]

extern crate alloc;

pub mod config;
pub mod console;
mod heap_allocator;
mod lang_items;
mod syscall;

use syscall::*;

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    heap_allocator::init_heap();
    // heap_allocator::heap_test();
    // We don't need to clear bss, because the frame allocator will do that for us.
    exit(main());
    unreachable!("Unreachable! The program must be terminated")
}

// if user program not define `main`, compiler will link it to this, so called "weak"
#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
    panic!("No main defined");
}

// current we only support buf.len() == 1
pub fn read(fd: usize, buf: &mut [u8]) -> isize {
    sys_read(fd, buf)
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

pub fn get_time() -> isize {
    sys_get_time()
}

pub fn getpid() -> isize {
    sys_getpid()
}

pub fn fork() -> isize {
    sys_fork()
}

pub fn exec(path: &str) -> isize {
    sys_exec(path)
}

pub fn wait(exit_code: &mut i32) -> isize {
    waitpid(-1, exit_code)
}

pub fn waitpid(pid: isize, exit_code: &mut i32) -> isize {
    sys_waitpid(pid, exit_code as *mut i32)
}

pub fn sleep(len_ms: usize) -> isize {
    let start = sys_get_time();
    while sys_get_time() < start + len_ms as isize {
        sys_yield();
    }
    0
}

pub fn mmap(start: usize, len: usize, prot: usize) -> isize {
    sys_mmap(start, len, prot)
}

pub fn munmap(start: usize, len: usize) -> isize {
    sys_munmap(start, len)
}
