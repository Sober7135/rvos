mod address;
mod frame_allocator;
mod heap_allocator;
mod page_table;

use self::{
    frame_allocator::frame_allocator_test, heap_allocator::heap_test, page_table::translate_test,
};

pub(crate) use heap_allocator::init_heap;

pub(crate) fn test() {
    heap_test();
    frame_allocator_test();
    translate_test();
}
