pub(crate) fn console_putchar(c: usize) {
    #[allow(deprecated)]
    sbi_rt::legacy::console_putchar(c);
}

pub(crate) fn shutdown(failure: bool) -> ! {
    use sbi_rt::{NoReason, Shutdown, SystemFailure};
    if !failure {
        sbi_rt::system_reset(Shutdown, NoReason);
    } else {
        sbi_rt::system_reset(Shutdown, SystemFailure);
    }
    unreachable!()
}
