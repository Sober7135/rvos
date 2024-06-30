#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(panic_info_message)]

use core::arch::global_asm;
use log::{debug, error, info, trace, warn};

use crate::sbi::shutdown;

mod console;
mod init;
mod lang_items;
mod logger;
mod sbi;

global_asm!(include_str!("entry.asm"));

fn clear_bss() {
    extern "C" {
        fn ebss();
        fn sbss();
    }
    (ebss as usize..sbss as usize)
        .for_each(|address| unsafe { (address as *mut u8).write_volatile(0) })
}

fn print_segment_info(segment: &str, start: usize, end: usize) {
    trace!("[kernel] .{:6}: [{:#x}, {:#x}]", segment, start, end);
}

#[no_mangle]
fn rust_main() {
    extern "C" {
        fn stext();
        fn etext();
        fn srodata();
        fn erodata();
        fn sdata();
        fn edata();
        fn sbss();
        fn ebss();
        fn stack_lower_bound();
        fn stack_top();
    }
    clear_bss();
    logger::init();
    print_segment_info("text", stext as usize, etext as usize);
    print_segment_info("rodata", srodata as usize, erodata as usize);
    print_segment_info("data", sdata as usize, edata as usize);
    print_segment_info("bss", sbss as usize, ebss as usize);
    print_segment_info("stack", stack_lower_bound as usize, stack_top as usize);

    trace!("THIS IS TRACE");
    info!("THIS IS INFO");
    debug!("THIS IS DEBUG");
    warn!("THIS IS WARN");
    error!("THIS IS ERROR");

    print!("Hello, World!\n");

    // panic!("THIS IS PANIC");

    shutdown(false);

}
