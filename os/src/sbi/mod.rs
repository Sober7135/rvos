// https://github.com/riscv-non-isa/riscv-sbi-doc/releases

mod binary;
mod legacy;
mod srst;
mod timer;

use binary::*;
use core::arch::asm;

pub fn console_putchar(c: usize) -> usize {
    legacy::sbi_call_1(legacy::Eid::CONSOLE_PUTCHAR, c)
}

pub fn console_getchar() -> usize {
    legacy::sbi_call_0(legacy::Eid::CONSOLE_GETCHAR)
}

pub fn shutdown(failure: bool) -> ! {
    use srst::*;
    if !failure {
        srst::system_reset(ResetType::Shutdown, ResetReason::NoReason);
    } else {
        srst::system_reset(ResetType::Shutdown, ResetReason::SystemFailure);
    }
    unreachable!()
}

pub fn set_timer(stime_value: usize) {
    timer::set_timer(stime_value);
}
