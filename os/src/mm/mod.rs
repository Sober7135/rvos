mod address;
mod frame_allocator;
mod heap_allocator;
pub mod kernel_space;
mod memory_set;
mod page_table;

pub use address::*;
pub use memory_set::{MapPermission, MemorySet};
pub use page_table::copy_from_user;

#[allow(unused)]
pub fn test() {
    // heap_test();
    // frame_allocator_test();
    // translate_test();
    kernel_space::remap_test();
}

pub fn init() {
    heap_allocator::init_heap();
    kernel_space::init();
}
