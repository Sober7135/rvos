use core::fmt::Debug;

use super::address::{PhysicalAddr, PhysicalPageNumber};
use crate::{
    config::{MEMORY_END, PAGE_SIZE},
    sync::Mutex,
};
use alloc::vec::Vec;
use lazy_static::*;
use log::info;

type FrameAllocatorImpl = StackAllocator;

lazy_static! {
    static ref FRAME_ALLOCATOR: Mutex<FrameAllocatorImpl> = {
        extern "C" {
            fn ekernel();
        }
        // compute the first ppn and last ppn
        Mutex::new(FrameAllocatorImpl::new(
            PhysicalAddr::from(ekernel as usize).ceil(),
            PhysicalAddr::from(MEMORY_END).floor(),
        ))
    };
}

pub fn frame_alloc() -> Option<FrameTracker> {
    FRAME_ALLOCATOR.lock().alloca().map(FrameTracker::new)
}

pub fn frame_dealloc(ppn: PhysicalPageNumber) {
    FRAME_ALLOCATOR.lock().dealloca(ppn)
}

pub struct FrameTracker {
    pub ppn: PhysicalPageNumber,
}

impl Debug for FrameTracker {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("FrameTracker: PPN={:#x}", self.ppn.0))
    }
}

impl From<PhysicalPageNumber> for FrameTracker {
    fn from(value: PhysicalPageNumber) -> Self {
        Self { ppn: value }
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        frame_dealloc(self.ppn)
    }
}

impl FrameTracker {
    pub fn new(ppn: PhysicalPageNumber) -> Self {
        let pa = PhysicalAddr::from(ppn).0 as *mut u8;
        unsafe { pa.write_bytes(0, PAGE_SIZE) }

        Self { ppn }
    }
}

pub trait FrameAllocator {
    fn new(start: PhysicalPageNumber, end: PhysicalPageNumber) -> Self;
    fn alloca(&mut self) -> Option<PhysicalPageNumber>;
    fn dealloca(&mut self, ppn: PhysicalPageNumber);
}

struct StackAllocator {
    current: usize,
    end: usize,
    recycled: Vec<usize>,
}

impl FrameAllocator for StackAllocator {
    fn new(start: PhysicalPageNumber, end: PhysicalPageNumber) -> Self {
        Self {
            current: start.into(),
            end: end.into(),
            recycled: Vec::new(),
        }
    }

    fn alloca(&mut self) -> Option<PhysicalPageNumber> {
        if let Some(ppn) = self.recycled.pop() {
            Some(ppn.into())
        } else {
            if self.current == self.end {
                return None;
            }
            let ret = self.current;
            self.current += 1;
            Some(ret.into())
        }
    }

    fn dealloca(&mut self, ppn: PhysicalPageNumber) {
        if ppn.0 >= self.current || self.recycled.iter().any(|&v| v == ppn.0) {
            panic!("Frame ppn={:#x} has not been allocated!", ppn.0);
        }
        self.recycled.push(ppn.into())
    }
}

#[allow(unused)]
pub fn frame_allocator_test() {
    let mut v: Vec<FrameTracker> = Vec::new();
    for i in 0..5 {
        let frame = frame_alloc().unwrap();
        info!("{:?}", frame);
        v.push(frame);
    }
    v.clear();
    for i in 0..5 {
        let frame = frame_alloc().unwrap();
        info!("{:?}", frame);
        v.push(frame);
    }
    drop(v);
    info!("frame_allocator_test PASSED!");
}
