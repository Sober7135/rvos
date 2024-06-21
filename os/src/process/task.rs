use alloc::{
    sync::{Arc, Weak},
    vec::Vec,
};

use crate::{loader::get_app_data_by_name, sync::Mutex, trap::TrapContext};

use super::{
    context::TaskContext,
    kernel_space::{kstack_alloc, kstack_dealloc},
    kernel_stack_position,
    manager::add_task,
    pid::{pid_alloc, PidHandle},
    state::TaskState,
    translate_str, MemorySet, PhysicalPageNumber, VirtualAddr, TRAP_CONTEXT,
};

#[derive(Debug)]
pub struct TaskControlBlock {
    // immutable
    pub pid: PidHandle,

    // mutable
    pub inner: Mutex<TaskControlBlockInner>,
}

#[derive(Debug)]
pub struct TaskControlBlockInner {
    pub state: TaskState,
    context: TaskContext,
    pub memory_set: MemorySet,
    trap_context_ppn: PhysicalPageNumber,
    // Application data can only appear in the region where the application address space is less than base_size bytes. With it, we can clearly know how much data of the application resides in memory.
    base_size: usize,

    pub parent: Option<Weak<TaskControlBlock>>,
    pub children: Vec<Arc<TaskControlBlock>>,
    pub exit_code: i32,
}

impl TaskControlBlock {
    pub fn from_elf(elf_data: &[u8]) -> Self {
        let pid = pid_alloc();
        let (_, kstack_top) = kstack_alloc(pid.0);

        let status = TaskState::Runnable;
        let task_cx = TaskContext::goto_trap_return(kstack_top);
        let (mm_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);

        let trap_context_ppn = mm_set
            .translate(VirtualAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .get_ppn();

        let tcb = Self {
            pid,
            inner: Mutex::new(TaskControlBlockInner {
                state: status,
                context: task_cx,
                memory_set: mm_set,
                trap_context_ppn,
                base_size: user_sp,
                parent: None,
                children: Vec::new(),
                exit_code: 0,
            }),
        };

        // TODO get_trap_context use lock, we can not use it. TO BE OPTIMAZIED.
        let trap_cx = tcb.get_trap_context();
        *trap_cx = TrapContext::app_init_context(entry_point, kstack_top, user_sp);

        tcb
    }

    pub fn get_trap_context(&self) -> &'static mut TrapContext {
        self.inner.lock().get_trap_context()
    }

    pub fn get_user_token(&self) -> usize {
        self.inner.lock().get_user_token()
    }

    pub fn get_state(&self) -> TaskState {
        self.inner.lock().state
    }

    pub fn get_pid(&self) -> usize {
        self.pid.0
    }

    // TODO
    pub fn exec(&self, path: *const u8) -> isize {
        // debug!("");
        let mut inner = self.inner.lock();
        let path = translate_str(inner.get_user_token(), path);

        if let Some(data) = get_app_data_by_name(&path) {
            let (mm_set, user_sp, entry_point) = MemorySet::from_elf(data);
            let trap_context_ppn = mm_set
                .translate(VirtualAddr::from(TRAP_CONTEXT).into())
                .unwrap()
                .get_ppn();

            inner.memory_set = mm_set;
            inner.trap_context_ppn = trap_context_ppn;
            inner.base_size = user_sp;
            *inner.get_trap_context() = TrapContext::app_init_context(
                entry_point,
                kernel_stack_position(self.pid.0).1,
                user_sp,
            );

            // debug!("");
            0
        } else {
            -1
        }
    }

    // TODO lazy
    pub fn fork(self: &Arc<Self>) -> Arc<Self> {
        let pid = pid_alloc();
        let (_, kstack_top) = kstack_alloc(pid.0);

        let mut parent_inner = self.inner.lock();

        let mm_set = MemorySet::from_other_proc(&parent_inner.memory_set);
        let trap_context_ppn = mm_set
            .translate(VirtualAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .get_ppn();

        let inner = TaskControlBlockInner {
            state: TaskState::Runnable,
            context: TaskContext::goto_trap_return(kstack_top), // TODO
            memory_set: mm_set,
            trap_context_ppn,
            base_size: parent_inner.base_size,
            parent: Some(Arc::downgrade(self)),
            children: Vec::new(),
            exit_code: 0,
        };

        inner.get_trap_context().kernel_sp = kstack_top;

        let child = Arc::new(Self {
            pid,
            inner: Mutex::new(inner),
        });
        add_task(child.clone());

        parent_inner.children.push(child.clone());
        child
    }
}

impl Drop for TaskControlBlock {
    fn drop(&mut self) {
        kstack_dealloc(self.pid.0);
    }
}

impl Drop for TaskControlBlockInner {
    fn drop(&mut self) {
        self.memory_set.recycle_data_pages();
    }
}

impl TaskControlBlockInner {
    pub fn get_trap_context(&self) -> &'static mut TrapContext {
        self.trap_context_ppn.get_mut()
    }

    pub fn get_user_token(&self) -> usize {
        self.memory_set.get_token()
    }

    pub fn get_task_context_ptr(&self) -> *const TaskContext {
        &self.context as *const TaskContext
    }

    pub fn get_task_context_ptr_mut(&mut self) -> *mut TaskContext {
        &mut self.context as *mut TaskContext
    }
}
