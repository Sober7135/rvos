mod inode;
mod stdio;

use crate::mm::UserBuffer;
/// File trait
pub trait File: Send + Sync {
    #[allow(unused)]
    /// If readable
    fn readable(&self) -> bool;

    #[allow(unused)]
    /// If writable
    fn writable(&self) -> bool;

    #[allow(unused)]
    /// Read file to `UserBuffer`
    fn read(&self, buf: UserBuffer) -> usize;

    #[allow(unused)]
    /// Write `UserBuffer` to file
    fn write(&self, buf: UserBuffer) -> usize;
}

pub use inode::{list_apps, open_file, OpenFlags};
#[allow(unused_imports)]
pub use stdio::{Stdin, Stdout};
