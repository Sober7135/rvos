use crate::trap::trap_return;

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub(super) struct TaskContext {
    ra: usize,
    sp: usize,
    s: [usize; 12],
}

impl TaskContext {
    pub(crate) fn goto_trap_return(kstack_top: usize) -> Self {
        Self {
            ra: trap_return as usize,
            sp: kstack_top,
            s: [0; 12],
        }
    }
}
