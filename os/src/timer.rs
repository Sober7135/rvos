use crate::config::*;
use crate::sbi;
use riscv::register::time;

pub(crate) fn get_time() -> usize {
    time::read()
}

pub(crate) fn get_time_ms() -> usize {
    time::read() / (TIMEBASE_FREQUENCY / MSEC_PER_SEC)
}

pub(crate) fn set_next_trigger() {
    sbi::set_timer(get_time() + TIMEBASE_FREQUENCY / TICK_PER_SEC)
}
