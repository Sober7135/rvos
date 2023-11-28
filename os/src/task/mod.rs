use crate::config::*;
use crate::loader::{get_num_apps, init_app_cx};
use crate::sbi::shutdown;
use crate::sync::UpSafeCell;
use core::arch::global_asm;
use lazy_static::lazy_static;

mod context;
mod status;
mod switch;

use context::TaskContext;
use log::{debug, info};

use self::status::TaskStatus;
use self::switch::__switch;

#[derive(Debug, Clone, Copy)]
pub(crate) struct TaskControlBlock {
    status: TaskStatus,
    context: TaskContext,
}

impl TaskControlBlock {
    pub(crate) fn zero_init() -> Self {
        Self {
            status: TaskStatus::init(),
            context: TaskContext::zero_init(),
        }
    }
}

pub(crate) struct TaskManager {
    num_apps: usize,
    inner: UpSafeCell<TaskManagerInner>,
}

#[derive(Debug)]
struct TaskManagerInner {
    tasks: [TaskControlBlock; MAX_APP_NUM],
    current_task: usize,
}

lazy_static! {
    pub(crate) static ref TASK_MANAGER: TaskManager = {
        let num_apps = get_num_apps();
        let mut tasks = [TaskControlBlock::zero_init(); MAX_APP_NUM];
        for (i, task) in tasks.iter_mut().enumerate().take(num_apps) {
            task.status = status::TaskStatus::Ready;
            task.context.init(init_app_cx(i));
        }

        TaskManager {
            num_apps,
            inner: unsafe {
                UpSafeCell::new(TaskManagerInner {
                    tasks,
                    current_task: 0,
                })
            },
        }
    };
}

pub(crate) fn run_first_task() {
    TASK_MANAGER.run_first_app();
}

pub(crate) fn run_next_task() {
    TASK_MANAGER.run_next_task();
}

pub(crate) fn mark_current_exited() {
    TASK_MANAGER.mark_current_exited();
}

pub(crate) fn mark_current_suspend() {
    TASK_MANAGER.mark_current_suspend();
}

impl TaskManager {
    fn run_first_app(&self) {
        info!("[kernel] Starting running app_{}", 0);
        let mut inner = self.inner.exclusive_access();
        let task = &mut inner.tasks[0];
        task.status = TaskStatus::Running;
        let next_task_cx_ptr = &task.context as *const TaskContext;
        let mut _fake = TaskContext::default();
        drop(inner);

        unsafe { __switch(&mut _fake as *mut TaskContext, next_task_cx_ptr) }
    }

    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        (current + 1..current + self.num_apps + 1)
            .map(|index| index % self.num_apps)
            .find(|index| inner.tasks[*index].status == TaskStatus::Ready)
    }

    fn mark_current_exited(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].status = TaskStatus::Exited;
        debug!(
            "[kernel][mark_current_exit] current={}\n{:#x}",
            current, inner.tasks[current].context.ra
        )
    }

    fn mark_current_suspend(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].status = TaskStatus::Ready;
        debug!(
            "[kernel][mark_current_suspend] current={}\nra={:#x}",
            current, inner.tasks[current].context.ra
        )
    }

    fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            info!("[kernel] Starting running app_{}", next);
            let mut inner = self.inner.exclusive_access();
            let current = inner.current_task;
            let current_task_cx_ptr = &mut inner.tasks[current].context as *mut TaskContext;
            let next_task_context_ptr = &inner.tasks[next].context as *const TaskContext;
            inner.tasks[next].status = TaskStatus::Running;
            inner.current_task = next;

            debug!(
                "[kernel][run_next_task] current={}\nra={:#x}",
                current, inner.tasks[current].context.ra
            );
            debug!(
                "[kernel][run_next_task] next={}\nra={:#x}",
                current, inner.tasks[current].context.ra
            );
            drop(inner);

            unsafe { __switch(current_task_cx_ptr, next_task_context_ptr) }
        } else {
            info!("All applications are done!");
            info!("shutdown...");
            shutdown(false);
        }
    }
}
