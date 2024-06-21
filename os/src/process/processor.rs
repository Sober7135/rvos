use crate::{sync::Mutex, trap::TrapContext};

use super::{
    context::TaskContext,
    manager::{add_task, fetch_task},
    state::TaskState,
    switch::__switch,
    TaskControlBlock,
};

use alloc::sync::Arc;
use lazy_static::*;

lazy_static! {
    static ref PROCESSOR: Mutex<Processor> = Mutex::new(Processor::new());
}

pub struct Processor {
    current: Option<Arc<TaskControlBlock>>,
}

impl Processor {
    fn new() -> Self {
        Self { current: None }
    }

    #[allow(unused)]
    fn take_current(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.current.take()
    }

    fn current(&self) -> Option<Arc<TaskControlBlock>> {
        self.current.as_ref().map(|value| value.clone())
    }

    fn replace(&mut self, next: Arc<TaskControlBlock>) -> Option<Arc<TaskControlBlock>> {
        self.current.replace(next)
    }
}

#[allow(unused)]
pub fn take_current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.lock().take_current()
}

#[allow(unused)]
pub fn replace(next: Arc<TaskControlBlock>) -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.lock().replace(next)
}

pub fn get_current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.lock().current()
}

pub fn get_current_user_token() -> usize {
    get_current_task().unwrap().get_user_token()
}

pub fn get_current_trap_context() -> &'static mut TrapContext {
    get_current_task().unwrap().get_trap_context()
}

pub fn schedule() {
    // fetch
    let mut next = fetch_task();
    while next.is_none() {
        next = fetch_task();
    }
    let next_task = next.unwrap();

    let mut cpu = PROCESSOR.lock();
    let current = cpu.replace(next_task);
    let mut inner = cpu.current.as_ref().unwrap().inner.lock();
    let next = inner.get_task_context_ptr();
    inner.state = TaskState::Running;
    drop(inner);
    drop(cpu);

    let current = if current.is_none() || current.as_ref().unwrap().get_state() == TaskState::Zombie
    {
        &mut TaskContext::default() as *mut TaskContext
    } else {
        let current = current.unwrap();
        // if current is Some, add it to task manager.
        add_task(current.clone()); // TODO we are holding current's lock
        let mut task = current.inner.lock();
        // current will live enough longer, so we use ptr to avoid lifetime check.
        task.get_task_context_ptr_mut()
    };

    unsafe { __switch(current, next as *mut TaskContext) }
}
