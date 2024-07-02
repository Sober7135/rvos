use alloc::vec::Vec;
use lazy_static::*;

use crate::sync::Mutex;

lazy_static! {
    static ref PID_ALLOCATOR: Mutex<PidAllocator> = Mutex::new(PidAllocator::new());
}

#[derive(Debug)]
pub struct PidHandle(pub usize);

#[derive(Debug)]
struct PidAllocator {
    current: usize,
    recycled: Vec<usize>,
}

impl Drop for PidHandle {
    fn drop(&mut self) {
        pid_dealloc(self.0)
    }
}

impl PidAllocator {
    pub fn new() -> Self {
        Self {
            current: 0,
            recycled: vec![],
        }
    }

    pub fn alloc(&mut self) -> PidHandle {
        if let Some(pid) = self.recycled.pop() {
            PidHandle(pid)
        } else {
            let ret = PidHandle(self.current);
            self.current += 1;
            ret
        }
    }

    pub fn dealloc(&mut self, pid: usize) {
        self.recycled.push(pid);
    }
}

pub fn pid_alloc() -> PidHandle {
    PID_ALLOCATOR.lock().alloc()
}

pub fn pid_dealloc(pid: usize) {
    PID_ALLOCATOR.lock().dealloc(pid)
}
