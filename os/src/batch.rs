use crate::{println, sbi::shutdown, sync::UpSafeCell, trap::TrapContext};
use core::arch::asm;
use lazy_static::lazy_static;
use log::{info, trace};

const MAX_APP_NUM: usize = 16;
const APP_BASE_ADDRESS: usize = 0x8040_0000;
const APP_SIZE_LIMIT: usize = 0x2_0000;

const KERNEL_STACK_SIZE: usize = 4096 * 2; // 8KiB
const USER_STACK_SIZE: usize = 4096 * 2; // 8KiB

#[repr(align(4096))]
struct KernelStack {
    data: [u8; KERNEL_STACK_SIZE],
}

#[repr(align(4096))]
struct UserStack {
    data: [u8; USER_STACK_SIZE],
}

static KERNEL_STACK: KernelStack = KernelStack {
    data: [0; KERNEL_STACK_SIZE],
};
static USER_STACK: UserStack = UserStack {
    data: [0; USER_STACK_SIZE],
};

impl KernelStack {
    /// get initial stack pointer
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }

    fn push_context(&self, ctxt: TrapContext) -> &'static mut TrapContext {
        let ptr = (KERNEL_STACK.get_sp() - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
        unsafe { *ptr = ctxt };
        unsafe { ptr.as_mut().unwrap() }
    }
}

impl UserStack {
    /// get initial stack pointer
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }
}

pub(crate) struct AppManager {
    num_app: usize,
    current_app: usize,
    apps_start: [usize; MAX_APP_NUM + 1], // MAX_APP_NUM + 1 to copy the last app's end address
}

impl AppManager {
    pub(crate) fn print_app_info(&self) {
        trace!("[kernel] num_app = {}", self.num_app);
        for i in 0..self.num_app {
            trace!(
                "[kernel] app_{} [{:#x},{:#x})",
                i,
                self.apps_start[i],
                self.apps_start[i + 1]
            );
        }
    }

    pub(crate) fn load_app(&self, app_id: usize) {
        if app_id >= self.num_app {
            info!("[kernel] All programs completed");
            shutdown(false);
        }

        info!("[kernel] load and then run app_{}", app_id);
        unsafe {
            // clear app area
            core::ptr::write_bytes(APP_BASE_ADDRESS as *mut u8, 0, APP_SIZE_LIMIT);

            core::ptr::copy(
                self.apps_start[app_id] as *const u8,
                APP_BASE_ADDRESS as *mut u8,
                self.apps_start[app_id + 1] - self.apps_start[app_id],
            );

            asm!("fence.i")
        };
    }

    pub(crate) fn move_to_next(&mut self) {
        self.current_app += 1;
    }
}

lazy_static! {
    static ref APP_MANANGER: UpSafeCell<AppManager> = unsafe {
        UpSafeCell::new({
            extern "C" {
                fn _num_app();
            }
            let num_app_ptr = _num_app as usize as *const usize;
            let num_app = num_app_ptr.read_volatile() ;
            let mut apps_start = [0; MAX_APP_NUM + 1];
            // num_app + 1 to copy the last app's end address
            core::ptr::copy(num_app_ptr.add(1), apps_start.as_mut_ptr(), num_app + 1);
            AppManager {
                num_app,
                current_app: 0,
                apps_start,
            }
        })
    };
}

pub(crate) fn init() {
    APP_MANANGER.exclusive_access().print_app_info();
}

pub(crate) fn run_next_app() -> ! {
    let mut app_manager = APP_MANANGER.exclusive_access();
    let current_app = app_manager.current_app;
    app_manager.load_app(current_app);
    app_manager.move_to_next();
    drop(app_manager);

    // we need to store kernel context to kernel stack
    extern "C" {
        fn __restore(addr: usize);
    }
    unsafe {
        __restore(KERNEL_STACK.push_context(TrapContext::app_init_context(
            APP_BASE_ADDRESS,
            USER_STACK.get_sp(),
        )) as *mut TrapContext as usize)
    }
    unreachable!()
}
