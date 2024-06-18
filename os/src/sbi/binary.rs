#![allow(unused)]

use super::*;

pub struct Eid;

impl Eid {
    pub const SRST: usize = 0x53525354;
    pub const TIME: usize = 0x54494D45;
}

pub struct SbiRet {
    error: usize,
    value: usize,
}

#[inline(always)]
pub fn sbi_call_0(eid: usize, fid: usize) -> SbiRet {
    let (error, value);
    unsafe {
        asm!(
            "ecall",
            in("a6") fid, in("a7") eid,
            lateout("a0") error,
            lateout("a1") value,
        )
    }
    SbiRet { error, value }
}

#[inline(always)]
pub fn sbi_call_1(eid: usize, fid: usize, arg0: usize) -> SbiRet {
    let (error, value);
    unsafe {
        asm!(
            "ecall",
            in("a6") fid, in("a7") eid,
            inlateout("a0") arg0 => error,
            lateout("a1") value,
        )
    }
    SbiRet { error, value }
}

#[inline(always)]
pub fn sbi_call_2(eid: usize, fid: usize, arg0: usize, arg1: usize) -> SbiRet {
    let (error, value);
    unsafe {
        asm!(
            "ecall",
            in("a6") fid,
            in("a7") eid,
            inlateout("a0") arg0 => error,
            inlateout("a1") arg1 => value,
        )
    }
    SbiRet { error, value }
}

#[inline(always)]
pub fn sbi_call_3(eid: usize, fid: usize, arg0: usize, arg1: usize, arg2: usize) -> SbiRet {
    let (error, value);
    unsafe {
        asm!(
            "ecall",
            in("a6") fid,
            in("a7") eid,
            inlateout("a0") arg0 => error,
            inlateout("a1") arg1 => value,
            in("a2") arg2,
        )
    }
    SbiRet { error, value }
}

#[inline(always)]
pub fn sbi_call_4(
    eid: usize,
    fid: usize,
    arg0: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
) -> SbiRet {
    let (error, value);
    unsafe {
        asm!(
            "ecall",
            in("a6") fid,
            in("a7") eid,
            inlateout("a0") arg0 => error,
            inlateout("a1") arg1 => value,
            in("a2") arg2,
            in("a3") arg3,
        )
    }
    SbiRet { error, value }
}
