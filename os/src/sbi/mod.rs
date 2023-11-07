// https://www.scs.stanford.edu/~zyedidia/docs/riscv/riscv-sbi.pdf

mod binary;
mod legacy;
mod srst;

use core::arch::asm;

pub fn console_putchar(c: usize) -> usize {
    legacy::sbi_call_1(legacy::Eid::CONSOLE_PUTCHAR, c)
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
