#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(panic_info_message)]

use core::arch::global_asm;
use log::{debug, error, info, trace, warn};

mod loader;
#[macro_use]
mod console;
mod config;
mod lang_items;
mod logger;
mod sbi;
mod stack_trace;
mod sync;
mod syscall;
mod task;
mod trap;

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));

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

    trace!("[kernel][TEST] THIS IS TRACE");
    info!("[kernel][TEST] THIS IS INFO");
    debug!("[kernel][TEST] THIS IS DEBUG");
    warn!("[kernel][TEST] THIS IS WARN");
    error!("[kernel][TEST] THIS IS ERROR");

    println!("[kernel][TEST] Hello, World!");

    trap::init();
    loader::load_apps();
    task::run_first_task();
    // TODO
    todo!()
}
