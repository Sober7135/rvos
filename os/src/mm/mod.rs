mod address;
mod frame_allocator;
mod heap_allocator;
mod memory_set;
mod page_table;

pub(crate) use address::*;
use memory_set::remap_test;
pub(crate) use memory_set::{MapPermission, MemorySet, KERNEL_SPACE};
pub(crate) use page_table::copy_from_user;

#[allow(unused)]
pub(crate) fn test() {
    // heap_test();
    // frame_allocator_test();
    // translate_test();
    remap_test();
}

pub(crate) fn init() {
    heap_allocator::init_heap();
    KERNEL_SPACE.lock().activate();
}
