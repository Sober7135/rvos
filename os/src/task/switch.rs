use super::*;

global_asm!(include_str!("switch.S"));

extern "C" {
    pub(crate) fn __switch(
        current_task_cx_ptr: *mut TaskContext,
        next_task_cx_ptr: *const TaskContext,
    );
}
