#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub(super) struct TaskContext {
    pub ra: usize,
    pub sp: usize,
    s: [usize; 12],
}

impl TaskContext {
    pub(crate) fn zero_init() -> Self {
        Self::default()
    }

    pub(crate) fn init(&mut self, kstack_ptr: usize) {
        extern "C" {
            fn __restore();
        }

        // So when the __switch ret, the pc point to __restore, so we get back to user space
        self.ra = __restore as usize;
        self.sp = kstack_ptr;
    }
}
