use core::borrow::Borrow;

use crate::sync::Mutex;
use alloc::{collections::VecDeque, sync::Arc};

use super::task::TaskControlBlock;
use lazy_static::lazy_static;

lazy_static! {
    static ref TASK_MANAGER: Mutex<TaskManager> = Mutex::new(TaskManager::new());
}

// FCFS, first come first served
#[derive(Debug)]
pub struct TaskManager {
    runnable_queue: VecDeque<Arc<TaskControlBlock>>,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            runnable_queue: VecDeque::new(),
        }
    }

    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.runnable_queue.push_back(task);
    }

    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.runnable_queue.pop_front()
    }

    pub fn mmap(&mut self, start: usize, len: usize, port: usize) -> Result<(), ()> {
        if let Some(n) = self.runnable_queue.pop_front() {
            n.inner.lock().memory_set.mmap(start, len, port)
        } else {
            Err(())
        }
    }

    /// Unmap a area for the current 'Running' task's program
    pub fn munmap(&mut self, start: usize, len: usize) -> Result<(), ()> {
        if let Some(n) = self.runnable_queue.pop_front() {
            n.inner.lock().memory_set.munmap(start, len)
        } else {
            Err(())
        }
    }
}

pub fn add_task(task: Arc<TaskControlBlock>) {
    TASK_MANAGER.lock().add(task)
}

pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    TASK_MANAGER.lock().fetch()
}

pub fn mmap(start: usize, len: usize, port: usize) -> Result<(), ()> {
    TASK_MANAGER.lock().mmap(start, len, port)
}

/// Unmap a area for the current 'Running' task's program
pub fn munmap(start: usize, len: usize) -> Result<(), ()> {
    TASK_MANAGER.lock().munmap(start, len)
}
