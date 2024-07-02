use alloc::sync::Arc;
use lazy_static::*;
use log::info;

use super::{MapPermission, MemorySet, VirtualAddr};
use crate::{config::kernel_stack_position, sync::Mutex};

lazy_static! {
    static ref KERNEL_SPACE: Arc<Mutex<MemorySet>> = Arc::new(Mutex::new(MemorySet::new_kernel()));
}

pub fn init() {
    KERNEL_SPACE.lock().activate();
}

pub fn kstack_alloc(pid: usize) -> (usize, usize) {
    let (kstack_bottom, kstack_top) = kernel_stack_position(pid);
    KERNEL_SPACE.lock().insert_framed_area(
        kstack_bottom.into(),
        kstack_top.into(),
        MapPermission::R | MapPermission::W,
    );
    (kstack_bottom, kstack_top)
}

pub fn kstack_dealloc(pid: usize) {
    let (kstack_bottom, _) = kernel_stack_position(pid);
    let kstack_bottom_va: VirtualAddr = kstack_bottom.into();
    KERNEL_SPACE
        .lock()
        .remove_area_with_start_vpn(kstack_bottom_va.into());
}

pub fn get_kernel_token() -> usize {
    KERNEL_SPACE.lock().get_token()
}

#[allow(unused)]
pub fn remap_test() {
    extern "C" {
        fn stext();
        fn etext();
        fn srodata();
        fn erodata();
        fn sdata();
        fn edata();
        fn sbss(); // start of block start symbol
        fn ebss(); // end of block start symbol
        fn ekernel();
        fn strampoline();
    }

    let mut kernel_space = KERNEL_SPACE.lock();
    let mtext: VirtualAddr = (((stext as usize) + (etext as usize)) / 2).into();
    let mrodata: VirtualAddr = (((srodata as usize) + (erodata as usize)) / 2).into();
    let mdata: VirtualAddr = (((sdata as usize) + (edata as usize)) / 2).into();
    let mbss: VirtualAddr = (((sbss as usize) + (ebss as usize)) / 2).into();

    // text
    let mut pte = kernel_space.page_table.translate(mtext.into()).unwrap();
    assert!(pte.readable());
    assert!(!pte.writable());
    assert!(pte.executable());

    // rodata
    pte = kernel_space.page_table.translate(mrodata.into()).unwrap();
    assert!(pte.readable());
    assert!(!pte.writable());
    assert!(!pte.executable());

    // data
    pte = kernel_space.page_table.translate(mdata.into()).unwrap();
    assert!(pte.readable());
    assert!(pte.writable());
    assert!(!pte.executable());

    // bss
    pte = kernel_space.page_table.translate(mbss.into()).unwrap();
    assert!(pte.readable());
    assert!(pte.writable());
    assert!(!pte.executable());

    info!("remap_test PASSED");
}
