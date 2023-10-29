// legacy.rs
#![allow(unused)]
use super::*;

pub(crate) struct EID;
impl EID {
    pub(crate) const CONSOLE_PUTCHAR: usize = 0x01;
}

#[inline(always)]
pub(crate) fn sbi_call_0(eid: usize, fid: usize) -> usize {
    let error;
    unsafe {
        asm!(
            "ecall",
            in("a7") eid,
            lateout("a0") error,
        )
    }
    error
}

#[inline(always)]
pub(crate) fn sbi_call_1(eid: usize, arg0: usize) -> usize {
    let error;
    unsafe {
        asm!(
            "ecall",
            in("a7") eid,
            inlateout("a0") arg0 => error,
        )
    }
    error
}

#[inline(always)]
pub(crate) fn sbi_call_2(eid: usize, arg0: usize, arg1: usize) -> usize {
    let error;
    unsafe {
        asm!(
            "ecall",
            in("a7") eid,
            inlateout("a0") arg0 => error,
            in("a1") arg1,
        )
    }
    error
}

#[inline(always)]
pub(crate) fn sbi_call_3(eid: usize, arg0: usize, arg1: usize, arg2: usize) -> usize {
    let error;
    unsafe {
        asm!(
            "ecall",
            in("a7") eid,
            inlateout("a0") arg0 => error,
            in("a1") arg1,
            in("a2") arg2,
        )
    }
    error
}
