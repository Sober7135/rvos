#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

#[macro_use]
extern crate alloc;
extern crate bitflags;
extern crate buddy_system_allocator;

use core::arch::global_asm;
use process::processor::schedule;

#[macro_use]
mod console;
mod config;
mod drivers;
mod fs;
mod lang_items;
mod logger;
mod mm;
mod process;
mod qemu;
mod sbi;
mod stack_trace;
mod sync;
mod syscall;
mod timer;
mod trap;

global_asm!(include_str!("entry.asm"));

fn clear_bss() {
    extern "C" {
        fn ebss();
        fn sbss();
    }
    (ebss as usize..sbss as usize)
        .for_each(|address| unsafe { (address as *mut u8).write_volatile(0) })
}

#[no_mangle]
fn rust_main() {
    clear_bss();
    logger::init();
    mm::init();
    trap::init();
    trap::enable_timer_interrupt();
    timer::set_next_trigger();
    fs::list_apps();
    process::add_init_proc();
    schedule();
}
