use crate::config::*;
use crate::trap::TrapContext;
use core::arch::asm;

#[derive(Debug, Clone, Copy)]
#[repr(align(4096))]
struct KernelStack {
    data: [u8; KERNEL_STACK_SIZE],
}

#[derive(Debug, Clone, Copy)]
#[repr(align(4096))]
struct UserStack {
    data: [u8; USER_STACK_SIZE],
}

static KERNEL_STACK: [KernelStack; MAX_APP_NUM] = [KernelStack {
    data: [0; KERNEL_STACK_SIZE],
}; MAX_APP_NUM];

static USER_STACK: [UserStack; MAX_APP_NUM] = [UserStack {
    data: [0; USER_STACK_SIZE],
}; MAX_APP_NUM];

impl KernelStack {
    /// get initial stack pointer
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }

    fn push_context(&self, ctxt: TrapContext) -> usize {
        let ptr = (self.get_sp() - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
        unsafe { *ptr = ctxt };
        ptr as usize
    }
}

impl UserStack {
    /// get initial stack pointer
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }
}

pub(crate) fn get_num_apps() -> usize {
    extern "C" {
        fn _num_apps();
    }
    unsafe { (_num_apps as *const usize).read_volatile() }
}

pub(crate) fn get_base_i(app_id: usize) -> usize {
    APP_BASE_ADDRESS + app_id * APP_SIZE_LIMIT
}

pub(crate) fn load_apps() {
    extern "C" {
        fn _num_apps();
    }
    let num_apps = get_num_apps();
    let num_app_ptr = _num_apps as *const usize;
    let mut apps_start = [0; MAX_APP_NUM + 1];
    // num_apps + 1 to copy the last app's end address
    unsafe { core::ptr::copy(num_app_ptr.add(1), apps_start.as_mut_ptr(), num_apps + 1) }
    // clear all
    unsafe { core::ptr::write_bytes(APP_BASE_ADDRESS as *mut u8, 0, num_apps * APP_SIZE_LIMIT) }
    // clear i-cache
    unsafe { asm!("fence.i") }
    // load apps
    for i in 0..num_apps {
        unsafe {
            // clear app area
            core::ptr::copy(
                apps_start[i] as *const u8,
                get_base_i(i) as *mut u8,
                apps_start[i + 1] - apps_start[i],
            );
        };
    }
}

pub(crate) fn init_app_cx(i: usize) -> usize {
    KERNEL_STACK[i].push_context(TrapContext::app_init_context(
        get_base_i(i),
        USER_STACK[i].get_sp(),
    ))
}
