mod address;
mod frame_allocator;
mod heap_allocator;
pub mod kernel_space;
mod memory_set;
mod page_table;
mod user_buffer;

pub use address::*;
pub use frame_allocator::{frame_alloc, frame_dealloc, FrameTracker};
pub use memory_set::{MapPermission, MemorySet};
#[allow(unused_imports)]
pub use page_table::{
    transfer_byte_buffer, translate_ref, translate_refmut, translate_str, PageTable,
};
pub use user_buffer::UserBuffer;

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
