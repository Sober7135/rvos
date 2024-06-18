use crate::config::HEAP_ORDER_SIZE;
use crate::config::KERNEL_HEAP_SIZE;
use buddy_system_allocator::LockedHeap;
use log::info;

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap<HEAP_ORDER_SIZE> = LockedHeap::new();

static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0u8; KERNEL_HEAP_SIZE];

pub fn init_heap() {
    info!("in init_heap");
    unsafe {
        HEAP_ALLOCATOR
            .lock()
            .init(HEAP_SPACE.as_ptr() as usize, KERNEL_HEAP_SIZE)
    }
}

#[alloc_error_handler]
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout)
}

#[allow(unused)]
pub fn heap_test() {
    use alloc::boxed::Box;
    use alloc::vec;
    extern "C" {
        fn sbss();
        fn ebss();
    }

    let bss_range = sbss as usize..ebss as usize;
    let mut v = vec![];
    for i in 0..1145 {
        v.push(i);
    }
    for (i, item) in v.iter().enumerate() {
        assert_eq!(*item, i);
    }
    assert!(bss_range.contains(&(v.as_ptr() as usize)));

    let b = Box::new(114514);
    assert_eq!(*b, 114514);
    assert!(bss_range.contains(&(b.as_ref() as *const _ as usize)));

    info!("[kernel] HEAP TEST PASSED!");
}
