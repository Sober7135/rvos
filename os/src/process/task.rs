use alloc::{
    sync::{Arc, Weak},
    vec::Vec,
};

use crate::{
    fs::{File, Stdin, Stdout},
    process::{mark_current_suspend, processor::schedule},
    sync::Mutex,
    trap::TrapContext,
};

use super::{
    context::TaskContext,
    kernel_space::{kstack_alloc, kstack_dealloc},
    kernel_stack_position,
    manager::add_task,
    pid::{pid_alloc, PidHandle},
    state::TaskState,
    translate_refmut, translate_str, MemorySet, PhysicalPageNumber, VirtualAddr, TRAP_CONTEXT,
};

pub struct TaskControlBlock {
    // immutable
    pub pid: PidHandle,

    // mutable
    pub inner: Mutex<TaskControlBlockInner>,
}

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
    pub fd_table: Vec<Option<Arc<dyn File + Send + Sync>>>,
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
                fd_table: vec![
                    // 0 -> stdin
                    Some(Arc::new(Stdin)),
                    // 1 -> stdout
                    Some(Arc::new(Stdout)),
                    // 2 -> stderr
                    Some(Arc::new(Stdout)),
                ],
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

    pub fn is_zombie(&self) -> bool {
        self.inner.lock().state == TaskState::Zombie
    }

    pub fn get_pid(&self) -> usize {
        self.pid.0
    }

    // TODO
    pub fn exec(&self, data: &[u8]) -> isize {
        // debug!("");
        let mut inner = self.inner.lock();

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

        0
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

        let new_fd_table = parent_inner
            .fd_table
            .iter()
            .map(|file| file.clone())
            .collect();

        let inner = TaskControlBlockInner {
            state: TaskState::Runnable,
            context: TaskContext::goto_trap_return(kstack_top), // TODO
            memory_set: mm_set,
            trap_context_ppn,
            base_size: parent_inner.base_size,
            parent: Some(Arc::downgrade(self)),
            children: Vec::new(),
            exit_code: 0,
            fd_table: new_fd_table,
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

    // waitpid
    pub fn waitpid(&self, pid: isize, exit_code_ptr: *mut i32) -> isize {
        let mut inner = self.inner.lock();
        // if there is no child, return -2
        if inner.children.is_empty() {
            return -2;
        }
        drop(inner);
        let idx;
        loop {
            inner = self.inner.lock();
            if let Some((index, _)) = inner
                .children
                .iter()
                .enumerate()
                .find(|(_, p)| (pid == -1 || (p.get_pid() == pid as usize)) && p.is_zombie())
            {
                idx = index;
                break;
            } else {
                drop(inner);
                mark_current_suspend();
                schedule();
            }
        }
        // Found
        let child = inner.children.remove(idx);
        let child_inner = child.inner.lock();
        assert_eq!(Arc::strong_count(&child), 1);
        if !exit_code_ptr.is_null() {
            *translate_refmut(inner.get_user_token(), exit_code_ptr) = child_inner.exit_code;
        }
        child.get_pid() as isize
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

    pub fn alloc_fd(&mut self) -> usize {
        if let Some(fd) = (0..self.fd_table.len()).find(|fd| self.fd_table[*fd].is_none()) {
            fd
        } else {
            self.fd_table.push(None);
            self.fd_table.len() - 1
        }
    }
}
