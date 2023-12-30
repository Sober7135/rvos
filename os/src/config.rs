pub(crate) const MAX_APP_NUM: usize = 16;

pub(crate) const APP_BASE_ADDRESS: usize = 0x8040_0000;
pub(crate) const APP_SIZE_LIMIT: usize = 0x2_0000;

pub(crate) const KERNEL_STACK_SIZE: usize = 4096 * 2; // 8KiB
pub(crate) const USER_STACK_SIZE: usize = 4096 * 2; // 8KiB

// qemu-system-riscv64 -machine virt,dumpdtb=dump.dtb
// dtc dump.dtb | vi -
pub(crate) const TIMEBASE_FREQUENCY: usize = 0x989680;
pub(crate) const TICK_PER_SEC: usize = 100;
pub(crate) const MSEC_PER_SEC: usize = 1000;

pub(crate) const HEAP_ORDER_SIZE: usize = 32;
pub(crate) const KERNEL_HEAP_SIZE: usize = 0x300_000;

// page
pub(crate) const PAGE_SIZE_BITS: usize = 12;
pub(crate) const PAGE_SIZE: usize = 1 << PAGE_SIZE_BITS;
pub(crate) const PA_WIDTH_SV39: usize = 56;
pub(crate) const PPN_WIDTH_SV39: usize = PA_WIDTH_SV39 - PAGE_SIZE_BITS;

pub(crate) const MEMORY_END: usize = 0x8080_0000;
