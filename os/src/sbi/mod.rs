mod binary;
mod legacy;
mod sbi;
mod srst;

use core::arch::asm;
pub use sbi::{console_putchar, shutdown};
