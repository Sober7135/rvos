use super::*;

pub fn console_putchar(c: usize) -> usize {
    legacy::sbi_call_1(legacy::EID::CONSOLE_PUTCHAR, c)
}

pub fn shutdown(failure: bool) -> ! {
    use srst::*;
    if !failure {
        srst::system_reset(ResetType::SHUTDOWN, ResetReason::NoReason);
    } else {
        srst::system_reset(ResetType::SHUTDOWN, ResetReason::SystemFailure);
    }
    unreachable!()
}

// DEAD CODE
#[cfg(test)]
mod dead_code {
    #[test]
    fn dead() {
        sbi_rt::legacy::console_putchar(112);
    }
}
