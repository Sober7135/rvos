use crate::sbi::shutdown;
use crate::stack_trace::print_stack_trace;
use core::panic::PanicInfo;
use log::error;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        error!(
            "[kernel] Panicked at {}:{} {}",
            location.file(),
            location.line(),
            info.message()
        );
    } else {
        error!("[kernel] Panicked: {}", info.message());
    }
    unsafe { print_stack_trace() }
    shutdown(true)
}
