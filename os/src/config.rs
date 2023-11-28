pub(crate) const MAX_APP_NUM: usize = 16;

pub(crate) const APP_BASE_ADDRESS: usize = 0x8040_0000;
pub(crate) const APP_SIZE_LIMIT: usize = 0x2_0000;

pub(crate) const KERNEL_STACK_SIZE: usize = 4096 * 2; // 8KiB
pub(crate) const USER_STACK_SIZE: usize = 4096 * 2; // 8KiB