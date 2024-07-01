use crate::config::*;
use crate::loader::{get_app_data, get_num_apps};
use crate::mm::*;
use crate::sbi::shutdown;
use crate::sync::UpSafeCell;
use crate::trap::TrapContext;
use alloc::vec::Vec;
use core::arch::global_asm;
use lazy_static::lazy_static;

mod context;
mod status;
mod switch;

use context::TaskContext;
use log::info;

use self::status::TaskStatus;
use self::switch::__switch;

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct TaskControlBlock {
    status: TaskStatus,
    context: TaskContext,
    memory_set: MemorySet,
    trap_cx_ppn: PhysicalPageNumber,
    // TODO what is this
    base_size: usize,
}

impl TaskControlBlock {
    pub(crate) fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }

    pub(crate) fn new(elf_data: &[u8], app_id: usize) -> Self {
        let status = TaskStatus::Ready;
        let (kstack_bottom, kstack_top) = kernel_stack_position(app_id);
        let task_cx = TaskContext::goto_trap_return(kstack_top);
        let (mm_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);

        let trap_cx_ppn = mm_set
            .translate(VirtualAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .get_ppn();

        KERNEL_SPACE.exclusive_access().insert_framed_area(
            kstack_bottom.into(),
            kstack_top.into(),
            MapPermission::R | MapPermission::W,
        );

        let tcb = Self {
            status,
            context: task_cx,
            memory_set: mm_set,
            trap_cx_ppn,
            base_size: user_sp,
        };

        let trap_cx = tcb.get_trap_cx();
        *trap_cx = TrapContext::app_init_context(entry_point, kstack_top, user_sp);

        tcb
    }

    pub(crate) fn get_user_token(&self) -> usize {
        self.memory_set.get_token()
    }
}

pub(crate) struct TaskManager {
    num_apps: usize,
    inner: UpSafeCell<TaskManagerInner>,
}

#[derive(Debug)]
struct TaskManagerInner {
    tasks: Vec<TaskControlBlock>,
    current_task: usize,
}

lazy_static! {
    pub(crate) static ref TASK_MANAGER: TaskManager = {
        let num_apps = get_num_apps();
        let mut tasks = Vec::new();
        let apps_num = get_num_apps();
        for i in 0..apps_num {
            tasks.push(TaskControlBlock::new(get_app_data(i), i));
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

pub(crate) fn get_current_user_token() -> usize {
    TASK_MANAGER.get_current_user_token()
}

pub(crate) fn get_current_trap_cx() -> &'static mut TrapContext {
    TASK_MANAGER.get_current_trap_cx()
}

impl TaskManager {
    fn run_first_app(&self) {
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
    }

    fn mark_current_suspend(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].status = TaskStatus::Ready;
    }

    fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.exclusive_access();
            let current = inner.current_task;
            let current_task_cx_ptr = &mut inner.tasks[current].context as *mut TaskContext;
            let next_task_context_ptr = &inner.tasks[next].context as *const TaskContext;
            inner.tasks[next].status = TaskStatus::Running;
            inner.current_task = next;

            drop(inner);

            unsafe { __switch(current_task_cx_ptr, next_task_context_ptr) }
        } else {
            info!("All applications are done!");
            info!("shutdown...");
            shutdown(false);
        }
    }

    fn get_current_user_token(&self) -> usize {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].get_user_token()
    }

    fn get_current_trap_cx(&self) -> &'static mut TrapContext {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].get_trap_cx()
    }
}
