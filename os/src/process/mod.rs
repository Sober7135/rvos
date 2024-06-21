mod context;
mod manager;
mod pid;
pub mod processor;
mod state;
mod switch;
mod task;

use alloc::sync::Arc;
use lazy_static::lazy_static;
use manager::add_task;

use crate::config::*;
use crate::loader::get_app_data_by_name;
use crate::mm::*;
use crate::sbi::shutdown;
use context::TaskContext;
use core::arch::global_asm;
use processor::get_current_task;
use state::TaskState;
use task::TaskControlBlock;

lazy_static! {
    static ref INIT_PROC: Arc<TaskControlBlock> = Arc::new(TaskControlBlock::from_elf(
        get_app_data_by_name("init_proc").unwrap()
    ));
}

pub fn add_init_proc() {
    add_task(INIT_PROC.clone())
}

const IDLE_PID: usize = 0;

// just mark, not really exit
pub fn mark_current_exit(exit_code: i32) {
    // after this line, the processor's current task will be None
    let current = get_current_task().unwrap();
    if current.get_pid() == IDLE_PID {
        if exit_code == 0 {
            shutdown(false);
        } else {
            shutdown(true);
        }
    }

    let mut inner = current.inner.lock();
    // set task state to Exited
    inner.state = TaskState::Zombie;
    inner.exit_code = exit_code;

    {
        let mut init = INIT_PROC.inner.lock();
        for child in &inner.children {
            child.inner.lock().parent = Some(Arc::downgrade(&INIT_PROC));
            init.children.push(child.clone());
        }
    }
    // TODO is this necessary
    // clear children
    // inner.children.clear();
    // TODO reclaim memory in advance to fully utilize it.
    // inner.memory_set.recycle_data_pages();
    // drop(inner);
}

pub fn mark_current_suspend() {
    let current = get_current_task().unwrap();

    let mut inner = current.inner.lock();
    inner.state = TaskState::Runnable;
    drop(inner);
    add_task(current);
}
